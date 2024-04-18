#![allow(unused)]
use crate::*;
use sp_runtime::{traits::Zero, DispatchError, Saturating};
use sp_std::{collections::btree_set::BTreeSet, marker::PhantomData, vec::Vec};

pub(crate) struct AttributeMapperV4;

impl<BlockNumber> AttributeMapper<BlockNumber> for AttributeMapperV4 {
	fn rarity(target: &Avatar<BlockNumber>) -> u8 {
		todo!()
	}

	fn force(target: &Avatar<BlockNumber>) -> u8 {
		todo!()
	}
}

pub(crate) struct MinterV4<T: Config>(pub PhantomData<T>);

impl<T: Config> Minter<T> for MinterV4<T> {
	fn mint(
		player: &T::AccountId,
		season_id: &SeasonId,
		mint_option: &MintOption,
	) -> Result<Vec<AvatarIdOf<T>>, DispatchError> {
		todo!()
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
		todo!()
	}
}
