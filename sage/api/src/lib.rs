#![cfg_attr(not(feature = "std"), no_std)]

mod error;
mod traits;

pub use error::Error;
pub use traits::{AsErrorCode, AssetT, SageApi, SageGameTransition};
