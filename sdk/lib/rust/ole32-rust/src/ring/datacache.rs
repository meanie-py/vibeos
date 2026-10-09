//! OLE 2 data cache — translation of dll/win32/ole32/datacache.c
//!
//! The presentation cache: IOleCache2/IDataObject/IViewObject2/IPersistStorage over
//! cached DIB/metafile presentations held in `\002OlePresXXX` / `CONTENTS` streams.
use crate::prelude::*;
use core::ffi::c_void;

extern "C" {
    /// libwine debug.c:274 — variadic, hence cdecl on every arch. The prelude's
    /// `debug_shims::wine_dbg_sprintf` is a NON-variadic placeholder; `debugstr_formatetc`
    /// is a real (non-static) ole32 export consumed by sibling C TUs, so it must keep the
    /// C's formatting call. Same local re-declaration as ring/clipboard.rs.
    fn wine_dbg_sprintf(format: *const c_char, ...) -> *const c_char;
}

/// `IViewObject2::Draw`'s `pfnContinue` callback. com-sys spells the slot's callback type
/// per arch (`extern "stdcall"` on i386, `extern "C"` elsewhere) — exactly what
/// STDMETHODCALLTYPE lowers to — and rustc will not unify `extern "system"` with either,
/// so it is mirrored here rather than hand-rolled with a single ABI.
pub type PFN_CONTINUE = Option<unsafe extern "system" fn(dwContinue: ULONG_PTR) -> BOOL>;

/// oleidl.idl:509 `tagDISCARDCACHE` — NOT in crate::win_consts yet (owed).
/// NOTE `DISCARDCACHE_SAVEIFDIRTY` is **0**, and `DISCARDCACHE_NOSAVE` is 1.

