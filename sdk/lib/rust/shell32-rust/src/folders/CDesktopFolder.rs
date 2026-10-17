// rustconv-assembly: free-functions
use crate::prelude::*;
// The crate's own functions: the definition is the declaration.
use crate::reg_folder::CRegFolder_CreateInstance;
use crate::wine::shellpath::SHGetSpecialFolderPathW;
// One declaration per external symbol: island.rs.
use crate::island::HeapAlloc;
use core::ffi::c_void;
// Substrate comes from the bindgen island via `prelude`, and from nowhere else.
// Re-importing the same names through shellcom/types/win32 does not add anything the
// prelude lacks -- it supplies them by a second path, and every one becomes ambiguous
// at the use site. See prelude.rs for why this is "instead of", not "as well as".
//
// `IShellView` is the sibling `--cpp-com` assembly's (CDesktopFolderEnum.rs, `crate::
// desktop_folder_enum`) real lpVtbl-shaped rendering of that COM interface -- the island's own
// opaque-class rendering has no callable `GetCurrentInfo` slot. Same precedent as CNetFolder.rs
// importing `IDefaultExtractIconInit`/`QCMINFO` from its own sibling enum module rather than
// re-deriving a third time. Every call site below that needs the island's OWN (opaque)
// `IShellView` -- i.e. `SHCreateShellFolderView`'s `view` out-param, whose signature is fixed at
// ITS OWN declaration in the island and unaffected by this shadowing import -- uses `.cast()`
// so inference picks the right one rather than the bare (now-shadowed) name.
use crate::desktop_folder_enum::IShellView;

// ---- file-scope preamble (translated) ----
//
// shfldr.h's REQUIREDREGITEM/REGFOLDERINFO/REGFOLDERINITDATA field names (clsid/pszCpl/Order,
// PidlType/Count/Items/clsid/pszParsingPath/pszEnumKeyName, psfOuter/pInfo) -- the assembler
// invented different ones (szModule/Order, Type/nCount/pItems/szModule/szName,
// psfOuter/pRegFolderInfo) that don't exist on the real bindgen structs; ground truth restored
// here from dll/win32/shell32/shfldr.h. `pszCpl` is genuinely `LPCSTR` (ANSI, a .cpl module
// name), unlike this file's other WSTR fields -- inlined as a byte-string literal (matches
// open_with_menu.rs's `CMVERBMAP { Verb: b"openas\0".as_ptr() as LPCSTR, ... }`) rather than a
// named CPP_COM_WSTR_ static, since the WSTR naming would misdescribe it.
unsafe impl Sync for REQUIREDREGITEM {}
unsafe impl Sync for REGFOLDERINFO {}

pub static g_RequiredItems: [REQUIREDREGITEM; 3] = [
    REQUIREDREGITEM {
        clsid: core::ptr::addr_of!(CLSID_MyComputer),
        pszCpl: b"sysdm.cpl\0".as_ptr() as LPCSTR,
        Order: REGITEMORDER_MYCOMPUTER as BYTE,
    },
    REQUIREDREGITEM {
        clsid: core::ptr::addr_of!(CLSID_NetworkPlaces),
        pszCpl: b"ncpa.cpl\0".as_ptr() as LPCSTR,
        Order: REGITEMORDER_NETHOOD as BYTE,
    },
    REQUIREDREGITEM {
        clsid: core::ptr::addr_of!(CLSID_Internet),
        pszCpl: b"inetcpl.cpl\0".as_ptr() as LPCSTR,
        Order: REGITEMORDER_INTERNET as BYTE,
    },
];

pub static CPP_COM_WSTR_0: [WCHAR; 1] = [0];
pub static CPP_COM_WSTR_686789759: [WCHAR; 8] = [
    'D' as WCHAR, 'e' as WCHAR, 's' as WCHAR, 'k' as WCHAR,
    't' as WCHAR, 'o' as WCHAR, 'p' as WCHAR, 0,
];

pub static g_RegFolderInfo: REGFOLDERINFO = REGFOLDERINFO {
    PidlType: PT_DESKTOP_REGITEM as PIDLTYPE,
    Count: 3,
    Items: core::ptr::addr_of!(g_RequiredItems) as *const REQUIREDREGITEM,
    clsid: core::ptr::addr_of!(CLSID_ShellDesktop),
    pszParsingPath: CPP_COM_WSTR_0.as_ptr(),
    pszEnumKeyName: CPP_COM_WSTR_686789759.as_ptr(),
};

