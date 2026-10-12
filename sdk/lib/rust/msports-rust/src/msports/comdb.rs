// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_06AF8F617F65: [u16; 6] = [0x0043u16, 0x006F, 0x006D, 0x0044, 0x0042, 0x0000];
pub static RUSTCONV_WSTR_6180B91436ED: [u16; 11] = [
    0x0043u16, 0x006F, 0x006D, 0x0044, 0x0042, 0x004D, 0x0075, 0x0074, 0x0065, 0x0078,
    0x0000,
];
pub static RUSTCONV_WSTR_8A3038F3E9A7: [u16; 50] = [
    0x0053u16, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x005C, 0x0043, 0x0075, 0x0072,
    0x0072, 0x0065, 0x006E, 0x0074, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F,
    0x006C, 0x0053, 0x0065, 0x0074, 0x005C, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072,
    0x006F, 0x006C, 0x005C, 0x0043, 0x004F, 0x004D, 0x0020, 0x004E, 0x0061, 0x006D,
    0x0065, 0x0020, 0x0041, 0x0072, 0x0062, 0x0069, 0x0074, 0x0065, 0x0072, 0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----

pub type PCOMDB = *mut COMDB;

pub const BITS_PER_BYTE: u32 = 8;

pub const BITMAP_SIZE_INCREMENT: u32 = 0x400;

pub const BITMAP_SIZE_INVALID_BITS: u32 = 0x3FF;

#[no_mangle]
pub unsafe extern "system" fn ComDBClaimNextFreePort(hComDB: HCOMDB, ComNumber: LPDWORD) -> LONG {
    let mut pComDB: PCOMDB = core::ptr::null_mut::<_COMDB>();
    let mut dwBitIndex: DWORD = 0;
    let mut dwByteIndex: DWORD = 0;
    let mut dwSize: DWORD = 0;
    let mut dwType: DWORD = 0;
    let mut pBitmap: PBYTE = core::ptr::null_mut::<BYTE>();
    let mut cMask: BYTE = 0;
    let mut lError: LONG = 0;

    if hComDB == INVALID_HANDLE_VALUE as HCOMDB || hComDB.is_null() || ComNumber.is_null() {
        return ERROR_INVALID_PARAMETER as LONG;
    }

    pComDB = hComDB as PCOMDB;

    /* Wait for the mutex */
    WaitForSingleObject((*pComDB).hMutex, INFINITE);

    /* Get the required bitmap size */
    lError = RegQueryValueExW(
        (*pComDB).hKey,
        RUSTCONV_WSTR_06AF8F617F65.as_ptr(),
        core::ptr::null_mut::<DWORD>(),
        core::ptr::addr_of_mut!(dwType),
        core::ptr::null_mut::<BYTE>(),
        core::ptr::addr_of_mut!(dwSize),
    ) as LONG;
    'done: {
        if lError != ERROR_SUCCESS as LONG {
            ERR!("Failed to query the bitmap size!\n");
            break 'done;
        }

        /* Allocate the bitmap */
        pBitmap = HeapAlloc(
            GetProcessHeap(),
            HEAP_ZERO_MEMORY,
            dwSize as SIZE_T,
        ) as PBYTE;
        if pBitmap.is_null() {
            ERR!("Failed to allocate the bitmap!\n");
            lError = ERROR_NOT_ENOUGH_MEMORY as LONG;
            break 'done;
        }

        /* Read the bitmap */
        lError = RegQueryValueExW(
            (*pComDB).hKey,
            RUSTCONV_WSTR_06AF8F617F65.as_ptr(),
            core::ptr::null_mut::<DWORD>(),
            core::ptr::addr_of_mut!(dwType),
            pBitmap as LPBYTE,
            core::ptr::addr_of_mut!(dwSize),
        ) as LONG;
        if lError != ERROR_SUCCESS as LONG {
            break 'done;
        }

        lError = ERROR_INVALID_PARAMETER as LONG;
        dwBitIndex = 0 as DWORD;
        while dwBitIndex < dwSize.wrapping_mul(BITS_PER_BYTE as DWORD) {
            /* Calculate the byte index and a mask for the affected bit */
            dwByteIndex = dwBitIndex / (BITS_PER_BYTE as DWORD);
            cMask = (1 as BYTE) << (dwBitIndex % (BITS_PER_BYTE as DWORD));

            if (*pBitmap.add(dwByteIndex as usize) & cMask) == (0 as BYTE) {
                *pBitmap.add(dwByteIndex as usize) |= cMask;
                *ComNumber = dwBitIndex.wrapping_add(1 as DWORD);
                lError = ERROR_SUCCESS as LONG;
                break;
            }
            dwBitIndex = dwBitIndex.wrapping_add(1);
        }

        /* Save the bitmap if it was modified */
        if lError == ERROR_SUCCESS as LONG {
            lError = RegSetValueExW(
                (*pComDB).hKey,
                RUSTCONV_WSTR_06AF8F617F65.as_ptr(),
                0 as DWORD,
                REG_BINARY as DWORD,
                pBitmap as *const BYTE,
                dwSize,
            ) as LONG;
        }
    }

    /* Release the mutex */
    ReleaseMutex((*pComDB).hMutex);

    /* Release the bitmap */
    if !pBitmap.is_null() {
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            pBitmap as PVOID,
        );
    }

    return lError;
}

