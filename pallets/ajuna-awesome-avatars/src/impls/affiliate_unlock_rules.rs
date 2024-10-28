use crate::*;
use frame_support::traits::ExistenceRequirement;
use pallet_ajuna_affiliates::traits::AffiliateUnlockRules;

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Debug, Eq, PartialEq)]
pub struct AffiliateUnlockParams<AccountId> {
	pub target: UnlockTarget<AccountId>,
	pub season_id: SeasonId,
}

impl<T: Config> AffiliateUnlockRules for Pallet<T> {
	type AccountId = AccountIdFor<T>;
	type UnlockParameters = AffiliateUnlockParams<Self::AccountId>;

	fn execute_unlock_rule_for(
		account: &Self::AccountId,
		params: Self::UnlockParameters,
	) -> Result<(), DispatchError> {
		let AffiliateUnlockParams { target, season_id } = params;
		ensure!(Seasons::<T>::contains_key(season_id), Error::<T>::UnknownSeason);

		match target {
			UnlockTarget::OneselfFree => {
				// Check criteria
				let affiliate_unlock = SeasonUnlocks::<T>::get(season_id)
					.map(|config| config.affiliate_unlock.unwrap_or_default())
					.ok_or::<DispatchError>(Error::<T>::FeatureLockedInSeason.into())?;

				let player_stats = SeasonStats::<T>::get(season_id, account);

				if Self::evaluate_unlock_state(&affiliate_unlock, &player_stats) {
					PlayerSeasonConfigs::<T>::mutate(account, season_id, |config| {
						config.locks.affiliate = true;
					});
					Ok(())
				} else {
					Err(Error::<T>::UnlockCriteriaNotFulfilled.into())
				}
			},
			UnlockTarget::OneselfPaying => {
				// Subtract amount for paying if account not affiliator
				PlayerSeasonConfigs::<T>::try_mutate(account, season_id, |config| {
					if !config.locks.affiliate {
						let GlobalConfig { affiliate_config, .. } = GlobalConfigs::<T>::get();
						ensure!(
							affiliate_config.affiliator_enable_fee > 0_u32.into(),
							Error::<T>::FeatureLockedThroughPayment
						);
						T::Currency::transfer(
							account,
							&Self::treasury_account_id(),
							affiliate_config.affiliator_enable_fee,
							ExistenceRequirement::KeepAlive,
						)?;
						config.locks.affiliate = true;
					}
					Ok(())
				})
			},
			UnlockTarget::OtherPaying(other) => {
				// Subtract amount for paying if other not affiliator
				PlayerSeasonConfigs::<T>::try_mutate(&other, season_id, |config| {
					if !config.locks.affiliate {
						let GlobalConfig { affiliate_config, .. } = GlobalConfigs::<T>::get();
						ensure!(
							affiliate_config.affiliator_enable_fee > 0_u32.into(),
							Error::<T>::FeatureLockedThroughPayment
						);
						T::Currency::transfer(
							account,
							&Self::treasury_account_id(),
							affiliate_config.affiliator_enable_fee,
							ExistenceRequirement::KeepAlive,
						)?;
						config.locks.affiliate = true;
					}
					Ok(())
				})
			},
		}
	}
}
