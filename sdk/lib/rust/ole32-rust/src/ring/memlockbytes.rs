use crate::prelude::*;
use core::ffi::c_void;

extern "C" {

    fn memcpy(dest: *mut core::ffi::c_void, src: *const core::ffi::c_void, n: usize) -> *mut core::ffi::c_void;
}


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
    QueryInterface: HGLOBALLockBytesImpl_QueryInterface,
    AddRef: HGLOBALLockBytesImpl_AddRef,
    Release: HGLOBALLockBytesImpl_Release,
    ReadAt: HGLOBALLockBytesImpl_ReadAt,
    WriteAt: HGLOBALLockBytesImpl_WriteAt,
    Flush: HGLOBALLockBytesImpl_Flush,
    SetSize: HGLOBALLockBytesImpl_SetSize,
    LockRegion: HGLOBALLockBytesImpl_LockRegion,
    UnlockRegion: HGLOBALLockBytesImpl_UnlockRegion,
    Stat: HGLOBALLockBytesImpl_Stat,
};

unsafe fn impl_from_ILockBytes(iface: *mut ILockBytes) -> *mut HGLOBALLockBytesImpl {
    (iface as *mut u8).offset(-(core::mem::offset_of!(HGLOBALLockBytesImpl, ILockBytes_iface) as isize)) as *mut HGLOBALLockBytesImpl
}

#[no_mangle]
pub unsafe extern "system" fn CreateILockBytesOnHGlobal(
    global: HGLOBAL,
    delete_on_release: BOOL,
    ret: *mut *mut ILockBytes,
) -> HRESULT {

    let lockbytes = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<HGLOBALLockBytesImpl>(),
    ) as *mut HGLOBALLockBytesImpl;

    if lockbytes.is_null() {
        return E_OUTOFMEMORY;
    }

    (*lockbytes).ILockBytes_iface.lpVtbl = &HGLOBALLockBytesImpl_Vtbl;
    (*lockbytes).ref_ = 1;

    (*lockbytes).supportHandle = global;
    (*lockbytes).deleteOnRelease = delete_on_release;

    if (*lockbytes).supportHandle == 0 as HGLOBAL {
        (*lockbytes).supportHandle = GlobalAlloc(GMEM_MOVEABLE | GMEM_NODISCARD, 0);
    }

    (*lockbytes).byteArraySize.u.__bindgen_anon_1.HighPart = 0;
    (*lockbytes).byteArraySize.u.__bindgen_anon_1.LowPart = GlobalSize((*lockbytes).supportHandle) as u32;

    *ret = &mut (*lockbytes).ILockBytes_iface;

    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn GetHGlobalFromILockBytes(iface: *mut ILockBytes, phglobal: *mut HGLOBAL) -> HRESULT {
    let This = impl_from_ILockBytes(iface);
    let mut stbuf: STATSTG = core::mem::zeroed();
    let mut hres: HRESULT;
    let mut start: ULARGE_INTEGER = core::mem::zeroed();
    let mut xread: ULONG = 0;

    *phglobal = 0;
    if (*This).ILockBytes_iface.lpVtbl == core::ptr::addr_of!(HGLOBALLockBytesImpl_Vtbl) {
        *phglobal = (*This).supportHandle;
        if *phglobal == 0 {
            return E_INVALIDARG;
        }
        return S_OK;
    }

    hres = ILockBytes_Stat(iface, &mut stbuf, STATFLAG_NONAME);
    if hres != S_OK {
        ERR!(b"Cannot ILockBytes_Stat, %x\n\0".as_ptr() as PCHAR, hres);
        return hres;
    }
    TRACE!(b"cbSize is %s\n\0".as_ptr() as PCHAR, wine_dbgstr_longlong(stbuf.cbSize.QuadPart));
    *phglobal = GlobalAlloc(GMEM_MOVEABLE | GMEM_SHARE, stbuf.cbSize.u.__bindgen_anon_1.LowPart);
    if *phglobal == 0 {
        return E_INVALIDARG;
    }
    memset(core::ptr::addr_of_mut!(start) as *mut core::ffi::c_void, 0, core::mem::size_of::<ULARGE_INTEGER>());
    hres = ILockBytes_ReadAt(iface, start, GlobalLock(*phglobal), stbuf.cbSize.u.__bindgen_anon_1.LowPart, &mut xread);
    GlobalUnlock(*phglobal);
    if hres != S_OK {
        FIXME!(b"%p->ReadAt failed with %x\n\0".as_ptr() as PCHAR, iface, hres);
        return hres;
    }
    if stbuf.cbSize.u.__bindgen_anon_1.LowPart != xread {
        FIXME!(b"Read size is not requested size %d vs %d?\n\0".as_ptr() as PCHAR, stbuf.cbSize.u.__bindgen_anon_1.LowPart, xread);
    }
    S_OK
}

