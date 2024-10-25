use crate::*;
use pallet_ajuna_affiliates::traits::AffiliateUnlockRules;

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Debug, Eq, PartialEq)]
pub struct AffiliateUnlockParams<AccountId> {
	pub target: UnlockTarget<AccountId>,
	pub season_id: SeasonId,
}

impl<T: Config> AffiliateUnlockRules for Pallet<T> {
	type AccountId = AccountIdFor<T>;
	type UnlockParameters = AffiliateUnlockParams<Self::AccountId>;

	fn try_validate_unlock(
		account: &Self::AccountId,
		params: Self::UnlockParameters,
	) -> Result<Self::AccountId, DispatchError> {
		let AffiliateUnlockParams { target, season_id } = params;
		ensure!(Seasons::<T>::contains_key(season_id), Error::<T>::UnknownSeason);

		match target {
			UnlockTarget::OneselfFree => {
				// Check criteria
				if let Some(UnlockConfigs { affiliate_unlock: Some(unlock_vec), .. }) =
					SeasonUnlocks::<T>::get(season_id)
				{
					let player_stats = SeasonStats::<T>::get(season_id, account);

					if Self::evaluate_unlock_state(&unlock_vec, &player_stats) {
						PlayerSeasonConfigs::<T>::mutate(account, season_id, |config| {
							config.locks.affiliate = true;
						});
						Ok(account.clone())
					} else {
						Err(Error::<T>::UnlockCriteriaNotFulfilled.into())
					}
				} else {
					Err(Error::<T>::FeatureLockedInSeason.into())
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
							AllowDeath,
						)?;
						config.locks.affiliate = true;
					}
					Ok(account.clone())
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
							AllowDeath,
						)?;
						config.locks.affiliate = true;
					}
					Ok(other.clone())
				})
			},
		}
	}
}
