use crate::withdraw_credit::WithdrawCredit;
use core::{fmt::Debug, marker::PhantomData};
use frame_support::{
	pallet_prelude::DispatchError,
	traits::{
		fungible, fungibles,
		tokens::{Balance as TokenBalance, Preservation},
		Defensive, Imbalance,
	},
	BoundedVec,
};
use parity_scale_codec::{Decode, Encode, EncodeLike, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::TokenError;

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
pub struct Payment<AccountId, Balance> {
	beneficiary: AccountId,
	amount: Balance,
}

impl<AccountId, Balance> Payment<AccountId, Balance> {
	pub fn new(beneficiary: AccountId, amount: Balance) -> Self {
		Self { beneficiary, amount }
	}
}

#[derive(Debug, Encode, Decode, PartialEq, Eq, Clone, MaxEncodedLen, TypeInfo)]
pub enum PaymentKind<PaymentAssetId> {
	Asset(PaymentAssetId),
	Voucher,
}

/// Abstraction of withdrawing fees in one asset and return allocating the fees in the same or
/// another asset.
pub trait FeeHandler {
	type AccountId;

	type Payment: Clone + Eq + Debug + TypeInfo + MaxEncodedLen + EncodeLike + Decode;

	/// Scalar type of the fee balance.
	type Balance;

	type AffiliateFeeIdentifier;
	type TournamentFeeIdentifier;

	/// Withdraws the `base_fee` denominated in `payment` allocate shares of the base fee to
	/// the affiliate, tournament, and treasury if implemented.
	fn withdraw_and_pay_fees(
		payer: &Self::AccountId,
		payment: Self::Payment,
		base_fee: Self::Balance,
		tournament_id: &Self::TournamentFeeIdentifier,
		affiliate_id: &Self::AffiliateFeeIdentifier,
		treasury_pot: &Self::AccountId,
	) -> Result<(), DispatchError>;

	/// Withdraws the `amount` denominated in `payment` and allocates it fully to the
	/// `treasury_pot`.
	fn withdraw_and_deposit_into_treasury(
		who: &Self::AccountId,
		payment: Self::Payment,
		treasury_pot: &Self::AccountId,
		amount: Self::Balance,
	) -> Result<(), DispatchError>;
}

pub type AffiliateFeeDistribution<AccountId, Balance, MaxDistribution> =
	BoundedVec<Payment<AccountId, Balance>, MaxDistribution>;
pub type TournamentFeeDistribution<AccountId, Balance> = Payment<AccountId, Balance>;

pub struct AssetGameFeeHandler<
	AccountId,
	Balance,
	PaymentAssets,
	WithdrawPaymentAssets,
	VoucherAsset,
	WithdrawVoucherAsset,
	Affiliate,
	AffiliateMaxDistribution,
	Tournament,
> {
	_phantom: PhantomData<(
		AccountId,
		Balance,
		PaymentAssets,
		WithdrawPaymentAssets,
		VoucherAsset,
		WithdrawVoucherAsset,
		Affiliate,
		AffiliateMaxDistribution,
		Tournament,
	)>,
}

impl<
		AccountId,
		Balance,
		PaymentAssets,
		WPA,
		VoucherAsset,
		WVA,
		Affiliate,
		AffiliateMaxDistribution,
		Tournament,
	> FeeHandler
	for AssetGameFeeHandler<
		AccountId,
		Balance,
		PaymentAssets,
		WPA,
		VoucherAsset,
		WVA,
		Affiliate,
		AffiliateMaxDistribution,
		Tournament,
	> where
	// This is satisfied by the `pallet-assets`, `pallet-asset-conversion` and the `NativeAndAssets`
	// struct.
	Balance: TokenBalance,
	PaymentAssets: fungibles::Inspect<AccountId, Balance = Balance, AssetId = WPA::AssetId>
		+ fungibles::Balanced<AccountId>,
	WPA: WithdrawCredit<
		AccountId = AccountId,
		Assets = PaymentAssets,
		Credit = fungibles::Credit<AccountId, PaymentAssets>,
		Balance = Balance,
	>,
	WPA::AssetId: 'static,

	VoucherAsset: fungible::Inspect<AccountId, Balance = Balance> + fungible::Balanced<AccountId>,
	WVA: WithdrawCredit<
		AccountId = AccountId,
		AssetId = (),
		Assets = VoucherAsset,
		Credit = fungible::Credit<AccountId, VoucherAsset>,
		Balance = Balance,
	>,

	Affiliate: DistributeFee<
		AccountId = AccountId,
		Balance = Balance,
		FeeDistribution = AffiliateFeeDistribution<AccountId, Balance, AffiliateMaxDistribution>,
	>,
	Tournament: DistributeFee<
		AccountId = AccountId,
		Balance = Balance,
		FeeDistribution = TournamentFeeDistribution<AccountId, Balance>,
	>,
{
	type AccountId = AccountId;
	type Payment = PaymentKind<WPA::AssetId>;
	type Balance = Balance;
	type AffiliateFeeIdentifier = Affiliate::FeeIdentifier;
	type TournamentFeeIdentifier = Tournament::FeeIdentifier;

	fn withdraw_and_pay_fees(
		payer: &Self::AccountId,
		payment: Self::Payment,
		base_fee: Self::Balance,
		tournament_id: &Self::TournamentFeeIdentifier,
		affiliate_id: &Self::AffiliateFeeIdentifier,
		treasury_pot: &Self::AccountId,
	) -> Result<(), DispatchError> {
		match payment {
			PaymentKind::Asset(payment) => {
				// The credit may be in any asset as implemented by `WithdrawAsset`.
				let fee_credit = WPA::withdraw_credit(payer, payment, base_fee)?;

				let remaining_credit =
					Self::try_propagate_tournament_fee(fee_credit, payer, tournament_id)?;

				let remaining_credit2 =
					Self::try_propagate_chain_fee(remaining_credit, payer, affiliate_id)?;

				Self::deposit_into_treasury(treasury_pot, remaining_credit2)
			},
			PaymentKind::Voucher => Self::try_consume_vouchers(payer, base_fee),
		}
	}

	fn withdraw_and_deposit_into_treasury(
		who: &Self::AccountId,
		payment: Self::Payment,
		treasury_pot: &Self::AccountId,
		amount: Self::Balance,
	) -> Result<(), DispatchError> {
		match payment {
			PaymentKind::Asset(asset_id) => {
				let credit = WPA::withdraw_credit(who, asset_id, amount)?;
				Self::deposit_into_treasury(treasury_pot, credit)
			},
			PaymentKind::Voucher => Self::try_consume_vouchers(who, amount),
		}
	}
}

impl<
		AccountId,
		Balance,
		PaymentAssets,
		WPA,
		VoucherAsset,
		WVA,
		Affiliate,
		AffiliateMaxDistribution,
		Tournament,
	>
	AssetGameFeeHandler<
		AccountId,
		Balance,
		PaymentAssets,
		WPA,
		VoucherAsset,
		WVA,
		Affiliate,
		AffiliateMaxDistribution,
		Tournament,
	> where
	Balance: TokenBalance,
	PaymentAssets: fungibles::Inspect<AccountId, Balance = Balance, AssetId = WPA::AssetId>
		+ fungibles::Balanced<AccountId>,
	WPA: WithdrawCredit<
		AccountId = AccountId,
		Assets = PaymentAssets,
		Credit = fungibles::Credit<AccountId, PaymentAssets>,
		Balance = Balance,
	>,

	VoucherAsset: fungible::Inspect<AccountId, Balance = Balance> + fungible::Balanced<AccountId>,
	WVA: WithdrawCredit<
		AccountId = AccountId,
		AssetId = (),
		Assets = VoucherAsset,
		Credit = fungible::Credit<AccountId, VoucherAsset>,
		Balance = Balance,
	>,

	Affiliate: DistributeFee<
		AccountId = AccountId,
		Balance = Balance,
		FeeDistribution = AffiliateFeeDistribution<AccountId, Balance, AffiliateMaxDistribution>,
	>,
	Tournament: DistributeFee<
		AccountId = AccountId,
		Balance = Balance,
		FeeDistribution = TournamentFeeDistribution<AccountId, Balance>,
	>,
{
	/// Distributes an already withdrawn `fee_credit` to the affiliates of `account`.
	///
	/// Returns the remaining `fee_credit` after this operation.
	fn try_propagate_chain_fee(
		fee_credit: WPA::Credit,
		account: &WPA::AccountId,
		identifier: &Affiliate::FeeIdentifier,
	) -> Result<WPA::Credit, DispatchError> {
		let mut final_fee = fee_credit;

		if let Some(a) = Affiliate::distribute_fee(final_fee.peek(), account, identifier) {
			for allocation in a {
				if allocation.amount > 0_u32.into() {
					let affiliate_fee = final_fee.extract(allocation.amount);

					if let Err(credit) =
						WPA::Assets::resolve(&allocation.beneficiary, affiliate_fee)
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
		fee_credit: WPA::Credit,
		account: &WPA::AccountId,
		identifier: &Tournament::FeeIdentifier,
	) -> Result<WPA::Credit, DispatchError> {
		let mut final_fee = fee_credit;

		if let Some(allocation) = Tournament::distribute_fee(final_fee.peek(), account, identifier)
		{
			if allocation.amount > 0_u32.into() {
				let tournament_credit = final_fee.extract(allocation.amount);

				if let Err(credit) =
					WPA::Assets::resolve(&allocation.beneficiary, tournament_credit)
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
		key: &WPA::AccountId,
		credit: WPA::Credit,
	) -> Result<(), DispatchError> {
		if let Err(_credit) = WPA::Assets::resolve(key, credit) {
			// We decide to continue here, because the error has nothing to do with the
			// account sending the transaction. It would be a bad user experience if
			// the transaction fails because we can't allocate the fees to the recipient.
			log::error!(
				"Could deposit to treasury, it probably doesn't exist, burning the credit..."
			);
		}
		Ok(())
	}

	fn try_consume_vouchers(who: &AccountId, amount: Balance) -> Result<(), DispatchError> {
		let debt = WVA::Assets::rescind(amount);
		WVA::Assets::settle(who, debt, Preservation::Protect)
			.map(|_| ())
			.map_err(|_| DispatchError::Token(TokenError::FundsUnavailable))
	}
}

pub struct NativeGameFeeHandler<
	AccountId,
	Balance,
	PaymentAsset,
	WithdrawPaymentAsset,
	VoucherAsset,
	WithdrawVoucherAsset,
	Affiliate,
	AffiliateMaxDistribution,
	Tournament,
> {
	_phantom: PhantomData<(
		AccountId,
		Balance,
		PaymentAsset,
		WithdrawPaymentAsset,
		VoucherAsset,
		WithdrawVoucherAsset,
		Affiliate,
		AffiliateMaxDistribution,
		Tournament,
	)>,
}

impl<
		AccountId,
		Balance,
		PaymentAsset,
		WPA,
		VoucherAsset,
		WVA,
		Affiliate,
		AffiliateMaxDistribution,
		Tournament,
	> FeeHandler
	for NativeGameFeeHandler<
		AccountId,
		Balance,
		PaymentAsset,
		WPA,
		VoucherAsset,
		WVA,
		Affiliate,
		AffiliateMaxDistribution,
		Tournament,
	> where
	Balance: TokenBalance,
	// This is satisfied by the `pallet-balances`.
	PaymentAsset: fungible::Inspect<AccountId, Balance = Balance> + fungible::Balanced<AccountId>,
	WPA: WithdrawCredit<
		AccountId = AccountId,
		AssetId = (),
		Assets = PaymentAsset,
		Credit = fungible::Credit<AccountId, PaymentAsset>,
		Balance = Balance,
	>,

	VoucherAsset: fungible::Inspect<AccountId, Balance = Balance> + fungible::Balanced<AccountId>,
	WVA: WithdrawCredit<
		AccountId = AccountId,
		AssetId = (),
		Assets = VoucherAsset,
		Credit = fungible::Credit<AccountId, VoucherAsset>,
		Balance = Balance,
	>,

	Affiliate: DistributeFee<
		AccountId = AccountId,
		Balance = Balance,
		FeeDistribution = AffiliateFeeDistribution<AccountId, Balance, AffiliateMaxDistribution>,
	>,
	Tournament: DistributeFee<
		AccountId = AccountId,
		Balance = Balance,
		FeeDistribution = TournamentFeeDistribution<AccountId, Balance>,
	>,
{
	type AccountId = AccountId;
	type Payment = PaymentKind<()>;
	type Balance = Balance;
	type AffiliateFeeIdentifier = Affiliate::FeeIdentifier;
	type TournamentFeeIdentifier = Tournament::FeeIdentifier;

	fn withdraw_and_pay_fees(
		payer: &Self::AccountId,
		payment: Self::Payment,
		base_fee: Self::Balance,
		tournament_id: &Self::TournamentFeeIdentifier,
		affiliate_id: &Self::AffiliateFeeIdentifier,
		treasury_pot: &Self::AccountId,
	) -> Result<(), DispatchError> {
		match payment {
			PaymentKind::Asset(payment_asset) => {
				// The credit may be in any asset as implemented by `WithdrawAsset`.
				let fee_credit = WPA::withdraw_credit(payer, payment_asset, base_fee)?;

				let remaining_credit =
					Self::try_propagate_tournament_fee(fee_credit, payer, tournament_id)?;

				let remaining_credit2 =
					Self::try_propagate_chain_fee(remaining_credit, payer, affiliate_id)?;

				Self::deposit_into_treasury(treasury_pot, remaining_credit2)
			},
			PaymentKind::Voucher => Self::try_consume_vouchers(payer, base_fee),
		}
	}

	fn withdraw_and_deposit_into_treasury(
		who: &Self::AccountId,
		payment: Self::Payment,
		treasury_pot: &Self::AccountId,
		amount: Self::Balance,
	) -> Result<(), DispatchError> {
		match payment {
			PaymentKind::Asset(payment_asset) => {
				let credit = WPA::withdraw_credit(who, payment_asset, amount)?;
				Self::deposit_into_treasury(treasury_pot, credit)
			},
			PaymentKind::Voucher => Self::try_consume_vouchers(who, amount),
		}
	}
}

impl<
		AccountId,
		Balance,
		PaymentAsset,
		WPA,
		VoucherAsset,
		WVA,
		Affiliate,
		AffiliateMaxDistribution,
		Tournament,
	>
	NativeGameFeeHandler<
		AccountId,
		Balance,
		PaymentAsset,
		WPA,
		VoucherAsset,
		WVA,
		Affiliate,
		AffiliateMaxDistribution,
		Tournament,
	> where
	Balance: TokenBalance,
	PaymentAsset: fungible::Inspect<AccountId, Balance = Balance> + fungible::Balanced<AccountId>,
	WPA: WithdrawCredit<
		AccountId = AccountId,
		AssetId = (),
		Assets = PaymentAsset,
		Credit = fungible::Credit<AccountId, PaymentAsset>,
		Balance = Balance,
	>,

	VoucherAsset: fungible::Inspect<AccountId, Balance = Balance> + fungible::Balanced<AccountId>,
	WVA: WithdrawCredit<
		AccountId = AccountId,
		AssetId = (),
		Assets = VoucherAsset,
		Credit = fungible::Credit<AccountId, VoucherAsset>,
		Balance = Balance,
	>,

	Affiliate: DistributeFee<
		AccountId = AccountId,
		Balance = Balance,
		FeeDistribution = AffiliateFeeDistribution<AccountId, Balance, AffiliateMaxDistribution>,
	>,
	Tournament: DistributeFee<
		AccountId = AccountId,
		Balance = Balance,
		FeeDistribution = TournamentFeeDistribution<AccountId, Balance>,
	>,
{
	/// Distributes an already withdrawn `fee_credit` to the affiliates of `account`.
	///
	/// Returns the remaining `fee_credit` after this operation.
	fn try_propagate_chain_fee(
		fee_credit: WPA::Credit,
		account: &WPA::AccountId,
		identifier: &Affiliate::FeeIdentifier,
	) -> Result<WPA::Credit, DispatchError> {
		let mut final_fee = fee_credit;

		if let Some(a) = Affiliate::distribute_fee(final_fee.peek(), account, identifier) {
			for allocation in a {
				if allocation.amount > 0_u32.into() {
					let affiliate_fee = final_fee.extract(allocation.amount);
					if let Err(credit) =
						WPA::Assets::resolve(&allocation.beneficiary, affiliate_fee)
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
		fee_credit: WPA::Credit,
		account: &WPA::AccountId,
		identifier: &Tournament::FeeIdentifier,
	) -> Result<WPA::Credit, DispatchError> {
		let mut final_fee = fee_credit;

		if let Some(allocation) = Tournament::distribute_fee(final_fee.peek(), account, identifier)
		{
			if allocation.amount > 0_u32.into() {
				let tournament_credit = final_fee.extract(allocation.amount);

				if let Err(credit) =
					WPA::Assets::resolve(&allocation.beneficiary, tournament_credit)
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
		key: &WPA::AccountId,
		credit: WPA::Credit,
	) -> Result<(), DispatchError> {
		if let Err(_credit) = WPA::Assets::resolve(key, credit) {
			// We decide to continue here, because the error has nothing to do with the
			// account sending the transaction. It would be a bad user experience if
			// the transaction fails because we can't allocate the fees to the recipient.
			log::error!(
				"Could deposit to treasury, it probably doesn't exist, burning the credit..."
			);
		}
		Ok(())
	}

	fn try_consume_vouchers(who: &AccountId, amount: Balance) -> Result<(), DispatchError> {
		let debt = WVA::Assets::rescind(amount);
		WVA::Assets::settle(who, debt, Preservation::Protect)
			.map(|_| ())
			.map_err(|_| DispatchError::Token(TokenError::FundsUnavailable))
	}
}
