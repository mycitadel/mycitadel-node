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
use std::str::FromStr;

use bitcoin::{OutPoint, Txid};
use wallet::bip32::UnhardenedIndex;
use wallet::blockchain::ParseError;
use wallet::{AddressPayload, Slice32, TimeHeight};

#[derive(
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
#[display(inner)]
pub enum PaymentConfirmation {
    Txid(Txid),
}

impl FromStr for PaymentConfirmation {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(PaymentConfirmation::Txid(s.parse()?))
    }
}

#[serde_as]
#[derive(
    Serialize,
    Deserialize,
    Getters,
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
#[display("{confirmation}@{paid}")]
pub struct PaymentSlip {
    #[serde_as(as = "DisplayFromStr")]
    paid: TimeHeight,

    #[serde_as(as = "DisplayFromStr")]
    confirmation: PaymentConfirmation,
}

impl FromStr for PaymentSlip {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut data = s.split(&[':', '@'][..]);
        let me = Self {
            paid: data.next().ok_or(ParseError)?.parse()?,
            confirmation: data.next().ok_or(ParseError)?.parse()?,
        };
        if data.next().is_some() {
            Err(ParseError)
        } else {
            Ok(me)
        }
    }
}

#[derive(
    Serialize,
    Deserialize,
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
pub enum PaymentDirecton {
    #[display("+", alt = "->")]
    Incoming,

    #[display("-", alt = "<-")]
    Outcoming,
}

#[serde_as]
#[derive(
    Serialize,
    Deserialize,
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
#[display(
    "{direction:#} {txid}:{vout} {direction}{value} @ {mined_at}: {details}\n"
)]
pub struct Operation {
    pub direction: PaymentDirecton,

    #[serde_as(as = "chrono::DateTime<chrono::Utc>>")]
    pub created_at: NaiveDateTime,

    #[serde_as(as = "DisplayFromStr")]
    pub mined_at: TimeHeight,

    #[serde_as(as = "DisplayFromStr")]
    pub txid: Txid,

    pub vout: u16,

    #[serde_as(as = "DisplayFromStr")]
    pub value: bitcoin::Amount,

    pub invoice: String,

    pub details: String,
}

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
    pub tweak: Option<(Slice32, u16)>,

    /// Address controlling the output
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub address: Option<AddressPayload>,
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
                    .map(AddressPayload::from_str)
                    .transpose()
                    .ok()
                    .flatten(),
            }),
            _ => Err(ParseError),
        }
    }
}
