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

use super::{AssetCommand, Command, WalletCommand, WalletCreateCommand};
use crate::rpc;
use crate::rpc::Reply;
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
            Command::Asset { subcommand } => subcommand.exec(client),
            _ => unimplemented!(),
        }
    }
}

impl Exec for WalletCommand {
    type Client = Client;
    type Error = Error;

    fn exec(self, client: &mut Self::Client) -> Result<(), Self::Error> {
        match self {
            WalletCommand::Create {
                subcommand:
                    WalletCreateCommand::SingleSig { name, pubkey_chain },
                bare,
                legacy,
                segwit: _,
                taproot,
            } => {
                let category = if bare {
                    descriptor::OuterCategory::Bare
                } else if legacy {
                    descriptor::OuterCategory::Hashed
                } else if taproot {
                    descriptor::OuterCategory::Taproot
                } else {
                    descriptor::OuterCategory::SegWit
                };
                eprintln!(
                    "Creating single-sig {} wallet with public key generator {}",
                    category.to_string().green(),
                    pubkey_chain.to_string().green(),

                );
                client
                    .create_single_sig(name, pubkey_chain, category)?
                    .report_error("during wallet creation")
                    .map(|reply| {
                        match reply {
                            Reply::Contract(contract) => {
                                eprint!(
                                    "Wallet named '{}' was successfully created.\
                                     Use the following string as the wallet id: ",
                                    contract.name().green().bold()
                                );
                                println!("{}", contract.id().to_string().bright_green());
                            }
                            _ => eprintln!(
                                "Unexpected server response; please check that \
                                 the client version matches server"
                            )
                        }
                    })
            }
            WalletCommand::List => client
                .contract_list()?
                .report_error("listing wallets")
                .map(|reply| {
                    eprintln!("Known wallets:");
                    println!(
                        "{}",
                        serde_yaml::to_string(&reply)
                            .expect("Error presenting data as YAML")
                    );
                }),
            _ => unimplemented!(),
        }
    }
}

impl Exec for AssetCommand {
    type Client = Client;
    type Error = Error;

    fn exec(self, client: &mut Self::Client) -> Result<(), Self::Error> {
        match self {
            AssetCommand::List => client
                .asset_list()?
                .report_error("listing assets")
                .map(|reply| {
                    eprintln!("Known assets:");
                    println!(
                        "{}",
                        serde_yaml::to_string(&reply)
                            .expect("Error presenting data as YAML")
                    );
                }),
            AssetCommand::Import { genesis } => client
                .asset_import(genesis)?
                .report_error("importing asset")
                .map(|reply| {
                    eprintln!("Asset successfully imported:");
                    println!(
                        "{}",
                        serde_yaml::to_string(&reply)
                            .expect("Error presenting data as YAML")
                    )
                }),
        }
    }
}
