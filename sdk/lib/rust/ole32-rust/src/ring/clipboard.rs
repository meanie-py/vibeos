//! OLE 2 clipboard support — translation of dll/win32/ole32/clipboard.c.
//!
//! OleSetClipboard / OleGetClipboard / OleFlushClipboard / OleIsCurrentClipboard,
//! the internal IDataObject ("snapshot") that wraps the Win32 clipboard, its hidden
//! owner window (WM_RENDERFORMAT / WM_RENDERALLFORMATS / WM_DESTROYCLIPBOARD), the
//! 'Ole Private Data' clipboard format and the CF_* <-> FORMATETC/STGMEDIUM
//! conversions.
use crate::prelude::*;
use core::ffi::{c_char, c_int, c_void};

// Both of these are DEFINED by a sibling ring module AND (stale) re-declared as an
// extern in crate::ole32_exports, so the bare name is ambiguous through the prelude
// globs. An explicit `use` outranks a glob — pin the real definitions.
// (ole32_exports.rs's own header says a ring-defined function must not be redeclared
// there; these two slipped through. Reported to the integrator.)
use crate::ring::memlockbytes::CreateILockBytesOnHGlobal;
use crate::ring::ole2::ReleaseStgMedium;

/// minwindef.h: `#define FALSE 0` — crate::win_consts carries TRUE but not FALSE.
const FALSE: BOOL = 0;

/// winuser.h:80 `#define CF_ENHMETAFILE 14`. crate::win_consts has CF_BITMAP /
/// CF_DIB / CF_HDROP / CF_METAFILEPICT / CF_OEMTEXT / CF_TEXT / CF_UNICODETEXT but
/// NOT this one — and its absence is not a compile error in `get_tymed_from_nonole_cf`,
/// it is a SILENT MISCOMPILE: rustc reads the unresolved `CF_ENHMETAFILE` match arm as
/// a fresh BINDING that matches everything, so the CF_METAFILEPICT and CF_BITMAP arms
/// below it become unreachable and every non-text format is handed TYMED_ENHMF.
/// (Reported to the integrator; belongs in win_consts.rs.)
const CF_ENHMETAFILE: DWORD = 14;

// ---------------------------------------------------------------------------
// Imports the shared surface does not carry yet (reported to the integrator).
// ABI notes: STDAPI/WINAPI => `extern "system"` (stdcall on i386). The two
// ole32-internal storage32 helpers are plain C functions (no WINAPI in
// storage32.h:166-167), so they are genuinely cdecl.
// ---------------------------------------------------------------------------
extern "system" {
    /// storage32.c (WINAPI) — not in crate::ole32_exports yet.
    fn StgCreateDocfileOnILockBytes(
        plkbyt: *mut ILockBytes,
        grfMode: DWORD,
        reserved: DWORD,
        ppstgOpen: *mut *mut IStorage,
    ) -> HRESULT;
    /// compobj.c (WINAPI) — not in crate::ole32_exports yet.
    fn ProgIDFromCLSID(clsid: REFCLSID, lplpszProgID: *mut LPOLESTR) -> HRESULT;
    /// user32 (WINAPI) — not in crate::user32 yet.
    fn DefWindowProcW(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
}

extern "C" {
    /// storage32.h:166 — `HRESULT STORAGE_CreateOleStream(IStorage*, DWORD)` (cdecl).
    fn STORAGE_CreateOleStream(pstg: *mut IStorage, flags: DWORD) -> HRESULT;
    /// storage32.h:167 — `HRESULT OLECONVERT_CreateCompObjStream(LPSTORAGE, LPCSTR)` (cdecl).
    fn OLECONVERT_CreateCompObjStream(pstg: *mut IStorage, strOleTypeName: *const c_char) -> HRESULT;
    /// libwine debug.c:274 — variadic, hence cdecl on every arch.
    fn wine_dbg_sprintf(format: *const c_char, ...) -> *const c_char;
}

/// `HENHMETAFILE` is ambiguous through the prelude: com-sys spells it
/// `*mut HENHMETAFILE__` (that is what `STGMEDIUM.hEnhMetaFile` is typed as) while
/// `crate::user32` spells it `*mut c_void` (what `CopyEnhMetaFileW` takes). Pin the
/// com-sys one for our signatures and cast at the user32 boundary.
type HENHMF = ::com_sys::HENHMETAFILE;

/// winuser.h — the two clipboard-render messages `crate::win_consts` is missing
/// (it has WM_RENDERALLFORMATS = 0x306 but not its two siblings).
const WM_RENDERFORMAT: UINT = 0x0305;
const WM_DESTROYCLIPBOARD: UINT = 0x0307;
/// winuser.h: `#define HWND_MESSAGE ((HWND)-3)`
const HWND_MESSAGE: HWND = -3isize as HWND;
/// wingdi.h: `#define SRCCOPY (DWORD)0x00CC0020`
const SRCCOPY: DWORD = 0x00CC_0020;

/// winuser.h `tagWNDCLASSEXW` — not in the shared surface (crate::user32 declares
/// RegisterClassExW with an opaque `*const c_void` param).
#[repr(C)]
#[derive(Copy, Clone)]
struct WNDCLASSEXW {
    cbSize: UINT,
    style: UINT,
    lpfnWndProc: Option<unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT>,
    cbClsExtra: c_int,
    cbWndExtra: c_int,
    hInstance: HINSTANCE,
    hIcon: HANDLE,
    hCursor: HANDLE,
    hbrBackground: HANDLE,
    lpszMenuName: *const u16,
    lpszClassName: *const u16,
    hIconSm: HANDLE,
}

/// wingdi.h `tagBITMAP` — only used by dup_bitmap's GetObjectW call.
#[repr(C)]
#[derive(Copy, Clone)]
struct BITMAP {
    bmType: LONG,
    bmWidth: LONG,
    bmHeight: LONG,
    bmWidthBytes: LONG,
    bmPlanes: WORD,
    bmBitsPixel: WORD,
    bmBits: LPVOID,
}

/// compobj_private.h:343 — `static inline HRESULT copy_formatetc(FORMATETC*, const FORMATETC*)`.
/// A private header inline, so it has no link symbol; it is reproduced here.
#[inline]
// `pub` because this is a compobj_private.h `static inline`, not a clipboard.c local:
// datacache.c and oleobj.c call it too, and reach it through the prelude's ring glob.
pub unsafe fn copy_formatetc(dst: *mut FORMATETC, src: *const FORMATETC) -> HRESULT {
    *dst = *src;
    if !(*src).ptd.is_null() {
        (*dst).ptd = CoTaskMemAlloc((*(*src).ptd).tdSize as SIZE_T) as *mut DVTARGETDEVICE;
        if (*dst).ptd.is_null() {
            return E_OUTOFMEMORY;
        }
        memcpy(
            (*dst).ptd as *mut c_void,
            (*src).ptd as *const c_void,
            (*(*src).ptd).tdSize as usize,
        );
    }
    S_OK
}

// ---- file-scope preamble (translated) ----

/// Structure of 'Ole Private Data' clipboard format.
#[repr(C)]
pub struct ole_priv_data_entry {
    pub fmtetc: FORMATETC,
    /// Has this cf been added to the list already
    pub first_use: DWORD,
    pub unk: [DWORD; 2],
}

pub type Pole_priv_data_entry = *mut ole_priv_data_entry;

#[repr(C)]
pub struct ole_priv_data {
    pub unk1: DWORD,
    /// in bytes of the entire structure
    pub size: DWORD,
    pub unk2: DWORD,
    /// no. of format entries
    pub count: DWORD,
    pub unk3: [DWORD; 2],
    /// array of size count (C flexible array — NEVER index this with a runtime
    /// index in Rust, that is a bounds-checked [T; 1]. Go through `entries_of()`.)
    pub entries: [ole_priv_data_entry; 1],
}

pub type Pole_priv_data = *mut ole_priv_data;

/// The C `data->entries[i]` on the trailing flexible array. `[T; 1]` in Rust is a
/// real one-element array, so `entries[i]` for i>0 is an out-of-bounds index (panic,
/// not the C behaviour). Every runtime-indexed access must go through this.
#[inline]
unsafe fn entries_of(data: *mut ole_priv_data) -> *mut ole_priv_data_entry {
    core::ptr::addr_of_mut!((*data).entries) as *mut ole_priv_data_entry
}

/// Consumer snapshot. Represents the state of the ole clipboard returned by
/// OleGetClipboard().
#[repr(C)]
pub struct snapshot {
    pub IDataObject_iface: IDataObject,
    pub ref_: LONG,
    /// Clipboard sequence number corresponding to this snapshot
    pub seq_no: DWORD,
    /// If we unmarshal a remote data object we hold a ref here
    pub data: *mut IDataObject,
}

pub type Psnapshot = *mut snapshot;

#[repr(C)]
pub struct ole_clipbrd {
    /// Latest consumer snapshot
    pub latest_snapshot: *mut snapshot,
    /// Hidden clipboard window
    pub window: HWND,
    /// Source object passed to OleSetClipboard
    pub src_data: *mut IDataObject,
    /// Cached result from the enumeration of src data object
    pub cached_enum: *mut ole_priv_data,
    /// Stream onto which to marshal src_data
    pub marshal_data: *mut IStream,
}

pub type Pole_clipbrd = *mut ole_clipbrd;

#[repr(C)]
pub struct PresentationDataHeader {
    pub unknown1: [BYTE; 28],
    pub dwObjectExtentX: DWORD,
    pub dwObjectExtentY: DWORD,
    pub dwSize: DWORD,
}

pub type PPresentationDataHeader = *mut PresentationDataHeader;

/// The one and only ole_clipbrd object, created on first use by clipbrd_create()
pub static mut theOleClipboard: *mut ole_clipbrd = core::ptr::null_mut();

static LATEST_SNAPSHOT_CS_NAME: &[u8; 34] = b"clipboard.c: clipboard last snap\0\0";

pub static mut latest_snapshot_cs_debug: CRITICAL_SECTION_DEBUG = CRITICAL_SECTION_DEBUG {
    Type: 0,
    CreatorBackTraceIndex: 0,
    CriticalSection: core::ptr::addr_of_mut!(latest_snapshot_cs),
    ProcessLocksList: LIST_ENTRY {
        Flink: unsafe {
            core::ptr::addr_of_mut!(latest_snapshot_cs_debug.ProcessLocksList) as *mut LIST_ENTRY
        },
        Blink: unsafe {
            core::ptr::addr_of_mut!(latest_snapshot_cs_debug.ProcessLocksList) as *mut LIST_ENTRY
        },
    },
    EntryCount: 0,
    ContentionCount: 0,
    WineDebugString: LATEST_SNAPSHOT_CS_NAME.as_ptr() as *const c_char,
};

pub static mut latest_snapshot_cs: CRITICAL_SECTION = CRITICAL_SECTION {
    DebugInfo: core::ptr::addr_of_mut!(latest_snapshot_cs_debug) as *mut c_void,
    LockCount: -1,
    RecursionCount: 0,
    OwningThread: core::ptr::null_mut(),
    LockSemaphore: core::ptr::null_mut(),
    SpinCount: 0,
};

/// Name of our registered OLE clipboard window class
pub static clipbrd_wndclass: [WCHAR; 16] = [
    'C' as u16, 'L' as u16, 'I' as u16, 'P' as u16, 'B' as u16, 'R' as u16, 'D' as u16,
    'W' as u16, 'N' as u16, 'D' as u16, 'C' as u16, 'L' as u16, 'A' as u16, 'S' as u16,
    'S' as u16, 0,
];

pub static mut ownerlink_clipboard_format: UINT = 0;
pub static mut filename_clipboard_format: UINT = 0;
pub static mut filenameW_clipboard_format: UINT = 0;
pub static mut dataobject_clipboard_format: UINT = 0;
pub static mut embedded_object_clipboard_format: UINT = 0;
pub static mut embed_source_clipboard_format: UINT = 0;
pub static mut custom_link_source_clipboard_format: UINT = 0;
pub static mut link_source_clipboard_format: UINT = 0;
pub static mut object_descriptor_clipboard_format: UINT = 0;
pub static mut link_source_descriptor_clipboard_format: UINT = 0;
pub static mut ole_private_data_clipboard_format: UINT = 0;
static mut wine_marshal_clipboard_format: UINT = 0;

#[repr(C)]
pub struct enum_fmtetc {
    pub IEnumFORMATETC_iface: IEnumFORMATETC,
    pub ref_: LONG,
    /// current enumerator position
    pub pos: UINT,
    pub data: *mut ole_priv_data,
}

pub type Penum_fmtetc = *mut enum_fmtetc;

pub static efvt: IEnumFORMATETCVtbl = IEnumFORMATETCVtbl {
    QueryInterface: Some(OLEClipbrd_IEnumFORMATETC_QueryInterface),
    AddRef: Some(OLEClipbrd_IEnumFORMATETC_AddRef),
    Release: Some(OLEClipbrd_IEnumFORMATETC_Release),
    Next: Some(OLEClipbrd_IEnumFORMATETC_Next),
    Skip: Some(OLEClipbrd_IEnumFORMATETC_Skip),
    Reset: Some(OLEClipbrd_IEnumFORMATETC_Reset),
    Clone: Some(OLEClipbrd_IEnumFORMATETC_Clone),
};

pub static snapshot_vtable: IDataObjectVtbl = IDataObjectVtbl {
    QueryInterface: Some(snapshot_QueryInterface),
    AddRef: Some(snapshot_AddRef),
    Release: Some(snapshot_Release),
    GetData: Some(snapshot_GetData),
    GetDataHere: Some(snapshot_GetDataHere),
    QueryGetData: Some(snapshot_QueryGetData),
    GetCanonicalFormatEtc: Some(snapshot_GetCanonicalFormatEtc),
    SetData: Some(snapshot_SetData),
    EnumFormatEtc: Some(snapshot_EnumFormatEtc),
    DAdvise: Some(snapshot_DAdvise),
    DUnadvise: Some(snapshot_DUnadvise),
    EnumDAdvise: Some(snapshot_EnumDAdvise),
};

/// Returns a ptr to a target device at a given offset from the start of the
/// ole_priv_data. Used when unpacking ole private data from the clipboard.
unsafe fn td_offs_to_ptr(data: *mut ole_priv_data, off: DWORD_PTR) -> *mut DVTARGETDEVICE {
    if off == 0 {
        return core::ptr::null_mut();
    }
    (data as *mut u8).offset(off as isize) as *mut DVTARGETDEVICE
}

/// Get the offset from the start of the ole_priv_data of the idx'th target device.
/// Used when packing ole private data to the clipboard.
unsafe fn td_get_offs(data: *mut ole_priv_data, idx: DWORD) -> DWORD_PTR {
    let entry = entries_of(data).add(idx as usize);
    if (*entry).fmtetc.ptd.is_null() {
        return 0;
    }
    ((*entry).fmtetc.ptd as *const u8).offset_from(data as *const u8) as DWORD_PTR
}

unsafe fn impl_from_IDataObject(iface: *mut IDataObject) -> *mut snapshot {
    CONTAINING_RECORD!(iface, snapshot, IDataObject_iface)
}

/// static INIT_ONCE init_once = INIT_ONCE_STATIC_INIT (get_ole_clipbrd's)
static mut clipbrd_init_once: crate::compobj::ffi::INIT_ONCE = core::ptr::null_mut();

/// wine-10: the clipboard object is created lazily on first use and lives until
/// process detach (clipbrd_destroy). It no longer depends on OleInitialize -- the
/// old `ole_inits == 0 -> CO_E_NOTINITIALIZED` here made OleGetClipboard fail
/// before OleInitialize, and Wine's test_get_clipboard_uninitialized then released
/// the NULL it got back. The entry points that DO need OLE initialized
/// (OleSetClipboard, OleFlushClipboard) check ole_inits themselves.
unsafe fn get_ole_clipbrd(clipbrd: *mut *mut ole_clipbrd) -> HRESULT {
    if crate::compobj::ffi::InitOnceExecuteOnce(
        core::ptr::addr_of_mut!(clipbrd_init_once),
        Some(clipbrd_create),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    ) == 0
    {
        *clipbrd = core::ptr::null_mut();
        return CO_E_NOTINITIALIZED;
    }

    *clipbrd = theOleClipboard;

    S_OK
}

unsafe fn dump_fmtetc(fmt: *const FORMATETC) -> *const c_char {
    if fmt.is_null() {
        return b"(null)\0".as_ptr() as *const c_char;
    }
    wine_dbg_sprintf(
        b"cf %04x ptd %p aspect %x lindex %d tymed %x\0".as_ptr() as *const c_char,
        (*fmt).cfFormat as UINT,
        (*fmt).ptd,
        (*fmt).dwAspect,
        (*fmt).lindex,
        (*fmt).tymed,
    )
}

/*---------------------------------------------------------------------*
 *  Implementation of the internal IEnumFORMATETC interface returned by
 *  the OLE clipboard's IDataObject.
 *---------------------------------------------------------------------*/

unsafe fn impl_from_IEnumFORMATETC(iface: *mut IEnumFORMATETC) -> *mut enum_fmtetc {
    CONTAINING_RECORD!(iface, enum_fmtetc, IEnumFORMATETC_iface)
}

stdmethod! {
fn OLEClipbrd_IEnumFORMATETC_QueryInterface(
    iface: *mut IEnumFORMATETC,
    riid: *const IID,
    ppvObj: *mut LPVOID,
) -> HRESULT {
    let _This = impl_from_IEnumFORMATETC(iface);

    TRACE!(b"(%p)->(IID: %s, %p)\n\0".as_ptr(), This, debugstr_guid(riid), ppvObj);

    *ppvObj = core::ptr::null_mut();

    if IsEqualIID(riid, &IID_IUnknown) || IsEqualIID(riid, &IID_IEnumFORMATETC) {
        *ppvObj = iface as LPVOID;
    }

    if !(*ppvObj).is_null() {
        IEnumFORMATETC_AddRef(*ppvObj as *mut IEnumFORMATETC);
        TRACE!(b"-- Interface: (%p)->(%p)\n\0".as_ptr(), ppvObj, *ppvObj);
        return S_OK;
    }

    TRACE!(b"-- Interface: E_NOINTERFACE\n\0".as_ptr());
    E_NOINTERFACE
}
}

stdmethod! {
fn OLEClipbrd_IEnumFORMATETC_AddRef(iface: *mut IEnumFORMATETC) -> ULONG {
    let This = impl_from_IEnumFORMATETC(iface);
    TRACE!(b"(%p)->(count=%u)\n\0".as_ptr(), This, (*This).ref_);

    InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref_)) as ULONG
}
}

