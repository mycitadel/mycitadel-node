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
use std::str::FromStr;

use bitcoin::Txid;
use wallet::bip32::UnhardenedIndex;
use wallet::blockchain::ParseError;
use wallet::TimeHeight;

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

#[cfg_attr(
    feature = "serde",
    serde_as,
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate")
)]
#[derive(
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
    #[cfg_attr(feature = "serde", serde(with = "As::<DisplayFromStr>"))]
    paid: TimeHeight,

    #[cfg_attr(feature = "serde", serde(with = "As::<DisplayFromStr>"))]
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

#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate")
)]
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
pub enum PaymentDirecton {
    #[display("+", alt = "->")]
    Incoming,

    #[display("-", alt = "<-")]
    Outcoming,
}

#[cfg_attr(
    feature = "serde",
    serde_as,
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate")
)]
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
#[display(
    "{direction:#} {txid}:{vout} {direction}{value} @ {mined_at}: {details}\n"
)]
pub struct Operation {
    pub direction: PaymentDirecton,

    #[cfg_attr(
        feature = "serde",
        serde(with = "As::<chrono::DateTime<chrono::Utc>>")
    )]
    pub created_at: NaiveDateTime,

    #[cfg_attr(feature = "serde", serde(with = "As::<DisplayFromStr>"))]
    pub mined_at: TimeHeight,

    #[cfg_attr(feature = "serde", serde(with = "As::<DisplayFromStr>"))]
    pub txid: Txid,

    pub vout: u16,

    #[cfg_attr(feature = "serde", serde(with = "As::<DisplayFromStr>"))]
    pub value: bitcoin::Amount,

    pub invoice: String,

    pub details: String,
}

#[cfg_attr(
    feature = "serde",
    serde_as,
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate")
)]
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
#[repr(C)]
#[display("{value}@{height}>{offset}>{vout}%{index}")]
pub struct Unspent {
    /// Amount (in native atomic asset amount) of unspent asset
    pub value: u64,

    /// Height of the block where transaction is mined.
    /// Set to 0 for transactions in the mempool
    pub height: u32,

    /// Offset of the transaction within the block
    pub offset: u16,

    /// Transaction output containing asset
    pub vout: u16,

    /// Index used by the description in deriving script from the transaction
    /// output
    pub index: UnhardenedIndex,
}

impl FromStr for Unspent {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.split(&['@', '>', '%'][..]);
        match (
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
                Some(vout),
                Some(index),
                None,
            ) => Ok(Unspent {
                value: value.parse()?,
                height: height.parse()?,
                offset: offset.parse()?,
                vout: vout.parse()?,
                index: index.parse().map_err(|_| ParseError)?,
            }),
            _ => Err(ParseError),
        }
    }
}
