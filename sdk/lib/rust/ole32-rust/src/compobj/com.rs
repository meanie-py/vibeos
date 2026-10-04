//! COM interface/vtbl declarations, IIDs/CLSIDs, HRESULT + COM enum constants
//! used by the compobj.c translation.
//!
//! HAND-DECLARED, kept in this ONE module so the swap to the future bindgen
//! `com-sys` crate is mechanical. Slot order in every vtbl is the COM ABI —
//! verified against sdk/include/psdk/*.idl — NEVER reorder.
//!
//! `extern "system"` = stdcall on i386, C on amd64/arm64 (the one true COM
//! calling convention on all three targets).

#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code)]

use core::ffi::c_void;
use super::ffi::{BOOL, DWORD, HANDLE, HRESULT, LPCWSTR, LPWSTR, ULONG, ULONG_PTR};

// ---------------------------------------------------------------------------
// GUID / IID / CLSID
// ---------------------------------------------------------------------------

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct GUID {
    pub Data1: u32,
    pub Data2: u16,
    pub Data3: u16,
    pub Data4: [u8; 8],
}
pub type IID = GUID;
pub type CLSID = GUID;
pub type REFIID = *const IID;
pub type REFCLSID = *const CLSID;
pub type REFGUID = *const GUID;

/// `IsEqualGUID` inline form (guiddef.h) — the exported ole32 `IsEqualGUID`
/// symbol lives in compobj.rs; internal comparisons use this.
#[inline]
pub unsafe fn guid_eq(a: *const GUID, b: *const GUID) -> bool {
    // memcmp(a, b, sizeof(GUID)) == 0
    let a = &*(a as *const [u8; 16]);
    let b = &*(b as *const [u8; 16]);
    a == b
}

/// `debugstr_guid` equivalent for the ERR!/FIXME! macros.
pub struct DbgGuid(pub *const GUID);
impl core::fmt::Display for DbgGuid {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.0.is_null() {
            return f.write_str("(null)");
        }
        let g = unsafe { &*self.0 };
        write!(
            f,
            "{{{:08x}-{:04x}-{:04x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}}}",
            g.Data1, g.Data2, g.Data3, g.Data4[0], g.Data4[1], g.Data4[2], g.Data4[3],
            g.Data4[4], g.Data4[5], g.Data4[6], g.Data4[7]
        )
    }
}

macro_rules! def_guid {
    ($name:ident, $d1:expr, $d2:expr, $d3:expr, $($d4:expr),+) => {
        pub static $name: GUID = GUID {
            Data1: $d1, Data2: $d2, Data3: $d3, Data4: [$($d4),+],
        };
    };
}

// Values verified against sdk/include/psdk/*.idl and sdk/lib/uuid/cguid_i.c.
def_guid!(GUID_NULL,                    0x00000000, 0x0000, 0x0000, 0,0,0,0,0,0,0,0);
pub static CLSID_NULL: GUID = GUID_NULL;
def_guid!(IID_IUnknown,                 0x00000000, 0x0000, 0x0000, 0xC0,0,0,0,0,0,0,0x46);
def_guid!(IID_IClassFactory,            0x00000001, 0x0000, 0x0000, 0xC0,0,0,0,0,0,0,0x46);
def_guid!(IID_IMarshal,                 0x00000003, 0x0000, 0x0000, 0xC0,0,0,0,0,0,0,0x46);
def_guid!(IID_ISynchronize,             0x00000030, 0x0000, 0x0000, 0xC0,0,0,0,0,0,0,0x46);
def_guid!(IID_ISynchronizeHandle,       0x00000031, 0x0000, 0x0000, 0xC0,0,0,0,0,0,0,0x46);
def_guid!(IID_IInitializeSpy,           0x00000034, 0x0000, 0x0000, 0xC0,0,0,0,0,0,0,0x46);
def_guid!(IID_IPersistStorage,          0x0000010a, 0x0000, 0x0000, 0xC0,0,0,0,0,0,0,0x46);
def_guid!(IID_IPersistFile,             0x0000010b, 0x0000, 0x0000, 0xC0,0,0,0,0,0,0,0x46);
def_guid!(IID_IClientSecurity,          0x0000013D, 0x0000, 0x0000, 0xC0,0,0,0,0,0,0,0x46);
def_guid!(IID_IServerSecurity,          0x0000013E, 0x0000, 0x0000, 0xC0,0,0,0,0,0,0,0x46);
def_guid!(IID_IGlobalOptions,           0x0000015B, 0x0000, 0x0000, 0xC0,0,0,0,0,0,0,0x46);
def_guid!(IID_IObjContext,              0x000001c6, 0x0000, 0x0000, 0xC0,0,0,0,0,0,0,0x46);
def_guid!(IID_IComThreadingInfo,        0x000001ce, 0x0000, 0x0000, 0xC0,0,0,0,0,0,0,0x46);
def_guid!(IID_IContextCallback,         0x000001da, 0x0000, 0x0000, 0xC0,0,0,0,0,0,0,0x46);
def_guid!(IID_IServiceProvider,         0x6d5140c1, 0x7436, 0x11ce, 0x80,0x34,0x00,0xaa,0x00,0x60,0x09,0xfa);
def_guid!(CLSID_StdGlobalInterfaceTable,0x00000323, 0x0000, 0x0000, 0xC0,0,0,0,0,0,0,0x46);
def_guid!(CLSID_ManualResetEvent,       0x0000032c, 0x0000, 0x0000, 0xC0,0,0,0,0,0,0,0x46);
def_guid!(CLSID_InProcFreeMarshaler,    0x0000033a, 0x0000, 0x0000, 0xC0,0,0,0,0,0,0,0x46);
def_guid!(CLSID_GlobalOptions,          0x0000034b, 0x0000, 0x0000, 0xC0,0,0,0,0,0,0,0x46);

