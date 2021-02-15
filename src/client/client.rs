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

use std::str::FromStr;

use internet2::zmqsocket::{self, ZmqType};
use internet2::{
    session, CreateUnmarshaller, PlainTranscoder, Session, TypedEnum,
    Unmarshall, Unmarshaller,
};
use invoice::{Beneficiary, Invoice};
use lnpbp::chain::AssetId;
use lnpbp::client_side_validation::Conceal;
use rgb::{AtomicValue, Genesis};
use wallet::bip32::{PubkeyChain, UnhardenedIndex};
use wallet::descriptor::OuterCategory;

use super::Config;
use crate::model::ContractId;
use crate::rpc::{message, Reply, Request};
use crate::Error;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub enum InvoiceType {
    AddressUtxo,
    Descriptor,
    Psbt,
}
#[repr(C)]
pub struct Client {
    config: Config,
    session_rpc: session::Raw<PlainTranscoder, zmqsocket::Connection>,
    unmarshaller: Unmarshaller<Reply>,
}

impl Client {
    pub fn with(config: Config) -> Result<Self, Error> {
        debug!("Initializing runtime");
        trace!("Connecting to mycitadel daemon at {}", config.rpc_endpoint);
        let session_rpc = session::Raw::with_zmq_unencrypted(
            ZmqType::Req,
            &config.rpc_endpoint,
            None,
            None,
        )?;
        Ok(Self {
            config,
            session_rpc,
            unmarshaller: Reply::create_unmarshaller(),
        })
    }

    pub fn request(&mut self, request: Request) -> Result<Reply, Error> {
        trace!("Sending request to the server: {:?}", request);
        let data = request.serialize();
        trace!("Raw request data ({} bytes): {:02X?}", data.len(), data);
        self.session_rpc.send_raw_message(&data)?;
        trace!("Awaiting reply");
        let raw = self.session_rpc.recv_raw_message()?;
        trace!("Got reply ({} bytes), parsing: {:02X?}", raw.len(), raw);
        let reply = self.unmarshaller.unmarshall(&raw)?;
        trace!("Reply: {:?}", reply);
        Ok((&*reply).clone())
    }
}

impl Client {
    pub fn contract_list(&mut self) -> Result<Reply, Error> {
        self.request(Request::ListContracts)
    }

    pub fn single_sig_create(
        &mut self,
        name: String,
        pubkey_chain: PubkeyChain,
        category: OuterCategory,
    ) -> Result<Reply, Error> {
        self.request(Request::CreateSingleSig(message::SingleSigInfo {
            name,
            pubkey_chain,
            category,
        }))
    }

    pub fn contract_rename(
        &mut self,
        contract_id: ContractId,
        name: String,
    ) -> Result<Reply, Error> {
        self.request(Request::RenameContract(message::RenameContractRequest {
            contract_id,
            name,
        }))
    }

    pub fn contract_delete(
        &mut self,
        contract_id: ContractId,
    ) -> Result<Reply, Error> {
        self.request(Request::DeleteContract(contract_id))
    }

    pub fn contract_balance(
        &mut self,
        contract_id: ContractId,
        rescan: bool,
        lookup_depth: u8,
    ) -> Result<Reply, Error> {
        if rescan {
            self.request(Request::SyncContract(message::SyncContractRequest {
                contract_id,
                lookup_depth,
            }))
        } else {
            self.request(Request::ContractUnspent(contract_id))
        }
    }

    pub fn address_list(
        &mut self,
        contract_id: ContractId,
        rescan: bool,
        lookup_depth: u8,
    ) -> Result<Reply, Error> {
        if rescan {
            self.request(Request::SyncContract(
                message::SyncContractRequest {
                    contract_id,
                    lookup_depth,
                },
            ))?;
        }
        self.request(Request::UsedAddresses(contract_id))
    }

    pub fn address_create(
        &mut self,
        contract_id: ContractId,
        index: Option<UnhardenedIndex>,
        mark_used: bool,
        legacy: bool,
    ) -> Result<Reply, Error> {
        self.request(Request::NextAddress(message::NextAddressRequest {
            contract_id,
            index,
            legacy,
            mark_used,
        }))
    }

    pub fn invoice_create(
        &mut self,
        category: InvoiceType,
        contract_id: ContractId,
        asset_id: Option<rgb::ContractId>,
        amount: AtomicValue,
        merchant: Option<String>,
        purpose: Option<String>,
        unmark: bool,
        legacy: bool,
    ) -> Result<Invoice, Error> {
        let (beneficiary, reveal_data) = match (category, asset_id) {
            (InvoiceType::AddressUtxo, Some(asset_id)) => {
                let seal =
                    match self.request(Request::BlindUtxo(contract_id))? {
                        Reply::BlindUtxo(seal) => seal,
                        _ => Err(Error::UnexpectedApi)?,
                    };
                (Beneficiary::BlindUtxo(seal.conceal()), Some(seal))
            }
            (InvoiceType::AddressUtxo, None) => {
                let address = match self.request(Request::NextAddress(
                    message::NextAddressRequest {
                        contract_id,
                        index: None,
                        legacy,
                        mark_used: !unmark,
                    },
                ))? {
                    Reply::AddressDerivation(ad) => ad.address,
                    _ => Err(Error::UnexpectedApi)?,
                };
                (Beneficiary::Address(address), None)
            }
            _ => unimplemented!(),
        };
        let inv = Invoice {
            amount: invoice::AmountExt::Normal(amount),
            beneficiaries: vec![beneficiary],
            asset: asset_id.map(|id| AssetId::from(id)),
            merchant,
            purpose,
            ..Invoice::default()
        };
        self.request(Request::AddInvoice(message::AddInvoiceRequest {
            invoice: inv.clone(),
            source_info: bmap! { contract_id => reveal_data },
        }))?;
        Ok(inv)
    }

    pub fn invoice_list(
        &mut self,
        contract_id: ContractId,
    ) -> Result<Reply, Error> {
        self.request(Request::ListInvoices(contract_id))
    }

    pub fn asset_list(&mut self) -> Result<Reply, Error> {
        self.request(Request::ListAssets)
    }

    pub fn asset_import(
        &mut self,
        genesis_bech: String,
    ) -> Result<Reply, Error> {
        let genesis = Genesis::from_str(&genesis_bech).map_err(|err| {
            error!("Wrong genesis data: {}", err);
            Error::EmbeddedNodeInitError
        })?;
        self.request(Request::ImportAsset(genesis))
    }
}
