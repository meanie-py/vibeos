use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_16 {
    pub keyNameA: LPSTR,
    pub keyNameW: LPWSTR,
}

static mut rsDummyRegStream: ISHRegStream = unsafe {
    ISHRegStream {
        IStream_iface: IStream {
            lpVtbl: &raw const DummyRegStreamVTable,
        },
        ref_: 1 as LONG,
        hKey: ::core::ptr::null::<HKEY__>() as *mut HKEY__,
        pbBuffer: ::core::ptr::null::<BYTE>() as *mut BYTE,
        dwLength: 0 as DWORD,
        dwPos: 0 as DWORD,
        dwMode: STGM_READWRITE as DWORD,
        u: C2RustUnnamed_16 {
            keyNameA: ::core::ptr::null::<CHAR>() as *mut CHAR,
        },
        bUnicode: (FALSE as i32),
    }
};

// WINE_DEFAULT_DEBUG_CHANNEL(shell)
// COBJMACROS: no declaration emitted


pub type ISHRegStream_u = C2RustUnnamed_16;

#[repr(C)]
pub struct ISHRegStream {
    pub IStream_iface: IStream,
    pub ref_: LONG,
    pub hKey: HKEY,
    pub pbBuffer: LPBYTE,
    pub dwLength: DWORD,
    pub dwPos: DWORD,
    pub dwMode: DWORD,
    pub u: ISHRegStream_u,
    pub bUnicode: BOOL,
}
pub type PISHRegStream = *mut ISHRegStream;

unsafe extern "C" fn impl_from_IStream(iface: *mut IStream) -> *mut ISHRegStream {
    return (iface as *mut u8).sub(core::mem::offset_of!(ISHRegStream, IStream_iface)) as *mut ISHRegStream;
}

unsafe extern "system" fn IStream_fnQueryInterface(iface: *mut IStream, riid: REFIID, ppvObj: *mut LPVOID) -> HRESULT {
	let This: *mut ISHRegStream = impl_from_IStream(iface);

	*ppvObj = core::ptr::null_mut();

	if IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown)) != 0 || IsEqualIID!(riid, core::ptr::addr_of!(IID_IStream)) != 0 {
		*ppvObj = core::ptr::addr_of_mut!((*This).IStream_iface) as LPVOID;
	}

	if !(*ppvObj).is_null() {
		IStream_AddRef(*ppvObj as *mut IStream);
		return S_OK;
	}

	return E_NOINTERFACE;
}

unsafe extern "system" fn IStream_fnAddRef(iface: *mut IStream) -> ULONG {
    let mut This: *mut ISHRegStream = impl_from_IStream(iface);
    let mut refCount: ULONG =
        InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref_) as *mut i32) as ULONG;
    TRACE!("(%p)->(ref before=%u)\n", This, refCount.wrapping_sub(1));

    return refCount;
}

unsafe extern "system" fn IStream_fnRelease(iface: *mut IStream) -> ULONG {
    let This: *mut ISHRegStream = impl_from_IStream(iface);
    let refCount: ULONG = InterlockedDecrement(core::ptr::addr_of_mut!((*This).ref_) as *mut ::core::ffi::c_long) as ULONG;

    if refCount == 0 {
        if !(*This).hKey.is_null() {
            // write back data in REG_BINARY
            if (*This).dwMode == STGM_READWRITE || (*This).dwMode == STGM_WRITE {
                if (*This).dwLength != 0 {
                    if (*This).bUnicode != 0 {
                        RegSetValueExW(
                            (*This).hKey,
                            (*This).u.keyNameW as LPCWSTR,
                            0,
                            REG_BINARY,
                            (*This).pbBuffer as *const BYTE,
                            (*This).dwLength,
                        );
                    } else {
                        RegSetValueExA(
                            (*This).hKey,
                            (*This).u.keyNameA as LPCSTR,
                            0,
                            REG_BINARY,
                            (*This).pbBuffer as *const BYTE,
                            (*This).dwLength,
                        );
                    }
                } else {
                    if (*This).bUnicode != 0 {
                        RegDeleteValueW((*This).hKey, (*This).u.keyNameW as LPCWSTR);
                    } else {
                        RegDeleteValueA((*This).hKey, (*This).u.keyNameA as LPCSTR);
                    }
                }
            }

            RegCloseKey((*This).hKey);
        }

        HeapFree(GetProcessHeap(), 0, (*This).u.keyNameA as PVOID);
        HeapFree(GetProcessHeap(), 0, (*This).pbBuffer as PVOID);
        HeapFree(GetProcessHeap(), 0, This as PVOID);
        return 0;
    }

    return refCount;
}

