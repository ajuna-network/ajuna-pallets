use sp_runtime::DispatchError;

/// Abstraction around transferring funds without disclosing the
/// internals of how the funds are managed.
pub trait TransferFunds {
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
