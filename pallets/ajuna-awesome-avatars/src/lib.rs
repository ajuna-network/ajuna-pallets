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

//! # Ajuna Awesome Avatars
//!
//! The Awesome Ajuna Avatars is a collective game based on the Heroes of Ajuna.
//!
//! - [`Config`]
//! - [`Call`]
//! - [`Error`]
//! - [`Event`]
//! - [`Pallet`]
//!
//! ## Overview
//!
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! Some dispatchable functions can be called only from the organizer.
//!
//! * `mint` - Create a new AAA.
//! * `forge` - Sacrifice a batch of avatars in order to improve a leader.
//! * `transfer_free_mints` - Send free mints to another player.
//! * `set_price` - Assign a price to an avatar.
//! * `remove_price` - Remove the price of an avatar.
//! * `buy` - Buy an avatar.
//! * `upgrade_storage` - Upgrade the capacity to hold avatars.
//! * `set_organizer` - Set the game organizer.
//! * `set_treasurer` - Set the treasurer.
//! * `set_season` - Add a new season.
//! * `update_global_config` - Update the configuration.
//! * `set_free_mints` - Set a number of free mints to a player.
//!
//! ### Public Functions
//!
//! * `do_forge` - Forge avatar.
//! * `do_mint` - Mint avatar.
//! * `ensure_season_schedule` - Given a season id and a season schedule, validate them.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate core;

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(any(test, feature = "runtime-benchmarks"))]
pub mod benchmark_helper;

pub mod impls;
pub mod migration;
pub mod types;
pub mod weights;

