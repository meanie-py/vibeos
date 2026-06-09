//! Rust translation of `sdk/lib/rtl/sysvol.c` —
//! `RtlCreateSystemVolumeInformationFolder`: create the
//! `\…\System Volume Information` directory on a volume root with a
//! LocalSystem-owned, admin-inheritable security descriptor; on failure,
//! take ownership (SeTakeOwnershipPrivilege) and retry, then normalize the
//! owner/DACL. Three file-static helpers + the one NTAPI entry point.
//!
//! Uses the ntkrnl-sys SE types + the Rtl SE exports (sid/acl/sd are Rust)
//! + Zw security/file I/O + the shared RtlpAllocateMemory/RtlpFreeMemory
//! (rtlp.h; `RtlpAllocate/FreeStringMemory` are #defines onto those, so we
//! call the real symbols directly).
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case, non_upper_case_globals, non_camel_case_types)]
// The SD pointers are pre-initialized to NULL for the failure-path cleanup
// guards (required on 64-bit; flagged "never read" on i386 where that path
// is cfg'd out) — faithful to the C, which NULL-inits them for the same reason.
#![allow(unused_assignments)]

use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ddk::{
    ACE, ACL, HANDLE, PACE, PACL, PISECURITY_DESCRIPTOR, PSECURITY_DESCRIPTOR, PSID,
    PUNICODE_STRING, RtlAbsoluteToSelfRelativeSD, RtlAddAccessAllowedAceEx,
    RtlAllocateAndInitializeSid, RtlCreateAcl, RtlCreateSecurityDescriptor, RtlEqualSid, RtlFreeSid,
    RtlGetAce, RtlGetDaclSecurityDescriptor, RtlGetOwnerSecurityDescriptor, RtlInitUnicodeString,
    RtlLengthSid, RtlSelfRelativeToAbsoluteSD2, RtlSetDaclSecurityDescriptor,
    RtlSetOwnerSecurityDescriptor, SID_IDENTIFIER_AUTHORITY, TOKEN_PRIVILEGES,
    ZwAdjustPrivilegesToken, ZwClose, ZwCreateFile, ZwOpenFile, ZwOpenProcessToken,
    ZwQuerySecurityObject, ZwSetSecurityObject, _IO_STATUS_BLOCK, _OBJECT_ATTRIBUTES,
    _SECURITY_DESCRIPTOR, _UNICODE_STRING,
};
use ntkrnl_sys::rtl::RtlCopyMemory;

// Fixed winnt.h / ntifs.h values not surfaced by ntkrnl-sys.
const SE_TAKE_OWNERSHIP_PRIVILEGE: u32 = 9;
const FILE_DIRECTORY_FILE: ULONG = 0x0000_0001;
const FILE_OPEN_IF: ULONG = 0x0000_0003;
const FILE_ATTRIBUTE_HIDDEN: ULONG = 0x0000_0002;
const FILE_ATTRIBUTE_SYSTEM: ULONG = 0x0000_0004;
const FILE_SYNCHRONOUS_IO_NONALERT: ULONG = 0x0000_0020;

// Shared user/kernel RTL allocation helpers (rtlp.h; defined in libsupp).
// `RtlpAllocateStringMemory`/`RtlpFreeStringMemory` are #defines onto these.
extern "system" {
    fn RtlpAllocateMemory(Bytes: SIZE_T, Tag: ULONG) -> PVOID;
    fn RtlpFreeMemory(Mem: PVOID, Tag: ULONG);
}

// L"System Volume Information\0" (RtlInitUnicodeString scans for the NUL).
static SVI_NAME: [u16; 26] = [
    0x0053, 0x0079, 0x0073, 0x0074, 0x0065, 0x006d, 0x0020, 0x0056, 0x006f, 0x006c, 0x0075, 0x006d,
    0x0065, 0x0020, 0x0049, 0x006e, 0x0066, 0x006f, 0x0072, 0x006d, 0x0061, 0x0074, 0x0069, 0x006f,
    0x006e, 0x0000,
];

