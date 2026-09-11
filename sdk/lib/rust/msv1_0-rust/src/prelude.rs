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

// ================= CRT ========================================================================
// NEVER hand-write memcpy/memset/memmove/memcmp in Rust: LLVM folds any byte-copy loop back into
// a call to the function itself → infinite self-recursion → silent hang. Import them (AGENTS.md).
extern "C" {
    pub fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn memset(dest: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    pub fn memmove(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn memcmp(a: *const c_void, b: *const c_void, n: usize) -> c_int;
}

// The wide/narrow CRT the module uses. Declared here and not by the island because the CRT
// headers are outside the allowlist — pulling <string.h> in would re-emit a second set of
// every str* under a different Rust identity.
extern "C" {
    pub fn wcslen(s: *const WCHAR) -> usize;
    pub fn wcscpy(dst: *mut WCHAR, src: *const WCHAR) -> *mut WCHAR;
    pub fn wcscat(dst: *mut WCHAR, src: *const WCHAR) -> *mut WCHAR;
    pub fn wcschr(s: *const WCHAR, c: WCHAR) -> *mut WCHAR;
    pub fn _wcsicmp(a: *const WCHAR, b: *const WCHAR) -> c_int;
    pub fn strcpy(dst: *mut c_char, src: *const c_char) -> *mut c_char;
}

// bindgen does not emit NULL (a macro) nor NT_SUCCESS (an inline).
pub const NULL: *mut c_void = core::ptr::null_mut();
#[inline(always)]
pub fn NT_SUCCESS(s: NTSTATUS) -> bool {
    s >= 0
}

// ================= constants bindgen cannot fold ==============================================
// Every one of these is a CAST EXPRESSION in its header (`((NTSTATUS)0xC0000064)`), which
// bindgen does not evaluate, so the island carries none of them. TRANSCRIBED FROM THE HEADER,
// value and all, never from memory — inventing NT status codes from recall is exactly how 61
// wrong kernel stop codes shipped once (AGENTS.md, §9.113). Header + line cited per line so the
// next reader can re-check in one grep.
//
// psdk/ntstatus.h:
pub const STATUS_NO_SUCH_USER: NTSTATUS = 0xC0000064u32 as NTSTATUS;
pub const STATUS_ACCOUNT_RESTRICTION: NTSTATUS = 0xC000006Eu32 as NTSTATUS;
pub const STATUS_WRONG_PASSWORD: NTSTATUS = 0xC000006Au32 as NTSTATUS;
pub const STATUS_INTERNAL_ERROR: NTSTATUS = 0xC00000E5u32 as NTSTATUS;
pub const STATUS_PASSWORD_MUST_CHANGE: NTSTATUS = 0xC0000224u32 as NTSTATUS;
pub const STATUS_PASSWORD_EXPIRED: NTSTATUS = 0xC0000071u32 as NTSTATUS;
pub const STATUS_INVALID_WORKSTATION: NTSTATUS = 0xC0000070u32 as NTSTATUS;
pub const STATUS_INVALID_LOGON_HOURS: NTSTATUS = 0xC000006Fu32 as NTSTATUS;
pub const STATUS_BAD_VALIDATION_CLASS: NTSTATUS = 0xC00000A7u32 as NTSTATUS;
pub const STATUS_ACCOUNT_LOCKED_OUT: NTSTATUS = 0xC0000234u32 as NTSTATUS;
pub const STATUS_ACCOUNT_EXPIRED: NTSTATUS = 0xC0000193u32 as NTSTATUS;
pub const STATUS_ACCOUNT_DISABLED: NTSTATUS = 0xC0000072u32 as NTSTATUS;

// xdk/ntdef.template.h `#define UNICODE_NULL ((WCHAR)0)` — the module uses it both as a value
// (`*pEnd = UNICODE_NULL`) and inside `sizeof(UNICODE_NULL)`, so it must be a typed WCHAR.
pub const UNICODE_NULL: WCHAR = 0;
// psdk/winbase.h `#define INVALID_HANDLE_VALUE ((HANDLE)(LONG_PTR)-1)`, compared against a
// cast ClientRequest. `usize::MAX` and not `-1isize`: HANDLE is a pointer, and the C cast is
// through LONG_PTR, so the bit pattern is all-ones at the target's pointer width.
pub const INVALID_HANDLE_VALUE: HANDLE = usize::MAX as HANDLE;
// psdk/winerror.h `#define S_OK ((HRESULT)0L)` — the intsafe converters return HRESULT.
pub const S_OK: HRESULT = 0;
// xdk/setypes.h:554 `#define SECURITY_NT_AUTHORITY {0,0,0,0,0,5}` — a brace initializer for the
// six-byte Value member, NOT the struct: the C writes `SID_IDENTIFIER_AUTHORITY A =
// {SECURITY_NT_AUTHORITY}`, whose outer braces are the struct and inner the array. Kept as the
// array so both spellings in the module (`{ Value: SECURITY_NT_AUTHORITY }` and a whole-struct
// initializer) mean the same thing they do in C.
pub const SECURITY_NT_AUTHORITY: [BYTE; 6] = [0, 0, 0, 0, 0, 5];

// ================= header macros ==============================================================
// C macros never reach the island. All of these stay `macro_rules!` and never become
// out-of-line fns: an out-of-line no-op/forwarder is an optimization barrier that uncloaks
// latent ABI mismatches into `ud2` (field manual §9.65).

// ndk/rtlfuncs.h:1127 `#define RtlGetProcessHeap() (NtCurrentPeb()->ProcessHeap)` — a macro
// over the PEB, so there is no symbol to import.
#[macro_export]
macro_rules! RtlGetProcessHeap { () => { (*NtCurrentPeb()).ProcessHeap }; }

// winnt.h macros over the CRT block ops. Trailing comma tolerated: the C is line-wrapped.
#[macro_export]
macro_rules! RtlCopyMemory {
    ($dst:expr, $src:expr, $len:expr $(,)?) => {
        core::ptr::copy_nonoverlapping($src as *const u8, $dst as *mut u8, $len as usize)
    };
}
// The zeroing macros live in `win32macros-rust`, ported once from the header and
// known-answer tested there. Re-exported under the same names, so no call site changed.
pub use win32macros_rust::{RtlZeroMemory, ZeroMemory};
// `#define RtlEqualMemory(d,s,l) (!memcmp((d),(s),(l)))` — a BOOLEAN, and the C uses it
// directly as an `if` condition, so this yields a Rust bool.
#[macro_export]
macro_rules! RtlEqualMemory {
    ($a:expr, $b:expr, $len:expr $(,)?) => {
        $crate::prelude::memcmp($a as *const c_void, $b as *const c_void, $len as usize) == 0
    };
}

// msv1_0.h:9 — the module's own macro, verbatim:
//   #define FIXUP_POINTER(Pointer, Offset) \
//       ((Pointer != NULL) ? ((PWSTR)((ULONG_PTR)Pointer + Offset)) : NULL)
// Relocates a client-supplied pointer into the LSA's copy of the submit buffer. The NULL guard
// is load-bearing: without it a NULL Buffer would become the bare Offset and be dereferenced.
#[macro_export]
macro_rules! FIXUP_POINTER {
    ($ptr:expr, $off:expr $(,)?) => {{
        let p = $ptr;
        if !p.is_null() {
            (p as ULONG_PTR).wrapping_add($off as ULONG_PTR) as PWSTR
        } else {
            core::ptr::null_mut()
        }
    }};
}

// xdk/ntbasedef.h:712 `#define ARRAYSIZE(A) RTL_NUMBER_OF_V2(A)` — the element count of a C
// array. Used to size a stack WCHAR[] before GetComputerNameW, so it must be the ARRAY length,
// not the byte length.
#[macro_export]
macro_rules! ARRAYSIZE { ($a:expr) => { ($a).len() as DWORD }; }

// ndk/rtltypes.h:327 `RTL_CONSTANT_STRING` — a compile-time UNICODE_STRING over a wide literal.
// Length excludes the terminator, MaximumLength includes it, both in BYTES. `w!` supplies the
// backing storage with one stable address (see its comment).
#[macro_export]
macro_rules! RTL_CONSTANT_STRING {
    ($s:expr) => {{
        const N: usize = $s.len(); // ASCII, so chars == UTF-16 code units
        UNICODE_STRING {
            Length: (N * 2) as USHORT,
            MaximumLength: ((N + 1) * 2) as USHORT,
            Buffer: $crate::w!($s) as *mut WCHAR,
        }
    }};
}

// psdk/intsafe.h:296 `DEFINE_SAFE_CONVERT_UTOX(UIntPtrToUShort, UINT_PTR, USHORT)` — a static
// inline that writes the narrowed value and returns S_OK, or writes USHORT_ERROR and returns
// INTSAFE_E_ARITHMETIC_OVERFLOW. The overflow arm is why this cannot be a bare `as` cast: the
// one call site branches on the HRESULT to reject an over-long computer name.
#[inline(always)]
pub unsafe fn UIntPtrToUShort(operand: UINT_PTR, result: *mut USHORT) -> HRESULT {
    if operand <= u16::MAX as UINT_PTR {
        *result = operand as USHORT;
        S_OK
    } else {
        *result = u16::MAX; // USHORT_ERROR
        0x80070216u32 as HRESULT // INTSAFE_E_ARITHMETIC_OVERFLOW
    }
}

pub use crate::{ARRAYSIZE, FIXUP_POINTER, RtlCopyMemory, RtlEqualMemory, RtlGetProcessHeap, RTL_CONSTANT_STRING};

// ================= the crate's own cross-TU surface ===========================================
// `ntlm/util.c` and `sam.c` define helpers the other TUs call. In C they resolve through
// util.h/sam.h; in Rust each TU is a module, so the prelude re-exports them the same way it
// re-exports the island. Named explicitly, not globbed: a glob would also pull each module's
// file-scope statics into every other module.
pub use crate::msv1_0::sam::SamValidateUser;
pub use crate::ntlm::util::{
    NtlmAllocateClientBuffer, NtlmCopyToClientBuffer, NtlmFixupAStr, NtlmFixupAndValidateUStr,
    NtlmFreeClientBuffer, NtlmUStrAlloc, NtlmUStrFree, NtlmUStrWriteToStruct,
};

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
