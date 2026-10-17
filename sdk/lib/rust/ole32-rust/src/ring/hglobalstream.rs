//! HGLOBAL Stream implementation — translation of dll/win32/ole32/hglobalstream.c
//!
//! An IStream backed by an HGLOBAL. The HGLOBAL itself lives in a refcounted
//! `handle_wrapper` so that clones share one buffer.
use crate::prelude::*;
use core::ffi::c_void;

// ---------------------------------------------------------------------------
// Local bits the shared surface does not (yet) carry.
// ---------------------------------------------------------------------------

// winbase.h GlobalAlloc flags missing from crate::win32 (which has GMEM_MOVEABLE).
const GMEM_NODISCARD: u32 = 0x0020;
const GMEM_SHARE: u32 = 0x2000;

/// The C writes a literal source-name into `lock.DebugInfo->Spare[0]` (and clears
/// it again before DeleteCriticalSection). That is a debugger annotation only — it
/// has no effect on locking. crate::win32's CRITICAL_SECTION keeps DebugInfo
/// opaque (`*mut c_void`), so — exactly as the marshal lane already does for
/// DEBUG_SET_CRITSEC_NAME / DEBUG_CLEAR_CRITSEC_NAME (marshal/types.rs) — these
/// are translated as no-ops.
#[inline]
unsafe fn debug_set_critsec_name(_cs: *mut CRITICAL_SECTION, _name: &str) {}
#[inline]
unsafe fn debug_clear_critsec_name(_cs: *mut CRITICAL_SECTION) {}

/// com-sys spells the IStreamVtbl slots `unsafe extern "stdcall" fn` on i386 and
/// `unsafe extern "C" fn` on amd64/arm64 — i.e. exactly what STDMETHODCALLTYPE
/// compiles to per arch. `extern "system"` *lowers* to the same ABI, but rustc
/// keeps it a DISTINCT fn type that will not unify with the bindgen slot type, so
/// the vtbl impls must spell the ABI the way com-sys does. This macro does that
/// once, so no method has to carry a per-arch cfg.
macro_rules! stdmethod {
    ($(
        $(#[$attr:meta])*
        unsafe fn $name:ident ( $($arg:ident : $ty:ty),* $(,)? ) -> $ret:ty $body:block
    )*) => {$(
        $(#[$attr])*
        unsafe extern "system" fn $name($($arg: $ty),*) -> $ret $body
    )*};
}

// ---------------------------------------------------------------------------
// struct handle_wrapper
// ---------------------------------------------------------------------------

#[repr(C)]
pub struct handle_wrapper {
    pub ref_: LONG,
    pub hglobal: HGLOBAL,
    pub size: ULONG,
    pub delete_on_release: BOOL,
    pub lock: CRITICAL_SECTION,
}

unsafe fn handle_addref(handle: *mut handle_wrapper) {
    InterlockedIncrement(core::ptr::addr_of_mut!((*handle).ref_));
}

unsafe fn handle_release(handle: *mut handle_wrapper) {
    let ref_ = InterlockedDecrement(core::ptr::addr_of_mut!((*handle).ref_)) as ULONG;

    if ref_ == 0 {
        if (*handle).delete_on_release != 0 {
            GlobalFree((*handle).hglobal);
            (*handle).hglobal = core::ptr::null_mut();
        }

        debug_clear_critsec_name(core::ptr::addr_of_mut!((*handle).lock));
        DeleteCriticalSection(core::ptr::addr_of_mut!((*handle).lock));
        HeapFree(GetProcessHeap(), 0, handle as LPVOID);
    }
}

unsafe fn handle_read(
    handle: *mut handle_wrapper,
    pos: *mut ULONG,
    dest: *mut c_void,
    len: ULONG,
) -> ULONG {
    let mut len = len;

    EnterCriticalSection(core::ptr::addr_of_mut!((*handle).lock));

    if *pos < (*handle).size {
        len = core::cmp::min((*handle).size - *pos, len);
    } else {
        len = 0;
    }

    let source: LPVOID = GlobalLock((*handle).hglobal);
    if !source.is_null() {
        // memcpy(dest, (char *)source + *pos, len);
        core::ptr::copy_nonoverlapping(
            (source as *const u8).add(*pos as usize),
            dest as *mut u8,
            len as usize,
        );
        *pos += len;
        GlobalUnlock((*handle).hglobal);
    } else {
        WARN!("read from invalid hglobal %p\n", (*handle).hglobal);
        len = 0;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*handle).lock));
    len
}

