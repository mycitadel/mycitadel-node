// Keyring: private/public key managing service
// Written in 2020 by
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

use super::message::{IdentityInfo, SignerAccount};
use crate::data::WalletContract;

#[derive(Clone, Debug, Display, LnpApi)]
#[lnp_api(encoding = "strict")]
#[encoding_crate(lnpbp::strict_encoding)]
#[non_exhaustive]
pub enum Request {
    #[lnp_api(type = 0x0010)]
    #[display("list_wallets()")]
    ListWallets,

    #[lnp_api(type = 0x0012)]
    #[display("list_identities()")]
    ListIdentities,

    #[lnp_api(type = 0x0014)]
    #[display("list_assets()")]
    ListAssets,

    #[lnp_api(type = 0x0020)]
    #[display("add_wallet({0})")]
    AddWallet(WalletContract),

    #[lnp_api(type = 0x0030)]
    #[display("add_signing({0})")]
    AddSigner(SignerAccount),

    #[lnp_api(type = 0x0040)]
    #[display("add_identity({0})")]
    AddIdentity(IdentityInfo),

    #[lnp_api(type = 0x0050)]
    #[display("add_asset({0})")]
    AddAsset(Genesis),
}
