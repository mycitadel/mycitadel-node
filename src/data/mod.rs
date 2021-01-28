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

pub mod identity;
pub mod signer;
pub mod wallet;

pub use self::wallet::{Wallet, WalletContract, WalletId};
pub use signer::SignerId;

// -----------------------------------------------------------------------------

use std::collections::BTreeMap;

use crate::rpc::message::{IdentityInfo, SignerAccount};

#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate")
)]
#[derive(Clone, PartialEq, Debug, Default, StrictEncode, StrictDecode)]
#[strict_encoding_crate(lnpbp::strict_encoding)]
pub struct Data {
    pub wallets: BTreeMap<WalletId, Wallet>,
    pub signers: BTreeMap<SignerId, SignerAccount>,
    pub identities: BTreeMap<rgb::ContractId, IdentityInfo>,
    pub assets: BTreeMap<rgb::ContractId, rgb20::Asset>,
}