pub const dwDesktopAttributes: DWORD =
    SFGAO_HASSUBFOLDER | SFGAO_FILESYSTEM | SFGAO_FOLDER | SFGAO_FILESYSANCESTOR |
    SFGAO_STORAGEANCESTOR | SFGAO_HASPROPSHEET | SFGAO_STORAGE;

pub const dwMyComputerAttributes: DWORD =
    SFGAO_CANRENAME | SFGAO_CANDELETE | SFGAO_HASPROPSHEET | SFGAO_DROPTARGET |
    SFGAO_FILESYSANCESTOR | SFGAO_FOLDER | SFGAO_HASSUBFOLDER | SFGAO_CANLINK;

pub static mut dwMyNetPlacesAttributes: DWORD =
    SFGAO_CANRENAME | SFGAO_CANDELETE | SFGAO_HASPROPSHEET | SFGAO_DROPTARGET |
    SFGAO_FILESYSANCESTOR | SFGAO_FOLDER | SFGAO_HASSUBFOLDER | SFGAO_CANLINK;
unsafe extern "C" fn IsSelf(cidl: UINT, apidl: PCUITEMID_CHILD_ARRAY) -> BOOLEAN {
    if cidl == 0 || (cidl == 1 && !apidl.is_null() && _ILIsEmpty((*apidl.offset(0))) != 0) {
        1u8
    } else {
        0u8
    }
}
unsafe extern "C" fn IsRegItem(pidl: PCUITEMID_CHILD) -> *const CLSID {
    if !pidl.is_null() && (*pidl).mkid.cb == 2 + 2 + core::mem::size_of::<CLSID>() as u16 && (*pidl).mkid.abID[0] == PT_DESKTOP_REGITEM as BYTE {
        // C: `(const CLSID*)(&pidl->mkid.abID[2])`. `abID` is a bindgen flexible-array member
        // (`[BYTE; 1]`) -- indexing it with a literal >= 1 (`abID[2]`) is out-of-bounds on the
        // declared length and `cargo check` doesn't catch it (the `unconditional_panic` lint only
        // fires under full codegen, not `--emit=metadata`), so it would have shipped a silent
        // runtime panic. Use `.as_ptr().add(2)` instead, matching drives_folder.rs's identical
        // `abid_ptr.add(2)` for the same `mkid.abID[2]` C expression.
        return (*pidl).mkid.abID.as_ptr().add(2) as *const CLSID;
    }
    core::ptr::null()
}

unsafe fn IsRegItem_2(pidl: PCUITEMID_CHILD, clsid: *const GUID) -> bool {
    let pClass = IsRegItem(pidl);
    // GUID has no `PartialEq` impl (bindgen doesn't derive it); compare field-wise like every
    // other IID/CLSID comparison in this crate (`cpp_com_iid_equal`, matching drives_folder.rs's
    // own `IsRegItem_2_body` doing the identical comparison for the identical C++ overload).
    // `cpp_com_iid_equal`'s `REFIID` is `com_sys::_GUID`-based; `CLSID` here (from the island via
    // `ntdll_sys::nt`) is a different nominal GUID type of the same ABI shape -- `.cast()`.
    !pClass.is_null() && cpp_com_iid_equal(pClass.cast(), clsid.cast())
}
unsafe extern "C" fn MarkAsCommonItem(pidl: LPITEMIDLIST) {
    (*((*pidl).mkid.abID.as_ptr() as *mut PIDLDATA)).type_ |= PT_FS_COMMON_FLAG as BYTE;
}

#[inline]
unsafe fn cpp_com_iid_equal(left: REFIID, right: REFIID) -> bool {
    !left.is_null() && !right.is_null()
        && (*left).Data1 == (*right).Data1
        && (*left).Data2 == (*right).Data2
        && (*left).Data3 == (*right).Data3
        && (*left).Data4 == (*right).Data4
}

extern "C" {
    pub static IID_IUnknown: IID;
}

#[no_mangle]
pub unsafe extern "system" fn SHGetDesktopFolder(psf: *mut *mut IShellFolder) -> HRESULT {
    let mut hres: HRESULT = S_OK;

    if psf.is_null() {
        return E_INVALIDARG;
    }
    *psf = core::ptr::null_mut();
    // C++ `CDesktopFolder::_CreatorClass::CreateInstance(NULL, IID_PPV_ARG(IShellFolder, psf))`
    // -- an ATL template instantiation with no translated body under that name anywhere; the
    // concrete Rust factory (CDesktopFolder is Rust-owned, desktop_folder.rs) is
    // `CDesktopFolder_CreateInstance`, matching every other `_CreatorClass::CreateInstance` /
    // `ShellObjectCreatorInit<T>` resolution in this crate (see CNetFolder.rs's identical note).
    hres = crate::desktop_folder::CDesktopFolder_CreateInstance(
        core::ptr::null_mut(),
        core::ptr::addr_of!(IID_IShellFolder).cast(), (psf).cast(),
    );

    hres
}

