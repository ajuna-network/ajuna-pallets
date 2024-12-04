use frame_support::{
	pallet_prelude::DispatchError,
	traits::{
		fungibles::{Balanced, Credit},
		tokens::Balance,
	},
};
use parity_scale_codec::{Decode, Encode, EncodeLike, MaxEncodedLen};
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

	fn withdraw_credit(
		who: &Self::AccountId,
		asset_id: Self::AssetId,
		credit: Self::Balance,
	) -> Result<Credit<Self::AccountId, Self::Assets>, DispatchError> {
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

	type Assets: Balanced<Self::AccountId>;
	type Balance: Balance;

	fn withdraw_credit(
		who: &Self::AccountId,
		asset_id: Self::AssetId,
		credit: Self::Balance,
	) -> Result<Credit<Self::AccountId, Self::Assets>, DispatchError>;
}

pub type CreditOf<T> = Credit<<T as WithdrawCredit>::AccountId, <T as WithdrawCredit>::Assets>;

// #[cfg(test)]
// mod tests {
// 	use super::*;
// 	use frame_support::sp_runtime::TokenError;
//
// 	struct AlwaysAllowWithdraw;
// 	struct AlwaysDenyWithdraw;
//
// 	impl EnsureWhitelistedAsset for AlwaysAllowWithdraw {
// 		type AssetId = u8;
//
// 		fn ensure_whitelisted(_asset_id: &Self::AssetId) -> Result<(), DispatchError> {
// 			Ok(())
// 		}
// 	}
//
// 	impl EnsureWhitelistedAsset for AlwaysDenyWithdraw {
// 		type AssetId = u8;
//
// 		fn ensure_whitelisted(_asset_id: &Self::AssetId) -> Result<(), DispatchError> {
// 			Err(DispatchError::Token(TokenError::Unsupported))
// 		}
// 	}
//
// 	struct MockWithdraw;
//
// 	impl WithdrawCredit for MockWithdraw {
// 		type AccountId = u8;
// 		type AssetId = u8;
// 		type Assets = u8;
// 		type Balance = u8;
//
// 		fn withdraw_credit(
// 			_who: &Self::AccountId,
// 			_asset_id: Self::AssetId,
// 			credit: Self::Balance,
// 		) -> Result<Credit<Self::AccountId, Self::Assets>, DispatchError> {
// 			Ok(Credit::try_from(credit).unwrap())
// 		}
// 	}
//
// 	#[test]
// 	fn can_withdraw_whitelisted_asset() {
// 		let credit = 2;
//
// 		assert_eq!(
// 			WithdrawWhitelistedCredit::<AlwaysAllowWithdraw, MockWithdraw>::withdraw_credit(
// 				&1u8, 1, credit
// 			),
// 			Ok(credit)
// 		)
// 	}
//
// 	#[test]
// 	fn cannot_withdraw_whitelisted_asset() {
// 		let credit = 2;
//
// 		assert_eq!(
// 			WithdrawWhitelistedCredit::<AlwaysDenyWithdraw, MockWithdraw>::withdraw_credit(
// 				&1u8, 1, credit
// 			),
// 			Err(DispatchError::Token(TokenError::Unsupported))
// 		)
// 	}
// }
