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

use std::io;

use amplify::IoError;
#[cfg(any(feature = "node", feature = "client"))]
use internet2::TypeId;
use internet2::{presentation, transport};
#[cfg(any(feature = "node", feature = "client"))]
use microservices::rpc;

use crate::storage;

#[derive(Clone, Debug, Display, From, Error)]
#[display(doc_comments)]
#[non_exhaustive]
pub enum Error {
    /// Generic I/O error: {0:?}
    #[from(io::Error)]
    Io(IoError),

    /// RPC error: {0}
    #[cfg(any(feature = "node", feature = "client"))]
    #[from]
    Rpc(rpc::Error),

    /// General networking error: {0}
    #[from]
    Networking(presentation::Error),

    /// Transport-level interface error: {0}
    #[cfg(any(feature = "node", feature = "client"))]
    #[from]
    Transport(transport::Error),

    /// Provided RPC request (type id {0}) is not supported
    #[cfg(any(feature = "node", feature = "client"))]
    NotSupported(TypeId),

    /// Storage-level error:\n {0}
    #[cfg(any(feature = "server", feature = "embedded"))]
    #[from]
    StorageDriver(storage::Error),

    // TODO: split client- and server-side error types
    /// Server-reported failure
    #[from]
    ServerFailure(rpc::Failure),

    /// Error initializing embedded node
    EmbeddedNodeError,
}

impl microservices::error::Error for Error {}

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
