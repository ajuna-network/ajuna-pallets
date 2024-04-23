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

//! Migrations for the AAA pallet version 5 to version 6.
//!
//! It consists of a classical single block migration and several multiblock migrations (mbm).
//!
//! ** All migrations have to be employed in a single runtime upgrade! **
//!
//! Additionally, the order of the mbm matters, which is further elaborated in the `mbm` module
//! docs.

use super::{OnRuntimeUpgrade, LOG_TARGET};
use crate::*;
use frame_support::{
	migrations::{SteppedMigration, SteppedMigrationError},
	weights::WeightMeter,
};

pub mod benchmarks;
pub mod weights;

pub use weights::WeightInfo;

mod v5 {
	use crate::{
		pallet::BalanceOf,
		types::{
			AffiliateConfig, Avatar, AvatarTransferConfig, Dna, DnaEncoding, Fee, ForgeConfig,
			FreeMintTransferMode, FreemintTransferConfig, GlobalConfig, Locks, LogicGeneration,
			MaxSeasons, MintConfig, MintCount, MintFees, NftTransferConfig, PlayStats,
			PlayerSeasonConfig, RarityPercent, RarityTier, SacrificeCount, Season, SeasonId,
			SeasonInfo, SoulCount, Stat, Stats, StorageTier, TradeConfig, TradeFilter,
		},
		AvatarIdOf, Config, Pallet,
	};
	use frame_support::{
		pallet_prelude::{ConstU32, Decode, Encode, MaxEncodedLen, OptionQuery, TypeInfo},
		storage_alias, BoundedBTreeSet, BoundedVec, Identity,
	};
	use frame_system::pallet_prelude::BlockNumberFor;

	#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Debug, Default, PartialEq)]
	pub struct MintConfigV5<T: Config> {
		pub open: bool,
		pub cooldown: BlockNumberFor<T>,
		pub free_mint_fee_multiplier: MintCount,
	}

	impl<T> MintConfigV5<T>
	where
		T: Config,
	{
		pub fn migrate_to_v6(self) -> MintConfig<BlockNumberFor<T>> {
			MintConfig {
				open: self.open,
				cooldown: self.cooldown,
				free_mint_fee_multiplier: self.free_mint_fee_multiplier,
			}
		}
	}

	#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Debug, Default, PartialEq)]
	pub struct ForgeConfigV5 {
		pub open: bool,
	}

	impl ForgeConfigV5 {
		pub fn migrate_to_v6(self) -> ForgeConfig {
			ForgeConfig { open: self.open }
		}
	}

	#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Debug, Default, PartialEq)]
	pub struct TransferConfigV5 {
		pub open: bool,
		pub free_mint_transfer_fee: MintCount,
		pub min_free_mint_transfer: MintCount,
	}

	impl TransferConfigV5 {
		pub fn migrate_to_v6(self) -> AvatarTransferConfig {
			AvatarTransferConfig { open: self.open }
		}
	}

	#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Debug, Default, PartialEq)]
	pub struct TradeConfigV5 {
		pub open: bool,
	}

	impl TradeConfigV5 {
		pub fn migrate_to_v6(self) -> TradeConfig {
			TradeConfig { open: self.open }
		}
	}

	#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Debug, Default, PartialEq)]
	pub struct NftTransferConfigV5 {
		pub open: bool,
	}

	impl NftTransferConfigV5 {
		pub fn migrate_to_v6(self) -> NftTransferConfig {
			NftTransferConfig { open: self.open }
		}
	}

	#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Debug, Default, PartialEq)]
	pub struct GlobalConfigV5<T: Config> {
		pub mint: MintConfigV5<T>,
		pub forge: ForgeConfigV5,
		pub transfer: TransferConfigV5,
		pub trade: TradeConfigV5,
		pub nft_transfer: NftTransferConfigV5,
	}

