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

//! Storage drivers

pub mod file;

pub use file::{FileConfig, FileDriver};

// -----------------------------------------------------------------------------

use rgb::{ContractId, Genesis};
use rgb20::Asset;

use crate::data::{SignerId, WalletContract, WalletId};
use crate::rpc::message::{IdentityInfo, SignerAccount};

pub trait Driver {
    fn wallets(&self) -> Result<Vec<WalletContract>, Error>;
    fn add_wallet(&mut self, contract: WalletContract) -> Result<(), Error>;

    fn signers(&self) -> Result<Vec<SignerAccount>, Error>;
    fn add_signer(&mut self, account: SignerAccount) -> Result<(), Error>;

    fn identities(&self) -> Result<Vec<IdentityInfo>, Error>;
    fn add_identity(&mut self, identity: IdentityInfo) -> Result<(), Error>;

    fn assets(&self) -> Result<Vec<Asset>, Error>;
    fn add_asset(&mut self, genesis: Genesis) -> Result<(), Error>;
}

#[derive(Clone, Eq, PartialEq, Hash, Debug, Display, Error, From)]
#[display(doc_comments)]
#[non_exhaustive]
pub enum Error {
    /// I/O error during storage operations. Details: {0}
    #[from]
    #[from(std::io::Error)]
    Io(amplify::IoError),

    /// Wallet corresponding to the provided descriptor already exists.
    /// If you are trying to update wallet name use wallet rename command.
    ///
    /// Details on existing wallet: {0}
    #[from]
    WalletExists(WalletId),

    /// Signer with the provided id {0} already exists
    #[from]
    SignerExists(SignerId),

    /// Identity with the provided id {0} already exists
    #[from]
    IdentityExists(ContractId),

    /// Error in strict data encoding: {0}
    /// Make sure that the storage is not broken.
    #[from]
    StrictEncoding(strict_encoding::Error),

    /// Error in YAML data encoding: {0}
    /// Make sure that the storage is not broken.
    #[cfg(feature = "serde_yaml")]
    #[from(serde_yaml::Error)]
    YamlEncoding,

    /// Error in YAML data encoding: {0}
    /// Make sure that the storage is not broken.
    #[cfg(feature = "serde_json")]
    #[from(serde_json::Error)]
    JsonEncoding,

    /// Error in YAML data encoding: {0}
    /// Make sure that the storage is not broken.
    #[cfg(feature = "toml")]
    #[from(toml::de::Error)]
    #[from(toml::ser::Error)]
    TomlEncoding,
}
