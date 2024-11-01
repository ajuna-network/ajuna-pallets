use crate::primitives::Error;
use std::marker::PhantomData;

pub trait HandleFees {
	type Balance;

	fn handle_fees(balance: Self::Balance) -> Result<(), Error>;
}

pub struct FeeHandler<Balance> {
	_phantom: PhantomData<Balance>,
}

impl<Balance> HandleFees for FeeHandler<Balance> {
	type Balance = Balance;

	fn handle_fees(_balance: Self::Balance) -> Result<(), Error> {
		todo!()
	}
}
