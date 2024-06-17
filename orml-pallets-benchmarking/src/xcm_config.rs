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

use super::mock::{
	AllPalletsWithSystem, Balances, MockAccountId, MockBalance, ParachainInfo, Runtime,
	RuntimeCall, RuntimeEvent, RuntimeOrigin, XcmPallet,
};

use frame_support::{
	parameter_types,
	traits::{ConstU32, Contains, Everything, EverythingBut, Nothing},
	weights::Weight,
};
use frame_system::EnsureRoot;
use orml_traits::{
	location::{RelativeReserveProvider, Reserve},
	parameter_type_with_key,
};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use polkadot_parachain_primitives::primitives::Id as ParaId;
use scale_info::TypeInfo;
use sp_runtime::{
	traits::{Convert, Get},
	RuntimeDebug,
};
use staging_xcm::prelude::*;
use staging_xcm_builder::{
	AccountId32Aliases, AllowKnownQueryResponses, AllowSubscriptionsFrom,
	AllowTopLevelPaidExecutionFrom, Case, ChildParachainConvertsVia,
	ChildSystemParachainAsSuperuser, DescribeAllTerminal, EnsureDecodableXcm, FixedRateOfFungible,
	FixedWeightBounds, FrameTransactionalProcessor, FungibleAdapter, HashedDescription, IsConcrete,
	SignedAccountId32AsNative, SignedToAccountId32, SovereignSignedViaLocation, TakeWeightCredit,
	XcmFeeManagerFromComponents, XcmFeeToAccount,
};
use staging_xcm_executor::XcmExecutor;
use std::{cell::RefCell, marker::PhantomData};

parameter_types! {
	pub Para3000: u32 = 3000;
	pub Para3000Location: Location = Parachain(Para3000::get()).into();
	pub Para3000PaymentAmount: u128 = 1;
	pub Para3000PaymentAssets: Assets = Assets::from(Asset::from((Here, Para3000PaymentAmount::get())));
}

// This child parachain is a system parachain trusted to teleport native token.
pub const SOME_SYSTEM_PARA: u32 = 1001;

// This child parachain acts as trusted reserve for its assets in tests.
// BAJUN allowed to teleport to/from here.
pub const FOREIGN_ASSET_RESERVE_PARA_ID: u32 = 2001;
// Inner junction of reserve asset on `FOREIGN_ASSET_RESERVE_PARA_ID`.
pub const FOREIGN_ASSET_INNER_JUNCTION: Junction = GeneralIndex(1234567);

// This child parachain acts as trusted reserve for say.. AJUN that can be used for fees.
pub const AJUN_RESERVE_PARA_ID: u32 = 2002;
// Inner junction of reserve asset on `AJUN_RESERVE_PARA_ID`.
pub const AJUN_INNER_JUNCTION: Junction = PalletInstance(42);

// This child parachain is a trusted teleporter for BAJUN
// We'll use BAJUN in tests that teleport fees.
pub const BAJUN_PARA_ID: u32 = 2003;

// This child parachain is used for filtered/disallowed assets.
pub const FILTERED_PARA_ID: u32 = 2010;

