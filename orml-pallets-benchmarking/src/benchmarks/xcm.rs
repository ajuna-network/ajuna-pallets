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

use crate::{utils::get_account, xcm_config::Para3000};

use frame_benchmarking::benchmarks;
use frame_system::RawOrigin;
use sp_std::prelude::*;
use staging_xcm::prelude::*;

use orml_xcm::Call;

pub struct Pallet<T: Config>(orml_xcm::Pallet<T>);

pub trait Config: orml_xcm::Config {}

benchmarks! {
	send_as_sovereign {
		let from = get_account::<T>();
		let dest: VersionedLocation = Parachain(Para3000::get()).into();
		let message = VersionedXcm::V4(Xcm::new());
	}: _(RawOrigin::Root, Box::new(dest.clone()), Box::new(message.clone()))
	verify {
		crate::mock::System::assert_last_event(crate::mock::RuntimeEvent::OrmlXcm(
			orml_xcm::Event::Sent { to: dest.try_into().unwrap(), message: message.try_into().unwrap() }
		));
	}

	impl_benchmark_test_suite!(
		Pallet,
		crate::mock::new_test_ext(),
		crate::mock::Runtime
	);
}