#[no_mangle]
pub unsafe extern "system" fn ComDBClaimPort(
    mut hComDB: HCOMDB,
    mut ComNumber: DWORD,
    mut ForceClaim: BOOL,
    mut Forced: PBOOL,
) -> LONG {
    let mut pComDB: PCOMDB = core::ptr::null_mut();
    let mut dwBitIndex: DWORD = 0;
    let mut dwByteIndex: DWORD = 0;
    let mut dwType: DWORD = 0;
    let mut dwSize: DWORD = 0;
    let mut pBitmap: PBYTE = core::ptr::null_mut();
    let mut cMask: BYTE = 0;
    let mut lError: LONG = 0;

    if hComDB == INVALID_HANDLE_VALUE as HCOMDB
        || hComDB.is_null()
        || ComNumber == 0 as DWORD
        || ComNumber > COMDB_MAX_PORTS_ARBITRATED as DWORD
    {
        return ERROR_INVALID_PARAMETER as LONG;
    }

    pComDB = hComDB as PCOMDB;

    /* Wait for the mutex */
    WaitForSingleObject((*pComDB).hMutex, INFINITE as DWORD);

    /* Get the required bitmap size */
    lError = RegQueryValueExW(
        (*pComDB).hKey,
        RUSTCONV_WSTR_06AF8F617F65.as_ptr(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(dwType),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(dwSize),
    ) as LONG;
    if lError != ERROR_SUCCESS as LONG {
        ERR!("Failed to query the bitmap size!\n");
        // goto done
        ReleaseMutex((*pComDB).hMutex);
        if !pBitmap.is_null() {
            HeapFree(
                GetProcessHeap(),
                0 as DWORD,
                pBitmap as PVOID,
            );
        }
        return lError;
    }

    /* Allocate the bitmap */
    pBitmap = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY as DWORD,
        dwSize as SIZE_T,
    ) as PBYTE;
    if pBitmap.is_null() {
        ERR!("Failed to allocate the bitmap!\n");
        lError = ERROR_NOT_ENOUGH_MEMORY as LONG;
        // goto done
        ReleaseMutex((*pComDB).hMutex);
        if !pBitmap.is_null() {
            HeapFree(
                GetProcessHeap(),
                0 as DWORD,
                pBitmap as PVOID,
            );
        }
        return lError;
    }

    /* Read the bitmap */
    lError = RegQueryValueExW(
        (*pComDB).hKey,
        RUSTCONV_WSTR_06AF8F617F65.as_ptr(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(dwType),
        pBitmap as LPBYTE,
        core::ptr::addr_of_mut!(dwSize),
    ) as LONG;
    if lError != ERROR_SUCCESS as LONG {
        // goto done
        ReleaseMutex((*pComDB).hMutex);
        if !pBitmap.is_null() {
            HeapFree(
                GetProcessHeap(),
                0 as DWORD,
                pBitmap as PVOID,
            );
        }
        return lError;
    }

    /* Get the bit index */
    dwBitIndex = ComNumber.wrapping_sub(1 as DWORD);

    /* Check if the bit to set fits into the bitmap */
    if dwBitIndex >= dwSize.wrapping_mul(BITS_PER_BYTE as DWORD) {
        lError = ERROR_INVALID_PARAMETER as LONG;
        // goto done
        ReleaseMutex((*pComDB).hMutex);
        if !pBitmap.is_null() {
            HeapFree(
                GetProcessHeap(),
                0 as DWORD,
                pBitmap as PVOID,
            );
        }
        return lError;
    }

    /* Calculate the byte index and a mask for the affected bit */
    dwByteIndex = dwBitIndex / (BITS_PER_BYTE as DWORD);
    cMask = (1 as BYTE) << (dwBitIndex % (BITS_PER_BYTE as DWORD));

    lError = ERROR_SHARING_VIOLATION as LONG;

    /* Check if the bit is not set */
    if (*pBitmap.offset(dwByteIndex as isize) & cMask) == (0 as BYTE) {
        /* Set the bit */
        *pBitmap.offset(dwByteIndex as isize) |= cMask;
        lError = ERROR_SUCCESS as LONG;
    }

    /* Save the bitmap if it was modified */
    if lError == ERROR_SUCCESS as LONG {
        lError = RegSetValueExW(
            (*pComDB).hKey,
            RUSTCONV_WSTR_06AF8F617F65.as_ptr(),
            0 as DWORD,
            REG_BINARY as DWORD,
            pBitmap as *const BYTE,
            dwSize,
        ) as LONG;
    }

    /* done: Release the mutex */
    ReleaseMutex((*pComDB).hMutex);

    /* Release the bitmap */
    if !pBitmap.is_null() {
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            pBitmap as PVOID,
        );
    }

    return lError;
}

