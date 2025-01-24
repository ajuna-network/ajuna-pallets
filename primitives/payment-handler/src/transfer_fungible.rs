use sp_std::marker::PhantomData;
use frame_support::traits::fungibles;
use frame_support::traits::tokens::{Fortitude, Preservation};
use sp_runtime::DispatchError;
use crate::{AssetGameFeeHandler, IdentifyVoucherOrAssetId, WithdrawCredit};

/// Abstraction around transferring funds without disclosing the
/// internals of how the funds are managed.
///
/// The main benefit of this trait is that the uses doesn't have
/// to care if the fungible or the fungibles is used behind the
/// scenes. The `AssetId` will be `()` for the fungible
/// implementation.
pub trait TransferFungible {
    type AccountId;

    type AssetId;

    type Balance;

    fn transfer(
        asset_id: Self::AssetId,
        from: &Self::AccountId,
        to: &Self::AccountId,
        amount: Self::Balance,
    ) -> Result<(), DispatchError>;

    fn transfer_all(
        asset_id: Self::AssetId,
        from: &Self::AccountId,
        to: &Self::AccountId,
    ) -> Result<(), DispatchError>;
}

pub struct TransferAll<W, I>(PhantomData<(W, I)>);

impl<AccountId, Assets, W, I> TransferFungible
for TransferAll<W, I>
where
    Assets: fungibles::Balanced<AccountId, Balance=W::Balance>
    + fungibles::Inspect<AccountId, Balance=W::Balance, AssetId=I::AssetId>,
    W: WithdrawCredit<
        AccountId=AccountId,
        Assets=Assets,
        Credit=fungibles::Credit<AccountId, Assets>,
        AssetId=I,
    >,
    I: IdentifyVoucherOrAssetId
{
    type AccountId = AccountId;
    type AssetId = W::AssetId;
    type Balance = W::Balance;

    fn transfer(
        asset_id: Self::AssetId,
        from: &Self::AccountId,
        to: &Self::AccountId,
        amount: Self::Balance,
    ) -> Result<(), DispatchError> {
        todo!()
    }

    fn transfer_all(
        asset_id: Self::AssetId,
        from: &Self::AccountId,
        to: &Self::AccountId,
    ) -> Result<(), DispatchError> {
        let balance = W::Assets::reducible_balance(
            asset_id.as_asset_id().unwrap().clone(),
            from,
            Preservation::Preserve,
            Fortitude::Polite,
        );

        Self::transfer(asset_id, from, to, balance)
    }
}

#[cfg(test)]
mod tests {
    use crate::mock::{AssetId, Assets, ExtBuilder, MockVoucherHandler, Test, WithdrawWhitelistedAssets, ALICE, FERDIE, WHITELISTED_ASSET_ID, WHITELISTED_ASSET_ID_PAYMENT};
    use crate::{WithdrawCreditOrVoucher, WithdrawKind};
    use super::*;

    type TestTransfer = TransferAll<WithdrawCreditOrVoucher<WithdrawWhitelistedAssets, MockVoucherHandler>, WithdrawKind<AssetId>>;


    #[test]
    fn transfer_works() {
        ExtBuilder::default().build().execute_with(|| {
            let fee = 20;
            let alice_balance_before = Assets::balance(WHITELISTED_ASSET_ID, ALICE);
            let fee_beneficiary = FERDIE;

            TestTransfer::transfer(
                WHITELISTED_ASSET_ID_PAYMENT,
                &ALICE,
                &fee_beneficiary,
                fee,
            )
                .unwrap();

            assert_eq!(
                Assets::balance(WHITELISTED_ASSET_ID, ALICE),
                alice_balance_before - fee
            );
            assert_eq!(Assets::balance(WHITELISTED_ASSET_ID, fee_beneficiary), fee)
        });
    }
}