parameter_types! {
	pub const RelayLocation: Location = Here.into_location();
	pub const NativeAsset: Asset = Asset {
		fun: Fungible(10),
		id: AssetId(Here.into_location()),
	};
	pub SystemParachainLocation: Location = Location::new(
		0,
		[Parachain(SOME_SYSTEM_PARA)]
	);
	pub ForeignReserveLocation: Location = Location::new(
		0,
		[Parachain(FOREIGN_ASSET_RESERVE_PARA_ID)]
	);
	pub PaidParaForeignReserveLocation: Location = Location::new(
		0,
		[Parachain(Para3000::get())]
	);
	pub ForeignAsset: Asset = Asset {
		fun: Fungible(10),
		id: AssetId(Location::new(
			0,
			[Parachain(FOREIGN_ASSET_RESERVE_PARA_ID), FOREIGN_ASSET_INNER_JUNCTION],
		)),
	};
	pub PaidParaForeignAsset: Asset = Asset {
		fun: Fungible(10),
		id: AssetId(Location::new(
			0,
			[Parachain(Para3000::get())],
		)),
	};
	pub AjunReserveLocation: Location = Location::new(
		0,
		[Parachain(AJUN_RESERVE_PARA_ID)]
	);
	pub Ajun: Asset = Asset {
		fun: Fungible(10),
		id: AssetId(Location::new(
			0,
			[Parachain(AJUN_RESERVE_PARA_ID), AJUN_INNER_JUNCTION],
		)),
	};
	pub BajunTeleportLocation: Location = Location::new(
		0,
		[Parachain(BAJUN_PARA_ID)]
	);
	pub Bajun: Asset = Asset {
		fun: Fungible(10),
		id: AssetId(Location::new(
			0,
			[Parachain(BAJUN_PARA_ID)],
		)),
	};
	pub FilteredTeleportLocation: Location = Location::new(
		0,
		[Parachain(FILTERED_PARA_ID)]
	);
	pub FilteredTeleportAsset: Asset = Asset {
		fun: Fungible(10),
		id: AssetId(Location::new(
			0,
			[Parachain(FILTERED_PARA_ID)],
		)),
	};
	pub const AnyNetwork: Option<NetworkId> = None;
	pub UniversalLocation: InteriorLocation = GlobalConsensus(ByGenesis([0; 32])).into();
	pub UnitWeightCost: u64 = 1_000;
	pub CheckingAccount: MockAccountId = XcmPallet::check_account();
}

pub type SovereignAccountOf = (
	ChildParachainConvertsVia<ParaId, MockAccountId>,
	AccountId32Aliases<AnyNetwork, MockAccountId>,
	HashedDescription<MockAccountId, DescribeAllTerminal>,
);

pub type AssetTransactors =
	(FungibleAdapter<Balances, IsConcrete<RelayLocation>, SovereignAccountOf, MockAccountId, ()>,);

type LocalOriginConverter = (
	SovereignSignedViaLocation<SovereignAccountOf, RuntimeOrigin>,
	SignedAccountId32AsNative<AnyNetwork, RuntimeOrigin>,
	ChildSystemParachainAsSuperuser<ParaId, RuntimeOrigin>,
);

parameter_types! {
	pub const BaseXcmWeight: Weight = Weight::from_parts(1_000, 1_000);
	pub const MaxAssetsForTransfer: usize = 2;
	pub CurrencyPerSecondPerByte: (AssetId, u128, u128) = (AssetId(RelayLocation::get()), 1, 1);
	pub TrustedLocal: (AssetFilter, Location) = (All.into(), Here.into());
	pub TrustedSystemPara: (AssetFilter, Location) = (NativeAsset::get().into(), SystemParachainLocation::get());
	pub TrustedBajun: (AssetFilter, Location) = (Bajun::get().into(), BajunTeleportLocation::get());
	pub TrustedFilteredTeleport: (AssetFilter, Location) = (FilteredTeleportAsset::get().into(), FilteredTeleportLocation::get());
	pub TeleportBajunToForeign: (AssetFilter, Location) = (Bajun::get().into(), ForeignReserveLocation::get());
	pub TrustedForeign: (AssetFilter, Location) = (ForeignAsset::get().into(), ForeignReserveLocation::get());
	pub TrustedPaidParaForeign: (AssetFilter, Location) = (PaidParaForeignAsset::get().into(), PaidParaForeignReserveLocation::get());

	pub TrustedAjun: (AssetFilter, Location) = (Ajun::get().into(), AjunReserveLocation::get());
	pub const MaxInstructions: u32 = 100;
	pub const MaxAssetsIntoHolding: u32 = 64;
	pub XcmFeesTargetAccount: MockAccountId = MockAccountId::new([167u8; 32]);
}

pub const XCM_FEES_NOT_WAIVED_USER_ACCOUNT: [u8; 32] = [37u8; 32];

pub struct XcmFeesNotWaivedLocations;
impl Contains<Location> for XcmFeesNotWaivedLocations {
	fn contains(location: &Location) -> bool {
		matches!(
			location.unpack(),
			(0, [Junction::AccountId32 { network: None, id: XCM_FEES_NOT_WAIVED_USER_ACCOUNT }])
		)
	}
}

