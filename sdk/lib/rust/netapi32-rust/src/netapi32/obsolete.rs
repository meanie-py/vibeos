use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn NetAlertRaise(
    AlertEventName: LPCWSTR,
    Buffer: LPVOID,
    BufferSize: DWORD,
) -> DWORD {
    return ERROR_NOT_SUPPORTED;
}

#[no_mangle]
pub unsafe extern "system" fn NetAlertRaiseEx(
    AlertEventName: LPCWSTR,
    VariableInfo: LPVOID,
    VariableInfoSize: DWORD,
    ServiceName: LPCWSTR,
) -> DWORD {
    ERROR_NOT_SUPPORTED
}

#[no_mangle]
pub unsafe extern "system" fn NetAuditClear(
    server: LPCWSTR,
    backupfile: LPCWSTR,
    service: LPCWSTR,
) -> u32 {
    ERROR_NOT_SUPPORTED
}

#[no_mangle]
pub unsafe extern "system" fn NetAuditRead(
    server: LPCWSTR,
    service: LPCWSTR,
    auditloghandle: LPHLOG,
    offset: DWORD,
    reserved1: LPDWORD,
    reserved2: DWORD,
    offsetflag: DWORD,
    bufptr: *mut LPBYTE,
    prefmaxlen: DWORD,
    bytesread: LPDWORD,
    totalavailable: LPDWORD,
) -> DWORD {
    ERROR_NOT_SUPPORTED as DWORD
}

#[no_mangle]
pub unsafe extern "system" fn NetAuditWrite(
    type_: u32,
    buf: *mut u8,
    numbytes: u32,
    service: *const u16,
    reserved: *mut u8,
) -> u32 {
    ERROR_NOT_SUPPORTED
}
#[no_mangle]
pub unsafe extern "system" fn NetConfigGet(
    server: LPCWSTR,
    component: LPCWSTR,
    parameter: LPCWSTR,
    bufptr: *mut LPBYTE,
) -> NET_API_STATUS {
    return ERROR_NOT_SUPPORTED;
}

#[no_mangle]
pub unsafe extern "system" fn NetConfigGetAll(
    server: LPCWSTR,
    component: LPCWSTR,
    bufptr: *mut LPBYTE,
) -> u32 {
    ERROR_NOT_SUPPORTED as u32
}

#[no_mangle]
pub unsafe extern "system" fn NetConfigSet(
    server: LPCWSTR,
    reserved1: LPCWSTR,
    component: LPCWSTR,
    level: DWORD,
    reserved2: DWORD,
    buf: LPBYTE,
    reserved3: DWORD,
) -> DWORD {
    ERROR_NOT_SUPPORTED
}

#[no_mangle]
pub unsafe extern "system" fn NetErrorLogClear(
    UncServerName: LPCWSTR,
    BackupFile: LPCWSTR,
    Reserved: LPBYTE,
) -> u32 {
    ERROR_NOT_SUPPORTED
}

#[no_mangle]
pub unsafe extern "system" fn NetErrorLogRead(
    _UncServerName: LPCWSTR,
    _Reserved1: LPWSTR,
    _ErrorLogHandle: LPHLOG,
    _Offset: DWORD,
    _Reserved2: LPDWORD,
    _Reserved3: DWORD,
    _OffsetFlag: DWORD,
    _BufPtr: *mut LPBYTE,
    _PrefMaxSize: DWORD,
    _BytesRead: LPDWORD,
    _TotalAvailable: LPDWORD,
) -> DWORD {
    ERROR_NOT_SUPPORTED as DWORD
}

#[no_mangle]
pub unsafe extern "system" fn NetErrorLogWrite(
    Reserved1: LPBYTE,
    Code: DWORD,
    Component: LPCWSTR,
    Buffer: LPBYTE,
    NumBytes: DWORD,
    MsgBuf: LPBYTE,
    StrCount: DWORD,
    Reserved2: LPBYTE,
) -> DWORD {
    ERROR_NOT_SUPPORTED
}