// ---------------------------------------------------------------------------
// HRESULT values (winerror.h)
// ---------------------------------------------------------------------------

pub const S_OK: HRESULT = 0;
pub const NOERROR: HRESULT = 0;
pub const S_FALSE: HRESULT = 1;
pub const E_NOTIMPL: HRESULT = 0x8000_4001u32 as HRESULT;
pub const E_NOINTERFACE: HRESULT = 0x8000_4002u32 as HRESULT;
pub const E_POINTER: HRESULT = 0x8000_4003u32 as HRESULT;
pub const E_FAIL: HRESULT = 0x8000_4005u32 as HRESULT;
pub const E_UNEXPECTED: HRESULT = 0x8000_FFFFu32 as HRESULT;
pub const E_ACCESSDENIED: HRESULT = 0x8007_0005u32 as HRESULT;
pub const E_OUTOFMEMORY: HRESULT = 0x8007_000Eu32 as HRESULT;
pub const E_INVALIDARG: HRESULT = 0x8007_0057u32 as HRESULT;

pub const CO_S_NOTALLINTERFACES: HRESULT = 0x0008_0012;

pub const REGDB_E_READREGDB: HRESULT = 0x8004_0150u32 as HRESULT;
pub const REGDB_E_WRITEREGDB: HRESULT = 0x8004_0151u32 as HRESULT;
pub const REGDB_E_KEYMISSING: HRESULT = 0x8004_0152u32 as HRESULT;
pub const REGDB_E_CLASSNOTREG: HRESULT = 0x8004_0154u32 as HRESULT;
pub const REGDB_E_IIDNOTREG: HRESULT = 0x8004_0155u32 as HRESULT;

pub const CLASS_E_NOAGGREGATION: HRESULT = 0x8004_0110u32 as HRESULT;
pub const CLASS_E_CLASSNOTAVAILABLE: HRESULT = 0x8004_0111u32 as HRESULT;

pub const CO_E_CLASSSTRING: HRESULT = 0x8004_01F3u32 as HRESULT;
pub const CO_E_IIDSTRING: HRESULT = 0x8004_01F4u32 as HRESULT;
pub const CO_E_DLLNOTFOUND: HRESULT = 0x8004_01F8u32 as HRESULT;
pub const CO_E_OBJISREG: HRESULT = 0x8004_01FCu32 as HRESULT;
pub const CO_E_NOTINITIALIZED: HRESULT = 0x8004_01F0u32 as HRESULT;
pub const CO_E_NOT_SUPPORTED: HRESULT = 0x8000_4021u32 as HRESULT;

pub const RPC_E_CALL_CANCELED: HRESULT = 0x8001_0002u32 as HRESULT;
pub const RPC_E_CHANGED_MODE: HRESULT = 0x8001_0106u32 as HRESULT;
pub const RPC_E_WRONG_THREAD: HRESULT = 0x8001_010Eu32 as HRESULT;
pub const RPC_E_NO_SYNC: HRESULT = 0x8001_0120u32 as HRESULT;
pub const RPC_S_CALLPENDING: HRESULT = 0x8001_0115u32 as HRESULT;
pub const RPC_E_CALL_COMPLETE: HRESULT = 0x8001_0117u32 as HRESULT;

#[inline]
pub fn SUCCEEDED(hr: HRESULT) -> bool { hr >= 0 }
#[inline]
pub fn FAILED(hr: HRESULT) -> bool { hr < 0 }

// ---------------------------------------------------------------------------
// COM enums / flags used by compobj.c
// ---------------------------------------------------------------------------

// CLSCTX (wtypesbase.idl)
pub const CLSCTX_INPROC_SERVER: DWORD = 0x1;
pub const CLSCTX_INPROC_HANDLER: DWORD = 0x2;
pub const CLSCTX_LOCAL_SERVER: DWORD = 0x4;
pub const CLSCTX_INPROC_SERVER16: DWORD = 0x8;
pub const CLSCTX_REMOTE_SERVER: DWORD = 0x10;
pub const CLSCTX_INPROC: DWORD = CLSCTX_INPROC_SERVER | CLSCTX_INPROC_HANDLER;
pub const CLSCTX_APPCONTAINER: DWORD = 0x40_0000;
/// compobj_private.h wine-internal flag.
pub const WINE_CLSCTX_DONT_HOST: DWORD = 0x8000_0000;

// COINIT (objbase.h). COINIT_MULTITHREADED is 0 — never test against it.
pub const COINIT_APARTMENTTHREADED: DWORD = 0x2;
pub const COINIT_MULTITHREADED: DWORD = 0x0;

// REGCLS (objbase.h)
pub const REGCLS_SINGLEUSE: DWORD = 0;
pub const REGCLS_MULTIPLEUSE: DWORD = 1;
pub const REGCLS_MULTI_SEPARATE: DWORD = 2;
pub const REGCLS_SUSPENDED: DWORD = 4;

// MSHCTX / MSHLFLAGS (wtypesbase.idl)
pub const MSHCTX_LOCAL: DWORD = 0;
pub const MSHCTX_INPROC: DWORD = 3;
pub const MSHLFLAGS_NORMAL: DWORD = 0;
pub const MSHLFLAGS_TABLESTRONG: DWORD = 1;
pub const MSHLFLAGS_TABLEWEAK: DWORD = 2;

