use crate::cmd::console::ConOutPrintfPaging;
use crate::cmd::console::ConOutResPaging;
use crate::cmd::console::ConPrintfVPaging;
use crate::cmd::cmd::ConvertULargeInteger;
use crate::cmd::console::GetScreenSize;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use ntdll_sys::nt::*;
pub static RUSTCONV_WSTR_96A296D224F2: [u16; 1] = [0x0000u16];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut recurse_dir_cnt: ULONG = 0;

static mut recurse_file_cnt: ULONG = 0;

static mut recurse_bytes: ULONGLONG = 0;

pub const TF_CREATIONDATE: i32 = 0;
pub const TF_MODIFIEDDATE: i32 = 1;
pub const TF_LASTACCESSEDDATE: i32 = 2;

pub const ORDER_NAME: i32 = 0;
pub const ORDER_SIZE: i32 = 1;
pub const ORDER_DIRECTORY: i32 = 2;
pub const ORDER_EXTENSION: i32 = 3;
pub const ORDER_TIME: i32 = 4;

#[repr(C)]
#[allow(non_snake_case, non_camel_case_types)]
pub struct _DirSwitchesFlags_stAttribs {
    pub dwAttribVal: DWORD,
    pub dwAttribMask: DWORD,
}

#[repr(C)]
#[allow(non_snake_case, non_camel_case_types)]
pub struct _DirSwitchesFlags_stOrderBy {
    pub eCriteria: [i32; 3],
    pub bCriteriaRev: [BOOL; 3],
    pub sCriteriaCount: i16,
}

#[repr(C)]
#[allow(non_snake_case, non_camel_case_types)]
pub struct _DirSwitchesFlags_stTimeField {
    pub eTimeField: i32,
}

#[repr(C)]
#[allow(non_snake_case, non_camel_case_types)]
pub struct _DirSwitchesFlags {
    pub bBareFormat: BOOL,
    pub bTSeparator: BOOL,
    pub bWideList: BOOL,
    pub bWideListColSort: BOOL,
    pub bLowerCase: BOOL,
    pub bNewLongList: BOOL,
    pub bPause: BOOL,
    pub bUser: BOOL,
    pub bRecursive: BOOL,
    pub bShortName: BOOL,
    pub b4Digit: BOOL,
    pub bDataStreams: BOOL,
    pub stAttribs: _DirSwitchesFlags_stAttribs,
    pub stOrderBy: _DirSwitchesFlags_stOrderBy,
    pub stTimeField: _DirSwitchesFlags_stTimeField,
}
pub type DIRSWITCHFLAGS = _DirSwitchesFlags;
pub type LPDIRSWITCHFLAGS = *mut DIRSWITCHFLAGS;

#[repr(C)]
#[allow(non_snake_case, non_camel_case_types)]
pub struct _DIRFINDSTREAMNODE {
    pub stStreamInfo: WIN32_FIND_STREAM_DATA,
    pub ptrNext: *mut _DIRFINDSTREAMNODE,
}
pub type DIRFINDSTREAMNODE = _DIRFINDSTREAMNODE;
pub type PDIRFINDSTREAMNODE = *mut DIRFINDSTREAMNODE;

#[repr(C)]
#[allow(non_snake_case, non_camel_case_types)]
pub struct _DIRFINDINFO {
    pub stFindInfo: WIN32_FIND_DATA,
    pub ptrHead: PDIRFINDSTREAMNODE,
}
pub type DIRFINDINFO = _DIRFINDINFO;
pub type PDIRFINDINFO = *mut DIRFINDINFO;

#[repr(C)]
#[allow(non_snake_case, non_camel_case_types)]
pub struct _DIRFINDLISTNODE {
    pub stInfo: DIRFINDINFO,
    pub ptrNext: *mut _DIRFINDLISTNODE,
}
pub type DIRFINDLISTNODE = _DIRFINDLISTNODE;
pub type PDIRFINDLISTNODE = *mut DIRFINDLISTNODE;