unsafe fn handle_write(
    handle: *mut handle_wrapper,
    pos: *mut ULONG,
    source: *const c_void,
    len: ULONG,
) -> ULONG {
    let mut len = len;

    if len == 0 {
        return 0;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*handle).lock));

    // The C's `goto done` (OOM path) skips the copy but still leaves the critsec.
    'done: {
        if (*pos).wrapping_add(len) > (*handle).size {
            let hglobal: HGLOBAL = GlobalReAlloc(
                (*handle).hglobal,
                (*pos).wrapping_add(len) as SIZE_T,
                GMEM_MOVEABLE,
            );
            if !hglobal.is_null() {
                (*handle).hglobal = hglobal;
                (*handle).size = (*pos).wrapping_add(len);
            } else {
                len = 0;
                break 'done;
            }
        }

        let dest: LPVOID = GlobalLock((*handle).hglobal);
        if !dest.is_null() {
            // memcpy((char *)dest + *pos, source, len);
            core::ptr::copy_nonoverlapping(
                source as *const u8,
                (dest as *mut u8).add(*pos as usize),
                len as usize,
            );
            *pos += len;
            GlobalUnlock((*handle).hglobal);
        } else {
            WARN!("write to invalid hglobal %p\n", (*handle).hglobal);
            /* len = 0; */
        }
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*handle).lock));
    len
}

unsafe fn handle_gethglobal(handle: *mut handle_wrapper) -> HGLOBAL {
    (*handle).hglobal
}

unsafe fn handle_setsize(handle: *mut handle_wrapper, size: ULONG) -> HRESULT {
    let mut hr: HRESULT = S_OK;

    EnterCriticalSection(core::ptr::addr_of_mut!((*handle).lock));

    if (*handle).size != size {
        let hglobal: HGLOBAL = GlobalReAlloc((*handle).hglobal, size as SIZE_T, GMEM_MOVEABLE);
        if !hglobal.is_null() {
            (*handle).hglobal = hglobal;
            (*handle).size = size;
        } else {
            hr = E_OUTOFMEMORY;
        }
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*handle).lock));
    hr
}

unsafe fn handle_getsize(handle: *mut handle_wrapper) -> ULONG {
    (*handle).size
}

unsafe fn handle_create(hglobal: HGLOBAL, delete_on_release: BOOL) -> *mut handle_wrapper {
    let handle = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<handle_wrapper>() as usize,
    ) as *mut handle_wrapper;
    if !handle.is_null() {
        (*handle).ref_ = 1;
        (*handle).hglobal = hglobal;
        (*handle).size = GlobalSize(hglobal) as ULONG;
        (*handle).delete_on_release = delete_on_release;
        InitializeCriticalSection(core::ptr::addr_of_mut!((*handle).lock));
        debug_set_critsec_name(
            core::ptr::addr_of_mut!((*handle).lock),
            "hglobalstream.c: handle_wrapper.lock",
        );
    }
    handle
}

// ---------------------------------------------------------------------------
// HGLOBALStreamImpl — IStream over an HGLOBAL.
// ---------------------------------------------------------------------------

#[repr(C)]
pub struct HGLOBALStreamImpl {
    pub IStream_iface: IStream,
    pub ref_: LONG,

    pub handle: *mut handle_wrapper,

