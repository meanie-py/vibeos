use ntkrnl_sys::ddk::*;
use ntkrnl_sys::base::{STATUS_SUCCESS, STATUS_INSUFFICIENT_RESOURCES, STATUS_NO_MEMORY, STATUS_NOT_IMPLEMENTED, NT_SUCCESS, TRUE, FALSE};
use ntkrnl_sys::constants::*;
use ntkrnl_sys::mm::{PagedPool, NonPagedPool};
use ntkrnl_sys::ob::*;
use ntkrnl_sys::ps::PsGetCurrentProcess;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::probe::*;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

extern "C" {
    fn ReleaseCapturedUnicodeString(Name: *mut _UNICODE_STRING, PreviousMode: KPROCESSOR_MODE);
    fn ProbeAndCaptureUnicodeString(
        Dest: *mut _UNICODE_STRING,
        PreviousMode: KPROCESSOR_MODE,
        Src: *const _UNICODE_STRING,
    ) -> NTSTATUS;
    static mut SeTokenObjectType: PVOID;
    static mut SeExports: *mut _SE_EXPORTS;
    static mut SeLocalSystemSid: PSID;
}

const SecurityIdentification: SECURITY_IMPERSONATION_LEVEL = _SECURITY_IMPERSONATION_LEVEL_SecurityIdentification;
const TokenImpersonation: TOKEN_TYPE = _TOKEN_TYPE_TokenImpersonation;
const AuditEventObjectAccess: _AUDIT_EVENT_TYPE = _AUDIT_EVENT_TYPE_AuditEventObjectAccess;
const AuditEventDirectoryServiceAccess: _AUDIT_EVENT_TYPE = _AUDIT_EVENT_TYPE_AuditEventDirectoryServiceAccess;

const SEP_PRIVILEGE_SET_MAX_COUNT: ULONG = 60;

static mut SeSubsystemName: _UNICODE_STRING = _UNICODE_STRING {
    Length: 16,
    MaximumLength: 18,
    Buffer: win32macros_rust::wide_bytes!(b"S\0e\0c\0u\0r\0i\0t\0y\0\0\0") as PWCHAR,
};

#[no_mangle]
pub unsafe extern "system" fn SeDetailedAuditingWithToken(_token: PTOKEN) -> BOOLEAN {
    let _token = opaque_ptr(_token);
    0u8
}

#[no_mangle]
pub unsafe extern "system" fn SeAuditProcessCreate(_process: PEPROCESS) {
    let _process = opaque_ptr(_process);
}

#[no_mangle]
pub unsafe extern "system" fn SeAuditProcessExit(_process: PEPROCESS) {
    let _process = opaque_ptr(_process);
}

#[no_mangle]
pub unsafe extern "system" fn SeInitializeProcessAuditName(
    file_object: PFILE_OBJECT,
    do_audit: BOOLEAN,
    audit_info: *mut POBJECT_NAME_INFORMATION,
) -> NTSTATUS {
    let file_object = opaque_ptr(file_object);
    let audit_info = opaque_ptr(audit_info);
    let mut local_name_info: OBJECT_NAME_INFORMATION = core::mem::zeroed();
    let mut object_name_info: POBJECT_NAME_INFORMATION = core::ptr::null_mut();
    let mut return_length: ULONG = 8;
    let mut status: NTSTATUS;

    *audit_info = core::ptr::null_mut();

    if do_audit != 0 {
    }

    status = ObQueryNameString(
        file_object as PVOID,
        &mut local_name_info,
        core::mem::size_of::<OBJECT_NAME_INFORMATION>() as ULONG,
        &mut return_length,
    );

    if ((status == STATUS_BUFFER_OVERFLOW)
        || (status == STATUS_BUFFER_TOO_SMALL)
        || (status == STATUS_INFO_LENGTH_MISMATCH))
        && (return_length != core::mem::size_of::<OBJECT_NAME_INFORMATION>() as ULONG)
    {
        object_name_info = ExAllocatePoolWithTag(NonPagedPool, return_length as SIZE_T, TAG_SEPA)
            as POBJECT_NAME_INFORMATION;
        if !object_name_info.is_null() {
            status = ObQueryNameString(
                file_object as PVOID,
                object_name_info,
                return_length,
                &mut return_length,
            );
        }
    }

    if !object_name_info.is_null()
        && (status < 0
            || return_length == core::mem::size_of::<OBJECT_NAME_INFORMATION>() as ULONG)
    {
        if !object_name_info.is_null() {
            ExFreePool(object_name_info as PVOID);
        }

        return_length = core::mem::size_of::<OBJECT_NAME_INFORMATION>() as ULONG;
        object_name_info = ExAllocatePoolWithTag(
            NonPagedPool,
            core::mem::size_of::<OBJECT_NAME_INFORMATION>() as SIZE_T,
            TAG_SEPA,
        ) as POBJECT_NAME_INFORMATION;
        if !object_name_info.is_null() {
            core::ptr::write_bytes(object_name_info as *mut u8, 0, return_length as usize);
            status = STATUS_SUCCESS;
        }
    }

    if object_name_info.is_null() {
        status = STATUS_NO_MEMORY;
    }

    *audit_info = object_name_info;

    status
}

#[no_mangle]
pub unsafe extern "system" fn SeLocateProcessImageName(
    process: PEPROCESS,
    process_image_name: *mut PUNICODE_STRING,
) -> NTSTATUS {
    let process = opaque_ptr(process);
    let process_image_name = opaque_ptr(process_image_name);
    let mut audit_name: POBJECT_NAME_INFORMATION;
    let mut image_name: PUNICODE_STRING;
    let mut file_object: PFILE_OBJECT = core::ptr::null_mut();
    let mut status: NTSTATUS = STATUS_SUCCESS;

    *process_image_name = core::ptr::null_mut();

    let eprocess = process as *mut _EPROCESS;
    audit_name = (*eprocess).SeAuditProcessCreationInfo.ImageFileName;

    if audit_name.is_null() {
        status = PsReferenceProcessFilePointer(process, &mut file_object);
        if status < 0 {
            return status;
        }

        status = SeInitializeProcessAuditName(file_object, 1u8, &mut audit_name);
        if status >= 0 {
            let old_value = InterlockedCompareExchangePointer(
                core::ptr::addr_of_mut!((*eprocess).SeAuditProcessCreationInfo.ImageFileName)
                    as *mut PVOID,
                audit_name as PVOID,
                core::ptr::null_mut(),
            );
            if !old_value.is_null() {
                ExFreePool(audit_name as PVOID);
            }
        }

        ObDereferenceObject(file_object as PVOID);
        if status < 0 {
            return status;
        }
    }

    audit_name = (*eprocess).SeAuditProcessCreationInfo.ImageFileName;

    image_name = ExAllocatePoolWithTag(
        NonPagedPool,
        (*audit_name).Name.MaximumLength as SIZE_T
            + core::mem::size_of::<_UNICODE_STRING>() as SIZE_T ,
        TAG_SEPA,
    ) as PUNICODE_STRING;
    if image_name.is_null() {
        return STATUS_NO_MEMORY;
    }

    core::ptr::copy_nonoverlapping(
        core::ptr::addr_of!((*audit_name).Name) as *const u8,
        image_name as *mut u8,
        (*audit_name).Name.MaximumLength as usize + core::mem::size_of::<_UNICODE_STRING>(),
    );

    (*image_name).Buffer = (image_name as usize + core::mem::size_of::<_UNICODE_STRING>()) as PWCHAR;

    *process_image_name = image_name;

    status
}

unsafe fn SepAdtCloseObjectAuditAlarm(
    _subsystem_name: PUNICODE_STRING,
    _handle_id: PVOID,
    _sid: PSID,
) {
}

unsafe fn SepAdtPrivilegedServiceAuditAlarm(
    _subject_context: PSECURITY_SUBJECT_CONTEXT,
    _subsystem_name: PUNICODE_STRING,
    _service_name: PUNICODE_STRING,
    _token: PTOKEN,
    _primary_token: PTOKEN,
    _privileges: PPRIVILEGE_SET,
    _access_granted: BOOLEAN,
) {
}

#[no_mangle]
pub unsafe extern "system" fn SePrivilegedServiceAuditAlarm(
    service_name: PUNICODE_STRING,
    subject_context: PSECURITY_SUBJECT_CONTEXT,
    privilege_set: PPRIVILEGE_SET,
    access_granted: BOOLEAN,
) {
    let service_name = opaque_ptr(service_name);
    let subject_context = opaque_ptr(subject_context);
    let privilege_set = opaque_ptr(privilege_set);
    let effective_token: PTOKEN;
    let user_sid: PSID;

    if !(*subject_context).ClientToken.is_null() {
        effective_token = (*subject_context).ClientToken as PTOKEN;
    } else {
        effective_token = (*subject_context).PrimaryToken as PTOKEN;
    }

    user_sid = (*(*effective_token).UserAndGroups).Sid;

    if RtlEqualSid(user_sid, SeLocalSystemSid) != 0 {
        return;
    }

    if RtlEqualSid(user_sid, (*SeExports).SeNetworkServiceSid) != 0
        || RtlEqualSid(user_sid, (*SeExports).SeLocalServiceSid) != 0
    {
        return;
    }

    SepAdtPrivilegedServiceAuditAlarm(
        subject_context,
        &raw mut SeSubsystemName,
        service_name,
        (*subject_context).ClientToken as PTOKEN,
        (*subject_context).PrimaryToken as PTOKEN,
        privilege_set,
        access_granted,
    );
}

unsafe fn SepAccessCheckAndAuditAlarmWorker(
    _subsystem_name: PUNICODE_STRING,
    _handle_id: PVOID,
    _subject_context: PSECURITY_SUBJECT_CONTEXT,
    _object_type_name: PUNICODE_STRING,
    _object_name: PUNICODE_STRING,
    _security_descriptor: PSECURITY_DESCRIPTOR,
    _principal_self_sid: PSID,
    desired_access: ACCESS_MASK,
    _audit_type: AUDIT_EVENT_TYPE,
    _have_audit_privilege: BOOLEAN,
    _object_type_list: POBJECT_TYPE_LIST_INTERNAL,
    object_type_list_length: ULONG,
    _generic_mapping: PGENERIC_MAPPING,
    granted_access_list: PACCESS_MASK,
    access_status_list: PNTSTATUS,
    generate_on_close: PBOOLEAN,
    use_result_list: BOOLEAN,
) -> NTSTATUS {
    let result_list_length: ULONG;

    result_list_length = if use_result_list != 0 {
        object_type_list_length
    } else {
        1
    };

    for i in 0..result_list_length {
        *granted_access_list.offset(i as isize) = desired_access;
        *access_status_list.offset(i as isize) = STATUS_SUCCESS;
    }

    *generate_on_close = 0u8;

    STATUS_SUCCESS
}

