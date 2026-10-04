use crate::prelude::*;

extern "C" {

    fn wcscmp(s1: *const u16, s2: *const u16) -> i32;
}

extern "system" {
    // ole32's own export (implemented in moniker.c); not part of the shared
    // win32 surface yet.
    fn GetRunningObjectTable(
        reserved: DWORD,
        pprot: *mut *mut IRunningObjectTable,
    ) -> HRESULT;
}
// ---- file-scope preamble (translated) ----
pub const BINDCTX_FIRST_TABLE_SIZE: u32 = 4;

#[repr(C)]
pub struct BindCtxObject {
    pub pObj: *mut IUnknown,
    pub pkeyObj: LPOLESTR,
    pub regType: u8,
}

pub type PBindCtxObject = *mut BindCtxObject;

/// objidl.idl `BIND_OPTS3`: `BIND_OPTS2` plus the `hwnd` wine-10 stores. com-sys's
/// island predates it, so the layout is mirrored here field for field.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct BIND_OPTS3 {
    pub cbStruct: DWORD,
    pub grfFlags: DWORD,
    pub grfMode: DWORD,
    pub dwTickCountDeadline: DWORD,
    pub dwTrackFlags: DWORD,
    pub dwClassContext: DWORD,
    pub locale: LCID,
    pub pServerInfo: *mut COSERVERINFO,
    pub hwnd: HWND,
}

#[repr(C)]
pub struct BindCtxImpl {
    pub IBindCtx_iface: IBindCtx,
    pub ref_: i32,
    pub bindCtxTable: *mut BindCtxObject,
    pub bindCtxTableLastIndex: u32,
    pub bindCtxTableSize: u32,
    pub options: BIND_OPTS3,
}

pub type PBindCtxImpl = *mut BindCtxImpl;

/// The vtbl slots in com-sys are typed with the target's STDMETHODCALLTYPE ABI
/// (`stdcall` on i386, `C` elsewhere). Our method impls are declared
/// `extern "system"`, which lowers to exactly that ABI — but since rustc 1.92 the
/// `"system"` fn *type* no longer unifies with `"C"`, so the (codegen-identical)
/// fn pointer needs a transmute to land in the slot.
macro_rules! vtbl_slot {
    ($f:ident, ( $($ty:ty),* $(,)? ) -> $ret:ty) => {
        Some(unsafe {
            core::mem::transmute::<unsafe extern "system" fn($($ty),*) -> $ret, _>($f)
        })
    };
}

pub static VT_BindCtxImpl: IBindCtxVtbl = IBindCtxVtbl {
    QueryInterface: vtbl_slot!(BindCtxImpl_QueryInterface, (*mut IBindCtx, *const IID, *mut LPVOID) -> HRESULT),
    AddRef: vtbl_slot!(BindCtxImpl_AddRef, (*mut IBindCtx) -> ULONG),
    Release: vtbl_slot!(BindCtxImpl_Release, (*mut IBindCtx) -> ULONG),
    RegisterObjectBound: vtbl_slot!(BindCtxImpl_RegisterObjectBound, (*mut IBindCtx, *mut IUnknown) -> HRESULT),
    RevokeObjectBound: vtbl_slot!(BindCtxImpl_RevokeObjectBound, (*mut IBindCtx, *mut IUnknown) -> HRESULT),
    ReleaseBoundObjects: vtbl_slot!(BindCtxImpl_ReleaseBoundObjects, (*mut IBindCtx) -> HRESULT),
    SetBindOptions: vtbl_slot!(BindCtxImpl_SetBindOptions, (*mut IBindCtx, *mut BIND_OPTS) -> HRESULT),
    GetBindOptions: vtbl_slot!(BindCtxImpl_GetBindOptions, (*mut IBindCtx, *mut BIND_OPTS) -> HRESULT),
    GetRunningObjectTable: vtbl_slot!(BindCtxImpl_GetRunningObjectTable, (*mut IBindCtx, *mut *mut IRunningObjectTable) -> HRESULT),
    RegisterObjectParam: vtbl_slot!(BindCtxImpl_RegisterObjectParam, (*mut IBindCtx, LPOLESTR, *mut IUnknown) -> HRESULT),
    GetObjectParam: vtbl_slot!(BindCtxImpl_GetObjectParam, (*mut IBindCtx, LPOLESTR, *mut *mut IUnknown) -> HRESULT),
    EnumObjectParam: vtbl_slot!(BindCtxImpl_EnumObjectParam, (*mut IBindCtx, *mut *mut IEnumString) -> HRESULT),
    RevokeObjectParam: vtbl_slot!(BindCtxImpl_RevokeObjectParam, (*mut IBindCtx, LPOLESTR) -> HRESULT),
};

