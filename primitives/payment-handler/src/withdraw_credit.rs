use crate::voucher_handler::VoucherHandler;
use core::{fmt::Debug, marker::PhantomData};
use frame_support::{
	pallet_prelude::{DispatchError, Encode},
	traits::{
		fungible,
		fungible::Balanced as AssetBalanced,
		fungibles,
		fungibles::Balanced as AssetsBalanced,
		tokens::{Balance as TokenBalance, Fortitude, Precision, Preservation},
	},
};
use parity_scale_codec::{Decode, EncodeLike, MaxEncodedLen};
use scale_info::TypeInfo;

/// Implements `WithdrawCredit`, but ensures that only whitelisted assets are withdrawn.
pub struct WithdrawWhitelistedCredit<Whitelist, Withdraw>(PhantomData<(Whitelist, Withdraw)>);

pub trait EnsureWhitelistedAsset {
	type AssetId;

	fn ensure_whitelisted(asset_id: &Self::AssetId) -> Result<(), DispatchError>;
}

pub struct AllowAllAssets<A>(PhantomData<A>);

impl<AssetId> EnsureWhitelistedAsset for AllowAllAssets<AssetId> {
	type AssetId = AssetId;

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
	type Balance: TokenBalance;

	type Credit;

	fn withdraw_credit(
		who: &Self::AccountId,
		asset_id: Self::AssetId,
		credit: Self::Balance,
	) -> Result<Self::Credit, DispatchError>;
}

pub struct WithdrawNative<T, I>(PhantomData<(T, I)>);

impl<T: pallet_balances::Config<I> + frame_system::Config, I: 'static> WithdrawCredit
	for WithdrawNative<T, I>
{
	type AccountId = T::AccountId;
	type AssetId = ();
	type Assets = pallet_balances::Pallet<T, I>;
	type Balance = T::Balance;
	type Credit = fungible::Credit<Self::AccountId, pallet_balances::Pallet<T, I>>;

	fn withdraw_credit(
		who: &Self::AccountId,
		_: Self::AssetId,
		credit: Self::Balance,
	) -> Result<Self::Credit, DispatchError> {
		pallet_balances::Pallet::<T, I>::withdraw(
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
	) -> Result<Self::Credit, DispatchError> {
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

pub struct WithdrawFungibles<Fungibles, AccountId>(PhantomData<(Fungibles, AccountId)>);

impl<Fungibles, AccountId> WithdrawCredit for WithdrawFungibles<Fungibles, AccountId>
where
	Fungibles: fungibles::Inspect<AccountId> + fungibles::Balanced<AccountId>,
{
	type AccountId = AccountId;
	type AssetId = Fungibles::AssetId;
	type Assets = Fungibles;
	type Balance = Fungibles::Balance;
	type Credit = fungibles::Credit<Self::AccountId, Fungibles>;

	fn withdraw_credit(
		who: &Self::AccountId,
		asset_id: Self::AssetId,
		credit: Self::Balance,
	) -> Result<Self::Credit, DispatchError> {
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

#[derive(Debug, Encode, Decode, PartialEq, Eq, Clone, MaxEncodedLen, TypeInfo)]
pub enum WithdrawKind<AssetId> {
	Payment(AssetId),
	Voucher,
}

pub struct WithdrawCreditOrVoucher<W, V>(PhantomData<(W, V)>);

impl<AccountId, Balance, W, V> WithdrawCredit for WithdrawCreditOrVoucher<W, V>
where
	Balance: TokenBalance,
	W: WithdrawCredit<AccountId = AccountId, Balance = Balance>,
	W::AssetId: 'static,
	V: VoucherHandler<AccountId = AccountId, Balance = Balance>,
{
	type AccountId = AccountId;
	type AssetId = WithdrawKind<W::AssetId>;
	type Assets = W::Assets;
	type Balance = Balance;
	type Credit = Option<W::Credit>;

	fn withdraw_credit(
		who: &Self::AccountId,
		asset_id: Self::AssetId,
		credit: Self::Balance,
	) -> Result<Self::Credit, DispatchError> {
		match asset_id {
			WithdrawKind::Payment(payment_asset_id) =>
				W::withdraw_credit(who, payment_asset_id, credit).map(Some),
			WithdrawKind::Voucher => V::consume_vouchers_from(who, credit).map(|_| None),
		}
	}
}