#[no_mangle]
pub unsafe extern "system" fn ComDBClose(hComDB: HCOMDB) -> LONG {
    let mut pComDB: PCOMDB;

    if hComDB == HCOMDB_INVALID_HANDLE_VALUE ||
        hComDB.is_null() {
        return ERROR_INVALID_PARAMETER as LONG;
    }

    pComDB = hComDB as PCOMDB;

    /* Close the registry key */
    if !(*pComDB).hKey.is_null() {
        RegCloseKey((*pComDB).hKey);
    }

    /* Close the mutex */
    if !(*pComDB).hMutex.is_null() {
        CloseHandle((*pComDB).hMutex);
    }

    /* Release the database */
    HeapFree(GetProcessHeap(), 0, pComDB as PVOID);

    return ERROR_SUCCESS as LONG;
}

#[no_mangle]
pub unsafe extern "system" fn ComDBGetCurrentPortUsage(
    hComDB: HCOMDB,
    Buffer: PBYTE,
    BufferSize: DWORD,
    ReportType: DWORD,
    MaxPortsReported: LPDWORD,
) -> LONG {
    let mut pComDB: PCOMDB = core::ptr::null_mut();
    let mut dwBitIndex: DWORD = 0;
    let mut dwByteIndex: DWORD = 0;
    let mut dwType: DWORD = 0;
    let mut dwSize: DWORD = 0;
    let mut pBitmap: PBYTE = core::ptr::null_mut();
    let mut cMask: BYTE = 0;
    let mut lError: LONG = ERROR_SUCCESS as LONG;

    if hComDB == INVALID_HANDLE_VALUE as HCOMDB
        || hComDB.is_null()
        || (Buffer.is_null() && MaxPortsReported.is_null())
        || (ReportType != CDB_REPORT_BITS as DWORD && ReportType != CDB_REPORT_BYTES as DWORD)
    {
        return ERROR_INVALID_PARAMETER as LONG;
    }

    pComDB = hComDB as PCOMDB;

    /* Wait for the mutex */
    WaitForSingleObject((*pComDB).hMutex, INFINITE as DWORD);

    'done: {
        /* Get the required bitmap size */
        lError = RegQueryValueExW(
            (*pComDB).hKey,
            RUSTCONV_WSTR_06AF8F617F65.as_ptr(),
            core::ptr::null_mut(),
            &mut dwType,
            core::ptr::null_mut(),
            &mut dwSize,
        ) as LONG;
        if lError != ERROR_SUCCESS as LONG {
            ERR!("Failed to query the bitmap size!\n");
            break 'done;
        }

        /* Allocate the bitmap */
        pBitmap = HeapAlloc(
            GetProcessHeap(),
            HEAP_ZERO_MEMORY as DWORD,
            (dwSize) as SIZE_T,
        ) as PBYTE;
        if pBitmap.is_null() {
            ERR!("Failed to allocate the bitmap!\n");
            lError = ERROR_NOT_ENOUGH_MEMORY as LONG;
            break 'done;
        }

        /* Read the bitmap */
        lError = RegQueryValueExW(
            (*pComDB).hKey,
            RUSTCONV_WSTR_06AF8F617F65.as_ptr(),
            core::ptr::null_mut(),
            &mut dwType,
            pBitmap,
            &mut dwSize,
        ) as LONG;
        if lError != ERROR_SUCCESS as LONG {
            break 'done;
        }

        if Buffer.is_null() {
            *MaxPortsReported = dwSize.wrapping_mul(BITS_PER_BYTE as DWORD);
        } else {
            if ReportType == CDB_REPORT_BITS as DWORD {
                /* Clear the buffer */
                memset(
                    Buffer as *mut core::ffi::c_void,
                    0 as core::ffi::c_int,
                    BufferSize as usize,
                );

                memcpy(
                    Buffer as *mut core::ffi::c_void,
                    pBitmap as *const core::ffi::c_void,
                    min!(dwSize, BufferSize) as usize,
                );

                if !MaxPortsReported.is_null() {
                    *MaxPortsReported = min!(dwSize, BufferSize).wrapping_mul(BITS_PER_BYTE as DWORD);
                }
            } else if ReportType == CDB_REPORT_BYTES as DWORD {
                /* Clear the buffer */
                memset(
                    Buffer as *mut core::ffi::c_void,
                    0 as core::ffi::c_int,
                    BufferSize as usize,
                );

                dwBitIndex = 0 as DWORD;
                while dwBitIndex
                    < min!(dwSize.wrapping_mul(BITS_PER_BYTE as DWORD), BufferSize)
                {
                    /* Calculate the byte index and a mask for the affected bit */
                    dwByteIndex = dwBitIndex / (BITS_PER_BYTE as DWORD);
                    cMask = (1
                        << (dwBitIndex % (BITS_PER_BYTE as DWORD))) as BYTE;

                    if (*pBitmap.add(dwByteIndex as usize) & cMask) != (0 as BYTE) {
                        *Buffer.add(dwBitIndex as usize) = 1 as BYTE;
                    }
                    dwBitIndex = dwBitIndex.wrapping_add(1);
                }
            }
        }
    }

    /* done: */
    /* Release the mutex */
    ReleaseMutex((*pComDB).hMutex);

    /* Release the bitmap */
    HeapFree(
        GetProcessHeap(),
        0 as DWORD,
        pBitmap as *mut core::ffi::c_void,
    );

    return lError;
}