unsafe extern "system" fn IStream_fnRead(iface: *mut IStream, pv: *mut ::core::ffi::c_void, cb: ULONG, pcbRead: *mut ULONG) -> HRESULT {
    let This: *mut ISHRegStream = impl_from_IStream(iface);
    let mut dwBytesToRead: DWORD;

    if (*This).dwPos >= (*This).dwLength {
        dwBytesToRead = 0;
    } else {
        dwBytesToRead = (*This).dwLength.wrapping_sub((*This).dwPos);
    }

    dwBytesToRead = if cb > dwBytesToRead { dwBytesToRead } else { cb };
    if dwBytesToRead != 0 {
        memmove(pv, (*This).pbBuffer.offset((*This).dwPos as isize) as *const ::core::ffi::c_void, dwBytesToRead as usize);
        (*This).dwPos = (*This).dwPos.wrapping_add(dwBytesToRead);
    }

    if !pcbRead.is_null() {
        *pcbRead = dwBytesToRead;
    }

    return S_OK;
}

unsafe extern "system" fn IStream_fnWrite(
    iface: *mut IStream,
    pv: *const core::ffi::c_void,
    cb: ULONG,
    pcbWritten: *mut ULONG,
) -> HRESULT {
    let mut This: *mut ISHRegStream = impl_from_IStream(iface);
    let mut newLen: DWORD = (*This).dwPos.wrapping_add(cb as DWORD);
    WINE_TRACE!("(%p, %p, %d, %p)\n", This, pv, cb, pcbWritten);
    if newLen < (*This).dwPos {
        return STG_E_INSUFFICIENTMEMORY;
    }
    if newLen > (*This).dwLength {
        let mut newBuf: LPBYTE = HeapReAlloc(
            GetProcessHeap(),
            HEAP_ZERO_MEMORY as DWORD,
            (*This).pbBuffer as LPVOID,
            newLen as SIZE_T,
        ) as LPBYTE;
        if newBuf.is_null() {
            return STG_E_INSUFFICIENTMEMORY;
        }
        (*This).dwLength = newLen;
        (*This).pbBuffer = newBuf;
    }
    memmove(
        (*This).pbBuffer.offset((*This).dwPos as isize) as *mut core::ffi::c_void,
        pv,
        cb as usize,
    );
    (*This).dwPos = (*This).dwPos.wrapping_add(cb as DWORD);
    if !pcbWritten.is_null() {
        *pcbWritten = cb;
    }
    return S_OK;
}

unsafe extern "system" fn IStream_fnSeek(iface: *mut IStream, dlibMove: LARGE_INTEGER, dwOrigin: DWORD, plibNewPosition: *mut ULARGE_INTEGER) -> HRESULT {
    let mut This: *mut ISHRegStream = impl_from_IStream(iface);
    let mut tmp: LARGE_INTEGER = _LARGE_INTEGER {
        __bindgen_anon_1: _LARGE_INTEGER__bindgen_ty_1 { LowPart: 0,
            HighPart: 0,
        },
    };
    WINE_TRACE!("(%p, %s, %d %p)\n", This, wine_dbgstr_longlong(dlibMove.QuadPart as ULONGLONG), dwOrigin, plibNewPosition);
    if dwOrigin == STREAM_SEEK_SET {
        tmp = dlibMove;
    } else if dwOrigin == STREAM_SEEK_CUR {
        tmp.QuadPart = (*This).dwPos as LONGLONG + dlibMove.QuadPart;
    } else if dwOrigin == STREAM_SEEK_END {
        tmp.QuadPart = (*This).dwLength as LONGLONG + dlibMove.QuadPart;
    } else {
        return STG_E_INVALIDPARAMETER;
    }
    if tmp.QuadPart < 0 {
        return STG_E_INVALIDFUNCTION;
    }
    (*This).dwPos = tmp.u.LowPart;
    if !plibNewPosition.is_null() {
        (*plibNewPosition).QuadPart = (*This).dwPos as ULONGLONG;
    }
    return S_OK;
}

