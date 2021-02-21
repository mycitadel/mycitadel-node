// MyCitadel C bindings library (libmycitadel)
// Written in 2020 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.

use std::ffi::c_void;
use std::os::raw::{c_char, c_int};
use std::ptr;

use mycitadel::{rpc, Client, Error};

use crate::error::*;
use crate::ToCharPtr;

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

    pub(crate) fn inner(&mut self) -> Option<&mut Client> {
        if self.inner.is_null() {
            self.set_error_no(ERRNO_UNINIT);
            return None;
        }
        let boxed = unsafe { Box::from_raw(self.inner as *mut Client) };
        Some(Box::leak(boxed))
    }

    fn set_success(&mut self) {
        self.err_no = SUCCESS;
        self.message = ptr::null()
    }

    pub(crate) fn set_error_details(&mut self, err_no: c_int, msg: &str) {
        self.err_no = err_no;
        self.message = msg.to_char_ptr();
    }

    pub(crate) fn set_error_no(&mut self, err_no: c_int) {
        let message = match err_no {
            ERRNO_UNINIT => "MyCitadel client is not yet initialized",
            _ => panic!("Error in mycitadel_error_t::with"),
        };
        self.set_error_details(err_no, message);
    }

    pub(crate) fn set_error(&mut self, err: mycitadel::Error) {
        let err_no = match err {
            Error::Io(_) => ERRNO_IO,
            Error::Rpc(_) => ERRNO_RPC,
            Error::Networking(_) => ERRNO_NET,
            Error::Transport(_) => ERRNO_TRANSPORT,
            Error::NotSupported(_) => ERRNO_NOTSUPPORTED,
            Error::StorageDriver(_) => ERRNO_STORAGE,
            Error::ServerFailure(_) => ERRNO_SERVERFAIL,
            Error::EmbeddedNodeInitError => ERRNO_EMBEDDEDFAIL,
            _ => c_int::MAX,
        };
        self.set_error_details(err_no, &err.to_string());
    }

    pub(crate) fn set_failure(&mut self, failure: microservices::rpc::Failure) {
        self.err_no = ERRNO_SERVERFAIL;
        self.message = failure.to_string().to_char_ptr();
    }

    pub(crate) fn is_ok(&self) -> bool {
        self.message.is_null() && self.err_no == SUCCESS
    }

    pub(crate) fn has_err(&self) -> bool {
        self.err_no != SUCCESS && !self.message.is_null()
    }

    pub(crate) fn process_response(
        &mut self,
        response: Result<rpc::Reply, Error>,
    ) -> *const c_char {
        response
            .map_err(|err| {
                self.set_error(err);
                ()
            })
            .and_then(|reply| {
                if let rpc::Reply::Failure(failure) = reply {
                    self.set_failure(failure);
                    Err(())
                } else {
                    Ok(reply)
                }
            })
            .map(|result| match serde_json::to_string(&result) {
                Ok(json) => {
                    self.set_success();
                    json.to_char_ptr()
                }
                Err(err) => {
                    self.set_error_details(
                        ERRNO_JSON,
                        &format!("Unable to JSON-encode response: {}", err),
                    );
                    ptr::null()
                }
            })
            .unwrap_or(ptr::null())
    }

    pub(crate) fn call(&mut self, request: rpc::Request) -> *const c_char {
        self.inner()
            .map(|inner| inner.request(request))
            .map(|response| self.process_response(response))
            .unwrap_or(ptr::null())
    }
}
