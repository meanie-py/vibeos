//! Crate-wide prelude: the universal symbols the per-file translations expect in
//! scope. Body modules add `use crate::prelude::*;`.
//!
//! Base = the user-mode Native API surface from ntdll-sys (Nt*/Rtl*/Ldr* + NT
//! base types). On top we add the SSPI / LSA-LPC / NTLM-MSV / wincred types
//! (curated from secur32-sys so base types don't collide with ntdll_sys::nt),
//! the SEC_E_*/SEC_I_* status codes (bindgen can't const-eval their HRESULT
//! macros), TRUE/FALSE as BOOL, and the no-op CRT/Wine helpers. Grows per wave.
#![allow(non_snake_case, non_upper_case_globals, dead_code)]

pub use ntdll_sys::nt::*;

// Win32 module-handle types exist in BOTH ntdll-sys and secur32-sys as distinct
// nominal types (same layout). secur32-sys structs (SecureProvider.lib etc.)
// embed the secur32-sys versions, so standardize the crate on those (explicit
// `use` wins over the nt::* glob — no ambiguity).
pub use secur32_sys::{HMODULE, HINSTANCE, HINSTANCE__};

// TRUE/FALSE: bindgen types the bare `#define TRUE 1` as u32, but BOOL is c_int
// (i32). Explicit BOOL-typed defs here shadow the nt glob's u32 versions. Files
// import ONLY this prelude so there's a single, correctly-typed pair.
pub const TRUE: BOOL = 1;
pub const FALSE: BOOL = 0;

// ---- SSPI core type surface (curated from secur32-sys) -------------------
pub use secur32_sys::{
    // buffers + handles
    SecBuffer, PSecBuffer, SecBufferDesc, PSecBufferDesc,
    SecHandle, PSecHandle, CredHandle, PCredHandle, CtxtHandle, PCtxtHandle,
    SECURITY_STATUS, SEC_CHAR, SEC_WCHAR, SEC_GET_KEY_FN,
    TimeStamp, PTimeStamp,
    // package info
    SecPkgInfoA, PSecPkgInfoA, SecPkgInfoW, PSecPkgInfoW,
    // function tables
    SecurityFunctionTableA, PSecurityFunctionTableA,
    SecurityFunctionTableW, PSecurityFunctionTableW,
    // context-attribute query structs (A/W families)
    SecPkgContext_Sizes, SecPkgContext_StreamSizes,
    SecPkgContext_NamesA, SecPkgContext_NamesW,
    SecPkgContext_Lifespan, SecPkgContext_Flags,
    SecPkgContext_PackageInfoA, SecPkgContext_PackageInfoW,
    SecPkgContext_NegotiationInfoA, SecPkgContext_NegotiationInfoW,
    SecPkgContext_AuthorityA, SecPkgContext_AuthorityW,
    SecPkgContext_KeyInfoA, SecPkgContext_KeyInfoW,
    SecPkgContext_SessionKey, SecPkgContext_PasswordExpiry,
    // credentials (wincred)
    CREDENTIALA, PCREDENTIALA, CREDENTIALW, PCREDENTIALW,
    // auth identity (sspi)
    SEC_WINNT_AUTH_IDENTITY_A, PSEC_WINNT_AUTH_IDENTITY_A,
    SEC_WINNT_AUTH_IDENTITY_W, PSEC_WINNT_AUTH_IDENTITY_W,
    // logon-session / policy enums (ntsecapi)
    SECURITY_LOGON_SESSION_DATA, PSECURITY_LOGON_SESSION_DATA,
    POLICY_NOTIFICATION_INFORMATION_CLASS,
};

