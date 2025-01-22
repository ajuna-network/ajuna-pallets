use crate::seasons::{HeroJamSeasonData, HeroJamSeasonId};

use ajuna_primitives::season_manager::SeasonsBenchmarkHelper;

pub struct GameSeasonsBenchmarkHelper;

impl SeasonsBenchmarkHelper<HeroJamSeasonId, HeroJamSeasonData> for GameSeasonsBenchmarkHelper {
	fn create_season_id() -> HeroJamSeasonId {
		HeroJamSeasonId::default()
	}

	fn create_season_data() -> HeroJamSeasonData {}
}
