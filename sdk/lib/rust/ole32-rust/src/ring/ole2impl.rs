use crate::prelude::*;
use core::ffi::c_void;

extern "C" {

    fn CopyEnhMetaFileW(hSrc: HANDLE, lpFileName: *const u16) -> HANDLE;
    fn CopyMetaFileW(hSrc: HANDLE, lpFileName: *const u16) -> HANDLE;
    fn GetPaletteEntries(hpal: HANDLE, iStart: UINT, cEntries: UINT, pPalEntries: *mut PALETTEENTRY) -> UINT;
    fn CreatePalette(plpal: *const LOGPALETTE) -> HANDLE;
    fn GetObjectW(h: HANDLE, c: c_int, pv: *mut c_void) -> c_int;
    fn GetBitmapBits(hbm: HANDLE, cb: LONG, lpvBits: LPVOID) -> LONG;
    fn CreateBitmapIndirect(pbm: *const BITMAP) -> HANDLE;
    fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    fn CreateFileMoniker(lpszPathName: *const u16, ppmk: *mut *mut IMoniker) -> HRESULT;
    fn BindMoniker(pmk: *mut IMoniker, grfOpt: u32, iidResult: *const CLSID, ppvResult: *mut LPVOID) -> HRESULT;
    fn OleLoad(pStg: *mut IStorage, riid: *const CLSID, pClientSite: *mut IOleClientSite, ppvObj: *mut LPVOID) -> HRESULT;
    fn ReleaseStgMedium(pmedium: *mut STGMEDIUM) -> ();
    fn memset(s: *mut core::ffi::c_void, c: i32, n: usize) -> *mut core::ffi::c_void;
}

// ---- file-scope preamble (translated) ----
// No file-scope declarations to translate from the provided preamble.
// The C preamble contains only preprocessor directives and a Wine debug macro,
// which have no Rust equivalents at the declaration level.

#[no_mangle]
pub unsafe extern "system" fn OleQueryCreateFromData(data: *mut IDataObject) -> HRESULT {
    let mut enum_fmt: *mut IEnumFORMATETC = core::ptr::null_mut();
    let mut fmt: FORMATETC = core::mem::zeroed();
    let mut found_static: BOOL = 0;
    let mut hr: HRESULT;

    hr = IDataObject_EnumFormatEtc(data, DATADIR_GET, &mut enum_fmt);

    if FAILED(hr) != 0 {
        return hr;
    }

    loop {
        hr = IEnumFORMATETC_Next(enum_fmt, 1, &mut fmt, core::ptr::null_mut());
        if hr == S_OK {
            if fmt.cfFormat == embedded_object_clipboard_format ||
               fmt.cfFormat == embed_source_clipboard_format ||
               fmt.cfFormat == filename_clipboard_format
            {
                IEnumFORMATETC_Release(enum_fmt);
                return S_OK;
            }

            if fmt.cfFormat == CF_METAFILEPICT ||
               fmt.cfFormat == CF_BITMAP ||
               fmt.cfFormat == CF_DIB
            {
                found_static = 1;
            }
        }

        if hr != S_OK {
            break;
        }
    }

    IEnumFORMATETC_Release(enum_fmt);

    if found_static != 0 {
        OLE_S_STATIC
    } else {
        S_FALSE
    }
}

unsafe fn init_fmtetc(fmt: *mut FORMATETC, cf: CLIPFORMAT, tymed: TYMED) {
    (*fmt).cfFormat = cf;
    (*fmt).ptd = core::ptr::null_mut();
    (*fmt).dwAspect = DVASPECT_CONTENT;
    (*fmt).lindex = -1i32;
    (*fmt).tymed = tymed;
}

