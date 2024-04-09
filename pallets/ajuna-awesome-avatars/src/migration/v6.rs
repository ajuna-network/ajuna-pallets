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

use super::*;
use sp_std::collections::btree_map::BTreeMap;

#[derive(Decode)]
pub struct MintConfigV5<T: Config> {
	pub open: bool,
	pub cooldown: BlockNumberFor<T>,
	pub free_mint_fee_multiplier: MintCount,
}

impl<T> MintConfigV5<T>
where
	T: Config,
{
	fn migrate_to_v6(self) -> MintConfig<BlockNumberFor<T>> {
		MintConfig {
			open: self.open,
			cooldown: self.cooldown,
			free_mint_fee_multiplier: self.free_mint_fee_multiplier,
		}
	}
}

#[derive(Decode)]
pub struct ForgeConfigV5 {
	pub open: bool,
}

impl ForgeConfigV5 {
	fn migrate_to_v6(self) -> ForgeConfig {
		ForgeConfig { open: self.open }
	}
}

#[derive(Decode)]
pub struct TransferConfigV5 {
	pub open: bool,
	pub free_mint_transfer_fee: MintCount,
	pub min_free_mint_transfer: MintCount,
}

impl TransferConfigV5 {
	fn migrate_to_v6(self) -> AvatarTransferConfig {
		AvatarTransferConfig { open: self.open }
	}
}

#[derive(Decode)]
pub struct TradeConfigV5 {
	pub open: bool,
}

impl TradeConfigV5 {
	fn migrate_to_v6(self) -> TradeConfig {
		TradeConfig { open: self.open }
	}
}

#[derive(Decode)]
pub struct NftTransferConfigV5 {
	pub open: bool,
}

impl NftTransferConfigV5 {
	fn migrate_to_v6(self) -> NftTransferConfig {
		NftTransferConfig { open: self.open }
	}
}

