use frame_support::pallet_prelude::{Decode, Encode, MaxEncodedLen, TypeInfo};

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Debug, PartialEq, Eq)]
pub enum AffiliateMethods {
	Mint,
	UpgradeStorage,
	Buy,
}
