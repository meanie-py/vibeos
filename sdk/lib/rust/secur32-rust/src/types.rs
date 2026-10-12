//! `types.rs` — secur32-private C structs/enums/consts that live in the module's
//! own headers (not any SDK header bindgen sees), so they're hand-translated here
//! and shared across the body modules via `use crate::types::*;` (re-exported by
//! the prelude). Sources: secur32_priv.h (SecureProvider/SecurePackage),
//! wine/ntlm.h (HelperMode/arc4_info/NegoHelper/NtlmCredentials/SignDirection +
//! NTLMSSP_* flags), wine/hmac_md5.h (MD5_CTX/HMAC_MD5_CTX).
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code)]

use crate::prelude::*;
use secur32_sys::{SecPkgInfoA, SecPkgInfoW, SecurityFunctionTableA, SecurityFunctionTableW};
use core::ffi::c_void;

// ---- wine/list.h: `struct list` and its inline helpers, from the shared port ----
pub use winelist_rust::{list, list_add_tail, list_init};

// ---- secur32_priv.h: provider/package registry ---------------------------
#[repr(C)]
#[derive(Copy, Clone)]
pub struct _SecureProvider {
    pub entry: list,
    pub loaded: BOOL,
    pub moduleName: PWSTR,
    pub lib: HMODULE,
    pub fnTableA: SecurityFunctionTableA,
    pub fnTableW: SecurityFunctionTableW,
}
pub type SecureProvider = _SecureProvider;
pub type PSecureProvider = *mut _SecureProvider;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct _SecurePackage {
    pub entry: list,
    pub infoW: SecPkgInfoW,
    pub provider: *mut SecureProvider,
}
pub type SecurePackage = _SecurePackage;
pub type PSecurePackage = *mut _SecurePackage;

// ---- wine/hmac_md5.h: MD5 + HMAC-MD5 contexts ----------------------------
#[repr(C)]
#[derive(Copy, Clone)]
pub struct MD5_CTX {
    pub i: [u32; 2],
    pub buf: [u32; 4],
    pub in_: [u8; 64],
    pub digest: [u8; 16],
}
pub type PMD5_CTX = *mut MD5_CTX;

// md4.h: NB field order differs from MD5_CTX (buf[4] before i[2]).
#[repr(C)]
#[derive(Copy, Clone)]
pub struct MD4_CTX {
    pub buf: [u32; 4],
    pub i: [u32; 2],
    pub in_: [u8; 64],
    pub digest: [u8; 16],
}
pub type PMD4_CTX = *mut MD4_CTX;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct HMAC_MD5_CTX {
    pub ctx: MD5_CTX,
    pub outer_padding: [u8; 64],
}
pub type PHMAC_MD5_CTX = *mut HMAC_MD5_CTX;

// ---- wine/ntlm.h: NTLM SSP helper state ----------------------------------
pub const NTLM_SERVER: u32 = 0;
pub const NTLM_CLIENT: u32 = 1;
pub const NUM_HELPER_MODES: u32 = 2;
pub type HelperMode = u32;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct tag_arc4_info {
    pub x: u8,
    pub y: u8,
    pub state: [u8; 256],
}
pub type arc4_info = tag_arc4_info;
pub type Parc4_info = *mut tag_arc4_info;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct _NegoHelper_crypt_ntlm {
    pub seq_num: ULONG,
    pub a4i: *mut arc4_info,
}
#[repr(C)]
#[derive(Copy, Clone)]
pub struct _NegoHelper_crypt_ntlm2 {
    pub send_sign_key: *mut BYTE,
    pub send_seal_key: *mut BYTE,
    pub recv_sign_key: *mut BYTE,
    pub recv_seal_key: *mut BYTE,
    pub send_seq_no: ULONG,
    pub recv_seq_no: ULONG,
    pub send_a4i: *mut arc4_info,
    pub recv_a4i: *mut arc4_info,
}
#[repr(C)]
#[derive(Copy, Clone)]
pub struct _NegoHelper_crypt {
    pub ntlm: _NegoHelper_crypt_ntlm,
    pub ntlm2: _NegoHelper_crypt_ntlm2,
}
#[repr(C)]
#[derive(Copy, Clone)]
pub struct _NegoHelper {
    pub helper_pid: HANDLE, // __REACTOS__: HANDLE (not pid_t)
    pub mode: HelperMode,
    pub pipe_in: i32,
    pub pipe_out: i32,
    pub major: i32,
    pub minor: i32,
    pub micro: i32,
    pub com_buf: *mut i8,
    pub com_buf_size: i32,
    pub com_buf_offset: i32,
    pub session_key: *mut BYTE,
    pub neg_flags: ULONG,
    pub crypt: _NegoHelper_crypt,
}
pub type NegoHelper = _NegoHelper;
pub type PNegoHelper = *mut _NegoHelper;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct _NtlmCredentials {
    pub mode: HelperMode,
    pub username_arg: *mut i8,
    pub domain_arg: *mut i8,
    pub password: *mut i8, // not nul-terminated
    pub pwlen: i32,
    pub no_cached_credentials: i32,
}
pub type NtlmCredentials = _NtlmCredentials;
pub type PNtlmCredentials = *mut _NtlmCredentials;

