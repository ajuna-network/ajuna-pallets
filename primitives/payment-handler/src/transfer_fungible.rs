use sp_runtime::DispatchError;

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
