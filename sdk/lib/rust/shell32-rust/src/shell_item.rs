// Deterministic from shobjidl.h
#[repr(C)]
pub struct IShellItem {
    pub lpVtbl: *const IShellItemVtbl,
}
#[repr(C)]
pub struct IShellItemVtbl {
    pub QueryInterface: unsafe extern "system" fn(
        This: *mut IShellItem,
        riid: REFIID,
        ppvObject: *mut *mut core::ffi::c_void,
    ) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IShellItem) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IShellItem) -> ULONG,
    pub BindToHandler: unsafe extern "system" fn(
        This: *mut IShellItem,
        pbc: *mut IBindCtx,
        rbhid: REFGUID,
        riid: REFIID,
        ppvOut: *mut *mut core::ffi::c_void,
    ) -> HRESULT,
    pub GetParent:
        unsafe extern "system" fn(This: *mut IShellItem, ppsi: *mut *mut IShellItem) -> HRESULT,
    pub GetDisplayName: unsafe extern "system" fn(
        This: *mut IShellItem,
        sigdnName: SIGDN,
        ppszName: *mut LPWSTR,
    ) -> HRESULT,
    pub GetAttributes: unsafe extern "system" fn(
        This: *mut IShellItem,
        sfgaoMask: SFGAOF,
        psfgaoAttribs: *mut SFGAOF,
    ) -> HRESULT,
    pub Compare: unsafe extern "system" fn(
        This: *mut IShellItem,
        psi: *mut IShellItem,
        hint: SICHINTF,
        piOrder: *mut i32,
    ) -> HRESULT,
}
extern "C" {
    pub static IID_IShellItem: IID;
}

// Deterministic from shobjidl.h
#[repr(C)]
pub struct IPersistIDList {
    pub lpVtbl: *const IPersistIDListVtbl,
}
#[repr(C)]
pub struct IPersistIDListVtbl {
    pub QueryInterface: unsafe extern "system" fn(
        This: *mut IPersistIDList,
        riid: REFIID,
        ppvObject: *mut *mut core::ffi::c_void,
    ) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IPersistIDList) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IPersistIDList) -> ULONG,
    pub GetClassID:
        unsafe extern "system" fn(This: *mut IPersistIDList, pClassID: *mut CLSID) -> HRESULT,
    pub SetIDList:
        unsafe extern "system" fn(This: *mut IPersistIDList, pidl: PCIDLIST_ABSOLUTE) -> HRESULT,
    pub GetIDList: unsafe extern "system" fn(
        This: *mut IPersistIDList,
        ppidl: *mut PIDLIST_ABSOLUTE,
    ) -> HRESULT,
}
extern "C" {
    pub static IID_IPersistIDList: IID;
}

// Deterministic COM scaffold for CShellItem; method bodies are generated separately.
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
pub struct CShellItem {
    pub IShellItem_iface: IShellItem,
    pub IPersistIDList_iface: IPersistIDList,
    pub ref_count: core::sync::atomic::AtomicI32,
    pub m_pidl: LPITEMIDLIST,
}

unsafe extern "system" fn CShellItem_IShellItem_QueryInterface(
    This: *mut IShellItem,
    riid: REFIID,
    ppvObject: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellItem, IShellItem_iface))
        as *mut CShellItem;
    CShellItem_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CShellItem_IShellItem_AddRef(This: *mut IShellItem) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellItem, IShellItem_iface))
        as *mut CShellItem;
    CShellItem_AddRef(this)
}
unsafe extern "system" fn CShellItem_IShellItem_Release(This: *mut IShellItem) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellItem, IShellItem_iface))
        as *mut CShellItem;
    CShellItem_Release(this)
}
unsafe extern "system" fn CShellItem_IShellItem_BindToHandler(
    This: *mut IShellItem,
    pbc: *mut IBindCtx,
    rbhid: REFGUID,
    riid: REFIID,
    ppvOut: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellItem, IShellItem_iface))
        as *mut CShellItem;
    CShellItem_BindToHandler_body(this, pbc, rbhid, riid, ppvOut)
}
unsafe extern "system" fn CShellItem_IShellItem_GetParent(
    This: *mut IShellItem,
    ppsi: *mut *mut IShellItem,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellItem, IShellItem_iface))
        as *mut CShellItem;
    CShellItem_GetParent_body(this, ppsi)
}
unsafe extern "system" fn CShellItem_IShellItem_GetDisplayName(
    This: *mut IShellItem,
    sigdnName: SIGDN,
    ppszName: *mut LPWSTR,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellItem, IShellItem_iface))
        as *mut CShellItem;
    CShellItem_GetDisplayName_body(this, sigdnName, ppszName)
}
unsafe extern "system" fn CShellItem_IShellItem_GetAttributes(
    This: *mut IShellItem,
    sfgaoMask: SFGAOF,
    psfgaoAttribs: *mut SFGAOF,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellItem, IShellItem_iface))
        as *mut CShellItem;
    CShellItem_GetAttributes_body(this, sfgaoMask, psfgaoAttribs)
}
unsafe extern "system" fn CShellItem_IShellItem_Compare(
    This: *mut IShellItem,
    psi: *mut IShellItem,
    hint: SICHINTF,
    piOrder: *mut i32,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellItem, IShellItem_iface))
        as *mut CShellItem;
    CShellItem_Compare_body(this, psi, hint, piOrder)
}

