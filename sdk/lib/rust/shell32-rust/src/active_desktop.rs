// Deterministic from shlobj.h
#[repr(C)]
pub struct IActiveDesktop {
    pub lpVtbl: *const IActiveDesktopVtbl,
}
#[repr(C)]
pub struct IActiveDesktopVtbl {
    pub QueryInterface: unsafe extern "system" fn(
        This: *mut IActiveDesktop,
        riid: REFIID,
        ppv: *mut *mut core::ffi::c_void,
    ) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IActiveDesktop) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IActiveDesktop) -> ULONG,
    pub ApplyChanges:
        unsafe extern "system" fn(This: *mut IActiveDesktop, dwFlags: DWORD) -> HRESULT,
    pub GetWallpaper: unsafe extern "system" fn(
        This: *mut IActiveDesktop,
        pwszWallpaper: PWSTR,
        cchWallpaper: UINT,
        dwFlags: DWORD,
    ) -> HRESULT,
    pub SetWallpaper: unsafe extern "system" fn(
        This: *mut IActiveDesktop,
        pwszWallpaper: PCWSTR,
        dwReserved: DWORD,
    ) -> HRESULT,
    pub GetWallpaperOptions: unsafe extern "system" fn(
        This: *mut IActiveDesktop,
        pwpo: LPWALLPAPEROPT,
        dwReserved: DWORD,
    ) -> HRESULT,
    pub SetWallpaperOptions: unsafe extern "system" fn(
        This: *mut IActiveDesktop,
        pwpo: LPCWALLPAPEROPT,
        dwReserved: DWORD,
    ) -> HRESULT,
    pub GetPattern: unsafe extern "system" fn(
        This: *mut IActiveDesktop,
        pwszPattern: PWSTR,
        cchPattern: UINT,
        dwReserved: DWORD,
    ) -> HRESULT,
    pub SetPattern: unsafe extern "system" fn(
        This: *mut IActiveDesktop,
        pwszPattern: PCWSTR,
        dwReserved: DWORD,
    ) -> HRESULT,
    pub GetDesktopItemOptions: unsafe extern "system" fn(
        This: *mut IActiveDesktop,
        pco: LPCOMPONENTSOPT,
        dwReserved: DWORD,
    ) -> HRESULT,
    pub SetDesktopItemOptions: unsafe extern "system" fn(
        This: *mut IActiveDesktop,
        pco: LPCCOMPONENTSOPT,
        dwReserved: DWORD,
    ) -> HRESULT,
    pub AddDesktopItem: unsafe extern "system" fn(
        This: *mut IActiveDesktop,
        pcomp: LPCCOMPONENT,
        dwReserved: DWORD,
    ) -> HRESULT,
    pub AddDesktopItemWithUI: unsafe extern "system" fn(
        This: *mut IActiveDesktop,
        hwnd: HWND,
        pcomp: LPCOMPONENT,
        dwReserved: DWORD,
    ) -> HRESULT,
    pub ModifyDesktopItem: unsafe extern "system" fn(
        This: *mut IActiveDesktop,
        pcomp: LPCCOMPONENT,
        dwFlags: DWORD,
    ) -> HRESULT,
    pub RemoveDesktopItem: unsafe extern "system" fn(
        This: *mut IActiveDesktop,
        pcomp: LPCCOMPONENT,
        dwReserved: DWORD,
    ) -> HRESULT,
    pub GetDesktopItemCount: unsafe extern "system" fn(
        This: *mut IActiveDesktop,
        pcItems: *mut i32,
        dwReserved: DWORD,
    ) -> HRESULT,
    pub GetDesktopItem: unsafe extern "system" fn(
        This: *mut IActiveDesktop,
        nComponent: i32,
        pcomp: LPCOMPONENT,
        dwReserved: DWORD,
    ) -> HRESULT,
    pub GetDesktopItemByID: unsafe extern "system" fn(
        This: *mut IActiveDesktop,
        dwID: ULONG_PTR,
        pcomp: LPCOMPONENT,
        dwReserved: DWORD,
    ) -> HRESULT,
    pub GenerateDesktopItemHtml: unsafe extern "system" fn(
        This: *mut IActiveDesktop,
        pwszFileName: PCWSTR,
        pcomp: LPCOMPONENT,
        dwReserved: DWORD,
    ) -> HRESULT,
    pub AddUrl: unsafe extern "system" fn(
        This: *mut IActiveDesktop,
        hwnd: HWND,
        pszSource: PCWSTR,
        pcomp: LPCOMPONENT,
        dwFlags: DWORD,
    ) -> HRESULT,
    pub GetDesktopItemBySource: unsafe extern "system" fn(
        This: *mut IActiveDesktop,
        pwszSource: PCWSTR,
        pcomp: LPCOMPONENT,
        dwReserved: DWORD,
    ) -> HRESULT,
}
extern "C" {
    pub static IID_IActiveDesktop: IID;
}

