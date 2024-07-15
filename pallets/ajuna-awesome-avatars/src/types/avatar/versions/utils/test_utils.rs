use super::*;

pub(crate) fn create_random_avatar<T, F, const N: usize>(
	creator: &T::AccountId,
	initial_dna: Option<[u8; N]>,
	avatar_build_fn: Option<F>,
	encoding: DnaEncoding,
) -> (AvatarIdOf<T>, WrappedAvatar<BlockNumberFor<T>>)
where
	F: FnOnce(AvatarOf<T>) -> WrappedAvatar<BlockNumberFor<T>>,
	T: Config,
{
	let base_avatar = AvatarOf::<T> {
		season_id: 0,
		encoding,
		dna: BoundedVec::try_from(initial_dna.unwrap_or([0_u8; N]).to_vec())
			.expect("Should create DNA!"),
		souls: 0,
		minted_at: 0_u32.into(),
	};

	let avatar = match avatar_build_fn {
		None => WrappedAvatar::new(base_avatar),
		Some(f) => f(base_avatar),
	};
	(Pallet::<T>::random_hash(b"mock_avatar", creator), avatar)
}

pub(crate) fn is_leader_forged<T: Config>(output: &LeaderForgeOutput<T>) -> bool {
	matches!(output, LeaderForgeOutput::Forged(_, _))
}

pub(crate) fn is_leader_consumed<T: Config>(output: &LeaderForgeOutput<T>) -> bool {
	matches!(output, LeaderForgeOutput::Consumed(_))
}

pub(crate) fn is_leader_unchanged<T: Config>(output: &LeaderForgeOutput<T>) -> bool {
	matches!(output, LeaderForgeOutput::Unchanged(_))
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

pub(crate) fn is_unchanged<T: Config>(output: &ForgeOutput<T>) -> bool {
	matches!(output, ForgeOutput::Unchanged(_))
}
