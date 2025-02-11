use frame_support::pallet_prelude::{Decode, Encode, MaxEncodedLen, TypeInfo};

/// Errors that may happen during the execution of a SAGE transition.
#[derive(
	Encode, Decode, MaxEncodedLen, TypeInfo, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord,
)]
pub enum TransitionError {
	TransferError,
	FeeError,
	VoucherNotAllowed,
	AssetLength,
	AssetOwnership,
	Transition { code: u8 },
}