// Deterministic from oaidl.h
#[repr(C)]
pub struct IPropertyBag {
    pub lpVtbl: *const IPropertyBagVtbl,
}
#[repr(C)]
pub struct IPropertyBagVtbl {
    pub QueryInterface: unsafe extern "system" fn(
        This: *mut IPropertyBag,
        riid: REFIID,
        ppvObject: *mut *mut core::ffi::c_void,
    ) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IPropertyBag) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IPropertyBag) -> ULONG,
    pub Read: unsafe extern "system" fn(
        This: *mut IPropertyBag,
        pszPropName: LPCOLESTR,
        pVar: *mut VARIANT,
        pErrorLog: *mut IErrorLog,
    ) -> HRESULT,
    pub Write: unsafe extern "system" fn(
        This: *mut IPropertyBag,
        pszPropName: LPCOLESTR,
        pVar: *mut VARIANT,
    ) -> HRESULT,
}
extern "C" {
    pub static IID_IPropertyBag: IID;
}

// Deterministic COM scaffold for CActiveDesktop; method bodies are generated separately.
extern "system" {
    fn GetProcessHeap() -> HANDLE;
    fn HeapFree(heap: HANDLE, flags: DWORD, memory: *mut core::ffi::c_void) -> BOOL;
    fn Shell32RustObjectLock();
    fn Shell32RustObjectUnlock();
}
extern "C" {
    pub static IID_IUnknown: IID;
}

#[repr(C)]
pub struct CActiveDesktop {
    pub IActiveDesktop_iface: IActiveDesktop,
    pub IPropertyBag_iface: IPropertyBag,
    pub ref_count: core::sync::atomic::AtomicI32,
}

