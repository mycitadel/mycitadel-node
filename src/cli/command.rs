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

use colored::Colorize;
use microservices::shell::Exec;
use wallet::descriptor;

use super::{Command, WalletCommand, WalletCreateCommand};
use crate::data::WalletContract;
use crate::rpc;
use crate::{Client, Error};

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
    type Client = Client;
    type Error = Error;

    #[inline]
    fn exec(self, client: &mut Self::Client) -> Result<(), Self::Error> {
        match self {
            Command::Wallet { subcommand } => subcommand.exec(client),
        }
    }
}

impl Exec for WalletCommand {
    type Client = Client;
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
    type Client = Client;
    type Error = Error;

    fn exec(self, client: &mut Self::Client) -> Result<(), Self::Error> {
        match self {
            WalletCreateCommand::Current {
                name,
                variants,
                template,
            } => {
                let descriptor = descriptor::Generator { variants, template };
                eprintln!("Creating current wallet with descriptor template generator {}", descriptor.to_string().yellow());
                let contract = WalletContract::Current {
                    name: name.clone(),
                    descriptor: descriptor.clone(),
                };
                let id = contract.id();
                client.wallet_create_current(contract)?.report_error("during wallet creation").map(|_| {
                    eprint!("Wallet named '{}' was successfully created.\nUse the following string as the wallet id: ", name.yellow().bold());
                    println!("{}", id.to_string().bright_yellow());
                })
            }
        }
    }
}
