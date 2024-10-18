use crate::AsErrorCode;

pub enum Error {
	InvalidTransitionId,
	InvalidAssetLength,
	TransferError,
	FeeError,
}

impl AsErrorCode for Error {
	fn as_error_code(&self) -> u8 {
		use Error::*;
		match self {
			InvalidTransitionId => 0,
			InvalidAssetLength => 1,
			TransferError => 2,
			FeeError => 3,
		}
	}
}
