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
use std::io;
use std::ops::RangeInclusive;

use bitcoin::Address;
use invoice::Invoice;
use lnpbp::seals::{OutpointHash, OutpointReveal};
use lnpbp::strict_encoding::{self, StrictDecode, StrictEncode};
use rgb::Consignment;
use wallet::bip32::{PubkeyChain, UnhardenedIndex};
use wallet::{descriptor, Psbt};

use crate::model;

#[serde_as]
#[derive(
    Serialize,
    Deserialize,
    Clone,
    Eq,
    PartialEq,
    PartialOrd,
    Ord,
    Hash,
    Debug,
    Display,
    StrictEncode,
    StrictDecode,
)]
#[display("create_single_sig({category}({pubkey_chain}), \"{name}\")")]
pub struct SingleSigInfo {
    pub name: String,
    #[serde_as(as = "DisplayFromStr")]
    pub pubkey_chain: PubkeyChain,
    #[serde_as(as = "DisplayFromStr")]
    pub category: descriptor::OuterCategory,
}

#[derive(
    Serialize,
    Deserialize,
    Clone,
    Eq,
    PartialEq,
    PartialOrd,
    Ord,
    Hash,
    Debug,
    Display,
    StrictEncode,
    StrictDecode,
)]
#[display("sync_contract({contract_id}, depth: {lookup_depth})")]
pub struct SyncContractRequest {
    pub contract_id: model::ContractId,
    pub lookup_depth: u8,
}

#[serde_as]
#[derive(
    Serialize,
    Deserialize,
    Clone,
    Eq,
    PartialEq,
    PartialOrd,
    Ord,
    Hash,
    Debug,
    Display,
    StrictEncode,
    StrictDecode,
)]
#[display("rename_contract({contract_id}, \"{name}\")")]
pub struct RenameContractRequest {
    #[serde_as(as = "DisplayFromStr")]
    pub contract_id: model::ContractId,
    pub name: String,
}

#[serde_as]
#[derive(
    Serialize,
    Deserialize,
    Clone,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    Debug,
    Display,
    StrictEncode,
    StrictDecode,
)]
#[display(
    "next_address({contract_id}, legacy: {legacy}, mark_used: {mark_used})"
)]
pub struct NextAddressRequest {
    #[serde_as(as = "DisplayFromStr")]
    pub contract_id: model::ContractId,
    pub index: Option<UnhardenedIndex>,
    pub legacy: bool,
    pub mark_used: bool,
}

#[derive(
    Serialize,
    Deserialize,
    Clone,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    Debug,
    Display,
    StrictEncode,
    StrictDecode,
)]
#[display("{contract_id}.{address}")]
pub struct ContractAddressTuple {
    pub contract_id: model::ContractId,
    pub address: Address,
}

impl ContractAddressTuple {
    pub fn new(
        contract_id: model::ContractId,
        address: Address,
    ) -> ContractAddressTuple {
        ContractAddressTuple {
            contract_id,
            address,
        }
    }
}

#[derive(
    Serialize,
    Deserialize,
    Clone,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    Debug,
    Display,
    StrictEncode,
    StrictDecode,
)]
#[display("add_invoice({invoice}, ...)")]
pub struct AddInvoiceRequest {
    pub invoice: Invoice,
    pub source_info: BTreeMap<model::ContractId, Option<OutpointReveal>>,
}

#[serde_as]
#[derive(
    Serialize, Deserialize, Clone, PartialEq, Debug, StrictEncode, StrictDecode,
)]
pub enum RgbReceiver {
    BlindUtxo(OutpointHash),
    Descriptor {
        #[serde_as(as = "DisplayFromStr")]
        descriptor: descriptor::Compact,
        /// Amount of statoshis to give away with the descriptor-based payment
        giveaway: u64,
    },
    Psbt(Psbt),
}

#[serde_as]
#[derive(
    Serialize, Deserialize, Clone, PartialEq, Debug, StrictEncode, StrictDecode,
)]
#[serde(tag = "type")]
pub enum TransferInfo {
    Bitcoin(#[serde_as(as = "DisplayFromStr")] descriptor::Compact),

    Rgb {
        contract_id: rgb::ContractId,
        receiver: RgbReceiver,
    },
}

impl TransferInfo {
    pub fn contract_id(&self) -> rgb::ContractId {
        match self {
            TransferInfo::Bitcoin(_) => rgb::ContractId::default(),
            TransferInfo::Rgb { contract_id, .. } => *contract_id,
        }
    }

