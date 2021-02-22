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

use std::collections::{BTreeMap, HashMap, HashSet};
use std::convert::TryInto;

use bitcoin::secp256k1::rand::{rngs::ThreadRng, RngCore};
use bitcoin::{OutPoint, Transaction, TxIn, TxOut, Txid};
use electrum_client::{Client as ElectrumClient, ElectrumApi};
use internet2::zmqsocket::{self, ZmqType};
use internet2::ZmqSocketAddr;
use internet2::{
    session, CreateUnmarshaller, PlainTranscoder, Session, TypedEnum,
    Unmarshall, Unmarshaller,
};
use lnpbp::seals::OutpointReveal;
use lnpbp::strict_encoding::StrictDecode;
use microservices::node::TryService;
use microservices::rpc::Failure;
use microservices::FileFormat;
use rgb::{SealDefinition, SealEndpoint, Validity, PSBT_OUT_PUBKEY};
use rgb20::Asset;
use rgb_node::rpc::reply::SyncFormat;
use rgb_node::rpc::reply::Transfer;
use rgb_node::util::ToBech32Data;
use wallet::bip32::{ChildIndex, UnhardenedIndex};
use wallet::descriptor::ContractDescriptor;
use wallet::psbt::raw::ProprietaryKey;
use wallet::script::PubkeyScript;
use wallet::{psbt, AddressPayload, Psbt};

use super::Config;
use crate::cache::{self, Driver as CacheDriver};
use crate::model::{Contract, Policy, Utxo};
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

    /// Random number generator (used in creation of blinding secrets)
    rng: ThreadRng,

    /// Known blockchain height by the last received block header
    known_height: u32,
}

