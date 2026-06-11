use win32k_sys::win32k::*;
use ntkrnl_sys::ddk::{ObAssignSecurity, ObGetObjectSecurity, ObReleaseObjectSecurity, RtlAbsoluteToSelfRelativeSD, RtlAddAccessAllowedAceEx, RtlCreateAcl, RtlCreateSecurityDescriptor, RtlLengthSid, RtlSetDaclSecurityDescriptor, RtlSetGroupSecurityDescriptor, RtlSetOwnerSecurityDescriptor, ZwAllocateVirtualMemory, ZwClose, ZwFreeVirtualMemory, ZwOpenProcessToken, ZwOpenThreadToken, ZwQueryInformationToken};
use ntkrnl_sys::base::{NT_SUCCESS, STATUS_NO_MEMORY, STATUS_SUCCESS};
use ntkrnl_sys::constants::{STATUS_BUFFER_TOO_SMALL, STATUS_NO_TOKEN, STATUS_UNSUCCESSFUL};

// ExDesktopObjectType / SeExports are DATA exports from ntoskrnl (ntoskrnl.spec
// `@ extern`). The win32k_sys glob declares them as plain externs, which resolve
// to garbage on amd64 (PE DLL data imports need __imp_ indirection — see
// [[feedback_exeventobjecttype_amd64_ptr_to_ptr]] / csr.rs LpcPortObjectType).
// Per-arch: i386/arm64 raw-dylib +verbatim makes the thunk (symbol IS the value);
// amd64 binds __imp_ explicitly (ptr-to-ptr) and derefs once. The local externs
// shadow the (weaker) glob imports.
#[cfg(any(target_arch = "x86", target_arch = "aarch64"))]
#[link(name = "ntoskrnl.exe", kind = "raw-dylib", modifiers = "+verbatim")]
extern "C" {
    static mut ExDesktopObjectType: *mut _OBJECT_TYPE;
    static mut SeExports: PSE_EXPORTS;
}
#[cfg(target_arch = "x86_64")]
extern "C" {
    #[link_name = "__imp_ExDesktopObjectType"]
    static mut ExDesktopObjectType_imp: *mut *mut _OBJECT_TYPE;
    #[link_name = "__imp_SeExports"]
    static mut SeExports_imp: *mut PSE_EXPORTS;
}
#[cfg(any(target_arch = "x86", target_arch = "aarch64"))]
#[inline(always)]
unsafe fn ex_desktop_object_type() -> PVOID {
    ExDesktopObjectType as PVOID
}
#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn ex_desktop_object_type() -> PVOID {
    *ExDesktopObjectType_imp as PVOID
}
#[cfg(any(target_arch = "x86", target_arch = "aarch64"))]
#[inline(always)]
unsafe fn se_exports() -> PSE_EXPORTS {
    SeExports
}
#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn se_exports() -> PSE_EXPORTS {
    *SeExports_imp
}

unsafe fn IntGetCurrentAccessToken() -> HANDLE {
    let mut Status: NTSTATUS;
    let mut TokenHandle: HANDLE = core::ptr::null_mut();

    Status = ZwOpenThreadToken(
        -2isize as HANDLE,
        TOKEN_QUERY,
        0u8,
        core::ptr::addr_of_mut!(TokenHandle),
    );
    if !NT_SUCCESS(Status) {
        if Status == STATUS_NO_TOKEN {
            Status = ZwOpenProcessToken(
                -1isize as HANDLE,
                TOKEN_QUERY,
                core::ptr::addr_of_mut!(TokenHandle),
            );
            if !NT_SUCCESS(Status) {
                return core::ptr::null_mut();
            }

            return TokenHandle;
        }

        return core::ptr::null_mut();
    }

    TokenHandle
}

