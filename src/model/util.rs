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

use bitcoin::BlockHash;
use chrono::NaiveDateTime;
use std::str::FromStr;

use lnp::ChannelId;

/// Error parsing string representation of wallet data/structure
#[derive(
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Debug,
    Display,
    From,
    Error,
)]
#[display(doc_comments)]
#[from(bitcoin::hashes::hex::Error)]
#[from(chrono::ParseError)]
#[from(std::num::ParseIntError)]
pub struct FromStrError;

// TODO: Consider moving to descriptor wallet lib, BPro lib, LNP/BP Core lib
#[derive(
    Getters,
    Clone,
    Ord,
    PartialOrd,
    Eq,
    PartialEq,
    Hash,
    Debug,
    Display,
    StrictEncode,
    StrictDecode,
)]
#[display("{block_height}:{block_hash}@{timestamp}")]
pub struct BlockchainTimepair {
    timestamp: NaiveDateTime,
    block_height: u32,
    block_hash: BlockHash,
}

impl FromStr for BlockchainTimepair {
    type Err = FromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut data = s.split(&[':', '@'][..]);
        let me = Self {
            timestamp: data.next().ok_or(FromStrError)?.parse()?,
            block_height: data.next().ok_or(FromStrError)?.parse()?,
            block_hash: data.next().ok_or(FromStrError)?.parse()?,
        };
        if data.next().is_some() {
            Err(FromStrError)
        } else {
            Ok(me)
        }
    }
}

// TODO: Consider moving to descriptor wallet lib, BPro lib, LNP/BP Core lib
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate")
)]
#[derive(
    Clone,
    Ord,
    PartialOrd,
    Eq,
    PartialEq,
    Hash,
    Debug,
    Display,
    StrictEncode,
    StrictDecode,
)]
pub enum TxConfirmation {
    #[display("{block_height}:{block_hash}")]
    Blockchain {
        block_height: u32,
        block_hash: BlockHash,
    },

    #[display("{state_no}@{channel_id}")]
    Lightning {
        channel_id: ChannelId,
        state_no: u64,
    },
}

impl FromStr for TxConfirmation {
    type Err = FromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut data = s.split(&[':', '@'][..]);
        let me = if s.contains(':') {
            TxConfirmation::Blockchain {
                block_height: data.next().ok_or(FromStrError)?.parse()?,
                block_hash: data.next().ok_or(FromStrError)?.parse()?,
            }
        } else {
            TxConfirmation::Lightning {
                channel_id: data.next().ok_or(FromStrError)?.parse()?,
                state_no: data.next().ok_or(FromStrError)?.parse()?,
            }
        };
        if data.next().is_some() {
            Err(FromStrError)
        } else {
            Ok(me)
        }
    }
}