// ---- SSPI package/buffer/negotiation flag consts (from secur32-sys) -------
pub use secur32_sys::{
    SECPKG_FLAG_INTEGRITY, SECPKG_FLAG_PRIVACY, SECPKG_FLAG_TOKEN_ONLY,
    SECPKG_FLAG_DATAGRAM, SECPKG_FLAG_CONNECTION, SECPKG_FLAG_MULTI_REQUIRED,
    SECPKG_FLAG_CLIENT_ONLY, SECPKG_FLAG_EXTENDED_ERROR, SECPKG_FLAG_IMPERSONATION,
    SECPKG_FLAG_ACCEPT_WIN32_NAME, SECPKG_FLAG_STREAM, SECPKG_FLAG_NEGOTIABLE,
    SECPKG_FLAG_GSS_COMPATIBLE, SECPKG_FLAG_LOGON, SECPKG_FLAG_ASCII_BUFFERS,
    SECPKG_FLAG_FRAGMENT, SECPKG_FLAG_MUTUAL_AUTH, SECPKG_FLAG_DELEGATION,
    SECPKG_FLAG_READONLY_WITH_CHECKSUM, SECPKG_FLAG_RESTRICTED_TOKENS,
    SECPKG_FLAG_NEGO_EXTENDER, SECPKG_FLAG_NEGOTIABLE2,
    SECPKG_FLAG_APPCONTAINER_PASSTHROUGH, SECPKG_FLAG_APPCONTAINER_CHECKS,
    SECPKG_FLAG_APPLY_LOOPBACK,
    SECPKG_NEGOTIATION_COMPLETE, SECPKG_NEGOTIATION_OPTIMISTIC,
    SECPKG_NEGOTIATION_IN_PROGRESS,
    SEC_WINNT_AUTH_IDENTITY_ANSI, SEC_WINNT_AUTH_IDENTITY_UNICODE,
};

// ---- consts bindgen can't see (function-like / HRESULT macros) ------------
pub const CP_ACP: UINT = 0;
pub const SEC_E_ILLEGAL_MESSAGE: SECURITY_STATUS = 0x8009_0326u32 as i32;
// rpcdce.h authentication-service identifiers
pub const RPC_C_AUTHN_GSS_NEGOTIATE: ULONG = 9;
pub const RPC_C_AUTHN_GSS_KERBEROS: ULONG = 16;
// lsass/lsass.h LPC request opcodes — exact _LSA_API_NUMBER enum order
// (CALL_AUTHENTICATION_PACKAGE=0 … POLICY_CHANGE_NOTIFY=6). VERIFIED vs header.
pub const LSASS_REQUEST_CALL_AUTHENTICATION_PACKAGE: ULONG = 0;
pub const LSASS_REQUEST_DEREGISTER_LOGON_PROCESS: ULONG = 1;
pub const LSASS_REQUEST_LOGON_USER: ULONG = 2;
pub const LSASS_REQUEST_LOOKUP_AUTHENTICATION_PACKAGE: ULONG = 3;
pub const LSASS_REQUEST_ENUM_LOGON_SESSIONS: ULONG = 4;
pub const LSASS_REQUEST_GET_LOGON_SESSION_DATA: ULONG = 5;
pub const LSASS_REQUEST_POLICY_CHANGE_NOTIFY: ULONG = 6;

// LSA-LPC port-message structs (lsass/lsass.h) from secur32-sys.
pub use secur32_sys::{
    LSA_API_MSG, PLSA_API_MSG, LSA_CONNECTION_INFO, PLSA_CONNECTION_INFO,
    LSA_API_NUMBER, PLSA_API_NUMBER,
};
// LSA_PORT_MESSAGE_SIZE = sizeof(LSA_API_MSG); LSA_PORT_DATA_SIZE(c) is the
// function-like macro sizeof(ULONG)+sizeof(NTSTATUS)+sizeof(c).
pub const LSA_PORT_MESSAGE_SIZE: usize = core::mem::size_of::<LSA_API_MSG>();
// C: #define LSA_PORT_DATA_SIZE(c) (sizeof(ULONG)+sizeof(NTSTATUS)+sizeof(c))
// `c` is the message-body field whose size is taken (sizeof the value).
#[macro_export]
macro_rules! LSA_PORT_DATA_SIZE {
    ($c:expr) => {
        (core::mem::size_of::<u32>() + core::mem::size_of::<i32>()
            + core::mem::size_of_val(&$c))
    };
}
pub use crate::LSA_PORT_DATA_SIZE;

