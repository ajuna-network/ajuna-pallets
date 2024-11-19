pub enum Error {
	InvalidTransitionId,
	InvalidAssetLength,
	TransferError,
	FeeError,
	Transition { error: u8 },
}

pub trait AsErrorCode {
	fn as_error_code(&self) -> u8;
}

impl AsErrorCode for u8 {
	fn as_error_code(&self) -> u8 {
		*self
	}
}

impl AsErrorCode for Error {
	fn as_error_code(&self) -> u8 {
		match self {
			Error::InvalidTransitionId => 0,
			Error::InvalidAssetLength => 1,
			Error::TransferError => 2,
			Error::FeeError => 3,
			Error::Transition { error } => *error,
		}
	}
}
