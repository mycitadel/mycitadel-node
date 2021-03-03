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

//! Storage drivers

pub mod file;

pub use file::{FileConfig, FileDriver};

// -----------------------------------------------------------------------------

use invoice::Invoice;
use lnpbp::seals::OutpointReveal;

use crate::model::{
    self, Contract, ContractId, Operation, Policy, TweakedOutput,
};
use crate::rpc::message::{IdentityInfo, SignerAccountInfo};

pub trait Driver {
    fn contracts(&self) -> Result<Vec<Contract>, Error>;
    fn contract_ref(&self, contract_id: ContractId)
        -> Result<&Contract, Error>;
    fn add_contract(&mut self, contract: Contract) -> Result<Contract, Error>;
    fn rename_contract(
        &mut self,
        contract_id: ContractId,
        new_name: String,
    ) -> Result<(), Error>;
    fn delete_contract(&mut self, contract_id: ContractId)
        -> Result<(), Error>;

    fn policy(&self, contract_id: ContractId) -> Result<&Policy, Error>;

    fn add_invoice(
        &mut self,
        contract_id: ContractId,
        invoice: Invoice,
        reveal_info: Vec<OutpointReveal>,
    ) -> Result<(), Error>;

    fn add_p2c_tweak(
        &mut self,
        contract_id: ContractId,
        tweak: TweakedOutput,
    ) -> Result<(), Error>;

    fn register_operation(
        &mut self,
        contract_id: ContractId,
        operation: Operation,
    ) -> Result<(), Error>;

    fn history(
        &self,
        contract_id: ContractId,
    ) -> Result<Vec<&Operation>, Error>;

    fn signers(&self) -> Result<Vec<SignerAccountInfo>, Error>;
    fn add_signer(
        &mut self,
        account: SignerAccountInfo,
    ) -> Result<SignerAccountInfo, Error>;

    fn identities(&self) -> Result<Vec<IdentityInfo>, Error>;
    fn add_identity(
        &mut self,
        identity: IdentityInfo,
    ) -> Result<IdentityInfo, Error>;
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
    ContractExists(model::ContractId),

    /// Contract with the given id {0} is not found
    ContractNotFound(model::ContractId),

    /// Identity with the provided id {0} already exists
    #[from]
    IdentityExists(rgb::ContractId),

    /// Error in strict data encoding: {0}
    /// Make sure that the storage is not broken.
    #[from]
    StrictEncoding(strict_encoding::Error),

    /// error in YAML data encoding: {0}
    YamlEncoding(String),

    /// error in YAML data encoding
    #[from(serde_json::Error)]
    JsonEncoding,

    /// error in YAML data encoding
    #[from(toml::de::Error)]
    #[from(toml::ser::Error)]
    TomlEncoding,
}

impl From<serde_yaml::Error> for Error {
    fn from(err: serde_yaml::Error) -> Self {
        Error::YamlEncoding(err.to_string())
    }
}
