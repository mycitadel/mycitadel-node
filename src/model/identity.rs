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

mod deriver;

pub use deriver::*;

pub struct IdentityKey {
    pub identity_id: IdentityId,
    pub used_indexes: UsedIndexes,
    pub scope_xpub: ExtendedPubKey,
    pub scope_index: HardenedIndex,
}

pub struct CompatibleKey {
    pub xpub: ExtendedPubKey,
    pub key_source: KeySource,
    pub change_path: bool,
    pub used_indexes: UsedIndexes,
}
