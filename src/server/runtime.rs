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

use chrono::{NaiveDateTime, Utc};
use std::collections::{BTreeMap, BTreeSet};
use std::convert::TryInto;

use bitcoin::secp256k1::rand::{rngs::ThreadRng, RngCore};
use bitcoin::{OutPoint, PublicKey, Script, Transaction, TxIn, TxOut, Txid};
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
use miniscript::{Descriptor, DescriptorTrait};
use rgb::{SealDefinition, SealEndpoint, Validity};
use rgb20::Asset;
use rgb_node::rpc::reply::SyncFormat;
use rgb_node::rpc::reply::Transfer;
use rgb_node::util::ToBech32Data;
use wallet::bip32::{ChildIndex, UnhardenedIndex};
use wallet::descriptor::ContractDescriptor;
use wallet::psbt::{ProprietaryKey, ProprietaryWalletInput};
use wallet::script::PubkeyScript;
use wallet::{psbt, AddressCompat, Psbt, Slice32};

use super::Config;
use crate::cache::{self, Driver as CacheDriver};
use crate::model::{
    Contract, ContractMeta, Operation, PaymentDirecton, Policy, PsbtWrapper,
    SpendingPolicy, TweakedOutput, Utxo,
};
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
                    .map(ContractMeta::from)
                    .map(Reply::Contract)
                    .map_err(Error::from)
            }

            Request::ContractOperations(contract_id) => self
                .storage
                .contract_ref(contract_id)
                .map(|contract| contract.history())
                .map(Reply::Operations)
                .map_err(Error::from),

            Request::ListContracts => self
                .storage
                .contracts()
                .map(|vec| vec.into_iter().map(ContractMeta::from).collect::<Vec<_>>())
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
                debug!("Synchronizing contract data with electrum server");

                let lookup_depth = UnhardenedIndex::from(lookup_depth);

                let contract = self.storage.contract_ref(contract_id).map_err(Error::from)?;
                let policy =
                    self.storage.policy(contract_id).map_err(Error::from)?;

                let mut unspent: Vec<Utxo> = vec![];
                let mut outpoints: BTreeSet<OutPoint> = bset![];
                let mut mine_info: BTreeMap<(u32, u16), Txid> = bmap!{};

                let mut index_offset = UnhardenedIndex::zero();
                let last_used_index = self.cache.last_used_derivation(contract_id).unwrap_or_default();

                let mut scripts: Vec<(UnhardenedIndex, Script, Option<TweakedOutput>)> = contract
                    .data()
                    .p2c_tweaks()
                    .into_iter()
                    .map(|tweak| (tweak.derivation_index, tweak.script.clone(), Some(tweak.clone())))
                    .collect();
                debug!("Requesting unspent information for {} known tweaked scripts", scripts.len());

                loop {
                    let mut count = 0usize;
                    trace!("{:#?}", scripts);

                    let txid_map = self
                        .electrum
                        .batch_script_list_unspent(&scripts.iter().map(|(_, script, _)| script.clone()).collect::<Vec<_>>())
                        .map_err(|_| Error::Electrum)?
                        .into_iter()
                        .zip(scripts)
                        .fold(
                            BTreeMap::<(u32, Txid), Vec<(u16, u64, UnhardenedIndex, Script, Option<TweakedOutput>)>>::new(),
                            |mut map, (found, (derivation_index, script, tweak))| {
                                for item in found {
                                    map.entry((item.height as u32, item.tx_hash))
                                        .or_insert(Vec::new())
                                        .push((item.tx_pos as u16, item.value, derivation_index, script.clone(), tweak.clone()));
                                    count += 1;
                                }
                                map
                            }
                        );
                    debug!("Found {} unspent outputs in the batch", count);
                    trace!("{:#?}", txid_map);

                    trace!("Resolving block transaction position for {} transactions", txid_map.len());
                    for ((height, txid), outs) in txid_map {
                        match self.electrum.transaction_get_merkle(&txid, height as usize) {
                            Ok(res) => {
                                mine_info.insert((height, res.pos as u16), txid);
                                for (vout, value, derivation_index, script, tweak) in outs {
                                    if !outpoints.insert(OutPoint::new(txid, vout as u32)) {
                                        continue
                                    }
                                    unspent.push(Utxo {
                                        value,
                                        height,
                                        offset: res.pos as u16,
                                        txid,
                                        vout,
                                        derivation_index,
                                        tweak: tweak.map(|tweak| (tweak.tweak, tweak.pubkey)),
                                        address: contract.chain().try_into().ok().and_then(|network| AddressCompat::from_script(&script, network))
                                    });
                                }
                            },
                            Err(err) => warn!(
                                "Unable to get tx block position for {} at height {}: electrum server error {:?}",
                                txid, height, err
                            ),
                        }
                    }

                    if count == 0 && index_offset > last_used_index {
                        debug!(
                            "No unspent outputs are found in the batch and we \
                            are behind the last used derivation; stopping search"
                        );
                        break;
                    }

                    if index_offset == UnhardenedIndex::largest() {
                        debug!("Reached last possible index number, breaking");
                        break;
                    }
                    let from = index_offset;
                    index_offset = index_offset
                        .checked_add(lookup_depth)
                        .unwrap_or(UnhardenedIndex::largest());
                    scripts = policy
                        .derive_scripts(from..index_offset)
                        .into_iter()
                        .map(|(derivation_index, script)| (derivation_index, script, None))
                        .collect();
                    debug!("Generating next spending script batch");
                }

                while let Ok(Some(info)) = self.electrum.block_headers_pop() {
                    debug!("Updating known blockchain height: {}", info.height);
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
                        if amounts.is_empty() {
                            continue;
                        }
                        let amount = amounts.iter().sum();
                        if amount > 0 {
                            let mut u = utxo.clone();
                            u.value = amount;
                            assets.entry(asset_id).or_insert(vec![]).push(u);
                        }
                    }
                }

                trace!("Transaction mining info: {:#?}", mine_info);
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

            Request::ComposeTransfer(message::ComposeTransferRequest { pay_from, asset_value, bitcoin_fee, transfer_info, invoice }) => {
                let contract = self.storage.contract_ref(pay_from).map_err(Error::from)?;
                let policy: Policy = contract.policy().clone();

                // For pure bitcoin transfers we must avoid using outputs
                // containing RGB assets
                // TODO: Support using RGB-containing outputs moving RGB assets
                //       if possible
                let mut coins = if transfer_info.is_rgb() {
                    self.cache
                        .unspent(pay_from)
                        .map_err(Error::from)?
                        .get(&transfer_info.contract_id())
                        .cloned()
                        .unwrap_or_default()
                } else {
                    self.cache
                        .unspent_bitcoin_only(pay_from)
                        .map_err(Error::from)?
                }.into_iter().collect::<Vec<_>>();

                // TODO: Implement more coin-selection strategies
                coins.sort_by(|a, b| a.value.cmp(&b.value));
                coins.reverse();

                trace!("Found coins: {:#?}", coins);

                // Collecting RGB witness/bitcoin payment inputs
                let mut asset_input_amount = 0u64;
                let asset_fee = if transfer_info.is_rgb() { 0 } else { bitcoin_fee };
                let balance_before = coins.iter().map(|utxo| utxo.value).sum();

                let mut asset_change_outpoint = None;
                let selected_utxos: Vec<Utxo> = coins.into_iter().filter_map(|utxo| {
                    if asset_input_amount >= asset_value + asset_fee {
                        debug!("Change value {} will be allocated to {}", asset_input_amount - asset_value - asset_fee, utxo.outpoint());
                        asset_change_outpoint = asset_change_outpoint.or(Some(utxo.outpoint()));
                        return None
                    }
                    if utxo.value == 0 {
                        return None
                    }
                    asset_input_amount += utxo.value;
                    trace!("Adding {} to the inputs with {} sats; total input value is {}", utxo.outpoint(), utxo.value, asset_input_amount);
                    Some(utxo)
                }).collect();
                let tx_inputs: Vec<TxIn> = selected_utxos.iter().map(|utxo| {
                    TxIn {
                        previous_output: utxo.outpoint(),
                        script_sig: Default::default(),
                        sequence: 0,
                        witness: vec![],
                    }
                }).collect();
                if asset_input_amount < asset_value + asset_fee {
                    Err(Error::ServerFailure(Failure {
                        code: 0,
                        info: format!(
                            "Insufficient funds{}",
                            if transfer_info.is_rgb() {
                                ""
                            } else {
                                " on bitcoin outputs which do not have RGB assets on them"
                            }
                        )
                    }))?;
                }

                // Constructing RGB witness/bitcoin payment transaction outputs
                let mut tx_outputs = vec![];
                let mut bitcoin_value = 0u64;
                let mut bitcoin_giveaway = None;
                let rgb_endpoint = if let Some(descriptor) = transfer_info.bitcoin_descriptor() {
                    // We need this output only for bitcoin payments
                    trace!("Adding output paying {} to {}", asset_value, descriptor);
                    bitcoin_value = asset_value;
                    tx_outputs.push((TxOut {
                        value: asset_value,
                        script_pubkey: PubkeyScript::from(descriptor).into(),
                    }, None));
                    SealEndpoint::TxOutpoint(default!())
                } else if let message::TransferInfo::Rgb {
                    contract_id,
                    receiver: message::RgbReceiver::Descriptor { ref descriptor, giveaway }
                } = transfer_info {
                    // We need this output only for descriptor-based RGB payments
                    trace!("Adding output paying {} bitcoin giveaway to {}", giveaway, descriptor);
                    bitcoin_giveaway = Some(giveaway);
                    bitcoin_value = giveaway;
                    tx_outputs.push((TxOut {
                        value: giveaway,
                        script_pubkey: PubkeyScript::from(descriptor.clone()).into(),
                    }, None));
                    SealEndpoint::with_vout(tx_outputs.len() as u32 - 1, &mut self.rng)
                } else if let message::TransferInfo::Rgb {
                    contract_id: _, receiver: message::RgbReceiver::BlindUtxo(hash)
                } = transfer_info {
                    SealEndpoint::TxOutpoint(hash)
                } else {
                    unimplemented!()
                };
                debug!("RGB endpoint will be {:?}", rgb_endpoint);

                // Get to known how much bitcoins we are spending
                let all_unspent = self.cache
                    .unspent(pay_from)
                    .map_err(Error::from)?;
                let bitcoin_utxos = all_unspent
                    .get(&rgb::ContractId::default())
                    .ok_or(Error::CacheInconsistency)?;
                let outpoints = selected_utxos
                    .iter()
                    .map(Utxo::outpoint)
                    .collect::<BTreeSet<_>>();
                let bitcoin_input_amount = bitcoin_utxos
                    .iter()
                    .filter(|bitcoin_utxo| outpoints.contains(&bitcoin_utxo.outpoint()))
                    .fold(0u64, |sum, utxo| sum + utxo.value);

                // Adding bitcoin change output, if needed
                let mut output_derivation_indexes = set![];
                let (bitcoin_change, change_vout) = if bitcoin_input_amount > bitcoin_value + bitcoin_fee {
                    let change = bitcoin_input_amount - bitcoin_value - bitcoin_fee;
                    let change_index = self.cache
                        .next_unused_derivation(pay_from).map_err(Error::from)?;
                    let change_address = contract.derive_address(change_index, false).ok_or(Error::ServerFailure(Failure {
                        code: 0,
                        info: s!("Unable to derive change address"),
                    }))?.address;
                    self.cache.use_address_derivation(pay_from, change_address.clone(), change_index).map_err(Error::from)?;
                    trace!("Adding change output paying {} to our address {} at derivation index {}", change, change_address, change_index);
                    tx_outputs.push((TxOut {
                        value: change,
                        script_pubkey: change_address.script_pubkey(),
                    }, Some(change_index)));
                    output_derivation_indexes.insert(change_index);
                    (change, Some(tx_outputs.len() as u32 - 1))
                } else {
                    (0, None)
                };

                // Adding RGB change output, if needed
                // NB: Right now, we use really dumb algorithm, allocating 
                //     change to first found outpoint with existing assignment
                //     of the same asset, or, if none, to the bitcoin change
                //     output - failing if neither of them is present. We can
                //     be much smarter, assigning to existing bitcoin utxos,
                //     or creating new output for RGB change
                let mut rgb_change = bmap! {};
                if asset_input_amount > asset_value && transfer_info.is_rgb() {
                    let change = asset_input_amount - asset_value;
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
                trace!("RGB change: {:?}", rgb_change);

                // Constructing bitcoin payment PSBT (for bitcoin payments) or
                // RGB witness PSBT prototype for the commitment (for RGB
                // payments)
                let psbt_inputs = tx_inputs.iter().zip(&selected_utxos).map(|(txin, utxo)| {
                    let mut input = psbt::Input::default();
                    // TODO: cache transactions
                    input.non_witness_utxo = self.electrum.transaction_get(&txin.previous_output.txid).ok();
                    input.bip32_derivation = policy.bip32_derivations(utxo.derivation_index);
                    let script = policy.derive_descriptor(utxo.derivation_index, false).as_ref().map(Descriptor::explicit_script);
                    if policy.is_scripted() {
                        if policy.has_witness() {
                            input.witness_script = script;
                        } else {
                            input.redeem_script = script;
                        }
                    }
                    if let Some((tweak, pubkey)) = utxo.tweak {
                        input.p2c_tweak_add(pubkey, tweak);
                    }
                    input
                }).collect();
                let psbt_outputs = tx_outputs.iter().map(|(txout, index)| {
                    let mut output = psbt::Output::default();
                    if let Some(index) = index {
                        output.proprietary.insert(
                            ProprietaryKey {
                                prefix: rgb::PSBT_PREFIX.to_vec(),
                                subtype: rgb::PSBT_OUT_PUBKEY,
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
                            input: tx_inputs,
                            output: tx_outputs.iter().map(|(txout, _)| txout.clone()).collect(),
                        },
                        version: 0,
                        xpub: none!(),
                        proprietary: none!(),
                        unknown: none!()
                    },
                    inputs: psbt_inputs,
                    outputs: psbt_outputs,
                };
                trace!("Prepared PSBT: {:#?}", psbt);

                // Committing to RGB transfer into the witness transaction and
                // producing consignments (applies to RGB payments only)
                let timestamp = NaiveDateTime::from_timestamp(
                    Utc::now().timestamp(), 0
                );
                let payment_data = if let message::TransferInfo::Rgb { contract_id: asset_id, ref receiver} = transfer_info {
                    let Transfer { consignment, disclosure, witness } = self.rgb20_client.transfer(
                        asset_id,
                        selected_utxos.iter().map(Utxo::outpoint).collect(),
                        bmap! { rgb_endpoint => asset_value },
                        rgb_change.clone(),
                        psbt.clone()
                    ).map_err(Error::from)?;
                    let txid = witness.global.unsigned_tx.txid();
                    for (vout, out) in witness.outputs.iter().enumerate() {
                        let tweak = out.proprietary.get(&ProprietaryKey {
                            prefix: rgb::PSBT_PREFIX.to_vec(),
                            subtype: rgb::PSBT_OUT_TWEAK,
                            key: vec![],
                        })
                            .and_then(Slice32::from_slice);
                        let pubkey = out.proprietary.get(&ProprietaryKey {
                            prefix: rgb::PSBT_PREFIX.to_vec(),
                            subtype: rgb::PSBT_OUT_PUBKEY,
                            key: vec![],
                        })
                            .map(Vec::as_slice)
                            .map(PublicKey::from_slice)
                            .transpose()
                            .ok()
                            .flatten();
                        let derivation_index = tx_outputs[vout].1;
                        if let (Some(pubkey), Some(tweak), Some(derivation_index)) = (pubkey, tweak, derivation_index) {
                            let tweaked_output = TweakedOutput {
                                outpoint: OutPoint::new(txid, vout as u32),
                                script: psbt.global.unsigned_tx.output[vout].script_pubkey.clone(),
                                tweak,
                                pubkey,
                                derivation_index
                            };
                            debug!("Extracted tweak information from witness PSBT: {:?}", tweaked_output);
                            self.storage.add_p2c_tweak(pay_from, tweaked_output).map_err(Error::from)?;
                        }
                    }

                    // Self-enclosing disclosure.
                    self.rgb20_client.enclose(disclosure.clone()).map_err(Error::from)?;

                    // Creation history record
                    let operation = Operation {
                        txid: psbt.global.unsigned_tx.txid(),
                        direction: PaymentDirecton::Outcoming {
                            published: false,
                            asset_change: rgb_change.values().sum(),
                            bitcoin_change,
                            change_outputs: change_vout.into_iter().map(|vout| vout as u16).collect(),
                            giveaway: bitcoin_giveaway,
                            paid_bitcoin_fee: bitcoin_fee,
                            output_derivation_indexes,
                            invoice,
                        },
                        created_at: timestamp,
                        height: 0,
                        asset_id: None,
                        balance_before,
                        bitcoin_volume: bitcoin_input_amount,
                        asset_volume: asset_input_amount,
                        bitcoin_value,
                        asset_value,
                        tx_fee: bitcoin_fee,
                        psbt: PsbtWrapper(psbt.clone()),
                        disclosure: Some(disclosure),
                        notes: None
                    };
                    trace!("Creating operation for the history record: {:#?}", operation);
                    self.storage.register_operation(pay_from, operation).map_err(Error::from)?;

                    trace!("Witness PSBT: {:#?}", psbt);
                    let mut concealed = consignment.clone();
                    concealed.finalize(&bset![ rgb_endpoint ], asset_id);
                    message::PreparedTransfer {
                        psbt: witness,
                        consignments: Some(message::ConsignmentPair {
                            revealed: consignment,
                            concealed
                        })
                    }
                } else {
                    // Creation history record
                    let operation = Operation {
                        txid: psbt.global.unsigned_tx.txid(),
                        direction: PaymentDirecton::Outcoming {
                            published: false,
                            asset_change: bitcoin_change,
                            bitcoin_change,
                            change_outputs: change_vout.into_iter().map(|vout| vout as u16).collect(),
                            giveaway: None,
                            paid_bitcoin_fee: bitcoin_fee,
                            output_derivation_indexes,
                            invoice,
                        },
                        created_at: timestamp,
                        height: 0,
                        asset_id: None,
                        balance_before,
                        bitcoin_volume: bitcoin_input_amount,
                        asset_volume: bitcoin_input_amount,
                        bitcoin_value,
                        asset_value,
                        tx_fee: bitcoin_fee,
                        psbt: PsbtWrapper(psbt.clone()),
                        disclosure: None,
                        notes: None
                    };
                    trace!("Creating operation for the history record: {:#?}", operation);
                    self.storage.register_operation(pay_from, operation).map_err(Error::from)?;

                    // TODO: If any of bitcoin inputs contain some RGB assets
                    //       we must do an "internal transfer"
                    message::PreparedTransfer { psbt, consignments: None }
                };

                Ok(Reply::PreparedPayment(payment_data))
            },

            Request::FinalizeTransfer(mut psbt) => {
                debug!("Finalizing the provided PSBT");
                match miniscript::psbt::finalize(&mut psbt, &wallet::SECP256K1)
                    .and_then(|_| miniscript::psbt::extract(&psbt, &wallet::SECP256K1)) {
                    Ok(tx) => {
                        // TODO: Update saved PSBT
                        trace!("Finalized PSBT: {:#?}", psbt);
                        debug!("Publishing transaction to bitcoin network via Electrum server");
                        trace!("{:#?}", tx);
                        self.electrum
                            .transaction_broadcast(&tx)
                            .map(|_| Reply::Success)
                            .map_err(|err| {
                                error!("Electrum server error: {:?}", err);
                                err
                            })
                            .map_err(Error::from)
                    }
                    Err(err) => {
                        error!("Error finalizing PSBT: {}", err);
                        Ok(Reply::Failure(Failure {
                            code: 0,
                            info: err.to_string()
                        }))
                    }
                }
            }

            Request::AcceptTransfer(consignment) => {
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
                .map(|arg| arg.into_iter().map(|(k, v)| (k, v.into_iter().collect())).collect())
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
