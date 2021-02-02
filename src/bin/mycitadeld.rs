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

//! Main executable for MyCitadel runtime: main node managing microservice

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

use microservices::shell::LogLevel;
use mycitadel::daemon::{self, Config, Opts};

fn main() {
    println!("mycitadeld: MyCitadel node daemon");

    let opts = Opts::parse();
    LogLevel::from_verbosity_flag_count(opts.shared.verbose).apply();

    trace!("Command-line arguments: {:#?}", &opts);

    let mut config: Config = opts.clone().into();

    trace!("Daemon configuration: {:#?}", &config);
    config.process();
    trace!("Processed configuration: {:#?}", &opts);

    /*
    use self::internal::ResultExt;
    let (config_from_file, _) =
        internal::Config::custom_args_and_optional_files(std::iter::empty::<
            &str,
        >())
        .unwrap_or_exit();
     */

    debug!("Starting runtime ...");
    daemon::run(config).expect("Error running mycitadeld runtime");

    unreachable!()
}
