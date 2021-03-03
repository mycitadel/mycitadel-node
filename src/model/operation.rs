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
pub enum PaymentDirecton {
    Incoming {
        giveaway: Option<u64>,
        #[serde_as(as = "HashSet<DisplayFromStr>")]
        input_derivation_indexes: HashSet<UnhardenedIndex>,
    },

    Outcoming {
        published: bool,
        asset_change: u64,
        bitcoin_change: u64,
        change_outputs: HashSet<u16>,
        giveaway: Option<u64>,
        amount: u64,
        paid_bitcoin_fee: u64,
        #[serde_as(as = "HashSet<DisplayFromStr>")]
        output_derivation_indexes: HashSet<UnhardenedIndex>,
        invoice: Invoice,
    },
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
    pub tx_fee: u64,

    pub psbt: Psbt, // Even if we have only tx data, we wrap them in PSBT
    pub consignment: Option<Consignment>,

    #[serde_as(as = "Option<_>")]
    pub notes: Option<String>,
}

impl Operation {}
