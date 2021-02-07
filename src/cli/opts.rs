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

use clap::{AppSettings, ArgGroup, Clap, ValueHint};
use wallet::bip32::PubkeyChain;

pub const MYCITADEL_CLI_CONFIG: &'static str = "{data_dir}/mycitadel-cli.toml";

#[derive(Clap, Clone, Debug)]
#[clap(
    name = "mycitadel-cli",
    bin_name = "mycitadel-cli",
    author,
    version,
    setting = AppSettings::ColoredHelp
)]
pub struct Opts {
    /// These params can be read also from the configuration file, not just
    /// command-line args or environment variables
    #[clap(flatten)]
    pub shared: crate::opts::SharedOpts,

    /// Path to the configuration file.
    ///
    /// NB: Command-line options override configuration file values.
    #[clap(
        short,
        long,
        default_value = MYCITADEL_CLI_CONFIG,
        env = "MYCITADEL_CLI_CONFIG",
        value_hint = ValueHint::FilePath
    )]
    pub config: String,

    /// Command to execute
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Clap, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display)]
pub enum Command {
    /// Wallet management commands
    #[display("wallet {subcommand}")]
    Wallet {
        #[clap(subcommand)]
        subcommand: WalletCommand,
    },

    /// Asset management commands
    #[display("asset {subcommand}")]
    Asset {
        #[clap(subcommand)]
        subcommand: AssetCommand,
    },
}

#[derive(Clap, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display)]
pub enum WalletCommand {
    /// Creates wallet with a given name and descriptor parameters
    #[display("create {subcommand}")]
    Create {
        #[clap(subcommand)]
        subcommand: WalletCreateCommand,
    },

    /// Lists existing wallets
    #[display("list")]
    List,
}

#[derive(Clap, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display)]
pub enum WalletCreateCommand {
    /// Creates current single-sig wallet account
    #[display("single-sig {name} {pubkey_chain}")]
    #[clap(group = ArgGroup::new("descriptor").required(false))]
    SingleSig {
        /// Wallet name
        #[clap()]
        name: String,

        /// Extended public key with derivation info.
        ///
        /// It should be a BIP32 derivation string which provides an extended
        /// public key value at the level after which no hardened
        /// derives is used. For instance,
        /// `m/84'/0'=[xpub661MyMwAqRbcFtXgS5sYJABqqG9YLmC4Q1Rdap9gSE8NqtwybGhePY2gZ29ESFjqJoCu1Rupje8YtGqsefD265TMg7usUDFdp6W1EGMcet8]/*`,
        /// or, simply
        /// `[xpub661MyMwAqRbcFtXgS5sYJABqqG9YLmC4Q1Rdap9gSE8NqtwybGhePY2gZ29ESFjqJoCu1Rupje8YtGqsefD265TMg7usUDFdp6W1EGMcet8]/*`,
        /// if you dont want your wallet to keep public key source information.
        ///
        /// You can use more advanced scenarios allowing full record of the
        /// key origin and extending derivation paths with range values:
        /// `![6734cda8]/84'/0'/1'
        /// =[xpub661MyMwAqRbcFtXgS5sYJABqqG9YLmC4Q1Rdap9gSE8NqtwybGhePY2gZ29ESFjqJoCu1Rupje8YtGqsefD265TMg7usUDFdp6W1EGMcet8]/
        /// 0-1/*`
        #[clap()]
        pubkey_chain: PubkeyChain,

        /// Creates old "bare" wallets, where public key is kept in the
        /// explicit form within bitcoin transaction P2PK output
        #[clap(long, takes_value = false, group = "descriptor")]
        bare: bool,

        /// Whether create a pre-SegWit wallet (P2PKH) rather than SegWit
        /// (P2WPKH). If you'd like to use legacy SegWit-style addresses
        /// (P2WPKH-in-P2SH), do not use this flag, create normal
        /// SegWit wallet instead and specify `--legacy` option when
        /// requesting new address
        #[clap(long, takes_value = false, group = "descriptor")]
        legacy: bool,

        /// Recommended SegWit wallet with P2WKH and P2WPKH-in-P2SH outputs
        #[clap(long, takes_value = false, group = "descriptor")]
        segwit: bool,

        /// Reserved for the future taproot P2TR outputs
        #[clap(long, takes_value = false, group = "descriptor")]
        taproot: bool,
    },

    /// Lists existing wallets
    #[display("list")]
    List,
}

#[derive(Clap, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display)]
pub enum AssetCommand {
    /// Lists known assets
    #[display("list")]
    List,

    /// Import asset genesis data
    #[display("import")]
    Import {
        /// Bech32-representation of the asset genesis (string starting with
        /// `genesis1....`
        #[clap()]
        genesis: String,
    },
}
