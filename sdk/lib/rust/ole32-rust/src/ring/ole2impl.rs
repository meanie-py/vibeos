//! ole2impl.c — Ole2 Create functions implementation.
use crate::prelude::*;

// ---------------------------------------------------------------------------
// Locally-declared bits the shared surface does not carry yet.
//
// Deliberately PRIVATE (no `pub`) so nothing leaks through
// `crate::prelude`'s `pub use crate::ring::ole2impl::*` glob and collides with a
// sibling ring module. All hoist candidates — see the report:
//   * iids.rs        : IID_IOleCache, IID_IOleCache2, CLSID_Picture_{Dib,Metafile,EnhMetafile}
//   * ole32_exports  : OleCreateDefaultHandler, WriteClassStg
//   * gdi32.rs       : the palette/bitmap/metafile-copy slice below
//   * win_consts.rs  : CF_PALETTE, CF_ENHMETAFILE
//   * (a BITMAP struct — com-sys has PALETTEENTRY/LOGPALETTE but not tagBITMAP)
// ---------------------------------------------------------------------------

// GUIDs are DECLARED, never defined here (a hand-transcribed byte is a silent
// QueryInterface miss). Same linkage rule as iids.rs: data symbols carry no
// calling convention, so `extern "C"` is exact on every arch.
extern "C" {
    static IID_IOleCache: GUID;
    static IID_IOleCache2: GUID;
    static CLSID_Picture_Dib: GUID;
    static CLSID_Picture_Metafile: GUID;
    static CLSID_Picture_EnhMetafile: GUID;
}

// ole32's own exports that ole32_exports.rs does not list yet. WINAPI → `extern
// "system"`; an `extern "C"` decl would ask the linker for an undecorated name on
// i386 instead of `_WriteClassStg@8`.
extern "system" {
    fn OleCreateDefaultHandler(
        clsid: *const CLSID,
        pUnkOuter: *mut IUnknown,
        riid: REFIID,
        ppvObj: *mut LPVOID,
    ) -> HRESULT;
    fn WriteClassStg(pStg: *mut IStorage, rclsid: *const CLSID) -> HRESULT;
}

/// wingdi.h `tagBITMAP`. com-sys carries PALETTEENTRY/LOGPALETTE but not this one.
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

// gdi32's palette / bitmap / metafile-copy slice. WINAPI → `extern "system"`
// (the raw translation had these `extern "C"`, which is the i386 stdcall-decoration
// bug: `_CreatePalette@4` would go unresolved).
extern "system" {
    fn CopyEnhMetaFileW(hemfSrc: HANDLE, lpszFile: *const u16) -> HANDLE;
    fn CopyMetaFileW(hmfSrc: HANDLE, lpszFile: *const u16) -> HANDLE;
    fn GetPaletteEntries(
        hpal: HANDLE,
        iStart: UINT,
        cEntries: UINT,
        pPalEntries: *mut PALETTEENTRY,
    ) -> UINT;
    fn CreatePalette(plpal: *const LOGPALETTE) -> HANDLE;
    fn GetObjectW(h: HANDLE, c: c_int, pv: LPVOID) -> c_int;
    fn GetBitmapBits(hbit: HANDLE, cb: LONG, lpvBits: LPVOID) -> LONG;
    fn CreateBitmapIndirect(pbm: *const BITMAP) -> HANDLE;
}

/// winuser.h clipboard formats
const CF_ENHMETAFILE: DWORD = 14;
const CF_PALETTE: DWORD = 9;

// ---- file-scope preamble (translated) ----

