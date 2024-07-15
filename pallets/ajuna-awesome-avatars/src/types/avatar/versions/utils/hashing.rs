use super::*;

pub(crate) struct HashProvider<T: Config, const N: usize> {
	pub(crate) hash: [u8; N],
	current_index: usize,
	_marker: PhantomData<T>,
}

impl<T: Config, const N: usize> Default for HashProvider<T, N> {
	fn default() -> Self {
		Self { hash: [0; N], current_index: 0, _marker: PhantomData }
	}
}

impl<T: Config, const N: usize> HashProvider<T, N> {
	pub fn new(hash: &T::Hash) -> Self {
		Self::new_starting_at(hash, 0)
	}

	#[cfg(test)]
	pub fn new_with_bytes(bytes: [u8; N]) -> Self {
		Self { hash: bytes, current_index: 0, _marker: PhantomData }
	}

	pub fn new_starting_at(hash: &T::Hash, index: usize) -> Self {
		// TODO: Improve
		let mut bytes = [0; N];

		let hash_ref = hash.as_ref();
		let hash_len = hash_ref.len();

		bytes[0..hash_len].copy_from_slice(hash_ref);

		Self { hash: bytes, current_index: index, _marker: PhantomData }
	}

	pub fn full_hash(&self, mutate_seed: usize) -> T::Hash {
		let mut full_hash = self.hash;

		for (i, hash) in full_hash.iter_mut().enumerate() {
			*hash = self.hash[(i + mutate_seed) % N];
		}

		T::Hashing::hash(&full_hash)
	}

	pub fn next(&mut self) -> u8 {
		<Self as Iterator>::next(self).unwrap_or_default()
	}
}

impl<T: Config, const N: usize> Iterator for HashProvider<T, N> {
	type Item = u8;

	fn next(&mut self) -> Option<Self::Item> {
		let item = self.hash[self.current_index];
		self.current_index = (self.current_index + 1) % N;
		Some(item)
	}
}
