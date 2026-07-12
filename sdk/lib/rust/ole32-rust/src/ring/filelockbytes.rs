use crate::prelude::*;
use core::ffi::c_void;

extern "C" {
    fn CloseHandle(hObject: HANDLE) -> BOOL;
    fn SetFilePointerEx( hFile: HANDLE, liDistanceToMove: LARGE_INTEGER, lpNewFilePointer: *mut LARGE_INTEGER, dwMoveMethod: DWORD, ) -> BOOL;
    fn ReadFile( hFile: HANDLE, lpBuffer: LPVOID, nNumberOfBytesToRead: DWORD, lpNumberOfBytesRead: LPDWORD, lpOverlapped: *mut OVERLAPPED, ) -> BOOL;
    fn WriteFile( hFile: HANDLE, lpBuffer: *const core::ffi::c_void, nNumberOfBytesToWrite: DWORD, lpNumberOfBytesWritten: *mut DWORD, lpOverlapped: *mut OVERLAPPED, ) -> BOOL;
    fn SetEndOfFile(hFile: HANDLE) -> BOOL;
    fn CoTaskMemAlloc(cb: usize) -> *mut core::ffi::c_void;
    fn GetFileSize(hFile: HANDLE, lpFileSizeHigh: *mut DWORD) -> DWORD;
    fn GetFullPathNameW( lpFileName: LPCWSTR, nBufferLength: DWORD, lpBuffer: LPWSTR, lpFilePart: *mut LPWSTR, ) -> DWORD;
    fn lstrcpynW(lpString1: LPWSTR, lpString2: LPCWSTR, iMaxLength: i32) -> LPWSTR;
    fn LockFileEx( hFile: HANDLE, dwFlags: DWORD, dwReserved: DWORD, nNumberOfBytesToLockLow: DWORD, nNumberOfBytesToLockHigh: DWORD, lpOverlapped: *mut OVERLAPPED, ) -> BOOL;
    fn UnlockFileEx( hFile: HANDLE, dwReserved: DWORD, nNumberOfBytesToUnlockLow: DWORD, nNumberOfBytesToUnlockHigh: DWORD, lpOverlapped: *mut OVERLAPPED, ) -> BOOL;
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
    QueryInterface: FileLockBytesImpl_QueryInterface,
    AddRef: FileLockBytesImpl_AddRef,
    Release: FileLockBytesImpl_Release,
    ReadAt: FileLockBytesImpl_ReadAt,
    WriteAt: FileLockBytesImpl_WriteAt,
    Flush: FileLockBytesImpl_Flush,
    SetSize: FileLockBytesImpl_SetSize,
    LockRegion: FileLockBytesImpl_LockRegion,
    UnlockRegion: FileLockBytesImpl_UnlockRegion,
    Stat: FileLockBytesImpl_Stat,
};

unsafe fn impl_from_ILockBytes(iface: *mut ILockBytes) -> *mut FileLockBytesImpl {
    (iface as *mut u8).offset(-((core::mem::offset_of!(FileLockBytesImpl, ILockBytes_iface)) as isize)) as *mut FileLockBytesImpl
}

unsafe fn GetProtectMode(openFlags: DWORD) -> DWORD {
    match STGM_ACCESS_MODE(openFlags) {
        STGM_WRITE | STGM_READWRITE => PAGE_READWRITE,
        _ => PAGE_READONLY,
    }
}
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
        core::mem::size_of::<FileLockBytesImpl>(),
    ) as *mut FileLockBytesImpl;

    if This.is_null() {
        return E_OUTOFMEMORY;
    }

    (*This).ILockBytes_iface.lpVtbl = &FileLockBytesImpl_Vtbl;
    (*This).ref_0 = 1;
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
            lstrcpynW(fullpath.as_mut_ptr(), pwcsName, MAX_PATH as i32);
        }
        (*This).pwcsName = HeapAlloc(
            GetProcessHeap(),
            0,
            ((lstrlenW(fullpath.as_ptr()) + 1) as usize) * core::mem::size_of::<WCHAR>(),
        ) as LPWSTR;
        if (*This).pwcsName.is_null() {
            HeapFree(GetProcessHeap(), 0, This as *mut core::ffi::c_void);
            return E_OUTOFMEMORY;
        }
        lstrcpyW((*This).pwcsName, fullpath.as_ptr());
    } else {
        (*This).pwcsName = core::ptr::null_mut();
    }

    *pLockBytes = &mut (*This).ILockBytes_iface;

    S_OK
}