    /* current position of the cursor */
    pub currentPosition: ULARGE_INTEGER,
}

#[inline]
unsafe fn impl_from_IStream(iface: *mut IStream) -> *mut HGLOBALStreamImpl {
    // CONTAINING_RECORD(iface, HGLOBALStreamImpl, IStream_iface)
    (iface as *mut u8)
        .offset(-(core::mem::offset_of!(HGLOBALStreamImpl, IStream_iface) as isize))
        as *mut HGLOBALStreamImpl
}

unsafe fn HGLOBALStreamImpl_Create() -> *mut HGLOBALStreamImpl {
    let This = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<HGLOBALStreamImpl>() as usize,
    ) as *mut HGLOBALStreamImpl;
    if !This.is_null() {
        (*This).IStream_iface.lpVtbl =
            &HGLOBALStreamImplVtbl as *const IStreamVtbl as *mut IStreamVtbl;
        (*This).ref_ = 1;
    }
    This
}

stdmethod! {

unsafe fn HGLOBALStreamImpl_QueryInterface(
    iface: *mut IStream,
    riid: *const IID,
    ppvObject: *mut *mut c_void,
) -> HRESULT {
    let This = impl_from_IStream(iface);

    if ppvObject.is_null() {
        return E_INVALIDARG;
    }

    *ppvObject = core::ptr::null_mut();

    if IsEqualIID(&IID_IUnknown, riid)
        || IsEqualIID(&IID_ISequentialStream, riid)
        || IsEqualIID(&IID_IStream, riid)
    {
        *ppvObject = core::ptr::addr_of_mut!((*This).IStream_iface) as *mut c_void;
    }

    if (*ppvObject).is_null() {
        return E_NOINTERFACE;
    }

    IStream_AddRef(iface);

    S_OK
}

unsafe fn HGLOBALStreamImpl_AddRef(iface: *mut IStream) -> ULONG {
    let This = impl_from_IStream(iface);
    InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref_)) as ULONG
}

unsafe fn HGLOBALStreamImpl_Release(iface: *mut IStream) -> ULONG {
    let This = impl_from_IStream(iface);
    let ref_ = InterlockedDecrement(core::ptr::addr_of_mut!((*This).ref_)) as ULONG;

    if ref_ == 0 {
        handle_release((*This).handle);
        HeapFree(GetProcessHeap(), 0, This as LPVOID);
    }

    ref_
}

/// ISequentialStream::Read — reads a block at the current position, then moves the
/// current position to the end of the read block.
unsafe fn HGLOBALStreamImpl_Read(
    iface: *mut IStream,
    pv: *mut c_void,
    cb: ULONG,
    pcbRead: *mut ULONG,
) -> HRESULT {
    let This = impl_from_IStream(iface);

    TRACE!("(%p, %p, %d, %p)\n", iface, pv, cb, pcbRead);

    let num_bytes = handle_read(
        (*This).handle,
        core::ptr::addr_of_mut!((*This).currentPosition.u.LowPart),
        pv,
        cb,
    );
    if !pcbRead.is_null() {
        *pcbRead = num_bytes;
    }

    S_OK
}

/// ISequentialStream::Write — writes a block at the current position, growing the
/// stream if it is too small to fit, then moves the current position to the end of
/// the written block.
unsafe fn HGLOBALStreamImpl_Write(
    iface: *mut IStream,
    pv: *const c_void,
    cb: ULONG,
    pcbWritten: *mut ULONG,
) -> HRESULT {
    let This = impl_from_IStream(iface);

    TRACE!("(%p, %p, %d, %p)\n", iface, pv, cb, pcbWritten);

    let num_bytes = handle_write(
        (*This).handle,
        core::ptr::addr_of_mut!((*This).currentPosition.u.LowPart),
        pv,
        cb,
    );
    if !pcbWritten.is_null() {
        *pcbWritten = num_bytes;
    }

    if num_bytes < cb {
        E_OUTOFMEMORY
    } else {
        S_OK
    }
}

