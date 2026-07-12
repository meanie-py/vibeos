use crate::prelude::*;
use core::ffi::c_void;

extern "C" {

    fn wcscmp(s1: *const u16, s2: *const u16) -> i32;
    fn IsClipboardFormatAvailable(format: u32) -> i32;
}
extern "C" {
    fn DeleteMetaFile(hmf: HMETAFILE) -> i32;
    fn CreateCompatibleDC(hdc: HDC) -> HDC;
    fn SelectObject(hdc: HDC, h: HGDIOBJ) -> HGDIOBJ;
    fn GetObjectW(h: HGDIOBJ, c: i32, pv: LPVOID) -> i32;
    fn CreateCompatibleBitmap(hdc: HDC, cx: i32, cy: i32) -> HBITMAP;
    fn BitBlt(hdc: HDC, x: i32, y: i32, cx: i32, cy: i32, hdcSrc: HDC, x1: i32, y1: i32, rop: DWORD) -> i32;
    fn DeleteDC(hdc: HDC) -> i32;
    fn CopyEnhMetaFileW(hEnh: HENHMETAFILE, lpFileName: *const u16) -> HENHMETAFILE;
    fn ReleaseStgMedium(pmedium: *mut STGMEDIUM) -> ();
    fn memset(s: *mut core::ffi::c_void, c: i32, n: usize) -> *mut core::ffi::c_void;
    fn GetClipboardData(uFormat: u32) -> HGLOBAL;
    fn RegisterClipboardFormatW(lpszFormat: *const u16) -> u32;
    fn CoReleaseMarshalData(pStm: *mut IStream) -> HRESULT;
    fn GetModuleHandleW(lpModuleName: *const u16) -> HINSTANCE;
    fn RegisterClassExW(lpWndClass: *const WNDCLASSEXW) -> u16;
    fn CreateWindowW( lpClassName: *const u16, lpWindowName: *const u16, dwStyle: u32, x: i32, y: i32, nWidth: i32, nHeight: i32, hWndParent: HWND, hMenu: *mut core::ffi::c_void, hInstance: HINSTANCE, lpParam: LPVOID, ) -> HWND;
    fn SetClipboardData(uFormat: u32, hMem: *mut core::ffi::c_void) -> *mut core::ffi::c_void;
    fn CopyMetaFileW(hSrcMetaFile: HMETAFILE, lpFileName: LPCWSTR) -> HMETAFILE;
    fn CreateStreamOnHGlobal(hGlobal: HGLOBAL, fDeleteOnRelease: i32, ppstm: *mut *mut IStream) -> HRESULT;
    fn CreateILockBytesOnHGlobal(hGlobal: HGLOBAL, fDeleteOnRelease: i32, pplkbyt: *mut *mut ILockBytes) -> HRESULT;
    fn StgIsStorageILockBytes(plkbyt: *mut ILockBytes) -> HRESULT;
    fn StgOpenStorageOnILockBytes( plkbyt: *mut ILockBytes, pstgPriority: *mut IStorage, grfMode: u32, snbExclude: *mut *mut u16, reserved: u32, ppstgOpen: *mut *mut IStorage, ) -> HRESULT;
    fn GetHGlobalFromStream(pstm: *mut IStream, phglobal: *mut HGLOBAL) -> HRESULT;
    fn CoUnmarshalInterface(pStm: *mut IStream, riid: *const CLSID, ppv: *mut *mut core::ffi::c_void) -> HRESULT;
    fn EnumClipboardFormats(format: UINT) -> UINT;
    fn GetClipboardFormatNameW(format: UINT, lpszFormatName: LPWSTR, cchMaxCount: i32) -> i32;
    fn GetClipboardSequenceNumber() -> DWORD;
    fn DestroyWindow(hWnd: HWND) -> i32;
    fn UnregisterClassW(lpClassName: *const u16, hInstance: HINSTANCE) -> i32;
    fn OpenClipboard(hWndNewOwner: HWND) -> i32;
    fn EmptyClipboard() -> i32;
    fn CloseClipboard() -> i32;
}

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct ole_priv_data_entry {
    pub fmtetc: FORMATETC,
    pub first_use: DWORD,
    pub unk: [DWORD; 2],
}

pub type Pole_priv_data_entry = *mut ole_priv_data_entry;

#[repr(C)]
pub struct ole_priv_data {
    pub unk1: DWORD,
    pub size: DWORD,
    pub unk2: DWORD,
    pub count: DWORD,
    pub unk3: [DWORD; 2],
    pub entries: [ole_priv_data_entry; 1],
}

pub type Pole_priv_data = *mut ole_priv_data;

#[repr(C)]
pub struct snapshot {
    pub IDataObject_iface: IDataObject,
    pub ref_: LONG,
    pub seq_no: DWORD,
    pub data: *mut IDataObject,
}

pub type Psnapshot = *mut snapshot;