stdmethod! {
fn OLEClipbrd_IEnumFORMATETC_Release(iface: *mut IEnumFORMATETC) -> ULONG {
    let This = impl_from_IEnumFORMATETC(iface);

    TRACE!(b"(%p)->(count=%u)\n\0".as_ptr(), This, (*This).ref_);

    let ref_count = InterlockedDecrement(core::ptr::addr_of_mut!((*This).ref_)) as ULONG;
    if ref_count == 0 {
        TRACE!(b"() - destroying IEnumFORMATETC(%p)\n\0".as_ptr(), This);
        HeapFree(GetProcessHeap(), 0, (*This).data as LPVOID);
        HeapFree(GetProcessHeap(), 0, This as LPVOID);
    }
    ref_count
}
}

stdmethod! {
fn OLEClipbrd_IEnumFORMATETC_Next(
    iface: *mut IEnumFORMATETC,
    celt: ULONG,
    rgelt: *mut FORMATETC,
    pceltFethed: *mut ULONG,
) -> HRESULT {
    let This = impl_from_IEnumFORMATETC(iface);
    let cfetch: UINT;
    let mut hres = S_FALSE;

    TRACE!(b"(%p)->(pos=%u)\n\0".as_ptr(), This, (*This).pos);

    if (*This).pos < (*(*This).data).count {
        let mut n = (*(*This).data).count - (*This).pos;
        if n >= celt {
            n = celt;
            hres = S_OK;
        }
        cfetch = n;

        for i in 0..cfetch {
            let entry = entries_of((*This).data).add((*This).pos as usize);
            hres = copy_formatetc(rgelt.add(i as usize), core::ptr::addr_of!((*entry).fmtetc));
            (*This).pos += 1;
            if FAILED(hres) {
                return hres;
            }
        }
    } else {
        cfetch = 0;
    }

    if !pceltFethed.is_null() {
        *pceltFethed = cfetch;
    }

    hres
}
}

stdmethod! {
fn OLEClipbrd_IEnumFORMATETC_Skip(iface: *mut IEnumFORMATETC, celt: ULONG) -> HRESULT {
    let This = impl_from_IEnumFORMATETC(iface);
    TRACE!(b"(%p)->(num=%u)\n\0".as_ptr(), This, celt);

    (*This).pos += celt;
    if (*This).pos > (*(*This).data).count {
        (*This).pos = (*(*This).data).count;
        return S_FALSE;
    }
    S_OK
}
}

stdmethod! {
fn OLEClipbrd_IEnumFORMATETC_Reset(iface: *mut IEnumFORMATETC) -> HRESULT {
    let This = impl_from_IEnumFORMATETC(iface);
    TRACE!(b"(%p)->()\n\0".as_ptr(), This);

    (*This).pos = 0;
    S_OK
}
}

stdmethod! {
fn OLEClipbrd_IEnumFORMATETC_Clone(
    iface: *mut IEnumFORMATETC,
    obj: *mut *mut IEnumFORMATETC,
) -> HRESULT {
    let This = impl_from_IEnumFORMATETC(iface);

    TRACE!(b"(%p)->(%p)\n\0".as_ptr(), This, obj);

    if obj.is_null() {
        return E_INVALIDARG;
    }
    *obj = core::ptr::null_mut();

    let size = (*(*This).data).size;
    let new_data = HeapAlloc(GetProcessHeap(), 0, size as SIZE_T) as *mut ole_priv_data;
    if new_data.is_null() {
        return E_OUTOFMEMORY;
    }
    memcpy(
        new_data as *mut c_void,
        (*This).data as *const c_void,
        size as usize,
    );

    /* Fixup any target device ptrs */
    let mut i: DWORD = 0;
    while i < (*(*This).data).count {
        (*entries_of(new_data).add(i as usize)).fmtetc.ptd =
            td_offs_to_ptr(new_data, td_get_offs((*This).data, i));
        i += 1;
    }

    enum_fmtetc_construct(new_data, (*This).pos, obj)
}
}

/// Creates an IEnumFORMATETC enumerator from ole_priv_data which it then owns.
unsafe fn enum_fmtetc_construct(
    data: *mut ole_priv_data,
    pos: UINT,
    obj: *mut *mut IEnumFORMATETC,
) -> HRESULT {
    *obj = core::ptr::null_mut();
    let ef = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<enum_fmtetc>() as SIZE_T,
    ) as *mut enum_fmtetc;
    if ef.is_null() {
        return E_OUTOFMEMORY;
    }

    (*ef).ref_ = 1;
    (*ef).IEnumFORMATETC_iface.lpVtbl =
        &efvt as *const IEnumFORMATETCVtbl as *mut IEnumFORMATETCVtbl;
    (*ef).data = data;
    (*ef).pos = pos;

    TRACE!(b"(%p)->()\n\0".as_ptr(), ef);
    *obj = core::ptr::addr_of_mut!((*ef).IEnumFORMATETC_iface);
    S_OK
}

/// Helper method to duplicate an HGLOBAL chunk of memory
unsafe fn dup_global_mem(src: HGLOBAL, flags: DWORD, dst: *mut HGLOBAL) -> HRESULT {
    *dst = core::ptr::null_mut();
    if src.is_null() {
        return S_FALSE;
    }

    let size: SIZE_T = GlobalSize(src);

    *dst = GlobalAlloc(flags, size);
    if (*dst).is_null() {
        return E_OUTOFMEMORY;
    }

    let src_ptr: LPVOID = GlobalLock(src);
    let dst_ptr: LPVOID = GlobalLock(*dst);

    memcpy(dst_ptr, src_ptr, size as usize);

    GlobalUnlock(*dst);
    GlobalUnlock(src);

    S_OK
}

/// Helper function to duplicate a handle to a METAFILEPICT, and the contained HMETAFILE.
unsafe fn dup_metafilepict(src: HGLOBAL, pdest: *mut HGLOBAL) -> HRESULT {
    let mut dest: HGLOBAL = core::ptr::null_mut();

    *pdest = core::ptr::null_mut();

    /* Copy the METAFILEPICT structure. */
    let hr = dup_global_mem(src, GMEM_DDESHARE | GMEM_MOVEABLE, &mut dest);
    if FAILED(hr) {
        return hr;
    }

    let dest_ptr = GlobalLock(dest) as *mut METAFILEPICT;
    if dest_ptr.is_null() {
        return E_FAIL;
    }

    /* Give the new METAFILEPICT a separate HMETAFILE. */
    (*dest_ptr).hMF = CopyMetaFileW((*dest_ptr).hMF as HANDLE, core::ptr::null()) as HMETAFILE;
    if !(*dest_ptr).hMF.is_null() {
        GlobalUnlock(dest);
        *pdest = dest;
        S_OK
    } else {
        GlobalUnlock(dest);
        GlobalFree(dest);
        E_FAIL
    }
}

