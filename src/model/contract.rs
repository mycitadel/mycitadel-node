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
use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::time::{SystemTime, UNIX_EPOCH};

use bitcoin::{OutPoint, PublicKey, Script, Txid};
use invoice::Invoice;
use lnpbp::client_side_validation::{
    CommitConceal, CommitEncode, ConsensusCommit,
};
use lnpbp::seals::{OutpointHash, OutpointReveal};
use lnpbp::Chain;
use miniscript::ForEachKey;
use strict_encoding::StrictEncode;
use wallet::bip32::{PubkeyChain, UnhardenedIndex};
use wallet::Slice32;

use super::{ContractId, Operation, Policy, PolicyType, State};
use crate::model::AddressDerivation;

#[serde_as]
#[derive(
    Serialize,
    Deserialize,
    Getters,
    Clone,
    PartialEq,
    Debug,
    Display,
    StrictEncode,
    StrictDecode,
)]
#[display("{policy}")]
pub struct Contract {
    /// Unique contract id used to identify contract across different
    /// application instances. Created as a taproot-style bitcoin tagged
    /// hash out of strict-encoded wallet policy data: when policy
    /// changes contract id changes; if two contract on different devices have
    /// the same underlying policies they will have the same id.
    ///
    /// The id is kept pre-computed: the contract policy can't be changed after
    /// the creation, so there is no need to perform expensive commitment
    /// process each time we need contract id
    // TODO: Consider using descriptor checksum instead
    #[serde_as(as = "DisplayFromStr")]
    id: ContractId,

    pub name: String,

    #[serde_as(as = "DisplayFromStr")]
    chain: Chain,

    policy: Policy,

    #[serde_as(as = "chrono::DateTime<chrono::Utc>")]
    created_at: NaiveDateTime,

    #[serde(flatten)]
    data: ContractData,
}

#[serde_as]
#[derive(
    Serialize,
    Deserialize,
    Getters,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Debug,
    StrictEncode,
    StrictDecode,
)]
#[serde(rename_all = "camelCase")]
pub struct ContractMeta {
    #[serde_as(as = "DisplayFromStr")]
    id: ContractId,

    name: String,

    #[serde_as(as = "DisplayFromStr")]
    chain: Chain,

    policy: Policy,

    #[serde_as(as = "chrono::DateTime<chrono::Utc>")]
    created_at: NaiveDateTime,
}

impl From<Contract> for ContractMeta {
    fn from(contract: Contract) -> Self {
        ContractMeta {
            id: contract.id,
            name: contract.name,
            chain: contract.chain,
            policy: contract.policy,
            created_at: contract.created_at,
        }
    }
}

#[serde_as]
#[derive(
    Serialize,
    Deserialize,
    Getters,
    Clone,
    PartialEq,
    Debug,
    Default,
    StrictEncode,
    StrictDecode,
)]
pub struct ContractData {
    state: State,

    // TODO: Must be moved into rgb-node
    #[serde_as(as = "BTreeMap<DisplayFromStr, _>")]
    blinding_factors: BTreeMap<OutpointHash, OutpointReveal>,

    #[serde_as(as = "Vec<DisplayFromStr>")]
    sent_invoices: Vec<Invoice>,

    #[serde_as(as = "BTreeMap<DisplayFromStr, chrono::DateTime<chrono::Utc>>")]
    unpaid_invoices: BTreeMap<Invoice, NaiveDateTime>,

    p2c_tweaks: BTreeSet<TweakedOutput>,

    #[serde_as(as = "Vec<(DisplayFromStr, _)>")]
    operations: BTreeMap<Txid, Operation>,
}

impl ConsensusCommit for Contract {
    type Commitment = ContractId;
}

impl CommitEncode for Contract {
    fn commit_encode<E: io::Write>(&self, e: E) -> usize {
        self.policy
            .strict_encode(e)
            .expect("Memory encoders does not fail")
    }
}

impl Contract {
    pub fn with(policy: Policy, name: String, chain: Chain) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Failed time service");
        Contract {
            id: policy.id(),
            name,
            chain,
            policy,
            created_at: NaiveDateTime::from_timestamp(
                timestamp.as_secs() as i64,
                0,
            ),
            data: ContractData::default(),
        }
    }

    pub fn policy_type(&self) -> PolicyType {
        self.policy.policy_type()
    }

    pub fn pubkeychains(&self) -> Vec<PubkeyChain> {
        let mut pubkeychains = vec![];
        self.policy.to_descriptor().for_each_key(|key| {
            match key {
                miniscript::ForEach::Key(pubkeychain) => {
                    pubkeychains.push(pubkeychain.clone())
                }
                miniscript::ForEach::Hash(_) => unreachable!(),
            };
            true
        });
        pubkeychains
    }

    pub fn derive_address(
        &self,
        index: UnhardenedIndex,
        legacy: bool,
    ) -> Option<AddressDerivation> {
        self.policy.derive_address(index, &self.chain, legacy)
    }

    pub fn tweaked_script_iter(&self) -> impl Iterator<Item = Script> + '_ {
        self.data
            .p2c_tweaks
            .iter()
            .map(|tweak| tweak.script.clone())
    }

    // TODO: This must be private and must be used by storage driver only
    pub(crate) fn add_p2c_tweak(&mut self, tweak: TweakedOutput) {
        self.data.p2c_tweaks.insert(tweak);
    }

    // TODO: This must be private and must be used by storage driver only
    pub(crate) fn add_operation(&mut self, operation: Operation) {
        self.data
            .operations
            .insert(operation.psbt.global.unsigned_tx.txid(), operation);
    }

    // TODO: This must be private and must be used by storage driver only
    pub(crate) fn history(&self) -> Vec<&Operation> {
        self.data.operations.values().collect()
    }

    // TODO: This must be private and must be used by storage driver only
    pub(crate) fn add_invoice(&mut self, invoice: Invoice) {
        if !self.data.sent_invoices.contains(&invoice) {
            self.data.sent_invoices.push(invoice)
        }
    }

    // TODO: This must be private and must be used by storage driver only
    pub(crate) fn add_blinding(&mut self, outpoint_reveal: OutpointReveal) {
        self.data
            .blinding_factors
            .insert(outpoint_reveal.commit_conceal(), outpoint_reveal);
    }
}

#[derive(
    Getters,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Debug,
    StrictEncode,
    StrictDecode,
    Serialize,
    Deserialize,
)]
pub struct TweakedOutput {
    pub outpoint: OutPoint,
    pub script: Script,
    pub tweak: Slice32,
    pub pubkey: PublicKey,
    pub derivation_index: UnhardenedIndex,
}
