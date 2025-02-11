pub enum TransitionError {
	TransferError,
	FeeError,
	VoucherNotAllowed,
	AssetLength,
	AssetOwnership,
	Transition { code: u8 },
}
