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
use std::io;
use std::ops::Range;

use bitcoin::util::bip32::ChildNumber;
use bitcoin::Script;
use internet2::RemoteNodeAddr;
use lnp::ChannelId;
use lnpbp::client_side_validation::{CommitEncode, ConsensusCommit};
use miniscript::{Descriptor, DescriptorTrait, TranslatePk2};
use strict_encoding::{self, StrictDecode, StrictEncode};
use wallet::bip32::{ChildIndex, PubkeyChain, TerminalStep};
use wallet::descriptor::ContractDescriptor;

use super::ContractId;

/// Defines a type of a wallet contract basing on the banking use case,
/// abstracting the underlying technology(ies) into specific contract details
#[derive(
    Clone,
    Ord,
    PartialOrd,
    Eq,
    PartialEq,
    Hash,
    Debug,
    Display,
    Serialize,
    Deserialize,
)]
#[serde(rename = "lowercase")]
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

#[serde_as]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
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
#[non_exhaustive]
#[display(inner)]
pub enum Policy {
    Current(#[serde_as(as = "DisplayFromStr")] ContractDescriptor<PubkeyChain>),

    Instant(ChannelDescriptor),

    Saving(#[serde_as(as = "DisplayFromStr")] ContractDescriptor<PubkeyChain>),
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

    pub fn to_descriptor(&self) -> Descriptor<PubkeyChain> {
        match self {
            Policy::Current(descriptor) => descriptor.to_descriptor(false),
            Policy::Instant(channel) => channel.to_descriptor(),
            Policy::Saving(descriptor) => descriptor.to_descriptor(false),
        }
    }

    pub fn derive_scripts(&self, range: Range<u32>) -> Vec<Script> {
        let d = self.to_descriptor();
        let mut scripts = vec![];
        for index in range {
            scripts.push(
                d.translate_pk2_infallible(|chain| {
                    // TODO: Add convenience PubkeyChain methods
                    let mut path = chain.terminal_path.clone();
                    if path.last() == Some(&TerminalStep::Wildcard) {
                        path.remove(path.len() - 1);
                    }
                    path.push(TerminalStep::Index(index.into()));
                    chain
                        .branch_xpub
                        .derive_pub(
                            &wallet::SECP256K1,
                            &path
                                .into_iter()
                                .filter_map(TerminalStep::index)
                                .map(|index| ChildNumber::Normal { index })
                                .collect::<Vec<_>>(),
                        )
                        .expect("Unhardened derivation can't fail")
                        .public_key
                })
                .script_pubkey(),
            )
        }
        scripts
    }
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
#[display("{channel_id}")]
pub struct ChannelDescriptor {
    channel_id: ChannelId,

    #[serde_as(as = "Vec<DisplayFromStr>")]
    peers: Vec<RemoteNodeAddr>,
}

impl ChannelDescriptor {
    // TODO: Store base points in the channel descriptor and use them to derive
    //       descriptors for all channel transaction outputs to monitor their
    //       onchain status
    pub fn to_descriptor(&self) -> Descriptor<PubkeyChain> {
        unimplemented!()
    }
}
