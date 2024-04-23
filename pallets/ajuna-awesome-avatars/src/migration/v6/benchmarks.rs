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
use sp_runtime::BuildStorage;

fn account<T: Config>(name: &'static str) -> T::AccountId {
	let index = 0;
	let seed = 0;
	frame_benchmarking::account(name, index, seed)
}

#[benchmarks]
mod benches {
	use super::*;
	use frame_system::pallet_prelude::BlockNumberFor;

	/// Benchmark a single step of the `v1::LazyMigrationV1` migration.
	#[benchmark]
	fn step() {
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

	impl_benchmark_test_suite!(Pallet, new_test_ext(), crate::mock::Test);
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let t = frame_system::GenesisConfig::<crate::mock::Test>::default()
		.build_storage()
		.unwrap();
	sp_io::TestExternalities::new(t)
}
