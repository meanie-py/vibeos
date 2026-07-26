//! rasadhlp-rust prelude — the single import surface the translated modules use.
//! Bridges the bindgen `types` island and the translated C. Anything bindgen leaves out
//! (CRT, inlines, sibling-C externs, hand-transcribed consts) is declared here ONCE.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

pub use crate::types::*;
pub use core::ffi::{c_char, c_int, c_long, c_short, c_uchar, c_uint, c_ushort, c_void};
pub use core::ptr::{addr_of, addr_of_mut};

// ================= debug ======================================================================
// reactos/debug.h DPRINT/DPRINT1 — no-op MACROS, never out-of-line fns
// (field manual §9.65: an out-of-line no-op is an optimization barrier).
#[macro_export] macro_rules! DPRINT  { ($($arg:tt)*) => {{}}; }
#[macro_export] macro_rules! DPRINT1 { ($($arg:tt)*) => {{}}; }
pub use crate::{DPRINT, DPRINT1};

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

// ============ wide literals ==================================================
// Same shape as netid-rust's: ASCII source literal -> NUL-terminated UTF-16, with
// the ASCII range asserted at COMPILE time so a non-ASCII literal cannot silently
// truncate.
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
        &OUT
    }};
}

// ntdef.h: `#define RTL_CONSTANT_STRING(s) { sizeof(s)-sizeof((s)[0]), sizeof(s), s }`
// — Length EXCLUDES the NUL, MaximumLength includes it, both in BYTES.
//
// A macro, never an out-of-line fn (AGENTS.md): the C is a compile-time initializer.
// The argument is an ASCII literal that `w!` widens; the C spells it `L"..."` and a
// UNICODE_STRING over ANSI bytes would compile fine and then fail to open the device
// at runtime, which is the whole reason this is not a byte-string passthrough.
#[macro_export]
macro_rules! RTL_CONSTANT_STRING {
    ($s:expr) => {{
        const W: &[u16] = $crate::w!($s);
        UNICODE_STRING {
            Length: ((W.len() - 1) * 2) as USHORT,
            MaximumLength: (W.len() * 2) as USHORT,
            Buffer: W.as_ptr() as *mut u16,
        }
    }};
}

// RtlCopyMemory/RtlZeroMemory are winnt.h MACROS over memcpy/memset. Out-of-line
// no-op-able fns are the §9.65 trap; keep them macros. Trailing comma tolerated
// because the C is often line-wrapped.
#[macro_export]
macro_rules! RtlCopyMemory {
    ($dst:expr, $src:expr, $len:expr $(,)?) => {
        core::ptr::copy_nonoverlapping($src as *const u8, $dst as *mut u8, $len as usize)
    };
}
#[macro_export]
macro_rules! RtlZeroMemory {
    ($dst:expr, $len:expr $(,)?) => {
        core::ptr::write_bytes($dst as *mut u8, 0u8, $len as usize)
    };
}
pub use crate::{w, RtlCopyMemory, RtlZeroMemory, RTL_CONSTANT_STRING};

// ============ enum members ===================================================
// bindgen prefixes an enum's members with its tag, so the C's bare `AutoDialIp`
// does not resolve. These feed `.Family` assignments and comparisons; a bare
// unresolved name in a `match` PATTERN would instead become a silent catch-all
// (field manual §9.112), so they are aliased rather than re-declared as literals.
pub const AutoDialIp: AUTODIAL_FAMILY = AUTODIAL_FAMILY_AutoDialIp;
pub const AutoDialIpx: AUTODIAL_FAMILY = AUTODIAL_FAMILY_AutoDialIpx;
pub const AutoDialNetBios: AUTODIAL_FAMILY = AUTODIAL_FAMILY_AutoDialNetBios;
pub const AutoDialIpHost: AUTODIAL_FAMILY = AUTODIAL_FAMILY_AutoDialIpHost;
pub const ConnectionIpxLana: CONNECTION_FAMILY = CONNECTION_FAMILY_ConnectionIpxLana;
pub const ConnectionIp: CONNECTION_FAMILY = CONNECTION_FAMILY_ConnectionIp;
pub const ConnectionIpHost: CONNECTION_FAMILY = CONNECTION_FAMILY_ConnectionIpHost;
pub const ConnectionNetBiosMac: CONNECTION_FAMILY = CONNECTION_FAMILY_ConnectionNetBiosMac;

