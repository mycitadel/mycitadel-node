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

use internet2::zmqsocket::{self, ZmqType};
use internet2::ZmqSocketAddr;
use internet2::{
    session, CreateUnmarshaller, PlainTranscoder, Session, TypedEnum,
    Unmarshall, Unmarshaller,
};
use lnpbp::strict_encoding::StrictDecode;
use microservices::node::TryService;
use microservices::FileFormat;
use rgb20::Asset;
use rgb_node::rpc::reply::SyncFormat;
use rgb_node::util::ToBech32Data;
use wallet::descriptor::ContractDescriptor;

use super::Config;
use crate::cache::{self, Driver as CacheDriver};
use crate::model::{Contract, Policy};
use crate::rpc::{Reply, Request};
use crate::storage::{self, Driver as StorageDriver};
use crate::Error;

pub fn run(config: Config) -> Result<(), Error> {
    let runtime = Runtime::init(config)?;

    runtime.run_or_panic("mycitadeld");

    Ok(())
}

pub struct Runtime {
    /// Original configuration object
    config: Config,

    /// Stored sessions
    session_rpc: session::Raw<PlainTranscoder, zmqsocket::Connection>,

    /// Wallet data storage
    storage: storage::FileDriver,

    /// Wallet data cache
    cache: cache::FileDriver,

    /// Unmarshaller instance used for parsing RPC request
    unmarshaller: Unmarshaller<Request>,

    /// RGB20 (fungibled) daemon client
    rgb20_client: rgb_node::i9n::Runtime,
}

impl Runtime {
    pub fn init(config: Config) -> Result<Self, Error> {
        debug!("Initializing wallet storage {:?}", config.storage_conf());
        let storage = storage::FileDriver::with(config.storage_conf())?;

        debug!("Initializing wallet cache {:?}", config.cache_conf());
        let cache = cache::FileDriver::with(config.cache_conf())?;

        debug!("Opening RPC API socket {}", config.rpc_endpoint);
        let session_rpc = session::Raw::with_zmq_unencrypted(
            ZmqType::Rep,
            &config.rpc_endpoint,
            None,
            None,
        )?;

        debug!("Connecting RGB node embedded runtime...");
        let rgb20_client =
            rgb_node::i9n::Runtime::init(rgb_node::i9n::Config {
                verbose: config.verbose,
                data_dir: config.data_dir.clone().to_string_lossy().to_string(),
                electrum_server: config.electrum_server.clone(),
                stash_rpc_endpoint: ZmqSocketAddr::Inproc(s!("stash.rpc")),
                contract_endpoints: map! {
                    rgb_node::rgbd::ContractName::Fungible => config.rgb20_endpoint.clone()
                },
                network: config.chain.clone(),
                run_embedded: false,
            })
            .map_err(|_| Error::EmbeddedNodeError)?;

        debug!("RGB node runtime has successfully connected");

        info!("MyCitadel runtime started successfully");

        Ok(Self {
            config,
            session_rpc,
            storage,
            cache,
            rgb20_client,
            unmarshaller: Request::create_unmarshaller(),
        })
    }
}

impl TryService for Runtime {
    type ErrorType = Error;

    fn try_run_loop(mut self) -> Result<(), Self::ErrorType> {
        loop {
            match self.run() {
                Ok(_) => debug!("API request processing complete"),
                Err(err) => {
                    error!("Error processing API request: {}", err);
                    Err(err)?;
                }
            }
        }
    }
}

impl Runtime {
    fn run(&mut self) -> Result<(), Error> {
        trace!("Awaiting for ZMQ RPC requests...");
        let raw = self.session_rpc.recv_raw_message()?;
        let reply = self.rpc_process(raw).unwrap_or_else(|err| err);
        trace!("Preparing ZMQ RPC reply: {:?}", reply);
        let data = reply.serialize();
        trace!(
            "Sending {} bytes back to the client over ZMQ RPC",
            data.len()
        );
        self.session_rpc.send_raw_message(&data)?;
        Ok(())
    }

    fn rpc_process(&mut self, raw: Vec<u8>) -> Result<Reply, Reply> {
        trace!(
            "Got {} bytes over ZMQ RPC: {}",
            raw.len(),
            raw.to_bech32data()
        );
        let message = (&*self.unmarshaller.unmarshall(&raw)?).clone();
        debug!(
            "Received ZMQ RPC request #{}: {}",
            message.get_type(),
            message
        );
        match message {
            Request::ListContracts => self
                .storage
                .contracts()
                .map(Reply::Contracts)
                .map_err(Error::from),
            Request::ContractUnspent(id) => self
                .cache
                .unspent(id)
                .map(Reply::ContractUnspent)
                .map_err(Error::from),
            Request::ListIdentities => self
                .storage
                .identities()
                .map(Reply::Identities)
                .map_err(Error::from),
            Request::ListAssets => self
                .rgb20_client
                .list_assets(FileFormat::StrictEncode)
                .map_err(|_| storage::Error::Remote)
                .and_then(|SyncFormat(_, data)| {
                    Vec::<Asset>::strict_deserialize(data)
                        .map_err(storage::Error::from)
                })
                .map(Reply::Assets)
                .map_err(Error::from),
            Request::CreateSingleSig(req) => {
                let contract = Contract::with(
                    Policy::Current(ContractDescriptor::SingleSig {
                        category: req.category,
                        pk: req.pubkey_chain,
                    }),
                    req.name,
                );
                self.storage
                    .add_contract(contract)
                    .map(Reply::Contract)
                    .map_err(Error::from)
            }
            Request::AddSigner(account) => self
                .storage
                .add_signer(account)
                .map(|_| Reply::Success)
                .map_err(Error::from),
            Request::AddIdentity(identity) => self
                .storage
                .add_identity(identity)
                .map(|_| Reply::Success)
                .map_err(Error::from),
            Request::ImportAsset(genesis) => self
                .rgb20_client
                .import_asset(genesis)
                .map(Reply::Asset)
                .map_err(Error::from),
            _ => unimplemented!(),
        }
        .map_err(Error::into)
    }
}
