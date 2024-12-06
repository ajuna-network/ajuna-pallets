use frame_support::{pallet_prelude::DispatchError, traits::tokens::Balance};
use parity_scale_codec::{Decode, EncodeLike, MaxEncodedLen};
use scale_info::TypeInfo;
use std::{fmt::Debug, marker::PhantomData};

/// Implements `WithdrawCredit`, but ensures that only whitelisted assets are withdrawn.
pub struct WithdrawWhitelistedCredit<Whitelist, Withdraw>(PhantomData<(Whitelist, Withdraw)>);

pub trait EnsureWhitelistedAsset {
	type AssetId;

	fn ensure_whitelisted(asset_id: &Self::AssetId) -> Result<(), DispatchError>;
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
