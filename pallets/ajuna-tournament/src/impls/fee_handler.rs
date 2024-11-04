use super::*;
use ajuna_primitives::fee_handler::FeeProvider;
use sp_arithmetic::traits::{CheckedDiv, Saturating};

impl<T: Config<I>, I: 'static> FeeProvider for Pallet<T, I> {
	type AccountId = AccountIdFor<T>;
	type FeeIdentifier = TournamentCategoryIdFor<T, I>;
	type FeeCurrency = BalanceOf<T, I>;
	type FeeOutput = (BalanceOf<T, I>, Self::AccountId);

	fn get_fee_from(
		base_fee: Self::FeeCurrency,
		_account: &Self::AccountId,
		identifier: &Self::FeeIdentifier,
	) -> Self::FeeOutput {
		let is_tournament_in_active_period = matches!(
			Self::get_active_tournament_state_for(identifier),
			TournamentState::ActivePeriod(_)
		);

		let tournament_account = Self::get_treasury_account_for(identifier);

		match Self::get_active_tournament_config_for(identifier) {
			Some((_, TournamentConfig { take_fee_percentage: Some(fee_perc), .. }))
				if is_tournament_in_active_period =>
			{
				let tournament_fee = base_fee
					.saturating_mul(fee_perc.into())
					.checked_div(&100_u32.into())
					.unwrap_or_default();

				(tournament_fee, tournament_account)
			},
			_ => (0_u32.into(), tournament_account),
		}
	}
}
