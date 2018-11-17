// Copyright 2015-2018 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

//! Utilities and helpers for transaction dispatch.

use std::fmt::Debug;
use std::ops::Deref;
use std::sync::Arc;

use light::cache::Cache as LightDataCache;
use light::client::LightChainClient;
use light::on_demand::{request, OnDemand};
use light::TransactionQueue as LightTransactionQueue;
use hash::keccak;
use ethereum_types::{H256, H520, Address, U256};
use bytes::Bytes;
use parking_lot::{Mutex, RwLock};
use stats::Corpus;

use ethkey::{Password, Signature};
use sync::LightSync;
use ethcore::ids::BlockId;
use ethcore::client::BlockChainClient;
use ethcore::miner::{self, MinerService};
use ethcore::account_provider::AccountProvider;
use crypto::DEFAULT_MAC;
use transaction::{Action, SignedTransaction, PendingTransaction, Transaction};

use jsonrpc_core::{BoxFuture, Result, Error};
use jsonrpc_core::futures::{future, Future, Poll, Async};
use jsonrpc_core::futures::future::Either;
use v1::helpers::{errors, nonce, TransactionRequest, FilledTransactionRequest, ConfirmationPayload};
use v1::types::{
	H256 as RpcH256, H520 as RpcH520, Bytes as RpcBytes,
	RichRawTransaction as RpcRichRawTransaction,
	ConfirmationPayload as RpcConfirmationPayload,
	ConfirmationResponse,
	SignRequest as RpcSignRequest,
	DecryptRequest as RpcDecryptRequest,
};
use rlp;

pub use self::nonce::Reservations;

/// Has the capability to dispatch, sign, and decrypt.
///
/// Requires a clone implementation, with the implication that it be cheap;
/// usually just bumping a reference count or two.
pub trait Dispatcher: Send + Sync + Clone {
	// TODO: when ATC exist, use zero-cost
	// type Out<T>: IntoFuture<T, Error>

	/// Fill optional fields of a transaction request, fetching gas price but not nonce.
	fn fill_optional_fields(&self, request: TransactionRequest, default_sender: Address, force_nonce: bool)
		-> BoxFuture<FilledTransactionRequest>;

	/// Sign the given transaction request without dispatching, fetching appropriate nonce.
	fn sign(&self, accounts: Arc<AccountProvider>, filled: FilledTransactionRequest, password: SignWith)
		-> BoxFuture<WithToken<SignedTransaction>>;

	/// Converts a `SignedTransaction` into `RichRawTransaction`
	fn enrich(&self, SignedTransaction) -> RpcRichRawTransaction;

	/// "Dispatch" a local transaction.
	fn dispatch_transaction(&self, signed_transaction: PendingTransaction)
		-> Result<H256>;
}

/// A dispatcher which uses references to a client and miner in order to sign
/// requests locally.
#[derive(Debug)]
pub struct FullDispatcher<C, M> {
	client: Arc<C>,
	miner: Arc<M>,
	nonces: Arc<Mutex<nonce::Reservations>>,
	gas_price_percentile: usize,
}

impl<C, M> FullDispatcher<C, M> {
	/// Create a `FullDispatcher` from Arc references to a client and miner.
	pub fn new(
		client: Arc<C>,
		miner: Arc<M>,
		nonces: Arc<Mutex<nonce::Reservations>>,
		gas_price_percentile: usize,
	) -> Self {
		FullDispatcher {
			client,
			miner,
			nonces,
			gas_price_percentile,
		}
	}
}

impl<C, M> Clone for FullDispatcher<C, M> {
	fn clone(&self) -> Self {
		FullDispatcher {
			client: self.client.clone(),
			miner: self.miner.clone(),
			nonces: self.nonces.clone(),
			gas_price_percentile: self.gas_price_percentile,
		}
	}
}

impl<C: miner::BlockChainClient, M: MinerService> FullDispatcher<C, M> {
	fn state_nonce(&self, from: &Address) -> U256 {
		self.miner.next_nonce(&*self.client, from)
	}