	impl<T> GlobalConfigV5<T>
	where
		T: Config,
	{
		pub fn migrate_to_v6(self) -> GlobalConfig<BlockNumberFor<T>, BalanceOf<T>> {
			let free_mint_transfer_fee = self.transfer.free_mint_transfer_fee;
			let min_free_mint_transfer = self.transfer.min_free_mint_transfer;

			GlobalConfig {
				mint: self.mint.migrate_to_v6(),
				forge: self.forge.migrate_to_v6(),
				avatar_transfer: self.transfer.migrate_to_v6(),
				freemint_transfer: FreemintTransferConfig {
					mode: FreeMintTransferMode::Open,
					free_mint_transfer_fee,
					min_free_mint_transfer,
				},
				trade: self.trade.migrate_to_v6(),
				nft_transfer: self.nft_transfer.migrate_to_v6(),
				affiliate_config: AffiliateConfig::default(),
			}
		}
	}

	#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Debug, Default, PartialEq)]
	pub struct FeeV5<T: Config> {
		pub mint: MintFees<BalanceOf<T>>,
		pub transfer_avatar: BalanceOf<T>,
		pub buy_minimum: BalanceOf<T>,
		pub buy_percent: u8,
		pub upgrade_storage: BalanceOf<T>,
		pub prepare_avatar: BalanceOf<T>,
	}

	impl<T> FeeV5<T>
	where
		T: Config,
	{
		pub fn migrate_to_v6(self) -> Fee<BalanceOf<T>> {
			Fee {
				mint: self.mint,
				transfer_avatar: self.transfer_avatar,
				buy_minimum: self.buy_minimum,
				buy_percent: self.buy_percent,
				upgrade_storage: self.upgrade_storage,
				prepare_avatar: self.prepare_avatar,
				set_price_unlock: Default::default(),
				avatar_transfer_unlock: Default::default(),
			}
		}
	}

	#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Debug, Default, PartialEq)]
	pub struct SeasonV5<T: Config> {
		pub name: BoundedVec<u8, ConstU32<100>>,
		pub description: BoundedVec<u8, ConstU32<1_000>>,
		pub early_start: BlockNumberFor<T>,
		pub start: BlockNumberFor<T>,
		pub end: BlockNumberFor<T>,
		pub max_tier_forges: u32,
		pub max_variations: u8,
		pub max_components: u8,
		pub min_sacrifices: SacrificeCount,
		pub max_sacrifices: SacrificeCount,
		pub tiers: BoundedVec<RarityTier, ConstU32<6>>,
		pub single_mint_probs: BoundedVec<RarityPercent, ConstU32<5>>,
		pub batch_mint_probs: BoundedVec<RarityPercent, ConstU32<5>>,
		pub base_prob: RarityPercent,
		pub per_period: BlockNumberFor<T>,
		pub periods: u16,
		pub trade_filters: BoundedVec<TradeFilter, ConstU32<100>>,
		pub fee: FeeV5<T>,
		pub mint_logic: LogicGeneration,
		pub forge_logic: LogicGeneration,
	}

	impl<T> SeasonV5<T>
	where
		T: Config,
	{
		pub fn migrate_to_v6(self) -> Season<BlockNumberFor<T>, BalanceOf<T>> {
			Season {
				max_tier_forges: self.max_tier_forges,
				max_variations: self.max_variations,
				max_components: self.max_components,
				min_sacrifices: self.min_sacrifices,
				max_sacrifices: self.max_sacrifices,
				tiers: self.tiers,
				single_mint_probs: self.single_mint_probs,
				batch_mint_probs: self.batch_mint_probs,
				base_prob: self.base_prob,
				per_period: self.per_period,
				periods: self.periods,
				fee: self.fee.migrate_to_v6(),
				mint_logic: self.mint_logic,
				forge_logic: self.forge_logic,
			}
		}
	}

	#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Debug, Default, PartialEq)]
	pub struct TradeStatsV5 {
		pub bought: Stat,
		pub sold: Stat,
	}

