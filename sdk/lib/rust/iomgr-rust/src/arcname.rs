use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::inline::common::InitializeObjectAttributes;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

use ntkrnl_sys::ddk::{
    ANSI_STRING,
    ExFreePool,
    HalExamineMBR,
    _IO_STATUS_BLOCK,
    _OBJECT_ATTRIBUTES,
    _UNICODE_STRING,
    _EVENT_TYPE_NotificationEvent,
    _KWAIT_REASON_Executive,
    IoBuildDeviceIoControlRequest,
    IoBuildSynchronousFsdRequest,
    IoGetConfigurationInformation,
    IofCallDriver,
    IopFetchConfigurationInformation,
    IopStoreSystemPartitionInformation,
    IoReadPartitionTableEx,
    KEVENT,
    KeInitializeEvent,
    KeWaitForSingleObject,
    LARGE_INTEGER,
    PANSI_STRING,
    PARC_DISK_INFORMATION,
    PARC_DISK_SIGNATURE,
    PDEVICE_OBJECT,
    PDRIVE_LAYOUT_INFORMATION_EX,
    PFILE_OBJECT,
    GUID,
    PGUID,
    PIRP,
    PLOADER_PARAMETER_BLOCK,
    NtCreateSymbolicLinkObject,
    NtMakeTemporaryObject,
    NtOpenSymbolicLinkObject,
    NtQuerySymbolicLinkObject,
    ObCloseHandle,
    RtlCopyString,
    RtlEqualString,
    RtlFreeUnicodeString,
    RtlInitAnsiString,
    RtlInitUnicodeString,
    STORAGE_DEVICE_NUMBER,
    IoGetDeviceObjectPointer,
    IoCreateSymbolicLink,
};

// `extern "system"` maps to stdcall on i386 (with `@N` decoration) and
// cdecl on amd64/arm64. Both functions are NTAPI in `ntdll/rtl`.
extern "system" {
    fn RtlAnsiStringToUnicodeString(
        DestinationString: *mut _UNICODE_STRING,
        SourceString: PANSI_STRING,
        AllocateDestinationString: BOOLEAN,
    ) -> NTSTATUS;

    fn RtlUnicodeStringToAnsiString(
        DestinationString: *mut ANSI_STRING,
        SourceString: *mut _UNICODE_STRING,
        AllocateDestinationString: BOOLEAN,
    ) -> NTSTATUS;
}

extern "C" {
    fn strcmp(s1: *const i8, s2: *const i8) -> i32;
    fn strlen(s: *const i8) -> usize;
    fn strstr(haystack: *const i8, needle: *const i8) -> *mut i8;
    fn sprintf(buf: *mut i8, fmt: *const i8, ...) -> i32;
    fn wcslen(s: *const u16) -> usize;
}
#[inline(always)]
unsafe fn IoCallDriver(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    IofCallDriver(DeviceObject, Irp)
}
// IoAssignArcName is a macro: #define IoAssignArcName(ArcName, DeviceName) IoCreateSymbolicLink(ArcName, DeviceName)
#[inline(always)]
unsafe fn IoAssignArcName(ArcName: *mut _UNICODE_STRING, DeviceName: *mut _UNICODE_STRING) {
    IoCreateSymbolicLink(ArcName, DeviceName);
}
// C `IsEqualGUID` (guiddef.h) is not a real symbol — for non-C++ it's the macro
// `(!memcmp(rguid1, rguid2, sizeof(GUID)))`. The prior `extern "C"` decl demanded a
// symbol that exists nowhere; it only linked on amd64/arm64 because --gc-sections
// dead-stripped the caller, and broke on i386 where the reference is retained. Mirror
// the C macro inline so it resolves on every arch with no phantom external dependency.
#[inline(always)]
unsafe fn IsEqualGUID(guid1: PGUID, guid2: PGUID) -> BOOLEAN {
    let n = core::mem::size_of::<GUID>();
    let a = core::slice::from_raw_parts(guid1 as *const u8, n);
    let b = core::slice::from_raw_parts(guid2 as *const u8, n);
    (a == b) as BOOLEAN
}

#[no_mangle]
pub static mut IoArcHalDeviceName: _UNICODE_STRING = unsafe { core::mem::zeroed() };
#[no_mangle]
pub static mut IoArcBootDeviceName: _UNICODE_STRING = unsafe { core::mem::zeroed() };
#[no_mangle]
pub static mut IoLoaderArcBootDeviceName: *mut i8 = core::ptr::null_mut();

extern "C" {
    static mut IoRemoteBootClient: BOOLEAN;
}

const GUID_DEVINTERFACE_CDROM: GUID = GUID {
    Data1: 0x53f56308,
    Data2: 0xb6bf,
    Data3: 0x11d0,
    Data4: [0x94, 0xf2, 0x00, 0xa0, 0xc9, 0x1e, 0xfb, 0x8b],
};

const GUID_DEVINTERFACE_DISK: GUID = GUID {
    Data1: 0x53f56307,
    Data2: 0xb6bf,
    Data3: 0x11d0,
    Data4: [0x94, 0xf2, 0x00, 0xa0, 0xc9, 0x1e, 0xfb, 0x8b],
};

const TAG_IO: ULONG = 0x20206F49;
const FILE_READ_ATTRIBUTES: ULONG = 0x0080;
const IOCTL_STORAGE_GET_DEVICE_NUMBER: ULONG = 0x002D1080;
const IOCTL_DISK_GET_DRIVE_GEOMETRY: ULONG = 0x00070000;
const IRP_MJ_READ: ULONG = 0x03;
const PARTITION_STYLE_MBR: i32 = 0;
const PARTITION_STYLE_GPT: i32 = 1;
const SYMBOLIC_LINK_ALL_ACCESS: ULONG = 0x000F0001;
const ANSI_NULL: i8 = 0;
const UNICODE_NULL: u16 = 0;