// ---- SSPI status codes (HRESULT macros bindgen can't const-eval) ----------
// SECURITY_STATUS is LONG; these are the canonical SEC_E_*/SEC_I_* values.
pub const SEC_E_OK: SECURITY_STATUS = 0x0000_0000u32 as i32;
pub const SEC_E_INSUFFICIENT_MEMORY: SECURITY_STATUS = 0x8009_0300u32 as i32;
pub const SEC_E_INVALID_HANDLE: SECURITY_STATUS = 0x8009_0301u32 as i32;
pub const SEC_E_UNSUPPORTED_FUNCTION: SECURITY_STATUS = 0x8009_0302u32 as i32;
pub const SEC_E_TARGET_UNKNOWN: SECURITY_STATUS = 0x8009_0303u32 as i32;
pub const SEC_E_INTERNAL_ERROR: SECURITY_STATUS = 0x8009_0304u32 as i32;
pub const SEC_E_SECPKG_NOT_FOUND: SECURITY_STATUS = 0x8009_0305u32 as i32;
pub const SEC_E_NOT_OWNER: SECURITY_STATUS = 0x8009_0306u32 as i32;
pub const SEC_E_CANNOT_INSTALL: SECURITY_STATUS = 0x8009_0307u32 as i32;
pub const SEC_E_INVALID_TOKEN: SECURITY_STATUS = 0x8009_0308u32 as i32;
pub const SEC_E_CANNOT_PACK: SECURITY_STATUS = 0x8009_0309u32 as i32;
pub const SEC_E_QOP_NOT_SUPPORTED: SECURITY_STATUS = 0x8009_030Au32 as i32;
pub const SEC_E_NO_IMPERSONATION: SECURITY_STATUS = 0x8009_030Bu32 as i32;
pub const SEC_E_LOGON_DENIED: SECURITY_STATUS = 0x8009_030Cu32 as i32;
pub const SEC_E_UNKNOWN_CREDENTIALS: SECURITY_STATUS = 0x8009_030Du32 as i32;
pub const SEC_E_NO_CREDENTIALS: SECURITY_STATUS = 0x8009_030Eu32 as i32;
pub const SEC_E_MESSAGE_ALTERED: SECURITY_STATUS = 0x8009_030Fu32 as i32;
pub const SEC_E_OUT_OF_SEQUENCE: SECURITY_STATUS = 0x8009_0310u32 as i32;
pub const SEC_E_NO_AUTHENTICATING_AUTHORITY: SECURITY_STATUS = 0x8009_0311u32 as i32;
pub const SEC_E_BUFFER_TOO_SMALL: SECURITY_STATUS = 0x8009_0321u32 as i32;
pub const SEC_E_WRONG_PRINCIPAL: SECURITY_STATUS = 0x8009_0322u32 as i32;
pub const SEC_E_UNSUPPORTED_PREAUTH: SECURITY_STATUS = 0x8009_0343u32 as i32;
pub const SEC_E_NO_CONTEXT: SECURITY_STATUS = 0x8009_0361u32 as i32;
pub const SEC_I_CONTINUE_NEEDED: SECURITY_STATUS = 0x0009_0312;
pub const SEC_I_COMPLETE_NEEDED: SECURITY_STATUS = 0x0009_0313;
pub const SEC_I_COMPLETE_AND_CONTINUE: SECURITY_STATUS = 0x0009_0314;
pub const SEC_I_LOCAL_LOGON: SECURITY_STATUS = 0x0009_0315;
pub const SEC_I_CONTEXT_EXPIRED: SECURITY_STATUS = 0x0009_0317;
pub const SEC_I_INCOMPLETE_CREDENTIALS: SECURITY_STATUS = 0x0009_0320;
pub const SEC_I_RENEGOTIATE: SECURITY_STATUS = 0x0009_0321;
pub const SEC_I_NO_LSA_CONTEXT: SECURITY_STATUS = 0x0009_0323;

