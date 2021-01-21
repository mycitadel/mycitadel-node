// Keyring: private/public key managing service
// Written in 2020 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the AGPL License
// along with this software.
// If not, see <https://www.gnu.org/licenses/agpl-3.0-standalone.html>.

use std::io;
use std::ops::RangeInclusive;

use lnpbp::strict_encoding::{self, StrictDecode, StrictEncode};
use wallet::descriptor;

#[derive(Clone, Eq, PartialEq, Hash, Debug, Display)]
#[display("{key}")]
pub struct SignerAccount {
    pub title: String,
    pub key: descriptor::SingleSig,
    pub used: Vec<RangeInclusive<u32>>,
}

impl StrictEncode for SignerAccount {
    fn strict_encode<E: io::Write>(
        &self,
        mut e: E,
    ) -> Result<usize, strict_encoding::Error> {
        let len = strict_encode_list!(e; self.title, self.key);
        let ranges = self
            .used
            .iter()
            .map(|range| (*range.start(), *range.end()))
            .collect::<Vec<_>>();
        Ok(len + ranges.strict_encode(&mut e)?)
    }
}

impl StrictDecode for SignerAccount {
    fn strict_decode<D: io::Read>(
        mut d: D,
    ) -> Result<Self, strict_encoding::Error> {
        Ok(Self {
            title: StrictDecode::strict_decode(&mut d)?,
            key: StrictDecode::strict_decode(&mut d)?,
            used: Vec::<(u32, u32)>::strict_decode(&mut d)?
                .into_iter()
                .map(|(start, end)| RangeInclusive::new(start, end))
                .collect(),
        })
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug, Display)]
#[display("{key}")]
pub struct IdentityInfo {
    pub name: String,
    pub key: descriptor::SingleSig,
    pub known: Vec<RangeInclusive<u32>>,
}

impl StrictEncode for IdentityInfo {
    fn strict_encode<E: io::Write>(
        &self,
        mut e: E,
    ) -> Result<usize, strict_encoding::Error> {
        let len = strict_encode_list!(e; self.name, self.key);
        let ranges = self
            .known
            .iter()
            .map(|range| (*range.start(), *range.end()))
            .collect::<Vec<_>>();
        Ok(len + ranges.strict_encode(&mut e)?)
    }
}

impl StrictDecode for IdentityInfo {
    fn strict_decode<D: io::Read>(
        mut d: D,
    ) -> Result<Self, strict_encoding::Error> {
        Ok(Self {
            name: StrictDecode::strict_decode(&mut d)?,
            key: StrictDecode::strict_decode(&mut d)?,
            known: Vec::<(u32, u32)>::strict_decode(&mut d)?
                .into_iter()
                .map(|(start, end)| RangeInclusive::new(start, end))
                .collect(),
        })
    }
}