#[repr(C)]
pub struct ole_clipbrd {
    pub latest_snapshot: *mut snapshot,
    pub window: HWND,
    pub src_data: *mut IDataObject,
    pub cached_enum: *mut ole_priv_data,
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

pub static mut theOleClipboard: *mut ole_clipbrd = core::ptr::null_mut();

pub static mut latest_snapshot_cs: CRITICAL_SECTION = CRITICAL_SECTION {
    DebugInfo: unsafe { &latest_snapshot_cs_debug as *const _ as *mut _ },
    LockCount: -1,
    RecursionCount: 0,
    OwningThread: core::ptr::null_mut(),
    LockSemaphore: core::ptr::null_mut(),
    SpinCount: 0,
};

pub static mut latest_snapshot_cs_debug: CRITICAL_SECTION_DEBUG = CRITICAL_SECTION_DEBUG {
    Type: 0,
    CreatorBackTraceIndex: 0,
    CriticalSection: unsafe { &latest_snapshot_cs as *const _ as *mut _ },
    ProcessLocksList: _LIST_ENTRY {
        Flink: unsafe { &latest_snapshot_cs_debug.ProcessLocksList as *const _ as *mut _ },
        Blink: unsafe { &latest_snapshot_cs_debug.ProcessLocksList as *const _ as *mut _ },
    },
    EntryCount: 0,
    ContentionCount: 0,
    Flags: 0,
    CreatorBackTraceIndexHigh: 0,
    SpareWORD: 0,
};

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
pub static mut wine_marshal_clipboard_format: UINT = 0;

#[repr(C)]
pub struct enum_fmtetc {
    pub IEnumFORMATETC_iface: IEnumFORMATETC,
    pub ref_: LONG,
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

unsafe fn td_offs_to_ptr(data: *mut ole_priv_data, off: DWORD_PTR) -> *mut DVTARGETDEVICE {
    if off == 0 {
        return core::ptr::null_mut();
    }
    (data as *mut u8).offset(off as isize) as *mut DVTARGETDEVICE
}

unsafe fn td_get_offs(data: *mut ole_priv_data, idx: DWORD) -> DWORD_PTR {
    if (*data).entries[idx as usize].fmtetc.ptd.is_null() {
        return 0;
    }
    ((*data).entries[idx as usize].fmtetc.ptd as *const u8)
        .offset_from(data as *const u8) as DWORD_PTR
}

unsafe fn impl_from_IDataObject(iface: *mut IDataObject) -> *mut snapshot {
    (iface as *mut u8).offset(-(core::mem::offset_of!(snapshot, IDataObject_iface) as isize)) as *mut snapshot
}

unsafe fn get_ole_clipbrd(clipbrd: *mut *mut ole_clipbrd) -> HRESULT {
    let info = COM_CurrentInfo();
    *clipbrd = core::ptr::null_mut();

    if (*info).ole_inits == 0 {
        return CO_E_NOTINITIALIZED;
    }
    *clipbrd = theOleClipboard;

    S_OK
}

unsafe fn dump_fmtetc(fmt: *const FORMATETC) -> *const i8 {
    if fmt.is_null() {
        return b"(null)\0".as_ptr() as *const i8;
    }
    wine_dbg_sprintf(
        b"cf %04x ptd %p aspect %x lindex %d tymed %x\0".as_ptr() as *const i8,
        (*fmt).cfFormat,
        (*fmt).ptd,
        (*fmt).dwAspect,
        (*fmt).lindex,
        (*fmt).tymed,
    )
}

unsafe fn impl_from_IEnumFORMATETC(iface: *mut IEnumFORMATETC) -> *mut enum_fmtetc {
    (iface as *mut u8).offset(-{
        let dummy = core::ptr::null::<enum_fmtetc>();
        core::ptr::addr_of!((*dummy).IEnumFORMATETC_iface) as isize
    }) as *mut enum_fmtetc
}

unsafe fn OLEClipbrd_IEnumFORMATETC_QueryInterface(
    iface: *mut IEnumFORMATETC,
    riid: *const GUID,
    ppvObj: *mut LPVOID
) -> HRESULT {

    let This = impl_from_IEnumFORMATETC(iface);

    TRACE!(b"(%p)->(IID: %s, %p)\n\0".as_ptr(), This, debugstr_guid(riid), ppvObj);

    *ppvObj = core::ptr::null_mut();

    if IsEqualIID(riid, &IID_IUnknown) != 0 ||
       IsEqualIID(riid, &IID_IEnumFORMATETC) != 0
    {
        *ppvObj = iface as LPVOID;
    }

    if !(*ppvObj).is_null()
    {
        IEnumFORMATETC_AddRef(*ppvObj as *mut IEnumFORMATETC);
        TRACE!(b"-- Interface: (%p)->(%p)\n\0".as_ptr(), ppvObj, *ppvObj);
        return S_OK;
    }

    TRACE!(b"-- Interface: E_NOINTERFACE\n\0".as_ptr());
    E_NOINTERFACE
}

unsafe fn OLEClipbrd_IEnumFORMATETC_AddRef(iface: *mut IEnumFORMATETC) -> u32 {
    let This = impl_from_IEnumFORMATETC(iface);
    TRACE!(b"(%p)->(count=%u)\n\0".as_ptr() as *const i8, This, (*This).ref_);

    InterlockedIncrement(&mut (*This).ref_) as u32
}

unsafe fn OLEClipbrd_IEnumFORMATETC_Release(iface: *mut IEnumFORMATETC) -> ULONG {
    let This = impl_from_IEnumFORMATETC(iface);
    let ref_count: ULONG;

    TRACE!(b"(%p)->(count=%u)\n\0".as_ptr() as *const i8, This, (*This).ref_);

    ref_count = InterlockedDecrement(&mut (*This).ref_) as ULONG;
    if ref_count == 0 {
        TRACE!(b"() - destroying IEnumFORMATETC(%p)\n\0".as_ptr() as *const i8, This);
        HeapFree(GetProcessHeap(), 0, (*This).data as LPVOID);
        HeapFree(GetProcessHeap(), 0, This as LPVOID);
    }
    ref_count
}

unsafe fn OLEClipbrd_IEnumFORMATETC_Next(
    iface: *mut IEnumFORMATETC,
    celt: ULONG,
    rgelt: *mut FORMATETC,
    pceltFethed: *mut ULONG,
) -> HRESULT {
    let This = impl_from_IEnumFORMATETC(iface);
    let mut cfetch: UINT;
    let mut hres = S_FALSE;

    TRACE!(b"(%p)->(pos=%u)\n\0".as_ptr() as *const i8, This, (*This).pos);

    if (*This).pos < (*(*This).data).count {
        cfetch = (*(*This).data).count - (*This).pos;
        if cfetch >= celt {
            cfetch = celt;
            hres = S_OK;
        }

        for i in 0..cfetch {
            hres = copy_formatetc(
                rgelt.add(i as usize),
                core::ptr::addr_of_mut!((*(*(*This).data).entries.as_mut_ptr().add((*This).pos as usize)).fmtetc),
            );
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
unsafe fn OLEClipbrd_IEnumFORMATETC_Skip(iface: *mut IEnumFORMATETC, celt: ULONG) -> HRESULT {
    let This = impl_from_IEnumFORMATETC(iface);
    TRACE!(b"(%p)->(num=%u)\n\0".as_ptr() as PCHAR, This, celt);

    (*This).pos += celt;
    if (*This).pos > (*(*This).data).count {
        (*This).pos = (*(*This).data).count;
        return S_FALSE;
    }
    S_OK
}

unsafe fn OLEClipbrd_IEnumFORMATETC_Reset(iface: *mut IEnumFORMATETC) -> HRESULT {
    let This = impl_from_IEnumFORMATETC(iface);
    TRACE!(b"(%p)->()\n\0".as_ptr() as *const i8, This);

    (*This).pos = 0;
    S_OK
}
unsafe fn OLEClipbrd_IEnumFORMATETC_Clone(
    iface: *mut IEnumFORMATETC,
    obj: *mut *mut IEnumFORMATETC,
) -> HRESULT {
    let This = impl_from_IEnumFORMATETC(iface);
    let mut new_data: *mut ole_priv_data;
    let mut i: DWORD;

    TRACE!(b"(%p)->(%p)\n\0".as_ptr() as *const i8, This, obj);

    if obj.is_null() {
        return E_INVALIDARG;
    }
    *obj = core::ptr::null_mut();

    new_data = HeapAlloc(GetProcessHeap(), 0, (*(*This).data).size as usize) as *mut ole_priv_data;
    if new_data.is_null() {
        return E_OUTOFMEMORY;
    }
    core::ptr::copy_nonoverlapping(
        (*This).data as *const u8,
        new_data as *mut u8,
        (*(*This).data).size as usize,
    );

    i = 0;
    while i < (*(*This).data).count {
        (*new_data).entries[i as usize].fmtetc.ptd =
            td_offs_to_ptr(new_data, td_get_offs((*This).data, i));
        i += 1;
    }

    return enum_fmtetc_construct(new_data, (*This).pos, obj);
}

unsafe fn enum_fmtetc_construct(data: *mut ole_priv_data, pos: UINT, obj: *mut *mut IEnumFORMATETC) -> HRESULT {
    let ef: *mut enum_fmtetc;

    *obj = core::ptr::null_mut();
    ef = HeapAlloc(GetProcessHeap(), 0, core::mem::size_of::<enum_fmtetc>()) as *mut enum_fmtetc;
    if ef.is_null() {
        return E_OUTOFMEMORY;
    }

    (*ef).ref_ = 1;
    (*ef).IEnumFORMATETC_iface.lpVtbl = &efvt;
    (*ef).data = data;
    (*ef).pos = pos;

    TRACE!(b"(%p)->()\n\0".as_ptr() as PCHAR, ef);
    *obj = core::ptr::addr_of_mut!((*ef).IEnumFORMATETC_iface);
    S_OK
}

unsafe fn dup_global_mem(src: HGLOBAL, flags: DWORD, dst: *mut HGLOBAL) -> HRESULT {
    let mut src_ptr: LPVOID;
    let mut dst_ptr: LPVOID;
    let size: DWORD;

    *dst = core::ptr::null_mut();
    if src.is_null() {
        return S_FALSE;
    }

    size = GlobalSize(src);

    *dst = GlobalAlloc(flags, size);
    if (*dst).is_null() {
        return E_OUTOFMEMORY;
    }

    src_ptr = GlobalLock(src);
    dst_ptr = GlobalLock(*dst);

    memcpy(dst_ptr, src_ptr, size);

    GlobalUnlock(*dst);
    GlobalUnlock(src);

    S_OK
}

unsafe fn dup_metafilepict(src: HGLOBAL, pdest: *mut HGLOBAL) -> HRESULT {
    let mut dest: HGLOBAL;
    let dest_ptr: *mut METAFILEPICT;
    let hr: HRESULT;

    *pdest = core::ptr::null_mut();

    hr = dup_global_mem(src, GMEM_DDESHARE | GMEM_MOVEABLE, &mut dest);
    if FAILED(hr) != 0 {
        return hr;
    }

    dest_ptr = GlobalLock(dest) as *mut METAFILEPICT;
    if dest_ptr.is_null() {
        return E_FAIL;
    }

    (*dest_ptr).hMF = CopyMetaFileW((*dest_ptr).hMF, core::ptr::null_mut());
    if !(*dest_ptr).hMF.is_null() {
        GlobalUnlock(dest);
        *pdest = dest;
        return S_OK;
    } else {
        GlobalUnlock(dest);
        GlobalFree(dest);
        return E_FAIL;
    }
}

unsafe fn free_metafilepict(src: HGLOBAL) {

    let src_ptr = GlobalLock(src) as *mut METAFILEPICT;
    if !src_ptr.is_null() {
        DeleteMetaFile((*src_ptr).hMF);
        GlobalUnlock(src);
    }
    GlobalFree(src);
}

unsafe fn dup_bitmap(src: HBITMAP, pdest: *mut HBITMAP) -> HRESULT {

    const SRCCOPY: u32 = 0x00CC0020;
    const S_OK: HRESULT = 0;
    const E_FAIL: HRESULT = 0x80004005u32 as i32;

    #[repr(C)]
    struct BITMAP {
        bmType: i32,
        bmWidth: i32,
        bmHeight: i32,
        bmWidthBytes: i32,
        bmPlanes: u16,
        bmBitsPixel: u16,
        bmBits: LPVOID,
    }

    let src_dc: HDC;
    let orig_src_bitmap: HGDIOBJ;
    let mut bm: BITMAP = core::mem::zeroed();
    let dest: HBITMAP;

    src_dc = CreateCompatibleDC(core::ptr::null_mut());
    orig_src_bitmap = SelectObject(src_dc, src as HGDIOBJ);
    GetObjectW(src as HGDIOBJ, core::mem::size_of::<BITMAP>() as i32, &mut bm as *mut BITMAP as LPVOID);
    dest = CreateCompatibleBitmap(src_dc, bm.bmWidth, bm.bmHeight);
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
    if !dest.is_null() { S_OK } else { E_FAIL }
}

unsafe fn render_embed_source_hack(data: *mut IDataObject, fmt: LPFORMATETC) -> HRESULT {
    let mut std: STGMEDIUM = core::mem::zeroed();
    let mut hStorage: HGLOBAL = 0 as HGLOBAL;
    let mut hr: HRESULT = S_OK;
    let mut ptrILockBytes: *mut ILockBytes = core::ptr::null_mut();

    core::ptr::write_bytes(&mut std, 0, 1);
    std.tymed = TYMED_ISTORAGE;
    (*fmt).tymed = TYMED_ISTORAGE;

    hStorage = GlobalAlloc(GMEM_SHARE | GMEM_MOVEABLE, 0);
    if hStorage.is_null() {
        return E_OUTOFMEMORY;
    }
    hr = CreateILockBytesOnHGlobal(hStorage, 0, &mut ptrILockBytes);
    if FAILED(hr) != 0 {
        GlobalFree(hStorage);
        return hr;
    }

    hr = StgCreateDocfileOnILockBytes(ptrILockBytes, STGM_SHARE_EXCLUSIVE | STGM_READWRITE, 0, &mut (*std.__bindgen_anon_1.pstg.as_mut()));
    ILockBytes_Release(ptrILockBytes);

    hr = IDataObject_GetDataHere((*theOleClipboard).src_data, fmt, &mut std);
    if FAILED(hr) != 0 {
        WARN!(b"() : IDataObject_GetDataHere failed to render clipboard data! (%x)\n\0".as_ptr() as *const i8, hr);
        GlobalFree(hStorage);
        return hr;
    }

    if 1 != 0 {
        let mut fmt2: FORMATETC = core::mem::zeroed();
        let mut std2: STGMEDIUM = core::mem::zeroed();
        let mut mfp: *mut METAFILEPICT = core::ptr::null_mut();

        fmt2.cfFormat = CF_METAFILEPICT;
        fmt2.ptd = core::ptr::null_mut();
        fmt2.dwAspect = DVASPECT_CONTENT;
        fmt2.lindex = -1i32;
        fmt2.tymed = TYMED_MFPICT;

        core::ptr::write_bytes(&mut std2, 0, 1);
        std2.tymed = TYMED_MFPICT;

        hr = IDataObject_GetData((*theOleClipboard).src_data, &mut fmt2, &mut std2);
        if SUCCEEDED(hr) != 0 {
            mfp = GlobalLock((*std2.__bindgen_anon_1.hGlobal.as_ref())) as *mut METAFILEPICT;
        }

        if !mfp.is_null() {
            let mut name: [u16; 12] = [2, 'O' as u16, 'l' as u16, 'e' as u16, 'P' as u16, 'r' as u16, 'e' as u16, 's' as u16, '0' as u16, '0' as u16, '0' as u16, 0];
            let mut pStream: *mut IStream = core::ptr::null_mut();
            let mut mfBits: *mut core::ffi::c_void;
            let mut pdh: PresentationDataHeader = core::mem::zeroed();
            let mut nSize: i32;
            let mut clsID: CLSID = core::mem::zeroed();
            let mut strProgID: *mut u16 = core::ptr::null_mut();
            let mut strOleTypeName: [i8; 51] = [0; 51];
            let OlePresStreamHeader: [u8; 28] = [
                0xFF, 0xFF, 0xFF, 0xFF, 0x03, 0x00, 0x00, 0x00,
                0x04, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
                0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00
            ];

            nSize = GetMetaFileBitsEx((*mfp).hMF, 0, core::ptr::null_mut());

            core::ptr::write_bytes(&mut pdh, 0, 1);
            core::ptr::copy_nonoverlapping(OlePresStreamHeader.as_ptr(), &mut pdh as *mut _ as *mut u8, core::mem::size_of_val(&OlePresStreamHeader));

            pdh.dwObjectExtentX = (*mfp).xExt as DWORD;
            pdh.dwObjectExtentY = (*mfp).yExt as DWORD;
            pdh.dwSize = nSize as DWORD;

            hr = IStorage_CreateStream((*std.__bindgen_anon_1.pstg.as_ref()), name.as_ptr(), STGM_CREATE | STGM_SHARE_EXCLUSIVE | STGM_READWRITE, 0, 0, &mut pStream);

            hr = IStream_Write(pStream, &pdh as *const _ as *const core::ffi::c_void, core::mem::size_of::<PresentationDataHeader>() as u32, core::ptr::null_mut());

            mfBits = HeapAlloc(GetProcessHeap(), 0, nSize as usize);
            nSize = GetMetaFileBitsEx((*mfp).hMF, nSize as u32, mfBits);

            hr = IStream_Write(pStream, mfBits, nSize as u32, core::ptr::null_mut());

            IStream_Release(pStream);

            HeapFree(GetProcessHeap(), 0, mfBits);

            GlobalUnlock((*std2.__bindgen_anon_1.hGlobal.as_ref()));
            ReleaseStgMedium(&mut std2);

            ReadClassStg((*std.__bindgen_anon_1.pstg.as_ref()), &mut clsID);
            ProgIDFromCLSID(&clsID, &mut strProgID);

            WideCharToMultiByte(CP_ACP, 0, strProgID, -1, strOleTypeName.as_mut_ptr(), core::mem::size_of_val(&strOleTypeName) as i32, core::ptr::null_mut(), core::ptr::null_mut());
            STORAGE_CreateOleStream((*std.__bindgen_anon_1.pstg.as_ref()), 0);
            OLECONVERT_CreateCompObjStream((*std.__bindgen_anon_1.pstg.as_ref()), strOleTypeName.as_ptr());
            CoTaskMemFree(strProgID as *mut core::ffi::c_void);
        }
    }

    if SetClipboardData((*fmt).cfFormat, hStorage as HANDLE) == 0 as HANDLE {
        WARN!(b"() : Failed to set rendered clipboard data into clipboard!\n\0".as_ptr() as *const i8);
        GlobalFree(hStorage);
        hr = CLIPBRD_E_CANT_SET;
    }

    ReleaseStgMedium(&mut std);
    return hr;
}

unsafe fn find_format_in_list(
    entries: *mut ole_priv_data_entry,
    num: DWORD,
    cf: UINT,
) -> *mut ole_priv_data_entry {
    for i in 0..num {
        if (*entries.add(i as usize)).fmtetc.cfFormat == cf {
            return entries.add(i as usize);
        }
    }
    core::ptr::null_mut()
}

unsafe fn get_data_from_storage(data: *mut IDataObject, fmt: *mut FORMATETC, mem: *mut HGLOBAL) -> HRESULT {
    let mut h: HGLOBAL;
    let mut stg: *mut IStorage = core::ptr::null_mut();
    let mut hr: HRESULT;
    let mut stg_fmt: FORMATETC;
    let mut med: STGMEDIUM;
    let mut lbs: *mut ILockBytes = core::ptr::null_mut();

    *mem = core::ptr::null_mut();

    h = GlobalAlloc(GMEM_DDESHARE | GMEM_MOVEABLE, 0);
    if h.is_null() {
        return E_OUTOFMEMORY;
    }

    hr = CreateILockBytesOnHGlobal(h, 0u8, &mut lbs);
    if SUCCEEDED(hr) != 0 {
        hr = StgCreateDocfileOnILockBytes(lbs, STGM_CREATE | STGM_SHARE_EXCLUSIVE | STGM_READWRITE, 0, &mut stg);
        ILockBytes_Release(lbs);
    }
    if FAILED(hr) != 0 {
        GlobalFree(h);
        return hr;
    }

    stg_fmt = *fmt;
    med.tymed = TYMED_ISTORAGE;
    stg_fmt.tymed = TYMED_ISTORAGE;
    (*med.u.pstg()) = stg;
    med.pUnkForRelease = core::ptr::null_mut();

    hr = IDataObject_GetDataHere(data, &mut stg_fmt, &mut med);
    if FAILED(hr) != 0 {
        memset(&mut med as *mut _ as *mut core::ffi::c_void, 0, core::mem::size_of::<STGMEDIUM>());
        hr = IDataObject_GetData(data, &mut stg_fmt, &mut med);
        if FAILED(hr) != 0 {
            IStorage_Release(stg);
            GlobalFree(h);
            return hr;
        }

        hr = IStorage_CopyTo(*(*med.u.pstg()), 0, core::ptr::null_mut(), core::ptr::null_mut(), stg);
        ReleaseStgMedium(&mut med);
        if FAILED(hr) != 0 {
            IStorage_Release(stg);
            GlobalFree(h);
            return hr;
        }
    }
    *mem = h;

    IStorage_Release(stg);
    if FAILED(hr) != 0 {
        GlobalFree(h);
    }
    return hr;
}

unsafe fn get_data_from_stream(data: *mut IDataObject, fmt: *mut FORMATETC, mem: *mut HGLOBAL) -> HRESULT {
    let mut h: HGLOBAL;
    let mut stm: *mut IStream = core::ptr::null_mut();
    let mut hr: HRESULT;
    let mut stm_fmt: FORMATETC;
    let mut med: STGMEDIUM;

    *mem = core::ptr::null_mut();

    h = GlobalAlloc(GMEM_DDESHARE | GMEM_MOVEABLE, 0);
    if h.is_null() {
        return E_OUTOFMEMORY;
    }

    hr = CreateStreamOnHGlobal(h, 0, &mut stm);
    if FAILED(hr) != 0 {
        GlobalFree(h);
        return hr;
    }

    stm_fmt = *fmt;
    med.tymed = TYMED_ISTREAM;
    stm_fmt.tymed = TYMED_ISTREAM;
    (*med.u.pstm()) = stm;
    med.pUnkForRelease = core::ptr::null_mut();

    hr = IDataObject_GetDataHere(data, &mut stm_fmt, &mut med);
    if FAILED(hr) != 0 {
        let mut offs: LARGE_INTEGER;
        let mut pos: ULARGE_INTEGER;

        memset(&mut med as *mut _ as *mut core::ffi::c_void, 0, core::mem::size_of::<STGMEDIUM>());
        hr = IDataObject_GetData(data, &mut stm_fmt, &mut med);
        if FAILED(hr) != 0 {
            if !stm.is_null() {
                IStream_Release(stm);
            }
            GlobalFree(h);
            return hr;
        }

        offs.QuadPart = 0;
        IStream_Seek(*med.u.pstm(), offs, STREAM_SEEK_CUR, &mut pos);
        IStream_Seek(*med.u.pstm(), offs, STREAM_SEEK_SET, core::ptr::null_mut());
        hr = IStream_CopyTo(*med.u.pstm(), stm, pos, core::ptr::null_mut(), core::ptr::null_mut());
        ReleaseStgMedium(&mut med);
        if FAILED(hr) != 0 {
            if !stm.is_null() {
                IStream_Release(stm);
            }
            GlobalFree(h);
            return hr;
        }
    }
    *mem = h;
    IStream_Release(stm);
    return S_OK;
}

unsafe fn get_data_from_global(data: *mut IDataObject, fmt: *mut FORMATETC, mem: *mut HGLOBAL) -> HRESULT {
    let mut h: HGLOBAL = core::ptr::null_mut();
    let mut hr: HRESULT;
    let mut mem_fmt: FORMATETC;
    let mut med: STGMEDIUM;

    *mem = core::ptr::null_mut();

    mem_fmt = *fmt;
    mem_fmt.tymed = TYMED_HGLOBAL;
    core::ptr::write_bytes(&mut med as *mut STGMEDIUM, 0, 1);

    hr = IDataObject_GetData(data, &mut mem_fmt, &mut med);
    if FAILED(hr) != 0 {
        return hr;
    }

    hr = dup_global_mem(med.u.hGlobal, GMEM_DDESHARE | GMEM_MOVEABLE, &mut h);

    if SUCCEEDED(hr) != 0 {
        *mem = h;
    }

    ReleaseStgMedium(&mut med);

    return hr;
}

unsafe fn get_data_from_enhmetafile(data: *mut IDataObject, fmt: *mut FORMATETC, mem: *mut HGLOBAL) -> HRESULT {

    let mut copy: HENHMETAFILE;
    let mut hr: HRESULT;
    let mut mem_fmt: FORMATETC;
    let mut med: STGMEDIUM;

    *mem = core::ptr::null_mut();

    mem_fmt = *fmt;
    mem_fmt.tymed = TYMED_ENHMF;
    memset(core::ptr::addr_of_mut!(med) as *mut core::ffi::c_void, 0, core::mem::size_of::<STGMEDIUM>());

    hr = IDataObject_GetData(data, core::ptr::addr_of_mut!(mem_fmt), core::ptr::addr_of_mut!(med));
    if FAILED(hr) != 0 {
        return hr;
    }

    copy = CopyEnhMetaFileW((*med.u.hEnhMetaFile()), core::ptr::null());
    if !copy.is_null() {
        *mem = copy as HGLOBAL;
    } else {
        hr = E_FAIL;
    }

    ReleaseStgMedium(core::ptr::addr_of_mut!(med));

    return hr;
}

unsafe fn get_data_from_metafilepict(data: *mut IDataObject, fmt: *mut FORMATETC, mem: *mut HGLOBAL) -> HRESULT {
    let mut copy: HGLOBAL;
    let mut hr: HRESULT;
    let mut mem_fmt: FORMATETC;
    let mut med: STGMEDIUM;

    *mem = core::ptr::null_mut();

    mem_fmt = *fmt;
    mem_fmt.tymed = TYMED_MFPICT;
    memset(core::ptr::addr_of_mut!(med) as *mut core::ffi::c_void, 0, core::mem::size_of::<STGMEDIUM>());

    hr = IDataObject_GetData(data, core::ptr::addr_of!(mem_fmt), core::ptr::addr_of_mut!(med));
    if FAILED(hr) != 0 { return hr; }

    hr = dup_metafilepict((*med.u).hMetaFilePict, core::ptr::addr_of_mut!(copy));

    if SUCCEEDED(hr) != 0 { *mem = copy; }

    ReleaseStgMedium(core::ptr::addr_of_mut!(med));

    return hr;
}

unsafe fn get_data_from_bitmap(data: *mut IDataObject, fmt: *mut FORMATETC, hbm: *mut HBITMAP) -> HRESULT {
    let mut copy: HBITMAP = core::ptr::null_mut();
    let mut hr: HRESULT;
    let mut mem_fmt: FORMATETC = core::mem::zeroed();
    let mut med: STGMEDIUM = core::mem::zeroed();

    *hbm = core::ptr::null_mut();

    mem_fmt = *fmt;
    mem_fmt.tymed = TYMED_GDI;
    core::ptr::write_bytes(&mut med as *mut STGMEDIUM, 0, 1);

    hr = IDataObject_GetData(data, &mut mem_fmt, &mut med);
    if FAILED(hr) != 0 { return hr; }

    hr = dup_bitmap((*med.u).hBitmap, &mut copy);

    if SUCCEEDED(hr) != 0 { *hbm = copy; }

    ReleaseStgMedium(&mut med);

    return hr;
}

unsafe fn render_format(data: *mut IDataObject, fmt: LPFORMATETC) -> HRESULT {
    let mut clip_data: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut hr: HRESULT;

    if (*fmt).cfFormat == embed_source_clipboard_format {
        return render_embed_source_hack(data, fmt);
    }

    if (*fmt).tymed & TYMED_ISTORAGE != 0 {
        hr = get_data_from_storage(data, fmt, &mut clip_data as *mut *mut core::ffi::c_void as *mut HGLOBAL);
    } else if (*fmt).tymed & TYMED_ISTREAM != 0 {
        hr = get_data_from_stream(data, fmt, &mut clip_data as *mut *mut core::ffi::c_void as *mut HGLOBAL);
    } else if (*fmt).tymed & TYMED_HGLOBAL != 0 {
        hr = get_data_from_global(data, fmt, &mut clip_data as *mut *mut core::ffi::c_void as *mut HGLOBAL);
    } else if (*fmt).tymed & TYMED_ENHMF != 0 {
        hr = get_data_from_enhmetafile(data, fmt, &mut clip_data as *mut *mut core::ffi::c_void as *mut HGLOBAL);
    } else if (*fmt).tymed & TYMED_MFPICT != 0 {
        hr = get_data_from_metafilepict(data, fmt, &mut clip_data as *mut *mut core::ffi::c_void as *mut HGLOBAL);
    } else if (*fmt).tymed & TYMED_GDI != 0 {
        hr = get_data_from_bitmap(data, fmt, &mut clip_data as *mut *mut core::ffi::c_void as *mut HBITMAP);
    } else {
        FIXME!(b"Unhandled tymed %x\n\0".as_ptr() as *const i8, (*fmt).tymed);
        hr = DV_E_FORMATETC;
    }

    if SUCCEEDED(hr) != 0 {
        if SetClipboardData((*fmt).cfFormat, clip_data) == 0 {
            WARN!(b"() : Failed to set rendered clipboard data into clipboard!\n\0".as_ptr() as *const i8);
            if (*fmt).tymed & TYMED_MFPICT != 0 {
                free_metafilepict(clip_data as HGLOBAL);
            } else if (*fmt).tymed & TYMED_GDI != 0 {
                DeleteObject(clip_data as HGDIOBJ);
            } else {
                GlobalFree(clip_data as HGLOBAL);
            }
            hr = CLIPBRD_E_CANT_SET;
        }
    }

    hr
}

unsafe fn snapshot_QueryInterface(
    iface: *mut IDataObject,
    riid: *const CLSID,
    ppvObject: *mut *mut core::ffi::c_void
) -> HRESULT {
    let This = impl_from_IDataObject(iface);
    TRACE!(b"(%p)->(IID:%s, %p)\n\0".as_ptr() as *const i8, This, debugstr_guid(riid), ppvObject);

    if This.is_null() || ppvObject.is_null() {
        return E_INVALIDARG;
    }

    *ppvObject = core::ptr::null_mut();

    if IsEqualIID(&IID_IUnknown, riid) != 0 ||
       IsEqualIID(&IID_IDataObject, riid) != 0 {
        *ppvObject = iface as *mut core::ffi::c_void;
    } else {
        WARN!(b"() : asking for unsupported interface %s\n\0".as_ptr() as *const i8, debugstr_guid(riid));
        return E_NOINTERFACE;
    }

    IUnknown_AddRef(*ppvObject as *mut IUnknown);

    S_OK
}

unsafe fn snapshot_AddRef(iface: *mut IDataObject) -> ULONG {
    let This = impl_from_IDataObject(iface);

    TRACE!(b"(%p)->(count=%u)\n\0".as_ptr() as PCHAR, This, (*This).ref_);

    InterlockedIncrement(&mut (*This).ref_) as ULONG
}

unsafe fn snapshot_Release(iface: *mut IDataObject) -> u32 {
    let This = impl_from_IDataObject(iface);
    let mut ref_count: u32;

    TRACE!(b"(%p)->(count=%u)\n\0".as_ptr() as *const i8, This, (*This).ref_count);

    ref_count = InterlockedDecrement(&mut (*This).ref_count) as u32;

    if ref_count == 0 {
        EnterCriticalSection(&mut latest_snapshot_cs);
        if (*This).ref_count != 0 {
            LeaveCriticalSection(&mut latest_snapshot_cs);
            return ref_count;
        }
        if (*theOleClipboard).latest_snapshot == This {
            (*theOleClipboard).latest_snapshot = core::ptr::null_mut();
        }
        LeaveCriticalSection(&mut latest_snapshot_cs);

        if !(*This).data.is_null() {
            IDataObject_Release((*This).data);
        }
        HeapFree(GetProcessHeap(), 0, This as LPVOID);
    }

    ref_count
}

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

unsafe fn get_current_dataobject(data: *mut *mut IDataObject) -> HRESULT {

    const S_FALSE: HRESULT = 1;
    const STREAM_SEEK_SET: u32 = 0;

    let mut hr: HRESULT = S_FALSE;
    let wnd: HWND = get_current_ole_clip_window();
    let h: HGLOBAL;
    let ptr: LPVOID;
    let mut stm: *mut IStream = core::ptr::null_mut();
    let mut pos: LARGE_INTEGER = core::mem::zeroed();

    *data = core::ptr::null_mut();
    if wnd.is_null() {
        return S_FALSE;
    }

    h = GetClipboardData(wine_marshal_clipboard_format);
    if h.is_null() {
        return S_FALSE;
    }
    if GlobalSize(h) <= 1 {
        return S_FALSE;
    }
    ptr = GlobalLock(h);
    if ptr.is_null() {
        return S_FALSE;
    }

    hr = CreateStreamOnHGlobal(core::ptr::null_mut(), 1, &mut stm);
    if FAILED(hr) != 0 {
        GlobalUnlock(h);
        return hr;
    }

    hr = IStream_Write(stm, ptr, GlobalSize(h) as u32, core::ptr::null_mut());
    if SUCCEEDED(hr) != 0 {
        pos.QuadPart = 0;
        IStream_Seek(stm, pos, STREAM_SEEK_SET, core::ptr::null_mut());
        hr = CoUnmarshalInterface(stm, &IID_IDataObject as *const CLSID, data as *mut *mut core::ffi::c_void);
    }
    IStream_Release(stm);

    GlobalUnlock(h);
    hr
}

unsafe fn get_tymed_from_nonole_cf(cf: UINT) -> DWORD {
    if cf >= 0xc000 {
        return TYMED_ISTREAM | TYMED_HGLOBAL;
    }

    match cf {
        CF_TEXT | CF_OEMTEXT | CF_UNICODETEXT => TYMED_ISTREAM | TYMED_HGLOBAL,
        CF_ENHMETAFILE => TYMED_ENHMF,
        CF_METAFILEPICT => TYMED_MFPICT,
        CF_BITMAP => TYMED_GDI,
        _ => {
            FIXME!(b"returning TYMED_NULL for cf %04x\n\0".as_ptr() as PCHAR, cf);
            TYMED_NULL
        }
    }
}

unsafe fn get_priv_data(data: *mut *mut ole_priv_data) -> HRESULT {

    let handle: HGLOBAL;
    let hr: HRESULT = S_OK;
    let mut ret: *mut ole_priv_data = core::ptr::null_mut();

    *data = core::ptr::null_mut();

    handle = GetClipboardData(ole_private_data_clipboard_format);
    if !handle.is_null() {
        let src = GlobalLock(handle) as *mut ole_priv_data;
        if !src.is_null() {
            ret = HeapAlloc(GetProcessHeap(), 0, (*src).size as usize) as *mut ole_priv_data;
            if ret.is_null() {
                GlobalUnlock(handle);
                return E_OUTOFMEMORY;
            }
            core::ptr::copy_nonoverlapping(src as *const u8, ret as *mut u8, (*src).size as usize);
            GlobalUnlock(handle);

            for i in 0..(*ret).count {
                let entry_ptr = (*ret).entries.as_mut_ptr().add(i as usize);
                (*entry_ptr).fmtetc.ptd = td_offs_to_ptr(ret, (*entry_ptr).fmtetc.ptd as DWORD_PTR);
            }
        }
    }

    if ret.is_null() {
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
            if GetClipboardFormatNameW(cf, buf.as_mut_ptr(), 256) != 0 {
                TRACE!(b"cf %04x %s\n\0".as_ptr(), cf, debugstr_w(buf.as_ptr()));
            } else {
                TRACE!(b"cf %04x\n\0".as_ptr(), cf);
            }
            count += 1;
        }
        TRACE!(b"count %d\n\0".as_ptr(), count);

        size = core::mem::offset_of!(ole_priv_data, entries) as DWORD;
        size += count * core::mem::size_of::<ole_priv_data_entry>() as DWORD;

        ret = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, size as usize) as *mut ole_priv_data;
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
            let entry_ptr = (*ret).entries.as_mut_ptr().add(idx as usize);
            (*entry_ptr).fmtetc.cfFormat = cf as u16;
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

unsafe fn get_stgmed_for_global(h: HGLOBAL, med: *mut STGMEDIUM) -> HRESULT {
    let hr: HRESULT;

    (*med).pUnkForRelease = core::ptr::null_mut();
    (*med).tymed = TYMED_NULL;

    hr = dup_global_mem(h, GMEM_MOVEABLE, core::ptr::addr_of_mut!((*med).u.hGlobal));

    if SUCCEEDED(hr) != 0 {
        (*med).tymed = TYMED_HGLOBAL;
    }

    hr
}

unsafe fn get_stgmed_for_stream(h: HGLOBAL, med: *mut STGMEDIUM) -> HRESULT {
    let mut hr: HRESULT;
    let mut dst: HGLOBAL;

    (*med).pUnkForRelease = core::ptr::null_mut();
    (*med).tymed = TYMED_NULL;

    hr = dup_global_mem(h, GMEM_MOVEABLE, &mut dst);
    if FAILED(hr) != 0 {
        return hr;
    }

    hr = CreateStreamOnHGlobal(dst, 1, &mut (*med).u.pstm);
    if FAILED(hr) != 0 {
        GlobalFree(dst);
        return hr;
    }

    (*med).tymed = TYMED_ISTREAM;
    hr
}

unsafe fn get_stgmed_for_storage(h: HGLOBAL, med: *mut STGMEDIUM) -> HRESULT {
    let mut dst: HGLOBAL;
    let mut lbs: *mut ILockBytes = core::ptr::null_mut();

    (*med).pUnkForRelease = core::ptr::null_mut();
    (*med).tymed = TYMED_NULL;

    let mut hr = dup_global_mem(h, GMEM_MOVEABLE, &mut dst);
    if FAILED(hr) != 0 {
        return hr;
    }

    hr = CreateILockBytesOnHGlobal(dst, 1, &mut lbs);
    if FAILED(hr) != 0 {
        GlobalFree(dst);
        return hr;
    }

    hr = StgIsStorageILockBytes(lbs);
    if hr != S_OK {
        ILockBytes_Release(lbs);
        GlobalFree(dst);
        return if SUCCEEDED(hr) != 0 { E_FAIL } else { hr };
    }

    hr = StgOpenStorageOnILockBytes(
        lbs,
        core::ptr::null_mut(),
        STGM_SHARE_EXCLUSIVE | STGM_READWRITE,
        core::ptr::null_mut(),
        0,
        &mut (*med).__bindgen_anon_1.pstg,
    );
    ILockBytes_Release(lbs);
    if FAILED(hr) != 0 {
        GlobalFree(dst);
        return hr;
    }

    (*med).tymed = TYMED_ISTORAGE;
    hr
}

unsafe fn get_stgmed_for_emf(hemf: HENHMETAFILE, med: *mut STGMEDIUM) -> HRESULT {

    (*med).pUnkForRelease = core::ptr::null_mut();
    (*med).tymed = 0;

    (*med).u.hEnhMetaFile = CopyEnhMetaFileW(hemf, core::ptr::null());
    if (*med).u.hEnhMetaFile.is_null() {
        return 0x8007000E;
    }
    (*med).tymed = 8;
    return 0;
}

unsafe fn get_stgmed_for_bitmap(hbmp: HBITMAP, med: *mut STGMEDIUM) -> HRESULT {
    let hr: HRESULT;

    (*med).pUnkForRelease = core::ptr::null_mut();
    (*med).tymed = TYMED_NULL;

    hr = dup_bitmap(hbmp, core::ptr::addr_of_mut!((*med).u.hBitmap));

    if FAILED(hr) != 0 {
        return hr;
    }

    (*med).tymed = TYMED_GDI;
    return S_OK;
}

unsafe fn string_off_equal(t1: *const DVTARGETDEVICE, off1: u16, t2: *const DVTARGETDEVICE, off2: u16) -> u8 {
    let str1: *const u16;
    let str2: *const u16;

    if off1 == 0 && off2 == 0 {
        return 1u8;
    }
    if off1 == 0 || off2 == 0 {
        return 0u8;
    }

    str1 = (t1 as *const u8).offset(off1 as isize) as *const u16;
    str2 = (t2 as *const u8).offset(off2 as isize) as *const u16;

    if wcscmp(str1, str2) == 0 {
        1u8
    } else {
        0u8
    }
}

unsafe fn td_equal(t1: *const DVTARGETDEVICE, t2: *const DVTARGETDEVICE) -> u8 {
    if t1.is_null() && t2.is_null() {
        return 1u8;
    }
    if t1.is_null() || t2.is_null() {
        return 0u8;
    }

    if string_off_equal(t1, (*t1).tdDriverNameOffset, t2, (*t2).tdDriverNameOffset) == 0 {
        return 0u8;
    }
    if string_off_equal(t1, (*t1).tdDeviceNameOffset, t2, (*t2).tdDeviceNameOffset) == 0 {
        return 0u8;
    }
    if string_off_equal(t1, (*t1).tdPortNameOffset, t2, (*t2).tdPortNameOffset) == 0 {
        return 0u8;
    }

    1u8
}
unsafe fn snapshot_GetData(
    iface: *mut IDataObject,
    fmt: *mut FORMATETC,
    med: *mut STGMEDIUM,
) -> HRESULT {
    let This = impl_from_IDataObject(iface);
    let mut h: *mut core::ffi::c_void;
    let mut hr: HRESULT;
    let mut enum_data: *mut ole_priv_data = core::ptr::null_mut();
    let entry: *mut ole_priv_data_entry;
    let mask: DWORD;

    TRACE!(
        b"(%p, %p {%s}, %p)\n\0".as_ptr() as *const i8,
        iface,
        fmt,
        dump_fmtetc(fmt),
        med,
    );

    if fmt.is_null() || med.is_null() {
        return E_INVALIDARG;
    }

    memset(
        med as *mut core::ffi::c_void,
        0,
        core::mem::size_of::<STGMEDIUM>(),
    );

    if OpenClipboard(core::ptr::null_mut()) == 0 {
        return CLIPBRD_E_CANT_OPEN;
    }

    if (*This).data.is_null() {
        hr = get_current_dataobject(&mut (*This).data);
    }

    if !(*This).data.is_null() {
        hr = IDataObject_GetData((*This).data, fmt, med);
        if SUCCEEDED(hr) != 0 {
            CloseClipboard();
            return hr;
        }
    }

    if (*fmt).lindex != -1 {
        hr = DV_E_FORMATETC;
        HeapFree(GetProcessHeap(), 0, enum_data as LPVOID);
        if CloseClipboard() == 0 {
            hr = CLIPBRD_E_CANT_CLOSE;
        }
        return hr;
    }

    if IsClipboardFormatAvailable((*fmt).cfFormat) == 0 {
        hr = DV_E_FORMATETC;
        HeapFree(GetProcessHeap(), 0, enum_data as LPVOID);
        if CloseClipboard() == 0 {
            hr = CLIPBRD_E_CANT_CLOSE;
        }
        return hr;
    }

    hr = get_priv_data(&mut enum_data);
    if FAILED(hr) != 0 {
        HeapFree(GetProcessHeap(), 0, enum_data as LPVOID);
        if CloseClipboard() == 0 {
            hr = CLIPBRD_E_CANT_CLOSE;
        }
        return hr;
    }

    entry = find_format_in_list((*enum_data).entries.as_mut_ptr(), (*enum_data).count, (*fmt).cfFormat);
    if !entry.is_null() {
        if td_equal((*fmt).ptd, (*entry).fmtetc.ptd) == 0 {
            hr = DV_E_FORMATETC;
            HeapFree(GetProcessHeap(), 0, enum_data as LPVOID);
            if CloseClipboard() == 0 {
                hr = CLIPBRD_E_CANT_CLOSE;
            }
            return hr;
        }
        mask = (*fmt).tymed & (*entry).fmtetc.tymed;
        if mask == 0 && ((*entry).fmtetc.tymed & (TYMED_ISTREAM | TYMED_HGLOBAL | TYMED_ISTORAGE)) != 0 {
            mask = (*fmt).tymed & (TYMED_ISTREAM | TYMED_HGLOBAL | TYMED_ISTORAGE);
        }
    } else {
        mask = (*fmt).tymed & get_tymed_from_nonole_cf((*fmt).cfFormat);
    }

    if mask == 0 {
        hr = DV_E_TYMED;
        HeapFree(GetProcessHeap(), 0, enum_data as LPVOID);
        if CloseClipboard() == 0 {
            hr = CLIPBRD_E_CANT_CLOSE;
        }
        return hr;
    }

    h = GetClipboardData((*fmt).cfFormat);
    if h.is_null() {
        hr = DV_E_FORMATETC;
        HeapFree(GetProcessHeap(), 0, enum_data as LPVOID);
        if CloseClipboard() == 0 {
            hr = CLIPBRD_E_CANT_CLOSE;
        }
        return hr;
    }

    if (mask & TYMED_HGLOBAL) != 0 {
        hr = get_stgmed_for_global(h as HGLOBAL, med);
    } else if (mask & TYMED_ISTREAM) != 0 {
        hr = get_stgmed_for_stream(h as HGLOBAL, med);
    } else if (mask & TYMED_ISTORAGE) != 0 {
        hr = get_stgmed_for_storage(h as HGLOBAL, med);
    } else if (mask & TYMED_ENHMF) != 0 {
        hr = get_stgmed_for_emf(h as HENHMETAFILE, med);
    } else if (mask & TYMED_GDI) != 0 {
        hr = get_stgmed_for_bitmap(h as HBITMAP, med);
    } else {
        FIXME!(
            b"Unhandled tymed - mask %x req tymed %x\n\0".as_ptr() as *const i8,
            mask,
            (*fmt).tymed,
        );
        hr = E_FAIL;
        HeapFree(GetProcessHeap(), 0, enum_data as LPVOID);
        if CloseClipboard() == 0 {
            hr = CLIPBRD_E_CANT_CLOSE;
        }
        return hr;
    }

    HeapFree(GetProcessHeap(), 0, enum_data as LPVOID);
    if CloseClipboard() == 0 {
        hr = CLIPBRD_E_CANT_CLOSE;
    }
    hr
}

unsafe fn snapshot_GetDataHere(
    iface: *mut IDataObject,
    fmt: *mut FORMATETC,
    med: *mut STGMEDIUM,
) -> HRESULT {
    let This = impl_from_IDataObject(iface);
    let mut h: *mut core::ffi::c_void;
    let mut hr: HRESULT;
    let mut enum_data: *mut ole_priv_data = core::ptr::null_mut();
    let mut entry: *mut ole_priv_data_entry;
    let mut supported: DWORD;

    if fmt.is_null() || med.is_null() {
        return E_INVALIDARG;
    }

    TRACE!(
        b"(%p, %p {%s}, %p (tymed %x)\n\0".as_ptr() as *const i8,
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
        if SUCCEEDED(hr) != 0 {
            CloseClipboard();
            return hr;
        }
    }

    h = GetClipboardData((*fmt).cfFormat);
    if h.is_null() {
        hr = DV_E_FORMATETC;
        'end: loop {
            HeapFree(GetProcessHeap(), 0, enum_data as LPVOID);
            if CloseClipboard() == 0 {
                hr = CLIPBRD_E_CANT_CLOSE;
            }
            return hr;
        }
    }

    hr = get_priv_data(core::ptr::addr_of_mut!(enum_data));
    if FAILED(hr) != 0 {
        'end: loop {
            HeapFree(GetProcessHeap(), 0, enum_data as LPVOID);
            if CloseClipboard() == 0 {
                hr = CLIPBRD_E_CANT_CLOSE;
            }
            return hr;
        }
    }

    entry = find_format_in_list((*enum_data).entries.as_mut_ptr(), (*enum_data).count, (*fmt).cfFormat);
    if !entry.is_null() {
        if td_equal((*fmt).ptd, (*entry).fmtetc.ptd) == 0 {
            hr = DV_E_FORMATETC;
            'end: loop {
                HeapFree(GetProcessHeap(), 0, enum_data as LPVOID);
                if CloseClipboard() == 0 {
                    hr = CLIPBRD_E_CANT_CLOSE;
                }
                return hr;
            }
        }
        supported = (*entry).fmtetc.tymed;
    } else {
        supported = TYMED_HGLOBAL;
    }

    match (*med).tymed {
        TYMED_HGLOBAL => {
            let src_size = GlobalSize(h);
            let dst_size = GlobalSize((*med).u.hGlobal);
            hr = E_FAIL;
            if dst_size >= src_size {
                let src = GlobalLock(h);
                let dst = GlobalLock((*med).u.hGlobal);

                memcpy(dst, src, src_size as usize);
                GlobalUnlock((*med).u.hGlobal);
                GlobalUnlock(h);
                hr = S_OK;
            }
        }
        TYMED_ISTREAM => {
            let src_size = GlobalSize(h);
            let src = GlobalLock(h);
            hr = IStream_Write((*med).u.pstm, src, src_size, core::ptr::null_mut());
            GlobalUnlock(h);
        }
        TYMED_ISTORAGE => {
            let mut copy: STGMEDIUM = core::mem::zeroed();
            if (supported & TYMED_ISTORAGE) == 0 {
                hr = E_FAIL;
                'end: loop {
                    HeapFree(GetProcessHeap(), 0, enum_data as LPVOID);
                    if CloseClipboard() == 0 {
                        hr = CLIPBRD_E_CANT_CLOSE;
                    }
                    return hr;
                }
            }
            hr = get_stgmed_for_storage(h as HGLOBAL, core::ptr::addr_of_mut!(copy));
            if SUCCEEDED(hr) != 0 {
                hr = IStorage_CopyTo(
                    copy.u.pstg,
                    0,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    (*med).u.pstg,
                );
                ReleaseStgMedium(core::ptr::addr_of_mut!(copy));
            }
        }
        _ => {
            FIXME!(
                b"Unhandled tymed - supported %x req tymed %x\n\0".as_ptr() as *const i8,
                supported,
                (*med).tymed,
            );
            hr = E_FAIL;
            'end: loop {
                HeapFree(GetProcessHeap(), 0, enum_data as LPVOID);
                if CloseClipboard() == 0 {
                    hr = CLIPBRD_E_CANT_CLOSE;
                }
                return hr;
            }
        }
    }