#[derive(Decode)]
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
	fn migrate_to_v6(self) -> GlobalConfig<BlockNumberFor<T>, BalanceOf<T>> {
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

#[derive(Decode)]
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
	fn migrate_to_v6(self) -> Fee<BalanceOf<T>> {
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

#[derive(Decode)]
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
	fn migrate_to_v6(self) -> Season<BlockNumberFor<T>, BalanceOf<T>> {
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

#[derive(Decode)]
pub struct TradeStatsV5 {
	pub bought: Stat,
	pub sold: Stat,
}

#[derive(Decode)]
pub struct PlayStatsV5<T: Config> {
	pub first: BlockNumberFor<T>,
	pub last: BlockNumberFor<T>,
	pub seasons_participated: BoundedBTreeSet<SeasonId, MaxSeasons>,
}

impl<T> PlayStatsV5<T>
where
	T: Config,
{
	fn migrate_to_v6(self) -> PlayStats<BlockNumberFor<T>> {
		PlayStats { first: self.first, last: self.last }
	}
}

#[derive(Decode)]
pub struct StatsV5<T: Config> {
	pub mint: PlayStatsV5<T>,
	pub forge: PlayStatsV5<T>,
	pub trade: TradeStatsV5,
}

impl<T> StatsV5<T>
where
	T: Config,
{
	fn migrate_to_v6(self) -> Stats<BlockNumberFor<T>> {
		Stats { mint: self.mint.migrate_to_v6(), forge: self.forge.migrate_to_v6() }
	}
}
#[derive(Decode)]
pub struct PlayerSeasonConfigV5<T: Config> {
	pub storage_tier: StorageTier,
	pub stats: StatsV5<T>,
}

impl<T> PlayerSeasonConfigV5<T>
where
	T: Config,
{
	fn migrate_to_v6(self) -> PlayerSeasonConfig<BlockNumberFor<T>> {
		PlayerSeasonConfig {
			storage_tier: self.storage_tier,
			stats: self.stats.migrate_to_v6(),
			locks: Locks::default(),
		}
	}
}

#[derive(Decode)]
pub struct SeasonInfoV5 {
	pub minted: Stat,
	pub forged: Stat,
}

impl SeasonInfoV5 {
	fn migrate_to_v6(self, bought: Stat, sold: Stat) -> SeasonInfo {
		SeasonInfo { minted: self.minted, free_minted: 0, forged: self.forged, bought, sold }
	}
}

#[derive(Decode)]
pub struct AvatarV5 {
	pub season_id: SeasonId,
	pub encoding: DnaEncoding,
	pub dna: Dna,
	pub souls: SoulCount,
}

impl AvatarV5 {
	fn migrate_to_v6<BlockNumber>(self) -> Avatar<BlockNumber>
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

pub struct MigrateToV6<T>(sp_std::marker::PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for MigrateToV6<T> {
	fn on_runtime_upgrade() -> Weight {
		let current_version = Pallet::<T>::in_code_storage_version();
		let onchain_version = Pallet::<T>::on_chain_storage_version();
		if onchain_version == 5 && current_version == 6 {
			let _ = GlobalConfigs::<T>::translate::<GlobalConfigV5<T>, _>(|old_config| {
				old_config.map(|old| old.migrate_to_v6())
			});

			log::info!(target: LOG_TARGET, "Updated GlobalConfig from v5 to v6");

			let mut seasons_translated = 0;

			Seasons::<T>::translate::<SeasonV5<T>, _>(|season_id, old_season| {
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

				SeasonTradeFilters::<T>::insert(
					season_id,
					TradeFilters(old_season.trade_filters.clone()),
				);

				seasons_translated += 1;

				Some(old_season.migrate_to_v6())
			});

			log::info!(target: LOG_TARGET, "Updated {} Season entries from v5 to v6", seasons_translated);

			let mut trade_stats_map = BTreeMap::<(SeasonId, T::AccountId), (Stat, Stat)>::new();
			let mut player_season_configs_translated = 0;

			PlayerSeasonConfigs::<T>::translate::<PlayerSeasonConfigV5<T>, _>(
				|account, season_id, old_config| {
					trade_stats_map.insert(
						(season_id, account),
						(old_config.stats.trade.bought, old_config.stats.trade.sold),
					);

					player_season_configs_translated += 1;

					Some(old_config.migrate_to_v6())
				},
			);

			log::info!(target: LOG_TARGET, "Updated {} PlayerSeasonConfigs entries from v5 to v6", player_season_configs_translated);

			let mut season_stats_translated = 0;

			SeasonStats::<T>::translate::<SeasonInfoV5, _>(
				|season_id, account, old_season_info| {
					if let Some((bought, sold)) = trade_stats_map.remove(&(season_id, account)) {
						season_stats_translated += 1;

						Some(old_season_info.migrate_to_v6(bought, sold))
					} else {
						log::error!(target: LOG_TARGET, "Missing trade mapping in SeasonStats from v5 to v6");
						None
					}
				},
			);

			log::info!(target: LOG_TARGET, "Updated {} SeasonStats entries from v5 to v6", season_stats_translated);

			let mut avatars_translated = 0;

			Avatars::<T>::translate::<(AccountIdFor<T>, AvatarV5), _>(|_, (account, avatar)| {
				avatars_translated += 1;

				Some((account, avatar.migrate_to_v6()))
			});

			log::info!(target: LOG_TARGET, "Updated {} Avatar entries from v5 to v6", avatars_translated);

			current_version.put::<Pallet<T>>();
			log::info!(target: LOG_TARGET, "Upgraded storage to version {:?}", current_version);

			let total_reads = seasons_translated +
				player_season_configs_translated +
				season_stats_translated +
				avatars_translated;
			let total_writes = (seasons_translated * 4) +
				player_season_configs_translated +
				season_stats_translated +
				avatars_translated;

			T::DbWeight::get().reads_writes(total_reads, total_writes)
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
