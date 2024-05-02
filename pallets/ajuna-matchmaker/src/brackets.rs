//! # Matchmaker Brackets (based on Transient RingBuffer implementation)
//!
//! This pallet provides a trait and implementation for a brackets ranked system that
//! abstracts over storage items and presents them as multiple brackets, with each
//! having a FIFO queue. This allows an implementation of a matchmaker over different
//! ranking brackets.
//!
//! Note: You might want to introduce a helper function that wraps the complex
//! types and just returns the boxed trait object.
use core::marker::PhantomData;
use frame_support::storage::{StorageDoubleMap, StorageMap, StorageValue};
use parity_scale_codec::{Codec, EncodeLike};
use sp_std::vec::Vec;

/// Trait object presenting the brackets interface.
pub trait BracketsTrait<ItemKey, Item>
where
	ItemKey: Codec + EncodeLike,
	Item: Codec + EncodeLike,
{
	/// Store all changes made in the underlying storage.
	///
	/// Data is not guaranteed to be consistent before this call.
	///
	/// Implementation note: Call in `drop` to increase ergonomics.
	fn commit(&self);
	/// Push an item onto the end of the queue.
	fn push(&mut self, b: Bracket, j: ItemKey, i: Item) -> bool;
	/// Pop an item from the start of the queue.
	///
	/// Returns `None` if the queue is empty.
	fn pop(&mut self, b: Bracket) -> Option<Item>;
	/// Return whether the queue is empty.
	fn is_empty(&self, b: Bracket) -> bool;
	/// Return the size of the brackets queue.
	fn size(&self, b: Bracket) -> BufferIndex;
	/// Return whether the item_key is queued or not.
	fn is_queued(&self, j: ItemKey) -> bool;
}

// There is no equivalent trait in std so we create one.
pub trait WrappingOps {
	fn wrapping_add(self, rhs: Self) -> Self;
	fn wrapping_sub(self, rhs: Self) -> Self;
}

macro_rules! impl_wrapping_ops {
	($type:ty) => {
		impl WrappingOps for $type {
			fn wrapping_add(self, rhs: Self) -> Self {
				self.wrapping_add(rhs)
			}
			fn wrapping_sub(self, rhs: Self) -> Self {
				self.wrapping_sub(rhs)
			}
		}
	};
}

impl_wrapping_ops!(u8);
impl_wrapping_ops!(u16);
impl_wrapping_ops!(u32);
impl_wrapping_ops!(u64);

pub type BufferIndex = u16;
pub type BufferIndexVector = Vec<(BufferIndex, BufferIndex)>;
pub type Bracket = u8;

/// Transient backing data that is the backbone of the trait object.
pub struct BracketsTransient<ItemKey, Item, C, B, M, N>
where
	ItemKey: Codec + EncodeLike,
	Item: Codec + EncodeLike,
	C: StorageValue<Bracket, Query = Bracket>,
	B: StorageMap<Bracket, (BufferIndex, BufferIndex), Query = (BufferIndex, BufferIndex)>,
	M: StorageDoubleMap<Bracket, BufferIndex, ItemKey, Query = Option<ItemKey>>,
	N: StorageDoubleMap<Bracket, ItemKey, Item, Query = Option<Item>>,
{
	index_vector: BufferIndexVector,
	_phantom: PhantomData<(ItemKey, Item, C, B, M, N)>,
}

impl<ItemKey, Item, C, B, M, N> BracketsTransient<ItemKey, Item, C, B, M, N>
where
	ItemKey: Codec + EncodeLike,
	Item: Codec + EncodeLike,
	C: StorageValue<Bracket, Query = Bracket>,
	B: StorageMap<Bracket, (BufferIndex, BufferIndex), Query = (BufferIndex, BufferIndex)>,
	M: StorageDoubleMap<Bracket, BufferIndex, ItemKey, Query = Option<ItemKey>>,
	N: StorageDoubleMap<Bracket, ItemKey, Item, Query = Option<Item>>,
{
	/// Create a new `BracketsTransient` that backs the brackets implementation.
	///
	/// Initializes itself from the bounds storage `B`.
	pub fn new() -> BracketsTransient<ItemKey, Item, C, B, M, N> {
		// get brackets count
		let brackets_count = C::get();

		// initialize all brackets
		let mut index_vector = Vec::new();
		for i in 0..brackets_count {
			let (start, end) = B::get(i);
			index_vector.push((start, end));
		}

		BracketsTransient { index_vector, _phantom: PhantomData }
	}
}

