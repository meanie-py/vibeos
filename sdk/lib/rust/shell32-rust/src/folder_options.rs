//! Rust-owned `CFolderOptions` COM identity, state, property pages, and default-view
//! settings helpers.

use crate::shellcom::*;
// The crate's own functions: the definition is the declaration.
use crate::dialogs_filetypes_tu::FolderOptionsFileTypesDlg;
use crate::dialogs_general::FolderOptionsGeneralDlg;
use crate::dialogs_view::FolderOptionsViewDlg;
use crate::def_view::SHCreateShellFolderView;
use crate::desktop_folder_tu::SHGetDesktopFolder;
use crate::propsheet::SH_CreatePropertySheetPage;
use crate::propsheet::SH_CreatePropertySheetPageEx;
// One declaration per external symbol: island.rs.
use crate::island::CoCreateInstance;
use crate::island::DestroyPropertySheetPage;
use crate::island::IUnknown_Exec;
use crate::island::IUnknown_QueryService;
use crate::types::*;
use core::ffi::c_void;

pub type EXPPS = UINT;
pub type LPFNSVADDPROPSHEETPAGE =
    Option<unsafe extern "system" fn(HPROPSHEETPAGE, LPARAM) -> BOOL>;
pub type DLGPROC = unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> INT_PTR;
pub type LPFNPSPCALLBACKW =
    unsafe extern "system" fn(HWND, UINT, *mut PROPSHEETPAGEW) -> UINT;

pub use crate::island::PROPSHEETPAGEW;

#[repr(C)]
struct IGlobalFolderSettings {
    lpVtbl: *const IGlobalFolderSettingsVtbl,
}

#[repr(C)]
struct IGlobalFolderSettingsVtbl {
    QueryInterface: unsafe extern "system" fn(
        *mut IGlobalFolderSettings,
        REFIID,
        *mut *mut c_void,
    ) -> HRESULT,
    AddRef: unsafe extern "system" fn(*mut IGlobalFolderSettings) -> ULONG,
    Release: unsafe extern "system" fn(*mut IGlobalFolderSettings) -> ULONG,
    Get: unsafe extern "system" fn(*mut IGlobalFolderSettings, *mut c_void, UINT) -> HRESULT,
    Set: unsafe extern "system" fn(
        *mut IGlobalFolderSettings,
        *const c_void,
        UINT,
        UINT,
    ) -> HRESULT,
}

#[repr(C)]
struct IBrowserService2 {
    lpVtbl: *const IBrowserService2Vtbl,
}

// shdeprecated.idl: IBrowserService contributes 30 methods after IUnknown;
// IBrowserService2 then publishes WndProcBS followed by SetAsDefFolderSettings.
#[repr(C)]
struct IBrowserService2Vtbl {
    QueryInterface:
        unsafe extern "system" fn(*mut IBrowserService2, REFIID, *mut *mut c_void) -> HRESULT,
    AddRef: unsafe extern "system" fn(*mut IBrowserService2) -> ULONG,
    Release: unsafe extern "system" fn(*mut IBrowserService2) -> ULONG,
    browser_service_methods: [usize; 30],
    WndProcBS: usize,
    SetAsDefFolderSettings: unsafe extern "system" fn(*mut IBrowserService2) -> HRESULT,
}

#[repr(C)]
struct SFV_CREATE {
    cbSize: UINT,
    pshf: *mut IShellFolder,
    psvOuter: *mut c_void,
    psfvcb: *mut c_void,
}

#[repr(C)]
pub struct IShellExtInit {
    pub lpVtbl: *const IShellExtInitVtbl,
}

