// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_075DE2B906DB: [u16; 2] = [0x000Au16, 0x0000];
pub static RUSTCONV_WSTR_1B9E0FE7B740: [u16; 2] = [0x003Fu16, 0x0000];
pub static RUSTCONV_WSTR_52E9E3ADD38B: [u16; 2] = [0x0073u16, 0x0000];
pub static RUSTCONV_WSTR_8B72557F3C97: [u16; 2] = [0x0074u16, 0x0000];
pub static RUSTCONV_WSTR_A138E0184120: [u16; 67] = [
    0x000Au16, 0x002A, 0x0057, 0x0041, 0x0052, 0x004E, 0x0049, 0x004E, 0x0047, 0x002A,
    0x003A, 0x0020, 0x0054, 0x0068, 0x0069, 0x0073, 0x0020, 0x0070, 0x0072, 0x006F,
    0x0067, 0x0072, 0x0061, 0x006D, 0x0020, 0x0069, 0x0073, 0x0020, 0x0069, 0x006E,
    0x0063, 0x006F, 0x006D, 0x0070, 0x006C, 0x0065, 0x0074, 0x0065, 0x0020, 0x0061,
    0x006E, 0x0064, 0x0020, 0x006D, 0x0061, 0x0079, 0x0020, 0x006E, 0x006F, 0x0074,
    0x0020, 0x0077, 0x006F, 0x0072, 0x006B, 0x0020, 0x0070, 0x0072, 0x006F, 0x0070,
    0x0065, 0x0072, 0x006C, 0x0079, 0x002E, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_D877BF4E5023: [u16; 2] = [0x0072u16, 0x0000];

use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn ShowHeader() {
    let mut szComputerName: [u16; (MAX_STRING_SIZE) as usize] = [0; (MAX_STRING_SIZE) as usize];
    let mut comp_size: u32 = (MAX_STRING_SIZE) as u32;

    GetComputerNameW(szComputerName.as_mut_ptr(), &mut comp_size);

    ConPuts(StdOut, RUSTCONV_WSTR_A138E0184120.as_ptr());

    ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());
    ConResPuts(StdOut, (IDS_APP_HEADER) as UINT);
    ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());
    ConResPuts(StdOut, (IDS_APP_LICENSE) as UINT);
    ConResPrintf(StdOut, (IDS_APP_CURR_COMPUTER) as u32, szComputerName.as_mut_ptr());
}
#[no_mangle]
pub unsafe extern "C" fn RunScript(filename: LPCWSTR) -> EXIT_CODE {
    let mut script: *mut FILE = core::ptr::null_mut();
    let mut tmp_string: [u16; (MAX_STRING_SIZE) as usize] = [0; (MAX_STRING_SIZE) as usize];
    let mut Result: EXIT_CODE;

    script = _wfopen(filename, RUSTCONV_WSTR_D877BF4E5023.as_ptr());
    if script.is_null() {
        ConResPrintf(StdErr, (IDS_ERROR_MSG_NO_SCRIPT) as u32, filename);
        return (FALSE) as EXIT_CODE;
    }

    while !fgetws(
        tmp_string.as_mut_ptr(),
        MAX_STRING_SIZE,
        script,
    )
    .is_null()
    {
        Result = InterpretScript(tmp_string.as_mut_ptr());
        if Result != EXIT_SUCCESS {
            fclose(script);
            return if Result == EXIT_EXIT {
                EXIT_SUCCESS
            } else {
                Result
            };
        }
    }

    fclose(script);

    return EXIT_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn wmain(
    argc: i32,
    argv: *const LPWSTR,
) -> i32 {
    let mut script: LPCWSTR = core::ptr::null_mut();
    let mut tmpBuffer: LPCWSTR = core::ptr::null_mut();
    let mut appTitle: [u16; 50] = [0; 50];
    let mut index: i32;
    let mut timeout: i32;
    let mut result: i32 = (EXIT_SUCCESS) as i32;

    ConInitStdStreams();

    K32LoadStringW(
        GetModuleHandle(core::ptr::null_mut()),
        (IDS_APP_HEADER) as u32,
        appTitle.as_mut_ptr(),
        (ARRAYSIZE!(appTitle)) as i32,
    );
    SetConsoleTitleW(appTitle.as_ptr());

    timeout = 0;

    CreatePartitionList();
    CreateVolumeList();

    if argc < 2 {
        ShowHeader();
        InterpretMain();
    } else {
        index = 1;
        while index < argc {
            if *(*argv.add(index as usize)).add(0) == '/' as u16
                || *(*argv.add(index as usize)).add(0) == '-' as u16
            {
                tmpBuffer = (*argv.add(index as usize)).add(1);
            } else {
                ConResPrintf(StdErr, (IDS_ERROR_MSG_BAD_ARG) as u32, *argv.add(index as usize));
                result = (EXIT_SYNTAX) as i32;
                break;
            }

            if _wcsicmp(tmpBuffer, RUSTCONV_WSTR_1B9E0FE7B740.as_ptr()) == 0 {
                ConResPuts(StdOut, (IDS_APP_USAGE) as UINT);
                result = (EXIT_SUCCESS) as i32;
                break;
            } else if _wcsicmp(tmpBuffer, RUSTCONV_WSTR_52E9E3ADD38B.as_ptr()) == 0 {
                if (index + 1) < argc {
                    index += 1;
                    script = *argv.add(index as usize);
                }
            } else if _wcsicmp(tmpBuffer, RUSTCONV_WSTR_8B72557F3C97.as_ptr()) == 0 {
                if (index + 1) < argc {
                    index += 1;
                    timeout = _wtoi(*argv.add(index as usize));

                    if timeout < 0 {
                        timeout = 0;
                    }
                }
            } else {
                ConResPrintf(StdErr, (IDS_ERROR_MSG_BAD_ARG) as u32, tmpBuffer);
                result = (EXIT_SYNTAX) as i32;
                break;
            }

            index += 1;
        }

        if (result) as u32 == EXIT_SUCCESS {
            ShowHeader();

            if !script.is_null() {
                if timeout > 0 {
                    Sleep((timeout * 1000) as DWORD);
                }

                result = (RunScript(script)) as i32;
                if (result) as u32 != EXIT_SUCCESS {
                    // goto done
                } else {
                    ConResPuts(StdOut, (IDS_APP_LEAVING) as UINT);
                }
            } else {
                ConResPrintf(StdErr, (IDS_ERROR_MSG_NO_SCRIPT) as u32, script);
                result = (EXIT_SYNTAX) as i32;
            }
        } else {
            // error path from argument parsing
        }
    }

    if (result) as u32 == EXIT_SUCCESS {
        ConResPuts(StdOut, (IDS_APP_LEAVING) as UINT);
    }

    DestroyVolumeList();
    DestroyPartitionList();

    result
}