pub type Barrier = (
	TakeWeightCredit,
	AllowTopLevelPaidExecutionFrom<Everything>,
	AllowKnownQueryResponses<XcmPallet>,
	AllowSubscriptionsFrom<Everything>,
);

thread_local! {
	pub static SENT_XCM: RefCell<Vec<(Location, Xcm<()>)>> = RefCell::new(Vec::new());
	pub static FAIL_SEND_XCM: RefCell<bool> = RefCell::new(false);
}
pub(crate) fn fake_message_hash<T>(message: &Xcm<T>) -> XcmHash {
	message.using_encoded(sp_io::hashing::blake2_256)
}
pub struct TestSendXcm;
impl SendXcm for TestSendXcm {
	type Ticket = (Location, Xcm<()>);
	fn validate(
		dest: &mut Option<Location>,
		msg: &mut Option<Xcm<()>>,
	) -> SendResult<(Location, Xcm<()>)> {
		if FAIL_SEND_XCM.with(|q| *q.borrow()) {
			return Err(SendError::Transport("Intentional send failure used in tests"));
		}
		let pair = (dest.take().unwrap(), msg.take().unwrap());
		Ok((pair, Assets::new()))
	}
	fn deliver(pair: (Location, Xcm<()>)) -> Result<XcmHash, SendError> {
		let hash = fake_message_hash(&pair.1);
		SENT_XCM.with(|q| q.borrow_mut().push(pair));
		Ok(hash)
	}
}
/// Sender that returns error if `X8` junction and stops routing
pub struct TestSendXcmErrX8;
impl SendXcm for TestSendXcmErrX8 {
	type Ticket = (Location, Xcm<()>);
	fn validate(
		dest: &mut Option<Location>,
		_: &mut Option<Xcm<()>>,
	) -> SendResult<(Location, Xcm<()>)> {
		if dest.as_ref().unwrap().len() == 8 {
			dest.take();
			Err(SendError::Transport("Destination location full"))
		} else {
			Err(SendError::NotApplicable)
		}
	}
	fn deliver(pair: (Location, Xcm<()>)) -> Result<XcmHash, SendError> {
		let hash = fake_message_hash(&pair.1);
		SENT_XCM.with(|q| q.borrow_mut().push(pair));
		Ok(hash)
	}
}

pub type XcmRouter = EnsureDecodableXcm<(TestSendXcmErrX8, TestSendXcm)>;

pub struct XcmConfig;
impl staging_xcm_executor::Config for XcmConfig {
	type RuntimeCall = RuntimeCall;
	type XcmSender = XcmRouter;
	type AssetTransactor = AssetTransactors;
	type OriginConverter = LocalOriginConverter;
	type IsReserve = (Case<TrustedForeign>, Case<TrustedAjun>, Case<TrustedPaidParaForeign>);
	type IsTeleporter = (
		Case<TrustedLocal>,
		Case<TrustedSystemPara>,
		Case<TrustedBajun>,
		Case<TeleportBajunToForeign>,
		Case<TrustedFilteredTeleport>,
	);
	type Aliasers = Nothing;
	type UniversalLocation = UniversalLocation;
	type Barrier = Barrier;
	type Weigher = FixedWeightBounds<BaseXcmWeight, RuntimeCall, MaxInstructions>;
	type Trader = FixedRateOfFungible<CurrencyPerSecondPerByte, ()>;
	type ResponseHandler = XcmPallet;
	type AssetTrap = XcmPallet;
	type AssetLocker = ();
	type AssetExchanger = ();
	type AssetClaims = XcmPallet;
	type SubscriptionService = XcmPallet;
	type PalletInstancesInfo = AllPalletsWithSystem;
	type MaxAssetsIntoHolding = MaxAssetsIntoHolding;
	type FeeManager = XcmFeeManagerFromComponents<
		EverythingBut<XcmFeesNotWaivedLocations>,
		XcmFeeToAccount<Self::AssetTransactor, MockAccountId, XcmFeesTargetAccount>,
	>;
	type MessageExporter = ();
	type UniversalAliases = Nothing;
	type CallDispatcher = RuntimeCall;
	type SafeCallFilter = Everything;
	type TransactionalProcessor = FrameTransactionalProcessor;
	type HrmpNewChannelOpenRequestHandler = ();
	type HrmpChannelAcceptedHandler = ();
	type HrmpChannelClosingHandler = ();
	type XcmRecorder = XcmPallet;
}

