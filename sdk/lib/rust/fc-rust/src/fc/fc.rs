// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_075DE2B906DB: [u16; 2] = [0x000Au16, 0x0000];
pub static RUSTCONV_WSTR_27D452FB378E: [u16; 4] = [0x0042u16, 0x0049, 0x004E, 0x0000];
pub static RUSTCONV_WSTR_2EBEB4925984: [u16; 5] = [0x002Fu16, 0x004F, 0x0046, 0x0046, 0x0000];
pub static RUSTCONV_WSTR_31A10D3954D0: [u16; 18] = [
    0x0025u16, 0x0030, 0x0038, 0x006C, 0x0058, 0x003A, 0x0020, 0x0025, 0x0030, 0x0032,
    0x0058, 0x0020, 0x0025, 0x0030, 0x0032, 0x0058, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_3DB566BC3DA4: [u16; 4] = [0x0043u16, 0x004F, 0x004D, 0x0000];
pub static RUSTCONV_WSTR_47AEC1CCED2E: [u16; 9] = [0x002Fu16, 0x004F, 0x0046, 0x0046, 0x004C, 0x0049, 0x004E, 0x0045, 0x0000];
pub static RUSTCONV_WSTR_481F1A6D4120: [u16; 5] = [0x0025u16, 0x006C, 0x0073, 0x000A, 0x0000];
pub static RUSTCONV_WSTR_55B8302A0040: [u16; 4] = [0x004Fu16, 0x0042, 0x004A, 0x0000];
pub static RUSTCONV_WSTR_578D89EBADFC: [u16; 4] = [0x0053u16, 0x0059, 0x0053, 0x0000];
pub static RUSTCONV_WSTR_5BFB8DE9D124: [u16; 5] = [0x002Eu16, 0x002E, 0x002E, 0x000A, 0x0000];
pub static RUSTCONV_WSTR_5CDCE2B4827A: [u16; 11] = [
    0x002Au16, 0x002A, 0x002A, 0x002A, 0x002A, 0x0020, 0x0025, 0x006C, 0x0073, 0x000A,
    0x0000,
];
pub static RUSTCONV_WSTR_8C7CF1E9BC62: [u16; 4] = [0x0045u16, 0x0058, 0x0045, 0x0000];
pub static RUSTCONV_WSTR_9577F614E547: [u16; 8] = [0x002Au16, 0x002A, 0x002A, 0x002A, 0x002A, 0x000A, 0x000A, 0x0000];
pub static RUSTCONV_WSTR_9FE91A4E9A02: [u16; 11] = [
    0x0025u16, 0x0035, 0x0064, 0x003A, 0x0020, 0x0020, 0x0025, 0x0068, 0x0073, 0x000A,
    0x0000,
];
pub static RUSTCONV_WSTR_AF7911875FAF: [u16; 3] = [0x002Fu16, 0x004C, 0x0000];
pub static RUSTCONV_WSTR_CEA0D44628E8: [u16; 21] = [
    0x0025u16, 0x0030, 0x0031, 0x0036, 0x0049, 0x0036, 0x0034, 0x0058, 0x003A, 0x0020,
    0x0025, 0x0030, 0x0032, 0x0058, 0x0020, 0x0025, 0x0030, 0x0032, 0x0058, 0x000A,
    0x0000,
];
pub static RUSTCONV_WSTR_E430E73AD95B: [u16; 5] = [0x0025u16, 0x0068, 0x0073, 0x000A, 0x0000];
pub static RUSTCONV_WSTR_E6C5E9A6F74F: [u16; 11] = [
    0x0025u16, 0x0035, 0x0064, 0x003A, 0x0020, 0x0020, 0x0025, 0x006C, 0x0073, 0x000A,
    0x0000,
];
pub static RUSTCONV_WSTR_F30666ECB99B: [u16; 4] = [0x004Cu16, 0x0049, 0x0042, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// No file-scope declarations to emit: the includes are handled by the caller, `IS_DOTS` and `HasWildcard` are function-like macros (skipped), and `main` is a function definition.
#[no_mangle]
pub unsafe extern "C" fn NoDifference() -> FCRET {
    ConResPuts(core::ptr::addr_of_mut!(csStdOut), IDS_NO_DIFFERENCE as UINT);
    return FCRET_IDENTICAL;
}
#[no_mangle]
pub unsafe extern "C" fn Different(file0: LPCWSTR, file1: LPCWSTR) -> FCRET {
    ConResPrintf(StdOut, IDS_DIFFERENT, file0, file1);
    return FCRET_DIFFERENT;
}
#[no_mangle]
pub unsafe extern "C" fn LongerThan(file0: LPCWSTR, file1: LPCWSTR) -> FCRET {
    ConResPrintf(core::ptr::addr_of_mut!(csStdOut), IDS_LONGER_THAN as UINT, file0, file1);
    return FCRET_DIFFERENT;
}
#[no_mangle]
pub unsafe extern "C" fn OutOfMemory() -> FCRET {
    ConResPuts(StdErr, IDS_OUT_OF_MEMORY);
    return FCRET_INVALID;
}
#[no_mangle]
pub unsafe extern "C" fn CannotRead(file: LPCWSTR) -> FCRET {
    ConResPrintf(StdErr, IDS_CANNOT_READ as UINT, file);
    return FCRET_INVALID;
}
#[no_mangle]
pub unsafe extern "C" fn InvalidSwitch() -> FCRET {
    ConResPuts(StdErr, IDS_INVALID_SWITCH as UINT);
    return FCRET_INVALID;
}
#[no_mangle]
pub unsafe extern "C" fn ResyncFailed() -> FCRET {
    ConResPuts(StdOut, IDS_RESYNC_FAILED);
    return FCRET_DIFFERENT;
}
#[no_mangle]
pub unsafe extern "C" fn PrintCaption(file: LPCWSTR) {
    ConPrintf(StdOut, RUSTCONV_WSTR_5CDCE2B4827A.as_ptr(), file);
}
#[no_mangle]
pub unsafe extern "C" fn PrintEndOfDiff() {
    ConPuts(StdOut, RUSTCONV_WSTR_9577F614E547.as_ptr());
}
#[no_mangle]
pub unsafe extern "C" fn PrintDots() {
    ConPuts(StdOut, RUSTCONV_WSTR_5BFB8DE9D124.as_ptr());
}
#[no_mangle]
pub unsafe extern "C" fn PrintLineW(pFC: *const FILECOMPARE, lineno: DWORD, psz: LPCWSTR) {
    if ((*pFC).dwFlags & FLAG_N) != 0 {
        ConPrintf(StdOut, RUSTCONV_WSTR_E6C5E9A6F74F.as_ptr(), lineno, psz);
    } else {
        ConPrintf(StdOut, RUSTCONV_WSTR_481F1A6D4120.as_ptr(), psz);
    }
}
#[no_mangle]
pub unsafe extern "C" fn PrintLineA(pFC: *const FILECOMPARE, lineno: DWORD, psz: LPCSTR) {
    if (*pFC).dwFlags & FLAG_N != 0 {
        ConPrintf(StdOut, RUSTCONV_WSTR_9FE91A4E9A02.as_ptr() as PCWSTR, lineno, psz);
    } else {
        ConPrintf(StdOut, RUSTCONV_WSTR_E430E73AD95B.as_ptr() as PCWSTR, psz);
    }
}
#[no_mangle]
pub unsafe extern "C" fn DoOpenFileForInput(file: LPCWSTR) -> HANDLE {
    let mut hFile: HANDLE = CreateFileW(
        file,
        GENERIC_READ as DWORD,
        FILE_SHARE_READ as DWORD,
        core::ptr::null_mut(),
        OPEN_EXISTING as DWORD,
        0 as DWORD,
        core::ptr::null_mut(),
    );
    if hFile == INVALID_HANDLE_VALUE {
        ConResPrintf(
            core::ptr::addr_of_mut!(csStdErr),
            IDS_CANNOT_OPEN as UINT,
            file,
        );
    }
    return hFile;
}

unsafe extern "C" fn BinaryFileCompare(pFC: *mut FILECOMPARE) -> FCRET {
    let mut ret: FCRET;
    let mut hFile0: HANDLE;
    let mut hFile1: HANDLE;
    let mut hMapping0: HANDLE = core::ptr::null_mut();
    let mut hMapping1: HANDLE = core::ptr::null_mut();
    let mut pb0: LPBYTE = core::ptr::null_mut();
    let mut pb1: LPBYTE = core::ptr::null_mut();
    let mut ib: LARGE_INTEGER = core::mem::zeroed();
    let mut cb0: LARGE_INTEGER = core::mem::zeroed();
    let mut cb1: LARGE_INTEGER = core::mem::zeroed();
    let mut cbCommon: LARGE_INTEGER = core::mem::zeroed();
    let mut cbView: DWORD = 0;
    let mut ibView: DWORD = 0;
    let mut fDifferent: BOOL = (FALSE as i32);

    hFile0 = DoOpenFileForInput((*pFC).file[0 as usize]);
    if hFile0 == INVALID_HANDLE_VALUE {
        return FCRET_CANT_FIND;
    }
    hFile1 = DoOpenFileForInput((*pFC).file[1 as usize]);
    if hFile1 == INVALID_HANDLE_VALUE {
        CloseHandle(hFile0);
        return FCRET_CANT_FIND;
    }

    loop {
        if _wcsicmp(
            (*pFC).file[0 as usize],
            (*pFC).file[1 as usize],
        ) == 0
        {
            ret = NoDifference();
            break;
        }
        if GetFileSizeEx(hFile0, core::ptr::addr_of_mut!(cb0)) == 0 {
            ret = CannotRead((*pFC).file[0 as usize]);
            break;
        }
        if GetFileSizeEx(hFile1, core::ptr::addr_of_mut!(cb1)) == 0 {
            ret = CannotRead((*pFC).file[1 as usize]);
            break;
        }
        cbCommon.QuadPart = min!(cb0.QuadPart, cb1.QuadPart);
        if cbCommon.QuadPart > 0 as LONGLONG {
            hMapping0 = CreateFileMappingW(
                hFile0,
                core::ptr::null_mut(),
                PAGE_READONLY as DWORD,
                cb0.__bindgen_anon_1.HighPart as DWORD,
                cb0.__bindgen_anon_1.LowPart,
                core::ptr::null(),
            );
            if hMapping0.is_null() {
                ret = CannotRead((*pFC).file[0 as usize]);
                break;
            }
            hMapping1 = CreateFileMappingW(
                hFile1,
                core::ptr::null_mut(),
                PAGE_READONLY as DWORD,
                cb1.__bindgen_anon_1.HighPart as DWORD,
                cb1.__bindgen_anon_1.LowPart,
                core::ptr::null(),
            );
            if hMapping1.is_null() {
                ret = CannotRead((*pFC).file[1 as usize]);
                break;
            }

            ret = FCRET_IDENTICAL;
            ib.QuadPart = 0 as LONGLONG;
            while ib.QuadPart < cbCommon.QuadPart {
                cbView = min!(
                    cbCommon.QuadPart - ib.QuadPart,
                    MAX_VIEW_SIZE as LONGLONG,
                ) as DWORD;
                pb0 = MapViewOfFile(
                    hMapping0,
                    FILE_MAP_READ as DWORD,
                    ib.__bindgen_anon_1.HighPart as DWORD,
                    ib.__bindgen_anon_1.LowPart,
                    cbView as SIZE_T,
                ) as LPBYTE;
                pb1 = MapViewOfFile(
                    hMapping1,
                    FILE_MAP_READ as DWORD,
                    ib.__bindgen_anon_1.HighPart as DWORD,
                    ib.__bindgen_anon_1.LowPart,
                    cbView as SIZE_T,
                ) as LPBYTE;
                if pb0.is_null() || pb1.is_null() {
                    ret = OutOfMemory();
                    break;
                }
                ibView = 0 as DWORD;
                while ibView < cbView {
                    if !(*pb0.add(ibView as usize) == *pb1.add(ibView as usize)) {
                        fDifferent = (TRUE as i32);
                        if cbCommon.QuadPart > MAXDWORD as LONGLONG {
                            ConPrintf(
                                StdOut,
                                RUSTCONV_WSTR_CEA0D44628E8.as_ptr(),
                                ib.QuadPart,
                                *pb0.add(ibView as usize) as core::ffi::c_uint,
                                *pb1.add(ibView as usize) as core::ffi::c_uint,
                            );
                        } else {
                            ConPrintf(
                                StdOut,
                                RUSTCONV_WSTR_31A10D3954D0.as_ptr(),
                                ib.__bindgen_anon_1.LowPart,
                                *pb0.add(ibView as usize) as core::ffi::c_uint,
                                *pb1.add(ibView as usize) as core::ffi::c_uint,
                            );
                        }
                    }
                    ib.QuadPart += 1;
                    ibView += 1;
                }
                UnmapViewOfFile(pb0 as LPCVOID);
                UnmapViewOfFile(pb1 as LPCVOID);
                pb1 = core::ptr::null_mut();
                pb0 = pb1;
            }
            if ret != FCRET_IDENTICAL {
                break;
            }
        }

        if cb0.QuadPart < cb1.QuadPart {
            ret = LongerThan(
                (*pFC).file[1 as usize],
                (*pFC).file[0 as usize],
            );
        } else if cb0.QuadPart > cb1.QuadPart {
            ret = LongerThan(
                (*pFC).file[0 as usize],
                (*pFC).file[1 as usize],
            );
        } else if fDifferent != 0 {
            ret = Different(
                (*pFC).file[0 as usize],
                (*pFC).file[1 as usize],
            );
        } else {
            ret = NoDifference();
        }

        if !((0) != 0) {
            break;
        }
    }

    UnmapViewOfFile(pb0 as LPCVOID);
    UnmapViewOfFile(pb1 as LPCVOID);
    CloseHandle(hMapping0);
    CloseHandle(hMapping1);
    CloseHandle(hFile0);
    CloseHandle(hFile1);
    return ret;
}

unsafe extern "C" fn TextFileCompare(pFC: *mut FILECOMPARE) -> FCRET {
    let mut ret: FCRET = FCRET_IDENTICAL;
    let mut hFile0: HANDLE;
    let mut hFile1: HANDLE;
    let mut hMapping0: HANDLE = core::ptr::null_mut();
    let mut hMapping1: HANDLE = core::ptr::null_mut();
    let mut cb0: LARGE_INTEGER = core::mem::zeroed();
    let mut cb1: LARGE_INTEGER = core::mem::zeroed();
    let fUnicode: BOOL = (((*pFC).dwFlags & FLAG_U) != 0) as BOOL;

    hFile0 = DoOpenFileForInput((*pFC).file[0 as usize]);
    if hFile0 == INVALID_HANDLE_VALUE {
        return FCRET_CANT_FIND;
    }
    hFile1 = DoOpenFileForInput((*pFC).file[1 as usize]);
    if hFile1 == INVALID_HANDLE_VALUE {
        CloseHandle(hFile0);
        return FCRET_CANT_FIND;
    }

    loop {
        if _wcsicmp(
            (*pFC).file[0 as usize],
            (*pFC).file[1 as usize],
        ) == 0
        {
            ret = NoDifference();
            break;
        }
        if GetFileSizeEx(hFile0, core::ptr::addr_of_mut!(cb0)) == 0 {
            ret = CannotRead((*pFC).file[0 as usize]);
            break;
        }
        if GetFileSizeEx(hFile1, core::ptr::addr_of_mut!(cb1)) == 0 {
            ret = CannotRead((*pFC).file[1 as usize]);
            break;
        }
        if cb0.QuadPart == 0 && cb1.QuadPart == 0 {
            ret = NoDifference();
            break;
        }
        if cb0.QuadPart > 0 {
            hMapping0 = CreateFileMappingW(
                hFile0,
                core::ptr::null_mut(),
                PAGE_READONLY as DWORD,
                cb0.__bindgen_anon_1.HighPart as DWORD,
                cb0.__bindgen_anon_1.LowPart,
                core::ptr::null::<WCHAR>(),
            );
            if hMapping0.is_null() {
                ret = CannotRead((*pFC).file[0 as usize]);
                break;
            }
        }
        if cb1.QuadPart > 0 {
            hMapping1 = CreateFileMappingW(
                hFile1,
                core::ptr::null_mut(),
                PAGE_READONLY as DWORD,
                cb1.__bindgen_anon_1.HighPart as DWORD,
                cb1.__bindgen_anon_1.LowPart,
                core::ptr::null::<WCHAR>(),
            );
            if hMapping1.is_null() {
                ret = CannotRead((*pFC).file[1 as usize]);
                break;
            }
        }
        if fUnicode != 0 {
            ret = TextCompareW(
                pFC,
                core::ptr::addr_of_mut!(hMapping0),
                core::ptr::addr_of_mut!(cb0),
                core::ptr::addr_of_mut!(hMapping1),
                core::ptr::addr_of_mut!(cb1),
            );
        } else {
            ret = TextCompareA(
                pFC,
                core::ptr::addr_of_mut!(hMapping0),
                core::ptr::addr_of_mut!(cb0),
                core::ptr::addr_of_mut!(hMapping1),
                core::ptr::addr_of_mut!(cb1),
            );
        }
        if 0 == 0 {
            break;
        }
    }

    CloseHandle(hMapping0);
    CloseHandle(hMapping1);
    CloseHandle(hFile0);
    CloseHandle(hFile1);
    return ret;
}

unsafe extern "C" fn IsBinaryExt(filename: LPCWSTR) -> BOOL {
    // Don't change this array. This is by design.
    // See also: https://docs.microsoft.com/en-us/windows-server/administration/windows-commands/fc
    static mut s_exts: [LPCWSTR; 6] = [
        RUSTCONV_WSTR_8C7CF1E9BC62.as_ptr(),
        RUSTCONV_WSTR_3DB566BC3DA4.as_ptr(),
        RUSTCONV_WSTR_578D89EBADFC.as_ptr(),
        RUSTCONV_WSTR_55B8302A0040.as_ptr(),
        RUSTCONV_WSTR_F30666ECB99B.as_ptr(),
        RUSTCONV_WSTR_27D452FB378E.as_ptr(),
    ];
    let mut iext: size_t;
    let mut pch: LPCWSTR;
    let mut ext: LPCWSTR;
    let pch0: LPCWSTR = wcsrchr(filename, 0x005Cu16) as LPCWSTR;
    let pch1: LPCWSTR = wcsrchr(filename, 0x002Fu16) as LPCWSTR;
    if pch0.is_null() && pch1.is_null() {
        pch = filename;
    } else if pch0.is_null() && !pch1.is_null() {
        pch = pch1;
    } else if !pch0.is_null() && pch1.is_null() {
        pch = pch0;
    } else if pch0 < pch1 {
        pch = pch1;
    } else {
        pch = pch0;
    }

    ext = wcsrchr(pch, 0x002Eu16) as LPCWSTR;
    if !ext.is_null() {
        ext = ext.offset(1);
        iext = 0;
        while iext < _countof!(s_exts) as usize {
            if _wcsicmp(ext, s_exts[iext as usize]) == 0 {
                return (TRUE) as BOOL;
            }
            iext += 1;
        }
    }
    return (FALSE) as BOOL;
}

unsafe extern "C" fn FileCompare(pFC: *mut FILECOMPARE) -> FCRET {
    let ret: FCRET;
    ConResPrintf(
        &raw mut csStdOut,
        IDS_COMPARING as UINT,
        (*pFC).file[0 as usize],
        (*pFC).file[1 as usize],
    );

    if ((*pFC).dwFlags & (FLAG_L as DWORD)) == 0
        && (((*pFC).dwFlags & (FLAG_B as DWORD)) != 0
            || IsBinaryExt((*pFC).file[0 as usize]) != 0
            || IsBinaryExt((*pFC).file[1 as usize]) != 0)
    {
        ret = BinaryFileCompare(pFC);
    } else {
        ret = TextFileCompare(pFC);
    }

    ConPuts(&raw mut csStdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());
    return ret;
}

unsafe extern "C" fn IsTitleWild(filename: LPCWSTR) -> BOOL {
    let pch: LPCWSTR = PathFindFileNameW(filename) as LPCWSTR;
    return (!pch.is_null()
        && *pch == 0x002Au16
        && *pch.offset(1 as isize) == 0x002Eu16
        && HasWildcard!(pch.offset(2 as isize)) == false) as BOOL;
}

unsafe extern "C" fn FileCompareOneSideWild(pFC: *const FILECOMPARE, bWildRight: BOOL) -> FCRET {
    let mut ret: FCRET = FCRET_IDENTICAL;
    let mut find: WIN32_FIND_DATAW = core::mem::zeroed();
    let mut hFind: HANDLE = core::ptr::null_mut::<core::ffi::c_void>();
    let mut szPath: [WCHAR; (MAX_PATH) as _] = core::mem::zeroed();
    let mut fc: FILECOMPARE = core::mem::zeroed();

    hFind = FindFirstFileW((*pFC).file[bWildRight as usize], core::ptr::addr_of_mut!(find));
    if hFind == INVALID_HANDLE_VALUE {
        ConResPrintf(
            StdErr,
            IDS_CANNOT_OPEN as UINT,
            (*pFC).file[bWildRight as usize],
        );
        ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());
        return FCRET_CANT_FIND;
    }
    StringCbCopyW(
        core::ptr::addr_of_mut!(szPath) as *mut WCHAR as STRSAFE_LPWSTR,
        core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>() as size_t,
        (*pFC).file[bWildRight as usize],
    );

    fc = *pFC;
    fc.file[(bWildRight == 0) as usize] = (*pFC).file[(bWildRight == 0) as usize];
    fc.file[bWildRight as usize] = core::ptr::addr_of_mut!(szPath) as *mut WCHAR as LPCWSTR;
    loop {
        'body: {
            let pch: *mut WCHAR = core::ptr::addr_of_mut!(find.cFileName) as *mut WCHAR;
            if !(*pch == '.' as u16
                && (*pch.add(1) == 0
                    || (*pch.add(1) == '.' as u16 && *pch.add(2) == 0)))
            {
                // replace file title
                PathRemoveFileSpecW(core::ptr::addr_of_mut!(szPath) as *mut WCHAR as LPWSTR);
                PathAppendW(
                    core::ptr::addr_of_mut!(szPath) as *mut WCHAR as LPWSTR,
                    core::ptr::addr_of_mut!(find.cFileName) as *mut WCHAR as LPCWSTR,
                );

                match FileCompare(core::ptr::addr_of_mut!(fc)) {
                    FCRET_IDENTICAL => {
                        break 'body;
                    }
                    FCRET_DIFFERENT => {
                        if ret != FCRET_INVALID {
                            ret = FCRET_DIFFERENT;
                        }
                        break 'body;
                    }
                    _ => {
                        ret = FCRET_INVALID;
                        break 'body;
                    }
                }
            }
        }
        if !(FindNextFileW(hFind, core::ptr::addr_of_mut!(find)) != 0) {
            break;
        }
    }

    FindClose(hFind);
    return ret;
}

