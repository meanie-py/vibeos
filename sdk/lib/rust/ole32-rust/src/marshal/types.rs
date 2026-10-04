//! ole32 Lane B — shared declarations module.
//!
//! Everything the marshal/stubmanager/rpc/ftmarshal/git translations share:
//! base Win32 types, GUIDs/IIDs/CLSIDs, HRESULT constants, the wine `struct
//! list` mirror, the compobj_private.h / dcom.h private-type mirrors, the
//! hand-declared COM interface vtbls (to be replaced by the `com-sys` bindgen
//! crate at integration — keep ALL interface decls in this one module so the
//! swap is mechanical), the RPC-runtime (rpcrt4) types, and the extern-symbol
//! import surface (kernel32 / user32 / advapi32 / rpcrt4 / Lane-A compobj.c /
//! still-C ole32 files).
//!
//! Body modules do `use super::types::*;`.
#![allow(
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals,
    dead_code,
    unused_imports,
    unused_variables,
    unused_unsafe,
    unused_mut,
    unused_assignments,
    static_mut_refs,
    clippy::all
)]

pub use core::ffi::{c_char, c_int, c_long, c_uint, c_ulong, c_void};

// ---------------------------------------------------------------------------
// Base Win32 type aliases
// ---------------------------------------------------------------------------
pub type BOOL = i32;
pub type BOOLEAN = u8;
pub type BYTE = u8;
pub type UCHAR = u8;
pub type WORD = u16;
pub type USHORT = u16;
pub type SHORT = i16;
pub type DWORD = u32;
pub type ULONG = u32;
pub type UINT = u32;
pub type LONG = i32;
pub type INT = i32;
pub type HRESULT = i32;
pub type ULONGLONG = u64;
pub type LONGLONG = i64;
pub type ULONG_PTR = usize;
pub type LONG_PTR = isize;
pub type DWORD_PTR = usize;
pub type SIZE_T = usize;
pub type WCHAR = u16;
pub type OLECHAR = u16;
pub type CHAR = i8;
pub type PVOID = *mut c_void;
pub type LPVOID = *mut c_void;
pub type LPCVOID = *const c_void;
pub type HANDLE = *mut c_void;
pub type HWND = *mut c_void;
pub type HKEY = *mut c_void;
pub type HTASK = *mut c_void;
pub type SC_HANDLE = *mut c_void;
pub type LPWSTR = *mut u16;
pub type LPCWSTR = *const u16;
pub type LPSTR = *mut i8;
pub type LPCSTR = *const i8;
pub type LPBYTE = *mut u8;
pub type WPARAM = usize;
pub type LPARAM = isize;
pub type LRESULT = isize;
pub type LSTATUS = i32;
pub type REGSAM = u32;
pub type ACCESS_MASK = u32;

// LARGE_INTEGER / ULARGE_INTEGER: the C is a union with QuadPart; a bare
// i64/u64 is ABI-identical (size 8, align 8, by-value passing identical on
// i686/x86_64/aarch64 windows). Call sites that used .u.LowPart/.HighPart or
// .QuadPart are translated to plain integer ops.
pub type LARGE_INTEGER = i64;
pub type ULARGE_INTEGER = u64;

// DCOM identifier types (dcom.idl)
pub type OXID = u64;
pub type OID = u64;
pub type MSHCTX = u32; // C enum tagMSHCTX (int); u32 is ABI-identical here
pub type MSHLFLAGS = u32; // C enum tagMSHLFLAGS
pub type CALLTYPE = u32;
pub type RPC_STATUS = i32;
pub type RPC_BINDING_HANDLE = *mut c_void;
pub type RPC_IF_HANDLE = *mut c_void;
pub type RPCOLEDATAREP = u32;

pub const TRUE: BOOL = 1;
pub const FALSE: BOOL = 0;
pub const NULL: PVOID = core::ptr::null_mut();
pub const INVALID_HANDLE_VALUE: HANDLE = -1isize as HANDLE;
pub const MAX_PATH: usize = 260;

// ---------------------------------------------------------------------------
// GUID / IID / CLSID / IPID
// ---------------------------------------------------------------------------
#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct GUID {
    pub Data1: u32,
    pub Data2: u16,
    pub Data3: u16,
    pub Data4: [u8; 8],
}
pub type IID = GUID;
pub type CLSID = GUID;
pub type IPID = GUID;
pub type UUID = GUID;
pub type REFIID = *const IID;
pub type REFGUID = *const GUID;
pub type REFCLSID = *const CLSID;
pub type REFIPID = *const IPID;

pub const fn guid(d1: u32, d2: u16, d3: u16, d4: [u8; 8]) -> GUID {
    GUID { Data1: d1, Data2: d2, Data3: d3, Data4: d4 }
}
/// DEFINE_OLEGUID(name, l, w1, w2): {l, w1, w2, C000-000000000046}
pub const fn oleguid(d1: u32, d2: u16, d3: u16) -> GUID {
    guid(d1, d2, d3, [0xC0, 0, 0, 0, 0, 0, 0, 0x46])
}

#[inline]
pub unsafe fn IsEqualGUID(a: *const GUID, b: *const GUID) -> bool {
    *a == *b
}
#[inline]
pub unsafe fn IsEqualIID(a: *const IID, b: *const IID) -> bool {
    *a == *b
}
#[inline]
pub unsafe fn IsEqualCLSID(a: *const CLSID, b: *const CLSID) -> bool {
    *a == *b
}

pub const GUID_NULL: GUID = guid(0, 0, 0, [0; 8]);
pub const IID_NULL: IID = GUID_NULL;

// Interface IDs (values verified against sdk/include/psdk/*.idl + dcom.idl).
// These are LOCAL const mirrors of the uuid-lib data symbols; only their
// values matter (IsEqualIID compares content). At integration the com-sys
// bindgen crate may replace them.
pub const IID_IUnknown: IID = oleguid(0x00000000, 0, 0);
pub const IID_IClassFactory: IID = oleguid(0x00000001, 0, 0);
pub const IID_IMarshal: IID = oleguid(0x00000003, 0, 0);
pub const IID_IExternalConnection: IID = oleguid(0x00000019, 0, 0);
pub const IID_IMultiQI: IID = oleguid(0x00000020, 0, 0);
pub const IID_IClientSecurity: IID = oleguid(0x0000013D, 0, 0);
pub const IID_IRemUnknown: IID = oleguid(0x00000131, 0, 0);
pub const IID_IGlobalInterfaceTable: IID = oleguid(0x00000146, 0, 0);
pub const IID_IStream: IID = oleguid(0x0000000C, 0, 0);
pub const IID_IPSFactoryBuffer: IID =
    guid(0xD5F569D0, 0x593B, 0x101A, [0xB5, 0x69, 0x08, 0x00, 0x2B, 0x2D, 0xBF, 0x7A]);
pub const IID_IRpcStubBuffer: IID =
    guid(0xD5F56AFC, 0x593B, 0x101A, [0xB5, 0x69, 0x08, 0x00, 0x2B, 0x2D, 0xBF, 0x7A]);
pub const IID_IRpcProxyBuffer: IID =
    guid(0xD5F56A34, 0x593B, 0x101A, [0xB5, 0x69, 0x08, 0x00, 0x2B, 0x2D, 0xBF, 0x7A]);
pub const IID_IRpcChannelBuffer: IID =
    guid(0xD5F56B60, 0x593B, 0x101A, [0xB5, 0x69, 0x08, 0x00, 0x2B, 0x2D, 0xBF, 0x7A]);
pub const IID_IServiceProvider: IID =
    guid(0x6D5140C1, 0x7436, 0x11CE, [0x80, 0x34, 0x00, 0xAA, 0x00, 0x60, 0x09, 0xFA]);
pub const IID_IChannelHook: IID =
    guid(0x1008C4A0, 0x7613, 0x11CF, [0x9A, 0xF1, 0x00, 0x20, 0xAF, 0x6E, 0x72, 0xF4]);

pub const CLSID_StdMarshal: CLSID = oleguid(0x00000017, 0, 0);
pub const CLSID_InProcFreeMarshaler: CLSID = oleguid(0x0000033A, 0, 0);
/// compobj_private.h: DEFINE_OLEGUID(CLSID_DfMarshal, 0x0000030b, 0, 0)
pub const CLSID_DfMarshal: CLSID = oleguid(0x0000030B, 0, 0);

// ---------------------------------------------------------------------------
// HRESULT / error constants (values verified against psdk/winerror.h)
// ---------------------------------------------------------------------------
pub const S_OK: HRESULT = 0;
pub const S_FALSE: HRESULT = 1;
pub const E_NOTIMPL: HRESULT = 0x80004001u32 as i32;
pub const E_NOINTERFACE: HRESULT = 0x80004002u32 as i32;
pub const E_POINTER: HRESULT = 0x80004003u32 as i32;
pub const E_FAIL: HRESULT = 0x80004005u32 as i32;
pub const E_UNEXPECTED: HRESULT = 0x8000FFFFu32 as i32;
pub const E_OUTOFMEMORY: HRESULT = 0x8007000Eu32 as i32;
pub const E_INVALIDARG: HRESULT = 0x80070057u32 as i32;
pub const CO_E_NOTINITIALIZED: HRESULT = 0x800401F0u32 as i32;
pub const CO_E_OBJNOTCONNECTED: HRESULT = 0x800401FDu32 as i32;
pub const RPC_E_CALL_REJECTED: HRESULT = 0x80010001u32 as i32;
pub const RPC_E_CANTCALLOUT_ININPUTSYNCCALL: HRESULT = 0x8001010Du32 as i32;
pub const RPC_E_WRONG_THREAD: HRESULT = 0x8001010Eu32 as i32;
pub const RPC_E_DISCONNECTED: HRESULT = 0x80010108u32 as i32;
pub const RPC_E_RETRY: HRESULT = 0x80010109u32 as i32;
pub const RPC_E_VERSION_MISMATCH: HRESULT = 0x80010110u32 as i32;
pub const RPC_E_INVALID_HEADER: HRESULT = 0x80010111u32 as i32;
pub const RPC_E_INVALID_OBJECT: HRESULT = 0x80010114u32 as i32;
pub const RPC_E_INVALID_OBJREF: HRESULT = 0x8001011Du32 as i32;
pub const STG_E_READFAULT: HRESULT = 0x8003001Eu32 as i32;
pub const STG_E_MEDIUMFULL: HRESULT = 0x80030070u32 as i32;
pub const REGDB_E_CLASSNOTREG: HRESULT = 0x80040154u32 as i32;

pub const ERROR_SUCCESS: DWORD = 0;
pub const ERROR_OUTOFMEMORY: DWORD = 14;
pub const ERROR_IO_PENDING: DWORD = 997;
pub const ERROR_PIPE_CONNECTED: DWORD = 535;
pub const ERROR_SERVICE_ALREADY_RUNNING: DWORD = 1056;
pub const ERROR_FUNCTION_FAILED: DWORD = 1627;
pub const RPC_S_OK: RPC_STATUS = 0;
pub const RPC_S_INVALID_BOUND: HRESULT = 1734; // used as an HRESULT return, like the C

#[inline]
pub fn SUCCEEDED(hr: HRESULT) -> bool {
    hr >= 0
}
#[inline]
pub fn FAILED(hr: HRESULT) -> bool {
    hr < 0
}
/// winerror.h HRESULT_FROM_WIN32
#[inline]
pub fn HRESULT_FROM_WIN32(x: u32) -> HRESULT {
    if x == 0 {
        S_OK
    } else {
        ((x & 0x0000FFFF) | (7 << 16) | 0x80000000) as i32
    }
}

