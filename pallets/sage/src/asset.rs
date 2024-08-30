use frame_support::{
	pallet_prelude::{Decode, Encode, MaxEncodedLen, TypeInfo},
	Parameter,
};
use sp_runtime::traits::Member;

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Debug, Default, PartialEq, Eq)]
pub struct Asset<Data, BlockNumber>
where
	Data: Parameter + Member,
{
	pub(crate) data: Data,
	pub(crate) created_at: BlockNumber,
}