unsafe extern "system" fn IStream_fnSetSize(mut iface: *mut IStream, mut libNewSize: ULARGE_INTEGER) -> HRESULT {
    let mut This: *mut ISHRegStream = impl_from_IStream(iface);
    let mut newLen: DWORD = 0;
    let mut newBuf: LPBYTE = core::ptr::null_mut();

    TRACE!("(%p, %s)\n", This, wine_dbgstr_longlong(libNewSize.QuadPart));

    /* we cut off the high part here */
    newLen = libNewSize.u.LowPart;
    newBuf = HeapReAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        (*This).pbBuffer as LPVOID,
        (newLen) as SIZE_T,
    ) as LPBYTE;
    if newBuf.is_null() {
        return STG_E_INSUFFICIENTMEMORY;
    }

    (*This).pbBuffer = newBuf;
    (*This).dwLength = newLen;

    return S_OK;
}

unsafe extern "system" fn IStream_fnCopyTo(iface: *mut IStream, pstm: *mut IStream, cb: ULARGE_INTEGER, pcbRead: *mut ULARGE_INTEGER, pcbWritten: *mut ULARGE_INTEGER) -> HRESULT {
    let mut This: *mut ISHRegStream = impl_from_IStream(iface);
    if !pcbRead.is_null() {
        (*pcbRead).QuadPart = 0;
    }
    if !pcbWritten.is_null() {
        (*pcbWritten).QuadPart = 0;
    }
    return E_NOTIMPL;
}

unsafe extern "system" fn IStream_fnCommit(iface: *mut IStream, grfCommitFlags: DWORD) -> HRESULT {
    let This: *mut ISHRegStream = impl_from_IStream(iface);
    let _ = This;
    let _ = grfCommitFlags;

    /* commit not supported by this stream */
    return E_NOTIMPL;
}

unsafe extern "system" fn IStream_fnRevert(iface: *mut IStream) -> HRESULT {
    let mut This: *mut ISHRegStream = impl_from_IStream(iface);

    /* revert not supported by this stream */
    return E_NOTIMPL;
}

unsafe extern "system" fn IStream_fnLockUnlockRegion(iface: *mut IStream, libOffset: ULARGE_INTEGER, cb: ULARGE_INTEGER, dwLockType: DWORD) -> HRESULT {
    let mut This: *mut ISHRegStream = impl_from_IStream(iface);

    /* lock/unlock not supported by this stream */
    return E_NOTIMPL;
}
unsafe extern "system" fn IStream_fnStat(iface: *mut IStream, pstatstg: *mut STATSTG, grfStatFlag: DWORD) -> HRESULT {
    let This: *mut ISHRegStream = impl_from_IStream(iface);

    (*pstatstg).pwcsName = core::ptr::null_mut();
    (*pstatstg).type_ = STGTY_STREAM as DWORD;
    (*pstatstg).cbSize.QuadPart = (*This).dwLength as ULONGLONG;
    (*pstatstg).mtime.dwHighDateTime = 0 as DWORD;
    (*pstatstg).mtime.dwLowDateTime = 0 as DWORD;
    (*pstatstg).ctime.dwHighDateTime = 0 as DWORD;
    (*pstatstg).ctime.dwLowDateTime = 0 as DWORD;
    (*pstatstg).atime.dwHighDateTime = 0 as DWORD;
    (*pstatstg).atime.dwLowDateTime = 0 as DWORD;
    (*pstatstg).grfMode = (*This).dwMode;
    (*pstatstg).grfLocksSupported = 0 as DWORD;
    (*pstatstg).clsid = CLSID_NULL;
    (*pstatstg).grfStateBits = 0 as DWORD;
    (*pstatstg).reserved = 0 as DWORD;

    return S_OK;
}