// SECURITY_NT_AUTHORITY = {0,0,0,0,0,5}
static mut LocalSystemAuthority: SID_IDENTIFIER_AUTHORITY = SID_IDENTIFIER_AUTHORITY {
    Value: [0, 0, 0, 0, 0, 5],
};

unsafe fn RtlpSysVolCreateSecurityDescriptor(
    SecurityDescriptor: *mut PISECURITY_DESCRIPTOR,
    SystemSid: *mut PSID,
) -> NTSTATUS {
    let mut AbsSD: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
    let mut LocalSystemSid: PSID = core::ptr::null_mut();
    let mut Dacl: PACL = core::ptr::null_mut();
    let DaclSize: ULONG;
    let mut Status: NTSTATUS;

    Status = RtlAllocateAndInitializeSid(
        core::ptr::addr_of_mut!(LocalSystemAuthority),
        1,
        SECURITY_LOCAL_SYSTEM_RID as ULONG,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        core::ptr::addr_of_mut!(LocalSystemSid),
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    'cleanup: {
        AbsSD = RtlpAllocateMemory(
            core::mem::size_of::<_SECURITY_DESCRIPTOR>() as SIZE_T,
            u32::from_le_bytes(*b"SeSd"),
        ) as PSECURITY_DESCRIPTOR;
        if AbsSD.is_null() {
            Status = STATUS_NO_MEMORY;
            break 'cleanup;
        }

        Status = RtlCreateSecurityDescriptor(AbsSD, SECURITY_DESCRIPTOR_REVISION as ULONG);
        if !NT_SUCCESS(Status) {
            break 'cleanup;
        }

        DaclSize = (core::mem::size_of::<ACL>()
            + core::mem::size_of::<ACE>()) as ULONG
            + RtlLengthSid(LocalSystemSid);
        Dacl = RtlpAllocateMemory(DaclSize as SIZE_T, u32::from_le_bytes(*b"SeAc")) as PACL;
        if Dacl.is_null() {
            Status = STATUS_NO_MEMORY;
            break 'cleanup;
        }

        Status = RtlCreateAcl(Dacl, DaclSize, ACL_REVISION as ULONG);
        if !NT_SUCCESS(Status) {
            break 'cleanup;
        }

        Status = RtlAddAccessAllowedAceEx(
            Dacl,
            ACL_REVISION as ULONG,
            (OBJECT_INHERIT_ACE | CONTAINER_INHERIT_ACE) as ULONG,
            (STANDARD_RIGHTS_ALL | SPECIFIC_RIGHTS_ALL) as ULONG,
            LocalSystemSid,
        );
        if !NT_SUCCESS(Status) {
            break 'cleanup;
        }

        Status = RtlSetDaclSecurityDescriptor(AbsSD, 1u8, Dacl, 0u8);

        if NT_SUCCESS(Status) {
            *SecurityDescriptor = AbsSD as PISECURITY_DESCRIPTOR;
            *SystemSid = LocalSystemSid;
            return Status;
        }
        break 'cleanup;
    }

    if !LocalSystemSid.is_null() {
        RtlFreeSid(LocalSystemSid);
    }

    if !Dacl.is_null() {
        RtlpFreeMemory(Dacl as PVOID, u32::from_le_bytes(*b"SeAc"));
    }

    if !AbsSD.is_null() {
        RtlpFreeMemory(AbsSD as PVOID, u32::from_le_bytes(*b"SeSd"));
    }

    Status
}

