// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_0D857F239DCA: [WCHAR; 8] = [0x0061, 0x0075, 0x0074, 0x006F, 0x0072, 0x0075, 0x006E, 0x0000];
pub static CPP_COM_WSTR_2618E721CE41: [WCHAR; 29] = [0x0025, 0x0073, 0x005C, 0x0044, 0x0072, 0x0069, 0x0076, 0x0065, 0x0049, 0x0063, 0x006F, 0x006E, 0x0073, 0x005C, 0x0025, 0x0063, 0x005C, 0x0044, 0x0065, 0x0066, 0x0061, 0x0075, 0x006C, 0x0074, 0x0049, 0x0063, 0x006F, 0x006E, 0x0000];
pub static CPP_COM_WSTR_32491EF24F87: [WCHAR; 6] = [0x0044, 0x0072, 0x0069, 0x0076, 0x0065, 0x0000];
pub static CPP_COM_WSTR_6D957E170BC9: [WCHAR; 6] = [0x006C, 0x0061, 0x0062, 0x0065, 0x006C, 0x0000];
pub static CPP_COM_WSTR_6DBED233E95A: [WCHAR; 48] = [0x0041, 0x0070, 0x0070, 0x006C, 0x0069, 0x0063, 0x0061, 0x0074, 0x0069, 0x006F, 0x006E, 0x0073, 0x005C, 0x0045, 0x0078, 0x0070, 0x006C, 0x006F, 0x0072, 0x0065, 0x0072, 0x002E, 0x0065, 0x0078, 0x0065, 0x005C, 0x0044, 0x0072, 0x0069, 0x0076, 0x0065, 0x0073, 0x005C, 0x0025, 0x0063, 0x005C, 0x0044, 0x0065, 0x0066, 0x0061, 0x0075, 0x006C, 0x0074, 0x0049, 0x0063, 0x006F, 0x006E, 0x0000];
pub static CPP_COM_WSTR_7BE637194C2D: [WCHAR; 5] = [0x0069, 0x0063, 0x006F, 0x006E, 0x0000];
pub static CPP_COM_WSTR_7F8732C542BA: [WCHAR; 4] = [0x0041, 0x003A, 0x005C, 0x0000];
pub static CPP_COM_WSTR_BA2342D760A2: [WCHAR; 6] = [0x0025, 0x0073, 0x005C, 0x0025, 0x0073, 0x0000];
pub static CPP_COM_WSTR_DFA857FD38EE: [WCHAR; 16] = [0x0025, 0x0063, 0x003A, 0x005C, 0x0061, 0x0075, 0x0074, 0x006F, 0x0072, 0x0075, 0x006E, 0x002E, 0x0069, 0x006E, 0x0066, 0x0000];
pub static CPP_COM_WSTR_PHYSICALDRIVE: [WCHAR; 8] = // L"\\\\.\\%c:" (CDrivesFolder.cpp:416)
    [0x005C, 0x005C, 0x002E, 0x005C, 0x0025, 0x0063, 0x003A, 0x0000];
pub static CPP_COM_WSTR_EB4CE2024A43: [WCHAR; 7] = [0x0046, 0x006F, 0x006C, 0x0064, 0x0065, 0x0072, 0x0000];

// The island/bindgen substrate (via `prelude`) is the ONLY substrate source for this TU; the
// `types`/`win32`/`shellcom` globs the assembler also emits are dropped rather than joined,
// because carrying both makes every shared name ambiguous (E0659) at 70+ sites. See prelude.rs.
use core::ffi::c_void;
use crate::prelude::*;

// --- substrate the island does not carry ---
use crate::shell32::sdk_consts::SHCOLSTATE_TYPE_STR;
use crate::shell32::shlfolder::{AddClassKeyToArray, SH_GetApidlFromDataObject};
use crate::dialogs_item_prop::SHELL32_ShowFilesystemItemPropertiesDialogAsync;
use crate::drives_folder_enum::CDrivesFolderEnum;
// Defined by the drvdefext free-function slice; this TU reached it through a C-linkage
// forwarder that is leaving with the .cpp, so call the Rust definition directly.
use crate::drv_def_ext_tu::IsDriveFloppyW;
use crate::desktop_folder::DFM_CMD_PROPERTIES;
use crate::wine::classes::HCR_GetIconW;
// The island renders IDefaultExtractIconInit as an OPAQUE class with no method slots, so the
// `initIcon->SetNormalIcon(..)` calls below have nothing to dispatch through. `file_sys_enum`
// carries the real lpVtbl shape (same C ABI, distinct nominal type), which is where
// cfs_folder_tu.rs resolves the identical calls.
use crate::file_sys_enum::IDefaultExtractIconInit;

use crate::win32::{CStubWindow32_CreateStub_RustBridge, CStubWindow32_DestroyStub_RustBridge,
                   CSTUBWINDOW32_TYPE_FORMATDRIVE};

#[inline]
fn HResultFromWin32(err: DWORD) -> HRESULT { HRESULT_FROM_WIN32(err as i32) }


// Missing from `sdk_consts`, as in every other flipped folder module in this crate.
pub const SHCOLSTATE_ONBYDEFAULT: DWORD = 0x10; // shtypes.idl:125
pub const STRRET_WSTR: UINT = 0;

unsafe extern "C" {
    fn _ILCreateDrive(lpszNew: LPCWSTR) -> LPITEMIDLIST;
    fn _ILGetDrive(pidl: LPCITEMIDLIST, pOut: LPWSTR, uSize: UINT) -> BOOL;
    fn _ILFreeaPidl(apidl: *mut LPITEMIDLIST, cidl: UINT);
    fn HLM_GetIconW(index: UINT, destination: LPWSTR, capacity: UINT, index_out: *mut INT) -> BOOL;
    fn SHELL32_DefaultContextMenuCallBack(psf: *mut IShellFolder, pdtobj: *mut IDataObject, uMsg: UINT) -> HRESULT;
    // CRegFolder.cpp stays C++; the same bridge CControlPanelFolder.rs uses.
    fn CRegFolder_CreateInstance(pInit: PREGFOLDERINITDATA, pidlRoot: LPCITEMIDLIST, riid: REFIID, ppv: *mut *mut c_void) -> HRESULT;
}

// CDrivesFolder.cpp: `enum { CMDID_FORMAT, CMDID_EJECT, CMDID_DISCONNECT }`. Upstream 62519e131a7
// moved these from the old 1/2/3 workaround to 0/1/2 once CDefaultContextMenu learned
// DFM_MAPCOMMANDNAME, and idCmdFirst now advances by exactly one past the last id.
const CMDID_FORMAT: UINT = 0;
const CMDID_EJECT: UINT = 1;
const CMDID_DISCONNECT: UINT = 2;