unsafe fn impl_from_IBindCtx(iface: *mut IBindCtx) -> *mut BindCtxImpl {
    (iface as *mut u8).offset(-(core::mem::offset_of!(BindCtxImpl, IBindCtx_iface) as isize)) as *mut BindCtxImpl
}
unsafe extern "system" fn BindCtxImpl_QueryInterface(
    iface: *mut IBindCtx,
    riid: *const IID,
    ppvObject: *mut LPVOID,
) -> HRESULT {
    let This = impl_from_IBindCtx(iface);

    TRACE!(
        b"(%p %s %p)\n\0".as_ptr(),
        This,
        debugstr_guid(riid),
        ppvObject,
    );

    if ppvObject.is_null() {
        return E_POINTER;
    }

    *ppvObject = core::ptr::null_mut();

    if IsEqualIID(core::ptr::addr_of!(IID_IUnknown), riid)
        || IsEqualIID(core::ptr::addr_of!(IID_IBindCtx), riid)
    {
        *ppvObject = core::ptr::addr_of_mut!((*This).IBindCtx_iface) as LPVOID;
        IBindCtx_AddRef(iface);
        return S_OK;
    }

    E_NOINTERFACE
}

unsafe extern "system" fn BindCtxImpl_AddRef(iface: *mut IBindCtx) -> u32 {
    let This = impl_from_IBindCtx(iface);

    InterlockedIncrement(&mut (*This).ref_) as u32
}

unsafe fn BindCtxImpl_Destroy(This: *mut BindCtxImpl) -> HRESULT {
    TRACE!(b"(%p)\n\0".as_ptr() as *const i8, This);

    HeapFree(GetProcessHeap(), 0, (*This).bindCtxTable as LPVOID);

    HeapFree(GetProcessHeap(), 0, This as LPVOID);

    return S_OK;
}
unsafe extern "system" fn BindCtxImpl_Release(iface: *mut IBindCtx) -> ULONG {
    let This = impl_from_IBindCtx(iface);
    let ref_count = InterlockedDecrement(&mut (*This).ref_) as ULONG;
    if ref_count == 0 {
        BindCtxImpl_ReleaseBoundObjects(&mut (*This).IBindCtx_iface);
        BindCtxImpl_Destroy(This);
    }
    ref_count
}

unsafe extern "system" fn BindCtxImpl_RegisterObjectBound(iface: *mut IBindCtx, punk: *mut IUnknown) -> HRESULT {
    let This = impl_from_IBindCtx(iface);
    let lastIndex = (*This).bindCtxTableLastIndex;

    TRACE!(b"(%p,%p)\n\0".as_ptr() as *const i8, This, punk);

    if punk.is_null() {
        return S_OK;
    }

    if lastIndex == (*This).bindCtxTableSize {
        let hr = BindCtxImpl_ExpandTable(This);
        if FAILED(hr) {
            return hr;
        }
    }

    IUnknown_AddRef(punk);

    (*(*This).bindCtxTable.offset(lastIndex as isize)).pObj = punk;
    (*(*This).bindCtxTable.offset(lastIndex as isize)).pkeyObj = core::ptr::null_mut();
    (*(*This).bindCtxTable.offset(lastIndex as isize)).regType = 0;
    (*This).bindCtxTableLastIndex += 1;

    S_OK
}
unsafe extern "system" fn BindCtxImpl_RevokeObjectBound(
    iface: *mut IBindCtx,
    punk: *mut IUnknown,
) -> HRESULT {
    let mut index: DWORD = 0;
    let mut j: DWORD;

    let This = impl_from_IBindCtx(iface);

    TRACE!(b"(%p,%p)\n\0".as_ptr() as *const i8, This, punk);

    if punk.is_null() {
        return E_INVALIDARG;
    }

    if BindCtxImpl_GetObjectIndex(This, punk, core::ptr::null_mut(), &mut index) == S_FALSE {
        return MK_E_NOTBOUND;
    }

    if !(*This).bindCtxTable.offset(index as isize).read().pObj.is_null() {
        IUnknown_Release((*This).bindCtxTable.offset(index as isize).read().pObj);
    }
    HeapFree(
        GetProcessHeap(),
        0,
        (*This).bindCtxTable.offset(index as isize).read().pkeyObj as LPVOID,
    );

    j = index;
    while j < (*This).bindCtxTableLastIndex - 1 {
        (*This).bindCtxTable.offset(j as isize).write(
            (*This).bindCtxTable.offset((j + 1) as isize).read()
        );
        j += 1;
    }

    (*This).bindCtxTableLastIndex -= 1;

    S_OK
}
unsafe extern "system" fn BindCtxImpl_ReleaseBoundObjects(iface: *mut IBindCtx) -> HRESULT {
    let This = impl_from_IBindCtx(iface);

    TRACE!(b"(%p)\n\0".as_ptr() as *const i8, This);

    let mut i: u32 = 0;
    while i < (*This).bindCtxTableLastIndex {
        if !(*(*This).bindCtxTable.offset(i as isize)).pObj.is_null() {
            IUnknown_Release((*(*This).bindCtxTable.offset(i as isize)).pObj);
        }
        HeapFree(
            GetProcessHeap(),
            0,
            (*(*This).bindCtxTable.offset(i as isize)).pkeyObj as LPVOID,
        );
        i += 1;
    }

    (*This).bindCtxTableLastIndex = 0;

    S_OK
}

