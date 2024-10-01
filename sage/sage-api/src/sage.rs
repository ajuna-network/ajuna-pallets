use crate::{
	handle_fees::HandleFees,
	primitives::{Asset, Error},
};
use std::marker::PhantomData;

/// The idea is that we can aggregate all the functionality of what we consider necessary for one
/// game studio into the `SageEngine` such that it can conveniently be passed around to the game
/// logic's rust implementation.
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

/// The aggregated trait that the `SageEngine` implements such that it can access all features of
/// our pallets.
///
/// This will be much more elaborate in th actual implementation.
pub trait SageApi {
	type Balance;

	type AccountId;

	fn transfer_ownership(asset: Asset, to: Self::AccountId) -> Result<(), Error>;
	fn handle_fees(balance: Self::Balance) -> Result<(), Error>;
}

/// trait covering the core functionalities
///
/// In reality this will be configured in the runtime, and the implementation will get the
/// functionality of our other pallets via associated types.
///
/// ```doc
/// pub struct SageCoreInstance;
/// impl SageCore for SageCoreInstance; {
/// 	type SageSystem: SageSystem,
/// 	type SageCollections: SageCollections,
/// 	type Affiliates: Affiliates,
/// 	type Tournaments: Tournaments,
/// 	...etc.
/// }
/// ```
pub trait SageCore {
	type AccountId;

	fn transfer_ownership(asset: Asset, to: Self::AccountId) -> Result<(), Error>;
}
