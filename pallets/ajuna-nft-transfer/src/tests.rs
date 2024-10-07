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

use crate::{mock::*, traits::*, Error, *};
use frame_support::{
	assert_err, assert_noop, assert_ok,
	traits::tokens::nonfungibles_v2::{Create, Inspect},
};
use frame_system::pallet_prelude::BlockNumberFor;
use parity_scale_codec::Encode;
use sp_runtime::{testing::H256, DispatchError};

type CollectionConfig =
	pallet_nfts::CollectionConfig<MockBalance, BlockNumberFor<Test>, MockCollectionId>;

fn create_collection(organizer: MockAccountId) -> MockCollectionId {
	<Test as Config>::NftHelper::create_collection(
		&organizer,
		&NftTransfer::account_id(),
		&CollectionConfig::default(),
	)
	.expect("Should have create contract collection")
}

mod ipfs {
	use super::*;
	use sp_runtime::DispatchError;

	#[test]
	fn set_service_account_works() {
		ExtBuilder::default().build().execute_with(|| {
			assert_eq!(ServiceAccount::<Test>::get(), None);
			assert_ok!(Nft::set_service_account(RuntimeOrigin::root(), ALICE));
			assert_eq!(ServiceAccount::<Test>::get(), Some(ALICE));
			System::assert_last_event(mock::RuntimeEvent::Nft(crate::Event::ServiceAccountSet {
				service_account: ALICE,
			}));
		});
	}

	#[test]
	fn set_service_account_rejects_non_root_calls() {
		ExtBuilder::default().build().execute_with(|| {
			assert_noop!(
				Nft::set_service_account(RuntimeOrigin::signed(BOB), ALICE),
				DispatchError::BadOrigin
			);
		});
	}

	#[test]
	fn prepare_avatar_works() {
		let prepare_fee = 999;
		let initial_balance = prepare_fee + MockExistentialDeposit::get();

		ExtBuilder::default()
			.balances(&[(ALICE, initial_balance)])
			.build()
			.execute_with(|| {
				let avatar_id = MockAssetManager::create_assets(ALICE, 1)[0];
				assert_ok!(AAvatars::set_service_account(RuntimeOrigin::root(), BOB));
				assert_eq!(Balances::free_balance(ALICE), initial_balance);
				assert_ok!(AAvatars::prepare_avatar(RuntimeOrigin::signed(ALICE), avatar_id));
				assert_eq!(Balances::free_balance(ALICE), initial_balance - prepare_fee);
				assert_eq!(Preparation::<Test>::get(avatar_id).unwrap().to_vec(), Vec::<u8>::new());
				System::assert_last_event(mock::RuntimeEvent::AAvatars(
					crate::Event::PreparedAvatar { avatar_id },
				));
			});
	}

	#[test]
	fn prepare_avatar_rejects_forging_trading_and_transferring() {
		ExtBuilder::default().build().execute_with(|| {
			let avatar_ids = MockAssetManager::create_assets(ALICE, 5);
			let avatar_id = avatar_ids[0];
			assert_ok!(AAvatars::set_service_account(RuntimeOrigin::root(), ALICE));
			assert_ok!(AAvatars::prepare_avatar(RuntimeOrigin::signed(ALICE), avatar_id));

			for extrinsic in [
				AAvatars::set_price(RuntimeOrigin::signed(ALICE), avatar_id, 1_000),
				AAvatars::transfer_avatar(RuntimeOrigin::signed(ALICE), BOB, avatar_id),
				AAvatars::forge(RuntimeOrigin::signed(ALICE), avatar_id, avatar_ids[1..3].to_vec()),
			] {
				assert_noop!(extrinsic, Error::<Test>::AlreadyPrepared);
			}
		});
	}

	#[test]
	fn prepare_avatar_rejects_unsigned_calls() {
		ExtBuilder::default().build().execute_with(|| {
			let avatar_id = MockAssetManager::create_assets(ALICE, 1)[0];
			assert_noop!(
				AAvatars::prepare_avatar(RuntimeOrigin::none(), avatar_id),
				DispatchError::BadOrigin
			);
		});
	}

