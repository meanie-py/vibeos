use crate::prelude::*;
use crate::zipfldr::CZipFolder_hpp_h::IsEqual__1;
use crate::zipfldr::zipfldr_cpp_h::g_FFunc;
use crate::zipfldr::zipfldr_cpp_h::g_ModuleRefCnt;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_8D71B3FAAB82: [WCHAR; 2] = [0x0020, 0x0000];
pub static CPP_COM_WSTR_96A296D224F2: [WCHAR; 1] = [0x0000];
pub static CPP_COM_WSTR_BCE941D196C2: [WCHAR; 2] = [0x002F, 0x0000];
pub static CPP_COM_WSTR_BF60CB94ADE1: [WCHAR; 4] = [0x005A, 0x0049, 0x0050, 0x0000];
pub static CPP_COM_WSTR_C5D4AD672797: [WCHAR; 5] = [0x0025, 0x0064, 0x0025, 0x0025, 0x0000];
pub static CPP_COM_WSTR_EB4CE2024A43: [WCHAR; 7] = [0x0046, 0x006F, 0x006C, 0x0064, 0x0065, 0x0072, 0x0000];

// --local-raii CDataObjectHIDA: hand-verified lowering (spliced verbatim; workers received this API as a sealed contract)
// --local-raii CDataObjectHIDA: hand-verified lowering (spliced verbatim; workers received this API as a sealed contract)
//
// THE canonical copy. Pass it as
//   rustconv translate <tu> --free-functions --local-raii CDataObjectHIDA=tools/rustconv/data/local_raii/CDataObjectHIDA.rs
// Extracted from shell32/shldataobject.rs, which is one of the three already-flipped TUs that
// each had to carry an identical copy (the others: shell32/shlfolder.rs, dialogs/item_prop.rs).
// A fourth, COpenWithMenu.cpp, stalled unpaid for want of it — the refusal is `translate`
// declining to guess drop semantics, and the fix was always a file, never a retry.
// `hida_clipformat()` comes from the crate prelude (shellutils.rs:68); this file assumes it.
// Hand-verified Rust lowering of the C++ RAII struct `CDataObjectHIDA`
// (sdk/include/reactos/shellutils.h:738). Consumed by
// `rustconv translate --free-functions --local-raii CDataObjectHIDA=<this file>`:
// spliced verbatim into the assembled module, and its API sealed into each
// affected worker prompt so the model maps C++ usage onto this fixed surface
// instead of inventing the drop semantics.
//
// It borrows the HIDA out of an IDataObject WITHOUT copying it, so the medium
// must stay locked for exactly the lifetime of the guard — which is the whole
// reason it is RAII and the whole reason a hand lowering is required.
//
// C++ semantics preserved exactly:
//   ctor        — CreateCIDA(pDataObject, &m_cida, m_medium); m_hr = result
//   dtor        — DestroyCIDA(m_cida, m_medium)                → impl Drop
//   hr()        — the stored ctor HRESULT
//   operator bool()        → is_valid()      (m_cida != nullptr)
//   operator const CIDA*() → as_ptr()        (the implicit conversion at a call
//                                             site such as `f(x, cida)`)
//   operator->()           → (*cida.as_ptr()).field
//
// Two details that look like bugs and are not, so do not "fix" them:
//   * DestroyCIDA calls ReleaseStgMedium unconditionally. On the failure paths
//     CreateCIDA has already set tymed = TYMED_NULL, which makes that call a
//     documented no-op — including after CreateCIDA itself released the medium.
//   * C++ leaves m_medium uninitialized and only assigns pUnkForRelease. Every
//     path assigns tymed before anything reads it (GetData on success,
//     TYMED_NULL otherwise). Zeroing here is strictly safer and changes no path.
struct CDataObjectHIDA {
    m_medium: STGMEDIUM,
    m_cida: *const CIDA,
    m_hr: HRESULT,
}

impl CDataObjectHIDA {
    unsafe fn new(pDataObject: *mut IDataObject) -> CDataObjectHIDA {
        let mut medium: STGMEDIUM = core::mem::zeroed();
        let mut cida: *const CIDA = core::ptr::null();
        let hr = CDataObjectHIDA::CreateCIDA(pDataObject, &mut cida, &mut medium);
        CDataObjectHIDA { m_medium: medium, m_cida: cida, m_hr: hr }
    }

    unsafe fn CreateCIDA(
        pDataObject: *mut IDataObject,
        ppcida: *mut *const CIDA,
        medium: *mut STGMEDIUM,
    ) -> HRESULT {
        *ppcida = core::ptr::null();
        (*medium).pUnkForRelease = core::ptr::null_mut();
        // g_cfHIDA in C++ is a DECLSPEC_SELECTANY cache of the registered format.
        let mut format = FORMATETC {
            cfFormat: hida_clipformat(),
            ptd: core::ptr::null_mut(),
            dwAspect: DVASPECT_CONTENT,
            lindex: -1,
            tymed: TYMED_HGLOBAL,
        };
        let hr = (((*(*pDataObject).lpVtbl).GetData).unwrap_unchecked())(
            pDataObject,
            (&mut format as *mut FORMATETC).cast(),
            medium.cast(),
        );
        if hr >= 0 {
            *ppcida = GlobalLock((*medium).__bindgen_anon_1.hGlobal) as *const CIDA;
            if !(*ppcida).is_null() {
                return S_OK;
            }
            ReleaseStgMedium(medium);
            (*medium).tymed = TYMED_NULL;
            return E_UNEXPECTED;
        }
        (*medium).tymed = TYMED_NULL;
        hr
    }

    /// Mirrors the C++ public static. It is not only the destructor's body:
    /// dll/shellext/cabview/folder.cpp calls CreateCIDA/DestroyCIDA directly to
    /// hold a CIDA across a scope the guard cannot span, so it must stay callable.
    /// shell32-rust's prelude re-exports it and move_to_menu.rs reaches it through
    /// CDataObjectHIDA_DestroyCIDA_RustBridge — a lowering without it links short.
    unsafe fn DestroyCIDA(pcida: *const CIDA, medium: *mut STGMEDIUM) {
        if !pcida.is_null() {
            GlobalUnlock((*medium).__bindgen_anon_1.hGlobal);
        }
        ReleaseStgMedium(medium);
    }

    unsafe fn hr(&self) -> HRESULT {
        self.m_hr
    }

    unsafe fn is_valid(&self) -> bool {
        !self.m_cida.is_null()
    }

    unsafe fn as_ptr(&self) -> *const CIDA {
        self.m_cida
    }
}

impl Drop for CDataObjectHIDA {
    fn drop(&mut self) {
        unsafe { CDataObjectHIDA::DestroyCIDA(self.m_cida, &mut self.m_medium) }
    }
}

use core::ffi::c_void;

extern "C" {
    fn strcmp(a: *const core::ffi::c_char, b: *const core::ffi::c_char) -> core::ffi::c_int;
}

// ---- file-scope preamble (translated) ----
pub static FmtIdZipFolder: GUID = GUID {
    Data1: 0xE88DCCE0,
    Data2: 0xB7B3,
    Data3: 0x11D1,
    Data4: [0xA9, 0xF0, 0x00, 0xAA, 0x00, 0x60, 0xFA, 0x31],
};









































unsafe extern "C" fn MapScidToColumn(scid: &SHCOLUMNID) -> i32 {
    let mut i: UINT = 0;
    while i < (_countof!(g_ColumnDefs) as UINT) {
        if IsEqual__1(scid, &g_ColumnDefs[i as usize]) {
            return i as i32;
        }
        i += 1;
    }
    return -(1 as i32);
}

