//! shell32's COM server: the class-object table, the class factories, DllGetClassObject,
//! DllCanUnloadNow and the module lock count — what ATL's CComModule + BEGIN_OBJECT_MAP did in
//! shell32.cpp. Phase 2 of retiring that file; registration (DllRegisterServer and the object
//! map's .rgs resources) stays C++ until Phase 3.
//!
//! ATL SEMANTICS REPRODUCED, because each is observable by a COM client:
//!   * DllGetClassObject: NULL `ppv` -> E_POINTER; `*ppv` cleared first; ONE class factory per
//!     CLSID, created on first request and then CACHED (CComModule::GetClassObject's `pCF`);
//!     every call is a QueryInterface on that cached factory; an unknown CLSID ->
//!     CLASS_E_CLASSNOTAVAILABLE (AtlComModuleGetClassObject's result for no match).
//!   * The cached factory is a CComObjectCached<CComClassFactory>: the module owns one
//!     reference, and a CLIENT reference — the transition 1 -> 2 — LOCKS THE MODULE, dropping
//!     back to 1 unlocks it. So a client holding a class object keeps DllCanUnloadNow at
//!     S_FALSE. The factories here are statics whose count starts at that module reference and
//!     never reaches zero, which is the cached object's lifetime until process exit.
//!   * CComClassFactory::CreateInstance: NULL `ppvObj` -> E_POINTER; aggregation is refused
//!     (CLASS_E_NOAGGREGATION) unless the caller asks for IID_IUnknown; otherwise the class's
//!     creator. LockServer(TRUE/FALSE) = module Lock/Unlock.
//!
//! ONE lock count. Every Rust COM object locks the module through Shell32RustObjectLock/Unlock
//! (the rustconv scaffold's _DEFAULT_MODULE_LOCK), and after Phase 1 no C++ COM object is left
//! in shell32, so this counter is the whole of what CComModule's m_nLockCnt used to hold.
//! menuband-rust's objects lock through MenuBandRustModuleLock/Unlock, which in shell32.dll
//! are the same count (rshell.dll provides its own pair over rshell's module).
//!
//! The table below is generated mechanically from shell32.cpp's BEGIN_OBJECT_MAP (same order,
//! same CLSID -> *_CreateInstance pairs the 36 CXxxRustEntry::_CreatorClass shims called);
//! every creator was checked to be a single 3-argument `extern "system"` definition, which is
//! what the C++ declared (`extern "C" HRESULT WINAPI`) — a cdecl one would corrupt the stack on
//! i386.
#![allow(non_upper_case_globals)]

use crate::island::{BOOL, GUID, HRESULT, IID, LPVOID, ULONG};

// `REFIID` is a C macro (`const IID *`), so the island has no such name.
type REFIID = *const IID;
use core::ffi::c_void;
use core::sync::atomic::{AtomicI32, Ordering};

const S_OK: HRESULT = 0;
const S_FALSE: HRESULT = 1;
const E_POINTER: HRESULT = 0x8000_4003u32 as HRESULT;
const E_NOINTERFACE: HRESULT = 0x8000_4002u32 as HRESULT;
const CLASS_E_NOAGGREGATION: HRESULT = 0x8004_0110u32 as HRESULT;
const CLASS_E_CLASSNOTAVAILABLE: HRESULT = 0x8004_0111u32 as HRESULT;

type Creator = unsafe extern "system" fn(outer: *mut c_void, riid: REFIID, ppv: *mut LPVOID) -> HRESULT;