unsafe extern "system" fn CActiveDesktop_IActiveDesktop_QueryInterface(
    This: *mut IActiveDesktop,
    riid: REFIID,
    ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CActiveDesktop, IActiveDesktop_iface))
        as *mut CActiveDesktop;
    CActiveDesktop_QueryInterface(this, riid, ppv)
}
unsafe extern "system" fn CActiveDesktop_IActiveDesktop_AddRef(This: *mut IActiveDesktop) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CActiveDesktop, IActiveDesktop_iface))
        as *mut CActiveDesktop;
    CActiveDesktop_AddRef(this)
}
unsafe extern "system" fn CActiveDesktop_IActiveDesktop_Release(
    This: *mut IActiveDesktop,
) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CActiveDesktop, IActiveDesktop_iface))
        as *mut CActiveDesktop;
    CActiveDesktop_Release(this)
}
unsafe extern "system" fn CActiveDesktop_IActiveDesktop_ApplyChanges(
    This: *mut IActiveDesktop,
    dwFlags: DWORD,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CActiveDesktop, IActiveDesktop_iface))
        as *mut CActiveDesktop;
    CActiveDesktop_ApplyChanges_body(this, dwFlags)
}
unsafe extern "system" fn CActiveDesktop_IActiveDesktop_GetWallpaper(
    This: *mut IActiveDesktop,
    pwszWallpaper: PWSTR,
    cchWallpaper: UINT,
    dwFlags: DWORD,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CActiveDesktop, IActiveDesktop_iface))
        as *mut CActiveDesktop;
    CActiveDesktop_GetWallpaper_body(this, pwszWallpaper, cchWallpaper, dwFlags)
}
unsafe extern "system" fn CActiveDesktop_IActiveDesktop_SetWallpaper(
    This: *mut IActiveDesktop,
    pwszWallpaper: PCWSTR,
    dwReserved: DWORD,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CActiveDesktop, IActiveDesktop_iface))
        as *mut CActiveDesktop;
    CActiveDesktop_SetWallpaper_body(this, pwszWallpaper, dwReserved)
}
unsafe extern "system" fn CActiveDesktop_IActiveDesktop_GetWallpaperOptions(
    This: *mut IActiveDesktop,
    pwpo: LPWALLPAPEROPT,
    dwReserved: DWORD,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CActiveDesktop, IActiveDesktop_iface))
        as *mut CActiveDesktop;
    CActiveDesktop_GetWallpaperOptions_body(this, pwpo, dwReserved)
}
unsafe extern "system" fn CActiveDesktop_IActiveDesktop_SetWallpaperOptions(
    This: *mut IActiveDesktop,
    pwpo: LPCWALLPAPEROPT,
    dwReserved: DWORD,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CActiveDesktop, IActiveDesktop_iface))
        as *mut CActiveDesktop;
    CActiveDesktop_SetWallpaperOptions_body(this, pwpo, dwReserved)
}
unsafe extern "system" fn CActiveDesktop_IActiveDesktop_GetPattern(
    This: *mut IActiveDesktop,
    pwszPattern: PWSTR,
    cchPattern: UINT,
    dwReserved: DWORD,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CActiveDesktop, IActiveDesktop_iface))
        as *mut CActiveDesktop;
    CActiveDesktop_GetPattern_body(this, pwszPattern, cchPattern, dwReserved)
}
unsafe extern "system" fn CActiveDesktop_IActiveDesktop_SetPattern(
    This: *mut IActiveDesktop,
    pwszPattern: PCWSTR,
    dwReserved: DWORD,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CActiveDesktop, IActiveDesktop_iface))
        as *mut CActiveDesktop;
    CActiveDesktop_SetPattern_body(this, pwszPattern, dwReserved)
}
unsafe extern "system" fn CActiveDesktop_IActiveDesktop_GetDesktopItemOptions(
    This: *mut IActiveDesktop,
    pco: LPCOMPONENTSOPT,
    dwReserved: DWORD,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CActiveDesktop, IActiveDesktop_iface))
        as *mut CActiveDesktop;
    CActiveDesktop_GetDesktopItemOptions_body(this, pco, dwReserved)
}
unsafe extern "system" fn CActiveDesktop_IActiveDesktop_SetDesktopItemOptions(
    This: *mut IActiveDesktop,
    pco: LPCCOMPONENTSOPT,
    dwReserved: DWORD,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CActiveDesktop, IActiveDesktop_iface))
        as *mut CActiveDesktop;
    CActiveDesktop_SetDesktopItemOptions_body(this, pco, dwReserved)
}
unsafe extern "system" fn CActiveDesktop_IActiveDesktop_AddDesktopItem(
    This: *mut IActiveDesktop,
    pcomp: LPCCOMPONENT,
    dwReserved: DWORD,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CActiveDesktop, IActiveDesktop_iface))
        as *mut CActiveDesktop;
    CActiveDesktop_AddDesktopItem_body(this, pcomp, dwReserved)
}
unsafe extern "system" fn CActiveDesktop_IActiveDesktop_AddDesktopItemWithUI(
    This: *mut IActiveDesktop,
    hwnd: HWND,
    pcomp: LPCOMPONENT,
    dwReserved: DWORD,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CActiveDesktop, IActiveDesktop_iface))
        as *mut CActiveDesktop;
    CActiveDesktop_AddDesktopItemWithUI_body(this, hwnd, pcomp, dwReserved)
}
unsafe extern "system" fn CActiveDesktop_IActiveDesktop_ModifyDesktopItem(
    This: *mut IActiveDesktop,
    pcomp: LPCCOMPONENT,
    dwFlags: DWORD,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CActiveDesktop, IActiveDesktop_iface))
        as *mut CActiveDesktop;
    CActiveDesktop_ModifyDesktopItem_body(this, pcomp, dwFlags)
}
unsafe extern "system" fn CActiveDesktop_IActiveDesktop_RemoveDesktopItem(
    This: *mut IActiveDesktop,
    pcomp: LPCCOMPONENT,
    dwReserved: DWORD,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CActiveDesktop, IActiveDesktop_iface))
        as *mut CActiveDesktop;
    CActiveDesktop_RemoveDesktopItem_body(this, pcomp, dwReserved)
}
unsafe extern "system" fn CActiveDesktop_IActiveDesktop_GetDesktopItemCount(
    This: *mut IActiveDesktop,
    pcItems: *mut i32,
    dwReserved: DWORD,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CActiveDesktop, IActiveDesktop_iface))
        as *mut CActiveDesktop;
    CActiveDesktop_GetDesktopItemCount_body(this, pcItems, dwReserved)
}
unsafe extern "system" fn CActiveDesktop_IActiveDesktop_GetDesktopItem(
    This: *mut IActiveDesktop,
    nComponent: i32,
    pcomp: LPCOMPONENT,
    dwReserved: DWORD,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CActiveDesktop, IActiveDesktop_iface))
        as *mut CActiveDesktop;
    CActiveDesktop_GetDesktopItem_body(this, nComponent, pcomp, dwReserved)
}
unsafe extern "system" fn CActiveDesktop_IActiveDesktop_GetDesktopItemByID(
    This: *mut IActiveDesktop,
    dwID: ULONG_PTR,
    pcomp: LPCOMPONENT,
    dwReserved: DWORD,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CActiveDesktop, IActiveDesktop_iface))
        as *mut CActiveDesktop;
    CActiveDesktop_GetDesktopItemByID_body(this, dwID, pcomp, dwReserved)
}
unsafe extern "system" fn CActiveDesktop_IActiveDesktop_GenerateDesktopItemHtml(
    This: *mut IActiveDesktop,
    pwszFileName: PCWSTR,
    pcomp: LPCOMPONENT,
    dwReserved: DWORD,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CActiveDesktop, IActiveDesktop_iface))
        as *mut CActiveDesktop;
    CActiveDesktop_GenerateDesktopItemHtml_body(this, pwszFileName, pcomp, dwReserved)
}
unsafe extern "system" fn CActiveDesktop_IActiveDesktop_AddUrl(
    This: *mut IActiveDesktop,
    hwnd: HWND,
    pszSource: PCWSTR,
    pcomp: LPCOMPONENT,
    dwFlags: DWORD,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CActiveDesktop, IActiveDesktop_iface))
        as *mut CActiveDesktop;
    CActiveDesktop_AddUrl_body(this, hwnd, pszSource, pcomp, dwFlags)
}
unsafe extern "system" fn CActiveDesktop_IActiveDesktop_GetDesktopItemBySource(
    This: *mut IActiveDesktop,
    pwszSource: PCWSTR,
    pcomp: LPCOMPONENT,
    dwReserved: DWORD,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CActiveDesktop, IActiveDesktop_iface))
        as *mut CActiveDesktop;
    CActiveDesktop_GetDesktopItemBySource_body(this, pwszSource, pcomp, dwReserved)
}

