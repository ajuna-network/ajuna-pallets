use super::*;

pub(crate) const MIN_SACRIFICE: usize = 1;
pub(crate) const MAX_SACRIFICE: usize = 4;

pub(crate) const STARDUST_PER_TRAVEL_POINT: u16 = 3;

pub(crate) const MOON_MINTED_TRAVEL_POINTS_DEC: u8 = 1;
pub(crate) const CAPTAIN_MINTED_TRAVEL_POINTS_INC: u32 = 1;

// Calculations based on a 6s block time
pub(crate) const MINT_TRAVEL_POINT_COOLDOWN: u32 = 10 * 60 * 24;
pub(crate) const MINT_TRAVEL_POINT_INIT: u8 = 10;
// Calculations based on a 6s block time
pub(crate) const MINT_TRAVEL_POINT_BLOCK_MINTS_PERIOD: u32 = 10 * 60 * 24 * 7 * 40;
