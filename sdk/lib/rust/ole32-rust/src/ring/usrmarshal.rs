//! usrmarshal.c — the NDR user-marshal helpers (`*_UserSize` / `*_UserMarshal` /
//! `*_UserUnmarshal` / `*_UserFree`) plus the call_as/local proxy+stub thunks.
//!
//! ABI NOTE (load-bearing): every entry point here is `__RPC_USER` / `WINAPI` /
//! `CALLBACK` / `__RPC_STUB` in the C — all of which are `__stdcall` on i386. They
//! are exported from ole32.spec as `@ stdcall …`, and the widl-generated proxy
//! tables (ole32_{unknwn,objidl,oleidl}_p.c) call the _Proxy/_Stub thunks through
//! stdcall function pointers. So they are all `extern "system"`, never `extern "C"`
//! (which links on amd64/arm64 and silently breaks i386 — wrong decoration and a
//! stack imbalance on every call).
//!
//! WIRE-FORMAT NOTE: `ALIGN_LENGTH!` / `ALIGN_POINTER!` are ASSIGNMENT macros (as in
//! the C). Using them as expressions/calls and dropping the result shifts every
//! subsequent offset — see winemacros.rs.
use crate::prelude::*;
use core::ffi::{c_int, c_void};

extern "C" {
    /// wine/debug.h — genuinely cdecl varargs (a variadic Win32 fn is __cdecl even on i386).
    fn wine_dbg_sprintf(format: *const i8, ...) -> *const i8;
}

extern "system" {
    // wingdi.h — WINAPI. Missing from crate::gdi32 / crate::user32 (owed: hoist it there).
    fn SetMetaFileBitsEx(cbBuffer: UINT, lpData: *const u8) -> HMETAFILE;

    // widl-generated proxy thunks (add_rpcproxy_files: ole32_unknwn.idl /
    // ole32_objidl.idl / ole32_oleidl.idl). STDMETHODCALLTYPE → stdcall on i386.
    fn IClassFactory_RemoteCreateInstance_Proxy(
        This: *mut IClassFactory,
        riid: REFIID,
        ppvObject: *mut *mut IUnknown,
    ) -> HRESULT;
    fn IEnumUnknown_RemoteNext_Proxy(
        This: *mut IEnumUnknown,
        celt: ULONG,
        rgelt: *mut *mut IUnknown,
        pceltFetched: *mut ULONG,
    ) -> HRESULT;
    fn IEnumMoniker_RemoteNext_Proxy(
        This: *mut IEnumMoniker,
        celt: ULONG,
        rgelt: *mut *mut IMoniker,
        pceltFetched: *mut ULONG,
    ) -> HRESULT;
    fn IMoniker_RemoteBindToStorage_Proxy(
        This: *mut IMoniker,
        pbc: *mut IBindCtx,
        pmkToLeft: *mut IMoniker,
        riid: REFIID,
        ppvObj: *mut *mut IUnknown,
    ) -> HRESULT;
    fn IEnumString_RemoteNext_Proxy(
        This: *mut IEnumString,
        celt: ULONG,
        rgelt: *mut LPOLESTR,
        pceltFetched: *mut ULONG,
    ) -> HRESULT;
    fn ISequentialStream_RemoteRead_Proxy(
        This: *mut ISequentialStream,
        pv: *mut c_void,
        cb: ULONG,
        pcbRead: *mut ULONG,
    ) -> HRESULT;
    fn ISequentialStream_RemoteWrite_Proxy(
        This: *mut ISequentialStream,
        pv: *const c_void,
        cb: ULONG,
        pcbWritten: *mut ULONG,
    ) -> HRESULT;
    fn IStream_RemoteSeek_Proxy(
        This: *mut IStream,
        dlibMove: LARGE_INTEGER,
        dwOrigin: DWORD,
        plibNewPosition: *mut ULARGE_INTEGER,
    ) -> HRESULT;
    fn IStream_RemoteCopyTo_Proxy(
        This: *mut IStream,
        pstm: *mut IStream,
        cb: ULARGE_INTEGER,
        pcbRead: *mut ULARGE_INTEGER,
        pcbWritten: *mut ULARGE_INTEGER,
    ) -> HRESULT;
    fn IEnumSTATSTG_RemoteNext_Proxy(
        This: *mut IEnumSTATSTG,
        celt: ULONG,
        rgelt: *mut STATSTG,
        pceltFetched: *mut ULONG,
    ) -> HRESULT;
    fn IStorage_RemoteOpenStream_Proxy(
        This: *mut IStorage,
        pwcsName: LPCOLESTR,
        cbReserved1: ULONG,
        reserved1: *mut u8,
        grfMode: DWORD,
        reserved2: DWORD,
        ppstm: *mut *mut IStream,
    ) -> HRESULT;
    fn IStorage_RemoteEnumElements_Proxy(
        This: *mut IStorage,
        reserved1: DWORD,
        cbReserved2: ULONG,
        reserved2: *mut u8,
        reserved3: DWORD,
        ppenum: *mut *mut IEnumSTATSTG,
    ) -> HRESULT;
    fn ILockBytes_RemoteReadAt_Proxy(
        This: *mut ILockBytes,
        ulOffset: ULARGE_INTEGER,
        pv: *mut c_void,
        cb: ULONG,
        pcbRead: *mut ULONG,
    ) -> HRESULT;
    fn ILockBytes_RemoteWriteAt_Proxy(
        This: *mut ILockBytes,
        ulOffset: ULARGE_INTEGER,
        pv: *const c_void,
        cb: ULONG,
        pcbWritten: *mut ULONG,
    ) -> HRESULT;
    fn IFillLockBytes_RemoteFillAppend_Proxy(
        This: *mut IFillLockBytes,
        pv: *const c_void,
        cb: ULONG,
        pcbWritten: *mut ULONG,
    ) -> HRESULT;
    fn IFillLockBytes_RemoteFillAt_Proxy(
        This: *mut IFillLockBytes,
        ulOffset: ULARGE_INTEGER,
        pv: *const c_void,
        cb: ULONG,
        pcbWritten: *mut ULONG,
    ) -> HRESULT;
    fn IEnumFORMATETC_RemoteNext_Proxy(
        This: *mut IEnumFORMATETC,
        celt: ULONG,
        rgelt: *mut FORMATETC,
        pceltFetched: *mut ULONG,
    ) -> HRESULT;
    fn IEnumSTATDATA_RemoteNext_Proxy(
        This: *mut IEnumSTATDATA,
        celt: ULONG,
        rgelt: *mut STATDATA,
        pceltFetched: *mut ULONG,
    ) -> HRESULT;
    fn IAdviseSink_RemoteOnDataChange_Proxy(
        This: *mut IAdviseSink,
        pFormatetc: *mut FORMATETC,
        pStgmed: *mut ASYNC_STGMEDIUM,
    ) -> HRESULT;
    fn IAdviseSink_RemoteOnViewChange_Proxy(
        This: *mut IAdviseSink,
        dwAspect: DWORD,
        lindex: LONG,
    ) -> HRESULT;
    fn IAdviseSink_RemoteOnRename_Proxy(This: *mut IAdviseSink, pmk: *mut IMoniker) -> HRESULT;
    fn IAdviseSink_RemoteOnSave_Proxy(This: *mut IAdviseSink) -> HRESULT;
    fn IAdviseSink_RemoteOnClose_Proxy(This: *mut IAdviseSink) -> HRESULT;
    fn IAdviseSink2_RemoteOnLinkSrcChange_Proxy(
        This: *mut IAdviseSink2,
        pmk: *mut IMoniker,
    ) -> HRESULT;
    fn IDataObject_RemoteGetData_Proxy(
        This: *mut IDataObject,
        pformatetcIn: *mut FORMATETC,
        pRemoteMedium: *mut STGMEDIUM,
    ) -> HRESULT;
    fn IDataObject_RemoteGetDataHere_Proxy(
        This: *mut IDataObject,
        pformatetc: *mut FORMATETC,
        pRemoteMedium: *mut STGMEDIUM,
    ) -> HRESULT;
    fn IOleInPlaceActiveObject_RemoteTranslateAccelerator_Proxy(
        This: *mut IOleInPlaceActiveObject,
    ) -> HRESULT;
    fn IOleCache2_RemoteUpdateCache_Proxy(
        This: *mut IOleCache2,
        pDataObject: LPDATAOBJECT,
        grfUpdf: DWORD,
        pReserved: LONG_PTR,
    ) -> HRESULT;
    fn IEnumOLEVERB_RemoteNext_Proxy(
        This: *mut IEnumOLEVERB,
        celt: ULONG,
        rgelt: LPOLEVERB,
        pceltFetched: *mut ULONG,
    ) -> HRESULT;
}

// ---- file-scope preamble (translated) ----

/// usrmarshal.c: `USER_MARSHAL_PTR_PREFIX` — 'U' | 's'<<8 | 'e'<<16 | 'r'<<24.
pub const USER_MARSHAL_PTR_PREFIX: DWORD =
    ('U' as DWORD) | (('s' as DWORD) << 8) | (('e' as DWORD) << 16) | (('r' as DWORD) << 24);

/// rpcndr.h — NDR data-representation tag used only by the trace helper.
const NDR_LOCAL_DATA_REPRESENTATION: DWORD = 0x0000_0010;

/// wtypes.h `RemotableHandle` — the encapsulated union widl emits for
/// `union _RemotableHandle switch (long fContext) { WDT_INPROC_CALL: long hInproc;
/// WDT_REMOTE_CALL: long hRemote; }`.  Both arms are `long` (32-bit), NOT pointers.
#[repr(C)]
#[derive(Copy, Clone)]
pub union RemotableHandle_u {
    pub hInproc: LONG,
    pub hRemote: LONG,
}
#[repr(C)]
#[derive(Copy, Clone)]
pub struct RemotableHandle {
    pub fContext: LONG,
    pub u: RemotableHandle_u,
}

/// wingdi.h `tagBITMAP`. The first six fields (through `bmBitsPixel`, 20 bytes) are
/// exactly the prefix that HBITMAP_UserMarshal blits onto the wire — that prefix
/// length is `FIELD_OFFSET(userBITMAP, cbSize)`, so the two layouts must agree.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct BITMAP {
    pub bmType: LONG,
    pub bmWidth: LONG,
    pub bmHeight: LONG,
    pub bmWidthBytes: LONG,
    pub bmPlanes: WORD,
    pub bmBitsPixel: WORD,
    pub bmBits: LPVOID,
}

/// wtypes.idl `userBITMAP` — the wire form of a bitmap. `cbSize` sits at offset 20,
/// i.e. right after the 20-byte BITMAP header prefix that gets memcpy'd.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct userBITMAP {
    pub bmType: LONG,
    pub bmWidth: LONG,
    pub bmHeight: LONG,
    pub bmWidthBytes: LONG,
    pub bmPlanes: WORD,
    pub bmBitsPixel: WORD,
    pub cbSize: ULONG,
    // followed by `byte pBuffer[cbSize]` (conformant array — not modelled)
}

/// wtypes.idl `remoteMETAFILEPICT`. Only mm/xExt/yExt are ever touched here; the
/// buffer is advanced by `3 * sizeof(ULONG)` explicitly, so the trailing pointer's
/// alignment padding is irrelevant to the wire layout.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct remoteMETAFILEPICT {
    pub mm: LONG,
    pub xExt: LONG,
    pub yExt: LONG,
    pub hMF: *mut c_void, // userHMETAFILE *
}

/// objidl.idl: `typedef [wire_marshal(wireASYNC_STGMEDIUM)] STGMEDIUM ASYNC_STGMEDIUM;`
pub type ASYNC_STGMEDIUM = STGMEDIUM;

/// objidl.idl `_FLAG_STGMEDIUM`.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct FLAG_STGMEDIUM {
    pub ContextFlags: LONG,
    pub fPassOwnership: LONG,
    pub Stgmed: STGMEDIUM,
}

/// wingdi.h `HBRUSH` — missing from com-sys/user32; opaque handle (owed: hoist it into
/// the shared surface). Kept module-private so it does not leak through the prelude's
/// `ring::usrmarshal::*` glob and collide with ring/ole2.rs's own local HBRUSH.
/// (Transparent alias → still fine in the public fn signatures below.)
type HBRUSH = *mut c_void;

/// rpcndr.h `RPC_STATUS`. Module-private for the same reason (compobj/marshal each
/// carry their own).
type RPC_STATUS = LONG;

/// rpcndr.h `MIDL_STUB_MESSAGE` — PREFIX ONLY, up to and including `pfnFree`, which
/// is all SNB_Unmarshal/SNB_UserFree touch. The trailing ~40 fields are deliberately
/// not modelled: we only ever hold a `*mut MIDL_STUB_MESSAGE` handed to us by the RPC
/// runtime, never allocate one, so only the offsets of the fields above matter.
/// DO NOT `size_of::<MIDL_STUB_MESSAGE>()` — it is not the C size.
///
/// `pfnAllocate`/`pfnFree` are `__RPC_API`, i.e. __stdcall — hence `extern "system"`.
#[repr(C)]
pub struct MIDL_STUB_MESSAGE {
    pub RpcMsg: *mut c_void,
    pub Buffer: *mut u8,
    pub BufferStart: *mut u8,
    pub BufferEnd: *mut u8,
    pub BufferMark: *mut u8,
    pub BufferLength: ULONG,
    pub MemorySize: ULONG,
    pub Memory: *mut u8,
    pub IsClient: u8,
    pub Pad: u8,
    pub uFlags2: u16,
    pub ReuseBuffer: c_int,
    pub pAllocAllNodesContext: *mut c_void,
    pub pPointerQueueState: *mut c_void,
    pub IgnoreEmbeddedPointers: c_int,
    pub PointerBufferMark: *mut u8,
    pub CorrDespIncrement: u8,
    pub uFlags: u8,
    pub UniquePtrCount: u16,
    pub MaxCount: ULONG_PTR,
    pub Offset: ULONG,
    pub ActualCount: ULONG,
    pub pfnAllocate: Option<unsafe extern "system" fn(SIZE_T) -> *mut c_void>,
    pub pfnFree: Option<unsafe extern "system" fn(*mut c_void)>,
}

/// rpcndr.h `USER_MARSHAL_CB` — what the RPC runtime actually passes as `ULONG *pFlags`.
#[repr(C)]
pub struct USER_MARSHAL_CB {
    pub Flags: ULONG,
    pub pStubMsg: *mut MIDL_STUB_MESSAGE,
    pub pReserve: *const u8,
    pub Signature: ULONG,
    pub CBType: c_int,
    pub pFormat: *const u8,
    pub pTypeFormat: *const u8,
}

/// usrmarshal.c: the on-wire header of a marshalled SNB.
#[repr(C)]
pub struct SNB_wire {
    pub charcnt: ULONG,
    pub strcnt: ULONG,
    pub datalen: ULONG,
    pub data: [WCHAR; 1],
}

unsafe fn debugstr_user_flags(pFlags: *mut ULONG) -> *const i8 {
    let mut buf: [i8; 12] = [0; 12];
    let loword: *const i8;

    match LOWORD(*pFlags) as DWORD {
        MSHCTX_LOCAL => loword = b"MSHCTX_LOCAL\0".as_ptr() as *const i8,
        MSHCTX_NOSHAREDMEM => loword = b"MSHCTX_NOSHAREDMEM\0".as_ptr() as *const i8,
        MSHCTX_DIFFERENTMACHINE => loword = b"MSHCTX_DIFFERENTMACHINE\0".as_ptr() as *const i8,
        MSHCTX_INPROC => loword = b"MSHCTX_INPROC\0".as_ptr() as *const i8,
        _ => {
            sprintf(
                buf.as_mut_ptr(),
                b"%d\0".as_ptr() as *const i8,
                LOWORD(*pFlags) as c_int,
            );
            loword = buf.as_ptr();
        }
    }

    if HIWORD(*pFlags) as DWORD == NDR_LOCAL_DATA_REPRESENTATION {
        wine_dbg_sprintf(
            b"MAKELONG(%s, NDR_LOCAL_DATA_REPRESENTATION)\0".as_ptr() as *const i8,
            loword,
        )
    } else {
        wine_dbg_sprintf(
            b"MAKELONG(%s, 0x%04x)\0".as_ptr() as *const i8,
            loword,
            HIWORD(*pFlags) as c_int,
        )
    }
}

/******************************************************************************
 *           CLIPFORMAT_UserSize [OLE32.@]
 */
#[no_mangle]
pub unsafe extern "system" fn CLIPFORMAT_UserSize(
    pFlags: *mut ULONG,
    size: ULONG,
    pCF: *mut CLIPFORMAT,
) -> ULONG {
    TRACE!(
        b"(%s, %d, %p\n\0".as_ptr() as *const i8,
        debugstr_user_flags(pFlags),
        size,
        pCF,
    );

    let mut size = size;
    ALIGN_LENGTH!(size, 3);

    size += 8;

    /* only need to marshal the name if it is not a pre-defined type and
     * we are going remote */
    if (*pCF >= 0xc000) && (LOWORD(*pFlags) as DWORD == MSHCTX_DIFFERENTMACHINE) {
        let mut format: [WCHAR; 255] = [0; 255];
        size += 3 * core::mem::size_of::<UINT>() as ULONG;
        /* urg! this function is badly designed because it won't tell us how
         * much space is needed without doing a dummy run of storing the
         * name into a buffer */
        let ret = GetClipboardFormatNameW(
            *pCF as UINT,
            format.as_mut_ptr(),
            (format.len() - 1) as c_int,
        );
        if ret == 0 {
            RaiseException(DV_E_CLIPFORMAT as DWORD, 0, 0, core::ptr::null());
        }
        size += (ret + 1) as ULONG * core::mem::size_of::<WCHAR>() as ULONG;
    }
    size
}