// shfldr.h: `HRESULT CRegFolder_CreateInstance(PREGFOLDERINITDATA pInit, LPCITEMIDLIST pidlRoot,
// REFIID riid, void **ppv);` -- no WINAPI, so cdecl like every other un-annotated shfldr.h/
// shellutils.h helper this crate calls directly (e.g. `IDataObject_Constructor`).
extern "C" {
}

#[no_mangle]
pub unsafe extern "system" fn CDesktopFolder_CreateRegFolder_RustBridge(
    psfOuter: *mut IShellFolder,
    pidlRoot: LPCITEMIDLIST,
    riid: REFIID,
    ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let mut RegInit = REGFOLDERINITDATA {
        psfOuter: psfOuter,
        pInfo: core::ptr::addr_of!(g_RegFolderInfo),
    };
    CRegFolder_CreateInstance(core::ptr::addr_of_mut!(RegInit), pidlRoot, riid, ppv)
}

#[no_mangle]
pub unsafe extern "system" fn CDesktopFolderEnum_CreateInstance_RustBridge(
    pRegFolder: *mut IShellFolder2,
    dwFlags: DWORD,
    pRegEnum: *mut IEnumIDList,
    pDesktopEnum: *mut IEnumIDList,
    pCommonEnum: *mut IEnumIDList,
    riid: REFIID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    // `ShellObjectCreatorInit<CDesktopFolderEnum>` -- template instantiation with no translated
    // body; the concrete non-aggregating factory is the sibling `--cpp-com` assembly's
    // `desktop_folder_enum::CDesktopFolderEnum_CreateInstance` (matches CNetFolder.rs's identical
    // resolution for `CNetFolderEnum_CreateInstance`). Its `IEnumIDList`/`pRegFolder` params are
    // that module's OWN real-vtable types, a different nominal type than this file's
    // prelude-sourced (opaque) ones despite the identical C ABI layout -- `.cast()` at each site
    // rather than changing this bridge's own (ground-truth-matching) signature.
    crate::desktop_folder_enum::CDesktopFolderEnum_CreateInstance(
        pRegFolder.cast(),
        dwFlags,
        pRegEnum.cast(),
        pDesktopEnum.cast(),
        pCommonEnum.cast(),
        riid,
        ppv,
    )
}

// =====================================================================================
// CDesktopFolderViewCB (CDesktopFolder.h) -- file-local IShellFolderViewCB/IFolderFilter
// companion class. Translated in full here: it is part of this TU's retained closure (the
// class body lives in CDesktopFolder.cpp, not in a separate compilation unit the way
// CRecycleBinFolderViewCB.cpp is), and `CDesktopFolder_CreateShellFolderView_RustBridge`
// below -- whose signature is fixed, matching desktop_folder.rs's caller -- needs a real
// factory + vtables to compose it.
// =====================================================================================

// Deterministic from shobjidl.h
#[repr(C)]
pub struct IShellFolderViewCB {
    pub lpVtbl: *const IShellFolderViewCBVtbl,
}
#[repr(C)]
pub struct IShellFolderViewCBVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IShellFolderViewCB, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IShellFolderViewCB) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IShellFolderViewCB) -> ULONG,
    pub MessageSFVCB: unsafe extern "system" fn(This: *mut IShellFolderViewCB, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> HRESULT,
}
extern "C" {
    pub static IID_IShellFolderViewCB: IID;
}

// Deterministic from shobjidl.h
#[repr(C)]
pub struct IFolderFilter {
    pub lpVtbl: *const IFolderFilterVtbl,
}
#[repr(C)]
pub struct IFolderFilterVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IFolderFilter, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IFolderFilter) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IFolderFilter) -> ULONG,
    pub ShouldShow: unsafe extern "system" fn(This: *mut IFolderFilter, psf: *mut IShellFolder, pidlFolder: PCIDLIST_ABSOLUTE, pidlItem: PCUITEMID_CHILD) -> HRESULT,
    pub GetEnumFlags: unsafe extern "system" fn(This: *mut IFolderFilter, psf: *mut IShellFolder, pidlFolder: PCIDLIST_ABSOLUTE, phwnd: *mut HWND, pgrfFlags: *mut DWORD) -> HRESULT,
}
extern "C" {
    pub static IID_IFolderFilter: IID;
}

