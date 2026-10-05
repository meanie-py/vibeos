//! FileMoniker implementation — translation of dll/win32/ole32/filemoniker.c
//!
//! IMoniker (+ IPersistStream via the IMoniker vtbl, as COM inherits it) and IROTData
//! over a single file-path string. Refcount-, string- and CoTaskMem-heavy; the C is the
//! oracle for every alloc/free pairing here.
use crate::prelude::*;
use core::ffi::c_void;

// ---------------------------------------------------------------------------
// Local bits the shared surface does not (yet) carry.
//
// All three items below belong in the shared surface (IID_IClassActivator in
// crate::iids — classmoniker.rs needs it too; the file-attribute trio in
// crate::win32). Declared privately here so they cannot collide with a future
// shared definition (a local item shadows a glob import).
// ---------------------------------------------------------------------------

extern "C" {
    /// uuid.lib data symbol (same class as every other IID_* in crate::iids).
    static IID_IClassActivator: GUID;
}

/// fileapi.h: `GET_FILEEX_INFO_LEVELS::GetFileExInfoStandard`
const GetFileExInfoStandard: c_int = 0;

/// winnls.h: `MB_ERR_INVALID_CHARS`
const MB_ERR_INVALID_CHARS: DWORD = 0x0000_0008;

/// fileapi.h: WIN32_FILE_ATTRIBUTE_DATA
#[repr(C)]
#[derive(Copy, Clone)]
struct WIN32_FILE_ATTRIBUTE_DATA {
    dwFileAttributes: DWORD,
    ftCreationTime: FILETIME,
    ftLastAccessTime: FILETIME,
    ftLastWriteTime: FILETIME,
    nFileSizeHigh: DWORD,
    nFileSizeLow: DWORD,
}

extern "system" {
    // WINAPI → stdcall on i386: `extern "system"`, never `extern "C"`.
    fn GetFileAttributesExW(
        lpFileName: LPCWSTR,
        fInfoLevelId: c_int,
        lpFileInformation: LPVOID,
    ) -> BOOL;
}

// ---------------------------------------------------------------------------
// struct FileMonikerImpl
// ---------------------------------------------------------------------------

/// filemoniker.c: `typedef struct FileMonikerImpl` (`ref` → `ref_`, a Rust keyword).
#[repr(C)]
pub struct FileMonikerImpl {
    pub IMoniker_iface: IMoniker,
    pub IROTData_iface: IROTData,
    pub ref_: LONG,
    pub filePathName: LPOLESTR,
    pub pMarshal: *mut IUnknown,
}

#[inline]
unsafe fn impl_from_IMoniker(iface: *mut IMoniker) -> *mut FileMonikerImpl {
    CONTAINING_RECORD!(iface, FileMonikerImpl, IMoniker_iface)
}

#[inline]
unsafe fn impl_from_IROTData(iface: *mut IROTData) -> *mut FileMonikerImpl {
    CONTAINING_RECORD!(iface, FileMonikerImpl, IROTData_iface)
}

// ---------------------------------------------------------------------------
// IMoniker
// ---------------------------------------------------------------------------

stdmethod! {
fn FileMonikerImpl_QueryInterface(
    iface: *mut IMoniker,
    riid: *const IID,
    ppvObject: *mut *mut c_void,
) -> HRESULT {
    let This = impl_from_IMoniker(iface);

    TRACE!(b"(%p,%s,%p)\n\0".as_ptr() as *const c_char, This, debugstr_guid(riid as *const c_void), ppvObject);

    /* Perform a sanity check on the parameters.*/
    if ppvObject.is_null() {
        return E_INVALIDARG;
    }

    /* Initialize the return parameter */
    *ppvObject = core::ptr::null_mut();

    /* Compare the riid with the interface IDs implemented by this object.*/
    if IsEqualIID(core::ptr::addr_of!(IID_IUnknown), riid)
        || IsEqualIID(core::ptr::addr_of!(IID_IPersist), riid)
        || IsEqualIID(core::ptr::addr_of!(IID_IPersistStream), riid)
        || IsEqualIID(core::ptr::addr_of!(IID_IMoniker), riid)
        || IsEqualGUID(core::ptr::addr_of!(CLSID_FileMoniker), riid)
    {
        *ppvObject = iface as *mut c_void;
    } else if IsEqualIID(core::ptr::addr_of!(IID_IROTData), riid) {
        *ppvObject = core::ptr::addr_of_mut!((*This).IROTData_iface) as *mut c_void;
    } else if IsEqualIID(core::ptr::addr_of!(IID_IMarshal), riid) {
        let mut hr: HRESULT = S_OK;
        if (*This).pMarshal.is_null() {
            hr = MonikerMarshal_Create(iface, core::ptr::addr_of_mut!((*This).pMarshal));
        }
        if hr != S_OK {
            return hr;
        }
        return IUnknown_QueryInterface((*This).pMarshal, riid, ppvObject);
    }

    /* Check that we obtained an interface.*/
    if (*ppvObject).is_null() {
        return E_NOINTERFACE;
    }

    /* QueryInterface always increases the reference count by one when it is successful */
    IMoniker_AddRef(iface);

    S_OK
}
}

stdmethod! {
fn FileMonikerImpl_AddRef(iface: *mut IMoniker) -> ULONG {
    let This = impl_from_IMoniker(iface);

    TRACE!(b"(%p)\n\0".as_ptr() as *const c_char, iface);

    InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref_)) as ULONG
}
}

stdmethod! {
fn FileMonikerImpl_Release(iface: *mut IMoniker) -> ULONG {
    let This = impl_from_IMoniker(iface);

    TRACE!(b"(%p)\n\0".as_ptr() as *const c_char, iface);

    let ref_ = InterlockedDecrement(core::ptr::addr_of_mut!((*This).ref_)) as ULONG;

    /* destroy the object if there are no more references to it */
    if ref_ == 0 {
        FileMonikerImpl_Destroy(This);
    }

    ref_
}
}

stdmethod! {
fn FileMonikerImpl_GetClassID(iface: *mut IMoniker, pClassID: *mut CLSID) -> HRESULT {
    TRACE!(b"(%p,%p)\n\0".as_ptr() as *const c_char, iface, pClassID);

    if pClassID.is_null() {
        return E_POINTER;
    }

    *pClassID = CLSID_FileMoniker;

    S_OK
}
}

stdmethod! {
/// Note that the OLE-provided implementations of IPersistStream::IsDirty always return
/// S_FALSE because their internal state never changes.
fn FileMonikerImpl_IsDirty(iface: *mut IMoniker) -> HRESULT {
    TRACE!(b"(%p)\n\0".as_ptr() as *const c_char, iface);

    S_FALSE
}
}

/// filemoniker.c `FileMonikerImpl_Load`'s `fail:` label: free both scratch buffers and
/// map a non-failure HRESULT to E_FAIL. (HeapFree of NULL is what the C does too — both
/// pointers are initialised to NULL and only some paths have allocated them.)
#[inline]
unsafe fn load_fail(res: HRESULT, filePathA: *mut c_char, filePathW: *mut u16) -> HRESULT {
    HeapFree(GetProcessHeap(), 0, filePathA as LPVOID);
    HeapFree(GetProcessHeap(), 0, filePathW as LPVOID);

    if SUCCEEDED(res) {
        E_FAIL
    } else {
        res
    }
}