// IopCreateArcNames + IopCreateArcNamesDisk are below.

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn IopCreateArcNamesCd(LoaderBlock: PLOADER_PARAMETER_BLOCK) -> NTSTATUS {
    let LoaderBlock = opaque_ptr(LoaderBlock);
    crate::dbg_print!(b"[CDARC] enter ArcBootDeviceName=%s\r\n\0", (*LoaderBlock).ArcBootDeviceName);
    let mut Irp: PIRP;
    let mut Event: KEVENT = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let mut NextEntry: PLIST_ENTRY;
    let mut FileObject: PFILE_OBJECT = core::ptr::null_mut();
    let mut DeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut StartingOffset: LARGE_INTEGER = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut PartitionBuffer: *mut ULONG = core::ptr::null_mut();
    let mut Buffer: [i8; 128] = [0; 128];
    let mut ArcBuffer: [i8; 128] = [0; 128];
    let mut NotEnabledPresent: BOOLEAN = 0u8;
    let mut DeviceNumber: STORAGE_DEVICE_NUMBER = core::mem::zeroed();
    let mut DeviceStringA: ANSI_STRING = core::mem::zeroed();
    let mut ArcNameStringA: ANSI_STRING = core::mem::zeroed();
    let mut SymbolicLinkList: *mut u16 = core::ptr::null_mut();
    let mut lSymbolicLinkList: *mut u16;
    let mut ArcDiskSignature: PARC_DISK_SIGNATURE = core::ptr::null_mut();
    let mut DeviceStringW: _UNICODE_STRING = core::mem::zeroed();
    let mut ArcNameStringW: _UNICODE_STRING = core::mem::zeroed();
    let mut DiskNumber: ULONG;
    let mut CdRomCount: ULONG;
    let mut CheckSum: ULONG;
    let mut i: ULONG;
    let mut EnabledDisks: ULONG = 0;
    let ArcDiskInformation: PARC_DISK_INFORMATION = (*LoaderBlock).ArcDiskInformation;

    CdRomCount = (*IoGetConfigurationInformation()).CdRomCount;

    Status = IopFetchConfigurationInformation(
        core::ptr::addr_of_mut!(SymbolicLinkList),
        GUID_DEVINTERFACE_CDROM,
        CdRomCount,
        core::ptr::addr_of_mut!(EnabledDisks),
    );
    crate::dbg_print!(b"[CDARC] fetch st=%x links=%p enabled=%d count=%d\r\n\0", Status, SymbolicLinkList, EnabledDisks, CdRomCount);
    if Status < 0 {
        NotEnabledPresent = 1u8;
    }
    lSymbolicLinkList = SymbolicLinkList;
    Status = STATUS_SUCCESS;

    NextEntry = (*ArcDiskInformation).DiskSignatureListHead.Flink;
    while NextEntry as PVOID != core::ptr::addr_of_mut!((*ArcDiskInformation).DiskSignatureListHead) as PVOID {
        ArcDiskSignature = NextEntry as PARC_DISK_SIGNATURE;

        if strcmp((*LoaderBlock).ArcBootDeviceName, (*ArcDiskSignature).ArcName) == 0 {
            break;
        }

        ArcDiskSignature = core::ptr::null_mut();
        NextEntry = (*NextEntry).Flink;
    }

    if ArcDiskSignature.is_null() {
        crate::dbg_print!(b"[CDARC] no matching ArcDiskSignature, returning\r\n\0");
        Status = STATUS_SUCCESS;
        core::mem::forget(Status);
        if !PartitionBuffer.is_null() {
            ExFreePoolWithTag(PartitionBuffer as PVOID, TAG_IO);
        }
        if !SymbolicLinkList.is_null() {
            ExFreePool(SymbolicLinkList as PVOID);
        }
        return Status;
    }
    crate::dbg_print!(b"[CDARC] matched signature, CdRomCount=%d EnabledDisks=%d\r\n\0", CdRomCount, EnabledDisks);

    PartitionBuffer = ExAllocatePoolWithTag(NonPagedPoolCacheAligned, 2048, TAG_IO) as *mut ULONG;
    if PartitionBuffer.is_null() {
        Status = STATUS_SUCCESS;
        core::mem::forget(Status);
        if !PartitionBuffer.is_null() {
            ExFreePoolWithTag(PartitionBuffer as PVOID, TAG_IO);
        }
        if !SymbolicLinkList.is_null() {
            ExFreePool(SymbolicLinkList as PVOID);
        }
        return Status;
    }

    if EnabledDisks > CdRomCount {
        CdRomCount = EnabledDisks;
    }

    if NotEnabledPresent != 0 && EnabledDisks == 0 {
        CdRomCount = CdRomCount.wrapping_add(5);
    }

    if CdRomCount == 0 {
        Status = STATUS_SUCCESS;
        core::mem::forget(Status);
        if !PartitionBuffer.is_null() {
            ExFreePoolWithTag(PartitionBuffer as PVOID, TAG_IO);
        }
        if !SymbolicLinkList.is_null() {
            ExFreePool(SymbolicLinkList as PVOID);
        }
        return Status;
    }

    DiskNumber = 0;
    EnabledDisks = 0;
    while DiskNumber < CdRomCount {
        if !lSymbolicLinkList.is_null() && *lSymbolicLinkList != UNICODE_NULL {
            RtlInitUnicodeString(core::ptr::addr_of_mut!(DeviceStringW), lSymbolicLinkList);
            lSymbolicLinkList = lSymbolicLinkList.add(wcslen(lSymbolicLinkList) + 1);

            Status = IoGetDeviceObjectPointer(
                core::ptr::addr_of_mut!(DeviceStringW),
                FILE_READ_ATTRIBUTES,
                core::ptr::addr_of_mut!(FileObject),
                core::ptr::addr_of_mut!(DeviceObject),
            );
            crate::dbg_print!(b"[CDARC] enabled disk %d getdev st=%x f=%p dev=%p\r\n\0", DiskNumber, Status, FileObject, DeviceObject);
            if Status < 0 {
                sprintf(
                    Buffer.as_mut_ptr(),
                    b"\\Device\\CdRom%lu\0".as_ptr() as *const i8,
                    DiskNumber,
                );
                RtlInitAnsiString(core::ptr::addr_of_mut!(DeviceStringA), Buffer.as_ptr());
                Status = RtlAnsiStringToUnicodeString(core::ptr::addr_of_mut!(DeviceStringW), core::ptr::addr_of_mut!(DeviceStringA), 1u8);
                if Status < 0 {
                    core::mem::forget(Status);
                    if !PartitionBuffer.is_null() {
                        ExFreePoolWithTag(PartitionBuffer as PVOID, TAG_IO);
                    }
                    if !SymbolicLinkList.is_null() {
                        ExFreePool(SymbolicLinkList as PVOID);
                    }
                    return Status;
                }

                Status = IoGetDeviceObjectPointer(
                    core::ptr::addr_of_mut!(DeviceStringW),
                    FILE_READ_ATTRIBUTES,
                    core::ptr::addr_of_mut!(FileObject),
                    core::ptr::addr_of_mut!(DeviceObject),
                );
                crate::dbg_print!(b"[CDARC] enabled disk %d legacy getdev st=%x f=%p dev=%p\r\n\0", DiskNumber, Status, FileObject, DeviceObject);
                if Status < 0 {
                    RtlFreeUnicodeString(core::ptr::addr_of_mut!(DeviceStringW));
                    core::mem::forget(Status);
                    if !PartitionBuffer.is_null() {
                        ExFreePoolWithTag(PartitionBuffer as PVOID, TAG_IO);
                    }
                    if !SymbolicLinkList.is_null() {
                        ExFreePool(SymbolicLinkList as PVOID);
                    }
                    return Status;
                }
            } else {
                Irp = IoBuildDeviceIoControlRequest(
                    IOCTL_STORAGE_GET_DEVICE_NUMBER,
                    DeviceObject,
                    core::ptr::null_mut(),
                    0,
                    core::ptr::addr_of_mut!(DeviceNumber) as PVOID,
                    core::mem::size_of::<STORAGE_DEVICE_NUMBER>() as ULONG,
                    0u8,
                    core::ptr::addr_of_mut!(Event),
                    core::ptr::addr_of_mut!(IoStatusBlock),
                );
                if Irp.is_null() {
                    ObDereferenceObject(FileObject as PVOID);
                    Status = STATUS_INSUFFICIENT_RESOURCES;
                    core::mem::forget(Status);
                    if !PartitionBuffer.is_null() {
                        ExFreePoolWithTag(PartitionBuffer as PVOID, TAG_IO);
                    }
                    if !SymbolicLinkList.is_null() {
                        ExFreePool(SymbolicLinkList as PVOID);
                    }
                    return Status;
                }

                KeInitializeEvent(core::ptr::addr_of_mut!(Event), _EVENT_TYPE_NotificationEvent, 0u8);
                Status = IoCallDriver(DeviceObject, Irp);
                if Status == STATUS_PENDING {
                    KeWaitForSingleObject(
                        core::ptr::addr_of_mut!(Event) as PVOID,
                        _KWAIT_REASON_Executive,
                        0i8,
                        0u8,
                        core::ptr::null_mut(),
                    );
                    Status = IoStatusBlock.__bindgen_anon_1.Status;
                }
                crate::dbg_print!(b"[CDARC] enabled disk %d getnum st=%x iosb=%x devnum=%d\r\n\0", DiskNumber, Status, IoStatusBlock.__bindgen_anon_1.Status, DeviceNumber.DeviceNumber);
                if Status < 0 {
                    ObDereferenceObject(FileObject as PVOID);
                    core::mem::forget(Status);
                    if !PartitionBuffer.is_null() {
                        ExFreePoolWithTag(PartitionBuffer as PVOID, TAG_IO);
                    }
                    if !SymbolicLinkList.is_null() {
                        ExFreePool(SymbolicLinkList as PVOID);
                    }
                    return Status;
                }

                sprintf(
                    Buffer.as_mut_ptr(),
                    b"\\Device\\CdRom%lu\0".as_ptr() as *const i8,
                    DeviceNumber.DeviceNumber,
                );
                RtlInitAnsiString(core::ptr::addr_of_mut!(DeviceStringA), Buffer.as_ptr());
                Status = RtlAnsiStringToUnicodeString(core::ptr::addr_of_mut!(DeviceStringW), core::ptr::addr_of_mut!(DeviceStringA), 1u8);
                if Status < 0 {
                    ObDereferenceObject(FileObject as PVOID);
                    core::mem::forget(Status);
                    if !PartitionBuffer.is_null() {
                        ExFreePoolWithTag(PartitionBuffer as PVOID, TAG_IO);
                    }
                    if !SymbolicLinkList.is_null() {
                        ExFreePool(SymbolicLinkList as PVOID);
                    }
                    return Status;
                }
            }
        } else {
            sprintf(
                Buffer.as_mut_ptr(),
                b"\\Device\\CdRom%lu\0".as_ptr() as *const i8,
                EnabledDisks,
            );
            EnabledDisks = EnabledDisks.wrapping_add(1);
            RtlInitAnsiString(core::ptr::addr_of_mut!(DeviceStringA), Buffer.as_ptr());
            Status = RtlAnsiStringToUnicodeString(core::ptr::addr_of_mut!(DeviceStringW), core::ptr::addr_of_mut!(DeviceStringA), 1u8);
            if Status < 0 {
                core::mem::forget(Status);
                if !PartitionBuffer.is_null() {
                    ExFreePoolWithTag(PartitionBuffer as PVOID, TAG_IO);
                }
                if !SymbolicLinkList.is_null() {
                    ExFreePool(SymbolicLinkList as PVOID);
                }
                return Status;
            }

            Status = IoGetDeviceObjectPointer(
                core::ptr::addr_of_mut!(DeviceStringW),
                FILE_READ_ATTRIBUTES,
                core::ptr::addr_of_mut!(FileObject),
                core::ptr::addr_of_mut!(DeviceObject),
            );
            if Status < 0 {
                RtlFreeUnicodeString(core::ptr::addr_of_mut!(DeviceStringW));
                crate::dbg_print!(b"[CDARC] fallback disk %d getdev st=%x\r\n\0", DiskNumber, Status);
                core::mem::forget(Status);
                if !PartitionBuffer.is_null() {
                    ExFreePoolWithTag(PartitionBuffer as PVOID, TAG_IO);
                }
                if !SymbolicLinkList.is_null() {
                    ExFreePool(SymbolicLinkList as PVOID);
                }
                // No CD-ROM device at all: the CD image may be on a disk-class device.
                return IopAssignCdArcNameFromDisk(LoaderBlock, (*ArcDiskSignature).CheckSum);
            }
        }

        StartingOffset.QuadPart = 0x8000;
        CheckSum = 0;
        Irp = IoBuildSynchronousFsdRequest(
            IRP_MJ_READ,
            DeviceObject,
            PartitionBuffer as PVOID,
            2048,
            core::ptr::addr_of_mut!(StartingOffset),
            core::ptr::addr_of_mut!(Event),
            core::ptr::addr_of_mut!(IoStatusBlock),
        );
        if !Irp.is_null() {
            KeInitializeEvent(core::ptr::addr_of_mut!(Event), _EVENT_TYPE_NotificationEvent, 0u8);
            Status = IoCallDriver(DeviceObject, Irp);
            if Status == STATUS_PENDING {
                KeWaitForSingleObject(
                    core::ptr::addr_of_mut!(Event) as PVOID,
                    _KWAIT_REASON_Executive,
                    0i8,
                    0u8,
                    core::ptr::null_mut(),
                );
                Status = IoStatusBlock.__bindgen_anon_1.Status;
            }

            crate::dbg_print!(b"[CDARC] disk %d read st=%x iosb=%x irp=%p\r\n\0", DiskNumber, Status, IoStatusBlock.__bindgen_anon_1.Status, Irp);
            if Status >= 0 {
                i = 0;
                while i < 2048 / core::mem::size_of::<ULONG>() as u32 {
                    CheckSum = CheckSum.wrapping_add(*PartitionBuffer.add(i as usize));
                    i = i.wrapping_add(1);
                }
            }
        }

        ObDereferenceObject(FileObject as PVOID);

        crate::dbg_print!(b"[CDARC] disk %d checksum=%x expected=%x\r\n\0", DiskNumber, CheckSum, (*ArcDiskSignature).CheckSum);
        if CheckSum.wrapping_add((*ArcDiskSignature).CheckSum) == 0
            || strcmp((*LoaderBlock).ArcBootDeviceName, (*ArcDiskSignature).ArcName) == 0
        {
            crate::dbg_print!(b"[CDARC] checksum/loader match, creating ArcName\r\n\0");
            sprintf(
                ArcBuffer.as_mut_ptr(),
                b"\\ArcName\\%s\0".as_ptr() as *const i8,
                (*LoaderBlock).ArcBootDeviceName,
            );
            RtlInitAnsiString(core::ptr::addr_of_mut!(ArcNameStringA), ArcBuffer.as_ptr());
            Status = RtlAnsiStringToUnicodeString(core::ptr::addr_of_mut!(ArcNameStringW), core::ptr::addr_of_mut!(ArcNameStringA), 1u8);
            if Status >= 0 {
                crate::dbg_print!(b"[CDARC] IoAssignArcName(%s) -> ...\r\n\0", ArcBuffer.as_ptr());
                IoAssignArcName(core::ptr::addr_of_mut!(ArcNameStringW), core::ptr::addr_of_mut!(DeviceStringW));
                RtlFreeUnicodeString(core::ptr::addr_of_mut!(ArcNameStringW));
            }

            RtlFreeUnicodeString(core::ptr::addr_of_mut!(DeviceStringW));
            core::mem::forget(Status);
            if !PartitionBuffer.is_null() {
                ExFreePoolWithTag(PartitionBuffer as PVOID, TAG_IO);
            }
            if !SymbolicLinkList.is_null() {
                ExFreePool(SymbolicLinkList as PVOID);
            }
            return Status;
        }

        RtlFreeUnicodeString(core::ptr::addr_of_mut!(DeviceStringW));
        DiskNumber = DiskNumber.wrapping_add(1);
    }

    if !PartitionBuffer.is_null() {
        ExFreePoolWithTag(PartitionBuffer as PVOID, TAG_IO);
    }
    if !SymbolicLinkList.is_null() {
        ExFreePool(SymbolicLinkList as PVOID);
    }
    // No CD-ROM matched the boot CD's checksum: look for it on a disk-class device.
    IopAssignCdArcNameFromDisk(LoaderBlock, (*ArcDiskSignature).CheckSum)
}

