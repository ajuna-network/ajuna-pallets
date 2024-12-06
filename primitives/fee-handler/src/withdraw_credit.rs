use frame_support::{
	pallet_prelude::DispatchError,
	traits::{
		fungible,
		fungible::Balanced,
		fungibles,
		fungibles::{Balanced as AssetsBalanced, Credit},
		tokens::{Balance, Fortitude, Precision, Preservation},
	},
};
use parity_scale_codec::{Decode, EncodeLike, MaxEncodedLen};
use scale_info::TypeInfo;
use std::{fmt::Debug, marker::PhantomData};

/// Implements `WithdrawCredit`, but ensures that only whitelisted assets are withdrawn.
pub struct WithdrawWhitelistedCredit<Whitelist, Withdraw>(PhantomData<(Whitelist, Withdraw)>);

pub trait EnsureWhitelistedAsset {
	type AssetId;

	fn ensure_whitelisted(asset_id: &Self::AssetId) -> Result<(), DispatchError>;
}

pub struct AllowAllAssets;

impl EnsureWhitelistedAsset for AllowAllAssets {
	type AssetId = ();

	fn ensure_whitelisted(_: &Self::AssetId) -> Result<(), DispatchError> {
		Ok(())
	}
}

impl<Whitelist: EnsureWhitelistedAsset<AssetId = Withdraw::AssetId>, Withdraw: WithdrawCredit>
	WithdrawCredit for WithdrawWhitelistedCredit<Whitelist, Withdraw>
{
	type AccountId = Withdraw::AccountId;
	type AssetId = Withdraw::AssetId;
	type Assets = Withdraw::Assets;
	type Balance = Withdraw::Balance;
	type Credit = Withdraw::Credit;

	fn withdraw_credit(
		who: &Self::AccountId,
		asset_id: Self::AssetId,
		credit: Self::Balance,
	) -> Result<Self::Credit, DispatchError> {
		Whitelist::ensure_whitelisted(&asset_id)?;
		Withdraw::withdraw_credit(who, asset_id, credit)
	}
}

/// Abstraction to withdraw some asset from an account, and return a credit in some asset.
///
/// Currently, this is intended to be implemented by the `SwapCredit` adapter we have in the
/// ajuna-parachain, which will convert whitelisted assets into the native currency via the
/// `pallet-asset-conversion` and return a credit in the native balance, which can then be allocated
/// to the affiliates, or the specific treasury pots.
pub trait WithdrawCredit {
	type AccountId;
	type AssetId: Clone + Eq + Debug + TypeInfo + MaxEncodedLen + EncodeLike + Decode;

	type Assets;
	type Balance: Balance;

	type Credit;

	fn withdraw_credit(
		who: &Self::AccountId,
		asset_id: Self::AssetId,
		credit: Self::Balance,
	) -> Result<Self::Credit, DispatchError>;
}

pub struct WithdrawNative<T>(PhantomData<T>);

impl<T: pallet_balances::Config + frame_system::Config> WithdrawCredit for WithdrawNative<T> {
	type AccountId = T::AccountId;
	type AssetId = ();
	type Assets = pallet_balances::Pallet<T>;
	type Balance = T::Balance;
	type Credit = fungible::Credit<Self::AccountId, pallet_balances::Pallet<T>>;

	fn withdraw_credit(
		who: &Self::AccountId,
		_: Self::AssetId,
		credit: Self::Balance,
	) -> Result<Self::Credit, DispatchError> {
		pallet_balances::Pallet::<T>::withdraw(
			who,
			credit,
			Precision::Exact,
			Preservation::Preserve,
			Fortitude::Polite,
		)
	}
}

pub struct WithdrawAsset<T>(PhantomData<T>);

impl<T: pallet_assets::Config + frame_system::Config> WithdrawCredit for WithdrawAsset<T> {
	type AccountId = T::AccountId;
	type AssetId = T::AssetId;
	type Assets = pallet_assets::Pallet<T>;
	type Balance = T::Balance;
	type Credit = fungibles::Credit<Self::AccountId, pallet_assets::Pallet<T>>;

	fn withdraw_credit(
		who: &Self::AccountId,
		asset_id: Self::AssetId,
		credit: Self::Balance,
	) -> Result<Credit<Self::AccountId, Self::Assets>, DispatchError> {
		let asset_fee_credit = Self::Assets::withdraw(
			asset_id.clone(),
			who,
			credit,
			Precision::Exact,
			Preservation::Preserve,
			Fortitude::Polite,
		)?;

		Ok(asset_fee_credit)
	}
}