unsafe fn HGLOBALLockBytesImpl_QueryInterface(
    iface: *mut ILockBytes,
    riid: *const IID,
    ppvObject: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let This = impl_from_ILockBytes(iface);

    if ppvObject.is_null() {
        return E_INVALIDARG;
    }

    *ppvObject = core::ptr::null_mut();

    if IsEqualIID(riid, &IID_IUnknown) != 0 ||
       IsEqualIID(riid, &IID_ILockBytes) != 0
    {
        *ppvObject = core::ptr::addr_of_mut!((*This).ILockBytes_iface) as *mut core::ffi::c_void;
    } else {
        return E_NOINTERFACE;
    }

    ILockBytes_AddRef(iface);

    return S_OK;
}

unsafe fn HGLOBALLockBytesImpl_AddRef(iface: *mut ILockBytes) -> ULONG {
    let This = impl_from_ILockBytes(iface);
    InterlockedIncrement(&mut (*This).ref_) as ULONG
}
unsafe fn HGLOBALLockBytesImpl_Release(iface: *mut ILockBytes) -> ULONG {

    let This = impl_from_ILockBytes(iface);
    let ref_count = InterlockedDecrement(&mut (*This).ref_);
    if ref_count == 0 {
        if (*This).deleteOnRelease != 0 {
            GlobalFree((*This).supportHandle);
            (*This).supportHandle = 0 as HGLOBAL;
        }
        HeapFree(GetProcessHeap(), 0, This as *mut core::ffi::c_void);
    }

    ref_count as ULONG
}
unsafe fn HGLOBALLockBytesImpl_ReadAt(
    iface: *mut ILockBytes,
    ulOffset: ULARGE_INTEGER,
    pv: *mut core::ffi::c_void,
    cb: ULONG,
    pcbRead: *mut ULONG,
) -> HRESULT {

    let This = impl_from_ILockBytes(iface);

    let mut bytesReadBuffer: ULONG = 0;
    let pcbRead = if pcbRead.is_null() {
        &mut bytesReadBuffer as *mut ULONG
    } else {
        pcbRead
    };

    if (*ulOffset.u).__bindgen_anon_1.LowPart > (*(*This).byteArraySize.u).__bindgen_anon_1.LowPart {
        return E_FAIL;
    }

    let bytesToReadFromBuffer = core::cmp::min(
        (*(*This).byteArraySize.u).__bindgen_anon_1.LowPart - (*ulOffset.u).__bindgen_anon_1.LowPart,
        cb
    );

    let supportBuffer = GlobalLock((*This).supportHandle);

    memcpy(
        pv,
        (supportBuffer as *const u8).add((*ulOffset.u).__bindgen_anon_1.LowPart as usize) as *const core::ffi::c_void,
        bytesToReadFromBuffer as usize
    );

    *pcbRead = bytesToReadFromBuffer;

    GlobalUnlock((*This).supportHandle);

    if *pcbRead == cb {
        return S_OK;
    }

    STG_E_READFAULT
}