/// Helper function to GlobalFree a handle to a METAFILEPICT, and also free the
/// contained HMETAFILE.
unsafe fn free_metafilepict(src: HGLOBAL) {
    let src_ptr = GlobalLock(src) as *mut METAFILEPICT;
    if !src_ptr.is_null() {
        DeleteMetaFile((*src_ptr).hMF);
        GlobalUnlock(src);
    }
    GlobalFree(src);
}

/// Helper function to duplicate an HBITMAP.
unsafe fn dup_bitmap(src: HBITMAP, pdest: *mut HBITMAP) -> HRESULT {
    let mut bm: BITMAP = core::mem::zeroed();

    let src_dc: HDC = CreateCompatibleDC(core::ptr::null_mut());
    let orig_src_bitmap: HGDIOBJ = SelectObject(src_dc, src as HGDIOBJ);
    GetObjectW(
        src as HANDLE,
        core::mem::size_of::<BITMAP>() as c_int,
        &mut bm as *mut BITMAP as LPVOID,
    );
    let dest: HBITMAP = CreateCompatibleBitmap(src_dc, bm.bmWidth, bm.bmHeight);
    if !dest.is_null() {
        let dest_dc: HDC = CreateCompatibleDC(core::ptr::null_mut());
        let orig_dest_bitmap: HGDIOBJ = SelectObject(dest_dc, dest as HGDIOBJ);
        BitBlt(dest_dc, 0, 0, bm.bmWidth, bm.bmHeight, src_dc, 0, 0, SRCCOPY);
        SelectObject(dest_dc, orig_dest_bitmap);
        DeleteDC(dest_dc);
    }
    SelectObject(src_dc, orig_src_bitmap);
    DeleteDC(src_dc);
    *pdest = dest;
    if !dest.is_null() {
        S_OK
    } else {
        E_FAIL
    }
}

/// This is clearly a hack and has no place in the clipboard code.
unsafe fn render_embed_source_hack(data: *mut IDataObject, fmt: LPFORMATETC) -> HRESULT {
    let _ = data; /* the C ignores its `data` arg too and goes at theOleClipboard->src_data */
    let mut std: STGMEDIUM = core::mem::zeroed();
    let mut hr: HRESULT;
    let mut ptrILockBytes: *mut ILockBytes = core::ptr::null_mut();

    core::ptr::write_bytes(&mut std as *mut STGMEDIUM, 0, 1);
    std.tymed = TYMED_ISTORAGE;
    (*fmt).tymed = TYMED_ISTORAGE;

    let hStorage: HGLOBAL = GlobalAlloc(GMEM_SHARE | GMEM_MOVEABLE, 0);
    if hStorage.is_null() {
        return E_OUTOFMEMORY;
    }
    hr = CreateILockBytesOnHGlobal(hStorage, 0, &mut ptrILockBytes);
    if FAILED(hr) {
        GlobalFree(hStorage);
        return hr;
    }

    hr = StgCreateDocfileOnILockBytes(
        ptrILockBytes,
        STGM_SHARE_EXCLUSIVE | STGM_READWRITE,
        0,
        core::ptr::addr_of_mut!(std.__bindgen_anon_1.pstg),
    );
    ILockBytes_Release(ptrILockBytes);

    hr = IDataObject_GetDataHere((*theOleClipboard).src_data, fmt, &mut std);
    if FAILED(hr) {
        WARN!(
            b"() : IDataObject_GetDataHere failed to render clipboard data! (%x)\n\0".as_ptr(),
            hr
        );
        GlobalFree(hStorage);
        return hr;
    }

    if true
    /* check whether the presentation data is already -not- present */
    {
        let mut fmt2: FORMATETC = core::mem::zeroed();
        let mut std2: STGMEDIUM = core::mem::zeroed();
        let mut mfp: *mut METAFILEPICT = core::ptr::null_mut();

        fmt2.cfFormat = CF_METAFILEPICT as CLIPFORMAT;
        fmt2.ptd = core::ptr::null_mut();
        fmt2.dwAspect = DVASPECT_CONTENT;
        fmt2.lindex = -1;
        fmt2.tymed = TYMED_MFPICT;

        core::ptr::write_bytes(&mut std2 as *mut STGMEDIUM, 0, 1);
        std2.tymed = TYMED_MFPICT;

        /* Get the metafile picture out of it */
        hr = IDataObject_GetData((*theOleClipboard).src_data, &mut fmt2, &mut std2);
        if SUCCEEDED(hr) {
            mfp = GlobalLock(std2.__bindgen_anon_1.hGlobal) as *mut METAFILEPICT;
        }

        if !mfp.is_null() {
            let name: [u16; 12] = [
                2, 'O' as u16, 'l' as u16, 'e' as u16, 'P' as u16, 'r' as u16, 'e' as u16,
                's' as u16, '0' as u16, '0' as u16, '0' as u16, 0,
            ];
            let mut pStream: *mut IStream = core::ptr::null_mut();
            let mut pdh: PresentationDataHeader = core::mem::zeroed();
            let mut clsID: CLSID = core::mem::zeroed();
            let mut strProgID: LPOLESTR = core::ptr::null_mut();
            let mut strOleTypeName: [c_char; 51] = [0; 51];
            let OlePresStreamHeader: [u8; 28] = [
                0xFF, 0xFF, 0xFF, 0xFF, 0x03, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x01,
                0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00,
            ];

            let mut nSize: c_int =
                GetMetaFileBitsEx((*mfp).hMF as HANDLE, 0, core::ptr::null_mut()) as c_int;

            core::ptr::write_bytes(&mut pdh as *mut PresentationDataHeader, 0, 1);
            memcpy(
                &mut pdh as *mut PresentationDataHeader as *mut c_void,
                OlePresStreamHeader.as_ptr() as *const c_void,
                core::mem::size_of_val(&OlePresStreamHeader),
            );

            pdh.dwObjectExtentX = (*mfp).xExt as DWORD;
            pdh.dwObjectExtentY = (*mfp).yExt as DWORD;
            pdh.dwSize = nSize as DWORD;

            hr = IStorage_CreateStream(
                std.__bindgen_anon_1.pstg,
                name.as_ptr() as LPCOLESTR,
                STGM_CREATE | STGM_SHARE_EXCLUSIVE | STGM_READWRITE,
                0,
                0,
                &mut pStream,
            );

            hr = IStream_Write(
                pStream,
                &pdh as *const PresentationDataHeader as *const c_void,
                core::mem::size_of::<PresentationDataHeader>() as ULONG,
                core::ptr::null_mut(),
            );

            let mfBits: LPVOID = HeapAlloc(GetProcessHeap(), 0, nSize as SIZE_T);
            nSize = GetMetaFileBitsEx((*mfp).hMF as HANDLE, nSize as UINT, mfBits) as c_int;

            hr = IStream_Write(
                pStream,
                mfBits as *const c_void,
                nSize as ULONG,
                core::ptr::null_mut(),
            );

            IStream_Release(pStream);

            HeapFree(GetProcessHeap(), 0, mfBits);

            GlobalUnlock(std2.__bindgen_anon_1.hGlobal);
            ReleaseStgMedium(&mut std2);

            ReadClassStg(std.__bindgen_anon_1.pstg, &mut clsID);
            ProgIDFromCLSID(&clsID, &mut strProgID);

            WideCharToMultiByte(
                CP_ACP,
                0,
                strProgID,
                -1,
                strOleTypeName.as_mut_ptr(),
                core::mem::size_of_val(&strOleTypeName) as c_int,
                core::ptr::null(),
                core::ptr::null_mut(),
            );
            STORAGE_CreateOleStream(std.__bindgen_anon_1.pstg, 0);
            OLECONVERT_CreateCompObjStream(
                std.__bindgen_anon_1.pstg,
                strOleTypeName.as_ptr(),
            );
            CoTaskMemFree(strProgID as LPVOID);
        }
    }

    if SetClipboardData((*fmt).cfFormat as UINT, hStorage).is_null() {
        WARN!(b"() : Failed to set rendered clipboard data into clipboard!\n\0".as_ptr());
        GlobalFree(hStorage);
        hr = CLIPBRD_E_CANT_SET;
    }

    ReleaseStgMedium(&mut std);
    hr
}

/// Returns the first entry that matches the provided clipboard format.
unsafe fn find_format_in_list(
    entries: *mut ole_priv_data_entry,
    num: DWORD,
    cf: UINT,
) -> *mut ole_priv_data_entry {
    for i in 0..num {
        if (*entries.add(i as usize)).fmtetc.cfFormat as UINT == cf {
            return entries.add(i as usize);
        }
    }
    core::ptr::null_mut()
}

/// Returns storage data in an HGLOBAL.
unsafe fn get_data_from_storage(
    data: *mut IDataObject,
    fmt: *mut FORMATETC,
    mem: *mut HGLOBAL,
) -> HRESULT {
    let mut stg: *mut IStorage = core::ptr::null_mut();
    let mut hr: HRESULT;
    let mut stg_fmt: FORMATETC;
    let mut med: STGMEDIUM = core::mem::zeroed();
    let mut lbs: *mut ILockBytes = core::ptr::null_mut();

    *mem = core::ptr::null_mut();

    let h: HGLOBAL = GlobalAlloc(GMEM_DDESHARE | GMEM_MOVEABLE, 0);
    if h.is_null() {
        return E_OUTOFMEMORY;
    }

    hr = CreateILockBytesOnHGlobal(h, 0, &mut lbs);
    if SUCCEEDED(hr) {
        hr = StgCreateDocfileOnILockBytes(
            lbs,
            STGM_CREATE | STGM_SHARE_EXCLUSIVE | STGM_READWRITE,
            0,
            &mut stg,
        );
        ILockBytes_Release(lbs);
    }
    if FAILED(hr) {
        GlobalFree(h);
        return hr;
    }

    stg_fmt = *fmt;
    med.tymed = TYMED_ISTORAGE;
    stg_fmt.tymed = TYMED_ISTORAGE;
    med.__bindgen_anon_1.pstg = stg;
    med.pUnkForRelease = core::ptr::null_mut();

    hr = IDataObject_GetDataHere(data, &mut stg_fmt, &mut med);
    if FAILED(hr) {
        memset(
            &mut med as *mut STGMEDIUM as *mut c_void,
            0,
            core::mem::size_of::<STGMEDIUM>(),
        );
        hr = IDataObject_GetData(data, &mut stg_fmt, &mut med);
        if FAILED(hr) {
            /* end: */
            IStorage_Release(stg);
            GlobalFree(h);
            return hr;
        }

        hr = IStorage_CopyTo(
            med.__bindgen_anon_1.pstg,
            0,
            core::ptr::null(),
            core::ptr::null_mut(),
            stg,
        );
        ReleaseStgMedium(&mut med);
        if FAILED(hr) {
            /* end: */
            IStorage_Release(stg);
            GlobalFree(h);
            return hr;
        }
    }
    *mem = h;

    /* end: */
    IStorage_Release(stg);
    if FAILED(hr) {
        GlobalFree(h);
    }
    hr
}