stdmethod! {
/// locates and reads from the stream the filePath string written by FileMonikerImpl_Save
fn FileMonikerImpl_Load(iface: *mut IMoniker, pStm: *mut IStream) -> HRESULT {
    let This = impl_from_IMoniker(iface);
    let mut res: HRESULT;
    let mut filePathA: *mut c_char = core::ptr::null_mut();
    let mut filePathW: *mut u16 = core::ptr::null_mut();
    let mut bread: ULONG = 0;
    let mut wbuffer: u16 = 0;
    let mut dwbuffer: DWORD = 0;
    let mut bytesA: DWORD = 0;
    let bytesW: DWORD;
    let len: DWORD;

    TRACE!(b"(%p,%p)\n\0".as_ptr() as *const c_char, iface, pStm);

    /* first WORD */
    res = IStream_Read(pStm, core::ptr::addr_of_mut!(wbuffer) as *mut c_void,
                       core::mem::size_of::<u16>() as ULONG, &mut bread);
    if bread != core::mem::size_of::<u16>() as ULONG {
        WARN!(b"Couldn't read 0 word\n\0".as_ptr() as *const c_char);
        return load_fail(res, filePathA, filePathW);
    }

    /* read filePath string length (plus one) */
    res = IStream_Read(pStm, core::ptr::addr_of_mut!(bytesA) as *mut c_void,
                       core::mem::size_of::<DWORD>() as ULONG, &mut bread);
    if bread != core::mem::size_of::<DWORD>() as ULONG {
        WARN!(b"Couldn't read file string length\n\0".as_ptr() as *const c_char);
        return load_fail(res, filePathA, filePathW);
    }

    /* read filePath string */
    filePathA = HeapAlloc(GetProcessHeap(), 0, bytesA as SIZE_T) as *mut c_char;
    if filePathA.is_null() {
        res = E_OUTOFMEMORY;
        return load_fail(res, filePathA, filePathW);
    }

    res = IStream_Read(pStm, filePathA as *mut c_void, bytesA, &mut bread);
    if bread != bytesA {
        WARN!(b"Couldn't read file path string\n\0".as_ptr() as *const c_char);
        return load_fail(res, filePathA, filePathW);
    }

    /* read the unknown value (the C ignores this read's HRESULT, keeping the previous
     * `res` for the fail path) */
    IStream_Read(pStm, core::ptr::addr_of_mut!(wbuffer) as *mut c_void,
                 core::mem::size_of::<u16>() as ULONG, &mut bread);
    if bread != core::mem::size_of::<u16>() as ULONG {
        WARN!(b"Couldn't read unknown value\n\0".as_ptr() as *const c_char);
        return load_fail(res, filePathA, filePathW);
    }

    /* read the DEAD constant */
    IStream_Read(pStm, core::ptr::addr_of_mut!(wbuffer) as *mut c_void,
                 core::mem::size_of::<u16>() as ULONG, &mut bread);
    if bread != core::mem::size_of::<u16>() as ULONG {
        WARN!(b"Couldn't read DEAD constant\n\0".as_ptr() as *const c_char);
        return load_fail(res, filePathA, filePathW);
    }

    let mut i: c_int = 0;
    while i < 5 {
        res = IStream_Read(pStm, core::ptr::addr_of_mut!(dwbuffer) as *mut c_void,
                           core::mem::size_of::<DWORD>() as ULONG, &mut bread);
        if bread != core::mem::size_of::<DWORD>() as ULONG {
            WARN!(b"Couldn't read 0 padding\n\0".as_ptr() as *const c_char);
            return load_fail(res, filePathA, filePathW);
        }
        i += 1;
    }

    res = IStream_Read(pStm, core::ptr::addr_of_mut!(dwbuffer) as *mut c_void,
                       core::mem::size_of::<DWORD>() as ULONG, &mut bread);
    if bread != core::mem::size_of::<DWORD>() as ULONG {
        return load_fail(res, filePathA, filePathW);
    }

    if dwbuffer == 0 {
        /* No W-string */
        bytesA -= 1;
        len = MultiByteToWideChar(CP_ACP, MB_ERR_INVALID_CHARS, filePathA, bytesA as c_int,
                                  core::ptr::null_mut(), 0) as DWORD;
        if len == 0 {
            return load_fail(res, filePathA, filePathW);
        }

        filePathW = HeapAlloc(GetProcessHeap(), 0,
                              ((len as usize + 1) * core::mem::size_of::<u16>()) as SIZE_T) as *mut u16;
        if filePathW.is_null() {
            res = E_OUTOFMEMORY;
            return load_fail(res, filePathA, filePathW);
        }
        MultiByteToWideChar(CP_ACP, MB_ERR_INVALID_CHARS, filePathA, -1, filePathW,
                            (len + 1) as c_int);

        /* succeed: */
        HeapFree(GetProcessHeap(), 0, filePathA as LPVOID);
        HeapFree(GetProcessHeap(), 0, (*This).filePathName as LPVOID);
        (*This).filePathName = filePathW;

        return S_OK;
    }

    if dwbuffer < 6 {
        return load_fail(res, filePathA, filePathW);
    }

    bytesW = dwbuffer - 6;

    res = IStream_Read(pStm, core::ptr::addr_of_mut!(dwbuffer) as *mut c_void,
                       core::mem::size_of::<DWORD>() as ULONG, &mut bread);
    if bread != core::mem::size_of::<DWORD>() as ULONG || dwbuffer != bytesW {
        return load_fail(res, filePathA, filePathW);
    }

    res = IStream_Read(pStm, core::ptr::addr_of_mut!(wbuffer) as *mut c_void,
                       core::mem::size_of::<u16>() as ULONG, &mut bread);
    if bread != core::mem::size_of::<u16>() as ULONG || wbuffer != 0x3 {
        return load_fail(res, filePathA, filePathW);
    }

    len = bytesW / core::mem::size_of::<u16>() as DWORD;
    filePathW = HeapAlloc(GetProcessHeap(), 0,
                          ((len as usize + 1) * core::mem::size_of::<u16>()) as SIZE_T) as *mut u16;
    if filePathW.is_null() {
        res = E_OUTOFMEMORY;
        return load_fail(res, filePathA, filePathW);
    }
    res = IStream_Read(pStm, filePathW as *mut c_void, bytesW, &mut bread);
    if bread != bytesW {
        return load_fail(res, filePathA, filePathW);
    }

    *filePathW.add(len as usize) = 0;

    /* succeed: */
    HeapFree(GetProcessHeap(), 0, filePathA as LPVOID);
    HeapFree(GetProcessHeap(), 0, (*This).filePathName as LPVOID);
    (*This).filePathName = filePathW;

    S_OK
}
}

stdmethod! {
/// Data which must be written on stream is:
/// 1) WORD constant: zero
/// 2) length of the path string ("\0" included)
/// 3) path string type A
/// 4) Unknown WORD value: frequently 0xFFFF
/// 5) WORD constant: 0xDEAD
/// 6) five DWORD constant: zero
/// 7) if we're only writing the multibyte version, write a zero DWORD and finish
/// 8) DWORD: double-length of the path string type W ("\0" not included)
/// 9) WORD constant: 0x3
/// 10) filePath unicode string
fn FileMonikerImpl_Save(iface: *mut IMoniker, pStm: *mut IStream, fClearDirty: BOOL) -> HRESULT {
    let This = impl_from_IMoniker(iface);
    let mut res: HRESULT;
    let filePathW: LPOLESTR = (*This).filePathName;
    let filePathA: *mut c_char;
    let bytesA: DWORD;
    let mut bytesW: DWORD;
    let len: DWORD;

    static FFFF: u16 = 0xFFFF; /* Constants */
    static DEAD: u16 = 0xDEAD;
    static ZERO: DWORD = 0;
    static THREE: u16 = 0x3;

    let mut bUsedDefault: BOOL = 0;
    let mut bWriteWide: BOOL;

    TRACE!(b"(%p,%p,%d)\n\0".as_ptr() as *const c_char, iface, pStm, fClearDirty);

    if pStm.is_null() {
        return E_POINTER;
    }

    /* write a 0 WORD */
    res = IStream_Write(pStm, core::ptr::addr_of!(ZERO) as *const c_void,
                        core::mem::size_of::<u16>() as ULONG, core::ptr::null_mut());
    if FAILED(res) { return res; }

    /* write length of filePath string ( 0 included )*/
    bytesA = WideCharToMultiByte(CP_ACP, 0, filePathW, -1, core::ptr::null_mut(), 0,
                                 core::ptr::null(), core::ptr::null_mut()) as DWORD;
    res = IStream_Write(pStm, core::ptr::addr_of!(bytesA) as *const c_void,
                        core::mem::size_of::<DWORD>() as ULONG, core::ptr::null_mut());
    if FAILED(res) { return res; }

    /* write A string (with '\0') */
    filePathA = HeapAlloc(GetProcessHeap(), 0, bytesA as SIZE_T) as *mut c_char;
    if filePathA.is_null() {
        return E_OUTOFMEMORY;
    }
    WideCharToMultiByte(CP_ACP, 0, filePathW, -1, filePathA, bytesA as c_int,
                        core::ptr::null(), &mut bUsedDefault);
    res = IStream_Write(pStm, filePathA as *const c_void, bytesA, core::ptr::null_mut());
    HeapFree(GetProcessHeap(), 0, filePathA as LPVOID);
    if FAILED(res) { return res; }

    /* write a WORD 0xFFFF */
    res = IStream_Write(pStm, core::ptr::addr_of!(FFFF) as *const c_void,
                        core::mem::size_of::<u16>() as ULONG, core::ptr::null_mut());
    if FAILED(res) { return res; }

    /* write a WORD 0xDEAD */
    res = IStream_Write(pStm, core::ptr::addr_of!(DEAD) as *const c_void,
                        core::mem::size_of::<u16>() as ULONG, core::ptr::null_mut());
    if FAILED(res) { return res; }

    /* write 5 zero DWORDs */
    let mut i: c_int = 0;
    while i < 5 {
        res = IStream_Write(pStm, core::ptr::addr_of!(ZERO) as *const c_void,
                            core::mem::size_of::<DWORD>() as ULONG, core::ptr::null_mut());
        if FAILED(res) { return res; }
        i += 1;
    }

    /* Write the wide version if:
     *    + couldn't convert to CP_ACP,
     * or + it's a directory,
     * or + there's a character > 0xFF
     */
    len = lstrlenW(filePathW) as DWORD;
    bWriteWide = (bUsedDefault != 0
        || (len > 0 && *filePathW.add((len - 1) as usize) == '\\' as u16)) as BOOL;
    if bWriteWide == 0 {
        let mut pch: LPOLESTR = filePathW;
        while *pch != 0 {
            if *pch > 0xFF {
                bWriteWide = 1;
                break;
            }
            pch = pch.add(1);
        }
    }

    if bWriteWide == 0 {
        return IStream_Write(pStm, core::ptr::addr_of!(ZERO) as *const c_void,
                             core::mem::size_of::<DWORD>() as ULONG, core::ptr::null_mut());
    }

    /* write bytes needed for the filepathW (without 0) + 6 */
    bytesW = len * core::mem::size_of::<u16>() as DWORD + 6;
    res = IStream_Write(pStm, core::ptr::addr_of!(bytesW) as *const c_void,
                        core::mem::size_of::<DWORD>() as ULONG, core::ptr::null_mut());
    if FAILED(res) { return res; }

    /* try again, without the extra 6 */
    bytesW -= 6;
    res = IStream_Write(pStm, core::ptr::addr_of!(bytesW) as *const c_void,
                        core::mem::size_of::<DWORD>() as ULONG, core::ptr::null_mut());
    if FAILED(res) { return res; }

    /* write a WORD 3 */
    res = IStream_Write(pStm, core::ptr::addr_of!(THREE) as *const c_void,
                        core::mem::size_of::<u16>() as ULONG, core::ptr::null_mut());
    if FAILED(res) { return res; }

    /* write W string (no 0) */
    IStream_Write(pStm, filePathW as *const c_void, bytesW, core::ptr::null_mut())
}
}