impl<ItemKey, Item, C, B, M, N> Drop for BracketsTransient<ItemKey, Item, C, B, M, N>
where
	ItemKey: Codec + EncodeLike,
	Item: Codec + EncodeLike,
	C: StorageValue<Bracket, Query = Bracket>,
	B: StorageMap<Bracket, (BufferIndex, BufferIndex), Query = (BufferIndex, BufferIndex)>,
	M: StorageDoubleMap<Bracket, BufferIndex, ItemKey, Query = Option<ItemKey>>,
	N: StorageDoubleMap<Bracket, ItemKey, Item, Query = Option<Item>>,
{
	/// Commit on `drop`.
	fn drop(&mut self) {
		<Self as BracketsTrait<ItemKey, Item>>::commit(self);
	}
}

/// Brackets implementation based on `BracketsTransient`
impl<ItemKey, Item, C, B, M, N> BracketsTrait<ItemKey, Item>
	for BracketsTransient<ItemKey, Item, C, B, M, N>
where
	ItemKey: Codec + EncodeLike,
	Item: Codec + EncodeLike,
	C: StorageValue<Bracket, Query = Bracket>,
	B: StorageMap<Bracket, (BufferIndex, BufferIndex), Query = (BufferIndex, BufferIndex)>,
	M: StorageDoubleMap<Bracket, BufferIndex, ItemKey, Query = Option<ItemKey>>,
	N: StorageDoubleMap<Bracket, ItemKey, Item, Query = Option<Item>>,
{
	/// Commit the (potentially) changed bounds to storage.
	fn commit(&self) {
		// commit indicies on all brackets
		for i in 0..self.index_vector.len() {
			let (v_start, v_end) = self.index_vector[i];
			B::insert(i as Bracket, (v_start, v_end));
		}
	}

	/// Push an item onto the end of the queue.
	///
	/// Will insert the new item, but will not update the bounds in storage.
	fn push(&mut self, bracket: Bracket, item_key: ItemKey, item: Item) -> bool {
		let (mut v_start, mut v_end) = self.index_vector[bracket as usize];

		// ensure that the key is not queued
		for i in 0..self.index_vector.len() {
			if N::contains_key(i as Bracket, &item_key) {
				return false;
			}
		}

		// insert the item key and the item
		N::insert(bracket, &item_key, item);
		M::insert(bracket, v_end, item_key);

		// this will intentionally overflow and wrap around when bonds_end
		// reaches `Index::max_value` because we want a brackets.
		let next_index = v_end.wrapping_add(1 as u16);
		if next_index == v_start {
			// queue presents as empty but is not
			// --> overwrite the oldest item in the FIFO brackets
			v_start = v_start.wrapping_add(1 as u16);
		}
		v_end = next_index;

		self.index_vector[bracket as usize] = (v_start, v_end);
		true
	}

	/// Pop an item from the start of the queue.
	///
	/// Will remove the item, but will not update the bounds in storage.
	fn pop(&mut self, bracket: Bracket) -> Option<Item> {
		if self.is_empty(bracket) {
			return None;
		}

		let (mut v_start, v_end) = self.index_vector[bracket as usize];

		M::take(bracket, v_start)
			.and_then(|item_key| N::take(bracket, item_key))
			.map(|item| {
				v_start = v_start.wrapping_add(1 as u16);
				self.index_vector[bracket as usize] = (v_start, v_end);
				item
			})
	}

	/// Return whether to consider the queue empty.
	fn is_empty(&self, bracket: Bracket) -> bool {
		let (v_start, v_end) = self.index_vector[bracket as usize];

		v_start == v_end
	}

	/// Return the current size of the ring buffer as a BufferIndex.
	fn size(&self, bracket: Bracket) -> BufferIndex {
		let (v_start, v_end) = self.index_vector[bracket as usize];

		if v_start <= v_end {
			v_end - v_start
		} else {
			(BufferIndex::MAX - v_start) + v_end
		}
	}

	/// Return whether the item_key is queued or not.
	fn is_queued(&self, item_key: ItemKey) -> bool {
		// check all brackets if key is queued
		for i in 0..self.index_vector.len() {
			if N::contains_key(i as Bracket, &item_key) {
				return true;
			}
		}

		false
	}
}
