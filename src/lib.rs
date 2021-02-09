// MyCitadel: node, wallet library & command-line tool
// Written in 2021 by
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
extern crate amplify;
#[macro_use]
extern crate amplify_derive;
#[macro_use]
extern crate lnpbp;
#[cfg_attr(feature = "_rpc", macro_use)]
extern crate internet2;

#[macro_use]
extern crate log;

#[macro_use]
extern crate serde_with;

#[cfg(feature = "cli")]
pub mod cli;
mod error;
pub mod model;
#[cfg(feature = "shell")]
pub mod opts;
#[cfg(feature = "_rpc")]
pub mod rpc;

#[cfg(feature = "client")]
pub mod client;
#[cfg(all(feature = "cli", feature = "node"))]
mod embedded;
#[cfg(feature = "node")]
pub mod server;

#[cfg(feature = "node")]
pub mod cache;
#[cfg(feature = "node")]
pub mod chainapi;
#[cfg(feature = "node")]
pub mod chainwatch;
#[cfg(feature = "node")]
pub mod storage;

#[cfg(feature = "client")]
pub use client::Client;
#[cfg(all(feature = "cli", feature = "node"))]
pub use embedded::{run_embedded, Opts as EmbeddedOpts};
pub use error::Error;