    HeapFree(GetProcessHeap(), 0, enum_data as LPVOID);
    if CloseClipboard() == 0 {
        hr = CLIPBRD_E_CANT_CLOSE;
    }
    hr
}

unsafe fn snapshot_QueryGetData(iface: *mut IDataObject, fmt: *mut FORMATETC) -> HRESULT {

    TRACE!(b"(%p, %p {%s})\n\0".as_ptr() as *const i8, iface, fmt, dump_fmtetc(fmt));

    if fmt.is_null() {
        return 0x80070057u32 as HRESULT;
    }

    if (*fmt).dwAspect != 1 {
        return 0x80040064u32 as HRESULT;
    }

    if (*fmt).lindex != -1 {
        return 0x80040064u32 as HRESULT;
    }

    if IsClipboardFormatAvailable((*fmt).cfFormat) != 0 {
        0x00000000u32 as HRESULT
    } else {
        0x80040064u32 as HRESULT
    }
}

unsafe fn snapshot_GetCanonicalFormatEtc(
    iface: *mut IDataObject,
    fmt_in: *mut FORMATETC,
    fmt_out: *mut FORMATETC,
) -> HRESULT {
    TRACE!(
        b"(%p, %p, %p)\n\0".as_ptr() as *const i8,
        iface,
        fmt_in,
        fmt_out,
    );

    if fmt_in.is_null() || fmt_out.is_null() {
        return E_INVALIDARG;
    }

    *fmt_out = *fmt_in;
    DATA_S_SAMEFORMATETC
}

