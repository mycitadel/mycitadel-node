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

use libc::{c_char, c_int};
use serde::Serialize;
use std::convert::TryFrom;
use std::str::FromStr;

use rgb::bech32::Error;
use rgb::Bech32;
use rgb20::Asset;

use crate::{TryAsStr, TryIntoRaw, TryIntoString};

pub const BECH32_OK: c_int = 0;
pub const BECH32_ERR_HRP: c_int = 1;
pub const BECH32_ERR_CHECKSUM: c_int = 2;
pub const BECH32_ERR_ENCODING: c_int = 3;
pub const BECH32_ERR_PAYLOAD: c_int = 4;
pub const BECH32_ERR_UNSUPPORTED: c_int = 5;
pub const BECH32_ERR_INTERNAL: c_int = 6;
pub const BECH32_ERR_NULL: c_int = 7;

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
pub const BECH32_RGB_SCHEMA: c_int = 0x0310;
pub const BECH32_RGB_GENESIS: c_int = 0x0311;
pub const BECH32_RGB_CONSIGNMENT: c_int = 0x0320;

pub const BECH32_RGB20_ASSET: c_int = 0x0320;

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
        match serde_json::to_string(value).map(String::try_into_raw) {
            Ok(Some(json)) => Self {
                status: BECH32_OK,
                category,
                bech32m: false,
                details: json,
            },
            Ok(None) => bech32_info_t::with_null_value(),
            Err(err) => Self {
                status: BECH32_ERR_INTERNAL,
                category,
                bech32m: false,
                details: format!("Unable to encode details as JSON: {}", err)
                    .try_into_raw()
                    .unwrap_or("Unable to encode details as JSON".as_ptr()
                        as *const c_char),
            },
        }
    }

    pub fn with_null_value() -> Self {
        Self {
            status: BECH32_ERR_NULL,
            category: BECH32_UNKNOWN,
            bech32m: false,
            details: s!("Value must not be null").try_into_raw().unwrap(),
        }
    }

    pub fn with_wrong_payload() -> Self {
        Self {
            status: BECH32_ERR_PAYLOAD,
            category: BECH32_UNKNOWN,
            bech32m: false,
            details: s!("Payload format does not match bech32 type")
                .try_into_raw()
                .unwrap(),
        }
    }

    pub fn unsuported() -> Self {
        Self {
            status: BECH32_ERR_UNSUPPORTED,
            category: BECH32_UNKNOWN,
            bech32m: false,
            details: s!("This specific kind of Bech32 is not yet supported")
                .try_into_raw()
                .unwrap(),
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
            details: err
                .to_string()
                .try_into_raw()
                .unwrap_or("Unknown error".as_ptr() as *const c_char),
        }
    }
}

#[no_mangle]
pub extern "C" fn lnpbp_bech32_release(info: bech32_info_t) {
    (info.details as *mut c_char).try_into_string();
}

#[no_mangle]
pub extern "C" fn lnpbp_bech32_info(bech_str: *const c_char) -> bech32_info_t {
    bech_str
        .try_as_str()
        .map(|s| match Bech32::from_str(s) {
            Ok(Bech32::Genesis(genesis)) => Asset::try_from(genesis.clone())
                .map(|asset| {
                    bech32_info_t::with_value(BECH32_RGB20_ASSET, &asset)
                })
                .unwrap_or_else(|_| {
                    bech32_info_t::with_value(BECH32_RGB_GENESIS, &genesis)
                }),
            Ok(_) => bech32_info_t::unsuported(),
            Err(err) => bech32_info_t::from(err),
        })
        .unwrap_or(bech32_info_t::with_null_value())
}
