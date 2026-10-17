//! memlockbytes.c — global-memory (HGLOBAL) implementation of ILockBytes.
use crate::prelude::*;

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct HGLOBALLockBytesImpl {
    pub ILockBytes_iface: ILockBytes,
    pub ref_: LONG,
    pub supportHandle: HGLOBAL,
    pub deleteOnRelease: BOOL,
    pub byteArraySize: ULARGE_INTEGER,
}

pub type PHGLOBALLockBytesImpl = *mut HGLOBALLockBytesImpl;

pub static HGLOBALLockBytesImpl_Vtbl: ILockBytesVtbl = ILockBytesVtbl {
    QueryInterface: Some(HGLOBALLockBytesImpl_QueryInterface),
    AddRef: Some(HGLOBALLockBytesImpl_AddRef),
    Release: Some(HGLOBALLockBytesImpl_Release),
    ReadAt: Some(HGLOBALLockBytesImpl_ReadAt),
    WriteAt: Some(HGLOBALLockBytesImpl_WriteAt),
    Flush: Some(HGLOBALLockBytesImpl_Flush),
    SetSize: Some(HGLOBALLockBytesImpl_SetSize),
    LockRegion: Some(HGLOBALLockBytesImpl_LockRegion),
    UnlockRegion: Some(HGLOBALLockBytesImpl_UnlockRegion),
    Stat: Some(HGLOBALLockBytesImpl_Stat),
};

#[inline]
fn hglobal_lockbytes_vtbl_ptr() -> *mut ILockBytesVtbl {
    &HGLOBALLockBytesImpl_Vtbl as *const ILockBytesVtbl as *mut ILockBytesVtbl
}

unsafe fn impl_from_ILockBytes(iface: *mut ILockBytes) -> *mut HGLOBALLockBytesImpl {
    CONTAINING_RECORD!(iface, HGLOBALLockBytesImpl, ILockBytes_iface)
}

/// memlockbytes.c:98 CreateILockBytesOnHGlobal [OLE32.@]
#[no_mangle]
pub unsafe extern "system" fn CreateILockBytesOnHGlobal(
    global: HGLOBAL,
    delete_on_release: BOOL,
    ret: *mut *mut ILockBytes,
) -> HRESULT {
    let lockbytes = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<HGLOBALLockBytesImpl>() as usize,
    ) as *mut HGLOBALLockBytesImpl;

    if lockbytes.is_null() {
        return E_OUTOFMEMORY;
    }

    (*lockbytes).ILockBytes_iface.lpVtbl = hglobal_lockbytes_vtbl_ptr();
    (*lockbytes).ref_ = 1;

    /*
     * Initialize the support.
     */
    (*lockbytes).supportHandle = global;
    (*lockbytes).deleteOnRelease = delete_on_release;

    /*
     * This method will allocate a handle if one is not supplied.
     */
    if (*lockbytes).supportHandle.is_null() {
        (*lockbytes).supportHandle = GlobalAlloc(GMEM_MOVEABLE | GMEM_NODISCARD, 0);
    }

    /*
     * Initialize the size of the array to the size of the handle.
     */
    (*lockbytes).byteArraySize.u.HighPart = 0;
    (*lockbytes).byteArraySize.u.LowPart = GlobalSize((*lockbytes).supportHandle) as DWORD;

    *ret = core::ptr::addr_of_mut!((*lockbytes).ILockBytes_iface);

    S_OK
}

/// memlockbytes.c:145 GetHGlobalFromILockBytes [OLE32.@]
#[no_mangle]
pub unsafe extern "system" fn GetHGlobalFromILockBytes(
    iface: *mut ILockBytes,
    phglobal: *mut HGLOBAL,
) -> HRESULT {
    let This = impl_from_ILockBytes(iface);
    let mut stbuf: STATSTG = core::mem::zeroed();
    let hres: HRESULT;
    let mut start: ULARGE_INTEGER = core::mem::zeroed();
    let mut xread: ULONG = 0;

    *phglobal = core::ptr::null_mut();
    if (*This).ILockBytes_iface.lpVtbl == hglobal_lockbytes_vtbl_ptr() {
        *phglobal = (*This).supportHandle;
        if (*phglobal).is_null() {
            return E_INVALIDARG;
        }
        return S_OK;
    }

    /* It is not our lockbytes implementation, so use a more generic way */
    hres = ILockBytes_Stat(iface, &mut stbuf, STATFLAG_NONAME);
    if hres != S_OK {
        ERR!("Cannot ILockBytes_Stat, %x\n", hres);
        return hres;
    }
    TRACE!("cbSize is %s\n", stbuf.cbSize.QuadPart);
    *phglobal = GlobalAlloc(
        GMEM_MOVEABLE | GMEM_SHARE,
        stbuf.cbSize.u.LowPart as SIZE_T,
    );
    if (*phglobal).is_null() {
        return E_INVALIDARG;
    }
    memset(
        core::ptr::addr_of_mut!(start) as *mut c_void,
        0,
        core::mem::size_of::<ULARGE_INTEGER>(),
    );
    let hres = ILockBytes_ReadAt(
        iface,
        start,
        GlobalLock(*phglobal),
        stbuf.cbSize.u.LowPart,
        &mut xread,
    );
    GlobalUnlock(*phglobal);
    if hres != S_OK {
        FIXME!("%p->ReadAt failed with %x\n", iface, hres);
        return hres;
    }
    if stbuf.cbSize.u.LowPart != xread {
        FIXME!(
            "Read size is not requested size %d vs %d?\n",
            stbuf.cbSize.u.LowPart,
            xread
        );
    }
    S_OK
}