extern "system" {
    fn GetProcessHeap() -> HANDLE;
    fn HeapFree(heap: HANDLE, flags: DWORD, memory: *mut core::ffi::c_void) -> BOOL;
    fn Shell32RustObjectLock();
    fn Shell32RustObjectUnlock();
}

#[repr(C)]
pub struct CDesktopFolderViewCB {
    pub IShellFolderViewCB_iface: IShellFolderViewCB,
    pub IFolderFilter_iface: IFolderFilter,
    pub ref_count: core::sync::atomic::AtomicI32,
    pub m_pShellView: *mut IShellView, // Not ref-counted, matches the C++ member comment.
    pub m_IsProgmanHosted: u8,
}

unsafe extern "system" fn CDesktopFolderViewCB_IShellFolderViewCB_QueryInterface(This: *mut IShellFolderViewCB, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDesktopFolderViewCB, IShellFolderViewCB_iface)) as *mut CDesktopFolderViewCB;
    CDesktopFolderViewCB_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CDesktopFolderViewCB_IShellFolderViewCB_AddRef(This: *mut IShellFolderViewCB) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDesktopFolderViewCB, IShellFolderViewCB_iface)) as *mut CDesktopFolderViewCB;
    CDesktopFolderViewCB_AddRef(this)
}
unsafe extern "system" fn CDesktopFolderViewCB_IShellFolderViewCB_Release(This: *mut IShellFolderViewCB) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDesktopFolderViewCB, IShellFolderViewCB_iface)) as *mut CDesktopFolderViewCB;
    CDesktopFolderViewCB_Release(this)
}
unsafe extern "system" fn CDesktopFolderViewCB_IShellFolderViewCB_MessageSFVCB(This: *mut IShellFolderViewCB, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDesktopFolderViewCB, IShellFolderViewCB_iface)) as *mut CDesktopFolderViewCB;
    CDesktopFolderViewCB_MessageSFVCB_body(this, uMsg, wParam, lParam)
}

pub static CDESKTOPFOLDERVIEWCB_ISHELLFOLDERVIEWCB_VTBL: IShellFolderViewCBVtbl = IShellFolderViewCBVtbl {
    QueryInterface: CDesktopFolderViewCB_IShellFolderViewCB_QueryInterface,
    AddRef: CDesktopFolderViewCB_IShellFolderViewCB_AddRef,
    Release: CDesktopFolderViewCB_IShellFolderViewCB_Release,
    MessageSFVCB: CDesktopFolderViewCB_IShellFolderViewCB_MessageSFVCB,
};

unsafe extern "system" fn CDesktopFolderViewCB_IFolderFilter_QueryInterface(This: *mut IFolderFilter, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDesktopFolderViewCB, IFolderFilter_iface)) as *mut CDesktopFolderViewCB;
    CDesktopFolderViewCB_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CDesktopFolderViewCB_IFolderFilter_AddRef(This: *mut IFolderFilter) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDesktopFolderViewCB, IFolderFilter_iface)) as *mut CDesktopFolderViewCB;
    CDesktopFolderViewCB_AddRef(this)
}
unsafe extern "system" fn CDesktopFolderViewCB_IFolderFilter_Release(This: *mut IFolderFilter) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDesktopFolderViewCB, IFolderFilter_iface)) as *mut CDesktopFolderViewCB;
    CDesktopFolderViewCB_Release(this)
}
unsafe extern "system" fn CDesktopFolderViewCB_IFolderFilter_ShouldShow(This: *mut IFolderFilter, psf: *mut IShellFolder, pidlFolder: PCIDLIST_ABSOLUTE, pidlItem: PCUITEMID_CHILD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDesktopFolderViewCB, IFolderFilter_iface)) as *mut CDesktopFolderViewCB;
    CDesktopFolderViewCB_ShouldShow_body(this, psf, pidlFolder, pidlItem)
}
unsafe extern "system" fn CDesktopFolderViewCB_IFolderFilter_GetEnumFlags(This: *mut IFolderFilter, psf: *mut IShellFolder, pidlFolder: PCIDLIST_ABSOLUTE, phwnd: *mut HWND, pgrfFlags: *mut DWORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDesktopFolderViewCB, IFolderFilter_iface)) as *mut CDesktopFolderViewCB;
    CDesktopFolderViewCB_GetEnumFlags_body(this, psf, pidlFolder, phwnd, pgrfFlags)
}

