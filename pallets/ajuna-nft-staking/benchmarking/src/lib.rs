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

#![cfg(feature = "runtime-benchmarks")]
#![cfg_attr(not(feature = "std"), no_std)]

mod mock;

use frame_benchmarking::v2::*;
use frame_support::{
	pallet_prelude::*,
	traits::{
		Currency, Get,
		tokens::nonfungibles_v2::{Create, Mutate},
	},
};
use frame_system::{RawOrigin, pallet_prelude::BlockNumberFor};
use pallet_ajuna_nft_staking::{
	BenchmarkHelper as NftStakingBenchmarkHelper, Config as NftStakingConfig, *,
};
use pallet_nfts::{BenchmarkHelper, ItemConfig};
use sp_runtime::{
	DispatchError,
	traits::{BlockNumberProvider, One, UniqueSaturatedFrom, UniqueSaturatedInto},
};
use sp_std::{vec, vec::Vec};

// Creator's collections.
const CONTRACT_COLLECTION: u16 = 0;
const REWARD_COLLECTION: u16 = 1;
// Staker's collections.
const STAKE_COLLECTION: u16 = 2;
const FEE_COLLECTION: u16 = 3;
// Sniper's collections.
const SNIPER_STAKE_COLLECTION: u16 = 4;
const SNIPER_FEE_COLLECTION: u16 = 5;
// Unified attribute value for all contracts.
const ATTRIBUTE_VALUE: u8 = 10;

enum Mode {
	Staker,
	Sniper,
}
impl Mode {
	fn collections(self) -> (u16, u16) {
		match self {
			Self::Staker => (STAKE_COLLECTION, FEE_COLLECTION),
			Self::Sniper => (SNIPER_STAKE_COLLECTION, SNIPER_FEE_COLLECTION),
		}
	}
}

pub struct Pallet<T: Config>(pallet_ajuna_nft_staking::Pallet<T>);
pub trait Config: NftStakingConfig + pallet_nfts::Config + pallet_balances::Config {}

type AccountIdFor<T> = <T as frame_system::Config>::AccountId;
type CurrencyOf<T> = <T as NftStakingConfig>::Currency;
type BalanceOf<T> = <CurrencyOf<T> as Currency<AccountIdFor<T>>>::Balance;
type CollectionIdOf<T> = <T as NftStakingConfig>::CollectionId;
type ItemIdOf<T> = <T as NftStakingConfig>::ItemId;
type ContractOf<T> = Contract<
	BalanceOf<T>,
	CollectionIdOf<T>,
	<T as NftStakingConfig>::ItemId,
	BlockNumberFor<T>,
	<T as NftStakingConfig>::KeyLimit,
	<T as NftStakingConfig>::ValueLimit,
>;

type NftCurrencyOf<T> = <T as pallet_nfts::Config>::Currency;
type NftBalanceOf<T> = <NftCurrencyOf<T> as Currency<AccountIdFor<T>>>::Balance;
type NftCollectionIdOf<T> = <T as pallet_nfts::Config>::CollectionId;
type CollectionDeposit<T> = <T as pallet_nfts::Config>::CollectionDeposit;
type ItemDeposit<T> = <T as pallet_nfts::Config>::ItemDeposit;

type BlockNumberForNft<T> =
	<<T as pallet_nfts::Config>::BlockNumberProvider as BlockNumberProvider>::BlockNumber;
type CollectionConfigOf<T> =
	pallet_nfts::CollectionConfig<NftBalanceOf<T>, BlockNumberForNft<T>, NftCollectionIdOf<T>>;

fn account<T: Config>(name: &'static str) -> T::AccountId {
	let account = frame_benchmarking::account(name, Default::default(), Default::default());
	CurrencyOf::<T>::make_free_balance_be(&account, 999_999_999_u64.unique_saturated_into());
	account
}