pub static CSHELLITEM_ISHELLITEM_VTBL: IShellItemVtbl = IShellItemVtbl {
    QueryInterface: CShellItem_IShellItem_QueryInterface,
    AddRef: CShellItem_IShellItem_AddRef,
    Release: CShellItem_IShellItem_Release,
    BindToHandler: CShellItem_IShellItem_BindToHandler,
    GetParent: CShellItem_IShellItem_GetParent,
    GetDisplayName: CShellItem_IShellItem_GetDisplayName,
    GetAttributes: CShellItem_IShellItem_GetAttributes,
    Compare: CShellItem_IShellItem_Compare,
};

unsafe extern "system" fn CShellItem_IPersistIDList_QueryInterface(
    This: *mut IPersistIDList,
    riid: REFIID,
    ppvObject: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellItem, IPersistIDList_iface))
        as *mut CShellItem;
    CShellItem_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CShellItem_IPersistIDList_AddRef(This: *mut IPersistIDList) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellItem, IPersistIDList_iface))
        as *mut CShellItem;
    CShellItem_AddRef(this)
}
unsafe extern "system" fn CShellItem_IPersistIDList_Release(This: *mut IPersistIDList) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellItem, IPersistIDList_iface))
        as *mut CShellItem;
    CShellItem_Release(this)
}
unsafe extern "system" fn CShellItem_IPersistIDList_GetClassID(
    This: *mut IPersistIDList,
    pClassID: *mut CLSID,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellItem, IPersistIDList_iface))
        as *mut CShellItem;
    CShellItem_GetClassID_body(this, pClassID)
}
unsafe extern "system" fn CShellItem_IPersistIDList_SetIDList(
    This: *mut IPersistIDList,
    pidl: PCIDLIST_ABSOLUTE,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellItem, IPersistIDList_iface))
        as *mut CShellItem;
    CShellItem_SetIDList_body(this, pidl)
}
unsafe extern "system" fn CShellItem_IPersistIDList_GetIDList(
    This: *mut IPersistIDList,
    ppidl: *mut PIDLIST_ABSOLUTE,
) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CShellItem, IPersistIDList_iface))
        as *mut CShellItem;
    CShellItem_GetIDList_body(this, ppidl)
}

pub static CSHELLITEM_IPERSISTIDLIST_VTBL: IPersistIDListVtbl = IPersistIDListVtbl {
    QueryInterface: CShellItem_IPersistIDList_QueryInterface,
    AddRef: CShellItem_IPersistIDList_AddRef,
    Release: CShellItem_IPersistIDList_Release,
    GetClassID: CShellItem_IPersistIDList_GetClassID,
    SetIDList: CShellItem_IPersistIDList_SetIDList,
    GetIDList: CShellItem_IPersistIDList_GetIDList,
};

unsafe fn CShellItem_QueryInterface(
    this: *mut CShellItem,
    riid: REFIID,
    ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() {
        return E_POINTER;
    }
    *ppv = core::ptr::null_mut();
    if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IUnknown))
        || cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IShellItem))
    {
        *ppv = core::ptr::addr_of_mut!((*this).IShellItem_iface) as *mut core::ffi::c_void;
    } else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IPersistIDList)) {
        *ppv = core::ptr::addr_of_mut!((*this).IPersistIDList_iface) as *mut core::ffi::c_void;
    } else {
        return E_NOINTERFACE;
    }
    CShellItem_AddRef(this);
    S_OK
}

unsafe fn CShellItem_AddRef(this: *mut CShellItem) -> ULONG {
    (*this)
        .ref_count
        .fetch_add(1, core::sync::atomic::Ordering::SeqCst) as ULONG
        + 1
}