unsafe extern "C" fn FileCompareWildTitle(pFC: *const FILECOMPARE) -> FCRET {
    let mut ret: FCRET = FCRET_IDENTICAL;
    let mut find: WIN32_FIND_DATAW = core::mem::zeroed();
    let mut hFind: HANDLE = core::ptr::null_mut();
    let mut szPath0: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut szPath1: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut fc: FILECOMPARE = core::mem::zeroed();
    let mut pch: LPWSTR = core::ptr::null_mut();

    hFind = FindFirstFileW((*pFC).file[0 as usize], &raw mut find);
    if hFind == INVALID_HANDLE_VALUE {
        ConResPrintf(
            &raw mut csStdErr,
            IDS_CANNOT_OPEN as UINT,
            (*pFC).file[0 as usize],
        );
        ConPuts(&raw mut csStdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());
        return FCRET_CANT_FIND;
    }
    StringCbCopyW(
        &raw mut szPath0 as STRSAFE_LPWSTR,
        core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>() as size_t,
        (*pFC).file[0 as usize],
    );
    StringCbCopyW(
        &raw mut szPath1 as STRSAFE_LPWSTR,
        core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>() as size_t,
        (*pFC).file[1 as usize],
    );
    pch = PathFindExtensionW((*pFC).file[1 as usize]);

    fc = *pFC;
    fc.file[0 as usize] = &raw mut szPath0 as *mut WCHAR as LPCWSTR;
    fc.file[1 as usize] = &raw mut szPath1 as *mut WCHAR as LPCWSTR;
    loop {
        'body: {
            if (*(&raw mut find.cFileName as *mut WCHAR) as i32 == '.' as i32
                && (find.cFileName[1] as i32 == 0
                    || find.cFileName[1] as i32 == '.' as i32
                        && find.cFileName[2] as i32 == 0))
            {
                break 'body;
            }

            // replace file title
            PathRemoveFileSpecW(&raw mut szPath0 as LPWSTR);
            PathRemoveFileSpecW(&raw mut szPath1 as LPWSTR);
            PathAppendW(
                &raw mut szPath0 as LPWSTR,
                &raw mut find.cFileName as *mut WCHAR as LPCWSTR,
            );
            PathAppendW(
                &raw mut szPath1 as LPWSTR,
                &raw mut find.cFileName as *mut WCHAR as LPCWSTR,
            );

            // replace dot extension
            PathRemoveExtensionW(&raw mut szPath1 as LPWSTR);
            PathAddExtensionW(&raw mut szPath1 as LPWSTR, pch as LPCWSTR);

            match FileCompare(&raw mut fc) as i32 {
                FCRET_IDENTICAL => {}
                FCRET_DIFFERENT => {
                    if ret as i32 != FCRET_INVALID {
                        ret = FCRET_DIFFERENT;
                    }
                }
                _ => {
                    ret = FCRET_INVALID;
                }
            }
        }
        if !(FindNextFileW(hFind, &raw mut find) != 0) {
            break;
        }
    }

    FindClose(hFind);
    return ret;
}