pub type PGETFREEDISKSPACEEX = unsafe extern "system" fn(
    LPCTSTR,
    PULARGE_INTEGER,
    PULARGE_INTEGER,
    PULARGE_INTEGER,
) -> BOOL;
unsafe extern "C" fn DirHelp() {
    ConOutResPaging(TRUE, STRING_DIR_HELP1);
}
#[no_mangle]
pub unsafe extern "C" fn IsDotDirectory(pszPath: LPCTSTR) -> BOOL {
    return (*pszPath.offset(0 as isize) == '.' as TCHAR
        && (*pszPath.offset(1 as isize) == 0 as TCHAR
            || (*pszPath.offset(1 as isize) == '.' as TCHAR
                && *pszPath.offset(2 as isize) == 0 as TCHAR))) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn IsDotDirectoryN(pPath: *const TCHAR, Length: SIZE_T) -> BOOL {
    return ((Length == 1 && *pPath.add(0) == _T('.')) || (Length == 2 && *pPath.add(0) == _T('.') && *pPath.add(1) == _T('.'))) as BOOL;
}

unsafe extern "C" fn DirReadParam(Line: LPTSTR, params: *mut LPTSTR, entries: LPINT, lpFlags: LPDIRSWITCHFLAGS) -> BOOL {
    let mut Line = Line;
    let mut cCurSwitch: TCHAR = 0;
    let mut cCurChar: TCHAR = 0;
    let mut cCurUChar: TCHAR = 0;
    let mut bNegative: BOOL = 0;
    let mut bPNegative: BOOL = 0;
    let mut bIntoQuotes: BOOL = 0;
    let mut ptrStart: LPTSTR = core::ptr::null_mut();
    let mut ptrEnd: LPTSTR = core::ptr::null_mut();
    let mut bOrderByNoPar: BOOL = 0;
    let mut temp: LPTSTR = core::ptr::null_mut();

    /* Initialize parameter array */
    *params = core::ptr::null_mut();
    *entries = 0;

    /* Initialize variables; */
    cCurSwitch = _T(' ');
    bNegative = FALSE;
    bPNegative = FALSE;

    bOrderByNoPar = FALSE;

    while *Line != 0 {
        cCurChar = *Line;
        cCurUChar = _totupper(*Line);

        if cCurSwitch == _T('/') {
            while _istspace(*Line) != 0 {
                Line = Line.add(1);
            }

            bNegative = (*Line == _T('-')) as BOOL;
            Line = Line.offset(bNegative as isize);

            cCurChar = *Line;
            cCurUChar = _totupper(*Line);

            if cCurUChar == _T('A') || cCurUChar == _T('T') || cCurUChar == _T('O') {
                match cCurUChar {
                    _ if cCurUChar == _T('A') => {
                        (*lpFlags).stAttribs.dwAttribVal = 0;
                        (*lpFlags).stAttribs.dwAttribMask = 0;
                        if bNegative != 0 {
                            (*lpFlags).stAttribs.dwAttribMask = (FILE_ATTRIBUTE_HIDDEN | FILE_ATTRIBUTE_SYSTEM);
                        }
                    }
                    _ if cCurUChar == _T('T') => {
                        if bNegative != 0 {
                            (*lpFlags).stTimeField.eTimeField = TF_MODIFIEDDATE;
                        }
                    }
                    _ if cCurUChar == _T('O') => {
                        bOrderByNoPar = (bNegative == 0) as BOOL;
                        (*lpFlags).stOrderBy.sCriteriaCount = 0;
                    }
                    _ => {}
                }

                if bNegative == 0 {
                    cCurSwitch = cCurUChar;
                    Line = Line.add(1);
                    if *Line == _T(':') {
                        Line = Line.add(1);
                    }
                    continue;
                }
            } else if cCurUChar == _T('L') {
                (*lpFlags).bLowerCase = (bNegative == 0) as BOOL;
            } else if cCurUChar == _T('B') {
                (*lpFlags).bBareFormat = (bNegative == 0) as BOOL;
            } else if cCurUChar == _T('C') {
                (*lpFlags).bTSeparator = (bNegative == 0) as BOOL;
            } else if cCurUChar == _T('W') {
                (*lpFlags).bWideList = (bNegative == 0) as BOOL;
            } else if cCurUChar == _T('D') {
                (*lpFlags).bWideListColSort = (bNegative == 0) as BOOL;
            } else if cCurUChar == _T('N') {
                (*lpFlags).bNewLongList = (bNegative == 0) as BOOL;
            } else if cCurUChar == _T('P') {
                (*lpFlags).bPause = (bNegative == 0) as BOOL;
            } else if cCurUChar == _T('Q') {
                (*lpFlags).bUser = (bNegative == 0) as BOOL;
            } else if cCurUChar == _T('S') {
                (*lpFlags).bRecursive = (bNegative == 0) as BOOL;
            } else if cCurUChar == _T('X') {
                (*lpFlags).bShortName = (bNegative == 0) as BOOL;
            } else if cCurUChar == _T('R') {
                (*lpFlags).bDataStreams = (bNegative == 0) as BOOL;
            } else if cCurChar == _T('4') {
                (*lpFlags).b4Digit = (bNegative == 0) as BOOL;
            } else if cCurChar == _T('?') {
                DirHelp();
                return FALSE;
            } else {
                error_invalid_switch(_totupper(*Line) as TCHAR);
                return FALSE;
            }

            /* Make sure there's no extra characters at the end of the switch */
            if *Line.add(1 as usize) != 0
                && *Line.add(1 as usize) != _T('/')
                && _istspace(*Line.add(1 as usize)) == 0
            {
                error_parameter_format(*Line.add(1 as usize));
                return FALSE;
            }

            cCurSwitch = _T(' ');
        } else if cCurSwitch == _T(' ') {
            if cCurChar == _T('/') {
                cCurSwitch = _T('/');
            } else if _istspace(cCurChar) != 0 {
                /* do nothing */
            } else {
                ptrStart = Line;
                bIntoQuotes = FALSE;
                while *Line != 0 {
                    if bIntoQuotes == 0 && (*Line == _T('/') || _istspace(*Line) != 0) {
                        break;
                    }
                    bIntoQuotes ^= (*Line == _T('"')) as BOOL;
                    Line = Line.add(1);
                }
                ptrEnd = Line;

                temp = cmd_alloc(((ptrEnd.offset_from(ptrStart) + 1) as usize) * (core::mem::size_of::<TCHAR>() as usize));
                if temp.is_null() {
                    return FALSE;
                }
                memcpy(
                    temp as *mut core::ffi::c_void,
                    ptrStart as *const core::ffi::c_void,
                    (ptrEnd.offset_from(ptrStart) as usize) * (core::mem::size_of::<TCHAR>() as usize),
                );
                *temp.offset(ptrEnd.offset_from(ptrStart)) = _T('\0');
                StripQuotes(temp);
                if add_entry(entries, params, temp) == 0 {
                    cmd_free(temp);
                    freep(*params);
                    return FALSE;
                }

                cmd_free(temp);
                continue;
            }
        } else {
            if cCurChar == _T('/') || _istspace(cCurChar) != 0 {
                cCurSwitch = _T(' ');
                continue;
            }
            match cCurSwitch {
                _ if cCurSwitch == _T('A') => {
                    if cCurChar == _T('-') {
                        bPNegative = TRUE;
                    } else if cCurUChar == _T('D') {
                        (*lpFlags).stAttribs.dwAttribMask |= (FILE_ATTRIBUTE_DIRECTORY as DWORD);
                        if bPNegative != 0 {
                            (*lpFlags).stAttribs.dwAttribVal &= !(FILE_ATTRIBUTE_DIRECTORY as DWORD);
                        } else {
                            (*lpFlags).stAttribs.dwAttribVal |= (FILE_ATTRIBUTE_DIRECTORY as DWORD);
                        }
                    } else if cCurUChar == _T('R') {
                        (*lpFlags).stAttribs.dwAttribMask |= (FILE_ATTRIBUTE_READONLY as DWORD);
                        if bPNegative != 0 {
                            (*lpFlags).stAttribs.dwAttribVal &= !(FILE_ATTRIBUTE_READONLY as DWORD);
                        } else {
                            (*lpFlags).stAttribs.dwAttribVal |= (FILE_ATTRIBUTE_READONLY as DWORD);
                        }
                    } else if cCurUChar == _T('H') {
                        (*lpFlags).stAttribs.dwAttribMask |= (FILE_ATTRIBUTE_HIDDEN as DWORD);
                        if bPNegative != 0 {
                            (*lpFlags).stAttribs.dwAttribVal &= !(FILE_ATTRIBUTE_HIDDEN as DWORD);
                        } else {
                            (*lpFlags).stAttribs.dwAttribVal |= (FILE_ATTRIBUTE_HIDDEN as DWORD);
                        }
                    } else if cCurUChar == _T('A') {
                        (*lpFlags).stAttribs.dwAttribMask |= (FILE_ATTRIBUTE_ARCHIVE as DWORD);
                        if bPNegative != 0 {
                            (*lpFlags).stAttribs.dwAttribVal &= !(FILE_ATTRIBUTE_ARCHIVE as DWORD);
                        } else {
                            (*lpFlags).stAttribs.dwAttribVal |= (FILE_ATTRIBUTE_ARCHIVE as DWORD);
                        }
                    } else if cCurUChar == _T('S') {
                        (*lpFlags).stAttribs.dwAttribMask |= (FILE_ATTRIBUTE_SYSTEM as DWORD);
                        if bPNegative != 0 {
                            (*lpFlags).stAttribs.dwAttribVal &= !(FILE_ATTRIBUTE_SYSTEM as DWORD);
                        } else {
                            (*lpFlags).stAttribs.dwAttribVal |= (FILE_ATTRIBUTE_SYSTEM as DWORD);
                        }
                    } else {
                        error_parameter_format(_totupper(*Line) as TCHAR);
                        return FALSE;
                    }
                }
                _ if cCurSwitch == _T('T') => {
                    if cCurUChar == _T('C') {
                        (*lpFlags).stTimeField.eTimeField = TF_CREATIONDATE;
                    } else if cCurUChar == _T('A') {
                        (*lpFlags).stTimeField.eTimeField = TF_LASTACCESSEDDATE;
                    } else if cCurUChar == _T('W') {
                        (*lpFlags).stTimeField.eTimeField = TF_MODIFIEDDATE;
                    } else {
                        error_parameter_format(_totupper(*Line) as TCHAR);
                        return FALSE;
                    }
                }
                _ if cCurSwitch == _T('O') => {
                    bOrderByNoPar = FALSE;

                    if cCurChar == _T('-') {
                        bPNegative = TRUE;
                    } else if cCurUChar == _T('N') {
                        if ((*lpFlags).stOrderBy.sCriteriaCount as i32) < 3 {
                            (*lpFlags).stOrderBy.sCriteriaCount += 1;
                        }
                        (*lpFlags).stOrderBy.bCriteriaRev[((*lpFlags).stOrderBy.sCriteriaCount as i32 - 1) as usize] = bPNegative;
                        (*lpFlags).stOrderBy.eCriteria[((*lpFlags).stOrderBy.sCriteriaCount as i32 - 1) as usize] = ORDER_NAME;
                    } else if cCurUChar == _T('S') {
                        if ((*lpFlags).stOrderBy.sCriteriaCount as i32) < 3 {
                            (*lpFlags).stOrderBy.sCriteriaCount += 1;
                        }
                        (*lpFlags).stOrderBy.bCriteriaRev[((*lpFlags).stOrderBy.sCriteriaCount as i32 - 1) as usize] = bPNegative;
                        (*lpFlags).stOrderBy.eCriteria[((*lpFlags).stOrderBy.sCriteriaCount as i32 - 1) as usize] = ORDER_SIZE;
                    } else if cCurUChar == _T('G') {
                        if ((*lpFlags).stOrderBy.sCriteriaCount as i32) < 3 {
                            (*lpFlags).stOrderBy.sCriteriaCount += 1;
                        }
                        (*lpFlags).stOrderBy.bCriteriaRev[((*lpFlags).stOrderBy.sCriteriaCount as i32 - 1) as usize] = bPNegative;
                        (*lpFlags).stOrderBy.eCriteria[((*lpFlags).stOrderBy.sCriteriaCount as i32 - 1) as usize] = ORDER_DIRECTORY;
                    } else if cCurUChar == _T('E') {
                        if ((*lpFlags).stOrderBy.sCriteriaCount as i32) < 3 {
                            (*lpFlags).stOrderBy.sCriteriaCount += 1;
                        }
                        (*lpFlags).stOrderBy.bCriteriaRev[((*lpFlags).stOrderBy.sCriteriaCount as i32 - 1) as usize] = bPNegative;
                        (*lpFlags).stOrderBy.eCriteria[((*lpFlags).stOrderBy.sCriteriaCount as i32 - 1) as usize] = ORDER_EXTENSION;
                    } else if cCurUChar == _T('D') {
                        if ((*lpFlags).stOrderBy.sCriteriaCount as i32) < 3 {
                            (*lpFlags).stOrderBy.sCriteriaCount += 1;
                        }
                        (*lpFlags).stOrderBy.bCriteriaRev[((*lpFlags).stOrderBy.sCriteriaCount as i32 - 1) as usize] = bPNegative;
                        (*lpFlags).stOrderBy.eCriteria[((*lpFlags).stOrderBy.sCriteriaCount as i32 - 1) as usize] = ORDER_TIME;
                    } else {
                        error_parameter_format(_totupper(*Line) as TCHAR);
                        return FALSE;
                    }
                }
                _ => {}
            }
            /* We check if we calculated the negative value and release the flag */
            if cCurChar != _T('-') && bPNegative != 0 {
                bPNegative = FALSE;
            }
        }

        Line = Line.add(1);
    }

    /* /O with no switch parameters acts like /O:GN */
    if bOrderByNoPar != 0 {
        (*lpFlags).stOrderBy.sCriteriaCount = 2;
        (*lpFlags).stOrderBy.eCriteria[0 as usize] = ORDER_DIRECTORY;
        (*lpFlags).stOrderBy.bCriteriaRev[0 as usize] = FALSE;
        (*lpFlags).stOrderBy.eCriteria[1 as usize] = ORDER_NAME;
        (*lpFlags).stOrderBy.bCriteriaRev[1 as usize] = FALSE;
    }

    return TRUE;
}

unsafe extern "C" fn DirPrintf(lpFlags: LPDIRSWITCHFLAGS, szFormat: LPTSTR, mut args: ...) -> BOOL {
    let mut Done: BOOL = TRUE;
    let mut arg_ptr: va_list;
    va_start(arg_ptr, szFormat);
    if (*lpFlags).bPause != 0 {
        Done = ConPrintfVPaging(core::ptr::addr_of_mut!(StdOutPager), FALSE, szFormat, arg_ptr);
    } else {
        ConPrintfV(StdOut, szFormat, arg_ptr);
    }
    va_end(arg_ptr);
    return Done;
}

unsafe extern "C" fn PrintDirectoryHeader(szPath: LPCTSTR, lpFlags: LPDIRSWITCHFLAGS) -> BOOL {
    let mut szMsg: [TCHAR; RC_STRING_MAX_SIZE] = [0; RC_STRING_MAX_SIZE];
    let mut szFullDir: LPCTSTR;
    let mut szRootName: [TCHAR; MAX_PATH] = [0; MAX_PATH];
    let mut szVolName: [TCHAR; 80] = [0; 80];
    let mut dwSerialNr: DWORD = 0;

    if (*lpFlags).bBareFormat != 0 {
        return TRUE;
    }

    szFullDir = szPath;

    /* Get the media ID of the drive */
    if GetVolumePathName(szFullDir, szRootName.as_mut_ptr(), ARRAYSIZE(szRootName)) == 0
        || GetVolumeInformation(
            szRootName.as_mut_ptr() as LPCTSTR,
            szVolName.as_mut_ptr(),
            ARRAYSIZE(szVolName),
            core::ptr::addr_of_mut!(dwSerialNr),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            0,
        ) == 0
    {
        return TRUE;
    }

    /* Print drive info */
    if szVolName[0] != _T('\0') {
        LoadString(
            CMD_ModuleHandle,
            STRING_DIR_HELP2,
            szMsg.as_mut_ptr(),
            ARRAYSIZE(szMsg),
        );
        DirPrintf(
            lpFlags,
            szMsg.as_mut_ptr(),
            _totupper(szRootName[0]),
            szVolName.as_mut_ptr(),
        );
    } else {
        LoadString(
            CMD_ModuleHandle,
            STRING_DIR_HELP3,
            szMsg.as_mut_ptr(),
            ARRAYSIZE(szMsg),
        );
        DirPrintf(
            lpFlags,
            szMsg.as_mut_ptr(),
            _totupper(szRootName[0]),
        );
    }

    /* Print the volume serial number if the return was successful */
    LoadString(
        CMD_ModuleHandle,
        STRING_DIR_HELP4,
        szMsg.as_mut_ptr(),
        ARRAYSIZE(szMsg),
    );
    DirPrintf(lpFlags, szMsg.as_mut_ptr(), HIWORD(dwSerialNr), LOWORD(dwSerialNr));

    return TRUE;
}

unsafe extern "C" fn DirPrintFileDateTime(
    lpDate: *mut TCHAR,
    lpTime: *mut TCHAR,
    lpFile: LPWIN32_FIND_DATA,
    lpFlags: LPDIRSWITCHFLAGS,
) {
    let mut ft: FILETIME = core::mem::zeroed();
    let mut dt: SYSTEMTIME = core::mem::zeroed();

    /* Select the right time field */
    match (*lpFlags).stTimeField.eTimeField {
        TF_CREATIONDATE => {
            if FileTimeToLocalFileTime(core::ptr::addr_of_mut!((*lpFile).ftCreationTime), core::ptr::addr_of_mut!(ft)) == 0 {
                return;
            }
            FileTimeToSystemTime(core::ptr::addr_of_mut!(ft), core::ptr::addr_of_mut!(dt));
        }
        TF_LASTACCESSEDDATE => {
            if FileTimeToLocalFileTime(core::ptr::addr_of_mut!((*lpFile).ftLastAccessTime), core::ptr::addr_of_mut!(ft)) == 0 {
                return;
            }
            FileTimeToSystemTime(core::ptr::addr_of_mut!(ft), core::ptr::addr_of_mut!(dt));
        }
        TF_MODIFIEDDATE => {
            if FileTimeToLocalFileTime(core::ptr::addr_of_mut!((*lpFile).ftLastWriteTime), core::ptr::addr_of_mut!(ft)) == 0 {
                return;
            }
            FileTimeToSystemTime(core::ptr::addr_of_mut!(ft), core::ptr::addr_of_mut!(dt));
        }
        _ => {}
    }

    FormatDate(lpDate, core::ptr::addr_of_mut!(dt), (*lpFlags).b4Digit);
    FormatTime(lpTime, core::ptr::addr_of_mut!(dt));
}
#[no_mangle]
pub unsafe extern "C" fn FormatDate(lpDate: *mut TCHAR, dt: LPSYSTEMTIME, b4Digit: BOOL) -> INT {
    /* Format date */
    let wYear: WORD = if b4Digit != 0 {
        (*dt).wYear
    } else {
        (*dt).wYear % (100 as WORD)
    };
    match nDateFormat {
        0 | _ => {
            /* mmddyy */
            return _stprintf(
                lpDate,
                _T("%02d%c%02d%c%0*d"),
                (*dt).wMonth as INT,
                cDateSeparator as INT,
                (*dt).wDay as INT,
                cDateSeparator as INT,
                (if b4Digit != 0 { 4 } else { 2 }) as INT,
                wYear as INT,
            );
        }
        1 => {
            /* ddmmyy */
            return _stprintf(
                lpDate,
                _T("%02d%c%02d%c%0*d"),
                (*dt).wDay as INT,
                cDateSeparator as INT,
                (*dt).wMonth as INT,
                cDateSeparator as INT,
                (if b4Digit != 0 { 4 } else { 2 }) as INT,
                wYear as INT,
            );
        }
        2 => {
            /* yymmdd */
            return _stprintf(
                lpDate,
                _T("%0*d%c%02d%c%02d"),
                (if b4Digit != 0 { 4 } else { 2 }) as INT,
                wYear as INT,
                cDateSeparator as INT,
                (*dt).wMonth as INT,
                cDateSeparator as INT,
                (*dt).wDay as INT,
            );
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn FormatTime(lpTime: *mut TCHAR, dt: LPSYSTEMTIME) -> INT {
    /* Format Time */
    match nTimeFormat {
        1 => { /* 24 hour format */
            let fmt: [u16; 11] = ['%' as u16, '0' as u16, '2' as u16, 'd' as u16, '%' as u16, 'c' as u16, '%' as u16, '0' as u16, '2' as u16, 'u' as u16, 0];
            return _stprintf(
                lpTime,
                fmt.as_ptr(),
                (*dt).wHour as INT,
                cTimeSeparator as INT,
                (*dt).wMinute as INT,
            );
        }
        0 | _ => { /* 12 hour format */
            let fmt: [u16; 15] = ['%' as u16, '0' as u16, '2' as u16, 'd' as u16, '%' as u16, 'c' as u16, '%' as u16, '0' as u16, '2' as u16, 'u' as u16, ' ' as u16, '%' as u16, 'c' as u16, 'M' as u16, 0];
            return _stprintf(
                lpTime,
                fmt.as_ptr(),
                (if ((*dt).wHour as INT) == 0 {
                    12
                } else if ((*dt).wHour as INT) <= 12 {
                    (*dt).wHour as INT
                } else {
                    (*dt).wHour as INT - 12
                }),
                cTimeSeparator as INT,
                (*dt).wMinute as INT,
                (if ((*dt).wHour as INT) <= 11 {
                    'A' as u16 as INT
                } else {
                    'P' as u16 as INT
                }),
            );
        }
    }
}

unsafe extern "C" fn GetUserDiskFreeSpace(mut lpRoot: LPCTSTR, mut lpFreeSpace: PULARGE_INTEGER) {
    let mut pGetFreeDiskSpaceEx: PGETFREEDISKSPACEEX = None;
    let mut hInstance: HINSTANCE = core::ptr::null_mut();
    let mut dwSecPerCl: DWORD = 0;
    let mut dwBytPerSec: DWORD = 0;
    let mut dwFreeCl: DWORD = 0;
    let mut dwTotCl: DWORD = 0;
    let mut TotalNumberOfBytes: ULARGE_INTEGER = core::mem::zeroed();
    let mut TotalNumberOfFreeBytes: ULARGE_INTEGER = core::mem::zeroed();

    (*lpFreeSpace).QuadPart = 0;

    let mut KERNEL32: [u16; 9] = ['K' as u16, 'E' as u16, 'R' as u16, 'N' as u16, 'E' as u16, 'L' as u16, '3' as u16, '2' as u16, 0];
    hInstance = GetModuleHandle(KERNEL32.as_ptr()) as HINSTANCE;
    if !hInstance.is_null() {
        pGetFreeDiskSpaceEx = core::mem::transmute::<FARPROC, PGETFREEDISKSPACEEX>(
            GetProcAddress(hInstance, b"GetDiskFreeSpaceExW\0".as_ptr() as LPCSTR),
        );
        if pGetFreeDiskSpaceEx.is_some() {
            if pGetFreeDiskSpaceEx.expect("non-null function pointer")(
                lpRoot,
                lpFreeSpace,
                &mut TotalNumberOfBytes,
                &mut TotalNumberOfFreeBytes,
            ) != FALSE
            {
                return;
            }
        }
    }

    GetDiskFreeSpace(
        lpRoot,
        &mut dwSecPerCl,
        &mut dwBytPerSec,
        &mut dwFreeCl,
        &mut dwTotCl,
    );

    (*lpFreeSpace).QuadPart =
        dwSecPerCl.wrapping_mul(dwBytPerSec).wrapping_mul(dwFreeCl) as ULONGLONG;
}

unsafe extern "C" fn PrintSummary(
    szPath: LPCTSTR,
    ulFiles: ULONG,
    ulDirs: ULONG,
    u64Bytes: ULONGLONG,
    lpFlags: LPDIRSWITCHFLAGS,
    TotalSummary: BOOL,
) -> INT {
    let mut szMsg: [TCHAR; RC_STRING_MAX_SIZE] = [0; RC_STRING_MAX_SIZE];
    let mut szBuffer: [TCHAR; 64] = [0; 64];
    let mut uliFree: ULARGE_INTEGER = core::mem::zeroed();

    /* Here we check if we didn't find anything */
    if ulFiles.wrapping_add(ulDirs) == 0 {
        if (*lpFlags).bRecursive == 0 || (TotalSummary != 0 && (*lpFlags).bRecursive != 0) {
            error_file_not_found();
        }
        return 1;
    }

    /* In bare format we don't print results */
    if (*lpFlags).bBareFormat != 0 {
        return 0;
    }

    /* Print recursive specific results */

    /* Take this code offline to fix /S does not print double info */
    if TotalSummary != 0 && (*lpFlags).bRecursive != 0 {
        ConvertULargeInteger(
            u64Bytes,
            core::ptr::addr_of_mut!(szBuffer) as LPTSTR,
            ARRAYSIZE(szBuffer) as UINT,
            (*lpFlags).bTSeparator,
        );
        LoadString(
            CMD_ModuleHandle,
            STRING_DIR_HELP5,
            core::ptr::addr_of_mut!(szMsg) as LPTSTR,
            ARRAYSIZE(szMsg) as INT,
        );
        DirPrintf(
            lpFlags,
            core::ptr::addr_of_mut!(szMsg) as LPTSTR,
            ulFiles,
            core::ptr::addr_of_mut!(szBuffer) as LPTSTR,
        );
    } else {
        /* Print File Summary */
        /* Condition to print summary is:
        If we are not in bare format and if we have results! */
        ConvertULargeInteger(
            u64Bytes,
            core::ptr::addr_of_mut!(szBuffer) as LPTSTR,
            ARRAYSIZE(szBuffer) as UINT,
            (*lpFlags).bTSeparator,
        );
        LoadString(
            CMD_ModuleHandle,
            STRING_DIR_HELP8,
            core::ptr::addr_of_mut!(szMsg) as LPTSTR,
            ARRAYSIZE(szMsg) as INT,
        );
        DirPrintf(
            lpFlags,
            core::ptr::addr_of_mut!(szMsg) as LPTSTR,
            ulFiles,
            core::ptr::addr_of_mut!(szBuffer) as LPTSTR,
        );
    }

    /* Print total directories and free space */
    if (*lpFlags).bRecursive == 0 || TotalSummary != 0 {
        GetUserDiskFreeSpace(szPath, core::ptr::addr_of_mut!(uliFree));
        ConvertULargeInteger(
            uliFree.QuadPart,
            core::ptr::addr_of_mut!(szBuffer) as LPTSTR,
            ARRAYSIZE(szBuffer) as UINT,
            (*lpFlags).bTSeparator,
        );
        LoadString(
            CMD_ModuleHandle,
            STRING_DIR_HELP6,
            core::ptr::addr_of_mut!(szMsg) as LPTSTR,
            ARRAYSIZE(szMsg) as INT,
        );
        DirPrintf(
            lpFlags,
            core::ptr::addr_of_mut!(szMsg) as LPTSTR,
            ulDirs,
            core::ptr::addr_of_mut!(szBuffer) as LPTSTR,
        );
    }

    return 0;
}
#[no_mangle]
pub unsafe extern "C" fn getExt(file: *const TCHAR) -> *mut TCHAR {
    static mut NoExt: [TCHAR; 1] = [0 as TCHAR];
    let NoExtPtr: *mut TCHAR = core::ptr::addr_of_mut!(NoExt) as *mut TCHAR;
    let mut lastdot: *mut TCHAR = _tcsrchr(file, _T('.')) as *mut TCHAR;
    return if !lastdot.is_null() {
        lastdot.offset(1 as isize)
    } else {
        NoExtPtr
    };
}

unsafe extern "C" fn getName(file: *const TCHAR, dest: *mut TCHAR) -> LPTSTR {
    let mut iLen: INT_PTR = 0;
    let mut end: LPTSTR = core::ptr::null_mut();

    /* Check for dot-directories "." and ".." */
    if IsDotDirectory(file) != 0 {
        _tcscpy(dest, file);
        return dest;
    }

    end = _tcsrchr(file, _T('.'));
    if end.is_null() {
        iLen = _tcslen(file) as INT_PTR;
    } else {
        iLen = end.offset_from(file) as core::ffi::c_longlong as INT_PTR;
    }

    _tcsncpy(dest, file, iLen as usize);
    *dest.offset(iLen as isize) = _T('\0');

    return dest;
}

unsafe extern "C" fn DirPrintNewList(
    ptrFiles: *mut PDIRFINDINFO,
    dwCount: DWORD,
    szCurPath: LPCTSTR,
    lpFlags: LPDIRSWITCHFLAGS,
) {
    let mut i: DWORD;
    let mut szSize: [TCHAR; 30] = [0; 30];
    let mut szShortName: [TCHAR; 15] = [0; 15];
    let mut szDate: [TCHAR; 20] = [0; 20];
    let mut szTime: [TCHAR; 20] = [0; 20];
    let mut iSizeFormat: INT;
    let mut u64FileSize: ULARGE_INTEGER = core::mem::zeroed();
    let mut ptrCurStream: PDIRFINDSTREAMNODE;

    i = 0;
    while i < dwCount && CheckCtrlBreak(BREAK_INPUT) == 0 {
        /* Calculate size */
        if (**ptrFiles.offset(i as isize)).stFindInfo.dwFileAttributes & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
            /* Junction */
            iSizeFormat = -(14) as INT;
            _tcscpy(szSize.as_mut_ptr(), _T("<JUNCTION>"));
        } else if (**ptrFiles.offset(i as isize)).stFindInfo.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY != 0 {
            /* Directory */
            iSizeFormat = -(14) as INT;
            _tcscpy(szSize.as_mut_ptr(), _T("<DIR>"));
        } else {
            /* File */
            iSizeFormat = 14 as INT;
            u64FileSize.__bindgen_anon_1.HighPart = (**ptrFiles.offset(i as isize)).stFindInfo.nFileSizeHigh;
            u64FileSize.__bindgen_anon_1.LowPart = (**ptrFiles.offset(i as isize)).stFindInfo.nFileSizeLow;
            ConvertULargeInteger(
                u64FileSize.QuadPart,
                szSize.as_mut_ptr(),
                20,
                (*lpFlags).bTSeparator,
            );
        }

        /* Calculate short name */
        szShortName[0] = _T('\0');
        if (*lpFlags).bShortName != 0 {
            _stprintf(
                szShortName.as_mut_ptr(),
                _T(" %-12s"),
                core::ptr::addr_of_mut!((**ptrFiles.offset(i as isize)).stFindInfo.cAlternateFileName),
            );
        }

        /* Format date and time */
        DirPrintFileDateTime(
            szDate.as_mut_ptr(),
            szTime.as_mut_ptr(),
            core::ptr::addr_of_mut!((**ptrFiles.offset(i as isize)).stFindInfo),
            lpFlags,
        );

        /* Print the line */
        DirPrintf(
            lpFlags,
            _T("%10s  %-6s    %*s%s %s\n"),
            szDate.as_mut_ptr(),
            szTime.as_mut_ptr(),
            iSizeFormat,
            szSize.as_mut_ptr(),
            szShortName.as_mut_ptr(),
            core::ptr::addr_of_mut!((**ptrFiles.offset(i as isize)).stFindInfo.cFileName),
        );

        /* Now, loop on the streams */
        ptrCurStream = (**ptrFiles.offset(i as isize)).ptrHead;
        while !ptrCurStream.is_null() {
            ConvertULargeInteger(
                (*ptrCurStream).stStreamInfo.StreamSize.QuadPart as ULONGLONG,
                szSize.as_mut_ptr(),
                20,
                (*lpFlags).bTSeparator,
            );

            /* Print the line */
            DirPrintf(
                lpFlags,
                _T("%10s  %-6s    %*s%s %s%s\n"),
                RUSTCONV_WSTR_96A296D224F2.as_ptr(),
                RUSTCONV_WSTR_96A296D224F2.as_ptr(),
                16,
                szSize.as_mut_ptr(),
                RUSTCONV_WSTR_96A296D224F2.as_ptr(),
                core::ptr::addr_of_mut!((**ptrFiles.offset(i as isize)).stFindInfo.cFileName),
                core::ptr::addr_of_mut!((*ptrCurStream).stStreamInfo.cStreamName),
            );
            ptrCurStream = (*ptrCurStream).ptrNext;
        }
        i += 1;
    }
}

unsafe extern "C" fn DirPrintWideList(
    ptrFiles: *mut PDIRFINDINFO,
    dwCount: DWORD,
    szCurPath: LPCTSTR,
    lpFlags: LPDIRSWITCHFLAGS,
) {
    let mut iScreenWidth: SHORT = 0;
    let mut iColumns: USHORT = 0;
    let mut iLines: USHORT = 0;
    let mut iLongestName: UINT_PTR = 0;
    let mut szTempFname: [TCHAR; MAX_PATH] = [0; MAX_PATH];
    let mut i: DWORD = 0;
    let mut j: DWORD = 0;
    let mut temp: DWORD = 0;
    let bCJK: BOOL = IsCJKCodePage(OutputCodePage);
    let mut cxWidth: SIZE_T = 0;

    /* Calculate longest name */
    iLongestName = 1 as UINT_PTR;
    i = 0 as DWORD;
    while i < dwCount {
        cxWidth = ConGetTextWidth(
            core::ptr::addr_of_mut!((**ptrFiles.add(i as usize)).stFindInfo.cFileName) as LPCTSTR,
        );

        /* Directories need 2 additional characters for brackets */
        if ((**ptrFiles.add(i as usize)).stFindInfo.dwFileAttributes
            & FILE_ATTRIBUTE_DIRECTORY as DWORD)
            != 0
        {
            cxWidth = cxWidth.wrapping_add(2 as SIZE_T);
        }

        iLongestName = if iLongestName > cxWidth {
            iLongestName
        } else {
            cxWidth
        };
        i = i.wrapping_add(1);
    }

    /* Count the highest number of columns */
    GetScreenSize(&raw mut iScreenWidth, core::ptr::null_mut::<SHORT>());
    iColumns = (iScreenWidth as UINT_PTR).wrapping_div(iLongestName) as USHORT;

    /* Check if there is enough space for spaces between names */
    if iLongestName
        .wrapping_mul(iColumns as UINT_PTR)
        .wrapping_add(iColumns as UINT_PTR)
        >= (iScreenWidth as UINT) as UINT_PTR
    {
        iColumns = iColumns.wrapping_sub(1);
    }

    /* A last check at iColumns to avoid division by zero */
    if iColumns == 0 {
        iColumns = 1 as USHORT;
    }

    /* Calculate the lines that will be printed */
    iLines = (dwCount
        .wrapping_add(iColumns as DWORD)
        .wrapping_sub(1 as DWORD)
        .wrapping_div(iColumns as DWORD)) as USHORT;

    i = 0 as DWORD;
    while i < iLines as DWORD && CheckCtrlBreak(BREAK_INPUT) == 0 {
        j = 0 as DWORD;
        while j < iColumns as DWORD {
            if (*lpFlags).bWideListColSort != 0 {
                /* Print Column sorted */
                temp = j.wrapping_mul(iLines as DWORD).wrapping_add(i);
            } else {
                /* Print Line sorted */
                temp = i.wrapping_mul(iColumns as DWORD).wrapping_add(j);
            }

            if temp >= dwCount {
                break;
            }

            if ((**ptrFiles.add(temp as usize)).stFindInfo.dwFileAttributes
                & FILE_ATTRIBUTE_DIRECTORY as DWORD)
                != 0
            {
                let mut fmt_dir: [u16; 5] = ['[' as u16, '%' as u16, 's' as u16, ']' as u16, 0];
                _stprintf(
                    szTempFname.as_mut_ptr(),
                    fmt_dir.as_mut_ptr() as LPCTSTR,
                    core::ptr::addr_of_mut!((**ptrFiles.add(temp as usize)).stFindInfo.cFileName),
                );
            } else {
                let mut fmt_file: [u16; 3] = ['%' as u16, 's' as u16, 0];
                _stprintf(
                    szTempFname.as_mut_ptr(),
                    fmt_file.as_mut_ptr() as LPCTSTR,
                    core::ptr::addr_of_mut!((**ptrFiles.add(temp as usize)).stFindInfo.cFileName),
                );
            }

            if bCJK != 0 {
                cxWidth = ConGetTextWidth(szTempFname.as_mut_ptr() as LPCTSTR);
                /* Print string and add space padding */
                let mut fmt_cjk: [u16; 6] = [
                    '%' as u16,
                    's' as u16,
                    '%' as u16,
                    '*' as u16,
                    's' as u16,
                    0,
                ];
                let mut empty: [u16; 1] = [0];
                DirPrintf(
                    lpFlags,
                    fmt_cjk.as_mut_ptr() as LPTSTR,
                    szTempFname.as_mut_ptr(),
                    (iLongestName as SIZE_T)
                        .wrapping_add(1 as SIZE_T)
                        .wrapping_sub(cxWidth),
                    empty.as_mut_ptr(),
                );
            } else {
                let mut fmt_pad: [u16; 5] = ['%' as u16, '-' as u16, '*' as u16, 's' as u16, 0];
                DirPrintf(
                    lpFlags,
                    fmt_pad.as_mut_ptr() as LPTSTR,
                    iLongestName.wrapping_add(1 as UINT_PTR),
                    szTempFname.as_mut_ptr(),
                );
            }
            j = j.wrapping_add(1);
        }

        /* Add a new line after the last item in the column */
        let mut fmt_nl: [u16; 2] = ['\n' as u16, 0];
        DirPrintf(lpFlags, fmt_nl.as_mut_ptr() as LPTSTR);
        i = i.wrapping_add(1);
    }
}

unsafe extern "C" fn DirPrintOldList(
    ptrFiles: *mut PDIRFINDINFO,
    dwCount: DWORD,
    szCurPath: LPCTSTR,
    lpFlags: LPDIRSWITCHFLAGS,
) {
    let mut i: DWORD;
    let mut szName: [TCHAR; 10] = [0; 10];
    let mut szExt: [TCHAR; 5] = [0; 5];
    let mut szDate: [TCHAR; 30] = [0; 30];
    let mut szTime: [TCHAR; 30] = [0; 30];
    let mut szSize: [TCHAR; 30] = [0; 30];
    let mut iSizeFormat: i32;
    let mut u64FileSize: ULARGE_INTEGER = core::mem::zeroed();

    i = 0;
    while i < dwCount && CheckCtrlBreak(BREAK_INPUT) == 0 {
        /* Broke 8.3 format */
        if *(core::ptr::addr_of!((*(*ptrFiles.offset(i as isize))).stFindInfo.cAlternateFileName) as *const TCHAR) != 0
        {
            /* If the file is long named then we read the alter name */
            getName(
                core::ptr::addr_of!((*(*ptrFiles.offset(i as isize))).stFindInfo.cAlternateFileName) as LPCTSTR,
                core::ptr::addr_of_mut!(szName) as LPTSTR,
            );
            _tcscpy(
                core::ptr::addr_of_mut!(szExt) as LPTSTR,
                getExt(
                    core::ptr::addr_of!((*(*ptrFiles.offset(i as isize))).stFindInfo.cAlternateFileName) as LPCTSTR,
                ),
            );
        } else {
            /* If the file is not long name we read its original name */
            getName(
                core::ptr::addr_of!((*(*ptrFiles.offset(i as isize))).stFindInfo.cFileName) as LPCTSTR,
                core::ptr::addr_of_mut!(szName) as LPTSTR,
            );
            _tcscpy(
                core::ptr::addr_of_mut!(szExt) as LPTSTR,
                getExt(
                    core::ptr::addr_of!((*(*ptrFiles.offset(i as isize))).stFindInfo.cFileName) as LPCTSTR,
                ),
            );
        }

        /* Calculate size */
        if ((*(*ptrFiles.offset(i as isize))).stFindInfo.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) != 0
        {
            /* Directory, no size it's a directory */
            iSizeFormat = -17;
            _tcscpy(
                core::ptr::addr_of_mut!(szSize) as LPTSTR,
                _T("<DIR>"),
            );
        } else {
            /* File */
            iSizeFormat = 17;
            u64FileSize.__bindgen_anon_1.HighPart = (*(*ptrFiles.offset(i as isize))).stFindInfo.nFileSizeHigh;
            u64FileSize.__bindgen_anon_1.LowPart = (*(*ptrFiles.offset(i as isize))).stFindInfo.nFileSizeLow;
            ConvertULargeInteger(
                u64FileSize.QuadPart,
                core::ptr::addr_of_mut!(szSize) as LPTSTR,
                20,
                (*lpFlags).bTSeparator,
            );
        }

        /* Format date and time */
        DirPrintFileDateTime(
            core::ptr::addr_of_mut!(szDate) as LPTSTR,
            core::ptr::addr_of_mut!(szTime) as LPTSTR,
            core::ptr::addr_of_mut!((*(*ptrFiles.offset(i as isize))).stFindInfo) as LPWIN32_FIND_DATA,
            lpFlags,
        );

        /* Print the line */
        DirPrintf(
            lpFlags,
            _T("%-8s %-3s  %*s %s  %s\n"),
            core::ptr::addr_of_mut!(szName) as LPTSTR,       /* The file's 8.3 name */
            core::ptr::addr_of_mut!(szExt) as LPTSTR,        /* The file's 8.3 extension */
            iSizeFormat,                                     /* print format for size column */
            core::ptr::addr_of_mut!(szSize) as LPTSTR,       /* The size of file or "<DIR>" for dirs */
            core::ptr::addr_of_mut!(szDate) as LPTSTR,       /* The date of file/dir */
            core::ptr::addr_of_mut!(szTime) as LPTSTR,       /* The time of file/dir */
        );

        i = i.wrapping_add(1);
    }
}

unsafe extern "C" fn DirPrintBareList(
    ptrFiles: *mut PDIRFINDINFO,
    dwCount: DWORD,
    szCurPath: LPCTSTR,
    lpFlags: LPDIRSWITCHFLAGS,
) {
    let mut i: DWORD;

    i = 0;
    while i < dwCount && CheckCtrlBreak(BREAK_INPUT) == 0 {
        'body: {
            if IsDotDirectory(
                core::ptr::addr_of_mut!((**ptrFiles.add(i as usize)).stFindInfo.cFileName) as LPCTSTR,
            ) != 0
            {
                /* At bare format we don't print the dot-directories "." and ".." */
                break 'body;
            }
            if (*lpFlags).bRecursive != 0 {
                /* At recursive mode we print full path of file */
                static FMT_RECURSE: [u16; 7] = [
                    '%' as u16, 's' as u16, '\\' as u16, '%' as u16, 's' as u16, '\n' as u16, 0,
                ];
                DirPrintf(
                    lpFlags,
                    FMT_RECURSE.as_ptr() as LPTSTR,
                    szCurPath,
                    core::ptr::addr_of_mut!((**ptrFiles.add(i as usize)).stFindInfo.cFileName) as LPTSTR,
                );
            } else {
                /* If we are not in recursive mode we print the file names */
                static FMT_PLAIN: [u16; 4] = ['%' as u16, 's' as u16, '\n' as u16, 0];
                DirPrintf(
                    lpFlags,
                    FMT_PLAIN.as_ptr() as LPTSTR,
                    core::ptr::addr_of_mut!((**ptrFiles.add(i as usize)).stFindInfo.cFileName) as LPTSTR,
                );
            }
        }
        i = i.wrapping_add(1);
    }
}

unsafe extern "C" fn DirPrintFiles(
    mut ptrFiles: *mut PDIRFINDINFO,
    mut dwCount: DWORD,
    mut szCurPath: LPCTSTR,
    mut lpFlags: LPDIRSWITCHFLAGS,
) {
    let mut szMsg: [TCHAR; RC_STRING_MAX_SIZE] = [0; RC_STRING_MAX_SIZE];
    let mut szTemp: [TCHAR; MAX_PATH] = [0; MAX_PATH];

    /* Print trailing backslash for root directory of drive */
    _tcscpy(szTemp.as_mut_ptr(), szCurPath);
    if _tcslen(szTemp.as_mut_ptr()) == 2
        && szTemp[1] == _T(':')
    {
        _tcscat(szTemp.as_mut_ptr(), _T("\\"));
    }

    /* Condition to print header:
       We are not printing in bare format
       and if we are in recursive mode... we must have results */
    if (*lpFlags).bBareFormat == 0
        && !((*lpFlags).bRecursive != 0 && dwCount <= 0)
    {
        LoadString(
            CMD_ModuleHandle,
            STRING_DIR_HELP7,
            szMsg.as_mut_ptr(),
            ARRAYSIZE(szMsg),
        );
        if DirPrintf(lpFlags, szMsg.as_mut_ptr(), szTemp.as_mut_ptr()) == 0 {
            return;
        }
    }

    if (*lpFlags).bBareFormat != 0 {
        /* Bare format */
        DirPrintBareList(ptrFiles, dwCount, szCurPath, lpFlags);
    } else if (*lpFlags).bShortName != 0 {
        /* New list style / Short names */
        DirPrintNewList(ptrFiles, dwCount, szCurPath, lpFlags);
    } else if (*lpFlags).bWideListColSort != 0 || (*lpFlags).bWideList != 0 {
        /* Wide list */
        DirPrintWideList(ptrFiles, dwCount, szCurPath, lpFlags);
    } else if (*lpFlags).bNewLongList != 0 {
        /* New list style*/
        DirPrintNewList(ptrFiles, dwCount, szCurPath, lpFlags);
    } else {
        /* If nothing is selected old list is the default */
        DirPrintOldList(ptrFiles, dwCount, szCurPath, lpFlags);
    }
}

unsafe extern "C" fn CompareFiles(
    lpFile1: PDIRFINDINFO,
    lpFile2: PDIRFINDINFO,
    lpFlags: LPDIRSWITCHFLAGS,
) -> BOOL {
    let mut u64File1: ULARGE_INTEGER = core::mem::zeroed();
    let mut u64File2: ULARGE_INTEGER = core::mem::zeroed();
    let mut i: i32 = 0;
    let mut iComp: i32 = 0; /* The comparison result */

    /* Calculate criteria by order given from user */
    while i < (*lpFlags).stOrderBy.sCriteriaCount as i32 {
        /* Calculate criteria */
        let eCrit: i32 = (*lpFlags).stOrderBy.eCriteria[i as usize] as i32;
        if eCrit == ORDER_SIZE {
            /* Order by size /o:s */
            /* concat the 32bit integers to a 64bit */
            u64File1.__bindgen_anon_1.LowPart = (*lpFile1).stFindInfo.nFileSizeLow;
            u64File1.__bindgen_anon_1.HighPart = (*lpFile1).stFindInfo.nFileSizeHigh;
            u64File2.__bindgen_anon_1.LowPart = (*lpFile2).stFindInfo.nFileSizeLow;
            u64File2.__bindgen_anon_1.HighPart = (*lpFile2).stFindInfo.nFileSizeHigh;

            /* In case that difference is too big for a long */
            if u64File1.QuadPart < u64File2.QuadPart {
                iComp = -1;
            } else if u64File1.QuadPart > u64File2.QuadPart {
                iComp = 1;
            } else {
                iComp = 0;
            }
        } else if eCrit == ORDER_DIRECTORY {
            /* Order by directory attribute /o:g */
            iComp = (((*lpFile2).stFindInfo.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY)
                .wrapping_sub((*lpFile1).stFindInfo.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY))
                as i32;
        } else if eCrit == ORDER_EXTENSION {
            /* Order by extension name /o:e */
            iComp = _tcsicmp(
                getExt(
                    core::ptr::addr_of_mut!((*lpFile1).stFindInfo.cFileName) as *const TCHAR,
                ),
                getExt(
                    core::ptr::addr_of_mut!((*lpFile2).stFindInfo.cFileName) as *const TCHAR,
                ),
            ) as i32;
        } else if eCrit == ORDER_NAME {
            /* Order by filename /o:n */
            iComp = _tcsicmp(
                core::ptr::addr_of_mut!((*lpFile1).stFindInfo.cFileName) as *const TCHAR,
                core::ptr::addr_of_mut!((*lpFile2).stFindInfo.cFileName) as *const TCHAR,
            ) as i32;
        } else if eCrit == ORDER_TIME {
            /* Order by file's time /o:t */
            /* We compare files based on the time field selected by /t */
            let eTime: i32 = (*lpFlags).stTimeField.eTimeField as i32;
            if eTime == TF_CREATIONDATE {
                /* concat the 32bit integers to a 64bit */
                u64File1.__bindgen_anon_1.LowPart = (*lpFile1).stFindInfo.ftCreationTime.dwLowDateTime;
                u64File1.__bindgen_anon_1.HighPart = (*lpFile1).stFindInfo.ftCreationTime.dwHighDateTime;
                u64File2.__bindgen_anon_1.LowPart = (*lpFile2).stFindInfo.ftCreationTime.dwLowDateTime;
                u64File2.__bindgen_anon_1.HighPart = (*lpFile2).stFindInfo.ftCreationTime.dwHighDateTime;
            } else if eTime == TF_LASTACCESSEDDATE {
                /* concat the 32bit integers to a 64bit */
                u64File1.__bindgen_anon_1.LowPart = (*lpFile1).stFindInfo.ftLastAccessTime.dwLowDateTime;
                u64File1.__bindgen_anon_1.HighPart = (*lpFile1).stFindInfo.ftLastAccessTime.dwHighDateTime;
                u64File2.__bindgen_anon_1.LowPart = (*lpFile2).stFindInfo.ftLastAccessTime.dwLowDateTime;
                u64File2.__bindgen_anon_1.HighPart = (*lpFile2).stFindInfo.ftLastAccessTime.dwHighDateTime;
            } else if eTime == TF_MODIFIEDDATE {
                /* concat the 32bit integers to a 64bit */
                u64File1.__bindgen_anon_1.LowPart = (*lpFile1).stFindInfo.ftLastWriteTime.dwLowDateTime;
                u64File1.__bindgen_anon_1.HighPart = (*lpFile1).stFindInfo.ftLastWriteTime.dwHighDateTime;
                u64File2.__bindgen_anon_1.LowPart = (*lpFile2).stFindInfo.ftLastWriteTime.dwLowDateTime;
                u64File2.__bindgen_anon_1.HighPart = (*lpFile2).stFindInfo.ftLastWriteTime.dwHighDateTime;
            }

            /* In case that difference is too big for a long */
            if u64File1.QuadPart < u64File2.QuadPart {
                iComp = -1;
            } else if u64File1.QuadPart > u64File2.QuadPart {
                iComp = 1;
            } else {
                iComp = 0;
            }
        } else {
            /* No criteria matched: switch has no default, nothing happens */
        }

        /* Reverse if desired */
        if (*lpFlags).stOrderBy.bCriteriaRev[i as usize] != 0 {
            iComp *= -1;
        }

        /* If that criteria was enough for distinguishing
           the files/dirs,there is no need to calculate the others*/
        if iComp != 0 {
            break;
        }

        i += 1;
    }

    /* Translate the value of iComp to boolean */
    return (iComp > 0) as BOOL;
}

unsafe extern "C" fn QsortFiles(
    ptrArray: *mut PDIRFINDINFO,
    mut i: ::core::ffi::c_int,
    mut j: ::core::ffi::c_int,
    lpFlags: LPDIRSWITCHFLAGS,
) {
    let mut lpTemp: PDIRFINDINFO = core::ptr::null_mut();
    let mut Way: BOOL = 0;
    if i < j {
        let mut First: ::core::ffi::c_int = i;
        let mut Last: ::core::ffi::c_int = j;
        let mut Temp: ::core::ffi::c_int = 0;
        Way = TRUE as BOOL;
        while i != j {
            if Way == CompareFiles(
                *ptrArray.offset(i as isize),
                *ptrArray.offset(j as isize),
                lpFlags,
            ) {
                /* Swap the pointers of the array */
                lpTemp = *ptrArray.offset(i as isize);
                *ptrArray.offset(i as isize) = *ptrArray.offset(j as isize);
                *ptrArray.offset(j as isize) = lpTemp;

                /* Swap the indexes for inverting sorting */
                Temp = i;
                i = j;
                j = Temp;

                Way = (Way == 0) as ::core::ffi::c_int as BOOL;
            }

            j += ((Way == 0) as ::core::ffi::c_int - Way) as ::core::ffi::c_int;
        }

        QsortFiles(ptrArray, First, i - 1, lpFlags);
        QsortFiles(ptrArray, i + 1, Last, lpFlags);
    }
}

unsafe extern "C" fn DirNodeCleanup(mut ptrStartNode: PDIRFINDLISTNODE, mut pdwCount: PDWORD) {
    let mut ptrNextNode: PDIRFINDLISTNODE;
    let mut ptrFreeNode: PDIRFINDSTREAMNODE;
    while !ptrStartNode.is_null() {
        ptrNextNode = (*ptrStartNode).ptrNext;
        while !(*ptrStartNode).stInfo.ptrHead.is_null() {
            ptrFreeNode = (*ptrStartNode).stInfo.ptrHead;
            (*ptrStartNode).stInfo.ptrHead = (*ptrFreeNode).ptrNext;
            cmd_free(ptrFreeNode as *mut core::ffi::c_void);
        }
        cmd_free(ptrStartNode as *mut core::ffi::c_void);
        ptrStartNode = ptrNextNode;
        *pdwCount = (*pdwCount).wrapping_sub(1);
    }
}

unsafe extern "C" fn DirList(
    mut szFullPath: LPTSTR,
    mut pszFilePart: LPTSTR,
    mut lpFlags: LPDIRSWITCHFLAGS,
) -> INT {
    let mut hSearch: HANDLE = core::ptr::null_mut();
    let mut hRecSearch: HANDLE = core::ptr::null_mut();
    let mut hStreams: HANDLE = core::ptr::null_mut();
    let mut wfdFileInfo: WIN32_FIND_DATA = core::mem::zeroed();
    let mut ptrFileArray: *mut PDIRFINDINFO = core::ptr::null_mut();
    let mut ptrStartNode: PDIRFINDLISTNODE = core::ptr::null_mut();
    let mut ptrNextNode: PDIRFINDLISTNODE = core::ptr::null_mut();
    let mut szSubPath: [TCHAR; MAX_PATH] = [0; MAX_PATH];
    let mut pszSubFilePart: LPTSTR = core::ptr::null_mut();
    let mut cPathSep: TCHAR = 0;
    let mut dwCount: DWORD = 0;
    let mut dwCountFiles: DWORD = 0;
    let mut dwCountDirs: DWORD = 0;
    let mut u64CountBytes: ULONGLONG = 0;
    let mut u64Temp: ULARGE_INTEGER = core::mem::zeroed();
    let mut wfsdStreamInfo: WIN32_FIND_STREAM_DATA = core::mem::zeroed();
    let mut ptrCurNode: *mut PDIRFINDSTREAMNODE = core::ptr::null_mut();
    static mut pFindFirstStreamW: Option<
        unsafe extern "C" fn(
            LPCWSTR,
            STREAM_INFO_LEVELS,
            LPVOID,
            DWORD,
        ) -> HANDLE,
    > = None;
    static mut pFindNextStreamW: Option<
        unsafe extern "C" fn(HANDLE, LPVOID) -> BOOL,
    > = None;

    /* Initialize variables */
    ptrStartNode = core::ptr::null_mut();
    ptrNextNode = core::ptr::null_mut();
    dwCount = 0;
    dwCountFiles = 0;
    dwCountDirs = 0;
    u64CountBytes = 0;

    /* Prepare the linked list, first node is allocated */
    ptrStartNode = cmd_alloc((core::mem::size_of::<DIRFINDLISTNODE>()) as size_t) as PDIRFINDLISTNODE;
    if ptrStartNode.is_null() {
        return 1;
    }
    (*ptrStartNode).stInfo.ptrHead = core::ptr::null_mut();
    ptrNextNode = ptrStartNode;

    /* Collect the results for the current directory */
    hSearch = FindFirstFile(szFullPath, core::ptr::addr_of_mut!(wfdFileInfo));
    if hSearch != INVALID_HANDLE_VALUE {
        loop {
            if (wfdFileInfo.dwFileAttributes & (*lpFlags).stAttribs.dwAttribMask)
                == ((*lpFlags).stAttribs.dwAttribMask & (*lpFlags).stAttribs.dwAttribVal)
            {
                (*ptrNextNode).ptrNext =
                    cmd_alloc((core::mem::size_of::<DIRFINDLISTNODE>()) as size_t) as PDIRFINDLISTNODE;
                if (*ptrNextNode).ptrNext.is_null() {
                    DirNodeCleanup(ptrStartNode, core::ptr::addr_of_mut!(dwCount));
                    FindClose(hSearch);
                    return 1;
                }

                /* Copy the info of search at linked list */
                memcpy(
                    core::ptr::addr_of_mut!((*(*ptrNextNode).ptrNext).stInfo.stFindInfo)
                        as *mut core::ffi::c_void,
                    core::ptr::addr_of_mut!(wfdFileInfo) as *const core::ffi::c_void,
                    (core::mem::size_of::<WIN32_FIND_DATA>()) as size_t,
                );

                /* If lower case is selected do it here */
                if (*lpFlags).bLowerCase != 0 {
                    _tcslwr(core::ptr::addr_of_mut!(
                        (*(*ptrNextNode).ptrNext).stInfo.stFindInfo.cAlternateFileName
                    ) as *mut TCHAR);
                    _tcslwr(core::ptr::addr_of_mut!(
                        (*(*ptrNextNode).ptrNext).stInfo.stFindInfo.cFileName
                    ) as *mut TCHAR);
                }

                /* No streams (yet?) */
                (*(*ptrNextNode).ptrNext).stInfo.ptrHead = core::ptr::null_mut();

                /* Alternate streams are only displayed with new long list */
                if (*lpFlags).bNewLongList != 0 && (*lpFlags).bDataStreams != 0 {
                    if pFindFirstStreamW.is_none() {
                        let k32: [u16; 9] = [
                            'k' as u16, 'e' as u16, 'r' as u16, 'n' as u16, 'e' as u16,
                            'l' as u16, '3' as u16, '2' as u16, 0,
                        ];
                        pFindFirstStreamW = core::mem::transmute::<
                            PVOID,
                            Option<
                                unsafe extern "C" fn(
                                    LPCWSTR,
                                    STREAM_INFO_LEVELS,
                                    LPVOID,
                                    DWORD,
                                ) -> HANDLE,
                            >,
                        >(
                            GetProcAddress(
                                GetModuleHandle(k32.as_ptr()) as HINSTANCE,
                                b"FindFirstStreamW\0".as_ptr() as LPCSTR,
                            ) as PVOID,
                        );
                        pFindNextStreamW = core::mem::transmute::<
                            PVOID,
                            Option<
                                unsafe extern "C" fn(
                                    HANDLE,
                                    LPVOID,
                                ) -> BOOL,
                            >,
                        >(
                            GetProcAddress(
                                GetModuleHandle(k32.as_ptr()) as HINSTANCE,
                                b"FindNextStreamW\0".as_ptr() as LPCSTR,
                            ) as PVOID,
                        );
                    }

                    /* Try to get stream information */
                    if pFindFirstStreamW.is_some() && pFindNextStreamW.is_some() {
                        hStreams = pFindFirstStreamW(
                            wfdFileInfo.cFileName.as_ptr(),
                            FindStreamInfoStandard,
                            core::ptr::addr_of_mut!(wfsdStreamInfo) as LPVOID,
                            0,
                        );
                    } else {
                        hStreams = INVALID_HANDLE_VALUE;
                    }

                    if hStreams != INVALID_HANDLE_VALUE {
                        /* We totally ignore first stream. It contains data about ::$DATA */
                        ptrCurNode = core::ptr::addr_of_mut!(
                            (*(*ptrNextNode).ptrNext).stInfo.ptrHead
                        );
                        while pFindNextStreamW(
                            hStreams,
                            core::ptr::addr_of_mut!(wfsdStreamInfo) as LPVOID,
                        ) != 0
                        {
                            *ptrCurNode =
                                cmd_alloc((core::mem::size_of::<DIRFINDSTREAMNODE>()) as size_t) as PDIRFINDSTREAMNODE;
                            if (*ptrCurNode).is_null() {
                                DirNodeCleanup(
                                    ptrStartNode,
                                    core::ptr::addr_of_mut!(dwCount),
                                );
                                FindClose(hStreams);
                                FindClose(hSearch);
                                return 1;
                            }

                            memcpy(
                                core::ptr::addr_of_mut!((**ptrCurNode).stStreamInfo)
                                    as *mut core::ffi::c_void,
                                core::ptr::addr_of_mut!(wfsdStreamInfo)
                                    as *const core::ffi::c_void,
                                (core::mem::size_of::<WIN32_FIND_STREAM_DATA>()) as size_t,
                            );

                            /* If lower case is selected do it here */
                            if (*lpFlags).bLowerCase != 0 {
                                _tcslwr(core::ptr::addr_of_mut!(
                                    (**ptrCurNode).stStreamInfo.cStreamName
                                ) as *mut TCHAR);
                            }

                            ptrCurNode = core::ptr::addr_of_mut!((**ptrCurNode).ptrNext);
                        }

                        FindClose(hStreams);
                        *ptrCurNode = core::ptr::null_mut();
                    }
                }

                /* Continue at next node at linked list */
                ptrNextNode = (*ptrNextNode).ptrNext;
                dwCount = dwCount.wrapping_add(1);

                /* Grab statistics */
                if (wfdFileInfo.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) != 0 {
                    /* Directory */
                    dwCountDirs = dwCountDirs.wrapping_add(1);
                } else {
                    /* File */
                    dwCountFiles = dwCountFiles.wrapping_add(1);
                    u64Temp.__bindgen_anon_1.HighPart = wfdFileInfo.nFileSizeHigh;
                    u64Temp.__bindgen_anon_1.LowPart = wfdFileInfo.nFileSizeLow;
                    u64CountBytes = u64CountBytes.wrapping_add(u64Temp.QuadPart);
                }
            }
            if !(FindNextFile(hSearch, core::ptr::addr_of_mut!(wfdFileInfo)) != 0) {
                break;
            }
        }
        FindClose(hSearch);
    }

    /* Terminate list */
    (*ptrNextNode).ptrNext = core::ptr::null_mut();

    /* Calculate and allocate space need for making an array of pointers */
    ptrFileArray = cmd_alloc(
        (core::mem::size_of::<PDIRFINDINFO>() as size_t).wrapping_mul(dwCount as size_t),
    ) as *mut PDIRFINDINFO;
    if ptrFileArray.is_null() {
        DirNodeCleanup(ptrStartNode, core::ptr::addr_of_mut!(dwCount));
        return 1;
    }

    /*
     * Create an array of pointers from the linked list
     * this will be used to sort and print data, rather than the list
     */
    ptrNextNode = ptrStartNode;
    dwCount = 0;
    while !(*ptrNextNode).ptrNext.is_null() {
        *ptrFileArray.add(dwCount as usize) =
            core::ptr::addr_of_mut!((*(*ptrNextNode).ptrNext).stInfo) as PDIRFINDINFO;
        ptrNextNode = (*ptrNextNode).ptrNext;
        dwCount = dwCount.wrapping_add(1);
    }

    /* Sort Data if requested */
    if ((*lpFlags).stOrderBy.sCriteriaCount as core::ffi::c_int) > 0 {
        QsortFiles(
            ptrFileArray,
            0,
            dwCount.wrapping_sub(1 as DWORD) as core::ffi::c_int,
            lpFlags,
        );
    }

    /* Print Data */
    cPathSep = *pszFilePart.offset(-(1 as isize));
    *pszFilePart.offset(-(1 as isize)) = '\0' as u16; /* Truncate to directory name only */
    DirPrintFiles(ptrFileArray, dwCount, szFullPath, lpFlags);

    if (*lpFlags).bRecursive != 0 {
        PrintSummary(
            szFullPath,
            dwCountFiles as ULONG,
            dwCountDirs as ULONG,
            u64CountBytes,
            lpFlags,
            FALSE,
        );
    }
    *pszFilePart.offset(-(1 as isize)) = cPathSep;

    /* Free array */
    cmd_free(ptrFileArray as *mut core::ffi::c_void);

    /* Free linked list */
    DirNodeCleanup(ptrStartNode, core::ptr::addr_of_mut!(dwCount));

    if CheckCtrlBreak(BREAK_INPUT) != 0 {
        return 1;
    }

    /* Add statistics to recursive statistics */
    recurse_dir_cnt = recurse_dir_cnt.wrapping_add(dwCountDirs as ULONG);
    recurse_file_cnt = recurse_file_cnt.wrapping_add(dwCountFiles as ULONG);
    recurse_bytes = recurse_bytes.wrapping_add(u64CountBytes);

    /*
     * Do the recursive job if requested.
     * The recursion is done on ALL (independent of their attributes)
     * directories of the current one.
     */
    if (*lpFlags).bRecursive != 0 {
        let star_dot_star: [u16; 4] = ['*' as u16, '.' as u16, '*' as u16, 0];
        let backslash: [u16; 2] = ['\\' as u16, 0];

        /* The new search is involving any *.* file */
        memcpy(
            addr_of_mut_szSubPath(&mut szSubPath),
            szFullPath as *const core::ffi::c_void,
            (pszFilePart.offset_from(szFullPath) as size_t)
                * (core::mem::size_of::<TCHAR>() as size_t),
        );
        _tcscpy(
            (core::ptr::addr_of_mut!(szSubPath) as *mut TCHAR)
                .offset(pszFilePart.offset_from(szFullPath)),
            star_dot_star.as_ptr(),
        );

        hRecSearch = FindFirstFile(
            core::ptr::addr_of_mut!(szSubPath) as *mut TCHAR,
            core::ptr::addr_of_mut!(wfdFileInfo),
        );
        if hRecSearch != INVALID_HANDLE_VALUE {
            loop {
                /* We search for directories other than "." and ".." */
                if IsDotDirectory(wfdFileInfo.cFileName.as_ptr()) == 0
                    && (wfdFileInfo.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) != 0
                {
                    /* Concat the path and the directory to do recursive */
                    memcpy(
                        addr_of_mut_szSubPath(&mut szSubPath),
                        szFullPath as *const core::ffi::c_void,
                        (pszFilePart.offset_from(szFullPath) as size_t)
                            * (core::mem::size_of::<TCHAR>() as size_t),
                    );
                    _tcscpy(
                        (core::ptr::addr_of_mut!(szSubPath) as *mut TCHAR)
                            .offset(pszFilePart.offset_from(szFullPath)),
                        wfdFileInfo.cFileName.as_ptr(),
                    );
                    _tcscat(
                        core::ptr::addr_of_mut!(szSubPath) as *mut TCHAR,
                        backslash.as_ptr(),
                    );
                    pszSubFilePart = (core::ptr::addr_of_mut!(szSubPath) as *mut TCHAR).offset(
                        _tcslen(core::ptr::addr_of_mut!(szSubPath) as *mut TCHAR)
                            as isize,
                    ) as LPTSTR;
                    _tcscat(pszSubFilePart, pszFilePart);

                    /* We do the same for the directory */
                    if DirList(
                        core::ptr::addr_of_mut!(szSubPath) as *mut TCHAR,
                        pszSubFilePart,
                        lpFlags,
                    ) != 0
                    {
                        FindClose(hRecSearch);
                        return 1;
                    }
                }
                if !(FindNextFile(hRecSearch, core::ptr::addr_of_mut!(wfdFileInfo)) != 0)
                {
                    break;
                }
            }
        }
        FindClose(hRecSearch);
    }

    return 0;
}

unsafe extern "C" fn ResolvePattern(
    mut pszPattern: LPTSTR,
    nBufferLength: DWORD,
    pszFullPath: LPTSTR,
    ppszPatternPart: *mut LPTSTR,
) {
    let mut pCurDir: LPTSTR = core::ptr::null_mut();
    let mut pNextDir: LPTSTR = core::ptr::null_mut();
    let mut ptr: LPTSTR = core::ptr::null_mut();
    let mut pszPatternPart: LPTSTR = core::ptr::null_mut();
    let mut szNewPattern: [TCHAR; MAX_PATH] = [0; MAX_PATH];

    /* Make a copy of the path pattern */
    _tcscpy(core::ptr::addr_of_mut!(szNewPattern) as LPTSTR, pszPattern);
    pszPattern = core::ptr::addr_of_mut!(szNewPattern) as LPTSTR;

    /* TRACE("Original pszPattern: %S\n", pszPattern); -- debug-only macro, omitted */

    /* Convert slashes into backslashes */
    pNextDir = pszPattern;
    loop {
        pNextDir = _tcschr(pNextDir, _T('/'));
        if pNextDir.is_null() {
            break;
        }
        let fresh0 = pNextDir;
        pNextDir = pNextDir.add(1);
        *fresh0 = _T('\\');
    }

    /*
     * Find any dots-only directory and collapse any "." or ".." special
     * directory that immediately follows it.
     */
    pNextDir = pszPattern;
    while !pNextDir.is_null() {
        pCurDir = pNextDir;

        /* Find the next path separator in the pattern */
        pNextDir = _tcschr(pNextDir, _T('\\'));
        if pNextDir.is_null() {
            break;
        }

        /* Ignore the special "." and ".." directories that are correctly handled */
        if (pNextDir.offset_from(pCurDir) == 0 as isize)
            || IsDotDirectoryN(pCurDir, pNextDir.offset_from(pCurDir) as usize) != 0
        {
            /* Found such a directory, ignore */
            pNextDir = pNextDir.add(1);
            continue;
        }

        /* Check whether this is a dots-only directory */
        ptr = pCurDir;
        while ptr < pNextDir {
            if *ptr != _T('.') {
                break;
            }
            ptr = ptr.add(1);
        }
        if ptr < pNextDir {
            /* Not a dots-only directory, ignore */
            pNextDir = pNextDir.add(1);
            continue;
        }

        /* Skip any consecutive backslashes */
        ptr = pNextDir;
        while *ptr == _T('\\') {
            ptr = ptr.add(1);
        }

        /* pCurDir is a dots-only directory, perform partial canonicalization */

        /* Remove any following "." directory */
        if *ptr.offset(0 as isize) == _T('.')
            && (*ptr.offset(1 as isize) == _T('\\')
                || *ptr.offset(1 as isize) == 0 as TCHAR)
        {
            memmove(
                pNextDir as *mut core::ffi::c_void,
                ptr.offset(1 as isize) as *const core::ffi::c_void,
                (_tcslen(ptr.offset(1 as isize))
                    + 1)
                    * core::mem::size_of::<TCHAR>(),
            );
        }
        /* Remove any following ".." directory */
        else if *ptr.offset(0 as isize) == _T('.')
            && *ptr.offset(1 as isize) == _T('.')
            && (*ptr.offset(2 as isize) == _T('\\')
                || *ptr.offset(2 as isize) == 0 as TCHAR)
        {
            /* Skip any consecutive backslashes before the next directory */
            ptr = ptr.offset(2 as isize);
            while *ptr == _T('\\') {
                ptr = ptr.add(1);
            }

            memmove(
                pCurDir as *mut core::ffi::c_void,
                ptr as *const core::ffi::c_void,
                (_tcslen(ptr) + 1)
                    * core::mem::size_of::<TCHAR>(),
            );
            pNextDir = pCurDir;
        } else {
            pNextDir = pNextDir.add(1);

            /* Collapse consecutive backslashes */
            if ptr > pNextDir {
                memmove(
                    pNextDir as *mut core::ffi::c_void,
                    ptr as *const core::ffi::c_void,
                    (_tcslen(ptr) + 1)
                        * core::mem::size_of::<TCHAR>(),
                );
            }
        }
    }

    /* An empty pattern means we enumerate all files in the current directory */
    if *pszPattern == 0 {
        _tcscpy(pszPattern, ['*' as TCHAR, 0 as TCHAR].as_ptr() as LPTSTR);
    }

    /* TRACE("New pszPattern: %S\n", pszPattern); -- debug-only macro, omitted */

    /* Create the full path */
    if GetFullPathName(
        pszPattern,
        nBufferLength,
        pszFullPath,
        core::ptr::addr_of_mut!(pszPatternPart),
    ) == 0 as DWORD
    {
        _tcscpy(pszFullPath, pszPattern);
        pszPatternPart = core::ptr::null_mut();
    }

    /* TRACE debug-only macros omitted */

    /*
     * Restore the correct file name component in case the pattern contained
     * trailing dots that have been skipped by GetFullPathName().
     */

    /* Find the last path separator in the original szPath */
    pNextDir = _tcsrchr(pszPattern, _T('\\'));
    if !pNextDir.is_null() {
        /* Skip past the separator and look at the path */
        pNextDir = pNextDir.add(1);
    } else {
        /* The pattern is the path we need to look at */
        pNextDir = pszPattern;
    }

    if _istalpha(*pNextDir.offset(0 as isize)) != 0
        && *pNextDir.offset(1 as isize) == _T(':')
        && *pNextDir.offset(2 as isize) != _T('\\')
    {
        /*
         * The syntax "<drive_letter>:" without any trailing backslash actually
         * means: "current directory on this drive".
         */
    } else if pszPatternPart.is_null() {
        /* ASSERT debug-only macro omitted */

        /* Anything NOT being "." or ".." (the special directories) must be fully restored */
        if *pNextDir != 0 && IsDotDirectory(pNextDir) == 0 {
            pszPatternPart = pszFullPath.add(_tcslen(pszFullPath) as usize);
            _tcscpy(pszPatternPart, pNextDir);
            pszPatternPart = core::ptr::null_mut();
        }
    } else if _tcscmp(pNextDir, pszPatternPart) != 0 as i32 {
        /*
         * For example, pszPatternPart == "." or ".." and we do not need to
         * do anything for these, or pszPatternPart == "dir\\noextfile." and
         * we need to restore all the trailing points.
         */

        /* Anything NOT being "." or ".." (the special directories) must be fully restored */
        if *pNextDir != 0 && IsDotDirectory(pNextDir) == 0 {
            /* Restore the correct file criterion */
            _tcscpy(pszPatternPart, pNextDir);
        }
    }

    /* TRACE debug-only macro omitted */

    /*
     * If no wildcard or file was specified and this is a directory,
     * display all files in it.
     */
    if pszPatternPart.is_null() || IsExistingDirectory(pszFullPath) != 0 {
        pszPatternPart = pszFullPath.add(_tcslen(pszFullPath) as usize);
        if *pszPatternPart.offset(-(1 as isize)) != _T('\\') {
            let fresh1 = pszPatternPart;
            pszPatternPart = pszPatternPart.add(1);
            *fresh1 = _T('\\');
        }
        _tcscpy(pszPatternPart, ['*' as TCHAR, 0 as TCHAR].as_ptr() as LPTSTR);
    }

    /* TRACE debug-only macro omitted */

    if !ppszPatternPart.is_null() {
        *ppszPatternPart = pszPatternPart;
    }
}
#[no_mangle]
pub unsafe extern "C" fn CommandDir(rest: LPTSTR) -> INT {
    let mut dircmd: [TCHAR; MAX_PATH] = [0; MAX_PATH];   /* A variable to store the DIRCMD environment variable */
    let mut prev_volume: [TCHAR; MAX_PATH] = [0; MAX_PATH];
    let mut szFullPath: [TCHAR; MAX_PATH] = [0; MAX_PATH];
    let mut params: *mut LPTSTR = core::ptr::null_mut();
    let mut pszFilePart: LPTSTR;
    let mut cPathSep: TCHAR;
    let mut entries: INT = 0;
    let mut loop_0: UINT = 0;
    let mut stFlags: DIRSWITCHFLAGS = core::mem::zeroed();
    let mut ret: INT = 1;
    let mut ChangedVolume: BOOL;

    /* Initialize Switch Flags < Default switches are set here! > */
    stFlags.b4Digit = TRUE;
    stFlags.bBareFormat = FALSE;
    stFlags.bDataStreams = FALSE;
    stFlags.bLowerCase = FALSE;
    stFlags.bNewLongList = TRUE;
    stFlags.bPause = FALSE;
    stFlags.bRecursive = FALSE;
    stFlags.bShortName = FALSE;
    stFlags.bTSeparator = TRUE;
    stFlags.bUser = FALSE;
    stFlags.bWideList = FALSE;
    stFlags.bWideListColSort = FALSE;
    stFlags.stTimeField.eTimeField = TF_MODIFIEDDATE;
    stFlags.stAttribs.dwAttribMask = FILE_ATTRIBUTE_HIDDEN | FILE_ATTRIBUTE_SYSTEM;
    stFlags.stAttribs.dwAttribVal = 0;
    stFlags.stOrderBy.sCriteriaCount = 0;

    nErrorLevel = 0;

    'cleanup: {
        /* Read the parameters from the DIRCMD environment variable */
        if GetEnvironmentVariable(_T("DIRCMD"), dircmd.as_mut_ptr(), ARRAYSIZE(dircmd)) != 0 {
            if DirReadParam(
                dircmd.as_mut_ptr(),
                core::ptr::addr_of_mut!(params),
                core::ptr::addr_of_mut!(entries),
                core::ptr::addr_of_mut!(stFlags),
            ) == 0
            {
                nErrorLevel = 1;
                break 'cleanup;
            }
        }

        /* Read the parameters */
        if DirReadParam(
            rest,
            core::ptr::addr_of_mut!(params),
            core::ptr::addr_of_mut!(entries),
            core::ptr::addr_of_mut!(stFlags),
        ) == 0
            || CheckCtrlBreak(BREAK_INPUT) != 0
        {
            nErrorLevel = 1;
            break 'cleanup;
        }

        /* Default to current directory */
        if entries == 0 {
            if add_entry(
                core::ptr::addr_of_mut!(entries),
                core::ptr::addr_of_mut!(params),
                _T("*"),
            ) == 0
            {
                nErrorLevel = 1;
                break 'cleanup;
            }
        }

        prev_volume[0] = _T('\0');

        /* Reset paging state */
        if stFlags.bPause != 0 {
            ConOutPrintfPaging(TRUE, _T(""));
        }

        loop_0 = 0;
        while loop_0 < (entries as UINT) {
            if CheckCtrlBreak(BREAK_INPUT) != 0 {
                nErrorLevel = 1;
                break 'cleanup;
            }

            recurse_dir_cnt = 0;
            recurse_file_cnt = 0;
            recurse_bytes = 0;

            /* <Debug :>
               Uncomment this to show the final state of switch flags*/
            {
                let mut i: i32;
                TRACE!("Attributes mask/value %x/%x\n", stFlags.stAttribs.dwAttribMask, stFlags.stAttribs.dwAttribVal);
                TRACE!("(B) Bare format : %i\n", stFlags.bBareFormat);
                TRACE!("(C) Thousand : %i\n", stFlags.bTSeparator);
                TRACE!("(W) Wide list : %i\n", stFlags.bWideList);
                TRACE!("(D) Wide list sort by column : %i\n", stFlags.bWideListColSort);
                TRACE!("(L) Lowercase : %i\n", stFlags.bLowerCase);
                TRACE!("(N) New : %i\n", stFlags.bNewLongList);
                TRACE!("(O) Order : %i\n", stFlags.stOrderBy.sCriteriaCount);
                i = 0;
                while i < stFlags.stOrderBy.sCriteriaCount as i32 {
                    TRACE!(
                        " Order Criteria [%i]: %i (Reversed: %i)\n",
                        i,
                        stFlags.stOrderBy.eCriteria[i as usize],
                        stFlags.stOrderBy.bCriteriaRev[i as usize]
                    );
                    i += 1;
                }
                TRACE!("(P) Pause : %i\n", stFlags.bPause);
                TRACE!("(Q) Owner : %i\n", stFlags.bUser);
                TRACE!("(R) Data stream : %i\n", stFlags.bDataStreams);
                TRACE!("(S) Recursive : %i\n", stFlags.bRecursive);
                TRACE!("(T) Time field : %i\n", stFlags.stTimeField.eTimeField);
                TRACE!("(X) Short names : %i\n", stFlags.bShortName);
                TRACE!("Parameter : %s\n", debugstr_aw(*params.add(loop_0 as usize)));
            }

            /* Print the drive header if the volume changed */
            ChangedVolume = TRUE;

            if stFlags.bBareFormat == 0
                && GetVolumePathName(
                    *params.add(loop_0 as usize),
                    szFullPath.as_mut_ptr(),
                    ARRAYSIZE(szFullPath),
                ) != 0
            {
                if _tcscmp(szFullPath.as_mut_ptr(), prev_volume.as_mut_ptr()) == 0 {
                    ChangedVolume = FALSE;
                } else {
                    _tcscpy(prev_volume.as_mut_ptr(), szFullPath.as_mut_ptr());
                }
            }

            /* Resolve the pattern */
            ResolvePattern(
                *params.add(loop_0 as usize),
                ARRAYSIZE(szFullPath),
                szFullPath.as_mut_ptr(),
                core::ptr::addr_of_mut!(pszFilePart),
            );

            /* Print the header */
            cPathSep = *pszFilePart.offset(-(1) as isize);
            *pszFilePart.offset(-(1) as isize) = _T('\0'); /* Truncate to directory name only */
            if ChangedVolume != 0
                && stFlags.bBareFormat == 0
                && PrintDirectoryHeader(
                    szFullPath.as_mut_ptr() as LPCTSTR,
                    core::ptr::addr_of_mut!(stFlags),
                ) == 0
            {
                nErrorLevel = 1;
                break 'cleanup;
            }
            *pszFilePart.offset(-(1) as isize) = cPathSep;

            /* Perform the actual directory listing */
            if DirList(
                szFullPath.as_mut_ptr(),
                pszFilePart,
                core::ptr::addr_of_mut!(stFlags),
            ) != 0
            {
                nErrorLevel = 1;
                break 'cleanup;
            }

            /* Print the footer */
            *pszFilePart.offset(-(1) as isize) = _T('\0'); /* Truncate to directory name only */
            PrintSummary(
                szFullPath.as_mut_ptr() as LPCTSTR,
                recurse_file_cnt,
                recurse_dir_cnt,
                recurse_bytes,
                core::ptr::addr_of_mut!(stFlags),
                TRUE,
            );

            loop_0 = loop_0.wrapping_add(1);
        }

        ret = 0;
    }

    /* cleanup: */
    freep(params);

    return ret;
}
