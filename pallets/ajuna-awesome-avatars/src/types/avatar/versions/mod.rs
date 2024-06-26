mod v1;
mod v2;
mod v3;
mod v4;

pub(crate) use v1::{AttributeMapperV1, ForgerV1, MinterV1};
pub(crate) use v2::{AttributeMapperV2, ForgerV2, MinterV2};
pub(crate) use v3::{AttributeMapperV3, ForgerV3, MinterV3};
pub(crate) use v4::{AttributeMapperV4, ForgerV4, MinterV4};

use crate::*;
use frame_support::pallet_prelude::*;
use sp_std::vec::Vec;

pub(crate) trait AttributeMapper<BlockNumber> {
	/// Used to obtain the RarityTier of a given avatar as an u8.
	fn rarity(target: &Avatar<BlockNumber>) -> u8;

	/// Used to get the Force of a given avatar as an u8.
	fn force(target: &Avatar<BlockNumber>) -> u8;
}

pub(crate) trait Minter<T: Config> {
	fn mint(
		player: &T::AccountId,
		season_id: &SeasonId,
		mint_option: &MintOption,
	) -> Result<Vec<AvatarIdOf<T>>, DispatchError>;
}

/// A tuple containing and avatar identifier with its represented avatar, used as forging inputs.
pub(crate) type ForgeItem<T> = (AvatarIdOf<T>, AvatarOf<T>);
/// Number of components upgraded after a forge in a given Avatar.
pub(crate) type UpgradedComponents = u8;

/// Enum used to express the possible results of the forge on the leader avatar.
pub(crate) enum LeaderForgeOutput<T: Config> {
	/// The leader avatar was forged (mutated) in some way.
	Forged(ForgeItem<T>, UpgradedComponents),
	/// The leader avatar was consumed in the forging process.
	Consumed(AvatarIdOf<T>),
	/// The leader avatar was left unchanged.
	Unchanged(ForgeItem<T>),
}
/// Enum used to express the possible results of the forge on the other avatars, also called
/// sacrifices.
pub(crate) enum ForgeOutput<T: Config> {
	/// The avatar was forged (mutate) in some way.
	Forged(ForgeItem<T>, UpgradedComponents),
	/// A new avatar was created from the forging process.
	Minted(AvatarOf<T>),
	/// The avatar was consumed in the forging process.
	Consumed(AvatarIdOf<T>),
	/// The avatar was not changed in the forging process.
	Unchanged(ForgeItem<T>),
}

/// Trait used to define the surface logic of the forging algorithm.
pub(crate) trait Forger<T: Config> {
	/// Tries to use the supplied inputs and forge them.
	fn forge(
		player: &T::AccountId,
		season_id: SeasonId,
		season: &SeasonOf<T>,
		input_leader: ForgeItem<T>,
		input_sacrifices: Vec<ForgeItem<T>>,
		restricted: bool,
	) -> Result<(LeaderForgeOutput<T>, Vec<ForgeOutput<T>>), DispatchError>;
}
