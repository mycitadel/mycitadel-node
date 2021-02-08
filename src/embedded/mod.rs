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
use crate::server;
use crate::Error;
use microservices::node::TryService;

pub fn run_embedded(mut config: server::Config) -> Result<Client, Error> {
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
    let stash_endpoint = ZmqSocketAddr::Inproc(s!("stash.rpc"));
    let stash = format!("{data_dir}/stash/", data_dir = data_dir_str);
    let cache = format!("{data_dir}/", data_dir = data_dir_str);
    let index = format!("{data_dir}/index.dat", data_dir = data_dir_str);

    debug!("Launching RGB node embedded runtime...");
    let stashd = rgb_node::stashd::Runtime::init(rgb_node::stashd::Config {
        verbose,
        data_dir: data_dir.clone(),
        stash: stash.clone(),
        index: index.clone(),
        rpc_endpoint: stash_endpoint.clone(),
        network: network.clone(),
        electrum_server: electrum_server.clone(),
        ..default!()
    })
    .expect("Error launching RGB node stashd thread");

    let rgb20d =
        rgb_node::fungibled::Runtime::init(rgb_node::fungibled::Config {
            verbose,
            data_dir,
            cache,
            format: FileFormat::Yaml,
            rpc_endpoint: rgb20_endpoint,
            stash_rpc: stash_endpoint,
            network,
        })
        .expect("Error launching RGB node fungibled thread");

    thread::spawn(move || {
        stashd.run_or_panic("stashd");
    });

    thread::spawn(move || {
        rgb20d.run_or_panic("fungibled");
    });
    debug!("RGB node embedded runtime has successfully started");

    let rpc_endpoint = config.rpc_endpoint.clone();
    thread::spawn(move || {
        server::run(config).expect("Error in MyCitadel runtime")
    });

    Client::with(client::Config {
        rpc_endpoint,
        verbose,
    })
}
