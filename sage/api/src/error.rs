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