unsafe fn get_storage(data: *mut IDataObject, stg: *mut IStorage, src_cf: *mut UINT, other_fmts: BOOL) -> HRESULT {
    static fmt_id: [UINT; 3] = [CF_METAFILEPICT, CF_BITMAP, CF_DIB];
    let mut i: UINT;
    let mut hr: HRESULT;
    let mut fmt: FORMATETC = core::mem::zeroed();
    let mut med: STGMEDIUM = core::mem::zeroed();
    let mut persist: *mut IPersistStorage = core::ptr::null_mut();
    let mut clsid: CLSID = core::mem::zeroed();

    if !src_cf.is_null() {
        *src_cf = 0;
    }

    init_fmtetc(&mut fmt, embedded_object_clipboard_format, TYMED_ISTORAGE);
    med.tymed = TYMED_ISTORAGE;
    med.u.pstg = stg;
    med.pUnkForRelease = core::ptr::null_mut();
    hr = IDataObject_GetDataHere(data, &mut fmt, &mut med);
    if SUCCEEDED(hr) != 0 {
        if !src_cf.is_null() {
            *src_cf = embedded_object_clipboard_format;
        }
        return hr;
    }

    init_fmtetc(&mut fmt, embed_source_clipboard_format, TYMED_ISTORAGE);
    med.tymed = TYMED_ISTORAGE;
    med.u.pstg = stg;
    med.pUnkForRelease = core::ptr::null_mut();
    hr = IDataObject_GetDataHere(data, &mut fmt, &mut med);
    if SUCCEEDED(hr) != 0 {
        if !src_cf.is_null() {
            *src_cf = embed_source_clipboard_format;
        }
        return hr;
    }

    if other_fmts != 0 {
        i = 0;
        while i < fmt_id.len() as UINT {
            init_fmtetc(&mut fmt, fmt_id[i as usize], TYMED_ISTORAGE);
            hr = IDataObject_QueryGetData(data, &mut fmt);
            if SUCCEEDED(hr) != 0 {
                if !src_cf.is_null() {
                    *src_cf = fmt_id[i as usize];
                }
                return hr;
            }
            i += 1;
        }
    }

    hr = IDataObject_QueryInterface(data, &IID_IPersistStorage, &mut persist as *mut *mut IPersistStorage as *mut LPVOID);
    if FAILED(hr) != 0 {
        return hr;
    }

    hr = IPersistStorage_GetClassID(persist, &mut clsid);
    if FAILED(hr) != 0 {
        IPersistStorage_Release(persist);
        return hr;
    }

    hr = IStorage_SetClass(stg, &clsid);
    if FAILED(hr) != 0 {
        IPersistStorage_Release(persist);
        return hr;
    }

    hr = IPersistStorage_Save(persist, stg, 0);
    if FAILED(hr) != 0 {
        IPersistStorage_Release(persist);
        return hr;
    }

    hr = IPersistStorage_SaveCompleted(persist, core::ptr::null_mut());

    IPersistStorage_Release(persist);

    hr
}
#[no_mangle]
pub unsafe extern "system" fn OleCreateFromDataEx(
    data: *mut IDataObject,
    iid: *const CLSID,
    flags: DWORD,
    renderopt: DWORD,
    num_cache_fmts: ULONG,
    adv_flags: *mut DWORD,
    cache_fmts: *mut FORMATETC,
    sink: *mut IAdviseSink,
    conns: *mut DWORD,
    client_site: *mut IOleClientSite,
    stg: *mut IStorage,
    obj: *mut LPVOID,
) -> HRESULT {
    let mut hr: HRESULT;
    let mut src_cf: UINT = 0;

    FIXME!(
        b"(%p, %s, %08x, %08x, %d, %p, %p, %p, %p, %p, %p, %p): stub\n\0".as_ptr() as *const i8,
        data,
        debugstr_guid(iid),
        flags,
        renderopt,
        num_cache_fmts,
        adv_flags,
        cache_fmts,
        sink,
        conns,
        client_site,
        stg,
        obj,
    );

    hr = get_storage(data, stg, &mut src_cf, 1);
    if FAILED(hr) {
        return hr;
    }

    hr = OleLoad(stg, iid, client_site, obj);
    if FAILED(hr) {
        return hr;
    }

    hr
}

