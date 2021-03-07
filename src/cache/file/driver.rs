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

use std::collections::{BTreeMap, BTreeSet, HashSet};

use bitcoin::{Address, OutPoint, Txid};
use wallet::bip32::{ChildIndex, UnhardenedIndex};

use super::FileDriver;
use crate::cache::{Driver, Error};
use crate::model::{Allocations, ContractId, Utxo};

impl Driver for FileDriver {
    fn blockpos_to_txid(&self, height: u32, offset: u16) -> Option<Txid> {
        self.cache.mine_info.get(&(height, offset)).copied()
    }

    fn unspent(
        &self,
        contract_id: ContractId,
    ) -> Result<BTreeMap<rgb::ContractId, HashSet<Utxo>>, Error> {
        self.map_contract_or_default(contract_id, |c| c.unspent.clone())
    }

    fn unspent_bitcoin_only(
        &self,
        contract_id: ContractId,
    ) -> Result<HashSet<Utxo>, Error> {
        let unspent = self.unspent(contract_id)?;
        let outpoints = self
            .allocations(contract_id)?
            .into_iter()
            .filter_map(|(outpoint, mut assets)| {
                // Removing bitcoins from accounting
                assets.remove(&rgb::ContractId::default());
                if assets.values().sum::<u64>() == 0 {
                    Some(outpoint)
                } else {
                    None
                }
            })
            .collect::<BTreeSet<_>>();
        Ok(unspent
            .get(&rgb::ContractId::default())
            .map(|utxo_set| {
                utxo_set
                    .into_iter()
                    .filter(|utxo| outpoints.contains(&utxo.outpoint()))
                    .map(Utxo::clone)
                    .collect()
            })
            .unwrap_or_default())
    }

    fn allocations(
        &self,
        contract_id: ContractId,
    ) -> Result<Allocations, Error> {
        self.map_contract_or_default(contract_id, |cache| {
            cache.unspent.iter().fold(
                Allocations::new(),
                |mut allocations, (asset_id, utxos)| {
                    for utxo in utxos {
                        *allocations
                            .entry(utxo.outpoint())
                            .or_insert(default!())
                            .entry(*asset_id)
                            .or_insert(0) += utxo.value;
                    }
                    allocations
                },
            )
        })
    }

    fn utxo(
        &self,
        contract_id: ContractId,
    ) -> Result<BTreeSet<OutPoint>, Error> {
        self.map_contract_or_default(contract_id, |cache| cache.utxo.clone())
    }

    fn update(
        &mut self,
        contract_id: ContractId,
        mine_info: BTreeMap<(u32, u16), Txid>,
        updated_height: Option<u32>,
        utxo: BTreeSet<OutPoint>,
        unspent: BTreeMap<rgb::ContractId, Vec<Utxo>>,
    ) -> Result<(), Error> {
        self.cache.mine_info.extend(mine_info);
        let cache = self
            .cache
            .descriptors
            .entry(contract_id)
            .or_insert(default!());
        cache.unspent = unspent
            .into_iter()
            .map(|(asset_id, utxos)| {
                (
                    asset_id,
                    utxos.into_iter().filter(|utxo| utxo.value > 0).collect(),
                )
            })
            .collect();
        cache.utxo = utxo;
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
                .and_then(UnhardenedIndex::checked_inc)
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

    fn last_used_derivation(
        &self,
        contract_id: ContractId,
    ) -> Option<UnhardenedIndex> {
        let cache = self.cache.descriptors.get(&contract_id)?;
        Some(
            cache
                .used_address_derivations
                .values()
                .copied()
                .max_by_key(|index| index.clone())
                .unwrap_or_default(),
        )
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
