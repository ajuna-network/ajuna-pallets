use ajuna_primitives::treasury_manager::TreasuryManager;
use core::marker::PhantomData;
use frame_support::{
	pallet_prelude::{DispatchError, InvalidTransaction},
	sp_runtime::{traits::CheckedSub, ArithmeticError},
	traits::{
		fungibles, fungibles::Mutate, tokens::Preservation, Currency,
		ExistenceRequirement::KeepAlive, Get,
	},
	Parameter,
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
		base_fee: Self::FeeBalance,
		account: &Self::AccountId,
		identifier: &Self::AffiliateFeeIdentifier,
	) -> Result<Self::FeeBalance, DispatchError>;

	fn try_propagate_tournament_fee(
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

impl<T, PaymentAsset, Affiliate, Tournament, Treasury> FeeHandler
	for GameFeeHandler<T, PaymentAsset, Affiliate, Tournament, Treasury>
where
	PaymentAsset: Get<T::AssetKind>,
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

	fn try_propagate_chain_fee(
		base_fee: Self::FeeBalance,
		account: &Self::AccountId,
		identifier: &Self::AffiliateFeeIdentifier,
	) -> Result<Self::FeeBalance, DispatchError> {
		let mut final_fee = base_fee;

		for (transfer_fee, chain_account) in Affiliate::get_fee_from(base_fee, account, identifier)
		{
			if transfer_fee > 0_u32.into() {
				T::Assets::transfer(
					PaymentAsset::get(),
					account,
					&chain_account,
					transfer_fee,
					Preservation::Preserve,
				)?;
				final_fee = final_fee
					.checked_sub(&transfer_fee)
					.ok_or(DispatchError::Arithmetic(ArithmeticError::Underflow))?;
			}
		}

		Ok(final_fee)
	}

	fn try_propagate_tournament_fee(
		base_fee: Self::FeeBalance,
		account: &Self::AccountId,
		identifier: &Self::TournamentFeeIdentifier,
	) -> Result<Self::FeeBalance, DispatchError> {
		let (tournament_fee, tournament_account) =
			Tournament::get_fee_from(base_fee, account, identifier);

		if tournament_fee > 0_u32.into() {
			T::Assets::transfer(
				PaymentAsset::get(),
				account,
				&tournament_account,
				tournament_fee,
				Preservation::Preserve,
			)?;
			base_fee
				.checked_sub(&tournament_fee)
				.ok_or(DispatchError::Arithmetic(ArithmeticError::Underflow))
		} else {
			Ok(base_fee)
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
