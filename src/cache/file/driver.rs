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

use super::FileDriver;
use crate::cache::{Driver, Error};
use crate::model::{ContractId, Unspent};

impl Driver for FileDriver {
    fn unspent(&self, contract_id: ContractId) -> Result<Vec<Unspent>, Error> {
        self.cache
            .descriptors
            .get(&contract_id)
            .map(|c| c.unspent.clone())
            .ok_or(Error::NotFound(contract_id.to_string()))
    }
}
