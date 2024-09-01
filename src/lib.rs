///! Numerai API wrapper for Rust

#[cfg(all(feature = "async", feature = "blocking"))]
compile_error!("Cannot compile with both async and blocking features enabled. Please choose one or the other.");

pub mod api;
pub mod utils;

pub use api::numerapi::NumerAPI;
pub use api::signals::SignalsAPI;
pub use api::crypto::CryptoAPI;


/*
""" Numerai Python API"""

import pkg_resources

try:
    __version__ = pkg_resources.get_distribution(__name__).version
except pkg_resources.DistributionNotFound:
    __version__ = 'unknown'


# pylint: disable=wrong-import-position
from numerapi.numerapi import NumerAPI
from numerapi.signalsapi import SignalsAPI
from numerapi.cryptoapi import CryptoAPI
# pylint: enable=wrong-import-position
*/