unsafe fn CShellItem_Release(this: *mut CShellItem) -> ULONG {
    let refs = (*this)
        .ref_count
        .fetch_sub(1, core::sync::atomic::Ordering::SeqCst)
        - 1;
    if refs == 0 {
        CShellItem_dtor_body(this);
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

// Activation: CLSID_ShellItem currently maps to CShellItem in shell32.cpp.
// The generated C++ adapter redirects ATL activation to this factory.
#[no_mangle]
pub unsafe extern "system" fn CShellItem_CreateInstance(
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
        (core::mem::size_of::<CShellItem>()) as _,
    ) as *mut CShellItem;
    if this.is_null() {
        return E_OUTOFMEMORY;
    }
    core::ptr::write(
        this,
        CShellItem {
            IShellItem_iface: IShellItem {
                lpVtbl: core::ptr::addr_of!(CSHELLITEM_ISHELLITEM_VTBL),
            },
            IPersistIDList_iface: IPersistIDList {
                lpVtbl: core::ptr::addr_of!(CSHELLITEM_IPERSISTIDLIST_VTBL),
            },
            ref_count: core::sync::atomic::AtomicI32::new(1),
            m_pidl: core::mem::zeroed(),
        },
    );
    Shell32RustObjectLock();
    CShellItem_ctor_body(this);
    let hr = CShellItem_QueryInterface(this, riid, ppv);
    CShellItem_Release(this);
    hr
}

// Deterministic C++ support closure selected from source identifiers.

extern "system" {
    fn ILFindLastID(pidl: LPCITEMIDLIST) -> LPITEMIDLIST;
    fn ILRemoveLastID(pidl: LPITEMIDLIST) -> BOOL;
    fn SHGetNameFromIDList(pidl: PCIDLIST_ABSOLUTE, sigdn: SIGDN, name: *mut PWSTR) -> HRESULT;
}

extern "C" {
}

extern "C" {
    static BHID_DataObject: GUID;
    static BHID_SFObject: GUID;
    static BHID_SFUIObject: GUID;
    static BHID_SFViewObject: GUID;
    static CLSID_ShellItem: GUID;
    static IID_IDataObject: GUID;
    static IID_IPersistFolder2: GUID;
    static IID_IShellFolder: GUID;
}

pub struct CppComPtrGuard<T> {
    slot: *mut *mut T,
}
impl<T> CppComPtrGuard<T> {
    pub const fn new(slot: *mut *mut T) -> Self {
        Self { slot }
    }
}
impl<T> Drop for CppComPtrGuard<T> {
    fn drop(&mut self) {
        unsafe {
            let value = *self.slot;
            if !value.is_null() {
                let unknown = value as *mut IUnknown;
                ((*(*unknown).lpVtbl).Release)(unknown);
                *self.slot = core::ptr::null_mut();
            }
        }
    }
}

use crate::shellcom::*;
// The crate's own functions: the definition is the declaration.
use crate::wine::pidl::ILClone;
use crate::wine::pidl::ILCombine;
use crate::wine::pidl::ILFree;
use crate::desktop_folder_tu::SHGetDesktopFolder;
use crate::wine::pidl::_ILIsDesktop;
// One declaration per external symbol: island.rs.
use crate::island::HeapAlloc;
use crate::types::*;
use core::ffi::c_void;

pub unsafe fn CShellItem_ctor_body(this: *mut CShellItem) -> () {}

#[no_mangle]
pub unsafe extern "system" fn SHCreateShellItem(
    mut pidlParent: PCIDLIST_ABSOLUTE,
    psfParent: *mut IShellFolder,
    pidl: PCUITEMID_CHILD,
    ppsi: *mut *mut IShellItem,
) -> HRESULT {
    *ppsi = core::ptr::null_mut();

    if pidl.is_null() {
        return E_INVALIDARG;
    }

    let new_pidl: LPITEMIDLIST;
    if !pidlParent.is_null() || !psfParent.is_null() {
        let mut temp_parent: LPITEMIDLIST = core::ptr::null_mut();
        if pidlParent.is_null() {
            let mut ppf2_parent: *mut IPersistFolder2 = core::ptr::null_mut();
            let hr = ((*(*psfParent).lpVtbl).QueryInterface)(
                psfParent,
                core::ptr::addr_of!(IID_IPersistFolder2),
                core::ptr::addr_of_mut!(ppf2_parent).cast(),
            );
            if hr < 0 {
                if !ppf2_parent.is_null() {
                    ((*(*ppf2_parent).lpVtbl).Release)(ppf2_parent);
                }
                return E_NOINTERFACE;
            }

            let hr = ((*(*ppf2_parent).lpVtbl).GetCurFolder)(ppf2_parent, &mut temp_parent);
            ((*(*ppf2_parent).lpVtbl).Release)(ppf2_parent);
            if hr < 0 {
                return E_NOINTERFACE;
            }
            pidlParent = temp_parent;
        }

        new_pidl = ILCombine(pidlParent, pidl);
        ILFree(temp_parent);
        if new_pidl.is_null() {
            return E_OUTOFMEMORY;
        }
    } else {
        new_pidl = ILClone(pidl);
        if new_pidl.is_null() {
            return E_OUTOFMEMORY;
        }
    }

    let mut new_shell_item: *mut IShellItem = core::ptr::null_mut();
    let mut hr = CShellItem_CreateInstance(
        core::ptr::null_mut(),
        core::ptr::addr_of!(IID_IShellItem),
        core::ptr::addr_of_mut!(new_shell_item).cast(),
    );
    if hr < 0 {
        ILFree(new_pidl);
        return hr;
    }

    let mut new_persist_id_list: *mut IPersistIDList = core::ptr::null_mut();
    hr = ((*(*new_shell_item).lpVtbl).QueryInterface)(
        new_shell_item,
        core::ptr::addr_of!(IID_IPersistIDList),
        core::ptr::addr_of_mut!(new_persist_id_list).cast(),
    );
    if hr >= 0 {
        hr = ((*(*new_persist_id_list).lpVtbl).SetIDList)(new_persist_id_list, new_pidl);
    }
    ILFree(new_pidl);

    if !new_persist_id_list.is_null() {
        ((*(*new_persist_id_list).lpVtbl).Release)(new_persist_id_list);
    }
    if hr < 0 {
        ((*(*new_shell_item).lpVtbl).Release)(new_shell_item);
        return hr;
    }

    *ppsi = new_shell_item;
    hr
}

pub unsafe fn CShellItem_dtor_body(this: *mut CShellItem) -> () {
    ILFree((*this).m_pidl);
}

pub unsafe fn CShellItem_get_parent_pidl_body(
    this: *mut CShellItem,
    parent_pidl: *mut LPITEMIDLIST,
) -> HRESULT {
    let cloned = ILClone((*this).m_pidl);
    *parent_pidl = cloned;
    if cloned.is_null() {
        *parent_pidl = core::ptr::null_mut();
        return E_OUTOFMEMORY;
    }
    if ILRemoveLastID(cloned) != 0 {
        return S_OK;
    }
    ILFree(cloned);
    *parent_pidl = core::ptr::null_mut();
    E_INVALIDARG
}

pub unsafe fn CShellItem_get_parent_shellfolder_body(
    this: *mut CShellItem,
    ppsf: *mut *mut IShellFolder,
) -> HRESULT {
    let mut hr: HRESULT;
    let mut parent_pidl: LPITEMIDLIST = core::ptr::null_mut();

    hr = CShellItem_get_parent_pidl_body(this, &mut parent_pidl);
    if hr >= 0 {
        let mut desktop: *mut IShellFolder = core::ptr::null_mut();
        let _desktop_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(desktop));
        hr = SHGetDesktopFolder(&mut desktop as *mut *mut IShellFolder);
        if hr >= 0 {
            let riid = core::ptr::addr_of!(IID_IShellFolder);
            let ppv = ppsf.cast();
            hr = ((*(*desktop).lpVtbl).BindToObject)(
                desktop,
                parent_pidl,
                core::ptr::null_mut(),
                riid,
                ppv,
            );
        }
        ILFree(parent_pidl);
        if !desktop.is_null() {
            ((*(*desktop).lpVtbl).Release)(desktop);
            desktop = core::ptr::null_mut();
        }
    }

    hr
}