unsafe fn HGLOBALLockBytesImpl_WriteAt(
    iface: *mut ILockBytes,
    ulOffset: ULARGE_INTEGER,
    pv: *const core::ffi::c_void,
    cb: u32,
    pcbWritten: *mut u32,
) -> HRESULT {

    let This = impl_from_ILockBytes(iface);

    let supportBuffer: *mut core::ffi::c_void;
    let mut newSize: ULARGE_INTEGER = core::mem::zeroed();
    let mut bytesWritten: u32 = 0;

    let pcbWritten = if pcbWritten.is_null() {
        &mut bytesWritten as *mut u32
    } else {
        pcbWritten
    };

    if cb == 0 {
        return 0; // S_OK
    } else {
        newSize.u.__bindgen_anon_1.HighPart = 0;
        newSize.u.__bindgen_anon_1.LowPart = (*ulOffset.u).__bindgen_anon_1.LowPart + cb;
    }

    if newSize.u.__bindgen_anon_1.LowPart > (*(*This).byteArraySize.u).__bindgen_anon_1.LowPart {
        if ILockBytes_SetSize(iface, newSize) == 0x80030070 {
            return 0x80030070; // STG_E_MEDIUMFULL
        }
    }

    supportBuffer = GlobalLock((*This).supportHandle);

    memcpy(
        (supportBuffer as *mut u8).add((*ulOffset.u).__bindgen_anon_1.LowPart as usize) as *mut core::ffi::c_void,
        pv,
        cb as usize,
    );

    *pcbWritten = cb;

    GlobalUnlock((*This).supportHandle);

    return 0; // S_OK
}

unsafe fn HGLOBALLockBytesImpl_Flush(iface: *mut ILockBytes) -> HRESULT {
    S_OK
}
unsafe fn HGLOBALLockBytesImpl_SetSize(
    iface: *mut ILockBytes,
    libNewSize: ULARGE_INTEGER,
) -> HRESULT {
    let This = impl_from_ILockBytes(iface);
    let mut supportHandle: HGLOBAL;

    if libNewSize.u.HighPart != 0 {
        return STG_E_INVALIDFUNCTION;
    }

    if (*This).byteArraySize.u.LowPart == libNewSize.u.LowPart {
        return S_OK;
    }

    supportHandle = GlobalReAlloc((*This).supportHandle, libNewSize.u.LowPart, 0);

    if supportHandle == 0 as HGLOBAL {
        return STG_E_MEDIUMFULL;
    }

    (*This).supportHandle = supportHandle;
    (*This).byteArraySize.u.LowPart = libNewSize.u.LowPart;

    return S_OK;
}

unsafe fn HGLOBALLockBytesImpl_LockRegion(
    iface: *mut ILockBytes,
    libOffset: ULARGE_INTEGER,
    cb: ULARGE_INTEGER,
    dwLockType: DWORD,
) -> HRESULT {
    STG_E_INVALIDFUNCTION
}

unsafe fn HGLOBALLockBytesImpl_UnlockRegion(
    iface: *mut ILockBytes,
    libOffset: ULARGE_INTEGER,
    cb: ULARGE_INTEGER,
    dwLockType: DWORD,
) -> HRESULT {
    STG_E_INVALIDFUNCTION
}

unsafe fn HGLOBALLockBytesImpl_Stat(
    iface: *mut ILockBytes,
    pstatstg: *mut STATSTG,
    grfStatFlag: DWORD,
) -> HRESULT {
    let This = impl_from_ILockBytes(iface);

    core::ptr::write_bytes(pstatstg, 0, 1);

    (*pstatstg).pwcsName = core::ptr::null_mut();
    (*pstatstg).type_ = STGTY_LOCKBYTES;
    (*pstatstg).cbSize = (*This).byteArraySize;

    S_OK
}
