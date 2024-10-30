//! This module defines some types we use in the ajuna runtime.
//!
//! We have this here, because we want to make the live easier for our downstream
//! developers, so that they don't have the hassle with generics.

use frame_support::sp_runtime::AccountId32;

pub type AccountId = AccountId32;
pub type Balance = u128;