unsafe fn RtlpSysVolCheckOwnerAndSecurity(
    DirectoryHandle: HANDLE,
    SecurityDescriptor: PISECURITY_DESCRIPTOR,
) -> NTSTATUS {
    let mut RelSD: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
    let mut NewRelSD: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
    let mut AbsSD: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
    #[cfg(target_pointer_width = "64")]
    let mut AbsSDAllocated: BOOLEAN = 0u8;
    let mut AdminSid: PSID = core::ptr::null_mut();
    let mut LocalSystemSid: PSID = core::ptr::null_mut();
    let mut DescriptorSize: ULONG = 0;
    let mut AbsSDSize: ULONG;
    let mut RelSDSize: ULONG = 0;
    let mut Dacl: PACL = core::ptr::null_mut();
    let mut DaclPresent: BOOLEAN = 0u8;
    let mut DaclDefaulted: BOOLEAN = 0u8;
    let mut OwnerSid: PSID = core::ptr::null_mut();
    let mut OwnerDefaulted: BOOLEAN = 0u8;
    let mut AceIndex: ULONG;
    let mut Ace: PACE = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    Status = 'cleanup: loop {
        Status = ZwQuerySecurityObject(
            DirectoryHandle,
            OWNER_SECURITY_INFORMATION | DACL_SECURITY_INFORMATION,
            core::ptr::null_mut(),
            0,
            &mut DescriptorSize,
        );
        if Status != STATUS_BUFFER_TOO_SMALL {
            break 'cleanup STATUS_SUCCESS;
        }

        RelSD = RtlpAllocateMemory(DescriptorSize as SIZE_T, u32::from_le_bytes(*b"SeSd"));
        if RelSD.is_null() {
            break 'cleanup STATUS_NO_MEMORY;
        }

        Status = ZwQuerySecurityObject(
            DirectoryHandle,
            OWNER_SECURITY_INFORMATION | DACL_SECURITY_INFORMATION,
            RelSD,
            DescriptorSize,
            &mut DescriptorSize,
        );
        if !NT_SUCCESS(Status) {
            break 'cleanup Status;
        }

        Status = RtlGetOwnerSecurityDescriptor(RelSD, &mut OwnerSid, &mut OwnerDefaulted);
        if !NT_SUCCESS(Status) {
            break 'cleanup Status;
        }

        Status = RtlGetDaclSecurityDescriptor(RelSD, &mut DaclPresent, &mut Dacl, &mut DaclDefaulted);
        if !NT_SUCCESS(Status) {
            break 'cleanup Status;
        }

        Status = RtlAllocateAndInitializeSid(
            core::ptr::addr_of_mut!(LocalSystemAuthority),
            2,
            SECURITY_BUILTIN_DOMAIN_RID as ULONG,
            DOMAIN_ALIAS_RID_ADMINS as ULONG,
            0,
            0,
            0,
            0,
            0,
            0,
            &mut AdminSid,
        );
        if !NT_SUCCESS(Status) {
            break 'cleanup Status;
        }

        Status = RtlAllocateAndInitializeSid(
            core::ptr::addr_of_mut!(LocalSystemAuthority),
            1,
            SECURITY_LOCAL_SYSTEM_RID as ULONG,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            &mut LocalSystemSid,
        );
        if !NT_SUCCESS(Status) {
            break 'cleanup Status;
        }

        if !OwnerSid.is_null()
            && RtlEqualSid(OwnerSid, AdminSid) != 0
            && DaclPresent != 0
            && !Dacl.is_null()
        {
            AceIndex = 0;
            loop {
                Status = RtlGetAce(Dacl, {
                    let i = AceIndex;
                    AceIndex += 1;
                    i
                }, &mut Ace as *mut PACE as *mut PVOID);
                if !NT_SUCCESS(Status) {
                    Ace = core::ptr::null_mut();
                } else if !Ace.is_null()
                    && (*Ace).Header.AceType == ACCESS_ALLOWED_ACE_TYPE
                {
                    if RtlEqualSid(Ace.add(1) as PSID, LocalSystemSid) != 0 {
                        if ((*Ace).Header.AceFlags & OBJECT_INHERIT_ACE) == 0
                            || ((*Ace).Header.AceFlags & CONTAINER_INHERIT_ACE) == 0
                        {
                            (*Ace).Header.AceFlags |=
                                (OBJECT_INHERIT_ACE | CONTAINER_INHERIT_ACE) as u8;
                            Status = ZwSetSecurityObject(
                                DirectoryHandle,
                                DACL_SECURITY_INFORMATION,
                                RelSD,
                            );
                        } else {
                            Status = STATUS_SUCCESS;
                        }

                        break 'cleanup Status;
                    }
                }

                if Ace.is_null() {
                    break;
                }
            }
        }

        AbsSDSize = DescriptorSize;

        Status = RtlSelfRelativeToAbsoluteSD2(RelSD, &mut AbsSDSize);

        #[cfg(target_pointer_width = "64")]
        {
            if Status == STATUS_BUFFER_TOO_SMALL {
                AbsSD = RtlpAllocateMemory(DescriptorSize as SIZE_T, u32::from_le_bytes(*b"SeSd"));
                if AbsSD.is_null() {
                    break 'cleanup STATUS_NO_MEMORY;
                }

                AbsSDAllocated = 1u8;

                RtlCopyMemory(AbsSD, RelSD, DescriptorSize as usize);

                Status = RtlSelfRelativeToAbsoluteSD2(AbsSD, &mut AbsSDSize);
            } else {
                AbsSD = RelSD;
            }
        }
        #[cfg(not(target_pointer_width = "64"))]
        {
            AbsSD = RelSD;
        }

        if !NT_SUCCESS(Status) {
            break 'cleanup Status;
        }

        Status = RtlSetOwnerSecurityDescriptor(AbsSD, AdminSid, 0u8);
        if !NT_SUCCESS(Status) {
            break 'cleanup Status;
        }

        Status = RtlSetDaclSecurityDescriptor(
            AbsSD,
            1u8,
            (*SecurityDescriptor).Dacl,
            0u8,
        );
        if !NT_SUCCESS(Status) {
            break 'cleanup Status;
        }

        Status = RtlAbsoluteToSelfRelativeSD(AbsSD, core::ptr::null_mut(), &mut RelSDSize);
        if Status != STATUS_BUFFER_TOO_SMALL {
            break 'cleanup Status;
        }

        NewRelSD = RtlpAllocateMemory(RelSDSize as SIZE_T, u32::from_le_bytes(*b"SeSd"));
        if NewRelSD.is_null() {
            break 'cleanup STATUS_NO_MEMORY;
        }

        Status = RtlAbsoluteToSelfRelativeSD(AbsSD, NewRelSD, &mut RelSDSize);
        if Status == STATUS_BUFFER_TOO_SMALL {
            break 'cleanup Status;
        }

        Status = ZwSetSecurityObject(
            DirectoryHandle,
            OWNER_SECURITY_INFORMATION | DACL_SECURITY_INFORMATION,
            NewRelSD,
        );

        break 'cleanup Status;
    };

    if !AdminSid.is_null() {
        RtlFreeSid(AdminSid);
    }

    if !LocalSystemSid.is_null() {
        RtlFreeSid(LocalSystemSid);
    }

    if !RelSD.is_null() {
        RtlpFreeMemory(RelSD, u32::from_le_bytes(*b"SeSd"));
    }

    if !NewRelSD.is_null() {
        RtlpFreeMemory(NewRelSD, u32::from_le_bytes(*b"SeSd"));
    }

    #[cfg(target_pointer_width = "64")]
    {
        if AbsSDAllocated != 0 {
            RtlpFreeMemory(AbsSD, u32::from_le_bytes(*b"SeSd"));
        }
    }

    Status
}

