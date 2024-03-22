use frame_support::PalletId;
use parity_scale_codec::{Decode, Encode, Error, Input};
use sp_runtime::TypeId;

#[derive(Clone, PartialEq, Eq)]
pub struct TournamentTreasuryAccount<SeasonId> {
	pub pallet_id: PalletId,
	pub season_id: SeasonId,
}

type TreasuryAccountEncodec<'a, SeasonId> = (&'a PalletId, &'a [u8; 1], &'a SeasonId);

type TreasuryAccountDecodec<SeasonId> = (PalletId, [u8; 1], SeasonId);

impl<SeasonId: Encode> Encode for TournamentTreasuryAccount<SeasonId> {
	fn encode(&self) -> Vec<u8> {
		// This codec will fit into the indexers rendering design such that we can
		// see the treasury accounts as "<pallet_id>/season_id".
		let data: TreasuryAccountEncodec<SeasonId> = (&self.pallet_id, b"/", &self.season_id);
		data.encode()
	}
}

impl<SeasonId: Decode> Decode for TournamentTreasuryAccount<SeasonId> {
	fn decode<I: Input>(input: &mut I) -> Result<Self, Error> {
		let tuple = TreasuryAccountDecodec::decode(input)?;
		Ok(Self::new(tuple.0, tuple.2))
	}
}

impl<SeasonId> TournamentTreasuryAccount<SeasonId> {
	pub fn new(pallet_id: PalletId, season_id: SeasonId) -> Self {
		Self { pallet_id, season_id }
	}
}

impl<SeasonId> TypeId for TournamentTreasuryAccount<SeasonId> {
	// I don't know yet the full implications of the TypeId.
	//
	// However, this is the same type that is used for the pallet id.
	// I believe this is used by indexers to identify accounts from pallet
	// instances, hence we should use the same identifier as the PalletId.
	const TYPE_ID: [u8; 4] = *b"modl";
}