// ---- NT helpers the translations expect (mirror advapi32-rust prelude) ----
#[inline]
pub fn NT_SUCCESS(s: NTSTATUS) -> bool {
    s >= 0
}

#[inline]
pub unsafe fn RtlGetProcessHeap() -> PVOID {
    // PEB->ProcessHeap. ntdll exports RtlGetCurrentPeb / the heap via NtCurrentPeb.
    (*NtCurrentPeb()).ProcessHeap
}

// RtlZeroMemory/RtlCopyMemory/RtlFillMemory — ReactOS macros over memset/memcpy.
#[inline]
pub unsafe fn RtlZeroMemory(dst: PVOID, len: SIZE_T) {
    crate::externs::memset(dst as *mut _, 0, len as usize);
}
#[inline]
pub unsafe fn RtlCopyMemory(dst: PVOID, src: *const core::ffi::c_void, len: SIZE_T) {
    crate::externs::memcpy(dst as *mut _, src, len as usize);
}
#[inline]
pub unsafe fn RtlFillMemory(dst: PVOID, len: SIZE_T, fill: u8) {
    crate::externs::memset(dst as *mut _, fill as i32, len as usize);
}

#[inline]
pub fn min<T: PartialOrd>(a: T, b: T) -> T {
    if a < b { a } else { b }
}
#[inline]
pub fn max<T: PartialOrd>(a: T, b: T) -> T {
    if a > b { a } else { b }
}

// Re-export the cross-file/sibling-DLL extern decls so body modules see them via
// `use crate::prelude::*;` (kernel32 Heap*/MultiByteToWideChar/CRT memcpy/…).
pub use crate::externs::*;

// Re-export the secur32-private C structs/enums/consts (SecureProvider/NegoHelper/
// arc4_info/MD5_CTX/NTLMSSP_* …) so body modules see them via the prelude glob.
pub use crate::types::*;

// ---- core::ffi scalar aliases the translations emit bare (c_char/c_int/…) -----
pub use core::ffi::{c_char, c_schar, c_uchar, c_short, c_ushort, c_int, c_uint, c_long, c_ulong};