#[no_mangle]
pub unsafe extern "system" fn NetMessageBufferSend(
    _servername: LPCWSTR,
    _msgname: LPCWSTR,
    _fromname: LPCWSTR,
    _buf: LPBYTE,
    _buflen: DWORD,
) -> u32 {
    ERROR_NOT_SUPPORTED as u32
}

#[no_mangle]
pub unsafe extern "system" fn NetMessageNameAdd(servername: LPCWSTR, msgname: LPCWSTR) -> u32 {
    ERROR_NOT_SUPPORTED
}

#[no_mangle]
pub unsafe extern "system" fn NetMessageNameDel(
    servername: LPCWSTR,
    msgname: LPCWSTR,
) -> DWORD {
    ERROR_NOT_SUPPORTED as DWORD
}

#[no_mangle]
pub unsafe extern "system" fn NetMessageNameEnum(
    _servername: LPCWSTR,
    _level: DWORD,
    _bufptr: *mut LPBYTE,
    _prefmaxlen: DWORD,
    _entriesread: LPDWORD,
    _totalentries: LPDWORD,
    _resume_handle: LPDWORD,
) -> DWORD {
    ERROR_NOT_SUPPORTED as DWORD
}

#[no_mangle]
pub unsafe extern "system" fn NetMessageNameGetInfo(
    servername: LPCWSTR,
    msgname: LPCWSTR,
    level: u32,
    bufptr: *mut *mut u8,
) -> u32 {
    ERROR_NOT_SUPPORTED
}

#[no_mangle]
pub unsafe extern "system" fn NetReplExportDirAdd(
    servername: LPCWSTR,
    level: DWORD,
    buf: LPBYTE,
    parm_err: LPDWORD,
) -> DWORD {
    ERROR_NOT_SUPPORTED as DWORD
}

#[no_mangle]
pub unsafe extern "system" fn NetReplExportDirDel(
    servername: LPCWSTR,
    dirname: LPCWSTR,
) -> u32 {
    ERROR_NOT_SUPPORTED as u32
}

#[no_mangle]
pub unsafe extern "system" fn NetReplExportDirEnum(
    servername: LPCWSTR,
    level: DWORD,
    bufptr: *mut LPBYTE,
    prefmaxlen: DWORD,
    entriesread: LPDWORD,
    totalentries: LPDWORD,
    resumehandle: LPDWORD,
) -> DWORD {
    ERROR_NOT_SUPPORTED
}

#[no_mangle]
pub unsafe extern "system" fn NetReplExportDirGetInfo(
    servername: LPCWSTR,
    dirname: LPCWSTR,
    level: u32,
    bufptr: *mut *mut u8,
) -> u32 {
    ERROR_NOT_SUPPORTED
}

#[no_mangle]
pub unsafe extern "system" fn NetReplExportDirLock(
    servername: LPCWSTR,
    dirname: LPCWSTR,
) -> u32 {
    ERROR_NOT_SUPPORTED
}

#[no_mangle]
pub unsafe extern "system" fn NetReplExportDirSetInfo(
    servername: LPCWSTR,
    dirname: LPCWSTR,
    level: u32,
    buf: LPBYTE,
    parm_err: LPDWORD,
) -> u32 {
    ERROR_NOT_SUPPORTED
}

#[no_mangle]
pub unsafe extern "system" fn NetReplExportDirUnlock(
    _servername: LPCWSTR,
    _dirname: LPCWSTR,
    _unlockforce: u32,
) -> u32 {
    ERROR_NOT_SUPPORTED
}

#[no_mangle]
pub unsafe extern "system" fn NetReplGetInfo(
    servername: LPCWSTR,
    level: DWORD,
    bufptr: *mut LPBYTE,
) -> DWORD {
    ERROR_NOT_SUPPORTED
}