/******************************************************************************
 *           CLIPFORMAT_UserMarshal [OLE32.@]
 */
#[no_mangle]
pub unsafe extern "system" fn CLIPFORMAT_UserMarshal(
    pFlags: *mut ULONG,
    mut pBuffer: *mut u8,
    pCF: *mut CLIPFORMAT,
) -> *mut u8 {
    TRACE!(
        b"(%s, %p, &0x%04x\n\0".as_ptr() as *const i8,
        debugstr_user_flags(pFlags),
        pBuffer,
        *pCF,
    );

    ALIGN_POINTER!(pBuffer, 3);

    if (*pCF >= 0xc000) && (LOWORD(*pFlags) as DWORD == MSHCTX_DIFFERENTMACHINE) {
        let mut format: [WCHAR; 255] = [0; 255];
        let mut len: UINT;

        *(pBuffer as *mut DWORD) = WDT_REMOTE_CALL;
        pBuffer = pBuffer.add(4);
        *(pBuffer as *mut DWORD) = *pCF as DWORD;
        pBuffer = pBuffer.add(4);

        len = GetClipboardFormatNameW(
            *pCF as UINT,
            format.as_mut_ptr(),
            (format.len() - 1) as c_int,
        ) as UINT;
        if len == 0 {
            RaiseException(DV_E_CLIPFORMAT as DWORD, 0, 0, core::ptr::null());
        }
        len += 1;
        *(pBuffer as *mut UINT) = len;
        pBuffer = pBuffer.add(core::mem::size_of::<UINT>());
        *(pBuffer as *mut UINT) = 0;
        pBuffer = pBuffer.add(core::mem::size_of::<UINT>());
        *(pBuffer as *mut UINT) = len;
        pBuffer = pBuffer.add(core::mem::size_of::<UINT>());
        TRACE!(
            b"marshaling format name %s\n\0".as_ptr() as *const i8,
            debugstr_w(format.as_ptr() as *const c_void),
        );
        memcpy(
            pBuffer as *mut c_void,
            format.as_ptr() as *const c_void,
            (len as usize) * core::mem::size_of::<WCHAR>(),
        );
        pBuffer = pBuffer.add((len as usize) * core::mem::size_of::<WCHAR>());
    } else {
        *(pBuffer as *mut DWORD) = WDT_INPROC_CALL;
        pBuffer = pBuffer.add(4);
        *(pBuffer as *mut DWORD) = *pCF as DWORD;
        pBuffer = pBuffer.add(4);
    }

    pBuffer
}

/******************************************************************************
 *           CLIPFORMAT_UserUnmarshal [OLE32.@]
 */
#[no_mangle]
pub unsafe extern "system" fn CLIPFORMAT_UserUnmarshal(
    _pFlags: *mut ULONG,
    mut pBuffer: *mut u8,
    pCF: *mut CLIPFORMAT,
) -> *mut u8 {
    TRACE!(
        b"(%s, %p, %p\n\0".as_ptr() as *const i8,
        debugstr_user_flags(pFlags),
        pBuffer,
        pCF,
    );

    ALIGN_POINTER!(pBuffer, 3);

    let fContext: LONG = *(pBuffer as *const DWORD) as LONG;
    pBuffer = pBuffer.add(4);

    if fContext == WDT_INPROC_CALL as LONG {
        *pCF = *(pBuffer as *const CLIPFORMAT);
        pBuffer = pBuffer.add(4);
    } else if fContext == WDT_REMOTE_CALL as LONG {
        let cf: CLIPFORMAT;
        let len: UINT;

        /* pointer ID for registered clip format string */
        if *(pBuffer as *const DWORD) == 0 {
            RaiseException(RPC_S_INVALID_BOUND, 0, 0, core::ptr::null());
        }
        pBuffer = pBuffer.add(4);

        len = *(pBuffer as *const UINT);
        pBuffer = pBuffer.add(core::mem::size_of::<UINT>());
        if *(pBuffer as *const UINT) != 0 {
            RaiseException(RPC_S_INVALID_BOUND, 0, 0, core::ptr::null());
        }
        pBuffer = pBuffer.add(core::mem::size_of::<UINT>());
        if *(pBuffer as *const UINT) != len {
            RaiseException(RPC_S_INVALID_BOUND, 0, 0, core::ptr::null());
        }
        pBuffer = pBuffer.add(core::mem::size_of::<UINT>());
        if *((pBuffer as *const WCHAR).add(len as usize - 1)) != 0 {
            RaiseException(RPC_S_INVALID_BOUND, 0, 0, core::ptr::null());
        }
        TRACE!(
            b"unmarshaling clip format %s\n\0".as_ptr() as *const i8,
            debugstr_w(pBuffer as *const c_void),
        );
        cf = RegisterClipboardFormatW(pBuffer as LPCWSTR) as CLIPFORMAT;
        pBuffer = pBuffer.add((len as usize) * core::mem::size_of::<WCHAR>());
        if cf == 0 {
            RaiseException(DV_E_CLIPFORMAT as DWORD, 0, 0, core::ptr::null());
        }
        *pCF = cf;
    } else {
        /* code not really appropriate, but nearest I can find */
        RaiseException(RPC_S_INVALID_TAG, 0, 0, core::ptr::null());
    }
    pBuffer
}

/******************************************************************************
 *           CLIPFORMAT_UserFree [OLE32.@]
 */
#[no_mangle]
pub unsafe extern "system" fn CLIPFORMAT_UserFree(_pFlags: *mut ULONG, _pCF: *mut CLIPFORMAT) {
    /* there is no inverse of the RegisterClipboardFormat function,
     * so nothing to do */
}

unsafe fn handle_UserSize(pFlags: *mut ULONG, StartingSize: ULONG, _handle: *mut HANDLE) -> ULONG {
    if LOWORD(*pFlags) as DWORD == MSHCTX_DIFFERENTMACHINE {
        ERR!(b"can't remote a local handle\n\0".as_ptr() as *const i8);
        RaiseException(RPC_S_INVALID_TAG, 0, 0, core::ptr::null());
        return StartingSize;
    }

    let mut StartingSize = StartingSize;
    ALIGN_LENGTH!(StartingSize, 3);
    StartingSize + core::mem::size_of::<RemotableHandle>() as ULONG
}

unsafe fn handle_UserMarshal(
    pFlags: *mut ULONG,
    mut pBuffer: *mut u8,
    handle: *mut HANDLE,
) -> *mut u8 {
    if LOWORD(*pFlags) as DWORD == MSHCTX_DIFFERENTMACHINE {
        ERR!(b"can't remote a local handle\n\0".as_ptr() as *const i8);
        RaiseException(RPC_S_INVALID_TAG, 0, 0, core::ptr::null());
        return pBuffer;
    }

    ALIGN_POINTER!(pBuffer, 3);
    let remhandle = pBuffer as *mut RemotableHandle;
    (*remhandle).fContext = WDT_INPROC_CALL as LONG;
    (*remhandle).u.hInproc = (*handle) as LONG_PTR as LONG;
    pBuffer.add(core::mem::size_of::<RemotableHandle>())
}

unsafe fn handle_UserUnmarshal(
    _pFlags: *mut ULONG,
    mut pBuffer: *mut u8,
    handle: *mut HANDLE,
) -> *mut u8 {
    ALIGN_POINTER!(pBuffer, 3);
    let remhandle = pBuffer as *mut RemotableHandle;
    if (*remhandle).fContext != WDT_INPROC_CALL as LONG {
        RaiseException(RPC_X_BAD_STUB_DATA, 0, 0, core::ptr::null());
    }
    *handle = (*remhandle).u.hInproc as LONG_PTR as HANDLE;
    pBuffer.add(core::mem::size_of::<RemotableHandle>())
}

unsafe fn handle_UserFree(_pFlags: *mut ULONG, _handle: *mut HANDLE) {
    /* nothing to do */
}

/// usrmarshal.c `IMPL_WIREM_HANDLE(type)` — the four exported thunks per handle type.
macro_rules! IMPL_WIREM_HANDLE {
    ($ty:ty, $size:ident, $marshal:ident, $unmarshal:ident, $free:ident) => {
        #[no_mangle]
        pub unsafe extern "system" fn $size(
            pFlags: *mut ULONG,
            StartingSize: ULONG,
            handle: *mut $ty,
        ) -> ULONG {
            TRACE!(
                b"(%s, %d, %p\n\0".as_ptr() as *const i8,
                debugstr_user_flags(pFlags),
                StartingSize,
                handle,
            );
            handle_UserSize(pFlags, StartingSize, handle as *mut HANDLE)
        }

        #[no_mangle]
        pub unsafe extern "system" fn $marshal(
            pFlags: *mut ULONG,
            pBuffer: *mut u8,
            handle: *mut $ty,
        ) -> *mut u8 {
            TRACE!(
                b"(%s, %p, &%p\n\0".as_ptr() as *const i8,
                debugstr_user_flags(pFlags),
                pBuffer,
                *handle,
            );
            handle_UserMarshal(pFlags, pBuffer, handle as *mut HANDLE)
        }

        #[no_mangle]
        pub unsafe extern "system" fn $unmarshal(
            pFlags: *mut ULONG,
            pBuffer: *mut u8,
            handle: *mut $ty,
        ) -> *mut u8 {
            TRACE!(
                b"(%s, %p, %p\n\0".as_ptr() as *const i8,
                debugstr_user_flags(pFlags),
                pBuffer,
                handle,
            );
            handle_UserUnmarshal(pFlags, pBuffer, handle as *mut HANDLE)
        }

        #[no_mangle]
        pub unsafe extern "system" fn $free(pFlags: *mut ULONG, handle: *mut $ty) {
            TRACE!(
                b"(%s, &%p\n\0".as_ptr() as *const i8,
                debugstr_user_flags(pFlags),
                *handle,
            );
            handle_UserFree(pFlags, handle as *mut HANDLE)
        }
    };
}

IMPL_WIREM_HANDLE!(
    HACCEL,
    HACCEL_UserSize,
    HACCEL_UserMarshal,
    HACCEL_UserUnmarshal,
    HACCEL_UserFree
);
IMPL_WIREM_HANDLE!(
    HMENU,
    HMENU_UserSize,
    HMENU_UserMarshal,
    HMENU_UserUnmarshal,
    HMENU_UserFree
);
IMPL_WIREM_HANDLE!(
    HWND,
    HWND_UserSize,
    HWND_UserMarshal,
    HWND_UserUnmarshal,
    HWND_UserFree
);
IMPL_WIREM_HANDLE!(
    HDC,
    HDC_UserSize,
    HDC_UserMarshal,
    HDC_UserUnmarshal,
    HDC_UserFree
);
IMPL_WIREM_HANDLE!(
    HICON,
    HICON_UserSize,
    HICON_UserMarshal,
    HICON_UserUnmarshal,
    HICON_UserFree
);
IMPL_WIREM_HANDLE!(
    HBRUSH,
    HBRUSH_UserSize,
    HBRUSH_UserMarshal,
    HBRUSH_UserUnmarshal,
    HBRUSH_UserFree
);

/******************************************************************************
 *           HGLOBAL_UserSize [OLE32.@]
 */
#[no_mangle]
pub unsafe extern "system" fn HGLOBAL_UserSize(
    pFlags: *mut ULONG,
    StartingSize: ULONG,
    phGlobal: *mut HGLOBAL,
) -> ULONG {
    let mut size: ULONG = StartingSize;

    TRACE!(
        b"(%s, %d, %p\n\0".as_ptr() as *const i8,
        debugstr_user_flags(pFlags),
        StartingSize,
        phGlobal,
    );

    ALIGN_LENGTH!(size, 3);

    size += core::mem::size_of::<ULONG>() as ULONG;

    if LOWORD(*pFlags) as DWORD == MSHCTX_INPROC {
        size += core::mem::size_of::<HGLOBAL>() as ULONG;
    } else {
        size += core::mem::size_of::<ULONG>() as ULONG;
        if !(*phGlobal).is_null() {
            size += 3 * core::mem::size_of::<ULONG>() as ULONG;
            let ret: SIZE_T = GlobalSize(*phGlobal);
            size += ret as ULONG;
        }
    }

    size
}

/******************************************************************************
 *           HGLOBAL_UserMarshal [OLE32.@]
 */
#[no_mangle]
pub unsafe extern "system" fn HGLOBAL_UserMarshal(
    pFlags: *mut ULONG,
    mut pBuffer: *mut u8,
    phGlobal: *mut HGLOBAL,
) -> *mut u8 {
    TRACE!(
        b"(%s, %p, &%p\n\0".as_ptr() as *const i8,
        debugstr_user_flags(pFlags),
        pBuffer,
        *phGlobal,
    );

    ALIGN_POINTER!(pBuffer, 3);

    if LOWORD(*pFlags) as DWORD == MSHCTX_INPROC {
        if core::mem::size_of::<HGLOBAL>() == 8 {
            *(pBuffer as *mut ULONG) = WDT_INPROC64_CALL;
        } else {
            *(pBuffer as *mut ULONG) = WDT_INPROC_CALL;
        }
        pBuffer = pBuffer.add(core::mem::size_of::<ULONG>());
        *(pBuffer as *mut HGLOBAL) = *phGlobal;
        pBuffer = pBuffer.add(core::mem::size_of::<HGLOBAL>());
    } else {
        *(pBuffer as *mut ULONG) = WDT_REMOTE_CALL;
        pBuffer = pBuffer.add(core::mem::size_of::<ULONG>());
        *(pBuffer as *mut ULONG) = HandleToULong(*phGlobal);
        pBuffer = pBuffer.add(core::mem::size_of::<ULONG>());
        if !(*phGlobal).is_null() {
            let size: SIZE_T = GlobalSize(*phGlobal);
            *(pBuffer as *mut ULONG) = size as ULONG;
            pBuffer = pBuffer.add(core::mem::size_of::<ULONG>());
            *(pBuffer as *mut ULONG) = HandleToULong(*phGlobal);
            pBuffer = pBuffer.add(core::mem::size_of::<ULONG>());
            *(pBuffer as *mut ULONG) = size as ULONG;
            pBuffer = pBuffer.add(core::mem::size_of::<ULONG>());

            let memory = GlobalLock(*phGlobal);
            memcpy(pBuffer as *mut c_void, memory as *const c_void, size as usize);
            pBuffer = pBuffer.add(size as usize);
            GlobalUnlock(*phGlobal);
        }
    }

    pBuffer
}

/******************************************************************************
 *           HGLOBAL_UserUnmarshal [OLE32.@]
 */
#[no_mangle]
pub unsafe extern "system" fn HGLOBAL_UserUnmarshal(
    _pFlags: *mut ULONG,
    mut pBuffer: *mut u8,
    phGlobal: *mut HGLOBAL,
) -> *mut u8 {
    TRACE!(
        b"(%s, %p, &%p\n\0".as_ptr() as *const i8,
        debugstr_user_flags(pFlags),
        pBuffer,
        *phGlobal,
    );

    ALIGN_POINTER!(pBuffer, 3);

    let fContext: ULONG = *(pBuffer as *const ULONG);
    pBuffer = pBuffer.add(core::mem::size_of::<ULONG>());

    if ((fContext == WDT_INPROC_CALL) && (core::mem::size_of::<HGLOBAL>() < 8))
        || ((fContext == WDT_INPROC64_CALL) && (core::mem::size_of::<HGLOBAL>() == 8))
    {
        *phGlobal = *(pBuffer as *const HGLOBAL);
        pBuffer = pBuffer.add(core::mem::size_of::<HGLOBAL>());
    } else if fContext == WDT_REMOTE_CALL {
        let handle: ULONG = *(pBuffer as *const ULONG);
        pBuffer = pBuffer.add(core::mem::size_of::<ULONG>());

        if handle != 0 {
            let size: ULONG = *(pBuffer as *const ULONG);
            pBuffer = pBuffer.add(core::mem::size_of::<ULONG>());
            /* redundancy is bad - it means you have to check consistency like this: */
            if *(pBuffer as *const ULONG) != handle {
                RaiseException(RPC_X_BAD_STUB_DATA, 0, 0, core::ptr::null());
                return pBuffer;
            }
            pBuffer = pBuffer.add(core::mem::size_of::<ULONG>());
            /* redundancy is bad - it means you have to check consistency like this: */
            if *(pBuffer as *const ULONG) != size {
                RaiseException(RPC_X_BAD_STUB_DATA, 0, 0, core::ptr::null());
                return pBuffer;
            }
            pBuffer = pBuffer.add(core::mem::size_of::<ULONG>());

            /* FIXME: check size is not too big */

            *phGlobal = GlobalAlloc(GMEM_MOVEABLE, size as SIZE_T);
            let memory = GlobalLock(*phGlobal);
            memcpy(memory, pBuffer as *const c_void, size as usize);
            pBuffer = pBuffer.add(size as usize);
            GlobalUnlock(*phGlobal);
        } else {
            *phGlobal = core::ptr::null_mut();
        }
    } else {
        RaiseException(RPC_S_INVALID_TAG, 0, 0, core::ptr::null());
    }

    pBuffer
}

/******************************************************************************
 *           HGLOBAL_UserFree [OLE32.@]
 */
