// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed {
    pub Access: DWORD,
    pub uID: UINT,
}

pub static RUSTCONV_WSTR_075DE2B906DB: [u16; 2] = [0x000Au16, 0x0000];
pub static RUSTCONV_WSTR_6A48B4F65983: [u16; 4] = [0x0025u16, 0x0073, 0x003A, 0x0000];
pub static RUSTCONV_WSTR_80BC5395DDEA: [u16; 7] = [0x0025u16, 0x0073, 0x005C, 0x0025, 0x0073, 0x003A, 0x0000];
pub static RUSTCONV_WSTR_96A296D224F2: [u16; 1] = [0x0000u16];
pub static RUSTCONV_WSTR_A4F63208272C: [u16; 4] = [0x0025u16, 0x002A, 0x0073, 0x0000];
pub static RUSTCONV_WSTR_B523E0ACF790: [u16; 4] = [0x0025u16, 0x0073, 0x0020, 0x0000];
pub static RUSTCONV_WSTR_D4CC14FBABA5: [u16; 5] = [0x000Au16, 0x0025, 0x0073, 0x0020, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut OptionC: BOOL = (FALSE) as i32;

#[no_mangle]
pub static mut OptionE: BOOL = (FALSE) as i32;

#[no_mangle]
pub static mut OptionT: BOOL = (FALSE) as i32;

#[no_mangle]
pub static mut OptionG: BOOL = (FALSE) as i32;

#[no_mangle]
pub static mut OptionR: BOOL = (FALSE) as i32;

#[no_mangle]
pub static mut OptionP: BOOL = (FALSE) as i32;

#[no_mangle]
pub static mut OptionD: BOOL = (FALSE) as i32;

#[no_mangle]
pub static mut PUser: LPCTSTR = ::core::ptr::null::<WCHAR>();

#[no_mangle]
pub static mut DUser: LPCTSTR = ::core::ptr::null::<WCHAR>();

#[no_mangle]
pub static mut GPerm: LPCTSTR = ::core::ptr::null::<WCHAR>();

#[no_mangle]
pub static mut PPerm: LPCTSTR = ::core::ptr::null::<WCHAR>();

#[no_mangle]
pub static mut RUser: LPCTSTR = ::core::ptr::null::<WCHAR>();

#[no_mangle]
pub static mut GUser: LPCTSTR = ::core::ptr::null::<WCHAR>();

static mut FileGenericMapping: _GENERIC_MAPPING = _GENERIC_MAPPING {
    GenericRead: FILE_GENERIC_READ as ACCESS_MASK,
    GenericWrite: FILE_GENERIC_WRITE as ACCESS_MASK,
    GenericExecute: FILE_GENERIC_EXECUTE as ACCESS_MASK,
    GenericAll: FILE_ALL_ACCESS as ACCESS_MASK,
};

pub const STANDARD_RIGHTS_READ: u32 = 0x0002_0000; // READ_CONTROL
pub const STANDARD_RIGHTS_WRITE: u32 = 0x0002_0000; // READ_CONTROL
pub const STANDARD_RIGHTS_EXECUTE: u32 = 0x0002_0000; // READ_CONTROL
pub const FILE_GENERIC_READ: u32 = 0x0012_0089;
pub const FILE_GENERIC_WRITE: u32 = 0x0012_0116;
pub const FILE_GENERIC_EXECUTE: u32 = 0x0012_00A0;

unsafe extern "C" fn PrintError(dwError: DWORD) {
    if dwError == ERROR_SUCCESS as DWORD {
        return;
    }
    ConMsgPuts(
        core::ptr::addr_of_mut!(csStdErr),
        FORMAT_MESSAGE_FROM_SYSTEM as DWORD,
        core::ptr::null_mut::<core::ffi::c_void>(),
        dwError,
        LANG_USER_DEFAULT as DWORD,
    );
}

unsafe extern "C" fn PrintFileDacl(FilePath: LPTSTR, FileName: LPTSTR) -> BOOL {
    let mut Length: SIZE_T;
    let mut SecurityDescriptor: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
    let mut SDSize: DWORD = 0 as DWORD;
    let mut FullFileName: [TCHAR; (MAX_PATH + 1) as usize] =
        [0; (MAX_PATH + 1) as usize];
    let mut Error: BOOL = FALSE as BOOL;
    let mut Ret: BOOL = FALSE as BOOL;

    Length = (_tcslen(FilePath) + _tcslen(FileName)) as _;
    if Length > MAX_PATH as SIZE_T {
        /* file name too long */
        SetLastError(ERROR_FILE_NOT_FOUND);
        return FALSE as BOOL;
    }

    _tcscpy(FullFileName.as_mut_ptr(), FilePath);
    _tcscat(FullFileName.as_mut_ptr(), FileName);

    /* find out how much memory we need */
    if GetFileSecurity(
        FullFileName.as_mut_ptr(),
        DACL_SECURITY_INFORMATION as DWORD,
        core::ptr::null_mut(),
        0 as DWORD,
        core::ptr::addr_of_mut!(SDSize),
    ) == 0
        && GetLastError() != ERROR_INSUFFICIENT_BUFFER as DWORD
    {
        return FALSE as BOOL;
    }

    SecurityDescriptor = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        SDSize as SIZE_T,
    ) as PSECURITY_DESCRIPTOR;
    if !SecurityDescriptor.is_null() {
        if GetFileSecurity(
            FullFileName.as_mut_ptr(),
            DACL_SECURITY_INFORMATION as DWORD,
            SecurityDescriptor,
            SDSize,
            core::ptr::addr_of_mut!(SDSize),
        ) != 0
        {
            let mut Dacl: PACL = core::ptr::null_mut();
            let mut DaclPresent: BOOL = 0;
            let mut DaclDefaulted: BOOL = 0;

            if GetSecurityDescriptorDacl(
                SecurityDescriptor,
                core::ptr::addr_of_mut!(DaclPresent),
                core::ptr::addr_of_mut!(Dacl),
                core::ptr::addr_of_mut!(DaclDefaulted),
            ) != 0
            {
                if DaclPresent != 0 {
                    let mut Ace: PACCESS_ALLOWED_ACE = core::ptr::null_mut();
                    let mut AceIndex: DWORD = 0 as DWORD;
                    static mut AccessRights: [C2RustUnnamed; 26] = [
                        C2RustUnnamed {
                            Access: FILE_WRITE_ATTRIBUTES as DWORD,
                            uID: IDS_FILE_WRITE_ATTRIBUTES as UINT,
                        },
                        C2RustUnnamed {
                            Access: FILE_READ_ATTRIBUTES as DWORD,
                            uID: IDS_FILE_READ_ATTRIBUTES as UINT,
                        },
                        C2RustUnnamed {
                            Access: FILE_DELETE_CHILD as DWORD,
                            uID: IDS_FILE_DELETE_CHILD as UINT,
                        },
                        C2RustUnnamed {
                            Access: FILE_EXECUTE as DWORD,
                            uID: IDS_FILE_EXECUTE as UINT,
                        },
                        C2RustUnnamed {
                            Access: FILE_WRITE_EA as DWORD,
                            uID: IDS_FILE_WRITE_EA as UINT,
                        },
                        C2RustUnnamed {
                            Access: FILE_READ_EA as DWORD,
                            uID: IDS_FILE_READ_EA as UINT,
                        },
                        C2RustUnnamed {
                            Access: FILE_APPEND_DATA as DWORD,
                            uID: IDS_FILE_APPEND_DATA as UINT,
                        },
                        C2RustUnnamed {
                            Access: FILE_WRITE_DATA as DWORD,
                            uID: IDS_FILE_WRITE_DATA as UINT,
                        },
                        C2RustUnnamed {
                            Access: FILE_READ_DATA as DWORD,
                            uID: IDS_FILE_READ_DATA as UINT,
                        },
                        C2RustUnnamed {
                            Access: FILE_GENERIC_EXECUTE as DWORD,
                            uID: IDS_FILE_GENERIC_EXECUTE as UINT,
                        },
                        C2RustUnnamed {
                            Access: FILE_GENERIC_WRITE as DWORD,
                            uID: IDS_FILE_GENERIC_WRITE as UINT,
                        },
                        C2RustUnnamed {
                            Access: FILE_GENERIC_READ as DWORD,
                            uID: IDS_FILE_GENERIC_READ as UINT,
                        },
                        C2RustUnnamed {
                            Access: GENERIC_ALL as DWORD,
                            uID: IDS_GENERIC_ALL as UINT,
                        },
                        C2RustUnnamed {
                            Access: GENERIC_EXECUTE as DWORD,
                            uID: IDS_GENERIC_EXECUTE as UINT,
                        },
                        C2RustUnnamed {
                            Access: GENERIC_WRITE as DWORD,
                            uID: IDS_GENERIC_WRITE as UINT,
                        },
                        C2RustUnnamed {
                            Access: GENERIC_READ as DWORD,
                            uID: IDS_GENERIC_READ as UINT,
                        },
                        C2RustUnnamed {
                            Access: MAXIMUM_ALLOWED as DWORD,
                            uID: IDS_MAXIMUM_ALLOWED as UINT,
                        },
                        C2RustUnnamed {
                            Access: ACCESS_SYSTEM_SECURITY as DWORD,
                            uID: IDS_ACCESS_SYSTEM_SECURITY as UINT,
                        },
                        C2RustUnnamed {
                            Access: SPECIFIC_RIGHTS_ALL as DWORD,
                            uID: IDS_SPECIFIC_RIGHTS_ALL as UINT,
                        },
                        C2RustUnnamed {
                            Access: STANDARD_RIGHTS_REQUIRED as DWORD,
                            uID: IDS_STANDARD_RIGHTS_REQUIRED as UINT,
                        },
                        C2RustUnnamed {
                            Access: SYNCHRONIZE as DWORD,
                            uID: IDS_SYNCHRONIZE as UINT,
                        },
                        C2RustUnnamed {
                            Access: WRITE_OWNER as DWORD,
                            uID: IDS_WRITE_OWNER as UINT,
                        },
                        C2RustUnnamed {
                            Access: WRITE_DAC as DWORD,
                            uID: IDS_WRITE_DAC as UINT,
                        },
                        C2RustUnnamed {
                            Access: READ_CONTROL as DWORD,
                            uID: IDS_READ_CONTROL as UINT,
                        },
                        C2RustUnnamed {
                            Access: DELETE as DWORD,
                            uID: IDS_DELETE as UINT,
                        },
                        C2RustUnnamed {
                            Access: STANDARD_RIGHTS_ALL as DWORD,
                            uID: IDS_STANDARD_RIGHTS_ALL as UINT,
                        },
                    ];
                    let mut x: DWORD = 0;

                    /* dump the ACL */
                    'ace: while GetAce(
                        Dacl,
                        AceIndex,
                        core::ptr::addr_of_mut!(Ace) as *mut PVOID,
                    ) != 0
                    {
                        let mut Use: SID_NAME_USE = 0 as SID_NAME_USE;
                        let mut NameSize: DWORD = 0 as DWORD;
                        let mut DomainSize: DWORD = 0 as DWORD;
                        let mut Name: LPTSTR = core::ptr::null_mut();
                        let mut Domain: LPTSTR = core::ptr::null_mut();
                        let mut SidString: LPTSTR = core::ptr::null_mut();
                        let mut IndentAccess: size_t = 0;
                        let mut AccessMask: DWORD = (*Ace).Mask;
                        let mut Sid: PSID = core::ptr::addr_of_mut!((*Ace).SidStart) as PSID;

                        /* attempt to translate the SID into a readable string */
                        'sid: {
                            'build: {
                                if LookupAccountSid(
                                    core::ptr::null_mut(),
                                    Sid,
                                    Name,
                                    core::ptr::addr_of_mut!(NameSize),
                                    Domain,
                                    core::ptr::addr_of_mut!(DomainSize),
                                    core::ptr::addr_of_mut!(Use),
                                ) == 0
                                {
                                    if GetLastError() == ERROR_NONE_MAPPED as DWORD
                                        || NameSize == 0 as DWORD
                                    {
                                        /* goto BuildSidString */
                                        break 'build;
                                    } else {
                                        if GetLastError() != ERROR_INSUFFICIENT_BUFFER as DWORD
                                        {
                                            Error = TRUE as BOOL;
                                            break 'ace;
                                        }

                                        Name = HeapAlloc(
                                            GetProcessHeap(),
                                            0 as DWORD,
                                            (NameSize.wrapping_add(DomainSize) as SIZE_T)
                                                .wrapping_mul(core::mem::size_of::<
                                                    TCHAR,
                                                >(
                                                ) as SIZE_T),
                                        ) as LPTSTR;
                                        if Name.is_null() {
                                            SetLastError(
                                                ERROR_NOT_ENOUGH_MEMORY as DWORD,
                                            );
                                            Error = TRUE as BOOL;
                                            break 'ace;
                                        }

                                        Domain = Name.add(NameSize as usize);
                                        *Name.add(0 as usize) =
                                            ('\0' as u16);
                                        if DomainSize != 0 as DWORD {
                                            *Domain.add(0 as usize) =
                                                ('\0' as u16);
                                        }
                                        if LookupAccountSid(
                                            core::ptr::null_mut(),
                                            Sid,
                                            Name,
                                            core::ptr::addr_of_mut!(NameSize),
                                            Domain,
                                            core::ptr::addr_of_mut!(DomainSize),
                                            core::ptr::addr_of_mut!(Use),
                                        ) == 0
                                        {
                                            HeapFree(
                                                GetProcessHeap(),
                                                0 as DWORD,
                                                Name as PVOID,
                                            );
                                            Name = core::ptr::null_mut();
                                            /* goto BuildSidString */
                                            break 'build;
                                        }
                                        /* lookup succeeded: skip BuildSidString */
                                        break 'sid;
                                    }
                                }
                            }
                            /* BuildSidString: */
                            if ConvertSidToStringSid(
                                Sid,
                                core::ptr::addr_of_mut!(SidString),
                            ) == 0
                            {
                                Error = TRUE as BOOL;
                                break 'ace;
                            }
                        }

                        /* print the file name or space */
                        ConPrintf(
                            StdOut,
                            RUSTCONV_WSTR_B523E0ACF790.as_ptr(),
                            FullFileName.as_mut_ptr(),
                        );

                        /* attempt to map the SID to a user name */
                        if AceIndex == 0 as DWORD {
                            let mut i: size_t = 0 as size_t;

                            /* overwrite the full file name with spaces so we
                               only print the file name once */
                            while FullFileName[i as usize] != ('\0' as u16) {
                                let fresh15 = i;
                                i = i.wrapping_add(1);
                                FullFileName[fresh15 as usize] = (' ' as u16);
                            }
                        }

                        /* print the domain and/or user if possible, or the SID string */
                        if !Name.is_null()
                            && *Domain.add(0 as usize) != ('\0' as u16)
                        {
                            ConPrintf(StdOut, RUSTCONV_WSTR_80BC5395DDEA.as_ptr(), Domain, Name);
                            IndentAccess = _tcslen(Domain)
                                .wrapping_add(_tcslen(Name))
                                as size_t;
                        } else {
                            let mut DisplayString: LPTSTR =
                                if !Name.is_null() { Name } else { SidString };

                            ConPrintf(StdOut, RUSTCONV_WSTR_6A48B4F65983.as_ptr(), DisplayString);
                            IndentAccess = _tcslen(DisplayString) as size_t;
                        }

                        /* print the ACE Flags */
                        if ((*Ace).Header.AceFlags as u32 & (CONTAINER_INHERIT_ACE as u32)) != 0
                        {
                            IndentAccess = IndentAccess.wrapping_add(
                                ConResPuts(StdOut, IDS_ABBR_CI) as size_t,
                            );
                        }
                        if ((*Ace).Header.AceFlags as u32 & (OBJECT_INHERIT_ACE as u32)) != 0
                        {
                            IndentAccess = IndentAccess.wrapping_add(
                                ConResPuts(StdOut, IDS_ABBR_OI) as size_t,
                            );
                        }
                        if ((*Ace).Header.AceFlags as u32 & (INHERIT_ONLY_ACE as u32)) != 0
                        {
                            IndentAccess = IndentAccess.wrapping_add(
                                ConResPuts(StdOut, IDS_ABBR_IO) as size_t,
                            );
                        }

                        IndentAccess = IndentAccess.wrapping_add(2 as size_t);

                        /* print the access rights */
                        MapGenericMask(
                            core::ptr::addr_of_mut!(AccessMask),
                            core::ptr::addr_of!(FileGenericMapping) as *mut _GENERIC_MAPPING,
                        );
                        if ((*Ace).Header.AceType as u32 & (ACCESS_DENIED_ACE_TYPE as u32)) != 0
                        {
                            if AccessMask == FILE_ALL_ACCESS as DWORD {
                                ConResPuts(StdOut, IDS_ABBR_NONE);
                            } else {
                                ConResPuts(StdOut, IDS_DENY);
                                /* goto PrintSpecialAccess */
                            }
                        } else {
                            if AccessMask == FILE_ALL_ACCESS as DWORD {
                                ConResPuts(StdOut, IDS_ABBR_FULL);
                            } else if ((*Ace).Mask
                                & ((GENERIC_READ as DWORD)
                                    | (GENERIC_EXECUTE as DWORD)))
                                == 0
                                && AccessMask
                                    == ((FILE_GENERIC_READ as DWORD)
                                        | (FILE_EXECUTE as DWORD))
                            {
                                ConResPuts(StdOut, IDS_ABBR_READ);
                            } else if AccessMask
                                == ((FILE_GENERIC_READ as DWORD)
                                    | (FILE_GENERIC_WRITE as DWORD)
                                    | (FILE_EXECUTE as DWORD)
                                    | (DELETE as DWORD))
                            {
                                ConResPuts(StdOut, IDS_ABBR_CHANGE);
                            } else if AccessMask == FILE_GENERIC_WRITE as DWORD {
                                ConResPuts(StdOut, IDS_ABBR_WRITE);
                            } else {
                                ConResPuts(StdOut, IDS_ALLOW);
                            }
                        }

                        /* PrintSpecialAccess label is reached by fall-through from the
                           deny-other branch and the final else branch above; the
                           other branches skip it via break 'access_done */
                        'access_done: {
                            if (*Ace).Header.AceType as u32 & (ACCESS_DENIED_ACE_TYPE as u32) != 0
                            {
                                if AccessMask == FILE_ALL_ACCESS as DWORD {
                                    break 'access_done;
                                }
                            } else if AccessMask == FILE_ALL_ACCESS as DWORD
                                || ((*Ace).Mask
                                    & ((GENERIC_READ as DWORD)
                                        | (GENERIC_EXECUTE as DWORD)))
                                    == 0
                                    && AccessMask
                                        == ((FILE_GENERIC_READ as DWORD)
                                            | (FILE_EXECUTE as DWORD))
                                || AccessMask
                                    == ((FILE_GENERIC_READ as DWORD)
                                        | (FILE_GENERIC_WRITE as DWORD)
                                        | (FILE_EXECUTE as DWORD)
                                        | (DELETE as DWORD))
                                || AccessMask == FILE_GENERIC_WRITE as DWORD
                            {
                                break 'access_done;
                            }

                            /* PrintSpecialAccess: */
                            ConResPuts(StdOut, IDS_SPECIAL_ACCESS);

                            /* print the special access rights */
                            x = ARRAYSIZE!(AccessRights) as DWORD;
                            loop {
                                let fresh16 = x;
                                x = x.wrapping_sub(1);
                                if !(fresh16 != 0 as DWORD) {
                                    break;
                                }
                                if ((*Ace).Mask & AccessRights[x as usize].Access)
                                    == AccessRights[x as usize].Access
                                {
                                    ConPrintf(
                                        StdOut,
                                        RUSTCONV_WSTR_D4CC14FBABA5.as_ptr(),
                                        FullFileName.as_mut_ptr(),
                                    );
                                    ConPrintf(
                                        StdOut,
                                        RUSTCONV_WSTR_A4F63208272C.as_ptr(),
                                        IndentAccess,
                                        RUSTCONV_WSTR_96A296D224F2.as_ptr(),
                                    );
                                    ConResPuts(StdOut, AccessRights[x as usize].uID);
                                }
                            }

                            ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());
                        }

                        ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());

                        /* free up all resources */
                        if !Name.is_null() {
                            HeapFree(
                                GetProcessHeap(),
                                0 as DWORD,
                                Name as PVOID,
                            );
                        }

                        if !SidString.is_null() {
                            LocalFree(SidString as HLOCAL);
                        }

                        AceIndex = AceIndex.wrapping_add(1);
                    }

                    if Error == 0 {
                        Ret = TRUE as BOOL;
                    }
                } else {
                    SetLastError(ERROR_NO_SECURITY_ON_OBJECT as DWORD);
                }
            }
        }

        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            SecurityDescriptor as PVOID,
        );
    } else {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY as DWORD);
    }

    return Ret;
}

