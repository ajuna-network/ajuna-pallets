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

	pub fn get_segment_attribute_of_one(&self, idx: usize, start_bit: u8, bits: u8) -> u8 {
		let value = self.inner.dna[idx];

		let total_mov = 8_u8.saturating_sub(start_bit + bits);
		let mask = u8::MAX >> start_bit;

		(value & mask) >> total_mov
	}

	pub fn set_segment_attribute_of_one(&mut self, idx: usize, start_bit: u8, bits: u8, value: u8) {
		if bits == 8 {
			self.inner.dna[idx] = value;
		} else {
			let mov = 8_u8.saturating_sub(start_bit + bits);
			let dna_mask = (0b1111_1111 ^ u8::MAX >> start_bit) | (0b1111_1111 ^ u8::MAX << mov);
			let value_mask = 0b1111_1111 ^ dna_mask;
			self.inner.dna[idx] = (self.inner.dna[idx] & dna_mask) | ((value << mov) & value_mask);
		}
	}

	#[inline]
	fn get_lower_segment(&self, idx: usize, bits: u8) -> u8 {
		let mask = if bits == 8 { 0b1111_1111 } else { u8::MAX >> (8_u8.saturating_sub(bits)) };
		self.inner.dna[idx] & mask
	}

	#[inline]
	fn get_upper_segment(&self, idx: usize, bits: u8) -> u8 {
		let mask = if bits == 8 { 0b1111_1111 } else { 0b1111_1111 ^ (u8::MAX >> bits) };
		self.inner.dna[idx] & mask
	}

	pub fn get_segmented_attribute_of_two(&self, start_idx: usize, segment_bits: &[u8; 2]) -> u16 {
		let mut bytes = [0; 2];

		bytes[0] = self.get_lower_segment(start_idx, segment_bits[0]);
		bytes[1] = self.get_upper_segment(start_idx + 1, segment_bits[1]);

		u16::from_be_bytes(bytes) >> 8_u8.saturating_sub(segment_bits[1])
	}

	#[inline]
	fn set_lower_segment(&mut self, idx: usize, bits: u8, value: u8) {
		let value_mask =
			if bits == 8 { 0b1111_1111 } else { u8::MAX >> (8_u8.saturating_sub(bits)) };
		let dna_mask = 0b1111_1111 ^ value_mask;
		self.inner.dna[idx] = (self.inner.dna[idx] & dna_mask) | (value & value_mask);
	}

	#[inline]
	fn set_upper_segment(&mut self, idx: usize, bits: u8, value: u8) {
		let value_mask = if bits == 8 { 0b1111_1111 } else { 0b1111_1111 ^ (u8::MAX >> bits) };
		let dna_mask = 0b1111_1111 ^ value_mask;
		self.inner.dna[idx] = (self.inner.dna[idx] & dna_mask) | (value & value_mask);
	}

	pub fn set_segmented_attribute_of_two(
		&mut self,
		start_idx: usize,
		segment_bits: &[u8; 2],
		value: u16,
	) {
		let total_bits = (segment_bits[0] + segment_bits[1]) as u16;
		let masked_value = (u16::MAX >> 16_u16.saturating_sub(total_bits)) & value;

		let lower_value = (masked_value >> segment_bits[1]) as u8;
		self.set_lower_segment(start_idx, segment_bits[0], lower_value);
		let upper_value = (masked_value << 8_u16.saturating_sub(segment_bits[1] as u16)) as u8;
		self.set_upper_segment(start_idx + 1, segment_bits[1], upper_value);
	}

	pub fn get_segmented_attribute_of_three(
		&self,
		start_idx: usize,
		segment_bits: &[u8; 2],
	) -> u32 {
		let mut bytes = [0; 4];

		bytes[1] = self.get_lower_segment(start_idx, segment_bits[0]);
		bytes[2] = self.inner.dna[start_idx + 1];
		bytes[3] = self.get_upper_segment(start_idx + 2, segment_bits[1]);

		u32::from_be_bytes(bytes) >> 8_u8.saturating_sub(segment_bits[1])
	}

	pub fn set_segmented_attribute_of_three(
		&mut self,
		start_idx: usize,
		segment_bits: &[u8; 2],
		value: u32,
	) {
		let total_bits = (segment_bits[0] + segment_bits[1] + 8) as u32;
		let masked_value = (u32::MAX >> 32_u32.saturating_sub(total_bits)) & value;

		let lower_value = (masked_value >> (segment_bits[1] + 8)) as u8;
		self.set_lower_segment(start_idx, segment_bits[0], lower_value);

		let middle_value = (masked_value >> segment_bits[1]) as u8;
		self.inner.dna[start_idx + 1] = middle_value;

		let upper_value = (masked_value << 8_u32.saturating_sub(segment_bits[1] as u32)) as u8;
		self.set_upper_segment(start_idx + 2, segment_bits[1], upper_value);
	}

	pub fn get_segmented_attribute_of_eight(
		&self,
		start_idx: usize,
		segment_bits: &[u8; 2],
	) -> u64 {
		let mut bytes = [0; 8];

		bytes[0] = self.get_lower_segment(start_idx, segment_bits[0]);
		bytes[1] = self.inner.dna[start_idx + 1];
		bytes[2] = self.inner.dna[start_idx + 2];
		bytes[3] = self.inner.dna[start_idx + 3];
		bytes[4] = self.inner.dna[start_idx + 4];
		bytes[5] = self.inner.dna[start_idx + 5];
		bytes[6] = self.inner.dna[start_idx + 6];
		bytes[7] = self.get_upper_segment(start_idx + 7, segment_bits[1]);

		u64::from_be_bytes(bytes) >> 8_u8.saturating_sub(segment_bits[1])
	}

	pub fn set_segmented_attribute_of_eight(
		&mut self,
		start_idx: usize,
		segment_bits: &[u8; 2],
		value: u64,
	) {
		let total_bits = (segment_bits[0] + segment_bits[1] + 48) as u64;
		let masked_value = (u64::MAX >> 64_u64.saturating_sub(total_bits)) & value;

		let lower_value = (masked_value >> (segment_bits[1] + 48)) as u8;
		self.set_lower_segment(start_idx, segment_bits[0], lower_value);

		for (i, idx) in ((start_idx + 1)..(start_idx + 7)).enumerate() {
			let mov = 40 - (8 * i as u8);
			let middle_value = (masked_value >> (segment_bits[1] + mov)) as u8;
			self.inner.dna[idx] = middle_value;
		}

		let upper_value = (masked_value << 8_u64.saturating_sub(segment_bits[1] as u64)) as u8;
		self.set_upper_segment(start_idx + 7, segment_bits[1], upper_value);
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::mock::*;

	#[test]
	fn test_get_segment_attribute_of_one() {
		ExtBuilder::default().build().execute_with(|| {
			let mut avatar = WrappedAvatar::new(Avatar::<BlockNumberFor<Test>>::default());
			avatar.inner.dna = BoundedVec::try_from([0_u8; 32].to_vec()).unwrap();

			avatar.inner.dna[1] = 0b0011_0011;
			let value = avatar.get_segment_attribute_of_one(1, 2, 2);
			assert_eq!(value, 0b0000_0011);

			avatar.inner.dna[4] = 0b1011_0101;
			let value = avatar.get_segment_attribute_of_one(4, 3, 4);
			assert_eq!(value, 0b0000_1010);

			avatar.inner.dna[11] = 0b1011_0101;
			let value = avatar.get_segment_attribute_of_one(11, 0, 8);
			assert_eq!(value, 0b1011_0101);
		});
	}

	#[test]
	fn test_set_segment_attribute_of_one() {
		ExtBuilder::default().build().execute_with(|| {
			let mut avatar = WrappedAvatar::new(Avatar::<BlockNumberFor<Test>>::default());
			avatar.inner.dna = BoundedVec::try_from([0_u8; 32].to_vec()).unwrap();

			avatar.inner.dna[3] = 0b1000_0101;
			let value = 0b0110_1011;
			avatar.set_segment_attribute_of_one(3, 1, 2, value);
			assert_eq!(avatar.inner.dna[3], 0b1110_0101);

			avatar.inner.dna[3] = 0b1000_0100;
			let value = 0b0110_1011;
			avatar.set_segment_attribute_of_one(3, 0, 7, value);
			assert_eq!(avatar.inner.dna[3], 0b1101_0110);

			avatar.inner.dna[3] = 0b1000_0000;
			let value = 0b0110_1011;
			avatar.set_segment_attribute_of_one(3, 5, 1, value);
			assert_eq!(avatar.inner.dna[3], 0b1000_0100);

			avatar.inner.dna[3] = 0b1000_0000;
			let value = 0b0110_1011;
			avatar.set_segment_attribute_of_one(3, 0, 8, value);
			assert_eq!(avatar.inner.dna[3], 0b0110_1011);
		});
	}

	#[test]
	fn test_get_segmented_attribute_of_two() {
		ExtBuilder::default().build().execute_with(|| {
			let mut avatar = WrappedAvatar::new(Avatar::<BlockNumberFor<Test>>::default());
			avatar.inner.dna = BoundedVec::try_from([0_u8; 32].to_vec()).unwrap();

			avatar.inner.dna[3] = 0b0011_0011;
			avatar.inner.dna[4] = 0b1010_1010;
			let bits = [5, 2];
			let value = avatar.get_segmented_attribute_of_two(3, &bits);
			assert_eq!(value, 0b0000_0000_0100_1110);

			avatar.inner.dna[11] = 0b1010_1010;
			avatar.inner.dna[12] = 0b1100_1001;
			let bits = [6, 7];
			let value = avatar.get_segmented_attribute_of_two(11, &bits);
			assert_eq!(value, 0b0001_0101_0110_0100);

			avatar.inner.dna[20] = 0b1010_1010;
			avatar.inner.dna[21] = 0b0101_0101;
			let bits = [8, 8];
			let value = avatar.get_segmented_attribute_of_two(20, &bits);
			assert_eq!(value, 0b1010_1010_0101_0101);
		});
	}

	#[test]
	fn test_set_segmented_attribute_of_two() {
		ExtBuilder::default().build().execute_with(|| {
			let mut avatar = WrappedAvatar::new(Avatar::<BlockNumberFor<Test>>::default());
			avatar.inner.dna = BoundedVec::try_from([0_u8; 32].to_vec()).unwrap();

			avatar.inner.dna[3] = 0b1010_0101;
			let bits = [4, 8];
			let value = 0b0000_0010_1111_1011;
			avatar.set_segmented_attribute_of_two(3, &bits, value);
			assert_eq!(avatar.inner.dna[3], 0b1010_0010);
			assert_eq!(avatar.inner.dna[4], 0b1111_1011);

			avatar.inner.dna[7] = 0b1100_1000;
			avatar.inner.dna[8] = 0b1111_0011;
			let bits = [2, 3];
			let value = 0b0100_0001_0110_1000;
			avatar.set_segmented_attribute_of_two(7, &bits, value);
			assert_eq!(avatar.inner.dna[7], 0b1100_1001);
			assert_eq!(avatar.inner.dna[8], 0b0001_0011);
		});
	}

	#[test]
	fn test_get_segmented_attribute_of_three() {
		ExtBuilder::default().build().execute_with(|| {
			let mut avatar = WrappedAvatar::new(Avatar::<BlockNumberFor<Test>>::default());
			avatar.inner.dna = BoundedVec::try_from([0_u8; 32].to_vec()).unwrap();

			avatar.inner.dna[3] = 0b0011_0011;
			avatar.inner.dna[4] = 0b1110_1010;
			avatar.inner.dna[5] = 0b1010_1010;
			let bits = [5, 2];
			let value = avatar.get_segmented_attribute_of_three(3, &bits);
			assert_eq!(value, 0b0000_0000_0000_0000_0100_1111_1010_1010);

			avatar.inner.dna[11] = 0b1010_1010;
			avatar.inner.dna[12] = 0b0000_1101;
			avatar.inner.dna[13] = 0b1111_1111;
			let bits = [6, 7];
			let value = avatar.get_segmented_attribute_of_three(11, &bits);
			assert_eq!(value, 0b0000_0000_0001_0101_0000_0110_1111_1111);

			avatar.inner.dna[20] = 0b1010_1010;
			avatar.inner.dna[21] = 0b0101_0101;
			avatar.inner.dna[22] = 0b1111_1111;
			let bits = [8, 8];
			let value = avatar.get_segmented_attribute_of_three(20, &bits);
			assert_eq!(value, 0b0000_0000_1010_1010_0101_0101_1111_1111);
		});
	}

	#[test]
	fn test_set_segmented_attribute_of_three() {
		ExtBuilder::default().build().execute_with(|| {
			let mut avatar = WrappedAvatar::new(Avatar::<BlockNumberFor<Test>>::default());
			avatar.inner.dna = BoundedVec::try_from([0_u8; 32].to_vec()).unwrap();

			avatar.inner.dna[3] = 0b1010_0101;
			let bits = [5, 8];
			let value = 0b0000_0000_0001_0101_0000_0110_1111_1111;
			avatar.set_segmented_attribute_of_three(3, &bits, value);
			assert_eq!(avatar.inner.dna[3], 0b1011_0101);
			assert_eq!(avatar.inner.dna[4], 0b0000_0110);
			assert_eq!(avatar.inner.dna[5], 0b1111_1111);

			avatar.inner.dna[7] = 0b1100_1000;
			avatar.inner.dna[8] = 0b1111_0011;
			let bits = [2, 3];
			let value = 0b0000_0000_0001_0101_0000_0110_1111_1111;
			avatar.set_segmented_attribute_of_three(7, &bits, value);
			assert_eq!(avatar.inner.dna[7], 0b1100_1000);
			assert_eq!(avatar.inner.dna[8], 0b1101_1111);
			assert_eq!(avatar.inner.dna[9], 0b1110_0000);

			avatar.inner.dna[11] = 0b1001_1010;
			avatar.inner.dna[12] = 0b0011_0011;
			let bits = [8, 1];
			let value = 0b0000_0000_1010_1110_1110_1010_1010_1011;
			avatar.set_segmented_attribute_of_three(11, &bits, value);
			assert_eq!(avatar.inner.dna[11], 0b0111_0101);
			assert_eq!(avatar.inner.dna[12], 0b0101_0101);
			assert_eq!(avatar.inner.dna[13], 0b1000_0000);
		});
	}

	#[test]
	fn test_get_segmented_attribute_of_eight() {
		ExtBuilder::default().build().execute_with(|| {
			let mut avatar = WrappedAvatar::new(Avatar::<BlockNumberFor<Test>>::default());
			avatar.inner.dna = BoundedVec::try_from([0_u8; 32].to_vec()).unwrap();

			avatar.inner.dna[3] = 0b0011_0011;
			avatar.inner.dna[4] = 0b1110_1010;
			avatar.inner.dna[5] = 0b1010_1010;
			avatar.inner.dna[6] = 0b1110_1110;
			avatar.inner.dna[7] = 0b1010_1011;
			avatar.inner.dna[8] = 0b1011_1010;
			avatar.inner.dna[9] = 0b1111_1111;
			avatar.inner.dna[10] = 0b0100_1000;
			let bits = [5, 2];
			let value = avatar.get_segmented_attribute_of_eight(3, &bits);
			assert_eq!(
				value,
				0b0000_0000_0100_1111_1010_1010_1010_1011_1011_1010_1010_1110_1110_1011_1111_1101
			);
		});
	}

	#[test]
	fn test_set_segmented_attribute_of_eight() {
		ExtBuilder::default().build().execute_with(|| {
			let mut avatar = WrappedAvatar::new(Avatar::<BlockNumberFor<Test>>::default());
			avatar.inner.dna = BoundedVec::try_from([0_u8; 32].to_vec()).unwrap();

			avatar.inner.dna[3] = 0b0011_0011;
			avatar.inner.dna[4] = 0b1110_1010;
			avatar.inner.dna[5] = 0b1010_1010;
			avatar.inner.dna[6] = 0b1110_1110;
			avatar.inner.dna[7] = 0b1010_1011;
			avatar.inner.dna[8] = 0b1011_1010;
			avatar.inner.dna[9] = 0b1111_1111;
			avatar.inner.dna[10] = 0b0100_1000;

			let bits = [7, 1];
			let value =
				0b1100_1100_1100_1100_1100_1100_0011_1001_1010_1010_1111_0000_1100_1010_0011_0011;
			avatar.set_segmented_attribute_of_eight(3, &bits, value);
			assert_eq!(avatar.inner.dna[3], 0b0110_0110);
			assert_eq!(avatar.inner.dna[4], 0b0110_0110);
			assert_eq!(avatar.inner.dna[5], 0b0001_1100);
			assert_eq!(avatar.inner.dna[6], 0b1101_0101);
			assert_eq!(avatar.inner.dna[7], 0b0111_1000);
			assert_eq!(avatar.inner.dna[8], 0b0110_0101);
			assert_eq!(avatar.inner.dna[9], 0b0001_1001);
			assert_eq!(avatar.inner.dna[10], 0b1100_1000);
		});
	}
}
