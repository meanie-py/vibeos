use crate::prelude::*;
use core::ffi::c_void;

extern "C" {

    fn wcschr(s: *const u16, c: u16) -> *const u16;
    fn WideCharToMultiByte( CodePage: u32, dwFlags: u32, lpWideCharStr: *const u16, cchWideChar: i32, lpMultiByteStr: *mut i8, cbMultiByte: i32, lpDefaultChar: *const i8, lpUsedDefaultChar: *mut i32, ) -> i32;
    fn memcpy(dest: *mut core::ffi::c_void, src: *const core::ffi::c_void, n: usize) -> *mut core::ffi::c_void;
    fn towupper(c: i32) -> i32;
}
extern "C" {
    fn StgOpenStorage( pwcsName: LPCWSTR, pstgPriority: *mut IStorage, grfMode: LPDWORD, snbExclude: *mut core::ffi::c_void, reserved: LPDWORD, ppstgOpen: *mut *mut IStorage, ) -> HRESULT;
    fn CoTaskMemFree(pv: *mut core::ffi::c_void);
    fn CreateAntiMoniker(ppmk: *mut *mut IMoniker) -> HRESULT;
    fn CoCreateInstance( rclsid: *const CLSID, pUnkOuter: *mut IUnknown, dwClsContext: u32, riid: *const CLSID, ppv: *mut *mut core::ffi::c_void, ) -> HRESULT;
    fn GetClassFile(szFilename: LPCWSTR, pclsid: *mut CLSID) -> HRESULT;
    fn CoTaskMemAlloc(cb: usize) -> *mut core::ffi::c_void;
    fn CreateBindCtx(reserved: u32, ppbc: *mut *mut IBindCtx) -> HRESULT;
    fn CreateGenericComposite( pmkFirst: *mut IMoniker, pmkRest: *mut IMoniker, ppmkComposite: *mut *mut IMoniker, ) -> HRESULT;
    fn lstrcatW(lpString1: LPWSTR, lpString2: LPCWSTR) -> LPWSTR;
    fn wcscmp(string1: *const u16, string2: *const u16) -> i32;
    fn GetFullPathNameW(lpFileName: LPCWSTR, nBufferLength: DWORD, lpBuffer: LPWSTR, lpFilePart: *mut LPWSTR) -> DWORD;
    fn GetFileAttributesW(lpFileName: LPCWSTR) -> DWORD;
}

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct FileMonikerImpl {
    pub IMoniker_iface: IMoniker,
    pub IROTData_iface: IROTData,
    pub ref_: LONG,
    pub filePathName: LPOLESTR,
    pub pMarshal: *mut IUnknown,
}

pub type PFileMonikerImpl = *mut FileMonikerImpl;

pub static VT_FileMonikerImpl: IMonikerVtbl = IMonikerVtbl {
    QueryInterface: FileMonikerImpl_QueryInterface,
    AddRef: FileMonikerImpl_AddRef,
    Release: FileMonikerImpl_Release,
    GetClassID: FileMonikerImpl_GetClassID,
    IsDirty: FileMonikerImpl_IsDirty,
    Load: FileMonikerImpl_Load,
    Save: FileMonikerImpl_Save,
    GetSizeMax: FileMonikerImpl_GetSizeMax,
    BindToObject: FileMonikerImpl_BindToObject,
    BindToStorage: FileMonikerImpl_BindToStorage,
    Reduce: FileMonikerImpl_Reduce,
    ComposeWith: FileMonikerImpl_ComposeWith,
    Enum: FileMonikerImpl_Enum,
    IsEqual: FileMonikerImpl_IsEqual,
    Hash: FileMonikerImpl_Hash,
    IsRunning: FileMonikerImpl_IsRunning,
    GetTimeOfLastChange: FileMonikerImpl_GetTimeOfLastChange,
    Inverse: FileMonikerImpl_Inverse,
    CommonPrefixWith: FileMonikerImpl_CommonPrefixWith,
    RelativePathTo: FileMonikerImpl_RelativePathTo,
    GetDisplayName: FileMonikerImpl_GetDisplayName,
    ParseDisplayName: FileMonikerImpl_ParseDisplayName,
    IsSystemMoniker: FileMonikerImpl_IsSystemMoniker,
};

pub static VT_ROTDataImpl: IROTDataVtbl = IROTDataVtbl {
    QueryInterface: FileMonikerROTDataImpl_QueryInterface,
    AddRef: FileMonikerROTDataImpl_AddRef,
    Release: FileMonikerROTDataImpl_Release,
    GetComparisonData: FileMonikerROTDataImpl_GetComparisonData,
};

unsafe fn impl_from_IMoniker(iface: *mut IMoniker) -> *mut FileMonikerImpl {
    (iface as *mut u8).offset(-(core::mem::offset_of!(FileMonikerImpl, IMoniker_iface) as isize)) as *mut FileMonikerImpl
}

unsafe fn impl_from_IROTData(iface: *mut IROTData) -> *mut FileMonikerImpl {
    (iface as *mut u8).offset(-(core::mem::offset_of!(FileMonikerImpl, IROTData_iface) as isize)) as *mut FileMonikerImpl
}
unsafe fn FileMonikerImpl_QueryInterface(
    iface: *mut IMoniker,
    riid: *const CLSID,
    ppvObject: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let This = impl_from_IMoniker(iface);

    TRACE!(
        b"(%p,%s,%p)\n\0".as_ptr() as *const i8,
        This,
        debugstr_guid(riid),
        ppvObject,
    );

    if ppvObject.is_null() {
        return 0x80070057u32 as HRESULT; // E_INVALIDARG
    }

    *ppvObject = core::ptr::null_mut();

    if IsEqualIID(core::ptr::addr_of!(IID_IUnknown), riid) != 0
        || IsEqualIID(core::ptr::addr_of!(IID_IPersist), riid) != 0
        || IsEqualIID(core::ptr::addr_of!(IID_IPersistStream), riid) != 0
        || IsEqualIID(core::ptr::addr_of!(IID_IMoniker), riid) != 0
    {
        *ppvObject = iface as *mut core::ffi::c_void;
    } else if IsEqualIID(core::ptr::addr_of!(IID_IROTData), riid) != 0 {
        *ppvObject = core::ptr::addr_of_mut!((*This).IROTData_iface) as *mut core::ffi::c_void;
    } else if IsEqualIID(core::ptr::addr_of!(IID_IMarshal), riid) != 0 {
        let mut hr: HRESULT = 0; // S_OK
        if (*This).pMarshal.is_null() {
            hr = MonikerMarshal_Create(iface, core::ptr::addr_of_mut!((*This).pMarshal));
        }
        if hr != 0 {
            return hr;
        }
        return IUnknown_QueryInterface((*This).pMarshal, riid, ppvObject);
    }

    if (*ppvObject).is_null() {
        return 0x80004002u32 as HRESULT; // E_NOINTERFACE
    }

    IMoniker_AddRef(iface);

    0 // S_OK
}

unsafe fn FileMonikerImpl_AddRef(iface: *mut IMoniker) -> u32 {
    let This = impl_from_IMoniker(iface);

    InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref_)) as u32
}

unsafe fn FileMonikerImpl_Release(iface: *mut IMoniker) -> u32 {
    let This = impl_from_IMoniker(iface);
    let ref_count = InterlockedDecrement(&mut (*This).ref_ as *mut i32) as u32;

    if ref_count == 0 {
        FileMonikerImpl_Destroy(This);
    }

    ref_count
}

unsafe fn FileMonikerImpl_GetClassID(iface: *mut IMoniker, pClassID: *mut CLSID) -> HRESULT {
    TRACE!(b"(%p,%p)\n\0".as_ptr() as *const i8, iface, pClassID);

    if pClassID.is_null() {
        return E_POINTER;
    }

    *pClassID = CLSID_FileMoniker;

    S_OK
}

unsafe fn FileMonikerImpl_IsDirty(iface: *mut IMoniker) -> HRESULT {
    TRACE!(b"(%p)\n\0".as_ptr() as *const i8, iface);
    S_FALSE
}