#[no_mangle]
pub unsafe extern "system" fn NetReplImportDirAdd(
    servername: LPCWSTR,
    level: DWORD,
    buf: LPBYTE,
    parm_err: LPDWORD,
) -> DWORD {
    return ERROR_NOT_SUPPORTED as DWORD;
}

#[no_mangle]
pub unsafe extern "system" fn NetReplImportDirDel(
    servername: LPCWSTR,
    dirname: LPCWSTR,
) -> u32 {
    ERROR_NOT_SUPPORTED as u32
}

#[no_mangle]
pub unsafe extern "system" fn NetReplImportDirEnum(
    servername: LPCWSTR,
    level: DWORD,
    bufptr: *mut LPBYTE,
    prefmaxlen: DWORD,
    entriesread: LPDWORD,
    totalentries: LPDWORD,
    resumehandle: LPDWORD,
) -> DWORD {
    ERROR_NOT_SUPPORTED
}

#[no_mangle]
pub unsafe extern "system" fn NetReplImportDirGetInfo(
    servername: LPCWSTR,
    dirname: LPCWSTR,
    level: u32,
    bufptr: *mut LPBYTE,
) -> u32 {
    ERROR_NOT_SUPPORTED
}

#[no_mangle]
pub unsafe extern "system" fn NetReplImportDirLock(
    servername: LPCWSTR,
    dirname: LPCWSTR,
) -> u32 {
    ERROR_NOT_SUPPORTED
}

#[no_mangle]
pub unsafe extern "system" fn NetReplImportDirUnlock(
    _servername: LPCWSTR,
    _dirname: LPCWSTR,
    _unlockforce: DWORD,
) -> DWORD {
    ERROR_NOT_SUPPORTED
}

#[no_mangle]
pub unsafe extern "system" fn NetReplSetInfo(
    _servername: LPCWSTR,
    _level: u32,
    _buf: *const u8,
    _parm_err: *mut u32,
) -> u32 {
    ERROR_NOT_SUPPORTED as u32
}

#[no_mangle]
pub unsafe extern "system" fn NetServiceControl(
    servername: LPCWSTR,
    service: LPCWSTR,
    opcode: DWORD,
    arg: DWORD,
    bufptr: *mut LPBYTE,
) -> DWORD {
    ERROR_NOT_SUPPORTED
}

#[no_mangle]
pub unsafe extern "system" fn NetServiceEnum(
    servername: *const u16,
    level: u32,
    bufptr: *mut *mut u8,
    prefmaxlen: u32,
    entriesread: *mut u32,
    totalentries: *mut u32,
    resume_handle: *mut u32,
) -> u32 {
    ERROR_NOT_SUPPORTED
}

#[no_mangle]
pub unsafe extern "system" fn NetServiceGetInfo(
    servername: LPCWSTR,
    service: LPCWSTR,
    level: DWORD,
    bufptr: *mut LPBYTE,
) -> DWORD {
    ERROR_NOT_SUPPORTED
}

