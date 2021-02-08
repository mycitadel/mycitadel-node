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