// The verbs DFM_MAPCOMMANDNAME resolves. Divergence: upstream's array has no `{ NULL }`
// terminator, but SHELL_MapContextMenuVerbToCmdId walks until `Verb == NULL`, so an unknown
// verb reads past the end of it. Every other CMVERBMAP in shell32 is terminated; so is this one.
static mut g_VerbMap: [CMVERBMAP; 4] = [
    CMVERBMAP { Verb: b"format\0".as_ptr() as LPCSTR, CmdId: CMDID_FORMAT as WORD },
    CMVERBMAP { Verb: b"eject\0".as_ptr() as LPCSTR, CmdId: CMDID_EJECT as WORD },
    CMVERBMAP { Verb: b"disconnect\0".as_ptr() as LPCSTR, CmdId: CMDID_DISCONNECT as WORD },
    CMVERBMAP { Verb: core::ptr::null(), CmdId: 0 },
];

// winioctl.h CTL_CODE expansions; values cross-checked against ntkrnl-sys/constants.rs and
// cdrom-rust, which independently carry the same numbers.
const FSCTL_LOCK_VOLUME: DWORD = 0x00090018;
const FSCTL_UNLOCK_VOLUME: DWORD = 0x0009001C;
const FSCTL_DISMOUNT_VOLUME: DWORD = 0x00090020;
const IOCTL_STORAGE_MEDIA_REMOVAL: DWORD = 0x002D4804;
const IOCTL_STORAGE_EJECT_MEDIA: DWORD = 0x002D4808;

// The two file-scope literals of g_RegFolderInfo (CDrivesFolder.cpp:67-68). The assembler named
// them but emitted neither, so they are spelled out here rather than left dangling.
pub static CPP_COM_WSTR_20D04FE0: [WCHAR; 40] = [
    0x003A, 0x003A, 0x007B, 0x0032, 0x0030, 0x0044, 0x0030, 0x0034, 0x0046, 0x0045,
    0x0030, 0x002D, 0x0033, 0x0041, 0x0045, 0x0041, 0x002D, 0x0031, 0x0030, 0x0036,
    0x0039, 0x002D, 0x0041, 0x0032, 0x0044, 0x0038, 0x002D, 0x0030, 0x0038, 0x0030,
    0x0030, 0x0032, 0x0042, 0x0033, 0x0030, 0x0033, 0x0030, 0x0039, 0x007D, 0x0000,
];
pub static CPP_COM_WSTR_MyComputer: [WCHAR; 11] = [
    0x004D, 0x0079, 0x0043, 0x006F, 0x006D, 0x0070, 0x0075, 0x0074, 0x0065, 0x0072, 0x0000,
];

// shfldr.h:167 — a `static __inline`, so there is no out-of-line symbol to import.
#[inline]
unsafe fn SHELL32_GUIDToStringW(guid: *const CLSID, str_: LPWSTR) -> INT {
    StringFromGUID2(guid.cast(), str_, 40)
}

// Win32 pointer/integer narrowing macros, not functions.
#[inline]
fn PtrToUlong(p: *const c_void) -> ULONG {
    p as usize as ULONG
}
#[inline]
pub fn UlongToPtr(v: ULONG) -> *mut c_void {
    v as usize as *mut c_void
}

/// CDrivesFolder.cpp:96. C++ OVERLOADS `GetDriveNumber` on its parameter (the PIDL form above,
/// this path form here); Rust has no overloading, so this is a distinct name. Only the PIDL
/// form came across, leaving four call sites — all of them passing a `PCWSTR` — bound to a
/// function that would have reinterpreted the string as a PIDL.
unsafe fn GetDriveNumber_Path(DrivePath: PCWSTR) -> i8 {
    let number: WORD = ((*DrivePath.offset(0) | 32) as WORD).wrapping_sub(b'a' as WORD);
    if number < 26 { number as i8 } else { -1 }
}

// ---- file-scope preamble (translated) ----
pub static iDriveIconIds: [i32; 7] = [
    IDI_SHELL_NOT_CONNECTED_HDD as i32,
    IDI_SHELL_NOT_CONNECTED_HDD as i32,
    IDI_SHELL_REMOVEABLE as i32,
    IDI_SHELL_DRIVE as i32,
    IDI_SHELL_NETDRIVE as i32,
    IDI_SHELL_CDROM as i32,
    IDI_SHELL_RAMDISK as i32,
];

pub static iDriveTypeIds: [i32; 7] = [
    IDS_DRIVE_FIXED as i32,
    IDS_DRIVE_FIXED as i32,
    IDS_DRIVE_REMOVABLE as i32,
    IDS_DRIVE_FIXED as i32,
    IDS_DRIVE_NETWORK as i32,
    IDS_DRIVE_CDROM as i32,
    IDS_DRIVE_FIXED as i32,
];

// REQUIREDREGITEM and REGFOLDERINFO come from the island (shfldr.h:53-66). The translated
// slice had re-declared both, and BOTH shadows were layout-wrong, not merely nominally
// distinct:
//   REQUIREDREGITEM  clsid by VALUE (16 bytes) for `REFCLSID` (a pointer); `pszName: *const
//                    u16` for a NARROW `LPCSTR`; `dwOrder: u32` for a `BYTE`.
//   REGFOLDERINFO    `dwType: u32` + `cRequiredItems: usize` where the C has two adjacent
//                    BYTEs (PidlType, Count); clsid by value again.
// CRegFolder_CreateInstance reads this through the island's spelling, so the shadow would have
// fed it garbage — the My Computer folder's required-items table and its CLSID both. This is
// the "never hand-roll a shadow struct for a type bindgen already emits" rule, in its
// layout-corrupting form rather than its E0308 form.
pub static g_RequiredItems: [REQUIREDREGITEM; 1] = [
    REQUIREDREGITEM {
        clsid: core::ptr::addr_of!(CLSID_ControlPanel),
        pszCpl: core::ptr::null(),
        Order: REGITEMORDER_MYCOMPUTER_CONTROLS as BYTE,
    },
];