pub static CACTIVEDESKTOP_IACTIVEDESKTOP_VTBL: IActiveDesktopVtbl = IActiveDesktopVtbl {
    QueryInterface: CActiveDesktop_IActiveDesktop_QueryInterface,
    AddRef: CActiveDesktop_IActiveDesktop_AddRef,
    Release: CActiveDesktop_IActiveDesktop_Release,
    ApplyChanges: CActiveDesktop_IActiveDesktop_ApplyChanges,
    GetWallpaper: CActiveDesktop_IActiveDesktop_GetWallpaper,
    SetWallpaper: CActiveDesktop_IActiveDesktop_SetWallpaper,
    GetWallpaperOptions: CActiveDesktop_IActiveDesktop_GetWallpaperOptions,
    SetWallpaperOptions: CActiveDesktop_IActiveDesktop_SetWallpaperOptions,
    GetPattern: CActiveDesktop_IActiveDesktop_GetPattern,
    SetPattern: CActiveDesktop_IActiveDesktop_SetPattern,
    GetDesktopItemOptions: CActiveDesktop_IActiveDesktop_GetDesktopItemOptions,
    SetDesktopItemOptions: CActiveDesktop_IActiveDesktop_SetDesktopItemOptions,
    AddDesktopItem: CActiveDesktop_IActiveDesktop_AddDesktopItem,
    AddDesktopItemWithUI: CActiveDesktop_IActiveDesktop_AddDesktopItemWithUI,
    ModifyDesktopItem: CActiveDesktop_IActiveDesktop_ModifyDesktopItem,
    RemoveDesktopItem: CActiveDesktop_IActiveDesktop_RemoveDesktopItem,
    GetDesktopItemCount: CActiveDesktop_IActiveDesktop_GetDesktopItemCount,
    GetDesktopItem: CActiveDesktop_IActiveDesktop_GetDesktopItem,
    GetDesktopItemByID: CActiveDesktop_IActiveDesktop_GetDesktopItemByID,
    GenerateDesktopItemHtml: CActiveDesktop_IActiveDesktop_GenerateDesktopItemHtml,
    AddUrl: CActiveDesktop_IActiveDesktop_AddUrl,
    GetDesktopItemBySource: CActiveDesktop_IActiveDesktop_GetDesktopItemBySource,
};