unsafe extern "C" fn FileCompareBothWild(pFC: *const FILECOMPARE) -> FCRET {
    let mut ret: FCRET = FCRET_IDENTICAL;
    let mut find0: WIN32_FIND_DATAW = core::mem::zeroed();
    let mut find1: WIN32_FIND_DATAW = core::mem::zeroed();
    let mut hFind0: HANDLE = core::ptr::null_mut();
    let mut hFind1: HANDLE = core::ptr::null_mut();
    let mut szPath0: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut szPath1: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut fc: FILECOMPARE = core::mem::zeroed();

    hFind0 = FindFirstFileW(
        (*pFC).file[0 as usize],
        core::ptr::addr_of_mut!(find0),
    );
    if hFind0 == INVALID_HANDLE_VALUE {
        ConResPrintf(
            StdErr,
            IDS_CANNOT_OPEN,
            (*pFC).file[0 as usize],
        );
        ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());
        return FCRET_CANT_FIND;
    }
    hFind1 = FindFirstFileW(
        (*pFC).file[1 as usize],
        core::ptr::addr_of_mut!(find1),
    );
    if hFind1 == INVALID_HANDLE_VALUE {
        // DIVERGENCE: the C closes its find handles with CloseHandle, which does not
        // release a FindFirstFileW search (its two sibling functions use FindClose).
        FindClose(hFind0);
        ConResPrintf(
            StdErr,
            IDS_CANNOT_OPEN,
            (*pFC).file[1 as usize],
        );
        ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());
        return FCRET_CANT_FIND;
    }
    StringCbCopyW(
        core::ptr::addr_of_mut!(szPath0) as *mut WCHAR,
        core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>() as size_t,
        (*pFC).file[0 as usize],
    );
    StringCbCopyW(
        core::ptr::addr_of_mut!(szPath1) as *mut WCHAR,
        core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>() as size_t,
        (*pFC).file[1 as usize],
    );

    fc = *pFC;
    fc.file[0 as usize] =
        core::ptr::addr_of_mut!(szPath0) as *mut WCHAR as LPCWSTR;
    fc.file[1 as usize] =
        core::ptr::addr_of_mut!(szPath1) as *mut WCHAR as LPCWSTR;
    'quit: {
        loop {
            while IS_DOTS!(core::ptr::addr_of_mut!(find0.cFileName) as *mut WCHAR) != false {
                if FindNextFileW(hFind0, core::ptr::addr_of_mut!(find0)) == 0 {
                    break 'quit;
                }
            }
            while IS_DOTS!(core::ptr::addr_of_mut!(find1.cFileName) as *mut WCHAR) != false {
                if FindNextFileW(hFind1, core::ptr::addr_of_mut!(find1)) == 0 {
                    break 'quit;
                }
            }

            // replace file title
            PathRemoveFileSpecW(core::ptr::addr_of_mut!(szPath0) as *mut WCHAR as LPWSTR);
            PathRemoveFileSpecW(core::ptr::addr_of_mut!(szPath1) as *mut WCHAR as LPWSTR);
            PathAppendW(
                core::ptr::addr_of_mut!(szPath0) as *mut WCHAR as LPWSTR,
                core::ptr::addr_of_mut!(find0.cFileName) as *mut WCHAR as LPCWSTR,
            );
            PathAppendW(
                core::ptr::addr_of_mut!(szPath1) as *mut WCHAR as LPWSTR,
                core::ptr::addr_of_mut!(find1.cFileName) as *mut WCHAR as LPCWSTR,
            );

            match FileCompare(core::ptr::addr_of_mut!(fc)) as i32 {
                FCRET_IDENTICAL => {}
                FCRET_DIFFERENT => {
                    if ret != FCRET_INVALID {
                        ret = FCRET_DIFFERENT;
                    }
                }
                _ => {
                    ret = FCRET_INVALID;
                }
            }
            if !(FindNextFileW(hFind0, core::ptr::addr_of_mut!(find0)) != 0
                && FindNextFileW(hFind1, core::ptr::addr_of_mut!(find1)) != 0)
            {
                break;
            }
        }
    }
    FindClose(hFind0);
    FindClose(hFind1);
    return ret;
}

