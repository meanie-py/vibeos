use crate::prelude::*;
use crate::zipfldr::zipfldr_cpp_h::g_ModuleRefCnt;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_02D2E571821E: [WCHAR; 5] = [0x002E, 0x007A, 0x0069, 0x0070, 0x0000];
pub static CPP_COM_WSTR_1420FA6867AD: [WCHAR; 10] = [0x0020, 0x0028, 0x0025, 0x0075, 0x0029, 0x002E, 0x007A, 0x0069, 0x0070, 0x0000];
pub static CPP_COM_WSTR_77ECA80E282A: [WCHAR; 3] = [0x002E, 0x002E, 0x0000];
pub static CPP_COM_WSTR_C245AFF91259: [WCHAR; 2] = [0x002E, 0x0000];
pub static CPP_COM_WSTR_E8A4B2EE7EDE: [WCHAR; 2] = [0x002A, 0x0000];

use core::ffi::c_void;

extern "C" {
    // zlib (linked static) and msvcrt; outside the island.
    fn crc32(crc: uLong, buf: *const u8, len: uInt) -> uLong;
    fn strlen(s: *const core::ffi::c_char) -> usize;
}

unsafe extern "C" fn CalculateNameCRC32(name: PCSTR) -> DWORD {
    return crc32(0, name as *const u8, strlen(name) as uInt) as DWORD;
}

unsafe fn DoGetZipName(filename: PCWSTR) -> CStringW {
    let mut szPath = [0u16; (MAX_PATH) as _];
    StringCbCopyW(szPath.as_mut_ptr(), core::mem::size_of::<[u16; (MAX_PATH) as _]>(), filename);
    PathRemoveExtensionW(szPath.as_mut_ptr());

    let mut ret = CStringW::from_ptr(szPath.as_ptr());
    ret += CPP_COM_WSTR_02D2E571821E.as_ptr();

    let mut i: u32 = 2;
    while PathFileExistsW(ret.GetString()) != 0 {
        let mut str = CStringW::new();
        let v: u32 = i;
        i += 1;
        atl_format!(str, CPP_COM_WSTR_1420FA6867AD.as_ptr(), v);

        ret = CStringW::from_ptr(szPath.as_ptr());
        ret += str.GetString();
    }

    return ret;
}

unsafe fn DoGetBaseName(filename: PCWSTR) -> CStringW {
    let mut szBaseName: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    StringCbCopyW(szBaseName.as_mut_ptr(), core::mem::size_of_val(&szBaseName), filename);
    PathRemoveFileSpecW(szBaseName.as_mut_ptr());
    PathAddBackslashW(szBaseName.as_mut_ptr());
    return CStringW::from_ptr(szBaseName.as_ptr());
}

unsafe fn DoGetNameInZip(basename: &CStringW, filename: &CStringW, nCodePage: UINT) -> CStringA {
    let mut basenameI = CStringW::new();
    basenameI.SetString(basename.GetString(), basename.GetLength());
    let mut filenameI = CStringW::new();
    filenameI.SetString(filename.GetString(), filename.GetLength());
    basenameI.MakeUpper();
    filenameI.MakeUpper();

    let mut ret = CStringW::new();
    if filenameI.FindStr(basenameI.GetString(), 0) == 0 {
        ret = filename.Mid(basename.GetLength());
    } else {
        ret.SetString(filename.GetString(), filename.GetLength());
    }

    ret.Replace(0x005Cu16 as WCHAR, 0x002Fu16 as WCHAR);

    return CW2AEX::<MAX_PATH>((&ret).GetString(), nCodePage);
}

