use crate::withdraw_credit::WithdrawCredit;
use core::marker::PhantomData;
use frame_support::{
	pallet_prelude::DispatchError,
	sp_runtime::TokenError,
	traits::{fungible, fungibles, ConstU32, Imbalance},
	BoundedVec,
};
use parity_scale_codec::{Decode, Encode};

/// Distributes shares of a base fee to some beneficiaries.
pub trait DistributeFee {
	/// AccountId type used.
	type AccountId;

	/// Scalar balance type.
	type Balance;

	/// Fee identifier used to derive the fee distribution.
	type FeeIdentifier;

	/// Maximum number of distributions of shares from the base fee to beneficiaries.
	type MaxDistributions;

	fn distribute_fee(
		base_fee: Self::Balance,
		account: &Self::AccountId,
		identifier: &Self::FeeIdentifier,
	) -> Option<BoundedVec<Payment<Self::AccountId, Self::Balance>, Self::MaxDistributions>>;
}

/// Payment to be executed.
#[derive(Debug, Encode, Decode, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Payment<AccountId, Balance> {
	beneficiary: AccountId,
	amount: Balance,
}

impl<AccountId, Balance> Payment<AccountId, Balance> {
	pub fn new(beneficiary: AccountId, amount: Balance) -> Self {
		Self { beneficiary, amount }
	}
}

/// Abstraction of withdrawing fees in one asset and return allocating the fees in the same or
/// another asset.
pub trait FeeHandler {
	type AccountId;

	type AssetId;

	/// Scalar type of the fee balance.
	type Balance;

	type AffiliateFeeIdentifier;
	type TournamentFeeIdentifier;

	/// Withdraws the `base_fee` denominated in `payment_asset` allocate shares of the base fee to
	/// the affiliate, tournament, and treasury if implemented.
	fn withdraw_and_pay_fees(
		payer: &Self::AccountId,
		payment_asset: Self::AssetId,
		base_fee: Self::Balance,
		tournament_id: &Self::TournamentFeeIdentifier,
		affiliate_id: &Self::AffiliateFeeIdentifier,
		treasury_pot: &Self::AccountId,
	) -> Result<(), DispatchError>;

	/// Withdraws the `base_fee` denominated in `payment_asset` allocate it fully to the
	/// `treasury_pot`.
	fn withdraw_and_deposit_into_treasury(
		who: &Self::AccountId,
		asset_id: Self::AssetId,
		treasury_pot: &Self::AccountId,
		amount: Self::Balance,
	) -> Result<(), DispatchError>;
}

pub struct AssetGameFeeHandler<AccountId, Assets, WithdrawAsset, Affiliate, Tournament> {
	_phantom: PhantomData<(AccountId, Assets, WithdrawAsset, Affiliate, Tournament)>,
}

impl<AccountId, Assets, W, Affiliate, Tournament> FeeHandler
	for AssetGameFeeHandler<AccountId, Assets, W, Affiliate, Tournament>
where
	Assets: fungibles::Inspect<AccountId, Balance = W::Balance, AssetId = W::AssetId>
		+ fungibles::Balanced<AccountId>,

	W: WithdrawCredit<
		AccountId = AccountId,
		Assets = Assets,
		Credit = fungibles::Credit<AccountId, Assets>,
	>,

	Affiliate: DistributeFee<AccountId = W::AccountId, Balance = W::Balance>,
	Tournament:
		DistributeFee<AccountId = AccountId, Balance = W::Balance, MaxDistributions = ConstU32<1>>,
{
	type AccountId = AccountId;
	type AssetId = W::AssetId;
	type Balance = W::Balance;
	type AffiliateFeeIdentifier = Affiliate::FeeIdentifier;
	type TournamentFeeIdentifier = Tournament::FeeIdentifier;

	fn withdraw_and_pay_fees(
		payer: &Self::AccountId,
		payment_asset: Self::AssetId,
		base_fee: Self::Balance,
		tournament_id: &Self::TournamentFeeIdentifier,
		affiliate_id: &Self::AffiliateFeeIdentifier,
		treasury_pot: &Self::AccountId,
	) -> Result<(), DispatchError> {
		// The credit may be in any asset as implemented by `WithdrawAsset`.
		let fee_credit = W::withdraw_credit(payer, payment_asset, base_fee)?;

		let remaining_credit =
			Self::try_propagate_tournament_fee(fee_credit, payer, tournament_id)?;

		let remaining_credit2 =
			Self::try_propagate_chain_fee(remaining_credit, payer, affiliate_id)?;

		Self::deposit_into_treasury(treasury_pot, remaining_credit2)
	}

	fn withdraw_and_deposit_into_treasury(
		who: &Self::AccountId,
		asset_id: Self::AssetId,
		treasury_pot: &Self::AccountId,
		amount: Self::Balance,
	) -> Result<(), DispatchError> {
		let credit = W::withdraw_credit(who, asset_id, amount)?;
		Self::deposit_into_treasury(treasury_pot, credit)
	}
}