pub unsafe fn CShellItem_get_shellfolder_body(
    this: *mut CShellItem,
    pbc: *mut IBindCtx,
    riid: REFIID,
    ppvOut: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let mut psf: *mut IShellFolder = core::ptr::null_mut();
    let _psf_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(psf));
    let mut psfDesktop: *mut IShellFolder = core::ptr::null_mut();
    let _psfDesktop_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(psfDesktop));

    let ret = SHGetDesktopFolder((&mut psfDesktop) as *mut *mut IShellFolder);
    if ((ret) < 0) {
        return ret;
    }

    if _ILIsDesktop((*this).m_pidl) != 0 {
        if !psfDesktop.is_null() {
            ((*(*psfDesktop).lpVtbl).AddRef)(psfDesktop);
        }
        psf = psfDesktop;
    } else {
        let ret = ((*(*psfDesktop).lpVtbl).BindToObject)(
            psfDesktop,
            (*this).m_pidl,
            pbc,
            core::ptr::addr_of!(IID_IShellFolder),
            core::ptr::addr_of_mut!(psf).cast(),
        );
        if ((ret) < 0) {
            if !psfDesktop.is_null() {
                ((*(*psfDesktop).lpVtbl).Release)(psfDesktop);
                psfDesktop = core::ptr::null_mut();
            }
            return ret;
        }
    }

    let result = ((*(*psf).lpVtbl).QueryInterface)(psf, riid, ppvOut);

    if !psf.is_null() {
        ((*(*psf).lpVtbl).Release)(psf);
        psf = core::ptr::null_mut();
    }
    if !psfDesktop.is_null() {
        ((*(*psfDesktop).lpVtbl).Release)(psfDesktop);
        psfDesktop = core::ptr::null_mut();
    }

    result
}

