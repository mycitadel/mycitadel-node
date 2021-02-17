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

use rgb::Genesis;

use super::message::{
    AddInvoiceRequest, ComposePaymentRequest, ContractAddressTuple,
    IdentityInfo, NextAddressRequest, RenameContractRequest, SignerAccountInfo,
    SingleSigInfo, SyncContractRequest,
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
    #[display(inner)]
    SyncContract(SyncContractRequest),

    #[api(type = 0x0102)]
    #[display("contract_unspent({0})")]
    ContractUnspent(ContractId),

    #[api(type = 0x0110)]
    #[display(inner)]
    CreateSingleSig(SingleSigInfo),

    #[api(type = 0x0120)]
    #[display(inner)]
    RenameContract(RenameContractRequest),

    #[api(type = 0x0140)]
    #[display("delete_contract({0})")]
    DeleteContract(ContractId),

    #[api(type = 0x0310)]
    #[display("used_addresses({0})")]
    UsedAddresses(ContractId),

    #[api(type = 0x0312)]
    #[display("next_address({0})")]
    NextAddress(NextAddressRequest),

    #[api(type = 0x0314)]
    #[display("unuse_address(...)")]
    UnuseAddress(ContractAddressTuple),

    #[api(type = 0x0320)]
    #[display("blind_utxo({0})")]
    BlindUtxo(ContractId),

    #[api(type = 0x0400)]
    #[display("list_invoices({0})")]
    ListInvoices(ContractId),

    #[api(type = 0x0410)]
    #[display(inner)]
    AddInvoice(AddInvoiceRequest),

    #[api(type = 0x0420)]
    #[display(inner)]
    ComposePayment(ComposePaymentRequest),

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
