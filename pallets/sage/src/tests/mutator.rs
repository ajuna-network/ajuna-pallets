use super::*;

#[test]
fn test_mutator_works() {
	ExtBuilder::default()
		.existential_deposit(100)
		.balances(&[(ALICE, 1000), (BOB, 1000)])
		.build()
		.execute_with(|| {
			run_to_block(10);

			let mutation_id_1 = 0;
			let asset_id_1 = 1;
			let asset_1 =
				Asset { data: MockAssetData { asset_type: 11, asset_subtype: 12 }, created_at: 10 };

			Assets::<Test, Instance1>::insert(asset_id_1, asset_1);

			assert_ok!(SageAlpha::mutate_state(
				RuntimeOrigin::signed(ALICE),
				mutation_id_1,
				vec![asset_id_1]
			));
		});
}