unsafe fn snapshot_SetData(
    iface: *mut IDataObject,
    fmt: *mut FORMATETC,
    med: *mut STGMEDIUM,
    release: BOOL,
) -> HRESULT {
    TRACE!(b"(%p, %p, %p, %d): not implemented\n\0".as_ptr() as *const i8, iface, fmt, med, release);
    E_NOTIMPL
}

unsafe fn snapshot_EnumFormatEtc(iface: *mut IDataObject, dir: DWORD,
                                 enum_fmt: *mut *mut IEnumFORMATETC) -> HRESULT {
    let mut hr: HRESULT;
    let mut data: *mut ole_priv_data = core::ptr::null_mut();

    TRACE!(b"(%p, %x, %p)\n\0".as_ptr() as *const i8, iface, dir, enum_fmt);

    *enum_fmt = core::ptr::null_mut();

    if dir != DATADIR_GET { return E_NOTIMPL; }
    if OpenClipboard(core::ptr::null_mut()) == 0 { return CLIPBRD_E_CANT_OPEN; }

    hr = get_priv_data(core::ptr::addr_of_mut!(data));

    if FAILED(hr) != 0 {
        if CloseClipboard() == 0 { hr = CLIPBRD_E_CANT_CLOSE; }
        return hr;
    }

    hr = enum_fmtetc_construct(data, 0, enum_fmt);

    if CloseClipboard() == 0 { hr = CLIPBRD_E_CANT_CLOSE; }
    hr
}

