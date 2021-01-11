// MyCitadel: node, wallet library & command-line tool
// Written in 2020 by
//     Dr. Maxim Orlovsky <orlovsky@mycitadel.io>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the AGPL License
// along with this software.
// If not, see <https://www.gnu.org/licenses/agpl-3.0-standalone.html>.

#![recursion_limit = "256"]
// Coding conventions
#![deny(
    non_upper_case_globals,
    non_camel_case_types,
    non_snake_case,
    unused_mut,
    // unused_imports,
    // dead_code
    // missing_docs,
)]

extern crate amplify;
#[macro_use]
extern crate amplify_derive;
#[macro_use]
extern crate lnpbp_derive;
#[macro_use]
extern crate lnpbp;

#[cfg(feature = "serde")]
extern crate serde_crate as serde;
#[cfg(feature = "serde")]
#[macro_use]
extern crate serde_with;

#[cfg(feature = "shell")]
extern crate clap;
#[cfg(feature = "shell")]
#[macro_use]
extern crate log;

#[cfg(feature = "cli")]
pub mod cli;
#[cfg(any(feature = "node", feature = "client"))]
mod config;
mod error;
#[cfg(feature = "shell")]
pub mod opts;
#[cfg(any(feature = "node", feature = "client"))]
pub mod rpc;

#[cfg(feature = "node")]
pub mod daemon;
pub mod data;
#[cfg(feature = "node")]
pub mod storage;

#[cfg(any(feature = "node", feature = "client"))]
pub use config::Config;
pub use error::Error;
#[cfg(feature = "node")]
pub use storage::Storage;
