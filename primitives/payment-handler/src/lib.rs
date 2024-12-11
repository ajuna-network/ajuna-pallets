#![cfg_attr(not(feature = "std"), no_std)]

mod fee_handler;
mod withdraw_credit;

pub use fee_handler::*;
use frame_support::traits::{fungible, fungibles, Imbalance};
pub use withdraw_credit::*;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;
mod voucher_handler;

pub trait IntoFungiblesCredit<AccountId, B>
where
	B: fungibles::Inspect<AccountId> + fungibles::Balanced<AccountId>,
{
	fn into_credit(self) -> fungibles::Credit<AccountId, B>;
}

impl<AccountId, B> IntoFungiblesCredit<AccountId, B> for fungibles::Credit<AccountId, B>
where
	B: fungibles::Inspect<AccountId> + fungibles::Balanced<AccountId>,
{
	fn into_credit(self) -> fungibles::Credit<AccountId, B> {
		self
	}
}

impl<AccountId, B> IntoFungiblesCredit<AccountId, B> for Option<fungibles::Credit<AccountId, B>>
where
	B: fungibles::Inspect<AccountId> + fungibles::Balanced<AccountId>,
	B::AssetId: Default,
{
	fn into_credit(self) -> fungibles::Credit<AccountId, B> {
		match self {
			Some(credit) => credit,
			None => fungibles::Credit::<AccountId, B>::zero(B::AssetId::default()),
		}
	}
}

pub trait IntoFungibleCredit<AccountId, B>
where
	B: fungible::Inspect<AccountId> + fungible::Balanced<AccountId>,
{
	fn into_credit(self) -> fungible::Credit<AccountId, B>;
}

impl<AccountId, B> IntoFungibleCredit<AccountId, B> for fungible::Credit<AccountId, B>
where
	B: fungible::Inspect<AccountId> + fungible::Balanced<AccountId>,
{
	fn into_credit(self) -> fungible::Credit<AccountId, B> {
		self
	}
}

impl<AccountId, B> IntoFungibleCredit<AccountId, B> for Option<fungible::Credit<AccountId, B>>
where
	B: fungibles::Inspect<AccountId> + fungible::Balanced<AccountId>,
{
	fn into_credit(self) -> fungible::Credit<AccountId, B> {
		match self {
			Some(credit) => credit,
			None => fungible::Credit::<AccountId, B>::zero(),
		}
	}
}