#[no_mangle]
pub unsafe extern "system" fn OleCreateFromData(
    data: *mut IDataObject,
    iid: *const CLSID,
    renderopt: DWORD,
    fmt: *mut FORMATETC,
    client_site: *mut IOleClientSite,
    stg: *mut IStorage,
    obj: *mut LPVOID,
) -> HRESULT {
    let advf: DWORD = ADVF_PRIMEFIRST;

    OleCreateFromDataEx(
        data,
        iid,
        0,
        renderopt,
        if !fmt.is_null() { 1 } else { 0 },
        if !fmt.is_null() { &advf as *const DWORD as *mut DWORD } else { core::ptr::null_mut() },
        fmt,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        client_site,
        stg,
        obj,
    )
}

#[no_mangle]
pub unsafe extern "system" fn OleCreateLinkFromData(
    data: *mut IDataObject,
    iid: *const CLSID,
    renderopt: DWORD,
    fmt: *mut FORMATETC,
    client_site: *mut IOleClientSite,
    stg: *mut IStorage,
    obj: *mut LPVOID,
) -> HRESULT {

    FIXME!(
        b"%p,%s,%08x,%p,%p,%p,%p: semi-stub\n\0".as_ptr(),
        data,
        debugstr_guid(iid),
        renderopt,
        fmt,
        client_site,
        stg,
        obj,
    );
    OleCreateFromData(data, iid, renderopt, fmt, client_site, stg, obj)
}