/// Returns stream data in an HGLOBAL.
unsafe fn get_data_from_stream(
    data: *mut IDataObject,
    fmt: *mut FORMATETC,
    mem: *mut HGLOBAL,
) -> HRESULT {
    let mut stm: *mut IStream = core::ptr::null_mut();
    let mut hr: HRESULT;
    let mut stm_fmt: FORMATETC;
    let mut med: STGMEDIUM = core::mem::zeroed();

    *mem = core::ptr::null_mut();

    let h: HGLOBAL = GlobalAlloc(GMEM_DDESHARE | GMEM_MOVEABLE, 0);
    if h.is_null() {
        return E_OUTOFMEMORY;
    }

    hr = CreateStreamOnHGlobal(h, 0, &mut stm);
    if FAILED(hr) {
        /* error: */
        if !stm.is_null() {
            IStream_Release(stm);
        }
        GlobalFree(h);
        return hr;
    }

    stm_fmt = *fmt;
    med.tymed = TYMED_ISTREAM;
    stm_fmt.tymed = TYMED_ISTREAM;
    med.__bindgen_anon_1.pstm = stm;
    med.pUnkForRelease = core::ptr::null_mut();

    hr = IDataObject_GetDataHere(data, &mut stm_fmt, &mut med);
    if FAILED(hr) {
        let mut offs: LARGE_INTEGER = core::mem::zeroed();
        let mut pos: ULARGE_INTEGER = core::mem::zeroed();

        memset(
            &mut med as *mut STGMEDIUM as *mut c_void,
            0,
            core::mem::size_of::<STGMEDIUM>(),
        );
        hr = IDataObject_GetData(data, &mut stm_fmt, &mut med);
        if FAILED(hr) {
            /* error: */
            if !stm.is_null() {
                IStream_Release(stm);
            }
            GlobalFree(h);
            return hr;
        }

        offs.QuadPart = 0;
        IStream_Seek(
            med.__bindgen_anon_1.pstm,
            offs,
            STREAM_SEEK_CUR,
            &mut pos,
        );
        IStream_Seek(
            med.__bindgen_anon_1.pstm,
            offs,
            STREAM_SEEK_SET,
            core::ptr::null_mut(),
        );
        hr = IStream_CopyTo(
            med.__bindgen_anon_1.pstm,
            stm,
            pos,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        ReleaseStgMedium(&mut med);
        if FAILED(hr) {
            /* error: */
            if !stm.is_null() {
                IStream_Release(stm);
            }
            GlobalFree(h);
            return hr;
        }
    }
    *mem = h;
    IStream_Release(stm);
    S_OK
}

/// Returns global data in an HGLOBAL.
unsafe fn get_data_from_global(
    data: *mut IDataObject,
    fmt: *mut FORMATETC,
    mem: *mut HGLOBAL,
) -> HRESULT {
    let mut h: HGLOBAL = core::ptr::null_mut();
    let mut hr: HRESULT;
    let mut mem_fmt: FORMATETC;
    let mut med: STGMEDIUM = core::mem::zeroed();

    *mem = core::ptr::null_mut();

    mem_fmt = *fmt;
    mem_fmt.tymed = TYMED_HGLOBAL;
    core::ptr::write_bytes(&mut med as *mut STGMEDIUM, 0, 1);

    hr = IDataObject_GetData(data, &mut mem_fmt, &mut med);
    if FAILED(hr) {
        return hr;
    }

    hr = dup_global_mem(
        med.__bindgen_anon_1.hGlobal,
        GMEM_DDESHARE | GMEM_MOVEABLE,
        &mut h,
    );

    if SUCCEEDED(hr) {
        *mem = h;
    }

    ReleaseStgMedium(&mut med);

    hr
}

unsafe fn get_data_from_enhmetafile(
    data: *mut IDataObject,
    fmt: *mut FORMATETC,
    mem: *mut HGLOBAL,
) -> HRESULT {
    let mut hr: HRESULT;
    let mut mem_fmt: FORMATETC;
    let mut med: STGMEDIUM = core::mem::zeroed();

    *mem = core::ptr::null_mut();

    mem_fmt = *fmt;
    mem_fmt.tymed = TYMED_ENHMF;
    memset(
        core::ptr::addr_of_mut!(med) as *mut c_void,
        0,
        core::mem::size_of::<STGMEDIUM>(),
    );

    hr = IDataObject_GetData(
        data,
        core::ptr::addr_of_mut!(mem_fmt),
        core::ptr::addr_of_mut!(med),
    );
    if FAILED(hr) {
        return hr;
    }

    let copy: HENHMF = CopyEnhMetaFileW(
        med.__bindgen_anon_1.hEnhMetaFile as crate::user32::HENHMETAFILE,
        core::ptr::null(),
    ) as HENHMF;
    if !copy.is_null() {
        *mem = copy as HGLOBAL;
    } else {
        hr = E_FAIL;
    }

    ReleaseStgMedium(core::ptr::addr_of_mut!(med));

    hr
}

unsafe fn get_data_from_metafilepict(
    data: *mut IDataObject,
    fmt: *mut FORMATETC,
    mem: *mut HGLOBAL,
) -> HRESULT {
    let mut copy: HGLOBAL = core::ptr::null_mut();
    let mut hr: HRESULT;
    let mut mem_fmt: FORMATETC;
    let mut med: STGMEDIUM = core::mem::zeroed();

    *mem = core::ptr::null_mut();

    mem_fmt = *fmt;
    mem_fmt.tymed = TYMED_MFPICT;
    memset(
        core::ptr::addr_of_mut!(med) as *mut c_void,
        0,
        core::mem::size_of::<STGMEDIUM>(),
    );

    hr = IDataObject_GetData(
        data,
        core::ptr::addr_of_mut!(mem_fmt),
        core::ptr::addr_of_mut!(med),
    );
    if FAILED(hr) {
        return hr;
    }

    hr = dup_metafilepict(
        med.__bindgen_anon_1.hMetaFilePict as HGLOBAL,
        core::ptr::addr_of_mut!(copy),
    );

    if SUCCEEDED(hr) {
        *mem = copy;
    }

    ReleaseStgMedium(core::ptr::addr_of_mut!(med));

    hr
}

/// Returns bitmap in an HBITMAP.
unsafe fn get_data_from_bitmap(
    data: *mut IDataObject,
    fmt: *mut FORMATETC,
    hbm: *mut HBITMAP,
) -> HRESULT {
    let mut copy: HBITMAP = core::ptr::null_mut();
    let mut hr: HRESULT;
    let mut mem_fmt: FORMATETC;
    let mut med: STGMEDIUM = core::mem::zeroed();

    *hbm = core::ptr::null_mut();

    mem_fmt = *fmt;
    mem_fmt.tymed = TYMED_GDI;
    core::ptr::write_bytes(&mut med as *mut STGMEDIUM, 0, 1);

    hr = IDataObject_GetData(data, &mut mem_fmt, &mut med);
    if FAILED(hr) {
        return hr;
    }

    hr = dup_bitmap(med.__bindgen_anon_1.hBitmap, &mut copy);

    if SUCCEEDED(hr) {
        *hbm = copy;
    }

    ReleaseStgMedium(&mut med);

    hr
}

/// Render the clipboard data. Note that this call will delegate to the source
/// data object.
unsafe fn render_format(data: *mut IDataObject, fmt: LPFORMATETC) -> HRESULT {
    /* HGLOBAL unless otherwise specified */
    let mut clip_data: HANDLE = core::ptr::null_mut();
    let mut hr: HRESULT;

    /* Embed source hack */
    if (*fmt).cfFormat as UINT == embed_source_clipboard_format {
        return render_embed_source_hack(data, fmt);
    }

    if (*fmt).tymed & TYMED_ISTORAGE != 0 {
        hr = get_data_from_storage(data, fmt, &mut clip_data);
    } else if (*fmt).tymed & TYMED_ISTREAM != 0 {
        hr = get_data_from_stream(data, fmt, &mut clip_data);
    } else if (*fmt).tymed & TYMED_HGLOBAL != 0 {
        hr = get_data_from_global(data, fmt, &mut clip_data);
    } else if (*fmt).tymed & TYMED_ENHMF != 0 {
        hr = get_data_from_enhmetafile(data, fmt, &mut clip_data);
    } else if (*fmt).tymed & TYMED_MFPICT != 0 {
        /* Returns global handle to METAFILEPICT, containing a copied HMETAFILE */
        hr = get_data_from_metafilepict(data, fmt, &mut clip_data);
    } else if (*fmt).tymed & TYMED_GDI != 0 {
        /* Returns HBITMAP not HGLOBAL */
        hr = get_data_from_bitmap(
            data,
            fmt,
            &mut clip_data as *mut HANDLE as *mut HBITMAP,
        );
    } else {
        FIXME!(b"Unhandled tymed %x\n\0".as_ptr(), (*fmt).tymed);
        hr = DV_E_FORMATETC;
    }

    if SUCCEEDED(hr) {
        if SetClipboardData((*fmt).cfFormat as UINT, clip_data).is_null() {
            WARN!(b"() : Failed to set rendered clipboard data into clipboard!\n\0".as_ptr());
            if (*fmt).tymed & TYMED_MFPICT != 0 {
                free_metafilepict(clip_data);
            } else if (*fmt).tymed & TYMED_GDI != 0 {
                DeleteObject(clip_data as HGDIOBJ);
            } else {
                GlobalFree(clip_data);
            }
            hr = CLIPBRD_E_CANT_SET;
        }
    }

    hr
}

/*---------------------------------------------------------------------*
 *  Implementation of the internal IDataObject interface exposed by
 *  the OLE clipboard.
 *---------------------------------------------------------------------*/

stdmethod! {
fn snapshot_QueryInterface(
    iface: *mut IDataObject,
    riid: *const IID,
    ppvObject: *mut *mut c_void,
) -> HRESULT {
    let This = impl_from_IDataObject(iface);
    TRACE!(b"(%p)->(IID:%s, %p)\n\0".as_ptr(), This, debugstr_guid(riid), ppvObject);

    if This.is_null() || ppvObject.is_null() {
        return E_INVALIDARG;
    }

    *ppvObject = core::ptr::null_mut();

    if IsEqualIID(&IID_IUnknown, riid) || IsEqualIID(&IID_IDataObject, riid) {
        *ppvObject = iface as *mut c_void;
    } else {
        WARN!(
            b"() : asking for unsupported interface %s\n\0".as_ptr(),
            debugstr_guid(riid)
        );
        return E_NOINTERFACE;
    }

    IUnknown_AddRef(*ppvObject as *mut IUnknown);

    S_OK
}
}

stdmethod! {
fn snapshot_AddRef(iface: *mut IDataObject) -> ULONG {
    let This = impl_from_IDataObject(iface);

    TRACE!(b"(%p)->(count=%u)\n\0".as_ptr(), This, (*This).ref_);

    InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref_)) as ULONG
}
}

stdmethod! {
fn snapshot_Release(iface: *mut IDataObject) -> ULONG {
    let This = impl_from_IDataObject(iface);

    TRACE!(b"(%p)->(count=%u)\n\0".as_ptr(), This, (*This).ref_);

    let ref_count = InterlockedDecrement(core::ptr::addr_of_mut!((*This).ref_)) as ULONG;

    if ref_count == 0 {
        EnterCriticalSection(core::ptr::addr_of_mut!(latest_snapshot_cs));
        if (*This).ref_ != 0 {
            LeaveCriticalSection(core::ptr::addr_of_mut!(latest_snapshot_cs));
            return ref_count;
        }
        if (*theOleClipboard).latest_snapshot == This {
            (*theOleClipboard).latest_snapshot = core::ptr::null_mut();
        }
        LeaveCriticalSection(core::ptr::addr_of_mut!(latest_snapshot_cs));

        if !(*This).data.is_null() {
            IDataObject_Release((*This).data);
        }
        HeapFree(GetProcessHeap(), 0, This as LPVOID);
    }

    ref_count
}
}

/// Return the window that owns the ole clipboard.
///
/// If the clipboard is flushed or not owned by ole this will return NULL.
unsafe fn get_current_ole_clip_window() -> HWND {
    let h: HGLOBAL = GetClipboardData(dataobject_clipboard_format);
    if h.is_null() {
        return core::ptr::null_mut();
    }
    let ptr: *mut HWND = GlobalLock(h) as *mut HWND;
    if ptr.is_null() {
        return core::ptr::null_mut();
    }
    let wnd: HWND = *ptr;
    GlobalUnlock(h);
    wnd
}

/// Return an unmarshalled IDataObject if there is a current (ie non-flushed)
/// object on the ole clipboard.
unsafe fn get_current_dataobject(data: *mut *mut IDataObject) -> HRESULT {
    let mut hr: HRESULT;
    let wnd: HWND = get_current_ole_clip_window();
    let mut stm: *mut IStream = core::ptr::null_mut();
    let mut pos: LARGE_INTEGER = core::mem::zeroed();

    *data = core::ptr::null_mut();
    if wnd.is_null() {
        return S_FALSE;
    }

    let h: HGLOBAL = GetClipboardData(wine_marshal_clipboard_format);
    if h.is_null() {
        return S_FALSE;
    }
    if GlobalSize(h) <= 1 {
        return S_FALSE;
    }
    let ptr: LPVOID = GlobalLock(h);
    if ptr.is_null() {
        return S_FALSE;
    }

    hr = CreateStreamOnHGlobal(core::ptr::null_mut(), 1, &mut stm);
    if FAILED(hr) {
        /* end: */
        GlobalUnlock(h);
        return hr;
    }

    hr = IStream_Write(stm, ptr, GlobalSize(h) as ULONG, core::ptr::null_mut());
    if SUCCEEDED(hr) {
        pos.QuadPart = 0;
        IStream_Seek(stm, pos, STREAM_SEEK_SET, core::ptr::null_mut());
        hr = CoUnmarshalInterface(stm, &IID_IDataObject, data as *mut LPVOID);
    }
    IStream_Release(stm);

    /* end: */
    GlobalUnlock(h);
    hr
}

