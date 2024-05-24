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

pub use pallet::*;

pub mod consts;
#[cfg(test)]
mod mock;
pub(crate) mod random;
pub(crate) mod simulator;
#[cfg(test)]
mod tests;
pub mod types;

use frame_support::{
	pallet_prelude::*,
	traits::{Currency, Randomness},
};
use frame_system::{ensure_root, ensure_signed, pallet_prelude::*};
use sp_runtime::traits::{Hash, TrailingZeroInput};

use crate::types::*;
use crate::consts::*;
use crate::simulator::*;

pub const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	pub(crate) type AccountIdFor<T> = <T as frame_system::Config>::AccountId;

	pub(crate) type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdFor<T>>>::Balance;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: IsType<<Self as frame_system::Config>::RuntimeEvent> + From<Event<Self>>;

		type Currency: Currency<Self::AccountId>;

		type Randomness: Randomness<Self::Hash, BlockNumberFor<Self>>;
	}

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	pub type Organizer<T: Config> = StorageValue<_, AccountIdFor<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn players)]
	pub type Players<T: Config> = StorageMap<_, Identity, AccountIdFor<T>, PlayerData, ValueQuery>;

	#[pallet::storage]
	pub type NextShipIds<T: Config> = StorageMap<_, Identity, AccountIdFor<T>, ShipId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn ships)]
	pub type Ships<T: Config> =
		StorageDoubleMap<_, Identity, AccountIdFor<T>, Identity, ShipId, Ship, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn commanders)]
	pub type Commanders<T: Config> = StorageDoubleMap<
		_,
		Identity,
		AccountIdFor<T>,
		Identity,
		CommanderId,
		CommanderData,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn defense_fleets)]
	pub type DefenseFleets<T: Config> =
		StorageMap<_, Identity, AccountIdFor<T>, Fleet, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn offense_fleets)]
	pub type OffenseFleets<T: Config> =
		StorageMap<_, Identity, AccountIdFor<T>, Fleet, OptionQuery>;

	#[pallet::storage]
	pub type NextEngagementId<T: Config> =
		StorageMap<_, Identity, AccountIdFor<T>, EngagementId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn engagement_history)]
	pub type EngagementHistory<T: Config> = StorageDoubleMap<
		_,
		Identity,
		AccountIdFor<T>,
		Identity,
		EngagementId,
		EngagementResult,
		OptionQuery,
	>;

	#[pallet::storage]
	pub type NextLogId<T: Config> =
		StorageMap<_, Identity, AccountIdFor<T>, EngagementId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn engagement_logs)]
	pub type EngagementLogs<T: Config> =
		StorageDoubleMap<_, Identity, AccountIdFor<T>, Identity, LogId, EngagementLog, OptionQuery>;

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
		fn build(&self) {}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// An organizer has been set.
		OrganizerSet { organizer: AccountIdFor<T> },
		/// The account has rolled a commander
		CommanderRolled { account: AccountIdFor<T>, commander_id: CommanderId },
		/// A commander has received experience.
		CommanderReceivedExp { account: AccountIdFor<T>, commander: CommanderId, current_exp: u32 },
		/// The account has registered a new ship in their hangar
		ShipRegistered { account: AccountIdFor<T>, ship_id: ShipId, ship: Ship },
		/// The account has registered the fleet used for defense in engagements
		DefenseFleetRegistered { by: AccountIdFor<T> },
		/// The account has registered the fleet used for offensives in engagements
		OffenseFleetRegistered { by: AccountIdFor<T> },
		/// An engagement between two players finished
		EngagementFinished {
			attacker: AccountIdFor<T>,
			target: AccountIdFor<T>,
			result: EngagementResult,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		/// There is no account set as the organizer
		OrganizerNotSet,
		/// The account is not a registered player
		AccountIsNotPlayer,
		/// The player could not be found
		PlayerNotFound,
		/// The requested commander could not be found
		/// associated with the given player
		CommanderNotFound,
		/// The requested ship could not be found
		/// registered with the given player
		ShipNotFound,
		/// The account trying to engage doesn't have
		/// an offensive fleet registered
		OffensiveFleetNotRegistered,
		/// The target account or the engagement
		/// doesn't have a defensive fleet registered
		DefensiveFleetNotRegistered,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Set game organizer.
		///
		/// The organizer account is like an admin, allowed to perform certain operations
		/// related with the game configuration.
		///
		/// It can only be set by a root account.
		///
		/// Emits `OrganizerSet` event when successful.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(0)]
		#[pallet::weight({10_000})]
		pub fn set_organizer(origin: OriginFor<T>, organizer: T::AccountId) -> DispatchResult {
			ensure_root(origin)?;
			Organizer::<T>::put(&organizer);
			Self::deposit_event(Event::OrganizerSet { organizer });
			Ok(())
		}

		/// Buy a loot crate.
		///
		/// Tries to buy a loot crate, which may contain a new commander.
		///
		/// It can only be set by a player account.
		///
		/// Emits `CommanderRolled` event when successful.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(1)]
		#[pallet::weight({10_000})]
		pub fn buy_loot_crate(origin: OriginFor<T>) -> DispatchResult {
			let account = ensure_signed(origin)?;

			Self::try_buy_loot_crate_for(&account)
		}

		/// Adds ship to the hangar making it usable in both defense and offense fleets
		#[pallet::call_index(2)]
		#[pallet::weight({10_000})]
		pub fn add_ship(origin: OriginFor<T>, ship: Ship) -> DispatchResult {
			let account = ensure_signed(origin)?;

			let next_ship_id = NextShipIds::<T>::mutate(&account, |ship_id| {
				let assigned_id = *ship_id;
				*ship_id = ship_id.saturating_add(1);
				assigned_id
			});

			// Just used as a placeholder for now
			T::Currency::slash(&account, BalanceOf::<T>::from(10_u8));

			Ships::<T>::insert(&account, next_ship_id, ship);

			Self::deposit_event(Event::<T>::ShipRegistered {
				account,
				ship_id: next_ship_id,
				ship,
			});

			Ok(())
		}

		/// Registers a fleet to be used for ranked battles as the defense fleet
		#[pallet::call_index(3)]
		#[pallet::weight({10_000})]
		pub fn register_defense(origin: OriginFor<T>, fleet: Fleet) -> DispatchResult {
			let account = ensure_signed(origin)?;
			Self::is_valid_fleet_for(&account, &fleet)?;

			DefenseFleets::<T>::insert(&account, fleet);

			Self::deposit_event(Event::<T>::DefenseFleetRegistered { by: account });

			Ok(())
		}

		/// Registers a fleet to be used for ranked battles as the attacker fleet
		#[pallet::call_index(4)]
		#[pallet::weight({10_000})]
		pub fn register_offense(origin: OriginFor<T>, fleet: Fleet) -> DispatchResult {
			let account = ensure_signed(origin)?;
			Self::is_valid_fleet_for(&account, &fleet)?;

			OffenseFleets::<T>::insert(&account, fleet);

			Self::deposit_event(Event::<T>::OffenseFleetRegistered { by: account });

			Ok(())
		}

		/// Tries to initiate a battle with the 'target' player.
		///
		/// Will use the registered offense fleet for attack, will fail if no fleet has been
		/// registered for attack.
		#[pallet::call_index(5)]
		#[pallet::weight({10_000})]
		pub fn engage_player(origin: OriginFor<T>, target: AccountIdFor<T>) -> DispatchResult {
			let attacker = ensure_signed(origin)?;
			let offensive_fleet = {
				let maybe_offensive_fleet = OffenseFleets::<T>::get(&attacker);
				ensure!(maybe_offensive_fleet.is_some(), Error::<T>::OffensiveFleetNotRegistered);
				maybe_offensive_fleet.unwrap()
			};
			let offensive_commander = offensive_fleet.commander;

			let defensive_fleet = {
				let maybe_defensive_fleet = DefenseFleets::<T>::get(&target);
				ensure!(maybe_defensive_fleet.is_some(), Error::<T>::DefensiveFleetNotRegistered);
				maybe_defensive_fleet.unwrap()
			};
			let defensive_commander = defensive_fleet.commander;

			let seed = {
				let hash = Self::random_hash(b"engagement_roll", &attacker);
				let mut roller = random::DiceRoller::<T, 32, { u8::MAX }>::new(&hash);
				roller.next_seed()
			};

			let (engagement_result, attacker_moves, defender_moves) =
				BattleSimulator::<T>::simulate_engagement(
					&attacker,
					offensive_fleet,
					&target,
					defensive_fleet,
					seed,
					true,
				)?;

			// Mark results of the fight on the leaderboard and adjust commander xp
			if engagement_result.attacker_defeated {
				Self::mark_ranked_win_for(&target)?;
				Self::mark_ranked_loss_for(&attacker)?;
				Self::try_add_exp_to_commander_for(
					&target,
					defensive_commander,
					consts::XP_PER_RANKED_WIN,
				)?;
			} else if engagement_result.defender_defeated {
				Self::mark_ranked_win_for(&attacker)?;
				Self::mark_ranked_loss_for(&target)?;
				Self::try_add_exp_to_commander_for(
					&attacker,
					offensive_commander,
					consts::XP_PER_RANKED_WIN,
				)?;
			}

			Self::insert_engagement_in_players_history(&attacker, engagement_result.clone());

			if let Some(moves) = attacker_moves {
				Self::try_insert_engagement_logs(&attacker, moves)?;
			}

			if let Some(moves) = defender_moves {
				Self::try_insert_engagement_logs(&target, moves)?;
			}

			Self::deposit_event(Event::<T>::EngagementFinished {
				attacker,
				target,
				result: engagement_result,
			});

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		pub(crate) fn random_hash(phrase: &[u8], who: &T::AccountId) -> T::Hash {
			let (seed, _) = T::Randomness::random(phrase);
			let seed = T::Hash::decode(&mut TrailingZeroInput::new(seed.as_ref()))
				.expect("input is padded with zeroes; qed");
			let nonce = frame_system::Pallet::<T>::account_nonce(who);
			frame_system::Pallet::<T>::inc_account_nonce(who);
			(seed, &who, nonce.encode()).using_encoded(T::Hashing::hash)
		}

		/// Check that the origin is an organizer account.
		// For now we don't use it for anything apart from tests
		#[cfg(test)]
		pub(crate) fn ensure_organizer(
			origin: OriginFor<T>,
		) -> Result<T::AccountId, DispatchError> {
			let maybe_organizer = ensure_signed(origin)?;
			let existing_organizer = Organizer::<T>::get().ok_or(Error::<T>::OrganizerNotSet)?;
			ensure!(maybe_organizer == existing_organizer, DispatchError::BadOrigin);
			Ok(maybe_organizer)
		}

		pub(crate) fn try_add_commander_to(
			account: &AccountIdFor<T>,
			commander_id: CommanderId,
		) -> DispatchResult {
			Commanders::<T>::try_mutate(account, commander_id, |data| {
				if data.is_some() {
					// Adjust the commander xp
					Self::try_add_exp_to_commander_for(account, commander_id, XP_PER_LOOT_CRATE)
				} else {
					*data = Some(CommanderData { exp: XP_PER_LOOT_CRATE });
					Ok(())
				}
			})
		}

		fn is_valid_fleet_for(account: &AccountIdFor<T>, fleet: &Fleet) -> DispatchResult {
			ensure!(
				Commanders::<T>::contains_key(account, fleet.commander),
				Error::<T>::CommanderNotFound
			);

			ensure!(
				fleet
					.composition
					.iter()
					.all(|FleetWing { ship_id, .. }| Ships::<T>::contains_key(account, ship_id)),
				Error::<T>::ShipNotFound
			);

			Ok(())
		}

		fn try_add_exp_to_commander_for(
			account: &AccountIdFor<T>,
			commander_id: CommanderId,
			exp: u32,
		) -> DispatchResult {
			Commanders::<T>::try_mutate(account, commander_id, |maybe_data| {
				if let Some(ref mut data) = maybe_data {
					data.exp = data.exp.saturating_add(exp);

					Self::deposit_event(Event::CommanderReceivedExp {
						account: account.clone(),
						commander: commander_id,
						current_exp: data.exp,
					});

					Ok(())
				} else {
					Err(Error::<T>::CommanderNotFound.into())
				}
			})
		}

		fn mark_ranked_win_for(account: &AccountIdFor<T>) -> DispatchResult {
			Players::<T>::mutate(account, |data| {
				data.ranked_wins = data.ranked_wins.saturating_add(1);

				Ok(())
			})
		}

		fn mark_ranked_loss_for(account: &AccountIdFor<T>) -> DispatchResult {
			Players::<T>::mutate(account, |data| {
				data.ranked_losses = data.ranked_losses.saturating_add(1);

				Ok(())
			})
		}

		fn insert_engagement_in_players_history(
			account: &AccountIdFor<T>,
			engagement: EngagementResult,
		) {
			let next_engagement_id = NextEngagementId::<T>::mutate(account, |engagement_id| {
				let assigned_id = *engagement_id;
				*engagement_id = engagement_id.saturating_add(1);
				assigned_id
			});

			EngagementHistory::<T>::insert(account, next_engagement_id, engagement);
		}

		fn try_insert_engagement_logs(account: &AccountIdFor<T>, log: Vec<Move>) -> DispatchResult {
			let engagement_log =
				EngagementLog::try_from(log).map_err(|_| Error::<T>::ShipNotFound)?;

			let next_log_id = NextLogId::<T>::mutate(account, |log_id| {
				let assigned_id = *log_id;
				*log_id = log_id.saturating_add(1);
				assigned_id
			});

			EngagementLogs::<T>::insert(account, next_log_id, engagement_log);

			Ok(())
		}

		fn try_buy_loot_crate_for(account: &AccountIdFor<T>) -> DispatchResult {
			let mut rolled_commander: u8 = 0;

			let hash = Self::random_hash(b"commander_roll", account);
			let mut roller = random::DiceRoller::<T, 32, 101>::new(&hash);

			let roll = roller.next();
			// Probability to get the lowest commander
			let mut prob: u8 = 75;

			for i in 0..consts::MAX_COMMANDERS as u8 {
				if roll < prob {
					rolled_commander = i;
					break;
				}

				// Define new probability window for next commander
				prob += (100 - prob) / 2;
			}

			Self::try_add_commander_to(account, rolled_commander)?;

			Self::deposit_event(Event::<T>::CommanderRolled {
				account: account.clone(),
				commander_id: rolled_commander,
			});

			Ok(())
		}
	}
}
