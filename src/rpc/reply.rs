// Keyring: private/public key managing service
// Written in 2021 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the AGPL License
// along with this software.
// If not, see <https://www.gnu.org/licenses/agpl-3.0-standalone.html>.

use std::collections::BTreeMap;

use internet2::presentation;
use microservices::{rpc, rpc_connection};

use crate::model::{Contract, Unspent};
use crate::rpc::message::IdentityInfo;
use crate::Error;

#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", untagged)
)]
#[derive(Clone, Debug, Display, Api)]
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

    #[api(type = 0x0202)]
    #[display("contract_unspent(...)")]
    ContractUnspent(BTreeMap<rgb::ContractId, Vec<Unspent>>),

    #[api(type = 0x0202)]
    #[display("asset({0})")]
    Asset(rgb20::Asset),

    #[api(type = 0x0203)]
    #[display("assets(...)")]
    Assets(Vec<rgb20::Asset>),

    #[api(type = 0x0204)]
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
        rpc::Failure {
            code: 1, // TODO: Create errno types
            info: err.to_string(),
        }
    }
}

impl From<Error> for Reply {
    fn from(err: Error) -> Self {
        Reply::Failure(err.into())
    }
}