impl<AccountId, Assets, W, Affiliate, Tournament>
	AssetGameFeeHandler<AccountId, Assets, W, Affiliate, Tournament>
where
	Assets: fungibles::Inspect<AccountId, Balance = W::Balance, AssetId = W::AssetId>
		+ fungibles::Balanced<AccountId>,

	W: WithdrawCredit<
		AccountId = AccountId,
		Assets = Assets,
		Credit = fungibles::Credit<AccountId, Assets>,
	>,

	Affiliate: DistributeFee<AccountId = W::AccountId, Balance = W::Balance>,
	Tournament:
		DistributeFee<AccountId = AccountId, Balance = W::Balance, MaxDistributions = ConstU32<1>>,
{
	/// Distributes an already withdrawn `fee_credit` to the affiliates of `account`.
	///
	/// Returns the remaining `fee_credit` after this operation.
	fn try_propagate_chain_fee(
		fee_credit: W::Credit,
		account: &W::AccountId,
		identifier: &Affiliate::FeeIdentifier,
	) -> Result<W::Credit, DispatchError> {
		let mut final_fee = fee_credit;

		if let Some(a) = Affiliate::distribute_fee(final_fee.peek(), account, identifier) {
			for allocation in a {
				if allocation.amount > 0_u32.into() {
					let affiliate_fee = final_fee.extract(allocation.amount);
					W::Assets::resolve(&allocation.beneficiary, affiliate_fee)
						.map_err(|_| DispatchError::Token(TokenError::CannotCreate))?;
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
		fee_credit: W::Credit,
		account: &W::AccountId,
		identifier: &Tournament::FeeIdentifier,
	) -> Result<W::Credit, DispatchError> {
		let mut final_fee = fee_credit;

		if let Some(fee) = Tournament::distribute_fee(final_fee.peek(), account, identifier) {
			assert_eq!(fee.len(), 1, "invalid fee provider implementation");
			let allocation = &fee[0];

			if allocation.amount > 0_u32.into() {
				let tournament_credit = final_fee.extract(allocation.amount);
				W::Assets::resolve(&allocation.beneficiary, tournament_credit)
					.map_err(|_| DispatchError::Token(TokenError::CannotCreate))?;
			}
		}

		Ok(final_fee)
	}

	fn deposit_into_treasury(key: &W::AccountId, credit: W::Credit) -> Result<(), DispatchError> {
		W::Assets::resolve(key, credit).map_err(|_| DispatchError::Token(TokenError::CannotCreate))
	}
}

pub struct NativeGameFeeHandler<AccountId, Balances, WithdrawAsset, Affiliate, Tournament> {
	_phantom: PhantomData<(AccountId, Balances, WithdrawAsset, Affiliate, Tournament)>,
}

impl<AccountId, Balances, W, Affiliate, Tournament> FeeHandler
	for NativeGameFeeHandler<AccountId, Balances, W, Affiliate, Tournament>
where
	Balances: fungible::Inspect<AccountId, Balance = W::Balance> + fungible::Balanced<AccountId>,

	W: WithdrawCredit<
		AccountId = AccountId,
		Assets = Balances,
		AssetId = (),
		Credit = fungible::Credit<AccountId, Balances>,
	>,

	Affiliate: DistributeFee<AccountId = W::AccountId, Balance = W::Balance>,
	Tournament:
		DistributeFee<AccountId = AccountId, Balance = W::Balance, MaxDistributions = ConstU32<1>>,
{
	type AccountId = AccountId;
	type AssetId = ();
	type Balance = W::Balance;
	type AffiliateFeeIdentifier = Affiliate::FeeIdentifier;
	type TournamentFeeIdentifier = Tournament::FeeIdentifier;

	fn withdraw_and_pay_fees(
		payer: &Self::AccountId,
		payment_asset: Self::AssetId,
		base_fee: Self::Balance,
		tournament_id: &Self::TournamentFeeIdentifier,
		affiliate_id: &Self::AffiliateFeeIdentifier,
		treasury_pot: &Self::AccountId,
	) -> Result<(), DispatchError> {
		// The credit may be in any asset as implemented by `WithdrawAsset`.
		let fee_credit = W::withdraw_credit(payer, payment_asset, base_fee)?;

		let remaining_credit =
			Self::try_propagate_tournament_fee(fee_credit, payer, tournament_id)?;

		let remaining_credit2 =
			Self::try_propagate_chain_fee(remaining_credit, payer, affiliate_id)?;

		Self::deposit_into_treasury(treasury_pot, remaining_credit2)
	}

	fn withdraw_and_deposit_into_treasury(
		who: &Self::AccountId,
		asset_id: Self::AssetId,
		treasury_pot: &Self::AccountId,
		amount: Self::Balance,
	) -> Result<(), DispatchError> {
		let credit = W::withdraw_credit(who, asset_id, amount)?;
		Self::deposit_into_treasury(treasury_pot, credit)
	}
}

impl<AccountId, Assets, W, Affiliate, Tournament>
	NativeGameFeeHandler<AccountId, Assets, W, Affiliate, Tournament>
where
	Assets: fungible::Inspect<AccountId, Balance = W::Balance> + fungible::Balanced<AccountId>,

	W: WithdrawCredit<
		AccountId = AccountId,
		Assets = Assets,
		Credit = fungible::Credit<AccountId, Assets>,
	>,

	Affiliate: DistributeFee<AccountId = W::AccountId, Balance = W::Balance>,
	Tournament:
		DistributeFee<AccountId = AccountId, Balance = W::Balance, MaxDistributions = ConstU32<1>>,
{
	/// Distributes an already withdrawn `fee_credit` to the affiliates of `account`.
	///
	/// Returns the remaining `fee_credit` after this operation.
	fn try_propagate_chain_fee(
		fee_credit: W::Credit,
		account: &W::AccountId,
		identifier: &Affiliate::FeeIdentifier,
	) -> Result<W::Credit, DispatchError> {
		let mut final_fee = fee_credit;

		if let Some(a) = Affiliate::distribute_fee(final_fee.peek(), account, identifier) {
			for allocation in a {
				if allocation.amount > 0_u32.into() {
					let affiliate_fee = final_fee.extract(allocation.amount);
					W::Assets::resolve(&allocation.beneficiary, affiliate_fee)
						.map_err(|_| DispatchError::Token(TokenError::CannotCreate))?;
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
		fee_credit: W::Credit,
		account: &W::AccountId,
		identifier: &Tournament::FeeIdentifier,
	) -> Result<W::Credit, DispatchError> {
		let mut final_fee = fee_credit;

		if let Some(fee) = Tournament::distribute_fee(final_fee.peek(), account, identifier) {
			assert_eq!(fee.len(), 1, "invalid fee provider implementation");
			let allocation = &fee[0];

			if allocation.amount > 0_u32.into() {
				let tournament_credit = final_fee.extract(allocation.amount);
				W::Assets::resolve(&allocation.beneficiary, tournament_credit)
					.map_err(|_| DispatchError::Token(TokenError::CannotCreate))?;
			}
		}

		Ok(final_fee)
	}

	fn deposit_into_treasury(key: &W::AccountId, credit: W::Credit) -> Result<(), DispatchError> {
		W::Assets::resolve(key, credit).map_err(|_| DispatchError::Token(TokenError::CannotCreate))
	}
}