pub static g_RegFolderInfo: REGFOLDERINFO = REGFOLDERINFO {
    PidlType: PT_COMPUTER_REGITEM as PIDLTYPE,
    Count: 1, // _countof(g_RequiredItems)
    Items: g_RequiredItems.as_ptr(),
    clsid: core::ptr::addr_of!(CLSID_MyComputer),
    pszParsingPath: CPP_COM_WSTR_20D04FE0.as_ptr(),
    pszEnumKeyName: CPP_COM_WSTR_MyComputer.as_ptr(),
};

// CDrivesFolder.cpp:102. This slice takes ownership of the definition now that the .cpp is
// gone; `#[no_mangle]` because drives_folder.rs still reaches it as an `extern static mut`
// under the unmangled C name, and the closure lists it as a `data` symbol the Rust archive
// needs. Without the attribute the symbol is Rust-mangled and that extern dangles.
#[no_mangle]
pub static mut g_IsFloppyCache: UINT = 0;

pub const RETRY_COUNT: i32 = 3;

pub const RETRY_SLEEP: i32 = 250;

#[repr(C)]
pub struct shvheader {
    pub colnameid: i32,
    pub colstate: SHCOLSTATEF,
    pub fmt: i32,
    pub cxChar: i32,
}

pub static MyComputerSFHeader: [shvheader; 5] = [
    shvheader {
        colnameid: IDS_SHV_COLUMN_NAME as i32,
        colstate: SHCOLSTATE_TYPE_STR | SHCOLSTATE_ONBYDEFAULT,
        fmt: LVCFMT_LEFT as i32,
        cxChar: 15,
    },
    shvheader {
        colnameid: IDS_SHV_COLUMN_TYPE as i32,
        colstate: SHCOLSTATE_TYPE_STR | SHCOLSTATE_ONBYDEFAULT,
        fmt: LVCFMT_LEFT as i32,
        cxChar: 10,
    },
    shvheader {
        colnameid: IDS_SHV_COLUMN_DISK_CAPACITY as i32,
        colstate: SHCOLSTATE_TYPE_STR | SHCOLSTATE_ONBYDEFAULT,
        fmt: LVCFMT_RIGHT as i32,
        cxChar: 10,
    },
    shvheader {
        colnameid: IDS_SHV_COLUMN_DISK_AVAILABLE as i32,
        colstate: SHCOLSTATE_TYPE_STR | SHCOLSTATE_ONBYDEFAULT,
        fmt: LVCFMT_RIGHT as i32,
        cxChar: 10,
    },
    shvheader {
        colnameid: IDS_SHV_COLUMN_COMMENTS as i32,
        colstate: SHCOLSTATE_TYPE_STR,
        fmt: LVCFMT_LEFT as i32,
        cxChar: 10,
    },
];

pub const dwComputerAttributes: u32 = SFGAO_CANRENAME
    | SFGAO_CANDELETE
    | SFGAO_HASPROPSHEET
    | SFGAO_DROPTARGET
    | SFGAO_FILESYSANCESTOR
    | SFGAO_FOLDER
    | SFGAO_HASSUBFOLDER
    | SFGAO_CANLINK;

pub const dwControlPanelAttributes: u32 = SFGAO_HASSUBFOLDER | SFGAO_FOLDER | SFGAO_CANLINK;

pub const dwDriveAttributes: u32 = SFGAO_HASSUBFOLDER
    | SFGAO_FILESYSTEM
    | SFGAO_FOLDER
    | SFGAO_FILESYSANCESTOR
    | SFGAO_DROPTARGET
    | SFGAO_HASPROPSHEET
    | SFGAO_CANRENAME
    | SFGAO_CANLINK
    | SFGAO_CANCOPY;
unsafe extern "C" fn IsRegItem(pidl: PCUITEMID_CHILD) -> *const CLSID {
    if !pidl.is_null() && (*pidl).mkid.cb == 2 + 2 + core::mem::size_of::<CLSID>() as u16 {
        if (*pidl).mkid.abID[0] == PT_SHELLEXT as u8 || (*pidl).mkid.abID[0] == PT_GUID as u8 {
            // `abID` is a C flexible array member, which bindgen renders as `[BYTE; 1]`, so
            // indexing [2] is a const-evaluable out-of-bounds and rustc denies it
            // (unconditional_panic). The C is pointer arithmetic into the tail, so spell it
            // that way. NB: `cargo check` on the dev profile does NOT fire this lint — it
            // needs const-prop, so only the optimized build catches it.
            return core::ptr::addr_of!((*pidl).mkid.abID).cast::<u8>().add(2) as *const CLSID;
        }
    }
    core::ptr::null()
}

unsafe fn IsRegItem_2(pidl: PCUITEMID_CHILD, clsid: *const CLSID) -> bool {
    let pClass = IsRegItem(pidl);
    !pClass.is_null() && IsEqualGUID(pClass, clsid)
}
unsafe extern "C" fn GetDriveNumber(pidl: PCUITEMID_CHILD) -> i8 {
    if _ILIsDrive(pidl) == 0 {
        return -1;
    }
    let letter = (*(*(pidl as *const PIDLDATA)).u.drive.as_ref()).szDriveName[0];
    let number: BYTE = ((letter as BYTE) | 32).wrapping_sub(b'a');
    if number < 26 {
        number as i8
    } else {
        -1
    }
}
unsafe extern "C" fn GetCachedDriveType(DrvNum: i8) -> u32 {
    if DrvNum < 0 || DrvNum >= 26 {
        return DRIVE_UNKNOWN;
    }
    if g_IsFloppyCache & (1 << DrvNum) != 0 {
        return DRIVE_REMOVABLE;
    }
    let DrvPath: [u16; 4] = [
        LOWORD((b'A' as u32) + (DrvNum as u32)) as u16,
        b':' as u16,
        b'\\' as u16,
        0u16,
    ];
    GetDriveTypeW(DrvPath.as_ptr())
}