pub static CDESKTOPFOLDERVIEWCB_IFOLDERFILTER_VTBL: IFolderFilterVtbl = IFolderFilterVtbl {
    QueryInterface: CDesktopFolderViewCB_IFolderFilter_QueryInterface,
    AddRef: CDesktopFolderViewCB_IFolderFilter_AddRef,
    Release: CDesktopFolderViewCB_IFolderFilter_Release,
    ShouldShow: CDesktopFolderViewCB_IFolderFilter_ShouldShow,
    GetEnumFlags: CDesktopFolderViewCB_IFolderFilter_GetEnumFlags,
};

unsafe fn CDesktopFolderViewCB_QueryInterface(this: *mut CDesktopFolderViewCB, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IUnknown)) || cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IShellFolderViewCB)) {
        *ppv = core::ptr::addr_of_mut!((*this).IShellFolderViewCB_iface) as *mut core::ffi::c_void;
    } else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IFolderFilter)) {
        *ppv = core::ptr::addr_of_mut!((*this).IFolderFilter_iface) as *mut core::ffi::c_void;
    } else {
        return E_NOINTERFACE;
    }
    CDesktopFolderViewCB_AddRef(this);
    S_OK
}

unsafe fn CDesktopFolderViewCB_AddRef(this: *mut CDesktopFolderViewCB) -> ULONG {
    (*this).ref_count.fetch_add(1, core::sync::atomic::Ordering::SeqCst) as ULONG + 1
}

unsafe fn CDesktopFolderViewCB_Release(this: *mut CDesktopFolderViewCB) -> ULONG {
    let refs = (*this).ref_count.fetch_sub(1, core::sync::atomic::Ordering::SeqCst) - 1;
    if refs == 0 {
        Shell32RustObjectUnlock();
        HeapFree(GetProcessHeap(), 0, this as *mut core::ffi::c_void);
    }
    refs as ULONG
}

// reactos/shellutils.h `ShellObjectCreator<CDesktopFolderViewCB>` -- the class is translated
// here, so the template instantiation is local: create the object with one reference (the
// reference the C++ `CComPtr<CDesktopFolderViewCB> sfviewcb;` would hold), matching
// dialogs/drvdefext.rs's identical `ShellObjectCreator` pattern for its own Rust-owned class.
unsafe fn ShellObjectCreator(ppv: *mut *mut CDesktopFolderViewCB) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    let this = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, (core::mem::size_of::<CDesktopFolderViewCB>()) as _) as *mut CDesktopFolderViewCB;
    if this.is_null() { return E_OUTOFMEMORY; }
    core::ptr::write(this, CDesktopFolderViewCB {
        IShellFolderViewCB_iface: IShellFolderViewCB { lpVtbl: core::ptr::addr_of!(CDESKTOPFOLDERVIEWCB_ISHELLFOLDERVIEWCB_VTBL) },
        IFolderFilter_iface: IFolderFilter { lpVtbl: core::ptr::addr_of!(CDESKTOPFOLDERVIEWCB_IFOLDERFILTER_VTBL) },
        ref_count: core::sync::atomic::AtomicI32::new(1),
        m_pShellView: core::ptr::null_mut(),
        m_IsProgmanHosted: 0, // C++ `CDesktopFolderViewCB() : m_IsProgmanHosted(0) {}`
    });
    Shell32RustObjectLock();
    *ppv = this;
    S_OK
}

// C++ `void Initialize(IShellView *psv) { m_pShellView = psv; }` -- a plain non-virtual setter,
// not a COM_MAP method.
pub unsafe fn CDesktopFolderViewCB_Initialize_body(this: *mut CDesktopFolderViewCB, psv: *mut IShellView) {
    (*this).m_pShellView = psv;
}

// C++ `static bool IsProgmanHostedBrowser(IShellView *psv)`.
unsafe fn CDesktopFolderViewCB_IsProgmanHostedBrowser_of(psv: *mut IShellView) -> bool {
    let mut settings: FOLDERSETTINGS = core::mem::zeroed();
    let hr = ((*(*psv).lpVtbl).GetCurrentInfo)(psv, &mut settings);
    hr >= 0 && (settings.fFlags & FWF_DESKTOP) != 0
}