fn assert_last_event<T: Config>(avatars_event: Event<T>) {
	let event = <T as frame_system::Config>::RuntimeEvent::from(avatars_event);
	frame_system::Pallet::<T>::assert_last_event(event);
}

fn create_creator<T: Config>(reward_item: Option<Vec<u16>>) -> Result<T::AccountId, DispatchError> {
	let creator = account::<T>("creator");
	create_contract_collection::<T>(&creator)?; // reserve CONTRACT_COLLECTION
	create_collections::<T>(&creator, 1)?; // reserve REWARD_COLLECTION
	if let Some(item_ids) = reward_item {
		item_ids
			.into_iter()
			.try_for_each(|item_id| mint_item::<T>(&creator, REWARD_COLLECTION, item_id))?;
	}
	Creator::<T>::put(&creator);
	Ok(creator)
}

fn create_contract_collection<T: Config>(creator: &T::AccountId) -> DispatchResult {
	create_collection::<T>(creator)?;
	ContractCollectionId::<T>::put(CollectionIdOf::<T>::from(CONTRACT_COLLECTION));
	Ok(())
}

fn create_collections<T: Config>(creator: &T::AccountId, n: usize) -> DispatchResult {
	(0..n).try_for_each(|_| create_collection::<T>(creator))
}

fn create_collection<T: Config>(owner: &T::AccountId) -> DispatchResult {
	let _ = NftCurrencyOf::<T>::deposit_creating(owner, CollectionDeposit::<T>::get());
	<pallet_nfts::Pallet<T> as Create<T::AccountId, CollectionConfigOf<T>>>::create_collection(
		owner,
		owner,
		&pallet_nfts::CollectionConfig {
			settings: Default::default(),
			max_supply: Default::default(),
			mint_settings: Default::default(),
		},
	)?;
	Ok(())
}

fn create_contract<T: Config>(
	creator: T::AccountId,
	contract_id: ItemIdOf<T>,
	contract: ContractOf<T>,
) -> DispatchResult {
	pallet_ajuna_nft_staking::Pallet::<T>::create(
		RawOrigin::Signed(creator).into(),
		contract_id,
		contract,
		None,
		None,
	)
}

fn accept_contract<T: Config>(
	num_stake_clauses: u32,
	num_fee_clauses: u32,
	staker: T::AccountId,
	contract_id: ItemIdOf<T>,
	mode: Mode,
) -> DispatchResult {
	let (stakes, fees) = stakes_and_fees::<T>(num_stake_clauses, num_fee_clauses, &staker, mode)?;
	pallet_ajuna_nft_staking::Pallet::<T>::accept(
		RawOrigin::Signed(staker).into(),
		contract_id,
		stakes,
		fees,
	)
}

fn mint_item<T: Config>(owner: &T::AccountId, collection_id: u16, item_id: u16) -> DispatchResult {
	let collection_id = &T::Helper::collection(collection_id);
	let item_id = &T::Helper::item(item_id);
	let _ = NftCurrencyOf::<T>::deposit_creating(owner, ItemDeposit::<T>::get());
	<pallet_nfts::Pallet<T> as Mutate<T::AccountId, ItemConfig>>::mint_into(
		collection_id,
		item_id,
		owner,
		&ItemConfig::default(),
		false,
	)?;
	Ok(())
}

fn set_attribute<T: Config>(
	collection_id: u16,
	item_id: u16,
	key: u8,
	value: u8,
) -> DispatchResult {
	let collection_id = &T::Helper::collection(collection_id);
	let item_id = &T::Helper::item(item_id);
	<pallet_nfts::Pallet<T> as Mutate<T::AccountId, ItemConfig>>::set_attribute(
		collection_id,
		item_id,
		&[key],
		&[value],
	)?;
	Ok(())
}

