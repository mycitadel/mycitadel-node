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

use rgb::Genesis;
use rgb20::Asset;

use crate::data::WalletContract;
use crate::rpc::message::{IdentityInfo, SignerAccount};

pub trait Driver {
    fn wallets(&self) -> Result<Vec<WalletContract>, Error>;
    fn add_wallet(
        &mut self,
        contract: WalletContract,
    ) -> Result<(), Self::Error>;

    fn signers(&self) -> Result<Vec<SignerAccount>, Error>;
    fn add_signer(&mut self, account: SignerAccount)
        -> Result<(), Self::Error>;

    fn identities(&self) -> Result<Vec<IdentityInfo>, Error>;
    fn add_idenity(
        &mut self,
        identity: IdentityInfo,
    ) -> Result<(), Self::Error>;

    fn assets(&self) -> Result<Vec<Asset>, Self::Error>;
    fn add_asset(&mut self, genesis: Genesis) -> Result<(), Error>;
}

#[derive(
    Copy,
    Clone,
    Ord,
    PartialOrd,
    Eq,
    PartialEq,
    Hash,
    Debug,
    Display,
    Error,
    From,
)]
#[display(doc_comments)]
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
    WalletExists(WalletContract),

    #[from]
    SignerExists(SignerAccount),

    #[from]
    IdentityExists(IdentityInfo),
}