/// com-sys's vtbl slots are `unsafe extern "stdcall" fn` on i386 and `unsafe extern "C" fn`
/// elsewhere. `extern "system"` lowers to the same ABI but stays a DISTINCT fn type that
/// will not unify with the bindgen slot, so every vtbl method is declared through this
/// macro. (Same shape as ring/hglobalstream.rs and ring/ifs.rs.)
macro_rules! stdmethod {
    ($(
        $(#[$attr:meta])*
        unsafe fn $name:ident ( $($arg:ident : $ty:ty),* $(,)? ) -> $ret:ty $body:block
    )*) => {$(
        $(#[$attr])*
        unsafe extern "system" fn $name($($arg: $ty),*) -> $ret $body
    )*};
}

// ---- file-scope preamble (translated) ----

/// The header of the `\002OlePresXXX` stream. WIRE FORMAT.
#[repr(C)]
pub struct PresentationDataHeader {
    pub tdSize: DWORD,
    pub dvAspect: DVASPECT,
    pub lindex: DWORD,
    pub advf: DWORD,
    pub unknown7: DWORD,
    pub dwObjectExtentX: DWORD,
    pub dwObjectExtentY: DWORD,
    pub dwSize: DWORD,
}

pub type PPresentationDataHeader = *mut PresentationDataHeader;

pub const STREAM_NUMBER_NOT_SET: i32 = -2;
pub const STREAM_NUMBER_CONTENTS: i32 = -1;

#[repr(C)]
pub struct DataCacheEntry {
    /// `struct list entry` — the wine intrusive list node, NOT an NT LIST_ENTRY.
    pub entry: list,
    pub fmtetc: FORMATETC,
    pub stgmedium: STGMEDIUM,
    pub id: DWORD,
    pub dirty: BOOL,
    pub load_stream_num: c_int,
    pub save_stream_num: c_int,
    pub sink_id: DWORD,
    pub advise_flags: DWORD,
}

pub type PDataCacheEntry = *mut DataCacheEntry;

#[repr(C)]
pub struct DataCache {
    pub IUnknown_inner: IUnknown,
    pub IDataObject_iface: IDataObject,
    pub IPersistStorage_iface: IPersistStorage,
    pub IViewObject2_iface: IViewObject2,
    pub IOleCache2_iface: IOleCache2,
    pub IOleCacheControl_iface: IOleCacheControl,
    pub IAdviseSink_iface: IAdviseSink,
    pub ref_: LONG,
    pub outer_unk: *mut IUnknown,
    pub sinkAspects: DWORD,
    pub sinkAdviseFlag: DWORD,
    pub sinkInterface: *mut IAdviseSink,
    pub clsid: CLSID,
    pub clsid_static: BOOL,
    pub presentationStorage: *mut IStorage,
    pub cache_list: list,
    pub last_cache_id: DWORD,
    pub dirty: BOOL,
    pub running_object: *mut IDataObject,
}

pub type PDataCache = *mut DataCache;

pub static CONTENTS: [WCHAR; 9] = [67u16, 79u16, 78u16, 84u16, 69u16, 78u16, 84u16, 83u16, 0u16];

/// The placeable-metafile header that prefixes a CONTENTS-stream WMF.
/// `#include <pshpack2.h>` in the C — sizeof is **22**, not 28. The 2-byte packing is
/// WIRE FORMAT: a naturally-aligned version would pad `reserved`/`checksum` and corrupt
/// every saved metafile.
#[repr(C, packed(2))]
#[derive(Copy, Clone)]
pub struct meta_placeable {
    pub key: DWORD,
    pub hwmf: WORD,
    pub bounding_box: [WORD; 4],
    pub inch: WORD,
    pub reserved: DWORD,
    pub checksum: WORD,
}

pub type Pmeta_placeable = *mut meta_placeable;

unsafe fn impl_from_IDataObject(iface: *mut IDataObject) -> *mut DataCache {
    CONTAINING_RECORD!(iface, DataCache, IDataObject_iface)
}

unsafe fn impl_from_IUnknown(iface: *mut IUnknown) -> *mut DataCache {
    CONTAINING_RECORD!(iface, DataCache, IUnknown_inner)
}

unsafe fn impl_from_IPersistStorage(iface: *mut IPersistStorage) -> *mut DataCache {
    CONTAINING_RECORD!(iface, DataCache, IPersistStorage_iface)
}

unsafe fn impl_from_IViewObject2(iface: *mut IViewObject2) -> *mut DataCache {
    CONTAINING_RECORD!(iface, DataCache, IViewObject2_iface)
}

unsafe fn impl_from_IOleCache2(iface: *mut IOleCache2) -> *mut DataCache {
    CONTAINING_RECORD!(iface, DataCache, IOleCache2_iface)
}

unsafe fn impl_from_IOleCacheControl(iface: *mut IOleCacheControl) -> *mut DataCache {
    CONTAINING_RECORD!(iface, DataCache, IOleCacheControl_iface)
}

unsafe fn impl_from_IAdviseSink(iface: *mut IAdviseSink) -> *mut DataCache {
    CONTAINING_RECORD!(iface, DataCache, IAdviseSink_iface)
}

#[no_mangle]
pub unsafe extern "C" fn debugstr_formatetc(formatetc: *const FORMATETC) -> *const c_char {
    wine_dbg_sprintf(
        b"{ cfFormat = 0x%x, ptd = %p, dwAspect = %d, lindex = %d, tymed = %d }\0".as_ptr()
            as *const c_char,
        (*formatetc).cfFormat as c_uint,
        (*formatetc).ptd,
        (*formatetc).dwAspect,
        (*formatetc).lindex,
        (*formatetc).tymed,
    )
}

/// Size of the bitmap info structure including the colour table.
unsafe fn bitmap_info_size(info: *const BITMAPINFO, coloruse: WORD) -> c_int {
    let mut colors: c_uint;
    let size: c_uint;
    let mut masks: c_uint = 0;

    if (*info).bmiHeader.biSize == core::mem::size_of::<BITMAPCOREHEADER>() as DWORD {
        let core_hdr = info as *const BITMAPCOREHEADER;
        colors = if (*core_hdr).bcBitCount <= 8 {
            1u32 << (*core_hdr).bcBitCount
        } else {
            0
        };
        return (core::mem::size_of::<BITMAPCOREHEADER>() as c_uint
            + colors
                * if coloruse == DIB_RGB_COLORS as WORD {
                    core::mem::size_of::<RGBTRIPLE>() as c_uint
                } else {
                    core::mem::size_of::<WORD>() as c_uint
                }) as c_int;
    } else {
        colors = (*info).bmiHeader.biClrUsed;
        if colors > 256 {
            colors = 256;
        }
        if colors == 0 && (*info).bmiHeader.biBitCount <= 8 {
            colors = 1u32 << (*info).bmiHeader.biBitCount;
        }
        if (*info).bmiHeader.biCompression == BI_BITFIELDS {
            masks = 3;
        }
        let floor =
            core::mem::size_of::<BITMAPINFOHEADER>() as c_uint + masks * core::mem::size_of::<DWORD>() as c_uint;
        size = max((*info).bmiHeader.biSize, floor);
        return (size
            + colors
                * if coloruse == DIB_RGB_COLORS as WORD {
                    core::mem::size_of::<RGBQUAD>() as c_uint
                } else {
                    core::mem::size_of::<WORD>() as c_uint
                }) as c_int;
    }
}

unsafe fn DataCacheEntry_Destroy(cache: *mut DataCache, cache_entry: *mut DataCacheEntry) {
    list_remove(addr_of_mut!((*cache_entry).entry));
    CoTaskMemFree((*cache_entry).fmtetc.ptd as LPVOID);
    ReleaseStgMedium(addr_of_mut!((*cache_entry).stgmedium));
    if (*cache_entry).sink_id != 0 {
        IDataObject_DUnadvise((*cache).running_object, (*cache_entry).sink_id);
    }

    HeapFree(GetProcessHeap(), 0, cache_entry as LPVOID);
}

unsafe fn DataCache_Destroy(ptrToDestroy: *mut DataCache) {
    TRACE!(b"()\n\0".as_ptr() as *const c_char);

    if !(*ptrToDestroy).sinkInterface.is_null() {
        IAdviseSink_Release((*ptrToDestroy).sinkInterface);
        (*ptrToDestroy).sinkInterface = core::ptr::null_mut();
    }

    LIST_FOR_EACH_ENTRY_SAFE!(
        cache_entry,
        addr_of_mut!((*ptrToDestroy).cache_list),
        DataCacheEntry,
        entry,
        {
            DataCacheEntry_Destroy(ptrToDestroy, cache_entry);
        }
    );

    if !(*ptrToDestroy).presentationStorage.is_null() {
        IStorage_Release((*ptrToDestroy).presentationStorage);
        (*ptrToDestroy).presentationStorage = core::ptr::null_mut();
    }

    HeapFree(GetProcessHeap(), 0, ptrToDestroy as LPVOID);
}

unsafe fn DataCache_GetEntryForFormatEtc(
    This: *mut DataCache,
    formatetc: *const FORMATETC,
) -> *mut DataCacheEntry {
    let mut fmt: FORMATETC = *formatetc;

    if fmt.cfFormat as DWORD == CF_BITMAP {
        fmt.cfFormat = CF_DIB as CLIPFORMAT;
        fmt.tymed = TYMED_HGLOBAL;
    }

    LIST_FOR_EACH_ENTRY!(
        cache_entry,
        addr_of_mut!((*This).cache_list),
        DataCacheEntry,
        entry,
        {
            /* FIXME: also compare DVTARGETDEVICEs */
            if (fmt.cfFormat == (*cache_entry).fmtetc.cfFormat)
                && (fmt.dwAspect == (*cache_entry).fmtetc.dwAspect)
                && (fmt.lindex == (*cache_entry).fmtetc.lindex)
                && ((fmt.tymed == (*cache_entry).fmtetc.tymed)
                    || (*cache_entry).fmtetc.cfFormat == 0)
            {
                return cache_entry;
            }
        }
    );

    core::ptr::null_mut()
}

/// The cache entry associated with a static CLSID: first in the list, connection id == 1.
unsafe fn get_static_entry(
    cache: *mut DataCache,
    cache_entry: *mut *mut DataCacheEntry,
) -> HRESULT {
    let head: *mut list = list_head(addr_of_mut!((*cache).cache_list));
    let mut hr: HRESULT = E_FAIL;

    *cache_entry = core::ptr::null_mut();

    if !head.is_null() {
        let entry: *mut DataCacheEntry =
            list_entry::<DataCacheEntry>(head, core::mem::offset_of!(DataCacheEntry, entry));
        if (*entry).id == 1 {
            *cache_entry = entry;
            hr = S_OK;
        }
    }

    hr
}

unsafe fn check_valid_formatetc(fmt: *const FORMATETC) -> HRESULT {
    /* DVASPECT_ICON must be CF_METAFILEPICT */
    if (*fmt).dwAspect == DVASPECT_ICON && (*fmt).cfFormat as DWORD != CF_METAFILEPICT {
        return DV_E_FORMATETC;
    }

    if (*fmt).cfFormat == 0
        || ((*fmt).cfFormat as DWORD == CF_METAFILEPICT && (*fmt).tymed == TYMED_MFPICT)
        || ((*fmt).cfFormat as DWORD == CF_BITMAP && (*fmt).tymed == TYMED_GDI)
        || ((*fmt).cfFormat as DWORD == CF_DIB && (*fmt).tymed == TYMED_HGLOBAL)
        || ((*fmt).cfFormat as DWORD == CF_ENHMETAFILE && (*fmt).tymed == TYMED_ENHMF)
    {
        S_OK
    } else if (*fmt).tymed == TYMED_HGLOBAL {
        CACHE_S_FORMATETC_NOTSUPPORTED
    } else {
        WARN!(
            b"invalid clipformat/tymed combination: %d/%d\n\0".as_ptr() as *const c_char,
            (*fmt).cfFormat,
            (*fmt).tymed,
        );
        DV_E_TYMED
    }
}

unsafe fn init_cache_entry(
    entry: *mut DataCacheEntry,
    fmt: *const FORMATETC,
    advf: DWORD,
    id: DWORD,
) -> BOOL {
    let hr: HRESULT = copy_formatetc(addr_of_mut!((*entry).fmtetc), fmt);
    if FAILED(hr) {
        return 0;
    }

    (*entry).stgmedium.tymed = TYMED_NULL;
    (*entry).stgmedium.pUnkForRelease = core::ptr::null_mut();
    (*entry).id = id;
    (*entry).dirty = 1;
    (*entry).load_stream_num = STREAM_NUMBER_NOT_SET;
    (*entry).save_stream_num = STREAM_NUMBER_NOT_SET;
    (*entry).sink_id = 0;
    (*entry).advise_flags = advf;

    1
}

unsafe fn DataCache_CreateEntry(
    This: *mut DataCache,
    formatetc: *const FORMATETC,
    advf: DWORD,
    automatic: BOOL,
    cache_entry: *mut *mut DataCacheEntry,
) -> HRESULT {
    let id: DWORD = if automatic != 0 {
        1
    } else {
        (*This).last_cache_id
    };

    let hr: HRESULT = check_valid_formatetc(formatetc);
    if FAILED(hr) {
        return hr;
    }
    if hr == CACHE_S_FORMATETC_NOTSUPPORTED {
        TRACE!(
            b"creating unsupported format %d\n\0".as_ptr() as *const c_char,
            (*formatetc).cfFormat
        );
    }

    let entry: *mut DataCacheEntry = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<DataCacheEntry>() as SIZE_T,
    ) as *mut DataCacheEntry;
    if entry.is_null() {
        return E_OUTOFMEMORY;
    }

    // C: `if (!init_cache_entry(...)) goto fail;` — fail: HeapFree + E_OUTOFMEMORY.
    if init_cache_entry(entry, formatetc, advf, id) == 0 {
        HeapFree(GetProcessHeap(), 0, entry as LPVOID);
        return E_OUTOFMEMORY;
    }

    if automatic != 0 {
        list_add_head(
            addr_of_mut!((*This).cache_list),
            addr_of_mut!((*entry).entry),
        );
    } else {
        list_add_tail(
            addr_of_mut!((*This).cache_list),
            addr_of_mut!((*entry).entry),
        );
        (*This).last_cache_id += 1;
    }

    if !cache_entry.is_null() {
        *cache_entry = entry;
    }
    hr
}

/// Fire an OnViewChange notification to the advise sink registered with the datacache.
unsafe fn DataCache_FireOnViewChange(this: *mut DataCache, aspect: DWORD, lindex: LONG) {
    TRACE!(
        b"(%p, %x, %d)\n\0".as_ptr() as *const c_char,
        this,
        aspect,
        lindex
    );

    if ((*this).sinkAspects & aspect) != 0 {
        if !(*this).sinkInterface.is_null() {
            IAdviseSink_OnViewChange((*this).sinkInterface, aspect, lindex);

            /* Some sinks want to be unregistered after the first notification. */
            if ((*this).sinkAdviseFlag & ADVF_ONLYONCE) != 0 {
                IAdviseSink_Release((*this).sinkInterface);

                (*this).sinkInterface = core::ptr::null_mut();
                (*this).sinkAspects = 0;
                (*this).sinkAdviseFlag = 0;
            }
        }
    }
}

unsafe fn read_clipformat(stream: *mut IStream, clipformat: *mut CLIPFORMAT) -> HRESULT {
    let mut length: DWORD = 0;
    let mut read: ULONG = 0;

    *clipformat = 0;

    let mut hr = IStream_Read(
        stream,
        addr_of_mut!(length) as *mut c_void,
        core::mem::size_of::<DWORD>() as ULONG,
        addr_of_mut!(read),
    );
    if hr != S_OK || read != core::mem::size_of::<DWORD>() as ULONG {
        return DV_E_CLIPFORMAT;
    }
    if length == 0 {
        /* No clipboard format present */
        return S_OK;
    }
    if length == 0xffff_ffff {
        let mut cf: DWORD = 0;
        hr = IStream_Read(
            stream,
            addr_of_mut!(cf) as *mut c_void,
            core::mem::size_of::<DWORD>() as ULONG,
            addr_of_mut!(read),
        );
        if hr != S_OK || read != core::mem::size_of::<DWORD>() as ULONG {
            return DV_E_CLIPFORMAT;
        }
        *clipformat = cf as CLIPFORMAT;
    } else {
        let format_name = HeapAlloc(GetProcessHeap(), 0, length as SIZE_T) as *mut c_char;
        if format_name.is_null() {
            return E_OUTOFMEMORY;
        }
        hr = IStream_Read(
            stream,
            format_name as *mut c_void,
            length,
            addr_of_mut!(read),
        );
        if hr != S_OK || read != length || *format_name.add((length - 1) as usize) != 0 {
            HeapFree(GetProcessHeap(), 0, format_name as LPVOID);
            return DV_E_CLIPFORMAT;
        }
        *clipformat = RegisterClipboardFormatA(format_name) as CLIPFORMAT;
        HeapFree(GetProcessHeap(), 0, format_name as LPVOID);
    }
    S_OK
}

unsafe fn write_clipformat(stream: *mut IStream, clipformat: CLIPFORMAT) -> HRESULT {
    let mut length: DWORD;
    let mut hr: HRESULT;
    let mut format_name: [c_char; 256] = [0; 256];

    if clipformat == 0 {
        length = 0;
    } else if (clipformat as DWORD) < 0xc000 {
        length = 0xffff_ffff; /* C: `length = -1` on a DWORD */
    } else {
        length = GetClipboardFormatNameA(
            clipformat as UINT,
            format_name.as_mut_ptr(),
            format_name.len() as c_int,
        ) as DWORD;
        /* If there is a clipboard format name, we need to include its terminating \0 */
        if length != 0 {
            length += 1;
        }
    }

    hr = IStream_Write(
        stream,
        addr_of!(length) as *const c_void,
        core::mem::size_of::<DWORD>() as ULONG,
        core::ptr::null_mut(),
    );
    if FAILED(hr) || clipformat == 0 {
        return hr;
    }

    if (clipformat as DWORD) < 0xc000 {
        let cf: DWORD = clipformat as DWORD;
        hr = IStream_Write(
            stream,
            addr_of!(cf) as *const c_void,
            core::mem::size_of::<DWORD>() as ULONG,
            core::ptr::null_mut(),
        );
    } else {
        hr = IStream_Write(
            stream,
            format_name.as_ptr() as *const c_void,
            length,
            core::ptr::null_mut(),
        );
    }

    hr
}

unsafe fn open_pres_stream(
    stg: *mut IStorage,
    stream_number: c_int,
    stm: *mut *mut IStream,
) -> HRESULT {
    // C builds the name unconditionally with int arithmetic, then overrides it for CONTENTS.
    let pres: [WCHAR; 12] = [
        2,
        'O' as WCHAR,
        'l' as WCHAR,
        'e' as WCHAR,
        'P' as WCHAR,
        'r' as WCHAR,
        'e' as WCHAR,
        's' as WCHAR,
        ('0' as c_int + (stream_number / 100) % 10) as WCHAR,
        ('0' as c_int + (stream_number / 10) % 10) as WCHAR,
        ('0' as c_int + stream_number % 10) as WCHAR,
        0,
    ];
    let mut name: *const WCHAR = pres.as_ptr();

    if stream_number == STREAM_NUMBER_NOT_SET {
        return E_FAIL;
    }
    if stream_number == STREAM_NUMBER_CONTENTS {
        name = CONTENTS.as_ptr();
    }

    IStorage_OpenStream(
        stg,
        name,
        core::ptr::null_mut(),
        STGM_READ | STGM_SHARE_EXCLUSIVE,
        0,
        stm,
    )
}

unsafe fn synthesize_emf(data: HMETAFILEPICT, med: *mut STGMEDIUM) -> HRESULT {
    let mut hr: HRESULT = E_FAIL;

    let pict: *mut METAFILEPICT = GlobalLock(data) as *mut METAFILEPICT;
    if pict.is_null() {
        return hr;
    }

    let size: UINT = GetMetaFileBitsEx((*pict).hMF, 0, core::ptr::null_mut());
    let bits: LPVOID = HeapAlloc(GetProcessHeap(), 0, size as SIZE_T);
    if !bits.is_null() {
        GetMetaFileBitsEx((*pict).hMF, size, bits);
        (*med).__bindgen_anon_1.hEnhMetaFile =
            SetWinMetaFileBits(size, bits as *const BYTE, core::ptr::null_mut(), pict);
        HeapFree(GetProcessHeap(), 0, bits);
        (*med).tymed = TYMED_ENHMF;
        (*med).pUnkForRelease = core::ptr::null_mut();
        hr = S_OK;
    }

    GlobalUnlock(data);
    hr
}

unsafe fn load_mf_pict(cache_entry: *mut DataCacheEntry, stm: *mut IStream) -> HRESULT {
    let mut stat: STATSTG = core::mem::zeroed();
    let mut current_pos: ULARGE_INTEGER = core::mem::zeroed();
    let mut header: PresentationDataHeader = core::mem::zeroed();
    let mut clipformat: CLIPFORMAT = 0;
    let offset_zero: LARGE_INTEGER = core::mem::zeroed();
    let mut read: ULONG = 0;
    let mut mf_place: meta_placeable = core::mem::zeroed();

    let mut hr = IStream_Stat(stm, addr_of_mut!(stat), STATFLAG_NONAME);
    if FAILED(hr) {
        return hr;
    }

    if (*cache_entry).load_stream_num != STREAM_NUMBER_CONTENTS {
        hr = read_clipformat(stm, addr_of_mut!(clipformat));
        if hr != S_OK {
            return hr;
        }
        hr = IStream_Read(
            stm,
            addr_of_mut!(header) as *mut c_void,
            core::mem::size_of::<PresentationDataHeader>() as ULONG,
            addr_of_mut!(read),
        );
        if hr != S_OK {
            return hr;
        }
    } else {
        hr = IStream_Read(
            stm,
            addr_of_mut!(mf_place) as *mut c_void,
            core::mem::size_of::<meta_placeable>() as ULONG,
            addr_of_mut!(read),
        );
        if hr != S_OK {
            return hr;
        }
    }

    hr = IStream_Seek(
        stm,
        offset_zero,
        STREAM_SEEK_CUR,
        addr_of_mut!(current_pos),
    );
    if FAILED(hr) {
        return hr;
    }
    stat.cbSize.QuadPart -= current_pos.QuadPart;

    let hmfpict: HGLOBAL = GlobalAlloc(
        GMEM_MOVEABLE,
        core::mem::size_of::<METAFILEPICT>() as SIZE_T,
    );
    if hmfpict.is_null() {
        return E_OUTOFMEMORY;
    }
    let mfpict: *mut METAFILEPICT = GlobalLock(hmfpict) as *mut METAFILEPICT;

    let bits: LPVOID = HeapAlloc(GetProcessHeap(), 0, stat.cbSize.u.LowPart as SIZE_T);
    if bits.is_null() {
        GlobalFree(hmfpict);
        return E_OUTOFMEMORY;
    }

    hr = IStream_Read(stm, bits, stat.cbSize.u.LowPart, addr_of_mut!(read));

    if SUCCEEDED(hr) {
        (*mfpict).mm = MM_ANISOTROPIC;
        /* FIXME: get this from the stream */
        if (*cache_entry).load_stream_num != STREAM_NUMBER_CONTENTS {
            (*mfpict).xExt = header.dwObjectExtentX as LONG;
            (*mfpict).yExt = header.dwObjectExtentY as LONG;
        } else {
            // C: WORDs promote to int BEFORE the subtraction — do the widening first.
            let bb = mf_place.bounding_box;
            let inch = mf_place.inch as c_int;
            (*mfpict).xExt = ((bb[2] as c_int - bb[0] as c_int) * 2540) / inch;
            (*mfpict).yExt = ((bb[3] as c_int - bb[1] as c_int) * 2540) / inch;
        }
        (*mfpict).hMF = SetMetaFileBitsEx(stat.cbSize.u.LowPart, bits as *const BYTE);
        if (*mfpict).hMF.is_null() {
            hr = E_FAIL;
        }
    }

    GlobalUnlock(hmfpict);
    if SUCCEEDED(hr) {
        (*cache_entry).stgmedium.tymed = TYMED_MFPICT;
        (*cache_entry).stgmedium.__bindgen_anon_1.hMetaFilePict = hmfpict;
    } else {
        GlobalFree(hmfpict);
    }

    HeapFree(GetProcessHeap(), 0, bits);

    hr
}

unsafe fn load_dib(cache_entry: *mut DataCacheEntry, stm: *mut IStream) -> HRESULT {
    let mut stat: STATSTG = core::mem::zeroed();
    let mut read: ULONG = 0;
    let info_size: ULONG;
    let bi_size: ULONG;
    let mut file: BITMAPFILEHEADER = core::mem::zeroed();
    let info: *mut BITMAPINFOHEADER;
    let mut cf: CLIPFORMAT = 0;
    let mut pres: PresentationDataHeader = core::mem::zeroed();
    let mut current_pos: ULARGE_INTEGER = core::mem::zeroed();
    let offset_zero: LARGE_INTEGER = core::mem::zeroed();

    let mut hr = IStream_Stat(stm, addr_of_mut!(stat), STATFLAG_NONAME);
    if FAILED(hr) {
        return hr;
    }

    if (*cache_entry).load_stream_num != STREAM_NUMBER_CONTENTS {
        hr = read_clipformat(stm, addr_of_mut!(cf));
        if hr != S_OK {
            return hr;
        }
        hr = IStream_Read(
            stm,
            addr_of_mut!(pres) as *mut c_void,
            core::mem::size_of::<PresentationDataHeader>() as ULONG,
            addr_of_mut!(read),
        );
        if hr != S_OK {
            return hr;
        }
    } else {
        hr = IStream_Read(
            stm,
            addr_of_mut!(file) as *mut c_void,
            core::mem::size_of::<BITMAPFILEHEADER>() as ULONG,
            addr_of_mut!(read),
        );
        if hr != S_OK {
            return hr;
        }
    }

    hr = IStream_Seek(
        stm,
        offset_zero,
        STREAM_SEEK_CUR,
        addr_of_mut!(current_pos),
    );
    if FAILED(hr) {
        return hr;
    }
    stat.cbSize.QuadPart -= current_pos.QuadPart;

    let hglobal: HGLOBAL = GlobalAlloc(GMEM_MOVEABLE, stat.cbSize.u.LowPart as SIZE_T);
    if hglobal.is_null() {
        return E_OUTOFMEMORY;
    }
    let dib: *mut BYTE = GlobalLock(hglobal) as *mut BYTE;

    // C uses `goto fail` for every error below: GlobalUnlock + GlobalFree + return hr.
    macro_rules! fail {
        () => {{
            GlobalUnlock(hglobal);
            GlobalFree(hglobal);
            return hr
        }};
    }

    /* read first DWORD of BITMAPINFOHEADER */
    hr = IStream_Read(
        stm,
        dib as *mut c_void,
        core::mem::size_of::<DWORD>() as ULONG,
        addr_of_mut!(read),
    );
    if hr != S_OK {
        fail!();
    }
    bi_size = *(dib as *const DWORD);
    if stat.cbSize.QuadPart < bi_size as u64 {
        fail!();
    }

    /* read rest of BITMAPINFOHEADER */
    hr = IStream_Read(
        stm,
        dib.add(core::mem::size_of::<DWORD>()) as *mut c_void,
        bi_size - core::mem::size_of::<DWORD>() as ULONG,
        addr_of_mut!(read),
    );
    if hr != S_OK {
        fail!();
    }

    info_size = bitmap_info_size(dib as *const BITMAPINFO, DIB_RGB_COLORS as WORD) as ULONG;
    if stat.cbSize.QuadPart < info_size as u64 {
        fail!();
    }
    if info_size > bi_size {
        hr = IStream_Read(
            stm,
            dib.add(bi_size as usize) as *mut c_void,
            info_size - bi_size,
            addr_of_mut!(read),
        );
        if hr != S_OK {
            fail!();
        }
    }
    stat.cbSize.QuadPart -= info_size as u64;

    /* set Stream pointer to beginning of bitmap bits */
    if (*cache_entry).load_stream_num == STREAM_NUMBER_CONTENTS && file.bfOffBits != 0 {
        let mut skip: LARGE_INTEGER = core::mem::zeroed();

        // C: DWORD arithmetic, then widened into the LONGLONG.
        skip.QuadPart = (file.bfOffBits)
            .wrapping_sub(core::mem::size_of::<BITMAPFILEHEADER>() as DWORD)
            .wrapping_sub(info_size) as i64;
        if stat.cbSize.QuadPart < skip.QuadPart as u64 {
            fail!();
        }
        hr = IStream_Seek(stm, skip, STREAM_SEEK_CUR, core::ptr::null_mut());
        if hr != S_OK {
            fail!();
        }
        stat.cbSize.QuadPart -= skip.QuadPart as u64;
    }

    hr = IStream_Read(
        stm,
        dib.add(info_size as usize) as *mut c_void,
        stat.cbSize.u.LowPart,
        addr_of_mut!(read),
    );
    if hr != S_OK {
        fail!();
    }

    if bi_size >= core::mem::size_of::<BITMAPINFOHEADER>() as ULONG {
        info = dib as *mut BITMAPINFOHEADER;
        if (*info).biXPelsPerMeter == 0 || (*info).biYPelsPerMeter == 0 {
            let hdc = GetDC(core::ptr::null_mut());
            (*info).biXPelsPerMeter = MulDiv(GetDeviceCaps(hdc, LOGPIXELSX), 10000, 254);
            (*info).biYPelsPerMeter = MulDiv(GetDeviceCaps(hdc, LOGPIXELSY), 10000, 254);
            ReleaseDC(core::ptr::null_mut(), hdc);
        }
    }

    GlobalUnlock(hglobal);

    (*cache_entry).stgmedium.tymed = TYMED_HGLOBAL;
    (*cache_entry).stgmedium.__bindgen_anon_1.hGlobal = hglobal;

    hr
}

unsafe fn load_emf(cache_entry: *mut DataCacheEntry, stm: *mut IStream) -> HRESULT {
    let mut hr: HRESULT;

    if (*cache_entry).load_stream_num != STREAM_NUMBER_CONTENTS {
        let mut stgmed: STGMEDIUM = core::mem::zeroed();

        hr = load_mf_pict(cache_entry, stm);
        if SUCCEEDED(hr) {
            hr = synthesize_emf(
                (*cache_entry).stgmedium.__bindgen_anon_1.hMetaFilePict,
                addr_of_mut!(stgmed),
            );
            ReleaseStgMedium(addr_of_mut!((*cache_entry).stgmedium));
        }
        if SUCCEEDED(hr) {
            (*cache_entry).stgmedium = stgmed;
        }
    } else {
        let mut stat: STATSTG = core::mem::zeroed();
        let mut read: ULONG = 0;
        let size_bits: ULONG;

        hr = IStream_Stat(stm, addr_of_mut!(stat), STATFLAG_NONAME);

        if SUCCEEDED(hr) {
            let data: *mut BYTE =
                HeapAlloc(GetProcessHeap(), 0, stat.cbSize.u.LowPart as SIZE_T) as *mut BYTE;
            if data.is_null() {
                return E_OUTOFMEMORY;
            }

            hr = IStream_Read(
                stm,
                data as *mut c_void,
                stat.cbSize.u.LowPart,
                addr_of_mut!(read),
            );
            if hr != S_OK {
                HeapFree(GetProcessHeap(), 0, data as LPVOID);
                return hr;
            }

            if read
                <= core::mem::size_of::<DWORD>() as ULONG
                    + core::mem::size_of::<ENHMETAHEADER>() as ULONG
            {
                HeapFree(GetProcessHeap(), 0, data as LPVOID);
                return E_FAIL;
            }
            size_bits = read
                - core::mem::size_of::<DWORD>() as ULONG
                - core::mem::size_of::<ENHMETAHEADER>() as ULONG;
            (*cache_entry).stgmedium.__bindgen_anon_1.hEnhMetaFile =
                SetEnhMetaFileBits(size_bits, data.add((read - size_bits) as usize));
            (*cache_entry).stgmedium.tymed = TYMED_ENHMF;
            (*cache_entry).stgmedium.pUnkForRelease = core::ptr::null_mut();

            HeapFree(GetProcessHeap(), 0, data as LPVOID);
        }
    }

    hr
}

/// Read the requested presentation into the cache entry.
unsafe fn DataCacheEntry_LoadData(
    cache_entry: *mut DataCacheEntry,
    stg: *mut IStorage,
) -> HRESULT {
    let mut hr: HRESULT;
    let mut stm: *mut IStream = core::ptr::null_mut();

    if stg.is_null() {
        return OLE_E_BLANK;
    }

    hr = open_pres_stream(stg, (*cache_entry).load_stream_num, addr_of_mut!(stm));
    if FAILED(hr) {
        return hr;
    }

    match (*cache_entry).fmtetc.cfFormat as DWORD {
        CF_METAFILEPICT => {
            hr = load_mf_pict(cache_entry, stm);
        }
        CF_DIB => {
            hr = load_dib(cache_entry, stm);
        }
        CF_ENHMETAFILE => {
            hr = load_emf(cache_entry, stm);
        }
        _ => {
            FIXME!(
                b"Unimplemented clip format %x\n\0".as_ptr() as *const c_char,
                (*cache_entry).fmtetc.cfFormat as core::ffi::c_uint
            );
            hr = E_NOTIMPL;
        }
    }

    IStream_Release(stm);
    hr
}

unsafe fn init_stream_header(entry: *mut DataCacheEntry, header: *mut PresentationDataHeader) {
    if !(*entry).fmtetc.ptd.is_null() {
        FIXME!(b"ptd not serialized\n\0".as_ptr() as *const c_char);
    }
    (*header).tdSize = core::mem::size_of::<DWORD>() as DWORD;
    (*header).dvAspect = (*entry).fmtetc.dwAspect as DVASPECT;
    (*header).lindex = (*entry).fmtetc.lindex as DWORD;
    (*header).advf = (*entry).advise_flags;
    (*header).unknown7 = 0;
    (*header).dwObjectExtentX = 0;
    (*header).dwObjectExtentY = 0;
    (*header).dwSize = 0;
}

unsafe fn save_dib(entry: *mut DataCacheEntry, contents: BOOL, stream: *mut IStream) -> HRESULT {
    let mut hr: HRESULT = S_OK;
    let mut data_size: c_int = 0;
    let mut bmi: *mut BITMAPINFO = core::ptr::null_mut();

    if (*entry).stgmedium.tymed != TYMED_NULL {
        data_size = GlobalSize((*entry).stgmedium.__bindgen_anon_1.hGlobal) as c_int;
        bmi = GlobalLock((*entry).stgmedium.__bindgen_anon_1.hGlobal) as *mut BITMAPINFO;
    }

    if contents == 0 {
        let mut header: PresentationDataHeader = core::mem::zeroed();

        init_stream_header(entry, addr_of_mut!(header));
        hr = write_clipformat(stream, (*entry).fmtetc.cfFormat);
        if FAILED(hr) {
            // C: `goto end;` — the `end:` label only does the GlobalUnlock below.
            if !bmi.is_null() {
                GlobalUnlock((*entry).stgmedium.__bindgen_anon_1.hGlobal);
            }
            return hr;
        }
        if data_size != 0 {
            header.dwSize = data_size as DWORD;
            /* Size in units of 0.01mm (ie. MM_HIMETRIC) */
            if (*bmi).bmiHeader.biXPelsPerMeter != 0 && (*bmi).bmiHeader.biYPelsPerMeter != 0 {
                header.dwObjectExtentX = MulDiv(
                    (*bmi).bmiHeader.biWidth,
                    100000,
                    (*bmi).bmiHeader.biXPelsPerMeter,
                ) as DWORD;
                header.dwObjectExtentY = MulDiv(
                    (*bmi).bmiHeader.biHeight,
                    100000,
                    (*bmi).bmiHeader.biYPelsPerMeter,
                ) as DWORD;
            } else {
                let hdc: HDC = GetDC(core::ptr::null_mut());
                header.dwObjectExtentX = MulDiv(
                    (*bmi).bmiHeader.biWidth,
                    2540,
                    GetDeviceCaps(hdc, LOGPIXELSX),
                ) as DWORD;
                header.dwObjectExtentY = MulDiv(
                    (*bmi).bmiHeader.biHeight,
                    2540,
                    GetDeviceCaps(hdc, LOGPIXELSY),
                ) as DWORD;
                ReleaseDC(core::ptr::null_mut(), hdc);
            }
        }
        hr = IStream_Write(
            stream,
            addr_of!(header) as *const c_void,
            core::mem::size_of::<PresentationDataHeader>() as ULONG,
            core::ptr::null_mut(),
        );
        if hr == S_OK && data_size != 0 {
            hr = IStream_Write(
                stream,
                bmi as *const c_void,
                data_size as ULONG,
                core::ptr::null_mut(),
            );
        }
    } else if data_size != 0 {
        let mut bmp_fhdr: BITMAPFILEHEADER = core::mem::zeroed();

        bmp_fhdr.bfType = 0x4d42;
        bmp_fhdr.bfSize = (data_size + core::mem::size_of::<BITMAPFILEHEADER>() as c_int) as DWORD;
        bmp_fhdr.bfReserved1 = 0;
        bmp_fhdr.bfReserved2 = 0;
        bmp_fhdr.bfOffBits = (bitmap_info_size(bmi, DIB_RGB_COLORS as WORD)
            + core::mem::size_of::<BITMAPFILEHEADER>() as c_int)
            as DWORD;
        hr = IStream_Write(
            stream,
            addr_of!(bmp_fhdr) as *const c_void,
            core::mem::size_of::<BITMAPFILEHEADER>() as ULONG,
            core::ptr::null_mut(),
        );
        if hr == S_OK {
            hr = IStream_Write(
                stream,
                bmi as *const c_void,
                data_size as ULONG,
                core::ptr::null_mut(),
            );
        }
    }

    // C: `end:`
    if !bmi.is_null() {
        GlobalUnlock((*entry).stgmedium.__bindgen_anon_1.hGlobal);
    }
    hr
}

unsafe fn save_mfpict(entry: *mut DataCacheEntry, contents: BOOL, stream: *mut IStream) -> HRESULT {
    let mut hr: HRESULT = S_OK;
    let mut data_size: c_int = 0;
    let mut data: LPVOID = core::ptr::null_mut();
    let mfpict: *mut METAFILEPICT;

    if contents == 0 {
        let mut header: PresentationDataHeader = core::mem::zeroed();

        init_stream_header(entry, addr_of_mut!(header));
        hr = write_clipformat(stream, (*entry).fmtetc.cfFormat);
        if FAILED(hr) {
            return hr;
        }
        if (*entry).stgmedium.tymed != TYMED_NULL {
            mfpict = GlobalLock((*entry).stgmedium.__bindgen_anon_1.hMetaFilePict)
                as *mut METAFILEPICT;
            if mfpict.is_null() {
                return DV_E_STGMEDIUM;
            }
            data_size = GetMetaFileBitsEx((*mfpict).hMF, 0, core::ptr::null_mut()) as c_int;
            header.dwObjectExtentX = (*mfpict).xExt as DWORD;
            header.dwObjectExtentY = (*mfpict).yExt as DWORD;
            header.dwSize = data_size as DWORD;
            data = HeapAlloc(GetProcessHeap(), 0, header.dwSize as SIZE_T);
            if data.is_null() {
                GlobalUnlock((*entry).stgmedium.__bindgen_anon_1.hMetaFilePict);
                return E_OUTOFMEMORY;
            }
            GetMetaFileBitsEx((*mfpict).hMF, header.dwSize, data);
            GlobalUnlock((*entry).stgmedium.__bindgen_anon_1.hMetaFilePict);
        }
        hr = IStream_Write(
            stream,
            addr_of!(header) as *const c_void,
            core::mem::size_of::<PresentationDataHeader>() as ULONG,
            core::ptr::null_mut(),
        );
        if hr == S_OK && data_size != 0 {
            hr = IStream_Write(stream, data, data_size as ULONG, core::ptr::null_mut());
        }
        HeapFree(GetProcessHeap(), 0, data);
    } else if (*entry).stgmedium.tymed != TYMED_NULL {
        let mut meta_place_rec: meta_placeable = core::mem::zeroed();
        let mut check: *mut WORD;

        mfpict =
            GlobalLock((*entry).stgmedium.__bindgen_anon_1.hMetaFilePict) as *mut METAFILEPICT;
        if mfpict.is_null() {
            return DV_E_STGMEDIUM;
        }
        data_size = GetMetaFileBitsEx((*mfpict).hMF, 0, core::ptr::null_mut()) as c_int;
        data = HeapAlloc(GetProcessHeap(), 0, data_size as SIZE_T);
        if data.is_null() {
            GlobalUnlock((*entry).stgmedium.__bindgen_anon_1.hMetaFilePict);
            return E_OUTOFMEMORY;
        }
        GetMetaFileBitsEx((*mfpict).hMF, data_size as UINT, data);

        /* units are in 1/8th of a point (1 point is 1/72th of an inch) */
        meta_place_rec.key = 0x9ac6cdd7;
        meta_place_rec.hwmf = 0;
        meta_place_rec.inch = 576;
        meta_place_rec.bounding_box[0] = 0;
        meta_place_rec.bounding_box[1] = 0;
        meta_place_rec.bounding_box[2] = 0;
        meta_place_rec.bounding_box[3] = 0;
        meta_place_rec.checksum = 0;
        meta_place_rec.reserved = 0;

        /* These values are rounded down so MulDiv won't do the right thing */
        let inch = meta_place_rec.inch as i64;
        meta_place_rec.bounding_box[2] = ((*mfpict).xExt as i64 * inch / 2540) as WORD;
        meta_place_rec.bounding_box[3] = ((*mfpict).yExt as i64 * inch / 2540) as WORD;
        GlobalUnlock((*entry).stgmedium.__bindgen_anon_1.hMetaFilePict);

        check = addr_of_mut!(meta_place_rec) as *mut WORD;
        while check != addr_of_mut!(meta_place_rec.checksum) {
            meta_place_rec.checksum ^= *check;
            check = check.add(1);
        }
        hr = IStream_Write(
            stream,
            addr_of!(meta_place_rec) as *const c_void,
            core::mem::size_of::<meta_placeable>() as ULONG,
            core::ptr::null_mut(),
        );
        if hr == S_OK && data_size != 0 {
            hr = IStream_Write(stream, data, data_size as ULONG, core::ptr::null_mut());
        }
        HeapFree(GetProcessHeap(), 0, data);
    }

    hr
}

unsafe fn save_emf(entry: *mut DataCacheEntry, contents: BOOL, stream: *mut IStream) -> HRESULT {
    let mut hr: HRESULT = S_OK;
    let mut data_size: c_int;
    let data: *mut BYTE;

    if contents == 0 {
        let mut header: PresentationDataHeader = core::mem::zeroed();
        let mfpict: *mut METAFILEPICT;
        let hdc: HDC = GetDC(core::ptr::null_mut());

        init_stream_header(entry, addr_of_mut!(header));
        hr = write_clipformat(stream, (*entry).fmtetc.cfFormat);
        if FAILED(hr) {
            ReleaseDC(core::ptr::null_mut(), hdc);
            return hr;
        }
        data_size = GetWinMetaFileBits(
            (*entry).stgmedium.__bindgen_anon_1.hEnhMetaFile,
            0,
            core::ptr::null_mut(),
            MM_ANISOTROPIC,
            hdc,
        ) as c_int;
        header.dwSize = data_size as DWORD;
        data = HeapAlloc(GetProcessHeap(), 0, header.dwSize as SIZE_T) as *mut BYTE;
        if data.is_null() {
            ReleaseDC(core::ptr::null_mut(), hdc);
            return E_OUTOFMEMORY;
        }
        GetWinMetaFileBits(
            (*entry).stgmedium.__bindgen_anon_1.hEnhMetaFile,
            header.dwSize,
            data,
            MM_ANISOTROPIC,
            hdc,
        );
        ReleaseDC(core::ptr::null_mut(), hdc);
        mfpict = data as *mut METAFILEPICT;
        header.dwObjectExtentX = (*mfpict).xExt as DWORD;
        header.dwObjectExtentY = (*mfpict).yExt as DWORD;
        hr = IStream_Write(
            stream,
            addr_of!(header) as *const c_void,
            core::mem::size_of::<PresentationDataHeader>() as ULONG,
            core::ptr::null_mut(),
        );
        if hr == S_OK && data_size != 0 {
            hr = IStream_Write(
                stream,
                data as *const c_void,
                data_size as ULONG,
                core::ptr::null_mut(),
            );
        }
        HeapFree(GetProcessHeap(), 0, data as LPVOID);
    } else if (*entry).stgmedium.tymed != TYMED_NULL {
        const DW: usize = core::mem::size_of::<DWORD>();
        let emh: usize = core::mem::size_of::<ENHMETAHEADER>();

        data_size = GetEnhMetaFileBits(
            (*entry).stgmedium.__bindgen_anon_1.hEnhMetaFile,
            0,
            core::ptr::null_mut(),
        ) as c_int;
        data = HeapAlloc(
            GetProcessHeap(),
            0,
            (DW + emh + data_size as usize) as SIZE_T,
        ) as *mut BYTE;
        if data.is_null() {
            return E_OUTOFMEMORY;
        }
        *(data as *mut DWORD) = emh as DWORD;
        GetEnhMetaFileBits(
            (*entry).stgmedium.__bindgen_anon_1.hEnhMetaFile,
            data_size as UINT,
            data.add(DW + emh),
        );
        memcpy(
            data.add(DW) as *mut c_void,
            data.add(DW + emh) as *const c_void,
            emh,
        );
        data_size += (DW + emh) as c_int;
        hr = IStream_Write(
            stream,
            data as *const c_void,
            data_size as ULONG,
            core::ptr::null_mut(),
        );
        HeapFree(GetProcessHeap(), 0, data as LPVOID);
    }

    hr
}

unsafe fn save_view_cache(entry: *mut DataCacheEntry, stream: *mut IStream) -> HRESULT {
    let mut header: PresentationDataHeader = core::mem::zeroed();

    init_stream_header(entry, addr_of_mut!(header));
    let mut hr = write_clipformat(stream, (*entry).fmtetc.cfFormat);
    if SUCCEEDED(hr) {
        // C: FIELD_OFFSET(PresentationDataHeader, unknown7) — a SHORT write, header is truncated.
        hr = IStream_Write(
            stream,
            addr_of!(header) as *const c_void,
            core::mem::offset_of!(PresentationDataHeader, unknown7) as ULONG,
            core::ptr::null_mut(),
        );
    }

    hr
}

unsafe fn create_stream(
    cache_entry: *mut DataCacheEntry,
    storage: *mut IStorage,
    contents: BOOL,
    stream: *mut *mut IStream,
) -> HRESULT {
    let pres: [WCHAR; 12] = [
        2,
        'O' as WCHAR,
        'l' as WCHAR,
        'e' as WCHAR,
        'P' as WCHAR,
        'r' as WCHAR,
        'e' as WCHAR,
        's' as WCHAR,
        ('0' as c_int + ((*cache_entry).save_stream_num / 100) % 10) as WCHAR,
        ('0' as c_int + ((*cache_entry).save_stream_num / 10) % 10) as WCHAR,
        ('0' as c_int + (*cache_entry).save_stream_num % 10) as WCHAR,
        0,
    ];
    let name: *const WCHAR = if contents != 0 {
        CONTENTS.as_ptr()
    } else {
        pres.as_ptr()
    };

    IStorage_CreateStream(
        storage,
        name,
        STGM_READWRITE | STGM_SHARE_EXCLUSIVE | STGM_CREATE,
        0,
        0,
        stream,
    )
}

unsafe fn DataCacheEntry_Save(
    cache_entry: *mut DataCacheEntry,
    storage: *mut IStorage,
    same_as_load: BOOL,
) -> HRESULT {
    let _ = same_as_load;
    let mut stream: *mut IStream = core::ptr::null_mut();
    let contents: BOOL = if (*cache_entry).id == 1 { 1 } else { 0 };

    TRACE!(
        b"stream_number = %d, fmtetc = %s\n\0".as_ptr() as *const c_char,
        (*cache_entry).save_stream_num,
        debugstr_formatetc(addr_of!((*cache_entry).fmtetc)),
    );

    let mut hr = create_stream(cache_entry, storage, contents, addr_of_mut!(stream));
    if FAILED(hr) {
        return hr;
    }

    match (*cache_entry).fmtetc.cfFormat as DWORD {
        CF_DIB => {
            hr = save_dib(cache_entry, contents, stream);
        }
        CF_METAFILEPICT => {
            hr = save_mfpict(cache_entry, contents, stream);
        }
        CF_ENHMETAFILE => {
            hr = save_emf(cache_entry, contents, stream);
        }
        0 => {
            hr = save_view_cache(cache_entry, stream);
        }
        _ => {
            FIXME!(
                b"got unsupported clipboard format %x\n\0".as_ptr() as *const c_char,
                (*cache_entry).fmtetc.cfFormat as core::ffi::c_uint,
            );
        }
    }

    IStream_Release(stream);
    hr
}

/// Copy a STGMEDIUM of type bitmap, MF, EMF or HGLOBAL. Does no checking of whether
/// src_stgm has a supported tymed — the caller must have done that.
unsafe fn copy_stg_medium(
    cf: CLIPFORMAT,
    dest_stgm: *mut STGMEDIUM,
    src_stgm: *const STGMEDIUM,
) -> HRESULT {
    if (*src_stgm).tymed == TYMED_MFPICT {
        let src_mfpict = GlobalLock((*src_stgm).__bindgen_anon_1.hMetaFilePict)
            as *const METAFILEPICT;

        if src_mfpict.is_null() {
            return DV_E_STGMEDIUM;
        }
        (*dest_stgm).__bindgen_anon_1.hMetaFilePict = GlobalAlloc(
            GMEM_MOVEABLE,
            core::mem::size_of::<METAFILEPICT>() as SIZE_T,
        );
        let dest_mfpict = GlobalLock((*dest_stgm).__bindgen_anon_1.hMetaFilePict)
            as *mut METAFILEPICT;
        if dest_mfpict.is_null() {
            GlobalUnlock((*src_stgm).__bindgen_anon_1.hMetaFilePict);
            return E_OUTOFMEMORY;
        }
        *dest_mfpict = *src_mfpict;
        (*dest_mfpict).hMF = CopyMetaFileW((*src_mfpict).hMF, core::ptr::null());
        GlobalUnlock((*src_stgm).__bindgen_anon_1.hMetaFilePict);
        GlobalUnlock((*dest_stgm).__bindgen_anon_1.hMetaFilePict);
    } else if (*src_stgm).tymed != TYMED_NULL {
        (*dest_stgm).__bindgen_anon_1.hGlobal = OleDuplicateData(
            (*src_stgm).__bindgen_anon_1.hGlobal,
            cf,
            GMEM_MOVEABLE,
        );
        if (*dest_stgm).__bindgen_anon_1.hGlobal.is_null() {
            return E_OUTOFMEMORY;
        }
    }
    (*dest_stgm).tymed = (*src_stgm).tymed;
    (*dest_stgm).pUnkForRelease = (*src_stgm).pUnkForRelease;
    if !(*dest_stgm).pUnkForRelease.is_null() {
        IUnknown_AddRef((*dest_stgm).pUnkForRelease);
    }
    S_OK
}

unsafe fn synthesize_dib(bm: HBITMAP, med: *mut STGMEDIUM) -> HRESULT {
    let hdc = GetDC(core::ptr::null_mut());
    let mut header: BITMAPINFOHEADER = core::mem::zeroed();
    let hr: HRESULT = E_FAIL;

    memset(
        addr_of_mut!(header) as *mut c_void,
        0,
        core::mem::size_of::<BITMAPINFOHEADER>(),
    );
    header.biSize = core::mem::size_of::<BITMAPINFOHEADER>() as DWORD;
    if GetDIBits(
        hdc,
        bm,
        0,
        0,
        core::ptr::null_mut(),
        addr_of_mut!(header) as *mut BITMAPINFO,
        DIB_RGB_COLORS,
    ) == 0
    {
        // C: `goto done;`
        ReleaseDC(core::ptr::null_mut(), hdc);
        return hr;
    }

    let header_size: DWORD = bitmap_info_size(
        addr_of!(header) as *const BITMAPINFO,
        DIB_RGB_COLORS as WORD,
    ) as DWORD;
    (*med).__bindgen_anon_1.hGlobal = GlobalAlloc(
        GMEM_MOVEABLE,
        (header_size + header.biSizeImage) as SIZE_T,
    );
    if (*med).__bindgen_anon_1.hGlobal.is_null() {
        // C: `goto done;`
        ReleaseDC(core::ptr::null_mut(), hdc);
        return hr;
    }

    let bmi = GlobalLock((*med).__bindgen_anon_1.hGlobal) as *mut BITMAPINFO;
    memset(bmi as *mut c_void, 0, header_size as usize);
    memcpy(
        bmi as *mut c_void,
        addr_of!(header) as *const c_void,
        header.biSize as usize,
    );
    GetDIBits(
        hdc,
        bm,
        0,
        abs(header.biHeight) as UINT,
        (bmi as *mut BYTE).add(header_size as usize) as *mut c_void,
        bmi,
        DIB_RGB_COLORS,
    );
    GlobalUnlock((*med).__bindgen_anon_1.hGlobal);
    (*med).tymed = TYMED_HGLOBAL;
    (*med).pUnkForRelease = core::ptr::null_mut();
    let hr = S_OK;

    // C: `done:`
    ReleaseDC(core::ptr::null_mut(), hdc);
    hr
}

unsafe fn synthesize_bitmap(dib: HGLOBAL, med: *mut STGMEDIUM) -> HRESULT {
    let mut hr: HRESULT = E_FAIL;
    let hdc: HDC = GetDC(core::ptr::null_mut());

    let bmi = GlobalLock(dib) as *mut BITMAPINFO;
    if !bmi.is_null() {
        /* FIXME: validate data size */
        let offset = bitmap_info_size(bmi, DIB_RGB_COLORS as WORD);
        let bits_ptr = (bmi as *const BYTE).offset(offset as isize) as *const c_void;

        (*med).__bindgen_anon_1.hBitmap = CreateDIBitmap(
            hdc,
            addr_of!((*bmi).bmiHeader),
            CBM_INIT,
            bits_ptr,
            bmi,
            DIB_RGB_COLORS,
        );
        GlobalUnlock(dib);
        (*med).tymed = TYMED_GDI;
        (*med).pUnkForRelease = core::ptr::null_mut();
        hr = S_OK;
    }
    ReleaseDC(core::ptr::null_mut(), hdc);
    hr
}

unsafe fn DataCacheEntry_SetData(
    cache_entry: *mut DataCacheEntry,
    formatetc: *const FORMATETC,
    stgmedium: *mut STGMEDIUM,
    fRelease: BOOL,
) -> HRESULT {
    let mut copy: STGMEDIUM = core::mem::zeroed();
    let hr: HRESULT;
    let mut stgmedium = stgmedium;
    let mut fRelease = fRelease;

    if ((*cache_entry).fmtetc.cfFormat == 0 && (*formatetc).cfFormat == 0)
        || ((*cache_entry).fmtetc.tymed == TYMED_NULL && (*formatetc).tymed == TYMED_NULL)
        || (*stgmedium).tymed == TYMED_NULL
    {
        WARN!(b"invalid formatetc\n\0".as_ptr() as *const c_char);
        return DV_E_FORMATETC;
    }

    (*cache_entry).dirty = 1;
    ReleaseStgMedium(addr_of_mut!((*cache_entry).stgmedium));

    if (*formatetc).cfFormat as DWORD == CF_BITMAP {
        hr = synthesize_dib((*stgmedium).__bindgen_anon_1.hBitmap, addr_of_mut!(copy));
        if FAILED(hr) {
            return hr;
        }
        if fRelease != 0 {
            ReleaseStgMedium(stgmedium);
        }
        stgmedium = addr_of_mut!(copy);
        fRelease = 1;
    } else if (*formatetc).cfFormat as DWORD == CF_METAFILEPICT
        && (*cache_entry).fmtetc.cfFormat as DWORD == CF_ENHMETAFILE
    {
        hr = synthesize_emf(
            (*stgmedium).__bindgen_anon_1.hMetaFilePict,
            addr_of_mut!(copy),
        );
        if FAILED(hr) {
            return hr;
        }
        if fRelease != 0 {
            ReleaseStgMedium(stgmedium);
        }
        stgmedium = addr_of_mut!(copy);
        fRelease = 1;
    }

    if fRelease != 0 {
        (*cache_entry).stgmedium = *stgmedium;
        S_OK
    } else {
        copy_stg_medium(
            (*cache_entry).fmtetc.cfFormat,
            addr_of_mut!((*cache_entry).stgmedium),
            stgmedium,
        )
    }
}

unsafe fn DataCacheEntry_GetData(
    cache_entry: *mut DataCacheEntry,
    stg: *mut IStorage,
    fmt: *mut FORMATETC,
    stgmedium: *mut STGMEDIUM,
) -> HRESULT {
    if (*cache_entry).stgmedium.tymed == TYMED_NULL
        && (*cache_entry).load_stream_num != STREAM_NUMBER_NOT_SET
    {
        let hr = DataCacheEntry_LoadData(cache_entry, stg);
        if FAILED(hr) {
            return hr;
        }
    }
    if (*cache_entry).stgmedium.tymed == TYMED_NULL {
        return OLE_E_BLANK;
    }

    if (*fmt).cfFormat as DWORD == CF_BITMAP {
        return synthesize_bitmap(
            (*cache_entry).stgmedium.__bindgen_anon_1.hGlobal,
            stgmedium,
        );
    }

    copy_stg_medium(
        (*cache_entry).fmtetc.cfFormat,
        stgmedium,
        addr_of!((*cache_entry).stgmedium),
    )
}

unsafe fn DataCacheEntry_DiscardData(cache_entry: *mut DataCacheEntry) -> HRESULT {
    ReleaseStgMedium(addr_of_mut!((*cache_entry).stgmedium));
    S_OK
}

unsafe fn tymed_from_cf(cf: DWORD) -> DWORD {
    match cf {
        CF_BITMAP => TYMED_GDI,
        CF_METAFILEPICT => TYMED_MFPICT,
        CF_ENHMETAFILE => TYMED_ENHMF,
        // C: `case CF_DIB: default:` — both fall through to TYMED_HGLOBAL.
        _ => TYMED_HGLOBAL,
    }
}

/// Creates an appropriate cache entry for one of the CLSID_Picture_ classes. The
/// connection id of the entry is one. Any pre-existing automatic entry is re-assigned a
/// new connection id and moved to the end of the list.
unsafe fn create_automatic_entry(cache: *mut DataCache, clsid: *const CLSID) -> HRESULT {
    #[repr(C)]
    struct data {
        clsid: *const CLSID,
        fmt: FORMATETC,
    }

    // C: `static const struct data data[]`. Held as a function-local (read-only walk, never
    // mutated) because the raw `*const CLSID` members make it non-`Sync` as a `static`.
    let table: [data; 4] = [
        data {
            clsid: addr_of!(CLSID_Picture_Dib),
            fmt: FORMATETC {
                cfFormat: CF_DIB as CLIPFORMAT,
                ptd: core::ptr::null_mut(),
                dwAspect: DVASPECT_CONTENT,
                lindex: -1,
                tymed: TYMED_HGLOBAL,
            },
        },
        data {
            clsid: addr_of!(CLSID_Picture_Metafile),
            fmt: FORMATETC {
                cfFormat: CF_METAFILEPICT as CLIPFORMAT,
                ptd: core::ptr::null_mut(),
                dwAspect: DVASPECT_CONTENT,
                lindex: -1,
                tymed: TYMED_MFPICT,
            },
        },
        data {
            clsid: addr_of!(CLSID_Picture_EnhMetafile),
            fmt: FORMATETC {
                cfFormat: CF_ENHMETAFILE as CLIPFORMAT,
                ptd: core::ptr::null_mut(),
                dwAspect: DVASPECT_CONTENT,
                lindex: -1,
                tymed: TYMED_ENHMF,
            },
        },
        data {
            clsid: core::ptr::null(),
            fmt: FORMATETC {
                cfFormat: 0,
                ptd: core::ptr::null_mut(),
                dwAspect: 0,
                lindex: 0,
                tymed: 0,
            },
        },
    ];

    if IsEqualCLSID(addr_of!((*cache).clsid), clsid) {
        return S_OK;
    }

    /* move and reassign any pre-existing automatic entry */
    let head = list_head(addr_of_mut!((*cache).cache_list));
    if !head.is_null() {
        let entry =
            list_entry::<DataCacheEntry>(head, core::mem::offset_of!(DataCacheEntry, entry));
        if (*entry).id == 1 {
            list_remove(addr_of_mut!((*entry).entry));
            (*entry).id = (*cache).last_cache_id;
            (*cache).last_cache_id += 1;
            list_add_tail(
                addr_of_mut!((*cache).cache_list),
                addr_of_mut!((*entry).entry),
            );
        }
    }

    let mut ptr = table.as_ptr();
    while !(*ptr).clsid.is_null() {
        if IsEqualCLSID(clsid, (*ptr).clsid) {
            (*cache).clsid_static = 1;
            return DataCache_CreateEntry(
                cache,
                addr_of!((*ptr).fmt),
                0,
                1,
                core::ptr::null_mut(),
            );
        }
        ptr = ptr.add(1);
    }
    (*cache).clsid_static = 0;
    S_OK
}

// =====================================================================================
// Non-delegating IUnknown
// =====================================================================================

stdmethod! {

unsafe fn DataCache_NDIUnknown_QueryInterface(
    iface: *mut IUnknown,
    riid: *const IID,
    ppvObject: *mut *mut c_void,
) -> HRESULT {
    let this = impl_from_IUnknown(iface);

    if ppvObject.is_null() {
        return E_INVALIDARG;
    }

    *ppvObject = core::ptr::null_mut();

    if IsEqualIID(addr_of!(IID_IUnknown), riid) {
        if (*this).outer_unk == iface {
            /* non-aggregated, return IUnknown from IOleCache2 */
            *ppvObject = addr_of_mut!((*this).IOleCache2_iface) as *mut c_void;
        } else {
            *ppvObject = iface as *mut c_void;
        }
    } else if IsEqualIID(addr_of!(IID_IDataObject), riid) {
        *ppvObject = addr_of_mut!((*this).IDataObject_iface) as *mut c_void;
    } else if IsEqualIID(addr_of!(IID_IPersistStorage), riid)
        || IsEqualIID(addr_of!(IID_IPersist), riid)
    {
        *ppvObject = addr_of_mut!((*this).IPersistStorage_iface) as *mut c_void;
    } else if IsEqualIID(addr_of!(IID_IViewObject), riid)
        || IsEqualIID(addr_of!(IID_IViewObject2), riid)
    {
        *ppvObject = addr_of_mut!((*this).IViewObject2_iface) as *mut c_void;
    } else if IsEqualIID(addr_of!(IID_IOleCache), riid)
        || IsEqualIID(addr_of!(IID_IOleCache2), riid)
    {
        *ppvObject = addr_of_mut!((*this).IOleCache2_iface) as *mut c_void;
    } else if IsEqualIID(addr_of!(IID_IOleCacheControl), riid) {
        *ppvObject = addr_of_mut!((*this).IOleCacheControl_iface) as *mut c_void;
    }

    if (*ppvObject).is_null() {
        WARN!(
            b"() : asking for unsupported interface %s\n\0".as_ptr() as *const c_char,
            debugstr_guid(riid as *const c_void)
        );
        return E_NOINTERFACE;
    }

    IUnknown_AddRef(*ppvObject as *mut IUnknown);

    S_OK
}

unsafe fn DataCache_NDIUnknown_AddRef(iface: *mut IUnknown) -> ULONG {
    let this = impl_from_IUnknown(iface);
    InterlockedIncrement(addr_of_mut!((*this).ref_)) as ULONG
}

unsafe fn DataCache_NDIUnknown_Release(iface: *mut IUnknown) -> ULONG {
    let this = impl_from_IUnknown(iface);
    let ref_count = InterlockedDecrement(addr_of_mut!((*this).ref_)) as ULONG;

    if ref_count == 0 {
        DataCache_Destroy(this);
    }

    ref_count
}

// =====================================================================================
// IDataObject
// =====================================================================================

unsafe fn DataCache_IDataObject_QueryInterface(
    iface: *mut IDataObject,
    riid: *const IID,
    ppvObject: *mut *mut c_void,
) -> HRESULT {
    let this = impl_from_IDataObject(iface);
    IUnknown_QueryInterface((*this).outer_unk, riid, ppvObject)
}

unsafe fn DataCache_IDataObject_AddRef(iface: *mut IDataObject) -> ULONG {
    let this = impl_from_IDataObject(iface);
    IUnknown_AddRef((*this).outer_unk)
}

unsafe fn DataCache_IDataObject_Release(iface: *mut IDataObject) -> ULONG {
    let this = impl_from_IDataObject(iface);
    IUnknown_Release((*this).outer_unk)
}

unsafe fn DataCache_GetData(
    iface: *mut IDataObject,
    pformatetcIn: *mut FORMATETC,
    pmedium: *mut STGMEDIUM,
) -> HRESULT {
    let This = impl_from_IDataObject(iface);

    TRACE!(
        b"(%p, %s, %p)\n\0".as_ptr() as *const c_char,
        iface,
        debugstr_formatetc(pformatetcIn),
        pmedium,
    );

    core::ptr::write_bytes(pmedium, 0, 1);

    let cache_entry = DataCache_GetEntryForFormatEtc(This, pformatetcIn);
    if cache_entry.is_null() {
        return OLE_E_BLANK;
    }

    DataCacheEntry_GetData(
        cache_entry,
        (*This).presentationStorage,
        pformatetcIn,
        pmedium,
    )
}

unsafe fn DataCache_GetDataHere(
    iface: *mut IDataObject,
    pformatetc: *mut FORMATETC,
    pmedium: *mut STGMEDIUM,
) -> HRESULT {
    let _ = (iface, pformatetc, pmedium);
    FIXME!(b"stub\n\0".as_ptr() as *const c_char);
    E_NOTIMPL
}

unsafe fn DataCache_QueryGetData(iface: *mut IDataObject, fmt: *mut FORMATETC) -> HRESULT {
    let This = impl_from_IDataObject(iface);

    TRACE!(
        b"(%p)->(%s)\n\0".as_ptr() as *const c_char,
        iface,
        debugstr_formatetc(fmt)
    );
    let cache_entry = DataCache_GetEntryForFormatEtc(This, fmt);

    if !cache_entry.is_null() { S_OK } else { S_FALSE }
}

unsafe fn DataCache_GetCanonicalFormatEtc(
    iface: *mut IDataObject,
    pformatectIn: *mut FORMATETC,
    pformatetcOut: *mut FORMATETC,
) -> HRESULT {
    let _ = (iface, pformatectIn, pformatetcOut);
    TRACE!(b"()\n\0".as_ptr() as *const c_char);
    E_NOTIMPL
}

/// Delegated to the IOleCache2 implementation.
unsafe fn DataCache_IDataObject_SetData(
    iface: *mut IDataObject,
    pformatetc: *mut FORMATETC,
    pmedium: *mut STGMEDIUM,
    fRelease: BOOL,
) -> HRESULT {
    let mut oleCache: *mut IOleCache2 = core::ptr::null_mut();

    let mut hres = IDataObject_QueryInterface(
        iface,
        addr_of!(IID_IOleCache2),
        addr_of_mut!(oleCache) as *mut *mut c_void,
    );

    if FAILED(hres) {
        return E_UNEXPECTED;
    }

    hres = IOleCache2_SetData(oleCache, pformatetc, pmedium, fRelease);

    IOleCache2_Release(oleCache);

    hres
}

unsafe fn DataCache_EnumFormatEtc(
    iface: *mut IDataObject,
    dwDirection: DWORD,
    ppenumFormatEtc: *mut *mut IEnumFORMATETC,
) -> HRESULT {
    let _ = (iface, dwDirection, ppenumFormatEtc);
    TRACE!(b"()\n\0".as_ptr() as *const c_char);
    E_NOTIMPL
}

unsafe fn DataCache_DAdvise(
    iface: *mut IDataObject,
    pformatetc: *mut FORMATETC,
    advf: DWORD,
    pAdvSink: *mut IAdviseSink,
    pdwConnection: *mut DWORD,
) -> HRESULT {
    let _ = (iface, pformatetc, advf, pAdvSink, pdwConnection);
    TRACE!(b"()\n\0".as_ptr() as *const c_char);
    OLE_E_ADVISENOTSUPPORTED
}

unsafe fn DataCache_DUnadvise(iface: *mut IDataObject, dwConnection: DWORD) -> HRESULT {
    let _ = (iface, dwConnection);
    TRACE!(b"()\n\0".as_ptr() as *const c_char);
    OLE_E_NOCONNECTION
}

unsafe fn DataCache_EnumDAdvise(
    iface: *mut IDataObject,
    ppenumAdvise: *mut *mut IEnumSTATDATA,
) -> HRESULT {
    let _ = (iface, ppenumAdvise);
    TRACE!(b"()\n\0".as_ptr() as *const c_char);
    OLE_E_ADVISENOTSUPPORTED
}

// =====================================================================================
// IPersistStorage
// =====================================================================================

unsafe fn DataCache_IPersistStorage_QueryInterface(
    iface: *mut IPersistStorage,
    riid: *const IID,
    ppvObject: *mut *mut c_void,
) -> HRESULT {
    let this = impl_from_IPersistStorage(iface);
    IUnknown_QueryInterface((*this).outer_unk, riid, ppvObject)
}

unsafe fn DataCache_IPersistStorage_AddRef(iface: *mut IPersistStorage) -> ULONG {
    let this = impl_from_IPersistStorage(iface);
    IUnknown_AddRef((*this).outer_unk)
}

unsafe fn DataCache_IPersistStorage_Release(iface: *mut IPersistStorage) -> ULONG {
    let this = impl_from_IPersistStorage(iface);
    IUnknown_Release((*this).outer_unk)
}

unsafe fn DataCache_GetClassID(iface: *mut IPersistStorage, clsid: *mut CLSID) -> HRESULT {
    let This = impl_from_IPersistStorage(iface);

    TRACE!(
        b"(%p, %p) returning %s\n\0".as_ptr() as *const c_char,
        iface,
        clsid,
        debugstr_guid(addr_of!((*This).clsid) as *const c_void)
    );
    *clsid = (*This).clsid;

    S_OK
}

unsafe fn DataCache_IsDirty(iface: *mut IPersistStorage) -> HRESULT {
    let This = impl_from_IPersistStorage(iface);

    TRACE!(b"(%p)\n\0".as_ptr() as *const c_char, iface);

    if (*This).dirty != 0 {
        return S_OK;
    }

    LIST_FOR_EACH_ENTRY!(
        cache_entry,
        addr_of_mut!((*This).cache_list),
        DataCacheEntry,
        entry,
        {
            if (*cache_entry).dirty != 0 {
                return S_OK;
            }
        }
    );

    S_FALSE
}

/// Simply stores the storage pointer.
unsafe fn DataCache_InitNew(iface: *mut IPersistStorage, pStg: *mut IStorage) -> HRESULT {
    let This = impl_from_IPersistStorage(iface);
    let mut clsid: CLSID = core::mem::zeroed();

    TRACE!(b"(%p, %p)\n\0".as_ptr() as *const c_char, iface, pStg);

    if !(*This).presentationStorage.is_null() {
        return CO_E_ALREADYINITIALIZED;
    }

    (*This).presentationStorage = pStg;

    IStorage_AddRef((*This).presentationStorage);
    (*This).dirty = 1;
    ReadClassStg(pStg, addr_of_mut!(clsid));
    let hr = create_automatic_entry(This, addr_of!(clsid));
    if FAILED(hr) {
        IStorage_Release(pStg);
        (*This).presentationStorage = core::ptr::null_mut();
        return hr;
    }
    (*This).clsid = clsid;

    S_OK
}

} // stdmethod!

unsafe fn add_cache_entry(
    This: *mut DataCache,
    fmt: *const FORMATETC,
    advf: DWORD,
    stream_number: c_int,
) -> HRESULT {
    let mut cache_entry: *mut DataCacheEntry;
    let mut hr: HRESULT = S_OK;

    TRACE!(
        b"loading entry with formatetc: %s\n\0".as_ptr() as *const c_char,
        debugstr_formatetc(fmt),
    );

    cache_entry = DataCache_GetEntryForFormatEtc(This, fmt);
    if cache_entry.is_null() {
        hr = DataCache_CreateEntry(This, fmt, advf, 0, addr_of_mut!(cache_entry));
    }
    if SUCCEEDED(hr) {
        DataCacheEntry_DiscardData(cache_entry);
        (*cache_entry).load_stream_num = stream_number;
        (*cache_entry).save_stream_num = stream_number;
        (*cache_entry).dirty = 0;
    }
    hr
}

unsafe fn parse_pres_streams(cache: *mut DataCache, stg: *mut IStorage) -> HRESULT {
    let mut hr: HRESULT;
    let mut stm: *mut IStream = core::ptr::null_mut();
    let mut header: PresentationDataHeader = core::mem::zeroed();
    let mut actual_read: ULONG = 0;
    let mut clipformat: CLIPFORMAT = 0;
    let mut fmtetc: FORMATETC = core::mem::zeroed();
    let mut stream_number: c_int = 0;

    // C: `do { ... } while (hr == S_OK);`
    loop {
        hr = open_pres_stream(stg, stream_number, addr_of_mut!(stm));
        if FAILED(hr) {
            break;
        }

        hr = read_clipformat(stm, addr_of_mut!(clipformat));

        if hr == S_OK {
            hr = IStream_Read(
                stm,
                addr_of_mut!(header) as *mut c_void,
                core::mem::size_of::<PresentationDataHeader>() as ULONG,
                addr_of_mut!(actual_read),
            );
        }

        if hr == S_OK && actual_read == core::mem::size_of::<PresentationDataHeader>() as ULONG {
            fmtetc.cfFormat = clipformat;
            fmtetc.ptd = core::ptr::null_mut(); /* FIXME */
            fmtetc.dwAspect = header.dvAspect as DWORD;
            fmtetc.lindex = header.lindex as LONG;
            fmtetc.tymed = tymed_from_cf(clipformat as DWORD);

            add_cache_entry(cache, addr_of!(fmtetc), header.advf, stream_number);
        }
        IStream_Release(stm);
        stream_number += 1;

        if hr != S_OK {
            break;
        }
    }

    S_OK
}

unsafe fn parse_contents_stream(cache: *mut DataCache, stg: *mut IStorage) -> HRESULT {
    let mut stm: *mut IStream = core::ptr::null_mut();
    let mut cache_entry: *mut DataCacheEntry = core::ptr::null_mut();

    let mut hr = open_pres_stream(stg, STREAM_NUMBER_CONTENTS, addr_of_mut!(stm));
    if FAILED(hr) {
        return hr;
    }

    hr = get_static_entry(cache, addr_of_mut!(cache_entry));
    if hr == S_OK {
        (*cache_entry).load_stream_num = STREAM_NUMBER_CONTENTS;
        (*cache_entry).save_stream_num = STREAM_NUMBER_CONTENTS;
        (*cache_entry).dirty = 0;
    }

    IStream_Release(stm);
    hr
}

stdmethod! {

/// Doesn't actually load anything: holds on to the storage pointer and loads the
/// presentation lazily in GetData/Draw.
unsafe fn DataCache_Load(iface: *mut IPersistStorage, stg: *mut IStorage) -> HRESULT {
    let This = impl_from_IPersistStorage(iface);
    let mut hr: HRESULT;
    let mut clsid: CLSID = core::mem::zeroed();

    TRACE!(b"(%p, %p)\n\0".as_ptr() as *const c_char, iface, stg);

    IPersistStorage_HandsOffStorage(iface);

    LIST_FOR_EACH_ENTRY_SAFE!(
        entry,
        addr_of_mut!((*This).cache_list),
        DataCacheEntry,
        entry,
        {
            DataCacheEntry_Destroy(This, entry);
        }
    );
    (*This).clsid = CLSID_NULL;

    ReadClassStg(stg, addr_of_mut!(clsid));
    hr = create_automatic_entry(This, addr_of!(clsid));
    if FAILED(hr) {
        return hr;
    }

    (*This).clsid = clsid;

    if (*This).clsid_static != 0 {
        hr = parse_contents_stream(This, stg);
        if FAILED(hr) {
            hr = parse_pres_streams(This, stg);
        }
    } else {
        hr = parse_pres_streams(This, stg);
    }

    if SUCCEEDED(hr) {
        (*This).dirty = 0;
        (*This).presentationStorage = stg;
        IStorage_AddRef((*This).presentationStorage);
    }

    hr
}

unsafe fn DataCache_Save(
    iface: *mut IPersistStorage,
    stg: *mut IStorage,
    same_as_load: BOOL,
) -> HRESULT {
    let This = impl_from_IPersistStorage(iface);
    let mut hr: HRESULT = S_OK;
    let mut stream_number: c_int = 0;

    TRACE!(
        b"(%p, %p, %d)\n\0".as_ptr() as *const c_char,
        iface,
        stg,
        same_as_load,
    );

    /* assign stream numbers to the cache entries */
    LIST_FOR_EACH_ENTRY!(
        cache_entry,
        addr_of_mut!((*This).cache_list),
        DataCacheEntry,
        entry,
        {
            if (*cache_entry).save_stream_num != stream_number {
                (*cache_entry).dirty = 1; /* needs to be written out again */
                (*cache_entry).save_stream_num = stream_number;
            }
            stream_number += 1;
        }
    );

    /* write out the cache entries */
    LIST_FOR_EACH_ENTRY!(
        cache_entry,
        addr_of_mut!((*This).cache_list),
        DataCacheEntry,
        entry,
        {
            if same_as_load == 0 || (*cache_entry).dirty != 0 {
                hr = DataCacheEntry_Save(cache_entry, stg, same_as_load);
                if FAILED(hr) {
                    break;
                }

                if same_as_load != 0 {
                    (*cache_entry).dirty = 0;
                }
            }
        }
    );

    if same_as_load != 0 {
        (*This).dirty = 0;
    }
    hr
}

unsafe fn DataCache_SaveCompleted(
    iface: *mut IPersistStorage,
    pStgNew: *mut IStorage,
) -> HRESULT {
    TRACE!(b"(%p, %p)\n\0".as_ptr() as *const c_char, iface, pStgNew);

    if !pStgNew.is_null() {
        IPersistStorage_HandsOffStorage(iface);

        DataCache_Load(iface, pStgNew);
    }

    S_OK
}

unsafe fn DataCache_HandsOffStorage(iface: *mut IPersistStorage) -> HRESULT {
    let this = impl_from_IPersistStorage(iface);

    TRACE!(b"(%p)\n\0".as_ptr() as *const c_char, iface);

    if !(*this).presentationStorage.is_null() {
        IStorage_Release((*this).presentationStorage);
        (*this).presentationStorage = core::ptr::null_mut();
    }

    S_OK
}

// =====================================================================================
// IViewObject2
// =====================================================================================

unsafe fn DataCache_IViewObject2_QueryInterface(
    iface: *mut IViewObject2,
    riid: *const IID,
    ppvObject: *mut *mut c_void,
) -> HRESULT {
    let this = impl_from_IViewObject2(iface);
    IUnknown_QueryInterface((*this).outer_unk, riid, ppvObject)
}

unsafe fn DataCache_IViewObject2_AddRef(iface: *mut IViewObject2) -> ULONG {
    let this = impl_from_IViewObject2(iface);
    IUnknown_AddRef((*this).outer_unk)
}

unsafe fn DataCache_IViewObject2_Release(iface: *mut IViewObject2) -> ULONG {
    let this = impl_from_IViewObject2(iface);
    IUnknown_Release((*this).outer_unk)
}

unsafe fn DataCache_Draw(
    iface: *mut IViewObject2,
    dwDrawAspect: DWORD,
    lindex: LONG,
    pvAspect: *mut c_void,
    ptd: *mut DVTARGETDEVICE,
    hdcTargetDev: HDC,
    hdcDraw: HDC,
    lprcBounds: LPCRECTL,
    lprcWBounds: LPCRECTL,
    pfnContinue: PFN_CONTINUE,
    dwContinue: ULONG_PTR,
) -> HRESULT {
    let _ = (pvAspect, ptd, hdcTargetDev, lprcWBounds);
    let This = impl_from_IViewObject2(iface);
    let mut hres: HRESULT;

    TRACE!(
        b"(%p, %x, %d, %p, %p, %p, %p, %p, %p, %lx)\n\0".as_ptr() as *const c_char,
        iface,
        dwDrawAspect,
        lindex,
        pvAspect,
        hdcTargetDev,
        hdcDraw,
        lprcBounds,
        lprcWBounds,
        dwContinue,
    );

    if lprcBounds.is_null() {
        return E_INVALIDARG;
    }

    // Hand-rolled walk (not LIST_FOR_EACH_ENTRY!), from when the macro's link-advance sat
    // AFTER the body and the C body's `continue` would have spun forever there. The macro now
    // advances on `continue`; this walk advances before each one by hand.
    let head = addr_of_mut!((*This).cache_list);
    let mut current = (*head).next;

    while current != head {
        let cache_entry: *mut DataCacheEntry =
            list_entry::<DataCacheEntry>(current, core::mem::offset_of!(DataCacheEntry, entry));
        let next = (*current).next;

        /* FIXME: compare ptd too */
        if ((*cache_entry).fmtetc.dwAspect != dwDrawAspect)
            || ((*cache_entry).fmtetc.lindex != lindex)
        {
            current = next;
            continue;
        }

        /* if the data hasn't been loaded yet, do it now */
        if ((*cache_entry).stgmedium.tymed == TYMED_NULL)
            && ((*cache_entry).load_stream_num != STREAM_NUMBER_NOT_SET)
        {
            hres = DataCacheEntry_LoadData(cache_entry, (*This).presentationStorage);
            if FAILED(hres) {
                current = next;
                continue;
            }
        }

        /* no data */
        if (*cache_entry).stgmedium.tymed == TYMED_NULL {
            current = next;
            continue;
        }

        if let Some(continue_fn) = pfnContinue {
            if continue_fn(dwContinue) == 0 {
                return E_ABORT;
            }
        }

        match (*cache_entry).fmtetc.cfFormat as DWORD {
            CF_METAFILEPICT => {
                /* We have to be careful not to modify the state of the DC. */
                let mut oldWindowExt: SIZE = core::mem::zeroed();
                let mut oldViewportExt: SIZE = core::mem::zeroed();
                let mut oldViewportOrg: POINT = core::mem::zeroed();

                if (*cache_entry).stgmedium.tymed != TYMED_MFPICT {
                    current = next;
                    continue;
                }

                let mfpict = GlobalLock((*cache_entry).stgmedium.__bindgen_anon_1.hMetaFilePict)
                    as *mut METAFILEPICT;
                if mfpict.is_null() {
                    current = next;
                    continue;
                }

                let prevMapMode: c_int = SetMapMode(hdcDraw, (*mfpict).mm);

                SetWindowExtEx(
                    hdcDraw,
                    (*mfpict).xExt,
                    (*mfpict).yExt,
                    addr_of_mut!(oldWindowExt),
                );

                SetViewportExtEx(
                    hdcDraw,
                    (*lprcBounds).right - (*lprcBounds).left,
                    (*lprcBounds).bottom - (*lprcBounds).top,
                    addr_of_mut!(oldViewportExt),
                );

                SetViewportOrgEx(
                    hdcDraw,
                    (*lprcBounds).left,
                    (*lprcBounds).top,
                    addr_of_mut!(oldViewportOrg),
                );

                PlayMetaFile(hdcDraw, (*mfpict).hMF);

                SetWindowExtEx(
                    hdcDraw,
                    oldWindowExt.cx,
                    oldWindowExt.cy,
                    core::ptr::null_mut(),
                );

                SetViewportExtEx(
                    hdcDraw,
                    oldViewportExt.cx,
                    oldViewportExt.cy,
                    core::ptr::null_mut(),
                );

                SetViewportOrgEx(
                    hdcDraw,
                    oldViewportOrg.x,
                    oldViewportOrg.y,
                    core::ptr::null_mut(),
                );

                SetMapMode(hdcDraw, prevMapMode);

                GlobalUnlock((*cache_entry).stgmedium.__bindgen_anon_1.hMetaFilePict);

                return S_OK;
            }
            CF_DIB => {
                if (*cache_entry).stgmedium.tymed != TYMED_HGLOBAL {
                    current = next;
                    continue;
                }

                let info = GlobalLock((*cache_entry).stgmedium.__bindgen_anon_1.hGlobal)
                    as *mut BITMAPINFO;
                if info.is_null() {
                    current = next;
                    continue;
                }

                let bits = (info as *mut BYTE)
                    .offset(bitmap_info_size(info, DIB_RGB_COLORS as WORD) as isize);
                StretchDIBits(
                    hdcDraw,
                    (*lprcBounds).left,
                    (*lprcBounds).top,
                    (*lprcBounds).right - (*lprcBounds).left,
                    (*lprcBounds).bottom - (*lprcBounds).top,
                    0,
                    0,
                    (*info).bmiHeader.biWidth,
                    (*info).bmiHeader.biHeight,
                    bits as *const c_void,
                    info as *const BITMAPINFO,
                    DIB_RGB_COLORS,
                    SRCCOPY,
                );

                GlobalUnlock((*cache_entry).stgmedium.__bindgen_anon_1.hGlobal);
                return S_OK;
            }
            _ => {}
        }

        current = next;
    }

    WARN!(b"no data could be found to be drawn\n\0".as_ptr() as *const c_char);

    OLE_E_BLANK
}

unsafe fn DataCache_GetColorSet(
    iface: *mut IViewObject2,
    dwDrawAspect: DWORD,
    lindex: LONG,
    pvAspect: *mut c_void,
    ptd: *mut DVTARGETDEVICE,
    hicTargetDev: HDC,
    ppColorSet: *mut *mut LOGPALETTE,
) -> HRESULT {
    let _ = (iface, dwDrawAspect, lindex, pvAspect, ptd, hicTargetDev, ppColorSet);
    FIXME!(b"stub\n\0".as_ptr() as *const c_char);
    E_NOTIMPL
}

unsafe fn DataCache_Freeze(
    iface: *mut IViewObject2,
    dwDrawAspect: DWORD,
    lindex: LONG,
    pvAspect: *mut c_void,
    pdwFreeze: *mut DWORD,
) -> HRESULT {
    let _ = (iface, dwDrawAspect, lindex, pvAspect, pdwFreeze);
    FIXME!(b"stub\n\0".as_ptr() as *const c_char);
    E_NOTIMPL
}

unsafe fn DataCache_Unfreeze(iface: *mut IViewObject2, dwFreeze: DWORD) -> HRESULT {
    let _ = (iface, dwFreeze);
    FIXME!(b"stub\n\0".as_ptr() as *const c_char);
    E_NOTIMPL
}

/// Sets up an advisory sink with the data cache: when the object's view changes, this
/// sink is called.
unsafe fn DataCache_SetAdvise(
    iface: *mut IViewObject2,
    aspects: DWORD,
    advf: DWORD,
    pAdvSink: *mut IAdviseSink,
) -> HRESULT {
    let this = impl_from_IViewObject2(iface);

    TRACE!(
        b"(%p, %x, %x, %p)\n\0".as_ptr() as *const c_char,
        iface,
        aspects,
        advf,
        pAdvSink,
    );

    /* A call to this function removes the previous sink */
    if !(*this).sinkInterface.is_null() {
        IAdviseSink_Release((*this).sinkInterface);
        (*this).sinkInterface = core::ptr::null_mut();
        (*this).sinkAspects = 0;
        (*this).sinkAdviseFlag = 0;
    }

    if !pAdvSink.is_null() {
        (*this).sinkInterface = pAdvSink;
        (*this).sinkAspects = aspects;
        (*this).sinkAdviseFlag = advf;

        IAdviseSink_AddRef((*this).sinkInterface);
    }

    if (advf & ADVF_PRIMEFIRST) != 0 {
        DataCache_FireOnViewChange(this, aspects, -1);
    }

    S_OK
}

unsafe fn DataCache_GetAdvise(
    iface: *mut IViewObject2,
    pAspects: *mut DWORD,
    pAdvf: *mut DWORD,
    ppAdvSink: *mut *mut IAdviseSink,
) -> HRESULT {
    let this = impl_from_IViewObject2(iface);

    TRACE!(
        b"(%p, %p, %p, %p)\n\0".as_ptr() as *const c_char,
        iface,
        pAspects,
        pAdvf,
        ppAdvSink,
    );

    if !pAspects.is_null() {
        *pAspects = (*this).sinkAspects;
    }

    if !pAdvf.is_null() {
        *pAdvf = (*this).sinkAdviseFlag;
    }

    if !ppAdvSink.is_null() {
        if !(*this).sinkInterface.is_null() {
            IAdviseSink_QueryInterface(
                (*this).sinkInterface,
                addr_of!(IID_IAdviseSink),
                ppAdvSink as *mut *mut c_void,
            );
        } else {
            *ppAdvSink = core::ptr::null_mut();
        }
    }

    S_OK
}

unsafe fn DataCache_GetExtent(
    iface: *mut IViewObject2,
    dwDrawAspect: DWORD,
    lindex: LONG,
    ptd: *mut DVTARGETDEVICE,
    lpsizel: LPSIZEL,
) -> HRESULT {
    let This = impl_from_IViewObject2(iface);
    let mut hres: HRESULT;

    TRACE!(
        b"(%p, %x, %d, %p, %p)\n\0".as_ptr() as *const c_char,
        iface,
        dwDrawAspect,
        lindex,
        ptd,
        lpsizel,
    );

    if lpsizel.is_null() {
        return E_POINTER;
    }

    (*lpsizel).cx = 0;
    (*lpsizel).cy = 0;

    if lindex != -1 {
        FIXME!(
            b"Unimplemented flag lindex = %d\n\0".as_ptr() as *const c_char,
            lindex
        );
    }

    if !ptd.is_null() {
        FIXME!(b"Unimplemented ptd = %p\n\0".as_ptr() as *const c_char, ptd);
    }

    // Hand-rolled walk — the C body uses `continue` (see the note in DataCache_Draw).
    let head = addr_of_mut!((*This).cache_list);
    let mut link = (*head).next;

    while link != head {
        let cache_entry: *mut DataCacheEntry =
            list_entry::<DataCacheEntry>(link, core::mem::offset_of!(DataCacheEntry, entry));
        let next = (*link).next;

        if ((*cache_entry).fmtetc.dwAspect != dwDrawAspect)
            || ((*cache_entry).fmtetc.lindex != lindex)
        {
            link = next;
            continue;
        }

        if ((*cache_entry).stgmedium.tymed == TYMED_NULL)
            && ((*cache_entry).load_stream_num != STREAM_NUMBER_NOT_SET)
        {
            hres = DataCacheEntry_LoadData(cache_entry, (*This).presentationStorage);
            if FAILED(hres) {
                link = next;
                continue;
            }
        }

        if (*cache_entry).stgmedium.tymed == TYMED_NULL {
            link = next;
            continue;
        }

        match (*cache_entry).fmtetc.cfFormat as DWORD {
            CF_METAFILEPICT => {
                if (*cache_entry).stgmedium.tymed != TYMED_MFPICT {
                    link = next;
                    continue;
                }

                let mfpict = GlobalLock((*cache_entry).stgmedium.__bindgen_anon_1.hMetaFilePict)
                    as *mut METAFILEPICT;
                if mfpict.is_null() {
                    link = next;
                    continue;
                }

                (*lpsizel).cx = (*mfpict).xExt;
                (*lpsizel).cy = (*mfpict).yExt;

                GlobalUnlock((*cache_entry).stgmedium.__bindgen_anon_1.hMetaFilePict);

                return S_OK;
            }
            CF_DIB => {
                if (*cache_entry).stgmedium.tymed != TYMED_HGLOBAL {
                    link = next;
                    continue;
                }

                let info = GlobalLock((*cache_entry).stgmedium.__bindgen_anon_1.hGlobal)
                    as *mut BITMAPINFOHEADER;
                if info.is_null() {
                    link = next;
                    continue;
                }

                if (*info).biXPelsPerMeter != 0 && (*info).biYPelsPerMeter != 0 {
                    (*lpsizel).cx = (*info).biWidth * 100000 / (*info).biXPelsPerMeter;
                    (*lpsizel).cy = (*info).biHeight * 100000 / (*info).biYPelsPerMeter;
                } else {
                    let hdc = GetDC(core::ptr::null_mut());
                    (*lpsizel).cx = (*info).biWidth * 2540 / GetDeviceCaps(hdc, LOGPIXELSX);
                    (*lpsizel).cy = (*info).biHeight * 2540 / GetDeviceCaps(hdc, LOGPIXELSY);
                    ReleaseDC(core::ptr::null_mut(), hdc);
                }

                GlobalUnlock((*cache_entry).stgmedium.__bindgen_anon_1.hGlobal);

                return S_OK;
            }
            _ => {}
        }

        link = next;
    }

    WARN!(b"no data could be found to get the extents from\n\0".as_ptr() as *const c_char);

    OLE_E_BLANK
}

// =====================================================================================
// IOleCache2
// =====================================================================================

unsafe fn DataCache_IOleCache2_QueryInterface(
    iface: *mut IOleCache2,
    riid: *const IID,
    ppvObject: *mut *mut c_void,
) -> HRESULT {
    let this = impl_from_IOleCache2(iface);
    IUnknown_QueryInterface((*this).outer_unk, riid, ppvObject)
}

unsafe fn DataCache_IOleCache2_AddRef(iface: *mut IOleCache2) -> ULONG {
    let this = impl_from_IOleCache2(iface);
    IUnknown_AddRef((*this).outer_unk)
}

unsafe fn DataCache_IOleCache2_Release(iface: *mut IOleCache2) -> ULONG {
    let this = impl_from_IOleCache2(iface);
    IUnknown_Release((*this).outer_unk)
}

} // stdmethod!

/// Set up the sink connection to the running object.
unsafe fn setup_sink(This: *mut DataCache, cache_entry: *mut DataCacheEntry) -> HRESULT {
    let mut hr: HRESULT = S_FALSE;

    /* Clear the ADVFCACHE_* bits. Native also sets the two highest bits for some reason. */
    let flags: DWORD = (*cache_entry).advise_flags
        & !(ADVFCACHE_NOHANDLER | ADVFCACHE_FORCEBUILTIN | ADVFCACHE_ONSAVE);

    if !(*This).running_object.is_null() {
        if (flags & ADVF_NODATA) == 0 {
            hr = IDataObject_DAdvise(
                (*This).running_object,
                addr_of_mut!((*cache_entry).fmtetc),
                flags,
                addr_of_mut!((*This).IAdviseSink_iface),
                addr_of_mut!((*cache_entry).sink_id),
            );
        }
    }
    hr
}

stdmethod! {

unsafe fn DataCache_Cache(
    iface: *mut IOleCache2,
    pformatetc: *mut FORMATETC,
    advf: DWORD,
    pdwConnection: *mut DWORD,
) -> HRESULT {
    let This = impl_from_IOleCache2(iface);
    let mut cache_entry: *mut DataCacheEntry;
    let mut fmt_cpy: FORMATETC;

    TRACE!(
        b"(%p, 0x%x, %p)\n\0".as_ptr() as *const c_char,
        pformatetc,
        advf,
        pdwConnection
    );

    if pformatetc.is_null() || pdwConnection.is_null() {
        return E_INVALIDARG;
    }

    TRACE!(
        b"pformatetc = %s\n\0".as_ptr() as *const c_char,
        debugstr_formatetc(pformatetc)
    );

    fmt_cpy = *pformatetc; /* No need for a deep copy */
    if fmt_cpy.cfFormat as DWORD == CF_BITMAP && fmt_cpy.tymed == TYMED_GDI {
        fmt_cpy.cfFormat = CF_DIB as CLIPFORMAT;
        fmt_cpy.tymed = TYMED_HGLOBAL;
    }

    /* View caching DVASPECT_ICON gets converted to CF_METAFILEPICT */
    if fmt_cpy.dwAspect == DVASPECT_ICON && fmt_cpy.cfFormat == 0 {
        fmt_cpy.cfFormat = CF_METAFILEPICT as CLIPFORMAT;
        fmt_cpy.tymed = TYMED_MFPICT;
    }

    *pdwConnection = 0;

    cache_entry = DataCache_GetEntryForFormatEtc(This, addr_of!(fmt_cpy));
    if !cache_entry.is_null() {
        TRACE!(b"found an existing cache entry\n\0".as_ptr() as *const c_char);
        *pdwConnection = (*cache_entry).id;
        return CACHE_S_SAMECACHE;
    }

    if (*This).clsid_static != 0 && fmt_cpy.dwAspect != DVASPECT_ICON {
        return DV_E_FORMATETC;
    }

    let hr = DataCache_CreateEntry(This, addr_of!(fmt_cpy), advf, 0, addr_of_mut!(cache_entry));

    if SUCCEEDED(hr) {
        *pdwConnection = (*cache_entry).id;
        setup_sink(This, cache_entry);
    }

    hr
}

unsafe fn DataCache_Uncache(iface: *mut IOleCache2, dwConnection: DWORD) -> HRESULT {
    let This = impl_from_IOleCache2(iface);

    TRACE!(b"(%d)\n\0".as_ptr() as *const c_char, dwConnection);

    LIST_FOR_EACH_ENTRY!(
        cache_entry,
        addr_of_mut!((*This).cache_list),
        DataCacheEntry,
        entry,
        {
            if (*cache_entry).id == dwConnection {
                DataCacheEntry_Destroy(This, cache_entry);
                return S_OK;
            }
        }
    );

    WARN!(
        b"no connection found for %d\n\0".as_ptr() as *const c_char,
        dwConnection
    );

    OLE_E_NOCONNECTION
}

unsafe fn DataCache_EnumCache(
    iface: *mut IOleCache2,
    ppenumSTATDATA: *mut *mut IEnumSTATDATA,
) -> HRESULT {
    let This = impl_from_IOleCache2(iface);
    let mut i: c_int = 0;
    let mut count: c_int = 0;
    let mut hr: HRESULT;

    TRACE!(
        b"(%p, %p)\n\0".as_ptr() as *const c_char,
        This,
        ppenumSTATDATA
    );

    LIST_FOR_EACH_ENTRY!(
        cache_entry,
        addr_of_mut!((*This).cache_list),
        DataCacheEntry,
        entry,
        {
            count += 1;
            if (*cache_entry).fmtetc.cfFormat as DWORD == CF_DIB {
                count += 1;
            }
        }
    );

    let data = CoTaskMemAlloc((count as usize * core::mem::size_of::<STATDATA>()) as SIZE_T)
        as *mut STATDATA;
    if data.is_null() {
        return E_OUTOFMEMORY;
    }

    // C: `goto fail` — free every ptd copied so far, free the array, return hr.
    // UPSTREAM BUG: on the `i == count` paths the C jumps to `fail` with `hr` still
    // UNINITIALIZED (it is only assigned by copy_formatetc). Those paths are unreachable
    // (count is computed by the identical walk), so E_FAIL is used here rather than
    // reproducing a read of an indeterminate value.
    macro_rules! fail {
        () => {{
            while i > 0 {
                i -= 1;
                CoTaskMemFree((*data.offset(i as isize)).formatetc.ptd as LPVOID);
            }
            CoTaskMemFree(data as LPVOID);
            return hr
        }};
    }

    LIST_FOR_EACH_ENTRY!(
        cache_entry,
        addr_of_mut!((*This).cache_list),
        DataCacheEntry,
        entry,
        {
            if i == count {
                hr = E_FAIL;
                fail!();
            }
            hr = copy_formatetc(
                addr_of_mut!((*data.offset(i as isize)).formatetc),
                addr_of!((*cache_entry).fmtetc),
            );
            if FAILED(hr) {
                fail!();
            }
            (*data.offset(i as isize)).advf = (*cache_entry).advise_flags;
            (*data.offset(i as isize)).pAdvSink = core::ptr::null_mut();
            (*data.offset(i as isize)).dwConnection = (*cache_entry).id;
            i += 1;

            if (*cache_entry).fmtetc.cfFormat as DWORD == CF_DIB {
                if i == count {
                    hr = E_FAIL;
                    fail!();
                }
                hr = copy_formatetc(
                    addr_of_mut!((*data.offset(i as isize)).formatetc),
                    addr_of!((*cache_entry).fmtetc),
                );
                if FAILED(hr) {
                    fail!();
                }
                (*data.offset(i as isize)).formatetc.cfFormat = CF_BITMAP as CLIPFORMAT;
                (*data.offset(i as isize)).formatetc.tymed = TYMED_GDI;
                (*data.offset(i as isize)).advf = (*cache_entry).advise_flags;
                (*data.offset(i as isize)).pAdvSink = core::ptr::null_mut();
                (*data.offset(i as isize)).dwConnection = (*cache_entry).id;
                i += 1;
            }
        }
    );

    hr = EnumSTATDATA_Construct(
        core::ptr::null_mut(),
        0,
        i as DWORD,
        data,
        0,
        ppenumSTATDATA,
    );
    if SUCCEEDED(hr) {
        return hr;
    }

    fail!()
}

unsafe fn DataCache_InitCache(iface: *mut IOleCache2, pDataObject: *mut IDataObject) -> HRESULT {
    TRACE!(
        b"(%p %p)\n\0".as_ptr() as *const c_char,
        iface,
        pDataObject
    );
    IOleCache2_UpdateCache(
        iface,
        pDataObject,
        UPDFCACHE_ALLBUTNODATACACHE,
        core::ptr::null_mut(),
    )
}

unsafe fn DataCache_IOleCache2_SetData(
    iface: *mut IOleCache2,
    pformatetc: *mut FORMATETC,
    pmedium: *mut STGMEDIUM,
    fRelease: BOOL,
) -> HRESULT {
    let This = impl_from_IOleCache2(iface);

    TRACE!(
        b"(%p, %p, %s)\n\0".as_ptr() as *const c_char,
        pformatetc,
        pmedium,
        if fRelease != 0 {
            b"TRUE\0".as_ptr() as *const c_char
        } else {
            b"FALSE\0".as_ptr() as *const c_char
        },
    );
    TRACE!(
        b"formatetc = %s\n\0".as_ptr() as *const c_char,
        debugstr_formatetc(pformatetc),
    );

    let cache_entry = DataCache_GetEntryForFormatEtc(This, pformatetc);
    if !cache_entry.is_null() {
        let hr = DataCacheEntry_SetData(cache_entry, pformatetc, pmedium, fRelease);

        if SUCCEEDED(hr) {
            DataCache_FireOnViewChange(
                This,
                (*cache_entry).fmtetc.dwAspect,
                (*cache_entry).fmtetc.lindex,
            );
        }

        return hr;
    }
    WARN!(b"cache entry not found\n\0".as_ptr() as *const c_char);

    OLE_E_BLANK
}

} // stdmethod!

unsafe fn entry_updatable(entry: *mut DataCacheEntry, mode: DWORD) -> BOOL {
    let is_blank: BOOL = if (*entry).stgmedium.tymed == TYMED_NULL {
        1
    } else {
        0
    };

    if (mode & UPDFCACHE_ONLYIFBLANK) != 0 && is_blank == 0 {
        return 0;
    }

    if (mode & UPDFCACHE_NODATACACHE) != 0 && ((*entry).advise_flags & ADVF_NODATA) != 0 {
        return 1;
    }
    if (mode & UPDFCACHE_ONSAVECACHE) != 0 && ((*entry).advise_flags & ADVFCACHE_ONSAVE) != 0 {
        return 1;
    }
    if (mode & UPDFCACHE_ONSTOPCACHE) != 0 && ((*entry).advise_flags & ADVF_DATAONSTOP) != 0 {
        return 1;
    }
    if (mode & UPDFCACHE_NORMALCACHE) != 0 && (*entry).advise_flags == 0 {
        return 1;
    }
    if (mode & UPDFCACHE_IFBLANK) != 0
        && (is_blank != 0 && ((*entry).advise_flags & ADVF_NODATA) == 0)
    {
        return 1;
    }

    0
}

stdmethod! {

unsafe fn DataCache_UpdateCache(
    iface: *mut IOleCache2,
    pDataObject: LPDATAOBJECT,
    grfUpdf: DWORD,
    pReserved: LPVOID,
) -> HRESULT {
    let _ = pReserved; /* C: `reserved` is only ever traced */
    let This = impl_from_IOleCache2(iface);
    let mut med: STGMEDIUM = core::mem::zeroed();
    let mut hr: HRESULT;
    let view_list: [CLIPFORMAT; 4] = [
        CF_METAFILEPICT as CLIPFORMAT,
        CF_ENHMETAFILE as CLIPFORMAT,
        CF_DIB as CLIPFORMAT,
        CF_BITMAP as CLIPFORMAT,
    ];
    let mut fmt: FORMATETC = core::mem::zeroed();
    let mut slots: c_int = 0;
    let mut done_one: BOOL = 0;

    TRACE!(
        b"(%p %p %08x %p)\n\0".as_ptr() as *const c_char,
        iface,
        pDataObject,
        grfUpdf,
        pReserved,
    );

    // Hand-rolled walk — the C body uses `continue` (see the note in DataCache_Draw).
    let head = addr_of_mut!((*This).cache_list);
    let mut link = (*head).next;

    while link != head {
        let cache_entry: *mut DataCacheEntry =
            list_entry::<DataCacheEntry>(link, core::mem::offset_of!(DataCacheEntry, entry));
        let next = (*link).next;

        slots += 1;

        if entry_updatable(cache_entry, grfUpdf) == 0 {
            done_one = 1;
            link = next;
            continue;
        }

        fmt = (*cache_entry).fmtetc;

        if fmt.cfFormat != 0 {
            hr = IDataObject_GetData(pDataObject, addr_of_mut!(fmt), addr_of_mut!(med));
            if hr != S_OK && fmt.cfFormat as DWORD == CF_DIB {
                fmt.cfFormat = CF_BITMAP as CLIPFORMAT;
                fmt.tymed = TYMED_GDI;
                hr = IDataObject_GetData(pDataObject, addr_of_mut!(fmt), addr_of_mut!(med));
            }
            if hr != S_OK && fmt.cfFormat as DWORD == CF_ENHMETAFILE {
                fmt.cfFormat = CF_METAFILEPICT as CLIPFORMAT;
                fmt.tymed = TYMED_MFPICT;
                hr = IDataObject_GetData(pDataObject, addr_of_mut!(fmt), addr_of_mut!(med));
            }
            if hr == S_OK {
                hr = DataCacheEntry_SetData(
                    cache_entry,
                    addr_of!(fmt),
                    addr_of_mut!(med),
                    1,
                );
                if hr != S_OK {
                    ReleaseStgMedium(addr_of_mut!(med));
                } else {
                    done_one = 1;
                }
            }
        } else {
            let mut i: usize = 0;
            while i < view_list.len() {
                fmt.cfFormat = view_list[i];
                fmt.tymed = tymed_from_cf(fmt.cfFormat as DWORD);
                hr = IDataObject_QueryGetData(pDataObject, addr_of_mut!(fmt));
                if hr == S_OK {
                    hr = IDataObject_GetData(pDataObject, addr_of_mut!(fmt), addr_of_mut!(med));
                    if hr == S_OK {
                        if fmt.cfFormat as DWORD == CF_BITMAP {
                            (*cache_entry).fmtetc.cfFormat = CF_DIB as CLIPFORMAT;
                            (*cache_entry).fmtetc.tymed = TYMED_HGLOBAL;
                        } else {
                            (*cache_entry).fmtetc.cfFormat = fmt.cfFormat;
                            (*cache_entry).fmtetc.tymed = fmt.tymed;
                        }
                        hr = DataCacheEntry_SetData(
                            cache_entry,
                            addr_of!(fmt),
                            addr_of_mut!(med),
                            1,
                        );
                        if hr != S_OK {
                            ReleaseStgMedium(addr_of_mut!(med));
                        } else {
                            done_one = 1;
                        }
                        break;
                    }
                }
                i += 1;
            }
        }

        link = next;
    }

    if slots == 0 || done_one != 0 {
        S_OK
    } else {
        CACHE_E_NOCACHE_UPDATED
    }
}

unsafe fn DataCache_DiscardCache(iface: *mut IOleCache2, dwDiscardOptions: DWORD) -> HRESULT {
    let This = impl_from_IOleCache2(iface);
    let mut hr: HRESULT = S_OK;

    TRACE!(b"(%d)\n\0".as_ptr() as *const c_char, dwDiscardOptions);

    if dwDiscardOptions == DISCARDCACHE_SAVEIFDIRTY {
        hr = DataCache_Save(
            addr_of_mut!((*This).IPersistStorage_iface),
            (*This).presentationStorage,
            1,
        );
    }

    LIST_FOR_EACH_ENTRY!(
        cache_entry,
        addr_of_mut!((*This).cache_list),
        DataCacheEntry,
        entry,
        {
            hr = DataCacheEntry_DiscardData(cache_entry);
            if FAILED(hr) {
                break;
            }
        }
    );

    hr
}

// =====================================================================================
// IOleCacheControl
// =====================================================================================

unsafe fn DataCache_IOleCacheControl_QueryInterface(
    iface: *mut IOleCacheControl,
    riid: *const IID,
    ppvObject: *mut *mut c_void,
) -> HRESULT {
    let this = impl_from_IOleCacheControl(iface);
    IUnknown_QueryInterface((*this).outer_unk, riid, ppvObject)
}

unsafe fn DataCache_IOleCacheControl_AddRef(iface: *mut IOleCacheControl) -> ULONG {
    let this = impl_from_IOleCacheControl(iface);
    IUnknown_AddRef((*this).outer_unk)
}

unsafe fn DataCache_IOleCacheControl_Release(iface: *mut IOleCacheControl) -> ULONG {
    let this = impl_from_IOleCacheControl(iface);
    IUnknown_Release((*this).outer_unk)
}

unsafe fn DataCache_OnRun(iface: *mut IOleCacheControl, pDataObject: LPDATAOBJECT) -> HRESULT {
    let This = impl_from_IOleCacheControl(iface);

    TRACE!(
        b"(%p)->(%p)\n\0".as_ptr() as *const c_char,
        iface,
        pDataObject
    );

    if !(*This).running_object.is_null() {
        return S_OK;
    }

    /* No reference is taken on the data object */
    (*This).running_object = pDataObject;

    LIST_FOR_EACH_ENTRY!(
        cache_entry,
        addr_of_mut!((*This).cache_list),
        DataCacheEntry,
        entry,
        {
            setup_sink(This, cache_entry);
        }
    );

    S_OK
}

unsafe fn DataCache_OnStop(iface: *mut IOleCacheControl) -> HRESULT {
    let This = impl_from_IOleCacheControl(iface);

    TRACE!(b"(%p)\n\0".as_ptr() as *const c_char, iface);

    if (*This).running_object.is_null() {
        return S_OK;
    }

    LIST_FOR_EACH_ENTRY!(
        cache_entry,
        addr_of_mut!((*This).cache_list),
        DataCacheEntry,
        entry,
        {
            if (*cache_entry).sink_id != 0 {
                IDataObject_DUnadvise((*This).running_object, (*cache_entry).sink_id);
                (*cache_entry).sink_id = 0;
            }
        }
    );

    /* No ref taken in OnRun, so no Release call here */
    (*This).running_object = core::ptr::null_mut();
    S_OK
}

// =====================================================================================
// IAdviseSink — an internal object to the data cache. QI'ing its ptr doesn't give access
// to the cache's other interfaces. No ref count: it lives as long as the cache.
// =====================================================================================

unsafe fn DataCache_IAdviseSink_QueryInterface(
    iface: *mut IAdviseSink,
    riid: *const IID,
    ppvObject: *mut *mut c_void,
) -> HRESULT {
    *ppvObject = core::ptr::null_mut();
    if IsEqualIID(addr_of!(IID_IUnknown), riid) || IsEqualIID(addr_of!(IID_IAdviseSink), riid) {
        *ppvObject = iface as *mut c_void;
    }

    if !(*ppvObject).is_null() {
        IAdviseSink_AddRef(iface);
        return S_OK;
    }
    E_NOINTERFACE
}

unsafe fn DataCache_IAdviseSink_AddRef(iface: *mut IAdviseSink) -> ULONG {
    let _ = iface;
    2
}

unsafe fn DataCache_IAdviseSink_Release(iface: *mut IAdviseSink) -> ULONG {
    let _ = iface;
    1
}

unsafe fn DataCache_OnDataChange(
    iface: *mut IAdviseSink,
    pFormatetc: *mut FORMATETC,
    pStgmed: *mut STGMEDIUM,
) -> () {
    let This = impl_from_IAdviseSink(iface);
    TRACE!(
        b"(%p)->(%s, %p)\n\0".as_ptr() as *const c_char,
        This,
        debugstr_formatetc(pFormatetc),
        pStgmed
    );
    IOleCache2_SetData(
        addr_of_mut!((*This).IOleCache2_iface),
        pFormatetc,
        pStgmed,
        0,
    );
}

unsafe fn DataCache_OnViewChange(iface: *mut IAdviseSink, dwAspect: DWORD, lindex: LONG) -> () {
    let _ = (iface, dwAspect, lindex);
    FIXME!(b"stub\n\0".as_ptr() as *const c_char);
}

unsafe fn DataCache_OnRename(iface: *mut IAdviseSink, pmk: *mut IMoniker) -> () {
    let _ = (iface, pmk);
    FIXME!(b"stub\n\0".as_ptr() as *const c_char);
}

unsafe fn DataCache_OnSave(iface: *mut IAdviseSink) -> () {
    let _ = iface;
    FIXME!(b"stub\n\0".as_ptr() as *const c_char);
}

unsafe fn DataCache_OnClose(iface: *mut IAdviseSink) -> () {
    let _ = iface;
    FIXME!(b"stub\n\0".as_ptr() as *const c_char);
}

} // stdmethod!

// =====================================================================================
// Virtual function tables for the DataCache class.
// =====================================================================================

pub static DataCache_NDIUnknown_VTable: IUnknownVtbl = IUnknownVtbl {
    QueryInterface: Some(DataCache_NDIUnknown_QueryInterface),
    AddRef: Some(DataCache_NDIUnknown_AddRef),
    Release: Some(DataCache_NDIUnknown_Release),
};

pub static DataCache_IDataObject_VTable: IDataObjectVtbl = IDataObjectVtbl {
    QueryInterface: Some(DataCache_IDataObject_QueryInterface),
    AddRef: Some(DataCache_IDataObject_AddRef),
    Release: Some(DataCache_IDataObject_Release),
    GetData: Some(DataCache_GetData),
    GetDataHere: Some(DataCache_GetDataHere),
    QueryGetData: Some(DataCache_QueryGetData),
    GetCanonicalFormatEtc: Some(DataCache_GetCanonicalFormatEtc),
    SetData: Some(DataCache_IDataObject_SetData),
    EnumFormatEtc: Some(DataCache_EnumFormatEtc),
    DAdvise: Some(DataCache_DAdvise),
    DUnadvise: Some(DataCache_DUnadvise),
    EnumDAdvise: Some(DataCache_EnumDAdvise),
};

pub static DataCache_IPersistStorage_VTable: IPersistStorageVtbl = IPersistStorageVtbl {
    QueryInterface: Some(DataCache_IPersistStorage_QueryInterface),
    AddRef: Some(DataCache_IPersistStorage_AddRef),
    Release: Some(DataCache_IPersistStorage_Release),
    GetClassID: Some(DataCache_GetClassID),
    IsDirty: Some(DataCache_IsDirty),
    InitNew: Some(DataCache_InitNew),
    Load: Some(DataCache_Load),
    Save: Some(DataCache_Save),
    SaveCompleted: Some(DataCache_SaveCompleted),
    HandsOffStorage: Some(DataCache_HandsOffStorage),
};

pub static DataCache_IViewObject2_VTable: IViewObject2Vtbl = IViewObject2Vtbl {
    QueryInterface: Some(DataCache_IViewObject2_QueryInterface),
    AddRef: Some(DataCache_IViewObject2_AddRef),
    Release: Some(DataCache_IViewObject2_Release),
    Draw: Some(DataCache_Draw),
    GetColorSet: Some(DataCache_GetColorSet),
    Freeze: Some(DataCache_Freeze),
    Unfreeze: Some(DataCache_Unfreeze),
    SetAdvise: Some(DataCache_SetAdvise),
    GetAdvise: Some(DataCache_GetAdvise),
    GetExtent: Some(DataCache_GetExtent),
};

pub static DataCache_IOleCache2_VTable: IOleCache2Vtbl = IOleCache2Vtbl {
    QueryInterface: Some(DataCache_IOleCache2_QueryInterface),
    AddRef: Some(DataCache_IOleCache2_AddRef),
    Release: Some(DataCache_IOleCache2_Release),
    Cache: Some(DataCache_Cache),
    Uncache: Some(DataCache_Uncache),
    EnumCache: Some(DataCache_EnumCache),
    InitCache: Some(DataCache_InitCache),
    SetData: Some(DataCache_IOleCache2_SetData),
    UpdateCache: Some(DataCache_UpdateCache),
    DiscardCache: Some(DataCache_DiscardCache),
};

pub static DataCache_IOleCacheControl_VTable: IOleCacheControlVtbl = IOleCacheControlVtbl {
    QueryInterface: Some(DataCache_IOleCacheControl_QueryInterface),
    AddRef: Some(DataCache_IOleCacheControl_AddRef),
    Release: Some(DataCache_IOleCacheControl_Release),
    OnRun: Some(DataCache_OnRun),
    OnStop: Some(DataCache_OnStop),
};

pub static DataCache_IAdviseSink_VTable: IAdviseSinkVtbl = IAdviseSinkVtbl {
    QueryInterface: Some(DataCache_IAdviseSink_QueryInterface),
    AddRef: Some(DataCache_IAdviseSink_AddRef),
    Release: Some(DataCache_IAdviseSink_Release),
    OnDataChange: Some(DataCache_OnDataChange),
    OnViewChange: Some(DataCache_OnViewChange),
    OnRename: Some(DataCache_OnRename),
    OnSave: Some(DataCache_OnSave),
    OnClose: Some(DataCache_OnClose),
};

// =====================================================================================
// Method implementation for DataCache class.
// =====================================================================================

unsafe fn DataCache_Construct(clsid: *const CLSID, pUnkOuter: *mut IUnknown) -> *mut DataCache {
    let newObject: *mut DataCache = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<DataCache>() as SIZE_T,
    ) as *mut DataCache;

    if newObject.is_null() {
        return newObject;
    }

    (*newObject).IDataObject_iface.lpVtbl = addr_of!(DataCache_IDataObject_VTable) as *mut IDataObjectVtbl;
    (*newObject).IUnknown_inner.lpVtbl = addr_of!(DataCache_NDIUnknown_VTable) as *mut IUnknownVtbl;
    (*newObject).IPersistStorage_iface.lpVtbl = addr_of!(DataCache_IPersistStorage_VTable) as *mut IPersistStorageVtbl;
    (*newObject).IViewObject2_iface.lpVtbl = addr_of!(DataCache_IViewObject2_VTable) as *mut IViewObject2Vtbl;
    (*newObject).IOleCache2_iface.lpVtbl = addr_of!(DataCache_IOleCache2_VTable) as *mut IOleCache2Vtbl;
    (*newObject).IOleCacheControl_iface.lpVtbl = addr_of!(DataCache_IOleCacheControl_VTable) as *mut IOleCacheControlVtbl;
    (*newObject).IAdviseSink_iface.lpVtbl = addr_of!(DataCache_IAdviseSink_VTable) as *mut IAdviseSinkVtbl;
    (*newObject).outer_unk = if !pUnkOuter.is_null() {
        pUnkOuter
    } else {
        addr_of_mut!((*newObject).IUnknown_inner)
    };
    (*newObject).ref_ = 1;

    (*newObject).sinkAspects = 0;
    (*newObject).sinkAdviseFlag = 0;
    (*newObject).sinkInterface = core::ptr::null_mut();
    (*newObject).clsid = CLSID_NULL;
    (*newObject).clsid_static = 0;
    (*newObject).presentationStorage = core::ptr::null_mut();
    list_init(addr_of_mut!((*newObject).cache_list));
    (*newObject).last_cache_id = 2;
    (*newObject).dirty = 0;
    (*newObject).running_object = core::ptr::null_mut();

    create_automatic_entry(newObject, clsid);
    (*newObject).clsid = *clsid;

    newObject
}

