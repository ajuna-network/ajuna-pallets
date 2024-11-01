use super::*;
use ajuna_primitives::fee_handler::FeeHandler;

impl<T: Config<I>, I: 'static> FeeHandler for Pallet<T, I> {
	type AccountId = AccountIdFor<T>;
	type FeeIdentifier = RuleIdentifierFor<T, I>;
	type FeeCurrency = BalanceOf<T, I>;

	fn get_fee(account: &Self::AccountId, identifier: &Self::FeeIdentifier) -> Self::FeeCurrency {
		todo!()
	}
}