#[repr(C)]
pub struct IShellExtInitVtbl {
    pub QueryInterface:
        unsafe extern "system" fn(*mut IShellExtInit, REFIID, *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(*mut IShellExtInit) -> ULONG,
    pub Release: unsafe extern "system" fn(*mut IShellExtInit) -> ULONG,
    pub Initialize: unsafe extern "system" fn(
        *mut IShellExtInit,
        PCIDLIST_ABSOLUTE,
        *mut IDataObject,
        HKEY,
    ) -> HRESULT,
}

#[repr(C)]
pub struct IShellPropSheetExt {
    pub lpVtbl: *const IShellPropSheetExtVtbl,
}

#[repr(C)]
pub struct IShellPropSheetExtVtbl {
    pub QueryInterface: unsafe extern "system" fn(
        *mut IShellPropSheetExt,
        REFIID,
        *mut *mut c_void,
    ) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(*mut IShellPropSheetExt) -> ULONG,
    pub Release: unsafe extern "system" fn(*mut IShellPropSheetExt) -> ULONG,
    pub AddPages: unsafe extern "system" fn(
        *mut IShellPropSheetExt,
        LPFNSVADDPROPSHEETPAGE,
        LPARAM,
    ) -> HRESULT,
    pub ReplacePage: unsafe extern "system" fn(
        *mut IShellPropSheetExt,
        EXPPS,
        LPFNSVADDPROPSHEETPAGE,
        LPARAM,
    ) -> HRESULT,
}

#[repr(C)]
pub struct IObjectWithSite {
    pub lpVtbl: *const IObjectWithSiteVtbl,
}

#[repr(C)]
pub struct IObjectWithSiteVtbl {
    pub QueryInterface:
        unsafe extern "system" fn(*mut IObjectWithSite, REFIID, *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(*mut IObjectWithSite) -> ULONG,
    pub Release: unsafe extern "system" fn(*mut IObjectWithSite) -> ULONG,
    pub SetSite: unsafe extern "system" fn(*mut IObjectWithSite, *mut IUnknown) -> HRESULT,
    pub GetSite:
        unsafe extern "system" fn(*mut IObjectWithSite, REFIID, *mut *mut c_void) -> HRESULT,
}

extern "C" {
    static IID_IUnknown: IID;
    static IID_IShellExtInit: IID;
    static IID_IShellPropSheetExt: IID;
    static IID_IObjectWithSite: IID;
    static IID_IGlobalFolderSettings: IID;
    static CLSID_GlobalFolderSettings: CLSID;
    static IID_IBrowserService2: IID;
    static IID_IShellBrowser: IID;
    static CGID_DefView: GUID;

}

extern "system" {
    fn GetProcessHeap() -> HANDLE;
    #[link_name = "HeapAlloc"]
    fn cpp_com_HeapAlloc(heap: HANDLE, flags: DWORD, bytes: usize) -> *mut c_void;
    #[link_name = "HeapFree"]
    fn cpp_com_HeapFree(heap: HANDLE, flags: DWORD, memory: *mut c_void) -> BOOL;
    fn Shell32RustObjectLock();
    fn Shell32RustObjectUnlock();
}

#[repr(C)]
pub struct CFolderOptions {
    pub IShellPropSheetExt_iface: IShellPropSheetExt,
    pub IShellExtInit_iface: IShellExtInit,
    pub IObjectWithSite_iface: IObjectWithSite,
    pub ref_count: core::sync::atomic::AtomicI32,
    pub m_pSite: *mut IUnknown,
}

const DFSA_QUERY: i32 = -1;
const DFSA_RESET: i32 = 0;
const DFSA_APPLY: i32 = 1;
const CLSCTX_INPROC_SERVER: DWORD = 1;
const DVCMDID_RESET_DEFAULTFOLDER_SETTINGS: DWORD = 1;
const PSPCB_RELEASE: UINT = 1;
const IDD_FOLDER_OPTIONS_GENERAL: WORD = 17;
const IDD_FOLDER_OPTIONS_VIEW: WORD = 18;
const IDD_FOLDER_OPTIONS_FILETYPES: WORD = 19;

#[inline]
unsafe fn iid_equal(left: REFIID, right: REFIID) -> bool {
    !left.is_null()
        && !right.is_null()
        && (*left).Data1 == (*right).Data1
        && (*left).Data2 == (*right).Data2
        && (*left).Data3 == (*right).Data3
        && (*left).Data4 == (*right).Data4
}

unsafe fn assign_site(this: *mut CFolderOptions, value: *mut IUnknown) {
    let old = (*this).m_pSite;
    if old == value {
        return;
    }
    if !value.is_null() {
        ((*(*value).lpVtbl).AddRef)(value);
    }
    (*this).m_pSite = value;
    if !old.is_null() {
        ((*(*old).lpVtbl).Release)(old);
    }
}

unsafe fn query_interface(
    this: *mut CFolderOptions,
    riid: REFIID,
    out: *mut *mut c_void,
) -> HRESULT {
    if out.is_null() {
        return E_POINTER;
    }
    *out = core::ptr::null_mut();
    if iid_equal(riid, core::ptr::addr_of!(IID_IUnknown))
        || iid_equal(riid, core::ptr::addr_of!(IID_IShellPropSheetExt))
    {
        *out = core::ptr::addr_of_mut!((*this).IShellPropSheetExt_iface).cast();
    } else if iid_equal(riid, core::ptr::addr_of!(IID_IShellExtInit)) {
        *out = core::ptr::addr_of_mut!((*this).IShellExtInit_iface).cast();
    } else if iid_equal(riid, core::ptr::addr_of!(IID_IObjectWithSite)) {
        *out = core::ptr::addr_of_mut!((*this).IObjectWithSite_iface).cast();
    } else {
        return E_NOINTERFACE;
    }
    add_ref(this);
    S_OK
}

unsafe fn add_ref(this: *mut CFolderOptions) -> ULONG {
    (*this)
        .ref_count
        .fetch_add(1, core::sync::atomic::Ordering::SeqCst) as ULONG
        + 1
}

unsafe fn release(this: *mut CFolderOptions) -> ULONG {
    let refs = (*this)
        .ref_count
        .fetch_sub(1, core::sync::atomic::Ordering::SeqCst)
        - 1;
    if refs == 0 {
        assign_site(this, core::ptr::null_mut());
        Shell32RustObjectUnlock();
        cpp_com_HeapFree(GetProcessHeap(), 0, this.cast());
    }
    refs as ULONG
}

macro_rules! iface_this {
    ($this:expr, $field:ident) => {{
        ($this as *mut u8).sub(core::mem::offset_of!(CFolderOptions, $field))
            as *mut CFolderOptions
    }};
}

unsafe extern "system" fn prop_query(
    this: *mut IShellPropSheetExt,
    riid: REFIID,
    out: *mut *mut c_void,
) -> HRESULT {
    query_interface(iface_this!(this, IShellPropSheetExt_iface), riid, out)
}
unsafe extern "system" fn prop_add_ref(this: *mut IShellPropSheetExt) -> ULONG {
    add_ref(iface_this!(this, IShellPropSheetExt_iface))
}
unsafe extern "system" fn prop_release(this: *mut IShellPropSheetExt) -> ULONG {
    release(iface_this!(this, IShellPropSheetExt_iface))
}
unsafe extern "system" fn prop_add_pages(
    this: *mut IShellPropSheetExt,
    add_page: LPFNSVADDPROPSHEETPAGE,
    parameter: LPARAM,
) -> HRESULT {
    unsafe extern "system" fn lifetime_callback(
        _window: HWND,
        message: UINT,
        page: *mut PROPSHEETPAGEW,
    ) -> UINT {
        if message == PSPCB_RELEASE {
            let object = (*page).lParam as *mut IUnknown;
            ((*(*object).lpVtbl).Release)(object);
        }
        TRUE as UINT
    }

    unsafe fn add_property_page(
        page: HPROPSHEETPAGE,
        add_page: LPFNSVADDPROPSHEETPAGE,
        parameter: LPARAM,
    ) -> HRESULT {
        if page.is_null() {
            return E_FAIL;
        }
        if add_page.unwrap_unchecked()(page, parameter) != FALSE {
            return S_OK;
        }
        DestroyPropertySheetPage((page).cast());
        E_FAIL
    }

    let object = this.cast::<IUnknown>();
    let sheet_parameter = object as LPARAM;
    let page = SH_CreatePropertySheetPageEx(
        IDD_FOLDER_OPTIONS_GENERAL,
        Some(FolderOptionsGeneralDlg),
        sheet_parameter,
        core::ptr::null(),
        Some(lifetime_callback),
    );
    let mut hr = add_property_page(page, add_page, parameter);
    if hr < 0 {
        return hr;
    }

    // The general page owns this reference until PSPCB_RELEASE.
    ((*(*object).lpVtbl).AddRef)(object);

    let page = SH_CreatePropertySheetPage(
        IDD_FOLDER_OPTIONS_VIEW,
        Some(FolderOptionsViewDlg),
        sheet_parameter,
        core::ptr::null(),
    );
    if page.is_null() || add_page.unwrap_unchecked()(page, parameter) == FALSE {
        return E_FAIL;
    }

    let page = SH_CreatePropertySheetPage(
        IDD_FOLDER_OPTIONS_FILETYPES,
        Some(FolderOptionsFileTypesDlg),
        sheet_parameter,
        core::ptr::null(),
    );
    if page.is_null() || add_page.unwrap_unchecked()(page, parameter) == FALSE {
        E_FAIL
    } else {
        S_OK
    }
}
unsafe extern "system" fn prop_replace_page(
    _this: *mut IShellPropSheetExt,
    _page: EXPPS,
    _replace: LPFNSVADDPROPSHEETPAGE,
    _parameter: LPARAM,
) -> HRESULT {
    E_NOTIMPL
}

pub static CFOLDEROPTIONS_ISHELLPROPSHEETEXT_VTBL: IShellPropSheetExtVtbl =
    IShellPropSheetExtVtbl {
        QueryInterface: prop_query,
        AddRef: prop_add_ref,
        Release: prop_release,
        AddPages: prop_add_pages,
        ReplacePage: prop_replace_page,
    };

unsafe extern "system" fn init_query(
    this: *mut IShellExtInit,
    riid: REFIID,
    out: *mut *mut c_void,
) -> HRESULT {
    query_interface(iface_this!(this, IShellExtInit_iface), riid, out)
}
unsafe extern "system" fn init_add_ref(this: *mut IShellExtInit) -> ULONG {
    add_ref(iface_this!(this, IShellExtInit_iface))
}
unsafe extern "system" fn init_release(this: *mut IShellExtInit) -> ULONG {
    release(iface_this!(this, IShellExtInit_iface))
}
unsafe extern "system" fn initialize(
    _this: *mut IShellExtInit,
    _folder: PCIDLIST_ABSOLUTE,
    _data_object: *mut IDataObject,
    _program_key: HKEY,
) -> HRESULT {
    S_OK
}

pub static CFOLDEROPTIONS_ISHELLEXTINIT_VTBL: IShellExtInitVtbl = IShellExtInitVtbl {
    QueryInterface: init_query,
    AddRef: init_add_ref,
    Release: init_release,
    Initialize: initialize,
};

unsafe extern "system" fn site_query(
    this: *mut IObjectWithSite,
    riid: REFIID,
    out: *mut *mut c_void,
) -> HRESULT {
    query_interface(iface_this!(this, IObjectWithSite_iface), riid, out)
}
unsafe extern "system" fn site_add_ref(this: *mut IObjectWithSite) -> ULONG {
    add_ref(iface_this!(this, IObjectWithSite_iface))
}
unsafe extern "system" fn site_release(this: *mut IObjectWithSite) -> ULONG {
    release(iface_this!(this, IObjectWithSite_iface))
}
unsafe extern "system" fn set_site(
    this: *mut IObjectWithSite,
    site: *mut IUnknown,
) -> HRESULT {
    assign_site(iface_this!(this, IObjectWithSite_iface), site);
    S_OK
}
unsafe extern "system" fn get_site(
    this: *mut IObjectWithSite,
    riid: REFIID,
    out: *mut *mut c_void,
) -> HRESULT {
    if out.is_null() {
        return E_POINTER;
    }
    let object = iface_this!(this, IObjectWithSite_iface);
    let site = (*object).m_pSite;
    if site.is_null() {
        *out = core::ptr::null_mut();
        E_FAIL
    } else {
        ((*(*site).lpVtbl).QueryInterface)(site, riid, out)
    }
}

pub static CFOLDEROPTIONS_IOBJECTWITHSITE_VTBL: IObjectWithSiteVtbl = IObjectWithSiteVtbl {
    QueryInterface: site_query,
    AddRef: site_add_ref,
    Release: site_release,
    SetSite: set_site,
    GetSite: get_site,
};

#[no_mangle]
pub unsafe extern "system" fn CFolderOptions_CreateInstance(
    outer: *mut IUnknown,
    riid: REFIID,
    out: *mut *mut c_void,
) -> HRESULT {
    if out.is_null() {
        return E_POINTER;
    }
    *out = core::ptr::null_mut();
    if !outer.is_null() {
        return CLASS_E_NOAGGREGATION;
    }
    let this = cpp_com_HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        core::mem::size_of::<CFolderOptions>(),
    ) as *mut CFolderOptions;
    if this.is_null() {
        return E_OUTOFMEMORY;
    }
    core::ptr::write(
        this,
        CFolderOptions {
            IShellPropSheetExt_iface: IShellPropSheetExt {
                lpVtbl: core::ptr::addr_of!(CFOLDEROPTIONS_ISHELLPROPSHEETEXT_VTBL),
            },
            IShellExtInit_iface: IShellExtInit {
                lpVtbl: core::ptr::addr_of!(CFOLDEROPTIONS_ISHELLEXTINIT_VTBL),
            },
            IObjectWithSite_iface: IObjectWithSite {
                lpVtbl: core::ptr::addr_of!(CFOLDEROPTIONS_IOBJECTWITHSITE_VTBL),
            },
            ref_count: core::sync::atomic::AtomicI32::new(1),
            m_pSite: core::ptr::null_mut(),
        },
    );
    Shell32RustObjectLock();
    let hr = query_interface(this, riid, out);
    release(this);
    hr
}

unsafe fn reset_global_folder_settings() -> HRESULT {
    let mut settings = core::ptr::null_mut::<IGlobalFolderSettings>();
    let mut hr = CoCreateInstance(
        core::ptr::addr_of!(CLSID_GlobalFolderSettings),
        core::ptr::null_mut(),
        CLSCTX_INPROC_SERVER,
        core::ptr::addr_of!(IID_IGlobalFolderSettings),
        core::ptr::addr_of_mut!(settings).cast(),
    );
    if hr >= 0 {
        hr = ((*(*settings).lpVtbl).Set)(settings, core::ptr::null(), 0, 0);
        ((*(*settings).lpVtbl).Release)(settings);
    }
    hr
}

#[inline]
unsafe fn exec_reset_def_view_folder_settings(object: *mut IUnknown) -> HRESULT {
    IUnknown_Exec(
        (object).cast(),
        core::ptr::addr_of!(CGID_DefView),
        DVCMDID_RESET_DEFAULTFOLDER_SETTINGS,
        0,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    )
}

unsafe fn reset_def_view_folder_settings() -> HRESULT {
    let mut folder = core::ptr::null_mut::<IShellFolder>();
    let mut hr = SHGetDesktopFolder(core::ptr::addr_of_mut!(folder));
    if hr >= 0 {
        let create = SFV_CREATE {
            cbSize: core::mem::size_of::<SFV_CREATE>() as UINT,
            pshf: folder,
            psvOuter: core::ptr::null_mut(),
            psfvcb: core::ptr::null_mut(),
        };
        let mut view = core::ptr::null_mut::<IUnknown>();
        hr = SHCreateShellFolderView((core::ptr::addr_of!(create)).cast(), (core::ptr::addr_of_mut!(view)).cast());
        if hr >= 0 {
            hr = exec_reset_def_view_folder_settings(view);
            if !view.is_null() {
                ((*(*view).lpVtbl).Release)(view);
            }
        }
        if !folder.is_null() {
            ((*(*folder).lpVtbl).Release)(folder);
        }
    }
    hr
}

unsafe fn reset_global_and_def_view_folder_settings() -> HRESULT {
    // Match the source: the DefView result is intentionally ignored.
    let _ = reset_def_view_folder_settings();
    reset_global_folder_settings()
}

unsafe fn handle_def_folder_settings(site: *mut IUnknown, action: i32) -> HRESULT {
    let mut browser = core::ptr::null_mut::<IBrowserService2>();
    let mut hr = IUnknown_QueryService(
        (site).cast(),
        core::ptr::addr_of!(IID_IShellBrowser),
        core::ptr::addr_of!(IID_IBrowserService2),
        core::ptr::addr_of_mut!(browser).cast(),
    );
    if hr >= 0 {
        if action == DFSA_APPLY {
            hr = ((*(*browser).lpVtbl).SetAsDefFolderSettings)(browser);
        } else if action == DFSA_RESET {
            // The command result is intentionally ignored by the C++ source.
            let _ = exec_reset_def_view_folder_settings(browser.cast());
        }
        ((*(*browser).lpVtbl).Release)(browser);
    }

    if action == DFSA_RESET {
        // Force a DefView reset even when the browser hosts a third-party view.
        hr = reset_global_and_def_view_folder_settings();
    }
    hr
}

#[no_mangle]
pub unsafe extern "system" fn CFolderOptions_CanSetDefFolderSettings_RustBridge(
    object: *mut c_void,
) -> BOOL {
    if object.is_null() {
        return FALSE;
    }
    let this = object.cast::<CFolderOptions>();
    (handle_def_folder_settings((*this).m_pSite, DFSA_QUERY) >= 0) as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn CFolderOptions_ApplyDefFolderSettings_RustBridge(
    object: *mut c_void,
    reset_to_default: BOOL,
) -> HRESULT {
    if object.is_null() {
        return E_POINTER;
    }
    let this = object.cast::<CFolderOptions>();
    handle_def_folder_settings(
        (*this).m_pSite,
        if reset_to_default != FALSE {
            DFSA_RESET
        } else {
            DFSA_APPLY
        },
    )
}

#[no_mangle]
pub unsafe extern "system" fn CFolderOptions_ResetGlobalAndDefViewFolderSettings_RustBridge(
) -> HRESULT {
    reset_global_and_def_view_folder_settings()
}