unsafe fn SepAccessCheckAndAuditAlarm(
    subsystem_name: PUNICODE_STRING,
    handle_id: PVOID,
    client_token_handle: PHANDLE,
    object_type_name: PUNICODE_STRING,
    object_name: PUNICODE_STRING,
    security_descriptor: PSECURITY_DESCRIPTOR,
    principal_self_sid: PSID,
    desired_access: ACCESS_MASK,
    audit_type: AUDIT_EVENT_TYPE,
    flags: ULONG,
    object_type_list: POBJECT_TYPE_LIST,
    object_type_list_length: ULONG,
    generic_mapping: PGENERIC_MAPPING,
    granted_access_list: PACCESS_MASK,
    access_status_list: PNTSTATUS,
    generate_on_close: PBOOLEAN,
    use_result_list: BOOLEAN,
) -> NTSTATUS {
    let mut subject_context: SECURITY_SUBJECT_CONTEXT = core::mem::zeroed();
    let mut result_list_length: ULONG;
    let mut local_generic_mapping: _GENERIC_MAPPING;
    let mut subject_context_token: PTOKEN = core::ptr::null_mut();
    let mut client_token: PTOKEN = core::ptr::null_mut();
    let mut allocated_result_lists: BOOLEAN;
    let mut have_audit_privilege: BOOLEAN;
    let mut captured_security_descriptor: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
    let mut captured_subsystem_name: _UNICODE_STRING = core::mem::zeroed();
    let mut captured_object_type_name: _UNICODE_STRING = core::mem::zeroed();
    let mut captured_object_name: _UNICODE_STRING = core::mem::zeroed();
    let mut granted_access: ACCESS_MASK = 0;
    let mut access_status: NTSTATUS = 0;
    let mut safe_granted_access_list: PACCESS_MASK;
    let mut safe_access_status_list: PNTSTATUS;
    let mut captured_principal_self_sid: PSID = core::ptr::null_mut();
    let mut captured_object_type_list: POBJECT_TYPE_LIST_INTERNAL = core::ptr::null_mut();
    let mut local_generate_on_close: BOOLEAN = 0;
    let mut status: NTSTATUS;

    allocated_result_lists = 0u8;

    if (audit_type != AuditEventObjectAccess)
        && (audit_type != AuditEventDirectoryServiceAccess)
    {
        return STATUS_INVALID_PARAMETER;
    }

    SeCaptureSubjectContext(&mut subject_context);

    if client_token_handle.is_null() {
        if subject_context.ClientToken.is_null() {
            status = STATUS_NO_IMPERSONATION_TOKEN;
            SeReleaseSubjectContext(&mut subject_context);
            return status;
        }

        if subject_context.ImpersonationLevel < SecurityIdentification {
            status = STATUS_BAD_IMPERSONATION_LEVEL;
            SeReleaseSubjectContext(&mut subject_context);
            return status;
        }
    }

    if use_result_list != 0 {
        result_list_length = object_type_list_length;
        if (result_list_length == 0) || (result_list_length > 0x1000) {
            status = STATUS_INVALID_PARAMETER;
            SeReleaseSubjectContext(&mut subject_context);
            return status;
        }

        safe_granted_access_list = ExAllocatePoolWithTag(
            PagedPool,
            2 * result_list_length as SIZE_T * core::mem::size_of::<ULONG>() as SIZE_T ,
            TAG_SEPA,
        ) as PACCESS_MASK;
        if safe_granted_access_list.is_null() {
            status = STATUS_INSUFFICIENT_RESOURCES;
            SeReleaseSubjectContext(&mut subject_context);
            return status;
        }

        safe_access_status_list = safe_granted_access_list.offset(result_list_length as isize) as PNTSTATUS;
        allocated_result_lists = 1u8;
    } else {
        result_list_length = 1;
        safe_granted_access_list = &mut granted_access;
        safe_access_status_list = &mut access_status;
    }

    if let Err(s) = probe_user_buffer(
        access_status_list as *const core::ffi::c_void,
        (result_list_length as usize) * core::mem::size_of::<NTSTATUS>(),
        core::mem::size_of::<NTSTATUS>(),
        UserMode,
    ) {
        if allocated_result_lists != 0 {
            ExFreePoolWithTag(safe_granted_access_list as PVOID, TAG_SEPA);
        }
        SeReleaseSubjectContext(&mut subject_context);
        return s;
    }

    if let Err(s) = probe_user_buffer(
        granted_access_list as *const core::ffi::c_void,
        (result_list_length as usize) * core::mem::size_of::<ACCESS_MASK>(),
        core::mem::size_of::<ACCESS_MASK>(),
        UserMode,
    ) {
        if allocated_result_lists != 0 {
            ExFreePoolWithTag(safe_granted_access_list as PVOID, TAG_SEPA);
        }
        SeReleaseSubjectContext(&mut subject_context);
        return s;
    }

    if let Err(s) = probe_user_buffer(
        generic_mapping as *const core::ffi::c_void,
        core::mem::size_of::<_GENERIC_MAPPING>(),
        core::mem::size_of::<ULONG>(),
        UserMode,
    ) {
        if allocated_result_lists != 0 {
            ExFreePoolWithTag(safe_granted_access_list as PVOID, TAG_SEPA);
        }
        SeReleaseSubjectContext(&mut subject_context);
        return s;
    }

    local_generic_mapping = *generic_mapping;

    if !client_token_handle.is_null() {
        status = ObReferenceObjectByHandle(
            *client_token_handle,
            TOKEN_QUERY,
            SeTokenObjectType as *mut _OBJECT_TYPE,
            UserMode,
            &mut client_token as *mut PTOKEN as *mut PVOID,
            core::ptr::null_mut(),
        );
        if status < 0 {
            if allocated_result_lists != 0 {
                ExFreePoolWithTag(safe_granted_access_list as PVOID, TAG_SEPA);
            }
            SeReleaseSubjectContext(&mut subject_context);
            return status;
        }

        subject_context_token = subject_context.ClientToken as PTOKEN;
        subject_context.ClientToken = client_token as PACCESS_TOKEN;
    }

    have_audit_privilege = SeCheckAuditPrivilege(&mut subject_context, UserMode);
    if have_audit_privilege == 0 && (flags & AUDIT_ALLOW_NO_PRIVILEGE) == 0 {
        status = STATUS_PRIVILEGE_NOT_HELD;
        if !client_token.is_null() {
            ObDereferenceObject(client_token as PVOID);
            subject_context.ClientToken = subject_context_token as PACCESS_TOKEN;
        }
        if allocated_result_lists != 0 {
            ExFreePoolWithTag(safe_granted_access_list as PVOID, TAG_SEPA);
        }
        SeReleaseSubjectContext(&mut subject_context);
        return status;
    }

    if (desired_access & (GENERIC_READ | GENERIC_WRITE | GENERIC_EXECUTE | GENERIC_ALL)) != 0 {
        status = STATUS_GENERIC_NOT_MAPPED;
        if !client_token.is_null() {
            ObDereferenceObject(client_token as PVOID);
            subject_context.ClientToken = subject_context_token as PACCESS_TOKEN;
        }
        if allocated_result_lists != 0 {
            ExFreePoolWithTag(safe_granted_access_list as PVOID, TAG_SEPA);
        }
        SeReleaseSubjectContext(&mut subject_context);
        return status;
    }

    status = SeCaptureSecurityDescriptor(
        security_descriptor,
        UserMode,
        PagedPool,
        0u8,
        &mut captured_security_descriptor,
    );
    if status < 0 {
        if !client_token.is_null() {
            ObDereferenceObject(client_token as PVOID);
            subject_context.ClientToken = subject_context_token as PACCESS_TOKEN;
        }
        if allocated_result_lists != 0 {
            ExFreePoolWithTag(safe_granted_access_list as PVOID, TAG_SEPA);
        }
        SeReleaseSubjectContext(&mut subject_context);
        return status;
    }

    if SepGetOwnerFromDescriptor(captured_security_descriptor).is_null()
        || SepGetGroupFromDescriptor(captured_security_descriptor).is_null()
    {
        status = STATUS_INVALID_SECURITY_DESCR;
        if !captured_security_descriptor.is_null() {
            SeReleaseSecurityDescriptor(captured_security_descriptor, UserMode, 0u8);
        }
        if !client_token.is_null() {
            ObDereferenceObject(client_token as PVOID);
            subject_context.ClientToken = subject_context_token as PACCESS_TOKEN;
        }
        if allocated_result_lists != 0 {
            ExFreePoolWithTag(safe_granted_access_list as PVOID, TAG_SEPA);
        }
        SeReleaseSubjectContext(&mut subject_context);
        return status;
    }

    status = ProbeAndCaptureUnicodeString(
        &mut captured_subsystem_name,
        UserMode,
        subsystem_name,
    );
    if status < 0 {
        if !captured_security_descriptor.is_null() {
            SeReleaseSecurityDescriptor(captured_security_descriptor, UserMode, 0u8);
        }
        if !client_token.is_null() {
            ObDereferenceObject(client_token as PVOID);
            subject_context.ClientToken = subject_context_token as PACCESS_TOKEN;
        }
        if allocated_result_lists != 0 {
            ExFreePoolWithTag(safe_granted_access_list as PVOID, TAG_SEPA);
        }
        SeReleaseSubjectContext(&mut subject_context);
        return status;
    }

    status = ProbeAndCaptureUnicodeString(
        &mut captured_object_type_name,
        UserMode,
        object_type_name,
    );
    if status < 0 {
        if !captured_subsystem_name.Buffer.is_null() {
            ReleaseCapturedUnicodeString(&mut captured_subsystem_name, UserMode);
        }
        if !captured_security_descriptor.is_null() {
            SeReleaseSecurityDescriptor(captured_security_descriptor, UserMode, 0u8);
        }
        if !client_token.is_null() {
            ObDereferenceObject(client_token as PVOID);
            subject_context.ClientToken = subject_context_token as PACCESS_TOKEN;
        }
        if allocated_result_lists != 0 {
            ExFreePoolWithTag(safe_granted_access_list as PVOID, TAG_SEPA);
        }
        SeReleaseSubjectContext(&mut subject_context);
        return status;
    }

    status = ProbeAndCaptureUnicodeString(
        &mut captured_object_name,
        UserMode,
        object_name,
    );
    if status < 0 {
        if !captured_object_type_name.Buffer.is_null() {
            ReleaseCapturedUnicodeString(&mut captured_object_type_name, UserMode);
        }
        if !captured_subsystem_name.Buffer.is_null() {
            ReleaseCapturedUnicodeString(&mut captured_subsystem_name, UserMode);
        }
        if !captured_security_descriptor.is_null() {
            SeReleaseSecurityDescriptor(captured_security_descriptor, UserMode, 0u8);
        }
        if !client_token.is_null() {
            ObDereferenceObject(client_token as PVOID);
            subject_context.ClientToken = subject_context_token as PACCESS_TOKEN;
        }
        if allocated_result_lists != 0 {
            ExFreePoolWithTag(safe_granted_access_list as PVOID, TAG_SEPA);
        }
        SeReleaseSubjectContext(&mut subject_context);
        return status;
    }

    if !principal_self_sid.is_null() {
        status = SepCaptureSid(
            principal_self_sid,
            UserMode,
            PagedPool,
            0u8,
            &mut captured_principal_self_sid,
        );
        if status < 0 {
            if !captured_object_name.Buffer.is_null() {
                ReleaseCapturedUnicodeString(&mut captured_object_name, UserMode);
            }
            if !captured_object_type_name.Buffer.is_null() {
                ReleaseCapturedUnicodeString(&mut captured_object_type_name, UserMode);
            }
            if !captured_subsystem_name.Buffer.is_null() {
                ReleaseCapturedUnicodeString(&mut captured_subsystem_name, UserMode);
            }
            if !captured_security_descriptor.is_null() {
                SeReleaseSecurityDescriptor(captured_security_descriptor, UserMode, 0u8);
            }
            if !client_token.is_null() {
                ObDereferenceObject(client_token as PVOID);
                subject_context.ClientToken = subject_context_token as PACCESS_TOKEN;
            }
            if allocated_result_lists != 0 {
                ExFreePoolWithTag(safe_granted_access_list as PVOID, TAG_SEPA);
            }
            SeReleaseSubjectContext(&mut subject_context);
            return status;
        }
    }

    status = SeCaptureObjectTypeList(
        object_type_list,
        object_type_list_length,
        UserMode,
        &mut captured_object_type_list,
    );
    if status < 0 {
        if !captured_principal_self_sid.is_null() {
            SepReleaseSid(captured_principal_self_sid, UserMode, 0u8);
        }
        if !captured_object_name.Buffer.is_null() {
            ReleaseCapturedUnicodeString(&mut captured_object_name, UserMode);
        }
        if !captured_object_type_name.Buffer.is_null() {
            ReleaseCapturedUnicodeString(&mut captured_object_type_name, UserMode);
        }
        if !captured_subsystem_name.Buffer.is_null() {
            ReleaseCapturedUnicodeString(&mut captured_subsystem_name, UserMode);
        }
        if !captured_security_descriptor.is_null() {
            SeReleaseSecurityDescriptor(captured_security_descriptor, UserMode, 0u8);
        }
        if !client_token.is_null() {
            ObDereferenceObject(client_token as PVOID);
            subject_context.ClientToken = subject_context_token as PACCESS_TOKEN;
        }
        if allocated_result_lists != 0 {
            ExFreePoolWithTag(safe_granted_access_list as PVOID, TAG_SEPA);
        }
        SeReleaseSubjectContext(&mut subject_context);
        return status;
    }

    status = SepAccessCheckAndAuditAlarmWorker(
        &mut captured_subsystem_name,
        handle_id,
        &mut subject_context,
        &mut captured_object_type_name,
        &mut captured_object_name,
        captured_security_descriptor,
        captured_principal_self_sid,
        desired_access,
        audit_type,
        have_audit_privilege,
        captured_object_type_list,
        object_type_list_length,
        &mut local_generic_mapping,
        safe_granted_access_list,
        safe_access_status_list,
        &mut local_generate_on_close,
        use_result_list,
    );

    if status >= 0 {
        for i in 0..result_list_length {
            *access_status_list.offset(i as isize) = *safe_access_status_list.offset(i as isize);
            *granted_access_list.offset(i as isize) = *safe_granted_access_list.offset(i as isize);
        }
        // Write-back to user-mode pointer
        if let Err(s) = probe_write_user(generate_on_close, local_generate_on_close, UserMode) {
            status = s;
        }
    }

    if !captured_object_type_list.is_null() {
        SeReleaseObjectTypeList(captured_object_type_list, UserMode);
    }
    if !captured_principal_self_sid.is_null() {
        SepReleaseSid(captured_principal_self_sid, UserMode, 0u8);
    }
    if !captured_object_name.Buffer.is_null() {
        ReleaseCapturedUnicodeString(&mut captured_object_name, UserMode);
    }
    if !captured_object_type_name.Buffer.is_null() {
        ReleaseCapturedUnicodeString(&mut captured_object_type_name, UserMode);
    }
    if !captured_subsystem_name.Buffer.is_null() {
        ReleaseCapturedUnicodeString(&mut captured_subsystem_name, UserMode);
    }
    if !captured_security_descriptor.is_null() {
        SeReleaseSecurityDescriptor(captured_security_descriptor, UserMode, 0u8);
    }
    if !client_token.is_null() {
        ObDereferenceObject(client_token as PVOID);
        subject_context.ClientToken = subject_context_token as PACCESS_TOKEN;
    }
    if allocated_result_lists != 0 {
        ExFreePoolWithTag(safe_granted_access_list as PVOID, TAG_SEPA);
    }

    SeReleaseSubjectContext(&mut subject_context);

    status
}

#[no_mangle]
pub unsafe extern "system" fn SeAuditHardLinkCreation(
    _file_name: PUNICODE_STRING,
    _link_name: PUNICODE_STRING,
    _b_success: BOOLEAN,
) {
    let _file_name = opaque_ptr(_file_name);
    let _link_name = opaque_ptr(_link_name);
}

#[no_mangle]
pub unsafe extern "system" fn SeAuditingFileEvents(
    _access_granted: BOOLEAN,
    _security_descriptor: PSECURITY_DESCRIPTOR,
) -> BOOLEAN {
    let _security_descriptor = opaque_ptr(_security_descriptor);
    0u8
}

#[no_mangle]
pub unsafe extern "system" fn SeAuditingFileEventsWithContext(
    _access_granted: BOOLEAN,
    _security_descriptor: PSECURITY_DESCRIPTOR,
    _subject_security_context: PSECURITY_SUBJECT_CONTEXT,
) -> BOOLEAN {
    let _security_descriptor = opaque_ptr(_security_descriptor);
    let _subject_security_context = opaque_ptr(_subject_security_context);
    0u8
}

#[no_mangle]
pub unsafe extern "system" fn SeAuditingHardLinkEvents(
    _access_granted: BOOLEAN,
    _security_descriptor: PSECURITY_DESCRIPTOR,
) -> BOOLEAN {
    let _security_descriptor = opaque_ptr(_security_descriptor);
    0u8
}

#[no_mangle]
pub unsafe extern "system" fn SeAuditingHardLinkEventsWithContext(
    _access_granted: BOOLEAN,
    _security_descriptor: PSECURITY_DESCRIPTOR,
    _subject_security_context: PSECURITY_SUBJECT_CONTEXT,
) -> BOOLEAN {
    let _security_descriptor = opaque_ptr(_security_descriptor);
    let _subject_security_context = opaque_ptr(_subject_security_context);
    0u8
}

#[no_mangle]
pub unsafe extern "system" fn SeAuditingFileOrGlobalEvents(
    _access_granted: BOOLEAN,
    _security_descriptor: PSECURITY_DESCRIPTOR,
    _subject_security_context: PSECURITY_SUBJECT_CONTEXT,
) -> BOOLEAN {
    let _security_descriptor = opaque_ptr(_security_descriptor);
    let _subject_security_context = opaque_ptr(_subject_security_context);
    0u8
}

#[no_mangle]
pub unsafe extern "system" fn SeCloseObjectAuditAlarm(
    _object: PVOID,
    _handle: HANDLE,
    _perform_action: BOOLEAN,
) {
    let _object = opaque_ptr(_object);
    let _handle = opaque_ptr(_handle);
}

#[no_mangle]
pub unsafe extern "system" fn SeDeleteObjectAuditAlarm(
    _object: PVOID,
    _handle: HANDLE,
) {
    let _object = opaque_ptr(_object);
    let _handle = opaque_ptr(_handle);
}

#[no_mangle]
pub unsafe extern "system" fn SeOpenObjectAuditAlarm(
    _object_type_name: PUNICODE_STRING,
    _object: PVOID,
    _absolute_object_name: PUNICODE_STRING,
    _security_descriptor: PSECURITY_DESCRIPTOR,
    _access_state: PACCESS_STATE,
    _object_created: BOOLEAN,
    _access_granted: BOOLEAN,
    access_mode: KPROCESSOR_MODE,
    _generate_on_close: PBOOLEAN,
) {
    let _object_type_name = opaque_ptr(_object_type_name);
    let _object = opaque_ptr(_object);
    let _absolute_object_name = opaque_ptr(_absolute_object_name);
    let _security_descriptor = opaque_ptr(_security_descriptor);
    let _access_state = opaque_ptr(_access_state);
    let _generate_on_close = opaque_ptr(_generate_on_close);
    if access_mode == KernelMode {
        return;
    }
}

#[no_mangle]
pub unsafe extern "system" fn SeOpenObjectForDeleteAuditAlarm(
    _object_type_name: PUNICODE_STRING,
    _object: PVOID,
    _absolute_object_name: PUNICODE_STRING,
    _security_descriptor: PSECURITY_DESCRIPTOR,
    _access_state: PACCESS_STATE,
    _object_created: BOOLEAN,
    _access_granted: BOOLEAN,
    _access_mode: KPROCESSOR_MODE,
    _generate_on_close: PBOOLEAN,
) {
    let _object_type_name = opaque_ptr(_object_type_name);
    let _object = opaque_ptr(_object);
    let _absolute_object_name = opaque_ptr(_absolute_object_name);
    let _security_descriptor = opaque_ptr(_security_descriptor);
    let _access_state = opaque_ptr(_access_state);
    let _generate_on_close = opaque_ptr(_generate_on_close);
}

#[no_mangle]
pub unsafe extern "system" fn SePrivilegeObjectAuditAlarm(
    _handle: HANDLE,
    _subject_context: PSECURITY_SUBJECT_CONTEXT,
    _desired_access: ACCESS_MASK,
    _privileges: PPRIVILEGE_SET,
    _access_granted: BOOLEAN,
    _current_mode: KPROCESSOR_MODE,
) {
    let _handle = opaque_ptr(_handle);
    let _subject_context = opaque_ptr(_subject_context);
    let _privileges = opaque_ptr(_privileges);
}

#[no_mangle]
pub unsafe extern "system" fn NtCloseObjectAuditAlarm(
    subsystem_name: PUNICODE_STRING,
    handle_id: PVOID,
    generate_on_close: BOOLEAN,
) -> NTSTATUS {
    let subsystem_name = opaque_ptr(subsystem_name);
    let handle_id = opaque_ptr(handle_id);
    let mut subject_context: SECURITY_SUBJECT_CONTEXT = core::mem::zeroed();
    let mut captured_subsystem_name: _UNICODE_STRING = core::mem::zeroed();
    let previous_mode: KPROCESSOR_MODE;
    let use_impersonation_token: BOOLEAN;
    let current_thread: PETHREAD;
    let mut copy_on_open: BOOLEAN = 0;
    let mut effective_only: BOOLEAN = 0;
    let mut impersonation_level: SECURITY_IMPERSONATION_LEVEL = 0;
    let mut status: NTSTATUS;
    let mut token: PTOKEN;

    previous_mode = ExGetPreviousMode();

    if generate_on_close == 0 {
        return STATUS_SUCCESS;
    }

    SeCaptureSubjectContext(&mut subject_context);

    if SeCheckAuditPrivilege(&mut subject_context, previous_mode) == 0 {
        status = STATUS_PRIVILEGE_NOT_HELD;
        SeReleaseSubjectContext(&mut subject_context);
        return status;
    }

    status = ProbeAndCaptureUnicodeString(
        &mut captured_subsystem_name,
        previous_mode,
        subsystem_name,
    );
    if status < 0 {
        SeReleaseSubjectContext(&mut subject_context);
        return status;
    }

    current_thread = PsGetCurrentThread();
    if PsIsThreadImpersonating(current_thread) != 0 {
        token = PsReferenceImpersonationToken(
            current_thread,
            &mut copy_on_open,
            &mut effective_only,
            &mut impersonation_level,
        ) as PTOKEN;
        use_impersonation_token = 1u8;
    } else {
        token = PsReferencePrimaryToken(PsGetCurrentProcess() as *mut _EPROCESS) as PTOKEN;
        use_impersonation_token = 0u8;
    }

    SepAdtCloseObjectAuditAlarm(
        &mut captured_subsystem_name,
        handle_id,
        (*(*token).UserAndGroups).Sid,
    );

    ReleaseCapturedUnicodeString(&mut captured_subsystem_name, previous_mode);

    if use_impersonation_token != 0 {
        PsDereferenceImpersonationToken(token as PVOID);
    } else {
        PsDereferencePrimaryToken(token as PVOID);
    }

    status = STATUS_SUCCESS;

    SeReleaseSubjectContext(&mut subject_context);

    status
}

#[no_mangle]
pub unsafe extern "system" fn NtDeleteObjectAuditAlarm(
    _subsystem_name: PUNICODE_STRING,
    _handle_id: PVOID,
    _generate_on_close: BOOLEAN,
) -> NTSTATUS {
    let _subsystem_name = opaque_ptr(_subsystem_name);
    let _handle_id = opaque_ptr(_handle_id);
    STATUS_NOT_IMPLEMENTED
}

unsafe fn SepOpenObjectAuditAlarm(
    _subject_context: PSECURITY_SUBJECT_CONTEXT,
    _subsystem_name: PUNICODE_STRING,
    _handle_id: PVOID,
    _object_type_name: PUNICODE_STRING,
    _object_name: PUNICODE_STRING,
    _security_descriptor: PSECURITY_DESCRIPTOR,
    _client_token: PTOKEN,
    _desired_access: ACCESS_MASK,
    _granted_access: ACCESS_MASK,
    _privileges: PPRIVILEGE_SET,
    _object_creation: BOOLEAN,
    _access_granted: BOOLEAN,
    generate_on_close: PBOOLEAN,
) {
    *generate_on_close = 0u8;
}