stdmethod! {
/// memlockbytes.c:193 HGLOBALLockBytesImpl_QueryInterface [VTABLE]
fn HGLOBALLockBytesImpl_QueryInterface(
    iface: *mut ILockBytes,
    riid: *const IID,
    ppvObject: *mut *mut c_void,
) -> HRESULT {
    let This = impl_from_ILockBytes(iface);

    if ppvObject.is_null() {
        return E_INVALIDARG;
    }

    *ppvObject = core::ptr::null_mut();

    if IsEqualIID(riid, core::ptr::addr_of!(IID_IUnknown))
        || IsEqualIID(riid, core::ptr::addr_of!(IID_ILockBytes))
    {
        *ppvObject = core::ptr::addr_of_mut!((*This).ILockBytes_iface) as *mut c_void;
    } else {
        return E_NOINTERFACE;
    }

    ILockBytes_AddRef(iface);

    S_OK
}
}

stdmethod! {
/// memlockbytes.c:222 HGLOBALLockBytesImpl_AddRef [VTABLE]
fn HGLOBALLockBytesImpl_AddRef(iface: *mut ILockBytes) -> ULONG {
    let This = impl_from_ILockBytes(iface);
    InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref_)) as ULONG
}
}

stdmethod! {
/// memlockbytes.c:232 HGLOBALLockBytesImpl_Release [VTABLE]
fn HGLOBALLockBytesImpl_Release(iface: *mut ILockBytes) -> ULONG {
    let This = impl_from_ILockBytes(iface);

    let ref_count = InterlockedDecrement(core::ptr::addr_of_mut!((*This).ref_));
    if ref_count == 0 {
        if (*This).deleteOnRelease != 0 {
            GlobalFree((*This).supportHandle);
            (*This).supportHandle = core::ptr::null_mut();
        }
        HeapFree(GetProcessHeap(), 0, This as LPVOID);
    }

    ref_count as ULONG
}
}

stdmethod! {
/// memlockbytes.c:259 HGLOBALLockBytesImpl_ReadAt [VTABLE]
fn HGLOBALLockBytesImpl_ReadAt(
    iface: *mut ILockBytes,
    ulOffset: ULARGE_INTEGER,
    pv: *mut c_void,
    cb: ULONG,
    pcbRead: *mut ULONG,
) -> HRESULT {
    let This = impl_from_ILockBytes(iface);

    let mut bytesReadBuffer: ULONG = 0;

    /*
     * If the caller is not interested in the number of bytes read,
     * we use another buffer to avoid "if" statements in the code.
     */
    let pcbRead = if pcbRead.is_null() {
        core::ptr::addr_of_mut!(bytesReadBuffer)
    } else {
        pcbRead
    };

    /*
     * Make sure the offset is valid.
     */
    if ulOffset.u.LowPart > (*This).byteArraySize.u.LowPart {
        return E_FAIL;
    }

    /*
     * Using the known size of the array, calculate the number of bytes
     * to read.
     */
    let bytesToReadFromBuffer = core::cmp::min(
        (*This).byteArraySize.u.LowPart - ulOffset.u.LowPart,
        cb,
    );

    /*
     * Lock the buffer in position and copy the data.
     */
    let supportBuffer = GlobalLock((*This).supportHandle);

    memcpy(
        pv,
        (supportBuffer as *const u8).add(ulOffset.u.LowPart as usize) as *const c_void,
        bytesToReadFromBuffer as usize,
    );

    /*
     * Return the number of bytes read.
     */
    *pcbRead = bytesToReadFromBuffer;

    /*
     * Cleanup
     */
    GlobalUnlock((*This).supportHandle);

    if *pcbRead == cb {
        return S_OK;
    }

    STG_E_READFAULT
}
}