stdmethod! {
fn FileMonikerImpl_GetSizeMax(iface: *mut IMoniker, pcbSize: *mut ULARGE_INTEGER) -> HRESULT {
    let This = impl_from_IMoniker(iface);

    TRACE!(b"(%p,%p)\n\0".as_ptr() as *const c_char, iface, pcbSize);

    if pcbSize.is_null() {
        return E_POINTER;
    }

    /* We could calculate exactly (see ...::Save()) but instead
     * we'll make a quick over-estimate, like Windows (NT4, XP) does.
     */
    (*pcbSize).u.LowPart = 0x38 + 4 * lstrlenW((*This).filePathName) as DWORD;
    (*pcbSize).u.HighPart = 0;

    S_OK
}
}

/// filemoniker.c: `HRESULT FileMonikerImpl_Destroy(FileMonikerImpl *This)`
pub unsafe fn FileMonikerImpl_Destroy(This: *mut FileMonikerImpl) -> HRESULT {
    TRACE!(b"(%p)\n\0".as_ptr() as *const c_char, This);

    if !(*This).pMarshal.is_null() {
        IUnknown_Release((*This).pMarshal);
    }
    HeapFree(GetProcessHeap(), 0, (*This).filePathName as LPVOID);
    HeapFree(GetProcessHeap(), 0, This as LPVOID);

    S_OK
}

