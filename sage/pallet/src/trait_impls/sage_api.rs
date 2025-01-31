use crate::{Config, Pallet, TransitionConfigOf, TransitionConfigStore};
use ajuna_primitives::sage_api::SageApi;

impl<T: Config<I>, I: 'static> SageApi for Pallet<T, I> {
	type TransitionConfig = TransitionConfigOf<T, I>;

	fn get_transition_config() -> Self::TransitionConfig {
		TransitionConfigStore::<T, I>::get()
	}
}
