//! `dll/win32/shell32/dialogs/drive.cpp`, translated and FLIPPED — the C is commented out of
//! the CMake source list and these definitions are what shell32 links. `SHFormatDrive` is the
//! one export; everything else is file-local and reached only through the dialog it drives.
//!
//! Substrate note, because this TU is the reason the island grew: the disk IOCTL surface
//! (`DeviceIoControl`, `GET_LENGTH_INFORMATION`, `DISK_GEOMETRY`, `PARTITION_INFORMATION{,_EX}`,
//! `PARTITION_STYLE_*`) was in NO island — `ioapiset.h` and `psdk/winioctl.h` were missing from
//! the shell32 allowlist in `tools/rustconv/gen_driver_types.sh`, so a translation had no layout
//! for the buffers it hands `DeviceIoControl` and no value for the codes it selects on. Adding
//! the two headers is the fix; the four `IOCTL_DISK_*` codes still have to be COMPUTED (they are
//! `CTL_CODE(...)` function-like macros, which bindgen cannot emit) and live in `prelude.rs`.
//!
//! A PAID artifact — do not delete it to tidy up.

// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_075DE2B906DB: [WCHAR; 2] = [0x000A, 0x0000];
pub static CPP_COM_WSTR_066EF8DA5776: [WCHAR; 5] = [0x0033, 0x0032, 0x0054, 0x0042, 0x0000];
pub static CPP_COM_WSTR_3EB1C24C873D: [WCHAR; 5] = [0x0033, 0x0032, 0x0047, 0x0042, 0x0000];
pub static CPP_COM_WSTR_41561F7DFB27: [WCHAR; 4] = [0x0034, 0x0047, 0x0042, 0x0000];
pub static CPP_COM_WSTR_4FBD8FD4AFB2: [WCHAR; 6] = [0x0032, 0x0035, 0x0036, 0x0054, 0x0042, 0x0000];
pub static CPP_COM_WSTR_543786EDFEED: [WCHAR; 5] = [0x0046, 0x0041, 0x0054, 0x0058, 0x0000];
pub static CPP_COM_WSTR_637507E33EFF: [WCHAR; 9] = [
    0x0031, 0x0047, 0x0042, 0x002F, 0x0033, 0x0032, 0x0047, 0x0042, 0x0000,
];
pub static CPP_COM_WSTR_7599AC666CC2: [WCHAR; 6] = [0x0042, 0x0074, 0x0072, 0x0046, 0x0053, 0x0000];
pub static CPP_COM_WSTR_7960487F3F18: [WCHAR; 5] = [0x0031, 0x0036, 0x0045, 0x0042, 0x0000];
pub static CPP_COM_WSTR_7B07D8ECA2BA: [WCHAR; 5] = [0x004E, 0x0054, 0x0046, 0x0053, 0x0000];
pub static CPP_COM_WSTR_8F502580DE9A: [WCHAR; 6] = [0x0046, 0x0041, 0x0054, 0x0031, 0x0036, 0x0000];
pub static CPP_COM_WSTR_A953DDDD78B1: [WCHAR; 4] = [0x0046, 0x0041, 0x0054, 0x0000];
pub static CPP_COM_WSTR_BB53EE05898F: [WCHAR; 6] = [0x0046, 0x0041, 0x0054, 0x0033, 0x0032, 0x0000];
pub static CPP_COM_WSTR_C8D83D0D9E56: [WCHAR; 5] = [0x0045, 0x0058, 0x0054, 0x0032, 0x0000];
pub static CPP_COM_WSTR_DBA59E42764C: [WCHAR; 9] = [
    0x004D, 0x0042, 0x0052, 0x0020, 0x0028, 0x0025, 0x0064, 0x0029, 0x0000,
];
pub static CPP_COM_WSTR_EF3F9C6228AC: [WCHAR; 10] = [
    0x0047, 0x0050, 0x0054, 0x0020, 0x0025, 0x0073, 0x0020, 0x0025, 0x0073, 0x0000,
];
pub static CPP_COM_WSTR_F3A549CDECCE: [WCHAR; 7] = [
    0x0020, 0x0028, 0x0025, 0x0063, 0x003A, 0x0029, 0x0000,
];

use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----

extern "system" {
    /// `drive.cpp:28` `extern BOOL IsDriveFloppyW(LPCWSTR)`. C++ linkage in
    /// `dialogs/drvdefext.cpp`, so it is reached through the `EXTERN_C ... WINAPI` bridge
    /// `CDrivesFolder.cpp:1500` already exports for `drives_folder.rs` and `drvdefext.rs`.
    #[link_name = "IsDriveFloppyW_RustBridge"]
    fn IsDriveFloppyW(pszDriveRoot: LPCWSTR) -> BOOL;

    /// shlwapi ordinal 16. RE-DECLARED here, shadowing the island's, so the thread procedure
    /// can be spelled `extern "system"` once instead of `#[cfg]`-duplicated per arch: the
    /// island types this parameter `LPTHREAD_START_ROUTINE`, which comes from `ntdll-sys` and
    /// so carries bindgen's LITERAL convention (`stdcall` on i386, `C` on 64-bit). Those are
    /// three spellings of one ABI but three distinct Rust fn-pointer types. Same reasoning as
    /// `prelude::DLGPROC`; the ABI is unchanged, only how it is written.
    fn SHCreateThread(
        pfnThreadProc: Option<unsafe extern "system" fn(pData: *mut c_void) -> DWORD>,
        pData: *mut c_void,
        flags: DWORD,
        pfnCallback: Option<unsafe extern "system" fn(pData: *mut c_void) -> DWORD>,
    ) -> BOOL;

    /// Same treatment, same reason: the island types `arg4` `DLGPROC`, which is `ntdll-sys`'s
    /// and so literally `stdcall` on i386. `prelude::DLGPROC` is the `extern "system"` spelling
    /// of the identical ABI, and declaring against it is what lets `FormatDriveDlg` exist once
    /// instead of once per arch.
    fn DialogBoxParamW(
        hInstance: HINSTANCE,
        lpTemplateName: LPCWSTR,
        hWndParent: HWND,
        lpDialogFunc: DLGPROC,
        dwInitParam: LPARAM,
    ) -> INT_PTR;
}

/// `drive.cpp:30-39`. FILE-LOCAL, so it is in no header and no island can carry it.
///
/// `Tip` is not an ordinary buffer: `CreateTipText` hands its address to `NtQueryObject` as an
/// `OBJECT_NAME_INFORMATION`, which is a `UNICODE_STRING` followed by the characters it points
/// at. The nested struct exists to give that overlay a name, so the field ORDER and the 400-char
/// tail are load-bearing rather than cosmetic.
#[repr(C)]
pub struct tagTip {
    pub Name: UNICODE_STRING,
    pub Buffer: [WCHAR; 400],
}

