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
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::io;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

use bitcoin::hashes::{sha256, sha256t};
use bitcoin::{BlockHash, OutPoint, Txid};
use internet2::RemoteNodeAddr;
use lnp::ChannelId;
use lnpbp::bech32::ToBech32IdString;
use lnpbp::client_side_validation::{CommitEncode, ConsensusCommit};
use lnpbp::commit_verify::CommitVerify;
use lnpbp::strict_encoding::StrictEncode;
use lnpbp::tagged_hash::{self, TaggedHash};
use wallet::{descriptor, Psbt};

// --- Wallet primitives

/// Error parsing string representation of wallet data/structure
#[derive(
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Debug,
    Display,
    From,
    Error,
)]
#[display(doc_comments)]
#[from(bitcoin::hashes::hex::Error)]
#[from(chrono::ParseError)]
#[from(std::num::ParseIntError)]
pub struct FromStrError;

// TODO: Consider moving to descriptor wallet lib, BPro lib, LNP/BP Core lib
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
#[display("{block_height}:{block_hash}@{timestamp}")]
pub struct BlockchainTimepair {
    timestamp: NaiveDateTime,
    block_height: u32,
    block_hash: BlockHash,
}

impl FromStr for BlockchainTimepair {
    type Err = FromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut data = s.split(&[':', '@'][..]);
        let me = Self {
            timestamp: data.next().ok_or(FromStrError)?.parse()?,
            block_height: data.next().ok_or(FromStrError)?.parse()?,
            block_hash: data.next().ok_or(FromStrError)?.parse()?,
        };
        if data.next().is_some() {
            Err(FromStrError)
        } else {
            Ok(me)
        }
    }
}

// TODO: Consider moving to descriptor wallet lib, BPro lib, LNP/BP Core lib
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
    #[display("{block_height}:{block_hash}")]
    Blockchain {
        block_height: u32,
        block_hash: BlockHash,
    },

    #[display("{state_no}@{channel_id}")]
    Lightning {
        channel_id: ChannelId,
        state_no: u64,
    },
}

impl FromStr for TxConfirmation {
    type Err = FromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut data = s.split(&[':', '@'][..]);
        let me = if s.contains(':') {
            TxConfirmation::Blockchain {
                block_height: data.next().ok_or(FromStrError)?.parse()?,
                block_hash: data.next().ok_or(FromStrError)?.parse()?,
            }
        } else {
            TxConfirmation::Lightning {
                channel_id: data.next().ok_or(FromStrError)?.parse()?,
                state_no: data.next().ok_or(FromStrError)?.parse()?,
            }
        };
        if data.next().is_some() {
            Err(FromStrError)
        } else {
            Ok(me)
        }
    }
}

// --- Wallet identifiers

pub struct WalletIdTag;

impl sha256t::Tag for WalletIdTag {
    #[inline]
    fn engine() -> sha256::HashEngine {
        let midstate = sha256::Midstate::from_inner(
            **tagged_hash::Midstate::with("mycitadel:wallet"),
        );
        sha256::HashEngine::from_midstate(midstate, 64)
    }
}

#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate")
)]
#[derive(
    Wrapper,
    Copy,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Display,
    Default,
    From,
    StrictEncode,
    StrictDecode,
)]
#[wrapper(
    Debug, FromStr, LowerHex, Index, IndexRange, IndexFrom, IndexTo, IndexFull
)]
#[display(WalletId::to_bech32_id_string)]
pub struct WalletId(sha256t::Hash<WalletIdTag>);

impl<MSG> CommitVerify<MSG> for WalletId
where
    MSG: AsRef<[u8]>,
{
    #[inline]
    fn commit(msg: &MSG) -> WalletId {
        <WalletId as TaggedHash<_>>::hash(msg)
    }
}

// --- Payment slip

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
#[display(inner)]
pub enum PaymentConfirmation {
    Txid(Txid),
}

impl FromStr for PaymentConfirmation {
    type Err = FromStrError;

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
#[strict_encoding_crate(lnpbp::strict_encoding)]
#[display("{confirmation}@{paid}")]
pub struct PaymentSlip {
    #[cfg_attr(feature = "serde", serde(with = "As::<DisplayFromStr>"))]
    paid: BlockchainTimepair,

