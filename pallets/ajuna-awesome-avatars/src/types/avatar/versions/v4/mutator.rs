use super::*;
use crate::types::Avatar;

pub(crate) trait AvatarMutator<T: Config> {
	fn mutate_from_base(
		&self,
		base_avatar: AvatarOf<T>,
		hash_provider: &mut HashProvider<T, 32>,
	) -> Result<AvatarOf<T>, ()>;
}
