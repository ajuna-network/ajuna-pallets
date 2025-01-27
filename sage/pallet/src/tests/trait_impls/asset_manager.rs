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

const TEST_LOCK_ID: &[u8; 8] = b"testlock";

mod lock_asset {
	use super::*;

	#[test]
	fn can_lock_asset_successfully() {
		ExtBuilder::default()
			.balances(&[(ALICE, 1_000_000)])
			.locks(&[
				(ALICE, SEASON_ID_0, Locks::all_unlocked()),
				(BOB, SEASON_ID_0, Locks::all_unlocked()),
				(Sage::technical_account_id(), SEASON_ID_0, Locks::all_unlocked()),
			])
			.build()
			.execute_with(|| {
				let asset_ids = create_assets::<()>(SEASON_ID_0, ALICE, 1);
				let asset_id = asset_ids[0];
				let expected_lock = Lock { id: *TEST_LOCK_ID, locker: ALICE };

				assert_ok!(<Sage as AssetManager>::lock_asset(*TEST_LOCK_ID, ALICE, asset_id));
				assert!(<Sage as AssetManager>::is_locked(&asset_id).is_some());
				System::assert_has_event(RuntimeEvent::Sage(Event::AssetLocked {
					asset_id,
					lock: expected_lock,
				}));

				// Ensure ownership transferred to technical account
				let technical_account = Sage::technical_account_id();

				assert!(!AssetOwners::<Test, ()>::contains_key((ALICE, SEASON_ID_0, asset_id)));
				assert!(!AssetOwners::<Test, ()>::contains_key((
					technical_account,
					SEASON_ID_0,
					asset_id
				)));
				assert_eq!(Assets::<Test, ()>::get(asset_id).unwrap().0, technical_account);
			});
	}

	#[test]
	fn cannot_lock_unowned_asset() {
		ExtBuilder::default()
			.balances(&[(ALICE, 1_000), (BOB, 1_000)])
			.build()
			.execute_with(|| {
				let asset_ids = create_assets::<()>(SEASON_ID_0, BOB, 1);
				let asset_id = asset_ids[0];
				assert_noop!(
					<Sage as AssetManager>::lock_asset(*TEST_LOCK_ID, ALICE, asset_id),
					Error::<Test, ()>::AssetNotOwned
				);
			});
	}

	#[test]
	fn cannot_lock_asset_on_trade() {
		ExtBuilder::default()
			.organizer(ALICE)
			.balances(&[(ALICE, 1_000)])
			.locks(&[(CHARLIE, SEASON_ID_0, Locks::all_unlocked())])
			.build()
			.execute_with(|| {
				let filter = AssetFilter::Trade(AssetType::Hero);
				assert_ok!(Sage::update_asset_filter(
					RuntimeOrigin::signed(ALICE),
					SEASON_ID_0,
					filter
				));

				let asset_ids = create_assets::<()>(SEASON_ID_0, CHARLIE, 1);
				let asset_id = asset_ids[0];

				assert_ok!(Sage::set_asset_price(RuntimeOrigin::signed(CHARLIE), asset_id, 1_000));
				assert_noop!(
					<Sage as AssetManager>::lock_asset(*TEST_LOCK_ID, CHARLIE, asset_id),
					Error::<Test, ()>::CannotLockAssetInTrade
				);
			});
	}

	#[test]
	fn cannot_lock_already_locked_asset() {
		ExtBuilder::default().balances(&[(ALICE, 1_000)]).build().execute_with(|| {
			let asset_ids = create_assets::<()>(SEASON_ID_0, DAVE, 1);
			let asset_id = asset_ids[0];
			assert_ok!(<Sage as AssetManager>::lock_asset(*TEST_LOCK_ID, DAVE, asset_id));
			assert_noop!(
				<Sage as AssetManager>::lock_asset(
					*TEST_LOCK_ID,
					Sage::technical_account_id(),
					asset_id
				),
				Error::<Test, ()>::AssetLocked
			);
		});
	}
}

