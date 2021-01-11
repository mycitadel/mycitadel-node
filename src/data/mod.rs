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

use lnpbp::bp::descriptor;
use lnpbp::lnp::{ChannelId, RemoteNodeAddr};
#[cfg(feature = "serde")]
use serde_with::DisplayFromStr;

#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate")
)]
#[derive(Getters, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct Wallet {
    title: String,
    contract: WalletContract,
}

impl Wallet {
    // pub fn id(&self) -> AccountId {}

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
#[non_exhaustive]
pub enum WalletContract {
    #[display("current:{0}")]
    Current(descriptor::Generator),

    #[display("instant:{0}")]
    Instant(InstantContract),

    #[display("saving:{0}")]
    Saving(SavingContract),
}

impl WalletContract {
    pub fn contract_type(&self) -> ContractType {
        match self {
            WalletContract::Current(_) => ContractType::Current,
            WalletContract::Instant(_) => ContractType::Instant,
            WalletContract::Saving(_) => ContractType::Saving,
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
#[non_exhaustive]
pub enum SavingContract {
    #[display(inner)]
    MultiSig(descriptor::MultiSig),

    #[display("covenant")]
    Covenant,
}
