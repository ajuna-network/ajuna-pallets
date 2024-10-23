use crate::{mock::*, *};
use frame_support::{assert_noop, assert_ok};
use sp_runtime::bounded_vec;

mod extrinsic {
	use super::*;

	#[test]
	fn add_affiliate_to_account() {
		let initial_balance = 1_000_000;
		ExtBuilder::default()
			.balances(&[(ALICE, initial_balance)])
			.affiliators(&[ALICE])
			.build()
			.execute_with(|| {
				assert_eq!(
					Affiliators::<Test, AffiliatesInstance1>::get(ALICE),
					AffiliatorState { status: AffiliatableStatus::Affiliatable(0), affiliates: 0 }
				);

				assert_ok!(AffiliatesAlpha::add_affiliation(RuntimeOrigin::signed(BOB), None, 0));

				System::assert_last_event(mock::RuntimeEvent::AffiliatesAlpha(
					Event::AccountAffiliated { account: BOB, to: ALICE },
				));

				assert_eq!(
					Affiliatees::<Test, AffiliatesInstance1>::get(BOB),
					Some(bounded_vec![ALICE])
				);
				assert_eq!(
					Affiliators::<Test, AffiliatesInstance1>::get(ALICE),
					AffiliatorState { status: AffiliatableStatus::Affiliatable(0), affiliates: 1 }
				);
			});
	}

	#[test]
	fn add_affiliate_to_another_account() {
		let initial_balance = 1_000_000;
		ExtBuilder::default()
			.balances(&[(ALICE, initial_balance)])
			.affiliators(&[ALICE])
			.build()
			.execute_with(|| {
				MockAccountManager::try_add_to_whitelist(&AffiliateWhitelistKey::get(), &CHARLIE)
					.expect("Should be whitelisted");

				assert_ok!(AffiliatesAlpha::add_affiliation(
					RuntimeOrigin::signed(CHARLIE),
					Some(BOB),
					0
				));

				System::assert_last_event(mock::RuntimeEvent::AffiliatesAlpha(
					Event::AccountAffiliated { account: BOB, to: ALICE },
				));

				assert_eq!(
					Affiliatees::<Test, AffiliatesInstance1>::get(BOB),
					Some(bounded_vec![ALICE])
				);
				assert_eq!(
					Affiliators::<Test, AffiliatesInstance1>::get(ALICE),
					AffiliatorState { status: AffiliatableStatus::Affiliatable(0), affiliates: 1 }
				);
			});
	}

	#[test]
	fn cannot_affiliate_to_non_enabled_account() {
		let initial_balance = 1_000_000;
		ExtBuilder::default()
			.balances(&[(ALICE, initial_balance)])
			.build()
			.execute_with(|| {
				assert_eq!(
					Affiliators::<Test, AffiliatesInstance1>::get(ALICE),
					AffiliatorState { status: AffiliatableStatus::NonAffiliatable, affiliates: 0 }
				);

				assert_noop!(
					AffiliatesAlpha::add_affiliation(RuntimeOrigin::signed(BOB), None, 0),
					Error::<Test, AffiliatesInstance1>::AffiliatorNotFound
				);
			});
	}

	#[test]
	fn cannot_affiliate_another_account_if_not_in_whitelist() {
		let initial_balance = 1_000_000;
		ExtBuilder::default()
			.balances(&[(ALICE, initial_balance)])
			.affiliators(&[ALICE])
			.build()
			.execute_with(|| {
				assert_eq!(
					Affiliators::<Test, AffiliatesInstance1>::get(ALICE),
					AffiliatorState { status: AffiliatableStatus::Affiliatable(0), affiliates: 0 }
				);

				assert_noop!(
					AffiliatesAlpha::add_affiliation(RuntimeOrigin::signed(CHARLIE), Some(BOB), 0),
					Error::<Test, AffiliatesInstance1>::AffiliateOthersOnlyWhiteListed
				);
			});
	}

