//! `CDropTargetHelper` COM object. Interface shapes, vtable order, pointer recovery,
//! QueryInterface, lifetime, and activation are emitted deterministically by rustconv.

use crate::shellcom::*;
// One declaration per external symbol: island.rs.
use crate::island::HeapAlloc;
use crate::island::IUnknown_SetSite;
use crate::island::SHSimulateDrop;
use crate::types::*;
use core::ffi::c_void;

// Deterministic from WIDL-generated shobjidl.h.
#[repr(C)]
pub struct IDragSourceHelper {
    pub lpVtbl: *const IDragSourceHelperVtbl,
}

#[repr(C)]
pub struct IDragSourceHelperVtbl {
    pub QueryInterface:
        unsafe extern "system" fn(*mut IDragSourceHelper, REFIID, *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(*mut IDragSourceHelper) -> ULONG,
    pub Release: unsafe extern "system" fn(*mut IDragSourceHelper) -> ULONG,
    pub InitializeFromBitmap: unsafe extern "system" fn(
        *mut IDragSourceHelper,
        LPSHDRAGIMAGE,
        *mut IDataObject,
    ) -> HRESULT,
    pub InitializeFromWindow: unsafe extern "system" fn(
        *mut IDragSourceHelper,
        HWND,
        *mut POINT,
        *mut IDataObject,
    ) -> HRESULT,
}

extern "C" {
    pub static IID_IDragSourceHelper: IID;
}

#[repr(C)]
pub struct IDropTargetHelper {
    pub lpVtbl: *const IDropTargetHelperVtbl,
}

#[repr(C)]
pub struct IDropTargetHelperVtbl {
    pub QueryInterface:
        unsafe extern "system" fn(*mut IDropTargetHelper, REFIID, *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(*mut IDropTargetHelper) -> ULONG,
    pub Release: unsafe extern "system" fn(*mut IDropTargetHelper) -> ULONG,
    pub DragEnter: unsafe extern "system" fn(
        *mut IDropTargetHelper,
        HWND,
        *mut IDataObject,
        *mut POINT,
        DWORD,
    ) -> HRESULT,
    pub DragLeave: unsafe extern "system" fn(*mut IDropTargetHelper) -> HRESULT,
    pub DragOver: unsafe extern "system" fn(*mut IDropTargetHelper, *mut POINT, DWORD) -> HRESULT,
    pub Drop: unsafe extern "system" fn(
        *mut IDropTargetHelper,
        *mut IDataObject,
        *mut POINT,
        DWORD,
    ) -> HRESULT,
    pub Show: unsafe extern "system" fn(*mut IDropTargetHelper, BOOL) -> HRESULT,
}

extern "C" {
    pub static IID_IDropTargetHelper: IID;
    pub static IID_IUnknown: IID;
}

extern "system" {
    fn GetProcessHeap() -> HANDLE;
    fn HeapFree(heap: HANDLE, flags: DWORD, memory: *mut c_void) -> BOOL;
    fn Shell32RustObjectLock();
    fn Shell32RustObjectUnlock();
    fn SH32_ExtCoCreateInstance(
        class_name: LPCWSTR,
        clsid: REFCLSID,
        outer: *mut IUnknown,
        class_context: DWORD,
        riid: REFIID,
        object: *mut *mut c_void,
    ) -> HRESULT;
}

const CLSCTX_ALL: DWORD = 0x17;

#[repr(C)]
pub struct CDropTargetHelper {
    pub IDragSourceHelper_iface: IDragSourceHelper,
    pub IDropTargetHelper_iface: IDropTargetHelper,
    pub ref_count: core::sync::atomic::AtomicI32,
}

unsafe fn object_from_drag_source(this: *mut IDragSourceHelper) -> *mut CDropTargetHelper {
    (this as *mut u8).sub(core::mem::offset_of!(
        CDropTargetHelper,
        IDragSourceHelper_iface
    )) as *mut CDropTargetHelper
}

unsafe fn object_from_drop_target(this: *mut IDropTargetHelper) -> *mut CDropTargetHelper {
    (this as *mut u8).sub(core::mem::offset_of!(
        CDropTargetHelper,
        IDropTargetHelper_iface
    )) as *mut CDropTargetHelper
}

unsafe extern "system" fn drag_source_QueryInterface(
    this: *mut IDragSourceHelper,
    riid: REFIID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    query_interface(object_from_drag_source(this), riid, ppv)
}

unsafe extern "system" fn drag_source_AddRef(this: *mut IDragSourceHelper) -> ULONG {
    add_ref(object_from_drag_source(this))
}

unsafe extern "system" fn drag_source_Release(this: *mut IDragSourceHelper) -> ULONG {
    release(object_from_drag_source(this))
}

unsafe extern "system" fn drag_source_InitializeFromBitmap(
    this: *mut IDragSourceHelper,
    pshdi: LPSHDRAGIMAGE,
    data_object: *mut IDataObject,
) -> HRESULT {
    CDropTargetHelper_InitializeFromBitmap_body(object_from_drag_source(this), pshdi, data_object)
}

unsafe extern "system" fn drag_source_InitializeFromWindow(
    this: *mut IDragSourceHelper,
    hwnd: HWND,
    point: *mut POINT,
    data_object: *mut IDataObject,
) -> HRESULT {
    CDropTargetHelper_InitializeFromWindow_body(
        object_from_drag_source(this),
        hwnd,
        point,
        data_object,
    )
}

pub static DRAG_SOURCE_VTBL: IDragSourceHelperVtbl = IDragSourceHelperVtbl {
    QueryInterface: drag_source_QueryInterface,
    AddRef: drag_source_AddRef,
    Release: drag_source_Release,
    InitializeFromBitmap: drag_source_InitializeFromBitmap,
    InitializeFromWindow: drag_source_InitializeFromWindow,
};

unsafe extern "system" fn drop_target_QueryInterface(
    this: *mut IDropTargetHelper,
    riid: REFIID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    query_interface(object_from_drop_target(this), riid, ppv)
}

unsafe extern "system" fn drop_target_AddRef(this: *mut IDropTargetHelper) -> ULONG {
    add_ref(object_from_drop_target(this))
}

unsafe extern "system" fn drop_target_Release(this: *mut IDropTargetHelper) -> ULONG {
    release(object_from_drop_target(this))
}

unsafe extern "system" fn drop_target_DragEnter(
    this: *mut IDropTargetHelper,
    hwnd: HWND,
    data_object: *mut IDataObject,
    point: *mut POINT,
    effect: DWORD,
) -> HRESULT {
    CDropTargetHelper_DragEnter_body(
        object_from_drop_target(this),
        hwnd,
        data_object,
        point,
        effect,
    )
}

unsafe extern "system" fn drop_target_DragLeave(this: *mut IDropTargetHelper) -> HRESULT {
    CDropTargetHelper_DragLeave_body(object_from_drop_target(this))
}

unsafe extern "system" fn drop_target_DragOver(
    this: *mut IDropTargetHelper,
    point: *mut POINT,
    effect: DWORD,
) -> HRESULT {
    CDropTargetHelper_DragOver_body(object_from_drop_target(this), point, effect)
}

unsafe extern "system" fn drop_target_Drop(
    this: *mut IDropTargetHelper,
    data_object: *mut IDataObject,
    point: *mut POINT,
    effect: DWORD,
) -> HRESULT {
    CDropTargetHelper_Drop_body(object_from_drop_target(this), data_object, point, effect)
}

unsafe extern "system" fn drop_target_Show(this: *mut IDropTargetHelper, show: BOOL) -> HRESULT {
    CDropTargetHelper_Show_body(object_from_drop_target(this), show)
}

pub static DROP_TARGET_VTBL: IDropTargetHelperVtbl = IDropTargetHelperVtbl {
    QueryInterface: drop_target_QueryInterface,
    AddRef: drop_target_AddRef,
    Release: drop_target_Release,
    DragEnter: drop_target_DragEnter,
    DragLeave: drop_target_DragLeave,
    DragOver: drop_target_DragOver,
    Drop: drop_target_Drop,
    Show: drop_target_Show,
};

#[inline]
unsafe fn iid_equal(left: REFIID, right: REFIID) -> bool {
    !left.is_null()
        && !right.is_null()
        && (*left).Data1 == (*right).Data1
        && (*left).Data2 == (*right).Data2
        && (*left).Data3 == (*right).Data3
        && (*left).Data4 == (*right).Data4
}

unsafe fn query_interface(
    this: *mut CDropTargetHelper,
    riid: REFIID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    if ppv.is_null() {
        return E_POINTER;
    }
    *ppv = core::ptr::null_mut();
    if iid_equal(riid, core::ptr::addr_of!(IID_IUnknown))
        || iid_equal(riid, core::ptr::addr_of!(IID_IDragSourceHelper))
    {
        *ppv = core::ptr::addr_of_mut!((*this).IDragSourceHelper_iface) as *mut c_void;
    } else if iid_equal(riid, core::ptr::addr_of!(IID_IDropTargetHelper)) {
        *ppv = core::ptr::addr_of_mut!((*this).IDropTargetHelper_iface) as *mut c_void;
    } else {
        return E_NOINTERFACE;
    }
    add_ref(this);
    S_OK
}

unsafe fn add_ref(this: *mut CDropTargetHelper) -> ULONG {
    (*this)
        .ref_count
        .fetch_add(1, core::sync::atomic::Ordering::SeqCst) as ULONG
        + 1
}

unsafe fn release(this: *mut CDropTargetHelper) -> ULONG {
    let refs = (*this)
        .ref_count
        .fetch_sub(1, core::sync::atomic::Ordering::SeqCst)
        - 1;
    if refs == 0 {
        CDropTargetHelper_dtor_body(this);
        Shell32RustObjectUnlock();
        HeapFree(GetProcessHeap(), 0, this as *mut c_void);
    }
    refs as ULONG
}

#[no_mangle]
pub unsafe extern "system" fn CDropTargetHelper_CreateInstance(
    outer: *mut IUnknown,
    riid: REFIID,
    ppv: *mut *mut c_void,
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
        (core::mem::size_of::<CDropTargetHelper>()) as _,
    ) as *mut CDropTargetHelper;
    if this.is_null() {
        return E_OUTOFMEMORY;
    }
    core::ptr::write(
        this,
        CDropTargetHelper {
            IDragSourceHelper_iface: IDragSourceHelper {
                lpVtbl: core::ptr::addr_of!(DRAG_SOURCE_VTBL),
            },
            IDropTargetHelper_iface: IDropTargetHelper {
                lpVtbl: core::ptr::addr_of!(DROP_TARGET_VTBL),
            },
            ref_count: core::sync::atomic::AtomicI32::new(1),
        },
    );
    Shell32RustObjectLock();
    CDropTargetHelper_ctor_body(this);
    let hr = query_interface(this, riid, ppv);
    release(this);
    hr
}

/// Mirrors CScopedSetObjectWithSite around the shlwapi drop helper.
unsafe fn simulate_drop_with_site(
    drop_target: *mut IDropTarget,
    data_object: *mut IDataObject,
    key_state: DWORD,
    point: *mut POINTL,
    effect: *mut DWORD,
    site: *mut IUnknown,
) -> HRESULT {
    let guarded_object = if site.is_null() {
        core::ptr::null_mut()
    } else {
        let object = drop_target.cast::<IUnknown>();
        IUnknown_SetSite((object).cast(), (site).cast());
        object
    };

    let result = SHSimulateDrop((drop_target).cast(), (data_object).cast(), key_state, point, effect);
    IUnknown_SetSite((guarded_object).cast(), core::ptr::null_mut());
    result
}

/// Undocumented shell32 ordinal 751.
#[no_mangle]
pub unsafe extern "system" fn SHSimulateDropOnClsid(
    clsid: REFCLSID,
    site: *mut IUnknown,
    data_object: *mut IDataObject,
) -> HRESULT {
    let mut drop_target: *mut IDropTarget = core::ptr::null_mut();
    let result = SH32_ExtCoCreateInstance(
        core::ptr::null(),
        clsid,
        core::ptr::null_mut(),
        CLSCTX_ALL,
        core::ptr::addr_of!(IID_IDropTarget),
        core::ptr::addr_of_mut!(drop_target).cast(),
    );
    if result < 0 {
        return result;
    }

    let result = simulate_drop_with_site(
        drop_target,
        data_object,
        0,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        site,
    );
    ((*(*drop_target).lpVtbl).Release)(drop_target);
    result
}

pub unsafe fn CDropTargetHelper_ctor_body(_this: *mut CDropTargetHelper) {}

pub unsafe fn CDropTargetHelper_dtor_body(_this: *mut CDropTargetHelper) {}

pub unsafe fn CDropTargetHelper_InitializeFromBitmap_body(
    _this: *mut CDropTargetHelper,
    _pshdi: LPSHDRAGIMAGE,
    _data_object: *mut IDataObject,
) -> HRESULT {
    FIXME!("CDropTargetHelper::InitializeFromBitmap");
    E_NOTIMPL
}

pub unsafe fn CDropTargetHelper_InitializeFromWindow_body(
    _this: *mut CDropTargetHelper,
    _hwnd: HWND,
    _point: *mut POINT,
    _data_object: *mut IDataObject,
) -> HRESULT {
    FIXME!("CDropTargetHelper::InitializeFromWindow");
    E_NOTIMPL
}

pub unsafe fn CDropTargetHelper_DragEnter_body(
    _this: *mut CDropTargetHelper,
    _hwnd: HWND,
    _data_object: *mut IDataObject,
    _point: *mut POINT,
    _effect: DWORD,
) -> HRESULT {
    FIXME!("CDropTargetHelper::DragEnter");
    E_NOTIMPL
}

pub unsafe fn CDropTargetHelper_DragLeave_body(_this: *mut CDropTargetHelper) -> HRESULT {
    FIXME!("CDropTargetHelper::DragLeave");
    E_NOTIMPL
}

pub unsafe fn CDropTargetHelper_DragOver_body(
    _this: *mut CDropTargetHelper,
    _point: *mut POINT,
    _effect: DWORD,
) -> HRESULT {
    FIXME!("CDropTargetHelper::DragOver");
    E_NOTIMPL
}

pub unsafe fn CDropTargetHelper_Drop_body(
    _this: *mut CDropTargetHelper,
    _data_object: *mut IDataObject,
    _point: *mut POINT,
    _effect: DWORD,
) -> HRESULT {
    FIXME!("CDropTargetHelper::Drop");
    E_NOTIMPL
}

pub unsafe fn CDropTargetHelper_Show_body(_this: *mut CDropTargetHelper, _show: BOOL) -> HRESULT {
    FIXME!("CDropTargetHelper::Show");
    E_NOTIMPL
}
