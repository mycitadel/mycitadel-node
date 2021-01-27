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
    unused_imports,
    // dead_code
    // missing_docs,
)]

#[macro_use]
extern crate amplify_derive;
#[macro_use]
extern crate lnpbp;
#[cfg_attr(feature = "_rpc", macro_use)]
extern crate internet2;

#[macro_use]
extern crate log;

#[cfg(feature = "serde")]
extern crate serde_crate as serde;
#[cfg(feature = "serde")]
#[macro_use]
extern crate serde_with;

#[cfg(feature = "cli")]
pub mod cli;
#[cfg(feature = "_rpc")]
mod config;
mod error;
#[cfg(feature = "shell")]
pub mod opts;
#[cfg(feature = "_rpc")]
pub mod rpc;

#[cfg(feature = "node")]
pub mod daemon;
pub mod data;
#[cfg(feature = "node")]
pub mod storage;

#[cfg(feature = "_rpc")]
pub use config::Config;
pub use error::Error;