unsafe extern "C" fn DoGetFileTimeInfo(hFile: HANDLE, pzi: *mut zip_fileinfo) -> BOOL {
    let mut ft: FILETIME = core::mem::zeroed();
    let mut ftLocal: FILETIME = core::mem::zeroed();
    ZeroMemory!(pzi as *mut core::ffi::c_void, core::mem::size_of::<zip_fileinfo>());
    if GetFileTime(hFile, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::addr_of_mut!(ft)) != 0 {
        let mut st: SYSTEMTIME = core::mem::zeroed();
        FileTimeToLocalFileTime(core::ptr::addr_of_mut!(ft), core::ptr::addr_of_mut!(ftLocal));
        FileTimeToSystemTime(core::ptr::addr_of_mut!(ftLocal), core::ptr::addr_of_mut!(st));
        (*pzi).tmz_date.tm_sec = st.wSecond as _;
        (*pzi).tmz_date.tm_min = st.wMinute as _;
        (*pzi).tmz_date.tm_hour = st.wHour as _;
        (*pzi).tmz_date.tm_mday = st.wDay as _;
        (*pzi).tmz_date.tm_mon = (st.wMonth - 1) as _;
        (*pzi).tmz_date.tm_year = st.wYear as _;
        return (TRUE) as BOOL;
    }
    return (FALSE) as BOOL;
}

unsafe fn DoAddFilesFromItem(files: &mut CSimpleArray<CStringW>, item: PCWSTR) {
    if PathIsDirectoryW(item) == 0 {
        files.Add(CStringW::from_ptr(item));
        return;
    }

    let mut szPath: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    StringCbCopyW(szPath.as_mut_ptr(), core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>(), item);
    PathAppendW(szPath.as_mut_ptr(), CPP_COM_WSTR_E8A4B2EE7EDE.as_ptr());

    let mut find: WIN32_FIND_DATAW = core::mem::zeroed();
    let mut hFind: HANDLE = FindFirstFileW(szPath.as_mut_ptr(), core::ptr::addr_of_mut!(find));
    if hFind == INVALID_HANDLE_VALUE {
        return;
    }

    loop {
        'body: {
            if wcscmp(find.cFileName.as_ptr(), CPP_COM_WSTR_C245AFF91259.as_ptr()) == 0
                || wcscmp(find.cFileName.as_ptr(), CPP_COM_WSTR_77ECA80E282A.as_ptr()) == 0
            {
                break 'body;
            }

            StringCbCopyW(szPath.as_mut_ptr(), core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>(), item);
            PathAppendW(szPath.as_mut_ptr(), find.cFileName.as_ptr());

            if (find.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) != 0 {
                DoAddFilesFromItem(files, szPath.as_ptr());
            } else {
                files.Add(CStringW::from_ptr(szPath.as_ptr()));
            }
        }
        if !(FindNextFileW(hFind, core::ptr::addr_of_mut!(find)) != 0) {
            break;
        }
    }

    FindClose(hFind);
}

unsafe extern "system" fn create_zip_function(arg: *mut core::ffi::c_void) -> u32 {
    let pCreator: *mut CZipCreator = arg as *mut CZipCreator;
    let mut result: u32 = (*(*pCreator).m_pimpl).JustDoIt();

    if result == 0 && !(*pCreator).m_pidlNotify.is_null() {
        SHChangeNotify((SHCNE_UPDATEDIR) as LONG, SHCNF_IDLIST, (*pCreator).m_pidlNotify.as_ptr() as LPCVOID, core::ptr::null_mut());
    }

    return result;
}