unsafe fn RtlpSysVolTakeOwnership(
    DirectoryPath: PUNICODE_STRING,
    SecurityDescriptor: PSECURITY_DESCRIPTOR,
) -> NTSTATUS {
    let mut TokenPrivileges: TOKEN_PRIVILEGES = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut AbsSD: _SECURITY_DESCRIPTOR = core::mem::zeroed();
    let mut AdminSid: PSID = core::ptr::null_mut();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut TokenEnabled: u8 = 0u8;
    let mut hToken: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut hDirectory: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut Status: NTSTATUS;
    let mut ReturnLength: u32 = 0;

    'cleanup: loop {
        Status = ZwOpenProcessToken(
            -1isize as *mut core::ffi::c_void,
            (TOKEN_QUERY | TOKEN_ADJUST_PRIVILEGES) as u32,
            &mut hToken,
        );
        if !NT_SUCCESS(Status) {
            break 'cleanup;
        }

        TokenPrivileges.PrivilegeCount = 1;
        TokenPrivileges.Privileges[0].Luid.LowPart = SE_TAKE_OWNERSHIP_PRIVILEGE as u32;
        TokenPrivileges.Privileges[0].Luid.HighPart = 0;
        TokenPrivileges.Privileges[0].Attributes = SE_PRIVILEGE_ENABLED as u32;
        Status = ZwAdjustPrivilegesToken(
            hToken,
            0u8,
            &mut TokenPrivileges,
            core::mem::size_of::<TOKEN_PRIVILEGES>() as u32,
            &mut TokenPrivileges,
            &mut ReturnLength,
        );
        if !NT_SUCCESS(Status) {
            break 'cleanup;
        }
        TokenEnabled = if TokenPrivileges.PrivilegeCount != 0 { 1u8 } else { 0u8 };

        ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
        ObjectAttributes.RootDirectory = core::ptr::null_mut();
        ObjectAttributes.ObjectName = DirectoryPath;
        ObjectAttributes.Attributes = 0;
        ObjectAttributes.SecurityDescriptor = SecurityDescriptor;
        ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

        Status = ZwOpenFile(
            &mut hDirectory,
            (SYNCHRONIZE | WRITE_OWNER) as u32,
            &mut ObjectAttributes,
            &mut IoStatusBlock,
            (FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE) as u32,
            (FILE_DIRECTORY_FILE | FILE_SYNCHRONOUS_IO_NONALERT) as u32,
        );
        if !NT_SUCCESS(Status) {
            break 'cleanup;
        }

        Status = RtlAllocateAndInitializeSid(
            core::ptr::addr_of_mut!(LocalSystemAuthority),
            2,
            SECURITY_BUILTIN_DOMAIN_RID as u32,
            DOMAIN_ALIAS_RID_ADMINS as u32,
            0,
            0,
            0,
            0,
            0,
            0,
            &mut AdminSid,
        );
        if !NT_SUCCESS(Status) {
            break 'cleanup;
        }

        Status = RtlCreateSecurityDescriptor(
            &mut AbsSD as *mut _ as PSECURITY_DESCRIPTOR,
            SECURITY_DESCRIPTOR_REVISION as u32,
        );
        if !NT_SUCCESS(Status) {
            break 'cleanup;
        }

        Status = RtlSetOwnerSecurityDescriptor(
            &mut AbsSD as *mut _ as PSECURITY_DESCRIPTOR,
            AdminSid,
            0u8,
        );
        if !NT_SUCCESS(Status) {
            break 'cleanup;
        }

        Status = ZwSetSecurityObject(
            hDirectory,
            OWNER_SECURITY_INFORMATION,
            &mut AbsSD as *mut _ as PSECURITY_DESCRIPTOR,
        );

        break 'cleanup;
    }

    if TokenEnabled != 0 {
        let _Status2 = ZwAdjustPrivilegesToken(
            hToken,
            0u8,
            &mut TokenPrivileges,
            0,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
    }

    if !AdminSid.is_null() {
        RtlFreeSid(AdminSid);
    }

    if !hDirectory.is_null() {
        ZwClose(hDirectory);
    }

    if !hToken.is_null() {
        ZwClose(hToken);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn RtlCreateSystemVolumeInformationFolder(
    VolumeRootPath: PUNICODE_STRING,
) -> NTSTATUS {
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut hDirectory: HANDLE = core::ptr::null_mut();
    let mut DirectoryName: _UNICODE_STRING = core::mem::zeroed();
    let mut NewPath: _UNICODE_STRING = core::mem::zeroed();
    let mut PathLen: ULONG;
    let mut SecurityDescriptor: PISECURITY_DESCRIPTOR = core::ptr::null_mut();
    let mut SystemSid: PSID = core::ptr::null_mut();
    let mut AddSep: BOOLEAN = 0u8;
    let mut Status: NTSTATUS;

    RtlInitUnicodeString(&mut DirectoryName, SVI_NAME.as_ptr());

    PathLen = (*VolumeRootPath).Length as ULONG + DirectoryName.Length as ULONG;

    if PathLen > 0xFFFC {
        return STATUS_INVALID_PARAMETER;
    }

    if *(*VolumeRootPath)
        .Buffer
        .add(((*VolumeRootPath).Length as usize / core::mem::size_of::<u16>()) - 1)
        != b'\\' as u16
    {
        AddSep = 1u8;
        PathLen += core::mem::size_of::<u16>() as ULONG;
    }

    NewPath.MaximumLength = PathLen as USHORT + core::mem::size_of::<u16>() as USHORT;
    NewPath.Buffer = RtlpAllocateMemory(NewPath.MaximumLength as SIZE_T, u32::from_le_bytes(*b"RTSU")) as *mut u16;
    if NewPath.Buffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    NewPath.Length = (*VolumeRootPath).Length;
    RtlCopyMemory(
        NewPath.Buffer as PVOID,
        (*VolumeRootPath).Buffer as PVOID,
        NewPath.Length as usize,
    );
    if AddSep != 0 {
        *NewPath
            .Buffer
            .add(NewPath.Length as usize / core::mem::size_of::<u16>()) = b'\\' as u16;
        NewPath.Length += core::mem::size_of::<u16>() as USHORT;
    }
    RtlCopyMemory(
        NewPath
            .Buffer
            .add(NewPath.Length as usize / core::mem::size_of::<u16>()) as PVOID,
        DirectoryName.Buffer as PVOID,
        DirectoryName.Length as usize,
    );
    NewPath.Length += DirectoryName.Length;
    *NewPath
        .Buffer
        .add(NewPath.Length as usize / core::mem::size_of::<u16>()) = 0u16;

    Status = RtlpSysVolCreateSecurityDescriptor(&mut SecurityDescriptor, &mut SystemSid);
    if NT_SUCCESS(Status) {
        ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
        ObjectAttributes.RootDirectory = core::ptr::null_mut();
        ObjectAttributes.Attributes = 0;
        ObjectAttributes.ObjectName = &mut NewPath;
        ObjectAttributes.SecurityDescriptor = SecurityDescriptor as PVOID;
        ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

        Status = ZwCreateFile(
            &mut hDirectory,
            SYNCHRONIZE | WRITE_OWNER | WRITE_DAC | READ_CONTROL,
            &mut ObjectAttributes,
            &mut IoStatusBlock,
            core::ptr::null_mut(),
            FILE_ATTRIBUTE_SYSTEM | FILE_ATTRIBUTE_HIDDEN,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            FILE_OPEN_IF,
            FILE_DIRECTORY_FILE | FILE_SYNCHRONOUS_IO_NONALERT,
            core::ptr::null_mut(),
            0,
        );
        if !NT_SUCCESS(Status) {
            Status = RtlpSysVolTakeOwnership(&mut NewPath, SecurityDescriptor as PSECURITY_DESCRIPTOR);

            if NT_SUCCESS(Status) {
                Status = ZwCreateFile(
                    &mut hDirectory,
                    SYNCHRONIZE | WRITE_OWNER | WRITE_DAC | READ_CONTROL,
                    &mut ObjectAttributes,
                    &mut IoStatusBlock,
                    core::ptr::null_mut(),
                    FILE_ATTRIBUTE_SYSTEM | FILE_ATTRIBUTE_HIDDEN,
                    FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
                    FILE_OPEN_IF,
                    FILE_DIRECTORY_FILE | FILE_SYNCHRONOUS_IO_NONALERT,
                    core::ptr::null_mut(),
                    0,
                );
            }
        }

        if NT_SUCCESS(Status) {
            Status = RtlpSysVolCheckOwnerAndSecurity(hDirectory, SecurityDescriptor);
            ZwClose(hDirectory);
        }

        RtlpFreeMemory((*SecurityDescriptor).Dacl as PVOID, u32::from_le_bytes(*b"SeAc"));
        RtlpFreeMemory(SecurityDescriptor as PVOID, u32::from_le_bytes(*b"SeSd"));

        RtlFreeSid(SystemSid);
    }

    RtlpFreeMemory(NewPath.Buffer as PVOID, u32::from_le_bytes(*b"RTSU"));
    Status
}
