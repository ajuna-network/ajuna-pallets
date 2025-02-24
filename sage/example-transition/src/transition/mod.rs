use crate::{
	asset::{Asset, AssetId, VariantType},
	error::*,
	rules::*,
	transition::utils::CasinoJamUtils,
};

use ajuna_primitives::{payment_handler::NativeId, sage_api::SageApi};
use sage_api::{rules::*, traits::TransitionOutput, SageGameTransition, TransitionError};

use frame_support::{
	pallet_prelude::{Decode, Encode, TypeInfo},
	Parameter,
};
use parity_scale_codec::{Codec, MaxEncodedLen};
use sp_core::H256;
use sp_runtime::{
	traits::{AtLeast32BitUnsigned, BlockNumber as BlockNumberT, Member},
	SaturatedConversion,
};
use sp_std::{marker::PhantomData, vec::Vec};

mod enums;
mod utils;

pub use enums::*;
pub(crate) use utils::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum AssetType {
	Player,
	Machine(MachineType),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum CasinoAction {
	Create(AssetType),
	Deposit(AssetType, TokenType),
	Gamble(MultiplierType),
	Withdraw(AssetType, TokenType),
	Rent(RentDuration),
	Reserve(ReservationDuration),
	Release,
	Kick,
	Return,
}

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Debug, Default, Clone, PartialEq, Eq)]
pub struct CasinoJamTransitionConfig {
	pub reward_multiplier: u8,
}

pub struct CasinoJamTransition<AccountId, BlockNumber, Sage> {
	_phantom: PhantomData<(AccountId, BlockNumber, Sage)>,
}