#[repr(C)]
pub struct FORMAT_DRIVE_CONTEXT {
    /// The drive NUMBER, not a drive letter — every use adds `'A'`/`'a'` first.
    pub Drive: WCHAR,
    pub Options: UINT,
    pub Result: UINT,
    pub bFormattingNow: BOOL,
    pub hWndMain: HWND,
    pub hWndTip: HWND,
    pub hWndTipTrigger: HWND,
    pub Tip: tagTip,
}
pub type PFORMAT_DRIVE_CONTEXT = *mut FORMAT_DRIVE_CONTEXT;

/// `drive.cpp:41-46` `static inline BOOL DevIoCtl(...)`.
#[inline]
unsafe fn DevIoCtl(
    hDevice: HANDLE,
    Code: UINT,
    pIn: LPVOID,
    cbIn: UINT,
    pOut: LPVOID,
    cbOut: UINT,
) -> BOOL {
    let mut cb: DWORD = 0;
    DeviceIoControl(
        hDevice,
        Code,
        pIn,
        cbIn,
        pOut,
        cbOut,
        core::ptr::addr_of_mut!(cb),
        core::ptr::null_mut(),
    )
}

/// `drive.cpp:48-51`.
unsafe fn IsFloppy(pszDrive: PCWSTR) -> BOOL {
    // C: `GetDriveTypeW(..) == DRIVE_REMOVABLE && IsDriveFloppyW(..)`. The right operand is a
    // BOOL, not a bool, so it needs the `!= 0` that `&&` supplies implicitly in C.
    (GetDriveTypeW(pszDrive) == DRIVE_REMOVABLE && IsDriveFloppyW(pszDrive) != 0) as BOOL
}

/// `drive.cpp:65-72`.
unsafe fn IsSystemDrive(pContext: PFORMAT_DRIVE_CONTEXT) -> BOOL {
    let mut wszSystemDrv: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
    wszSystemDrv[0] = 0; // UNICODE_NULL
    // `GetSystemDirectory` is the A/W macro; the DLL builds UNICODE, so it is the W entry point.
    GetSystemDirectoryW(wszSystemDrv.as_mut_ptr(), _countof!(wszSystemDrv));
    // Widened to 32 bits to match C's integer promotion: both operands are `int` there, so a
    // drive number near the top of the WCHAR range must not wrap before the comparison.
    ((wszSystemDrv[0] as UINT | 32) == (*pContext).Drive as UINT + 0x0061) as BOOL
}

/// `drive.cpp:74-80`.
unsafe fn OpenLogicalDriveHandle(DriveNumber: WORD) -> HANDLE {
    let szPath: [WCHAR; 7] = [
        0x005C,
        0x005C,
        0x003F,
        0x005C,
        DriveNumber.wrapping_add(0x0041),
        0x003A,
        0x0000,
    ];
    CreateFileW(
        szPath.as_ptr(),
        FILE_READ_DATA,
        FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
        core::ptr::null_mut(),
        OPEN_EXISTING,
        0,
        core::ptr::null_mut(),
    )
}

/// `drive.cpp:82-106`. C++ `ULARGE_INTEGER &Result` — the reference lowers to a pointer.
unsafe fn GetLogicalDriveSize(DriveNumber: WORD, Result: *mut ULARGE_INTEGER) -> BOOL {
    let mut bSuccess: BOOL = FALSE;
    let szDrivePath: [WCHAR; 4] = [DriveNumber.wrapping_add(0x0041), 0x003A, 0x005C, 0x0000];

    let hDevice: HANDLE = OpenLogicalDriveHandle(DriveNumber);
    if hDevice != INVALID_HANDLE_VALUE {
        let mut LengthInfo: GET_LENGTH_INFORMATION = core::mem::zeroed();
        bSuccess = DevIoCtl(
            hDevice,
            IOCTL_DISK_GET_LENGTH_INFO,
            core::ptr::null_mut(),
            0,
            core::ptr::addr_of_mut!(LengthInfo) as LPVOID,
            core::mem::size_of::<GET_LENGTH_INFORMATION>() as UINT,
        );
        (*Result).QuadPart = LengthInfo.Length.QuadPart as ULONGLONG;
        if bSuccess == 0 && GetDriveTypeW(szDrivePath.as_ptr()) == DRIVE_REMOVABLE {
            // Blank floppy
            let mut dg: DISK_GEOMETRY = core::mem::zeroed();
            bSuccess = DevIoCtl(
                hDevice,
                IOCTL_DISK_GET_DRIVE_GEOMETRY,
                core::ptr::null_mut(),
                0,
                core::ptr::addr_of_mut!(dg) as LPVOID,
                core::mem::size_of::<DISK_GEOMETRY>() as UINT,
            );
            if bSuccess != 0 {
                // C multiplies a LONGLONG by three DWORDs, so the whole product is 64-bit
                // SIGNED and only the store to `QuadPart` reinterprets it.
                (*Result).QuadPart = (dg.Cylinders.QuadPart
                    * dg.TracksPerCylinder as LONGLONG
                    * dg.SectorsPerTrack as LONGLONG
                    * dg.BytesPerSector as LONGLONG) as ULONGLONG;
            }
        }
        CloseHandle(hDevice);
    }
    if bSuccess == 0 || (*Result).QuadPart == 0 {
        // Note: Not exact if NTFS quotas are in effect
        bSuccess = GetDiskFreeSpaceExW(
            szDrivePath.as_ptr(),
            core::ptr::null_mut(),
            Result,
            core::ptr::null_mut(),
        );
    }
    bSuccess
}