mod unlock_asset {
	use super::*;

	#[test]
	fn can_unlock_asset_successfully() {
		ExtBuilder::default().balances(&[(ALICE, 1_000)]).build().execute_with(|| {
			let asset_ids = create_assets::<()>(SEASON_ID_0, ALICE, 1);
			let asset_id = asset_ids[0];
			let expected_lock = Lock { id: *TEST_LOCK_ID, locker: ALICE };

			assert_ok!(<Sage as AssetManager>::lock_asset(*TEST_LOCK_ID, ALICE, asset_id));
			assert_eq!(
				LockedAssets::<Test, ()>::get(asset_id),
				Some(Lock { id: *TEST_LOCK_ID, locker: ALICE })
			);
			assert_ok!(<Sage as AssetManager>::unlock_asset(*TEST_LOCK_ID, ALICE, asset_id));
			System::assert_has_event(RuntimeEvent::Sage(Event::AssetLocked {
				asset_id,
				lock: expected_lock,
			}));
			assert_eq!(LockedAssets::<Test, ()>::get(asset_id), None);
		});
	}

	#[test]
	fn cannot_unlock_non_owned_asset() {
		ExtBuilder::default()
			.balances(&[(ALICE, 1_000), (BOB, 5_000)])
			.build()
			.execute_with(|| {
				let asset_ids = create_assets::<()>(SEASON_ID_0, BOB, 1);
				let asset_id = asset_ids[0];
				assert_ok!(<Sage as AssetManager>::lock_asset(*TEST_LOCK_ID, BOB, asset_id));
				assert_noop!(
					<Sage as AssetManager>::unlock_asset(*TEST_LOCK_ID, ALICE, asset_id),
					Error::<Test, ()>::AssetNotOwned
				);
			});
	}

	#[test]
	fn cannot_unlock_asset_locked_by_other_application() {
		ExtBuilder::default().balances(&[(ALICE, 1_000)]).build().execute_with(|| {
			let asset_ids = create_assets::<()>(SEASON_ID_0, ALICE, 1);
			let asset_id = asset_ids[0];

			let other_lock_id = b"otherapp";
			assert_ok!(<Sage as AssetManager>::lock_asset(*other_lock_id, ALICE, asset_id));

			assert_noop!(
				<Sage as AssetManager>::unlock_asset(*TEST_LOCK_ID, ALICE, asset_id),
				Error::<Test, ()>::AssetLockedByOtherApplication
			);
		});
	}
}

mod asset_funds_manager {
	use super::*;
	use frame_support::assert_err;
	use sp_runtime::{ModuleError, TokenError};

	#[test]
	fn depositing_to_asset_works() {
		ExtBuilder::default().balances(&[(ALICE, 1_000)]).build().execute_with(|| {
			let asset_ids = create_assets::<()>(SEASON_ID_0, ALICE, 1);
			let asset_id = asset_ids[0];
			let asset_balance = 10;

			assert_eq!(
				<Sage as AssetFundsManager>::inspect_asset_funds(&asset_id, &NATIVE_PAYMENT),
				0
			);
			assert_ok!(<Sage as AssetFundsManager>::deposit_funds_to_asset(
				&asset_id,
				&ALICE,
				NATIVE_PAYMENT,
				asset_balance
			));
			assert_eq!(
				<Sage as AssetFundsManager>::inspect_asset_funds(&asset_id, &NATIVE_PAYMENT),
				asset_balance
			);

			// money went from Alice to the asset
			assert_eq!(
				<<Test as Config>::Fungible as fungible::Inspect<_>>::balance(&ALICE),
				1_000 - asset_balance
			);

			// Add more money to see if depositing to existing asset funds works
			assert_ok!(<Sage as AssetFundsManager>::deposit_funds_to_asset(
				&asset_id,
				&ALICE,
				NATIVE_PAYMENT,
				asset_balance
			));
			assert_eq!(
				<Sage as AssetFundsManager>::inspect_asset_funds(&asset_id, &NATIVE_PAYMENT),
				2 * asset_balance
			);

			assert_eq!(
				<<Test as Config>::Fungible as fungible::Inspect<_>>::balance(&ALICE),
				1_000 - 2 * asset_balance
			);
		});
	}