impl<AccountId, BlockNumber, Balance, Sage> CasinoJamTransition<AccountId, BlockNumber, Sage>
where
	AccountId: Member + Codec,
	BlockNumber: BlockNumberT,
	Balance: Member + Parameter + AtLeast32BitUnsigned + MaxEncodedLen,
	Sage: SageApi<
		AccountId = AccountId,
		AssetId = AssetId,
		Asset = Asset<BlockNumber>,
		Balance = Balance,
		BlockNumber = BlockNumber,
		TransitionConfig = CasinoJamTransitionConfig,
		HashOutput = H256,
	>,
{
	fn try_get_asset(asset_id: &AssetId) -> Result<Asset<BlockNumber>, TransitionError> {
		let asset = Sage::get_asset(asset_id)
			.map_err(|_| TransitionError::Transition { code: ASSET_NOT_FOUND })?;

		Ok(asset)
	}

	fn try_get_assets(
		asset_ids: &[AssetId],
	) -> Result<Vec<(AssetId, Asset<BlockNumber>)>, TransitionError> {
		asset_ids
			.iter()
			.copied()
			.map(|asset_id| Ok((asset_id, Self::try_get_asset(&asset_id)?)))
			.collect::<Result<Vec<_>, _>>()
	}

	fn get_asset_funds(
		asset_id: &AssetId,
		payment_asset: Option<&Sage::FungiblesAssetId>,
	) -> Balance {
		let fund_id = if let Some(payment) = payment_asset {
			payment
		} else {
			&Sage::FungiblesAssetId::get_native_id()
		};

		Sage::inspect_asset_funds(asset_id, fund_id)
	}

	fn deposit_funds_to_asset(
		asset_id: &AssetId,
		from: &AccountId,
		amount: Balance,
	) -> Result<(), TransitionError> {
		let fund_id = Sage::FungiblesAssetId::get_native_id();
		Sage::deposit_funds_to_asset(asset_id, from, fund_id, amount)
			.map_err(|_| TransitionError::Transition { code: ASSET_COULD_NOT_RECEIVE_FUNDS })
	}

	fn withdraw_funds_from_asset(
		asset_id: &AssetId,
		to: &AccountId,
		amount: Balance,
	) -> Result<(), TransitionError> {
		let fund_id = Sage::FungiblesAssetId::get_native_id();
		Sage::transfer_funds_from_asset(asset_id, to, fund_id, amount)
			.map_err(|_| TransitionError::Transition { code: ASSET_COULD_NOT_WITHDRAW_FUNDS })
	}

	fn generate_asset_id(nonce: u64) -> AssetId {
		let hash_base = Sage::random_hash(b"create_asset");
		let current_block_number = Sage::get_current_block_number();

		hash_base
			.0
			.into_iter()
			.fold(0 as AssetId, |acc, value| acc.saturating_add(value as AssetId))
			.saturating_add(current_block_number.saturated_into())
			.saturating_add(nonce as AssetId)
	}

	fn verify_transition_rules(
		transition_id: &CasinoAction,
		account_id: &AccountId,
		asset_ids: &[AssetId],
	) -> Result<Vec<(AssetId, Asset<BlockNumber>)>, TransitionError> {
		let mut maybe_assets = None;

		match transition_id {
			CasinoAction::Create(create_action) => {
				ensure_asset_length(asset_ids, 0)?;

				match create_action {
					AssetType::Player => {
						ensure_account_has_no_asset_of_type::<_, _, Sage>(
							account_id,
							VariantType::Player(PlayerType::Human),
						)?;
					},
					AssetType::Machine(machine_type) => {
						ensure_account_has_no_asset_of_type::<_, _, Sage>(
							account_id,
							VariantType::Machine(*machine_type),
						)?;
					},
				}
			},
			CasinoAction::Deposit(asset_type, _) | CasinoAction::Withdraw(asset_type, _) => {
				let assets = Self::try_get_assets(asset_ids)?;

				ensure_asset_length(asset_ids, 1)?;
				ensure_owner_of::<_, _, Sage>(asset_ids, account_id)?;
				match asset_type {
					AssetType::Player => {
						ensure_asset_type_at(&assets, VariantType::Player(PlayerType::Human), 0)?;
					},
					AssetType::Machine(machine_type) => {
						ensure_asset_type_at(&assets, VariantType::Machine(*machine_type), 0)?;
					},
				}

				maybe_assets = Some(assets);
			},
			CasinoAction::Gamble(_) => {
				let assets = Self::try_get_assets(asset_ids)?;

				ensure_asset_length(asset_ids, 4)?;
				ensure_owner_of::<_, _, Sage>(&asset_ids[0..2], account_id)?;
				ensure_asset_type_at(&assets, VariantType::Player(PlayerType::Human), 0)?;
				ensure_asset_type_at(&assets, VariantType::Player(PlayerType::Tracker), 1)?;
				ensure_asset_type_at(&assets, VariantType::Seat, 2)?;
				ensure_asset_type_at(&assets, VariantType::Machine(MachineType::Bandit), 3)?;

				maybe_assets = Some(assets);
			},
			CasinoAction::Rent(_) => {
				let assets = Self::try_get_assets(asset_ids)?;

				ensure_asset_length(asset_ids, 1)?;
				ensure_owner_of::<_, _, Sage>(asset_ids, account_id)?;
				ensure_asset_type_at(&assets, VariantType::Machine(MachineType::Bandit), 0)?;

				maybe_assets = Some(assets);
			},
			CasinoAction::Reserve(_) | CasinoAction::Release => {
				let assets = Self::try_get_assets(asset_ids)?;

				ensure_asset_length(asset_ids, 2)?;
				ensure_owner_of::<_, _, Sage>(&asset_ids[0..1], account_id)?;
				ensure_asset_type_at(&assets, VariantType::Player(PlayerType::Human), 0)?;
				ensure_asset_type_at(&assets, VariantType::Seat, 1)?;

				maybe_assets = Some(assets);
			},
			CasinoAction::Kick => {
				let assets = Self::try_get_assets(asset_ids)?;

				ensure_asset_length(asset_ids, 3)?;
				ensure_owner_of::<_, _, Sage>(&asset_ids[0..1], account_id)?;
				ensure_asset_type_at(&assets, VariantType::Player(PlayerType::Human), 0)?;
				ensure_asset_type_at(&assets, VariantType::Player(PlayerType::Human), 1)?;
				ensure_asset_type_at(&assets, VariantType::Seat, 2)?;

				maybe_assets = Some(assets);
			},
			CasinoAction::Return => {
				let assets = Self::try_get_assets(asset_ids)?;

				ensure_asset_length(asset_ids, 2)?;
				ensure_owner_of::<_, _, Sage>(asset_ids, account_id)?;
				ensure_asset_type_at(&assets, VariantType::Machine(MachineType::Bandit), 0)?;
				ensure_asset_type_at(&assets, VariantType::Seat, 1)?;

				maybe_assets = Some(assets);
			},
		}

		if maybe_assets.is_none() {
			maybe_assets = Some(Self::try_get_assets(asset_ids)?);
		}

		Ok(maybe_assets.unwrap())
	}

	fn transition_assets(
		transition_id: &CasinoAction,
		account_id: &AccountId,
		mut assets: Vec<(AssetId, Asset<BlockNumber>)>,
		payment_asset: Option<Sage::FungiblesAssetId>,
	) -> Result<Vec<TransitionOutput<AssetId, Asset<BlockNumber>>>, TransitionError> {
		let output = match transition_id {
			CasinoAction::Create(action) => {
				let current_block = Sage::get_current_block_number();

				match action {
					AssetType::Player => {
						let player_id = Self::generate_asset_id(17);
						let player = Asset::new_player(player_id, current_block);
						let tracker_id = Self::generate_asset_id(31);
						let tracker = Asset::new_tracker(tracker_id, current_block);

						sp_std::vec![
							TransitionOutput::Minted(player),
							TransitionOutput::Minted(tracker)
						]
					},
					AssetType::Machine(machine_type) => match machine_type {
						MachineType::Bandit => {
							let machine_id = Self::generate_asset_id(13);
							let machine = Asset::new_bandit_machine(machine_id, current_block);
							sp_std::vec![TransitionOutput::Minted(machine)]
						},
					},
				}
			},
			CasinoAction::Deposit(_, token_type) => {
				let (asset_id, asset) =
					assets.pop().ok_or(TransitionError::Transition { code: ASSET_NOT_FOUND })?;

				let fund_amount: Balance = token_type.as_value().into();
				let asset_funds = Self::get_asset_funds(&asset_id, payment_asset.as_ref());

				if asset_funds.checked_add(&fund_amount).is_some() {
					Self::deposit_funds_to_asset(&asset_id, account_id, fund_amount)?;

					sp_std::vec![TransitionOutput::Mutated(asset_id, asset)]
				} else {
					return Err(TransitionError::Transition { code: ASSET_COULD_NOT_RECEIVE_FUNDS });
				}
			},
			CasinoAction::Gamble(amount) => {
				let (bandit_id, mut machine_asset) =
					assets.pop().ok_or(TransitionError::Transition { code: ASSET_NOT_FOUND })?;
				let machine = machine_asset.try_as_machine()?;
				let (seat_id, mut seat_asset) =
					assets.pop().ok_or(TransitionError::Transition { code: ASSET_NOT_FOUND })?;
				let (tracker_id, mut tracker_asset) =
					assets.pop().ok_or(TransitionError::Transition { code: ASSET_NOT_FOUND })?;
				let player_tracker = tracker_asset.try_as_player()?;
				let (human_id, human_asset) =
					assets.pop().ok_or(TransitionError::Transition { code: ASSET_NOT_FOUND })?;

				// We first clear the tracker results
				player_tracker.try_as_tracker()?.clear();

				// We verify that we can move the required funds from player to machine
				let play_fee: Balance = amount.as_value().into();

				let player_funds = Self::get_asset_funds(&human_id, payment_asset.as_ref());
				let bandit_funds = Self::get_asset_funds(&bandit_id, payment_asset.as_ref());
				let can_withdraw_play_fee = player_funds.checked_sub(&play_fee).is_some();
				let can_deposit_play_fee = bandit_funds.checked_add(&play_fee).is_some();

				if !can_withdraw_play_fee {
					return Err(TransitionError::Transition {
						code: ASSET_COULD_NOT_WITHDRAW_PLAY_FEE,
					});
				}
				if !can_deposit_play_fee {
					return Err(TransitionError::Transition {
						code: ASSET_COULD_NOT_RECEIVE_PLAY_FEE,
					});
				}

				let spin_times = sp_std::cmp::min(amount.as_value(), 4);
				let value_1 = machine.value_1_factor.get_value_for(machine.value_1_mul);
				let max_reward_2 = machine.value_2_factor.get_value_for(machine.value_2_mul);
				let max_reward_3 = machine.value_3_factor.get_value_for(machine.value_3_mul);

				// calculate minimum of funds required for the bandit
				// to pay the fix max rewards possible
				let min_reward = value_1;
				let jackpot_max_reward = max_reward_2;
				let special_max_reward = max_reward_3;

				let spin_max_reward = min_reward.saturating_mul(8192);
				let max_reward =
					spin_max_reward.saturating_mul(spin_times).saturating_add(special_max_reward);

				// Verify that we can pay the maximum reward
				let can_withdraw_max_reward =
					bandit_funds.checked_sub(&max_reward.into()).is_some();
				let can_deposit_max_reward = player_funds.checked_add(&max_reward.into()).is_some();

				if !can_withdraw_max_reward {
					return Err(TransitionError::Transition {
						code: ASSET_COULD_NOT_WITHDRAW_MAX_REWARD,
					});
				}
				if !can_deposit_max_reward {
					return Err(TransitionError::Transition {
						code: ASSET_COULD_NOT_RECEIVE_MAX_REWARD,
					});
				}

				// Now we spin the machine!
				let hash = Sage::random_hash(b"casino_gamble").0;
				let full_spins = {
					let maybe_full_spins = CasinoJamUtils::spins(
						spin_times as u8,
						min_reward,
						jackpot_max_reward,
						special_max_reward,
						&hash,
					);

					if maybe_full_spins.is_none() {
						return Err(TransitionError::Transition {
							code: COULD_NOT_PERFORM_MACHINE_SPINS,
						});
					}

					maybe_full_spins.unwrap()
				};

				let reward: Balance = full_spins
					.spin_results
					.iter()
					.map(|spin| spin.reward)
					.fold(0_u32, |acc, value| acc.saturating_add(value))
					.saturating_add(full_spins.jackpot_reward)
					.saturating_add(full_spins.special_reward)
					.into();

				// Verify if we can actually transfer the reward
				let can_withdraw_reward = bandit_funds.checked_sub(&reward).is_some();
				let can_deposit_reward = player_funds.checked_add(&reward).is_some();
				if !can_withdraw_reward {
					return Err(TransitionError::Transition {
						code: ASSET_COULD_NOT_WITHDRAW_SPIN_REWARD,
					});
				}
				if !can_deposit_reward {
					return Err(TransitionError::Transition {
						code: ASSET_COULD_NOT_RECEIVE_SPIN_REWARD,
					});
				}

				// First we transfer the play_fee to the bandit machine
				Self::withdraw_funds_from_asset(&bandit_id, account_id, play_fee.clone())?;
				Self::deposit_funds_to_asset(&human_id, account_id, play_fee)?;

				// Then we register the spin result into the tracker
				{
					let tracker = player_tracker.try_as_tracker()?;

					for (i, spin) in full_spins.spin_results.iter().enumerate() {
						match i {
							0 => tracker.slot_a_result = spin.get_packed(),
							1 => tracker.slot_b_result = spin.get_packed(),
							2 => tracker.slot_c_result = spin.get_packed(),
							3 => tracker.slot_d_result = spin.get_packed(),
							_ => {},
						}
					}
				}

				// Finally we transfer the reward to the player asset
				Self::withdraw_funds_from_asset(&bandit_id, account_id, reward.clone())?;
				Self::deposit_funds_to_asset(&human_id, account_id, reward)?;

				// We update the seat
				let seat = seat_asset.try_as_seat()?;
				let current_block = Sage::get_current_block_number();

				seat.player_action_count = seat.player_action_count.saturating_add(1);
				seat.last_action_block =
					current_block.saturating_sub(seat.reservation_start_block).saturated_into();

				sp_std::vec![
					TransitionOutput::Mutated(human_id, human_asset),
					TransitionOutput::Mutated(tracker_id, tracker_asset),
					TransitionOutput::Mutated(seat_id, seat_asset),
					TransitionOutput::Mutated(bandit_id, machine_asset),
				]
			},
			CasinoAction::Withdraw(_, token_type) => {
				let (asset_id, mut asset) =
					assets.pop().ok_or(TransitionError::Transition { code: ASSET_NOT_FOUND })?;

				if let Ok(machine) = asset.try_as_machine() {
					if machine.seat_linked > 0 {
						return Err(TransitionError::Transition {
							code: MACHINE_STILL_HAS_LINKED_SEATS,
						});
					}
				}

				let withdraw_amount: Balance = token_type.as_value().into();
				let asset_funds = Self::get_asset_funds(&asset_id, payment_asset.as_ref());

				if asset_funds.checked_sub(&withdraw_amount).is_some() {
					Self::withdraw_funds_from_asset(&asset_id, account_id, withdraw_amount)?;

					sp_std::vec![TransitionOutput::Mutated(asset_id, asset)]
				} else {
					return Err(TransitionError::Transition { code: ASSET_COULD_NOT_RECEIVE_FUNDS });
				}
			},
			CasinoAction::Rent(rent_duration) => {
				let (asset_id, mut asset) =
					assets.pop().ok_or(TransitionError::Transition { code: ASSET_NOT_FOUND })?;
				let machine = asset.try_as_machine()?;

				if machine.seat_linked >= machine.seat_limit {
					return Err(TransitionError::Transition {
						code: MACHINE_CANNOT_RENT_MORE_SEATS,
					});
				}

				machine.seat_linked = machine.seat_linked.saturating_add(1);

				let seat_id = Self::generate_asset_id(3);
				let current_block = Sage::get_current_block_number();
				let seat = Asset::<BlockNumber>::new_seat(
					seat_id,
					current_block,
					asset_id,
					*rent_duration,
				);

				sp_std::vec![
					TransitionOutput::Mutated(asset_id, asset),
					TransitionOutput::Minted(seat)
				]
			},
			CasinoAction::Reserve(reservation_duration) => {
				let (asset_id_2, mut asset_2) =
					assets.pop().ok_or(TransitionError::Transition { code: ASSET_NOT_FOUND })?;
				let (asset_id_1, mut asset_1) =
					assets.pop().ok_or(TransitionError::Transition { code: ASSET_NOT_FOUND })?;
				let human = asset_1.try_as_player()?.try_as_human()?;

				let current_block = Sage::get_current_block_number();

				if asset_2.try_as_seat()?.player_id.is_some() || human.seat_id.is_some() {
					return Err(TransitionError::Transition { code: SEAT_IS_NOT_LINKED_TO_PLAYER });
				}

				let rent_blocks = asset_2.try_as_seat()?.rent_duration.get_rent_duration_blocks();
				let last_block_of_validity = asset_2.genesis.saturating_add(rent_blocks.into());

				// Verify if seat is running out of time with this reservation
				let reservation_blocks = reservation_duration.get_reservation_duration_blocks();
				if current_block > last_block_of_validity.saturating_sub(reservation_blocks.into())
				{
					return Err(TransitionError::Transition { code: SEAT_RESERVATION_HAS_EXPIRED });
				}

				let player_fee = asset_2.try_as_seat()?.player_fee as u32;
				let reservation_fee =
					reservation_duration.get_reservation_duration_fees(player_fee);

				let player_funds = Self::get_asset_funds(&asset_id_1, payment_asset.as_ref());
				let seat_funds = Self::get_asset_funds(&asset_id_2, payment_asset.as_ref());

				let can_withdraw_play_fee =
					player_funds.checked_sub(&reservation_fee.into()).is_some();
				let can_deposit_play_fee =
					seat_funds.checked_add(&reservation_fee.into()).is_some();
				if !can_withdraw_play_fee {
					return Err(TransitionError::Transition {
						code: ASSET_COULD_NOT_WITHDRAW_PLAY_FEE,
					});
				}
				if !can_deposit_play_fee {
					return Err(TransitionError::Transition {
						code: ASSET_COULD_NOT_RECEIVE_PLAY_FEE,
					});
				}

				Self::withdraw_funds_from_asset(&asset_id_1, account_id, reservation_fee.into())?;
				Self::deposit_funds_to_asset(&asset_id_2, account_id, reservation_fee.into())?;

				let seat = asset_2.try_as_seat()?;

				human.seat_id = Some(asset_id_2);
				seat.player_id = Some(asset_id_1);
				seat.reservation_start_block = current_block;
				seat.reservation_duration = *reservation_duration;
				seat.last_action_block = 0;
				seat.player_action_count = 0;

				sp_std::vec![
					TransitionOutput::Mutated(asset_id_1, asset_1),
					TransitionOutput::Mutated(asset_id_2, asset_2),
				]
			},
			CasinoAction::Release => {
				let (asset_id_2, mut asset_2) =
					assets.pop().ok_or(TransitionError::Transition { code: ASSET_NOT_FOUND })?;
				let seat = asset_2.try_as_seat()?;
				let (asset_id_1, mut asset_1) =
					assets.pop().ok_or(TransitionError::Transition { code: ASSET_NOT_FOUND })?;
				let human = asset_1.try_as_player()?.try_as_human()?;

				// seat is not occupied, player is not seated, or they are not linked to each oth
				if !seat.is_linked_to(asset_id_1) || !human.is_linked_to(asset_id_2) {
					return Err(TransitionError::Transition {
						code: SEAT_IS_NOT_LINKED_TO_SPECIFIED_PLAYER,
					});
				}

				let seat_funds = Self::get_asset_funds(&asset_id_2, payment_asset.as_ref());
				if seat_funds.is_zero() {
					return Err(TransitionError::Transition { code: SEAT_HAS_NO_FUNDS });
				}

				let full_reservation_fee =
					seat.reservation_duration.get_reservation_duration_fees(seat.player_fee as u32);
				let usage_fee = SEAT_USAGE_FEE_PERC * full_reservation_fee.saturating_div(100);
				let reservation_fee: Balance =
					full_reservation_fee.saturating_sub(usage_fee).into();

				let can_withdraw_seat_fee = seat_funds.checked_sub(&reservation_fee).is_some();
				let can_deposit_seat_fee = seat_funds.checked_add(&reservation_fee).is_some();
				if !can_withdraw_seat_fee {
					return Err(TransitionError::Transition {
						code: ASSET_COULD_NOT_WITHDRAW_PLAY_FEE,
					});
				}
				if !can_deposit_seat_fee {
					return Err(TransitionError::Transition {
						code: ASSET_COULD_NOT_RECEIVE_PLAY_FEE,
					});
				}

				Self::withdraw_funds_from_asset(&asset_id_2, account_id, reservation_fee.clone())?;
				Self::deposit_funds_to_asset(&asset_id_1, account_id, reservation_fee)?;

				seat.release();
				human.release();

				sp_std::vec![
					TransitionOutput::Mutated(asset_id_1, asset_1),
					TransitionOutput::Mutated(asset_id_2, asset_2),
				]
			},
			CasinoAction::Kick => {
				let (seat_id, mut asset_3) =
					assets.pop().ok_or(TransitionError::Transition { code: ASSET_NOT_FOUND })?;
				let seat = asset_3.try_as_seat()?;
				let (human_id, mut asset_2) =
					assets.pop().ok_or(TransitionError::Transition { code: ASSET_NOT_FOUND })?;
				let human = asset_2.try_as_player()?.try_as_human()?;
				let (sniper_id, asset_1) =
					assets.pop().ok_or(TransitionError::Transition { code: ASSET_NOT_FOUND })?;

				let current_block = Sage::get_current_block_number();

				// seat is not occupied, player is not seated, or they are not linked to each oth
				if !seat.is_linked_to(human_id) || !human.is_linked_to(seat_id) {
					return Err(TransitionError::Transition {
						code: SEAT_IS_NOT_LINKED_TO_SPECIFIED_PLAYER,
					});
				}

				let reservation_blocks =
					seat.reservation_duration.get_reservation_duration_blocks();
				let is_reservation_valid =
					seat.reservation_start_block.saturating_add(reservation_blocks.into()) >=
						current_block;
				let is_grace_period = seat
					.reservation_start_block
					.saturating_add(seat.last_action_block.into())
					.saturating_sub(seat.player_grace_period.into()) >=
					current_block;

				if !is_reservation_valid && !is_grace_period {
					return Err(TransitionError::Transition { code: SEAT_RESERVATION_IS_NOT_VALID });
				}

				let reservation_fee = Self::get_asset_funds(&seat_id, payment_asset.as_ref());
				if reservation_fee.is_zero() {
					return Err(TransitionError::Transition { code: SEAT_HAS_NO_FUNDS });
				}

				Self::withdraw_funds_from_asset(&seat_id, account_id, reservation_fee.clone())?;
				Self::deposit_funds_to_asset(&sniper_id, account_id, reservation_fee)?;

				human.release();
				seat.release();

				sp_std::vec![
					TransitionOutput::Mutated(sniper_id, asset_1),
					TransitionOutput::Mutated(human_id, asset_2),
					TransitionOutput::Mutated(seat_id, asset_3),
				]
			},
			CasinoAction::Return => {
				let (seat_id, mut asset_2) =
					assets.pop().ok_or(TransitionError::Transition { code: ASSET_NOT_FOUND })?;
				let seat = asset_2.try_as_seat()?;
				let (machine_id, mut asset_1) =
					assets.pop().ok_or(TransitionError::Transition { code: ASSET_NOT_FOUND })?;
				let machine = asset_1.try_as_machine()?;

				if seat.machine_id.is_none() || seat.machine_id != Some(machine_id) {
					return Err(TransitionError::Transition { code: SEAT_IS_NOT_LINED_TO_MACHINE });
				}

				if machine.seat_linked == 0 {
					return Err(TransitionError::Transition { code: MACHINE_HAS_NO_LINKED_SEATS });
				}

				if seat.player_id.is_some() {
					return Err(TransitionError::Transition {
						code: SEAT_IS_STILL_LINKED_TO_PLAYER,
					});
				}

				machine.seat_linked -= 1;

				let seat_funds = Self::get_asset_funds(&seat_id, payment_asset.as_ref());
				if !seat_funds.is_zero() {
					let can_withdraw_seat_funds = seat_funds.checked_sub(&seat_funds).is_some();
					let can_deposit_seat_funds = seat_funds.checked_add(&seat_funds).is_some();
					if !can_withdraw_seat_funds {
						return Err(TransitionError::Transition {
							code: ASSET_COULD_NOT_WITHDRAW_PLAY_FEE,
						});
					}
					if !can_deposit_seat_funds {
						return Err(TransitionError::Transition {
							code: ASSET_COULD_NOT_RECEIVE_PLAY_FEE,
						});
					}

					Self::withdraw_funds_from_asset(&seat_id, account_id, seat_funds.clone())?;
					Self::deposit_funds_to_asset(&machine_id, account_id, seat_funds)?;
				}

				seat.release();

				sp_std::vec![
					TransitionOutput::Mutated(machine_id, asset_1),
					TransitionOutput::Consumed(seat_id),
				]
			},
		};

		Ok(output)
	}
}