/// `drive.cpp:108-149`. C++ `FORMAT_DRIVE_CONTEXT &Ctx`.
unsafe fn CreateTipText(Ctx: PFORMAT_DRIVE_CONTEXT) -> PWSTR {
    let hDevice = OpenLogicalDriveHandle((*Ctx).Drive);
    if hDevice == INVALID_HANDLE_VALUE {
        return core::ptr::null_mut();
    }

    let mut cb: ULONG = 0;
    ZeroMemory(
        core::ptr::addr_of_mut!((*Ctx).Tip),
        core::mem::size_of::<tagTip>(),
    );
    NtQueryObject(
        hDevice,
        ObjectNameInformation,
        core::ptr::addr_of_mut!((*Ctx).Tip) as PVOID,
        core::mem::size_of::<tagTip>() as ULONG,
        core::ptr::addr_of_mut!(cb),
    );
    if !(*Ctx).Tip.Name.Buffer.is_null() && *(*Ctx).Tip.Name.Buffer == 0x005C {
        StringCbCatW(
            (*Ctx).Tip.Name.Buffer,
            core::mem::size_of::<tagTip>() - core::mem::size_of::<UNICODE_STRING>(),
            CPP_COM_WSTR_075DE2B906DB.as_ptr(),
        );
    } else {
        // C: `(Ctx.Tip.Name.Buffer = Ctx.Tip.Buffer)[0] = UNICODE_NULL` — the assignment's
        // VALUE is what gets indexed, so the store lands in the buffer just pointed at.
        (*Ctx).Tip.Name.Buffer = (*Ctx).Tip.Buffer.as_mut_ptr();
        *(*Ctx).Tip.Name.Buffer = 0;
    }

    let mut pie: PARTITION_INFORMATION_EX = core::mem::zeroed();
    if DevIoCtl(
        hDevice,
        IOCTL_DISK_GET_PARTITION_INFO_EX,
        core::ptr::null_mut(),
        0,
        core::ptr::addr_of_mut!(pie) as LPVOID,
        core::mem::size_of::<PARTITION_INFORMATION_EX>() as UINT,
    ) == 0
    {
        pie.PartitionStyle = PARTITION_STYLE_RAW;
        let mut pi: PARTITION_INFORMATION = core::mem::zeroed();
        if DevIoCtl(
            hDevice,
            IOCTL_DISK_GET_PARTITION_INFO,
            core::ptr::null_mut(),
            0,
            core::ptr::addr_of_mut!(pi) as LPVOID,
            core::mem::size_of::<PARTITION_INFORMATION>() as UINT,
        ) != 0
        {
            pie.PartitionStyle = PARTITION_STYLE_MBR;
            pie.PartitionNumber = pi.PartitionNumber;
        }
    }
    CloseHandle(hDevice);

    let mut szBuf: [WCHAR; 150] = [0; 150];
    let mut szGuid: [WCHAR; 39] = [0; 39];
    let pszTip: PWSTR = (*Ctx).Tip.Name.Buffer;
    szBuf[0] = 0;
    if pie.PartitionStyle == PARTITION_STYLE_GPT {
        StringFromGUID2(
            core::ptr::addr_of!(pie.__bindgen_anon_1.Gpt.PartitionId).cast(),
            szGuid.as_mut_ptr(),
            _countof!(szGuid) as INT,
        );
        strsafe_rust::StringCchPrintfW!(
            szBuf.as_mut_ptr(),
            _countof!(szBuf) as usize,
            CPP_COM_WSTR_EF3F9C6228AC.as_ptr(),
            szGuid.as_ptr(),
            core::ptr::addr_of!(pie.__bindgen_anon_1.Gpt.Name) as PCWSTR,
        );
    }
    if pie.PartitionStyle == PARTITION_STYLE_MBR {
        strsafe_rust::StringCchPrintfW!(
            szBuf.as_mut_ptr(),
            _countof!(szBuf) as usize,
            CPP_COM_WSTR_DBA59E42764C.as_ptr(),
            pie.PartitionNumber,
        );
    }
    // C: `sizeof(Ctx.Tip) - sizeof(Ctx.Tip.Name)`, and `Name` IS a UNICODE_STRING — the same
    // quantity the first StringCbCatW above spells with the type.
    StringCbCatW(
        pszTip,
        core::mem::size_of::<tagTip>() - core::mem::size_of::<UNICODE_STRING>(),
        szBuf.as_ptr(),
    );
    pszTip
}

/// `drive.cpp:151-244`.
unsafe fn GetDefaultClusterSize(
    szFs: LPWSTR,
    pClusterSize: PDWORD,
    TotalNumberOfBytes: PULARGE_INTEGER,
) -> BOOL {
    let ClusterSize: DWORD;

    if _wcsicmp(szFs, CPP_COM_WSTR_8F502580DE9A.as_ptr()) == 0
        || _wcsicmp(szFs, CPP_COM_WSTR_A953DDDD78B1.as_ptr()) == 0
    // REACTOS HACK
    {
        if (*TotalNumberOfBytes).QuadPart <= (16 * 1024 * 1024) {
            ClusterSize = 2048;
        } else if (*TotalNumberOfBytes).QuadPart <= (32 * 1024 * 1024) {
            ClusterSize = 512;
        } else if (*TotalNumberOfBytes).QuadPart <= (64 * 1024 * 1024) {
            ClusterSize = 1024;
        } else if (*TotalNumberOfBytes).QuadPart <= (128 * 1024 * 1024) {
            ClusterSize = 2048;
        } else if (*TotalNumberOfBytes).QuadPart <= (256 * 1024 * 1024) {
            ClusterSize = 4096;
        } else if (*TotalNumberOfBytes).QuadPart <= (512 * 1024 * 1024) {
            ClusterSize = 8192;
        } else if (*TotalNumberOfBytes).QuadPart <= (1024 * 1024 * 1024) {
            ClusterSize = 16384;
        } else if (*TotalNumberOfBytes).QuadPart <= (2048u64 * 1024 * 1024) {
            ClusterSize = 32768;
        } else if (*TotalNumberOfBytes).QuadPart <= (4096u64 * 1024 * 1024) {
            ClusterSize = 8192;
        } else {
            return FALSE;
        }
    } else if _wcsicmp(szFs, CPP_COM_WSTR_BB53EE05898F.as_ptr()) == 0 {
        if (*TotalNumberOfBytes).QuadPart <= (64 * 1024 * 1024) {
            ClusterSize = 512;
        } else if (*TotalNumberOfBytes).QuadPart <= (128 * 1024 * 1024) {
            ClusterSize = 1024;
        } else if (*TotalNumberOfBytes).QuadPart <= (256 * 1024 * 1024) {
            ClusterSize = 2048;
        } else if (*TotalNumberOfBytes).QuadPart <= (8192u64 * 1024 * 1024) {
            ClusterSize = 2048;
        } else if (*TotalNumberOfBytes).QuadPart <= (16384u64 * 1024 * 1024) {
            ClusterSize = 8192;
        } else if (*TotalNumberOfBytes).QuadPart <= (32768u64 * 1024 * 1024) {
            ClusterSize = 16384;
        } else {
            return FALSE;
        }
    } else if _wcsicmp(szFs, CPP_COM_WSTR_543786EDFEED.as_ptr()) == 0 {
        if (*TotalNumberOfBytes).QuadPart <= (16 * 1024 * 1024) {
            ClusterSize = 2048;
        } else if (*TotalNumberOfBytes).QuadPart <= (32 * 1024 * 1024) {
            ClusterSize = 512;
        } else if (*TotalNumberOfBytes).QuadPart <= (64 * 1024 * 1024) {
            ClusterSize = 1024;
        } else if (*TotalNumberOfBytes).QuadPart <= (128 * 1024 * 1024) {
            ClusterSize = 2048;
        } else if (*TotalNumberOfBytes).QuadPart <= (256 * 1024 * 1024) {
            ClusterSize = 4096;
        } else if (*TotalNumberOfBytes).QuadPart <= (8192u64 * 1024 * 1024) {
            ClusterSize = 2048;
        } else if (*TotalNumberOfBytes).QuadPart <= (16384u64 * 1024 * 1024) {
            ClusterSize = 8192;
        } else if (*TotalNumberOfBytes).QuadPart <= (32768u64 * 1024 * 1024) {
            ClusterSize = 16384;
        } else {
            return FALSE;
        }
    } else if _wcsicmp(szFs, CPP_COM_WSTR_7B07D8ECA2BA.as_ptr()) == 0 {
        if (*TotalNumberOfBytes).QuadPart <= (512 * 1024 * 1024) {
            ClusterSize = 512;
        } else if (*TotalNumberOfBytes).QuadPart <= (1024 * 1024 * 1024) {
            ClusterSize = 1024;
        } else if (*TotalNumberOfBytes).QuadPart <= (2048u64 * 1024 * 1024) {
            ClusterSize = 2048;
        } else {
            ClusterSize = 2048;
        }
    } else if _wcsicmp(szFs, CPP_COM_WSTR_C8D83D0D9E56.as_ptr()) == 0 {
        // auto block size calculation
        ClusterSize = 0;
    } else if _wcsicmp(szFs, CPP_COM_WSTR_7599AC666CC2.as_ptr()) == 0 {
        // auto block size calculation
        ClusterSize = 0;
    } else {
        return FALSE;
    }

    *pClusterSize = ClusterSize;
    TRUE
}