unsafe extern "system" fn BindCtxImpl_SetBindOptions(iface: *mut IBindCtx, pbindopts: *mut BIND_OPTS) -> HRESULT {
    let This = impl_from_IBindCtx(iface);

    TRACE!(b"(%p,%p)\n\0".as_ptr() as *const i8, This, pbindopts);

    if pbindopts.is_null() {
        return E_POINTER;
    }

    if (*pbindopts).cbStruct > core::mem::size_of::<BIND_OPTS3>() as u32 {
        WARN!(b"invalid size\n\0".as_ptr() as *const i8);
        return E_INVALIDARG; /* FIXME : not verified */
    }
    core::ptr::copy_nonoverlapping(
        pbindopts as *const u8,
        core::ptr::addr_of_mut!((*This).options) as *mut u8,
        (*pbindopts).cbStruct as usize
    );
    S_OK
}

unsafe extern "system" fn BindCtxImpl_GetBindOptions(iface: *mut IBindCtx, pbindopts: *mut BIND_OPTS) -> HRESULT {
    let This = impl_from_IBindCtx(iface);
    let mut cbStruct: ULONG;

    TRACE!(b"(%p,%p)\n\0".as_ptr() as *const i8, This, pbindopts);

    if pbindopts.is_null() {
        return E_POINTER;
    }

    cbStruct = (*pbindopts).cbStruct;
    if cbStruct > core::mem::size_of::<BIND_OPTS3>() as ULONG {
        cbStruct = core::mem::size_of::<BIND_OPTS3>() as ULONG;
    }

    core::ptr::copy_nonoverlapping(
        core::ptr::addr_of!((*This).options) as *const u8,
        pbindopts as *mut u8,
        cbStruct as usize
    );
    (*pbindopts).cbStruct = cbStruct;

    S_OK
}

unsafe extern "system" fn BindCtxImpl_GetRunningObjectTable(
    iface: *mut IBindCtx,
    pprot: *mut *mut IRunningObjectTable,
) -> HRESULT {
    let _This = impl_from_IBindCtx(iface);

    TRACE!(b"(%p,%p)\n\0".as_ptr() as *const i8, This, pprot);

    if pprot.is_null() {
        return E_POINTER;
    }

    return GetRunningObjectTable(0, pprot);
}

