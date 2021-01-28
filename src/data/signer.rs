// MyCitadel: node, wallet library & command-line tool
// Written in 2020 by
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

use bitcoin::hashes::{sha256, sha256t};
use lnpbp::bech32::ToBech32IdString;
use lnpbp::commit_verify::CommitVerify;
use lnpbp::tagged_hash::{self, TaggedHash};

// --- Signer identifiers

pub struct SignerIdTag;

impl sha256t::Tag for SignerIdTag {
    #[inline]
    fn engine() -> sha256::HashEngine {
        let midstate = sha256::Midstate::from_inner(
            **tagged_hash::Midstate::with("mycitadel:signer"),
        );
        sha256::HashEngine::from_midstate(midstate, 64)
    }
}

#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate")
)]
#[derive(
    Wrapper,
    Copy,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Display,
    Default,
    From,
    StrictEncode,
    StrictDecode,
)]
#[wrapper(
    Debug, FromStr, LowerHex, Index, IndexRange, IndexFrom, IndexTo, IndexFull
)]
#[display(SignerId::to_bech32_id_string)]
pub struct SignerId(sha256t::Hash<SignerIdTag>);

impl<MSG> CommitVerify<MSG> for SignerId
where
    MSG: AsRef<[u8]>,
{
    #[inline]
    fn commit(msg: &MSG) -> SignerId {
        <SignerId as TaggedHash<_>>::hash(msg)
    }
}
