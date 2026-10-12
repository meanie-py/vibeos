// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_5462AED96D21: [u16; 3] = [0x002Fu16, 0x003F, 0x0000];

use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn PrintError(dwError: DWORD) {
    if dwError == ERROR_SUCCESS as DWORD {
        return;
    }

    ConMsgPuts(
        StdErr,
        FORMAT_MESSAGE_FROM_SYSTEM as DWORD,
        core::ptr::null_mut(),
        dwError,
        LANG_USER_DEFAULT as DWORD,
    );
}

unsafe extern "C" fn IsDataUnicode(hGlobal: HGLOBAL) -> BOOL {
    let mut bReturn: BOOL;
    let mut lpBuffer: LPVOID;

    lpBuffer = GlobalLock(hGlobal);
    bReturn = IsTextUnicode(lpBuffer as *const ::core::ffi::c_void, GlobalSize(hGlobal) as ::core::ffi::c_int, core::ptr::null_mut::<::core::ffi::c_int>());
    GlobalUnlock(hGlobal);

    return bReturn;
}

#[no_mangle]
pub unsafe extern "C" fn wmain(argc: ::core::ffi::c_int, argv: *mut *mut wchar_t) -> ::core::ffi::c_int {
    let mut hInput: HANDLE = core::ptr::null_mut();
    let mut dwBytesRead: DWORD = 0;
    let mut bStatus: BOOL = 0;
    let mut hBuffer: HGLOBAL = core::ptr::null_mut();
    let mut hTemp: HGLOBAL = core::ptr::null_mut();
    let mut lpBuffer: LPBYTE = core::ptr::null_mut();
    let mut dwSize: SIZE_T = 0;

    /* Initialize the Console Standard Streams */
    hInput = GetStdHandle(STD_INPUT_HANDLE);
    ConInitStdStreams();

    /* Check for usage */
    if argc > 1
        && wcsncmp(
            *argv.offset(1 as isize),
            RUSTCONV_WSTR_5462AED96D21.as_ptr(),
            2,
        ) == 0
    {
        ConResPuts(StdOut, IDS_HELP);
        return 0;
    }

    if GetFileType(hInput) == FILE_TYPE_CHAR {
        ConResPuts(StdOut, IDS_USAGE);
        return 0;
    }

    'failure: {
        /* Initialize a growable buffer for holding clipboard data */
        hBuffer = GlobalAlloc(GMEM_MOVEABLE | GMEM_ZEROINIT, 4096);
        if hBuffer.is_null() {
            break 'failure;
        }

        /*
         * Read data from the input stream by chunks of 4096 bytes
         * and resize the buffer each time when needed.
         */
        loop {
            lpBuffer = GlobalLock(hBuffer) as LPBYTE;
            if lpBuffer.is_null() {
                break 'failure;
            }

            bStatus = ReadFile(
                hInput,
                lpBuffer.offset(dwSize as isize) as LPVOID,
                4096,
                &mut dwBytesRead,
                core::ptr::null_mut(),
            );
            dwSize = dwSize.wrapping_add(dwBytesRead as SIZE_T);
            GlobalUnlock(hBuffer);

            hTemp = GlobalReAlloc(
                hBuffer,
                GlobalSize(hBuffer).wrapping_add(4096 as SIZE_T),
                GMEM_ZEROINIT,
            );
            if hTemp.is_null() {
                break 'failure;
            }

            hBuffer = hTemp;

            if !(bStatus != 0 && dwBytesRead > 0) {
                break;
            }
        }

        /*
         * Resize the buffer to the total size of data read.
         * Note that, if the call fails, we still have the old buffer valid.
         * The old buffer would be larger than the actual size of data it contains,
         * but this is not a problem for us.
         */
        hTemp = GlobalReAlloc(
            hBuffer,
            dwSize.wrapping_add(core::mem::size_of::<WCHAR>() as SIZE_T),
            GMEM_ZEROINIT,
        );
        if !hTemp.is_null() {
            hBuffer = hTemp;
        }

        /* Attempt to open the clipboard */
        if OpenClipboard(core::ptr::null_mut()) == 0 {
            break 'failure;
        }

        /* Empty it, copy our data, then close it */

        EmptyClipboard();

        if IsDataUnicode(hBuffer) != 0 {
            SetClipboardData(CF_UNICODETEXT, hBuffer as HANDLE);
        } else {
            // TODO: Convert text from current console page to standard ANSI.
            // Alternatively one can use CF_OEMTEXT as done here.
            SetClipboardData(CF_OEMTEXT, hBuffer as HANDLE);
        }

        CloseClipboard();
        return 0;
    }

    // Failure:
    if !hBuffer.is_null() {
        GlobalFree(hBuffer);
    }
    PrintError(GetLastError());
    return -1;
}