	/// Imports transaction to the miner's queue.
	pub fn dispatch_transaction(client: &C, miner: &M, signed_transaction: PendingTransaction, trusted: bool) -> Result<H256> {
		let hash = signed_transaction.transaction.hash();

		// use `import_claimed_local_transaction` so we can decide (based on config flags) if we want to treat
		// it as local or not. Nodes with public RPC interfaces will want these transactions to be treated like
		// external transactions.
		miner.import_claimed_local_transaction(client, signed_transaction, trusted)
			.map_err(errors::transaction)
			.map(|_| hash)
	}
}

impl<C: miner::BlockChainClient + BlockChainClient, M: MinerService> Dispatcher for FullDispatcher<C, M> {
	fn fill_optional_fields(&self, request: TransactionRequest, default_sender: Address, force_nonce: bool)
		-> BoxFuture<FilledTransactionRequest>
	{
		let request = request;
		let from = request.from.unwrap_or(default_sender);
		let nonce = if force_nonce {
			request.nonce.or_else(|| Some(self.state_nonce(&from)))
		} else {
			request.nonce
		};

		Box::new(future::ok(FilledTransactionRequest {
			from,
			used_default_from: request.from.is_none(),
			to: request.to,
			nonce,
			gas_price: request.gas_price.unwrap_or_else(|| {
				default_gas_price(&*self.client, &*self.miner, self.gas_price_percentile)
			}),
			gas: request.gas.unwrap_or_else(|| self.miner.sensible_gas_limit()),
			value: request.value.unwrap_or_else(|| 0.into()),
			data: request.data.unwrap_or_else(Vec::new),
			condition: request.condition,
		}))
	}

	fn sign(&self, accounts: Arc<AccountProvider>, filled: FilledTransactionRequest, password: SignWith)
		-> BoxFuture<WithToken<SignedTransaction>>
	{
		let chain_id = self.client.signing_chain_id();

		if let Some(nonce) = filled.nonce {
			return Box::new(future::done(sign_transaction(&*accounts, filled, chain_id, nonce, password)));
		}

		let state = self.state_nonce(&filled.from);
		let reserved = self.nonces.lock().reserve(filled.from, state);

		Box::new(ProspectiveSigner::new(accounts, filled, chain_id, reserved, password))
	}

	fn enrich(&self, signed_transaction: SignedTransaction) -> RpcRichRawTransaction {
		RpcRichRawTransaction::from_signed(signed_transaction)
	}

	fn dispatch_transaction(&self, signed_transaction: PendingTransaction) -> Result<H256> {
		Self::dispatch_transaction(&*self.client, &*self.miner, signed_transaction, true)
	}
}

/// Get a recent gas price corpus.
// TODO: this could be `impl Trait`.
pub fn fetch_gas_price_corpus(
	sync: Arc<LightSync>,
	client: Arc<LightChainClient>,
	on_demand: Arc<OnDemand>,
	cache: Arc<Mutex<LightDataCache>>,
) -> BoxFuture<Corpus<U256>> {
	const GAS_PRICE_SAMPLE_SIZE: usize = 100;

	if let Some(cached) = { cache.lock().gas_price_corpus() } {
		return Box::new(future::ok(cached))
	}

	let cache = cache.clone();
	let eventual_corpus = sync.with_context(|ctx| {
		// get some recent headers with gas used,
		// and request each of the blocks from the network.
		let block_requests = client.ancestry_iter(BlockId::Latest)
			.filter(|hdr| hdr.gas_used() != U256::default())
			.take(GAS_PRICE_SAMPLE_SIZE)
			.map(|hdr| request::Body(hdr.into()))
			.collect::<Vec<_>>();

		// when the blocks come in, collect gas prices into a vector
		on_demand.request(ctx, block_requests)
			.expect("no back-references; therefore all back-references are valid; qed")
			.map(|bodies| {
				bodies.into_iter().fold(Vec::new(), |mut v, block| {
					for t in block.transaction_views().iter() {
						v.push(t.gas_price())
					}

					v
				})
			})
			.map(move |prices| {
				// produce a corpus from the vector and cache it.
				// It's later used to get a percentile for default gas price.
				let corpus: ::stats::Corpus<_> = prices.into();
				cache.lock().set_gas_price_corpus(corpus.clone());
				corpus
			})
	});

	match eventual_corpus {
		Some(corp) => Box::new(corp.map_err(|_| errors::no_light_peers())),
		None => Box::new(future::err(errors::network_disabled())),
	}
}

