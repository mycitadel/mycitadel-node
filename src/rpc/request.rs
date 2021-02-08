// Keyring: private/public key managing service
// Written in 2021 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the AGPL License
// along with this software.
// If not, see <https://www.gnu.org/licenses/agpl-3.0-standalone.html>.

use bitcoin::Address;
use rgb::Genesis;

use super::message::{
    ContractRenameRequest, IdentityInfo, SignerAccountInfo, SingleSigInfo,
};
use crate::model::ContractId;

#[derive(Clone, Debug, Display, Api)]
#[api(encoding = "strict")]
#[non_exhaustive]
pub enum Request {
    #[api(type = 0x0100)]
    #[display("list_contracts()")]
    ListContracts,

    #[api(type = 0x0101)]
    #[display("contract_balance({0})")]
    ContractBalance(ContractId),

    #[api(type = 0x0110)]
    #[display(inner)]
    CreateSingleSig(SingleSigInfo),

    // Multisig 112
    // Scripted 114
    #[api(type = 0x0120)]
    #[display(inner)]
    RenameContract(ContractRenameRequest),

    #[api(type = 0x0140)]
    #[display("delete_contract({0})")]
    DeleteContract(ContractId),

    #[api(type = 0x0300)]
    #[display(inner)]
    ListAddresses(u8),

    #[api(type = 0x0310)]
    #[display(inner)]
    CreateAddress(u8),

    #[api(type = 0x0320)]
    #[display("mark_used({0})")]
    MarkUsed(Address),

    #[api(type = 0x0332)]
    #[display("mark_unused({0})")]
    MarkUnused(Address),

    #[api(type = 0x0500)]
    #[display("list_identities()")]
    ListIdentities,

    #[api(type = 0x0610)]
    #[display("add_signing({0})")]
    AddSigner(SignerAccountInfo),

    #[api(type = 0x0510)]
    #[display("add_identity({0})")]
    AddIdentity(IdentityInfo),

    #[api(type = 0x0700)]
    #[display("list_assets()")]
    ListAssets,

    #[api(type = 0x0710)]
    #[display("import_asset({0})")]
    ImportAsset(Genesis),
}
