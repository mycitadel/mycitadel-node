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

use std::io;

use amplify::IoError;
#[cfg(any(feature = "node", feature = "client"))]
use lnpbp::lnp::TypeId;
use lnpbp::lnp::{presentation, transport};
#[cfg(any(feature = "node", feature = "client"))]
use lnpbp_services::{esb, rpc};

use crate::storage;

#[derive(Clone, Debug, Display, From, Error)]
#[display(doc_comments)]
#[non_exhaustive]
pub enum Error {
    /// I/O error: {0:?}
    #[from(io::Error)]
    Io(IoError),

    /// ESB error: {0}
    #[cfg(any(feature = "node", feature = "client"))]
    #[from]
    Esb(esb::Error),

    /// RPC error: {0}
    #[cfg(any(feature = "node", feature = "client"))]
    #[from]
    Rpc(rpc::Error),

    /// Peer interface error: {0}
    #[from]
    Peer(presentation::Error),

    /// Bridge interface error: {0}
    #[cfg(any(feature = "node", feature = "client"))]
    #[from(zmq::Error)]
    #[from]
    Bridge(transport::Error),

    /// Provided RPC request is not supported for the used type of endpoint
    #[cfg(any(feature = "node", feature = "client"))]
    NotSupported(TypeId),

    /// Peer does not respond to ping messages
    NotResponding,

    /// Peer has misbehaved LN peer protocol rules
    Misbehaving,

    /// unrecoverable error "{0}"
    Terminate(String),

    #[cfg(any(feature = "server", feature = "embedded"))]
    #[from]
    StorageDriver(storage::driver::Error),

    /// Other error type with string explanation
    #[display(inner)]
    #[from(amplify::internet::NoOnionSupportError)]
    Other(String),
}

impl lnpbp_services::error::Error for Error {}

#[cfg(any(feature = "node", feature = "client"))]
impl From<Error> for esb::Error {
    fn from(err: Error) -> Self {
        match err {
            Error::Esb(err) => err,
            err => esb::Error::ServiceError(err.to_string()),
        }
    }
}

#[cfg(any(feature = "node", feature = "client"))]
impl From<Error> for rpc::Error {
    fn from(err: Error) -> Self {
        match err {
            Error::Rpc(err) => err,
            err => rpc::Error::ServerFailure(rpc::Failure {
                code: 2000,
                info: err.to_string(),
            }),
        }
    }
}
