//! Example handle fees implementation.
//!
//! The real one is going to be more complex it should handle:
//! * Payment into treasury
//! * Quote USDT price to drive AJUN fee
//! * Handle tournament fees (if configured)
//! * Handle affiliation fees (if configured)

use std::marker::PhantomData;

pub trait HandleFees {
	type Balance;

	fn handle_fees(balance: Self::Balance) -> Result<(), ()>;
}

pub struct FeeHandler<Balance> {
	_phantom: PhantomData<Balance>,
}

impl<Balance> HandleFees for FeeHandler<Balance> {
	type Balance = Balance;

	fn handle_fees(_balance: Self::Balance) -> Result<(), ()> {
		todo!()
	}
}