unsafe fn FileMonikerImpl_Load(iface: *mut IMoniker, pStm: *mut IStream) -> HRESULT {
    let This = impl_from_IMoniker(iface);
    let mut res: HRESULT;
    let mut filePathA: *mut i8 = core::ptr::null_mut();
    let mut filePathW: *mut u16 = core::ptr::null_mut();
    let mut bread: u32;
    let mut wbuffer: u16;
    let mut dwbuffer: u32;
    let mut bytesA: u32;
    let mut bytesW: u32;
    let mut len: u32;
    let mut i: i32;

    TRACE!(b"(%p,%p)\n\0".as_ptr() as *const i8, iface, pStm);

    res = IStream_Read(pStm, &mut wbuffer as *mut _ as *mut core::ffi::c_void, core::mem::size_of::<u16>() as u32, &mut bread);
    if bread != core::mem::size_of::<u16>() as u32 {
        WARN!(b"Couldn't read 0 word\n\0".as_ptr() as *const i8);
        'fail: {
            HeapFree(GetProcessHeap(), 0, filePathA as *mut core::ffi::c_void);
            HeapFree(GetProcessHeap(), 0, filePathW as *mut core::ffi::c_void);

            if SUCCEEDED(res) != 0 {
                res = 0x80004005u32 as i32;
            }
            return res;
        }
    }

    res = IStream_Read(pStm, &mut bytesA as *mut _ as *mut core::ffi::c_void, core::mem::size_of::<u32>() as u32, &mut bread);
    if bread != core::mem::size_of::<u32>() as u32 {
        WARN!(b"Couldn't read file string length\n\0".as_ptr() as *const i8);
        'fail: {
            HeapFree(GetProcessHeap(), 0, filePathA as *mut core::ffi::c_void);
            HeapFree(GetProcessHeap(), 0, filePathW as *mut core::ffi::c_void);

            if SUCCEEDED(res) != 0 {
                res = 0x80004005u32 as i32;
            }
            return res;
        }
    }

    filePathA = HeapAlloc(GetProcessHeap(), 0, bytesA as usize) as *mut i8;
    if filePathA.is_null() {
        res = 0x8007000Eu32 as i32;
        'fail: {
            HeapFree(GetProcessHeap(), 0, filePathA as *mut core::ffi::c_void);
            HeapFree(GetProcessHeap(), 0, filePathW as *mut core::ffi::c_void);

            if SUCCEEDED(res) != 0 {
                res = 0x80004005u32 as i32;
            }
            return res;
        }
    }

    res = IStream_Read(pStm, filePathA as *mut core::ffi::c_void, bytesA, &mut bread);
    if bread != bytesA {
        WARN!(b"Couldn't read file path string\n\0".as_ptr() as *const i8);
        'fail: {
            HeapFree(GetProcessHeap(), 0, filePathA as *mut core::ffi::c_void);
            HeapFree(GetProcessHeap(), 0, filePathW as *mut core::ffi::c_void);

            if SUCCEEDED(res) != 0 {
                res = 0x80004005u32 as i32;
            }
            return res;
        }
    }

    IStream_Read(pStm, &mut wbuffer as *mut _ as *mut core::ffi::c_void, core::mem::size_of::<u16>() as u32, &mut bread);
    if bread != core::mem::size_of::<u16>() as u32 {
        WARN!(b"Couldn't read unknown value\n\0".as_ptr() as *const i8);
        'fail: {
            HeapFree(GetProcessHeap(), 0, filePathA as *mut core::ffi::c_void);
            HeapFree(GetProcessHeap(), 0, filePathW as *mut core::ffi::c_void);

            if SUCCEEDED(res) != 0 {
                res = 0x80004005u32 as i32;
            }
            return res;
        }
    }

    IStream_Read(pStm, &mut wbuffer as *mut _ as *mut core::ffi::c_void, core::mem::size_of::<u16>() as u32, &mut bread);
    if bread != core::mem::size_of::<u16>() as u32 {
        WARN!(b"Couldn't read DEAD constant\n\0".as_ptr() as *const i8);
        'fail: {
            HeapFree(GetProcessHeap(), 0, filePathA as *mut core::ffi::c_void);
            HeapFree(GetProcessHeap(), 0, filePathW as *mut core::ffi::c_void);

            if SUCCEEDED(res) != 0 {
                res = 0x80004005u32 as i32;
            }
            return res;
        }
    }

    i = 0;
    while i < 5 {
        res = IStream_Read(pStm, &mut dwbuffer as *mut _ as *mut core::ffi::c_void, core::mem::size_of::<u32>() as u32, &mut bread);
        if bread != core::mem::size_of::<u32>() as u32 {
            WARN!(b"Couldn't read 0 padding\n\0".as_ptr() as *const i8);
            'fail: {
                HeapFree(GetProcessHeap(), 0, filePathA as *mut core::ffi::c_void);
                HeapFree(GetProcessHeap(), 0, filePathW as *mut core::ffi::c_void);

                if SUCCEEDED(res) != 0 {
                    res = 0x80004005u32 as i32;
                }
                return res;
            }
        }
        i += 1;
    }

    res = IStream_Read(pStm, &mut dwbuffer as *mut _ as *mut core::ffi::c_void, core::mem::size_of::<u32>() as u32, &mut bread);
    if bread != core::mem::size_of::<u32>() as u32 {
        'fail: {
            HeapFree(GetProcessHeap(), 0, filePathA as *mut core::ffi::c_void);
            HeapFree(GetProcessHeap(), 0, filePathW as *mut core::ffi::c_void);

            if SUCCEEDED(res) != 0 {
                res = 0x80004005u32 as i32;
            }
            return res;
        }
    }

    if dwbuffer == 0 {
        bytesA -= 1;
        len = MultiByteToWideChar(0, 8, filePathA, bytesA as i32, core::ptr::null_mut(), 0) as u32;
        if len == 0 {
            'fail: {
                HeapFree(GetProcessHeap(), 0, filePathA as *mut core::ffi::c_void);
                HeapFree(GetProcessHeap(), 0, filePathW as *mut core::ffi::c_void);

                if SUCCEEDED(res) != 0 {
                    res = 0x80004005u32 as i32;
                }
                return res;
            }
        }

        filePathW = HeapAlloc(GetProcessHeap(), 0, ((len + 1) * core::mem::size_of::<u16>() as u32) as usize) as *mut u16;
        if filePathW.is_null() {
            res = 0x8007000Eu32 as i32;
            'fail: {
                HeapFree(GetProcessHeap(), 0, filePathA as *mut core::ffi::c_void);
                HeapFree(GetProcessHeap(), 0, filePathW as *mut core::ffi::c_void);

                if SUCCEEDED(res) != 0 {
                    res = 0x80004005u32 as i32;
                }
                return res;
            }
        }
        MultiByteToWideChar(0, 8, filePathA, -1, filePathW, (len + 1) as i32);
        'succeed: {
            HeapFree(GetProcessHeap(), 0, filePathA as *mut core::ffi::c_void);
            HeapFree(GetProcessHeap(), 0, (*This).filePathName as *mut core::ffi::c_void);
            (*This).filePathName = filePathW;

            return 0;
        }
    }

    if dwbuffer < 6 {
        'fail: {
            HeapFree(GetProcessHeap(), 0, filePathA as *mut core::ffi::c_void);
            HeapFree(GetProcessHeap(), 0, filePathW as *mut core::ffi::c_void);

            if SUCCEEDED(res) != 0 {
                res = 0x80004005u32 as i32;
            }
            return res;
        }
    }

    bytesW = dwbuffer - 6;

    res = IStream_Read(pStm, &mut dwbuffer as *mut _ as *mut core::ffi::c_void, core::mem::size_of::<u32>() as u32, &mut bread);
    if bread != core::mem::size_of::<u32>() as u32 || dwbuffer != bytesW {
        'fail: {
            HeapFree(GetProcessHeap(), 0, filePathA as *mut core::ffi::c_void);
            HeapFree(GetProcessHeap(), 0, filePathW as *mut core::ffi::c_void);

            if SUCCEEDED(res) != 0 {
                res = 0x80004005u32 as i32;
            }
            return res;
        }
    }

    res = IStream_Read(pStm, &mut wbuffer as *mut _ as *mut core::ffi::c_void, core::mem::size_of::<u16>() as u32, &mut bread);
    if bread != core::mem::size_of::<u16>() as u32 || wbuffer != 0x3 {
        'fail: {
            HeapFree(GetProcessHeap(), 0, filePathA as *mut core::ffi::c_void);
            HeapFree(GetProcessHeap(), 0, filePathW as *mut core::ffi::c_void);

            if SUCCEEDED(res) != 0 {
                res = 0x80004005u32 as i32;
            }
            return res;
        }
    }

    len = bytesW / core::mem::size_of::<u16>() as u32;
    filePathW = HeapAlloc(GetProcessHeap(), 0, ((len + 1) * core::mem::size_of::<u16>() as u32) as usize) as *mut u16;
    if filePathW.is_null() {
        res = 0x8007000Eu32 as i32;
        'fail: {
            HeapFree(GetProcessHeap(), 0, filePathA as *mut core::ffi::c_void);
            HeapFree(GetProcessHeap(), 0, filePathW as *mut core::ffi::c_void);

            if SUCCEEDED(res) != 0 {
                res = 0x80004005u32 as i32;
            }
            return res;
        }
    }
    res = IStream_Read(pStm, filePathW as *mut core::ffi::c_void, bytesW, &mut bread);
    if bread != bytesW {
        'fail: {
            HeapFree(GetProcessHeap(), 0, filePathA as *mut core::ffi::c_void);
            HeapFree(GetProcessHeap(), 0, filePathW as *mut core::ffi::c_void);

            if SUCCEEDED(res) != 0 {
                res = 0x80004005u32 as i32;
            }
            return res;
        }
    }

    *filePathW.add(len as usize) = 0;

    HeapFree(GetProcessHeap(), 0, filePathA as *mut core::ffi::c_void);
    HeapFree(GetProcessHeap(), 0, (*This).filePathName as *mut core::ffi::c_void);
    (*This).filePathName = filePathW;

    return 0;
}

