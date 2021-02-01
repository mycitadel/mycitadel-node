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
use mycitadel::rpc;

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
        mycitadel::run_embedded(mycitadel::daemon::Config {
            verbose: 4,
            chain,
            rpc_endpoint: ZmqSocketAddr::Inproc(s!("mycitadel.rpc")),
            rgb20_endpoint: ZmqSocketAddr::Inproc(s!("rgb20.rpc")),
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
        Ok(genesis) => client.call(rpc::Request::AddAsset(genesis)),
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
