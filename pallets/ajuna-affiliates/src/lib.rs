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

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;
pub mod impls;
pub mod traits;
pub mod weights;

use crate::weights::WeightInfo;

use ajuna_primitives::account_manager::{AccountManager, WhitelistKey};

use frame_support::{pallet_prelude::*, traits::Currency};
use frame_system::{ensure_signed, pallet_prelude::*};
use sp_runtime::ArithmeticError;
use sp_std::vec::Vec;

use traits::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	pub(crate) type BalanceOf<T, I> =
		<<T as Config<I>>::Currency as Currency<AccountIdFor<T>>>::Balance;

	pub type AffiliatedAccountsOf<T, I> =
		BoundedVec<<T as frame_system::Config>::AccountId, <T as Config<I>>::AffiliateMaxLevel>;

	pub type AccountIdFor<T> = <T as frame_system::Config>::AccountId;
	pub type RuleIdentifierFor<T, I> = <T as Config<I>>::RuleIdentifier;
	pub type AffiliateMaxLevelFor<T, I> = <T as Config<I>>::AffiliateMaxLevel;
	pub type FeePropagationOf<T, I> = FeePropagation<<T as Config<I>>::AffiliateMaxLevel>;

	#[cfg(feature = "runtime-benchmarks")]
	pub trait BenchmarkHelper<RuleIdParameter, UnlockParams> {
		fn create_rule_id(id: u32) -> RuleIdParameter;

		fn create_params(id: u32) -> UnlockParams;
	}
	#[cfg(feature = "runtime-benchmarks")]
	impl<RuleIdParameter: From<u32>, UnlockParams: From<u32>>
		BenchmarkHelper<RuleIdParameter, UnlockParams> for ()
	{
		fn create_rule_id(id: u32) -> RuleIdParameter {
			id.into()
		}

		fn create_params(id: u32) -> UnlockParams {
			id.into()
		}
	}

	/// The current storage version.
	const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T, I = ()>(PhantomData<(T, I)>);

	#[pallet::config]
	pub trait Config<I: 'static = ()>: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self, I>>
			+ IsType<<Self as frame_system::Config>::RuntimeEvent>;

		type Currency: Currency<Self::AccountId>;

		#[pallet::constant]
		type WhitelistKey: Get<WhitelistKey>;

		type AccountManager: AccountManager<AccountId = AccountIdFor<Self>>;

		/// The rule identifier type at runtime.
		type RuleIdentifier: Parameter + MaxEncodedLen;

		/// The maximum depth of the affiliate relation chain,
		#[pallet::constant]
		type AffiliateMaxLevel: Get<u32>;

		type UnlockParameters: Parameter;

		type AffiliatesUnlockRules: AffiliateUnlockRules<
			AccountId = AccountIdFor<Self>,
			UnlockParameters = Self::UnlockParameters,
		>;

		type WeightInfo: WeightInfo;

		#[cfg(feature = "runtime-benchmarks")]
		type BenchmarkHelper: BenchmarkHelper<Self::RuleIdentifier, Self::UnlockParameters>;
	}

	/// Stores the affiliated accounts from the perspectives of the affiliatee
	#[pallet::storage]
	#[pallet::getter(fn affiliatees)]
	pub type Affiliatees<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Identity, T::AccountId, AffiliatedAccountsOf<T, I>, OptionQuery>;

	/// Store affiliators aka accounts that have affilatees and earn rewards from them.
	/// Such accounts can't be affiliatees anymore.
	#[pallet::storage]
	#[pallet::getter(fn affiliators)]
	pub type Affiliators<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Identity, T::AccountId, AffiliatorState, ValueQuery>;

	/// Stores the affiliate logic rules
	#[pallet::storage]
	pub type AffiliateRules<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Blake2_128Concat, T::RuleIdentifier, FeePropagationOf<T, I>, OptionQuery>;

	#[pallet::storage]
	pub type NextAffiliateId<T: Config<I>, I: 'static = ()> =
		StorageValue<_, AffiliateId, ValueQuery>;

	#[pallet::storage]
	pub type AffiliateIdMapping<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Blake2_128Concat, AffiliateId, T::AccountId, OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config<I>, I: 'static = ()> {
		AccountMarkedAsAffiliatable { account: T::AccountId, affiliate_id: AffiliateId },
		AccountAffiliated { account: T::AccountId, to: T::AccountId },
		AccountUnaffiliated { account: T::AccountId },
		RuleAdded { rule_id: T::RuleIdentifier },
		RuleCleared { rule_id: T::RuleIdentifier },
	}

	#[pallet::error]
	pub enum Error<T, I = ()> {
		/// An account cannot affiliate itself
		CannotAffiliateSelf,
		/// The account is not allowed to receive affiliates
		TargetAccountIsNotAffiliatable,
		/// Only whitelisted accounts can affiliate for others
		AffiliateOthersOnlyWhiteListed,
		/// No account matches the provided affiliator identifier
		AffiliatorNotFound,
		/// This account has reached the affiliate limit
		CannotAffiliateMoreAccounts,
		/// This account has already been affiliated by another affiliator
		CannotAffiliateAlreadyAffiliatedAccount,
		/// This account is already an affiliator, so it cannot affiliate to another account
		CannotAffiliateToExistingAffiliator,
		/// The account is blocked, so it cannot be affiliated to
		CannotAffiliateBlocked,
		/// The given extrinsic identifier is already paired with an affiliate rule
		ExtrinsicAlreadyHasRule,
		/// The given extrinsic identifier is not associated with any rule
		ExtrinsicHasNoRule,
	}

	#[pallet::call]
	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		#[pallet::call_index(0)]
		#[pallet::weight({T::WeightInfo::enable_affiliator()})]
		pub fn enable_affiliator(
			origin: OriginFor<T>,
			target: Option<AccountIdFor<T>>,
			params: T::UnlockParameters,
		) -> DispatchResult {
			let account = ensure_signed(origin)?;

			T::AffiliatesUnlockRules::execute_unlock_rule_for(&account, params)?;

			let account_to_mark = if let Some(other) = target { other } else { account };

			Self::try_mark_account_as_affiliatable(&account_to_mark)?;

			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight({T::WeightInfo::add_affiliation()})]
		pub fn add_affiliation(
			origin: OriginFor<T>,
			target_affiliatee: Option<AccountIdFor<T>>,
			affiliate_id: AffiliateId,
		) -> DispatchResult {
			let signer = ensure_signed(origin)?;

			let account = if let Some(acc) = target_affiliatee {
				ensure!(
					T::AccountManager::is_whitelisted_for(&T::WhitelistKey::get(), &signer),
					Error::<T, I>::AffiliateOthersOnlyWhiteListed
				);
				acc
			} else {
				signer
			};

			if let Some(affiliator) = Self::get_account_for_id(affiliate_id) {
				Self::try_add_affiliate_to(&affiliator, &account)
			} else {
				Err(Error::<T, I>::AffiliatorNotFound.into())
			}
		}

		#[pallet::call_index(2)]
		#[pallet::weight({T::WeightInfo::remove_affiliation()})]
		pub fn remove_affiliation(origin: OriginFor<T>, account: T::AccountId) -> DispatchResult {
			let maybe_organizer = ensure_signed(origin)?;
			T::AccountManager::is_organizer(&maybe_organizer)?;
			Self::try_clear_affiliation_for(&account)
		}

		#[pallet::call_index(3)]
		#[pallet::weight({T::WeightInfo::set_rule_for()})]
		pub fn set_rule_for(
			origin: OriginFor<T>,
			rule_id: RuleIdentifierFor<T, I>,
			rule: FeePropagationOf<T, I>,
		) -> DispatchResult {
			let account = ensure_signed(origin)?;
			T::AccountManager::is_organizer(&account)?;

			Self::try_add_rule_for(rule_id, rule)
		}

		#[pallet::call_index(4)]
		#[pallet::weight({T::WeightInfo::clear_rule_for()})]
		pub fn clear_rule_for(
			origin: OriginFor<T>,
			rule_id: RuleIdentifierFor<T, I>,
		) -> DispatchResult {
			let account = ensure_signed(origin)?;
			T::AccountManager::is_organizer(&account)?;

			<Self as RuleMutator<_, _>>::clear_rule_for(rule_id);

			Ok(())
		}
	}

	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		fn add_new_affiliate_to(
			affiliator: T::AccountId,
			affiliatee: T::AccountId,
		) -> DispatchResult {
			let mut accounts = Affiliatees::<T, I>::get(&affiliator).unwrap_or_default();

			Self::try_add_account_to(&mut accounts, affiliator.clone())?;

			Affiliatees::<T, I>::insert(affiliatee, accounts);
			Affiliators::<T, I>::try_mutate(&affiliator, |state| {
				state.affiliates = state
					.affiliates
					.checked_add(1)
					.ok_or(DispatchError::Arithmetic(ArithmeticError::Overflow))?;

				Ok(())
			})
		}

		fn try_add_account_to(
			accounts: &mut AffiliatedAccountsOf<T, I>,
			account: T::AccountId,
		) -> DispatchResult {
			if accounts.len() == T::AffiliateMaxLevel::get() as usize {
				accounts.pop();
			}
			accounts
				.try_insert(0, account)
				.map_err(|_| Error::<T, I>::CannotAffiliateMoreAccounts.into())
		}
	}

	impl<T: Config<I>, I: 'static> AffiliateInspector<AccountIdFor<T>> for Pallet<T, I> {
		fn get_affiliator_chain_for(account: &AccountIdFor<T>) -> Option<Vec<AccountIdFor<T>>> {
			Affiliatees::<T, I>::get(account).map(|accounts| accounts.into_inner())
		}

		fn get_affiliate_count_for(account: &AccountIdFor<T>) -> u32 {
			Affiliators::<T, I>::get(account).affiliates
		}

		fn get_account_for_id(affiliate_id: AffiliateId) -> Option<AccountIdFor<T>> {
			AffiliateIdMapping::<T, I>::get(affiliate_id)
		}
	}

	impl<T: Config<I>, I: 'static> AffiliateMutator<AccountIdFor<T>> for Pallet<T, I> {
		fn try_mark_account_as_affiliatable(account: &AccountIdFor<T>) -> DispatchResult {
			Affiliators::<T, I>::try_mutate(account, |state| match state.status {
				AffiliatableStatus::NonAffiliatable => {
					let next_id =
						NextAffiliateId::<T, I>::try_mutate(|value: &mut AffiliateId| {
							let next_id = *value;

							*value = value
								.checked_add(1)
								.ok_or(DispatchError::Arithmetic(ArithmeticError::Overflow))?;

							Ok::<AffiliateId, DispatchError>(next_id)
						})?;

					AffiliateIdMapping::<T, I>::insert(next_id, account.clone());

					state.status = AffiliatableStatus::Affiliatable(next_id);

					Self::deposit_event(Event::AccountMarkedAsAffiliatable {
						account: account.clone(),
						affiliate_id: next_id,
					});

					Ok(())
				},
				AffiliatableStatus::Affiliatable(_) => Ok(()),
				AffiliatableStatus::Blocked => Err(Error::<T, I>::CannotAffiliateBlocked.into()),
			})
		}

		fn mark_account_as_blocked(account: &AccountIdFor<T>) {
			Affiliators::<T, I>::mutate(account, |state| {
				state.status = AffiliatableStatus::Blocked;
			});
		}

		fn try_add_affiliate_to(
			account: &AccountIdFor<T>,
			affiliate: &AccountIdFor<T>,
		) -> DispatchResult {
			ensure!(account != affiliate, Error::<T, I>::CannotAffiliateSelf);

			let affiliate_state = Affiliators::<T, I>::get(affiliate);
			ensure!(
				affiliate_state.affiliates == 0,
				Error::<T, I>::CannotAffiliateToExistingAffiliator
			);

			ensure!(
				!Affiliatees::<T, I>::contains_key(affiliate),
				Error::<T, I>::CannotAffiliateAlreadyAffiliatedAccount
			);

			let affiliator_state = Affiliators::<T, I>::get(account);
			ensure!(
				matches!(affiliator_state.status, AffiliatableStatus::Affiliatable(_)),
				Error::<T, I>::TargetAccountIsNotAffiliatable
			);

			Self::add_new_affiliate_to(account.clone(), affiliate.clone())?;

			Self::deposit_event(Event::AccountAffiliated {
				account: affiliate.clone(),
				to: account.clone(),
			});

			Ok(())
		}

		fn try_clear_affiliation_for(account: &AccountIdFor<T>) -> DispatchResult {
			let result = Affiliatees::<T, I>::take(account)
				.and_then(|mut affiliate_chain| {
					if affiliate_chain.is_empty() {
						None
					} else {
						Some(affiliate_chain.remove(0))
					}
				})
				.map_or_else(
					|| Ok(()),
					|affiliator| {
						Affiliators::<T, I>::try_mutate(&affiliator, |state| {
							state.affiliates = state
								.affiliates
								.checked_sub(1)
								.ok_or(DispatchError::Arithmetic(ArithmeticError::Underflow))?;

							Ok(())
						})
					},
				);

			Self::deposit_event(Event::AccountUnaffiliated { account: account.clone() });

			result
		}

		fn force_set_affiliatee_chain_for(
			account: &AccountIdFor<T>,
			chain: Vec<AccountIdFor<T>>,
		) -> DispatchResult {
			let chain = AffiliatedAccountsOf::<T, I>::try_from(chain)
				.map_err(|_| Error::<T, I>::CannotAffiliateMoreAccounts)?;
			Affiliatees::<T, I>::insert(account, chain);

			Ok(())
		}
	}

	impl<T: Config<I>, I: 'static> RuleInspector<T::RuleIdentifier, T::AffiliateMaxLevel>
		for Pallet<T, I>
	{
		fn get_rule_for(rule_id: &T::RuleIdentifier) -> Option<FeePropagationOf<T, I>> {
			AffiliateRules::<T, I>::get(rule_id)
		}
	}

	impl<T: Config<I>, I: 'static> RuleMutator<T::RuleIdentifier, T::AffiliateMaxLevel>
		for Pallet<T, I>
	{
		fn try_add_rule_for(
			rule_id: T::RuleIdentifier,
			rule: FeePropagationOf<T, I>,
		) -> DispatchResult {
			ensure!(
				!AffiliateRules::<T, I>::contains_key(rule_id.clone()),
				Error::<T, I>::ExtrinsicAlreadyHasRule
			);
			AffiliateRules::<T, I>::insert(rule_id.clone(), rule);
			Self::deposit_event(Event::RuleAdded { rule_id });

			Ok(())
		}

		fn clear_rule_for(rule_id: T::RuleIdentifier) {
			AffiliateRules::<T, I>::remove(rule_id.clone());

			Self::deposit_event(Event::RuleCleared { rule_id });
		}
	}

	impl<T: Config<I>, I: 'static> RuleExecutor<T::RuleIdentifier, T::AffiliateMaxLevel>
		for Pallet<T, I>
	{
		fn try_execute_rule_for<F, R>(
			rule_id: T::RuleIdentifier,
			rule_fn: F,
		) -> Result<R, DispatchError>
		where
			F: Fn(FeePropagationOf<T, I>) -> Result<R, DispatchError>,
		{
			if let Some(rule) = Self::get_rule_for(&rule_id) {
				rule_fn(rule)
			} else {
				Err(Error::<T, I>::ExtrinsicHasNoRule.into())
			}
		}
	}
}
