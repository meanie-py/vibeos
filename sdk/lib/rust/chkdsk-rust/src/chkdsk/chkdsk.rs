// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_075DE2B906DB: [u16; 2] = [0x000Au16, 0x0000];
pub static RUSTCONV_WSTR_102027A5A8A8: [u16; 10] = [0x0055u16, 0x004E, 0x004B, 0x004E, 0x004F, 0x0057, 0x004E, 0x0044, 0x000A, 0x0000];
pub static RUSTCONV_WSTR_1C2203FC1A00: [u16; 16] = [
    0x0046u16, 0x0053, 0x004E, 0x004F, 0x0054, 0x0053, 0x0055, 0x0050, 0x0050, 0x004F,
    0x0052, 0x0054, 0x0045, 0x0044, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_26554534B697: [u16; 19] = [
    0x0044u16, 0x004F, 0x004E, 0x0045, 0x0057, 0x0049, 0x0054, 0x0048, 0x0053, 0x0054,
    0x0052, 0x0055, 0x0043, 0x0054, 0x0055, 0x0052, 0x0045, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_4A34C150559F: [u16; 21] = [
    0x0043u16, 0x004C, 0x0055, 0x0053, 0x0054, 0x0045, 0x0052, 0x0053, 0x0049, 0x005A,
    0x0045, 0x0054, 0x004F, 0x004F, 0x0053, 0x004D, 0x0041, 0x004C, 0x004C, 0x000A,
    0x0000,
];
pub static RUSTCONV_WSTR_5132838EC735: [u16; 10] = [0x0055u16, 0x004E, 0x004B, 0x004E, 0x004F, 0x0057, 0x004E, 0x0043, 0x000A, 0x0000];
pub static RUSTCONV_WSTR_5815EC76BB90: [u16; 10] = [0x0055u16, 0x004E, 0x004B, 0x004E, 0x004F, 0x0057, 0x004E, 0x0034, 0x000A, 0x0000];
pub static RUSTCONV_WSTR_5A4B4B9582E5: [u16; 10] = [0x0055u16, 0x004E, 0x004B, 0x004E, 0x004F, 0x0057, 0x004E, 0x0039, 0x000A, 0x0000];
pub static RUSTCONV_WSTR_633E530910CF: [u16; 10] = [0x0055u16, 0x004E, 0x004B, 0x004E, 0x004F, 0x0057, 0x004E, 0x0035, 0x000A, 0x0000];
pub static RUSTCONV_WSTR_815A3FD323C9: [u16; 10] = [0x0055u16, 0x004E, 0x004B, 0x004E, 0x004F, 0x0057, 0x004E, 0x0041, 0x000A, 0x0000];
pub static RUSTCONV_WSTR_BE193C67CE83: [u16; 3] = [0x0025u16, 0x0053, 0x0000];
pub static RUSTCONV_WSTR_C0FE14364DC7: [u16; 20] = [
    0x0049u16, 0x004E, 0x0053, 0x0055, 0x0046, 0x0046, 0x0049, 0x0043, 0x0049, 0x0045,
    0x004E, 0x0054, 0x0052, 0x0049, 0x0047, 0x0048, 0x0054, 0x0053, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_D52366BE8A52: [u16; 10] = [0x0055u16, 0x004E, 0x004B, 0x004E, 0x004F, 0x0057, 0x004E, 0x0033, 0x000A, 0x0000];
pub static RUSTCONV_WSTR_E2D5D44565C5: [u16; 10] = [0x0055u16, 0x004E, 0x004B, 0x004E, 0x004F, 0x0057, 0x004E, 0x0032, 0x000D, 0x0000];
pub static RUSTCONV_WSTR_F37F62BC0070: [u16; 8] = [0x005Cu16, 0x005C, 0x002E, 0x005C, 0x0025, 0x0043, 0x003A, 0x0000];
pub static RUSTCONV_WSTR_FC4B443CE21B: [u16; 19] = [
    0x0053u16, 0x0054, 0x0052, 0x0055, 0x0043, 0x0054, 0x0055, 0x0052, 0x0045, 0x0050,
    0x0052, 0x004F, 0x0047, 0x0052, 0x0045, 0x0053, 0x0053, 0x000A, 0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut Error: BOOL = (FALSE as i32);

#[no_mangle]
pub static mut FixErrors: BOOL = (FALSE as i32);

#[no_mangle]
pub static mut SkipClean: BOOL = (FALSE as i32);

#[no_mangle]
pub static mut ScanSectors: BOOL = (FALSE as i32);

#[no_mangle]
pub static mut Verbose: BOOL = (FALSE as i32);

#[no_mangle]
pub static mut Drive: PWCHAR = ::core::ptr::null::<WCHAR>() as *mut WCHAR;

#[no_mangle]
pub static mut CurrentDirectory: [WCHAR; 1024] = [0; 1024];

pub const WIN32_NO_STATUS: () = ();

pub const NTOS_MODE_USER: () = ();

pub const FMIFS_IMPORT_DLL: () = ();

unsafe extern "C" fn PrintWin32Error(Message: UINT, ErrorCode: DWORD) {
    ConResPuts(core::ptr::addr_of_mut!(csStdErr), Message);
    ConMsgPuts(
        core::ptr::addr_of_mut!(csStdErr),
        FORMAT_MESSAGE_FROM_SYSTEM as DWORD,
        core::ptr::null_mut::<core::ffi::c_void>(),
        ErrorCode,
        LANG_USER_DEFAULT as DWORD,
    );
    ConPuts(core::ptr::addr_of_mut!(csStdErr), RUSTCONV_WSTR_075DE2B906DB.as_ptr());
}

#[no_mangle]
pub unsafe extern "system" fn CtrlCIntercept(dwCtrlType: DWORD) -> BOOL {
    (TRUE as i32)
}

unsafe extern "C" fn Usage(mut ProgramName: PWCHAR) {
    ConResPrintf(&raw mut csStdOut, IDS_USAGE as UINT, ProgramName);
}

unsafe extern "C" fn ParseCommandLine(
    mut argc: ::core::ffi::c_int,
    mut argv: *mut *mut WCHAR,
) -> ::core::ffi::c_int {
    let mut i: ::core::ffi::c_int = 0;
    let mut gotFix: BOOLEAN = (FALSE as u8);
    let mut gotVerbose: BOOLEAN = (FALSE as u8);
    let mut gotClean: BOOLEAN = (FALSE as u8);
    // let mut gotScan: BOOLEAN = FALSE;
    i = 1;
    while i < argc {
        match *(*argv.offset(i as isize)).offset(0 as isize)
        {
            0x002Du16 | 0x002Fu16 => {
                match *(*argv.offset(i as isize)).offset(1 as isize)
                {
                    0x0046u16 | 0x0066u16 => {
                        if gotFix != 0 {
                            return i;
                        }
                        FixErrors = (TRUE as i32);
                        gotFix = (TRUE as u8);
                    }
                    0x0056u16 | 0x0076u16 => {
                        if gotVerbose != 0 {
                            return i;
                        }
                        Verbose = (TRUE as i32);
                        gotVerbose = (TRUE as u8);
                    }
                    0x0052u16 | 0x0072u16 => {
                        if gotFix != 0 {
                            return i;
                        }
                        ScanSectors = (TRUE as i32);
                        gotFix = (TRUE as u8);
                    }
                    0x0043u16 | 0x0063u16 => {
                        if gotClean != 0 {
                            return i;
                        }
                        SkipClean = (TRUE as i32);
                        gotClean = (TRUE as u8);
                    }
                    _ => return i,
                }
            }
            _ => {
                if !Drive.is_null() {
                    return i;
                }
                if *(*argv.offset(i as isize)).offset(1 as isize)
                    != 0x003Au16
                {
                    return i;
                }
                Drive = *argv.offset(i as isize) as PWCHAR;
            }
        }
        i += 1;
    }
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn ChkdskCallback(
    Command: CALLBACKCOMMAND,
    Modifier: DWORD,
    Argument: PVOID,
) -> BOOLEAN {
    let mut Ret: BOOLEAN;
    let mut percent: PDWORD;
    let mut status: PBOOLEAN;
    let mut output: PTEXTOUTPUT;

    //
    // We get other types of commands,
    // but we don't have to pay attention to them
    //
    Ret = (TRUE as u8);
    match Command {
        UNKNOWN2 => {
            ConPuts(StdOut, RUSTCONV_WSTR_E2D5D44565C5.as_ptr());
        }

        UNKNOWN3 => {
            ConPuts(StdOut, RUSTCONV_WSTR_D52366BE8A52.as_ptr());
        }

        UNKNOWN4 => {
            ConPuts(StdOut, RUSTCONV_WSTR_5815EC76BB90.as_ptr());
        }

        UNKNOWN5 => {
            ConPuts(StdOut, RUSTCONV_WSTR_633E530910CF.as_ptr());
        }

        FSNOTSUPPORTED => {
            ConPuts(StdOut, RUSTCONV_WSTR_1C2203FC1A00.as_ptr());
        }

        VOLUMEINUSE => {
            ConResPuts(StdOut, IDS_VOLUME_IN_USE);
            Ret = (FALSE as u8);
        }

        UNKNOWN9 => {
            ConPuts(StdOut, RUSTCONV_WSTR_5A4B4B9582E5.as_ptr());
        }

        UNKNOWNA => {
            ConPuts(StdOut, RUSTCONV_WSTR_815A3FD323C9.as_ptr());
        }

        UNKNOWNC => {
            ConPuts(StdOut, RUSTCONV_WSTR_5132838EC735.as_ptr());
        }

        UNKNOWND => {
            ConPuts(StdOut, RUSTCONV_WSTR_102027A5A8A8.as_ptr());
        }

        INSUFFICIENTRIGHTS => {
            ConPuts(StdOut, RUSTCONV_WSTR_C0FE14364DC7.as_ptr());
        }

        STRUCTUREPROGRESS => {
            ConPuts(StdOut, RUSTCONV_WSTR_FC4B443CE21B.as_ptr());
        }

        DONEWITHSTRUCTURE => {
            ConPuts(StdOut, RUSTCONV_WSTR_26554534B697.as_ptr());
        }

        CLUSTERSIZETOOSMALL => {
            ConPuts(StdOut, RUSTCONV_WSTR_4A34C150559F.as_ptr());
        }

        PROGRESS => {
            percent = Argument as PDWORD;
            ConResPrintf(StdOut, IDS_PERCENT_COMPL, *percent);
        }

        OUTPUT => {
            output = Argument as PTEXTOUTPUT;
            ConPrintf(StdOut, RUSTCONV_WSTR_BE193C67CE83.as_ptr(), (*output).Output);
        }

        DONE => {
            status = Argument as PBOOLEAN;
            if *status == (FALSE as u8) {
                ConResPrintf(StdOut, IDS_CHKDSK_FAIL, Modifier);
                if Modifier == (STATUS_DISK_CORRUPT_ERROR) as u32 {
                    ConResPuts(StdOut, IDS_RUN_AGAIN);
                } else {
                    ConPrintf(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());
                }
                Error = (TRUE as i32);
            }
        }

        _ => {}
    }
    return Ret;
}

#[no_mangle]
pub unsafe extern "C" fn wmain(mut argc: c_int, mut argv: *mut *mut WCHAR) -> c_int {
    let mut badArg: c_int = 0;
    let mut volumeHandle: HANDLE = core::ptr::null_mut();
    let mut fileSystem: [WCHAR; 1024] = [0; 1024];
    let mut volumeName: [WCHAR; 1024] = [0; 1024];
    let mut serialNumber: DWORD = 0;
    let mut flags: DWORD = 0;
    let mut maxComponent: DWORD = 0;

    /* Initialize the Console Standard Streams */
    ConInitStdStreams();

    ConResPuts(StdOut, IDS_ABOUT as UINT);

    //
    // Parse command line
    //
    badArg = ParseCommandLine(argc, argv);
    if badArg != 0 {
        ConResPrintf(StdOut, IDS_BAD_ARGUMENT as UINT, *argv.offset(badArg as isize));
        Usage(*argv.offset(0 as isize));
        return -1;
    }

    //
    // Get the drive's format
    //
    if Drive.is_null() {
        if GetCurrentDirectoryW(ARRAYSIZE!(CurrentDirectory) as DWORD, &raw mut CurrentDirectory as LPWSTR) == 0 {
            PrintWin32Error(IDS_NO_CURRENT_DIR as UINT, GetLastError());
            return -1;
        }
    } else {
        wcscpy(&raw mut CurrentDirectory as *mut wchar_t, Drive as *const wchar_t);
    }
    CurrentDirectory[2] = 0x005Cu16;
    CurrentDirectory[3] = 0x0000u16;
    Drive = &raw mut CurrentDirectory as *mut WCHAR as PWCHAR;

    //
    // Determine the drive's file system format, which we need to
    // tell chkdsk
    //
    if GetVolumeInformationW(
        Drive as LPCWSTR,
        &raw mut volumeName as LPWSTR,
        ARRAYSIZE!(volumeName) as DWORD,
        &raw mut serialNumber,
        &raw mut maxComponent,
        &raw mut flags,
        &raw mut fileSystem as LPWSTR,
        ARRAYSIZE!(fileSystem) as DWORD,
    ) == 0
    {
        PrintWin32Error(IDS_NO_QUERY_VOL as UINT, GetLastError());
        return -1;
    }

    //
    // If they want to fix, we need to have access to the drive
    //
    if FixErrors != 0 {
        swprintf(
            &raw mut volumeName as *mut wchar_t,
            RUSTCONV_WSTR_F37F62BC0070.as_ptr(),
            *Drive.offset(0 as isize) as c_int,
        );
        volumeHandle = CreateFileW(
            &raw mut volumeName as *mut WCHAR as LPCWSTR,
            GENERIC_WRITE as DWORD,
            0 as DWORD,
            core::ptr::null_mut(),
            OPEN_EXISTING as DWORD,
            0 as DWORD,
            0 as HANDLE,
        );
        if volumeHandle == INVALID_HANDLE_VALUE {
            ConResPuts(StdErr, IDS_VOLUME_IN_USE_PROC as UINT);
            return -1;
        }
        CloseHandle(volumeHandle);

        //
        // Can't let the user break out of a chkdsk that can modify the drive
        //
        SetConsoleCtrlHandler(
            Some(CtrlCIntercept),
            TRUE as BOOL,
        );
    }

    //
    // Just do it
    //
    ConResPrintf(
        StdOut,
        IDS_FILE_SYSTEM as UINT,
        &raw mut fileSystem as *mut WCHAR,
        Drive,
    );
    Chkdsk(
        Drive,
        &raw mut fileSystem as PWCHAR,
        FixErrors as BOOLEAN,
        Verbose as BOOLEAN,
        SkipClean as BOOLEAN,
        ScanSectors as BOOLEAN,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        Some(ChkdskCallback),
    );

    if Error != 0 {
        return -1;
    }
    return 0;
}
