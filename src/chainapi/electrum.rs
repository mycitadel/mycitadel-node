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

//! Module responsible for requesting blockchain data

use std::net::IpAddr;

use electrum_client::Client as ElectrumClient;

use super::Driver;
use bitcoin::{Address, Transaction, Txid};
use wallet::bip32::{PubkeyChain, UnhardenedIndex};
use wallet::descriptor::ContractDescriptor;

pub struct ElectrumConfig {
    pub electrum: IpAddr,
}

pub struct ElectrumDriver {
    electrum: ElectrumClient,
}

impl Driver for ElectrumDriver {
    fn transactions_by_txid(
        txid: Vec<Txid>,
        only_mined: bool,
    ) -> Vec<Transaction> {
        unimplemented!()
    }

    fn txid_by_address(address: Address) -> Vec<Txid> {
        unimplemented!()
    }

    fn txid_by_derived_descriptor(
        descriptor: ContractDescriptor<PubkeyChain>,
        index: UnhardenedIndex,
    ) -> Vec<Txid> {
        unimplemented!()
    }
}
