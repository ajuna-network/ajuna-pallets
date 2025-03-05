use sp_std::vec::Vec;

pub(crate) const ASSET_COLLECTION_ID: u8 = 1;

pub(crate) const BANDIT_MAX_SPINS: u8 = 4;

pub(crate) const SEAT_USAGE_FEE_PERC: u32 = 1;
pub(crate) const BASE_RESERVATION_TIME: u32 = BLOCKS_PER_MINUTE * 5;

pub(crate) const BLOCKS_PER_MINUTE: u32 = 10;
pub(crate) const BLOCKS_PER_HOUR: u32 = 60 * BLOCKS_PER_MINUTE;
pub(crate) const BLOCKS_PER_DAY: u32 = 24 * BLOCKS_PER_HOUR;

pub(super) struct FullSpin {
	pub spin_results: Vec<SpinResult>,
	pub jackpot_reward: u32,
	pub special_reward: u32,
}

pub(super) struct SpinResult {
	pub slot_1: u8,
	pub slot_2: u8,
	pub slot_3: u8,
	pub bonus_1: u8,
	pub bonus_2: u8,
	pub reward: u32,
}

impl SpinResult {
	pub(super) fn get_packed(&self) -> (u16, u8) {
		CasinoJamUtils::pack_slot_result(
			self.slot_1,
			self.slot_2,
			self.slot_3,
			self.bonus_1,
			self.bonus_2,
		)
	}
}

pub(super) struct CasinoJamUtils;

impl CasinoJamUtils {
	pub(super) fn pack_slot_result(
		slot_1: u8,
		slot_2: u8,
		slot_3: u8,
		bonus_1: u8,
		bonus_2: u8,
	) -> (u16, u8) {
		let mut slot: u16 = 0;

		slot |= (slot_1 as u16 & 0x0F) << 12; // Bits 15-12
		slot |= (slot_2 as u16 & 0x0F) << 8; // Bits 11-8
		slot |= (slot_3 as u16 & 0x0F) << 4; // Bits 7-4

		let mut bonus = 0;
		bonus |= (bonus_1 & 0x0F) << 4; // Bits 3-2
		bonus |= bonus_2 & 0x0F; // Bits 1-0

		(slot, bonus)
	}

	fn single_spin_reward(min_reward: u32, spin: &SpinResult) -> u32 {
		let factor_multiplier = match (spin.slot_1, spin.slot_2, spin.slot_3) {
			(0, 0, 0) => 0,
			(1, 1, 1) => 5,
			(2, 2, 2) => 10,
			(3, 3, 3) => 25,
			(4, 4, 4) => 50,
			(5, 5, 5) => 100,
			(6, 6, 6) => 200,
			(7, 7, 7) => 500,
			(8, 8, 8) => 750,
			(9, 9, 9) => 1500,
			_ => 0,
		};
		let spin_factor = min_reward.saturating_mul(factor_multiplier);

		let bonus_multiplier = match (spin.bonus_1, spin.bonus_2) {
			(1, 1) => 1,
			(2, 2) => 2,
			(3, 3) => 2,
			(4, 4) => 2,
			(5, 5) => 2,
			(6, 6) => 4,
			(7, 7) => 4,
			(8, 8) => 4,
			(9, 9) => 8,
			_ => 0,
		};
		let bonus_factor = min_reward.saturating_mul(bonus_multiplier);

		let is_full_line = spin.slot_1 == spin.bonus_1 && spin_factor > 0 && bonus_factor > 0;

		let mut reward = spin_factor;

		if spin_factor > 0 {
			if is_full_line {
				reward = spin_factor.saturating_mul(128_u32.saturating_div(bonus_factor))
			} else if bonus_factor > 0 {
				reward = spin_factor.saturating_add(32_u32.saturating_mul(bonus_factor))
			}
		}

		if reward == 0 {
			reward = bonus_factor.saturating_div(min_reward);
		}

		reward
	}

	pub(super) fn spins(
		spin_times: u8,
		min_spin_reward: u32,
		_jackpot_max_reward: u32,
		_special_max_reward: u32,
		hash: &[u8; 32],
	) -> Option<FullSpin> {
		if !(1..=4).contains(&spin_times) {
			return None;
		}

		let mut spin_results = Vec::with_capacity(spin_times as usize);

		for i in 0..spin_times {
			let offset = (i * 5) as usize;
			let mut spin_result = SpinResult {
				slot_1: CasinoJamUtils::get_slot(hash[offset]),
				slot_2: CasinoJamUtils::get_slot(hash[offset + 1]),
				slot_3: CasinoJamUtils::get_slot(hash[offset + 2]),
				bonus_1: CasinoJamUtils::get_slot(hash[offset + 3]),
				bonus_2: CasinoJamUtils::get_slot(hash[offset + 4]),
				reward: 0,
			};

			spin_result.reward = CasinoJamUtils::single_spin_reward(min_spin_reward, &spin_result);

			spin_results.push(spin_result);
		}

		Some(FullSpin { spin_results, jackpot_reward: 0, special_reward: 0 })
	}

	fn get_slot(hash_byte: u8) -> u8 {
		match hash_byte {
			..52 => 0,
			52..95 => 1,
			95..133 => 2,
			133..167 => 3,
			167..195 => 4,
			195..218 => 5,
			218..235 => 6,
			235..247 => 7,
			247..253 => 8,
			253.. => 9,
		}
	}
}
