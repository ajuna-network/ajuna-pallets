use frame_support::pallet_prelude::*;

#[derive(Encode, Decode, DecodeWithMemTracking, MaxEncodedLen, TypeInfo, Clone, Debug, Default, PartialEq)]
pub struct GeneralConfig<Balance> {
	pub open: bool,
	pub transfer_fee: Balance,
}