	#[test]
	fn prepare_avatar_rejects_unowned_avatars() {
		ExtBuilder::default().build().execute_with(|| {
			let avatar_id = MockAssetManager::create_assets(ALICE, 1)[0];
			assert_noop!(
				AAvatars::prepare_avatar(RuntimeOrigin::signed(BOB), avatar_id),
				Error::<Test>::Ownership
			);
		});
	}

	#[test]
	fn prepare_avatar_rejects_avatars_in_trade() {
		ExtBuilder::default()
			.trade_filters(&[(SEASON_ID, TradeFilters::default())])
			.locks(&[(ALICE, SEASON_ID, Locks::all_unlocked())])
			.build()
			.execute_with(|| {
				let avatar_id = MockAssetManager::create_assets(ALICE, 1)[0];
				assert_ok!(AAvatars::set_price(RuntimeOrigin::signed(ALICE), avatar_id, 1));
				assert_noop!(
					AAvatars::prepare_avatar(RuntimeOrigin::signed(ALICE), avatar_id),
					Error::<Test>::AvatarInTrade
				);
			});
	}

	#[test]
	fn prepare_avatar_rejects_when_closed() {
		ExtBuilder::default().build().execute_with(|| {
			let avatar_id = MockAssetManager::create_assets(ALICE, 1)[0];
			GlobalConfigs::<Test>::mutate(|config| config.nft_transfer.open = false);
			assert_noop!(
				AAvatars::prepare_avatar(RuntimeOrigin::signed(ALICE), avatar_id),
				Error::<Test>::NftTransferClosed
			);
		});
	}

	#[test]
	fn prepare_avatar_rejects_locked_avatars() {
		ExtBuilder::default().build().execute_with(|| {
			let avatar_id = MockAssetManager::create_assets(ALICE, 1)[0];
			LockedAvatars::<Test>::insert(avatar_id, ());
			assert_noop!(
				AAvatars::prepare_avatar(RuntimeOrigin::signed(ALICE), avatar_id),
				Error::<Test>::AvatarLocked
			);
		});
	}

	#[test]
	fn prepare_avatar_rejects_already_prepared_avatars() {
		ExtBuilder::default().build().execute_with(|| {
			let avatar_id = MockAssetManager::create_assets(ALICE, 1)[0];
			let ipfs_url = IpfsUrl::try_from(Vec::new()).unwrap();
			Preparation::<Test>::insert(avatar_id, ipfs_url);
			assert_noop!(
				AAvatars::prepare_avatar(RuntimeOrigin::signed(ALICE), avatar_id),
				Error::<Test>::AlreadyPrepared
			);
		});
	}

	#[test]
	fn prepare_avatar_rejects_insufficient_balance() {
		ExtBuilder::default()
			.balances(&[(ALICE, MockExistentialDeposit::get()), (BOB, 999_999)])
			.build()
			.execute_with(|| {
				let avatar_id = MockAssetManager::create_assets(ALICE, 1)[0];
				assert_ok!(AAvatars::set_service_account(RuntimeOrigin::root(), BOB));
				assert_noop!(
					AAvatars::prepare_avatar(RuntimeOrigin::signed(ALICE), avatar_id),
					sp_runtime::TokenError::FundsUnavailable
				);
			});
	}

	#[test]
	fn unprepare_avatar_works() {
		ExtBuilder::default().build().execute_with(|| {
			let avatar_id = MockAssetManager::create_assets(ALICE, 1)[0];
			assert_ok!(AAvatars::set_service_account(RuntimeOrigin::root(), ALICE));
			assert_ok!(AAvatars::prepare_avatar(RuntimeOrigin::signed(ALICE), avatar_id));
			assert_ok!(AAvatars::unprepare_avatar(RuntimeOrigin::signed(ALICE), avatar_id));
			assert!(!Preparation::<Test>::contains_key(avatar_id));
			System::assert_last_event(mock::RuntimeEvent::AAvatars(
				crate::Event::UnpreparedAvatar { avatar_id },
			));
		});
	}

	#[test]
	fn unprepare_avatar_rejects_unsigned_calls() {
		ExtBuilder::default().build().execute_with(|| {
			assert_noop!(
				AAvatars::unprepare_avatar(RuntimeOrigin::none(), H256::random()),
				DispatchError::BadOrigin
			);
		});
	}

