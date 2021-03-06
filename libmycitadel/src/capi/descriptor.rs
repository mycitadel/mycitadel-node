// C library for building descriptor-based bitcoin wallets
//
// Written in 2021 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the Apache 2.0 License
// along with this software.
// If not, see <https://opensource.org/licenses/Apache-2.0>.

use libc::c_char;
use serde_with::DisplayFromStr;
use std::convert::{TryFrom, TryInto};
use std::ffi::CStr;
use std::str::FromStr;

use bitcoin::hashes::hex::ToHex;
use bitcoin::util::address::{Address, Payload};
use bitcoin::util::bip32::{ExtendedPubKey, Fingerprint};
use bitcoin::{OutPoint, XpubIdentifier};
use miniscript::descriptor::{Descriptor, DescriptorType, ShInner, WshInner};
use miniscript::{ForEach, ForEachKey};
use wallet::bip32::{BranchStep, PubkeyChain, TerminalStep, XpubRef};
use wallet::descriptor::FullType;
use wallet::{
    descriptor, AddressFormat, AddressNetwork, AddressPayload, WitnessVersion,
};

use super::signer::err_type;
use super::string_result_t;

#[serde_as]
#[derive(
    Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Serialize, Deserialize,
)]
#[serde(rename_all = "camelCase")]
pub struct DescriptorInfo {
    pub full_type: descriptor::FullType,
    pub outer_type: descriptor::OuterType,
    pub inner_type: descriptor::InnerType,
    pub content_type: descriptor::ContentType,
    pub descr_type: String,
    #[serde_as(as = "Option<_>")]
    pub addr_type: Option<String>,
    pub category: descriptor::Category,
    pub is_nestable: bool,
    pub is_sorted: bool,
    pub descriptor: String,
    pub policy: String,
    #[serde_as(as = "Option<_>")]
    pub sigs_required: Option<usize>,
    pub keys: Vec<PubkeyChainInfo>,
    pub keyspace_size: usize,
    #[serde_as(as = "Option<_>")]
    pub checksum: Option<String>,
}

#[serde_as]
#[derive(
    Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Serialize, Deserialize,
)]
#[serde(rename_all = "camelCase")]
pub struct XpubRefInfo {
    pub fingerprint: Fingerprint,
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub identifier: Option<XpubIdentifier>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub xpubkey: Option<ExtendedPubKey>,
}

#[serde_as]
#[derive(
    Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Serialize, Deserialize,
)]
#[serde(rename_all = "camelCase")]
pub struct XpubInfo {
    pub fingerprint: Fingerprint,
    pub identifier: XpubIdentifier,
    #[serde_as(as = "DisplayFromStr")]
    pub xpubkey: ExtendedPubKey,
}

#[serde_as]
#[derive(
    Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Serialize, Deserialize,
)]
#[serde(rename_all = "camelCase")]
pub struct PubkeyChainInfo {
    pub full_key: String,
    pub seed_based: bool,
    #[serde_as(as = "Option<_>")]
    pub master: Option<XpubRefInfo>,
    #[serde_as(as = "Vec<_>")]
    pub source_path: Vec<BranchStep>,
    pub branch: XpubInfo,
    #[serde_as(as = "Option<_>")]
    pub revocation_seal: Option<OutPoint>,
    #[serde_as(as = "Vec<DisplayFromStr>")]
    pub terminal_path: Vec<TerminalStep>,
    pub keyspace_size: usize,
}

impl TryFrom<XpubRef> for XpubRefInfo {
    type Error = ();

    fn try_from(xpub_ref: XpubRef) -> Result<Self, Self::Error> {
        Ok(XpubRefInfo {
            fingerprint: xpub_ref.fingerprint().ok_or(())?,
            identifier: xpub_ref.identifier(),
            xpubkey: xpub_ref.xpubkey(),
        })
    }
}

impl From<ExtendedPubKey> for XpubInfo {
    fn from(xpub: ExtendedPubKey) -> Self {
        XpubInfo {
            fingerprint: xpub.fingerprint(),
            identifier: xpub.identifier(),
            xpubkey: xpub,
        }
    }
}