/// Returns a eth_sign-compatible hash of data to sign.
/// The data is prepended with special message to prevent
/// malicious DApps from using the function to sign forged transactions.
pub fn eth_data_hash(mut data: Bytes) -> H256 {
	let mut message_data =
		format!("\x19Ethereum Signed Message:\n{}", data.len())
		.into_bytes();
	message_data.append(&mut data);
	keccak(message_data)
}

/// Dispatcher for light clients -- fetches default gas price, next nonce, etc. from network.
#[derive(Clone)]
pub struct LightDispatcher {
	/// Sync service.
	pub sync: Arc<LightSync>,
	/// Header chain client.
	pub client: Arc<LightChainClient>,
	/// On-demand request service.
	pub on_demand: Arc<OnDemand>,
	/// Data cache.
	pub cache: Arc<Mutex<LightDataCache>>,
	/// Transaction queue.
	pub transaction_queue: Arc<RwLock<LightTransactionQueue>>,
	/// Nonce reservations
	pub nonces: Arc<Mutex<nonce::Reservations>>,
	/// Gas Price percentile value used as default gas price.
	pub gas_price_percentile: usize,
}

impl LightDispatcher {
	/// Create a new `LightDispatcher` from its requisite parts.
	///
	/// For correct operation, the OnDemand service is assumed to be registered as a network handler,
	pub fn new(
		sync: Arc<LightSync>,
		client: Arc<LightChainClient>,
		on_demand: Arc<OnDemand>,
		cache: Arc<Mutex<LightDataCache>>,
		transaction_queue: Arc<RwLock<LightTransactionQueue>>,
		nonces: Arc<Mutex<nonce::Reservations>>,
		gas_price_percentile: usize,
	) -> Self {
		LightDispatcher {
			sync,
			client,
			on_demand,
			cache,
			transaction_queue,
			nonces,
			gas_price_percentile,
		}
	}

	/// Get a recent gas price corpus.
	// TODO: this could be `impl Trait`.
	pub fn gas_price_corpus(&self) -> BoxFuture<Corpus<U256>> {
		fetch_gas_price_corpus(
			self.sync.clone(),
			self.client.clone(),
			self.on_demand.clone(),
			self.cache.clone(),
		)
	}

	/// Get an account's next nonce.
	pub fn next_nonce(&self, addr: Address) -> BoxFuture<U256> {
		// fast path where we don't go to network; nonce provided or can be gotten from queue.
		let maybe_nonce = self.transaction_queue.read().next_nonce(&addr);
		if let Some(nonce) = maybe_nonce {
			return Box::new(future::ok(nonce))
		}

		let best_header = self.client.best_block_header();
		let account_start_nonce = self.client.engine().account_start_nonce(best_header.number());
		let nonce_future = self.sync.with_context(|ctx| self.on_demand.request(ctx, request::Account {
			header: best_header.into(),
			address: addr,
		}).expect("no back-references; therefore all back-references valid; qed"));

		match nonce_future {
			Some(x) => Box::new(
				x.map(move |acc| acc.map_or(account_start_nonce, |acc| acc.nonce))
					.map_err(|_| errors::no_light_peers())
			),
			None => Box::new(future::err(errors::network_disabled()))
		}
	}
}

