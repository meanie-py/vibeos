// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_189877A0FAAE: [WCHAR; 5] = [0x005C, 0x005C, 0x002E, 0x005C, 0x0000];
pub static CPP_COM_WSTR_F3A549CDECCE: [WCHAR; 7] = [0x0020, 0x0028, 0x0025, 0x0063, 0x003A, 0x0029, 0x0000];

use core::ffi::c_void;
// Substrate comes from the bindgen island via `prelude`, and from nowhere else.
// Re-importing the same names through shellcom/types/win32 does not add anything the
// prelude lacks -- it supplies them by a second path, and every one becomes ambiguous
// at the use site. See prelude.rs for why this is "instead of", not "as well as".
use crate::prelude::*;

// ---- file-scope preamble (translated) ----
// drvdefext.cpp:35-40 `typedef enum { HWPD_STANDARDLIST = 0, HWPD_LARGELIST, HWPD_MAX =
// HWPD_LARGELIST } HWPAGE_DISPLAYMODE`. C legally repeats a discriminant (HWPD_MAX aliases
// HWPD_LARGELIST); Rust's fieldless-enum discriminants must be distinct, so this is a plain
// integer + consts, matching the same substitution drvdefext.rs already made for this type.
pub type HWPAGE_DISPLAYMODE = DWORD;
pub const HWPD_STANDARDLIST: HWPAGE_DISPLAYMODE = 0;
pub const HWPD_LARGELIST: HWPAGE_DISPLAYMODE = 1;
pub const HWPD_MAX: HWPAGE_DISPLAYMODE = HWPD_LARGELIST;

pub type PHWPAGE_DISPLAYMODE = *mut HWPAGE_DISPLAYMODE;

// wdm.h/devioctl.h `typedef ULONG DEVICE_TYPE;`. Not carried by the island as a named alias
// (bindgen inlined the one struct field that uses it, FILE_FS_DEVICE_INFORMATION::DeviceType,
// straight to DWORD), so it is restored here for GetDriveTypeAndCharacteristics below.
pub type DEVICE_TYPE = ULONG;

pub static mut hChkdskDrvDialog: HWND = core::ptr::null_mut();
pub static mut bChkdskSuccess: BOOLEAN = 0;

