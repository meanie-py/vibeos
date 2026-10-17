// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_4F2D892D6AF4: [WCHAR; 2] = [0x0029, 0x0000];
pub static CPP_COM_WSTR_8D71B3FAAB82: [WCHAR; 2] = [0x0020, 0x0000];
pub static CPP_COM_WSTR_ACBE5EA9B8F8: [WCHAR; 3] = [0x0020, 0x0028, 0x0000];
pub static CPP_COM_WSTR_EAFE2DCE91A2: [WCHAR; 6] = [0x0025, 0x0049, 0x0036, 0x0034, 0x0075, 0x0000];

// The island/bindgen substrate (via `prelude`) is the ONLY substrate source for this TU; the
// `types`/`win32`/`shellcom` globs the assembler also emits are dropped rather than joined,
// because carrying both makes every shared name ambiguous (E0659) at 70+ sites. See prelude.rs.
use core::ffi::c_void;
use crate::prelude::*;

// --- substrate this slice needs and the island does not carry ---
//
// The bridges below must call the REAL implementations, never the sibling class slice's
// declarations of the same names: `crate::file_def_ext` spells `PathIsDotOrDotDotW` and friends
// with `#[link_name = "..._RustBridge"]`, so importing those here would make every bridge an
// unconditional call to ITSELF. That exact shape shipped once in drvdefext_tu.rs.
//
// PathIsDotOrDotDotW / SHELL_DisplayNameOf / SHELL32_GetDllFromRundll32CommandLine are C++ and
// were reachable only through the C-linkage forwarders this .cpp used to publish. They are now
// EXTERN_C in precomp.h — the same fix, and for the same reason, as the DataObject_GetHIDACount
// group already documented there.
// `extern "C"`, NOT "system": all four are declared EXTERN_C *without* WINAPI, so they are cdecl
// (`_Name`) and not stdcall (`_Name@N`). Getting this wrong is invisible on amd64/arm64, where
// the two conventions coincide, and shows up only as an i386 LINK_FAIL naming the @N form.
unsafe extern "C" {
    fn PathIsDotOrDotDotW(path: LPCWSTR) -> BOOL;
    fn SHELL_DisplayNameOf(
        psf: *mut IShellFolder,
        pidl: LPCITEMIDLIST,
        flags: UINT,
        ppStr: *mut LPWSTR,
    ) -> HRESULT;
    fn SHELL32_GetDllFromRundll32CommandLine(cmd: LPCWSTR, out: LPWSTR, cch: SIZE_T) -> HRESULT;
    fn DataObject_GetHIDACount(pdo: *mut IDataObject) -> HRESULT;
}

// `SH_CreatePropertySheetPageEx` is already Rust (propsheet.rs) but is not re-exported by the
// prelude the way its sibling `SH_CreatePropertySheetPage` is.
use crate::propsheet::SH_CreatePropertySheetPageEx;

// One definition, owned by the class slice of this same TU (README point 4: pick ONE owner
// rather than letting each slice re-emit the shared preamble).
use crate::file_def_ext::StringCchCopyExW;

// PROPSHEETPAGE callback, spelled as the class slice spells it.
type LPFNPSPCALLBACK = unsafe extern "system" fn(HWND, UINT, *mut PROPSHEETPAGEW) -> UINT;

const LOCALE_USER_DEFAULT: DWORD = 0x0400;

#[inline]
fn NT_SUCCESS(status: NTSTATUS) -> bool {
    status >= 0
}

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct DIRTREESTATS {
    pub pDirCount: *mut u32,
    pub pFileCount: *mut u32,
    // PULARGE_INTEGER in the C, not a bare u64 pointer: the union spelling is what the
    // `.QuadPart` / `.u.LowPart` accesses below need.
    pub pTotVirtSize: PULARGE_INTEGER,
    pub pTotPhysSize: PULARGE_INTEGER,
    pub nTick: u32,
    pub fAttribSet: u32,
    pub fAttribAll: u32,
    pub bMultipleTypes: i32,
    pub bDeepRecurse: i32,
    pub szType: [u16; 100],
}

pub type PDIRTREESTATS = *mut DIRTREESTATS;

pub static s_szShellClassInfo: [u16; 17] = [
    0x002e, 0x0053, 0x0068, 0x0065, 0x006c, 0x006c, 0x0043, 0x006c,
    0x0061, 0x0073, 0x0073, 0x0049, 0x006e, 0x0066, 0x006f, 0x0000,
    0x0000,
];

