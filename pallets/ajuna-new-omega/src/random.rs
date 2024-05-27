// Ajuna Node
// Copyright (C) 2022 BlogaTech AG

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use super::*;

use std::marker::PhantomData;

pub(crate) struct DiceRoller<T: Config, const N: usize, const M: u8> {
	pub(crate) hash_array: [u8; N],
	current_index: usize,
	_marker: PhantomData<T>,
}

impl<T: Config, const N: usize, const M: u8> Default for DiceRoller<T, N, M> {
	fn default() -> Self {
		Self { hash_array: [0; N], current_index: 0, _marker: PhantomData }
	}
}

impl<T: Config, const N: usize, const M: u8> DiceRoller<T, N, M> {
	pub fn new(hash: &T::Hash) -> Self {
		let mut bytes = [0; N];

		let hash_ref = hash.as_ref();
		let hash_len = hash_ref.len();

		bytes[0..hash_len].copy_from_slice(hash_ref);

		Self { hash_array: bytes, current_index: 0, _marker: PhantomData }
	}

	pub fn next(&mut self) -> u8 {
		<Self as Iterator>::next(self).unwrap_or_default()
	}

	pub fn next_seed(&mut self) -> u64 {
		u64::from_ne_bytes([
			self.next(),
			self.next(),
			self.next(),
			self.next(),
			self.next(),
			self.next(),
			self.next(),
			self.next(),
		])
	}
}

impl<T: Config, const N: usize, const M: u8> Iterator for DiceRoller<T, N, M> {
	type Item = u8;

	fn next(&mut self) -> Option<Self::Item> {
		let item = self.hash_array[self.current_index];
		self.current_index = (self.current_index + 1) % N;
		Some(item % M)
	}
}