unsafe fn GetCachedDriveType_2(DrivePath: PCWSTR) -> u32 {
    GetCachedDriveType(GetDriveNumber_Path(DrivePath))
}
unsafe extern "C" fn IsFloppyDrive(DrivePath: PCWSTR) -> bool {
    let DrvNum = GetDriveNumber_Path(DrivePath);
    if DrvNum < 0 {
        return false;
    }
    if g_IsFloppyCache & (1 << DrvNum) != 0 {
        return true;
    }
    let Type = GetCachedDriveType(DrvNum);
    if Type == DRIVE_REMOVABLE && IsDriveFloppyW(DrivePath) != 0 {
        g_IsFloppyCache |= 1 << DrvNum;
        return true;
    }
    false
}
#[no_mangle]
pub unsafe extern "C" fn GetDrivePath(pidl: PCUITEMID_CHILD, Path: PWSTR) -> i8 {
    let number = GetDriveNumber(pidl);
    if number < 0 {
        return number;
    }
    *Path.offset(0) = (b'A' as u16) + (number as u16);
    *Path.offset(1) = 2 as u16;
    *Path.offset(3) = b'\\' as u16;
    *Path.offset(3) = 0u16;
    return number;
}
unsafe extern "C" fn _ILGetRemovableTypeId(pidl: PCUITEMID_CHILD) -> u32 {
    let mut buf: [u16; 8] = [0; 8];
    if GetDrivePath(pidl, buf.as_mut_ptr()) >= 0 && IsFloppyDrive(buf.as_ptr()) {
        return SHDID_COMPUTER_DRIVE35;
    }
    SHDID_COMPUTER_REMOVABLE
}
#[no_mangle]
pub unsafe extern "C" fn _ILGetDriveType(pidl: LPCITEMIDLIST) -> u32 {
    let mut szDrive: [u16; 8] = [0; 8];
    if _ILGetDrive(pidl, szDrive.as_mut_ptr(), _countof!(szDrive)) == 0 {
        return DRIVE_UNKNOWN;
    }
    return GetCachedDriveType(GetDriveNumber_Path(szDrive.as_ptr()));
}
unsafe extern "C" fn SHELL_GetAutoRunInfPath(DrvPath: PCWSTR, AriPath: PWSTR, ForInvoke: BOOL) -> u32 {
    let DrvNum = GetDriveNumber_Path(DrvPath);
    if DrvNum < 0 || *DrvPath.offset(1) != ':' as u16 || (*DrvPath.offset(2) != 0 && (*DrvPath.offset(2) != '\\' as u16 || *DrvPath.offset(3) != 0)) {
        return 0;
    }
    if ForInvoke == 0 && IsFloppyDrive(DrvPath) {
        return 0;
    }
    if GetCachedDriveType(DrvNum) <= DRIVE_NO_ROOT_DIR {
        return 0;
    }
    (wsprintfW(AriPath, CPP_COM_WSTR_DFA857FD38EE.as_ptr(), *DrvPath.offset(0) as INT)) as u32
}
#[no_mangle]
pub unsafe extern "C" fn SHELL32_IsShellFolderNamespaceItemHidden(SubKey: PCWSTR, Clsid: REFCLSID) -> BOOL {
    let mut path: [u16; MAX_PATH as usize] = core::mem::zeroed();
    let mut name: [u16; CHARS_IN_GUID as usize] = core::mem::zeroed();
    wsprintfW(path.as_mut_ptr(), CPP_COM_WSTR_BA2342D760A2.as_ptr(), REGSTR_PATH_EXPLORER, SubKey);
    SHELL32_GUIDToStringW(Clsid.cast(), name.as_mut_ptr());
    // Check the value in both HKEY_CURRENT_USER and HKEY_LOCAL_MACHINE (upstream
    // 9342ed43d9a; the pre-fix body read HKCU only). Returns true if the item should
    // be hidden in DefView but not in the Explorer folder tree.
    crate::island::SHRegGetBoolUSValueW(path.as_ptr(), name.as_ptr(), FALSE, FALSE)
}
unsafe extern "C" fn TryToLockOrUnlockDrive(hDrive: HANDLE, bLock: BOOL) -> BOOL {
    let mut dwError: DWORD = NO_ERROR;
    let mut dwBytesReturned: DWORD = 0;
    let dwCode: DWORD = if bLock != 0 { FSCTL_LOCK_VOLUME } else { FSCTL_UNLOCK_VOLUME };
    let mut i: DWORD = 0;
    while i < RETRY_COUNT as u32 {
        if DeviceIoControl(
            hDrive,
            dwCode,
            core::ptr::null_mut(),
            0,
            core::ptr::null_mut(),
            0,
            &mut dwBytesReturned,
            core::ptr::null_mut(),
        ) != 0
        {
            return TRUE;
        }

        dwError = GetLastError();
        if dwError == ERROR_INVALID_FUNCTION {
            break;
        }

        Sleep(RETRY_SLEEP as u32);
        i += 1;
    }
    SetLastError(dwError);
    return FALSE;
}
unsafe extern "C" fn DoEjectDrive(physical: *const WCHAR, nDriveType: UINT, pnStringID: *mut INT) -> BOOL {
    let dwAccessMode = GENERIC_READ;
    let dwShareMode = FILE_SHARE_READ | FILE_SHARE_WRITE;

    let hDrive = CreateFileW(physical, dwAccessMode, dwShareMode, core::ptr::null_mut(), OPEN_EXISTING, 0, core::ptr::null_mut());
    if hDrive == INVALID_HANDLE_VALUE {
        return FALSE;
    }

    let mut bResult: BOOL;
    let mut bNeedUnlock = FALSE;
    let mut dwBytesReturned: DWORD = 0;
    let mut dwError = NO_ERROR;
    let mut removal: PREVENT_MEDIA_REMOVAL = core::mem::zeroed();
    loop {
        bResult = TryToLockOrUnlockDrive(hDrive, TRUE);
        if bResult == 0 {
            dwError = GetLastError();
            *pnStringID = IDS_CANTLOCKVOLUME as i32;
            break;
        }
        bResult = DeviceIoControl(hDrive, FSCTL_DISMOUNT_VOLUME, core::ptr::null_mut(), 0, core::ptr::null_mut(), 0, &mut dwBytesReturned, core::ptr::null_mut());
        if bResult == 0 {
            dwError = GetLastError();
            *pnStringID = IDS_CANTDISMOUNTVOLUME as i32;
            bNeedUnlock = TRUE;
            break;
        }
        removal.PreventMediaRemoval = FALSE as u8;
        bResult = DeviceIoControl(hDrive, IOCTL_STORAGE_MEDIA_REMOVAL, &mut removal as *mut _ as *mut _, core::mem::size_of::<PREVENT_MEDIA_REMOVAL>() as DWORD, core::ptr::null_mut(), 0, &mut dwBytesReturned, core::ptr::null_mut());
        if bResult == 0 {
            *pnStringID = IDS_CANTEJECTMEDIA as i32;
            dwError = GetLastError();
            bNeedUnlock = TRUE;
            break;
        }
        bResult = DeviceIoControl(hDrive, IOCTL_STORAGE_EJECT_MEDIA, core::ptr::null_mut(), 0, core::ptr::null_mut(), 0, &mut dwBytesReturned, core::ptr::null_mut());
        if bResult == 0 {
            *pnStringID = IDS_CANTEJECTMEDIA as i32;
            dwError = GetLastError();
            bNeedUnlock = TRUE;
            break;
        }
        break;
    }

    if bNeedUnlock != 0 {
        TryToLockOrUnlockDrive(hDrive, FALSE);
    }

    CloseHandle(hDrive);

    SetLastError(dwError);
    bResult
}
// `CStubWindow32 stub;` is an ATL CWindowImpl<> local, still owned by utils.cpp — it cannot
// come across, and the translated `CStubWindow32::new()` / `.CreateStub()` / `.DestroyWindow()`
// name a Rust type that does not exist. Reach it the way propsheet.rs already does, through the
// C-linkage `CStubWindow32_*_RustBridge` pair, and drive the window by its HWND afterwards.
// One `extern "system"` definition covers both arches; the assembler's cfg-split into stdcall
// and C forms was only spelling the same convention twice.
unsafe extern "system" fn DoFormatDriveThread(args: LPVOID) -> DWORD {
    let nDrive: UINT = PtrToUlong(args);
    // Arbitrary path, it just needs to include nDrive.
    let szPath: [WCHAR; 2] = [LOWORD((b'A' as DWORD).wrapping_add(nDrive)) as WCHAR, 0];
    let mut hwnd: HWND = core::ptr::null_mut();
    let mut stub: *mut c_void = core::ptr::null_mut();
    let hr: HRESULT = CStubWindow32_CreateStub_RustBridge(
        CSTUBWINDOW32_TYPE_FORMATDRIVE,
        szPath.as_ptr(),
        core::ptr::null(),
        core::ptr::addr_of_mut!(hwnd).cast(),
        &mut stub,
    );
    if hr < 0 {
        return hr as DWORD;
    }
    SHFormatDrive(hwnd, nDrive, SHFMT_ID_DEFAULT, 0);
    let ok = DestroyWindow(hwnd);
    CStubWindow32_DestroyStub_RustBridge(stub);
    ok as DWORD
}
unsafe extern "C" fn DoFormatDriveAsync(hwnd: HWND, nDrive: UINT) -> HRESULT {
    let succ = SHCreateThread(
        core::mem::transmute(DoFormatDriveThread as usize),
        UlongToPtr(nDrive),
        CTF_PROCESS_REF,
        None,
    );
    if succ != 0 {
        S_OK
    } else {
        E_FAIL
    }
}
#[no_mangle]
pub unsafe extern "system" fn DrivesContextMenuCallback(
    psf: *mut IShellFolder,
    hwnd: HWND,
    pdtobj: *mut IDataObject,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> HRESULT {
    if uMsg != DFM_MERGECONTEXTMENU && uMsg != DFM_INVOKECOMMAND && uMsg != DFM_MAPCOMMANDNAME {
        return SHELL32_DefaultContextMenuCallBack(psf, pdtobj, uMsg);
    }

    let mut pidlFolder: PIDLIST_ABSOLUTE = core::ptr::null_mut();
    let mut apidl: *mut PITEMID_CHILD = core::ptr::null_mut();
    let mut cidl: UINT = 0;
    let mut dwFlags: DWORD = 0;
    let mut hr: HRESULT = SH_GetApidlFromDataObject(
        pdtobj,
        &mut pidlFolder,
        &mut apidl,
        &mut cidl,
    );
    if FAILED_UNEXPECTEDLY!(hr) {
        return hr;
    }

    let mut szDrive: [WCHAR; 8] = [0; 8];
    if _ILGetDrive(
        *apidl.offset(0 as isize),
        szDrive.as_mut_ptr(),
        _countof!(szDrive),
    ) == 0
    {
        SHFree(pidlFolder as *mut c_void);
        _ILFreeaPidl(apidl, cidl);
        return E_FAIL;
    }
    let nDriveType: UINT = GetCachedDriveType_2(szDrive.as_ptr());
    if GetVolumeInformationW(
        szDrive.as_ptr(),
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        &mut dwFlags,
        core::ptr::null_mut(),
        0,
    ) == 0
    {
        if nDriveType >= DRIVE_REMOTE {
            dwFlags = FILE_READ_ONLY_VOLUME;
        } else {
            dwFlags = 0;
        }
    }

    if uMsg == DFM_MERGECONTEXTMENU {
        let pqcminfo: *mut QCMINFO = lParam as *mut QCMINFO;

        let idCmdFirst: UINT = (*pqcminfo).idCmdFirst;
        let mut idCmd: UINT = 0;
        if (dwFlags & FILE_READ_ONLY_VOLUME) == 0
            && nDriveType != DRIVE_REMOTE
            && cidl == 1
        {
            idCmd = idCmdFirst + CMDID_FORMAT;
            _InsertMenuItemW(
                (*pqcminfo).hmenu,
                (*pqcminfo).indexMenu,
                TRUE,
                0,
                MFT_SEPARATOR,
                core::ptr::null_mut(),
                0,
            );
            (*pqcminfo).indexMenu += 1;
            _InsertMenuItemW(
                (*pqcminfo).hmenu,
                (*pqcminfo).indexMenu,
                TRUE,
                idCmd,
                MFT_STRING,
                MAKEINTRESOURCEW(IDS_FORMATDRIVE),
                MFS_ENABLED,
            );
            (*pqcminfo).indexMenu += 1;
        }
        if nDriveType == DRIVE_REMOVABLE || nDriveType == DRIVE_CDROM {
            idCmd = idCmdFirst + CMDID_EJECT;
            _InsertMenuItemW(
                (*pqcminfo).hmenu,
                (*pqcminfo).indexMenu,
                TRUE,
                0,
                MFT_SEPARATOR,
                core::ptr::null_mut(),
                0,
            );
            (*pqcminfo).indexMenu += 1;
            _InsertMenuItemW(
                (*pqcminfo).hmenu,
                (*pqcminfo).indexMenu,
                TRUE,
                idCmd,
                MFT_STRING,
                MAKEINTRESOURCEW(IDS_EJECT),
                MFS_ENABLED,
            );
            (*pqcminfo).indexMenu += 1;
        }
        if nDriveType == DRIVE_REMOTE {
            idCmd = idCmdFirst + CMDID_DISCONNECT;
            _InsertMenuItemW(
                (*pqcminfo).hmenu,
                (*pqcminfo).indexMenu,
                TRUE,
                0,
                MFT_SEPARATOR,
                core::ptr::null_mut(),
                0,
            );
            (*pqcminfo).indexMenu += 1;
            _InsertMenuItemW(
                (*pqcminfo).hmenu,
                (*pqcminfo).indexMenu,
                TRUE,
                idCmd,
                MFT_STRING,
                MAKEINTRESOURCEW(IDS_DISCONNECT),
                MFS_ENABLED,
            );
            (*pqcminfo).indexMenu += 1;
        }

        if idCmd != 0 {
            (*pqcminfo).idCmdFirst = idCmd + 1; // Note: This assumes the items above are added in ascending id order
        }
        hr = S_OK;
    } else if uMsg == DFM_INVOKECOMMAND {
        let mut wszBuf: [WCHAR; 4] = [0; 4];
        core::ptr::copy_nonoverlapping(
            CPP_COM_WSTR_7F8732C542BA.as_ptr(),
            wszBuf.as_mut_ptr(),
            CPP_COM_WSTR_7F8732C542BA.len(),
        );
        wszBuf[0] = szDrive[0] as WCHAR;

        let mut nStringID: INT = 0;
        let mut dwError: DWORD = NO_ERROR;

        if wParam == DFM_CMD_PROPERTIES as WPARAM {
            debug_assert!(!pdtobj.is_null());
            hr = SHELL32_ShowFilesystemItemPropertiesDialogAsync(pdtobj);
        } else {
            if wParam == CMDID_FORMAT as WPARAM {
                hr = DoFormatDriveAsync(hwnd, szDrive[0] as u32 - 'A' as u32);
            } else if wParam == CMDID_EJECT as WPARAM {
                let mut physical: [WCHAR; 10] = [0; 10];
                wsprintfW(
                    physical.as_mut_ptr(),
                    CPP_COM_WSTR_PHYSICALDRIVE.as_ptr(),
                    szDrive[0] as INT,
                );

                if DoEjectDrive(physical.as_ptr(), nDriveType, &mut nStringID) != 0 {
                    SHChangeNotify(
                        SHCNE_MEDIAREMOVED as i32,
                        SHCNF_PATHW | SHCNF_FLUSHNOWAIT,
                        wszBuf.as_ptr() as *const c_void,
                        core::ptr::null_mut(),
                    );
                } else {
                    dwError = GetLastError();
                }
            } else if wParam == CMDID_DISCONNECT as WPARAM {
                wszBuf[2] = UNICODE_NULL;
                dwError = WNetCancelConnection2W(
                    wszBuf.as_ptr(),
                    0,
                    FALSE,
                );
                if dwError == NO_ERROR {
                    SHChangeNotify(
                        SHCNE_DRIVEREMOVED as i32,
                        SHCNF_PATHW | SHCNF_FLUSHNOWAIT,
                        wszBuf.as_ptr() as *const c_void,
                        core::ptr::null_mut(),
                    );
                } else {
                    nStringID = IDS_CANTDISCONNECT as i32;
                }
            } else if hr == S_OK {
                hr = E_INVALIDARG; // Don't eat DFM_CMD_* in CDefaultContextMenu::Do*
            }
        }

        if nStringID != 0 {
            let mut szFormat: [WCHAR; 128] = [0; 128];
            let mut szMessage: [WCHAR; 128] = [0; 128];
            LoadStringW(
                shell32_hInstance,
                nStringID as UINT,
                szFormat.as_mut_ptr(),
                _countof!(szFormat) as i32,
            );
            wsprintfW(szMessage.as_mut_ptr(), szFormat.as_ptr(), dwError);
            MessageBoxW(
                hwnd,
                szMessage.as_ptr(),
                core::ptr::null_mut(),
                MB_ICONERROR,
            );
        }
    } else if uMsg == DFM_MAPCOMMANDNAME {
        let mut ici: CMINVOKECOMMANDINFOEX = core::mem::zeroed();
        ici.cbSize = core::mem::size_of::<CMINVOKECOMMANDINFOEX>() as DWORD;
        ici.fMask = SEE_MASK_UNICODE as DWORD; // CMIC_MASK_UNICODE (shobjidl.idl: == SEE_MASK_UNICODE)
        ici.lpVerbW = lParam as PWSTR as LPCWSTR;
        hr = crate::shell32::utils::SHELL_MapContextMenuVerbToCmdId(
            &mut ici as *mut CMINVOKECOMMANDINFOEX as LPCMINVOKECOMMANDINFO,
            core::ptr::addr_of!(g_VerbMap).cast(),
        );
        if hr >= 0 {
            *(wParam as *mut INT) = hr;
            hr = S_OK;
        }
    }

    SHFree(pidlFolder as *mut c_void);
    _ILFreeaPidl(apidl, cidl);

    hr
}
#[no_mangle]
pub unsafe extern "C" fn CDrivesContextMenu_CreateInstance(
    pidlFolder: PCIDLIST_ABSOLUTE,
    hwnd: HWND,
    cidl: UINT,
    apidl: PCUITEMID_CHILD_ARRAY,
    psf: *mut IShellFolder,
    ppcm: *mut *mut IContextMenu,
) -> HRESULT {
    let mut keys: CRegKeyHandleArray = core::mem::zeroed();
    AddClassKeyToArray(CPP_COM_WSTR_32491EF24F87.as_ptr(), keys.hKeys.as_mut_ptr(), &mut keys.cKeys);
    AddClassKeyToArray(CPP_COM_WSTR_EB4CE2024A43.as_ptr(), keys.hKeys.as_mut_ptr(), &mut keys.cKeys);
    CDefFolderMenu_Create2(
        pidlFolder,
        hwnd,
        cidl,
        apidl,
        psf,
        // This callback's IShellFolder/IDataObject resolve from `shellcom` while the island's
        // LPFNDFMCALLBACK spells them from `com_sys` — one C ABI, two nominal types. Transmute
        // to the NAMED typedef so the calling convention stays explicit rather than inferred;
        // an unannotated transmute at a callback-install site is how a convention gets erased.
        core::mem::transmute::<_, LPFNDFMCALLBACK>(Some(
            DrivesContextMenuCallback
                as unsafe extern "system" fn(*mut IShellFolder, HWND, *mut IDataObject, UINT, WPARAM, LPARAM) -> HRESULT,
        )),
        keys.cKeys,
        keys.hKeys.as_ptr(),
        ppcm,
    )
}
unsafe extern "C" fn getIconLocationForDrive(
    pszDrivePath: PCWSTR,
    szIconFile: PWSTR,
    cchMax: u32,
    piIndex: *mut i32,
    pGilOut: *mut u32,
) -> HRESULT {
    let mut wszPath: [u16; MAX_PATH as usize] = core::mem::zeroed();
    let mut wszAutoRunInfPath: [u16; MAX_PATH as usize] = core::mem::zeroed();
    let mut wszValue: [u16; MAX_PATH as usize] = core::mem::zeroed();
    let mut wszTemp: [u16; MAX_PATH as usize] = core::mem::zeroed();

    if SHELL_GetAutoRunInfPath(pszDrivePath, wszAutoRunInfPath.as_mut_ptr(), 0) == 0 {
        return E_FAIL;
    }
    wcscpy(wszPath.as_mut_ptr(), pszDrivePath);

    if GetPrivateProfileStringW(
        CPP_COM_WSTR_0D857F239DCA.as_ptr(),
        CPP_COM_WSTR_7BE637194C2D.as_ptr(),
        core::ptr::null(),
        wszValue.as_mut_ptr(),
        _countof!(wszValue),
        wszAutoRunInfPath.as_ptr(),
    ) != 0
        && wszValue[0] != 0
    {
        ExpandEnvironmentStringsW(wszValue.as_ptr(), wszTemp.as_mut_ptr(), _countof!(wszTemp));

        *piIndex = PathParseIconLocationW(wszTemp.as_mut_ptr());

        if PathIsRelativeW(wszTemp.as_ptr()) != 0 {
            PathAppendW(wszPath.as_mut_ptr(), wszTemp.as_ptr());
        } else {
            StringCchCopyW(wszPath.as_mut_ptr(), _countof!(wszPath) as usize, wszTemp.as_ptr());
        }

        GetFullPathNameW(wszPath.as_ptr(), cchMax, szIconFile, core::ptr::null_mut());
        return S_OK;
    }
    E_FAIL
}
unsafe extern "C" fn getLabelForDriveFromAutoRun(wszPath: PCWSTR, szLabel: PWSTR, cchMax: u32) -> i32 {
    let mut wszAutoRunInfPath: [u16; MAX_PATH as usize] = core::mem::zeroed();
    let mut wszTemp: [u16; MAX_PATH as usize] = core::mem::zeroed();

    if SHELL_GetAutoRunInfPath(wszPath, wszAutoRunInfPath.as_mut_ptr(), 0) == 0 {
        return E_FAIL;
    }

    if GetPrivateProfileStringW(CPP_COM_WSTR_0D857F239DCA.as_ptr(), CPP_COM_WSTR_6D957E170BC9.as_ptr(), core::ptr::null_mut(), wszTemp.as_mut_ptr(), _countof!(wszTemp), wszAutoRunInfPath.as_ptr()) != 0 && wszTemp[0] != 0 {
        return StringCchCopyW(szLabel, cchMax as usize, wszTemp.as_ptr());
    }
    E_FAIL
}
unsafe extern "C" fn GetRawDriveLabel(DrivePath: PCWSTR, szLabel: PWSTR, cchMax: u32) -> HRESULT {
    if GetVolumeInformationW(DrivePath, szLabel, cchMax, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), 0) != 0 {
        return if *szLabel != 0 { S_OK } else { S_FALSE };
    }
    HResultFromWin32(GetLastError())
}
unsafe extern "C" fn GetDriveLabel(DrivePath: PCWSTR, szLabel: PWSTR, cchMax: u32) -> HRESULT {
    let hr = getLabelForDriveFromAutoRun(DrivePath, szLabel, cchMax);
    if hr == S_OK {
        S_OK
    } else {
        GetRawDriveLabel(DrivePath, szLabel, cchMax)
    }
}
unsafe extern "C" fn GetRegCustomizedDriveIcon(
    DriveNum: i8,
    szIconFile: PWSTR,
    cchMax: u32,
    piIndex: *mut i32,
    pGilOut: *mut u32,
) -> bool {
    let mut szKey: [u16; 200] = [0; 200];
    let chDrv: u16 = (b'A' as u16) + (DriveNum as u16);
    wsprintfW(szKey.as_mut_ptr(), CPP_COM_WSTR_2618E721CE41.as_ptr(), REGSTR_PATH_EXPLORER, chDrv as INT);
    let mut cb: u32 = cchMax * core::mem::size_of::<u16>() as u32;
    let mut err: u32 = (RegGetValueW(HKEY_LOCAL_MACHINE, szKey.as_ptr(), core::ptr::null(), RRF_RT_REG_SZ, core::ptr::null_mut(), szIconFile as *mut core::ffi::c_void, core::ptr::addr_of_mut!(cb))) as u32;
    if err != ERROR_SUCCESS {
        cb = cchMax * core::mem::size_of::<u16>() as u32;
        wsprintfW(szKey.as_mut_ptr(), CPP_COM_WSTR_6DBED233E95A.as_ptr(), chDrv as INT);
        err = (RegGetValueW(HKEY_CLASSES_ROOT, szKey.as_ptr(), core::ptr::null(), RRF_RT_REG_SZ, core::ptr::null_mut(), szIconFile as *mut core::ffi::c_void, core::ptr::addr_of_mut!(cb))) as u32;
    }
    if err != ERROR_SUCCESS {
        return false;
    }
    *piIndex = PathParseIconLocationW(szIconFile);
    *szIconFile != UNICODE_NULL
}
#[no_mangle]
pub unsafe extern "C" fn CDrivesExtractIcon_CreateInstance(
    psf: *mut IShellFolder,
    pidl: LPCITEMIDLIST,
    riid: REFIID,
    ppvOut: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let mut initIcon: *mut IDefaultExtractIconInit = core::ptr::null_mut();
    let hr = SHCreateDefaultExtractIcon(
        core::ptr::addr_of!(IID_IDefaultExtractIconInit), core::ptr::addr_of_mut!(initIcon).cast()
    );
    if FAILED_UNEXPECTEDLY!(hr) {
        return hr;
    }

    let mut wTemp: [u16; MAX_PATH as usize] = core::mem::zeroed();
    let mut szDrive: [u16; 8] = core::mem::zeroed();
    let mut icon_idx: i32 = 0;
    let mut reg_idx: i32 = 0;
    let mut GilOut: u32 = 0;
    let mut bFloppy: u8 = FALSE as u8;
    let DriveNum: i8 = GetDrivePath(pidl, szDrive.as_mut_ptr());
    let mut DriveType: u32 = GetCachedDriveType(DriveNum);
    if DriveType > DRIVE_RAMDISK {
        DriveType = DRIVE_FIXED;
    }

    match DriveType {
        DRIVE_REMOVABLE => {
            bFloppy = IsFloppyDrive(szDrive.as_ptr()) as u8;
            reg_idx = if bFloppy != 0 {
                IDI_SHELL_3_14_FLOPPY as i32
            } else {
                IDI_SHELL_REMOVEABLE as i32
            };
        }
        DRIVE_FIXED => {
            reg_idx = IDI_SHELL_DRIVE as i32;
        }
        DRIVE_REMOTE => {
            reg_idx = IDI_SHELL_NETDRIVE as i32;
        }
        DRIVE_CDROM => {
            reg_idx = IDI_SHELL_CDROM as i32;
        }
        DRIVE_RAMDISK => {
            reg_idx = IDI_SHELL_RAMDISK as i32;
        }
        _ => {}
    }

    let hr = getIconLocationForDrive(
        szDrive.as_ptr(),
        wTemp.as_mut_ptr(),
        _countof!(wTemp),
        &mut icon_idx,
        &mut GilOut,
    );
    if hr >= 0 {
        ((*(*initIcon).lpVtbl).SetNormalIcon)(initIcon, wTemp.as_ptr(), icon_idx);
    } else if DriveType > DRIVE_NO_ROOT_DIR
        && GetRegCustomizedDriveIcon(
            DriveNum,
            wTemp.as_mut_ptr(),
            _countof!(wTemp),
            &mut icon_idx,
            &mut GilOut,
        )
    {
        ((*(*initIcon).lpVtbl).SetNormalIcon)(initIcon, wTemp.as_ptr(), icon_idx);
    } else if reg_idx != 0
        && HLM_GetIconW(
            (reg_idx - 1) as u32,
            wTemp.as_mut_ptr(),
            _countof!(wTemp),
            &mut icon_idx,
        ) != 0
    {
        ((*(*initIcon).lpVtbl).SetNormalIcon)(initIcon, wTemp.as_ptr(), icon_idx);
    } else if DriveType == DRIVE_FIXED
        && HCR_GetIconW(
            CPP_COM_WSTR_32491EF24F87.as_ptr(),
            wTemp.as_mut_ptr(),
            core::ptr::null_mut(),
            _countof!(wTemp),
            &mut icon_idx,
        ) != 0
    {
        ((*(*initIcon).lpVtbl).SetNormalIcon)(initIcon, wTemp.as_ptr(), icon_idx);
    } else {
        if DriveType == DRIVE_REMOVABLE && bFloppy != 0 {
            icon_idx = IDI_SHELL_3_14_FLOPPY as i32;
        } else {
            icon_idx = iDriveIconIds[DriveType as usize];
        }
        ((*(*initIcon).lpVtbl).SetNormalIcon)(initIcon, swShell32Name.as_ptr(), -icon_idx);
    }

    ((*(*initIcon).lpVtbl).QueryInterface)(initIcon, riid.cast(), ppvOut)
}