unsafe extern "C" fn AddBackslash(FilePath: LPTSTR) {
    let BACKSLASH: [TCHAR; 2] = ['\\' as TCHAR, 0];
    let mut pch: LPTSTR = _tcsrchr(FilePath as *const TCHAR, ('\\' as TCHAR)) as LPTSTR;
    if pch.is_null() || *pch.add(1) != 0 {
        _tcscat(FilePath, BACKSLASH.as_ptr());
    }
}

unsafe extern "C" fn GetPathOfFile(FilePath: LPTSTR, pszFiles: LPCTSTR) -> BOOL {
    let mut FullPath: [TCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut pch: LPTSTR = core::ptr::null_mut();
    let mut attrs: DWORD = 0;

    // The C's _tcsncpy(.., MAX_PATH) leaves FilePath unterminated for a MAX_PATH-long
    // argument, and _tcsrchr then scans past it. Both copies here always terminate.
    StringCchCopyW(FilePath, MAX_PATH as usize, pszFiles);
    pch = _tcsrchr(FilePath, ('\\' as u16));
    if !pch.is_null() {
        *pch = 0;
        if GetFullPathName(
            FilePath,
            MAX_PATH,
            core::ptr::addr_of_mut!(FullPath) as LPTSTR,
            core::ptr::null_mut(),
        ) == 0
        {
            PrintError(GetLastError());
            return (FALSE) as BOOL;
        }
        StringCchCopyW(FilePath, MAX_PATH as usize, core::ptr::addr_of_mut!(FullPath) as LPTSTR);

        attrs = GetFileAttributes(FilePath);
        if attrs == 0xFFFFFFFF || (attrs & FILE_ATTRIBUTE_DIRECTORY) == 0 {
            PrintError(ERROR_DIRECTORY);
            return (FALSE) as BOOL;
        }
    } else {
        GetCurrentDirectory(MAX_PATH, FilePath);
    }

    AddBackslash(FilePath);
    return (TRUE) as BOOL;
}

unsafe extern "C" fn PrintDaclsOfFiles(pszFiles: LPCTSTR) -> BOOL {
    let mut FilePath: [TCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut FindData: WIN32_FIND_DATA = core::mem::zeroed();
    let mut hFind: HANDLE = core::ptr::null_mut();
    let mut LastError: DWORD = 0;

    if GetPathOfFile(core::ptr::addr_of_mut!(FilePath) as LPTSTR, pszFiles) == 0 {
        return (FALSE) as BOOL;
    }

    hFind = FindFirstFile(pszFiles, (core::ptr::addr_of_mut!(FindData)).cast());
    if hFind == INVALID_HANDLE_VALUE {
        return (FALSE) as BOOL;
    }
    'dowhile: loop {
        'body: {
            if (FindData.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) != 0 {
                break 'body;
            }

            if PrintFileDacl(
                core::ptr::addr_of_mut!(FilePath) as LPTSTR,
                core::ptr::addr_of_mut!(FindData.cFileName) as LPTSTR,
            ) == 0
            {
                LastError = GetLastError();
                if LastError == ERROR_ACCESS_DENIED {
                    PrintError(LastError);
                    if OptionC == 0 {
                        FindClose(hFind);
                        return (FALSE) as BOOL;
                    }
                } else {
                    break 'dowhile;
                }
            } else {
                ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());
            }
        }
        if FindNextFile(hFind, (core::ptr::addr_of_mut!(FindData)).cast()) == 0 {
            break 'dowhile;
        }
    }
    LastError = GetLastError();
    FindClose(hFind);

    if LastError != ERROR_NO_MORE_FILES {
        PrintError(LastError);
        return (FALSE) as BOOL;
    }

    return (TRUE) as BOOL;
}

