use super::*;

pub(crate) type WrappedForgeItem<T> = (AvatarIdOf<T>, WrappedAvatar<BlockNumberFor<T>>);

#[derive(Clone)]
pub(crate) struct WrappedAvatar<BlockNumber> {
	inner: Avatar<BlockNumber>,
}

#[allow(dead_code)]
impl<BlockNumber> WrappedAvatar<BlockNumber>
where
	BlockNumber: sp_runtime::traits::BlockNumber,
{
	pub fn new(avatar: Avatar<BlockNumber>) -> Self {
		Self { inner: avatar }
	}

	pub fn unwrap(self) -> Avatar<BlockNumber> {
		self.inner
	}

	pub fn get_dna(&self) -> &Dna {
		&self.inner.dna
	}

	pub fn get_souls(&self) -> SoulCount {
		self.inner.souls
	}

	pub fn set_souls(&mut self, souls: SoulCount) {
		self.inner.souls = souls;
	}

	pub fn add_souls(&mut self, souls: SoulCount) {
		self.inner.souls += souls;
	}

	pub fn dec_souls(&mut self, souls: SoulCount) {
		self.inner.souls -= souls;
	}

	pub fn can_use(&self, quantity: u8) -> bool {
		self.get_quantity() >= quantity
	}

	pub fn use_avatar(&mut self, quantity: u8) -> (bool, bool, SoulCount) {
		let current_qty = self.get_quantity();

		if current_qty < quantity {
			return (false, false, 0)
		}

		let new_qty = current_qty - quantity;
		self.set_quantity(new_qty);

		let (avatar_consumed, output_soul_points) = if new_qty == 0 {
			let soul_points = self.get_souls();
			self.set_souls(0);
			(true, soul_points)
		} else {
			let diff = self.get_custom_type_1::<u8>().saturating_mul(quantity) as SoulCount;
			self.set_souls(self.get_souls().saturating_sub(diff));
			(false, diff)
		};

		(true, avatar_consumed, output_soul_points)
	}

	// TODO: Improve return type to [[u8; 3]; 10] if possible
	pub fn spec_byte_split_ten(&self) -> Vec<Vec<u8>> {
		self.get_specs()
			.into_iter()
			.flat_map(|entry| [entry >> 4, entry & 0x0F])
			.take(30)
			.collect::<Vec<u8>>()
			.chunks_exact(3)
			.map(|item| item.into())
			.collect::<Vec<Vec<u8>>>()
	}

	pub fn spec_byte_split_ten_count(&self) -> usize {
		self.spec_byte_split_ten()
			.into_iter()
			.filter(|segment| segment.iter().sum::<u8>() > 0)
			.count()
	}

	pub fn get_item_type<T>(&self) -> T
	where
		T: ByteConvertible,
	{
		DnaUtils::read_attribute(&self.inner, AvatarAttr::ItemType)
	}

	pub fn set_item_type<T>(&mut self, item_type: T)
	where
		T: ByteConvertible,
	{
		DnaUtils::write_attribute(&mut self.inner, AvatarAttr::ItemType, &item_type)
	}

	pub fn same_item_type<T>(&self, other: &WrappedAvatar<BlockNumber>) -> bool
	where
		T: ByteConvertible + Eq,
	{
		self.get_item_type::<T>() == other.get_item_type::<T>()
	}

	pub fn get_item_sub_type<T>(&self) -> T
	where
		T: ByteConvertible,
	{
		DnaUtils::read_attribute::<T>(&self.inner, AvatarAttr::ItemSubType)
	}

	pub fn set_item_sub_type<T>(&mut self, item_sub_type: T)
	where
		T: ByteConvertible,
	{
		DnaUtils::write_attribute::<T>(&mut self.inner, AvatarAttr::ItemSubType, &item_sub_type)
	}

	pub fn same_item_sub_type(&self, other: &WrappedAvatar<BlockNumber>) -> bool {
		self.get_item_sub_type::<u8>().cmp(&other.get_item_sub_type::<u8>()).is_eq()
	}

	pub fn get_class_type_1<T>(&self) -> T
	where
		T: ByteConvertible,
	{
		DnaUtils::read_attribute::<T>(&self.inner, AvatarAttr::ClassType1)
	}

	pub fn set_class_type_1<T>(&mut self, class_type_1: T)
	where
		T: ByteConvertible,
	{
		DnaUtils::write_attribute::<T>(&mut self.inner, AvatarAttr::ClassType1, &class_type_1)
	}

	pub fn same_class_type_1(&self, other: &WrappedAvatar<BlockNumber>) -> bool {
		self.get_class_type_1::<u8>().cmp(&other.get_class_type_1::<u8>()).is_eq()
	}

	pub fn get_class_type_2<T>(&self) -> T
	where
		T: ByteConvertible,
	{
		DnaUtils::read_attribute::<T>(&self.inner, AvatarAttr::ClassType2)
	}

	pub fn set_class_type_2<T>(&mut self, class_type_2: T)
	where
		T: ByteConvertible,
	{
		DnaUtils::write_attribute::<T>(&mut self.inner, AvatarAttr::ClassType2, &class_type_2)
	}

	pub fn same_class_type_2(&self, other: &WrappedAvatar<BlockNumber>) -> bool {
		self.get_class_type_2::<u8>().cmp(&other.get_class_type_2::<u8>()).is_eq()
	}

	pub fn get_custom_type_1<T>(&self) -> T
	where
		T: ByteConvertible,
	{
		DnaUtils::read_attribute::<T>(&self.inner, AvatarAttr::CustomType1)
	}

	pub fn set_custom_type_1<T>(&mut self, custom_type_1: T)
	where
		T: ByteConvertible,
	{
		DnaUtils::write_attribute::<T>(&mut self.inner, AvatarAttr::CustomType1, &custom_type_1)
	}

	pub fn same_custom_type_1(&self, other: &WrappedAvatar<BlockNumber>) -> bool {
		self.get_custom_type_1::<u8>().cmp(&other.get_custom_type_1::<u8>()).is_eq()
	}

	pub fn get_custom_type_2<T>(&self) -> T
	where
		T: ByteConvertible,
	{
		DnaUtils::read_attribute::<T>(&self.inner, AvatarAttr::CustomType2)
	}

	pub fn set_custom_type_2<T>(&mut self, custom_type_2: T)
	where
		T: ByteConvertible,
	{
		DnaUtils::write_attribute::<T>(&mut self.inner, AvatarAttr::CustomType2, &custom_type_2)
	}

	pub fn same_custom_type_2(&self, other: &WrappedAvatar<BlockNumber>) -> bool {
		self.get_custom_type_2::<u8>().cmp(&other.get_custom_type_2::<u8>()).is_eq()
	}

	pub fn get_rarity(&self) -> RarityTier {
		DnaUtils::read_attribute(&self.inner, AvatarAttr::RarityTier)
	}

	pub fn set_rarity(&mut self, rarity: RarityTier) {
		DnaUtils::write_attribute::<RarityTier>(&mut self.inner, AvatarAttr::RarityTier, &rarity)
	}

	pub fn same_rarity(&self, other: &WrappedAvatar<BlockNumber>) -> bool {
		self.get_rarity().cmp(&other.get_rarity()).is_eq()
	}

	pub fn get_quantity(&self) -> u8 {
		DnaUtils::read_attribute_raw(&self.inner, AvatarAttr::Quantity)
	}

	pub fn set_quantity(&mut self, quantity: u8) {
		DnaUtils::write_attribute_raw(&mut self.inner, AvatarAttr::Quantity, quantity)
	}

	pub fn same_quantity(&self, other: &WrappedAvatar<BlockNumber>) -> bool {
		self.get_quantity().cmp(&other.get_quantity()).is_eq()
	}

	pub fn get_spec<T>(&self, spec_index: SpecIdx) -> T
	where
		T: ByteConvertible,
	{
		DnaUtils::read_spec::<T>(&self.inner, spec_index)
	}

	pub fn set_spec(&mut self, spec_index: SpecIdx, value: u8) {
		DnaUtils::write_spec(&mut self.inner, spec_index, value);
	}

	pub fn get_specs(&self) -> [u8; 16] {
		DnaUtils::read_specs(&self.inner)
	}

	pub fn set_specs(&mut self, spec_bytes: [u8; 16]) {
		DnaUtils::write_specs(&mut self.inner, spec_bytes)
	}

	pub fn same_specs(&self, other: &WrappedAvatar<BlockNumber>) -> bool {
		self.get_specs() == other.get_specs()
	}

	pub fn same_spec_at(&self, other: &WrappedAvatar<BlockNumber>, position: SpecIdx) -> bool {
		DnaUtils::read_spec_raw(&self.inner, position) ==
			DnaUtils::read_spec_raw(&other.inner, position)
	}

	pub fn get_progress(&self) -> [u8; 11] {
		DnaUtils::read_progress(&self.inner)
	}

	pub fn set_progress(&mut self, progress_array: [u8; 11]) {
		DnaUtils::write_progress(&mut self.inner, progress_array)
	}

	pub fn same_progress(&self, other: &WrappedAvatar<BlockNumber>) -> bool {
		self.get_progress() == other.get_progress()
	}

	pub fn has_type<T>(&self, item_type: T) -> bool
	where
		T: ByteConvertible + Eq,
	{
		self.get_item_type::<T>() == item_type
	}

	pub fn has_subtype<T>(&self, item_sub_type: T) -> bool
	where
		T: ByteConvertible + Eq,
	{
		self.get_item_sub_type::<T>() == item_sub_type
	}

	pub fn has_full_type<T, U>(&self, item_type: T, item_sub_type: U) -> bool
	where
		T: ByteConvertible + Eq,
		U: ByteConvertible + Eq,
	{
		self.has_type(item_type) && self.has_subtype(item_sub_type)
	}

	pub fn has_zeroed_class_types(&self) -> bool {
		self.get_class_type_1::<u8>() == 0 && self.get_class_type_2::<u8>() == 0
	}

	pub fn same_full_type(&self, other: &WrappedAvatar<BlockNumber>) -> bool {
		self.same_item_type::<u8>(other) && self.same_item_sub_type(other)
	}

	pub fn same_full_and_class_types(&self, other: &WrappedAvatar<BlockNumber>) -> bool {
		self.same_full_type(other) && self.same_class_type_1(other) && self.same_class_type_2(other)
	}

	pub fn same_assemble_version(&self, other: &WrappedAvatar<BlockNumber>) -> bool {
		self.same_item_type::<u8>(other) &&
			self.same_class_type_1(other) &&
			self.same_class_type_2(other)
	}
}