#[no_mangle]
pub unsafe extern "system" fn CDrivesFolder_CreateRegFolder_RustBridge(
    psfOuter: *mut IShellFolder,
    pidlRoot: *const ITEMIDLIST,
    riid: *const GUID,
    ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let RegInit = REGFOLDERINITDATA {
        psfOuter,
        pInfo: core::ptr::addr_of!(g_RegFolderInfo),
    };
    CRegFolder_CreateInstance(core::ptr::addr_of!(RegInit) as PREGFOLDERINITDATA, pidlRoot, riid, ppv)
}

#[no_mangle]
pub unsafe extern "system" fn CDrivesFolderEnum_CreateInstance_RustBridge(
    hwndOwner: HWND,
    dwFlags: DWORD,
    pRegEnumerator: *mut IEnumIDList,
    riid: REFIID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    // `ShellObjectCreatorInit<CDrivesFolderEnum>` is a C++ template instantiation with no
    // translated body; the concrete factory is the sibling `--cpp-com` assembly. Its
    // `IEnumIDList` is that module's own real-vtable type, nominally distinct from this
    // file's prelude-sourced opaque one over an identical ABI, hence the `.cast()`.
    crate::drives_folder_enum::CDrivesFolderEnum_CreateInstance(
        hwndOwner, dwFlags, pRegEnumerator.cast(), riid, ppv)
}