// ---------------------------------------------------------------------------
// Misc Win32 / COM / RPC constants (values verified against the SDK headers)
// ---------------------------------------------------------------------------
pub const MSHLFLAGS_NORMAL: MSHLFLAGS = 0;
pub const MSHLFLAGS_TABLESTRONG: MSHLFLAGS = 1;
pub const MSHLFLAGS_TABLEWEAK: MSHLFLAGS = 2;
pub const MSHLFLAGS_NOPING: MSHLFLAGS = 4;
/// compobj_private.h private flag: "this is a rem unknown object"
pub const MSHLFLAGSP_REMUNKNOWN: MSHLFLAGS = 0x80000000;

pub const MSHCTX_LOCAL: MSHCTX = 0;
pub const MSHCTX_NOSHAREDMEM: MSHCTX = 1;
pub const MSHCTX_DIFFERENTMACHINE: MSHCTX = 2;
pub const MSHCTX_INPROC: MSHCTX = 3;
pub const MSHCTX_CROSSCTX: MSHCTX = 4;

// dcom.idl
pub const OBJREF_SIGNATURE: u32 = 0x574F454D; /* "MEOW" */
pub const OBJREF_STANDARD: u32 = 0x1;
pub const OBJREF_HANDLER: u32 = 0x2;
pub const OBJREF_CUSTOM: u32 = 0x4;
pub const SORF_NULL: u32 = 0x0;
pub const SORF_OXRES1: u32 = 0x1;
pub const SORF_OXRES2: u32 = 0x20;
pub const SORF_NOPING: u32 = 0x1000;
/// marshal.c private flag: object was marshaled table-weak
pub const SORFP_TABLEWEAK: u32 = SORF_OXRES1;
/// marshal.c private flag: no lifetime management on the proxy
pub const SORFP_NOLIFETIMEMGMT: u32 = SORF_OXRES2;

pub const COM_MAJOR_VERSION: u16 = 5;
pub const COM_MINOR_VERSION: u16 = 3;
pub const ORPCF_NULL: u32 = 0;
pub const ORPCF_LOCAL: u32 = 1;
pub const ORPCF_RESERVED1: u32 = 2;
pub const ORPCF_RESERVED2: u32 = 4;
pub const ORPCF_RESERVED3: u32 = 8;
pub const ORPCF_RESERVED4: u32 = 16;

pub const EXTCONN_STRONG: DWORD = 0x0001;

pub const CALLTYPE_TOPLEVEL: CALLTYPE = 1;
pub const CALLTYPE_NESTED: CALLTYPE = 2;
pub const CALLTYPE_ASYNC: CALLTYPE = 3;
pub const CALLTYPE_TOPLEVEL_CALLPENDING: CALLTYPE = 4;
pub const SERVERCALL_ISHANDLED: DWORD = 0;
pub const SERVERCALL_REJECTED: DWORD = 1;
pub const SERVERCALL_RETRYLATER: DWORD = 2;

pub const COINIT_MULTITHREADED: DWORD = 0;

pub const CLSCTX_INPROC_SERVER: DWORD = 0x1;
/// compobj_private.h WINE_CLSCTX_DONT_HOST
pub const WINE_CLSCTX_DONT_HOST: DWORD = 0x80000000;

pub const RPC_C_AUTHN_LEVEL_DEFAULT: DWORD = 0;
pub const RPC_C_AUTHN_LEVEL_NONE: DWORD = 1;
pub const RPC_C_IMP_LEVEL_DEFAULT: DWORD = 0;
pub const RPC_C_LISTEN_MAX_CALLS_DEFAULT: u32 = 1234;
pub const RPC_C_PROTSEQ_MAX_REQS_DEFAULT: u32 = 10;
pub const RPC_IF_AUTOLISTEN: u32 = 0x01;
pub const RPC_IF_OLE: u32 = 0x02;
pub const RPC_FLAGS_VALID_BIT: u32 = 0x00008000;
pub const EOAC_NONE: DWORD = 0;

pub const HEAP_ZERO_MEMORY: DWORD = 0x00000008;
pub const INFINITE: DWORD = 0xFFFFFFFF;
pub const WAIT_OBJECT_0: DWORD = 0;
pub const WT_EXECUTEDEFAULT: ULONG = 0;
pub const DETACHED_PROCESS: DWORD = 0x00000008;
pub const GENERIC_READ: DWORD = 0x80000000;
pub const GENERIC_WRITE: DWORD = 0x40000000;
pub const OPEN_EXISTING: DWORD = 3;
pub const NMPWAIT_WAIT_FOREVER: DWORD = 0xFFFFFFFF;
pub const PIPE_ACCESS_DUPLEX: DWORD = 3;
pub const FILE_FLAG_OVERLAPPED: DWORD = 0x40000000;
pub const PIPE_TYPE_BYTE: DWORD = 0;
pub const PIPE_WAIT: DWORD = 0;
pub const PIPE_UNLIMITED_INSTANCES: DWORD = 255;
pub const KEY_READ: REGSAM = 0x20019;
pub const REG_SZ: DWORD = 1;
pub const SC_MANAGER_CONNECT: DWORD = 1;
pub const SERVICE_START: DWORD = 16;
pub const STATFLAG_NONAME: DWORD = 1;
pub const STREAM_SEEK_SET: DWORD = 0;

/// compobj_private.h: DCOM messages used by the apartment window
pub const WM_USER: u32 = 0x0400;
pub const DM_EXECUTERPC: u32 = WM_USER + 0;
pub const DM_HOSTOBJECT: u32 = WM_USER + 1;

pub const CHARS_IN_GUID: usize = 39;

// ---------------------------------------------------------------------------
// Wine debug channel macros.
// Convention (noted in MANIFEST): TRACE/WARN compile to no-ops (wine default:
// those channels are off), ERR/FIXME are LOUD — they emit the file:line and
// the raw format string via OutputDebugStringA (format ARGUMENTS are not
// interpolated; no_std/no_alloc). This keeps every error path visible.
// ---------------------------------------------------------------------------
macro_rules! TRACE {
    ($($t:tt)*) => {{}};
}
macro_rules! WARN {
    ($($t:tt)*) => {{}};
}
// ERR/FIXME reach the loud sink by SYMBOL (an extern decl is expanded at each
// use site) so the macros work regardless of how the integrator lays out the
// modules — no `$crate::` path dependence.
macro_rules! ERR {
    ($fmt:literal $($rest:tt)*) => {{
        extern "C" {
            fn ole32_rust_dbg_out(s: *const core::ffi::c_char);
        }
        unsafe {
            ole32_rust_dbg_out(
                concat!("ole32:err:", file!(), ":", line!(), ": ", $fmt, "\0").as_ptr()
                    as *const core::ffi::c_char,
            )
        }
    }};
    ($($t:tt)*) => {{
        extern "C" {
            fn ole32_rust_dbg_out(s: *const core::ffi::c_char);
        }
        unsafe {
            ole32_rust_dbg_out(
                concat!("ole32:err:", file!(), ":", line!(), "\n\0").as_ptr()
                    as *const core::ffi::c_char,
            )
        }
    }};
}
macro_rules! FIXME {
    ($fmt:literal $($rest:tt)*) => {{
        extern "C" {
            fn ole32_rust_dbg_out(s: *const core::ffi::c_char);
        }
        unsafe {
            ole32_rust_dbg_out(
                concat!("ole32:fixme:", file!(), ":", line!(), ": ", $fmt, "\0").as_ptr()
                    as *const core::ffi::c_char,
            )
        }
    }};
    ($($t:tt)*) => {{
        extern "C" {
            fn ole32_rust_dbg_out(s: *const core::ffi::c_char);
        }
        unsafe {
            ole32_rust_dbg_out(
                concat!("ole32:fixme:", file!(), ":", line!(), "\n\0").as_ptr()
                    as *const core::ffi::c_char,
            )
        }
    }};
}
pub(crate) use ERR;
pub(crate) use FIXME;
pub(crate) use TRACE;
pub(crate) use WARN;

/// The loud sink behind ERR!/FIXME! — a linker-visible symbol so the macros
/// can reach it from any module layout. `s` must be NUL-terminated.
#[no_mangle]
pub extern "C" fn ole32_rust_dbg_out(s: *const c_char) {
    unsafe { OutputDebugStringA(s) }
}

/// C `assert()` translation: the shipping C is compiled NDEBUG (assert is a
/// no-op); we keep the check as a LOUD print (no abort) so a violated
/// invariant is visible instead of silent.
macro_rules! c_assert {
    ($cond:expr) => {
        if !($cond) {
            extern "C" {
                fn ole32_rust_dbg_out(s: *const core::ffi::c_char);
            }
            unsafe {
                ole32_rust_dbg_out(
                    concat!(
                        "ole32:err:ASSERT FAILED: ",
                        stringify!($cond),
                        " at ",
                        file!(),
                        ":",
                        line!(),
                        "\n\0"
                    )
                    .as_ptr() as *const core::ffi::c_char,
                )
            }
        }
    };
}
pub(crate) use c_assert;

// ---------------------------------------------------------------------------
// CONTAINING_RECORD
// ---------------------------------------------------------------------------
macro_rules! containing_record {
    ($ptr:expr, $ty:ty, $field:ident) => {
        ($ptr as *mut u8).sub(core::mem::offset_of!($ty, $field)) as *mut $ty
    };
}
pub(crate) use containing_record;

// ---------------------------------------------------------------------------
// wine/list.h mirror (doubly-linked circular list)
// ---------------------------------------------------------------------------
#[repr(C)]
pub struct list {
    pub next: *mut list,
    pub prev: *mut list,
}

#[inline]
pub unsafe fn list_init(l: *mut list) {
    (*l).next = l;
    (*l).prev = l;
}
#[inline]
pub unsafe fn list_add_after(elem: *mut list, to_add: *mut list) {
    (*to_add).next = (*elem).next;
    (*to_add).prev = elem;
    (*(*elem).next).prev = to_add;
    (*elem).next = to_add;
}
#[inline]
pub unsafe fn list_add_before(elem: *mut list, to_add: *mut list) {
    (*to_add).next = elem;
    (*to_add).prev = (*elem).prev;
    (*(*elem).prev).next = to_add;
    (*elem).prev = to_add;
}
#[inline]
pub unsafe fn list_add_head(l: *mut list, elem: *mut list) {
    list_add_after(l, elem);
}
#[inline]
pub unsafe fn list_add_tail(l: *mut list, elem: *mut list) {
    list_add_before(l, elem);
}
#[inline]
pub unsafe fn list_remove(elem: *mut list) {
    (*(*elem).next).prev = (*elem).prev;
    (*(*elem).prev).next = (*elem).next;
}
#[inline]
pub unsafe fn list_next(l: *const list, elem: *const list) -> *mut list {
    let ret = (*elem).next;
    if ret as *const list == l {
        core::ptr::null_mut()
    } else {
        ret
    }
}
#[inline]
pub unsafe fn list_head(l: *const list) -> *mut list {
    list_next(l, l)
}
#[inline]
pub unsafe fn list_empty(l: *const list) -> bool {
    (*l).next as *const list == l
}

// ---------------------------------------------------------------------------
// Interlocked helpers (winnt.h inline intrinsics — no exported symbols).
// Semantics match the C: return values follow the Win32 contracts.
// ---------------------------------------------------------------------------
use core::sync::atomic::{AtomicI32, AtomicPtr, Ordering};