    #[cfg_attr(feature = "serde", serde(with = "As::<DisplayFromStr>"))]
    confirmation: PaymentConfirmation,
}

impl FromStr for PaymentSlip {
    type Err = FromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut data = s.split(&[':', '@'][..]);
        let me = Self {
            paid: data.next().ok_or(FromStrError)?.parse()?,
            confirmation: data.next().ok_or(FromStrError)?.parse()?,
        };
        if data.next().is_some() {
            Err(FromStrError)
        } else {
            Ok(me)
        }
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
    Getters, Clone, PartialEq, Debug, Display, StrictEncode, StrictDecode,
)]
#[strict_encoding_crate(lnpbp::strict_encoding)]
#[display("{id}:{contract}")]
pub struct Wallet {
    /// Unique wallet id used to identify wallet accross different application
    /// instances. Created as a taproot-style bitcoin tagged hash out of
    /// strict-encoded wallet contract data: when contract changes walled id
    /// changes; if two wallets on different devices have the same underlying
    /// contract they will have the same id.
    ///
    /// The id is kept pre-computed: the wallet contract can't be changed after
    /// the creation, so there is no need to perform expensive commitment
    /// process each time we need wallet id
    #[cfg_attr(feature = "serde", serde(with = "As::<DisplayFromStr>"))]
    id: WalletId,

    #[cfg_attr(feature = "serde", serde(flatten))]
    contract: WalletContract,

    #[cfg_attr(
        feature = "serde",
        serde(with = "As::<chrono::DateTime<chrono::Utc>>")
    )]
    created_at: NaiveDateTime,

    #[cfg_attr(
        feature = "serde",
        serde(with = "As::<Option<DisplayFromStr>>")
    )]
    checked_at: Option<BlockchainTimepair>,

    #[cfg_attr(
        feature = "serde",
        serde(with = "As::<Vec<(DisplayFromStr, DisplayFromStr)>>")
    )]
    blinding_factors: BTreeMap<OutPoint, u64>,

    #[cfg_attr(feature = "serde", serde(with = "As::<Vec<DisplayFromStr>>"))]
    sent_invoices: Vec<String>,

    #[cfg_attr(feature = "serde", serde(with = "As::<Vec<DisplayFromStr>>"))]
    received_invoices: Vec<String>,

    #[cfg_attr(
        feature = "serde",
        serde(with = "As::<Vec<(DisplayFromStr, DisplayFromStr)>>")
    )]
    paid_invoices: BTreeMap<String, PaymentSlip>,

    transactions: BTreeMap<Txid, Psbt>,

    /* #[cfg_attr(
        feature = "serde",
        serde(with = "As::<Vec<(DisplayFromStr, _)>>")
    )]*/
    // Due to some weird bug the variant above ^^^ is not working
    #[serde_as(as = "Vec<(DisplayFromStr, _)>")]
    operations: BTreeMap<BlockchainTimepair, Operation>,
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
    pub fn with(contract: WalletContract) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Failed time service");
        Wallet {
            id: contract.id(),
            contract,
            created_at: NaiveDateTime::from_timestamp(
                timestamp.as_secs() as i64,
                0,
            ),
            checked_at: None,
            blinding_factors: empty!(),
            sent_invoices: empty!(),
            received_invoices: empty!(),
            paid_invoices: empty!(),
            transactions: empty!(),
            operations: empty!(),
        }
    }

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
    Hash,
    Debug,
    Display,
    StrictEncode,
    StrictDecode,
)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", rename_all = "lowercase", tag = "account")
)]
#[strict_encoding_crate(lnpbp::strict_encoding)]
#[non_exhaustive]
pub enum WalletContract {
    #[display("{descriptor}")]
    Current {
        name: String,
        #[cfg_attr(feature = "serde", serde(with = "As::<DisplayFromStr>"))]
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

impl ConsensusCommit for WalletContract {
    type Commitment = WalletId;
}

impl CommitEncode for WalletContract {
    fn commit_encode<E: io::Write>(self, e: E) -> usize {
        self.strict_encode(e)
            .expect("Memory encoders does not fail")
    }
}

impl WalletContract {
    pub fn id(&self) -> WalletId {
        self.clone().consensus_commit()
    }

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
    Hash,
    Debug,
    Display,
    StrictEncode,
    StrictDecode,
)]
#[strict_encoding_crate(lnpbp::strict_encoding)]
#[display("{channel_id}")]
pub struct InstantContract {
    channel_id: ChannelId,

    #[cfg_attr(feature = "serde", serde(with = "As::<Vec<DisplayFromStr>>"))]
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
    Hash,
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
#[strict_encoding_crate(lnpbp::strict_encoding)]
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
    pub mined_at: BlockchainTimepair,

    #[cfg_attr(feature = "serde", serde(with = "As::<DisplayFromStr>"))]
    pub txid: Txid,

    pub vout: u16,

    #[cfg_attr(feature = "serde", serde(with = "As::<DisplayFromStr>"))]
    pub value: bitcoin::Amount,

    pub invoice: String,

    pub details: String,
}