unsafe extern "C" fn GrantUserAccessRights(
    mut FilePath: LPCTSTR,
    mut File: LPCTSTR,
    mut User: LPCTSTR,
    mut Perm: TCHAR,
) -> BOOL {
    /* TODO & FIXME */
    match Perm {
        p if p == ('R' as u16) => {
            // Read
        }
        p if p == ('W' as u16) => {
            // Write
        }
        p if p == ('C' as u16) => {
            // Change (write)
        }
        p if p == ('F' as u16) => {
            // Full control
        }
        _ => {}
    }
    return (FALSE) as BOOL;
}

unsafe extern "C" fn ReplaceUserAccessRights(
    FilePath: LPCTSTR,
    File: LPCTSTR,
    User: LPCTSTR,
    Perm: TCHAR,
) -> BOOL {
    if Perm == ('N' as u16) {
        // None
    } else if Perm == ('R' as u16) {
        // Read
    } else if Perm == ('W' as u16) {
        // Write
    } else if Perm == ('C' as u16) {
        // Change (write)
    } else if Perm == ('F' as u16) {
        // Full control
    } else {
        // default
    }
    return (FALSE) as BOOL;
}

unsafe extern "C" fn EditUserAccessRights(
    mut FilePath: LPCTSTR,
    mut File: LPCTSTR,
    mut User: LPCTSTR,
    mut Perm: TCHAR,
) -> BOOL {
    /* TODO & FIXME */
    if Perm == ('N' as u16) {
        // None
    } else if Perm == ('R' as u16) {
        // Read
    } else if Perm == ('W' as u16) {
        // Write
    } else if Perm == ('C' as u16) {
        // Change (write)
    } else if Perm == ('F' as u16) {
        // Full control
    } else {
        // default
    }
    return (FALSE) as BOOL;
}

