// MyCitadel: node, wallet library & command-line tool
// Written in 2020 by
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

use std::collections::BTreeMap;
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::io;

use bitcoin::hashes::{sha256, sha256t};
use bitcoin::{BlockHash, Transaction, Txid};
use chrono::NaiveDateTime;
use internet2::RemoteNodeAddr;
use lnp::channel::TxRole;
use lnp::ChannelId;
use lnpbp::client_side_validation::{
    commit_strategy, CommitEncode, CommitEncodeWithStrategy, ConsensusCommit,
};
use lnpbp::commit_verify::CommitVerify;
use lnpbp::strict_encoding::StrictEncode;
use lnpbp::tagged_hash::{self, TaggedHash};
#[cfg(feature = "serde")]
use serde_with::DisplayFromStr;
use wallet::{descriptor, Psbt};

// --- Wallet primitives

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
#[strict_encoding_crate(lnpbp::strict_encoding)]
#[display("{height}:{block_hadh}@{timestamp}")]
pub struct BlockchainTimepair {
    timestamp: NaiveDateTime,
    height: u32,
    block_hash: BlockHash,
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
#[strict_encoding_crate(lnpbp::strict_encoding)]
#[display("{0}:\t{1}\n")]
pub struct TimestampedData<T>(NaiveDateTime, T)
where
    T: Sized + Clone + Eq + Ord + Hash + Debug + Display;

impl<T> TimestampedData<T>
where
    T: Sized + Clone + Eq + Ord + Hash + Debug + Display,
{
    pub fn timestamp(&self) -> NaiveDateTime {
        self.0
    }

    pub fn data(&self) -> &T {
        &self.1
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
#[strict_encoding_crate(lnpbp::strict_encoding)]
#[display("{0}:\t{1}\n")]
pub struct VerifiedTx(Transaction, TxConfirmation);

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
#[strict_encoding_crate(lnpbp::strict_encoding)]
pub enum TxConfirmation {
    #[display("{height}:{block_hash}")]
    Blockchain { height: u32, block_hash: BlockHash },

    #[display("{state_no}@{channel_id}}")]
    Lightning {
        channel_id: ChannelId,
        state_no: u64,
    },
}

// --- Wallet identifiers

/// Tag used for [`NodeId`] and [`ContractId`] hash types
struct WalletIdTag;

impl sha256t::Tag for WalletIdTag {
    #[inline]
    fn engine() -> sha256::HashEngine {
        let midstate = sha256::Midstate::from_inner(
            *tagged_hash::Midstate::with("mycitadel:wallet"),
        );
        sha256::HashEngine::from_midstate(midstate, 64)
    }
}

/// Unique node (genesis, extensions & state transition) identifier equivalent
/// to the commitment hash
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate")
)]
#[derive(
    Wrapper, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, From,
)]
#[wrapper(
    Debug, Display, LowerHex, Index, IndexRange, IndexFrom, IndexTo, IndexFull
)]
pub struct WalletId(sha256t::Hash<WalletIdTag>);

impl<MSG> CommitVerify<MSG> for WalletId
where
    MSG: AsRef<[u8]>,
{
    #[inline]
    fn commit(msg: &MSG) -> WalletId {
        <WalletId as TaggedHash>::hash(msg)
    }
}

// --- Wallet data structure

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
pub struct Wallet {
    id: WalletId,

    #[cfg_attr(feature = "serde", serde(flatten))]
    contract: WalletContract,

    #[cfg_attr(feature = "serde", serde_as("BTreeMap<_, DisplayFromStr>"))]
    drafts: BTreeMap<Txid, Psbt>,

    #[cfg_attr(feature = "serde", serde_as("BTreeMap<_, DisplayFromStr>"))]
    history: BTreeMap<Txid, TimestampedData<Transaction>>,

    #[cfg_attr(feature = "serde", serde_as("DisplayFromStr"))]
    verified_at: BlockchainTimepair,

    #[cfg_attr(feature = "serde", serde_as("BTreeMap<_, DisplayFromStr>"))]
    cache: BTreeMap<Txid, TimestampedData<VerifiedTx>>,
}

impl ConsensusCommit for Wallet {
    type Commitment = WalletId;
}

impl CommitEncode for Wallet {
    fn commit_encode<E: io::Write>(self, e: E) -> usize {
        self.contract
            .strict_encode(e)
            .expect("Memory encoders does not fail")
    }
}

impl Wallet {
    /*
    pub fn id(&self) -> WalletId {
        self.clone().consensus_commit()
    }
     */

    pub fn contract_type(&self) -> ContractType {
        self.contract.contract_type()
    }
}

/// Defines a type of a wallet contract basing on the banking use case,
/// abstracting the underlying technology(ies) into specific contract details
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Display)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", rename = "lowercase")
)]
#[non_exhaustive]
pub enum ContractType {
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
    Debug,
    Display,
    StrictEncode,
    StrictDecode,
)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", rename = "lowercase")
)]
#[strict_encoding_crate(lnpbp::strict_encoding)]
#[non_exhaustive]
pub enum WalletContract {
    #[display("{descriptor}")]
    Current {
        name: String,
        descriptor: descriptor::Generator,
    },

    #[display("{contract}")]
    Instant {
        name: String,
        contract: InstantContract,
    },

    #[display("{contract}")]
    Saving {
        name: String,
        contract: SavingContract,
    },
}

impl WalletContract {
    pub fn contract_type(&self) -> ContractType {
        match self {
            WalletContract::Current { .. } => ContractType::Current,
            WalletContract::Instant { .. } => ContractType::Instant,
            WalletContract::Saving { .. } => ContractType::Saving,
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
    Debug,
    Display,
    StrictEncode,
    StrictDecode,
)]
#[strict_encoding_crate(lnpbp::strict_encoding)]
#[display("{channel_id}")]
pub struct InstantContract {
    channel_id: ChannelId,

    #[serde_as(as = "Vec<DisplayFromStr>")]
    peers: Vec<RemoteNodeAddr>,

    #[cfg_attr(feature = "serde", serde(skip))]
    state: Option<Box<[u8]>>,
}

#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", rename = "lowercase")
)]
#[derive(
    Clone,
    Ord,
    PartialOrd,
    Eq,
    PartialEq,
    Debug,
    Display,
    StrictEncode,
    StrictDecode,
)]
#[strict_encoding_crate(lnpbp::strict_encoding)]
#[non_exhaustive]
pub enum SavingContract {
    #[display(inner)]
    MultiSig(descriptor::MultiSig),

    #[display("covenant")]
    Covenant,
}
