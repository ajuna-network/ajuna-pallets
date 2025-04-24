// Ajuna Node
// Copyright (C) 2022 BlogaTech AG
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use crate::{
	asset::{Asset, AssetId, AssetVariant, PlayerVariant, VariantType},
	transition::{AssetType, CasinoAction},
};

use ajuna_primitives::payment_handler::WithdrawKind;
use sage_api::benchmarks::SageBenchmarkHelper;

use crate::{prelude::MachineType, transition::PlayerType};
use frame_support::traits::fungible::NativeOrWithId;
use sp_runtime::traits::BlockNumber as BlockNumberT;
use sp_std::{marker::PhantomData, vec::Vec};

pub struct GameBenchmarkHelper<BlockNumber>(PhantomData<BlockNumber>);

impl<BlockNumber>
	SageBenchmarkHelper<
		AssetId,
		Asset<BlockNumber>,
		CasinoAction,
		VariantType,
		VariantType,
		WithdrawKind<NativeOrWithId<AssetId>>,
	> for GameBenchmarkHelper<BlockNumber>
where
	BlockNumber: BlockNumberT,
{
	fn create_asset(seed: u32) -> (AssetId, Asset<BlockNumber>) {
		let asset_id = AssetId::from(seed);
		let asset = Asset::<BlockNumber>::new_player(asset_id, 0_u32.into());

		(asset_id, asset)
	}

	fn create_bench_transition() -> (CasinoAction, Vec<AssetId>) {
		(CasinoAction::Create(AssetType::Player), Vec::with_capacity(0))
	}

	fn create_trade_filter_for(asset: &Asset<BlockNumber>) -> VariantType {
		match asset.variant {
			AssetVariant::Player(player_type) => match player_type {
				PlayerVariant::Human(_) => VariantType::Player(PlayerType::Human),
				PlayerVariant::Tracker(_) => VariantType::Player(PlayerType::Tracker),
			},
			AssetVariant::Machine(_) => VariantType::Machine(MachineType::Bandit),
			AssetVariant::Seat(_) => VariantType::Seat,
		}
	}

	fn create_transfer_filter_for(asset: &Asset<BlockNumber>) -> VariantType {
		match asset.variant {
			AssetVariant::Player(player_type) => match player_type {
				PlayerVariant::Human(_) => VariantType::Player(PlayerType::Human),
				PlayerVariant::Tracker(_) => VariantType::Player(PlayerType::Tracker),
			},
			AssetVariant::Machine(_) => VariantType::Machine(MachineType::Bandit),
			AssetVariant::Seat(_) => VariantType::Seat,
		}
	}

	fn create_payment_kind() -> WithdrawKind<NativeOrWithId<AssetId>> {
		WithdrawKind::Payment(NativeOrWithId::Native)
	}
}
