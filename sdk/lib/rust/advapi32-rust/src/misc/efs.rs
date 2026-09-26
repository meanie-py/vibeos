use crate::prelude::*;
use core::ffi::c_void;

extern "system" {
}

// ---- file-scope preamble (translated) ----
// No translatable file-scope declarations.

#[no_mangle]
pub unsafe extern "system" fn AddUsersToEncryptedFile(
    lpcwstr: LPCWSTR,
    pencryption_certificate_list: PENCRYPTION_CERTIFICATE_LIST,
) -> DWORD {
    FIXME!("%s() not implemented!\n", b"AddUsersToEncryptedFile\0".as_ptr() as LPCSTR);
    ERROR_CALL_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn DecryptFileA(lpFileName: LPCSTR, dwReserved: u32) -> i32 {
    let mut FileName: _UNICODE_STRING = core::mem::zeroed();

    if RtlCreateUnicodeStringFromAsciiz(&mut FileName, lpFileName) == 0 {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }

    let ret = DecryptFileW(FileName.Buffer, dwReserved);

    if !FileName.Buffer.is_null() {
        RtlFreeUnicodeString(&mut FileName);
    }
    ret
}

#[no_mangle]
pub unsafe extern "system" fn DecryptFileW(lpFileName: LPCWSTR, dwReserved: u32) -> i32 {
    1
}

#[no_mangle]
pub unsafe extern "system" fn EncryptFileA(lpFileName: LPCSTR) -> BOOL {
    let mut FileName: _UNICODE_STRING = core::mem::zeroed();

    if RtlCreateUnicodeStringFromAsciiz(&mut FileName, lpFileName) == 0 {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }

    let ret = EncryptFileW(FileName.Buffer);

    if !FileName.Buffer.is_null() {
        RtlFreeUnicodeString(&mut FileName);
    }
    ret
}

#[no_mangle]
pub unsafe extern "system" fn EncryptFileW(lpFileName: LPCWSTR) -> BOOL {
    let _ = lpFileName;
    1
}

#[no_mangle]
pub unsafe extern "system" fn EncryptionDisable(DirPath: LPCWSTR, Disable: BOOL) -> BOOL {
    FIXME!("%s() not implemented!\n", b"EncryptionDisable\0".as_ptr() as LPCSTR);
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    FALSE
}

#[no_mangle]
pub unsafe extern "system" fn FileEncryptionStatusA(
    lpFileName: LPCSTR,
    lpStatus: LPDWORD,
) -> BOOL {
    let mut FileName: _UNICODE_STRING = core::mem::zeroed();
    let mut ret: BOOL = 0;

    FileName.Buffer = core::ptr::null_mut();

    'cleanup: {
        if RtlCreateUnicodeStringFromAsciiz(&mut FileName, lpFileName) == 0 {
            SetLastError(ERROR_NOT_ENOUGH_MEMORY);
            break 'cleanup;
        }

        ret = FileEncryptionStatusW(FileName.Buffer, lpStatus);
    }

    if !FileName.Buffer.is_null() {
        RtlFreeUnicodeString(&mut FileName);
    }

    ret
}

#[no_mangle]
pub unsafe extern "system" fn FileEncryptionStatusW(
    lpFileName: LPCWSTR,
    lpStatus: LPDWORD,
) -> BOOL {
    if lpStatus.is_null() {
        return FALSE;
    }

    *lpStatus = FILE_SYSTEM_NOT_SUPPORT;

    TRUE
}

#[no_mangle]
pub unsafe extern "system" fn FreeEncryptionCertificateHashList(
    _pencryption_certificate_hash_list: PENCRYPTION_CERTIFICATE_HASH_LIST,
) {
}