pub type LocalOriginToLocation = SignedToAccountId32<RuntimeOrigin, MockAccountId, AnyNetwork>;

parameter_types! {
	pub static AdvertisedXcmVersion: XcmVersion = 4;
}

pub struct XcmTeleportFiltered;
impl Contains<(Location, Vec<Asset>)> for XcmTeleportFiltered {
	fn contains(t: &(Location, Vec<Asset>)) -> bool {
		let filtered = FilteredTeleportAsset::get();
		t.1.iter().any(|asset| asset == &filtered)
	}
}

/// fallback implementation
pub struct TestWeightInfo;
impl pallet_xcm::WeightInfo for TestWeightInfo {
	fn send() -> Weight {
		Weight::from_parts(100_000_000, 0)
	}

	fn teleport_assets() -> Weight {
		Weight::from_parts(100_000_000, 0)
	}

	fn reserve_transfer_assets() -> Weight {
		Weight::from_parts(100_000_000, 0)
	}

	fn transfer_assets() -> Weight {
		Weight::from_parts(100_000_000, 0)
	}

	fn execute() -> Weight {
		Weight::from_parts(100_000_000, 0)
	}

	fn force_xcm_version() -> Weight {
		Weight::from_parts(100_000_000, 0)
	}

	fn force_default_xcm_version() -> Weight {
		Weight::from_parts(100_000_000, 0)
	}

	fn force_subscribe_version_notify() -> Weight {
		Weight::from_parts(100_000_000, 0)
	}

	fn force_unsubscribe_version_notify() -> Weight {
		Weight::from_parts(100_000_000, 0)
	}

	fn force_suspension() -> Weight {
		Weight::from_parts(100_000_000, 0)
	}

	fn migrate_supported_version() -> Weight {
		Weight::from_parts(100_000_000, 0)
	}

	fn migrate_version_notifiers() -> Weight {
		Weight::from_parts(100_000_000, 0)
	}

	fn already_notified_target() -> Weight {
		Weight::from_parts(100_000_000, 0)
	}

	fn notify_current_targets() -> Weight {
		Weight::from_parts(100_000_000, 0)
	}

	fn notify_target_migration_fail() -> Weight {
		Weight::from_parts(100_000_000, 0)
	}

	fn migrate_version_notify_targets() -> Weight {
		Weight::from_parts(100_000_000, 0)
	}

	fn migrate_and_notify_old_targets() -> Weight {
		Weight::from_parts(100_000_000, 0)
	}

	fn new_query() -> Weight {
		Weight::from_parts(100_000_000, 0)
	}

	fn take_response() -> Weight {
		Weight::from_parts(100_000_000, 0)
	}

	fn claim_assets() -> Weight {
		Weight::from_parts(100_000_000, 0)
	}
}

impl pallet_xcm::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type CurrencyMatcher = IsConcrete<RelayLocation>;
	type SendXcmOrigin = staging_xcm_builder::EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
	type XcmRouter = XcmRouter;
	type ExecuteXcmOrigin =
		staging_xcm_builder::EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
	type XcmExecuteFilter = Everything;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type XcmTeleportFilter = EverythingBut<XcmTeleportFiltered>;
	type XcmReserveTransferFilter = Everything;
	type Weigher = FixedWeightBounds<BaseXcmWeight, RuntimeCall, MaxInstructions>;
	type UniversalLocation = UniversalLocation;
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;
	type AdvertisedXcmVersion = AdvertisedXcmVersion;
	type AdminOrigin = EnsureRoot<MockAccountId>;
	type TrustedLockers = ();
	type SovereignAccountOf = AccountId32Aliases<(), MockAccountId>;
	type MaxLockers = ConstU32<8>;
	type MaxRemoteLockConsumers = ConstU32<0>;
	type RemoteLockConsumerIdentifier = ();
	type WeightInfo = TestWeightInfo;
}

