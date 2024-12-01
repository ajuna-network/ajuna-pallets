use ajuna_primitives::treasury_manager::TreasuryManager;
use core::marker::PhantomData;
use frame_support::{
	pallet_prelude::DispatchError,
	sp_runtime::{traits::CheckedSub, TokenError},
	traits::{fungibles, fungibles::Balanced, tokens::Precision, Get},
};
use pallet_asset_conversion::Pallet as AssetConversion;

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

pub trait FeeHandler {
	type AccountId;

	type AssetId;

	/// Scalar type of the fee balance.
	type FeeBalance;

	type AffiliateFeeIdentifier;
	type TournamentFeeIdentifier;
	type TreasuryKey;

	fn try_propagate_chain_fee(
		payment_asset: Self::AssetId,
		base_fee: Self::FeeBalance,
		account: &Self::AccountId,
		identifier: &Self::AffiliateFeeIdentifier,
	) -> Result<Self::FeeBalance, DispatchError>;

	fn try_propagate_tournament_fee(
		payment_asset: Self::AssetId,
		base_fee: Self::FeeBalance,
		account: &Self::AccountId,
		identifier: &Self::TournamentFeeIdentifier,
	) -> Result<Self::FeeBalance, DispatchError>;

	fn deposit_fee_into_treasury(
		depositor: &Self::AccountId,
		key: &Self::TreasuryKey,
		fee: Self::FeeBalance,
	) -> Result<(), DispatchError>;
}

pub struct GameFeeHandler<AssetConversion, PaymentAsset, Affiliate, Tournament, Treasury> {
	_phantom: PhantomData<(AssetConversion, PaymentAsset, Affiliate, Tournament, Treasury)>,
}

impl<T, DepositAsset, Affiliate, Tournament, Treasury> FeeHandler
	for GameFeeHandler<T, DepositAsset, Affiliate, Tournament, Treasury>
where
	DepositAsset: Get<T::AssetKind>,
	T: pallet_asset_conversion::Config,
	T::Assets: fungibles::Inspect<T::AccountId, Balance = T::Balance, AssetId = T::AssetKind>,

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
	Treasury: TreasuryManager<AccountId = T::AccountId, Currency = T::Balance>,
{
	type AccountId = T::AccountId;
	type AssetId = T::AssetKind;
	type FeeBalance = T::Balance;
	type AffiliateFeeIdentifier = Affiliate::FeeIdentifier;
	type TournamentFeeIdentifier = Tournament::FeeIdentifier;
	type TreasuryKey = Treasury::TreasuryPotKey;

	/// Distributes an already withdrawn `fee_credit` to the affiliates of `account`.
	///
	/// Returns the remaining `fee_credit` after this operation.
	fn try_propagate_chain_fee(
		deposit_asset: Self::AssetId,
		fee_credit: Self::FeeBalance,
		account: &Self::AccountId,
		identifier: &Self::AffiliateFeeIdentifier,
	) -> Result<Self::FeeBalance, DispatchError> {
		let mut final_fee = fee_credit;

		for (transfer_fee, chain_account) in
			Affiliate::get_fee_from(fee_credit, account, identifier)
		{
			if transfer_fee > 0_u32.into() {
				final_fee = final_fee
					.checked_sub(&transfer_fee)
					.ok_or(DispatchError::Token(TokenError::FundsUnavailable))?;

				let debt = T::Assets::deposit(
					deposit_asset.clone(),
					&chain_account,
					transfer_fee.clone(),
					Precision::BestEffort,
				)?;

				if debt.peek() != transfer_fee {
					// we should never reach this arm, but it is better to double-check.
					return Err(DispatchError::Other("Unexpected error in fee payment."));
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
		deposit_asset: Self::AssetId,
		fee_credit: Self::FeeBalance,
		account: &Self::AccountId,
		identifier: &Self::TournamentFeeIdentifier,
	) -> Result<Self::FeeBalance, DispatchError> {
		let (tournament_fee, tournament_account) =
			Tournament::get_fee_from(fee_credit, account, identifier);

		if tournament_fee > 0_u32.into() {
			let remaining_credit = fee_credit
				.checked_sub(&tournament_fee)
				.ok_or(DispatchError::Token(TokenError::FundsUnavailable))?;

			let debt = T::Assets::deposit(
				deposit_asset,
				&tournament_account,
				tournament_fee.clone(),
				Precision::BestEffort,
			)?;

			if debt.peek() == tournament_fee {
				Ok(remaining_credit)
			} else {
				// we should never reach this arm, but it is better to double-check.
				Err(DispatchError::Other("Unexpected error in fee payment."))
			}
		} else {
			Ok(fee_credit)
		}
	}

	fn deposit_fee_into_treasury(
		depositor: &Self::AccountId,
		key: &Self::TreasuryKey,
		fee: Self::FeeBalance,
	) -> Result<(), DispatchError> {
		Treasury::deposit_into(depositor, key, fee)
	}
}
