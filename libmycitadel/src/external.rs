// MyCitadel C bindings library (libmycitadel)
// Written in 2020 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.

#![allow(dead_code)]

use libc::c_char;
use std::path::PathBuf;
use std::ptr;
use std::str::FromStr;

use internet2::ZmqSocketAddr;
use invoice::Invoice;
use lnpbp::Chain;
use mycitadel::client::InvoiceType;
use mycitadel::Client;
use wallet::bip32::PubkeyChain;
use wallet::descriptor;

use crate::error::*;
use crate::helpers::{TryAsStr, TryIntoString};
use crate::{mycitadel_client_t, TryIntoRaw};

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[repr(C)]
pub enum descriptor_type {
    BARE,
    HASHED,
    SEGWIT,
    TAPROOT,
}

impl From<descriptor_type> for descriptor::OuterCategory {
    fn from(t: descriptor_type) -> Self {
        match t {
            descriptor_type::BARE => descriptor::OuterCategory::Bare,
            descriptor_type::HASHED => descriptor::OuterCategory::Hashed,
            descriptor_type::SEGWIT => descriptor::OuterCategory::SegWit,
            descriptor_type::TAPROOT => descriptor::OuterCategory::Taproot,
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[repr(C)]
pub enum invoice_type {
    ADDRESS_UTXO,
    DESCRIPTOR,
    PSBT,
}

impl From<invoice_type> for InvoiceType {
    fn from(t: invoice_type) -> Self {
        match t {
            invoice_type::ADDRESS_UTXO => InvoiceType::AddressUtxo,
            invoice_type::DESCRIPTOR => InvoiceType::Descriptor,
            invoice_type::PSBT => InvoiceType::Psbt,
        }
    }
}

#[no_mangle]
pub extern "C" fn release_string(s: *mut c_char) {
    s.try_into_string();
}

#[no_mangle]
pub extern "C" fn mycitadel_run_embedded(
    chain: *const c_char,
    data_dir: *const c_char,
    electrum_server: *const c_char,
) -> *mut mycitadel_client_t {
    let mut config = mycitadel::server::Config {
        verbose: 4,
        rpc_endpoint: ZmqSocketAddr::Inproc(s!("mycitadel.rpc")),
        rgb20_endpoint: ZmqSocketAddr::Inproc(s!("rgb20.rpc")),
        rgb_embedded: true,
        ..default!()
    };

    if let Some(chain) = chain.try_as_str() {
        if let Ok(chain) = Chain::from_str(chain) {
            config.chain = chain;
        } else {
            return mycitadel_client_t::from_custom_err(
                ERRNO_CHAIN,
                &format!("Unknown chain {}", chain),
            );
        }
    }

    if let Some(data_dir) = data_dir.try_as_str() {
        config.data_dir = PathBuf::from(data_dir);
    }

    if let Some(electrum_server) = electrum_server.try_as_str() {
        config.electrum_server = electrum_server.to_string();
    }

    mycitadel::run_embedded(config)
        .map(mycitadel_client_t::with)
        .unwrap_or_else(mycitadel_client_t::from_err)
}

#[no_mangle]
pub extern "C" fn mycitadel_is_ok(client: *mut mycitadel_client_t) -> bool {
    mycitadel_client_t::from_raw(client).is_ok()
}

#[no_mangle]
pub extern "C" fn mycitadel_has_err(client: *mut mycitadel_client_t) -> bool {
    mycitadel_client_t::from_raw(client).has_err()
}

#[no_mangle]
pub extern "C" fn mycitadel_contract_list(
    client: *mut mycitadel_client_t,
) -> *const c_char {
    let client = mycitadel_client_t::from_raw(client);
    client
        .try_as_opaque()
        .map(Client::contract_list)
        .map(|response| client.process_response(response))
        .unwrap_or(ptr::null())
}

#[no_mangle]
pub extern "C" fn mycitadel_single_sig_create(
    client: *mut mycitadel_client_t,
    name: *const c_char,
    keychain: *const c_char,
    category: descriptor_type,
) -> *const c_char {
    let client = mycitadel_client_t::from_raw(client);

    let (keychain, name) = match (|| {
        Some((
            client.parse_string(keychain, "keychain parameter").ok()?,
            client.parse_string(name, "contract name").ok()?,
        ))
    })() {
        None => return ptr::null(),
        Some(v) => v,
    };

    let pubkey_chain = match PubkeyChain::from_str(&keychain) {
        Ok(pubkey_chain) => pubkey_chain,
        Err(err) => {
            client.set_error_details(
                ERRNO_PARSE,
                &format!("invalid keychain data for wallet creation: {}", err),
            );
            return ptr::null();
        }
    };
    client
        .try_as_opaque()
        .map(|opaque| {
            opaque.single_sig_create(name, pubkey_chain, category.into())
        })
        .map(|response| client.process_response(response))
        .unwrap_or(ptr::null())
}

#[no_mangle]
pub extern "C" fn mycitadel_contract_rename(
    client: *mut mycitadel_client_t,
    contract_id: *const c_char,
    new_name: *const c_char,
) -> *const c_char {
    let client = mycitadel_client_t::from_raw(client);

    let (contract_id, new_name) = match (|| {
        Some((
            client.parse_contract_id(contract_id).ok()?,
            client.parse_string(new_name, "new name").ok()?,
        ))
    })() {
        None => return ptr::null(),
        Some(v) => v,
    };

    client
        .try_as_opaque()
        .map(|opaque| opaque.contract_rename(contract_id, new_name))
        .map(|response| client.process_response(response))
        .unwrap_or(ptr::null())
}

#[no_mangle]
pub extern "C" fn mycitadel_contract_delete(
    client: *mut mycitadel_client_t,
    contract_id: *const c_char,
) -> *const c_char {
    let client = mycitadel_client_t::from_raw(client);

    let contract_id = match client.parse_contract_id(contract_id).ok() {
        None => return ptr::null(),
        Some(v) => v,
    };

    client
        .try_as_opaque()
        .map(|opaque| opaque.contract_delete(contract_id))
        .map(|response| client.process_response(response))
        .unwrap_or(ptr::null())
}

#[no_mangle]
pub extern "C" fn mycitadel_contract_balance(
    client: *mut mycitadel_client_t,
    contract_id: *const c_char,
    rescan: bool,
    lookup_depth: u8,
) -> *const c_char {
    let client = mycitadel_client_t::from_raw(client);

    let contract_id = match client.parse_contract_id(contract_id).ok() {
        None => return ptr::null(),
        Some(v) => v,
    };

    client
        .try_as_opaque()
        .map(|opaque| {
            opaque.contract_balance(contract_id, rescan, lookup_depth)
        })
        .map(|response| client.process_response(response))
        .unwrap_or(ptr::null())
}

#[no_mangle]
pub extern "C" fn mycitadel_address_list(
    client: *mut mycitadel_client_t,
    contract_id: *const c_char,
    rescan: bool,
    lookup_depth: u8,
) -> *const c_char {
    let client = mycitadel_client_t::from_raw(client);

    let contract_id = match client.parse_contract_id(contract_id).ok() {
        None => return ptr::null(),
        Some(v) => v,
    };

    client
        .try_as_opaque()
        .map(|opaque| opaque.address_list(contract_id, rescan, lookup_depth))
        .map(|response| client.process_response(response))
        .unwrap_or(ptr::null())
}

#[no_mangle]
pub extern "C" fn mycitadel_address_create(
    client: *mut mycitadel_client_t,
    contract_id: *const c_char,
    mark_used: bool,
    legacy: bool,
) -> *const c_char {
    let client = mycitadel_client_t::from_raw(client);

    let contract_id = match client.parse_contract_id(contract_id).ok() {
        None => return ptr::null(),
        Some(v) => v,
    };

    client
        .try_as_opaque()
        .map(|opaque| {
            opaque.address_create(contract_id, None, mark_used, legacy)
        })
        .map(|response| client.process_response(response))
        .unwrap_or(ptr::null())
}

#[no_mangle]
pub extern "C" fn mycitadel_invoice_create(
    client: *mut mycitadel_client_t,
    category: invoice_type,
    contract_id: *const c_char,
    asset_id: *const c_char,
    amount: u64,
    merchant: *const c_char,
    purpose: *const c_char,
    unmark: bool,
    legacy: bool,
) -> *const c_char {
    let client = mycitadel_client_t::from_raw(client);

    let (contract_id, asset_id) = match (|| {
        Some((
            client.parse_contract_id(contract_id).ok()?,
            client.parse_asset_id(asset_id).ok()?,
        ))
    })() {
        None => return ptr::null(),
        Some(v) => v,
    };

    let result = client.try_as_opaque().map(|inner| {
        inner.invoice_create(
            category.into(),
            contract_id,
            asset_id,
            amount,
            merchant.try_as_str(),
            purpose.try_as_str(),
            unmark,
            legacy,
        )
    });
    result
        .and_then(|reply| reply.map_err(|err| client.set_error(err)).ok())
        .and_then(|invoice| invoice.to_string().try_into_raw())
        .unwrap_or(ptr::null())
}

#[no_mangle]
pub extern "C" fn mycitadel_invoice_list(
    client: *mut mycitadel_client_t,
    contract_id: *const c_char,
) -> *const c_char {
    let client = mycitadel_client_t::from_raw(client);

    let contract_id = match client.parse_contract_id(contract_id).ok() {
        None => return ptr::null(),
        Some(v) => v,
    };

    client
        .try_as_opaque()
        .map(|opaque| opaque.invoice_list(contract_id))
        .map(|response| client.process_response(response))
        .unwrap_or(ptr::null())
}

#[no_mangle]
pub extern "C" fn mycitadel_invoice_pay(
    client: *mut mycitadel_client_t,
    contract_id: *const c_char,
    invoice: *const c_char,
    fee: u64,
    giveaway: u64,
) -> *const c_char {
    let client = mycitadel_client_t::from_raw(client);

    let (contract_id, invoice) = match (|| {
        Some((
            client.parse_contract_id(contract_id).ok()?,
            client.parse_string(invoice, "invoice").ok()?,
        ))
    })() {
        None => return ptr::null(),
        Some(v) => v,
    };

    let invoice = match Invoice::from_str(invoice) {
        Ok(invoice) => invoice,
        Err(err) => {
            client.set_error(err.into());
            return ptr::null();
        }
    };

    let result = client.try_as_opaque().map(|inner| {
        inner.invoice_pay(
            contract_id,
            invoice,
            None,
            fee,
            if giveaway > 0 { Some(giveaway) } else { None },
        )
    });
    result
        .and_then(|reply| reply.map_err(|err| client.set_error(err)).ok())
        .and_then(|info| info.to_string().try_into_raw())
        .unwrap_or(ptr::null())
}

#[no_mangle]
pub extern "C" fn mycitadel_invoice_accept(
    client: *mut mycitadel_client_t,
    contract_id: *const c_char,
) -> *const c_char {
    let client = mycitadel_client_t::from_raw(client);

    let contract_id = match client.parse_contract_id(contract_id).ok() {
        None => return ptr::null(),
        Some(v) => v,
    };

    client
        .try_as_opaque()
        .map(|opaque| opaque.invoice_list(contract_id))
        .map(|response| client.process_response(response))
        .unwrap_or(ptr::null())
}

#[no_mangle]
pub extern "C" fn mycitadel_asset_list(
    client: *mut mycitadel_client_t,
) -> *const c_char {
    let client = mycitadel_client_t::from_raw(client);
    client
        .try_as_opaque()
        .map(Client::asset_list)
        .map(|response| client.process_response(response))
        .unwrap_or(ptr::null())
}

#[no_mangle]
pub extern "C" fn mycitadel_asset_import(
    client: *mut mycitadel_client_t,
    genesis_b32: *const c_char,
) -> *const c_char {
    let client = mycitadel_client_t::from_raw(client);

    let genesis_b32 = match client.parse_string(genesis_b32, "genesis").ok() {
        None => return ptr::null(),
        Some(v) => v,
    };

    client
        .try_as_opaque()
        .map(|opaque| opaque.asset_import(genesis_b32))
        .map(|response| client.process_response(response))
        .unwrap_or(ptr::null())
}