#[no_mangle]
pub unsafe extern "system" fn OleCreateStaticFromData(
    data: *mut IDataObject,
    iid: *const CLSID,
    renderopt: DWORD,
    fmt: *mut FORMATETC,
    client_site: *mut IOleClientSite,
    stg: *mut IStorage,
    obj: *mut LPVOID,
) -> HRESULT {
    let mut hr: HRESULT;
    let mut clsid: CLSID;
    let mut ole_object: *mut IOleObject = core::ptr::null_mut();
    let mut ole_cache: *mut IOleCache2 = core::ptr::null_mut();
    let mut persist: *mut IPersistStorage = core::ptr::null_mut();
    let mut connection: DWORD = 0;
    let mut stgmedium: STGMEDIUM = core::mem::zeroed();
    let mut ole_typename: LPOLESTR;

    TRACE!(
        b"(%p, %s, 0x%08x, %p, %p, %p, %p)\n\0".as_ptr() as *const i8,
        data,
        debugstr_guid(iid),
        renderopt,
        fmt,
        client_site,
        stg,
        obj,
    );

    if obj.is_null() || stg.is_null() {
        return E_INVALIDARG;
    }

    if renderopt != OLERENDER_FORMAT {
        FIXME!(b"semi-stub\n\0".as_ptr() as *const i8);
        return OleCreateFromData(data, iid, renderopt, fmt, client_site, stg, obj);
    }

    if fmt.is_null() {
        return E_INVALIDARG;
    }

    hr = IDataObject_GetData(data, fmt, core::ptr::addr_of_mut!(stgmedium));
    if FAILED(hr) != 0 {
        return hr;
    }

    match (*fmt).cfFormat {
        CF_BITMAP | CF_DIB => {
            clsid = CLSID_Picture_Dib;
        }
        CF_ENHMETAFILE => {
            clsid = CLSID_Picture_EnhMetafile;
        }
        CF_METAFILEPICT => {
            clsid = CLSID_Picture_Metafile;
        }
        _ => {
            ReleaseStgMedium(core::ptr::addr_of_mut!(stgmedium));
            return DV_E_CLIPFORMAT;
        }
    }

    hr = OleCreateDefaultHandler(
        core::ptr::addr_of!(clsid),
        core::ptr::null_mut(),
        core::ptr::addr_of!(IID_IOleObject),
        core::ptr::addr_of_mut!(ole_object) as *mut *mut core::ffi::c_void,
    );
    if FAILED(hr) != 0 {
        'end: {
            break 'end;
        }
    } else {
        if !client_site.is_null() {
            hr = IOleObject_SetClientSite(ole_object, client_site);
            if FAILED(hr) != 0 {
                'end: {
                    break 'end;
                }
            }
        }

        hr = IOleObject_QueryInterface(
            ole_object,
            core::ptr::addr_of!(IID_IOleCache2),
            core::ptr::addr_of_mut!(ole_cache) as *mut *mut core::ffi::c_void,
        );
        if FAILED(hr) != 0 {
            'end: {
                break 'end;
            }
        }

        hr = IOleObject_QueryInterface(
            ole_object,
            core::ptr::addr_of!(IID_IPersistStorage),
            core::ptr::addr_of_mut!(persist) as *mut *mut core::ffi::c_void,
        );
        if FAILED(hr) != 0 {
            'end: {
                break 'end;
            }
        }

        hr = WriteClassStg(stg, core::ptr::addr_of!(clsid));
        if FAILED(hr) != 0 {
            'end: {
                break 'end;
            }
        }

        hr = IPersistStorage_InitNew(persist, stg);
        if FAILED(hr) != 0 {
            'end: {
                break 'end;
            }
        }

        hr = IOleCache2_Cache(ole_cache, fmt, ADVF_PRIMEFIRST, core::ptr::addr_of_mut!(connection));
        if FAILED(hr) != 0 {
            'end: {
                break 'end;
            }
        }

        hr = IOleCache2_SetData(ole_cache, fmt, core::ptr::addr_of_mut!(stgmedium), 1);
        if FAILED(hr) != 0 {
            'end: {
                break 'end;
            }
        }
        stgmedium.tymed = TYMED_NULL;

        hr = IOleObject_GetUserType(ole_object, USERCLASSTYPE_FULL, core::ptr::addr_of_mut!(ole_typename));
        if FAILED(hr) != 0 {
            ole_typename = core::ptr::null_mut();
        }
        hr = WriteFmtUserTypeStg(stg, (*fmt).cfFormat, ole_typename);
        CoTaskMemFree(ole_typename as *mut core::ffi::c_void);
        if FAILED(hr) != 0 {
            'end: {
                break 'end;
            }
        }

        hr = IPersistStorage_Save(persist, stg, 1);
        if FAILED(hr) != 0 {
            'end: {
                break 'end;
            }
        }

        hr = IPersistStorage_SaveCompleted(persist, core::ptr::null_mut());
        if FAILED(hr) != 0 {
            'end: {
                break 'end;
            }
        }

        hr = IOleObject_QueryInterface(ole_object, iid, obj);
    }

    if stgmedium.tymed == TYMED_NULL {
        ReleaseStgMedium(core::ptr::addr_of_mut!(stgmedium));
    }
    if !persist.is_null() {
        IPersistStorage_Release(persist);
    }
    if !ole_cache.is_null() {
        IOleCache2_Release(ole_cache);
    }
    if !ole_object.is_null() {
        IOleObject_Release(ole_object);
    }
    hr
}