/// datacache.c:3036 CreateDataCache [OLE32.@]
///
/// Creates a data cache to allow an object to render one or more of its views, whether
/// running or not.
#[no_mangle]
pub unsafe extern "system" fn CreateDataCache(
    pUnkOuter: *mut IUnknown,
    rclsid: *const CLSID,
    riid: *const IID,
    ppvObj: *mut LPVOID,
) -> HRESULT {
    TRACE!(
        b"(%s, %p, %s, %p)\n\0".as_ptr() as *const c_char,
        debugstr_guid(rclsid as *const c_void),
        pUnkOuter,
        debugstr_guid(riid as *const c_void),
        ppvObj,
    );

    if ppvObj.is_null() {
        return E_POINTER;
    }

    *ppvObj = core::ptr::null_mut();

    /*
     * If this cache is constructed for aggregation, make sure the caller is requesting the
     * IUnknown interface — it's the only time the non-delegating IUnknown pointer can be
     * returned to the outside.
     */
    if !pUnkOuter.is_null() && !IsEqualIID(addr_of!(IID_IUnknown), riid) {
        return E_INVALIDARG;
    }

    let newCache: *mut DataCache = DataCache_Construct(rclsid, pUnkOuter);

    if newCache.is_null() {
        return E_OUTOFMEMORY;
    }

    let hr = IUnknown_QueryInterface(addr_of_mut!((*newCache).IUnknown_inner), riid, ppvObj);
    IUnknown_Release(addr_of_mut!((*newCache).IUnknown_inner));

    hr
}