unsafe fn FileLockBytesImpl_QueryInterface(
    iface: *mut ILockBytes,
    riid: REFIID,
    ppvObject: *mut *mut core::ffi::c_void,
) -> HRESULT {

    if IsEqualIID(riid, &IID_IUnknown) != 0 || IsEqualIID(riid, &IID_ILockBytes) != 0 {
        *ppvObject = iface as *mut core::ffi::c_void;
    } else {
        *ppvObject = core::ptr::null_mut();
        return E_NOINTERFACE;
    }

    IUnknown_AddRef(*ppvObject as *mut IUnknown);

    S_OK
}

unsafe fn FileLockBytesImpl_AddRef(iface: *mut ILockBytes) -> u32 {
    let This = impl_from_ILockBytes(iface);
    InterlockedIncrement(&mut (*This).ref_) as u32
}

unsafe fn FileLockBytesImpl_Release(iface: *mut ILockBytes) -> ULONG {
    let This = impl_from_ILockBytes(iface);
    let ref_count: ULONG;

    ref_count = InterlockedDecrement(&mut (*This).ref_) as ULONG;

    if ref_count == 0 {
        CloseHandle((*This).hfile);
        HeapFree(GetProcessHeap(), 0, (*This).pwcsName as *mut _);
        HeapFree(GetProcessHeap(), 0, This as *mut _);
    }

    return ref_count;
}

unsafe fn FileLockBytesImpl_ReadAt(
    iface: *mut ILockBytes,
    ulOffset: ULARGE_INTEGER,
    pv: *mut core::ffi::c_void,
    cb: ULONG,
    pcbRead: *mut ULONG,
) -> HRESULT {

    let This = impl_from_ILockBytes(iface);
    let mut bytes_left = cb;
    let mut readPtr = pv as LPBYTE;
    let mut ret: BOOL;
    let mut offset: LARGE_INTEGER = core::mem::zeroed();
    let mut cbRead: ULONG;

    if This.is_null() {
        return E_FAIL;
    }

    if !pcbRead.is_null() {
        *pcbRead = 0;
    }

    offset.QuadPart = ulOffset.QuadPart;

    ret = SetFilePointerEx((*This).hfile, offset, core::ptr::null_mut(), FILE_BEGIN);

    if ret == 0 {
        return STG_E_READFAULT;
    }

    while bytes_left != 0 {
        ret = ReadFile((*This).hfile, readPtr as LPVOID, bytes_left, &mut cbRead, core::ptr::null_mut());

        if ret == 0 || cbRead == 0 {
            return STG_E_READFAULT;
        }

        if !pcbRead.is_null() {
            *pcbRead += cbRead;
        }

        bytes_left -= cbRead;
        readPtr = readPtr.add(cbRead as usize);
    }

    S_OK
}

