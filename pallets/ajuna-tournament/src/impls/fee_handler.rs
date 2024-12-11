use super::*;
use ajuna_primitives::fee_handler::{DistributeFee, Payment};
use sp_arithmetic::traits::{CheckedDiv, Saturating};
use sp_std::vec;

impl<T: Config<I>, I: 'static> DistributeFee for Pallet<T, I> {
	type AccountId = AccountIdFor<T>;
	type Balance = BalanceOf<T, I>;
	type FeeIdentifier = TournamentCategoryIdFor<T, I>;
	type FeeDistribution = Payment<Self::AccountId, Self::Balance>;

	fn distribute_fee(
		base_fee: Self::Balance,
		_account: &Self::AccountId,
		identifier: &Self::FeeIdentifier,
	) -> Option<Self::FeeDistribution> {
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

				Some(Payment::new(tournament_account, tournament_fee))
			},
			_ => None,
		}
	}
}
