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
use colored::Colorize;

use citadel::client::{Client, Config};
use citadel::Error;
use microservices::shell::{Exec, LogLevel};
use mycitadel::cli::Opts;

fn main() {
    let opts = Opts::parse();
    LogLevel::from_verbosity_flag_count(opts.shared.verbose).apply();
    trace!("Command-line arguments: {:#?}", &opts);

    let config: Config = opts.clone().into();
    trace!("Tool configuration: {:#?}", &config);

    let mut client = Client::with(config).expect("Error initializing client");

    trace!("Executing command: {}", opts.command);
    opts.command
        .exec(&mut client)
        .unwrap_or_else(|err| match err {
            Error::ServerFailure(_) => {}
            err => {
                eprintln!(
                    "{} {}\n",
                    "Error:".bright_red(),
                    err.to_string().replace(": ", "\n  > ").red()
                )
            }
        });
}