unsafe fn snapshot_DAdvise(
    iface: *mut IDataObject,
    fmt: *mut FORMATETC,
    flags: DWORD,
    sink: *mut IAdviseSink,
    conn: *mut DWORD,
) -> HRESULT {
    TRACE!(
        b"(%p, %p, %x, %p, %p): not implemented\n\0".as_ptr() as PCSTR,
        iface,
        fmt,
        flags,
        sink,
        conn,
    );
    E_NOTIMPL
}

unsafe fn snapshot_DUnadvise(iface: *mut IDataObject, conn: DWORD) -> HRESULT {
    TRACE!(b"(%p, %d): not implemented\n\0".as_ptr() as PCHAR, iface, conn);
    E_NOTIMPL
}

unsafe fn snapshot_EnumDAdvise(
    iface: *mut IDataObject,
    enum_advise: *mut *mut IEnumSTATDATA,
) -> HRESULT {
    TRACE!(
        b"(%p, %p): not implemented\n\0".as_ptr() as PCHAR,
        iface,
        enum_advise,
    );
    E_NOTIMPL
}

unsafe fn snapshot_construct(seq_no: DWORD) -> *mut snapshot {
    let This: *mut snapshot = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<snapshot>()
    ) as *mut snapshot;

    if This.is_null() {
        return core::ptr::null_mut();
    }

    (*This).IDataObject_iface.lpVtbl = &snapshot_vtable;
    (*This).ref_ = 0;
    (*This).seq_no = seq_no;
    (*This).data = core::ptr::null_mut();

    This
}

