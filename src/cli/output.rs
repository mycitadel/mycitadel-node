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

use chrono::NaiveDateTime;
use colored::Colorize;
use serde::Serialize;
use std::collections::{BTreeMap, HashMap};
use std::fmt::Display;

use amplify::Wrapper;
use bitcoin::hashes::{sha256t, Hash};
use invoice::Invoice;
use wallet::bip32::UnhardenedIndex;

use citadel::model::{AddressDerivation, ContractMeta, Utxo};

use super::Formatting;

pub trait OutputCompact {
    fn output_compact(&self) -> String;
}

pub trait OutputFormat: OutputCompact + Serialize {
    fn output_print(&self, format: Formatting) {
        match format {
            Formatting::Id => println!("{}", self.output_id_string()),
            Formatting::Compact => println!("{}", self.output_compact()),
            Formatting::Tab => println!("{}", self.output_fields().join("\t")),
            Formatting::Csv => println!("{}", self.output_fields().join(",")),
            Formatting::Yaml => {
                println!("{}", serde_yaml::to_string(self).unwrap_or_default())
            }
            Formatting::Json => {
                println!("{}", serde_json::to_string(self).unwrap_or_default())
            }
        }
    }

    fn output_headers() -> Vec<String>;
    fn output_id_string(&self) -> String;
    fn output_fields(&self) -> Vec<String>;
}

#[doc(hidden)]
impl<T> OutputCompact for Vec<T>
where
    T: OutputCompact,
{
    fn output_compact(&self) -> String {
        unreachable!()
    }
}

impl<T> OutputFormat for Vec<T>
where
    T: OutputFormat,
{
    fn output_print(&self, format: Formatting) {
        if self.is_empty() {
            eprintln!("{}", "No items".red());
            return;
        }
        let headers = T::output_headers();
        if format == Formatting::Tab {
            println!("{}", headers.join("\t").bright_green())
        } else if format == Formatting::Csv {
            println!("{}", headers.join(","))
        }
        self.iter().for_each(|t| t.output_print(format));
    }

    #[doc(hidden)]
    fn output_id_string(&self) -> String {
        unreachable!()
    }

    #[doc(hidden)]
    fn output_headers() -> Vec<String> {
        unreachable!()
    }

    #[doc(hidden)]
    fn output_fields(&self) -> Vec<String> {
        unreachable!()
    }
}

impl<K, V> OutputCompact for HashMap<K, V>
where
    K: Display,
    V: OutputCompact,
{
    fn output_compact(&self) -> String {
        unimplemented!()
    }
}

impl<K, V> OutputFormat for HashMap<K, V>
where
    K: Clone + Display + std::hash::Hash + Eq + Serialize,
    V: OutputFormat + Serialize,
{
    fn output_print(&self, format: Formatting) {
        if self.is_empty() {
            eprintln!("{}", "No items".red());
            return;
        }
        let headers = Self::output_headers();
        if format == Formatting::Tab {
            println!("{}", headers.join("\t").bright_green())
        } else if format == Formatting::Csv {
            println!("{}", headers.join(","))
        }

        match format {
            Formatting::Yaml => {
                println!("{}", serde_yaml::to_string(self).unwrap_or_default())
            }

            Formatting::Json => {
                println!("{}", serde_json::to_string(self).unwrap_or_default())
            }

            _ => self.iter().for_each(|(id, rec)| match format {
                Formatting::Id => println!("{}", id),
                Formatting::Compact => {
                    println!("{}#{}", rec.output_compact(), id)
                }
                Formatting::Tab => {
                    println!("{}\t{}", id, rec.output_fields().join("\t"))
                }
                Formatting::Csv => {
                    println!("{},{}", id, rec.output_fields().join(","))
                }
                _ => unreachable!(),
            }),
        }
    }

    fn output_headers() -> Vec<String> {
        let mut vec = vec![s!("ID")];
        vec.extend(V::output_headers());
        vec
    }

    #[doc(hidden)]
    fn output_id_string(&self) -> String {
        unreachable!()
    }

    #[doc(hidden)]
    fn output_fields(&self) -> Vec<String> {
        unreachable!()
    }
}

