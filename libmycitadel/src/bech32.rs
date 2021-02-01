// MyCitadel C bindings library (libmycitadel)
// Written in 2020 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.

#![allow(dead_code)]

// TODO: Move to rgb-core library

use serde::Serialize;
use std::convert::TryFrom;
use std::os::raw::{c_char, c_int};
use std::str::FromStr;

use rgb::bech32::Error;
use rgb::Bech32;
use rgb20::Asset;

use crate::{ptr_to_string, ToCharPtr};

pub const BECH32_OK: c_int = 0;
pub const BECH32_ERR_HRP: c_int = 1;
pub const BECH32_ERR_CHECKSUM: c_int = 2;
pub const BECH32_ERR_ENCODING: c_int = 3;
pub const BECH32_ERR_PAYLOAD: c_int = 4;
pub const BECH32_ERR_UNSUPPORTED: c_int = 5;
pub const BECH32_ERR_INTERNAL: c_int = 6;

pub const BECH32_UNKNOWN: c_int = 0;
pub const BECH32_URL: c_int = 1;

pub const BECH32_BC_ADDRESS: c_int = 0x0100;
pub const BECH32_LN_BOLT11: c_int = 0x0101;

pub const BECH32_LNPBP_ID: c_int = 0x0200;
pub const BECH32_LNPBP_DATA: c_int = 0x0201;
pub const BECH32_LNPBP_ZDATA: c_int = 0x0202;
pub const BECH32_LNPBP_INVOICE: c_int = 0x0210;

pub const BECH32_RGB_SCHEMA_ID: c_int = 0x0300;
pub const BECH32_RGB_CONTRACT_ID: c_int = 0x0301;
pub const BECH32_RGB_SCHEMA: c_int = 0x0302;
pub const BECH32_RGB_GENESIS: c_int = 0x0303;
pub const BECH32_RGB_CONSIGNMENT: c_int = 0x0304;

#[allow(non_camel_case_types)]
#[repr(C)]
pub struct bech32_info_t {
    pub status: c_int,
    pub category: c_int,
    pub bech32m: bool,
    pub details: *const c_char,
}

impl bech32_info_t {
    pub fn with_value<T>(category: c_int, value: &T) -> Self
    where
        T: ?Sized + Serialize,
    {
        match serde_json::to_string(value) {
            Ok(json) => Self {
                status: BECH32_OK,
                category,
                bech32m: false,
                details: json.to_char_ptr(),
            },
            Err(_) => Self {
                status: BECH32_ERR_INTERNAL,
                category,
                bech32m: false,
                details: "Unable to encode details as JSON".to_char_ptr(),
            },
        }
    }

    pub fn with_wrong_payload() -> Self {
        Self {
            status: BECH32_ERR_PAYLOAD,
            category: BECH32_UNKNOWN,
            bech32m: false,
            details: "Payload format does not match bech32 type".to_char_ptr(),
        }
    }

    pub fn unsuported() -> Self {
        Self {
            status: BECH32_ERR_UNSUPPORTED,
            category: BECH32_UNKNOWN,
            bech32m: false,
            details: "This specific kind of Bech32 is not yet supported"
                .to_char_ptr(),
        }
    }
}

impl From<Error> for bech32_info_t {
    fn from(err: rgb::bech32::Error) -> Self {
        let status = match err {
            Error::Bech32Error(bech32::Error::InvalidChecksum) => {
                BECH32_ERR_CHECKSUM
            }
            Error::Bech32Error(bech32::Error::MissingSeparator) => {
                BECH32_ERR_HRP
            }
            Error::Bech32Error(_) => BECH32_ERR_ENCODING,
            _ => BECH32_ERR_PAYLOAD,
        };
        Self {
            status,
            category: BECH32_UNKNOWN,
            bech32m: false,
            details: err.to_string().to_char_ptr(),
        }
    }
}

#[no_mangle]
pub extern "C" fn lnpbp_bech32_info(bech_str: *const c_char) -> bech32_info_t {
    match Bech32::from_str(&ptr_to_string(bech_str)) {
        Ok(Bech32::Genesis(genesis)) => Asset::try_from(genesis)
            .map(|asset| bech32_info_t::with_value(BECH32_RGB_GENESIS, &asset))
            .unwrap_or_else(|_| bech32_info_t::with_wrong_payload()),
        Ok(_) => bech32_info_t::unsuported(),
        Err(err) => bech32_info_t::from(err),
    }
}
