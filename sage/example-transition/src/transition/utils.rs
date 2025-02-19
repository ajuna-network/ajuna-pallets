use sp_std::vec::Vec;

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
	pub(super) fn get_packed(&self) -> u16 {
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
	) -> u16 {
		let mut result: u16 = 0;

		result |= (slot_1 as u16 & 0x0F) << 12; // Bits 15-12
		result |= (slot_2 as u16 & 0x0F) << 8; // Bits 11-8
		result |= (slot_3 as u16 & 0x0F) << 4; // Bits 7-4
		result |= (bonus_1 as u16 & 0x03) << 2; // Bits 3-2
		result |= bonus_2 as u16 & 0x03; // Bits 1-0

		result
	}

	fn single_spin_reward(min_reward: u32, spin: &SpinResult) -> u32 {
		let factor_multiplier = match (spin.slot_1, spin.slot_2, spin.slot_3) {
			(0, 0, 0) => 2,
			(1, 1, 1) => 4,
			(2, 2, 2) => 8,
			(3, 3, 3) => 16,
			(4, 4, 4) => 1,
			(5, 5, 5) => 32,
			(6, 6, 6) => 64,
			(7, 7, 7) => 128,
			(8, 8, 8) => 256,
			(9, 9, 9) => 512,
			_ => 0,
		};
		let spin_factor = min_reward.saturating_mul(factor_multiplier);

		let bonus_factor = match (spin.bonus_1, spin.bonus_2) {
			(4, 4) => 1,
			(5, 5) => 2,
			(6, 6) => 4,
			_ => 0,
		};

		let is_full_line = spin.slot_1 == spin.slot_2 &&
			spin.slot_2 == spin.slot_3 &&
			spin.slot_3 == spin.bonus_1 &&
			spin.bonus_1 == spin.bonus_2;

		let mut reward = spin_factor;

		if is_full_line {
			reward = spin_factor.saturating_mul(128_u32.saturating_add(bonus_factor));
		} else if spin_factor > 0 && bonus_factor > 0 {
			reward = spin_factor.saturating_add(512_u32.saturating_mul(bonus_factor));
		}

		if reward == 0 {
			if let (4, 4) = (spin.bonus_1, spin.bonus_2) {
				reward = 1;
			}
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
				slot_1: hash[offset] % 8,
				slot_2: hash[offset + 1] % 8,
				slot_3: hash[offset + 2] % 8,
				bonus_1: hash[offset + 3] % 8,
				bonus_2: hash[offset + 4] % 8,
				reward: 0,
			};

			spin_result.reward = CasinoJamUtils::single_spin_reward(min_spin_reward, &spin_result);

			spin_results.push(spin_result);
		}

		Some(FullSpin { spin_results, jackpot_reward: 0, special_reward: 0 })
	}
}