impl<K, V> OutputCompact for BTreeMap<K, Vec<V>>
where
    K: Display,
    V: OutputCompact,
{
    fn output_compact(&self) -> String {
        unimplemented!()
    }
}

impl<K, V> OutputFormat for BTreeMap<K, Vec<V>>
where
    K: Clone + Display + Ord + Serialize,
    V: OutputFormat + Ord + Serialize,
{
    fn output_print(&self, format: Formatting) {
        if self.values().all(Vec::is_empty) {
            eprintln!("{}", "No items".red());
            return;
        }
        let headers = Self::output_headers();
        if format == Formatting::Tab {
            println!("{}", headers.join("\t").bright_green())
        } else if format == Formatting::Csv {
            println!("{}", headers.join(","))
        }

        match format {
            Formatting::Yaml => {
                println!("{}", serde_yaml::to_string(self).unwrap_or_default())
            }

            Formatting::Json => {
                println!("{}", serde_json::to_string(self).unwrap_or_default())
            }

            _ => self.iter().for_each(|(id, details)| {
                let id = id.to_string().as_str().bright_white();
                details.iter().for_each(|rec| match format {
                    Formatting::Id => println!("{}", id),
                    Formatting::Compact => {
                        println!("{}#{}", rec.output_compact(), id)
                    }
                    Formatting::Tab => {
                        println!("{}\t{}", id, rec.output_fields().join("\t"))
                    }
                    Formatting::Csv => {
                        println!("{},{}", id, rec.output_fields().join(","))
                    }
                    _ => unreachable!(),
                })
            }),
        }
    }

    fn output_headers() -> Vec<String> {
        let mut vec = vec![s!("ID")];
        vec.extend(V::output_headers());
        vec
    }

    #[doc(hidden)]
    fn output_id_string(&self) -> String {
        unreachable!()
    }

    #[doc(hidden)]
    fn output_fields(&self) -> Vec<String> {
        unreachable!()
    }
}

// MARK: Contract --------------------------------------------------------------

impl OutputCompact for ContractMeta {
    fn output_compact(&self) -> String {
        format!("{}", self.policy())
    }
}

impl OutputFormat for ContractMeta {
    fn output_headers() -> Vec<String> {
        vec![s!("ID"), s!("Policy"), s!("Name"), s!("Created")]
    }

    fn output_id_string(&self) -> String {
        self.id().to_string()
    }

    fn output_fields(&self) -> Vec<String> {
        vec![
            self.id().to_string().as_str().bright_white().to_string(),
            self.policy().to_string(),
            self.name().to_owned(),
            self.created_at().to_string(),
        ]
    }
}

// MARK: UnhardenedIndex -------------------------------------------------------

impl OutputCompact for UnhardenedIndex {
    fn output_compact(&self) -> String {
        self.to_string()
    }
}

impl OutputFormat for UnhardenedIndex {
    fn output_headers() -> Vec<String> {
        vec![s!("ID"), s!("Index")]
    }

    fn output_id_string(&self) -> String {
        self.to_string()
    }

    fn output_fields(&self) -> Vec<String> {
        vec![self.to_string()]
    }
}

// MARK: Unspent ---------------------------------------------------------------

impl OutputCompact for Utxo {
    fn output_compact(&self) -> String {
        self.to_string()
    }
}

impl OutputFormat for Utxo {
    fn output_id_string(&self) -> String {
        format!("{}", self.value)
    }

    fn output_headers() -> Vec<String> {
        vec![
            s!("Value"),
            s!("Block height"),
            s!("Block tx offset"),
            s!("Output no"),
            s!("Derivation index"),
        ]
    }

