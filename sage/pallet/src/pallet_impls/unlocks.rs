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
	pallet::{PlayerStatsOf, SeasonConfigOf},
	AccountIdOf, Config, Error, LockableFeature, Pallet, PlayerSeasonConfigs, PlayerSeasonStats,
	SeasonIdOf, SeasonUnlocks, UnlockConfig, UnlockTarget,
};
use ajuna_primitives::season_manager::SeasonManager;
use frame_support::{
	pallet_prelude::*,
	traits::{Currency, ExistenceRequirement::KeepAlive},
};

impl<T: Config<I>, I: 'static> Pallet<T, I> {
	pub(crate) fn unlock_asset_trading_for(
		account: AccountIdOf<T>,
		target: UnlockTarget<AccountIdOf<T>>,
		season_id: SeasonIdOf<T, I>,
	) -> DispatchResult {
		match target {
			UnlockTarget::OneselfFree => {
				if let Some(unlock_config) =
					SeasonUnlocks::<T, I>::get(&season_id, LockableFeature::TradeAsset)
				{
					let player_stats = PlayerSeasonStats::<T, I>::get(&account, &season_id);

					if Self::evaluate_unlock_state(&unlock_config, &player_stats) {
						PlayerSeasonConfigs::<T, I>::mutate(&account, &season_id, |config| {
							config.locks.asset_trade = true;
						});

						Ok(())
					} else {
						Err(Error::<T, I>::UnlockCriteriaNotFulfilled.into())
					}
				} else {
					Err(Error::<T, I>::FeatureLockedInSeason.into())
				}
			},
			UnlockTarget::OneselfPaying => {
				PlayerSeasonConfigs::<T, I>::try_mutate(&account, &season_id, |config| {
					if !config.locks.asset_trade {
						let SeasonConfigOf::<T, I> { fee, .. } =
							T::SeasonHandler::get_season_config_for(&season_id)?;
						// TODO: Should be handled by FeeHandler

						/*T::Currency::transfer(
							&account,
							&Self::treasury_account_id(),
							fee.unlock_trade_asset,
							KeepAlive,
						)?;*/
						config.locks.asset_trade = true;
					}
					Ok(())
				})
			},
			UnlockTarget::OtherPaying(other) => {
				PlayerSeasonConfigs::<T, I>::try_mutate(&other, &season_id, |config| {
					if !config.locks.asset_trade {
						let SeasonConfigOf::<T, I> { fee, .. } =
							T::SeasonHandler::get_season_config_for(&season_id)?;
						// TODO: Should be handled by FeeHandler
						/*T::Currency::transfer(
							&account,
							&Self::treasury_account_id(),
							fee.unlock_trade_asset,
							KeepAlive,
						)?;*/
						config.locks.asset_trade = true;
					}
					Ok(())
				})
			},
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

					if Self::evaluate_unlock_state(&unlock_config, &player_stats) {
						PlayerSeasonConfigs::<T, I>::mutate(&account, &season_id, |config| {
							config.locks.asset_transfer = true;
						});

						Ok(())
					} else {
						Err(Error::<T, I>::UnlockCriteriaNotFulfilled.into())
					}
				} else {
					Err(Error::<T, I>::FeatureLockedInSeason.into())
				}
			},
			UnlockTarget::OneselfPaying => {
				PlayerSeasonConfigs::<T, I>::try_mutate(&account, &season_id, |config| {
					if !config.locks.asset_trade {
						// TODO: Should be handled by FeeHandler
						let SeasonConfigOf::<T, I> { fee, .. } =
							T::SeasonHandler::get_season_config_for(&season_id)?;
						// TODO: Should be handled by FeeHandler
						/*T::Currency::transfer(
							&account,
							&Self::treasury_account_id(),
							fee.unlock_transfer_asset,
							AllowDeath,
						)?;*/
						config.locks.asset_transfer = true;
					}
					Ok(())
				})
			},
			UnlockTarget::OtherPaying(other) => {
				PlayerSeasonConfigs::<T, I>::try_mutate(&other, &season_id, |config| {
					if !config.locks.asset_trade {
						let SeasonConfigOf::<T, I> { fee, .. } =
							T::SeasonHandler::get_season_config_for(&season_id)?;
						// TODO: Should be handled by FeeHandler
						/*T::Currency::transfer(
							&account,
							&Self::treasury_account_id(),
							fee.unlock_transfer_asset,
							AllowDeath,
						)?;*/
						config.locks.asset_transfer = true;
					}
					Ok(())
				})
			},
		}
	}

	fn evaluate_unlock_state(config: &UnlockConfig, account_stats: &PlayerStatsOf<T>) -> bool {
		let minted = u8::try_from(account_stats.minted_amount).unwrap_or(u8::MAX);
		//let free_minted = u8::try_from(account_stats.free_minted).unwrap_or(u8::MAX);
		let forged = u8::try_from(account_stats.forged_amount).unwrap_or(u8::MAX);
		let bought = u8::try_from(account_stats.bought_amount).unwrap_or(u8::MAX);
		let sold = u8::try_from(account_stats.sold_amount).unwrap_or(u8::MAX);

		config[0] <= minted &&
			//config[1] <= free_minted &&
			config[2] <= forged &&
			config[3] <= bought &&
			config[4] <= sold
	}
}