unsafe extern "C" fn WildcardFileCompare(pFC: *mut FILECOMPARE) -> FCRET {
    let mut fWild0: BOOL = 0;
    let mut fWild1: BOOL = 0;

    if ((*pFC).dwFlags & (FLAG_HELP as DWORD)) != 0 {
        ConResPuts(core::ptr::addr_of_mut!(csStdOut), IDS_USAGE as UINT);
        return FCRET_INVALID;
    }

    if (*pFC).file[0 as usize].is_null()
        || (*pFC).file[1 as usize].is_null()
    {
        ConResPuts(core::ptr::addr_of_mut!(csStdErr), IDS_NEEDS_FILES as UINT);
        return FCRET_INVALID;
    }

    fWild0 = HasWildcard!((*pFC).file[0 as usize]) as BOOL;
    fWild1 = HasWildcard!((*pFC).file[1 as usize]) as BOOL;
    if fWild0 != 0 && fWild1 != 0 {
        if IsTitleWild((*pFC).file[0 as usize]) != 0
            && IsTitleWild((*pFC).file[1 as usize]) != 0
        {
            return FileCompareWildTitle(pFC);
        } else {
            return FileCompareBothWild(pFC);
        }
    } else if fWild0 != 0 {
        return FileCompareOneSideWild(pFC, FALSE as BOOL);
    } else if fWild1 != 0 {
        return FileCompareOneSideWild(pFC, TRUE as BOOL);
    }
    return FileCompare(pFC);
}