unsafe fn get_tymed_from_nonole_cf(cf: UINT) -> DWORD {
    if cf >= 0xc000 {
        return TYMED_ISTREAM | TYMED_HGLOBAL;
    }

    match cf {
        // wine-10 adds CF_HDROP (dropped files are an HGLOBAL DROPFILES).
        CF_TEXT | CF_OEMTEXT | CF_UNICODETEXT | CF_HDROP => TYMED_ISTREAM | TYMED_HGLOBAL,
        CF_ENHMETAFILE => TYMED_ENHMF,
        CF_METAFILEPICT => TYMED_MFPICT,
        CF_BITMAP => TYMED_GDI,
        _ => {
            FIXME!(b"returning TYMED_NULL for cf %04x\n\0".as_ptr(), cf);
            TYMED_NULL
        }
    }
}

/// Returns a copy of the Ole Private Data
unsafe fn get_priv_data(data: *mut *mut ole_priv_data) -> HRESULT {
    let hr: HRESULT = S_OK;
    let mut ret: *mut ole_priv_data = core::ptr::null_mut();

    *data = core::ptr::null_mut();

    let handle: HGLOBAL = GetClipboardData(ole_private_data_clipboard_format);
    if !handle.is_null() {
        let src = GlobalLock(handle) as *mut ole_priv_data;
        if !src.is_null() {
            /* FIXME: sanity check on size */
            ret = HeapAlloc(GetProcessHeap(), 0, (*src).size as SIZE_T) as *mut ole_priv_data;
            if ret.is_null() {
                GlobalUnlock(handle);
                return E_OUTOFMEMORY;
            }
            memcpy(
                ret as *mut c_void,
                src as *const c_void,
                (*src).size as usize,
            );
            GlobalUnlock(handle);

            /* Fixup any target device offsets to ptrs */
            for i in 0..(*ret).count {
                let entry_ptr = entries_of(ret).add(i as usize);
                (*entry_ptr).fmtetc.ptd = td_offs_to_ptr(ret, (*entry_ptr).fmtetc.ptd as DWORD_PTR);
            }
        }
    }

    if ret.is_null() {
        /* Non-ole data */
        let mut cf: UINT;
        let mut count: DWORD = 0;
        let mut size: DWORD;

        cf = 0;
        loop {
            cf = EnumClipboardFormats(cf);
            if cf == 0 {
                break;
            }
            let mut buf: [u16; 256] = [0; 256];
            if GetClipboardFormatNameW(cf, buf.as_mut_ptr(), buf.len() as c_int) != 0 {
                TRACE!(b"cf %04x %s\n\0".as_ptr(), cf, debugstr_w(buf.as_ptr()));
            } else {
                TRACE!(b"cf %04x\n\0".as_ptr(), cf);
            }
            count += 1;
        }
        TRACE!(b"count %d\n\0".as_ptr(), count);

        size = core::mem::offset_of!(ole_priv_data, entries) as DWORD;
        size += count * core::mem::size_of::<ole_priv_data_entry>() as DWORD;

        /* There are holes in fmtetc so zero init */
        ret = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, size as SIZE_T) as *mut ole_priv_data;
        if ret.is_null() {
            return E_OUTOFMEMORY;
        }
        (*ret).size = size;
        (*ret).count = count;

        cf = 0;
        let mut idx: DWORD = 0;
        loop {
            cf = EnumClipboardFormats(cf);
            if cf == 0 {
                break;
            }
            let entry_ptr = entries_of(ret).add(idx as usize);
            (*entry_ptr).fmtetc.cfFormat = cf as CLIPFORMAT;
            (*entry_ptr).fmtetc.ptd = core::ptr::null_mut();
            (*entry_ptr).fmtetc.dwAspect = DVASPECT_CONTENT;
            (*entry_ptr).fmtetc.lindex = -1;
            (*entry_ptr).fmtetc.tymed = get_tymed_from_nonole_cf(cf);
            (*entry_ptr).first_use = 1;
            idx += 1;
        }
    }

    *data = ret;
    hr
}

/// Returns a stg medium with a copy of the global handle
unsafe fn get_stgmed_for_global(h: HGLOBAL, med: *mut STGMEDIUM) -> HRESULT {
    (*med).pUnkForRelease = core::ptr::null_mut();
    (*med).tymed = TYMED_NULL;

    let hr = dup_global_mem(
        h,
        GMEM_MOVEABLE,
        core::ptr::addr_of_mut!((*med).__bindgen_anon_1.hGlobal),
    );

    if SUCCEEDED(hr) {
        (*med).tymed = TYMED_HGLOBAL;
    }

    hr
}

/// Returns a stg medium with a stream based on the handle
unsafe fn get_stgmed_for_stream(h: HGLOBAL, med: *mut STGMEDIUM) -> HRESULT {
    let mut dst: HGLOBAL = core::ptr::null_mut();

    (*med).pUnkForRelease = core::ptr::null_mut();
    (*med).tymed = TYMED_NULL;

    let mut hr = dup_global_mem(h, GMEM_MOVEABLE, &mut dst);
    if FAILED(hr) {
        return hr;
    }

    hr = CreateStreamOnHGlobal(
        dst,
        1,
        core::ptr::addr_of_mut!((*med).__bindgen_anon_1.pstm),
    );
    if FAILED(hr) {
        GlobalFree(dst);
        return hr;
    }

    (*med).tymed = TYMED_ISTREAM;
    hr
}

/// Returns a stg medium with a storage based on the handle
unsafe fn get_stgmed_for_storage(h: HGLOBAL, med: *mut STGMEDIUM) -> HRESULT {
    let mut dst: HGLOBAL = core::ptr::null_mut();
    let mut lbs: *mut ILockBytes = core::ptr::null_mut();

    (*med).pUnkForRelease = core::ptr::null_mut();
    (*med).tymed = TYMED_NULL;

    let mut hr = dup_global_mem(h, GMEM_MOVEABLE, &mut dst);
    if FAILED(hr) {
        return hr;
    }

    hr = CreateILockBytesOnHGlobal(dst, 1, &mut lbs);
    if FAILED(hr) {
        GlobalFree(dst);
        return hr;
    }

    hr = StgIsStorageILockBytes(lbs);
    if hr != S_OK {
        ILockBytes_Release(lbs);
        GlobalFree(dst);
        return if SUCCEEDED(hr) { E_FAIL } else { hr };
    }

    hr = StgOpenStorageOnILockBytes(
        lbs,
        core::ptr::null_mut(),
        STGM_SHARE_EXCLUSIVE | STGM_READWRITE,
        core::ptr::null_mut(),
        0,
        core::ptr::addr_of_mut!((*med).__bindgen_anon_1.pstg),
    );
    ILockBytes_Release(lbs);
    if FAILED(hr) {
        GlobalFree(dst);
        return hr;
    }

    (*med).tymed = TYMED_ISTORAGE;
    hr
}

/// Returns a stg medium with an enhanced metafile based on the handle
unsafe fn get_stgmed_for_emf(hemf: HENHMF, med: *mut STGMEDIUM) -> HRESULT {
    (*med).pUnkForRelease = core::ptr::null_mut();
    (*med).tymed = TYMED_NULL;

    (*med).__bindgen_anon_1.hEnhMetaFile = CopyEnhMetaFileW(
        hemf as crate::user32::HENHMETAFILE,
        core::ptr::null(),
    ) as HENHMF;
    if (*med).__bindgen_anon_1.hEnhMetaFile.is_null() {
        return E_OUTOFMEMORY;
    }
    (*med).tymed = TYMED_ENHMF;
    S_OK
}

/// Returns a stg medium with a bitmap based on the handle
unsafe fn get_stgmed_for_bitmap(hbmp: HBITMAP, med: *mut STGMEDIUM) -> HRESULT {
    (*med).pUnkForRelease = core::ptr::null_mut();
    (*med).tymed = TYMED_NULL;

    let hr = dup_bitmap(hbmp, core::ptr::addr_of_mut!((*med).__bindgen_anon_1.hBitmap));

    if FAILED(hr) {
        return hr;
    }

    (*med).tymed = TYMED_GDI;
    S_OK
}

unsafe fn string_off_equal(
    t1: *const DVTARGETDEVICE,
    off1: WORD,
    t2: *const DVTARGETDEVICE,
    off2: WORD,
) -> BOOL {
    if off1 == 0 && off2 == 0 {
        return TRUE as BOOL;
    }
    if off1 == 0 || off2 == 0 {
        return FALSE;
    }

    let str1 = (t1 as *const u8).offset(off1 as isize) as *const u16;
    let str2 = (t2 as *const u8).offset(off2 as isize) as *const u16;

    if wcscmp(str1, str2) == 0 {
        TRUE as BOOL
    } else {
        FALSE
    }
}

unsafe fn td_equal(t1: *const DVTARGETDEVICE, t2: *const DVTARGETDEVICE) -> BOOL {
    if t1.is_null() && t2.is_null() {
        return TRUE as BOOL;
    }
    if t1.is_null() || t2.is_null() {
        return FALSE;
    }

    if string_off_equal(t1, (*t1).tdDriverNameOffset, t2, (*t2).tdDriverNameOffset) == FALSE {
        return FALSE;
    }
    if string_off_equal(t1, (*t1).tdDeviceNameOffset, t2, (*t2).tdDeviceNameOffset) == FALSE {
        return FALSE;
    }
    if string_off_equal(t1, (*t1).tdPortNameOffset, t2, (*t2).tdPortNameOffset) == FALSE {
        return FALSE;
    }

    /* FIXME check devmode? */

    TRUE as BOOL
}

stdmethod! {
fn snapshot_GetData(
    iface: *mut IDataObject,
    fmt: *mut FORMATETC,
    med: *mut STGMEDIUM,
) -> HRESULT {
    let This = impl_from_IDataObject(iface);
    let mut hr: HRESULT = S_OK;
    let mut enum_data: *mut ole_priv_data = core::ptr::null_mut();
    let entry: *mut ole_priv_data_entry;
    let mut mask: DWORD;

    TRACE!(
        b"(%p, %p {%s}, %p)\n\0".as_ptr(),
        iface,
        fmt,
        dump_fmtetc(fmt),
        med,
    );

    if fmt.is_null() || med.is_null() {
        return E_INVALIDARG;
    }

    memset(
        med as *mut c_void,
        0,
        core::mem::size_of::<STGMEDIUM>(),
    );

    if OpenClipboard(core::ptr::null_mut()) == 0 {
        return CLIPBRD_E_CANT_OPEN;
    }

    if (*This).data.is_null() {
        hr = get_current_dataobject(core::ptr::addr_of_mut!((*This).data));
    }

    if !(*This).data.is_null() {
        hr = IDataObject_GetData((*This).data, fmt, med);
        if SUCCEEDED(hr) {
            CloseClipboard();
            return hr;
        }
    }

    'end: {
        if (*fmt).lindex != -1 {
            hr = DV_E_FORMATETC;
            break 'end;
        }

        if IsClipboardFormatAvailable((*fmt).cfFormat as UINT) == 0 {
            hr = DV_E_FORMATETC;
            break 'end;
        }

        hr = get_priv_data(&mut enum_data);
        if FAILED(hr) {
            break 'end;
        }

        entry = find_format_in_list(
            entries_of(enum_data),
            (*enum_data).count,
            (*fmt).cfFormat as UINT,
        );
        if !entry.is_null() {
            if td_equal((*fmt).ptd, (*entry).fmtetc.ptd) == FALSE {
                hr = DV_E_FORMATETC;
                break 'end;
            }
            mask = (*fmt).tymed & (*entry).fmtetc.tymed;
            if mask == 0
                && ((*entry).fmtetc.tymed & (TYMED_ISTREAM | TYMED_HGLOBAL | TYMED_ISTORAGE)) != 0
            {
                mask = (*fmt).tymed & (TYMED_ISTREAM | TYMED_HGLOBAL | TYMED_ISTORAGE);
            }
        } else {
            /* non-Ole format */
            mask = (*fmt).tymed & get_tymed_from_nonole_cf((*fmt).cfFormat as UINT);
        }

        if mask == 0 {
            hr = DV_E_TYMED;
            break 'end;
        }

        let h: HANDLE = GetClipboardData((*fmt).cfFormat as UINT);
        if h.is_null() {
            hr = DV_E_FORMATETC;
            break 'end;
        }

        if (mask & TYMED_HGLOBAL) != 0 {
            hr = get_stgmed_for_global(h, med);
        } else if (mask & TYMED_ISTREAM) != 0 {
            hr = get_stgmed_for_stream(h, med);
        } else if (mask & TYMED_ISTORAGE) != 0 {
            hr = get_stgmed_for_storage(h, med);
        } else if (mask & TYMED_ENHMF) != 0 {
            hr = get_stgmed_for_emf(h as HENHMF, med);
        } else if (mask & TYMED_GDI) != 0 {
            hr = get_stgmed_for_bitmap(h as HBITMAP, med);
        } else {
            FIXME!(
                b"Unhandled tymed - mask %x req tymed %x\n\0".as_ptr(),
                mask,
                (*fmt).tymed,
            );
            hr = E_FAIL;
            break 'end;
        }
    }

    /* end: */
    HeapFree(GetProcessHeap(), 0, enum_data as LPVOID);
    if CloseClipboard() == 0 {
        hr = CLIPBRD_E_CANT_CLOSE;
    }
    hr
}
}

