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

use clap::{AppSettings, Clap, ValueHint};
use internet2::ZmqSocketAddr;
use lnpbp::Chain;
use microservices::FileFormat;
use std::fs;
use std::path::PathBuf;

#[cfg(any(target_os = "linux"))]
pub const MYCITADEL_DATA_DIR: &'static str = "~/.mycitadel";
#[cfg(any(target_os = "freebsd", target_os = "openbsd", target_os = "netbsd"))]
pub const MYCITADEL_DATA_DIR: &'static str = "~/.mycitadel";
#[cfg(target_os = "macos")]
pub const MYCITADEL_DATA_DIR: &'static str =
    "~/Library/Application Support/MyCitadel";
#[cfg(target_os = "windows")]
pub const MYCITADEL_DATA_DIR: &'static str = "~\\AppData\\Local\\MyCitadel";
#[cfg(target_os = "ios")]
pub const MYCITADEL_DATA_DIR: &'static str = "~/Documents";
#[cfg(target_os = "android")]
pub const MYCITADEL_DATA_DIR: &'static str = ".";

pub const MYCITADEL_CONFIG: &'static str = "{data_dir}/mycitadeld.toml";
#[cfg(feature = "serde_yaml")]
pub const MYCITADEL_STORAGE_FORMAT: FileFormat = FileFormat::Yaml;
#[cfg(not(feature = "serde_yaml"))]
pub const MYCITADEL_STORAGE_FORMAT: FileFormat = FileFormat::StrictEncoded;
pub const MYCITADEL_STORAGE_FILE: &'static str = "accounts.yaml";
pub const MYCITADEL_ELECTRUM_SERVER: &'static str =
    "http://pandora.network:60000";

#[derive(Clap, Clone, PartialEq, Eq, Hash, Debug)]
#[clap(
    name = "mycitadeld",
    bin_name = "mycitadeld",
    author,
    version,
    setting = AppSettings::ColoredHelp
)]
pub struct Opts {
    /// These params can be read also from the configuration file, not just
    /// command-line args or environment variables
    #[clap(flatten)]
    pub shared: crate::opts::SharedOpts,

    /// Blockchain to use
    #[clap(
        short = 'n',
        long,
        alias = "network",
        default_value = "signet",
        env = "MYCITADEL_NETWORK"
    )]
    pub chain: Chain,

    /// Initializes config file with the default values
    #[clap(long)]
    pub init: bool,

    /// Data directory path
    ///
    /// Path to the directory that contains LNP Node data, and where ZMQ RPC
    /// socket files are located
    #[clap(
        short,
        long,
        default_value = MYCITADEL_DATA_DIR,
        env = "MYCITADEL_DATA_DIR",
        value_hint = ValueHint::DirPath
    )]
    pub data_dir: PathBuf,

    /// Electrum server connection string
    #[clap(long, default_value = MYCITADEL_ELECTRUM_SERVER, env = "MYCITADEL_ELECTRUM_SERVER")]
    pub electrum_server: String,

    /// Path to the configuration file.
    ///
    /// NB: Command-line options override configuration file values.
    #[clap(
        short,
        long,
        default_value = MYCITADEL_CONFIG,
        env = "MYCITADEL_CONFIG",
        value_hint = ValueHint::FilePath
    )]
    pub config: String,
}

impl Opts {
    pub fn process(&mut self) {
        self.data_dir = PathBuf::from(
            shellexpand::tilde(&self.data_dir.to_string_lossy().to_string())
                .to_string(),
        );
        fs::create_dir_all(&self.data_dir)
            .expect("Unable to access data directory");

        let me = self.clone();

        match self.shared.rpc_socket {
            ZmqSocketAddr::Ipc(ref mut path) => {
                me.process_dir(path);
            }
            _ => {}
        }

        me.process_dir(&mut self.config);
    }

    pub fn process_dir(&self, path: &mut String) {
        *path = path.replace("{data_dir}", &self.data_dir.to_string_lossy());
        *path = path.replace("{network}", &self.chain.to_string());
        *path = shellexpand::tilde(path).to_string();
    }
}