#[no_mangle]
pub unsafe extern "C" fn wmain(argc: INT, argv: *mut *mut WCHAR) -> INT {
    let mut fc: FILECOMPARE = core::mem::zeroed();
    fc.dwFlags = 0;
    fc.n = 100;
    fc.nnnn = 2;
    let mut endptr: PWCHAR = core::ptr::null_mut();
    let mut i: INT;

    /* Initialize the Console Standard Streams */
    ConInitStdStreams();

    i = 1 as INT;
    while i < argc {
        'body: {
            if *(*argv.offset(i as isize)).offset(0 as isize) != 0x002Fu16 {
                if fc.file[0 as usize].is_null() {
                    fc.file[0 as usize] = *argv.offset(i as isize) as LPCWSTR;
                } else if fc.file[1 as usize].is_null() {
                    fc.file[1 as usize] = *argv.offset(i as isize) as LPCWSTR;
                } else {
                    return InvalidSwitch() as INT;
                }
                break 'body;
            }
            match towupper((*(*argv.offset(i as isize)).offset(1 as isize)) as _) {
                0x0041u16 => {
                    fc.dwFlags |= FLAG_A;
                }
                0x0042u16 => {
                    fc.dwFlags |= FLAG_B;
                }
                0x0043u16 => {
                    fc.dwFlags |= FLAG_C;
                }
                0x004Cu16 => {
                    if _wcsicmp(*argv.offset(i as isize), RUSTCONV_WSTR_AF7911875FAF.as_ptr()) == 0 {
                        fc.dwFlags |= FLAG_L;
                    } else if towupper((*(*argv.offset(i as isize)).offset(2 as isize)) as _) == (0x0042u16) as _ {
                        if iswdigit(*(*argv.offset(i as isize)).offset(3 as isize)) != 0 {
                            fc.dwFlags |= FLAG_LBn;
                            fc.n = wcstoul(
                                (*argv.offset(i as isize)).offset(3 as isize),
                                core::ptr::addr_of_mut!(endptr),
                                10,
                            ) as INT;
                            if endptr.is_null() || *endptr != 0 {
                                return InvalidSwitch() as INT;
                            }
                        } else {
                            return InvalidSwitch() as INT;
                        }
                    }
                }
                0x004Eu16 => {
                    fc.dwFlags |= FLAG_N;
                }
                0x004Fu16 => {
                    if _wcsicmp(*argv.offset(i as isize), RUSTCONV_WSTR_2EBEB4925984.as_ptr()) == 0
                        || _wcsicmp(*argv.offset(i as isize), RUSTCONV_WSTR_47AEC1CCED2E.as_ptr()) == 0
                    {
                        fc.dwFlags |= FLAG_OFFLINE;
                    }
                }
                0x0054u16 => {
                    fc.dwFlags |= FLAG_T;
                }
                0x0055u16 => {
                    fc.dwFlags |= FLAG_U;
                }
                0x0057u16 => {
                    fc.dwFlags |= FLAG_W;
                }
                0x0030u16
                | 0x0031u16
                | 0x0032u16
                | 0x0033u16
                | 0x0034u16
                | 0x0035u16
                | 0x0036u16
                | 0x0037u16
                | 0x0038u16
                | 0x0039u16 => {
                    fc.nnnn = wcstoul(
                        (*argv.offset(i as isize)).offset(1 as isize),
                        core::ptr::addr_of_mut!(endptr),
                        10,
                    ) as INT;
                    if endptr.is_null() || *endptr != 0 {
                        return InvalidSwitch() as INT;
                    }
                    fc.dwFlags |= FLAG_nnnn;
                }
                0x003Fu16 => {
                    fc.dwFlags |= FLAG_HELP;
                }
                _ => {
                    return InvalidSwitch() as INT;
                }
            }
        }
        i += 1;
    }
    return WildcardFileCompare(core::ptr::addr_of_mut!(fc)) as INT;
}
