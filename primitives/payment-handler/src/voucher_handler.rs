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