unsafe extern "C" fn DenyUserAccess(FilePath: LPCTSTR, File: LPCTSTR, User: LPCTSTR) -> BOOL {
    (FALSE) as i32
}

unsafe extern "C" fn RevokeUserAccessRights(
    FilePath: LPCTSTR,
    File: LPCTSTR,
    User: LPCTSTR,
) -> BOOL {
    (FALSE) as i32
}

unsafe extern "C" fn ChangeFileACL(FilePath: LPCTSTR, File: LPCTSTR) -> BOOL {
    if OptionG != 0 {
        /* Grant specified user access rights. */
        GrantUserAccessRights(FilePath, File, GUser, *GPerm);
    }

    if OptionP != 0 {
        if OptionE == 0 {
            /* Replace specified user's access rights. */
            ReplaceUserAccessRights(FilePath, File, PUser, *PPerm);
        } else {
            /* Edit ACL instead of replacing it. */
            EditUserAccessRights(FilePath, File, PUser, *PPerm);
        }
    }

    if OptionD != 0 {
        /* Deny specified user access. */
        DenyUserAccess(FilePath, File, DUser);
    }

    if OptionR != 0 {
        /* Revoke specified user's access rights. */
        RevokeUserAccessRights(FilePath, File, RUser);
    }

    return (TRUE) as BOOL;
}

