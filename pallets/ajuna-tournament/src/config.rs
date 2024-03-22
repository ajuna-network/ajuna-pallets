use super::{TournamentId, MAX_PLAYERS};
use frame_support::{
	pallet_prelude::{ConstU32, Decode, Encode, MaxEncodedLen, TypeInfo},
	BoundedVec,
};

pub type Percentage = u8;

pub type RewardDistributionTable = BoundedVec<Percentage, ConstU32<MAX_PLAYERS>>;

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Debug, Default, PartialEq)]
pub enum GoldenDuckConfig {
	#[default]
	Disabled,
	Enabled(Percentage),
}

/// Describes the configuration of a given tournament
#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Debug, PartialEq)]
pub struct TournamentConfig<BlockNumber, Balance> {
	/// Block in which the tournament starts.
	pub start: BlockNumber,
	/// Block in which the tournament finishes.
	pub active_end: BlockNumber,
	/// Block in which the claiming period for tournament rewards end.
	pub claim_end: BlockNumber,
	/// Optional funds that will be transferred by the tournament creator
	/// to the tournament's treasury on success.
	pub initial_reward: Option<Balance>,
	/// Optional cap to the maximum amount of funds to be distributed among the winners.
	pub max_reward: Option<Balance>,
	pub take_fee_percentage: Option<Percentage>,
	/// Distribution table that indicates how the reward should be split among the tournament
	/// winners in the form of [1st %, 2nd %, 3rd %, ....]
	pub reward_distribution: RewardDistributionTable,
	/// Golden duck configuration, either disabled or enabled with a winnings percentage
	pub golden_duck_config: GoldenDuckConfig,
	/// Maximum amount of players that can be ranked in the tournament
	pub max_players: u32,
}

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Debug, PartialEq)]
pub enum TournamentScheduledAction<SeasonId> {
	StartActivePhase(SeasonId, TournamentId),
	SwitchToClaimPhase(SeasonId, TournamentId),
	EndClaimPhase(SeasonId, TournamentId),
}

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Debug, Default, PartialEq)]
pub enum TournamentState<Balance> {
	#[default]
	Inactive,
	ActivePeriod(TournamentId),
	ClaimPeriod(TournamentId, Balance),
}

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Debug, Default, PartialEq)]
pub enum GoldenDuckState<EntityId> {
	#[default]
	Disabled,
	Enabled(Percentage, Option<EntityId>),
}