stdmethod! {
/// memlockbytes.c:331 HGLOBALLockBytesImpl_WriteAt [VTABLE]
fn HGLOBALLockBytesImpl_WriteAt(
    iface: *mut ILockBytes,
    ulOffset: ULARGE_INTEGER,
    pv: *const c_void,
    cb: ULONG,
    pcbWritten: *mut ULONG,
) -> HRESULT {
    let This = impl_from_ILockBytes(iface);

    let mut newSize: ULARGE_INTEGER = core::mem::zeroed();
    let mut bytesWritten: ULONG = 0;

    /*
     * If the caller is not interested in the number of bytes written,
     * we use another buffer to avoid "if" statements in the code.
     */
    let pcbWritten = if pcbWritten.is_null() {
        core::ptr::addr_of_mut!(bytesWritten)
    } else {
        pcbWritten
    };

    if cb == 0 {
        return S_OK;
    } else {
        newSize.u.HighPart = 0;
        // C computes this in 32-bit ULONG arithmetic (wraps mod 2^32).
        newSize.u.LowPart = ulOffset.u.LowPart.wrapping_add(cb);
    }

    /*
     * Verify if we need to grow the stream
     */
    if newSize.u.LowPart > (*This).byteArraySize.u.LowPart {
        /* grow stream */
        if ILockBytes_SetSize(iface, newSize) == STG_E_MEDIUMFULL {
            return STG_E_MEDIUMFULL;
        }
    }

    /*
     * Lock the buffer in position and copy the data.
     */
    let supportBuffer = GlobalLock((*This).supportHandle);

    memcpy(
        (supportBuffer as *mut u8).add(ulOffset.u.LowPart as usize) as *mut c_void,
        pv,
        cb as usize,
    );

    /*
     * Return the number of bytes written.
     */
    *pcbWritten = cb;

    /*
     * Cleanup
     */
    GlobalUnlock((*This).supportHandle);

    S_OK
}
}

stdmethod! {
/// memlockbytes.c:396 HGLOBALLockBytesImpl_Flush [VTABLE]
fn HGLOBALLockBytesImpl_Flush(iface: *mut ILockBytes) -> HRESULT {
    let _ = iface;
    S_OK
}
}

stdmethod! {
/// memlockbytes.c:408 HGLOBALLockBytesImpl_SetSize [VTABLE]
fn HGLOBALLockBytesImpl_SetSize(
    iface: *mut ILockBytes,
    libNewSize: ULARGE_INTEGER,
) -> HRESULT {
    let This = impl_from_ILockBytes(iface);

    /*
     * As documented.
     */
    if libNewSize.u.HighPart != 0 {
        return STG_E_INVALIDFUNCTION;
    }

    if (*This).byteArraySize.u.LowPart == libNewSize.u.LowPart {
        return S_OK;
    }

    /*
     * Re allocate the HGlobal to fit the new size of the stream.
     */
    // wine-10 (coml2) passes GMEM_MOVEABLE. With 0, GlobalReAlloc refuses size 0 for a
    // live block, so StgCreateDocfileOnILockBytes never truncated the array and the
    // header write kept the caller's old bytes: an INVALIDHEADER docfile.
    let supportHandle = GlobalReAlloc(
        (*This).supportHandle,
        libNewSize.u.LowPart as SIZE_T,
        GMEM_MOVEABLE,
    );

    if supportHandle.is_null() {
        return STG_E_MEDIUMFULL;
    }

    (*This).supportHandle = supportHandle;
    (*This).byteArraySize.u.LowPart = libNewSize.u.LowPart;

    S_OK
}
}

stdmethod! {
/// memlockbytes.c:445 HGLOBALLockBytesImpl_LockRegion [VTABLE]
fn HGLOBALLockBytesImpl_LockRegion(
    iface: *mut ILockBytes,
    libOffset: ULARGE_INTEGER,
    cb: ULARGE_INTEGER,
    dwLockType: DWORD,
) -> HRESULT {
    let _ = (iface, libOffset, cb, dwLockType);
    STG_E_INVALIDFUNCTION
}
}

stdmethod! {
/// memlockbytes.c:461 HGLOBALLockBytesImpl_UnlockRegion [VTABLE]
fn HGLOBALLockBytesImpl_UnlockRegion(
    iface: *mut ILockBytes,
    libOffset: ULARGE_INTEGER,
    cb: ULARGE_INTEGER,
    dwLockType: DWORD,
) -> HRESULT {
    let _ = (iface, libOffset, cb, dwLockType);
    STG_E_INVALIDFUNCTION
}
}

stdmethod! {
/// memlockbytes.c:478 HGLOBALLockBytesImpl_Stat [VTABLE]
fn HGLOBALLockBytesImpl_Stat(
    iface: *mut ILockBytes,
    pstatstg: *mut STATSTG,
    grfStatFlag: DWORD,
) -> HRESULT {
    let _ = grfStatFlag;
    let This = impl_from_ILockBytes(iface);

    memset(pstatstg as *mut c_void, 0, core::mem::size_of::<STATSTG>());

    (*pstatstg).pwcsName = core::ptr::null_mut();
    (*pstatstg).type_ = STGTY_LOCKBYTES;
    (*pstatstg).cbSize = (*This).byteArraySize;

    S_OK
}
}