    pub fn bitcoin_descriptor(&self) -> Option<descriptor::Compact> {
        match self {
            TransferInfo::Bitcoin(descr) => Some(descr.clone()),
            TransferInfo::Rgb { .. } => None,
        }
    }

    pub fn rgb_descriptor(&self) -> Option<descriptor::Compact> {
        match self {
            TransferInfo::Bitcoin(_) => None,
            TransferInfo::Rgb {
                receiver: RgbReceiver::Descriptor { descriptor, .. },
                ..
            } => Some(descriptor.clone()),
            _ => None,
        }
    }

    pub fn is_rgb(&self) -> bool {
        match self {
            TransferInfo::Bitcoin(_) => false,
            TransferInfo::Rgb { .. } => true,
        }
    }
}

#[derive(
    Serialize,
    Deserialize,
    Clone,
    PartialEq,
    Debug,
    Display,
    StrictEncode,
    StrictDecode,
)]
#[display(
    "compose_payment(from: {pay_from}, amount: {amount}, fee: {bitcoin_fee}, ...)"
)]
pub struct ComposeTransferRequest {
    pub pay_from: model::ContractId,
    pub bitcoin_fee: u64,
    pub amount: u64,
    pub transfer_info: TransferInfo,
}

#[derive(Clone, PartialEq, Debug, Display, StrictEncode, StrictDecode)]
#[display("prepared_transfer(...)")]
pub struct PreparedTransfer {
    pub psbt: Psbt,
    pub consignments: Option<ConsignmentPair>,
}

#[derive(Clone, PartialEq, Debug, StrictEncode, StrictDecode)]
pub struct ConsignmentPair {
    pub revealed: Consignment,
    pub concealed: Consignment,
}

#[serde_as]
#[derive(
    Serialize, Deserialize, Clone, Eq, PartialEq, Hash, Debug, Display,
)]
#[display("{key}")]
pub struct SignerAccountInfo {
    pub title: String,
    #[serde_as(as = "DisplayFromStr")]
    pub key: descriptor::SingleSig,
    pub used: Vec<RangeInclusive<u32>>,
}

impl StrictEncode for SignerAccountInfo {
    fn strict_encode<E: io::Write>(
        &self,
        mut e: E,
    ) -> Result<usize, strict_encoding::Error> {
        let len = strict_encode_list!(e; self.title, self.key);
        let ranges = self
            .used
            .iter()
            .map(|range| (*range.start(), *range.end()))
            .collect::<Vec<_>>();
        Ok(len + ranges.strict_encode(&mut e)?)
    }
}

impl StrictDecode for SignerAccountInfo {
    fn strict_decode<D: io::Read>(
        mut d: D,
    ) -> Result<Self, strict_encoding::Error> {
        Ok(Self {
            title: StrictDecode::strict_decode(&mut d)?,
            key: StrictDecode::strict_decode(&mut d)?,
            used: Vec::<(u32, u32)>::strict_decode(&mut d)?
                .into_iter()
                .map(|(start, end)| RangeInclusive::new(start, end))
                .collect(),
        })
    }
}

#[serde_as]
#[derive(
    Serialize, Deserialize, Clone, Eq, PartialEq, Hash, Debug, Display,
)]
#[display("{key}")]
pub struct IdentityInfo {
    pub name: String,
    #[serde_as(as = "DisplayFromStr")]
    pub key: descriptor::SingleSig,
    pub known: Vec<RangeInclusive<u32>>,
}

impl StrictEncode for IdentityInfo {
    fn strict_encode<E: io::Write>(
        &self,
        mut e: E,
    ) -> Result<usize, strict_encoding::Error> {
        let len = strict_encode_list!(e; self.name, self.key);
        let ranges = self
            .known
            .iter()
            .map(|range| (*range.start(), *range.end()))
            .collect::<Vec<_>>();
        Ok(len + ranges.strict_encode(&mut e)?)
    }
}

impl StrictDecode for IdentityInfo {
    fn strict_decode<D: io::Read>(
        mut d: D,
    ) -> Result<Self, strict_encoding::Error> {
        Ok(Self {
            name: StrictDecode::strict_decode(&mut d)?,
            key: StrictDecode::strict_decode(&mut d)?,
            known: Vec::<(u32, u32)>::strict_decode(&mut d)?
                .into_iter()
                .map(|(start, end)| RangeInclusive::new(start, end))
                .collect(),
        })
    }
}
