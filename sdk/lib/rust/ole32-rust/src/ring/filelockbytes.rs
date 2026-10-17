//! filelockbytes.c — file-backed ILockBytes implementation.
use crate::prelude::*;

// ---------------------------------------------------------------------------
// Locally-defined bits the shared surface does not carry yet.
//
// These are deliberately PRIVATE (no `pub`) so they are not re-exported through
// `crate::prelude`'s `pub use crate::ring::filelockbytes::*` glob and cannot
// collide with a sibling ring module. If/when they are hoisted into
// crate::win_consts / a shared win32 types module, delete them from here.
// ---------------------------------------------------------------------------

/// winbase.h OVERLAPPED. The `union { struct { DWORD Offset; DWORD OffsetHigh; };
/// PVOID Pointer; }` is flattened to its two-DWORD arm — that arm is the only one
/// LockFileEx/UnlockFileEx use, and the flat form is layout-identical on all three
/// arches (the union's pointer arm has the same size+alignment as the DWORD pair
/// under the surrounding ULONG_PTR alignment). Same shape as marshal/types.rs.
#[repr(C)]
struct OVERLAPPED {
    Internal: usize,
    InternalHigh: usize,
    Offset: DWORD,
    OffsetHigh: DWORD,
    hEvent: HANDLE,
}

/// winerror.h
const ERROR_ACCESS_DENIED: DWORD = 5;
const ERROR_LOCK_VIOLATION: DWORD = 33;
const ERROR_NOT_SUPPORTED: DWORD = 50;

/// objbase.h: `#define STGM_ACCESS_MODE(stgm) ((stgm) & 0x0000000f)`
#[inline]
fn STGM_ACCESS_MODE(stgm: DWORD) -> DWORD {
    stgm & 0x0000_000f
}

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct FileLockBytesImpl {
    pub ILockBytes_iface: ILockBytes,
    pub ref_: LONG,
    pub hfile: HANDLE,
    pub flProtect: DWORD,
    pub pwcsName: LPWSTR,
}

pub type PFileLockBytesImpl = *mut FileLockBytesImpl;

pub static FileLockBytesImpl_Vtbl: ILockBytesVtbl = ILockBytesVtbl {
    QueryInterface: Some(FileLockBytesImpl_QueryInterface),
    AddRef: Some(FileLockBytesImpl_AddRef),
    Release: Some(FileLockBytesImpl_Release),
    ReadAt: Some(FileLockBytesImpl_ReadAt),
    WriteAt: Some(FileLockBytesImpl_WriteAt),
    Flush: Some(FileLockBytesImpl_Flush),
    SetSize: Some(FileLockBytesImpl_SetSize),
    LockRegion: Some(FileLockBytesImpl_LockRegion),
    UnlockRegion: Some(FileLockBytesImpl_UnlockRegion),
    Stat: Some(FileLockBytesImpl_Stat),
};

unsafe fn impl_from_ILockBytes(iface: *mut ILockBytes) -> *mut FileLockBytesImpl {
    CONTAINING_RECORD!(iface, FileLockBytesImpl, ILockBytes_iface)
}

/// filelockbytes.c:73 GetProtectMode
fn GetProtectMode(openFlags: DWORD) -> DWORD {
    match STGM_ACCESS_MODE(openFlags) {
        STGM_WRITE | STGM_READWRITE => PAGE_READWRITE,
        _ => PAGE_READONLY,
    }
}