unsafe extern "C" fn FindItem(List: &mut IEnumIDList, Path: PCWSTR) -> PIDLIST_RELATIVE {
    let mut pidl: PITEMID_CHILD = core::ptr::null_mut();
    while List.Next(1, core::ptr::addr_of_mut!(pidl), core::ptr::null_mut()) == S_OK {
        let p: *mut ZipPidlEntry = pidl as *mut ZipPidlEntry;
        if lstrcmpiW(core::ptr::addr_of!((*p).Name) as PCWSTR, Path) == 0 {
            return pidl;
        }
        CoTaskMemFree(pidl as LPVOID);
    }
    return core::ptr::null_mut();
}

impl CZipFolder {
    pub unsafe fn DeleteItems(&mut self, pDataObj: CComPtr<IDataObject>) -> HRESULT {
        let this_: *mut Self = self;
        let mut cida = CDataObjectHIDA::new(pDataObj.p);
        if !cida.is_valid() || (*cida.as_ptr()).cidl <= 0 {
            return E_FAIL;
        }

        // Get the target paths
        let mut targetPaths: CAtlList<CStringW> = CAtlList::new();
        let mut iFile: UINT = 0;
        while iFile < (*cida.as_ptr()).cidl {
            let pidlRelative: PCUIDLIST_RELATIVE = HIDA_GetPIDLItem(cida.as_ptr(), iFile);
            let pEntry: *const ZipPidlEntry = _ZipFromIL(pidlRelative);
            if !pEntry.is_null() {
                let mut fullPath: CStringW = (*this_).m_ZipDir.clone() + core::ptr::addr_of!((*pEntry).Name).cast();
                // For folders, end with a slash
                if (*pEntry).ZipType == ZIP_PIDL_DIRECTORY && fullPath.Right(1).Compare(CPP_COM_WSTR_BCE941D196C2.as_ptr()) != 0 {
                    fullPath += CPP_COM_WSTR_BCE941D196C2.as_ptr();
                }
                targetPaths.AddTail(fullPath);
            }
            iFile += 1;
        }

        // Create a temporary file
        let mut szTempPath: [WCHAR; (MAX_PATH) as _] = core::mem::zeroed();
        let mut szTempFile: [WCHAR; (MAX_PATH) as _] = core::mem::zeroed();
        GetTempPathW(MAX_PATH, szTempPath.as_mut_ptr());
        GetTempFileNameW(szTempPath.as_ptr(), CPP_COM_WSTR_BF60CB94ADE1.as_ptr(), 0, szTempFile.as_mut_ptr());

        // Close the current handle to work with the ZIP file
        (*this_).Close();

        let mut ffunc: zlib_filefunc64_def = core::mem::zeroed();
        fill_win32_filefunc64W(core::ptr::addr_of_mut!(ffunc));

        let mut hr: HRESULT = S_OK;
        let uf: unzFile = unzOpen2_64((*this_).m_ZipFile.GetString() as *const c_void, core::ptr::addr_of_mut!(ffunc));
        let zf: zipFile = zipOpen2_64((szTempFile.as_ptr()).cast(), (APPEND_STATUS_CREATE as i32), core::ptr::null_mut(), core::ptr::addr_of_mut!(ffunc));

        if uf.is_null() || zf.is_null() {
            DPRINT1!("Cannot open file\n");
            if !uf.is_null() {
                unzClose(uf);
            }
            if !zf.is_null() {
                zipClose(zf, core::ptr::null_mut());
            }
            return E_FAIL;
        }

        // Scan all entries in the original ZIP
        if unzGoToFirstFile(uf) == (UNZ_OK as i32) {
            'do_loop: loop {
                'body: {
                    // Read file entry
                    let mut info: unz_file_info64 = core::mem::zeroed();
                    let mut szNameA: [i8; (MAX_PATH) as _] = core::mem::zeroed();
                    if unzGetCurrentFileInfo64(
                        uf,
                        core::ptr::addr_of_mut!(info),
                        szNameA.as_mut_ptr(),
                        core::mem::size_of_val(&szNameA) as u32,
                        core::ptr::null_mut(),
                        0,
                        core::ptr::null_mut(),
                        0,
                    ) != (UNZ_OK as i32)
                    {
                        break 'body;
                    }

                    // Read extra field
                    let mut extra: CAtlArray<BYTE> = CAtlArray::new();
                    if info.size_file_extra > 0 {
                        extra.SetCount((info.size_file_extra) as _);
                        unzGetCurrentFileInfo64(
                            uf,
                            core::ptr::null_mut(),
                            core::ptr::null_mut(),
                            0,
                            (extra.GetData()).cast(),
                            info.size_file_extra,
                            core::ptr::null_mut(),
                            0,
                        );
                    }

                    let utf8Name: CStringA = CZipEnumerator::GetUtf8Name(szNameA.as_ptr(), extra.GetData(), extra.GetCount() as DWORD);
                    let mut currentEntryName: CStringW = CStringW::new();
                    if utf8Name.GetLength() > 0 {
                        currentEntryName = CA2WEX::<MAX_PATH>(utf8Name.GetString() as LPCSTR, CP_UTF8);
                    } else if (info.flag & MINIZIP_UTF8_FLAG) != 0 {
                        currentEntryName = CA2WEX::<MAX_PATH>(szNameA.as_ptr(), CP_UTF8);
                    } else {
                        currentEntryName = CA2WEX::<MAX_PATH>(szNameA.as_ptr(), CP_ACP);
                    }

                    currentEntryName.Replace(0x005Cu16, 0x002Fu16);

                    // Check if it is on the deletion target list
                    let mut bSkip: bool = false;
                    let mut pos: POSITION = targetPaths.GetHeadPosition();
                    while !pos.is_null() {
                        let target: &CStringW = &*targetPaths.GetNext(&mut pos);
                        // Check for an exact match (file) or a prefix match (folder)
                        if currentEntryName == *target || currentEntryName.Left(target.GetLength()) == *target {
                            bSkip = true;
                            break;
                        }
                    }

                    if !bSkip {
                        // If not to be deleted, copy to new ZIP
                        hr = (*this_).CopyZipEntry(uf, zf, core::ptr::addr_of_mut!(info), szNameA.as_ptr());
                        if FAILED_UNEXPECTEDLY!(hr) {
                            break 'do_loop;
                        }
                    }
                }
                if !(unzGoToNextFile(uf) == (UNZ_OK as i32)) {
                    break 'do_loop;
                }
            }
        }

        unzClose(uf);
        zipClose(zf, core::ptr::null_mut());

        // Replace the original file with the temporary file
        if hr >= 0
            && ReplaceFileW((*this_).m_ZipFile.GetString(), szTempFile.as_ptr(), core::ptr::null_mut(), 0, core::ptr::null_mut(), core::ptr::null_mut()) != 0
        {
            // Notify the shell that the folder contents have changed
            SHChangeNotify((SHCNE_UPDATEDIR) as LONG, SHCNF_IDLIST, (*this_).m_CurDir.m_pData as LPCVOID, core::ptr::null_mut());
        } else {
            DPRINT1!("Failed to replace file\n");
            DeleteFileW(szTempFile.as_ptr());
            hr = E_FAIL;
        }

        return hr;
    }
    pub unsafe fn DoDeleteItems(&mut self, pDataObj: CComPtr<IDataObject>) -> HRESULT {
        let this_: *mut Self = self;
        let mut message = CStringW::from_resource(IDS_CONFIRMDELETE_TEXT);
        let mut title = CStringW::from_resource(IDS_FRIENDLYNAME);
        if MessageBoxW((*this_).m_hwnd, message.GetString(), title.GetString(), MB_ICONWARNING | MB_YESNOCANCEL) != (IDYES as i32) {
            return S_FALSE;
        }

        let hr: HRESULT = (*this_).DeleteItems(pDataObj);
        if FAILED_UNEXPECTEDLY!(hr) {
            message.LoadStringW(IDS_CANTDELETEFILE);
            MessageBoxW((*this_).m_hwnd, message.GetString(), title.GetString(), MB_ICONERROR);
        }

        return hr;
    }
    pub unsafe fn CopyZipEntry(&mut self, uf: unzFile, zf: zipFile, info: *mut unz_file_info64, nameA: LPCSTR) -> HRESULT {
        let this_: *mut Self = self;
        // Get extra field
        let mut extra: CAtlArray<BYTE> = CAtlArray::new();
        if (*info).size_file_extra > 0 {
            extra.SetCount((*info).size_file_extra as usize);
            if unzGetCurrentFileInfo64(uf, core::ptr::null_mut(), core::ptr::null_mut(), 0, extra.GetData() as *mut core::ffi::c_void,
                                        (*info).size_file_extra, core::ptr::null_mut(), 0) != (UNZ_OK as i32) {
                DPRINT1!("Cannot get extra fields\n");
                return E_FAIL;
            }
        }

        if unzOpenCurrentFile(uf) != (UNZ_OK as i32) {
            DPRINT1!("Cannot open current file\n");
            return E_FAIL;
        }

        let mut zi: zip_fileinfo = core::mem::zeroed();
        zi.dosDate = (*info).dosDate;
        zi.internal_fa = (*info).internal_fa;
        zi.external_fa = (*info).external_fa;

        let err: INT = zipOpenNewFileInZip3_64(zf, nameA, core::ptr::addr_of!(zi),
                                          extra.GetData() as *const core::ffi::c_void, extra.GetCount() as UINT,
                                          extra.GetData() as *const core::ffi::c_void, extra.GetCount() as UINT,
                                          core::ptr::null_mut(), (Z_DEFLATED as i32), Z_DEFAULT_COMPRESSION, 0,
                                          -MAX_WBITS, (DEF_MEM_LEVEL as i32), (Z_DEFAULT_STRATEGY as i32),
                                          core::ptr::null_mut(), 0, ((*info).flag) as i32);
        if err != 0 {
            DPRINT1!("err: %d\n", err);
            unzCloseCurrentFile(uf);
            return E_FAIL;
        }

        let mut buffer = [0u8; 4096];
        let mut read: INT;
        loop {
            read = unzReadCurrentFile(uf, buffer.as_mut_ptr() as *mut core::ffi::c_void, core::mem::size_of_val(&buffer) as UINT);
            if !(read > 0) {
                break;
            }
            zipWriteInFileInZip(zf, buffer.as_mut_ptr() as *const core::ffi::c_void, read as UINT);
        }

        zipCloseFileInZip(zf);
        unzCloseCurrentFile(uf);
        return S_OK;
    }
    pub unsafe extern "system" fn s_ExtractProc(arg: LPVOID) -> DWORD {
        let mut ZipFile: CComBSTR = CComBSTR::new();
        ZipFile.Attach(arg as BSTR);

        _CZipExtract_runWizard(ZipFile.m_str);

        InterlockedDecrement(core::ptr::addr_of_mut!(g_ModuleRefCnt));
        return 0;
    }
    pub unsafe fn _GetFileTimeString(&mut self, lpFileTime: LPFILETIME, pwszResult: PWSTR, cchResult: UINT) -> BOOL {
        let this_: *mut Self = self;
        let mut st: SYSTEMTIME = core::mem::zeroed();

        if FileTimeToSystemTime(lpFileTime, core::ptr::addr_of_mut!(st)) == 0 {
            return (FALSE) as BOOL;
        }

        let mut cchRemaining: usize = cchResult as usize;
        let mut pwszEnd: PWSTR = pwszResult;
        let mut cchWritten: i32 = GetDateFormatW(LOCALE_USER_DEFAULT, DATE_SHORTDATE, core::ptr::addr_of!(st), core::ptr::null_mut(), pwszEnd, cchRemaining as i32);
        if cchWritten != 0 {
            cchWritten -= 1; // GetDateFormatW returns count with terminating zero
        } else {
            return (FALSE) as BOOL;
        }
        cchRemaining = cchRemaining.wrapping_sub(cchWritten as usize);
        pwszEnd = pwszEnd.offset(cchWritten as isize);

        StringCchCopyExW(pwszEnd, cchRemaining, CPP_COM_WSTR_8D71B3FAAB82.as_ptr(), core::ptr::addr_of_mut!(pwszEnd), core::ptr::addr_of_mut!(cchRemaining), 0);

        cchWritten = GetTimeFormatW(LOCALE_USER_DEFAULT, 0, core::ptr::addr_of!(st), core::ptr::null_mut(), pwszEnd, cchRemaining as i32);
        if cchWritten != 0 {
            cchWritten -= 1; // GetTimeFormatW returns count with terminating zero
        } else {
            return (FALSE) as BOOL;
        }

        return (TRUE) as BOOL;
    }
    pub unsafe extern "system" fn ZipFolderMenuCallback(psf: *mut IShellFolder, hwnd: HWND, pdtobj: *mut IDataObject, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> HRESULT {
        let pThis: *mut CZipFolder = psf as *mut CZipFolder;
        if pThis.is_null() {
            return E_FAIL;
        }

        (*pThis).m_pDataObj.assign(pdtobj);

        match uMsg {
            DFM_MERGECONTEXTMENU => {
                let mut spContextMenu: CComPtr<IContextMenu> = CComPtr::new(); // CComQIIDPtr<I_ID(IContextMenu)>(psf)
                (*psf).QueryInterface(&IID_IContextMenu, spContextMenu.GetAddressOf() as *mut *mut c_void);
                if spContextMenu.is_null() {
                    DPRINT1!("E_NOINTERFACE\n");
                    return E_NOINTERFACE;
                }

                let pqcminfo: *mut QCMINFO = lParam as *mut QCMINFO;
                let hr: HRESULT = (*spContextMenu.Get()).QueryContextMenu(
                    (*pqcminfo).hmenu,
                    (*pqcminfo).indexMenu,
                    (*pqcminfo).idCmdFirst,
                    (*pqcminfo).idCmdLast,
                    CMF_NORMAL,
                );
                if FAILED_UNEXPECTEDLY!(hr) {
                    return hr;
                }

                (*pqcminfo).idCmdFirst += HRESULT_CODE!(hr) as UINT;
                return S_OK;
            }
            DFM_INVOKECOMMANDEX => {
                return E_NOTIMPL;
            }
            DFM_INVOKECOMMAND => {
                if wParam == (DFM_CMD_DELETE) as _ {
                    return (*pThis).DoDeleteItems(CComPtr::from_addref(pdtobj)); // CComPtr<IDataObject>(pdtobj) AddRefs
                }

                let mut spContextMenu: CComPtr<IContextMenu> = CComPtr::new(); // CComQIIDPtr<I_ID(IContextMenu)>(psf)
                (*psf).QueryInterface(&IID_IContextMenu, spContextMenu.GetAddressOf() as *mut *mut c_void);
                if spContextMenu.is_null() {
                    DPRINT1!("E_NOINTERFACE\n");
                    return E_NOINTERFACE;
                }

                let mut ici: CMINVOKECOMMANDINFO = core::mem::zeroed();
                ici.cbSize = core::mem::size_of::<CMINVOKECOMMANDINFO>() as u32;
                ici.hwnd = hwnd;
                ici.lpVerb = wParam as LPSTR;
                ici.nShow = (SW_SHOWNORMAL as i32);

                return (*spContextMenu.Get()).InvokeCommand(core::ptr::addr_of_mut!(ici));
            }
            DFM_GETDEFSTATICID => {
                // Required for Windows 7 to pick a default
                return S_FALSE;
            }
            DFM_WM_INITMENUPOPUP => {
                // FIXME: Make it effective in `CDefViewBckgrndMenu`
                // Disable [Paste] / [Paste link] menu items
                EnableMenuItem(wParam as HMENU, FCIDM_SHVIEW_INSERT, MF_BYCOMMAND | MF_GRAYED);
                EnableMenuItem(wParam as HMENU, FCIDM_SHVIEW_INSERTLINK, MF_BYCOMMAND | MF_GRAYED);
            }
            _ => {}
        }
        return E_NOTIMPL;
    }
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        (*this_).m_UnzipFile = core::ptr::null_mut();
        (*this_).m_hwnd = core::ptr::null_mut();
        {
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        (*this_).Close();
    }
    pub unsafe fn Initialize__0(&mut self, zipFile: PCWSTR, zipDir: PCWSTR, curDir: PCUIDLIST_ABSOLUTE, pidl: PCUIDLIST_RELATIVE) -> HRESULT {
        let this_: *mut Self = self;
        (*this_).m_ZipFile = CStringW::from_ptr(zipFile);
        (*this_).m_ZipDir = CStringW::from_ptr(zipDir);

        (*this_).m_CurDir.Attach(ILCombine(curDir, pidl) as *mut ITEMIDLIST);
        return S_OK;
    }
    pub unsafe fn Close(&mut self) {
        let this_: *mut Self = self;
        if !(*this_).m_UnzipFile.is_null() {
            unzClose((*this_).m_UnzipFile);
        }
        (*this_).m_UnzipFile = core::ptr::null_mut();
    }
    pub unsafe fn getZip(&mut self) -> unzFile {
        let this_: *mut Self = self;
        if (*this_).m_UnzipFile.is_null() {
            (*this_).m_UnzipFile = unzOpen2_64(((*this_).m_ZipFile.GetString()).cast(), core::ptr::addr_of_mut!(g_FFunc));
        }
        return (*this_).m_UnzipFile;
    }
    pub unsafe fn GetDefaultColumnState(&mut self, iColumn: UINT, pcsFlags: *mut DWORD) -> HRESULT {
        let this_: *mut Self = self;
        if pcsFlags.is_null() || iColumn >= _countof!(g_ColumnDefs) {
            return E_INVALIDARG;
        }
        *pcsFlags = (g_ColumnDefs[iColumn as usize].ColumnFlags) as _;
        return S_OK;
    }
    pub unsafe fn GetDetailsEx(&mut self, pidl: PCUITEMID_CHILD, pscid: *const SHCOLUMNID, pv: *mut VARIANT) -> HRESULT {
        let this_: *mut Self = self;
        if pidl.is_null() || pscid.is_null() || pv.is_null() {
            return E_INVALIDARG;
        }

        V_VT!(pv) = VT_EMPTY;

        let curpidl: PCUIDLIST_RELATIVE = ILGetNext(pidl);
        if (*curpidl).mkid.cb != 0 {
            DPRINT1!("ERROR, unhandled PIDL!\n");
            return E_FAIL;
        }
        let zipEntry: *const ZipPidlEntry = _ZipFromIL(pidl);
        if zipEntry.is_null() {
            return E_INVALIDARG;
        }
        let isDir: bool = (*zipEntry.cast_mut()).IsDirectory();

        // Handle the non-string columns here so the caller gets the correct variant type
        if !isDir && IsEqual__1(&(*pscid), &g_ColumnDefs[(COL_SIZE) as usize]) != false {
            V_VT!(pv) = (VT_UI8 as u16);
            V_UI8!(pv) = (*zipEntry).UncompressedSize;
            return S_OK;
        } else if !isDir && IsEqual__1(&(*pscid), &g_ColumnDefs[(COL_COMPRSIZE) as usize]) != false {
            V_VT!(pv) = (VT_UI8 as u16);
            V_UI8!(pv) = (*zipEntry).CompressedSize;
            return S_OK;
        } else if !isDir && IsEqual__1(&(*pscid), &g_ColumnDefs[(COL_DATE_MOD) as usize]) != false {
            if DosDateTimeToVariantTime(HIWORD!((*zipEntry).DosDate), LOWORD!((*zipEntry).DosDate), &mut V_DATE!(pv) as *mut _) != 0 {
                V_VT!(pv) = (VT_DATE as u16);
                return S_OK;
            }
        }

        let mut hr: HRESULT = E_FAIL;
        let col: i32 = MapScidToColumn(&(*pscid));
        if col >= 0 {
            let mut sd: SHELLDETAILS = core::mem::zeroed();
            hr = (*this_).GetDetailsOf(pidl, col as UINT, core::ptr::addr_of_mut!(sd));
            if hr >= 0 {
                let mut str: CComHeapPtr<WCHAR> = CComHeapPtr::new();
                hr = StrRetToStrW(core::ptr::addr_of_mut!(sd.str_), pidl, str.GetAddressOf());
                if hr >= 0 {
                    let bstr = SysAllocString(str.op_to_LPCWSTR());
                    V_BSTR!(pv) = bstr;
                    hr = if (bstr).is_null() { E_OUTOFMEMORY } else { S_OK };
                    if hr >= 0 {
                        V_VT!(pv) = (VT_BSTR as u16);
                    }
                }
            }
        }
        return hr;
    }
    pub unsafe fn GetDetailsOf(&mut self, pidl: PCUITEMID_CHILD, iColumn: UINT, psd: *mut SHELLDETAILS) -> HRESULT {
        let this_: *mut Self = self;
        if iColumn >= _countof!(g_ColumnDefs) as UINT {
            return E_FAIL;
        }

        (*psd).cxChar = (g_ColumnDefs[iColumn as usize].cxChar) as _;
        (*psd).fmt = (g_ColumnDefs[iColumn as usize].fmt) as _;

        if pidl.is_null() {
            return SHSetStrRet!(core::ptr::addr_of_mut!((*psd).str_), _AtlBaseModule.GetResourceInstance(), g_ColumnDefs[iColumn as usize].iResource);
        }

        let curpidl: PCUIDLIST_RELATIVE = ILGetNext(pidl);
        if (*curpidl).mkid.cb != 0 {
            DPRINT1!("ERROR, unhandled PIDL!\n");
            return E_FAIL;
        }

        let zipEntry: *const ZipPidlEntry = _ZipFromIL(pidl);
        if zipEntry.is_null() {
            return E_INVALIDARG;
        }

        let mut Buffer: [WCHAR; 100] = [0u16; 100];
        let isDir: bool = (*zipEntry).ZipType == ZIP_PIDL_DIRECTORY;
        match (iColumn) as i32 {
            COL_NAME => {
                return (*this_).GetDisplayNameOf(pidl, SHGDN_INFOLDER, core::ptr::addr_of_mut!((*psd).str_));
            }
            COL_TYPE => {
                let mut shfi: SHFILEINFOW = core::mem::zeroed();
                let dwAttributes: DWORD = if isDir { FILE_ATTRIBUTE_DIRECTORY } else { FILE_ATTRIBUTE_NORMAL };
                let firet: ULONG_PTR = SHGetFileInfoW(core::ptr::addr_of!((*zipEntry).Name).cast(), dwAttributes, core::ptr::addr_of_mut!(shfi), (core::mem::size_of::<SHFILEINFOW>()) as UINT, SHGFI_USEFILEATTRIBUTES | SHGFI_TYPENAME);
                if firet == 0 {
                    return E_FAIL;
                }
                return SHSetStrRet!(core::ptr::addr_of_mut!((*psd).str_), core::ptr::addr_of_mut!(shfi.szTypeName).cast());
            }
            COL_COMPRSIZE | COL_SIZE => {
                if isDir {
                    return SHSetStrRet!(core::ptr::addr_of_mut!((*psd).str_), CPP_COM_WSTR_96A296D224F2.as_ptr());
                }

                let Size: ULONG64 = if iColumn == (COL_COMPRSIZE as UINT) { (*zipEntry).CompressedSize } else { (*zipEntry).UncompressedSize };
                if StrFormatByteSizeW((Size) as LONGLONG, Buffer.as_mut_ptr(), _countof!(Buffer)) == core::ptr::null_mut() {
                    return E_FAIL;
                }
                return SHSetStrRet!(core::ptr::addr_of_mut!((*psd).str_), Buffer.as_mut_ptr());
            }
            COL_PASSWORD => {
                if isDir {
                    return SHSetStrRet!(core::ptr::addr_of_mut!((*psd).str_), CPP_COM_WSTR_96A296D224F2.as_ptr());
                }
                return SHSetStrRet!(core::ptr::addr_of_mut!((*psd).str_), _AtlBaseModule.GetResourceInstance(), if (*zipEntry).Password != 0 { IDS_YES } else { IDS_NO });
            }
            COL_RATIO => {
                if isDir {
                    return SHSetStrRet!(core::ptr::addr_of_mut!((*psd).str_), CPP_COM_WSTR_96A296D224F2.as_ptr());
                }

                let mut ratio: i32 = 0;
                if (*zipEntry).UncompressedSize != 0 {
                    ratio = 100 - ((((*zipEntry).CompressedSize * 100) / (*zipEntry).UncompressedSize) as i32);
                }
                StringCchPrintfW!(Buffer.as_mut_ptr(), _countof!(Buffer), CPP_COM_WSTR_C5D4AD672797.as_ptr(), ratio);
                return SHSetStrRet!(core::ptr::addr_of_mut!((*psd).str_), Buffer.as_mut_ptr());
            }
            COL_DATE_MOD => {
                if isDir {
                    return SHSetStrRet!(core::ptr::addr_of_mut!((*psd).str_), CPP_COM_WSTR_96A296D224F2.as_ptr());
                }
                let mut ftLocal: FILETIME = core::mem::zeroed();
                DosDateTimeToFileTime(((*zipEntry).DosDate >> 16) as WORD, (*zipEntry).DosDate as WORD, core::ptr::addr_of_mut!(ftLocal));
                if (*this_)._GetFileTimeString(core::ptr::addr_of_mut!(ftLocal), Buffer.as_mut_ptr(), _countof!(Buffer) as UINT) == 0 {
                    return E_FAIL;
                }
                return SHSetStrRet!(core::ptr::addr_of_mut!((*psd).str_), Buffer.as_mut_ptr());
            }
            _ => {}
        }

        UNIMPLEMENTED!();
        return E_NOTIMPL;
    }
    pub unsafe fn MapColumnToSCID(&mut self, column: UINT, pscid: *mut SHCOLUMNID) -> HRESULT {
        let this_: *mut Self = self;
        if (column as usize) < (_countof!(g_ColumnDefs) as usize) && !g_ColumnDefs[column as usize].pkg.is_null() {
            (*pscid).fmtid = *g_ColumnDefs[column as usize].pkg;
            (*pscid).pid = (g_ColumnDefs[column as usize].pki) as _;
            return S_OK;
        }
        return E_FAIL;
    }
    pub unsafe fn ParseDisplayName(&mut self, hwndOwner: HWND, pbc: LPBC, lpszDisplayName: LPOLESTR, pchEaten: *mut ULONG, ppidl: *mut PIDLIST_RELATIVE, pdwAttributes: *mut ULONG) -> HRESULT {
        let this_: *mut Self = self;
        if !pchEaten.is_null() {
            *pchEaten = 0;
        }

        let dwFlags: DWORD = SHCONTF_FOLDERS | SHCONTF_NONFOLDERS | SHCONTF_INCLUDEHIDDEN | SHCONTF_INCLUDESUPERHIDDEN;
        let mut pEnum: CComPtr<IEnumIDList> = CComPtr::new();
        let hr: HRESULT = (*this_).EnumObjects(hwndOwner, dwFlags, pEnum.GetAddressOf());
        if hr < 0 {
            return hr;
        }

        let pidl: PIDLIST_RELATIVE = FindItem(&mut *pEnum.Get(), lpszDisplayName);
        if pidl.is_null() {
            return HRESULT_FROM_WIN32!(ERROR_FILE_NOT_FOUND);
        }

        *ppidl = pidl;
        if !pchEaten.is_null() {
            *pchEaten = lstrlenW(lpszDisplayName) as ULONG;
        }
        if !pdwAttributes.is_null() {
            (*this_).GetAttributesOf(1, ppidl as _, pdwAttributes as *mut DWORD);
        }
        return S_OK;
    }
    pub unsafe fn BindToObject(&mut self, pidl: PCUIDLIST_RELATIVE, pbcReserved: LPBC, riid: &IID, ppvOut: *mut LPVOID) -> HRESULT {
        let this_: *mut Self = self;
        if IsEqualGUID!(&(*riid), &(IID_IShellFolder)) {
            let mut newZipDir = CStringW::from_ptr((*this_).m_ZipDir.GetString());
            let mut curpidl = pidl;
            while (*curpidl).mkid.cb != 0 {
                let zipEntry: *const ZipPidlEntry = _ZipFromIL(curpidl);
                if zipEntry.is_null() {
                    return E_FAIL;
                }
                newZipDir += core::ptr::addr_of!((*zipEntry).Name).cast();
                newZipDir.AppendChar(0x002Fu16);

                curpidl = ILGetNext(curpidl);
            }
            return ShellObjectCreatorInit!(CZipFolder, Initialize__0, [(*this_).m_ZipFile.GetString(), newZipDir.GetString(), (*this_).m_CurDir.m_pData, pidl], riid, ppvOut);
        }
        DbgPrint(b"%s(%S) UNHANDLED\n\0".as_ptr() as *const i8, b"BindToObject\0".as_ptr() as *const i8, guid2string(riid));
        return E_NOTIMPL;
    }
    pub unsafe fn CompareIDs(&mut self, lParam: LPARAM, pidl1: PCUIDLIST_RELATIVE, pidl2: PCUIDLIST_RELATIVE) -> HRESULT {
        let this_: *mut Self = self;
        let zipEntry1: *const ZipPidlEntry = _ZipFromIL(pidl1);
        let zipEntry2: *const ZipPidlEntry = _ZipFromIL(pidl2);

        if zipEntry1.is_null() || zipEntry2.is_null() {
            return E_INVALIDARG;
        }

        let mut result: i32 = 0;
        if (*zipEntry1).ZipType != (*zipEntry2).ZipType {
            result = (*zipEntry1).ZipType as i32 - (*zipEntry2).ZipType as i32;
        } else {
            result = StrCmpIW(
                core::ptr::addr_of!((*zipEntry1).Name) as *const WCHAR,
                core::ptr::addr_of!((*zipEntry2).Name) as *const WCHAR,
            );
        }

        if result == 0 && (*zipEntry1).ZipType == ZIP_PIDL_DIRECTORY {
            let child1: PCUIDLIST_RELATIVE = ILGetNext(pidl1);
            let child2: PCUIDLIST_RELATIVE = ILGetNext(pidl2);

            if (*child1).mkid.cb != 0 && (*child2).mkid.cb != 0 {
                return (*this_).CompareIDs(lParam, child1, child2);
            } else if (*child1).mkid.cb != 0 {
                result = 1;
            } else if (*child2).mkid.cb != 0 {
                result = -1;
            }
        }

        return MAKE_COMPARE_HRESULT!(result);
    }
    pub unsafe fn CreateViewObject(&mut self, hwndOwner: HWND, riid: &IID, ppvOut: *mut LPVOID) -> HRESULT {
        let this_: *mut Self = self;
        (*this_).m_hwnd = if !hwndOwner.is_null() { hwndOwner } else { (*this_).m_hwnd };

        static UnknownIID: GUID = GUID {
            Data1: 0x93F81976,
            Data2: 0x6A0D,
            Data3: 0x42C3,
            Data4: [0x94, 0xDD, 0xAA, 0x25, 0x8A, 0x15, 0x54, 0x70],
        };
        if IsEqualGUID!(&(*riid), &(IID_IShellView)) {
            let mut sfvparams: SFV_CREATE = SFV_CREATE {
                cbSize: (core::mem::size_of::<SFV_CREATE>()) as _,
                pshf: core::ptr::addr_of_mut!((*this_).__base2) as *mut IShellFolder,
                ..core::mem::zeroed()
            };
            let mut pcb: CComPtr<IShellFolderViewCB> = CComPtr::new();

            let mut hr: HRESULT = _CFolderViewCB_CreateInstance(&IID_IShellFolderViewCB, pcb.GetAddressOf() as *mut *mut core::ffi::c_void);
            if FAILED_UNEXPECTEDLY!(hr) {
                return hr;
            }

            sfvparams.psfvcb = pcb.Get();
            hr = SHCreateShellFolderView(&mut sfvparams, ppvOut as *mut *mut IShellView);

            return hr;
        } else if IsEqualGUID!(&(*riid), &(IID_IExplorerCommandProvider)) {
            return _CExplorerCommandProvider_CreateInstance(core::ptr::addr_of_mut!((*this_).__base3), riid, ppvOut);
        } else if IsEqualGUID!(&(*riid), &(IID_IContextMenu)) {
            // Folder context menu
            return (*this_).QueryInterface(riid, ppvOut);
        } else if IsEqualGUID!(&(*riid), &(IID_IDropTarget)) {
            *ppvOut = core::ptr::addr_of_mut!((*this_).__base7) as LPVOID;
            (*this_).AddRef();
            return S_OK;
        }
        if !IsEqualGUID!(&(UnknownIID), &(*riid)) {
            DbgPrint(b"%s(%S) UNHANDLED\n\0".as_ptr() as *const i8, b"CreateViewObject\0".as_ptr() as *const i8, guid2string(riid));
        }
        return E_NOTIMPL;
    }
    pub unsafe fn GetAttributesOf(&mut self, cidl: UINT, apidl: PCUITEMID_CHILD_ARRAY, rgfInOut: *mut DWORD) -> HRESULT {
        let this_: *mut Self = self;
        if rgfInOut.is_null() || cidl == 0 || apidl.is_null() {
            return E_INVALIDARG;
        }

        *rgfInOut = 0;

        //static DWORD dwFileAttrs = SFGAO_STREAM | SFGAO_HASPROPSHEET | SFGAO_CANDELETE | SFGAO_CANCOPY | SFGAO_CANMOVE;
        //static DWORD dwFolderAttrs = SFGAO_FOLDER | SFGAO_DROPTARGET | SFGAO_HASPROPSHEET | SFGAO_CANDELETE | SFGAO_STORAGE | SFGAO_CANCOPY | SFGAO_CANMOVE;
        let dwFileAttrs: DWORD = SFGAO_CANDELETE | SFGAO_STREAM;
        let dwFolderAttrs: DWORD = SFGAO_CANDELETE | SFGAO_FOLDER | SFGAO_HASSUBFOLDER | SFGAO_BROWSABLE | SFGAO_DROPTARGET;

        let mut apidl = apidl;
        let mut cidl = cidl;
        while cidl > 0 && !(*apidl).is_null() {
            let zipEntry: *const ZipPidlEntry = _ZipFromIL(*apidl);

            if !zipEntry.is_null() {
                if (*zipEntry).ZipType == ZIP_PIDL_FILE {
                    *rgfInOut |= dwFileAttrs;
                } else {
                    *rgfInOut |= dwFolderAttrs;
                }
            } else {
                *rgfInOut = 0;
            }

            apidl = apidl.add(1);
            cidl -= 1;
        }

        *rgfInOut &= !(SFGAO_FILESYSTEM);
        *rgfInOut &= !(SFGAO_VALIDATE);
        return S_OK;
    }
    pub unsafe fn GetUIObjectOf(&mut self, hwndOwner: HWND, cidl: UINT, apidl: PCUITEMID_CHILD_ARRAY, riid: &IID, prgfInOut: *mut UINT, ppvOut: *mut LPVOID) -> HRESULT {
        let this_: *mut Self = self;
        (*this_).m_hwnd = if (hwndOwner as usize) != 0 { hwndOwner } else { (*this_).m_hwnd };
        if (IsEqualGUID!(&(*riid), &(IID_IExtractIconA)) || IsEqualGUID!(&(*riid), &(IID_IExtractIconW))) && cidl == 1 {
            let zipEntry: *const ZipPidlEntry = _ZipFromIL(*apidl);
            if !zipEntry.is_null() {
                let dwAttributes: DWORD = if (*zipEntry).ZipType == ZIP_PIDL_DIRECTORY { FILE_ATTRIBUTE_DIRECTORY } else { FILE_ATTRIBUTE_NORMAL };
                return SHCreateFileExtractIconW(core::ptr::addr_of!((*zipEntry).Name).cast(), dwAttributes, riid, ppvOut);
            }
        }
        else if IsEqualGUID!(&(*riid), &(IID_IContextMenu)) && cidl >= 0 {
            // Context menu of an object inside the zip
            let zipEntry: *const ZipPidlEntry = _ZipFromIL(*apidl);
            if !zipEntry.is_null() {
                let mut keys: [HKEY; 1] = [0 as HKEY; 1];
                let mut nkeys: i32 = 0;
                if (*zipEntry).ZipType == ZIP_PIDL_DIRECTORY {
                    let res: LSTATUS = RegOpenKeyExW(HKEY_CLASSES_ROOT, CPP_COM_WSTR_EB4CE2024A43.as_ptr(), 0, KEY_READ | KEY_QUERY_VALUE, keys.as_mut_ptr());
                    if res != (ERROR_SUCCESS as LSTATUS) {
                        return E_FAIL;
                    }
                    nkeys += 1;
                }
                return CDefFolderMenu_Create2(core::ptr::null_mut(), hwndOwner, cidl, apidl, core::ptr::addr_of_mut!((*this_).__base2) as *mut IShellFolder, Some(CZipFolder::ZipFolderMenuCallback), (nkeys) as UINT, keys.as_mut_ptr(), ppvOut as *mut *mut IContextMenu);
            }
        }
        else if IsEqualGUID!(&(*riid), &(IID_IDataObject)) && cidl >= 1 {
            return CIDLData_CreateFromIDArray((*this_).m_CurDir.m_pData, cidl, apidl, ppvOut as *mut *mut IDataObject);
        }
        else if IsEqualGUID!(&(*riid), &(IID_IDropTarget)) {
            (*this_).AddRef();
            *ppvOut = core::ptr::addr_of_mut!((*this_).__base7) as LPVOID;
            return S_OK;
        }

        DbgPrint(b"%s(%S) UNHANDLED\n\0".as_ptr() as *const i8, b"GetUIObjectOf\0".as_ptr() as *const i8, guid2string(riid));
        return E_NOINTERFACE;
    }
    pub unsafe fn GetDisplayNameOf(&mut self, pidl: PCUITEMID_CHILD, dwFlags: DWORD, strRet: LPSTRRET) -> HRESULT {
        let this_: *mut Self = self;
        if pidl.is_null() {
            return S_FALSE;
        }

        let curpidl: PCUIDLIST_RELATIVE = ILGetNext(pidl);
        if (*curpidl).mkid.cb != 0 {
            DPRINT1!("ERROR, unhandled PIDL!\n");
            return E_FAIL;
        }

        let zipEntry: *const ZipPidlEntry = _ZipFromIL(pidl);
        if zipEntry.is_null() {
            return E_FAIL;
        }

        if dwFlags & SHGDN_FORPARSING != 0 {
            if dwFlags & SHGDN_INFOLDER != 0 {
                return SHSetStrRet!(strRet, core::ptr::addr_of!((*zipEntry).Name) as *const WCHAR);
            }

            let mut parent: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
            if SHGetPathFromIDListW((*this_).m_CurDir.Get(), parent.as_mut_ptr()) == 0 {
                return E_FAIL;
            }
            let cch: UINT = lstrlenW(parent.as_ptr()) as UINT + 1 + lstrlenW(core::ptr::addr_of!((*zipEntry).Name).cast()) as UINT + 1;
            (*strRet).uType = STRRET_WSTR;
            (*strRet).__bindgen_anon_1.pOleStr = SHAlloc((cch as usize * core::mem::size_of::<WCHAR>()) as SIZE_T) as LPWSTR;
            if (*strRet).__bindgen_anon_1.pOleStr.is_null() {
                return E_OUTOFMEMORY;
            }
            lstrcpyW((*strRet).__bindgen_anon_1.pOleStr, parent.as_ptr());
            PathAppendW((*strRet).__bindgen_anon_1.pOleStr, core::ptr::addr_of!((*zipEntry).Name).cast());
            return S_OK;
        }

        let mut fi: SHFILEINFOW = core::mem::zeroed();
        let attr: DWORD = if (*zipEntry.cast_mut()).IsDirectory() { FILE_ATTRIBUTE_DIRECTORY } else { 0 };
        if SHGetFileInfoW(core::ptr::addr_of!((*zipEntry).Name).cast(), attr, core::ptr::addr_of_mut!(fi), core::mem::size_of::<SHFILEINFOW>() as UINT, SHGFI_DISPLAYNAME | SHGFI_USEFILEATTRIBUTES) != 0 {
            return SHSetStrRet!(strRet, core::ptr::addr_of!(fi.szDisplayName) as *const WCHAR);
        }
        return SHSetStrRet!(strRet, core::ptr::addr_of!((*zipEntry).Name) as *const WCHAR);
    }
    pub unsafe fn GetCommandString(&mut self, idCmd: UINT_PTR, uFlags: UINT, pwReserved: *mut UINT, pszName: LPSTR, cchMax: UINT) -> HRESULT {
        let this_: *mut Self = self;
        if idCmd != 0 {
            return E_INVALIDARG;
        }

        match uFlags {
            GCS_VERBA => {
                return StringCchCopyA(pszName, (cchMax) as _, EXTRACT_VERBA.as_ptr() as *const i8);
            }
            GCS_VERBW => {
                return StringCchCopyW(pszName as PWSTR, (cchMax) as _, EXTRACT_VERBW.as_ptr());
            }
            GCS_HELPTEXTA => {
                let mut helpText = CStringA::from_ptr(MAKEINTRESOURCEA!(IDS_HELPTEXT));
                return StringCchCopyA(pszName, (cchMax) as _, (helpText.GetString()).cast());
            }
            GCS_HELPTEXTW => {
                let mut helpText = CStringW::from_ansi(MAKEINTRESOURCEA!(IDS_HELPTEXT));
                return StringCchCopyW(pszName as PWSTR, (cchMax) as _, helpText.GetString());
            }
            GCS_VALIDATEA | GCS_VALIDATEW => {
                return S_OK;
            }
            _ => {}
        }

        return E_INVALIDARG;
    }
    pub unsafe fn InvokeCommand(&mut self, pici: LPCMINVOKECOMMANDINFO) -> HRESULT {
        let this_: *mut Self = self;
        if pici.is_null() || ((*pici).cbSize != core::mem::size_of::<CMINVOKECOMMANDINFO>() as DWORD && (*pici).cbSize != core::mem::size_of::<CMINVOKECOMMANDINFOEX>() as DWORD) {
            return E_INVALIDARG;
        }

        if (*pici).lpVerb == MAKEINTRESOURCEA!(0)
            || (IS_INTRESOURCE!((*pici).lpVerb) == false && strcmp((*pici).lpVerb, EXTRACT_VERBA.as_ptr() as *const i8) == 0)
        {
            let ZipFile: BSTR = (*this_).m_ZipFile.AllocSysString();
            InterlockedIncrement(core::ptr::addr_of_mut!(g_ModuleRefCnt));

            let mut tid: DWORD = 0;
            let hThread: HANDLE = CreateThread(
                core::ptr::null_mut(),
                0,
                Some(CZipFolder::s_ExtractProc),
                ZipFile as PVOID,
                0,
                core::ptr::addr_of_mut!(tid),
            );
            if !hThread.is_null() {
                CloseHandle(hThread);
                return S_OK;
            }
        }

        if (*pici).lpVerb == MAKEINTRESOURCEA!(DFM_CMD_DELETE)
            || (IS_INTRESOURCE!((*pici).lpVerb) == false && strcmp((*pici).lpVerb, b"delete\0".as_ptr() as *const i8) == 0)
        {
            // C passes CComPtr<IDataObject> by value: a copy, i.e. its own reference.
            return (*this_).DoDeleteItems(CComPtr::from_addref((*this_).m_pDataObj.p));
        }

        E_INVALIDARG
    }
    pub unsafe fn QueryContextMenu(&mut self, hmenu: HMENU, mut indexMenu: UINT, idCmdFirst: UINT, idCmdLast: UINT, uFlags: UINT) -> HRESULT {
        let this_: *mut Self = self;
        let mut idCmd: UINT = idCmdFirst;

        if (uFlags & CMF_DEFAULTONLY) == 0 {
            let mut menuText = CStringW::from_ptr(MAKEINTRESOURCEW!(IDS_MENUITEM));

            if indexMenu != 0 {
                InsertMenuW(hmenu, { let t = indexMenu; indexMenu += 1; t }, MF_BYPOSITION | MF_SEPARATOR, 0, core::ptr::null_mut());
            }
            InsertMenuW(hmenu, { let t = indexMenu; indexMenu += 1; t }, MF_BYPOSITION | MF_STRING, { let t = idCmd; idCmd += 1; (t) as _ }, menuText.GetString()); // Command 0
        }

        return MAKE_HRESULT!(SEVERITY_SUCCESS, FACILITY_NULL, idCmd.wrapping_sub(idCmdFirst));
    }
    pub unsafe fn Initialize__1(&mut self, pidlFolder: PCIDLIST_ABSOLUTE, pDataObj: LPDATAOBJECT, hkeyProgID: HKEY) -> HRESULT {
        let this_: *mut Self = self;
        let mut etc: FORMATETC = FORMATETC {
            cfFormat: CF_HDROP as u16,
            ptd: core::ptr::null_mut(),
            dwAspect: DVASPECT_CONTENT as u32,
            lindex: -(1 as i32),
            tymed: TYMED_HGLOBAL as u32,
        };
        let mut stg: STGMEDIUM = core::mem::zeroed();

        let mut hr: HRESULT = (*pDataObj).GetData(core::ptr::addr_of_mut!(etc), core::ptr::addr_of_mut!(stg));
        if FAILED_UNEXPECTEDLY!(hr) {
            return hr;
        }

        hr = E_FAIL;
        let hdrop: HDROP = GlobalLock(stg.__bindgen_anon_1.hGlobal) as HDROP; // COMMENT_TOKEN_ECHO____
        if !hdrop.is_null() {
            let uNumFiles: UINT = DragQueryFileW(hdrop, 0xFFFFFFFF as UINT, core::ptr::null_mut(), 0 as UINT); // COMMENT_TOKEN_ECHO____
            if uNumFiles == 1 as UINT {
                let mut szFile: [u16; (MAX_PATH * 2) as usize] = core::mem::zeroed();
                if DragQueryFileW(hdrop, 0 as UINT, szFile.as_mut_ptr(), _countof!(szFile)) != 0 { // COMMENT_TOKEN_ECHO____
                    let mut pidl: CComHeapPtr<ITEMIDLIST> = CComHeapPtr::new();
                    hr = SHParseDisplayName(szFile.as_ptr(), core::ptr::null_mut(), pidl.GetAddressOf(), 0 as u32, core::ptr::null_mut()); // COMMENT_TOKEN_ECHO____
                    if !FAILED_UNEXPECTEDLY!(hr) {
                        hr = (*this_).Initialize__2(pidl.Get());
                    }
                } else {
                    DbgPrint(b"Failed to query the file.\r\n\0".as_ptr() as *const i8);
                }
            } else {
                DbgPrint(b"Invalid number of files: %d\r\n\0".as_ptr() as *const i8, uNumFiles);
            }
            GlobalUnlock(stg.__bindgen_anon_1.hGlobal); // COMMENT_TOKEN_ECHO____
        } else {
            DbgPrint(b"Could not lock stg.hGlobal\r\n\0".as_ptr() as *const i8);
        }
        ReleaseStgMedium(core::ptr::addr_of_mut!(stg)); // COMMENT_TOKEN_ECHO____
        return hr;
    }
    pub unsafe fn IsDirty(&mut self) -> HRESULT {
        let this_: *mut Self = self;
        S_FALSE
    }
    pub unsafe fn Load(&mut self, pszFileName: LPCOLESTR, dwMode: DWORD) -> HRESULT {
        let this_: *mut Self = self;
        (*this_).m_ZipFile = CStringW::from_ptr(pszFileName);

        let mut pidl: CComHeapPtr<ITEMIDLIST> = CComHeapPtr::new();
        let hr: HRESULT = SHParseDisplayName(pszFileName, core::ptr::null_mut(), pidl.put(), 0, core::ptr::null_mut());
        if ((hr) >= 0)
        {
            (*this_).m_CurDir.Attach(pidl.Detach());
        }

        return S_OK;
    }
    pub unsafe fn Save(&mut self, pszFileName: LPCOLESTR,
        fRemember: BOOL,
    ) -> HRESULT {
        let this_: *mut Self = self;
        E_NOTIMPL
    }
    pub unsafe fn SaveCompleted(&mut self, pszFileName: LPCOLESTR,
    ) -> HRESULT {
        let this_: *mut Self = self;
        S_OK
    }
    pub unsafe fn GetCurFile(&mut self, ppszFileName: *mut LPOLESTR) -> HRESULT {
        let this_: *mut Self = self;
        if ppszFileName.is_null() {
            return E_INVALIDARG;
        }

        *ppszFileName = core::ptr::null_mut();

        if (*this_).m_ZipFile.IsEmpty() {
            return S_FALSE;
        }

        *ppszFileName = CoTaskMemAlloc(
            ((((*this_).m_ZipFile.GetLength() + 1) as usize)
                * core::mem::size_of::<WCHAR>()) as SIZE_T,
        ) as LPOLESTR;
        if (*ppszFileName).is_null() {
            return E_OUTOFMEMORY;
        }

        wcscpy(*ppszFileName, (*this_).m_ZipFile.GetString() as *mut WCHAR);
        return S_OK;
    }
    pub unsafe fn Initialize__2(&mut self, pidl: PCIDLIST_ABSOLUTE) -> HRESULT {
        let this_: *mut Self = self;
        let mut tmpPath: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

        if SHGetPathFromIDListW(pidl, tmpPath.as_mut_ptr()) != 0 {
            (*this_).m_ZipFile = CStringW::from_ptr(tmpPath.as_ptr());
            (*this_).m_CurDir.Attach(ILClone(pidl));
            return S_OK;
        }
        DbgPrint(b"%s() => Unable to parse pidl\n\0".as_ptr() as *const i8, b"Initialize\0".as_ptr() as *const i8);
        return E_INVALIDARG;
    }
    pub unsafe fn DragEnter(&mut self, pDataObj: *mut IDataObject, grfKeyState: DWORD, pt: POINTL, pdwEffect: *mut DWORD) -> HRESULT {
        let this_: *mut Self = self;
        let _ = (this_, pDataObj, grfKeyState, pt);
        *pdwEffect &= DROPEFFECT_COPY;
        return S_OK;
    }
    pub unsafe fn DragOver(&mut self, grfKeyState: DWORD, pt: POINTL, pdwEffect: *mut DWORD) -> HRESULT {
        let this_: *mut Self = self;
        *pdwEffect &= DROPEFFECT_COPY;
        return S_OK;
    }
    pub unsafe fn DragLeave(&mut self) -> HRESULT {
        let this_: *mut Self = self;
        S_OK
    }
    pub unsafe fn Drop(&mut self, pDataObj: *mut IDataObject, grfKeyState: DWORD, pt: POINTL, pdwEffect: *mut DWORD) -> HRESULT {
        let this_: *mut Self = self;
        let mut sm: STGMEDIUM = core::mem::zeroed();
        let mut fe: FORMATETC = FORMATETC { cfFormat: (CF_HDROP as u16), ptd: core::ptr::null_mut(), dwAspect: DVASPECT_CONTENT, lindex: -1, tymed: TYMED_HGLOBAL };
        let mut hr: HRESULT = (*pDataObj).GetData(core::ptr::addr_of_mut!(fe), core::ptr::addr_of_mut!(sm));
        if FAILED_UNEXPECTEDLY!(hr) {
            return hr;
        }

        let mut hDrop: HDROP = GlobalLock(sm.__bindgen_anon_1.hGlobal) as HDROP;
        if !hDrop.is_null() {
            // Close the ZIP file before appending (it will be automatically
            // reopened next time getZip() is called)
            (*this_).Close();

            // Create creator
            let mut pCreator: *mut CZipCreator = CZipCreator::DoCreate((*this_).m_ZipFile.GetString(), (*this_).m_ZipDir.GetString());

            (*pCreator).SetNotifyPidl((*this_).m_CurDir.Get());

            // Add dropped files
            let mut fileCount: UINT = DragQueryFileW(hDrop, 0xFFFFFFFF, core::ptr::null_mut(), 0);
            let mut i: UINT = 0;
            while i < fileCount {
                let mut szFilePath: [WCHAR; (MAX_PATH) as _] = core::mem::zeroed();
                DragQueryFileW(hDrop, i, szFilePath.as_mut_ptr(), MAX_PATH);
                (*pCreator).DoAddItem(szFilePath.as_ptr());
                i += 1;
            }

            CZipCreator::runThread(pCreator);

            GlobalUnlock(sm.__bindgen_anon_1.hGlobal);
            *pdwEffect = DROPEFFECT_COPY;
        }
        ReleaseStgMedium(core::ptr::addr_of_mut!(sm));

        return S_OK;
    }
}