stdmethod! {
fn snapshot_GetDataHere(
    iface: *mut IDataObject,
    fmt: *mut FORMATETC,
    med: *mut STGMEDIUM,
) -> HRESULT {
    let This = impl_from_IDataObject(iface);
    let mut hr: HRESULT = S_OK;
    let mut enum_data: *mut ole_priv_data = core::ptr::null_mut();
    let entry: *mut ole_priv_data_entry;
    let supported: DWORD;

    if fmt.is_null() || med.is_null() {
        return E_INVALIDARG;
    }

    TRACE!(
        b"(%p, %p {%s}, %p (tymed %x)\n\0".as_ptr(),
        iface,
        fmt,
        dump_fmtetc(fmt),
        med,
        (*med).tymed,
    );

    if OpenClipboard(core::ptr::null_mut()) == 0 {
        return CLIPBRD_E_CANT_OPEN;
    }

    if (*This).data.is_null() {
        hr = get_current_dataobject(core::ptr::addr_of_mut!((*This).data));
    }

    if !(*This).data.is_null() {
        hr = IDataObject_GetDataHere((*This).data, fmt, med);
        if SUCCEEDED(hr) {
            CloseClipboard();
            return hr;
        }
    }

    'end: {
        let h: HANDLE = GetClipboardData((*fmt).cfFormat as UINT);
        if h.is_null() {
            hr = DV_E_FORMATETC;
            break 'end;
        }

        hr = get_priv_data(core::ptr::addr_of_mut!(enum_data));
        if FAILED(hr) {
            break 'end;
        }

        entry = find_format_in_list(
            entries_of(enum_data),
            (*enum_data).count,
            (*fmt).cfFormat as UINT,
        );
        if !entry.is_null() {
            if td_equal((*fmt).ptd, (*entry).fmtetc.ptd) == FALSE {
                hr = DV_E_FORMATETC;
                break 'end;
            }
            supported = (*entry).fmtetc.tymed;
        } else {
            /* non-Ole format */
            supported = TYMED_HGLOBAL;
        }

        match (*med).tymed {
            TYMED_HGLOBAL => {
                let src_size = GlobalSize(h);
                let dst_size = GlobalSize((*med).__bindgen_anon_1.hGlobal);
                hr = E_FAIL;
                if dst_size >= src_size {
                    let src = GlobalLock(h);
                    let dst = GlobalLock((*med).__bindgen_anon_1.hGlobal);

                    memcpy(dst, src, src_size as usize);
                    GlobalUnlock((*med).__bindgen_anon_1.hGlobal);
                    GlobalUnlock(h);
                    hr = S_OK;
                }
            }
            TYMED_ISTREAM => {
                let src_size = GlobalSize(h);
                let src = GlobalLock(h);
                hr = IStream_Write(
                    (*med).__bindgen_anon_1.pstm,
                    src,
                    src_size as ULONG,
                    core::ptr::null_mut(),
                );
                GlobalUnlock(h);
            }
            TYMED_ISTORAGE => {
                let mut copy: STGMEDIUM = core::mem::zeroed();
                if (supported & TYMED_ISTORAGE) == 0 {
                    hr = E_FAIL;
                    break 'end;
                }
                hr = get_stgmed_for_storage(h, core::ptr::addr_of_mut!(copy));
                if SUCCEEDED(hr) {
                    hr = IStorage_CopyTo(
                        copy.__bindgen_anon_1.pstg,
                        0,
                        core::ptr::null(),
                        core::ptr::null_mut(),
                        (*med).__bindgen_anon_1.pstg,
                    );
                    ReleaseStgMedium(core::ptr::addr_of_mut!(copy));
                }
            }
            _ => {
                FIXME!(
                    b"Unhandled tymed - supported %x req tymed %x\n\0".as_ptr(),
                    supported,
                    (*med).tymed,
                );
                hr = E_FAIL;
                break 'end;
            }
        }
    }

    /* end: */
    HeapFree(GetProcessHeap(), 0, enum_data as LPVOID);
    if CloseClipboard() == 0 {
        hr = CLIPBRD_E_CANT_CLOSE;
    }
    hr
}
}

// The OLE Clipboard's implementation of this method delegates to a data source
// if there is one or wraps around the windows clipboard function
// IsClipboardFormatAvailable() otherwise.
stdmethod! {
fn snapshot_QueryGetData(iface: *mut IDataObject, fmt: *mut FORMATETC) -> HRESULT {
    TRACE!(b"(%p, %p {%s})\n\0".as_ptr(), iface, fmt, dump_fmtetc(fmt));

    if fmt.is_null() {
        return E_INVALIDARG;
    }

    if (*fmt).dwAspect != DVASPECT_CONTENT {
        return DV_E_FORMATETC;
    }

    if (*fmt).lindex != -1 {
        return DV_E_FORMATETC;
    }

    if IsClipboardFormatAvailable((*fmt).cfFormat as UINT) != 0 {
        S_OK
    } else {
        DV_E_CLIPFORMAT
    }
}
}

stdmethod! {
fn snapshot_GetCanonicalFormatEtc(
    iface: *mut IDataObject,
    fmt_in: *mut FORMATETC,
    fmt_out: *mut FORMATETC,
) -> HRESULT {
    TRACE!(b"(%p, %p, %p)\n\0".as_ptr(), iface, fmt_in, fmt_out);

    if fmt_in.is_null() || fmt_out.is_null() {
        return E_INVALIDARG;
    }

    *fmt_out = *fmt_in;
    DATA_S_SAMEFORMATETC
}
}

// The OLE Clipboard does not implement this method
stdmethod! {
fn snapshot_SetData(
    iface: *mut IDataObject,
    fmt: *mut FORMATETC,
    med: *mut STGMEDIUM,
    release: BOOL,
) -> HRESULT {
    TRACE!(
        b"(%p, %p, %p, %d): not implemented\n\0".as_ptr(),
        iface,
        fmt,
        med,
        release
    );
    E_NOTIMPL
}
}

stdmethod! {
fn snapshot_EnumFormatEtc(
    iface: *mut IDataObject,
    dir: DWORD,
    enum_fmt: *mut *mut IEnumFORMATETC,
) -> HRESULT {
    let mut hr: HRESULT;
    let mut data: *mut ole_priv_data = core::ptr::null_mut();

    TRACE!(b"(%p, %x, %p)\n\0".as_ptr(), iface, dir, enum_fmt);

    *enum_fmt = core::ptr::null_mut();

    if dir != DATADIR_GET {
        return E_NOTIMPL;
    }
    if OpenClipboard(core::ptr::null_mut()) == 0 {
        return CLIPBRD_E_CANT_OPEN;
    }

    hr = get_priv_data(core::ptr::addr_of_mut!(data));

    if SUCCEEDED(hr) {
        hr = enum_fmtetc_construct(data, 0, enum_fmt);
    }

    /* end: */
    if CloseClipboard() == 0 {
        hr = CLIPBRD_E_CANT_CLOSE;
    }
    hr
}
}

// The OLE Clipboard does not implement this method
stdmethod! {
fn snapshot_DAdvise(
    iface: *mut IDataObject,
    fmt: *mut FORMATETC,
    flags: DWORD,
    sink: *mut IAdviseSink,
    conn: *mut DWORD,
) -> HRESULT {
    TRACE!(
        b"(%p, %p, %x, %p, %p): not implemented\n\0".as_ptr(),
        iface,
        fmt,
        flags,
        sink,
        conn,
    );
    E_NOTIMPL
}
}

// The OLE Clipboard does not implement this method
stdmethod! {
fn snapshot_DUnadvise(iface: *mut IDataObject, conn: DWORD) -> HRESULT {
    TRACE!(b"(%p, %d): not implemented\n\0".as_ptr(), iface, conn);
    E_NOTIMPL
}
}

// The OLE Clipboard does not implement this method
stdmethod! {
fn snapshot_EnumDAdvise(
    iface: *mut IDataObject,
    enum_advise: *mut *mut IEnumSTATDATA,
) -> HRESULT {
    TRACE!(
        b"(%p, %p): not implemented\n\0".as_ptr(),
        iface,
        enum_advise,
    );
    E_NOTIMPL
}
}

/*---------------------------------------------------------------------*
 *           Internal implementation methods for the OLE clipboard
 *---------------------------------------------------------------------*/

unsafe fn snapshot_construct(seq_no: DWORD) -> *mut snapshot {
    let This: *mut snapshot = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<snapshot>() as SIZE_T,
    ) as *mut snapshot;

    if This.is_null() {
        return core::ptr::null_mut();
    }

    (*This).IDataObject_iface.lpVtbl =
        &snapshot_vtable as *const IDataObjectVtbl as *mut IDataObjectVtbl;
    (*This).ref_ = 0;
    (*This).seq_no = seq_no;
    (*This).data = core::ptr::null_mut();

    This
}

unsafe fn register_clipboard_formats() {
    static OwnerLink: [u16; 10] = [
        b'O' as u16, b'w' as u16, b'n' as u16, b'e' as u16, b'r' as u16, b'L' as u16,
        b'i' as u16, b'n' as u16, b'k' as u16, 0,
    ];
    static FileName: [u16; 9] = [
        b'F' as u16, b'i' as u16, b'l' as u16, b'e' as u16, b'N' as u16, b'a' as u16,
        b'm' as u16, b'e' as u16, 0,
    ];
    static FileNameW: [u16; 10] = [
        b'F' as u16, b'i' as u16, b'l' as u16, b'e' as u16, b'N' as u16, b'a' as u16,
        b'm' as u16, b'e' as u16, b'W' as u16, 0,
    ];
    static DataObject: [u16; 11] = [
        b'D' as u16, b'a' as u16, b't' as u16, b'a' as u16, b'O' as u16, b'b' as u16,
        b'j' as u16, b'e' as u16, b'c' as u16, b't' as u16, 0,
    ];
    static EmbeddedObject: [u16; 16] = [
        b'E' as u16, b'm' as u16, b'b' as u16, b'e' as u16, b'd' as u16, b'd' as u16,
        b'e' as u16, b'd' as u16, b' ' as u16, b'O' as u16, b'b' as u16, b'j' as u16,
        b'e' as u16, b'c' as u16, b't' as u16, 0,
    ];
    static EmbedSource: [u16; 13] = [
        b'E' as u16, b'm' as u16, b'b' as u16, b'e' as u16, b'd' as u16, b' ' as u16,
        b'S' as u16, b'o' as u16, b'u' as u16, b'r' as u16, b'c' as u16, b'e' as u16, 0,
    ];
    static CustomLinkSource: [u16; 19] = [
        b'C' as u16, b'u' as u16, b's' as u16, b't' as u16, b'o' as u16, b'm' as u16,
        b' ' as u16, b'L' as u16, b'i' as u16, b'n' as u16, b'k' as u16, b' ' as u16,
        b'S' as u16, b'o' as u16, b'u' as u16, b'r' as u16, b'c' as u16, b'e' as u16, 0,
    ];
    static LinkSource: [u16; 12] = [
        b'L' as u16, b'i' as u16, b'n' as u16, b'k' as u16, b' ' as u16, b'S' as u16,
        b'o' as u16, b'u' as u16, b'r' as u16, b'c' as u16, b'e' as u16, 0,
    ];
    static ObjectDescriptor: [u16; 18] = [
        b'O' as u16, b'b' as u16, b'j' as u16, b'e' as u16, b'c' as u16, b't' as u16,
        b' ' as u16, b'D' as u16, b'e' as u16, b's' as u16, b'c' as u16, b'r' as u16,
        b'i' as u16, b'p' as u16, b't' as u16, b'o' as u16, b'r' as u16, 0,
    ];
    static LinkSourceDescriptor: [u16; 23] = [
        b'L' as u16, b'i' as u16, b'n' as u16, b'k' as u16, b' ' as u16, b'S' as u16,
        b'o' as u16, b'u' as u16, b'r' as u16, b'c' as u16, b'e' as u16, b' ' as u16,
        b'D' as u16, b'e' as u16, b's' as u16, b'c' as u16, b'r' as u16, b'i' as u16,
        b'p' as u16, b't' as u16, b'o' as u16, b'r' as u16, 0,
    ];
    static OlePrivateData: [u16; 17] = [
        b'O' as u16, b'l' as u16, b'e' as u16, b' ' as u16, b'P' as u16, b'r' as u16,
        b'i' as u16, b'v' as u16, b'a' as u16, b't' as u16, b'e' as u16, b' ' as u16,
        b'D' as u16, b'a' as u16, b't' as u16, b'a' as u16, 0,
    ];
    static WineMarshalledDataObject: [u16; 27] = [
        b'W' as u16, b'i' as u16, b'n' as u16, b'e' as u16, b' ' as u16, b'M' as u16,
        b'a' as u16, b'r' as u16, b's' as u16, b'h' as u16, b'a' as u16, b'l' as u16,
        b'l' as u16, b'e' as u16, b'd' as u16, b' ' as u16, b'D' as u16, b'a' as u16,
        b't' as u16, b'a' as u16, b'O' as u16, b'b' as u16, b'j' as u16, b'e' as u16,
        b'c' as u16, b't' as u16, 0,
    ];

    ownerlink_clipboard_format = RegisterClipboardFormatW(OwnerLink.as_ptr());
    filename_clipboard_format = RegisterClipboardFormatW(FileName.as_ptr());
    filenameW_clipboard_format = RegisterClipboardFormatW(FileNameW.as_ptr());
    dataobject_clipboard_format = RegisterClipboardFormatW(DataObject.as_ptr());
    embedded_object_clipboard_format = RegisterClipboardFormatW(EmbeddedObject.as_ptr());
    embed_source_clipboard_format = RegisterClipboardFormatW(EmbedSource.as_ptr());
    custom_link_source_clipboard_format = RegisterClipboardFormatW(CustomLinkSource.as_ptr());
    link_source_clipboard_format = RegisterClipboardFormatW(LinkSource.as_ptr());
    object_descriptor_clipboard_format = RegisterClipboardFormatW(ObjectDescriptor.as_ptr());
    link_source_descriptor_clipboard_format =
        RegisterClipboardFormatW(LinkSourceDescriptor.as_ptr());
    ole_private_data_clipboard_format = RegisterClipboardFormatW(OlePrivateData.as_ptr());

    wine_marshal_clipboard_format = RegisterClipboardFormatW(WineMarshalledDataObject.as_ptr());
}