/// filelockbytes.c:89 FileLockBytesImpl_Construct
///
/// Not WINAPI in the C (a plain internal `HRESULT FileLockBytesImpl_Construct(...)`
/// called from storage32), so cdecl — `extern "C"` is correct here, unlike the
/// WINAPI entry points.
#[no_mangle]
pub unsafe extern "C" fn FileLockBytesImpl_Construct(
    hFile: HANDLE,
    openFlags: DWORD,
    pwcsName: LPCWSTR,
    pLockBytes: *mut *mut ILockBytes,
) -> HRESULT {
    let mut fullpath: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];

    if hFile == INVALID_HANDLE_VALUE {
        return E_FAIL;
    }

    let This = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<FileLockBytesImpl>() as usize,
    ) as *mut FileLockBytesImpl;

    if This.is_null() {
        return E_OUTOFMEMORY;
    }

    (*This).ILockBytes_iface.lpVtbl =
        &FileLockBytesImpl_Vtbl as *const ILockBytesVtbl as *mut ILockBytesVtbl;
    (*This).ref_ = 1;
    (*This).hfile = hFile;
    (*This).flProtect = GetProtectMode(openFlags);

    if !pwcsName.is_null() {
        if GetFullPathNameW(
            pwcsName,
            MAX_PATH,
            fullpath.as_mut_ptr(),
            core::ptr::null_mut(),
        ) == 0
        {
            lstrcpynW(fullpath.as_mut_ptr(), pwcsName, MAX_PATH as c_int);
        }
        (*This).pwcsName = HeapAlloc(
            GetProcessHeap(),
            0,
            (((lstrlenW(fullpath.as_ptr()) + 1) as usize) * core::mem::size_of::<WCHAR>())
                as usize,
        ) as LPWSTR;
        if (*This).pwcsName.is_null() {
            HeapFree(GetProcessHeap(), 0, This as LPVOID);
            return E_OUTOFMEMORY;
        }
        lstrcpyW((*This).pwcsName, fullpath.as_ptr());
    } else {
        (*This).pwcsName = core::ptr::null_mut();
    }

    *pLockBytes = core::ptr::addr_of_mut!((*This).ILockBytes_iface);

    S_OK
}

stdmethod! {
/// filelockbytes.c:131 FileLockBytesImpl_QueryInterface [VTABLE]
fn FileLockBytesImpl_QueryInterface(
    iface: *mut ILockBytes,
    riid: REFIID,
    ppvObject: *mut *mut c_void,
) -> HRESULT {
    if IsEqualIID(riid, core::ptr::addr_of!(IID_IUnknown))
        || IsEqualIID(riid, core::ptr::addr_of!(IID_ILockBytes))
    {
        *ppvObject = iface as *mut c_void;
    } else {
        *ppvObject = core::ptr::null_mut();
        return E_NOINTERFACE;
    }

    IUnknown_AddRef(*ppvObject as *mut IUnknown);

    S_OK
}
}

stdmethod! {
/// filelockbytes.c:147 FileLockBytesImpl_AddRef [VTABLE]
fn FileLockBytesImpl_AddRef(iface: *mut ILockBytes) -> ULONG {
    let This = impl_from_ILockBytes(iface);
    InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref_)) as ULONG
}
}

stdmethod! {
/// filelockbytes.c:153 FileLockBytesImpl_Release [VTABLE]
fn FileLockBytesImpl_Release(iface: *mut ILockBytes) -> ULONG {
    let This = impl_from_ILockBytes(iface);

    let ref_count = InterlockedDecrement(core::ptr::addr_of_mut!((*This).ref_)) as ULONG;

    if ref_count == 0 {
        CloseHandle((*This).hfile);
        HeapFree(GetProcessHeap(), 0, (*This).pwcsName as LPVOID);
        HeapFree(GetProcessHeap(), 0, This as LPVOID);
    }

    ref_count
}
}

stdmethod! {
/// filelockbytes.c:178 FileLockBytesImpl_ReadAt [VTABLE]
fn FileLockBytesImpl_ReadAt(
    iface: *mut ILockBytes,
    ulOffset: ULARGE_INTEGER,
    pv: *mut c_void,
    cb: ULONG,
    pcbRead: *mut ULONG,
) -> HRESULT {
    let This = impl_from_ILockBytes(iface);
    let mut bytes_left: ULONG = cb;
    let mut readPtr = pv as *mut u8;
    let mut ret: BOOL;
    let mut offset: LARGE_INTEGER = core::mem::zeroed();
    let mut cbRead: ULONG = 0;

    TRACE!(
        "(%p)-> %i %p %i %p\n",
        This,
        ulOffset.u.LowPart,
        pv,
        cb,
        pcbRead
    );

    /* verify a sane environment */
    if This.is_null() {
        return E_FAIL;
    }

    if !pcbRead.is_null() {
        *pcbRead = 0;
    }

    // Full 64-bit offset — NOT just LowPart.
    offset.QuadPart = ulOffset.QuadPart as LONGLONG;

    ret = SetFilePointerEx((*This).hfile, offset, core::ptr::null_mut(), FILE_BEGIN);

    if ret == 0 {
        return STG_E_READFAULT;
    }

    while bytes_left != 0 {
        ret = ReadFile(
            (*This).hfile,
            readPtr as LPVOID,
            bytes_left,
            core::ptr::addr_of_mut!(cbRead),
            core::ptr::null_mut(),
        );

        if ret == 0 || cbRead == 0 {
            return STG_E_READFAULT;
        }

        if !pcbRead.is_null() {
            *pcbRead += cbRead;
        }

        bytes_left -= cbRead;
        readPtr = readPtr.add(cbRead as usize);
    }

    TRACE!("finished\n");
    S_OK
}
}

