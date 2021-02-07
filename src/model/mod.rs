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

mod contract;
mod operation;
mod policy;
mod state;
mod util;

pub use contract::*;
pub use operation::*;
pub use policy::*;
pub use state::*;
pub use util::*;

// -----------------------------------------------------------------------------

use std::collections::BTreeMap;

use crate::rpc::message::IdentityInfo;

#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate")
)]
#[derive(Clone, PartialEq, Debug, Default, StrictEncode, StrictDecode)]
pub struct Wallet {
    pub contracts: BTreeMap<ContractId, Contract>,
    pub identities: BTreeMap<rgb::ContractId, IdentityInfo>,
    pub assets: BTreeMap<rgb::ContractId, rgb20::Asset>,
}
