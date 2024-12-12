use crate::withdraw_credit::WithdrawCredit;

use core::{fmt::Debug, marker::PhantomData};
use frame_support::{
	pallet_prelude::DispatchError,
	traits::{fungible, fungibles, Defensive, Imbalance},
	BoundedVec,
};
use parity_scale_codec::{Decode, Encode, EncodeLike, MaxEncodedLen};
use scale_info::TypeInfo;

/// Distributes shares of a base fee to some beneficiaries.
pub trait DistributeFee {
	/// AccountId type used.
	type AccountId;

	/// Scalar balance type.
	type Balance;

	/// Fee identifier used to derive the fee distribution.
	type FeeIdentifier;

	/// Type of the fee distribution
	type FeeDistribution;

	fn distribute_fee(
		base_fee: Self::Balance,
		account: &Self::AccountId,
		identifier: &Self::FeeIdentifier,
	) -> Option<Self::FeeDistribution>;
}

/// Payment to be executed.
#[derive(Debug, Encode, Decode, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct PaymentFee<AccountId, Balance> {
	beneficiary: AccountId,
	amount: Balance,
}

impl<AccountId, Balance> PaymentFee<AccountId, Balance> {
	pub fn new(beneficiary: AccountId, amount: Balance) -> Self {
		Self { beneficiary, amount }
	}
}

/// Abstraction of withdrawing fees in one asset and return allocating the fees in the same or
/// another asset.
pub trait FeeHandler {
	type AccountId;

	type PaymentKind: Clone + Eq + Debug + TypeInfo + MaxEncodedLen + EncodeLike + Decode;

	/// Scalar type of the fee balance.
	type Balance;

	type AffiliateFeeIdentifier;
	type TournamentFeeIdentifier;

	/// Withdraws the `base_fee` denominated in `payment` allocate shares of the base fee to
	/// the affiliate, tournament, and treasury if implemented.
	fn withdraw_and_pay_fees(
		payer: &Self::AccountId,
		payment: Self::PaymentKind,
		base_fee: Self::Balance,
		tournament_id: &Self::TournamentFeeIdentifier,
		affiliate_id: &Self::AffiliateFeeIdentifier,
		treasury_pot: &Self::AccountId,
	) -> Result<(), DispatchError>;

	/// Withdraws the `amount` denominated in `payment` and allocates it fully to the
	/// `treasury_pot`.
	fn withdraw_and_deposit_into_treasury(
		who: &Self::AccountId,
		payment: Self::PaymentKind,
		treasury_pot: &Self::AccountId,
		amount: Self::Balance,
	) -> Result<(), DispatchError>;
}

pub type AffiliateFeeDistribution<AccountId, Balance, MaxDistribution> =
	BoundedVec<PaymentFee<AccountId, Balance>, MaxDistribution>;
pub type TournamentFeeDistribution<AccountId, Balance> = PaymentFee<AccountId, Balance>;

pub struct AssetGameFeeHandler<AccountId, Assets, Withdraw, Affiliate, MaxAffiliates, Tournament> {
	_phantom: PhantomData<(AccountId, Assets, Withdraw, Affiliate, MaxAffiliates, Tournament)>,
}

impl<AccountId, Assets, W, Affiliate, MaxAffiliates, Tournament> FeeHandler
	for AssetGameFeeHandler<AccountId, Assets, W, Affiliate, MaxAffiliates, Tournament>