unsafe fn FileLockBytesImpl_WriteAt(
    iface: *mut ILockBytes,
    ulOffset: ULARGE_INTEGER,
    pv: *const core::ffi::c_void,
    cb: ULONG,
    pcbWritten: *mut ULONG,
) -> HRESULT {

    let This = impl_from_ILockBytes(iface);
    let mut bytes_left = cb;
    let mut writePtr = pv as *const BYTE;
    let mut ret: BOOL;
    let mut offset: LARGE_INTEGER = core::mem::zeroed();
    let mut cbWritten: ULONG;

    TRACE!(
        b"(%p)-> %i %p %i %p\n\0".as_ptr(),
        This,
        ulOffset.u.LowPart,
        pv,
        cb,
        pcbWritten,
    );

    if This.is_null() {
        return E_FAIL;
    }

    if (*This).flProtect != PAGE_READWRITE {
        return STG_E_ACCESSDENIED;
    }

    if !pcbWritten.is_null() {
        *pcbWritten = 0;
    }

    offset.QuadPart = ulOffset.u.QuadPart;

    ret = SetFilePointerEx((*This).hfile, offset, core::ptr::null_mut(), FILE_BEGIN);

    if ret == 0 {
        return STG_E_WRITEFAULT;
    }

    while bytes_left != 0 {
        ret = WriteFile(
            (*This).hfile,
            writePtr as *const core::ffi::c_void,
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

    TRACE!(b"finished\n\0".as_ptr());
    S_OK
}

unsafe fn FileLockBytesImpl_Flush(iface: *mut ILockBytes) -> HRESULT {
    S_OK
}

unsafe fn FileLockBytesImpl_SetSize(iface: *mut ILockBytes, newSize: ULARGE_INTEGER) -> HRESULT {

    let This = impl_from_ILockBytes(iface);
    let hr: HRESULT = 0;
    let mut newpos: LARGE_INTEGER = core::mem::zeroed();

    TRACE!(b"new size %u\n\0".as_ptr() as *const i8, (*newSize.u.s()).__bindgen_anon_1.LowPart);

    newpos.QuadPart = *newSize.QuadPart;
    if SetFilePointerEx((*This).hfile, newpos, core::ptr::null_mut(), FILE_BEGIN) != 0 {
        SetEndOfFile((*This).hfile);
    }

    hr
}

unsafe fn get_lock_error() -> HRESULT {
    match GetLastError() {
        ERROR_ACCESS_DENIED => STG_E_ACCESSDENIED,
        ERROR_LOCK_VIOLATION => STG_E_LOCKVIOLATION,
        ERROR_NOT_SUPPORTED => STG_E_INVALIDFUNCTION,
        _ => {
            FIXME!(b"no mapping for error %d\n\0".as_ptr() as *const i8, GetLastError());
            STG_E_INVALIDFUNCTION
        }
    }
}

unsafe fn FileLockBytesImpl_LockRegion(
    iface: *mut ILockBytes,
    libOffset: ULARGE_INTEGER,
    cb: ULARGE_INTEGER,
    dwLockType: DWORD,
) -> HRESULT {

    let This = impl_from_ILockBytes(iface);
    let mut ol: OVERLAPPED = core::mem::zeroed();
    let mut lock_flags = LOCKFILE_FAIL_IMMEDIATELY;

    TRACE!(
        b"ofs %u count %u flags %x\n\0".as_ptr() as LPCSTR,
        (*libOffset.u.s()).__bindgen_anon_1.LowPart,
        (*cb.u.s()).__bindgen_anon_1.LowPart,
        dwLockType,
    );

    if dwLockType & LOCK_WRITE != 0 {
        return STG_E_INVALIDFUNCTION;
    }

    if dwLockType & (LOCK_EXCLUSIVE | LOCK_ONLYONCE) != 0 {
        lock_flags |= LOCKFILE_EXCLUSIVE_LOCK;
    }

    ol.hEvent = core::ptr::null_mut();
    (*ol.u.s_mut()).Offset = (*libOffset.u.s()).__bindgen_anon_1.LowPart;
    (*ol.u.s_mut()).OffsetHigh = (*libOffset.u.s()).__bindgen_anon_1.HighPart;

    if LockFileEx(
        (*This).hfile,
        lock_flags,
        0,
        (*cb.u.s()).__bindgen_anon_1.LowPart,
        (*cb.u.s()).__bindgen_anon_1.HighPart,
        &mut ol,
    ) != 0
    {
        return S_OK;
    }
    get_lock_error()
}

unsafe fn FileLockBytesImpl_UnlockRegion(
    iface: *mut ILockBytes,
    libOffset: ULARGE_INTEGER,
    cb: ULARGE_INTEGER,
    dwLockType: DWORD,
) -> HRESULT {

    let This = impl_from_ILockBytes(iface);
    let mut ol: OVERLAPPED = core::mem::zeroed();

    TRACE!(
        b"ofs %u count %u flags %x\n\0".as_ptr() as *const i8,
        (*libOffset.u.s()).__bindgen_anon_1.LowPart,
        (*cb.u.s()).__bindgen_anon_1.LowPart,
        dwLockType,
    );

    if dwLockType & LOCK_WRITE != 0 {
        return STG_E_INVALIDFUNCTION;
    }

    ol.hEvent = core::ptr::null_mut();
    (*ol.u.s_mut()).Offset = (*libOffset.u.s()).__bindgen_anon_1.LowPart;
    (*ol.u.s_mut()).OffsetHigh = (*libOffset.u.s()).__bindgen_anon_1.HighPart;

    if UnlockFileEx(
        (*This).hfile,
        0,
        (*cb.u.s()).__bindgen_anon_1.LowPart,
        (*cb.u.s()).__bindgen_anon_1.HighPart,
        &mut ol,
    ) != 0
    {
        return S_OK;
    }
    get_lock_error()
}
unsafe fn FileLockBytesImpl_Stat(
    iface: *mut ILockBytes,
    pstatstg: *mut STATSTG,
    grfStatFlag: DWORD,
) -> HRESULT {

    let This = impl_from_ILockBytes(iface);

    if !(STATFLAG_NONAME & grfStatFlag != 0) && !(*This).pwcsName.is_null() {
        (*pstatstg).pwcsName = CoTaskMemAlloc(
            ((lstrlenW((*This).pwcsName) + 1) as usize) * core::mem::size_of::<WCHAR>()
        ) as LPWSTR;

        lstrcpyW((*pstatstg).pwcsName, (*This).pwcsName);
    } else {
        (*pstatstg).pwcsName = core::ptr::null_mut();
    }

    (*pstatstg).type_ = STGTY_LOCKBYTES;

    (*pstatstg).cbSize.u.__bindgen_anon_1.LowPart = GetFileSize(
        (*This).hfile,
        core::ptr::addr_of_mut!((*pstatstg).cbSize.u.__bindgen_anon_1.HighPart)
    );

    (*pstatstg).grfLocksSupported = LOCK_EXCLUSIVE | LOCK_ONLYONCE | WINE_LOCK_READ;

    S_OK
}
