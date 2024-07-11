use crate::{
	mock::{MockAccountId, Test},
	pallet::AvatarIdOf,
	types::{
		avatar::versions::{utils::*, v4::AvatarBuilder},
		DnaEncoding, ForgeOutput, LeaderForgeOutput, RarityTier, SoulCount,
	},
	AvatarOf, Config, Force, Pallet,
};
use frame_system::pallet_prelude::BlockNumberFor;
use sp_core::bounded::BoundedVec;

pub const HASH_BYTES: [u8; 32] = [
	1, 2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79, 83, 89,
	97, 101, 103, 107, 109, 113, 127,
];

pub(crate) fn create_random_unprospected_moon(
	account: &MockAccountId,
	stardust_amt: u8,
	coordinates: (u32, u32),
) -> (AvatarIdOf<Test>, WrappedAvatar<BlockNumberFor<Test>>) {
	create_random_avatar::<Test, _, 35>(
		account,
		None,
		Some(|avatar| {
			AvatarBuilder::with_base_avatar(avatar)
				.structured_into_unprospected_moon(stardust_amt, coordinates.0, coordinates.1)
				.build_wrapped()
		}),
		DnaEncoding::V4,
	)
}

pub(crate) fn create_random_captain(
	account: &MockAccountId,
	captain_type: u8,
	leaderboard_points: u16,
) -> (AvatarIdOf<Test>, WrappedAvatar<BlockNumberFor<Test>>) {
	create_random_avatar::<Test, _, 35>(
		account,
		None,
		Some(|avatar| {
			AvatarBuilder::with_base_avatar(avatar)
				.structured_into_captain(captain_type, leaderboard_points)
				.build_wrapped()
		}),
		DnaEncoding::V4,
	)
}

pub(crate) fn create_random_cluster_map(
	account: &MockAccountId,
	main_cluster: &[(u8, u8)],
	cluster: &[(u8, u8)],
	coordinates: (u32, u32),
) -> (AvatarIdOf<Test>, WrappedAvatar<BlockNumberFor<Test>>) {
	create_random_avatar::<Test, _, 35>(
		account,
		None,
		Some(|avatar| {
			AvatarBuilder::with_base_avatar(avatar)
				.structured_into_cluster_map(main_cluster, cluster, coordinates.0, coordinates.1)
				.build_wrapped()
		}),
		DnaEncoding::V4,
	)
}
