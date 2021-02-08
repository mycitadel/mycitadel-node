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

#[cfg(feature = "serde")]
use serde_with::{As, DisplayFromStr};
use wallet::Slice32;

#[cfg_attr(
    feature = "serde",
    serde_as,
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate")
)]
#[derive(
    Copy,
    Clone,
    Ord,
    PartialOrd,
    Eq,
    PartialEq,
    Hash,
    Default,
    Debug,
    Display,
    StrictEncode,
    StrictDecode,
)]
#[display("<internal state data>")]
pub struct State {
    #[cfg_attr(feature = "serde", serde(with = "As::<DisplayFromStr>"))]
    pub mu_sig: Slice32,
}