impl Dispatcher for LightDispatcher {
	fn fill_optional_fields(&self, request: TransactionRequest, default_sender: Address, force_nonce: bool)
		-> BoxFuture<FilledTransactionRequest>
	{
		const DEFAULT_GAS_PRICE: U256 = U256([0, 0, 0, 21_000_000]);

		let gas_limit = self.client.best_block_header().gas_limit();
		let request_gas_price = request.gas_price.clone();
		let request_nonce = request.nonce.clone();
		let from = request.from.unwrap_or(default_sender);

		let with_gas_price = move |gas_price| {
			let request = request;
			FilledTransactionRequest {
				from: from.clone(),
				used_default_from: request.from.is_none(),
				to: request.to,
				nonce: request.nonce,
				gas_price: gas_price,
				gas: request.gas.unwrap_or_else(|| gas_limit / 3),
				value: request.value.unwrap_or_else(|| 0.into()),
				data: request.data.unwrap_or_else(Vec::new),
				condition: request.condition,
			}
		};

		// fast path for known gas price.
		let gas_price_percentile = self.gas_price_percentile;
		let gas_price = match request_gas_price {
			Some(gas_price) => Either::A(future::ok(with_gas_price(gas_price))),
			None => Either::B(fetch_gas_price_corpus(
				self.sync.clone(),
				self.client.clone(),
				self.on_demand.clone(),
				self.cache.clone()
			).and_then(move |corp| match corp.percentile(gas_price_percentile) {
				Some(percentile) => Ok(*percentile),
				None => Ok(DEFAULT_GAS_PRICE), // fall back to default on error.
			}).map(with_gas_price))
		};

		match (request_nonce, force_nonce) {
			(_, false) | (Some(_), true) => Box::new(gas_price),
			(None, true) => {
				let next_nonce = self.next_nonce(from);
				Box::new(gas_price.and_then(move |mut filled| next_nonce
					.map_err(|_| errors::no_light_peers())
					.map(move |nonce| {
						filled.nonce = Some(nonce);
						filled
					})
				))
			},
		}
	}

	fn sign(&self, accounts: Arc<AccountProvider>, filled: FilledTransactionRequest, password: SignWith)
		-> BoxFuture<WithToken<SignedTransaction>>
	{
		let chain_id = self.client.signing_chain_id();

		// fast path for pre-filled nonce.
		if let Some(nonce) = filled.nonce {
			return Box::new(future::done(sign_transaction(&*accounts, filled, chain_id, nonce, password)))
		}

		let nonces = self.nonces.clone();
		Box::new(self.next_nonce(filled.from)
			.map_err(|_| errors::no_light_peers())
			.and_then(move |nonce| {
				let reserved = nonces.lock().reserve(filled.from, nonce);

				ProspectiveSigner::new(accounts, filled, chain_id, reserved, password)
			}))
	}

	fn enrich(&self, signed_transaction: SignedTransaction) -> RpcRichRawTransaction {
		RpcRichRawTransaction::from_signed(signed_transaction)
	}

	fn dispatch_transaction(&self, signed_transaction: PendingTransaction) -> Result<H256> {
		let hash = signed_transaction.transaction.hash();

		self.transaction_queue.write().import(signed_transaction)
			.map_err(errors::transaction)
			.map(|_| hash)
	}
}

fn sign_transaction(
	accounts: &AccountProvider,
	filled: FilledTransactionRequest,
	chain_id: Option<u64>,
	nonce: U256,
	password: SignWith,
) -> Result<WithToken<SignedTransaction>> {
	let t = Transaction {
		nonce: nonce,
		action: filled.to.map_or(Action::Create, Action::Call),
		gas: filled.gas,
		gas_price: filled.gas_price,
		value: filled.value,
		data: filled.data,
	};

	if accounts.is_hardware_address(&filled.from) {
		return hardware_signature(accounts, filled.from, t, chain_id).map(WithToken::No)
	}

	let hash = t.hash(chain_id);
	let signature = signature(accounts, filled.from, hash, password)?;

	Ok(signature.map(|sig| {
		SignedTransaction::new(t.with_signature(sig, chain_id))
			.expect("Transaction was signed by AccountsProvider; it never produces invalid signatures; qed")
	}))
}