pub static s_szIconIndex: [u16; 10] = [
    0x0049, 0x0063, 0x006f, 0x006e, 0x0049, 0x006e, 0x0064, 0x0065,
    0x0078, 0x0000,
];

pub static s_szIconFile: [u16; 9] = [
    0x0049, 0x0063, 0x006f, 0x006e, 0x0046, 0x0069, 0x006c, 0x0065,
    0x0000,
];

pub static s_szIconResource: [u16; 13] = [
    0x0049, 0x0063, 0x006f, 0x006e, 0x0052, 0x0065, 0x0073, 0x006f,
    0x0075, 0x0072, 0x0063, 0x0065, 0x0000,
];
unsafe extern "C" fn SH32_GetFileFindData(pszPath: PCWSTR, pWFD: *mut WIN32_FIND_DATAW) -> BOOL {
    let hFind = FindFirstFileW(pszPath, pWFD);
    if hFind != INVALID_HANDLE_VALUE { FindClose(hFind) } else { FALSE }
}
#[no_mangle]
pub unsafe extern "C" fn GetPhysicalFileSize(pszPath: PCWSTR, Size: PULARGE_INTEGER) -> BOOL {
    let hFile = CreateFileW(
        pszPath,
        FILE_READ_ATTRIBUTES,
        FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
        core::ptr::null_mut(),
        OPEN_EXISTING,
        FILE_FLAG_OPEN_NO_RECALL,
        core::ptr::null_mut(),
    );
    if hFile == INVALID_HANDLE_VALUE {
        return FALSE;
    }

    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut FileInfo: FILE_STANDARD_INFORMATION = core::mem::zeroed();
    let Status = NtQueryInformationFile(
        hFile,
        &mut IoStatusBlock,
        &mut FileInfo as *mut _ as *mut core::ffi::c_void,
        core::mem::size_of::<FILE_STANDARD_INFORMATION>() as u32,
        FileStandardInformation,
    );
    if NT_SUCCESS(Status) {
        (*Size).QuadPart = FileInfo.AllocationSize.QuadPart as u64;
    }
    CloseHandle(hFile);
    NT_SUCCESS(Status) as BOOL
}
unsafe extern "C" fn GetFileStats(
    hFile: HANDLE,
    pVirtSize: PULARGE_INTEGER,
    pPhysSize: PULARGE_INTEGER,
) -> UINT {
    let mut Status: NTSTATUS;
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut BasicInfo: FILE_BASIC_INFORMATION = core::mem::zeroed();

    Status = NtQueryInformationFile(
        hFile,
        &mut IoStatusBlock,
        &mut BasicInfo as *mut _ as *mut c_void,
        core::mem::size_of::<FILE_BASIC_INFORMATION>() as u32,
        FileBasicInformation,
    );

    if !NT_SUCCESS(Status) {
        return INVALID_FILE_ATTRIBUTES;
    }

    if (!pVirtSize.is_null() || !pPhysSize.is_null())
        && (BasicInfo.FileAttributes & FILE_ATTRIBUTE_DIRECTORY) == 0
    {
        let mut FileInfo: FILE_STANDARD_INFORMATION = core::mem::zeroed();
        Status = NtQueryInformationFile(
            hFile,
            &mut IoStatusBlock,
            &mut FileInfo as *mut _ as *mut c_void,
            core::mem::size_of::<FILE_STANDARD_INFORMATION>() as u32,
            FileStandardInformation,
        );

        if !NT_SUCCESS(Status) {
            return INVALID_FILE_ATTRIBUTES;
        }

        if !pVirtSize.is_null() {
            (*pVirtSize).QuadPart = FileInfo.EndOfFile.QuadPart as u64;
        }
        if !pPhysSize.is_null() {
            (*pPhysSize).QuadPart = FileInfo.AllocationSize.QuadPart as u64;
        }
    }

    BasicInfo.FileAttributes
}