#[no_mangle]
pub unsafe extern "system" fn OleCreateFromFileEx(
    clsid: *const CLSID,
    filename: *const u16,
    iid: *const CLSID,
    flags: u32,
    renderopt: u32,
    num_fmts: u32,
    adv_flags: *mut u32,
    fmts: *mut FORMATETC,
    sink: *mut IAdviseSink,
    conns: *mut u32,
    client_site: *mut IOleClientSite,
    stg: *mut IStorage,
    obj: *mut LPVOID,
) -> HRESULT {
    let mut hr: HRESULT;
    let mut mon: *mut IMoniker = core::ptr::null_mut();
    let mut data: *mut IDataObject = core::ptr::null_mut();
    let mut unk: *mut IUnknown = core::ptr::null_mut();
    let mut cache: *mut IOleCache = core::ptr::null_mut();
    let mut i: u32;

    TRACE!(
        b"cls %s, %s, iid %s, flags %d, render opts %d, num fmts %d, adv flags %p, fmts %p\n\0".as_ptr() as *const i8,
        debugstr_guid(clsid),
        debugstr_w(filename),
        debugstr_guid(iid),
        flags,
        renderopt,
        num_fmts,
        adv_flags,
        fmts,
    );
    TRACE!(
        b"sink %p, conns %p, client site %p, storage %p, obj %p\n\0".as_ptr() as *const i8,
        sink,
        conns,
        client_site,
        stg,
        obj,
    );
    i = 0;
    while i < num_fmts {
        TRACE!(
            b"\t%d: fmt %s adv flags %d\n\0".as_ptr() as *const i8,
            i,
            debugstr_formatetc(fmts.add(i as usize)),
            *adv_flags.add(i as usize),
        );
        i += 1;
    }

    hr = CreateFileMoniker(filename, &mut mon);
    if FAILED(hr) != 0 {
        return hr;
    }

    hr = BindMoniker(mon, 0, &IID_IDataObject, &mut data as *mut *mut IDataObject as *mut LPVOID);
    IMoniker_Release(mon);
    if FAILED(hr) != 0 {
        return hr;
    }

    hr = get_storage(data, stg, core::ptr::null_mut(), 0);
    if FAILED(hr) != 0 {
        IDataObject_Release(data);
        return hr;
    }

    hr = OleLoad(stg, &IID_IUnknown, client_site, &mut unk as *mut *mut IUnknown as *mut LPVOID);
    if FAILED(hr) != 0 {
        IDataObject_Release(data);
        return hr;
    }

    if renderopt == OLERENDER_FORMAT {
        hr = IUnknown_QueryInterface(unk, &IID_IOleCache, &mut cache as *mut *mut IOleCache as *mut LPVOID);
        if FAILED(hr) != 0 {
            if !unk.is_null() {
                IUnknown_Release(unk);
            }
            IDataObject_Release(data);
            return hr;
        }

        i = 0;
        while i < num_fmts {
            let mut med: STGMEDIUM = core::mem::zeroed();
            let mut dummy_conn: u32 = 0;

            memset(&mut med as *mut STGMEDIUM as *mut core::ffi::c_void, 0, core::mem::size_of::<STGMEDIUM>());
            hr = IDataObject_GetData(data, fmts.add(i as usize), &mut med);
            if FAILED(hr) != 0 {
                if !cache.is_null() {
                    IOleCache_Release(cache);
                }
                if !unk.is_null() {
                    IUnknown_Release(unk);
                }
                IDataObject_Release(data);
                return hr;
            }
            hr = IOleCache_Cache(cache, fmts.add(i as usize), *adv_flags.add(i as usize), &mut dummy_conn);
            if SUCCEEDED(hr) != 0 {
                hr = IOleCache_SetData(cache, fmts.add(i as usize), &mut med, 1);
            }
            if FAILED(hr) != 0 {
                ReleaseStgMedium(&mut med);
                if !cache.is_null() {
                    IOleCache_Release(cache);
                }
                if !unk.is_null() {
                    IUnknown_Release(unk);
                }
                IDataObject_Release(data);
                return hr;
            }
            i += 1;
        }
    }

    hr = IUnknown_QueryInterface(unk, iid, obj);

    if !cache.is_null() {
        IOleCache_Release(cache);
    }
    if !unk.is_null() {
        IUnknown_Release(unk);
    }
    IDataObject_Release(data);
    hr
}

#[no_mangle]
pub unsafe extern "system" fn OleCreateFromFile(
    clsid: *const CLSID,
    filename: *const u16,
    iid: *const CLSID,
    renderopt: u32,
    fmt: *mut FORMATETC,
    client_site: *mut IOleClientSite,
    storage: *mut IStorage,
    obj: *mut LPVOID,
) -> HRESULT {
    let advf: u32 = 1;

    OleCreateFromFileEx(
        clsid,
        filename,
        iid,
        0,
        renderopt,
        if !fmt.is_null() { 1 } else { 0 },
        if !fmt.is_null() { &advf as *const u32 } else { core::ptr::null() },
        fmt,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        client_site,
        storage,
        obj,
    )
}