/// IStream::Seek — moves the stream pointer. Landing past the end of the stream is
/// legal; the next Write grows the stream (this is how it is documented).
unsafe fn HGLOBALStreamImpl_Seek(
    iface: *mut IStream,
    dlibMove: LARGE_INTEGER,
    dwOrigin: DWORD,
    plibNewPosition: *mut ULARGE_INTEGER,
) -> HRESULT {
    let This = impl_from_IStream(iface);

    let mut newPosition: ULARGE_INTEGER = (*This).currentPosition;
    let mut hr: HRESULT = S_OK;

    TRACE!(
        "(%p, %x%08x, %d, %p)\n",
        iface,
        dlibMove.u.HighPart,
        dlibMove.u.LowPart,
        dwOrigin,
        plibNewPosition
    );

    // The C's `goto end` — every exit path still publishes plibNewPosition.
    'end: {
        //
        // The file pointer is moved depending on the given "function" parameter.
        //
        match dwOrigin {
            STREAM_SEEK_SET => {
                newPosition.u.HighPart = 0;
                newPosition.u.LowPart = 0;
            }
            STREAM_SEEK_CUR => {}
            STREAM_SEEK_END => {
                newPosition.QuadPart = handle_getsize((*This).handle) as ULONGLONG;
            }
            _ => {
                hr = STG_E_SEEKERROR;
                break 'end;
            }
        }

        newPosition.u.HighPart = 0;
        // C: `newPosition.u.LowPart += dlibMove.QuadPart;` — the 64-bit add is
        // truncated back into the 32-bit LowPart, and that wrap is exactly what the
        // backwards-past-start check below detects.
        newPosition.u.LowPart = newPosition.u.LowPart.wrapping_add(dlibMove.QuadPart as DWORD);

        if dlibMove.u.LowPart >= 0x80000000 && newPosition.u.LowPart >= dlibMove.u.LowPart {
            /* We tried to seek backwards and went past the start. */
            hr = STG_E_SEEKERROR;
            break 'end;
        }

        (*This).currentPosition = newPosition;
    }

    if !plibNewPosition.is_null() {
        *plibNewPosition = (*This).currentPosition;
    }

    hr
}

/// IStream::SetSize — HighPart is ignored, as shown in tests.
unsafe fn HGLOBALStreamImpl_SetSize(iface: *mut IStream, libNewSize: ULARGE_INTEGER) -> HRESULT {
    let This = impl_from_IStream(iface);

    TRACE!("(%p, %d)\n", iface, libNewSize.u.LowPart);

    handle_setsize((*This).handle, libNewSize.u.LowPart)
}