/// filedefext.cpp:91. C++ OVERLOADS `GetFileStats` on its first parameter (HANDLE above,
/// PCWSTR here); Rust has no overloading, so the path form is a distinct name. The translated
/// slice carried only the HANDLE form, which left the one call site below — the `else` branch
/// of ProcessDirStatsItem, i.e. every non-directory item in a folder's Properties size
/// calculation — with no callee at all.
unsafe extern "C" fn GetFileStats_Path(
    pszPath: PCWSTR,
    pVirtSize: PULARGE_INTEGER,
    pPhysSize: PULARGE_INTEGER,
) -> UINT {
    let hFile = CreateFileW(
        pszPath,
        FILE_READ_ATTRIBUTES,
        FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
        core::ptr::null_mut(),
        OPEN_EXISTING,
        FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_NO_RECALL,
        core::ptr::null_mut(),
    );
    if hFile != INVALID_HANDLE_VALUE {
        let Attrib = GetFileStats(hFile, pVirtSize, pPhysSize);
        CloseHandle(hFile);
        return Attrib;
    }

    let mut Data: WIN32_FIND_DATAW = core::mem::zeroed();
    if SH32_GetFileFindData(pszPath, &mut Data) == 0 {
        return INVALID_FILE_ATTRIBUTES;
    }
    if !pVirtSize.is_null() {
        (*pVirtSize).u.LowPart = Data.nFileSizeLow;
        (*pVirtSize).u.HighPart = Data.nFileSizeHigh;
    }
    if !pPhysSize.is_null() {
        (*pPhysSize).u.LowPart = Data.nFileSizeLow;
        (*pPhysSize).u.HighPart = Data.nFileSizeHigh;
    }
    Data.dwFileAttributes
}

#[no_mangle]
pub unsafe extern "C" fn SH_FormatInteger(Num: i64, pwszResult: PWSTR, cchResultMax: u32) -> u32 {
    let mut wszNumber: [u16; 24] = core::mem::zeroed();
    swprintf(wszNumber.as_mut_ptr(), CPP_COM_WSTR_EAFE2DCE91A2.as_ptr(), Num);

    let mut wszDecimalSep: [u16; 8] = core::mem::zeroed();
    let mut wszThousandSep: [u16; 8] = core::mem::zeroed();
    GetLocaleInfoW(LOCALE_USER_DEFAULT, LOCALE_SDECIMAL, wszDecimalSep.as_mut_ptr(), _countof!(wszDecimalSep) as INT);
    GetLocaleInfoW(LOCALE_USER_DEFAULT, LOCALE_STHOUSAND, wszThousandSep.as_mut_ptr(), _countof!(wszThousandSep) as INT);

    let mut nf: NUMBERFMTW = core::mem::zeroed();
    ZeroMemory(core::ptr::addr_of_mut!(nf) as *mut core::ffi::c_void, core::mem::size_of::<NUMBERFMTW>());
    nf.lpDecimalSep = wszDecimalSep.as_mut_ptr();
    nf.lpThousandSep = wszThousandSep.as_mut_ptr();

    let mut wszGrouping: [u16; 12] = core::mem::zeroed();
    let cchGrouping: i32 = GetLocaleInfoW(LOCALE_USER_DEFAULT,
                                     LOCALE_SGROUPING,
                                     wszGrouping.as_mut_ptr(),
                                     _countof!(wszGrouping) as INT);

    let mut i: i32 = 0;
    while i < cchGrouping {
        let wch: u16 = wszGrouping[i as usize];

        if wch >= 0x0030u16 && wch <= 0x0039u16 {
            nf.Grouping = nf.Grouping * 10 + (wch - 0x0030u16) as u32;
        } else if wch != 0x003Bu16 {
            break;
        }
        i += 1;
    }

    if (nf.Grouping % 10) == 0 {
        nf.Grouping /= 10;
    } else {
        nf.Grouping *= 10;
    }

    let cchResult: i32 = GetNumberFormatW(LOCALE_USER_DEFAULT,
                                    0,
                                    wszNumber.as_mut_ptr(),
                                    core::ptr::addr_of_mut!(nf),
                                    pwszResult,
                                    cchResultMax as INT);

    if cchResult == 0 {
        return 0;
    }

    (cchResult - 1) as u32
}
#[no_mangle]
pub unsafe extern "C" fn SH_FormatByteSize(cbSize: i64, pwszResult: PWSTR, cchResultMax: u32) -> u32 {
    let cchWritten = SH_FormatInteger(cbSize, pwszResult, cchResultMax);
    if cchWritten == 0 {
        return 0;
    }

    // `cchRemaining` is size_t in the C and StringCchCopyExW updates it in place, so it stays
    // usize here rather than being narrowed to the UINT that `cchResultMax` is.
    let pwszEnd = pwszResult.offset(cchWritten as isize);
    let mut cchRemaining: usize = cchResultMax.wrapping_sub(cchWritten) as usize;
    let mut pwszEndOut = pwszEnd;
    StringCchCopyExW(
        pwszEnd,
        cchRemaining,
        CPP_COM_WSTR_8D71B3FAAB82.as_ptr(),
        &mut pwszEndOut,
        &mut cchRemaining,
        0,
    );
    let cchWritten2 = LoadStringW(
        shell32_hInstance,
        IDS_BYTES_FORMAT,
        pwszEndOut,
        cchRemaining as INT,
    );
    cchRemaining = cchRemaining.wrapping_sub(cchWritten2 as usize);

    cchResultMax.wrapping_sub(cchRemaining as UINT)
}