	#[test]
	fn unprepare_avatar_rejects_unowned_avatars() {
		ExtBuilder::default().build().execute_with(|| {
			let avatar_id = MockAssetManager::create_assets(ALICE, 1)[0];
			assert_noop!(
				AAvatars::unprepare_avatar(RuntimeOrigin::signed(BOB), avatar_id),
				Error::<Test>::Ownership
			);
		});
	}

	#[test]
	fn unprepare_avatar_rejects_when_closed() {
		ExtBuilder::default().build().execute_with(|| {
			let avatar_id = MockAssetManager::create_assets(ALICE, 1)[0];
			GlobalConfigs::<Test>::mutate(|config| config.nft_transfer.open = false);
			assert_noop!(
				AAvatars::unprepare_avatar(RuntimeOrigin::signed(BOB), avatar_id),
				Error::<Test>::Ownership
			);
		});
	}

	#[test]
	fn unprepare_avatar_rejects_unprepared_avatars() {
		ExtBuilder::default().build().execute_with(|| {
			let avatar_id = MockAssetManager::create_assets(ALICE, 1)[0];
			assert_noop!(
				AAvatars::unprepare_avatar(RuntimeOrigin::signed(ALICE), avatar_id),
				Error::<Test>::NotPrepared
			);
		});
	}

	#[test]
	fn prepare_ipfs_works() {
		ExtBuilder::default().build().execute_with(|| {
			let avatar_id = MockAssetManager::create_assets(ALICE, 1)[0];
			assert_ok!(AAvatars::set_service_account(RuntimeOrigin::root(), ALICE));
			assert_ok!(AAvatars::prepare_avatar(RuntimeOrigin::signed(ALICE), avatar_id));
			ServiceAccount::<Test>::put(BOB);

			let ipfs_url = b"ipfs://{CID}/{optional path to resource}".to_vec();
			let ipfs_url = IpfsUrl::try_from(ipfs_url).unwrap();
			assert_ok!(AAvatars::prepare_ipfs(
				RuntimeOrigin::signed(BOB),
				avatar_id,
				ipfs_url.clone()
			));
			assert_eq!(Preparation::<Test>::get(avatar_id).unwrap(), ipfs_url);
			System::assert_last_event(mock::RuntimeEvent::AAvatars(
				crate::Event::PreparedIpfsUrl { url: ipfs_url },
			));

			let ipfs_url = b"ipfs://123".to_vec();
			let ipfs_url = IpfsUrl::try_from(ipfs_url).unwrap();
			assert_ok!(AAvatars::prepare_ipfs(
				RuntimeOrigin::signed(BOB),
				avatar_id,
				ipfs_url.clone()
			));
			assert_eq!(Preparation::<Test>::get(avatar_id).unwrap(), ipfs_url);
			System::assert_last_event(mock::RuntimeEvent::AAvatars(
				crate::Event::PreparedIpfsUrl { url: ipfs_url },
			));
		});
	}

	#[test]
	fn prepare_ipfs_rejects_empty_url() {
		ExtBuilder::default().build().execute_with(|| {
			let avatar_id = MockAssetManager::create_assets(ALICE, 1)[0];
			assert_ok!(AAvatars::set_service_account(RuntimeOrigin::root(), ALICE));
			assert_ok!(AAvatars::prepare_avatar(RuntimeOrigin::signed(ALICE), avatar_id));
			ServiceAccount::<Test>::put(BOB);

			assert_noop!(
				AAvatars::prepare_ipfs(RuntimeOrigin::signed(BOB), avatar_id, IpfsUrl::default()),
				Error::<Test>::EmptyIpfsUrl
			);
		});
	}
}

mod store_as_nft {
	use super::*;
	use sp_runtime::traits::Get;