#[no_mangle]
pub unsafe extern "system" fn NtOpenObjectAuditAlarm(
    subsystem_name: PUNICODE_STRING,
    handle_id: PVOID,
    object_type_name: PUNICODE_STRING,
    object_name: PUNICODE_STRING,
    security_descriptor: PSECURITY_DESCRIPTOR,
    client_token_handle: HANDLE,
    desired_access: ACCESS_MASK,
    granted_access: ACCESS_MASK,
    privilege_set: PPRIVILEGE_SET,
    object_creation: BOOLEAN,
    access_granted: BOOLEAN,
    generate_on_close: PBOOLEAN,
) -> NTSTATUS {
    let subsystem_name = opaque_ptr(subsystem_name);
    let handle_id = opaque_ptr(handle_id);
    let object_type_name = opaque_ptr(object_type_name);
    let object_name = opaque_ptr(object_name);
    let security_descriptor = opaque_ptr(security_descriptor);
    let client_token_handle = opaque_ptr(client_token_handle);
    let privilege_set = opaque_ptr(privilege_set);
    let generate_on_close = opaque_ptr(generate_on_close);
    let mut client_token: PTOKEN = core::ptr::null_mut();
    let mut captured_security_descriptor: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
    let mut captured_subsystem_name: _UNICODE_STRING = core::mem::zeroed();
    let mut captured_object_type_name: _UNICODE_STRING = core::mem::zeroed();
    let mut captured_object_name: _UNICODE_STRING = core::mem::zeroed();
    let mut privilege_count: ULONG;
    let mut privilege_set_size: ULONG;
    let mut captured_privilege_set: PPRIVILEGE_SET = core::ptr::null_mut();
    let mut local_generate_on_close: BOOLEAN = 0;
    let mut captured_handle_id: PVOID = core::ptr::null_mut();
    let mut subject_context: SECURITY_SUBJECT_CONTEXT = core::mem::zeroed();
    let mut status: NTSTATUS;

    status = ObReferenceObjectByHandle(
        client_token_handle,
        TOKEN_QUERY,
        SeTokenObjectType as *mut _OBJECT_TYPE,
        UserMode,
        &mut client_token as *mut PTOKEN as *mut PVOID,
        core::ptr::null_mut(),
    );
    if status < 0 {
        return status;
    }

    let _obj_guard = ObjRefGuard::new(client_token as PVOID);

    SeCaptureSubjectContext(&mut subject_context);

    if ((*client_token).TokenType == TokenImpersonation)
        && ((*client_token).ImpersonationLevel < SecurityIdentification)
    {
        status = STATUS_BAD_IMPERSONATION_LEVEL;
        SeReleaseSubjectContext(&mut subject_context);
        return status;
    }

    if SeCheckAuditPrivilege(&mut subject_context, UserMode) == 0 {
        status = STATUS_PRIVILEGE_NOT_HELD;
        SeReleaseSubjectContext(&mut subject_context);
        return status;
    }

    if security_descriptor.is_null() {
        status = STATUS_SUCCESS;
        SeReleaseSubjectContext(&mut subject_context);
        return status;
    }

    status = SeCaptureSecurityDescriptor(
        security_descriptor,
        UserMode,
        PagedPool,
        0u8,
        &mut captured_security_descriptor,
    );
    if status < 0 {
        SeReleaseSubjectContext(&mut subject_context);
        return status;
    }

    if !privilege_set.is_null() {
        if let Err(s) = probe_user_buffer(
            privilege_set as *const core::ffi::c_void,
            core::mem::size_of::<PRIVILEGE_SET>(),
            core::mem::size_of::<ULONG>(),
            UserMode,
        ) {
            if !captured_security_descriptor.is_null() {
                SeReleaseSecurityDescriptor(captured_security_descriptor, UserMode, 0u8);
            }
            SeReleaseSubjectContext(&mut subject_context);
            return s;
        }

        privilege_count = (*privilege_set).PrivilegeCount;
        if privilege_count > SEP_PRIVILEGE_SET_MAX_COUNT {
            status = STATUS_INVALID_PARAMETER;
            if !captured_security_descriptor.is_null() {
                SeReleaseSecurityDescriptor(captured_security_descriptor, UserMode, 0u8);
            }
            SeReleaseSubjectContext(&mut subject_context);
            return status;
        }

        privilege_set_size = core::mem::size_of::<PRIVILEGE_SET>() as ULONG
            + (privilege_count.saturating_sub(1)) * core::mem::size_of::<LUID_AND_ATTRIBUTES>() as ULONG;

        if let Err(s) = probe_user_buffer(
            privilege_set as *const core::ffi::c_void,
            privilege_set_size as usize,
            core::mem::size_of::<ULONG>(),
            UserMode,
        ) {
            if !captured_security_descriptor.is_null() {
                SeReleaseSecurityDescriptor(captured_security_descriptor, UserMode, 0u8);
            }
            SeReleaseSubjectContext(&mut subject_context);
            return s;
        }

        captured_privilege_set = ExAllocatePoolWithTag(
            PagedPool,
            privilege_set_size as SIZE_T,
            TAG_PRIVILEGE_SET,
        ) as PPRIVILEGE_SET;
        if captured_privilege_set.is_null() {
            status = STATUS_INSUFFICIENT_RESOURCES;
            if !captured_security_descriptor.is_null() {
                SeReleaseSecurityDescriptor(captured_security_descriptor, UserMode, 0u8);
            }
            SeReleaseSubjectContext(&mut subject_context);
            return status;
        }

        let _pool_guard = PoolAllocGuard::new(captured_privilege_set as PVOID, TAG_PRIVILEGE_SET);

        if let Err(s) = safe_copy_from_user(
            captured_privilege_set as PVOID,
            privilege_set as *const core::ffi::c_void,
            privilege_set_size as usize,
            core::mem::align_of::<ULONG>(),
            UserMode,
        ) {
            if !captured_security_descriptor.is_null() {
                SeReleaseSecurityDescriptor(captured_security_descriptor, UserMode, 0u8);
            }
            SeReleaseSubjectContext(&mut subject_context);
            return s;
        }

        if !handle_id.is_null() {
            if let Err(s) = probe_user_buffer(
                handle_id,
                core::mem::size_of::<PVOID>(),
                core::mem::size_of::<PVOID>(),
                UserMode,
            ) {
                if !captured_security_descriptor.is_null() {
                    SeReleaseSecurityDescriptor(captured_security_descriptor, UserMode, 0u8);
                }
                SeReleaseSubjectContext(&mut subject_context);
                return s;
            }
            captured_handle_id = *(handle_id as *const PVOID);
        }

        if let Err(s) = probe_user_buffer(
            generate_on_close as *const core::ffi::c_void,
            core::mem::size_of::<BOOLEAN>(),
            core::mem::size_of::<BOOLEAN>(),
            UserMode,
        ) {
            if !captured_security_descriptor.is_null() {
                SeReleaseSecurityDescriptor(captured_security_descriptor, UserMode, 0u8);
            }
            SeReleaseSubjectContext(&mut subject_context);
            return s;
        }

        status = ProbeAndCaptureUnicodeString(
            &mut captured_subsystem_name,
            UserMode,
            subsystem_name,
        );
        if status < 0 {
            if !captured_security_descriptor.is_null() {
                SeReleaseSecurityDescriptor(captured_security_descriptor, UserMode, 0u8);
            }
            SeReleaseSubjectContext(&mut subject_context);
            return status;
        }

        status = ProbeAndCaptureUnicodeString(
            &mut captured_object_type_name,
            UserMode,
            object_type_name,
        );
        if status < 0 {
            if !captured_subsystem_name.Buffer.is_null() {
                ReleaseCapturedUnicodeString(&mut captured_subsystem_name, UserMode);
            }
            if !captured_security_descriptor.is_null() {
                SeReleaseSecurityDescriptor(captured_security_descriptor, UserMode, 0u8);
            }
            SeReleaseSubjectContext(&mut subject_context);
            return status;
        }

        status = ProbeAndCaptureUnicodeString(
            &mut captured_object_name,
            UserMode,
            object_name,
        );
        if status < 0 {
            if !captured_object_type_name.Buffer.is_null() {
                ReleaseCapturedUnicodeString(&mut captured_object_type_name, UserMode);
            }
            if !captured_subsystem_name.Buffer.is_null() {
                ReleaseCapturedUnicodeString(&mut captured_subsystem_name, UserMode);
            }
            if !captured_security_descriptor.is_null() {
                SeReleaseSecurityDescriptor(captured_security_descriptor, UserMode, 0u8);
            }
            SeReleaseSubjectContext(&mut subject_context);
            return status;
        }

        SepOpenObjectAuditAlarm(
            &mut subject_context,
            &mut captured_subsystem_name,
            captured_handle_id,
            &mut captured_object_type_name,
            &mut captured_object_name,
            captured_security_descriptor,
            client_token,
            desired_access,
            granted_access,
            captured_privilege_set,
            object_creation,
            access_granted,
            &mut local_generate_on_close,
        );

        status = STATUS_SUCCESS;

        // Write-back to user-mode pointer
        if let Err(s) = probe_write_user(generate_on_close, local_generate_on_close, UserMode) {
            status = s;
        }

        if !captured_object_name.Buffer.is_null() {
            ReleaseCapturedUnicodeString(&mut captured_object_name, UserMode);
        }
        if !captured_object_type_name.Buffer.is_null() {
            ReleaseCapturedUnicodeString(&mut captured_object_type_name, UserMode);
        }
        if !captured_subsystem_name.Buffer.is_null() {
            ReleaseCapturedUnicodeString(&mut captured_subsystem_name, UserMode);
        }
        if !captured_security_descriptor.is_null() {
            SeReleaseSecurityDescriptor(captured_security_descriptor, UserMode, 0u8);
        }

        _pool_guard.disarm();
        SeReleaseSubjectContext(&mut subject_context);

        return status;
    }

    // privilege_set is null path
    if !handle_id.is_null() {
        if let Err(s) = probe_user_buffer(
            handle_id,
            core::mem::size_of::<PVOID>(),
            core::mem::size_of::<PVOID>(),
            UserMode,
        ) {
            if !captured_security_descriptor.is_null() {
                SeReleaseSecurityDescriptor(captured_security_descriptor, UserMode, 0u8);
            }
            SeReleaseSubjectContext(&mut subject_context);
            return s;
        }
        captured_handle_id = *(handle_id as *const PVOID);
    }

    if let Err(s) = probe_user_buffer(
        generate_on_close as *const core::ffi::c_void,
        core::mem::size_of::<BOOLEAN>(),
        core::mem::size_of::<BOOLEAN>(),
        UserMode,
    ) {
        if !captured_security_descriptor.is_null() {
            SeReleaseSecurityDescriptor(captured_security_descriptor, UserMode, 0u8);
        }
        SeReleaseSubjectContext(&mut subject_context);
        return s;
    }

    status = ProbeAndCaptureUnicodeString(
        &mut captured_subsystem_name,
        UserMode,
        subsystem_name,
    );
    if status < 0 {
        if !captured_security_descriptor.is_null() {
            SeReleaseSecurityDescriptor(captured_security_descriptor, UserMode, 0u8);
        }
        SeReleaseSubjectContext(&mut subject_context);
        return status;
    }

    status = ProbeAndCaptureUnicodeString(
        &mut captured_object_type_name,
        UserMode,
        object_type_name,
    );
    if status < 0 {
        if !captured_subsystem_name.Buffer.is_null() {
            ReleaseCapturedUnicodeString(&mut captured_subsystem_name, UserMode);
        }
        if !captured_security_descriptor.is_null() {
            SeReleaseSecurityDescriptor(captured_security_descriptor, UserMode, 0u8);
        }
        SeReleaseSubjectContext(&mut subject_context);
        return status;
    }

    status = ProbeAndCaptureUnicodeString(
        &mut captured_object_name,
        UserMode,
        object_name,
    );
    if status < 0 {
        if !captured_object_type_name.Buffer.is_null() {
            ReleaseCapturedUnicodeString(&mut captured_object_type_name, UserMode);
        }
        if !captured_subsystem_name.Buffer.is_null() {
            ReleaseCapturedUnicodeString(&mut captured_subsystem_name, UserMode);
        }
        if !captured_security_descriptor.is_null() {
            SeReleaseSecurityDescriptor(captured_security_descriptor, UserMode, 0u8);
        }
        SeReleaseSubjectContext(&mut subject_context);
        return status;
    }

    SepOpenObjectAuditAlarm(
        &mut subject_context,
        &mut captured_subsystem_name,
        captured_handle_id,
        &mut captured_object_type_name,
        &mut captured_object_name,
        captured_security_descriptor,
        client_token,
        desired_access,
        granted_access,
        captured_privilege_set,
        object_creation,
        access_granted,
        &mut local_generate_on_close,
    );

    status = STATUS_SUCCESS;

    // Write-back to user-mode pointer
    if let Err(s) = probe_write_user(generate_on_close, local_generate_on_close, UserMode) {
        status = s;
    }

    if !captured_object_name.Buffer.is_null() {
        ReleaseCapturedUnicodeString(&mut captured_object_name, UserMode);
    }
    if !captured_object_type_name.Buffer.is_null() {
        ReleaseCapturedUnicodeString(&mut captured_object_type_name, UserMode);
    }
    if !captured_subsystem_name.Buffer.is_null() {
        ReleaseCapturedUnicodeString(&mut captured_subsystem_name, UserMode);
    }
    if !captured_security_descriptor.is_null() {
        SeReleaseSecurityDescriptor(captured_security_descriptor, UserMode, 0u8);
    }

    SeReleaseSubjectContext(&mut subject_context);

    status
}

#[no_mangle]
pub unsafe extern "system" fn NtPrivilegedServiceAuditAlarm(
    subsystem_name: PUNICODE_STRING,
    service_name: PUNICODE_STRING,
    client_token_handle: HANDLE,
    privileges: PPRIVILEGE_SET,
    access_granted: BOOLEAN,
) -> NTSTATUS {
    let subsystem_name = opaque_ptr(subsystem_name);
    let service_name = opaque_ptr(service_name);
    let client_token_handle = opaque_ptr(client_token_handle);
    let privileges = opaque_ptr(privileges);
    let previous_mode: KPROCESSOR_MODE;
    let mut client_token: PTOKEN = core::ptr::null_mut();
    let mut captured_privileges: PPRIVILEGE_SET = core::ptr::null_mut();
    let mut captured_subsystem_name: _UNICODE_STRING = core::mem::zeroed();
    let mut captured_service_name: _UNICODE_STRING = core::mem::zeroed();
    let mut privilege_count: ULONG;
    let mut privileges_size: ULONG;
    let mut subject_context: SECURITY_SUBJECT_CONTEXT = core::mem::zeroed();
    let mut status: NTSTATUS;

    previous_mode = ExGetPreviousMode();

    status = ObReferenceObjectByHandle(
        client_token_handle,
        TOKEN_QUERY,
        SeTokenObjectType as *mut _OBJECT_TYPE,
        previous_mode,
        &mut client_token as *mut PTOKEN as *mut PVOID,
        core::ptr::null_mut(),
    );
    if status < 0 {
        return status;
    }

    let _obj_guard = ObjRefGuard::new(client_token as PVOID);

    if ((*client_token).TokenType == TokenImpersonation)
        && ((*client_token).ImpersonationLevel < SecurityIdentification)
    {
        return STATUS_BAD_IMPERSONATION_LEVEL;
    }

    SeCaptureSubjectContext(&mut subject_context);

    if SeCheckAuditPrivilege(&mut subject_context, previous_mode) == 0 {
        status = STATUS_PRIVILEGE_NOT_HELD;
        SeReleaseSubjectContext(&mut subject_context);
        return status;
    }

    if !subsystem_name.is_null() {
        status = ProbeAndCaptureUnicodeString(
            &mut captured_subsystem_name,
            previous_mode,
            subsystem_name,
        );
        if status < 0 {
            SeReleaseSubjectContext(&mut subject_context);
            return status;
        }
    }

    if !service_name.is_null() {
        status = ProbeAndCaptureUnicodeString(
            &mut captured_service_name,
            previous_mode,
            service_name,
        );
        if status < 0 {
            if !captured_subsystem_name.Buffer.is_null() {
                ReleaseCapturedUnicodeString(&mut captured_subsystem_name, previous_mode);
            }
            SeReleaseSubjectContext(&mut subject_context);
            return status;
        }
    }

    if let Err(s) = probe_user_buffer(
        privileges as *const core::ffi::c_void,
        core::mem::size_of::<PRIVILEGE_SET>(),
        core::mem::size_of::<ULONG>(),
        previous_mode,
    ) {
        if !captured_service_name.Buffer.is_null() {
            ReleaseCapturedUnicodeString(&mut captured_service_name, previous_mode);
        }
        if !captured_subsystem_name.Buffer.is_null() {
            ReleaseCapturedUnicodeString(&mut captured_subsystem_name, previous_mode);
        }
        SeReleaseSubjectContext(&mut subject_context);
        return s;
    }

    privilege_count = (*privileges).PrivilegeCount;
    if privilege_count > SEP_PRIVILEGE_SET_MAX_COUNT {
        status = STATUS_INVALID_PARAMETER;
        if !captured_service_name.Buffer.is_null() {
            ReleaseCapturedUnicodeString(&mut captured_service_name, previous_mode);
        }
        if !captured_subsystem_name.Buffer.is_null() {
            ReleaseCapturedUnicodeString(&mut captured_subsystem_name, previous_mode);
        }
        SeReleaseSubjectContext(&mut subject_context);
        return status;
    }

    privileges_size = core::mem::size_of::<PRIVILEGE_SET>() as ULONG
        + (privilege_count.saturating_sub(1)) * core::mem::size_of::<LUID_AND_ATTRIBUTES>() as ULONG;

    if let Err(s) = probe_user_buffer(
        privileges as *const core::ffi::c_void,
        privileges_size as usize,
        core::mem::size_of::<ULONG>(),
        previous_mode,
    ) {
        if !captured_service_name.Buffer.is_null() {
            ReleaseCapturedUnicodeString(&mut captured_service_name, previous_mode);
        }
        if !captured_subsystem_name.Buffer.is_null() {
            ReleaseCapturedUnicodeString(&mut captured_subsystem_name, previous_mode);
        }
        SeReleaseSubjectContext(&mut subject_context);
        return s;
    }

    captured_privileges = ExAllocatePoolWithTag(
        PagedPool,
        privileges_size as SIZE_T,
        TAG_PRIVILEGE_SET,
    ) as PPRIVILEGE_SET;
    if captured_privileges.is_null() {
        status = STATUS_INSUFFICIENT_RESOURCES;
        if !captured_service_name.Buffer.is_null() {
            ReleaseCapturedUnicodeString(&mut captured_service_name, previous_mode);
        }
        if !captured_subsystem_name.Buffer.is_null() {
            ReleaseCapturedUnicodeString(&mut captured_subsystem_name, previous_mode);
        }
        SeReleaseSubjectContext(&mut subject_context);
        return status;
    }

    let _pool_guard = PoolAllocGuard::new(captured_privileges as PVOID, TAG_PRIVILEGE_SET);

    if let Err(s) = safe_copy_from_user(
        captured_privileges as PVOID,
        privileges as *const core::ffi::c_void,
        privileges_size as usize,
        core::mem::align_of::<ULONG>(),
        previous_mode,
    ) {
        if !captured_service_name.Buffer.is_null() {
            ReleaseCapturedUnicodeString(&mut captured_service_name, previous_mode);
        }
        if !captured_subsystem_name.Buffer.is_null() {
            ReleaseCapturedUnicodeString(&mut captured_subsystem_name, previous_mode);
        }
        SeReleaseSubjectContext(&mut subject_context);
        return s;
    }

    SepAdtPrivilegedServiceAuditAlarm(
        &mut subject_context,
        if !subsystem_name.is_null() {
            &mut captured_subsystem_name
        } else {
            core::ptr::null_mut()
        },
        if !service_name.is_null() {
            &mut captured_service_name
        } else {
            core::ptr::null_mut()
        },
        client_token,
        subject_context.PrimaryToken as PTOKEN,
        captured_privileges,
        access_granted,
    );

    status = STATUS_SUCCESS;

    if !captured_subsystem_name.Buffer.is_null() {
        ReleaseCapturedUnicodeString(&mut captured_subsystem_name, previous_mode);
    }
    if !captured_service_name.Buffer.is_null() {
        ReleaseCapturedUnicodeString(&mut captured_service_name, previous_mode);
    }

    _pool_guard.disarm();
    SeReleaseSubjectContext(&mut subject_context);

    status
}

