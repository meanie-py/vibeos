// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_075DE2B906DB: [u16; 2] = [0x000Au16, 0x0000];

use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn HelpCommandList() {
    let mut cmdptr: PCOMMAND = core::ptr::null_mut();
    let mut szFormat: [u16; 64] = [0; 64];
    let mut szOutput: [u16; 256] = [0; 256];

    K32LoadStringW(
        GetModuleHandle(core::ptr::null_mut()),
        (IDS_HELP_FORMAT_STRING) as u32,
        szFormat.as_mut_ptr(),
        (ARRAYSIZE!(szFormat)) as i32,
    );

    ConResPuts(StdOut, (IDS_APP_HEADER) as UINT);
    ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());

    cmdptr = (cmds).as_mut_ptr();
    while !(*cmdptr).cmd1.is_null() {
        if !(*cmdptr).cmd1.is_null()
            && (*cmdptr).cmd2.is_null()
            && (*cmdptr).cmd3.is_null()
            && (*cmdptr).help != IDS_NONE
        {
            K32LoadStringW(
                GetModuleHandle(core::ptr::null_mut()),
                ((*cmdptr).help) as u32,
                szOutput.as_mut_ptr(),
                (ARRAYSIZE!(szOutput)) as i32,
            );
            ConPrintf(StdOut, szFormat.as_ptr(), (*cmdptr).cmd1, szOutput.as_ptr());
        }
        cmdptr = cmdptr.offset(1);
    }

    ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());
}
#[no_mangle]
pub unsafe extern "C" fn HelpCommand(pCommand: PCOMMAND) -> EXIT_CODE {
    let mut cmdptr: PCOMMAND;
    let mut bSubCommands: BOOL = FALSE;
    let mut szFormat: [WCHAR; 64] = [0; 64];
    let mut szOutput: [WCHAR; 256] = [0; 256];

    K32LoadStringW(
        GetModuleHandle(core::ptr::null_mut()),
        (IDS_HELP_FORMAT_STRING) as u32,
        szFormat.as_mut_ptr(),
        (ARRAYSIZE!(szFormat)) as i32,
    );

    ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());

    cmdptr = (cmds).as_mut_ptr();
    while !(*cmdptr).cmd1.is_null() {
        if !(*pCommand).cmd1.is_null() && (*pCommand).cmd2.is_null() && (*pCommand).cmd3.is_null() {
            if !(*cmdptr).cmd1.is_null()
                && _wcsicmp((*pCommand).cmd1, (*cmdptr).cmd1) == 0
                && !(*cmdptr).cmd2.is_null()
                && (*cmdptr).cmd3.is_null()
                && (*cmdptr).help != IDS_NONE
            {
                K32LoadStringW(
                    GetModuleHandle(core::ptr::null_mut()),
                    ((*cmdptr).help) as u32,
                    szOutput.as_mut_ptr(),
                    (ARRAYSIZE!(szOutput)) as i32,
                );
                ConPrintf(StdOut, szFormat.as_ptr(), (*cmdptr).cmd2, szOutput.as_ptr());
                bSubCommands = TRUE;
            }
        } else if !(*pCommand).cmd1.is_null() && !(*pCommand).cmd2.is_null() && (*pCommand).cmd3.is_null() {
            if !(*cmdptr).cmd1.is_null()
                && _wcsicmp((*pCommand).cmd1, (*cmdptr).cmd1) == 0
                && !(*cmdptr).cmd2.is_null()
                && _wcsicmp((*pCommand).cmd2, (*cmdptr).cmd2) == 0
                && !(*cmdptr).cmd3.is_null()
                && (*cmdptr).help != IDS_NONE
            {
                K32LoadStringW(
                    GetModuleHandle(core::ptr::null_mut()),
                    ((*cmdptr).help) as u32,
                    szOutput.as_mut_ptr(),
                    (ARRAYSIZE!(szOutput)) as i32,
                );
                ConPrintf(StdOut, szFormat.as_ptr(), (*cmdptr).cmd3, szOutput.as_ptr());
                bSubCommands = TRUE;
            }
        } else if !(*pCommand).cmd1.is_null() && !(*pCommand).cmd2.is_null() && !(*pCommand).cmd3.is_null() {
            if !(*cmdptr).cmd1.is_null()
                && _wcsicmp((*pCommand).cmd1, (*cmdptr).cmd1) == 0
                && !(*cmdptr).cmd2.is_null()
                && _wcsicmp((*pCommand).cmd2, (*cmdptr).cmd2) == 0
                && !(*cmdptr).cmd3.is_null()
                && _wcsicmp((*pCommand).cmd3, (*cmdptr).cmd3) == 0
                && (*cmdptr).help_detail != (MSG_NONE) as u32
            {
                ConMsgPuts(
                    StdOut,
                    FORMAT_MESSAGE_FROM_HMODULE,
                    core::ptr::null_mut(),
                    (*cmdptr).help_detail,
                    LANG_USER_DEFAULT,
                );
                bSubCommands = TRUE;
            }
        }
        cmdptr = cmdptr.offset(1);
    }

    if bSubCommands == FALSE && (*pCommand).help_detail != (MSG_NONE) as u32 {
        ConMsgPuts(
            StdOut,
            FORMAT_MESSAGE_FROM_HMODULE,
            core::ptr::null_mut(),
            (*pCommand).help_detail,
            LANG_USER_DEFAULT,
        );
    }

    ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());

    return EXIT_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn help_main(argc: INT, argv: *mut PWSTR) -> EXIT_CODE {
    let mut cmdptr: PCOMMAND;
    let mut cmdptr1: PCOMMAND = core::ptr::null_mut();
    let mut cmdptr2: PCOMMAND = core::ptr::null_mut();
    let mut cmdptr3: PCOMMAND = core::ptr::null_mut();

    if argc == 1 {
        HelpCommandList();
        return EXIT_SUCCESS;
    }

    cmdptr = &raw mut cmds as *mut COMMAND as PCOMMAND;
    while !(*cmdptr).cmd1.is_null() {
        if cmdptr1.is_null()
            && (!(*cmdptr).cmd1.is_null()
                && _wcsicmp(
                    *argv.offset(1 as isize) as *const u16,
                    (*cmdptr).cmd1 as *const u16,
                ) == 0)
        {
            cmdptr1 = cmdptr;
        }
        if cmdptr2.is_null()
            && argc >= 3
            && (!(*cmdptr).cmd1.is_null()
                && _wcsicmp(
                    *argv.offset(1 as isize) as *const u16,
                    (*cmdptr).cmd1 as *const u16,
                ) == 0)
            && (!(*cmdptr).cmd2.is_null()
                && _wcsicmp(
                    *argv.offset(2 as isize) as *const u16,
                    (*cmdptr).cmd2 as *const u16,
                ) == 0)
        {
            cmdptr2 = cmdptr;
        }
        if cmdptr3.is_null()
            && argc >= 4
            && (!(*cmdptr).cmd1.is_null()
                && _wcsicmp(
                    *argv.offset(1 as isize) as *const u16,
                    (*cmdptr).cmd1 as *const u16,
                ) == 0)
            && (!(*cmdptr).cmd2.is_null()
                && _wcsicmp(
                    *argv.offset(2 as isize) as *const u16,
                    (*cmdptr).cmd2 as *const u16,
                ) == 0)
            && (!(*cmdptr).cmd3.is_null()
                && _wcsicmp(
                    *argv.offset(3 as isize) as *const u16,
                    (*cmdptr).cmd3 as *const u16,
                ) == 0)
        {
            cmdptr3 = cmdptr;
        }
        cmdptr = cmdptr.offset(1);
    }

    if !cmdptr3.is_null() {
        return HelpCommand(cmdptr3);
    } else if !cmdptr2.is_null() {
        return HelpCommand(cmdptr2);
    } else if !cmdptr1.is_null() {
        return HelpCommand(cmdptr1);
    }

    HelpCommandList();

    return EXIT_SUCCESS;
}
