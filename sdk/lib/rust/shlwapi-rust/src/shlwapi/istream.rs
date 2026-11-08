use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// COBJMACROS, NONAMELESSUNION, NO_SHLWAPI_REG, NO_SHLWAPI_PATH: valueless preprocessor flags, no Rust constant emitted.
// WINE_DEFAULT_DEBUG_CHANNEL(shell): debug-channel macro invocation, not translated at file scope.
// STGM_ACCESS_MODE / STGM_SHARE_MODE / STGM_CREATE_MODE: function-like macros, inlined by the functions.
// static IStream_fnCommit prototype and SHLWAPI_fsVTable: skipped (fn declaration / COM vtable table).

#[repr(C)]
pub struct ISHFileStream {
    pub IStream_iface: IStream,
    pub ref_: LONG,
    pub hFile: HANDLE,
    pub dwMode: DWORD,
    pub lpszPath: LPOLESTR,
    pub type_: DWORD,
    pub grfStateBits: DWORD,
}
unsafe extern "C" fn impl_from_IStream(mut iface: *mut IStream) -> *mut ISHFileStream {
    return (iface as usize).wrapping_sub(core::mem::offset_of!(ISHFileStream, IStream_iface))
        as *mut ISHFileStream;
}
unsafe extern "system" fn IStream_fnQueryInterface(iface: *mut IStream, riid: *const IID, ppvObj: *mut LPVOID) -> HRESULT {
    let mut This: *mut ISHFileStream = impl_from_IStream(iface);

    *ppvObj = core::ptr::null_mut();

    if IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown)) != 0
        || IsEqualIID!(riid, core::ptr::addr_of!(IID_IStream)) != 0
    {
        IStream_AddRef(iface);
        *ppvObj = iface as LPVOID;
        return S_OK;
    }
    return E_NOINTERFACE;
}

unsafe extern "system" fn IStream_fnAddRef(iface: *mut IStream) -> ULONG {
    let This: *mut ISHFileStream = impl_from_IStream(iface);
    let refCount: ULONG =
        InterlockedIncrement(&raw mut (*This).ref_ as *mut ::core::ffi::c_long) as ULONG;

    TRACE!("(%p)->(ref before=%u)\n", This, refCount.wrapping_sub(1 as ULONG));

    return refCount;
}
unsafe extern "system" fn IStream_fnRelease(mut iface: *mut IStream) -> ULONG {
    let mut This: *mut ISHFileStream = impl_from_IStream(iface);
    let mut refCount: ULONG =
        InterlockedDecrement(core::ptr::addr_of_mut!((*This).ref_) as *mut ::core::ffi::c_long)
            as ULONG;
    WINE_TRACE!("(%p)->(ref before=%u)\n", This, refCount.wrapping_add(1 as ULONG));
    if refCount == 0 {
        IStream_fnCommit(iface, 0);
        LocalFree((*This).lpszPath as HLOCAL);
        CloseHandle((*This).hFile);
        HeapFree(GetProcessHeap(), 0, This as PVOID);
    }
    return refCount;
}
unsafe extern "system" fn IStream_fnRead(iface: *mut IStream, pv: *mut core::ffi::c_void, cb: ULONG, pcbRead: *mut ULONG) -> HRESULT {
    let This: *mut ISHFileStream = impl_from_IStream(iface);
    let mut dwRead: DWORD = 0;

    if ReadFile((*This).hFile, pv, cb, core::ptr::addr_of_mut!(dwRead), core::ptr::null_mut()) == 0 {
        WARN!("error %d reading file\n", GetLastError());
        return S_FALSE;
    }
    if !pcbRead.is_null() {
        *pcbRead = dwRead;
    }
    return if dwRead == cb { S_OK } else { S_FALSE };
}

unsafe extern "system" fn IStream_fnWrite(iface: *mut IStream, pv: *const core::ffi::c_void, cb: ULONG, pcbWritten: *mut ULONG) -> HRESULT {
    let mut This: *mut ISHFileStream = impl_from_IStream(iface);
    let mut dwWritten: DWORD = 0;

    let __mode: DWORD = STGM_ACCESS_MODE!((*This).dwMode);
    if __mode == STGM_WRITE || __mode == STGM_READWRITE {
    } else {
        return STG_E_ACCESSDENIED;
    }

    if WriteFile((*This).hFile, pv as LPCVOID, cb as DWORD, core::ptr::addr_of_mut!(dwWritten), core::ptr::null_mut()) == 0 {
        return HRESULT_FROM_WIN32!(GetLastError());
    }

    if !pcbWritten.is_null() {
        *pcbWritten = dwWritten as ULONG;
    }
    return S_OK;
}

