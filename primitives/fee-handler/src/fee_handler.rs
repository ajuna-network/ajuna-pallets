use core::marker::PhantomData;
use frame_support::{
	pallet_prelude::DispatchError,
	sp_runtime::TokenError,
	traits::{fungibles, fungibles::Balanced, ConstU32, Get},
	BoundedVec,
};
use pallet_asset_conversion::{CreditOf, Pallet as AssetConversion};

pub trait DenominatedToFee {
	type Balance;

	/// The asset id of the asset denominating the fee.
	type DenominatedAssetId;

	/// Kind of asset to be swapped.
	type AssetKind;

	/// Converts the
	fn nominal_asset_fee(
		denominated: Self::Balance,
		asset: Self::AssetKind,
	) -> Result<Self::Balance, DispatchError>;
}

pub struct ConvertToNativeFee<A, D, T>(PhantomData<(A, D, T)>);

impl<D, N, T> DenominatedToFee for ConvertToNativeFee<D, N, T>
where
	D: Get<T::AssetKind>,
	N: Get<T::AssetKind>,
	T: pallet_asset_conversion::Config,
{
	type Balance = T::Balance;
	type DenominatedAssetId = D;

	type AssetKind = T::AssetKind;

	fn nominal_asset_fee(
		denominated: Self::Balance,
		_: Self::AssetKind,
	) -> Result<Self::Balance, DispatchError> {
		// Convert the fee denominated in `D` into the native fee.
		let asset_fee = AssetConversion::<T>::quote_price_tokens_for_exact_tokens(
			D::get(),
			N::get(),
			denominated,
			true,
		)
		.ok_or(DispatchError::Other("can't convert"))?;

		Ok(asset_fee)
	}
}

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
	) -> BoundedVec<Payment<Self::AccountId, Self::Balance>, Self::MaxDistributions>;
}

/// Payment to be executed.
pub struct Payment<AccountId, Balance> {
	beneficiary: AccountId,
	amount: Balance,
}

pub trait EnsureWhitelistedAsset {
	type AssetId;

	fn ensure_whitelisted(asset_id: &Self::AssetId) -> Result<(), DispatchError>;
}

/// Abstraction to withdraw some asset from an account, and return a credit in some asset.
///
/// Currently, this is intended to be implemented by the `SwapCredit` adapter we have in the
/// ajuna-parachain, which will convert whitelisted assets into the native currency via the
/// `pallet-asset-conversion` and return a credit in the native balance, which can then be allocated
/// to the affiliates, or the specific treasury pots.
pub trait WithdrawFee {
	type AccountId;
	type AssetId;
	type Balance;
	type Credit;

	fn withdraw_fee(
		payer: &Self::AccountId,
		asset_id: Self::AssetId,
		fee: Self::Balance,
	) -> Result<Self::Credit, DispatchError>;
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

pub struct GameFeeHandler<AssetConversion, WithdrawAsset, DepositAsset, Affiliate, Tournament> {
	_phantom: PhantomData<(AssetConversion, WithdrawAsset, DepositAsset, Affiliate, Tournament)>,
}

impl<T, WithdrawAsset, DepositAsset, Affiliate, Tournament> FeeHandler
	for GameFeeHandler<T, WithdrawAsset, DepositAsset, Affiliate, Tournament>
where
	DepositAsset: Get<T::AssetKind>,
	T: pallet_asset_conversion::Config + frame_system::Config,
	T::Assets: fungibles::Inspect<
		<T as frame_system::Config>::AccountId,
		Balance = T::Balance,
		AssetId = T::AssetKind,
	>,

	WithdrawAsset: WithdrawFee<
		AccountId = <T as frame_system::Config>::AccountId,
		AssetId = T::AssetKind,
		Balance = T::Balance,
		Credit = CreditOf<T>,
	>,

