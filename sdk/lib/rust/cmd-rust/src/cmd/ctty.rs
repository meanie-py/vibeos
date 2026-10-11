use crate::cmd::console::ConOutResPaging;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_3DB566BC3DA4: [u16; 4] = [0x0043u16, 0x004F, 0x004D, 0x0000];
pub static RUSTCONV_WSTR_5BD07A46AC2D: [u16; 4] = [0x0043u16, 0x004F, 0x004E, 0x0000];
pub static RUSTCONV_WSTR_5F451136015F: [u16; 4] = [0x0041u16, 0x0055, 0x0058, 0x0000];
pub static RUSTCONV_WSTR_648E5B4273EE: [u16; 4] = [0x0050u16, 0x0052, 0x004E, 0x0000];
pub static RUSTCONV_WSTR_A8E70516B833: [u16; 4] = [0x004Eu16, 0x0055, 0x004C, 0x0000];
pub static RUSTCONV_WSTR_C372F2CCF22D: [u16; 4] = [0x004Cu16, 0x0050, 0x0054, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// No file-scope declarations: the only `static`s in this preamble (`DosLPTDevice` through `DosNULDevice`, and `SecAttr`) are function-local, so they belong to the function bodies, not this pass.

unsafe extern "C" fn CheckTerminalDeviceType(mut pszName: LPCTSTR) -> WORD {
    /* Console reserved "file" names */
    let DosLPTDevice: LPCWSTR = RUSTCONV_WSTR_C372F2CCF22D.as_ptr();
    let DosCOMDevice: LPCWSTR = RUSTCONV_WSTR_3DB566BC3DA4.as_ptr();
    let DosPRNDevice: LPCWSTR = RUSTCONV_WSTR_648E5B4273EE.as_ptr();
    let DosAUXDevice: LPCWSTR = RUSTCONV_WSTR_5F451136015F.as_ptr();
    let DosCONDevice: LPCWSTR = RUSTCONV_WSTR_5BD07A46AC2D.as_ptr();
    let DosNULDevice: LPCWSTR = RUSTCONV_WSTR_A8E70516B833.as_ptr();

    let mut DeviceName: LPCWSTR;
    let mut DeviceNameInfo: ULONG;
    let mut DeviceType: WORD = 0 as WORD; // 0: Unknown; 1: CON; 2: COM etc...

    // UNICODE means that TCHAR == WCHAR == UTF-16
    let unicode_build: bool = true;
    let mut buffer: *mut WCHAR = core::ptr::null_mut();
    if !unicode_build {
        /* Convert from the current process/thread's codepage to UTF-16 */
        let mut len: DWORD = (strlen(pszName as *const i8) as DWORD)
            + 1 as DWORD;
        buffer = cmd_alloc!((len as usize) * core::mem::size_of::<WCHAR>()) as *mut WCHAR;
        if buffer.is_null() {
            // SetLastError(ERROR_NOT_ENOUGH_MEMORY);
            return FALSE as WORD;
        }
        len = MultiByteToWideChar(
            CP_THREAD_ACP as UINT, // CP_ACP, CP_OEMCP
            0 as DWORD,
            pszName as *const i8,
            len as INT,
            buffer,
            len as INT,
        ) as DWORD;
        DeviceName = buffer as LPCWSTR;
    } else {
        DeviceName = pszName as LPCWSTR;
    }

    /*
     * Check whether we deal with a DOS device, and if so,
     * strip the path till the file name.
     * Therefore, things like \\.\CON or C:\some_path\COM1
     * are transformed into CON or COM1, for example.
     */
    DeviceNameInfo = RtlIsDosDeviceName_U(DeviceName as PCWSTR);
    if DeviceNameInfo != 0 as ULONG {
        DeviceName = (DeviceName as ULONG_PTR).wrapping_add(
            ((DeviceNameInfo >> 16) & 0xFFFF as ULONG) as ULONG_PTR,
        ) as LPCWSTR;

        if _wcsnicmp(DeviceName as *const WCHAR, DosCONDevice as *const WCHAR, 3 as usize)
            == 0 as i32
        {
            DeviceType = 1 as WORD;
        } else if _wcsnicmp(DeviceName as *const WCHAR, DosLPTDevice as *const WCHAR, 3 as usize)
            == 0 as i32
            || _wcsnicmp(DeviceName as *const WCHAR, DosCOMDevice as *const WCHAR, 3 as usize)
                == 0 as i32
            || _wcsnicmp(DeviceName as *const WCHAR, DosPRNDevice as *const WCHAR, 3 as usize)
                == 0 as i32
            || _wcsnicmp(DeviceName as *const WCHAR, DosAUXDevice as *const WCHAR, 3 as usize)
                == 0 as i32
            || _wcsnicmp(DeviceName as *const WCHAR, DosNULDevice as *const WCHAR, 3 as usize)
                == 0 as i32
        {
            DeviceType = 2 as WORD;
        }
        // else DeviceType = 0;
    }

    if !unicode_build {
        cmd_free!(buffer);
    }

    return DeviceType;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_ctty(param: LPTSTR) -> INT {
    static mut SecAttr: SECURITY_ATTRIBUTES = SECURITY_ATTRIBUTES {
        nLength: core::mem::size_of::<SECURITY_ATTRIBUTES>() as DWORD,
        lpSecurityDescriptor: core::ptr::null_mut(),
        bInheritHandle: (TRUE) as i32,
    };

    let mut Success: BOOL;
    let mut DeviceType: WORD;
    let mut hDevice: HANDLE;
    let mut hStdHandles: [HANDLE; 3] = [core::ptr::null_mut(); 3];

    let slash_help: [u16; 3] = ['/' as u16, '?' as u16, 0];
    let conin_name: [u16; 7] = ['C' as u16, 'O' as u16, 'N' as u16, 'I' as u16, 'N' as u16, '$' as u16, 0];
    let conout_name: [u16; 8] = ['C' as u16, 'O' as u16, 'N' as u16, 'O' as u16, 'U' as u16, 'T' as u16, '$' as u16, 0];

    /* The user asked for help */
    if _tcsncmp(param as LPCTSTR, slash_help.as_ptr() as LPCTSTR, 2) == 0
    {
        ConOutResPaging((TRUE) as BOOL, STRING_CTTY_HELP);
        return 0;
    }

    if *param == 0
    {
        error_req_param_missing();
        return 1;
    }

    /* Check whether this is a valid terminal device name */
    DeviceType = CheckTerminalDeviceType(param as LPCTSTR);
    if DeviceType == 1
    {
        /*
         * Special case for CON device.
         */
        let mut bRetry: BOOL = (FALSE) as i32;

        /* RetryOpenConsole: label lowered to loop head; goto becomes continue */
        loop
        {
            /*
             * If we previously failed in opening handles to the console,
             * this means the existing console is almost destroyed.
             * Close the existing console and allocate and open a new one.
             */
            if bRetry != 0
            {
                FreeConsole();
                if AllocConsole() == 0
                {
                    return 1;
                }
            }

            /* Attempt to retrieve a handle for standard input */
            hStdHandles[0] = CreateFile(
                conin_name.as_ptr() as LPCTSTR,
                GENERIC_READ | GENERIC_WRITE,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                core::ptr::addr_of_mut!(SecAttr),
                OPEN_EXISTING,
                0,
                core::ptr::null_mut(),
            );
            if hStdHandles[0] == INVALID_HANDLE_VALUE
            {
                // TODO: Error
                // error_no_rw_device(param);

                if bRetry != 0
                {
                    return 1;
                }
                bRetry = (TRUE) as i32;
                continue;
            }

            /* Attempt to retrieve a handle for standard output.
             * Note that GENERIC_READ is needed for IsConsoleHandle() to succeed afterwards. */
            hStdHandles[1] = CreateFile(
                conout_name.as_ptr() as LPCTSTR,
                GENERIC_READ | GENERIC_WRITE,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                core::ptr::addr_of_mut!(SecAttr),
                OPEN_EXISTING,
                0,
                core::ptr::null_mut(),
            );
            if hStdHandles[1] == INVALID_HANDLE_VALUE
            {
                // TODO: Error
                // error_no_rw_device(param);

                CloseHandle(hStdHandles[0]);

                if bRetry != 0
                {
                    return 1;
                }
                bRetry = (TRUE) as i32;
                continue;
            }

            break;
        }

        /* Duplicate a handle for standard error */
        Success = DuplicateHandle(
            GetCurrentProcess(),
            hStdHandles[1],
            GetCurrentProcess(),
            core::ptr::addr_of_mut!(hStdHandles[2]),
            0, // GENERIC_WRITE,
            (TRUE) as BOOL,
            DUPLICATE_SAME_ACCESS /* 0 */,
        );
        if Success == 0
        {
            // TODO: Error
            // error_no_rw_device(param);
            CloseHandle(hStdHandles[1]);
            CloseHandle(hStdHandles[0]);
            return 1;
        }
    }
    else if DeviceType == 2
    {
        /*
         * COM and the other devices can only be opened once.
         * Since we need different handles, we need to duplicate them.
         */

        /* Attempt to retrieve a handle to the device for read/write access */
        hDevice = CreateFile(
            param as LPCWSTR,
            GENERIC_READ | GENERIC_WRITE,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            core::ptr::addr_of_mut!(SecAttr),
            OPEN_EXISTING,
            0, // FILE_FLAG_OVERLAPPED, // 0,
            core::ptr::null_mut(),
        );
        if hDevice == INVALID_HANDLE_VALUE
        {
            // TODO: Error
            // error_no_rw_device(param);
            return 1;
        }

        /* Duplicate a handle for standard input */
        Success = DuplicateHandle(
            GetCurrentProcess(),
            hDevice,
            GetCurrentProcess(),
            core::ptr::addr_of_mut!(hStdHandles[0]),
            GENERIC_READ,
            (TRUE) as BOOL,
            0,
        );
        if Success == 0
        {
            // TODO: Error
            // error_no_rw_device(param);
            CloseHandle(hDevice);
            return 1;
        }

        /* Duplicate a handle for standard output */
        Success = DuplicateHandle(
            GetCurrentProcess(),
            hDevice,
            GetCurrentProcess(),
            core::ptr::addr_of_mut!(hStdHandles[1]),
            GENERIC_WRITE,
            (TRUE) as BOOL,
            0,
        );
        if Success == 0
        {
            // TODO: Error
            // error_no_rw_device(param);
            CloseHandle(hStdHandles[0]);
            CloseHandle(hDevice);
            return 1;
        }

        /* Duplicate a handle for standard error */
        Success = DuplicateHandle(
            GetCurrentProcess(),
            hDevice,
            GetCurrentProcess(),
            core::ptr::addr_of_mut!(hStdHandles[2]),
            GENERIC_WRITE,
            (TRUE) as BOOL,
            0,
        );
        if Success == 0
        {
            // TODO: Error
            // error_no_rw_device(param);
            CloseHandle(hStdHandles[1]);
            CloseHandle(hStdHandles[0]);
            CloseHandle(hDevice);
            return 1;
        }

        /* Now get rid of the main device handle */
        CloseHandle(hDevice);
    }
    else
    {
        ConOutResPrintf!(STRING_ERROR_INVALID_DEVICE, param);
        return 1;
    }

    /* Now set the standard handles */

    hDevice = GetHandle(0);
    if hDevice != INVALID_HANDLE_VALUE
    {
        CloseHandle(hDevice);
    }
    SetHandle(0, hStdHandles[0]);

    hDevice = GetHandle(1);
    if hDevice != INVALID_HANDLE_VALUE
    {
        CloseHandle(hDevice);
    }
    SetHandle(1, hStdHandles[1]);

    hDevice = GetHandle(2);
    if hDevice != INVALID_HANDLE_VALUE
    {
        CloseHandle(hDevice);
    }
    SetHandle(2, hStdHandles[2]);

    return 0;
}
