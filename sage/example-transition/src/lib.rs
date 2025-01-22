#![cfg_attr(not(feature = "std"), no_std)]

pub mod asset;
mod benchmarks;
pub mod filter;
mod rules;
mod seasons;
mod tournament;
pub mod transition;

/// This contains all modules required for the runtime integration
/// of the gameplay logic into a SAGE instance.
pub mod prelude {
	pub use crate::{
		asset::{
			hero_jam::{AssetSubType, AssetType, HeroJamAsset, StateType},
			Asset, AssetId, AssetVariant,
		},
		benchmarks::sage::GameBenchmarkHelper,
		filter::GameFilter,
		seasons::{HeroJamSeasonData, HeroJamSeasonId},
		transition::{GameTransition, TransitionIdentifier},
	};
}