#[repr(C)]
pub struct CDrop {
    pub hr: HRESULT,
    pub stgm: STGMEDIUM,
    pub hDrop: HDROP,
}
unsafe extern "C" fn GetDriveNameWithLetter(pwszText: *mut u16, cchTextMax: u32, pwszDrive: PCWSTR) {
    let mut dwMaxComp: u32 = 0;
    let mut dwFileSys: u32 = 0;
    let mut cchText: usize = 0;

    if GetVolumeInformationW(
        pwszDrive,
        pwszText,
        cchTextMax,
        core::ptr::null_mut(),
        &mut dwMaxComp,
        &mut dwFileSys,
        core::ptr::null_mut(),
        0
    ) != 0
    {
        cchText = wcslen(pwszText);
        if cchText == 0 {
            cchText = LoadStringW(
                shell32_hInstance,
                IDS_DRIVE_FIXED,
                pwszText,
                cchTextMax as INT
            ) as usize;
        }
    }

    strsafe_rust::StringCchPrintfW!(
        pwszText.offset(cchText as isize),
        (cchTextMax as usize).wrapping_sub(cchText),
        CPP_COM_WSTR_F3A549CDECCE.as_ptr(),
        // C variadic promotion: the `%c` argument (`pwszDrive[0]`, a WCHAR) is passed through
        // `...` as a promoted int-width value, and u16 does not implement Rust's VaArgSafe.
        *pwszDrive.offset(0) as UINT
    );
}
unsafe extern "C" fn InitializeChkDskDialog(hwndDlg: HWND, pwszDrive: PCWSTR) {
    let mut wszText: [u16; 100] = [0; 100];
    let mut Length: u32;
    SetWindowLongPtr(hwndDlg, DWLP_USER, pwszDrive as INT_PTR);

    Length = GetWindowTextW(hwndDlg, wszText.as_mut_ptr(), (core::mem::size_of_val(&wszText) / core::mem::size_of::<u16>()) as INT) as UINT;
    wszText[Length as usize] = 0x0020u16;
    GetDriveNameWithLetter(
        wszText.as_mut_ptr().offset((Length + 1) as isize),
        ((core::mem::size_of_val(&wszText) / core::mem::size_of::<u16>()) as u32) - Length - 1,
        pwszDrive
    );
    SetWindowTextW(hwndDlg, wszText.as_ptr());
}
// drvdefext.cpp:84 `static BOOLEAN NTAPI ChkdskCallback(...)`. NTAPI is `__stdcall` on x86 and
// the plain C convention elsewhere -- exactly what Rust's `extern "system"` already dispatches
// per target, so (unlike the machine-assembled cfg(x86)="stdcall"/cfg(not)="C" split this TU
// started with) a single "system" definition is both correct AND required: Chkdsk's
// PFMIFSCALLBACK parameter is typed `Option<unsafe extern "system" fn(...)>` in the island, and
// "system"/"stdcall"/"C" are distinct Rust fn-pointer types even on the target where they share
// an ABI, so `Some(ChkdskCallback)` only type-checks if the definition itself says "system".
// Same shape as drive.rs's FormatExCB (the FormatEx callback), which was never cfg-split.
unsafe extern "system" fn ChkdskCallback(
    Command: CALLBACKCOMMAND,
    SubAction: ULONG,
    ActionInfo: PVOID,
) -> BOOLEAN {
    let mut Progress: PDWORD;
    let mut pSuccess: PBOOLEAN;
    // bindgen prefixes every CALLBACKCOMMAND enumerator with the enum name, so the bare C
    // spellings (PROGRESS/DONE/...) are not constants here -- they would be fresh, always-
    // matching bindings that silently run the wrong arm for every command. Field manual:
    // drive.cpp's FormatExCB carries the same fix and the same warning.
    match Command {
        CALLBACKCOMMAND_PROGRESS => {
            Progress = ActionInfo as PDWORD;
            SendDlgItemMessageW(
                hChkdskDrvDialog,
                14002,
                PBM_SETPOS,
                (*Progress) as WPARAM,
                0,
            );
        }
        CALLBACKCOMMAND_DONE => {
            pSuccess = ActionInfo as PBOOLEAN;
            bChkdskSuccess = (*pSuccess);
        }
        CALLBACKCOMMAND_VOLUMEINUSE
        | CALLBACKCOMMAND_INSUFFICIENTRIGHTS
        | CALLBACKCOMMAND_FSNOTSUPPORTED
        | CALLBACKCOMMAND_CLUSTERSIZETOOSMALL => {
            bChkdskSuccess = FALSE as BOOLEAN;
        }
        _ => {}
    }

    return TRUE as BOOLEAN;
}
unsafe extern "C" fn ChkDskNow(hwndDlg: HWND, pwszDrive: PCWSTR) {
    let mut wszFs: [u16; 30] = [0; 30];
    let mut TotalNumberOfFreeBytes: ULARGE_INTEGER = core::mem::zeroed();
    let mut FreeBytesAvailableUser: ULARGE_INTEGER = core::mem::zeroed();
    let mut bCorrectErrors: BOOLEAN = FALSE as BOOLEAN;
    let mut bScanDrive: BOOLEAN = FALSE as BOOLEAN;

    if GetVolumeInformationW(
        pwszDrive,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        wszFs.as_mut_ptr(),
        _countof!(wszFs),
    ) == 0
    {
        return;
    }

    if GetDiskFreeSpaceExW(
        pwszDrive,
        &mut FreeBytesAvailableUser,
        &mut TotalNumberOfFreeBytes,
        core::ptr::null_mut(),
    ) == 0
    {
        return;
    }

    if SendDlgItemMessageW(
        hwndDlg,
        14000,
        BM_GETCHECK,
        0,
        0,
    ) == BST_CHECKED as LRESULT
    {
        bCorrectErrors = TRUE as BOOLEAN;
    }

    if SendDlgItemMessageW(
        hwndDlg,
        14001,
        BM_GETCHECK,
        0,
        0,
    ) == BST_CHECKED as LRESULT
    {
        bScanDrive = TRUE as BOOLEAN;
    }

    hChkdskDrvDialog = hwndDlg;
    bChkdskSuccess = FALSE as BOOLEAN;
    SendDlgItemMessageW(
        hwndDlg,
        14002,
        PBM_SETRANGE,
        0,
        MAKELONG!(0, 100) as LPARAM,
    );
    Chkdsk(
        pwszDrive as LPWSTR,
        wszFs.as_mut_ptr() as LPWSTR,
        bCorrectErrors,
        TRUE as BOOLEAN,
        FALSE as BOOLEAN,
        bScanDrive,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        Some(ChkdskCallback),
    );

    hChkdskDrvDialog = core::ptr::null_mut();
    bChkdskSuccess = FALSE as BOOLEAN;
}
unsafe extern "system" fn ChkDskDlg(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    match uMsg {
        WM_INITDIALOG => {
            SetWindowLongPtr(hwndDlg, DWLP_USER, lParam as LONG_PTR);
            InitializeChkDskDialog(hwndDlg, lParam as LPCWSTR);
            return TRUE as INT_PTR;
        }
        WM_COMMAND => {
            // LOWORD's island helper takes a DWORD; WPARAM is pointer-width (u64 on
            // amd64/arm64), so truncate to DWORD first -- truncation before-or-after
            // masking to the low 16 bits gives the same low word either way. The match
            // scrutinee is then cast to UINT to meet IDCANCEL/IDOK, which -- like every
            // resource ID here -- are untyped-int constants (u32), not u16.
            match LOWORD(wParam as DWORD) as UINT {
                IDCANCEL => {
                    EndDialog(hwndDlg, 0);
                }
                IDOK => {
                    let pwszDrive = GetWindowLongPtrW(hwndDlg, DWLP_USER) as LPCWSTR;
                    ChkDskNow(hwndDlg, pwszDrive);
                }
                _ => {}
            }
        }
        _ => {}
    }

    FALSE as INT_PTR
}
unsafe extern "C" fn GetDriveTypeAndCharacteristics(
    hDevice: HANDLE,
    pDeviceType: *mut DEVICE_TYPE,
    pCharacteristics: *mut ULONG,
) -> BOOL {
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut DeviceInfo: FILE_FS_DEVICE_INFORMATION = core::mem::zeroed();

    let Status = NtQueryVolumeInformationFile(
        hDevice,
        &mut IoStatusBlock,
        &mut DeviceInfo as *mut _ as PVOID,
        core::mem::size_of::<FILE_FS_DEVICE_INFORMATION>() as ULONG,
        FileFsDeviceInformation,
    );
    if Status == NO_ERROR as NTSTATUS {
        *pDeviceType = DeviceInfo.DeviceType;
        *pCharacteristics = DeviceInfo.Characteristics;
        return TRUE;
    }

    return FALSE;
}
#[no_mangle]
pub unsafe extern "C" fn IsDriveFloppyW(pszDriveRoot: PCWSTR) -> BOOL {
    let RootPath: PCWSTR = pszDriveRoot;
    let mut szRoot: [WCHAR; 16] = [0; 16];
    let mut szDeviceName: [WCHAR; 16] = [0; 16];
    let mut uType: UINT;
    let mut hDevice: HANDLE;
    let mut DeviceType: DEVICE_TYPE = 0;
    let mut ulCharacteristics: ULONG = 0;
    let mut ret: BOOL;

    lstrcpynW(szRoot.as_mut_ptr(), RootPath, _countof!(szRoot) as INT);

    if 0x0061u16 <= szRoot[0] && szRoot[0] <= 0x007Au16 {
        szRoot[0] = (szRoot[0] as i32 + (0x0041u16 as i32 - 0x0061u16 as i32)) as u16;
    }

    if 0x0041u16 <= szRoot[0] && szRoot[0] <= 0x005Au16 &&
        szRoot[1] == 0x003Au16 && szRoot[2] == 0 {
        szRoot[2] = 0x005Cu16;
        szRoot[3] = 0;
    }

    if PathIsRootW(szRoot.as_ptr()) == 0 {
        return FALSE;
    }

    uType = GetDriveTypeW(szRoot.as_ptr());
    if uType == DRIVE_REMOVABLE {
        if szRoot[0] == 0x0041u16 || szRoot[0] == 0x0042u16 {
            return TRUE;
        }
    } else {
        return FALSE;
    }

    lstrcpynW(szDeviceName.as_mut_ptr(), CPP_COM_WSTR_189877A0FAAE.as_ptr(), _countof!(szDeviceName) as INT);
    szDeviceName[4] = szRoot[0];
    szDeviceName[5] = 0x003Au16;
    szDeviceName[6] = UNICODE_NULL;

    hDevice = CreateFileW(
        szDeviceName.as_ptr(),
        FILE_READ_ATTRIBUTES,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        core::ptr::null_mut(),
        OPEN_EXISTING,
        0,
        core::ptr::null_mut()
    );
    if hDevice == INVALID_HANDLE_VALUE {
        return FALSE;
    }

    ret = FALSE;
    if GetDriveTypeAndCharacteristics(hDevice, &mut DeviceType, &mut ulCharacteristics) != 0 {
        if (ulCharacteristics & FILE_FLOPPY_DISKETTE) == FILE_FLOPPY_DISKETTE {
            ret = TRUE;
        }
    }

    CloseHandle(hDevice);

    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn IsDriveFloppyA(pszDriveRoot: PCSTR) -> BOOL {
    let mut szRoot: [WCHAR; 8] = [0; 8];
    MultiByteToWideChar(CP_ACP, 0, pszDriveRoot, -1, szRoot.as_mut_ptr(), _countof!(szRoot) as INT);
    return IsDriveFloppyW(szRoot.as_ptr());
}
unsafe extern "C" fn IsValidDrivePath(Path: PCWSTR) -> bool {
    GetDriveTypeW(Path) > DRIVE_NO_ROOT_DIR
}

// `SH_FormatByteSize_RustBridge` is deliberately NOT defined here.
//
// The real `SH_FormatByteSize` is C++-linkage in dialogs/filedefext.cpp:274 -- a DIFFERENT TU,
// and one still live in CMake. drvdefext.cpp only ever carried a forwarder to it. Publishing
// that forwarder from this file while importing it under the same `link_name` produced a
// `#[no_mangle]` function whose entire body was a call to ITSELF: unconditional infinite
// recursion, and a stack overflow the first time the drive property sheet drew a size.
//
// The forwarder moves to filedefext.cpp instead -- the TU that owns the function -- so exactly
// one definition exists before and after this flip. Rust reaches it the way drvdefext.rs:955
// already does, by importing the bridge name; there is nothing for this file to export.

#[no_mangle]
pub unsafe extern "system" fn ChkDskDlg_RustBridge(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    ChkDskDlg(hwndDlg, uMsg, wParam, lParam)
}