unsafe fn FileMonikerImpl_Save(iface: *mut IMoniker, pStm: *mut IStream, fClearDirty: i32) -> HRESULT {
    let This = impl_from_IMoniker(iface);
    let mut res: HRESULT;
    let filePathW = (*This).filePathName;
    let mut filePathA: *mut i8;
    let mut bytesA: u32;
    let mut bytesW: u32;
    let len: u32;

    static FFFF: u16 = 0xFFFF;
    static DEAD: u16 = 0xDEAD;
    static ZERO: u32 = 0;
    static THREE: u16 = 0x3;

    let mut i: i32;
    let mut bUsedDefault: i32 = 0;
    let bWriteWide: i32;

    TRACE!(b"(%p,%p,%d)\n\0".as_ptr() as *const i8, iface, pStm, fClearDirty);

    if pStm.is_null() {
        return 0x80004003u32 as HRESULT;
    }

    res = IStream_Write(pStm, &ZERO as *const u32 as *const core::ffi::c_void, core::mem::size_of::<u16>() as u32, core::ptr::null_mut());
    if FAILED(res) { return res; }

    bytesA = WideCharToMultiByte(0, 0, filePathW, -1, core::ptr::null_mut(), 0, core::ptr::null(), core::ptr::null_mut()) as u32;
    res = IStream_Write(pStm, &bytesA as *const u32 as *const core::ffi::c_void, core::mem::size_of::<u32>() as u32, core::ptr::null_mut());
    if FAILED(res) { return res; }

    filePathA = HeapAlloc(GetProcessHeap(), 0, bytesA as usize) as *mut i8;
    if filePathA.is_null() {
        return 0x8007000Eu32 as HRESULT;
    }
    WideCharToMultiByte(0, 0, filePathW, -1, filePathA, bytesA as i32, core::ptr::null(), &mut bUsedDefault);
    res = IStream_Write(pStm, filePathA as *const core::ffi::c_void, bytesA, core::ptr::null_mut());
    HeapFree(GetProcessHeap(), 0, filePathA as *mut core::ffi::c_void);
    if FAILED(res) { return res; }

    res = IStream_Write(pStm, &FFFF as *const u16 as *const core::ffi::c_void, core::mem::size_of::<u16>() as u32, core::ptr::null_mut());
    if FAILED(res) { return res; }

    res = IStream_Write(pStm, &DEAD as *const u16 as *const core::ffi::c_void, core::mem::size_of::<u16>() as u32, core::ptr::null_mut());
    if FAILED(res) { return res; }

    i = 0;
    while i < 5 {
        res = IStream_Write(pStm, &ZERO as *const u32 as *const core::ffi::c_void, core::mem::size_of::<u32>() as u32, core::ptr::null_mut());
        if FAILED(res) { return res; }
        i += 1;
    }

    len = lstrlenW(filePathW) as u32;
    bWriteWide = if bUsedDefault != 0 || (len > 0 && *filePathW.offset((len - 1) as isize) == '\\' as u16) {
        1
    } else {
        0
    };

    if bWriteWide == 0 {
        let mut pch = filePathW;
        while *pch != 0 {
            if *pch > 0xFF {
                return IStream_Write(pStm, &ZERO as *const u32 as *const core::ffi::c_void, core::mem::size_of::<u32>() as u32, core::ptr::null_mut());
            }
            pch = pch.offset(1);
        }
    }

    if bWriteWide == 0 {
        return IStream_Write(pStm, &ZERO as *const u32 as *const core::ffi::c_void, core::mem::size_of::<u32>() as u32, core::ptr::null_mut());
    }

    bytesW = len * core::mem::size_of::<u16>() as u32 + 6;
    res = IStream_Write(pStm, &bytesW as *const u32 as *const core::ffi::c_void, core::mem::size_of::<u32>() as u32, core::ptr::null_mut());
    if FAILED(res) { return res; }

    bytesW -= 6;
    res = IStream_Write(pStm, &bytesW as *const u32 as *const core::ffi::c_void, core::mem::size_of::<u32>() as u32, core::ptr::null_mut());
    if FAILED(res) { return res; }

    res = IStream_Write(pStm, &THREE as *const u16 as *const core::ffi::c_void, core::mem::size_of::<u16>() as u32, core::ptr::null_mut());
    if FAILED(res) { return res; }

    IStream_Write(pStm, filePathW as *const core::ffi::c_void, bytesW, core::ptr::null_mut())
}

unsafe fn FileMonikerImpl_GetSizeMax(iface: *mut IMoniker, pcbSize: *mut ULARGE_INTEGER) -> HRESULT {
    let This = impl_from_IMoniker(iface);

    if pcbSize.is_null() {
        return 0x80004003u32 as HRESULT;
    }

    (*pcbSize).u.__bindgen_anon_1.LowPart = 0x38u32.wrapping_add((4u32).wrapping_mul(lstrlenW((*This).filePathName) as u32));
    (*pcbSize).u.__bindgen_anon_1.HighPart = 0;

    0x00000000u32 as HRESULT
}
#[no_mangle]
pub unsafe extern "C" fn FileMonikerImpl_Destroy(This: *mut FileMonikerImpl) -> HRESULT {

    TRACE!(b"(%p)\n\0".as_ptr(), This);

    if !(*This).pMarshal.is_null() {
        IUnknown_Release((*This).pMarshal);
    }
    HeapFree(GetProcessHeap(), 0, (*This).filePathName as *mut core::ffi::c_void);
    HeapFree(GetProcessHeap(), 0, This as *mut core::ffi::c_void);

    0
}