extern "C" {
    static IID_IUnknown: IID;
    static IID_IClassFactory: IID;
    static CLSID_ActiveDesktop: GUID;
    static CLSID_AdminFolderShortcut: GUID;
    static CLSID_ControlPanel: GUID;
    static CLSID_CopyAsPathMenu: GUID;
    static CLSID_CopyToMenu: GUID;
    static CLSID_DragDropHelper: GUID;
    static CLSID_ExeDropHandler: GUID;
    static CLSID_FontsFolderShortcut: GUID;
    static CLSID_MenuBand: GUID;
    static CLSID_MenuBandSite: GUID;
    static CLSID_MenuDeskBar: GUID;
    static CLSID_MergedFolder: GUID;
    static CLSID_MoveToMenu: GUID;
    static CLSID_MyComputer: GUID;
    static CLSID_MyDocuments: GUID;
    static CLSID_NetworkPlaces: GUID;
    static CLSID_NewMenu: GUID;
    static CLSID_OpenControlPanel: GUID;
    static CLSID_OpenWithMenu: GUID;
    static CLSID_Printers: GUID;
    static CLSID_QueryAssociations: GUID;
    static CLSID_RecycleBin: GUID;
    static CLSID_RecycleBinCleaner: GUID;
    static CLSID_SendToMenu: GUID;
    static CLSID_Shell: GUID;
    static CLSID_ShellDesktop: GUID;
    static CLSID_ShellDrvDefExt: GUID;
    static CLSID_ShellFSFolder: GUID;
    static CLSID_ShellFileDefExt: GUID;
    static CLSID_ShellFldSetExt: GUID;
    static CLSID_ShellItem: GUID;
    static CLSID_ShellLibrary: GUID;
    static CLSID_ShellLink: GUID;
    static CLSID_StartMenu: GUID;
    static CLSID_UserEventTimer: GUID;
    static CLSID_UserNotification: GUID;
}

extern "system" {
    fn CActiveDesktop_CreateInstance(outer: *mut c_void, riid: REFIID, ppv: *mut LPVOID) -> HRESULT;
    fn CAdminToolsFolder_CreateInstance(outer: *mut c_void, riid: REFIID, ppv: *mut LPVOID) -> HRESULT;
    fn CControlPanelFolder_CreateInstance(outer: *mut c_void, riid: REFIID, ppv: *mut LPVOID) -> HRESULT;
    fn CCopyAsPathMenu_CreateInstance(outer: *mut c_void, riid: REFIID, ppv: *mut LPVOID) -> HRESULT;
    fn CCopyToMenu_CreateInstance(outer: *mut c_void, riid: REFIID, ppv: *mut LPVOID) -> HRESULT;
    fn CDesktopFolder_CreateInstance(outer: *mut c_void, riid: REFIID, ppv: *mut LPVOID) -> HRESULT;
    fn CDrivesFolder_CreateInstance(outer: *mut c_void, riid: REFIID, ppv: *mut LPVOID) -> HRESULT;
    fn CDropTargetHelper_CreateInstance(outer: *mut c_void, riid: REFIID, ppv: *mut LPVOID) -> HRESULT;
    fn CDrvDefExt_CreateInstance(outer: *mut c_void, riid: REFIID, ppv: *mut LPVOID) -> HRESULT;
    fn CExeDropHandler_CreateInstance(outer: *mut c_void, riid: REFIID, ppv: *mut LPVOID) -> HRESULT;
    fn CFSFolder_CreateInstance(outer: *mut c_void, riid: REFIID, ppv: *mut LPVOID) -> HRESULT;
    fn CFileDefExt_CreateInstance(outer: *mut c_void, riid: REFIID, ppv: *mut LPVOID) -> HRESULT;
    fn CFolderOptions_CreateInstance(outer: *mut c_void, riid: REFIID, ppv: *mut LPVOID) -> HRESULT;
    fn CFontsFolder_CreateInstance(outer: *mut c_void, riid: REFIID, ppv: *mut LPVOID) -> HRESULT;
    fn CMenuBand_CreateInstance(outer: *mut c_void, riid: REFIID, ppv: *mut LPVOID) -> HRESULT;
    fn CMenuDeskBar_CreateInstance(outer: *mut c_void, riid: REFIID, ppv: *mut LPVOID) -> HRESULT;
    fn CMenuSite_CreateInstance(outer: *mut c_void, riid: REFIID, ppv: *mut LPVOID) -> HRESULT;
    fn CMergedFolder_CreateInstance(outer: *mut c_void, riid: REFIID, ppv: *mut LPVOID) -> HRESULT;
    fn CMoveToMenu_CreateInstance(outer: *mut c_void, riid: REFIID, ppv: *mut LPVOID) -> HRESULT;
    fn CMyDocsFolder_CreateInstance(outer: *mut c_void, riid: REFIID, ppv: *mut LPVOID) -> HRESULT;
    fn CNetFolder_CreateInstance(outer: *mut c_void, riid: REFIID, ppv: *mut LPVOID) -> HRESULT;
    fn CNewMenu_CreateInstance(outer: *mut c_void, riid: REFIID, ppv: *mut LPVOID) -> HRESULT;
    fn COpenControlPanel_CreateInstance(outer: *mut c_void, riid: REFIID, ppv: *mut LPVOID) -> HRESULT;
    fn COpenWithMenu_CreateInstance(outer: *mut c_void, riid: REFIID, ppv: *mut LPVOID) -> HRESULT;
    fn CPrinterFolder_CreateInstance(outer: *mut c_void, riid: REFIID, ppv: *mut LPVOID) -> HRESULT;
    fn CQueryAssociations_CreateInstance(outer: *mut c_void, riid: REFIID, ppv: *mut LPVOID) -> HRESULT;
    fn CRecycleBinCleaner_CreateInstance(outer: *mut c_void, riid: REFIID, ppv: *mut LPVOID) -> HRESULT;
    fn CRecycleBin_CreateInstance(outer: *mut c_void, riid: REFIID, ppv: *mut LPVOID) -> HRESULT;
    fn CSendToMenu_CreateInstance(outer: *mut c_void, riid: REFIID, ppv: *mut LPVOID) -> HRESULT;
    fn CShellDispatch_CreateInstance(outer: *mut c_void, riid: REFIID, ppv: *mut LPVOID) -> HRESULT;
    fn CShellItem_CreateInstance(outer: *mut c_void, riid: REFIID, ppv: *mut LPVOID) -> HRESULT;
    fn CShellLibrary_CreateInstance(outer: *mut c_void, riid: REFIID, ppv: *mut LPVOID) -> HRESULT;
    fn CShellLink_CreateInstance(outer: *mut c_void, riid: REFIID, ppv: *mut LPVOID) -> HRESULT;
    fn CStartMenuDummy_CreateInstance(outer: *mut c_void, riid: REFIID, ppv: *mut LPVOID) -> HRESULT;
    fn CUserEventTimer_CreateInstance(outer: *mut c_void, riid: REFIID, ppv: *mut LPVOID) -> HRESULT;
    fn CUserNotification_CreateInstance(outer: *mut c_void, riid: REFIID, ppv: *mut LPVOID) -> HRESULT;
}

