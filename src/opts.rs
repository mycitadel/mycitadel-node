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

use clap::{Clap, ValueHint};
use std::net::SocketAddr;

use internet2::ZmqSocketAddr;

pub const MYCITADEL_RPC_SOCKET_NAME: &'static str =
    "lnpz://0.0.0.0:61399?api=rpc"; //"ipc:{data_dir}/zmq.rpc";

#[derive(Clap, Clone, PartialEq, Eq, Hash, Debug)]
pub struct SharedOpts {
    /// Set verbosity level
    ///
    /// Can be used multiple times to increase verbosity
    #[clap(short, long, global = true, parse(from_occurrences))]
    pub verbose: u8,

    /// Use Tor
    ///
    /// If set, specifies SOCKS5 proxy used for Tor connectivity and directs
    /// all network traffic through Tor network.
    /// If the argument is provided in form of flag, without value, uses
    /// `127.0.0.1:9050` as default Tor proxy address.
    #[clap(
        short = 'T',
        long,
        alias = "tor",
        env = "MYCITADEL_TOR_PROXY",
        value_hint = ValueHint::Hostname
    )]
    pub tor_proxy: Option<Option<SocketAddr>>,

    /// ZMQ socket name/address for daemon RPC interface
    ///
    /// Internal interface for control PRC protocol communications
    /// Defaults to `ctl.rpc` file inside `--data-dir` directory, unless
    /// `--use-threads` is specified; in that cases uses in-memory
    /// communication protocol.
    #[clap(
        short = 'x',
        long,
        env = "MYCITADEL_RPC_SOCKET",
        value_hint = ValueHint::FilePath,
        default_value = MYCITADEL_RPC_SOCKET_NAME
    )]
    pub rpc_socket: ZmqSocketAddr,
}