unsafe fn FileMonikerImpl_BindToObject(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    riid: *const CLSID,
    ppvResult: *mut *mut core::ffi::c_void,
) -> HRESULT {

    let This = impl_from_IMoniker(iface);
    let mut res: HRESULT = 0x80004005u32 as HRESULT;
    let mut clsID: CLSID = core::mem::zeroed();
    let mut pObj: *mut IUnknown = core::ptr::null_mut();
    let mut prot: *mut IRunningObjectTable = core::ptr::null_mut();
    let mut ppf: *mut IPersistFile = core::ptr::null_mut();
    let mut pcf: *mut IClassFactory = core::ptr::null_mut();
    let mut pca: *mut IClassActivator = core::ptr::null_mut();

    *ppvResult = core::ptr::null_mut();

    TRACE!(
        b"(%p,%p,%p,%s,%p)\n\0".as_ptr(),
        iface,
        pbc,
        pmkToLeft,
        debugstr_guid(riid),
        ppvResult,
    );

    if pmkToLeft.is_null() {
        res = IBindCtx_GetRunningObjectTable(pbc, &mut prot);

        if SUCCEEDED(res) != 0 {
            res = IRunningObjectTable_GetObject(prot, iface, &mut pObj);

            if res != 0 {
                res = GetClassFile((*This).filePathName, &mut clsID);
                if SUCCEEDED(res) != 0 {
                    res = CoCreateInstance(
                        &clsID,
                        core::ptr::null_mut(),
                        0x00000015u32,
                        &IID_IPersistFile,
                        &mut ppf as *mut *mut IPersistFile as *mut *mut core::ffi::c_void,
                    );
                    if SUCCEEDED(res) != 0 {
                        res = IPersistFile_Load(ppf, (*This).filePathName, 0x00000000u32);
                        if SUCCEEDED(res) != 0 {
                            pObj = ppf as *mut IUnknown;
                            IUnknown_AddRef(pObj);
                        }
                    }
                }
            }
        }
    } else {
        res = IMoniker_BindToObject(
            pmkToLeft,
            pbc,
            core::ptr::null_mut(),
            &IID_IClassFactory,
            &mut pcf as *mut *mut IClassFactory as *mut *mut core::ffi::c_void,
        );

        if res == 0x80004002u32 as HRESULT {
            res = IMoniker_BindToObject(
                pmkToLeft,
                pbc,
                core::ptr::null_mut(),
                &IID_IClassActivator,
                &mut pca as *mut *mut IClassActivator as *mut *mut core::ffi::c_void,
            );

            if res == 0x80004002u32 as HRESULT {
                return 0x80004009u32 as HRESULT;
            }
        }
        if !pcf.is_null() {
            IClassFactory_CreateInstance(
                pcf,
                core::ptr::null_mut(),
                &IID_IPersistFile,
                &mut ppf as *mut *mut IPersistFile as *mut *mut core::ffi::c_void,
            );

            res = IPersistFile_Load(ppf, (*This).filePathName, 0x00000000u32);

            if SUCCEEDED(res) != 0 {
                pObj = ppf as *mut IUnknown;
                IUnknown_AddRef(pObj);
            }
        }
        if !pca.is_null() {
            FIXME!(b"()\n\0".as_ptr());
        }
    }

    if !pObj.is_null() {
        res = IUnknown_QueryInterface(pObj, riid, ppvResult);

        IBindCtx_RegisterObjectBound(pbc, *ppvResult as *mut IUnknown);

        IUnknown_Release(pObj);
    }

    if !prot.is_null() {
        IRunningObjectTable_Release(prot);
    }

    if !ppf.is_null() {
        IPersistFile_Release(ppf);
    }

    if !pca.is_null() {
        IClassActivator_Release(pca);
    }

    if !pcf.is_null() {
        IClassFactory_Release(pcf);
    }

    res
}
unsafe fn FileMonikerImpl_BindToStorage(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    riid: *const CLSID,
    ppvObject: *mut *mut core::ffi::c_void,
) -> HRESULT {

    let mut filePath: LPWSTR = core::ptr::null_mut();
    let mut pstg: *mut IStorage = core::ptr::null_mut();
    let mut res: HRESULT = 0;

    TRACE!(
        b"(%p,%p,%p,%s,%p)\n\0".as_ptr() as *const core::ffi::c_char,
        iface,
        pbc,
        pmkToLeft,
        debugstr_guid(riid),
        ppvObject,
    );

    if pmkToLeft.is_null() {
        if IsEqualIID(&IID_IStorage, riid) != 0 {
            IMoniker_GetDisplayName(iface, pbc, pmkToLeft, &mut filePath);

            res = StgOpenStorage(
                filePath,
                core::ptr::null_mut(),
                STGM_READWRITE | STGM_SHARE_DENY_WRITE,
                core::ptr::null_mut(),
                0,
                &mut pstg,
            );

            if SUCCEEDED(res) != 0 {
                *ppvObject = pstg as *mut core::ffi::c_void;
            }

            CoTaskMemFree(filePath as *mut core::ffi::c_void);
        } else {
            if (IsEqualIID(&IID_IStream, riid) != 0) || (IsEqualIID(&IID_ILockBytes, riid) != 0) {
                return E_FAIL;
            } else {
                return E_NOINTERFACE;
            }
        }
    } else {
        FIXME!(
            b"(%p,%p,%p,%s,%p)\n\0".as_ptr() as *const core::ffi::c_char,
            iface,
            pbc,
            pmkToLeft,
            debugstr_guid(riid),
            ppvObject,
        );

        return E_NOTIMPL;
    }
    res
}

unsafe fn FileMonikerImpl_Reduce(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    dwReduceHowFar: DWORD,
    ppmkToLeft: *mut *mut IMoniker,
    ppmkReduced: *mut *mut IMoniker,
) -> HRESULT {
    TRACE!(
        b"(%p,%p,%d,%p,%p)\n\0".as_ptr() as *const i8,
        iface,
        pbc,
        dwReduceHowFar,
        ppmkToLeft,
        ppmkReduced,
    );

    if ppmkReduced.is_null() {
        return E_POINTER;
    }

    IMoniker_AddRef(iface);

    *ppmkReduced = iface;

    MK_S_REDUCED_TO_SELF
}

unsafe fn free_stringtable(stringTable: *mut LPOLESTR) {

    let mut i: i32 = 0;
    while !(*stringTable.offset(i as isize)).is_null() {
        CoTaskMemFree(*stringTable.offset(i as isize) as *mut core::ffi::c_void);
        i += 1;
    }
    CoTaskMemFree(stringTable as *mut core::ffi::c_void);
}
unsafe fn FileMonikerImpl_ComposeWith(
    iface: *mut IMoniker,
    pmkRight: *mut IMoniker,
    fOnlyIfNotGeneric: i32,
    ppmkComposite: *mut *mut IMoniker,
) -> HRESULT {
    let mut res: HRESULT;
    let mut str1: LPOLESTR = core::ptr::null_mut();
    let mut str2: LPOLESTR = core::ptr::null_mut();
    let mut strDec1: *mut LPOLESTR = core::ptr::null_mut();
    let mut strDec2: *mut LPOLESTR = core::ptr::null_mut();
    let mut newStr: LPOLESTR = core::ptr::null_mut();
    let twoPoint: [u16; 3] = [b'.' as u16, b'.' as u16, 0];
    let bkSlash: [u16; 2] = [b'\\' as u16, 0];
    let mut bind: *mut IBindCtx = core::ptr::null_mut();
    let mut i: i32 = 0;
    let mut j: i32 = 0;
    let mut lastIdx1: i32 = 0;
    let mut lastIdx2: i32 = 0;
    let mut mkSys: u32 = 0;

    TRACE!(
        b"(%p,%p,%d,%p)\n\0".as_ptr() as *const i8,
        iface,
        pmkRight,
        fOnlyIfNotGeneric,
        ppmkComposite,
    );

    if ppmkComposite.is_null() {
        return 0x80004003u32 as HRESULT;
    }

    if pmkRight.is_null() {
        return 0x80070057u32 as HRESULT;
    }

    *ppmkComposite = core::ptr::null_mut();

    IMoniker_IsSystemMoniker(pmkRight, &mut mkSys);

    if mkSys == 1 {
        CreateBindCtx(0, &mut bind);

        IMoniker_GetDisplayName(iface, bind, core::ptr::null_mut(), &mut str1);
        IMoniker_GetDisplayName(pmkRight, bind, core::ptr::null_mut(), &mut str2);

        lastIdx1 = FileMonikerImpl_DecomposePath(str1, &mut strDec1) - 1;
        lastIdx2 = FileMonikerImpl_DecomposePath(str2, &mut strDec2) - 1;

        if (lastIdx1 == -1 && lastIdx2 > -1)
            || (lastIdx1 == 1 && wcscmp(*strDec1.offset(0), twoPoint.as_ptr()) == 0)
        {
            res = 0x800401E4u32 as HRESULT;
        } else {
            if wcscmp(*strDec1.offset(lastIdx1 as isize), bkSlash.as_ptr()) == 0 {
                lastIdx1 -= 1;
            }

            i = 0;
            while lastIdx1 >= 0
                && !(*strDec2.offset(i as isize)).is_null()
                && wcscmp(*strDec2.offset(i as isize), twoPoint.as_ptr()) == 0
            {
                lastIdx1 -= 2;
                i += 2;
            }

            newStr = HeapAlloc(
                GetProcessHeap(),
                0,
                (core::mem::size_of::<u16>() as u32
                    * (lstrlenW(str1) + lstrlenW(str2) + 1) as u32) as usize,
            ) as LPOLESTR;

            if !newStr.is_null() {
                *newStr.offset(0) = 0;
                j = 0;
                while j <= lastIdx1 {
                    lstrcatW(newStr, *strDec1.offset(j as isize));
                    j += 1;
                }

                if ((*strDec2.offset(i as isize)).is_null() && lastIdx1 > -1 && lastIdx2 > -1)
                    || wcscmp(*strDec2.offset(i as isize), bkSlash.as_ptr()) != 0
                {
                    lstrcatW(newStr, bkSlash.as_ptr());
                }

                j = i;
                while j <= lastIdx2 {
                    lstrcatW(newStr, *strDec2.offset(j as isize));
                    j += 1;
                }

                res = CreateFileMoniker(newStr, ppmkComposite);

                HeapFree(GetProcessHeap(), 0, newStr as *mut _);
            } else {
                res = 0x8007000Eu32 as HRESULT;
            }
        }

        free_stringtable(strDec1);
        free_stringtable(strDec2);

        CoTaskMemFree(str1 as *mut _);
        CoTaskMemFree(str2 as *mut _);

        return res;
    } else if mkSys == 2 {
        *ppmkComposite = core::ptr::null_mut();
        return 0;
    } else if fOnlyIfNotGeneric != 0 {
        *ppmkComposite = core::ptr::null_mut();
        return 0x800401E2u32 as HRESULT;
    } else {
        return CreateGenericComposite(iface, pmkRight, ppmkComposite);
    }
}