// ── the module lock count ───────────────────────────────────────────────────────────────────

static LOCK_COUNT: AtomicI32 = AtomicI32::new(0);

#[no_mangle]
pub unsafe extern "system" fn Shell32RustObjectLock() {
    LOCK_COUNT.fetch_add(1, Ordering::SeqCst);
}

#[no_mangle]
pub unsafe extern "system" fn Shell32RustObjectUnlock() {
    LOCK_COUNT.fetch_sub(1, Ordering::SeqCst);
}

/// menuband-rust's module-lock hooks; in shell32.dll they are shell32's own count.
#[no_mangle]
pub unsafe extern "system" fn MenuBandRustModuleLock() {
    Shell32RustObjectLock();
}

#[no_mangle]
pub unsafe extern "system" fn MenuBandRustModuleUnlock() {
    Shell32RustObjectUnlock();
}

#[no_mangle]
pub unsafe extern "system" fn DllCanUnloadNow() -> HRESULT {
    if LOCK_COUNT.load(Ordering::SeqCst) == 0 { S_OK } else { S_FALSE }
}

// ── the cached class factories ──────────────────────────────────────────────────────────────

#[repr(C)]
struct ClassFactoryVtbl {
    QueryInterface: unsafe extern "system" fn(*mut ClassFactory, REFIID, *mut LPVOID) -> HRESULT,
    AddRef: unsafe extern "system" fn(*mut ClassFactory) -> ULONG,
    Release: unsafe extern "system" fn(*mut ClassFactory) -> ULONG,
    CreateInstance:
        unsafe extern "system" fn(*mut ClassFactory, *mut c_void, REFIID, *mut LPVOID) -> HRESULT,
    LockServer: unsafe extern "system" fn(*mut ClassFactory, BOOL) -> HRESULT,
}

