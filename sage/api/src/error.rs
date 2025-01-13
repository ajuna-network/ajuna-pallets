pub enum TransitionError {
	InvalidTransitionId,
	TransferError,
	FeeError,
	Transition { error: u8 },
}

pub enum RuleError {
	AssetLength,
	AssetOwnership,
	Other { error: u8 },
}

pub trait AsErrorCode {
	fn as_error_code(&self) -> u8;
}

impl AsErrorCode for u8 {
	fn as_error_code(&self) -> u8 {
		*self
	}
}

impl AsErrorCode for TransitionError {
	fn as_error_code(&self) -> u8 {
		match self {
			TransitionError::InvalidTransitionId => 0,
			TransitionError::TransferError => 1,
			TransitionError::FeeError => 2,
			TransitionError::Transition { error } => *error,
		}
	}
}

impl AsErrorCode for RuleError {
	fn as_error_code(&self) -> u8 {
		match self {
			RuleError::AssetLength => 0,
			RuleError::AssetOwnership => 1,
			RuleError::Other { error } => *error,
		}
	}
}
