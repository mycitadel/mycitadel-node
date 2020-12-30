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

use std::any::Any;

use lnpbp::lnp::zmqsocket::{self, ZmqType};
use lnpbp::lnp::{
    session, CreateUnmarshaller, PlainTranscoder, Session, TypedEnum,
    Unmarshall, Unmarshaller,
};
use lnpbp_services::node::TryService;

use crate::rpc::{Reply, Request};
use crate::Storage;
use crate::{Config, Error};

pub fn run(config: Config) -> Result<(), Error> {
    let runtime = Runtime::init(config)?;

    runtime.run_or_panic("keyringd");

    Ok(())
}

pub struct Runtime {
    /// Original configuration object
    config: Config,

    /// Stored sessions
    session_rpc: session::Raw<PlainTranscoder, zmqsocket::Connection>,

    /// Secure key vault
    storage: Storage,

    /// Unmarshaller instance used for parsing RPC request
    unmarshaller: Unmarshaller<Request>,
}

impl Runtime {
    pub fn init(config: Config) -> Result<Self, Error> {
        debug!("Initializing data storage {}", config.storage_conf());
        let storage = Storage::with(&config.storage_conf())?;

        debug!("Opening ZMQ socket {}", config.rpc_endpoint);
        let session_rpc = session::Raw::with_zmq_unencrypted(
            ZmqType::Rep,
            &config.rpc_endpoint,
            None,
            None,
        )?;

        Ok(Self {
            config,
            session_rpc,
            storage,
            unmarshaller: Request::create_unmarshaller(),
        })
    }
}

impl TryService for Runtime {
    type ErrorType = Error;

    fn try_run_loop(mut self) -> Result<(), Self::ErrorType> {
        loop {
            match self.run() {
                Ok(_) => debug!("API request processing complete"),
                Err(err) => {
                    error!("Error processing API request: {}", err);
                    Err(err)?;
                }
            }
        }
    }
}

impl Runtime {
    fn run(&mut self) -> Result<(), Error> {
        trace!("Awaiting for ZMQ RPC requests...");
        let raw = self.session_rpc.recv_raw_message()?;
        let reply = self.rpc_process(raw).unwrap_or_else(|err| err);
        trace!("Preparing ZMQ RPC reply: {:?}", reply);
        let data = reply.serialize();
        trace!(
            "Sending {} bytes back to the client over ZMQ RPC",
            data.len()
        );
        self.session_rpc.send_raw_message(&data)?;
        Ok(())
    }

    fn rpc_process(&mut self, raw: Vec<u8>) -> Result<Reply, Reply> {
        trace!("Got {} bytes over ZMQ RPC", raw.len());
        let message = (&*self.unmarshaller.unmarshall(&raw)?).clone();
        debug!("Received ZMQ RPC request: {:?}", message.type_id());
        match message {
            Request::List => unimplemented!(),
        }
    }
}