    fn output_fields(&self) -> Vec<String> {
        vec![
            self.value.to_string(),
            self.height.to_string(),
            self.offset.to_string(),
            self.vout.to_string(),
            self.derivation_index.to_string(),
        ]
    }
}

// MARK: AddressDerivation -----------------------------------------------------

impl OutputCompact for AddressDerivation {
    fn output_compact(&self) -> String {
        self.address.to_string()
    }
}

impl OutputFormat for AddressDerivation {
    fn output_headers() -> Vec<String> {
        vec![s!("Address"), s!("Derivation index")]
    }

    fn output_id_string(&self) -> String {
        self.address.to_string()
    }

    fn output_fields(&self) -> Vec<String> {
        vec![
            self.address.to_string(),
            self.derivation
                .last()
                .expect("derivation path must has at least one element")
                .to_string(),
        ]
    }
}

// MARK: Asset -----------------------------------------------------------------

impl OutputCompact for rgb20::Asset {
    fn output_compact(&self) -> String {
        format!("{}#{}", self.ticker(), self.id())
    }
}

impl OutputFormat for rgb20::Asset {
    fn output_headers() -> Vec<String> {
        vec![
            s!("Ticker"),
            s!("Name"),
            s!("Id"),
            s!("Precision"),
            s!("Issue date"),
            s!("In circulation"),
            s!("Inflation cap."),
        ]
    }

    fn output_id_string(&self) -> String {
        self.id().to_string()
    }

    fn output_fields(&self) -> Vec<String> {
        let bitcoin_id = rgb::ContractId::from_inner(
            sha256t::Hash::from_inner(wallet::BITCOIN_GENESIS_BLOCKHASH.into()),
        );
        if *self.id() == default!() {
            return vec![
                s!("BTC").as_str().bright_yellow().to_string(),
                s!("Bitcoin"),
                bitcoin_id.to_string().as_str().bright_white().to_string(),
                s!("2009-01-03 19:15:00"),
                s!(">~18624337 BTC"),
                s!("21000000 BTC"),
            ];
        }
        vec![
            self.ticker()
                .to_owned()
                .as_str()
                .bright_yellow()
                .to_string(),
            self.name().to_owned(),
            self.id().to_string().as_str().bright_white().to_string(),
            self.decimal_precision().to_string(),
            self.date().to_string(),
            self.accounting_supply(rgb20::SupplyMeasure::KnownCirculating)
                .to_string(),
            self.accounting_supply(rgb20::SupplyMeasure::IssueLimit)
                .to_string(),
        ]
    }
}

// MARK: Invoice ---------------------------------------------------------------

impl OutputCompact for Invoice {
    fn output_compact(&self) -> String {
        self.beneficiary().to_string()
    }
}

impl OutputFormat for Invoice {
    fn output_headers() -> Vec<String> {
        vec![
            s!("Invoice"),
            s!("No beneficiaries"),
            s!("First beneficiary"),
            s!("Amount"),
            s!("Asset"),
            s!("Recurrent"),
            s!("Expiry"),
            s!("Merchant"),
            s!("Purpose"),
        ]
    }

    fn output_id_string(&self) -> String {
        self.to_string()
    }

    fn output_fields(&self) -> Vec<String> {
        vec![
            self.to_string().as_str().bright_white().to_string(),
            self.beneficiaries().count().to_string(),
            self.output_compact(),
            self.amount().to_string(),
            self.asset()
                .map(|asset_id| {
                    rgb::ContractId::from_inner(sha256t::Hash::from_inner(
                        asset_id.into_inner(),
                    ))
                    .to_string()
                })
                .unwrap_or(s!("BTC")),
            self.recurrent().to_string(),
            self.expiry()
                .as_ref()
                .map(NaiveDateTime::to_string)
                .unwrap_or(s!("-")),
            self.merchant().clone().unwrap_or(s!("-")),
            self.purpose().clone().unwrap_or(s!("-")),
        ]
    }
}
