use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::traits::Member;
use sp_std::vec::Vec;

/// The aggregated trait that the `SageEngine` implements such that it can access all features of
/// our pallets.
///
/// This will be much more elaborate in th actual implementation.
pub trait SageApi {
	type Asset: AssetT;

	type Balance;

	type AccountId;

	type Error: AsErrorCode;

	fn transfer_ownership(asset: Self::Asset, to: Self::AccountId) -> Result<(), Self::Error>;
	fn handle_fees(balance: Self::Balance) -> Result<(), Self::Error>;
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

pub trait SageGameTransition {
	type Asset: AssetT + Member + Encode + Decode + MaxEncodedLen + TypeInfo;

	type AccountId;

	type Balance;

	/// An optional extra, which is simply forwarded to the `verify_rule` and `do_transition`
	/// method. If you don't need custom arguments, you can define that type as `()`.
	type Extra: Member + Encode + Decode + MaxEncodedLen + TypeInfo;

	type Error: AsErrorCode;

	fn verify_rule(
		transition_id: u32,
		assets: &[Self::Asset],
		extra: &Self::Extra,
	) -> Result<(), Self::Error>;

	fn do_transition(
		transition_id: u32,
		assets: Vec<Self::Asset>,
		extra: Self::Extra,
	) -> Result<(), Self::Error>;
}
