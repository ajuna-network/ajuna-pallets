use crate::{
	mock::{MockAccountId, Test},
	pallet::AvatarIdOf,
	types::{
		avatar::versions::utils::{AvatarBuilder, WrappedAvatar},
		DnaEncoding, ForgeOutput, LeaderForgeOutput, RarityTier, SoulCount,
	},
	AvatarOf, Config, Force, Pallet,
};
use frame_system::pallet_prelude::BlockNumberFor;
use sp_core::bounded::BoundedVec;
use crate::types::avatar::versions::v4::AvatarBuilder;

pub const HASH_BYTES: [u8; 32] = [
	1, 2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79, 83, 89,
	97, 101, 103, 107, 109, 113, 127,
];

pub(crate) fn create_random_avatar<T, F>(
	creator: &T::AccountId,
	initial_dna: Option<[u8; 32]>,
	avatar_build_fn: Option<F>,
) -> (AvatarIdOf<T>, WrappedAvatar<BlockNumberFor<T>>)
where
	F: FnOnce(AvatarOf<T>) -> WrappedAvatar<BlockNumberFor<T>>,
	T: Config,
{
	let base_avatar = AvatarOf::<T> {
		season_id: 0,
		encoding: DnaEncoding::V4,
		dna: BoundedVec::try_from(initial_dna.unwrap_or([0_u8; 32]).to_vec())
			.expect("Should create DNA!"),
		souls: 0,
		minted_at: 0_u32.into(),
	};

	let avatar = match avatar_build_fn {
		None => WrappedAvatar::new(base_avatar),
		Some(f) => f(base_avatar),
	};
	(Pallet::<T>::random_hash(b"mock_avatar_v4", creator), avatar)
}

pub(crate) fn create_random_unprospected_moon(
	account: &MockAccountId,
	stardust_amt: u16,
) -> (AvatarIdOf<Test>, WrappedAvatar<BlockNumberFor<Test>>) {
	crate::types::avatar::versions::v2::create_random_avatar::<Test, _>(
		account,
		None,
		Some(|avatar| {
			AvatarBuilder::with_base_avatar(avatar).into_unprospected_moon(stardust_amt).build_wrapped()
		}),
	)
}

pub(crate) fn is_leader_forged<T: Config>(output: &LeaderForgeOutput<T>) -> bool {
	matches!(output, LeaderForgeOutput::Forged(_, _))
}

pub(crate) fn is_leader_consumed<T: Config>(output: &LeaderForgeOutput<T>) -> bool {
	matches!(output, LeaderForgeOutput::Consumed(_))
}

pub(crate) fn is_forged<T: Config>(output: &ForgeOutput<T>) -> bool {
	matches!(output, ForgeOutput::Forged(_, _))
}

pub(crate) fn is_minted<T: Config>(output: &ForgeOutput<T>) -> bool {
	matches!(output, ForgeOutput::Minted(_))
}

pub(crate) fn is_consumed<T: Config>(output: &ForgeOutput<T>) -> bool {
	matches!(output, ForgeOutput::Consumed(_))
}
