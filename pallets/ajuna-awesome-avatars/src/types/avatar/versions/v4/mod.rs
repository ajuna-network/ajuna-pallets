use super::*;
use sp_runtime::{traits::Zero, DispatchError, Saturating};
use sp_std::{collections::btree_set::BTreeSet, marker::PhantomData, vec::Vec};

mod combinator;
mod constants;
mod mutator;
mod types;

pub use combinator::*;
pub use constants::*;
pub use mutator::*;
pub use types::*;

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

#[derive(Debug, Clone, PartialEq, Ord, PartialOrd, Eq)]
pub(crate) enum ForgeType {
	None,
	ExtractStardust,
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

		ensure!(
			input_sacrifices.len() >= MIN_SACRIFICE && input_sacrifices.len() <= MAX_SACRIFICE,
			Error::<T>::IncompatibleAvatarVersions
		);

		let (leader_id, leader) = input_leader;
		let wrapped_leader = WrappedAvatar::new(leader);

		let sacrifices = input_sacrifices
			.into_iter()
			.map(|(id, sacrifice)| (id, WrappedAvatar::new(sacrifice)))
			.collect::<Vec<_>>();
		let wrapped_sacrifices = sacrifices.iter().map(|(_, avatar)| avatar).collect::<Vec<_>>();

		let forge_type = Self::determine_forge_type(&wrapped_leader, wrapped_sacrifices.as_slice());

		ensure!(
			!restricted || !forge_type.is_restricted(),
			Error::<T>::InsufficientStorageForForging
		);

		AvatarCombinator::<T>::combine_avatars_in(
			forge_type,
			season_id,
			season,
			(leader_id, wrapped_leader),
			sacrifices,
			&mut hash_provider,
		)
	}
}

impl<T: Config> ForgerV4<T> {
	fn determine_forge_type(
		leader: &WrappedAvatar<BlockNumberFor<T>>,
		sacrifices: &[&WrappedAvatar<BlockNumberFor<T>>],
	) -> ForgeType {
		match leader.get_item_type::<ItemType>() {
			ItemType::Celestial => todo!(),
			ItemType::Construction => todo!(),
			ItemType::Lifeform => todo!(),
			ItemType::Resource => todo!(),
			ItemType::Navigation => todo!(),
		}
	}
}