#[no_mangle]
pub unsafe extern "system" fn QueryRecoveryAgentsOnEncryptedFile(
    lpctstr: LPCWSTR,
    pencryption_certificate_hash_list: *mut PENCRYPTION_CERTIFICATE_HASH_LIST,
) -> DWORD {
    ERROR_CALL_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn QueryUsersOnEncryptedFile(
    lpctstr: LPCWSTR,
    pencryption_certificate_hash_list: *mut PENCRYPTION_CERTIFICATE_HASH_LIST,
) -> DWORD {
    ERROR_CALL_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn RemoveUsersFromEncryptedFile(
    lpcwstr: LPCWSTR,
    pencryption_certificate_hash_list: PENCRYPTION_CERTIFICATE_HASH_LIST,
) -> DWORD {
    FIXME!("%s() not implemented!\n", b"RemoveUsersFromEncryptedFile\0".as_ptr() as LPCSTR);
    ERROR_CALL_NOT_IMPLEMENTED
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS system libraries
//  * FILE:            dlls/win32/advapi32/misc/efs.c
//  * PURPOSE:         Encrypted File System support
//  * PROGRAMMER:      Christoph_vW
//  */
//
// #include <advapi32.h>
//
// #include <winefs.h>
//
// WINE_DEFAULT_DEBUG_CHANNEL(advapi);
//
// /*
//  * @unimplemented
//  */
// DWORD WINAPI
// AddUsersToEncryptedFile(LPCWSTR lpcwstr,
//                         PENCRYPTION_CERTIFICATE_LIST pencryption_certificate_list)
// {
//     FIXME("%s() not implemented!\n", __FUNCTION__);
//     return ERROR_CALL_NOT_IMPLEMENTED;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL WINAPI
// DecryptFileA(LPCSTR lpFileName,
//              DWORD dwReserved)
// {
//     UNICODE_STRING FileName;
//     BOOL ret;
//
//     if (!RtlCreateUnicodeStringFromAsciiz(&FileName, lpFileName))
//     {
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         return FALSE;
//     }
//
//     ret = DecryptFileW(FileName.Buffer, dwReserved);
//
//     if (FileName.Buffer != NULL)
//         RtlFreeUnicodeString(&FileName);
//     return ret;
// }
//
//
// /*
//  * @unimplemented
//  */
// BOOL WINAPI
// DecryptFileW(LPCWSTR lpFileName,
//              DWORD dwReserved)
// {
//     FIXME("%s(%S) not implemented!\n", __FUNCTION__, lpFileName);
//     return TRUE;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL WINAPI
// EncryptFileA(LPCSTR lpFileName)
// {
//     UNICODE_STRING FileName;
//     BOOL ret;
//
//     if (!RtlCreateUnicodeStringFromAsciiz(&FileName, lpFileName))
//     {
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         return FALSE;
//     }
//
//     ret = EncryptFileW(FileName.Buffer);
//
//     if (FileName.Buffer != NULL)
//         RtlFreeUnicodeString(&FileName);
//     return ret;
// }
//
//
// /*
//  * @unimplemented
//  */
// BOOL WINAPI
// EncryptFileW(LPCWSTR lpFileName)
// {
//     FIXME("%s() not implemented!\n", __FUNCTION__);
//     return TRUE;
// }
//
//
// /*
//  * @unimplemented
//  */
// BOOL WINAPI
// EncryptionDisable(LPCWSTR DirPath,
//                   BOOL Disable)
// {
//     FIXME("%s() not implemented!\n", __FUNCTION__);
//     SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
//     return FALSE;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL WINAPI
// FileEncryptionStatusA(LPCSTR lpFileName,
//                       LPDWORD lpStatus)
// {
//     UNICODE_STRING FileName;
//     BOOL ret = FALSE;
//
//     TRACE("(%s, %p)\n", lpFileName, lpStatus);
//
//     FileName.Buffer = NULL;
//
//     if (!RtlCreateUnicodeStringFromAsciiz(&FileName, lpFileName))
//     {
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         goto cleanup;
//     }
//
//     ret = FileEncryptionStatusW(FileName.Buffer, lpStatus);
//
// cleanup:
//     if (FileName.Buffer != NULL)
//         RtlFreeUnicodeString(&FileName);
//
//     return ret;
// }
//
// /*
//  * @unimplemented
//  */
// BOOL WINAPI
// FileEncryptionStatusW(LPCWSTR lpFileName,
//                       LPDWORD lpStatus)
// {
//     FIXME("%s(%S) not implemented!\n", __FUNCTION__, lpFileName);
//
//     if (!lpStatus)
//         return FALSE;
//
//     *lpStatus = FILE_SYSTEM_NOT_SUPPORT;
//
//     return TRUE;
// }
//
//
// /*
//  * @unimplemented
//  */
// VOID WINAPI
// FreeEncryptionCertificateHashList(PENCRYPTION_CERTIFICATE_HASH_LIST pencryption_certificate_hash_list)
// {
//     FIXME("%s() not implemented!\n", __FUNCTION__);
//     return;
// }
//
//
// /*
//  * @unimplemented
//  */
// DWORD WINAPI
// QueryRecoveryAgentsOnEncryptedFile(LPCWSTR lpctstr,
//                                    PENCRYPTION_CERTIFICATE_HASH_LIST* pencryption_certificate_hash_list)
// {
//     FIXME("%s() not implemented!\n", __FUNCTION__);
//     return ERROR_CALL_NOT_IMPLEMENTED;
// }
//
//
// /*
//  * @unimplemented
//  */
// DWORD WINAPI
// QueryUsersOnEncryptedFile(LPCWSTR lpctstr,
//                           PENCRYPTION_CERTIFICATE_HASH_LIST* pencryption_certificate_hash_list)
// {
//     FIXME("%s() not implemented!\n", __FUNCTION__);
//     return ERROR_CALL_NOT_IMPLEMENTED;
// }
//
//
// /*
//  * @unimplemented
//  */
// DWORD WINAPI
// RemoveUsersFromEncryptedFile(LPCWSTR lpcwstr,
//                              PENCRYPTION_CERTIFICATE_HASH_LIST pencryption_certificate_hash_list)
// {
//     FIXME("%s() not implemented!\n", __FUNCTION__);
//     return ERROR_CALL_NOT_IMPLEMENTED;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
