use crate::shellcom::*;
use crate::types::*;
use core::ffi::c_void;

pub type LPOLESTR = *mut WCHAR;
pub type HGLOBAL = HANDLE;
pub type CLIPFORMAT = u16;

const E_FAIL: HRESULT = 0x80004005u32 as HRESULT;
const CF_HDROP: CLIPFORMAT = 15;
const TYMED_HGLOBAL: DWORD = 1;
const DVASPECT_CONTENT: DWORD = 1;
const DROPEFFECT_NONE: DWORD = 0;
const DROPEFFECT_COPY: DWORD = 1;
const SW_SHOWNORMAL: INT = 1;
const MAX_PATH: usize = 260;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct POINTL {
    pub x: LONG,
    pub y: LONG,
}

#[repr(C)]
struct FORMATETC {
    cfFormat: CLIPFORMAT,
    ptd: *mut c_void,
    dwAspect: DWORD,
    lindex: LONG,
    tymed: DWORD,
}

#[repr(C)]
union STGMEDIUM_UNION {
    hGlobal: HGLOBAL,
    pointer: *mut c_void,
}

#[repr(C)]
struct STGMEDIUM {
    tymed: DWORD,
    u: STGMEDIUM_UNION,
    pUnkForRelease: *mut IUnknown,
}

#[repr(C)]
struct DROPFILES {
    pFiles: DWORD,
    pt: POINT,
    fNC: BOOL,
    fWide: BOOL,
}
type LPDROPFILES = *mut DROPFILES;

