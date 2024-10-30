use crate::Error;
use ajuna_primitives::runtime_types::{AccountId, Balance};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::traits::Member;
use sp_std::vec::Vec;
use std::marker::PhantomData;

/// The aggregated trait that the `SageEngine` implements such that it can access all features of
/// our pallets.
///
/// This will be much more elaborate in th actual implementation.
pub trait SageApi {
	type AssetId;

	type Asset: AssetT;

	type Balance;

	type AccountId;

	fn ensure_ownership(
		account: &Self::AccountId,
		asset: &Self::AssetId,
	) -> Result<(), crate::Error>;

	fn try_mutate_asset<R, F: FnOnce(&mut Self::Asset) -> Result<R, crate::Error>>(
		asset: &Self::AssetId,
		f: F,
	) -> Result<R, crate::Error>;
	fn transfer_ownership(asset: Self::AssetId, to: Self::AccountId) -> Result<(), crate::Error>;
	fn handle_fees(balance: Self::Balance) -> Result<(), crate::Error>;
}

pub trait SageApiConcrete {
	type AssetId;

	type Asset: AssetT;

	fn ensure_ownership(account: AccountId, asset: &Self::AssetId) -> Result<(), crate::Error>;

	fn try_mutate_asset<R, F: FnOnce(&mut Self::Asset) -> Result<R, crate::Error>>(
		asset: &Self::AssetId,
		f: F,
	) -> Result<R, crate::Error>;
	fn transfer_ownership(asset: Self::AssetId, to: AccountId) -> Result<(), crate::Error>;
	fn handle_fees(balance: Balance) -> Result<(), crate::Error>;
}

pub type AjunaSageCore<AssetId, Asset> = SageCore<Balance, AccountId, AssetId, Asset>;

pub struct SageCore<Balance, AccountId, AssetId, Asset> {
	_phantom: PhantomData<(Balance, AccountId, AssetId, Asset)>,
}

impl<Balance, AccountId, AssetId, Asset: AssetT> SageApi
	for SageCore<Balance, AccountId, AssetId, Asset>
{
	type AssetId = AssetId;
	type Asset = Asset;
	type Balance = Balance;
	type AccountId = AccountId;

	fn ensure_ownership(account: &Self::AccountId, asset: &Self::AssetId) -> Result<(), Error> {
		todo!()
	}

	fn try_mutate_asset<R, F: FnOnce(&mut Self::Asset) -> Result<R, Error>>(
		asset: &Self::AssetId,
		f: F,
	) -> Result<R, Error> {
		todo!()
	}

	fn transfer_ownership(asset: Self::AssetId, to: Self::AccountId) -> Result<(), Error> {
		todo!()
	}

	fn handle_fees(balance: Self::Balance) -> Result<(), Error> {
		todo!()
	}
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
	type AssetId: Member + Encode + Decode + MaxEncodedLen + TypeInfo;
	type Asset: AssetT + Member + Encode + Decode + MaxEncodedLen + TypeInfo;

	type SageApi: SageApi<AssetId = Self::AssetId, Asset = Self::Asset>;

	/// Transition Id type, can be a simple u32, or an enum.
	type TransitionId: Member + Encode + Decode + MaxEncodedLen + TypeInfo;

	/// An optional extra, which is simply forwarded to the `verify_rule` and `do_transition`
	/// method. If you don't need custom arguments, you can define that type as `()`.
	type Extra: Member + Encode + Decode + MaxEncodedLen + TypeInfo;

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