#[no_mangle]
pub unsafe extern "system" fn HGLOBAL_UserFree(pFlags: *mut ULONG, phGlobal: *mut HGLOBAL) {
    TRACE!(
        b"(%s, &%p\n\0".as_ptr() as *const i8,
        debugstr_user_flags(pFlags),
        *phGlobal,
    );

    if LOWORD(*pFlags) as DWORD != MSHCTX_INPROC && !(*phGlobal).is_null() {
        GlobalFree(*phGlobal);
    }
}

/******************************************************************************
 *           HBITMAP_UserSize [OLE32.@]
 */
#[no_mangle]
pub unsafe extern "system" fn HBITMAP_UserSize(
    flags: *mut ULONG,
    size: ULONG,
    bmp: *mut HBITMAP,
) -> ULONG {
    TRACE!(
        b"(%s, %d, %p)\n\0".as_ptr() as *const i8,
        debugstr_user_flags(flags),
        size,
        *bmp,
    );

    let mut size = size;
    ALIGN_LENGTH!(size, 3);

    size += core::mem::size_of::<ULONG>() as ULONG;
    if LOWORD(*flags) as DWORD == MSHCTX_INPROC {
        size += core::mem::size_of::<ULONG>() as ULONG;
    } else {
        size += core::mem::size_of::<ULONG>() as ULONG;

        if !(*bmp).is_null() {
            size += core::mem::size_of::<ULONG>() as ULONG;
            size += FIELD_OFFSET!(userBITMAP, cbSize) as ULONG;
            size += GetBitmapBits(*bmp, 0, core::ptr::null_mut()) as ULONG;
        }
    }

    size
}

/******************************************************************************
 *           HBITMAP_UserMarshal [OLE32.@]
 */
#[no_mangle]
pub unsafe extern "system" fn HBITMAP_UserMarshal(
    flags: *mut ULONG,
    mut buffer: *mut u8,
    bmp: *mut HBITMAP,
) -> *mut u8 {
    TRACE!(
        b"(%s, %p, %p)\n\0".as_ptr() as *const i8,
        debugstr_user_flags(flags),
        buffer,
        *bmp,
    );

    ALIGN_POINTER!(buffer, 3);

    if LOWORD(*flags) as DWORD == MSHCTX_INPROC {
        *(buffer as *mut ULONG) = WDT_INPROC_CALL;
        buffer = buffer.add(core::mem::size_of::<ULONG>());
        *(buffer as *mut ULONG) = (*bmp) as ULONG_PTR as ULONG;
        buffer = buffer.add(core::mem::size_of::<ULONG>());
    } else {
        *(buffer as *mut ULONG) = WDT_REMOTE_CALL;
        buffer = buffer.add(core::mem::size_of::<ULONG>());
        *(buffer as *mut ULONG) = (*bmp) as ULONG_PTR as ULONG;
        buffer = buffer.add(core::mem::size_of::<ULONG>());

        if !(*bmp).is_null() {
            const HEADER_SIZE: ULONG = core::mem::offset_of!(userBITMAP, cbSize) as ULONG;
            let mut bitmap: BITMAP = core::mem::zeroed();
            let bitmap_size: ULONG;

            bitmap_size = GetBitmapBits(*bmp, 0, core::ptr::null_mut()) as ULONG;
            *(buffer as *mut ULONG) = bitmap_size;
            buffer = buffer.add(core::mem::size_of::<ULONG>());

            GetObjectW(
                (*bmp) as HANDLE,
                core::mem::size_of::<BITMAP>() as c_int,
                core::ptr::addr_of_mut!(bitmap) as LPVOID,
            );
            memcpy(
                buffer as *mut c_void,
                core::ptr::addr_of!(bitmap) as *const c_void,
                HEADER_SIZE as usize,
            );
            buffer = buffer.add(HEADER_SIZE as usize);

            GetBitmapBits(*bmp, bitmap_size as LONG, buffer as LPVOID);
            buffer = buffer.add(bitmap_size as usize);
        }
    }
    buffer
}

/******************************************************************************
 *           HBITMAP_UserUnmarshal [OLE32.@]
 */
#[no_mangle]
pub unsafe extern "system" fn HBITMAP_UserUnmarshal(
    _flags: *mut ULONG,
    mut buffer: *mut u8,
    bmp: *mut HBITMAP,
) -> *mut u8 {
    TRACE!(
        b"(%s, %p, %p)\n\0".as_ptr() as *const i8,
        debugstr_user_flags(flags),
        buffer,
        bmp,
    );

    ALIGN_POINTER!(buffer, 3);

    let context: ULONG = *(buffer as *const ULONG);
    buffer = buffer.add(core::mem::size_of::<ULONG>());

    if context == WDT_INPROC_CALL {
        *bmp = *(buffer as *const HBITMAP);
        buffer = buffer.add(core::mem::size_of::<HBITMAP>());
    } else if context == WDT_REMOTE_CALL {
        let handle: ULONG = *(buffer as *const ULONG);
        buffer = buffer.add(core::mem::size_of::<ULONG>());

        if handle != 0 {
            const HEADER_SIZE: ULONG = core::mem::offset_of!(userBITMAP, cbSize) as ULONG;
            let mut bitmap: BITMAP = core::mem::zeroed();
            let bitmap_size: ULONG;

            bitmap_size = *(buffer as *const ULONG);
            buffer = buffer.add(core::mem::size_of::<ULONG>());
            let bits = HeapAlloc(GetProcessHeap(), 0, bitmap_size as usize);

            memcpy(
                core::ptr::addr_of_mut!(bitmap) as *mut c_void,
                buffer as *const c_void,
                HEADER_SIZE as usize,
            );
            buffer = buffer.add(HEADER_SIZE as usize);

            memcpy(bits, buffer as *const c_void, bitmap_size as usize);
            buffer = buffer.add(bitmap_size as usize);

            bitmap.bmBits = bits;
            *bmp = (CreateBitmapIndirect((core::ptr::addr_of!(bitmap) as *const c_void).cast())).cast();

            HeapFree(GetProcessHeap(), 0, bits);
        } else {
            *bmp = core::ptr::null_mut();
        }
    } else {
        RaiseException(RPC_S_INVALID_TAG, 0, 0, core::ptr::null());
    }

    buffer
}

/******************************************************************************
 *           HBITMAP_UserFree [OLE32.@]
 */
#[no_mangle]
pub unsafe extern "system" fn HBITMAP_UserFree(flags: *mut ULONG, bmp: *mut HBITMAP) {
    TRACE!(
        b"(%s, %p)\n\0".as_ptr() as *const i8,
        debugstr_user_flags(flags),
        *bmp,
    );

    if LOWORD(*flags) as DWORD != MSHCTX_INPROC {
        DeleteObject(*bmp as HGDIOBJ);
    }
}

/******************************************************************************
 *           HPALETTE_UserSize [OLE32.@]
 */
#[no_mangle]
pub unsafe extern "system" fn HPALETTE_UserSize(
    _pFlags: *mut ULONG,
    StartingSize: ULONG,
    _phPal: *mut crate::user32::HPALETTE,
) -> ULONG {
    FIXME!(b":stub\n\0".as_ptr() as *const i8);
    StartingSize
}

/******************************************************************************
 *           HPALETTE_UserMarshal [OLE32.@]
 */
#[no_mangle]
pub unsafe extern "system" fn HPALETTE_UserMarshal(
    _pFlags: *mut ULONG,
    pBuffer: *mut u8,
    _phPal: *mut crate::user32::HPALETTE,
) -> *mut u8 {
    FIXME!(b":stub\n\0".as_ptr() as *const i8);
    pBuffer
}

/******************************************************************************
 *           HPALETTE_UserUnmarshal [OLE32.@]
 */
#[no_mangle]
pub unsafe extern "system" fn HPALETTE_UserUnmarshal(
    _pFlags: *mut ULONG,
    pBuffer: *mut u8,
    _phPal: *mut crate::user32::HPALETTE,
) -> *mut u8 {
    FIXME!(b":stub\n\0".as_ptr() as *const i8);
    pBuffer
}

/******************************************************************************
 *           HPALETTE_UserFree [OLE32.@]
 */
#[no_mangle]
pub unsafe extern "system" fn HPALETTE_UserFree(
    _pFlags: *mut ULONG,
    _phPal: *mut crate::user32::HPALETTE,
) {
    FIXME!(b":stub\n\0".as_ptr() as *const i8);
}

/******************************************************************************
 *           HMETAFILE_UserSize [OLE32.@]
 */
#[no_mangle]
pub unsafe extern "system" fn HMETAFILE_UserSize(
    pFlags: *mut ULONG,
    StartingSize: ULONG,
    phmf: *mut HMETAFILE,
) -> ULONG {
    let mut size: ULONG = StartingSize;

    TRACE!(
        b"(%s, %d, &%p\n\0".as_ptr() as *const i8,
        debugstr_user_flags(pFlags),
        StartingSize,
        *phmf,
    );

    ALIGN_LENGTH!(size, 3);

    size += core::mem::size_of::<ULONG>() as ULONG;
    if LOWORD(*pFlags) as DWORD == MSHCTX_INPROC {
        size += core::mem::size_of::<ULONG_PTR>() as ULONG;
    } else {
        size += core::mem::size_of::<ULONG>() as ULONG;

        if !(*phmf).is_null() {
            let mfsize: UINT;

            size += 2 * core::mem::size_of::<ULONG>() as ULONG;
            mfsize = GetMetaFileBitsEx(*phmf as HANDLE, 0, core::ptr::null_mut());
            size += mfsize;
        }
    }

    size
}

/******************************************************************************
 *           HMETAFILE_UserMarshal [OLE32.@]
 */
#[no_mangle]
pub unsafe extern "system" fn HMETAFILE_UserMarshal(
    pFlags: *mut ULONG,
    mut pBuffer: *mut u8,
    phmf: *mut HMETAFILE,
) -> *mut u8 {
    TRACE!(
        b"(%s, %p, &%p\n\0".as_ptr() as *const i8,
        debugstr_user_flags(pFlags),
        pBuffer,
        *phmf,
    );

    ALIGN_POINTER!(pBuffer, 3);

    if LOWORD(*pFlags) as DWORD == MSHCTX_INPROC {
        if core::mem::size_of::<HMETAFILE>() == 8 {
            *(pBuffer as *mut ULONG) = WDT_INPROC64_CALL;
        } else {
            *(pBuffer as *mut ULONG) = WDT_INPROC_CALL;
        }
        pBuffer = pBuffer.add(core::mem::size_of::<ULONG>());
        *(pBuffer as *mut HMETAFILE) = *phmf;
        pBuffer = pBuffer.add(core::mem::size_of::<HMETAFILE>());
    } else {
        *(pBuffer as *mut ULONG) = WDT_REMOTE_CALL;
        pBuffer = pBuffer.add(core::mem::size_of::<ULONG>());
        *(pBuffer as *mut ULONG) = (*phmf) as ULONG_PTR as ULONG;
        pBuffer = pBuffer.add(core::mem::size_of::<ULONG>());

        if !(*phmf).is_null() {
            let mfsize: UINT = GetMetaFileBitsEx(*phmf as HANDLE, 0, core::ptr::null_mut());

            *(pBuffer as *mut ULONG) = mfsize;
            pBuffer = pBuffer.add(core::mem::size_of::<ULONG>());
            *(pBuffer as *mut ULONG) = mfsize;
            pBuffer = pBuffer.add(core::mem::size_of::<ULONG>());
            GetMetaFileBitsEx(*phmf as HANDLE, mfsize, pBuffer as LPVOID);
            pBuffer = pBuffer.add(mfsize as usize);
        }
    }

    pBuffer
}

/******************************************************************************
 *           HMETAFILE_UserUnmarshal [OLE32.@]
 */
#[no_mangle]
pub unsafe extern "system" fn HMETAFILE_UserUnmarshal(
    _pFlags: *mut ULONG,
    mut pBuffer: *mut u8,
    phmf: *mut HMETAFILE,
) -> *mut u8 {
    TRACE!(
        b"(%s, %p, %p\n\0".as_ptr() as *const i8,
        debugstr_user_flags(pFlags),
        pBuffer,
        phmf,
    );

    ALIGN_POINTER!(pBuffer, 3);

    let fContext: ULONG = *(pBuffer as *const ULONG);
    pBuffer = pBuffer.add(core::mem::size_of::<ULONG>());

    if ((fContext == WDT_INPROC_CALL) && (core::mem::size_of::<HMETAFILE>() < 8))
        || ((fContext == WDT_INPROC64_CALL) && (core::mem::size_of::<HMETAFILE>() == 8))
    {
        *phmf = *(pBuffer as *const HMETAFILE);
        pBuffer = pBuffer.add(core::mem::size_of::<HMETAFILE>());
    } else if fContext == WDT_REMOTE_CALL {
        let handle: ULONG = *(pBuffer as *const ULONG);
        pBuffer = pBuffer.add(core::mem::size_of::<ULONG>());

        if handle != 0 {
            let size: ULONG = *(pBuffer as *const ULONG);
            pBuffer = pBuffer.add(core::mem::size_of::<ULONG>());
            if size != *(pBuffer as *const ULONG) {
                RaiseException(RPC_X_BAD_STUB_DATA, 0, 0, core::ptr::null());
                return pBuffer;
            }
            pBuffer = pBuffer.add(core::mem::size_of::<ULONG>());
            *phmf = SetMetaFileBitsEx(size, pBuffer);
            pBuffer = pBuffer.add(size as usize);
        } else {
            *phmf = core::ptr::null_mut();
        }
    } else {
        RaiseException(RPC_S_INVALID_TAG, 0, 0, core::ptr::null());
    }

    pBuffer
}

/******************************************************************************
 *           HMETAFILE_UserFree [OLE32.@]
 */
#[no_mangle]
pub unsafe extern "system" fn HMETAFILE_UserFree(pFlags: *mut ULONG, phmf: *mut HMETAFILE) {
    TRACE!(
        b"(%s, &%p\n\0".as_ptr() as *const i8,
        debugstr_user_flags(pFlags),
        *phmf,
    );

    if LOWORD(*pFlags) as DWORD != MSHCTX_INPROC {
        DeleteMetaFile(*phmf);
    }
}

/******************************************************************************
 *           HENHMETAFILE_UserSize [OLE32.@]
 */
#[no_mangle]
pub unsafe extern "system" fn HENHMETAFILE_UserSize(
    pFlags: *mut ULONG,
    size: ULONG,
    phEmf: *mut crate::user32::HENHMETAFILE,
) -> ULONG {
    TRACE!(
        b"(%s, %d, %p\n\0".as_ptr() as *const i8,
        debugstr_user_flags(pFlags),
        size,
        *phEmf,
    );

    let mut size = size;
    ALIGN_LENGTH!(size, 3);

    size += core::mem::size_of::<ULONG>() as ULONG;
    if LOWORD(*pFlags) as DWORD == MSHCTX_INPROC {
        size += core::mem::size_of::<ULONG_PTR>() as ULONG;
    } else {
        size += core::mem::size_of::<ULONG>() as ULONG;

        if !(*phEmf).is_null() {
            let emfsize: UINT;

            size += 2 * core::mem::size_of::<ULONG>() as ULONG;
            emfsize = GetEnhMetaFileBits(*phEmf, 0, core::ptr::null_mut());
            size += emfsize;
        }
    }

    size
}

/******************************************************************************
 *           HENHMETAFILE_UserMarshal [OLE32.@]
 */
#[no_mangle]
pub unsafe extern "system" fn HENHMETAFILE_UserMarshal(
    pFlags: *mut ULONG,
    mut pBuffer: *mut u8,
    phEmf: *mut crate::user32::HENHMETAFILE,
) -> *mut u8 {
    TRACE!(
        b"(%s, %p, &%p\n\0".as_ptr() as *const i8,
        debugstr_user_flags(pFlags),
        pBuffer,
        *phEmf,
    );

    ALIGN_POINTER!(pBuffer, 3);

    if LOWORD(*pFlags) as DWORD == MSHCTX_INPROC {
        if core::mem::size_of::<crate::user32::HENHMETAFILE>() == 8 {
            *(pBuffer as *mut ULONG) = WDT_INPROC64_CALL;
        } else {
            *(pBuffer as *mut ULONG) = WDT_INPROC_CALL;
        }
        pBuffer = pBuffer.add(core::mem::size_of::<ULONG>());
        *(pBuffer as *mut crate::user32::HENHMETAFILE) = *phEmf;
        pBuffer = pBuffer.add(core::mem::size_of::<crate::user32::HENHMETAFILE>());
    } else {
        *(pBuffer as *mut ULONG) = WDT_REMOTE_CALL;
        pBuffer = pBuffer.add(core::mem::size_of::<ULONG>());
        *(pBuffer as *mut ULONG) = (*phEmf) as ULONG_PTR as ULONG;
        pBuffer = pBuffer.add(core::mem::size_of::<ULONG>());

        if !(*phEmf).is_null() {
            let emfsize: UINT = GetEnhMetaFileBits(*phEmf, 0, core::ptr::null_mut());

            *(pBuffer as *mut ULONG) = emfsize;
            pBuffer = pBuffer.add(core::mem::size_of::<ULONG>());
            *(pBuffer as *mut ULONG) = emfsize;
            pBuffer = pBuffer.add(core::mem::size_of::<ULONG>());
            GetEnhMetaFileBits(*phEmf, emfsize, pBuffer);
            pBuffer = pBuffer.add(emfsize as usize);
        }
    }

    pBuffer
}

/******************************************************************************
 *           HENHMETAFILE_UserUnmarshal [OLE32.@]
 */
