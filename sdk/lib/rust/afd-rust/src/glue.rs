//! Harvested afd #define consts / type aliases / macro shims the per-arch bindgen
//! substrate (`types`) doesn't emit. Values match drivers/network/afd/include/
//! {afd,debug}.h. The struct + most flag consts (AFD_FCB, FUNCTION_*, SOCKET_STATE_*,
//! AFD_EVENT_*, P-prefixed pointer aliases) ARE in `types`; only the items below are
//! genuinely absent (CCHAR/UINT base aliases, the multichar TAG_AFD_* pool tags that
//! bindgen skips, the debug trace levels, and the release-noop ASSERT/MIN/MAX macros
//! the C source uses as call expressions).

use ntkrnl_sys::ddk::*;
use ntkrnl_sys::driver_ddk::{MmGetSystemAddressForMdlSafe, NormalPagePriority};

// --- kernel DATA exports (POBJECT_TYPE globals) -----------------------------
// ntoskrnl exports these as data, not functions. A plain `extern` references the
// bare symbol (`_ExEventObjectType`) which the import lib doesn't provide — only
// `__imp__ExEventObjectType`. raw-dylib routes the reference through the import
// thunk so it resolves. CRITICAL: each of these is a single POBJECT_TYPE
// (`*mut OBJECT_TYPE`), NOT a double pointer. The C source writes `*IoFileObjectType`
// only because ReactOS's __CREATE_NTOS_DATA_IMPORT_ALIAS makes the bare C symbol the
// IAT slot itself (a POBJECT_TYPE*); raw-dylib already reads THROUGH that thunk, so
// in Rust the static's value IS the POBJECT_TYPE and call-sites must NOT deref it.
// (Matches cdfs-rust/fastfat-rust fsctrl.rs, which pass `IoFileObjectType` directly.)
#[link(name = "ntoskrnl.exe", kind = "raw-dylib", modifiers = "+verbatim")]
extern "C" {
    pub static ExEventObjectType: *mut core::ffi::c_void;
    pub static IoFileObjectType: *mut core::ffi::c_void;
}

// --- base type aliases not in the kernel bindgen ---------------------------
pub type CCHAR = i8;
pub type UINT = u32;
pub type INT = i32;
pub type DWORD = u32;

// bindgen raw struct names the translation uses (ntkrnl-sys re-exports only the
// non-underscore aliases). Map them back onto the single shared ddk identity.
pub type _IO_STACK_LOCATION = IO_STACK_LOCATION;
pub type _IO_STATUS_BLOCK = IO_STATUS_BLOCK;
pub type _IRP = IRP;
pub type _MDL = MDL;
pub type _LIST_ENTRY = LIST_ENTRY;
pub type _UNICODE_STRING = UNICODE_STRING;
pub type _FILE_OBJECT = FILE_OBJECT;

// --- TDI flags (tdi.h) — not in the kernel bindgen -------------------------
pub const TDI_RECEIVE_NORMAL: u32 = 0x00000020;
pub const TDI_RECEIVE_PEEK: u32 = 0x00000080;
pub const TDI_DISCONNECT_ABORT: u32 = 0x0002;
pub const TDI_DISCONNECT_RELEASE: u32 = 0x0004;
pub const TDI_QUERY_ADDRESS_INFO: u32 = 0x00000003;

// --- winsock FD_* event bit indices (winsock2.h) ---------------------------
pub const FD_READ_BIT: u32 = 0;
pub const FD_WRITE_BIT: u32 = 1;
pub const FD_OOB_BIT: u32 = 2;
pub const FD_ACCEPT_BIT: u32 = 3;
pub const FD_CONNECT_BIT: u32 = 4;
pub const FD_CLOSE_BIT: u32 = 5;

// --- NTSTATUS / access masks not exported by ntkrnl-sys --------------------
pub const STATUS_NO_SUCH_DEVICE: i32 = 0xC000000Eu32 as i32;
pub const STATUS_CANT_WAIT: i32 = 0xC00000D8u32 as i32;
pub const STATUS_INVALID_CONNECTION: i32 = 0xC0000140u32 as i32;
pub const EVENT_ALL_ACCESS: u32 = 0x1F0003;

// --- misc kernel consts only in ntkrnl-sys's hal bindgen, not ddk ----------
pub const FO_NAMED_PIPE: u32 = 0x00000080;
pub const FILE_DEVICE_NAMED_PIPE: u32 = 0x00000011;
// IRP_MJ_QUERY_VOLUME_INFORMATION was duplicated here. It now lives in the authoritative
// bindgen island (ntkrnl_sys::driver_ddk, added 2026-07-25 by 6ae2ae1931d), and main.rs
// glob-imports BOTH that and crate::glue — two different items under one name is
// `error[E0659]: ambiguous`, which broke this crate on ALL THREE arches on mainline. Same
// resolution as fltmgr-rust: the island wins, the hand-written copy goes. Sibling crates
// that keep their own copy (msfs/mup/npfs) do NOT glob driver_ddk, so they are unaffected —
// and their use sites index MajorFunction without a cast, so they need the `usize` spelling.