impl CZipCreator {
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        InterlockedDecrement(core::ptr::addr_of_mut!(g_ModuleRefCnt));
        CZipCreatorImpl__delete((*this_).m_pimpl);
    }
    pub unsafe fn DoCreate(pszExistingZip: PCWSTR, pszTargetDir: PCWSTR) -> *mut CZipCreator {
        let mut pCreator: *mut CZipCreator = CZipCreator__new_0();
        if !pszExistingZip.is_null() {
            (*(*pCreator).m_pimpl).m_ExistingZip = CStringW::from_ptr(pszExistingZip);
        }
        if !pszTargetDir.is_null() {
            (*(*pCreator).m_pimpl).m_TargetDir = CStringW::from_ptr(pszTargetDir);
        }
        return pCreator;
    }
    pub unsafe fn DoAddItem__body(&mut self, pszFile: PCWSTR) {
        let this_: *mut Self = self;
        // canonicalize path
        let mut szPath = [0 as WCHAR; (MAX_PATH) as _];
        GetFullPathNameW(pszFile, _countof!(szPath) as u32, szPath.as_mut_ptr(), core::ptr::null_mut());

        (*(*this_).m_pimpl).m_items.Add(CStringW::from_ptr(szPath.as_ptr()));
    }
    pub unsafe fn runThread(pCreator: *mut CZipCreator) -> BOOL {
        let mut tid: u32 = 0;
        let hThread: HANDLE = _beginthreadex(
            core::ptr::null_mut(),
            0,
            Some(create_zip_function),
            pCreator as *mut core::ffi::c_void,
            0,
            core::ptr::addr_of_mut!(tid),
        ) as HANDLE;

        if !hThread.is_null() {
            CloseHandle(hThread);
            return (TRUE) as BOOL;
        }

        DPRINT1!("hThread == NULL\n");

        let mut strTitle: CStringW = CStringW::from_ptr(MAKEINTRESOURCEW!(IDS_ERRORTITLE));
        let mut strText: CStringW = CStringW::from_ptr(MAKEINTRESOURCEW!(IDS_CANTSTARTTHREAD));
        MessageBoxW(
            core::ptr::null_mut(),
            strText.GetString(),
            strTitle.GetString(),
            MB_ICONERROR,
        );

        CZipCreator__delete(pCreator);
        return (FALSE) as BOOL;
    }
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        (*this_).m_pimpl = CZipCreatorImpl__new_0();
        {
            InterlockedIncrement(core::ptr::addr_of_mut!(g_ModuleRefCnt));
        }
    }
}

