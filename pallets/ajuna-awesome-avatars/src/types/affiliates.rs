use frame_support::pallet_prelude::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen, TypeInfo};

#[derive(Encode, Decode, DecodeWithMemTracking, MaxEncodedLen, TypeInfo, Clone, Debug, PartialEq, Eq)]
pub enum AffiliateMethods {
	Mint,
	UpgradeStorage,
	Buy,
}