	#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Debug, Default, PartialEq)]
	pub struct PlayStatsV5<BlockNumber> {
		pub first: BlockNumber,
		pub last: BlockNumber,
		pub seasons_participated: BoundedBTreeSet<SeasonId, MaxSeasons>,
	}

	impl<BlockNumber> PlayStatsV5<BlockNumber> {
		pub fn migrate_to_v6(self) -> PlayStats<BlockNumber> {
			PlayStats { first: self.first, last: self.last }
		}
	}

	#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Debug, Default, PartialEq)]
	pub struct StatsV5<BlockNumber> {
		pub mint: PlayStatsV5<BlockNumber>,
		pub forge: PlayStatsV5<BlockNumber>,
		pub trade: TradeStatsV5,
	}

	impl<BlockNumber> StatsV5<BlockNumber> {
		pub fn migrate_to_v6(self) -> Stats<BlockNumber> {
			Stats { mint: self.mint.migrate_to_v6(), forge: self.forge.migrate_to_v6() }
		}
	}

	#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Debug, Default, PartialEq)]
	pub struct PlayerSeasonConfigV5<BlockNumber> {
		pub storage_tier: StorageTier,
		pub stats: StatsV5<BlockNumber>,
	}

	impl<BlockNumber> PlayerSeasonConfigV5<BlockNumber> {
		pub fn migrate_to_v6(self) -> PlayerSeasonConfig<BlockNumber> {
			PlayerSeasonConfig {
				storage_tier: self.storage_tier,
				stats: self.stats.migrate_to_v6(),
				locks: Locks::default(),
			}
		}
	}

	#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Debug, Default, PartialEq)]
	pub struct SeasonInfoV5 {
		pub minted: Stat,
		pub forged: Stat,
	}

	impl SeasonInfoV5 {
		pub fn migrate_to_v6(self, bought: Stat, sold: Stat) -> SeasonInfo {
			SeasonInfo { minted: self.minted, free_minted: 0, forged: self.forged, bought, sold }
		}
	}

	#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Debug, Default, PartialEq)]
	pub struct AvatarV5 {
		pub season_id: SeasonId,
		pub encoding: DnaEncoding,
		pub dna: Dna,
		pub souls: SoulCount,
	}

	impl AvatarV5 {
		pub fn migrate_to_v6<BlockNumber>(self) -> Avatar<BlockNumber>
		where
			BlockNumber: sp_runtime::traits::BlockNumber,
		{
			Avatar {
				season_id: self.season_id,
				encoding: self.encoding,
				dna: self.dna,
				souls: self.souls,
				minted_at: Default::default(),
			}
		}
	}

	#[storage_alias]
	pub type PlayerSeasonConfigs<T: Config> = StorageDoubleMap<
		Pallet<T>,
		Identity,
		<T as frame_system::Config>::AccountId,
		Identity,
		SeasonId,
		PlayerSeasonConfigV5<BlockNumberFor<T>>,
		OptionQuery,
	>;

	#[storage_alias]
	pub type SeasonStats<T: Config> = StorageDoubleMap<
		Pallet<T>,
		Identity,
		SeasonId,
		Identity,
		<T as frame_system::Config>::AccountId,
		SeasonInfoV5,
		OptionQuery,
	>;

	#[storage_alias]
	pub type Avatars<T: Config> = StorageMap<
		Pallet<T>,
		Identity,
		AvatarIdOf<T>,
		(<T as frame_system::Config>::AccountId, AvatarV5),
	>;
}

/// This migration updates the `GlobalConfig` and the `Seasons` in the classical single
/// block migration and it also sets the storage version to t6.
pub struct MigrateToV6<T>(sp_std::marker::PhantomData<T>);