unsafe extern "system" fn IStream_fnSeek(
    iface: *mut IStream,
    dlibMove: LARGE_INTEGER,
    dwOrigin: DWORD,
    pNewPos: *mut ULARGE_INTEGER,
) -> HRESULT {
    let This: *mut ISHFileStream = impl_from_IStream(iface);
    let mut dwPos: DWORD;

    IStream_fnCommit(iface, 0); /* If ever buffered, this will be needed */
    dwPos = SetFilePointer((*This).hFile, dlibMove.u.LowPart as LONG, core::ptr::null_mut::<LONG>(), dwOrigin);
    if dwPos == INVALID_SET_FILE_POINTER {
        return HRESULT_FROM_WIN32!(GetLastError());
    }

    if !pNewPos.is_null() {
        (*pNewPos).u.HighPart = 0;
        (*pNewPos).u.LowPart = dwPos;
    }
    return S_OK;
}

unsafe extern "system" fn IStream_fnSetSize(
    iface: *mut IStream,
    libNewSize: ULARGE_INTEGER,
) -> HRESULT {
    let mut This: *mut ISHFileStream = impl_from_IStream(iface);
    WINE_TRACE!("(%p,%d)\n", This, libNewSize.u.LowPart);
    IStream_fnCommit(iface, 0);
    if SetFilePointer(
        (*This).hFile,
        libNewSize.QuadPart as LONG,
        ::core::ptr::null_mut::<LONG>(),
        FILE_BEGIN as DWORD,
    ) == 0
    {
        return E_FAIL;
    }
    if SetEndOfFile((*This).hFile) == 0 {
        return E_FAIL;
    }
    return S_OK;
}

unsafe extern "system" fn IStream_fnCopyTo(iface: *mut IStream, pstm: *mut IStream, cb: ULARGE_INTEGER, pcbRead: *mut ULARGE_INTEGER, pcbWritten: *mut ULARGE_INTEGER) -> HRESULT {
    let mut This: *mut ISHFileStream = impl_from_IStream(iface);
    let mut copyBuff: [::core::ffi::c_char; 1024] = [0; 1024];
    let mut ulSize: ULONGLONG = 0;
    let mut hRet: HRESULT = S_OK;

    if !pcbRead.is_null() {
        (*pcbRead).QuadPart = 0;
    }
    if !pcbWritten.is_null() {
        (*pcbWritten).QuadPart = 0;
    }

    if pstm.is_null() {
        return S_OK;
    }

    IStream_fnCommit(iface, 0);

    /* Copy data */
    ulSize = cb.QuadPart;
    while ulSize != 0 {
        let mut ulLeft: ULONG = 0;
        let mut ulRead: ULONG = 0;
        let mut ulWritten: ULONG = 0;

        ulLeft = (if ulSize > core::mem::size_of::<[::core::ffi::c_char; 1024]>() as ULONGLONG {
            core::mem::size_of::<[::core::ffi::c_char; 1024]>() as ULONG
        } else {
            ulSize as ULONG
        });

        /* Read */
        hRet = IStream_fnRead(
            iface,
            &raw mut copyBuff as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
            ulLeft,
            &raw mut ulRead,
        );
        if hRet < 0 || ulRead == 0 {
            break;
        }
        if !pcbRead.is_null() {
            (*pcbRead).QuadPart = (*pcbRead).QuadPart.wrapping_add(ulRead as ULONGLONG);
        }

        /* Write */
        hRet = IStream_fnWrite(
            pstm,
            &raw mut copyBuff as *mut ::core::ffi::c_char as *const ::core::ffi::c_void,
            ulRead,
            &raw mut ulWritten,
        );
        if !pcbWritten.is_null() {
            (*pcbWritten).QuadPart = (*pcbWritten).QuadPart.wrapping_add(ulWritten as ULONGLONG);
        }
        if hRet < 0 || ulWritten != ulLeft {
            break;
        }

        ulSize = ulSize.wrapping_sub(ulLeft as ULONGLONG);
    }
    return hRet;
}