// ---- SSPI request/return/buffer/attribute flag consts (from secur32-sys) ------
pub use secur32_sys::{
    ASC_REQ_ALLOCATE_MEMORY, ASC_REQ_ALLOW_CONTEXT_REPLAY, ASC_REQ_ALLOW_NON_USER_LOGONS,
    ASC_REQ_ALLOW_NULL_SESSION, ASC_REQ_CALL_LEVEL, ASC_REQ_CONFIDENTIALITY, ASC_REQ_CONNECTION,
    ASC_REQ_DATAGRAM, ASC_REQ_DELEGATE, ASC_REQ_EXTENDED_ERROR, ASC_REQ_FRAGMENT_NO_TOKEN,
    ASC_REQ_FRAGMENT_SUPPLIED, ASC_REQ_FRAGMENT_TO_FIT, ASC_REQ_IDENTIFY, ASC_REQ_INTEGRITY,
    ASC_REQ_LICENSING, ASC_REQ_MUTUAL_AUTH, ASC_REQ_REPLAY_DETECT, ASC_REQ_SEQUENCE_DETECT,
    ASC_REQ_STREAM, ASC_REQ_USE_DCE_STYLE, ASC_REQ_USE_SESSION_KEY, ASC_RET_ALLOCATED_MEMORY,
    ASC_RET_ALLOW_CONTEXT_REPLAY, ASC_RET_ALLOW_NON_USER_LOGONS, ASC_RET_CALL_LEVEL,
    ASC_RET_CONFIDENTIALITY, ASC_RET_CONNECTION, ASC_RET_DATAGRAM, ASC_RET_DELEGATE,
    ASC_RET_EXTENDED_ERROR, ASC_RET_FRAGMENT_ONLY, ASC_RET_IDENTIFY, ASC_RET_INTEGRITY,
    ASC_RET_LICENSING, ASC_RET_MUTUAL_AUTH, ASC_RET_NO_TOKEN, ASC_RET_NULL_SESSION,
    ASC_RET_REPLAY_DETECT, ASC_RET_SEQUENCE_DETECT, ASC_RET_STREAM, ASC_RET_THIRD_LEG_FAILED,
    ASC_RET_USED_DCE_STYLE, ASC_RET_USE_SESSION_KEY,
    ISC_REQ_ALLOCATE_MEMORY, ISC_REQ_CALL_LEVEL, ISC_REQ_CONFIDENTIALITY, ISC_REQ_CONNECTION,
    ISC_REQ_DATAGRAM, ISC_REQ_DELEGATE, ISC_REQ_EXTENDED_ERROR, ISC_REQ_FRAGMENT_SUPPLIED,
    ISC_REQ_FRAGMENT_TO_FIT, ISC_REQ_IDENTIFY, ISC_REQ_INTEGRITY, ISC_REQ_MANUAL_CRED_VALIDATION,
    ISC_REQ_MUTUAL_AUTH, ISC_REQ_NULL_SESSION, ISC_REQ_PROMPT_FOR_CREDS, ISC_REQ_REPLAY_DETECT,
    ISC_REQ_RESERVED1, ISC_REQ_SEQUENCE_DETECT, ISC_REQ_STREAM, ISC_REQ_USE_DCE_STYLE,
    ISC_REQ_USE_SESSION_KEY, ISC_REQ_USE_SUPPLIED_CREDS, ISC_RET_ALLOCATED_MEMORY,
    ISC_RET_CALL_LEVEL, ISC_RET_CONFIDENTIALITY, ISC_RET_CONNECTION, ISC_RET_DATAGRAM,
    ISC_RET_DELEGATE, ISC_RET_EXTENDED_ERROR, ISC_RET_FRAGMENT_ONLY, ISC_RET_IDENTIFY,
    ISC_RET_INTEGRITY, ISC_RET_INTERMEDIATE_RETURN, ISC_RET_MANUAL_CRED_VALIDATION,
    ISC_RET_MUTUAL_AUTH, ISC_RET_NULL_SESSION, ISC_RET_REPLAY_DETECT, ISC_RET_RESERVED1,
    ISC_RET_SEQUENCE_DETECT, ISC_RET_STREAM, ISC_RET_USED_COLLECTED_CREDS, ISC_RET_USED_DCE_STYLE,
    ISC_RET_USED_SUPPLIED_CREDS, ISC_RET_USE_SESSION_KEY,
    SECBUFFER_ATTRMASK, SECBUFFER_CHANNEL_BINDINGS, SECBUFFER_DATA, SECBUFFER_EMPTY,
    SECBUFFER_EXTRA, SECBUFFER_MECHLIST, SECBUFFER_MECHLIST_SIGNATURE, SECBUFFER_MISSING,
    SECBUFFER_NEGOTIATION_INFO, SECBUFFER_PADDING, SECBUFFER_PKG_PARAMS, SECBUFFER_READONLY,
    SECBUFFER_READONLY_WITH_CHECKSUM, SECBUFFER_RESERVED, SECBUFFER_STREAM, SECBUFFER_STREAM_HEADER,
    SECBUFFER_STREAM_TRAILER, SECBUFFER_TARGET, SECBUFFER_TOKEN, SECBUFFER_VERSION,
    SECPKG_ATTR_ACCESS_TOKEN, SECPKG_ATTR_APP_DATA, SECPKG_ATTR_AUTHENTICATION_ID,
    SECPKG_ATTR_AUTHORITY, SECPKG_ATTR_CIPHER_STRENGTHS, SECPKG_ATTR_CLIENT_SPECIFIED_TARGET,
    SECPKG_ATTR_CONNECTION_INFO, SECPKG_ATTR_CONTEXT_DELETED, SECPKG_ATTR_CREDENTIAL_NAME,
    SECPKG_ATTR_DCE_INFO, SECPKG_ATTR_EAP_KEY_BLOCK, SECPKG_ATTR_ENDPOINT_BINDINGS,
    SECPKG_ATTR_FLAGS, SECPKG_ATTR_ISSUER_LIST, SECPKG_ATTR_ISSUER_LIST_EX, SECPKG_ATTR_KEY_INFO,
    SECPKG_ATTR_LAST_CLIENT_TOKEN_STATUS, SECPKG_ATTR_LIFESPAN, SECPKG_ATTR_LOCAL_CERT_CONTEXT,
    SECPKG_ATTR_LOCAL_CRED, SECPKG_ATTR_LOGOFF_TIME, SECPKG_ATTR_MAPPED_CRED_ATTR,
    SECPKG_ATTR_NAMES, SECPKG_ATTR_NATIVE_NAMES, SECPKG_ATTR_NEGOTIATION_INFO,
    SECPKG_ATTR_NEGO_INFO_FLAG_NO_KERBEROS, SECPKG_ATTR_NEGO_INFO_FLAG_NO_NTLM,
    SECPKG_ATTR_NEGO_KEYS, SECPKG_ATTR_NEGO_PKG_INFO, SECPKG_ATTR_NEGO_STATUS,
    SECPKG_ATTR_PACKAGE_INFO, SECPKG_ATTR_PASSWORD_EXPIRY, SECPKG_ATTR_PROMPTING_NEEDED,
    SECPKG_ATTR_PROTO_INFO, SECPKG_ATTR_REMOTE_CERT_CONTEXT, SECPKG_ATTR_REMOTE_CRED,
    SECPKG_ATTR_ROOT_STORE, SECPKG_ATTR_SESSION_INFO, SECPKG_ATTR_SESSION_KEY, SECPKG_ATTR_SIZES,
    SECPKG_ATTR_STREAM_SIZES, SECPKG_ATTR_SUBJECT_SECURITY_ATTRIBUTES, SECPKG_ATTR_SUPPORTED_ALGS,
    SECPKG_ATTR_SUPPORTED_PROTOCOLS, SECPKG_ATTR_TARGET, SECPKG_ATTR_TARGET_INFORMATION,
    SECPKG_ATTR_UNIQUE_BINDINGS, SECPKG_ATTR_USER_FLAGS, SECPKG_ATTR_USE_VALIDATED,
    SECURITY_ENTRYPOINT_ANSIA, SECURITY_ENTRYPOINT_ANSIW, SECURITY_NATIVE_DREP,
    SECURITY_NETWORK_DREP, SECURITY_SUPPORT_PROVIDER_INTERFACE_VERSION, NERR_Success,
    CRED_TYPE_DOMAIN_PASSWORD, SECPKG_CRED_ATTR_NAMES,
    SECPKG_CRED_INBOUND, SECPKG_CRED_OUTBOUND, SECPKG_CRED_BOTH, SECPKG_CRED_DEFAULT,
};