/// wine-10 clipbrd_create: the InitOnce callback that creates the clipboard object.
unsafe extern "system" fn clipbrd_create(
    _init_once: *mut crate::compobj::ffi::INIT_ONCE,
    _parameter: LPVOID,
    _context: *mut LPVOID,
) -> BOOL {
    TRACE!(b"()\n\0".as_ptr());

    register_clipboard_formats();

    {
        let clipbrd = HeapAlloc(
            GetProcessHeap(),
            0,
            core::mem::size_of::<ole_clipbrd>() as SIZE_T,
        ) as *mut ole_clipbrd;
        if clipbrd.is_null() {
            ERR!(b"No memory.\n\0".as_ptr());
            return 0;
        }

        (*clipbrd).latest_snapshot = core::ptr::null_mut();
        (*clipbrd).window = core::ptr::null_mut();
        (*clipbrd).src_data = core::ptr::null_mut();
        (*clipbrd).cached_enum = core::ptr::null_mut();

        let h: HGLOBAL = GlobalAlloc(GMEM_DDESHARE | GMEM_MOVEABLE, 0);
        if h.is_null() {
            ERR!(b"No memory.\n\0".as_ptr());
            HeapFree(GetProcessHeap(), 0, clipbrd as LPVOID);
            return 0;
        }

        if FAILED(CreateStreamOnHGlobal(
            h,
            1,
            core::ptr::addr_of_mut!((*clipbrd).marshal_data),
        )) {
            ERR!(b"CreateStreamOnHGlobal failed.\n\0".as_ptr());
            GlobalFree(h);
            HeapFree(GetProcessHeap(), 0, clipbrd as LPVOID);
            return 0;
        }

        theOleClipboard = clipbrd;
    }
    1
}

/// Enumerate all formats supported by the source and make those formats available
/// using delayed rendering using SetClipboardData. Cache the enumeration list and
/// make that list visible as the 'Ole Private Data' format on the clipboard.
unsafe fn set_clipboard_formats(clipbrd: *mut ole_clipbrd, data: *mut IDataObject) -> HRESULT {
    let mut fmt: FORMATETC = core::mem::zeroed();
    let mut enum_fmt: *mut IEnumFORMATETC = core::ptr::null_mut();
    let mut target_offset: DWORD_PTR;
    let mut count: DWORD = 0;
    let mut needed: DWORD = core::mem::size_of::<ole_priv_data>() as DWORD;
    let mut idx: DWORD;

    let hr = IDataObject_EnumFormatEtc(data, DATADIR_GET, &mut enum_fmt);
    if FAILED(hr) {
        return hr;
    }

    while IEnumFORMATETC_Next(enum_fmt, 1, &mut fmt, core::ptr::null_mut()) == S_OK {
        count += 1;
        needed += core::mem::size_of::<ole_priv_data_entry>() as DWORD;
        if !fmt.ptd.is_null() {
            needed += (*fmt.ptd).tdSize;
            CoTaskMemFree(fmt.ptd as LPVOID);
        }
    }

    /* Windows pads the list with two empty ole_priv_data_entries, one after the
     * entries array and one after the target device data. Allocating with zero
     * init to zero these pads. */

    /* initialisation of needed includes one of these. */
    needed += core::mem::size_of::<ole_priv_data_entry>() as DWORD;
    let priv_data_handle: HGLOBAL = GlobalAlloc(
        GMEM_DDESHARE | GMEM_MOVEABLE | GMEM_ZEROINIT,
        needed as SIZE_T,
    );
    let priv_data = GlobalLock(priv_data_handle) as *mut ole_priv_data;

    (*priv_data).unk1 = 0;
    (*priv_data).size = needed;
    (*priv_data).unk2 = 1;
    (*priv_data).count = count;
    (*priv_data).unk3[0] = 0;
    (*priv_data).unk3[1] = 0;

    IEnumFORMATETC_Reset(enum_fmt);

    idx = 0;
    /* FIELD_OFFSET(ole_priv_data, entries[count + 1]) — count entries + one pad. */
    target_offset = (core::mem::offset_of!(ole_priv_data, entries)
        + (count as usize + 1) * core::mem::size_of::<ole_priv_data_entry>())
        as DWORD_PTR;

    while IEnumFORMATETC_Next(enum_fmt, 1, &mut fmt, core::ptr::null_mut()) == S_OK {
        TRACE!(b"%s\n\0".as_ptr(), dump_fmtetc(&fmt));

        let entry = entries_of(priv_data).add(idx as usize);
        (*entry).fmtetc = fmt;
        if !fmt.ptd.is_null() {
            memcpy(
                (priv_data as *mut u8).add(target_offset as usize) as *mut c_void,
                fmt.ptd as *const c_void,
                (*fmt.ptd).tdSize as usize,
            );
            (*entry).fmtetc.ptd = target_offset as *mut DVTARGETDEVICE;
            target_offset += (*fmt.ptd).tdSize as DWORD_PTR;
            CoTaskMemFree(fmt.ptd as LPVOID);
        }

        (*entry).first_use = if find_format_in_list(
            entries_of(priv_data),
            idx,
            fmt.cfFormat as UINT,
        )
        .is_null()
        {
            1
        } else {
            0
        };
        (*entry).unk[0] = 0;
        (*entry).unk[1] = 0;

        if (*entry).first_use != 0 {
            SetClipboardData(fmt.cfFormat as UINT, core::ptr::null_mut());
        }

        idx += 1;
    }

    IEnumFORMATETC_Release(enum_fmt);

    /* Cache the list and fixup any target device offsets to ptrs */
    (*clipbrd).cached_enum =
        HeapAlloc(GetProcessHeap(), 0, needed as SIZE_T) as *mut ole_priv_data;
    memcpy(
        (*clipbrd).cached_enum as *mut c_void,
        priv_data as *const c_void,
        needed as usize,
    );
    idx = 0;
    while idx < (*(*clipbrd).cached_enum).count {
        let entry = entries_of((*clipbrd).cached_enum).add(idx as usize);
        (*entry).fmtetc.ptd =
            td_offs_to_ptr((*clipbrd).cached_enum, (*entry).fmtetc.ptd as DWORD_PTR);
        idx += 1;
    }

    GlobalUnlock(priv_data_handle);
    if SetClipboardData(ole_private_data_clipboard_format, priv_data_handle).is_null() {
        GlobalFree(priv_data_handle);
        return CLIPBRD_E_CANT_SET;
    }

    S_OK
}

unsafe fn get_clipbrd_window(clipbrd: *mut ole_clipbrd, wnd: *mut HWND) -> HRESULT {
    /* __REACTOS__: The clipboard window can get destroyed if the thread that
     * created it dies so we may need to create it again */
    if IsWindow((*clipbrd).window) == 0 {
        (*clipbrd).window = create_clipbrd_window();
    }

    if (*clipbrd).window.is_null() {
        (*clipbrd).window = create_clipbrd_window();
    }

    *wnd = (*clipbrd).window;
    if !(*wnd).is_null() {
        S_OK
    } else {
        E_FAIL
    }
}

/// Releases the data and sets the stream back to zero size.
unsafe fn release_marshal_data(stm: *mut IStream) {
    let mut pos: LARGE_INTEGER = core::mem::zeroed();
    let mut size: ULARGE_INTEGER = core::mem::zeroed();
    pos.QuadPart = 0;
    size.QuadPart = 0;

    IStream_Seek(stm, pos, STREAM_SEEK_SET, core::ptr::null_mut());
    CoReleaseMarshalData(stm);
    IStream_Seek(stm, pos, STREAM_SEEK_SET, core::ptr::null_mut());
    IStream_SetSize(stm, size);
}

/// Sets the marshalled dataobject to the clipboard. In the flushed case we set a
/// zero sized HGLOBAL to clear the old marshalled data.
unsafe fn expose_marshalled_dataobject(
    clipbrd: *mut ole_clipbrd,
    data: *mut IDataObject,
) -> HRESULT {
    let mut h: HGLOBAL = core::ptr::null_mut();

    if !data.is_null() {
        let mut h_stm: HGLOBAL = core::ptr::null_mut();
        GetHGlobalFromStream((*clipbrd).marshal_data, &mut h_stm);
        dup_global_mem(h_stm, GMEM_DDESHARE | GMEM_MOVEABLE, &mut h);
    } else {
        /* flushed */
        h = GlobalAlloc(GMEM_DDESHARE | GMEM_MOVEABLE, 1);
    }

    if h.is_null() {
        return E_OUTOFMEMORY;
    }

    if SetClipboardData(wine_marshal_clipboard_format, h).is_null() {
        GlobalFree(h);
        return CLIPBRD_E_CANT_SET;
    }
    S_OK
}

/// Clears and sets the clipboard's src IDataObject.
///
/// To marshal the source dataobject we do something rather different from Windows.
/// We set a clipboard format which contains the marshalled data.
/// Windows sets two window props one of which is an IID, the other is an endpoint
/// number.
unsafe fn set_src_dataobject(clipbrd: *mut ole_clipbrd, data: *mut IDataObject) -> HRESULT {
    let mut wnd: HWND = core::ptr::null_mut();

    let mut hr = get_clipbrd_window(clipbrd, &mut wnd);
    if FAILED(hr) {
        return hr;
    }

    if !(*clipbrd).src_data.is_null() {
        release_marshal_data((*clipbrd).marshal_data);

        IDataObject_Release((*clipbrd).src_data);
        (*clipbrd).src_data = core::ptr::null_mut();
        HeapFree(GetProcessHeap(), 0, (*clipbrd).cached_enum as LPVOID);
        (*clipbrd).cached_enum = core::ptr::null_mut();
    }

    if !data.is_null() {
        let mut unk: *mut IUnknown = core::ptr::null_mut();

        IDataObject_AddRef(data);
        (*clipbrd).src_data = data;

        IDataObject_QueryInterface(
            data,
            &IID_IUnknown,
            &mut unk as *mut *mut IUnknown as *mut *mut c_void,
        );
        hr = CoMarshalInterface(
            (*clipbrd).marshal_data,
            &IID_IDataObject,
            unk,
            MSHCTX_LOCAL,
            core::ptr::null_mut(),
            MSHLFLAGS_TABLESTRONG,
        );
        /* Don't hold a ref on IUnknown, we have one on IDataObject. */
        IUnknown_Release(unk);
        if FAILED(hr) {
            return hr;
        }
        hr = set_clipboard_formats(clipbrd, data);
    }
    hr
}