	#[test]
	fn remove_affiliate_chain() {
		let initial_balance = 1_000_000;
		ExtBuilder::default()
			.balances(&[(ALICE, initial_balance)])
			.organizer(ALICE)
			.affiliators(&[ALICE])
			.build()
			.execute_with(|| {
				assert_eq!(
					Affiliators::<Test, AffiliatesInstance1>::get(ALICE),
					AffiliatorState { status: AffiliatableStatus::Affiliatable(0), affiliates: 0 }
				);

				assert_ok!(AffiliatesAlpha::add_affiliation(RuntimeOrigin::signed(BOB), None, 0));
				assert_eq!(
					Affiliatees::<Test, AffiliatesInstance1>::get(BOB),
					Some(bounded_vec![ALICE])
				);
				assert_eq!(
					Affiliators::<Test, AffiliatesInstance1>::get(ALICE),
					AffiliatorState { status: AffiliatableStatus::Affiliatable(0), affiliates: 1 }
				);

				assert_ok!(AffiliatesAlpha::remove_affiliation(RuntimeOrigin::signed(ALICE), BOB));
				assert_eq!(Affiliatees::<Test, AffiliatesInstance1>::get(BOB), None);
				assert_eq!(
					Affiliators::<Test, AffiliatesInstance1>::get(ALICE),
					AffiliatorState { status: AffiliatableStatus::Affiliatable(0), affiliates: 0 }
				);
			});
	}
	#[test]
	fn remove_affiliate_only_allowed_to_organizer() {
		let initial_balance = 1_000_000;
		ExtBuilder::default()
			.balances(&[(ALICE, initial_balance)])
			.organizer(ALICE)
			.affiliators(&[ALICE])
			.build()
			.execute_with(|| {
				assert_eq!(
					Affiliators::<Test, AffiliatesInstance1>::get(ALICE),
					AffiliatorState { status: AffiliatableStatus::Affiliatable(0), affiliates: 0 }
				);

				assert_ok!(AffiliatesAlpha::add_affiliation(RuntimeOrigin::signed(BOB), None, 0));
				assert_eq!(
					Affiliatees::<Test, AffiliatesInstance1>::get(BOB),
					Some(bounded_vec![ALICE])
				);
				assert_eq!(
					Affiliators::<Test, AffiliatesInstance1>::get(ALICE),
					AffiliatorState { status: AffiliatableStatus::Affiliatable(0), affiliates: 1 }
				);

				// CHARLIE cannot remove affiliation since he is not the organizer
				assert_noop!(
					AffiliatesAlpha::remove_affiliation(RuntimeOrigin::signed(CHARLIE), BOB),
					DispatchError::Other(ACCOUNT_IS_NOT_ORGANIZER)
				);

				// ALICE can because she is set as the organizer
				assert_ok!(AffiliatesAlpha::remove_affiliation(RuntimeOrigin::signed(ALICE), BOB));
				assert_eq!(Affiliatees::<Test, AffiliatesInstance1>::get(BOB), None);
				assert_eq!(
					Affiliators::<Test, AffiliatesInstance1>::get(ALICE),
					AffiliatorState { status: AffiliatableStatus::Affiliatable(0), affiliates: 0 }
				);
			});
	}

	#[test]
	fn set_rule_works() {
		let initial_balance = 1_000_000;
		ExtBuilder::default()
			.balances(&[(ALICE, initial_balance)])
			.organizer(ALICE)
			.build()
			.execute_with(|| {
				let rule =
					MockRuntimeRule::try_from(vec![10, 20]).expect("Should create fee propagation");
				assert_ok!(AffiliatesAlpha::set_rule_for(
					RuntimeOrigin::signed(ALICE),
					0,
					rule.clone()
				));

				System::assert_last_event(RuntimeEvent::AffiliatesAlpha(Event::RuleAdded {
					rule_id: 0,
				}));

				assert_eq!(AffiliateRules::<Test, AffiliatesInstance1>::get(0), Some(rule));
			});
	}

	#[test]
	fn set_rule_not_allowed_for_non_organizer() {
		let initial_balance = 1_000_000;
		ExtBuilder::default()
			.balances(&[(ALICE, initial_balance)])
			.organizer(ALICE)
			.build()
			.execute_with(|| {
				let rule =
					MockRuntimeRule::try_from(vec![10, 20]).expect("Should create fee propagation");
				assert_noop!(
					AffiliatesAlpha::set_rule_for(RuntimeOrigin::signed(BOB), 0, rule.clone()),
					DispatchError::Other(ACCOUNT_IS_NOT_ORGANIZER)
				);
			});
	}

	#[test]
	fn clear_rule_works() {
		let initial_balance = 1_000_000;
		ExtBuilder::default()
			.balances(&[(ALICE, initial_balance)])
			.organizer(ALICE)
			.build()
			.execute_with(|| {
				let rule =
					MockRuntimeRule::try_from(vec![10, 20]).expect("Should create fee propagation");
				assert_ok!(AffiliatesAlpha::set_rule_for(
					RuntimeOrigin::signed(ALICE),
					1,
					rule.clone()
				));
				assert_eq!(AffiliateRules::<Test, AffiliatesInstance1>::get(1), Some(rule));
				assert_ok!(AffiliatesAlpha::clear_rule_for(RuntimeOrigin::signed(ALICE), 1));

				System::assert_last_event(RuntimeEvent::AffiliatesAlpha(Event::RuleCleared {
					rule_id: 1,
				}));
			});
	}

	#[test]
	fn clear_rule_not_allowed_for_non_organizer() {
		let initial_balance = 1_000_000;
		ExtBuilder::default()
			.balances(&[(ALICE, initial_balance)])
			.organizer(ALICE)
			.build()
			.execute_with(|| {
				let rule =
					MockRuntimeRule::try_from(vec![10, 20]).expect("Should create fee propagation");
				assert_ok!(AffiliatesAlpha::set_rule_for(
					RuntimeOrigin::signed(ALICE),
					1,
					rule.clone()
				));
				assert_eq!(AffiliateRules::<Test, AffiliatesInstance1>::get(1), Some(rule));
				assert_noop!(
					AffiliatesAlpha::clear_rule_for(RuntimeOrigin::signed(BOB), 1),
					DispatchError::Other(ACCOUNT_IS_NOT_ORGANIZER)
				);
			});
	}
}

mod add_rule {
	use super::*;

