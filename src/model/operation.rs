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
use std::collections::HashSet;
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

use bitcoin::consensus::{deserialize, serialize};
use bitcoin::Txid;
use invoice::Invoice;
use rgb::Consignment;
use wallet::bip32::UnhardenedIndex;
use wallet::Psbt;

#[serde_as]
#[derive(
    Serialize,
    Deserialize,
    Clone,
    Eq,
    PartialEq,
    Debug,
    StrictEncode,
    StrictDecode,
)]
#[serde(rename_all = "lowercase")]
pub enum PaymentDirecton {
    Incoming {
        giveaway: Option<u64>,
        #[serde_as(as = "HashSet<_>")]
        input_derivation_indexes: HashSet<UnhardenedIndex>,
    },

    Outcoming {
        published: bool,
        asset_change: u64,
        bitcoin_change: u64,
        change_outputs: HashSet<u16>,
        giveaway: Option<u64>,
        paid_bitcoin_fee: u64,
        #[serde_as(as = "HashSet<_>")]
        output_derivation_indexes: HashSet<UnhardenedIndex>,
        #[serde_as(as = "DisplayFromStr")]
        invoice: Invoice,
    },
}

#[derive(Debug, Clone, PartialEq, StrictEncode, StrictDecode)]
pub struct PsbtWrapper(pub Psbt);

impl Display for PsbtWrapper {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&base64::encode(serialize(&self.0)))
    }
}

impl FromStr for PsbtWrapper {
    type Err = bitcoin::consensus::encode::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(deserialize(
            &base64::decode(&s).map_err(|_| Self::Err::NonMinimalVarInt)?,
        )?))
    }
}

#[serde_as]
#[derive(
    Serialize, Deserialize, Clone, PartialEq, Debug, StrictEncode, StrictDecode,
)]
pub struct Operation {
    pub direction: PaymentDirecton,
    #[serde_as(as = "chrono::DateTime<chrono::Utc>>")]
    pub created_at: NaiveDateTime,

    pub height: i64,

    #[serde_as(as = "Option<DisplayFromStr>")]
    pub asset_id: Option<rgb::ContractId>,
    pub balance_before: u64,
    pub bitcoin_volume: u64,
    pub asset_volume: u64,
    pub bitcoin_value: u64,
    pub asset_value: u64,
    pub tx_fee: u64,

    pub txid: Txid, // White this can be retrieved from PSBT,
    // we still cache them since we extensively use them as IDs
    #[serde_as(as = "DisplayFromStr")]
    pub psbt: PsbtWrapper, /* Even if we have only tx data, we wrap them in
                            * PSBT */
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub consignment: Option<Consignment>,

    #[serde_as(as = "Option<_>")]
    pub notes: Option<String>,
}

impl Operation {}