	#[test]
	fn depositing_to_asset_fails_if_missing_funds() {
		ExtBuilder::default().balances(&[(ALICE, 1_000)]).build().execute_with(|| {
			let asset_ids = create_assets::<()>(SEASON_ID_0, ALICE, 1);
			let asset_id = asset_ids[0];
			let asset_balance = 1_000;

			assert_eq!(
				<Sage as AssetFundsManager>::inspect_asset_funds(&asset_id, &NATIVE_PAYMENT),
				0
			);
			assert_err!(
				<Sage as AssetFundsManager>::deposit_funds_to_asset(
					&asset_id,
					&ALICE,
					NATIVE_PAYMENT,
					asset_balance
				),
				TokenError::FundsUnavailable
			);

			// Alice still has all her money
			assert_eq!(
				<<Test as Config>::Fungible as fungible::Inspect<_>>::balance(&ALICE),
				1_000
			);
		});
	}

	#[test]
	fn transfer_funds_from_asset_works() {
		ExtBuilder::default().balances(&[(ALICE, 1_000)]).build().execute_with(|| {
			let asset_ids = create_assets::<()>(SEASON_ID_0, ALICE, 1);
			let ed = <<Test as Config>::Fungible as fungible::Inspect<_>>::minimum_balance();
			let asset_id = asset_ids[0];
			let asset_balance = 10;

			<<Test as Config>::Fungible as fungible::Mutate<_>>::set_balance(
				&Sage::assets_funds_pot(),
				ed,
			);

			assert_eq!(
				<Sage as AssetFundsManager>::inspect_asset_funds(&asset_id, &NATIVE_PAYMENT),
				0
			);
			assert_ok!(<Sage as AssetFundsManager>::deposit_funds_to_asset(
				&asset_id,
				&ALICE,
				NATIVE_PAYMENT,
				asset_balance
			));
			assert_eq!(
				<Sage as AssetFundsManager>::inspect_asset_funds(&asset_id, &NATIVE_PAYMENT),
				asset_balance
			);
			assert_eq!(
				<<Test as Config>::Fungible as fungible::Inspect<_>>::balance(&ALICE),
				1_000 - asset_balance
			);

			assert_ok!(<Sage as AssetFundsManager>::transfer_funds_from_asset(
				&asset_id,
				&ALICE,
				NATIVE_PAYMENT,
				// in this case the account can't be reaped, so we keep the ED.
				asset_balance
			));

			assert_eq!(
				<Sage as AssetFundsManager>::inspect_asset_funds(&asset_id, &NATIVE_PAYMENT),
				0
			);

			assert_eq!(
				<<Test as Config>::Fungible as fungible::Inspect<_>>::balance(&ALICE),
				1_000
			);
		});
	}

	#[test]
	fn transfer_funds_from_asset_cant_remove_more_than_owned() {
		ExtBuilder::default().balances(&[(ALICE, 1_000)]).build().execute_with(|| {
			let asset_ids = create_assets::<()>(SEASON_ID_0, ALICE, 1);
			let ed = <<Test as Config>::Fungible as fungible::Inspect<_>>::minimum_balance();
			let asset_id = asset_ids[0];
			let asset_balance = 10;

			<<Test as Config>::Fungible as fungible::Mutate<_>>::set_balance(
				&Sage::assets_funds_pot(),
				ed,
			);

			assert_eq!(
				<Sage as AssetFundsManager>::inspect_asset_funds(&asset_id, &NATIVE_PAYMENT),
				0
			);
			assert_ok!(<Sage as AssetFundsManager>::deposit_funds_to_asset(
				&asset_id,
				&ALICE,
				NATIVE_PAYMENT,
				asset_balance
			));
			assert_eq!(
				<Sage as AssetFundsManager>::inspect_asset_funds(&asset_id, &NATIVE_PAYMENT),
				asset_balance
			);
			assert_eq!(
				<<Test as Config>::Fungible as fungible::Inspect<_>>::balance(&ALICE),
				1_000 - asset_balance
			);

			assert_err!(
				<Sage as AssetFundsManager>::transfer_funds_from_asset(
					&asset_id,
					&ALICE,
					NATIVE_PAYMENT,
					asset_balance + 1
				),
				DispatchError::Module(ModuleError {
					index: 3,
					error: [16, 0, 0, 0],
					message: Some("AssetsFundsTooLow")
				})
			);

			// Alice did not receive any money as the transfer failed
			assert_eq!(
				<<Test as Config>::Fungible as fungible::Inspect<_>>::balance(&ALICE),
				1_000 - asset_balance
			);
		});
	}