/// We only have one global config and 2 seasons, we assume that we can do this in one block.
pub fn migrate_global_config_and_seasons<T: Config>() -> Weight {
	let _ = GlobalConfigs::<T>::translate::<v5::GlobalConfigV5<T>, _>(|old_config| {
		old_config.map(|old| old.migrate_to_v6())
	});

	log::info!(target: LOG_TARGET, "Updated GlobalConfig from v5 to v6");

	let mut seasons_translated = 0;

	Seasons::<T>::translate::<v5::SeasonV5<T>, _>(|season_id, old_season| {
		SeasonMetas::<T>::insert(
			season_id,
			SeasonMeta {
				name: old_season.name.clone(),
				description: old_season.description.clone(),
			},
		);

		SeasonSchedules::<T>::insert(
			season_id,
			SeasonSchedule {
				early_start: old_season.early_start,
				start: old_season.start,
				end: old_season.end,
			},
		);

		SeasonTradeFilters::<T>::insert(season_id, TradeFilters(old_season.trade_filters.clone()));

		seasons_translated += 1;

		Some(old_season.migrate_to_v6())
	});

	log::info!(target: LOG_TARGET, "Updated {} Season entries from v5 to v6", seasons_translated);

	let total_reads = seasons_translated;
	let total_writes = seasons_translated * 4;

	T::DbWeight::get().reads_writes(total_reads, total_writes)
}

impl<T: Config> OnRuntimeUpgrade for MigrateToV6<T> {
	fn on_runtime_upgrade() -> Weight {
		let current_version = Pallet::<T>::in_code_storage_version();
		let onchain_version = Pallet::<T>::on_chain_storage_version();
		if onchain_version == 5 && current_version == 6 {
			let mut weight = Weight::default();

			weight.saturating_accrue(migrate_global_config_and_seasons::<T>());

			current_version.put::<Pallet<T>>();
			log::info!(target: LOG_TARGET, "Upgraded storage to version {:?}", current_version);

			weight
		} else {
			log::info!(
				target: LOG_TARGET,
				"Migration for v6 did not execute."
			);
			T::DbWeight::get().reads(1)
		}
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(_state: Vec<u8>) -> Result<(), sp_runtime::DispatchError> {
		let current_version = Pallet::<T>::in_code_storage_version();
		let onchain_version = Pallet::<T>::on_chain_storage_version();

		if onchain_version == 5 && current_version == 6 {
			log::info!(
				target: LOG_TARGET,
				"Nothing to check in v5 -> v6 migration."
			);
		} else {
			log::info!(
				target: LOG_TARGET,
				"Migration post upgrade v6 did not execute."
			);
		}

		Ok(())
	}
}

/// The code following here is for the multiblock migration.
///
/// **NOTE** We have multiple multiblock migrations and the order **matters**.
/// Order:
///    1. LazyMigrationPlayerSeasonConfigsV5ToV6
///    2. LazyMigrationSeasonStatsV5ToV6
///    3. LazyMigrationAvatarV5ToV6
///    4. LazyTradeStatsMapCleanup
///
/// We have multiple multiblock migrations that are interdependent because one writes
/// into the auxiliary storage `TradeStatsMap`, where a following migration reads from there, and we
/// also have a cleanup migration, which removes all the remaining entries from the `TradeStatsMap`
/// storage.
///
/// The code is inspired by the following example:
/// https://github.com/paritytech/polkadot-sdk/blob/master/substrate/frame/examples/multi-block-migrations/src/lib.rs
pub mod mbm {
	use super::*;

