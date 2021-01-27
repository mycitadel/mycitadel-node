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

use microservices::shell::Exec;
use wallet::descriptor;

use super::{Command, WalletCommand, WalletCreateCommand};
use crate::data::WalletContract;
use crate::rpc;
use crate::Error;

impl rpc::Reply {
    pub fn report_error(self, msg: &str) -> Result<Self, Error> {
        match self {
            rpc::Reply::Failure(failure) => {
                error!("Error {} #{}: {}", msg, failure.code, failure.info);
                Err(failure)?
            }
            _ => Ok(self),
        }
    }
}

impl Exec for Command {
    type Client = rpc::Client;
    type Error = Error;

    #[inline]
    fn exec(self, client: &mut rpc::Client) -> Result<(), Self::Error> {
        match self {
            Command::Wallet { subcommand } => subcommand.exec(client),
        }
    }
}

impl Exec for WalletCommand {
    type Client = rpc::Client;
    type Error = Error;

    fn exec(self, client: &mut Self::Client) -> Result<(), Self::Error> {
        match self {
            WalletCommand::Create { subcommand } => subcommand.exec(client),
            WalletCommand::List => client
                .wallet_list()?
                .report_error("listing wallets")
                .map(|reply| {
                    eprintln!("Known wallets:");
                    println!(
                        "{}",
                        serde_yaml::to_string(&reply)
                            .expect("Error presenting data as YAML")
                    );
                }),
        }
    }
}

impl Exec for WalletCreateCommand {
    type Client = rpc::Client;
    type Error = Error;

    fn exec(self, client: &mut Self::Client) -> Result<(), Self::Error> {
        match self {
            WalletCreateCommand::Current {
                name,
                variants,
                template,
            } => {
                let descriptor = descriptor::Generator { variants, template };
                info!("Creating current wallet with descriptor {}", descriptor);
                client.wallet_create_current(WalletContract::Current {
                    name: name.clone(),
                    descriptor: descriptor.clone(),
                })?.report_error("during wallet creation").map(|_| {
                    eprint!("Wallet named '{}' was successfully created.\nUse the following string as the wallet id: ", name);
                    println!("{}", descriptor);
                })
            }
        }
    }
}