unsafe extern "C" fn ChangeACLsOfFiles(pszFiles: LPCTSTR) -> BOOL {
    let mut FilePath: [TCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut hFind: HANDLE = core::ptr::null_mut();
    let mut FindData: WIN32_FIND_DATA = core::mem::zeroed();
    let mut LastError: DWORD = 0;

    /*
     * get the file path
     */
    if !(GetPathOfFile(core::ptr::addr_of_mut!(FilePath) as LPTSTR, pszFiles) != 0) {
        return (FALSE) as BOOL;
    }

    /*
     * search for files in current directory
     */
    hFind = FindFirstFile(pszFiles, (core::ptr::addr_of_mut!(FindData)).cast());
    if hFind == INVALID_HANDLE_VALUE {
        return (FALSE) as BOOL;
    }
    'dowhile: loop {
        'body: {
            if (FindData.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) != 0 {
                break 'body;
            }

            if ChangeFileACL(
                core::ptr::addr_of!(FilePath) as LPCTSTR,
                core::ptr::addr_of!(FindData.cFileName) as LPCTSTR,
            ) == 0
            {
                LastError = GetLastError();
                if LastError == ERROR_ACCESS_DENIED {
                    PrintError(LastError);
                    if OptionC == 0 {
                        FindClose(hFind);
                        return (FALSE) as BOOL;
                    }
                } else {
                    break 'dowhile;
                }
            }
        }
        if !(FindNextFile(hFind, (core::ptr::addr_of_mut!(FindData)).cast()) != 0) {
            break 'dowhile;
        }
    }
    LastError = GetLastError();
    FindClose(hFind);

    if LastError != ERROR_NO_MORE_FILES {
        PrintError(LastError);
        return (FALSE) as BOOL;
    }

    return (TRUE) as BOOL;
}