#[no_mangle]
pub unsafe extern "system" fn ComDBOpen(phComDB: *mut HCOMDB) -> LONG {
    let mut pComDB: PCOMDB = core::ptr::null_mut();
    let mut dwDisposition: DWORD = 0;
    let mut dwType: DWORD = 0;
    let mut dwSize: DWORD = 0;
    let mut pBitmap: PBYTE = core::ptr::null_mut();
    let mut lError: LONG = 0;

    /* Allocate a new database */
    pComDB = HeapAlloc(GetProcessHeap(),
                       HEAP_ZERO_MEMORY,
                       core::mem::size_of::<COMDB>() as SIZE_T) as PCOMDB;
    if pComDB.is_null() {
        ERR!("Failed to allocate the database!\n");
        *phComDB = HCOMDB_INVALID_HANDLE_VALUE;
        return (ERROR_ACCESS_DENIED) as LONG;
    }

    /* Create a mutex to protect the database */
    (*pComDB).hMutex = CreateMutexW(core::ptr::null_mut(),
                                  (FALSE) as BOOL,
                                  RUSTCONV_WSTR_6180B91436ED.as_ptr());
    if (*pComDB).hMutex.is_null() {
        ERR!("Failed to create the mutex!\n");
        HeapFree(GetProcessHeap(), 0, pComDB as PVOID);
        *phComDB = HCOMDB_INVALID_HANDLE_VALUE;
        return (ERROR_ACCESS_DENIED) as LONG;
    }

    /* Wait for the mutex */
    WaitForSingleObject((*pComDB).hMutex, INFINITE);

    /* Create or open the database key */
    lError = RegCreateKeyExW(HKEY_LOCAL_MACHINE,
                             RUSTCONV_WSTR_8A3038F3E9A7.as_ptr(),
                             0,
                             core::ptr::null_mut(),
                             0,
                             KEY_ALL_ACCESS,
                             core::ptr::null_mut(),
                             core::ptr::addr_of_mut!((*pComDB).hKey),
                             core::ptr::addr_of_mut!(dwDisposition)) as LONG;

    'done: {
        if lError != (ERROR_SUCCESS as i32) {
            break 'done;
        }

        /* Get the required bitmap size */
        lError = RegQueryValueExW((*pComDB).hKey,
                              RUSTCONV_WSTR_06AF8F617F65.as_ptr(),
                              core::ptr::null_mut(),
                              core::ptr::addr_of_mut!(dwType),
                              core::ptr::null_mut(),
                              core::ptr::addr_of_mut!(dwSize)) as LONG;
        if lError == (ERROR_FILE_NOT_FOUND as i32) {
            /* Allocate a new bitmap */
            dwSize = (COMDB_MIN_PORTS_ARBITRATED / BITS_PER_BYTE) as DWORD;
            pBitmap = HeapAlloc(GetProcessHeap(),
                                HEAP_ZERO_MEMORY,
                                dwSize as SIZE_T) as PBYTE;
            if pBitmap.is_null() {
                ERR!("Failed to allocate the bitmap!\n");
                lError = (ERROR_ACCESS_DENIED as i32);
                break 'done;
            }

            /* Write the bitmap to the registry */
            lError = RegSetValueExW((*pComDB).hKey,
                                RUSTCONV_WSTR_06AF8F617F65.as_ptr(),
                                0,
                                REG_BINARY,
                                pBitmap as *const BYTE,
                                dwSize) as LONG;

            HeapFree(GetProcessHeap(), 0, pBitmap as PVOID);
        }
    }

    /* Release the mutex */
    ReleaseMutex((*pComDB).hMutex);

    if lError != (ERROR_SUCCESS as i32) {
        /* Clean up in case of failure */
        if !(*pComDB).hKey.is_null() {
            RegCloseKey((*pComDB).hKey);
        }

        if !(*pComDB).hMutex.is_null() {
            CloseHandle((*pComDB).hMutex);
        }

        HeapFree(GetProcessHeap(), 0, pComDB as PVOID);

        *phComDB = HCOMDB_INVALID_HANDLE_VALUE;
    } else {
        /* Return the database handle */
        *phComDB = pComDB as HCOMDB;
    }

    return lError;
}

