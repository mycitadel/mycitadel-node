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

use std::collections::BTreeMap;

use super::Error;
use crate::model::{ContractId, Unspent};

pub trait Driver {
    fn unspent(
        &self,
        contract_id: ContractId,
    ) -> Result<BTreeMap<rgb::ContractId, Vec<Unspent>>, Error>;

    fn update(
        &mut self,
        contract_id: ContractId,
        updated_height: Option<u32>,
        unspent: BTreeMap<rgb::ContractId, Vec<Unspent>>,
    ) -> Result<(), Error>;
}