// ============ the ACD device ioctls ==========================================
// precomp.h:20-35 builds these with CTL_CODE(), a macro EXPRESSION, so bindgen does
// not fold them and the island carries only FILE_DEVICE_ACD. DERIVED here from the
// same three inputs (all in the island / -sys glob) rather than pasted as a magic
// number, so the value stays checkable against the driver.
pub const fn CTL_CODE(DeviceType: u32, Function: u32, Method: u32, Access: u32) -> u32 {
    (DeviceType << 16) | (Access << 14) | (Function << 2) | Method
}
pub const IOCTL_ACD_CONNECT_ADDRESS: u32 =
    CTL_CODE(FILE_DEVICE_ACD, 5, METHOD_BUFFERED, FILE_READ_ACCESS);

// ============ Win32 imports ==================================================
extern "system" {
    pub fn CloseHandle(hObject: HANDLE) -> BOOL;
    pub fn WaitForSingleObject(hHandle: HANDLE, dwMilliseconds: DWORD) -> DWORD;
    pub fn CreateEventW(
        lpEventAttributes: *mut SECURITY_ATTRIBUTES,
        bManualReset: BOOL,
        bInitialState: BOOL,
        lpName: LPCWSTR,
    ) -> HANDLE;
    pub fn DisableThreadLibraryCalls(hLibModule: HMODULE) -> BOOL;
    pub fn WideCharToMultiByte(
        CodePage: UINT,
        dwFlags: DWORD,
        lpWideCharStr: LPCWSTR,
        cchWideChar: c_int,
        lpMultiByteStr: LPSTR,
        cbMultiByte: c_int,
        lpDefaultChar: *const c_char,
        lpUsedDefaultChar: *mut BOOL,
    ) -> c_int;
    // ws2_32; rasadhlp already imports it (add_importlibs ... ws2_32).
    pub fn inet_addr(cp: *const c_char) -> ULONG;
}

extern "C" {
    pub fn strlen(s: *const c_char) -> usize;
    pub fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
}

// The C calls `CreateEvent`, which is winbase.h's UNICODE alias.
pub use CreateEventW as CreateEvent;
// msvcrt: in-place ASCII lowercase. Imported, never hand-rolled.
extern "C" {
    pub fn _strlwr(s: *mut c_char) -> *mut c_char;
}

// ============ nsp_dns.h service GUIDs ========================================
// The header DEFINES these (`const GUID DECLSPEC_SELECTANY X = SVCID_...;`), it does
// not merely declare them — so they came from whichever TU included it, and winsock.c
// was that TU. Now that it is Rust, the crate owns the definitions; BLOCKLIST_VARS=1
// keeps the island from re-declaring them `extern` (which linked to nothing).
//
// Built from svcguid.h's own macros rather than pasted as opaque bytes, so the values
// stay checkable against the header.
const fn SVCID_TCP_RR(port: u32, rr: u16) -> GUID {
    GUID {
        Data1: (0x0009 << 16) | port,
        Data2: 0,
        Data3: rr,
        Data4: [0xC0, 0, 0, 0, 0, 0, 0, 0x46],
    }
}
const fn svcid_inet(low: u32) -> GUID {
    GUID { Data1: low, Data2: 0, Data3: 0, Data4: [0xC0, 0, 0, 0, 0, 0, 0, 0x46] }
}

// svcguid.h:8-11
#[no_mangle]
pub static HostnameGuid: GUID = svcid_inet(0x0002a800); // SVCID_HOSTNAME
#[no_mangle]
pub static AddressGuid: GUID = svcid_inet(0x0002a801); // SVCID_INET_HOSTADDRBYINETSTRING
#[no_mangle]
pub static IANAGuid: GUID = svcid_inet(0x0002a802); // SVCID_INET_SERVICEBYNAME
#[no_mangle]
pub static HostAddrByNameGuid: GUID = svcid_inet(0x0002a803); // SVCID_INET_HOSTADDRBYNAME
#[no_mangle]
pub static InetHostName: GUID = svcid_inet(0x0002a803); // ditto — the header aliases it
// SVCID_DNS_TYPE_AAAA = SVCID_DNS(0x001c) = SVCID_TCP_RR(53, 0x001c)
#[no_mangle]
pub static Ipv6Guid: GUID = SVCID_TCP_RR(53, 0x001c);