	Affiliate: DistributeFee<AccountId = T::AccountId, Balance = T::Balance>,
	Tournament: DistributeFee<
		AccountId = T::AccountId,
		Balance = T::Balance,
		MaxDistributions = ConstU32<1>,
	>,
{
	type AccountId = T::AccountId;
	type AssetId = T::AssetKind;
	type Balance = T::Balance;
	type AffiliateFeeIdentifier = Affiliate::FeeIdentifier;
	type TournamentFeeIdentifier = Tournament::FeeIdentifier;

	fn withdraw_and_pay_fees(
		payer: &Self::AccountId,
		payment_asset: Self::AssetId,
		base_fee: Self::Balance,
		tournament_id: &Self::TournamentFeeIdentifier,
		affiliate_id: &Self::AffiliateFeeIdentifier,
		treasury_pot: &T::AccountId,
	) -> Result<(), DispatchError> {
		let fee_credit = WithdrawAsset::withdraw_fee(payer, payment_asset, base_fee)?;

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
		let credit = WithdrawAsset::withdraw_fee(who, asset_id, amount)?;
		Self::deposit_into_treasury(treasury_pot, credit)
	}
}

impl<T, WithdrawAsset, DepositAsset, Affiliate, Tournament>
	GameFeeHandler<T, WithdrawAsset, DepositAsset, Affiliate, Tournament>
where
	DepositAsset: Get<T::AssetKind>,
	T: pallet_asset_conversion::Config,
	T::Assets: fungibles::Inspect<T::AccountId, Balance = T::Balance, AssetId = T::AssetKind>,

	WithdrawAsset: WithdrawFee<AssetId = T::AssetKind, Balance = T::Balance, Credit = CreditOf<T>>,

	Affiliate: DistributeFee<AccountId = T::AccountId, Balance = T::Balance>,
	Tournament: DistributeFee<
		AccountId = T::AccountId,
		Balance = T::Balance,
		MaxDistributions = ConstU32<1>,
	>,
{
	/// Distributes an already withdrawn `fee_credit` to the affiliates of `account`.
	///
	/// Returns the remaining `fee_credit` after this operation.
	fn try_propagate_chain_fee(
		fee_credit: CreditOf<T>,
		account: &T::AccountId,
		identifier: &Affiliate::FeeIdentifier,
	) -> Result<CreditOf<T>, DispatchError> {
		let mut final_fee = fee_credit;

		for allocation in Affiliate::distribute_fee(final_fee.peek(), account, identifier) {
			if allocation.amount > 0_u32.into() {
				let affiliate_fee = final_fee.extract(allocation.amount);
				T::Assets::resolve(&allocation.beneficiary, affiliate_fee)
					.map_err(|_| DispatchError::Token(TokenError::CannotCreate))?;
			}
		}

		Ok(final_fee)
	}

	/// Deposits the tournament fee into the tournament pot. The fee is taken from a previously
	/// withdrawn `fee_credit`.
	///
	/// Returns the remaining credit after taking the fee.
	fn try_propagate_tournament_fee(
		fee_credit: CreditOf<T>,
		account: &T::AccountId,
		identifier: &Tournament::FeeIdentifier,
	) -> Result<CreditOf<T>, DispatchError> {
		let fees = Tournament::distribute_fee(fee_credit.peek(), account, identifier);

		assert_eq!(fees.len(), 1, "invalid fee provider implementation");

		let allocation = &fees[0];

		if allocation.amount > 0_u32.into() {
			let (remaining_credit, tournament_credit) = fee_credit.split(allocation.amount);
			T::Assets::resolve(&allocation.beneficiary, tournament_credit)
				.map_err(|_| DispatchError::Token(TokenError::CannotCreate))?;
			Ok(remaining_credit)
		} else {
			Ok(fee_credit)
		}
	}

	fn deposit_into_treasury(key: &T::AccountId, credit: CreditOf<T>) -> Result<(), DispatchError> {
		T::Assets::resolve(key, credit).map_err(|_| DispatchError::Token(TokenError::CannotCreate))
	}
}
