use frame_support::PalletId;
use parity_scale_codec::{Decode, Encode, Error, Input};
use sp_runtime::TypeId;
use sp_std::vec::Vec;

#[derive(Clone, PartialEq, Eq)]
pub struct TournamentTreasuryAccount<CategoryId> {
	pub pallet_id: PalletId,
	pub category_id: CategoryId,
}

type TreasuryAccountEncodec<'a, CategoryId> = (&'a PalletId, &'a [u8; 1], &'a CategoryId);

type TreasuryAccountDecodec<CategoryId> = (PalletId, [u8; 1], CategoryId);

impl<CategoryId: Encode> Encode for TournamentTreasuryAccount<CategoryId> {
	fn encode(&self) -> Vec<u8> {
		// This codec will fit into the indexers rendering design such that we can
		// see the treasury accounts as "<pallet_id>/category_id".
		let data: TreasuryAccountEncodec<CategoryId> = (&self.pallet_id, b"/", &self.category_id);
		data.encode()
	}
}

impl<CategoryId: Decode> Decode for TournamentTreasuryAccount<CategoryId> {
	fn decode<I: Input>(input: &mut I) -> Result<Self, Error> {
		let tuple = TreasuryAccountDecodec::decode(input)?;
		Ok(Self::new(tuple.0, tuple.2))
	}
}

impl<CategoryId> TournamentTreasuryAccount<CategoryId> {
	pub fn new(pallet_id: PalletId, category_id: CategoryId) -> Self {
		Self { pallet_id, category_id }
	}
}

impl<CategoryId> TypeId for TournamentTreasuryAccount<CategoryId> {
	// I don't know yet the full implications of the TypeId.
	//
	// However, this is the same type that is used for the pallet id.
	// I believe this is used by indexers to identify accounts from pallet
	// instances, hence we should use the same identifier as the PalletId.
	const TYPE_ID: [u8; 4] = *b"modl";
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn tournament_treasury_account_codec_works() {
		let pallet_id = PalletId(*b"ajn/trsy");
		let tournament_account = TournamentTreasuryAccount::new(pallet_id, 2u32);

		let encoded = tournament_account.encode();
		let decoded = TournamentTreasuryAccount::<u32>::decode(&mut encoded.as_slice()).unwrap();

		// PalletId does not implement debug...
		assert_eq!(encoded, decoded.encode())
	}
}