// STREAM_SEEK / STATFLAG (objidlbase.idl)
pub const STREAM_SEEK_SET: DWORD = 0;
pub const STATFLAG_NONAME: DWORD = 1;

// APTTYPE / APTTYPEQUALIFIER / THDTYPE (objidlbase.idl)
pub type APTTYPE = i32;
pub const APTTYPE_CURRENT: APTTYPE = -1;
pub const APTTYPE_STA: APTTYPE = 0;
pub const APTTYPE_MTA: APTTYPE = 1;
pub const APTTYPE_NA: APTTYPE = 2;
pub const APTTYPE_MAINSTA: APTTYPE = 3;
pub type APTTYPEQUALIFIER = i32;
pub const APTTYPEQUALIFIER_NONE: APTTYPEQUALIFIER = 0;
pub const APTTYPEQUALIFIER_IMPLICIT_MTA: APTTYPEQUALIFIER = 1;
pub type THDTYPE = i32;
pub const THDTYPE_BLOCKMESSAGES: THDTYPE = 0;
pub const THDTYPE_PROCESSMESSAGES: THDTYPE = 1;

// COWAIT (objbase.h)
pub const COWAIT_WAITALL: DWORD = 1;
pub const COWAIT_ALERTABLE: DWORD = 2;

// PENDINGTYPE / PENDINGMSG (objidl.idl)
pub const PENDINGTYPE_TOPLEVEL: DWORD = 1;
pub const PENDINGTYPE_NESTED: DWORD = 2;
pub const PENDINGMSG_CANCELCALL: DWORD = 0;
pub const PENDINGMSG_WAITNOPROCESS: DWORD = 1;
pub const PENDINGMSG_WAITDEFPROCESS: DWORD = 2;

// DVASPECT (wtypes.idl)
pub const DVASPECT_CONTENT: DWORD = 1;
pub const DVASPECT_THUMBNAIL: DWORD = 2;
pub const DVASPECT_ICON: DWORD = 4;
pub const DVASPECT_DOCPRINT: DWORD = 8;

// GLOBALOPT (objidlbase.idl) — only passed through, opaque value.
pub type GLOBALOPT_PROPERTIES = u32;

/// CHARS_IN_GUID (compobj_private.h) — includes the NUL.
pub const CHARS_IN_GUID: usize = 39;

// ---------------------------------------------------------------------------
// OXID / OID / IPID (dcom.idl)
// ---------------------------------------------------------------------------

pub type OXID = u64;
pub type OID = u64;
pub type IPID = GUID;

// ---------------------------------------------------------------------------
// Small COM-adjacent structs
// ---------------------------------------------------------------------------

/// MULTI_QI (objidlbase.idl)
#[repr(C)]
pub struct MULTI_QI {
    pub pIID: *const IID,
    pub pItf: *mut IUnknown,
    pub hr: HRESULT,
}

/// COSERVERINFO (objidlbase.idl) — only pwszName/pAuthInfo are ever read here.
#[repr(C)]
pub struct COSERVERINFO {
    pub dwReserved1: DWORD,
    pub pwszName: LPWSTR,
    pub pAuthInfo: *mut c_void, // COAUTHINFO*
    pub dwReserved2: DWORD,
}

/// BIND_OPTS (objidl.idl)
#[repr(C)]
pub struct BIND_OPTS {
    pub cbStruct: DWORD,
    pub grfFlags: DWORD,
    pub grfMode: DWORD,
    pub dwTickCountDeadline: DWORD,
}

/// STATSTG (objidlbase.idl)
#[repr(C)]
pub struct STATSTG {
    pub pwcsName: LPWSTR,
    pub type_: DWORD,
    pub cbSize: u64,
    pub mtime: super::ffi::FILETIME,
    pub ctime: super::ffi::FILETIME,
    pub atime: super::ffi::FILETIME,
    pub grfMode: DWORD,
    pub grfLocksSupported: DWORD,
    pub clsid: CLSID,
    pub grfStateBits: DWORD,
    pub reserved: DWORD,
}

/// SOLE_AUTHENTICATION_SERVICE — opaque here (CoInitializeSecurity stub).
pub type SOLE_AUTHENTICATION_SERVICE = c_void;
/// RPC_AUTHZ_HANDLE (rpcdce.h)
pub type RPC_AUTHZ_HANDLE = *mut c_void;
/// ComCallData / PFNCONTEXTCALL (ctxtcall.idl) — opaque pass-through.
pub type ComCallData = c_void;
pub type PFNCONTEXTCALL = Option<unsafe extern "system" fn(*mut ComCallData) -> HRESULT>;

// ---------------------------------------------------------------------------
// IUnknown
// ---------------------------------------------------------------------------

