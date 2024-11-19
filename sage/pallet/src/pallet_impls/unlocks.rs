// Ajuna Node
// Copyright (C) 2022 BlogaTech AG

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use crate::{
	pallet::PlayerStatsOf, AccountIdOf, Config, Error, Event, LockableFeature, Pallet,
	PlayerSeasonConfigs, PlayerSeasonStats, SeasonIdOf, SeasonUnlocks, UnlockRule, UnlockTarget,
};
use ajuna_primitives::{fee_handler::FeeHandler, season_manager::SeasonManager};
use frame_support::pallet_prelude::*;

impl<T: Config<I>, I: 'static> Pallet<T, I> {
	pub(crate) fn unlock_asset_trading_for(
		account: AccountIdOf<T>,
		target: UnlockTarget<AccountIdOf<T>>,
		season_id: SeasonIdOf<T, I>,
	) -> DispatchResult {
		match target {
			UnlockTarget::OneselfFree => {
				// TODO: Is this naming correct?
				let unlock_config =
					SeasonUnlocks::<T, I>::get(&season_id, LockableFeature::TradeAsset)
						.ok_or(Error::<T, I>::FeatureLockedInSeason)?;

				let player_stats = PlayerSeasonStats::<T, I>::get(&account, &season_id);

				PlayerSeasonConfigs::<T, I>::try_mutate(&account, &season_id, |config| {
					if config.locks.asset_trade {
						// early return if already unlocked
						return Ok(());
					}

					if Self::evaluate_unlock_state(&unlock_config, &player_stats) {
						config.locks.asset_trade = true;

						Self::deposit_event(Event::FeatureUnlocked {
							feature: LockableFeature::TradeAsset,
							season_id: season_id.clone(),
							account: account.clone(),
						});

						Ok(())
					} else {
						Err(Error::<T, I>::UnlockCriteriaNotFulfilled.into())
					}
				})
			},
			UnlockTarget::OneselfPaying => Self::unlock_paying(
				account.clone(),
				account,
				season_id,
				LockableFeature::TradeAsset,
			),
			UnlockTarget::OtherPaying(other) =>
				Self::unlock_paying(account, other, season_id, LockableFeature::TradeAsset),
		}
	}

	pub(crate) fn unlock_asset_transfer_for(
		account: AccountIdOf<T>,
		target: UnlockTarget<AccountIdOf<T>>,
		season_id: SeasonIdOf<T, I>,
	) -> DispatchResult {
		match target {
			UnlockTarget::OneselfFree => {
				if let Some(unlock_config) =
					SeasonUnlocks::<T, I>::get(&season_id, LockableFeature::TransferAsset)
				{
					let player_stats = PlayerSeasonStats::<T, I>::get(&account, &season_id);

					PlayerSeasonConfigs::<T, I>::try_mutate(&account, &season_id, |config| {
						if !config.locks.asset_transfer {
							if Self::evaluate_unlock_state(&unlock_config, &player_stats) {
								config.locks.asset_transfer = true;

								Self::deposit_event(Event::FeatureUnlocked {
									feature: LockableFeature::TransferAsset,
									season_id: season_id.clone(),
									account: account.clone(),
								});

								Ok(())
							} else {
								Err(Error::<T, I>::UnlockCriteriaNotFulfilled.into())
							}
						} else {
							Ok(())
						}
					})
				} else {
					// TODO: Is this naming correct?
					Err(Error::<T, I>::FeatureLockedInSeason.into())
				}
			},
			UnlockTarget::OneselfPaying => Self::unlock_paying(
				account.clone(),
				account,
				season_id,
				LockableFeature::TransferAsset,
			),
			UnlockTarget::OtherPaying(other) =>
				Self::unlock_paying(account, other, season_id, LockableFeature::TransferAsset),
		}
	}

	fn unlock_paying(
		payer: AccountIdOf<T>,
		target: AccountIdOf<T>,
		season_id: SeasonIdOf<T, I>,
		feature: LockableFeature,
	) -> DispatchResult {
		PlayerSeasonConfigs::<T, I>::try_mutate(&target, &season_id, |config| {
			let feature_lock = match feature {
				LockableFeature::TradeAsset => &mut config.locks.asset_trade,
				LockableFeature::TransferAsset => &mut config.locks.asset_transfer,
			};

			if *feature_lock {
				// early return if already unlocked
				return Ok(());
			}

			let fee = T::SeasonHandler::get_season_config_for(&season_id)?.fee;
			let feature_fee = match feature {
				LockableFeature::TradeAsset => fee.unlock_trade_asset,
				LockableFeature::TransferAsset => fee.unlock_transfer_asset,
			};

			T::FeeHandler::deposit_fee_into_treasury(&payer, &season_id, feature_fee)?;
			*feature_lock = true;

			Ok::<(), DispatchError>(())
		})?;

		Self::deposit_event(Event::FeatureUnlocked { feature, season_id, account: target });
		Ok(())
	}

	fn evaluate_unlock_state(config: &UnlockRule, account_stats: &PlayerStatsOf<T>) -> bool {
		let minted = u8::try_from(account_stats.minted_amount).unwrap_or(u8::MAX);
		//let free_minted = u8::try_from(account_stats.free_minted).unwrap_or(u8::MAX);
		let forged = u8::try_from(account_stats.forged_amount).unwrap_or(u8::MAX);
		let bought = u8::try_from(account_stats.bought_amount).unwrap_or(u8::MAX);
		let sold = u8::try_from(account_stats.sold_amount).unwrap_or(u8::MAX);

		if config.len() != 5 {
			return false
		}

		config[0] <= minted &&
			//config[1] <= free_minted &&
			config[2] <= forged &&
			config[3] <= bought &&
			config[4] <= sold
	}
}