use crate::{types::*, weights::WeightInfo};
use ajuna_primitives::{
	account_manager::{AccountManager, WhitelistKey},
	asset_manager::{AssetManager, Lock, LockIdentifier},
};
use frame_support::{
	pallet_prelude::*,
	traits::{Currency, ExistenceRequirement::AllowDeath, Randomness, WithdrawReasons},
	PalletId,
};
use frame_system::{ensure_root, ensure_signed, pallet_prelude::*};
use pallet_ajuna_affiliates::traits::{
	AffiliateInspector, AffiliateMutator, RuleInspector, RuleMutator,
};
use pallet_ajuna_tournament::{
	config::{TournamentConfig, TournamentState},
	traits::{TournamentClaimer, TournamentInspector, TournamentMutator, TournamentRanker},
};
use sp_runtime::{
	traits::{
		AccountIdConversion, CheckedDiv, CheckedSub, Hash, Saturating, TrailingZeroInput,
		UniqueSaturatedInto, Zero,
	},
	ArithmeticError,
};
use sp_std::prelude::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use pallet_ajuna_affiliates::traits::{AffiliateId, RuleExecutor};
	use pallet_ajuna_tournament::{Percentage, TournamentId};
	use sp_std::collections::vec_deque::VecDeque;

	pub(crate) type AccountIdFor<T> = <T as frame_system::Config>::AccountId;
	pub(crate) type SeasonOf<T> = Season<BlockNumberFor<T>, BalanceOf<T>>;
	pub(crate) type SeasonScheduleOf<T> = SeasonSchedule<BlockNumberFor<T>>;
	pub(crate) type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdFor<T>>>::Balance;
	pub type AvatarIdOf<T> = <T as frame_system::Config>::Hash;
	pub type AvatarOf<T> = Avatar<BlockNumberFor<T>>;
	pub(crate) type BoundedAvatarIdsOf<T> = BoundedVec<AvatarIdOf<T>, MaxAvatarsPerPlayer>;
	pub(crate) type GlobalConfigOf<T> = GlobalConfig<BlockNumberFor<T>, BalanceOf<T>>;
	pub type FeePropagationOf<T> = FeePropagation<<T as Config>::FeeChainMaxLength>;
	pub type AvatarRankerFor<T> = AvatarRanker<AvatarIdOf<T>, BlockNumberFor<T>>;
	pub type TournamentConfigFor<T> = TournamentConfig<BlockNumberFor<T>, BalanceOf<T>>;

	pub(crate) const MAX_PERCENTAGE: u8 = 100;

	#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Debug, PartialEq)]
	pub enum WhitelistOperation {
		AddAccount,
		RemoveAccount,
		ClearList,
	}

	#[pallet::pallet]
	#[pallet::storage_version(migration::STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		#[pallet::constant]
		type PalletId: Get<PalletId>;

		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		type Currency: Currency<Self::AccountId>;

		type Randomness: Randomness<Self::Hash, BlockNumberFor<Self>>;

		/// The maximum depth of the propagation fee chain,
		#[pallet::constant]
		type FeeChainMaxLength: Get<u32>;

		type AffiliateHandler: AffiliateInspector<AccountIdFor<Self>>
			+ AffiliateMutator<AccountIdFor<Self>>
			+ RuleInspector<AffiliateMethods, FeePropagationOf<Self>>
			+ RuleMutator<AffiliateMethods, FeePropagationOf<Self>>
			+ RuleExecutor<AffiliateMethods, FeePropagationOf<Self>>;

		type TournamentHandler: TournamentInspector<SeasonId, BlockNumberFor<Self>, BalanceOf<Self>, AccountIdFor<Self>>
			+ TournamentMutator<AccountIdFor<Self>, SeasonId, BlockNumberFor<Self>, BalanceOf<Self>>
			+ TournamentRanker<SeasonId, AvatarOf<Self>, AvatarIdOf<Self>>
			+ TournamentClaimer<SeasonId, AccountIdFor<Self>, AvatarIdOf<Self>>;

		type WeightInfo: WeightInfo;
	}

	#[pallet::storage]
	pub type Organizer<T: Config> = StorageValue<_, T::AccountId, OptionQuery>;

	#[pallet::storage]
	pub type Treasurer<T: Config> = StorageMap<_, Identity, SeasonId, T::AccountId, OptionQuery>;

	/// List of accounts allowed to transfer free mints.
	/// A maximum of 3 different accounts can be on the list.
	#[pallet::storage]
	#[pallet::getter(fn whitelist)]
	pub type WhitelistedAccounts<T: Config> =
		StorageValue<_, BoundedVec<T::AccountId, ConstU32<3>>, ValueQuery>;

	#[pallet::storage]
	pub type CurrentSeasonStatus<T: Config> = StorageValue<_, SeasonStatus, ValueQuery>;

	/// Storage for the seasons.
	#[pallet::storage]
	pub type Seasons<T: Config> = StorageMap<_, Identity, SeasonId, SeasonOf<T>, OptionQuery>;

	/// Storage for the season's metadata.
	#[pallet::storage]
	pub type SeasonMetas<T: Config> = StorageMap<_, Identity, SeasonId, SeasonMeta, OptionQuery>;

	/// Storage for the season's schedules.
	#[pallet::storage]
	pub type SeasonSchedules<T: Config> =
		StorageMap<_, Identity, SeasonId, SeasonScheduleOf<T>, OptionQuery>;

	/// Storage for the season's trade filters.
	#[pallet::storage]
	pub type SeasonTradeFilters<T: Config> =
		StorageMap<_, Identity, SeasonId, TradeFilters, OptionQuery>;

	/// Storage for the season's different unlock-ables.
	#[pallet::storage]
	pub type SeasonUnlocks<T: Config> =
		StorageMap<_, Identity, SeasonId, UnlockConfigs, OptionQuery>;

	#[pallet::storage]
	pub type Treasury<T: Config> = StorageMap<_, Identity, SeasonId, BalanceOf<T>, ValueQuery>;

	#[pallet::storage]
	pub type GlobalConfigs<T: Config> = StorageValue<_, GlobalConfigOf<T>, ValueQuery>;

	#[pallet::storage]
	pub type Avatars<T: Config> =
		StorageMap<_, Identity, AvatarIdOf<T>, (T::AccountId, AvatarOf<T>)>;

	#[pallet::storage]
	pub type Owners<T: Config> = StorageDoubleMap<
		_,
		Identity,
		T::AccountId,
		Identity,
		SeasonId,
		BoundedAvatarIdsOf<T>,
		ValueQuery,
	>;

	#[pallet::storage]
	pub type LockedAvatars<T: Config> =
		StorageMap<_, Identity, AvatarIdOf<T>, Lock<AccountIdFor<T>>>;

	#[pallet::storage]
	pub type PlayerConfigs<T: Config> =
		StorageMap<_, Identity, T::AccountId, PlayerConfig, ValueQuery>;

	#[pallet::storage]
	pub type PlayerSeasonConfigs<T: Config> = StorageDoubleMap<
		_,
		Identity,
		T::AccountId,
		Identity,
		SeasonId,
		PlayerSeasonConfig<BlockNumberFor<T>>,
		ValueQuery,
	>;

	/// This is only an intermediate storage that is being used during the multiblock runtime
	/// migration of v5 to v6. It should be removed afterward.
	#[pallet::storage]
	pub type TradeStatsMap<T: Config> =
		StorageDoubleMap<_, Identity, SeasonId, Identity, T::AccountId, (Stat, Stat), OptionQuery>;

	#[pallet::storage]
	pub type SeasonStats<T: Config> =
		StorageDoubleMap<_, Identity, SeasonId, Identity, T::AccountId, SeasonInfo, ValueQuery>;

	#[pallet::storage]
	pub type Trade<T: Config> =
		StorageDoubleMap<_, Identity, SeasonId, Identity, AvatarIdOf<T>, BalanceOf<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn rankers)]
	pub type TournamentRankers<T: Config> = StorageDoubleMap<
		_,
		Identity,
		SeasonId,
		Identity,
		TournamentId,
		AvatarRankerFor<T>,
		OptionQuery,
	>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		_phantom: sp_std::marker::PhantomData<T>,
	}

	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			GenesisConfig { _phantom: Default::default() }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			CurrentSeasonStatus::<T>::put(SeasonStatus {
				season_id: 1,
				early: Default::default(),
				active: Default::default(),
				early_ended: Default::default(),
				max_tier_avatars: Default::default(),
			});
			GlobalConfigs::<T>::put(GlobalConfig {
				mint: MintConfig { open: true, cooldown: 5_u8.into(), free_mint_fee_multiplier: 1 },
				forge: ForgeConfig { open: true },
				avatar_transfer: AvatarTransferConfig { open: true },
				freemint_transfer: FreemintTransferConfig {
					mode: FreeMintTransferMode::Open,
					free_mint_transfer_fee: 1,
					min_free_mint_transfer: 1,
				},
				trade: TradeConfig { open: true },
				nft_transfer: NftTransferConfig { open: true },
				affiliate_config: AffiliateConfig::default(),
			});
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// An organizer has been set.
		OrganizerSet { organizer: T::AccountId },
		/// A treasurer has been set for a season.
		TreasurerSet { season_id: SeasonId, treasurer: T::AccountId },
		/// A season's treasury has been claimed by a treasurer.
		TreasuryClaimed { season_id: SeasonId, treasurer: T::AccountId, amount: BalanceOf<T> },
		/// The season configuration for {season_id} has been updated.
		UpdatedSeason {
			season_id: SeasonId,
			season: Option<SeasonOf<T>>,
			meta: Option<SeasonMeta>,
			schedule: Option<SeasonScheduleOf<T>>,
			trade_filters: Option<TradeFilters>,
		},
		/// Global configuration updated.
		UpdatedGlobalConfig(GlobalConfigOf<T>),
		/// Avatars minted.
		AvatarsMinted { avatar_ids: Vec<AvatarIdOf<T>> },
		/// Avatar forged.
		AvatarsForged { avatar_ids: Vec<(AvatarIdOf<T>, UpgradedComponents)> },
		/// Avatar transferred.
		AvatarTransferred { from: T::AccountId, to: T::AccountId, avatar_id: AvatarIdOf<T> },
		/// A season has started.
		SeasonStarted(SeasonId),
		/// A season has finished.
		SeasonFinished(SeasonId),
		/// Free mints transferred between accounts.
		FreeMintsTransferred { from: T::AccountId, to: T::AccountId, how_many: MintCount },
		/// Free mints set for target account.
		FreeMintsSet { target: T::AccountId, how_many: MintCount },
		/// Avatar has price set for trade.
		AvatarPriceSet { avatar_id: AvatarIdOf<T>, price: BalanceOf<T> },
		/// Avatar has price removed for trade.
		AvatarPriceUnset { avatar_id: AvatarIdOf<T> },
		/// Avatar has been traded.
		AvatarTraded {
			avatar_id: AvatarIdOf<T>,
			from: T::AccountId,
			to: T::AccountId,
			price: BalanceOf<T>,
		},
		/// Avatar locked.
		AvatarLocked { avatar_id: AvatarIdOf<T> },
		/// Avatar unlocked.
		AvatarUnlocked { avatar_id: AvatarIdOf<T> },
		/// Storage tier has been upgraded.
		StorageTierUpgraded { account: T::AccountId, season_id: SeasonId },
		/// Unlock configurations updated.
		UpdatedUnlockConfigs { season_id: SeasonId, unlock_configs: UnlockConfigs },
	}

	#[pallet::error]
	pub enum Error<T> {
		/// There is no account set as the organizer
		OrganizerNotSet,
		/// The season starts before the previous season has ended.
		EarlyStartTooEarly,
		/// The season season start later than its early access
		EarlyStartTooLate,
		/// The season start date is newer than its end date.
		SeasonStartTooLate,
		/// The season ends after the new season has started.
		SeasonEndTooLate,
		/// The season's per period and periods configuration overflows.
		PeriodConfigOverflow,
		/// The season's periods configuration is indivisible by max variation.
		PeriodsIndivisible,
		/// The season doesn't exist.
		UnknownSeason,
		/// The avatar doesn't exist.
		UnknownAvatar,
		/// The avatar for sale doesn't exist.
		UnknownAvatarForSale,
		/// The tier doesn't exist.
		UnknownTier,
		/// The treasurer doesn't exist.
		UnknownTreasurer,
		/// The preparation doesn't exist.
		UnknownPreparation,
		/// The season ID of a season to create is not sequential.
		NonSequentialSeasonId,
		/// The sum of the given single mint probabilities overflows.
		SingleMintProbsOverflow,
		/// The sum of the given batch mint probabilities overflows.
		BatchMintProbsOverflow,
		/// Rarity percentages don't add up to 100
		IncorrectRarityPercentages,
		/// Max tier is achievable through forging only. Therefore the number of rarity percentages
		/// must be less than that of tiers for a season.
		TooManyRarityPercentages,
		/// The given base probability is too high. It must be less than 100.
		BaseProbTooHigh,
		/// Some rarity tier are duplicated.
		DuplicatedRarityTier,
		/// Minting is not available at the moment.
		MintClosed,
		/// Forging is not available at the moment.
		ForgeClosed,
		/// Transfer is not available at the moment.
		TransferClosed,
		/// Trading is not available at the moment.
		TradeClosed,
		/// Free mint transfer is not available at the moment.
		FreeMintTransferClosed,
		/// Attempt to mint or forge outside of an active season.
		SeasonClosed,
		/// Attempt to mint when the season has ended prematurely.
		PrematureSeasonEnd,
		/// Max ownership reached.
		MaxOwnershipReached,
		/// Max storage tier reached.
		MaxStorageTierReached,
		/// Avatar belongs to someone else.
		Ownership,
		/// Attempt to buy his or her own avatar.
		AlreadyOwned,
		/// Incorrect DNA.
		IncorrectDna,
		/// Incorrect data.
		IncorrectData,
		/// Incorrect Avatar ID.
		IncorrectAvatarId,
		/// Incorrect season ID.
		IncorrectSeasonId,
		/// The player must wait cooldown period.
		MintCooldown,
		/// The season's max components value is less than the minimum allowed (1).
		MaxComponentsTooLow,
		/// The season's max components value is more than the maximum allowed (random byte: 32).
		MaxComponentsTooHigh,
		/// The season's max variations value is less than the minimum allowed (1).
		MaxVariationsTooLow,
		/// The season's max variations value is more than the maximum allowed (15).
		MaxVariationsTooHigh,
		/// The player has not enough free mints available.
		InsufficientFreeMints,
		/// The player has not enough balance available.
		InsufficientBalance,
		/// Attempt to transfer, issue or withdraw free mints lower than the minimum allowed.
		TooLowFreeMints,
		/// Less than minimum allowed sacrifices are used for forging.
		TooFewSacrifices,
		/// More than maximum allowed sacrifices are used for forging.
		TooManySacrifices,
		/// Leader is being sacrificed.
		LeaderSacrificed,
		/// This avatar cannot be used in trades.
		AvatarCannotBeTraded,
		/// An avatar listed for trade is used to forge.
		AvatarInTrade,
		/// The avatar is currently locked and cannot be used.
		AvatarLocked,
		/// The avatar is locked by another application.
		AvatarLockedByOtherApplication,
		/// The avatar is not currently locked and cannot be unlocked.
		AvatarNotLocked,
		/// Tried to forge avatars from different seasons.
		IncorrectAvatarSeason,
		/// Tried to forge avatars with different DNA versions.
		IncompatibleAvatarVersions,
		/// There's not enough space to hold the forging results
		InsufficientStorageForForging,
		/// Tried transferring to his or her own account.
		CannotTransferToSelf,
		/// Tried transferring while the account still hasn't minted and forged anything.
		CannotTransferFromInactiveAccount,
		/// Tried claiming treasury during a season.
		CannotClaimDuringSeason,
		/// Tried claiming treasury which is zero.
		CannotClaimZero,
		/// The components tried to mint were not compatible.
		IncompatibleMintComponents,
		/// The components tried to forge were not compatible.
		IncompatibleForgeComponents,
		/// The amount of sacrifices is not sufficient for forging.
		InsufficientSacrifices,
		/// The amount of sacrifices is too much for forging.
		ExcessiveSacrifices,
		/// Tried to prepare an IPFS URL for an avatar with an empty URL.
		EmptyIpfsUrl,
		/// The account trying to be whitelisted is already in the whitelist
		AccountAlreadyInWhitelist,
		/// Cannot add more accounts to the whitelist.
		WhitelistedAccountsLimitReached,
		/// No account matches the provided affiliator identifier
		AffiliatorNotFound,
		/// The feature is locked for the current player
		FeatureLocked,
		/// The feature trying to be unlocked is not available for the selected season
		FeatureLockedInSeason,
		/// The feature trying to be unlocked cannot be unlocked with payment
		FeatureLockedThroughPayment,
		/// The feature trying to be unlocked has missing requirements to be fulfilled by
		/// the account trying to unlock it
		UnlockCriteriaNotFulfilled,
		/// Couldn't find a tournament ranker for the active tournament; qed
		TournamentRankerNotFound,
		/// Only whitelisted accounts can affiliate for others
		AffiliateOthersOnlyWhiteListed,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(now: BlockNumberFor<T>) -> Weight {
			let current_season_id = CurrentSeasonStatus::<T>::get().season_id;
			let mut weight = T::DbWeight::get().reads(1);

			if let Some(current_season) = SeasonSchedules::<T>::get(current_season_id) {
				weight.saturating_accrue(T::DbWeight::get().reads(1));

				if now <= current_season.end {
					Self::start_season(&mut weight, now, current_season_id, &current_season);
				} else {
					Self::finish_season(&mut weight, now, current_season_id);
				}
			}

			weight
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Issue a new avatar.
		///
		/// Emits `AvatarsMinted` event when successful.
		///
		/// Weight: `O(n)` where:
		/// - `n = max avatars per player`
		#[pallet::call_index(0)]
		#[pallet::weight({
			let n = MaxAvatarsPerPlayer::get();
			T::WeightInfo::mint_normal(n)
				.max(T::WeightInfo::mint_free(n))
		})]
		pub fn mint(origin: OriginFor<T>, mint_option: MintOption) -> DispatchResult {
			let player = ensure_signed(origin)?;
			Self::do_mint(&player, &mint_option)
		}

		/// Forge an avatar.
		///
		/// This action can enhance the skills of an avatar by consuming a batch of avatars.
		/// The minimum and maximum number of avatars that can be utilized for forging is
		/// defined in the season configuration.
		///
		/// Emits `AvatarForged` event when successful.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::forge(MaxAvatarsPerPlayer::get()))]
		pub fn forge(
			origin: OriginFor<T>,
			leader: AvatarIdOf<T>,
			sacrifices: Vec<AvatarIdOf<T>>,
		) -> DispatchResult {
			let player = ensure_signed(origin)?;
			Self::do_forge(&player, &leader, sacrifices)
		}

		#[pallet::call_index(2)]
		#[pallet::weight({
			let n = MaxAvatarsPerPlayer::get();
			T::WeightInfo::transfer_avatar_normal(n)
				.max(T::WeightInfo::transfer_avatar_organizer(n))
		})]
		pub fn transfer_avatar(
			origin: OriginFor<T>,
			to: T::AccountId,
			avatar_id: AvatarIdOf<T>,
		) -> DispatchResult {
			let GlobalConfig { avatar_transfer, .. } = GlobalConfigs::<T>::get();
			let from = match Self::ensure_organizer(origin.clone()) {
				Ok(organizer) => organizer,
				_ => {
					ensure!(avatar_transfer.open, Error::<T>::TransferClosed);
					ensure_signed(origin)?
				},
			};
			ensure!(from != to, Error::<T>::CannotTransferToSelf);
			ensure!(Self::ensure_for_trade(&avatar_id).is_err(), Error::<T>::AvatarInTrade);
			Self::ensure_unlocked(&avatar_id)?;

			let avatar = Self::ensure_ownership(&from, &avatar_id)?;
			ensure!(
				PlayerSeasonConfigs::<T>::get(&from, avatar.season_id).locks.avatar_transfer,
				Error::<T>::FeatureLocked
			);
			let Season { fee, .. } = Self::seasons(&avatar.season_id)?;
			T::Currency::withdraw(&from, fee.transfer_avatar, WithdrawReasons::FEE, AllowDeath)?;
			Self::deposit_into_treasury(&avatar.season_id, fee.transfer_avatar);

			Self::do_transfer_avatar(&from, &to, &avatar.season_id, &avatar_id)?;
			Self::deposit_event(Event::AvatarTransferred { from, to, avatar_id });
			Ok(())
		}

		/// Transfer free mints to a given account.
		///
		/// Emits `FreeMintsTransferred` event when successful.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(3)]
		#[pallet::weight(T::WeightInfo::transfer_free_mints())]
		pub fn transfer_free_mints(
			origin: OriginFor<T>,
			to: T::AccountId,
			how_many: MintCount,
		) -> DispatchResult {
			let from = ensure_signed(origin)?;
			ensure!(from != to, Error::<T>::CannotTransferToSelf);

			let GlobalConfig { freemint_transfer, .. } = GlobalConfigs::<T>::get();

			match freemint_transfer.mode {
				FreeMintTransferMode::Closed =>
					Err::<(), DispatchError>(Error::<T>::FreeMintTransferClosed.into()),
				FreeMintTransferMode::WhitelistOnly => {
					let whitelisted_accounts = WhitelistedAccounts::<T>::get();
					ensure!(
						whitelisted_accounts.contains(&from),
						Error::<T>::FreeMintTransferClosed
					);
					Ok(())
				},
				FreeMintTransferMode::Open => {
					let (season_id, _) = Self::current_season_with_id()?;
					let SeasonInfo { minted, forged, .. } = SeasonStats::<T>::get(season_id, &from);

					ensure!(
						minted > 0 && forged > 0,
						Error::<T>::CannotTransferFromInactiveAccount
					);

					Ok(())
				},
			}?;

			let GlobalConfig { freemint_transfer, .. } = GlobalConfigs::<T>::get();
			ensure!(
				how_many >= freemint_transfer.min_free_mint_transfer,
				Error::<T>::TooLowFreeMints
			);
			let sender_free_mints = PlayerConfigs::<T>::get(&from)
				.free_mints
				.checked_sub(
					how_many
						.checked_add(freemint_transfer.free_mint_transfer_fee)
						.ok_or(ArithmeticError::Overflow)?,
				)
				.ok_or(Error::<T>::InsufficientFreeMints)?;
			let dest_free_mints = PlayerConfigs::<T>::get(&to)
				.free_mints
				.checked_add(how_many)
				.ok_or(ArithmeticError::Overflow)?;

			PlayerConfigs::<T>::mutate(&from, |config| config.free_mints = sender_free_mints);
			PlayerConfigs::<T>::mutate(&to, |config| config.free_mints = dest_free_mints);

			Self::deposit_event(Event::FreeMintsTransferred { from, to, how_many });
			Ok(())
		}

		/// Set the price of a given avatar.
		///
		/// Only allowed while trade period is open.
		///
		/// Emits `AvatarPriceSet` event when successful.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(4)]
		#[pallet::weight(T::WeightInfo::set_price())]
		pub fn set_price(
			origin: OriginFor<T>,
			avatar_id: AvatarIdOf<T>,
			#[pallet::compact] price: BalanceOf<T>,
		) -> DispatchResult {
			let seller = ensure_signed(origin)?;
			ensure!(GlobalConfigs::<T>::get().trade.open, Error::<T>::TradeClosed);
			let avatar = Self::ensure_ownership(&seller, &avatar_id)?;
			ensure!(
				PlayerSeasonConfigs::<T>::get(&seller, avatar.season_id).locks.set_price,
				Error::<T>::FeatureLocked
			);
			Self::ensure_unlocked(&avatar_id)?;
			Self::ensure_tradable(&avatar)?;
			Trade::<T>::insert(avatar.season_id, avatar_id, price);
			Self::deposit_event(Event::AvatarPriceSet { avatar_id, price });
			Ok(())
		}

		/// Remove the price of a given avatar.
		///
		/// Only allowed while trade period is open.
		///
		/// Emits `AvatarPriceUnset` event when successful.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(5)]
		#[pallet::weight(T::WeightInfo::remove_price())]
		pub fn remove_price(origin: OriginFor<T>, avatar_id: AvatarIdOf<T>) -> DispatchResult {
			let seller = ensure_signed(origin)?;
			ensure!(GlobalConfigs::<T>::get().trade.open, Error::<T>::TradeClosed);
			Self::ensure_for_trade(&avatar_id)?;
			let avatar = Self::ensure_ownership(&seller, &avatar_id)?;
			Trade::<T>::remove(avatar.season_id, avatar_id);
			Self::deposit_event(Event::AvatarPriceUnset { avatar_id });
			Ok(())
		}

		/// Buy the given avatar.
		///
		/// It consumes tokens for the trade operation. The avatar will be owned by the
		/// player after the transaction.
		///
		/// Only allowed while trade period is open.
		///
		/// Emits `AvatarTraded` event when successful.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(6)]
		#[pallet::weight(T::WeightInfo::buy(MaxAvatarsPerPlayer::get()))]
		pub fn buy(origin: OriginFor<T>, avatar_id: AvatarIdOf<T>) -> DispatchResult {
			let buyer = ensure_signed(origin)?;
			let GlobalConfig { trade, affiliate_config, .. } = GlobalConfigs::<T>::get();
			ensure!(trade.open, Error::<T>::TradeClosed);

			let (seller, price) = Self::ensure_for_trade(&avatar_id)?;
			ensure!(buyer != seller, Error::<T>::AlreadyOwned);
			T::Currency::transfer(&buyer, &seller, price, AllowDeath)?;

			let avatar = Self::ensure_ownership(&seller, &avatar_id)?;
			let (current_season_id, Season { fee, .. }) = Self::current_season_with_id()?;

			let trade_fee = {
				let base_fee = fee.buy_minimum.max(
					price.saturating_mul(fee.buy_percent.unique_saturated_into()) /
						MAX_PERCENTAGE.unique_saturated_into(),
				);

				if affiliate_config.mode == AffiliateMode::Open && affiliate_config.enabled_in_buy {
					Self::try_propagate_chain_fee(AffiliateMethods::Buy, &buyer, base_fee)?
				} else {
					base_fee
				}
			};
			T::Currency::withdraw(&buyer, trade_fee, WithdrawReasons::FEE, AllowDeath)?;
			Self::deposit_into_treasury(&avatar.season_id, trade_fee);

			Self::do_transfer_avatar(&seller, &buyer, &avatar.season_id, &avatar_id)?;
			Trade::<T>::remove(avatar.season_id, avatar_id);

			SeasonStats::<T>::mutate(current_season_id, &buyer, |stats| {
				stats.bought.saturating_inc()
			});
			SeasonStats::<T>::mutate(current_season_id, &seller, |stats| {
				stats.sold.saturating_inc()
			});

			Self::deposit_event(Event::AvatarTraded { avatar_id, from: seller, to: buyer, price });
			Ok(())
		}

		/// Upgrade the avatar inventory space in a season.
		///
		/// * If called with a value in the **beneficiary** parameter, that account will receive the
		///   upgrade
		/// instead of the caller.
		/// * If the **in_season** parameter contains a value, this will set which specific season
		/// will the storage be upgraded for, if no value is set then the current season will be the
		/// one for which the storage will be upgraded.
		///
		/// In all cases the upgrade fees are **paid by the caller**.
		///
		/// Emits `StorageTierUpgraded` event when successful.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(7)]
		#[pallet::weight(T::WeightInfo::upgrade_storage())]
		pub fn upgrade_storage(
			origin: OriginFor<T>,
			beneficiary: Option<AccountIdFor<T>>,
			in_season: Option<SeasonId>,
		) -> DispatchResult {
			let caller = ensure_signed(origin)?;
			let (season_id, Season { fee, .. }) = {
				if let Some(season_id) = in_season {
					(season_id, Self::seasons(&season_id)?)
				} else {
					Self::current_season_with_id()?
				}
			};
			let account_to_upgrade = beneficiary.unwrap_or_else(|| caller.clone());

			let storage_tier =
				PlayerSeasonConfigs::<T>::get(&account_to_upgrade, season_id).storage_tier;
			ensure!(storage_tier != StorageTier::Max, Error::<T>::MaxStorageTierReached);

			let upgrade_fee = {
				let base_fee = fee.upgrade_storage;
				let GlobalConfig { affiliate_config, .. } = GlobalConfigs::<T>::get();

				if affiliate_config.mode == AffiliateMode::Open &&
					affiliate_config.enabled_in_upgrade
				{
					Self::try_propagate_chain_fee(
						AffiliateMethods::UpgradeStorage,
						&caller,
						base_fee,
					)?
				} else {
					base_fee
				}
			};

			T::Currency::withdraw(&caller, upgrade_fee, WithdrawReasons::FEE, AllowDeath)?;
			Self::deposit_into_treasury(&season_id, upgrade_fee);

			PlayerSeasonConfigs::<T>::mutate(&account_to_upgrade, season_id, |account| {
				account.storage_tier = storage_tier.upgrade()
			});
			Self::deposit_event(Event::StorageTierUpgraded {
				account: account_to_upgrade,
				season_id,
			});
			Ok(())
		}

		/// Set game organizer.
		///
		/// The organizer account is like an admin, allowed to perform certain operations
		/// related with the game configuration like `set_season`, `ensure_free_mint` and
		/// `update_global_config`.
		///
		/// It can only be set by a root account.
		///
		/// Emits `OrganizerSet` event when successful.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(8)]
		#[pallet::weight(T::WeightInfo::set_organizer())]
		pub fn set_organizer(origin: OriginFor<T>, organizer: T::AccountId) -> DispatchResult {
			ensure_root(origin)?;
			Organizer::<T>::put(&organizer);
			Self::deposit_event(Event::OrganizerSet { organizer });
			Ok(())
		}

		/// Set treasurer.
		///
		/// This is an additional treasury.
		///
		/// It can only be set by a root account.
		///
		/// Emits `TreasurerSet` event when successful.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(9)]
		#[pallet::weight(T::WeightInfo::set_treasurer())]
		pub fn set_treasurer(
			origin: OriginFor<T>,
			season_id: SeasonId,
			treasurer: T::AccountId,
		) -> DispatchResult {
			ensure_root(origin)?;
			Treasurer::<T>::insert(season_id, &treasurer);
			Self::deposit_event(Event::TreasurerSet { season_id, treasurer });
			Ok(())
		}

		/// Claim treasury of a season.
		///
		/// The origin of this call must be signed by a treasurer account associated with the given
		/// season ID. The treasurer of a season can claim the season's associated treasury once the
		/// season finishes.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(10)]
		#[pallet::weight(T::WeightInfo::claim_treasury())]
		pub fn claim_treasury(origin: OriginFor<T>, season_id: SeasonId) -> DispatchResult {
			let maybe_treasurer = ensure_signed(origin)?;
			let treasurer = Treasurer::<T>::get(season_id).ok_or(Error::<T>::UnknownTreasurer)?;
			ensure!(maybe_treasurer == treasurer, DispatchError::BadOrigin);

			let (current_season_id, season_schedule) = Self::current_season_schedule_with_id()?;
			ensure!(
				season_id < current_season_id ||
					(season_id == current_season_id &&
						<frame_system::Pallet<T>>::block_number() > season_schedule.end),
				Error::<T>::CannotClaimDuringSeason
			);

			let amount = Treasury::<T>::take(season_id);
			ensure!(!amount.is_zero(), Error::<T>::CannotClaimZero);

			T::Currency::transfer(&Self::treasury_account_id(), &treasurer, amount, AllowDeath)?;
			Self::deposit_event(Event::TreasuryClaimed { season_id, treasurer, amount });
			Ok(())
		}

		/// Set season.
		///
		/// Creates a new season. The new season can overlap with the already existing.
		///
		/// It can only be set by an organizer account.
		///
		/// Emits `UpdatedSeason` event when successful.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(11)]
		#[pallet::weight(T::WeightInfo::set_season())]
		pub fn set_season(
			origin: OriginFor<T>,
			season_id: SeasonId,
			season: Option<SeasonOf<T>>,
			season_meta: Option<SeasonMeta>,
			season_schedule: Option<SeasonScheduleOf<T>>,
			trade_filters: Option<TradeFilters>,
		) -> DispatchResult {
			Self::ensure_organizer(origin)?;

			if let Some(mut season_update) = season.clone() {
				season_update.validate::<T>()?;
				Seasons::<T>::insert(season_id, &season_update);
			}

			if let Some(meta_update) = season_meta.clone() {
				SeasonMetas::<T>::insert(season_id, meta_update);
			}

			if let Some(schedule_update) = season_schedule.clone() {
				Self::ensure_season_schedule(season_id, &schedule_update)?;
				SeasonSchedules::<T>::insert(season_id, &schedule_update);
			}

			if let Some(filter_update) = trade_filters.clone() {
				SeasonTradeFilters::<T>::insert(season_id, filter_update);
			}

			Self::deposit_event(Event::UpdatedSeason {
				season_id,
				season,
				meta: season_meta,
				schedule: season_schedule,
				trade_filters,
			});

			Ok(())
		}

		/// Update global configuration.
		///
		/// It can only be called by an organizer account.
		///
		/// Emits `UpdatedGlobalConfig` event when successful.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(12)]
		#[pallet::weight(T::WeightInfo::update_global_config())]
		pub fn update_global_config(
			origin: OriginFor<T>,
			new_global_config: GlobalConfigOf<T>,
		) -> DispatchResult {
			Self::ensure_organizer(origin)?;
			GlobalConfigs::<T>::put(&new_global_config);
			Self::deposit_event(Event::UpdatedGlobalConfig(new_global_config));
			Ok(())
		}

		/// Set free mints.
		///
		/// It can only be called by an organizer account.
		///
		/// Emits `FreeMintSet` event when successful.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(13)]
		#[pallet::weight(T::WeightInfo::set_free_mints())]
		pub fn set_free_mints(
			origin: OriginFor<T>,
			target: T::AccountId,
			how_many: MintCount,
		) -> DispatchResult {
			Self::ensure_organizer(origin)?;
			PlayerConfigs::<T>::mutate(&target, |config| config.free_mints = how_many);
			Self::deposit_event(Event::FreeMintsSet { target, how_many });
			Ok(())
		}

		/// Locks an avatar to be tokenized as an NFT.
		///
		/// The origin of this call must specify an avatar, owned by the origin, to prevent it from
		/// forging, trading and transferring it to other players. When successful, the ownership of
		/// the avatar is removed from the player.
		///
		/// Locking an avatar allows for new
		/// ways of interacting with it currently under development.
		///
		/// Weight: `O(n)` where:
		/// - `n = max avatars per player`
		#[pallet::call_index(15)]
		#[pallet::weight(T::WeightInfo::lock_avatar(MaxAvatarsPerPlayer::get()))]
		pub fn lock_avatar(origin: OriginFor<T>, avatar_id: AvatarIdOf<T>) -> DispatchResult {
			let player = ensure_signed(origin)?;

			Self::lock_asset(T::PalletId::get().0, player, avatar_id)?;

			Ok(())
		}

		/// Unlocks an avatar removing its NFT representation.
		///
		/// The origin of this call must specify an avatar, owned and locked by the origin, to allow
		/// forging, trading and transferring it to other players. When successful, the ownership of
		/// the avatar is transferred from the pallet's technical account back to the player and its
		/// existing NFT representation is destroyed.
		///
		/// Weight: `O(n)` where:
		/// - `n = max avatars per player`
		#[pallet::call_index(16)]
		#[pallet::weight(T::WeightInfo::unlock_avatar(MaxAvatarsPerPlayer::get()))]
		pub fn unlock_avatar(origin: OriginFor<T>, avatar_id: AvatarIdOf<T>) -> DispatchResult {
			let player = ensure_signed(origin)?;

			Self::unlock_asset(T::PalletId::get().0, player, avatar_id)?;
			Ok(())
		}

		#[pallet::call_index(21)]
		#[pallet::weight({1000})]
		pub fn modify_freemint_whitelist(
			origin: OriginFor<T>,
			account: AccountIdFor<T>,
			operation: WhitelistOperation,
		) -> DispatchResult {
			let _ = Self::ensure_organizer(origin)?;

			match operation {
				WhitelistOperation::AddAccount =>
					WhitelistedAccounts::<T>::try_mutate(move |account_list| {
						ensure!(
							!account_list.contains(&account),
							Error::<T>::AccountAlreadyInWhitelist
						);

						account_list
							.try_push(account)
							.map_err(|_| Error::<T>::WhitelistedAccountsLimitReached.into())
					}),
				WhitelistOperation::RemoveAccount =>
					WhitelistedAccounts::<T>::try_mutate(move |account_list| {
						account_list.retain(|entry| entry != &account);

						Ok(())
					}),
				WhitelistOperation::ClearList =>
					WhitelistedAccounts::<T>::try_mutate(move |account_list| {
						account_list.clear();

						Ok(())
					}),
			}
		}

		#[pallet::call_index(22)]
		#[pallet::weight({1000})]
		pub fn add_affiliation(
			origin: OriginFor<T>,
			target_affiliatee: Option<AccountIdFor<T>>,
			affiliate_id: AffiliateId,
		) -> DispatchResult {
			let signer = ensure_signed(origin)?;

			let account = if let Some(acc) = target_affiliatee {
				let whitelisted_accounts = WhitelistedAccounts::<T>::get();
				ensure!(
					whitelisted_accounts.contains(&signer),
					Error::<T>::AffiliateOthersOnlyWhiteListed
				);
				acc
			} else {
				signer
			};

			if let Some(affiliator) = T::AffiliateHandler::get_account_for_id(affiliate_id) {
				T::AffiliateHandler::try_add_affiliate_to(&affiliator, &account)
			} else {
				Err(Error::<T>::AffiliatorNotFound.into())
			}
		}

		#[pallet::call_index(23)]
		#[pallet::weight({1000})]
		pub fn enable_affiliator(
			origin: OriginFor<T>,
			target: UnlockTarget<T::AccountId>,
			season_id: SeasonId,
		) -> DispatchResult {
			let account = ensure_signed(origin)?;
			ensure!(Seasons::<T>::contains_key(season_id), Error::<T>::UnknownSeason);

			match target {
				UnlockTarget::OneselfFree => {
					// Check criteria
					if let Some(UnlockConfigs { affiliate_unlock: Some(unlock_vec), .. }) =
						SeasonUnlocks::<T>::get(season_id)
					{
						let player_stats = SeasonStats::<T>::get(season_id, &account);

						if Self::evaluate_unlock_state(&unlock_vec, &player_stats) {
							PlayerSeasonConfigs::<T>::mutate(&account, season_id, |config| {
								config.locks.affiliate = true;
							});

							T::AffiliateHandler::try_mark_account_as_affiliatable(&account)
						} else {
							Err(Error::<T>::UnlockCriteriaNotFulfilled.into())
						}
					} else {
						Err(Error::<T>::FeatureLockedInSeason.into())
					}
				},
				UnlockTarget::OneselfPaying => {
					// Substract amout for paying if account not affiliator
					PlayerSeasonConfigs::<T>::try_mutate(&account, season_id, |config| {
						if !config.locks.affiliate {
							let GlobalConfig { affiliate_config, .. } = GlobalConfigs::<T>::get();
							ensure!(
								affiliate_config.affiliator_enable_fee > 0_u32.into(),
								Error::<T>::FeatureLockedThroughPayment
							);
							T::Currency::transfer(
								&account,
								&Self::treasury_account_id(),
								affiliate_config.affiliator_enable_fee,
								AllowDeath,
							)?;
							T::AffiliateHandler::try_mark_account_as_affiliatable(&account)?;
							config.locks.affiliate = true;
						}
						Ok(())
					})
				},
				UnlockTarget::OtherPaying(other) => {
					// Substract amout for paying if other not affiliator
					PlayerSeasonConfigs::<T>::try_mutate(&other, season_id, |config| {
						if !config.locks.affiliate {
							let GlobalConfig { affiliate_config, .. } = GlobalConfigs::<T>::get();
							ensure!(
								affiliate_config.affiliator_enable_fee > 0_u32.into(),
								Error::<T>::FeatureLockedThroughPayment
							);
							T::Currency::transfer(
								&account,
								&Self::treasury_account_id(),
								affiliate_config.affiliator_enable_fee,
								AllowDeath,
							)?;
							T::AffiliateHandler::try_mark_account_as_affiliatable(&other)?;
							config.locks.affiliate = true;
						}
						Ok(())
					})
				},
			}
		}

		#[pallet::call_index(24)]
		#[pallet::weight({1000})]
		pub fn remove_affiliation(origin: OriginFor<T>, account: T::AccountId) -> DispatchResult {
			let _ = Self::ensure_organizer(origin)?;
			T::AffiliateHandler::try_clear_affiliation_for(&account)
		}

		#[pallet::call_index(25)]
		#[pallet::weight({1000})]
		pub fn set_rule_for(
			origin: OriginFor<T>,
			rule_id: AffiliateMethods,
			rule: FeePropagationOf<T>,
		) -> DispatchResult {
			let _ = Self::ensure_organizer(origin)?;

			T::AffiliateHandler::try_add_rule_for(rule_id, rule)
		}

		#[pallet::call_index(26)]
		#[pallet::weight({1000})]
		pub fn clear_rule_for(origin: OriginFor<T>, rule_id: AffiliateMethods) -> DispatchResult {
			let _ = Self::ensure_organizer(origin)?;

			T::AffiliateHandler::clear_rule_for(rule_id);

			Ok(())
		}

		#[pallet::call_index(27)]
		#[pallet::weight({1000})]
		pub fn enable_set_avatar_price(
			origin: OriginFor<T>,
			target: UnlockTarget<T::AccountId>,
			season_id: SeasonId,
		) -> DispatchResult {
			let account = ensure_signed(origin)?;
			ensure!(Seasons::<T>::contains_key(season_id), Error::<T>::UnknownSeason);

			match target {
				UnlockTarget::OneselfFree => {
					if let Some(UnlockConfigs { set_price_unlock: Some(unlock_vec), .. }) =
						SeasonUnlocks::<T>::get(season_id)
					{
						let player_stats = SeasonStats::<T>::get(season_id, &account);

						if Self::evaluate_unlock_state(&unlock_vec, &player_stats) {
							PlayerSeasonConfigs::<T>::mutate(account, season_id, |config| {
								config.locks.set_price = true;
							});

							Ok(())
						} else {
							Err(Error::<T>::UnlockCriteriaNotFulfilled.into())
						}
					} else {
						Err(Error::<T>::FeatureLockedInSeason.into())
					}
				},
				UnlockTarget::OneselfPaying => {
					// Substract amout for paying if account not set price unlocked
					PlayerSeasonConfigs::<T>::try_mutate(&account, season_id, |config| {
						if !config.locks.set_price {
							let Season { fee, .. } = Self::seasons(&season_id)?;
							ensure!(
								fee.set_price_unlock > 0_u32.into(),
								Error::<T>::FeatureLockedThroughPayment
							);
							T::Currency::transfer(
								&account,
								&Self::treasury_account_id(),
								fee.set_price_unlock,
								AllowDeath,
							)?;
							config.locks.set_price = true;
						}
						Ok(())
					})
				},
				UnlockTarget::OtherPaying(other) => {
					// Substract amout for paying if other not affiliator
					PlayerSeasonConfigs::<T>::try_mutate(&other, season_id, |config| {
						if !config.locks.set_price {
							let Season { fee, .. } = Self::seasons(&season_id)?;
							ensure!(
								fee.set_price_unlock > 0_u32.into(),
								Error::<T>::FeatureLockedThroughPayment
							);
							T::Currency::transfer(
								&account,
								&Self::treasury_account_id(),
								fee.set_price_unlock,
								AllowDeath,
							)?;
							config.locks.set_price = true;
						}
						Ok(())
					})
				},
			}
		}

		#[pallet::call_index(28)]
		#[pallet::weight({1000})]
		pub fn enable_avatar_transfer(
			origin: OriginFor<T>,
			target: UnlockTarget<T::AccountId>,
			season_id: SeasonId,
		) -> DispatchResult {
			let account = ensure_signed(origin)?;
			ensure!(Seasons::<T>::contains_key(season_id), Error::<T>::UnknownSeason);

			match target {
				UnlockTarget::OneselfFree => {
					if let Some(UnlockConfigs {
						avatar_transfer_unlock: Some(unlock_vec), ..
					}) = SeasonUnlocks::<T>::get(season_id)
					{
						let player_stats = SeasonStats::<T>::get(season_id, &account);

						if Self::evaluate_unlock_state(&unlock_vec, &player_stats) {
							PlayerSeasonConfigs::<T>::mutate(account, season_id, |config| {
								config.locks.avatar_transfer = true;
							});

							Ok(())
						} else {
							Err(Error::<T>::UnlockCriteriaNotFulfilled.into())
						}
					} else {
						Err(Error::<T>::FeatureLockedInSeason.into())
					}
				},
				UnlockTarget::OneselfPaying => {
					// Substract amout for paying if account not set price unlocked
					PlayerSeasonConfigs::<T>::try_mutate(&account, season_id, |config| {
						if !config.locks.avatar_transfer {
							let Season { fee, .. } = Self::seasons(&season_id)?;
							ensure!(
								fee.avatar_transfer_unlock > 0_u32.into(),
								Error::<T>::FeatureLockedThroughPayment
							);
							T::Currency::transfer(
								&account,
								&Self::treasury_account_id(),
								fee.avatar_transfer_unlock,
								AllowDeath,
							)?;
							config.locks.avatar_transfer = true;
						}
						Ok(())
					})
				},
				UnlockTarget::OtherPaying(other) => {
					// Substract amout for paying if other not affiliator
					PlayerSeasonConfigs::<T>::try_mutate(&other, season_id, |config| {
						if !config.locks.avatar_transfer {
							let Season { fee, .. } = Self::seasons(&season_id)?;
							ensure!(
								fee.avatar_transfer_unlock > 0_u32.into(),
								Error::<T>::FeatureLockedThroughPayment
							);
							T::Currency::transfer(
								&account,
								&Self::treasury_account_id(),
								fee.avatar_transfer_unlock,
								AllowDeath,
							)?;
							config.locks.avatar_transfer = true;
						}
						Ok(())
					})
				},
			}
		}

		#[pallet::call_index(29)]
		#[pallet::weight({1000})]
		pub fn set_unlock_config(
			origin: OriginFor<T>,
			season_id: SeasonId,
			unlock_configs: UnlockConfigs,
		) -> DispatchResult {
			Self::ensure_organizer(origin)?;
			SeasonUnlocks::<T>::insert(season_id, &unlock_configs);
			Self::deposit_event(Event::UpdatedUnlockConfigs { season_id, unlock_configs });
			Ok(())
		}

		#[pallet::call_index(30)]
		#[pallet::weight({10_000})]
		pub fn create_tournament(
			origin: OriginFor<T>,
			season_id: SeasonId,
			config: TournamentConfigFor<T>,
			with_ranker: AvatarRankerFor<T>,
		) -> DispatchResult {
			let organizer = Self::ensure_organizer(origin)?;
			let tournament_id = T::TournamentHandler::try_create_new_tournament_for(
				&organizer, &season_id, config,
			)?;

			TournamentRankers::<T>::insert(season_id, tournament_id, with_ranker);

			Ok(())
		}

		#[pallet::call_index(31)]
		#[pallet::weight({10_000})]
		pub fn remove_latest_tournament(
			origin: OriginFor<T>,
			season_id: SeasonId,
		) -> DispatchResult {
			let _ = Self::ensure_organizer(origin)?;
			T::TournamentHandler::try_remove_latest_tournament_for(&season_id)
		}

		#[pallet::call_index(32)]
		#[pallet::weight({10_000})]
		pub fn claim_tournament_reward_for(
			origin: OriginFor<T>,
			season_id: SeasonId,
			avatar_id: AvatarIdOf<T>,
		) -> DispatchResult {
			let account = ensure_signed(origin)?;
			Self::ensure_ownership(&account, &avatar_id)?;

			T::TournamentHandler::try_claim_tournament_reward_for(&season_id, &account, &avatar_id)
		}

		#[pallet::call_index(33)]
		#[pallet::weight({10_000})]
		pub fn claim_golden_duck_for(
			origin: OriginFor<T>,
			season_id: SeasonId,
			avatar_id: AvatarIdOf<T>,
		) -> DispatchResult {
			let account = ensure_signed(origin)?;
			Self::ensure_ownership(&account, &avatar_id)?;

			T::TournamentHandler::try_claim_golden_duck_for(&season_id, &account, &avatar_id)
		}

		#[pallet::call_index(34)]
		#[pallet::weight({10_000})]
		pub fn force_set_affiliatee_state(
			origin: OriginFor<T>,
			account: AccountIdFor<T>,
			chain: Vec<AccountIdFor<T>>,
		) -> DispatchResult {
			ensure_root(origin)?;

			T::AffiliateHandler::force_set_affiliatee_chain_for(&account, chain)
		}
	}

	impl<T: Config> Pallet<T> {
		/// The account ID of the treasury.
		pub fn treasury_account_id() -> T::AccountId {
			T::PalletId::get().into_account_truncating()
		}

		/// The account ID of the treasury.
		pub fn technical_account_id() -> T::AccountId {
			T::PalletId::get().into_sub_account_truncating(b"technical")
		}

		pub(crate) fn deposit_into_treasury(season_id: &SeasonId, amount: BalanceOf<T>) {
			Treasury::<T>::mutate(season_id, |bal| bal.saturating_accrue(amount));
			T::Currency::deposit_creating(&Self::treasury_account_id(), amount);
		}

		/// Check that the origin is an organizer account.
		pub(crate) fn ensure_organizer(
			origin: OriginFor<T>,
		) -> Result<T::AccountId, DispatchError> {
			let maybe_organizer = ensure_signed(origin)?;
			let existing_organizer = Organizer::<T>::get().ok_or(Error::<T>::OrganizerNotSet)?;
			ensure!(maybe_organizer == existing_organizer, DispatchError::BadOrigin);
			Ok(maybe_organizer)
		}

		pub(crate) fn ensure_season_schedule(
			season_id: SeasonId,
			season_schedule: &SeasonScheduleOf<T>,
		) -> DispatchResult {
			season_schedule.validate::<T>()?;

			let prev_season_id = season_id.checked_sub(1).ok_or(ArithmeticError::Underflow)?;
			let next_season_id = season_id.checked_add(1).ok_or(ArithmeticError::Overflow)?;

			if prev_season_id > 0 {
				let prev_schedule = SeasonSchedules::<T>::get(prev_season_id)
					.ok_or(Error::<T>::NonSequentialSeasonId)?;
				ensure!(
					prev_schedule.end < season_schedule.early_start,
					Error::<T>::EarlyStartTooEarly
				);
			}
			if let Some(next_schedule) = SeasonSchedules::<T>::get(next_season_id) {
				ensure!(
					season_schedule.end < next_schedule.early_start,
					Error::<T>::SeasonEndTooLate
				);
			}

			Ok(())
		}

		pub(crate) fn random_hash(phrase: &[u8], who: &T::AccountId) -> T::Hash {
			let (seed, _) = T::Randomness::random(phrase);
			let seed = T::Hash::decode(&mut TrailingZeroInput::new(seed.as_ref()))
				.expect("input is padded with zeroes; qed");
			let nonce = frame_system::Pallet::<T>::account_nonce(who);
			frame_system::Pallet::<T>::inc_account_nonce(who);
			(seed, &who, nonce.encode()).using_encoded(T::Hashing::hash)
		}

		/// Mint a new avatar.
		pub(crate) fn do_mint(player: &T::AccountId, mint_option: &MintOption) -> DispatchResult {
			let (season_id, season) = Self::current_season_with_id()?;

			Self::ensure_for_mint(player, &season_id, mint_option)?;

			let generated_avatar_ids = match season.mint_logic {
				LogicGeneration::First => MinterV1::<T>::mint(player, &season_id, mint_option),
				LogicGeneration::Second => MinterV2::<T>::mint(player, &season_id, mint_option),
				LogicGeneration::Third => MinterV3::<T>::mint(player, &season_id, mint_option),
				LogicGeneration::Fourth => MinterV4::<T>::mint(player, &season_id, mint_option),
			}?;

			let is_tournament_in_active_period = matches!(
				T::TournamentHandler::get_active_tournament_state_for(&season_id),
				TournamentState::ActivePeriod(_)
			);

			let GlobalConfig { mint, affiliate_config, .. } = GlobalConfigs::<T>::get();
			match mint_option.payment {
				MintPayment::Normal => {
					let mint_fee = {
						let base_fee = season.fee.mint.fee_for(&mint_option.pack_size);

						let updated_fee =
							match T::TournamentHandler::get_active_tournament_config_for(&season_id)
							{
								Some((
									_,
									TournamentConfig {
										take_fee_percentage: Some(fee_perc), ..
									},
								)) if is_tournament_in_active_period => Self::try_propagate_tournament_fee(
									&season_id, player, fee_perc, base_fee,
								)?,
								_ => base_fee,
							};

						if affiliate_config.mode == AffiliateMode::Open &&
							affiliate_config.enabled_in_mint
						{
							Self::try_propagate_chain_fee(
								AffiliateMethods::Mint,
								player,
								updated_fee,
							)?
						} else {
							updated_fee
						}
					};

					T::Currency::withdraw(player, mint_fee, WithdrawReasons::FEE, AllowDeath)?;
					Self::deposit_into_treasury(&season_id, mint_fee);
				},
				MintPayment::Free => {
					let mint_fee = (mint_option.pack_size.as_mint_count())
						.saturating_mul(mint.free_mint_fee_multiplier);
					PlayerConfigs::<T>::try_mutate(player, |config| -> DispatchResult {
						config.free_mints = config
							.free_mints
							.checked_sub(mint_fee)
							.ok_or(Error::<T>::InsufficientFreeMints)?;
						Ok(())
					})?;
				},
			};

			PlayerSeasonConfigs::<T>::try_mutate(
				player,
				season_id,
				|PlayerSeasonConfig { stats, .. }| -> DispatchResult {
					let current_block = <frame_system::Pallet<T>>::block_number();
					if stats.mint.first.is_zero() {
						stats.mint.first = current_block;
					}
					stats.mint.last = current_block;
					Ok(())
				},
			)?;
			SeasonStats::<T>::mutate(season_id, player, |info| match mint_option.payment {
				MintPayment::Free =>
					info.free_minted.saturating_accrue(generated_avatar_ids.len() as Stat),
				MintPayment::Normal =>
					info.minted.saturating_accrue(generated_avatar_ids.len() as Stat),
			});

			if is_tournament_in_active_period &&
				T::TournamentHandler::is_golden_duck_enabled_for(&season_id)
			{
				for avatar_id in generated_avatar_ids.iter() {
					T::TournamentHandler::try_rank_entity_for_golden_duck(&season_id, avatar_id)?;
				}
			}

			Self::deposit_event(Event::AvatarsMinted { avatar_ids: generated_avatar_ids });
			Ok(())
		}

		/// Enhance an avatar using a batch of avatars.
		pub(crate) fn do_forge(
			player: &T::AccountId,
			leader_id: &AvatarIdOf<T>,
			sacrifice_ids: Vec<AvatarIdOf<T>>,
		) -> DispatchResult {
			let GlobalConfig { forge, .. } = GlobalConfigs::<T>::get();
			ensure!(forge.open, Error::<T>::ForgeClosed);

			let (leader, sacrifice_ids, sacrifices, season_id, season) =
				Self::ensure_for_forge(player, leader_id, sacrifice_ids)?;

			let avatar_count = Owners::<T>::get(player, season_id).len();
			let max_storage =
				PlayerSeasonConfigs::<T>::get(player, season_id).storage_tier as usize;
			let restricted_forge = max_storage == avatar_count;

			let input_leader = (*leader_id, leader);
			let input_sacrifices =
				sacrifice_ids.into_iter().zip(sacrifices).collect::<Vec<ForgeItem<T>>>();
			let (output_leader, output_other) = match season.forge_logic {
				LogicGeneration::First => ForgerV1::<T>::forge(
					player,
					season_id,
					&season,
					input_leader.clone(),
					input_sacrifices.clone(),
					false,
				),
				LogicGeneration::Second => ForgerV2::<T>::forge(
					player,
					season_id,
					&season,
					input_leader.clone(),
					input_sacrifices.clone(),
					restricted_forge,
				),
				LogicGeneration::Third => ForgerV3::<T>::forge(
					player,
					season_id,
					&season,
					input_leader.clone(),
					input_sacrifices.clone(),
					false,
				),
				LogicGeneration::Fourth => ForgerV4::<T>::forge(
					player,
					season_id,
					&season,
					input_leader.clone(),
					input_sacrifices.clone(),
					restricted_forge,
				),
			}?;

			Self::process_leader_forge_output(
				player,
				&season_id,
				&season,
				input_leader,
				output_leader,
				input_sacrifices,
			)?;
			Self::process_other_forge_outputs(player, &season_id, output_other)?;
			Self::update_forging_statistics_for_player(player, season_id)?;
			Ok(())
		}

		fn do_transfer_avatar(
			from: &T::AccountId,
			to: &T::AccountId,
			season_id: &SeasonId,
			avatar_id: &AvatarIdOf<T>,
		) -> DispatchResult {
			let mut from_avatar_ids = Owners::<T>::get(from, season_id);
			from_avatar_ids.retain(|existing_avatar_id| existing_avatar_id != avatar_id);

			let mut to_avatar_ids = Owners::<T>::get(to, season_id);
			to_avatar_ids
				.try_push(*avatar_id)
				.map_err(|_| Error::<T>::MaxOwnershipReached)?;
			ensure!(
				to_avatar_ids.len() <=
					PlayerSeasonConfigs::<T>::get(to, season_id).storage_tier as usize,
				Error::<T>::MaxOwnershipReached
			);

			Owners::<T>::mutate(from, season_id, |avatar_ids| *avatar_ids = from_avatar_ids);
			Owners::<T>::mutate(to, season_id, |avatar_ids| *avatar_ids = to_avatar_ids);
			Avatars::<T>::try_mutate(avatar_id, |maybe_avatar| -> DispatchResult {
				let (from_owner, _) = maybe_avatar.as_mut().ok_or(Error::<T>::UnknownAvatar)?;
				*from_owner = to.clone();
				Ok(())
			})
		}

		fn current_season_with_id() -> Result<(SeasonId, SeasonOf<T>), DispatchError> {
			let mut current_status = CurrentSeasonStatus::<T>::get();
			let season = match Seasons::<T>::get(current_status.season_id) {
				Some(season) if current_status.is_in_season() => season,
				_ => {
					if current_status.season_id > 1 {
						current_status.season_id.saturating_dec();
					}
					Self::seasons(&current_status.season_id)?
				},
			};
			Ok((current_status.season_id, season))
		}

		fn current_season_schedule_with_id(
		) -> Result<(SeasonId, SeasonScheduleOf<T>), DispatchError> {
			let mut current_status = CurrentSeasonStatus::<T>::get();
			let season_schedule = match SeasonSchedules::<T>::get(current_status.season_id) {
				Some(season_schedule) if current_status.is_in_season() => season_schedule,
				_ => {
					if current_status.season_id > 1 {
						current_status.season_id.saturating_dec();
					}
					Self::season_schedules(&current_status.season_id)?
				},
			};
			Ok((current_status.season_id, season_schedule))
		}

		fn season_with_id_for(
			avatar: &AvatarOf<T>,
		) -> Result<(SeasonId, SeasonOf<T>), DispatchError> {
			let season_id = avatar.season_id;
			let season = Self::seasons(&season_id)?;

			Ok((season_id, season))
		}

		pub(crate) fn ensure_for_mint(
			player: &T::AccountId,
			season_id: &SeasonId,
			mint_option: &MintOption,
		) -> DispatchResult {
			let GlobalConfig { mint, .. } = GlobalConfigs::<T>::get();
			ensure!(mint.open, Error::<T>::MintClosed);

			let player_season_config = PlayerSeasonConfigs::<T>::get(player, season_id);

			let current_block = <frame_system::Pallet<T>>::block_number();
			let last_block = player_season_config.stats.mint.last;
			if !last_block.is_zero() {
				ensure!(current_block >= last_block + mint.cooldown, Error::<T>::MintCooldown);
			}

			let SeasonStatus { active, early, early_ended, .. } = CurrentSeasonStatus::<T>::get();
			let free_mints = PlayerConfigs::<T>::get(player).free_mints;
			let is_whitelisted = free_mints > Zero::zero();
			let is_free_mint = mint_option.payment == MintPayment::Free;
			ensure!(!early_ended || is_free_mint, Error::<T>::PrematureSeasonEnd);
			ensure!(active || early && (is_whitelisted || is_free_mint), Error::<T>::SeasonClosed);

			let mint_count = mint_option.pack_size.as_mint_count();
			let (_, Season { fee, .. }) = Self::current_season_with_id()?;
			match mint_option.payment {
				MintPayment::Normal => {
					let fee = fee.mint.fee_for(&mint_option.pack_size);
					T::Currency::free_balance(player)
						.checked_sub(&fee)
						.ok_or(Error::<T>::InsufficientBalance)?;
				},
				MintPayment::Free => {
					let fee = mint_count.saturating_mul(mint.free_mint_fee_multiplier);
					free_mints.checked_sub(fee).ok_or(Error::<T>::InsufficientFreeMints)?;
				},
			};

			let current_count = Owners::<T>::get(player, season_id).len() as u16;
			let max_count = player_season_config.storage_tier as u16;
			ensure!(current_count + mint_count <= max_count, Error::<T>::MaxOwnershipReached);
			Ok(())
		}

		fn ensure_for_forge(
			player: &T::AccountId,
			leader_id: &AvatarIdOf<T>,
			sacrifice_ids: Vec<AvatarIdOf<T>>,
		) -> Result<
			(AvatarOf<T>, Vec<AvatarIdOf<T>>, Vec<AvatarOf<T>>, SeasonId, SeasonOf<T>),
			DispatchError,
		> {
			let sacrifice_count = sacrifice_ids.len() as u8;

			let leader = Self::ensure_ownership(player, leader_id)?;
			let (season_id, season) = Self::season_with_id_for(&leader)?;

			ensure!(sacrifice_count >= season.min_sacrifices, Error::<T>::TooFewSacrifices);
			ensure!(sacrifice_count <= season.max_sacrifices, Error::<T>::TooManySacrifices);
			ensure!(!sacrifice_ids.contains(leader_id), Error::<T>::LeaderSacrificed);
			ensure!(
				sacrifice_ids.iter().all(|id| Self::ensure_for_trade(id).is_err()),
				Error::<T>::AvatarInTrade
			);
			ensure!(Self::ensure_for_trade(leader_id).is_err(), Error::<T>::AvatarInTrade);
			Self::ensure_unlocked(leader_id)?;

			let deduplicated_sacrifice_ids = {
				let mut id_queue = sacrifice_ids.into_iter().collect::<VecDeque<_>>();

				let mut dedup_id_list = Vec::with_capacity(4);

				while let Some(item) = id_queue.pop_front() {
					if !dedup_id_list.contains(&item) {
						dedup_id_list.push(item);
					}
				}

				dedup_id_list
			};
			let sacrifices = deduplicated_sacrifice_ids
				.iter()
				.map(|id| {
					let avatar = Self::ensure_ownership(player, id)?;
					ensure!(avatar.season_id == season_id, Error::<T>::IncorrectAvatarSeason);
					ensure!(
						avatar.encoding == leader.encoding,
						Error::<T>::IncompatibleAvatarVersions
					);
					Self::ensure_unlocked(id)?;
					Ok(avatar)
				})
				.collect::<Result<Vec<AvatarOf<T>>, DispatchError>>()?;

			Ok((leader, deduplicated_sacrifice_ids, sacrifices, season_id, season))
		}

		fn process_leader_forge_output(
			player: &AccountIdFor<T>,
			season_id: &SeasonId,
			season: &SeasonOf<T>,
			input_leader: ForgeItem<T>,
			output_leader: LeaderForgeOutput<T>,
			input_sacrifices: Vec<ForgeItem<T>>,
		) -> DispatchResult {
			match output_leader {
				LeaderForgeOutput::Forged((leader_id, leader), upgraded_components) => {
					let prev_leader_tier = input_leader.1.rarity();
					let after_leader_tier = leader.rarity();
					let max_tier = season.max_tier() as u8;

					if prev_leader_tier != max_tier && after_leader_tier == max_tier {
						CurrentSeasonStatus::<T>::mutate(|status| {
							status.max_tier_avatars.saturating_inc();
							if status.max_tier_avatars == season.max_tier_forges {
								status.early_ended = true;
							}
						});

						let is_tournament_in_active_period = matches!(
							T::TournamentHandler::get_active_tournament_state_for(season_id),
							TournamentState::ActivePeriod(_)
						);

						// If the leader avatar has turned into a Legendary avatar and
						// the tournament is in its active phase then we try to rank it
						if is_tournament_in_active_period {
							if let Some((tournament_id, config)) =
								T::TournamentHandler::get_active_tournament_config_for(season_id)
							{
								let sacrifices_are_in_bounds =
									input_sacrifices.into_iter().all(|(_, sacrifice)| {
										sacrifice.minted_at >= config.start &&
											sacrifice.minted_at <= config.active_end
									});

								let leader_in_bounds = input_leader.1.minted_at >= config.start &&
									input_leader.1.minted_at <= config.active_end;

								if sacrifices_are_in_bounds && leader_in_bounds {
									let ranker =
										TournamentRankers::<T>::get(season_id, tournament_id)
											.ok_or(Error::<T>::TournamentRankerNotFound)?;

									T::TournamentHandler::try_rank_entity_in_tournament_for(
										season_id, &leader_id, &leader, &ranker,
									)?;
								}
							}
						}
					}

					Avatars::<T>::insert(leader_id, (player, leader));

					// TODO: May change in the future
					Self::deposit_event(Event::AvatarsForged {
						avatar_ids: vec![(leader_id, upgraded_components)],
					});
				},
				LeaderForgeOutput::Consumed(leader_id) =>
					Self::remove_avatar_from(player, season_id, &leader_id),
				LeaderForgeOutput::Unchanged(_) => {},
			}

			Ok(())
		}

		fn process_other_forge_outputs(
			player: &AccountIdFor<T>,
			season_id: &SeasonId,
			other_outputs: Vec<ForgeOutput<T>>,
		) -> DispatchResult {
			let mut minted_avatars: Vec<AvatarIdOf<T>> = Vec::with_capacity(0);
			let mut forged_avatars: Vec<(AvatarIdOf<T>, UpgradedComponents)> =
				Vec::with_capacity(0);

			for output in other_outputs {
				match output {
					ForgeOutput::Forged((avatar_id, avatar), upgraded_components) => {
						Avatars::<T>::insert(avatar_id, (player, avatar));
						forged_avatars.push((avatar_id, upgraded_components));
					},
					ForgeOutput::Minted(avatar) => {
						let avatar_id = Self::random_hash(b"create_avatar", player);
						Self::try_add_avatar_to(player, season_id, avatar_id, avatar)?;
						minted_avatars.push(avatar_id);
					},
					ForgeOutput::Consumed(avatar_id) =>
						Self::remove_avatar_from(player, season_id, &avatar_id),
					ForgeOutput::Unchanged(_) => {},
				}
			}

			// TODO: May be removed in the future
			if !minted_avatars.is_empty() {
				Self::deposit_event(Event::AvatarsMinted { avatar_ids: minted_avatars });
			}

			// TODO: May change in the future
			if !forged_avatars.is_empty() {
				Self::deposit_event(Event::AvatarsForged { avatar_ids: forged_avatars });
			}

			Ok(())
		}

		fn update_forging_statistics_for_player(
			player: &AccountIdFor<T>,
			season_id: SeasonId,
		) -> DispatchResult {
			let current_block = <frame_system::Pallet<T>>::block_number();

			PlayerSeasonConfigs::<T>::try_mutate(
				player,
				season_id,
				|PlayerSeasonConfig { stats, .. }| -> DispatchResult {
					if stats.forge.first.is_zero() {
						stats.forge.first = current_block;
					}
					stats.forge.last = current_block;
					Ok(())
				},
			)?;

			SeasonStats::<T>::mutate(season_id, player, |info| {
				info.forged.saturating_inc();
			});

			Ok(())
		}

		fn try_add_avatar_to(
			player: &AccountIdFor<T>,
			season_id: &SeasonId,
			avatar_id: AvatarIdOf<T>,
			avatar: AvatarOf<T>,
		) -> DispatchResult {
			Avatars::<T>::insert(avatar_id, (player, avatar));
			Owners::<T>::try_append(&player, &season_id, avatar_id)
				.map_err(|_| Error::<T>::MaxOwnershipReached)?;
			Ok(())
		}

		fn remove_avatar_from(
			player: &AccountIdFor<T>,
			season_id: &SeasonId,
			avatar_id: &AvatarIdOf<T>,
		) {
			Avatars::<T>::remove(avatar_id);
			Owners::<T>::mutate(player, season_id, |avatars| {
				avatars.retain(|id| id != avatar_id);
			});
		}

		pub(crate) fn try_remove_avatar_ownership_from(
			player: &AccountIdFor<T>,
			season_id: &SeasonId,
			avatar_id: &AvatarIdOf<T>,
		) -> DispatchResult {
			Owners::<T>::mutate(player, season_id, |avatars| {
				avatars.retain(|id| id != avatar_id);
			});

			Avatars::<T>::try_mutate(avatar_id, |maybe_avatar| -> DispatchResult {
				let (from_owner, _) = maybe_avatar.as_mut().ok_or(Error::<T>::UnknownAvatar)?;
				*from_owner = Self::technical_account_id();
				Ok(())
			})
		}

		pub(crate) fn try_restore_avatar_ownership_to(
			player: &AccountIdFor<T>,
			season_id: &SeasonId,
			avatar_id: &AvatarIdOf<T>,
		) -> DispatchResult {
			Owners::<T>::try_mutate(player, season_id, |avatars| {
				avatars.try_push(*avatar_id).map_err(|_| Error::<T>::MaxOwnershipReached)?;
				ensure!(
					avatars.len() <=
						PlayerSeasonConfigs::<T>::get(player, season_id).storage_tier as usize,
					Error::<T>::MaxOwnershipReached
				);
				Ok::<_, DispatchError>(())
			})?;

			Avatars::<T>::try_mutate(avatar_id, |maybe_avatar| -> DispatchResult {
				let (from_owner, _) = maybe_avatar.as_mut().ok_or(Error::<T>::UnknownAvatar)?;
				*from_owner = player.clone();
				Ok(())
			})
		}

		pub(crate) fn ensure_for_trade(
			avatar_id: &AvatarIdOf<T>,
		) -> Result<(T::AccountId, BalanceOf<T>), DispatchError> {
			let (seller, avatar) = Self::avatars(avatar_id)?;
			let price = Trade::<T>::get(avatar.season_id, avatar_id)
				.ok_or(Error::<T>::UnknownAvatarForSale)?;
			Ok((seller, price))
		}

		fn ensure_unlocked(avatar_id: &AvatarIdOf<T>) -> DispatchResult {
			ensure!(!LockedAvatars::<T>::contains_key(avatar_id), Error::<T>::AvatarLocked);
			Ok(())
		}

		fn ensure_tradable(avatar: &AvatarOf<T>) -> DispatchResult {
			let trade_filters = SeasonTradeFilters::<T>::get(avatar.season_id)
				.ok_or::<DispatchError>(Error::<T>::UnknownSeason.into())?;
			ensure!(trade_filters.is_tradable(avatar), Error::<T>::AvatarCannotBeTraded);
			Ok(())
		}

		fn start_season(
			weight: &mut Weight,
			block_number: BlockNumberFor<T>,
			season_id: SeasonId,
			season: &SeasonScheduleOf<T>,
		) {
			let is_current_season_active = CurrentSeasonStatus::<T>::get().active;
			weight.saturating_accrue(T::DbWeight::get().reads(1));

			if !is_current_season_active {
				CurrentSeasonStatus::<T>::mutate(|status| {
					status.early = season.is_early(block_number);
					status.active = season.is_active(block_number);

					if season.is_active(block_number) {
						Self::deposit_event(Event::SeasonStarted(season_id));
					}
				});

				weight.saturating_accrue(T::DbWeight::get().writes(1));
			}
		}

		fn finish_season(
			weight: &mut Weight,
			block_number: BlockNumberFor<T>,
			season_id: SeasonId,
		) {
			let next_season_id = season_id.saturating_add(1);

			CurrentSeasonStatus::<T>::mutate(|status| {
				status.season_id = next_season_id;
				status.early = false;
				status.active = false;
				status.early_ended = false;
				status.max_tier_avatars = Zero::zero();
			});
			Self::deposit_event(Event::SeasonFinished(season_id));
			weight.saturating_accrue(T::DbWeight::get().writes(1));

			if let Some(next_season) = SeasonSchedules::<T>::get(next_season_id) {
				Self::start_season(weight, block_number, next_season_id, &next_season);
			}
		}

		pub(crate) fn avatars(
			avatar_id: &AvatarIdOf<T>,
		) -> Result<(T::AccountId, AvatarOf<T>), DispatchError> {
			let (owner, avatar) = Avatars::<T>::get(avatar_id).ok_or(Error::<T>::UnknownAvatar)?;
			Ok((owner, avatar))
		}

		pub(crate) fn seasons(season_id: &SeasonId) -> Result<SeasonOf<T>, DispatchError> {
			let season = Seasons::<T>::get(season_id).ok_or(Error::<T>::UnknownSeason)?;
			Ok(season)
		}

		fn season_schedules(season_id: &SeasonId) -> Result<SeasonScheduleOf<T>, DispatchError> {
			let season_schedule =
				SeasonSchedules::<T>::get(season_id).ok_or(Error::<T>::UnknownSeason)?;
			Ok(season_schedule)
		}

		fn try_propagate_tournament_fee(
			season_id: &SeasonId,
			account: &T::AccountId,
			percentage: Percentage,
			base_fee: BalanceOf<T>,
		) -> Result<BalanceOf<T>, DispatchError> {
			let tournament_fee = base_fee
				.saturating_mul(percentage.into())
				.checked_div(&100_u32.into())
				.unwrap_or_default();

			if tournament_fee > 0_u32.into() {
				let tournament_account = T::TournamentHandler::get_treasury_account_for(season_id);

				T::Currency::transfer(account, &tournament_account, tournament_fee, AllowDeath)?;
				Ok(base_fee.saturating_sub(tournament_fee))
			} else {
				Ok(base_fee)
			}
		}

		fn try_propagate_chain_fee(
			rule_id: AffiliateMethods,
			account: &T::AccountId,
			base_fee: BalanceOf<T>,
		) -> Result<BalanceOf<T>, DispatchError> {
			let final_fee = if let Some(chain) =
				T::AffiliateHandler::get_affiliator_chain_for(account)
			{
				T::AffiliateHandler::try_execute_rule_for(rule_id, |rule| {
					let mut final_fee = base_fee;
					for (rule_perc, chain_acc) in rule.into_iter().zip(chain.clone()) {
						let transfer_fee = base_fee
							.saturating_mul(rule_perc.into())
							.checked_div(&100_u32.into())
							.unwrap_or_default();

						if transfer_fee > 0_u32.into() {
							T::Currency::transfer(account, &chain_acc, transfer_fee, AllowDeath)?;
							final_fee = final_fee.saturating_sub(transfer_fee);
						}
					}

					Ok(final_fee)
				})?
			} else {
				base_fee
			};

			Ok(final_fee)
		}

		fn evaluate_unlock_state(
			config: &BoundedVec<u8, ConstU32<5>>,
			account_stats: &SeasonInfo,
		) -> bool {
			let minted = u8::try_from(account_stats.minted).unwrap_or(u8::MAX);
			let free_minted = u8::try_from(account_stats.free_minted).unwrap_or(u8::MAX);
			let forged = u8::try_from(account_stats.forged).unwrap_or(u8::MAX);
			let bought = u8::try_from(account_stats.bought).unwrap_or(u8::MAX);
			let sold = u8::try_from(account_stats.sold).unwrap_or(u8::MAX);

			config[0] <= minted &&
				config[1] <= free_minted &&
				config[2] <= forged &&
				config[3] <= bought &&
				config[4] <= sold
		}
	}
}