/// IStream::CopyTo — copies `cb` bytes to the `pstm` IStream.
unsafe fn HGLOBALStreamImpl_CopyTo(
    iface: *mut IStream,
    pstm: *mut IStream,
    cb: ULARGE_INTEGER,
    pcbRead: *mut ULARGE_INTEGER,
    pcbWritten: *mut ULARGE_INTEGER,
) -> HRESULT {
    let mut cb = cb;
    let mut hr: HRESULT = S_OK;
    let mut tmpBuffer: [BYTE; 128] = [0; 128];
    let mut bytesRead: ULONG = 0;
    let mut bytesWritten: ULONG = 0;
    let mut copySize: ULONG;
    let mut totalBytesRead: ULARGE_INTEGER = core::mem::zeroed();
    let mut totalBytesWritten: ULARGE_INTEGER = core::mem::zeroed();

    TRACE!(
        "(%p, %p, %d, %p, %p)\n",
        iface,
        pstm,
        cb.u.LowPart,
        pcbRead,
        pcbWritten
    );

    if pstm.is_null() {
        return STG_E_INVALIDPOINTER;
    }

    totalBytesRead.QuadPart = 0;
    totalBytesWritten.QuadPart = 0;

    while cb.QuadPart > 0 {
        if cb.QuadPart >= core::mem::size_of_val(&tmpBuffer) as ULONGLONG {
            copySize = core::mem::size_of_val(&tmpBuffer) as ULONG;
        } else {
            copySize = cb.u.LowPart;
        }

        hr = IStream_Read(
            iface,
            tmpBuffer.as_mut_ptr() as *mut c_void,
            copySize,
            core::ptr::addr_of_mut!(bytesRead),
        );
        if FAILED(hr) {
            break;
        }

        totalBytesRead.QuadPart += bytesRead as ULONGLONG;

        if bytesRead != 0 {
            hr = IStream_Write(
                pstm,
                tmpBuffer.as_ptr() as *const c_void,
                bytesRead,
                core::ptr::addr_of_mut!(bytesWritten),
            );
            if FAILED(hr) {
                break;
            }

            totalBytesWritten.QuadPart += bytesWritten as ULONGLONG;
        }

        if bytesRead != copySize {
            cb.QuadPart = 0;
        } else {
            cb.QuadPart -= bytesRead as ULONGLONG;
        }
    }

    if !pcbRead.is_null() {
        (*pcbRead).QuadPart = totalBytesRead.QuadPart;
    }
    if !pcbWritten.is_null() {
        (*pcbWritten).QuadPart = totalBytesWritten.QuadPart;
    }

    hr
}

/// IStream::Commit — for HGLOBAL-backed streams this does nothing.
unsafe fn HGLOBALStreamImpl_Commit(_iface: *mut IStream, _grfCommitFlags: DWORD) -> HRESULT {
    S_OK
}

/// IStream::Revert — for HGLOBAL-backed streams this does nothing.
unsafe fn HGLOBALStreamImpl_Revert(_iface: *mut IStream) -> HRESULT {
    S_OK
}

/// IStream::LockRegion — unsupported for HGLOBAL-backed streams.
unsafe fn HGLOBALStreamImpl_LockRegion(
    _iface: *mut IStream,
    _libOffset: ULARGE_INTEGER,
    _cb: ULARGE_INTEGER,
    _dwLockType: DWORD,
) -> HRESULT {
    STG_E_INVALIDFUNCTION
}

/// IStream::UnlockRegion — for HGLOBAL-backed streams this does nothing.
unsafe fn HGLOBALStreamImpl_UnlockRegion(
    _iface: *mut IStream,
    _libOffset: ULARGE_INTEGER,
    _cb: ULARGE_INTEGER,
    _dwLockType: DWORD,
) -> HRESULT {
    S_OK
}

/// IStream::Stat — returns information about the current stream.
unsafe fn HGLOBALStreamImpl_Stat(
    iface: *mut IStream,
    pstatstg: *mut STATSTG,
    _grfStatFlag: DWORD,
) -> HRESULT {
    let This = impl_from_IStream(iface);

    // memset(pstatstg, 0, sizeof(STATSTG));
    core::ptr::write_bytes(pstatstg as *mut u8, 0, core::mem::size_of::<STATSTG>());

    (*pstatstg).pwcsName = core::ptr::null_mut();
    (*pstatstg).type_ = STGTY_STREAM;
    (*pstatstg).cbSize.QuadPart = handle_getsize((*This).handle) as ULONGLONG;

    S_OK
}