pub const NTLM_SEND: u32 = 0;
pub const NTLM_RECV: u32 = 1;
pub type SignDirection = u32;
// bindgen-style enumerator aliases (the translation emits the type-prefixed name).
pub const SignDirection_NTLM_SEND: SignDirection = NTLM_SEND;
pub const SignDirection_NTLM_RECV: SignDirection = NTLM_RECV;

// ---- lmwksta.h WKSTA_USER_INFO_1 (netapi32; not in secur32-sys bindgen) ----
#[repr(C)]
#[derive(Copy, Clone)]
pub struct _WKSTA_USER_INFO_1 {
    pub wkui1_username: LPWSTR,
    pub wkui1_logon_domain: LPWSTR,
    pub wkui1_oth_domains: LPWSTR,
    pub wkui1_logon_server: LPWSTR,
}
pub type WKSTA_USER_INFO_1 = _WKSTA_USER_INFO_1;
pub type PWKSTA_USER_INFO_1 = *mut _WKSTA_USER_INFO_1;
pub type LPWKSTA_USER_INFO_1 = *mut _WKSTA_USER_INFO_1;

// ---- sspi.h INIT_SECURITY_INTERFACE_{A,W} fn-pointer typedefs ----------------
// typedef PSecurityFunctionTable{A,W} (SEC_ENTRY *INIT_SECURITY_INTERFACE_{A,W})(void);
pub type INIT_SECURITY_INTERFACE_A =
    Option<unsafe extern "system" fn() -> PSecurityFunctionTableA>;
pub type INIT_SECURITY_INTERFACE_W =
    Option<unsafe extern "system" fn() -> PSecurityFunctionTableW>;

// NTLMSSP negotiate flags (wine/ntlm.h)
pub const NTLMSSP_NEGOTIATE_UNICODE: u32 = 0x0000_0001;
pub const NTLMSSP_NEGOTIATE_OEM: u32 = 0x0000_0002;
pub const NTLMSSP_REQUEST_TARGET: u32 = 0x0000_0004;
pub const NTLMSSP_NEGOTIATE_SIGN: u32 = 0x0000_0010;
pub const NTLMSSP_NEGOTIATE_SEAL: u32 = 0x0000_0020;
pub const NTLMSSP_NEGOTIATE_DATAGRAM_STYLE: u32 = 0x0000_0040;
pub const NTLMSSP_NEGOTIATE_LM_SESSION_KEY: u32 = 0x0000_0080;
pub const NTLMSSP_NEGOTIATE_NTLM: u32 = 0x0000_0200;
pub const NTLMSSP_NEGOTIATE_DOMAIN_SUPPLIED: u32 = 0x0000_1000;
pub const NTLMSSP_NEGOTIATE_WORKSTATION_SUPPLIED: u32 = 0x0000_2000;
pub const NTLMSSP_NEGOTIATE_LOCAL_CALL: u32 = 0x0000_4000;
pub const NTLMSSP_NEGOTIATE_ALWAYS_SIGN: u32 = 0x0000_8000;
pub const NTLMSSP_NEGOTIATE_TARGET_TYPE_DOMAIN: u32 = 0x0001_0000;
pub const NTLMSSP_NEGOTIATE_TARGET_TYPE_SERVER: u32 = 0x0002_0000;
pub const NTLMSSP_NEGOTIATE_NTLM2: u32 = 0x0008_0000;
pub const NTLMSSP_NEGOTIATE_TARGET_INFO: u32 = 0x0080_0000;
pub const NTLMSSP_NEGOTIATE_128: u32 = 0x2000_0000;
pub const NTLMSSP_NEGOTIATE_KEY_EXCHANGE: u32 = 0x4000_0000;
pub const NTLMSSP_NEGOTIATE_56: u32 = 0x8000_0000;
