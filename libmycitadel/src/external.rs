// MyCitadel C bindings library (libmycitadel)
// Written in 2020 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.

use std::ffi::{c_void, CStr, CString};
use std::os::raw::{c_char, c_int};
use std::path::PathBuf;
use std::ptr;
use std::str::FromStr;

use internet2::ZmqSocketAddr;
use lnpbp::Chain;
use mycitadel::{rpc, Client, Error};

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

pub trait ToCharPtr {
    fn to_char_ptr(&self) -> *const c_char;
}

impl<T> ToCharPtr for T
where
    T: AsRef<[u8]>,
{
    fn to_char_ptr(&self) -> *const c_char {
        let mut vec = self.as_ref().to_owned();
        for c in &mut vec {
            if *c == 0 {
                *c = 0x20;
            }
        }
        CString::new(vec).expect("ToCharPtr is broken").into_raw()
    }
}

pub(crate) fn ptr_to_string(ptr: *const c_char) -> String {
    unsafe { CStr::from_ptr(ptr) }.to_string_lossy().to_string()
}

#[allow(non_camel_case_types)]
#[repr(C)]
pub struct mycitadel_client_t {
    inner: *mut c_void,
    message: *const c_char,
    err_no: c_int,
}

impl mycitadel_client_t {
    pub(crate) fn with(inner_client: Client) -> Self {
        mycitadel_client_t {
            inner: Box::into_raw(Box::new(inner_client)) as *mut c_void,
            err_no: SUCCESS,
            message: ptr::null(),
        }
    }

    pub(crate) fn from_err(error: mycitadel::Error) -> Self {
        let mut me = mycitadel_client_t {
            inner: ptr::null_mut(),
            err_no: c_int::MAX,
            message: ptr::null(),
        };
        me.set_error(error);
        me
    }

    pub(crate) fn from_custom_err(err_no: c_int, msg: &str) -> Self {
        let mut me = mycitadel_client_t {
            inner: ptr::null_mut(),
            err_no,
            message: ptr::null(),
        };
        me.set_error_details(err_no, msg);
        me
    }

    fn set_success(&mut self) {
        self.err_no = SUCCESS;
        self.message = ptr::null()
    }

    fn set_error_details(&mut self, err_no: c_int, msg: &str) {
        self.err_no = err_no;
        self.message = msg.to_char_ptr();
    }

    fn set_error_no(&mut self, err_no: c_int) {
        let message = match err_no {
            ERRNO_UNINIT => "MyCitadel client is not yet initialized",
            _ => panic!("Error in mycitadel_error_t::with"),
        };
        self.set_error_details(err_no, message);
    }

    fn set_error(&mut self, err: mycitadel::Error) {
        let err_no = match err {
            Error::Io(_) => ERRNO_IO,
            Error::Rpc(_) => ERRNO_RPC,
            Error::Networking(_) => ERRNO_NET,
            Error::Transport(_) => ERRNO_TRANSPORT,
            Error::NotSupported(_) => ERRNO_NOTSUPPORTED,
            Error::StorageDriver(_) => ERRNO_STORAGE,
            Error::ServerFailure(_) => ERRNO_SERVERFAIL,
            Error::EmbeddedNodeError => ERRNO_EMBEDDEDFAIL,
            _ => c_int::MAX,
        };
        self.set_error_details(err_no, &err.to_string());
    }

    pub(crate) fn is_ok(&self) -> bool {
        self.inner.is_null() && self.err_no == SUCCESS
    }

    pub(crate) fn has_err(&self) -> bool {
        self.err_no != SUCCESS && !self.message.is_null()
    }

    fn inner(&mut self) -> Option<&mut Client> {
        if self.is_ok() {
            return None;
        }
        let boxed = unsafe { Box::from_raw(self.inner as *mut Client) };
        Some(Box::leak(boxed))
    }

    pub(crate) fn call(&mut self, request: rpc::Request) -> *const c_char {
        let inner = match self.inner() {
            None => {
                self.set_error_no(ERRNO_UNINIT);
                return ptr::null();
            }
            Some(inner) => inner,
        };
        match inner.request(request.clone()) {
            Err(err) => {
                self.set_error(err);
                ptr::null()
            }
            Ok(result) => {
                if let Ok(json) = serde_json::to_string(&result) {
                    self.set_success();
                    json.to_char_ptr()
                } else {
                    self.set_error_details(
                        ERRNO_JSON,
                        &format!(
                            "Unable to JSON-encode response for the call {}",
                            request
                        ),
                    );
                    ptr::null()
                }
            }
        }
    }
}

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
