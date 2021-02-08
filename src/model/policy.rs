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

#[cfg(feature = "serde")]
use serde_with::{As, DisplayFromStr};
use std::io;

use internet2::RemoteNodeAddr;
use lnp::ChannelId;
use lnpbp::client_side_validation::{CommitEncode, ConsensusCommit};
use strict_encoding::{self, StrictDecode, StrictEncode};
use wallet::bip32::PubkeyChain;
use wallet::descriptor::ContractDescriptor;

use super::ContractId;

/// Defines a type of a wallet contract basing on the banking use case,
/// abstracting the underlying technology(ies) into specific contract details
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Display)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", rename = "lowercase")
)]
#[non_exhaustive]
pub enum PolicyType {
    /// Accounts that allow spending with a simple procedure (like single
    /// signature). However the actual transfer may take some time (like mining
    /// onchain transaction). Analogous to "paying with gold coins" or
    /// "doing a SWFIT/SEPA transfer". May require use of hardware wallet
    /// devices
    #[display("current")]
    Current,

    /// Instant payment accounts allowing simple & fasm payments with strict
    /// limits. Must not require any hardware security device for processing.
    /// The main technology is the Lightning network, with different forms
    /// of fast payment channels on top of it (currently only BOLT-3-based).
    /// Analogous to credit cards payments and instant payment systems
    /// (PayPal, QIWI etc).
    #[display("instant")]
    Instant,

    /// Accounts with complex spending processes, requiring hardware devices,
    /// multiple signatures, timelocks and other forms of limitations.
    #[display("saving")]
    Saving,

    /// Future forms of smart-contracts for borrowing money and assets. Will
    /// probably require some advanced smart contract technology, like
    /// new forms of scriptless scripts and/or RGB schemata + simplicity
    /// scripting.
    #[display("loan")]
    Loan,

    /// May also be used for providing funds to liquidity pools etc.
    #[display("staking")]
    Staking,

    #[display("trading")]
    Trading,

    #[display("storage")]
    Storage,

    #[display("computing")]
    Computing,
}

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
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", rename_all = "lowercase")
)]
#[non_exhaustive]
#[display(inner)]
pub enum Policy {
    Current(
        #[cfg_attr(feature = "serde", serde(with = "As::<DisplayFromStr>"))]
        ContractDescriptor<PubkeyChain>,
    ),

    Instant(ChannelDescriptor),

    Saving(
        #[cfg_attr(feature = "serde", serde(with = "As::<DisplayFromStr>"))]
        ContractDescriptor<PubkeyChain>,
    ),
}

impl ConsensusCommit for Policy {
    type Commitment = ContractId;
}

impl CommitEncode for Policy {
    fn commit_encode<E: io::Write>(self, e: E) -> usize {
        self.strict_encode(e)
            .expect("Memory encoders does not fail")
    }
}

impl Policy {
    pub fn id(&self) -> ContractId {
        self.clone().consensus_commit()
    }

    pub fn policy_type(&self) -> PolicyType {
        match self {
            Policy::Current { .. } => PolicyType::Current,
            Policy::Instant { .. } => PolicyType::Instant,
            Policy::Saving { .. } => PolicyType::Saving,
        }
    }
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
#[display("{channel_id}")]
pub struct ChannelDescriptor {
    channel_id: ChannelId,

    #[cfg_attr(feature = "serde", serde(with = "As::<Vec<DisplayFromStr>>"))]
    peers: Vec<RemoteNodeAddr>,
}
