use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::macros::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ps::PsGetCurrentProcess;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::probe::*;

use ntkrnl_sys::ddk::{
    _OBJECT_SYMBOLIC_LINK, _EPROCESS, _UNICODE_STRING,
    POBJECT_SYMBOLIC_LINK, POBJECT_HEADER, POBJECT_TYPE, POBJECT_DIRECTORY, POBJECT_HEADER_NAME_INFO,
    PDEVICE_MAP, PDEVICE_OBJECT, PACCESS_STATE, PSECURITY_QUALITY_OF_SERVICE,
    ACCESS_MASK, KPROCESSOR_MODE, HANDLE, PHANDLE, POBJECT_ATTRIBUTES, PUNICODE_STRING,
    PLARGE_INTEGER,
    ExFreePool,
    ObpInitializeLookupContext, ObpReleaseLookupContext,
    ObpLookupEntryDirectory, ObpReferenceNameInfo, ObpDereferenceNameInfo,
    ObpAcquireObjectLock, ObpReleaseObjectLock,
    ObReferenceObjectByPointer, ObCreateObject, ObInsertObject, ObOpenObjectByName,
    ObReferenceObjectByHandle,
    OBP_LOOKUP_CONTEXT,
    RtlInitUnicodeString, RtlUpcaseUnicodeChar,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

// ============================================================================
// Constants not in ntkrnl-sys (C #defines)
// ============================================================================

const STATUS_NAME_TOO_LONG: NTSTATUS = 0xC0000106_u32 as i32;

// Drive type constants (from ndk/obtypes.h)
const DOSDEVICE_DRIVE_UNKNOWN: ULONG = 0;
const DOSDEVICE_DRIVE_CALCULATE: ULONG = 1;
const DOSDEVICE_DRIVE_REMOVABLE: ULONG = 2;
const DOSDEVICE_DRIVE_FIXED: ULONG = 3;
const DOSDEVICE_DRIVE_REMOTE: ULONG = 4;
const DOSDEVICE_DRIVE_CDROM: ULONG = 5;
const DOSDEVICE_DRIVE_RAMDISK: ULONG = 6;

// FILE_DEVICE_* constants (from xdk/iotypes.h)
const FILE_DEVICE_CD_ROM: ULONG = 0x00000002;
const FILE_DEVICE_CD_ROM_FILE_SYSTEM: ULONG = 0x00000003;
const FILE_DEVICE_DISK: ULONG = 0x00000007;
const FILE_DEVICE_DISK_FILE_SYSTEM: ULONG = 0x00000008;
const FILE_DEVICE_FILE_SYSTEM: ULONG = 0x00000009;
const FILE_DEVICE_NETWORK: ULONG = 0x00000012;
const FILE_DEVICE_NETWORK_FILE_SYSTEM: ULONG = 0x00000014;
const FILE_DEVICE_VIRTUAL_DISK: ULONG = 0x00000024;

// FILE_REMOVABLE_MEDIA (from xdk/iotypes.h)
const FILE_REMOVABLE_MEDIA: ULONG = 0x00000001;

// Pool tag for symlink target (from ntoskrnl/include/internal/tag.h)
// TAG_SYMLINK_TARGET = 'TMYS' = 0x534D5954
const TAG_SYMLINK_TARGET: ULONG = u32::from_le_bytes(*b"SYMT");

// ============================================================================
// Extern declarations for kernel functions/globals not in ntkrnl-sys
// ============================================================================

extern "C" {
    static mut IoDeviceObjectType: POBJECT_TYPE;
}

// KeQuerySystemTime: real function on i386, macro on amd64/arm64 reading
// SharedUserData->SystemTime (xdk/amd64/ke.h). This used to read the INTERRUPT time
// (uptime) there, so symbolic links got an uptime-derived CreationTime on 64-bit.
#[cfg(target_arch = "x86")]
extern "system" {
    fn KeQuerySystemTime(CurrentTime: PLARGE_INTEGER);
}

#[cfg(not(target_arch = "x86"))]
use ntkrnl_sys::inline::ke::KeQuerySystemTime;

// ============================================================================
// Inline probe helpers (C macros in probe.h)
// ============================================================================

/// probe.h's ProbeForWriteUnicodeString: range-check, then read the struct and
/// write it back to prove the page writable. The C raises into the caller's
/// _SEH2_TRY; this returns the code. (It was a bare read/write with no range
/// check, so a user caller could have the kernel rewrite a kernel address.)
#[inline]
unsafe fn ProbeForWriteUnicodeString(ptr: *mut _UNICODE_STRING) -> Result<_UNICODE_STRING, NTSTATUS> {
    let val = ntkrnl_sys::probe::probe_read_generic(ptr as *const _UNICODE_STRING, 1, 1)?;
    ntkrnl_sys::probe::write_user(ptr, val, 0)?;
    Ok(val)
}

#[no_mangle]
pub static mut ObpSymbolicLinkObjectType: POBJECT_TYPE = core::ptr::null_mut();

unsafe fn ObpProcessDosDeviceSymbolicLink(
    SymbolicLink: POBJECT_SYMBOLIC_LINK,
    DeleteLink: BOOLEAN,
) {
    let mut DeviceMap: PDEVICE_MAP;
    let mut TargetPath: _UNICODE_STRING = core::mem::zeroed();
    let mut LocalTarget: _UNICODE_STRING = core::mem::zeroed();
    let mut NameDirectory: POBJECT_DIRECTORY;
    let mut DirectoryObject: POBJECT_DIRECTORY;
    let mut MaxReparse: ULONG;
    let mut LookupContext: OBP_LOOKUP_CONTEXT = core::mem::zeroed();
    let mut DriveType: ULONG;
    let mut ObjectHeader: POBJECT_HEADER;
    let mut ObjectNameInfo: POBJECT_HEADER_NAME_INFO;
    let mut DirectoryLocked: BOOLEAN;
    let mut Object: PVOID;

    MaxReparse = 32;
    NameDirectory = core::ptr::null_mut();

    ObjectHeader = OBJECT_TO_OBJECT_HEADER(SymbolicLink as PVOID);
    ObjectNameInfo = OBJECT_HEADER_TO_NAME_INFO(ObjectHeader);

    if ObjectNameInfo != core::ptr::null_mut() {
        NameDirectory = (*ObjectNameInfo).Directory;
    }

    ObpInitializeLookupContext(&mut LookupContext);

    if (*SymbolicLink).LinkTargetObject != core::ptr::null_mut() || DeleteLink == 0 {
        DirectoryObject = ObpRootDirectoryObject;

        LocalTarget = (*SymbolicLink).LinkTarget;

        if ObpLUIDDeviceMapsEnabled != 0 {
            DeviceMap = ObSystemDeviceMap;
        } else {
            DeviceMap = (*(PsGetCurrentProcess() as *mut _EPROCESS)).DeviceMap as PDEVICE_MAP;
        }

        loop {
            if DeviceMap != core::ptr::null_mut() {
                if (LocalTarget.Buffer as ULONG_PTR) & 7 == 0 {
                    if (*DeviceMap).DosDevicesDirectory != core::ptr::null_mut() {
                        if LocalTarget.Length >= ObpDosDevicesShortName.Length
                            && (*(LocalTarget.Buffer as *const ULONGLONG)
                                == *(&raw const ObpDosDevicesShortNamePrefix as *const _UNICODE_STRING as *const u64))
                        {
                            DirectoryObject = (*DeviceMap).DosDevicesDirectory;

                            LocalTarget.Length -= ObpDosDevicesShortName.Length;
                            LocalTarget.Buffer = LocalTarget.Buffer.offset(
                                (ObpDosDevicesShortName.Length
                                    / core::mem::size_of::<WCHAR>() as USHORT) as isize,
                            );
                        }
                    }
                }
            }

            loop {
                if *LocalTarget.Buffer.offset(0) == OBJ_NAME_PATH_SEPARATOR as u16 {
                    LocalTarget.Buffer = LocalTarget.Buffer.offset(1);
                    LocalTarget.Length -= core::mem::size_of::<WCHAR>() as USHORT;
                }

                TargetPath = LocalTarget;

                if LocalTarget.Length != 0 {
                    loop {
                        if *LocalTarget.Buffer.offset(0) == OBJ_NAME_PATH_SEPARATOR as u16 {
                            break;
                        }

                        LocalTarget.Buffer = LocalTarget.Buffer.offset(1);
                        LocalTarget.Length -= core::mem::size_of::<WCHAR>() as USHORT;

                        if LocalTarget.Length == 0 {
                            break;
                        }
                    }
                }

                TargetPath.Length -= LocalTarget.Length;

                if TargetPath.Length == 0 {
                    ObpReleaseLookupContext(&mut LookupContext);
                    return;
                }

                if DirectoryObject == NameDirectory {
                    DirectoryLocked = LookupContext.DirectoryLocked;
                    LookupContext.DirectoryLocked = 1;
                } else {
                    DirectoryLocked = 0;
                }

                Object = ObpLookupEntryDirectory(
                    DirectoryObject,
                    &mut TargetPath as *mut _UNICODE_STRING as PUNICODE_STRING,
                    0,
                    0,
                    &mut LookupContext,
                );

                if DirectoryObject == NameDirectory {
                    LookupContext.DirectoryLocked = DirectoryLocked;
                }

                if Object == core::ptr::null_mut() {
                    break;
                }

                if (*OBJECT_TO_OBJECT_HEADER(Object)).Type != ObpDirectoryObjectType {
                    if (*OBJECT_TO_OBJECT_HEADER(Object)).Type != ObpSymbolicLinkObjectType
                        || (*((Object as POBJECT_SYMBOLIC_LINK))).DosDeviceDriveIndex != 0
                    {
                        break;
                    }

                    if MaxReparse == 0 {
                        Object = core::ptr::null_mut();
                        break;
                    }

                    MaxReparse -= 1;

                    DirectoryObject = ObpRootDirectoryObject;

                    LocalTarget = (*(Object as POBJECT_SYMBOLIC_LINK)).LinkTarget;

                    continue;
                }

                DirectoryObject = Object as POBJECT_DIRECTORY;
            }

            break;
        }
    } else {
        Object = core::ptr::null_mut();
    }

    DeviceMap = core::ptr::null_mut();
    if (*SymbolicLink).DosDeviceDriveIndex != 0 {
        ObjectHeader = OBJECT_TO_OBJECT_HEADER(SymbolicLink as PVOID);
        ObjectNameInfo = ObpReferenceNameInfo(ObjectHeader);
        if ObjectNameInfo != core::ptr::null_mut() {
            if (*ObjectNameInfo).Directory != core::ptr::null_mut() {
                DeviceMap = (*(*ObjectNameInfo).Directory).DeviceMap;
            }

            ObpDereferenceNameInfo(ObjectNameInfo);
        }
    }

    if DeleteLink != 0 {
        RtlInitUnicodeString(&mut (*SymbolicLink).LinkTargetRemaining, core::ptr::null_mut());

        if (*SymbolicLink).LinkTargetObject != core::ptr::null_mut() {
            ObDereferenceObject((*SymbolicLink).LinkTargetObject);
            (*SymbolicLink).LinkTargetObject = core::ptr::null_mut();
        }

        if DeviceMap != core::ptr::null_mut() {
            KeAcquireGuardedMutex(core::ptr::addr_of_mut!(ObpDeviceMapLock) as PKGUARDED_MUTEX);

            (*DeviceMap).DriveType[((*SymbolicLink).DosDeviceDriveIndex - 1) as usize] =
                DOSDEVICE_DRIVE_UNKNOWN as u8;
            (*DeviceMap).DriveMap &= !(1 << ((*SymbolicLink).DosDeviceDriveIndex - 1));

            KeReleaseGuardedMutex(core::ptr::addr_of_mut!(ObpDeviceMapLock) as PKGUARDED_MUTEX);

            (*SymbolicLink).DosDeviceDriveIndex = 0;
        }
    } else {
        DriveType = DOSDEVICE_DRIVE_CALCULATE;

        if Object != core::ptr::null_mut()
            && (*SymbolicLink).DosDeviceDriveIndex != 0
            && (*OBJECT_TO_OBJECT_HEADER(Object)).Type == IoDeviceObjectType
        {
            DriveType = match (*(Object as PDEVICE_OBJECT)).DeviceType {
                FILE_DEVICE_VIRTUAL_DISK => DOSDEVICE_DRIVE_RAMDISK,
                FILE_DEVICE_CD_ROM | FILE_DEVICE_CD_ROM_FILE_SYSTEM => DOSDEVICE_DRIVE_CDROM,
                FILE_DEVICE_DISK | FILE_DEVICE_DISK_FILE_SYSTEM | FILE_DEVICE_FILE_SYSTEM => {
                    if (*(Object as PDEVICE_OBJECT)).Characteristics & FILE_REMOVABLE_MEDIA != 0 {
                        DOSDEVICE_DRIVE_REMOVABLE
                    } else {
                        DOSDEVICE_DRIVE_FIXED
                    }
                }
                FILE_DEVICE_NETWORK | FILE_DEVICE_NETWORK_FILE_SYSTEM => DOSDEVICE_DRIVE_REMOTE,
                _ => DOSDEVICE_DRIVE_UNKNOWN,
            };
        }

        if DeviceMap != core::ptr::null_mut() {
            KeAcquireGuardedMutex(core::ptr::addr_of_mut!(ObpDeviceMapLock) as PKGUARDED_MUTEX);

            (*DeviceMap).DriveType[((*SymbolicLink).DosDeviceDriveIndex - 1) as usize] =
                DriveType as u8;
            (*DeviceMap).DriveMap |= 1 << ((*SymbolicLink).DosDeviceDriveIndex - 1);

            KeReleaseGuardedMutex(core::ptr::addr_of_mut!(ObpDeviceMapLock) as PKGUARDED_MUTEX);
        }
    }

    ObpReleaseLookupContext(&mut LookupContext);
}

#[no_mangle]
pub unsafe extern "system" fn ObpDeleteSymbolicLinkName(SymbolicLink: POBJECT_SYMBOLIC_LINK) {
    ObpProcessDosDeviceSymbolicLink(SymbolicLink, 1);
}

#[no_mangle]
pub unsafe extern "system" fn ObpCreateSymbolicLinkName(SymbolicLink: POBJECT_SYMBOLIC_LINK) {
    let mut UpperDrive: WCHAR;
    let mut ObjectHeader: POBJECT_HEADER;
    let mut ObjectNameInfo: POBJECT_HEADER_NAME_INFO;

    ObjectHeader = OBJECT_TO_OBJECT_HEADER(SymbolicLink as PVOID);
    ObjectNameInfo = ObpReferenceNameInfo(ObjectHeader);

    if ObjectNameInfo == core::ptr::null_mut() {
        return;
    }

    if (*ObjectNameInfo).Directory != core::ptr::null_mut()
        && (*(*ObjectNameInfo).Directory).DeviceMap != core::ptr::null_mut()
    {
        if (*ObjectNameInfo).Name.Length == 2 * core::mem::size_of::<WCHAR>() as USHORT {
            if *(*ObjectNameInfo).Name.Buffer.offset(1) == b':' as u16 {
                UpperDrive = RtlUpcaseUnicodeChar(*(*ObjectNameInfo).Name.Buffer.offset(0));
                if UpperDrive >= b'A' as u16 && UpperDrive <= b'Z' as u16 {
                    (*SymbolicLink).DosDeviceDriveIndex = (UpperDrive - (b'A' as u16 - 1)) as ULONG;
                }
            }
        }

        ObpProcessDosDeviceSymbolicLink(SymbolicLink, 0);
    }

    ObpDereferenceNameInfo(ObjectNameInfo);
}

#[no_mangle]
pub unsafe extern "system" fn ObpDeleteSymbolicLink(ObjectBody: PVOID) {
    let ObjectBody = opaque_ptr(ObjectBody);
    let SymlinkObject = ObjectBody as POBJECT_SYMBOLIC_LINK;

    if (*SymlinkObject).LinkTarget.Buffer != core::ptr::null_mut() {
        ExFreePool((*SymlinkObject).LinkTarget.Buffer as PVOID);
        (*SymlinkObject).LinkTarget.Buffer = core::ptr::null_mut();
    }
}

#[no_mangle]
pub unsafe extern "system" fn ObpParseSymbolicLink(
    ParsedObject: PVOID,
    ObjectType: PVOID,
    AccessState: PACCESS_STATE,
    AccessMode: KPROCESSOR_MODE,
    Attributes: ULONG,
    FullPath: PUNICODE_STRING,
    RemainingName: PUNICODE_STRING,
    Context: PVOID,
    SecurityQos: PSECURITY_QUALITY_OF_SERVICE,
    NextObject: *mut PVOID,
) -> NTSTATUS {
    let ParsedObject = opaque_ptr(ParsedObject);
    let ObjectType = opaque_ptr(ObjectType);
    let AccessState = opaque_ptr(AccessState);
    let FullPath = opaque_ptr(FullPath);
    let RemainingName = opaque_ptr(RemainingName);
    let Context = opaque_ptr(Context);
    let NextObject = opaque_ptr(NextObject);
    let SymlinkObject = ParsedObject as POBJECT_SYMBOLIC_LINK;
    let mut TargetPath: PUNICODE_STRING;
    let mut NewTargetPath: PWSTR;
    let mut LengthUsed: SIZE_T;
    let mut MaximumLength: SIZE_T;
    let mut TempLength: SIZE_T;
    let mut Status: NTSTATUS;

    *NextObject = core::ptr::null_mut();

    if (*RemainingName).Length == 0 {
        if ObjectType != core::ptr::null_mut() {
            Status = ObReferenceObjectByPointer(
                ParsedObject,
                0,
                ObjectType as POBJECT_TYPE,
                AccessMode,
            );
            if Status >= 0 {
                *NextObject = ParsedObject;
            }

            if Status >= 0 || Status != STATUS_OBJECT_TYPE_MISMATCH {
                return Status;
            }
        }
    } else if *(*RemainingName).Buffer.offset(0) != OBJ_NAME_PATH_SEPARATOR as u16 {
        return STATUS_OBJECT_TYPE_MISMATCH;
    }

    if (*SymlinkObject).LinkTargetObject != core::ptr::null_mut() {
        if (*SymlinkObject).LinkTargetRemaining.Length == 0 {
            *NextObject = (*SymlinkObject).LinkTargetObject;
            return STATUS_REPARSE_OBJECT;
        }

        TempLength = (*SymlinkObject).LinkTargetRemaining.Length as SIZE_T;
        if *(*SymlinkObject)
            .LinkTargetRemaining
            .Buffer
            .offset((TempLength / core::mem::size_of::<WCHAR>()) as isize - 1)
            == OBJ_NAME_PATH_SEPARATOR as u16
            && *(*RemainingName).Buffer.offset(0) == OBJ_NAME_PATH_SEPARATOR as u16
        {
            TempLength -= core::mem::size_of::<WCHAR>();
        }

        LengthUsed = TempLength + (*RemainingName).Length as SIZE_T;
        LengthUsed += core::mem::size_of::<WCHAR>()
            * ((*RemainingName).Buffer as usize - (*FullPath).Buffer as usize)
            / core::mem::size_of::<WCHAR>();

        if LengthUsed > 0xFFF0 {
            return STATUS_NAME_TOO_LONG;
        }

        if (*FullPath).MaximumLength as SIZE_T > LengthUsed {
            if (*RemainingName).Length != 0 {
                core::ptr::copy(
                    (*RemainingName).Buffer as *const u8,
                    ((*RemainingName).Buffer as usize + TempLength) as *mut u8,
                    (*RemainingName).Length as usize,
                );
            }

            core::ptr::copy_nonoverlapping(
                (*SymlinkObject).LinkTargetRemaining.Buffer as *const u8,
                (*RemainingName).Buffer as *mut u8,
                TempLength,
            );

            (*FullPath).Length += (*SymlinkObject).LinkTargetRemaining.Length;
            (*RemainingName).Length += (*SymlinkObject).LinkTargetRemaining.Length;
            (*RemainingName).MaximumLength +=
                (*RemainingName).Length + core::mem::size_of::<WCHAR>() as USHORT;
            *(*FullPath)
                .Buffer
                .offset(((*FullPath).Length / core::mem::size_of::<WCHAR>() as USHORT) as isize) =
                0;

            *NextObject = (*SymlinkObject).LinkTargetObject;
            return STATUS_REPARSE_OBJECT;
        }

        MaximumLength = LengthUsed + core::mem::size_of::<WCHAR>();
        NewTargetPath = ExAllocatePoolWithTag(
            NonPagedPool,
            MaximumLength as SIZE_T,
            OB_NAME_TAG,
        ) as PWSTR;
        if NewTargetPath == core::ptr::null_mut() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        core::ptr::copy_nonoverlapping(
            (*FullPath).Buffer as *const u8,
            NewTargetPath as *mut u8,
            core::mem::size_of::<WCHAR>()
                * ((*RemainingName).Buffer as usize - (*FullPath).Buffer as usize)
                / core::mem::size_of::<WCHAR>(),
        );

        if (*RemainingName).Length != 0 {
            core::ptr::copy_nonoverlapping(
                (*RemainingName).Buffer as *const u8,
                (NewTargetPath as usize
                    + ((*RemainingName).Buffer as usize - (*FullPath).Buffer as usize)
                    + TempLength) as *mut u8,
                (*RemainingName).Length as usize,
            );
        }

        core::ptr::copy_nonoverlapping(
            (*SymlinkObject).LinkTargetRemaining.Buffer as *const u8,
            (NewTargetPath as usize
                + ((*RemainingName).Buffer as usize - (*FullPath).Buffer as usize))
                as *mut u8,
            TempLength,
        );

        ExFreePool((*FullPath).Buffer as PVOID);

        (*FullPath).Buffer = NewTargetPath;
        (*FullPath).MaximumLength = MaximumLength as USHORT;
        (*FullPath).Length = LengthUsed as USHORT;

        (*RemainingName).Length = (LengthUsed
            + NewTargetPath as usize
            - (NewTargetPath as usize
                + ((*RemainingName).Buffer as usize - (*FullPath).Buffer as usize)))
            as USHORT;
        (*RemainingName).Buffer = (NewTargetPath as usize
            + ((*RemainingName).Buffer as usize - (*FullPath).Buffer as usize))
            as PWSTR;
        (*RemainingName).MaximumLength =
            (*RemainingName).Length + core::mem::size_of::<WCHAR>() as USHORT;

        *NextObject = (*SymlinkObject).LinkTargetObject;
        return STATUS_REPARSE_OBJECT;
    }

    TargetPath = &mut (*SymlinkObject).LinkTarget;
    TempLength = (*TargetPath).Length as SIZE_T;

    if TempLength != 0 && (*RemainingName).Length != 0 {
        if *(*TargetPath)
            .Buffer
            .offset((TempLength / core::mem::size_of::<WCHAR>()) as isize - 1)
            == OBJ_NAME_PATH_SEPARATOR as u16
            && *(*RemainingName).Buffer.offset(0) == OBJ_NAME_PATH_SEPARATOR as u16
        {
            TempLength -= core::mem::size_of::<WCHAR>();
        }
    }

    LengthUsed = TempLength + (*RemainingName).Length as SIZE_T;

    if LengthUsed > 0xFFF0 {
        return STATUS_NAME_TOO_LONG;
    }

    if (*FullPath).MaximumLength as SIZE_T <= LengthUsed {
        MaximumLength = LengthUsed + core::mem::size_of::<WCHAR>();
        NewTargetPath = ExAllocatePoolWithTag(
            NonPagedPool,
            MaximumLength as SIZE_T,
            OB_NAME_TAG,
        ) as PWSTR;
        if NewTargetPath == core::ptr::null_mut() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }
    } else {
        MaximumLength = (*FullPath).MaximumLength as SIZE_T;
        NewTargetPath = (*FullPath).Buffer;
    }

    if (*RemainingName).Length != 0 {
        core::ptr::copy(
            (*RemainingName).Buffer as *const u8,
            (NewTargetPath as usize + TempLength) as *mut u8,
            (*RemainingName).Length as usize,
        );
    }

    core::ptr::copy_nonoverlapping(
        (*TargetPath).Buffer as *const u8,
        NewTargetPath as *mut u8,
        TempLength,
    );
    *(NewTargetPath as *mut u16).offset((LengthUsed / core::mem::size_of::<WCHAR>()) as isize) = 0;

    if NewTargetPath != (*FullPath).Buffer {
        ExFreePool((*FullPath).Buffer as PVOID);
    }

    (*FullPath).Length = LengthUsed as USHORT;
    (*FullPath).MaximumLength = MaximumLength as USHORT;
    (*FullPath).Buffer = NewTargetPath;

    STATUS_REPARSE
}

