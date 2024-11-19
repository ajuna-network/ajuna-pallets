use frame_support::Parameter;
use parity_scale_codec::MaxEncodedLen;
use scale_info::TypeInfo;
use sp_runtime::traits::Member;
use sp_std::vec::Vec;

/// The aggregated trait that the `SageEngine` implements such that it can access all features of
/// our pallets.
///
/// This will be much more elaborate in th actual implementation.
pub trait SageApi {
	type AssetId: Member + Parameter + MaxEncodedLen;

	type Asset: AssetT;

	type Balance;

	type AccountId;

	fn ensure_ownership(
		account: &Self::AccountId,
		asset: &Self::AssetId,
	) -> Result<(), crate::Error>;

	fn try_mutate_asset<F: FnOnce(&mut Self::Asset) -> Result<(), crate::Error>>(
		asset: &Self::AssetId,
		f: F,
	) -> Result<(), crate::Error>;
	fn transfer_ownership(asset: Self::AssetId, to: Self::AccountId) -> Result<(), crate::Error>;
	fn handle_fees(balance: Self::Balance) -> Result<(), crate::Error>;
}

pub trait AssetT {
	fn collection_id(&self) -> u32;
	fn asset_type(&self) -> u32;
	fn dna(&self) -> [u8; 32];
	fn minted_at(&self) -> u32;
}

pub trait AsErrorCode {
	fn as_error_code(&self) -> u8;
}

impl AsErrorCode for u8 {
	fn as_error_code(&self) -> u8 {
		*self
	}
}

pub type AccountIdOf<T> = <<T as SageGameTransition>::SageApi as SageApi>::AccountId;
pub type BalanceOf<T> = <<T as SageGameTransition>::SageApi as SageApi>::Balance;

pub trait SageGameTransition {
	type AssetId: Member + Parameter + MaxEncodedLen + TypeInfo;
	type Asset: AssetT + Member + Parameter + MaxEncodedLen + TypeInfo;

	type SageApi: SageApi<AssetId = Self::AssetId, Asset = Self::Asset>;

	/// Transition Id type, can be a simple u32, or an enum.
	type TransitionId: Member + Parameter + Ord + PartialOrd + MaxEncodedLen + TypeInfo;

	/// Transition config type
	type TransitionConfig: Member + Parameter + MaxEncodedLen + TypeInfo + Default;

	/// An optional extra, which is simply forwarded to the `verify_rule` and `do_transition`
	/// method. If you don't need custom arguments, you can define that type as `()`.
	type Extra: Member + Parameter + MaxEncodedLen + TypeInfo;

	fn verify_rule(
		transition_id: Self::TransitionId,
		account_id: &AccountIdOf<Self>,
		asset_ids: &[Self::AssetId],
		extra: &Self::Extra,
	) -> Result<(), crate::Error>;

	fn do_transition(
		transition_id: Self::TransitionId,
		account_id: AccountIdOf<Self>,
		assets_ids: Vec<Self::AssetId>,
		extra: Self::Extra,
	) -> Result<(), crate::Error>;
}