#[no_mangle]
pub unsafe extern "system" fn SH_FormatByteSize_RustBridge(
    cbSize: LONGLONG,
    pwszResult: LPWSTR,
    cchResultMax: UINT,
) -> UINT {
    SH_FormatByteSize(cbSize, pwszResult, cchResultMax)
}
#[no_mangle]
pub unsafe extern "C" fn SH_FormatFileSizeWithBytes(
    lpQwSize: *const ULARGE_INTEGER,
    pwszResult: PWSTR,
    cchResultMax: u32,
) -> PWSTR {
    if StrFormatByteSizeW((*lpQwSize).QuadPart as LONGLONG, pwszResult, cchResultMax).is_null() {
        return core::ptr::null_mut();
    }

    if (*lpQwSize).QuadPart < 1024 {
        return pwszResult;
    }

    let cchWritten: u32 = wcslen(pwszResult) as u32;
    let pwszEnd: PWSTR = pwszResult.offset(cchWritten as isize);
    let mut cchRemaining: usize = (cchResultMax - cchWritten) as usize;
    let mut pwszEnd_out: PWSTR = pwszEnd;
    StringCchCopyExW(
        pwszEnd,
        cchRemaining,
        CPP_COM_WSTR_ACBE5EA9B8F8.as_ptr(),
        &mut pwszEnd_out,
        &mut cchRemaining,
        0,
    );

    let cchWritten2: u32 = SH_FormatByteSize((*lpQwSize).QuadPart as LONGLONG, pwszEnd_out, cchRemaining as UINT);
    let pwszEnd_final: PWSTR = pwszEnd_out.offset(cchWritten2 as isize);
    let cchRemaining_final: usize = cchRemaining - cchWritten2 as usize;

    StringCchCopyW(pwszEnd_final, cchRemaining_final, CPP_COM_WSTR_4F2D892D6AF4.as_ptr());

    pwszResult
}
unsafe extern "C" fn ProcessDirStatsItem(
    pszPath: PCWSTR,
    Stats: *mut DIRTREESTATS,
    pWFD: *mut WIN32_FIND_DATAW,
) -> UINT {
    let fAttrib: UINT;
    let mut nVirtSize: ULARGE_INTEGER = core::mem::zeroed();
    let mut nPhysSize: ULARGE_INTEGER = core::mem::zeroed();
    if !pWFD.is_null() && ((*pWFD).dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) != 0 {
        fAttrib = (*pWFD).dwFileAttributes;
    } else {
        fAttrib = GetFileStats_Path(pszPath, &mut nVirtSize, &mut nPhysSize);
        if fAttrib == INVALID_FILE_ATTRIBUTES {
            return fAttrib;
        }
    }

    (*Stats).fAttribSet &= fAttrib;
    (*Stats).fAttribAll |= fAttrib;
    if (fAttrib & FILE_ATTRIBUTE_DIRECTORY) != 0 {
        *(*Stats).pDirCount += 1;
    } else {
        *(*Stats).pFileCount += 1;
        (*(*Stats).pTotVirtSize).QuadPart += nVirtSize.QuadPart;
        (*(*Stats).pTotPhysSize).QuadPart += nPhysSize.QuadPart;
    }

    if (*Stats).bMultipleTypes == 0 {
        let mut shfi: SHFILEINFOW = core::mem::zeroed();
        if SHGetFileInfoW(
            pszPath,
            fAttrib,
            &mut shfi,
            core::mem::size_of::<SHFILEINFOW>() as DWORD,
            SHGFI_TYPENAME | SHGFI_USEFILEATTRIBUTES,
        ) == 0
        {
            (*Stats).bMultipleTypes = TRUE;
        } else if (*Stats).szType[0] == 0 {
            wcscpy((*Stats).szType.as_mut_ptr(), core::ptr::addr_of!(shfi.szTypeName) as *const WCHAR);
        } else {
            (*Stats).bMultipleTypes = lstrcmpW((*Stats).szType.as_ptr(), core::ptr::addr_of!(shfi.szTypeName) as *const WCHAR) as BOOL;
        }
    }
    fAttrib
}