// C++ `bool IsProgmanHostedBrowser()` (instance overload, caches into `m_IsProgmanHosted`).
pub unsafe fn CDesktopFolderViewCB_IsProgmanHostedBrowser_body(this: *mut CDesktopFolderViewCB) -> bool {
    const UNINITIALIZED: u8 = 0;
    const NOT_HOSTED: u8 = 1;
    const IS_HOSTED: u8 = 2;
    if (*this).m_IsProgmanHosted == UNINITIALIZED {
        (*this).m_IsProgmanHosted = if !(*this).m_pShellView.is_null()
            && CDesktopFolderViewCB_IsProgmanHostedBrowser_of((*this).m_pShellView)
        {
            IS_HOSTED
        } else {
            NOT_HOSTED
        };
    }
    (*this).m_IsProgmanHosted == IS_HOSTED
}

pub const SSF_STARTPANELON: DWORD = 0x00200000;
/// shobjidl.h `FWF_DESKTOP` -- the island only carries it under an anonymous
/// `__WIDL_shobjidl_generated_name_00000024_FWF_DESKTOP` name; every file that needs it keeps
/// its own friendly copy (matches CDesktopFolderEnum.rs's identical local `FWF_DESKTOP`).
pub const FWF_DESKTOP: DWORD = 0x20;

pub static CPP_COM_WSTR_NEWSTARTPANEL: [WCHAR; 31] = [
    'H' as WCHAR, 'i' as WCHAR, 'd' as WCHAR, 'e' as WCHAR, 'D' as WCHAR, 'e' as WCHAR, 's' as WCHAR,
    'k' as WCHAR, 't' as WCHAR, 'o' as WCHAR, 'p' as WCHAR, 'I' as WCHAR, 'c' as WCHAR, 'o' as WCHAR,
    'n' as WCHAR, 's' as WCHAR, '\\' as WCHAR, 'N' as WCHAR, 'e' as WCHAR, 'w' as WCHAR, 'S' as WCHAR,
    't' as WCHAR, 'a' as WCHAR, 'r' as WCHAR, 't' as WCHAR, 'P' as WCHAR, 'a' as WCHAR, 'n' as WCHAR,
    'e' as WCHAR, 'l' as WCHAR, 0,
];
pub static CPP_COM_WSTR_CLASSICSTARTMENU: [WCHAR; 34] = [
    'H' as WCHAR, 'i' as WCHAR, 'd' as WCHAR, 'e' as WCHAR, 'D' as WCHAR, 'e' as WCHAR, 's' as WCHAR,
    'k' as WCHAR, 't' as WCHAR, 'o' as WCHAR, 'p' as WCHAR, 'I' as WCHAR, 'c' as WCHAR, 'o' as WCHAR,
    'n' as WCHAR, 's' as WCHAR, '\\' as WCHAR, 'C' as WCHAR, 'l' as WCHAR, 'a' as WCHAR, 's' as WCHAR,
    's' as WCHAR, 'i' as WCHAR, 'c' as WCHAR, 'S' as WCHAR, 't' as WCHAR, 'a' as WCHAR, 'r' as WCHAR,
    't' as WCHAR, 'M' as WCHAR, 'e' as WCHAR, 'n' as WCHAR, 'u' as WCHAR, 0,
];

extern "system" {
    #[link_name = "SHELL32_IsShellFolderNamespaceItemHidden_RustBridge"]
    fn SHELL32_IsShellFolderNamespaceItemHidden(SubKey: LPCWSTR, Clsid: REFCLSID) -> BOOL;
}

// C++ `HRESULT WINAPI CDesktopFolderViewCB::ShouldShow(IShellFolder*, PCIDLIST_ABSOLUTE,
// PCUITEMID_CHILD)`.
pub unsafe fn CDesktopFolderViewCB_ShouldShow_body(this: *mut CDesktopFolderViewCB, psf: *mut IShellFolder, pidlFolder: PCIDLIST_ABSOLUTE, pidlItem: PCUITEMID_CHILD) -> HRESULT {
    let _ = psf;
    let _ = pidlFolder;
    let pClsid = IsRegItem(pidlItem);
    if CDesktopFolderViewCB_IsProgmanHostedBrowser_body(this) && !pClsid.is_null() {
        let new_start = SHELL_GetSetting!(SSF_STARTPANELON, fStartPanelOn);
        let sub_key: LPCWSTR = if new_start != 0 {
            CPP_COM_WSTR_NEWSTARTPANEL.as_ptr()
        } else {
            CPP_COM_WSTR_CLASSICSTARTMENU.as_ptr()
        };
        // Same `com_sys::_GUID` vs `ntdll_sys::nt::_GUID` split as `cpp_com_iid_equal` above.
        return if SHELL32_IsShellFolderNamespaceItemHidden(sub_key, pClsid.cast()) != 0 { S_FALSE } else { S_OK };
    }
    S_OK
}

