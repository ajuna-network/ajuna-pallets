//! Benchmark the multi-block-migration.

#![cfg(any(test, feature = "runtime-benchmarks"))]

use crate::{
	migration::{
		v6,
		v6::{weights, weights::WeightInfo},
	},
	Config, Pallet,
};
use frame_benchmarking::v2::*;
use frame_support::{migrations::SteppedMigration, weights::WeightMeter};

fn account<T: Config>(name: &'static str) -> T::AccountId {
	let index = 0;
	let seed = 0;
	frame_benchmarking::account(name, index, seed)
}

#[benchmarks]
mod benches {
	use super::*;
	use crate::{AvatarIdOf, TradeStatsMap};
	use frame_system::pallet_prelude::BlockNumberFor;

	/// Benchmark a single step of the `v1::LazyMigrationV1` migration.
	#[benchmark]
	fn player_season_configs_step() {
		let test_account = account::<T>("test-account");
		let season_id = 1;
		let old_player_season_config = v6::v5::PlayerSeasonConfigV5::<BlockNumberFor<T>>::default();

		v6::v5::PlayerSeasonConfigs::<T>::insert(
			&test_account,
			season_id,
			old_player_season_config,
		);
		let mut meter = WeightMeter::new();

		#[block]
		{
			v6::mbm::LazyMigrationPlayerSeasonConfigsV5ToV6::<T, weights::SubstrateWeight<T>>::step(None, &mut meter).unwrap();
		}

		// Check that the new storage is decodable:
		assert_eq!(
			crate::PlayerSeasonConfigs::<T>::get(&test_account, season_id),
			crate::PlayerSeasonConfig::default()
		);

		// uses twice the weight once for migration and then for checking if there is another key.
		assert_eq!(meter.consumed(), weights::SubstrateWeight::<T>::step() * 2);
	}

	#[benchmark]
	fn season_stats_step() {
		let season_id = 1;
		let test_account = account::<T>("test-account");
		let old_season_info = v6::v5::SeasonInfoV5::default();
		let trade_stats = (1, 2);

		v6::v5::SeasonStats::<T>::insert(season_id, &test_account, old_season_info);

		// Worst case scenario is when we have trade stats for the given key.
		TradeStatsMap::<T>::insert(season_id, &test_account, trade_stats);

		let mut meter = WeightMeter::new();

		#[block]
		{
			v6::mbm::LazyMigrationSeasonStatsV5ToV6::<T, weights::SubstrateWeight<T>>::step(
				None, &mut meter,
			)
			.unwrap();
		}

		assert_eq!(
			crate::SeasonStats::<T>::get(season_id, &test_account),
			crate::SeasonInfo { bought: 1, sold: 2, ..Default::default() }
		);

		// uses twice the weight once for migration and then for checking if there is another key.
		assert_eq!(meter.consumed(), weights::SubstrateWeight::<T>::step() * 2);
	}

	#[benchmark]
	fn avatar_step() {
		let test_account = account::<T>("test-account");
		let old_avatar = v6::v5::AvatarV5::default();
		let avatar_id = AvatarIdOf::<T>::default();

		v6::v5::Avatars::<T>::insert(&avatar_id, (test_account.clone(), old_avatar.clone()));

		let mut meter = WeightMeter::new();

		#[block]
		{
			v6::mbm::LazyMigrationAvatarV5ToV6::<T, weights::SubstrateWeight<T>>::step(
				None, &mut meter,
			)
			.unwrap();
		}

		assert_eq!(
			crate::Avatars::<T>::get(&avatar_id),
			Some((
				test_account,
				crate::Avatar::<BlockNumberFor<T>> {
					season_id: old_avatar.season_id,
					encoding: old_avatar.encoding,
					dna: old_avatar.dna,
					souls: old_avatar.souls,
					minted_at: BlockNumberFor::<T>::default(),
				}
			))
		);

		// uses twice the weight once for migration and then for checking if there is another key.
		assert_eq!(meter.consumed(), weights::SubstrateWeight::<T>::step() * 2);
	}

	#[benchmark]
	fn trade_stats_map_cleanup_step() {
		let season_id = 1;
		let test_account = account::<T>("test-account");
		let trade_stats = (1, 2);

		TradeStatsMap::<T>::insert(season_id, &test_account, trade_stats);

		let mut meter = WeightMeter::new();

		#[block]
		{
			v6::mbm::LazyTradeStatsMapCleanup::<T, weights::SubstrateWeight<T>>::step(
				None, &mut meter,
			)
			.unwrap();
		}

		assert!(!TradeStatsMap::<T>::contains_key(season_id, &test_account));

		// uses twice the weight once for migration and then for checking if there is another key.
		assert_eq!(meter.consumed(), weights::SubstrateWeight::<T>::step() * 2);
	}

	impl_benchmark_test_suite!(Pallet, new_test_ext(), crate::mock::Test);
}

#[cfg(test)]
pub fn new_test_ext() -> sp_io::TestExternalities {
	use sp_runtime::BuildStorage;

	let t = frame_system::GenesisConfig::<crate::mock::Test>::default()
		.build_storage()
		.unwrap();
	sp_io::TestExternalities::new(t)
}