impl Runtime {
    pub fn init(config: Config) -> Result<Self, Error> {
        debug!("Initializing wallet storage {:?}", config.storage_conf());
        let storage = storage::FileDriver::with(config.storage_conf())?;

        debug!("Initializing wallet cache {:?}", config.cache_conf());
        let cache = cache::FileDriver::with(config.cache_conf())?;

        debug!("Initializing random number generator");
        let rng = bitcoin::secp256k1::rand::thread_rng();

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
        let known_height = electrum.block_headers_subscribe()?.height as u32;

        let rgb_config = rgb_node::i9n::Config {
            verbose: config.verbose,
            data_dir: config.data_dir.clone().to_string_lossy().to_string(),
            electrum_server: config.electrum_server.clone(),
            stash_rpc_endpoint: ZmqSocketAddr::Inproc(s!("stash.rpc")),
            contract_endpoints: map! {
                rgb_node::rgbd::ContractName::Fungible => config.rgb20_endpoint.clone()
            },
            network: config.chain.clone(),
            run_embedded: config.rgb_embedded,
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
            rng,
            unmarshaller: Request::create_unmarshaller(),
            known_height,
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
            Request::CreateSingleSig(req) => {
                let contract = Contract::with(
                    Policy::Current(ContractDescriptor::SingleSig {
                        category: req.category,
                        pk: req.pubkey_chain,
                    }),
                    req.name,
                    self.config.chain.clone(),
                );
                self.storage
                    .add_contract(contract)
                    .map(Reply::Contract)
                    .map_err(Error::from)
            }

            Request::ListContracts => self
                .storage
                .contracts()
                .map(Reply::Contracts)
                .map_err(Error::from),

            Request::RenameContract(message::RenameContractRequest {
                contract_id,
                name,
            }) => self
                .storage
                .rename_contract(contract_id, name)
                .map(|_| Reply::Success)
                .map_err(Error::from),

            Request::DeleteContract(contract_id) => self
                .storage
                .delete_contract(contract_id)
                .map(|_| Reply::Success)
                .map_err(Error::from),

            Request::SyncContract(message::SyncContractRequest {
                contract_id,
                lookup_depth,
            }) => {
                let policy =
                    self.storage.policy(contract_id).map_err(Error::from)?;
                let lookup_depth = UnhardenedIndex::from(lookup_depth);
                let mut unspent: Vec<Utxo> = vec![];
                let mut outpoints: Vec<OutPoint> = vec![];
                let mut mine_info: BTreeMap<(u32, u16), Txid> = bmap!{};
                let mut index_offset = UnhardenedIndex::zero();
                loop {
                    let to = index_offset
                        .checked_add(lookup_depth)
                        .unwrap_or(UnhardenedIndex::largest());
                    let scripts = policy.derive_scripts(index_offset..to);
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
                                .map(|res| {
                                    let block_pos = (res.block_height as u32, res.pos as u16);
                                    mine_info.insert(block_pos, *txid);
                                    (*txid, block_pos)
                                })
                                .ok()
                        })
                        .collect::<HashMap<_, _>>();
                    trace!("Found txids: {:#?}", txids);
                    let batch = res
                        .iter()
                        .zip(scripts)
                        .filter_map(|(res, script)| {
                            let derivation_index = index_offset;
                            // If we overflow we simply ignore these iterations
                            index_offset.checked_inc_assign()?;
                            let _txids = txids.clone();
                            let r = res.iter().filter_map(move |entry| {
                                let tx_info = _txids.get(&entry.tx_hash)?;
                                let unspent = Utxo {
                                    value: entry.value,
                                    height: tx_info.0.try_into().ok()?,
                                    offset: tx_info.1.try_into().ok()?,
                                    txid: entry.tx_hash,
                                    address: AddressPayload::from_script(&script),
                                    vout: entry.tx_pos.try_into().ok()?,
                                    derivation_index,
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

                while let Ok(Some(info)) = self.electrum.block_headers_pop() {
                    self.known_height = info.height as u32;
                }

                let mut assets =
                    bmap! { rgb::ContractId::default() => unspent.clone() };
                for (utxo, outpoint) in unspent.iter_mut().zip(outpoints.iter())
                {
                    for (asset_id, amounts) in self
                        .rgb20_client
                        .outpoint_assets(*outpoint)
                        .map_err(Error::from)?
                    {
                        let mut u = utxo.clone();
                        u.value = amounts.iter().sum();
                        assets.entry(asset_id).or_insert(vec![]).push(u);
                    }
                }

                trace!("Mine info: {:#?}", mine_info);
                self.cache
                    .update(
                        contract_id,
                        mine_info,
                        Some(self.known_height),
                        outpoints,
                        assets.clone(),
                    )
                    .map_err(Error::from)?;
                Ok(Reply::ContractUnspent(assets))
            }

            Request::UsedAddresses(contract_id) => self
                .cache
                .used_address_derivations(contract_id)
                .map(Reply::Addresses)
                .map_err(Error::from),

            Request::NextAddress(message::NextAddressRequest {
                contract_id,
                index,
                legacy,
                mark_used,
            }) => self
                .storage
                .contract_ref(contract_id)
                .map_err(Error::from)?
                .derive_address(
                    index.unwrap_or(
                        self.cache
                            .next_unused_derivation(contract_id)
                            .map_err(Error::from)?,
                    ),
                    legacy,
                )
                .and_then(|address_derivation| {
                    if mark_used {
                        self.cache.use_address_derivation(
                            contract_id,
                            address_derivation.address.clone(),
                            *address_derivation.derivation.last().expect(
                                "derivation path must always have at least one element"
                            ),
                        ).ok()?;
                    }
                    Some(address_derivation)
                })
                .map(Reply::AddressDerivation)
                .ok_or(Error::ServerFailure(Failure {
                    code: 0,
                    info: s!("Unable to derive address for the provided network/chain"),
                })),

            Request::UnuseAddress(message::ContractAddressTuple {
                contract_id,
                address,
            }) => self
                .cache
                .forget_address(contract_id, &address)
                .map(|_| Reply::Success)
                .map_err(Error::from),

            Request::BlindUtxo(contract_id) => self
                .cache
                .utxo(contract_id)
                .map_err(Error::from)
                .and_then(|utxo| {
                    utxo.into_iter().next().ok_or(Error::ServerFailure(
                        Failure {
                            code: 0,
                            info: s!("No UTXO available"),
                        },
                    ))
                })
                .map(|outpoint| OutpointReveal::from(outpoint))
                .map(Reply::BlindUtxo),

            Request::ListInvoices(contract_id) => {
                self.storage
                    .contract_ref(contract_id)
                    .map(|contract| contract.data().sent_invoices().clone())
                    .map(Reply::Invoices)
                    .map_err(Error::from)
            },

            Request::AddInvoice(message::AddInvoiceRequest { invoice, source_info }) => {
                for (contract_id, outpoint_reveal) in source_info {
                    self.storage.add_invoice(
                        contract_id,
                        invoice.clone(),
                        outpoint_reveal.map(|r| vec![r]).unwrap_or_default()
                    ).map_err(Error::from)?;
                }
                Ok(Reply::Success)
            },

            Request::ComposePayment(message::ComposePaymentRequest { pay_from, amount, bitcoin_fee, transfer_info }) => {
                let contract = self.storage.contract_ref(pay_from).map_err(Error::from)?;
                let policy = contract.policy().clone();
                let mut coins = self.cache.unspent(pay_from).map_err(Error::from)?.get(&transfer_info.contract_id()).cloned().unwrap_or_default();
                // TODO: Implement more coinselection strategies
                coins.sort_by(|a, b| a.value.cmp(&b.value));
                trace!("Found coins: {:#?}", coins);

                // Collecting RGB witness/bitcoin payment inputs
                let mut input_amount = 0u64;
                let asset_fee = if transfer_info.is_rgb() { 0 } else { bitcoin_fee };

                let mut asset_change_outpoint = None;
                let spent_outpoints: Vec<OutPoint> = coins.into_iter().filter_map(|unspent| {
                    self.cache.blockpos_to_txid(unspent.height, unspent.offset).and_then(|txid| {
                        let outpoint = OutPoint::new(txid, unspent.vout as u32);
                        if input_amount >= amount + asset_fee {
                            asset_change_outpoint = Some(outpoint);
                            return None
                        }
                        input_amount += unspent.value;
                        trace!("Adding {} to the inputs with {} sats; total input value is {}", outpoint, unspent.value, input_amount);
                        Some(outpoint)
                    })
                }).collect();
                let input: Vec<TxIn> = spent_outpoints.iter().map(|outpoint| {
                    TxIn {
                        previous_output: *outpoint,
                        script_sig: Default::default(),
                        sequence: 0,
                        witness: vec![],
                    }
                }).collect();
                if input_amount < amount + asset_fee {
                    Err(Error::ServerFailure(Failure {
                        code: 0,
                        info: s!("Insufficient funds")
                    }))?;
                }

                // Constructing RGB witness/bitcoin payment transaction outputs
                let mut output = vec![];
                let mut bitcoin_amount = 0u64;
                let rgb_endpoint = if let Some(descriptor) = transfer_info.bitcoin_descriptor() {
                    // We need this output only for bitcoin payments
                    trace!("Adding output paying {} to {}", amount, descriptor);
                    bitcoin_amount = amount;
                    output.push((TxOut {
                        value: amount,
                        script_pubkey: PubkeyScript::from(descriptor).into(),
                    }, None));
                    SealEndpoint::TxOutpoint(default!())
                } else if let message::TransferInfo::Rgb {
                    contract_id,
                    receiver: message::RgbReceiver::Descriptor { ref descriptor, giveaway }
                } = transfer_info {
                    // We need this output only for descriptor-based RGB payments
                    trace!("Adding output paying {} bitcoin giveaway to {}", giveaway, descriptor);
                    bitcoin_amount = giveaway;
                    output.push((TxOut {
                        value: giveaway,
                        script_pubkey: PubkeyScript::from(descriptor.clone()).into(),
                    }, None));
                    SealEndpoint::with_vout(output.len() as u32 - 1, &mut self.rng)
                } else if let message::TransferInfo::Rgb {
                    contract_id: _, receiver: message::RgbReceiver::BlindUtxo(hash)
                } = transfer_info {
                    SealEndpoint::TxOutpoint(hash)
                } else {
                    unimplemented!()
                };

                // Get to known how much bitcoins we are spending
                let mut input_bitcoin_amount = 0u64;
                let mut outpoint_check = spent_outpoints.clone();
                for unspent in self.cache.unspent(pay_from).map_err(Error::from)?.get(&rgb::ContractId::default()).ok_or(Error::CacheInconsistency)? {
                    let txid = self.cache.blockpos_to_txid(unspent.height, unspent.offset).ok_or(Error::CacheInconsistency)?;
                    if let Some(index) = outpoint_check.iter().position(|o| *o == OutPoint::new(txid, unspent.vout as u32)) {
                        outpoint_check.remove(index);
                        input_bitcoin_amount += unspent.value;
                    }
                }
                if outpoint_check.len() > 0 {
                    Err(Error::CacheInconsistency)?
                }

                // Adding bitcoin change output, if needed
                let change_vout = if input_bitcoin_amount > bitcoin_amount + bitcoin_fee {
                    let change = input_bitcoin_amount - bitcoin_amount - bitcoin_fee;
                    let change_index = self.cache
                        .next_unused_derivation(pay_from).map_err(Error::from)?;
                    let change_address = contract.derive_address(change_index, false).ok_or(Error::ServerFailure(Failure {
                        code: 0,
                        info: s!("Unable to derive change address"),
                    }))?.address;
                    trace!("Adding change output paying {} to our address {} at derivation index {}", change, change_address, change_index);
                    output.push((TxOut {
                        value: change,
                        script_pubkey: change_address.script_pubkey(),
                    }, Some(change_index)));
                    Some(output.len() as u32 - 1)
                } else {
                    None
                };

                // Adding RGB change output, if needed
                // NB: Right now, we use really dumb algorithm, allocating 
                //     change to first found outpoint with existing assignment
                //     of the same asset, or, if none, to the bitcoin change
                //     output - failing if neither of them is present. We can
                //     be much smarter, assigning to existing bitcoin utxos,
                //     or creating new output for RGB change
                let mut rgb_change = bmap! {};
                if input_amount > amount && transfer_info.is_rgb() {
                    let change = input_amount - amount;
                    rgb_change.insert(
                        asset_change_outpoint.map(|outpoint| SealDefinition::TxOutpoint(
                            OutpointReveal {
                                blinding: self.rng.next_u64(),
                                txid: outpoint.txid,
                                vout: outpoint.vout
                            }
                        )).or_else(|| change_vout.map(|vout| SealDefinition::WitnessVout {
                            vout,
                            blinding: self.rng.next_u64()
                        })).ok_or(Error::ServerFailure(Failure {
                            code: 0,
                            info: s!("Can't allocate RGB change")
                        }))?,
                        change
                    );
                }

                // Constructing bitcoin payment PSBT (for bitcoin payments) or
                // RGB witness PSBT prototype for the commitment (for RGB
                // payments)
                let inputs = input.iter().map(|txin| {
                    let mut input = psbt::Input::default();
                    // TODO: cache transactions
                    input.non_witness_utxo = self.electrum.transaction_get(&txin.previous_output.txid).ok();
                    input
                }).collect();
                let outputs = output.iter().map(|(txout, index)| {
                    let mut output = psbt::Output::default();
                    if let Some(index) = index {
                        output.proprietary.insert(
                            ProprietaryKey {
                                prefix: b"RGB".to_vec(),
                                subtype: PSBT_OUT_PUBKEY,
                                key: vec![],
                            },
                            policy.first_public_key(*index).to_bytes(),
                        );
                    }
                    output
                }).collect();
                let psbt = Psbt {
                    global: psbt::Global {
                        unsigned_tx: Transaction {
                            version: 1,
                            lock_time: 0,
                            input,
                            output: output.into_iter().map(|x| x.0).collect(),
                        },
                        version: 0,
                        xpub: none!(),
                        proprietary: none!(),
                        unknown: none!()
                    },
                    inputs,
                    outputs,
                };
                trace!("Resulting PSBT: {:#?}", psbt);

                // Committing to RGB transfer into the witness transaction and
                // producing consignments (applies to RGB payments only)
                let payment_data = if let message::TransferInfo::Rgb { contract_id: asset_id, ref receiver} = transfer_info {
                    let Transfer { consignment, witness } = self.rgb20_client.transfer(
                        asset_id,
                        spent_outpoints.into_iter().collect(),
                        bmap! { rgb_endpoint => amount },
                        rgb_change,
                        psbt.clone()
                    ).map_err(Error::from)?;
                    message::PreparedPayment { psbt: witness, consignment: Some(consignment) }
                } else {
                    message::PreparedPayment { psbt, consignment: None }
                };

                Ok(Reply::PreparedPayment(payment_data))
            },

            Request::AcceptPayment(consignment) => {
                let status = self.rgb20_client.validate(consignment.clone()).map_err(Error::from)?;
                if status.validity() == Validity::Valid {
                    let hashes = consignment.endpoints.iter().filter_map(|(_, seal_endpoint)| match seal_endpoint {
                        SealEndpoint::TxOutpoint(hash) => Some(*hash),
                        SealEndpoint::WitnessVout { .. } => None,
                    }).collect::<Vec<_>>();
                    let revel_outpoints = self.storage
                        .contracts().map_err(Error::from)?
                        .iter()
                        .flat_map(|contract| contract.data().blinding_factors())
                        .filter_map(|(hash, reveal)| {
                            if hashes.contains(hash) {
                                Some(*reveal)
                            } else {
                                None
                            }
                        }).collect();
                    self.rgb20_client.accept(consignment, revel_outpoints).map_err(Error::from)?;
                }
                Ok(Reply::Validation(status))
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

            Request::ListAssets => self
                .rgb20_client
                .list_assets(FileFormat::StrictEncode)
                .map_err(Error::from)
                .and_then(|SyncFormat(_, data)| {
                    Vec::<Asset>::strict_deserialize(data).map_err(Error::from)
                })
                .map(Reply::Assets),

        }
        .map_err(Error::into)
    }
}