where
	// This is satisfied by the `pallet-assets`, `pallet-asset-conversion` and the
	// `NativeAndAssets` struct.
	Assets: fungibles::Balanced<AccountId, Balance = W::Balance>,
	W: WithdrawCredit<
		AccountId = AccountId,
		Assets = Assets,
		Credit = fungibles::Credit<AccountId, Assets>,
	>,

	Affiliate: DistributeFee<
		AccountId = AccountId,
		Balance = W::Balance,
		FeeDistribution = AffiliateFeeDistribution<AccountId, W::Balance, MaxAffiliates>,
	>,
	Tournament: DistributeFee<
		AccountId = AccountId,
		Balance = W::Balance,
		FeeDistribution = TournamentFeeDistribution<AccountId, W::Balance>,
	>,
{
	type AccountId = AccountId;
	type PaymentKind = W::AssetId;
	type Balance = W::Balance;
	type AffiliateFeeIdentifier = Affiliate::FeeIdentifier;
	type TournamentFeeIdentifier = Tournament::FeeIdentifier;

	fn withdraw_and_pay_fees(
		payer: &Self::AccountId,
		payment: Self::PaymentKind,
		base_fee: Self::Balance,
		tournament_id: &Self::TournamentFeeIdentifier,
		affiliate_id: &Self::AffiliateFeeIdentifier,
		treasury_pot: &Self::AccountId,
	) -> Result<(), DispatchError> {
		// The credit may be in any asset as implemented by `WithdrawAsset`.
		if let Some(fee_credit) = W::withdraw_credit(payer, payment.clone(), base_fee)? {
			let remaining_credit =
				Self::try_propagate_tournament_fee(fee_credit, payer, tournament_id)?;

			let remaining_credit2 =
				Self::try_propagate_chain_fee(remaining_credit, payer, affiliate_id)?;

			Self::deposit_into_treasury(treasury_pot, remaining_credit2)
		} else {
			Ok(())
		}
	}

	fn withdraw_and_deposit_into_treasury(
		who: &Self::AccountId,
		payment: Self::PaymentKind,
		treasury_pot: &Self::AccountId,
		amount: Self::Balance,
	) -> Result<(), DispatchError> {
		if let Some(credit) = W::withdraw_credit(who, payment, amount)? {
			Self::deposit_into_treasury(treasury_pot, credit)
		} else {
			Ok(())
		}
	}
}

impl<AccountId, Assets, W, Affiliate, MaxAffiliates, Tournament>
	AssetGameFeeHandler<AccountId, Assets, W, Affiliate, MaxAffiliates, Tournament>
where
	Assets: fungibles::Balanced<AccountId, Balance = W::Balance>,
	W: WithdrawCredit<
		AccountId = AccountId,
		Assets = Assets,
		Credit = fungibles::Credit<AccountId, Assets>,
	>,

	Affiliate: DistributeFee<
		AccountId = AccountId,
		Balance = W::Balance,
		FeeDistribution = AffiliateFeeDistribution<AccountId, W::Balance, MaxAffiliates>,
	>,
	Tournament: DistributeFee<
		AccountId = AccountId,
		Balance = W::Balance,
		FeeDistribution = TournamentFeeDistribution<AccountId, W::Balance>,
	>,
{
	/// Distributes an already withdrawn `fee_credit` to the affiliates of `account`.
	///
	/// Returns the remaining `fee_credit` after this operation.
	fn try_propagate_chain_fee(
		fee_credit: fungibles::Credit<AccountId, Assets>,
		account: &AccountId,
		identifier: &Affiliate::FeeIdentifier,
	) -> Result<fungibles::Credit<AccountId, Assets>, DispatchError> {
		let mut final_fee = fee_credit;

		if let Some(a) = Affiliate::distribute_fee(final_fee.peek(), account, identifier) {
			for allocation in a {
				if allocation.amount > 0_u32.into() {
					let affiliate_fee = final_fee.extract(allocation.amount);

					if let Err(credit) = W::Assets::resolve(&allocation.beneficiary, affiliate_fee)
					{
						// We decide to continue here, because the error has nothing to do with the
						// account sending the transaction. It would be a bad user experience if
						// the transaction fails because we can't allocate the fees to the
						// recipient.
						log::error!(
							"Could not deposit to affiliate account, it probably doesn't exist."
						);

						// Reabsorb the credit; it can still be used.
						let _ = final_fee.subsume(credit).defensive();
					}
				}
			}
		}

		Ok(final_fee)
	}

	/// Deposits the tournament fee into the tournament pot. The fee is taken from a previously
	/// withdrawn `fee_credit`.
	///
	/// Returns the remaining credit after taking the fee.
	fn try_propagate_tournament_fee(
		fee_credit: fungibles::Credit<AccountId, Assets>,
		account: &AccountId,
		identifier: &Tournament::FeeIdentifier,
	) -> Result<fungibles::Credit<AccountId, Assets>, DispatchError> {
		let mut final_fee = fee_credit;

		if let Some(allocation) = Tournament::distribute_fee(final_fee.peek(), account, identifier)
		{
			if allocation.amount > 0_u32.into() {
				let tournament_credit = final_fee.extract(allocation.amount);

				if let Err(credit) = W::Assets::resolve(&allocation.beneficiary, tournament_credit)
				{
					// We decide to continue here, because the error has nothing to do with the
					// account sending the transaction. It would be a bad user experience if
					// the transaction fails because we can't allocate the fees to the recipient.
					log::error!(
						"Could not deposit to tournament account, it probably doesn't exist."
					);
					// Reabsorb the credit; it can still be used.
					let _ = final_fee.subsume(credit).defensive();
				}
			}
		}

		Ok(final_fee)
	}

	fn deposit_into_treasury(
		key: &W::AccountId,
		credit: fungibles::Credit<AccountId, Assets>,
	) -> Result<(), DispatchError> {
		if let Err(_credit) = W::Assets::resolve(key, credit) {
			// We decide to continue here, because the error has nothing to do with the
			// account sending the transaction. It would be a bad user experience if
			// the transaction fails because we can't allocate the fees to the recipient.
			log::error!(
				"Could not deposit to treasury, it probably doesn't exist, burning the credit..."
			);
		}
		Ok(())
	}
}