unsafe fn register_clipboard_formats() {
    static OwnerLink: [u16; 10] = [b'O' as u16, b'w' as u16, b'n' as u16, b'e' as u16, b'r' as u16, b'L' as u16, b'i' as u16, b'n' as u16, b'k' as u16, 0];
    static FileName: [u16; 9] = [b'F' as u16, b'i' as u16, b'l' as u16, b'e' as u16, b'N' as u16, b'a' as u16, b'm' as u16, b'e' as u16, 0];
    static FileNameW: [u16; 10] = [b'F' as u16, b'i' as u16, b'l' as u16, b'e' as u16, b'N' as u16, b'a' as u16, b'm' as u16, b'e' as u16, b'W' as u16, 0];
    static DataObject: [u16; 11] = [b'D' as u16, b'a' as u16, b't' as u16, b'a' as u16, b'O' as u16, b'b' as u16, b'j' as u16, b'e' as u16, b'c' as u16, b't' as u16, 0];
    static EmbeddedObject: [u16; 16] = [b'E' as u16, b'm' as u16, b'b' as u16, b'e' as u16, b'd' as u16, b'd' as u16, b'e' as u16, b'd' as u16, b' ' as u16, b'O' as u16, b'b' as u16, b'j' as u16, b'e' as u16, b'c' as u16, b't' as u16, 0];
    static EmbedSource: [u16; 13] = [b'E' as u16, b'm' as u16, b'b' as u16, b'e' as u16, b'd' as u16, b' ' as u16, b'S' as u16, b'o' as u16, b'u' as u16, b'r' as u16, b'c' as u16, b'e' as u16, 0];
    static CustomLinkSource: [u16; 19] = [b'C' as u16, b'u' as u16, b's' as u16, b't' as u16, b'o' as u16, b'm' as u16, b' ' as u16, b'L' as u16, b'i' as u16, b'n' as u16, b'k' as u16, b' ' as u16, b'S' as u16, b'o' as u16, b'u' as u16, b'r' as u16, b'c' as u16, b'e' as u16, 0];
    static LinkSource: [u16; 12] = [b'L' as u16, b'i' as u16, b'n' as u16, b'k' as u16, b' ' as u16, b'S' as u16, b'o' as u16, b'u' as u16, b'r' as u16, b'c' as u16, b'e' as u16, 0];
    static ObjectDescriptor: [u16; 18] = [b'O' as u16, b'b' as u16, b'j' as u16, b'e' as u16, b'c' as u16, b't' as u16, b' ' as u16, b'D' as u16, b'e' as u16, b's' as u16, b'c' as u16, b'r' as u16, b'i' as u16, b'p' as u16, b't' as u16, b'o' as u16, b'r' as u16, 0];
    static LinkSourceDescriptor: [u16; 23] = [b'L' as u16, b'i' as u16, b'n' as u16, b'k' as u16, b' ' as u16, b'S' as u16, b'o' as u16, b'u' as u16, b'r' as u16, b'c' as u16, b'e' as u16, b' ' as u16, b'D' as u16, b'e' as u16, b's' as u16, b'c' as u16, b'r' as u16, b'i' as u16, b'p' as u16, b't' as u16, b'o' as u16, b'r' as u16, 0];
    static OlePrivateData: [u16; 17] = [b'O' as u16, b'l' as u16, b'e' as u16, b' ' as u16, b'P' as u16, b'r' as u16, b'i' as u16, b'v' as u16, b'a' as u16, b't' as u16, b'e' as u16, b' ' as u16, b'D' as u16, b'a' as u16, b't' as u16, b'a' as u16, 0];
    static WineMarshalledDataObject: [u16; 27] = [b'W' as u16, b'i' as u16, b'n' as u16, b'e' as u16, b' ' as u16, b'M' as u16, b'a' as u16, b'r' as u16, b's' as u16, b'h' as u16, b'a' as u16, b'l' as u16, b'l' as u16, b'e' as u16, b'd' as u16, b' ' as u16, b'D' as u16, b'a' as u16, b't' as u16, b'a' as u16, b'O' as u16, b'b' as u16, b'j' as u16, b'e' as u16, b'c' as u16, b't' as u16, 0];

    ownerlink_clipboard_format = RegisterClipboardFormatW(OwnerLink.as_ptr());
    filename_clipboard_format = RegisterClipboardFormatW(FileName.as_ptr());
    filenameW_clipboard_format = RegisterClipboardFormatW(FileNameW.as_ptr());
    dataobject_clipboard_format = RegisterClipboardFormatW(DataObject.as_ptr());
    embedded_object_clipboard_format = RegisterClipboardFormatW(EmbeddedObject.as_ptr());
    embed_source_clipboard_format = RegisterClipboardFormatW(EmbedSource.as_ptr());
    custom_link_source_clipboard_format = RegisterClipboardFormatW(CustomLinkSource.as_ptr());
    link_source_clipboard_format = RegisterClipboardFormatW(LinkSource.as_ptr());
    object_descriptor_clipboard_format = RegisterClipboardFormatW(ObjectDescriptor.as_ptr());
    link_source_descriptor_clipboard_format = RegisterClipboardFormatW(LinkSourceDescriptor.as_ptr());
    ole_private_data_clipboard_format = RegisterClipboardFormatW(OlePrivateData.as_ptr());

    wine_marshal_clipboard_format = RegisterClipboardFormatW(WineMarshalledDataObject.as_ptr());
}
#[no_mangle]
pub unsafe extern "C" fn OLEClipbrd_Initialize() {

    register_clipboard_formats();

    if theOleClipboard.is_null() {
        let clipbrd: *mut ole_clipbrd;
        let h: HGLOBAL;

        TRACE!(b"()\n\0".as_ptr());

        clipbrd = HeapAlloc(GetProcessHeap(), 0, core::mem::size_of::<ole_clipbrd>()) as *mut ole_clipbrd;
        if clipbrd.is_null() {
            return;
        }

        (*clipbrd).latest_snapshot = core::ptr::null_mut();
        (*clipbrd).window = core::ptr::null_mut();
        (*clipbrd).src_data = core::ptr::null_mut();
        (*clipbrd).cached_enum = core::ptr::null_mut();

        h = GlobalAlloc(GMEM_DDESHARE | GMEM_MOVEABLE, 0);
        if h.is_null() {
            HeapFree(GetProcessHeap(), 0, clipbrd as LPVOID);
            return;
        }

        if FAILED(CreateStreamOnHGlobal(h, 1, core::ptr::addr_of_mut!((*clipbrd).marshal_data))) != 0 {
            GlobalFree(h);
            HeapFree(GetProcessHeap(), 0, clipbrd as LPVOID);
            return;
        }

        theOleClipboard = clipbrd;
    }
}