	#[test]
	fn add_rule_should_work() {
		ExtBuilder::default().build().execute_with(|| {
			let rule_id = 0;
			let rule = MockRuntimeRule::default();

			assert_ok!(
				<AffiliatesAlpha as RuleMutator<MockRuleId, MockRuntimeRule>>::try_add_rule_for(
					rule_id,
					rule.clone()
				)
			);

			System::assert_last_event(mock::RuntimeEvent::AffiliatesAlpha(
				crate::Event::RuleAdded { rule_id },
			));

			assert_eq!(AffiliateRules::<Test, Instance1>::get(rule_id), Some(rule));
		})
	}

	#[test]
	fn add_multiple_rules_should_work() {
		ExtBuilder::default().build().execute_with(|| {
			let rule_id_1 = 0;
			let rule_1 = MockRuntimeRule::try_from(vec![1, 1]).expect("Create MockRuntimeRule");

			let rule_id_2 = 1;
			let rule_2 = MockRuntimeRule::try_from(vec![2, 2]).expect("Create MockRuntimeRule");

			assert_ok!(
				<AffiliatesAlpha as RuleMutator<MockRuleId, MockRuntimeRule>>::try_add_rule_for(
					rule_id_1,
					rule_1.clone()
				)
			);
			System::assert_last_event(mock::RuntimeEvent::AffiliatesAlpha(
				crate::Event::RuleAdded { rule_id: rule_id_1 },
			));

			assert_ok!(
				<AffiliatesAlpha as RuleMutator<MockRuleId, MockRuntimeRule>>::try_add_rule_for(
					rule_id_2,
					rule_2.clone()
				)
			);
			System::assert_last_event(mock::RuntimeEvent::AffiliatesAlpha(
				crate::Event::RuleAdded { rule_id: rule_id_2 },
			));

			assert_eq!(AffiliateRules::<Test, Instance1>::get(rule_id_1), Some(rule_1));
			assert_eq!(AffiliateRules::<Test, Instance1>::get(rule_id_2), Some(rule_2));
		})
	}

	#[test]
	fn cannot_add_rule_to_already_marked_extrinsic() {
		ExtBuilder::default().build().execute_with(|| {
			let rule_id = 0;
			let rule = MockRuntimeRule::default();
			assert_ok!(
				<AffiliatesAlpha as RuleMutator<MockRuleId, MockRuntimeRule>>::try_add_rule_for(
					rule_id, rule
				)
			);

			let rule_2 = MockRuntimeRule::default();
			assert_noop!(
				<AffiliatesAlpha as RuleMutator<MockRuleId, MockRuntimeRule>>::try_add_rule_for(
					rule_id, rule_2
				),
				Error::<Test, Instance1>::ExtrinsicAlreadyHasRule
			);
		})
	}
}

mod clear_rule {
	use super::*;

	#[test]
	fn clear_rule_should_work() {
		ExtBuilder::default().build().execute_with(|| {
			let rule_id = 0;
			let rule = MockRuntimeRule::default();

			assert_ok!(
				<AffiliatesAlpha as RuleMutator<MockRuleId, MockRuntimeRule>>::try_add_rule_for(
					rule_id,
					rule.clone()
				)
			);

			System::assert_last_event(mock::RuntimeEvent::AffiliatesAlpha(
				crate::Event::RuleAdded { rule_id },
			));

			assert_eq!(AffiliateRules::<Test, Instance1>::get(rule_id), Some(rule));

			<AffiliatesAlpha as RuleMutator<MockRuleId, MockRuntimeRule>>::clear_rule_for(rule_id);

			System::assert_last_event(mock::RuntimeEvent::AffiliatesAlpha(
				crate::Event::RuleCleared { rule_id },
			));

			assert_eq!(AffiliateRules::<Test, Instance1>::get(rule_id), None);
		})
	}

	#[test]
	fn clear_rule_for_non_existent_rule() {
		ExtBuilder::default().build().execute_with(|| {
			let rule_id = 0;

			<AffiliatesAlpha as RuleMutator<MockRuleId, MockRuntimeRule>>::clear_rule_for(rule_id);

			System::assert_last_event(mock::RuntimeEvent::AffiliatesAlpha(
				crate::Event::RuleCleared { rule_id },
			));

			assert_eq!(AffiliateRules::<Test, Instance1>::get(rule_id), None);
		})
	}

	#[test]
	fn clear_rule_only_affects_selected_rule() {
		ExtBuilder::default().build().execute_with(|| {
			let rule_id_1 = 0;
			let rule_1 = MockRuntimeRule::try_from(vec![1, 1]).expect("Create MockRuntimeRule");

			let rule_id_2 = 1;
			let rule_2 = MockRuntimeRule::try_from(vec![2, 2]).expect("Create MockRuntimeRule");

			assert_ok!(
				<AffiliatesAlpha as RuleMutator<MockRuleId, MockRuntimeRule>>::try_add_rule_for(
					rule_id_1,
					rule_1.clone()
				)
			);
			assert_ok!(
				<AffiliatesAlpha as RuleMutator<MockRuleId, MockRuntimeRule>>::try_add_rule_for(
					rule_id_2,
					rule_2.clone()
				)
			);

			assert_eq!(AffiliateRules::<Test, Instance1>::get(rule_id_1), Some(rule_1.clone()));
			assert_eq!(AffiliateRules::<Test, Instance1>::get(rule_id_2), Some(rule_2));

			<AffiliatesAlpha as RuleMutator<MockRuleId, MockRuntimeRule>>::clear_rule_for(
				rule_id_2,
			);

			System::assert_last_event(mock::RuntimeEvent::AffiliatesAlpha(
				crate::Event::RuleCleared { rule_id: rule_id_2 },
			));

			assert_eq!(AffiliateRules::<Test, Instance1>::get(rule_id_1), Some(rule_1));
			assert_eq!(AffiliateRules::<Test, Instance1>::get(rule_id_2), None);
		})
	}
}

