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

use clap::{AppSettings, Clap, ValueHint};

pub const MYCITADEL_CLI_CONFIG: &'static str = "{data_dir}/mycitadel-cli.toml";

#[derive(Clap, Clone, Debug)]
#[clap(
    name = "mycitadel-cli",
    bin_name = "mycitadel-cli",
    author,
    version,
    setting = AppSettings::ColoredHelp
)]
pub struct Opts {
    /// These params can be read also from the configuration file, not just
    /// command-line args or environment variables
    #[clap(flatten)]
    pub shared: crate::opts::Opts,

    /// Path to the configuration file.
    ///
    /// NB: Command-line options override configuration file values.
    #[clap(
        short,
        long,
        default_value = MYCITADEL_CLI_CONFIG,
        env = "MYCITADEL_CLI_CONFIG",
        value_hint = ValueHint::FilePath
    )]
    pub config: String,

    /// Command to execute
    #[clap(subcommand)]
    pub command: Command,
}

impl Opts {
    pub fn process(&mut self) {
        self.shared.process();
        self.shared.process_dir(&mut self.config);
    }
}

#[derive(Clap, Clone, Debug)]
pub enum Command {}