unsafe fn set_clipboard_formats(clipbrd: *mut ole_clipbrd, data: *mut IDataObject) -> HRESULT {
    let mut hr: HRESULT;
    let mut fmt: FORMATETC = core::mem::zeroed();
    let mut enum_fmt: *mut IEnumFORMATETC = core::ptr::null_mut();
    let mut priv_data_handle: HGLOBAL;
    let mut target_offset: DWORD_PTR;
    let mut priv_data: *mut ole_priv_data;
    let mut count: DWORD = 0;
    let mut needed: DWORD = core::mem::size_of::<ole_priv_data>() as DWORD;
    let mut idx: DWORD;

    hr = IDataObject_EnumFormatEtc(data, DATADIR_GET, &mut enum_fmt);
    if FAILED(hr) != 0 {
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

    needed += core::mem::size_of::<ole_priv_data_entry>() as DWORD;
    priv_data_handle = GlobalAlloc(GMEM_DDESHARE | GMEM_MOVEABLE | GMEM_ZEROINIT, needed as usize);
    priv_data = GlobalLock(priv_data_handle) as *mut ole_priv_data;

    (*priv_data).unk1 = 0;
    (*priv_data).size = needed;
    (*priv_data).unk2 = 1;
    (*priv_data).count = count;
    (*priv_data).unk3[0] = 0;
    (*priv_data).unk3[1] = 0;

    IEnumFORMATETC_Reset(enum_fmt);

    idx = 0;
    target_offset = FIELD_OFFSET(ole_priv_data, entries, count as usize + 1);

    while IEnumFORMATETC_Next(enum_fmt, 1, &mut fmt, core::ptr::null_mut()) == S_OK {
        TRACE!(b"%s\n\0".as_ptr() as *const i8, dump_fmtetc(&fmt));

        (*priv_data).entries[idx as usize].fmtetc = fmt;
        if !fmt.ptd.is_null() {
            core::ptr::copy_nonoverlapping(
                fmt.ptd as *const u8,
                (priv_data as *mut u8).add(target_offset as usize),
                (*fmt.ptd).tdSize as usize,
            );
            (*priv_data).entries[idx as usize].fmtetc.ptd = target_offset as *mut DVTARGETDEVICE;
            target_offset += (*fmt.ptd).tdSize as DWORD_PTR;
            CoTaskMemFree(fmt.ptd as LPVOID);
        }

        (*priv_data).entries[idx as usize].first_use = if find_format_in_list(
            (*priv_data).entries.as_mut_ptr(),
            idx,
            fmt.cfFormat,
        ).is_null() { 1 } else { 0 };
        (*priv_data).entries[idx as usize].unk[0] = 0;
        (*priv_data).entries[idx as usize].unk[1] = 0;

        if (*priv_data).entries[idx as usize].first_use != 0 {
            SetClipboardData(fmt.cfFormat, core::ptr::null_mut());
        }

        idx += 1;
    }

    IEnumFORMATETC_Release(enum_fmt);

    (*clipbrd).cached_enum = HeapAlloc(GetProcessHeap(), 0, needed as usize) as *mut ole_priv_data;
    core::ptr::copy_nonoverlapping(
        priv_data as *const u8,
        (*clipbrd).cached_enum as *mut u8,
        needed as usize,
    );
    idx = 0;
    while idx < (*(*clipbrd).cached_enum).count {
        (*(*clipbrd).cached_enum).entries[idx as usize].fmtetc.ptd =
            td_offs_to_ptr((*clipbrd).cached_enum, (*(*clipbrd).cached_enum).entries[idx as usize].fmtetc.ptd as DWORD_PTR);
        idx += 1;
    }

    GlobalUnlock(priv_data_handle);
    if SetClipboardData(ole_private_data_clipboard_format, priv_data_handle) == 0 {
        GlobalFree(priv_data_handle);
        return CLIPBRD_E_CANT_SET;
    }

    S_OK
}

unsafe fn get_clipbrd_window(clipbrd: *mut ole_clipbrd, wnd: *mut HWND) -> HRESULT {
    if IsWindow((*clipbrd).window) == 0 {
        (*clipbrd).window = create_clipbrd_window();
    }

    if (*clipbrd).window.is_null() {
        (*clipbrd).window = create_clipbrd_window();
    }

    *wnd = (*clipbrd).window;
    if !(*wnd).is_null() { S_OK } else { E_FAIL }
}

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

const STREAM_SEEK_SET: DWORD = 0;

unsafe fn expose_marshalled_dataobject(clipbrd: *mut ole_clipbrd, data: *mut IDataObject) -> HRESULT {

    let h: HGLOBAL;

    if !data.is_null() {
        let mut h_stm: HGLOBAL = core::ptr::null_mut();
        GetHGlobalFromStream((*clipbrd).marshal_data, &mut h_stm);
        let mut temp_h: HGLOBAL = core::ptr::null_mut();
        dup_global_mem(h_stm, GMEM_DDESHARE | GMEM_MOVEABLE, &mut temp_h);
        h = temp_h;
    } else {
        h = GlobalAlloc(GMEM_DDESHARE | GMEM_MOVEABLE, 1);
    }

    if h.is_null() {
        return 0x8007000E; // E_OUTOFMEMORY
    }

    if SetClipboardData(wine_marshal_clipboard_format, h).is_null() {
        GlobalFree(h);
        return 0x800401D3; // CLIPBRD_E_CANT_SET
    }
    0x00000000 // S_OK
}

unsafe fn set_src_dataobject(clipbrd: *mut ole_clipbrd, data: *mut IDataObject) -> HRESULT {
    let mut hr: HRESULT;
    let mut wnd: HWND = core::ptr::null_mut();

    hr = get_clipbrd_window(clipbrd, &mut wnd);
    if FAILED(hr) != 0 {
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

        IDataObject_QueryInterface(data, &IID_IUnknown, &mut unk as *mut *mut IUnknown as *mut *mut core::ffi::c_void);
        hr = CoMarshalInterface((*clipbrd).marshal_data, &IID_IDataObject, unk,
                                MSHCTX_LOCAL, core::ptr::null_mut(), MSHLFLAGS_TABLESTRONG);
        IUnknown_Release(unk);
        if FAILED(hr) != 0 {
            return hr;
        }
        hr = set_clipboard_formats(clipbrd, data);
    }
    hr
}
#[no_mangle]
pub unsafe extern "C" fn OLEClipbrd_UnInitialize() {

    let clipbrd: *mut ole_clipbrd = theOleClipboard;

    TRACE!(b"()\n\0".as_ptr());

    if !clipbrd.is_null() {
        static OLE32W: [u16; 6] = [b'o' as u16, b'l' as u16, b'e' as u16, b'3' as u16, b'2' as u16, 0];
        let hinst: HINSTANCE = GetModuleHandleW(OLE32W.as_ptr());

        if !(*clipbrd).src_data.is_null() {
            IDataObject_AddRef((*clipbrd).src_data);
            set_src_dataobject(clipbrd, core::ptr::null_mut());
        }

        if !(*clipbrd).window.is_null() {
            DestroyWindow((*clipbrd).window);
            UnregisterClassW(clipbrd_wndclass.as_ptr(), hinst);
        }

        IStream_Release((*clipbrd).marshal_data);
        HeapFree(GetProcessHeap(), 0, clipbrd as LPVOID);
        theOleClipboard = core::ptr::null_mut();
    }
}
unsafe fn clipbrd_wndproc(hwnd: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let mut clipbrd: *mut ole_clipbrd = core::ptr::null_mut();

    get_ole_clipbrd(&mut clipbrd);
    if clipbrd.is_null() {
        return DefWindowProcW(hwnd, message, wparam, lparam);
    }

    match message {
        WM_RENDERFORMAT => {
            if !(*clipbrd).cached_enum.is_null() {
                let cf = wparam as UINT;
                let entry: *mut ole_priv_data_entry;

                TRACE!(b"(): WM_RENDERFORMAT(cfFormat=%x)\n\0".as_ptr() as *const i8, cf);
                entry = find_format_in_list(
                    (*(*clipbrd).cached_enum).entries.as_mut_ptr(),
                    (*(*clipbrd).cached_enum).count,
                    cf,
                );

                if !entry.is_null() {
                    render_format((*clipbrd).src_data, &mut (*entry).fmtetc);
                }
            }
        }

        WM_RENDERALLFORMATS => {
            let mut i: DWORD;
            let entries: *mut ole_priv_data_entry;

            TRACE!(b"(): WM_RENDERALLFORMATS\n\0".as_ptr() as *const i8);

            if clipbrd.is_null() || (*clipbrd).cached_enum.is_null() {
                return 0;
            }
            entries = (*(*clipbrd).cached_enum).entries.as_mut_ptr();
            i = 0;
            while i < (*(*clipbrd).cached_enum).count {
                if (*entries.add(i as usize)).first_use != 0 {
                    render_format((*clipbrd).src_data, &mut (*entries.add(i as usize)).fmtetc);
                }
                i += 1;
            }
        }

        WM_DESTROYCLIPBOARD => {
            TRACE!(b"(): WM_DESTROYCLIPBOARD\n\0".as_ptr() as *const i8);

            set_src_dataobject(clipbrd, core::ptr::null_mut());
        }

        _ => {
            return DefWindowProcW(hwnd, message, wparam, lparam);
        }
    }

    0
}

unsafe fn create_clipbrd_window() -> HWND {

    static ole32W: [u16; 6] = [b'o' as u16, b'l' as u16, b'e' as u16, b'3' as u16, b'2' as u16, 0];
    static title: [u16; 16] = [
        b'C' as u16, b'l' as u16, b'i' as u16, b'p' as u16, b'b' as u16, b'o' as u16, b'a' as u16,
        b'r' as u16, b'd' as u16, b'W' as u16, b'i' as u16, b'n' as u16, b'd' as u16, b'o' as u16,
        b'w' as u16, 0,
    ];

    let hinst = GetModuleHandleW(ole32W.as_ptr());

    let mut class: WNDCLASSEXW = core::mem::zeroed();
    class.cbSize = core::mem::size_of::<WNDCLASSEXW>() as u32;
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

    RegisterClassExW(&class);

    CreateWindowW(
        clipbrd_wndclass.as_ptr(),
        title.as_ptr(),
        WS_POPUP | WS_CLIPSIBLINGS | WS_OVERLAPPED,
        0,
        0,
        0,
        0,
        -3isize as HWND,
        core::ptr::null_mut(),
        hinst,
        core::ptr::null_mut(),
    )
}

unsafe fn set_dataobject_format(hwnd: HWND) -> HRESULT {
    let h: HGLOBAL = GlobalAlloc(GMEM_DDESHARE | GMEM_MOVEABLE, core::mem::size_of::<HWND>());
    let mut data: *mut HWND;

    if h.is_null() {
        return E_OUTOFMEMORY;
    }

    data = GlobalLock(h) as *mut HWND;
    *data = hwnd;
    GlobalUnlock(h);

    if SetClipboardData(dataobject_clipboard_format, h as *mut core::ffi::c_void).is_null() {
        GlobalFree(h);
        return CLIPBRD_E_CANT_SET;
    }

    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn OleSetClipboard(data: *mut IDataObject) -> HRESULT {
    let mut hr: HRESULT;
    let mut clipbrd: *mut ole_clipbrd = core::ptr::null_mut();
    let mut wnd: HWND = core::ptr::null_mut();

    TRACE!(b"(%p)\n\0".as_ptr(), data);

    hr = get_ole_clipbrd(&mut clipbrd);
    if FAILED(hr) != 0 {
        return hr;
    }

    hr = get_clipbrd_window(clipbrd, &mut wnd);
    if FAILED(hr) != 0 {
        return hr;
    }

    if OpenClipboard(wnd) == 0 {
        return 0x800401D0u32 as HRESULT;
    }

    'end: loop {
        if EmptyClipboard() == 0 {
            hr = 0x800401D1u32 as HRESULT;
            break 'end;
        }

        hr = set_src_dataobject(clipbrd, data);
        if FAILED(hr) != 0 {
            break 'end;
        }

        if !data.is_null() {
            hr = expose_marshalled_dataobject(clipbrd, data);
            if FAILED(hr) != 0 {
                break 'end;
            }
            hr = set_dataobject_format(wnd);
        }

        break;
    }

    if CloseClipboard() == 0 {
        hr = 0x800401D4u32 as HRESULT;
    }

    if FAILED(hr) != 0 {
        expose_marshalled_dataobject(clipbrd, core::ptr::null_mut());
        set_src_dataobject(clipbrd, core::ptr::null_mut());
    }

    hr
}

#[no_mangle]
pub unsafe extern "system" fn OleGetClipboard(obj: *mut *mut IDataObject) -> HRESULT {

    if obj.is_null() {
        return E_INVALIDARG;
    }
    *obj = core::ptr::null_mut();

    let mut clipbrd: *mut ole_clipbrd = core::ptr::null_mut();
    let hr = get_ole_clipbrd(&mut clipbrd);
    if FAILED(hr) != 0 {
        return hr;
    }

    let seq_no = GetClipboardSequenceNumber();
    EnterCriticalSection(&mut latest_snapshot_cs);
    if !(*clipbrd).latest_snapshot.is_null() && (*(*clipbrd).latest_snapshot).seq_no != seq_no {
        (*clipbrd).latest_snapshot = core::ptr::null_mut();
    }

    if (*clipbrd).latest_snapshot.is_null() {
        (*clipbrd).latest_snapshot = snapshot_construct(seq_no);
        if (*clipbrd).latest_snapshot.is_null() {
            LeaveCriticalSection(&mut latest_snapshot_cs);
            return E_OUTOFMEMORY;
        }
    }

    *obj = &mut (*(*clipbrd).latest_snapshot).IDataObject_iface;
    IDataObject_AddRef(*obj);
    LeaveCriticalSection(&mut latest_snapshot_cs);

    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn OleFlushClipboard() -> HRESULT {
    let mut clipbrd: *mut ole_clipbrd = core::ptr::null_mut();
    let mut wnd: HWND = core::ptr::null_mut();

    TRACE!(b"()\n\0".as_ptr() as *const i8);

    let mut hr = get_ole_clipbrd(&mut clipbrd);
    if FAILED(hr) != 0 {
        return hr;
    }

    hr = get_clipbrd_window(clipbrd, &mut wnd);
    if FAILED(hr) != 0 {
        return hr;
    }

    if (*clipbrd).src_data.is_null() {
        return 0;
    }

    if OpenClipboard(wnd) == 0 {
        return 0x800401D3u32 as HRESULT;
    }

    SendMessageW(wnd, WM_RENDERALLFORMATS, 0, 0);

    hr = set_dataobject_format(core::ptr::null_mut());

    expose_marshalled_dataobject(clipbrd, core::ptr::null_mut());
    set_src_dataobject(clipbrd, core::ptr::null_mut());

    if CloseClipboard() == 0 {
        hr = 0x800401D4u32 as HRESULT;
    }

    hr
}

#[no_mangle]
pub unsafe extern "system" fn OleIsCurrentClipboard(data: *mut IDataObject) -> HRESULT {
    let mut clipbrd: *mut ole_clipbrd = core::ptr::null_mut();

    let hr = get_ole_clipbrd(&mut clipbrd);
    if FAILED(hr) != 0 {
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