	#[test]
	fn can_store_item_successfully() {
		ExtBuilder::default()
			.balances(&[(ALICE, CollectionDeposit::get() + 999), (BOB, ItemDeposit::get() + 999)])
			.build()
			.execute_with(|| {
				let collection_id = create_collection(ALICE);
				let item_id = H256::random();
				let item = MockItem::default();
				let url = b"ipfs://test".to_vec();

				assert_ok!(NftTransfer::store_as_nft(
					BOB,
					collection_id,
					item_id,
					item.clone(),
					url.clone().try_into().unwrap(),
				));
				assert_eq!(Nft::collection_owner(collection_id), Some(ALICE));
				assert_eq!(Nft::owner(collection_id, item_id), Some(BOB));
				assert_eq!(
					Nft::system_attribute(&collection_id, Some(&item_id), MockItem::ITEM_CODE),
					Some(item.encode())
				);
				assert_eq!(
					Nft::system_attribute(&collection_id, Some(&item_id), MockItem::IPFS_URL_CODE),
					Some(url)
				);
				for (attribute_code, encoded_attributes) in item.get_encoded_attributes() {
					assert_eq!(
						Nft::system_attribute(&collection_id, Some(&item_id), &attribute_code),
						Some(encoded_attributes.to_vec())
					);
				}
				assert_eq!(
					NftStatuses::<Test>::get(collection_id, item_id),
					Some(NftStatus::Stored)
				);

				// check players pay for the item deposit
				assert_eq!(Balances::free_balance(BOB), 999);

				System::assert_last_event(mock::RuntimeEvent::NftTransfer(
					crate::Event::ItemStored { collection_id, item_id, owner: BOB },
				));
			});
	}

	#[test]
	fn can_store_item_and_transfer() {
		ExtBuilder::default()
			.balances(&[(ALICE, CollectionDeposit::get() + 999), (BOB, ItemDeposit::get() + 999)])
			.build()
			.execute_with(|| {
				let collection_id = create_collection(ALICE);
				let item_id = H256::random();
				let item = MockItem::default();
				let url = b"ipfs://test".to_vec();

				assert_ok!(NftTransfer::store_as_nft(
					BOB,
					collection_id,
					item_id,
					item.clone(),
					url.clone().try_into().unwrap(),
				));

				System::assert_last_event(mock::RuntimeEvent::NftTransfer(
					crate::Event::ItemStored { collection_id, item_id, owner: BOB },
				));

				assert_eq!(Nft::collection_owner(collection_id), Some(ALICE));
				assert_eq!(Nft::owner(collection_id, item_id), Some(BOB));
				assert_eq!(
					Nft::system_attribute(&collection_id, Some(&item_id), MockItem::ITEM_CODE),
					Some(item.encode())
				);
				assert_eq!(
					Nft::system_attribute(&collection_id, Some(&item_id), MockItem::IPFS_URL_CODE),
					Some(url)
				);

				assert_ok!(Nft::do_transfer(collection_id, item_id, ALICE, |_, _| { Ok(()) }));

				assert_eq!(Nft::owner(collection_id, item_id), Some(ALICE));

				System::assert_last_event(mock::RuntimeEvent::Nft(
					pallet_nfts::Event::Transferred {
						collection: collection_id,
						item: item_id,
						from: BOB,
						to: ALICE,
					},
				));
			});
	}

	#[test]
	fn cannot_store_empty_ipfs_url() {
		ExtBuilder::default()
			.balances(&[(ALICE, CollectionDeposit::get() + ItemDeposit::get() + 999)])
			.build()
			.execute_with(|| {
				let collection_id = create_collection(ALICE);
				let item_id = H256::random();
				let item = MockItem::default();
				let url = vec![];

				assert_err!(
					NftTransfer::store_as_nft(
						ALICE,
						collection_id,
						item_id,
						item,
						url.try_into().unwrap()
					),
					Error::<Test>::EmptyIpfsUrl
				);
			});
	}

	#[test]
	fn cannot_store_duplicates_under_same_collection() {
		ExtBuilder::default()
			.balances(&[(ALICE, CollectionDeposit::get() + ItemDeposit::get() + 999)])
			.build()
			.execute_with(|| {
				let collection_id = create_collection(ALICE);
				let item_id = H256::random();
				let item = MockItem::default();
				let url = b"ipfs://test".to_vec();

				assert_ok!(NftTransfer::store_as_nft(
					ALICE,
					collection_id,
					item_id,
					item.clone(),
					url.clone().try_into().unwrap()
				));
				assert_noop!(
					NftTransfer::store_as_nft(
						ALICE,
						collection_id,
						item_id,
						item,
						url.try_into().unwrap()
					),
					pallet_nfts::Error::<Test>::AlreadyExists
				);
			});
	}

