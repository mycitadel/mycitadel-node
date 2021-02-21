// MyCitadel C bindings library (libmycitadel)
// Written in 2020 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.

// TODO: Consider moving to rust-amplify library

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

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

impl ToString for *const c_char {
    fn to_string(&self) -> String {
        unsafe { CStr::from_ptr(ptr) }.to_string_lossy().to_string()
    }
}

impl ToString for *mut c_char {
    fn to_string(&self) -> String {
        unsafe { CStr::from_ptr(ptr) }.to_string_lossy().to_string()
    }
}

pub fn ptr_to_string(ptr: *const c_char) -> String {
    unsafe { CStr::from_ptr(ptr) }.to_string_lossy().to_string()
}
