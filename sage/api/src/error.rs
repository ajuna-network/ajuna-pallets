pub enum TransitionError {
	InvalidTransitionId,
	TransferError,
	FeeError,
	AssetLength,
	AssetOwnership,
	Transition { code: u8 },
}