#[derive(Debug, Clone, Copy)]
enum ProspectiveSignerState {
	TryProspectiveSign,
	WaitForNonce,
	Finish,
}

struct ProspectiveSigner {
	accounts: Arc<AccountProvider>,
	filled: FilledTransactionRequest,
	chain_id: Option<u64>,
	reserved: nonce::Reserved,
	password: SignWith,
	state: ProspectiveSignerState,
	prospective: Option<Result<WithToken<SignedTransaction>>>,
	ready: Option<nonce::Ready>,
}

impl ProspectiveSigner {
	pub fn new(
		accounts: Arc<AccountProvider>,
		filled: FilledTransactionRequest,
		chain_id: Option<u64>,
		reserved: nonce::Reserved,
		password: SignWith,
	) -> Self {
		// If the account is permanently unlocked we can try to sign
		// using prospective nonce. This should speed up sending
		// multiple subsequent transactions in multi-threaded RPC environment.
		let is_unlocked_permanently = accounts.is_unlocked_permanently(&filled.from);
		let has_password = password.is_password();

		ProspectiveSigner {
			accounts,
			filled,
			chain_id,
			reserved,
			password,
			state: if is_unlocked_permanently || has_password {
				ProspectiveSignerState::TryProspectiveSign
			} else {
				ProspectiveSignerState::WaitForNonce
			},
			prospective: None,
			ready: None,
		}
	}

	fn sign(&self, nonce: &U256) -> Result<WithToken<SignedTransaction>> {
		sign_transaction(
			&*self.accounts,
			self.filled.clone(),
			self.chain_id,
			*nonce,
			self.password.clone()
		)
	}

	fn poll_reserved(&mut self) -> Poll<nonce::Ready, Error> {
		self.reserved.poll().map_err(|_| errors::internal("Nonce reservation failure", ""))
	}
}

impl Future for ProspectiveSigner {
	type Item = WithToken<SignedTransaction>;
	type Error = Error;

	fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
		use self::ProspectiveSignerState::*;

		loop {
			match self.state {
				TryProspectiveSign => {
					// Try to poll reserved, it might be ready.
					match self.poll_reserved()? {
						Async::NotReady => {
							self.state = WaitForNonce;
							self.prospective = Some(self.sign(self.reserved.prospective_value()));
						},
						Async::Ready(nonce) => {
							self.state = Finish;
							self.prospective = Some(self.sign(nonce.value()));
							self.ready = Some(nonce);
						},
					}
				},
				WaitForNonce => {
					let nonce = try_ready!(self.poll_reserved());
					let result = match (self.prospective.take(), nonce.matches_prospective()) {
						(Some(prospective), true) => prospective,
						_ => self.sign(nonce.value()),
					};
					self.state = Finish;
					self.prospective = Some(result);
					self.ready = Some(nonce);
				},
				Finish => {
					if let (Some(result), Some(nonce)) = (self.prospective.take(), self.ready.take()) {
						// Mark nonce as used on successful signing
						return result.map(move |tx| {
							nonce.mark_used();
							Async::Ready(tx)
						})
					} else {
						panic!("Poll after ready.");
					}
				}
			}
		}
	}
}

/// Single-use account token.
pub type AccountToken = Password;

/// Values used to unlock accounts for signing.
#[derive(Clone, PartialEq)]
pub enum SignWith {
	/// Nothing -- implies the account is already unlocked.
	Nothing,
	/// Unlock with password.
	Password(Password),
	/// Unlock with single-use token.
	Token(AccountToken),
}

impl SignWith {
	fn is_password(&self) -> bool {
		if let SignWith::Password(_) = *self {
			true
		} else {
			false
		}
	}
}

