// Ajuna Node
// Copyright (C) 2022 BlogaTech AG
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use crate::{IdentifyVoucherOrAssetId, voucher_handler::VoucherHandler};
use core::{fmt::Debug, marker::PhantomData};
use frame_support::{
	pallet_prelude::{DispatchError, Encode},
	traits::{
		fungible,
		fungible::NativeOrWithId,
		fungibles,
		tokens::{AssetId, Balance, Fortitude, Precision, Preservation},
	},
};
use parity_scale_codec::{Decode, DecodeWithMemTracking, MaxEncodedLen};
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
		preservation: Preservation,
	) -> Result<Option<Self::Credit>, DispatchError> {
		Whitelist::ensure_whitelisted(&asset_id)?;
		Withdraw::withdraw_credit(who, asset_id, credit, preservation)
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
	type AssetId: AssetId;

	type Assets;
	type Balance: Balance;

	type Credit;

	fn withdraw_credit(
		who: &Self::AccountId,
		asset_id: Self::AssetId,
		credit: Self::Balance,
		preservation: Preservation,
	) -> Result<Option<Self::Credit>, DispatchError>;
}

pub struct WithdrawNative<AccountId, Fungible>(PhantomData<(AccountId, Fungible)>);

impl<AccountId, Fungible> WithdrawCredit for WithdrawNative<AccountId, Fungible>
where
	Fungible: fungible::Balanced<AccountId>,
{
	type AccountId = AccountId;
	type AssetId = ();
	type Assets = Fungible;
	type Balance = Fungible::Balance;
	type Credit = fungible::Credit<AccountId, Fungible>;

	fn withdraw_credit(
		who: &Self::AccountId,
		_: Self::AssetId,
		credit: Self::Balance,
		preservation: Preservation,
	) -> Result<Option<Self::Credit>, DispatchError> {
		Self::Assets::withdraw(who, credit, Precision::Exact, preservation, Fortitude::Polite)
			.map(Some)
	}
}

pub struct WithdrawFungibles<AccountId, Fungibles>(PhantomData<(AccountId, Fungibles)>);

impl<AccountId, Fungibles> WithdrawCredit for WithdrawFungibles<AccountId, Fungibles>
where
	Fungibles: fungibles::Balanced<AccountId>,
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
		preservation: Preservation,
	) -> Result<Option<Self::Credit>, DispatchError> {
		Self::Assets::withdraw(
			asset_id,
			who,
			credit,
			Precision::Exact,
			preservation,
			Fortitude::Polite,
		)
		.map(Some)
	}
}

#[derive(
	Debug, Encode, Decode, DecodeWithMemTracking, PartialEq, Eq, Clone, MaxEncodedLen, TypeInfo,
)]
pub enum WithdrawKind<AssetId> {
	Payment(AssetId),
	Voucher,
}

// Not 100% sure if this is too generic and might lead to issues.
// So far it works though.
impl<AssetId: Ord> From<AssetId> for WithdrawKind<AssetId> {
	fn from(value: AssetId) -> Self {
		Self::Payment(value)
	}
}

impl<AssetId: NativeId> NativeId for WithdrawKind<AssetId> {
	fn get_native_id() -> Self {
		Self::Payment(AssetId::get_native_id())
	}

	fn is_native_id(&self) -> bool {
		match self {
			Self::Payment(asset_id) => asset_id.is_native_id(),
			Self::Voucher => false,
		}
	}
}

impl<AssetId> VoucherId for WithdrawKind<AssetId> {
	fn get_voucher_id() -> Option<Self> {
		Some(Self::Voucher)
	}

	fn is_voucher_id(&self) -> bool {
		match self {
			Self::Voucher => true,
			Self::Payment(_) => false,
		}
	}
}

impl<AssetId: Ord> NativeId for NativeOrWithId<AssetId> {
	fn get_native_id() -> Self {
		NativeOrWithId::Native
	}

	fn is_native_id(&self) -> bool {
		match self {
			NativeOrWithId::Native => true,
			NativeOrWithId::WithId(_) => false,
		}
	}
}

pub trait NativeId {
	fn get_native_id() -> Self;

	fn is_native_id(&self) -> bool;
}

pub trait VoucherId: Sized {
	/// Make this an option in case that vouchers are not supported.
	fn get_voucher_id() -> Option<Self>;

	fn is_voucher_id(&self) -> bool;
}

impl<AssetId> IdentifyVoucherOrAssetId for WithdrawKind<AssetId>
where
	AssetId: frame_support::traits::tokens::AssetId,
{
	type AssetId = AssetId;

	fn is_voucher(&self) -> bool {
		self == &WithdrawKind::Voucher
	}

	fn as_asset_id(&self) -> Option<&Self::AssetId> {
		match self {
			WithdrawKind::Payment(asset_id) => Some(asset_id),
			WithdrawKind::Voucher => None,
		}
	}
}

pub struct WithdrawCreditOrVoucher<W, V>(PhantomData<(W, V)>);

impl<AccountId, B, W, V> WithdrawCredit for WithdrawCreditOrVoucher<W, V>
where
	B: Balance,
	W: WithdrawCredit<AccountId = AccountId, Balance = B>,
	W::AssetId: 'static,
	V: VoucherHandler<AccountId = AccountId, Balance = B>,
{
	type AccountId = AccountId;
	type AssetId = WithdrawKind<W::AssetId>;
	type Assets = W::Assets;
	type Balance = B;
	type Credit = W::Credit;

	fn withdraw_credit(
		who: &Self::AccountId,
		asset_id: Self::AssetId,
		credit: Self::Balance,
		preservation: Preservation,
	) -> Result<Option<Self::Credit>, DispatchError> {
		match asset_id {
			WithdrawKind::Payment(payment_asset_id) =>
				W::withdraw_credit(who, payment_asset_id, credit, preservation),
			WithdrawKind::Voucher => V::consume_vouchers_from(who, credit).map(|_| None),
		}
	}
}
