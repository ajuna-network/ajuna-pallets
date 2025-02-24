#![cfg_attr(not(feature = "std"), no_std)]

pub mod asset;
mod benchmarks;
pub mod error;
pub mod filter;
mod rules;
pub mod transition;

/// This contains all modules required for the runtime integration
/// of the gameplay logic into a SAGE instance.
pub mod prelude {
	pub use crate::{
		asset::{Asset, AssetId, AssetVariant, MachineSubVariant, MachineVariant, VariantType},
		benchmarks::GameBenchmarkHelper,
		error,
		filter::GameFilter,
		transition::{
			AssetType, CasinoAction, CasinoJamTransition, CasinoJamTransitionConfig, MachineType,
			MultiplierType, PlayerType, RentDuration, ReservationDuration, TokenType,
		},
	};
}
