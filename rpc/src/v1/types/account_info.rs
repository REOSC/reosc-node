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

/// Account information.
#[derive(Debug, Default, Clone, PartialEq, Serialize)]
pub struct AccountInfo {
	/// Account name
	pub name: String,
}

/// Extended account information (used by `parity_allAccountInfo`).
#[derive(Debug, Default, Clone, PartialEq, Serialize)]
pub struct ExtAccountInfo {
	/// Account name
	pub name: String,
	/// Account meta JSON
	pub meta: String,
	/// Account UUID (`None` for address book entries)
	#[serde(skip_serializing_if = "Option::is_none")]
	pub uuid: Option<String>,
}

/// Hardware wallet information.
#[derive(Debug, Default, Clone, PartialEq, Serialize)]
pub struct HwAccountInfo {
	/// Device name.
	pub name: String,
	/// Device manufacturer.
	pub manufacturer: String,
}
