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

use serde_with::{As, DisplayFromStr};
use std::collections::BTreeMap;

use bitcoin::Address;
use internet2::presentation;
use invoice::Invoice;
use lnpbp::seals::OutpointReveal;
use microservices::{rpc, rpc_connection};
use wallet::bip32::UnhardenedIndex;

use crate::model::{AddressDerivation, Contract, Utxo};
use crate::rpc::message::{IdentityInfo, PreparedPayment};
use crate::Error;

#[derive(Serialize, Deserialize, Clone, Debug, Display, Api)]
#[api(encoding = "strict")]
#[non_exhaustive]
pub enum Reply {
    #[api(type = 0x0100)]
    #[display("success()")]
    Success,

    #[api(type = 0x0101)]
    #[display("failure({0})")]
    Failure(microservices::rpc::Failure),

    #[api(type = 0x0200)]
    #[display("contracts(...)")]
    Contracts(Vec<Contract>),

    #[api(type = 0x0201)]
    #[display("contracts(...)")]
    Contract(Contract),

    #[serde(with = "As::<BTreeMap<DisplayFromStr, Vec<DisplayFromStr>>>")]
    #[api(type = 0x0202)]
    #[display("contract_unspent(...)")]
    ContractUnspent(BTreeMap<rgb::ContractId, Vec<Utxo>>),

    #[api(type = 0x0310)]
    #[display("addresses(...)")]
    Addresses(BTreeMap<Address, UnhardenedIndex>),

    #[api(type = 0x0311)]
    #[display("address_derivation({0})")]
    AddressDerivation(AddressDerivation),

    #[api(type = 0x0320)]
    #[display("blind_utxo({0})")]
    BlindUtxo(OutpointReveal),

    #[api(type = 0x0330)]
    #[display("invoices(...)")]
    Invoices(Vec<Invoice>),

    #[api(type = 0x0340)]
    // TODO: Display PSBT once it will support `Display` trait
    #[display(inner)]
    #[serde(skip)]
    PreparedPayment(PreparedPayment),

    #[api(type = 0x0341)]
    #[display("validation({0})")]
    #[serde(skip)]
    Validation(rgb::validation::Status),

    #[api(type = 0x0700)]
    #[display("asset({0})")]
    Asset(rgb20::Asset),

    #[api(type = 0x0701)]
    #[display("assets(...)")]
    Assets(Vec<rgb20::Asset>),

    #[api(type = 0x0500)]
    #[display("identities(...)")]
    Identities(Vec<IdentityInfo>),
}

impl rpc_connection::Reply for Reply {}

impl From<presentation::Error> for Reply {
    fn from(err: presentation::Error) -> Self {
        // TODO: Save error code taken from `Error::to_value()` after
        //       implementation of `ToValue` trait and derive macro for enums
        Reply::Failure(microservices::rpc::Failure {
            code: 0,
            info: format!("{}", err),
        })
    }
}

impl From<Error> for rpc::Failure {
    fn from(err: Error) -> Self {
        match err {
            Error::ServerFailure(failure) => failure,
            err => rpc::Failure {
                code: 1,
                info: err.to_string(),
            },
        }
    }
}

impl From<Error> for Reply {
    fn from(err: Error) -> Self {
        Reply::Failure(err.into())
    }
}

impl Reply {
    pub fn inner_to_json(&self) -> Result<String, serde_json::Error> {
        match self {
            Reply::Success => Ok(s!("")),
            Reply::Failure(err) => {
                Ok(format!(r#"{{"error": "{}"}}"#, err.to_string()))
            }
            Reply::Contracts(data) => serde_json::to_string(data),
            Reply::Contract(data) => serde_json::to_string(data),
            Reply::ContractUnspent(data) => serde_json::to_string(data),
            Reply::Addresses(data) => serde_json::to_string(data),
            Reply::AddressDerivation(data) => serde_json::to_string(data),
            Reply::BlindUtxo(data) => serde_json::to_string(data),
            Reply::Invoices(data) => serde_json::to_string(data),
            Reply::PreparedPayment(data) => Ok(s!("{}")),
            Reply::Validation(data) => serde_json::to_string(data),
            Reply::Asset(data) => serde_json::to_string(data),
            Reply::Assets(data) => serde_json::to_string(data),
            Reply::Identities(data) => serde_json::to_string(data),
        }
    }
}