#[no_mangle]
pub unsafe extern "system" fn HENHMETAFILE_UserUnmarshal(
    _pFlags: *mut ULONG,
    mut pBuffer: *mut u8,
    phEmf: *mut crate::user32::HENHMETAFILE,
) -> *mut u8 {
    TRACE!(
        b"(%s, %p, %p\n\0".as_ptr() as *const i8,
        debugstr_user_flags(pFlags),
        pBuffer,
        phEmf,
    );

    ALIGN_POINTER!(pBuffer, 3);

    let fContext: ULONG = *(pBuffer as *const ULONG);
    pBuffer = pBuffer.add(core::mem::size_of::<ULONG>());

    if ((fContext == WDT_INPROC_CALL)
        && (core::mem::size_of::<crate::user32::HENHMETAFILE>() < 8))
        || ((fContext == WDT_INPROC64_CALL)
            && (core::mem::size_of::<crate::user32::HENHMETAFILE>() == 8))
    {
        *phEmf = *(pBuffer as *const crate::user32::HENHMETAFILE);
        pBuffer = pBuffer.add(core::mem::size_of::<crate::user32::HENHMETAFILE>());
    } else if fContext == WDT_REMOTE_CALL {
        let handle: ULONG = *(pBuffer as *const ULONG);
        pBuffer = pBuffer.add(core::mem::size_of::<ULONG>());

        if handle != 0 {
            let size: ULONG = *(pBuffer as *const ULONG);
            pBuffer = pBuffer.add(core::mem::size_of::<ULONG>());
            if size != *(pBuffer as *const ULONG) {
                RaiseException(RPC_X_BAD_STUB_DATA, 0, 0, core::ptr::null());
                return pBuffer;
            }
            pBuffer = pBuffer.add(core::mem::size_of::<ULONG>());
            *phEmf = SetEnhMetaFileBits(size, pBuffer);
            pBuffer = pBuffer.add(size as usize);
        } else {
            *phEmf = core::ptr::null_mut();
        }
    } else {
        RaiseException(RPC_S_INVALID_TAG, 0, 0, core::ptr::null());
    }

    pBuffer
}

/******************************************************************************
 *           HENHMETAFILE_UserFree [OLE32.@]
 */
#[no_mangle]
pub unsafe extern "system" fn HENHMETAFILE_UserFree(
    pFlags: *mut ULONG,
    phEmf: *mut crate::user32::HENHMETAFILE,
) {
    TRACE!(
        b"(%s, &%p\n\0".as_ptr() as *const i8,
        debugstr_user_flags(pFlags),
        *phEmf,
    );

    if LOWORD(*pFlags) as DWORD != MSHCTX_INPROC {
        DeleteEnhMetaFile(*phEmf);
    }
}

/******************************************************************************
 *           HMETAFILEPICT_UserSize [OLE32.@]
 */
#[no_mangle]
pub unsafe extern "system" fn HMETAFILEPICT_UserSize(
    pFlags: *mut ULONG,
    size: ULONG,
    phMfp: *mut HMETAFILEPICT,
) -> ULONG {
    TRACE!(
        b"(%s, %d, &%p)\n\0".as_ptr() as *const i8,
        debugstr_user_flags(pFlags),
        size,
        *phMfp,
    );

    let mut size = size;
    ALIGN_LENGTH!(size, 3);

    size += core::mem::size_of::<ULONG>() as ULONG;

    if LOWORD(*pFlags) as DWORD == MSHCTX_INPROC {
        size += core::mem::size_of::<HMETAFILEPICT>() as ULONG;
    } else {
        size += core::mem::size_of::<ULONG>() as ULONG;

        if !(*phMfp).is_null() {
            let mfpict = GlobalLock(*phMfp) as *mut METAFILEPICT;

            /* FIXME: raise an exception if mfpict is NULL? */
            size += 3 * core::mem::size_of::<ULONG>() as ULONG;
            size += core::mem::size_of::<ULONG>() as ULONG;

            size = HMETAFILE_UserSize(pFlags, size, core::ptr::addr_of_mut!((*mfpict).hMF));

            GlobalUnlock(*phMfp);
        }
    }

    size
}

/******************************************************************************
 *           HMETAFILEPICT_UserMarshal [OLE32.@]
 */
#[no_mangle]
pub unsafe extern "system" fn HMETAFILEPICT_UserMarshal(
    pFlags: *mut ULONG,
    mut pBuffer: *mut u8,
    phMfp: *mut HMETAFILEPICT,
) -> *mut u8 {
    TRACE!(
        b"(%s, %p, &%p)\n\0".as_ptr() as *const i8,
        debugstr_user_flags(pFlags),
        pBuffer,
        *phMfp,
    );

    ALIGN_POINTER!(pBuffer, 3);

    if LOWORD(*pFlags) as DWORD == MSHCTX_INPROC {
        if core::mem::size_of::<HMETAFILEPICT>() == 8 {
            *(pBuffer as *mut ULONG) = WDT_INPROC64_CALL;
        } else {
            *(pBuffer as *mut ULONG) = WDT_INPROC_CALL;
        }
        pBuffer = pBuffer.add(core::mem::size_of::<ULONG>());
        *(pBuffer as *mut HMETAFILEPICT) = *phMfp;
        pBuffer = pBuffer.add(core::mem::size_of::<HMETAFILEPICT>());
    } else {
        *(pBuffer as *mut ULONG) = WDT_REMOTE_CALL;
        pBuffer = pBuffer.add(core::mem::size_of::<ULONG>());
        *(pBuffer as *mut ULONG) = (*phMfp) as ULONG_PTR as ULONG;
        pBuffer = pBuffer.add(core::mem::size_of::<ULONG>());

        if !(*phMfp).is_null() {
            let mfpict = GlobalLock(*phMfp) as *mut METAFILEPICT;
            let remmfpict = pBuffer as *mut remoteMETAFILEPICT;

            /* FIXME: raise an exception if mfpict is NULL? */
            (*remmfpict).mm = (*mfpict).mm;
            (*remmfpict).xExt = (*mfpict).xExt;
            (*remmfpict).yExt = (*mfpict).yExt;
            pBuffer = pBuffer.add(3 * core::mem::size_of::<ULONG>());
            *(pBuffer as *mut ULONG) = USER_MARSHAL_PTR_PREFIX;
            pBuffer = pBuffer.add(core::mem::size_of::<ULONG>());

            pBuffer = HMETAFILE_UserMarshal(pFlags, pBuffer, core::ptr::addr_of_mut!((*mfpict).hMF));

            GlobalUnlock(*phMfp);
        }
    }
    pBuffer
}

/******************************************************************************
 *           HMETAFILEPICT_UserUnmarshal [OLE32.@]
 */
#[no_mangle]
pub unsafe extern "system" fn HMETAFILEPICT_UserUnmarshal(
    pFlags: *mut ULONG,
    mut pBuffer: *mut u8,
    phMfp: *mut HMETAFILEPICT,
) -> *mut u8 {
    TRACE!(
        b"(%s, %p, %p)\n\0".as_ptr() as *const i8,
        debugstr_user_flags(pFlags),
        pBuffer,
        phMfp,
    );

    ALIGN_POINTER!(pBuffer, 3);

    let fContext: ULONG = *(pBuffer as *const ULONG);
    pBuffer = pBuffer.add(core::mem::size_of::<ULONG>());

    if (fContext == WDT_INPROC_CALL) || (fContext == WDT_INPROC64_CALL) {
        *phMfp = *(pBuffer as *const HMETAFILEPICT);
        pBuffer = pBuffer.add(core::mem::size_of::<HMETAFILEPICT>());
    } else {
        let handle: ULONG = *(pBuffer as *const ULONG);
        pBuffer = pBuffer.add(core::mem::size_of::<ULONG>());
        *phMfp = core::ptr::null_mut();

        if handle != 0 {
            let remmfpict = pBuffer as *const remoteMETAFILEPICT;

            *phMfp = GlobalAlloc(
                GMEM_MOVEABLE,
                core::mem::size_of::<METAFILEPICT>() as SIZE_T,
            );
            if (*phMfp).is_null() {
                RpcRaiseException(E_OUTOFMEMORY);
            }

            let mfpict = GlobalLock(*phMfp) as *mut METAFILEPICT;
            (*mfpict).mm = (*remmfpict).mm;
            (*mfpict).xExt = (*remmfpict).xExt;
            (*mfpict).yExt = (*remmfpict).yExt;
            pBuffer = pBuffer.add(3 * core::mem::size_of::<ULONG>());
            let user_marshal_prefix: ULONG = *(pBuffer as *const ULONG);
            pBuffer = pBuffer.add(core::mem::size_of::<ULONG>());

            if user_marshal_prefix != USER_MARSHAL_PTR_PREFIX {
                RpcRaiseException(RPC_X_INVALID_TAG as RPC_STATUS);
            }

            pBuffer =
                HMETAFILE_UserUnmarshal(pFlags, pBuffer, core::ptr::addr_of_mut!((*mfpict).hMF));

            GlobalUnlock(*phMfp);
        }
    }
    pBuffer
}

/******************************************************************************
 *           HMETAFILEPICT_UserFree [OLE32.@]
 */
#[no_mangle]
pub unsafe extern "system" fn HMETAFILEPICT_UserFree(
    pFlags: *mut ULONG,
    phMfp: *mut HMETAFILEPICT,
) {
    TRACE!(
        b"(%s, &%p)\n\0".as_ptr() as *const i8,
        debugstr_user_flags(pFlags),
        *phMfp,
    );

    if (LOWORD(*pFlags) as DWORD != MSHCTX_INPROC) && !(*phMfp).is_null() {
        let mfpict: *mut METAFILEPICT;

        mfpict = GlobalLock(*phMfp) as *mut METAFILEPICT;
        /* FIXME: raise an exception if mfpict is NULL? */
        HMETAFILE_UserFree(pFlags, core::ptr::addr_of_mut!((*mfpict).hMF));
        GlobalUnlock(*phMfp);

        GlobalFree(*phMfp);
    }
}

/******************************************************************************
 *           WdtpInterfacePointer_UserSize [OLE32.@]
 */
#[no_mangle]
pub unsafe extern "system" fn WdtpInterfacePointer_UserSize(
    _pFlags: *mut ULONG,
    RealFlags: ULONG,
    StartingSize: ULONG,
    punk: *mut IUnknown,
    riid: REFIID,
) -> ULONG {
    let mut marshal_size: DWORD = 0;
    let hr: HRESULT;

    TRACE!(
        b"(%s, 0%x, %d, %p, %s)\n\0".as_ptr() as *const i8,
        debugstr_user_flags(pFlags),
        RealFlags,
        StartingSize,
        punk,
        debugstr_guid(riid as *const c_void),
    );

    hr = CoGetMarshalSizeMax(
        &mut marshal_size,
        riid,
        punk,
        LOWORD(RealFlags) as DWORD,
        core::ptr::null_mut(),
        tagMSHLFLAGS_MSHLFLAGS_NORMAL as DWORD,
    );
    if FAILED(hr) {
        return StartingSize;
    }

    let mut StartingSize = StartingSize;
    ALIGN_LENGTH!(StartingSize, 3);
    StartingSize += 2 * core::mem::size_of::<DWORD>() as ULONG;
    StartingSize + marshal_size
}

/******************************************************************************
 *           WdtpInterfacePointer_UserMarshal [OLE32.@]
 */
#[no_mangle]
pub unsafe extern "system" fn WdtpInterfacePointer_UserMarshal(
    _pFlags: *mut ULONG,
    RealFlags: ULONG,
    mut pBuffer: *mut u8,
    punk: *mut IUnknown,
    riid: REFIID,
) -> *mut u8 {
    let h: HGLOBAL = GlobalAlloc(GMEM_MOVEABLE, 0);
    let mut stm: *mut IStream = core::ptr::null_mut();
    let size: DWORD;
    let ptr: LPVOID;

    TRACE!(
        b"(%s, 0x%x, %p, &%p, %s)\n\0".as_ptr() as *const i8,
        debugstr_user_flags(pFlags),
        RealFlags,
        pBuffer,
        punk,
        debugstr_guid(riid as *const c_void),
    );

    if h.is_null() {
        return core::ptr::null_mut();
    }
    if CreateStreamOnHGlobal(h, 1, &mut stm) != S_OK {
        GlobalFree(h);
        return core::ptr::null_mut();
    }

    if CoMarshalInterface(
        (stm).cast(),
        (riid).cast(),
        (punk).cast(),
        LOWORD(RealFlags) as DWORD,
        core::ptr::null_mut(),
        tagMSHLFLAGS_MSHLFLAGS_NORMAL as DWORD,
    ) != S_OK
    {
        IStream_Release(stm);
        return pBuffer;
    }

    ALIGN_POINTER!(pBuffer, 3);
    size = GlobalSize(h) as DWORD;

    *(pBuffer as *mut DWORD) = size;
    pBuffer = pBuffer.add(core::mem::size_of::<DWORD>());
    *(pBuffer as *mut DWORD) = size;
    pBuffer = pBuffer.add(core::mem::size_of::<DWORD>());

    ptr = GlobalLock(h);
    memcpy(pBuffer as *mut c_void, ptr as *const c_void, size as usize);
    GlobalUnlock(h);

    IStream_Release(stm);
    pBuffer.add(size as usize)
}

/******************************************************************************
 *           WdtpInterfacePointer_UserUnmarshal [OLE32.@]
 */
#[no_mangle]
pub unsafe extern "system" fn WdtpInterfacePointer_UserUnmarshal(
    _pFlags: *mut ULONG,
    mut pBuffer: *mut u8,
    ppunk: *mut *mut IUnknown,
    riid: REFIID,
) -> *mut u8 {
    let hr: HRESULT;
    let h: HGLOBAL;
    let mut stm: *mut IStream = core::ptr::null_mut();
    let size: DWORD;
    let ptr: LPVOID;
    let orig: *mut IUnknown;

    TRACE!(
        b"(%s, %p, %p, %s)\n\0".as_ptr() as *const i8,
        debugstr_user_flags(pFlags),
        pBuffer,
        ppunk,
        debugstr_guid(riid as *const c_void),
    );

    ALIGN_POINTER!(pBuffer, 3);

    size = *(pBuffer as *const DWORD);
    pBuffer = pBuffer.add(core::mem::size_of::<DWORD>());
    if size != *(pBuffer as *const DWORD) {
        RaiseException(RPC_X_BAD_STUB_DATA, 0, 0, core::ptr::null());
    }

    pBuffer = pBuffer.add(core::mem::size_of::<DWORD>());

    /* FIXME: sanity check on size */

    h = GlobalAlloc(GMEM_MOVEABLE, size as SIZE_T);
    if h.is_null() {
        RaiseException(RPC_X_NO_MEMORY, 0, 0, core::ptr::null());
    }

    if CreateStreamOnHGlobal(h, 1, &mut stm) != S_OK {
        GlobalFree(h);
        RaiseException(RPC_X_NO_MEMORY, 0, 0, core::ptr::null());
    }

    ptr = GlobalLock(h);
    memcpy(ptr, pBuffer as *const c_void, size as usize);
    GlobalUnlock(h);

    orig = *ppunk;
    hr = CoUnmarshalInterface((stm).cast(), (riid).cast(), ppunk as *mut LPVOID);
    IStream_Release(stm);

    if hr != S_OK {
        RaiseException(hr as DWORD, 0, 0, core::ptr::null());
    }

    if !orig.is_null() {
        IUnknown_Release(orig);
    }

    pBuffer.add(size as usize)
}

/******************************************************************************
 *           WdtpInterfacePointer_UserFree [OLE32.@]
 */
#[no_mangle]
pub unsafe extern "system" fn WdtpInterfacePointer_UserFree(punk: *mut IUnknown) {
    TRACE!(b"(%p)\n\0".as_ptr() as *const i8, punk);
    if !punk.is_null() {
        IUnknown_Release(punk);
    }
}

/******************************************************************************
 *           STGMEDIUM_UserSize [OLE32.@]
 */