#[no_mangle]
pub unsafe extern "system" fn ComDBReleasePort(hComDB: HCOMDB, ComNumber: DWORD) -> LONG {
    let mut pComDB: PCOMDB = core::ptr::null_mut();
    let mut dwByteIndex: DWORD = 0;
    let mut dwBitIndex: DWORD = 0;
    let mut dwType: DWORD = 0;
    let mut dwSize: DWORD = 0;
    let mut pBitmap: PBYTE = core::ptr::null_mut();
    let mut cMask: BYTE = 0;
    let mut lError: LONG = 0;

    if hComDB == (INVALID_HANDLE_VALUE).cast()
        || ComNumber == 0
        || ComNumber > COMDB_MAX_PORTS_ARBITRATED
    {
        return (ERROR_INVALID_PARAMETER) as LONG;
    }

    pComDB = hComDB as PCOMDB;

    /* Wait for the mutex */
    WaitForSingleObject((*pComDB).hMutex, INFINITE);

    'done: {
        /* Get the required bitmap size */
        lError = RegQueryValueExW(
            (*pComDB).hKey,
            RUSTCONV_WSTR_06AF8F617F65.as_ptr(),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(dwType),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(dwSize),
        ) as LONG;
        if lError != (ERROR_SUCCESS as i32) {
            ERR!("Failed to query the bitmap size!\n");
            break 'done;
        }

        /* Allocate the bitmap */
        pBitmap = HeapAlloc(
            GetProcessHeap(),
            HEAP_ZERO_MEMORY,
            dwSize as SIZE_T,
        ) as PBYTE;
        if pBitmap.is_null() {
            ERR!("Failed to allocate the bitmap!\n");
            lError = (ERROR_NOT_ENOUGH_MEMORY as i32);
            break 'done;
        }

        /* Read the bitmap */
        lError = RegQueryValueExW(
            (*pComDB).hKey,
            RUSTCONV_WSTR_06AF8F617F65.as_ptr(),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(dwType),
            pBitmap,
            core::ptr::addr_of_mut!(dwSize),
        ) as LONG;
        if lError != (ERROR_SUCCESS as i32) {
            break 'done;
        }

        /* Get the bit index */
        dwBitIndex = ComNumber.wrapping_sub(1);

        /* Check if the bit to set fits into the bitmap */
        if dwBitIndex >= dwSize.wrapping_mul(BITS_PER_BYTE) {
            lError = (ERROR_INVALID_PARAMETER as i32);
            break 'done;
        }

        /* Calculate the byte index and a mask for the affected bit */
        dwByteIndex = dwBitIndex / BITS_PER_BYTE;
        cMask = (1 << (dwBitIndex % BITS_PER_BYTE)) as BYTE;

        /* Release the port */
        *pBitmap.add(dwByteIndex as usize) &= !cMask;

        lError = RegSetValueExW(
            (*pComDB).hKey,
            RUSTCONV_WSTR_06AF8F617F65.as_ptr(),
            0,
            REG_BINARY,
            pBitmap,
            dwSize,
        ) as LONG;
    }

    /* Release the mutex */
    ReleaseMutex((*pComDB).hMutex);

    /* Release the bitmap */
    if !pBitmap.is_null() {
        HeapFree(GetProcessHeap(), 0, pBitmap as PVOID);
    }

    return lError;
}

