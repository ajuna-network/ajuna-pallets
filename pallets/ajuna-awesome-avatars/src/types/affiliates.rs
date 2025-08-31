use frame_support::pallet_prelude::{
	Decode, DecodeWithMemTracking, Encode, MaxEncodedLen, TypeInfo,
};
use sp_runtime::BoundedVec;

#[derive(
	Encode, Decode, DecodeWithMemTracking, MaxEncodedLen, TypeInfo, Clone, Debug, PartialEq, Eq,
)]
pub enum AffiliateMethods {
	Mint,
	UpgradeStorage,
	Buy,
}

pub type FeePropagation<T> = BoundedVec<u8, T>;