#[no_mangle]
pub unsafe extern "system" fn CDrivesContextMenu_CreateInstance_RustBridge(
    pidlFolder: PCIDLIST_ABSOLUTE,
    hwnd: HWND,
    cidl: UINT,
    apidl: PCUITEMID_CHILD_ARRAY,
    psf: *mut IShellFolder,
    ppcm: *mut *mut IContextMenu,
) -> HRESULT {
    CDrivesContextMenu_CreateInstance(pidlFolder, hwnd, cidl, apidl, psf, ppcm)
}

#[no_mangle]
pub unsafe extern "system" fn CDrivesExtractIcon_CreateInstance_RustBridge(
    psf: *mut IShellFolder,
    pidl: LPCITEMIDLIST,
    riid: REFIID,
    ppvOut: *mut LPVOID,
) -> HRESULT {
    CDrivesExtractIcon_CreateInstance(psf, pidl, riid, ppvOut)
}

#[no_mangle]
pub unsafe extern "system" fn SHELL32_IsShellFolderNamespaceItemHidden_RustBridge(
    SubKey: PCWSTR,
    pClsid: *const CLSID,
) -> BOOL {
    SHELL32_IsShellFolderNamespaceItemHidden(SubKey, pClsid.cast())
}

#[no_mangle]
pub unsafe extern "system" fn IsDriveFloppyW_RustBridge(pszDriveRoot: LPCWSTR) -> BOOL {
    IsDriveFloppyW(pszDriveRoot)
}

#[no_mangle]
pub unsafe extern "system" fn CDrivesFolder_SetDriveLabel_RustBridge(
    hwndOwner: HWND,
    DrivePath: PCWSTR,
    Label: PCWSTR,
) -> HRESULT {
    // `CDrivesFolder::SetDriveLabel` is a STATIC member, so it left with the .cpp and there
    // is no C symbol to import. The class half carries the body; its `--cpp-com` lowering
    // still takes a leading `this` the static never reads, so a null receiver is correct.
    crate::drives_folder::CDrivesFolder_SetDriveLabel_body(
        core::ptr::null_mut(), hwndOwner.cast(), DrivePath, Label)
}