/// One per object-map entry. `refs` starts at 1: the module's own cached reference.
#[repr(C)]
struct ClassFactory {
    vtbl: *const ClassFactoryVtbl,
    refs: AtomicI32,
    create: Creator,
}

// SAFETY: the factories are only ever mutated through `refs`, an atomic.
unsafe impl Sync for ClassFactory {}

static CLASS_FACTORY_VTBL: ClassFactoryVtbl = ClassFactoryVtbl {
    QueryInterface: cf_query_interface,
    AddRef: cf_add_ref,
    Release: cf_release,
    CreateInstance: cf_create_instance,
    LockServer: cf_lock_server,
};

#[inline]
unsafe fn guid_eq(a: *const GUID, b: *const GUID) -> bool {
    core::slice::from_raw_parts(a as *const u8, core::mem::size_of::<GUID>())
        == core::slice::from_raw_parts(b as *const u8, core::mem::size_of::<GUID>())
}

unsafe extern "system" fn cf_query_interface(this: *mut ClassFactory, riid: REFIID, ppv: *mut LPVOID) -> HRESULT {
    if ppv.is_null() {
        return E_POINTER;
    }
    *ppv = core::ptr::null_mut();
    // CComClassFactory's COM map: IClassFactory, and IUnknown through it.
    if guid_eq(riid, &IID_IUnknown) || guid_eq(riid, &IID_IClassFactory) {
        cf_add_ref(this);
        *ppv = this as LPVOID;
        return S_OK;
    }
    E_NOINTERFACE
}

unsafe extern "system" fn cf_add_ref(this: *mut ClassFactory) -> ULONG {
    let n = (*this).refs.fetch_add(1, Ordering::SeqCst) + 1;
    if n == 2 {
        Shell32RustObjectLock(); // CComObjectCached: the first client reference locks the module
    }
    n as ULONG
}

unsafe extern "system" fn cf_release(this: *mut ClassFactory) -> ULONG {
    let n = (*this).refs.fetch_sub(1, Ordering::SeqCst) - 1;
    if n == 1 {
        Shell32RustObjectUnlock(); // ... and the last one unlocks it
    }
    n as ULONG
}

unsafe extern "system" fn cf_create_instance(
    this: *mut ClassFactory, outer: *mut c_void, riid: REFIID, ppv: *mut LPVOID,
) -> HRESULT {
    if ppv.is_null() {
        return E_POINTER;
    }
    *ppv = core::ptr::null_mut();
    if !outer.is_null() && !guid_eq(riid, &IID_IUnknown) {
        return CLASS_E_NOAGGREGATION;
    }
    ((*this).create)(outer, riid, ppv)
}

unsafe extern "system" fn cf_lock_server(_this: *mut ClassFactory, lock: BOOL) -> HRESULT {
    if lock != 0 { Shell32RustObjectLock() } else { Shell32RustObjectUnlock() }
    S_OK
}

macro_rules! factory {
    ($create:ident) => {
        ClassFactory { vtbl: &CLASS_FACTORY_VTBL, refs: AtomicI32::new(1), create: $create }
    };
}

