use crate::{
	asset::{
		hero_jam::{AssetSubType, AssetType, HeroJamAsset, StateType},
		Asset, AssetId,
		AssetVariant::HeroJam,
	},
	transition::{hero_jam::HeroAction, TransitionIdentifier},
};

use sage_api::benchmarks::SageBenchmarkHelper;

use ajuna_primitives::payment_handler::WithdrawKind;
use frame_support::traits::fungible::NativeOrWithId;
use sp_runtime::{traits::BlockNumber as BlockNumberT, SaturatedConversion};
use sp_std::{marker::PhantomData, vec::Vec};

pub struct GameBenchmarkHelper<BlockNumber>(PhantomData<BlockNumber>);

impl<BlockNumber>
	SageBenchmarkHelper<
		AssetId,
		Asset<BlockNumber>,
		TransitionIdentifier,
		AssetType,
		AssetType,
		WithdrawKind<NativeOrWithId<AssetId>>,
	> for GameBenchmarkHelper<BlockNumber>
where
	BlockNumber: BlockNumberT,
{
	fn create_asset(seed: u32) -> (AssetId, Asset<BlockNumber>) {
		let asset_id = AssetId::from(seed);
		let asset = Asset {
			asset_variant: HeroJam(HeroJamAsset {
				id: asset_id,
				asset_type: AssetType::Hero,
				asset_subtype: AssetSubType::None,
				energy: 0,
				fatigue: 0,
				state_type: StateType::None,
				state_sub_type: 0,
				state_sub_value: 0,
				state_change_block_number: 0_u32.saturated_into(),
				balance: 10,
			}),
		};

		(asset_id, asset)
	}

	fn create_bench_transition() -> (TransitionIdentifier, Vec<AssetId>) {
		(TransitionIdentifier::HeroJam(HeroAction::Create), sp_std::vec::Vec::with_capacity(0))
	}

	fn create_trade_filter_for(asset: &Asset<BlockNumber>) -> AssetType {
		match &asset.asset_variant {
			HeroJam(hero_jam) => hero_jam.asset_type,
		}
	}

	fn create_transfer_filter_for(asset: &Asset<BlockNumber>) -> AssetType {
		match &asset.asset_variant {
			HeroJam(hero_jam) => hero_jam.asset_type,
		}
	}

	fn create_payment_kind() -> WithdrawKind<NativeOrWithId<AssetId>> {
		WithdrawKind::Payment(NativeOrWithId::Native)
	}
}
