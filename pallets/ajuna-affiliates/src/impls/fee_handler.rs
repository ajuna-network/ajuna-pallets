use super::*;
use ajuna_primitives::fee_handler::{DistributeFee, Payment};
use frame_support::traits::Defensive;
use sp_runtime::{traits::CheckedDiv, Saturating};

impl<T: Config<I>, I: 'static> DistributeFee for Pallet<T, I> {
	type AccountId = AccountIdFor<T>;
	type FeeIdentifier = RuleIdentifierFor<T, I>;
	type Balance = BalanceOf<T, I>;
	type MaxDistributions = AffiliateMaxLevelFor<T, I>;

	fn distribute_fee(
		base_fee: Self::Balance,
		account: &Self::AccountId,
		identifier: &Self::FeeIdentifier,
	) -> Option<BoundedVec<Payment<Self::AccountId, Self::Balance>, Self::MaxDistributions>> {
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
					.map(|(fee, account)| Payment::new(account, fee))
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