#[no_mangle]
pub unsafe extern "system" fn PathIsDotOrDotDotW_RustBridge(path: LPCWSTR) -> BOOL {
    PathIsDotOrDotDotW(path)
}

#[no_mangle]
pub unsafe extern "system" fn SHELL_DisplayNameOf_RustBridge(
    folder: *mut IShellFolder,
    pidl: LPCITEMIDLIST,
    flags: DWORD,
    out: *mut LPWSTR,
) -> HRESULT {
    SHELL_DisplayNameOf(folder, pidl, flags, out)
}

#[no_mangle]
pub unsafe extern "system" fn SH_FormatFileSizeWithBytes_RustBridge(
    size: *const ULARGE_INTEGER,
    out: PWSTR,
    capacity: u32,
) -> PWSTR {
    return SH_FormatFileSizeWithBytes(size, out, capacity);
}

#[no_mangle]
pub unsafe extern "system" fn GetPhysicalFileSize_RustBridge(
    path: LPCWSTR,
    size: PULARGE_INTEGER,
) -> BOOL {
    GetPhysicalFileSize(path, size)
}

#[no_mangle]
pub unsafe extern "system" fn DataObject_GetHIDACount_RustBridge(
    data: *mut IDataObject,
) -> HRESULT {
    DataObject_GetHIDACount(data)
}
#[no_mangle]
pub unsafe extern "system" fn _ILCopyCidaToaPidl_RustBridge(
    folder: *mut LPITEMIDLIST,
    cida: *const CIDA,
) -> *mut LPITEMIDLIST {
    _ILCopyCidaToaPidl(folder, cida)
}

#[no_mangle]
pub unsafe extern "system" fn SH_CreatePropertySheetPage_RustBridge(
    id: WORD,
    proc: DLGPROC,
    param: LPARAM,
    title: LPCWSTR,
) -> HPROPSHEETPAGE {
    (SH_CreatePropertySheetPage(id, proc, param, title)).cast()
}

#[no_mangle]
pub unsafe extern "system" fn SH_CreatePropertySheetPageEx_RustBridge(
    id: WORD,
    proc: DLGPROC,
    param: LPARAM,
    title: LPCWSTR,
    callback: LPFNPSPCALLBACK,
) -> HPROPSHEETPAGE {
    // propsheet.rs is built on the hand-written `types`/`win32` substrate while this slice is
    // built on the island, so DLGPROC (Option<fn> there vs fn here), LPARAM (isize vs i32) and
    // HPROPSHEETPAGE (*mut c_void vs *mut HPROPSHEETPAGE__) are distinct nominal types over one
    // identical ABI. A pure forwarder is the right place to convert that seam. Option<fn> and fn
    // share a representation (null-pointer optimization), so the fn-pointer transmutes are sound.
    SH_CreatePropertySheetPageEx(
        id,
        core::mem::transmute(proc),
        (param) as LPARAM,
        title,
        core::mem::transmute(callback),
    ) as HPROPSHEETPAGE
}

#[no_mangle]
pub unsafe extern "system" fn SHELL32_GetDllFromRundll32CommandLine_RustBridge(
    command: LPCWSTR,
    out: LPWSTR,
    capacity: UINT,
) -> HRESULT {
    // The C declares cchMax as SIZE_T and the C++ forwarder passed a UINT into it. Cast with the
    // NAMED type: SIZE_T is u32 on i386 and u64 elsewhere, so a concrete width would bake one arch.
    SHELL32_GetDllFromRundll32CommandLine(command, out, capacity as SIZE_T)
}