mod affiliate_to {
	use super::*;

	#[test]
	fn affiliate_to_should_work() {
		ExtBuilder::default().build().execute_with(|| {
			let state =
				AffiliatorState { status: AffiliatableStatus::Affiliatable(0), affiliates: 0 };
			Affiliators::<Test, Instance1>::insert(BOB, state);

			assert_ok!(
				<AffiliatesAlpha as AffiliateMutator<AccountIdFor<Test>>>::try_add_affiliate_to(
					&BOB, &ALICE
				)
			);

			System::assert_last_event(mock::RuntimeEvent::AffiliatesAlpha(
				crate::Event::AccountAffiliated { account: ALICE, to: BOB },
			));

			assert_eq!(Affiliators::<Test, Instance1>::get(BOB).affiliates, 1);
			assert_eq!(Affiliatees::<Test, Instance1>::get(ALICE), Some(bounded_vec![BOB]));
		});
	}

	#[test]
	fn affiliate_to_should_work_with_chain() {
		ExtBuilder::default().build().execute_with(|| {
			let state =
				AffiliatorState { status: AffiliatableStatus::Affiliatable(0), affiliates: 0 };
			Affiliators::<Test, Instance1>::insert(BOB, state);
			Affiliators::<Test, Instance1>::insert(ALICE, state);
			Affiliators::<Test, Instance1>::insert(CHARLIE, state);
			Affiliators::<Test, Instance1>::insert(DAVE, state);

			// First step on the chain BOB <- ALICE
			assert_ok!(
				<AffiliatesAlpha as AffiliateMutator<AccountIdFor<Test>>>::try_add_affiliate_to(
					&BOB, &ALICE
				)
			);

			System::assert_last_event(mock::RuntimeEvent::AffiliatesAlpha(
				crate::Event::AccountAffiliated { account: ALICE, to: BOB },
			));

			assert_eq!(Affiliators::<Test, Instance1>::get(BOB).affiliates, 1);
			assert_eq!(Affiliatees::<Test, Instance1>::get(ALICE), Some(bounded_vec![BOB]));

			// Second step on the chain BOB <- ALICE <- CHARLIE
			assert_ok!(
				<AffiliatesAlpha as AffiliateMutator<AccountIdFor<Test>>>::try_add_affiliate_to(
					&ALICE, &CHARLIE
				)
			);

			System::assert_last_event(mock::RuntimeEvent::AffiliatesAlpha(
				crate::Event::AccountAffiliated { account: CHARLIE, to: ALICE },
			));

			assert_eq!(Affiliators::<Test, Instance1>::get(ALICE).affiliates, 1);
			assert_eq!(Affiliatees::<Test, Instance1>::get(ALICE), Some(bounded_vec![BOB]));
			assert_eq!(
				Affiliatees::<Test, Instance1>::get(CHARLIE),
				Some(bounded_vec![ALICE, BOB])
			);

			// Third step on the chain BOB <- ALICE <- CHARLIE <- DAVE
			assert_ok!(
				<AffiliatesAlpha as AffiliateMutator<AccountIdFor<Test>>>::try_add_affiliate_to(
					&CHARLIE, &DAVE
				)
			);

			System::assert_last_event(mock::RuntimeEvent::AffiliatesAlpha(
				crate::Event::AccountAffiliated { account: DAVE, to: CHARLIE },
			));

			assert_eq!(Affiliators::<Test, Instance1>::get(CHARLIE).affiliates, 1);
			assert_eq!(Affiliatees::<Test, Instance1>::get(ALICE), Some(bounded_vec![BOB]));
			assert_eq!(
				Affiliatees::<Test, Instance1>::get(CHARLIE),
				Some(bounded_vec![ALICE, BOB])
			);
			assert_eq!(
				Affiliatees::<Test, Instance1>::get(DAVE),
				Some(bounded_vec![CHARLIE, ALICE])
			);

			// Fourth step on the chain BOB <- ALICE <- CHARLIE <- DAVE <- EDWARD
			assert_ok!(
				<AffiliatesAlpha as AffiliateMutator<AccountIdFor<Test>>>::try_add_affiliate_to(
					&DAVE, &EDWARD
				)
			);

			System::assert_last_event(mock::RuntimeEvent::AffiliatesAlpha(
				crate::Event::AccountAffiliated { account: EDWARD, to: DAVE },
			));

			assert_eq!(Affiliators::<Test, Instance1>::get(DAVE).affiliates, 1);
			assert_eq!(Affiliatees::<Test, Instance1>::get(ALICE), Some(bounded_vec![BOB]));
			assert_eq!(
				Affiliatees::<Test, Instance1>::get(CHARLIE),
				Some(bounded_vec![ALICE, BOB])
			);
			assert_eq!(
				Affiliatees::<Test, Instance1>::get(DAVE),
				Some(bounded_vec![CHARLIE, ALICE])
			);
			assert_eq!(
				Affiliatees::<Test, Instance1>::get(EDWARD),
				Some(bounded_vec![DAVE, CHARLIE])
			);
		});
	}