#[allow(clippy::type_complexity)]
fn stakes_and_fees<T: Config>(
	num_stake_clauses: u32,
	num_fee_clauses: u32,
	who: &T::AccountId,
	mode: Mode,
) -> Result<(Vec<NftIdOf<T>>, Vec<NftIdOf<T>>), DispatchError> {
	let (stake_collection, fee_collection) = mode.collections();
	let mut stakes = Vec::new();
	let mut fees = Vec::new();
	for i in 0..num_stake_clauses {
		let item_id = i as u16;
		let attr_key = i;
		mint_item::<T>(who, stake_collection, item_id)?;
		set_attribute::<T>(stake_collection, item_id, (attr_key as u8) * 3, ATTRIBUTE_VALUE)?;
		stakes.push(NftId(
			CollectionIdOf::<T>::unique_saturated_from(stake_collection),
			T::BenchmarkHelper::item_id(item_id),
		));
	}
	for i in num_stake_clauses..num_stake_clauses + num_fee_clauses {
		let item_id = i as u16;
		let attr_key = i;
		mint_item::<T>(who, fee_collection, item_id)?;
		set_attribute::<T>(fee_collection, item_id, (attr_key as u8) * 3, ATTRIBUTE_VALUE)?;
		fees.push(NftId(
			CollectionIdOf::<T>::unique_saturated_from(fee_collection),
			T::BenchmarkHelper::item_id(item_id),
		));
	}
	Ok((stakes, fees))
}

fn contract_with<T: Config>(
	num_stake_clauses: u32,
	num_fee_clauses: u32,
	rewards: BoundedRewardsOf<T>,
	mode: Mode,
) -> ContractOf<T> {
	let (stake_collection, fee_collection) = mode.collections();
	ContractOf::<T> {
		activation: None,
		active_duration: 1_u32.unique_saturated_into(),
		claim_duration: 1_u32.unique_saturated_into(),
		stake_duration: 1_u32.unique_saturated_into(),
		stake_clauses: (0..num_stake_clauses)
			.map(|i| ContractClause {
				namespace: AttributeNamespace::Pallet,
				target_index: i as u8,
				clause: Clause::HasAttributeWithValue(
					CollectionIdOf::<T>::unique_saturated_from(stake_collection),
					T::BenchmarkHelper::contract_key((i as u8) * 3),
					AttributeValue::Equal(T::BenchmarkHelper::contract_value(ATTRIBUTE_VALUE)),
				),
			})
			.collect::<Vec<_>>()
			.try_into()
			.unwrap(),
		fee_clauses: (num_stake_clauses..num_stake_clauses + num_fee_clauses)
			.map(|i| ContractClause {
				namespace: AttributeNamespace::Pallet,
				target_index: (i - num_stake_clauses) as u8,
				clause: Clause::HasAttributeWithValue(
					CollectionIdOf::<T>::unique_saturated_from(fee_collection),
					T::BenchmarkHelper::contract_key((i as u8) * 3),
					AttributeValue::Equal(T::BenchmarkHelper::contract_value(ATTRIBUTE_VALUE)),
				),
			})
			.collect::<Vec<_>>()
			.try_into()
			.unwrap(),
		burn_fees: false,
		rewards,
		cancel_fee: 333_u64.unique_saturated_into(),
		nft_stake_amount: num_stake_clauses as u8,
		nft_fee_amount: num_fee_clauses as u8,
		is_snipeable: true,
	}
}