type IBindCtx = core::ffi::c_void;
type LPOLESTR = *mut u16;

unsafe fn FileMonikerImpl_Enum(
    iface: *mut IMoniker,
    fForward: BOOL,
    ppenumMoniker: *mut *mut IEnumMoniker,
) -> HRESULT {
    TRACE!(
        b"(%p,%d,%p)\n\0".as_ptr() as PCHAR,
        iface,
        fForward,
        ppenumMoniker,
    );

    if ppenumMoniker.is_null() {
        return E_POINTER;
    }

    *ppenumMoniker = core::ptr::null_mut();

    return S_OK;
}
unsafe fn FileMonikerImpl_IsEqual(iface: *mut IMoniker, pmkOtherMoniker: *mut IMoniker) -> HRESULT {
    let This = impl_from_IMoniker(iface);
    let mut clsid: CLSID = core::mem::zeroed();
    let mut filePath: LPWSTR = core::ptr::null_mut();
    let mut bind: *mut IBindCtx = core::ptr::null_mut();
    let mut res: HRESULT;

    TRACE!(b"(%p,%p)\n\0".as_ptr() as LPCWSTR, iface, pmkOtherMoniker);

    if pmkOtherMoniker.is_null() {
        return S_FALSE;
    }

    IMoniker_GetClassID(pmkOtherMoniker, &mut clsid);

    if IsEqualCLSID(&clsid, &CLSID_FileMoniker) == 0 {
        return S_FALSE;
    }

    res = CreateBindCtx(0, &mut bind);
    if FAILED(res) != 0 {
        return res;
    }

    res = S_FALSE;
    if SUCCEEDED(IMoniker_GetDisplayName(pmkOtherMoniker, bind, core::ptr::null_mut(), &mut filePath)) != 0 {
        if lstrcmpiW(filePath, (*This).filePathName) == 0 {
            res = S_OK;
        }
        CoTaskMemFree(filePath as *mut core::ffi::c_void);
    }

    IBindCtx_Release(bind);
    res
}

unsafe fn FileMonikerImpl_Hash(iface: *mut IMoniker, pdwHash: LPDWORD) -> HRESULT {

    let This = impl_from_IMoniker(iface);
    let mut h: i32 = 0;
    let mut i: i32;
    let skip: i32;
    let len: i32;
    let mut off: i32 = 0;
    let val: LPWSTR;

    if pdwHash.is_null() {
        return E_POINTER;
    }

    val = (*This).filePathName;
    len = lstrlenW(val);

    if len < 16 {
        i = len;
        while i > 0 {
            h = h * 37 + (*val.offset(off as isize)) as i32;
            off += 1;
            i -= 1;
        }
    } else {
        skip = len / 8;
        i = len;
        while i > 0 {
            h = h * 39 + (*val.offset(off as isize)) as i32;
            i -= skip;
            off += skip;
        }
    }

    *pdwHash = h as u32;

    return S_OK;
}

unsafe fn FileMonikerImpl_IsRunning(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    pmkNewlyRunning: *mut IMoniker,
) -> HRESULT {
    let mut rot: *mut IRunningObjectTable = core::ptr::null_mut();
    let mut res: HRESULT;

    TRACE!(
        b"(%p,%p,%p,%p)\n\0".as_ptr() as *const i8,
        iface,
        pbc,
        pmkToLeft,
        pmkNewlyRunning,
    );

    if !pmkNewlyRunning.is_null() && IMoniker_IsEqual(pmkNewlyRunning, iface) == S_OK {
        return S_OK;
    }

    if pbc.is_null() {
        return E_POINTER;
    }

    res = IBindCtx_GetRunningObjectTable(pbc, &mut rot);

    if FAILED(res) != 0 {
        return res;
    }

    res = IRunningObjectTable_IsRunning(rot, iface);

    IRunningObjectTable_Release(rot);

    res
}
unsafe fn FileMonikerImpl_GetTimeOfLastChange(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    pFileTime: *mut FILETIME,
) -> HRESULT {
    let This = impl_from_IMoniker(iface);
    let mut rot: *mut IRunningObjectTable = core::ptr::null_mut();
    let mut res: HRESULT;
    let mut info: WIN32_FILE_ATTRIBUTE_DATA = core::mem::zeroed();

    TRACE!(
        b"(%p,%p,%p,%p)\n\0".as_ptr() as *const i8,
        iface,
        pbc,
        pmkToLeft,
        pFileTime,
    );

    if pFileTime.is_null() {
        return E_POINTER;
    }

    if !pmkToLeft.is_null() {
        return E_INVALIDARG;
    }

    res = IBindCtx_GetRunningObjectTable(pbc, &mut rot);

    if FAILED(res) != 0 {
        return res;
    }

    res = IRunningObjectTable_GetTimeOfLastChange(rot, iface, pFileTime);

    if FAILED(res) != 0 {
        if GetFileAttributesExW(
            (*This).filePathName,
            GetFileExInfoStandard,
            &mut info as *mut _ as *mut core::ffi::c_void,
        ) == 0
        {
            return MK_E_NOOBJECT;
        }

        *pFileTime = info.ftLastWriteTime;
    }

    S_OK
}

unsafe fn FileMonikerImpl_Inverse(iface: *mut IMoniker, ppmk: *mut *mut IMoniker) -> HRESULT {

    TRACE!(b"(%p,%p)\n\0".as_ptr(), iface, ppmk);

    CreateAntiMoniker(ppmk)
}