#[no_mangle]
pub unsafe extern "system" fn NtPrivilegeObjectAuditAlarm(
    _subsystem_name: PUNICODE_STRING,
    _handle_id: PVOID,
    _client_token: HANDLE,
    _desired_access: ULONG,
    _privileges: PPRIVILEGE_SET,
    _access_granted: BOOLEAN,
) -> NTSTATUS {
    let _subsystem_name = opaque_ptr(_subsystem_name);
    let _handle_id = opaque_ptr(_handle_id);
    let _client_token = opaque_ptr(_client_token);
    let _privileges = opaque_ptr(_privileges);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn NtAccessCheckAndAuditAlarm(
    subsystem_name: PUNICODE_STRING,
    handle_id: PVOID,
    object_type_name: PUNICODE_STRING,
    object_name: PUNICODE_STRING,
    security_descriptor: PSECURITY_DESCRIPTOR,
    desired_access: ACCESS_MASK,
    generic_mapping: PGENERIC_MAPPING,
    object_creation: BOOLEAN,
    granted_access: PACCESS_MASK,
    access_status: PNTSTATUS,
    generate_on_close: PBOOLEAN,
) -> NTSTATUS {
    let subsystem_name = opaque_ptr(subsystem_name);
    let handle_id = opaque_ptr(handle_id);
    let object_type_name = opaque_ptr(object_type_name);
    let object_name = opaque_ptr(object_name);
    let security_descriptor = opaque_ptr(security_descriptor);
    let generic_mapping = opaque_ptr(generic_mapping);
    let access_status = opaque_ptr(access_status);
    let generate_on_close = opaque_ptr(generate_on_close);
    SepAccessCheckAndAuditAlarm(
        subsystem_name,
        handle_id,
        core::ptr::null_mut(),
        object_type_name,
        object_name,
        security_descriptor,
        core::ptr::null_mut(),
        desired_access,
        AuditEventObjectAccess,
        0,
        core::ptr::null_mut(),
        0,
        generic_mapping,
        granted_access,
        access_status,
        generate_on_close,
        0u8,
    )
}

#[no_mangle]
pub unsafe extern "system" fn NtAccessCheckByTypeAndAuditAlarm(
    subsystem_name: PUNICODE_STRING,
    handle_id: PVOID,
    object_type_name: PUNICODE_STRING,
    object_name: PUNICODE_STRING,
    security_descriptor: PSECURITY_DESCRIPTOR,
    principal_self_sid: PSID,
    desired_access: ACCESS_MASK,
    audit_type: AUDIT_EVENT_TYPE,
    flags: ULONG,
    object_type_list: POBJECT_TYPE_LIST,
    object_type_length: ULONG,
    generic_mapping: PGENERIC_MAPPING,
    object_creation: BOOLEAN,
    granted_access: PACCESS_MASK,
    access_status: PNTSTATUS,
    generate_on_close: PBOOLEAN,
) -> NTSTATUS {
    let subsystem_name = opaque_ptr(subsystem_name);
    let handle_id = opaque_ptr(handle_id);
    let object_type_name = opaque_ptr(object_type_name);
    let object_name = opaque_ptr(object_name);
    let security_descriptor = opaque_ptr(security_descriptor);
    let principal_self_sid = opaque_ptr(principal_self_sid);
    let generic_mapping = opaque_ptr(generic_mapping);
    let access_status = opaque_ptr(access_status);
    let generate_on_close = opaque_ptr(generate_on_close);
    SepAccessCheckAndAuditAlarm(
        subsystem_name,
        handle_id,
        core::ptr::null_mut(),
        object_type_name,
        object_name,
        security_descriptor,
        principal_self_sid,
        desired_access,
        audit_type,
        flags,
        object_type_list,
        object_type_length,
        generic_mapping,
        granted_access,
        access_status,
        generate_on_close,
        0u8,
    )
}

#[no_mangle]
pub unsafe extern "system" fn NtAccessCheckByTypeResultListAndAuditAlarm(
    subsystem_name: PUNICODE_STRING,
    handle_id: PVOID,
    object_type_name: PUNICODE_STRING,
    object_name: PUNICODE_STRING,
    security_descriptor: PSECURITY_DESCRIPTOR,
    principal_self_sid: PSID,
    desired_access: ACCESS_MASK,
    audit_type: AUDIT_EVENT_TYPE,
    flags: ULONG,
    object_type_list: POBJECT_TYPE_LIST,
    object_type_list_length: ULONG,
    generic_mapping: PGENERIC_MAPPING,
    object_creation: BOOLEAN,
    granted_access_list: PACCESS_MASK,
    access_status_list: PNTSTATUS,
    generate_on_close: PBOOLEAN,
) -> NTSTATUS {
    let subsystem_name = opaque_ptr(subsystem_name);
    let handle_id = opaque_ptr(handle_id);
    let object_type_name = opaque_ptr(object_type_name);
    let object_name = opaque_ptr(object_name);
    let security_descriptor = opaque_ptr(security_descriptor);
    let principal_self_sid = opaque_ptr(principal_self_sid);
    let generic_mapping = opaque_ptr(generic_mapping);
    let access_status_list = opaque_ptr(access_status_list);
    let generate_on_close = opaque_ptr(generate_on_close);
    SepAccessCheckAndAuditAlarm(
        subsystem_name,
        handle_id,
        core::ptr::null_mut(),
        object_type_name,
        object_name,
        security_descriptor,
        principal_self_sid,
        desired_access,
        audit_type,
        flags,
        object_type_list,
        object_type_list_length,
        generic_mapping,
        granted_access_list,
        access_status_list,
        generate_on_close,
        1u8,
    )
}

#[no_mangle]
pub unsafe extern "system" fn NtAccessCheckByTypeResultListAndAuditAlarmByHandle(
    subsystem_name: PUNICODE_STRING,
    handle_id: PVOID,
    client_token: HANDLE,
    object_type_name: PUNICODE_STRING,
    object_name: PUNICODE_STRING,
    security_descriptor: PSECURITY_DESCRIPTOR,
    principal_self_sid: PSID,
    desired_access: ACCESS_MASK,
    audit_type: AUDIT_EVENT_TYPE,
    flags: ULONG,
    object_type_list: POBJECT_TYPE_LIST,
    object_type_list_length: ULONG,
    generic_mapping: PGENERIC_MAPPING,
    _object_creation: BOOLEAN,
    granted_access_list: PACCESS_MASK,
    access_status_list: PNTSTATUS,
    generate_on_close: PBOOLEAN,
) -> NTSTATUS {
    let subsystem_name = opaque_ptr(subsystem_name);
    let handle_id = opaque_ptr(handle_id);
    let client_token = opaque_ptr(client_token);
    let object_type_name = opaque_ptr(object_type_name);
    let object_name = opaque_ptr(object_name);
    let security_descriptor = opaque_ptr(security_descriptor);
    let principal_self_sid = opaque_ptr(principal_self_sid);
    let generic_mapping = opaque_ptr(generic_mapping);
    let access_status_list = opaque_ptr(access_status_list);
    let generate_on_close = opaque_ptr(generate_on_close);
    SepAccessCheckAndAuditAlarm(
        subsystem_name,
        handle_id,
        &client_token as *const HANDLE as PHANDLE,
        object_type_name,
        object_name,
        security_descriptor,
        principal_self_sid,
        desired_access,
        audit_type,
        flags,
        object_type_list,
        object_type_list_length,
        generic_mapping,
        granted_access_list,
        access_status_list,
        generate_on_close,
        1u8,
    )
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS Kernel
//  * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
//  * PURPOSE:     Security auditing functions
//  * COPYRIGHT:   Copyright Eric Kohl
//  *              Copyright Timo Kreuzer <timo.kreuzer@reactos.org>
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// #define SEP_PRIVILEGE_SET_MAX_COUNT 60
//
// UNICODE_STRING SeSubsystemName = RTL_CONSTANT_STRING(L"Security");
//
// /* PRIVATE FUNCTIONS ***********************************************************/
//
// /**
//  * @unimplemented
//  * @brief
//  * Peforms a detailed security auditing with an access token.
//  *
//  * @param[in] Token
//  * A valid token object.
//  *
//  * @return
//  * To be added...
//  */
// BOOLEAN
// NTAPI
// SeDetailedAuditingWithToken(
//     _In_ PTOKEN Token)
// {
//     /* FIXME */
//     return FALSE;
// }
//
// /**
//  * @unimplemented
//  * @brief
//  * Peforms a security auditing against a process that is about to
//  * be created.
//  *
//  * @param[in] Process
//  * An object that points to a process which is in process of
//  * creation.
//  *
//  * @return
//  * Nothing.
//  */
// VOID
// NTAPI
// SeAuditProcessCreate(
//     _In_ PEPROCESS Process)
// {
//     /* FIXME */
// }
//
// /**
//  * @unimplemented
//  * @brief
//  * Peforms a security auditing against a process that is about to
//  * be terminated.
//  *
//  * @param[in] Process
//  * An object that points to a process which is in process of
//  * termination.
//  *
//  * @return
//  * Nothing.
//  */
// VOID
// NTAPI
// SeAuditProcessExit(
//     _In_ PEPROCESS Process)
// {
//     /* FIXME */
// }
//
// /**
//  * @brief
//  * Initializes a process audit name and returns it to the caller.
//  *
//  * @param[in] FileObject
//  * File object that points to a name to be queried.
//  *
//  * @param[in] DoAudit
//  * If set to TRUE, the function will perform various security
//  * auditing onto the audit name.
//  *
//  * @param[out] AuditInfo
//  * The returned audit info data.
//  *
//  * @return
//  * Returns STATUS_SUCCESS if process audit name initialization
//  * has completed successfully. STATUS_NO_MEMORY is returned if
//  * pool allocation for object name info has failed. A failure
//  * NTSTATUS code is returned otherwise.
//  */
// NTSTATUS
// NTAPI
// SeInitializeProcessAuditName(
//     _In_ PFILE_OBJECT FileObject,
//     _In_ BOOLEAN DoAudit,
//     _Out_ POBJECT_NAME_INFORMATION *AuditInfo)
// {
//     OBJECT_NAME_INFORMATION LocalNameInfo;
//     POBJECT_NAME_INFORMATION ObjectNameInfo = NULL;
//     ULONG ReturnLength = 8;
//     NTSTATUS Status;
//
//     PAGED_CODE();
//     ASSERT(AuditInfo);
//
//     /* Check if we should do auditing */
//     if (DoAudit)
//     {
//         /* FIXME: TODO */
//     }
//
//     /* Now query the name */
//     Status = ObQueryNameString(FileObject,
//                                &LocalNameInfo,
//                                sizeof(LocalNameInfo),
//                                &ReturnLength);
//     if (((Status == STATUS_BUFFER_OVERFLOW) ||
//          (Status == STATUS_BUFFER_TOO_SMALL) ||
//          (Status == STATUS_INFO_LENGTH_MISMATCH)) &&
//         (ReturnLength != sizeof(LocalNameInfo)))
//     {
//         /* Allocate required size */
//         ObjectNameInfo = ExAllocatePoolWithTag(NonPagedPool,
//                                                ReturnLength,
//                                                TAG_SEPA);
//         if (ObjectNameInfo)
//         {
//             /* Query the name again */
//             Status = ObQueryNameString(FileObject,
//                                        ObjectNameInfo,
//                                        ReturnLength,
//                                        &ReturnLength);
//         }
//     }
//
//     /* Check if we got here due to failure */
//     if ((ObjectNameInfo) &&
//         (!(NT_SUCCESS(Status)) || (ReturnLength == sizeof(LocalNameInfo))))
//     {
//         /* First, free any buffer we might've allocated */
//         ASSERT(FALSE);
//         if (ObjectNameInfo) ExFreePool(ObjectNameInfo);
//
//         /* Now allocate a temporary one */
//         ReturnLength = sizeof(OBJECT_NAME_INFORMATION);
//         ObjectNameInfo = ExAllocatePoolWithTag(NonPagedPool,
//                                                sizeof(OBJECT_NAME_INFORMATION),
//                                                TAG_SEPA);
//         if (ObjectNameInfo)
//         {
//             /* Clear it */
//             RtlZeroMemory(ObjectNameInfo, ReturnLength);
//             Status = STATUS_SUCCESS;
//         }
//     }
//
//     /* Check if memory allocation failed */
//     if (!ObjectNameInfo) Status = STATUS_NO_MEMORY;
//
//     /* Return the audit name */
//     *AuditInfo = ObjectNameInfo;
//
//     /* Return status */
//     return Status;
// }
//
// /**
//  * @brief
//  * Finds the process image name of a specific process.
//  *
//  * @param[in] Process
//  * Process object submitted by the caller, where the image name
//  * is to be located.
//  *
//  * @param[out] ProcessImageName
//  * An output Unicode string structure with the located process
//  * image name.
//  *
//  * @return
//  * Returns STATUS_SUCCESS if process image name has been located
//  * successfully. STATUS_NO_MEMORY is returned if pool allocation
//  * for the image name has failed. A failure NTSTATUS code is
//  * returned otherwise.
//  */
// NTSTATUS
// NTAPI
// SeLocateProcessImageName(
//     _In_ PEPROCESS Process,
//     _Out_ PUNICODE_STRING *ProcessImageName)
// {
//     POBJECT_NAME_INFORMATION AuditName;
//     PUNICODE_STRING ImageName;
//     PFILE_OBJECT FileObject;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     PAGED_CODE();
//
//     /* Assume failure */
//     *ProcessImageName = NULL;
//
//     /* Check if we have audit info */
//     AuditName = Process->SeAuditProcessCreationInfo.ImageFileName;
//     if (!AuditName)
//     {
//         /* Get the file object */
//         Status = PsReferenceProcessFilePointer(Process, &FileObject);
//         if (!NT_SUCCESS(Status)) return Status;
//
//         /* Initialize the audit structure */
//         Status = SeInitializeProcessAuditName(FileObject, TRUE, &AuditName);
//         if (NT_SUCCESS(Status))
//         {
//             /* Set it */
//             if (InterlockedCompareExchangePointer((PVOID*)&Process->
//                                                   SeAuditProcessCreationInfo.ImageFileName,
//                                                   AuditName,
//                                                   NULL))
//             {
//                 /* Someone beat us to it, deallocate our copy */
//                 ExFreePool(AuditName);
//             }
//         }
//
//         /* Dereference the file object */
//         ObDereferenceObject(FileObject);
//         if (!NT_SUCCESS(Status)) return Status;
//     }
//
//     /* Get audit info again, now we have it for sure */
//     AuditName = Process->SeAuditProcessCreationInfo.ImageFileName;
//
//     /* Allocate the output string */
//     ImageName = ExAllocatePoolWithTag(NonPagedPool,
//                                       AuditName->Name.MaximumLength +
//                                       sizeof(UNICODE_STRING),
//                                       TAG_SEPA);
//     if (!ImageName) return STATUS_NO_MEMORY;
//
//     /* Make a copy of it */
//     RtlCopyMemory(ImageName,
//                   &AuditName->Name,
//                   AuditName->Name.MaximumLength + sizeof(UNICODE_STRING));
//
//     /* Fix up the buffer */
//     ImageName->Buffer = (PWSTR)(ImageName + 1);
//
//     /* Return it */
//     *ProcessImageName = ImageName;
//
//     /* Return status */
//     return Status;
// }
//
// /**
//  * @brief
//  * Closes an audit alarm event of an object.
//  *
//  * @param[in] SubsystemName
//  * A Unicode string pointing to the name of the subsystem where auditing
//  * alarm event has to be closed.
//  *
//  * @param[in] HandleId
//  * A handle to an ID where such ID represents the identification of the
//  * object where audit alarm is to be closed.
//  *
//  * @param[in] Sid
//  * A SID that represents the user who attempted to close the audit
//  * alarm.
//  *
//  * @return
//  * Nothing.
//  */
// VOID
// NTAPI
// SepAdtCloseObjectAuditAlarm(
//     _In_ PUNICODE_STRING SubsystemName,
//     _In_ PVOID HandleId,
//     _In_ PSID Sid)
// {
//     UNIMPLEMENTED;
// }
//
// /**
//  * @brief
//  * Performs an audit alarm to a privileged service request.
//  * This is a worker function.
//  *
//  * @param[in] SubjectContext
//  * A security subject context used for the auditing process.
//  *
//  * @param[in] SubsystemName
//  * A Unicode string that represents the name of a subsystem that
//  * actuated the procedure of alarm auditing of a privileged
//  * service.
//  *
//  * @param[in] ServiceName
//  * A Unicode string that represents the name of a privileged
//  * service request for auditing.
//  *
//  * @param[in] Token
//  * An access token.
//  *
//  * @param[in] PrimaryToken
//  * A primary access token.
//  *
//  * @param[in] Privileges
//  * An array set of privileges used to check if the privileged
//  * service does actually have all the required set of privileges
//  * for security access.
//  *
//  * @param[in] AccessGranted
//  * When auditing is done, the function will return TRUE to the caller
//  * if access is granted, FALSE otherwise.
//  *
//  * @return
//  * Nothing.
//  */
// VOID
// NTAPI
// SepAdtPrivilegedServiceAuditAlarm(
//     _In_ PSECURITY_SUBJECT_CONTEXT SubjectContext,
//     _In_opt_ PUNICODE_STRING SubsystemName,
//     _In_opt_ PUNICODE_STRING ServiceName,
//     _In_ PTOKEN Token,
//     _In_ PTOKEN PrimaryToken,
//     _In_ PPRIVILEGE_SET Privileges,
//     _In_ BOOLEAN AccessGranted)
// {
//     DPRINT("SepAdtPrivilegedServiceAuditAlarm is unimplemented\n");
// }
//
// /**
//  * @brief
//  * Performs an audit alarm to a privileged service request.
//  *
//  * @param[in] ServiceName
//  * A Unicode string that represents the name of a privileged
//  * service request for auditing.
//  *
//  * @param[in] SubjectContext
//  * A security subject context used for the auditing process.
//  *
//  * @param[in] PrivilegeSet
//  * An array set of privileges used to check if the privileged
//  * service does actually have all the required set of privileges
//  * for security access.
//  *
//  * @param[in] AccessGranted
//  * When auditing is done, the function will return TRUE to the caller
//  * if access is granted, FALSE otherwise.
//  *
//  * @return
//  * Nothing.
//  */
// VOID
// NTAPI
// SePrivilegedServiceAuditAlarm(
//     _In_opt_ PUNICODE_STRING ServiceName,
//     _In_ PSECURITY_SUBJECT_CONTEXT SubjectContext,
//     _In_ PPRIVILEGE_SET PrivilegeSet,
//     _In_ BOOLEAN AccessGranted)
// {
//     PTOKEN EffectiveToken;
//     PSID UserSid;
//     PAGED_CODE();
//
//     /* Get the effective token */
//     if (SubjectContext->ClientToken != NULL)
//         EffectiveToken = SubjectContext->ClientToken;
//     else
//         EffectiveToken = SubjectContext->PrimaryToken;
//
//     /* Get the user SID */
//     UserSid = EffectiveToken->UserAndGroups->Sid;
//
//     /* Check if this is the local system SID */
//     if (RtlEqualSid(UserSid, SeLocalSystemSid))
//     {
//         /* Nothing to do */
//         return;
//     }
//
//     /* Check if this is the network service or local service SID */
//     if (RtlEqualSid(UserSid, SeExports->SeNetworkServiceSid) ||
//         RtlEqualSid(UserSid, SeExports->SeLocalServiceSid))
//     {
//         // FIXME: should continue for a certain set of privileges
//         return;
//     }
//
//     /* Call the worker function */
//     SepAdtPrivilegedServiceAuditAlarm(SubjectContext,
//                                       &SeSubsystemName,
//                                       ServiceName,
//                                       SubjectContext->ClientToken,
//                                       SubjectContext->PrimaryToken,
//                                       PrivilegeSet,
//                                       AccessGranted);
//
// }
//
// /**
//  * @unimplemented
//  * @brief
//  * Worker function that serves as the main heart and brain of the whole
//  * concept and implementation of auditing in the kernel.
//  *
//  * @param[in] SubsystemName
//  * A Unicode string that represents the name of a subsystem that
//  * actuates the auditing process.
//  *
//  * @param[in] HandleId
//  * A handle to an ID used to identify an object where auditing
//  * is to be done.
//  *
//  * @param[in] SubjectContext
//  * Security subject context.
//  *
//  * @param[in] ObjectTypeName
//  * A Unicode string that represents the name of an object type.
//  *
//  * @param[in] ObjectName
//  * The name of the object.
//  *
//  * @param[in] SecurityDescriptor
//  * A security descriptor with internal security information details
//  * for audit.
//  *
//  * @param[in] PrincipalSelfSid
//  * A principal self user SID.
//  *
//  * @param[in] DesiredAccess
//  * The desired access rights masks requested by the caller.
//  *
//  * @param[in] AuditType
//  * Type of audit to start. This parameter influences how an audit
//  * should be done.
//  *
//  * @param[in] HaveAuditPrivilege
//  * If set to TRUE, the security subject context has the audit privilege thus
//  * it is allowed the ability to perform the audit.
//  *
//  * @param[in] ObjectTypeList
//  * A list of object types.
//  *
//  * @param[in] ObjectTypeListLength
//  * The length size of the list.
//  *
//  * @param[in] GenericMapping
//  * The generic mapping table of access rights used whilst performing auditing
//  * sequence procedure.
//  *
//  * @param[out] GrantedAccessList
//  * This parameter is used to return to the caller a list of actual granted access
//  * rights masks that the audited object has.
//  *
//  * @param[out] AccessStatusList
//  * This parameter is used to return to the caller a list of status return codes.
//  * The function may actually return a single NTSTATUS code if the calling thread
//  * sets UseResultList parameter to FALSE.
//  *
//  * @param[out] GenerateOnClose
//  * Returns TRUE if the function has generated a list of granted access rights and
//  * status codes on termination, FALSE otherwise.
//  *
//  * @param[in] UseResultList
//  * If set to TRUE, the caller wants that the function should only return a single
//  * NTSTATUS code.
//  *
//  * @return
//  * Returns STATUS_SUCCESS if the function has completed the whole internal
//  * auditing procedure mechanism with success.
//  */
// _Must_inspect_result_
// static
// NTSTATUS
// SepAccessCheckAndAuditAlarmWorker(
//     _In_ PUNICODE_STRING SubsystemName,
//     _In_opt_ PVOID HandleId,
//     _In_ PSECURITY_SUBJECT_CONTEXT SubjectContext,
//     _In_ PUNICODE_STRING ObjectTypeName,
//     _In_ PUNICODE_STRING ObjectName,
//     _In_ PSECURITY_DESCRIPTOR SecurityDescriptor,
//     _In_opt_ PSID PrincipalSelfSid,
//     _In_ ACCESS_MASK DesiredAccess,
//     _In_ AUDIT_EVENT_TYPE AuditType,
//     _In_ BOOLEAN HaveAuditPrivilege,
//     _In_reads_opt_(ObjectTypeListLength) POBJECT_TYPE_LIST_INTERNAL ObjectTypeList,
//     _In_ ULONG ObjectTypeListLength,
//     _In_ PGENERIC_MAPPING GenericMapping,
//     _Out_writes_(ObjectTypeListLength) PACCESS_MASK GrantedAccessList,
//     _Out_writes_(ObjectTypeListLength) PNTSTATUS AccessStatusList,
//     _Out_ PBOOLEAN GenerateOnClose,
//     _In_ BOOLEAN UseResultList)
// {
//     ULONG ResultListLength, i;
//
//     /* Get the length of the result list */
//     ResultListLength = UseResultList ? ObjectTypeListLength : 1;
//
//     /// FIXME: we should do some real work here...
//     UNIMPLEMENTED;
//
//     /// HACK: we just pretend all access is granted!
//     for (i = 0; i < ResultListLength; i++)
//     {
//         GrantedAccessList[i] = DesiredAccess;
//         AccessStatusList[i] = STATUS_SUCCESS;
//     }
//
//     *GenerateOnClose = FALSE;
//
//     return STATUS_SUCCESS;
// }
//
// /**
//  * @brief
//  * Performs security auditing, if the specific object can be granted
//  * security access or not.
//  *
//  * @param[in] SubsystemName
//  * A Unicode string that represents the name of a subsystem that
//  * actuates the auditing process.
//  *
//  * @param[in] HandleId
//  * A handle to an ID used to identify an object where auditing
//  * is to be done.
//  *
//  * @param[in] SubjectContext
//  * Security subject context.
//  *
//  * @param[in] ObjectTypeName
//  * A Unicode string that represents the name of an object type.
//  *
//  * @param[in] ObjectName
//  * The name of the object.
//  *
//  * @param[in] SecurityDescriptor
//  * A security descriptor with internal security information details
//  * for audit.
//  *
//  * @param[in] PrincipalSelfSid
//  * A principal self user SID.
//  *
//  * @param[in] DesiredAccess
//  * The desired access rights masks requested by the caller.
//  *
//  * @param[in] AuditType
//  * Type of audit to start. This parameter influences how an audit
//  * should be done.
//  *
//  * @param[in] Flags
//  * Flag bitmask parameter.
//  *
//  * @param[in] HaveAuditPrivilege
//  * If set to TRUE, the security subject context has the audit privilege thus
//  * it is allowed the ability to perform the audit.
//  *
//  * @param[in] ObjectTypeList
//  * A list of object types.
//  *
//  * @param[in] ObjectTypeListLength
//  * The length size of the list.
//  *
//  * @param[in] GenericMapping
//  * The generic mapping table of access rights used whilst performing auditing
//  * sequence procedure.
//  *
//  * @param[out] GrantedAccessList
//  * This parameter is used to return to the caller a list of actual granted access
//  * rights masks that the audited object has.
//  *
//  * @param[out] AccessStatusList
//  * This parameter is used to return to the caller a list of status return codes.
//  * The function may actually return a single NTSTATUS code if the calling thread
//  * sets UseResultList parameter to FALSE.
//  *
//  * @param[out] GenerateOnClose
//  * Returns TRUE if the function has generated a list of granted access rights and
//  * status codes on termination, FALSE otherwise.
//  *
//  * @param[in] UseResultList
//  * If set to TRUE, the caller wants that the function should only return a single
//  * NTSTATUS code.
//  *
//  * @return
//  * Returns STATUS_SUCCESS if the function has completed the whole internal
//  * auditing procedure mechanism with success. STATUS_INVALID_PARAMETER is
//  * returned if one of the parameters do not satisfy the general requirements
//  * by the function. STATUS_INSUFFICIENT_RESOURCES is returned if pool memory
//  * allocation has failed. STATUS_PRIVILEGE_NOT_HELD is returned if the current
//  * security subject context does not have the required audit privilege to actually
//  * perform auditing in the first place. STATUS_INVALID_SECURITY_DESCR is returned
//  * if the security descriptor provided by the caller is not valid, that is, such
//  * descriptor doesn't belong to the main user (owner) and current group.
//  * STATUS_GENERIC_NOT_MAPPED is returned if the access rights masks aren't actually
//  * mapped. A failure NTSTATUS code is returned otherwise.
//  */
// _Must_inspect_result_
// NTSTATUS
// NTAPI
// SepAccessCheckAndAuditAlarm(
//     _In_ PUNICODE_STRING SubsystemName,
//     _In_opt_ PVOID HandleId,
//     _In_ PHANDLE ClientTokenHandle,
//     _In_ PUNICODE_STRING ObjectTypeName,
//     _In_ PUNICODE_STRING ObjectName,
//     _In_ PSECURITY_DESCRIPTOR SecurityDescriptor,
//     _In_opt_ PSID PrincipalSelfSid,
//     _In_ ACCESS_MASK DesiredAccess,
//     _In_ AUDIT_EVENT_TYPE AuditType,
//     _In_ ULONG Flags,
//     _In_reads_opt_(ObjectTypeListLength) POBJECT_TYPE_LIST ObjectTypeList,
//     _In_ ULONG ObjectTypeListLength,
//     _In_ PGENERIC_MAPPING GenericMapping,
//     _Out_writes_(ObjectTypeListLength) PACCESS_MASK GrantedAccessList,
//     _Out_writes_(ObjectTypeListLength) PNTSTATUS AccessStatusList,
//     _Out_ PBOOLEAN GenerateOnClose,
//     _In_ BOOLEAN UseResultList)
// {
//     SECURITY_SUBJECT_CONTEXT SubjectContext;
//     ULONG ResultListLength;
//     GENERIC_MAPPING LocalGenericMapping;
//     PTOKEN SubjectContextToken, ClientToken;
//     BOOLEAN AllocatedResultLists;
//     BOOLEAN HaveAuditPrivilege;
//     PSECURITY_DESCRIPTOR CapturedSecurityDescriptor;
//     UNICODE_STRING CapturedSubsystemName, CapturedObjectTypeName, CapturedObjectName;
//     ACCESS_MASK GrantedAccess, *SafeGrantedAccessList;
//     NTSTATUS AccessStatus, *SafeAccessStatusList;
//     PSID CapturedPrincipalSelfSid;
//     POBJECT_TYPE_LIST_INTERNAL CapturedObjectTypeList;
//     ULONG i;
//     BOOLEAN LocalGenerateOnClose;
//     NTSTATUS Status;
//     PAGED_CODE();
//
//     /* Only user mode is supported! */
//     ASSERT(ExGetPreviousMode() != KernelMode);
//
//     /* Start clean */
//     AllocatedResultLists = FALSE;
//     ClientToken = NULL;
//     CapturedSecurityDescriptor = NULL;
//     CapturedSubsystemName.Buffer = NULL;
//     CapturedObjectTypeName.Buffer = NULL;
//     CapturedObjectName.Buffer = NULL;
//     CapturedPrincipalSelfSid = NULL;
//     CapturedObjectTypeList = NULL;
//
//     /* Validate AuditType */
//     if ((AuditType != AuditEventObjectAccess) &&
//         (AuditType != AuditEventDirectoryServiceAccess))
//     {
//         DPRINT1("Invalid audit type: %u\n", AuditType);
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Capture the security subject context */
//     SeCaptureSubjectContext(&SubjectContext);
//
//     /* Did the caller pass a token handle? */
//     if (ClientTokenHandle == NULL)
//     {
//         /* Check if we have a token in the subject context */
//         if (SubjectContext.ClientToken == NULL)
//         {
//             Status = STATUS_NO_IMPERSONATION_TOKEN;
//             DPRINT1("No token\n");
//             goto Cleanup;
//         }
//
//         /* Check if we have a valid impersonation level */
//         if (SubjectContext.ImpersonationLevel < SecurityIdentification)
//         {
//             Status = STATUS_BAD_IMPERSONATION_LEVEL;
//             DPRINT1("Invalid impersonation level 0x%lx\n",
//                     SubjectContext.ImpersonationLevel);
//             goto Cleanup;
//         }
//     }
//
//     /* Are we using a result list? */
//     if (UseResultList)
//     {
//         /* The list length equals the object type list length */
//         ResultListLength = ObjectTypeListLength;
//         if ((ResultListLength == 0) || (ResultListLength > 0x1000))
//         {
//             Status = STATUS_INVALID_PARAMETER;
//             DPRINT1("Invalid ResultListLength: 0x%lx\n", ResultListLength);
//             goto Cleanup;
//         }
//
//         /* Allocate a safe buffer from paged pool */
//         SafeGrantedAccessList = ExAllocatePoolWithTag(PagedPool,
//                                                       2 * ResultListLength * sizeof(ULONG),
//                                                       TAG_SEPA);
//         if (SafeGrantedAccessList == NULL)
//         {
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//             DPRINT1("Failed to allocate access lists\n");
//             goto Cleanup;
//         }
//
//         SafeAccessStatusList = (PNTSTATUS)&SafeGrantedAccessList[ResultListLength];
//         AllocatedResultLists = TRUE;
//     }
//     else
//     {
//         /* List length is 1 */
//         ResultListLength = 1;
//         SafeGrantedAccessList = &GrantedAccess;
//         SafeAccessStatusList = &AccessStatus;
//     }
//
//     _SEH2_TRY
//     {
//         /* Probe output buffers */
//         ProbeForWrite(AccessStatusList,
//                       ResultListLength * sizeof(*AccessStatusList),
//                       sizeof(*AccessStatusList));
//         ProbeForWrite(GrantedAccessList,
//                       ResultListLength * sizeof(*GrantedAccessList),
//                       sizeof(*GrantedAccessList));
//
//         /* Probe generic mapping and make a local copy */
//         ProbeForRead(GenericMapping, sizeof(*GenericMapping), sizeof(ULONG));
//         LocalGenericMapping = * GenericMapping;
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = _SEH2_GetExceptionCode();
//         DPRINT1("Exception while probing parameters: 0x%lx\n", Status);
//         _SEH2_YIELD(goto Cleanup);
//     }
//     _SEH2_END;
//
//     /* Do we have a client token? */
//     if (ClientTokenHandle != NULL)
//     {
//         /* Reference the client token */
//         Status = ObReferenceObjectByHandle(*ClientTokenHandle,
//                                            TOKEN_QUERY,
//                                            SeTokenObjectType as *mut _OBJECT_TYPE,
//                                            UserMode,
//                                            (PVOID*)&ClientToken,
//                                            NULL);
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT1("Failed to reference token handle %p: %lx\n",
//                     *ClientTokenHandle, Status);
//             goto Cleanup;
//         }
//
//         SubjectContextToken = SubjectContext.ClientToken;
//         SubjectContext.ClientToken = ClientToken;
//     }
//
//     /* Check for audit privilege */
//     HaveAuditPrivilege = SeCheckAuditPrivilege(&SubjectContext, UserMode);
//     if (!HaveAuditPrivilege && !(Flags & AUDIT_ALLOW_NO_PRIVILEGE))
//     {
//         DPRINT1("Caller does not have SeAuditPrivilege\n");
//         Status = STATUS_PRIVILEGE_NOT_HELD;
//         goto Cleanup;
//     }
//
//     /* Generic access must already be mapped to non-generic access types! */
//     if (DesiredAccess & (GENERIC_READ | GENERIC_WRITE | GENERIC_EXECUTE | GENERIC_ALL))
//     {
//         DPRINT1("Generic access rights requested: 0x%lx\n", DesiredAccess);
//         Status = STATUS_GENERIC_NOT_MAPPED;
//         goto Cleanup;
//     }
//
//     /* Capture the security descriptor */
//     Status = SeCaptureSecurityDescriptor(SecurityDescriptor,
//                                          UserMode,
//                                          PagedPool,
//                                          FALSE,
//                                          &CapturedSecurityDescriptor);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to capture security descriptor!\n");
//         goto Cleanup;
//     }
//
//     /* Validate the Security descriptor */
//     if ((SepGetOwnerFromDescriptor(CapturedSecurityDescriptor) == NULL) ||
//         (SepGetGroupFromDescriptor(CapturedSecurityDescriptor) == NULL))
//     {
//         Status = STATUS_INVALID_SECURITY_DESCR;
//         DPRINT1("Invalid security descriptor\n");
//         goto Cleanup;
//     }
//
//     /* Probe and capture the subsystem name */
//     Status = ProbeAndCaptureUnicodeString(&CapturedSubsystemName,
//                                           UserMode,
//                                           SubsystemName);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to capture subsystem name!\n");
//         goto Cleanup;
//     }
//
//     /* Probe and capture the object type name */
//     Status = ProbeAndCaptureUnicodeString(&CapturedObjectTypeName,
//                                           UserMode,
//                                           ObjectTypeName);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to capture object type name!\n");
//         goto Cleanup;
//     }
//
//     /* Probe and capture the object name */
//     Status = ProbeAndCaptureUnicodeString(&CapturedObjectName,
//                                           UserMode,
//                                           ObjectName);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to capture object name!\n");
//         goto Cleanup;
//     }
//
//     /* Check if we have a PrincipalSelfSid */
//     if (PrincipalSelfSid != NULL)
//     {
//         /* Capture it */
//         Status = SepCaptureSid(PrincipalSelfSid,
//                                UserMode,
//                                PagedPool,
//                                FALSE,
//                                &CapturedPrincipalSelfSid);
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT1("Failed to capture PrincipalSelfSid!\n");
//             goto Cleanup;
//         }
//     }
//
//     /* Capture the object type list */
//     Status = SeCaptureObjectTypeList(ObjectTypeList,
//                                      ObjectTypeListLength,
//                                      UserMode,
//                                      &CapturedObjectTypeList);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to capture object type list!\n");
//         goto Cleanup;
//     }
//
//     /* Call the worker routine with the captured buffers */
//     Status = SepAccessCheckAndAuditAlarmWorker(&CapturedSubsystemName,
//                                                HandleId,
//                                                &SubjectContext,
//                                                &CapturedObjectTypeName,
//                                                &CapturedObjectName,
//                                                CapturedSecurityDescriptor,
//                                                CapturedPrincipalSelfSid,
//                                                DesiredAccess,
//                                                AuditType,
//                                                HaveAuditPrivilege,
//                                                CapturedObjectTypeList,
//                                                ObjectTypeListLength,
//                                                &LocalGenericMapping,
//                                                SafeGrantedAccessList,
//                                                SafeAccessStatusList,
//                                                &LocalGenerateOnClose,
//                                                UseResultList);
//     if (!NT_SUCCESS(Status))
//         goto Cleanup;
//
//     /* Enter SEH to copy the data back to user mode */
//     _SEH2_TRY
//     {
//         /* Loop all result entries (only 1 when no list was requested) */
//         ASSERT(UseResultList || (ResultListLength == 1));
//         for (i = 0; i < ResultListLength; i++)
//         {
//             AccessStatusList[i] = SafeAccessStatusList[i];
//             GrantedAccessList[i] = SafeGrantedAccessList[i];
//         }
//
//         *GenerateOnClose = LocalGenerateOnClose;
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = _SEH2_GetExceptionCode();
//         DPRINT1("Exception while copying back data: 0x%lx\n", Status);
//     }
//     _SEH2_END;
//
// Cleanup:
//
//     if (CapturedObjectTypeList != NULL)
//         SeReleaseObjectTypeList(CapturedObjectTypeList, UserMode);
//
//     if (CapturedPrincipalSelfSid != NULL)
//         SepReleaseSid(CapturedPrincipalSelfSid, UserMode, FALSE);
//
//     if (CapturedObjectName.Buffer != NULL)
//         ReleaseCapturedUnicodeString(&CapturedObjectName, UserMode);
//
//     if (CapturedObjectTypeName.Buffer != NULL)
//         ReleaseCapturedUnicodeString(&CapturedObjectTypeName, UserMode);
//
//     if (CapturedSubsystemName.Buffer != NULL)
//         ReleaseCapturedUnicodeString(&CapturedSubsystemName, UserMode);
//
//     if (CapturedSecurityDescriptor != NULL)
//         SeReleaseSecurityDescriptor(CapturedSecurityDescriptor, UserMode, FALSE);
//
//     if (ClientToken != NULL)
//     {
//         ObDereferenceObject(ClientToken);
//         SubjectContext.ClientToken = SubjectContextToken;
//     }
//
//     if (AllocatedResultLists)
//         ExFreePoolWithTag(SafeGrantedAccessList, TAG_SEPA);
//
//     /* Release the security subject context */
//     SeReleaseSubjectContext(&SubjectContext);
//
//     return Status;
// }
//
//
// /* PUBLIC FUNCTIONS ***********************************************************/
//
// /**
//  * @unimplemented
//  * @brief
//  * Performs an audit against a hard link creation.
//  *
//  * @param[in] FileName
//  * A Unicode string that points to the name of the file.
//  *
//  * @param[in] LinkName
//  * A Unicode string that points to a link.
//  *
//  * @param[out] bSuccess
//  * If TRUE, the function has successfully audited
//  * the hard link and security access can be granted,
//  * FALSE otherwise.
//  *
//  * @return
//  * Nothing.
//  */
// VOID
// NTAPI
// SeAuditHardLinkCreation(
//     _In_ PUNICODE_STRING FileName,
//     _In_ PUNICODE_STRING LinkName,
//     _In_ BOOLEAN bSuccess)
// {
//     UNIMPLEMENTED;
// }
//
// /**
//  * @unimplemented
//  * @brief
//  * Determines whether auditing against file events is being
//  * done or not.
//  *
//  * @param[in] AccessGranted
//  * If set to TRUE, the access attempt is deemed as successful
//  * otherwise set it to FALSE.
//  *
//  * @param[in] SecurityDescriptor
//  * A security descriptor.
//  *
//  * @return
//  * Returns TRUE if auditing is being currently done, FALSE otherwise.
//  */
// BOOLEAN
// NTAPI
// SeAuditingFileEvents(
//     _In_ BOOLEAN AccessGranted,
//     _In_ PSECURITY_DESCRIPTOR SecurityDescriptor)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// /**
//  * @unimplemented
//  * @brief
//  * Determines whether auditing against file events with subject context
//  * is being done or not.
//  *
//  * @param[in] AccessGranted
//  * If set to TRUE, the access attempt is deemed as successful
//  * otherwise set it to FALSE.
//  *
//  * @param[in] SecurityDescriptor
//  * A security descriptor.
//  *
//  * @param[in] SubjectSecurityContext
//  * If specified, the function will check if security auditing is currently
//  * being done with this context.
//  *
//  * @return
//  * Returns TRUE if auditing is being currently done, FALSE otherwise.
//  */
// BOOLEAN
// NTAPI
// SeAuditingFileEventsWithContext(
//     _In_ BOOLEAN AccessGranted,
//     _In_ PSECURITY_DESCRIPTOR SecurityDescriptor,
//     _In_opt_ PSECURITY_SUBJECT_CONTEXT SubjectSecurityContext)
// {
//     UNIMPLEMENTED_ONCE;
//     return FALSE;
// }
//
// /**
//  * @unimplemented
//  * @brief
//  * Determines whether auditing against hard links events is being
//  * done or not.
//  *
//  * @param[in] AccessGranted
//  * If set to TRUE, the access attempt is deemed as successful
//  * otherwise set it to FALSE.
//  *
//  * @param[in] SecurityDescriptor
//  * A security descriptor.
//  *
//  * @return
//  * Returns TRUE if auditing is being currently done, FALSE otherwise.
//  */
// BOOLEAN
// NTAPI
// SeAuditingHardLinkEvents(
//     _In_ BOOLEAN AccessGranted,
//     _In_ PSECURITY_DESCRIPTOR SecurityDescriptor)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// /**
//  * @unimplemented
//  * @brief
//  * Determines whether auditing against hard links events with subject context
//  * is being done or not.
//  *
//  * @param[in] AccessGranted
//  * If set to TRUE, the access attempt is deemed as successful
//  * otherwise set it to FALSE.
//  *
//  * @param[in] SecurityDescriptor
//  * A security descriptor.
//  *
//  * @param[in] SubjectSecurityContext
//  * If specified, the function will check if security auditing is currently
//  * being done with this context.
//  *
//  * @return
//  * Returns TRUE if auditing is being currently done, FALSE otherwise.
//  */
// BOOLEAN
// NTAPI
// SeAuditingHardLinkEventsWithContext(
//     _In_ BOOLEAN AccessGranted,
//     _In_ PSECURITY_DESCRIPTOR SecurityDescriptor,
//     _In_opt_ PSECURITY_SUBJECT_CONTEXT SubjectSecurityContext)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// /**
//  * @unimplemented
//  * @brief
//  * Determines whether auditing against files or global events with
//  * subject context is being done or not.
//  *
//  * @param[in] AccessGranted
//  * If set to TRUE, the access attempt is deemed as successful
//  * otherwise set it to FALSE.
//  *
//  * @param[in] SecurityDescriptor
//  * A security descriptor.
//  *
//  * @param[in] SubjectSecurityContext
//  * If specified, the function will check if security auditing is currently
//  * being done with this context.
//  *
//  * @return
//  * Returns TRUE if auditing is being currently done, FALSE otherwise.
//  */
// BOOLEAN
// NTAPI
// SeAuditingFileOrGlobalEvents(
//     _In_ BOOLEAN AccessGranted,
//     _In_ PSECURITY_DESCRIPTOR SecurityDescriptor,
//     _In_ PSECURITY_SUBJECT_CONTEXT SubjectSecurityContext)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// /**
//  * @unimplemented
//  * @brief
//  * Closes an alarm audit of an object.
//  *
//  * @param[in] Object
//  * An arbitrary pointer data that points to the object.
//  *
//  * @param[in] Handle
//  * A handle of the said object.
//  *
//  * @param[in] PerformAction
//  * Set this to TRUE to perform any auxiliary action, otherwise
//  * set to FALSE.
//  *
//  * @return
//  * Nothing.
//  */
// VOID
// NTAPI
// SeCloseObjectAuditAlarm(
//     _In_ PVOID Object,
//     _In_ HANDLE Handle,
//     _In_ BOOLEAN PerformAction)
// {
//     UNIMPLEMENTED;
// }
//
// /**
//  * @unimplemented
//  * @brief
//  * Deletes an alarm audit of an object.
//  *
//  * @param[in] Object
//  * An arbitrary pointer data that points to the object.
//  *
//  * @param[in] Handle
//  * A handle of the said object.
//  *
//  * @return
//  * Nothing.
//  */
// VOID NTAPI
// SeDeleteObjectAuditAlarm(
//     _In_ PVOID Object,
//     _In_ HANDLE Handle)
// {
//     UNIMPLEMENTED;
// }
//
// /**
//  * @unimplemented
//  * @brief
//  * Creates an audit with alarm notification of an object
//  * that is being opened.
//  *
//  * @param[in] ObjectTypeName
//  * A Unicode string that points to the object type name.
//  *
//  * @param[in] Object
//  * If specified, the function will use this parameter to
//  * directly open the object.
//  *
//  * @param[in] AbsoluteObjectName
//  * If specified, the function will use this parameter to
//  * directly open the object through the absolute name
//  * of the object.
//  *
//  * @param[in] SecurityDescriptor
//  * A security descriptor.
//  *
//  * @param[in] AccessState
//  * An access state right mask when opening the object.
//  *
//  * @param[in] ObjectCreated
//  * Set this to TRUE if the object has been fully created,
//  * FALSE otherwise.
//  *
//  * @param[in] AccessGranted
//  * Set this to TRUE if access was deemed as granted.
//  *
//  * @param[in] AccessMode
//  * Processor level access mode.
//  *
//  * @param[out] GenerateOnClose
//  * A boolean flag returned to the caller once audit generation procedure
//  * finishes.
//  *
//  * @return
//  * Nothing.
//  */
// VOID
// NTAPI
// SeOpenObjectAuditAlarm(
//     _In_ PUNICODE_STRING ObjectTypeName,
//     _In_opt_ PVOID Object,
//     _In_opt_ PUNICODE_STRING AbsoluteObjectName,
//     _In_ PSECURITY_DESCRIPTOR SecurityDescriptor,
//     _In_ PACCESS_STATE AccessState,
//     _In_ BOOLEAN ObjectCreated,
//     _In_ BOOLEAN AccessGranted,
//     _In_ KPROCESSOR_MODE AccessMode,
//     _Out_ PBOOLEAN GenerateOnClose)
// {
//     PAGED_CODE();
//
//     /* Audits aren't done on kernel-mode access */
//     if (AccessMode == KernelMode) return;
//
//     /* Otherwise, unimplemented! */
//     //UNIMPLEMENTED;
//     return;
// }
//
// /**
//  * @unimplemented
//  * @brief
//  * Creates an audit with alarm notification of an object
//  * that is being opened for deletion.
//  *
//  * @param[in] ObjectTypeName
//  * A Unicode string that points to the object type name.
//  *
//  * @param[in] Object
//  * If specified, the function will use this parameter to
//  * directly open the object.
//  *
//  * @param[in] AbsoluteObjectName
//  * If specified, the function will use this parameter to
//  * directly open the object through the absolute name
//  * of the object.
//  *
//  * @param[in] SecurityDescriptor
//  * A security descriptor.
//  *
//  * @param[in] AccessState
//  * An access state right mask when opening the object.
//  *
//  * @param[in] ObjectCreated
//  * Set this to TRUE if the object has been fully created,
//  * FALSE otherwise.
//  *
//  * @param[in] AccessGranted
//  * Set this to TRUE if access was deemed as granted.
//  *
//  * @param[in] AccessMode
//  * Processor level access mode.
//  *
//  * @param[out] GenerateOnClose
//  * A boolean flag returned to the caller once audit generation procedure
//  * finishes.
//  *
//  * @return
//  * Nothing.
//  */
// VOID NTAPI
// SeOpenObjectForDeleteAuditAlarm(
//     _In_ PUNICODE_STRING ObjectTypeName,
//     _In_opt_ PVOID Object,
//     _In_opt_ PUNICODE_STRING AbsoluteObjectName,
//     _In_ PSECURITY_DESCRIPTOR SecurityDescriptor,
//     _In_ PACCESS_STATE AccessState,
//     _In_ BOOLEAN ObjectCreated,
//     _In_ BOOLEAN AccessGranted,
//     _In_ KPROCESSOR_MODE AccessMode,
//     _Out_ PBOOLEAN GenerateOnClose)
// {
//     UNIMPLEMENTED;
// }
//
// /**
//  * @unimplemented
//  * @brief
//  * Raises an audit with alarm notification message
//  * when an object tries to acquire this privilege.
//  *
//  * @param[in] Handle
//  * A handle to an object.
//  *
//  * @param[in] SubjectContext
//  * The security subject context for auditing.
//  *
//  * @param[in] DesiredAccess
//  * The desired right access masks requested by the caller.
//  *
//  * @param[in] Privileges
//  * An array set of privileges for auditing.
//  *
//  * @param[out] AccessGranted
//  * When the auditing procedure routine ends, it returns TRUE to the
//  * caller if the object has the required privileges for access,
//  * FALSE otherwise.
//  *
//  * @param[in] CurrentMode
//  * Processor level access mode.
//  *
//  * @return
//  * Nothing.
//  */
// VOID
// NTAPI
// SePrivilegeObjectAuditAlarm(
//     _In_ HANDLE Handle,
//     _In_ PSECURITY_SUBJECT_CONTEXT SubjectContext,
//     _In_ ACCESS_MASK DesiredAccess,
//     _In_ PPRIVILEGE_SET Privileges,
//     _In_ BOOLEAN AccessGranted,
//     _In_ KPROCESSOR_MODE CurrentMode)
// {
//     UNIMPLEMENTED;
// }
//
// /* SYSTEM CALLS ***************************************************************/
//
// /**
//  * @brief
//  * Raises an alarm audit message when an object is about
//  * to be closed.
//  *
//  * @param[in] SubsystemName
//  * A Unicode string that points to the name of the subsystem.
//  *
//  * @param[in] HandleId
//  * A handle of an ID used for identification instance for auditing.
//  *
//  * @param[in] GenerateOnClose
//  * A boolean value previously created by the "open" equivalent of this
//  * function. If the caller explicitly sets this to FALSE, the function
//  * assumes that the object is not opened.
//  *
//  * @return
//  * Returns STATUS_SUCCESS if all the operations have completed successfully.
//  * STATUS_PRIVILEGE_NOT_HELD is returned if the security subject context
//  * does not have the audit privilege to actually begin auditing procedures
//  * in the first place.
//  */
// NTSTATUS
// NTAPI
// NtCloseObjectAuditAlarm(
//     _In_ PUNICODE_STRING SubsystemName,
//     _In_ PVOID HandleId,
//     _In_ BOOLEAN GenerateOnClose)
// {
//     SECURITY_SUBJECT_CONTEXT SubjectContext;
//     UNICODE_STRING CapturedSubsystemName;
//     KPROCESSOR_MODE PreviousMode;
//     BOOLEAN UseImpersonationToken;
//     PETHREAD CurrentThread;
//     BOOLEAN CopyOnOpen, EffectiveOnly;
//     SECURITY_IMPERSONATION_LEVEL ImpersonationLevel;
//     NTSTATUS Status;
//     PTOKEN Token;
//     PAGED_CODE();
//
//     /* Get the previous mode (only user mode is supported!) */
//     PreviousMode = ExGetPreviousMode();
//     ASSERT(PreviousMode != KernelMode);
//
//     /* Do we even need to do anything? */
//     if (!GenerateOnClose)
//     {
//         /* Nothing to do, return success */
//         return STATUS_SUCCESS;
//     }
//
//     /* Capture the security subject context */
//     SeCaptureSubjectContext(&SubjectContext);
//
//     /* Check for audit privilege */
//     if (!SeCheckAuditPrivilege(&SubjectContext, PreviousMode))
//     {
//         DPRINT1("Caller does not have SeAuditPrivilege\n");
//         Status = STATUS_PRIVILEGE_NOT_HELD;
//         goto Cleanup;
//     }
//
//     /* Probe and capture the subsystem name */
//     Status = ProbeAndCaptureUnicodeString(&CapturedSubsystemName,
//                                           PreviousMode,
//                                           SubsystemName);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to capture subsystem name!\n");
//         goto Cleanup;
//     }
//
//     /* Get the current thread and check if it's impersonating */
//     CurrentThread = PsGetCurrentThread();
//     if (PsIsThreadImpersonating(CurrentThread))
//     {
//         /* Get the impersonation token */
//         Token = PsReferenceImpersonationToken(CurrentThread,
//                                               &CopyOnOpen,
//                                               &EffectiveOnly,
//                                               &ImpersonationLevel);
//         UseImpersonationToken = TRUE;
//     }
//     else
//     {
//         /* Get the primary token */
//         Token = PsReferencePrimaryToken(PsGetCurrentProcess());
//         UseImpersonationToken = FALSE;
//     }
//
//     /* Call the internal function */
//     SepAdtCloseObjectAuditAlarm(&CapturedSubsystemName,
//                                 HandleId,
//                                 Token->UserAndGroups->Sid);
//
//     /* Release the captured subsystem name */
//     ReleaseCapturedUnicodeString(&CapturedSubsystemName, PreviousMode);
//
//     /* Check what token we used */
//     if (UseImpersonationToken)
//     {
//         /* Release impersonation token */
//         PsDereferenceImpersonationToken(Token);
//     }
//     else
//     {
//         /* Release primary token */
//         PsDereferencePrimaryToken(Token);
//     }
//
//     Status = STATUS_SUCCESS;
//
// Cleanup:
//
//     /* Release the security subject context */
//     SeReleaseSubjectContext(&SubjectContext);
//
//     return Status;
// }
//
// /**
//  * @unimplemented
//  * @brief
//  * Raises an alarm audit message when an object is about
//  * to be deleted.
//  *
//  * @param[in] SubsystemName
//  * A Unicode string that points to the name of the subsystem.
//  *
//  * @param[in] HandleId
//  * A handle of an ID used for identification instance for auditing.
//  *
//  * @param[in] GenerateOnClose
//  * A boolean value previously created by the "open" equivalent of this
//  * function. If the caller explicitly sets this to FALSE, the function
//  * assumes that the object is not opened.
//  *
//  * @return
//  * To be added...
//  */
// NTSTATUS NTAPI
// NtDeleteObjectAuditAlarm(
//     _In_ PUNICODE_STRING SubsystemName,
//     _In_ PVOID HandleId,
//     _In_ BOOLEAN GenerateOnClose)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// /**
//  * @unimplemented
//  * @brief
//  * Raises an alarm audit message when an object is about
//  * to be opened.
//  *
//  * @param[in] SubjectContext
//  * A security subject context for auditing.
//  *
//  * @param[in] SubsystemName
//  * A Unicode string that points to a name of the subsystem.
//  *
//  * @param[in] HandleId
//  * A handle to an ID used for identification instance for auditing.
//  *
//  * @param[in] ObjectTypeName
//  * A Unicode string that points to an object type name.
//  *
//  * @param[in] ObjectName
//  * The name of the object.
//  *
//  * @param[in] SecurityDescriptor
//  * A security descriptor.
//  *
//  * @param[in] ClientToken
//  * A client access token, representing the client we want to impersonate.
//  *
//  * @param[in] DesiredAccess
//  * The desired access rights masks requested by the caller.
//  *
//  * @param[in] GrantedAccess
//  * The granted access mask rights.
//  *
//  * @param[in] Privileges
//  * If specified, the function will use this set of privileges to audit.
//  *
//  * @param[in] ObjectCreation
//  * Set this to TRUE if the object has just been created.
//  *
//  * @param[in] AccessGranted
//  * Set this to TRUE if the access attempt was deemed as granted.
//  *
//  * @param[out] GenerateOnClose
//  * A boolean flag returned to the caller once audit generation procedure
//  * finishes.
//  *
//  * @return
//  * Nothing.
//  */
// VOID
// NTAPI
// SepOpenObjectAuditAlarm(
//     _In_ PSECURITY_SUBJECT_CONTEXT SubjectContext,
//     _In_ PUNICODE_STRING SubsystemName,
//     _In_opt_ PVOID HandleId,
//     _In_ PUNICODE_STRING ObjectTypeName,
//     _In_ PUNICODE_STRING ObjectName,
//     _In_opt_ PSECURITY_DESCRIPTOR SecurityDescriptor,
//     _In_ PTOKEN ClientToken,
//     _In_ ACCESS_MASK DesiredAccess,
//     _In_ ACCESS_MASK GrantedAccess,
//     _In_opt_ PPRIVILEGE_SET Privileges,
//     _In_ BOOLEAN ObjectCreation,
//     _In_ BOOLEAN AccessGranted,
//     _Out_ PBOOLEAN GenerateOnClose)
// {
//     DBG_UNREFERENCED_PARAMETER(SubjectContext);
//     DBG_UNREFERENCED_PARAMETER(SubsystemName);
//     DBG_UNREFERENCED_PARAMETER(HandleId);
//     DBG_UNREFERENCED_PARAMETER(ObjectTypeName);
//     DBG_UNREFERENCED_PARAMETER(ObjectName);
//     DBG_UNREFERENCED_PARAMETER(SecurityDescriptor);
//     DBG_UNREFERENCED_PARAMETER(ClientToken);
//     DBG_UNREFERENCED_PARAMETER(DesiredAccess);
//     DBG_UNREFERENCED_PARAMETER(GrantedAccess);
//     DBG_UNREFERENCED_PARAMETER(Privileges);
//     DBG_UNREFERENCED_PARAMETER(ObjectCreation);
//     DBG_UNREFERENCED_PARAMETER(AccessGranted);
//     UNIMPLEMENTED;
//     *GenerateOnClose = FALSE;
// }
//
// /**
//  * @brief
//  * Raises an alarm audit message when an object is about
//  * to be opened.
//  *
//  * @param[in] SubsystemName
//  * A Unicode string that points to a name of the subsystem.
//  *
//  * @param[in] HandleId
//  * A handle to an ID used for identification instance for auditing.
//  *
//  * @param[in] ObjectTypeName
//  * A Unicode string that points to an object type name.
//  *
//  * @param[in] ObjectName
//  * The name of the object.
//  *
//  * @param[in] SecurityDescriptor
//  * A security descriptor.
//  *
//  * @param[in] ClientTokenHandle
//  * A handle to a client access token.
//  *
//  * @param[in] DesiredAccess
//  * The desired access rights masks requested by the caller.
//  *
//  * @param[in] GrantedAccess
//  * The granted access mask rights.
//  *
//  * @param[in] PrivilegeSet
//  * If specified, the function will use this set of privileges to audit.
//  *
//  * @param[in] ObjectCreation
//  * Set this to TRUE if the object has just been created.
//  *
//  * @param[in] AccessGranted
//  * Set this to TRUE if the access attempt was deemed as granted.
//  *
//  * @param[out] GenerateOnClose
//  * A boolean flag returned to the caller once audit generation procedure
//  * finishes.
//  *
//  * @return
//  * Returns STATUS_SUCCESS if all the operations have been completed successfully.
//  * STATUS_PRIVILEGE_NOT_HELD is returned if the given subject context does not
//  * hold the required audit privilege to actually begin auditing in the first place.
//  * STATUS_BAD_IMPERSONATION_LEVEL is returned if the security impersonation level
//  * of the client token is not on par with the impersonation level that alllows
//  * impersonation. STATUS_INVALID_PARAMETER is returned if the caller has
//  * submitted a bogus set of privileges as such array set exceeds the maximum
//  * count of privileges that the kernel can accept. A failure NTSTATUS code
//  * is returned otherwise.
//  */
// __kernel_entry
// NTSTATUS
// NTAPI
// NtOpenObjectAuditAlarm(
//     _In_ PUNICODE_STRING SubsystemName,
//     _In_opt_ PVOID HandleId,
//     _In_ PUNICODE_STRING ObjectTypeName,
//     _In_ PUNICODE_STRING ObjectName,
//     _In_opt_ PSECURITY_DESCRIPTOR SecurityDescriptor,
//     _In_ HANDLE ClientTokenHandle,
//     _In_ ACCESS_MASK DesiredAccess,
//     _In_ ACCESS_MASK GrantedAccess,
//     _In_opt_ PPRIVILEGE_SET PrivilegeSet,
//     _In_ BOOLEAN ObjectCreation,
//     _In_ BOOLEAN AccessGranted,
//     _Out_ PBOOLEAN GenerateOnClose)
// {
//     PTOKEN ClientToken;
//     PSECURITY_DESCRIPTOR CapturedSecurityDescriptor;
//     UNICODE_STRING CapturedSubsystemName, CapturedObjectTypeName, CapturedObjectName;
//     ULONG PrivilegeCount, PrivilegeSetSize;
//     volatile PPRIVILEGE_SET CapturedPrivilegeSet;
//     BOOLEAN LocalGenerateOnClose;
//     PVOID CapturedHandleId;
//     SECURITY_SUBJECT_CONTEXT SubjectContext;
//     NTSTATUS Status;
//     PAGED_CODE();
//
//     /* Only user mode is supported! */
//     ASSERT(ExGetPreviousMode() != KernelMode);
//
//     /* Start clean */
//     ClientToken = NULL;
//     CapturedSecurityDescriptor = NULL;
//     CapturedPrivilegeSet = NULL;
//     CapturedSubsystemName.Buffer = NULL;
//     CapturedObjectTypeName.Buffer = NULL;
//     CapturedObjectName.Buffer = NULL;
//
//     /* Reference the client token */
//     Status = ObReferenceObjectByHandle(ClientTokenHandle,
//                                        TOKEN_QUERY,
//                                        SeTokenObjectType as *mut _OBJECT_TYPE,
//                                        UserMode,
//                                        (PVOID*)&ClientToken,
//                                        NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to reference token handle %p: %lx\n",
//                 ClientTokenHandle, Status);
//         return Status;
//     }
//
//     /* Capture the security subject context */
//     SeCaptureSubjectContext(&SubjectContext);
//
//     /* Validate the token's impersonation level */
//     if ((ClientToken->TokenType == TokenImpersonation) &&
//         (ClientToken->ImpersonationLevel < SecurityIdentification))
//     {
//         DPRINT1("Invalid impersonation level (%u)\n", ClientToken->ImpersonationLevel);
//         Status = STATUS_BAD_IMPERSONATION_LEVEL;
//         goto Cleanup;
//     }
//
//     /* Check for audit privilege */
//     if (!SeCheckAuditPrivilege(&SubjectContext, UserMode))
//     {
//         DPRINT1("Caller does not have SeAuditPrivilege\n");
//         Status = STATUS_PRIVILEGE_NOT_HELD;
//         goto Cleanup;
//     }
//
//     /* Check for NULL SecurityDescriptor */
//     if (SecurityDescriptor == NULL)
//     {
//         /* Nothing to do */
//         Status = STATUS_SUCCESS;
//         goto Cleanup;
//     }
//
//     /* Capture the security descriptor */
//     Status = SeCaptureSecurityDescriptor(SecurityDescriptor,
//                                          UserMode,
//                                          PagedPool,
//                                          FALSE,
//                                          &CapturedSecurityDescriptor);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to capture security descriptor!\n");
//         goto Cleanup;
//     }
//
//     _SEH2_TRY
//     {
//         /* Check if we have a privilege set */
//         if (PrivilegeSet != NULL)
//         {
//             /* Probe the basic privilege set structure */
//             ProbeForRead(PrivilegeSet, sizeof(PRIVILEGE_SET), sizeof(ULONG));
//
//             /* Validate privilege count */
//             PrivilegeCount = PrivilegeSet->PrivilegeCount;
//             if (PrivilegeCount > SEP_PRIVILEGE_SET_MAX_COUNT)
//             {
//                 Status = STATUS_INVALID_PARAMETER;
//                 _SEH2_YIELD(goto Cleanup);
//             }
//
//             /* Calculate the size of the PrivilegeSet structure */
//             PrivilegeSetSize = FIELD_OFFSET(PRIVILEGE_SET, Privilege[PrivilegeCount]);
//
//             /* Probe the whole structure */
//             ProbeForRead(PrivilegeSet, PrivilegeSetSize, sizeof(ULONG));
//
//             /* Allocate a temp buffer */
//             CapturedPrivilegeSet = ExAllocatePoolWithTag(PagedPool,
//                                                          PrivilegeSetSize,
//                                                          TAG_PRIVILEGE_SET);
//             if (CapturedPrivilegeSet == NULL)
//             {
//                 DPRINT1("Failed to allocate %u bytes\n", PrivilegeSetSize);
//                 Status = STATUS_INSUFFICIENT_RESOURCES;
//                 _SEH2_YIELD(goto Cleanup);
//             }
//
//             /* Copy the privileges */
//             RtlCopyMemory(CapturedPrivilegeSet, PrivilegeSet, PrivilegeSetSize);
//         }
//
//         if (HandleId != NULL)
//         {
//             ProbeForRead(HandleId, sizeof(PVOID), sizeof(PVOID));
//             CapturedHandleId = *(PVOID*)HandleId;
//         }
//
//         ProbeForWrite(GenerateOnClose, sizeof(BOOLEAN), sizeof(BOOLEAN));
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = _SEH2_GetExceptionCode();
//         DPRINT1("Exception while probing parameters: 0x%lx\n", Status);
//         _SEH2_YIELD(goto Cleanup);
//     }
//     _SEH2_END;
//
//     /* Probe and capture the subsystem name */
//     Status = ProbeAndCaptureUnicodeString(&CapturedSubsystemName,
//                                           UserMode,
//                                           SubsystemName);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to capture subsystem name!\n");
//         goto Cleanup;
//     }
//
//     /* Probe and capture the object type name */
//     Status = ProbeAndCaptureUnicodeString(&CapturedObjectTypeName,
//                                           UserMode,
//                                           ObjectTypeName);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to capture object type name!\n");
//         goto Cleanup;
//     }
//
//     /* Probe and capture the object name */
//     Status = ProbeAndCaptureUnicodeString(&CapturedObjectName,
//                                           UserMode,
//                                           ObjectName);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to capture object name!\n");
//         goto Cleanup;
//     }
//
//     /* Call the internal function */
//     SepOpenObjectAuditAlarm(&SubjectContext,
//                             &CapturedSubsystemName,
//                             CapturedHandleId,
//                             &CapturedObjectTypeName,
//                             &CapturedObjectName,
//                             CapturedSecurityDescriptor,
//                             ClientToken,
//                             DesiredAccess,
//                             GrantedAccess,
//                             CapturedPrivilegeSet,
//                             ObjectCreation,
//                             AccessGranted,
//                             &LocalGenerateOnClose);
//
//     Status = STATUS_SUCCESS;
//
//     /* Enter SEH to copy the data back to user mode */
//     _SEH2_TRY
//     {
//         *GenerateOnClose = LocalGenerateOnClose;
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = _SEH2_GetExceptionCode();
//         DPRINT1("Exception while copying back data: 0x%lx\n", Status);
//     }
//     _SEH2_END;
//
// Cleanup:
//
//     if (CapturedObjectName.Buffer != NULL)
//         ReleaseCapturedUnicodeString(&CapturedObjectName, UserMode);
//
//     if (CapturedObjectTypeName.Buffer != NULL)
//         ReleaseCapturedUnicodeString(&CapturedObjectTypeName, UserMode);
//
//     if (CapturedSubsystemName.Buffer != NULL)
//         ReleaseCapturedUnicodeString(&CapturedSubsystemName, UserMode);
//
//     if (CapturedSecurityDescriptor != NULL)
//         SeReleaseSecurityDescriptor(CapturedSecurityDescriptor, UserMode, FALSE);
//
//     if (CapturedPrivilegeSet != NULL)
//         ExFreePoolWithTag(CapturedPrivilegeSet, TAG_PRIVILEGE_SET);
//
//     /* Release the security subject context */
//     SeReleaseSubjectContext(&SubjectContext);
//
//     ObDereferenceObject(ClientToken);
//
//     return Status;
// }
//
// /**
//  * @brief
//  * Raises an alarm audit message when a caller attempts to request
//  * a privileged service call.
//  *
//  * @param[in] SubsystemName
//  * A Unicode string that points to a name of the subsystem.
//  *
//  * @param[in] ServiceName
//  * A Unicode string that points to a name of the privileged service.
//  *
//  * @param[in] ClientTokenHandle
//  * A handle to a client access token.
//  *
//  * @param[in] Privileges
//  * An array set of privileges.
//  *
//  * @param[in] AccessGranted
//  * Set this to TRUE if the access attempt was deemed as granted.
//  *
//  * @return
//  * Returns STATUS_SUCCESS if all the operations have been completed successfully.
//  * STATUS_PRIVILEGE_NOT_HELD is returned if the given subject context does not
//  * hold the required audit privilege to actually begin auditing in the first place.
//  * STATUS_BAD_IMPERSONATION_LEVEL is returned if the security impersonation level
//  * of the client token is not on par with the impersonation level that alllows
//  * impersonation. STATUS_INVALID_PARAMETER is returned if the caller has
//  * submitted a bogus set of privileges as such array set exceeds the maximum
//  * count of privileges that the kernel can accept. A failure NTSTATUS code
//  * is returned otherwise.
//  */
// __kernel_entry
// NTSTATUS
// NTAPI
// NtPrivilegedServiceAuditAlarm(
//     _In_opt_ PUNICODE_STRING SubsystemName,
//     _In_opt_ PUNICODE_STRING ServiceName,
//     _In_ HANDLE ClientTokenHandle,
//     _In_ PPRIVILEGE_SET Privileges,
//     _In_ BOOLEAN AccessGranted)
// {
//     KPROCESSOR_MODE PreviousMode;
//     PTOKEN ClientToken;
//     volatile PPRIVILEGE_SET CapturedPrivileges = NULL;
//     UNICODE_STRING CapturedSubsystemName;
//     UNICODE_STRING CapturedServiceName;
//     ULONG PrivilegeCount, PrivilegesSize;
//     SECURITY_SUBJECT_CONTEXT SubjectContext;
//     NTSTATUS Status;
//     PAGED_CODE();
//
//     /* Get the previous mode (only user mode is supported!) */
//     PreviousMode = ExGetPreviousMode();
//     ASSERT(PreviousMode != KernelMode);
//
//     CapturedSubsystemName.Buffer = NULL;
//     CapturedServiceName.Buffer = NULL;
//
//     /* Reference the client token */
//     Status = ObReferenceObjectByHandle(ClientTokenHandle,
//                                        TOKEN_QUERY,
//                                        SeTokenObjectType as *mut _OBJECT_TYPE,
//                                        PreviousMode,
//                                        (PVOID*)&ClientToken,
//                                        NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to reference client token: 0x%lx\n", Status);
//         return Status;
//     }
//
//     /* Validate the token's impersonation level */
//     if ((ClientToken->TokenType == TokenImpersonation) &&
//         (ClientToken->ImpersonationLevel < SecurityIdentification))
//     {
//         DPRINT1("Invalid impersonation level (%u)\n", ClientToken->ImpersonationLevel);
//         ObDereferenceObject(ClientToken);
//         return STATUS_BAD_IMPERSONATION_LEVEL;
//     }
//
//     /* Capture the security subject context */
//     SeCaptureSubjectContext(&SubjectContext);
//
//     /* Check for audit privilege */
//     if (!SeCheckAuditPrivilege(&SubjectContext, PreviousMode))
//     {
//         DPRINT1("Caller does not have SeAuditPrivilege\n");
//         Status = STATUS_PRIVILEGE_NOT_HELD;
//         goto Cleanup;
//     }
//
//     /* Do we have a subsystem name? */
//     if (SubsystemName != NULL)
//     {
//         /* Probe and capture the subsystem name */
//         Status = ProbeAndCaptureUnicodeString(&CapturedSubsystemName,
//                                               PreviousMode,
//                                               SubsystemName);
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT1("Failed to capture subsystem name!\n");
//             goto Cleanup;
//         }
//     }
//
//     /* Do we have a service name? */
//     if (ServiceName != NULL)
//     {
//         /* Probe and capture the service name */
//         Status = ProbeAndCaptureUnicodeString(&CapturedServiceName,
//                                               PreviousMode,
//                                               ServiceName);
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT1("Failed to capture service name!\n");
//             goto Cleanup;
//         }
//     }
//
//     _SEH2_TRY
//     {
//         /* Probe the basic privilege set structure */
//         ProbeForRead(Privileges, sizeof(PRIVILEGE_SET), sizeof(ULONG));
//
//         /* Validate privilege count */
//         PrivilegeCount = Privileges->PrivilegeCount;
//         if (PrivilegeCount > SEP_PRIVILEGE_SET_MAX_COUNT)
//         {
//             Status = STATUS_INVALID_PARAMETER;
//             _SEH2_YIELD(goto Cleanup);
//         }
//
//         /* Calculate the size of the Privileges structure */
//         PrivilegesSize = FIELD_OFFSET(PRIVILEGE_SET, Privilege[PrivilegeCount]);
//
//         /* Probe the whole structure */
//         ProbeForRead(Privileges, PrivilegesSize, sizeof(ULONG));
//
//         /* Allocate a temp buffer */
//         CapturedPrivileges = ExAllocatePoolWithTag(PagedPool,
//                                                    PrivilegesSize,
//                                                    TAG_PRIVILEGE_SET);
//         if (CapturedPrivileges == NULL)
//         {
//             DPRINT1("Failed to allocate %u bytes\n", PrivilegesSize);
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//             _SEH2_YIELD(goto Cleanup);
//         }
//
//         /* Copy the privileges */
//         RtlCopyMemory(CapturedPrivileges, Privileges, PrivilegesSize);
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = _SEH2_GetExceptionCode();
//         DPRINT1("Got exception 0x%lx\n", Status);
//         _SEH2_YIELD(goto Cleanup);
//     }
//     _SEH2_END;
//
//     /* Call the internal function */
//     SepAdtPrivilegedServiceAuditAlarm(&SubjectContext,
//                                       SubsystemName ? &CapturedSubsystemName : NULL,
//                                       ServiceName ? &CapturedServiceName : NULL,
//                                       ClientToken,
//                                       SubjectContext.PrimaryToken,
//                                       CapturedPrivileges,
//                                       AccessGranted);
//
//     Status = STATUS_SUCCESS;
//
// Cleanup:
//     /* Cleanup resources */
//     if (CapturedSubsystemName.Buffer != NULL)
//         ReleaseCapturedUnicodeString(&CapturedSubsystemName, PreviousMode);
//
//     if (CapturedServiceName.Buffer != NULL)
//         ReleaseCapturedUnicodeString(&CapturedServiceName, PreviousMode);
//
//     if (CapturedPrivileges != NULL)
//         ExFreePoolWithTag(CapturedPrivileges, TAG_PRIVILEGE_SET);
//
//     /* Release the security subject context */
//     SeReleaseSubjectContext(&SubjectContext);
//
//     ObDereferenceObject(ClientToken);
//
//     return Status;
// }
//
// /**
//  * @brief
//  * Raises an alarm audit message when a caller attempts to access a
//  * privileged object.
//  *
//  * @param[in] SubsystemName
//  * A Unicode string that points to a name of the subsystem.
//  *
//  * @param[in] HandleId
//  * A handle to an ID that is used as identification instance for auditing.
//  *
//  * @param[in] ClientToken
//  * A handle to a client access token.
//  *
//  * @param[in] DesiredAccess
//  * A handle to a client access token.
//  *
//  * @param[in] Privileges
//  * An array set of privileges.
//  *
//  * @param[in] AccessGranted
//  * Set this to TRUE if the access attempt was deemed as granted.
//  *
//  * @return
//  * To be added...
//  */
// NTSTATUS NTAPI
// NtPrivilegeObjectAuditAlarm(
//     _In_ PUNICODE_STRING SubsystemName,
//     _In_ PVOID HandleId,
//     _In_ HANDLE ClientToken,
//     _In_ ULONG DesiredAccess,
//     _In_ PPRIVILEGE_SET Privileges,
//     _In_ BOOLEAN AccessGranted)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// /**
//  * @brief
//  * Raises an alarm audit message when a caller attempts to access an
//  * object and determine if the access can be made.
//  *
//  * @param[in] SubsystemName
//  * A Unicode string that points to a name of the subsystem.
//  *
//  * @param[in] HandleId
//  * A handle to an ID that is used as identification instance for auditing.
//  *
//  * @param[in] ObjectTypeName
//  * The name of the object type.
//  *
//  * @param[in] ObjectName
//  * The object name.
//  *
//  * @param[in] SecurityDescriptor
//  * A security descriptor.
//  *
//  * @param[in] DesiredAccess
//  * The desired access rights masks requested by the caller.
//  *
//  * @param[in] GenericMapping
//  * The generic mapping of access mask rights.
//  *
//  * @param[in] ObjectCreation
//  * Set this to TRUE if the object has just been created.
//  *
//  * @param[out] GrantedAccess
//  * Returns the granted access rights.
//  *
//  * @param[out] AccessStatus
//  * Returns a NTSTATUS status code indicating whether access check
//  * can be granted or not.
//  *
//  * @param[out] GenerateOnClose
//  * Returns TRUE if the function has generated a list of granted access rights and
//  * status codes on termination, FALSE otherwise.
//  *
//  * @return
//  * See SepAccessCheckAndAuditAlarm.
//  */
// _Must_inspect_result_
// __kernel_entry
// NTSTATUS
// NTAPI
// NtAccessCheckAndAuditAlarm(
//     _In_ PUNICODE_STRING SubsystemName,
//     _In_opt_ PVOID HandleId,
//     _In_ PUNICODE_STRING ObjectTypeName,
//     _In_ PUNICODE_STRING ObjectName,
//     _In_ PSECURITY_DESCRIPTOR SecurityDescriptor,
//     _In_ ACCESS_MASK DesiredAccess,
//     _In_ PGENERIC_MAPPING GenericMapping,
//     _In_ BOOLEAN ObjectCreation,
//     _Out_ PACCESS_MASK GrantedAccess,
//     _Out_ PNTSTATUS AccessStatus,
//     _Out_ PBOOLEAN GenerateOnClose)
// {
//     /* Call the internal function */
//     return SepAccessCheckAndAuditAlarm(SubsystemName,
//                                        HandleId,
//                                        NULL,
//                                        ObjectTypeName,
//                                        ObjectName,
//                                        SecurityDescriptor,
//                                        NULL,
//                                        DesiredAccess,
//                                        AuditEventObjectAccess,
//                                        0,
//                                        NULL,
//                                        0,
//                                        GenericMapping,
//                                        GrantedAccess,
//                                        AccessStatus,
//                                        GenerateOnClose,
//                                        FALSE);
// }
//
// /**
//  * @brief
//  * Raises an alarm audit message when a caller attempts to access an
//  * object and determine if the access can be made by type.
//  *
//  * @param[in] SubsystemName
//  * A Unicode string that points to a name of the subsystem.
//  *
//  * @param[in] HandleId
//  * A handle to an ID that is used as identification instance for auditing.
//  *
//  * @param[in] ObjectTypeName
//  * The name of the object type.
//  *
//  * @param[in] ObjectName
//  * The object name.
//  *
//  * @param[in] SecurityDescriptor
//  * A security descriptor.
//  *
//  * @param[in] PrincipalSelfSid
//  * A principal self user SID.
//  *
//  * @param[in] DesiredAccess
//  * The desired access rights masks requested by the caller.
//  *
//  * @param[in] AuditType
//  * Type of audit to start, influencing how the audit should
//  * be done.
//  *
//  * @param[in] Flags
//  * Flag bitmask, used to check if auditing can be done
//  * without privileges.
//  *
//  * @param[in] ObjectTypeList
//  * A list of object types.
//  *
//  * @param[in] ObjectTypeLength
//  * The length size of the list.
//  *
//  * @param[in] GenericMapping
//  * The generic mapping of access mask rights.
//  *
//  * @param[in] ObjectCreation
//  * Set this to TRUE if the object has just been created.
//  *
//  * @param[out] GrantedAccess
//  * Returns the granted access rights.
//  *
//  * @param[out] AccessStatus
//  * Returns a NTSTATUS status code indicating whether access check
//  * can be granted or not.
//  *
//  * @param[out] GenerateOnClose
//  * Returns TRUE if the function has generated a list of granted access rights and
//  * status codes on termination, FALSE otherwise.
//  *
//  * @return
//  * See SepAccessCheckAndAuditAlarm.
//  */
// _Must_inspect_result_
// __kernel_entry
// NTSTATUS
// NTAPI
// NtAccessCheckByTypeAndAuditAlarm(
//     _In_ PUNICODE_STRING SubsystemName,
//     _In_opt_ PVOID HandleId,
//     _In_ PUNICODE_STRING ObjectTypeName,
//     _In_ PUNICODE_STRING ObjectName,
//     _In_ PSECURITY_DESCRIPTOR SecurityDescriptor,
//     _In_opt_ PSID PrincipalSelfSid,
//     _In_ ACCESS_MASK DesiredAccess,
//     _In_ AUDIT_EVENT_TYPE AuditType,
//     _In_ ULONG Flags,
//     _In_reads_opt_(ObjectTypeLength) POBJECT_TYPE_LIST ObjectTypeList,
//     _In_ ULONG ObjectTypeLength,
//     _In_ PGENERIC_MAPPING GenericMapping,
//     _In_ BOOLEAN ObjectCreation,
//     _Out_ PACCESS_MASK GrantedAccess,
//     _Out_ PNTSTATUS AccessStatus,
//     _Out_ PBOOLEAN GenerateOnClose)
// {
//     /* Call the internal function */
//     return SepAccessCheckAndAuditAlarm(SubsystemName,
//                                        HandleId,
//                                        NULL,
//                                        ObjectTypeName,
//                                        ObjectName,
//                                        SecurityDescriptor,
//                                        PrincipalSelfSid,
//                                        DesiredAccess,
//                                        AuditType,
//                                        Flags,
//                                        ObjectTypeList,
//                                        ObjectTypeLength,
//                                        GenericMapping,
//                                        GrantedAccess,
//                                        AccessStatus,
//                                        GenerateOnClose,
//                                        FALSE);
// }
//
// /**
//  * @brief
//  * Raises an alarm audit message when a caller attempts to access an
//  * object and determine if the access can be made by given type result.
//  *
//  * @param[in] SubsystemName
//  * A Unicode string that points to a name of the subsystem.
//  *
//  * @param[in] HandleId
//  * A handle to an ID that is used as identification instance for auditing.
//  *
//  * @param[in] ObjectTypeName
//  * The name of the object type.
//  *
//  * @param[in] ObjectName
//  * The object name.
//  *
//  * @param[in] SecurityDescriptor
//  * A security descriptor.
//  *
//  * @param[in] PrincipalSelfSid
//  * A principal self user SID.
//  *
//  * @param[in] DesiredAccess
//  * The desired access rights masks requested by the caller.
//  *
//  * @param[in] AuditType
//  * Type of audit to start, influencing how the audit should
//  * be done.
//  *
//  * @param[in] Flags
//  * Flag bitmask, used to check if auditing can be done
//  * without privileges.
//  *
//  * @param[in] ObjectTypeList
//  * A list of object types.
//  *
//  * @param[in] ObjectTypeLength
//  * The length size of the list.
//  *
//  * @param[in] GenericMapping
//  * The generic mapping of access mask rights.
//  *
//  * @param[in] ObjectCreation
//  * Set this to TRUE if the object has just been created.
//  *
//  * @param[out] GrantedAccessList
//  * Returns the granted access rights.
//  *
//  * @param[out] AccessStatusList
//  * Returns a NTSTATUS status code indicating whether access check
//  * can be granted or not.
//  *
//  * @param[out] GenerateOnClose
//  * Returns TRUE if the function has generated a list of granted access rights and
//  * status codes on termination, FALSE otherwise.
//  *
//  * @return
//  * See SepAccessCheckAndAuditAlarm.
//  */
// _Must_inspect_result_
// __kernel_entry
// NTSTATUS
// NTAPI
// NtAccessCheckByTypeResultListAndAuditAlarm(
//     _In_ PUNICODE_STRING SubsystemName,
//     _In_opt_ PVOID HandleId,
//     _In_ PUNICODE_STRING ObjectTypeName,
//     _In_ PUNICODE_STRING ObjectName,
//     _In_ PSECURITY_DESCRIPTOR SecurityDescriptor,
//     _In_opt_ PSID PrincipalSelfSid,
//     _In_ ACCESS_MASK DesiredAccess,
//     _In_ AUDIT_EVENT_TYPE AuditType,
//     _In_ ULONG Flags,
//     _In_reads_opt_(ObjectTypeListLength) POBJECT_TYPE_LIST ObjectTypeList,
//     _In_ ULONG ObjectTypeListLength,
//     _In_ PGENERIC_MAPPING GenericMapping,
//     _In_ BOOLEAN ObjectCreation,
//     _Out_writes_(ObjectTypeListLength) PACCESS_MASK GrantedAccessList,
//     _Out_writes_(ObjectTypeListLength) PNTSTATUS AccessStatusList,
//     _Out_ PBOOLEAN GenerateOnClose)
// {
//     /* Call the internal function */
//     return SepAccessCheckAndAuditAlarm(SubsystemName,
//                                        HandleId,
//                                        NULL,
//                                        ObjectTypeName,
//                                        ObjectName,
//                                        SecurityDescriptor,
//                                        PrincipalSelfSid,
//                                        DesiredAccess,
//                                        AuditType,
//                                        Flags,
//                                        ObjectTypeList,
//                                        ObjectTypeListLength,
//                                        GenericMapping,
//                                        GrantedAccessList,
//                                        AccessStatusList,
//                                        GenerateOnClose,
//                                        TRUE);
// }
//
// /**
//  * @brief
//  * Raises an alarm audit message when a caller attempts to access an
//  * object and determine if the access can be made by given type result
//  * and a token handle.
//  *
//  * @param[in] SubsystemName
//  * A Unicode string that points to a name of the subsystem.
//  *
//  * @param[in] HandleId
//  * A handle to an ID that is used as identification instance for auditing.
//  *
//  * @param[in] ClientToken
//  * A handle to a client access token.
//  *
//  * @param[in] ObjectTypeName
//  * The name of the object type.
//  *
//  * @param[in] ObjectName
//  * The object name.
//  *
//  * @param[in] SecurityDescriptor
//  * A security descriptor.
//  *
//  * @param[in] PrincipalSelfSid
//  * A principal self user SID.
//  *
//  * @param[in] DesiredAccess
//  * The desired access rights masks requested by the caller.
//  *
//  * @param[in] AuditType
//  * Type of audit to start, influencing how the audit should
//  * be done.
//  *
//  * @param[in] Flags
//  * Flag bitmask, used to check if auditing can be done
//  * without privileges.
//  *
//  * @param[in] ObjectTypeList
//  * A list of object types.
//  *
//  * @param[in] ObjectTypeLength
//  * The length size of the list.
//  *
//  * @param[in] GenericMapping
//  * The generic mapping of access mask rights.
//  *
//  * @param[in] ObjectCreation
//  * Set this to TRUE if the object has just been created.
//  *
//  * @param[out] GrantedAccessList
//  * Returns the granted access rights.
//  *
//  * @param[out] AccessStatusList
//  * Returns a NTSTATUS status code indicating whether access check
//  * can be granted or not.
//  *
//  * @param[out] GenerateOnClose
//  * Returns TRUE if the function has generated a list of granted access rights and
//  * status codes on termination, FALSE otherwise.
//  *
//  * @return
//  * See SepAccessCheckAndAuditAlarm.
//  */
// _Must_inspect_result_
// __kernel_entry
// NTSTATUS
// NTAPI
// NtAccessCheckByTypeResultListAndAuditAlarmByHandle(
//     _In_ PUNICODE_STRING SubsystemName,
//     _In_opt_ PVOID HandleId,
//     _In_ HANDLE ClientToken,
//     _In_ PUNICODE_STRING ObjectTypeName,
//     _In_ PUNICODE_STRING ObjectName,
//     _In_ PSECURITY_DESCRIPTOR SecurityDescriptor,
//     _In_opt_ PSID PrincipalSelfSid,
//     _In_ ACCESS_MASK DesiredAccess,
//     _In_ AUDIT_EVENT_TYPE AuditType,
//     _In_ ULONG Flags,
//     _In_reads_opt_(ObjectTypeListLength) POBJECT_TYPE_LIST ObjectTypeList,
//     _In_ ULONG ObjectTypeListLength,
//     _In_ PGENERIC_MAPPING GenericMapping,
//     _In_ BOOLEAN ObjectCreation,
//     _Out_writes_(ObjectTypeListLength) PACCESS_MASK GrantedAccessList,
//     _Out_writes_(ObjectTypeListLength) PNTSTATUS AccessStatusList,
//     _Out_ PBOOLEAN GenerateOnClose)
// {
//     UNREFERENCED_PARAMETER(ObjectCreation);
//
//     /* Call the internal function */
//     return SepAccessCheckAndAuditAlarm(SubsystemName,
//                                        HandleId,
//                                        &ClientToken,
//                                        ObjectTypeName,
//                                        ObjectName,
//                                        SecurityDescriptor,
//                                        PrincipalSelfSid,
//                                        DesiredAccess,
//                                        AuditType,
//                                        Flags,
//                                        ObjectTypeList,
//                                        ObjectTypeListLength,
//                                        GenericMapping,
//                                        GrantedAccessList,
//                                        AccessStatusList,
//                                        GenerateOnClose,
//                                        TRUE);
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
