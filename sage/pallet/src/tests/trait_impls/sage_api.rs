use super::*;

mod random_hash {
    use sp_runtime::traits::Header;
    use super::*;
	use ajuna_primitives::sage_api::SageApi;

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
	fn repeated_calls_are_different() {
		ExtBuilder::default()
			.build()
			.execute_with(|| {
                setup_blocks(30);

                let hash1 = <TestSageEngine as SageApi>::random_hash(b"hello");
				let hash2 = <TestSageEngine as SageApi>::random_hash(b"hello");

				assert_ne!(hash1, hash2);
			});
	}
}
