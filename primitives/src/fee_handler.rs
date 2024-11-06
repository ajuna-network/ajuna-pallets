use crate::treasury_manager::TreasuryManager;
use frame_support::{
	pallet_prelude::DispatchError,
	sp_runtime::{traits::CheckedSub, ArithmeticError},
	traits::{Currency, ExistenceRequirement::KeepAlive},
	Parameter,
};
use std::marker::PhantomData;

pub trait FeeProvider {
	type AccountId;
	type FeeIdentifier;
	type FeeCurrency;
	type FeeOutput;

	fn get_fee_from(
		base_fee: Self::FeeCurrency,
		account: &Self::AccountId,
		identifier: &Self::FeeIdentifier,
	) -> Self::FeeOutput;
}

pub trait FeeHandler {
	type AccountId;
	type FeeCurrency;
	type AffiliateFeeIdentifier;
	type TournamentFeeIdentifier;
	type TreasuryKey;

	fn try_propagate_chain_fee(
		base_fee: Self::FeeCurrency,
		account: &Self::AccountId,
		identifier: &Self::AffiliateFeeIdentifier,
	) -> Result<Self::FeeCurrency, DispatchError>;

	fn try_propagate_tournament_fee(
		base_fee: Self::FeeCurrency,
		account: &Self::AccountId,
		identifier: &Self::TournamentFeeIdentifier,
	) -> Result<Self::FeeCurrency, DispatchError>;

	fn deposit_fee_into_treasury(
		key: Self::TreasuryKey,
		fee: Self::FeeCurrency,
	) -> Result<(), DispatchError>;
}

pub struct GameFeeHandler<AccountId, Currency, Affiliate, Tournament, Treasury> {
	_phantom: PhantomData<(AccountId, Currency, Affiliate, Tournament, Treasury)>,
}

impl<AccountId, CurrencyHandler, Affiliate, Aid, Tournament, Tid, Treasury> FeeHandler
	for GameFeeHandler<AccountId, CurrencyHandler, Affiliate, Tournament, Treasury>
where
	AccountId: Parameter,
	CurrencyHandler: Currency<AccountId>,
	Affiliate: FeeProvider<
		AccountId = AccountId,
		FeeIdentifier = Aid,
		FeeCurrency = CurrencyHandler::Balance,
		FeeOutput = Vec<(CurrencyHandler::Balance, AccountId)>,
	>,
	Aid: Parameter,
	Tournament: FeeProvider<
		AccountId = AccountId,
		FeeIdentifier = Tid,
		FeeCurrency = CurrencyHandler::Balance,
		FeeOutput = (CurrencyHandler::Balance, AccountId),
	>,
	Tid: Parameter,
	Treasury: TreasuryManager<AccountId = AccountId, Currency = CurrencyHandler::Balance>,
{
	type AccountId = AccountId;
	type FeeCurrency = CurrencyHandler::Balance;
	type AffiliateFeeIdentifier = Aid;
	type TournamentFeeIdentifier = Tid;
	type TreasuryKey = Treasury::TreasuryPotKey;

	fn try_propagate_chain_fee(
		base_fee: Self::FeeCurrency,
		account: &Self::AccountId,
		identifier: &Self::AffiliateFeeIdentifier,
	) -> Result<Self::FeeCurrency, DispatchError> {
		let mut final_fee = base_fee;

		for (transfer_fee, chain_account) in Affiliate::get_fee_from(base_fee, account, identifier)
		{
			if transfer_fee > 0_u32.into() {
				CurrencyHandler::transfer(account, &chain_account, transfer_fee, KeepAlive)?;
				final_fee = final_fee
					.checked_sub(&transfer_fee)
					.ok_or(DispatchError::Arithmetic(ArithmeticError::Underflow))?;
			}
		}

		Ok(final_fee)
	}

	fn try_propagate_tournament_fee(
		base_fee: Self::FeeCurrency,
		account: &Self::AccountId,
		identifier: &Self::TournamentFeeIdentifier,
	) -> Result<Self::FeeCurrency, DispatchError> {
		let (tournament_fee, tournament_account) =
			Tournament::get_fee_from(base_fee, account, identifier);

		if tournament_fee > 0_u32.into() {
			CurrencyHandler::transfer(account, &tournament_account, tournament_fee, KeepAlive)?;
			base_fee
				.checked_sub(&tournament_fee)
				.ok_or(DispatchError::Arithmetic(ArithmeticError::Underflow))
		} else {
			Ok(base_fee)
		}
	}

	fn deposit_fee_into_treasury(
		key: Self::TreasuryKey,
		fee: Self::FeeCurrency,
	) -> Result<(), DispatchError> {
		Treasury::deposit_into(key, fee)
	}
}