pub unsafe fn CShellItem_BindToHandler_body(
    this: *mut CShellItem,
    pbc: *mut IBindCtx,
    rbhid: REFGUID,
    riid: REFIID,
    ppvOut: *mut *mut core::ffi::c_void,
) -> HRESULT {
    *ppvOut = core::ptr::null_mut();

    if cpp_com_iid_equal(rbhid, core::ptr::addr_of!(BHID_SFObject)) {
        return CShellItem_get_shellfolder_body(this, pbc, riid, ppvOut);
    } else if cpp_com_iid_equal(rbhid, core::ptr::addr_of!(BHID_SFUIObject)) {
        let mut psf_parent: *mut IShellFolder = core::ptr::null_mut();
        let _psf_parent_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(psf_parent));
        let ret: HRESULT;
        if _ILIsDesktop((*this).m_pidl) != 0 {
            ret = SHGetDesktopFolder(&mut psf_parent as *mut *mut IShellFolder);
        } else {
            ret = CShellItem_get_parent_shellfolder_body(
                this,
                &mut psf_parent as *mut *mut IShellFolder,
            );
        }
        if ((ret) < 0) {
            return ret;
        }
        let pidl: LPCITEMIDLIST = ILFindLastID((*this).m_pidl);
        let hr = ((*(*psf_parent).lpVtbl).GetUIObjectOf)(
            psf_parent,
            core::ptr::null_mut(),
            1,
            &pidl as *const LPCITEMIDLIST,
            riid,
            core::ptr::null_mut::<core::ffi::c_uint>(),
            ppvOut,
        );
        ((*(*psf_parent).lpVtbl).Release)(psf_parent);
        psf_parent = core::ptr::null_mut();
        return hr;
    } else if cpp_com_iid_equal(rbhid, core::ptr::addr_of!(BHID_DataObject)) {
        return CShellItem_BindToHandler_body(
            this,
            pbc,
            &BHID_SFUIObject as *const GUID,
            &IID_IDataObject as *const GUID,
            ppvOut,
        );
    } else if cpp_com_iid_equal(rbhid, core::ptr::addr_of!(BHID_SFViewObject)) {
        let mut psf: *mut IShellFolder = core::ptr::null_mut();
        let _psf_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(psf));
        let ret: HRESULT;
        ret = CShellItem_get_shellfolder_body(
            this,
            core::ptr::null_mut::<IBindCtx>(),
            core::ptr::addr_of!(IID_IShellFolder),
            core::ptr::addr_of_mut!(psf).cast(),
        );
        if ((ret) < 0) {
            return ret;
        }
        let hr = ((*(*psf).lpVtbl).CreateViewObject)(psf, core::ptr::null_mut(), riid, ppvOut);
        ((*(*psf).lpVtbl).Release)(psf);
        psf = core::ptr::null_mut();
        return hr;
    }

    return MK_E_NOOBJECT;
}

pub unsafe fn CShellItem_GetParent_body(
    this: *mut CShellItem,
    ppsi: *mut *mut IShellItem,
) -> HRESULT {
    let mut parent_pidl: LPITEMIDLIST = core::ptr::null_mut();
    let mut hr = CShellItem_get_parent_pidl_body(this, &mut parent_pidl);
    if hr >= 0 {
        hr = SHCreateShellItem(core::ptr::null(), core::ptr::null_mut(), parent_pidl, ppsi);
        ILFree(parent_pidl);
    }
    hr
}

pub unsafe fn CShellItem_GetDisplayName_body(
    this: *mut CShellItem,
    sigdnName: SIGDN,
    ppszName: *mut LPWSTR,
) -> HRESULT {
    SHGetNameFromIDList((*this).m_pidl, sigdnName, ppszName)
}

pub unsafe fn CShellItem_GetAttributes_body(
    this: *mut CShellItem,
    sfgaoMask: SFGAOF,
    psfgaoAttribs: *mut SFGAOF,
) -> HRESULT {
    let mut parent_folder: *mut IShellFolder = core::ptr::null_mut();
    let _parent_folder_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(parent_folder));
    let child_pidl: LPCITEMIDLIST;
    let mut hr: HRESULT;

    if _ILIsDesktop((*this).m_pidl) != 0 {
        hr = SHGetDesktopFolder(&mut parent_folder as *mut *mut IShellFolder);
    } else {
        hr = CShellItem_get_parent_shellfolder_body(
            this,
            &mut parent_folder as *mut *mut IShellFolder,
        );
    }
    if ((hr) < 0) {
        if !parent_folder.is_null() {
            ((*(*parent_folder).lpVtbl).Release)(parent_folder);
            parent_folder = core::ptr::null_mut();
        }
        return hr;
    }

    child_pidl = ILFindLastID((*this).m_pidl);
    *psfgaoAttribs = sfgaoMask;
    hr = ((*(*parent_folder).lpVtbl).GetAttributesOf)(parent_folder, 1, &child_pidl, psfgaoAttribs);
    *psfgaoAttribs &= sfgaoMask;

    if ((hr) < 0) {
        if !parent_folder.is_null() {
            ((*(*parent_folder).lpVtbl).Release)(parent_folder);
            parent_folder = core::ptr::null_mut();
        }
        return hr;
    }

    let result = if sfgaoMask == *psfgaoAttribs {
        S_OK
    } else {
        S_FALSE
    };
    if !parent_folder.is_null() {
        ((*(*parent_folder).lpVtbl).Release)(parent_folder);
        parent_folder = core::ptr::null_mut();
    }
    result
}

