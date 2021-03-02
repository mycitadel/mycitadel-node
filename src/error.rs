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

use crate::{cache, storage};

#[derive(Clone, Debug, Display, From, Error)]
#[display(doc_comments)]
#[non_exhaustive]
pub enum Error {
    /// generic I/O error - {0:?}
    #[from(io::Error)]
    Io(IoError),

    /// RPC error - {0}
    #[cfg(any(feature = "node", feature = "client"))]
    #[from]
    Rpc(rpc::Error),

    /// general networking error = {0}
    #[from]
    Networking(presentation::Error),

    /// transport-level interface error - {0}
    #[cfg(any(feature = "node", feature = "client"))]
    #[from]
    Transport(transport::Error),

    /// provided RPC request (type id {0}) is not supported
    #[cfg(any(feature = "node", feature = "client"))]
    NotSupported(TypeId),

    /// RGB node error - {0}
    #[cfg(any(feature = "server", feature = "embedded"))]
    #[from(rgb_node::i9n::Error)]
    RgbNode,

    /// electrum server error - {0}
    #[from(electrum_client::Error)]
    Electrum,

    /// storage failure - {0}
    #[cfg(any(feature = "server", feature = "embedded"))]
    #[from]
    StorageDriver(storage::Error),

    /// cache failure - {0}
    #[cfg(any(feature = "server", feature = "embedded"))]
    #[from]
    CacheDriver(cache::Error),

    // TODO: split client- and server-side error types
    /// server-reported failure
    #[from]
    #[display(inner)]
    ServerFailure(rpc::Failure),

    /// internal cache inconsistency; you need to refresh balances and try
    /// again
    CacheInconsistency,

    /// strict data encoding data failure - {0}
    #[from]
    StrictEncoding(strict_encoding::Error),

    /// in bitcoin consensus-encoded data failure
    #[from(bitcoin::consensus::encode::Error)]
    ConsensisEncoding,

    /// base64 encoding failure - {0}
    #[from]
    Base64(base64::DecodeError),

    /// bech32 encoding failure - {0}
    #[from]
    #[from(bech32::Error)]
    Bech32(lnpbp::bech32::Error),

    /// embedded node initialization failure
    EmbeddedNodeInitError,

    /// unexpected RPC API message; please check that the client version
    /// matches server
    UnexpectedApi,
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
