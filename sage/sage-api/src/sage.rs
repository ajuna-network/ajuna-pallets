use crate::{
	handle_fees::HandleFees,
	primitives::{Asset, Error},
};
use std::marker::PhantomData;

pub struct SageEngine<SageCore, FeeHandler> {
	_phantom: PhantomData<(SageCore, FeeHandler)>,
}

impl<Core: SageCore, FeeHandler: HandleFees> SageApi for SageEngine<Core, FeeHandler> {
	type Balance = FeeHandler::Balance;

	type AccountId = Core::AccountId;

	fn transfer_ownership(asset: Asset, to: Self::AccountId) -> Result<(), Error> {
		Core::transfer_ownership(asset, to)
	}

	fn handle_fees(balance: Self::Balance) -> Result<(), Error> {
		FeeHandler::handle_fees(balance)
	}
}

pub trait SageApi {
	type Balance;

	type AccountId;

	fn transfer_ownership(asset: Asset, to: Self::AccountId) -> Result<(), Error>;
	fn handle_fees(balance: Self::Balance) -> Result<(), Error>;
}

/// trait covering the core functionalities
pub trait SageCore {
	type AccountId;

	fn transfer_ownership(asset: Asset, to: Self::AccountId) -> Result<(), Error>;
}
