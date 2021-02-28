// MyCitadel C bindings library (libmycitadel)
// Written in 2020 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.

use libc::c_char;
use std::ptr;

use bitcoin::consensus::serialize;
use lnpbp::bech32::ToBech32String;
use mycitadel::client::InvoiceType;
use mycitadel::rpc::message;
use rgb::Consignment;
use wallet::descriptor;

use crate::TryIntoRaw;

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

#[allow(non_camel_case_types)]
#[repr(C)]
pub struct prepared_transfer_t {
    pub success: bool,
    pub consignment_bech32: *const c_char,
    pub psbt_base64: *const c_char,
}

impl prepared_transfer_t {
    pub fn failure() -> Self {
        prepared_transfer_t {
            success: false,
            consignment_bech32: ptr::null(),
            psbt_base64: ptr::null(),
        }
    }
}

impl From<message::PreparedPayment> for prepared_transfer_t {
    fn from(p: message::PreparedPayment) -> Self {
        prepared_transfer_t {
            success: true,
            consignment_bech32: p
                .consignment
                .as_ref()
                .map(Consignment::to_bech32_string)
                .and_then(String::try_into_raw)
                .unwrap_or(ptr::null()),
            psbt_base64: base64::encode(&serialize(&p.psbt))
                .try_into_raw()
                .expect("base64 PSBT representation contains zero byte"),
        }
    }
}
