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

mod opts;

pub use opts::Opts;

use std::thread;

use microservices::FileFormat;

use crate::client::{self, Client};
use crate::daemon;
use crate::Error;

pub fn run_embedded(mut opts: Opts) -> Result<Client, Error> {
    debug!("Starting RGBd node instance in background");

    opts.daemon.shared.rpc_socket = "inproc://mycitadel.rpc"
        .parse()
        .expect("MyCitadel RPC address error");

    let data_dir = opts.daemon.data_dir.clone();
    let network = opts.daemon.chain.clone();
    let verbose = opts.daemon.shared.verbose;
    let electrum_server = opts.daemon.electrum_server.clone();
    thread::spawn(move || {
        rgb_node::rgbd::main_with_config(rgb_node::rgbd::Config {
            data_dir,
            bin_dir: none!(),
            threaded: true,
            contracts: vec![rgb_node::rgbd::ContractName::Fungible],
            network,
            verbose,
            fungible_rpc_endpoint: s!("inproc://fungible.rpc"),
            fungible_pub_endpoint: s!("inproc://fungible.pub"),
            stash_rpc_endpoint: s!("inproc://stash.rpc"),
            stash_pub_endpoint: s!("inproc://stash.pub"),
            format: FileFormat::Json,
            cache: s!("{data_dir}/{network}/cache/fungible"),
            stash: s!("{data_dir}/{network}/stash/{id}/"),
            index: s!("{data_dir}/{network}/stash/{id}/index.dat"),
            p2p_endpoint: "0.0.0.0:6161"
                .parse()
                .expect("RGB P2P address error"),
            electrum_server,
        })
        .expect("Error in RGBd runtime");
    });

    debug!("Starting MyCitadel node instance in background");
    let config = daemon::Config::from(opts.daemon);
    let rpc_endpoint = config.rpc_endpoint.clone();
    thread::spawn(move || {
        daemon::run(config).expect("Error in MyCitadel daemon runtime")
    });

    Client::with(client::Config {
        rpc_endpoint,
        verbose,
    })
}
