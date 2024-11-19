use frame_support::Parameter;
use parity_scale_codec::{Codec, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::traits::Member;
use sp_std::vec::Vec;

pub enum TransitionOutput<AssetId, Asset> {
	Minted(Asset),
	Mutated(AssetId, Asset),
	Consumed(AssetId),
}

pub trait SageGameTransition {
	/// Transition identifier type.
	type TransitionId: Member + Parameter + Ord + PartialOrd + MaxEncodedLen + TypeInfo;
	type AccountId: Member + Codec;
	type AssetId: Member + Parameter + MaxEncodedLen + TypeInfo;
	type Asset: Member + Parameter + MaxEncodedLen + TypeInfo;
	/// An optional extra, which is simply forwarded to the `verify_rule` and `do_transition`
	/// method. If you don't need custom arguments, you can define that type as `()`.
	type Extra: Member + Parameter + MaxEncodedLen + TypeInfo;

	fn verify_rule(
		transition_id: &Self::TransitionId,
		account_id: &Self::AccountId,
		asset_ids: &[Self::AssetId],
		extra: &Self::Extra,
	) -> Result<(), crate::Error>;

	fn do_transition(
		transition_id: &Self::TransitionId,
		account_id: &Self::AccountId,
		assets_ids: &[Self::AssetId],
		extra: &Self::Extra,
	) -> Result<Vec<TransitionOutput<Self::AssetId, Self::Asset>>, crate::Error>;
}
