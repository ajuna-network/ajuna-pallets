#![cfg_attr(not(feature = "std"), no_std)]

pub mod error;
pub mod rules;
pub mod traits;

pub use error::Error;
pub use traits::{AsErrorCode, AssetT, SageApi, SageGameTransition};
