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

pub fn run_embedded(mut config: daemon::Config) -> Result<Client, Error> {
    debug!("Starting RGBd node instance in background");

    config.rpc_endpoint = "inproc://mycitadel.rpc"
        .parse()
        .expect("MyCitadel RPC address error");
    config.rgb20_endpoint = "inproc://rgb20.rpc"
        .parse()
        .expect("RGB20 RPC address error");

    let data_dir = config.data_dir.clone();
    let network = config.chain.clone();
    let verbose = config.verbose;
    let electrum_server = config.electrum_server.clone();
    let rgb20_endpoint = config.rgb20_endpoint.clone();
    thread::spawn(move || {
        rgb_node::rgbd::main_with_config(rgb_node::rgbd::Config {
            data_dir,
            bin_dir: none!(),
            threaded: true,
            contracts: vec![rgb_node::rgbd::ContractName::Fungible],
            network,
            verbose,
            fungible_rpc_endpoint: rgb20_endpoint.to_string(),
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
    let rpc_endpoint = config.rpc_endpoint.clone();
    thread::spawn(move || {
        daemon::run(config).expect("Error in MyCitadel daemon runtime")
    });

    Client::with(client::Config {
        rpc_endpoint,
        verbose,
    })
}
