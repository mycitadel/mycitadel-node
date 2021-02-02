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

use internet2::ZmqSocketAddr;
use microservices::FileFormat;

use crate::client::{self, Client};
use crate::daemon;
use crate::Error;

pub fn run_embedded(mut config: daemon::Config) -> Result<Client, Error> {
    trace!("MyCitadel runtime configuration: {:#?}", &config);
    config.process();
    trace!("Processed configuration: {:#?}", &config);

    debug!("Starting MyCitadel node runtime in background");
    let data_dir = config.data_dir.clone();
    let data_dir_str = data_dir.to_string_lossy().to_string();
    let network = config.chain.clone();
    let verbose = config.verbose;
    let electrum_server = config.electrum_server.clone();
    let rgb20_endpoint = config.rgb20_endpoint.clone();

    debug!("Launching RGB node embedded runtime...");
    thread::spawn(move || {
        rgb_node::rgbd::main_with_config(rgb_node::rgbd::Config {
            data_dir,
            bin_dir: none!(),
            threaded: true,
            contracts: vec![rgb_node::rgbd::ContractName::Fungible],
            network,
            verbose,
            fungible_rpc_endpoint: rgb20_endpoint,
            stash_rpc_endpoint: ZmqSocketAddr::Inproc(s!("stash.rpc")),
            format: FileFormat::Json,
            cache: format!("{data_dir}/cache/rgb20/", data_dir = data_dir_str),
            stash: format!("{data_dir}/stash/", data_dir = data_dir_str),
            index: format!(
                "{data_dir}/stash/index.dat",
                data_dir = data_dir_str
            ),
            electrum_server,
        })
        .expect("Error in RGB node runtime");
        debug!("RGB node embedded runtime has successfully started");
    });

    let rpc_endpoint = config.rpc_endpoint.clone();
    thread::spawn(move || {
        daemon::run(config).expect("Error in MyCitadel daemon runtime")
    });

    Client::with(client::Config {
        rpc_endpoint,
        verbose,
    })
}
