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

//! Command-line interface to MyCitadel node

#![recursion_limit = "256"]
// Coding conventions
#![deny(
    non_upper_case_globals,
    non_camel_case_types,
    non_snake_case,
    unused_mut,
    unused_imports,
    dead_code,
    missing_docs
)]

#[macro_use]
extern crate log;

use clap::Clap;

use microservices::shell::{Exec, LogLevel};
use mycitadel::{daemon, EmbeddedOpts, Error};

fn main() -> Result<(), Error> {
    println!("mycitadel: command-line runtime");

    let opts = EmbeddedOpts::parse();
    LogLevel::from_verbosity_flag_count(opts.daemon.shared.verbose).apply();

    trace!("Command-line arguments: {:#?}", &opts);

    let config = daemon::Config::from(opts.daemon);

    let mut client =
        mycitadel::run_embedded(config).expect("Error initializing MyCitadel");
    opts.command.exec(&mut client)
}