unsafe extern "system" fn BindCtxImpl_RegisterObjectParam(
    iface: *mut IBindCtx,
    pszkey: LPOLESTR,
    punk: *mut IUnknown,
) -> HRESULT {

    let mut index: u32 = 0;
    let This = impl_from_IBindCtx(iface);

    TRACE!(
        b"(%p,%s,%p)\n\0".as_ptr(),
        This,
        debugstr_w(pszkey),
        punk,
    );

    if punk.is_null() {
        return E_INVALIDARG;
    }

    if !pszkey.is_null() && BindCtxImpl_GetObjectIndex(This, core::ptr::null_mut(), pszkey, &mut index) == S_OK {
        TRACE!(b"Overwriting existing key\n\0".as_ptr());
        if !(*This).bindCtxTable.offset(index as isize).read().pObj.is_null() {
            IUnknown_Release((*This).bindCtxTable.offset(index as isize).read().pObj);
        }
        (*(*This).bindCtxTable.offset(index as isize)).pObj = punk;
        IUnknown_AddRef(punk);
        return S_OK;
    }

    if (*This).bindCtxTableLastIndex == (*This).bindCtxTableSize {
        let hr = BindCtxImpl_ExpandTable(This);
        if FAILED(hr) {
            return hr;
        }
    }

    (*(*This).bindCtxTable.offset((*This).bindCtxTableLastIndex as isize)).pObj = punk;
    (*(*This).bindCtxTable.offset((*This).bindCtxTableLastIndex as isize)).regType = 1;

    if pszkey.is_null() {
        (*(*This).bindCtxTable.offset((*This).bindCtxTableLastIndex as isize)).pkeyObj = core::ptr::null_mut();
    } else {
        (*(*This).bindCtxTable.offset((*This).bindCtxTableLastIndex as isize)).pkeyObj =
            HeapAlloc(
                GetProcessHeap(),
                0,
                (core::mem::size_of::<u16>() * (1 + lstrlenW(pszkey) as usize)) as _,
            ) as LPOLESTR;

        if (*This).bindCtxTable.offset((*This).bindCtxTableLastIndex as isize).read().pkeyObj.is_null() {
            return E_OUTOFMEMORY;
        }
        lstrcpyW(
            (*This).bindCtxTable.offset((*This).bindCtxTableLastIndex as isize).read().pkeyObj,
            pszkey,
        );
    }

    (*This).bindCtxTableLastIndex += 1;

    IUnknown_AddRef(punk);
    S_OK
}
unsafe extern "system" fn BindCtxImpl_GetObjectParam(
    iface: *mut IBindCtx,
    pszkey: LPOLESTR,
    punk: *mut *mut IUnknown,
) -> HRESULT {
    let mut index: DWORD = 0;
    let This = impl_from_IBindCtx(iface);

    TRACE!(
        b"(%p,%s,%p)\n\0".as_ptr() as *const i8,
        This,
        debugstr_w(pszkey),
        punk,
    );

    if punk == core::ptr::null_mut() {
        return E_POINTER;
    }

    *punk = core::ptr::null_mut();

    if BindCtxImpl_GetObjectIndex(This, core::ptr::null_mut(), pszkey, &mut index) == S_FALSE {
        return E_FAIL;
    }

    IUnknown_AddRef((*This).bindCtxTable.offset(index as isize).read().pObj);

    *punk = (*This).bindCtxTable.offset(index as isize).read().pObj;

    return S_OK;
}
unsafe extern "system" fn BindCtxImpl_RevokeObjectParam(
    iface: *mut IBindCtx,
    ppenum: LPOLESTR,
) -> HRESULT {
    let mut index: DWORD = 0;

    let This = impl_from_IBindCtx(iface);

    TRACE!(b"(%p,%s)\n\0".as_ptr() as *const i8, This, debugstr_w(ppenum));

    if BindCtxImpl_GetObjectIndex(This, core::ptr::null_mut(), ppenum, &mut index) == S_FALSE {
        return E_FAIL;
    }

    if !(*(*This).bindCtxTable.add(index as usize)).pObj.is_null() {
        IUnknown_Release((*(*This).bindCtxTable.add(index as usize)).pObj);
    }
    HeapFree(
        GetProcessHeap(),
        0,
        (*(*This).bindCtxTable.add(index as usize)).pkeyObj as LPVOID,
    );

    let mut j = index;
    while j < (*This).bindCtxTableLastIndex - 1 {
        (*This).bindCtxTable.add(j as usize).write(
            (*This).bindCtxTable.add((j + 1) as usize).read()
        );
        j += 1;
    }

    (*This).bindCtxTableLastIndex -= 1;

    S_OK
}

unsafe extern "system" fn BindCtxImpl_EnumObjectParam(
    _iface: *mut IBindCtx,
    pszkey: *mut *mut IEnumString,
) -> HRESULT {
    TRACE!(
        b"(%p,%p)\n\0".as_ptr() as *const i8,
        iface,
        pszkey,
    );

    *pszkey = core::ptr::null_mut();

    E_NOTIMPL
}