unsafe extern "system" fn IStream_fnCommit(iface: *mut IStream, grfCommitFlags: DWORD) -> HRESULT {
    let mut This: *mut ISHFileStream = impl_from_IStream(iface);

    /* Currently unbuffered: This function is not needed */
    return S_OK;
}
unsafe extern "system" fn IStream_fnRevert(iface: *mut IStream) -> HRESULT {
    let This: *mut ISHFileStream = impl_from_IStream(iface);

    return E_NOTIMPL;
}
unsafe extern "system" fn IStream_fnLockUnlockRegion(
    iface: *mut IStream,
    libOffset: ULARGE_INTEGER,
    cb: ULARGE_INTEGER,
    dwLockType: DWORD,
) -> HRESULT {
    let mut This: *mut ISHFileStream = impl_from_IStream(iface);
    WINE_TRACE!("(%p,%d,%d,%d)\n", This, libOffset.u.LowPart, cb.u.LowPart, dwLockType);
    return E_NOTIMPL;
}

unsafe extern "system" fn IStream_fnStat(iface: *mut IStream, lpStat: *mut STATSTG, grfStatFlag: DWORD) -> HRESULT {
    let This: *mut ISHFileStream = impl_from_IStream(iface);
    let mut fi: BY_HANDLE_FILE_INFORMATION = core::mem::zeroed();

    if lpStat.is_null() {
        return STG_E_INVALIDPOINTER;
    }

    memset(core::ptr::addr_of_mut!(fi) as *mut core::ffi::c_void, 0, core::mem::size_of::<BY_HANDLE_FILE_INFORMATION>() as size_t);
    GetFileInformationByHandle((*This).hFile, core::ptr::addr_of_mut!(fi));

    if (grfStatFlag & STATFLAG_NONAME) != 0 {
        (*lpStat).pwcsName = core::ptr::null_mut::<OLECHAR>();
    } else {
        (*lpStat).pwcsName = StrDupW((*This).lpszPath as *const WCHAR) as LPOLESTR;
    }
    (*lpStat).type_ = (*This).type_;
    (*lpStat).cbSize.u.LowPart = fi.nFileSizeLow;
    (*lpStat).cbSize.u.HighPart = fi.nFileSizeHigh;
    (*lpStat).mtime = fi.ftLastWriteTime;
    (*lpStat).ctime = fi.ftCreationTime;
    (*lpStat).atime = fi.ftLastAccessTime;
    (*lpStat).grfMode = (*This).dwMode;
    (*lpStat).grfLocksSupported = 0;
    memcpy(core::ptr::addr_of_mut!((*lpStat).clsid) as *mut core::ffi::c_void, core::ptr::addr_of!(IID_IStream) as *const core::ffi::c_void, core::mem::size_of::<CLSID>() as size_t);
    (*lpStat).grfStateBits = (*This).grfStateBits;
    (*lpStat).reserved = 0;

    return S_OK;
}

unsafe extern "system" fn IStream_fnClone(iface: *mut IStream, ppstm: *mut *mut IStream) -> HRESULT {
    let mut This: *mut ISHFileStream = impl_from_IStream(iface);

    if !ppstm.is_null() {
        *ppstm = core::ptr::null_mut();
    }
    return E_NOTIMPL;
}

unsafe extern "C" fn IStream_Create(lpszPath: LPCWSTR, hFile: HANDLE, dwMode: DWORD) -> *mut IStream {
    let mut fileStream: *mut ISHFileStream = core::ptr::null_mut::<ISHFileStream>();

    fileStream = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<ISHFileStream>() as SIZE_T,
    ) as *mut ISHFileStream;
    if fileStream.is_null() {
        return core::ptr::null_mut::<IStream>();
    }

    (*fileStream).IStream_iface.lpVtbl = core::ptr::addr_of!(SHLWAPI_fsVTable);
    (*fileStream).ref_ = 1;
    (*fileStream).hFile = hFile;
    (*fileStream).dwMode = dwMode;
    (*fileStream).lpszPath = StrDupW(lpszPath) as LPOLESTR;
    (*fileStream).type_ = 0; /* FIXME */
    (*fileStream).grfStateBits = 0; /* FIXME */

    TRACE!("Returning %p\n", fileStream);
    return core::ptr::addr_of_mut!((*fileStream).IStream_iface);
}

