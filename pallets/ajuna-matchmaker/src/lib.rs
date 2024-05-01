#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>
pub use pallet::*;

use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_std::{boxed::Box, vec::Vec};

use frame_support::traits::Get;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

mod brackets;

use brackets::{Bracket, BracketsTrait, BracketsTransient, BufferIndex};

#[derive(Encode, Decode, Clone, PartialEq, TypeInfo)]
pub enum MatchingType {
	// ranked matches, if no one in bracket drop down
	Simple,
	// only allow same bracket matches
	Same,
	// take only one of one bracket
	Mix,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
pub struct PlayerStruct<AccountId> {
	account: AccountId,
}

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;

	// important to use outside structs and consts
	use super::*;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Constant that indicates how many players are needed to create a new match.
		#[pallet::constant]
		type AmountPlayers: Get<u8>;

		/// Constant that indicates how many ranking brackets exist for players.
		#[pallet::constant]
		type AmountBrackets: Get<u8>;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::type_value]
	pub fn BracketsCountDefault<T: Config>() -> u8 {
		T::AmountBrackets::get()
	}

	#[pallet::storage]
	#[pallet::getter(fn brackets_count)]
	pub type BracketsCount<T: Config> = StorageValue<_, u8, ValueQuery, BracketsCountDefault<T>>;

	// Default value for Nonce
	#[pallet::type_value]
	pub fn BracketIndicesDefault<T: Config>() -> (BufferIndex, BufferIndex) {
		(0, 0)
	}

	#[pallet::storage]
	#[pallet::getter(fn indices)]
	pub type BracketIndices<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		Bracket,
		(BufferIndex, BufferIndex),
		ValueQuery,
		BracketIndicesDefault<T>,
	>;

	#[pallet::storage]
	#[pallet::getter(fn index_key)]
	pub type BracketIndexKeyMap<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		Bracket,
		Blake2_128Concat,
		BufferIndex,
		T::AccountId,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn key_value)]
	pub type BracketKeyValueMap<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		Bracket,
		Blake2_128Concat,
		T::AccountId,
		PlayerStruct<T::AccountId>,
		OptionQuery,
	>;

	// Pallets use events to inform users when important changes are made.
	// https://substrate.dev/docs/en/knowledgebase/runtime/events
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Queued event
		Queued(PlayerStruct<T::AccountId>),
		/// Popped event
		Popped(PlayerStruct<T::AccountId>),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Player has already queued, can not queue twice
		AlreadyQueued,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {}
}

impl<T: Config> Pallet<T> {
	/// Constructor function so we don't have to specify the types every time.
	///
	/// Constructs a ringbuffer transient and returns it as a boxed trait object.
	/// See [this part of the Rust book](https://doc.rust-lang.org/book/ch17-02-trait-objects.html#trait-objects-perform-dynamic-dispatch)
	fn queue_transient() -> Box<dyn BracketsTrait<T::AccountId, PlayerStruct<T::AccountId>>> {
		Box::new(BracketsTransient::<
			T::AccountId,
			PlayerStruct<T::AccountId>,
			BracketsCount<T>,
			BracketIndices<T>,
			BracketIndexKeyMap<T>,
			BracketKeyValueMap<T>,
		>::new())
	}

	fn do_add_queue(account: T::AccountId, bracket: u8) -> Result<(), sp_runtime::DispatchError> {
		let mut queue = Self::queue_transient();

		let player = PlayerStruct { account };
		// duplicate check if we can add key to the queue
		if !queue.push(bracket, player.account.clone(), player.clone()) {
			return Err(Error::<T>::AlreadyQueued.into());
		}

		Self::deposit_event(Event::Queued(player));

		Ok(())
	}

	fn do_empty_queue(bracket: u8) {
		let mut queue = Self::queue_transient();

		while queue.size(bracket) > 0 {
			queue.pop(bracket);
		}
	}

	fn do_all_empty_queue() {
		for i in 0..Self::brackets_count() {
			Self::do_empty_queue(i);
		}
	}

	fn do_try_match() -> Vec<T::AccountId> {
		let mut queue = Self::queue_transient();
		let max_players = T::AmountPlayers::get();

		let mut result: Vec<T::AccountId> = Vec::new();
		let mut brackets: Vec<Bracket> = Vec::new();
		// pass trough all brackets
		for i in 0..Self::brackets_count() {
			// skip if bracket is empty
			if queue.size(i) == 0 {
				continue;
			}
			// iterate for each slot occupied and fill, till player match size reached
			for _j in 0..queue.size(i) {
				if brackets.len() == max_players as usize {
					break;
				}
				brackets.push(i);
			}
			// leave if brackets is filled with brackets
			if brackets.len() == max_players as usize {
				break;
			}
		}
		// vec not filled with enough brackets leave
		if brackets.len() < max_players as usize {
			return result;
		}

		// pop from the harvested brackets players
		for bracket in brackets {
			if let Some(p) = queue.pop(bracket) {
				result.push(p.account.clone());
				Self::deposit_event(Event::Popped(p));
			}
		}
		// return result
		result
	}

	fn do_is_queued(account: T::AccountId) -> bool {
		Self::queue_transient().is_queued(account)
	}

	fn do_queue_size(bracket: u8) -> BufferIndex {
		Self::queue_transient().size(bracket)
	}

	fn do_all_queue_size() -> BufferIndex {
		let queue = Self::queue_transient();

		let mut total_queued: BufferIndex = 0;
		// count all existing brackets
		for i in 0..Self::brackets_count() {
			total_queued = total_queued.saturating_add(queue.size(i));
		}
		// return result
		total_queued
	}
}

impl<T: Config> MatchFunc<T::AccountId> for Pallet<T> {
	fn empty_queue(bracket: u8) {
		Self::do_empty_queue(bracket);
	}

	fn all_empty_queue() {
		Self::do_all_empty_queue();
	}

	fn add_queue(account: T::AccountId, bracket: u8) -> Result<(), sp_runtime::DispatchError> {
		Self::do_add_queue(account, bracket)
	}

	fn try_match() -> Vec<T::AccountId> {
		Self::do_try_match()
	}

	fn is_queued(account: T::AccountId) -> bool {
		Self::do_is_queued(account)
	}

	fn queue_size(bracket: u8) -> BufferIndex {
		Self::do_queue_size(bracket)
	}

	fn all_queue_size() -> BufferIndex {
		Self::do_all_queue_size()
	}
}

pub trait MatchFunc<AccountId> {
	/// empty specific bracket queue
	fn empty_queue(bracket: u8);

	/// empty all queues
	fn all_empty_queue();

	/// return true if adding account to bracket queue was successful
	fn add_queue(account: AccountId, bracket: u8) -> Result<(), sp_runtime::DispatchError>;

	/// try create a match
	fn try_match() -> Vec<AccountId>;

	// return true if an account is queued in any bracket
	fn is_queued(account: AccountId) -> bool;

	// return size of a specific bracket queue
	fn queue_size(bracket: u8) -> BufferIndex;

	// return total size of all queued accounts in all brackets
	fn all_queue_size() -> BufferIndex;
}
