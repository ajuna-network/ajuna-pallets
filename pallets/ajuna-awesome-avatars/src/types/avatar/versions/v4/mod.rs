use super::*;
use sp_runtime::{DispatchError, Saturating};
use sp_std::{marker::PhantomData, vec::Vec};

mod avatar_builder;
mod combinator;
mod constants;
mod dna_interpreters;
mod mutator;
#[cfg(test)]
mod test_utils;
mod types;

use avatar_builder::*;
use combinator::*;
use constants::*;
use dna_interpreters::*;
use mutator::*;
use types::*;

pub(crate) struct AttributeMapperV4;

impl<BlockNumber> AttributeMapper<BlockNumber> for AttributeMapperV4 {
	fn rarity(_target: &Avatar<BlockNumber>) -> u8 {
		todo!()
	}

	fn force(_target: &Avatar<BlockNumber>) -> u8 {
		todo!()
	}
}

pub(crate) struct MinterV4<T: Config>(pub PhantomData<T>);

impl<T: Config> Minter<T> for MinterV4<T> {
	fn mint(
		_player: &T::AccountId,
		_season_id: &SeasonId,
		_mint_option: &MintOption,
	) -> Result<Vec<AvatarIdOf<T>>, DispatchError> {
		todo!()
	}
}

impl<T: Config> MinterV4<T> {
	pub(super) fn generate_empty_dna<const N: usize>() -> Result<Dna, DispatchError> {
		Dna::try_from([0_u8; N].to_vec()).map_err(|_| Error::<T>::IncorrectDna.into())
	}
}

#[derive(Debug, Clone, PartialEq, Ord, PartialOrd, Eq)]
pub(crate) enum ForgeType {
	None,
	ExtractStardust,
	MintTravelPoint,
}

impl ForgeType {
	pub(crate) fn is_restricted(&self) -> bool {
		//matches!(self, ForgeType::ExtractStardust)
		false
	}
}

pub(crate) struct ForgerV4<T: Config>(pub PhantomData<T>);

impl<T: Config> Forger<T> for ForgerV4<T> {
	fn forge(
		player: &T::AccountId,
		season_id: SeasonId,
		season: &SeasonOf<T>,
		input_leader: ForgeItem<T>,
		input_sacrifices: Vec<ForgeItem<T>>,
		restricted: bool,
	) -> Result<(LeaderForgeOutput<T>, Vec<ForgeOutput<T>>), DispatchError> {
		let mut hash_provider =
			HashProvider::<T, 32>::new(&Pallet::<T>::random_hash(b"avatar_forger_v2", player));
		let current_block = <frame_system::Pallet<T>>::block_number();

		ensure!(
			input_sacrifices.len() >= MIN_SACRIFICE && input_sacrifices.len() <= MAX_SACRIFICE,
			Error::<T>::IncompatibleAvatarVersions
		);

		let (leader_id, leader) = input_leader;
		let mut wrapped_leader = WrappedAvatar::new(leader);

		let sacrifices = input_sacrifices
			.into_iter()
			.map(|(id, sacrifice)| (id, WrappedAvatar::new(sacrifice)))
			.collect::<Vec<_>>();
		let wrapped_sacrifices = sacrifices.iter().map(|(_, avatar)| avatar).collect::<Vec<_>>();

		let forge_type =
			Self::determine_forge_type(&mut wrapped_leader, wrapped_sacrifices.as_slice(), current_block);

		ensure!(
			!restricted || !forge_type.is_restricted(),
			Error::<T>::InsufficientStorageForForging
		);

		AvatarCombinator::<T>::combine_avatars_in(
			forge_type,
			season_id,
			season,
			current_block,
			(leader_id, wrapped_leader),
			sacrifices,
			&mut hash_provider,
		)
	}
}

impl<T: Config> ForgerV4<T> {
	fn determine_forge_type(
		leader: &mut WrappedAvatar<BlockNumberFor<T>>,
		sacrifices: &[&WrappedAvatar<BlockNumberFor<T>>],
		current_block: BlockNumberFor<T>,
	) -> ForgeType {
		match leader.get_item_type::<ItemType>() {
			ItemType::Celestial => match leader.get_item_sub_type::<CelestialItemType>() {
				CelestialItemType::UnprospectedMoon => {
					let contains_captain = sacrifices
						.iter()
						.any(|s| s.has_full_type(ItemType::Lifeform, LifeformItemType::Captain));
					let contains_cluster_map = sacrifices
						.iter()
						.any(|s| s.has_full_type(ItemType::Lifeform, LifeformItemType::ClusterMap));
					if sacrifices.len() == 2 && contains_captain && contains_cluster_map {
						ForgeType::ExtractStardust
					} else {
						ForgeType::None
					}
				},
				CelestialItemType::Moon => {
					let minted_at = leader.inner.minted_at;

					let moon_interpreter = MoonInterpreter::from_wrapper(leader);

					let blocks_mint_period = moon_interpreter.get_block_mints_period();
					let block_mint_cooldown = moon_interpreter.get_block_mints_cooldown();
					let minted_travel_points = moon_interpreter.get_minted_travel_points();

					let is_mint_period_over =
						minted_at.saturating_add(blocks_mint_period.into()) <= current_block;
					let is_mint_cooldown_over =
						minted_at.saturating_add(block_mint_cooldown.into()) <= current_block;

					if is_mint_period_over && is_mint_cooldown_over && minted_travel_points > 0 {
						let sacrifice_count = sacrifices.len();

						let contains_captain = sacrifices.iter().any(|s| {
							s.has_full_type(ItemType::Lifeform, LifeformItemType::Captain)
						});
						let contains_cluster_map = sacrifices.iter().any(|s| {
							s.has_full_type(ItemType::Lifeform, LifeformItemType::ClusterMap)
						});

						if sacrifice_count == 2 && contains_captain && contains_cluster_map {
							ForgeType::MintTravelPoint
						} else {
							ForgeType::None
						}
					} else {
						ForgeType::None
					}
				},
				_ => ForgeType::None,
			},
			ItemType::Construction => todo!(),
			ItemType::Lifeform => todo!(),
			ItemType::Resource => todo!(),
			ItemType::Navigation => todo!(),
		}
	}
}
