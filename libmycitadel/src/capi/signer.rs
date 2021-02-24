// C library for building descriptor-based bitcoin wallets
//
// Written in 2021 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the Apache 2.0 License
// along with this software.
// If not, see <https://opensource.org/licenses/Apache-2.0>.

use libc::c_char;
use rand::RngCore;
use std::convert::TryFrom;
use std::ffi::{CStr, CString};
use std::ops::Try;
use std::slice;
use std::str::{FromStr, Utf8Error};

use bip39::Mnemonic;
use bitcoin::util::bip32::{
    self, ChildNumber, DerivationPath, Error, ExtendedPrivKey, ExtendedPubKey,
};
use bitcoin::Network;
use wallet::bip32::{
    BranchStep, ChildIndex, HardenedIndex, HardenedNormalSplit, PubkeyChain,
    TerminalStep, XpubRef,
};

pub trait Wipe {
    unsafe fn wipe(self);
}

pub trait Clean {
    unsafe fn clean(&self);
}

impl Wipe for CString {
    unsafe fn wipe(self) {
        let len = self.as_bytes().len();
        let ptr = self.as_ptr() as *mut c_char;
        for i in 0..len as isize {
            *ptr.offset(i) = 0;
        }
        std::mem::drop(self);
    }
}

impl Clean for CStr {
    unsafe fn clean(&self) {
        let ptr = self.as_ptr() as *mut c_char;
        let mut i = 0;
        while *ptr.offset(i) != 0 {
            *ptr.offset(i) = 0;
            i += 1;
        }
    }
}

#[derive(
    Clone,
    Copy,
    Ord,
    PartialOrd,
    Eq,
    PartialEq,
    Hash,
    Debug,
    Display,
    Error,
    From,
)]
#[allow(non_camel_case_types)]
#[repr(C)]
#[display(doc_comments)]
pub enum err_type {
    #[display("")]
    success = 0,

    /// got a null pointer as one of the function arguments
    null_pointer,

    /// result data must be a valid string which does not contain zero bytes
    invalid_result_data,

    /// invalid mnemonic string
    #[from(bip39::Error)]
    invalid_mnemonic,

    /// invalid UTF-8 string
    #[from(Utf8Error)]
    invalid_utf8_string,

    /// wrong BIP32 extended public or private key data
    wrong_extended_key,

    /// unable to derive hardened path from a public key
    unable_to_derive_hardened,

    /// invalid derivation path
    invalid_derivation_path,

    /// general BIP32-specific failure
    bip32_failure,
}

impl Default for err_type {
    fn default() -> Self {
        err_type::success
    }
}

impl From<bip32::Error> for err_type {
    fn from(err: bip32::Error) -> Self {
        match err {
            Error::CannotDeriveFromHardenedKey => {
                err_type::unable_to_derive_hardened
            }

            Error::InvalidChildNumber(_)
            | Error::InvalidChildNumberFormat
            | Error::InvalidDerivationPathFormat => {
                err_type::invalid_derivation_path
            }

            Error::Base58(_)
            | Error::UnknownVersion(_)
            | Error::WrongExtendedKeyLength(_) => err_type::wrong_extended_key,

            Error::RngError(_) | Error::Ecdsa(_) => err_type::bip32_failure,
        }
    }
}

#[allow(non_camel_case_types)]
#[repr(C)]
pub struct string_result_t {
    pub code: err_type,
    pub details: result_details_t,
}

impl string_result_t {
    pub fn success(data: impl ToString) -> string_result_t {
        let (code, details) = match CString::new(data.to_string()) {
            Ok(s) => {
                (err_type::success, result_details_t { data: s.into_raw() })
            }
            Err(err) => (err_type::invalid_result_data, err.into()),
        };
        string_result_t { code, details }
    }

    pub fn error(code: err_type) -> string_result_t {
        string_result_t {
            code,
            details: result_details_t {
                data: CString::new(code.to_string())
                    .expect("Null byte in error_t code doc comments")
                    .into_raw(),
            },
        }
    }

    pub fn is_success(&self) -> bool {
        self.code == err_type::success
    }
}

impl Try for string_result_t {
    type Ok = result_details_t;
    type Error = err_type;

    fn into_result(self) -> Result<Self::Ok, Self::Error> {
        if self.is_success() {
            Ok(self.details)
        } else {
            Err(self.code)
        }
    }

    fn from_error(v: Self::Error) -> Self {
        v.into()
    }