#[no_mangle]
pub unsafe extern "system" fn STGMEDIUM_UserSize(
    pFlags: *mut ULONG,
    StartingSize: ULONG,
    pStgMedium: *mut STGMEDIUM,
) -> ULONG {
    let mut size: ULONG = StartingSize;

    TRACE!(
        b"(%s, %d, %p\n\0".as_ptr() as *const i8,
        debugstr_user_flags(pFlags),
        StartingSize,
        pStgMedium,
    );

    ALIGN_LENGTH!(size, 3);

    size += 2 * core::mem::size_of::<DWORD>() as ULONG;
    if (*pStgMedium).tymed != TYMED_NULL {
        size += core::mem::size_of::<DWORD>() as ULONG;
    }

    match (*pStgMedium).tymed {
        TYMED_NULL => {
            TRACE!(b"TYMED_NULL\n\0".as_ptr() as *const i8);
        }
        TYMED_HGLOBAL => {
            TRACE!(b"TYMED_HGLOBAL\n\0".as_ptr() as *const i8);
            if !(*pStgMedium).__bindgen_anon_1.hGlobal.is_null() {
                size = HGLOBAL_UserSize(
                    pFlags,
                    size,
                    core::ptr::addr_of_mut!((*pStgMedium).__bindgen_anon_1.hGlobal),
                );
            }
        }
        TYMED_FILE => {
            TRACE!(b"TYMED_FILE\n\0".as_ptr() as *const i8);
            if !(*pStgMedium).__bindgen_anon_1.lpszFileName.is_null() {
                TRACE!(
                    b"file name is %s\n\0".as_ptr() as *const i8,
                    debugstr_w((*pStgMedium).__bindgen_anon_1.lpszFileName as *const c_void),
                );
                size += 3 * core::mem::size_of::<DWORD>() as ULONG
                    + (lstrlenW((*pStgMedium).__bindgen_anon_1.lpszFileName) + 1) as ULONG
                        * core::mem::size_of::<WCHAR>() as ULONG;
            }
        }
        TYMED_ISTREAM => {
            TRACE!(b"TYMED_ISTREAM\n\0".as_ptr() as *const i8);
            if !(*pStgMedium).__bindgen_anon_1.pstm.is_null() {
                let mut unk: *mut IUnknown = core::ptr::null_mut();
                IStream_QueryInterface(
                    (*pStgMedium).__bindgen_anon_1.pstm,
                    &IID_IUnknown,
                    core::ptr::addr_of_mut!(unk) as *mut *mut c_void,
                );
                size = WdtpInterfacePointer_UserSize(
                    pFlags,
                    LOWORD(*pFlags) as ULONG,
                    size,
                    unk,
                    &IID_IStream,
                );
                IUnknown_Release(unk);
            }
        }
        TYMED_ISTORAGE => {
            TRACE!(b"TYMED_ISTORAGE\n\0".as_ptr() as *const i8);
            if !(*pStgMedium).__bindgen_anon_1.pstg.is_null() {
                let mut unk: *mut IUnknown = core::ptr::null_mut();
                IStorage_QueryInterface(
                    (*pStgMedium).__bindgen_anon_1.pstg,
                    &IID_IUnknown,
                    core::ptr::addr_of_mut!(unk) as *mut *mut c_void,
                );
                size = WdtpInterfacePointer_UserSize(
                    pFlags,
                    LOWORD(*pFlags) as ULONG,
                    size,
                    unk,
                    &IID_IStorage,
                );
                IUnknown_Release(unk);
            }
        }
        TYMED_GDI => {
            TRACE!(b"TYMED_GDI\n\0".as_ptr() as *const i8);
            if !(*pStgMedium).__bindgen_anon_1.hBitmap.is_null() {
                size = HBITMAP_UserSize(
                    pFlags,
                    size,
                    core::ptr::addr_of_mut!((*pStgMedium).__bindgen_anon_1.hBitmap),
                );
            }
        }
        TYMED_MFPICT => {
            TRACE!(b"TYMED_MFPICT\n\0".as_ptr() as *const i8);
            if !(*pStgMedium).__bindgen_anon_1.hMetaFilePict.is_null() {
                size = HMETAFILEPICT_UserSize(
                    pFlags,
                    size,
                    core::ptr::addr_of_mut!((*pStgMedium).__bindgen_anon_1.hMetaFilePict),
                );
            }
        }
        TYMED_ENHMF => {
            TRACE!(b"TYMED_ENHMF\n\0".as_ptr() as *const i8);
            if !(*pStgMedium).__bindgen_anon_1.hEnhMetaFile.is_null() {
                size = HENHMETAFILE_UserSize(
                    pFlags,
                    size,
                    core::ptr::addr_of_mut!((*pStgMedium).__bindgen_anon_1.hEnhMetaFile)
                        as *mut crate::user32::HENHMETAFILE,
                );
            }
        }
        _ => {
            RaiseException(DV_E_TYMED as DWORD, 0, 0, core::ptr::null());
        }
    }

    if !(*pStgMedium).pUnkForRelease.is_null() {
        size = WdtpInterfacePointer_UserSize(
            pFlags,
            LOWORD(*pFlags) as ULONG,
            size,
            (*pStgMedium).pUnkForRelease,
            &IID_IUnknown,
        );
    }

    size
}

/******************************************************************************
 *           STGMEDIUM_UserMarshal [OLE32.@]
 */
#[no_mangle]
pub unsafe extern "system" fn STGMEDIUM_UserMarshal(
    pFlags: *mut ULONG,
    mut pBuffer: *mut u8,
    pStgMedium: *mut STGMEDIUM,
) -> *mut u8 {
    TRACE!(
        b"(%s, %p, %p\n\0".as_ptr() as *const i8,
        debugstr_user_flags(pFlags),
        pBuffer,
        pStgMedium,
    );

    ALIGN_POINTER!(pBuffer, 3);

    *(pBuffer as *mut DWORD) = (*pStgMedium).tymed;
    pBuffer = pBuffer.add(core::mem::size_of::<DWORD>());
    if (*pStgMedium).tymed != TYMED_NULL {
        *(pBuffer as *mut DWORD) = (*pStgMedium).__bindgen_anon_1.pstg as DWORD_PTR as DWORD;
        pBuffer = pBuffer.add(core::mem::size_of::<DWORD>());
    }
    *(pBuffer as *mut DWORD) = (*pStgMedium).pUnkForRelease as DWORD_PTR as DWORD;
    pBuffer = pBuffer.add(core::mem::size_of::<DWORD>());

    match (*pStgMedium).tymed {
        TYMED_NULL => {
            TRACE!(b"TYMED_NULL\n\0".as_ptr() as *const i8);
        }
        TYMED_HGLOBAL => {
            TRACE!(b"TYMED_HGLOBAL\n\0".as_ptr() as *const i8);
            if !(*pStgMedium).__bindgen_anon_1.hGlobal.is_null() {
                pBuffer = HGLOBAL_UserMarshal(
                    pFlags,
                    pBuffer,
                    core::ptr::addr_of_mut!((*pStgMedium).__bindgen_anon_1.hGlobal),
                );
            }
        }
        TYMED_FILE => {
            TRACE!(b"TYMED_FILE\n\0".as_ptr() as *const i8);
            if !(*pStgMedium).__bindgen_anon_1.lpszFileName.is_null() {
                let len: DWORD = lstrlenW((*pStgMedium).__bindgen_anon_1.lpszFileName) as DWORD;
                /* conformance */
                *(pBuffer as *mut DWORD) = len + 1;
                pBuffer = pBuffer.add(core::mem::size_of::<DWORD>());
                /* offset */
                *(pBuffer as *mut DWORD) = 0;
                pBuffer = pBuffer.add(core::mem::size_of::<DWORD>());
                /* variance */
                *(pBuffer as *mut DWORD) = len + 1;
                pBuffer = pBuffer.add(core::mem::size_of::<DWORD>());

                TRACE!(
                    b"file name is %s\n\0".as_ptr() as *const i8,
                    debugstr_w((*pStgMedium).__bindgen_anon_1.lpszFileName as *const c_void),
                );
                memcpy(
                    pBuffer as *mut c_void,
                    (*pStgMedium).__bindgen_anon_1.lpszFileName as *const c_void,
                    ((len + 1) as usize) * core::mem::size_of::<WCHAR>(),
                );
            }
        }
        TYMED_ISTREAM => {
            TRACE!(b"TYMED_ISTREAM\n\0".as_ptr() as *const i8);
            if !(*pStgMedium).__bindgen_anon_1.pstm.is_null() {
                let mut unk: *mut IUnknown = core::ptr::null_mut();
                IStream_QueryInterface(
                    (*pStgMedium).__bindgen_anon_1.pstm,
                    &IID_IUnknown,
                    core::ptr::addr_of_mut!(unk) as *mut *mut c_void,
                );
                pBuffer = WdtpInterfacePointer_UserMarshal(
                    pFlags,
                    LOWORD(*pFlags) as ULONG,
                    pBuffer,
                    unk,
                    &IID_IStream,
                );
                IUnknown_Release(unk);
            }
        }
        TYMED_ISTORAGE => {
            TRACE!(b"TYMED_ISTORAGE\n\0".as_ptr() as *const i8);
            if !(*pStgMedium).__bindgen_anon_1.pstg.is_null() {
                let mut unk: *mut IUnknown = core::ptr::null_mut();
                IStorage_QueryInterface(
                    (*pStgMedium).__bindgen_anon_1.pstg,
                    &IID_IUnknown,
                    core::ptr::addr_of_mut!(unk) as *mut *mut c_void,
                );
                pBuffer = WdtpInterfacePointer_UserMarshal(
                    pFlags,
                    LOWORD(*pFlags) as ULONG,
                    pBuffer,
                    unk,
                    &IID_IStorage,
                );
                IUnknown_Release(unk);
            }
        }
        TYMED_GDI => {
            TRACE!(b"TYMED_GDI\n\0".as_ptr() as *const i8);
            if !(*pStgMedium).__bindgen_anon_1.hBitmap.is_null() {
                pBuffer = HBITMAP_UserMarshal(
                    pFlags,
                    pBuffer,
                    core::ptr::addr_of_mut!((*pStgMedium).__bindgen_anon_1.hBitmap),
                );
            }
        }
        TYMED_MFPICT => {
            TRACE!(b"TYMED_MFPICT\n\0".as_ptr() as *const i8);
            if !(*pStgMedium).__bindgen_anon_1.hMetaFilePict.is_null() {
                pBuffer = HMETAFILEPICT_UserMarshal(
                    pFlags,
                    pBuffer,
                    core::ptr::addr_of_mut!((*pStgMedium).__bindgen_anon_1.hMetaFilePict),
                );
            }
        }
        TYMED_ENHMF => {
            TRACE!(b"TYMED_ENHMF\n\0".as_ptr() as *const i8);
            if !(*pStgMedium).__bindgen_anon_1.hEnhMetaFile.is_null() {
                pBuffer = HENHMETAFILE_UserMarshal(
                    pFlags,
                    pBuffer,
                    core::ptr::addr_of_mut!((*pStgMedium).__bindgen_anon_1.hEnhMetaFile)
                        as *mut crate::user32::HENHMETAFILE,
                );
            }
        }
        _ => {
            RaiseException(DV_E_TYMED as DWORD, 0, 0, core::ptr::null());
        }
    }

    if !(*pStgMedium).pUnkForRelease.is_null() {
        pBuffer = WdtpInterfacePointer_UserMarshal(
            pFlags,
            LOWORD(*pFlags) as ULONG,
            pBuffer,
            (*pStgMedium).pUnkForRelease,
            &IID_IUnknown,
        );
    }

    pBuffer
}

/******************************************************************************
 *           STGMEDIUM_UserUnmarshal [OLE32.@]
 */
#[no_mangle]
pub unsafe extern "system" fn STGMEDIUM_UserUnmarshal(
    pFlags: *mut ULONG,
    mut pBuffer: *mut u8,
    pStgMedium: *mut STGMEDIUM,
) -> *mut u8 {
    let mut content: DWORD = 0;
    let releaseunk: DWORD;

    ALIGN_POINTER!(pBuffer, 3);

    TRACE!(
        b"(%s, %p, %p\n\0".as_ptr() as *const i8,
        debugstr_user_flags(pFlags),
        pBuffer,
        pStgMedium,
    );

    (*pStgMedium).tymed = *(pBuffer as *const DWORD);
    pBuffer = pBuffer.add(core::mem::size_of::<DWORD>());
    if (*pStgMedium).tymed != TYMED_NULL {
        content = *(pBuffer as *const DWORD);
        pBuffer = pBuffer.add(core::mem::size_of::<DWORD>());
    }
    releaseunk = *(pBuffer as *const DWORD);
    pBuffer = pBuffer.add(core::mem::size_of::<DWORD>());

    match (*pStgMedium).tymed {
        TYMED_NULL => {
            TRACE!(b"TYMED_NULL\n\0".as_ptr() as *const i8);
        }
        TYMED_HGLOBAL => {
            TRACE!(b"TYMED_HGLOBAL\n\0".as_ptr() as *const i8);
            if content != 0 {
                pBuffer = HGLOBAL_UserUnmarshal(
                    pFlags,
                    pBuffer,
                    core::ptr::addr_of_mut!((*pStgMedium).__bindgen_anon_1.hGlobal),
                );
            }
        }
        TYMED_FILE => {
            TRACE!(b"TYMED_FILE\n\0".as_ptr() as *const i8);
            if content != 0 {
                let conformance: DWORD;
                let variance: DWORD;
                conformance = *(pBuffer as *const DWORD);
                pBuffer = pBuffer.add(core::mem::size_of::<DWORD>());
                if *(pBuffer as *const DWORD) != 0 {
                    ERR!(
                        b"invalid offset %d\n\0".as_ptr() as *const i8,
                        *(pBuffer as *const DWORD),
                    );
                    RpcRaiseException(RPC_S_INVALID_BOUND as RPC_STATUS);
                }
                pBuffer = pBuffer.add(core::mem::size_of::<DWORD>());
                variance = *(pBuffer as *const DWORD);
                pBuffer = pBuffer.add(core::mem::size_of::<DWORD>());
                if conformance != variance {
                    ERR!(
                        b"conformance (%d) and variance (%d) should be equal\n\0".as_ptr()
                            as *const i8,
                        conformance,
                        variance,
                    );
                    RpcRaiseException(RPC_S_INVALID_BOUND as RPC_STATUS);
                }
                if conformance > 0x7fffffff {
                    ERR!(
                        b"conformance 0x%x too large\n\0".as_ptr() as *const i8,
                        conformance,
                    );
                    RpcRaiseException(RPC_S_INVALID_BOUND as RPC_STATUS);
                }
                (*pStgMedium).__bindgen_anon_1.lpszFileName = CoTaskMemAlloc(
                    (conformance as SIZE_T) * core::mem::size_of::<WCHAR>() as SIZE_T,
                ) as LPOLESTR;
                if (*pStgMedium).__bindgen_anon_1.lpszFileName.is_null() {
                    RpcRaiseException(ERROR_OUTOFMEMORY as RPC_STATUS);
                }
                TRACE!(
                    b"unmarshalled file name is %s\n\0".as_ptr() as *const i8,
                    debugstr_wn(pBuffer as *const c_void),
                );
                memcpy(
                    (*pStgMedium).__bindgen_anon_1.lpszFileName as *mut c_void,
                    pBuffer as *const c_void,
                    (variance as usize) * core::mem::size_of::<WCHAR>(),
                );
                pBuffer = pBuffer.add((variance as usize) * core::mem::size_of::<WCHAR>());
            } else {
                (*pStgMedium).__bindgen_anon_1.lpszFileName = core::ptr::null_mut();
            }
        }
        TYMED_ISTREAM => {
            TRACE!(b"TYMED_ISTREAM\n\0".as_ptr() as *const i8);
            if content != 0 {
                pBuffer = WdtpInterfacePointer_UserUnmarshal(
                    pFlags,
                    pBuffer,
                    core::ptr::addr_of_mut!((*pStgMedium).__bindgen_anon_1.pstm)
                        as *mut *mut IUnknown,
                    &IID_IStream,
                );
            } else {
                if !(*pStgMedium).__bindgen_anon_1.pstm.is_null() {
                    IStream_Release((*pStgMedium).__bindgen_anon_1.pstm);
                }
                (*pStgMedium).__bindgen_anon_1.pstm = core::ptr::null_mut();
            }
        }
        TYMED_ISTORAGE => {
            TRACE!(b"TYMED_ISTORAGE\n\0".as_ptr() as *const i8);
            if content != 0 {
                pBuffer = WdtpInterfacePointer_UserUnmarshal(
                    pFlags,
                    pBuffer,
                    core::ptr::addr_of_mut!((*pStgMedium).__bindgen_anon_1.pstg)
                        as *mut *mut IUnknown,
                    &IID_IStorage,
                );
            } else {
                if !(*pStgMedium).__bindgen_anon_1.pstg.is_null() {
                    IStorage_Release((*pStgMedium).__bindgen_anon_1.pstg);
                }
                (*pStgMedium).__bindgen_anon_1.pstg = core::ptr::null_mut();
            }
        }
        TYMED_GDI => {
            TRACE!(b"TYMED_GDI\n\0".as_ptr() as *const i8);
            if content != 0 {
                pBuffer = HBITMAP_UserUnmarshal(
                    pFlags,
                    pBuffer,
                    core::ptr::addr_of_mut!((*pStgMedium).__bindgen_anon_1.hBitmap),
                );
            } else {
                (*pStgMedium).__bindgen_anon_1.hBitmap = core::ptr::null_mut();
            }
        }
        TYMED_MFPICT => {
            TRACE!(b"TYMED_MFPICT\n\0".as_ptr() as *const i8);
            if content != 0 {
                pBuffer = HMETAFILEPICT_UserUnmarshal(
                    pFlags,
                    pBuffer,
                    core::ptr::addr_of_mut!((*pStgMedium).__bindgen_anon_1.hMetaFilePict),
                );
            } else {
                (*pStgMedium).__bindgen_anon_1.hMetaFilePict = core::ptr::null_mut();
            }
        }
        TYMED_ENHMF => {
            TRACE!(b"TYMED_ENHMF\n\0".as_ptr() as *const i8);
            if content != 0 {
                pBuffer = HENHMETAFILE_UserUnmarshal(
                    pFlags,
                    pBuffer,
                    core::ptr::addr_of_mut!((*pStgMedium).__bindgen_anon_1.hEnhMetaFile)
                        as *mut crate::user32::HENHMETAFILE,
                );
            } else {
                (*pStgMedium).__bindgen_anon_1.hEnhMetaFile = core::ptr::null_mut();
            }
        }
        _ => {
            RaiseException(DV_E_TYMED as DWORD, 0, 0, core::ptr::null());
        }
    }

    if releaseunk != 0 {
        pBuffer = WdtpInterfacePointer_UserUnmarshal(
            pFlags,
            pBuffer,
            core::ptr::addr_of_mut!((*pStgMedium).pUnkForRelease),
            &IID_IUnknown,
        );
    }
    /* Unlike the IStream / IStorage ifaces, the existing pUnkForRelease
       is left intact if a NULL ptr is unmarshalled - see the tests. */

    pBuffer
}

