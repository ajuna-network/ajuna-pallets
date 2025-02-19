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
