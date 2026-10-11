//! #[repr(C)] mirrors of the private types shared through
//! dll/win32/ole32/compobj_private.h (+ wine/list.h), and the compobj.c
//! file-local structs.
//!
//! HAZARD (from the lane brief): Lane B mirrors the same header. Field order
//! and types MUST match the C exactly — integration dedupes into one internal
//! module. Every type here is listed in MANIFEST.md.

#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code)]

use core::ffi::c_void;
use super::com::*;
use super::ffi::*;

// ---------------------------------------------------------------------------
// wine/list.h — doubly-linked list. Layout: { next, prev }.
// ---------------------------------------------------------------------------

pub use winelist_rust::{
    list, list_add_after, list_add_before, list_add_head, list_add_tail, list_empty, list_head,
    list_init, list_next, list_remove,
};

/// LIST_ENTRY(elem, type, field) — container-of. Call as
/// `list_entry::<OpenDll>(cursor, offset_of!(OpenDll, entry))`.
#[inline]
pub unsafe fn list_entry<T>(elem: *mut list, field_offset: usize) -> *mut T {
    (elem as *mut u8).sub(field_offset) as *mut T
}

// ---------------------------------------------------------------------------
// compobj_private.h shared types
// ---------------------------------------------------------------------------

pub type APARTMENT = apartment;

/// MSHLFLAGS typedef (wtypesbase) — plain DWORD.
pub type MSHLFLAGS = DWORD;

/// struct apartment (compobj_private.h) — field order/type exact.
#[repr(C)]
pub struct apartment {
    pub entry: list,

    pub refs: LONG,             // refcount of the apartment (LOCK)
    pub multi_threaded: BOOL,   // multi-threaded or single-threaded apartment? (RO)
    pub tid: DWORD,             // thread id (RO)
    pub oxid: OXID,             // object exporter ID (RO)
    pub ipidc: LONG,            // interface pointer ID counter, starts at 1 (LOCK)
    pub cs: CRITICAL_SECTION,   // thread safety
    pub proxies: list,          // imported objects (CS cs)
    pub stubmgrs: list,         // stub managers for exported objects (CS cs)
    pub remunk_exported: BOOL,  // has the IRemUnknown interface been created yet? (CS cs)
    pub remoting_started: LONG, // has the RPC system been started for this apartment? (LOCK)
    pub loaded_dlls: list,      // list of dlls loaded by this apartment (CS cs)
    pub host_apt_tid: DWORD,    // thread ID of apartment hosting objects of differing model (CS cs)
    pub host_apt_hwnd: HWND,    // handle to apartment window of host apartment (CS cs)
    pub local_server: *mut LocalServer, // A marshallable object exposing local servers (CS cs)
    pub being_destroyed: BOOL,  // is currently being destroyed

    pub oidc: OID,              // object ID counter, starts at 1, zero is invalid OID (CS cs)

    // STA-only fields
    pub win: HWND,               // message window (LOCK)
    pub filter: LPMESSAGEFILTER, // message filter (CS cs)
    pub main: BOOL,              // is this a main-threaded-apartment? (RO)

    // wine-10 (combase apartment.c): CoIncrementMTAUsage cookies holding the MTA alive
    pub usage_cookies: list,     // (CS csApartment)
}

/// struct init_spy (compobj_private.h)
#[repr(C)]
pub struct init_spy {
    pub entry: list,
    pub spy: *mut IInitializeSpy,
    pub id: u32,
}

/// struct oletls (compobj_private.h) — what TEB->ReservedForOle points at.
///
/// wine-10 (combase `struct tlsdata`) layout. It is NOT private: callers read
/// `flags` straight out of the TEB (Wine's compobj test does, at +0Ch on x86 /
/// +14h on x64), so the field order is ABI. Keep `marshal/types.rs`'s copy in step.
#[repr(C)]
pub struct oletls {
    pub apt: *mut apartment,
    pub errorinfo: *mut IErrorInfo,   // see errorinfo.c
    pub thread_seqid: DWORD,          // returned with CoGetCurrentProcess
    pub flags: DWORD,                 // tlsdata_flags (+0Ch on x86)
    pub unknown0: *mut c_void,
    pub inits: DWORD,                 // number of times CoInitializeEx called
    pub ole_inits: DWORD,             // number of times OleInitialize called
    pub causality_id: GUID,           // unique identifier for each COM call
    pub pending_call_count_client: LONG, // number of client calls pending
    pub pending_call_count_server: LONG, // number of server calls pending
    pub unknown: DWORD,
    pub context_token: *mut IObjContext, // (+38h on x86)
    pub call_state: *mut IUnknown,       // current call context (+3Ch on x86)
    pub unknown2: [DWORD; 46],
    pub cancel_object: *mut IUnknown, // cancel object set by CoSetCancelObject (+F8h on x86)
    pub state: *mut IUnknown,         // see CoSetState
    pub spies: list,                  // Spies installed with CoRegisterInitializeSpy
    pub spies_lock: DWORD,
    pub cancelcount: DWORD,           // Co{Enable,Disable}CallCancellation nesting
}

