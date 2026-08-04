//! msv1_0-rust prelude — the single import surface the translated modules use.
//! Bridges the bindgen `types` island and the translated C. Anything bindgen leaves out
//! (CRT, inlines, sibling-C externs, hand-transcribed consts) is declared here ONCE.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

pub use crate::types::*;
pub use core::ffi::{c_char, c_int, c_long, c_short, c_uchar, c_uint, c_ushort, c_void};
pub use core::ptr::{addr_of, addr_of_mut};

// ================= debug ======================================================================
// The wine/debug.h channel (TRACE/ERR/WARN/FIXME). Kept as no-op MACROS, never
// out-of-line fns: an out-of-line no-op is an optimization barrier that uncloaks latent
// ABI mismatches into ud2 (field manual §9.65).
#[macro_export] macro_rules! TRACE { ($($arg:tt)*) => {{}}; }
#[macro_export] macro_rules! ERR   { ($($arg:tt)*) => {{}}; }
#[macro_export] macro_rules! WARN  { ($($arg:tt)*) => {{}}; }
#[macro_export] macro_rules! FIXME { ($($arg:tt)*) => {{}}; }
pub use crate::{ERR, FIXME, TRACE, WARN};

// ================= CRT ========================================================================
// NEVER hand-write memcpy/memset/memmove/memcmp in Rust: LLVM folds any byte-copy loop back into
// a call to the function itself → infinite self-recursion → silent hang. Import them (AGENTS.md).
extern "C" {
    pub fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn memset(dest: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    pub fn memmove(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn memcmp(a: *const c_void, b: *const c_void, n: usize) -> c_int;
}

// bindgen does not emit NULL (a macro) nor NT_SUCCESS (an inline).
pub const NULL: *mut c_void = core::ptr::null_mut();
#[inline(always)]
pub fn NT_SUCCESS(s: NTSTATUS) -> bool {
    s >= 0
}

// The island re-exports the base NT identity: pub use ntdll_sys::nt::*;

// ================= enum member aliases ========================================================
// bindgen prefixes a C enum's members with the enum's tag, so the bare spelling the C uses is
// absent from the island. Alias, never re-declare: a literal here could drift silently.
//
// 14 of these 28 are `case` labels, which makes island completeness a CORRECTNESS
// property (§9.112): an unresolved name in a `match` arm is not an error, it is a fresh binding
// that matches EVERYTHING and leaves the following arms dead. `verify_match_consts` names them.
//
// NB the scan for this MUST be multi-line — bindgen wraps the declaration when the name is long,
// and a single-line regex reports a confident zero (that is how authz's seven got through).
pub const Batch: _SECURITY_LOGON_TYPE =
    _SECURITY_LOGON_TYPE_Batch;
pub const Interactive: _SECURITY_LOGON_TYPE =
    _SECURITY_LOGON_TYPE_Interactive;
pub const LsaTokenInformationV1: _LSA_TOKEN_INFORMATION_TYPE =
    _LSA_TOKEN_INFORMATION_TYPE_LsaTokenInformationV1;
pub const MsV1_0CacheLogon: _MSV1_0_PROTOCOL_MESSAGE_TYPE =
    _MSV1_0_PROTOCOL_MESSAGE_TYPE_MsV1_0CacheLogon;  // case label
pub const MsV1_0CacheLookup: _MSV1_0_PROTOCOL_MESSAGE_TYPE =
    _MSV1_0_PROTOCOL_MESSAGE_TYPE_MsV1_0CacheLookup;  // case label
pub const MsV1_0ChangeCachedPassword: _MSV1_0_PROTOCOL_MESSAGE_TYPE =
    _MSV1_0_PROTOCOL_MESSAGE_TYPE_MsV1_0ChangeCachedPassword;  // case label
pub const MsV1_0ChangePassword: _MSV1_0_PROTOCOL_MESSAGE_TYPE =
    _MSV1_0_PROTOCOL_MESSAGE_TYPE_MsV1_0ChangePassword;  // case label
pub const MsV1_0DeriveCredential: _MSV1_0_PROTOCOL_MESSAGE_TYPE =
    _MSV1_0_PROTOCOL_MESSAGE_TYPE_MsV1_0DeriveCredential;  // case label
pub const MsV1_0EnumerateUsers: _MSV1_0_PROTOCOL_MESSAGE_TYPE =
    _MSV1_0_PROTOCOL_MESSAGE_TYPE_MsV1_0EnumerateUsers;  // case label
pub const MsV1_0GenericPassthrough: _MSV1_0_PROTOCOL_MESSAGE_TYPE =
    _MSV1_0_PROTOCOL_MESSAGE_TYPE_MsV1_0GenericPassthrough;  // case label
pub const MsV1_0GetUserInfo: _MSV1_0_PROTOCOL_MESSAGE_TYPE =
    _MSV1_0_PROTOCOL_MESSAGE_TYPE_MsV1_0GetUserInfo;  // case label
pub const MsV1_0InteractiveLogon: _MSV1_0_LOGON_SUBMIT_TYPE =
    _MSV1_0_LOGON_SUBMIT_TYPE_MsV1_0InteractiveLogon;
pub const MsV1_0InteractiveProfile: _MSV1_0_PROFILE_BUFFER_TYPE =
    _MSV1_0_PROFILE_BUFFER_TYPE_MsV1_0InteractiveProfile;
pub const MsV1_0Lm20ChallengeRequest: _MSV1_0_PROTOCOL_MESSAGE_TYPE =
    _MSV1_0_PROTOCOL_MESSAGE_TYPE_MsV1_0Lm20ChallengeRequest;  // case label
pub const MsV1_0Lm20GetChallengeResponse: _MSV1_0_PROTOCOL_MESSAGE_TYPE =
    _MSV1_0_PROTOCOL_MESSAGE_TYPE_MsV1_0Lm20GetChallengeResponse;  // case label
pub const MsV1_0Lm20LogonProfile: _MSV1_0_PROFILE_BUFFER_TYPE =
    _MSV1_0_PROFILE_BUFFER_TYPE_MsV1_0Lm20LogonProfile;
pub const MsV1_0ReLogonUsers: _MSV1_0_PROTOCOL_MESSAGE_TYPE =
    _MSV1_0_PROTOCOL_MESSAGE_TYPE_MsV1_0ReLogonUsers;  // case label
pub const MsV1_0SubAuth: _MSV1_0_PROTOCOL_MESSAGE_TYPE =
    _MSV1_0_PROTOCOL_MESSAGE_TYPE_MsV1_0SubAuth;  // case label
pub const MsV1_0WorkstationUnlockLogon: _MSV1_0_LOGON_SUBMIT_TYPE =
    _MSV1_0_LOGON_SUBMIT_TYPE_MsV1_0WorkstationUnlockLogon;
pub const NetLogonLmKey: _LSA_SAM_NETLOGON_TYPE =
    _LSA_SAM_NETLOGON_TYPE_NetLogonLmKey;
pub const Network: _SECURITY_LOGON_TYPE =
    _SECURITY_LOGON_TYPE_Network;
pub const NtlmLsaMode: _NTLM_MODE =
    _NTLM_MODE_NtlmLsaMode;  // case label
pub const NtlmUnknownMode: _NTLM_MODE =
    _NTLM_MODE_NtlmUnknownMode;
pub const NtlmUserMode: _NTLM_MODE =
    _NTLM_MODE_NtlmUserMode;  // case label
pub const PolicyAccountDomainInformation: _POLICY_INFORMATION_CLASS =
    _POLICY_INFORMATION_CLASS_PolicyAccountDomainInformation;
pub const Service: _SECURITY_LOGON_TYPE =
    _SECURITY_LOGON_TYPE_Service;
pub const UserAllInformation: _USER_INFORMATION_CLASS =
    _USER_INFORMATION_CLASS_UserAllInformation;
pub const UserInternal2Information: _USER_INFORMATION_CLASS =
    _USER_INFORMATION_CLASS_UserInternal2Information;

// A wide string literal, NUL-terminated, with one stable address for the program's life.
// `static` and not `const`: a const is inlined at each use site, so `.as_ptr()` would hand the
// callee the address of a temporary. ASCII-only by construction, so a non-ASCII literal fails
// at COMPILE time instead of silently truncating.
//
// This replaces the byte-string-as-wide-pointer form, which is wrong twice over: `b"..."` has
// ALIGNMENT 1 (an odd address is an amd64-only 0x1E STATUS_DATATYPE_MISALIGNMENT), and Rust
// does not append the SECOND zero byte a UTF-16 terminator needs — `b"\\\0"` is one code unit
// with no terminator at all, and an odd byte count cannot be whole u16 units.
#[macro_export]
macro_rules! w {
    ($s:expr) => {{
        const INPUT: &[u8] = $s.as_bytes();
        static OUT: [u16; INPUT.len() + 1] = {
            let mut out = [0u16; INPUT.len() + 1];
            let mut i = 0;
            while i < INPUT.len() {
                assert!(INPUT[i] < 0x80, "w!() takes ASCII literals only");
                out[i] = INPUT[i] as u16;
                i += 1;
            }
            out // trailing element stays 0 => NUL-terminated
        };
        OUT.as_ptr()
    }};
}
pub use crate::w;