unsafe extern "C" fn ChangeACLsOfFilesInCurDir(pszFiles: LPCTSTR) -> BOOL {
    let mut hFind: HANDLE = core::ptr::null_mut();
    let mut FindData: WIN32_FIND_DATA = core::mem::zeroed::<WIN32_FIND_DATA>();
    let mut szCurDir: [TCHAR; (MAX_PATH) as _] = [0 as TCHAR; (MAX_PATH) as _];
    let mut LastError: DWORD;

    /*
     * get the file path (current directory)
     */
    GetCurrentDirectory(MAX_PATH, core::ptr::addr_of_mut!(szCurDir) as *mut TCHAR as LPTSTR);
    AddBackslash(core::ptr::addr_of_mut!(szCurDir) as *mut TCHAR as LPTSTR);

    /*
     * search for files in current directory
     */
    hFind = FindFirstFile(pszFiles, (core::ptr::addr_of_mut!(FindData)).cast());
    if hFind == INVALID_HANDLE_VALUE {
        return (FALSE) as BOOL;
    }
    'dw: loop {
        'body: {
            if (FindData.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) != 0 {
                break 'body;
            }

            if ChangeFileACL(
                core::ptr::addr_of_mut!(szCurDir) as *mut TCHAR as LPCTSTR,
                core::ptr::addr_of_mut!(FindData.cFileName) as *mut TCHAR as LPCTSTR,
            ) == 0
            {
                LastError = GetLastError();
                if LastError == ERROR_ACCESS_DENIED {
                    PrintError(LastError);
                    if OptionC == 0 {
                        FindClose(hFind);
                        return (FALSE) as BOOL;
                    }
                } else {
                    break 'dw;
                }
            }
        }
        if !(FindNextFile(hFind, (core::ptr::addr_of_mut!(FindData)).cast()) != 0) {
            break;
        }
    }
    LastError = GetLastError();
    FindClose(hFind);

    if LastError != ERROR_NO_MORE_FILES {
        PrintError(LastError);
        return (FALSE) as BOOL;
    }

    /*
     * search for subdirectory in current directory
     */
    hFind = FindFirstFile(
        ['*' as u16, 0u16].as_ptr() as LPCTSTR,
        (core::ptr::addr_of_mut!(FindData)).cast(),
    );
    if hFind == INVALID_HANDLE_VALUE {
        return (FALSE) as BOOL;
    }
    'dw2: loop {
        'body2: {
            if _tcscmp(
                core::ptr::addr_of_mut!(FindData.cFileName) as *mut TCHAR as LPCTSTR,
                ['.' as u16, 0u16].as_ptr() as LPCTSTR,
            ) == 0
                || _tcscmp(
                    core::ptr::addr_of_mut!(FindData.cFileName) as *mut TCHAR as LPCTSTR,
                    ['.' as u16, '.' as u16, 0u16].as_ptr() as LPCTSTR,
                ) == 0
            {
                break 'body2;
            }

            if (FindData.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) != 0 {
                GetCurrentDirectory(
                    MAX_PATH,
                    core::ptr::addr_of_mut!(szCurDir) as *mut TCHAR as LPTSTR,
                );
                if SetCurrentDirectory(
                    core::ptr::addr_of_mut!(FindData.cFileName) as *mut TCHAR as LPCTSTR,
                ) != 0
                {
                    ChangeACLsOfFilesInCurDir(pszFiles);
                    SetCurrentDirectory(
                        core::ptr::addr_of_mut!(szCurDir) as *mut TCHAR as LPCTSTR,
                    );
                } else {
                    LastError = GetLastError();
                    if LastError == ERROR_ACCESS_DENIED {
                        PrintError(LastError);
                        if OptionC == 0 {
                            FindClose(hFind);
                            return (FALSE) as BOOL;
                        }
                    } else {
                        break 'dw2;
                    }
                }
            }
        }
        if !(FindNextFile(hFind, (core::ptr::addr_of_mut!(FindData)).cast()) != 0) {
            break;
        }
    }
    LastError = GetLastError();
    FindClose(hFind);

    if LastError != ERROR_NO_MORE_FILES {
        PrintError(LastError);
        return (FALSE) as BOOL;
    }
    return (TRUE) as BOOL;
}