/// `drive.cpp:246-358`.
unsafe fn InsertDefaultClusterSizeForFs(hwndDlg: HWND, pContext: PFORMAT_DRIVE_CONTEXT) {
    let mut wszBuf: [WCHAR; 100] = [0; 100];
    let mut wszDefaultSize: [WCHAR; 100] = [0; 100];
    let pwszFsSizeLimit: PCWSTR;
    let iSelIndex: INT;
    let mut TotalNumberOfBytes: ULARGE_INTEGER = core::mem::zeroed();
    let mut ClusterSize: DWORD = 0;
    let mut lIndex: LRESULT;
    let mut hDlgCtrl: HWND;

    hDlgCtrl = GetDlgItem(hwndDlg, 28677);
    iSelIndex = SendMessage(hDlgCtrl, CB_GETCURSEL, 0, 0) as INT;
    if iSelIndex == CB_ERR {
        return;
    }

    if SendMessageW(
        hDlgCtrl,
        CB_GETLBTEXT,
        iSelIndex as WPARAM,
        wszBuf.as_mut_ptr() as LPARAM,
    ) == CB_ERR as LRESULT
    {
        return;
    }

    if GetLogicalDriveSize((*pContext).Drive, core::ptr::addr_of_mut!(TotalNumberOfBytes)) == 0 {
        return;
    }

    if _wcsicmp(wszBuf.as_ptr(), CPP_COM_WSTR_8F502580DE9A.as_ptr()) == 0
        || _wcsicmp(wszBuf.as_ptr(), CPP_COM_WSTR_A953DDDD78B1.as_ptr()) == 0
    // REACTOS HACK
    {
        pwszFsSizeLimit = CPP_COM_WSTR_41561F7DFB27.as_ptr();
    } else if _wcsicmp(wszBuf.as_ptr(), CPP_COM_WSTR_BB53EE05898F.as_ptr()) == 0 {
        pwszFsSizeLimit = CPP_COM_WSTR_3EB1C24C873D.as_ptr();
    } else if _wcsicmp(wszBuf.as_ptr(), CPP_COM_WSTR_543786EDFEED.as_ptr()) == 0 {
        pwszFsSizeLimit = CPP_COM_WSTR_637507E33EFF.as_ptr();
    } else if _wcsicmp(wszBuf.as_ptr(), CPP_COM_WSTR_7B07D8ECA2BA.as_ptr()) == 0 {
        pwszFsSizeLimit = CPP_COM_WSTR_4FBD8FD4AFB2.as_ptr();
    } else if _wcsicmp(wszBuf.as_ptr(), CPP_COM_WSTR_C8D83D0D9E56.as_ptr()) == 0 {
        pwszFsSizeLimit = CPP_COM_WSTR_066EF8DA5776.as_ptr();
    } else {
        pwszFsSizeLimit = CPP_COM_WSTR_7960487F3F18.as_ptr();
    }
    // Only reaches the TRACE below; kept bound so the branch chain above is not dead code.
    let _ = pwszFsSizeLimit;

    if _wcsicmp(wszBuf.as_ptr(), CPP_COM_WSTR_8F502580DE9A.as_ptr()) == 0
        || _wcsicmp(wszBuf.as_ptr(), CPP_COM_WSTR_A953DDDD78B1.as_ptr()) == 0
        || _wcsicmp(wszBuf.as_ptr(), CPP_COM_WSTR_BB53EE05898F.as_ptr()) == 0
        || _wcsicmp(wszBuf.as_ptr(), CPP_COM_WSTR_543786EDFEED.as_ptr()) == 0
        || _wcsicmp(wszBuf.as_ptr(), CPP_COM_WSTR_7B07D8ECA2BA.as_ptr()) == 0
        || _wcsicmp(wszBuf.as_ptr(), CPP_COM_WSTR_C8D83D0D9E56.as_ptr()) == 0
        || _wcsicmp(wszBuf.as_ptr(), CPP_COM_WSTR_7599AC666CC2.as_ptr()) == 0
    {
        if GetDefaultClusterSize(
            wszBuf.as_mut_ptr(),
            core::ptr::addr_of_mut!(ClusterSize),
            core::ptr::addr_of_mut!(TotalNumberOfBytes),
        ) == 0
        {
            SendMessageW(hDlgCtrl, CB_DELETESTRING, iSelIndex as WPARAM, 0);
            return;
        }

        if LoadStringW(
            shell32_hInstance,
            IDS_DEFAULT_CLUSTER_SIZE,
            wszDefaultSize.as_mut_ptr(),
            _countof!(wszDefaultSize) as INT,
        ) != 0
        {
            // Get the window handle of "allocation unit size" combobox
            hDlgCtrl = GetDlgItem(hwndDlg, 28680);
            SendMessageW(hDlgCtrl, CB_RESETCONTENT, 0, 0);
            lIndex = SendMessageW(
                hDlgCtrl,
                CB_ADDSTRING,
                0,
                wszDefaultSize.as_ptr() as LPARAM,
            );
            if lIndex != CB_ERR as LRESULT {
                SendMessageW(
                    hDlgCtrl,
                    CB_SETITEMDATA,
                    lIndex as WPARAM,
                    ClusterSize as LPARAM,
                );
            }
            SendMessageW(hDlgCtrl, CB_SETCURSEL, 0, 0);
        }

        if _wcsicmp(wszBuf.as_ptr(), CPP_COM_WSTR_7B07D8ECA2BA.as_ptr()) == 0 {
            ClusterSize = 512;
            // C reuses ONE variable as the loop counter AND the CB_ADDSTRING result, so a
            // successful insert overwrites the induction variable. Preserved deliberately:
            // "fixing" it here would change how many entries the combo gets.
            lIndex = 0;
            while lIndex < 4 {
                TotalNumberOfBytes.QuadPart = ClusterSize as ULONGLONG;
                if !StrFormatByteSizeW(
                    TotalNumberOfBytes.QuadPart as LONGLONG,
                    wszDefaultSize.as_mut_ptr(),
                    _countof!(wszDefaultSize),
                )
                .is_null()
                {
                    lIndex = SendMessageW(
                        hDlgCtrl,
                        CB_ADDSTRING,
                        0,
                        wszDefaultSize.as_ptr() as LPARAM,
                    );
                    if lIndex != CB_ERR as LRESULT {
                        SendMessageW(
                            hDlgCtrl,
                            CB_SETITEMDATA,
                            lIndex as WPARAM,
                            ClusterSize as LPARAM,
                        );
                    }
                }
                ClusterSize *= 2;
                lIndex += 1;
            }
        }

        SendMessageW(GetDlgItem(hwndDlg, 28675), BM_SETCHECK, BST_UNCHECKED as WPARAM, 0);
        // The `#if 0` arm above this in the C would enable the button for the compressing
        // filesystems; with it disabled, only the else branch survives.
        /* Disable the "Enable Compression" button */
        EnableWindow(GetDlgItem(hwndDlg, 28675), FALSE);
    } else {
        SendDlgItemMessageW(hwndDlg, 28680, CB_RESETCONTENT, iSelIndex as WPARAM, 0);
    }
}

