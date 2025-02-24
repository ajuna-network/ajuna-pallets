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

mod create;
mod deposit;
mod gamble;
mod kick;
mod release;
mod rent;
mod reserve;
mod r#return;
mod withdraw;

use super::*;

use ajuna_primitives::asset_manager::AssetInspector;
use example_transition::prelude::*;

pub(crate) fn create_player_and_tracker_for(account_id: MockAccountId) {
	let transition_id = CasinoAction::Create(AssetType::Player);
	assert_ok!(Sage::state_transition(
		RuntimeOrigin::signed(account_id),
		transition_id,
		vec![],
		(),
		SOME_NATIVE_PAYMENT
	));
	System::assert_last_event(RuntimeEvent::Sage(Event::TransitionExecuted {
		account: account_id,
		id: transition_id,
	}));
}

pub(crate) fn create_machine_for(account_id: MockAccountId) {
	let transition_id = CasinoAction::Create(AssetType::Machine(MachineType::Bandit));
	assert_ok!(Sage::state_transition(
		RuntimeOrigin::signed(account_id),
		transition_id,
		vec![],
		(),
		SOME_NATIVE_PAYMENT
	));
	System::assert_last_event(RuntimeEvent::Sage(Event::TransitionExecuted {
		account: account_id,
		id: transition_id,
	}));
}

pub(crate) fn get_assets_from(
	account_id: MockAccountId,
	variant_type: VariantType,
) -> Vec<(AssetId, Asset<BlockNumberFor<Test>>)> {
	Sage::iter_assets_from(&account_id)
		.filter(|(_, asset)| asset.variant.is_variant(variant_type))
		.collect()
}

pub(crate) fn run_to_block(n: u64) {
	while System::block_number() < n {
		if System::block_number() > 1 {
			Randomness::on_finalize(System::block_number());
			Sage::on_finalize(System::block_number());
			System::on_finalize(System::block_number());
		}
		System::set_block_number(System::block_number() + 1);
		System::on_initialize(System::block_number());
		Sage::on_initialize(System::block_number());
		Randomness::on_initialize(System::block_number());
	}
}