impl From<PubkeyChain> for PubkeyChainInfo {
    fn from(pubkey_chain: PubkeyChain) -> Self {
        PubkeyChainInfo {
            full_key: pubkey_chain.to_string(),
            keyspace_size: pubkey_chain.keyspace_size(),
            seed_based: pubkey_chain.seed_based,
            master: pubkey_chain.master.try_into().ok(),
            source_path: pubkey_chain.source_path,
            branch: pubkey_chain.branch_xpub.into(),
            revocation_seal: pubkey_chain.revocation_seal,
            terminal_path: pubkey_chain.terminal_path,
        }
    }
}

#[serde_as]
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AddressInfo {
    #[serde_as(as = "DisplayFromStr")]
    pub address: Address,
    #[serde(rename = "isBIP21")]
    pub is_bip21: bool,
    #[serde_as(as = "DisplayFromStr")]
    pub network: AddressNetwork,
    pub payload: String,
    #[serde_as(as = "Option<_>")]
    pub value: Option<u64>,
    #[serde_as(as = "Option<_>")]
    pub label: Option<String>,
    #[serde_as(as = "Option<_>")]
    pub message: Option<String>,
    #[serde_as(as = "DisplayFromStr")]
    pub format: AddressFormat,
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub witness_ver: Option<WitnessVersion>,
}

impl From<Address> for AddressInfo {
    fn from(address: Address) -> Self {
        let format = AddressFormat::from(address.clone());
        let payload = AddressPayload::try_from(address.payload.clone())
            .as_ref()
            .map(AddressPayload::to_string)
            .unwrap_or(match &address.payload {
                Payload::WitnessProgram { version, program } => {
                    format!("wp{}:{}", version.to_u8(), program.to_hex())
                }
                _ => unreachable!(),
            });

        AddressInfo {
            network: address.clone().into(),
            is_bip21: false,
            address,
            payload,
            value: None,
            label: None,
            message: None,
            format,
            witness_ver: format.witness_version(),
        }
    }
}

impl FromStr for AddressInfo {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(address) = Address::from_str(s) {
            return Ok(address.into());
        }
        if !s.to_lowercase().starts_with("bitcoin:") {
            return Err(());
        }
        let s = &s[8..];
        let mut split = s.split('?');
        let address = if let Some(addr) = split.next() {
            Address::from_str(addr).map_err(|_| ())?
        } else {
            return Err(());
        };
        let mut info = AddressInfo::from(address);
        if let Some(params) = split.next() {
            info.is_bip21 = true;
            for arg in params.split('&') {
                let mut split = arg.split('=');
                match (split.next(), split.next(), split.next()) {
                    (Some("amount"), Some(value), None) => {
                        let amount: f64 = value.parse().map_err(|_| ())?;
                        info.value = Some((amount * 100_000_000.0) as u64);
                    }
                    (Some("label"), Some(value), None) => {
                        info.label = Some(value.to_owned())
                    }
                    (Some("message"), Some(value), None) => {
                        info.message = Some(value.to_owned())
                    }
                    (Some("label"), ..)
                    | (Some("message"), ..)
                    | (Some("amount"), ..) => return Err(()),
                    _ => (),
                }
            }
        }
        Ok(info)
    }
}

#[no_mangle]
pub extern "C" fn lnpbp_address_parse(
    address: *const c_char,
) -> string_result_t {
    if address.is_null() {
        Err(err_type::null_pointer)?
    }
    let address = unsafe { CStr::from_ptr(address).to_str()? };
    let info =
        AddressInfo::from_str(address).map_err(|_| err_type::parse_error)?;
    let json = serde_json::to_string(&info)?;
    string_result_t::success(&json)
}