unsafe extern "system" fn CActiveDesktop_IPropertyBag_QueryInterface(
    This: *mut IPropertyBag,
    riid: REFIID,
    ppvObject: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CActiveDesktop, IPropertyBag_iface))
        as *mut CActiveDesktop;
    CActiveDesktop_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CActiveDesktop_IPropertyBag_AddRef(This: *mut IPropertyBag) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CActiveDesktop, IPropertyBag_iface))
        as *mut CActiveDesktop;
    CActiveDesktop_AddRef(this)
}
unsafe extern "system" fn CActiveDesktop_IPropertyBag_Release(This: *mut IPropertyBag) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CActiveDesktop, IPropertyBag_iface))
        as *mut CActiveDesktop;
    CActiveDesktop_Release(this)
}
unsafe extern "system" fn CActiveDesktop_IPropertyBag_Read(
    This: *mut IPropertyBag,
    pszPropName: LPCOLESTR,
    pVar: *mut VARIANT,
    pErrorLog: *mut IErrorLog,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CActiveDesktop, IPropertyBag_iface))
        as *mut CActiveDesktop;
    CActiveDesktop_Read_body(this, pszPropName, pVar, pErrorLog)
}
unsafe extern "system" fn CActiveDesktop_IPropertyBag_Write(
    This: *mut IPropertyBag,
    pszPropName: LPCOLESTR,
    pVar: *mut VARIANT,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CActiveDesktop, IPropertyBag_iface))
        as *mut CActiveDesktop;
    CActiveDesktop_Write_body(this, pszPropName, pVar)
}

pub static CACTIVEDESKTOP_IPROPERTYBAG_VTBL: IPropertyBagVtbl = IPropertyBagVtbl {
    QueryInterface: CActiveDesktop_IPropertyBag_QueryInterface,
    AddRef: CActiveDesktop_IPropertyBag_AddRef,
    Release: CActiveDesktop_IPropertyBag_Release,
    Read: CActiveDesktop_IPropertyBag_Read,
    Write: CActiveDesktop_IPropertyBag_Write,
};

unsafe fn CActiveDesktop_QueryInterface(
    this: *mut CActiveDesktop,
    riid: REFIID,
    ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() {
        return E_POINTER;
    }
    *ppv = core::ptr::null_mut();
    if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IUnknown))
        || cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IActiveDesktop))
    {
        *ppv = core::ptr::addr_of_mut!((*this).IActiveDesktop_iface) as *mut core::ffi::c_void;
    } else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IPropertyBag)) {
        *ppv = core::ptr::addr_of_mut!((*this).IPropertyBag_iface) as *mut core::ffi::c_void;
    } else {
        return E_NOINTERFACE;
    }
    CActiveDesktop_AddRef(this);
    S_OK
}

unsafe fn CActiveDesktop_AddRef(this: *mut CActiveDesktop) -> ULONG {
    (*this)
        .ref_count
        .fetch_add(1, core::sync::atomic::Ordering::SeqCst) as ULONG
        + 1
}

