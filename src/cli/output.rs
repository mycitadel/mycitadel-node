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
use serde::Serialize;
use std::collections::BTreeMap;
use std::fmt::Display;

use super::Formatting;
use crate::model::{Contract, Unspent};

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
            _ => eprintln!("Unsupported formatting option"),
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
        let headers = Self::output_headers();
        if format == Formatting::Tab {
            println!("{}", headers.join("\t").bright_green())
        } else if format == Formatting::Csv {
            println!("{}", headers.join(","))
        }

        let records = || {
            self.iter()
                .flat_map(|(id, vec)| {
                    let id = id.clone();
                    vec.iter().map(move |val| (id.clone(), val))
                })
                .collect::<BTreeMap<_, _>>()
        };
        match format {
            Formatting::Yaml => println!(
                "{}",
                serde_yaml::to_string(&records()).unwrap_or_default()
            ),

            Formatting::Json => println!(
                "{}",
                serde_json::to_string(&records()).unwrap_or_default()
            ),

            _ => self.iter().for_each(|(id, details)| match format {
                Formatting::Id => println!("{}", id),
                Formatting::Compact => {
                    println!("{}#{}", details.output_compact(), id)
                }
                Formatting::Tab => {
                    println!("{}\t{}", id, details.output_fields().join("\t"))
                }
                Formatting::Csv => {
                    println!("{},{}", id, details.output_fields().join(","))
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

// MARK: Contract --------------------------------------------------------------

impl OutputCompact for Contract {
    fn output_compact(&self) -> String {
        format!("{}", self.policy())
    }
}

impl OutputFormat for Contract {
    fn output_headers() -> Vec<String> {
        vec![s!("ID"), s!("Policy"), s!("Name"), s!("Created")]
    }

    fn output_id_string(&self) -> String {
        self.id().to_string()
    }

    fn output_fields(&self) -> Vec<String> {
        vec![
            self.id().to_string(),
            self.policy().to_string(),
            self.name().to_owned(),
            self.created_at().to_string(),
        ]
    }
}

// MARK: Unspent ---------------------------------------------------------------

impl OutputCompact for Unspent {
    fn output_compact(&self) -> String {
        self.to_string()
    }
}

impl OutputFormat for Unspent {
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
            self.index.to_string(),
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
        vec![
            self.ticker().to_owned(),
            self.name().to_owned(),
            self.id().to_string(),
            self.fractional_bits().to_string(),
            self.date().to_string(),
            self.accounting_supply(rgb20::SupplyMeasure::KnownCirculating)
                .to_string(),
            self.accounting_supply(rgb20::SupplyMeasure::IssueLimit)
                .to_string(),
        ]
    }
}