#[no_mangle]
pub unsafe extern "system" fn NtCreateSymbolicLinkObject(
    LinkHandle: PHANDLE,
    DesiredAccess: ACCESS_MASK,
    ObjectAttributes: POBJECT_ATTRIBUTES,
    LinkTarget: PUNICODE_STRING,
) -> NTSTATUS {
    let LinkHandle = opaque_ptr(LinkHandle);
    let ObjectAttributes = opaque_ptr(ObjectAttributes);
    let LinkTarget = opaque_ptr(LinkTarget);
    let mut hLink: HANDLE = core::mem::zeroed();
    let mut SymbolicLink: POBJECT_SYMBOLIC_LINK = core::ptr::null_mut();
    let mut CapturedLinkTarget: _UNICODE_STRING;
    let PreviousMode = ExGetPreviousMode();
    let mut Status: NTSTATUS;

    if PreviousMode != 0 {
        if let Err(s) = probe_user_buffer(LinkTarget as _, core::mem::size_of::<_UNICODE_STRING>(), core::mem::size_of::<ULONG>(), PreviousMode) {
            return s;
        }
        CapturedLinkTarget = core::ptr::read_volatile(LinkTarget);
        
        if let Err(s) = probe_user_buffer(
            CapturedLinkTarget.Buffer as _,
            CapturedLinkTarget.MaximumLength as usize,
            core::mem::size_of::<WCHAR>(),
            PreviousMode,
        ) {
            return s;
        }

        if let Err(s) = probe_write_handle(LinkHandle as _, PreviousMode) {
            return s;
        }
    } else {
        CapturedLinkTarget = *LinkTarget;
    }

    if CapturedLinkTarget.MaximumLength % core::mem::size_of::<WCHAR>() as USHORT != 0 {
        CapturedLinkTarget.MaximumLength = (CapturedLinkTarget.MaximumLength as usize
            & !(core::mem::size_of::<WCHAR>() - 1))
            as USHORT;
    }

    if (CapturedLinkTarget.Length % core::mem::size_of::<WCHAR>() as USHORT) != 0
        || (CapturedLinkTarget.MaximumLength < CapturedLinkTarget.Length)
        || CapturedLinkTarget.MaximumLength == 0
    {
        crate::dbg_print!(b"OB: Invalid symbolic link target - %wZ\n\0",
            &CapturedLinkTarget,
        );
        return STATUS_INVALID_PARAMETER;
    }

    Status = ObCreateObject(
        PreviousMode,
        ObpSymbolicLinkObjectType,
        ObjectAttributes,
        PreviousMode,
        core::ptr::null_mut(),
        core::mem::size_of::<_OBJECT_SYMBOLIC_LINK>() as ULONG,
        0,
        0,
        &mut SymbolicLink as *mut POBJECT_SYMBOLIC_LINK as *mut PVOID,
    );
    if Status >= 0 {
        KeQuerySystemTime(&mut (*SymbolicLink).CreationTime as *mut _ as PLARGE_INTEGER);

        (*SymbolicLink).LinkTarget.Length = CapturedLinkTarget.Length;
        (*SymbolicLink).LinkTarget.MaximumLength = CapturedLinkTarget.MaximumLength;
        (*SymbolicLink).LinkTarget.Buffer = ExAllocatePoolWithTag(
            PagedPool,
            CapturedLinkTarget.MaximumLength as SIZE_T,
            TAG_SYMLINK_TARGET,
        ) as PWSTR;
        if (*SymbolicLink).LinkTarget.Buffer == core::ptr::null_mut() {
            ObDereferenceObject(SymbolicLink as PVOID);
            return STATUS_NO_MEMORY;
        }

        // Fault-tolerant capture of the caller's link-target string. The C
        // original wrapped this copy in _SEH2_TRY (dead under DUMMY_PSEH); a
        // kernel-mode caller with a hostile LinkTarget.Buffer (kmtest
        // ObSymbolicLink) faulted memcpy+0x40. On fault, ObDereferenceObject
        // runs the delete procedure which frees the just-allocated buffer.
        if let Err(s) = safe_copy_from_user(
            (*SymbolicLink).LinkTarget.Buffer as *mut core::ffi::c_void,
            CapturedLinkTarget.Buffer as *const core::ffi::c_void,
            CapturedLinkTarget.MaximumLength as usize,
            core::mem::size_of::<WCHAR>(),
            PreviousMode,
        ) {
            ObDereferenceObject(SymbolicLink as PVOID);
            return s;
        }

        (*SymbolicLink).LinkTargetObject = core::ptr::null_mut();
        (*SymbolicLink).DosDeviceDriveIndex = 0;
        RtlInitUnicodeString(&mut (*SymbolicLink).LinkTargetRemaining, core::ptr::null_mut());

        Status = ObInsertObject(
            SymbolicLink as PVOID,
            core::ptr::null_mut(),
            DesiredAccess,
            0,
            core::ptr::null_mut(),
            &mut hLink,
        );
        if Status >= 0 {
            *LinkHandle = hLink;
        }
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtOpenSymbolicLinkObject(
    LinkHandle: PHANDLE,
    DesiredAccess: ACCESS_MASK,
    ObjectAttributes: POBJECT_ATTRIBUTES,
) -> NTSTATUS {
    let LinkHandle = opaque_ptr(LinkHandle);
    let ObjectAttributes = opaque_ptr(ObjectAttributes);
    let mut hLink: HANDLE = core::mem::zeroed();
    let PreviousMode = ExGetPreviousMode();
    let mut Status: NTSTATUS;

    if PreviousMode != 0 {
        if let Err(s) = probe_write_handle(LinkHandle as _, PreviousMode) {
            return s;
        }
    }

    Status = ObOpenObjectByName(
        ObjectAttributes,
        ObpSymbolicLinkObjectType,
        PreviousMode,
        core::ptr::null_mut(),
        DesiredAccess,
        core::ptr::null_mut(),
        &mut hLink,
    );

    *LinkHandle = hLink;

    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtQuerySymbolicLinkObject(
    LinkHandle: HANDLE,
    LinkTarget: PUNICODE_STRING,
    ResultLength: *mut ULONG,
) -> NTSTATUS {
    let LinkHandle = opaque_ptr(LinkHandle);
    let LinkTarget = opaque_ptr(LinkTarget);
    let ResultLength = opaque_ptr(ResultLength);
    let mut Status: NTSTATUS;
    let mut SafeLinkTarget: _UNICODE_STRING = core::mem::zeroed();
    let mut SymlinkObject: POBJECT_SYMBOLIC_LINK = core::ptr::null_mut();
    let mut ObjectHeader: POBJECT_HEADER;
    let mut LengthUsed: ULONG;
    let PreviousMode = ExGetPreviousMode();

    if PreviousMode != 0 {
        // C: under _SEH2_TRY, whose handler returns the exception code.
        SafeLinkTarget = match ProbeForWriteUnicodeString(LinkTarget as *mut _UNICODE_STRING) {
            Ok(target) => target,
            Err(status) => return status,
        };
        if let Err(s) = probe_user_buffer(
            SafeLinkTarget.Buffer as _,
            SafeLinkTarget.MaximumLength as usize,
            core::mem::size_of::<WCHAR>(),
            PreviousMode,
        ) {
            return s;
        }

        if ResultLength != core::ptr::null_mut() {
            if let Err(s) = probe_write_ulong(ResultLength, PreviousMode) {
                return s;
            }
        }
    } else {
        SafeLinkTarget = *(LinkTarget as *const _UNICODE_STRING);
    }

    Status = ObReferenceObjectByHandle(
        LinkHandle,
        SYMBOLIC_LINK_QUERY,
        ObpSymbolicLinkObjectType,
        PreviousMode,
        &mut SymlinkObject as *mut POBJECT_SYMBOLIC_LINK as *mut PVOID,
        core::ptr::null_mut(),
    );
    if Status >= 0 {
        ObjectHeader = OBJECT_TO_OBJECT_HEADER(SymlinkObject as PVOID);

        ObpAcquireObjectLock(ObjectHeader);

        LengthUsed = if ResultLength != core::ptr::null_mut() {
            (*SymlinkObject).LinkTarget.MaximumLength as ULONG
        } else {
            (*SymlinkObject).LinkTarget.Length as ULONG
        };

        // C: these three stores into the caller's memory sit in one _SEH2_TRY, whose
        // handler sets Status to the code. They run under the object lock, which the
        // fault-tolerant copy lets us still release.
        let write_back = |dst: PVOID, src: *const core::ffi::c_void, size: usize| -> Result<(), NTSTATUS> {
            safe_copy_raw(dst, src, size)
        };
        let mut written: Result<(), NTSTATUS> = Ok(());
        if LengthUsed <= SafeLinkTarget.MaximumLength as ULONG {
            let length = (*SymlinkObject).LinkTarget.Length;
            written = write_back(
                SafeLinkTarget.Buffer as PVOID,
                (*SymlinkObject).LinkTarget.Buffer as *const core::ffi::c_void,
                LengthUsed as usize,
            )
            .and_then(|()| {
                write_back(
                    core::ptr::addr_of_mut!((*LinkTarget).Length) as PVOID,
                    &length as *const _ as *const core::ffi::c_void,
                    core::mem::size_of_val(&length),
                )
            });
        } else {
            Status = STATUS_BUFFER_TOO_SMALL;
        }

        if written.is_ok() && ResultLength != core::ptr::null_mut() {
            let maximum = (*SymlinkObject).LinkTarget.MaximumLength as ULONG;
            written = write_back(
                ResultLength as PVOID,
                &maximum as *const ULONG as *const core::ffi::c_void,
                core::mem::size_of::<ULONG>(),
            );
        }
        if let Err(s) = written {
            Status = s;
        }

        ObpReleaseObjectLock(ObjectHeader);
        ObDereferenceObject(SymlinkObject as PVOID);
    }

    Status
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
// * PROJECT:         ReactOS Kernel
// * LICENSE:         GPL - See COPYING in the top level directory
// * FILE:            ntoskrnl/ob/oblink.c
// * PURPOSE:         Implements symbolic links
// * PROGRAMMERS:     Alex Ionescu (alex@relsoft.net)
// *                  David Welch (welch@mcmail.com)
// *                  Pierre Schweitzer
// */
//
// /* INCLUDES *****************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* GLOBALS ******************************************************************/
//
// POBJECT_TYPE ObpSymbolicLinkObjectType = NULL;
//
// /* PRIVATE FUNCTIONS *********************************************************/
//
// VOID
// ObpProcessDosDeviceSymbolicLink(IN POBJECT_SYMBOLIC_LINK SymbolicLink,
//                                 IN BOOLEAN DeleteLink)
// {
//     PDEVICE_MAP DeviceMap;
//     UNICODE_STRING TargetPath, LocalTarget;
//     POBJECT_DIRECTORY NameDirectory, DirectoryObject;
//     ULONG MaxReparse;
//     OBP_LOOKUP_CONTEXT LookupContext;
//     ULONG DriveType;
//     POBJECT_HEADER ObjectHeader;
//     POBJECT_HEADER_NAME_INFO ObjectNameInfo;
//     BOOLEAN DirectoryLocked;
//     PVOID Object;
//
//     /*
//      * To prevent endless reparsing, setting an upper limit on the
//      * number of reparses.
//      */
//     MaxReparse = 32;
//     NameDirectory = NULL;
//
//     /* Get header data */
//     ObjectHeader = OBJECT_TO_OBJECT_HEADER(SymbolicLink);
//     ObjectNameInfo = OBJECT_HEADER_TO_NAME_INFO(ObjectHeader);
//
//     /* Check if we have a directory in our symlink to use */
//     if (ObjectNameInfo != NULL)
//     {
//         NameDirectory = ObjectNameInfo->Directory;
//     }
//
//     /* Initialize lookup context */
//     ObpInitializeLookupContext(&LookupContext);
//
//     /*
//      * If we have to create the link, locate the IoDeviceObject if any
//      * this symbolic link points to.
//      */
//     if (SymbolicLink->LinkTargetObject != NULL || !DeleteLink)
//     {
//         /* Start the search from the root */
//         DirectoryObject = ObpRootDirectoryObject;
//
//         /* Keep track of our progress while parsing the name */
//         LocalTarget = SymbolicLink->LinkTarget;
//
//         /* If LUID mappings are enabled, use system map */
//         if (ObpLUIDDeviceMapsEnabled != 0)
//         {
//             DeviceMap = ObSystemDeviceMap;
//         }
//         /* Otherwise, use the one in the process */
//         else
//         {
//             DeviceMap = PsGetCurrentProcess()->DeviceMap;
//         }
//
// ReparseTargetPath:
//         /*
//          * If we have a device map active, check whether we have a drive
//          * letter prefixed with ??, if so, chomp it
//          */
//         if (DeviceMap != NULL)
//         {
//             if (!((ULONG_PTR)(LocalTarget.Buffer) & 7))
//             {
//                 if (DeviceMap->DosDevicesDirectory != NULL)
//                 {
//                     if (LocalTarget.Length >= ObpDosDevicesShortName.Length &&
//                         (*(PULONGLONG)LocalTarget.Buffer ==
//                          ObpDosDevicesShortNamePrefix.Alignment.QuadPart))
//                     {
//                         DirectoryObject = DeviceMap->DosDevicesDirectory;
//
//                         LocalTarget.Length -= ObpDosDevicesShortName.Length;
//                         LocalTarget.Buffer += (ObpDosDevicesShortName.Length / sizeof(WCHAR));
//                     }
//                 }
//             }
//         }
//
//         /* Try walking the target path and open each part of the path */
//         while (TRUE)
//         {
//             if (LocalTarget.Buffer[0] == OBJ_NAME_PATH_SEPARATOR)
//             {
//                 ++LocalTarget.Buffer;
//                 LocalTarget.Length -= sizeof(WCHAR);
//             }
//
//             /* Remember the current component of the target path */
//             TargetPath = LocalTarget;
//
//             /* Move forward to the next component of the target path */
//             if (LocalTarget.Length != 0)
//             {
//                 do
//                 {
//                     if (LocalTarget.Buffer[0] == OBJ_NAME_PATH_SEPARATOR)
//                     {
//                         break;
//                     }
//
//                     ++LocalTarget.Buffer;
//                     LocalTarget.Length -= sizeof(WCHAR);
//                 }
//                 while (LocalTarget.Length != 0);
//             }
//
//             TargetPath.Length -= LocalTarget.Length;
//
//             /*
//              * Finished processing the entire path, stop
//              * That's a failure case, we quit here
//              */
//             if (TargetPath.Length == 0)
//             {
//                 ObpReleaseLookupContext(&LookupContext);
//                 return;
//             }
//
//
//             /*
//              * Make sure a deadlock does not occur as an exclusive lock on a pushlock
//              * would have already taken one in ObpLookupObjectName() on the parent
//              * directory where the symlink is being created [ObInsertObject()].
//              * Prevent recursive locking by faking lock state in the lookup context
//              * when the current directory is same as the parent directory where
//              * the symlink is being created. If the lock state is not faked,
//              * ObpLookupEntryDirectory() will try to get a recursive lock on the
//              * pushlock and hang. For e.g. this happens when a substed drive is pointed to
//              * another substed drive.
//              */
//             if (DirectoryObject == NameDirectory)
//             {
//                 DirectoryLocked = LookupContext.DirectoryLocked;
//                 LookupContext.DirectoryLocked = TRUE;
//             }
//             else
//             {
//                 DirectoryLocked = FALSE;
//             }
//
//             Object = ObpLookupEntryDirectory(DirectoryObject,
//                                              &TargetPath,
//                                              0,
//                                              FALSE,
//                                              &LookupContext);
//
//             /* Locking was faked, undo it now */
//             if (DirectoryObject == NameDirectory)
//             {
//                 LookupContext.DirectoryLocked = DirectoryLocked;
//             }
//
//             /* Lookup failed, stop */
//             if (Object == NULL)
//             {
//                 break;
//             }
//
//             /* If we don't have a directory object, we'll have to handle the object */
//             if (OBJECT_TO_OBJECT_HEADER(Object)->Type != ObpDirectoryObjectType)
//             {
//                 /* If that's not a symbolic link, stop here, nothing to do */
//                 if (OBJECT_TO_OBJECT_HEADER(Object)->Type != ObpSymbolicLinkObjectType ||
//                     (((POBJECT_SYMBOLIC_LINK)Object)->DosDeviceDriveIndex != 0))
//                 {
//                     break;
//                 }
//
//                 /* We're out of reparse attempts */
//                 if (MaxReparse == 0)
//                 {
//                     Object = NULL;
//                     break;
//                 }
//
//                 --MaxReparse;
//
//                 /* Symlink points to another initialized symlink, ask caller to reparse */
//                 DirectoryObject = ObpRootDirectoryObject;
//
//                 LocalTarget = ((POBJECT_SYMBOLIC_LINK)Object)->LinkTarget;
//
//                 goto ReparseTargetPath;
//             }
//
//             /* Make this current directory, and continue search */
//             DirectoryObject = Object;
//         }
//     }
//
//     DeviceMap = NULL;
//     /* That's a drive letter, find a suitable device map */
//     if (SymbolicLink->DosDeviceDriveIndex != 0)
//     {
//         ObjectHeader = OBJECT_TO_OBJECT_HEADER(SymbolicLink);
//         ObjectNameInfo = ObpReferenceNameInfo(ObjectHeader);
//         if (ObjectNameInfo != NULL)
//         {
//             if (ObjectNameInfo->Directory != NULL)
//             {
//                 DeviceMap = ObjectNameInfo->Directory->DeviceMap;
//             }
//
//             ObpDereferenceNameInfo(ObjectNameInfo);
//         }
//     }
//
//     /* If we were asked to delete the symlink */
//     if (DeleteLink)
//     {
//         /* Zero its target */
//         RtlInitUnicodeString(&SymbolicLink->LinkTargetRemaining, NULL);
//
//         /* If we had a target objected, dereference it */
//         if (SymbolicLink->LinkTargetObject != NULL)
//         {
//             ObDereferenceObject(SymbolicLink->LinkTargetObject);
//             SymbolicLink->LinkTargetObject = NULL;
//         }
//
//         /* If it was a drive letter */
//         if (DeviceMap != NULL)
//         {
//             /* Acquire the device map lock */
//             KeAcquireGuardedMutex(&ObpDeviceMapLock);
//
//             /* Remove the drive entry */
//             DeviceMap->DriveType[SymbolicLink->DosDeviceDriveIndex - 1] =
//                 DOSDEVICE_DRIVE_UNKNOWN;
//             DeviceMap->DriveMap &=
//                 ~(1 << (SymbolicLink->DosDeviceDriveIndex - 1));
//
//             /* Release the device map lock */
//             KeReleaseGuardedMutex(&ObpDeviceMapLock);
//
//             /* Reset the drive index, valid drive index starts from 1 */
//             SymbolicLink->DosDeviceDriveIndex = 0;
//         }
//     }
//     else
//     {
//         DriveType = DOSDEVICE_DRIVE_CALCULATE;
//
//         /* If we have a drive letter and a pointer device object */
//         if (Object != NULL && SymbolicLink->DosDeviceDriveIndex != 0 &&
//             OBJECT_TO_OBJECT_HEADER(Object)->Type == IoDeviceObjectType)
//         {
//             /* Calculate the drive type */
//             switch(((PDEVICE_OBJECT)Object)->DeviceType)
//             {
//                 case FILE_DEVICE_VIRTUAL_DISK:
//                     DriveType = DOSDEVICE_DRIVE_RAMDISK;
//                     break;
//                 case FILE_DEVICE_CD_ROM:
//                 case FILE_DEVICE_CD_ROM_FILE_SYSTEM:
//                     DriveType = DOSDEVICE_DRIVE_CDROM;
//                     break;
//                 case FILE_DEVICE_DISK:
//                 case FILE_DEVICE_DISK_FILE_SYSTEM:
//                 case FILE_DEVICE_FILE_SYSTEM:
//                     if (((PDEVICE_OBJECT)Object)->Characteristics & FILE_REMOVABLE_MEDIA)
//                         DriveType = DOSDEVICE_DRIVE_REMOVABLE;
//                     else
//                         DriveType = DOSDEVICE_DRIVE_FIXED;
//                     break;
//                 case FILE_DEVICE_NETWORK:
//                 case FILE_DEVICE_NETWORK_FILE_SYSTEM:
//                     DriveType = DOSDEVICE_DRIVE_REMOTE;
//                     break;
//                 default:
//                     DPRINT1("Device Type %lu for %wZ is not known or unhandled\n",
//                             ((PDEVICE_OBJECT)Object)->DeviceType,
//                             &SymbolicLink->LinkTarget);
//                     DriveType = DOSDEVICE_DRIVE_UNKNOWN;
//             }
//         }
//
//         /* Add a new drive entry */
//         if (DeviceMap != NULL)
//         {
//             /* Acquire the device map lock */
//             KeAcquireGuardedMutex(&ObpDeviceMapLock);
//
//             DeviceMap->DriveType[SymbolicLink->DosDeviceDriveIndex - 1] =
//                 (UCHAR)DriveType;
//             DeviceMap->DriveMap |=
//                 1 << (SymbolicLink->DosDeviceDriveIndex - 1);
//
//             /* Release the device map lock */
//             KeReleaseGuardedMutex(&ObpDeviceMapLock);
//         }
//     }
//
//     /* Cleanup */
//     ObpReleaseLookupContext(&LookupContext);
// }
//
// VOID
// NTAPI
// ObpDeleteSymbolicLinkName(IN POBJECT_SYMBOLIC_LINK SymbolicLink)
// {
//     /* Just call the helper */
//     ObpProcessDosDeviceSymbolicLink(SymbolicLink, TRUE);
// }
//
// VOID
// NTAPI
// ObpCreateSymbolicLinkName(IN POBJECT_SYMBOLIC_LINK SymbolicLink)
// {
//     WCHAR UpperDrive;
//     POBJECT_HEADER ObjectHeader;
//     POBJECT_HEADER_NAME_INFO ObjectNameInfo;
//
//     /* Get header data */
//     ObjectHeader = OBJECT_TO_OBJECT_HEADER(SymbolicLink);
//     ObjectNameInfo = ObpReferenceNameInfo(ObjectHeader);
//
//     /* No name info, nothing to create */
//     if (ObjectNameInfo == NULL)
//     {
//         return;
//     }
//
//     /* If we have a device map, look for creating a letter based drive */
//     if (ObjectNameInfo->Directory != NULL &&
//         ObjectNameInfo->Directory->DeviceMap != NULL)
//     {
//         /* Is it a drive letter based name? */
//         if (ObjectNameInfo->Name.Length == 2 * sizeof(WCHAR))
//         {
//             if (ObjectNameInfo->Name.Buffer[1] == L':')
//             {
//                 UpperDrive = RtlUpcaseUnicodeChar(ObjectNameInfo->Name.Buffer[0]);
//                 if (UpperDrive >= L'A' && UpperDrive <= L'Z')
//                 {
//                     /* Compute its index (it's 1 based - 0 means no letter) */
//                     SymbolicLink->DosDeviceDriveIndex = UpperDrive - (L'A' - 1);
//                 }
//             }
//         }
//
//         /* Call the helper */
//         ObpProcessDosDeviceSymbolicLink(SymbolicLink, FALSE);
//     }
//
//     /* We're done */
//     ObpDereferenceNameInfo(ObjectNameInfo);
// }
//
// /*++
// * @name ObpDeleteSymbolicLink
// *
// *     The ObpDeleteSymbolicLink routine <FILLMEIN>
// *
// * @param ObjectBody
// *        <FILLMEIN>
// *
// * @return None.
// *
// * @remarks None.
// *
// *--*/
// VOID
// NTAPI
// ObpDeleteSymbolicLink(PVOID ObjectBody)
// {
//     POBJECT_SYMBOLIC_LINK SymlinkObject = (POBJECT_SYMBOLIC_LINK)ObjectBody;
//
//     /* Make sure that the symbolic link has a name */
//     if (SymlinkObject->LinkTarget.Buffer)
//     {
//         /* Free the name */
//         ExFreePool(SymlinkObject->LinkTarget.Buffer);
//         SymlinkObject->LinkTarget.Buffer = NULL;
//     }
// }
//
// /*++
// * @name ObpParseSymbolicLink
// *
// *     The ObpParseSymbolicLink routine <FILLMEIN>
// *
// * @param Object
// *        <FILLMEIN>
// *
// * @param NextObject
// *        <FILLMEIN>
// *
// * @param FullPath
// *        <FILLMEIN>
// *
// * @param RemainingPath
// *        <FILLMEIN>
// *
// * @param Attributes
// *        <FILLMEIN>
// *
// * @return STATUS_SUCCESS or appropriate error value.
// *
// * @remarks None.
// *
// *--*/
// NTSTATUS
// NTAPI
// ObpParseSymbolicLink(IN PVOID ParsedObject,
//                      IN PVOID ObjectType,
//                      IN OUT PACCESS_STATE AccessState,
//                      IN KPROCESSOR_MODE AccessMode,
//                      IN ULONG Attributes,
//                      IN OUT PUNICODE_STRING FullPath,
//                      IN OUT PUNICODE_STRING RemainingName,
//                      IN OUT PVOID Context OPTIONAL,
//                      IN PSECURITY_QUALITY_OF_SERVICE SecurityQos OPTIONAL,
//                      OUT PVOID *NextObject)
// {
//     POBJECT_SYMBOLIC_LINK SymlinkObject = (POBJECT_SYMBOLIC_LINK)ParsedObject;
//     PUNICODE_STRING TargetPath;
//     PWSTR NewTargetPath;
//     SIZE_T LengthUsed, MaximumLength, TempLength;
//     NTSTATUS Status;
//     PAGED_CODE();
//
//     /* Assume failure */
//     *NextObject = NULL;
//
//     /* Check if we're out of name to parse */
//     if (!RemainingName->Length)
//     {
//         /* Check if we got an object type */
//         if (ObjectType)
//         {
//             /* Reference the object only */
//             Status = ObReferenceObjectByPointer(ParsedObject,
//                                                 0,
//                                                 ObjectType,
//                                                 AccessMode);
//             if (NT_SUCCESS(Status))
//             {
//                 /* Return it */
//                 *NextObject = ParsedObject;
//             }
//
//             if ((NT_SUCCESS(Status)) || (Status != STATUS_OBJECT_TYPE_MISMATCH))
//             {
//                 /* Fail */
//                 return Status;
//             }
//         }
//     }
//     else if (RemainingName->Buffer[0] != OBJ_NAME_PATH_SEPARATOR)
//     {
//         /* Symbolic links must start with a backslash */
//         return STATUS_OBJECT_TYPE_MISMATCH;
//     }
//
//     /* Check if this symlink is bound to a specific object */
//     if (SymlinkObject->LinkTargetObject)
//     {
//         /* No name to reparse, directly reparse the object */
//         if (!SymlinkObject->LinkTargetRemaining.Length)
//         {
//             *NextObject = SymlinkObject->LinkTargetObject;
//             return STATUS_REPARSE_OBJECT;
//         }
//
//         TempLength = SymlinkObject->LinkTargetRemaining.Length;
//         /* The target and remaining names aren't empty, so check for slashes */
//         if (SymlinkObject->LinkTargetRemaining.Buffer[TempLength / sizeof(WCHAR) - 1] == OBJ_NAME_PATH_SEPARATOR &&
//             RemainingName->Buffer[0] == OBJ_NAME_PATH_SEPARATOR)
//         {
//             /* Reduce the length by one to cut off the extra '\' */
//             TempLength -= sizeof(OBJ_NAME_PATH_SEPARATOR);
//         }
//
//         /* Calculate the new length */
//         LengthUsed = TempLength + RemainingName->Length;
//         LengthUsed += (sizeof(WCHAR) * (RemainingName->Buffer - FullPath->Buffer));
//
//         /* Check if it's not too much */
//         if (LengthUsed > 0xFFF0)
//             return STATUS_NAME_TOO_LONG;
//
//         /* If FullPath is enough, use it */
//         if (FullPath->MaximumLength > LengthUsed)
//         {
//             /* Update remaining length if appropriate */
//             if (RemainingName->Length)
//             {
//                 RtlMoveMemory((PVOID)((ULONG_PTR)RemainingName->Buffer + TempLength),
//                               RemainingName->Buffer,
//                               RemainingName->Length);
//             }
//
//             /* And move the target object name */
//             RtlCopyMemory(RemainingName->Buffer,
//                           SymlinkObject->LinkTargetRemaining.Buffer,
//                           TempLength);
//
//             /* Finally update the full path with what we parsed */
//             FullPath->Length += SymlinkObject->LinkTargetRemaining.Length;
//             RemainingName->Length += SymlinkObject->LinkTargetRemaining.Length;
//             RemainingName->MaximumLength += RemainingName->Length + sizeof(WCHAR);
//             FullPath->Buffer[FullPath->Length / sizeof(WCHAR)] = UNICODE_NULL;
//
//             /* And reparse */
//             *NextObject = SymlinkObject->LinkTargetObject;
//             return STATUS_REPARSE_OBJECT;
//         }
//
//         /* FullPath is not enough, we'll have to reallocate */
//         MaximumLength = LengthUsed + sizeof(WCHAR);
//         NewTargetPath = ExAllocatePoolWithTag(NonPagedPool,
//                                               MaximumLength,
//                                               OB_NAME_TAG);
//         if (!NewTargetPath) return STATUS_INSUFFICIENT_RESOURCES;
//
//         /* Copy path begin */
//         RtlCopyMemory(NewTargetPath,
//                       FullPath->Buffer,
//                       sizeof(WCHAR) * (RemainingName->Buffer - FullPath->Buffer));
//
//         /* Copy path end (if any) */
//         if (RemainingName->Length)
//         {
//             RtlCopyMemory((PVOID)((ULONG_PTR)&NewTargetPath[RemainingName->Buffer - FullPath->Buffer] + TempLength),
//                           RemainingName->Buffer,
//                           RemainingName->Length);
//         }
//
//         /* And finish path with bound object */
//         RtlCopyMemory(&NewTargetPath[RemainingName->Buffer - FullPath->Buffer],
//                       SymlinkObject->LinkTargetRemaining.Buffer,
//                       TempLength);
//
//         /* Free old buffer */
//         ExFreePool(FullPath->Buffer);
//
//         /* Set new buffer in FullPath */
//         FullPath->Buffer = NewTargetPath;
//         FullPath->MaximumLength = MaximumLength;
//         FullPath->Length = LengthUsed;
//
//         /* Update remaining with what we handled */
//         RemainingName->Length = LengthUsed + (ULONG_PTR)NewTargetPath - (ULONG_PTR)&NewTargetPath[RemainingName->Buffer - FullPath->Buffer];
//         RemainingName->Buffer = &NewTargetPath[RemainingName->Buffer - FullPath->Buffer];
//         RemainingName->MaximumLength = RemainingName->Length + sizeof(WCHAR);
//
//         /* Reparse! */
//         *NextObject = SymlinkObject->LinkTargetObject;
//         return STATUS_REPARSE_OBJECT;
//     }
//
//     /* Set the target path and length */
//     TargetPath = &SymlinkObject->LinkTarget;
//     TempLength = TargetPath->Length;
//
//     /*
//      * Strip off the extra trailing '\', if we don't do this we will end up
//      * adding a extra '\' between TargetPath and RemainingName
//      * causing caller's like ObpLookupObjectName() to fail.
//      */
//     if (TempLength && RemainingName->Length)
//     {
//         /* The target and remaining names aren't empty, so check for slashes */
//         if ((TargetPath->Buffer[TempLength / sizeof(WCHAR) - 1] ==
//             OBJ_NAME_PATH_SEPARATOR) &&
//             (RemainingName->Buffer[0] == OBJ_NAME_PATH_SEPARATOR))
//         {
//             /* Reduce the length by one to cut off the extra '\' */
//             TempLength -= sizeof(OBJ_NAME_PATH_SEPARATOR);
//         }
//     }
//
//     /* Calculate the new length */
//     LengthUsed = TempLength + RemainingName->Length;
//
//     /* Check if it's not too much */
//     if (LengthUsed > 0xFFF0)
//         return STATUS_NAME_TOO_LONG;
//
//     /* Optimization: check if the new name is shorter */
//     if (FullPath->MaximumLength <= LengthUsed)
//     {
//         /* It's not, allocate a new one */
//         MaximumLength = LengthUsed + sizeof(WCHAR);
//         NewTargetPath = ExAllocatePoolWithTag(NonPagedPool,
//                                               MaximumLength,
//                                               OB_NAME_TAG);
//         if (!NewTargetPath) return STATUS_INSUFFICIENT_RESOURCES;
//     }
//     else
//     {
//         /* It is! Reuse the name... */
//         MaximumLength = FullPath->MaximumLength;
//         NewTargetPath = FullPath->Buffer;
//     }
//
//     /* Make sure we have a length */
//     if (RemainingName->Length)
//     {
//         /* Copy the new path */
//         RtlMoveMemory((PVOID)((ULONG_PTR)NewTargetPath + TempLength),
//                       RemainingName->Buffer,
//                       RemainingName->Length);
//     }
//
//     /* Copy the target path and null-terminate it */
//     RtlCopyMemory(NewTargetPath, TargetPath->Buffer, TempLength);
//     NewTargetPath[LengthUsed / sizeof(WCHAR)] = UNICODE_NULL;
//
//     /* If the optimization didn't work, free the old buffer */
//     if (NewTargetPath != FullPath->Buffer) ExFreePool(FullPath->Buffer);
//
//     /* Update the path values */
//     FullPath->Length = (USHORT)LengthUsed;
//     FullPath->MaximumLength = (USHORT)MaximumLength;
//     FullPath->Buffer = NewTargetPath;
//
//     /* Tell the parse routine to start reparsing */
//     return STATUS_REPARSE;
// }
//
// /* PUBLIC FUNCTIONS **********************************************************/
//
// /*++
// * @name NtCreateSymbolicLinkObject
// * @implemented NT4
// *
// *     The NtCreateSymbolicLinkObject opens or creates a symbolic link object.
// *
// * @param LinkHandle
// *        Variable which receives the symlink handle.
// *
// * @param DesiredAccess
// *        Desired access to the symlink.
// *
// * @param ObjectAttributes
// *        Structure describing the symlink.
// *
// * @param LinkTarget
// *        Unicode string defining the symlink's target
// *
// * @return STATUS_SUCCESS or appropriate error value.
// *
// * @remarks None.
// *
// *--*/
// NTSTATUS
// NTAPI
// NtCreateSymbolicLinkObject(OUT PHANDLE LinkHandle,
//                            IN ACCESS_MASK DesiredAccess,
//                            IN POBJECT_ATTRIBUTES ObjectAttributes,
//                            IN PUNICODE_STRING LinkTarget)
// {
//     HANDLE hLink;
//     POBJECT_SYMBOLIC_LINK SymbolicLink;
//     UNICODE_STRING CapturedLinkTarget;
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//     NTSTATUS Status;
//     PAGED_CODE();
//
//     /* Check if we need to probe parameters */
//     if (PreviousMode != KernelMode)
//     {
//         _SEH2_TRY
//         {
//             /* Probe the target */
//             CapturedLinkTarget = ProbeForReadUnicodeString(LinkTarget);
//             ProbeForRead(CapturedLinkTarget.Buffer,
//                          CapturedLinkTarget.MaximumLength,
//                          sizeof(WCHAR));
//
//             /* Probe the return handle */
//             ProbeForWriteHandle(LinkHandle);
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Return the exception code */
//             _SEH2_YIELD(return _SEH2_GetExceptionCode());
//         }
//         _SEH2_END;
//     }
//     else
//     {
//         /* No need to capture */
//         CapturedLinkTarget = *LinkTarget;
//     }
//
//     /* Check if the maximum length is odd */
//     if (CapturedLinkTarget.MaximumLength % sizeof(WCHAR))
//     {
//         /* Round it down */
//         CapturedLinkTarget.MaximumLength =
//             (USHORT)ALIGN_DOWN(CapturedLinkTarget.MaximumLength, WCHAR);
//     }
//
//     /* Fail if the length is odd, or if the maximum is smaller or 0 */
//     if ((CapturedLinkTarget.Length % sizeof(WCHAR)) ||
//         (CapturedLinkTarget.MaximumLength < CapturedLinkTarget.Length) ||
//         !(CapturedLinkTarget.MaximumLength))
//     {
//         /* This message is displayed on the debugger in Windows */
//         DbgPrint("OB: Invalid symbolic link target - %wZ\n",
//                  &CapturedLinkTarget);
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Create the object */
//     Status = ObCreateObject(PreviousMode,
//                             ObpSymbolicLinkObjectType,
//                             ObjectAttributes,
//                             PreviousMode,
//                             NULL,
//                             sizeof(OBJECT_SYMBOLIC_LINK),
//                             0,
//                             0,
//                             (PVOID*)&SymbolicLink);
//     if (NT_SUCCESS(Status))
//     {
//         /* Success! Fill in the creation time immediately */
//         KeQuerySystemTime(&SymbolicLink->CreationTime);
//
//         /* Setup the target name */
//         SymbolicLink->LinkTarget.Length = CapturedLinkTarget.Length;
//         SymbolicLink->LinkTarget.MaximumLength = CapturedLinkTarget.MaximumLength;
//         SymbolicLink->LinkTarget.Buffer =
//             ExAllocatePoolWithTag(PagedPool,
//                                   CapturedLinkTarget.MaximumLength,
//                                   TAG_SYMLINK_TARGET);
//         if (!SymbolicLink->LinkTarget.Buffer)
//         {
//             /* Dereference the symbolic link object and fail */
//             ObDereferenceObject(SymbolicLink);
//             return STATUS_NO_MEMORY;
//         }
//
//         /* Copy it */
//         _SEH2_TRY
//         {
//             RtlCopyMemory(SymbolicLink->LinkTarget.Buffer,
//                           CapturedLinkTarget.Buffer,
//                           CapturedLinkTarget.MaximumLength);
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             ObDereferenceObject(SymbolicLink);
//             _SEH2_YIELD(return _SEH2_GetExceptionCode());
//         }
//         _SEH2_END;
//
//         /* Initialize the remaining name, dos drive index and target object */
//         SymbolicLink->LinkTargetObject = NULL;
//         SymbolicLink->DosDeviceDriveIndex = 0;
//         RtlInitUnicodeString(&SymbolicLink->LinkTargetRemaining, NULL);
//
//         /* Insert it into the object tree */
//         Status = ObInsertObject(SymbolicLink,
//                                 NULL,
//                                 DesiredAccess,
//                                 0,
//                                 NULL,
//                                 &hLink);
//         if (NT_SUCCESS(Status))
//         {
//             _SEH2_TRY
//             {
//                 /* Return the handle to caller */
//                 *LinkHandle = hLink;
//             }
//             _SEH2_EXCEPT(ExSystemExceptionFilter())
//             {
//                 /* Get exception code */
//                 Status = _SEH2_GetExceptionCode();
//             }
//             _SEH2_END;
//         }
//     }
//
//     /* Return status to caller */
//     return Status;
// }
//
// /*++
// * @name NtOpenSymbolicLinkObject
// * @implemented NT4
// *
// *     The NtOpenSymbolicLinkObject opens a symbolic link object.
// *
// * @param LinkHandle
// *        Variable which receives the symlink handle.
// *
// * @param DesiredAccess
// *        Desired access to the symlink.
// *
// * @param ObjectAttributes
// *        Structure describing the symlink.
// *
// * @return STATUS_SUCCESS or appropriate error value.
// *
// * @remarks None.
// *
// *--*/
// NTSTATUS
// NTAPI
// NtOpenSymbolicLinkObject(OUT PHANDLE LinkHandle,
//                          IN ACCESS_MASK DesiredAccess,
//                          IN POBJECT_ATTRIBUTES ObjectAttributes)
// {
//     HANDLE hLink;
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//     NTSTATUS Status;
//     PAGED_CODE();
//
//     /* Check if we need to probe parameters */
//     if (PreviousMode != KernelMode)
//     {
//         _SEH2_TRY
//         {
//             /* Probe the return handle */
//             ProbeForWriteHandle(LinkHandle);
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Return the exception code */
//             _SEH2_YIELD(return _SEH2_GetExceptionCode());
//         }
//         _SEH2_END;
//     }
//
//     /* Open the object */
//     Status = ObOpenObjectByName(ObjectAttributes,
//                                 ObpSymbolicLinkObjectType,
//                                 PreviousMode,
//                                 NULL,
//                                 DesiredAccess,
//                                 NULL,
//                                 &hLink);
//
//     _SEH2_TRY
//     {
//         /* Return the handle to caller */
//         *LinkHandle = hLink;
//     }
//     _SEH2_EXCEPT(ExSystemExceptionFilter())
//     {
//         /* Get exception code */
//         Status = _SEH2_GetExceptionCode();
//     }
//     _SEH2_END;
//
//     /* Return status to caller */
//     return Status;
// }
//
// /*++
// * @name NtQuerySymbolicLinkObject
// * @implemented NT4
// *
// *     The NtQuerySymbolicLinkObject queries a symbolic link object.
// *
// * @param LinkHandle
// *        Symlink handle to query
// *
// * @param LinkTarget
// *        Unicode string defining the symlink's target
// *
// * @param ResultLength
// *        Caller supplied storage for the number of bytes written (or NULL).
// *
// * @return STATUS_SUCCESS or appropriate error value.
// *
// * @remarks None.
// *
// *--*/
// NTSTATUS
// NTAPI
// NtQuerySymbolicLinkObject(IN HANDLE LinkHandle,
//                           OUT PUNICODE_STRING LinkTarget,
//                           OUT PULONG ResultLength OPTIONAL)
// {
//     NTSTATUS Status;
//     UNICODE_STRING SafeLinkTarget = { 0, 0, NULL };
//     POBJECT_SYMBOLIC_LINK SymlinkObject;
//     POBJECT_HEADER ObjectHeader;
//     ULONG LengthUsed;
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//
//     PAGED_CODE();
//
//     if (PreviousMode != KernelMode)
//     {
//         _SEH2_TRY
//         {
//             /* Probe the unicode string for read and write */
//             ProbeForWriteUnicodeString(LinkTarget);
//
//             /* Probe the unicode string's buffer for write */
//             SafeLinkTarget = *LinkTarget;
//             ProbeForWrite(SafeLinkTarget.Buffer,
//                           SafeLinkTarget.MaximumLength,
//                           sizeof(WCHAR));
//
//             /* Probe the return length */
//             if (ResultLength) ProbeForWriteUlong(ResultLength);
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Return the exception code */
//             _SEH2_YIELD(return _SEH2_GetExceptionCode());
//         }
//         _SEH2_END;
//     }
//     else
//     {
//         /* No need to probe */
//         SafeLinkTarget = *LinkTarget;
//     }
//
//     /* Reference the object */
//     Status = ObReferenceObjectByHandle(LinkHandle,
//                                        SYMBOLIC_LINK_QUERY,
//                                        ObpSymbolicLinkObjectType,
//                                        PreviousMode,
//                                        (PVOID*)&SymlinkObject,
//                                        NULL);
//     if (NT_SUCCESS(Status))
//     {
//         /* Get the object header */
//         ObjectHeader = OBJECT_TO_OBJECT_HEADER(SymlinkObject);
//
//         /* Lock the object */
//         ObpAcquireObjectLock(ObjectHeader);
//
//         /*
//          * So here's the thing: If you specify a return length, then the
//          * implementation will use the maximum length. If you don't, then
//          * it will use the length.
//          */
//         LengthUsed = ResultLength ? SymlinkObject->LinkTarget.MaximumLength :
//                                     SymlinkObject->LinkTarget.Length;
//
//         /* Enter SEH so we can safely copy */
//         _SEH2_TRY
//         {
//             /* Make sure our buffer will fit */
//             if (LengthUsed <= SafeLinkTarget.MaximumLength)
//             {
//                 /* Copy the buffer */
//                 RtlCopyMemory(SafeLinkTarget.Buffer,
//                               SymlinkObject->LinkTarget.Buffer,
//                               LengthUsed);
//
//                 /* Copy the new length */
//                 LinkTarget->Length = SymlinkObject->LinkTarget.Length;
//             }
//             else
//             {
//                 /* Otherwise set the failure status */
//                 Status = STATUS_BUFFER_TOO_SMALL;
//             }
//
//             /* In both cases, check if the required length was requested */
//             if (ResultLength)
//             {
//                 /* Then return it */
//                 *ResultLength = SymlinkObject->LinkTarget.MaximumLength;
//             }
//         }
//         _SEH2_EXCEPT(ExSystemExceptionFilter())
//         {
//             /* Get the error code */
//             Status = _SEH2_GetExceptionCode();
//         }
//         _SEH2_END;
//
//         /* Unlock and dereference the object */
//         ObpReleaseObjectLock(ObjectHeader);
//         ObDereferenceObject(SymlinkObject);
//     }
//
//     /* Return query status */
//     return Status;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
