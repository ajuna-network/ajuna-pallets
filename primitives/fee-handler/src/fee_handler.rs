use ajuna_primitives::{runtime_types::AccountId, treasury_manager::TreasuryManager};
use core::marker::PhantomData;
use frame_support::{
	pallet_prelude::DispatchError,
	sp_runtime::{traits::CheckedSub, TokenError},
	traits::{fungibles, fungibles::Balanced, tokens::Precision, Get},
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

pub trait FeeProvider {
	type AccountId;
	type FeeIdentifier;
	type FeeCurrency;
	type FeeOutput;

	fn get_fee_from(
		base_fee: Self::FeeCurrency,
		account: &Self::AccountId,
		identifier: &Self::FeeIdentifier,
	) -> Self::FeeOutput;
}

pub trait EnsureWhitelistedAsset {
	type AssetId;

	fn ensure_whitelisted(asset_id: &Self::AssetId) -> Result<(), DispatchError>;
}

pub trait WithdrawFee {
	type AccountId;
	type AssetId;
	type Balance;
	type LiquidityInfo;

	fn withdraw_fee(
		payer: &Self::AccountId,
		asset_id: Self::AssetId,
		fee: Self::Balance,
	) -> Result<Self::LiquidityInfo, DispatchError>;
}

pub trait FeeHandler {
	type AccountId;

	type AssetId;

	/// Scalar type of the fee balance.
	type Balance;

	type AffiliateFeeIdentifier;
	type TournamentFeeIdentifier;

	fn withdraw_and_pay_fees(
		payer: &Self::AccountId,
		payment_asset: Self::AssetId,
		base_fee: Self::Balance,
		tournament_id: &Self::TournamentFeeIdentifier,
		affiliate_id: &Self::AffiliateFeeIdentifier,
		treasury_pot: &Self::AccountId,
	) -> Result<(), DispatchError>;

	fn withdraw_and_deposit_into_treasury(
		depositor: &Self::AccountId,
		key: &Self::AccountId,
		fee: Self::Balance,
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
		LiquidityInfo = CreditOf<T>,
	>,

	Affiliate: FeeProvider<
		AccountId = T::AccountId,
		FeeCurrency = T::Balance,
		FeeOutput = Vec<(T::Balance, T::AccountId)>,
	>,
	Tournament: FeeProvider<
		AccountId = T::AccountId,
		FeeCurrency = T::Balance,
		FeeOutput = (T::Balance, T::AccountId),
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
		depositor: &Self::AccountId,
		key: &Self::AccountId,
		fee: Self::Balance,
	) -> Result<(), DispatchError> {
		todo!()
	}
}

impl<T, WithdrawAsset, DepositAsset, Affiliate, Tournament>
	GameFeeHandler<T, WithdrawAsset, DepositAsset, Affiliate, Tournament>
where
	DepositAsset: Get<T::AssetKind>,
	T: pallet_asset_conversion::Config,
	T::Assets: fungibles::Inspect<T::AccountId, Balance = T::Balance, AssetId = T::AssetKind>,

	WithdrawAsset:
		WithdrawFee<AssetId = T::AssetKind, Balance = T::Balance, LiquidityInfo = CreditOf<T>>,

	Affiliate: FeeProvider<
		AccountId = T::AccountId,
		FeeCurrency = T::Balance,
		FeeOutput = Vec<(T::Balance, T::AccountId)>,
	>,
	Tournament: FeeProvider<
		AccountId = T::AccountId,
		FeeCurrency = T::Balance,
		FeeOutput = (T::Balance, T::AccountId),
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

		for (transfer_fee, chain_account) in
			Affiliate::get_fee_from(final_fee.peek(), account, identifier)
		{
			if transfer_fee > 0_u32.into() {
				let affiliate_fee = final_fee.extract(transfer_fee);
				T::Assets::resolve(&chain_account, affiliate_fee)
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
		let (tournament_fee, tournament_account) =
			Tournament::get_fee_from(fee_credit.peek(), account, identifier);

		if tournament_fee > 0_u32.into() {
			let (remaining_credit, tournament_credit) = fee_credit.split(tournament_fee);
			T::Assets::resolve(&tournament_account, tournament_credit)
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