	pub struct LazyMigrationPlayerSeasonConfigsV5ToV6<T: Config, W: weights::WeightInfo>(
		PhantomData<(T, W)>,
	);
	impl<T: Config, W: weights::WeightInfo> SteppedMigration
		for LazyMigrationPlayerSeasonConfigsV5ToV6<T, W>
	{
		type Cursor = (T::AccountId, SeasonId);
		// Without the explicit length here the construction of the ID would not be infallible.
		type Identifier = MigrationId<25>;

		/// The identifier of this migration. Which should be globally unique.
		fn id() -> Self::Identifier {
			MigrationId { pallet_id: *b"aaa-player-season-configs", version_from: 5, version_to: 6 }
		}

		/// The actual logic of the migration.
		///
		/// This function is called repeatedly until it returns `Ok(None)`, indicating that the
		/// migration is complete. Ideally, the migration should be designed in such a way that each
		/// step consumes as much weight as possible. However, this is simplified to perform one
		/// stored value mutation per block.
		fn step(
			mut cursor: Option<Self::Cursor>,
			meter: &mut WeightMeter,
		) -> Result<Option<Self::Cursor>, SteppedMigrationError> {
			let required = W::step();
			// If there is not enough weight for a single step, return an error. This case can be
			// problematic if it is the first migration that ran in this block. But there is nothing
			// that we can do about it here.
			if meter.remaining().any_lt(required) {
				return Err(SteppedMigrationError::InsufficientWeight { required });
			}

			let mut migration_count = 0u32;
			// We loop here to do as much progress as possible per step.
			loop {
				if meter.try_consume(required).is_err() {
					log::info!(
						target: LOG_TARGET,
						"Migrated {migration_count} PlayerSeasonConfigs. MBM is not finished yet."
					);
					break;
				}

				let mut iter = if let Some(last_key) = cursor {
					// If a cursor is provided, start iterating from the stored value
					// corresponding to the last key processed in the previous step.
					// Note that this only works if the old and the new map use the same way to hash
					// storage keys.
					v5::PlayerSeasonConfigs::<T>::iter_from(
						v5::PlayerSeasonConfigs::<T>::hashed_key_for(last_key.0, last_key.1),
					)
				} else {
					// If no cursor is provided, start iterating from the beginning.
					v5::PlayerSeasonConfigs::<T>::iter()
				};

				// If there's a next item in the iterator, perform the migration.
				if let Some((account, season_id, old_config)) = iter.next() {
					// Migrate the inner value: u32 -> u64.
					// We can just insert here since the old and the new map share the same
					// key-space. Otherwise it would have to invert the concat hash function and
					// re-hash it.

					TradeStatsMap::<T>::insert(
						season_id,
						&account,
						(old_config.stats.trade.bought, old_config.stats.trade.sold),
					);

					PlayerSeasonConfigs::<T>::insert(
						&account,
						season_id,
						old_config.migrate_to_v6(),
					);

					migration_count.saturating_inc();
					cursor = Some((account, season_id)) // Return the processed key as the new cursor.
				} else {
					log::info!(
						target: LOG_TARGET,
						"Migrated {migration_count} PlayerSeasonConfigs. Finished MBM Migration."
					);
					cursor = None; // Signal that the migration is complete (no more items to process).
					break
				}
			}
			Ok(cursor)
		}
	}

	pub struct LazyMigrationSeasonStatsV5ToV6<T: Config, W: weights::WeightInfo>(
		PhantomData<(T, W)>,
	);
	impl<T: Config, W: weights::WeightInfo> SteppedMigration for LazyMigrationSeasonStatsV5ToV6<T, W> {
		type Cursor = (SeasonId, T::AccountId);
		// Without the explicit length here the construction of the ID would not be infallible.
		type Identifier = MigrationId<16>;

		/// The identifier of this migration. Which should be globally unique.
		fn id() -> Self::Identifier {
			MigrationId { pallet_id: *b"aaa-season-stats", version_from: 5, version_to: 6 }
		}

