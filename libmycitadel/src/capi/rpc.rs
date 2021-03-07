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

use bitcoin::consensus::deserialize;
use bitcoin::Address;
use internet2::ZmqSocketAddr;
use invoice::{Beneficiary, Invoice};
use lnpbp::Chain;
use microservices::rpc::Failure;
use mycitadel::Client;
use rgb::Consignment;
use wallet::bip32::PubkeyChain;
use wallet::Psbt;

use super::{descriptor_type, invoice_type};
use crate::capi::{prepared_transfer_t, AddressInfo};
use crate::error::*;
use crate::helpers::{TryAsStr, TryIntoRaw};
use crate::mycitadel_client_t;

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
pub extern "C" fn mycitadel_contract_operations(
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
        .map(|opaque| opaque.contract_operations(contract_id))
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
    mark_used: bool,
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
            mark_used,
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
pub extern "C" fn mycitadel_address_pay(
    client: *mut mycitadel_client_t,
    address: *const c_char,
    amount: u64,
    fee: u64,
) -> *const c_char {
    let client = mycitadel_client_t::from_raw(client);

    let address = match client.parse_string(address, "address") {
        Ok(address) => address,
        Err(_) => {
            client.set_error(mycitadel::Error::ServerFailure(Failure {
                code: 0,
                info: s!("Address must not be an empty string"),
            }));
            return ptr::null();
        }
    };
    let address = match Address::from_str(address) {
        Ok(invoice) => invoice,
        Err(err) => {
            client.set_error(mycitadel::Error::ServerFailure(Failure {
                code: 0,
                info: format!("Invalid address data: {}", err),
            }));
            return ptr::null();
        }
    };
    let invoice =
        Invoice::new(Beneficiary::Address(address), Some(amount), None);
    let invoice = invoice.to_string();
    mycitadel_invoice_pay(
        client,
        ptr::null(),
        invoice.as_ptr() as *const c_char,
        fee,
        0,
    )
    .psbt_base64
}

#[no_mangle]
pub extern "C" fn mycitadel_invoice_pay(
    client: *mut mycitadel_client_t,
    contract_id: *const c_char,
    invoice: *const c_char,
    fee: u64,
    giveaway: u64,
) -> prepared_transfer_t {
    let client = mycitadel_client_t::from_raw(client);

    let (contract_id, invoice) = match (|| {
        Some((
            client.parse_contract_id(contract_id).ok()?,
            client.parse_string(invoice, "invoice").ok()?,
        ))
    })() {
        None => return prepared_transfer_t::failure(),
        Some(v) => v,
    };

    let invoice = match Invoice::from_str(invoice).or_else(|err| {
        println!("Parsing invoice as URL or bitcoin address: {}", invoice);
        AddressInfo::from_str(invoice)
            .map(Invoice::from)
            .map_err(|_| err)
    }) {
        Ok(invoice) => invoice,
        Err(err) => {
            client.set_error(err.into());
            return prepared_transfer_t::failure();
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
        .map(prepared_transfer_t::from)
        .unwrap_or(prepared_transfer_t::failure())
}

#[no_mangle]
pub extern "C" fn mycitadel_psbt_publish(
    client: *mut mycitadel_client_t,
    psbt: *const c_char,
) -> *const c_char {
    let client = mycitadel_client_t::from_raw(client);

    let psbt = match client.parse_string(psbt, "PSBT").ok() {
        None => return ptr::null(),
        Some(v) => v,
    };
    let psbt = match base64::decode(psbt) {
        Err(err) => {
            client.set_error_details(ERRNO_PARSE, err);
            return ptr::null();
        }
        Ok(v) => v,
    };
    let psbt: Psbt = match deserialize(&psbt) {
        Err(err) => {
            client.set_error_details(ERRNO_PARSE, err);
            return ptr::null();
        }
        Ok(v) => v,
    };

    if let Some(txid) = client
        .try_as_opaque()
        .map(|opaque| opaque.finalize_publish_psbt(psbt))
        .transpose()
        .map_err(|_| ())
        .and_then(|res| res.ok_or(()))
        .ok()
        .and_then(|txid| txid.to_string().try_into_raw())
    {
        client.set_success();
        return txid;
    }

    return ptr::null();
}

#[no_mangle]
pub extern "C" fn mycitadel_invoice_accept(
    client: *mut mycitadel_client_t,
    consignment: *const c_char,
) -> *const c_char {
    let client = mycitadel_client_t::from_raw(client);

    let consignment = match client.parse_string(consignment, "consignment").ok()
    {
        None => return ptr::null(),
        Some(v) => v,
    };
    let consignment = match Consignment::from_str(&consignment) {
        Err(err) => {
            client.set_error_details(ERRNO_PARSE, err);
            return ptr::null();
        }
        Ok(v) => v,
    };

    if let Some(status) = client
        .try_as_opaque()
        .map(|opaque| opaque.invoice_accept(consignment))
        .transpose()
        .map_err(|_| ())
        .and_then(|res| res.ok_or(()))
        .ok()
        .and_then(|status| status.to_string().try_into_raw())
    {
        client.set_success();
        return status;
    }

    return ptr::null();
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
