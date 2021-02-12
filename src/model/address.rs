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

use bitcoin::Address;
use wallet::bip32::UnhardenedIndex;

#[derive(
    Serialize,
    Deserialize,
    Clone,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    Debug,
    Display,
    StrictEncode,
    StrictDecode,
)]
#[display("{address}")]
pub struct AddressDerivation {
    pub address: Address,
    pub derivation: Vec<UnhardenedIndex>,
}

impl AddressDerivation {
    pub fn with(
        address: Address,
        derivation: Vec<UnhardenedIndex>,
    ) -> AddressDerivation {
        AddressDerivation {
            address,
            derivation,
        }
    }
}
