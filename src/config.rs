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

use std::convert::TryInto;
use std::path::PathBuf;

use internet2::zmqsocket::ZmqSocketAddr;
use lnpbp::Chain;
use microservices::FileFormat;

#[cfg(feature = "shell")]
use crate::opts::Opts;
use crate::storage;

/// Final configuration resulting from data contained in config file environment
/// variables and command-line options. For security reasons node key is kept
/// separately.
#[derive(Clone, PartialEq, Eq, Debug, Display)]
#[display(Debug)]
pub struct Config {
    /// Bitcoin blockchain to use (mainnet, testnet, signet, liquid etc)
    pub chain: Chain,

    /// ZMQ socket for RPC API
    pub rpc_endpoint: ZmqSocketAddr,

    /// Data location
    pub data_dir: PathBuf,
}

impl Config {
    pub fn storage_conf(&self) -> storage::driver::Config {
        storage::driver::Config::File(storage::file::FileConfig {
            location: self.data_dir.to_string_lossy().to_string(),
            format: FileFormat::StrictEncode,
        })
    }
}

#[cfg(feature = "shell")]
impl From<Opts> for Config {
    fn from(opts: Opts) -> Self {
        Config {
            chain: opts.chain,
            data_dir: opts.data_dir,
            rpc_endpoint: opts.rpc_socket.try_into().expect(
                "The provided socket address must be a valid ZMQ socket",
            ),
        }
    }
}