#[export_name = "wmain"]
pub unsafe extern "C" fn _tmain(mut argc: INT, mut argv: *mut *const TCHAR) -> INT {
    let mut i: INT;
    let mut pch: LPTSTR;
    let mut InvalidParameter: BOOL = (FALSE) as i32;

    /* Initialize the Console Standard Streams */
    ConInitStdStreams();

    if argc <= 1 {
        ConResPuts(StdOut, IDS_HELP);
        return 0;
    }

    // FIXME: Convert to proper parsing, with support for /?

    /*
     * parse command line options
     */
    i = 2;
    'scan: while i < argc {
        if _tcsicmp(*argv.offset(i as isize), (['/' as u16, 'T' as u16, 0].as_ptr())) == 0 {
            OptionT = (TRUE) as i32;
        } else if _tcsicmp(*argv.offset(i as isize), (['/' as u16, 'E' as u16, 0].as_ptr())) == 0 {
            OptionE = (TRUE) as i32;
        } else if _tcsicmp(*argv.offset(i as isize), (['/' as u16, 'C' as u16, 0].as_ptr())) == 0 {
            OptionC = (TRUE) as i32;
        } else if _tcsicmp(*argv.offset(i as isize), (['/' as u16, 'G' as u16, 0].as_ptr())) == 0 {
            if i + 1 < argc {
                i += 1;
                pch = _tcschr(*argv.offset(i as isize), (':' as u16));
                if !pch.is_null() {
                    OptionG = (TRUE) as i32;
                    *pch = 0;
                    GUser = *argv.offset(i as isize);
                    GPerm = pch.offset(1 as isize);
                    i += 1;
                    continue 'scan;
                }
            }
            InvalidParameter = (TRUE) as i32;
            break 'scan;
        } else if _tcsicmp(*argv.offset(i as isize), (['/' as u16, 'R' as u16, 0].as_ptr())) == 0 {
            if i + 1 < argc {
                i += 1;
                RUser = *argv.offset(i as isize);
                OptionR = (TRUE) as i32;
                i += 1;
                continue 'scan;
            }
            InvalidParameter = (TRUE) as i32;
            break 'scan;
        } else if _tcsicmp(*argv.offset(i as isize), (['/' as u16, 'P' as u16, 0].as_ptr())) == 0 {
            if i + 1 < argc {
                i += 1;
                pch = _tcschr(*argv.offset(i as isize), (':' as u16));
                if !pch.is_null() {
                    OptionP = (TRUE) as i32;
                    *pch = 0;
                    PUser = *argv.offset(i as isize);
                    PPerm = pch.offset(1 as isize);
                    i += 1;
                    continue 'scan;
                }
            }
            InvalidParameter = (TRUE) as i32;
            break 'scan;
        } else if _tcsicmp(*argv.offset(i as isize), (['/' as u16, 'D' as u16, 0].as_ptr())) == 0 {
            if i + 1 < argc {
                OptionD = (TRUE) as i32;
                i += 1;
                DUser = *argv.offset(i as isize);
                i += 1;
                continue 'scan;
            }
            InvalidParameter = (TRUE) as i32;
            break 'scan;
        } else {
            InvalidParameter = (TRUE) as i32;
            break 'scan;
        }
        i += 1;
    }

    if InvalidParameter != 0 {
        PrintError(ERROR_INVALID_PARAMETER);
        ConResPuts(StdOut, IDS_HELP);
        return 1;
    }

    /* /R is only valid with /E */
    if OptionR != 0 && OptionE == 0 {
        OptionR = (FALSE) as i32;
    }

    PrintDaclsOfFiles(*argv.offset(1 as isize));

    if OptionT != 0 {
        ChangeACLsOfFilesInCurDir(*argv.offset(1 as isize));
    } else {
        ChangeACLsOfFiles(*argv.offset(1 as isize));
    }

    return 0;
}