pub struct NativeGameFeeHandler<AccountId, Balances, Withdraw, Affiliate, MaxAffiliates, Tournament>
{
	_phantom: PhantomData<(AccountId, Balances, Withdraw, Affiliate, MaxAffiliates, Tournament)>,
}

impl<AccountId, Balances, W, Affiliate, MaxAffiliates, Tournament> FeeHandler
	for NativeGameFeeHandler<AccountId, Balances, W, Affiliate, MaxAffiliates, Tournament>
where
	// This is satisfied by the `pallet-balances`.
	Balances: fungible::Balanced<AccountId, Balance = W::Balance>,
	W: WithdrawCredit<
		AccountId = AccountId,
		Assets = Balances,
		Credit = fungible::Credit<AccountId, Balances>,
	>,

	Affiliate: DistributeFee<
		AccountId = AccountId,
		Balance = W::Balance,
		FeeDistribution = AffiliateFeeDistribution<AccountId, W::Balance, MaxAffiliates>,
	>,
	Tournament: DistributeFee<
		AccountId = AccountId,
		Balance = W::Balance,
		FeeDistribution = TournamentFeeDistribution<AccountId, W::Balance>,
	>,
{
	type AccountId = AccountId;
	// If not vouchers are to be used this can be ()
	type PaymentKind = W::AssetId;
	type Balance = W::Balance;
	type AffiliateFeeIdentifier = Affiliate::FeeIdentifier;
	type TournamentFeeIdentifier = Tournament::FeeIdentifier;

	fn withdraw_and_pay_fees(
		payer: &Self::AccountId,
		payment: Self::PaymentKind,
		base_fee: Self::Balance,
		tournament_id: &Self::TournamentFeeIdentifier,
		affiliate_id: &Self::AffiliateFeeIdentifier,
		treasury_pot: &Self::AccountId,
	) -> Result<(), DispatchError> {
		// The credit may be in any asset as implemented by `WithdrawAsset`.
		if let Some(fee_credit) = W::withdraw_credit(payer, payment, base_fee)? {
			let remaining_credit =
				Self::try_propagate_tournament_fee(fee_credit, payer, tournament_id)?;

			let remaining_credit2 =
				Self::try_propagate_chain_fee(remaining_credit, payer, affiliate_id)?;

			Self::deposit_into_treasury(treasury_pot, remaining_credit2)
		} else {
			Ok(())
		}
	}

	fn withdraw_and_deposit_into_treasury(
		who: &Self::AccountId,
		payment: Self::PaymentKind,
		treasury_pot: &Self::AccountId,
		amount: Self::Balance,
	) -> Result<(), DispatchError> {
		if let Some(credit) = W::withdraw_credit(who, payment, amount)? {
			Self::deposit_into_treasury(treasury_pot, credit)
		} else {
			Ok(())
		}
	}
}