unsafe extern "system" fn IStream_fnClone(iface: *mut IStream, ppstm: *mut *mut IStream) -> HRESULT {
    let This: *mut ISHRegStream = impl_from_IStream(iface);

    *ppstm = core::ptr::null_mut();

    /* clone not supported by this stream */
    return E_NOTIMPL;
}

unsafe extern "system" fn IStream_fnAddRefDummy(iface: *mut IStream) -> ULONG {
    let mut This: *mut ISHRegStream = impl_from_IStream(iface);
    TRACE!("(%p)\n", This);
    return 2;
}

unsafe extern "system" fn IStream_fnReleaseDummy(iface: *mut IStream) -> ULONG {
    let This: *mut ISHRegStream = impl_from_IStream(iface);
    TRACE!("(%p)\n", This);
    return 1;
}

unsafe extern "system" fn IStream_fnReadDummy(iface: *mut IStream, pv: LPVOID, cb: ULONG, pcbRead: *mut ULONG) -> HRESULT {
    if !pcbRead.is_null() {
        *pcbRead = 0;
    }
    return E_NOTIMPL;
}

unsafe extern "C" fn IStream_Create(hKey: HKEY, pbBuffer: LPBYTE, dwLength: DWORD) -> *mut ISHRegStream {
    let mut regStream: *mut ISHRegStream = core::ptr::null_mut::<ISHRegStream>();

    regStream = HeapAlloc(GetProcessHeap(), 0 as DWORD, core::mem::size_of::<ISHRegStream>() as SIZE_T) as *mut ISHRegStream;

    if !regStream.is_null() {
        (*regStream).IStream_iface.lpVtbl = core::ptr::addr_of!(rstvt);
        (*regStream).ref_ = 1 as LONG;
        (*regStream).hKey = hKey;
        (*regStream).pbBuffer = pbBuffer;
        (*regStream).dwLength = dwLength;
        (*regStream).dwPos = 0 as DWORD;
        (*regStream).dwMode = STGM_READWRITE as DWORD;
        (*regStream).u.keyNameA = core::ptr::null_mut::<CHAR>();
        (*regStream).bUnicode = FALSE as BOOL;
    }

    return regStream;
}