	#[test]
	fn affiliate_to_should_work_with_complex_chain() {
		ExtBuilder::default().build().execute_with(|| {
			let state =
				AffiliatorState { status: AffiliatableStatus::Affiliatable(0), affiliates: 0 };
			Affiliators::<Test, Instance1>::insert(BOB, state);
			Affiliators::<Test, Instance1>::insert(ALICE, state);
			Affiliators::<Test, Instance1>::insert(CHARLIE, state);
			Affiliators::<Test, Instance1>::insert(DAVE, state);

			// First step on first chain BOB <- ALICE
			assert_ok!(
				<AffiliatesAlpha as AffiliateMutator<AccountIdFor<Test>>>::try_add_affiliate_to(
					&BOB, &ALICE
				)
			);

			System::assert_last_event(mock::RuntimeEvent::AffiliatesAlpha(
				crate::Event::AccountAffiliated { account: ALICE, to: BOB },
			));

			assert_eq!(Affiliators::<Test, Instance1>::get(BOB).affiliates, 1);
			assert_eq!(Affiliatees::<Test, Instance1>::get(ALICE), Some(bounded_vec![BOB]));

			// First step on second chain DAVE <- CHARLIE
			assert_ok!(
				<AffiliatesAlpha as AffiliateMutator<AccountIdFor<Test>>>::try_add_affiliate_to(
					&DAVE, &CHARLIE
				)
			);

			System::assert_last_event(mock::RuntimeEvent::AffiliatesAlpha(
				crate::Event::AccountAffiliated { account: CHARLIE, to: DAVE },
			));

			assert_eq!(Affiliators::<Test, Instance1>::get(DAVE).affiliates, 1);
			assert_eq!(Affiliatees::<Test, Instance1>::get(CHARLIE), Some(bounded_vec![DAVE]));

			// Second step on second chain: DAVE <- [CHARLIE, EDWARD]
			assert_ok!(
				<AffiliatesAlpha as AffiliateMutator<AccountIdFor<Test>>>::try_add_affiliate_to(
					&DAVE, &EDWARD
				)
			);

			System::assert_last_event(mock::RuntimeEvent::AffiliatesAlpha(
				crate::Event::AccountAffiliated { account: EDWARD, to: DAVE },
			));

			assert_eq!(Affiliators::<Test, Instance1>::get(DAVE).affiliates, 2);
			assert_eq!(Affiliatees::<Test, Instance1>::get(CHARLIE), Some(bounded_vec![DAVE]));
			assert_eq!(Affiliatees::<Test, Instance1>::get(EDWARD), Some(bounded_vec![DAVE]));

			// Second step, linking both chains
			// Current chain state: BOB <- ALICE | DAVE <- [CHARLIE, EDWARD]
			assert_eq!(Affiliators::<Test, Instance1>::get(ALICE).affiliates, 0);
			assert_eq!(Affiliators::<Test, Instance1>::get(BOB).affiliates, 1);
			assert_eq!(Affiliators::<Test, Instance1>::get(CHARLIE).affiliates, 0);
			assert_eq!(Affiliators::<Test, Instance1>::get(DAVE).affiliates, 2);
			assert_eq!(Affiliators::<Test, Instance1>::get(EDWARD).affiliates, 0);

			assert_eq!(Affiliatees::<Test, Instance1>::get(ALICE), Some(bounded_vec![BOB]));
			assert_eq!(Affiliatees::<Test, Instance1>::get(BOB), None);
			assert_eq!(Affiliatees::<Test, Instance1>::get(CHARLIE), Some(bounded_vec![DAVE]));
			assert_eq!(Affiliatees::<Test, Instance1>::get(DAVE), None);
			assert_eq!(Affiliatees::<Test, Instance1>::get(EDWARD), Some(bounded_vec![DAVE]));

			assert_ok!(
				<AffiliatesAlpha as AffiliateMutator<AccountIdFor<Test>>>::try_clear_affiliation_for(
					&EDWARD
				)
			);

			assert_eq!(Affiliators::<Test, Instance1>::get(DAVE).affiliates, 1);
			assert_eq!(Affiliatees::<Test, Instance1>::get(EDWARD), None);

			assert_ok!(
				<AffiliatesAlpha as AffiliateMutator<AccountIdFor<Test>>>::try_add_affiliate_to(
					&ALICE, &EDWARD
				)
			);

			System::assert_last_event(mock::RuntimeEvent::AffiliatesAlpha(
				crate::Event::AccountAffiliated { account: EDWARD, to: ALICE },
			));

			assert_eq!(Affiliators::<Test, Instance1>::get(ALICE).affiliates, 1);
			assert_eq!(Affiliatees::<Test, Instance1>::get(EDWARD), Some(bounded_vec![ALICE, BOB]));

			assert_ok!(
				<AffiliatesAlpha as AffiliateMutator<AccountIdFor<Test>>>::try_clear_affiliation_for(
					&CHARLIE
				)
			);

			assert_eq!(Affiliators::<Test, Instance1>::get(DAVE).affiliates, 0);
			assert_eq!(Affiliatees::<Test, Instance1>::get(CHARLIE), None);

			assert_ok!(
				<AffiliatesAlpha as AffiliateMutator<AccountIdFor<Test>>>::try_add_affiliate_to(
					&ALICE, &CHARLIE
				)
			);

			assert_eq!(Affiliators::<Test, Instance1>::get(ALICE).affiliates, 2);
			assert_eq!(
				Affiliatees::<Test, Instance1>::get(CHARLIE),
				Some(bounded_vec![ALICE, BOB])
			);

			assert_ok!(
				<AffiliatesAlpha as AffiliateMutator<AccountIdFor<Test>>>::try_add_affiliate_to(
					&CHARLIE, &DAVE
				)
			);

			// Final chain state: BOB <- ALICE <- [CHARLIE <- DAVE, EDWARD]
			assert_eq!(Affiliators::<Test, Instance1>::get(ALICE).affiliates, 2);
			assert_eq!(Affiliators::<Test, Instance1>::get(BOB).affiliates, 1);
			assert_eq!(Affiliators::<Test, Instance1>::get(CHARLIE).affiliates, 1);
			assert_eq!(Affiliators::<Test, Instance1>::get(DAVE).affiliates, 0);
			assert_eq!(Affiliators::<Test, Instance1>::get(EDWARD).affiliates, 0);

			assert_eq!(Affiliatees::<Test, Instance1>::get(ALICE), Some(bounded_vec![BOB]));
			assert_eq!(Affiliatees::<Test, Instance1>::get(BOB), None);
			assert_eq!(
				Affiliatees::<Test, Instance1>::get(CHARLIE),
				Some(bounded_vec![ALICE, BOB])
			);
			assert_eq!(
				Affiliatees::<Test, Instance1>::get(DAVE),
				Some(bounded_vec![CHARLIE, ALICE])
			);
			assert_eq!(Affiliatees::<Test, Instance1>::get(EDWARD), Some(bounded_vec![ALICE, BOB]));
		});
	}