#[no_mangle]
pub unsafe extern "system" fn SHCreateStreamOnFileEx(
    lpszPath: LPCWSTR,
    dwMode: DWORD,
    dwAttributes: DWORD,
    bCreate: BOOL,
    lpTemplate: *mut IStream,
    lppStream: *mut *mut IStream,
) -> HRESULT {
    let mut dwAccess: DWORD = 0;
    let mut dwShare: DWORD = 0;
    let mut dwCreate: DWORD = 0;
    let mut hFile: HANDLE = core::ptr::null_mut::<core::ffi::c_void>();
    WINE_TRACE!("(%s,%d,0x%08X,%d,%p,%p)\n", debugstr_w(lpszPath as *const WCHAR), dwMode, dwAttributes, bCreate, lpTemplate, lppStream);
    if lpszPath.is_null() || lppStream.is_null() || !lpTemplate.is_null() {
        return E_INVALIDARG;
    }

    *lppStream = core::ptr::null_mut::<IStream>();

    /* Access */
    match STGM_ACCESS_MODE!(dwMode) {
        STGM_WRITE | STGM_READWRITE => {
            dwAccess = (GENERIC_READ | GENERIC_WRITE) as DWORD;
        }
        STGM_READ => {
            dwAccess = GENERIC_READ as DWORD;
        }
        _ => return E_INVALIDARG,
    }

    /* Sharing */
    match STGM_SHARE_MODE!(dwMode) {
        0 | STGM_SHARE_DENY_NONE => {
            dwShare = (FILE_SHARE_READ | FILE_SHARE_WRITE) as DWORD;
        }
        STGM_SHARE_DENY_READ => {
            dwShare = FILE_SHARE_WRITE as DWORD;
        }
        STGM_SHARE_DENY_WRITE => {
            dwShare = FILE_SHARE_READ as DWORD;
        }
        STGM_SHARE_EXCLUSIVE => {
            dwShare = 0 as DWORD;
        }
        _ => return E_INVALIDARG,
    }

    match STGM_CREATE_MODE!(dwMode) {
        STGM_FAILIFTHERE => {
            dwCreate = (if bCreate != 0 {
                CREATE_NEW
            } else {
                OPEN_EXISTING
            }) as DWORD;
        }
        STGM_CREATE => {
            dwCreate = CREATE_ALWAYS as DWORD;
        }
        _ => return E_INVALIDARG,
    }

    /* Open HANDLE to file */
    hFile = CreateFileW(
        lpszPath,
        dwAccess,
        dwShare,
        core::ptr::null_mut::<_SECURITY_ATTRIBUTES>(),
        dwCreate,
        dwAttributes,
        core::ptr::null_mut(),
    );

    if hFile == INVALID_HANDLE_VALUE {
        return HRESULT_FROM_WIN32!(GetLastError());
    }

    *lppStream = IStream_Create(lpszPath, hFile, dwMode);

    if (*lppStream).is_null() {
        CloseHandle(hFile);
        return E_OUTOFMEMORY;
    }
    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn SHCreateStreamOnFileW(
    mut lpszPath: LPCWSTR,
    mut dwMode: DWORD,
    mut lppStream: *mut *mut IStream,
) -> HRESULT {
    WINE_TRACE!("(%s,%d,%p)\n", debugstr_w(lpszPath as *const WCHAR), dwMode, lppStream);
    if lpszPath.is_null() || lppStream.is_null() {
        return E_INVALIDARG;
    }
    if (dwMode & (STGM_CONVERT | STGM_DELETEONRELEASE | STGM_TRANSACTED) as DWORD)
        != 0 as DWORD
    {
        return E_INVALIDARG;
    }
    return SHCreateStreamOnFileEx(
        lpszPath,
        dwMode,
        0 as DWORD,
        (FALSE) as BOOL,
        ::core::ptr::null_mut::<IStream>(),
        lppStream,
    );
}

#[no_mangle]
pub unsafe extern "system" fn SHCreateStreamOnFileA(
    lpszPath: LPCSTR,
    dwMode: DWORD,
    lppStream: *mut *mut IStream,
) -> HRESULT {
    let mut szPath: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    if lpszPath.is_null() {
        return HRESULT_FROM_WIN32!(ERROR_PATH_NOT_FOUND);
    }

    MultiByteToWideChar(
        CP_ACP as UINT,
        0 as DWORD,
        lpszPath,
        -1,
        core::ptr::addr_of_mut!(szPath) as *mut WCHAR as LPWSTR,
        (MAX_PATH as i32),
    );
    return SHCreateStreamOnFileW(
        core::ptr::addr_of_mut!(szPath) as *mut WCHAR as LPCWSTR,
        dwMode,
        lppStream,
    );
}

#[no_mangle]
pub unsafe extern "system" fn SHIStream_Read(
    lpStream: *mut IStream,
    lpvDest: LPVOID,
    ulSize: ULONG,
) -> HRESULT {
    let mut ulRead: ULONG = 0;
    let mut hRet: HRESULT = 0;

    hRet = IStream_Read(lpStream, lpvDest, ulSize, core::ptr::addr_of_mut!(ulRead));

    if (hRet >= 0) && ulRead != ulSize {
        hRet = E_FAIL;
    }
    return hRet;
}

#[no_mangle]
pub unsafe extern "system" fn SHIsEmptyStream(lpStream: *mut IStream) -> BOOL {
    let mut statstg: STATSTG = core::mem::zeroed();
    let mut bRet: BOOL = (TRUE as i32);

    memset(
        core::ptr::addr_of_mut!(statstg) as *mut core::ffi::c_void,
        0,
        core::mem::size_of::<STATSTG>(),
    );

    if (IStream_Stat(lpStream, core::ptr::addr_of_mut!(statstg), 1)) >= 0 {
        if statstg.cbSize.QuadPart != 0 {
            bRet = (FALSE as i32); /* Non-Zero */
        }
    } else {
        let mut dwDummy: DWORD = 0;

        /* Try to read from the stream */
        if (SHIStream_Read(
            lpStream,
            core::ptr::addr_of_mut!(dwDummy) as LPVOID,
            core::mem::size_of::<DWORD>() as ULONG,
        )) >= 0 {
            let mut zero: LARGE_INTEGER = core::mem::zeroed();
            zero.QuadPart = 0;

            IStream_Seek(
                lpStream,
                zero,
                0,
                core::ptr::null_mut::<ULARGE_INTEGER>(),
            );
            bRet = (FALSE as i32); /* Non-Zero */
        }
    }
    return bRet;
}

#[no_mangle]
pub unsafe extern "system" fn SHIStream_Write(lpStream: *mut IStream, lpvSrc: LPCVOID, ulSize: ULONG) -> HRESULT {
    let mut ulWritten: ULONG = 0;
    let mut hRet: HRESULT = 0;

    hRet = IStream_Write(lpStream, lpvSrc, ulSize, core::ptr::addr_of_mut!(ulWritten));

    if ((hRet) >= 0) && ulWritten != ulSize {
        hRet = E_FAIL;
    }

    return hRet;
}

#[no_mangle]
pub unsafe extern "system" fn IStream_Reset(lpStream: *mut IStream) -> HRESULT {
    let mut zero: LARGE_INTEGER = core::mem::zeroed();
    zero.QuadPart = 0;
    return IStream_Seek(lpStream, zero, 0, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn IStream_Size(
    lpStream: *mut IStream,
    lpulSize: *mut ULARGE_INTEGER,
) -> HRESULT {
    let mut statstg: STATSTG = core::mem::zeroed();
    let mut hRet: HRESULT = 0;
    WINE_TRACE!("(%p,%p)\n", lpStream, lpulSize);
    memset(
        core::ptr::addr_of_mut!(statstg) as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        core::mem::size_of::<STATSTG>() as size_t,
    );
    hRet = IStream_Stat(
        lpStream,
        core::ptr::addr_of_mut!(statstg),
        1 as DWORD,
    );
    if hRet >= 0 as HRESULT && !lpulSize.is_null() {
        *lpulSize = statstg.cbSize;
    }
    return hRet;
}

// COM vtable tables — generated from istream.c by vtbl_gen.py
/// istream.c:339 `static const IStreamVtbl SHLWAPI_fsVTable`  (slot order: com-sys/bindgen_i386.rs)
pub static SHLWAPI_fsVTable: IStreamVtbl = IStreamVtbl {
    QueryInterface: Some(IStream_fnQueryInterface),
    AddRef: Some(IStream_fnAddRef),
    Release: Some(IStream_fnRelease),
    Read: Some(IStream_fnRead),
    Write: Some(IStream_fnWrite),
    Seek: Some(IStream_fnSeek),
    SetSize: Some(IStream_fnSetSize),
    CopyTo: Some(IStream_fnCopyTo),
    Commit: Some(IStream_fnCommit),
    Revert: Some(IStream_fnRevert),
    LockRegion: Some(IStream_fnLockUnlockRegion),
    UnlockRegion: Some(IStream_fnLockUnlockRegion),
    Stat: Some(IStream_fnStat),
    Clone: Some(IStream_fnClone),
};
