// MyCitadel C bindings library (libmycitadel)
// Written in 2020 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.

use std::os::raw::c_int;

pub const SUCCESS: c_int = 0;
pub const ERRNO_IO: c_int = 1;
pub const ERRNO_RPC: c_int = 2;
pub const ERRNO_NET: c_int = 3;
pub const ERRNO_TRANSPORT: c_int = 4;
pub const ERRNO_NOTSUPPORTED: c_int = 5;
pub const ERRNO_STORAGE: c_int = 6;
pub const ERRNO_SERVERFAIL: c_int = 7;
pub const ERRNO_EMBEDDEDFAIL: c_int = 8;
pub const ERRNO_UNINIT: c_int = 100;
pub const ERRNO_CHAIN: c_int = 101;
pub const ERRNO_JSON: c_int = 102;
pub const ERRNO_BECH32: c_int = 103;
pub const ERRNO_PARSE: c_int = 104;
pub const ERRNO_NULL: c_int = 105;
