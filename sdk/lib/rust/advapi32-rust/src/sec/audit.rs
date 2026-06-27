use crate::prelude::*;
use core::ffi::c_void;

extern "C" {
}

// ---- file-scope preamble (translated) ----
// No translatable file-scope declarations.

#[no_mangle]
pub unsafe extern "system" fn AccessCheckAndAuditAlarmA(
    SubsystemName: LPCSTR,
    HandleId: LPVOID,
    ObjectTypeName: LPSTR,
    ObjectName: LPSTR,
    SecurityDescriptor: PSECURITY_DESCRIPTOR,
    DesiredAccess: u32,
    GenericMapping: PGENERIC_MAPPING,
    ObjectCreation: i32,
    GrantedAccess: LPDWORD,
    AccessStatus: LPBOOL,
    pfGenerateOnClose: LPBOOL,
) -> i32 {

    let mut SubsystemNameU: _UNICODE_STRING = core::mem::zeroed();
    let mut ObjectTypeNameU: _UNICODE_STRING = core::mem::zeroed();
    let mut ObjectNameU: _UNICODE_STRING = core::mem::zeroed();
    let mut LocalAccessStatus: i32 = 0;
    let mut GenerateOnClose: u8 = 0;
    let Status;

    RtlCreateUnicodeStringFromAsciiz(&mut SubsystemNameU, SubsystemName as *mut _);
    RtlCreateUnicodeStringFromAsciiz(&mut ObjectTypeNameU, ObjectTypeName as *mut _);
    RtlCreateUnicodeStringFromAsciiz(&mut ObjectNameU, ObjectName as *mut _);

    Status = NtAccessCheckAndAuditAlarm(
        &mut SubsystemNameU,
        HandleId,
        &mut ObjectTypeNameU,
        &mut ObjectNameU,
        SecurityDescriptor,
        DesiredAccess,
        GenericMapping,
        ObjectCreation as BOOLEAN,
        GrantedAccess,
        &mut LocalAccessStatus,
        &mut GenerateOnClose,
    );

    RtlFreeUnicodeString(&mut SubsystemNameU);
    RtlFreeUnicodeString(&mut ObjectTypeNameU);
    RtlFreeUnicodeString(&mut ObjectNameU);

    *pfGenerateOnClose = GenerateOnClose as i32;

    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    if !NT_SUCCESS(LocalAccessStatus) {
        *AccessStatus = 0;
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    *AccessStatus = 1;

    1
}

#[no_mangle]
pub unsafe extern "system" fn AccessCheckAndAuditAlarmW(
    SubsystemName: LPCWSTR,
    HandleId: LPVOID,
    ObjectTypeName: LPWSTR,
    ObjectName: LPWSTR,
    SecurityDescriptor: PSECURITY_DESCRIPTOR,
    DesiredAccess: u32,
    GenericMapping: PGENERIC_MAPPING,
    ObjectCreation: i32,
    GrantedAccess: LPDWORD,
    AccessStatus: LPBOOL,
    pfGenerateOnClose: LPBOOL,
) -> i32 {

    let mut SubsystemNameU: _UNICODE_STRING = core::mem::zeroed();
    let mut ObjectTypeNameU: _UNICODE_STRING = core::mem::zeroed();
    let mut ObjectNameU: _UNICODE_STRING = core::mem::zeroed();
    let mut LocalAccessStatus: i32 = 0;
    let mut GenerateOnClose: u8 = 0;

    RtlInitUnicodeString(&mut SubsystemNameU, SubsystemName as PWSTR);
    RtlInitUnicodeString(&mut ObjectTypeNameU, ObjectTypeName as PWSTR);
    RtlInitUnicodeString(&mut ObjectNameU, ObjectName as PWSTR);

    let Status = NtAccessCheckAndAuditAlarm(
        &mut SubsystemNameU,
        HandleId,
        &mut ObjectTypeNameU,
        &mut ObjectNameU,
        SecurityDescriptor,
        DesiredAccess,
        GenericMapping,
        ObjectCreation as u8,
        GrantedAccess,
        &mut LocalAccessStatus,
        &mut GenerateOnClose,
    );

    *pfGenerateOnClose = GenerateOnClose as i32;

    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    if !NT_SUCCESS(LocalAccessStatus) {
        *AccessStatus = 0;
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    *AccessStatus = 1;

    1
}

#[no_mangle]
pub unsafe extern "system" fn ObjectCloseAuditAlarmA(
    SubsystemName: LPCSTR,
    HandleId: LPVOID,
    GenerateOnClose: BOOL,
) -> BOOL {
    let mut Name: _UNICODE_STRING = core::mem::zeroed();
    let Status: NTSTATUS;

    if RtlCreateUnicodeStringFromAsciiz(&mut Name, SubsystemName) == 0 {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }

    Status = NtCloseObjectAuditAlarm(&mut Name, HandleId, GenerateOnClose as BOOLEAN);
    RtlFreeUnicodeString(&mut Name);
    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn ObjectCloseAuditAlarmW(
    SubsystemName: LPCWSTR,
    HandleId: LPVOID,
    GenerateOnClose: BOOL,
) -> BOOL {
    let mut Name: _UNICODE_STRING = core::mem::zeroed();

    RtlInitUnicodeString(&mut Name, SubsystemName as PWSTR);

    let Status = NtCloseObjectAuditAlarm(&mut Name, HandleId, GenerateOnClose as u8);
    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn ObjectDeleteAuditAlarmA(
    SubsystemName: LPCSTR,
    HandleId: LPVOID,
    GenerateOnClose: BOOL,
) -> BOOL {

    let mut Name: _UNICODE_STRING = core::mem::zeroed();

    if RtlCreateUnicodeStringFromAsciiz(&mut Name, SubsystemName) == 0 {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }

    let Status = NtDeleteObjectAuditAlarm(&mut Name, HandleId, GenerateOnClose as BOOLEAN);
    RtlFreeUnicodeString(&mut Name);
    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn ObjectDeleteAuditAlarmW(
    SubsystemName: LPCWSTR,
    HandleId: LPVOID,
    GenerateOnClose: BOOL,
) -> BOOL {
    let mut Name: _UNICODE_STRING = core::mem::zeroed();

    RtlInitUnicodeString(&mut Name, SubsystemName as PWSTR);

    let Status = NtDeleteObjectAuditAlarm(&mut Name, HandleId, GenerateOnClose as BOOLEAN);
    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn ObjectOpenAuditAlarmA(
    SubsystemName: LPCSTR,
    HandleId: LPVOID,
    ObjectTypeName: LPSTR,
    ObjectName: LPSTR,
    pSecurityDescriptor: PSECURITY_DESCRIPTOR,
    ClientToken: HANDLE,
    DesiredAccess: DWORD,
    GrantedAccess: DWORD,
    Privileges: PPRIVILEGE_SET,
    ObjectCreation: BOOL,
    AccessGranted: BOOL,
    GenerateOnClose: LPBOOL,
) -> BOOL {
    let mut SubsystemNameU: _UNICODE_STRING = core::mem::zeroed();
    let mut ObjectTypeNameU: _UNICODE_STRING = core::mem::zeroed();
    let mut ObjectNameU: _UNICODE_STRING = core::mem::zeroed();
    let Status: NTSTATUS;

    RtlCreateUnicodeStringFromAsciiz(&mut SubsystemNameU, SubsystemName as PCHAR);
    RtlCreateUnicodeStringFromAsciiz(&mut ObjectTypeNameU, ObjectTypeName as PCHAR);
    RtlCreateUnicodeStringFromAsciiz(&mut ObjectNameU, ObjectName as PCHAR);

    Status = NtOpenObjectAuditAlarm(
        &mut SubsystemNameU,
        HandleId,
        &mut ObjectTypeNameU,
        &mut ObjectNameU,
        pSecurityDescriptor,
        ClientToken,
        DesiredAccess,
        GrantedAccess,
        Privileges,
        ObjectCreation as BOOLEAN,
        AccessGranted as BOOLEAN,
        GenerateOnClose as PBOOLEAN,
    );

    RtlFreeUnicodeString(&mut SubsystemNameU);
    RtlFreeUnicodeString(&mut ObjectTypeNameU);
    RtlFreeUnicodeString(&mut ObjectNameU);

    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn ObjectOpenAuditAlarmW(
    SubsystemName: LPCWSTR,
    HandleId: LPVOID,
    ObjectTypeName: LPWSTR,
    ObjectName: LPWSTR,
    pSecurityDescriptor: PSECURITY_DESCRIPTOR,
    ClientToken: HANDLE,
    DesiredAccess: DWORD,
    GrantedAccess: DWORD,
    Privileges: PPRIVILEGE_SET,
    ObjectCreation: BOOL,
    AccessGranted: BOOL,
    GenerateOnClose: LPBOOL,
) -> BOOL {
    let mut SubsystemNameU: _UNICODE_STRING = core::mem::zeroed();
    let mut ObjectTypeNameU: _UNICODE_STRING = core::mem::zeroed();
    let mut ObjectNameU: _UNICODE_STRING = core::mem::zeroed();

    RtlInitUnicodeString(&mut SubsystemNameU, SubsystemName as PWSTR);
    RtlInitUnicodeString(&mut ObjectTypeNameU, ObjectTypeName as PWSTR);
    RtlInitUnicodeString(&mut ObjectNameU, ObjectName as PWSTR);

    let Status = NtOpenObjectAuditAlarm(
        &mut SubsystemNameU,
        HandleId,
        &mut ObjectTypeNameU,
        &mut ObjectNameU,
        pSecurityDescriptor,
        ClientToken,
        DesiredAccess,
        GrantedAccess,
        Privileges,
        ObjectCreation as BOOLEAN,
        AccessGranted as BOOLEAN,
        GenerateOnClose as PBOOLEAN,
    );

    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn ObjectPrivilegeAuditAlarmA(
    SubsystemName: LPCSTR,
    HandleId: LPVOID,
    ClientToken: HANDLE,
    DesiredAccess: DWORD,
    Privileges: PPRIVILEGE_SET,
    AccessGranted: BOOL,
) -> BOOL {
    let mut SubsystemNameU: _UNICODE_STRING = core::mem::zeroed();

    RtlCreateUnicodeStringFromAsciiz(
        &mut SubsystemNameU,
        SubsystemName as PCHAR,
    );

    let Status = NtPrivilegeObjectAuditAlarm(
        &mut SubsystemNameU,
        HandleId,
        ClientToken,
        DesiredAccess,
        Privileges,
        AccessGranted as BOOLEAN,
    );
    RtlFreeUnicodeString(&mut SubsystemNameU);
    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn ObjectPrivilegeAuditAlarmW(
    SubsystemName: LPCWSTR,
    HandleId: LPVOID,
    ClientToken: HANDLE,
    DesiredAccess: DWORD,
    Privileges: PPRIVILEGE_SET,
    AccessGranted: BOOL,
) -> BOOL {
    let mut SubsystemNameU: _UNICODE_STRING = core::mem::zeroed();
    let Status: NTSTATUS;

    RtlInitUnicodeString(&mut SubsystemNameU, SubsystemName as PWSTR);

    Status = NtPrivilegeObjectAuditAlarm(
        &mut SubsystemNameU,
        HandleId,
        ClientToken,
        DesiredAccess,
        Privileges,
        AccessGranted as BOOLEAN,
    );

    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn PrivilegedServiceAuditAlarmA(
    SubsystemName: LPCSTR,
    ServiceName: LPCSTR,
    ClientToken: HANDLE,
    Privileges: PPRIVILEGE_SET,
    AccessGranted: BOOL,
) -> BOOL {
    let mut SubsystemNameU: _UNICODE_STRING = core::mem::zeroed();
    let mut ServiceNameU: _UNICODE_STRING = core::mem::zeroed();
    let Status: NTSTATUS;

    RtlCreateUnicodeStringFromAsciiz(
        &mut SubsystemNameU,
        SubsystemName as PCHAR,
    );
    RtlCreateUnicodeStringFromAsciiz(
        &mut ServiceNameU,
        ServiceName as PCHAR,
    );

    Status = NtPrivilegedServiceAuditAlarm(
        &mut SubsystemNameU,
        &mut ServiceNameU,
        ClientToken,
        Privileges,
        AccessGranted as BOOLEAN,
    );
    RtlFreeUnicodeString(&mut SubsystemNameU);
    RtlFreeUnicodeString(&mut ServiceNameU);
    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return FALSE;
    }

    TRUE
}

#[no_mangle]
pub unsafe extern "system" fn PrivilegedServiceAuditAlarmW(
    SubsystemName: LPCWSTR,
    ServiceName: LPCWSTR,
    ClientToken: HANDLE,
    Privileges: PPRIVILEGE_SET,
    AccessGranted: BOOL,
) -> BOOL {
    let mut SubsystemNameU: _UNICODE_STRING = core::mem::zeroed();
    let mut ServiceNameU: _UNICODE_STRING = core::mem::zeroed();
    let Status: NTSTATUS;

    RtlInitUnicodeString(
        core::ptr::addr_of_mut!(SubsystemNameU),
        SubsystemName as PWSTR,
    );
    RtlInitUnicodeString(
        core::ptr::addr_of_mut!(ServiceNameU),
        ServiceName as PWSTR,
    );

    Status = NtPrivilegedServiceAuditAlarm(
        core::ptr::addr_of_mut!(SubsystemNameU),
        core::ptr::addr_of_mut!(ServiceNameU),
        ClientToken,
        Privileges,
        AccessGranted as BOOLEAN,
    );

    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn AccessCheckByTypeResultListAndAuditAlarmByHandleW(
    SubsystemName: LPCWSTR,
    HandleId: LPVOID,
    ClientToken: HANDLE,
    ObjectTypeName: LPCWSTR,
    ObjectName: LPCWSTR,
    pSecurityDescriptor: PSECURITY_DESCRIPTOR,
    PrincipalSelfSid: PSID,
    DesiredAccess: DWORD,
    AuditType: AUDIT_EVENT_TYPE,
    Flags: DWORD,
    ObjectTypeList: POBJECT_TYPE_LIST,
    ObjectTypeListLength: DWORD,
    GenericMapping: PGENERIC_MAPPING,
    ObjectCreation: BOOL,
    GrantedAccess: LPDWORD,
    AccessStatusList: LPDWORD,
    pfGenerateOnClose: LPBOOL,
) -> BOOL {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    FALSE
}

#[no_mangle]
pub unsafe extern "system" fn AccessCheckByTypeResultListAndAuditAlarmByHandleA(
    SubsystemName: LPCSTR,
    HandleId: LPVOID,
    ClientToken: HANDLE,
    ObjectTypeName: LPCSTR,
    ObjectName: LPCSTR,
    pSecurityDescriptor: PSECURITY_DESCRIPTOR,
    PrincipalSelfSid: PSID,
    DesiredAccess: DWORD,
    AuditType: AUDIT_EVENT_TYPE,
    Flags: DWORD,
    ObjectTypeList: POBJECT_TYPE_LIST,
    ObjectTypeListLength: DWORD,
    GenericMapping: PGENERIC_MAPPING,
    ObjectCreation: BOOL,
    GrantedAccess: LPDWORD,
    AccessStatusList: LPDWORD,
    pfGenerateOnClose: LPBOOL,
) -> BOOL {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    0
}

#[no_mangle]
pub unsafe extern "system" fn AccessCheckByTypeResultListAndAuditAlarmW(
    SubsystemName: LPCWSTR,
    HandleId: LPVOID,
    ObjectTypeName: LPCWSTR,
    ObjectName: LPCWSTR,
    pSecurityDescriptor: PSECURITY_DESCRIPTOR,
    PrincipalSelfSid: PSID,
    DesiredAccess: u32,
    AuditType: AUDIT_EVENT_TYPE,
    Flags: u32,
    ObjectTypeList: POBJECT_TYPE_LIST,
    ObjectTypeListLength: u32,
    GenericMapping: PGENERIC_MAPPING,
    ObjectCreation: i32,
    GrantedAccess: LPDWORD,
    AccessStatusList: LPDWORD,
    pfGenerateOnClose: LPBOOL,
) -> i32 {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    0
}

#[no_mangle]
pub unsafe extern "system" fn AccessCheckByTypeResultListAndAuditAlarmA(
    SubsystemName: LPCSTR,
    HandleId: LPVOID,
    ObjectTypeName: LPCSTR,
    ObjectName: LPCSTR,
    pSecurityDescriptor: PSECURITY_DESCRIPTOR,
    PrincipalSelfSid: PSID,
    DesiredAccess: u32,
    AuditType: AUDIT_EVENT_TYPE,
    Flags: u32,
    ObjectTypeList: POBJECT_TYPE_LIST,
    ObjectTypeListLength: u32,
    GenericMapping: PGENERIC_MAPPING,
    ObjectCreation: i32,
    GrantedAccess: LPDWORD,
    AccessStatusList: LPDWORD,
    pfGenerateOnClose: LPBOOL,
) -> i32 {
    FIXME!(b"%s() not implemented!\n\0".as_ptr() as LPCSTR, b"AccessCheckByTypeResultListAndAuditAlarmA\0".as_ptr() as LPCSTR);
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    0
}

#[no_mangle]
pub unsafe extern "system" fn AccessCheckByTypeAndAuditAlarmW(
    SubsystemName: LPCWSTR,
    HandleId: LPVOID,
    ObjectTypeName: LPCWSTR,
    ObjectName: LPCWSTR,
    pSecurityDescriptor: PSECURITY_DESCRIPTOR,
    PrincipalSelfSid: PSID,
    DesiredAccess: u32,
    AuditType: AUDIT_EVENT_TYPE,
    Flags: u32,
    ObjectTypeList: POBJECT_TYPE_LIST,
    ObjectTypeListLength: u32,
    GenericMapping: PGENERIC_MAPPING,
    ObjectCreation: i32,
    GrantedAccess: LPDWORD,
    AccessStatus: LPBOOL,
    pfGenerateOnClose: LPBOOL,
) -> i32 {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    0
}

#[no_mangle]
pub unsafe extern "system" fn AccessCheckByTypeAndAuditAlarmA(
    SubsystemName: LPCSTR,
    HandleId: LPVOID,
    ObjectTypeName: LPCSTR,
    ObjectName: LPCSTR,
    pSecurityDescriptor: PSECURITY_DESCRIPTOR,
    PrincipalSelfSid: PSID,
    DesiredAccess: u32,
    AuditType: AUDIT_EVENT_TYPE,
    Flags: u32,
    ObjectTypeList: POBJECT_TYPE_LIST,
    ObjectTypeListLength: u32,
    GenericMapping: PGENERIC_MAPPING,
    ObjectCreation: i32,
    GrantedAccess: LPDWORD,
    AccessStatus: LPBOOL,
    pfGenerateOnClose: LPBOOL,
) -> i32 {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    0
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS system libraries
//  * FILE:            lib/advapi32/sec/audit.c
//  * PURPOSE:         Audit functions
//  * PROGRAMMER:      Eric Kohl
//  * UPDATE HISTORY:
//  *                  Created 07/19/2003
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include <advapi32.h>
// WINE_DEFAULT_DEBUG_CHANNEL(advapi);
//
// /* FUNCTIONS ****************************************************************/
//
// /*
//  * @implemented
//  */
// BOOL WINAPI
// AccessCheckAndAuditAlarmA(LPCSTR SubsystemName,
//                           LPVOID HandleId,
//                           LPSTR ObjectTypeName,
//                           LPSTR ObjectName,
//                           PSECURITY_DESCRIPTOR SecurityDescriptor,
//                           DWORD DesiredAccess,
//                           PGENERIC_MAPPING GenericMapping,
//                           BOOL ObjectCreation,
//                           LPDWORD GrantedAccess,
//                           LPBOOL AccessStatus,
//                           LPBOOL pfGenerateOnClose)
// {
//     UNICODE_STRING SubsystemNameU;
//     UNICODE_STRING ObjectTypeNameU;
//     UNICODE_STRING ObjectNameU;
//     NTSTATUS LocalAccessStatus;
//     BOOLEAN GenerateOnClose;
//     NTSTATUS Status;
//
//     RtlCreateUnicodeStringFromAsciiz(&SubsystemNameU,
//                                      (PCHAR)SubsystemName);
//     RtlCreateUnicodeStringFromAsciiz(&ObjectTypeNameU,
//                                      (PCHAR)ObjectTypeName);
//     RtlCreateUnicodeStringFromAsciiz(&ObjectNameU,
//                                      (PCHAR)ObjectName);
//
//     Status = NtAccessCheckAndAuditAlarm(&SubsystemNameU,
//                                         HandleId,
//                                         &ObjectTypeNameU,
//                                         &ObjectNameU,
//                                         SecurityDescriptor,
//                                         DesiredAccess,
//                                         GenericMapping,
//                                         ObjectCreation as BOOLEAN,
//                                         GrantedAccess,
//                                         &LocalAccessStatus,
//                                         &GenerateOnClose);
//     RtlFreeUnicodeString(&SubsystemNameU);
//     RtlFreeUnicodeString(&ObjectTypeNameU);
//     RtlFreeUnicodeString(&ObjectNameU);
//
//     *pfGenerateOnClose = (BOOL)GenerateOnClose;
//
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     if (!NT_SUCCESS (LocalAccessStatus))
//     {
//         *AccessStatus = FALSE;
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     *AccessStatus = TRUE;
//
//     return TRUE;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL WINAPI
// AccessCheckAndAuditAlarmW(LPCWSTR SubsystemName,
//                           LPVOID HandleId,
//                           LPWSTR ObjectTypeName,
//                           LPWSTR ObjectName,
//                           PSECURITY_DESCRIPTOR SecurityDescriptor,
//                           DWORD DesiredAccess,
//                           PGENERIC_MAPPING GenericMapping,
//                           BOOL ObjectCreation,
//                           LPDWORD GrantedAccess,
//                           LPBOOL AccessStatus,
//                           LPBOOL pfGenerateOnClose)
// {
//     UNICODE_STRING SubsystemNameU;
//     UNICODE_STRING ObjectTypeNameU;
//     UNICODE_STRING ObjectNameU;
//     NTSTATUS LocalAccessStatus;
//     BOOLEAN GenerateOnClose;
//     NTSTATUS Status;
//
//     RtlInitUnicodeString(&SubsystemNameU,
//                          (PWSTR)SubsystemName);
//     RtlInitUnicodeString(&ObjectTypeNameU,
//                          (PWSTR)ObjectTypeName);
//     RtlInitUnicodeString(&ObjectNameU,
//                          (PWSTR)ObjectName);
//
//     Status = NtAccessCheckAndAuditAlarm(&SubsystemNameU,
//                                         HandleId,
//                                         &ObjectTypeNameU,
//                                         &ObjectNameU,
//                                         SecurityDescriptor,
//                                         DesiredAccess,
//                                         GenericMapping,
//                                         ObjectCreation as BOOLEAN,
//                                         GrantedAccess,
//                                         &LocalAccessStatus,
//                                         &GenerateOnClose);
//
//     *pfGenerateOnClose = (BOOL)GenerateOnClose;
//
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     if (!NT_SUCCESS(LocalAccessStatus))
//     {
//         *AccessStatus = FALSE;
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     *AccessStatus = TRUE;
//
//     return TRUE;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL WINAPI
// ObjectCloseAuditAlarmA(LPCSTR SubsystemName,
//                        LPVOID HandleId,
//                        BOOL GenerateOnClose)
// {
//     UNICODE_STRING Name;
//     NTSTATUS Status;
//
//     if (!RtlCreateUnicodeStringFromAsciiz(&Name, SubsystemName))
//     {
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         return FALSE;
//     }
//
//     Status = NtCloseObjectAuditAlarm(&Name,
//                                      HandleId,
//                                      GenerateOnClose);
//     RtlFreeUnicodeString(&Name);
//     if (!NT_SUCCESS (Status))
//     {
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL WINAPI
// ObjectCloseAuditAlarmW(LPCWSTR SubsystemName,
//                        LPVOID HandleId,
//                        BOOL GenerateOnClose)
// {
//     UNICODE_STRING Name;
//     NTSTATUS Status;
//
//     RtlInitUnicodeString(&Name,
//                          (PWSTR)SubsystemName);
//
//     Status = NtCloseObjectAuditAlarm(&Name,
//                                      HandleId,
//                                      GenerateOnClose);
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL WINAPI
// ObjectDeleteAuditAlarmA(LPCSTR SubsystemName,
//                         LPVOID HandleId,
//                         BOOL GenerateOnClose)
// {
//     UNICODE_STRING Name;
//     NTSTATUS Status;
//
//     if (!RtlCreateUnicodeStringFromAsciiz(&Name, SubsystemName))
//     {
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         return FALSE;
//     }
//
//     Status = NtDeleteObjectAuditAlarm(&Name,
//                                       HandleId,
//                                       GenerateOnClose);
//     RtlFreeUnicodeString(&Name);
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL WINAPI
// ObjectDeleteAuditAlarmW(LPCWSTR SubsystemName,
//                         LPVOID HandleId,
//                         BOOL GenerateOnClose)
// {
//     UNICODE_STRING Name;
//     NTSTATUS Status;
//
//     RtlInitUnicodeString(&Name,
//                          (PWSTR)SubsystemName);
//
//     Status = NtDeleteObjectAuditAlarm(&Name,
//                                       HandleId,
//                                       GenerateOnClose);
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL WINAPI
// ObjectOpenAuditAlarmA(LPCSTR SubsystemName,
//                       LPVOID HandleId,
//                       LPSTR ObjectTypeName,
//                       LPSTR ObjectName,
//                       PSECURITY_DESCRIPTOR pSecurityDescriptor,
//                       HANDLE ClientToken,
//                       DWORD DesiredAccess,
//                       DWORD GrantedAccess,
//                       PPRIVILEGE_SET Privileges,
//                       BOOL ObjectCreation,
//                       BOOL AccessGranted,
//                       LPBOOL GenerateOnClose)
// {
//     UNICODE_STRING SubsystemNameU;
//     UNICODE_STRING ObjectTypeNameU;
//     UNICODE_STRING ObjectNameU;
//     NTSTATUS Status;
//
//     RtlCreateUnicodeStringFromAsciiz(&SubsystemNameU,
//                                      (PCHAR)SubsystemName);
//     RtlCreateUnicodeStringFromAsciiz(&ObjectTypeNameU,
//                                      (PCHAR)ObjectTypeName);
//     RtlCreateUnicodeStringFromAsciiz(&ObjectNameU,
//                                      (PCHAR)ObjectName);
//
//     Status = NtOpenObjectAuditAlarm(&SubsystemNameU,
//                                     HandleId,
//                                     &ObjectTypeNameU,
//                                     &ObjectNameU,
//                                     pSecurityDescriptor,
//                                     ClientToken,
//                                     DesiredAccess,
//                                     GrantedAccess,
//                                     Privileges,
//                                     ObjectCreation as BOOLEAN,
//                                     AccessGranted as BOOLEAN,
//                                     (PBOOLEAN)GenerateOnClose);
//     RtlFreeUnicodeString(&SubsystemNameU);
//     RtlFreeUnicodeString(&ObjectTypeNameU);
//     RtlFreeUnicodeString(&ObjectNameU);
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL WINAPI
// ObjectOpenAuditAlarmW(LPCWSTR SubsystemName,
//                       LPVOID HandleId,
//                       LPWSTR ObjectTypeName,
//                       LPWSTR ObjectName,
//                       PSECURITY_DESCRIPTOR pSecurityDescriptor,
//                       HANDLE ClientToken,
//                       DWORD DesiredAccess,
//                       DWORD GrantedAccess,
//                       PPRIVILEGE_SET Privileges,
//                       BOOL ObjectCreation,
//                       BOOL AccessGranted,
//                       LPBOOL GenerateOnClose)
// {
//     UNICODE_STRING SubsystemNameU;
//     UNICODE_STRING ObjectTypeNameU;
//     UNICODE_STRING ObjectNameU;
//     NTSTATUS Status;
//
//     RtlInitUnicodeString(&SubsystemNameU,
//                          (PWSTR)SubsystemName);
//     RtlInitUnicodeString(&ObjectTypeNameU,
//                          (PWSTR)ObjectTypeName);
//     RtlInitUnicodeString(&ObjectNameU,
//                          (PWSTR)ObjectName);
//
//     Status = NtOpenObjectAuditAlarm(&SubsystemNameU,
//                                     HandleId,
//                                     &ObjectTypeNameU,
//                                     &ObjectNameU,
//                                     pSecurityDescriptor,
//                                     ClientToken,
//                                     DesiredAccess,
//                                     GrantedAccess,
//                                     Privileges,
//                                     ObjectCreation as BOOLEAN,
//                                     AccessGranted as BOOLEAN,
//                                     (PBOOLEAN)GenerateOnClose);
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL WINAPI
// ObjectPrivilegeAuditAlarmA(LPCSTR SubsystemName,
//                            LPVOID HandleId,
//                            HANDLE ClientToken,
//                            DWORD DesiredAccess,
//                            PPRIVILEGE_SET Privileges,
//                            BOOL AccessGranted)
// {
//     UNICODE_STRING SubsystemNameU;
//     NTSTATUS Status;
//
//     RtlCreateUnicodeStringFromAsciiz(&SubsystemNameU,
//                                      (PCHAR)SubsystemName);
//
//     Status = NtPrivilegeObjectAuditAlarm(&SubsystemNameU,
//                                          HandleId,
//                                          ClientToken,
//                                          DesiredAccess,
//                                          Privileges,
//                                          AccessGranted);
//     RtlFreeUnicodeString (&SubsystemNameU);
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL WINAPI
// ObjectPrivilegeAuditAlarmW(LPCWSTR SubsystemName,
//                            LPVOID HandleId,
//                            HANDLE ClientToken,
//                            DWORD DesiredAccess,
//                            PPRIVILEGE_SET Privileges,
//                            BOOL AccessGranted)
// {
//     UNICODE_STRING SubsystemNameU;
//     NTSTATUS Status;
//
//     RtlInitUnicodeString(&SubsystemNameU,
//                          (PWSTR)SubsystemName);
//
//     Status = NtPrivilegeObjectAuditAlarm(&SubsystemNameU,
//                                          HandleId,
//                                          ClientToken,
//                                          DesiredAccess,
//                                          Privileges,
//                                          AccessGranted);
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL WINAPI
// PrivilegedServiceAuditAlarmA(LPCSTR SubsystemName,
//                              LPCSTR ServiceName,
//                              HANDLE ClientToken,
//                              PPRIVILEGE_SET Privileges,
//                              BOOL AccessGranted)
// {
//     UNICODE_STRING SubsystemNameU;
//     UNICODE_STRING ServiceNameU;
//     NTSTATUS Status;
//
//     RtlCreateUnicodeStringFromAsciiz(&SubsystemNameU,
//                                      (PCHAR)SubsystemName);
//     RtlCreateUnicodeStringFromAsciiz(&ServiceNameU,
//                                      (PCHAR)ServiceName);
//
//     Status = NtPrivilegedServiceAuditAlarm(&SubsystemNameU,
//                                            &ServiceNameU,
//                                            ClientToken,
//                                            Privileges,
//                                            AccessGranted);
//     RtlFreeUnicodeString(&SubsystemNameU);
//     RtlFreeUnicodeString(&ServiceNameU);
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL WINAPI
// PrivilegedServiceAuditAlarmW(LPCWSTR SubsystemName,
//                              LPCWSTR ServiceName,
//                              HANDLE ClientToken,
//                              PPRIVILEGE_SET Privileges,
//                              BOOL AccessGranted)
// {
//     UNICODE_STRING SubsystemNameU;
//     UNICODE_STRING ServiceNameU;
//     NTSTATUS Status;
//
//     RtlInitUnicodeString(&SubsystemNameU,
//                          (PWSTR)SubsystemName);
//     RtlInitUnicodeString(&ServiceNameU,
//                          (PWSTR)ServiceName);
//
//     Status = NtPrivilegedServiceAuditAlarm(&SubsystemNameU,
//                                            &ServiceNameU,
//                                            ClientToken,
//                                            Privileges,
//                                            AccessGranted);
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
//
// /*
//  * @unimplemented
//  */
// BOOL WINAPI
// AccessCheckByTypeResultListAndAuditAlarmByHandleW(IN LPCWSTR SubsystemName,
//                                                   IN LPVOID HandleId,
//                                                   IN HANDLE ClientToken,
//                                                   IN LPCWSTR ObjectTypeName,
//                                                   IN LPCWSTR ObjectName,
//                                                   IN PSECURITY_DESCRIPTOR pSecurityDescriptor,
//                                                   IN PSID PrincipalSelfSid,
//                                                   IN DWORD DesiredAccess,
//                                                   IN AUDIT_EVENT_TYPE AuditType,
//                                                   IN DWORD Flags,
//                                                   IN POBJECT_TYPE_LIST ObjectTypeList,
//                                                   IN DWORD ObjectTypeListLength,
//                                                   IN PGENERIC_MAPPING GenericMapping,
//                                                   IN BOOL ObjectCreation,
//                                                   OUT LPDWORD GrantedAccess,
//                                                   OUT LPDWORD AccessStatusList,
//                                                   OUT LPBOOL pfGenerateOnClose)
// {
//     FIXME("%s() not implemented!\n", __FUNCTION__);
//     SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
//     return FALSE;
// }
//
//
// /*
//  * @unimplemented
//  */
// BOOL WINAPI
// AccessCheckByTypeResultListAndAuditAlarmByHandleA(IN LPCSTR SubsystemName,
//                                                   IN LPVOID HandleId,
//                                                   IN HANDLE ClientToken,
//                                                   IN LPCSTR ObjectTypeName,
//                                                   IN LPCSTR ObjectName,
//                                                   IN PSECURITY_DESCRIPTOR pSecurityDescriptor,
//                                                   IN PSID PrincipalSelfSid,
//                                                   IN DWORD DesiredAccess,
//                                                   IN AUDIT_EVENT_TYPE AuditType,
//                                                   IN DWORD Flags,
//                                                   IN POBJECT_TYPE_LIST ObjectTypeList,
//                                                   IN DWORD ObjectTypeListLength,
//                                                   IN PGENERIC_MAPPING GenericMapping,
//                                                   IN BOOL ObjectCreation,
//                                                   OUT LPDWORD GrantedAccess,
//                                                   OUT LPDWORD AccessStatusList,
//                                                   OUT LPBOOL pfGenerateOnClose)
// {
//     FIXME("%s() not implemented!\n", __FUNCTION__);
//     SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
//     return FALSE;
// }
//
//
// /*
//  * @unimplemented
//  */
// BOOL WINAPI
// AccessCheckByTypeResultListAndAuditAlarmW(IN LPCWSTR SubsystemName,
//                                           IN LPVOID HandleId,
//                                           IN LPCWSTR ObjectTypeName,
//                                           IN LPCWSTR ObjectName,
//                                           IN PSECURITY_DESCRIPTOR pSecurityDescriptor,
//                                           IN PSID PrincipalSelfSid,
//                                           IN DWORD DesiredAccess,
//                                           IN AUDIT_EVENT_TYPE AuditType,
//                                           IN DWORD Flags,
//                                           IN POBJECT_TYPE_LIST ObjectTypeList,
//                                           IN DWORD ObjectTypeListLength,
//                                           IN PGENERIC_MAPPING GenericMapping,
//                                           IN BOOL ObjectCreation,
//                                           OUT LPDWORD GrantedAccess,
//                                           OUT LPDWORD AccessStatusList,
//                                           OUT LPBOOL pfGenerateOnClose)
// {
//     FIXME("%s() not implemented!\n", __FUNCTION__);
//     SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
//     return FALSE;
// }
//
//
// /*
//  * @unimplemented
//  */
// BOOL WINAPI
// AccessCheckByTypeResultListAndAuditAlarmA(IN LPCSTR SubsystemName,
//                                           IN LPVOID HandleId,
//                                           IN LPCSTR ObjectTypeName,
//                                           IN LPCSTR ObjectName,
//                                           IN PSECURITY_DESCRIPTOR pSecurityDescriptor,
//                                           IN PSID PrincipalSelfSid,
//                                           IN DWORD DesiredAccess,
//                                           IN AUDIT_EVENT_TYPE AuditType,
//                                           IN DWORD Flags,
//                                           IN POBJECT_TYPE_LIST ObjectTypeList,
//                                           IN DWORD ObjectTypeListLength,
//                                           IN PGENERIC_MAPPING GenericMapping,
//                                           IN BOOL ObjectCreation,
//                                           OUT LPDWORD GrantedAccess,
//                                           OUT LPDWORD AccessStatusList,
//                                           OUT LPBOOL pfGenerateOnClose)
// {
//     FIXME("%s() not implemented!\n", __FUNCTION__);
//     SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
//     return FALSE;
// }
//
//
// /*
//  * @unimplemented
//  */
// BOOL WINAPI
// AccessCheckByTypeAndAuditAlarmW(IN LPCWSTR SubsystemName,
//                                 IN LPVOID HandleId,
//                                 IN LPCWSTR ObjectTypeName,
//                                 IN LPCWSTR ObjectName,
//                                 IN PSECURITY_DESCRIPTOR pSecurityDescriptor,
//                                 IN PSID PrincipalSelfSid,
//                                 IN DWORD DesiredAccess,
//                                 IN AUDIT_EVENT_TYPE AuditType,
//                                 IN DWORD Flags,
//                                 IN POBJECT_TYPE_LIST ObjectTypeList,
//                                 IN DWORD ObjectTypeListLength,
//                                 IN PGENERIC_MAPPING GenericMapping,
//                                 IN BOOL ObjectCreation,
//                                 OUT LPDWORD GrantedAccess,
//                                 OUT LPBOOL AccessStatus,
//                                 OUT LPBOOL pfGenerateOnClose)
// {
//     FIXME("%s() not implemented!\n", __FUNCTION__);
//     SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
//     return FALSE;
// }
//
//
// /*
//  * @unimplemented
//  */
// BOOL WINAPI
// AccessCheckByTypeAndAuditAlarmA(IN LPCSTR SubsystemName,
//                                 IN LPVOID HandleId,
//                                 IN LPCSTR ObjectTypeName,
//                                 IN LPCSTR ObjectName,
//                                 IN PSECURITY_DESCRIPTOR pSecurityDescriptor,
//                                 IN PSID PrincipalSelfSid,
//                                 IN DWORD DesiredAccess,
//                                 IN AUDIT_EVENT_TYPE AuditType,
//                                 IN DWORD Flags,
//                                 IN POBJECT_TYPE_LIST ObjectTypeList,
//                                 IN DWORD ObjectTypeListLength,
//                                 IN PGENERIC_MAPPING GenericMapping,
//                                 IN BOOL ObjectCreation,
//                                 OUT LPDWORD GrantedAccess,
//                                 OUT LPBOOL AccessStatus,
//                                 OUT LPBOOL pfGenerateOnClose)
// {
//     FIXME("%s() not implemented!\n", __FUNCTION__);
//     SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
//     return FALSE;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