// The ABI anchors (combase_private.h comments), and the marshal module's copy must agree.
const _: () = {
    use core::mem::{offset_of, size_of};
    const P: usize = core::mem::size_of::<usize>();
    assert!(offset_of!(oletls, flags) == 2 * P + 4);
    assert!(P != 4 || offset_of!(oletls, context_token) == 0x38);
    assert!(P != 4 || offset_of!(oletls, cancel_object) == 0xF8);
    assert!(offset_of!(oletls, state) == offset_of!(crate::marshal::types::oletls, state));
    assert!(offset_of!(oletls, spies) == offset_of!(crate::marshal::types::oletls, spies));
    assert!(size_of::<oletls>() == size_of::<crate::marshal::types::oletls>());
};

/// combase_private.h `enum tlsdata_flags`
pub const OLETLS_UUIDINITIALIZED: DWORD = 0x2;
pub const OLETLS_DISABLE_OLE1DDE: DWORD = 0x40;
pub const OLETLS_APARTMENTTHREADED: DWORD = 0x80;
pub const OLETLS_MULTITHREADED: DWORD = 0x100;

/// struct LocalServer (compobj.c) — marshallable object exposing registered
/// local servers via IServiceProvider.
#[repr(C)]
pub struct LocalServer {
    pub IServiceProvider_iface: IServiceProvider,
    pub ref_: LONG,
    pub apt: *mut apartment,
    pub marshal_stream: *mut IStream,
}

// ---------------------------------------------------------------------------
// compobj.c file-local types
// ---------------------------------------------------------------------------

/// enum comclass_threadingmodel
pub type comclass_threadingmodel = i32;
pub const ThreadingModel_Apartment: comclass_threadingmodel = 1;
pub const ThreadingModel_Free: comclass_threadingmodel = 2;
pub const ThreadingModel_No: comclass_threadingmodel = 3;
pub const ThreadingModel_Both: comclass_threadingmodel = 4;
pub const ThreadingModel_Neutral: comclass_threadingmodel = 5;

/// enum comclass_miscfields
pub type comclass_miscfields = i32;
pub const MiscStatus: comclass_miscfields = 1;
pub const MiscStatusIcon: comclass_miscfields = 2;
pub const MiscStatusContent: comclass_miscfields = 4;
pub const MiscStatusThumbnail: comclass_miscfields = 8;
pub const MiscStatusDocPrint: comclass_miscfields = 16;

/// struct comclassredirect_data (activation-context COM server section entry)
#[repr(C)]
pub struct comclassredirect_data {
    pub size: ULONG,
    pub res: BYTE,
    pub miscmask: BYTE,
    pub res1: [BYTE; 2],
    pub model: DWORD,
    pub clsid: GUID,
    pub alias: GUID,
    pub clsid2: GUID,
    pub tlbid: GUID,
    pub name_len: ULONG,
    pub name_offset: ULONG,
    pub progid_len: ULONG,
    pub progid_offset: ULONG,
    pub clrdata_len: ULONG,
    pub clrdata_offset: ULONG,
    pub miscstatus: DWORD,
    pub miscstatuscontent: DWORD,
    pub miscstatusthumbnail: DWORD,
    pub miscstatusicon: DWORD,
    pub miscstatusdocprint: DWORD,
}

/// struct ifacepsredirect_data (activation-context COM interface section entry)
#[repr(C)]
pub struct ifacepsredirect_data {
    pub size: ULONG,
    pub mask: DWORD,
    pub iid: GUID,
    pub nummethods: ULONG,
    pub tlbid: GUID,
    pub base: GUID,
    pub name_len: ULONG,
    pub name_offset: ULONG,
}

/// struct progidredirect_data (activation-context ProgID section entry)
#[repr(C)]
pub struct progidredirect_data {
    pub size: ULONG,
    pub reserved: DWORD,
    pub clsid_offset: ULONG,
}