unsafe fn FileMonikerImpl_CommonPrefixWith(
    iface: *mut IMoniker,
    pmkOther: *mut IMoniker,
    ppmkPrefix: *mut *mut IMoniker,
) -> HRESULT {
    let mut pathThis: LPOLESTR = core::ptr::null_mut();
    let mut pathOther: LPOLESTR = core::ptr::null_mut();
    let mut stringTable1: *mut LPOLESTR = core::ptr::null_mut();
    let mut stringTable2: *mut LPOLESTR = core::ptr::null_mut();
    let mut commonPath: LPOLESTR = core::ptr::null_mut();
    let mut bindctx: *mut IBindCtx = core::ptr::null_mut();
    let mut mkSys: DWORD = 0;
    let mut nb1: ULONG;
    let mut nb2: ULONG;
    let mut i: ULONG;
    let mut sameIdx: ULONG;
    let mut machineNameCase: BOOL = 0;
    let mut ret: HRESULT;

    if ppmkPrefix.is_null() {
        return E_POINTER;
    }

    if pmkOther.is_null() {
        return E_INVALIDARG;
    }

    *ppmkPrefix = core::ptr::null_mut();

    IMoniker_IsSystemMoniker(pmkOther, &mut mkSys);
    if mkSys != MKSYS_FILEMONIKER {
        return MonikerCommonPrefixWith(iface, pmkOther, ppmkPrefix);
    }

    ret = CreateBindCtx(0, &mut bindctx);
    if FAILED(ret) {
        return ret;
    }

    ret = IMoniker_GetDisplayName(iface, bindctx, core::ptr::null_mut(), &mut pathThis);
    if FAILED(ret) {
        IBindCtx_Release(bindctx);
        CoTaskMemFree(pathThis as *mut _);
        CoTaskMemFree(pathOther as *mut _);
        CoTaskMemFree(commonPath as *mut _);
        if !stringTable1.is_null() {
            free_stringtable(stringTable1);
        }
        if !stringTable2.is_null() {
            free_stringtable(stringTable2);
        }
        return ret;
    }

    ret = IMoniker_GetDisplayName(pmkOther, bindctx, core::ptr::null_mut(), &mut pathOther);
    if FAILED(ret) {
        IBindCtx_Release(bindctx);
        CoTaskMemFree(pathThis as *mut _);
        CoTaskMemFree(pathOther as *mut _);
        CoTaskMemFree(commonPath as *mut _);
        if !stringTable1.is_null() {
            free_stringtable(stringTable1);
        }
        if !stringTable2.is_null() {
            free_stringtable(stringTable2);
        }
        return ret;
    }

    nb1 = FileMonikerImpl_DecomposePath(pathThis, &mut stringTable1);
    if FAILED(nb1 as HRESULT) {
        ret = nb1 as HRESULT;
        IBindCtx_Release(bindctx);
        CoTaskMemFree(pathThis as *mut _);
        CoTaskMemFree(pathOther as *mut _);
        CoTaskMemFree(commonPath as *mut _);
        if !stringTable1.is_null() {
            free_stringtable(stringTable1);
        }
        if !stringTable2.is_null() {
            free_stringtable(stringTable2);
        }
        return ret;
    }

    nb2 = FileMonikerImpl_DecomposePath(pathOther, &mut stringTable2);
    if FAILED(nb2 as HRESULT) {
        ret = nb2 as HRESULT;
        IBindCtx_Release(bindctx);
        CoTaskMemFree(pathThis as *mut _);
        CoTaskMemFree(pathOther as *mut _);
        CoTaskMemFree(commonPath as *mut _);
        if !stringTable1.is_null() {
            free_stringtable(stringTable1);
        }
        if !stringTable2.is_null() {
            free_stringtable(stringTable2);
        }
        return ret;
    }

    if nb1 == 0 || nb2 == 0 {
        ret = MK_E_NOPREFIX;
        IBindCtx_Release(bindctx);
        CoTaskMemFree(pathThis as *mut _);
        CoTaskMemFree(pathOther as *mut _);
        CoTaskMemFree(commonPath as *mut _);
        if !stringTable1.is_null() {
            free_stringtable(stringTable1);
        }
        if !stringTable2.is_null() {
            free_stringtable(stringTable2);
        }
        return ret;
    }

    let len1 = lstrlenW(pathThis);
    let len2 = lstrlenW(pathOther);
    let min_len = min(len1, len2);
    commonPath = CoTaskMemAlloc((core::mem::size_of::<u16>() * (min_len as usize + 1)) as u32) as LPOLESTR;
    if commonPath.is_null() {
        ret = E_OUTOFMEMORY;
        IBindCtx_Release(bindctx);
        CoTaskMemFree(pathThis as *mut _);
        CoTaskMemFree(pathOther as *mut _);
        CoTaskMemFree(commonPath as *mut _);
        if !stringTable1.is_null() {
            free_stringtable(stringTable1);
        }
        if !stringTable2.is_null() {
            free_stringtable(stringTable2);
        }
        return ret;
    }

    *commonPath = 0;

    sameIdx = 0;
    while !(*stringTable1.offset(sameIdx as isize)).is_null()
        && !(*stringTable2.offset(sameIdx as isize)).is_null()
        && lstrcmpiW(*stringTable1.offset(sameIdx as isize), *stringTable2.offset(sameIdx as isize)) == 0
    {
        sameIdx += 1;
    }

    if sameIdx > 1 && *(*stringTable1.offset(0)) == b'\\' as u16 && *(*stringTable2.offset(1)) == b'\\' as u16 {
        machineNameCase = 1;

        i = 2;
        while i < sameIdx {
            if *(*stringTable1.offset(i as isize)) == b'\\' as u16
                && (i + 1 < sameIdx)
                && *(*stringTable1.offset((i + 1) as isize)) == b'\\' as u16
            {
                machineNameCase = 0;
                break;
            }
            i += 1;
        }
    }

    if machineNameCase != 0 && *(*stringTable1.offset((sameIdx - 1) as isize)) == b'\\' as u16 {
        sameIdx -= 1;
    }

    if machineNameCase != 0 && (sameIdx <= 3) && (nb1 > 3 || nb2 > 3) {
        ret = MK_E_NOPREFIX;
    } else {
        i = 0;
        while i < sameIdx {
            lstrcatW(commonPath, *stringTable1.offset(i as isize));
            i += 1;
        }
        ret = CreateFileMoniker(commonPath, ppmkPrefix);
    }

    IBindCtx_Release(bindctx);
    CoTaskMemFree(pathThis as *mut _);
    CoTaskMemFree(pathOther as *mut _);
    CoTaskMemFree(commonPath as *mut _);
    if !stringTable1.is_null() {
        free_stringtable(stringTable1);
    }
    if !stringTable2.is_null() {
        free_stringtable(stringTable2);
    }

    ret
}
#[no_mangle]
pub unsafe extern "C" fn FileMonikerImpl_DecomposePath(str: LPCWSTR, stringTable: *mut *mut LPWSTR) -> i32 {
    static bSlash: [u16; 2] = ['\\' as u16, 0];
    let mut word: LPWSTR;
    let mut i: i32 = 0;
    let mut j: i32;
    let mut tabIndex: i32 = 0;
    let mut ret: i32 = 0;
    let mut strgtable: *mut LPWSTR;

    let len: i32 = lstrlenW(str);

    TRACE!(b"%s, %p\n\0".as_ptr() as *const i8, debugstr_w(str), *stringTable);

    strgtable = CoTaskMemAlloc(((len + 1) as usize * core::mem::size_of::<LPWSTR>()) as u32) as *mut LPWSTR;

    if strgtable.is_null() {
        return 0x8007000E; // E_OUTOFMEMORY
    }

    word = CoTaskMemAlloc(((len + 1) as usize * core::mem::size_of::<u16>()) as u32) as LPWSTR;

    if word.is_null() {
        ret = 0x8007000E; // E_OUTOFMEMORY
        'lend: loop {
            if ret < 0 {
                i = 0;
                while i < tabIndex {
                    CoTaskMemFree(*strgtable.offset(i as isize) as *mut core::ffi::c_void);
                    i += 1;
                }
                CoTaskMemFree(strgtable as *mut core::ffi::c_void);
            }
            CoTaskMemFree(word as *mut core::ffi::c_void);
            return ret;
        }
    }

    while *str.offset(i as isize) != 0 {
        if *str.offset(i as isize) == bSlash[0] {
            *strgtable.offset(tabIndex as isize) = CoTaskMemAlloc((2 * core::mem::size_of::<u16>()) as u32) as LPWSTR;

            if (*strgtable.offset(tabIndex as isize)).is_null() {
                ret = 0x8007000E; // E_OUTOFMEMORY
                'lend: loop {
                    if ret < 0 {
                        i = 0;
                        while i < tabIndex {
                            CoTaskMemFree(*strgtable.offset(i as isize) as *mut core::ffi::c_void);
                            i += 1;
                        }
                        CoTaskMemFree(strgtable as *mut core::ffi::c_void);
                    }
                    CoTaskMemFree(word as *mut core::ffi::c_void);
                    return ret;
                }
            }

            lstrcpyW(*strgtable.offset(tabIndex as isize), bSlash.as_ptr());
            tabIndex += 1;
            i += 1;
        } else {
            j = 0;
            while *str.offset(i as isize) != 0 && *str.offset(i as isize) != bSlash[0] {
                *word.offset(j as isize) = *str.offset(i as isize);
                i += 1;
                j += 1;
            }

            *word.offset(j as isize) = 0;

            *strgtable.offset(tabIndex as isize) = CoTaskMemAlloc((core::mem::size_of::<u16>() * (j + 1) as usize) as u32) as LPWSTR;

            if (*strgtable.offset(tabIndex as isize)).is_null() {
                ret = 0x8007000E; // E_OUTOFMEMORY
                'lend: loop {
                    if ret < 0 {
                        i = 0;
                        while i < tabIndex {
                            CoTaskMemFree(*strgtable.offset(i as isize) as *mut core::ffi::c_void);
                            i += 1;
                        }
                        CoTaskMemFree(strgtable as *mut core::ffi::c_void);
                    }
                    CoTaskMemFree(word as *mut core::ffi::c_void);
                    return ret;
                }
            }

            lstrcpyW(*strgtable.offset(tabIndex as isize), word);
            tabIndex += 1;
        }
    }
    *strgtable.offset(tabIndex as isize) = core::ptr::null_mut();

    *stringTable = strgtable;

    ret = tabIndex;

    'lend: loop {
        if ret < 0 {
            i = 0;
            while i < tabIndex {
                CoTaskMemFree(*strgtable.offset(i as isize) as *mut core::ffi::c_void);
                i += 1;
            }
            CoTaskMemFree(strgtable as *mut core::ffi::c_void);
        }
        CoTaskMemFree(word as *mut core::ffi::c_void);
        return ret;
    }
}

