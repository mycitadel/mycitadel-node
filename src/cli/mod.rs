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

mod command;
mod opts;
mod output;
pub(self) mod util;

pub use opts::{
    AddressCommand, AssetCommand, Command, DescriptorOpts, Formatting,
    InvoiceCommand, Opts, PsbtFormat, WalletCommand, WalletCreateCommand,
    WalletOpts,
};
pub use output::OutputFormat;

// -----------------------------------------------------------------------------

use citadel::client::Config;
use std::convert::TryInto;

impl From<Opts> for Config {
    fn from(opts: crate::cli::Opts) -> Self {
        Config {
            rpc_endpoint: opts.shared.rpc_endpoint.try_into().expect(
                "The provided socket address must be a valid ZMQ socket",
            ),
            verbose: opts.shared.verbose,
        }
    }
}