stdmethod! {
fn FileMonikerImpl_BindToObject(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    riid: *const IID,
    ppvResult: *mut *mut c_void,
) -> HRESULT {
    let This = impl_from_IMoniker(iface);
    let mut res: HRESULT = E_FAIL;
    let mut clsID: CLSID = core::mem::zeroed();
    let mut pObj: *mut IUnknown = core::ptr::null_mut();
    let mut prot: *mut IRunningObjectTable = core::ptr::null_mut();
    let mut ppf: *mut IPersistFile = core::ptr::null_mut();
    let mut pcf: *mut IClassFactory = core::ptr::null_mut();
    let mut pca: *mut IClassActivator = core::ptr::null_mut();

    *ppvResult = core::ptr::null_mut();

    TRACE!(b"(%p,%p,%p,%s,%p)\n\0".as_ptr() as *const c_char, iface, pbc, pmkToLeft,
           debugstr_guid(riid as *const c_void), ppvResult);

    if pmkToLeft.is_null() {
        res = IBindCtx_GetRunningObjectTable(pbc, &mut prot);

        if SUCCEEDED(res) {
            /* if the requested class was loaded before ! we don't need to reload it */
            res = IRunningObjectTable_GetObject(prot, iface, &mut pObj);

            if res != S_OK {
                /* first activation of this class */
                res = crate::ring::moniker::GetClassFile((*This).filePathName, &mut clsID);
                if SUCCEEDED(res) {
                    res = CoCreateInstance(
                        &clsID,
                        core::ptr::null_mut(),
                        CLSCTX_SERVER,
                        core::ptr::addr_of!(IID_IPersistFile),
                        &mut ppf as *mut *mut IPersistFile as *mut LPVOID,
                    );
                    if SUCCEEDED(res) {
                        res = IPersistFile_Load(ppf, (*This).filePathName, STGM_READ);
                        if SUCCEEDED(res) {
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
            core::ptr::addr_of!(IID_IClassFactory),
            &mut pcf as *mut *mut IClassFactory as *mut *mut c_void,
        );

        if res == E_NOINTERFACE {
            res = IMoniker_BindToObject(
                pmkToLeft,
                pbc,
                core::ptr::null_mut(),
                core::ptr::addr_of!(IID_IClassActivator),
                &mut pca as *mut *mut IClassActivator as *mut *mut c_void,
            );

            if res == E_NOINTERFACE {
                return MK_E_INTERMEDIATEINTERFACENOTSUPPORTED;
            }
        }
        if !pcf.is_null() {
            IClassFactory_CreateInstance(
                pcf,
                core::ptr::null_mut(),
                core::ptr::addr_of!(IID_IPersistFile),
                &mut ppf as *mut *mut IPersistFile as *mut *mut c_void,
            );

            res = IPersistFile_Load(ppf, (*This).filePathName, STGM_READ);

            if SUCCEEDED(res) {
                pObj = ppf as *mut IUnknown;
                IUnknown_AddRef(pObj);
            }
        }
    }

    bind_to_object_finish(This, pbc, riid, ppvResult, res, pObj, prot, ppf, pcf, pca)
}
}

/// The tail of filemoniker.c's FileMonikerImpl_BindToObject (the shared
/// `if (pObj) … release everything …` epilogue after both branches).
#[allow(clippy::too_many_arguments)]
unsafe fn bind_to_object_finish(
    _This: *mut FileMonikerImpl,
    pbc: *mut IBindCtx,
    riid: *const IID,
    ppvResult: *mut *mut c_void,
    mut res: HRESULT,
    pObj: *mut IUnknown,
    prot: *mut IRunningObjectTable,
    ppf: *mut IPersistFile,
    pcf: *mut IClassFactory,
    pca: *mut IClassActivator,
) -> HRESULT {
    if !pca.is_null() {
        FIXME!(b"()\n\0".as_ptr() as *const c_char);

        /* res=GetClassFile(This->filePathName,&clsID);
         * if (SUCCEEDED(res)) {
         *     res=IClassActivator_GetClassObject(pca,&clsID,CLSCTX_ALL,0,&IID_IPersistFile,(void**)&ppf);
         *     if (SUCCEEDED(res)) { pObj=(IUnknown*)ppf; IUnknown_AddRef(pObj); }
         * } */
    }

    if !pObj.is_null() {
        /* get the requested interface from the loaded class */
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

stdmethod! {
/// wine-10.0: needs a bind context, opens with ITS grfMode (not a hard-coded
/// READWRITE|SHARE_DENY_WRITE) on the moniker's own path, and writes straight into
/// the caller's out-pointer.
fn FileMonikerImpl_BindToStorage(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    riid: *const IID,
    object: *mut *mut c_void,
) -> HRESULT {
    let moniker = impl_from_IMoniker(iface);
    let mut bind_opts: BIND_OPTS = core::mem::zeroed();

    TRACE!(b"(%p,%p,%p,%s,%p)\n\0".as_ptr() as *const c_char, iface, pbc, pmkToLeft,
           debugstr_guid(riid as *const c_void), object);

    if pbc.is_null() {
        return E_INVALIDARG;
    }

    bind_opts.cbStruct = core::mem::size_of::<BIND_OPTS>() as DWORD;
    let hr = IBindCtx_GetBindOptions(pbc, &mut bind_opts);
    if FAILED(hr) {
        return hr;
    }

    if pmkToLeft.is_null() {
        if IsEqualIID(core::ptr::addr_of!(IID_IStorage), riid) {
            return StgOpenStorage(
                (*moniker).filePathName,
                core::ptr::null_mut(),
                bind_opts.grfMode,
                core::ptr::null_mut(),
                0,
                object as *mut *mut IStorage,
            );
        } else if IsEqualIID(core::ptr::addr_of!(IID_IStream), riid)
            || IsEqualIID(core::ptr::addr_of!(IID_ILockBytes), riid)
        {
            return E_FAIL;
        } else {
            return E_NOINTERFACE;
        }
    }

    FIXME!(b"(%p,%p,%p,%s,%p)\n\0".as_ptr() as *const c_char, iface, pbc, pmkToLeft,
           debugstr_guid(riid as *const c_void), object);

    E_NOTIMPL
}
}

stdmethod! {
fn FileMonikerImpl_Reduce(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    dwReduceHowFar: DWORD,
    ppmkToLeft: *mut *mut IMoniker,
    ppmkReduced: *mut *mut IMoniker,
) -> HRESULT {
    TRACE!(b"(%p,%p,%d,%p,%p)\n\0".as_ptr() as *const c_char, iface, pbc, dwReduceHowFar,
           ppmkToLeft, ppmkReduced);

    if pbc.is_null() || ppmkReduced.is_null() {
        return E_INVALIDARG;
    }

    IMoniker_AddRef(iface);

    *ppmkReduced = iface;

    MK_S_REDUCED_TO_SELF
}
}

unsafe fn free_stringtable(stringTable: *mut LPOLESTR) {
    let mut i: c_int = 0;

    while !(*stringTable.offset(i as isize)).is_null() {
        CoTaskMemFree(*stringTable.offset(i as isize) as LPVOID);
        i += 1;
    }
    CoTaskMemFree(stringTable as LPVOID);
}

stdmethod! {
fn FileMonikerImpl_ComposeWith(
    iface: *mut IMoniker,
    pmkRight: *mut IMoniker,
    fOnlyIfNotGeneric: BOOL,
    ppmkComposite: *mut *mut IMoniker,
) -> HRESULT {
    let res: HRESULT;
    let mut str1: LPOLESTR = core::ptr::null_mut();
    let mut str2: LPOLESTR = core::ptr::null_mut();
    let mut strDec1: *mut LPOLESTR = core::ptr::null_mut();
    let mut strDec2: *mut LPOLESTR = core::ptr::null_mut();
    let newStr: LPOLESTR;
    const TWO_POINT: [u16; 3] = ['.' as u16, '.' as u16, 0];
    const BK_SLASH: [u16; 2] = ['\\' as u16, 0];
    let mut bind: *mut IBindCtx = core::ptr::null_mut();
    let mut i: c_int;
    let mut j: c_int;
    let mut lastIdx1: c_int;
    let lastIdx2: c_int;
    let mut mkSys: DWORD = 0;
    let mut order: DWORD = 0;

    TRACE!(b"(%p,%p,%d,%p)\n\0".as_ptr() as *const c_char, iface, pmkRight, fOnlyIfNotGeneric,
           ppmkComposite);

    if ppmkComposite.is_null() {
        return E_POINTER;
    }

    if pmkRight.is_null() {
        return E_INVALIDARG;
    }

    *ppmkComposite = core::ptr::null_mut();

    IMoniker_IsSystemMoniker(pmkRight, &mut mkSys);

    /* check if we have two FileMonikers to compose or not */
    if mkSys == MKSYS_FILEMONIKER {
        CreateBindCtx(0, &mut bind);

        IMoniker_GetDisplayName(iface, bind, core::ptr::null_mut(), &mut str1);
        IMoniker_GetDisplayName(pmkRight, bind, core::ptr::null_mut(), &mut str2);

        /* decompose pathnames of the two monikers : (to prepare the path merge operation ) */
        lastIdx1 = FileMonikerImpl_DecomposePath(str1, &mut strDec1) - 1;
        lastIdx2 = FileMonikerImpl_DecomposePath(str2, &mut strDec2) - 1;

        if (lastIdx1 == -1 && lastIdx2 > -1)
            || (lastIdx1 == 1 && wcscmp(*strDec1.offset(0), TWO_POINT.as_ptr()) == 0)
        {
            res = MK_E_SYNTAX;
        } else {
            if wcscmp(*strDec1.offset(lastIdx1 as isize), BK_SLASH.as_ptr()) == 0 {
                lastIdx1 -= 1;
            }

            /* for each "..\" in the left of str2 remove the right element from str1 */
            i = 0;
            while lastIdx1 >= 0
                && !(*strDec2.offset(i as isize)).is_null()
                && wcscmp(*strDec2.offset(i as isize), TWO_POINT.as_ptr()) == 0
            {
                lastIdx1 -= 2;
                i += 2;
            }

            /* the length of the composed path string is increased by the sum of the two
             * paths' lengths */
            newStr = HeapAlloc(
                GetProcessHeap(),
                0,
                (core::mem::size_of::<u16>()
                    * (lstrlenW(str1) + lstrlenW(str2) + 1) as usize) as SIZE_T,
            ) as LPOLESTR;

            if !newStr.is_null() {
                /* new path is the concatenation of the rest of str1 and str2 */
                *newStr = 0;
                j = 0;
                while j <= lastIdx1 {
                    lstrcatW(newStr, *strDec1.offset(j as isize));
                    j += 1;
                }

                if ((*strDec2.offset(i as isize)).is_null() && lastIdx1 > -1 && lastIdx2 > -1)
                    || wcscmp(*strDec2.offset(i as isize), BK_SLASH.as_ptr()) != 0
                {
                    lstrcatW(newStr, BK_SLASH.as_ptr());
                }

                j = i;
                while j <= lastIdx2 {
                    lstrcatW(newStr, *strDec2.offset(j as isize));
                    j += 1;
                }

                /* create a new moniker with the new string */
                res = CreateFileMoniker(newStr, ppmkComposite);

                /* free string memory used by this function */
                HeapFree(GetProcessHeap(), 0, newStr as LPVOID);
            } else {
                res = E_OUTOFMEMORY;
            }
        }

        free_stringtable(strDec1);
        free_stringtable(strDec2);

        CoTaskMemFree(str1 as LPVOID);
        CoTaskMemFree(str2 as LPVOID);

        res
    } else if crate::ring::antimoniker::is_anti_moniker(pmkRight, &mut order) {
        /* wine-10: an anti-moniker of order N cancels this file moniker, leaving N-1 */
        if order > 1 {
            crate::ring::antimoniker::create_anti_moniker(order - 1, ppmkComposite)
        } else {
            S_OK
        }
    } else if fOnlyIfNotGeneric != 0 {
        *ppmkComposite = core::ptr::null_mut();
        MK_E_NEEDGENERIC
    } else {
        crate::ring::compositemoniker::CreateGenericComposite(iface, pmkRight, ppmkComposite)
    }
}
}

stdmethod! {
fn FileMonikerImpl_Enum(
    iface: *mut IMoniker,
    fForward: BOOL,
    ppenumMoniker: *mut *mut IEnumMoniker,
) -> HRESULT {
    TRACE!(b"(%p,%d,%p)\n\0".as_ptr() as *const c_char, iface, fForward, ppenumMoniker);

    // Windows answers E_INVALIDARG here (moniker.c:2225, todo_wine in Wine, which
    // returns E_POINTER).
    if ppenumMoniker.is_null() {
        return E_INVALIDARG;
    }

    *ppenumMoniker = core::ptr::null_mut();

    S_OK
}
}

stdmethod! {
fn FileMonikerImpl_IsEqual(iface: *mut IMoniker, other: *mut IMoniker) -> HRESULT {
    let moniker = impl_from_IMoniker(iface);

    TRACE!(b"%p, %p.\n\0".as_ptr() as *const c_char, iface, other);

    if other.is_null() {
        return E_INVALIDARG;
    }

    /* wine-10: OUR file moniker by vtable, paths compared case-insensitively */
    if (*other).lpVtbl as *const IMonikerVtbl != core::ptr::addr_of!(VT_FileMonikerImpl) {
        return S_FALSE;
    }
    let other_moniker = impl_from_IMoniker(other);

    if crate::compobj::ffi::_wcsicmp((*moniker).filePathName, (*other_moniker).filePathName) == 0 {
        S_OK
    } else {
        S_FALSE
    }
}
}

stdmethod! {
fn FileMonikerImpl_Hash(iface: *mut IMoniker, pdwHash: *mut DWORD) -> HRESULT {
    let This = impl_from_IMoniker(iface);
    let mut h: c_int = 0;
    let mut i: c_int;
    let skip: c_int;
    let len: c_int;
    let mut off: c_int = 0;
    let val: LPOLESTR;

    if pdwHash.is_null() {
        return E_POINTER;
    }

    val = (*This).filePathName;
    len = lstrlenW(val);

    if len < 16 {
        i = len;
        while i > 0 {
            h = h.wrapping_mul(37).wrapping_add(*val.offset(off as isize) as c_int);
            off += 1;
            i -= 1;
        }
    } else {
        /* only sample some characters */
        skip = len / 8;
        i = len;
        while i > 0 {
            h = h.wrapping_mul(39).wrapping_add(*val.offset(off as isize) as c_int);
            i -= skip;
            off += skip;
        }
    }

    *pdwHash = h as DWORD;

    S_OK
}
}

stdmethod! {
fn FileMonikerImpl_IsRunning(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    pmkNewlyRunning: *mut IMoniker,
) -> HRESULT {
    let mut rot: *mut IRunningObjectTable = core::ptr::null_mut();
    let mut res: HRESULT;

    TRACE!(b"(%p,%p,%p,%p)\n\0".as_ptr() as *const c_char, iface, pbc, pmkToLeft,
           pmkNewlyRunning);

    if !pmkNewlyRunning.is_null() && IMoniker_IsEqual(pmkNewlyRunning, iface) == S_OK {
        return S_OK;
    }

    if pbc.is_null() {
        return E_POINTER;
    }

    res = IBindCtx_GetRunningObjectTable(pbc, &mut rot);

    if FAILED(res) {
        return res;
    }

    res = IRunningObjectTable_IsRunning(rot, iface);

    IRunningObjectTable_Release(rot);

    res
}
}

stdmethod! {
fn FileMonikerImpl_GetTimeOfLastChange(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    pFileTime: *mut FILETIME,
) -> HRESULT {
    let This = impl_from_IMoniker(iface);
    let mut rot: *mut IRunningObjectTable = core::ptr::null_mut();
    let mut res: HRESULT;
    let mut info: WIN32_FILE_ATTRIBUTE_DATA = core::mem::zeroed();

    TRACE!(b"(%p,%p,%p,%p)\n\0".as_ptr() as *const c_char, iface, pbc, pmkToLeft, pFileTime);

    if pFileTime.is_null() {
        return E_POINTER;
    }

    if !pmkToLeft.is_null() {
        return E_INVALIDARG;
    }

    res = IBindCtx_GetRunningObjectTable(pbc, &mut rot);

    if FAILED(res) {
        return res;
    }

    res = IRunningObjectTable_GetTimeOfLastChange(rot, iface, pFileTime);

    if FAILED(res) {
        /* the moniker is not registered */
        if GetFileAttributesExW(
            (*This).filePathName,
            GetFileExInfoStandard,
            core::ptr::addr_of_mut!(info) as LPVOID,
        ) == 0
        {
            return MK_E_NOOBJECT;
        }

        *pFileTime = info.ftLastWriteTime;
    }

    S_OK
}
}

stdmethod! {
fn FileMonikerImpl_Inverse(iface: *mut IMoniker, ppmk: *mut *mut IMoniker) -> HRESULT {
    TRACE!(b"(%p,%p)\n\0".as_ptr() as *const c_char, iface, ppmk);

    crate::ring::antimoniker::CreateAntiMoniker(ppmk)
}
}

/// filemoniker.c FileMonikerImpl_CommonPrefixWith's `failed:` label.
#[allow(clippy::too_many_arguments)]
unsafe fn common_prefix_cleanup(
    ret: HRESULT,
    bindctx: *mut IBindCtx,
    pathThis: LPOLESTR,
    pathOther: LPOLESTR,
    commonPath: LPOLESTR,
    stringTable1: *mut LPOLESTR,
    stringTable2: *mut LPOLESTR,
) -> HRESULT {
    IBindCtx_Release(bindctx);
    CoTaskMemFree(pathThis as LPVOID);
    CoTaskMemFree(pathOther as LPVOID);
    CoTaskMemFree(commonPath as LPVOID);
    if !stringTable1.is_null() {
        free_stringtable(stringTable1);
    }
    if !stringTable2.is_null() {
        free_stringtable(stringTable2);
    }

    ret
}

stdmethod! {
fn FileMonikerImpl_CommonPrefixWith(
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
    let nb1: c_int;
    let nb2: c_int;
    let mut i: c_int;
    let mut sameIdx: c_int;
    let mut machineNameCase: BOOL = 0;
    let mut ret: HRESULT;

    if ppmkPrefix.is_null() {
        return E_POINTER;
    }

    if pmkOther.is_null() {
        return E_INVALIDARG;
    }

    *ppmkPrefix = core::ptr::null_mut();

    /* check if we have the same type of moniker */
    IMoniker_IsSystemMoniker(pmkOther, &mut mkSys);
    if mkSys != MKSYS_FILEMONIKER {
        return MonikerCommonPrefixWith(iface, pmkOther, ppmkPrefix);
    }

    ret = CreateBindCtx(0, &mut bindctx);
    if FAILED(ret) {
        return ret;
    }

    /* create a string based on common part of the two paths */
    ret = IMoniker_GetDisplayName(iface, bindctx, core::ptr::null_mut(), &mut pathThis);
    if FAILED(ret) {
        return common_prefix_cleanup(ret, bindctx, pathThis, pathOther, commonPath,
                                     stringTable1, stringTable2);
    }

    ret = IMoniker_GetDisplayName(pmkOther, bindctx, core::ptr::null_mut(), &mut pathOther);
    if FAILED(ret) {
        return common_prefix_cleanup(ret, bindctx, pathThis, pathOther, commonPath,
                                     stringTable1, stringTable2);
    }

    nb1 = FileMonikerImpl_DecomposePath(pathThis, &mut stringTable1);
    if FAILED(nb1) {
        ret = nb1;
        return common_prefix_cleanup(ret, bindctx, pathThis, pathOther, commonPath,
                                     stringTable1, stringTable2);
    }

    nb2 = FileMonikerImpl_DecomposePath(pathOther, &mut stringTable2);
    if FAILED(nb2) {
        ret = nb2;
        return common_prefix_cleanup(ret, bindctx, pathThis, pathOther, commonPath,
                                     stringTable1, stringTable2);
    }

    if nb1 == 0 || nb2 == 0 {
        ret = MK_E_NOPREFIX;
        return common_prefix_cleanup(ret, bindctx, pathThis, pathOther, commonPath,
                                     stringTable1, stringTable2);
    }

    let len_this = lstrlenW(pathThis);
    let len_other = lstrlenW(pathOther);
    let min_len = if len_this < len_other { len_this } else { len_other };
    commonPath = CoTaskMemAlloc(
        (core::mem::size_of::<u16>() * (min_len as usize + 1)) as SIZE_T,
    ) as LPOLESTR;
    if commonPath.is_null() {
        ret = E_OUTOFMEMORY;
        return common_prefix_cleanup(ret, bindctx, pathThis, pathOther, commonPath,
                                     stringTable1, stringTable2);
    }

    *commonPath = 0;
    sameIdx = 0;
    while !(*stringTable1.offset(sameIdx as isize)).is_null()
        && !(*stringTable2.offset(sameIdx as isize)).is_null()
        && lstrcmpiW(*stringTable1.offset(sameIdx as isize),
                     *stringTable2.offset(sameIdx as isize)) == 0
    {
        sameIdx += 1;
    }

    if sameIdx > 1
        && **stringTable1.offset(0) == '\\' as u16
        && **stringTable2.offset(1) == '\\' as u16
    {
        machineNameCase = 1;

        i = 2;
        while i < sameIdx {
            if **stringTable1.offset(i as isize) == '\\' as u16
                && (i + 1 < sameIdx)
                && **stringTable1.offset((i + 1) as isize) == '\\' as u16
            {
                machineNameCase = 0;
                break;
            }
            i += 1;
        }
    }

    if machineNameCase != 0 && **stringTable1.offset((sameIdx - 1) as isize) == '\\' as u16 {
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

    common_prefix_cleanup(ret, bindctx, pathThis, pathOther, commonPath, stringTable1,
                          stringTable2)
}
}

/// filemoniker.c: `int FileMonikerImpl_DecomposePath(LPCOLESTR str, LPOLESTR **stringTable)`.
/// Non-static in the C (moniker.h declares it; moniker.c's MonikerCommonPrefixWith calls it).
#[no_mangle]
pub unsafe extern "C" fn FileMonikerImpl_DecomposePath(
    str: LPCOLESTR,
    stringTable: *mut *mut LPOLESTR,
) -> c_int {
    const B_SLASH: [u16; 2] = ['\\' as u16, 0];
    let word: LPOLESTR;
    let mut i: c_int = 0;
    let mut j: c_int;
    let mut tabIndex: c_int = 0;
    let mut ret: c_int = 0;
    let strgtable: *mut LPOLESTR;

    let len: c_int = lstrlenW(str);

    TRACE!(b"%s, %p\n\0".as_ptr() as *const c_char, debugstr_w(str as *const c_void),
           *stringTable);

    strgtable = CoTaskMemAlloc(
        ((len as usize + 1) * core::mem::size_of::<LPOLESTR>()) as SIZE_T,
    ) as *mut LPOLESTR;

    if strgtable.is_null() {
        return E_OUTOFMEMORY;
    }

    word = CoTaskMemAlloc(
        ((len as usize + 1) * core::mem::size_of::<u16>()) as SIZE_T,
    ) as LPOLESTR;

    if word.is_null() {
        ret = E_OUTOFMEMORY;
        return decompose_path_lend(ret, tabIndex, strgtable, word);
    }

    while *str.offset(i as isize) != 0 {
        if *str.offset(i as isize) == B_SLASH[0] {
            *strgtable.offset(tabIndex as isize) =
                CoTaskMemAlloc((2 * core::mem::size_of::<u16>()) as SIZE_T) as LPOLESTR;

            if (*strgtable.offset(tabIndex as isize)).is_null() {
                ret = E_OUTOFMEMORY;
                return decompose_path_lend(ret, tabIndex, strgtable, word);
            }

            lstrcpyW(*strgtable.offset(tabIndex as isize), B_SLASH.as_ptr());
            tabIndex += 1;

            i += 1;
        } else {
            j = 0;
            while *str.offset(i as isize) != 0 && *str.offset(i as isize) != B_SLASH[0] {
                *word.offset(j as isize) = *str.offset(i as isize);
                i += 1;
                j += 1;
            }

            *word.offset(j as isize) = 0;

            *strgtable.offset(tabIndex as isize) = CoTaskMemAlloc(
                (core::mem::size_of::<u16>() * (j + 1) as usize) as SIZE_T,
            ) as LPOLESTR;

            if (*strgtable.offset(tabIndex as isize)).is_null() {
                ret = E_OUTOFMEMORY;
                return decompose_path_lend(ret, tabIndex, strgtable, word);
            }

            lstrcpyW(*strgtable.offset(tabIndex as isize), word);
            tabIndex += 1;
        }
    }
    *strgtable.offset(tabIndex as isize) = core::ptr::null_mut();

    *stringTable = strgtable;

    ret = tabIndex;

    decompose_path_lend(ret, tabIndex, strgtable, word)
}

/// FileMonikerImpl_DecomposePath's `lend:` label.
unsafe fn decompose_path_lend(
    ret: c_int,
    tabIndex: c_int,
    strgtable: *mut LPOLESTR,
    word: LPOLESTR,
) -> c_int {
    if ret < 0 {
        let mut i: c_int = 0;
        while i < tabIndex {
            CoTaskMemFree(*strgtable.offset(i as isize) as LPVOID);
            i += 1;
        }

        CoTaskMemFree(strgtable as LPVOID);
    }

    CoTaskMemFree(word as LPVOID);

    ret
}

stdmethod! {
fn FileMonikerImpl_RelativePathTo(
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
    let relPath: LPOLESTR;
    let len1: c_int;
    let len2: c_int;
    let mut sameIdx: c_int = 0;
    let mut j: c_int;
    const BACK: [u16; 4] = ['.' as u16, '.' as u16, '\\' as u16, 0];

    TRACE!(b"(%p,%p,%p)\n\0".as_ptr() as *const c_char, iface, pmOther, ppmkRelPath);

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
    if FAILED(len1) {
        return E_OUTOFMEMORY;
    }
    len2 = FileMonikerImpl_DecomposePath(str2, &mut tabStr2);

    if FAILED(len2) {
        free_stringtable(tabStr1);
        return E_OUTOFMEMORY;
    }

    /* count the number of similar items from the begin of the two paths */
    while !(*tabStr1.offset(sameIdx as isize)).is_null()
        && !(*tabStr2.offset(sameIdx as isize)).is_null()
        && lstrcmpiW(*tabStr1.offset(sameIdx as isize), *tabStr2.offset(sameIdx as isize)) == 0
    {
        sameIdx += 1;
    }

    /* begin the construction of relativePath */
    /* if the two paths have a consecutive similar item from the begin ! the relativePath
     * will be composed by "..\\" in the begin */
    relPath = HeapAlloc(
        GetProcessHeap(),
        0,
        (core::mem::size_of::<u16>() * (1 + lstrlenW(str1) + lstrlenW(str2)) as usize) as SIZE_T,
    ) as LPOLESTR;

    *relPath = 0;

    if len2 > 0 && !(len1 == 1 && len2 == 1 && sameIdx == 0) {
        j = sameIdx;
        while !(*tabStr1.offset(j as isize)).is_null() {
            if **tabStr1.offset(j as isize) != '\\' as u16 {
                lstrcatW(relPath, BACK.as_ptr());
            }
            j += 1;
        }
    }

    /* add items of the second path (similar items with the first path are not included)
     * to the relativePath */
    j = sameIdx;
    while !(*tabStr2.offset(j as isize)).is_null() {
        lstrcatW(relPath, *tabStr2.offset(j as isize));
        j += 1;
    }

    res = CreateFileMoniker(relPath, ppmkRelPath);

    free_stringtable(tabStr1);
    free_stringtable(tabStr2);
    CoTaskMemFree(str1 as LPVOID);
    CoTaskMemFree(str2 as LPVOID);
    HeapFree(GetProcessHeap(), 0, relPath as LPVOID);

    if len1 == 0 || len2 == 0 || (len1 == 1 && len2 == 1 && sameIdx == 0) {
        return MK_S_HIM;
    }

    res
}
}

stdmethod! {
fn FileMonikerImpl_GetDisplayName(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    ppszDisplayName: *mut LPOLESTR,
) -> HRESULT {
    let This = impl_from_IMoniker(iface);
    let len: c_int = lstrlenW((*This).filePathName);

    TRACE!(b"(%p,%p,%p,%p)\n\0".as_ptr() as *const c_char, iface, pbc, pmkToLeft,
           ppszDisplayName);

    if ppszDisplayName.is_null() {
        return E_POINTER;
    }

    if !pmkToLeft.is_null() {
        return E_INVALIDARG;
    }

    *ppszDisplayName =
        CoTaskMemAlloc((core::mem::size_of::<u16>() * (len as usize + 1)) as SIZE_T) as LPOLESTR;
    if (*ppszDisplayName).is_null() {
        return E_OUTOFMEMORY;
    }

    lstrcpyW(*ppszDisplayName, (*This).filePathName);

    TRACE!(b"-- %s\n\0".as_ptr() as *const c_char,
           debugstr_w(*ppszDisplayName as *const c_void));

    S_OK
}
}

stdmethod! {
fn FileMonikerImpl_ParseDisplayName(
    iface: *mut IMoniker,
    pbc: *mut IBindCtx,
    pmkToLeft: *mut IMoniker,
    pszDisplayName: LPOLESTR,
    pchEaten: *mut ULONG,
    ppmkOut: *mut *mut IMoniker,
) -> HRESULT {
    FIXME!(b"(%p,%p,%p,%p,%p,%p),stub!\n\0".as_ptr() as *const c_char, iface, pbc, pmkToLeft,
           pszDisplayName, pchEaten, ppmkOut);

    E_NOTIMPL
}
}

stdmethod! {
fn FileMonikerImpl_IsSystemMoniker(iface: *mut IMoniker, pwdMksys: *mut DWORD) -> HRESULT {
    TRACE!(b"(%p,%p)\n\0".as_ptr() as *const c_char, iface, pwdMksys);

    if pwdMksys.is_null() {
        return E_POINTER;
    }

    *pwdMksys = MKSYS_FILEMONIKER;

    S_OK
}
}

// ---------------------------------------------------------------------------
// IROTData
// ---------------------------------------------------------------------------

stdmethod! {
fn FileMonikerROTDataImpl_QueryInterface(
    iface: *mut IROTData,
    riid: *const IID,
    ppvObject: *mut *mut c_void,
) -> HRESULT {
    let This = impl_from_IROTData(iface);

    TRACE!(b"(%p,%s,%p)\n\0".as_ptr() as *const c_char, This,
           debugstr_guid(riid as *const c_void), ppvObject);

    FileMonikerImpl_QueryInterface(core::ptr::addr_of_mut!((*This).IMoniker_iface), riid,
                                   ppvObject)
}
}

stdmethod! {
fn FileMonikerROTDataImpl_AddRef(iface: *mut IROTData) -> ULONG {
    let This = impl_from_IROTData(iface);

    TRACE!(b"(%p)\n\0".as_ptr() as *const c_char, This);

    IMoniker_AddRef(core::ptr::addr_of_mut!((*This).IMoniker_iface))
}
}

stdmethod! {
fn FileMonikerROTDataImpl_Release(iface: *mut IROTData) -> ULONG {
    let This = impl_from_IROTData(iface);

    TRACE!(b"(%p)\n\0".as_ptr() as *const c_char, This);

    FileMonikerImpl_Release(core::ptr::addr_of_mut!((*This).IMoniker_iface))
}
}

stdmethod! {
fn FileMonikerROTDataImpl_GetComparisonData(
    iface: *mut IROTData,
    pbData: *mut byte,
    cbMax: ULONG,
    pcbData: *mut ULONG,
) -> HRESULT {
    let This = impl_from_IROTData(iface);
    let len: c_int = lstrlenW((*This).filePathName) + 1;
    let mut i: c_int;
    let pszFileName: LPOLESTR;

    TRACE!(b"(%p, %u, %p)\n\0".as_ptr() as *const c_char, pbData, cbMax, pcbData);

    *pcbData = (core::mem::size_of::<CLSID>() + len as usize * core::mem::size_of::<u16>())
        as ULONG;
    if cbMax < *pcbData {
        return E_OUTOFMEMORY;
    }

    memcpy(
        pbData as *mut c_void,
        core::ptr::addr_of!(CLSID_FileMoniker) as *const c_void,
        core::mem::size_of::<CLSID>(),
    );
    pszFileName = pbData.add(core::mem::size_of::<CLSID>()) as LPOLESTR;
    i = 0;
    while i < len {
        *pszFileName.offset(i as isize) =
            towupper(*(*This).filePathName.offset(i as isize) as c_int) as u16;
        i += 1;
    }

    S_OK
}
}

// ---------------------------------------------------------------------------
// vtbls
// ---------------------------------------------------------------------------

/// Virtual function table for the FileMonikerImpl class which includes IPersist,
/// IPersistStream and IMoniker functions.
pub static VT_FileMonikerImpl: IMonikerVtbl = IMonikerVtbl {
    QueryInterface: Some(FileMonikerImpl_QueryInterface),
    AddRef: Some(FileMonikerImpl_AddRef),
    Release: Some(FileMonikerImpl_Release),
    GetClassID: Some(FileMonikerImpl_GetClassID),
    IsDirty: Some(FileMonikerImpl_IsDirty),
    Load: Some(FileMonikerImpl_Load),
    Save: Some(FileMonikerImpl_Save),
    GetSizeMax: Some(FileMonikerImpl_GetSizeMax),
    BindToObject: Some(FileMonikerImpl_BindToObject),
    BindToStorage: Some(FileMonikerImpl_BindToStorage),
    Reduce: Some(FileMonikerImpl_Reduce),
    ComposeWith: Some(FileMonikerImpl_ComposeWith),
    Enum: Some(FileMonikerImpl_Enum),
    IsEqual: Some(FileMonikerImpl_IsEqual),
    Hash: Some(FileMonikerImpl_Hash),
    IsRunning: Some(FileMonikerImpl_IsRunning),
    GetTimeOfLastChange: Some(FileMonikerImpl_GetTimeOfLastChange),
    Inverse: Some(FileMonikerImpl_Inverse),
    CommonPrefixWith: Some(FileMonikerImpl_CommonPrefixWith),
    RelativePathTo: Some(FileMonikerImpl_RelativePathTo),
    GetDisplayName: Some(FileMonikerImpl_GetDisplayName),
    ParseDisplayName: Some(FileMonikerImpl_ParseDisplayName),
    IsSystemMoniker: Some(FileMonikerImpl_IsSystemMoniker),
};

/// Virtual function table for the IROTData class.
pub static VT_ROTDataImpl: IROTDataVtbl = IROTDataVtbl {
    QueryInterface: Some(FileMonikerROTDataImpl_QueryInterface),
    AddRef: Some(FileMonikerROTDataImpl_AddRef),
    Release: Some(FileMonikerROTDataImpl_Release),
    GetComparisonData: Some(FileMonikerROTDataImpl_GetComparisonData),
};

// ---------------------------------------------------------------------------
// construction / factory
// ---------------------------------------------------------------------------

unsafe fn FileMonikerImpl_Construct(
    This: *mut FileMonikerImpl,
    lpszPathName: LPCOLESTR,
) -> HRESULT {
    let nb: c_int;
    let mut i: c_int;
    let mut sizeStr: c_int = lstrlenW(lpszPathName);
    let mut tabStr: *mut LPOLESTR = core::ptr::null_mut();
    const TWO_POINT: [u16; 3] = ['.' as u16, '.' as u16, 0];
    const BK_SLASH: [u16; 2] = ['\\' as u16, 0];
    let mut addBkSlash: BOOL;

    TRACE!(b"(%p,%s)\n\0".as_ptr() as *const c_char, This,
           debugstr_w(lpszPathName as *const c_void));

    /* Initialize the virtual function table. */
    (*This).IMoniker_iface.lpVtbl =
        core::ptr::addr_of!(VT_FileMonikerImpl) as *mut IMonikerVtbl;
    (*This).IROTData_iface.lpVtbl = core::ptr::addr_of!(VT_ROTDataImpl) as *mut IROTDataVtbl;
    (*This).ref_ = 0;
    (*This).pMarshal = core::ptr::null_mut();

    (*This).filePathName = HeapAlloc(
        GetProcessHeap(),
        0,
        (core::mem::size_of::<u16>() * (sizeStr as usize + 1)) as SIZE_T,
    ) as LPOLESTR;

    if (*This).filePathName.is_null() {
        return E_OUTOFMEMORY;
    }

    lstrcpyW((*This).filePathName, lpszPathName);

    nb = FileMonikerImpl_DecomposePath((*This).filePathName, &mut tabStr);

    if nb > 0 {
        addBkSlash = 1;
        if wcscmp(*tabStr.offset(0), TWO_POINT.as_ptr()) != 0 {
            addBkSlash = 0;
        } else {
            i = 0;
            while i < nb {
                if wcscmp(*tabStr.offset(i as isize), TWO_POINT.as_ptr()) != 0
                    && wcscmp(*tabStr.offset(i as isize), BK_SLASH.as_ptr()) != 0
                {
                    addBkSlash = 0;
                    break;
                } else if wcscmp(*tabStr.offset(i as isize), BK_SLASH.as_ptr()) == 0
                    && i < nb - 1
                    && wcscmp(*tabStr.offset((i + 1) as isize), BK_SLASH.as_ptr()) == 0
                {
                    **tabStr.offset(i as isize) = 0;
                    sizeStr -= 1;
                    addBkSlash = 0;
                    break;
                }
                i += 1;
            }
        }

        if wcscmp(*tabStr.offset((nb - 1) as isize), BK_SLASH.as_ptr()) == 0 {
            addBkSlash = 0;
        }

        (*This).filePathName = HeapReAlloc(
            GetProcessHeap(),
            0,
            (*This).filePathName as LPVOID,
            ((sizeStr as usize + 1) * core::mem::size_of::<u16>()) as SIZE_T,
        ) as LPOLESTR;

        *(*This).filePathName = 0;

        i = 0;
        while !(*tabStr.offset(i as isize)).is_null() {
            lstrcatW((*This).filePathName, *tabStr.offset(i as isize));
            i += 1;
        }

        if addBkSlash != 0 {
            lstrcatW((*This).filePathName, BK_SLASH.as_ptr());
        }
    }

    free_stringtable(tabStr);

    S_OK
}

/// CreateFileMoniker (OLE32.@)
#[no_mangle]
pub unsafe extern "system" fn CreateFileMoniker(
    lpszPathName: LPCOLESTR,
    ppmk: *mut *mut IMoniker,
) -> HRESULT {
    let newFileMoniker: *mut FileMonikerImpl;
    let mut hr: HRESULT;

    TRACE!(b"(%s,%p)\n\0".as_ptr() as *const c_char,
           debugstr_w(lpszPathName as *const c_void), ppmk);

    if ppmk.is_null() {
        return E_POINTER;
    }

    if lpszPathName.is_null() {
        return MK_E_SYNTAX;
    }

    *ppmk = core::ptr::null_mut();

    newFileMoniker = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<FileMonikerImpl>() as SIZE_T,
    ) as *mut FileMonikerImpl;

    if newFileMoniker.is_null() {
        return E_OUTOFMEMORY;
    }

    hr = FileMonikerImpl_Construct(newFileMoniker, lpszPathName);

    if SUCCEEDED(hr) {
        hr = IMoniker_QueryInterface(
            core::ptr::addr_of_mut!((*newFileMoniker).IMoniker_iface),
            core::ptr::addr_of!(IID_IMoniker),
            ppmk as *mut *mut c_void,
        );
    } else {
        HeapFree(GetProcessHeap(), 0, newFileMoniker as LPVOID);
    }

    hr
}

/// find a character from a set in reverse without the string having to be null-terminated
#[inline]
unsafe fn memrpbrkW(ptr: *const u16, n: usize, accept: *const u16) -> *mut u16 {
    let end = ptr.add(n);
    let mut ret: *const u16 = core::ptr::null();
    let mut p = ptr;
    while p < end {
        if !wcschr(accept, *p).is_null() {
            ret = p;
        }
        p = p.add(1);
    }
    ret as *mut u16
}

/// moniker.h: `HRESULT FileMoniker_CreateFromDisplayName(LPBC, LPCOLESTR, LPDWORD, IMoniker**)`
/// — a plain (cdecl) ole32-internal function, called from moniker.c's MkParseDisplayName.
#[no_mangle]
pub unsafe extern "C" fn FileMoniker_CreateFromDisplayName(
    pbc: *mut IBindCtx,
    szDisplayName: LPCOLESTR,
    pchEaten: LPDWORD,
    ppmk: *mut *mut IMoniker,
) -> HRESULT {
    const SEPARATORS: [u16; 5] = [':' as u16, '\\' as u16, '/' as u16, '!' as u16, 0];

    // The C is a for-loop whose FIRST iteration runs with `end` at the string end and
    // only then steps back to the previous separator:
    //   for (end = szDisplayName + lstrlenW(szDisplayName); end && (end != szDisplayName);
    //        end = memrpbrkW(szDisplayName, end - szDisplayName, wszSeparators))
    let mut end: *const u16 = szDisplayName.add(lstrlenW(szDisplayName) as usize);

    while !end.is_null() && end != szDisplayName {
        let hr: HRESULT;
        let mut rot: *mut IRunningObjectTable = core::ptr::null_mut();
        let mut file_moniker: *mut IMoniker = core::ptr::null_mut();
        let file_display_name: LPOLESTR;
        let full_path_name: LPOLESTR;
        let full_path_name_len: DWORD;
        let len: c_int = end.offset_from(szDisplayName) as c_int;

        file_display_name = HeapAlloc(
            GetProcessHeap(),
            0,
            ((len as usize + 1) * core::mem::size_of::<u16>()) as SIZE_T,
        ) as LPOLESTR;
        if file_display_name.is_null() {
            return E_OUTOFMEMORY;
        }
        memcpy(
            file_display_name as *mut c_void,
            szDisplayName as *const c_void,
            len as usize * core::mem::size_of::<u16>(),
        );
        *file_display_name.offset(len as isize) = 0;

        let mut hr_create = CreateFileMoniker(file_display_name, &mut file_moniker);
        if FAILED(hr_create) {
            HeapFree(GetProcessHeap(), 0, file_display_name as LPVOID);
            return hr_create;
        }

        hr_create = IBindCtx_GetRunningObjectTable(pbc, &mut rot);
        if FAILED(hr_create) {
            HeapFree(GetProcessHeap(), 0, file_display_name as LPVOID);
            IMoniker_Release(file_moniker);
            return hr_create;
        }

        hr = IRunningObjectTable_IsRunning(rot, file_moniker);
        IRunningObjectTable_Release(rot);
        if FAILED(hr) {
            HeapFree(GetProcessHeap(), 0, file_display_name as LPVOID);
            IMoniker_Release(file_moniker);
            return hr;
        }
        if hr == S_OK {
            TRACE!(b"found running file moniker for %s\n\0".as_ptr() as *const c_char,
                   debugstr_w(file_display_name as *const c_void));
            *pchEaten = len as DWORD;
            *ppmk = file_moniker;
            HeapFree(GetProcessHeap(), 0, file_display_name as LPVOID);
            return S_OK;
        }

        full_path_name_len =
            GetFullPathNameW(file_display_name, 0, core::ptr::null_mut(), core::ptr::null_mut());
        if full_path_name_len == 0 {
            HeapFree(GetProcessHeap(), 0, file_display_name as LPVOID);
            IMoniker_Release(file_moniker);
            return MK_E_SYNTAX;
        }
        full_path_name = HeapAlloc(
            GetProcessHeap(),
            0,
            (full_path_name_len as usize * core::mem::size_of::<u16>()) as SIZE_T,
        ) as LPOLESTR;
        if full_path_name.is_null() {
            HeapFree(GetProcessHeap(), 0, file_display_name as LPVOID);
            IMoniker_Release(file_moniker);
            return E_OUTOFMEMORY;
        }
        GetFullPathNameW(file_display_name, full_path_name_len, full_path_name,
                         core::ptr::null_mut());

        if GetFileAttributesW(full_path_name) == INVALID_FILE_ATTRIBUTES {
            TRACE!(b"couldn't open file %s\n\0".as_ptr() as *const c_char,
                   debugstr_w(full_path_name as *const c_void));
        } else {
            TRACE!(b"got file moniker for %s\n\0".as_ptr() as *const c_char,
                   debugstr_w(szDisplayName as *const c_void));
            *pchEaten = len as DWORD;
            *ppmk = file_moniker;
            HeapFree(GetProcessHeap(), 0, file_display_name as LPVOID);
            HeapFree(GetProcessHeap(), 0, full_path_name as LPVOID);
            return S_OK;
        }
        HeapFree(GetProcessHeap(), 0, file_display_name as LPVOID);
        HeapFree(GetProcessHeap(), 0, full_path_name as LPVOID);
        IMoniker_Release(file_moniker);

        /* the for-loop's increment: step back to the previous separator */
        end = memrpbrkW(szDisplayName, end.offset_from(szDisplayName) as usize,
                        SEPARATORS.as_ptr());
    }

    MK_E_CANTOPENFILE
}

#[no_mangle]
pub unsafe extern "system" fn FileMoniker_CreateInstance(
    iface: *mut IClassFactory,
    pUnk: *mut IUnknown,
    riid: *const IID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    let newFileMoniker: *mut FileMonikerImpl;
    let mut hr: HRESULT;
    const WSZ_EMPTY: [u16; 1] = [0];

    TRACE!(b"(%p, %s, %p)\n\0".as_ptr() as *const c_char, pUnk,
           debugstr_guid(riid as *const c_void), ppv);

    let _ = iface;

    *ppv = core::ptr::null_mut();

    if !pUnk.is_null() {
        return CLASS_E_NOAGGREGATION;
    }

    newFileMoniker = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<FileMonikerImpl>() as SIZE_T,
    ) as *mut FileMonikerImpl;
    if newFileMoniker.is_null() {
        return E_OUTOFMEMORY;
    }

    hr = FileMonikerImpl_Construct(newFileMoniker, WSZ_EMPTY.as_ptr());

    if SUCCEEDED(hr) {
        hr = IMoniker_QueryInterface(
            core::ptr::addr_of_mut!((*newFileMoniker).IMoniker_iface),
            riid,
            ppv,
        );
    }
    if FAILED(hr) {
        HeapFree(GetProcessHeap(), 0, newFileMoniker as LPVOID);
    }

    hr
}
