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

use std::collections::{HashMap, HashSet};
use std::convert::TryInto;

use bitcoin::OutPoint;
use electrum_client::{Client as ElectrumClient, ElectrumApi};
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
use wallet::bip32::{ChildIndex, UnhardenedIndex};
use wallet::descriptor::ContractDescriptor;

use super::Config;
use crate::cache::{self, Driver as CacheDriver};
use crate::model::{Contract, Policy, Unspent};
use crate::rpc::{message, Reply, Request};
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

    /// Electrum server connection
    electrum: ElectrumClient,

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

        debug!(
            "Connecting electrum server at {} ...",
            config.electrum_server
        );
        debug!("Electrum server successfully connected");
        let electrum =
            ElectrumClient::new(&config.electrum_server.to_string())?;
        debug!("Subscribing to new block notifications");
        electrum.block_headers_subscribe()?;

        let rgb_config = rgb_node::i9n::Config {
            verbose: config.verbose,
            data_dir: config.data_dir.clone().to_string_lossy().to_string(),
            electrum_server: config.electrum_server.clone(),
            stash_rpc_endpoint: ZmqSocketAddr::Inproc(s!("stash.rpc")),
            contract_endpoints: map! {
                rgb_node::rgbd::ContractName::Fungible => config.rgb20_endpoint.clone()
            },
            network: config.chain.clone(),
            run_embedded: false,
        };
        debug!(
            "Connecting RGB node embedded runtime using config {}...",
            rgb_config
        );
        let rgb20_client = rgb_node::i9n::Runtime::init(rgb_config)
            .map_err(|_| Error::EmbeddedNodeInitError)?;
        debug!("RGB node runtime successfully connected");

        info!("MyCitadel runtime started successfully");

        Ok(Self {
            config,
            session_rpc,
            electrum,
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

            Request::SyncContract(message::SyncContractRequest {
                contract_id,
                lookup_depth,
            }) => {
                let policy =
                    self.storage.policy(contract_id).map_err(Error::from)?;
                let mut unspent: Vec<Unspent> = vec![];
                let mut outpoints: Vec<OutPoint> = vec![];
                let mut index_offset = 0;
                loop {
                    let from: u32 = index_offset.into();
                    index_offset += lookup_depth as u32;
                    let scripts =
                        policy.derive_scripts(from..index_offset.into());
                    let res = self
                        .electrum
                        .batch_script_list_unspent(&scripts)
                        .map_err(|_| Error::Electrum)?;
                    let txids = res
                        .iter()
                        .flatten()
                        .map(|entry| (entry.tx_hash, entry.height))
                        .collect::<HashSet<_>>()
                        .iter()
                        .filter_map(|(txid, height)| {
                            self.electrum
                                .transaction_get_merkle(txid, *height)
                                .map(|res| (*txid, (res.block_height, res.pos)))
                                .ok()
                        })
                        .collect::<HashMap<_, _>>();
                    let batch = res
                        .iter()
                        .enumerate()
                        .filter_map(|(idx, res)| {
                            let index = UnhardenedIndex::from_index(
                                idx as u32 + index_offset,
                            )
                            .ok()?;
                            let _txids = txids.clone();
                            let r = res.iter().filter_map(move |entry| {
                                let ix_info = _txids.get(&entry.tx_hash)?;
                                let unspent = Unspent {
                                    value: entry.value,
                                    height: ix_info.0.try_into().ok()?,
                                    offset: ix_info.1.try_into().ok()?,
                                    vout: entry.tx_pos.try_into().ok()?,
                                    index,
                                };
                                let outpoint = OutPoint::new(
                                    entry.tx_hash,
                                    entry.tx_pos as u32,
                                );
                                Some((unspent, outpoint))
                            });
                            Some(r)
                        })
                        .flatten()
                        .unzip::<_, _, Vec<_>, Vec<_>>();
                    if batch.0.is_empty() {
                        break;
                    }
                    unspent.extend(batch.0);
                    outpoints.extend(batch.1);
                }

                let mut height = None;
                while let Ok(Some(info)) = self.electrum.block_headers_pop() {
                    height = Some(info.height as u32);
                }

                let mut assets =
                    bmap! { rgb::ContractId::default() => unspent.clone() };
                for (utxo, outpoint) in unspent.iter_mut().zip(outpoints) {
                    for (asset_id, amounts) in self
                        .rgb20_client
                        .outpoint_assets(outpoint)
                        .map_err(Error::from)?
                    {
                        let mut u = utxo.clone();
                        u.value = amounts.iter().sum();
                        assets.entry(asset_id).or_insert(vec![]).push(u);
                    }
                }

                self.cache
                    .update(contract_id, height, assets.clone())
                    .map_err(Error::from)?;
                Ok(Reply::ContractUnspent(assets))
            }

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
                .map_err(Error::from)
                .and_then(|SyncFormat(_, data)| {
                    Vec::<Asset>::strict_deserialize(data).map_err(Error::from)
                })
                .map(Reply::Assets),
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