#[repr(C)]
pub struct IUnknownVtbl {
    pub QueryInterface:
        unsafe extern "system" fn(*mut IUnknown, REFIID, *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(*mut IUnknown) -> ULONG,
    pub Release: unsafe extern "system" fn(*mut IUnknown) -> ULONG,
}
#[repr(C)]
pub struct IUnknown {
    pub lpVtbl: *const IUnknownVtbl,
}

/// COBJMACROS-shaped call helpers.
#[inline]
pub unsafe fn IUnknown_QueryInterface(
    p: *mut IUnknown,
    riid: REFIID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    ((*(*p).lpVtbl).QueryInterface)(p, riid, ppv)
}
#[inline]
pub unsafe fn IUnknown_AddRef(p: *mut IUnknown) -> ULONG {
    ((*(*p).lpVtbl).AddRef)(p)
}
#[inline]
pub unsafe fn IUnknown_Release(p: *mut IUnknown) -> ULONG {
    ((*(*p).lpVtbl).Release)(p)
}

// Interfaces where compobj.c only ever uses IUnknown methods on the pointer:
// keep them layout-identical aliases (the pointee is lpVtbl-first regardless).
pub type IErrorInfo = IUnknown;
pub type IEnumContextProps = IUnknown;
pub type IGlobalInterfaceTable = IUnknown;
pub type ISurrogate = IUnknown;
pub type IChannelHook = IUnknown;

// ---------------------------------------------------------------------------
// IClassFactory
// ---------------------------------------------------------------------------

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
#[repr(C)]
pub struct IClassFactory {
    pub lpVtbl: *const IClassFactoryVtbl,
}

// ---------------------------------------------------------------------------
// IStream : ISequentialStream : IUnknown (objidlbase.idl) — 14 slots
// ---------------------------------------------------------------------------

#[repr(C)]
pub struct IStreamVtbl {
    pub QueryInterface:
        unsafe extern "system" fn(*mut IStream, REFIID, *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(*mut IStream) -> ULONG,
    pub Release: unsafe extern "system" fn(*mut IStream) -> ULONG,
    pub Read: unsafe extern "system" fn(*mut IStream, *mut c_void, ULONG, *mut ULONG) -> HRESULT,
    pub Write:
        unsafe extern "system" fn(*mut IStream, *const c_void, ULONG, *mut ULONG) -> HRESULT,
    pub Seek: unsafe extern "system" fn(*mut IStream, i64, DWORD, *mut u64) -> HRESULT,
    pub SetSize: unsafe extern "system" fn(*mut IStream, u64) -> HRESULT,
    pub CopyTo:
        unsafe extern "system" fn(*mut IStream, *mut IStream, u64, *mut u64, *mut u64) -> HRESULT,
    pub Commit: unsafe extern "system" fn(*mut IStream, DWORD) -> HRESULT,
    pub Revert: unsafe extern "system" fn(*mut IStream) -> HRESULT,
    pub LockRegion: unsafe extern "system" fn(*mut IStream, u64, u64, DWORD) -> HRESULT,
    pub UnlockRegion: unsafe extern "system" fn(*mut IStream, u64, u64, DWORD) -> HRESULT,
    pub Stat: unsafe extern "system" fn(*mut IStream, *mut STATSTG, DWORD) -> HRESULT,
    pub Clone: unsafe extern "system" fn(*mut IStream, *mut *mut IStream) -> HRESULT,
}
#[repr(C)]
pub struct IStream {
    pub lpVtbl: *const IStreamVtbl,
}

// ---------------------------------------------------------------------------
// IMarshal (objidlbase.idl) — 9 slots; only DisconnectObject is called here.
// ---------------------------------------------------------------------------

#[repr(C)]
pub struct IMarshalVtbl {
    pub QueryInterface:
        unsafe extern "system" fn(*mut IMarshal, REFIID, *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(*mut IMarshal) -> ULONG,
    pub Release: unsafe extern "system" fn(*mut IMarshal) -> ULONG,
    pub GetUnmarshalClass: unsafe extern "system" fn(
        *mut IMarshal, REFIID, *mut c_void, DWORD, *mut c_void, DWORD, *mut CLSID,
    ) -> HRESULT,
    pub GetMarshalSizeMax: unsafe extern "system" fn(
        *mut IMarshal, REFIID, *mut c_void, DWORD, *mut c_void, DWORD, *mut DWORD,
    ) -> HRESULT,
    pub MarshalInterface: unsafe extern "system" fn(
        *mut IMarshal, *mut IStream, REFIID, *mut c_void, DWORD, *mut c_void, DWORD,
    ) -> HRESULT,
    pub UnmarshalInterface:
        unsafe extern "system" fn(*mut IMarshal, *mut IStream, REFIID, *mut *mut c_void) -> HRESULT,
    pub ReleaseMarshalData: unsafe extern "system" fn(*mut IMarshal, *mut IStream) -> HRESULT,
    pub DisconnectObject: unsafe extern "system" fn(*mut IMarshal, DWORD) -> HRESULT,
}
#[repr(C)]
pub struct IMarshal {
    pub lpVtbl: *const IMarshalVtbl,
}

// ---------------------------------------------------------------------------
// IMessageFilter (objidl.idl)
// ---------------------------------------------------------------------------

#[repr(C)]
pub struct IMessageFilterVtbl {
    pub QueryInterface:
        unsafe extern "system" fn(*mut IMessageFilter, REFIID, *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(*mut IMessageFilter) -> ULONG,
    pub Release: unsafe extern "system" fn(*mut IMessageFilter) -> ULONG,
    /// HandleInComingCall(DWORD dwCallType, HTASK, DWORD dwTickCount, LPINTERFACEINFO)
    pub HandleInComingCall:
        unsafe extern "system" fn(*mut IMessageFilter, DWORD, HANDLE, DWORD, *mut c_void) -> DWORD,
    /// RetryRejectedCall(HTASK, DWORD dwTickCount, DWORD dwRejectType)
    pub RetryRejectedCall:
        unsafe extern "system" fn(*mut IMessageFilter, HANDLE, DWORD, DWORD) -> DWORD,
    /// MessagePending(HTASK, DWORD dwTickCount, DWORD dwPendingType)
    pub MessagePending:
        unsafe extern "system" fn(*mut IMessageFilter, HANDLE, DWORD, DWORD) -> DWORD,
}
#[repr(C)]
pub struct IMessageFilter {
    pub lpVtbl: *const IMessageFilterVtbl,
}
pub type LPMESSAGEFILTER = *mut IMessageFilter;

// ---------------------------------------------------------------------------
// IInitializeSpy (objidl.idl)
// ---------------------------------------------------------------------------

#[repr(C)]
pub struct IInitializeSpyVtbl {
    pub QueryInterface:
        unsafe extern "system" fn(*mut IInitializeSpy, REFIID, *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(*mut IInitializeSpy) -> ULONG,
    pub Release: unsafe extern "system" fn(*mut IInitializeSpy) -> ULONG,
    pub PreInitialize: unsafe extern "system" fn(*mut IInitializeSpy, DWORD, DWORD) -> HRESULT,
    pub PostInitialize:
        unsafe extern "system" fn(*mut IInitializeSpy, HRESULT, DWORD, DWORD) -> HRESULT,
    pub PreUninitialize: unsafe extern "system" fn(*mut IInitializeSpy, DWORD) -> HRESULT,
    pub PostUninitialize: unsafe extern "system" fn(*mut IInitializeSpy, DWORD) -> HRESULT,
}
#[repr(C)]
pub struct IInitializeSpy {
    pub lpVtbl: *const IInitializeSpyVtbl,
}

// ---------------------------------------------------------------------------
// ISynchronize / ISynchronizeHandle (objidlbase.idl) — IMPLEMENTED here
// ---------------------------------------------------------------------------

#[repr(C)]
pub struct ISynchronizeVtbl {
    pub QueryInterface:
        unsafe extern "system" fn(*mut ISynchronize, REFIID, *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(*mut ISynchronize) -> ULONG,
    pub Release: unsafe extern "system" fn(*mut ISynchronize) -> ULONG,
    pub Wait: unsafe extern "system" fn(*mut ISynchronize, DWORD, DWORD) -> HRESULT,
    pub Signal: unsafe extern "system" fn(*mut ISynchronize) -> HRESULT,
    pub Reset: unsafe extern "system" fn(*mut ISynchronize) -> HRESULT,
}
#[repr(C)]
pub struct ISynchronize {
    pub lpVtbl: *const ISynchronizeVtbl,
}

#[repr(C)]
pub struct ISynchronizeHandleVtbl {
    pub QueryInterface:
        unsafe extern "system" fn(*mut ISynchronizeHandle, REFIID, *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(*mut ISynchronizeHandle) -> ULONG,
    pub Release: unsafe extern "system" fn(*mut ISynchronizeHandle) -> ULONG,
    pub GetHandle: unsafe extern "system" fn(*mut ISynchronizeHandle, *mut HANDLE) -> HRESULT,
}
#[repr(C)]
pub struct ISynchronizeHandle {
    pub lpVtbl: *const ISynchronizeHandleVtbl,
}

// ---------------------------------------------------------------------------
// IServiceProvider (servprov.idl) — IMPLEMENTED here (LocalServer)
// ---------------------------------------------------------------------------

#[repr(C)]
pub struct IServiceProviderVtbl {
    pub QueryInterface:
        unsafe extern "system" fn(*mut IServiceProvider, REFIID, *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(*mut IServiceProvider) -> ULONG,
    pub Release: unsafe extern "system" fn(*mut IServiceProvider) -> ULONG,
    pub QueryService:
        unsafe extern "system" fn(*mut IServiceProvider, REFGUID, REFIID, *mut *mut c_void)
            -> HRESULT,
}
#[repr(C)]
pub struct IServiceProvider {
    pub lpVtbl: *const IServiceProviderVtbl,
}

// ---------------------------------------------------------------------------
// IComThreadingInfo / IContextCallback / IObjContext — IMPLEMENTED here
// (the thread Context object)
// ---------------------------------------------------------------------------

#[repr(C)]
pub struct IComThreadingInfoVtbl {
    pub QueryInterface:
        unsafe extern "system" fn(*mut IComThreadingInfo, REFIID, *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(*mut IComThreadingInfo) -> ULONG,
    pub Release: unsafe extern "system" fn(*mut IComThreadingInfo) -> ULONG,
    pub GetCurrentApartmentType:
        unsafe extern "system" fn(*mut IComThreadingInfo, *mut APTTYPE) -> HRESULT,
    pub GetCurrentThreadType:
        unsafe extern "system" fn(*mut IComThreadingInfo, *mut THDTYPE) -> HRESULT,
    pub GetCurrentLogicalThreadId:
        unsafe extern "system" fn(*mut IComThreadingInfo, *mut GUID) -> HRESULT,
    pub SetCurrentLogicalThreadId:
        unsafe extern "system" fn(*mut IComThreadingInfo, REFGUID) -> HRESULT,
}
#[repr(C)]
pub struct IComThreadingInfo {
    pub lpVtbl: *const IComThreadingInfoVtbl,
}

#[repr(C)]
pub struct IContextCallbackVtbl {
    pub QueryInterface:
        unsafe extern "system" fn(*mut IContextCallback, REFIID, *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(*mut IContextCallback) -> ULONG,
    pub Release: unsafe extern "system" fn(*mut IContextCallback) -> ULONG,
    pub ContextCallback: unsafe extern "system" fn(
        *mut IContextCallback,
        PFNCONTEXTCALL,
        *mut ComCallData,
        REFIID,
        i32,
        *mut IUnknown,
    ) -> HRESULT,
}
#[repr(C)]
pub struct IContextCallback {
    pub lpVtbl: *const IContextCallbackVtbl,
}

#[repr(C)]
pub struct IObjContextVtbl {
    pub QueryInterface:
        unsafe extern "system" fn(*mut IObjContext, REFIID, *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(*mut IObjContext) -> ULONG,
    pub Release: unsafe extern "system" fn(*mut IObjContext) -> ULONG,
    pub SetProperty:
        unsafe extern "system" fn(*mut IObjContext, REFGUID, ULONG, *mut IUnknown) -> HRESULT,
    pub RemoveProperty: unsafe extern "system" fn(*mut IObjContext, REFGUID) -> HRESULT,
    pub GetProperty: unsafe extern "system" fn(
        *mut IObjContext,
        REFGUID,
        *mut ULONG,
        *mut *mut IUnknown,
    ) -> HRESULT,
    pub EnumContextProps:
        unsafe extern "system" fn(*mut IObjContext, *mut *mut IEnumContextProps) -> HRESULT,
    pub Reserved1: unsafe extern "system" fn(*mut IObjContext),
    pub Reserved2: unsafe extern "system" fn(*mut IObjContext),
    pub Reserved3: unsafe extern "system" fn(*mut IObjContext),
    pub Reserved4: unsafe extern "system" fn(*mut IObjContext),
    pub Reserved5: unsafe extern "system" fn(*mut IObjContext),
    pub Reserved6: unsafe extern "system" fn(*mut IObjContext),
    pub Reserved7: unsafe extern "system" fn(*mut IObjContext),
}
#[repr(C)]
pub struct IObjContext {
    pub lpVtbl: *const IObjContextVtbl,
}

// ---------------------------------------------------------------------------
// IGlobalOptions (objidlbase.idl) — IMPLEMENTED here
// ---------------------------------------------------------------------------

#[repr(C)]
pub struct IGlobalOptionsVtbl {
    pub QueryInterface:
        unsafe extern "system" fn(*mut IGlobalOptions, REFIID, *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(*mut IGlobalOptions) -> ULONG,
    pub Release: unsafe extern "system" fn(*mut IGlobalOptions) -> ULONG,
    pub Set: unsafe extern "system" fn(*mut IGlobalOptions, GLOBALOPT_PROPERTIES, ULONG_PTR)
        -> HRESULT,
    pub Query:
        unsafe extern "system" fn(*mut IGlobalOptions, GLOBALOPT_PROPERTIES, *mut ULONG_PTR)
            -> HRESULT,
}
#[repr(C)]
pub struct IGlobalOptions {
    pub lpVtbl: *const IGlobalOptionsVtbl,
}

// ---------------------------------------------------------------------------
// IClientSecurity / IServerSecurity (objidlbase.idl) — consumed
// ---------------------------------------------------------------------------

#[repr(C)]
pub struct IClientSecurityVtbl {
    pub QueryInterface:
        unsafe extern "system" fn(*mut IClientSecurity, REFIID, *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(*mut IClientSecurity) -> ULONG,
    pub Release: unsafe extern "system" fn(*mut IClientSecurity) -> ULONG,
    pub QueryBlanket: unsafe extern "system" fn(
        *mut IClientSecurity, *mut IUnknown, *mut DWORD, *mut DWORD, *mut LPWSTR,
        *mut DWORD, *mut DWORD, *mut *mut c_void, *mut DWORD,
    ) -> HRESULT,
    pub SetBlanket: unsafe extern "system" fn(
        *mut IClientSecurity, *mut IUnknown, DWORD, DWORD, LPCWSTR, DWORD, DWORD,
        *mut c_void, DWORD,
    ) -> HRESULT,
    pub CopyProxy:
        unsafe extern "system" fn(*mut IClientSecurity, *mut IUnknown, *mut *mut IUnknown)
            -> HRESULT,
}
#[repr(C)]
pub struct IClientSecurity {
    pub lpVtbl: *const IClientSecurityVtbl,
}

#[repr(C)]
pub struct IServerSecurityVtbl {
    pub QueryInterface:
        unsafe extern "system" fn(*mut IServerSecurity, REFIID, *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(*mut IServerSecurity) -> ULONG,
    pub Release: unsafe extern "system" fn(*mut IServerSecurity) -> ULONG,
    pub QueryBlanket: unsafe extern "system" fn(
        *mut IServerSecurity, *mut DWORD, *mut DWORD, *mut LPWSTR, *mut DWORD, *mut DWORD,
        *mut RPC_AUTHZ_HANDLE, *mut DWORD,
    ) -> HRESULT,
    pub ImpersonateClient: unsafe extern "system" fn(*mut IServerSecurity) -> HRESULT,
    pub RevertToSelf: unsafe extern "system" fn(*mut IServerSecurity) -> HRESULT,
    pub IsImpersonating: unsafe extern "system" fn(*mut IServerSecurity) -> BOOL,
}
#[repr(C)]
pub struct IServerSecurity {
    pub lpVtbl: *const IServerSecurityVtbl,
}

// ---------------------------------------------------------------------------
// IPersistFile / IPersistStorage / IStorage — consumed (Load / Stat only)
// ---------------------------------------------------------------------------

#[repr(C)]
pub struct IPersistFileVtbl {
    pub QueryInterface:
        unsafe extern "system" fn(*mut IPersistFile, REFIID, *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(*mut IPersistFile) -> ULONG,
    pub Release: unsafe extern "system" fn(*mut IPersistFile) -> ULONG,
    // IPersist
    pub GetClassID: unsafe extern "system" fn(*mut IPersistFile, *mut CLSID) -> HRESULT,
    // IPersistFile
    pub IsDirty: unsafe extern "system" fn(*mut IPersistFile) -> HRESULT,
    pub Load: unsafe extern "system" fn(*mut IPersistFile, LPCWSTR, DWORD) -> HRESULT,
    pub Save: unsafe extern "system" fn(*mut IPersistFile, LPCWSTR, BOOL) -> HRESULT,
    pub SaveCompleted: unsafe extern "system" fn(*mut IPersistFile, LPCWSTR) -> HRESULT,
    pub GetCurFile: unsafe extern "system" fn(*mut IPersistFile, *mut LPWSTR) -> HRESULT,
}
#[repr(C)]
pub struct IPersistFile {
    pub lpVtbl: *const IPersistFileVtbl,
}

#[repr(C)]
pub struct IPersistStorageVtbl {
    pub QueryInterface:
        unsafe extern "system" fn(*mut IPersistStorage, REFIID, *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(*mut IPersistStorage) -> ULONG,
    pub Release: unsafe extern "system" fn(*mut IPersistStorage) -> ULONG,
    // IPersist
    pub GetClassID: unsafe extern "system" fn(*mut IPersistStorage, *mut CLSID) -> HRESULT,
    // IPersistStorage
    pub IsDirty: unsafe extern "system" fn(*mut IPersistStorage) -> HRESULT,
    pub InitNew: unsafe extern "system" fn(*mut IPersistStorage, *mut IStorage) -> HRESULT,
    pub Load: unsafe extern "system" fn(*mut IPersistStorage, *mut IStorage) -> HRESULT,
    pub Save: unsafe extern "system" fn(*mut IPersistStorage, *mut IStorage, BOOL) -> HRESULT,
    pub SaveCompleted: unsafe extern "system" fn(*mut IPersistStorage, *mut IStorage) -> HRESULT,
    pub HandsOffStorage: unsafe extern "system" fn(*mut IPersistStorage) -> HRESULT,
}
#[repr(C)]
pub struct IPersistStorage {
    pub lpVtbl: *const IPersistStorageVtbl,
}

/// IStorage (objidl.idl) — 18 slots; only Stat (slot 17) is called here, but
/// the full slot list is spelled out so the offset is ABI-exact.
#[repr(C)]
pub struct IStorageVtbl {
    pub QueryInterface:
        unsafe extern "system" fn(*mut IStorage, REFIID, *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(*mut IStorage) -> ULONG,
    pub Release: unsafe extern "system" fn(*mut IStorage) -> ULONG,
    pub CreateStream: unsafe extern "system" fn(
        *mut IStorage, LPCWSTR, DWORD, DWORD, DWORD, *mut *mut IStream,
    ) -> HRESULT,
    pub OpenStream: unsafe extern "system" fn(
        *mut IStorage, LPCWSTR, *mut c_void, DWORD, DWORD, *mut *mut IStream,
    ) -> HRESULT,
    pub CreateStorage: unsafe extern "system" fn(
        *mut IStorage, LPCWSTR, DWORD, DWORD, DWORD, *mut *mut IStorage,
    ) -> HRESULT,
    pub OpenStorage: unsafe extern "system" fn(
        *mut IStorage, LPCWSTR, *mut IStorage, DWORD, *mut *mut u16, DWORD, *mut *mut IStorage,
    ) -> HRESULT,
    pub CopyTo: unsafe extern "system" fn(
        *mut IStorage, DWORD, *const IID, *mut *mut u16, *mut IStorage,
    ) -> HRESULT,
    pub MoveElementTo: unsafe extern "system" fn(
        *mut IStorage, LPCWSTR, *mut IStorage, LPCWSTR, DWORD,
    ) -> HRESULT,
    pub Commit: unsafe extern "system" fn(*mut IStorage, DWORD) -> HRESULT,
    pub Revert: unsafe extern "system" fn(*mut IStorage) -> HRESULT,
    pub EnumElements: unsafe extern "system" fn(
        *mut IStorage, DWORD, *mut c_void, DWORD, *mut *mut c_void,
    ) -> HRESULT,
    pub DestroyElement: unsafe extern "system" fn(*mut IStorage, LPCWSTR) -> HRESULT,
    pub RenameElement: unsafe extern "system" fn(*mut IStorage, LPCWSTR, LPCWSTR) -> HRESULT,
    pub SetElementTimes: unsafe extern "system" fn(
        *mut IStorage, LPCWSTR, *const super::ffi::FILETIME, *const super::ffi::FILETIME,
        *const super::ffi::FILETIME,
    ) -> HRESULT,
    pub SetClass: unsafe extern "system" fn(*mut IStorage, REFCLSID) -> HRESULT,
    pub SetStateBits: unsafe extern "system" fn(*mut IStorage, DWORD, DWORD) -> HRESULT,
    pub Stat: unsafe extern "system" fn(*mut IStorage, *mut STATSTG, DWORD) -> HRESULT,
}
#[repr(C)]
pub struct IStorage {
    pub lpVtbl: *const IStorageVtbl,
}

// ---------------------------------------------------------------------------
// IBindCtx (objidl.idl) — 13 slots; SetBindOptions + Release used.
// ---------------------------------------------------------------------------

#[repr(C)]
pub struct IBindCtxVtbl {
    pub QueryInterface:
        unsafe extern "system" fn(*mut IBindCtx, REFIID, *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(*mut IBindCtx) -> ULONG,
    pub Release: unsafe extern "system" fn(*mut IBindCtx) -> ULONG,
    pub RegisterObjectBound: unsafe extern "system" fn(*mut IBindCtx, *mut IUnknown) -> HRESULT,
    pub RevokeObjectBound: unsafe extern "system" fn(*mut IBindCtx, *mut IUnknown) -> HRESULT,
    pub ReleaseBoundObjects: unsafe extern "system" fn(*mut IBindCtx) -> HRESULT,
    pub SetBindOptions: unsafe extern "system" fn(*mut IBindCtx, *mut BIND_OPTS) -> HRESULT,
    pub GetBindOptions: unsafe extern "system" fn(*mut IBindCtx, *mut BIND_OPTS) -> HRESULT,
    pub GetRunningObjectTable:
        unsafe extern "system" fn(*mut IBindCtx, *mut *mut c_void) -> HRESULT,
    pub RegisterObjectParam:
        unsafe extern "system" fn(*mut IBindCtx, LPCWSTR, *mut IUnknown) -> HRESULT,
    pub GetObjectParam:
        unsafe extern "system" fn(*mut IBindCtx, LPCWSTR, *mut *mut IUnknown) -> HRESULT,
    pub EnumObjectParam: unsafe extern "system" fn(*mut IBindCtx, *mut *mut c_void) -> HRESULT,
    pub RevokeObjectParam: unsafe extern "system" fn(*mut IBindCtx, LPCWSTR) -> HRESULT,
}
#[repr(C)]
pub struct IBindCtx {
    pub lpVtbl: *const IBindCtxVtbl,
}

// ---------------------------------------------------------------------------
// IMoniker : IPersistStream : IPersist : IUnknown (objidl.idl) — 23 slots;
// only BindToObject (slot 8) + Release are called here.
// ---------------------------------------------------------------------------

#[repr(C)]
pub struct IMonikerVtbl {
    pub QueryInterface:
        unsafe extern "system" fn(*mut IMoniker, REFIID, *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(*mut IMoniker) -> ULONG,
    pub Release: unsafe extern "system" fn(*mut IMoniker) -> ULONG,
    // IPersist
    pub GetClassID: unsafe extern "system" fn(*mut IMoniker, *mut CLSID) -> HRESULT,
    // IPersistStream
    pub IsDirty: unsafe extern "system" fn(*mut IMoniker) -> HRESULT,
    pub Load: unsafe extern "system" fn(*mut IMoniker, *mut IStream) -> HRESULT,
    pub Save: unsafe extern "system" fn(*mut IMoniker, *mut IStream, BOOL) -> HRESULT,
    pub GetSizeMax: unsafe extern "system" fn(*mut IMoniker, *mut u64) -> HRESULT,
    // IMoniker
    pub BindToObject: unsafe extern "system" fn(
        *mut IMoniker, *mut IBindCtx, *mut IMoniker, REFIID, *mut *mut c_void,
    ) -> HRESULT,
    pub BindToStorage: unsafe extern "system" fn(
        *mut IMoniker, *mut IBindCtx, *mut IMoniker, REFIID, *mut *mut c_void,
    ) -> HRESULT,
    pub Reduce: unsafe extern "system" fn(
        *mut IMoniker, *mut IBindCtx, DWORD, *mut *mut IMoniker, *mut *mut IMoniker,
    ) -> HRESULT,
    pub ComposeWith: unsafe extern "system" fn(
        *mut IMoniker, *mut IMoniker, BOOL, *mut *mut IMoniker,
    ) -> HRESULT,
    pub Enum: unsafe extern "system" fn(*mut IMoniker, BOOL, *mut *mut c_void) -> HRESULT,
    pub IsEqual: unsafe extern "system" fn(*mut IMoniker, *mut IMoniker) -> HRESULT,
    pub Hash: unsafe extern "system" fn(*mut IMoniker, *mut DWORD) -> HRESULT,
    pub IsRunning: unsafe extern "system" fn(
        *mut IMoniker, *mut IBindCtx, *mut IMoniker, *mut IMoniker,
    ) -> HRESULT,
    pub GetTimeOfLastChange: unsafe extern "system" fn(
        *mut IMoniker, *mut IBindCtx, *mut IMoniker, *mut super::ffi::FILETIME,
    ) -> HRESULT,
    pub Inverse: unsafe extern "system" fn(*mut IMoniker, *mut *mut IMoniker) -> HRESULT,
    pub CommonPrefixWith:
        unsafe extern "system" fn(*mut IMoniker, *mut IMoniker, *mut *mut IMoniker) -> HRESULT,
    pub RelativePathTo:
        unsafe extern "system" fn(*mut IMoniker, *mut IMoniker, *mut *mut IMoniker) -> HRESULT,
    pub GetDisplayName: unsafe extern "system" fn(
        *mut IMoniker, *mut IBindCtx, *mut IMoniker, *mut LPWSTR,
    ) -> HRESULT,
    pub ParseDisplayName: unsafe extern "system" fn(
        *mut IMoniker, *mut IBindCtx, *mut IMoniker, LPCWSTR, *mut ULONG, *mut *mut IMoniker,
    ) -> HRESULT,
    pub IsSystemMoniker: unsafe extern "system" fn(*mut IMoniker, *mut DWORD) -> HRESULT,
}
#[repr(C)]
pub struct IMoniker {
    pub lpVtbl: *const IMonikerVtbl,
}