/// The boot CD's ArcName when the image is not on a CD-ROM device at all.
///
/// freeldr boots `cdrom(N)` whenever the medium carries ISO9660, whatever the device
/// (UefiMediumIsIso9660): an ISO served over usb-storage or virtio-blk is a DISK to the
/// kernel, so the CD scan above finds no `\Device\CdRom*` and the boot dies at 0x6B with no
/// `\ArcName\<boot>`. This walks `\Device\Harddisk<n>\Partition0`, checksums 2048 bytes at
/// 0x8000 exactly as the CD path does, and binds the boot ArcName to the disk that matches.
/// cdfs mounts ISO9660 on disks through its `\CdfsHdd` object. A divergence from the C,
/// which only ever looked at CD-ROM devices.
unsafe fn IopAssignCdArcNameFromDisk(LoaderBlock: PLOADER_PARAMETER_BLOCK, Expected: ULONG) -> NTSTATUS {
    let DiskCount = (*IoGetConfigurationInformation()).DiskCount;
    let Buffer = ExAllocatePoolWithTag(NonPagedPoolCacheAligned, 2048, TAG_IO) as *mut ULONG;
    if Buffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    let mut Result: NTSTATUS = STATUS_SUCCESS;
    for Disk in 0..DiskCount {
        let mut Name: [i8; 64] = [0; 64];
        sprintf(Name.as_mut_ptr(), b"\\Device\\Harddisk%lu\\Partition0\0".as_ptr() as *const i8, Disk);
        let mut NameA: ANSI_STRING = core::mem::zeroed();
        let mut NameW: _UNICODE_STRING = core::mem::zeroed();
        RtlInitAnsiString(core::ptr::addr_of_mut!(NameA), Name.as_ptr());
        if RtlAnsiStringToUnicodeString(core::ptr::addr_of_mut!(NameW), core::ptr::addr_of_mut!(NameA), 1u8) < 0 {
            continue;
        }
        let mut FileObject: PFILE_OBJECT = core::ptr::null_mut();
        let mut DeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
        let mut Status = IoGetDeviceObjectPointer(
            core::ptr::addr_of_mut!(NameW),
            FILE_READ_ATTRIBUTES,
            core::ptr::addr_of_mut!(FileObject),
            core::ptr::addr_of_mut!(DeviceObject),
        );
        if Status < 0 {
            RtlFreeUnicodeString(core::ptr::addr_of_mut!(NameW));
            continue;
        }
        let mut Event: KEVENT = core::mem::zeroed();
        let mut Iosb: _IO_STATUS_BLOCK = core::mem::zeroed();
        let mut Offset: LARGE_INTEGER = core::mem::zeroed();
        Offset.QuadPart = 0x8000;
        KeInitializeEvent(core::ptr::addr_of_mut!(Event), _EVENT_TYPE_NotificationEvent, 0u8);
        let Irp = IoBuildSynchronousFsdRequest(
            IRP_MJ_READ,
            DeviceObject,
            Buffer as PVOID,
            2048,
            core::ptr::addr_of_mut!(Offset),
            core::ptr::addr_of_mut!(Event),
            core::ptr::addr_of_mut!(Iosb),
        );
        let mut CheckSum: ULONG = 0;
        if !Irp.is_null() {
            Status = IoCallDriver(DeviceObject, Irp);
            if Status == STATUS_PENDING {
                KeWaitForSingleObject(
                    core::ptr::addr_of_mut!(Event) as PVOID,
                    _KWAIT_REASON_Executive,
                    0i8,
                    0u8,
                    core::ptr::null_mut(),
                );
                Status = Iosb.__bindgen_anon_1.Status;
            }
            if Status >= 0 {
                for i in 0..(2048 / core::mem::size_of::<ULONG>()) {
                    CheckSum = CheckSum.wrapping_add(*Buffer.add(i));
                }
            }
        }
        let Matched = Status >= 0 && CheckSum.wrapping_add(Expected) == 0;
        if Matched {
            // disk.sys marks partition zero VPB_RAW_MOUNT unless the registry says otherwise,
            // so only RAW may mount there -- right for a super-floppy, whose filesystem mounts
            // once through partition 1. An MBR-less ISO has no partition 1: its ISO9660 volume
            // IS partition zero, and with the flag cdfs never got the mount (0x7B, c0000001).
            // Lift it on this one device, the boot medium just identified, and nowhere else.
            let Real = (*FileObject).DeviceObject;
            if !Real.is_null() && !(*Real).Vpb.is_null() {
                (*(*Real).Vpb).Flags &= !0x0010; // VPB_RAW_MOUNT
            }
        }
        ObDereferenceObject(FileObject as PVOID);
        crate::dbg_print!(
            b"[CDARC] disk-class %s read st=%x checksum=%x expected=%x\r\n\0",
            Name.as_ptr(),
            Status,
            CheckSum,
            Expected,
        );
        if Matched {
            let mut Arc: [i8; 128] = [0; 128];
            sprintf(Arc.as_mut_ptr(), b"\\ArcName\\%s\0".as_ptr() as *const i8, (*LoaderBlock).ArcBootDeviceName);
            let mut ArcA: ANSI_STRING = core::mem::zeroed();
            let mut ArcW: _UNICODE_STRING = core::mem::zeroed();
            RtlInitAnsiString(core::ptr::addr_of_mut!(ArcA), Arc.as_ptr());
            Result = RtlAnsiStringToUnicodeString(core::ptr::addr_of_mut!(ArcW), core::ptr::addr_of_mut!(ArcA), 1u8);
            if Result >= 0 {
                crate::dbg_print!(b"[CDARC] boot CD found on %s -> %s\r\n\0", Name.as_ptr(), Arc.as_ptr());
                IoAssignArcName(core::ptr::addr_of_mut!(ArcW), core::ptr::addr_of_mut!(NameW));
                RtlFreeUnicodeString(core::ptr::addr_of_mut!(ArcW));
            }
            RtlFreeUnicodeString(core::ptr::addr_of_mut!(NameW));
            break;
        }
        RtlFreeUnicodeString(core::ptr::addr_of_mut!(NameW));
    }
    ExFreePoolWithTag(Buffer as PVOID, TAG_IO);
    Result
}

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn IopCreateArcNames(LoaderBlock: PLOADER_PARAMETER_BLOCK) -> NTSTATUS {
    let LoaderBlock = opaque_ptr(LoaderBlock);
    let mut Length: usize;
    let mut Status: NTSTATUS;
    let mut Buffer: [i8; 128] = [0; 128];
    let mut SingleDisk: BOOLEAN;
    let mut FoundBoot: BOOLEAN = 0u8;
    let mut SystemDevice: _UNICODE_STRING = core::mem::zeroed();
    let mut LoaderPathNameW: _UNICODE_STRING = core::mem::zeroed();
    let mut BootDeviceName: _UNICODE_STRING = core::mem::zeroed();
    let mut ArcSystemString: ANSI_STRING = core::mem::zeroed();
    let mut ArcString: ANSI_STRING = core::mem::zeroed();
    let mut LanmanRedirector: ANSI_STRING = core::mem::zeroed();
    let mut LoaderPathNameA: ANSI_STRING = core::mem::zeroed();
    let ArcDiskInfo: PARC_DISK_INFORMATION = (*LoaderBlock).ArcDiskInformation;

    // Check if we only have one disk on the machine
    SingleDisk = ((*(*ArcDiskInfo).DiskSignatureListHead.Flink).Flink
        == core::ptr::addr_of_mut!((*ArcDiskInfo).DiskSignatureListHead) as *mut _) as BOOLEAN;

    // Create the global HAL partition name
    sprintf(
        Buffer.as_mut_ptr(),
        b"\\ArcName\\%s\0".as_ptr() as *const i8,
        (*LoaderBlock).ArcHalDeviceName,
    );
    RtlInitAnsiString(core::ptr::addr_of_mut!(ArcString), Buffer.as_ptr());
    Status = RtlAnsiStringToUnicodeString(
        core::ptr::addr_of_mut!(IoArcHalDeviceName),
        core::ptr::addr_of_mut!(ArcString),
        1u8,
    );
    if Status < 0 {
        return Status;
    }

    // Create the global system partition name
    sprintf(
        Buffer.as_mut_ptr(),
        b"\\ArcName\\%s\0".as_ptr() as *const i8,
        (*LoaderBlock).ArcBootDeviceName,
    );
    RtlInitAnsiString(core::ptr::addr_of_mut!(ArcString), Buffer.as_ptr());
    Status = RtlAnsiStringToUnicodeString(
        core::ptr::addr_of_mut!(IoArcBootDeviceName),
        core::ptr::addr_of_mut!(ArcString),
        1u8,
    );
    if Status < 0 {
        return Status;
    }

    // Allocate memory for the loader ARC boot device name copy
    Length = strlen((*LoaderBlock).ArcBootDeviceName) + 1;
    IoLoaderArcBootDeviceName =
        ExAllocatePoolWithTag(PagedPool, Length, TAG_IO) as *mut i8;
    if !IoLoaderArcBootDeviceName.is_null() {
        core::ptr::copy_nonoverlapping(
            (*LoaderBlock).ArcBootDeviceName as *const u8,
            IoLoaderArcBootDeviceName as *mut u8,
            Length,
        );
    }

    // Check if we only found a disk, but we're booting from CD-ROM
    if SingleDisk != 0
        && !strstr(
            (*LoaderBlock).ArcBootDeviceName,
            b"cdrom\0".as_ptr() as *const i8,
        )
        .is_null()
    {
        SingleDisk = 0u8;
    }

    // Build the boot strings
    RtlInitAnsiString(
        core::ptr::addr_of_mut!(ArcSystemString),
        (*LoaderBlock).ArcHalDeviceName,
    );

    // If we are doing remote booting
    if IoRemoteBootClient != 0 {
        FoundBoot = 1u8;

        RtlInitAnsiString(
            core::ptr::addr_of_mut!(LanmanRedirector),
            b"\\Device\\LanmanRedirector\0".as_ptr() as *const i8,
        );
        Status = RtlAnsiStringToUnicodeString(
            core::ptr::addr_of_mut!(SystemDevice),
            core::ptr::addr_of_mut!(LanmanRedirector),
            1u8,
        );
        if Status < 0 {
            return Status;
        }

        sprintf(
            Buffer.as_mut_ptr(),
            b"\\ArcName\\%s\0".as_ptr() as *const i8,
            (*LoaderBlock).ArcBootDeviceName,
        );
        RtlInitAnsiString(core::ptr::addr_of_mut!(ArcString), Buffer.as_ptr());
        Status = RtlAnsiStringToUnicodeString(
            core::ptr::addr_of_mut!(BootDeviceName),
            core::ptr::addr_of_mut!(ArcString),
            1u8,
        );
        if Status >= 0 {
            IoAssignArcName(
                core::ptr::addr_of_mut!(BootDeviceName),
                core::ptr::addr_of_mut!(SystemDevice),
            );
            RtlFreeUnicodeString(core::ptr::addr_of_mut!(BootDeviceName));

            RtlInitAnsiString(
                core::ptr::addr_of_mut!(LoaderPathNameA),
                (*LoaderBlock).NtHalPathName,
            );
            Status = RtlAnsiStringToUnicodeString(
                core::ptr::addr_of_mut!(LoaderPathNameW),
                core::ptr::addr_of_mut!(LoaderPathNameA),
                1u8,
            );
            if Status < 0 {
                RtlFreeUnicodeString(core::ptr::addr_of_mut!(SystemDevice));
                return Status;
            }

            IopStoreSystemPartitionInformation(
                core::ptr::addr_of_mut!(SystemDevice),
                core::ptr::addr_of_mut!(LoaderPathNameW),
            );
        }

        RtlFreeUnicodeString(core::ptr::addr_of_mut!(SystemDevice));

        // Don't quit even if everything went fine — IopCreateArcNamesDisk
        // still needs to map devices with symlinks. It returns success if
        // mapping went fine even if it didn't find the boot device, and
        // it won't reset boot-device-finding status either.
    }

    // Loop every disk and try to find boot disk
    Status = IopCreateArcNamesDisk(
        LoaderBlock,
        SingleDisk,
        core::ptr::addr_of_mut!(FoundBoot),
    );
    // If it succeeded but we didn't find boot device, try to browse CDs
    if Status >= 0 && FoundBoot == 0 {
        Status = IopCreateArcNamesCd(LoaderBlock);
    }

    Status
}

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn IopCreateArcNamesDisk(
    LoaderBlock: PLOADER_PARAMETER_BLOCK,
    SingleDisk: BOOLEAN,
    FoundBoot: *mut BOOLEAN,
) -> NTSTATUS {
    let LoaderBlock = opaque_ptr(LoaderBlock);
    let FoundBoot = opaque_ptr(FoundBoot);
    let mut Irp: PIRP;
    let mut Data: PVOID = core::ptr::null_mut();
    let mut Event: KEVENT = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let mut NextEntry: ntkrnl_sys::base::PLIST_ENTRY;
    let mut FileObject: PFILE_OBJECT = core::ptr::null_mut();
    let mut DiskGeometry: ntkrnl_sys::ddk::DISK_GEOMETRY = core::mem::zeroed();
    let mut DeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut StartingOffset: LARGE_INTEGER = core::mem::zeroed();
    let mut PartitionBuffer: *mut ULONG = core::ptr::null_mut();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Buffer: [i8; 128] = [0; 128];
    let mut ArcBuffer: [i8; 128] = [0; 128];
    let mut NotEnabledPresent: BOOLEAN = 0u8;
    let mut DeviceNumber: STORAGE_DEVICE_NUMBER = core::mem::zeroed();
    let mut ArcDiskSignature: PARC_DISK_SIGNATURE;
    let mut SymbolicLinkList: *mut u16 = core::ptr::null_mut();
    let mut lSymbolicLinkList: *mut u16;
    let mut DriveLayout: PDRIVE_LAYOUT_INFORMATION_EX = core::ptr::null_mut();
    let mut DeviceStringW: _UNICODE_STRING = core::mem::zeroed();
    let mut ArcNameStringW: _UNICODE_STRING = core::mem::zeroed();
    let mut HalPathStringW: _UNICODE_STRING = core::mem::zeroed();
    let mut DiskNumber: ULONG;
    let mut DiskCount: ULONG;
    let mut CheckSum: ULONG;
    let mut i: ULONG;
    let mut Signature: ULONG = 0;
    let mut EnabledDisks: ULONG = 0;
    let ArcDiskInformation: PARC_DISK_INFORMATION = (*LoaderBlock).ArcDiskInformation;
    let mut ArcBootString: ANSI_STRING = core::mem::zeroed();
    let mut ArcSystemString: ANSI_STRING = core::mem::zeroed();
    let mut DeviceStringA: ANSI_STRING = core::mem::zeroed();
    let mut ArcNameStringA: ANSI_STRING = core::mem::zeroed();
    let mut HalPathStringA: ANSI_STRING = core::mem::zeroed();

    DeviceNumber.DeviceNumber = ULONG::MAX;
    DiskCount = (*IoGetConfigurationInformation()).DiskCount;

    Status = IopFetchConfigurationInformation(
        core::ptr::addr_of_mut!(SymbolicLinkList),
        GUID_DEVINTERFACE_DISK,
        DiskCount,
        core::ptr::addr_of_mut!(EnabledDisks),
    );
    if Status < 0 {
        NotEnabledPresent = 1u8;
    }

    lSymbolicLinkList = SymbolicLinkList;

    RtlInitAnsiString(
        core::ptr::addr_of_mut!(ArcBootString),
        (*LoaderBlock).ArcBootDeviceName,
    );
    RtlInitAnsiString(
        core::ptr::addr_of_mut!(ArcSystemString),
        (*LoaderBlock).ArcHalDeviceName,
    );

    if EnabledDisks > DiskCount {
        DiskCount = EnabledDisks;
    }

    if NotEnabledPresent != 0 && EnabledDisks == 0 {
        DiskCount = DiskCount.wrapping_add(20);
    }

    if DiskCount == 0 {
        // Cleanup path
        if !DriveLayout.is_null() {
            ExFreePool(DriveLayout as PVOID);
        }
        if !SymbolicLinkList.is_null() {
            ExFreePool(SymbolicLinkList as PVOID);
        }
        return STATUS_SUCCESS;
    }

    // Browse all disks
    DiskNumber = 0;
    'disk_loop: while DiskNumber < DiskCount {
        // ASSERT(DriveLayout == NULL); — invariant from cleanup-after-each-iteration
        if !lSymbolicLinkList.is_null() && *lSymbolicLinkList != UNICODE_NULL {
            // Build device name from first symbolic link
            RtlInitUnicodeString(core::ptr::addr_of_mut!(DeviceStringW), lSymbolicLinkList);
            // Advance symlink list pointer past this entry's NUL
            lSymbolicLinkList = lSymbolicLinkList.add(wcslen(lSymbolicLinkList) + 1);

            // Get its device + file object
            Status = IoGetDeviceObjectPointer(
                core::ptr::addr_of_mut!(DeviceStringW),
                FILE_READ_ATTRIBUTES,
                core::ptr::addr_of_mut!(FileObject),
                core::ptr::addr_of_mut!(DeviceObject),
            );
            if Status >= 0 {
                // Ask the device for its device number
                Irp = IoBuildDeviceIoControlRequest(
                    IOCTL_STORAGE_GET_DEVICE_NUMBER,
                    DeviceObject,
                    core::ptr::null_mut(),
                    0,
                    core::ptr::addr_of_mut!(DeviceNumber) as PVOID,
                    core::mem::size_of::<STORAGE_DEVICE_NUMBER>() as ULONG,
                    0u8,
                    core::ptr::addr_of_mut!(Event),
                    core::ptr::addr_of_mut!(IoStatusBlock),
                );
                if Irp.is_null() {
                    ObDereferenceObject(FileObject as PVOID);
                    Status = STATUS_INSUFFICIENT_RESOURCES;
                    break 'disk_loop;
                }

                KeInitializeEvent(
                    core::ptr::addr_of_mut!(Event),
                    _EVENT_TYPE_NotificationEvent,
                    0u8,
                );
                Status = IoCallDriver(DeviceObject, Irp);
                if Status == STATUS_PENDING {
                    KeWaitForSingleObject(
                        core::ptr::addr_of_mut!(Event) as PVOID,
                        _KWAIT_REASON_Executive,
                        0i8,
                        0u8,
                        core::ptr::null_mut(),
                    );
                    Status = IoStatusBlock.__bindgen_anon_1.Status;
                }

                if Status < 0 {
                    ObDereferenceObject(FileObject as PVOID);
                    DiskNumber = DiskNumber.wrapping_add(1);
                    continue;
                }
            }

            // End of enabled-disks enumeration
            if NotEnabledPresent != 0 && *lSymbolicLinkList == UNICODE_NULL {
                if DeviceNumber.DeviceNumber == ULONG::MAX {
                    DeviceNumber.DeviceNumber = 0;
                }
                if DeviceNumber.DeviceNumber > DiskNumber {
                    DiskNumber = DeviceNumber.DeviceNumber;
                }
                DiskCount = DiskNumber.wrapping_add(20);
            }
        } else {
            // Build device name "\Device\Harddisk%lu\Partition0"
            sprintf(
                Buffer.as_mut_ptr(),
                b"\\Device\\Harddisk%lu\\Partition0\0".as_ptr() as *const i8,
                DiskNumber,
            );
            RtlInitAnsiString(core::ptr::addr_of_mut!(DeviceStringA), Buffer.as_ptr());
            Status = RtlAnsiStringToUnicodeString(
                core::ptr::addr_of_mut!(DeviceStringW),
                core::ptr::addr_of_mut!(DeviceStringA),
                1u8,
            );
            if Status < 0 {
                break 'disk_loop;
            }

            Status = IoGetDeviceObjectPointer(
                core::ptr::addr_of_mut!(DeviceStringW),
                FILE_READ_ATTRIBUTES,
                core::ptr::addr_of_mut!(FileObject),
                core::ptr::addr_of_mut!(DeviceObject),
            );

            RtlFreeUnicodeString(core::ptr::addr_of_mut!(DeviceStringW));
            // Force DiskNumber-based naming for the rest of this iteration
            DeviceNumber.DeviceNumber = ULONG::MAX;
        }

        if Status < 0 {
            DiskNumber = DiskNumber.wrapping_add(1);
            continue;
        }

        // Ask disk for geometry
        Irp = IoBuildDeviceIoControlRequest(
            IOCTL_DISK_GET_DRIVE_GEOMETRY,
            DeviceObject,
            core::ptr::null_mut(),
            0,
            core::ptr::addr_of_mut!(DiskGeometry) as PVOID,
            core::mem::size_of::<ntkrnl_sys::ddk::DISK_GEOMETRY>() as ULONG,
            0u8,
            core::ptr::addr_of_mut!(Event),
            core::ptr::addr_of_mut!(IoStatusBlock),
        );
        if Irp.is_null() {
            ObDereferenceObject(FileObject as PVOID);
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'disk_loop;
        }

        KeInitializeEvent(
            core::ptr::addr_of_mut!(Event),
            _EVENT_TYPE_NotificationEvent,
            0u8,
        );
        Status = IoCallDriver(DeviceObject, Irp);
        if Status == STATUS_PENDING {
            KeWaitForSingleObject(
                core::ptr::addr_of_mut!(Event) as PVOID,
                _KWAIT_REASON_Executive,
                0i8,
                0u8,
                core::ptr::null_mut(),
            );
            Status = IoStatusBlock.__bindgen_anon_1.Status;
        }
        if Status < 0 {
            ObDereferenceObject(FileObject as PVOID);
            DiskNumber = DiskNumber.wrapping_add(1);
            continue;
        }

        // Read the partition table
        Status = IoReadPartitionTableEx(DeviceObject, core::ptr::addr_of_mut!(DriveLayout));
        if Status < 0 {
            ObDereferenceObject(FileObject as PVOID);
            DiskNumber = DiskNumber.wrapping_add(1);
            continue;
        }

        // Ensure at least 512 bytes per sector
        if DiskGeometry.BytesPerSector < 512 {
            DiskGeometry.BytesPerSector = 512;
        }

        // Check MBR type against EZ Drive type
        StartingOffset.QuadPart = 0;
        Data = core::ptr::null_mut();
        HalExamineMBR(
            DeviceObject,
            DiskGeometry.BytesPerSector,
            0x55,
            core::ptr::addr_of_mut!(Data),
        );
        if !Data.is_null() {
            // EZ Drive — start reading after it
            StartingOffset.QuadPart = DiskGeometry.BytesPerSector as i64;
            ExFreePool(Data);
        }

        // Allocate buffer for checksum read
        PartitionBuffer = ExAllocatePoolWithTag(
            NonPagedPoolCacheAligned,
            DiskGeometry.BytesPerSector as usize,
            TAG_IO,
        ) as *mut ULONG;
        if PartitionBuffer.is_null() {
            ObDereferenceObject(FileObject as PVOID);
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'disk_loop;
        }

        // Read first sector
        Irp = IoBuildSynchronousFsdRequest(
            IRP_MJ_READ,
            DeviceObject,
            PartitionBuffer as PVOID,
            DiskGeometry.BytesPerSector,
            core::ptr::addr_of_mut!(StartingOffset),
            core::ptr::addr_of_mut!(Event),
            core::ptr::addr_of_mut!(IoStatusBlock),
        );
        if Irp.is_null() {
            ExFreePoolWithTag(PartitionBuffer as PVOID, TAG_IO);
            PartitionBuffer = core::ptr::null_mut();
            ObDereferenceObject(FileObject as PVOID);
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'disk_loop;
        }

        KeInitializeEvent(
            core::ptr::addr_of_mut!(Event),
            _EVENT_TYPE_NotificationEvent,
            0u8,
        );
        Status = IoCallDriver(DeviceObject, Irp);
        if Status == STATUS_PENDING {
            KeWaitForSingleObject(
                core::ptr::addr_of_mut!(Event) as PVOID,
                _KWAIT_REASON_Executive,
                0i8,
                0u8,
                core::ptr::null_mut(),
            );
            Status = IoStatusBlock.__bindgen_anon_1.Status;
        }

        // Compute checksum if read succeeded
        CheckSum = 0;
        if Status >= 0 {
            i = 0;
            while i < (512 / core::mem::size_of::<ULONG>() as ULONG) {
                CheckSum = CheckSum.wrapping_add(*PartitionBuffer.add(i as usize));
                i = i.wrapping_add(1);
            }
        }

        ExFreePoolWithTag(PartitionBuffer as PVOID, TAG_IO);
        PartitionBuffer = core::ptr::null_mut();
        ObDereferenceObject(FileObject as PVOID);

        if Status < 0 {
            ExFreePool(DriveLayout as PVOID);
            DriveLayout = core::ptr::null_mut();
            DiskNumber = DiskNumber.wrapping_add(1);
            continue;
        }

        // Browse each ARC disk signature
        NextEntry = (*ArcDiskInformation).DiskSignatureListHead.Flink as ntkrnl_sys::base::PLIST_ENTRY;
        while NextEntry as PVOID
            != core::ptr::addr_of_mut!((*ArcDiskInformation).DiskSignatureListHead) as PVOID
        {
            // ARC_DISK_SIGNATURE.ListEntry is the first field; cast directly
            ArcDiskSignature = NextEntry as PARC_DISK_SIGNATURE;

            let single_mbr = SingleDisk != 0
                && DiskCount == 1
                && (*DriveLayout).PartitionStyle == PARTITION_STYLE_MBR as ULONG;
            let signature_match = IopVerifyDiskSignature(
                DriveLayout,
                ArcDiskSignature,
                core::ptr::addr_of_mut!(Signature),
            ) != 0
                && (*ArcDiskSignature).CheckSum.wrapping_add(CheckSum) == 0;

            if single_mbr || signature_match {
                let RealDiskNumber = if DeviceNumber.DeviceNumber != ULONG::MAX {
                    DeviceNumber.DeviceNumber
                } else {
                    DiskNumber
                };

                // Whole-disk device + ArcName
                sprintf(
                    Buffer.as_mut_ptr(),
                    b"\\Device\\Harddisk%lu\\Partition0\0".as_ptr() as *const i8,
                    RealDiskNumber,
                );
                RtlInitAnsiString(core::ptr::addr_of_mut!(DeviceStringA), Buffer.as_ptr());
                Status = RtlAnsiStringToUnicodeString(
                    core::ptr::addr_of_mut!(DeviceStringW),
                    core::ptr::addr_of_mut!(DeviceStringA),
                    1u8,
                );
                if Status < 0 {
                    break 'disk_loop;
                }

                sprintf(
                    ArcBuffer.as_mut_ptr(),
                    b"\\ArcName\\%s\0".as_ptr() as *const i8,
                    (*ArcDiskSignature).ArcName,
                );
                RtlInitAnsiString(core::ptr::addr_of_mut!(ArcNameStringA), ArcBuffer.as_ptr());
                Status = RtlAnsiStringToUnicodeString(
                    core::ptr::addr_of_mut!(ArcNameStringW),
                    core::ptr::addr_of_mut!(ArcNameStringA),
                    1u8,
                );
                if Status < 0 {
                    RtlFreeUnicodeString(core::ptr::addr_of_mut!(DeviceStringW));
                    break 'disk_loop;
                }

                IoAssignArcName(
                    core::ptr::addr_of_mut!(ArcNameStringW),
                    core::ptr::addr_of_mut!(DeviceStringW),
                );
                RtlFreeUnicodeString(core::ptr::addr_of_mut!(ArcNameStringW));
                RtlFreeUnicodeString(core::ptr::addr_of_mut!(DeviceStringW));

                // Per-partition device + ArcName
                i = 1;
                while i <= (*DriveLayout).PartitionCount {
                    sprintf(
                        Buffer.as_mut_ptr(),
                        b"\\Device\\Harddisk%lu\\Partition%lu\0".as_ptr() as *const i8,
                        RealDiskNumber,
                        i,
                    );
                    RtlInitAnsiString(core::ptr::addr_of_mut!(DeviceStringA), Buffer.as_ptr());
                    Status = RtlAnsiStringToUnicodeString(
                        core::ptr::addr_of_mut!(DeviceStringW),
                        core::ptr::addr_of_mut!(DeviceStringA),
                        1u8,
                    );
                    if Status < 0 {
                        break 'disk_loop;
                    }

                    // Partial ARC name (no \ArcName\ prefix) — used for boot/system match
                    sprintf(
                        ArcBuffer.as_mut_ptr(),
                        b"%spartition(%lu)\0".as_ptr() as *const i8,
                        (*ArcDiskSignature).ArcName,
                        i,
                    );
                    RtlInitAnsiString(core::ptr::addr_of_mut!(ArcNameStringA), ArcBuffer.as_ptr());

                    if RtlEqualString(
                        core::ptr::addr_of_mut!(ArcNameStringA),
                        core::ptr::addr_of_mut!(ArcBootString),
                        1u8,
                    ) != 0
                    {
                        *FoundBoot = 1u8;
                    }

                    if RtlEqualString(
                        core::ptr::addr_of_mut!(ArcNameStringA),
                        core::ptr::addr_of_mut!(ArcSystemString),
                        1u8,
                    ) != 0
                    {
                        RtlInitAnsiString(
                            core::ptr::addr_of_mut!(HalPathStringA),
                            (*LoaderBlock).NtHalPathName,
                        );
                        Status = RtlAnsiStringToUnicodeString(
                            core::ptr::addr_of_mut!(HalPathStringW),
                            core::ptr::addr_of_mut!(HalPathStringA),
                            1u8,
                        );
                        if Status < 0 {
                            RtlFreeUnicodeString(core::ptr::addr_of_mut!(DeviceStringW));
                            break 'disk_loop;
                        }

                        IopStoreSystemPartitionInformation(
                            core::ptr::addr_of_mut!(DeviceStringW),
                            core::ptr::addr_of_mut!(HalPathStringW),
                        );
                        RtlFreeUnicodeString(core::ptr::addr_of_mut!(HalPathStringW));
                    }

                    // Full ARC name with prefix
                    sprintf(
                        ArcBuffer.as_mut_ptr(),
                        b"\\ArcName\\%spartition(%lu)\0".as_ptr() as *const i8,
                        (*ArcDiskSignature).ArcName,
                        i,
                    );
                    RtlInitAnsiString(core::ptr::addr_of_mut!(ArcNameStringA), ArcBuffer.as_ptr());
                    Status = RtlAnsiStringToUnicodeString(
                        core::ptr::addr_of_mut!(ArcNameStringW),
                        core::ptr::addr_of_mut!(ArcNameStringA),
                        1u8,
                    );
                    if Status < 0 {
                        RtlFreeUnicodeString(core::ptr::addr_of_mut!(DeviceStringW));
                        break 'disk_loop;
                    }

                    IoAssignArcName(
                        core::ptr::addr_of_mut!(ArcNameStringW),
                        core::ptr::addr_of_mut!(DeviceStringW),
                    );
                    RtlFreeUnicodeString(core::ptr::addr_of_mut!(ArcNameStringW));
                    RtlFreeUnicodeString(core::ptr::addr_of_mut!(DeviceStringW));

                    i = i.wrapping_add(1);
                }
            }
            // else: duplicate signature / virus warning omitted (debug-only)

            NextEntry = (*NextEntry).Flink as ntkrnl_sys::base::PLIST_ENTRY;
        }

        // Release drive layout before next disk
        ExFreePool(DriveLayout as PVOID);
        DriveLayout = core::ptr::null_mut();

        DiskNumber = DiskNumber.wrapping_add(1);
    }

    Status = STATUS_SUCCESS;

    if !DriveLayout.is_null() {
        ExFreePool(DriveLayout as PVOID);
    }
    if !SymbolicLinkList.is_null() {
        ExFreePool(SymbolicLinkList as PVOID);
    }

    Status
}

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn IopReassignSystemRoot(
    LoaderBlock: PLOADER_PARAMETER_BLOCK,
    NtBootPath: PANSI_STRING,
) -> NTSTATUS {
    let LoaderBlock = opaque_ptr(LoaderBlock);
    let NtBootPath = opaque_ptr(NtBootPath);
    crate::dbg_print!(b"[REASSIGN] enter, ArcBootDeviceName=%s NtBootPathName=%s\r\n\0",
        (*LoaderBlock).ArcBootDeviceName, (*LoaderBlock).NtBootPathName);
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let mut Buffer: [i8; 256] = core::mem::zeroed();
    let mut AnsiBuffer: [i8; 256] = core::mem::zeroed();
    let mut ArcNameBuffer: [u16; 64] = core::mem::zeroed();
    let mut TargetString: ANSI_STRING = core::mem::zeroed();
    let mut ArcString: ANSI_STRING = core::mem::zeroed();
    let mut TempString: ANSI_STRING = core::mem::zeroed();
    let mut LinkName: _UNICODE_STRING = core::mem::zeroed();
    let mut TargetName: _UNICODE_STRING = core::mem::zeroed();
    let mut ArcName: _UNICODE_STRING = core::mem::zeroed();
    let mut LinkHandle: HANDLE = core::ptr::null_mut();

    // Create the Unicode name for the current ARC boot device
    sprintf(
        Buffer.as_mut_ptr(),
        b"\\ArcName\\%s\0".as_ptr() as *const i8,
        (*LoaderBlock).ArcBootDeviceName,
    );
    RtlInitAnsiString(&mut TargetString, Buffer.as_ptr());
    Status = RtlAnsiStringToUnicodeString(&mut TargetName, &mut TargetString, TRUE as BOOLEAN);
    if !NT_SUCCESS(Status) {
        return FALSE as NTSTATUS;
    }

    // Initialize the attributes and open the link
    InitializeObjectAttributes(
        &mut ObjectAttributes,
        &mut TargetName,
        OBJ_CASE_INSENSITIVE,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    crate::dbg_print!(b"[REASSIGN] NtOpenSymbolicLinkObject(\\ArcName\\<boot>) ...\r\n\0");
    Status = NtOpenSymbolicLinkObject(
        &mut LinkHandle,
        SYMBOLIC_LINK_ALL_ACCESS,
        &mut ObjectAttributes,
    );
    crate::dbg_print!(b"[REASSIGN] open ArcName status=%x handle=%p\r\n\0", Status, LinkHandle);
    if !NT_SUCCESS(Status) {
        // We failed, free the string
        RtlFreeUnicodeString(&mut TargetName);
        return FALSE as NTSTATUS;
    }

    // Query the current \SystemRoot
    ArcName.Buffer = ArcNameBuffer.as_mut_ptr();
    ArcName.Length = 0;
    ArcName.MaximumLength = core::mem::size_of_val(&ArcNameBuffer) as u16;
    Status = NtQuerySymbolicLinkObject(LinkHandle, &mut ArcName, core::ptr::null_mut());
    crate::dbg_print!(b"[REASSIGN] query ArcName status=%x len=%d\r\n\0", Status, ArcName.Length as u32);
    if !NT_SUCCESS(Status) {
        // We failed, free the string
        RtlFreeUnicodeString(&mut TargetName);
        return FALSE as NTSTATUS;
    }

    // Convert it to Ansi
    ArcString.Buffer = AnsiBuffer.as_mut_ptr();
    ArcString.Length = 0;
    ArcString.MaximumLength = core::mem::size_of_val(&AnsiBuffer) as u16;
    Status = RtlUnicodeStringToAnsiString(&mut ArcString, &mut ArcName, FALSE as BOOLEAN);
    AnsiBuffer[ArcString.Length as usize] = 0; // ANSI_NULL

    // Close the link handle and free the name
    ObCloseHandle(LinkHandle, 0); // KernelMode = 0
    RtlFreeUnicodeString(&mut TargetName);

    // Setup the system root name again
    RtlInitAnsiString(&mut TempString, b"\\SystemRoot\0".as_ptr() as *const i8);
    Status = RtlAnsiStringToUnicodeString(&mut LinkName, &mut TempString, TRUE as BOOLEAN);
    if !NT_SUCCESS(Status) {
        return FALSE as NTSTATUS;
    }

    // Open the symbolic link for it
    InitializeObjectAttributes(
        &mut ObjectAttributes,
        &mut LinkName,
        OBJ_CASE_INSENSITIVE,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    Status = NtOpenSymbolicLinkObject(
        &mut LinkHandle,
        SYMBOLIC_LINK_ALL_ACCESS,
        &mut ObjectAttributes,
    );
    if !NT_SUCCESS(Status) {
        return FALSE as NTSTATUS;
    }

    // Destroy it
    NtMakeTemporaryObject(LinkHandle);
    ObCloseHandle(LinkHandle, 0); // KernelMode = 0

    // Now create the new name for it
    sprintf(
        Buffer.as_mut_ptr(),
        b"%s%s\0".as_ptr() as *const i8,
        ArcString.Buffer,
        (*LoaderBlock).NtBootPathName,
    );

    // Copy it into the passed parameter and null-terminate it
    RtlCopyString(NtBootPath as *mut _, &ArcString as *const ANSI_STRING as *const _);
    let buf_len = strlen(Buffer.as_ptr());
    Buffer[buf_len - 1] = 0; // ANSI_NULL

    // Setup the Unicode-name for the new symbolic link value
    RtlInitAnsiString(&mut TargetString, Buffer.as_ptr());
    crate::dbg_print!(b"[REASSIGN] new \\SystemRoot target = %s\r\n\0", Buffer.as_ptr());
    InitializeObjectAttributes(
        &mut ObjectAttributes,
        &mut LinkName,
        OBJ_CASE_INSENSITIVE | OBJ_PERMANENT,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    Status = RtlAnsiStringToUnicodeString(&mut ArcName, &mut TargetString, TRUE as BOOLEAN);
    if !NT_SUCCESS(Status) {
        return FALSE as NTSTATUS;
    }

    // Create it
    Status = NtCreateSymbolicLinkObject(
        &mut LinkHandle,
        SYMBOLIC_LINK_ALL_ACCESS,
        &mut ObjectAttributes,
        &mut ArcName,
    );
    crate::dbg_print!(b"[REASSIGN] NtCreateSymbolicLinkObject status=%x handle=%p\r\n\0", Status, LinkHandle);

    // Free all the strings and close the handle and return success
    RtlFreeUnicodeString(&mut ArcName);
    RtlFreeUnicodeString(&mut LinkName);
    ObCloseHandle(LinkHandle, 0); // KernelMode = 0
    TRUE as NTSTATUS
}

#[no_mangle]
pub unsafe extern "C" fn IopVerifyDiskSignature(
    DriveLayout: PDRIVE_LAYOUT_INFORMATION_EX,
    ArcDiskSignature: PARC_DISK_SIGNATURE,
    Signature: *mut ULONG,
) -> BOOLEAN {
    let Signature = opaque_ptr(Signature);
    // Fail if the partition table is invalid
    if (*ArcDiskSignature).ValidPartitionTable == 0 {
        return FALSE as BOOLEAN;
    }

    // If the partition style is MBR
    if (*DriveLayout).PartitionStyle == PARTITION_STYLE_MBR as ULONG {
        // Check the MBR signature
        if (*DriveLayout).__bindgen_anon_1.Mbr.Signature == (*ArcDiskSignature).Signature {
            // And return it
            if !Signature.is_null() {
                *Signature = (*DriveLayout).__bindgen_anon_1.Mbr.Signature;
            }
            return TRUE as BOOLEAN;
        }
    }
    // If the partition style is GPT
    else if (*DriveLayout).PartitionStyle == PARTITION_STYLE_GPT as ULONG {
        // Verify whether the signature is GPT and compare the GUID
        if (*ArcDiskSignature).IsGpt != 0
            && IsEqualGUID(
                (*ArcDiskSignature).GptSignature.as_ptr() as PGUID,
                &(*DriveLayout).__bindgen_anon_1.Gpt.DiskId as *const GUID as PGUID,
            ) != 0
        {
            // There is no signature to return, just zero it
            if !Signature.is_null() {
                *Signature = 0;
            }
            return TRUE as BOOLEAN;
        }
    }

    // If we get there, something went wrong, so fail
    FALSE as BOOLEAN
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
// * PROJECT:         ReactOS Kernel
// * LICENSE:         GPL - See COPYING in the top level directory
// * FILE:            ntoskrnl/io/iomgr/arcname.c
// * PURPOSE:         ARC Path Initialization Functions
// * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
// *                  Eric Kohl
// *                  Pierre Schweitzer (pierre.schweitzer@reactos.org)
// */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* GLOBALS *******************************************************************/
//
// _UNICODE_STRING IoArcHalDeviceName, IoArcBootDeviceName;
// PCHAR IoLoaderArcBootDeviceName;
//
// /* FUNCTIONS *****************************************************************/
//
// CODE_SEG("INIT")
// NTSTATUS
// NTAPI
// IopCreateArcNamesCd(IN PLOADER_PARAMETER_BLOCK LoaderBlock);
//
// CODE_SEG("INIT")
// NTSTATUS
// NTAPI
// IopCreateArcNamesDisk(IN PLOADER_PARAMETER_BLOCK LoaderBlock,
//                       IN BOOLEAN SingleDisk,
//                       OUT PBOOLEAN FoundBoot);
//
// CODE_SEG("INIT")
// NTSTATUS
// NTAPI
// IopCreateArcNames(IN PLOADER_PARAMETER_BLOCK LoaderBlock)
// {
//     SIZE_T Length;
//     NTSTATUS Status;
//     CHAR Buffer[128];
//     BOOLEAN SingleDisk;
//     BOOLEAN FoundBoot = FALSE;
//     _UNICODE_STRING SystemDevice, LoaderPathNameW, BootDeviceName;
//     PARC_DISK_INFORMATION ArcDiskInfo = LoaderBlock->ArcDiskInformation;
//     ANSI_STRING ArcSystemString, ArcString, LanmanRedirector, LoaderPathNameA;
//
//     /* Check if we only have one disk on the machine */
//     SingleDisk = (ArcDiskInfo->DiskSignatureListHead.Flink->Flink ==
//                  &ArcDiskInfo->DiskSignatureListHead);
//
//     /* Create the global HAL partition name */
//     sprintf(Buffer, "\\ArcName\\%s", LoaderBlock->ArcHalDeviceName);
//     RtlInitAnsiString(&ArcString, Buffer);
//     Status = RtlAnsiStringToUnicodeString(&IoArcHalDeviceName, &ArcString, TRUE);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     /* Create the global system partition name */
//     sprintf(Buffer, "\\ArcName\\%s", LoaderBlock->ArcBootDeviceName);
//     RtlInitAnsiString(&ArcString, Buffer);
//     Status = RtlAnsiStringToUnicodeString(&IoArcBootDeviceName, &ArcString, TRUE);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     /* Allocate memory for the string */
//     Length = strlen(LoaderBlock->ArcBootDeviceName) + sizeof(ANSI_NULL);
//     IoLoaderArcBootDeviceName = ExAllocatePoolWithTag(PagedPool,
//                                                       Length,
//                                                       TAG_IO);
//     if (IoLoaderArcBootDeviceName)
//     {
//         /* Copy the name */
//         RtlCopyMemory(IoLoaderArcBootDeviceName,
//                       LoaderBlock->ArcBootDeviceName,
//                       Length);
//     }
//
//     /* Check if we only found a disk, but we're booting from CD-ROM */
//     if ((SingleDisk) && strstr(LoaderBlock->ArcBootDeviceName, "cdrom"))
//     {
//         /* Then disable single-disk mode, since there's a CD drive out there */
//         SingleDisk = FALSE;
//     }
//
//     /* Build the boot strings */
//     RtlInitAnsiString(&ArcSystemString, LoaderBlock->ArcHalDeviceName);
//
//     /* If we are doing remote booting */
//     if (IoRemoteBootClient)
//     {
//         /* Yes, we have found boot device */
//         FoundBoot = TRUE;
//
//         /* Get NT device name */
//         RtlInitAnsiString(&LanmanRedirector, "\\Device\\LanmanRedirector");
//         Status = RtlAnsiStringToUnicodeString(&SystemDevice, &LanmanRedirector, TRUE);
//         if (!NT_SUCCESS(Status))
//         {
//             return Status;
//         }
//
//         /* Get ARC booting device name (in net(0) something) */
//         sprintf(Buffer, "\\ArcName\\%s", LoaderBlock->ArcBootDeviceName);
//         RtlInitAnsiString(&ArcString, Buffer);
//         Status = RtlAnsiStringToUnicodeString(&BootDeviceName, &ArcString, TRUE);
//         if (NT_SUCCESS(Status))
//         {
//             /* Map ARC to NT name */
//             IoAssignArcName(&BootDeviceName, &SystemDevice);
//             RtlFreeUnicodeString(&BootDeviceName);
//
//             /* Now, get loader path name */
//             RtlInitAnsiString(&LoaderPathNameA, LoaderBlock->NtHalPathName);
//             Status = RtlAnsiStringToUnicodeString(&LoaderPathNameW, &LoaderPathNameA, TRUE);
//             if (!NT_SUCCESS(Status))
//             {
//                 RtlFreeUnicodeString(&SystemDevice);
//                 return Status;
//             }
//
//             /* And set it has system partition */
//             IopStoreSystemPartitionInformation(&SystemDevice, &LoaderPathNameW);
//         }
//
//         RtlFreeUnicodeString(&SystemDevice);
//
//         /* Don't quit here, even if everything went fine!
//          * We need IopCreateArcNamesDisk to properly map
//          * devices with symlinks.
//          * It will return success if the mapping process went fine
//          * even if it didn't find boot device.
//          * It won't reset boot device finding status as well.
//          */
//     }
//
//     /* Loop every disk and try to find boot disk */
//     Status = IopCreateArcNamesDisk(LoaderBlock, SingleDisk, &FoundBoot);
//     /* If it succeeded but we didn't find boot device, try to browse Cds */
//     if (NT_SUCCESS(Status) && !FoundBoot)
//     {
//         Status = IopCreateArcNamesCd(LoaderBlock);
//     }
//
//     /* Return success */
//     return Status;
// }
//
// CODE_SEG("INIT")
// NTSTATUS
// NTAPI
// IopCreateArcNamesCd(IN PLOADER_PARAMETER_BLOCK LoaderBlock)
// {
//     PIRP Irp;
//     KEVENT Event;
//     NTSTATUS Status;
//     PLIST_ENTRY NextEntry;
//     PFILE_OBJECT FileObject;
//     PDEVICE_OBJECT DeviceObject;
//     LARGE_INTEGER StartingOffset;
//     IO_STATUS_BLOCK IoStatusBlock;
//     PULONG PartitionBuffer = NULL;
//     CHAR Buffer[128], ArcBuffer[128];
//     BOOLEAN NotEnabledPresent = FALSE;
//     STORAGE_DEVICE_NUMBER DeviceNumber;
//     ANSI_STRING DeviceStringA, ArcNameStringA;
//     PWSTR SymbolicLinkList, lSymbolicLinkList;
//     PARC_DISK_SIGNATURE ArcDiskSignature = NULL;
//     _UNICODE_STRING DeviceStringW, ArcNameStringW;
//     ULONG DiskNumber, CdRomCount, CheckSum, i, EnabledDisks = 0;
//     PARC_DISK_INFORMATION ArcDiskInformation = LoaderBlock->ArcDiskInformation;
//
//     /* Get all the Cds present in the system */
//     CdRomCount = IoGetConfigurationInformation()->CdRomCount;
//
//     /* Get enabled Cds and check if result matches
//      * For the record, enabled Cds (or even disk) are Cds/disks
//      * that have been successfully handled by MountMgr driver
//      * and that already own their device name. This is the "new" way
//      * to handle them, that came with NT5.
//      * Currently, Windows 2003 provides an ARC names creation based
//      * on both enabled drives and not enabled drives (lack from
//      * the driver).
//      * Given the current ReactOS state, that's good for us.
//      * To sum up, this is NOT a hack or whatsoever.
//      */
//     Status = IopFetchConfigurationInformation(&SymbolicLinkList,
//                                               GUID_DEVINTERFACE_CDROM,
//                                               CdRomCount,
//                                               &EnabledDisks);
//     if (!NT_SUCCESS(Status))
//     {
//         NotEnabledPresent = TRUE;
//     }
//     /* Save symbolic link list address in order to free it after */
//     lSymbolicLinkList = SymbolicLinkList;
//     /* For the moment, we won't fail */
//     Status = STATUS_SUCCESS;
//
//     /* Browse all the ARC devices trying to find the one matching boot device */
//     for (NextEntry = ArcDiskInformation->DiskSignatureListHead.Flink;
//          NextEntry != &ArcDiskInformation->DiskSignatureListHead;
//          NextEntry = NextEntry->Flink)
//     {
//         ArcDiskSignature = CONTAINING_RECORD(NextEntry,
//                                              ARC_DISK_SIGNATURE,
//                                              ListEntry);
//
//         if (strcmp(LoaderBlock->ArcBootDeviceName, ArcDiskSignature->ArcName) == 0)
//         {
//             break;
//         }
//
//         ArcDiskSignature = NULL;
//     }
//
//     /* Not found... Not booting from a Cd */
//     if (!ArcDiskSignature)
//     {
//         DPRINT("Failed finding a cd that could match current boot device\n");
//         goto Cleanup;
//     }
//
//     /* Allocate needed space for reading Cd */
//     PartitionBuffer = ExAllocatePoolWithTag(NonPagedPoolCacheAligned, 2048, TAG_IO);
//     if (!PartitionBuffer)
//     {
//         DPRINT("Failed allocating resources!\n");
//         /* Here, we fail, BUT we return success, some Microsoft joke */
//         goto Cleanup;
//     }
//
//     /* If we have more enabled Cds, take that into account */
//     if (EnabledDisks > CdRomCount)
//     {
//         CdRomCount = EnabledDisks;
//     }
//
//     /* If we'll have to browse for none enabled Cds, fix higher count */
//     if (NotEnabledPresent && !EnabledDisks)
//     {
//         CdRomCount += 5;
//     }
//
//     /* Finally, if in spite of all that work, we still don't have Cds, leave */
//     if (!CdRomCount)
//     {
//         goto Cleanup;
//     }
//
//     /* Start browsing Cds */
//     for (DiskNumber = 0, EnabledDisks = 0; DiskNumber < CdRomCount; DiskNumber++)
//     {
//         /* Check if we have an enabled disk */
//         if (lSymbolicLinkList && *lSymbolicLinkList != UNICODE_NULL)
//         {
//             /* Create its device name using first symbolic link */
//             RtlInitUnicodeString(&DeviceStringW, lSymbolicLinkList);
//             /* Then, update symbolic links list */
//             lSymbolicLinkList += wcslen(lSymbolicLinkList) + (sizeof(UNICODE_NULL) / sizeof(WCHAR));
//
//             /* Get its associated device object and file object */
//             Status = IoGetDeviceObjectPointer(&DeviceStringW,
//                                               FILE_READ_ATTRIBUTES,
//                                               &FileObject,
//                                               &DeviceObject);
//             /* Failure? Good bye! */
//             if (!NT_SUCCESS(Status))
//             {
//                 goto Cleanup;
//             }
//
//             /* Now, we'll ask the device its device number */
//             Irp = IoBuildDeviceIoControlRequest(IOCTL_STORAGE_GET_DEVICE_NUMBER,
//                                                 DeviceObject,
//                                                 NULL,
//                                                 0,
//                                                 &DeviceNumber,
//                                                 sizeof(DeviceNumber),
//                                                 FALSE,
//                                                 &Event,
//                                                 &IoStatusBlock);
//             /* Failure? Good bye! */
//             if (!Irp)
//             {
//                 /* Dereference file object before leaving */
//                 ObDereferenceObject(FileObject);
//                 Status = STATUS_INSUFFICIENT_RESOURCES;
//                 goto Cleanup;
//             }
//
//             /* Call the driver, and wait for it if needed */
//             KeInitializeEvent(&Event, NotificationEvent, FALSE);
//             Status = IoCallDriver(DeviceObject, Irp);
//             if (Status == STATUS_PENDING)
//             {
//                 KeWaitForSingleObject(&Event, Executive, KernelMode, FALSE, NULL);
//                 Status = IoStatusBlock.Status;
//             }
//             if (!NT_SUCCESS(Status))
//             {
//                 ObDereferenceObject(FileObject);
//                 goto Cleanup;
//             }
//
//             /* Finally, build proper device name */
//             sprintf(Buffer, "\\Device\\CdRom%lu", DeviceNumber.DeviceNumber);
//             RtlInitAnsiString(&DeviceStringA, Buffer);
//             Status = RtlAnsiStringToUnicodeString(&DeviceStringW, &DeviceStringA, TRUE);
//             if (!NT_SUCCESS(Status))
//             {
//                 ObDereferenceObject(FileObject);
//                 goto Cleanup;
//             }
//         }
//         else
//         {
//             /* Create device name for the cd */
//             sprintf(Buffer, "\\Device\\CdRom%lu", EnabledDisks++);
//             RtlInitAnsiString(&DeviceStringA, Buffer);
//             Status = RtlAnsiStringToUnicodeString(&DeviceStringW, &DeviceStringA, TRUE);
//             if (!NT_SUCCESS(Status))
//             {
//                 goto Cleanup;
//             }
//
//             /* Get its device object */
//             Status = IoGetDeviceObjectPointer(&DeviceStringW,
//                                               FILE_READ_ATTRIBUTES,
//                                               &FileObject,
//                                               &DeviceObject);
//             if (!NT_SUCCESS(Status))
//             {
//                 RtlFreeUnicodeString(&DeviceStringW);
//                 goto Cleanup;
//             }
//         }
//
//         /* Initiate data for reading cd and compute checksum */
//         StartingOffset.QuadPart = 0x8000;
//         CheckSum = 0;
//         Irp = IoBuildSynchronousFsdRequest(IRP_MJ_READ,
//                                            DeviceObject,
//                                            PartitionBuffer,
//                                            2048,
//                                            &StartingOffset,
//                                            &Event,
//                                            &IoStatusBlock);
//         if (Irp)
//         {
//             /* Call the driver, and wait for it if needed */
//             KeInitializeEvent(&Event, NotificationEvent, FALSE);
//             Status = IoCallDriver(DeviceObject, Irp);
//             if (Status == STATUS_PENDING)
//             {
//                 KeWaitForSingleObject(&Event, Executive, KernelMode, FALSE, NULL);
//                 Status = IoStatusBlock.Status;
//             }
//
//             /* If reading succeeded, compute checksum by adding data, 2048 bytes checksum */
//             if (NT_SUCCESS(Status))
//             {
//                 for (i = 0; i < 2048 / sizeof(ULONG); i++)
//                 {
//                     CheckSum += PartitionBuffer[i];
//                 }
//             }
//         }
//
//         /* Dereference file object */
//         ObDereferenceObject(FileObject);
//
//         /* If checksums are matching, we have the proper cd */
//         if (CheckSum + ArcDiskSignature->CheckSum == 0)
//         {
//             /* Create ARC name */
//             sprintf(ArcBuffer, "\\ArcName\\%s", LoaderBlock->ArcBootDeviceName);
//             RtlInitAnsiString(&ArcNameStringA, ArcBuffer);
//             Status = RtlAnsiStringToUnicodeString(&ArcNameStringW, &ArcNameStringA, TRUE);
//             if (NT_SUCCESS(Status))
//             {
//                 /* Create symbolic link */
//                 IoAssignArcName(&ArcNameStringW, &DeviceStringW);
//                 RtlFreeUnicodeString(&ArcNameStringW);
//                 DPRINT("Boot device found\n");
//             }
//
//             /* And quit, whatever happens */
//             RtlFreeUnicodeString(&DeviceStringW);
//             goto Cleanup;
//         }
//
//         /* Free string before trying another disk */
//         RtlFreeUnicodeString(&DeviceStringW);
//     }
//
// Cleanup:
//     if (PartitionBuffer)
//     {
//         ExFreePoolWithTag(PartitionBuffer, TAG_IO);
//     }
//
//     if (SymbolicLinkList)
//     {
//         ExFreePool(SymbolicLinkList);
//     }
//
//     return Status;
// }
//
// CODE_SEG("INIT")
// NTSTATUS
// NTAPI
// IopCreateArcNamesDisk(IN PLOADER_PARAMETER_BLOCK LoaderBlock,
//                       IN BOOLEAN SingleDisk,
//                       OUT PBOOLEAN FoundBoot)
// {
//     PIRP Irp;
//     PVOID Data;
//     KEVENT Event;
//     NTSTATUS Status;
//     PLIST_ENTRY NextEntry;
//     PFILE_OBJECT FileObject;
//     DISK_GEOMETRY DiskGeometry;
//     PDEVICE_OBJECT DeviceObject;
//     LARGE_INTEGER StartingOffset;
//     PULONG PartitionBuffer = NULL;
//     IO_STATUS_BLOCK IoStatusBlock;
//     CHAR Buffer[128], ArcBuffer[128];
//     BOOLEAN NotEnabledPresent = FALSE;
//     STORAGE_DEVICE_NUMBER DeviceNumber;
//     PARC_DISK_SIGNATURE ArcDiskSignature;
//     PWSTR SymbolicLinkList, lSymbolicLinkList;
//     PDRIVE_LAYOUT_INFORMATION_EX DriveLayout = NULL;
//     _UNICODE_STRING DeviceStringW, ArcNameStringW, HalPathStringW;
//     ULONG DiskNumber, DiskCount, CheckSum, i, Signature, EnabledDisks = 0;
//     PARC_DISK_INFORMATION ArcDiskInformation = LoaderBlock->ArcDiskInformation;
//     ANSI_STRING ArcBootString, ArcSystemString, DeviceStringA, ArcNameStringA, HalPathStringA;
//
//     /* Initialise device number */
//     DeviceNumber.DeviceNumber = ULONG_MAX;
//     /* Get all the disks present in the system */
//     DiskCount = IoGetConfigurationInformation()->DiskCount;
//
//     /* Get enabled disks and check if result matches */
//     Status = IopFetchConfigurationInformation(&SymbolicLinkList,
//                                               GUID_DEVINTERFACE_DISK,
//                                               DiskCount,
//                                               &EnabledDisks);
//     if (!NT_SUCCESS(Status))
//     {
//         NotEnabledPresent = TRUE;
//     }
//
//     /* Save symbolic link list address in order to free it after */
//     lSymbolicLinkList = SymbolicLinkList;
//
//     /* Build the boot strings */
//     RtlInitAnsiString(&ArcBootString, LoaderBlock->ArcBootDeviceName);
//     RtlInitAnsiString(&ArcSystemString, LoaderBlock->ArcHalDeviceName);
//
//     /* If we have more enabled disks, take that into account */
//     if (EnabledDisks > DiskCount)
//     {
//         DiskCount = EnabledDisks;
//     }
//
//     /* If we'll have to browse for none enabled disks, fix higher count */
//     if (NotEnabledPresent && !EnabledDisks)
//     {
//         DiskCount += 20;
//     }
//
//     /* Finally, if in spite of all that work, we still don't have disks, leave */
//     if (!DiskCount)
//     {
//         goto Cleanup;
//     }
//
//     /* Start browsing disks */
//     for (DiskNumber = 0; DiskNumber < DiskCount; DiskNumber++)
//     {
//         ASSERT(DriveLayout == NULL);
//
//         /* Check if we have an enabled disk */
//         if (lSymbolicLinkList && *lSymbolicLinkList != UNICODE_NULL)
//         {
//             /* Create its device name using first symbolic link */
//             RtlInitUnicodeString(&DeviceStringW, lSymbolicLinkList);
//             /* Then, update symbolic links list */
//             lSymbolicLinkList += wcslen(lSymbolicLinkList) + (sizeof(UNICODE_NULL) / sizeof(WCHAR));
//
//             /* Get its associated device object and file object */
//             Status = IoGetDeviceObjectPointer(&DeviceStringW,
//                                               FILE_READ_ATTRIBUTES,
//                                               &FileObject,
//                                               &DeviceObject);
//             if (NT_SUCCESS(Status))
//             {
//                 /* Now, we'll ask the device its device number */
//                 Irp = IoBuildDeviceIoControlRequest(IOCTL_STORAGE_GET_DEVICE_NUMBER,
//                                                     DeviceObject,
//                                                     NULL,
//                                                     0,
//                                                     &DeviceNumber,
//                                                     sizeof(DeviceNumber),
//                                                     FALSE,
//                                                     &Event,
//                                                     &IoStatusBlock);
//                 /* Missing resources is a shame... No need to go farther */
//                 if (!Irp)
//                 {
//                     ObDereferenceObject(FileObject);
//                     Status = STATUS_INSUFFICIENT_RESOURCES;
//                     goto Cleanup;
//                 }
//
//                 /* Call the driver, and wait for it if needed */
//                 KeInitializeEvent(&Event, NotificationEvent, FALSE);
//                 Status = IoCallDriver(DeviceObject, Irp);
//                 if (Status == STATUS_PENDING)
//                 {
//                     KeWaitForSingleObject(&Event, Executive, KernelMode, FALSE, NULL);
//                     Status = IoStatusBlock.Status;
//                 }
//
//                 /* If we didn't get the appropriate data, just skip that disk */
//                 if (!NT_SUCCESS(Status))
//                 {
//                    ObDereferenceObject(FileObject);
//                    continue;
//                 }
//             }
//
//             /* End of enabled disks enumeration */
//             if (NotEnabledPresent && *lSymbolicLinkList == UNICODE_NULL)
//             {
//                 /* No enabled disk worked, reset field */
//                 if (DeviceNumber.DeviceNumber == ULONG_MAX)
//                 {
//                     DeviceNumber.DeviceNumber = 0;
//                 }
//
//                 /* Update disk number to enable the following not enabled disks */
//                 if (DeviceNumber.DeviceNumber > DiskNumber)
//                 {
//                     DiskNumber = DeviceNumber.DeviceNumber;
//                 }
//
//                 /* Increase a bit more */
//                 DiskCount = DiskNumber + 20;
//             }
//         }
//         else
//         {
//             /* Create device name for the disk */
//             sprintf(Buffer, "\\Device\\Harddisk%lu\\Partition0", DiskNumber);
//             RtlInitAnsiString(&DeviceStringA, Buffer);
//             Status = RtlAnsiStringToUnicodeString(&DeviceStringW, &DeviceStringA, TRUE);
//             if (!NT_SUCCESS(Status))
//             {
//                 goto Cleanup;
//             }
//
//             /* Get its device object */
//             Status = IoGetDeviceObjectPointer(&DeviceStringW,
//                                               FILE_READ_ATTRIBUTES,
//                                               &FileObject,
//                                               &DeviceObject);
//
//             RtlFreeUnicodeString(&DeviceStringW);
//             /* This is a security measure, to ensure DiskNumber will be used */
//             DeviceNumber.DeviceNumber = ULONG_MAX;
//         }
//
//         /* Something failed somewhere earlier, just skip the disk */
//         if (!NT_SUCCESS(Status))
//         {
//             continue;
//         }
//
//         /* Let's ask the disk for its geometry */
//         Irp = IoBuildDeviceIoControlRequest(IOCTL_DISK_GET_DRIVE_GEOMETRY,
//                                             DeviceObject,
//                                             NULL,
//                                             0,
//                                             &DiskGeometry,
//                                             sizeof(DiskGeometry),
//                                             FALSE,
//                                             &Event,
//                                             &IoStatusBlock);
//         /* Missing resources is a shame... No need to go farther */
//         if (!Irp)
//         {
//             ObDereferenceObject(FileObject);
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//             goto Cleanup;
//         }
//
//         /* Call the driver, and wait for it if needed */
//         KeInitializeEvent(&Event, NotificationEvent, FALSE);
//         Status = IoCallDriver(DeviceObject, Irp);
//         if (Status == STATUS_PENDING)
//         {
//             KeWaitForSingleObject(&Event, Executive, KernelMode, FALSE, NULL);
//             Status = IoStatusBlock.Status;
//         }
//         /* Failure, skip disk */
//         if (!NT_SUCCESS(Status))
//         {
//             ObDereferenceObject(FileObject);
//             continue;
//         }
//
//         /* Read the partition table */
//         Status = IoReadPartitionTableEx(DeviceObject,
//                                         &DriveLayout);
//         if (!NT_SUCCESS(Status))
//         {
//             ObDereferenceObject(FileObject);
//             continue;
//         }
//
//         /* Ensure we have at least 512 bytes per sector */
//         if (DiskGeometry.BytesPerSector < 512)
//         {
//             DiskGeometry.BytesPerSector = 512;
//         }
//
//         /* Check MBR type against EZ Drive type */
//         StartingOffset.QuadPart = 0;
//         HalExamineMBR(DeviceObject, DiskGeometry.BytesPerSector, 0x55, &Data);
//         if (Data)
//         {
//             /* If MBR is of the EZ Drive type, we'll read after it */
//             StartingOffset.QuadPart = DiskGeometry.BytesPerSector;
//             ExFreePool(Data);
//         }
//
//         /* Allocate for reading enough data for checksum */
//         PartitionBuffer = ExAllocatePoolWithTag(NonPagedPoolCacheAligned, DiskGeometry.BytesPerSector, TAG_IO);
//         if (!PartitionBuffer)
//         {
//             ObDereferenceObject(FileObject);
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//             goto Cleanup;
//         }
//
//         /* Read the first sector for computing checksum */
//         Irp = IoBuildSynchronousFsdRequest(IRP_MJ_READ,
//                                            DeviceObject,
//                                            PartitionBuffer,
//                                            DiskGeometry.BytesPerSector,
//                                            &StartingOffset,
//                                            &Event,
//                                            &IoStatusBlock);
//         if (!Irp)
//         {
//             ExFreePoolWithTag(PartitionBuffer, TAG_IO);
//             ObDereferenceObject(FileObject);
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//             goto Cleanup;
//         }
//
//         /* Call the driver to perform reading */
//         KeInitializeEvent(&Event, NotificationEvent, FALSE);
//         Status = IoCallDriver(DeviceObject, Irp);
//         if (Status == STATUS_PENDING)
//         {
//             KeWaitForSingleObject(&Event, Executive, KernelMode, FALSE, NULL);
//             Status = IoStatusBlock.Status;
//         }
//
//         /* If reading succeeded, calculate checksum by adding data */
//         if (NT_SUCCESS(Status))
//         {
//             for (i = 0, CheckSum = 0; i < 512 / sizeof(ULONG); i++)
//             {
//                 CheckSum += PartitionBuffer[i];
//             }
//         }
//
//         /* Release now unnecessary resources */
//         ExFreePoolWithTag(PartitionBuffer, TAG_IO);
//         ObDereferenceObject(FileObject);
//
//         /* If we failed, release drive layout before going to next disk */
//         if (!NT_SUCCESS(Status))
//         {
//             ExFreePool(DriveLayout);
//             DriveLayout = NULL;
//             continue;
//         }
//
//         /* Browse each ARC disk */
//         for (NextEntry = ArcDiskInformation->DiskSignatureListHead.Flink;
//              NextEntry != &ArcDiskInformation->DiskSignatureListHead;
//              NextEntry = NextEntry->Flink)
//         {
//             ArcDiskSignature = CONTAINING_RECORD(NextEntry,
//                                                  ARC_DISK_SIGNATURE,
//                                                  ListEntry);
//
//             /*
//              * If this is the only MBR disk in the ARC list and detected
//              * in the device tree, just go ahead and create the ArcName link.
//              * Otherwise, check whether the signatures and checksums match
//              * before creating the ArcName link.
//              */
//             if ((SingleDisk && (DiskCount == 1) &&
//                  (DriveLayout->PartitionStyle == PARTITION_STYLE_MBR)) ||
//                 (IopVerifyDiskSignature(DriveLayout, ArcDiskSignature, &Signature) &&
//                  (ArcDiskSignature->CheckSum + CheckSum == 0)))
//             {
//                 /* Create device name */
//                 sprintf(Buffer, "\\Device\\Harddisk%lu\\Partition0",
//                         (DeviceNumber.DeviceNumber != ULONG_MAX) ? DeviceNumber.DeviceNumber : DiskNumber);
//                 RtlInitAnsiString(&DeviceStringA, Buffer);
//                 Status = RtlAnsiStringToUnicodeString(&DeviceStringW, &DeviceStringA, TRUE);
//                 if (!NT_SUCCESS(Status))
//                 {
//                     goto Cleanup;
//                 }
//
//                 /* Create ARC name */
//                 sprintf(ArcBuffer, "\\ArcName\\%s", ArcDiskSignature->ArcName);
//                 RtlInitAnsiString(&ArcNameStringA, ArcBuffer);
//                 Status = RtlAnsiStringToUnicodeString(&ArcNameStringW, &ArcNameStringA, TRUE);
//                 if (!NT_SUCCESS(Status))
//                 {
//                     RtlFreeUnicodeString(&DeviceStringW);
//                     goto Cleanup;
//                 }
//
//                 /* Link both */
//                 IoAssignArcName(&ArcNameStringW, &DeviceStringW);
//
//                 /* And release strings */
//                 RtlFreeUnicodeString(&ArcNameStringW);
//                 RtlFreeUnicodeString(&DeviceStringW);
//
//                 /* Now, browse each partition */
//                 for (i = 1; i <= DriveLayout->PartitionCount; i++)
//                 {
//                     /* Create device name */
//                     sprintf(Buffer, "\\Device\\Harddisk%lu\\Partition%lu",
//                             (DeviceNumber.DeviceNumber != ULONG_MAX) ? DeviceNumber.DeviceNumber : DiskNumber, i);
//                     RtlInitAnsiString(&DeviceStringA, Buffer);
//                     Status = RtlAnsiStringToUnicodeString(&DeviceStringW, &DeviceStringA, TRUE);
//                     if (!NT_SUCCESS(Status))
//                     {
//                         goto Cleanup;
//                     }
//
//                     /* Create partial ARC name */
//                     sprintf(ArcBuffer, "%spartition(%lu)", ArcDiskSignature->ArcName, i);
//                     RtlInitAnsiString(&ArcNameStringA, ArcBuffer);
//
//                     /* Is that boot device? */
//                     if (RtlEqualString(&ArcNameStringA, &ArcBootString, TRUE))
//                     {
//                         DPRINT("Found boot device\n");
//                         *FoundBoot = TRUE;
//                     }
//
//                     /* Is that system partition? */
//                     if (RtlEqualString(&ArcNameStringA, &ArcSystemString, TRUE))
//                     {
//                         /* Create HAL path name */
//                         RtlInitAnsiString(&HalPathStringA, LoaderBlock->NtHalPathName);
//                         Status = RtlAnsiStringToUnicodeString(&HalPathStringW, &HalPathStringA, TRUE);
//                         if (!NT_SUCCESS(Status))
//                         {
//                             RtlFreeUnicodeString(&DeviceStringW);
//                             goto Cleanup;
//                         }
//
//                         /* Then store those information to registry */
//                         IopStoreSystemPartitionInformation(&DeviceStringW, &HalPathStringW);
//                         RtlFreeUnicodeString(&HalPathStringW);
//                     }
//
//                     /* Create complete ARC name */
//                     sprintf(ArcBuffer, "\\ArcName\\%spartition(%lu)", ArcDiskSignature->ArcName, i);
//                     RtlInitAnsiString(&ArcNameStringA, ArcBuffer);
//                     Status = RtlAnsiStringToUnicodeString(&ArcNameStringW, &ArcNameStringA, TRUE);
//                     if (!NT_SUCCESS(Status))
//                     {
//                         RtlFreeUnicodeString(&DeviceStringW);
//                         goto Cleanup;
//                     }
//
//                     /* Link device name & ARC name */
//                     IoAssignArcName(&ArcNameStringW, &DeviceStringW);
//
//                     /* Release strings */
//                     RtlFreeUnicodeString(&ArcNameStringW);
//                     RtlFreeUnicodeString(&DeviceStringW);
//                 }
//             }
//             else
//             {
//                 /* Debugging feedback: Warn in case there's a valid partition,
//                  * a matching signature, BUT a non-matching checksum: this can
//                  * be the sign of a duplicate signature, or even worse a virus
//                  * played with the partition table. */
//                 if (ArcDiskSignature->ValidPartitionTable &&
//                     (ArcDiskSignature->Signature == Signature) &&
//                     (ArcDiskSignature->CheckSum + CheckSum != 0))
//                 {
//                     DPRINT("Be careful, you have a duplicate disk signature, or a virus altered your MBR!\n");
//                 }
//             }
//         }
//
//         /* Finally, release drive layout */
//         ExFreePool(DriveLayout);
//         DriveLayout = NULL;
//     }
//
//     Status = STATUS_SUCCESS;
//
// Cleanup:
//     if (DriveLayout)
//     {
//         ExFreePool(DriveLayout);
//     }
//
//     if (SymbolicLinkList)
//     {
//         ExFreePool(SymbolicLinkList);
//     }
//
//     return Status;
// }
//
// CODE_SEG("INIT")
// NTSTATUS
// NTAPI
// IopReassignSystemRoot(IN PLOADER_PARAMETER_BLOCK LoaderBlock,
//                       OUT PANSI_STRING NtBootPath)
// {
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     NTSTATUS Status;
//     CHAR Buffer[256], AnsiBuffer[256];
//     WCHAR ArcNameBuffer[64];
//     ANSI_STRING TargetString, ArcString, TempString;
//     _UNICODE_STRING LinkName, TargetName, ArcName;
//     HANDLE LinkHandle;
//
//     /* Create the Unicode name for the current ARC boot device */
//     sprintf(Buffer, "\\ArcName\\%s", LoaderBlock->ArcBootDeviceName);
//     RtlInitAnsiString(&TargetString, Buffer);
//     Status = RtlAnsiStringToUnicodeString(&TargetName, &TargetString, TRUE);
//     if (!NT_SUCCESS(Status)) return FALSE;
//
//     /* Initialize the attributes and open the link */
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &TargetName,
//                                OBJ_CASE_INSENSITIVE,
//                                NULL,
//                                NULL);
//     Status = NtOpenSymbolicLinkObject(&LinkHandle,
//                                       SYMBOLIC_LINK_ALL_ACCESS,
//                                       &ObjectAttributes);
//     if (!NT_SUCCESS(Status))
//     {
//         /* We failed, free the string */
//         RtlFreeUnicodeString(&TargetName);
//         return FALSE;
//     }
//
//     /* Query the current \\SystemRoot */
//     ArcName.Buffer = ArcNameBuffer;
//     ArcName.Length = 0;
//     ArcName.MaximumLength = sizeof(ArcNameBuffer);
//     Status = NtQuerySymbolicLinkObject(LinkHandle, &ArcName, NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         /* We failed, free the string */
//         RtlFreeUnicodeString(&TargetName);
//         return FALSE;
//     }
//
//     /* Convert it to Ansi */
//     ArcString.Buffer = AnsiBuffer;
//     ArcString.Length = 0;
//     ArcString.MaximumLength = sizeof(AnsiBuffer);
//     Status = RtlUnicodeStringToAnsiString(&ArcString, &ArcName, FALSE);
//     AnsiBuffer[ArcString.Length] = ANSI_NULL;
//
//     /* Close the link handle and free the name */
//     ObCloseHandle(LinkHandle, KernelMode);
//     RtlFreeUnicodeString(&TargetName);
//
//     /* Setup the system root name again */
//     RtlInitAnsiString(&TempString, "\\SystemRoot");
//     Status = RtlAnsiStringToUnicodeString(&LinkName, &TempString, TRUE);
//     if (!NT_SUCCESS(Status)) return FALSE;
//
//     /* Open the symbolic link for it */
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &LinkName,
//                                OBJ_CASE_INSENSITIVE,
//                                NULL,
//                                NULL);
//     Status = NtOpenSymbolicLinkObject(&LinkHandle,
//                                       SYMBOLIC_LINK_ALL_ACCESS,
//                                       &ObjectAttributes);
//     if (!NT_SUCCESS(Status)) return FALSE;
//
//     /* Destroy it */
//     NtMakeTemporaryObject(LinkHandle);
//     ObCloseHandle(LinkHandle, KernelMode);
//
//     /* Now create the new name for it */
//     sprintf(Buffer, "%s%s", ArcString.Buffer, LoaderBlock->NtBootPathName);
//
//     /* Copy it into the passed parameter and null-terminate it */
//     RtlCopyString(NtBootPath, &ArcString);
//     Buffer[strlen(Buffer) - 1] = ANSI_NULL;
//
//     /* Setup the Unicode-name for the new symbolic link value */
//     RtlInitAnsiString(&TargetString, Buffer);
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &LinkName,
//                                OBJ_CASE_INSENSITIVE | OBJ_PERMANENT,
//                                NULL,
//                                NULL);
//     Status = RtlAnsiStringToUnicodeString(&ArcName, &TargetString, TRUE);
//     if (!NT_SUCCESS(Status)) return FALSE;
//
//     /* Create it */
//     Status = NtCreateSymbolicLinkObject(&LinkHandle,
//                                         SYMBOLIC_LINK_ALL_ACCESS,
//                                         &ObjectAttributes,
//                                         &ArcName);
//
//     /* Free all the strings and close the handle and return success */
//     RtlFreeUnicodeString(&ArcName);
//     RtlFreeUnicodeString(&LinkName);
//     ObCloseHandle(LinkHandle, KernelMode);
//     return TRUE;
// }
//
// BOOLEAN
// IopVerifyDiskSignature(
//     _In_ PDRIVE_LAYOUT_INFORMATION_EX DriveLayout,
//     _In_ PARC_DISK_SIGNATURE ArcDiskSignature,
//     _Out_ PULONG Signature)
// {
//     /* Fail if the partition table is invalid */
//     if (!ArcDiskSignature->ValidPartitionTable)
//         return FALSE;
//
//     /* If the partition style is MBR */
//     if (DriveLayout->PartitionStyle == PARTITION_STYLE_MBR)
//     {
//         /* Check the MBR signature */
//         if (DriveLayout->Mbr.Signature == ArcDiskSignature->Signature)
//         {
//             /* And return it */
//             if (Signature)
//                 *Signature = DriveLayout->Mbr.Signature;
//             return TRUE;
//         }
//     }
//     /* If the partition style is GPT */
//     else if (DriveLayout->PartitionStyle == PARTITION_STYLE_GPT)
//     {
//         /* Verify whether the signature is GPT and compare the GUID */
//         if (ArcDiskSignature->IsGpt &&
//             IsEqualGUID((PGUID)&ArcDiskSignature->GptSignature, &DriveLayout->Gpt.DiskId))
//         {
//             /* There is no signature to return, just zero it */
//             if (Signature)
//                 *Signature = 0;
//             return TRUE;
//         }
//     }
//
//     /* If we get there, something went wrong, so fail */
//     return FALSE;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