pub unsafe fn CShellItem_Compare_body(
    this: *mut CShellItem,
    oth: *mut IShellItem,
    hint: SICHINTF,
    piOrder: *mut i32,
) -> HRESULT {
    if piOrder.is_null() || oth.is_null() {
        return E_POINTER;
    }
    let mut hr: HRESULT;
    let mut pIDList: *mut IPersistIDList = core::ptr::null_mut();
    let _pIDList_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pIDList));
    hr = ((*(*oth).lpVtbl).QueryInterface)(
        oth,
        core::ptr::addr_of!(IID_IPersistIDList),
        core::ptr::addr_of_mut!(pIDList).cast(),
    );
    if hr >= 0 {
        let mut pidl: LPITEMIDLIST = core::ptr::null_mut();
        hr = ((*(*pIDList).lpVtbl).GetIDList)(pIDList, &mut pidl as *mut LPITEMIDLIST);
        if hr >= 0 {
            let mut psfDesktop: *mut IShellFolder = core::ptr::null_mut();
            let _psfDesktop_cpp_com_guard =
                CppComPtrGuard::new(core::ptr::addr_of_mut!(psfDesktop));
            hr = SHGetDesktopFolder(&mut psfDesktop as *mut *mut IShellFolder);
            if hr >= 0 {
                hr = ((*(*psfDesktop).lpVtbl).CompareIDs)(
                    psfDesktop,
                    hint as LPARAM,
                    (*this).m_pidl,
                    pidl,
                );
                *piOrder = (((hr) as u32) & 0xffff) as i16 as i32;
            }
            if !psfDesktop.is_null() {
                ((*(*psfDesktop).lpVtbl).Release)(psfDesktop);
                psfDesktop = core::ptr::null_mut();
            }
        }
        ILFree(pidl);
    }
    if hr < 0 {
        if !pIDList.is_null() {
            ((*(*pIDList).lpVtbl).Release)(pIDList);
            pIDList = core::ptr::null_mut();
        }
        return hr;
    }
    let order = *piOrder;
    if !pIDList.is_null() {
        ((*(*pIDList).lpVtbl).Release)(pIDList);
        pIDList = core::ptr::null_mut();
    }
    if order != 0 {
        return S_FALSE;
    } else {
        return S_OK;
    }
}

pub unsafe fn CShellItem_GetClassID_body(this: *mut CShellItem, pClassID: *mut CLSID) -> HRESULT {
    *pClassID = CLSID_ShellItem;
    S_OK
}

pub unsafe fn CShellItem_SetIDList_body(
    this: *mut CShellItem,
    pidlx: PCIDLIST_ABSOLUTE,
) -> HRESULT {
    let new_pidl = ILClone(pidlx);
    if !new_pidl.is_null() {
        ILFree((*this).m_pidl);
        (*this).m_pidl = new_pidl;
        return S_OK;
    } else {
        return E_OUTOFMEMORY;
    }
}

pub unsafe fn CShellItem_GetIDList_body(
    this: *mut CShellItem,
    ppidl: *mut PIDLIST_ABSOLUTE,
) -> HRESULT {
    let cloned_pidl = ILClone((*this).m_pidl);
    *ppidl = cloned_pidl as PIDLIST_ABSOLUTE;
    if !cloned_pidl.is_null() {
        S_OK
    } else {
        E_OUTOFMEMORY
    }
}

