// Ajuna Node
// Copyright (C) 2022 BlogaTech AG

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use crate::types::{Avatar, ByteConvertible, Force, RarityTier};
use frame_support::traits::Get;
use pallet_ajuna_nft_transfer::traits::{NFTAttribute, NftConvertible};
use parity_scale_codec::alloc::string::ToString;
use scale_info::prelude::format;
use sp_runtime::traits::UniqueSaturatedInto;
use sp_std::prelude::*;

const DNA_ATTRIBUTE: [u8; 3] = *b"DNA";
const SOUL_POINTS_ATTRIBUTE: [u8; 11] = *b"SOUL_POINTS";
const RARITY_ATTRIBUTE: [u8; 6] = *b"RARITY";
const FORCE_ATTRIBUTE: [u8; 5] = *b"FORCE";
const SEASON_ID_ATTRIBUTE: [u8; 9] = *b"SEASON_ID";
const MINTED_AT_ATTRIBUTE: [u8; 9] = *b"MINTED_AT";

impl<KL, VL, BlockNumber> NftConvertible<KL, VL> for Avatar<BlockNumber>
where
	KL: Get<u32>,
	VL: Get<u32>,
	BlockNumber: sp_runtime::traits::BlockNumber,
{
	const ITEM_CODE: &'static [u8] = b"AVATAR";
	const IPFS_URL_CODE: &'static [u8] = b"IPFS_URL";

	fn get_attribute_codes() -> Vec<NFTAttribute<KL>> {
		vec![
			DNA_ATTRIBUTE.to_vec().try_into().unwrap(),
			SOUL_POINTS_ATTRIBUTE.to_vec().try_into().unwrap(),
			RARITY_ATTRIBUTE.to_vec().try_into().unwrap(),
			FORCE_ATTRIBUTE.to_vec().try_into().unwrap(),
			SEASON_ID_ATTRIBUTE.to_vec().try_into().unwrap(),
			MINTED_AT_ATTRIBUTE.to_vec().try_into().unwrap(),
		]
	}

	fn get_encoded_attributes(&self) -> Vec<(NFTAttribute<KL>, NFTAttribute<VL>)> {
		vec![
			(
				DNA_ATTRIBUTE.to_vec().try_into().unwrap(),
				format!("0x{}", hex::encode(self.dna.as_slice()))
					.into_bytes()
					.try_into()
					.unwrap(),
			),
			(
				SOUL_POINTS_ATTRIBUTE.to_vec().try_into().unwrap(),
				format!("{}", self.souls).into_bytes().try_into().unwrap(),
			),
			(RARITY_ATTRIBUTE.to_vec().try_into().unwrap(), {
				let rarity_value = RarityTier::from_byte(if self.season_id == 1 {
					self.rarity() + 1
				} else {
					self.rarity()
				});
				rarity_value.to_string().to_uppercase().into_bytes().try_into().unwrap()
			}),
			(
				FORCE_ATTRIBUTE.to_vec().try_into().unwrap(),
				Force::from_byte(self.force())
					.to_string()
					.to_uppercase()
					.into_bytes()
					.try_into()
					.unwrap(),
			),
			(
				SEASON_ID_ATTRIBUTE.to_vec().try_into().unwrap(),
				format!("{}", self.season_id).into_bytes().try_into().unwrap(),
			),
			(
				MINTED_AT_ATTRIBUTE.to_vec().try_into().unwrap(),
				format!("{}", UniqueSaturatedInto::<u64>::unique_saturated_into(self.minted_at))
					.into_bytes()
					.try_into()
					.unwrap(),
			),
		]
	}
}

#[cfg(test)]
mod tests {
	use crate::types::{
		avatar::{
			nft::{
				DNA_ATTRIBUTE, FORCE_ATTRIBUTE, MINTED_AT_ATTRIBUTE, RARITY_ATTRIBUTE,
				SEASON_ID_ATTRIBUTE, SOUL_POINTS_ATTRIBUTE,
			},
			ByteConvertible,
		},
		Avatar, DnaEncoding, Force, RarityTier,
	};
	use frame_support::{
		__private::Get,
		pallet_prelude::{Decode, Encode, MaxEncodedLen, TypeInfo},
	};
	use pallet_ajuna_nft_transfer::traits::NftConvertible;
	use sp_core::bounded_vec;

	type TestAvatar = Avatar<u64>;

	#[derive(Debug, PartialEq, Eq, Clone, Encode, Decode, MaxEncodedLen, TypeInfo)]
	pub struct ParameterGet<const N: u32>;

	impl<const N: u32> Get<u32> for ParameterGet<N> {
		fn get() -> u32 {
			N
		}
	}

	// Same values as we put in AAA tests
	pub type KeyLimit = ParameterGet<32>;
	pub type ValueLimit = ParameterGet<200>;

	/// Avatar taken from the `can_lock_avatar_successfully` test.
	pub fn test_avatar() -> TestAvatar {
		Avatar {
			season_id: 1,
			dna: bounded_vec![
				0x24, 0x00, 0x41, 0x0F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
				0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
				0x00, 0x00, 0x00, 0x00
			],
			souls: 60,
			encoding: DnaEncoding::V2,
			minted_at: 3,
		}
	}

	#[test]
	fn get_attributes_codes_works() {
		let attribute_code =
			<TestAvatar as NftConvertible<KeyLimit, ValueLimit>>::get_attribute_codes();

		assert_eq!(attribute_code[0], DNA_ATTRIBUTE.to_vec());
		assert_eq!(attribute_code[1], SOUL_POINTS_ATTRIBUTE.to_vec());
		assert_eq!(attribute_code[2], RARITY_ATTRIBUTE.to_vec());
		assert_eq!(attribute_code[3], FORCE_ATTRIBUTE.to_vec());
		assert_eq!(attribute_code[4], SEASON_ID_ATTRIBUTE.to_vec());
		assert_eq!(attribute_code[5], MINTED_AT_ATTRIBUTE.to_vec());
	}

	#[test]
	fn get_encoded_attributes_works() {
		let avatar = test_avatar();

		let attributes =
			<TestAvatar as NftConvertible<KeyLimit, ValueLimit>>::get_encoded_attributes(&avatar);

		assert_eq!(attributes[0].0, DNA_ATTRIBUTE.to_vec());
		assert_eq!(attributes[1].0, SOUL_POINTS_ATTRIBUTE.to_vec());
		assert_eq!(attributes[2].0, RARITY_ATTRIBUTE.to_vec());
		assert_eq!(attributes[3].0, FORCE_ATTRIBUTE.to_vec());
		assert_eq!(attributes[4].0, SEASON_ID_ATTRIBUTE.to_vec());
		assert_eq!(attributes[5].0, MINTED_AT_ATTRIBUTE.to_vec());

		assert_eq!(
			attributes[0].1,
			format!("0x{}", hex::encode(avatar.dna.as_slice())).into_bytes()
		);
		assert_eq!(attributes[1].1, format!("{}", avatar.souls).into_bytes());
		assert_eq!(
			attributes[2].1,
			RarityTier::from_byte(if avatar.season_id == 1 {
				avatar.rarity() + 1
			} else {
				avatar.rarity()
			})
			.to_string()
			.to_uppercase()
			.into_bytes()
		);
		assert_eq!(
			attributes[3].1,
			Force::from_byte(avatar.force()).to_string().to_uppercase().into_bytes(),
		);
		assert_eq!(attributes[4].1, format!("{}", avatar.season_id).into_bytes());
		assert_eq!(attributes[5].1, format!("{}", avatar.minted_at).into_bytes());
	}
}