stdmethod! {
/// filelockbytes.c:233 FileLockBytesImpl_WriteAt [VTABLE]
fn FileLockBytesImpl_WriteAt(
    iface: *mut ILockBytes,
    ulOffset: ULARGE_INTEGER,
    pv: *const c_void,
    cb: ULONG,
    pcbWritten: *mut ULONG,
) -> HRESULT {
    let This = impl_from_ILockBytes(iface);
    let mut bytes_left: ULONG = cb;
    let mut writePtr = pv as *const u8;
    let mut ret: BOOL;
    let mut offset: LARGE_INTEGER = core::mem::zeroed();
    let mut cbWritten: ULONG = 0;

    TRACE!(
        "(%p)-> %i %p %i %p\n",
        This,
        ulOffset.u.LowPart,
        pv,
        cb,
        pcbWritten
    );

    /* verify a sane environment */
    if This.is_null() {
        return E_FAIL;
    }

    if (*This).flProtect != PAGE_READWRITE {
        return STG_E_ACCESSDENIED;
    }

    if !pcbWritten.is_null() {
        *pcbWritten = 0;
    }

    // Full 64-bit offset — NOT just LowPart.
    offset.QuadPart = ulOffset.QuadPart as LONGLONG;

    ret = SetFilePointerEx((*This).hfile, offset, core::ptr::null_mut(), FILE_BEGIN);

    if ret == 0 {
        return STG_E_WRITEFAULT;
    }

    while bytes_left != 0 {
        ret = WriteFile(
            (*This).hfile,
            writePtr as LPCVOID,
            bytes_left,
            core::ptr::addr_of_mut!(cbWritten),
            core::ptr::null_mut(),
        );

        if ret == 0 {
            return STG_E_WRITEFAULT;
        }

        if !pcbWritten.is_null() {
            *pcbWritten += cbWritten;
        }

        bytes_left -= cbWritten;
        writePtr = writePtr.add(cbWritten as usize);
    }

    TRACE!("finished\n");
    S_OK
}
}

stdmethod! {
/// filelockbytes.c:283 FileLockBytesImpl_Flush [VTABLE]
fn FileLockBytesImpl_Flush(iface: *mut ILockBytes) -> HRESULT {
    let _ = iface;
    S_OK
}
}

stdmethod! {
/// filelockbytes.c:294 FileLockBytesImpl_SetSize [VTABLE]
fn FileLockBytesImpl_SetSize(iface: *mut ILockBytes, newSize: ULARGE_INTEGER) -> HRESULT {
    let This = impl_from_ILockBytes(iface);
    let hr: HRESULT = S_OK;
    let mut newpos: LARGE_INTEGER = core::mem::zeroed();

    TRACE!("new size %u\n", newSize.u.LowPart);

    // Full 64-bit size — NOT just LowPart.
    newpos.QuadPart = newSize.QuadPart as LONGLONG;
    if SetFilePointerEx((*This).hfile, newpos, core::ptr::null_mut(), FILE_BEGIN) != 0 {
        SetEndOfFile((*This).hfile);
    }

    hr
}
}

/// filelockbytes.c:311 get_lock_error
unsafe fn get_lock_error() -> HRESULT {
    match GetLastError() {
        ERROR_LOCK_VIOLATION => STG_E_LOCKVIOLATION,
        ERROR_ACCESS_DENIED => STG_E_ACCESSDENIED,
        ERROR_NOT_SUPPORTED => STG_E_INVALIDFUNCTION,
        _ => {
            FIXME!("no mapping for error %d\n", GetLastError());
            STG_E_INVALIDFUNCTION
        }
    }
}