#[inline]
pub unsafe fn InterlockedIncrement(addend: *mut LONG) -> LONG {
    (&*(addend as *const AtomicI32)).fetch_add(1, Ordering::SeqCst) + 1
}
#[inline]
pub unsafe fn InterlockedDecrement(addend: *mut LONG) -> LONG {
    (&*(addend as *const AtomicI32)).fetch_sub(1, Ordering::SeqCst) - 1
}
#[inline]
pub unsafe fn InterlockedExchange(target: *mut LONG, value: LONG) -> LONG {
    (&*(target as *const AtomicI32)).swap(value, Ordering::SeqCst)
}
#[inline]
pub unsafe fn InterlockedExchangeAdd(addend: *mut LONG, value: LONG) -> LONG {
    (&*(addend as *const AtomicI32)).fetch_add(value, Ordering::SeqCst)
}
#[inline]
pub unsafe fn InterlockedCompareExchange(dest: *mut LONG, exchange: LONG, comparand: LONG) -> LONG {
    match (&*(dest as *const AtomicI32)).compare_exchange(
        comparand,
        exchange,
        Ordering::SeqCst,
        Ordering::SeqCst,
    ) {
        Ok(v) => v,
        Err(v) => v,
    }
}
#[inline]
pub unsafe fn InterlockedExchangePointer(target: *mut PVOID, value: PVOID) -> PVOID {
    (&*(target as *const AtomicPtr<c_void>)).swap(value, Ordering::SeqCst)
}
#[inline]
pub unsafe fn InterlockedCompareExchangePointer(
    dest: *mut PVOID,
    exchange: PVOID,
    comparand: PVOID,
) -> PVOID {
    match (&*(dest as *const AtomicPtr<c_void>)).compare_exchange(
        comparand,
        exchange,
        Ordering::SeqCst,
        Ordering::SeqCst,
    ) {
        Ok(v) => v,
        Err(v) => v,
    }
}

// ---------------------------------------------------------------------------
// min helper (C used min() from a macro)
// ---------------------------------------------------------------------------
#[inline]
pub fn min_u32(a: u32, b: u32) -> u32 {
    if a < b {
        a
    } else {
        b
    }
}

// ---------------------------------------------------------------------------
// CRT externs
// ---------------------------------------------------------------------------
extern "C" {
    pub fn memcpy(dst: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn memset(dst: *mut c_void, c: c_int, n: usize) -> *mut c_void;
}

// ---------------------------------------------------------------------------
// heap helpers (wine/heap.h inlines)
// ---------------------------------------------------------------------------
#[inline]
pub unsafe fn heap_alloc(len: usize) -> PVOID {
    HeapAlloc(GetProcessHeap(), 0, len)
}
#[inline]
pub unsafe fn heap_alloc_zero(len: usize) -> PVOID {
    HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, len)
}
#[inline]
pub unsafe fn heap_free(mem: PVOID) -> BOOL {
    HeapFree(GetProcessHeap(), 0, mem)
}

// ---------------------------------------------------------------------------
// CRITICAL_SECTION (winnt.h RTL_CRITICAL_SECTION mirror)
// ---------------------------------------------------------------------------
#[repr(C)]
pub struct CRITICAL_SECTION {
    pub DebugInfo: *mut c_void, // PRTL_CRITICAL_SECTION_DEBUG (diagnostic only here)
    pub LockCount: LONG,
    pub RecursionCount: LONG,
    pub OwningThread: HANDLE,
    pub LockSemaphore: HANDLE,
    pub SpinCount: ULONG_PTR,
}

/// Statically-initialized (never InitializeCriticalSection'd) critical
/// section, mirroring the C's `static CRITICAL_SECTION cs = { &debug, -1, 0,
/// 0, 0, 0 };` pattern. DebugInfo is left NULL — ReactOS RTL explicitly
/// treats NULL/-1 DebugInfo as "no debug info" (sdk/lib/rtl/critical.c);
/// only the wine debug NAME is lost (noted in MANIFEST).
pub const fn static_critsec() -> CRITICAL_SECTION {
    CRITICAL_SECTION {
        DebugInfo: core::ptr::null_mut(),
        LockCount: -1,
        RecursionCount: 0,
        OwningThread: core::ptr::null_mut(),
        LockSemaphore: core::ptr::null_mut(),
        SpinCount: 0,
    }
}

/// compobj_private.h DEBUG_SET_CRITSEC_NAME / DEBUG_CLEAR_CRITSEC_NAME:
/// diagnostic-only (writes a name into DebugInfo->Spare[0]); translated as
/// no-ops (noted in MANIFEST).
#[inline]
pub unsafe fn DEBUG_SET_CRITSEC_NAME(_cs: *mut CRITICAL_SECTION, _name: &str) {}
#[inline]
pub unsafe fn DEBUG_CLEAR_CRITSEC_NAME(_cs: *mut CRITICAL_SECTION) {}

// ---------------------------------------------------------------------------
// TEB access + COM_Current* (compobj_private.h static inlines — implemented
// locally, NOT extern; Lane A implements the same inlines from the same
// header, so behavior/layout must match exactly).
// ---------------------------------------------------------------------------
#[cfg(target_arch = "x86")]
#[inline(always)]
pub unsafe fn NtCurrentTeb() -> *mut u8 {
    let teb: *mut u8;
    core::arch::asm!("mov {}, fs:[0x18]", out(reg) teb, options(nostack, pure, readonly));
    teb
}
#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub unsafe fn NtCurrentTeb() -> *mut u8 {
    let teb: *mut u8;
    core::arch::asm!("mov {}, gs:[0x30]", out(reg) teb, options(nostack, pure, readonly));
    teb
}
#[cfg(target_arch = "aarch64")]
#[inline(always)]
pub unsafe fn NtCurrentTeb() -> *mut u8 {
    let teb: *mut u8;
    core::arch::asm!("mov {}, x18", out(reg) teb, options(nostack, pure, nomem));
    teb
}

/// TEB.ReservedForOle byte offset. i386: 0xF80 (the compobj_private.h comment
/// says exactly this); amd64/arm64 (LP64 TEB layout): 0x1758.
/// Integration should swap this for the bindgen TEB field access.
// nolint: magic-offset (TEB.ReservedForOle per winternl TEB layout; no bindgen TEB in this staging crate)
#[cfg(target_arch = "x86")]
pub const TEB_RESERVED_FOR_OLE: usize = 0xF80;
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
pub const TEB_RESERVED_FOR_OLE: usize = 0x1758;

#[inline]
pub unsafe fn teb_reserved_for_ole() -> *mut *mut oletls {
    NtCurrentTeb().add(TEB_RESERVED_FOR_OLE) as *mut *mut oletls
}

/// compobj_private.h COM_CurrentInfo(): per-thread oletls, created on demand.
pub unsafe fn COM_CurrentInfo() -> *mut oletls {
    let slot = teb_reserved_for_ole();
    if (*slot).is_null() {
        let info = heap_alloc_zero(core::mem::size_of::<oletls>()) as *mut oletls;
        if !info.is_null() {
            list_init(&mut (*info).spies);
        }
        *slot = info;
    }
    *slot
}

/// compobj_private.h COM_CurrentApt()
pub unsafe fn COM_CurrentApt() -> *mut apartment {
    let info = COM_CurrentInfo();
    if info.is_null() {
        core::ptr::null_mut()
    } else {
        (*info).apt
    }
}

/// compobj_private.h COM_CurrentCausalityId()
pub unsafe fn COM_CurrentCausalityId() -> GUID {
    let info = COM_CurrentInfo();
    if info.is_null() {
        return GUID_NULL;
    }
    if (*info).causality_id == GUID_NULL {
        CoCreateGuid(&mut (*info).causality_id);
    }
    (*info).causality_id
}