unsafe fn CActiveDesktop_Release(this: *mut CActiveDesktop) -> ULONG {
    let refs = (*this)
        .ref_count
        .fetch_sub(1, core::sync::atomic::Ordering::SeqCst)
        - 1;
    if refs == 0 {
        CActiveDesktop_dtor_body(this);
        Shell32RustObjectUnlock();
        HeapFree(GetProcessHeap(), 0, this as *mut core::ffi::c_void);
    }
    refs as ULONG
}

#[inline]
unsafe fn cpp_com_iid_equal(left: REFIID, right: REFIID) -> bool {
    !left.is_null()
        && !right.is_null()
        && (*left).Data1 == (*right).Data1
        && (*left).Data2 == (*right).Data2
        && (*left).Data3 == (*right).Data3
        && (*left).Data4 == (*right).Data4
}

// Activation: CLSID_ActiveDesktop currently maps to CActiveDesktop in shell32.cpp.
// The generated C++ adapter redirects ATL activation to this factory.
#[no_mangle]
pub unsafe extern "system" fn CActiveDesktop_CreateInstance(
    outer: *mut IUnknown,
    riid: REFIID,
    ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() {
        return E_POINTER;
    }
    *ppv = core::ptr::null_mut();
    if !outer.is_null() {
        return CLASS_E_NOAGGREGATION;
    }
    let this = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        (core::mem::size_of::<CActiveDesktop>()) as _,
    ) as *mut CActiveDesktop;
    if this.is_null() {
        return E_OUTOFMEMORY;
    }
    core::ptr::write(
        this,
        CActiveDesktop {
            IActiveDesktop_iface: IActiveDesktop {
                lpVtbl: core::ptr::addr_of!(CACTIVEDESKTOP_IACTIVEDESKTOP_VTBL),
            },
            IPropertyBag_iface: IPropertyBag {
                lpVtbl: core::ptr::addr_of!(CACTIVEDESKTOP_IPROPERTYBAG_VTBL),
            },
            ref_count: core::sync::atomic::AtomicI32::new(1),
        },
    );
    Shell32RustObjectLock();
    CActiveDesktop_ctor_body(this);
    let hr = CActiveDesktop_QueryInterface(this, riid, ppv);
    CActiveDesktop_Release(this);
    hr
}

use crate::shellcom::*;
// One declaration per external symbol: island.rs.
use crate::island::HeapAlloc;
use crate::types::*;
use core::ffi::c_void;

pub unsafe fn CActiveDesktop_ctor_body(this: *mut CActiveDesktop) -> () {}

pub unsafe fn CActiveDesktop_dtor_body(this: *mut CActiveDesktop) -> () {}

pub unsafe fn CActiveDesktop_ApplyChanges_body(
    this: *mut CActiveDesktop,
    dwFlags: DWORD,
) -> HRESULT {
    E_NOTIMPL
}

pub unsafe fn CActiveDesktop_GetWallpaper_body(
    this: *mut CActiveDesktop,
    pwszWallpaper: PWSTR,
    cchWallpaper: UINT,
    dwFlags: DWORD,
) -> HRESULT {
    E_NOTIMPL
}

pub unsafe fn CActiveDesktop_SetWallpaper_body(
    this: *mut CActiveDesktop,
    pwszWallpaper: PCWSTR,
    dwReserved: DWORD,
) -> HRESULT {
    E_NOTIMPL
}

pub unsafe fn CActiveDesktop_GetWallpaperOptions_body(
    this: *mut CActiveDesktop,
    pwpo: LPWALLPAPEROPT,
    dwReserved: DWORD,
) -> HRESULT {
    E_NOTIMPL
}

pub unsafe fn CActiveDesktop_SetWallpaperOptions_body(
    this: *mut CActiveDesktop,
    pwpo: LPCWALLPAPEROPT,
    dwReserved: DWORD,
) -> HRESULT {
    E_NOTIMPL
}

pub unsafe fn CActiveDesktop_GetPattern_body(
    this: *mut CActiveDesktop,
    pwszPattern: PWSTR,
    cchPattern: UINT,
    dwReserved: DWORD,
) -> HRESULT {
    E_NOTIMPL
}