		/// The actual logic of the migration.
		///
		/// This function is called repeatedly until it returns `Ok(None)`, indicating that the
		/// migration is complete. Ideally, the migration should be designed in such a way that each
		/// step consumes as much weight as possible. However, this is simplified to perform one
		/// stored value mutation per block.
		fn step(
			mut cursor: Option<Self::Cursor>,
			meter: &mut WeightMeter,
		) -> Result<Option<Self::Cursor>, SteppedMigrationError> {
			let required = W::step();
			// If there is not enough weight for a single step, return an error. This case can be
			// problematic if it is the first migration that ran in this block. But there is nothing
			// that we can do about it here.
			if meter.remaining().any_lt(required) {
				return Err(SteppedMigrationError::InsufficientWeight { required });
			}

			let mut migration_count = 0u32;
			// We loop here to do as much progress as possible per step.
			loop {
				if meter.try_consume(required).is_err() {
					log::info!(
					target: LOG_TARGET,
					"Migrated {migration_count} SeasonStats. MBM is not finished yet."
					);
					break;
				}

				let mut iter = if let Some(last_key) = cursor {
					// If a cursor is provided, start iterating from the stored value
					// corresponding to the last key processed in the previous step.
					// Note that this only works if the old and the new map use the same way to hash
					// storage keys.
					v5::SeasonStats::<T>::iter_from(v5::SeasonStats::<T>::hashed_key_for(
						last_key.0, last_key.1,
					))
				} else {
					// If no cursor is provided, start iterating from the beginning.
					v5::SeasonStats::<T>::iter()
				};

				// If there's a next item in the iterator, perform the migration.
				if let Some((season_id, account, old_season_info)) = iter.next() {
					if let Some((bought, sold)) = TradeStatsMap::<T>::take(season_id, &account) {
						SeasonStats::<T>::insert(
							season_id,
							&account,
							old_season_info.migrate_to_v6(bought, sold),
						);
					} else {
						log::error!(target: LOG_TARGET, "Missing trade mapping in SeasonStats from v5 to v6");
						SeasonStats::<T>::remove(season_id, &account);
					}

					migration_count.saturating_inc();
					cursor = Some((season_id, account)) // Return the processed key as the new cursor.
				} else {
					log::info!(
					target: LOG_TARGET,
					"Migrated {migration_count} SeasonStats. Finished MBM Migration."
					);
					cursor = None; // Signal that the migration is complete (no more items to process).
					break
				}
			}
			Ok(cursor)
		}
	}

	pub struct LazyMigrationAvatarV5ToV6<T: Config, W: weights::WeightInfo>(PhantomData<(T, W)>);
	impl<T: Config, W: weights::WeightInfo> SteppedMigration for LazyMigrationAvatarV5ToV6<T, W> {
		type Cursor = AvatarIdOf<T>;
		// Without the explicit length here the construction of the ID would not be infallible.
		type Identifier = MigrationId<10>;

		/// The identifier of this migration. Which should be globally unique.
		fn id() -> Self::Identifier {
			MigrationId { pallet_id: *b"aaa-avatar", version_from: 5, version_to: 6 }
		}

		/// The actual logic of the migration.
		///
		/// This function is called repeatedly until it returns `Ok(None)`, indicating that the
		/// migration is complete. Ideally, the migration should be designed in such a way that each
		/// step consumes as much weight as possible. However, this is simplified to perform one
		/// stored value mutation per block.
		fn step(
			mut cursor: Option<Self::Cursor>,
			meter: &mut WeightMeter,
		) -> Result<Option<Self::Cursor>, SteppedMigrationError> {
			let required = W::step();
			// If there is not enough weight for a single step, return an error. This case can be
			// problematic if it is the first migration that ran in this block. But there is nothing
			// that we can do about it here.
			if meter.remaining().any_lt(required) {
				return Err(SteppedMigrationError::InsufficientWeight { required });
			}

			let mut migration_count = 0u32;
			// We loop here to do as much progress as possible per step.
			loop {
				if meter.try_consume(required).is_err() {
					log::info!(
						target: LOG_TARGET,
						"Migrated {migration_count} Avatars. MBM is not finished yet."
					);
					break;
				}

				let mut iter = if let Some(last_key) = cursor {
					// If a cursor is provided, start iterating from the stored value
					// corresponding to the last key processed in the previous step.
					// Note that this only works if the old and the new map use the same way to hash
					// storage keys.
					v5::Avatars::<T>::iter_from(v5::Avatars::<T>::hashed_key_for(last_key))
				} else {
					// If no cursor is provided, start iterating from the beginning.
					v5::Avatars::<T>::iter()
				};

				// If there's a next item in the iterator, perform the migration.
				if let Some((avatar_id, old_account_avatar_tuple)) = iter.next() {
					Avatars::<T>::insert(
						avatar_id,
						(old_account_avatar_tuple.0, old_account_avatar_tuple.1.migrate_to_v6()),
					);

					migration_count.saturating_inc();
					cursor = Some(avatar_id) // Return the processed key as the new cursor.
				} else {
					log::info!(
						target: LOG_TARGET,
						"Migrated {migration_count} Avatars. Finished MBM Migration."
					);
					cursor = None; // Signal that the migration is complete (no more items to process).
					break
				}
			}
			Ok(cursor)
		}
	}

