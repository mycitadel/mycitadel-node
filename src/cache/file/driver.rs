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

use std::collections::{BTreeMap, HashSet};

use bitcoin::{Address, OutPoint};
use wallet::bip32::UnhardenedIndex;

use super::FileDriver;
use crate::cache::{Driver, Error};
use crate::model::{ContractId, Unspent};

impl Driver for FileDriver {
    fn unspent(
        &self,
        contract_id: ContractId,
    ) -> Result<BTreeMap<rgb::ContractId, Vec<Unspent>>, Error> {
        self.map_contract_or_default(contract_id, |c| c.unspent.clone())
    }

    fn utxo(
        &self,
        contract_id: ContractId,
    ) -> Result<HashSet<OutPoint>, Error> {
        self.map_contract_or_default(contract_id, |cache| cache.utxo.clone())
    }

    fn update(
        &mut self,
        contract_id: ContractId,
        updated_height: Option<u32>,
        utxo: Vec<OutPoint>,
        unspent: BTreeMap<rgb::ContractId, Vec<Unspent>>,
    ) -> Result<(), Error> {
        let cache = self
            .cache
            .descriptors
            .entry(contract_id)
            .or_insert(default!());
        cache.unspent = unspent;
        cache.utxo = utxo.into_iter().collect();
        if let Some(height) = updated_height {
            self.cache.known_height = height;
            cache.updated_height = height;
        }
        self.store()
    }

    fn used_address_derivations(
        &self,
        contract_id: ContractId,
    ) -> Result<BTreeMap<Address, UnhardenedIndex>, Error> {
        self.map_contract_or_default(contract_id, |cache| {
            cache.used_address_derivations.clone()
        })
    }

    fn used_addresses(
        &self,
        contract_id: ContractId,
    ) -> Result<HashSet<Address>, Error> {
        self.map_contract_or_default(contract_id, |cache| {
            cache
                .used_address_derivations
                .iter()
                .map(|(address, _)| address)
                .cloned()
                .collect()
        })
    }

    fn used_derivations(
        &self,
        contract_id: ContractId,
    ) -> Result<HashSet<UnhardenedIndex>, Error> {
        self.map_contract_or_default(contract_id, |cache| {
            cache
                .used_address_derivations
                .iter()
                .map(|(_, derivation)| derivation)
                .copied()
                .collect()
        })
    }

    fn next_unused_derivation(
        &self,
        contract_id: ContractId,
    ) -> Result<UnhardenedIndex, Error> {
        self.map_contract_or_default(contract_id, |cache| {
            cache
                .used_address_derivations
                .values()
                .max()
                .copied()
                .unwrap_or_default()
        })
    }

    fn use_address_derivation(
        &mut self,
        contract_id: ContractId,
        address: Address,
        path: UnhardenedIndex,
    ) -> Result<bool, Error> {
        self.with_contract(contract_id, |cache| {
            if cache
                .used_address_derivations
                .get(&address)
                .map(|p| p != &path)
                .unwrap_or(false)
            {
                Err(Error::WrongDerivation)
            } else {
                Ok(cache
                    .used_address_derivations
                    .insert(address, path)
                    .is_none())
            }
        })
    }

    fn forget_address(
        &mut self,
        contract_id: ContractId,
        address: &Address,
    ) -> Result<bool, Error> {
        self.with_contract(contract_id, |cache| {
            Ok(cache.used_address_derivations.remove(&address).is_some())
        })
    }

    fn address_derivation(
        &self,
        contract_id: ContractId,
        address: &Address,
    ) -> Option<UnhardenedIndex> {
        self.cache.descriptors.get(&contract_id).and_then(|cache| {
            cache.used_address_derivations.get(address).cloned()
        })
    }
}
