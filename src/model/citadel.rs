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

use serde_with::DisplayFromStr;
use std::collections::BTreeMap;

use crate::model::{Contract, ContractId};
use crate::rpc::message::IdentityInfo;

#[serde_as]
#[derive(
    Serialize,
    Deserialize,
    Clone,
    PartialEq,
    Debug,
    Default,
    StrictEncode,
    StrictDecode,
)]
pub struct Citadel {
    #[serde_as(as = "BTreeMap<DisplayFromStr, _>")]
    pub contracts: BTreeMap<ContractId, Contract>,

    #[serde_as(as = "BTreeMap<DisplayFromStr, _>")]
    pub identities: BTreeMap<rgb::ContractId, IdentityInfo>,

    #[serde_as(as = "BTreeMap<DisplayFromStr, _>")]
    pub assets: BTreeMap<rgb::ContractId, rgb20::Asset>,
}
