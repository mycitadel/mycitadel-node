// MyCitadel C bindings library (libmycitadel)
// Written in 2020 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.

use mycitadel::client::InvoiceType;
use wallet::descriptor;

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
