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
#[cfg(feature = "serde")]
use serde_with::{As, DisplayFromStr};
use std::collections::BTreeMap;

use bitcoin::{Address, BlockHash, Txid};
use wallet::bip32::UnhardenedIndex;

use crate::model::{ContractId, Unspent};

#[cfg_attr(
    feature = "serde",
    serde_as,
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate")
)]
#[derive(
    Getters, Clone, PartialEq, Debug, Default, StrictEncode, StrictDecode,
)]
pub(super) struct Cache {
    #[cfg_attr(feature = "serde", serde(with = "As::<DisplayFromStr>"))]
    pub known_height: usize,

    pub descriptors: BTreeMap<ContractId, ContractCache>,

    #[cfg_attr(
        feature = "serde",
        serde(with = "As::<Vec<(DisplayFromStr, DisplayFromStr)>>")
    )]
    pub block_info: Vec<(BlockHash, NaiveDateTime)>,

    #[cfg_attr(
        feature = "serde",
        serde(
            with = "As::<BTreeMap<DisplayFromStr, (DisplayFromStr, DisplayFromStr)>>"
        )
    )]
    /// Mapping transaction id to the block height and block offset
    pub mine_info: BTreeMap<Txid, (usize, usize)>,
}

#[cfg_attr(
    feature = "serde",
    serde_as,
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate")
)]
#[derive(Clone, PartialEq, Debug, StrictEncode, StrictDecode)]
pub(super) struct ContractCache {
    pub addresses: BTreeMap<Address, UnhardenedIndex>,

    pub used: BTreeMap<UnhardenedIndex, Address>,

    #[cfg_attr(feature = "serde", serde(with = "As::<Vec<DisplayFromStr>>"))]
    pub unspent: Vec<Unspent>,
}
