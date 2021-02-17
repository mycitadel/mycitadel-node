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
use std::collections::HashMap;
use std::str::FromStr;
use std::{fs, io};

use base64::display::Base64Display;
use bitcoin::consensus::encode::{serialize, Encodable};
use bitcoin::hashes::hex::ToHex;
use microservices::shell::Exec;
use rgb::{Consignment, Validity};
use strict_encoding::StrictEncode;

use super::{
    AddressCommand, AssetCommand, Command, InvoiceCommand, OutputFormat,
    PsbtFormat, WalletCommand, WalletCreateCommand, WalletOpts,
};
use crate::client::InvoiceType;
use crate::rpc::Reply;
use crate::{Client, Error};

const LOOKUP_DEPTH_DEFAULT: u8 = 20;

impl Reply {
    pub fn report_error(self, msg: &str) -> Result<Self, Error> {
        match self {
            Reply::Failure(failure) => {
                eprintln!(
                    "{} {} {}{}:\n{} {}",
                    "Error".bright_red(),
                    msg.bright_red(),
                    "#".bright_red().bold(),
                    failure.code.to_string().bright_red().bold(),
                    "-".red(),
                    failure.info.red()
                );
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
            Command::Address { subcommand } => subcommand.exec(client),
            Command::Invoice { subcommand } => subcommand.exec(client),
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
                    WalletCreateCommand::SingleSig {
                        name,
                        pubkey_chain,
                        opts,
                    },
            } => {
                let category = opts.descriptor_category();
                eprintln!(
                    "Creating single-sig {} wallet with public key generator {}",
                    category.to_string().yellow(),
                    pubkey_chain.to_string().yellow(),

                );
                client
                    .single_sig_create(name, pubkey_chain, category)?
                    .report_error("during wallet creation")
                    .and_then(|reply| match reply {
                        Reply::Contract(contract) => Ok(contract),
                        _ => Err(Error::UnexpectedApi),
                    })
                    .map(|contract| {
                        eprintln!(
                            "Wallet named '{}' was successfully created.\n\
                            Use the following string as the wallet id:",
                            contract.name().green()
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
            WalletCommand::Rename {
                wallet_id,
                new_name,
            } => client
                .contract_rename(wallet_id, new_name.clone())?
                .report_error("renaming wallet")
                .map(|_| {
                    eprintln!(
                        "Wallet with id {} was successfully renamed into '{}'",
                        wallet_id.to_string().yellow(),
                        new_name.bright_green()
                    );
                }),
            WalletCommand::Delete { wallet_id } => client
                .contract_delete(wallet_id)?
                .report_error("deleting wallet")
                .map(|_| {
                    eprintln!(
                        "Wallet with id {} was successfully {}",
                        wallet_id.to_string().yellow(),
                        "deleted".red()
                    );
                }),
            WalletCommand::Balance {
                scan_opts:
                    WalletOpts {
                        wallet_id,
                        rescan,
                        lookup_depth,
                        format,
                    },
            } => client
                .contract_balance(
                    wallet_id,
                    rescan,
                    lookup_depth.unwrap_or(LOOKUP_DEPTH_DEFAULT),
                )?
                .report_error("retrieving wallet balance")
                .and_then(|reply| match reply {
                    Reply::ContractUnspent(unspent) => Ok(unspent),
                    _ => Err(Error::UnexpectedApi),
                })
                .map(|unspent| unspent.output_print(format)),
        }
    }
}

impl Exec for AddressCommand {
    type Client = Client;
    type Error = Error;

    fn exec(self, client: &mut Self::Client) -> Result<(), Self::Error> {
        match self {
            AddressCommand::ListUsed {
                scan_opts:
                    WalletOpts {
                        wallet_id,
                        rescan,
                        lookup_depth,
                        format,
                    },
            } => client
                .address_list(
                    wallet_id,
                    rescan,
                    lookup_depth.unwrap_or(LOOKUP_DEPTH_DEFAULT),
                )?
                .report_error("retrieving used addresses")
                .and_then(|reply| match reply {
                    Reply::Addresses(addresses) => Ok(addresses),
                    _ => Err(Error::UnexpectedApi),
                })
                .map(|addresses| {
                    addresses
                        .into_iter()
                        .collect::<HashMap<_, _>>()
                        .output_print(format)
                }),
            AddressCommand::Create {
                wallet_id,
                mark_used,
                index,
                legacy,
                format,
            } => client
                .address_create(wallet_id, index, mark_used, legacy)?
                .report_error("generating address")
                .and_then(|reply| match reply {
                    Reply::AddressDerivation(ad) => Ok(ad),
                    _ => Err(Error::UnexpectedApi),
                })
                .map(|address_derivation| {
                    address_derivation.output_print(format)
                }),
            AddressCommand::MarkUsed { .. } => unimplemented!(),
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

impl Exec for InvoiceCommand {
    type Client = Client;
    type Error = Error;

    fn exec(self, client: &mut Self::Client) -> Result<(), Self::Error> {
        match self {
            InvoiceCommand::Create {
                wallet_id,
                asset_id,
                amount,
                merchant,
                purpose,
                unmark,
                legacy,
                descriptor,
                psbt,
            } => {
                // TODO: Check that asset id is known
                client
                    .invoice_create(
                        if descriptor {
                            InvoiceType::Descriptor
                        } else if psbt {
                            InvoiceType::Psbt
                        } else {
                            InvoiceType::AddressUtxo
                        },
                        wallet_id,
                        asset_id,
                        amount,
                        merchant,
                        purpose,
                        unmark,
                        legacy,
                    )
                    .map(|invoice| {
                        eprintln!("Invoice successfully created:");
                        println!(
                            "{}",
                            invoice.to_string().as_str().bright_green()
                        )
                    })
            }
            InvoiceCommand::List { wallet_id, format } => client
                .invoice_list(wallet_id)?
                .report_error("listing invoices")
                .and_then(|reply| match reply {
                    Reply::Invoices(list) => Ok(list),
                    _ => Err(Error::UnexpectedApi),
                })
                .map(|list| list.output_print(format)),
            InvoiceCommand::Info { invoice, format } => {
                Ok(invoice.output_print(format))
            }
            InvoiceCommand::Pay {
                invoice,
                wallet_id,
                amount,
                fee,
                output,
                consignment,
                format,
                giveaway,
            } => {
                let prepared_payment = client
                    .invoice_pay(wallet_id, invoice, amount, fee, giveaway)?;
                let (mut psbt_file, format) = if let Some(ref filename) = output
                {
                    (
                        Box::new(io::BufWriter::new(fs::File::create(
                            filename,
                        )?)) as Box<dyn io::Write>,
                        format.unwrap_or(PsbtFormat::Binary),
                    )
                } else {
                    (
                        Box::new(io::BufWriter::new(io::stdout()))
                            as Box<dyn io::Write>,
                        format.unwrap_or(PsbtFormat::Base64),
                    )
                };

                if output.is_none() {
                    eprint!("{} ", "PSBT:".bright_yellow());
                }
                match format {
                    PsbtFormat::Binary => {
                        prepared_payment
                            .psbt
                            .consensus_encode(&mut psbt_file)?;
                    }
                    PsbtFormat::Hexadecimal => {
                        psbt_file.write_all(
                            serialize(&prepared_payment.psbt)
                                .to_hex()
                                .as_bytes(),
                        )?;
                    }
                    PsbtFormat::Base64 => {
                        psbt_file.write_all(
                            Base64Display::with_config(
                                &serialize(&prepared_payment.psbt),
                                ::base64::STANDARD,
                            )
                            .to_string()
                            .as_bytes(),
                        )?;
                    }
                }
                psbt_file.flush()?;
                if output.is_none() {
                    eprintln!("\n");
                }
                if consignment.is_none() {
                    eprint!("{} ", "Consignment:".bright_yellow());
                }
                if let Some(data) = prepared_payment.consignment {
                    match consignment {
                        None => println!("{}", data),
                        Some(filename) => {
                            let file = fs::File::create(filename)?;
                            data.strict_encode(file)?;
                        }
                    }
                }
                Ok(())
            }
            InvoiceCommand::Accept { consignment, file } => {
                let consignment = if file {
                    unimplemented!()
                } else {
                    Consignment::from_str(&consignment)
                        .expect("bad consignment")
                };

                client.invoice_accept(consignment).map(|validation| {
                    match validation.validity() {
                        Validity::Valid => eprintln!(
                            "Transfer successfully validated & accepted. Please refresh wallet balance"
                        ),
                        Validity::UnresolvedTransactions => {
                            eprintln!(
                                "Transfer successfully validated, but not all of the transactions are mined.\n\
                             Please wait for the transaction to be mined and call the method once again.\n\
                             List of yet unmined transactions:"
                            );
                            for failure in validation.unresolved_txids {
                                eprintln!("- {}", failure)
                            }
                        },
                        Validity::Invalid => {
                            eprintln!("The provided consignment is invalid:");
                            for failure in validation.failures {
                                eprintln!("- {}", failure)
                            }
                        }
                    }
                })
            }
        }
    }
}