// Upstream-sync 2026-09: SHCreateItemFromIDList and SHGetItemFromObject are lost
// exports out of CShellItem.cpp (fully commented out; this Rust twin already covers
// CShellItem/CShellItemArray/SHCreateShellItem/SHCreateShellItemArrayFromDataObject but
// not these two). SHGetItemFromObject was the only one the scanner flagged;
// SHCreateItemFromIDList slipped past it (a separate scanner bug -- its spec line has a
// trailing `# Vista:330` comment that the entry regex doesn't tolerate, so it's never
// counted as an export at all -- flagged for the coordinator, not fixed here since it's
// outside the assigned 16). Implementing SHGetItemFromObject calls SHCreateItemFromIDList
// (comdlg32's itemdlg.c is a real caller of SHGetItemFromObject), so both are ported
// together here rather than leaving a dangling undefined-symbol dependency. Both are
// 1:1 with the C, built on SHCreateShellItem which already exists in this file.
#[no_mangle]
pub unsafe extern "system" fn SHCreateItemFromIDList(
    pidl: PCIDLIST_ABSOLUTE,
    riid: REFIID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    if ppv.is_null() {
        return E_POINTER;
    }
    *ppv = core::ptr::null_mut();

    if pidl.is_null() {
        return E_INVALIDARG;
    }

    let mut item: *mut IShellItem = core::ptr::null_mut();
    let hr = SHCreateShellItem(core::ptr::null(), core::ptr::null_mut(), pidl, &mut item);
    if hr < 0 {
        return hr;
    }

    let hr2 = ((*(*item).lpVtbl).QueryInterface)(item, riid, ppv);
    ((*(*item).lpVtbl).Release)(item);
    hr2
}

extern "system" {
    // wine/pidl.rs
    fn SHGetIDListFromObject(punk: *mut IUnknown, ppidl: *mut LPITEMIDLIST) -> HRESULT;
}

#[no_mangle]
pub unsafe extern "system" fn SHGetItemFromObject(
    punk: *mut IUnknown,
    riid: REFIID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    let mut pidl: LPITEMIDLIST = core::ptr::null_mut();
    let hr = SHGetIDListFromObject(punk, &mut pidl);
    let hr = if hr >= 0 {
        SHCreateItemFromIDList(pidl, riid, ppv)
    } else {
        hr
    };
    if !pidl.is_null() {
        ILFree(pidl);
    }
    hr
}

// Upstream-sync 2026-09 (round 2, after the scanner's tail-comment fix surfaced these):
// SHCreateItemFromParsingName / SHCreateItemFromRelativeName / SHCreateItemInKnownFolder /
// SHGetPropertyStoreFromParsingName. All four are 1:1 with CShellItem.cpp, built on the
// SHCreateItemFromIDList twin above.
struct PidlGuard {
    slot: *mut LPITEMIDLIST,
}
impl PidlGuard {
    const fn new(slot: *mut LPITEMIDLIST) -> Self {
        Self { slot }
    }
}
impl Drop for PidlGuard {
    fn drop(&mut self) {
        unsafe {
            let value = *self.slot;
            if !value.is_null() {
                ILFree(value);
                *self.slot = core::ptr::null_mut();
            }
        }
    }
}

extern "system" {
    // shlwapi/shell32 SDK import; already relied on by client/path.c-derived Rust and
    // the C original here (dll/win32/shell32/CShellItem.cpp:379).
    fn SHParseDisplayName(
        pszName: PCWSTR,
        pbc: *mut c_void,
        ppidl: *mut LPITEMIDLIST,
        sfgaoIn: DWORD,
        psfgaoOut: *mut DWORD,
    ) -> HRESULT;
}

#[no_mangle]
pub unsafe extern "system" fn SHCreateItemFromParsingName(
    pszPath: PCWSTR,
    pbc: *mut IBindCtx,
    riid: REFIID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    if ppv.is_null() {
        return E_POINTER;
    }
    *ppv = core::ptr::null_mut();

    let mut pidl: LPITEMIDLIST = core::ptr::null_mut();
    let _pidl_guard = PidlGuard::new(core::ptr::addr_of_mut!(pidl));

    let hr = SHParseDisplayName(
        pszPath,
        pbc.cast(),
        &mut pidl,
        0,
        core::ptr::null_mut(),
    );
    if hr >= 0 {
        SHCreateItemFromIDList(pidl, riid, ppv)
    } else {
        hr
    }
}