/******************************************************************************
 *           STGMEDIUM_UserFree [OLE32.@]
 */
#[no_mangle]
pub unsafe extern "system" fn STGMEDIUM_UserFree(flags: *mut ULONG, med: *mut STGMEDIUM) {
    TRACE!(
        b"(%s, %p)\n\0".as_ptr() as *const i8,
        debugstr_user_flags(flags),
        med,
    );

    // NOTE fully qualified on purpose. ReleaseStgMedium is DEFINED in ring::ole2 and
    // also declared in ole32_exports, so the bare name can be glob-ambiguous through the
    // prelude. Both spell it extern "system" (it is WINAPI; a C-ABI spelling would be an
    // i386 stdcall break), so the qualification is about name resolution, not ABI.
    match (*med).tymed {
        TYMED_NULL | TYMED_FILE | TYMED_ISTREAM | TYMED_ISTORAGE => {
            ReleaseStgMedium(med);
        }
        TYMED_HGLOBAL | TYMED_GDI | TYMED_MFPICT | TYMED_ENHMF => {
            if LOWORD(*flags) as DWORD == MSHCTX_INPROC {
                (*med).tymed = TYMED_NULL;
            }
            ReleaseStgMedium(med);
        }
        _ => {
            RaiseException(DV_E_TYMED as DWORD, 0, 0, core::ptr::null());
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn ASYNC_STGMEDIUM_UserSize(
    pFlags: *mut ULONG,
    StartingSize: ULONG,
    pStgMedium: *mut ASYNC_STGMEDIUM,
) -> ULONG {
    TRACE!(b"\n\0".as_ptr() as *const i8);
    STGMEDIUM_UserSize(pFlags, StartingSize, pStgMedium)
}

#[no_mangle]
pub unsafe extern "system" fn ASYNC_STGMEDIUM_UserMarshal(
    pFlags: *mut ULONG,
    pBuffer: *mut u8,
    pStgMedium: *mut ASYNC_STGMEDIUM,
) -> *mut u8 {
    TRACE!(b"\n\0".as_ptr() as *const i8);
    STGMEDIUM_UserMarshal(pFlags, pBuffer, pStgMedium)
}

#[no_mangle]
pub unsafe extern "system" fn ASYNC_STGMEDIUM_UserUnmarshal(
    pFlags: *mut ULONG,
    pBuffer: *mut u8,
    pStgMedium: *mut ASYNC_STGMEDIUM,
) -> *mut u8 {
    TRACE!(b"\n\0".as_ptr() as *const i8);
    STGMEDIUM_UserUnmarshal(pFlags, pBuffer, pStgMedium)
}

#[no_mangle]
pub unsafe extern "system" fn ASYNC_STGMEDIUM_UserFree(
    pFlags: *mut ULONG,
    pStgMedium: *mut ASYNC_STGMEDIUM,
) {
    TRACE!(b"\n\0".as_ptr() as *const i8);
    STGMEDIUM_UserFree(pFlags, pStgMedium);
}

#[no_mangle]
pub unsafe extern "system" fn FLAG_STGMEDIUM_UserSize(
    _pFlags: *mut ULONG,
    StartingSize: ULONG,
    _pStgMedium: *mut FLAG_STGMEDIUM,
) -> ULONG {
    FIXME!(b":stub\n\0".as_ptr() as *const i8);
    StartingSize
}

#[no_mangle]
pub unsafe extern "system" fn FLAG_STGMEDIUM_UserMarshal(
    _pFlags: *mut ULONG,
    pBuffer: *mut u8,
    _pStgMedium: *mut FLAG_STGMEDIUM,
) -> *mut u8 {
    FIXME!(b":stub\n\0".as_ptr() as *const i8);
    pBuffer
}

#[no_mangle]
pub unsafe extern "system" fn FLAG_STGMEDIUM_UserUnmarshal(
    _pFlags: *mut ULONG,
    pBuffer: *mut u8,
    _pStgMedium: *mut FLAG_STGMEDIUM,
) -> *mut u8 {
    FIXME!(b":stub\n\0".as_ptr() as *const i8);
    pBuffer
}

#[no_mangle]
pub unsafe extern "system" fn FLAG_STGMEDIUM_UserFree(
    _pFlags: *mut ULONG,
    _pStgMedium: *mut FLAG_STGMEDIUM,
) {
    FIXME!(b":stub\n\0".as_ptr() as *const i8);
}

#[no_mangle]
pub unsafe extern "system" fn SNB_UserSize(
    _pFlags: *mut ULONG,
    StartingSize: ULONG,
    pSnb: *mut SNB,
) -> ULONG {
    let mut size: ULONG = StartingSize;

    TRACE!(
        b"(%s, %d, %p\n\0".as_ptr() as *const i8,
        debugstr_user_flags(pFlags),
        StartingSize,
        pSnb,
    );

    ALIGN_LENGTH!(size, 3);

    /* two counters from RemSNB header, plus one more ULONG */
    size += 3 * core::mem::size_of::<ULONG>() as ULONG;

    /* now actual data length */
    if !(*pSnb).is_null() {
        let mut ptrW: *mut *mut WCHAR = *pSnb;

        while !(*ptrW).is_null() {
            size += (lstrlenW(*ptrW) + 1) as ULONG * core::mem::size_of::<WCHAR>() as ULONG;
            ptrW = ptrW.add(1);
        }
    }

    size
}

#[no_mangle]
pub unsafe extern "system" fn SNB_UserMarshal(
    _pFlags: *mut ULONG,
    mut pBuffer: *mut u8,
    pSnb: *mut SNB,
) -> *mut u8 {
    let wire: *mut SNB_wire;
    let mut size: ULONG;

    TRACE!(
        b"(%s, %p, %p)\n\0".as_ptr() as *const i8,
        debugstr_user_flags(pFlags),
        pBuffer,
        pSnb,
    );

    ALIGN_POINTER!(pBuffer, 3);

    wire = pBuffer as *mut SNB_wire;
    (*wire).charcnt = 0;
    (*wire).strcnt = 0;
    size = 3 * core::mem::size_of::<ULONG>() as ULONG;

    if !(*pSnb).is_null() {
        let mut ptrW: *mut *mut WCHAR = *pSnb;
        let mut dataW: *mut WCHAR = (*wire).data.as_mut_ptr();

        while !(*ptrW).is_null() {
            let len: ULONG = (lstrlenW(*ptrW) + 1) as ULONG;

            (*wire).strcnt += 1;
            (*wire).charcnt += len;
            memcpy(
                dataW as *mut c_void,
                *ptrW as *const c_void,
                (len as usize) * core::mem::size_of::<WCHAR>(),
            );
            dataW = dataW.add(len as usize);

            size += len * core::mem::size_of::<WCHAR>() as ULONG;
            ptrW = ptrW.add(1);
        }
    }

    (*wire).datalen = (*wire).charcnt;
    pBuffer.add(size as usize)
}

#[no_mangle]
pub unsafe extern "system" fn SNB_UserUnmarshal(
    pFlags: *mut ULONG,
    pBuffer: *mut u8,
    pSnb: *mut SNB,
) -> *mut u8 {
    let umcb = pFlags as *mut USER_MARSHAL_CB;
    let wire = pBuffer as *mut SNB_wire;

    TRACE!(
        b"(%s, %p, %p)\n\0".as_ptr() as *const i8,
        debugstr_user_flags(pFlags),
        pBuffer,
        pSnb,
    );

    if !(*pSnb).is_null() {
        ((*(*umcb).pStubMsg).pfnFree.unwrap_unchecked())(*pSnb as *mut c_void);
    }

    if (*wire).datalen == 0 {
        *pSnb = core::ptr::null_mut();
    } else {
        let mut src: *mut WCHAR = (*wire).data.as_mut_ptr();
        let mut dest: *mut WCHAR;
        let mut ptrW: *mut *mut WCHAR;

        ptrW = ((*(*umcb).pStubMsg).pfnAllocate.unwrap_unchecked())(
            (((*wire).strcnt + 1) as SIZE_T) * core::mem::size_of::<*mut WCHAR>() as SIZE_T
                + ((*wire).datalen as SIZE_T) * core::mem::size_of::<WCHAR>() as SIZE_T,
        ) as *mut *mut WCHAR;
        *pSnb = ptrW as SNB;
        dest = ptrW.add(((*wire).strcnt + 1) as usize) as *mut WCHAR;

        for _i in 0..(*wire).strcnt {
            let len: ULONG = lstrlenW(src) as ULONG;
            memcpy(
                dest as *mut c_void,
                src as *const c_void,
                ((len + 1) as usize) * core::mem::size_of::<WCHAR>(),
            );
            *ptrW = dest;
            src = src.add((len + 1) as usize);
            dest = dest.add((len + 1) as usize);
            ptrW = ptrW.add(1);
        }
        *ptrW = core::ptr::null_mut();
    }

    pBuffer.add(
        3 * core::mem::size_of::<ULONG>()
            + ((*wire).datalen as usize) * core::mem::size_of::<WCHAR>(),
    )
}

#[no_mangle]
pub unsafe extern "system" fn SNB_UserFree(pFlags: *mut ULONG, pSnb: *mut SNB) {
    let umcb = pFlags as *mut USER_MARSHAL_CB;
    TRACE!(b"(%p)\n\0".as_ptr() as *const i8, pSnb);
    if !(*pSnb).is_null() {
        ((*(*umcb).pStubMsg).pfnFree.unwrap_unchecked())(*pSnb as *mut c_void);
    }
}

/* call_as/local stubs for unknwn.idl */

#[no_mangle]
pub unsafe extern "system" fn IClassFactory_CreateInstance_Proxy(
    This: *mut IClassFactory,
    pUnkOuter: *mut IUnknown,
    riid: REFIID,
    ppvObject: *mut *mut c_void,
) -> HRESULT {
    TRACE!(
        b"(%p, %s, %p)\n\0".as_ptr() as *const i8,
        pUnkOuter,
        debugstr_guid(riid as *const c_void),
        ppvObject,
    );
    *ppvObject = core::ptr::null_mut();
    if !pUnkOuter.is_null() {
        ERR!(b"aggregation is not allowed on remote objects\n\0".as_ptr() as *const i8);
        return CLASS_E_NOAGGREGATION;
    }
    IClassFactory_RemoteCreateInstance_Proxy(This, riid, ppvObject as *mut *mut IUnknown)
}

#[no_mangle]
pub unsafe extern "system" fn IClassFactory_CreateInstance_Stub(
    This: *mut IClassFactory,
    riid: REFIID,
    ppvObject: *mut *mut IUnknown,
) -> HRESULT {
    TRACE!(
        b"(%s, %p)\n\0".as_ptr() as *const i8,
        debugstr_guid(riid as *const c_void),
        ppvObject,
    );
    IClassFactory_CreateInstance(
        This,
        core::ptr::null_mut(),
        riid,
        ppvObject as *mut *mut c_void,
    )
}

#[no_mangle]
pub unsafe extern "system" fn IClassFactory_LockServer_Proxy(
    _This: *mut IClassFactory,
    _fLock: BOOL,
) -> HRESULT {
    FIXME!(b":stub\n\0".as_ptr() as *const i8);
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn IClassFactory_LockServer_Stub(
    _This: *mut IClassFactory,
    _fLock: BOOL,
) -> HRESULT {
    FIXME!(b":stub\n\0".as_ptr() as *const i8);
    E_NOTIMPL
}

/* call_as/local stubs for objidl.idl */

#[no_mangle]
pub unsafe extern "system" fn IEnumUnknown_Next_Proxy(
    This: *mut IEnumUnknown,
    celt: ULONG,
    rgelt: *mut *mut IUnknown,
    pceltFetched: *mut ULONG,
) -> HRESULT {
    let mut fetched: ULONG = 0;
    TRACE!(
        b"(%p)->(%d, %p, %p)\n\0".as_ptr() as *const i8,
        This,
        celt,
        rgelt,
        pceltFetched,
    );
    let pceltFetched = if pceltFetched.is_null() {
        core::ptr::addr_of_mut!(fetched)
    } else {
        pceltFetched
    };
    IEnumUnknown_RemoteNext_Proxy(This, celt, rgelt, pceltFetched)
}

#[no_mangle]
pub unsafe extern "system" fn IEnumUnknown_Next_Stub(
    This: *mut IEnumUnknown,
    celt: ULONG,
    rgelt: *mut *mut IUnknown,
    pceltFetched: *mut ULONG,
) -> HRESULT {
    let hr: HRESULT;
    TRACE!(
        b"(%p)->(%d, %p, %p)\n\0".as_ptr() as *const i8,
        This,
        celt,
        rgelt,
        pceltFetched,
    );
    *pceltFetched = 0;
    hr = IEnumUnknown_Next(This, celt, rgelt, pceltFetched);
    if hr == S_OK {
        *pceltFetched = celt;
    }
    hr
}

#[no_mangle]
pub unsafe extern "system" fn IBindCtx_SetBindOptions_Proxy(
    _This: *mut IBindCtx,
    _pbindopts: *mut BIND_OPTS,
) -> HRESULT {
    FIXME!(b":stub\n\0".as_ptr() as *const i8);
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn IBindCtx_SetBindOptions_Stub(
    _This: *mut IBindCtx,
    _pbindopts: *mut BIND_OPTS2,
) -> HRESULT {
    FIXME!(b":stub\n\0".as_ptr() as *const i8);
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn IBindCtx_GetBindOptions_Proxy(
    _This: *mut IBindCtx,
    _pbindopts: *mut BIND_OPTS,
) -> HRESULT {
    FIXME!(b":stub\n\0".as_ptr() as *const i8);
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn IBindCtx_GetBindOptions_Stub(
    _This: *mut IBindCtx,
    _pbindopts: *mut BIND_OPTS2,
) -> HRESULT {
    FIXME!(b":stub\n\0".as_ptr() as *const i8);
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn IEnumMoniker_Next_Proxy(
    This: *mut IEnumMoniker,
    celt: ULONG,
    rgelt: *mut *mut IMoniker,
    pceltFetched: *mut ULONG,
) -> HRESULT {
    let mut fetched: ULONG = 0;
    TRACE!(
        b"(%p)->(%d, %p, %p)\n\0".as_ptr() as *const i8,
        This,
        celt,
        rgelt,
        pceltFetched,
    );
    let pceltFetched = if pceltFetched.is_null() {
        core::ptr::addr_of_mut!(fetched)
    } else {
        pceltFetched
    };
    IEnumMoniker_RemoteNext_Proxy(This, celt, rgelt, pceltFetched)
}

#[no_mangle]
pub unsafe extern "system" fn IEnumMoniker_Next_Stub(
    This: *mut IEnumMoniker,
    celt: ULONG,
    rgelt: *mut *mut IMoniker,
    pceltFetched: *mut ULONG,
) -> HRESULT {
    let hr: HRESULT;
    TRACE!(
        b"(%p)->(%d, %p, %p)\n\0".as_ptr() as *const i8,
        This,
        celt,
        rgelt,
        pceltFetched,
    );
    *pceltFetched = 0;
    hr = IEnumMoniker_Next(This, celt, rgelt, pceltFetched);
    if hr == S_OK {
        *pceltFetched = celt;
    }
    hr
}

#[no_mangle]
pub unsafe extern "system" fn IRunnableObject_IsRunning_Proxy(_This: *mut IRunnableObject) -> BOOL {
    let mut rv: BOOL = 0;
    FIXME!(b":stub\n\0".as_ptr() as *const i8);
    core::ptr::write_bytes(core::ptr::addr_of_mut!(rv), 0, 1);
    rv
}

#[no_mangle]
pub unsafe extern "system" fn IRunnableObject_IsRunning_Stub(
    _This: *mut IRunnableObject,
) -> HRESULT {
    FIXME!(b":stub\n\0".as_ptr() as *const i8);
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn IMoniker_BindToObject_Proxy(
    _This: *mut IMoniker,
    _pbc: *mut IBindCtx,
    _pmkToLeft: *mut IMoniker,
    _riidResult: REFIID,
    _ppvResult: *mut *mut c_void,
) -> HRESULT {
    FIXME!(b":stub\n\0".as_ptr() as *const i8);
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn IMoniker_BindToObject_Stub(
    _This: *mut IMoniker,
    _pbc: *mut IBindCtx,
    _pmkToLeft: *mut IMoniker,
    _riidResult: REFIID,
    _ppvResult: *mut *mut IUnknown,
) -> HRESULT {
    FIXME!(b":stub\n\0".as_ptr() as *const i8);
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn IMoniker_BindToStorage_Proxy(
    This: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    riid: REFIID,
    ppvObj: *mut *mut c_void,
) -> HRESULT {
    TRACE!(
        b"(%p)->(%p %p %s %p)\n\0".as_ptr() as *const i8,
        This,
        pbc,
        pmkToLeft,
        debugstr_guid(riid as *const c_void),
        ppvObj,
    );
    IMoniker_RemoteBindToStorage_Proxy(This, pbc, pmkToLeft, riid, ppvObj as *mut *mut IUnknown)
}

#[no_mangle]
pub unsafe extern "system" fn IMoniker_BindToStorage_Stub(
    This: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    riid: REFIID,
    ppvObj: *mut *mut IUnknown,
) -> HRESULT {
    TRACE!(
        b"(%p)->(%p %p %s %p)\n\0".as_ptr() as *const i8,
        This,
        pbc,
        pmkToLeft,
        debugstr_guid(riid as *const c_void),
        ppvObj,
    );
    IMoniker_BindToStorage(This, pbc, pmkToLeft, riid, ppvObj as *mut *mut c_void)
}

#[no_mangle]
pub unsafe extern "system" fn IEnumString_Next_Proxy(
    This: *mut IEnumString,
    celt: ULONG,
    rgelt: *mut LPOLESTR,
    pceltFetched: *mut ULONG,
) -> HRESULT {
    let mut fetched: ULONG = 0;
    TRACE!(
        b"(%p)->(%d, %p, %p)\n\0".as_ptr() as *const i8,
        This,
        celt,
        rgelt,
        pceltFetched,
    );
    let pceltFetched = if pceltFetched.is_null() {
        core::ptr::addr_of_mut!(fetched)
    } else {
        pceltFetched
    };
    IEnumString_RemoteNext_Proxy(This, celt, rgelt, pceltFetched)
}

#[no_mangle]
pub unsafe extern "system" fn IEnumString_Next_Stub(
    This: *mut IEnumString,
    celt: ULONG,
    rgelt: *mut LPOLESTR,
    pceltFetched: *mut ULONG,
) -> HRESULT {
    let hr: HRESULT;
    TRACE!(
        b"(%p)->(%d, %p, %p)\n\0".as_ptr() as *const i8,
        This,
        celt,
        rgelt,
        pceltFetched,
    );
    *pceltFetched = 0;
    hr = IEnumString_Next(This, celt, rgelt, pceltFetched);
    if hr == S_OK {
        *pceltFetched = celt;
    }
    hr
}

#[no_mangle]
pub unsafe extern "system" fn ISequentialStream_Read_Proxy(
    This: *mut ISequentialStream,
    pv: *mut c_void,
    cb: ULONG,
    pcbRead: *mut ULONG,
) -> HRESULT {
    let mut read: ULONG = 0;
    let hr: HRESULT;

    TRACE!(
        b"(%p)->(%p, %d, %p)\n\0".as_ptr() as *const i8,
        This,
        pv,
        cb,
        pcbRead,
    );

    hr = ISequentialStream_RemoteRead_Proxy(This, pv, cb, &mut read);
    if !pcbRead.is_null() {
        *pcbRead = read;
    }

    hr
}

#[no_mangle]
pub unsafe extern "system" fn ISequentialStream_Read_Stub(
    This: *mut ISequentialStream,
    pv: *mut u8,
    cb: ULONG,
    pcbRead: *mut ULONG,
) -> HRESULT {
    TRACE!(
        b"(%p)->(%p, %d, %p)\n\0".as_ptr() as *const i8,
        This,
        pv,
        cb,
        pcbRead,
    );
    ISequentialStream_Read(This, pv as *mut c_void, cb, pcbRead)
}

#[no_mangle]
pub unsafe extern "system" fn ISequentialStream_Write_Proxy(
    This: *mut ISequentialStream,
    pv: *const c_void,
    cb: ULONG,
    pcbWritten: *mut ULONG,
) -> HRESULT {
    let mut written: ULONG = 0;
    let hr: HRESULT;

    TRACE!(
        b"(%p)->(%p, %d, %p)\n\0".as_ptr() as *const i8,
        This,
        pv,
        cb,
        pcbWritten,
    );

    hr = ISequentialStream_RemoteWrite_Proxy(This, pv, cb, &mut written);
    if !pcbWritten.is_null() {
        *pcbWritten = written;
    }

    hr
}

#[no_mangle]
pub unsafe extern "system" fn ISequentialStream_Write_Stub(
    This: *mut ISequentialStream,
    pv: *const u8,
    cb: ULONG,
    pcbWritten: *mut ULONG,
) -> HRESULT {
    TRACE!(
        b"(%p)->(%p, %d, %p)\n\0".as_ptr() as *const i8,
        This,
        pv,
        cb,
        pcbWritten,
    );
    ISequentialStream_Write(This, pv as *const c_void, cb, pcbWritten)
}

#[no_mangle]
pub unsafe extern "system" fn IStream_Seek_Proxy(
    This: *mut IStream,
    dlibMove: LARGE_INTEGER,
    dwOrigin: DWORD,
    plibNewPosition: *mut ULARGE_INTEGER,
) -> HRESULT {
    let mut newpos: ULARGE_INTEGER = core::mem::zeroed();
    let hr: HRESULT;

    TRACE!(
        b"(%p)->(%s, %d, %p)\n\0".as_ptr() as *const i8,
        This,
        dwOrigin,
        plibNewPosition,
    );

    hr = IStream_RemoteSeek_Proxy(This, dlibMove, dwOrigin, &mut newpos);
    if !plibNewPosition.is_null() {
        *plibNewPosition = newpos;
    }

    hr
}

#[no_mangle]
pub unsafe extern "system" fn IStream_Seek_Stub(
    This: *mut IStream,
    dlibMove: LARGE_INTEGER,
    dwOrigin: DWORD,
    plibNewPosition: *mut ULARGE_INTEGER,
) -> HRESULT {
    TRACE!(
        b"(%p)->(%s, %d, %p)\n\0".as_ptr() as *const i8,
        This,
        dwOrigin,
        plibNewPosition,
    );
    IStream_Seek(This, dlibMove, dwOrigin, plibNewPosition)
}

#[no_mangle]
pub unsafe extern "system" fn IStream_CopyTo_Proxy(
    This: *mut IStream,
    pstm: *mut IStream,
    cb: ULARGE_INTEGER,
    pcbRead: *mut ULARGE_INTEGER,
    pcbWritten: *mut ULARGE_INTEGER,
) -> HRESULT {
    let mut read: ULARGE_INTEGER = core::mem::zeroed();
    let mut written: ULARGE_INTEGER = core::mem::zeroed();
    let hr: HRESULT;

    TRACE!(
        b"(%p)->(%p, %s, %p, %p)\n\0".as_ptr() as *const i8,
        This,
        pstm,
        pcbRead,
        pcbWritten,
    );

    hr = IStream_RemoteCopyTo_Proxy(
        This,
        pstm,
        cb,
        core::ptr::addr_of_mut!(read),
        core::ptr::addr_of_mut!(written),
    );
    if !pcbRead.is_null() {
        *pcbRead = read;
    }
    if !pcbWritten.is_null() {
        *pcbWritten = written;
    }

    hr
}

#[no_mangle]
pub unsafe extern "system" fn IStream_CopyTo_Stub(
    This: *mut IStream,
    pstm: *mut IStream,
    cb: ULARGE_INTEGER,
    pcbRead: *mut ULARGE_INTEGER,
    pcbWritten: *mut ULARGE_INTEGER,
) -> HRESULT {
    TRACE!(
        b"(%p)->(%p, %s, %p, %p)\n\0".as_ptr() as *const i8,
        This,
        pstm,
        pcbRead,
        pcbWritten,
    );

    IStream_CopyTo(This, pstm, cb, pcbRead, pcbWritten)
}

#[no_mangle]
pub unsafe extern "system" fn IEnumSTATSTG_Next_Proxy(
    This: *mut IEnumSTATSTG,
    celt: ULONG,
    rgelt: *mut STATSTG,
    pceltFetched: *mut ULONG,
) -> HRESULT {
    let mut fetched: ULONG = 0;
    TRACE!(
        b"(%p)->(%d, %p, %p)\n\0".as_ptr() as *const i8,
        This,
        celt,
        rgelt,
        pceltFetched,
    );
    let pceltFetched = if pceltFetched.is_null() {
        core::ptr::addr_of_mut!(fetched)
    } else {
        pceltFetched
    };
    IEnumSTATSTG_RemoteNext_Proxy(This, celt, rgelt, pceltFetched)
}

#[no_mangle]
pub unsafe extern "system" fn IEnumSTATSTG_Next_Stub(
    This: *mut IEnumSTATSTG,
    celt: ULONG,
    rgelt: *mut STATSTG,
    pceltFetched: *mut ULONG,
) -> HRESULT {
    let hr: HRESULT;
    TRACE!(
        b"(%p)->(%d, %p, %p)\n\0".as_ptr() as *const i8,
        This,
        celt,
        rgelt,
        pceltFetched,
    );
    *pceltFetched = 0;
    hr = IEnumSTATSTG_Next(This, celt, rgelt, pceltFetched);
    if hr == S_OK {
        *pceltFetched = celt;
    }
    hr
}

#[no_mangle]
pub unsafe extern "system" fn IStorage_OpenStream_Proxy(
    This: *mut IStorage,
    pwcsName: LPCOLESTR,
    reserved1: *mut c_void,
    grfMode: DWORD,
    reserved2: DWORD,
    ppstm: *mut *mut IStream,
) -> HRESULT {
    TRACE!(
        b"(%p)->(%s, %p, %08x, %d %p)\n\0".as_ptr() as *const i8,
        This,
        debugstr_w(pwcsName as *const c_void),
        reserved1,
        grfMode,
        reserved2,
        ppstm,
    );
    if !reserved1.is_null() {
        WARN!(b"reserved1 %p\n\0".as_ptr() as *const i8, reserved1);
    }

    IStorage_RemoteOpenStream_Proxy(
        This,
        pwcsName,
        0,
        core::ptr::null_mut(),
        grfMode,
        reserved2,
        ppstm,
    )
}

#[no_mangle]
pub unsafe extern "system" fn IStorage_OpenStream_Stub(
    This: *mut IStorage,
    pwcsName: LPCOLESTR,
    cbReserved1: ULONG,
    reserved1: *mut u8,
    grfMode: DWORD,
    reserved2: DWORD,
    ppstm: *mut *mut IStream,
) -> HRESULT {
    TRACE!(
        b"(%p)->(%s, %d, %p, %08x, %d %p)\n\0".as_ptr() as *const i8,
        This,
        debugstr_w(pwcsName as *const c_void),
        cbReserved1,
        reserved1,
        grfMode,
        reserved2,
        ppstm,
    );
    if cbReserved1 != 0 || !reserved1.is_null() {
        WARN!(
            b"cbReserved1 %d reserved1 %p\n\0".as_ptr() as *const i8,
            cbReserved1,
            reserved1,
        );
    }

    IStorage_OpenStream(
        This,
        pwcsName,
        core::ptr::null_mut(),
        grfMode,
        reserved2,
        ppstm,
    )
}

#[no_mangle]
pub unsafe extern "system" fn IStorage_EnumElements_Proxy(
    This: *mut IStorage,
    reserved1: DWORD,
    reserved2: *mut c_void,
    reserved3: DWORD,
    ppenum: *mut *mut IEnumSTATSTG,
) -> HRESULT {
    TRACE!(
        b"(%p)->(%d, %p, %d, %p)\n\0".as_ptr() as *const i8,
        This,
        reserved1,
        reserved2,
        reserved3,
        ppenum,
    );
    if !reserved2.is_null() {
        WARN!(b"reserved2 %p\n\0".as_ptr() as *const i8, reserved2);
    }

    IStorage_RemoteEnumElements_Proxy(
        This,
        reserved1,
        0,
        core::ptr::null_mut(),
        reserved3,
        ppenum,
    )
}

#[no_mangle]
pub unsafe extern "system" fn IStorage_EnumElements_Stub(
    This: *mut IStorage,
    reserved1: DWORD,
    cbReserved2: ULONG,
    reserved2: *mut u8,
    reserved3: DWORD,
    ppenum: *mut *mut IEnumSTATSTG,
) -> HRESULT {
    TRACE!(
        b"(%p)->(%d, %d, %p, %d, %p)\n\0".as_ptr() as *const i8,
        This,
        reserved1,
        cbReserved2,
        reserved2,
        reserved3,
        ppenum,
    );
    if cbReserved2 != 0 || !reserved2.is_null() {
        WARN!(
            b"cbReserved2 %d reserved2 %p\n\0".as_ptr() as *const i8,
            cbReserved2,
            reserved2,
        );
    }

    IStorage_EnumElements(This, reserved1, core::ptr::null_mut(), reserved3, ppenum)
}

#[no_mangle]
pub unsafe extern "system" fn ILockBytes_ReadAt_Proxy(
    This: *mut ILockBytes,
    ulOffset: ULARGE_INTEGER,
    pv: *mut c_void,
    cb: ULONG,
    pcbRead: *mut ULONG,
) -> HRESULT {
    let mut read: ULONG = 0;
    let hr: HRESULT;

    TRACE!(
        b"(%p)->(%s, %p, %d, %p)\n\0".as_ptr() as *const i8,
        This,
        pv,
        cb,
        pcbRead,
    );

    hr = ILockBytes_RemoteReadAt_Proxy(This, ulOffset, pv, cb, &mut read);
    if !pcbRead.is_null() {
        *pcbRead = read;
    }

    hr
}

#[no_mangle]
pub unsafe extern "system" fn ILockBytes_ReadAt_Stub(
    This: *mut ILockBytes,
    ulOffset: ULARGE_INTEGER,
    pv: *mut u8,
    cb: ULONG,
    pcbRead: *mut ULONG,
) -> HRESULT {
    TRACE!(
        b"(%p)->(%s, %p, %d, %p)\n\0".as_ptr() as *const i8,
        This,
        pv,
        cb,
        pcbRead,
    );
    ILockBytes_ReadAt(This, ulOffset, pv as *mut c_void, cb, pcbRead)
}

#[no_mangle]
pub unsafe extern "system" fn ILockBytes_WriteAt_Proxy(
    This: *mut ILockBytes,
    ulOffset: ULARGE_INTEGER,
    pv: *const c_void,
    cb: ULONG,
    pcbWritten: *mut ULONG,
) -> HRESULT {
    let mut written: ULONG = 0;
    let hr: HRESULT;

    TRACE!(
        b"(%p)->(%s, %p, %d, %p)\n\0".as_ptr() as *const i8,
        This,
        pv,
        cb,
        pcbWritten,
    );

    hr = ILockBytes_RemoteWriteAt_Proxy(This, ulOffset, pv, cb, &mut written);
    if !pcbWritten.is_null() {
        *pcbWritten = written;
    }

    hr
}

#[no_mangle]
pub unsafe extern "system" fn ILockBytes_WriteAt_Stub(
    This: *mut ILockBytes,
    ulOffset: ULARGE_INTEGER,
    pv: *const u8,
    cb: ULONG,
    pcbWritten: *mut ULONG,
) -> HRESULT {
    TRACE!(
        b"(%p)->(%s, %p, %d, %p)\n\0".as_ptr() as *const i8,
        This,
        pv,
        cb,
        pcbWritten,
    );
    ILockBytes_WriteAt(This, ulOffset, pv as *const c_void, cb, pcbWritten)
}

#[no_mangle]
pub unsafe extern "system" fn IFillLockBytes_FillAppend_Proxy(
    This: *mut IFillLockBytes,
    pv: *const c_void,
    cb: ULONG,
    pcbWritten: *mut ULONG,
) -> HRESULT {
    let mut written: ULONG = 0;
    let hr: HRESULT;

    TRACE!(
        b"(%p)->(%p, %d, %p)\n\0".as_ptr() as *const i8,
        This,
        pv,
        cb,
        pcbWritten,
    );

    hr = IFillLockBytes_RemoteFillAppend_Proxy(This, pv, cb, &mut written);
    if !pcbWritten.is_null() {
        *pcbWritten = written;
    }

    hr
}

#[no_mangle]
pub unsafe extern "system" fn IFillLockBytes_FillAppend_Stub(
    This: *mut IFillLockBytes,
    pv: *const u8,
    cb: ULONG,
    pcbWritten: *mut ULONG,
) -> HRESULT {
    TRACE!(
        b"(%p)->(%p, %d, %p)\n\0".as_ptr() as *const i8,
        This,
        pv,
        cb,
        pcbWritten,
    );
    IFillLockBytes_FillAppend(This, pv as *const c_void, cb, pcbWritten)
}

#[no_mangle]
pub unsafe extern "system" fn IFillLockBytes_FillAt_Proxy(
    This: *mut IFillLockBytes,
    ulOffset: ULARGE_INTEGER,
    pv: *const c_void,
    cb: ULONG,
    pcbWritten: *mut ULONG,
) -> HRESULT {
    let mut written: ULONG = 0;
    let hr: HRESULT;

    TRACE!(
        b"(%p)->(%s, %p, %d, %p)\n\0".as_ptr() as *const i8,
        This,
        pv,
        cb,
        pcbWritten,
    );

    hr = IFillLockBytes_RemoteFillAt_Proxy(This, ulOffset, pv, cb, &mut written);
    if !pcbWritten.is_null() {
        *pcbWritten = written;
    }

    hr
}

#[no_mangle]
pub unsafe extern "system" fn IFillLockBytes_FillAt_Stub(
    This: *mut IFillLockBytes,
    ulOffset: ULARGE_INTEGER,
    pv: *const u8,
    cb: ULONG,
    pcbWritten: *mut ULONG,
) -> HRESULT {
    TRACE!(
        b"(%p)->(%s, %p, %d, %p)\n\0".as_ptr() as *const i8,
        This,
        pv,
        cb,
        pcbWritten,
    );
    IFillLockBytes_FillAt(This, ulOffset, pv as *const c_void, cb, pcbWritten)
}

#[no_mangle]
pub unsafe extern "system" fn IEnumFORMATETC_Next_Proxy(
    This: *mut IEnumFORMATETC,
    celt: ULONG,
    rgelt: *mut FORMATETC,
    pceltFetched: *mut ULONG,
) -> HRESULT {
    let mut fetched: ULONG = 0;
    let pceltFetched = if pceltFetched.is_null() {
        core::ptr::addr_of_mut!(fetched)
    } else {
        pceltFetched
    };
    IEnumFORMATETC_RemoteNext_Proxy(This, celt, rgelt, pceltFetched)
}

#[no_mangle]
pub unsafe extern "system" fn IEnumFORMATETC_Next_Stub(
    This: *mut IEnumFORMATETC,
    celt: ULONG,
    rgelt: *mut FORMATETC,
    pceltFetched: *mut ULONG,
) -> HRESULT {
    let hr: HRESULT;
    *pceltFetched = 0;
    hr = IEnumFORMATETC_Next(This, celt, rgelt, pceltFetched);
    if hr == S_OK {
        *pceltFetched = celt;
    }
    hr
}

#[no_mangle]
pub unsafe extern "system" fn IEnumSTATDATA_Next_Proxy(
    This: *mut IEnumSTATDATA,
    celt: ULONG,
    rgelt: *mut STATDATA,
    pceltFetched: *mut ULONG,
) -> HRESULT {
    let mut fetched: ULONG = 0;
    TRACE!(
        b"(%p)->(%d, %p, %p)\n\0".as_ptr() as *const i8,
        This,
        celt,
        rgelt,
        pceltFetched,
    );
    let pceltFetched = if pceltFetched.is_null() {
        core::ptr::addr_of_mut!(fetched)
    } else {
        pceltFetched
    };
    IEnumSTATDATA_RemoteNext_Proxy(This, celt, rgelt, pceltFetched)
}

#[no_mangle]
pub unsafe extern "system" fn IEnumSTATDATA_Next_Stub(
    This: *mut IEnumSTATDATA,
    celt: ULONG,
    rgelt: *mut STATDATA,
    pceltFetched: *mut ULONG,
) -> HRESULT {
    let hr: HRESULT;
    TRACE!(
        b"(%p)->(%d, %p, %p)\n\0".as_ptr() as *const i8,
        This,
        celt,
        rgelt,
        pceltFetched,
    );
    *pceltFetched = 0;
    hr = IEnumSTATDATA_Next(This, celt, rgelt, pceltFetched);
    if hr == S_OK {
        *pceltFetched = celt;
    }
    hr
}

#[no_mangle]
pub unsafe extern "system" fn IAdviseSink_OnDataChange_Proxy(
    This: *mut IAdviseSink,
    pFormatetc: *mut FORMATETC,
    pStgmed: *mut STGMEDIUM,
) {
    TRACE!(
        b"(%p)->(%p, %p)\n\0".as_ptr() as *const i8,
        This,
        pFormatetc,
        pStgmed,
    );
    IAdviseSink_RemoteOnDataChange_Proxy(This, pFormatetc, pStgmed);
}

#[no_mangle]
pub unsafe extern "system" fn IAdviseSink_OnDataChange_Stub(
    This: *mut IAdviseSink,
    pFormatetc: *mut FORMATETC,
    pStgmed: *mut ASYNC_STGMEDIUM,
) -> HRESULT {
    TRACE!(
        b"(%p)->(%p, %p)\n\0".as_ptr() as *const i8,
        This,
        pFormatetc,
        pStgmed,
    );
    IAdviseSink_OnDataChange(This, pFormatetc, pStgmed);
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn IAdviseSink_OnViewChange_Proxy(
    This: *mut IAdviseSink,
    dwAspect: DWORD,
    lindex: LONG,
) {
    TRACE!(
        b"(%p)->(%d, %d)\n\0".as_ptr() as *const i8,
        This,
        dwAspect,
        lindex,
    );
    IAdviseSink_RemoteOnViewChange_Proxy(This, dwAspect, lindex);
}

#[no_mangle]
pub unsafe extern "system" fn IAdviseSink_OnViewChange_Stub(
    This: *mut IAdviseSink,
    dwAspect: DWORD,
    lindex: LONG,
) -> HRESULT {
    TRACE!(
        b"(%p)->(%d, %d)\n\0".as_ptr() as *const i8,
        This,
        dwAspect,
        lindex,
    );
    IAdviseSink_OnViewChange(This, dwAspect, lindex);
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn IAdviseSink_OnRename_Proxy(
    This: *mut IAdviseSink,
    pmk: *mut IMoniker,
) {
    TRACE!(b"(%p)->(%p)\n\0".as_ptr() as *const i8, This, pmk);
    IAdviseSink_RemoteOnRename_Proxy(This, pmk);
}

#[no_mangle]
pub unsafe extern "system" fn IAdviseSink_OnRename_Stub(
    This: *mut IAdviseSink,
    pmk: *mut IMoniker,
) -> HRESULT {
    TRACE!(b"(%p)->(%p)\n\0".as_ptr() as *const i8, This, pmk);
    IAdviseSink_OnRename(This, pmk);
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn IAdviseSink_OnSave_Proxy(This: *mut IAdviseSink) {
    TRACE!(b"(%p)\n\0".as_ptr() as *const i8, This);
    IAdviseSink_RemoteOnSave_Proxy(This);
}

#[no_mangle]
pub unsafe extern "system" fn IAdviseSink_OnSave_Stub(This: *mut IAdviseSink) -> HRESULT {
    TRACE!(b"(%p)\n\0".as_ptr() as *const i8, This);
    IAdviseSink_OnSave(This);
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn IAdviseSink_OnClose_Proxy(This: *mut IAdviseSink) {
    TRACE!(b"(%p)\n\0".as_ptr() as *const i8, This);
    IAdviseSink_RemoteOnClose_Proxy(This);
}

#[no_mangle]
pub unsafe extern "system" fn IAdviseSink_OnClose_Stub(This: *mut IAdviseSink) -> HRESULT {
    TRACE!(b"(%p)\n\0".as_ptr() as *const i8, This);
    IAdviseSink_OnClose(This);
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn IAdviseSink2_OnLinkSrcChange_Proxy(
    This: *mut IAdviseSink2,
    pmk: *mut IMoniker,
) {
    TRACE!(b"(%p)->(%p)\n\0".as_ptr() as *const i8, This, pmk);
    IAdviseSink2_RemoteOnLinkSrcChange_Proxy(This, pmk);
}

#[no_mangle]
pub unsafe extern "system" fn IAdviseSink2_OnLinkSrcChange_Stub(
    This: *mut IAdviseSink2,
    pmk: *mut IMoniker,
) -> HRESULT {
    TRACE!(b"(%p)->(%p)\n\0".as_ptr() as *const i8, This, pmk);
    IAdviseSink2_OnLinkSrcChange(This, pmk);
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn IDataObject_GetData_Proxy(
    This: *mut IDataObject,
    pformatetcIn: *mut FORMATETC,
    pmedium: *mut STGMEDIUM,
) -> HRESULT {
    TRACE!(
        b"(%p)->(%p, %p)\n\0".as_ptr() as *const i8,
        This,
        pformatetcIn,
        pmedium,
    );
    IDataObject_RemoteGetData_Proxy(This, pformatetcIn, pmedium)
}

#[no_mangle]
pub unsafe extern "system" fn IDataObject_GetData_Stub(
    This: *mut IDataObject,
    pformatetcIn: *mut FORMATETC,
    pRemoteMedium: *mut STGMEDIUM,
) -> HRESULT {
    TRACE!(
        b"(%p)->(%p, %p)\n\0".as_ptr() as *const i8,
        This,
        pformatetcIn,
        pRemoteMedium,
    );
    IDataObject_GetData(This, pformatetcIn, pRemoteMedium)
}

#[no_mangle]
pub unsafe extern "system" fn IDataObject_GetDataHere_Proxy(
    iface: *mut IDataObject,
    fmt: *mut FORMATETC,
    med: *mut STGMEDIUM,
) -> HRESULT {
    let release: *mut IUnknown;
    let mut stg: *mut IStorage = core::ptr::null_mut();
    let hr: HRESULT;

    TRACE!(b"(%p)->(%p, %p)\n\0".as_ptr() as *const i8, iface, fmt, med);

    if ((*med).tymed & (TYMED_HGLOBAL | TYMED_FILE | TYMED_ISTREAM | TYMED_ISTORAGE)) == 0 {
        return DV_E_TYMED;
    }
    if (*med).tymed != (*fmt).tymed {
        return DV_E_TYMED;
    }

    release = (*med).pUnkForRelease;
    (*med).pUnkForRelease = core::ptr::null_mut();

    if (*med).tymed == TYMED_ISTREAM || (*med).tymed == TYMED_ISTORAGE {
        stg = (*med).__bindgen_anon_1.pstg; /* This may actually be a stream, but that's ok */
        if !stg.is_null() {
            IStorage_AddRef(stg);
        }
    }

    hr = IDataObject_RemoteGetDataHere_Proxy(iface, fmt, med);

    (*med).pUnkForRelease = release;
    if !stg.is_null() {
        if !(*med).__bindgen_anon_1.pstg.is_null() {
            IStorage_Release((*med).__bindgen_anon_1.pstg);
        }
        (*med).__bindgen_anon_1.pstg = stg;
    }

    hr
}

#[no_mangle]
pub unsafe extern "system" fn IDataObject_GetDataHere_Stub(
    This: *mut IDataObject,
    pformatetc: *mut FORMATETC,
    pRemoteMedium: *mut STGMEDIUM,
) -> HRESULT {
    TRACE!(
        b"(%p)->(%p, %p)\n\0".as_ptr() as *const i8,
        This,
        pformatetc,
        pRemoteMedium,
    );
    IDataObject_GetDataHere(This, pformatetc, pRemoteMedium)
}

#[no_mangle]
pub unsafe extern "system" fn IDataObject_SetData_Proxy(
    _This: *mut IDataObject,
    _pformatetc: *mut FORMATETC,
    _pmedium: *mut STGMEDIUM,
    _fRelease: BOOL,
) -> HRESULT {
    FIXME!(b":stub\n\0".as_ptr() as *const i8);
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn IDataObject_SetData_Stub(
    _This: *mut IDataObject,
    _pformatetc: *mut FORMATETC,
    _pmedium: *mut FLAG_STGMEDIUM,
    _fRelease: BOOL,
) -> HRESULT {
    FIXME!(b":stub\n\0".as_ptr() as *const i8);
    E_NOTIMPL
}

/* call_as/local stubs for oleidl.idl */

#[no_mangle]
pub unsafe extern "system" fn IOleInPlaceActiveObject_TranslateAccelerator_Proxy(
    This: *mut IOleInPlaceActiveObject,
    _lpmsg: LPMSG,
) -> HRESULT {
    TRACE!(b"(%p %p)\n\0".as_ptr() as *const i8, This, lpmsg);
    IOleInPlaceActiveObject_RemoteTranslateAccelerator_Proxy(This)
}

#[no_mangle]
pub unsafe extern "system" fn IOleInPlaceActiveObject_TranslateAccelerator_Stub(
    _This: *mut IOleInPlaceActiveObject,
) -> HRESULT {
    TRACE!(b"(%p)\n\0".as_ptr() as *const i8, This);
    S_FALSE
}

#[no_mangle]
pub unsafe extern "system" fn IOleInPlaceActiveObject_ResizeBorder_Proxy(
    _This: *mut IOleInPlaceActiveObject,
    _prcBorder: LPCRECT,
    _pUIWindow: *mut IOleInPlaceUIWindow,
    _fFrameWindow: BOOL,
) -> HRESULT {
    FIXME!(b":stub\n\0".as_ptr() as *const i8);
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn IOleInPlaceActiveObject_ResizeBorder_Stub(
    _This: *mut IOleInPlaceActiveObject,
    _prcBorder: LPCRECT,
    _riid: REFIID,
    _pUIWindow: *mut IOleInPlaceUIWindow,
    _fFrameWindow: BOOL,
) -> HRESULT {
    FIXME!(b":stub\n\0".as_ptr() as *const i8);
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn IOleCache2_UpdateCache_Proxy(
    This: *mut IOleCache2,
    pDataObject: LPDATAOBJECT,
    grfUpdf: DWORD,
    pReserved: LPVOID,
) -> HRESULT {
    TRACE!(
        b"(%p, %p, 0x%08x, %p)\n\0".as_ptr() as *const i8,
        This,
        pDataObject,
        grfUpdf,
        pReserved,
    );
    IOleCache2_RemoteUpdateCache_Proxy(This, pDataObject, grfUpdf, pReserved as LONG_PTR)
}

#[no_mangle]
pub unsafe extern "system" fn IOleCache2_UpdateCache_Stub(
    This: *mut IOleCache2,
    pDataObject: LPDATAOBJECT,
    grfUpdf: DWORD,
    pReserved: LONG_PTR,
) -> HRESULT {
    TRACE!(
        b"(%p, %p, 0x%08x, %li)\n\0".as_ptr() as *const i8,
        This,
        pDataObject,
        grfUpdf,
        pReserved,
    );
    IOleCache2_UpdateCache(This, pDataObject, grfUpdf, pReserved as LPVOID)
}

#[no_mangle]
pub unsafe extern "system" fn IEnumOLEVERB_Next_Proxy(
    This: *mut IEnumOLEVERB,
    celt: ULONG,
    rgelt: LPOLEVERB,
    pceltFetched: *mut ULONG,
) -> HRESULT {
    let mut fetched: ULONG = 0;
    TRACE!(
        b"(%p)->(%d, %p, %p)\n\0".as_ptr() as *const i8,
        This,
        celt,
        rgelt,
        pceltFetched,
    );
    let pceltFetched = if pceltFetched.is_null() {
        core::ptr::addr_of_mut!(fetched)
    } else {
        pceltFetched
    };
    IEnumOLEVERB_RemoteNext_Proxy(This, celt, rgelt, pceltFetched)
}

#[no_mangle]
pub unsafe extern "system" fn IEnumOLEVERB_Next_Stub(
    This: *mut IEnumOLEVERB,
    celt: ULONG,
    rgelt: LPOLEVERB,
    pceltFetched: *mut ULONG,
) -> HRESULT {
    let hr: HRESULT;
    TRACE!(
        b"(%p)->(%d, %p, %p)\n\0".as_ptr() as *const i8,
        This,
        celt,
        rgelt,
        pceltFetched,
    );
    *pceltFetched = 0;
    hr = IEnumOLEVERB_Next(This, celt, rgelt, pceltFetched);
    if hr == S_OK {
        *pceltFetched = celt;
    }
    hr
}

#[no_mangle]
pub unsafe extern "system" fn IViewObject_Draw_Proxy(
    _This: *mut IViewObject,
    _dwDrawAspect: DWORD,
    _lindex: LONG,
    _pvAspect: *mut c_void,
    _ptd: *mut DVTARGETDEVICE,
    _hdcTargetDev: HDC,
    _hdcDraw: HDC,
    _lprcBounds: LPCRECTL,
    _lprcWBounds: LPCRECTL,
    _pfnContinue: Option<unsafe extern "system" fn(ULONG_PTR) -> BOOL>,
    _dwContinue: ULONG_PTR,
) -> HRESULT {
    FIXME!(b":stub\n\0".as_ptr() as *const i8);
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn IViewObject_Draw_Stub(
    _This: *mut IViewObject,
    _dwDrawAspect: DWORD,
    _lindex: LONG,
    _pvAspect: ULONG_PTR,
    _ptd: *mut DVTARGETDEVICE,
    _hdcTargetDev: ULONG_PTR,
    _hdcDraw: ULONG_PTR,
    _lprcBounds: LPCRECTL,
    _lprcWBounds: LPCRECTL,
    _pContinue: *mut IContinue,
) -> HRESULT {
    FIXME!(b":stub\n\0".as_ptr() as *const i8);
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn IViewObject_GetColorSet_Proxy(
    _This: *mut IViewObject,
    _dwDrawAspect: DWORD,
    _lindex: LONG,
    _pvAspect: *mut c_void,
    _ptd: *mut DVTARGETDEVICE,
    _hicTargetDev: HDC,
    _ppColorSet: *mut *mut LOGPALETTE,
) -> HRESULT {
    FIXME!(b":stub\n\0".as_ptr() as *const i8);
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn IViewObject_GetColorSet_Stub(
    _This: *mut IViewObject,
    _dwDrawAspect: DWORD,
    _lindex: LONG,
    _pvAspect: ULONG_PTR,
    _ptd: *mut DVTARGETDEVICE,
    _hicTargetDev: ULONG_PTR,
    _ppColorSet: *mut *mut LOGPALETTE,
) -> HRESULT {
    FIXME!(b":stub\n\0".as_ptr() as *const i8);
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn IViewObject_Freeze_Proxy(
    _This: *mut IViewObject,
    _dwDrawAspect: DWORD,
    _lindex: LONG,
    _pvAspect: *mut c_void,
    _pdwFreeze: *mut DWORD,
) -> HRESULT {
    FIXME!(b":stub\n\0".as_ptr() as *const i8);
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn IViewObject_Freeze_Stub(
    _This: *mut IViewObject,
    _dwDrawAspect: DWORD,
    _lindex: LONG,
    _pvAspect: ULONG_PTR,
    _pdwFreeze: *mut DWORD,
) -> HRESULT {
    FIXME!(b":stub\n\0".as_ptr() as *const i8);
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn IViewObject_GetAdvise_Proxy(
    _This: *mut IViewObject,
    _pAspects: *mut DWORD,
    _pAdvf: *mut DWORD,
    _ppAdvSink: *mut *mut IAdviseSink,
) -> HRESULT {
    FIXME!(b":stub\n\0".as_ptr() as *const i8);
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn IViewObject_GetAdvise_Stub(
    _This: *mut IViewObject,
    _pAspects: *mut DWORD,
    _pAdvf: *mut DWORD,
    _ppAdvSink: *mut *mut IAdviseSink,
) -> HRESULT {
    FIXME!(b":stub\n\0".as_ptr() as *const i8);
    E_NOTIMPL
}