impl crate::benchmarks::xcm::Config for Runtime {}

impl orml_xcm::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type SovereignOrigin = EnsureRoot<MockAccountId>;
}

parameter_types! {
	pub SelfReserveAlias: Location = Location::new(
		0,
		[AJUN_GENERAL_KEY]
	);
	// This is how we are going to detect whether the asset is a Reserve asset
	pub SelfLocation: Location = Location::here();
	// We need this to be able to catch when someone is trying to execute a non-
	// cross-chain transfer in xtokens through the absolute path way
	pub SelfLocationAbsolute: Location = Location::new(
		1,
		Parachain(ParachainInfo::parachain_id().into())
	);

}

parameter_type_with_key! {
	pub ParachainMinFee: |_location: Location| -> Option<u128> {
		None
	};
}

#[derive(
	Encode,
	Decode,
	Eq,
	PartialEq,
	Copy,
	Clone,
	RuntimeDebug,
	PartialOrd,
	Ord,
	TypeInfo,
	MaxEncodedLen,
)]
pub enum CurrencyId {
	Ajun,
}

const fn ajun_general_key() -> Junction {
	const AJUN_KEY: [u8; 32] = *b"AJUN\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
	GeneralKey { length: 4, data: AJUN_KEY }
}
const AJUN_GENERAL_KEY: Junction = ajun_general_key();

/// Converts a CurrencyId into a Location, used by xtoken for XCMP.
pub struct CurrencyIdConvert;
impl Convert<CurrencyId, Option<Location>> for CurrencyIdConvert {
	fn convert(id: CurrencyId) -> Option<Location> {
		match id {
			CurrencyId::Ajun => Some(Location::new(
				1,
				[Parachain(ParachainInfo::parachain_id().into()), AJUN_GENERAL_KEY],
			)),
		}
	}
}

pub struct AccountIdToLocation;
impl Convert<MockAccountId, Location> for AccountIdToLocation {
	fn convert(account: MockAccountId) -> Location {
		[AccountId32 { network: None, id: account.into() }].into()
	}
}

/// This struct offers uses RelativeReserveProvider to output relative views of Locations
/// However, additionally accepts a Location that aims at representing the chain part
/// (parent: 1, Parachain(paraId)) of the absolute representation of our chain.
/// If a token reserve matches against this absolute view, we return  Some(Location::here())
/// This helps users by preventing errors when they try to transfer a token through xtokens
/// to our chain (either inserting the relative or the absolute value).
pub struct AbsoluteAndRelativeReserve<AbsoluteLocation>(PhantomData<AbsoluteLocation>);
impl<AbsoluteLocation> Reserve for AbsoluteAndRelativeReserve<AbsoluteLocation>
where
	AbsoluteLocation: Get<Location>,
{
	fn reserve(asset: &Asset) -> Option<Location> {
		RelativeReserveProvider::reserve(asset).map(|relative_reserve| {
			if relative_reserve == AbsoluteLocation::get() {
				Location::here()
			} else {
				relative_reserve
			}
		})
	}
}

impl crate::benchmarks::xtokens::Config for Runtime {}

impl orml_xtokens::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Balance = MockBalance;
	type CurrencyId = CurrencyId;
	type CurrencyIdConvert = CurrencyIdConvert;
	type AccountIdToLocation = AccountIdToLocation;
	type SelfLocation = SelfLocation;
	type MinXcmFee = ParachainMinFee;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type LocationsFilter = Everything;
	type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;
	type BaseXcmWeight = BaseXcmWeight;
	type UniversalLocation = UniversalLocation;
	type MaxAssetsForTransfer = MaxAssetsForTransfer;
	type ReserveProvider = AbsoluteAndRelativeReserve<SelfLocationAbsolute>;
	type RateLimiter = ();
	type RateLimiterId = ();
}