pub unsafe fn CActiveDesktop_SetPattern_body(
    this: *mut CActiveDesktop,
    pwszPattern: PCWSTR,
    dwReserved: DWORD,
) -> HRESULT {
    E_NOTIMPL
}

pub unsafe fn CActiveDesktop_GetDesktopItemOptions_body(
    this: *mut CActiveDesktop,
    pco: LPCOMPONENTSOPT,
    dwReserved: DWORD,
) -> HRESULT {
    E_NOTIMPL
}

pub unsafe fn CActiveDesktop_SetDesktopItemOptions_body(
    this: *mut CActiveDesktop,
    pco: LPCCOMPONENTSOPT,
    dwReserved: DWORD,
) -> HRESULT {
    E_NOTIMPL
}

pub unsafe fn CActiveDesktop_AddDesktopItem_body(
    this: *mut CActiveDesktop,
    pcomp: LPCCOMPONENT,
    dwReserved: DWORD,
) -> HRESULT {
    E_NOTIMPL
}

pub unsafe fn CActiveDesktop_AddDesktopItemWithUI_body(
    this: *mut CActiveDesktop,
    hwnd: HWND,
    pcomp: LPCOMPONENT,
    dwReserved: DWORD,
) -> HRESULT {
    E_NOTIMPL
}

pub unsafe fn CActiveDesktop_ModifyDesktopItem_body(
    this: *mut CActiveDesktop,
    pcomp: LPCCOMPONENT,
    dwFlags: DWORD,
) -> HRESULT {
    E_NOTIMPL
}

pub unsafe fn CActiveDesktop_RemoveDesktopItem_body(
    this: *mut CActiveDesktop,
    pcomp: LPCCOMPONENT,
    dwReserved: DWORD,
) -> HRESULT {
    E_NOTIMPL
}

pub unsafe fn CActiveDesktop_GetDesktopItemCount_body(
    this: *mut CActiveDesktop,
    pcItems: *mut i32,
    dwReserved: DWORD,
) -> HRESULT {
    E_NOTIMPL
}

pub unsafe fn CActiveDesktop_GetDesktopItem_body(
    this: *mut CActiveDesktop,
    nComponent: i32,
    pcomp: LPCOMPONENT,
    dwReserved: DWORD,
) -> HRESULT {
    E_NOTIMPL
}

pub unsafe fn CActiveDesktop_GetDesktopItemByID_body(
    this: *mut CActiveDesktop,
    dwID: ULONG_PTR,
    pcomp: LPCOMPONENT,
    dwReserved: DWORD,
) -> HRESULT {
    E_NOTIMPL
}

pub unsafe fn CActiveDesktop_GenerateDesktopItemHtml_body(
    this: *mut CActiveDesktop,
    pwszFileName: PCWSTR,
    pcomp: LPCOMPONENT,
    dwReserved: DWORD,
) -> HRESULT {
    E_NOTIMPL
}

pub unsafe fn CActiveDesktop_AddUrl_body(
    this: *mut CActiveDesktop,
    hwnd: HWND,
    pszSource: PCWSTR,
    pcomp: LPCOMPONENT,
    dwFlags: DWORD,
) -> HRESULT {
    E_NOTIMPL
}

pub unsafe fn CActiveDesktop_GetDesktopItemBySource_body(
    this: *mut CActiveDesktop,
    pwszSource: PCWSTR,
    pcomp: LPCOMPONENT,
    dwReserved: DWORD,
) -> HRESULT {
    E_NOTIMPL
}

pub unsafe fn CActiveDesktop_Read_body(
    this: *mut CActiveDesktop,
    pszPropName: LPCOLESTR,
    pVar: *mut VARIANT,
    pErrorLog: *mut IErrorLog,
) -> HRESULT {
    E_NOTIMPL
}

pub unsafe fn CActiveDesktop_Write_body(
    this: *mut CActiveDesktop,
    pszPropName: LPCOLESTR,
    pVar: *mut VARIANT,
) -> HRESULT {
    E_NOTIMPL
}