impl CZipCreatorImpl {
    pub unsafe fn JustDoIt(&mut self) -> u32 {
        let this_: *mut Self = self;
        // TODO: Show progress.

        let mut strZipName: CStringW = CStringW::new();
        let appendMode: i32;
        if (*this_).m_ExistingZip.IsEmpty() {
            strZipName = DoGetZipName((*(*this_).m_items.GetAt(0)).GetString());
            appendMode = (APPEND_STATUS_CREATE as i32);
        } else {
            strZipName = CStringW::from_ptr((*this_).m_ExistingZip.GetString());
            appendMode = (APPEND_STATUS_ADDINZIP as i32);
        }

        if (*this_).m_items.GetSize() <= 0 {
            DPRINT1!("GetSize() <= 0\n");
            return (CZCERR_ZEROITEMS as u32);
        }

        let mut files: CSimpleArray<CStringW> = CSimpleArray::new();
        let mut iItem: i32 = 0;
        while iItem < (*this_).m_items.GetSize() {
            DoAddFilesFromItem(&mut files, (*(*this_).m_items.GetAt(iItem)).GetString());
            iItem += 1;
        }

        if files.GetSize() <= 0 {
            DPRINT1!("files.GetSize() <= 0\n");

            let mut strTitle: CStringW = CStringW::from_resource(IDS_ERRORTITLE);
            let mut strText: CStringW = CStringW::new();
            atl_format_id!(strText, IDS_NOFILES, (*(*this_).m_items.GetAt(0)).GetString());
            MessageBoxW(core::ptr::null_mut(), strText.GetString(), strTitle.GetString(), MB_ICONERROR);

            return (CZCERR_NOFILES as u32);
        }

        let mut ffunc: zlib_filefunc64_def = core::mem::zeroed();
        ZeroMemory!(core::ptr::addr_of_mut!(ffunc) as *mut core::ffi::c_void, core::mem::size_of::<zlib_filefunc64_def>());
        fill_win32_filefunc64W(core::ptr::addr_of_mut!(ffunc));

        let zf = zipOpen2_64(strZipName.GetString() as _, appendMode, core::ptr::null_mut(), core::ptr::addr_of_mut!(ffunc));
        if zf == core::ptr::null_mut() {
            DPRINT1!("zf == 0\n");

            let err: i32 = CZCERR_CREATE;

            let mut strTitle: CStringW = CStringW::from_resource(IDS_ERRORTITLE);
            let mut strText: CStringW = CStringW::new();
            atl_format_id!(strText, IDS_CANTCREATEZIP, strZipName.GetString(), err);
            MessageBoxW(core::ptr::null_mut(), strText.GetString(), strTitle.GetString(), MB_ICONERROR);

            return err as u32;
        }

        // TODO: password
        let password: *const i8 = core::ptr::null();

        // Use dwAllocationGranularity for file mapping
        let mut si: SYSTEM_INFO = core::mem::zeroed();
        GetSystemInfo(core::ptr::addr_of_mut!(si));
        let dwChunkSize: u64 = (si.dwAllocationGranularity as u64) * 256;

        // The loop for creating a ZIP file
        let mut strTarget: CStringW = CStringW::new();
        let mut strBaseName: CStringW = DoGetBaseName((*(*this_).m_items.GetAt(0)).GetString());
        let nCodePage: u32 = GetZipCodePage((FALSE) as BOOL);
        let mut err: i32 = (ZIP_OK as i32);
        let mut iFile: i32 = 0;
        while iFile < files.GetSize() && err == (ZIP_OK as i32) {
            let strFile: &CStringW = &*files.GetAt(iFile);
            let mut zi: zip_fileinfo = core::mem::zeroed();

            // Open the file for storing it into a ZIP file
            let hFile: HANDLE = CreateFileW(strFile.GetString(), GENERIC_READ, FILE_SHARE_READ, core::ptr::null_mut(),
                                       OPEN_EXISTING, FILE_FLAG_SEQUENTIAL_SCAN, core::ptr::null_mut());
            if hFile == INVALID_HANDLE_VALUE {
                DPRINT1!("Cannot open file '%S'\n", strFile.GetString());
                err = CZCERR_READ;
                break;
            }

            // Load the file time info
            DoGetFileTimeInfo(hFile, core::ptr::addr_of_mut!(zi));

            // Get the file size
            let mut fileSize: LARGE_INTEGER = core::mem::zeroed();
            if GetFileSizeEx(hFile, core::ptr::addr_of_mut!(fileSize)) == 0 {
                DPRINT1!("Cannot get file size '%S'\n", strFile.GetString());
                CloseHandle(hFile);
                err = CZCERR_READ;
                break;
            }

            let crc: u32 = 0;
            if !password.is_null() {
                // TODO: crc = ...;
            }

            // Get filename info
            let mut strNameInZip: CStringA = DoGetNameInZip(&strBaseName, strFile, nCodePage);
            let mut strNameInZipUTF8: CStringA = DoGetNameInZip(&strBaseName, strFile, CP_UTF8);
            if !(*this_).m_TargetDir.IsEmpty() {
                let strTargetDir: CStringA = CW2AEX::<MAX_PATH>((*this_).m_TargetDir.GetString(), nCodePage);
                strNameInZip = strTargetDir + strNameInZip;

                let strTargetDirUTF8: CStringA = CW2AEX::<MAX_PATH>((*this_).m_TargetDir.GetString(), CP_UTF8);
                strNameInZipUTF8 = strTargetDirUTF8 + strNameInZipUTF8;
            }

            // Prepare additional field (UTF-8 name)
            let mut extraField: CSimpleArray<BYTE> = CSimpleArray::new();
            if nCodePage != CP_UTF8 && strNameInZip != strNameInZipUTF8 {
                // Header
                let headerID: WORD = (EF_UNIPATH as u16);
                let dataSize: WORD = (1 + 4 + strNameInZipUTF8.GetLength()) as WORD;
                extraField.Add((headerID & 0xFF) as BYTE);
                extraField.Add(((headerID >> 8) & 0xFF) as BYTE);
                extraField.Add((dataSize & 0xFF) as BYTE);
                extraField.Add(((dataSize >> 8) & 0xFF) as BYTE);
                extraField.Add(1 as BYTE); // Version

                // CRC32
                let nameCRC: DWORD = CalculateNameCRC32((strNameInZip.GetString()).cast());
                extraField.Add((nameCRC & 0xFF) as BYTE);
                extraField.Add(((nameCRC >> 8) & 0xFF) as BYTE);
                extraField.Add(((nameCRC >> 16) & 0xFF) as BYTE);
                extraField.Add(((nameCRC >> 24) & 0xFF) as BYTE);

                // UTF-8 name
                let mut ich: i32 = 0;
                while ich < strNameInZipUTF8.GetLength() {
                    extraField.Add(strNameInZipUTF8.GetAt(ich) as BYTE);
                    ich += 1;
                }
            }

            // Add a new file entry into a ZIP file
            err = zipOpenNewFileInZip4_64(zf, (strNameInZip.GetString()).cast(), core::ptr::addr_of!(zi),
                                          if extraField.GetSize() != 0 { extraField.GetData() as *const core::ffi::c_void } else { core::ptr::null() },
                                          extraField.GetSize() as u32, core::ptr::null(), 0, core::ptr::null(), (Z_DEFLATED as i32),
                                          Z_DEFAULT_COMPRESSION, 0, -MAX_WBITS, (DEF_MEM_LEVEL as i32),
                                          (Z_DEFAULT_STRATEGY as i32), password, crc,
                                          MINIZIP_COMPATIBLE_VERSION,
                                          if nCodePage == CP_UTF8 { MINIZIP_UTF8_FLAG } else { 0 }, 1);
            if err != 0 {
                DPRINT1!("zipOpenNewFileInZip3_64\n");
                break;
            }

            if fileSize.QuadPart > 0 { // Not an empty file?
                // Use a file mapping for quick file access and large file support
                let hMapping: HANDLE = CreateFileMappingW(hFile, core::ptr::null_mut(), PAGE_READONLY, 0, 0, core::ptr::null());
                if hMapping.is_null() {
                    DPRINT1!("Cannot create file mapping\n");
                    err = CZCERR_READ;
                    CloseHandle(hFile);
                    break;
                }

                // The loop for reading and storing file contents
                let mut dwMapSize: DWORD = 0;
                let mut offset: u64 = 0;
                while offset < (fileSize.QuadPart as u64) && err == (ZIP_OK as i32) {
                    let dwDelta: u64 = (fileSize.QuadPart as u64) - offset;
                    dwMapSize = min!(dwChunkSize, dwDelta) as DWORD;
                    let offsetLow: DWORD = offset as DWORD;
                    let offsetHigh: DWORD = (offset >> 32) as DWORD;
                    let pData: PVOID = MapViewOfFile(hMapping, FILE_MAP_READ, offsetHigh, offsetLow,
                                                (dwMapSize) as SIZE_T);
                    if pData.is_null() {
                        DPRINT1!("Cannot map the view\n");
                        err = CZCERR_READ;
                        break;
                    }

                    err = zipWriteInFileInZip(zf, pData, dwMapSize);
                    UnmapViewOfFile(pData);
                    offset += dwMapSize as u64;
                }

                CloseHandle(hMapping);
            }

            CloseHandle(hFile);

            zipCloseFileInZip(zf);
            iFile += 1;
        }

        zipClose(zf, core::ptr::null());

        if err != 0 {
            if err != 0 && (*this_).m_ExistingZip.IsEmpty() {
                DeleteFileW(strZipName.GetString());
            }

            let mut strTitle: CStringW = CStringW::from_resource(IDS_ERRORTITLE);

            let mut strText: CStringW = CStringW::new();
            if err < 0 {
                atl_format_id!(strText, IDS_CANTCREATEZIP, strZipName.GetString(), err);
            } else {
                atl_format_id!(strText, IDS_CANTREADFILE, strTarget.GetString());
            }

            MessageBoxW(core::ptr::null_mut(), strText.GetString(), strTitle.GetString(), MB_ICONERROR);
        } else {
            let mut szFullPath: [WCHAR; (MAX_PATH) as _] = core::mem::zeroed();
            GetFullPathNameW(strZipName.GetString(), _countof!(szFullPath) as DWORD, szFullPath.as_mut_ptr(), core::ptr::null_mut());

            if (*this_).m_ExistingZip.IsEmpty() {
                SHChangeNotify((SHCNE_CREATE) as LONG, SHCNF_PATHW, szFullPath.as_ptr() as _, core::ptr::null());
            } else {
                SHChangeNotify((SHCNE_UPDATEITEM) as LONG, SHCNF_PATHW, szFullPath.as_ptr() as _, core::ptr::null());
            }
        }

        return err as u32;
    }
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;}
}
