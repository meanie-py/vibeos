//! `dll/win32/ole32/stg_stream.c` — the IStream implementation for streams contained
//! in a compound storage (`StgStreamImpl`).
//!
//! The COM method slots are emitted through the crate `stdmethod!` macro: com-sys spells
//! the IStreamVtbl slots `unsafe extern "stdcall" fn` on i386 and `unsafe extern "C" fn`
//! on amd64/arm64 (i.e. exactly what STDMETHODCALLTYPE lowers to per arch). `extern
//! "system"` lowers to the same ABI but stays a DISTINCT fn type that will not unify with
//! the bindgen slot type, so the impls must spell the ABI the way com-sys does.

use crate::prelude::*;
use core::ffi::c_void;

// ---------------------------------------------------------------------------
// IUnknown
// ---------------------------------------------------------------------------

stdmethod! {
fn StgStreamImpl_QueryInterface(
    iface: *mut IStream,
    riid: *const IID,
    ppvObject: *mut *mut c_void,
) -> HRESULT {
    let This = impl_from_IStream(iface);

    if ppvObject.is_null() {
        return E_INVALIDARG;
    }

    *ppvObject = core::ptr::null_mut();

    if IsEqualIID(addr_of!(IID_IUnknown), riid)
        || IsEqualIID(addr_of!(IID_ISequentialStream), riid)
        || IsEqualIID(addr_of!(IID_IStream), riid)
    {
        *ppvObject = addr_of_mut!((*This).IStream_iface) as *mut c_void;
    } else {
        return E_NOINTERFACE;
    }

    IStream_AddRef(iface);

    S_OK
}
}

stdmethod! {
fn StgStreamImpl_AddRef(iface: *mut IStream) -> ULONG {
    let This = impl_from_IStream(iface);
    InterlockedIncrement(addr_of_mut!((*This).ref_)) as ULONG
}
}

stdmethod! {
fn StgStreamImpl_Release(iface: *mut IStream) -> ULONG {
    let This = impl_from_IStream(iface);
    let ref_ = InterlockedDecrement(addr_of_mut!((*This).ref_)) as ULONG;

    if ref_ == 0 {
        TRACE!("(%p)\n", This);

        /*
         * Release the reference we are holding on the parent storage.
         *   IStorage_Release(&This->parentStorage->IStorage_iface);
         *
         * No, don't do this. Some apps call IStorage_Release without calling
         * IStream_Release first. If we grab a reference the file is not closed, and the
         * app fails when it tries to reopen the file (Easy-PC, for example). Just inform
         * the storage that we have closed the stream.
         */
        if !(*This).parentStorage.is_null() {
            StorageBaseImpl_RemoveStream((*This).parentStorage, This);
        }
        (*This).parentStorage = core::ptr::null_mut();
        HeapFree(GetProcessHeap(), 0, This as LPVOID);
    }

    ref_
}
}

// ---------------------------------------------------------------------------
// ISequentialStream
// ---------------------------------------------------------------------------

stdmethod! {
/// Reads a block of information from the stream at the current position, then moves the
/// current position to the end of the read block.
fn StgStreamImpl_Read(
    iface: *mut IStream,
    pv: *mut c_void,
    cb: ULONG,
    pcbRead: *mut ULONG,
) -> HRESULT {
    let This = impl_from_IStream(iface);

    let mut bytesReadBuffer: ULONG = 0;

    TRACE!("(%p, %p, %d, %p)\n", iface, pv, cb, pcbRead);

    if (*This).parentStorage.is_null() {
        WARN!("storage reverted\n");
        return STG_E_REVERTED;
    }

    /*
     * If the caller is not interested in the number of bytes read, we use another buffer
     * to avoid "if" statements in the code.
     */
    let pcbRead = if pcbRead.is_null() {
        addr_of_mut!(bytesReadBuffer)
    } else {
        pcbRead
    };

    let res = StorageBaseImpl_StreamReadAt(
        (*This).parentStorage,
        (*This).dirEntry,
        (*This).currentPosition,
        cb,
        pv,
        pcbRead,
    );

    if SUCCEEDED(res) {
        /* Advance the pointer for the number of positions read. */
        (*This).currentPosition.QuadPart =
            (*This).currentPosition.QuadPart.wrapping_add(*pcbRead as ULONGLONG);
    }

    TRACE!("<-- %08x\n", res);
    res
}
}