// ---- SSPI context/credential attribute structs + pointers (from secur32-sys) --
pub use secur32_sys::{
    LSA_HANDLE, LSA_OBJECT_ATTRIBUTES, PLSA_HANDLE, PLSA_OBJECT_ATTRIBUTES,
    POLICY_DNS_DOMAIN_INFO, PPOLICY_DNS_DOMAIN_INFO,
    PSecPkgContext_AccessToken, PSecPkgContext_AuthorityA, PSecPkgContext_AuthorityW,
    PSecPkgContext_AuthzID, PSecPkgContext_Bindings, PSecPkgContext_ConnectionInfo,
    PSecPkgContext_CredentialNameA, PSecPkgContext_CredentialNameW, PSecPkgContext_DceInfo,
    PSecPkgContext_Flags, PSecPkgContext_IssuerListInfoEx, PSecPkgContext_KeyInfoA,
    PSecPkgContext_KeyInfoW, PSecPkgContext_Lifespan, PSecPkgContext_NamesA,
    PSecPkgContext_NamesW, PSecPkgContext_NativeNamesA, PSecPkgContext_NativeNamesW,
    PSecPkgContext_NegotiationInfoA, PSecPkgContext_NegotiationInfoW, PSecPkgContext_PackageInfoA,
    PSecPkgContext_PackageInfoW, PSecPkgContext_PasswordExpiry, PSecPkgContext_ProtoInfoA,
    PSecPkgContext_ProtoInfoW, PSecPkgContext_SessionKey, PSecPkgContext_Sizes,
    PSecPkgContext_StreamSizes, PSecPkgContext_Target, PSecPkgContext_TargetInformation,
    PSecPkgContext_UserFlags, PSecPkgCredentials_NamesA, PSecPkgCredentials_NamesW,
    SecPkgContext_AccessToken, SecPkgContext_AuthzID, SecPkgContext_Bindings,
    SecPkgContext_ConnectionInfo, SecPkgContext_CredentialNameA, SecPkgContext_CredentialNameW,
    SecPkgContext_DceInfo, SecPkgContext_IssuerListInfoEx, SecPkgContext_NativeNamesA,
    SecPkgContext_NativeNamesW, SecPkgContext_ProtoInfoA, SecPkgContext_ProtoInfoW,
    SecPkgContext_Target, SecPkgContext_TargetInformation, SecPkgContext_UserFlags,
    SecPkgCredentials_NamesA, SecPkgCredentials_NamesW,
};

