use super::*;

mod random_hash {
	use super::*;
	use ajuna_primitives::sage_api::SageApi;
	use sp_runtime::traits::Header;

	fn setup_blocks(blocks: u64) {
		let mut parent_hash = System::parent_hash();

		for i in 1..(blocks + 1) {
			System::reset_events();
			System::initialize(&i, &parent_hash, &Default::default());
			Randomness::on_initialize(i);

			let header = System::finalize();
			parent_hash = header.hash();
			System::set_block_number(*header.number());
		}
	}

	#[test]
	fn hashes_with_same_subjects_are_the_same() {
		// This test should mostly highlight a caveat for this function. For the same subject this
		// function returns the same "random" hash during the entirety of a block.
		ExtBuilder::default().build().execute_with(|| {
			setup_blocks(30);

			let hash1 = <TestSageEngine as SageApi>::random_hash(b"hello");
			let hash2 = <TestSageEngine as SageApi>::random_hash(b"hello");

			assert_eq!(hash1, hash2);
		});
	}

	#[test]
	fn hashes_with_different_subjects_are_different() {
		ExtBuilder::default().build().execute_with(|| {
			setup_blocks(30);

			let hash1 = <TestSageEngine as SageApi>::random_hash(b"hello");
			let hash2 = <TestSageEngine as SageApi>::random_hash(b"world");

			assert_ne!(hash1, hash2);
		});
	}

	#[test]
	fn hashes_with_same_subjects_in_different_blocks_are_different() {
		ExtBuilder::default().build().execute_with(|| {
			setup_blocks(30);

			let hash1 = <TestSageEngine as SageApi>::random_hash(b"hello");

			setup_blocks(1);
			let hash2 = <TestSageEngine as SageApi>::random_hash(b"hello");

			assert_ne!(hash1, hash2);
		});
	}
}
