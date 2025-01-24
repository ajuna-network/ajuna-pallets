use frame_support::traits::tokens::AssetId;
use sp_runtime::DispatchError;

/// Abstraction of withdrawing 'vouchers' from an account, can be used as
/// alternate payment method.
pub trait VoucherHandler {
	type AccountId;

	type Balance;

	fn consume_vouchers_from(
		account: &Self::AccountId,
		amount: Self::Balance,
	) -> Result<(), DispatchError>;
}

pub trait IdentifyVoucherOrAssetId {
	type AssetId: AssetId;

	fn is_voucher(&self) -> bool;

	fn as_asset_id(&self) -> Option<&Self::AssetId>;
}