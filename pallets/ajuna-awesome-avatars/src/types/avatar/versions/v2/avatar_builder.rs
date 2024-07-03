use crate::types::{
	avatar::versions::{
		utils::*,
		v2::{
			BlueprintItemType, ColorType, EquippableItemType, EssenceItemType, ItemType,
			MaterialItemType, PetItemType, PetType, SlotType, SpecialItemType, GLIMMER_SP,
			PROGRESS_PROBABILITY_PERC, SCALING_FACTOR_PERC,
		},
	},
	*,
};

#[derive(Default)]
pub(crate) struct AvatarBuilder<BlockNumber> {
	inner: Avatar<BlockNumber>,
}

impl<BlockNumber> AvatarBuilder<BlockNumber>
where
	BlockNumber: sp_runtime::traits::BlockNumber,
{
	pub fn with_dna(season_id: SeasonId, dna: Dna, minted_at: BlockNumber) -> Self {
		Self { inner: Avatar { season_id, encoding: DnaEncoding::V2, dna, souls: 0, minted_at } }
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

	pub fn with_spec_byte_raw(mut self, spec_byte: SpecIdx, value: u8) -> Self {
		DnaUtils::<BlockNumber>::write_spec(&mut self.inner, spec_byte, value);
		self
	}

	pub fn with_spec_bytes(mut self, spec_bytes: [u8; 16]) -> Self {
		DnaUtils::<BlockNumber>::write_specs(&mut self.inner, spec_bytes);
		self
	}

	pub fn with_soul_count(mut self, soul_count: SoulCount) -> Self {
		self.inner.souls = soul_count;
		self
	}

	pub fn with_progress_array(mut self, progress_array: [u8; 11]) -> Self {
		DnaUtils::<BlockNumber>::write_progress(&mut self.inner, progress_array);
		self
	}

	pub fn into_pet(
		self,
		pet_type: &PetType,
		pet_variation: u8,
		spec_bytes: [u8; 16],
		progress_array: [u8; 11],
		soul_points: SoulCount,
	) -> Self {
		self.with_attribute(AvatarAttr::ItemType, &ItemType::Pet)
			.with_attribute(AvatarAttr::ItemSubType, &PetItemType::Pet)
			.with_attribute(AvatarAttr::ClassType1, &HexType::X0)
			.with_attribute(AvatarAttr::ClassType2, pet_type)
			.with_attribute(AvatarAttr::CustomType1, &HexType::X0)
			.with_attribute(AvatarAttr::RarityTier, &RarityTier::Legendary)
			.with_attribute_raw(AvatarAttr::Quantity, 1)
			.with_attribute_raw(AvatarAttr::CustomType2, pet_variation)
			.with_spec_bytes(spec_bytes)
			.with_progress_array(progress_array)
			.with_soul_count(soul_points)
	}

	pub fn into_generic_pet_part(self, slot_types: &[SlotType], quantity: u8) -> Self {
		let custom_type_1 = HexType::X1;

		let spec_bytes = {
			let mut spec_bytes = DnaUtils::<BlockNumber>::read_specs(&self.inner);

			for slot_index in slot_types.iter().map(|slot_type| slot_type.as_byte() as usize) {
				spec_bytes[slot_index] = spec_bytes[slot_index].saturating_add(1);
			}

			spec_bytes
		};

		self.with_attribute(AvatarAttr::ItemType, &ItemType::Pet)
			.with_attribute(AvatarAttr::ItemSubType, &PetItemType::PetPart)
			.with_attribute(AvatarAttr::ClassType1, &HexType::X0)
			.with_attribute(AvatarAttr::ClassType2, &HexType::X0)
			.with_attribute(AvatarAttr::CustomType1, &custom_type_1)
			.with_attribute(AvatarAttr::RarityTier, &RarityTier::Uncommon)
			.with_attribute_raw(AvatarAttr::Quantity, quantity)
			.with_attribute(AvatarAttr::CustomType2, &HexType::X0)
			.with_spec_bytes(spec_bytes)
			.with_soul_count(quantity as SoulCount * custom_type_1 as SoulCount)
	}

	#[cfg(test)]
	pub fn into_pet_part(self, pet_type: &PetType, slot_type: &SlotType, quantity: u8) -> Self {
		let custom_type_1 = HexType::X1;

		let base_seed = pet_type.as_byte() as usize + slot_type.as_byte() as usize;
		let base_0 = DnaUtils::<BlockNumber>::create_pattern::<NibbleType>(
			base_seed,
			EquippableItemType::ArmorBase.as_byte() as usize,
		);
		let comp_1 = DnaUtils::<BlockNumber>::create_pattern::<NibbleType>(
			base_seed,
			EquippableItemType::ArmorComponent1.as_byte() as usize,
		);
		let comp_2 = DnaUtils::<BlockNumber>::create_pattern::<NibbleType>(
			base_seed,
			EquippableItemType::ArmorComponent2.as_byte() as usize,
		);
		let comp_3 = DnaUtils::<BlockNumber>::create_pattern::<NibbleType>(
			base_seed,
			EquippableItemType::ArmorComponent3.as_byte() as usize,
		);

		self.with_attribute(AvatarAttr::ItemType, &ItemType::Pet)
			.with_attribute(AvatarAttr::ItemSubType, &PetItemType::PetPart)
			.with_attribute(AvatarAttr::ClassType1, slot_type)
			.with_attribute(AvatarAttr::ClassType2, pet_type)
			.with_attribute(AvatarAttr::CustomType1, &custom_type_1)
			.with_attribute(AvatarAttr::RarityTier, &RarityTier::Uncommon)
			.with_attribute_raw(AvatarAttr::Quantity, quantity)
			// Unused
			.with_attribute(AvatarAttr::CustomType2, &HexType::X0)
			.with_spec_byte_raw(
				SpecIdx::Byte1,
				DnaUtils::<BlockNumber>::enums_to_bits(&base_0) as u8,
			)
			.with_spec_byte_raw(
				SpecIdx::Byte2,
				DnaUtils::<BlockNumber>::enums_order_to_bits(&base_0) as u8,
			)
			.with_spec_byte_raw(
				SpecIdx::Byte3,
				DnaUtils::<BlockNumber>::enums_to_bits(&comp_1) as u8,
			)
			.with_spec_byte_raw(
				SpecIdx::Byte4,
				DnaUtils::<BlockNumber>::enums_order_to_bits(&comp_1) as u8,
			)
			.with_spec_byte_raw(
				SpecIdx::Byte5,
				DnaUtils::<BlockNumber>::enums_to_bits(&comp_2) as u8,
			)
			.with_spec_byte_raw(
				SpecIdx::Byte6,
				DnaUtils::<BlockNumber>::enums_order_to_bits(&comp_2) as u8,
			)
			.with_spec_byte_raw(
				SpecIdx::Byte7,
				DnaUtils::<BlockNumber>::enums_to_bits(&comp_3) as u8,
			)
			.with_spec_byte_raw(
				SpecIdx::Byte8,
				DnaUtils::<BlockNumber>::enums_order_to_bits(&comp_3) as u8,
			)
			.with_soul_count(quantity as SoulCount * custom_type_1 as SoulCount)
	}

	pub fn into_egg(
		self,
		rarity: &RarityTier,
		pet_variation: u8,
		soul_points: SoulCount,
		progress_array: [u8; 11],
	) -> Self {
		self.with_attribute(AvatarAttr::ItemType, &ItemType::Pet)
			.with_attribute(AvatarAttr::ItemSubType, &PetItemType::Egg)
			// Unused
			.with_attribute(AvatarAttr::ClassType1, &HexType::X0)
			// Unused
			.with_attribute(AvatarAttr::ClassType2, &HexType::X0)
			.with_attribute(AvatarAttr::CustomType1, &HexType::X0)
			.with_attribute(AvatarAttr::RarityTier, rarity)
			.with_attribute_raw(AvatarAttr::Quantity, 1)
			.with_attribute_raw(AvatarAttr::CustomType2, pet_variation)
			.with_progress_array(progress_array)
			.with_soul_count(soul_points)
	}

	pub fn into_material(self, material_type: &MaterialItemType, quantity: u8) -> Self {
		let sp_ratio = match *material_type {
			MaterialItemType::Ceramics | MaterialItemType::Electronics => HexType::X1,
			MaterialItemType::PowerCells | MaterialItemType::Polymers => HexType::X2,
			MaterialItemType::Superconductors | MaterialItemType::Metals => HexType::X3,
			MaterialItemType::Optics | MaterialItemType::Nanomaterials => HexType::X4,
		};

		self.with_attribute(AvatarAttr::ItemType, &ItemType::Material)
			.with_attribute(AvatarAttr::ItemSubType, material_type)
			.with_attribute(AvatarAttr::ClassType1, &HexType::X0)
			.with_attribute(AvatarAttr::ClassType2, &HexType::X0)
			.with_attribute(AvatarAttr::CustomType1, &sp_ratio)
			.with_attribute(AvatarAttr::RarityTier, &RarityTier::Common)
			.with_attribute_raw(AvatarAttr::Quantity, quantity)
			// Unused
			.with_attribute(AvatarAttr::CustomType2, &HexType::X0)
			.with_soul_count(quantity as SoulCount * sp_ratio as SoulCount)
	}

	pub fn into_glimmer(self, quantity: u8) -> Self {
		let custom_type_1 = HexType::from_byte(GLIMMER_SP);

		self.with_attribute(AvatarAttr::ItemType, &ItemType::Essence)
			.with_attribute(AvatarAttr::ItemSubType, &EssenceItemType::Glimmer)
			.with_attribute(AvatarAttr::ClassType1, &HexType::X0)
			.with_attribute(AvatarAttr::ClassType2, &HexType::X0)
			.with_attribute(AvatarAttr::CustomType1, &custom_type_1)
			// Unused
			.with_attribute(AvatarAttr::CustomType2, &HexType::X0)
			.with_attribute(AvatarAttr::RarityTier, &RarityTier::Uncommon)
			.with_attribute_raw(AvatarAttr::Quantity, quantity)
			.with_soul_count(quantity as SoulCount * custom_type_1 as SoulCount)
	}

	pub fn into_color_spark(
		self,
		color_pair: &(ColorType, ColorType),
		soul_points: SoulCount,
		progress_array: [u8; 11],
	) -> Self {
		self.with_attribute(AvatarAttr::ItemType, &ItemType::Essence)
			.with_attribute(AvatarAttr::ItemSubType, &EssenceItemType::ColorSpark)
			.with_attribute(AvatarAttr::ClassType1, &HexType::X0)
			.with_attribute(AvatarAttr::ClassType2, &HexType::X0)
			.with_attribute(AvatarAttr::CustomType1, &HexType::X0)
			// Unused
			.with_attribute(AvatarAttr::CustomType2, &HexType::X0)
			.with_attribute(AvatarAttr::RarityTier, &RarityTier::Rare)
			.with_attribute_raw(AvatarAttr::Quantity, 1)
			.with_spec_byte_raw(SpecIdx::Byte1, color_pair.0.as_byte())
			.with_spec_byte_raw(SpecIdx::Byte2, color_pair.1.as_byte())
			.with_spec_byte_raw(SpecIdx::Byte3, 0)
			.with_spec_byte_raw(SpecIdx::Byte4, 0)
			.with_spec_byte_raw(SpecIdx::Byte5, 0)
			.with_spec_byte_raw(SpecIdx::Byte6, 0)
			.with_spec_byte_raw(SpecIdx::Byte7, 0)
			.with_spec_byte_raw(SpecIdx::Byte8, 0)
			.with_progress_array(progress_array)
			.with_soul_count(soul_points)
	}

	pub fn into_glow_spark(
		self,
		force: &Force,
		soul_points: SoulCount,
		progress_array: [u8; 11],
	) -> Self {
		self.with_attribute(AvatarAttr::ItemType, &ItemType::Essence)
			.with_attribute(AvatarAttr::ItemSubType, &EssenceItemType::GlowSpark)
			.with_attribute(AvatarAttr::ClassType1, &HexType::X0)
			.with_attribute(AvatarAttr::ClassType2, &HexType::X0)
			.with_attribute(AvatarAttr::CustomType1, &HexType::X0)
			// Unused
			.with_attribute(AvatarAttr::CustomType2, &HexType::X0)
			.with_attribute(AvatarAttr::RarityTier, &RarityTier::Rare)
			.with_attribute_raw(AvatarAttr::Quantity, 1)
			.with_spec_byte_raw(SpecIdx::Byte1, force.as_byte())
			.with_spec_byte_raw(SpecIdx::Byte2, 0)
			.with_spec_byte_raw(SpecIdx::Byte3, 0)
			.with_spec_byte_raw(SpecIdx::Byte4, 0)
			.with_spec_byte_raw(SpecIdx::Byte5, 0)
			.with_spec_byte_raw(SpecIdx::Byte6, 0)
			.with_spec_byte_raw(SpecIdx::Byte7, 0)
			.with_spec_byte_raw(SpecIdx::Byte8, 0)
			.with_progress_array(progress_array)
			.with_soul_count(soul_points)
	}

	pub fn into_paint_flask(
		self,
		color_pair: &(ColorType, ColorType),
		soul_points: SoulCount,
		progress_array: [u8; 11],
	) -> Self {
		let color_bytes = ((color_pair.0.as_byte().saturating_sub(1)) << 6) |
			((color_pair.1.as_byte().saturating_sub(1)) << 4);

		self.with_attribute(AvatarAttr::ItemType, &ItemType::Essence)
			.with_attribute(AvatarAttr::ItemSubType, &EssenceItemType::PaintFlask)
			.with_attribute(AvatarAttr::ClassType1, &HexType::X0)
			.with_attribute(AvatarAttr::ClassType2, &HexType::X0)
			.with_attribute(AvatarAttr::CustomType1, &HexType::X0)
			// Unused
			.with_attribute(AvatarAttr::CustomType2, &HexType::X0)
			.with_attribute(AvatarAttr::RarityTier, &RarityTier::Epic)
			.with_attribute_raw(AvatarAttr::Quantity, 1)
			.with_spec_byte_raw(SpecIdx::Byte1, color_bytes)
			.with_spec_byte_raw(SpecIdx::Byte2, 0b0000_1000)
			.with_spec_byte_raw(SpecIdx::Byte3, 0)
			.with_spec_byte_raw(SpecIdx::Byte4, 0)
			.with_spec_byte_raw(SpecIdx::Byte5, 0)
			.with_spec_byte_raw(SpecIdx::Byte6, 0)
			.with_spec_byte_raw(SpecIdx::Byte7, 0)
			.with_spec_byte_raw(SpecIdx::Byte8, 0)
			.with_progress_array(progress_array)
			.with_soul_count(soul_points)
	}

	pub fn into_glow_flask(
		self,
		force: &Force,
		soul_points: SoulCount,
		progress_array: [u8; 11],
	) -> Self {
		self.with_attribute(AvatarAttr::ItemType, &ItemType::Essence)
			.with_attribute(AvatarAttr::ItemSubType, &EssenceItemType::GlowFlask)
			.with_attribute(AvatarAttr::ClassType1, &HexType::X0)
			.with_attribute(AvatarAttr::ClassType2, &HexType::X0)
			.with_attribute(AvatarAttr::CustomType1, &HexType::X0)
			// Unused
			.with_attribute(AvatarAttr::CustomType2, &HexType::X0)
			.with_attribute(AvatarAttr::RarityTier, &RarityTier::Epic)
			.with_attribute_raw(AvatarAttr::Quantity, 1)
			.with_spec_byte_raw(SpecIdx::Byte1, force.as_byte())
			.with_spec_byte_raw(SpecIdx::Byte2, 0)
			.with_spec_byte_raw(SpecIdx::Byte3, 0)
			.with_spec_byte_raw(SpecIdx::Byte4, 0)
			.with_spec_byte_raw(SpecIdx::Byte5, 0)
			.with_spec_byte_raw(SpecIdx::Byte6, 0)
			.with_spec_byte_raw(SpecIdx::Byte7, 0)
			.with_spec_byte_raw(SpecIdx::Byte8, 0)
			.with_progress_array(progress_array)
			.with_soul_count(soul_points)
	}

	pub fn try_into_armor_and_component<T: Config>(
		self,
		pet_type: &PetType,
		slot_type: &SlotType,
		equippable_type: &[EquippableItemType],
		rarity: &RarityTier,
		color_pair: &(ColorType, ColorType),
		force: &Force,
		soul_points: SoulCount,
		hash_provider: &mut HashProvider<T, 32>,
	) -> Result<Self, ()> {
		if equippable_type.is_empty() || equippable_type.iter().any(|equip| !equip.is_armor()) {
			return Err(())
		}

		let (armor_assemble_progress, color_flag) = {
			let mut color_flag = 0b0000_0000;
			let mut progress = DnaUtils::<BlockNumber>::enums_to_bits(equippable_type) as u8;

			if color_pair.0 != ColorType::Null && color_pair.1 != ColorType::Null {
				color_flag = 0b0000_1000;
				progress |= ((color_pair.0.as_byte().saturating_sub(1)) << 6) |
					((color_pair.1.as_byte().saturating_sub(1)) << 4)
			}

			(progress, color_flag)
		};

		// Guaranteed to work because of check above
		let first_equippable = equippable_type.first().unwrap();

		let progress_array = DnaUtils::<BlockNumber>::generate_progress(
			rarity,
			SCALING_FACTOR_PERC,
			Some(PROGRESS_PROBABILITY_PERC),
			hash_provider,
		);

		Ok(self
			.with_attribute(AvatarAttr::ItemType, &ItemType::Equippable)
			.with_attribute(AvatarAttr::ItemSubType, first_equippable)
			.with_attribute(AvatarAttr::ClassType1, slot_type)
			.with_attribute(AvatarAttr::ClassType2, pet_type)
			.with_attribute(AvatarAttr::CustomType1, &HexType::X0)
			.with_attribute(AvatarAttr::RarityTier, rarity)
			.with_attribute_raw(AvatarAttr::Quantity, 1)
			// Unused
			.with_attribute(AvatarAttr::CustomType2, &HexType::X0)
			.with_spec_byte_raw(SpecIdx::Byte1, armor_assemble_progress)
			.with_spec_byte_raw(SpecIdx::Byte2, force.as_byte() | color_flag)
			.with_spec_byte_raw(SpecIdx::Byte3, 0)
			.with_spec_byte_raw(SpecIdx::Byte4, 0)
			.with_spec_byte_raw(SpecIdx::Byte5, 0)
			.with_spec_byte_raw(SpecIdx::Byte6, 0)
			.with_spec_byte_raw(SpecIdx::Byte7, 0)
			.with_spec_byte_raw(SpecIdx::Byte8, 0)
			.with_progress_array(progress_array)
			.with_soul_count(soul_points))
	}

	pub fn try_into_weapon<T: Config>(
		self,
		pet_type: &PetType,
		slot_type: &SlotType,
		equippable_type: &EquippableItemType,
		color_pair: &(ColorType, ColorType),
		force: &Force,
		soul_points: SoulCount,
		hash_provider: &mut HashProvider<T, 32>,
	) -> Result<Self, ()> {
		if !equippable_type.is_weapon() {
			return Err(())
		}

		let (weapon_info, color_flag) = {
			let mut color_flag = 0b0000_0000;
			let mut info = DnaUtils::<BlockNumber>::enums_to_bits(&[*equippable_type]) as u8 >> 4;

			if color_pair.0 != ColorType::Null && color_pair.1 != ColorType::Null {
				color_flag = 0b0000_1000;
				info |= ((color_pair.0.as_byte().saturating_sub(1)) << 6) |
					((color_pair.1.as_byte().saturating_sub(1)) << 4)
			}

			(info, color_flag)
		};

		let rarity = RarityTier::Legendary;

		let progress_array = DnaUtils::<BlockNumber>::generate_progress(
			&rarity,
			SCALING_FACTOR_PERC,
			None,
			hash_provider,
		);

		Ok(self
			.with_attribute(AvatarAttr::ItemType, &ItemType::Equippable)
			.with_attribute(AvatarAttr::ItemSubType, equippable_type)
			.with_attribute(AvatarAttr::ClassType1, slot_type)
			.with_attribute(AvatarAttr::ClassType2, pet_type)
			.with_attribute(AvatarAttr::CustomType1, &HexType::X0)
			.with_attribute(AvatarAttr::RarityTier, &rarity)
			.with_attribute_raw(AvatarAttr::Quantity, 1)
			// Unused
			.with_attribute(AvatarAttr::CustomType2, &HexType::X0)
			.with_spec_byte_raw(SpecIdx::Byte1, weapon_info)
			.with_spec_byte_raw(SpecIdx::Byte2, force.as_byte() | color_flag)
			.with_spec_byte_raw(SpecIdx::Byte3, 0)
			.with_spec_byte_raw(SpecIdx::Byte4, 0)
			.with_spec_byte_raw(SpecIdx::Byte5, 0)
			.with_spec_byte_raw(SpecIdx::Byte6, 0)
			.with_spec_byte_raw(SpecIdx::Byte7, 0)
			.with_spec_byte_raw(SpecIdx::Byte8, 0)
			.with_progress_array(progress_array)
			.with_soul_count(soul_points))
	}

	pub fn into_blueprint(
		self,
		blueprint_type: &BlueprintItemType,
		pet_type: &PetType,
		slot_type: &SlotType,
		equippable_item_type: &EquippableItemType,
		pattern: &[MaterialItemType],
		quantity: u8,
	) -> Self {
		// TODO: add a quantity algorithm
		// - base 8 - 16 and
		// - components 6 - 12
		let mat_req1 = 1;
		let mat_req2 = 1;
		let mat_req3 = 1;
		let mat_req4 = 1;

		self.with_attribute(AvatarAttr::ItemType, &ItemType::Blueprint)
			.with_attribute(AvatarAttr::ItemSubType, blueprint_type)
			.with_attribute(AvatarAttr::ClassType1, slot_type)
			.with_attribute(AvatarAttr::ClassType2, pet_type)
			.with_attribute(AvatarAttr::CustomType1, &HexType::X1)
			.with_attribute(AvatarAttr::RarityTier, &RarityTier::Rare)
			.with_attribute_raw(AvatarAttr::Quantity, quantity)
			// Unused
			.with_attribute(AvatarAttr::CustomType2, &HexType::X0)
			.with_spec_byte_raw(
				SpecIdx::Byte1,
				DnaUtils::<BlockNumber>::enums_to_bits(pattern) as u8,
			)
			.with_spec_byte_raw(
				SpecIdx::Byte2,
				DnaUtils::<BlockNumber>::enums_order_to_bits(pattern) as u8,
			)
			.with_spec_byte_raw(SpecIdx::Byte3, equippable_item_type.as_byte())
			.with_spec_byte_raw(SpecIdx::Byte4, mat_req1)
			.with_spec_byte_raw(SpecIdx::Byte5, mat_req2)
			.with_spec_byte_raw(SpecIdx::Byte6, mat_req3)
			.with_spec_byte_raw(SpecIdx::Byte7, mat_req4)
			.with_soul_count(quantity as SoulCount)
	}

	pub fn into_unidentified(
		self,
		color_pair: (ColorType, ColorType),
		force: Force,
		soul_points: SoulCount,
	) -> Self {
		let git_info = 0b0000_1111 |
			((color_pair.0.as_byte().saturating_sub(1)) << 6 |
				(color_pair.1.as_byte().saturating_sub(1)) << 4);

		self.with_attribute(AvatarAttr::ItemType, &ItemType::Special)
			.with_attribute(AvatarAttr::ItemSubType, &SpecialItemType::Unidentified)
			.with_attribute(AvatarAttr::ClassType1, &HexType::X0)
			.with_attribute(AvatarAttr::ClassType2, &HexType::X0)
			.with_attribute(AvatarAttr::CustomType1, &HexType::X0)
			// Unused
			.with_attribute(AvatarAttr::CustomType2, &HexType::X0)
			.with_attribute(AvatarAttr::RarityTier, &RarityTier::Legendary)
			.with_attribute_raw(AvatarAttr::Quantity, 1)
			.with_spec_byte_raw(SpecIdx::Byte1, git_info)
			.with_spec_byte_raw(SpecIdx::Byte2, force.as_byte())
			.with_soul_count(soul_points)
	}

	pub fn into_dust(self, soul_points: SoulCount) -> Self {
		self.with_attribute(AvatarAttr::ItemType, &ItemType::Special)
			.with_attribute(AvatarAttr::ItemSubType, &SpecialItemType::Dust)
			.with_attribute(AvatarAttr::ClassType1, &HexType::X0)
			.with_attribute(AvatarAttr::ClassType2, &HexType::X0)
			.with_attribute(AvatarAttr::CustomType1, &HexType::X1)
			.with_attribute(AvatarAttr::CustomType2, &HexType::X0)
			.with_attribute(AvatarAttr::RarityTier, &RarityTier::Common)
			.with_attribute_raw(AvatarAttr::Quantity, soul_points as u8)
			.with_soul_count(soul_points)
	}

	pub fn into_toolbox(self, soul_points: SoulCount) -> Self {
		self.with_attribute(AvatarAttr::ItemType, &ItemType::Special)
			.with_attribute(AvatarAttr::ItemSubType, &SpecialItemType::ToolBox)
			.with_attribute(AvatarAttr::ClassType1, &HexType::X0)
			.with_attribute(AvatarAttr::ClassType2, &HexType::X0)
			.with_attribute(AvatarAttr::CustomType1, &HexType::X0)
			.with_attribute(AvatarAttr::CustomType2, &HexType::X0)
			.with_attribute(AvatarAttr::RarityTier, &RarityTier::Epic)
			.with_attribute_raw(AvatarAttr::Quantity, 1)
			.with_progress_array([0xBB; 11])
			.with_soul_count(soul_points)
	}

	pub fn build(self) -> Avatar<BlockNumber> {
		self.inner
	}

	#[allow(dead_code)]
	pub fn build_wrapped(self) -> WrappedAvatar<BlockNumber> {
		WrappedAvatar::new(self.inner)
	}
}