unsafe fn FileMonikerImpl_RelativePathTo(
    iface: *mut IMoniker,
    pmOther: *mut IMoniker,
    ppmkRelPath: *mut *mut IMoniker,
) -> HRESULT {
    let mut bind: *mut IBindCtx = core::ptr::null_mut();
    let mut res: HRESULT;
    let mut str1: LPOLESTR = core::ptr::null_mut();
    let mut str2: LPOLESTR = core::ptr::null_mut();
    let mut tabStr1: *mut LPOLESTR = core::ptr::null_mut();
    let mut tabStr2: *mut LPOLESTR = core::ptr::null_mut();
    let mut relPath: LPOLESTR;
    let mut len1: DWORD = 0;
    let mut len2: DWORD = 0;
    let mut sameIdx: DWORD = 0;
    let mut j: DWORD;
    static back: [u16; 4] = [0x002E, 0x002E, 0x005C, 0x0000]; // L"..\\"

    TRACE!(b"(%p,%p,%p)\n\0".as_ptr() as PCHAR, iface, pmOther, ppmkRelPath);

    if ppmkRelPath.is_null() {
        return E_POINTER;
    }

    if pmOther.is_null() {
        return E_INVALIDARG;
    }

    res = CreateBindCtx(0, &mut bind);
    if FAILED(res) {
        return res;
    }

    res = IMoniker_GetDisplayName(iface, bind, core::ptr::null_mut(), &mut str1);
    if FAILED(res) {
        return res;
    }
    res = IMoniker_GetDisplayName(pmOther, bind, core::ptr::null_mut(), &mut str2);
    if FAILED(res) {
        return res;
    }

    len1 = FileMonikerImpl_DecomposePath(str1, &mut tabStr1);
    if FAILED(len1 as HRESULT) {
        return E_OUTOFMEMORY;
    }
    len2 = FileMonikerImpl_DecomposePath(str2, &mut tabStr2);

    if FAILED(len2 as HRESULT) {
        free_stringtable(tabStr1);
        return E_OUTOFMEMORY;
    }

    while !(*tabStr1.add(sameIdx as usize)).is_null()
        && !(*tabStr2.add(sameIdx as usize)).is_null()
        && lstrcmpiW(*tabStr1.add(sameIdx as usize), *tabStr2.add(sameIdx as usize)) == 0
    {
        sameIdx += 1;
    }

    relPath = HeapAlloc(
        GetProcessHeap(),
        0,
        (core::mem::size_of::<u16>() as DWORD * (1 + lstrlenW(str1) as DWORD + lstrlenW(str2) as DWORD)) as usize,
    ) as LPOLESTR;

    *relPath = 0;

    if len2 > 0 && !(len1 == 1 && len2 == 1 && sameIdx == 0) {
        j = sameIdx;
        while !(*tabStr1.add(j as usize)).is_null() {
            if *(*tabStr1.add(j as usize)) != 0x005C {
                lstrcatW(relPath, back.as_ptr());
            }
            j += 1;
        }
    }

    j = sameIdx;
    while !(*tabStr2.add(j as usize)).is_null() {
        lstrcatW(relPath, *tabStr2.add(j as usize));
        j += 1;
    }

    res = CreateFileMoniker(relPath, ppmkRelPath);

    free_stringtable(tabStr1);
    free_stringtable(tabStr2);
    CoTaskMemFree(str1 as *mut core::ffi::c_void);
    CoTaskMemFree(str2 as *mut core::ffi::c_void);
    HeapFree(GetProcessHeap(), 0, relPath as *mut core::ffi::c_void);

    if len1 == 0 || len2 == 0 || (len1 == 1 && len2 == 1 && sameIdx == 0) {
        return MK_S_HIM;
    }

    res
}

unsafe fn FileMonikerImpl_GetDisplayName(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    ppszDisplayName: *mut LPOLESTR,
) -> HRESULT {

    let This = impl_from_IMoniker(iface);
    let len = lstrlenW((*This).filePathName);

    TRACE!(
        b"(%p,%p,%p,%p)\n\0".as_ptr(),
        iface,
        pbc,
        pmkToLeft,
        ppszDisplayName,
    );

    if ppszDisplayName.is_null() {
        return 0x80004003u32 as HRESULT;
    }

    if !pmkToLeft.is_null() {
        return 0x80070057u32 as HRESULT;
    }

    *ppszDisplayName = CoTaskMemAlloc((core::mem::size_of::<u16>() * (len as usize + 1))) as LPOLESTR;
    if (*ppszDisplayName).is_null() {
        return 0x8007000Eu32 as HRESULT;
    }

    lstrcpyW(*ppszDisplayName, (*This).filePathName);

    TRACE!(b"-- %s\n\0".as_ptr(), debugstr_w(*ppszDisplayName));

    0x00000000u32 as HRESULT
}

unsafe fn FileMonikerImpl_ParseDisplayName(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    pszDisplayName: LPOLESTR,
    pchEaten: *mut ULONG,
    ppmkOut: *mut *mut IMoniker,
) -> HRESULT {
    FIXME!(
        b"(%p,%p,%p,%p,%p,%p),stub!\n\0".as_ptr() as *const i8,
        iface,
        pbc,
        pmkToLeft,
        pszDisplayName,
        pchEaten,
        ppmkOut,
    );
    E_NOTIMPL
}

unsafe fn FileMonikerImpl_IsSystemMoniker(
    iface: *mut IMoniker,
    pwdMksys: LPDWORD,
) -> HRESULT {
    TRACE!(b"(%p,%p)\n\0".as_ptr() as *const i8, iface, pwdMksys);

    if pwdMksys.is_null() {
        return E_POINTER;
    }

    *pwdMksys = MKSYS_FILEMONIKER;

    return S_OK;
}
unsafe fn FileMonikerROTDataImpl_QueryInterface(
    iface: *mut IROTData,
    riid: *const CLSID,
    ppvObject: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let This = impl_from_IROTData(iface);


    TRACE!(
        b"(%p,%s,%p)\n\0".as_ptr(),
        This,
        debugstr_guid(riid),
        ppvObject,
    );

    FileMonikerImpl_QueryInterface(
        core::ptr::addr_of_mut!((*This).IMoniker_iface),
        riid,
        ppvObject,
    )
}

unsafe fn FileMonikerROTDataImpl_AddRef(iface: *mut IROTData) -> ULONG {
    let This = impl_from_IROTData(iface);


    TRACE!(b"(%p)\n\0".as_ptr(), This);

    IMoniker_AddRef(core::ptr::addr_of_mut!((*This).IMoniker_iface))
}

unsafe fn FileMonikerROTDataImpl_Release(iface: *mut IROTData) -> u32 {
    let This = impl_from_IROTData(iface);

    FileMonikerImpl_Release(core::ptr::addr_of_mut!((*This).IMoniker_iface))
}

unsafe fn FileMonikerROTDataImpl_GetComparisonData(
    iface: *mut IROTData,
    pbData: *mut u8,
    cbMax: u32,
    pcbData: *mut u32,
) -> HRESULT {

    let This = impl_from_IROTData(iface);
    let len = lstrlenW((*This).filePathName) + 1;
    let mut i: i32;
    let pszFileName: LPWSTR;

    *pcbData = (core::mem::size_of::<CLSID>() as i32 + len * core::mem::size_of::<u16>() as i32) as u32;
    if cbMax < *pcbData {
        return 0x8007000E;
    }

    memcpy(
        pbData as *mut core::ffi::c_void,
        core::ptr::addr_of!(CLSID_FileMoniker) as *const core::ffi::c_void,
        core::mem::size_of::<CLSID>(),
    );
    pszFileName = pbData.add(core::mem::size_of::<CLSID>()) as LPWSTR;
    i = 0;
    while i < len {
        *pszFileName.offset(i as isize) = towupper(*(*This).filePathName.offset(i as isize) as i32) as u16;
        i += 1;
    }

    0
}

