// MyCitadel C bindings library (libmycitadel)
// Written in 2020 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.

#![allow(dead_code)]

use rand::RngCore;
use std::os::raw::c_char;
use std::path::PathBuf;
use std::ptr;
use std::str::FromStr;

use bitcoin::util::bip32::{ChainCode, ChildNumber, ExtendedPrivKey};
use bitcoin::Network;
use bitcoin::{secp256k1, PrivateKey};
use internet2::ZmqSocketAddr;
use lnpbp::Chain;
use mycitadel::rpc;

use crate::error::*;
use crate::mycitadel_client_t;
use crate::ptr_to_string;

#[allow(non_camel_case_types)]
#[repr(C)]
pub struct mycitadel_mnemonic12_t {
    inner: [u8; 16],
}

#[allow(non_camel_case_types)]
#[repr(C)]
pub struct mycitadel_master_xpriv_t {
    inner: [u8; 64],
}

#[allow(non_camel_case_types)]
#[repr(C)]
pub struct mycitadel_xpriv_t {
    inner: [u8; 76],
}

#[allow(non_camel_case_types)]
#[repr(C)]
pub struct mycitadel_xpub_t {
    inner: [u8; 76],
}

#[no_manglee]
pub extern "C" fn mycitadel_seed16_create() -> mycitadel_mnemonic12_t {
    let mut inner = [0u8; 16];
    rand::thread_rng().fill_bytes(&inner);
    mycitadel_mnemonic12_t { inner }
}

#[no_mangle]
pub extern "C" fn mycitadel_seed16_wipe(mut seed: mycitadel_mnemonic12_t) {
    seed.inner.fill(0u8);
}

#[no_mangle]
pub extern "C" fn mycitadel_seed16_master(
    seed: mycitadel_mnemonic12_t,
    passwd: *const c_char,
    testnet: bool,
) -> mycitadel_master_xpriv_t {
    let mnemonic = bip39::Mnemonic::from_entropy(&seed.inner)?;
    let passwd = ptr_to_string(passwd);
    let seed = mnemonic.to_seed(&passwd);
    let xpriv = ExtendedPrivKey::new_master(
        if testnet {
            Network::Testnet
        } else {
            Network::Bitcoin
        },
        &seed,
    )?;
    let mut inner = [0u8; 64];
    inner[..32].copy_from_slice(xpriv.private_key.key);
    inner[32..].copy_from_slice(xpriv.chain_code.as_bytes());
    mycitadel_master_xpriv_t { inner }
}

#[no_mangle]
pub extern "C" fn mycitadel_xpriv_derive(
    master: mycitadel_master_xpriv_t,
    derivation: *const c_char,
    testnet: bool,
) -> mycitadel_xpriv_t {
    let mut priv_key = [0u8; 32];
    let mut chain_code = [0u8; 32];
    priv_key.copy_from_slice(&master.inner[..32]);
    chain_code.copy_from_slice(&master.inner[32..]);
    let network = if testnet {
        Network::Testnet
    } else {
        Network::Bitcoin
    };
    let master = ExtendedPrivKey {
        network,
        depth: 0,
        parent_fingerprint: Default::default(),
        child_number: ChildNumber::Normal { index: 0 },
        private_key: PrivateKey {
            compressed: true,
            network,
            key: secp256k1::SecretKey::from_slice(&priv_key)?,
        },
        chain_code: ChainCode::from(&chain_code),
    };
    let derivation = ptr_to_string(derivation);
    let xpriv = master.derive_priv(&wallet::SECP256K1, derivation.parse()?)?;
}

#[no_mangle]
pub extern "C" fn mycitadel_run_embedded(
    chain: *const c_char,
    data_dir: *const c_char,
    electrum_server: *const c_char,
) -> *mut mycitadel_client_t {
    let chain = ptr_to_string(chain);
    let client = if let Ok(chain) = Chain::from_str(&chain) {
        mycitadel::run_embedded(mycitadel::server::Config {
            verbose: 4,
            chain,
            rpc_endpoint: ZmqSocketAddr::Inproc(s!("mycitadel.rpc")),
            rgb20_endpoint: ZmqSocketAddr::Inproc(s!("rgb20.rpc")),
            rgb_embedded: true,
            data_dir: PathBuf::from(ptr_to_string(data_dir)),
            electrum_server: ptr_to_string(electrum_server),
        })
        .map(mycitadel_client_t::with)
        .unwrap_or_else(mycitadel_client_t::from_err)
    } else {
        mycitadel_client_t::from_custom_err(
            ERRNO_CHAIN,
            &format!("Unknown chain {}", chain),
        )
    };
    Box::into_raw(Box::new(client))
}

#[no_mangle]
pub extern "C" fn mycitadel_is_ok(client: *mut mycitadel_client_t) -> bool {
    unsafe { client.as_mut().expect("Wrong MyCitadel client pointer") }.is_ok()
}

#[no_mangle]
pub extern "C" fn mycitadel_has_err(client: *mut mycitadel_client_t) -> bool {
    unsafe { client.as_mut().expect("Wrong MyCitadel client pointer") }
        .has_err()
}

#[no_mangle]
pub extern "C" fn mycitadel_list_assets(
    client: *mut mycitadel_client_t,
) -> *const c_char {
    unsafe { client.as_mut().expect("Wrong MyCitadel client pointer") }
        .call(rpc::Request::ListAssets)
}

#[no_mangle]
pub extern "C" fn mycitadel_import_asset(
    client: *mut mycitadel_client_t,
    genesis_b32: *const c_char,
) -> *const c_char {
    let client =
        unsafe { client.as_mut().expect("Wrong MyCitadel client pointer") };
    let genesis_b32 = ptr_to_string(genesis_b32);
    match rgb::Genesis::from_str(&genesis_b32) {
        Ok(genesis) => client.call(rpc::Request::ImportAsset(genesis)),
        Err(err) => {
            client.set_error_details(
                ERRNO_BECH32,
                &format!(
                    "Error in Bech32 encoding of asset genesis: {}",
                    err.to_string()
                ),
            );
            ptr::null()
        }
    }
}
