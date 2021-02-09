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
use rgb::Genesis;
use wallet::bip32::PubkeyChain;
use wallet::descriptor::OuterCategory;

use super::Config;
use crate::model::ContractId;
use crate::rpc::{message, Reply, Request};
use crate::Error;

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
        trace!("Raw request data ({} bytes): {:?}", data.len(), data);
        self.session_rpc.send_raw_message(&data)?;
        trace!("Awaiting reply");
        let raw = self.session_rpc.recv_raw_message()?;
        trace!("Got reply ({} bytes), parsing", raw.len());
        let reply = self.unmarshaller.unmarshall(&raw)?;
        trace!("Reply: {:?}", reply);
        Ok((&*reply).clone())
    }
}

impl Client {
    pub fn contract_list(&mut self) -> Result<Reply, Error> {
        self.request(Request::ListContracts)
    }

    pub fn create_single_sig(
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