stdmethod! {
/// Writes a block of information to the stream at the current position, growing the
/// stream if it is too small to fit the block.
fn StgStreamImpl_Write(
    iface: *mut IStream,
    pv: *const c_void,
    cb: ULONG,
    pcbWritten: *mut ULONG,
) -> HRESULT {
    let This = impl_from_IStream(iface);

    let mut bytesWritten: ULONG = 0;

    TRACE!("(%p, %p, %d, %p)\n", iface, pv, cb, pcbWritten);

    /* Do we have permission to write to this stream? */
    match STGM_ACCESS_MODE((*This).grfMode) {
        STGM_WRITE | STGM_READWRITE => {}
        _ => {
            WARN!(
                "access denied by flags: 0x%x\n",
                STGM_ACCESS_MODE((*This).grfMode)
            );
            return STG_E_ACCESSDENIED;
        }
    }

    if pv.is_null() {
        return STG_E_INVALIDPOINTER;
    }

    if (*This).parentStorage.is_null() {
        WARN!("storage reverted\n");
        return STG_E_REVERTED;
    }

    /*
     * If the caller is not interested in the number of bytes written, we use another
     * buffer to avoid "if" statements in the code.
     */
    let pcbWritten = if pcbWritten.is_null() {
        addr_of_mut!(bytesWritten)
    } else {
        pcbWritten
    };

    /* Initialize the out parameter */
    *pcbWritten = 0;

    if cb == 0 {
        TRACE!("<-- S_OK, written 0\n");
        return S_OK;
    }

    let mut res = StorageBaseImpl_StreamWriteAt(
        (*This).parentStorage,
        (*This).dirEntry,
        (*This).currentPosition,
        cb,
        pv,
        pcbWritten,
    );

    /* Advance the position pointer for the number of positions written. */
    (*This).currentPosition.QuadPart =
        (*This).currentPosition.QuadPart.wrapping_add(*pcbWritten as ULONGLONG);

    if SUCCEEDED(res) {
        res = StorageBaseImpl_Flush((*This).parentStorage);
    }

    TRACE!("<-- %08x, written %u\n", res, *pcbWritten);
    res
}
}

// ---------------------------------------------------------------------------
// IStream
// ---------------------------------------------------------------------------

stdmethod! {
/// Moves the current stream pointer according to the parameters given.
fn StgStreamImpl_Seek(
    iface: *mut IStream,
    dlibMove: LARGE_INTEGER,
    dwOrigin: DWORD,
    plibNewPosition: *mut ULARGE_INTEGER,
) -> HRESULT {
    let This = impl_from_IStream(iface);

    let mut newPosition: ULARGE_INTEGER = core::mem::zeroed();
    let mut currentEntry: DirEntry = core::mem::zeroed();

    TRACE!("(%p, %d, %d, %p)\n", iface, dlibMove.u.LowPart, dwOrigin, plibNewPosition);

    /* fail if the stream has no parent (as does windows) */
    if (*This).parentStorage.is_null() {
        WARN!("storage reverted\n");
        return STG_E_REVERTED;
    }

    /*
     * The caller is allowed to pass in NULL as the new position return value. If it
     * happens, we assign it to a dynamic variable to avoid special cases in the code
     * below.
     */
    let plibNewPosition = if plibNewPosition.is_null() {
        addr_of_mut!(newPosition)
    } else {
        plibNewPosition
    };

    /* The file pointer is moved depending on the given "function" parameter. */
    match dwOrigin {
        STREAM_SEEK_SET => {
            (*plibNewPosition).u.HighPart = 0;
            (*plibNewPosition).u.LowPart = 0;
        }
        STREAM_SEEK_CUR => {
            *plibNewPosition = (*This).currentPosition;
        }
        STREAM_SEEK_END => {
            let hr = StorageBaseImpl_ReadDirEntry(
                (*This).parentStorage,
                (*This).dirEntry,
                addr_of_mut!(currentEntry),
            );
            if FAILED(hr) {
                return hr;
            }
            *plibNewPosition = currentEntry.size;
        }
        _ => {
            WARN!("invalid dwOrigin %d\n", dwOrigin);
            return STG_E_INVALIDFUNCTION;
        }
    }

    // C: `plibNewPosition->QuadPart += dlibMove.QuadPart;` — an unsigned 64-bit add of a
    // SIGNED 64-bit delta, i.e. the negative case wraps around. `as ULONGLONG` is the
    // two's-complement reinterpretation and `wrapping_add` the modular add, so this is
    // bit-exact with the C on both signs.
    (*plibNewPosition).QuadPart =
        (*plibNewPosition).QuadPart.wrapping_add(dlibMove.QuadPart as ULONGLONG);

    /* tell the caller what we calculated */
    (*This).currentPosition = *plibNewPosition;

    S_OK
}
}

