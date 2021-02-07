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

use rgb::Genesis;

use super::message::{CreateSingleSig, IdentityInfo, SignerAccount};

#[derive(Clone, Debug, Display, Api)]
#[api(encoding = "strict")]
#[non_exhaustive]
pub enum Request {
    #[api(type = 0x0010)]
    #[display("list_contracts()")]
    ListContracts,

    #[api(type = 0x0012)]
    #[display("list_identities()")]
    ListIdentities,

    #[api(type = 0x0014)]
    #[display("list_assets()")]
    ListAssets,

    #[api(type = 0x0020)]
    #[display("create_single_sig({0})")]
    CreateSingleSig(CreateSingleSig),

    #[api(type = 0x0030)]
    #[display("add_signing({0})")]
    AddSigner(SignerAccount),

    #[api(type = 0x0040)]
    #[display("add_identity({0})")]
    AddIdentity(IdentityInfo),

    #[api(type = 0x0050)]
    #[display("import_asset({0})")]
    ImportAsset(Genesis),
}
