// Ajuna Node
// Copyright (C) 2022 BlogaTech AG
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use frame_support::pallet_prelude::{Decode, Encode, MaxEncodedLen, TypeInfo};
use sp_runtime::DispatchError;

/// Errors that may happen during the execution of a SAGE transition.
#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Debug, Copy, Clone, PartialEq, Eq)]
pub enum TransitionError {
	TransferError,
	FeeError,
	VoucherNotAllowed,
	CouldNotCreateAssetId,
	AssetLength,
	AssetOwnership,
	/// The asset could not be decoded. Maybe it was the wrong asset type.
	AssetCouldNotBeDecoded,
	/// The asset data was too long and could be written back to the asset.
	AssetDataTooLong,
	Transition {
		code: u8,
	},
	Dispatch {
		error: DispatchError,
	},
}

impl From<DispatchError> for TransitionError {
	fn from(error: DispatchError) -> TransitionError {
		TransitionError::Dispatch { error }
	}
}

impl From<u8> for TransitionError {
	fn from(error: u8) -> TransitionError {
		TransitionError::Transition { code: error }
	}
}