// ---------------------------------------------------------------------------
// dcom.h wire types (from dcom.idl)
// ---------------------------------------------------------------------------
#[repr(C)]
#[derive(Copy, Clone)]
pub struct COMVERSION {
    pub MajorVersion: u16,
    pub MinorVersion: u16,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ORPC_EXTENT {
    pub id: GUID,
    pub size: u32,
    pub data: [u8; 0], // flexible array
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ORPC_EXTENT_ARRAY {
    pub size: u32,
    pub reserved: u32,
    pub extent: *mut *mut ORPC_EXTENT,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ORPCTHIS {
    pub version: COMVERSION,
    pub flags: u32,
    pub reserved1: u32,
    pub cid: GUID,
    pub extensions: *mut ORPC_EXTENT_ARRAY,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ORPCTHAT {
    pub flags: u32,
    pub extensions: *mut ORPC_EXTENT_ARRAY,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct DUALSTRINGARRAY {
    pub wNumEntries: u16,
    pub wSecurityOffset: u16,
    pub aStringArray: [u16; 1], // conformant array — header carries 1 slot like widl's dcom.h
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct OXID_INFO {
    pub dwTid: DWORD,
    pub dwPid: DWORD,
    pub ipidRemUnknown: IPID,
    pub dwAuthnHint: DWORD,
    pub psa: *mut DUALSTRINGARRAY,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct STDOBJREF {
    pub flags: u32,
    pub cPublicRefs: u32,
    pub oxid: OXID,
    pub oid: OID,
    pub ipid: IPID,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct OR_STANDARD {
    pub std: STDOBJREF,
    pub saResAddr: DUALSTRINGARRAY,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct OR_HANDLER {
    pub std: STDOBJREF,
    pub clsid: CLSID,
    pub saResAddr: DUALSTRINGARRAY,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct OR_CUSTOM {
    pub clsid: CLSID,
    pub cbExtension: u32,
    pub size: u32,
    pub pData: *mut u8,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union OBJREF_UNION {
    pub u_standard: OR_STANDARD,
    pub u_handler: OR_HANDLER,
    pub u_custom: OR_CUSTOM,
}

#[repr(C)]
pub struct OBJREF {
    pub signature: u32,
    pub flags: u32,
    pub iid: IID,
    pub u_objref: OBJREF_UNION,
}

// FIELD_OFFSET consts the C uses on OBJREF / OR_STANDARD.
/// FIELD_OFFSET(OBJREF, u_objref) — the common OBJREF header size.
pub const OBJREF_HDR_SIZE: usize = core::mem::offset_of!(OBJREF, u_objref);
/// FIELD_OFFSET(OBJREF, u_objref.u_standard.saResAddr.aStringArray)
pub const OBJREF_STANDARD_SIZE: usize = OBJREF_HDR_SIZE
    + core::mem::offset_of!(OR_STANDARD, saResAddr)
    + core::mem::offset_of!(DUALSTRINGARRAY, aStringArray);
/// FIELD_OFFSET(struct OR_STANDARD, saResAddr.aStringArray)
pub const OR_STANDARD_SIZE: usize = core::mem::offset_of!(OR_STANDARD, saResAddr)
    + core::mem::offset_of!(DUALSTRINGARRAY, aStringArray);
/// FIELD_OFFSET(OBJREF, u_objref.u_custom.pData)
pub const OBJREF_CUSTOM_HDR_SIZE: usize =
    OBJREF_HDR_SIZE + core::mem::offset_of!(OR_CUSTOM, pData);

#[repr(C)]
#[derive(Copy, Clone)]
pub struct REMQIRESULT {
    pub hResult: HRESULT,
    pub std: STDOBJREF,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct REMINTERFACEREF {
    pub ipid: IPID,
    pub cPublicRefs: u32,
    pub cPrivateRefs: u32,
}

#[repr(C)]
pub struct MULTI_QI {
    pub pIID: *const IID,
    pub pItf: *mut IUnknown,
    pub hr: HRESULT,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct FILETIME {
    pub dwLowDateTime: DWORD,
    pub dwHighDateTime: DWORD,
}

#[repr(C)]
pub struct STATSTG {
    pub pwcsName: LPWSTR,
    pub r#type: DWORD,
    pub cbSize: ULARGE_INTEGER,
    pub mtime: FILETIME,
    pub ctime: FILETIME,
    pub atime: FILETIME,
    pub grfMode: DWORD,
    pub grfLocksSupported: DWORD,
    pub clsid: CLSID,
    pub grfStateBits: DWORD,
    pub reserved: DWORD,
}

#[repr(C)]
pub struct INTERFACEINFO {
    pub pUnk: *mut IUnknown,
    pub iid: IID,
    pub wMethod: WORD,
}

#[repr(C)]
pub struct SChannelHookCallInfo {
    pub iid: IID,
    pub cbSize: DWORD,
    pub uCausality: GUID,
    pub dwServerPid: DWORD,
    pub iMethod: DWORD,
    pub pObject: *mut c_void,
}

// ---------------------------------------------------------------------------
// compobj_private.h private-type mirrors (shared with Lane A — field order and
// types MUST match the header exactly; integration dedupes)
// ---------------------------------------------------------------------------

/// compobj_private.h `struct ifstub`
#[repr(C)]
pub struct ifstub {
    pub entry: list,                     // entry in stub_manager->ifstubs (CS stub_manager->lock)
    pub stubbuffer: *mut IRpcStubBuffer, // RO
    pub iid: IID,                        // RO
    pub ipid: IPID,                      // RO
    pub iface: *mut IUnknown,            // RO
    pub flags: MSHLFLAGS,                // RO
    pub chan: *mut IRpcChannelBuffer,    // RO
}

/// compobj_private.h `struct stub_manager`
#[repr(C)]
pub struct stub_manager {
    pub entry: list,   // entry in apartment stubmgr list (CS apt->cs)
    pub ifstubs: list, // list of active ifstubs (CS lock)
    pub lock: CRITICAL_SECTION,
    pub apt: *mut apartment, // owning apt (RO)
    pub extrefs: ULONG,      // number of 'external' references (CS lock)
    pub refs: ULONG,         // internal reference count (CS apt->cs)
    pub weakrefs: ULONG,     // number of weak references (CS lock)
    pub oid: OID,            // apartment-scoped unique identifier (RO)
    pub object: *mut IUnknown, // the object we are managing the stub for (RO)
    pub next_ipid: ULONG,    // currently unused (LOCK)
    pub oxid_info: OXID_INFO, // RO
    pub extern_conn: *mut IExternalConnection,
    pub norm_refs: ULONG,  // refcount of normal marshals (CS lock)
    pub disconnected: BOOL, // CoDisconnectObject has been called (CS lock)
}

/// compobj_private.h `struct ifproxy`
#[repr(C)]
pub struct ifproxy {
    pub entry: list,                    // entry in proxy_manager list (CS parent->cs)
    pub parent: *mut proxy_manager,     // owning proxy_manager (RO)
    pub iface: LPVOID,                  // interface pointer (RO)
    pub stdobjref: STDOBJREF,           // marshal data for this object (RO)
    pub iid: IID,                       // interface ID (RO)
    pub proxy: *mut IRpcProxyBuffer,    // interface proxy (RO)
    pub refs: ULONG,                    // imported (public) references (LOCK)
    pub chan: *mut IRpcChannelBuffer,   // channel to object (CS parent->cs)
}

/// compobj_private.h `struct apartment` (Lane A owns the lifecycle; we only
/// read/lock through this mirror)
#[repr(C)]
pub struct apartment {
    pub entry: list,
    pub refs: LONG,            // refcount of the apartment (LOCK)
    pub multi_threaded: BOOL,  // multi-threaded or single-threaded apartment? (RO)
    pub tid: DWORD,            // thread id (RO)
    pub oxid: OXID,            // object exporter ID (RO)
    pub ipidc: LONG,           // interface pointer ID counter, starts at 1 (LOCK)
    pub cs: CRITICAL_SECTION,  // thread safety
    pub proxies: list,         // imported objects (CS cs)
    pub stubmgrs: list,        // stub managers for exported objects (CS cs)
    pub remunk_exported: BOOL, // has the IRemUnknown for this apartment been created yet? (CS cs)
    pub remoting_started: LONG, // has the RPC system been started for this apartment? (LOCK)
    pub loaded_dlls: list,     // list of dlls loaded by this apartment (CS cs)
    pub host_apt_tid: DWORD,   // thread ID of apartment hosting objects of differing model (CS cs)
    pub host_apt_hwnd: HWND,   // handle to apartment window of host apartment (CS cs)
    pub local_server: *mut c_void, // LocalServer* (CS cs)
    pub being_destroyed: BOOL, // is currently being destroyed
    pub oidc: OID,             // object ID counter, starts at 1, zero invalid (CS cs)
    // STA-only fields
    pub win: HWND,                    // message window (LOCK)
    pub filter: *mut IMessageFilter, // message filter (CS cs)
    pub main: BOOL,                   // is this a main-threaded-apartment? (RO)
}

/// compobj_private.h `struct oletls` (TEB->ReservedForOle)
#[repr(C)]
pub struct oletls {
    pub apt: *mut apartment,
    pub errorinfo: *mut c_void,  // IErrorInfo*
    pub thread_seqid: DWORD,     // wine-10: returned with CoGetCurrentProcess
    pub flags: DWORD,            // wine-10 tlsdata_flags (+0Ch on x86); apps read it
    pub unknown0: *mut c_void,
    pub inits: DWORD,     // number of times CoInitializeEx called
    pub ole_inits: DWORD, // number of times OleInitialize called
    pub causality_id: GUID,
    pub pending_call_count_client: LONG,
    pub pending_call_count_server: LONG,
    pub unknown: DWORD,
    pub context_token: *mut c_void, // IObjContext*
    pub call_state: *mut IUnknown,
    pub unknown2: [DWORD; 46],
    pub cancel_object: *mut IUnknown,
    pub state: *mut IUnknown, // see CoSetState
    pub spies: list, // Spies installed with CoRegisterInitializeSpy
    pub spies_lock: DWORD,
}

/// compobj_private.h static inline apartment_getoxid()
#[inline]
pub unsafe fn apartment_getoxid(apt: *const apartment, oxid: *mut OXID) -> HRESULT {
    *oxid = (*apt).oxid;
    S_OK
}

// ---------------------------------------------------------------------------
// COM interfaces — hand-declared vtbl structs (single module by design; a
// com-sys bindgen crate replaces these at integration).
// extern "system" == stdcall on i386, C on amd64/arm64 (the COM convention).
// ---------------------------------------------------------------------------

/// Wrapper making a read-only static holding raw pointers shareable (constant
/// vtbl-pointer objects, mirroring C `static const` COM objects).
#[repr(transparent)]
pub struct ComStatic<T>(pub T);
unsafe impl<T> Sync for ComStatic<T> {}

// ---- method-dispatch plumbing ----
// COM calls are made straight on the raw interface pointer
// (`p.QueryInterface(...)`), mirroring the C's IFoo_Method(p, ...) COBJMACROS.
// Each interface gets a pointer-trait implemented for `*mut IFoo` (the
// receiver is the pointer itself, by value — stable Rust, unlike
// `self: *mut Self` inherent methods).
macro_rules! com_methods {
    ($iface:ident, $ptr_trait:ident { $( fn $m:ident(self $(, $a:ident : $t:ty)* $(,)?) -> $r:ty => $slot:ident; )* }) => {
        pub trait $ptr_trait {
            $( unsafe fn $m(self $(, $a: $t)*) -> $r; )*
        }
        impl $ptr_trait for *mut $iface {
            $(
                #[inline]
                unsafe fn $m(self $(, $a: $t)*) -> $r {
                    ((*(*self).lpVtbl).$slot)(self $(, $a)*)
                }
            )*
        }
    };
}
pub(crate) use com_methods;

/// Declares an interface struct `$iface { lpVtbl: *const $vtbl }`.
macro_rules! com_iface {
    ($iface:ident, $vtbl:ident) => {
        #[repr(C)]
        pub struct $iface {
            pub lpVtbl: *const $vtbl,
        }
    };
}

// ---- IUnknown ----
#[repr(C)]
pub struct IUnknownVtbl {
    pub QueryInterface:
        unsafe extern "system" fn(*mut IUnknown, REFIID, *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(*mut IUnknown) -> ULONG,
    pub Release: unsafe extern "system" fn(*mut IUnknown) -> ULONG,
}
com_iface!(IUnknown, IUnknownVtbl);
com_methods!(IUnknown, IUnknownPtr {
    fn QueryInterface(self, riid: REFIID, ppv: *mut *mut c_void) -> HRESULT => QueryInterface;
    fn AddRef(self) -> ULONG => AddRef;
    fn Release(self) -> ULONG => Release;
});

// ---- IClassFactory ----
#[repr(C)]
pub struct IClassFactoryVtbl {
    pub QueryInterface:
        unsafe extern "system" fn(*mut IClassFactory, REFIID, *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(*mut IClassFactory) -> ULONG,
    pub Release: unsafe extern "system" fn(*mut IClassFactory) -> ULONG,
    pub CreateInstance: unsafe extern "system" fn(
        *mut IClassFactory,
        *mut IUnknown,
        REFIID,
        *mut *mut c_void,
    ) -> HRESULT,
    pub LockServer: unsafe extern "system" fn(*mut IClassFactory, BOOL) -> HRESULT,
}
com_iface!(IClassFactory, IClassFactoryVtbl);
com_methods!(IClassFactory, IClassFactoryPtr {
    fn QueryInterface(self, riid: REFIID, ppv: *mut *mut c_void) -> HRESULT => QueryInterface;
    fn AddRef(self) -> ULONG => AddRef;
    fn Release(self) -> ULONG => Release;
    fn CreateInstance(self, pUnkOuter: *mut IUnknown, riid: REFIID, ppv: *mut *mut c_void) -> HRESULT => CreateInstance;
    fn LockServer(self, fLock: BOOL) -> HRESULT => LockServer;
});

// ---- IMarshal ----
#[repr(C)]
pub struct IMarshalVtbl {
    pub QueryInterface:
        unsafe extern "system" fn(*mut IMarshal, REFIID, *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(*mut IMarshal) -> ULONG,
    pub Release: unsafe extern "system" fn(*mut IMarshal) -> ULONG,
    pub GetUnmarshalClass: unsafe extern "system" fn(
        *mut IMarshal,
        REFIID,
        *mut c_void,
        DWORD,
        *mut c_void,
        DWORD,
        *mut CLSID,
    ) -> HRESULT,
    pub GetMarshalSizeMax: unsafe extern "system" fn(
        *mut IMarshal,
        REFIID,
        *mut c_void,
        DWORD,
        *mut c_void,
        DWORD,
        *mut DWORD,
    ) -> HRESULT,
    pub MarshalInterface: unsafe extern "system" fn(
        *mut IMarshal,
        *mut IStream,
        REFIID,
        *mut c_void,
        DWORD,
        *mut c_void,
        DWORD,
    ) -> HRESULT,
    pub UnmarshalInterface: unsafe extern "system" fn(
        *mut IMarshal,
        *mut IStream,
        REFIID,
        *mut *mut c_void,
    ) -> HRESULT,
    pub ReleaseMarshalData: unsafe extern "system" fn(*mut IMarshal, *mut IStream) -> HRESULT,
    pub DisconnectObject: unsafe extern "system" fn(*mut IMarshal, DWORD) -> HRESULT,
}
com_iface!(IMarshal, IMarshalVtbl);
pub type LPMARSHAL = *mut IMarshal;
com_methods!(IMarshal, IMarshalPtr {
    fn QueryInterface(self, riid: REFIID, ppv: *mut *mut c_void) -> HRESULT => QueryInterface;
    fn AddRef(self) -> ULONG => AddRef;
    fn Release(self) -> ULONG => Release;
    fn GetUnmarshalClass(self, riid: REFIID, pv: *mut c_void, dwDestContext: DWORD, pvDestContext: *mut c_void, mshlflags: DWORD, pCid: *mut CLSID) -> HRESULT => GetUnmarshalClass;
    fn GetMarshalSizeMax(self, riid: REFIID, pv: *mut c_void, dwDestContext: DWORD, pvDestContext: *mut c_void, mshlflags: DWORD, pSize: *mut DWORD) -> HRESULT => GetMarshalSizeMax;
    fn MarshalInterface(self, pStm: *mut IStream, riid: REFIID, pv: *mut c_void, dwDestContext: DWORD, pvDestContext: *mut c_void, mshlflags: DWORD) -> HRESULT => MarshalInterface;
    fn UnmarshalInterface(self, pStm: *mut IStream, riid: REFIID, ppv: *mut *mut c_void) -> HRESULT => UnmarshalInterface;
    fn ReleaseMarshalData(self, pStm: *mut IStream) -> HRESULT => ReleaseMarshalData;
    fn DisconnectObject(self, dwReserved: DWORD) -> HRESULT => DisconnectObject;
});

// ---- IMultiQI ----
#[repr(C)]
pub struct IMultiQIVtbl {
    pub QueryInterface:
        unsafe extern "system" fn(*mut IMultiQI, REFIID, *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(*mut IMultiQI) -> ULONG,
    pub Release: unsafe extern "system" fn(*mut IMultiQI) -> ULONG,
    pub QueryMultipleInterfaces:
        unsafe extern "system" fn(*mut IMultiQI, ULONG, *mut MULTI_QI) -> HRESULT,
}
com_iface!(IMultiQI, IMultiQIVtbl);
com_methods!(IMultiQI, IMultiQIPtr {
    fn QueryInterface(self, riid: REFIID, ppv: *mut *mut c_void) -> HRESULT => QueryInterface;
    fn AddRef(self) -> ULONG => AddRef;
    fn Release(self) -> ULONG => Release;
    fn QueryMultipleInterfaces(self, cMQIs: ULONG, pMQIs: *mut MULTI_QI) -> HRESULT => QueryMultipleInterfaces;
});

// ---- IClientSecurity ----
#[repr(C)]
pub struct IClientSecurityVtbl {
    pub QueryInterface:
        unsafe extern "system" fn(*mut IClientSecurity, REFIID, *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(*mut IClientSecurity) -> ULONG,
    pub Release: unsafe extern "system" fn(*mut IClientSecurity) -> ULONG,
    pub QueryBlanket: unsafe extern "system" fn(
        *mut IClientSecurity,
        *mut IUnknown,
        *mut DWORD,
        *mut DWORD,
        *mut *mut OLECHAR,
        *mut DWORD,
        *mut DWORD,
        *mut *mut c_void,
        *mut DWORD,
    ) -> HRESULT,
    pub SetBlanket: unsafe extern "system" fn(
        *mut IClientSecurity,
        *mut IUnknown,
        DWORD,
        DWORD,
        *mut OLECHAR,
        DWORD,
        DWORD,
        *mut c_void,
        DWORD,
    ) -> HRESULT,
    pub CopyProxy: unsafe extern "system" fn(
        *mut IClientSecurity,
        *mut IUnknown,
        *mut *mut IUnknown,
    ) -> HRESULT,
}
com_iface!(IClientSecurity, IClientSecurityVtbl);
com_methods!(IClientSecurity, IClientSecurityPtr {
    fn QueryInterface(self, riid: REFIID, ppv: *mut *mut c_void) -> HRESULT => QueryInterface;
    fn AddRef(self) -> ULONG => AddRef;
    fn Release(self) -> ULONG => Release;
});

// ---- IRemUnknown ----
#[repr(C)]
pub struct IRemUnknownVtbl {
    pub QueryInterface:
        unsafe extern "system" fn(*mut IRemUnknown, REFIID, *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(*mut IRemUnknown) -> ULONG,
    pub Release: unsafe extern "system" fn(*mut IRemUnknown) -> ULONG,
    pub RemQueryInterface: unsafe extern "system" fn(
        *mut IRemUnknown,
        REFIPID,
        ULONG,
        USHORT,
        *mut IID,
        *mut *mut REMQIRESULT,
    ) -> HRESULT,
    pub RemAddRef: unsafe extern "system" fn(
        *mut IRemUnknown,
        USHORT,
        *mut REMINTERFACEREF,
        *mut HRESULT,
    ) -> HRESULT,
    pub RemRelease:
        unsafe extern "system" fn(*mut IRemUnknown, USHORT, *mut REMINTERFACEREF) -> HRESULT,
}
com_iface!(IRemUnknown, IRemUnknownVtbl);
com_methods!(IRemUnknown, IRemUnknownPtr {
    fn QueryInterface(self, riid: REFIID, ppv: *mut *mut c_void) -> HRESULT => QueryInterface;
    fn AddRef(self) -> ULONG => AddRef;
    fn Release(self) -> ULONG => Release;
    fn RemQueryInterface(self, ripid: REFIPID, cRefs: ULONG, cIids: USHORT, iids: *mut IID, ppQIResults: *mut *mut REMQIRESULT) -> HRESULT => RemQueryInterface;
    fn RemAddRef(self, cInterfaceRefs: USHORT, InterfaceRefs: *mut REMINTERFACEREF, pResults: *mut HRESULT) -> HRESULT => RemAddRef;
    fn RemRelease(self, cInterfaceRefs: USHORT, InterfaceRefs: *mut REMINTERFACEREF) -> HRESULT => RemRelease;
});

// ---- IStream (ISequentialStream + IStream) ----
#[repr(C)]
pub struct IStreamVtbl {
    pub QueryInterface:
        unsafe extern "system" fn(*mut IStream, REFIID, *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(*mut IStream) -> ULONG,
    pub Release: unsafe extern "system" fn(*mut IStream) -> ULONG,
    pub Read: unsafe extern "system" fn(*mut IStream, *mut c_void, ULONG, *mut ULONG) -> HRESULT,
    pub Write:
        unsafe extern "system" fn(*mut IStream, *const c_void, ULONG, *mut ULONG) -> HRESULT,
    pub Seek: unsafe extern "system" fn(
        *mut IStream,
        LARGE_INTEGER,
        DWORD,
        *mut ULARGE_INTEGER,
    ) -> HRESULT,
    pub SetSize: unsafe extern "system" fn(*mut IStream, ULARGE_INTEGER) -> HRESULT,
    pub CopyTo: unsafe extern "system" fn(
        *mut IStream,
        *mut IStream,
        ULARGE_INTEGER,
        *mut ULARGE_INTEGER,
        *mut ULARGE_INTEGER,
    ) -> HRESULT,
    pub Commit: unsafe extern "system" fn(*mut IStream, DWORD) -> HRESULT,
    pub Revert: unsafe extern "system" fn(*mut IStream) -> HRESULT,
    pub LockRegion: unsafe extern "system" fn(
        *mut IStream,
        ULARGE_INTEGER,
        ULARGE_INTEGER,
        DWORD,
    ) -> HRESULT,
    pub UnlockRegion: unsafe extern "system" fn(
        *mut IStream,
        ULARGE_INTEGER,
        ULARGE_INTEGER,
        DWORD,
    ) -> HRESULT,
    pub Stat: unsafe extern "system" fn(*mut IStream, *mut STATSTG, DWORD) -> HRESULT,
    pub Clone: unsafe extern "system" fn(*mut IStream, *mut *mut IStream) -> HRESULT,
}
com_iface!(IStream, IStreamVtbl);
pub type LPSTREAM = *mut IStream;
com_methods!(IStream, IStreamPtr {
    fn QueryInterface(self, riid: REFIID, ppv: *mut *mut c_void) -> HRESULT => QueryInterface;
    fn AddRef(self) -> ULONG => AddRef;
    fn Release(self) -> ULONG => Release;
    fn Read(self, pv: *mut c_void, cb: ULONG, pcbRead: *mut ULONG) -> HRESULT => Read;
    fn Write(self, pv: *const c_void, cb: ULONG, pcbWritten: *mut ULONG) -> HRESULT => Write;
    fn Seek(self, dlibMove: LARGE_INTEGER, dwOrigin: DWORD, plibNewPosition: *mut ULARGE_INTEGER) -> HRESULT => Seek;
    fn Stat(self, pstatstg: *mut STATSTG, grfStatFlag: DWORD) -> HRESULT => Stat;
    fn Clone(self, ppstm: *mut *mut IStream) -> HRESULT => Clone;
});

// ---- IRpcChannelBuffer ----
#[repr(C)]
pub struct RPCOLEMESSAGE {
    pub reserved1: *mut c_void,
    pub dataRepresentation: RPCOLEDATAREP,
    pub Buffer: *mut c_void,
    pub cbBuffer: ULONG,
    pub iMethod: ULONG,
    pub reserved2: [*mut c_void; 5],
    pub rpcFlags: ULONG,
}

#[repr(C)]
pub struct IRpcChannelBufferVtbl {
    pub QueryInterface:
        unsafe extern "system" fn(*mut IRpcChannelBuffer, REFIID, *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(*mut IRpcChannelBuffer) -> ULONG,
    pub Release: unsafe extern "system" fn(*mut IRpcChannelBuffer) -> ULONG,
    pub GetBuffer:
        unsafe extern "system" fn(*mut IRpcChannelBuffer, *mut RPCOLEMESSAGE, REFIID) -> HRESULT,
    pub SendReceive: unsafe extern "system" fn(
        *mut IRpcChannelBuffer,
        *mut RPCOLEMESSAGE,
        *mut ULONG,
    ) -> HRESULT,
    pub FreeBuffer:
        unsafe extern "system" fn(*mut IRpcChannelBuffer, *mut RPCOLEMESSAGE) -> HRESULT,
    pub GetDestCtx: unsafe extern "system" fn(
        *mut IRpcChannelBuffer,
        *mut DWORD,
        *mut *mut c_void,
    ) -> HRESULT,
    pub IsConnected: unsafe extern "system" fn(*mut IRpcChannelBuffer) -> HRESULT,
}
com_iface!(IRpcChannelBuffer, IRpcChannelBufferVtbl);
pub type LPRPCCHANNELBUFFER = *mut IRpcChannelBuffer;
com_methods!(IRpcChannelBuffer, IRpcChannelBufferPtr {
    fn QueryInterface(self, riid: REFIID, ppv: *mut *mut c_void) -> HRESULT => QueryInterface;
    fn AddRef(self) -> ULONG => AddRef;
    fn Release(self) -> ULONG => Release;
    fn GetDestCtx(self, pdwDestContext: *mut DWORD, ppvDestContext: *mut *mut c_void) -> HRESULT => GetDestCtx;
});

// ---- IRpcStubBuffer ----
#[repr(C)]
pub struct IRpcStubBufferVtbl {
    pub QueryInterface:
        unsafe extern "system" fn(*mut IRpcStubBuffer, REFIID, *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(*mut IRpcStubBuffer) -> ULONG,
    pub Release: unsafe extern "system" fn(*mut IRpcStubBuffer) -> ULONG,
    pub Connect: unsafe extern "system" fn(*mut IRpcStubBuffer, *mut IUnknown) -> HRESULT,
    pub Disconnect: unsafe extern "system" fn(*mut IRpcStubBuffer),
    pub Invoke: unsafe extern "system" fn(
        *mut IRpcStubBuffer,
        *mut RPCOLEMESSAGE,
        *mut IRpcChannelBuffer,
    ) -> HRESULT,
    pub IsIIDSupported:
        unsafe extern "system" fn(*mut IRpcStubBuffer, REFIID) -> *mut IRpcStubBuffer,
    pub CountRefs: unsafe extern "system" fn(*mut IRpcStubBuffer) -> ULONG,
    pub DebugServerQueryInterface:
        unsafe extern "system" fn(*mut IRpcStubBuffer, *mut *mut c_void) -> HRESULT,
    pub DebugServerRelease: unsafe extern "system" fn(*mut IRpcStubBuffer, *mut c_void),
}
com_iface!(IRpcStubBuffer, IRpcStubBufferVtbl);
com_methods!(IRpcStubBuffer, IRpcStubBufferPtr {
    fn QueryInterface(self, riid: REFIID, ppv: *mut *mut c_void) -> HRESULT => QueryInterface;
    fn AddRef(self) -> ULONG => AddRef;
    fn Release(self) -> ULONG => Release;
    fn Invoke(self, msg: *mut RPCOLEMESSAGE, chan: *mut IRpcChannelBuffer) -> HRESULT => Invoke;
});

// ---- IRpcProxyBuffer ----
#[repr(C)]
pub struct IRpcProxyBufferVtbl {
    pub QueryInterface:
        unsafe extern "system" fn(*mut IRpcProxyBuffer, REFIID, *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(*mut IRpcProxyBuffer) -> ULONG,
    pub Release: unsafe extern "system" fn(*mut IRpcProxyBuffer) -> ULONG,
    pub Connect:
        unsafe extern "system" fn(*mut IRpcProxyBuffer, *mut IRpcChannelBuffer) -> HRESULT,
    pub Disconnect: unsafe extern "system" fn(*mut IRpcProxyBuffer),
}
com_iface!(IRpcProxyBuffer, IRpcProxyBufferVtbl);
pub type LPRPCPROXYBUFFER = *mut IRpcProxyBuffer;
com_methods!(IRpcProxyBuffer, IRpcProxyBufferPtr {
    fn QueryInterface(self, riid: REFIID, ppv: *mut *mut c_void) -> HRESULT => QueryInterface;
    fn AddRef(self) -> ULONG => AddRef;
    fn Release(self) -> ULONG => Release;
    fn Connect(self, chan: *mut IRpcChannelBuffer) -> HRESULT => Connect;
    fn Disconnect(self) -> () => Disconnect;
});

// ---- IPSFactoryBuffer ----
#[repr(C)]
pub struct IPSFactoryBufferVtbl {
    pub QueryInterface:
        unsafe extern "system" fn(*mut IPSFactoryBuffer, REFIID, *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(*mut IPSFactoryBuffer) -> ULONG,
    pub Release: unsafe extern "system" fn(*mut IPSFactoryBuffer) -> ULONG,
    pub CreateProxy: unsafe extern "system" fn(
        *mut IPSFactoryBuffer,
        *mut IUnknown,
        REFIID,
        *mut *mut IRpcProxyBuffer,
        *mut *mut c_void,
    ) -> HRESULT,
    pub CreateStub: unsafe extern "system" fn(
        *mut IPSFactoryBuffer,
        REFIID,
        *mut IUnknown,
        *mut *mut IRpcStubBuffer,
    ) -> HRESULT,
}
com_iface!(IPSFactoryBuffer, IPSFactoryBufferVtbl);
com_methods!(IPSFactoryBuffer, IPSFactoryBufferPtr {
    fn QueryInterface(self, riid: REFIID, ppv: *mut *mut c_void) -> HRESULT => QueryInterface;
    fn AddRef(self) -> ULONG => AddRef;
    fn Release(self) -> ULONG => Release;
    fn CreateProxy(self, pUnkOuter: *mut IUnknown, riid: REFIID, ppProxy: *mut *mut IRpcProxyBuffer, ppv: *mut *mut c_void) -> HRESULT => CreateProxy;
    fn CreateStub(self, riid: REFIID, pUnkServer: *mut IUnknown, ppStub: *mut *mut IRpcStubBuffer) -> HRESULT => CreateStub;
});

// ---- IExternalConnection ----
#[repr(C)]
pub struct IExternalConnectionVtbl {
    pub QueryInterface:
        unsafe extern "system" fn(*mut IExternalConnection, REFIID, *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(*mut IExternalConnection) -> ULONG,
    pub Release: unsafe extern "system" fn(*mut IExternalConnection) -> ULONG,
    pub AddConnection: unsafe extern "system" fn(*mut IExternalConnection, DWORD, DWORD) -> DWORD,
    pub ReleaseConnection:
        unsafe extern "system" fn(*mut IExternalConnection, DWORD, DWORD, BOOL) -> DWORD,
}
com_iface!(IExternalConnection, IExternalConnectionVtbl);
com_methods!(IExternalConnection, IExternalConnectionPtr {
    fn QueryInterface(self, riid: REFIID, ppv: *mut *mut c_void) -> HRESULT => QueryInterface;
    fn AddRef(self) -> ULONG => AddRef;
    fn Release(self) -> ULONG => Release;
    fn AddConnection(self, extconn: DWORD, reserved: DWORD) -> DWORD => AddConnection;
    fn ReleaseConnection(self, extconn: DWORD, reserved: DWORD, fLastReleaseCloses: BOOL) -> DWORD => ReleaseConnection;
});

// ---- IChannelHook ----
#[repr(C)]
pub struct IChannelHookVtbl {
    pub QueryInterface:
        unsafe extern "system" fn(*mut IChannelHook, REFIID, *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(*mut IChannelHook) -> ULONG,
    pub Release: unsafe extern "system" fn(*mut IChannelHook) -> ULONG,
    pub ClientGetSize:
        unsafe extern "system" fn(*mut IChannelHook, REFGUID, REFIID, *mut ULONG),
    pub ClientFillBuffer: unsafe extern "system" fn(
        *mut IChannelHook,
        REFGUID,
        REFIID,
        *mut ULONG,
        *mut c_void,
    ),
    pub ClientNotify: unsafe extern "system" fn(
        *mut IChannelHook,
        REFGUID,
        REFIID,
        ULONG,
        *mut c_void,
        DWORD,
        HRESULT,
    ),
    pub ServerNotify: unsafe extern "system" fn(
        *mut IChannelHook,
        REFGUID,
        REFIID,
        ULONG,
        *mut c_void,
        DWORD,
    ),
    pub ServerGetSize:
        unsafe extern "system" fn(*mut IChannelHook, REFGUID, REFIID, HRESULT, *mut ULONG),
    pub ServerFillBuffer: unsafe extern "system" fn(
        *mut IChannelHook,
        REFGUID,
        REFIID,
        *mut ULONG,
        *mut c_void,
        HRESULT,
    ),
}
com_iface!(IChannelHook, IChannelHookVtbl);
com_methods!(IChannelHook, IChannelHookPtr {
    fn QueryInterface(self, riid: REFIID, ppv: *mut *mut c_void) -> HRESULT => QueryInterface;
    fn AddRef(self) -> ULONG => AddRef;
    fn Release(self) -> ULONG => Release;
});

// ---- IMessageFilter ----
#[repr(C)]
pub struct IMessageFilterVtbl {
    pub QueryInterface:
        unsafe extern "system" fn(*mut IMessageFilter, REFIID, *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(*mut IMessageFilter) -> ULONG,
    pub Release: unsafe extern "system" fn(*mut IMessageFilter) -> ULONG,
    pub HandleInComingCall: unsafe extern "system" fn(
        *mut IMessageFilter,
        DWORD,
        HTASK,
        DWORD,
        *mut INTERFACEINFO,
    ) -> DWORD,
    pub RetryRejectedCall:
        unsafe extern "system" fn(*mut IMessageFilter, HTASK, DWORD, DWORD) -> DWORD,
    pub MessagePending:
        unsafe extern "system" fn(*mut IMessageFilter, HTASK, DWORD, DWORD) -> DWORD,
}
com_iface!(IMessageFilter, IMessageFilterVtbl);
pub type LPMESSAGEFILTER = *mut IMessageFilter;

// ---- IServiceProvider ----
#[repr(C)]
pub struct IServiceProviderVtbl {
    pub QueryInterface:
        unsafe extern "system" fn(*mut IServiceProvider, REFIID, *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(*mut IServiceProvider) -> ULONG,
    pub Release: unsafe extern "system" fn(*mut IServiceProvider) -> ULONG,
    pub QueryService: unsafe extern "system" fn(
        *mut IServiceProvider,
        REFGUID,
        REFIID,
        *mut *mut c_void,
    ) -> HRESULT,
}
com_iface!(IServiceProvider, IServiceProviderVtbl);
com_methods!(IServiceProvider, IServiceProviderPtr {
    fn QueryInterface(self, riid: REFIID, ppv: *mut *mut c_void) -> HRESULT => QueryInterface;
    fn AddRef(self) -> ULONG => AddRef;
    fn Release(self) -> ULONG => Release;
    fn QueryService(self, guidService: REFGUID, riid: REFIID, ppv: *mut *mut c_void) -> HRESULT => QueryService;
});

// ---- IGlobalInterfaceTable ----
#[repr(C)]
pub struct IGlobalInterfaceTableVtbl {
    pub QueryInterface: unsafe extern "system" fn(
        *mut IGlobalInterfaceTable,
        REFIID,
        *mut *mut c_void,
    ) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(*mut IGlobalInterfaceTable) -> ULONG,
    pub Release: unsafe extern "system" fn(*mut IGlobalInterfaceTable) -> ULONG,
    pub RegisterInterfaceInGlobal: unsafe extern "system" fn(
        *mut IGlobalInterfaceTable,
        *mut IUnknown,
        REFIID,
        *mut DWORD,
    ) -> HRESULT,
    pub RevokeInterfaceFromGlobal:
        unsafe extern "system" fn(*mut IGlobalInterfaceTable, DWORD) -> HRESULT,
    pub GetInterfaceFromGlobal: unsafe extern "system" fn(
        *mut IGlobalInterfaceTable,
        DWORD,
        REFIID,
        *mut *mut c_void,
    ) -> HRESULT,
}
com_iface!(IGlobalInterfaceTable, IGlobalInterfaceTableVtbl);
com_methods!(IGlobalInterfaceTable, IGlobalInterfaceTablePtr {
    fn QueryInterface(self, riid: REFIID, ppv: *mut *mut c_void) -> HRESULT => QueryInterface;
    fn AddRef(self) -> ULONG => AddRef;
    fn Release(self) -> ULONG => Release;
});

// ---------------------------------------------------------------------------
// struct proxy_manager — the imported-object / proxy manager. In the C this
// is marshal.c-local; it lives here because compobj_private.h's `struct
// ifproxy.parent` points at it and types.rs owns all shared struct decls.
// Field order matches marshal.c exactly.
// ---------------------------------------------------------------------------
#[repr(C)]
pub struct proxy_manager {
    pub IMultiQI_iface: IMultiQI,
    pub IMarshal_iface: IMarshal,
    pub IClientSecurity_iface: IClientSecurity,
    pub parent: *mut apartment, // owning apartment (RO)
    pub entry: list,            // entry in apartment (CS parent->cs)
    pub oxid: OXID,             // object exported ID (RO)
    pub oxid_info: OXID_INFO,   // oxid info (RO)
    pub oid: OID,               // object ID (RO)
    pub interfaces: list,       // imported interfaces (CS cs)
    pub refs: LONG,             // proxy reference count (LOCK)
    pub cs: CRITICAL_SECTION,   // thread safety for this object and children
    pub sorflags: ULONG,        // STDOBJREF flags (RO)
    pub remunk: *mut IRemUnknown, // proxy to IRemUnknown for lifecycle mgmt (CS cs)
    pub remoting_mutex: HANDLE, // mutex for synchronizing access to IRemUnknown
    pub dest_context: MSHCTX,   // context used for activating optimisations (LOCK)
    pub dest_context_data: *mut c_void, // reserved context value (LOCK)
}

// ---------------------------------------------------------------------------
// RPC runtime (rpcdcep.h / rpcdce.h) types
// ---------------------------------------------------------------------------
#[repr(C)]
#[derive(Copy, Clone)]
pub struct RPC_VERSION {
    pub MajorVersion: u16,
    pub MinorVersion: u16,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct RPC_SYNTAX_IDENTIFIER {
    pub SyntaxGUID: GUID,
    pub SyntaxVersion: RPC_VERSION,
}

#[repr(C)]
pub struct RPC_MESSAGE {
    pub Handle: RPC_BINDING_HANDLE,
    pub DataRepresentation: ULONG,
    pub Buffer: *mut c_void,
    pub BufferLength: c_uint,
    pub ProcNum: c_uint,
    pub TransferSyntax: *mut RPC_SYNTAX_IDENTIFIER,
    pub RpcInterfaceInformation: *mut c_void,
    pub ReservedForRuntime: *mut c_void,
    pub ManagerEpv: *mut c_void,
    pub ImportContext: *mut c_void,
    pub RpcFlags: ULONG,
}

pub type RPC_DISPATCH_FUNCTION = unsafe extern "system" fn(*mut RPC_MESSAGE);

#[repr(C)]
pub struct RPC_DISPATCH_TABLE {
    pub DispatchTableCount: c_uint,
    pub DispatchTable: *mut RPC_DISPATCH_FUNCTION,
    pub Reserved: LONG_PTR,
}
unsafe impl Sync for RPC_DISPATCH_TABLE {}

#[repr(C)]
pub struct RPC_SERVER_INTERFACE {
    pub Length: c_uint,
    pub InterfaceId: RPC_SYNTAX_IDENTIFIER,
    pub TransferSyntax: RPC_SYNTAX_IDENTIFIER,
    pub DispatchTable: *mut RPC_DISPATCH_TABLE,
    pub RpcProtseqEndpointCount: c_uint,
    pub RpcProtseqEndpoint: *mut c_void,
    pub DefaultManagerEpv: *mut c_void,
    pub InterpreterInfo: *const c_void,
    pub Flags: c_uint,
}

#[repr(C)]
pub struct RPC_CLIENT_INTERFACE {
    pub Length: c_uint,
    pub InterfaceId: RPC_SYNTAX_IDENTIFIER,
    pub TransferSyntax: RPC_SYNTAX_IDENTIFIER,
    pub DispatchTable: *mut RPC_DISPATCH_TABLE,
    pub RpcProtseqEndpointCount: c_uint,
    pub RpcProtseqEndpoint: *mut c_void,
    pub Reserved: ULONG_PTR,
    pub InterpreterInfo: *const c_void,
    pub Flags: c_uint,
}

// ---------------------------------------------------------------------------
// Win32 misc structs used by rpc.c
// ---------------------------------------------------------------------------
#[repr(C)]
pub struct OVERLAPPED {
    pub Internal: ULONG_PTR,
    pub InternalHigh: ULONG_PTR,
    pub Offset: DWORD,
    pub OffsetHigh: DWORD,
    pub hEvent: HANDLE,
}

#[repr(C)]
pub struct STARTUPINFOW {
    pub cb: DWORD,
    pub lpReserved: LPWSTR,
    pub lpDesktop: LPWSTR,
    pub lpTitle: LPWSTR,
    pub dwX: DWORD,
    pub dwY: DWORD,
    pub dwXSize: DWORD,
    pub dwYSize: DWORD,
    pub dwXCountChars: DWORD,
    pub dwYCountChars: DWORD,
    pub dwFillAttribute: DWORD,
    pub dwFlags: DWORD,
    pub wShowWindow: WORD,
    pub cbReserved2: WORD,
    pub lpReserved2: LPBYTE,
    pub hStdInput: HANDLE,
    pub hStdOutput: HANDLE,
    pub hStdError: HANDLE,
}

#[repr(C)]
pub struct PROCESS_INFORMATION {
    pub hProcess: HANDLE,
    pub hThread: HANDLE,
    pub dwProcessId: DWORD,
    pub dwThreadId: DWORD,
}

pub type LPTHREAD_START_ROUTINE = unsafe extern "system" fn(LPVOID) -> DWORD;

// ---------------------------------------------------------------------------
// Extern imports — kernel32
// ---------------------------------------------------------------------------
extern "system" {
    pub fn GetProcessHeap() -> HANDLE;
    pub fn HeapAlloc(hHeap: HANDLE, dwFlags: DWORD, dwBytes: SIZE_T) -> PVOID;
    pub fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: PVOID) -> BOOL;
    pub fn GetCurrentProcessId() -> DWORD;
    pub fn GetCurrentThreadId() -> DWORD;
    pub fn GetLastError() -> DWORD;
    pub fn CloseHandle(hObject: HANDLE) -> BOOL;
    pub fn CreateEventW(
        lpEventAttributes: *mut c_void,
        bManualReset: BOOL,
        bInitialState: BOOL,
        lpName: LPCWSTR,
    ) -> HANDLE;
    pub fn SetEvent(hEvent: HANDLE) -> BOOL;
    pub fn CreateMutexW(
        lpMutexAttributes: *mut c_void,
        bInitialOwner: BOOL,
        lpName: LPCWSTR,
    ) -> HANDLE;
    pub fn ReleaseMutex(hMutex: HANDLE) -> BOOL;
    pub fn WaitForSingleObject(hHandle: HANDLE, dwMilliseconds: DWORD) -> DWORD;
    pub fn WaitForMultipleObjects(
        nCount: DWORD,
        lpHandles: *const HANDLE,
        bWaitAll: BOOL,
        dwMilliseconds: DWORD,
    ) -> DWORD;
    pub fn EnterCriticalSection(lpCriticalSection: *mut CRITICAL_SECTION);
    pub fn LeaveCriticalSection(lpCriticalSection: *mut CRITICAL_SECTION);
    pub fn InitializeCriticalSection(lpCriticalSection: *mut CRITICAL_SECTION);
    pub fn DeleteCriticalSection(lpCriticalSection: *mut CRITICAL_SECTION);
    pub fn CreateThread(
        lpThreadAttributes: *mut c_void,
        dwStackSize: SIZE_T,
        lpStartAddress: LPTHREAD_START_ROUTINE,
        lpParameter: LPVOID,
        dwCreationFlags: DWORD,
        lpThreadId: *mut DWORD,
    ) -> HANDLE;
    pub fn QueueUserWorkItem(
        Function: LPTHREAD_START_ROUTINE,
        Context: PVOID,
        Flags: ULONG,
    ) -> BOOL;
    pub fn GetTickCount() -> DWORD;
    pub fn Sleep(dwMilliseconds: DWORD);
    pub fn OutputDebugStringA(lpOutputString: LPCSTR);
    pub fn CreateFileW(
        lpFileName: LPCWSTR,
        dwDesiredAccess: DWORD,
        dwShareMode: DWORD,
        lpSecurityAttributes: *mut c_void,
        dwCreationDisposition: DWORD,
        dwFlagsAndAttributes: DWORD,
        hTemplateFile: HANDLE,
    ) -> HANDLE;
    pub fn ReadFile(
        hFile: HANDLE,
        lpBuffer: LPVOID,
        nNumberOfBytesToRead: DWORD,
        lpNumberOfBytesRead: *mut DWORD,
        lpOverlapped: *mut OVERLAPPED,
    ) -> BOOL;
    pub fn WriteFile(
        hFile: HANDLE,
        lpBuffer: LPCVOID,
        nNumberOfBytesToWrite: DWORD,
        lpNumberOfBytesWritten: *mut DWORD,
        lpOverlapped: *mut OVERLAPPED,
    ) -> BOOL;
    pub fn FlushFileBuffers(hFile: HANDLE) -> BOOL;
    pub fn CreateNamedPipeW(
        lpName: LPCWSTR,
        dwOpenMode: DWORD,
        dwPipeMode: DWORD,
        nMaxInstances: DWORD,
        nOutBufferSize: DWORD,
        nInBufferSize: DWORD,
        nDefaultTimeOut: DWORD,
        lpSecurityAttributes: *mut c_void,
    ) -> HANDLE;
    pub fn ConnectNamedPipe(hNamedPipe: HANDLE, lpOverlapped: *mut OVERLAPPED) -> BOOL;
    pub fn DisconnectNamedPipe(hNamedPipe: HANDLE) -> BOOL;
    pub fn WaitNamedPipeW(lpNamedPipeName: LPCWSTR, nTimeOut: DWORD) -> BOOL;
    pub fn GetOverlappedResult(
        hFile: HANDLE,
        lpOverlapped: *mut OVERLAPPED,
        lpNumberOfBytesTransferred: *mut DWORD,
        bWait: BOOL,
    ) -> BOOL;
    pub fn CreateProcessW(
        lpApplicationName: LPCWSTR,
        lpCommandLine: LPWSTR,
        lpProcessAttributes: *mut c_void,
        lpThreadAttributes: *mut c_void,
        bInheritHandles: BOOL,
        dwCreationFlags: DWORD,
        lpEnvironment: LPVOID,
        lpCurrentDirectory: LPCWSTR,
        lpStartupInfo: *mut STARTUPINFOW,
        lpProcessInformation: *mut PROCESS_INFORMATION,
    ) -> BOOL;
    pub fn lstrcpyW(lpString1: LPWSTR, lpString2: LPCWSTR) -> LPWSTR;
    pub fn lstrcatW(lpString1: LPWSTR, lpString2: LPCWSTR) -> LPWSTR;
}

// ---------------------------------------------------------------------------
// Extern imports — user32
// ---------------------------------------------------------------------------
extern "system" {
    pub fn PostMessageW(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> BOOL;
    pub fn InSendMessage() -> BOOL;
}

// ---------------------------------------------------------------------------
// Extern imports — advapi32 (service control + registry, rpc.c local-server path)
// ---------------------------------------------------------------------------
extern "system" {
    pub fn OpenSCManagerW(
        lpMachineName: LPCWSTR,
        lpDatabaseName: LPCWSTR,
        dwDesiredAccess: DWORD,
    ) -> SC_HANDLE;
    pub fn OpenServiceW(
        hSCManager: SC_HANDLE,
        lpServiceName: LPCWSTR,
        dwDesiredAccess: DWORD,
    ) -> SC_HANDLE;
    pub fn StartServiceW(
        hService: SC_HANDLE,
        dwNumServiceArgs: DWORD,
        lpServiceArgVectors: *const LPCWSTR,
    ) -> BOOL;
    pub fn CloseServiceHandle(hSCObject: SC_HANDLE) -> BOOL;
    pub fn RegQueryValueExW(
        hKey: HKEY,
        lpValueName: LPCWSTR,
        lpReserved: *mut DWORD,
        lpType: *mut DWORD,
        lpData: LPBYTE,
        lpcbData: *mut DWORD,
    ) -> LSTATUS;
    pub fn RegCloseKey(hKey: HKEY) -> LSTATUS;
}

// ---------------------------------------------------------------------------
// Extern imports — rpcrt4
// ---------------------------------------------------------------------------
extern "system" {
    pub fn UuidCreate(Uuid: *mut UUID) -> RPC_STATUS;
    pub fn UuidCreateNil(NilUuid: *mut UUID) -> RPC_STATUS;
    pub fn RpcStringBindingComposeW(
        ObjUuid: LPCWSTR,
        Protseq: LPCWSTR,
        NetworkAddr: LPCWSTR,
        Endpoint: LPCWSTR,
        Options: LPCWSTR,
        StringBinding: *mut LPWSTR,
    ) -> RPC_STATUS;
    pub fn RpcBindingFromStringBindingW(
        StringBinding: LPCWSTR,
        Binding: *mut RPC_BINDING_HANDLE,
    ) -> RPC_STATUS;
    pub fn RpcBindingSetObject(Binding: RPC_BINDING_HANDLE, ObjectUuid: *mut UUID) -> RPC_STATUS;
    pub fn RpcBindingInqObject(Binding: RPC_BINDING_HANDLE, ObjectUuid: *mut UUID) -> RPC_STATUS;
    pub fn RpcBindingFree(Binding: *mut RPC_BINDING_HANDLE) -> RPC_STATUS;
    pub fn RpcStringFreeW(String: *mut LPWSTR) -> RPC_STATUS;
    pub fn RpcRaiseException(exception: RPC_STATUS) -> !;
    pub fn RpcServerRegisterIfEx(
        IfSpec: RPC_IF_HANDLE,
        MgrTypeUuid: *mut UUID,
        MgrEpv: *mut c_void,
        Flags: c_uint,
        MaxCalls: c_uint,
        IfCallback: *mut c_void,
    ) -> RPC_STATUS;
    pub fn RpcServerUnregisterIf(
        IfSpec: RPC_IF_HANDLE,
        MgrTypeUuid: *mut UUID,
        WaitForCallsToComplete: c_uint,
    ) -> RPC_STATUS;
    pub fn RpcServerUseProtseqEpW(
        Protseq: LPCWSTR,
        MaxCalls: c_uint,
        Endpoint: LPCWSTR,
        SecurityDescriptor: *mut c_void,
    ) -> RPC_STATUS;
    pub fn I_RpcGetBuffer(Message: *mut RPC_MESSAGE) -> RPC_STATUS;
    pub fn I_RpcFreeBuffer(Message: *mut RPC_MESSAGE) -> RPC_STATUS;
    pub fn I_RpcSendReceive(Message: *mut RPC_MESSAGE) -> RPC_STATUS;
}

// ---------------------------------------------------------------------------
// Extern imports — ole32 exported APIs implemented OUTSIDE this file set
// (Lane A compobj.c, or still-C ole32 TUs). WINAPI → extern "system".
// ---------------------------------------------------------------------------
extern "system" {
    // compobj.c (Lane A)
    pub fn CoGetPSClsid(riid: REFIID, pclsid: *mut CLSID) -> HRESULT;
    pub fn CoGetClassObject(
        rclsid: REFCLSID,
        dwClsContext: DWORD,
        pServerInfo: *mut c_void,
        riid: REFIID,
        ppv: *mut *mut c_void,
    ) -> HRESULT;
    pub fn CoCreateInstance(
        rclsid: REFCLSID,
        pUnkOuter: *mut IUnknown,
        dwClsContext: DWORD,
        riid: REFIID,
        ppv: *mut *mut c_void,
    ) -> HRESULT;
    pub fn CoCreateGuid(pguid: *mut GUID) -> HRESULT;
    pub fn StringFromGUID2(rguid: REFGUID, lpsz: LPWSTR, cchMax: c_int) -> c_int;
    pub fn CoWaitForMultipleHandles(
        dwFlags: DWORD,
        dwTimeout: DWORD,
        cHandles: ULONG,
        pHandles: *const HANDLE,
        lpdwindex: *mut DWORD,
    ) -> HRESULT;
    // ifs.c (still C, Tier M)
    pub fn CoTaskMemAlloc(size: SIZE_T) -> PVOID;
    pub fn CoTaskMemFree(ptr: PVOID);
    // hglobalstream.c (still C, Tier M)
    pub fn CreateStreamOnHGlobal(
        hGlobal: *mut c_void,
        fDeleteOnRelease: BOOL,
        ppstm: *mut *mut IStream,
    ) -> HRESULT;
}

// ---------------------------------------------------------------------------
// Extern imports — ole32 internal (cdecl, DECLSPEC_HIDDEN) functions owned by
// Lane A's compobj.c. Listed in MANIFEST.
// ---------------------------------------------------------------------------
extern "C" {
    pub fn apartment_findfromoxid(oxid: OXID, ref_: BOOL) -> *mut apartment;
    pub fn apartment_findfromtid(tid: DWORD) -> *mut apartment;
    pub fn apartment_release(apt: *mut apartment) -> DWORD;
    pub fn apartment_createwindowifneeded(apt: *mut apartment) -> HRESULT;
    pub fn apartment_getwindow(apt: *const apartment) -> HWND;
    pub fn apartment_get_current_or_mta() -> *mut apartment;
    pub fn apartment_get_mta() -> *mut apartment;
    pub fn enter_apartment(info: *mut oletls, model: DWORD) -> HRESULT;
    pub fn leave_apartment(info: *mut oletls);
    pub fn COM_OpenKeyForCLSID(
        clsid: REFCLSID,
        keyname: LPCWSTR,
        access: REGSAM,
        key: *mut HKEY,
    ) -> HRESULT;
    pub fn COM_OpenKeyForAppIdFromCLSID(
        clsid: REFCLSID,
        access: REGSAM,
        subkey: *mut HKEY,
    ) -> HRESULT;
}

// ---------------------------------------------------------------------------
// Cross-file declarations for functions implemented INSIDE this lane's file
// set (each is #[no_mangle] extern in its owning module; declared here so the
// sibling modules can call them without depending on module paths).
// ---------------------------------------------------------------------------
extern "C" {
    // marshal.rs
    pub fn marshal_object(
        apt: *mut apartment,
        stdobjref: *mut STDOBJREF,
        riid: REFIID,
        object: *mut IUnknown,
        dest_context: DWORD,
        dest_context_data: *mut c_void,
        mshlflags: MSHLFLAGS,
    ) -> HRESULT;
    pub fn apartment_disconnectproxies(apt: *mut apartment) -> HRESULT;
    // stubmanager.rs
    pub fn stub_manager_int_release(This: *mut stub_manager) -> ULONG;
    pub fn stub_manager_ext_addref(m: *mut stub_manager, refs: ULONG, tableweak: BOOL) -> ULONG;
    pub fn stub_manager_ext_release(
        m: *mut stub_manager,
        refs: ULONG,
        tableweak: BOOL,
        last_unlock_releases: BOOL,
    ) -> ULONG;
    pub fn stub_manager_new_ifstub(
        m: *mut stub_manager,
        sb: *mut IRpcStubBuffer,
        iid: REFIID,
        dest_context: DWORD,
        dest_context_data: *mut c_void,
        flags: MSHLFLAGS,
    ) -> *mut ifstub;
    pub fn stub_manager_find_ifstub(
        m: *mut stub_manager,
        iid: REFIID,
        flags: MSHLFLAGS,
    ) -> *mut ifstub;
    pub fn get_stub_manager(apt: *mut apartment, oid: OID) -> *mut stub_manager;
    pub fn get_stub_manager_from_object(
        apt: *mut apartment,
        object: *mut IUnknown,
        alloc: BOOL,
    ) -> *mut stub_manager;
    pub fn stub_manager_notify_unmarshal(m: *mut stub_manager, ipid: *const IPID) -> BOOL;
    pub fn stub_manager_is_table_marshaled(m: *mut stub_manager, ipid: *const IPID) -> BOOL;
    pub fn stub_manager_release_marshal_data(
        m: *mut stub_manager,
        refs: ULONG,
        ipid: *const IPID,
        tableweak: BOOL,
    );
    pub fn stub_manager_disconnect(m: *mut stub_manager);
    pub fn ipid_get_dispatch_params(
        ipid: *const IPID,
        stub_apt: *mut *mut apartment,
        manager: *mut *mut stub_manager,
        stub: *mut *mut IRpcStubBuffer,
        chan: *mut *mut IRpcChannelBuffer,
        iid: *mut IID,
        iface: *mut *mut IUnknown,
    ) -> HRESULT;
    pub fn start_apartment_remote_unknown(apt: *mut apartment) -> HRESULT;
    // rpc.rs
    pub fn RPC_StartRemoting(apt: *mut apartment);
    pub fn RPC_CreateClientChannel(
        oxid: *const OXID,
        ipid: *const IPID,
        oxid_info: *const OXID_INFO,
        iid: *const IID,
        dest_context: DWORD,
        dest_context_data: *mut c_void,
        chan: *mut *mut IRpcChannelBuffer,
        apt: *mut apartment,
    ) -> HRESULT;
    pub fn RPC_CreateServerChannel(
        dest_context: DWORD,
        dest_context_data: *mut c_void,
        chan: *mut *mut IRpcChannelBuffer,
    ) -> HRESULT;
    pub fn RPC_ExecuteCall(params: *mut dispatch_params);
    pub fn RPC_RegisterInterface(riid: REFIID) -> HRESULT;
    pub fn RPC_UnregisterInterface(riid: REFIID, wait: BOOL);
    pub fn RPC_ResolveOxid(oxid: OXID, oxid_info: *mut OXID_INFO) -> HRESULT;
    // git.rs
    pub fn get_std_git() -> *mut IGlobalInterfaceTable;
}

// Exported (OLE32.@) functions owned by this lane's marshal.rs, called
// cross-file (rpc.c → CoUnmarshalInterface; git.c → CoMarshalInterface,
// CoUnmarshalInterface, CoReleaseMarshalData; ftmarshal.c →
// CoGetStandardMarshal). WINAPI → extern "system".
extern "system" {
    pub fn CoMarshalInterface(
        pStream: *mut IStream,
        riid: REFIID,
        pUnk: *mut IUnknown,
        dwDestContext: DWORD,
        pvDestContext: *mut c_void,
        mshlFlags: DWORD,
    ) -> HRESULT;
    pub fn CoUnmarshalInterface(pStream: *mut IStream, riid: REFIID, ppv: *mut LPVOID) -> HRESULT;
    pub fn CoReleaseMarshalData(pStream: *mut IStream) -> HRESULT;
    pub fn CoGetStandardMarshal(
        riid: REFIID,
        pUnk: *mut IUnknown,
        dwDestContext: DWORD,
        pvDestContext: LPVOID,
        mshlflags: DWORD,
        ppMarshal: *mut LPMARSHAL,
    ) -> HRESULT;
}

/// rpc.c `struct dispatch_params` — shared with compobj.c's apartment window
/// proc (DM_EXECUTERPC lParam) so the layout is part of the Lane A/B contract.
#[repr(C)]
pub struct dispatch_params {
    pub msg: *mut RPCOLEMESSAGE,        // message
    pub stub: *mut IRpcStubBuffer,      // stub buffer, if applicable
    pub chan: *mut IRpcChannelBuffer,   // server channel buffer, if applicable
    pub iid: IID,                       // ID of interface being called
    pub iface: *mut IUnknown,           // interface being called
    pub handle: HANDLE,                 // handle signaled when call finishes
    pub bypass_rpcrt: BOOL,             // bypass RPC runtime?
    pub status: RPC_STATUS,             // status (out)
    pub hr: HRESULT,                    // hresult (out)
}

// ---------------------------------------------------------------------------
// Compile-time UTF-16 literal (ASCII only), as in advapi32-rust.
// ---------------------------------------------------------------------------
macro_rules! wide {
    ($s:literal) => {{
        const B: &[u8] = $s.as_bytes();
        const N: usize = B.len() + 1;
        const W: [u16; N] = {
            let mut a = [0u16; N];
            let mut i = 0;
            while i < B.len() {
                a[i] = B[i] as u16;
                i += 1;
            }
            a
        };
        &W
    }};
}
pub(crate) use wide;

/// basetsd.h UlongToHandle
#[inline]
pub fn UlongToHandle(h: ULONG) -> HANDLE {
    h as usize as HANDLE
}

// ---------------------------------------------------------------------------
// Compile-time layout tripwires (wire format + struct-cast invariants).
// These mirror what the C's FIELD_OFFSET arithmetic silently assumes.
// ---------------------------------------------------------------------------
const _: () = {
    assert!(core::mem::size_of::<GUID>() == 16);
    assert!(core::mem::size_of::<STDOBJREF>() == 40);
    /* FIELD_OFFSET(OBJREF, u_objref) */
    assert!(OBJREF_HDR_SIZE == 24);
    /* FIELD_OFFSET(OBJREF, u_objref.u_standard.saResAddr.aStringArray) */
    assert!(OBJREF_STANDARD_SIZE == 24 + 40 + 4);
    /* FIELD_OFFSET(struct OR_STANDARD, saResAddr.aStringArray) */
    assert!(OR_STANDARD_SIZE == 40 + 4);
    /* OR_CUSTOM header = clsid + cbExtension + size */
    assert!(OBJREF_CUSTOM_HDR_SIZE - OBJREF_HDR_SIZE == 16 + 4 + 4);
    /* rpc.c casts RPC_MESSAGE* <-> RPCOLEMESSAGE* freely */
    assert!(core::mem::size_of::<RPC_MESSAGE>() == core::mem::size_of::<RPCOLEMESSAGE>());
    /* winnt.h RTL_CRITICAL_SECTION size (24 on 32-bit, 40 on 64-bit) */
    assert!(
        core::mem::size_of::<CRITICAL_SECTION>()
            == 4 * core::mem::size_of::<usize>() + 8
    );
};
