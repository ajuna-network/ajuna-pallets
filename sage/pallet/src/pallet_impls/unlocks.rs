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
	pallet::PlayerStatsOf, AccountIdOf, Config, Error, Event, FungiblesAssetIdOf, LockableFeature,
	Pallet, PlayerSeasonConfigs, PlayerSeasonStats, SeasonIdOf, SeasonUnlocks, UnlockRule,
	UnlockTarget,
};

use ajuna_primitives::{payment_handler::FeeHandler, season_manager::SeasonManager};

use frame_support::pallet_prelude::*;

impl<T: Config<I>, I: 'static> Pallet<T, I> {
	pub(crate) fn unlock_asset_trading_for(
		account: AccountIdOf<T>,
		target: UnlockTarget<AccountIdOf<T>>,
		season_id: SeasonIdOf<T, I>,
		payment: FungiblesAssetIdOf<T, I>,
	) -> DispatchResult {
		Self::unlock(account, target, season_id, LockableFeature::TradeAsset, payment)
	}

	pub(crate) fn unlock_asset_transfer_for(
		account: AccountIdOf<T>,
		target: UnlockTarget<AccountIdOf<T>>,
		season_id: SeasonIdOf<T, I>,
		payment: FungiblesAssetIdOf<T, I>,
	) -> DispatchResult {
		Self::unlock(account, target, season_id, LockableFeature::TransferAsset, payment)
	}

	fn unlock(
		account: AccountIdOf<T>,
		target: UnlockTarget<AccountIdOf<T>>,
		season_id: SeasonIdOf<T, I>,
		feature: LockableFeature,
		payment: FungiblesAssetIdOf<T, I>,
	) -> DispatchResult {
		match target {
			UnlockTarget::OneselfFree => Self::unlock_free(account, season_id, feature),
			UnlockTarget::OneselfPaying =>
				Self::unlock_paying(account.clone(), account, season_id, feature, payment),
			UnlockTarget::OtherPaying(other) =>
				Self::unlock_paying(account, other, season_id, feature, payment),
		}
	}

	fn unlock_free(
		account: AccountIdOf<T>,
		season_id: SeasonIdOf<T, I>,
		feature: LockableFeature,
	) -> DispatchResult {
		let unlock_config = SeasonUnlocks::<T, I>::get(&season_id, feature)
			.ok_or(Error::<T, I>::FeatureUnavailableInSeason)?;

		let player_stats = PlayerSeasonStats::<T, I>::get(&account, &season_id);

		if !Self::evaluate_unlock_state(&unlock_config, &player_stats) {
			return Err(Error::<T, I>::UnlockCriteriaNotFulfilled.into())
		}

		// after evaluating, we can enable the feature
		Self::enable_feature_in_config(&account, &season_id, feature);

		Self::deposit_event(Event::FeatureUnlocked { feature, season_id, account });
		Ok(())
	}

	fn unlock_paying(
		payer: AccountIdOf<T>,
		target: AccountIdOf<T>,
		season_id: SeasonIdOf<T, I>,
		feature: LockableFeature,
		payment: FungiblesAssetIdOf<T, I>,
	) -> DispatchResult {
		// first we pay
		let fee = T::SeasonHandler::get_season_config_for(&season_id)?.fee;
		let feature_fee = match feature {
			LockableFeature::TradeAsset => fee.unlock_trade_asset,
			LockableFeature::TransferAsset => fee.unlock_transfer_asset,
		};
		T::FeeHandler::withdraw_and_deposit_into(
			&payer,
			payment,
			&Self::treasury_account_id(),
			feature_fee,
		)?;

		// after payment, we can enable the feature
		Self::enable_feature_in_config(&target, &season_id, feature);

		Self::deposit_event(Event::FeatureUnlocked { feature, season_id, account: target });
		Ok(())
	}

	fn enable_feature_in_config(
		account: &AccountIdOf<T>,
		season_id: &SeasonIdOf<T, I>,
		feature: LockableFeature,
	) {
		PlayerSeasonConfigs::<T, I>::mutate(account, season_id, |config| {
			let feature_lock = match feature {
				LockableFeature::TradeAsset => &mut config.locks.asset_trade,
				LockableFeature::TransferAsset => &mut config.locks.asset_transfer,
			};

			*feature_lock = true;
		});
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