impl<AccountId, Balances, W, Affiliate, MaxAffiliates, Tournament>
	NativeGameFeeHandler<AccountId, Balances, W, Affiliate, MaxAffiliates, Tournament>
where
	Balances: fungible::Balanced<AccountId, Balance = W::Balance>,
	W: WithdrawCredit<
		AccountId = AccountId,
		Assets = Balances,
		Credit = fungible::Credit<AccountId, Balances>,
	>,

	Affiliate: DistributeFee<
		AccountId = AccountId,
		Balance = W::Balance,
		FeeDistribution = AffiliateFeeDistribution<AccountId, W::Balance, MaxAffiliates>,
	>,
	Tournament: DistributeFee<
		AccountId = AccountId,
		Balance = W::Balance,
		FeeDistribution = TournamentFeeDistribution<AccountId, W::Balance>,
	>,
{
	/// Distributes an already withdrawn `fee_credit` to the affiliates of `account`.
	///
	/// Returns the remaining `fee_credit` after this operation.
	fn try_propagate_chain_fee(
		fee_credit: fungible::Credit<AccountId, Balances>,
		account: &AccountId,
		identifier: &Affiliate::FeeIdentifier,
	) -> Result<fungible::Credit<AccountId, Balances>, DispatchError> {
		let mut final_fee = fee_credit;

		if let Some(a) = Affiliate::distribute_fee(final_fee.peek(), account, identifier) {
			for allocation in a {
				if allocation.amount > 0_u32.into() {
					let affiliate_fee = final_fee.extract(allocation.amount);
					if let Err(credit) = W::Assets::resolve(&allocation.beneficiary, affiliate_fee)
					{
						// We decide to continue here, because the error has nothing to do with the
						// account sending the transaction. It would be a bad user experience if
						// the transaction fails because we can't allocate the fees to the
						// recipient.
						log::error!(
							"Could not deposit to affiliate account, it probably doesn't exist."
						);
						// Reabsorb the credit; it can still be used.
						final_fee.subsume(credit);
					}
				}
			}
		}

		Ok(final_fee)
	}

	/// Deposits the tournament fee into the tournament pot. The fee is taken from a previously
	/// withdrawn `fee_credit`.
	///
	/// Returns the remaining credit after taking the fee.
	fn try_propagate_tournament_fee(
		fee_credit: fungible::Credit<AccountId, Balances>,
		account: &AccountId,
		identifier: &Tournament::FeeIdentifier,
	) -> Result<fungible::Credit<AccountId, Balances>, DispatchError> {
		let mut final_fee = fee_credit;

		if let Some(allocation) = Tournament::distribute_fee(final_fee.peek(), account, identifier)
		{
			if allocation.amount > 0_u32.into() {
				let tournament_credit = final_fee.extract(allocation.amount);

				if let Err(credit) = W::Assets::resolve(&allocation.beneficiary, tournament_credit)
				{
					// We decide to continue here, because the error has nothing to do with the
					// account sending the transaction. It would be a bad user experience if
					// the transaction fails because we can't allocate the fees to the recipient.
					log::error!(
						"Could not deposit to tournament account, it probably doesn't exist."
					);
					// Reabsorb the credit; it can still be used.
					final_fee.subsume(credit);
				}
			}
		}

		Ok(final_fee)
	}

	fn deposit_into_treasury(
		key: &AccountId,
		credit: fungible::Credit<AccountId, Balances>,
	) -> Result<(), DispatchError> {
		if let Err(_credit) = W::Assets::resolve(key, credit) {
			// We decide to continue here, because the error has nothing to do with the
			// account sending the transaction. It would be a bad user experience if
			// the transaction fails because we can't allocate the fees to the recipient.
			log::error!(
				"Could deposit to treasury, it probably doesn't exist, burning the credit..."
			);
		}
		Ok(())
	}
}
