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

pub const ERRNO_IO: c_int = 1;
pub const ERRNO_RPC: c_int = 2;
pub const ERRNO_NET: c_int = 3;
pub const ERRNO_TRANSPORT: c_int = 4;
pub const ERRNO_NOTSUPPORTED: c_int = 5;
pub const ERRNO_STORAGE: c_int = 6;
pub const ERRNO_SERVERFAIL: c_int = 7;
pub const ERRNO_EMBEDDEDFAIL: c_int = 8;
pub const ERRNO_CHAIN: c_int = 100;
pub const ERRNO_JSON: c_int = 101;

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
pub struct mycitadel_error_t {
    pub errno: c_int,
    pub message: *const c_char,
}

impl From<mycitadel::Error> for mycitadel_error_t {
    fn from(err: Error) -> Self {
        mycitadel_error_t {
            errno: match err {
                Error::Io(_) => ERRNO_IO,
                Error::Rpc(_) => ERRNO_RPC,
                Error::Networking(_) => ERRNO_NET,
                Error::Transport(_) => ERRNO_TRANSPORT,
                Error::NotSupported(_) => ERRNO_NOTSUPPORTED,
                Error::StorageDriver(_) => ERRNO_STORAGE,
                Error::ServerFailure(_) => ERRNO_SERVERFAIL,
                Error::EmbeddedNodeError => ERRNO_EMBEDDEDFAIL,
                _ => 0,
            },
            message: err.to_string().to_char_ptr(),
        }
    }
}

#[allow(non_camel_case_types)]
#[repr(C)]
pub struct mycitadel_client_t {
    _inner: *mut c_void,
    pub last_error: *mut mycitadel_error_t,
}

impl mycitadel_client_t {
    pub(crate) fn with(mut client: Client) -> Self {
        mycitadel_client_t {
            _inner: &mut client as *mut _ as *mut c_void,
            last_error: ptr::null_mut(),
        }
    }

    pub(crate) fn from_err(error: mycitadel::Error) -> Self {
        mycitadel_client_t {
            _inner: ptr::null_mut(),
            last_error: &mut mycitadel_error_t::from(error),
        }
    }

    pub(crate) fn from_custom_err(errno: c_int, msg: &str) -> Self {
        mycitadel_client_t {
            _inner: ptr::null_mut(),
            last_error: &mut mycitadel_error_t {
                errno,
                message: msg.to_char_ptr(),
            },
        }
    }

    #[no_mangle]
    pub extern "C" fn mycitadel_is_ok(&self) -> bool {
        self._inner != ptr::null_mut()
    }

    #[no_mangle]
    pub extern "C" fn mycitadel_has_err(&self) -> bool {
        self.last_error != ptr::null_mut()
    }

    fn inner(&mut self) -> &mut Client {
        if self.mycitadel_is_ok() {
            panic!("MyCitadel controller is not initialized")
        }
        let boxed = unsafe { Box::from_raw(self._inner as *mut Client) };
        Box::leak(boxed)
    }

    pub(crate) fn call(&mut self, request: rpc::Request) -> *const c_char {
        match self.inner().request(request.clone()) {
            Err(err) => {
                self.last_error = &mut mycitadel_error_t::from(err);
                ptr::null()
            }
            Ok(result) => {
                if let Ok(json) = serde_json::to_string(&result) {
                    self.last_error = ptr::null_mut();
                    json.to_char_ptr()
                } else {
                    self.last_error = &mut mycitadel_error_t {
                        errno: ERRNO_JSON,
                        message: format!(
                            "Unable to JSON-encode response for the call {}",
                            request
                        )
                        .to_char_ptr(),
                    };
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
    let chain = if let Ok(chain) = Chain::from_str(&chain) {
        chain
    } else {
        return &mut mycitadel_client_t::from_custom_err(
            ERRNO_CHAIN,
            &format!("Unknown chain {}", chain),
        );
    };
    &mut mycitadel::run_embedded(mycitadel::daemon::Config {
        verbose: 4,
        chain,
        rpc_endpoint: ZmqSocketAddr::Inproc(s!("mycitadel.rpc")),
        rgb20_endpoint: ZmqSocketAddr::Inproc(s!("rgb20.rpc")),
        data_dir: PathBuf::from(ptr_to_string(data_dir)),
        electrum_server: ptr_to_string(electrum_server),
    })
    .map(mycitadel_client_t::with)
    .unwrap_or_else(mycitadel_client_t::from_err)
}

#[no_mangle]
pub extern "C" fn mycitadel_list_assets(
    client: *mut mycitadel_client_t,
) -> *const c_char {
    unsafe { client.as_mut().expect("Wrong MyCitadel client pointer") }
        .call(rpc::Request::ListAssets)
}
