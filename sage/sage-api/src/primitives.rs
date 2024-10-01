use crate::handle_fees::HandleFees;
use frame_support::pallet_prelude::TypeInfo;
use sp_core::{Decode, Encode, MaxEncodedLen};
use std::marker::PhantomData;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub struct Asset {
	pub collection_id: u32,

	pub asset_type: u32,

	pub asset_sub_type: u32,

	pub dna: [u8; 32],

	pub minted_at: u32,
}

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

pub enum Error {
	InvalidTransitionId,
	InvalidAssetLength,
	TransferError,
	FeeError,
}

/// trait covering the core functionalities
pub trait SageCore {
	type AccountId;

	fn transfer_ownership(asset: Asset, to: Self::AccountId) -> Result<(), Error>;
}
