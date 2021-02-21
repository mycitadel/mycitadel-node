// MyCitadel C bindings library (libmycitadel)
// Written in 2020 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.

#![allow(dead_code)]

use std::os::raw::c_char;
use std::path::PathBuf;
use std::ptr;
use std::str::FromStr;

use internet2::ZmqSocketAddr;
use lnpbp::Chain;
use mycitadel::{rpc, Client};
use wallet::bip32::PubkeyChain;
use wallet::descriptor;

use crate::error::*;
use crate::mycitadel_client_t;
use crate::ptr_to_string;

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
pub extern "C" fn mycitadel_contract_list(
    client: *mut mycitadel_client_t,
) -> *const c_char {
    unsafe { client.as_mut().expect("Wrong MyCitadel client pointer") }
        .call(rpc::Request::ListContracts)
}

#[no_mangle]
pub extern "C" fn mycitadel_single_sig_create(
    client: *mut mycitadel_client_t,
    name: *const c_char,
    keychain: *const c_char,
    category: descriptor::OuterCategory,
) -> *const c_char {
    let client =
        unsafe { client.as_mut().expect("Wrong MyCitadel client pointer") };
    let pubkey_chain = match PubkeyChain::from_str(&ptr_to_string(keychain)) {
        Ok(pubkey_chain) => pubkey_chain,
        Err(err) => {
            client.set_error_details(
                ERRNO_PARSE,
                &format!("invalid keychain data for wallet creation: {}", err),
            );
            return ptr::null();
        }
    };
    let call = |client: &mut Client| {
        client.single_sig_create(ptr_to_string(name), pubkey_chain, category)
    };
    client
        .inner()
        .map(call)
        .map(|response| client.process_response(response))
        .unwrap_or(ptr::null())
}

#[no_mangle]
pub extern "C" fn mycitadel_contract_rename(
    client: *mut mycitadel_client_t,
    contract_id: *const c_char,
    new_name: *const c_char,
) -> *const c_char {
    let client =
        unsafe { client.as_mut().expect("Wrong MyCitadel client pointer") };
    let contract_id = client.contract_id(contract_id);
    client
        .inner()
        .and_then(|inner| {
            contract_id.map(|contract_id| {
                inner.contract_rename(contract_id, ptr_to_string(new_name))
            })
        })
        .map(|response| client.process_response(response))
        .unwrap_or(ptr::null())
}

#[no_mangle]
pub extern "C" fn mycitadel_contract_delete(
    client: *mut mycitadel_client_t,
    contract_id: *const c_char,
) -> *const c_char {
    let client =
        unsafe { client.as_mut().expect("Wrong MyCitadel client pointer") };
    let contract_id = client.contract_id(contract_id);
    client
        .inner()
        .and_then(|inner| {
            contract_id.map(|contract_id| inner.contract_delete(contract_id))
        })
        .map(|response| client.process_response(response))
        .unwrap_or(ptr::null())
}

#[no_mangle]
pub extern "C" fn mycitadel_list_assets(
    client: *mut mycitadel_client_t,
) -> *const c_char {
    let client =
        unsafe { client.as_mut().expect("Wrong MyCitadel client pointer") };
    client
        .inner()
        .map(Client::contract_list)
        .map(|response| client.process_response(response))
        .unwrap_or(ptr::null())
}

#[no_mangle]
pub extern "C" fn mycitadel_import_asset(
    client: *mut mycitadel_client_t,
    genesis_b32: *const c_char,
) -> *const c_char {
    let client =
        unsafe { client.as_mut().expect("Wrong MyCitadel client pointer") };
    let genesis_b32 = ptr_to_string(genesis_b32);
    client
        .inner()
        .map(|client| client.asset_import(genesis_b32))
        .map(|response| client.process_response(response))
        .unwrap_or(ptr::null())
}
