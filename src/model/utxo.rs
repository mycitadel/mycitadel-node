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
use std::str::FromStr;

use bitcoin::{OutPoint, PublicKey, Txid};
use rgb::AtomicValue;
use wallet::bip32::UnhardenedIndex;
use wallet::blockchain::ParseError;
use wallet::{AddressCompat, Slice32};

pub type Allocations =
    BTreeMap<OutPoint, BTreeMap<rgb::ContractId, AtomicValue>>;

#[serde_as]
#[derive(
    Serialize,
    Deserialize,
    Copy,
    Clone,
    Ord,
    PartialOrd,
    Eq,
    PartialEq,
    Hash,
    Debug,
    Display,
    StrictEncode,
    StrictDecode,
)]
#[display("{value}@{height}>{offset}>{txid}:{vout}%{derivation_index}")]
#[repr(C)]
pub struct Utxo {
    /// Amount (in native atomic asset amount) of unspent asset
    pub value: u64,

    /// Height of the block where transaction is mined.
    /// Set to 0 for transactions in the mempool
    pub height: u32,

    /// Offset of the transaction within the block
    pub offset: u16,

    /// Outpoint transaction id
    pub txid: Txid,

    /// Transaction output containing asset
    pub vout: u16,

    /// Index used by the description in deriving script from the transaction
    /// output
    pub derivation_index: UnhardenedIndex,

    /// Tweak (if any) applied to the public key and the index of the public
    /// key which receives tweak
    #[serde_as(as = "Option<(DisplayFromStr, _)>")]
    pub tweak: Option<(Slice32, PublicKey)>,

    /// Address controlling the output
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub address: Option<AddressCompat>,
}

impl Utxo {
    pub fn outpoint(&self) -> OutPoint {
        OutPoint {
            txid: self.txid,
            vout: self.vout as u32,
        }
    }
}

impl FromStr for Utxo {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.split(&['@', '>', '%', '=', ':'][..]);
        match (
            split.next(),
            split.next(),
            split.next(),
            split.next(),
            split.next(),
            split.next(),
            split.next(),
            split.next(),
        ) {
            (
                Some(value),
                Some(height),
                Some(offset),
                Some(txid),
                Some(vout),
                Some(index),
                address,
                None,
            ) => Ok(Utxo {
                value: value.parse()?,
                height: height.parse()?,
                offset: offset.parse()?,
                vout: vout.parse()?,
                txid: txid.parse()?,
                derivation_index: index.parse().map_err(|_| ParseError)?,
                tweak: None,
                address: address
                    .map(AddressCompat::from_str)
                    .transpose()
                    .ok()
                    .flatten(),
            }),
            _ => Err(ParseError),
        }
    }
}
