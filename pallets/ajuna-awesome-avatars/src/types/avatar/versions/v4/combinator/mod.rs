mod extract_stardust;
mod engage_in_event;
//mod start_harvesting_temp_nebula;
//mod start_haversting_moon;
mod upgrade_ship;
mod mint_travel_point;

#[cfg(test)]
use super::test_utils::*;

use super::{utils::*, *};

pub(super) struct AvatarCombinator<T: Config>(pub PhantomData<T>);

impl<T: Config> AvatarCombinator<T> {
	pub(super) fn combine_avatars_in(
		forge_type: ForgeType,
		_season_id: SeasonId,
		_season: &SeasonOf<T>,
		main: WrappedForgeItem<T>,
		mut sub_components: Vec<WrappedForgeItem<T>>,
		_hash_provider: &mut HashProvider<T, 32>,
	) -> Result<(LeaderForgeOutput<T>, Vec<ForgeOutput<T>>), DispatchError> {
		match forge_type {
			ForgeType::ExtractStardust => {
				// If we are in ExtractStardust that means all inputs should have already been
				// validated, so we can split the sub_components without danger
				let (captain, cluster_map) = {
					let first = sub_components.pop().expect("Should contain entry");
					let second = sub_components.pop().expect("Should contain entry");

					if first.1.has_full_type(ItemType::Lifeform, LifeformItemType::Captain) {
						(first, second)
					} else {
						(second, first)
					}
				};

				Self::extract_stardust(main, captain, cluster_map)
			},
			ForgeType::MintTravelPoint => {
				// If we are in MintTravelPoint that means all inputs should have already been
				// validated, so we can split the sub_components without danger
				let (captain, cluster_map) = {
					let first = sub_components.pop().expect("Should contain entry");
					let second = sub_components.pop().expect("Should contain entry");

					if first.1.has_full_type(ItemType::Lifeform, LifeformItemType::Captain) {
						(first, second)
					} else {
						(second, first)
					}
				};

				Self::mint_travel_point(main, captain, cluster_map)
			},
			ForgeType::None => Self::forge_none(main, sub_components),
		}
	}

	fn forge_none(
		input_leader: WrappedForgeItem<T>,
		input_sacrifices: Vec<WrappedForgeItem<T>>,
	) -> Result<(LeaderForgeOutput<T>, Vec<ForgeOutput<T>>), DispatchError> {
		Ok((
			LeaderForgeOutput::Unchanged((input_leader.0, input_leader.1.unwrap())),
			input_sacrifices
				.into_iter()
				.map(|sacrifice| ForgeOutput::Unchanged((sacrifice.0, sacrifice.1.unwrap())))
				.collect(),
		))
	}
}