impl<AccountId, BlockNumber, Balance, Sage> SageGameTransition
	for CasinoJamTransition<AccountId, BlockNumber, Sage>
where
	AccountId: Member + Codec,
	BlockNumber: BlockNumberT,
	Balance: Member + Parameter + AtLeast32BitUnsigned + MaxEncodedLen,
	Sage: SageApi<
		AccountId = AccountId,
		AssetId = AssetId,
		Asset = Asset<BlockNumber>,
		Balance = Balance,
		BlockNumber = BlockNumber,
		TransitionConfig = CasinoJamTransitionConfig,
		HashOutput = H256,
	>,
{
	type TransitionId = CasinoAction;
	type TransitionConfig = CasinoJamTransitionConfig;
	type AccountId = AccountId;
	type AssetId = AssetId;
	type Asset = Asset<BlockNumber>;
	type Extra = ();
	type PaymentFungible = Sage::FungiblesAssetId;

	fn do_transition(
		transition_id: &Self::TransitionId,
		account_id: &Self::AccountId,
		assets_ids: &[Self::AssetId],
		_: &Self::Extra,
		payment_asset: Option<Self::PaymentFungible>,
	) -> Result<Vec<TransitionOutput<Self::AssetId, Self::Asset>>, TransitionError> {
		let assets = Self::verify_transition_rules(transition_id, account_id, assets_ids)?;
		Self::transition_assets(transition_id, account_id, assets, payment_asset)
	}
}