unsafe fn BindCtxImpl_GetObjectIndex(
    This: *mut BindCtxImpl,
    punk: *mut IUnknown,
    pszkey: LPOLESTR,
    index: *mut DWORD,
) -> HRESULT {
    let mut i: DWORD;
    let mut found: BOOL = 0;

    if punk.is_null() {
        i = 0;
        while i < (*This).bindCtxTableLastIndex && found == 0 {
            if (*(*This).bindCtxTable.offset(i as isize)).regType == 1 {
                if ((*(*This).bindCtxTable.offset(i as isize)).pkeyObj.is_null() && pszkey.is_null())
                    || ((!(*(*This).bindCtxTable.offset(i as isize)).pkeyObj.is_null())
                        && (!pszkey.is_null())
                        && (wcscmp((*(*This).bindCtxTable.offset(i as isize)).pkeyObj, pszkey) == 0))
                {
                    found = 1;
                }
            }
            i += 1;
        }
    } else {
        i = 0;
        while i < (*This).bindCtxTableLastIndex && found == 0 {
            if (*(*This).bindCtxTable.offset(i as isize)).pObj == punk {
                found = 1;
            }
            i += 1;
        }
    }

    if !index.is_null() {
        *index = i.wrapping_sub(1);
    }

    if found != 0 {
        return S_OK;
    }
    TRACE!(b"key not found\n\0".as_ptr() as *const i8);
    return S_FALSE;
}

unsafe fn BindCtxImpl_ExpandTable(This: *mut BindCtxImpl) -> HRESULT {
    if (*This).bindCtxTableSize == 0 {
        (*This).bindCtxTableSize = BINDCTX_FIRST_TABLE_SIZE;
        (*This).bindCtxTable = HeapAlloc(
            GetProcessHeap(),
            HEAP_ZERO_MEMORY,
            (((*This).bindCtxTableSize as usize) * core::mem::size_of::<BindCtxObject>()) as _,
        ) as *mut BindCtxObject;
    } else {
        (*This).bindCtxTableSize *= 2;

        (*This).bindCtxTable = HeapReAlloc(
            GetProcessHeap(),
            HEAP_ZERO_MEMORY,
            (*This).bindCtxTable as LPVOID,
            (((*This).bindCtxTableSize as usize) * core::mem::size_of::<BindCtxObject>()) as _,
        ) as *mut BindCtxObject;
    }

    if (*This).bindCtxTable.is_null() {
        return E_OUTOFMEMORY;
    }

    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn CreateBindCtx(reserved: u32, ppbc: *mut *mut IBindCtx) -> HRESULT {

    TRACE!(b"(%d,%p)\n\0".as_ptr(), reserved, ppbc);

    if ppbc.is_null() {
        return E_INVALIDARG;
    }

    *ppbc = core::ptr::null_mut();

    if reserved != 0 {
        ERR!(b"reserved should be 0, not 0x%x\n\0".as_ptr(), reserved);
        return E_INVALIDARG;
    }

    let object = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, core::mem::size_of::<BindCtxImpl>() as _) as *mut BindCtxImpl;
    if object.is_null() {
        return E_OUTOFMEMORY;
    }

    (*object).IBindCtx_iface.lpVtbl = &VT_BindCtxImpl as *const IBindCtxVtbl as *mut IBindCtxVtbl;
    (*object).ref_ = 1;
    (*object).options.cbStruct = core::mem::size_of::<BIND_OPTS3>() as u32;
    (*object).options.grfMode = STGM_READWRITE;
    (*object).options.dwClassContext = CLSCTX_SERVER;
    (*object).options.locale = GetThreadLocale();

    *ppbc = core::ptr::addr_of_mut!((*object).IBindCtx_iface);

    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn BindMoniker(
    pmk: LPMONIKER,
    grfOpt: u32,
    riid: REFIID,
    ppvResult: *mut LPVOID,
) -> HRESULT {
    let mut res: HRESULT;
    let mut pbc: *mut IBindCtx = core::ptr::null_mut();

    TRACE!(
        b"(%p, %x, %s, %p)\n\0".as_ptr() as *const i8,
        pmk,
        grfOpt,
        debugstr_guid(riid),
        ppvResult,
    );

    res = CreateBindCtx(grfOpt, &mut pbc);
    if SUCCEEDED(res) {
        res = IMoniker_BindToObject(pmk, pbc, core::ptr::null_mut(), riid, ppvResult);
        IBindCtx_Release(pbc);
    }
    res
}