// PolicyDnsDomainInformation: bindgen constifies the _POLICY_INFORMATION_CLASS enum
// with a type-prefixed name; the translation uses the bare enumerator name.
pub use secur32_sys::_POLICY_INFORMATION_CLASS_PolicyDnsDomainInformation as PolicyDnsDomainInformation;

// ---- consts NOT in bindgen (verified vs SDK headers) ----------------------
// winerror.h SEC_E_INCOMPLETE_MESSAGE = _HRESULT_TYPEDEF_(0x80090318)
pub const SEC_E_INCOMPLETE_MESSAGE: SECURITY_STATUS = 0x8009_0318u32 as i32;
// rpcdce.h: #define RPC_C_AUTHN_WINNT 10
pub const RPC_C_AUTHN_WINNT: ULONG = 10;
// winnls.h: #define WC_NO_BEST_FIT_CHARS 0x0400
pub const WC_NO_BEST_FIT_CHARS: DWORD = 0x0400;
// wine/ntlm.h: #define CP_UNIXCP CP_ACP  (== 0)
pub const CP_UNIXCP: UINT = 0;
// ntsecapi.h: #define POLICY_VIEW_LOCAL_INFORMATION 1
pub const POLICY_VIEW_LOCAL_INFORMATION: ULONG = 1;

// secext.h EXTENDED_NAME_FORMAT enum (not bindgen'd into secur32-sys).
pub type EXTENDED_NAME_FORMAT = c_int;
pub const NameUnknown: EXTENDED_NAME_FORMAT = 0;
pub const NameFullyQualifiedDN: EXTENDED_NAME_FORMAT = 1;
pub const NameSamCompatible: EXTENDED_NAME_FORMAT = 2;
pub const NameDisplay: EXTENDED_NAME_FORMAT = 3;
pub const NameUniqueId: EXTENDED_NAME_FORMAT = 6;
pub const NameCanonical: EXTENDED_NAME_FORMAT = 7;
pub const NameUserPrincipal: EXTENDED_NAME_FORMAT = 8;
pub const NameCanonicalEx: EXTENDED_NAME_FORMAT = 9;
pub const NameServicePrincipal: EXTENDED_NAME_FORMAT = 10;
pub const NameDnsDomain: EXTENDED_NAME_FORMAT = 12;
// bindgen-constified spellings the translation also emits (match patterns).
pub const _EXTENDED_NAME_FORMAT_NameUnknown: EXTENDED_NAME_FORMAT = 0;
pub const _EXTENDED_NAME_FORMAT_NameFullyQualifiedDN: EXTENDED_NAME_FORMAT = 1;
pub const _EXTENDED_NAME_FORMAT_NameSamCompatible: EXTENDED_NAME_FORMAT = 2;
pub const _EXTENDED_NAME_FORMAT_NameDisplay: EXTENDED_NAME_FORMAT = 3;
pub const _EXTENDED_NAME_FORMAT_NameUniqueId: EXTENDED_NAME_FORMAT = 6;
pub const _EXTENDED_NAME_FORMAT_NameCanonical: EXTENDED_NAME_FORMAT = 7;
pub const _EXTENDED_NAME_FORMAT_NameUserPrincipal: EXTENDED_NAME_FORMAT = 8;
pub const _EXTENDED_NAME_FORMAT_NameCanonicalEx: EXTENDED_NAME_FORMAT = 9;
pub const _EXTENDED_NAME_FORMAT_NameServicePrincipal: EXTENDED_NAME_FORMAT = 10;
pub const _EXTENDED_NAME_FORMAT_NameDnsDomain: EXTENDED_NAME_FORMAT = 12;