/// shell32.cpp's BEGIN_OBJECT_MAP, in its order. `(&CLSID, factory)`.
static CLASS_OBJECTS: [(&'static GUID, ClassFactory); 36] = [
    (unsafe { &CLSID_ActiveDesktop }, factory!(CActiveDesktop_CreateInstance)),
    (unsafe { &CLSID_ShellFSFolder }, factory!(CFSFolder_CreateInstance)),
    (unsafe { &CLSID_MyComputer }, factory!(CDrivesFolder_CreateInstance)),
    (unsafe { &CLSID_ShellDesktop }, factory!(CDesktopFolder_CreateInstance)),
    (unsafe { &CLSID_ShellFileDefExt }, factory!(CFileDefExt_CreateInstance)),
    (unsafe { &CLSID_ShellDrvDefExt }, factory!(CDrvDefExt_CreateInstance)),
    (unsafe { &CLSID_ShellItem }, factory!(CShellItem_CreateInstance)),
    (unsafe { &CLSID_ShellLink }, factory!(CShellLink_CreateInstance)),
    (unsafe { &CLSID_Shell }, factory!(CShellDispatch_CreateInstance)),
    (unsafe { &CLSID_ShellLibrary }, factory!(CShellLibrary_CreateInstance)),
    (unsafe { &CLSID_DragDropHelper }, factory!(CDropTargetHelper_CreateInstance)),
    (unsafe { &CLSID_ControlPanel }, factory!(CControlPanelFolder_CreateInstance)),
    (unsafe { &CLSID_OpenControlPanel }, factory!(COpenControlPanel_CreateInstance)),
    (unsafe { &CLSID_MyDocuments }, factory!(CMyDocsFolder_CreateInstance)),
    (unsafe { &CLSID_NetworkPlaces }, factory!(CNetFolder_CreateInstance)),
    (unsafe { &CLSID_FontsFolderShortcut }, factory!(CFontsFolder_CreateInstance)),
    (unsafe { &CLSID_Printers }, factory!(CPrinterFolder_CreateInstance)),
    (unsafe { &CLSID_AdminFolderShortcut }, factory!(CAdminToolsFolder_CreateInstance)),
    (unsafe { &CLSID_ShellFldSetExt }, factory!(CFolderOptions_CreateInstance)),
    (unsafe { &CLSID_RecycleBinCleaner }, factory!(CRecycleBinCleaner_CreateInstance)),
    (unsafe { &CLSID_RecycleBin }, factory!(CRecycleBin_CreateInstance)),
    (unsafe { &CLSID_OpenWithMenu }, factory!(COpenWithMenu_CreateInstance)),
    (unsafe { &CLSID_NewMenu }, factory!(CNewMenu_CreateInstance)),
    (unsafe { &CLSID_SendToMenu }, factory!(CSendToMenu_CreateInstance)),
    (unsafe { &CLSID_CopyAsPathMenu }, factory!(CCopyAsPathMenu_CreateInstance)),
    (unsafe { &CLSID_CopyToMenu }, factory!(CCopyToMenu_CreateInstance)),
    (unsafe { &CLSID_MoveToMenu }, factory!(CMoveToMenu_CreateInstance)),
    (unsafe { &CLSID_StartMenu }, factory!(CStartMenuDummy_CreateInstance)),
    (unsafe { &CLSID_MenuBandSite }, factory!(CMenuSite_CreateInstance)),
    (unsafe { &CLSID_MenuBand }, factory!(CMenuBand_CreateInstance)),
    (unsafe { &CLSID_MenuDeskBar }, factory!(CMenuDeskBar_CreateInstance)),
    (unsafe { &CLSID_MergedFolder }, factory!(CMergedFolder_CreateInstance)),
    (unsafe { &CLSID_ExeDropHandler }, factory!(CExeDropHandler_CreateInstance)),
    (unsafe { &CLSID_QueryAssociations }, factory!(CQueryAssociations_CreateInstance)),
    (unsafe { &CLSID_UserNotification }, factory!(CUserNotification_CreateInstance)),
    (unsafe { &CLSID_UserEventTimer }, factory!(CUserEventTimer_CreateInstance)),
];

#[no_mangle]
pub unsafe extern "system" fn DllGetClassObject(rclsid: *const GUID, riid: REFIID, ppv: *mut LPVOID) -> HRESULT {
    if ppv.is_null() {
        return E_POINTER;
    }
    *ppv = core::ptr::null_mut();
    for (clsid, factory) in CLASS_OBJECTS.iter() {
        if guid_eq(rclsid, *clsid) {
            return cf_query_interface(factory as *const ClassFactory as *mut ClassFactory, riid, ppv);
        }
    }
    CLASS_E_CLASSNOTAVAILABLE
}