	#[test]
	fn transfer_all_funds_from_asset_works() {
		ExtBuilder::default().balances(&[(ALICE, 1_000)]).build().execute_with(|| {
			let asset_ids = create_assets::<()>(SEASON_ID_0, ALICE, 1);
			let ed = <<Test as Config>::Fungible as fungible::Inspect<_>>::minimum_balance();
			let asset_id = asset_ids[0];
			let asset_balance = 10;

			<<Test as Config>::Fungible as fungible::Mutate<_>>::set_balance(
				&Sage::assets_funds_pot(),
				ed,
			);

			assert_eq!(
				<Sage as AssetFundsManager>::inspect_asset_funds(&asset_id, &NATIVE_PAYMENT),
				0
			);
			assert_ok!(<Sage as AssetFundsManager>::deposit_funds_to_asset(
				&asset_id,
				&ALICE,
				NATIVE_PAYMENT,
				asset_balance
			));
			assert_eq!(
				<Sage as AssetFundsManager>::inspect_asset_funds(&asset_id, &NATIVE_PAYMENT),
				asset_balance
			);

			assert_ok!(<Sage as AssetFundsManager>::transfer_all_from_asset(
				&asset_id,
				&ALICE,
				NATIVE_PAYMENT,
			));

			assert_eq!(
				<Sage as AssetFundsManager>::inspect_asset_funds(&asset_id, &NATIVE_PAYMENT),
				0
			);

			// Alice has now her initial balance
			assert_eq!(
				<<Test as Config>::Fungible as fungible::Inspect<_>>::balance(&ALICE),
				1_000
			);
		});
	}

	#[test]
	fn transfer_all_funds_from_asset_fails_if_missing_funds() {
		ExtBuilder::default().balances(&[(ALICE, 1_000)]).build().execute_with(|| {
			let asset_ids = create_assets::<()>(SEASON_ID_0, ALICE, 1);
			let ed = <<Test as Config>::Fungible as fungible::Inspect<_>>::minimum_balance();
			let asset_id = asset_ids[0];

			<<Test as Config>::Fungible as fungible::Mutate<_>>::set_balance(
				&Sage::assets_funds_pot(),
				ed,
			);

			assert_eq!(
				<Sage as AssetFundsManager>::inspect_asset_funds(&asset_id, &NATIVE_PAYMENT),
				0
			);

			assert_err!(
				<Sage as AssetFundsManager>::transfer_all_from_asset(
					&asset_id,
					&ALICE,
					NATIVE_PAYMENT,
				),
				DispatchError::Module(ModuleError {
					index: 3,
					error: [16, 0, 0, 0],
					message: Some("AssetsFundsTooLow")
				})
			);

			assert_eq!(
				<Sage as AssetFundsManager>::inspect_asset_funds(&asset_id, &NATIVE_PAYMENT),
				0
			);

			// Alice has still her initial balance
			assert_eq!(
				<<Test as Config>::Fungible as fungible::Inspect<_>>::balance(&ALICE),
				1_000
			);
		});
	}
}