stdmethod! {
/// filelockbytes.c:324 FileLockBytesImpl_LockRegion [VTABLE]
fn FileLockBytesImpl_LockRegion(
    iface: *mut ILockBytes,
    libOffset: ULARGE_INTEGER,
    cb: ULARGE_INTEGER,
    dwLockType: DWORD,
) -> HRESULT {
    let This = impl_from_ILockBytes(iface);
    let mut ol: OVERLAPPED = core::mem::zeroed();
    let mut lock_flags: DWORD = LOCKFILE_FAIL_IMMEDIATELY;

    TRACE!(
        "ofs %u count %u flags %x\n",
        libOffset.u.LowPart,
        cb.u.LowPart,
        dwLockType
    );

    if dwLockType & LOCK_WRITE != 0 {
        return STG_E_INVALIDFUNCTION;
    }

    if dwLockType & (LOCK_EXCLUSIVE | LOCK_ONLYONCE) != 0 {
        lock_flags |= LOCKFILE_EXCLUSIVE_LOCK;
    }

    ol.hEvent = core::ptr::null_mut();
    ol.Offset = libOffset.u.LowPart;
    ol.OffsetHigh = libOffset.u.HighPart;

    if LockFileEx(
        (*This).hfile,
        lock_flags,
        0,
        cb.u.LowPart,
        cb.u.HighPart,
        core::ptr::addr_of_mut!(ol) as *mut c_void,
    ) != 0
    {
        return S_OK;
    }
    get_lock_error()
}
}

stdmethod! {
/// filelockbytes.c:348 FileLockBytesImpl_UnlockRegion [VTABLE]
fn FileLockBytesImpl_UnlockRegion(
    iface: *mut ILockBytes,
    libOffset: ULARGE_INTEGER,
    cb: ULARGE_INTEGER,
    dwLockType: DWORD,
) -> HRESULT {
    let This = impl_from_ILockBytes(iface);
    let mut ol: OVERLAPPED = core::mem::zeroed();

    TRACE!(
        "ofs %u count %u flags %x\n",
        libOffset.u.LowPart,
        cb.u.LowPart,
        dwLockType
    );

    if dwLockType & LOCK_WRITE != 0 {
        return STG_E_INVALIDFUNCTION;
    }

    ol.hEvent = core::ptr::null_mut();
    ol.Offset = libOffset.u.LowPart;
    ol.OffsetHigh = libOffset.u.HighPart;

    if UnlockFileEx(
        (*This).hfile,
        0,
        cb.u.LowPart,
        cb.u.HighPart,
        core::ptr::addr_of_mut!(ol) as *mut c_void,
    ) != 0
    {
        return S_OK;
    }
    get_lock_error()
}
}

stdmethod! {
/// filelockbytes.c:368 FileLockBytesImpl_Stat [VTABLE]
fn FileLockBytesImpl_Stat(
    iface: *mut ILockBytes,
    pstatstg: *mut STATSTG,
    grfStatFlag: DWORD,
) -> HRESULT {
    let This = impl_from_ILockBytes(iface);

    if (STATFLAG_NONAME & grfStatFlag) == 0 && !(*This).pwcsName.is_null() {
        (*pstatstg).pwcsName = CoTaskMemAlloc(
            (((lstrlenW((*This).pwcsName) + 1) as usize) * core::mem::size_of::<WCHAR>())
                as SIZE_T,
        ) as LPWSTR;

        lstrcpyW((*pstatstg).pwcsName, (*This).pwcsName);
    } else {
        (*pstatstg).pwcsName = core::ptr::null_mut();
    }

    (*pstatstg).type_ = STGTY_LOCKBYTES;

    // GetFileSize writes the high dword through the out-param — the HighPart of
    // cbSize must be the destination, otherwise files > 4 GB report a truncated size.
    (*pstatstg).cbSize.u.LowPart = GetFileSize(
        (*This).hfile,
        core::ptr::addr_of_mut!((*pstatstg).cbSize.u.HighPart),
    );

    /* FIXME: If the implementation is exported, we'll need to set other fields. */

    (*pstatstg).grfLocksSupported =
        (LOCK_EXCLUSIVE | LOCK_ONLYONCE | WINE_LOCK_READ) as DWORD;

    S_OK
}
}
