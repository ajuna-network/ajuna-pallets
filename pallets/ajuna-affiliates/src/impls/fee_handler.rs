use super::*;
use ajuna_primitives::fee_handler::FeeProvider;
use sp_runtime::{traits::CheckedDiv, Saturating};

impl<T: Config<I>, I: 'static> FeeProvider for Pallet<T, I> {
	type AccountId = AccountIdFor<T>;
	type FeeIdentifier = RuleIdentifierFor<T, I>;
	type FeeCurrency = BalanceOf<T, I>;
	type FeeOutput = Vec<(Self::FeeCurrency, Self::AccountId)>;

	fn get_fee_from(
		base_fee: Self::FeeCurrency,
		account: &Self::AccountId,
		identifier: &Self::FeeIdentifier,
	) -> Self::FeeOutput {
		if let Some(chain) = Self::get_affiliator_chain_for(account) {
			if let Some(rule_chain) = Self::get_rule_for(identifier) {
				rule_chain
					.into_iter()
					.map(|rule_perc| {
						base_fee
							.saturating_mul(rule_perc.into())
							.checked_div(&100_u32.into())
							.unwrap_or_default()
					})
					.zip(chain)
					.collect()
			} else {
				Vec::with_capacity(0)
			}
		} else {
			Vec::with_capacity(0)
		}
	}
}