unsafe fn FileMonikerImpl_Construct(This: *mut FileMonikerImpl, lpszPathName: LPCWSTR) -> HRESULT {
    let mut nb: i32;
    let mut i: i32;
    let mut sizeStr: i32 = lstrlenW(lpszPathName);
    let mut tabStr: *mut LPOLESTR = core::ptr::null_mut();
    static twoPoint: [u16; 3] = [b'.' as u16, b'.' as u16, 0];
    static bkSlash: [u16; 2] = [b'\\' as u16, 0];
    let mut addBkSlash: u8;

    TRACE!(b"(%p,%s)\n\0".as_ptr() as *const i8, This, debugstr_w(lpszPathName));

    (*This).IMoniker_iface.lpVtbl = &VT_FileMonikerImpl;
    (*This).IROTData_iface.lpVtbl = &VT_ROTDataImpl;
    (*This).ref_ = 0;
    (*This).pMarshal = core::ptr::null_mut();

    (*This).filePathName = HeapAlloc(GetProcessHeap(), 0, (core::mem::size_of::<u16>() * (sizeStr as usize + 1)) as usize) as LPOLESTR;

    if (*This).filePathName.is_null() {
        return 0x8007000E;
    }

    lstrcpyW((*This).filePathName, lpszPathName);

    nb = FileMonikerImpl_DecomposePath((*This).filePathName, &mut tabStr);

    if nb > 0 {
        addBkSlash = 1;
        if wcscmp(*tabStr.offset(0), twoPoint.as_ptr()) != 0 {
            addBkSlash = 0;
        } else {
            i = 0;
            while i < nb {
                if wcscmp(*tabStr.offset(i as isize), twoPoint.as_ptr()) != 0 && wcscmp(*tabStr.offset(i as isize), bkSlash.as_ptr()) != 0 {
                    addBkSlash = 0;
                    break;
                } else {
                    if wcscmp(*tabStr.offset(i as isize), bkSlash.as_ptr()) == 0 && i < nb - 1 && wcscmp(*tabStr.offset((i + 1) as isize), bkSlash.as_ptr()) == 0 {
                        *(*tabStr.offset(i as isize)) = 0;
                        sizeStr -= 1;
                        addBkSlash = 0;
                        break;
                    }
                }
                i += 1;
            }
        }

        if wcscmp(*tabStr.offset((nb - 1) as isize), bkSlash.as_ptr()) == 0 {
            addBkSlash = 0;
        }

        (*This).filePathName = HeapReAlloc(GetProcessHeap(), 0, (*This).filePathName as *mut core::ffi::c_void, (sizeStr as usize + 1) * core::mem::size_of::<u16>()) as LPOLESTR;

        *(*This).filePathName = 0;

        i = 0;
        while !(*tabStr.offset(i as isize)).is_null() {
            lstrcatW((*This).filePathName, *tabStr.offset(i as isize));
            i += 1;
        }

        if addBkSlash != 0 {
            lstrcatW((*This).filePathName, bkSlash.as_ptr());
        }
    }

    free_stringtable(tabStr);

    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn CreateFileMoniker(
    lpszPathName: LPCWSTR,
    ppmk: *mut *mut IMoniker,
) -> HRESULT {

    TRACE!(b"(%s,%p)\n\0".as_ptr(), debugstr_w(lpszPathName), ppmk);

    if ppmk.is_null() {
        return 0x80004003u32 as HRESULT;
    }

    if lpszPathName.is_null() {
        return 0x800401E4u32 as HRESULT;
    }

    *ppmk = core::ptr::null_mut();

    let newFileMoniker = HeapAlloc(GetProcessHeap(), 0, core::mem::size_of::<FileMonikerImpl>()) as *mut FileMonikerImpl;

    if newFileMoniker.is_null() {
        return 0x8007000Eu32 as HRESULT;
    }

    let hr = FileMonikerImpl_Construct(newFileMoniker, lpszPathName);

    let hr = if SUCCEEDED(hr) != 0 {
        IMoniker_QueryInterface(
            core::ptr::addr_of_mut!((*newFileMoniker).IMoniker_iface),
            &IID_IMoniker as *const CLSID,
            ppmk as *mut *mut core::ffi::c_void,
        )
    } else {
        HeapFree(GetProcessHeap(), 0, newFileMoniker as *mut core::ffi::c_void);
        hr
    };

    hr
}

unsafe fn memrpbrkW(ptr: *const u16, n: usize, accept: *const u16) -> *mut u16 {
    let end = ptr.add(n);
    let mut ret: *const u16 = core::ptr::null();
    let mut current = ptr;
    while current < end {
        if !wcschr(accept, *current).is_null() {
            ret = current;
        }
        current = current.add(1);
    }
    ret as *mut u16
}
#[no_mangle]
pub unsafe extern "C" fn FileMoniker_CreateFromDisplayName(
    pbc: *mut IBindCtx,
    szDisplayName: LPCWSTR,
    pchEaten: LPDWORD,
    ppmk: *mut *mut IMoniker,
) -> HRESULT {
    let wszSeparators: [u16; 5] = [':' as u16, '\\' as u16, '/' as u16, '!' as u16, 0];

    let mut end = szDisplayName.offset(lstrlenW(szDisplayName) as isize);

    while !end.is_null() && end != szDisplayName {
        end = memrpbrkW(szDisplayName, end.offset_from(szDisplayName) as usize, wszSeparators.as_ptr());
        if end.is_null() {
            break;
        }

        let len = end.offset_from(szDisplayName) as usize;

        let file_display_name = HeapAlloc(GetProcessHeap(), 0, (len + 1) * core::mem::size_of::<u16>()) as LPWSTR;
        if file_display_name.is_null() {
            return E_OUTOFMEMORY;
        }
        memcpy(file_display_name as *mut c_void, szDisplayName as *const c_void, len * core::mem::size_of::<u16>());
        *file_display_name.offset(len as isize) = 0;

        let mut file_moniker: *mut IMoniker = core::ptr::null_mut();
        let mut hr = CreateFileMoniker(file_display_name, &mut file_moniker);
        if FAILED(hr) != 0 {
            HeapFree(GetProcessHeap(), 0, file_display_name as LPVOID);
            return hr;
        }

        let mut rot: *mut IRunningObjectTable = core::ptr::null_mut();
        hr = IBindCtx_GetRunningObjectTable(pbc, &mut rot);
        if FAILED(hr) != 0 {
            HeapFree(GetProcessHeap(), 0, file_display_name as LPVOID);
            IMoniker_Release(file_moniker);
            return hr;
        }

        hr = IRunningObjectTable_IsRunning(rot, file_moniker);
        IRunningObjectTable_Release(rot);
        if FAILED(hr) != 0 {
            HeapFree(GetProcessHeap(), 0, file_display_name as LPVOID);
            IMoniker_Release(file_moniker);
            return hr;
        }
        if hr == S_OK {
            TRACE!(b"found running file moniker for %s\n\0".as_ptr() as *const c_char, debugstr_w(file_display_name));
            *pchEaten = len as u32;
            *ppmk = file_moniker;
            HeapFree(GetProcessHeap(), 0, file_display_name as LPVOID);
            return S_OK;
        }

        let full_path_name_len = GetFullPathNameW(file_display_name, 0, core::ptr::null_mut(), core::ptr::null_mut());
        if full_path_name_len == 0 {
            HeapFree(GetProcessHeap(), 0, file_display_name as LPVOID);
            IMoniker_Release(file_moniker);
            return MK_E_SYNTAX;
        }
        let full_path_name = HeapAlloc(GetProcessHeap(), 0, full_path_name_len as usize * core::mem::size_of::<u16>()) as LPWSTR;
        if full_path_name.is_null() {
            HeapFree(GetProcessHeap(), 0, file_display_name as LPVOID);
            IMoniker_Release(file_moniker);
            return E_OUTOFMEMORY;
        }
        GetFullPathNameW(file_display_name, full_path_name_len, full_path_name, core::ptr::null_mut());

        if GetFileAttributesW(full_path_name) == INVALID_FILE_ATTRIBUTES {
            TRACE!(b"couldn't open file %s\n\0".as_ptr() as *const c_char, debugstr_w(full_path_name));
        } else {
            TRACE!(b"got file moniker for %s\n\0".as_ptr() as *const c_char, debugstr_w(szDisplayName));
            *pchEaten = len as u32;
            *ppmk = file_moniker;
            HeapFree(GetProcessHeap(), 0, file_display_name as LPVOID);
            HeapFree(GetProcessHeap(), 0, full_path_name as LPVOID);
            return S_OK;
        }
        HeapFree(GetProcessHeap(), 0, file_display_name as LPVOID);
        HeapFree(GetProcessHeap(), 0, full_path_name as LPVOID);
        IMoniker_Release(file_moniker);
    }

    MK_E_CANTOPENFILE
}
#[no_mangle]
pub unsafe extern "system" fn FileMoniker_CreateInstance(
    iface: *mut IClassFactory,
    pUnk: *mut IUnknown,
    riid: *const GUID,
    ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    static wszEmpty: [WCHAR; 1] = [0];

    TRACE!(
        b"(%p, %s, %p)\n\0".as_ptr() as PCHAR,
        pUnk,
        debugstr_guid(riid),
        ppv,
    );

    *ppv = core::ptr::null_mut();

    if !pUnk.is_null() {
        return CLASS_E_NOAGGREGATION;
    }

    let newFileMoniker = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<FileMonikerImpl>(),
    ) as *mut FileMonikerImpl;

    if newFileMoniker.is_null() {
        return E_OUTOFMEMORY;
    }

    let mut hr = FileMonikerImpl_Construct(newFileMoniker, wszEmpty.as_ptr());

    if SUCCEEDED(hr) {
        hr = IMoniker_QueryInterface(
            core::ptr::addr_of_mut!((*newFileMoniker).IMoniker_iface),
            riid,
            ppv,
        );
    }

    if FAILED(hr) {
        HeapFree(GetProcessHeap(), 0, newFileMoniker as *mut core::ffi::c_void);
    }

    hr
}