/// `drive.cpp:360-370`.
unsafe fn EnableFormatDriveDlgControls(hwndDlg: HWND, EnableState: INT) {
    let CanClose: BOOL = (EnableState != 0) as BOOL;
    let Enable: BOOL = (EnableState > 0) as BOOL;
    let hSysMenu: HMENU = GetSystemMenu(hwndDlg, FALSE);
    EnableMenuItem(
        hSysMenu,
        SC_CLOSE,
        MF_BYCOMMAND | if CanClose != 0 { MF_ENABLED } else { MF_GRAYED },
    );
    EnableWindow(GetDlgItem(hwndDlg, IDCANCEL as INT), CanClose);
    static id: [WORD; 6] = [IDOK as WORD, 28673, 28677, 28680, 28679, 28674];
    for i in 0..(_countof!(id) as usize) {
        EnableWindow(GetDlgItem(hwndDlg, id[i] as INT), Enable);
    }
}

/// `drive.cpp:372-472`.
unsafe fn InitializeFormatDriveDlg(hwndDlg: HWND, pContext: PFORMAT_DRIVE_CONTEXT) {
    let szDrive: [WCHAR; 4] = [
        (*pContext).Drive.wrapping_add(0x0041),
        0x003A,
        0x005C,
        0x0000,
    ];
    let mut szText: [WCHAR; 120] = [0; 120];
    let mut szFs: [WCHAR; 30] = [0; 30];
    let mut cchText: SIZE_T;
    let mut TotalNumberOfBytes: ULARGE_INTEGER = core::mem::zeroed();
    let mut dwIndex: DWORD;
    let mut dwDefault: DWORD;
    let mut uMinor: UCHAR = 0;
    let mut uMajor: UCHAR = 0;
    let mut Latest: BOOLEAN = 0;
    let hwndFileSystems: HWND;

    (*pContext).hWndMain = hwndDlg;
    (*pContext).hWndTipTrigger = GetDlgItem(hwndDlg, 30000);
    let mut tool: TTTOOLINFOW = core::mem::zeroed();
    tool.cbSize = core::mem::size_of::<TTTOOLINFOW>() as UINT;
    tool.hwnd = hwndDlg;
    tool.uFlags = TTF_SUBCLASS | TTF_IDISHWND | TTF_TRACK | TTF_ABSOLUTE;
    tool.uId = (*pContext).hWndTipTrigger as UINT_PTR;
    // commctrl.h `#define LPSTR_TEXTCALLBACKW ((LPWSTR)-1L)` — a CAST macro, so bindgen has
    // nothing to bind and it has to be written out.
    tool.lpszText = -1isize as LPWSTR;
    (*pContext).hWndTip = CreateWindowExW(
        WS_EX_TOPMOST,
        TOOLTIPS_CLASSW.as_ptr() as LPCWSTR,
        core::ptr::null_mut(),
        WS_POPUP | TTS_BALLOON | TTS_NOPREFIX | TTS_ALWAYSTIP,
        0,
        0,
        0,
        0,
        hwndDlg,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    SendMessageW(
        (*pContext).hWndTip,
        TTM_ADDTOOLW,
        0,
        core::ptr::addr_of!(tool) as LPARAM,
    );
    let nIcoSize: UINT = GetSystemMetrics(SM_CXSMICON as INT) as UINT;
    let hIco: HICON = LoadImageW(
        shell32_hInstance,
        MAKEINTRESOURCEW(IDI_SHELL_IDEA),
        IMAGE_ICON,
        nIcoSize as INT,
        nIcoSize as INT,
        LR_SHARED,
    ) as HICON;
    // C: `SendMessageW(.., STM_SETICON, (WPARAM)hIco, 0)`. The icon travels in wPARAM — an
    // earlier pass had the two swapped, which sets no icon and passes 0 as the handle.
    SendMessageW(
        (*pContext).hWndTipTrigger,
        STM_SETICON,
        hIco as WPARAM,
        0,
    );

    // `cchText` keeps C's `SIZE_T` — pointer-width, so it is `u32` on i386 and `u64` elsewhere.
    // Every index and `.add()` therefore needs `as usize`; widening the variable to `usize`
    // instead would silently change the arithmetic the subtractions below do on i386.
    cchText = GetWindowTextW(hwndDlg, szText.as_mut_ptr(), _countof!(szText) as INT - 1) as SIZE_T;
    szText[cchText as usize] = 0x0020;
    cchText += 1;
    szFs[0] = 0;
    if GetVolumeInformationW(
        szDrive.as_ptr(),
        szText.as_mut_ptr().add(cchText as usize),
        (_countof!(szText) as SIZE_T - cchText) as DWORD,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        szFs.as_mut_ptr(),
        _countof!(szFs),
    ) != 0
    {
        if szText[cchText as usize] == 0 {
            /* load default volume label */
            cchText += LoadStringW(
                shell32_hInstance,
                IDS_DRIVE_FIXED,
                szText.as_mut_ptr().add(cchText as usize),
                _countof!(szText) as INT - cchText as INT,
            ) as SIZE_T;
        } else {
            /* set volume label */
            SetDlgItemTextW(hwndDlg, 28679, szText.as_ptr().add(cchText as usize));
            cchText += wcslen(szText.as_ptr().add(cchText as usize)) as SIZE_T;
        }
    }

    strsafe_rust::StringCchPrintfW!(
        szText.as_mut_ptr().add(cchText as usize),
        _countof!(szText) as usize - cchText as usize,
        CPP_COM_WSTR_F3A549CDECCE.as_ptr(),
        // `%c` in a wide printf takes a promoted int, which is also what C's default argument
        // promotion does to the WCHAR.
        szDrive[0] as UINT,
    );

    /* set window text */
    SetWindowTextW(hwndDlg, szText.as_ptr());

    if GetLogicalDriveSize((*pContext).Drive, core::ptr::addr_of_mut!(TotalNumberOfBytes)) != 0 {
        if !StrFormatByteSizeW(
            TotalNumberOfBytes.QuadPart as LONGLONG,
            szText.as_mut_ptr(),
            _countof!(szText),
        )
        .is_null()
        {
            /* add drive capacity */
            SendDlgItemMessageW(hwndDlg, 28673, CB_ADDSTRING, 0, szText.as_ptr() as LPARAM);
            SendDlgItemMessageW(hwndDlg, 28673, CB_SETCURSEL, 0, 0);
        }
    } else {
        /* No known size, don't allow format (no partition or no floppy) */
        EnableFormatDriveDlgControls(hwndDlg, -1);
    }

    if ((*pContext).Options & SHFMT_OPT_FULL) != 0 {
        /* check quick format button */
        SendDlgItemMessageW(hwndDlg, 28674, BM_SETCHECK, BST_CHECKED as WPARAM, 0);
    }

    /* enumerate all available filesystems */
    dwIndex = 0;
    dwDefault = 0;
    hwndFileSystems = GetDlgItem(hwndDlg, 28677);

    let mut iForceDefault: INT = -1;
    while QueryAvailableFileSystemFormat(
        dwIndex,
        szText.as_mut_ptr(),
        core::ptr::addr_of_mut!(uMajor),
        core::ptr::addr_of_mut!(uMinor),
        core::ptr::addr_of_mut!(Latest),
    ) != 0
    {
        if _wcsicmp(szText.as_ptr(), szFs.as_ptr()) == 0 {
            /* default to the same filesystem */
            dwDefault = dwIndex;
            iForceDefault = dwIndex as INT;
        }

        if iForceDefault < 0
            && _wcsicmp(szText.as_ptr(), CPP_COM_WSTR_7B07D8ECA2BA.as_ptr()) == 0
            && IsFloppy(szDrive.as_ptr()) == 0
        {
            dwDefault = dwIndex;
        }

        SendMessageW(
            hwndFileSystems,
            CB_ADDSTRING,
            0,
            szText.as_ptr() as LPARAM,
        );
        dwIndex += 1;
    }

    if dwIndex == 0 {
        // ERR("no filesystem providers\n");
        return;
    }

    /* select default filesys */
    SendMessageW(hwndFileSystems, CB_SETCURSEL, dwDefault as WPARAM, 0);
    /* setup cluster combo */
    InsertDefaultClusterSizeForFs(hwndDlg, pContext);
}

/// `drive.cpp:474-479` `static inline PFORMAT_DRIVE_CONTEXT GetFormatContext()`.
///
/// FormatEx does not allow us to specify a context parameter so we have to store it in the
/// thread.
#[inline]
unsafe fn GetFormatContext() -> PFORMAT_DRIVE_CONTEXT {
    (*NtCurrentTeb()).NtTib.ArbitraryUserPointer as PFORMAT_DRIVE_CONTEXT
}

/// `drive.cpp:481-521`.
///
/// The `match` arms name the island's `CALLBACKCOMMAND_*` constants, NOT the bare `PROGRESS` /
/// `DONE` / `VOLUMEINUSE` the enumerators have in C. bindgen prefixes every member of a C enum
/// with the enum's own name, so a bare `PROGRESS` in a pattern position is not a constant at
/// all — it is an irrefutable BINDING that matches every command, which would run the progress
/// arm for `DONE` and dereference a `PBOOLEAN` as a `PDWORD`. Note also that the ordinals are
/// not what the C order suggests: `DONE` is 11, and `DONEWITHSTRUCTURE` is the 1.
unsafe extern "system" fn FormatExCB(
    Command: CALLBACKCOMMAND,
    _SubAction: ULONG,
    ActionInfo: PVOID,
) -> BOOLEAN {
    let pCtx = GetFormatContext();
    let szDrive: [WCHAR; 4] = [(*pCtx).Drive.wrapping_add(0x0041), 0x003A, 0x005C, 0x0000];
    let mut szLabel: [WCHAR; 40] = [0; 40];
    let Progress: PDWORD;
    let pSuccess: PBOOLEAN;
    match Command {
        CALLBACKCOMMAND_PROGRESS => {
            Progress = ActionInfo as PDWORD;
            SendDlgItemMessageW(
                (*pCtx).hWndMain,
                28678,
                PBM_SETPOS,
                *Progress as WPARAM,
                0,
            );
        }
        CALLBACKCOMMAND_DONE => {
            pSuccess = ActionInfo as PBOOLEAN;
            (*pCtx).Result = *pSuccess as UINT;
            SendDlgItemMessageW(
                (*pCtx).hWndMain,
                28679,
                WM_GETTEXT,
                _countof!(szLabel) as WPARAM,
                szLabel.as_mut_ptr() as LPARAM,
            );
            SetVolumeLabelW(
                szDrive.as_ptr(),
                if szLabel[0] != 0 {
                    szLabel.as_ptr()
                } else {
                    core::ptr::null()
                },
            );
            // precomp.h:63 is `#define ShellMessageBoxW ShellMessageBoxWrapW`, so drive.cpp's
            // three calls never reached the export of that name — shell32 does not import it,
            // and the island's declaration links to nothing (i386: `_ShellMessageBoxW`,
            // undefined). Same trap shlfileop.cpp carried.
            ShellMessageBoxWrapW(
                shell32_hInstance,
                (*pCtx).hWndMain,
                MAKEINTRESOURCEW(IDS_FORMAT_COMPLETE),
                MAKEINTRESOURCEW(IDS_FORMAT_TITLE),
                MB_OK | MB_ICONINFORMATION,
            );
            SendDlgItemMessageW((*pCtx).hWndMain, 28678, PBM_SETPOS, 0, 0);
        }
        CALLBACKCOMMAND_VOLUMEINUSE
        | CALLBACKCOMMAND_INSUFFICIENTRIGHTS
        | CALLBACKCOMMAND_FSNOTSUPPORTED
        | CALLBACKCOMMAND_CLUSTERSIZETOOSMALL => {
            (*pCtx).Result = FALSE as UINT;
            // FIXME("Unsupported command in FormatExCB\n");
        }
        _ => {}
    }

    TRUE as BOOLEAN
}

/// `drive.cpp:523-641`.
unsafe fn FormatDrive(hwndDlg: HWND, pContext: PFORMAT_DRIVE_CONTEXT) {
    let szDrive: [WCHAR; 4] = [
        (*pContext).Drive.wrapping_add(0x0041),
        0x003A,
        0x005C,
        0x0000,
    ];
    let mut szFileSys: [WCHAR; 40] = [0; 40];
    let mut szLabel: [WCHAR; 40] = [0; 40];
    let mut iSelIndex: INT;
    let mut Length: UINT;
    let mut hDlgCtrl: HWND;
    let QuickFormat: BOOL;
    let ClusterSize: DWORD;
    let DriveType: DWORD;
    let MediaFlag: FMIFS_MEDIA_FLAG;

    /* get filesystem */
    hDlgCtrl = GetDlgItem(hwndDlg, 28677);
    iSelIndex = SendMessageW(hDlgCtrl, CB_GETCURSEL, 0, 0) as INT;
    if iSelIndex == CB_ERR {
        // ERR("Unable to get file system selection\n");
        return;
    }
    Length = SendMessageW(hDlgCtrl, CB_GETLBTEXTLEN, iSelIndex as WPARAM, 0) as UINT;
    if Length as INT == CB_ERR || Length + 1 > _countof!(szFileSys) {
        // ERR("Unable to get file system selection\n");
        return;
    }

    /* retrieve the file system */
    SendMessageW(
        hDlgCtrl,
        CB_GETLBTEXT,
        iSelIndex as WPARAM,
        szFileSys.as_mut_ptr() as LPARAM,
    );
    szFileSys[_countof!(szFileSys) as usize - 1] = 0;

    /* retrieve the volume label */
    // NOTE: `GetWindow`, not `GetDlgItem` — that is what the C says, and 28679 is not a
    // GW_* command, so this returns the dialog's first sibling in that direction rather
    // than the label edit. Upstream bug, preserved: changing it changes behaviour.
    hDlgCtrl = GetWindow(hwndDlg, 28679);
    Length = SendMessageW(hDlgCtrl, WM_GETTEXTLENGTH, 0, 0) as UINT;
    if Length + 1 > _countof!(szLabel) {
        // ERR("Unable to get volume label\n");
        return;
    }
    SendMessageW(
        hDlgCtrl,
        WM_GETTEXT,
        _countof!(szLabel) as WPARAM,
        szLabel.as_mut_ptr() as LPARAM,
    );
    szLabel[(core::mem::size_of_val(&szLabel) / core::mem::size_of::<WCHAR>()) - 1] = 0;

    /* check for quickformat */
    if SendDlgItemMessageW(hwndDlg, 28674, BM_GETCHECK, 0, 0) == BST_CHECKED as LRESULT {
        QuickFormat = TRUE;
    } else {
        QuickFormat = FALSE;
    }

    /* get the cluster size */
    hDlgCtrl = GetDlgItem(hwndDlg, 28680);
    iSelIndex = SendMessageW(hDlgCtrl, CB_GETCURSEL, 0, 0) as INT;
    if iSelIndex == CB_ERR {
        // FIXME("\n");
        return;
    }
    ClusterSize = SendMessageW(hDlgCtrl, CB_GETITEMDATA, iSelIndex as WPARAM, 0) as DWORD;
    if ClusterSize as INT == CB_ERR {
        // FIXME("\n");
        return;
    }

    hDlgCtrl = GetDlgItem(hwndDlg, 28680);
    SendMessageW(hDlgCtrl, PBM_SETRANGE, 0, MAKELONG!(0, 100) as LPARAM);

    /* See if the drive is removable or not */
    DriveType = GetDriveTypeW(szDrive.as_ptr());
    match DriveType {
        DRIVE_UNKNOWN | DRIVE_REMOTE | DRIVE_CDROM | DRIVE_NO_ROOT_DIR => {
            // FIXME("\n");
            return;
        }
        DRIVE_REMOVABLE => {
            MediaFlag = FMIFS_MEDIA_FLAG_FMIFS_FLOPPY;
        }
        DRIVE_FIXED | DRIVE_RAMDISK => {
            MediaFlag = FMIFS_MEDIA_FLAG_FMIFS_HARDDISK;
        }
        _ => {
            MediaFlag = FMIFS_MEDIA_FLAG_FMIFS_HARDDISK;
        }
    }

    /* Format the drive */
    (*pContext).Result = FALSE as UINT;
    let tib: *mut NT_TIB = core::ptr::addr_of_mut!((*NtCurrentTeb()).NtTib);
    let BakArbitraryUserPointer: PVOID = (*tib).ArbitraryUserPointer;
    (*tib).ArbitraryUserPointer = pContext as PVOID;

    FormatEx(
        szDrive.as_ptr() as PWCHAR,
        MediaFlag,
        szFileSys.as_mut_ptr(),
        szLabel.as_mut_ptr(),
        // fmifs types this BOOLEAN, so the BOOL narrows on the way in exactly as it does in C.
        QuickFormat as BOOLEAN,
        ClusterSize,
        Some(FormatExCB),
    );

    (*tib).ArbitraryUserPointer = BakArbitraryUserPointer;
    if (*pContext).Result == 0 {
        (*pContext).Result = SHFMT_ERROR;
    } else if QuickFormat != 0 {
        (*pContext).Result = SHFMT_OPT_FULL;
    } else {
        (*pContext).Result = FALSE as UINT;
    }
}

/// `drive.cpp:645-668` `static DWORD CALLBACK FormatDriveThread(PVOID)`.
///
/// `CALLBACK` is `__stdcall`, so this MUST carry a calling convention — it is handed to
/// `SHCreateThread`, which calls it from another thread. A plain `unsafe fn` is Rust's own
/// convention and on i386 the two disagree about who pops the argument, so the callee returns
/// to a shifted stack.
unsafe extern "system" fn FormatDriveThread(pThreadParameter: *mut c_void) -> DWORD {
    let pContext: PFORMAT_DRIVE_CONTEXT = pThreadParameter as PFORMAT_DRIVE_CONTEXT;
    let hwndDlg: HWND = (*pContext).hWndMain;
    let szDrive: [WCHAR; 4] = [
        (*pContext).Drive.wrapping_add(0x0041),
        0x003A,
        0x005C,
        0x0000,
    ];

    /* Disable controls during format */
    EnableFormatDriveDlgControls(hwndDlg, FALSE);

    SHChangeNotify(
        SHCNE_MEDIAREMOVED as LONG,
        SHCNF_PATHW | SHCNF_FLUSH,
        szDrive.as_ptr() as LPCVOID,
        core::ptr::null(),
    );

    FormatDrive(hwndDlg, pContext);

    if (*pContext).Result != SHFMT_ERROR {
        SHChangeNotify(
            SHCNE_MEDIAINSERTED as LONG,
            SHCNF_PATHW,
            szDrive.as_ptr() as LPCVOID,
            core::ptr::null(),
        );
    }
    SHChangeNotify(
        SHCNE_UPDATEITEM as LONG,
        SHCNF_PATHW,
        szDrive.as_ptr() as LPCVOID,
        core::ptr::null(),
    );

    /* Re-enable controls after format */
    EnableFormatDriveDlgControls(hwndDlg, TRUE);
    (*pContext).bFormattingNow = FALSE;

    0
}

/// `drive.cpp:670-754`.
///
/// Hand-written from the C rather than taken from the paid run: the only artifact for this
/// function is a `.partial` whose seal placeholders were never resolved, and re-deriving the
/// CALL map without the exact macro-authority set of that run risks silently renaming a callee.
unsafe extern "system" fn FormatDriveDlg(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let pContext: PFORMAT_DRIVE_CONTEXT =
        GetWindowLongPtrW(hwndDlg, DWLP_USER) as PFORMAT_DRIVE_CONTEXT;

    match uMsg {
        WM_INITDIALOG => {
            SetWindowLongPtrW(hwndDlg, DWLP_USER, lParam as LONG_PTR);
            InitializeFormatDriveDlg(hwndDlg, lParam as PFORMAT_DRIVE_CONTEXT);
            return TRUE as INT_PTR;
        }
        WM_COMMAND => {
            // The labelled block is C's inner `switch`: every `break` in it leaves the switch
            // and falls straight through to the outer `break`, i.e. to `return FALSE`.
            'inner: {
                match LOWORD(wParam as DWORD) as UINT {
                    IDOK => {
                        if (*pContext).bFormattingNow != 0 {
                            break 'inner;
                        }

                        if ShellMessageBoxWrapW(
                            shell32_hInstance,
                            hwndDlg,
                            MAKEINTRESOURCEW(IDS_FORMAT_WARNING),
                            MAKEINTRESOURCEW(IDS_FORMAT_TITLE),
                            MB_OKCANCEL | MB_ICONWARNING,
                        ) == IDOK as INT
                        {
                            (*pContext).bFormattingNow = TRUE;
                            SHCreateThread(
                                Some(FormatDriveThread),
                                pContext as *mut c_void,
                                CTF_COINIT | CTF_PROCESS_REF | CTF_FREELIBANDEXIT,
                                None,
                            );
                        }
                    }
                    IDCANCEL => {
                        if (*pContext).bFormattingNow != 0 {
                            break 'inner;
                        }

                        EndDialog(hwndDlg, (*pContext).Result as INT_PTR);
                    }
                    // filesystem combo
                    28677 => {
                        if HIWORD(wParam as DWORD) as UINT == CBN_SELENDOK {
                            if (*pContext).bFormattingNow != 0 {
                                break 'inner;
                            }

                            InsertDefaultClusterSizeForFs(hwndDlg, pContext);
                        }
                    }
                    _ => {}
                }
            }
        }
        WM_NOTIFY => {
            // C++ `NMTTDISPINFO &ttdi = *(NMTTDISPINFO*)lParam;`
            let ttdi: *mut NMTTDISPINFOW = lParam as *mut NMTTDISPINFOW;
            if (*ttdi).hdr.code == TTN_NEEDTEXTW as UINT {
                (*ttdi).uFlags |= TTF_DI_SETITEM;
                let pszTip: PWSTR = CreateTipText(pContext);
                if !pszTip.is_null() {
                    (*ttdi).lpszText = pszTip;
                }
            }
        }
        WM_NEXTDLGCTL => {
            // Delay our action until after focus has changed
            PostMessage(hwndDlg, WM_APP, TRUE as WPARAM, 0);
        }
        // Show/Hide tip if tabbed to the info icon
        WM_APP => {
            let mut tool: TTTOOLINFOW = core::mem::zeroed();
            tool.cbSize = core::mem::size_of::<TTTOOLINFOW>() as UINT;
            tool.hwnd = hwndDlg;
            tool.uId = (*pContext).hWndTipTrigger as UINT_PTR;
            if wParam != 0 && GetFocus() == (*pContext).hWndTipTrigger {
                let mut r: RECT = core::mem::zeroed();
                GetWindowRect((*pContext).hWndTipTrigger, core::ptr::addr_of_mut!(r));
                r.left += (r.right - r.left) / 2;
                r.top += (r.bottom - r.top) / 2;
                SendMessageW(
                    (*pContext).hWndTip,
                    TTM_TRACKPOSITION,
                    0,
                    MAKELONG!(r.left, r.top) as LPARAM,
                );
                SendMessageW(
                    (*pContext).hWndTip,
                    TTM_TRACKACTIVATE,
                    TRUE as WPARAM,
                    core::ptr::addr_of!(tool) as LPARAM,
                );
            } else {
                SendMessageW(
                    (*pContext).hWndTip,
                    TTM_TRACKACTIVATE,
                    FALSE as WPARAM,
                    core::ptr::addr_of!(tool) as LPARAM,
                );
            }
        }
        WM_MOVING => {
            SendMessage(hwndDlg, WM_APP, FALSE as WPARAM, 0);
        }
        _ => {}
    }
    FALSE as INT_PTR
}