#[repr(C)]
pub struct IPersist {
    pub lpVtbl: *const IPersistVtbl,
}
#[repr(C)]
pub struct IPersistVtbl {
    pub QueryInterface:
        unsafe extern "system" fn(*mut IPersist, REFIID, *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(*mut IPersist) -> ULONG,
    pub Release: unsafe extern "system" fn(*mut IPersist) -> ULONG,
    pub GetClassID: unsafe extern "system" fn(*mut IPersist, *mut CLSID) -> HRESULT,
}

#[repr(C)]
pub struct IPersistFile {
    pub lpVtbl: *const IPersistFileVtbl,
}
#[repr(C)]
pub struct IPersistFileVtbl {
    pub QueryInterface:
        unsafe extern "system" fn(*mut IPersistFile, REFIID, *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(*mut IPersistFile) -> ULONG,
    pub Release: unsafe extern "system" fn(*mut IPersistFile) -> ULONG,
    pub GetClassID: unsafe extern "system" fn(*mut IPersistFile, *mut CLSID) -> HRESULT,
    pub IsDirty: unsafe extern "system" fn(*mut IPersistFile) -> HRESULT,
    pub Load: unsafe extern "system" fn(*mut IPersistFile, LPCOLESTR, DWORD) -> HRESULT,
    pub Save: unsafe extern "system" fn(*mut IPersistFile, LPCOLESTR, BOOL) -> HRESULT,
    pub SaveCompleted: unsafe extern "system" fn(*mut IPersistFile, LPCOLESTR) -> HRESULT,
    pub GetCurFile: unsafe extern "system" fn(*mut IPersistFile, *mut LPOLESTR) -> HRESULT,
}

#[repr(C)]
pub struct IDropTarget {
    pub lpVtbl: *const IDropTargetVtbl,
}
#[repr(C)]
pub struct IDropTargetVtbl {
    pub QueryInterface:
        unsafe extern "system" fn(*mut IDropTarget, REFIID, *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(*mut IDropTarget) -> ULONG,
    pub Release: unsafe extern "system" fn(*mut IDropTarget) -> ULONG,
    pub DragEnter: unsafe extern "system" fn(
        *mut IDropTarget,
        *mut IDataObject,
        DWORD,
        POINTL,
        *mut DWORD,
    ) -> HRESULT,
    pub DragOver: unsafe extern "system" fn(*mut IDropTarget, DWORD, POINTL, *mut DWORD) -> HRESULT,
    pub DragLeave: unsafe extern "system" fn(*mut IDropTarget) -> HRESULT,
    pub Drop: unsafe extern "system" fn(
        *mut IDropTarget,
        *mut IDataObject,
        DWORD,
        POINTL,
        *mut DWORD,
    ) -> HRESULT,
}

extern "system" {
    fn GetProcessHeap() -> HANDLE;
    fn HeapAlloc(heap: HANDLE, flags: DWORD, bytes: usize) -> *mut c_void;
    fn HeapFree(heap: HANDLE, flags: DWORD, memory: *mut c_void) -> BOOL;
    fn Shell32RustObjectLock();
    fn Shell32RustObjectUnlock();
    fn GlobalLock(memory: HGLOBAL) -> *mut c_void;
    fn GlobalUnlock(memory: HGLOBAL) -> BOOL;
    fn ReleaseStgMedium(medium: *mut STGMEDIUM);
    fn StrChrW(start: LPCWSTR, character: WCHAR) -> LPWSTR;
    fn ShellExecuteW(
        hwnd: HWND,
        operation: LPCWSTR,
        file: LPCWSTR,
        parameters: LPCWSTR,
        directory: LPCWSTR,
        show: INT,
    ) -> HINSTANCE;
}

extern "C" {
    static IID_IUnknown: IID;
    static IID_IPersist: IID;
    static IID_IPersistFile: IID;
    static IID_IDropTarget: IID;
    static CLSID_ExeDropHandler: CLSID;
}

#[repr(C)]
pub struct CExeDropHandler {
    pub IPersist_iface: IPersist,
    pub IPersistFile_iface: IPersistFile,
    pub IDropTarget_iface: IDropTarget,
    pub ref_count: core::sync::atomic::AtomicI32,
    pub pclsid: *mut CLSID,
    pub sPathTarget: LPWSTR,
}

unsafe fn from_persist(this: *mut IPersist) -> *mut CExeDropHandler {
    (this as *mut u8).sub(core::mem::offset_of!(CExeDropHandler, IPersist_iface))
        as *mut CExeDropHandler
}
unsafe fn from_persist_file(this: *mut IPersistFile) -> *mut CExeDropHandler {
    (this as *mut u8).sub(core::mem::offset_of!(CExeDropHandler, IPersistFile_iface))
        as *mut CExeDropHandler
}
unsafe fn from_drop_target(this: *mut IDropTarget) -> *mut CExeDropHandler {
    (this as *mut u8).sub(core::mem::offset_of!(CExeDropHandler, IDropTarget_iface))
        as *mut CExeDropHandler
}

unsafe extern "system" fn persist_query(
    this: *mut IPersist,
    riid: REFIID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    query_interface(from_persist(this), riid, ppv)
}
unsafe extern "system" fn persist_addref(this: *mut IPersist) -> ULONG {
    add_ref(from_persist(this))
}
unsafe extern "system" fn persist_release(this: *mut IPersist) -> ULONG {
    release(from_persist(this))
}
unsafe extern "system" fn persist_get_class_id(
    this: *mut IPersist,
    class_id: *mut CLSID,
) -> HRESULT {
    get_class_id(from_persist(this), class_id)
}

unsafe extern "system" fn persist_file_query(
    this: *mut IPersistFile,
    riid: REFIID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    query_interface(from_persist_file(this), riid, ppv)
}
unsafe extern "system" fn persist_file_addref(this: *mut IPersistFile) -> ULONG {
    add_ref(from_persist_file(this))
}
unsafe extern "system" fn persist_file_release(this: *mut IPersistFile) -> ULONG {
    release(from_persist_file(this))
}
unsafe extern "system" fn persist_file_get_class_id(
    this: *mut IPersistFile,
    class_id: *mut CLSID,
) -> HRESULT {
    get_class_id(from_persist_file(this), class_id)
}
unsafe extern "system" fn persist_file_is_dirty(this: *mut IPersistFile) -> HRESULT {
    is_dirty(from_persist_file(this))
}
unsafe extern "system" fn persist_file_load(
    this: *mut IPersistFile,
    file: LPCOLESTR,
    mode: DWORD,
) -> HRESULT {
    load(from_persist_file(this), file, mode)
}
unsafe extern "system" fn persist_file_save(
    this: *mut IPersistFile,
    file: LPCOLESTR,
    remember: BOOL,
) -> HRESULT {
    save(from_persist_file(this), file, remember)
}
unsafe extern "system" fn persist_file_save_completed(
    this: *mut IPersistFile,
    file: LPCOLESTR,
) -> HRESULT {
    save_completed(from_persist_file(this), file)
}
unsafe extern "system" fn persist_file_get_cur_file(
    this: *mut IPersistFile,
    file: *mut LPOLESTR,
) -> HRESULT {
    get_cur_file(from_persist_file(this), file)
}

unsafe extern "system" fn drop_query(
    this: *mut IDropTarget,
    riid: REFIID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    query_interface(from_drop_target(this), riid, ppv)
}
unsafe extern "system" fn drop_addref(this: *mut IDropTarget) -> ULONG {
    add_ref(from_drop_target(this))
}
unsafe extern "system" fn drop_release(this: *mut IDropTarget) -> ULONG {
    release(from_drop_target(this))
}
unsafe extern "system" fn drop_drag_enter(
    this: *mut IDropTarget,
    data: *mut IDataObject,
    keys: DWORD,
    point: POINTL,
    effect: *mut DWORD,
) -> HRESULT {
    drag_enter(from_drop_target(this), data, keys, point, effect)
}
unsafe extern "system" fn drop_drag_over(
    this: *mut IDropTarget,
    keys: DWORD,
    point: POINTL,
    effect: *mut DWORD,
) -> HRESULT {
    drag_over(from_drop_target(this), keys, point, effect)
}
unsafe extern "system" fn drop_drag_leave(this: *mut IDropTarget) -> HRESULT {
    drag_leave(from_drop_target(this))
}
unsafe extern "system" fn drop_drop(
    this: *mut IDropTarget,
    data: *mut IDataObject,
    keys: DWORD,
    point: POINTL,
    effect: *mut DWORD,
) -> HRESULT {
    drop_files(from_drop_target(this), data, keys, point, effect)
}

pub static IPERSIST_VTBL: IPersistVtbl = IPersistVtbl {
    QueryInterface: persist_query,
    AddRef: persist_addref,
    Release: persist_release,
    GetClassID: persist_get_class_id,
};
pub static IPERSISTFILE_VTBL: IPersistFileVtbl = IPersistFileVtbl {
    QueryInterface: persist_file_query,
    AddRef: persist_file_addref,
    Release: persist_file_release,
    GetClassID: persist_file_get_class_id,
    IsDirty: persist_file_is_dirty,
    Load: persist_file_load,
    Save: persist_file_save,
    SaveCompleted: persist_file_save_completed,
    GetCurFile: persist_file_get_cur_file,
};
pub static IDROPTARGET_VTBL: IDropTargetVtbl = IDropTargetVtbl {
    QueryInterface: drop_query,
    AddRef: drop_addref,
    Release: drop_release,
    DragEnter: drop_drag_enter,
    DragOver: drop_drag_over,
    DragLeave: drop_drag_leave,
    Drop: drop_drop,
};

unsafe fn iid_equal(left: REFIID, right: REFIID) -> bool {
    !left.is_null()
        && !right.is_null()
        && (*left).Data1 == (*right).Data1
        && (*left).Data2 == (*right).Data2
        && (*left).Data3 == (*right).Data3
        && (*left).Data4 == (*right).Data4
}

unsafe fn query_interface(
    this: *mut CExeDropHandler,
    riid: REFIID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    if ppv.is_null() {
        return E_POINTER;
    }
    *ppv = core::ptr::null_mut();
    if iid_equal(riid, core::ptr::addr_of!(IID_IUnknown))
        || iid_equal(riid, core::ptr::addr_of!(IID_IPersist))
    {
        *ppv = core::ptr::addr_of_mut!((*this).IPersist_iface).cast();
    } else if iid_equal(riid, core::ptr::addr_of!(IID_IPersistFile)) {
        *ppv = core::ptr::addr_of_mut!((*this).IPersistFile_iface).cast();
    } else if iid_equal(riid, core::ptr::addr_of!(IID_IDropTarget)) {
        *ppv = core::ptr::addr_of_mut!((*this).IDropTarget_iface).cast();
    } else {
        return E_NOINTERFACE;
    }
    add_ref(this);
    S_OK
}

unsafe fn add_ref(this: *mut CExeDropHandler) -> ULONG {
    (*this)
        .ref_count
        .fetch_add(1, core::sync::atomic::Ordering::SeqCst) as ULONG
        + 1
}
unsafe fn release(this: *mut CExeDropHandler) -> ULONG {
    let refs = (*this)
        .ref_count
        .fetch_sub(1, core::sync::atomic::Ordering::SeqCst)
        - 1;
    if refs == 0 {
        Shell32RustObjectUnlock();
        // ~CExeDropHandler: the Load() copy was leaked on every drop (upstream 763ce84cf55).
        crate::wine::shellole::SHFree((*this).sPathTarget.cast());
        HeapFree(GetProcessHeap(), 0, this.cast());
    }
    refs as ULONG
}

unsafe fn drag_enter(
    _this: *mut CExeDropHandler,
    _data: *mut IDataObject,
    _keys: DWORD,
    _point: POINTL,
    effect: *mut DWORD,
) -> HRESULT {
    if *effect != DROPEFFECT_NONE {
        *effect = DROPEFFECT_COPY;
    }
    S_OK
}
unsafe fn drag_over(
    _this: *mut CExeDropHandler,
    _keys: DWORD,
    _point: POINTL,
    effect: *mut DWORD,
) -> HRESULT {
    *effect = DROPEFFECT_COPY;
    S_OK
}
unsafe fn drag_leave(_this: *mut CExeDropHandler) -> HRESULT {
    S_OK
}

unsafe fn init_format_etc(format: *mut FORMATETC, cf: CLIPFORMAT, tymed: DWORD) {
    (*format).cfFormat = cf;
    (*format).ptd = core::ptr::null_mut();
    (*format).dwAspect = DVASPECT_CONTENT;
    (*format).lindex = -1;
    (*format).tymed = tymed;
}

unsafe fn append_argument(end: &mut LPWSTR, remaining: &mut usize, source: LPCWSTR, quote: bool) {
    let mut write = |unit: WCHAR| {
        if *remaining > 1 {
            **end = unit;
            *end = (*end).add(1);
            *remaining -= 1;
        }
    };
    if quote {
        write(b'"' as WCHAR);
    }
    let mut source = source;
    while !source.is_null() && *source != 0 {
        write(*source);
        source = source.add(1);
    }
    if quote {
        write(b'"' as WCHAR);
    }
    write(b' ' as WCHAR);
    if *remaining != 0 {
        **end = 0;
    }
}

unsafe fn drop_files(
    this: *mut CExeDropHandler,
    data: *mut IDataObject,
    _keys: DWORD,
    _point: POINTL,
    _effect: *mut DWORD,
) -> HRESULT {
    let mut format: FORMATETC = core::mem::zeroed();
    let mut medium: STGMEDIUM = core::mem::zeroed();
    init_format_etc(&mut format, CF_HDROP, TYMED_HGLOBAL);
    let mut buffer = [0u16; MAX_PATH * 2 + 8];
    let mut end = buffer.as_mut_ptr();
    let mut remaining = buffer.len();

    if ((*(*data).lpVtbl).GetData)(
        data,
        (&mut format as *mut FORMATETC).cast(),
        (&mut medium as *mut STGMEDIUM).cast(),
    ) >= 0
    {
        let global = medium.u.hGlobal;
        let drop = GlobalLock(global) as LPDROPFILES;
        if drop.is_null() {
            ReleaseStgMedium(&mut medium);
            return E_FAIL;
        }
        let mut source = (drop as *mut u8).add((*drop).pFiles as usize) as LPWSTR;
        while *source != 0 {
            let quote = !StrChrW(source, b' ' as WCHAR).is_null() && remaining > 3;
            append_argument(
                &mut end,
                &mut remaining,
                source,
                quote,
            );
            while *source != 0 {
                source = source.add(1);
            }
            source = source.add(1);
        }
        GlobalUnlock(global);
        ReleaseStgMedium(&mut medium);
    }

    ShellExecuteW(
        core::ptr::null_mut(),
        OPEN.as_ptr(),
        (*this).sPathTarget,
        buffer.as_ptr(),
        core::ptr::null(),
        SW_SHOWNORMAL,
    );
    S_OK
}

unsafe fn get_cur_file(_this: *mut CExeDropHandler, _file: *mut LPOLESTR) -> HRESULT {
    E_NOTIMPL
}
unsafe fn is_dirty(_this: *mut CExeDropHandler) -> HRESULT {
    E_NOTIMPL
}
unsafe fn wide_len(mut text: LPCWSTR) -> usize {
    let start = text;
    while !text.is_null() && *text != 0 {
        text = text.add(1);
    }
    text.offset_from(start) as usize
}
unsafe fn load(this: *mut CExeDropHandler, file: LPCOLESTR, _mode: DWORD) -> HRESULT {
    let len = wide_len(file);
    let target =
        crate::wine::shellole::SHAlloc((len + 1) * core::mem::size_of::<WCHAR>()) as LPWSTR;
    (*this).sPathTarget = target;
    if target.is_null() {
        return E_OUTOFMEMORY;
    }
    core::ptr::copy_nonoverlapping(file, target, len + 1);
    S_OK
}
unsafe fn save(_this: *mut CExeDropHandler, _file: LPCOLESTR, _remember: BOOL) -> HRESULT {
    E_NOTIMPL
}
unsafe fn save_completed(_this: *mut CExeDropHandler, _file: LPCOLESTR) -> HRESULT {
    E_NOTIMPL
}
unsafe fn get_class_id(this: *mut CExeDropHandler, class_id: *mut CLSID) -> HRESULT {
    if class_id.is_null() {
        return E_POINTER;
    }
    *class_id = *(*this).pclsid;
    S_OK
}

static OPEN: [WCHAR; 5] = [0x006F, 0x0070, 0x0065, 0x006E, 0];

#[no_mangle]
pub unsafe extern "system" fn CExeDropHandler_CreateInstance(
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
        core::mem::size_of::<CExeDropHandler>(),
    ) as *mut CExeDropHandler;
    if this.is_null() {
        return E_OUTOFMEMORY;
    }
    core::ptr::write(
        this,
        CExeDropHandler {
            IPersist_iface: IPersist {
                lpVtbl: core::ptr::addr_of!(IPERSIST_VTBL),
            },
            IPersistFile_iface: IPersistFile {
                lpVtbl: core::ptr::addr_of!(IPERSISTFILE_VTBL),
            },
            IDropTarget_iface: IDropTarget {
                lpVtbl: core::ptr::addr_of!(IDROPTARGET_VTBL),
            },
            ref_count: core::sync::atomic::AtomicI32::new(1),
            pclsid: core::ptr::addr_of!(CLSID_ExeDropHandler) as *mut CLSID,
            sPathTarget: core::ptr::null_mut(),
        },
    );
    Shell32RustObjectLock();
    let result = query_interface(this, riid, ppv);
    release(this);
    result
}
