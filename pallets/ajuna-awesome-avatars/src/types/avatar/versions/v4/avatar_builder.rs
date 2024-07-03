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

	pub fn into_unprospected_moon(self, stardust_amt: u16) -> Self {
		self.with_attribute(AvatarAttr::ItemType, &ItemType::Celestial)
			.with_attribute(AvatarAttr::ItemSubType, &CelestialItemType::UnprospectedMoon)
	}

	pub fn build(self) -> Avatar<BlockNumber> {
		self.inner
	}

	#[allow(dead_code)]
	pub fn build_wrapped(self) -> WrappedAvatar<BlockNumber> {
		WrappedAvatar::new(self.inner)
	}
}