/// struct class_reg_data — union { actctx-triple | hkey } + BOOL hkey tag.
/// The C anonymous union is modeled with the actctx (larger) arm as storage
/// plus accessors; layout: union is 3 pointers max, HKEY overlays the first.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct class_reg_data_u_actctx {
    pub data: *mut comclassredirect_data,
    pub section: *mut c_void,
    pub hactctx: HANDLE,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union class_reg_data_u {
    pub actctx: class_reg_data_u_actctx,
    pub hkey: HKEY,
}
#[repr(C)]
pub struct class_reg_data {
    pub u: class_reg_data_u,
    pub hkey: BOOL, // tag: TRUE = u.hkey valid, FALSE = u.actctx valid
}

/// struct registered_psclsid
#[repr(C)]
pub struct registered_psclsid {
    pub entry: list,
    pub iid: IID,
    pub clsid: CLSID,
}

/// typedef struct tagRegisteredClass
#[repr(C)]
pub struct RegisteredClass {
    pub entry: list,
    pub classIdentifier: CLSID,
    pub apartment_id: OXID,
    pub classObject: *mut IUnknown,
    pub runContext: DWORD,
    pub connectFlags: DWORD,
    pub dwCookie: DWORD,
    pub RpcRegistration: *mut c_void,
}

/// DllGetClassObject / DllCanUnloadNow entry-point types.
/// NOTE: the C declares DllGetClassObjectFunc as CALLBACK and
/// DllCanUnloadNowFunc as WINAPI — both are stdcall on i386.
pub type DllGetClassObjectFunc =
    Option<unsafe extern "system" fn(REFCLSID, REFIID, *mut *mut c_void) -> HRESULT>;
pub type DllCanUnloadNowFunc = Option<unsafe extern "system" fn() -> HRESULT>;

/// typedef struct tagOpenDll
#[repr(C)]
pub struct OpenDll {
    pub refs: LONG,
    pub library_name: LPWSTR,
    pub library: HANDLE,
    pub DllGetClassObject: DllGetClassObjectFunc,
    pub DllCanUnloadNow: DllCanUnloadNowFunc,
    pub entry: list,
}

/// struct apartment_loaded_dll
#[repr(C)]
pub struct apartment_loaded_dll {
    pub entry: list,
    pub dll: *mut OpenDll,
    pub unload_time: DWORD,
    pub multi_threaded: BOOL,
}

/// struct host_object_params
#[repr(C)]
pub struct host_object_params {
    pub regdata: class_reg_data,
    pub clsid: CLSID,  // clsid of object to marshal
    pub iid: IID,      // interface to marshal
    pub event: HANDLE, // event signalling when ready for multi-threaded case
    pub hr: HRESULT,   // result for multi-threaded case
    pub stream: *mut IStream, // stream that the object will be marshaled into
    pub apartment_threaded: BOOL, // is the component purely apartment-threaded?
}

/// struct host_thread_params
#[repr(C)]
pub struct host_thread_params {
    pub threading_model: COINIT,
    pub ready_event: HANDLE,
    pub apartment_hwnd: HWND,
}

/// struct ManualResetEvent (MREImpl)
#[repr(C)]
pub struct MREImpl {
    pub ISynchronize_iface: ISynchronize,
    pub ISynchronizeHandle_iface: ISynchronizeHandle,
    pub ref_: LONG,
    pub event: HANDLE,
}

/// struct Context — the per-thread context object (CoGetContextToken).
#[repr(C)]
pub struct Context {
    pub IComThreadingInfo_iface: IComThreadingInfo,
    pub IContextCallback_iface: IContextCallback,
    pub IObjContext_iface: IObjContext,
    pub refs: LONG,
}

/// struct GlobalOptions
#[repr(C)]
pub struct GlobalOptions {
    pub IGlobalOptions_iface: IGlobalOptions,
    pub ref_: LONG,
}

/// Opaque types owned by other lanes/files — never dereferenced here.
pub enum stub_manager {}
pub enum dispatch_params {}

/// DCOM messages used by the apartment window (compobj_private.h; wine-only)
pub const DM_EXECUTERPC: UINT = WM_USER + 0; // WPARAM = 0, LPARAM = dispatch_params*
pub const DM_HOSTOBJECT: UINT = WM_USER + 1; // WPARAM = 0, LPARAM = host_object_params*
