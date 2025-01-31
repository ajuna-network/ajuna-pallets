pub trait SageApi {
	type TransitionConfig;

	fn get_transition_config() -> Self::TransitionConfig;
}