// --- MDL macros the C source uses (mm.h FORCEINLINE) -----------------------
#[inline(always)]
pub unsafe fn MmGetMdlByteCount(mdl: *mut MDL) -> u32 {
    (*mdl).ByteCount
}
// Non-"Safe" MdL system-VA macro: ntkrnl-sys only exposes the Safe variant, so
// map onto it with NormalPagePriority (afd always probes+locks first, so the VA
// is resident — the priority/NULL-return distinction is moot here).
#[inline(always)]
pub unsafe fn MmGetSystemAddressForMdl(mdl: *mut MDL) -> *mut core::ffi::c_void {
    MmGetSystemAddressForMdlSafe(mdl, NormalPagePriority as u32)
}

// --- TDI IP transport address (tdi.h) — blocklisted out of the substrate ----
// Layout is load-bearing: SatisfyPreAccept does ptr arithmetic on &Address[1].
#[repr(C)]
#[derive(Copy, Clone)]
pub struct TDI_ADDRESS_IP {
    pub sin_port: u16,
    pub in_addr: u32,
    pub sin_zero: [u8; 8],
}
#[repr(C)]
#[derive(Copy, Clone)]
pub struct TA_IP_ADDRESS_AddrIp {
    pub AddressLength: u16,
    pub AddressType: u16,
    pub Address: [TDI_ADDRESS_IP; 1],
}
#[repr(C)]
#[derive(Copy, Clone)]
pub struct TA_IP_ADDRESS {
    pub TAAddressCount: i32,
    pub Address: [TA_IP_ADDRESS_AddrIp; 1],
}
pub type PTA_IP_ADDRESS = *mut TA_IP_ADDRESS;

// UNREFERENCED_PARAMETER(x) and NT_VERIFY(x) are macros the C source uses as call
// expressions. UNREFERENCED_PARAMETER is a no-op; NT_VERIFY evaluates+returns its
// arg (the debug assertion is dropped under release). Generic so the per-call arg
// types (PDEVICE_OBJECT / PVOID / u8 / ...) all resolve.
#[inline(always)]
pub fn UNREFERENCED_PARAMETER<T>(_x: T) {}
#[inline(always)]
pub fn NT_VERIFY<T>(expr: T) -> T {
    expr
}

// AFD_DbgPrint is a debug-only macro (empty under release/NDEBUG, which is the
// ReactOS build config). Make it a no-op macro — faithful to the C #define and
// avoids any link symbol. #[macro_use] on `mod glue` puts it in scope crate-wide.
#[macro_export]
macro_rules! AFD_DbgPrint {
    ($($arg:tt)*) => {{}};
}

// --- debug trace levels (debug.h) ------------------------------------------
pub const MIN_TRACE: u32 = 0x00000001;
pub const MID_TRACE: u32 = 0x00000002;
pub const MAX_TRACE: u32 = 0x00000003;

// --- NTSTATUS codes the modules reference but ntkrnl-sys doesn't export -----
pub const STATUS_FILE_CLOSED: i32 = 0xC0000128u32 as i32;

// --- pool tags (afd.h) — C multichar 'BdfA' = (B<<24)|(d<<16)|(f<<8)|A. bindgen
//     drops implementation-defined multichar #defines, so re-emit them here. ----
const fn fourcc(s: &[u8; 4]) -> u32 {
    ((s[0] as u32) << 24) | ((s[1] as u32) << 16) | ((s[2] as u32) << 8) | (s[3] as u32)
}
pub const TAG_AFD_DATA_BUFFER: u32 = fourcc(b"BdfA");
pub const TAG_AFD_TRANSPORT_ADDRESS: u32 = fourcc(b"tdfA");
pub const TAG_AFD_SOCKET_CONTEXT: u32 = fourcc(b"XdfA");
pub const TAG_AFD_CONNECT_DATA: u32 = fourcc(b"cdfA");
pub const TAG_AFD_DISCONNECT_DATA: u32 = fourcc(b"ddfA");
pub const TAG_AFD_CONNECT_OPTIONS: u32 = fourcc(b"ocfA");
pub const TAG_AFD_DISCONNECT_OPTIONS: u32 = fourcc(b"odfA");
pub const TAG_AFD_SUPER_CONNECT_BUFFER: u32 = fourcc(b"sdfA");
pub const TAG_AFD_ACCEPT_QUEUE: u32 = fourcc(b"qafA");
pub const TAG_AFD_POLL_HANDLE: u32 = fourcc(b"hpfA");
pub const TAG_AFD_FCB: u32 = fourcc(b"cffA");
pub const TAG_AFD_ACTIVE_POLL: u32 = fourcc(b"pafA");
pub const TAG_AFD_EA_INFO: u32 = fourcc(b"aefA");
pub const TAG_AFD_STORED_DATAGRAM: u32 = fourcc(b"gsfA");
pub const TAG_AFD_SNMP_ADDRESS_INFO: u32 = fourcc(b"asfA");
pub const TAG_AFD_TDI_CONNECTION_INFORMATION: u32 = fourcc(b"cTfA");
pub const TAG_AFD_WSA_BUFFER: u32 = fourcc(b"bWfA");

// --- release-noop / value macros used as call expressions in the C source ---
// (ASSERT is the crate macro in lib.rs, debug_assert!-backed.)
// MIN/MAX(a,b) — afd uses the min/max value macros.
#[inline(always)]
pub fn MIN<T: PartialOrd>(a: T, b: T) -> T {
    if a < b {
        a
    } else {
        b
    }
}
#[inline(always)]
pub fn MAX<T: PartialOrd>(a: T, b: T) -> T {
    if a > b {
        a
    } else {
        b
    }
}
