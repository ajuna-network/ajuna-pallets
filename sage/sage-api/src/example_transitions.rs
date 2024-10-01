//! Some example transitions.
//!
//! These should be expanded to really showcase the power of the SageApi design.

use crate::{primitives, sage::SageApi};
use frame_support::ensure;

/// Verifies a transition rule with a given transition id.
pub fn verify_transition_rule<Sage: SageApi>(
	transition_id: u32,
	assets: &[Sage::Asset],
) -> Result<(), primitives::Error> {
	match transition_id {
		1 => rule_asset_length_1::<Sage>(assets),
		_ => Err(primitives::Error::InvalidTransitionId),
	}
}

/// Executes a transition with a given transition id.
pub fn transition<Sage: SageApi>(
	transition_id: u32,
	assets: Vec<Sage::Asset>,
) -> Result<(), primitives::Error> {
	match transition_id {
		1 => transition_one::<Sage>(assets),
		_ => Err(primitives::Error::InvalidTransitionId),
	}
}

/// One specific transition that a game wants to execute.
pub fn transition_one<Sage: SageApi>(_assets: Vec<Sage::Asset>) -> Result<(), primitives::Error> {
	todo!()
}

/// A rule that maybe many different transitions want to fulfill.
pub fn rule_asset_length_1<Sage: SageApi>(assets: &[Sage::Asset]) -> Result<(), primitives::Error> {
	ensure!(assets.len() == 1, primitives::Error::InvalidAssetLength);
	Ok(())
}