/// A value, potentially accompanied by a signing token.
pub enum WithToken<T> {
	/// No token.
	No(T),
	/// With token.
	Yes(T, AccountToken),
}

impl<T: Debug> Deref for WithToken<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		match *self {
			WithToken::No(ref v) => v,
			WithToken::Yes(ref v, _) => v,
		}
	}
}

impl<T: Debug> WithToken<T> {
	/// Map the value with the given closure, preserving the token.
	pub fn map<S, F>(self, f: F) -> WithToken<S> where
		S: Debug,
		F: FnOnce(T) -> S,
	{
		match self {
			WithToken::No(v) => WithToken::No(f(v)),
			WithToken::Yes(v, token) => WithToken::Yes(f(v), token),
		}
	}

	/// Convert into inner value, ignoring possible token.
	pub fn into_value(self) -> T {
		match self {
			WithToken::No(v) => v,
			WithToken::Yes(v, _) => v,
		}
	}

	/// Convert the `WithToken` into a tuple.
	pub fn into_tuple(self) -> (T, Option<AccountToken>) {
		match self {
			WithToken::No(v) => (v, None),
			WithToken::Yes(v, token) => (v, Some(token))
		}
	}
}

impl<T: Debug> From<(T, AccountToken)> for WithToken<T> {
	fn from(tuple: (T, AccountToken)) -> Self {
		WithToken::Yes(tuple.0, tuple.1)
	}
}

impl<T: Debug> From<(T, Option<AccountToken>)> for WithToken<T> {
	fn from(tuple: (T, Option<AccountToken>)) -> Self {
		match tuple.1 {
			Some(token) => WithToken::Yes(tuple.0, token),
			None => WithToken::No(tuple.0),
		}
	}
}

/// Execute a confirmation payload.
pub fn execute<D: Dispatcher + 'static>(
	dispatcher: D,
	accounts: Arc<AccountProvider>,
	payload: ConfirmationPayload,
	pass: SignWith
) -> BoxFuture<WithToken<ConfirmationResponse>> {
	match payload {
		ConfirmationPayload::SendTransaction(request) => {
			let condition = request.condition.clone().map(Into::into);
			Box::new(dispatcher.sign(accounts, request, pass)
				.map(move |v| v.map(move |tx| PendingTransaction::new(tx, condition)))
				.map(WithToken::into_tuple)
				.map(|(tx, token)| (tx, token, dispatcher))
				.and_then(|(tx, tok, dispatcher)| {
					dispatcher.dispatch_transaction(tx)
						.map(RpcH256::from)
						.map(ConfirmationResponse::SendTransaction)
						.map(move |h| WithToken::from((h, tok)))
				}))
		},
		ConfirmationPayload::SignTransaction(request) => {
			Box::new(dispatcher.sign(accounts, request, pass)
				.map(move |result| result
					.map(move |tx| dispatcher.enrich(tx))
					.map(ConfirmationResponse::SignTransaction)
				))
		},
		ConfirmationPayload::EthSignMessage(address, data) => {
			if accounts.is_hardware_address(&address) {
				let signature = accounts.sign_message_with_hardware(&address, &data)
					.map(|s| H520(s.into_electrum()))
					.map(RpcH520::from)
					.map(ConfirmationResponse::Signature)
					// TODO: is this correct? I guess the `token` is the wallet in this context
					.map(WithToken::No)
					.map_err(|e| errors::account("Error signing message with hardware_wallet", e));

				return Box::new(future::done(signature));
			}
			let hash = eth_data_hash(data);
			let res = signature(&accounts, address, hash, pass)
				.map(|result| result
					.map(|rsv| H520(rsv.into_electrum()))
					.map(RpcH520::from)
					.map(ConfirmationResponse::Signature)
				);
			Box::new(future::done(res))
		},
		ConfirmationPayload::Decrypt(address, data) => {
			if accounts.is_hardware_address(&address) {
				return Box::new(future::err(errors::unsupported("Decrypting via hardware wallets is not supported.", None)));
			}
			let res = decrypt(&accounts, address, data, pass)
				.map(|result| result
					.map(RpcBytes)
					.map(ConfirmationResponse::Decrypt)
				);
			Box::new(future::done(res))
		},
	}
}