    fn from_ok(v: Self::Ok) -> Self {
        string_result_t {
            code: err_type::success,
            details: v,
        }
    }
}

impl<E> From<E> for string_result_t
where
    E: std::error::Error + Clone + Into<err_type>,
{
    fn from(err: E) -> Self {
        string_result_t {
            code: err.clone().into(),
            details: result_details_t::from(err),
        }
    }
}

#[no_mangle]
pub extern "C" fn is_success(result: string_result_t) -> bool {
    result.is_success()
}

#[allow(non_camel_case_types)]
#[repr(C)]
pub union result_details_t {
    pub data: *const c_char,
    pub error: *const c_char,
}

impl<E> From<E> for result_details_t
where
    E: std::error::Error,
{
    fn from(err: E) -> Self {
        result_details_t {
            error: CString::new(err.to_string())
                .unwrap_or(
                    CString::new("no string error representation").unwrap(),
                )
                .into_raw(),
        }
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
#[allow(non_camel_case_types)]
#[repr(C)]
pub enum bip39_mnemonic_type {
    words_12,
    words_15,
    words_18,
    words_21,
    words_24,
}

impl bip39_mnemonic_type {
    pub fn byte_len(self) -> usize {
        match self {
            bip39_mnemonic_type::words_12 => 16,
            bip39_mnemonic_type::words_15 => 160 / 8,
            bip39_mnemonic_type::words_18 => 192 / 8,
            bip39_mnemonic_type::words_21 => 224 / 8,
            bip39_mnemonic_type::words_24 => 32,
        }
    }

    pub fn word_len(self) -> usize {
        (self.byte_len() * 8 + self.byte_len() * 8 / 32) / 11
    }
}

#[no_mangle]
pub unsafe extern "C" fn result_destroy(result: string_result_t) {
    let ptr = result.details.data;
    if ptr.is_null() {
        return;
    }
    let cs = CString::from_raw(ptr as *mut c_char);
    cs.wipe();
}

/// Creates a rust-owned mnemonic string. You MUSt always call
/// [`string_destroy`] right after storing the mnemonic string and
/// do not call other methods from this library on that mnemonic. If you need
/// to call [`bip39_master_xpriv`] you MUST read mnemonic again and provide
/// unowned string to the rust.
#[no_mangle]
pub extern "C" fn bip39_mnemonic_create(
    entropy: *const u8,
    mnemonic_type: bip39_mnemonic_type,
) -> string_result_t {
    let entropy = if entropy.is_null() {
        let mut inner = vec![0u8; mnemonic_type.byte_len()];
        rand::thread_rng().fill_bytes(&mut inner);
        inner
    } else {
        unsafe { slice::from_raw_parts(entropy, mnemonic_type.byte_len()) }
            .to_vec()
    };
    let mnemonic = bip39::Mnemonic::from_entropy(&entropy)?;
    string_result_t::success(mnemonic.as_str())
}

#[no_mangle]
pub extern "C" fn bip39_master_xpriv(
    seed_phrase: *mut c_char,
    passwd: *mut c_char,
    wipe: bool,
    testnet: bool,
) -> string_result_t {
    if seed_phrase.is_null() {
        Err(err_type::null_pointer)?
    }

    let password = if passwd.is_null() {
        ""
    } else {
        unsafe { CStr::from_ptr(passwd).to_str()? }
    };

    let mut seed = {
        let seed_phrase = unsafe { CString::from_raw(seed_phrase) };
        let mnemonic = Mnemonic::from_str(seed_phrase.to_str()?)?;
        let seed = mnemonic.to_seed(&password);
        if wipe {
            let len = mnemonic.as_str().len();
            let ptr = mnemonic.as_str().as_ptr() as *mut c_char;
            for i in 0..len as isize {
                unsafe { *ptr.offset(i) = 0 };
            }
            unsafe { seed_phrase.wipe() };
        }
        seed
    };
    let mut xpriv = ExtendedPrivKey::new_master(
        if testnet {
            Network::Testnet
        } else {
            Network::Bitcoin
        },
        &seed,
    )?;
    seed.fill(0u8);
    if wipe && !passwd.is_null() {
        let len = password.len();
        for i in 0..len as isize {
            unsafe { *passwd.offset(i) = 0 };
        }
    }
    let xpriv_str = xpriv.to_string();
    let ptr = xpriv.private_key.key.as_mut_ptr();
    for i in 0..32 {
        unsafe {
            *ptr.offset(i) = 0;
        }
    }
    string_result_t::success(&xpriv_str)
}

#[no_mangle]
pub extern "C" fn bip32_scoped_xpriv(
    master: *const c_char,
    clean: bool,
    derivation: *const c_char,
) -> string_result_t {
    let master_cstr = unsafe { CStr::from_ptr(master) };
    let mut master = ExtendedPrivKey::from_str(master_cstr.to_str()?)?;

    let derivation = unsafe { CStr::from_ptr(derivation).to_str()? };
    let derivation = derivation.replace("/*", "");
    let derivation = DerivationPath::from_str(&derivation)?;
    let (hardened, _) = derivation.hardened_normal_split();

    let mut xpriv = master.derive_priv(&wallet::SECP256K1, &hardened)?;

    if clean {
        unsafe { master_cstr.clean() };
    }

    let xpriv_str = xpriv.to_string();
    let ptr1 = master.private_key.key.as_mut_ptr();
    let ptr2 = xpriv.private_key.key.as_mut_ptr();
    for i in 0..32 {
        unsafe {
            *ptr1.offset(i) = 0;
            *ptr2.offset(i) = 0;
        }
    }
    string_result_t::success(&xpriv_str)
}

#[no_mangle]
pub extern "C" fn bip32_xpriv_to_xpub(
    xpriv: *mut c_char,
    wipe: bool,
) -> string_result_t {
    let xpriv_cstring = unsafe { CString::from_raw(xpriv) };

    let mut xpriv = ExtendedPrivKey::from_str(xpriv_cstring.to_str()?)?;
    let xpub = ExtendedPubKey::from_private(&wallet::SECP256K1, &xpriv);
    if wipe {
        unsafe { xpriv_cstring.wipe() };
    }

    let ptr = xpriv.private_key.key.as_mut_ptr();
    for i in 0..32 {
        unsafe {
            *ptr.offset(i) = 0;
        }
    }
    string_result_t::success(&xpub)
}

#[no_mangle]
pub extern "C" fn bip32_pubkey_chain_create(
    master_xpriv: *mut c_char,
    clean: bool,
    derivation: *const c_char,
) -> string_result_t {
    let master_cstr = unsafe { CStr::from_ptr(master_xpriv) };

    let derivation = unsafe { CStr::from_ptr(derivation).to_str()? };
    let derivation = derivation.replace("/*", "");
    let derivation = DerivationPath::from_str(&derivation)?;
    let (hardened, unhardened) = derivation.hardened_normal_split();

    let mut master_xpriv = ExtendedPrivKey::from_str(master_cstr.to_str()?)?;
    let master_xpub =
        ExtendedPubKey::from_private(&wallet::SECP256K1, &master_xpriv);
    let mut xpriv = master_xpriv.derive_priv(&wallet::SECP256K1, &hardened)?;
    if clean {
        unsafe { master_cstr.clean() };
    }
    let xpub = ExtendedPubKey::from_private(&wallet::SECP256K1, &xpriv);

    let ptr1 = master_xpriv.private_key.key.as_mut_ptr();
    let ptr2 = xpriv.private_key.key.as_mut_ptr();
    for i in 0..32 {
        unsafe {
            *ptr1.offset(i) = 0;
            *ptr2.offset(i) = 0;
        }
    }

    let mut source_path: Vec<ChildNumber> = hardened.into();
    let branch_index = source_path
        .pop()
        .map(HardenedIndex::try_from)
        .transpose()?
        .ok_or(bip32::Error::InvalidDerivationPathFormat)?;
    let mut terminal_path: Vec<TerminalStep> = unhardened
        .into_iter()
        .map(|idx| {
            TerminalStep::from_index(idx)
                .expect("Derivation::hardened_normal_split is broken")
        })
        .collect();
    terminal_path.push(TerminalStep::Wildcard);
    let pubkey_chain = PubkeyChain {
        seed_based: true,
        master: XpubRef::Fingerprint(master_xpub.fingerprint()),
        source_path: source_path.into_iter().map(BranchStep::from).collect(),
        branch_index,
        branch_xpub: xpub,
        revocation_seal: None,
        terminal_path,
    };

    string_result_t::success(&pubkey_chain)
}

#[no_mangle]
pub extern "C" fn psbt_sign(
    _psbt: *const c_char,
    _xpriv: *const c_char,
    _wipe: bool,
) -> string_result_t {
    unimplemented!()
}