stdmethod! {
/// Changes the size of a stream.
fn StgStreamImpl_SetSize(iface: *mut IStream, libNewSize: ULARGE_INTEGER) -> HRESULT {
    let This = impl_from_IStream(iface);

    TRACE!("(%p, %d)\n", iface, libNewSize.u.LowPart);

    if (*This).parentStorage.is_null() {
        WARN!("storage reverted\n");
        return STG_E_REVERTED;
    }

    /* As documented. */
    if libNewSize.u.HighPart != 0 {
        WARN!(
            "invalid value for libNewSize.u.HighPart %d\n",
            libNewSize.u.HighPart
        );
        return STG_E_INVALIDFUNCTION;
    }

    /* Do we have permission? */
    if ((*This).grfMode & (STGM_WRITE | STGM_READWRITE)) == 0 {
        WARN!("access denied\n");
        return STG_E_ACCESSDENIED;
    }

    let mut hr = StorageBaseImpl_StreamSetSize((*This).parentStorage, (*This).dirEntry, libNewSize);

    if SUCCEEDED(hr) {
        hr = StorageBaseImpl_Flush((*This).parentStorage);
    }

    hr
}
}

stdmethod! {
/// Copies `cb` bytes to the `pstm` IStream.
fn StgStreamImpl_CopyTo(
    iface: *mut IStream,
    pstm: *mut IStream,
    cb: ULARGE_INTEGER,
    pcbRead: *mut ULARGE_INTEGER,
    pcbWritten: *mut ULARGE_INTEGER,
) -> HRESULT {
    let This = impl_from_IStream(iface);
    let mut hr: HRESULT = S_OK;
    let mut tmpBuffer: [BYTE; 128] = [0; 128];
    let mut bytesRead: ULONG = 0;
    let mut bytesWritten: ULONG = 0;
    let mut copySize: ULONG;
    let mut totalBytesRead: ULARGE_INTEGER = core::mem::zeroed();
    let mut totalBytesWritten: ULARGE_INTEGER = core::mem::zeroed();

    // The C's `cb` is a by-value param it decrements in the loop; Rust params are not
    // `mut`, so shadow it with a local copy (same semantics — the caller's copy is
    // untouched either way).
    let mut cb = cb;

    TRACE!("(%p, %p, %d, %p, %p)\n", iface, pstm, cb.u.LowPart, pcbRead, pcbWritten);

    /* Sanity check */
    if (*This).parentStorage.is_null() {
        WARN!("storage reverted\n");
        return STG_E_REVERTED;
    }

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

        IStream_Read(
            iface,
            tmpBuffer.as_mut_ptr() as *mut c_void,
            copySize,
            addr_of_mut!(bytesRead),
        );

        totalBytesRead.QuadPart =
            totalBytesRead.QuadPart.wrapping_add(bytesRead as ULONGLONG);

        IStream_Write(
            pstm,
            tmpBuffer.as_ptr() as *const c_void,
            bytesRead,
            addr_of_mut!(bytesWritten),
        );

        totalBytesWritten.QuadPart =
            totalBytesWritten.QuadPart.wrapping_add(bytesWritten as ULONGLONG);

        /* Check that read & write operations were successful */
        if bytesRead != bytesWritten {
            hr = STG_E_MEDIUMFULL;
            WARN!("medium full\n");
            break;
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
}

stdmethod! {
/// For streams contained in structured storages this does nothing but flush the parent.
fn StgStreamImpl_Commit(iface: *mut IStream, grfCommitFlags: DWORD) -> HRESULT {
    let _ = grfCommitFlags;
    let This = impl_from_IStream(iface);

    if (*This).parentStorage.is_null() {
        WARN!("storage reverted\n");
        return STG_E_REVERTED;
    }

    StorageBaseImpl_Flush((*This).parentStorage)
}
}

stdmethod! {
/// For streams contained in structured storages this does nothing.
fn StgStreamImpl_Revert(iface: *mut IStream) -> HRESULT {
    let _ = iface;
    S_OK
}
}

stdmethod! {
fn StgStreamImpl_LockRegion(
    iface: *mut IStream,
    libOffset: ULARGE_INTEGER,
    cb: ULARGE_INTEGER,
    dwLockType: DWORD,
) -> HRESULT {
    let _ = (libOffset, cb, dwLockType);
    let This = impl_from_IStream(iface);

    if (*This).parentStorage.is_null() {
        WARN!("storage reverted\n");
        return STG_E_REVERTED;
    }

    FIXME!("not implemented!\n");
    E_NOTIMPL
}
}

stdmethod! {
fn StgStreamImpl_UnlockRegion(
    iface: *mut IStream,
    libOffset: ULARGE_INTEGER,
    cb: ULARGE_INTEGER,
    dwLockType: DWORD,
) -> HRESULT {
    let _ = (libOffset, cb, dwLockType);
    let This = impl_from_IStream(iface);

    if (*This).parentStorage.is_null() {
        WARN!("storage reverted\n");
        return STG_E_REVERTED;
    }

    FIXME!("not implemented!\n");
    E_NOTIMPL
}
}

stdmethod! {
/// Returns information about the current stream.
fn StgStreamImpl_Stat(
    iface: *mut IStream,
    pstatstg: *mut STATSTG,
    grfStatFlag: DWORD,
) -> HRESULT {
    let This = impl_from_IStream(iface);

    let mut currentEntry: DirEntry = core::mem::zeroed();

    TRACE!("%p %p %d\n", This, pstatstg, grfStatFlag);

    /* if stream has no parent, return STG_E_REVERTED */
    if (*This).parentStorage.is_null() {
        WARN!("storage reverted\n");
        return STG_E_REVERTED;
    }

    /* Read the information from the directory entry. */
    let hr = StorageBaseImpl_ReadDirEntry(
        (*This).parentStorage,
        (*This).dirEntry,
        addr_of_mut!(currentEntry),
    );

    if SUCCEEDED(hr) {
        StorageUtl_CopyDirEntryToSTATSTG(
            (*This).parentStorage,
            pstatstg,
            addr_of!(currentEntry),
            grfStatFlag as c_int,
        );

        (*pstatstg).grfMode = (*This).grfMode;

        /* In simple create mode cbSize is the current pos */
        if ((*(*This).parentStorage).openFlags & STGM_SIMPLE) != 0
            && (*(*This).parentStorage).create != 0
        {
            (*pstatstg).cbSize = (*This).currentPosition;
        }

        return S_OK;
    }

    WARN!("failed to read entry\n");
    hr
}
}

stdmethod! {
/// Returns a clone of the interface that allows for another seek pointer.
fn StgStreamImpl_Clone(iface: *mut IStream, ppstm: *mut *mut IStream) -> HRESULT {
    let This = impl_from_IStream(iface);
    let mut seek_pos: LARGE_INTEGER = core::mem::zeroed();

    TRACE!("%p %p\n", This, ppstm);

    /* Sanity check */
    if (*This).parentStorage.is_null() {
        return STG_E_REVERTED;
    }

    if ppstm.is_null() {
        return STG_E_INVALIDPOINTER;
    }

    let new_stream =
        StgStreamImpl_Construct((*This).parentStorage, (*This).grfMode, (*This).dirEntry);

    if new_stream.is_null() {
        /* Currently the only reason for new_stream=0 */
        return STG_E_INSUFFICIENTMEMORY;
    }

    *ppstm = addr_of_mut!((*new_stream).IStream_iface);
    IStream_AddRef(*ppstm);

    seek_pos.QuadPart = (*This).currentPosition.QuadPart as LONGLONG;

    IStream_Seek(*ppstm, seek_pos, STREAM_SEEK_SET, core::ptr::null_mut())
}
}

/// Virtual function table for the StgStreamImpl class.
///
/// Slot-for-slot with the C's `static const IStreamVtbl StgStreamVtbl` (stg_stream.c:637):
/// QueryInterface, AddRef, Release, Read, Write, Seek, SetSize, CopyTo, Commit, Revert,
/// LockRegion, UnlockRegion, Stat, Clone.
pub static StgStreamVtbl: IStreamVtbl = IStreamVtbl {
    QueryInterface: Some(StgStreamImpl_QueryInterface),
    AddRef: Some(StgStreamImpl_AddRef),
    Release: Some(StgStreamImpl_Release),
    Read: Some(StgStreamImpl_Read),
    Write: Some(StgStreamImpl_Write),
    Seek: Some(StgStreamImpl_Seek),
    SetSize: Some(StgStreamImpl_SetSize),
    CopyTo: Some(StgStreamImpl_CopyTo),
    Commit: Some(StgStreamImpl_Commit),
    Revert: Some(StgStreamImpl_Revert),
    LockRegion: Some(StgStreamImpl_LockRegion),
    UnlockRegion: Some(StgStreamImpl_UnlockRegion),
    Stat: Some(StgStreamImpl_Stat),
    Clone: Some(StgStreamImpl_Clone),
};

// ---------------------------------------------------------------------------
// StgStreamImpl implementation
// ---------------------------------------------------------------------------

/// Constructor for the StgStreamImpl class.
///
/// * `parentStorage` — the storage that contains the stream to open
/// * `dirEntry`      — index of the directory entry that points to this stream
#[no_mangle]
pub unsafe extern "C" fn StgStreamImpl_Construct(
    parentStorage: *mut StorageBaseImpl,
    grfMode: DWORD,
    dirEntry: DirRef,
) -> *mut StgStreamImpl {
    let newStream = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<StgStreamImpl>() as usize,
    ) as *mut StgStreamImpl;

    if !newStream.is_null() {
        /* Set-up the virtual function table and reference count. */
        (*newStream).IStream_iface.lpVtbl =
            addr_of!(StgStreamVtbl) as *const IStreamVtbl as *mut IStreamVtbl;
        (*newStream).ref_ = 0;

        (*newStream).parentStorage = parentStorage;

        /*
         * We want to nail-down the reference to the storage in case the stream out-lives
         * the storage in the client application.
         *
         *   IStorage_AddRef(&newStream->parentStorage->IStorage_iface);
         *
         * No, don't do this. Some apps call IStorage_Release without calling
         * IStream_Release first. If we grab a reference the file is not closed, and the
         * app fails when it tries to reopen the file (Easy-PC, for example).
         */

        (*newStream).grfMode = grfMode;
        (*newStream).dirEntry = dirEntry;

        /* Start the stream at the beginning. */
        (*newStream).currentPosition.u.HighPart = 0;
        (*newStream).currentPosition.u.LowPart = 0;

        /* add us to the storage's list of active streams */
        StorageBaseImpl_AddStream(parentStorage, newStream);
    }

    newStream
}