fn signature(accounts: &AccountProvider, address: Address, hash: H256, password: SignWith) -> Result<WithToken<Signature>> {
	match password.clone() {
		SignWith::Nothing => accounts.sign(address, None, hash).map(WithToken::No),
		SignWith::Password(pass) => accounts.sign(address, Some(pass), hash).map(WithToken::No),
		SignWith::Token(token) => accounts.sign_with_token(address, token, hash).map(Into::into),
	}.map_err(|e| match password {
		SignWith::Nothing => errors::signing(e),
		_ => errors::password(e),
	})
}

// obtain a hardware signature from the given account.
fn hardware_signature(accounts: &AccountProvider, address: Address, t: Transaction, chain_id: Option<u64>)
	-> Result<SignedTransaction>
{
	debug_assert!(accounts.is_hardware_address(&address));

	let mut stream = rlp::RlpStream::new();
	t.rlp_append_unsigned_transaction(&mut stream, chain_id);
	let signature = accounts.sign_transaction_with_hardware(&address, &t, chain_id, &stream.as_raw())
		.map_err(|e| {
			debug!(target: "miner", "Error signing transaction with hardware wallet: {}", e);
			errors::account("Error signing transaction with hardware wallet", e)
		})?;

	SignedTransaction::new(t.with_signature(signature, chain_id))
		.map_err(|e| {
			debug!(target: "miner", "Hardware wallet has produced invalid signature: {}", e);
			errors::account("Invalid signature generated", e)
		})
}

fn decrypt(accounts: &AccountProvider, address: Address, msg: Bytes, password: SignWith) -> Result<WithToken<Bytes>> {
	match password.clone() {
		SignWith::Nothing => accounts.decrypt(address, None, &DEFAULT_MAC, &msg).map(WithToken::No),
		SignWith::Password(pass) => accounts.decrypt(address, Some(pass), &DEFAULT_MAC, &msg).map(WithToken::No),
		SignWith::Token(token) => accounts.decrypt_with_token(address, token, &DEFAULT_MAC, &msg).map(Into::into),
	}.map_err(|e| match password {
		SignWith::Nothing => errors::signing(e),
		_ => errors::password(e),
	})
}

/// Extract the default gas price from a client and miner.
pub fn default_gas_price<C, M>(client: &C, miner: &M, percentile: usize) -> U256 where
	C: BlockChainClient,
	M: MinerService,
{
	client.gas_price_corpus(100).percentile(percentile).cloned().unwrap_or_else(|| miner.sensible_gas_price())
}

/// Convert RPC confirmation payload to signer confirmation payload.
/// May need to resolve in the future to fetch things like gas price.
pub fn from_rpc<D>(payload: RpcConfirmationPayload, default_account: Address, dispatcher: &D) -> BoxFuture<ConfirmationPayload>
	where D: Dispatcher
{
	match payload {
		RpcConfirmationPayload::SendTransaction(request) => {
			Box::new(dispatcher.fill_optional_fields(request.into(), default_account, false)
				.map(ConfirmationPayload::SendTransaction))
		},
		RpcConfirmationPayload::SignTransaction(request) => {
			Box::new(dispatcher.fill_optional_fields(request.into(), default_account, false)
				.map(ConfirmationPayload::SignTransaction))
		},
		RpcConfirmationPayload::Decrypt(RpcDecryptRequest { address, msg }) => {
			Box::new(future::ok(ConfirmationPayload::Decrypt(address.into(), msg.into())))
		},
		RpcConfirmationPayload::EthSignMessage(RpcSignRequest { address, data }) => {
			Box::new(future::ok(ConfirmationPayload::EthSignMessage(address.into(), data.into())))
		},
	}
}