#[no_mangle]
pub extern "C" fn lnpbp_descriptor_parse(
    descriptor: *const c_char,
) -> string_result_t {
    if descriptor.is_null() {
        Err(err_type::null_pointer)?
    }
    let descriptor = unsafe { CStr::from_ptr(descriptor).to_str()? };
    let descriptor = Descriptor::<PubkeyChain>::from_str(descriptor)?;

    let mut sigs_required = Some(1);
    let policy = match &descriptor {
        Descriptor::Bare(bare) => {
            sigs_required = None;
            bare.as_inner().to_string()
        }
        Descriptor::Pkh(pkh) => pkh.as_inner().to_string(),
        Descriptor::Wpkh(wpkh) => wpkh.as_inner().to_string(),
        Descriptor::Sh(sh) => match sh.as_inner() {
            ShInner::Wsh(wsh) => match wsh.as_inner() {
                WshInner::SortedMulti(sm) => {
                    sigs_required = Some(sm.k);
                    sm.to_string()
                }
                WshInner::Ms(ms) => {
                    sigs_required = None;
                    ms.as_inner().to_string()
                }
            },
            ShInner::Wpkh(wpkh) => wpkh.as_inner().to_string(),
            ShInner::SortedMulti(sm) => {
                sigs_required = Some(sm.k);
                sm.to_string()
            }
            ShInner::Ms(ms) => ms.to_string(),
        },
        Descriptor::Wsh(wsh) => match wsh.as_inner() {
            WshInner::SortedMulti(sm) => {
                sigs_required = Some(sm.k);
                sm.to_string()
            }
            WshInner::Ms(ms) => {
                sigs_required = None;
                ms.as_inner().to_string()
            }
        },
    };

    let mut keys = vec![];
    let mut keyspace_size = usize::MAX;
    descriptor.for_each_key(|for_each| {
        match for_each {
            ForEach::Key(pubkey_chain) => {
                keys.push(pubkey_chain.clone().into());
                keyspace_size = keyspace_size.min(pubkey_chain.keyspace_size());
            }
            ForEach::Hash(_) => unreachable!(),
        }
        true
    });

    let descr_type = descriptor.desc_type();
    let full_type = descriptor.clone().into();
    let descriptor = descriptor.to_string();
    let checksum = descriptor.split('#').last().map(str::to_owned);

    let (is_sorted, is_nestable) = match descr_type {
        DescriptorType::Bare
        | DescriptorType::Sh
        | DescriptorType::Pkh
        | DescriptorType::ShWsh
        | DescriptorType::ShWpkh => (false, false),
        DescriptorType::Wpkh | DescriptorType::Wsh => (false, true),
        DescriptorType::ShSortedMulti | DescriptorType::ShWshSortedMulti => {
            (true, false)
        }
        DescriptorType::WshSortedMulti => (true, true),
    };

    let (descr_type, addr_type) = match descr_type {
        DescriptorType::Bare => (
            if full_type == FullType::Pk {
                "Pk"
            } else {
                "Pkh"
            },
            None,
        ),
        DescriptorType::Sh | DescriptorType::ShSortedMulti => {
            ("Sh", Some("P2SH"))
        }
        DescriptorType::Pkh => ("Pkh", Some("P2PKH")),
        DescriptorType::Wpkh => ("Wpkh", Some("P2WPKH")),
        DescriptorType::Wsh | DescriptorType::WshSortedMulti => {
            ("Wsh", Some("P2WSH"))
        }
        DescriptorType::ShWpkh => ("ShWpkh", Some("P2WPKH-in-P2SH")),
        DescriptorType::ShWsh | DescriptorType::ShWshSortedMulti => {
            ("ShWsh", Some("P2WSH-in-P2SH"))
        }
    };

    let info = DescriptorInfo {
        full_type,
        outer_type: full_type.into(),
        inner_type: full_type.into(),
        content_type: full_type.into(),
        category: full_type.into(),
        descr_type: descr_type.to_owned(),
        addr_type: addr_type.map(str::to_owned),
        is_nestable,
        is_sorted,
        descriptor,
        policy,
        sigs_required,
        keys,
        keyspace_size,
        checksum,
    };

    let json = serde_json::to_string(&info)?;
    string_result_t::success(&json)
}
