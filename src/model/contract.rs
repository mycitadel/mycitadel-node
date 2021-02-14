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
use std::collections::BTreeMap;
use std::io;
use std::time::{SystemTime, UNIX_EPOCH};

use bitcoin::{OutPoint, Txid};
use lnpbp::client_side_validation::{CommitEncode, ConsensusCommit};
use lnpbp::Chain;
use strict_encoding::StrictEncode;
use wallet::bip32::UnhardenedIndex;
use wallet::{Psbt, TimeHeight};

use super::{ContractId, Operation, PaymentSlip, Policy, PolicyType, State};
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
#[display("{policy}#{id}")]
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
    #[serde_as(as = "DisplayFromStr")]
    id: ContractId,

    pub name: String,

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
    Debug,
    Default,
    StrictEncode,
    StrictDecode,
)]
pub struct ContractData {
    state: State,

    // TODO: Must be moved into rgb-node
    #[serde_as(as = "Vec<(DisplayFromStr, _)>")]
    blinding_factors: BTreeMap<OutPoint, u64>,

    sent_invoices: Vec<String>,

    received_invoices: Vec<String>,

    #[serde_as(as = "BTreeMap<_, DisplayFromStr>")]
    paid_invoices: BTreeMap<String, PaymentSlip>,

    transactions: BTreeMap<Txid, Psbt>,

    #[serde_as(as = "Vec<(DisplayFromStr, _)>")]
    operations: BTreeMap<TimeHeight, Operation>,
}

impl ConsensusCommit for Contract {
    type Commitment = ContractId;
}

impl CommitEncode for Contract {
    fn commit_encode<E: io::Write>(self, e: E) -> usize {
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

    pub fn derive_address(
        &self,
        index: UnhardenedIndex,
        legacy: bool,
    ) -> Option<AddressDerivation> {
        self.policy.derive_address(index, &self.chain, legacy)
    }
}