#[no_mangle]
pub unsafe extern "C" fn IntAllocateSecurityBuffer(mut Length: SIZE_T) -> PVOID {
    let mut Buffer: PVOID = core::ptr::null_mut();

    let Status = ZwAllocateVirtualMemory(
        -1isize as HANDLE,
        &mut Buffer,
        0,
        &mut Length,
        MEM_COMMIT,
        PAGE_READWRITE,
    );
    if !NT_SUCCESS(Status) {
        return core::ptr::null_mut();
    }

    Buffer
}

#[no_mangle]
pub unsafe extern "C" fn IntFreeSecurityBuffer(mut Buffer: PVOID) {
    let mut Size: SIZE_T = 0;

    ZwFreeVirtualMemory(
        -1isize as HANDLE,
        core::ptr::addr_of_mut!(Buffer),
        core::ptr::addr_of_mut!(Size),
        MEM_RELEASE,
    );
}

#[no_mangle]
pub unsafe extern "C" fn IntQueryUserSecurityIdentification(
    User: *mut PTOKEN_USER,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut UserToken: PTOKEN_USER = core::ptr::null_mut();
    let Token: HANDLE;
    let mut BufferLength: ULONG = 0;

    *User = core::ptr::null_mut();

    Token = IntGetCurrentAccessToken();
    if Token.is_null() {
        return STATUS_UNSUCCESSFUL;
    }

    Status = ZwQueryInformationToken(
        Token,
        TokenUser,
        core::ptr::null_mut(),
        0,
        core::ptr::addr_of_mut!(BufferLength),
    );
    if Status == STATUS_BUFFER_TOO_SMALL {
        UserToken = IntAllocateSecurityBuffer(BufferLength as SIZE_T) as PTOKEN_USER;
        if UserToken.is_null() {
            ZwClose(Token);
            return STATUS_NO_MEMORY;
        }
    } else if !NT_SUCCESS(Status) {
        ZwClose(Token);
        return Status;
    }

    Status = ZwQueryInformationToken(
        Token,
        TokenUser,
        UserToken as PVOID,
        BufferLength,
        core::ptr::addr_of_mut!(BufferLength),
    );
    if !NT_SUCCESS(Status) {
        IntFreeSecurityBuffer(UserToken as PVOID);
        ZwClose(Token);
        return Status;
    }

    *User = UserToken;
    ZwClose(Token);

    STATUS_SUCCESS
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn IntAssignDesktopSecurityOnParse(
    WinSta: PWINSTATION_OBJECT,
    Desktop: PDESKTOP,
    AccessState: PACCESS_STATE,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut CapturedDescriptor: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
    let mut MemoryAllocated: BOOLEAN = 0u8;

    Status = ObGetObjectSecurity(
        WinSta as *mut core::ffi::c_void,
        &mut CapturedDescriptor,
        &mut MemoryAllocated,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = ObAssignSecurity(
        AccessState as _,
        CapturedDescriptor,
        Desktop as *mut core::ffi::c_void,
        ex_desktop_object_type() as _,
    );

    ObReleaseObjectSecurity(CapturedDescriptor, MemoryAllocated);
    Status
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntAssignDesktopSecurityOnParse(
    WinSta: PWINSTATION_OBJECT,
    Desktop: PDESKTOP,
    AccessState: PACCESS_STATE,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut CapturedDescriptor: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
    let mut MemoryAllocated: BOOLEAN = 0u8;

    Status = ObGetObjectSecurity(
        WinSta as *mut core::ffi::c_void,
        &mut CapturedDescriptor,
        &mut MemoryAllocated,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = ObAssignSecurity(
        AccessState as _,
        CapturedDescriptor,
        Desktop as *mut core::ffi::c_void,
        ex_desktop_object_type() as _,
    );

    ObReleaseObjectSecurity(CapturedDescriptor, MemoryAllocated);
    Status
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn IntCreateServiceSecurity(
    ServiceSd: *mut PSECURITY_DESCRIPTOR,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut ServiceDacl: PACL;
    let DaclSize: ULONG;
    let mut RelSDSize: ULONG;
    let mut AbsSD: _SECURITY_DESCRIPTOR = core::mem::zeroed();
    let mut RelSD: PSECURITY_DESCRIPTOR;
    let mut pTokenUser: PTOKEN_USER;

    RelSDSize = 0;
    pTokenUser = core::ptr::null_mut();
    RelSD = core::ptr::null_mut();
    ServiceDacl = core::ptr::null_mut();

    Status = IntQueryUserSecurityIdentification(&mut pTokenUser);
    if pTokenUser.is_null() {
        return Status;
    }

    'quit: {
        Status = RtlCreateSecurityDescriptor(
            core::ptr::addr_of_mut!(AbsSD) as PSECURITY_DESCRIPTOR,
            SECURITY_DESCRIPTOR_REVISION as ULONG,
        );
        if !NT_SUCCESS(Status) {
            break 'quit;
        }

        DaclSize = core::mem::size_of::<ACL>() as ULONG
            + core::mem::size_of::<ACCESS_ALLOWED_ACE>() as ULONG
            + RtlLengthSid((*pTokenUser).User.Sid)
            + core::mem::size_of::<ACCESS_ALLOWED_ACE>() as ULONG
            + RtlLengthSid((*pTokenUser).User.Sid)
            + core::mem::size_of::<ACCESS_ALLOWED_ACE>() as ULONG
            + RtlLengthSid((*se_exports()).SeAliasAdminsSid)
            + core::mem::size_of::<ACCESS_ALLOWED_ACE>() as ULONG
            + RtlLengthSid((*se_exports()).SeAliasAdminsSid);

        ServiceDacl = IntAllocateSecurityBuffer(DaclSize as SIZE_T) as PACL;
        if ServiceDacl.is_null() {
            Status = STATUS_NO_MEMORY;
            break 'quit;
        }

        Status = RtlCreateAcl(ServiceDacl as _, DaclSize, ACL_REVISION as ULONG);
        if !NT_SUCCESS(Status) {
            break 'quit;
        }

        Status = RtlAddAccessAllowedAceEx(
            ServiceDacl as _,
            ACL_REVISION as ULONG,
            0,
            WINSTA_ACCESS_ALL,
            (*pTokenUser).User.Sid,
        );
        if !NT_SUCCESS(Status) {
            break 'quit;
        }

        Status = RtlAddAccessAllowedAceEx(
            ServiceDacl as _,
            ACL_REVISION as ULONG,
            (INHERIT_ONLY_ACE | NO_PROPAGATE_INHERIT_ACE | OBJECT_INHERIT_ACE) as ULONG,
            DESKTOP_ALL_ACCESS,
            (*pTokenUser).User.Sid,
        );
        if !NT_SUCCESS(Status) {
            break 'quit;
        }

        Status = RtlAddAccessAllowedAceEx(
            ServiceDacl as _,
            ACL_REVISION as ULONG,
            0,
            WINSTA_ENUMERATE,
            (*se_exports()).SeAliasAdminsSid,
        );
        if !NT_SUCCESS(Status) {
            break 'quit;
        }

        Status = RtlAddAccessAllowedAceEx(
            ServiceDacl as _,
            ACL_REVISION as ULONG,
            (INHERIT_ONLY_ACE | NO_PROPAGATE_INHERIT_ACE | OBJECT_INHERIT_ACE) as ULONG,
            DESKTOP_ENUMERATE | DESKTOP_READOBJECTS | DESKTOP_WRITEOBJECTS,
            (*se_exports()).SeAliasAdminsSid,
        );
        if !NT_SUCCESS(Status) {
            break 'quit;
        }

        Status = RtlSetDaclSecurityDescriptor(
            core::ptr::addr_of_mut!(AbsSD) as PSECURITY_DESCRIPTOR,
            1u8,
            ServiceDacl as _,
            0u8,
        );
        if !NT_SUCCESS(Status) {
            break 'quit;
        }

        Status = RtlSetOwnerSecurityDescriptor(
            core::ptr::addr_of_mut!(AbsSD) as PSECURITY_DESCRIPTOR,
            core::ptr::null_mut(),
            0u8,
        );
        if !NT_SUCCESS(Status) {
            break 'quit;
        }

        Status = RtlSetGroupSecurityDescriptor(
            core::ptr::addr_of_mut!(AbsSD) as PSECURITY_DESCRIPTOR,
            core::ptr::null_mut(),
            0u8,
        );
        if !NT_SUCCESS(Status) {
            break 'quit;
        }

        Status = RtlAbsoluteToSelfRelativeSD(
            core::ptr::addr_of_mut!(AbsSD) as PSECURITY_DESCRIPTOR,
            core::ptr::null_mut(),
            &mut RelSDSize,
        );
        if Status != STATUS_BUFFER_TOO_SMALL {
            break 'quit;
        }

        RelSD = IntAllocateSecurityBuffer(RelSDSize as SIZE_T) as PSECURITY_DESCRIPTOR;
        if RelSD.is_null() {
            Status = STATUS_NO_MEMORY;
            break 'quit;
        }

        Status = RtlAbsoluteToSelfRelativeSD(
            core::ptr::addr_of_mut!(AbsSD) as PSECURITY_DESCRIPTOR,
            RelSD,
            &mut RelSDSize,
        );
        if !NT_SUCCESS(Status) {
            break 'quit;
        }

        *ServiceSd = RelSD;
    }

    if !ServiceDacl.is_null() {
        IntFreeSecurityBuffer(ServiceDacl as PVOID);
    }

    if !pTokenUser.is_null() {
        IntFreeSecurityBuffer(pTokenUser as PVOID);
    }

    if !NT_SUCCESS(Status) {
        if !RelSD.is_null() {
            IntFreeSecurityBuffer(RelSD as PVOID);
        }
    }

    Status
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntCreateServiceSecurity(
    ServiceSd: *mut PSECURITY_DESCRIPTOR,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut ServiceDacl: PACL;
    let DaclSize: ULONG;
    let mut RelSDSize: ULONG;
    let mut AbsSD: _SECURITY_DESCRIPTOR = core::mem::zeroed();
    let mut RelSD: PSECURITY_DESCRIPTOR;
    let mut pTokenUser: PTOKEN_USER;

    RelSDSize = 0;
    pTokenUser = core::ptr::null_mut();
    RelSD = core::ptr::null_mut();
    ServiceDacl = core::ptr::null_mut();

    Status = IntQueryUserSecurityIdentification(&mut pTokenUser);
    if pTokenUser.is_null() {
        return Status;
    }

    'quit: {
        Status = RtlCreateSecurityDescriptor(
            core::ptr::addr_of_mut!(AbsSD) as PSECURITY_DESCRIPTOR,
            SECURITY_DESCRIPTOR_REVISION as ULONG,
        );
        if !NT_SUCCESS(Status) {
            break 'quit;
        }

        DaclSize = core::mem::size_of::<ACL>() as ULONG
            + core::mem::size_of::<ACCESS_ALLOWED_ACE>() as ULONG
            + RtlLengthSid((*pTokenUser).User.Sid)
            + core::mem::size_of::<ACCESS_ALLOWED_ACE>() as ULONG
            + RtlLengthSid((*pTokenUser).User.Sid)
            + core::mem::size_of::<ACCESS_ALLOWED_ACE>() as ULONG
            + RtlLengthSid((*se_exports()).SeAliasAdminsSid)
            + core::mem::size_of::<ACCESS_ALLOWED_ACE>() as ULONG
            + RtlLengthSid((*se_exports()).SeAliasAdminsSid);

        ServiceDacl = IntAllocateSecurityBuffer(DaclSize as SIZE_T) as PACL;
        if ServiceDacl.is_null() {
            Status = STATUS_NO_MEMORY;
            break 'quit;
        }

        Status = RtlCreateAcl(ServiceDacl as _, DaclSize, ACL_REVISION as ULONG);
        if !NT_SUCCESS(Status) {
            break 'quit;
        }

        Status = RtlAddAccessAllowedAceEx(
            ServiceDacl as _,
            ACL_REVISION as ULONG,
            0,
            WINSTA_ACCESS_ALL,
            (*pTokenUser).User.Sid,
        );
        if !NT_SUCCESS(Status) {
            break 'quit;
        }

        Status = RtlAddAccessAllowedAceEx(
            ServiceDacl as _,
            ACL_REVISION as ULONG,
            (INHERIT_ONLY_ACE | NO_PROPAGATE_INHERIT_ACE | OBJECT_INHERIT_ACE) as ULONG,
            DESKTOP_ALL_ACCESS,
            (*pTokenUser).User.Sid,
        );
        if !NT_SUCCESS(Status) {
            break 'quit;
        }

        Status = RtlAddAccessAllowedAceEx(
            ServiceDacl as _,
            ACL_REVISION as ULONG,
            0,
            WINSTA_ENUMERATE,
            (*se_exports()).SeAliasAdminsSid,
        );
        if !NT_SUCCESS(Status) {
            break 'quit;
        }

        Status = RtlAddAccessAllowedAceEx(
            ServiceDacl as _,
            ACL_REVISION as ULONG,
            (INHERIT_ONLY_ACE | NO_PROPAGATE_INHERIT_ACE | OBJECT_INHERIT_ACE) as ULONG,
            DESKTOP_ENUMERATE | DESKTOP_READOBJECTS | DESKTOP_WRITEOBJECTS,
            (*se_exports()).SeAliasAdminsSid,
        );
        if !NT_SUCCESS(Status) {
            break 'quit;
        }

        Status = RtlSetDaclSecurityDescriptor(
            core::ptr::addr_of_mut!(AbsSD) as PSECURITY_DESCRIPTOR,
            1u8,
            ServiceDacl as _,
            0u8,
        );
        if !NT_SUCCESS(Status) {
            break 'quit;
        }

        Status = RtlSetOwnerSecurityDescriptor(
            core::ptr::addr_of_mut!(AbsSD) as PSECURITY_DESCRIPTOR,
            core::ptr::null_mut(),
            0u8,
        );
        if !NT_SUCCESS(Status) {
            break 'quit;
        }

        Status = RtlSetGroupSecurityDescriptor(
            core::ptr::addr_of_mut!(AbsSD) as PSECURITY_DESCRIPTOR,
            core::ptr::null_mut(),
            0u8,
        );
        if !NT_SUCCESS(Status) {
            break 'quit;
        }

        Status = RtlAbsoluteToSelfRelativeSD(
            core::ptr::addr_of_mut!(AbsSD) as PSECURITY_DESCRIPTOR,
            core::ptr::null_mut(),
            &mut RelSDSize,
        );
        if Status != STATUS_BUFFER_TOO_SMALL {
            break 'quit;
        }

        RelSD = IntAllocateSecurityBuffer(RelSDSize as SIZE_T) as PSECURITY_DESCRIPTOR;
        if RelSD.is_null() {
            Status = STATUS_NO_MEMORY;
            break 'quit;
        }

        Status = RtlAbsoluteToSelfRelativeSD(
            core::ptr::addr_of_mut!(AbsSD) as PSECURITY_DESCRIPTOR,
            RelSD,
            &mut RelSDSize,
        );
        if !NT_SUCCESS(Status) {
            break 'quit;
        }

        *ServiceSd = RelSD;
    }

    if !ServiceDacl.is_null() {
        IntFreeSecurityBuffer(ServiceDacl as PVOID);
    }

    if !pTokenUser.is_null() {
        IntFreeSecurityBuffer(pTokenUser as PVOID);
    }

    if !NT_SUCCESS(Status) {
        if !RelSD.is_null() {
            IntFreeSecurityBuffer(RelSD as PVOID);
        }
    }

    Status
}