#[benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn set_creator() {
		let creator = account::<T>("creator");

		#[extrinsic_call]
		_(RawOrigin::Root, creator.clone());

		assert_last_event::<T>(Event::CreatorSet { creator });
	}

	#[benchmark]
	fn set_contract_collection_id() {
		let creator = create_creator::<T>(None).expect("Account should be created");
		let collection_id = CollectionIdOf::<T>::unique_saturated_from(0_u32);
		ContractCollectionId::<T>::kill();

		#[extrinsic_call]
		_(RawOrigin::Signed(creator), collection_id);

		assert_last_event::<T>(Event::ContractCollectionSet { collection_id });
	}

	#[benchmark]
	fn set_global_config() {
		let creator = create_creator::<T>(None).expect("Account should be created");
		let new_config = GlobalConfig::default();

		#[extrinsic_call]
		_(RawOrigin::Signed(creator), new_config);

		assert_last_event::<T>(Event::SetGlobalConfig { new_config });
	}

	#[benchmark]
	fn create_token_reward(
		m: Linear<0, { T::MaxStakingClauses::get() }>,
		n: Linear<0, { T::MaxFeeClauses::get() }>,
	) {
		let creator = create_creator::<T>(None).expect("Account should be created");
		let rewards: BoundedRewardsOf<T> =
			BoundedVec::try_from(vec![Reward::Tokens(123_u64.unique_saturated_into())])
				.expect("Should create rewards");
		let contract = contract_with::<T>(m, n, rewards, Mode::Staker);
		let contract_id = T::BenchmarkHelper::item_id(0_u16);

		#[extrinsic_call]
		create(RawOrigin::Signed(creator), contract_id, contract, None, None);

		assert_last_event::<T>(Event::Created { contract_id });
	}

	#[benchmark]
	fn create_nft_reward(
		m: Linear<0, { T::MaxStakingClauses::get() }>,
		n: Linear<0, { T::MaxFeeClauses::get() }>,
	) {
		let reward_nft_item = 123_u16;
		let rewards: BoundedRewardsOf<T> = BoundedVec::try_from(vec![Reward::Nft(NftId(
			REWARD_COLLECTION.unique_saturated_into(),
			T::BenchmarkHelper::item_id(reward_nft_item),
		))])
		.expect("Should create rewards");
		let contract = contract_with::<T>(m, n, rewards, Mode::Staker);
		let contract_id = T::BenchmarkHelper::item_id(0_u16);
		let creator =
			create_creator::<T>(Some(vec![reward_nft_item])).expect("Account should be created");

		#[extrinsic_call]
		create(RawOrigin::Signed(creator), contract_id, contract, None, None);

		assert_last_event::<T>(Event::Created { contract_id });
	}

	#[benchmark]
	fn remove_token_reward(
		m: Linear<0, { T::MaxStakingClauses::get() }>,
		n: Linear<0, { T::MaxFeeClauses::get() }>,
	) {
		let rewards: BoundedRewardsOf<T> =
			BoundedVec::try_from(vec![Reward::Tokens(123_u64.unique_saturated_into())])
				.expect("Should create rewards");
		let contract = contract_with::<T>(m, n, rewards, Mode::Staker);
		let contract_id = T::BenchmarkHelper::item_id(0_u16);
		let creator = create_creator::<T>(None).expect("Account should be created");
		create_contract::<T>(creator.clone(), contract_id, contract)
			.expect("Contract should be created");

		#[extrinsic_call]
		remove(RawOrigin::Signed(creator), contract_id);

		assert_last_event::<T>(Event::Removed { contract_id });
	}

	#[benchmark]
	fn remove_nft_reward(
		m: Linear<0, { T::MaxStakingClauses::get() }>,
		n: Linear<0, { T::MaxFeeClauses::get() }>,
	) {
		let reward_nft_item = 2_u16;
		let rewards: BoundedRewardsOf<T> = BoundedVec::try_from(vec![Reward::Nft(NftId(
			REWARD_COLLECTION.unique_saturated_into(),
			T::BenchmarkHelper::item_id(reward_nft_item),
		))])
		.expect("Should create rewards");
		let contract = contract_with::<T>(m, n, rewards, Mode::Staker);
		let contract_id = T::BenchmarkHelper::item_id(0_u16);
		let creator =
			create_creator::<T>(Some(vec![reward_nft_item])).expect("Account should be created");
		create_contract::<T>(creator.clone(), contract_id, contract)
			.expect("Contract should be created");

		#[extrinsic_call]
		remove(RawOrigin::Signed(creator), contract_id);

		assert_last_event::<T>(Event::Removed { contract_id });
	}

	#[benchmark]
	fn accept_token_reward(
		m: Linear<0, { T::MaxStakingClauses::get() }>,
		n: Linear<0, { T::MaxFeeClauses::get() }>,
	) {
		let rewards: BoundedRewardsOf<T> =
			BoundedVec::try_from(vec![Reward::Tokens(123_u64.unique_saturated_into())])
				.expect("Should create rewards");
		let contract = contract_with::<T>(m, n, rewards, Mode::Staker);
		let contract_id = T::BenchmarkHelper::item_id(0_u16);

		let creator = create_creator::<T>(None).expect("Account should be created");
		create_contract::<T>(creator, contract_id, contract).expect("Contract should be created");

		let by = account::<T>("staker");
		create_collections::<T>(&by, 2).expect("Collections should be created");
		let (stakes, fees) = stakes_and_fees::<T>(m, n, &by, Mode::Staker)
			.expect("Stakes and fees should be created");

		#[extrinsic_call]
		accept(RawOrigin::Signed(by.clone()), contract_id, stakes, fees);

		assert_last_event::<T>(Event::Accepted { by, contract_id });
	}

	#[benchmark]
	fn accept_nft_reward(
		m: Linear<0, { T::MaxStakingClauses::get() }>,
		n: Linear<0, { T::MaxFeeClauses::get() }>,
	) {
		let reward_nft_item = 2_u16;
		let rewards: BoundedRewardsOf<T> = BoundedVec::try_from(vec![Reward::Nft(NftId(
			REWARD_COLLECTION.unique_saturated_into(),
			T::BenchmarkHelper::item_id(reward_nft_item),
		))])
		.expect("Should create rewards");
		let contract = contract_with::<T>(m, n, rewards, Mode::Staker);
		let contract_id = T::BenchmarkHelper::item_id(0_u16);

		let creator =
			create_creator::<T>(Some(vec![reward_nft_item])).expect("Account should be created");
		create_contract::<T>(creator, contract_id, contract).expect("Contract should be created");

		let by = account::<T>("staker");
		create_collections::<T>(&by, 2).expect("Collections should be created");
		let (stakes, fees) = stakes_and_fees::<T>(m, n, &by, Mode::Staker)
			.expect("Stakes and fees should be created");

		#[extrinsic_call]
		accept(RawOrigin::Signed(by.clone()), contract_id, stakes, fees);

		assert_last_event::<T>(Event::Accepted { by, contract_id });
	}

	#[benchmark]
	fn cancel_token_reward(
		m: Linear<0, { T::MaxStakingClauses::get() }>,
		n: Linear<0, { T::MaxFeeClauses::get() }>,
	) {
		let rewards: BoundedRewardsOf<T> =
			BoundedVec::try_from(vec![Reward::Tokens(123_u64.unique_saturated_into())])
				.expect("Should create rewards");
		let mut contract = contract_with::<T>(m, n, rewards, Mode::Staker);
		contract.stake_duration = 100_u32.unique_saturated_into();
		let contract_id = T::BenchmarkHelper::item_id(0_u16);

		let creator = create_creator::<T>(None).expect("Account should be created");
		create_contract::<T>(creator, contract_id, contract).expect("Contract should be created");

		let by = account::<T>("staker");
		create_collections::<T>(&by, 2).expect("Collections should be created");
		accept_contract::<T>(m, n, by.clone(), contract_id, Mode::Staker)
			.expect("Contract should be accepted");

		#[extrinsic_call]
		cancel(RawOrigin::Signed(by.clone()), contract_id);

		assert_last_event::<T>(Event::Cancelled { by, contract_id });
	}

	#[benchmark]
	fn cancel_nft_reward(
		m: Linear<0, { T::MaxStakingClauses::get() }>,
		n: Linear<0, { T::MaxFeeClauses::get() }>,
	) {
		let reward_nft_item = 2_u16;
		let rewards: BoundedRewardsOf<T> = BoundedVec::try_from(vec![Reward::Nft(NftId(
			REWARD_COLLECTION.unique_saturated_into(),
			T::BenchmarkHelper::item_id(reward_nft_item),
		))])
		.expect("Should create rewards");
		let mut contract = contract_with::<T>(m, n, rewards, Mode::Staker);
		contract.stake_duration = 100_u32.unique_saturated_into();
		let contract_id = T::BenchmarkHelper::item_id(0_u16);

		let creator =
			create_creator::<T>(Some(vec![reward_nft_item])).expect("Account should be created");
		create_contract::<T>(creator, contract_id, contract).expect("Contract should be created");

		let by = account::<T>("staker");
		create_collections::<T>(&by, 2).expect("Collections should be created");
		accept_contract::<T>(m, n, by.clone(), contract_id, Mode::Staker)
			.expect("Contract should be accepted");

		#[extrinsic_call]
		cancel(RawOrigin::Signed(by.clone()), contract_id);

		assert_last_event::<T>(Event::Cancelled { by, contract_id });
	}

	#[benchmark]
	fn claim_token_reward(
		m: Linear<0, { T::MaxStakingClauses::get() }>,
		n: Linear<0, { T::MaxFeeClauses::get() }>,
	) {
		let rewards: BoundedRewardsOf<T> =
			BoundedVec::try_from(vec![Reward::Tokens(123_u64.unique_saturated_into())])
				.expect("Should create rewards");
		let contract = contract_with::<T>(m, n, rewards.clone(), Mode::Staker);
		let contract_id = T::BenchmarkHelper::item_id(0_u16);

		let creator = create_creator::<T>(None).expect("Account should be created");
		create_contract::<T>(creator, contract_id, contract).expect("Contract should be created");

		let by = account::<T>("staker");
		create_collections::<T>(&by, 2).expect("Collections should be created");
		accept_contract::<T>(m, n, by.clone(), contract_id, Mode::Staker)
			.expect("Contract should be accepted");

		#[extrinsic_call]
		claim(RawOrigin::Signed(by.clone()), contract_id, None);

		assert_last_event::<T>(Event::Claimed { by, contract_id, rewards });
	}

	#[benchmark]
	fn claim_nft_reward(
		m: Linear<0, { T::MaxStakingClauses::get() }>,
		n: Linear<0, { T::MaxFeeClauses::get() }>,
	) {
		let reward_nft_item = 2_u16;
		let rewards: BoundedRewardsOf<T> = BoundedVec::try_from(vec![Reward::Nft(NftId(
			REWARD_COLLECTION.unique_saturated_into(),
			T::BenchmarkHelper::item_id(reward_nft_item),
		))])
		.expect("Should create rewards");
		let contract = contract_with::<T>(m, n, rewards.clone(), Mode::Staker);
		let contract_id = T::BenchmarkHelper::item_id(0_u16);

		let creator =
			create_creator::<T>(Some(vec![reward_nft_item])).expect("Account should be created");
		create_contract::<T>(creator, contract_id, contract).expect("Contract should be created");

		let by = account::<T>("staker");
		create_collections::<T>(&by, 2).expect("Collections should be created");
		accept_contract::<T>(m, n, by.clone(), contract_id, Mode::Staker)
			.expect("Contract should be accepted");

		#[extrinsic_call]
		claim(RawOrigin::Signed(by.clone()), contract_id, None);

		assert_last_event::<T>(Event::Claimed { by, contract_id, rewards });
	}

	#[benchmark]
	fn snipe_token_reward(
		m: Linear<0, { T::MaxStakingClauses::get() }>,
		n: Linear<0, { T::MaxFeeClauses::get() }>,
	) {
		let rewards: BoundedRewardsOf<T> =
			BoundedVec::try_from(vec![Reward::Tokens(123_u64.unique_saturated_into())])
				.expect("Should create rewards");
		let contract = contract_with::<T>(m, n, rewards.clone(), Mode::Staker);
		let contract_id = T::BenchmarkHelper::item_id(0_u16);

		let mut sniper_contract = contract_with::<T>(m, n, rewards.clone(), Mode::Sniper);
		sniper_contract.stake_duration = 100_u32.unique_saturated_into();
		let sniper_contract_id = T::BenchmarkHelper::item_id(1_u16);

		let creator = create_creator::<T>(None).expect("Account should be created");
		create_contract::<T>(creator.clone(), contract_id, contract.clone())
			.expect("Contract should be created");
		create_contract::<T>(creator, sniper_contract_id, sniper_contract)
			.expect("Contract should be created");

		let by = account::<T>("staker");
		create_collections::<T>(&by, 2).expect("Collections should be created");
		accept_contract::<T>(m, n, by, contract_id, Mode::Staker)
			.expect("Contract should be accepted");

		let sniper = account::<T>("sniper");
		create_collections::<T>(&sniper, 2).expect("Collections should be created");
		accept_contract::<T>(m, n, sniper.clone(), sniper_contract_id, Mode::Sniper)
			.expect("Contract should be accepted");

		// Advance block past contract expiry.
		frame_system::Pallet::<T>::set_block_number(
			contract.stake_duration + contract.claim_duration + One::one(),
		);

		#[extrinsic_call]
		snipe(RawOrigin::Signed(sniper.clone()), contract_id);

		assert_last_event::<T>(Event::Sniped { by: sniper, contract_id, rewards });
	}

	#[benchmark]
	fn snipe_nft_reward(
		m: Linear<0, { T::MaxStakingClauses::get() }>,
		n: Linear<0, { T::MaxFeeClauses::get() }>,
	) {
		let reward_nft_item = 2_u16;
		let rewards: BoundedRewardsOf<T> = BoundedVec::try_from(vec![Reward::Nft(NftId(
			REWARD_COLLECTION.unique_saturated_into(),
			T::BenchmarkHelper::item_id(reward_nft_item),
		))])
		.expect("Should create rewards");
		let contract = contract_with::<T>(m, n, rewards.clone(), Mode::Staker);
		let contract_id = T::BenchmarkHelper::item_id(0_u16);

		let sniper_reward_nft_item = 123_u16;
		let sniper_rewards = BoundedVec::try_from(vec![Reward::Nft(NftId(
			REWARD_COLLECTION.unique_saturated_into(),
			T::BenchmarkHelper::item_id(sniper_reward_nft_item),
		))])
		.expect("Should create rewards");
		let mut sniper_contract = contract_with::<T>(m, n, sniper_rewards, Mode::Sniper);
		sniper_contract.stake_duration = 100_u32.unique_saturated_into();
		let sniper_contract_id = T::BenchmarkHelper::item_id(1_u16);

		let creator = create_creator::<T>(Some(vec![reward_nft_item, sniper_reward_nft_item]))
			.expect("Account should be created");
		create_contract::<T>(creator.clone(), contract_id, contract.clone())
			.expect("Contract should be created");
		create_contract::<T>(creator, sniper_contract_id, sniper_contract)
			.expect("Contract should be created");

		let by = account::<T>("staker");
		create_collections::<T>(&by, 2).expect("Collections should be created");
		accept_contract::<T>(m, n, by, contract_id, Mode::Staker)
			.expect("Contract should be accepted");

		let sniper = account::<T>("sniper");
		create_collections::<T>(&sniper, 2).expect("Collections should be created");
		accept_contract::<T>(m, n, sniper.clone(), sniper_contract_id, Mode::Sniper)
			.expect("Contract should be accepted");

		// Advance block past contract expiry.
		frame_system::Pallet::<T>::set_block_number(
			contract.stake_duration + contract.claim_duration + One::one(),
		);

		#[extrinsic_call]
		snipe(RawOrigin::Signed(sniper.clone()), contract_id);

		assert_last_event::<T>(Event::Sniped { by: sniper, contract_id, rewards });
	}

	impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Runtime);
}