#[no_mangle]
pub unsafe extern "system" fn SHCreateItemFromRelativeName(
    parent: *mut IShellItem,
    pszName: PCWSTR,
    pbc: *mut IBindCtx,
    riid: REFIID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    if ppv.is_null() {
        return E_POINTER;
    }
    *ppv = core::ptr::null_mut();

    if pszName.is_null() {
        return E_INVALIDARG;
    }

    let mut pidl_folder: LPITEMIDLIST = core::ptr::null_mut();
    let _pidl_folder_guard = PidlGuard::new(core::ptr::addr_of_mut!(pidl_folder));
    let hr = SHGetIDListFromObject(parent as *mut IUnknown, &mut pidl_folder);
    if hr != S_OK {
        return hr;
    }

    let mut desktop: *mut IShellFolder = core::ptr::null_mut();
    let _desktop_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(desktop));
    let hr = SHGetDesktopFolder(&mut desktop);
    if hr < 0 {
        return hr;
    }

    let mut folder: *mut IShellFolder = core::ptr::null_mut();
    let _folder_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(folder));
    if _ILIsDesktop(pidl_folder) == 0 {
        let hr = ((*(*desktop).lpVtbl).BindToObject)(
            desktop,
            pidl_folder,
            core::ptr::null_mut(),
            core::ptr::addr_of!(IID_IShellFolder),
            core::ptr::addr_of_mut!(folder).cast(),
        );
        if hr < 0 {
            return hr;
        }
    }

    let psf_bind = if !folder.is_null() { folder } else { desktop };
    let mut pidl_child: LPITEMIDLIST = core::ptr::null_mut();
    let _pidl_child_guard = PidlGuard::new(core::ptr::addr_of_mut!(pidl_child));
    let hr = ((*(*psf_bind).lpVtbl).ParseDisplayName)(
        psf_bind,
        core::ptr::null_mut(),
        pbc,
        pszName as LPWSTR,
        core::ptr::null_mut(),
        &mut pidl_child,
        core::ptr::null_mut(),
    );
    if hr < 0 {
        return hr;
    }

    SHCreateItemFromIDList(pidl_child, riid, ppv)
}

// Upstream ships a plain `return E_NOTIMPL;` here -- the real body sits behind `#if 0`
// with `//TODO: Import SHGetKnownFolderIDList from wine`, i.e. upstream itself hasn't
// wired this up (SHGetKnownFolderIDList doesn't exist in this tree, C or Rust). Porting
// what upstream ACTUALLY ships, not the dead #if-0'd body, per the coordinator's note.
#[no_mangle]
pub unsafe extern "system" fn SHCreateItemInKnownFolder(
    _kfid: REFGUID,
    _dwFlags: DWORD,
    _pszFileName: PCWSTR,
    _riid: REFIID,
    _ppv: *mut *mut c_void,
) -> HRESULT {
    E_NOTIMPL
}

// Minimal `IShellItem2 : IShellItem` vtable -- just enough to reach GetPropertyStore,
// the only method this TU calls. IID_IShellItem2 = {7E9FB0D3-919F-4307-AB2E-9B1860310C93}
// (shobjidl.idl), a real linkable symbol from sdk/lib/uuid (widl-generated
// shobjidl_i.c), same pattern as every other IID_* extern in this file.
#[repr(C)]
struct IShellItem2 {
    lpVtbl: *const IShellItem2Vtbl,
}
#[repr(C)]
struct IShellItem2Vtbl {
    QueryInterface:
        unsafe extern "system" fn(*mut IShellItem2, REFIID, *mut *mut c_void) -> HRESULT,
    AddRef: unsafe extern "system" fn(*mut IShellItem2) -> ULONG,
    Release: unsafe extern "system" fn(*mut IShellItem2) -> ULONG,
    BindToHandler: unsafe extern "system" fn(
        *mut IShellItem2,
        *mut IBindCtx,
        REFGUID,
        REFIID,
        *mut *mut c_void,
    ) -> HRESULT,
    GetParent: unsafe extern "system" fn(*mut IShellItem2, *mut *mut IShellItem) -> HRESULT,
    GetDisplayName:
        unsafe extern "system" fn(*mut IShellItem2, SIGDN, *mut LPWSTR) -> HRESULT,
    GetAttributes:
        unsafe extern "system" fn(*mut IShellItem2, SFGAOF, *mut SFGAOF) -> HRESULT,
    Compare: unsafe extern "system" fn(*mut IShellItem2, *mut IShellItem, SICHINTF, *mut i32) -> HRESULT,
    GetPropertyStore: unsafe extern "system" fn(
        *mut IShellItem2,
        crate::shellcom::GETPROPERTYSTOREFLAGS,
        REFIID,
        *mut *mut c_void,
    ) -> HRESULT,
}

extern "C" {
    static IID_IShellItem2: GUID;
}

#[no_mangle]
pub unsafe extern "system" fn SHGetPropertyStoreFromParsingName(
    pszPath: PCWSTR,
    pbc: *mut IBindCtx,
    flags: crate::shellcom::GETPROPERTYSTOREFLAGS,
    riid: REFIID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    if ppv.is_null() {
        return E_POINTER;
    }
    *ppv = core::ptr::null_mut();

    let mut item: *mut IShellItem2 = core::ptr::null_mut();
    let _item_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(item));
    let hr = SHCreateItemFromParsingName(
        pszPath,
        pbc,
        core::ptr::addr_of!(IID_IShellItem2),
        core::ptr::addr_of_mut!(item).cast(),
    );
    if hr >= 0 {
        ((*(*item).lpVtbl).GetPropertyStore)(item, flags, riid, ppv)
    } else {
        hr
    }
}
