use super::*;
use ajuna_primitives::payment_handler::{AffiliateFeeDistribution, DistributeFee, PaymentFee};
use frame_support::traits::Defensive;
use sp_runtime::{Saturating, traits::CheckedDiv};

impl<T: Config<I>, I: 'static> DistributeFee for Pallet<T, I> {
	type AccountId = AccountIdFor<T>;
	type Balance = BalanceOf<T, I>;
	type FeeIdentifier = RuleIdentifierFor<T, I>;
	type FeeDistribution =
		AffiliateFeeDistribution<Self::AccountId, Self::Balance, T::AffiliateMaxLevel>;

	fn distribute_fee(
		base_fee: Self::Balance,
		account: &Self::AccountId,
		identifier: &Self::FeeIdentifier,
	) -> Option<Self::FeeDistribution> {
		if let Some(chain) = Self::get_affiliator_chain_for(account) {
			if let Some(rule_chain) = Self::get_rule_for(identifier) {
				let payments: Vec<_> = rule_chain
					.into_iter()
					.map(|rule_perc| {
						base_fee
							.saturating_mul(rule_perc.into())
							.checked_div(&100_u32.into())
							.unwrap_or_default()
					})
					.zip(chain)
					.map(|(fee, account)| PaymentFee::new(account, fee))
					.collect();

				Some(payments.try_into().defensive_unwrap_or_default())
			} else {
				None
			}
		} else {
			None
		}
	}
}