#[no_mangle]
pub unsafe extern "system" fn NetServiceInstall(
    servername: LPCWSTR,
    service: LPCWSTR,
    argc: DWORD,
    argv: *mut LPCWSTR,
    bufptr: *mut LPBYTE,
) -> DWORD {
    return ERROR_NOT_SUPPORTED as DWORD;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         NetAPI DLL
//  * FILE:            dll/win32/netapi32/obsolete.c
//  * PURPOSE:         Obsolete functions
//  * PROGRAMMERS:     Eric Kohl (eric.kohl@reactos.org)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include "netapi32.h"
//
// #include <lmalert.h>
// #include <lmaudit.h>
// #include <lmconfig.h>
// #include <lmerrlog.h>
// #include <lmmsg.h>
// #include <lmrepl.h>
// #include <lmsvc.h>
//
// /* FUNCTIONS *****************************************************************/
//
// NET_API_STATUS
// WINAPI
// NetAlertRaise(
//      LPCWSTR AlertEventName,
//      LPVOID Buffer,
//      DWORD BufferSize)
// {
//     return ERROR_NOT_SUPPORTED;
// }
//
//
// NET_API_STATUS
// WINAPI
// NetAlertRaiseEx(
//      LPCWSTR AlertEventName,
//      LPVOID VariableInfo,
//      DWORD VariableInfoSize,
//      LPCWSTR ServiceName)
// {
//     return ERROR_NOT_SUPPORTED;
// }
//
//
// NET_API_STATUS
// WINAPI
// NetAuditClear(
//      LPCWSTR server,
//      LPCWSTR backupfile,
//      LPCWSTR service)
// {
//     return ERROR_NOT_SUPPORTED;
// }
//
//
// NET_API_STATUS
// WINAPI
// NetAuditRead(
//      LPCWSTR server,
//      LPCWSTR service,
//      LPHLOG auditloghandle,
//      DWORD offset,
//      LPDWORD reserved1,
//      DWORD reserved2,
//      DWORD offsetflag,
//      LPBYTE *bufptr,
//      DWORD prefmaxlen,
//      LPDWORD bytesread,
//      LPDWORD totalavailable)
// {
//     return ERROR_NOT_SUPPORTED;
// }
//
//
// NET_API_STATUS
// WINAPI
// NetAuditWrite(
//      DWORD type,
//      LPBYTE buf,
//      DWORD numbytes,
//      LPCWSTR service,
//      LPBYTE reserved)
// {
//     return ERROR_NOT_SUPPORTED;
// }
//
//
// NET_API_STATUS
// WINAPI
// NetConfigGet(
//      LPCWSTR server,
//      LPCWSTR component,
//      LPCWSTR parameter,
//      LPBYTE *bufptr)
// {
//     return ERROR_NOT_SUPPORTED;
// }
//
//
// NET_API_STATUS
// WINAPI
// NetConfigGetAll(
//      LPCWSTR server,
//      LPCWSTR component,
//      LPBYTE *bufptr)
// {
//     return ERROR_NOT_SUPPORTED;
// }
//
//
// NET_API_STATUS
// WINAPI
// NetConfigSet(
//      LPCWSTR server,
//      LPCWSTR reserved1,
//      LPCWSTR component,
//      DWORD level,
//      DWORD reserved2,
//      LPBYTE buf,
//      DWORD reserved3)
// {
//     return ERROR_NOT_SUPPORTED;
// }
//
//
// NET_API_STATUS
// WINAPI
// NetErrorLogClear(
//      LPCWSTR UncServerName,
//      LPCWSTR BackupFile,
//      LPBYTE Reserved)
// {
//     return ERROR_NOT_SUPPORTED;
// }
//
//
// NET_API_STATUS
// WINAPI
// NetErrorLogRead(
//      LPCWSTR UncServerName,
//      LPWSTR Reserved1,
//      LPHLOG ErrorLogHandle,
//      DWORD Offset,
//      LPDWORD Reserved2,
//      DWORD Reserved3,
//      DWORD OffsetFlag,
//      LPBYTE *BufPtr,
//      DWORD PrefMaxSize,
//      LPDWORD BytesRead,
//      LPDWORD TotalAvailable)
// {
//     return ERROR_NOT_SUPPORTED;
// }
//
//
// NET_API_STATUS
// WINAPI
// NetErrorLogWrite(
//      LPBYTE Reserved1,
//      DWORD Code,
//      LPCWSTR Component,
//      LPBYTE Buffer,
//      DWORD NumBytes,
//      LPBYTE MsgBuf,
//      DWORD StrCount,
//      LPBYTE Reserved2)
// {
//     return ERROR_NOT_SUPPORTED;
// }
//
//
// NET_API_STATUS
// WINAPI
// NetMessageBufferSend(
//      LPCWSTR servername,
//      LPCWSTR msgname,
//      LPCWSTR fromname,
//      LPBYTE buf,
//      DWORD buflen)
// {
//     return ERROR_NOT_SUPPORTED;
// }
//
//
// NET_API_STATUS
// WINAPI
// NetMessageNameAdd(
//      LPCWSTR servername,
//      LPCWSTR msgname)
// {
//     return ERROR_NOT_SUPPORTED;
// }
//
//
// NET_API_STATUS
// WINAPI
// NetMessageNameDel(
//      LPCWSTR servername,
//      LPCWSTR msgname)
// {
//     return ERROR_NOT_SUPPORTED;
// }
//
//
// NET_API_STATUS
// WINAPI
// NetMessageNameEnum(
//      LPCWSTR servername,
//      DWORD level,
//      LPBYTE *bufptr,
//      DWORD prefmaxlen,
//      LPDWORD entriesread,
//      LPDWORD totalentries,
//      LPDWORD resume_handle)
// {
//     return ERROR_NOT_SUPPORTED;
// }
//
//
// NET_API_STATUS
// WINAPI
// NetMessageNameGetInfo(
//      LPCWSTR servername,
//      LPCWSTR msgname,
//      DWORD level,
//      LPBYTE *bufptr)
// {
//     return ERROR_NOT_SUPPORTED;
// }
//
//
// NET_API_STATUS
// WINAPI
// NetReplExportDirAdd(
//      LPCWSTR servername,
//      DWORD level,
//      const LPBYTE buf,
//      LPDWORD parm_err)
// {
//     return ERROR_NOT_SUPPORTED;
// }
//
//
// NET_API_STATUS
// WINAPI
// NetReplExportDirDel(
//      LPCWSTR servername,
//      LPCWSTR dirname)
// {
//     return ERROR_NOT_SUPPORTED;
// }
//
//
// NET_API_STATUS
// WINAPI
// NetReplExportDirEnum(
//      LPCWSTR servername,
//      DWORD level,
//      LPBYTE *bufptr,
//      DWORD prefmaxlen,
//      LPDWORD entriesread,
//      LPDWORD totalentries,
//      LPDWORD resumehandle)
// {
//     return ERROR_NOT_SUPPORTED;
// }
//
//
// NET_API_STATUS
// WINAPI
// NetReplExportDirGetInfo(
//      LPCWSTR servername,
//      LPCWSTR dirname,
//      DWORD level,
//      LPBYTE *bufptr)
// {
//     return ERROR_NOT_SUPPORTED;
// }
//
//
// NET_API_STATUS
// WINAPI
// NetReplExportDirLock(
//      LPCWSTR servername,
//      LPCWSTR dirname)
// {
//     return ERROR_NOT_SUPPORTED;
// }
//
//
// NET_API_STATUS
// WINAPI
// NetReplExportDirSetInfo(
//      LPCWSTR servername,
//      LPCWSTR dirname,
//      DWORD level,
//      const LPBYTE buf,
//      LPDWORD parm_err)
// {
//     return ERROR_NOT_SUPPORTED;
// }
//
//
// NET_API_STATUS
// WINAPI
// NetReplExportDirUnlock(
//      LPCWSTR servername,
//      LPCWSTR dirname,
//      DWORD unlockforce)
// {
//     return ERROR_NOT_SUPPORTED;
// }
//
//
// NET_API_STATUS
// WINAPI
// NetReplGetInfo(
//      LPCWSTR servername,
//      DWORD level,
//      LPBYTE *bufptr)
// {
//     return ERROR_NOT_SUPPORTED;
// }
//
//
// NET_API_STATUS
// WINAPI
// NetReplImportDirAdd(
//      LPCWSTR servername,
//      DWORD level,
//      const LPBYTE buf,
//      LPDWORD parm_err)
// {
//     return ERROR_NOT_SUPPORTED;
// }
//
//
// NET_API_STATUS
// WINAPI
// NetReplImportDirDel(
//      LPCWSTR servername,
//      LPCWSTR dirname)
// {
//     return ERROR_NOT_SUPPORTED;
// }
//
//
// NET_API_STATUS
// WINAPI
// NetReplImportDirEnum(
//      LPCWSTR servername,
//      DWORD level,
//      LPBYTE *bufptr,
//      DWORD prefmaxlen,
//      LPDWORD entriesread,
//      LPDWORD totalentries,
//      LPDWORD resumehandle)
// {
//     return ERROR_NOT_SUPPORTED;
// }
//
//
// NET_API_STATUS
// WINAPI
// NetReplImportDirGetInfo(
//      LPCWSTR servername,
//      LPCWSTR dirname,
//      DWORD level,
//      LPBYTE *bufptr)
// {
//     return ERROR_NOT_SUPPORTED;
// }
//
//
// NET_API_STATUS
// WINAPI
// NetReplImportDirLock(
//      LPCWSTR servername,
//      LPCWSTR dirname)
// {
//     return ERROR_NOT_SUPPORTED;
// }
//
//
// NET_API_STATUS
// WINAPI
// NetReplImportDirUnlock(
//      LPCWSTR servername,
//      LPCWSTR dirname,
//      DWORD unlockforce)
// {
//     return ERROR_NOT_SUPPORTED;
// }
//
//
// NET_API_STATUS
// WINAPI
// NetReplSetInfo(
//      LPCWSTR servername,
//      DWORD level,
//      const LPBYTE buf,
//      LPDWORD parm_err)
// {
//     return ERROR_NOT_SUPPORTED;
// }
//
//
// NET_API_STATUS
// WINAPI
// NetServiceControl(
//      LPCWSTR servername,
//      LPCWSTR service,
//      DWORD opcode,
//      DWORD arg,
//      LPBYTE *bufptr)
// {
//     return ERROR_NOT_SUPPORTED;
// }
//
//
// NET_API_STATUS
// WINAPI
// NetServiceEnum(
//      LPCWSTR servername,
//      DWORD level,
//      LPBYTE *bufptr,
//      DWORD prefmaxlen,
//      LPDWORD entriesread,
//      LPDWORD totalentries,
//      LPDWORD resume_handle)
// {
//     return ERROR_NOT_SUPPORTED;
// }
//
//
// NET_API_STATUS
// WINAPI
// NetServiceGetInfo(
//      LPCWSTR servername,
//      LPCWSTR service,
//      DWORD level,
//      LPBYTE *bufptr)
// {
//     return ERROR_NOT_SUPPORTED;
// }
//
//
// NET_API_STATUS
// WINAPI
// NetServiceInstall(
//      LPCWSTR servername,
//      LPCWSTR service,
//      DWORD argc,
//      LPCWSTR argv[],
//      LPBYTE *bufptr)
// {
//     return ERROR_NOT_SUPPORTED;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

// NetGetAadJoinInformation reports the Azure AD (Entra ID) join state. This machine is not
// AAD-joined and no DSREG join provider exists, so the answer is "not joined": S_OK with a NULL
// join-info pointer, which is the documented not-joined result. Wine's version is a stub that
// returns ERROR_CALL_NOT_IMPLEMENTED; answering the question is more honest than failing it.
#[no_mangle]
pub unsafe extern "system" fn NetGetAadJoinInformation(
    tenant_id: LPCWSTR,
    join_info: *mut *mut c_void,
) -> i32 {
    let _ = tenant_id;
    if join_info.is_null() {
        return E_INVALIDARG_HR;
    }
    *join_info = core::ptr::null_mut();
    S_OK_HR
}

// NetGetAadJoinInformation never allocates join info on this machine (it always reports not
// joined), so there is nothing to release; freeing NULL is the documented no-op.
#[no_mangle]
pub unsafe extern "system" fn NetFreeAadJoinInformation(join_info: *mut c_void) {
    let _ = join_info;
}

const S_OK_HR: i32 = 0;
const E_INVALIDARG_HR: i32 = 0x8007_0057u32 as i32;
