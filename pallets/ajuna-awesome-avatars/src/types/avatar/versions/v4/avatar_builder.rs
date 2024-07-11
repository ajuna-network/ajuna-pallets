use super::*;

#[derive(Default)]
pub(crate) struct AvatarBuilder<BlockNumber> {
	inner: Avatar<BlockNumber>,
}

impl<BlockNumber> AvatarBuilder<BlockNumber>
where
	BlockNumber: sp_runtime::traits::BlockNumber,
{
	pub fn with_dna(season_id: SeasonId, dna: Dna, minted_at: BlockNumber) -> Self {
		Self { inner: Avatar { season_id, encoding: DnaEncoding::V4, dna, souls: 0, minted_at } }
	}

	pub fn with_base_avatar(avatar: Avatar<BlockNumber>) -> Self {
		Self { inner: avatar }
	}

	pub fn with_attribute<T>(self, attribute: AvatarAttr, value: &T) -> Self
	where
		T: ByteConvertible,
	{
		self.with_attribute_raw(attribute, value.as_byte())
	}

	pub fn with_attribute_raw(mut self, attribute: AvatarAttr, value: u8) -> Self {
		DnaUtils::<BlockNumber>::write_attribute_raw(&mut self.inner, attribute, value);
		self
	}

	pub fn with_soul_count(mut self, soul_count: SoulCount) -> Self {
		self.inner.souls = soul_count;
		self
	}

	pub fn structured_into_unprospected_moon(
		mut self,
		stardust_amt: u8,
		coord_x: u32,
		coord_y: u32,
	) -> Self {
		self = self
			.with_attribute(AvatarAttr::ItemType, &ItemType::Celestial)
			.with_attribute(AvatarAttr::ItemSubType, &CelestialItemType::UnprospectedMoon);

		let mut wrapped = WrappedAvatar::new(self.inner);
		let mut interpreter = UnprospectedMoonInterpreter::from_wrapper(&mut wrapped);

		interpreter.set_stardust_on_moon(stardust_amt);
		interpreter.set_coord(Coord::X, coord_x);
		interpreter.set_coord(Coord::Y, coord_y);
		// Add more logic here in the future

		self.inner = wrapped.unwrap();
		self
	}

	pub fn structured_into_moon(mut self, coord_x: u32, coord_y: u32) -> Self {
		self = self
			.with_attribute(AvatarAttr::ItemType, &ItemType::Celestial)
			.with_attribute(AvatarAttr::ItemSubType, &CelestialItemType::Moon);

		let mut wrapped = WrappedAvatar::new(self.inner);
		let mut interpreter = MoonInterpreter::from_wrapper(&mut wrapped);

		interpreter.set_coord(Coord::X, coord_x);
		interpreter.set_coord(Coord::Y, coord_y);
		// Add more logic here in the future

		self.inner = wrapped.unwrap();
		self
	}

	pub fn structured_into_ship(mut self, ship_type: u8, coord_x: u32, coord_y: u32) -> Self {
		self = self
			.with_attribute(AvatarAttr::ItemType, &ItemType::Construction)
			.with_attribute(AvatarAttr::ItemSubType, &ConstructionItemType::Ship)
			.with_attribute(AvatarAttr::ClassType1, &ship_type);

		let mut wrapped = WrappedAvatar::new(self.inner);
		let mut interpreter = ShipInterpreter::from_wrapper(&mut wrapped);

		interpreter.set_coord(Coord::X, coord_x);
		interpreter.set_coord(Coord::Y, coord_y);
		// Add more logic here in the future

		self.inner = wrapped.unwrap();
		self
	}

	pub fn structured_into_resource(mut self, resource_type: u8) -> Self {
		self = self
			.with_attribute(AvatarAttr::ItemType, &ItemType::Resource)
			.with_attribute(AvatarAttr::ItemSubType, &ResourceItemType::Resource)
			.with_attribute(AvatarAttr::ClassType1, &resource_type);

		let mut wrapped = WrappedAvatar::new(self.inner);
		let mut interpreter = ResourceInterpreter::from_wrapper(&mut wrapped);

		interpreter.set_resource_type(resource_type);

		self.inner = wrapped.unwrap();
		self
	}

	pub fn structured_into_captain(mut self, captain_type: u8, leaderboard_points: u16) -> Self {
		self = self
			.with_attribute(AvatarAttr::ItemType, &ItemType::Lifeform)
			.with_attribute(AvatarAttr::ItemSubType, &LifeformItemType::Captain)
			.with_attribute(AvatarAttr::ClassType1, &captain_type);

		let mut wrapped = WrappedAvatar::new(self.inner);
		let mut interpreter = CaptainInterpreter::from_wrapper(&mut wrapped);

		interpreter.set_leaderboard_points(leaderboard_points);
		// Add more logic here in the future

		self.inner = wrapped.unwrap();
		self
	}

	pub fn structured_into_cyoa(mut self, answers: &[(u8, u8)]) -> Self {
		self = self
			.with_attribute(AvatarAttr::ItemType, &ItemType::Lifeform)
			.with_attribute(AvatarAttr::ItemSubType, &LifeformItemType::Cyoa);

		let mut wrapped = WrappedAvatar::new(self.inner);
		let mut interpreter = CyoaInterpreter::from_wrapper(&mut wrapped);

		for (question_idx, value) in answers {
			interpreter.set_answer_question(*question_idx, *value);
		}

		self.inner = wrapped.unwrap();
		self
	}

	pub fn structured_into_cluster_map(
		mut self,
		main_cluster: &[(u8, u8)],
		cluster: &[(u8, u8)],
		coord_x: u32,
		coord_y: u32,
	) -> Self {
		self = self
			.with_attribute(AvatarAttr::ItemType, &ItemType::Lifeform)
			.with_attribute(AvatarAttr::ItemSubType, &LifeformItemType::ClusterMap);

		let mut wrapped = WrappedAvatar::new(self.inner);
		let mut interpreter = ClusterMapInterpreter::from_wrapper(&mut wrapped);

		for (main_cluster_idx, value) in main_cluster {
			interpreter.set_main_cluster(*main_cluster_idx, *value);
		}

		for (cluster_idx, value) in cluster {
			interpreter.set_cluster(*cluster_idx, *value);
		}

		interpreter.set_coord(Coord::X, coord_x);
		interpreter.set_coord(Coord::Y, coord_y);

		self.inner = wrapped.unwrap();
		self
	}

	pub fn structured_into_travel_point(mut self, coord_x: u32, coord_y: u32) -> Self {
		self = self
			.with_attribute(AvatarAttr::ItemType, &ItemType::Celestial)
			.with_attribute(AvatarAttr::ItemSubType, &CelestialItemType::TravelPoint);

		let mut wrapped = WrappedAvatar::new(self.inner);
		let mut interpreter = TravelPointInterpreter::from_wrapper(&mut wrapped);

		interpreter.set_coord(Coord::X, coord_x);
		interpreter.set_coord(Coord::Y, coord_y);

		self.inner = wrapped.unwrap();
		self
	}

	pub fn structured_into_nebula(mut self, resource_type: u8, coord_x: u32, coord_y: u32) -> Self {
		self = self
			.with_attribute(AvatarAttr::ItemType, &ItemType::Celestial)
			.with_attribute(AvatarAttr::ItemSubType, &CelestialItemType::Nebula);

		let mut wrapped = WrappedAvatar::new(self.inner);
		let mut interpreter = NebulaInterpreter::from_wrapper(&mut wrapped);

		interpreter.set_resource_type(resource_type);
		interpreter.set_coord(Coord::X, coord_x);
		interpreter.set_coord(Coord::Y, coord_y);

		self.inner = wrapped.unwrap();
		self
	}

	pub fn structured_into_temp_nebula(
		mut self,
		resource_type: u8,
		resource_amt: u8,
		coord_x: u32,
		coord_y: u32,
	) -> Self {
		self = self
			.with_attribute(AvatarAttr::ItemType, &ItemType::Celestial)
			.with_attribute(AvatarAttr::ItemSubType, &CelestialItemType::TempNebula);

		let mut wrapped = WrappedAvatar::new(self.inner);
		let mut interpreter = TempNebulaInterpreter::from_wrapper(&mut wrapped);

		interpreter.set_resource_type(resource_type);
		interpreter.set_resource_amount(resource_amt);
		interpreter.set_coord(Coord::X, coord_x);
		interpreter.set_coord(Coord::Y, coord_y);

		self.inner = wrapped.unwrap();
		self
	}

	pub fn build(self) -> Avatar<BlockNumber> {
		self.inner
	}

	pub fn build_wrapped(self) -> WrappedAvatar<BlockNumber> {
		WrappedAvatar::new(self.inner)
	}
}
