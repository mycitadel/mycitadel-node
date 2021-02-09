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

use super::{
    AssetCommand, Command, OutputFormat, WalletCommand, WalletCreateCommand,
    WalletOpts,
};
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
                    .and_then(|reply| match reply {
                        Reply::Contract(contract) => Ok(contract),
                        _ => Err(Error::UnexpectedApi),
                    })
                    .map(|contract| {
                        eprintln!(
                            "Wallet named '{}' was successfully created.\n
                             Use the following string as the wallet id:",
                            contract.name().green().bold()
                        );
                        println!(
                            "{}",
                            contract.id().to_string().bright_green()
                        );
                    })
            }
            WalletCommand::List { format } => client
                .contract_list()?
                .report_error("listing wallets")
                .and_then(|reply| match reply {
                    Reply::Contracts(contracts) => Ok(contracts),
                    _ => Err(Error::UnexpectedApi),
                })
                .map(|contracts| contracts.output_print(format)),
            WalletCommand::Balance {
                opts:
                    WalletOpts {
                        wallet_id,
                        rescan,
                        lookup_depth,
                        format,
                    },
            } => client
                .contract_balance(wallet_id, rescan, lookup_depth)?
                .report_error("retrieving wallet balance")
                .and_then(|reply| match reply {
                    Reply::ContractUnspent(unspent) => Ok(unspent),
                    _ => Err(Error::UnexpectedApi),
                })
                .map(|unspent| unspent.output_print(format)),
            _ => unimplemented!(),
        }
    }
}

impl Exec for AssetCommand {
    type Client = Client;
    type Error = Error;

    fn exec(self, client: &mut Self::Client) -> Result<(), Self::Error> {
        match self {
            AssetCommand::List { format } => client
                .asset_list()?
                .report_error("listing assets")
                .and_then(|reply| match reply {
                    Reply::Assets(assets) => Ok(assets),
                    _ => Err(Error::UnexpectedApi),
                })
                .map(|assets| assets.output_print(format)),
            AssetCommand::Import { genesis } => client
                .asset_import(genesis)?
                .report_error("importing asset")
                .and_then(|reply| match reply {
                    Reply::Asset(asset) => Ok(asset),
                    _ => Err(Error::UnexpectedApi),
                })
                .map(|asset| {
                    eprintln!("Asset successfully imported:");
                    println!(
                        "{}",
                        serde_yaml::to_string(&asset)
                            .expect("Error presenting data as YAML")
                    )
                }),
        }
    }
}
