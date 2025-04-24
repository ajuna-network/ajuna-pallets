// Ajuna Node
// Copyright (C) 2022 BlogaTech AG
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

#![cfg_attr(not(feature = "std"), no_std)]

mod fee_handler;
mod withdraw_credit;

pub use fee_handler::*;
pub use transfer_fungible::*;
pub use voucher_handler::*;
pub use withdraw_credit::*;

mod distribute_fee;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;
mod transfer_fungible;
mod voucher_handler;