#[no_mangle]
pub unsafe extern "system" fn ComDBResizeDatabase(hComDB: HCOMDB, NewSize: DWORD) -> LONG {
    let mut pComDB: PCOMDB;
    let mut pBitmap: PBYTE = core::ptr::null_mut();
    let mut dwSize: DWORD;
    let mut dwNewSize: DWORD = 0;
    let mut dwType: DWORD = 0;
    let mut lError: LONG;

    if hComDB == INVALID_HANDLE_VALUE as HCOMDB
        || hComDB.is_null()
        || (NewSize & (BITMAP_SIZE_INVALID_BITS as DWORD)) != 0
    {
        return ERROR_INVALID_PARAMETER as LONG;
    }

    pComDB = hComDB as PCOMDB;

    /* Wait for the mutex */
    WaitForSingleObject((*pComDB).hMutex, INFINITE as DWORD);

    dwSize = 0;

    /* Get the required bitmap size */
    lError = RegQueryValueExW(
        (*pComDB).hKey,
        RUSTCONV_WSTR_06AF8F617F65.as_ptr(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(dwType),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(dwSize),
    ) as LONG;

    'done: {
        if lError != ERROR_SUCCESS as LONG {
            break 'done;
        }

        /* Check the size limits */
        if NewSize > COMDB_MAX_PORTS_ARBITRATED as DWORD
            || NewSize <= dwSize.wrapping_mul(BITS_PER_BYTE as DWORD)
        {
            lError = ERROR_BAD_LENGTH as LONG;
            break 'done;
        }

        /* Calculate the new bitmap size */
        dwNewSize = NewSize / (BITS_PER_BYTE as DWORD);

        /* Allocate the new bitmap */
        pBitmap = HeapAlloc(
            GetProcessHeap(),
            HEAP_ZERO_MEMORY as DWORD,
            dwSize as SIZE_T,
        ) as PBYTE;
        if pBitmap.is_null() {
            ERR!("Failed to allocate the bitmap!\n");
            lError = ERROR_ACCESS_DENIED as LONG;
            break 'done;
        }

        /* Read the current bitmap */
        lError = RegQueryValueExW(
            (*pComDB).hKey,
            RUSTCONV_WSTR_06AF8F617F65.as_ptr(),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(dwType),
            pBitmap,
            core::ptr::addr_of_mut!(dwSize),
        ) as LONG;
        if lError != ERROR_SUCCESS as LONG {
            break 'done;
        }

        /* Write the new bitmap */
        lError = RegSetValueExW(
            (*pComDB).hKey,
            RUSTCONV_WSTR_06AF8F617F65.as_ptr(),
            0 as DWORD,
            REG_BINARY as DWORD,
            pBitmap as *const BYTE,
            dwNewSize,
        ) as LONG;
    }

    /* Release the mutex */
    ReleaseMutex((*pComDB).hMutex);

    if !pBitmap.is_null() {
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            pBitmap as PVOID,
        );
    }

    return lError;
}
