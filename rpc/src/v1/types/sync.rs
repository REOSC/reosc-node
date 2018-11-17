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

use std::collections::BTreeMap;
use sync::{self, PeerInfo as SyncPeerInfo, TransactionStats as SyncTransactionStats};
use serde::{Serialize, Serializer};
use v1::types::{U256, H512};

/// Sync info
#[derive(Default, Debug, Serialize, PartialEq)]
pub struct SyncInfo {
	/// Starting block
	#[serde(rename="startingBlock")]
	pub starting_block: U256,
	/// Current block
	#[serde(rename="currentBlock")]
	pub current_block: U256,
	/// Highest block seen so far
	#[serde(rename="highestBlock")]
	pub highest_block: U256,
	/// Warp sync snapshot chunks total.
	#[serde(rename="warpChunksAmount")]
	pub warp_chunks_amount: Option<U256>,
	/// Warp sync snpashot chunks processed.
	#[serde(rename="warpChunksProcessed")]
	pub warp_chunks_processed: Option<U256>,
}

/// Peers info
#[derive(Default, Debug, Serialize)]
pub struct Peers {
	/// Number of active peers
	pub active: usize,
	/// Number of connected peers
	pub connected: usize,
	/// Max number of peers
	pub max: u32,
	/// Detailed information on peers
	pub peers: Vec<PeerInfo>,
}

/// Peer connection information
#[derive(Default, Debug, Serialize)]
pub struct PeerInfo {
	/// Public node id
	pub id: Option<String>,
	/// Node client ID
	pub name: String,
	/// Capabilities
	pub caps: Vec<String>,
	/// Network information
	pub network: PeerNetworkInfo,
	/// Protocols information
	pub protocols: PeerProtocolsInfo,
}

/// Peer network information
#[derive(Default, Debug, Serialize)]
pub struct PeerNetworkInfo {
	/// Remote endpoint address
	#[serde(rename="remoteAddress")]
	pub remote_address: String,
	/// Local endpoint address
	#[serde(rename="localAddress")]
	pub local_address: String,
}

/// Peer protocols information
#[derive(Default, Debug, Serialize)]
pub struct PeerProtocolsInfo {
	/// Ethereum protocol information
	pub eth: Option<EthProtocolInfo>,
	/// PIP protocol information.
	pub pip: Option<PipProtocolInfo>,
}

/// Peer Ethereum protocol information
#[derive(Default, Debug, Serialize)]
pub struct EthProtocolInfo {
	/// Negotiated ethereum protocol version
	pub version: u32,
	/// Peer total difficulty if known
	pub difficulty: Option<U256>,
	/// SHA3 of peer best block hash
	pub head: String,
}

impl From<sync::EthProtocolInfo> for EthProtocolInfo {
	fn from(info: sync::EthProtocolInfo) -> Self {
		EthProtocolInfo {
			version: info.version,
			difficulty: info.difficulty.map(Into::into),
			head: format!("{:x}", info.head),
		}
	}
}

/// Peer PIP protocol information
#[derive(Default, Debug, Serialize)]
pub struct PipProtocolInfo {
	/// Negotiated PIP protocol version
	pub version: u32,
	/// Peer total difficulty
	pub difficulty: U256,
	/// SHA3 of peer best block hash
	pub head: String,
}

impl From<sync::PipProtocolInfo> for PipProtocolInfo {
	fn from(info: sync::PipProtocolInfo) -> Self {
		PipProtocolInfo {
			version: info.version,
			difficulty: info.difficulty.into(),
			head: format!("{:x}", info.head),
		}
	}
}

/// Sync status
#[derive(Debug, PartialEq)]
pub enum SyncStatus {
	/// Info when syncing
	Info(SyncInfo),
	/// Not syncing
	None
}

impl Serialize for SyncStatus {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where S: Serializer {
		match *self {
			SyncStatus::Info(ref info) => info.serialize(serializer),
			SyncStatus::None => false.serialize(serializer)
		}
	}
}

/// Propagation statistics for pending transaction.
#[derive(Default, Debug, Serialize)]
pub struct TransactionStats {
	/// Block no this transaction was first seen.
	#[serde(rename="firstSeen")]
	pub first_seen: u64,
	/// Peers this transaction was propagated to with count.
	#[serde(rename="propagatedTo")]
	pub propagated_to: BTreeMap<H512, usize>,
}

impl From<SyncPeerInfo> for PeerInfo {
	fn from(p: SyncPeerInfo) -> Self {
		PeerInfo {
			id: p.id,
			name: p.client_version,
			caps: p.capabilities,
			network: PeerNetworkInfo {
				remote_address: p.remote_address,
				local_address: p.local_address,
			},
			protocols: PeerProtocolsInfo {
				eth: p.eth_info.map(Into::into),
				pip: p.pip_info.map(Into::into),
			},
		}
	}
}

impl From<SyncTransactionStats> for TransactionStats {
	fn from(s: SyncTransactionStats) -> Self {
		TransactionStats {
			first_seen: s.first_seen,
			propagated_to: s.propagated_to
				.into_iter()
				.map(|(id, count)| (id.into(), count))
				.collect(),
		}
	}
}

/// Chain status.
#[derive(Default, Debug, Serialize)]
pub struct ChainStatus {
	/// Describes the gap in the blockchain, if there is one: (first, last)
	#[serde(rename="blockGap")]
	pub block_gap: Option<(U256, U256)>,
}

#[cfg(test)]
mod tests {
	use serde_json;
	use std::collections::BTreeMap;
	use super::{SyncInfo, SyncStatus, Peers, TransactionStats, ChainStatus};

	#[test]
	fn test_serialize_sync_info() {
		let t = SyncInfo::default();
		let serialized = serde_json::to_string(&t).unwrap();
		assert_eq!(serialized, r#"{"startingBlock":"0x0","currentBlock":"0x0","highestBlock":"0x0","warpChunksAmount":null,"warpChunksProcessed":null}"#);
	}

	#[test]
	fn test_serialize_peers() {
		let t = Peers::default();
		let serialized = serde_json::to_string(&t).unwrap();
		assert_eq!(serialized, r#"{"active":0,"connected":0,"max":0,"peers":[]}"#);
	}

	#[test]
	fn test_serialize_sync_status() {
		let t = SyncStatus::None;
		let serialized = serde_json::to_string(&t).unwrap();
		assert_eq!(serialized, "false");

		let t = SyncStatus::Info(SyncInfo::default());
		let serialized = serde_json::to_string(&t).unwrap();
		assert_eq!(serialized, r#"{"startingBlock":"0x0","currentBlock":"0x0","highestBlock":"0x0","warpChunksAmount":null,"warpChunksProcessed":null}"#);
	}

	#[test]
	fn test_serialize_block_gap() {
		let mut t = ChainStatus::default();
		let serialized = serde_json::to_string(&t).unwrap();
		assert_eq!(serialized, r#"{"blockGap":null}"#);

		t.block_gap = Some((1.into(), 5.into()));

		let serialized = serde_json::to_string(&t).unwrap();
		assert_eq!(serialized, r#"{"blockGap":["0x1","0x5"]}"#);
	}

	#[test]
	fn test_serialize_transaction_stats() {
		let stats = TransactionStats {
			first_seen: 100,
			propagated_to: map![
				10.into() => 50
			],
		};

		let serialized = serde_json::to_string(&stats).unwrap();
		assert_eq!(serialized, r#"{"firstSeen":100,"propagatedTo":{"0x0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a":50}}"#)
	}
}
