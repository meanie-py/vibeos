//! Rust-owned `CFolderOptions` COM identity and state.
//!
//! Property-page callbacks and the private `IBrowserService2` operation stay behind
//! narrow C++ bridges: those surfaces depend on source-local dialog procedures and a
//! very large inherited vtable, while the object's public ABI is deterministic.

use crate::shellcom::*;
use crate::types::*;
use core::ffi::c_void;

pub type EXPPS = UINT;
pub type LPFNSVADDPROPSHEETPAGE =
    Option<unsafe extern "system" fn(HPROPSHEETPAGE, LPARAM) -> BOOL>;

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
}

extern "system" {
    fn GetProcessHeap() -> HANDLE;
    #[link_name = "HeapAlloc"]
    fn cpp_com_HeapAlloc(heap: HANDLE, flags: DWORD, bytes: usize) -> *mut c_void;
    #[link_name = "HeapFree"]
    fn cpp_com_HeapFree(heap: HANDLE, flags: DWORD, memory: *mut c_void) -> BOOL;
    fn Shell32RustObjectLock();
    fn Shell32RustObjectUnlock();

    fn CFolderOptions_AddPages_CppBridge(
        object: *mut IUnknown,
        add_page: LPFNSVADDPROPSHEETPAGE,
        parameter: LPARAM,
    ) -> HRESULT;
    fn CFolderOptions_HandleDefFolderSettings_CppBridge(
        site: *mut IUnknown,
        action: i32,
    ) -> HRESULT;
    fn CFolderOptions_ResetGlobalAndDefViewFolderSettings_CppBridge() -> HRESULT;
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
    CFolderOptions_AddPages_CppBridge(this.cast(), add_page, parameter)
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

#[no_mangle]
pub unsafe extern "system" fn CFolderOptions_CanSetDefFolderSettings_RustBridge(
    object: *mut c_void,
) -> BOOL {
    if object.is_null() {
        return FALSE;
    }
    let this = object.cast::<CFolderOptions>();
    (CFolderOptions_HandleDefFolderSettings_CppBridge((*this).m_pSite, DFSA_QUERY) >= 0) as BOOL
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
    CFolderOptions_HandleDefFolderSettings_CppBridge(
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
    CFolderOptions_ResetGlobalAndDefViewFolderSettings_CppBridge()
}