	#[test]
	fn cannot_store_item_above_encoding_limit() {
		ExtBuilder::default()
			.balances(&[(ALICE, CollectionDeposit::get() + 999), (BOB, ItemDeposit::get() + 999)])
			.build()
			.execute_with(|| {
				let collection_id = create_collection(ALICE);
				let item_id = H256::random();
				let item = MockItem {
					field_1: vec![1; ValueLimit::get() as usize],
					field_2: 1,
					field_3: false,
				};
				let url = b"ipfs://test".to_vec();

				assert!(item.encode().len() > ValueLimit::get() as usize);
				// NOTE: As long as the execution is wrapped in an extrinsic, this is a noop.
				assert_err!(
					NftTransfer::store_as_nft(
						BOB,
						collection_id,
						item_id,
						item,
						url.try_into().unwrap()
					),
					pallet_nfts::Error::<Test>::IncorrectData
				);
			});
	}
}

mod recover_from_nft {
	use super::*;

	#[test]
	fn can_recover_item_successfully() {
		let initial_balance = ItemDeposit::get() + 999;
		ExtBuilder::default()
			.balances(&[(ALICE, CollectionDeposit::get() + 999), (BOB, initial_balance)])
			.build()
			.execute_with(|| {
				let collection_id = create_collection(ALICE);
				let item_id = H256::random();
				let item = MockItem::default();
				let url = b"ipfs://test".to_vec();

				assert_ok!(NftTransfer::store_as_nft(
					BOB,
					collection_id,
					item_id,
					item.clone(),
					url.try_into().unwrap()
				));
				assert_eq!(Balances::free_balance(BOB), 999);

				assert_eq!(NftTransfer::recover_from_nft(BOB, collection_id, item_id), Ok(item));
				assert!(NftStatuses::<Test>::get(collection_id, item_id).is_none());
				assert!(Nft::system_attribute(
					&collection_id,
					Some(&item_id),
					&MockItem::ITEM_CODE.encode()
				)
				.is_none());
				assert!(Nft::system_attribute(
					&collection_id,
					Some(&item_id),
					&MockItem::IPFS_URL_CODE.encode()
				)
				.is_none());
				for attribute_code in MockItem::get_attribute_codes() {
					assert!(Nft::attribute(&collection_id, &item_id, &attribute_code.encode())
						.is_none());
				}

				// check players are refunded the item deposit
				assert_eq!(Balances::free_balance(BOB), initial_balance);

				System::assert_last_event(mock::RuntimeEvent::NftTransfer(
					crate::Event::ItemRestored { collection_id, item_id, owner: BOB },
				));
			});
	}

	#[test]
	fn cannot_restore_uploaded_item() {
		ExtBuilder::default()
			.balances(&[(ALICE, CollectionDeposit::get() + 999), (BOB, ItemDeposit::get() + 999)])
			.build()
			.execute_with(|| {
				let collection_id = create_collection(ALICE);
				let item_id = H256::random();
				let item = MockItem::default();
				let url = b"ipfs://test".to_vec();

				assert_ok!(NftTransfer::store_as_nft(
					BOB,
					collection_id,
					item_id,
					item,
					url.try_into().unwrap()
				));
				NftStatuses::<Test>::insert(collection_id, item_id, NftStatus::Uploaded);

				assert_noop!(
					NftTransfer::recover_from_nft(BOB, collection_id, item_id)
						as Result<MockItem, _>,
					Error::<Test>::NftOutsideOfChain
				);
			});
	}

	#[test]
	fn cannot_restore_if_not_owned() {
		ExtBuilder::default()
			.balances(&[(ALICE, CollectionDeposit::get() + 999), (BOB, ItemDeposit::get() + 999)])
			.build()
			.execute_with(|| {
				let collection_id = create_collection(ALICE);
				let item_id = H256::random();
				let item = MockItem::default();
				let url = b"ipfs://test".to_vec();

				assert_ok!(NftTransfer::store_as_nft(
					BOB,
					collection_id,
					item_id,
					item,
					url.try_into().unwrap()
				));

				// NOTE: As long as the execution is wrapped in an extrinsic, this is a noop.
				assert_err!(
					NftTransfer::recover_from_nft(ALICE, collection_id, item_id)
						as Result<MockItem, _>,
					pallet_nfts::Error::<Test>::NoPermission
				);
			});
	}
}