	pub struct LazyTradeStatsMapCleanup<T: Config, W: weights::WeightInfo>(PhantomData<(T, W)>);
	impl<T: Config, W: weights::WeightInfo> SteppedMigration for LazyTradeStatsMapCleanup<T, W> {
		type Cursor = (SeasonId, T::AccountId);
		// Without the explicit length here the construction of the ID would not be infallible.
		type Identifier = MigrationId<23>;

		/// The identifier of this migration. Which should be globally unique.
		fn id() -> Self::Identifier {
			MigrationId { pallet_id: *b"aaa-trade-stats-cleanup", version_from: 5, version_to: 6 }
		}

		/// The actual logic of the migration.
		///
		/// This function is called repeatedly until it returns `Ok(None)`, indicating that the
		/// migration is complete. Ideally, the migration should be designed in such a way that each
		/// step consumes as much weight as possible. However, this is simplified to perform one
		/// stored value mutation per block.
		fn step(
			mut cursor: Option<Self::Cursor>,
			meter: &mut WeightMeter,
		) -> Result<Option<Self::Cursor>, SteppedMigrationError> {
			let required = W::step();
			// If there is not enough weight for a single step, return an error. This case can be
			// problematic if it is the first migration that ran in this block. But there is nothing
			// that we can do about it here.
			if meter.remaining().any_lt(required) {
				return Err(SteppedMigrationError::InsufficientWeight { required });
			}

			let mut migration_count = 0u32;
			// We loop here to do as much progress as possible per step.
			loop {
				if meter.try_consume(required).is_err() {
					log::info!(
						target: LOG_TARGET,
						"Cleaned up {migration_count} TradeStats. MBM is not finished yet."
					);
					break;
				}

				let mut iter = if let Some(last_key) = cursor {
					// If a cursor is provided, start iterating from the stored value
					// corresponding to the last key processed in the previous step.
					// Note that this only works if the old and the new map use the same way to hash
					// storage keys.
					TradeStatsMap::<T>::iter_from(TradeStatsMap::<T>::hashed_key_for(
						last_key.0, last_key.1,
					))
				} else {
					// If no cursor is provided, start iterating from the beginning.
					TradeStatsMap::<T>::iter()
				};

				// If there's a next item in the iterator, perform the migration.
				if let Some((season_id, account_id, _stats)) = iter.next() {
					TradeStatsMap::<T>::remove(season_id, &account_id);

					migration_count.saturating_inc();
					cursor = Some((season_id, account_id)) // Return the processed key as the new cursor.
				} else {
					log::info!(
						target: LOG_TARGET,
						"Cleaned up {migration_count} TradeStats. Finished MBM Migration."
					);
					cursor = None; // Signal that the migration is complete (no more items to process).
					break
				}
			}
			Ok(cursor)
		}
	}

	/// This has been introduced on latest master, but it doesn't exist yet in v1.10.0.
	///
	/// A generic migration identifier that can be used by MBMs.
	///
	/// It is not required that migrations use this identifier type, but it can help.
	#[derive(MaxEncodedLen, Encode, Decode)]
	pub struct MigrationId<const N: usize> {
		pub pallet_id: [u8; N],
		pub version_from: u8,
		pub version_to: u8,
	}
}