/// wine-10 clipbrd_uninitialize (OleUninitialize): drops the source data object and
/// the clipboard window, but KEEPS the object -- get_ole_clipbrd's InitOnce will
/// never recreate it, so freeing it here would leave theOleClipboard dangling for
/// the next OleInitialize. The object goes in clipbrd_destroy at process detach.
pub unsafe fn clipbrd_uninitialize() {
    let clipbrd: *mut ole_clipbrd = theOleClipboard;

    TRACE!(b"()\n\0".as_ptr());

    if !clipbrd.is_null() {
        static OLE32W: [u16; 6] = [
            b'o' as u16, b'l' as u16, b'e' as u16, b'3' as u16, b'2' as u16, 0,
        ];
        let hinst: HINSTANCE = GetModuleHandleW(OLE32W.as_ptr());

        /* OleUninitialize() does not release the reference to the dataobject, so
         * take an additional reference here. This reference is then leaked. */
        if !(*clipbrd).src_data.is_null() {
            IDataObject_AddRef((*clipbrd).src_data);
            set_src_dataobject(clipbrd, core::ptr::null_mut());
        }

        if !(*clipbrd).window.is_null() {
            DestroyWindow((*clipbrd).window);
            UnregisterClassW(clipbrd_wndclass.as_ptr(), hinst);
            (*clipbrd).window = core::ptr::null_mut();
        }
    }
}

/// wine-10 clipbrd_destroy (DllMain, DLL_PROCESS_DETACH): free the object itself.
pub unsafe fn clipbrd_destroy() {
    let clipbrd: *mut ole_clipbrd = theOleClipboard;

    if clipbrd.is_null() {
        return;
    }

    clipbrd_uninitialize();

    IStream_Release((*clipbrd).marshal_data);
    HeapFree(GetProcessHeap(), 0, clipbrd as LPVOID);
    theOleClipboard = core::ptr::null_mut();
}

unsafe extern "system" fn clipbrd_wndproc(
    hwnd: HWND,
    message: UINT,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let mut clipbrd: *mut ole_clipbrd = core::ptr::null_mut();

    get_ole_clipbrd(&mut clipbrd);
    /* __REACTOS__ */
    if clipbrd.is_null() {
        return DefWindowProcW(hwnd, message, wparam, lparam);
    }

    match message {
        WM_RENDERFORMAT => {
            /* __REACTOS__ */
            if !(*clipbrd).cached_enum.is_null() {
                let cf = wparam as UINT;

                TRACE!(b"(): WM_RENDERFORMAT(cfFormat=%x)\n\0".as_ptr(), cf);
                let entry = find_format_in_list(
                    entries_of((*clipbrd).cached_enum),
                    (*(*clipbrd).cached_enum).count,
                    cf,
                );

                if !entry.is_null() {
                    render_format((*clipbrd).src_data, core::ptr::addr_of_mut!((*entry).fmtetc));
                }
            }
        }

        WM_RENDERALLFORMATS => {
            TRACE!(b"(): WM_RENDERALLFORMATS\n\0".as_ptr());

            if clipbrd.is_null() || (*clipbrd).cached_enum.is_null() {
                return 0;
            }
            let entries = entries_of((*clipbrd).cached_enum);
            let mut i: DWORD = 0;
            while i < (*(*clipbrd).cached_enum).count {
                let entry = entries.add(i as usize);
                if (*entry).first_use != 0 {
                    render_format((*clipbrd).src_data, core::ptr::addr_of_mut!((*entry).fmtetc));
                }
                i += 1;
            }
        }

        WM_DESTROYCLIPBOARD => {
            TRACE!(b"(): WM_DESTROYCLIPBOARD\n\0".as_ptr());

            set_src_dataobject(clipbrd, core::ptr::null_mut());
        }

        _ => {
            return DefWindowProcW(hwnd, message, wparam, lparam);
        }
    }

    0
}

unsafe fn create_clipbrd_window() -> HWND {
    static ole32W: [u16; 6] = [
        b'o' as u16, b'l' as u16, b'e' as u16, b'3' as u16, b'2' as u16, 0,
    ];
    static title: [u16; 16] = [
        b'C' as u16, b'l' as u16, b'i' as u16, b'p' as u16, b'b' as u16, b'o' as u16,
        b'a' as u16, b'r' as u16, b'd' as u16, b'W' as u16, b'i' as u16, b'n' as u16,
        b'd' as u16, b'o' as u16, b'w' as u16, 0,
    ];

    let hinst: HINSTANCE = GetModuleHandleW(ole32W.as_ptr());

    let mut class: WNDCLASSEXW = core::mem::zeroed();
    class.cbSize = core::mem::size_of::<WNDCLASSEXW>() as UINT;
    class.style = 0;
    class.lpfnWndProc = Some(clipbrd_wndproc);
    class.cbClsExtra = 0;
    class.cbWndExtra = 0;
    class.hInstance = hinst;
    class.hIcon = core::ptr::null_mut();
    class.hCursor = core::ptr::null_mut();
    class.hbrBackground = core::ptr::null_mut();
    class.lpszMenuName = core::ptr::null();
    class.lpszClassName = clipbrd_wndclass.as_ptr();
    class.hIconSm = core::ptr::null_mut();

    RegisterClassExW(&class as *const WNDCLASSEXW as *const c_void);

    /* winuser.h: CreateWindowW(...) is CreateWindowExW(0, ...) */
    CreateWindowExW(
        0,
        clipbrd_wndclass.as_ptr(),
        title.as_ptr(),
        WS_POPUP | WS_CLIPSIBLINGS | WS_OVERLAPPED,
        0,
        0,
        0,
        0,
        HWND_MESSAGE,
        core::ptr::null_mut(),
        hinst,
        core::ptr::null_mut(),
    )
}

/// Windows creates a 'DataObject' clipboard format that contains the clipboard
/// window's HWND or NULL if the Ole clipboard has been flushed.
unsafe fn set_dataobject_format(hwnd: HWND) -> HRESULT {
    let h: HGLOBAL = GlobalAlloc(
        GMEM_DDESHARE | GMEM_MOVEABLE,
        core::mem::size_of::<HWND>() as SIZE_T,
    );

    if h.is_null() {
        return E_OUTOFMEMORY;
    }

    let data: *mut HWND = GlobalLock(h) as *mut HWND;
    *data = hwnd;
    GlobalUnlock(h);

    if SetClipboardData(dataobject_clipboard_format, h).is_null() {
        GlobalFree(h);
        return CLIPBRD_E_CANT_SET;
    }

    S_OK
}

/*---------------------------------------------------------------------*
 *           Win32 OLE clipboard API
 *---------------------------------------------------------------------*/

/// OleSetClipboard [OLE32.@]
///
/// Places a pointer to the specified data object onto the clipboard, making the
/// data object accessible to the OleGetClipboard function.
#[no_mangle]
pub unsafe extern "system" fn OleSetClipboard(data: *mut IDataObject) -> HRESULT {
    let mut clipbrd: *mut ole_clipbrd = core::ptr::null_mut();
    let mut wnd: HWND = core::ptr::null_mut();

    TRACE!(b"(%p)\n\0".as_ptr(), data);

    if (*crate::compobj::compobj::COM_CurrentInfo()).ole_inits == 0 {
        return CO_E_NOTINITIALIZED;
    }

    let mut hr = get_ole_clipbrd(&mut clipbrd);
    if FAILED(hr) {
        return hr;
    }

    hr = get_clipbrd_window(clipbrd, &mut wnd);
    if FAILED(hr) {
        return hr;
    }

    if OpenClipboard(wnd) == 0 {
        return CLIPBRD_E_CANT_OPEN;
    }

    'end: {
        if EmptyClipboard() == 0 {
            hr = CLIPBRD_E_CANT_EMPTY;
            break 'end;
        }

        hr = set_src_dataobject(clipbrd, data);
        if FAILED(hr) {
            break 'end;
        }

        if !data.is_null() {
            hr = expose_marshalled_dataobject(clipbrd, data);
            if FAILED(hr) {
                break 'end;
            }
            hr = set_dataobject_format(wnd);
        }
    }

    /* end: */
    if CloseClipboard() == 0 {
        hr = CLIPBRD_E_CANT_CLOSE;
    }

    if FAILED(hr) {
        expose_marshalled_dataobject(clipbrd, core::ptr::null_mut());
        set_src_dataobject(clipbrd, core::ptr::null_mut());
    }

    hr
}

/// OleGetClipboard [OLE32.@]
///
/// Returns a pointer to our internal IDataObject which represents the conceptual
/// state of the Windows clipboard. If the current clipboard already contains an
/// IDataObject, our internal IDataObject will delegate to this object.
#[no_mangle]
pub unsafe extern "system" fn OleGetClipboard(obj: *mut *mut IDataObject) -> HRESULT {
    let mut clipbrd: *mut ole_clipbrd = core::ptr::null_mut();

    TRACE!(b"(%p)\n\0".as_ptr(), obj);

    if obj.is_null() {
        return E_INVALIDARG;
    }
    *obj = core::ptr::null_mut();

    let hr = get_ole_clipbrd(&mut clipbrd);
    if FAILED(hr) {
        return hr;
    }

    let seq_no = GetClipboardSequenceNumber();
    EnterCriticalSection(core::ptr::addr_of_mut!(latest_snapshot_cs));
    if !(*clipbrd).latest_snapshot.is_null() && (*(*clipbrd).latest_snapshot).seq_no != seq_no {
        (*clipbrd).latest_snapshot = core::ptr::null_mut();
    }

    if (*clipbrd).latest_snapshot.is_null() {
        (*clipbrd).latest_snapshot = snapshot_construct(seq_no);
        if (*clipbrd).latest_snapshot.is_null() {
            LeaveCriticalSection(core::ptr::addr_of_mut!(latest_snapshot_cs));
            return E_OUTOFMEMORY;
        }
    }

    *obj = core::ptr::addr_of_mut!((*(*clipbrd).latest_snapshot).IDataObject_iface);
    IDataObject_AddRef(*obj);
    LeaveCriticalSection(core::ptr::addr_of_mut!(latest_snapshot_cs));

    S_OK
}

/// OleFlushClipboard [OLE32.@]
///
/// Renders the data from the source IDataObject into the windows clipboard.
#[no_mangle]
pub unsafe extern "system" fn OleFlushClipboard() -> HRESULT {
    let mut clipbrd: *mut ole_clipbrd = core::ptr::null_mut();
    let mut wnd: HWND = core::ptr::null_mut();

    TRACE!(b"()\n\0".as_ptr());

    let mut hr = get_ole_clipbrd(&mut clipbrd);
    if FAILED(hr) {
        return hr;
    }

    if (*crate::compobj::compobj::COM_CurrentInfo()).ole_inits == 0 {
        return E_FAIL;
    }

    hr = get_clipbrd_window(clipbrd, &mut wnd);
    if FAILED(hr) {
        return hr;
    }

    /* Already flushed or no source DataObject? Nothing to do. */
    if (*clipbrd).src_data.is_null() {
        return S_OK;
    }

    if OpenClipboard(wnd) == 0 {
        return CLIPBRD_E_CANT_OPEN;
    }

    SendMessageW(wnd, WM_RENDERALLFORMATS, 0, 0);

    hr = set_dataobject_format(core::ptr::null_mut());

    expose_marshalled_dataobject(clipbrd, core::ptr::null_mut());
    set_src_dataobject(clipbrd, core::ptr::null_mut());

    if CloseClipboard() == 0 {
        hr = CLIPBRD_E_CANT_CLOSE;
    }

    hr
}

/// OleIsCurrentClipboard [OLE32.@]
#[no_mangle]
pub unsafe extern "system" fn OleIsCurrentClipboard(data: *mut IDataObject) -> HRESULT {
    let mut clipbrd: *mut ole_clipbrd = core::ptr::null_mut();

    TRACE!(b"()\n\0".as_ptr());

    let hr = get_ole_clipbrd(&mut clipbrd);
    if FAILED(hr) {
        return hr;
    }

    if data.is_null() {
        return S_FALSE;
    }

    if data == (*clipbrd).src_data {
        S_OK
    } else {
        S_FALSE
    }
}
