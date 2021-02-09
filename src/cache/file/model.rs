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

use chrono::NaiveDateTime;
use serde_with::DisplayFromStr;
use std::collections::BTreeMap;

use bitcoin::{Address, BlockHash, Txid};
use wallet::bip32::UnhardenedIndex;

use crate::model::{ContractId, Unspent};

#[serde_as]
#[derive(
    Serialize,
    Deserialize,
    Getters,
    Clone,
    PartialEq,
    Debug,
    Default,
    StrictEncode,
    StrictDecode,
)]
pub(super) struct Cache {
    pub known_height: u32,

    pub descriptors: BTreeMap<ContractId, ContractCache>,

    #[serde_as(as = "Vec<(DisplayFromStr, _)>")]
    pub block_info: Vec<(BlockHash, NaiveDateTime)>,

    #[serde_as(as = "BTreeMap<DisplayFromStr, (_, _)>")]
    /// Mapping transaction id to the block height and block offset
    pub mine_info: BTreeMap<Txid, (u32, u16)>,
}

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
pub(super) struct ContractCache {
    pub updated_height: u32,

    pub addresses: BTreeMap<Address, UnhardenedIndex>,

    pub used: BTreeMap<UnhardenedIndex, Address>,

    #[serde_as(as = "BTreeMap<DisplayFromStr, Vec<DisplayFromStr>>")]
    pub unspent: BTreeMap<rgb::ContractId, Vec<Unspent>>,
}
