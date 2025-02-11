use frame_support::Parameter;
use parity_scale_codec::{Codec, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::traits::Member;
use sp_std::vec::Vec;

pub trait GetId<Id> {
	fn get_id(&self) -> Id;
}

pub enum TransitionOutput<AssetId, Asset> {
	Minted(Asset),
	Mutated(AssetId, Asset),
	Consumed(AssetId),
}

pub trait SageGameTransition {
	/// Transition identifier type.
	type TransitionId: Member + Parameter + MaxEncodedLen + TypeInfo;
	/// THe type of the specific configuration options for the transition
	type TransitionConfig: Member + Parameter + MaxEncodedLen + TypeInfo + Default;
	/// The account id type, usually 32 bytes long.
	type AccountId: Member + Codec;
	/// The asset id type of the sage assets defined by the developer.
	type AssetId: Member + Parameter + MaxEncodedLen + TypeInfo;
	/// The asset type used in sage defined by the game developer.
	type Asset: Member + Parameter + MaxEncodedLen + TypeInfo + GetId<Self::AssetId>;
	/// An optional extra, which is simply forwarded to the `verify_rule` and `do_transition`
	/// method. If you don't need custom arguments, you can define that type as `()`.
	type Extra: Member + Parameter + MaxEncodedLen + TypeInfo + Default;

	/// Defines the fungible asset that was used to pay the transaction. If the transition accesses
	/// user funds, it might want to use the same asset, as this implies that the user is willing
	/// to use this one instead of the native balance.
	///
	/// If you don't need fungible tokens in your transition you can set this as ().
	type PaymentFungible;

	fn do_transition(
		transition_id: &Self::TransitionId,
		account_id: &Self::AccountId,
		assets_ids: &[Self::AssetId],
		extra: &Self::Extra,
		payment_kind: Option<Self::PaymentFungible>,
	) -> Result<Vec<TransitionOutput<Self::AssetId, Self::Asset>>, crate::TransitionError>;
}
