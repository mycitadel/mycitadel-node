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

use std::convert::TryInto;

use internet2::zmqsocket::ZmqSocketAddr;

/// Final configuration resulting from data contained in config file environment
/// variables and command-line options. For security reasons node key is kept
/// separately.
#[derive(Clone, PartialEq, Eq, Debug, Display)]
#[display(Debug)]
pub struct Config {
    /// ZMQ socket for RPC API
    pub rpc_endpoint: ZmqSocketAddr,

    /// Verbosity level
    pub verbose: u8,
}

#[cfg(feature = "shell")]
impl From<crate::cli::Opts> for Config {
    fn from(opts: crate::cli::Opts) -> Self {
        Config {
            rpc_endpoint: opts.shared.rpc_socket.try_into().expect(
                "The provided socket address must be a valid ZMQ socket",
            ),
            verbose: opts.shared.verbose,
        }
    }
}