/// ole2impl.c:48 OleQueryCreateFromData [OLE32.@]
#[no_mangle]
pub unsafe extern "system" fn OleQueryCreateFromData(data: *mut IDataObject) -> HRESULT {
    let mut enum_fmt: *mut IEnumFORMATETC = core::ptr::null_mut();
    let mut fmt: FORMATETC = core::mem::zeroed();
    let mut found_static: BOOL = 0; // FALSE
    let mut hr: HRESULT;

    hr = IDataObject_EnumFormatEtc(data, DATADIR_GET, &mut enum_fmt);

    if FAILED(hr) {
        return hr;
    }

    // do { ... } while (hr == S_OK);
    loop {
        hr = IEnumFORMATETC_Next(enum_fmt, 1, &mut fmt, core::ptr::null_mut());
        if hr == S_OK {
            let cf = fmt.cfFormat as UINT;
            if cf == embedded_object_clipboard_format
                || cf == embed_source_clipboard_format
                || cf == filename_clipboard_format
            {
                IEnumFORMATETC_Release(enum_fmt);
                return S_OK;
            }

            let cf = fmt.cfFormat as DWORD;
            if cf == CF_METAFILEPICT || cf == CF_BITMAP || cf == CF_DIB {
                found_static = 1; // TRUE
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

/// ole2impl.c:84 init_fmtetc
unsafe fn init_fmtetc(fmt: *mut FORMATETC, cf: CLIPFORMAT, tymed: DWORD) {
    (*fmt).cfFormat = cf;
    (*fmt).ptd = core::ptr::null_mut();
    (*fmt).dwAspect = DVASPECT_CONTENT;
    (*fmt).lindex = -1i32;
    (*fmt).tymed = tymed;
}

/// ole2impl.c:100 get_storage
unsafe fn get_storage(
    data: *mut IDataObject,
    stg: *mut IStorage,
    src_cf: *mut UINT,
    other_fmts: BOOL,
) -> HRESULT {
    static fmt_id: [DWORD; 3] = [CF_METAFILEPICT, CF_BITMAP, CF_DIB];
    let mut i: UINT;
    let mut hr: HRESULT;
    let mut fmt: FORMATETC = core::mem::zeroed();
    let mut med: STGMEDIUM = core::mem::zeroed();
    let mut persist: *mut IPersistStorage = core::ptr::null_mut();
    let mut clsid: CLSID = core::mem::zeroed();

    if !src_cf.is_null() {
        *src_cf = 0;
    }

    /* CF_EMBEDEDOBJECT */
    init_fmtetc(
        &mut fmt,
        embedded_object_clipboard_format as CLIPFORMAT,
        TYMED_ISTORAGE,
    );
    med.tymed = TYMED_ISTORAGE;
    med.__bindgen_anon_1.pstg = stg;
    med.pUnkForRelease = core::ptr::null_mut();
    hr = IDataObject_GetDataHere(data, &mut fmt, &mut med);
    if SUCCEEDED(hr) {
        if !src_cf.is_null() {
            *src_cf = embedded_object_clipboard_format;
        }
        return hr;
    }

    /* CF_EMBEDSOURCE */
    init_fmtetc(
        &mut fmt,
        embed_source_clipboard_format as CLIPFORMAT,
        TYMED_ISTORAGE,
    );
    med.tymed = TYMED_ISTORAGE;
    med.__bindgen_anon_1.pstg = stg;
    med.pUnkForRelease = core::ptr::null_mut();
    hr = IDataObject_GetDataHere(data, &mut fmt, &mut med);
    if SUCCEEDED(hr) {
        if !src_cf.is_null() {
            *src_cf = embed_source_clipboard_format;
        }
        return hr;
    }

    if other_fmts != 0 {
        i = 0;
        while (i as usize) < fmt_id.len() {
            init_fmtetc(&mut fmt, fmt_id[i as usize] as CLIPFORMAT, TYMED_ISTORAGE);
            hr = IDataObject_QueryGetData(data, &mut fmt);
            if SUCCEEDED(hr) {
                if !src_cf.is_null() {
                    *src_cf = fmt_id[i as usize] as UINT;
                }
                return hr;
            }
            i += 1;
        }
    }

    /* IPersistStorage */
    hr = IDataObject_QueryInterface(
        data,
        core::ptr::addr_of!(IID_IPersistStorage),
        core::ptr::addr_of_mut!(persist) as *mut LPVOID,
    );
    if FAILED(hr) {
        return hr;
    }

    // C: `goto end` — the single cleanup is IPersistStorage_Release(persist).
    'end: {
        hr = IPersistStorage_GetClassID(persist, &mut clsid);
        if FAILED(hr) {
            break 'end;
        }

        hr = IStorage_SetClass(stg, &clsid);
        if FAILED(hr) {
            break 'end;
        }

        hr = IPersistStorage_Save(persist, stg, 0 /* FALSE */);
        if FAILED(hr) {
            break 'end;
        }

        hr = IPersistStorage_SaveCompleted(persist, core::ptr::null_mut());
    }

    IPersistStorage_Release(persist);

    hr
}

/// ole2impl.c:177 OleCreateFromDataEx [OLE32.@]
#[no_mangle]
pub unsafe extern "system" fn OleCreateFromDataEx(
    data: *mut IDataObject,
    iid: REFIID,
    _flags: DWORD,
    _renderopt: DWORD,
    _num_cache_fmts: ULONG,
    _adv_flags: *mut DWORD,
    _cache_fmts: *mut FORMATETC,
    _sink: *mut IAdviseSink,
    _conns: *mut DWORD,
    client_site: *mut IOleClientSite,
    stg: *mut IStorage,
    obj: *mut LPVOID,
) -> HRESULT {
    let mut hr: HRESULT;
    let mut src_cf: UINT = 0;

    FIXME!(
        "(%p, %s, %08x, %08x, %d, %p, %p, %p, %p, %p, %p, %p): stub\n",
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
        obj
    );

    hr = get_storage(data, stg, &mut src_cf, 1 /* TRUE */);
    if FAILED(hr) {
        return hr;
    }

    hr = OleLoad(stg, iid, client_site, obj);
    if FAILED(hr) {
        return hr;
    }

    /* FIXME: Init cache */

    hr
}

/// ole2impl.c:203 OleCreateFromData [OLE32.@]
#[no_mangle]
pub unsafe extern "system" fn OleCreateFromData(
    data: *mut IDataObject,
    iid: REFIID,
    renderopt: DWORD,
    fmt: *mut FORMATETC,
    client_site: *mut IOleClientSite,
    stg: *mut IStorage,
    obj: *mut LPVOID,
) -> HRESULT {
    let mut advf: DWORD = ADVF_PRIMEFIRST;

    OleCreateFromDataEx(
        data,
        iid,
        0,
        renderopt,
        if !fmt.is_null() { 1 } else { 0 },
        if !fmt.is_null() {
            core::ptr::addr_of_mut!(advf)
        } else {
            core::ptr::null_mut()
        },
        fmt,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        client_site,
        stg,
        obj,
    )
}

/// ole2impl.c:217 OleCreateLinkFromData [OLE32.@]
#[no_mangle]
pub unsafe extern "system" fn OleCreateLinkFromData(
    data: *mut IDataObject,
    iid: REFIID,
    renderopt: DWORD,
    fmt: *mut FORMATETC,
    client_site: *mut IOleClientSite,
    stg: *mut IStorage,
    obj: *mut LPVOID,
) -> HRESULT {
    FIXME!(
        "%p,%s,%08x,%p,%p,%p,%p: semi-stub\n",
        data,
        debugstr_guid(iid),
        renderopt,
        fmt,
        client_site,
        stg,
        obj
    );
    OleCreateFromData(data, iid, renderopt, fmt, client_site, stg, obj)
}

/// ole2impl.c:230 OleCreateStaticFromData [OLE32.@]
#[no_mangle]
pub unsafe extern "system" fn OleCreateStaticFromData(
    data: *mut IDataObject,
    iid: REFIID,
    renderopt: DWORD,
    fmt: *mut FORMATETC,
    client_site: *mut IOleClientSite,
    stg: *mut IStorage,
    obj: *mut LPVOID,
) -> HRESULT {
    let mut hr: HRESULT;
    let clsid: CLSID;
    let mut ole_object: *mut IOleObject = core::ptr::null_mut();
    let mut ole_cache: *mut IOleCache2 = core::ptr::null_mut();
    let mut persist: *mut IPersistStorage = core::ptr::null_mut();
    let mut connection: DWORD = 0;
    let mut stgmedium: STGMEDIUM = core::mem::zeroed();
    let mut ole_typename: LPOLESTR = core::ptr::null_mut();

    TRACE!(
        "(%p, %s, 0x%08x, %p, %p, %p, %p)\n",
        data,
        debugstr_guid(iid),
        renderopt,
        fmt,
        client_site,
        stg,
        obj
    );

    if obj.is_null() || stg.is_null() {
        return E_INVALIDARG;
    }

    if renderopt != OLERENDER_FORMAT {
        FIXME!("semi-stub\n");
        return OleCreateFromData(data, iid, renderopt, fmt, client_site, stg, obj);
    }

    if fmt.is_null() {
        return E_INVALIDARG;
    }

    hr = IDataObject_GetData(data, fmt, core::ptr::addr_of_mut!(stgmedium));
    if FAILED(hr) {
        return hr;
    }

    match (*fmt).cfFormat as DWORD {
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

    // C: every failure below is a `goto end` into the shared cleanup block.
    // (The raw translation emitted a self-breaking 'end block — a labelled block that
    // breaks out of ITSELF, i.e. a no-op that let execution fall straight through to
    // the next COM call with a NULL/unset interface pointer. Real single-exit block
    // here.)
    'end: {
        hr = OleCreateDefaultHandler(
            core::ptr::addr_of!(clsid),
            core::ptr::null_mut(),
            core::ptr::addr_of!(IID_IOleObject),
            core::ptr::addr_of_mut!(ole_object) as *mut LPVOID,
        );
        if FAILED(hr) {
            break 'end;
        }

        if !client_site.is_null() {
            hr = IOleObject_SetClientSite(ole_object, client_site);
            if FAILED(hr) {
                break 'end;
            }
        }

        hr = IOleObject_QueryInterface(
            ole_object,
            core::ptr::addr_of!(IID_IOleCache2),
            core::ptr::addr_of_mut!(ole_cache) as *mut LPVOID,
        );
        if FAILED(hr) {
            break 'end;
        }

        hr = IOleObject_QueryInterface(
            ole_object,
            core::ptr::addr_of!(IID_IPersistStorage),
            core::ptr::addr_of_mut!(persist) as *mut LPVOID,
        );
        if FAILED(hr) {
            break 'end;
        }

        hr = WriteClassStg(stg, core::ptr::addr_of!(clsid));
        if FAILED(hr) {
            break 'end;
        }

        hr = IPersistStorage_InitNew(persist, stg);
        if FAILED(hr) {
            break 'end;
        }

        hr = IOleCache2_Cache(
            ole_cache,
            fmt,
            ADVF_PRIMEFIRST,
            core::ptr::addr_of_mut!(connection),
        );
        if FAILED(hr) {
            break 'end;
        }

        hr = IOleCache2_SetData(ole_cache, fmt, core::ptr::addr_of_mut!(stgmedium), 1 /* TRUE */);
        if FAILED(hr) {
            break 'end;
        }
        stgmedium.tymed = TYMED_NULL;

        hr = IOleObject_GetUserType(
            ole_object,
            USERCLASSTYPE_FULL,
            core::ptr::addr_of_mut!(ole_typename),
        );
        if FAILED(hr) {
            ole_typename = core::ptr::null_mut();
        }
        hr = WriteFmtUserTypeStg(stg, (*fmt).cfFormat, ole_typename);
        CoTaskMemFree(ole_typename as LPVOID);
        if FAILED(hr) {
            break 'end;
        }

        hr = IPersistStorage_Save(persist, stg, 1 /* TRUE */);
        if FAILED(hr) {
            break 'end;
        }

        hr = IPersistStorage_SaveCompleted(persist, core::ptr::null_mut());
        if FAILED(hr) {
            break 'end;
        }

        hr = IOleObject_QueryInterface(ole_object, iid, obj);
    }

    // end:
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

/// ole2impl.c:336 OleCreateFromFileEx [OLE32.@]
#[no_mangle]
pub unsafe extern "system" fn OleCreateFromFileEx(
    _clsid: *const CLSID,
    filename: *const OLECHAR,
    iid: REFIID,
    _flags: DWORD,
    renderopt: DWORD,
    num_fmts: ULONG,
    adv_flags: *mut DWORD,
    fmts: *mut FORMATETC,
    _sink: *mut IAdviseSink,
    _conns: *mut DWORD,
    client_site: *mut IOleClientSite,
    stg: *mut IStorage,
    obj: *mut LPVOID,
) -> HRESULT {
    let mut hr: HRESULT;
    let mut mon: *mut IMoniker = core::ptr::null_mut();
    let mut data: *mut IDataObject = core::ptr::null_mut();
    let mut unk: *mut IUnknown = core::ptr::null_mut();
    let mut cache: *mut IOleCache = core::ptr::null_mut();
    let mut i: ULONG;

    TRACE!(
        "cls %s, %s, iid %s, flags %d, render opts %d, num fmts %d, adv flags %p, fmts %p\n",
        debugstr_guid(clsid),
        debugstr_w(filename as *const c_void),
        debugstr_guid(iid),
        flags,
        renderopt,
        num_fmts,
        adv_flags,
        fmts
    );
    TRACE!(
        "sink %p, conns %p, client site %p, storage %p, obj %p\n",
        sink,
        conns,
        client_site,
        stg,
        obj
    );
    i = 0;
    while i < num_fmts {
        TRACE!(
            "\t%d: fmt %s adv flags %d\n",
            i,
            debugstr_formatetc(fmts.add(i as usize)),
            *adv_flags.add(i as usize)
        );
        i += 1;
    }

    hr = CreateFileMoniker(filename, &mut mon);
    if FAILED(hr) {
        return hr;
    }

    hr = BindMoniker(
        mon,
        0,
        core::ptr::addr_of!(IID_IDataObject),
        core::ptr::addr_of_mut!(data) as *mut LPVOID,
    );
    IMoniker_Release(mon);
    if FAILED(hr) {
        return hr;
    }

    // C: every failure below is a `goto end` into the shared cleanup block.
    'end: {
        hr = get_storage(data, stg, core::ptr::null_mut(), 0 /* FALSE */);
        if FAILED(hr) {
            break 'end;
        }

        hr = OleLoad(
            stg,
            core::ptr::addr_of!(IID_IUnknown),
            client_site,
            core::ptr::addr_of_mut!(unk) as *mut LPVOID,
        );
        if FAILED(hr) {
            break 'end;
        }

        if renderopt == OLERENDER_FORMAT {
            hr = IUnknown_QueryInterface(
                unk,
                core::ptr::addr_of!(IID_IOleCache),
                core::ptr::addr_of_mut!(cache) as *mut LPVOID,
            );
            if FAILED(hr) {
                break 'end;
            }

            i = 0;
            while i < num_fmts {
                let mut med: STGMEDIUM = core::mem::zeroed();
                let mut dummy_conn: DWORD = 0;

                memset(
                    core::ptr::addr_of_mut!(med) as *mut c_void,
                    0,
                    core::mem::size_of::<STGMEDIUM>(),
                );
                hr = IDataObject_GetData(data, fmts.add(i as usize), &mut med);
                if FAILED(hr) {
                    break 'end;
                }
                hr = IOleCache_Cache(
                    cache,
                    fmts.add(i as usize),
                    *adv_flags.add(i as usize),
                    &mut dummy_conn,
                );
                if SUCCEEDED(hr) {
                    hr = IOleCache_SetData(cache, fmts.add(i as usize), &mut med, 1 /* TRUE */);
                }
                if FAILED(hr) {
                    ReleaseStgMedium(&mut med);
                    break 'end;
                }
                i += 1;
            }
        }

        hr = IUnknown_QueryInterface(unk, iid, obj);
    }

    // end:
    if !cache.is_null() {
        IOleCache_Release(cache);
    }
    if !unk.is_null() {
        IUnknown_Release(unk);
    }
    IDataObject_Release(data);
    hr
}

/// ole2impl.c:402 OleCreateFromFile [OLE32.@]
#[no_mangle]
pub unsafe extern "system" fn OleCreateFromFile(
    clsid: *const CLSID,
    filename: *const OLECHAR,
    iid: REFIID,
    renderopt: DWORD,
    fmt: *mut FORMATETC,
    client_site: *mut IOleClientSite,
    storage: *mut IStorage,
    obj: *mut LPVOID,
) -> HRESULT {
    // C: `DWORD advf = ADVF_PRIMEFIRST;` — ADVF_PRIMEFIRST is 2, not 1.
    let mut advf: DWORD = ADVF_PRIMEFIRST;

    OleCreateFromFileEx(
        clsid,
        filename,
        iid,
        0,
        renderopt,
        if !fmt.is_null() { 1 } else { 0 },
        if !fmt.is_null() {
            core::ptr::addr_of_mut!(advf)
        } else {
            core::ptr::null_mut()
        },
        fmt,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        client_site,
        storage,
        obj,
    )
}

/// ole2impl.c:425 OleDuplicateData [OLE32.@]
#[no_mangle]
pub unsafe extern "system" fn OleDuplicateData(
    hSrc: HANDLE,
    cfFormat: CLIPFORMAT,
    uiFlags: UINT,
) -> HANDLE {
    let mut hDst: HANDLE = core::ptr::null_mut();
    let mut uiFlags = uiFlags;

    TRACE!("(%p,%x,%x)\n", hSrc, cfFormat, uiFlags);

    if uiFlags == 0 {
        uiFlags = GMEM_MOVEABLE;
    }

    match cfFormat as DWORD {
        CF_ENHMETAFILE => {
            hDst = CopyEnhMetaFileW(hSrc, core::ptr::null());
        }
        CF_METAFILEPICT => {
            hDst = CopyMetaFileW(hSrc, core::ptr::null());
        }
        CF_PALETTE => {
            let nEntries: UINT = GetPaletteEntries(hSrc, 0, 0, core::ptr::null_mut());
            if nEntries == 0 {
                return core::ptr::null_mut();
            }
            // C: FIELD_OFFSET(LOGPALETTE, palPalEntry[nEntries])
            //  = offsetof(palPalEntry) + nEntries * sizeof(PALETTEENTRY)
            let logpalette = HeapAlloc(
                GetProcessHeap(),
                0,
                (core::mem::offset_of!(LOGPALETTE, palPalEntry)
                    + (nEntries as usize) * core::mem::size_of::<PALETTEENTRY>())
                    as SIZE_T,
            ) as *mut LOGPALETTE;
            if logpalette.is_null() {
                return core::ptr::null_mut();
            }
            if GetPaletteEntries(
                hSrc,
                0,
                nEntries,
                core::ptr::addr_of_mut!((*logpalette).palPalEntry) as *mut PALETTEENTRY,
            ) == 0
            {
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
            if GetObjectW(
                hSrc,
                core::mem::size_of::<BITMAP>() as c_int,
                core::ptr::addr_of_mut!(bm) as LPVOID,
            ) == 0
            {
                return core::ptr::null_mut();
            }
            let size: LONG = GetBitmapBits(hSrc, 0, core::ptr::null_mut());
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
            let size: SIZE_T = GlobalSize(hSrc);
            let pvSrc: LPVOID;
            let pvDst: LPVOID;

            /* allocate space for object */
            if size == 0 {
                return core::ptr::null_mut();
            }
            hDst = GlobalAlloc(uiFlags, size);
            if hDst.is_null() {
                return core::ptr::null_mut();
            }

            /* lock pointers */
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
            /* copy data */
            memcpy(pvDst, pvSrc, size as usize);

            /* cleanup */
            GlobalUnlock(hDst);
            GlobalUnlock(hSrc);
        }
    }

    TRACE!("returning %p\n", hDst);
    hDst
}