#[no_mangle]
pub unsafe extern "system" fn OleDuplicateData(
    hSrc: HANDLE,
    cfFormat: CLIPFORMAT,
    uiFlags: UINT,
) -> HANDLE {

    let mut hDst: HANDLE = core::ptr::null_mut();
    let mut uiFlags = uiFlags;

    TRACE!(b"(%p,%x,%x)\n\0".as_ptr(), hSrc, cfFormat, uiFlags);

    if uiFlags == 0 {
        uiFlags = GMEM_MOVEABLE;
    }

    match cfFormat {
        CF_ENHMETAFILE => {
            hDst = CopyEnhMetaFileW(hSrc, core::ptr::null());
        }
        CF_METAFILEPICT => {
            hDst = CopyMetaFileW(hSrc, core::ptr::null());
        }
        CF_PALETTE => {
            let nEntries = GetPaletteEntries(hSrc, 0, 0, core::ptr::null_mut());
            if nEntries == 0 {
                return core::ptr::null_mut();
            }
            let logpalette = HeapAlloc(
                GetProcessHeap(),
                0,
                core::mem::size_of::<LOGPALETTE>() + (nEntries as usize - 1) * core::mem::size_of::<PALETTEENTRY>(),
            ) as *mut LOGPALETTE;
            if logpalette.is_null() {
                return core::ptr::null_mut();
            }
            if GetPaletteEntries(hSrc, 0, nEntries, (*logpalette).palPalEntry.as_mut_ptr()) == 0 {
                HeapFree(GetProcessHeap(), 0, logpalette as LPVOID);
                return core::ptr::null_mut();
            }
            (*logpalette).palVersion = 0x300;
            (*logpalette).palNumEntries = nEntries as WORD;

            hDst = CreatePalette(logpalette);

            HeapFree(GetProcessHeap(), 0, logpalette as LPVOID);
        }
        CF_BITMAP => {
            let mut bm: BITMAP = core::mem::zeroed();
            if GetObjectW(hSrc, core::mem::size_of::<BITMAP>() as c_int, &mut bm as *mut _ as *mut c_void) == 0 {
                return core::ptr::null_mut();
            }
            let size = GetBitmapBits(hSrc, 0, core::ptr::null_mut());
            if size == 0 {
                return core::ptr::null_mut();
            }
            bm.bmBits = HeapAlloc(GetProcessHeap(), 0, size as SIZE_T);
            if bm.bmBits.is_null() {
                return core::ptr::null_mut();
            }
            if GetBitmapBits(hSrc, size, bm.bmBits) != 0 {
                hDst = CreateBitmapIndirect(&bm);
            }
            HeapFree(GetProcessHeap(), 0, bm.bmBits);
        }
        _ => {
            let size = GlobalSize(hSrc);
            let mut pvSrc: LPVOID = core::ptr::null_mut();
            let mut pvDst: LPVOID = core::ptr::null_mut();

            if size == 0 {
                return core::ptr::null_mut();
            }
            hDst = GlobalAlloc(uiFlags, size);
            if hDst.is_null() {
                return core::ptr::null_mut();
            }

            pvSrc = GlobalLock(hSrc);
            if pvSrc.is_null() {
                GlobalFree(hDst);
                return core::ptr::null_mut();
            }
            pvDst = GlobalLock(hDst);
            if pvDst.is_null() {
                GlobalUnlock(hSrc);
                GlobalFree(hDst);
                return core::ptr::null_mut();
            }

            memcpy(pvDst, pvSrc, size);

            GlobalUnlock(hDst);
            GlobalUnlock(hSrc);
        }
    }

    TRACE!(b"returning %p\n\0".as_ptr(), hDst);
    hDst
}