	#[test]
	fn affiliate_to_rejects_with_self_account() {
		ExtBuilder::default().build().execute_with(|| {
			assert_noop!(
				<AffiliatesAlpha as AffiliateMutator<AccountIdFor<Test>>>::try_add_affiliate_to(
					&ALICE, &ALICE
				),
				Error::<Test, Instance1>::CannotAffiliateSelf
			);
		});
	}

	#[test]
	fn affiliate_to_rejects_if_account_is_affiliator() {
		ExtBuilder::default().build().execute_with(|| {
			let state =
				AffiliatorState { status: AffiliatableStatus::Affiliatable(0), affiliates: 0 };
			Affiliators::<Test, Instance1>::insert(BOB, state);
			Affiliators::<Test, Instance1>::insert(ALICE, state);

			assert_ok!(
				<AffiliatesAlpha as AffiliateMutator<AccountIdFor<Test>>>::try_add_affiliate_to(
					&BOB, &ALICE
				)
			);

			assert_noop!(
				<AffiliatesAlpha as AffiliateMutator<AccountIdFor<Test>>>::try_add_affiliate_to(
					&ALICE, &BOB
				),
				Error::<Test, Instance1>::CannotAffiliateToExistingAffiliator
			);
		});
	}

	#[test]
	fn affiliate_to_rejects_affiliating_to_more_than_one_account() {
		ExtBuilder::default().build().execute_with(|| {
			let state =
				AffiliatorState { status: AffiliatableStatus::Affiliatable(0), affiliates: 0 };
			Affiliators::<Test, Instance1>::insert(BOB, state);
			Affiliators::<Test, Instance1>::insert(CHARLIE, state);

			assert_ok!(
				<AffiliatesAlpha as AffiliateMutator<AccountIdFor<Test>>>::try_add_affiliate_to(
					&BOB, &ALICE
				)
			);

			assert_noop!(
				<AffiliatesAlpha as AffiliateMutator<AccountIdFor<Test>>>::try_add_affiliate_to(
					&CHARLIE, &ALICE
				),
				Error::<Test, Instance1>::CannotAffiliateAlreadyAffiliatedAccount
			);
		});
	}

	#[test]
	fn affiliate_to_rejects_with_unaffiliable_account() {
		ExtBuilder::default().build().execute_with(|| {
			Affiliators::<Test, Instance1>::mutate(BOB, |state| {
				state.status = AffiliatableStatus::NonAffiliatable;
			});

			assert_noop!(
				<AffiliatesAlpha as AffiliateMutator<AccountIdFor<Test>>>::try_add_affiliate_to(
					&BOB, &ALICE
				),
				Error::<Test, Instance1>::TargetAccountIsNotAffiliatable
			);
		});
	}

	#[test]
	fn affiliate_to_rejects_with_blocked_account() {
		ExtBuilder::default().build().execute_with(|| {
			Affiliators::<Test, Instance1>::mutate(BOB, |state| {
				state.status = AffiliatableStatus::Blocked;
			});

			assert_noop!(
				<AffiliatesAlpha as AffiliateMutator<AccountIdFor<Test>>>::try_add_affiliate_to(
					&BOB, &ALICE
				),
				Error::<Test, Instance1>::TargetAccountIsNotAffiliatable
			);
		});
	}
}

