use frame_support::pallet_prelude::{Decode, Encode, MaxEncodedLen, TypeInfo};
use sp_runtime::BoundedVec;

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Debug, PartialEq, Eq)]
pub enum AffiliateMethods {
	Mint,
	UpgradeStorage,
	Buy,
}

pub type FeePropagation<T> = BoundedVec<u8, T>;