// lmaccess/lmcons NET_API_STATUS = DWORD
pub type NET_API_STATUS = DWORD;

// ---- wine/unicode.h inline WCHAR string helpers (translation calls them bare) -
#[inline]
pub unsafe fn strlenW(str_: PCWSTR) -> UINT {
    let mut s = str_;
    while *s != 0 { s = s.add(1); }
    s.offset_from(str_) as UINT
}
// wine strchrW: matches `ch` including the NUL terminator, returns NULL otherwise.
#[inline]
pub unsafe fn strchrW(str_: PCWSTR, ch: i32) -> PWSTR {
    let mut s = str_;
    loop {
        if *s as i32 == ch { return s as PWSTR; }
        if *s == 0 { return core::ptr::null_mut(); }
        s = s.add(1);
    }
}

// ---- cross-file symbol re-exports (definitions live in sibling body modules) --
// FreeContextBuffer is a secur32 export defined in sspi.rs but referenced by name
// in the kerberos/negotiate SSP function tables.
pub use crate::sspi::{
    FreeContextBuffer, SECUR32_AllocMultiByteFromWide, SECUR32_AllocWideFromMultiByte,
    SECUR32_findPackageA, SECUR32_findPackageW, SECUR32_addProvider, SECUR32_addPackages,
    SECUR32_freeProviders,
};
pub use crate::util::{
    ComputeCrc32, SECUR32_CreateNTLM1SessionKey, SECUR32_CreateNTLM2SubKeys,
    SECUR32_arc4Alloc, SECUR32_arc4Cleanup, SECUR32_arc4Init, SECUR32_arc4Process,
};
pub use crate::hmac_md5::{HMACMD5Final, HMACMD5Init, HMACMD5Update};
// wrapper.rs SSPI dispatchers referenced by name from the thunks.
pub use crate::wrapper::{
    AcquireCredentialsHandleA, AcquireCredentialsHandleW,
    ImportSecurityContextA, ImportSecurityContextW,
};
pub use crate::base64_codec::{encodeBase64, decodeBase64};
pub use crate::dispatcher::run_helper;
pub use crate::negotiate::{negoTableA, negoTableW};
pub use crate::ntlm::{
    ntlm_package_infoA, ntlm_package_infoW,
    ntlm_AcceptSecurityContext, ntlm_AcquireCredentialsHandleW, ntlm_DecryptMessage,
    ntlm_DeleteSecurityContext, ntlm_EncryptMessage, ntlm_FreeCredentialsHandle,
    ntlm_InitializeSecurityContextW, ntlm_MakeSignature, ntlm_QueryContextAttributesA,
    ntlm_QueryContextAttributesW, ntlm_VerifySignature,
};
pub use crate::thunks::{
    thunk_AcquireCredentialsHandleA, thunk_AcquireCredentialsHandleW,
    thunk_AddCredentialsA, thunk_AddCredentialsW,
    thunk_ImportSecurityContextA, thunk_ImportSecurityContextW,
    thunk_InitializeSecurityContextA, thunk_InitializeSecurityContextW,
    thunk_QueryContextAttributesA, thunk_QueryContextAttributesW,
    thunk_QueryCredentialsAttributesA, thunk_QueryCredentialsAttributesW,
    thunk_SetContextAttributesA, thunk_SetContextAttributesW,
};