unsafe fn HGLOBALStreamImpl_Clone(iface: *mut IStream, ppstm: *mut *mut IStream) -> HRESULT {
    let This = impl_from_IStream(iface);
    let mut dummy: ULARGE_INTEGER = core::mem::zeroed();
    let mut offset: LARGE_INTEGER = core::mem::zeroed();

    if ppstm.is_null() {
        return E_INVALIDARG;
    }

    *ppstm = core::ptr::null_mut();

    TRACE!(
        " Cloning %p (seek position=%d)\n",
        iface,
        (*This).currentPosition.u.LowPart
    );

    let clone: *mut HGLOBALStreamImpl = HGLOBALStreamImpl_Create();
    if clone.is_null() {
        return E_OUTOFMEMORY;
    }

    *ppstm = core::ptr::addr_of_mut!((*clone).IStream_iface);

    handle_addref((*This).handle);
    (*clone).handle = (*This).handle;

    offset.QuadPart = (*This).currentPosition.QuadPart as LONGLONG;
    IStream_Seek(
        *ppstm,
        offset,
        STREAM_SEEK_SET,
        core::ptr::addr_of_mut!(dummy),
    );
    S_OK
}

} // stdmethod!

pub static HGLOBALStreamImplVtbl: IStreamVtbl = IStreamVtbl {
    QueryInterface: Some(HGLOBALStreamImpl_QueryInterface),
    AddRef: Some(HGLOBALStreamImpl_AddRef),
    Release: Some(HGLOBALStreamImpl_Release),
    Read: Some(HGLOBALStreamImpl_Read),
    Write: Some(HGLOBALStreamImpl_Write),
    Seek: Some(HGLOBALStreamImpl_Seek),
    SetSize: Some(HGLOBALStreamImpl_SetSize),
    CopyTo: Some(HGLOBALStreamImpl_CopyTo),
    Commit: Some(HGLOBALStreamImpl_Commit),
    Revert: Some(HGLOBALStreamImpl_Revert),
    LockRegion: Some(HGLOBALStreamImpl_LockRegion),
    UnlockRegion: Some(HGLOBALStreamImpl_UnlockRegion),
    Stat: Some(HGLOBALStreamImpl_Stat),
    Clone: Some(HGLOBALStreamImpl_Clone),
};

/***********************************************************************
 *           CreateStreamOnHGlobal     [OLE32.@]
 */
#[no_mangle]
pub unsafe extern "system" fn CreateStreamOnHGlobal(
    hGlobal: HGLOBAL,
    fDeleteOnRelease: BOOL,
    ppstm: *mut LPSTREAM,
) -> HRESULT {
    if ppstm.is_null() {
        return E_INVALIDARG;
    }

    let This: *mut HGLOBALStreamImpl = HGLOBALStreamImpl_Create();
    if This.is_null() {
        return E_OUTOFMEMORY;
    }

    /* allocate a handle if one is not supplied */
    let mut hGlobal = hGlobal;
    if hGlobal.is_null() {
        hGlobal = GlobalAlloc(GMEM_MOVEABLE | GMEM_NODISCARD | GMEM_SHARE, 0);
    }

    (*This).handle = handle_create(hGlobal, fDeleteOnRelease);

    /* start at the beginning */
    (*This).currentPosition.u.HighPart = 0;
    (*This).currentPosition.u.LowPart = 0;

    *ppstm = core::ptr::addr_of_mut!((*This).IStream_iface);

    S_OK
}

/***********************************************************************
 *           GetHGlobalFromStream     [OLE32.@]
 */
#[no_mangle]
pub unsafe extern "system" fn GetHGlobalFromStream(
    pstm: *mut IStream,
    phglobal: *mut HGLOBAL,
) -> HRESULT {
    if pstm.is_null() || phglobal.is_null() {
        return E_INVALIDARG;
    }

    let pStream: *mut HGLOBALStreamImpl = impl_from_IStream(pstm);

    /*
     * Verify that the stream object was created with CreateStreamOnHGlobal.
     */
    if (*pStream).IStream_iface.lpVtbl
        == &HGLOBALStreamImplVtbl as *const IStreamVtbl as *mut IStreamVtbl
    {
        *phglobal = handle_gethglobal((*pStream).handle);
    } else {
        *phglobal = core::ptr::null_mut();
        return E_INVALIDARG;
    }

    S_OK
}