mod clear_affiliation {
	use super::*;

	#[test]
	fn clear_affiliation_should_work() {
		ExtBuilder::default().balances(&[(ALICE, 1_000_000)]).build().execute_with(|| {
			let state =
				AffiliatorState { status: AffiliatableStatus::Affiliatable(0), affiliates: 0 };
			Affiliators::<Test, Instance1>::insert(BOB, state);

			assert_ok!(
				<AffiliatesAlpha as AffiliateMutator<AccountIdFor<Test>>>::try_add_affiliate_to(
					&BOB, &ALICE
				)
			);

			System::assert_last_event(mock::RuntimeEvent::AffiliatesAlpha(
				crate::Event::AccountAffiliated { account: ALICE, to: BOB },
			));

			assert_eq!(Affiliators::<Test, Instance1>::get(BOB).affiliates, 1);
			assert_eq!(Affiliatees::<Test, Instance1>::get(ALICE), Some(bounded_vec![BOB]));

			assert_ok!(
				<AffiliatesAlpha as AffiliateMutator<AccountIdFor<Test>>>::try_clear_affiliation_for(
					&ALICE
				)
			);

			assert_eq!(Affiliators::<Test, Instance1>::get(BOB).affiliates, 0);
			assert_eq!(Affiliatees::<Test, Instance1>::get(ALICE), None);
		});
	}

	#[test]
	fn clear_affiliation_should_work_with_long_chains() {
		ExtBuilder::default().balances(&[(ALICE, 1_000_000)]).build().execute_with(|| {
			let state =
				AffiliatorState { status: AffiliatableStatus::Affiliatable(0), affiliates: 0 };
			Affiliators::<Test, Instance1>::insert(BOB, state);
			Affiliators::<Test, Instance1>::insert(DAVE, state);

			assert_ok!(
				<AffiliatesAlpha as AffiliateMutator<AccountIdFor<Test>>>::try_add_affiliate_to(
					&BOB, &ALICE
				)
			);
			System::assert_last_event(mock::RuntimeEvent::AffiliatesAlpha(
				crate::Event::AccountAffiliated { account: ALICE, to: BOB },
			));
			assert_eq!(Affiliators::<Test, Instance1>::get(BOB).affiliates, 1);
			assert_eq!(Affiliatees::<Test, Instance1>::get(ALICE), Some(bounded_vec![BOB]));

			assert_ok!(
				<AffiliatesAlpha as AffiliateMutator<AccountIdFor<Test>>>::try_add_affiliate_to(
					&BOB, &CHARLIE
				)
			);
			System::assert_last_event(mock::RuntimeEvent::AffiliatesAlpha(
				crate::Event::AccountAffiliated { account: CHARLIE, to: BOB },
			));
			assert_eq!(Affiliators::<Test, Instance1>::get(BOB).affiliates, 2);
			assert_eq!(Affiliatees::<Test, Instance1>::get(CHARLIE), Some(bounded_vec![BOB]));

			assert_ok!(
				<AffiliatesAlpha as AffiliateMutator<AccountIdFor<Test>>>::try_add_affiliate_to(
					&BOB, &DAVE
				)
			);
			System::assert_last_event(mock::RuntimeEvent::AffiliatesAlpha(
				crate::Event::AccountAffiliated { account: DAVE, to: BOB },
			));
			assert_eq!(Affiliators::<Test, Instance1>::get(BOB).affiliates, 3);
			assert_eq!(Affiliatees::<Test, Instance1>::get(DAVE), Some(bounded_vec![BOB]));

			assert_ok!(
				<AffiliatesAlpha as AffiliateMutator<AccountIdFor<Test>>>::try_add_affiliate_to(
					&DAVE, &EDWARD
				)
			);
			System::assert_last_event(mock::RuntimeEvent::AffiliatesAlpha(
				crate::Event::AccountAffiliated { account: EDWARD, to: DAVE },
			));
			assert_eq!(Affiliators::<Test, Instance1>::get(DAVE).affiliates, 1);
			assert_eq!(Affiliatees::<Test, Instance1>::get(EDWARD), Some(bounded_vec![DAVE, BOB]));

			assert_ok!(
				<AffiliatesAlpha as AffiliateMutator<AccountIdFor<Test>>>::try_clear_affiliation_for(
					&ALICE
				)
			);

			assert_eq!(Affiliators::<Test, Instance1>::get(BOB).affiliates, 2);
			assert_eq!(Affiliatees::<Test, Instance1>::get(ALICE), None);
			assert_eq!(Affiliators::<Test, Instance1>::get(DAVE).affiliates, 1);
			assert_eq!(Affiliatees::<Test, Instance1>::get(EDWARD), Some(bounded_vec![DAVE, BOB]));

			assert_ok!(
				<AffiliatesAlpha as AffiliateMutator<AccountIdFor<Test>>>::try_clear_affiliation_for(
					&EDWARD
				)
			);

			assert_eq!(Affiliators::<Test, Instance1>::get(BOB).affiliates, 2);
			assert_eq!(Affiliatees::<Test, Instance1>::get(ALICE), None);
			assert_eq!(Affiliators::<Test, Instance1>::get(DAVE).affiliates, 0);
			assert_eq!(Affiliatees::<Test, Instance1>::get(EDWARD), None);
		});
	}