/*************************************************************************
 *              SHFormatDrive (SHELL32.@)
 */
#[no_mangle]
pub unsafe extern "system" fn SHFormatDrive(
    hwnd: HWND,
    drive: UINT,
    _fmtID: UINT,
    options: UINT,
) -> DWORD {
    let mut Context: FORMAT_DRIVE_CONTEXT = core::mem::zeroed();
    let result: INT;

    Context.Drive = drive as WCHAR;
    Context.Options = options;
    Context.Result = FALSE as UINT;
    Context.bFormattingNow = FALSE;

    if IsSystemDrive(core::ptr::addr_of_mut!(Context)) == 0 {
        result = DialogBoxParamW(
            shell32_hInstance,
            MAKEINTRESOURCEW(IDD_FORMAT_DRIVE),
            hwnd,
            Some(FormatDriveDlg),
            core::ptr::addr_of_mut!(Context) as LPARAM,
        ) as INT;
    } else {
        result = SHFMT_ERROR as INT;
        ShellMessageBoxWrapW(
            shell32_hInstance,
            hwnd,
            MAKEINTRESOURCEW(IDS_NO_FORMAT),
            MAKEINTRESOURCEW(IDS_NO_FORMAT_TITLE),
            MB_OK | MB_ICONWARNING,
        );
        // TRACE("SHFormatDrive(): The provided drive for format is a system volume! ...");
    }

    result as DWORD
}
