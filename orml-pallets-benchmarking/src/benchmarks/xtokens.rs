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

use crate::{
	mock::{MockBalance, Runtime},
	utils::get_account,
	xcm_config::{CurrencyId, Para3000},
};

use frame_benchmarking::benchmarks;
use frame_system::RawOrigin;
use sp_std::prelude::*;
use staging_xcm::prelude::*;

use orml_xtokens::Call;

pub struct Pallet<T: Config>(orml_xtokens::Pallet<T>);

pub trait Config: orml_xtokens::Config {}

benchmarks! {
	transfer {
		let currency_id = CurrencyId::Ajun;
		let amount: MockBalance = 1000.into();
		let dest: VersionedLocation = Parachain(Para3000::get()).into();
		let dest_weight_limit = WeightLimit::Unlimited;
	}: _(RawOrigin::Root, currency_id, amount, Box::new(dest.clone()), dest_weight_limit)
	verify {
		crate::mock::System::assert_last_event(crate::mock::RuntimeEvent::OrmlXtokens(
			orml_xtokens::Event::TransferredAssets {
				sender: RawOrigin::Root.into(),
				assets: (dest.clone(), amount).into(),
				fee: 0.into(),
				dest: dest.into(),
			}
		));
	}

	/*transfer_multiasset {
	}: _(RawOrigin::Signed(from), to_lookup, schedule.clone())
	verify {
	}

	transfer_with_fee {
	}: _(RawOrigin::Signed(from), to_lookup, schedule.clone())
	verify {
	}

	transfer_multiasset_with_fee {
	}: _(RawOrigin::Signed(from), to_lookup, schedule.clone())
	verify {
	}

	transfer_multicurrencies {
	}: _(RawOrigin::Signed(from), to_lookup, schedule.clone())
	verify {
	}

	transfer_multiassets {
	}: _(RawOrigin::Signed(from), to_lookup, schedule.clone())
	verify {
	}*/

	impl_benchmark_test_suite!(
		Pallet,
		crate::mock::new_test_ext(),
		crate::mock::Runtime
	);
}
