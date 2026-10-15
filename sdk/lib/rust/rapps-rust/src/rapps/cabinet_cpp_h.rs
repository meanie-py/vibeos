use crate::prelude::*;
use crate::rapps::misc_cpp_h::SplitFileAndDirectory;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_B7FC02907B1F: [WCHAR; 2] = [0x005C, 0x0000];
pub static CPP_COM_WSTR_D9EC3BB1502E: [WCHAR; 12] = [0x0063, 0x0061, 0x0062, 0x0069, 0x006E, 0x0065, 0x0074, 0x002E, 0x0064, 0x006C, 0x006C, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub type fnFDICreate = Option<
    unsafe extern "C" fn(
        PFNALLOC,
        PFNFREE,
        PFNOPEN,
        PFNREAD,
        PFNWRITE,
        PFNCLOSE,
        PFNSEEK,
        i32,
        PERF,
    ) -> HFDI,
>;

pub type fnFDICopy = Option<
    unsafe extern "C" fn(
        HFDI,
        LPSTR,
        LPSTR,
        INT,
        PFNFDINOTIFY,
        PFNFDIDECRYPT,
        *mut core::ffi::c_void,
    ) -> BOOL,
>;

pub type fnFDIDestroy = Option<unsafe extern "C" fn(HFDI) -> BOOL>;


#[no_mangle]
pub unsafe extern "C" fn WideToMultiByte(szSource: &CStringW, szDest: &mut CStringA, Codepage: UINT) -> BOOL {
    // determine the needed size
    let mut sz: INT = WideCharToMultiByte(Codepage, 0, szSource.GetString(), -1, core::ptr::null_mut(), 0, core::ptr::null_mut(), core::ptr::null_mut());
    if sz == 0 {
        return (FALSE) as BOOL;
    }

    // do the actual conversion
    sz = WideCharToMultiByte(Codepage, 0, szSource.GetString(), -1, szDest.GetBuffer(sz), sz, core::ptr::null_mut(), core::ptr::null_mut());

    szDest.ReleaseBuffer();
    return (sz != 0) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn MultiByteToWide(szSource: *const CStringA, szDest: *mut CStringW, Codepage: UINT) -> BOOL {
    // determine the needed size
    let mut sz: INT = MultiByteToWideChar(Codepage, 0, (*szSource).GetString() as LPCSTR, -1, core::ptr::null_mut(), 0);
    if sz == 0 {
        return (FALSE) as BOOL;
    }

    // do the actual conversion
    sz = MultiByteToWideChar(Codepage, 0, (*szSource).GetString() as LPCSTR, -1, (*szDest).GetBuffer(sz), sz);

    (*szDest).ReleaseBuffer();
    return (sz != 0) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn fnMemAlloc(cb: ULONG) -> *mut core::ffi::c_void {
    return HeapAlloc(GetProcessHeap(), 0, (cb) as SIZE_T) as *mut core::ffi::c_void;
}
#[no_mangle]
pub unsafe extern "C" fn fnMemFree(pv: *mut core::ffi::c_void) {
    HeapFree(GetProcessHeap(), 0 /* NULL */, pv);
}
#[no_mangle]
pub unsafe extern "C" fn fnFileOpen(pszFile: *mut i8, oflag: i32, pmode: i32) -> INT_PTR {
    let mut hFile: HANDLE = core::ptr::null_mut();
    let mut dwDesiredAccess: DWORD = 0;
    let mut dwCreationDisposition: DWORD = 0;
    let mut szFileName: CStringW = CStringW::new();

    UNREFERENCED_PARAMETER!(pmode);

    if (oflag & (_O_RDWR as i32)) != 0 {
        dwDesiredAccess = GENERIC_READ | GENERIC_WRITE;
    } else if (oflag & (_O_WRONLY as i32)) != 0 {
        dwDesiredAccess = GENERIC_WRITE;
    } else {
        dwDesiredAccess = GENERIC_READ;
    }

    if (oflag & (_O_CREAT as i32)) != 0 {
        dwCreationDisposition = CREATE_ALWAYS;
    } else {
        dwCreationDisposition = OPEN_EXISTING;
    }

    MultiByteToWide(pszFile as *const CStringA, core::ptr::addr_of_mut!(szFileName), CP_UTF8);

    hFile = CreateFileW(
        szFileName.GetString(),
        dwDesiredAccess,
        FILE_SHARE_READ,
        core::ptr::null_mut(),
        dwCreationDisposition,
        FILE_ATTRIBUTE_NORMAL,
        core::ptr::null_mut(),
    );

    return hFile as INT_PTR;
}
#[no_mangle]
pub unsafe extern "C" fn fnFileRead(hf: INT_PTR, pv: *mut core::ffi::c_void, cb: UINT) -> UINT {
    let mut dwBytesRead: DWORD = 0;

    if ReadFile(hf as HANDLE, pv, cb, core::ptr::addr_of_mut!(dwBytesRead), core::ptr::null_mut()) == (FALSE as i32) {
        dwBytesRead = (0u32).wrapping_sub(1 as DWORD);
    }

    return dwBytesRead;
}
#[no_mangle]
pub unsafe extern "C" fn fnFileWrite(hf: INT_PTR, pv: *mut core::ffi::c_void, cb: UINT) -> UINT {
    let mut dwBytesWritten: DWORD = 0;

    if WriteFile(hf as HANDLE, pv, cb, core::ptr::addr_of_mut!(dwBytesWritten), core::ptr::null_mut()) == (FALSE as i32) {
        dwBytesWritten = (1 as DWORD).wrapping_neg();
    }

    return dwBytesWritten;
}
#[no_mangle]
pub unsafe extern "C" fn fnFileClose(hf: INT_PTR) -> i32 {
    return if CloseHandle(hf as HANDLE) != (FALSE as i32) { 0 } else { -1 };
}
#[no_mangle]
pub unsafe extern "C" fn fnFileSeek(hf: INT_PTR, dist: LONG, seektype: core::ffi::c_int) -> LONG {
    return (SetFilePointer(hf as HANDLE, dist, core::ptr::null_mut(), (seektype) as DWORD)) as LONG;
}
#[no_mangle]
pub unsafe extern "C" fn fnNotify(fdint: FDINOTIFICATIONTYPE, pfdin: PFDINOTIFICATION) -> INT_PTR {
    let mut iResult: INT_PTR = 0;
    let pND: *mut NotifyData = (*pfdin).pv as *mut NotifyData;

    'sw: {
        match fdint {
            fdintCOPY_FILE => {
                let mut szExtractDir = CStringW::new();
                let mut szCabFileName = CStringW::new();

                // Append the destination directory to the file name.
                MultiByteToWide((*pND).OutputDir as *const CStringA, addr_of_mut!(szExtractDir), CP_UTF8);
                let codepage: UINT = if ((*pfdin).attribs & (_A_NAME_IS_UTF as u16)) != 0 { CP_UTF8 } else { CP_ACP };
                MultiByteToWide((*pfdin).psz1 as *const CStringA, addr_of_mut!(szCabFileName), codepage);

                if NotifyFileExtractCallback((szCabFileName.GetString()).cast(), ((*pfdin).cb) as UINT64, ((*pfdin).attribs) as UINT,
                                           (*pND).Callback, (*pND).CallerCookie) == 0
                {
                    break 'sw; // Skip file
                }

                if szCabFileName.Find('\\' as u16, 0) >= 0
                {
                    let mut szNewDirName = CStringW::from_ptr(szExtractDir.GetString());
                    let mut nTokenPos: i32 = 0;
                    // We do not want to interpret the filename as directory,
                    // so bail out before the last token!
                    while szCabFileName.Find('\\' as u16, nTokenPos) >= 0
                    {
                        let token = szCabFileName.Tokenize((CPP_COM_WSTR_B7FC02907B1F).as_ptr(), &mut nTokenPos);
                        if token.IsEmpty()
                        {
                            break;
                        }

                        szNewDirName.Append((CPP_COM_WSTR_B7FC02907B1F).as_ptr());
                        szNewDirName.Append(token.GetString());
                        if CreateDirectoryW(szNewDirName.GetString(), core::ptr::null_mut()) == 0
                        {
                            let dwErr: DWORD = GetLastError();
                            if dwErr != ERROR_ALREADY_EXISTS
                            {
                                DPRINT1!(
                                    "ERROR: Unable to create directory %S (err %lu)\n", szNewDirName.GetString(), dwErr);
                            }
                        }
                    }
                }

                let mut szNewFileName = CStringW::from_ptr(szExtractDir.GetString());
                szNewFileName.Append((CPP_COM_WSTR_B7FC02907B1F).as_ptr());
                szNewFileName.Append(szCabFileName.GetString());

                let mut szFilePathUTF8 = CStringA::new();
                WideToMultiByte(&szNewFileName, &mut szFilePathUTF8, CP_UTF8);

                // Open the file
                iResult = fnFileOpen(szFilePathUTF8.GetString() as LPSTR, (_O_WRONLY | _O_CREAT) as i32, 0);
            }

            fdintCLOSE_FILE_INFO => {
                iResult = (fnFileClose((*pfdin).hf) == 0) as INT_PTR;
            }

            fdintNEXT_CABINET => {
                if (*pfdin).fdie != FDIERROR_NONE
                {
                    iResult = -1;
                }
            }

            fdintPARTIAL_FILE => {
                iResult = 0;
            }

            fdintCABINET_INFO => {
                iResult = 0;
            }

            fdintENUMERATE => {
                iResult = 0;
            }

            _ => {
                iResult = -1;
            }
        }
    }

    return iResult;
}
#[no_mangle]
pub unsafe extern "C" fn ExtractFilesFromCab__0(szCabName: &CStringW, szCabDir: &CStringW, szOutputDir: &CStringW, Callback: EXTRACTCALLBACK, Cookie: *mut core::ffi::c_void) -> BOOL {
    let mut hCabinetDll: HINSTANCE;
    let mut ExtractHandler: HFDI;
    let mut ExtractErrors: ERF = core::mem::zeroed();
    let mut szCabNameUTF8 = CStringA::new();
    let mut szCabDirUTF8 = CStringA::new();
    let mut szOutputDirUTF8 = CStringA::new();
    let mut pfnFDICreate: fnFDICreate;
    let mut pfnFDICopy: fnFDICopy;
    let mut pfnFDIDestroy: fnFDIDestroy;
    let mut bResult: BOOL;
    let mut nd: NotifyData = NotifyData {
        Callback: Callback,
        CallerCookie: Cookie,
        OutputDir: core::ptr::null(),
    };

    // Load cabinet.dll and extract needed functions
    hCabinetDll = LoadLibraryW(CPP_COM_WSTR_D9EC3BB1502E.as_ptr());

    if (hCabinetDll as usize) == 0 {
        return (FALSE) as BOOL;
    }

    pfnFDICreate = core::mem::transmute(GetProcAddress(hCabinetDll, b"FDICreate\0".as_ptr() as *const i8));
    pfnFDICopy = core::mem::transmute(GetProcAddress(hCabinetDll, b"FDICopy\0".as_ptr() as *const i8));
    pfnFDIDestroy = core::mem::transmute(GetProcAddress(hCabinetDll, b"FDIDestroy\0".as_ptr() as *const i8));

    if (pfnFDICreate).map_or(0, |f| f as usize) as usize == 0 || (pfnFDICopy).map_or(0, |f| f as usize) as usize == 0 || (pfnFDIDestroy).map_or(0, |f| f as usize) as usize == 0 {
        FreeLibrary((hCabinetDll).cast());
        return (FALSE) as BOOL;
    }

    // Create FDI context
    ExtractHandler = (pfnFDICreate.unwrap_unchecked())(
        Some(fnMemAlloc), Some(fnMemFree), Some(fnFileOpen), Some(fnFileRead), Some(fnFileWrite), Some(fnFileClose), Some(fnFileSeek), cpuUNKNOWN,
        core::ptr::addr_of_mut!(ExtractErrors));

    if (ExtractHandler as usize) == 0 {
        FreeLibrary((hCabinetDll).cast());
        return (FALSE) as BOOL;
    }

    // Create output dir
    bResult = CreateDirectoryW(szOutputDir.GetString(), core::ptr::null_mut());

    if bResult != 0 || GetLastError() == ERROR_ALREADY_EXISTS {
        // Convert wide strings to UTF-8
        bResult = WideToMultiByte(szCabName, &mut szCabNameUTF8, CP_UTF8);
        bResult &= WideToMultiByte(szCabDir, &mut szCabDirUTF8, CP_UTF8);
        bResult &= WideToMultiByte(szOutputDir, &mut szOutputDirUTF8, CP_UTF8);
    }

    // Perform extraction
    if bResult != 0 {
        // Add a slash to cab name as required by the api
        // szCabNameUTF8 = "\\" + szCabNameUTF8;  (atl_rust's CStringA has no operator+)
        let mut tmp = CStringA::new();
        let cablen: i32 = szCabNameUTF8.GetLength();
        let dst = tmp.GetBuffer(1 + cablen) as *mut u8;
        *dst = b'\\';
        core::ptr::copy_nonoverlapping(szCabNameUTF8.GetString(), dst.add(1), cablen as usize);
        tmp.ReleaseBufferSetLength(1 + cablen);
        szCabNameUTF8 = tmp;

        nd.OutputDir = (szOutputDirUTF8.GetString()).cast();
        bResult = (pfnFDICopy.unwrap_unchecked())(
            ExtractHandler,
            szCabNameUTF8.GetString() as LPSTR,
            szCabDirUTF8.GetString() as LPSTR,
            0,
            Some(fnNotify),
            None,
            core::ptr::addr_of_mut!(nd) as *mut core::ffi::c_void);
    }

    (pfnFDIDestroy.unwrap_unchecked())(ExtractHandler);
    FreeLibrary((hCabinetDll).cast());
    return bResult;
}
#[no_mangle]
pub unsafe extern "C" fn ExtractFilesFromCab__1(FullCabPath: LPCWSTR, szOutputDir: &CStringW, Callback: EXTRACTCALLBACK, Cookie: *mut core::ffi::c_void) -> BOOL {
    let mut dir = CStringW::new();
    let mut file = SplitFileAndDirectory(FullCabPath, core::ptr::addr_of_mut!(dir));
    return ExtractFilesFromCab__0(&file, &dir, szOutputDir, Callback, Cookie);
}

impl NotifyData {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;}
}