#[no_mangle]
pub unsafe extern "system" fn SHOpenRegStream2A(
    hKey: HKEY,
    pszSubkey: LPCSTR,
    pszValue: LPCSTR,
    dwMode: DWORD,
) -> *mut IStream {
    let mut tmp: *mut ISHRegStream = core::ptr::null_mut();
    let mut hStrKey: HKEY = core::ptr::null_mut();
    let mut lpBuff: LPBYTE = core::ptr::null_mut();
    let mut dwLength: DWORD = 0;
    let mut ret: LONG;

    if dwMode == STGM_READ {
        ret = RegOpenKeyExA(
            hKey,
            pszSubkey,
            0,
            KEY_READ,
            core::ptr::addr_of_mut!(hStrKey),
        );
    } else {
        ret = RegCreateKeyExA(
            hKey,
            pszSubkey,
            0,
            core::ptr::null_mut(),
            0,
            KEY_READ | KEY_WRITE,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(hStrKey),
            core::ptr::null_mut(),
        );
    }

    if ret == (ERROR_SUCCESS as LONG) {
        if dwMode == STGM_READ || dwMode == STGM_READWRITE {
            /* read initial data */
            ret = RegQueryValueExA(
                hStrKey,
                pszValue,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(dwLength),
            );
            if ret == (ERROR_SUCCESS as LONG) && dwLength != 0 {
                lpBuff = HeapAlloc(
                    GetProcessHeap(),
                    0,
                    dwLength as SIZE_T,
                ) as LPBYTE;
                RegQueryValueExA(
                    hStrKey,
                    pszValue,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    lpBuff,
                    core::ptr::addr_of_mut!(dwLength),
                );
            }
        }

        if dwLength == 0 {
            lpBuff = HeapAlloc(
                GetProcessHeap(),
                0,
                dwLength as SIZE_T,
            ) as LPBYTE;
        }

        tmp = IStream_Create(hStrKey, lpBuff, dwLength);
        if !tmp.is_null() {
            if !pszValue.is_null() {
                let len: core::ffi::c_int = lstrlenA(pszValue) + 1;
                (*tmp).u.keyNameA = HeapAlloc(
                    GetProcessHeap(),
                    0,
                    len as SIZE_T,
                ) as LPSTR;
                memcpy(
                    (*tmp).u.keyNameA as *mut core::ffi::c_void,
                    pszValue as *const core::ffi::c_void,
                    len as usize,
                );
            }

            (*tmp).dwMode = dwMode;
            (*tmp).bUnicode = (FALSE as i32);
            return core::ptr::addr_of_mut!((*tmp).IStream_iface) as *mut IStream;
        }
    }

    HeapFree(
        GetProcessHeap(),
        0,
        lpBuff as PVOID,
    );
    if !hStrKey.is_null() {
        RegCloseKey(hStrKey);
    }
    return core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "system" fn SHOpenRegStream2W(
    hKey: HKEY,
    pszSubkey: LPCWSTR,
    pszValue: LPCWSTR,
    dwMode: DWORD,
) -> *mut IStream {
    let mut tmp: *mut ISHRegStream = core::ptr::null_mut();
    let mut hStrKey: HKEY = core::ptr::null_mut();
    let mut lpBuff: LPBYTE = core::ptr::null_mut();
    let mut dwLength: DWORD = 0;
    let mut ret: LONG;

    if dwMode == STGM_READ {
        ret = RegOpenKeyExW(hKey, pszSubkey, 0, KEY_READ, core::ptr::addr_of_mut!(hStrKey)) as LONG;
    } else {
        /* in write mode we make sure the subkey exits */
        ret = RegCreateKeyExW(hKey, pszSubkey, 0, core::ptr::null_mut(), 0, KEY_READ | KEY_WRITE, core::ptr::null_mut(), core::ptr::addr_of_mut!(hStrKey), core::ptr::null_mut()) as LONG;
    }

    if ret == ERROR_SUCCESS as LONG {
        if dwMode == STGM_READ || dwMode == STGM_READWRITE {
            /* read initial data */
            ret = RegQueryValueExW(hStrKey, pszValue, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::addr_of_mut!(dwLength)) as LONG;
            if ret == ERROR_SUCCESS as LONG && dwLength != 0 {
                lpBuff = HeapAlloc(GetProcessHeap(), 0, dwLength as SIZE_T) as LPBYTE;
                RegQueryValueExW(hStrKey, pszValue, core::ptr::null_mut(), core::ptr::null_mut(), lpBuff, core::ptr::addr_of_mut!(dwLength));
            }
        }

        if dwLength == 0 {
            lpBuff = HeapAlloc(GetProcessHeap(), 0, dwLength as SIZE_T) as LPBYTE;
        }

        tmp = IStream_Create(hStrKey, lpBuff, dwLength);
        if !tmp.is_null() {
            if !pszValue.is_null() {
                let mut len: i32 = lstrlenW(pszValue) + 1;
                (*tmp).u.keyNameW = HeapAlloc(GetProcessHeap(), 0, (len as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T)) as LPWSTR;
                memcpy((*tmp).u.keyNameW as *mut core::ffi::c_void, pszValue as *const core::ffi::c_void, ((len as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T)) as _);
            }

            (*tmp).dwMode = dwMode;
            (*tmp).bUnicode = (TRUE as i32);
            return core::ptr::addr_of_mut!((*tmp).IStream_iface);
        }
    }

    HeapFree(GetProcessHeap(), 0, lpBuff as LPVOID);
    if !hStrKey.is_null() {
        RegCloseKey(hStrKey);
    }
    return core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "system" fn SHOpenRegStreamA(
    hkey: HKEY,
    pszSubkey: LPCSTR,
    pszValue: LPCSTR,
    dwMode: DWORD,
) -> *mut IStream {
    let mut iStream: *mut IStream;

    iStream = SHOpenRegStream2A(hkey, pszSubkey, pszValue, dwMode);
    return if !iStream.is_null() {
        iStream
    } else {
        core::ptr::addr_of_mut!(rsDummyRegStream.IStream_iface)
    };
}

#[no_mangle]
pub unsafe extern "system" fn SHOpenRegStreamW(
    mut hkey: HKEY,
    mut pszSubkey: LPCWSTR,
    mut pszValue: LPCWSTR,
    mut dwMode: DWORD,
) -> *mut IStream {
    let mut iStream: *mut IStream = core::ptr::null_mut::<IStream>();
    WINE_TRACE!("(%p,%s,%s,0x%08x)\n", hkey, debugstr_w(pszSubkey as *const WCHAR), debugstr_w(pszValue as *const WCHAR), dwMode);
    iStream = SHOpenRegStream2W(hkey, pszSubkey, pszValue, dwMode) as *mut IStream;
    return if !iStream.is_null() {
        iStream as *mut IStream
    } else {
        core::ptr::addr_of_mut!(rsDummyRegStream.IStream_iface)
    };
}

#[no_mangle]
pub unsafe extern "system" fn SHCreateMemStream(lpbData: *const BYTE, mut dwDataLen: UINT) -> *mut IStream {
    let mut strm: *mut ISHRegStream = core::ptr::null_mut();
    let mut lpbDup: LPBYTE;

    TRACE!("(%p,%d)\n", lpbData, dwDataLen);

    if lpbData.is_null() {
        dwDataLen = 0;
    }

    lpbDup = HeapAlloc(GetProcessHeap(), 0, dwDataLen as SIZE_T) as LPBYTE;

    if !lpbDup.is_null() {
        memcpy(lpbDup as *mut core::ffi::c_void, lpbData as *const core::ffi::c_void, dwDataLen as usize);
        strm = IStream_Create(core::ptr::null_mut(), lpbDup, dwDataLen as DWORD);

        if strm.is_null() {
            HeapFree(GetProcessHeap(), 0, lpbDup as LPVOID);
        }
    }
    if strm.is_null() {
        return core::ptr::null_mut();
    }
    return core::ptr::addr_of_mut!((*strm).IStream_iface) as *mut IStream;
}

#[no_mangle]
pub unsafe extern "system" fn SHCreateStreamWrapper(
    mut lpbData: LPBYTE,
    mut dwDataLen: DWORD,
    mut dwReserved: DWORD,
    mut lppStream: *mut *mut IStream,
) -> HRESULT {
    let mut strm: *mut ISHRegStream = core::ptr::null_mut();

    if !lppStream.is_null() {
        *lppStream = core::ptr::null_mut();
    }

    if dwReserved != 0 || lppStream.is_null() {
        return E_INVALIDARG;
    }

    strm = IStream_Create(core::ptr::null_mut(), lpbData, dwDataLen);

    if strm.is_null() {
        return E_OUTOFMEMORY;
    }

    IStream_QueryInterface(
        &raw mut (*strm).IStream_iface,
        &raw const IID_IStream,
        lppStream as *mut *mut core::ffi::c_void,
    );
    IStream_Release(&raw mut (*strm).IStream_iface);
    return S_OK;
}

// COM vtable tables — generated from regstream.c by vtbl_gen.py
/// regstream.c:354 `static const IStreamVtbl rstvt`  (slot order: com-sys/bindgen_i386.rs)
pub static rstvt: IStreamVtbl = IStreamVtbl {
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

/// regstream.c:404 `static const IStreamVtbl DummyRegStreamVTable`  (slot order: com-sys/bindgen_i386.rs)
pub static DummyRegStreamVTable: IStreamVtbl = IStreamVtbl {
    QueryInterface: Some(IStream_fnQueryInterface),
    AddRef: Some(IStream_fnAddRefDummy),
    Release: Some(IStream_fnReleaseDummy),
    Read: Some(IStream_fnReadDummy),
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