	#[test]
	fn clear_affiliation_returns_ok_if_no_affiliation_exists() {
		ExtBuilder::default().balances(&[(ALICE, 1_000_000)]).build().execute_with(|| {
			assert_eq!(Affiliatees::<Test, Instance1>::get(ALICE), None);

			assert_ok!(
				<AffiliatesAlpha as AffiliateMutator<AccountIdFor<Test>>>::try_clear_affiliation_for(
					&ALICE
				)
			);

			assert_eq!(Affiliatees::<Test, Instance1>::get(ALICE), None);
		});
	}
}

mod multi_instance_tests {
	use super::*;

	#[test]
	fn affiliates_in_one_instance_dont_affect_other_instance() {
		ExtBuilder::default().balances(&[(ALICE, 1_000_000)]).build().execute_with(|| {
			let state_alpha =
				AffiliatorState { status: AffiliatableStatus::Affiliatable(0), affiliates: 0 };
			Affiliators::<Test, Instance1>::insert(BOB, state_alpha);

			assert_ok!(
				<AffiliatesAlpha as AffiliateMutator<AccountIdFor<Test>>>::try_add_affiliate_to(
					&BOB, &ALICE
				)
			);

			System::assert_last_event(mock::RuntimeEvent::AffiliatesAlpha(
				crate::Event::AccountAffiliated { account: ALICE, to: BOB },
			));

			// Instance1 state contains the affiliated state
			assert_eq!(Affiliators::<Test, Instance1>::get(BOB).affiliates, 1);
			assert_eq!(Affiliatees::<Test, Instance1>::get(ALICE), Some(bounded_vec![BOB]));
			// Instance2 state contains no information as expected
			assert_eq!(Affiliators::<Test, Instance2>::get(BOB).affiliates, 0);
			assert_eq!(Affiliatees::<Test, Instance2>::get(ALICE), None);

			let state_beta =
				AffiliatorState { status: AffiliatableStatus::Affiliatable(0), affiliates: 0 };
			Affiliators::<Test, Instance2>::insert(ALICE, state_beta);

			// Trying to affiliate to ALICE on AffiliatesAlpha will fail since
			// she is not Affiliatable there
			assert_noop!(
				<AffiliatesAlpha as AffiliateMutator<AccountIdFor<Test>>>::try_add_affiliate_to(
					&ALICE, &CHARLIE
				),
				Error::<Test, Instance1>::TargetAccountIsNotAffiliatable
			);

			// In AffiliatesBeta it works as expected
			assert_ok!(
				<AffiliatesBeta as AffiliateMutator<AccountIdFor<Test>>>::try_add_affiliate_to(
					&ALICE, &CHARLIE
				)
			);

			// Instance1 state doesn't contain the affiliation changes
			assert_eq!(Affiliators::<Test, Instance1>::get(ALICE).affiliates, 0);
			assert_eq!(Affiliatees::<Test, Instance1>::get(CHARLIE), None);
			// While Instance2 does
			assert_eq!(Affiliators::<Test, Instance2>::get(ALICE).affiliates, 1);
			assert_eq!(Affiliatees::<Test, Instance2>::get(CHARLIE), Some(bounded_vec![ALICE]));
		});
	}

	#[test]
	fn rule_in_one_instance_doesnt_affect_other_instance() {
		ExtBuilder::default().balances(&[(ALICE, 1_000_000)]).build().execute_with(|| {
			let rule_id = 0;
			let rule = MockRuntimeRule::default();

			assert_ok!(
				<AffiliatesAlpha as RuleMutator<MockRuleId, MockRuntimeRule>>::try_add_rule_for(
					rule_id,
					rule.clone()
				)
			);

			System::assert_last_event(mock::RuntimeEvent::AffiliatesAlpha(
				crate::Event::RuleAdded { rule_id },
			));

			// The rule is added in Instance1
			assert_eq!(AffiliateRules::<Test, Instance1>::get(rule_id), Some(rule.clone()));
			// But not on Instance2
			assert_eq!(AffiliateRules::<Test, Instance2>::get(rule_id), None);

			<AffiliatesBeta as RuleMutator<MockRuleId, MockRuntimeRule>>::clear_rule_for(rule_id);

			// The rule remains in Instance1
			assert_eq!(AffiliateRules::<Test, Instance1>::get(rule_id), Some(rule));
			// No changes also in Instance2
			assert_eq!(AffiliateRules::<Test, Instance2>::get(rule_id), None);
		});
	}
}

mod force_affiliatees {
	use super::*;

	#[test]
	fn force_set_affiliation_state_works() {
		ExtBuilder::default().build().execute_with(|| {
			let account = ALICE;
			let chain = vec![BOB, CHARLIE];

			assert_eq!(Affiliatees::<Test, Instance1>::get(ALICE), None);

			assert_ok!(
				<AffiliatesAlpha as AffiliateMutator<AccountIdFor<Test>>>::force_set_affiliatee_chain_for(
					&account, chain.clone()
				)
			);

			assert_eq!(
				Affiliatees::<Test, Instance1>::get(account).map(|acc| acc.to_vec()),
				Some(chain)
			);
		});
	}
}