// C++ `STDMETHODIMP GetEnumFlags(IShellFolder*, PCIDLIST_ABSOLUTE, HWND*, DWORD*) override
// { return E_NOTIMPL; }` -- inline in the header, so no out-of-line body exists to translate.
pub unsafe fn CDesktopFolderViewCB_GetEnumFlags_body(this: *mut CDesktopFolderViewCB, psf: *mut IShellFolder, pidlFolder: PCIDLIST_ABSOLUTE, phwnd: *mut HWND, pgrfFlags: *mut DWORD) -> HRESULT {
    let _ = (this, psf, pidlFolder, phwnd, pgrfFlags);
    E_NOTIMPL
}

// Not in the island/prelude; desktop_folder.rs declares this same import locally too.
extern "system" {
}

#[inline]
unsafe fn StringCchCopyW(dst: LPWSTR, capacity: usize, src: LPCWSTR) -> HRESULT {
    if dst.is_null() || src.is_null() || capacity == 0 { return E_INVALIDARG; }
    let len = lstrlenW(src) as usize;
    if len >= capacity { *dst = 0; return E_FAIL; }
    let bytes = (len + 1) * core::mem::size_of::<WCHAR>();
    memcpy(dst.cast(), src.cast(), bytes);
    S_OK
}

// C++ `HRESULT WINAPI CDesktopFolderViewCB::MessageSFVCB(UINT, WPARAM, LPARAM)`.
pub unsafe fn CDesktopFolderViewCB_MessageSFVCB_body(this: *mut CDesktopFolderViewCB, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> HRESULT {
    match uMsg {
        SFVM_VIEWRELEASE => {
            (*this).m_pShellView = core::ptr::null_mut();
            return S_OK;
        }
        SFVM_GETCOMMANDDIR => {
            let mut buf = [0 as WCHAR; MAX_PATH as usize];
            if SHGetSpecialFolderPathW(core::ptr::null_mut(), buf.as_mut_ptr(), CSIDL_DESKTOPDIRECTORY as INT, TRUE) != 0 {
                return StringCchCopyW(lParam as LPWSTR, wParam as usize, buf.as_ptr());
            }
        }
        _ => {}
    }
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn CDesktopFolder_CreateShellFolderView_RustBridge(
    psf: *mut IShellFolder,
    ppvOut: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let mut sfviewcb: *mut CDesktopFolderViewCB = core::ptr::null_mut();
    let mut hr: HRESULT = ShellObjectCreator(&mut sfviewcb);
    if hr >= 0 {
        let mut create = SFV_CREATE {
            cbSize: core::mem::size_of::<SFV_CREATE>() as u32,
            pshf: psf,
            psvOuter: core::ptr::null_mut(),
            // `psfvcb`'s real field type is the ISLAND's own (opaque) `IShellFolderViewCB`, fixed
            // at SFV_CREATE's own declaration and unaffected by this file's local (real-vtable)
            // `IShellFolderViewCB` of the same name -- `.cast()` infers the right target instead
            // of naming it, matching the nominal-type-split hard rule.
            psfvcb: sfviewcb.cast(),
        };
        // Likewise `view`'s real parameter type is the island's own opaque `IShellView`, not this
        // file's imported (real-vtable) one -- `.cast()` here too.
        hr = SHCreateShellFolderView(&mut create, ppvOut.cast());
        if hr >= 0 {
            CDesktopFolderViewCB_Initialize_body(sfviewcb, (*ppvOut).cast());
        }
        CDesktopFolderViewCB_Release(sfviewcb);
    }
    hr
}


// ============================================================================
// ORIGINAL C SOURCE (this slice only)
// ============================================================================
//
// /*
//  *    Virtual Desktop Folder
//  *
//  *    Copyright 1997                Marcus Meissner
//  *    Copyright 1998, 1999, 2002    Juergen Schmied
//  *    Copyright 2009                Andrew Hill
//  *
//  * This library is free software; you can redistribute it and/or
//  * modify it under the terms of the GNU Lesser General Public
//  * License as published by the Free Software Foundation; either
//  * version 2.1 of the License, or (at your option) any later version.
//  *
//  * This library is distributed in the hope that it will be useful,
//  * but WITHOUT ANY WARRANTY; without even the implied warranty of
//  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
//  * Lesser General Public License for more details.
//  *
//  * You should have received a copy of the GNU Lesser General Public
//  * License along with this library; if not, write to the Free Software
//  * Foundation, Inc., 51 Franklin St, Fifth Floor, Boston, MA 02110-1301, USA
//  */
//
// #include <precomp.h>
// #include "CFSFolder.h" // Only for CFSFolder::*FSColumn* helpers!
//
// WINE_DEFAULT_DEBUG_CHANNEL(shell);
//
// extern BOOL SHELL32_IsShellFolderNamespaceItemHidden(LPCWSTR SubKey, REFCLSID Clsid);
//
// static const REQUIREDREGITEM g_RequiredItems[] =
// {
//     { CLSID_MyComputer, "sysdm.cpl", REGITEMORDER_MYCOMPUTER },
//     { CLSID_NetworkPlaces, "ncpa.cpl", REGITEMORDER_NETHOOD },
//     { CLSID_Internet, "inetcpl.cpl", REGITEMORDER_INTERNET },
// };
// static const REGFOLDERINFO g_RegFolderInfo =
// {
//     PT_DESKTOP_REGITEM,
//     _countof(g_RequiredItems), g_RequiredItems,
//     CLSID_ShellDesktop,
//     L"",
//     L"Desktop",
// };
//
// static BOOL IsSelf(UINT cidl, PCUITEMID_CHILD_ARRAY apidl)
// {
//     return cidl == 0 || (cidl == 1 && apidl && _ILIsEmpty(apidl[0]));
// }
// static const CLSID* IsRegItem(PCUITEMID_CHILD pidl)
// {
//     if (pidl && pidl->mkid.cb == 2 + 2 + sizeof(CLSID) && pidl->mkid.abID[0] == PT_DESKTOP_REGITEM)
//         return (const CLSID*)(&pidl->mkid.abID[2]);
//     return NULL;
// }
// static bool IsRegItem(PCUITEMID_CHILD pidl, REFCLSID clsid)
// {
//     const CLSID *pClass = IsRegItem(pidl);
//     return pClass && *pClass == clsid;
// }
// static inline void MarkAsCommonItem(LPITEMIDLIST pidl)
// {
//     ASSERT(_ILGetFSType(pidl) & PT_FS);
//     ((PIDLDATA*)pidl->mkid.abID)->type |= PT_FS_COMMON_FLAG;
// }
// HRESULT WINAPI SHGetDesktopFolder(IShellFolder **psf)
// {
//     HRESULT    hres = S_OK;
//     TRACE("\n");
//
//     if(!psf) return E_INVALIDARG;
//     *psf = NULL;
//     hres = CDesktopFolder::_CreatorClass::CreateInstance(NULL, IID_PPV_ARG(IShellFolder, psf));
//
//     TRACE("-- %p->(%p)\n",psf, *psf);
//     return hres;
// }
// extern "C" HRESULT WINAPI
// CDesktopFolder_CreateRegFolder_RustBridge(
//     IShellFolder *psfOuter, LPCITEMIDLIST pidlRoot, REFIID riid, void **ppv)
// {
//     REGFOLDERINITDATA RegInit = { psfOuter, &g_RegFolderInfo };
//     return CRegFolder_CreateInstance(&RegInit, pidlRoot, riid, ppv);
// }
// extern "C" HRESULT WINAPI
// CDesktopFolderEnum_CreateInstance_RustBridge(
//     IShellFolder2 *pRegFolder, DWORD dwFlags, IEnumIDList *pRegEnum,
//     IEnumIDList *pDesktopEnum, IEnumIDList *pCommonEnum, REFIID riid, void **ppv)
// {
//     return ShellObjectCreatorInit<CDesktopFolderEnum>(
//         pRegFolder, dwFlags, pRegEnum, pDesktopEnum, pCommonEnum, riid, ppv);
// }
// extern "C" HRESULT WINAPI
// CDesktopFolder_CreateShellFolderView_RustBridge(IShellFolder *psf, void **ppvOut)
// {
//     CComPtr<CDesktopFolderViewCB> sfviewcb;
//     HRESULT hr = ShellObjectCreator(sfviewcb);
//     if (SUCCEEDED(hr))
//     {
//         SFV_CREATE create = { sizeof(create), psf, NULL, sfviewcb };
//         hr = SHCreateShellFolderView(&create, (IShellView**)ppvOut);
//         if (SUCCEEDED(hr))
//             sfviewcb->Initialize((IShellView*)*ppvOut);
//     }
//     return hr;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
