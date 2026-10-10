use crate::types::*;
// InitSafeBootMode is a kernel DATA export — needs raw-dylib so the reference goes
// through the import thunk (__imp_) instead of expecting a local definition.
#[link(name = "ntoskrnl.exe", kind = "raw-dylib", modifiers = "+verbatim")]
extern "C" { static InitSafeBootMode: ULONG; }
use crate::glue::*;
use core::ffi::{c_char, c_int, c_long, c_uchar, c_uint, c_ushort};
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{DISK_CACHE_INFORMATION, DISK_GEOMETRY, _KWAIT_REASON_Executive, HalExamineMBR, _IO_STATUS_BLOCK, IoBuildDeviceIoControlRequest, IoCreateSymbolicLink, IoDeleteSymbolicLink, IoGetConfigurationInformation, IoOpenDeviceRegistryKey, KEVENT, KeInitializeEvent, KeInitializeMutex, KeInitializeSpinLock, KeWaitForSingleObject, _OBJECT_ATTRIBUTES, PCCHAR, PCONFIGURATION_INFORMATION, PDEVICE_OBJECT, PDRIVER_OBJECT, PIRP, RTL_QUERY_REGISTRY_TABLE, RtlInitUnicodeString, RtlQueryRegistryValues, STORAGE_HOTPLUG_INFO, _EVENT_TYPE_SynchronizationEvent, ZwClose, ZwMakeTemporaryObject, ZwOpenKey};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports (x86/amd64 only — arm64 types lack these)
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
use crate::types::{_DISK_GEOMETRY_SOURCE_DiskGeometryFromNT4 as DiskGeometryFromNT4, _DISK_GEOMETRY_SOURCE_DiskGeometryUnknown as DiskGeometryUnknown};
use ntkrnl_sys::ddk::{DEVICE_OBJECT, EVENT_TYPE, HANDLE, KPROCESSOR_MODE, KWAIT_REASON, POOL_TYPE, _UNICODE_STRING};
// resolve_imports.py: explicit kernel/private-type imports
use ntkrnl_sys::ddk::{ExFreePool};

// w! — NUL-terminated UTF-16 array from an ASCII literal (2-aligned 'static;
// a b"...\0" byte string cast to *const u16 is both ANSI-not-wide and
// potentially odd-aligned — see field manual / basesrv init.rs CSDVersion).
macro_rules! w {
    ($s:literal) => {{
        const __W: &[u16] = &{
            const BYTES: &[u8] = $s.as_bytes();
            let mut out = [0u16; { BYTES.len() + 1 }];
            let mut i = 0;
            while i < BYTES.len() {
                out[i] = BYTES[i] as u16;
                i += 1;
            }
            out
        };
        __W
    }};
}

// moved from extern "system": types_i386 declares these cdecl (no NTAPI in disk.h);
// the defs are extern "C" — CC must agree or LLVM folds callers to ud2.
extern "C" {
    fn DiskCreateFdo(DriverObject: PDRIVER_OBJECT, Pdo: PDEVICE_OBJECT, Count: *mut ULONG, SuppressMount: u32) -> NTSTATUS;
    pub fn DiskReadSignature(Fdo: PDEVICE_OBJECT) -> NTSTATUS;
    pub fn DiskReadDriveCapacity(Fdo: PDEVICE_OBJECT) -> NTSTATUS;
    fn DiskIsNT4Geometry(Ext: PFUNCTIONAL_DEVICE_EXTENSION) -> BOOLEAN;
}

extern "C" {
    fn DbgPrint(format: *const i8, ...);
    fn __int2c();
    fn TracePrint(level: i32, flags: u32, format: *const i8, ...);
}
extern "system" {
    fn KeGetCurrentIrql() -> u8;
}
extern "C" {
    fn RtlStringCchPrintfW( pszDest: *mut u16, cchDest: usize, pszFormat: *const u16, ... ) -> i32;
    fn StorDebugPrint(level: i32, flag: u32, format: PCCHAR, ...);
}
extern "system" {
    fn DiskGetCacheInformation(fdo: PFUNCTIONAL_DEVICE_EXTENSION, info: *mut DISK_CACHE_INFORMATION) -> NTSTATUS;
    fn DiskSetCacheInformation(fdo: PFUNCTIONAL_DEVICE_EXTENSION, info: *mut DISK_CACHE_INFORMATION) -> u32;
    fn ClassGetDeviceParameter(fdo: PFUNCTIONAL_DEVICE_EXTENSION, subkey: *mut u16, value: *mut u16, out: *mut ULONG);
    fn ClassInitializeSrbLookasideList(Ext: PCOMMON_DEVICE_EXTENSION, ListSize: ULONG);
    fn ClassReadDriveCapacity(Fdo: PDEVICE_OBJECT) -> NTSTATUS;
    fn ClassUpdateInformationInRegistry( Fdo: PDEVICE_OBJECT, Name: PCHAR, DeviceNumber: ULONG, InquiryData: PVOID, Len: ULONG, );
    fn ClassInitializeMediaChangeDetection(Ext: PFUNCTIONAL_DEVICE_EXTENSION, Name: PUCHAR);
}
// DiskDetectFailurePrediction is defined in diskwmi.rs (diskwmi.c); pnp only calls it.
// The spurious duplicate def the translation emitted here was removed.

// DiskGenerateDeviceName (pnp.c) — the bulk translate skipped it; hand-translated here.
// extern "system" matches the bindgen decl (stdcall on i386, cdecl on amd64/arm64).
extern "C" {
    // __cdecl (variadic) on all arches
    fn RtlStringCchPrintfA(dst: *mut i8, cch: usize, fmt: *const i8, ...) -> NTSTATUS;
    fn strlen(s: *const i8) -> usize;
}
extern "system" {
    // NTAPI = stdcall on i386, cdecl on amd64/arm64
    fn RtlStringCchCopyA(dst: *mut i8, cch: usize, src: *const i8) -> NTSTATUS;
}
const DISK_TAG_NAME: u32 = 0x4E446353; // 'NDcS'
#[no_mangle]
pub unsafe extern "C" fn DiskGenerateDeviceName(
    DeviceNumber: ULONG,
    RawName: *mut PCCHAR,
) -> NTSTATUS {
    let mut rawName: [i8; 64] = [0; 64];
    let seq = diskDeviceSequenceNumber;
    diskDeviceSequenceNumber += 1;
    let mut status = RtlStringCchPrintfA(
        rawName.as_mut_ptr(),
        63,
        b"\\Device\\Harddisk%d\\DR%d\0".as_ptr() as *const i8,
        DeviceNumber,
        seq,
    );
    if status < 0 {
        TracePrint(
            TRACE_LEVEL_ERROR as i32,
            TRACE_FLAG_PNP,
            b"DiskGenerateDeviceName: Format FDO name failed with error: 0x%X\n\0".as_ptr() as *const i8,
            status,
        );
        return status;
    }
    let len = strlen(rawName.as_ptr());
    *RawName = ExAllocatePoolWithTag(ntkrnl_sys::ddk::_POOL_TYPE_PagedPool, len + 1, DISK_TAG_NAME) as PCCHAR;
    if (*RawName).is_null() {
        return 0xC000009Au32 as i32; // STATUS_INSUFFICIENT_RESOURCES
    }
    status = RtlStringCchCopyA(*RawName, len + 1, rawName.as_ptr());
    if status < 0 {
        TracePrint(
            TRACE_LEVEL_ERROR as i32,
            TRACE_FLAG_PNP,
            b"DiskGenerateDeviceName: Device name copy failed with error: 0x%X\n\0".as_ptr() as *const i8,
            status,
        );
        ExFreePool(*RawName as *mut core::ffi::c_void);
        return status;
    }
    0 // STATUS_SUCCESS
}

pub const DiskDeviceUserWriteCacheSetting: [::core::ffi::c_ushort; 22] = unsafe {
    ::core::mem::transmute::<[u8; 44], [::core::ffi::c_ushort; 22]>(
        *b"U\0s\0e\0r\0W\0r\0i\0t\0e\0C\0a\0c\0h\0e\0S\0e\0t\0t\0i\0n\0g\0\0\0",
    )
};

pub const DiskDeviceCacheIsPowerProtected: [::core::ffi::c_ushort; 22] = unsafe {
    ::core::mem::transmute::<[u8; 44], [::core::ffi::c_ushort; 22]>(
        *b"C\0a\0c\0h\0e\0I\0s\0P\0o\0w\0e\0r\0P\0r\0o\0t\0e\0c\0t\0e\0d\0\0\0",
    )
};

pub static mut diskDeviceSequenceNumber: ULONG = 0 as ULONG;

pub const FDO_NAME_FORMAT: [::core::ffi::c_char; 24] = unsafe {
    ::core::mem::transmute::<[u8; 24], [::core::ffi::c_char; 24]>(*b"\\Device\\Harddisk%d\\DR%d\0")
};

const APC_LEVEL: u32 = 1;

// Forward extern declarations for non-imported functions and globals

const FILE_REMOVABLE_MEDIA: ULONG = 0x1;
const SRB_FLAGS_PORT_DRIVER_ALLOCSENSE: ULONG = 0x200000;
const SRB_FLAGS_QUEUE_ACTION_ENABLE: ULONG = 0x2;
const SRB_FLAGS_NO_QUEUE_FREEZE: ULONG = 0x100;
const DEV_SAFE_START_UNIT: USHORT = 0x4;
const PARTITION0_LIST_SIZE: ULONG = 4; // example constant
// scsi.h:596 `sizeof(SENSE_DATA_EX)` -- take it from the type, not from a guess.
const SENSE_BUFFER_SIZE_EX: SIZE_T = core::mem::size_of::<SENSE_DATA_EX>() as SIZE_T;
const SCSI_DISK_TIMEOUT: ULONG = 0x0A;
// disk.h:75: `#define NonPagedPoolNxCacheAligned NonPagedPoolCacheAligned` under
// `#ifdef __REACTOS__` -- NX pool is NT6.2 and this kernel has none, so the
// Windows value (0x200/0x204) is not what any TU here compiles.
const NonPagedPoolNxCacheAligned: POOL_TYPE = 4;
                                                 // NonPagedPoolCacheAligned
const DISK_TAG_START: ULONG = 0x73446353;
const MAX_SECTORS_PER_VERIFY: ULONG = 0x0100;
const TRUE: BOOLEAN = 1;
const FALSE: BOOLEAN = 0;
const STATUS_SUCCESS: NTSTATUS = 0;
const STATUS_INSUFFICIENT_RESOURCES: NTSTATUS = 0xC000009Au32 as i32 ;
const STATUS_UNSUCCESSFUL: NTSTATUS = 0xC0000001u32 as i32 ;
const STATUS_PENDING: NTSTATUS = 0x00000103;
const Executive: KWAIT_REASON = 0;
const SynchronizationEvent: EVENT_TYPE = 0x01;

// External declarations for symbols not in available imports
unsafe extern "system" {
    fn ClassDeleteSrbLookasideList(Extension: PCOMMON_DEVICE_EXTENSION);
}
// DiskDeleteSymbolicLinks is defined at the bottom of this file (re-integrated from the
// translation cache — the stitcher dropped it; the commented C body is around line 1699).

#[no_mangle]
pub unsafe extern "system" fn DiskAddDevice(
    DriverObject: PDRIVER_OBJECT,
    PhysicalDeviceObject: PDEVICE_OBJECT,
) -> NTSTATUS {
    // Required extern declarations for gap functions

    // Required constants not in available imports
    const APC_LEVEL: u8 = 1;
    const PLUGPLAY_REGKEY_DEVICE: u32 = 1;
    const RTL_QUERY_REGISTRY_TYPECHECK_SHIFT: u32 = 24;
    const TRACE_FLAG_PNP: u32 = 1; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)

    let mut rootPartitionMountable: ULONG = 0; // FALSE
    let mut configurationInformation: PCONFIGURATION_INFORMATION = core::ptr::null_mut();
    let mut diskCount: ULONG = 0;
    let mut status: NTSTATUS = 0;

    // PAGED_CODE() – anchor-preserved block
    {
        let current_irql = KeGetCurrentIrql();
        if (current_irql as i32) > (APC_LEVEL as i32) {
            DbgPrint(
                b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8,
                current_irql as i32,
            );
            if (0 as i32) == 0 {
                DbgPrint(
                    b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const i8,
                    b"drivers/storage/class/disk/pnp.c\0".as_ptr() as *const i8,
                    86i32,
                    b"0\0".as_ptr() as *const i8,
                );
                __int2c();
            }
        }
    }

    let mut deviceKey: HANDLE = core::ptr::null_mut();
    let mut diskKeyName: _UNICODE_STRING = core::mem::zeroed();
    let mut objectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut diskKey: HANDLE = core::ptr::null_mut();
    let mut queryTable: [RTL_QUERY_REGISTRY_TABLE; 2] = [
        core::mem::zeroed(),
        core::mem::zeroed(),
    ];

    status = IoOpenDeviceRegistryKey(
        PhysicalDeviceObject,
        PLUGPLAY_REGKEY_DEVICE,
        KEY_READ,
        &raw mut deviceKey,
    );

    if !(NT_SUCCESS(status)) {
        TracePrint(
            2,
            TRACE_FLAG_PNP,
            b"DiskAddDevice: Error %#08lx opening device key for pdo %p\n\0".as_ptr() as *const i8,
            status,
            PhysicalDeviceObject,
        );
    } else {
        RtlInitUnicodeString(
            &raw mut diskKeyName,
            w!("Disk").as_ptr() as *const u16,
        );

        objectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
        objectAttributes.RootDirectory = deviceKey;
        objectAttributes.ObjectName = &raw mut diskKeyName;
        objectAttributes.Attributes = (OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE) as ULONG;
        objectAttributes.SecurityDescriptor = core::ptr::null_mut();
        objectAttributes.SecurityQualityOfService = core::ptr::null_mut();

        status = ZwOpenKey(&raw mut diskKey, KEY_READ, &raw mut objectAttributes);
        ZwClose(deviceKey);

        if !(NT_SUCCESS(status)) {
            TracePrint(
                2,
                TRACE_FLAG_PNP,
                b"DiskAddDevice: Error %#08lx opening disk key for pdo %p device key %p\n\0"
                    .as_ptr() as *const i8,
                status,
                PhysicalDeviceObject,
                deviceKey,
            );
        } else {
            queryTable[0].Flags = (RTL_QUERY_REGISTRY_DIRECT | RTL_QUERY_REGISTRY_TYPECHECK) as ULONG;
            queryTable[0].Name = win32macros_rust::wide_bytes!(b"R\0o\0o\0t\0P\0a\0r\0t\0i\0t\0i\0o\0n\0M\0o\0u\0n\0t\0a\0b\0l\0e\0\0\0") as *const u16;
            queryTable[0].EntryContext = &raw mut rootPartitionMountable as *mut core::ffi::c_void;
            queryTable[0].DefaultType = (REG_DWORD << RTL_QUERY_REGISTRY_TYPECHECK_SHIFT | REG_NONE) as ULONG;

            status = RtlQueryRegistryValues(
                RTL_REGISTRY_HANDLE,
                diskKey as *const u16,
                &raw mut queryTable as *mut RTL_QUERY_REGISTRY_TABLE,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );

            if !(NT_SUCCESS(status)) {
                TracePrint(
                    2,
                    TRACE_FLAG_PNP,
                    b"DiskAddDevice: Error %#08lx reading value from disk key %p for pdo %p\n\0"
                        .as_ptr() as *const i8,
                    status,
                    diskKey,
                    PhysicalDeviceObject,
                );
            }
            ZwClose(diskKey);
        }
    }

    // FINALLY (explicit after the construct)
    if !(NT_SUCCESS(status)) {
        TracePrint(
            3,
            TRACE_FLAG_PNP,
            b"DiskAddDevice: Will %sallow file system to mount on partition zero of disk %p\n\0"
                .as_ptr() as *const i8,
            if rootPartitionMountable != 0 {
                b"\0".as_ptr() as *const i8
            } else {
                b"not \0".as_ptr() as *const i8
            },
            PhysicalDeviceObject,
        );
    }

    diskCount = 0;
    status = DiskCreateFdo(
        DriverObject,
        PhysicalDeviceObject,
        &raw mut diskCount,
        ((rootPartitionMountable == 0) as BOOLEAN) as _,
    );

    configurationInformation = IoGetConfigurationInformation();

    if NT_SUCCESS(status) {
        (*configurationInformation).DiskCount = (*configurationInformation).DiskCount.wrapping_add(1);
    }

    return status;
}

#[no_mangle]
pub unsafe extern "system" fn DiskInitFdo(Fdo: PDEVICE_OBJECT) -> NTSTATUS {
    let fdo_extension: PFUNCTIONAL_DEVICE_EXTENSION =
        (*Fdo).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    let disk_data: PDISK_DATA = (*fdo_extension).__bindgen_anon_1.CommonExtension.as_ref().DriverData as PDISK_DATA;
    let mut srb_flags: ULONG = 0;
    let mut time_out: ULONG = 0;
    let mut bytes_per_sector: ULONG = 0;
    let mut dm_skew: *mut ULONG = core::ptr::null_mut();
    let mut status: NTSTATUS = STATUS_SUCCESS;

    // PAGED_CODE() expansion
    if KeGetCurrentIrql() > (APC_LEVEL) as u8 {
        DbgPrint(
            (b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr()).cast(),
            KeGetCurrentIrql() as i32,
        );
        if false {
            DbgPrint(
                (b"Assertion failed at %s(%d): %s\n\0".as_ptr()).cast(),
                b"pnp.c\0".as_ptr() as *const i8, // simplified file name
                235,
                b"0\0".as_ptr() as *const i8,
            );
            __int2c();
        }
    }

    ClassInitializeSrbLookasideList(
        fdo_extension as PCOMMON_DEVICE_EXTENSION,
        PARTITION0_LIST_SIZE,
    );

    if (*(*fdo_extension).DeviceDescriptor).RemovableMedia != FALSE {
        (*Fdo).Characteristics |= FILE_REMOVABLE_MEDIA;
    }

    (*fdo_extension).SrbFlags |= SRB_FLAGS_PORT_DRIVER_ALLOCSENSE;

    if (*(*fdo_extension).DeviceDescriptor).CommandQueueing != FALSE
        && (*(*fdo_extension).AdapterDescriptor).CommandQueueing != FALSE
    {
        (*fdo_extension).SrbFlags |= SRB_FLAGS_QUEUE_ACTION_ENABLE;
    }

    ClassScanForSpecial(
        fdo_extension,
        (&raw const DiskBadControllers).cast::<CLASSPNP_SCAN_FOR_SPECIAL_INFO>() as *mut CLASSPNP_SCAN_FOR_SPECIAL_INFO,
        Some(DiskSetSpecialHacks),
    );

    core::ptr::write_bytes(
        core::ptr::addr_of_mut!((*fdo_extension).DiskGeometry) as *mut u8,
        0,
        core::mem::size_of::<DISK_GEOMETRY>(),
    );

    (*fdo_extension).SenseData = ExAllocatePoolWithTag(
        NonPagedPoolNxCacheAligned,
        SENSE_BUFFER_SIZE_EX,
        DISK_TAG_START,
    ) as PSENSE_DATA;

    if (*fdo_extension).SenseData.is_null() {
        const TRACE_FLAG_PNP: u32 = 1; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)
        TracePrint(
            2,
            TRACE_FLAG_PNP,
            b"DiskInitFdo: Can not allocate request sense buffer\n\0".as_ptr() as *const i8,
        );
        status = STATUS_INSUFFICIENT_RESOURCES;
        return status;
    }

    (*fdo_extension).SenseDataLength = SENSE_BUFFER_SIZE_EX as UCHAR;

    (*fdo_extension).__bindgen_anon_1.CommonExtension.as_mut().StartingOffset.QuadPart = 0;

    // IoTimeoutValue is a WIN8+ conditional field absent from this bindgen output;
    // ReactOS classpnp.c zeroes it unconditionally, so the condition is always false here.
    if !(*fdo_extension).MiniportDescriptor.is_null()
        && false /* (*(*fdo_extension).MiniportDescriptor).IoTimeoutValue > 0 */
    {
        (*fdo_extension).TimeOutValue =
            0u16 as ULONG; /* (*(*fdo_extension).MiniportDescriptor).IoTimeoutValue as ULONG */
    } else {
        time_out = ClassQueryTimeOutRegistryValue(Fdo);
        if time_out != 0 {
            (*fdo_extension).TimeOutValue = time_out;
        } else {
            (*fdo_extension).TimeOutValue = SCSI_DISK_TIMEOUT;
        }
    }

    if (*Fdo).Characteristics & FILE_REMOVABLE_MEDIA != 0 {
        ClassUpdateInformationInRegistry(
            Fdo,
            b"PhysicalDrive\0".as_ptr() as PCHAR,
            (*fdo_extension).DeviceNumber,
            core::ptr::null_mut(),
            0,
        );
        ClassInitializeMediaChangeDetection(
            fdo_extension,
            b"Disk\0".as_ptr() as PUCHAR,
        );
    } else {
        (*fdo_extension).DeviceFlags |= DEV_SAFE_START_UNIT;
        (*fdo_extension).SrbFlags |= SRB_FLAGS_NO_QUEUE_FREEZE;
    }

    srb_flags = (*fdo_extension).SrbFlags;

    ClassReadDriveCapacity(Fdo);

    bytes_per_sector = (*fdo_extension).DiskGeometry.BytesPerSector;
    if bytes_per_sector == 0 {
        (*fdo_extension).DiskGeometry.BytesPerSector = 512;
        bytes_per_sector = 512;
        (*fdo_extension).SectorShift = 9;
    }

    HalExamineMBR(
        (*fdo_extension).__bindgen_anon_1.CommonExtension.as_ref().DeviceObject,
        (*fdo_extension).DiskGeometry.BytesPerSector,
        0x54,
        &raw mut dm_skew as *mut PVOID,
    );

    if !dm_skew.is_null() {
        (*fdo_extension).DMSkew = *dm_skew as ULONG;
        (*fdo_extension).DMActive = TRUE;
        (*fdo_extension).DMByteSkew = (*fdo_extension).DMSkew.wrapping_mul(bytes_per_sector);
        ExFreePool(dm_skew as PVOID);
        dm_skew = core::ptr::null_mut();
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    if (*Fdo).Characteristics & FILE_REMOVABLE_MEDIA == 0 {
        DiskReadSignature(Fdo);
        DiskReadDriveCapacity(Fdo);

        if (*disk_data).GeometrySource == DiskGeometryUnknown {
            if DiskIsNT4Geometry(fdo_extension) != FALSE {
                (*disk_data).RealGeometry = (*fdo_extension).DiskGeometry;
                (*disk_data).RealGeometry.SectorsPerTrack = 0x20;
                (*disk_data).RealGeometry.TracksPerCylinder = 0x40;
                (*fdo_extension).DiskGeometry = (*disk_data).RealGeometry;
                (*disk_data).GeometrySource = DiskGeometryFromNT4;
            }
        }
    }

    DiskCreateSymbolicLinks(Fdo);

    // SCSI address block
    let mut irp: PIRP = core::ptr::null_mut();
    let mut event: KEVENT = core::mem::zeroed();
    let mut status_block: _IO_STATUS_BLOCK = core::mem::zeroed();
    KeInitializeEvent(&raw mut event, _EVENT_TYPE_SynchronizationEvent, FALSE);
    const IOCTL_SCSI_GET_ADDRESS: ULONG = 0x41018; // fallback local constant
    irp = IoBuildDeviceIoControlRequest(
        IOCTL_SCSI_GET_ADDRESS,
        (*fdo_extension).__bindgen_anon_1.CommonExtension.as_ref().LowerDeviceObject,
        core::ptr::null_mut(),
        0,
        &raw mut (*disk_data).ScsiAddress as PVOID,
        core::mem::size_of::<SCSI_ADDRESS>() as ULONG,
        FALSE,
        &raw mut event,
        &raw mut status_block,
    );
    status = STATUS_UNSUCCESSFUL;
    if !irp.is_null() {
        status = IoCallDriver(
            (*fdo_extension).__bindgen_anon_1.CommonExtension.as_ref().LowerDeviceObject,
            irp,
        );
        if status == STATUS_PENDING {
            KeWaitForSingleObject(
                &raw mut event as PVOID,
                _KWAIT_REASON_Executive,
                (0i32) as i8,
                FALSE,
                core::ptr::null_mut(),
            );
            status = status_block.__bindgen_anon_1.Status;
        }
    }

    if InitSafeBootMode == 0 {
        DiskDetectFailurePrediction(
            fdo_extension,
            (&raw mut (*disk_data).FailurePredictionCapability).cast(),
            ((status >= 0) as BOOLEAN) as _,
        );
        if (*disk_data).FailurePredictionCapability != 0 {
            (*disk_data).AllowFPPerfHit = TRUE;
            status = DiskEnableDisableFailurePredictPolling(
                fdo_extension,
                TRUE,
                DISK_DEFAULT_FAILURE_POLLING_PERIOD,
            );
            const TRACE_FLAG_PNP: u32 = 1; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)
            TracePrint(
                4,
                TRACE_FLAG_PNP,
                b"DiskInitFdo: Failure Prediction Poll enabled as %d for device %p, Status = %lx\n\0"
                    .as_ptr() as *const i8,
                (*disk_data).FailurePredictionCapability,
                Fdo,
                status,
            );
        }
    } else {
        (*disk_data).FailurePredictionCapability = 0;
    }

    KeInitializeMutex(
        &raw mut (*disk_data).VerifyMutex,
        MAX_SECTORS_PER_VERIFY,
    );

    core::ptr::write_bytes(
        core::ptr::addr_of_mut!((*disk_data).FlushContext) as *mut u8,
        0,
        core::mem::size_of::<DISK_GROUP_CONTEXT>(),
    );
    InitializeListHead(&raw mut (*disk_data).FlushContext.CurrList);
    InitializeListHead(&raw mut (*disk_data).FlushContext.NextList);
    KeInitializeSpinLock(&raw mut (*disk_data).FlushContext.Spinlock);
    KeInitializeEvent(
        &raw mut (*disk_data).FlushContext.Event,
        _EVENT_TYPE_SynchronizationEvent,
        FALSE,
    );

    (*fdo_extension).SrbFlags = srb_flags;

    STATUS_SUCCESS
}








#[no_mangle]
pub unsafe extern "system" fn DiskStopDevice(
    DeviceObject: PDEVICE_OBJECT,
    Type: UCHAR,
) -> NTSTATUS {
    let _ = &DeviceObject;
    let _ = &Type;
    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn DiskCreateSymbolicLinks(
    DeviceObject: PDEVICE_OBJECT
) {
    let commonExtension: PCOMMON_DEVICE_EXTENSION = (*DeviceObject).DeviceExtension as PCOMMON_DEVICE_EXTENSION;
    let diskData: PDISK_DATA = (*commonExtension).DriverData as PDISK_DATA;
    let mut wideSourceName: [u16; 64] = [0u16; 64];
    let mut unicodeSourceName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut status: NTSTATUS = 0;

    // PAGED_CODE() expansion
    if KeGetCurrentIrql() > 1 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0"
                .as_ptr() as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0 == 0 {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0"
                    .as_ptr() as *const i8,
                b"drivers/storage/class/disk/pnp.c\0"
                    .as_ptr() as *const i8,
                711i32,
                b"0\0".as_ptr() as *const i8,
            );
            __int2c();
        }
    }

    // NT_ASSERT(commonExtension->DeviceName.Buffer)
    if (*commonExtension).DeviceName.Buffer.is_null() {
        DbgPrint(
            b"Assertion failed at %s(%d): %s\n\0"
                .as_ptr() as *const i8,
            b"drivers/storage/class/disk/pnp.c\0"
                .as_ptr() as *const i8,
            718i32,
            b"commonExtension->DeviceName.Buffer\0"
                .as_ptr() as *const i8,
        );
        __int2c();
    }

    if (*diskData).LinkStatus.WellKnownNameCreated() == 0 {
        let fmt1: [u8; 60] = *b"\\\0D\0e\0v\0i\0c\0e\0\\\0H\0a\0r\0d\0d\0i\0s\0k\0%\0d\0\\\0P\0a\0r\0t\0i\0t\0i\0o\0n\x000\0\0\0";
        // Bind the transmuted [u16] array so it OUTLIVES the RtlStringCchPrintfW call
        // below. Taking `.as_ptr()` on the transmute temporary directly leaves fmt1_ptr
        // dangling the instant the statement ends (dangling_pointers_from_temporaries).
        let fmt1_wide: [u16; 30] = core::mem::transmute::<[u8; 60], [u16; 30]>(fmt1);
        let fmt1_ptr: *const u16 = fmt1_wide.as_ptr();

        status = RtlStringCchPrintfW(
            wideSourceName.as_mut_ptr(),
            core::mem::size_of::<[u16; 64]>() / core::mem::size_of::<u16>() - 1 ,
            fmt1_ptr,
            (*(*commonExtension).PartitionZeroExtension).DeviceNumber,
        );

        if status >= 0 {
            RtlInitUnicodeString(
                &mut unicodeSourceName,
                wideSourceName.as_ptr() as *const u16,
            );
            TracePrint(
                4,
                2u32,
                b"DiskCreateSymbolicLink: Linking %wZ to %wZ\n\0"
                    .as_ptr() as *const i8,
                &unicodeSourceName as *const _ as *const core::ffi::c_void,
                &(*commonExtension).DeviceName as *const _ as *const core::ffi::c_void,
            );
            status = IoCreateSymbolicLink(
                &mut unicodeSourceName,
                &mut (*commonExtension).DeviceName,
            );
            if status >= 0 {
                (*diskData).LinkStatus.set_WellKnownNameCreated(1);
            }
        }
    }

    if (*diskData).LinkStatus.PhysicalDriveLinkCreated() == 0 {
        let fmt2: [u8; 56] = *b"\\\0D\0o\0s\0D\0e\0v\0i\0c\0e\0s\0\\\0P\0h\0y\0s\0i\0c\0a\0l\0D\0r\0i\0v\0e\0%\0d\0\0\0";
        // Bind the transmuted [u16] array so it outlives the RtlStringCchPrintfW call.
        let fmt2_wide: [u16; 28] = core::mem::transmute::<[u8; 56], [u16; 28]>(fmt2);
        let fmt2_ptr: *const u16 = fmt2_wide.as_ptr();

        status = RtlStringCchPrintfW(
            wideSourceName.as_mut_ptr(),
            core::mem::size_of::<[u16; 64]>() / core::mem::size_of::<u16>() - 1 ,
            fmt2_ptr,
            (*(*commonExtension).PartitionZeroExtension).DeviceNumber,
        );

        if status >= 0 {
            RtlInitUnicodeString(
                &mut unicodeSourceName,
                wideSourceName.as_ptr() as *const u16,
            );
            TracePrint(
                4,
                2u32,
                b"DiskCreateSymbolicLink: Linking %wZ to %wZ\n\0"
                    .as_ptr() as *const i8,
                &unicodeSourceName as *const _ as *const core::ffi::c_void,
                &(*commonExtension).DeviceName as *const _ as *const core::ffi::c_void,
            );
            status = IoCreateSymbolicLink(
                &mut unicodeSourceName,
                &mut (*commonExtension).DeviceName,
            );
            if status >= 0 {
                (*diskData).LinkStatus.set_PhysicalDriveLinkCreated(1);
            }
        }
    }
}


#[no_mangle]
pub unsafe extern "system" fn DiskRemoveDevice(
    DeviceObject: *mut DEVICE_OBJECT,
    Type: u8,
) -> i32 {
    let commonExtension: PCOMMON_DEVICE_EXTENSION =
        (*DeviceObject).DeviceExtension as PCOMMON_DEVICE_EXTENSION;
    let fdoExtension: PFUNCTIONAL_DEVICE_EXTENSION =
        (*DeviceObject).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;

    // PAGED_CODE() check
    if KeGetCurrentIrql() as i32 > (APC_LEVEL) as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0 as i32 == 0 {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0" as *const u8 as *const i8,
                b"drivers/storage/class/disk/pnp.c\0"
                    as *const u8 as *const i8,
                869 as i32,
                b"0\0" as *const u8 as *const i8,
            );
            __int2c();
        }
    }

    if Type as i32 == (IRP_MN_QUERY_REMOVE_DEVICE) as i32 ||
       Type as i32 == (IRP_MN_CANCEL_REMOVE_DEVICE) as i32
    {
        return STATUS_SUCCESS;
    }

    if !(*fdoExtension).DeviceDirectory.is_null() {
        ZwMakeTemporaryObject((*fdoExtension).DeviceDirectory);
        ZwClose((*fdoExtension).DeviceDirectory);
        (*fdoExtension).DeviceDirectory = core::ptr::null_mut();
    }

    if Type as i32 == (IRP_MN_REMOVE_DEVICE) as i32 {
        if !(*fdoExtension).SenseData.is_null() {
            ExFreePool((*fdoExtension).SenseData as *mut core::ffi::c_void);
            (*fdoExtension).SenseData = core::ptr::null_mut::<_SENSE_DATA>();
        }
        let fresh2 = &mut (*IoGetConfigurationInformation()).DiskCount;
        *fresh2 = (*fresh2).wrapping_sub(1);
    }

    DiskDeleteSymbolicLinks(DeviceObject);

    if Type as i32 == (IRP_MN_REMOVE_DEVICE) as i32 {
        ClassDeleteSrbLookasideList(commonExtension);
    }

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "system" fn DiskStartFdo(Fdo: PDEVICE_OBJECT) -> NTSTATUS {
    // Unknown constants
    const APC_LEVEL: i32 = 1;
    const IOCTL_STORAGE_GET_HOTPLUG_INFO: u32 = 0x002D0C14;
    const TRACE_FLAG_PNP: u32 = 1; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)
    const DiskWriteCacheDefault: i32 = 0;
    const DiskWriteCacheDisable: i32 = 1;
    const DiskWriteCacheEnable: i32 = 2;

    // Extern declarations for unknown symbols

    // Global string references (assumed extern)

    let fdoExtension: PFUNCTIONAL_DEVICE_EXTENSION = (*Fdo).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    let commonExtension: PCOMMON_DEVICE_EXTENSION = core::ptr::addr_of_mut!((*fdoExtension).__bindgen_anon_1.CommonExtension) as PCOMMON_DEVICE_EXTENSION;
    let diskData: PDISK_DATA = (*commonExtension).DriverData as PDISK_DATA;
    let mut hotplugInfo: STORAGE_HOTPLUG_INFO = unsafe { core::mem::zeroed() };
    let mut cacheInfo: DISK_CACHE_INFORMATION = unsafe { core::mem::zeroed() };
    let mut isPowerProtected: ULONG = 0;
    let mut status: NTSTATUS = 0;

    // PAGED_CODE() expansion
    {
        let current_irql = KeGetCurrentIrql();
        if current_irql > (APC_LEVEL) as u8 {
            DbgPrint(
                b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8,
                current_irql as i32,
            );
            if 0 == 0 {
                DbgPrint(
                    b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const i8,
                    b"drivers/storage/class/disk/pnp.c\0".as_ptr() as *const i8,
                    948i32,
                    b"0\0".as_ptr() as *const i8,
                );
                __int2c();
            }
        }
    }

    let mut irp: PIRP = core::ptr::null_mut();
    let mut event: KEVENT = unsafe { core::mem::zeroed() };
    let mut statusBlock: _IO_STATUS_BLOCK = unsafe { core::mem::zeroed() };

    KeInitializeEvent(&raw mut event, _EVENT_TYPE_SynchronizationEvent, 0_i32 as BOOLEAN);

    irp = IoBuildDeviceIoControlRequest(
        IOCTL_STORAGE_GET_HOTPLUG_INFO,
        Fdo,
        core::ptr::null_mut(),
        0u32,
        &raw mut hotplugInfo as *mut core::ffi::c_void,
        core::mem::size_of::<STORAGE_HOTPLUG_INFO>() as u32,
        0_i32 as BOOLEAN,
        &raw mut event,
        &raw mut statusBlock,
    );

    if !irp.is_null() {
        status = IoCallDriver(Fdo, irp);
        if status == STATUS_PENDING {
            KeWaitForSingleObject(
                &raw mut event as *mut core::ffi::c_void,
                _KWAIT_REASON_Executive,
                0i32 as KPROCESSOR_MODE,
                0_i32 as BOOLEAN,
                (core::ptr::null_mut::<LARGE_INTEGER>()).cast(),
            );
            status = statusBlock.__bindgen_anon_1.Status;
        }
        if !(status >= 0) {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const i8,
                b"drivers/storage/class/disk/pnp.c\0".as_ptr() as *const i8,
                988i32,
                b"NT_SUCCESS(status)\0".as_ptr() as *const i8,
            );
            __int2c();
        }
    }

    // Clear DEV_WRITE_CACHE flag
    (*fdoExtension).DeviceFlags = ((*fdoExtension).DeviceFlags as i32 & !0x1_i32) as u16;
    // ADJUST_FUA_FLAG
    if (*fdoExtension).DeviceFlags as i32 & 0x1_i32 != 0
        && !((*fdoExtension).DeviceFlags as i32 & 0x10_i32 != 0)
        && !((*fdoExtension).ScanForSpecialFlags as u64 & 0x80_u64 != 0)
    {
        (*fdoExtension).CdbForceUnitAccess = 1;
    } else {
        (*fdoExtension).CdbForceUnitAccess = 0;
    }

    (*diskData).WriteCacheOverride = DiskWriteCacheDefault;

    ClassGetDeviceParameter(
        fdoExtension,
        DiskDeviceParameterSubkey.as_ptr() as *mut u16,
        DiskDeviceUserWriteCacheSetting.as_ptr() as *mut u16,
        &raw mut (*diskData).WriteCacheOverride as *mut ULONG,
    );

    if (*diskData).WriteCacheOverride == DiskWriteCacheDefault {
        if (*fdoExtension).ScanForSpecialFlags as u64 & 0x10_u64 != 0 {
            StorDebugPrint(
                3i32,
                TRACE_FLAG_PNP,
                b"DiskStartFdo: Turning off write cache for %p due to a firmware issue\n\0".as_ptr() as PCCHAR,
                Fdo as *mut core::ffi::c_void,
            );
            (*diskData).WriteCacheOverride = DiskWriteCacheDisable;
        } else if hotplugInfo.DeviceHotplug as i32 != 0
            && hotplugInfo.WriteCacheEnableOverride == 0
        {
            StorDebugPrint(
                3i32,
                TRACE_FLAG_PNP,
                b"DiskStartFdo: Turning off write cache for %p due to hotpluggable device\n\0".as_ptr() as PCCHAR,
                Fdo as *mut core::ffi::c_void,
            );
            (*diskData).WriteCacheOverride = DiskWriteCacheDisable;
        } else if hotplugInfo.MediaHotplug != 0 {
            StorDebugPrint(
                3i32,
                TRACE_FLAG_PNP,
                b"DiskStartFdo: Turning off write cache for %p due to unlockable media\n\0".as_ptr() as PCCHAR,
                Fdo as *mut core::ffi::c_void,
            );
            (*diskData).WriteCacheOverride = DiskWriteCacheDisable;
        }
    }

    status = DiskGetCacheInformation(fdoExtension, &raw mut cacheInfo);
    if status >= 0 {
        if cacheInfo.WriteCacheEnabled as i32 == 1 {
            // SET_FLAG DEV_WRITE_CACHE
            (*fdoExtension).DeviceFlags = ((*fdoExtension).DeviceFlags as i32 | 0x1_i32) as u16;
            // ADJUST_FUA_FLAG
            if (*fdoExtension).DeviceFlags as i32 & 0x1_i32 != 0
                && !((*fdoExtension).DeviceFlags as i32 & 0x10_i32 != 0)
                && !((*fdoExtension).ScanForSpecialFlags as u64 & 0x80_u64 != 0)
            {
                (*fdoExtension).CdbForceUnitAccess = 1;
            } else {
                (*fdoExtension).CdbForceUnitAccess = 0;
            }
            if (*diskData).WriteCacheOverride == DiskWriteCacheDisable {
                cacheInfo.WriteCacheEnabled = 0;
                DiskSetCacheInformation(fdoExtension, &raw mut cacheInfo);
            }
        } else if (*diskData).WriteCacheOverride == DiskWriteCacheEnable {
            cacheInfo.WriteCacheEnabled = 1;
            DiskSetCacheInformation(fdoExtension, &raw mut cacheInfo);
        }
    }

    // Clear DEV_POWER_PROTECTED
    (*fdoExtension).DeviceFlags = ((*fdoExtension).DeviceFlags as i32 & !0x10_i32) as u16;

    ClassGetDeviceParameter(
        fdoExtension,
        DiskDeviceParameterSubkey.as_ptr() as *mut u16,
        DiskDeviceCacheIsPowerProtected.as_ptr() as *mut u16,
        &raw mut isPowerProtected,
    );
    if isPowerProtected == 1 {
        (*fdoExtension).DeviceFlags = ((*fdoExtension).DeviceFlags as i32 | 0x10_i32) as u16;
    }

    // ADJUST_FUA_FLAG again
    if (*fdoExtension).DeviceFlags as i32 & 0x1_i32 != 0
        && !((*fdoExtension).DeviceFlags as i32 & 0x10_i32 != 0)
        && !((*fdoExtension).ScanForSpecialFlags as u64 & 0x80_u64 != 0)
    {
        (*fdoExtension).CdbForceUnitAccess = 1;
    } else {
        (*fdoExtension).CdbForceUnitAccess = 0;
    }

    STATUS_SUCCESS
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*++
//
// Copyright (C) Microsoft Corporation, 1991 - 1999
//
// Module Name:
//
//     pnp.c
//
// Abstract:
//
//     SCSI disk class driver
//
// Environment:
//
//     kernel mode only
//
// Notes:
//
// Revision History:
//
// --*/
//
// #include "disk.h"
//
//
// #ifdef DEBUG_USE_WPP
// #include "pnp.tmh"
// #endif
//
// #ifndef __REACTOS__
// extern PULONG InitSafeBootMode;
// #else
// extern NTSYSAPI ULONG InitSafeBootMode;
// #endif
// ULONG diskDeviceSequenceNumber = 0;
// extern BOOLEAN DiskIsPastReinit;
//
//
// #ifdef ALLOC_PRAGMA
//
// #pragma alloc_text(PAGE, DiskAddDevice)
// #pragma alloc_text(PAGE, DiskInitFdo)
// #pragma alloc_text(PAGE, DiskStartFdo)
// #pragma alloc_text(PAGE, DiskGenerateDeviceName)
// #pragma alloc_text(PAGE, DiskCreateSymbolicLinks)
// #pragma alloc_text(PAGE, DiskDeleteSymbolicLinks)
// #pragma alloc_text(PAGE, DiskRemoveDevice)
// #endif
//
//
// NTSTATUS
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// DiskAddDevice(
//     IN PDRIVER_OBJECT DriverObject,
//     IN PDEVICE_OBJECT PhysicalDeviceObject
//     )
//
// /*++
//
// Routine Description:
//
//     This routine gets a port drivers capabilities, obtains the
//     inquiry data, searches the SCSI bus for the port driver and creates
//     the device objects for the disks found.
//
// Arguments:
//
//     DriverObject - Pointer to driver object created by system.
//
//     Pdo - Device object use to send requests to port driver.
//
// Return Value:
//
//     True is returned if one disk was found and successfully created.
//
// --*/
//
// {
//     ULONG rootPartitionMountable = FALSE;
//
//     PCONFIGURATION_INFORMATION configurationInformation;
//     ULONG diskCount;
//
//     NTSTATUS status;
//
//     PAGED_CODE();
//
//     //
//     // See if we should be allowing file systems to mount on partition zero.
//     //
//
//     TRY {
//         HANDLE deviceKey = NULL;
//
//         _UNICODE_STRING diskKeyName;
//         _OBJECT_ATTRIBUTES objectAttributes = {0};
//         HANDLE diskKey;
//
//         RTL_QUERY_REGISTRY_TABLE queryTable[2] = { 0 };
//
//         status = IoOpenDeviceRegistryKey(PhysicalDeviceObject,
//                                          PLUGPLAY_REGKEY_DEVICE,
//                                          KEY_READ,
//                                          &deviceKey);
//
//         if(!NT_SUCCESS(status)) {
//             TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_PNP, "DiskAddDevice: Error %#08lx opening device key "
//                            "for pdo %p\n",
//                         status, PhysicalDeviceObject));
//             LEAVE;
//         }
//
//         RtlInitUnicodeString(&diskKeyName, L"Disk");
//         InitializeObjectAttributes(&objectAttributes,
//                                    &diskKeyName,
//                                    OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
//                                    deviceKey,
//                                    NULL);
//
//         status = ZwOpenKey(&diskKey, KEY_READ, &objectAttributes);
//         ZwClose(deviceKey);
//
//         if(!NT_SUCCESS(status)) {
//             TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_PNP, "DiskAddDevice: Error %#08lx opening disk key "
//                            "for pdo %p device key %p\n",
//                         status, PhysicalDeviceObject, deviceKey));
//             LEAVE;
//         }
//
//         queryTable[0].Flags = RTL_QUERY_REGISTRY_DIRECT | RTL_QUERY_REGISTRY_TYPECHECK;
//         queryTable[0].Name = L"RootPartitionMountable";
//         queryTable[0].EntryContext = &(rootPartitionMountable);
//         queryTable[0].DefaultType = (REG_DWORD << RTL_QUERY_REGISTRY_TYPECHECK_SHIFT) | REG_NONE;
//
// #ifdef _MSC_VER
// #pragma prefast(suppress:6309, "We don't have QueryRoutine so Context doesn't make any sense")
// #endif
//         status = RtlQueryRegistryValues(RTL_REGISTRY_HANDLE,
//                                         diskKey,
//                                         queryTable,
//                                         NULL,
//                                         NULL);
//
//         if(!NT_SUCCESS(status)) {
//             TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_PNP, "DiskAddDevice: Error %#08lx reading value from "
//                            "disk key %p for pdo %p\n",
//                         status, diskKey, PhysicalDeviceObject));
//         }
//
//         ZwClose(diskKey);
//
//     } FINALLY {
//
//         //
//         // Do nothing.
//         //
//
//         if(!NT_SUCCESS(status)) {
//             TracePrint((TRACE_LEVEL_WARNING, TRACE_FLAG_PNP, "DiskAddDevice: Will %sallow file system to mount on "
//                            "partition zero of disk %p\n",
//                         (rootPartitionMountable ? "" : "not "),
//                         PhysicalDeviceObject));
//         }
//     }
//
//     //
//     // Create device objects for disk
//     //
//
//     diskCount = 0;
//
//     status = DiskCreateFdo(
//                  DriverObject,
//                  PhysicalDeviceObject,
//                  &diskCount,
//                  (BOOLEAN) !rootPartitionMountable
//                  );
//
//     //
//     // Get the number of disks already initialized.
//     //
//
//     configurationInformation = IoGetConfigurationInformation();
//
//     if (NT_SUCCESS(status)) {
//
//         //
//         // Increment system disk device count.
//         //
//
//         configurationInformation->DiskCount++;
//
//     }
//
//     return status;
//
// } // end DiskAddDevice()
//
//
// NTSTATUS
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// DiskInitFdo(
//     IN PDEVICE_OBJECT Fdo
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is called to do one-time initialization of new device objects
//
//
// Arguments:
//
//     Fdo - a pointer to the functional device object for this device
//
// Return Value:
//
//     status
//
// --*/
//
// {
//     PFUNCTIONAL_DEVICE_EXTENSION fdoExtension = Fdo->DeviceExtension;
//     PDISK_DATA diskData = (PDISK_DATA) fdoExtension->CommonExtension.DriverData;
//
//     ULONG srbFlags = 0;
//     ULONG timeOut = 0;
//     ULONG bytesPerSector;
//
//     PULONG dmSkew;
//
//     NTSTATUS status = STATUS_SUCCESS;
//
//     PAGED_CODE();
//
//     //
//     // Build the lookaside list for srb's for the physical disk. Should only
//     // need a couple.  If this fails then we don't have an emergency SRB so
//     // fail the call to initialize.
//     //
//
//     ClassInitializeSrbLookasideList((PCOMMON_DEVICE_EXTENSION) fdoExtension,
//                                     PARTITION0_LIST_SIZE);
//
//     if (fdoExtension->DeviceDescriptor->RemovableMedia)
//     {
//         SET_FLAG(Fdo->Characteristics, FILE_REMOVABLE_MEDIA);
//     }
//
//     //
//     // Initialize the srb flags.
//     //
//
//     //
//     // Because all requests share a common sense buffer, it is possible
//     // for the buffer to be overwritten if the port driver completes
//     // multiple failed requests that require a request sense before the
//     // class driver's completion routine can consume the data in the buffer.
//     // To prevent this, we allow the port driver to allocate a unique sense
//     // buffer each time it needs one.  We are responsible for freeing this
//     // buffer.  This also allows the adapter to be configured to support
//     // additional sense data beyond the minimum 18 bytes.
//     //
//
//     SET_FLAG(fdoExtension->SrbFlags, SRB_FLAGS_PORT_DRIVER_ALLOCSENSE);
//
//     if (fdoExtension->DeviceDescriptor->CommandQueueing &&
//         fdoExtension->AdapterDescriptor->CommandQueueing) {
//
//         SET_FLAG(fdoExtension->SrbFlags, SRB_FLAGS_QUEUE_ACTION_ENABLE);
//
//     }
//
//     //
//     // Look for controllers that require special flags.
//     //
//
//     ClassScanForSpecial(fdoExtension, DiskBadControllers, DiskSetSpecialHacks);
//
//     //
//     // Clear buffer for drive geometry.
//     //
//
//     RtlZeroMemory(&(fdoExtension->DiskGeometry),
//                   sizeof(DISK_GEOMETRY));
//
//     //
//     // Allocate request sense buffer.
//     //
//
//     fdoExtension->SenseData = ExAllocatePoolWithTag(NonPagedPoolNxCacheAligned,
//                                                     SENSE_BUFFER_SIZE_EX,
//                                                     DISK_TAG_START);
//
//     if (fdoExtension->SenseData == NULL) {
//
//         //
//         // The buffer can not be allocated.
//         //
//
//         TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_PNP, "DiskInitFdo: Can not allocate request sense buffer\n"));
//
//         status = STATUS_INSUFFICIENT_RESOURCES;
//         return status;
//     }
//
//     //
//     // Set the buffer size of SenseData
//     //
//
//     fdoExtension->SenseDataLength = SENSE_BUFFER_SIZE_EX;
//
//     //
//     // Physical device object will describe the entire
//     // device, starting at byte offset 0.
//     //
//
//     fdoExtension->CommonExtension.StartingOffset.QuadPart = (LONGLONG)(0);
//
//     //
//     // Set timeout value in seconds.
//     //
//     if ( (fdoExtension->MiniportDescriptor != NULL) &&
//          (fdoExtension->MiniportDescriptor->IoTimeoutValue > 0) ) {
//         //
//         // use the value set by Storport miniport driver
//         //
//         fdoExtension->TimeOutValue = fdoExtension->MiniportDescriptor->IoTimeoutValue;
//     } else {
//         //
//         // get timeout value from registry
//         //
//         timeOut = ClassQueryTimeOutRegistryValue(Fdo);
//
//         if (timeOut) {
//             fdoExtension->TimeOutValue = timeOut;
//         } else {
//             fdoExtension->TimeOutValue = SCSI_DISK_TIMEOUT;
//         }
//     }
//     //
//     // If this is a removable drive, build an entry in devicemap\scsi
//     // indicating it's physicaldriveN name, set up the appropriate
//     // update partitions routine and set the flags correctly.
//     // note: only do this after the timeout value is set, above.
//     //
//
//     if (TEST_FLAG(Fdo->Characteristics, FILE_REMOVABLE_MEDIA)) {
//
//         ClassUpdateInformationInRegistry( Fdo,
//                                           "PhysicalDrive",
//                                           fdoExtension->DeviceNumber,
//                                           NULL,
//                                           0);
//         //
//         // Enable media change notification for removable disks
//         //
//         ClassInitializeMediaChangeDetection(fdoExtension,
//                                             (PUCHAR)"Disk");
//
//     } else {
//
//         SET_FLAG(fdoExtension->DeviceFlags, DEV_SAFE_START_UNIT);
//         SET_FLAG(fdoExtension->SrbFlags, SRB_FLAGS_NO_QUEUE_FREEZE);
//
//     }
//
//     //
//     // The commands we send during the init could cause the flags to change
//     // in case of any error.  Save the SRB flags locally and restore it at
//     // the end of this function, so that the class driver can get it.
//     //
//
//     srbFlags = fdoExtension->SrbFlags;
//
//
//     //
//     // Read the drive capacity.  Don't use the disk version of the routine here
//     // since we don't know the disk signature yet - the disk version will
//     // attempt to determine the BIOS reported geometry.
//     //
//
//     (VOID)ClassReadDriveCapacity(Fdo);
//
//     //
//     // Set up sector size fields.
//     //
//     // Stack variables will be used to update
//     // the partition device extensions.
//     //
//     // The device extension field SectorShift is
//     // used to calculate sectors in I/O transfers.
//     //
//     // The DiskGeometry structure is used to service
//     // IOCTls used by the format utility.
//     //
//
//     bytesPerSector = fdoExtension->DiskGeometry.BytesPerSector;
//
//     //
//     // Make sure sector size is not zero.
//     //
//
//     if (bytesPerSector == 0) {
//
//         //
//         // Default sector size for disk is 512.
//         //
//
//         bytesPerSector = fdoExtension->DiskGeometry.BytesPerSector = 512;
//         fdoExtension->SectorShift = 9;
//     }
//
//     //
//     // Determine is DM Driver is loaded on an IDE drive that is
//     // under control of Atapi - this could be either a crashdump or
//     // an Atapi device is sharing the controller with an IDE disk.
//     //
//
//     HalExamineMBR(fdoExtension->CommonExtension.DeviceObject,
//                   fdoExtension->DiskGeometry.BytesPerSector,
//                   (ULONG)0x54,
//                   (PVOID *)&dmSkew);
//
//     if (dmSkew) {
//
//         //
//         // Update the device extension, so that the call to IoReadPartitionTable
//         // will get the correct information. Any I/O to this disk will have
//         // to be skewed by *dmSkew sectors aka DMByteSkew.
//         //
//
//         fdoExtension->DMSkew     = *dmSkew;
//         fdoExtension->DMActive   = TRUE;
//         fdoExtension->DMByteSkew = fdoExtension->DMSkew * bytesPerSector;
//
//         FREE_POOL(dmSkew);
//     }
//
// #if defined(_X86_) || defined(_AMD64_)
//
//     //
//     // Try to read the signature off the disk and determine the correct drive
//     // geometry based on that.  This requires rereading the disk size to get
//     // the cylinder count updated correctly.
//     //
//
//     if(!TEST_FLAG(Fdo->Characteristics, FILE_REMOVABLE_MEDIA)) {
//
//         DiskReadSignature(Fdo);
//         DiskReadDriveCapacity(Fdo);
//
//         if (diskData->GeometrySource == DiskGeometryUnknown)
//         {
//             //
//             // Neither the  BIOS  nor the port driver could provide us with a  reliable
//             // geometry.  Before we use the default,  look to see if it was partitioned
//             // under Windows NT4 [or earlier] and apply the one that was used back then
//             //
//
//             if (DiskIsNT4Geometry(fdoExtension))
//             {
//                 diskData->RealGeometry = fdoExtension->DiskGeometry;
//                 diskData->RealGeometry.SectorsPerTrack   = 0x20;
//                 diskData->RealGeometry.TracksPerCylinder = 0x40;
//                 fdoExtension->DiskGeometry = diskData->RealGeometry;
//
//                 diskData->GeometrySource = DiskGeometryFromNT4;
//             }
//         }
//     }
//
// #endif
//
//     DiskCreateSymbolicLinks(Fdo);
//
//     //
//     // Get the SCSI address if it's available for use with SMART ioctls.
//     // SMART ioctls are used for failure prediction, so we need to get
//     // the SCSI address before initializing failure prediction.
//     //
//
//     {
//         PIRP irp;
//         KEVENT event;
//         _IO_STATUS_BLOCK statusBlock = { 0 };
//
//         KeInitializeEvent(&event, SynchronizationEvent, FALSE);
//
//         irp = IoBuildDeviceIoControlRequest(IOCTL_SCSI_GET_ADDRESS,
//                                             fdoExtension->CommonExtension.LowerDeviceObject,
//                                             NULL,
//                                             0L,
//                                             &(diskData->ScsiAddress),
//                                             sizeof(SCSI_ADDRESS),
//                                             FALSE,
//                                             &event,
//                                             &statusBlock);
//
//         status = STATUS_UNSUCCESSFUL;
//
//         if(irp != NULL) {
//
//             status = IoCallDriver(fdoExtension->CommonExtension.LowerDeviceObject, irp);
//
//             if(status == STATUS_PENDING) {
//                 KeWaitForSingleObject(&event,
//                                       Executive,
//                                       KernelMode,
//                                       FALSE,
//                                       NULL);
//                 status = statusBlock.Status;
//             }
//         }
//     }
//
//     //
//     // Determine the type of disk and enable failure prediction in the hardware
//     // and enable failure prediction polling.
//     //
//
//     if (InitSafeBootMode == 0) // __REACTOS__
//     {
//         DiskDetectFailurePrediction(fdoExtension,
//                                     &diskData->FailurePredictionCapability,
//                                     NT_SUCCESS(status));
//
//         if (diskData->FailurePredictionCapability != FailurePredictionNone)
//         {
//             //
//             // Cool, we've got some sort of failure prediction, enable it
//             // at the hardware and then enable polling for it
//             //
//
//             //
//             // By default we allow performance to be degradeded if failure
//             // prediction is enabled.
//             //
//
//             diskData->AllowFPPerfHit = TRUE;
//
//             //
//             // Enable polling only after Atapi and SBP2 add support for the new
//             // SRB flag that indicates that the request should not reset the
//             // drive spin down idle timer.
//             //
//
//             status = DiskEnableDisableFailurePredictPolling(fdoExtension,
//                                           TRUE,
//                                           DISK_DEFAULT_FAILURE_POLLING_PERIOD);
//
//             TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_PNP, "DiskInitFdo: Failure Prediction Poll enabled as "
//                            "%d for device %p, Status = %lx\n",
//                      diskData->FailurePredictionCapability,
//                      Fdo,
//                      status));
//         }
//     } else {
//
//         //
//         // In safe boot mode we do not enable failure prediction, as perhaps
//         // it is the reason why normal boot does not work
//         //
//
//         diskData->FailurePredictionCapability = FailurePredictionNone;
//
//     }
//
//     //
//     // Initialize the verify mutex
//     //
//
//     KeInitializeMutex(&diskData->VerifyMutex, MAX_SECTORS_PER_VERIFY);
//
//     //
//     // Initialize the flush group context
//     //
//
//     RtlZeroMemory(&diskData->FlushContext, sizeof(DISK_GROUP_CONTEXT));
//
//     InitializeListHead(&diskData->FlushContext.CurrList);
//     InitializeListHead(&diskData->FlushContext.NextList);
//
//     KeInitializeSpinLock(&diskData->FlushContext.Spinlock);
//     KeInitializeEvent(&diskData->FlushContext.Event, SynchronizationEvent, FALSE);
//
//
//     //
//     // Restore the saved value
//     //
//     fdoExtension->SrbFlags = srbFlags;
//
//     return STATUS_SUCCESS;
//
// } // end DiskInitFdo()
//
// NTSTATUS
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// DiskStopDevice(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN UCHAR Type
//     )
//
// {
//     UNREFERENCED_PARAMETER(DeviceObject);
//     UNREFERENCED_PARAMETER(Type);
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// DiskGenerateDeviceName(
//     IN ULONG DeviceNumber,
//     OUT PCCHAR *RawName
//     )
//
// /*++
//
// Routine Description:
//
//     This routine will allocate a unicode string buffer and then fill it in
//     with a generated name for the specified device object.
//
//     It is the responsibility of the user to allocate a _UNICODE_STRING structure
//     to pass in and to free UnicodeName->Buffer when done with it.
//
// Arguments:
//
//     DeviceObject - a pointer to the device object
//
//     UnicodeName - a unicode string to put the name buffer into
//
// Return Value:
//
//     status
//
// --*/
//
// #define FDO_NAME_FORMAT "\\Device\\Harddisk%d\\DR%d"
//
// {
//     CHAR rawName[64] = { 0 };
//     NTSTATUS status;
//
//     PAGED_CODE();
//
//         status = RtlStringCchPrintfA(rawName, sizeof(rawName) - 1, FDO_NAME_FORMAT, DeviceNumber,
//                                     diskDeviceSequenceNumber++);
//         if (!NT_SUCCESS(status)) {
//             TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_PNP, "DiskGenerateDeviceName: Format FDO name failed with error: 0x%X\n", status));
//             return status;
//         }
//
//     *RawName = ExAllocatePoolWithTag(ntkrnl_sys::ddk::_POOL_TYPE_PagedPool,
//                                      strlen(rawName) + 1,
//                                      DISK_TAG_NAME);
//
//     if(*RawName == NULL) {
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     status = RtlStringCchCopyA(*RawName, strlen(rawName) + 1, rawName);
//     if (!NT_SUCCESS(status)) {
//         TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_PNP, "DiskGenerateDeviceName: Device name copy failed with error: 0x%X\n", status));
//         FREE_POOL(*RawName);
//         return status;
//     }
//
//     TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_PNP, "DiskGenerateDeviceName: generated \"%s\"\n", rawName));
//
//     return STATUS_SUCCESS;
// }
//
//
// VOID
// DiskCreateSymbolicLinks(
//     IN PDEVICE_OBJECT DeviceObject
//     )
//
// /*++
//
// Routine Description:
//
//     This routine will generate a symbolic link for the specified device object
//     using the well known form \\Device\HarddiskX\PartitionY, where X and Y is
//     always 0 which represents the entire disk object.
//
//     This routine will not try to delete any previous symbolic link for the
//     same generated name - the caller must make sure the symbolic link has
//     been broken before calling this routine.
//
// Arguments:
//
//     DeviceObject - the device object to make a well known name for
//
// Return Value:
//
//     STATUS
//
// --*/
//
// {
//     PCOMMON_DEVICE_EXTENSION commonExtension = DeviceObject->DeviceExtension;
//     PDISK_DATA diskData = commonExtension->DriverData;
//
//     WCHAR wideSourceName[64] = { 0 };
//     _UNICODE_STRING unicodeSourceName;
//
//     NTSTATUS status;
//
//     PAGED_CODE();
//
//     //
//     // Build the destination for the link first using the device name
//     // stored in the device object
//     //
//
//     NT_ASSERT(commonExtension->DeviceName.Buffer);
//
//     if(!diskData->LinkStatus.WellKnownNameCreated) {
//         //
//         // Put together the source name using the partition and device number
//         // in the device extension and disk data segment
//         //
//
//         status = RtlStringCchPrintfW(wideSourceName, sizeof(wideSourceName) / sizeof(wideSourceName[0]) - 1,
//                                      L"\\Device\\Harddisk%d\\Partition0",
//                                      commonExtension->PartitionZeroExtension->DeviceNumber);
//
//         if (NT_SUCCESS(status)) {
//
//             RtlInitUnicodeString(&unicodeSourceName, wideSourceName);
//
//             TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_PNP, "DiskCreateSymbolicLink: Linking %wZ to %wZ\n",
//                        &unicodeSourceName,
//                        &commonExtension->DeviceName));
//
//             status = IoCreateSymbolicLink(&unicodeSourceName,
//                                           &commonExtension->DeviceName);
//
//             if(NT_SUCCESS(status)){
//                 diskData->LinkStatus.WellKnownNameCreated = TRUE;
//             }
//         }
//     }
//
//     if (!diskData->LinkStatus.PhysicalDriveLinkCreated) {
//
//         //
//         // Create a physical drive N link using the device number we saved
//         // away during AddDevice.
//         //
//
//         status = RtlStringCchPrintfW(wideSourceName, sizeof(wideSourceName) / sizeof(wideSourceName[0]) - 1,
//                                      L"\\DosDevices\\PhysicalDrive%d",
//                                      commonExtension->PartitionZeroExtension->DeviceNumber);
//         if (NT_SUCCESS(status)) {
//
//             RtlInitUnicodeString(&unicodeSourceName, wideSourceName);
//
//             TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_PNP, "DiskCreateSymbolicLink: Linking %wZ to %wZ\n",
//                         &unicodeSourceName,
//                         &(commonExtension->DeviceName)));
//
//             status = IoCreateSymbolicLink(&unicodeSourceName,
//                                           &(commonExtension->DeviceName));
//
//             if(NT_SUCCESS(status)) {
//                 diskData->LinkStatus.PhysicalDriveLinkCreated = TRUE;
//             }
//         }
//     }
//
//
//     return;
// }
//
//
// VOID
// DiskDeleteSymbolicLinks(
//     IN PDEVICE_OBJECT DeviceObject
//     )
//
// /*++
//
// Routine Description:
//
//     This routine will delete the well known name (symlink) for the specified
//     device.  It generates the link name using information stored in the
//     device extension
//
// Arguments:
//
//     DeviceObject - the device object we are unlinking
//
// Return Value:
//
//     status
//
// --*/
//
// {
//     PCOMMON_DEVICE_EXTENSION commonExtension = DeviceObject->DeviceExtension;
//     PDISK_DATA diskData = commonExtension->DriverData;
//
//     WCHAR wideLinkName[64] = { 0 };
//     _UNICODE_STRING unicodeLinkName;
//     NTSTATUS status;
//
//     PAGED_CODE();
//
//     if(diskData->LinkStatus.WellKnownNameCreated) {
//
//         status = RtlStringCchPrintfW(wideLinkName, sizeof(wideLinkName) / sizeof(wideLinkName[0]) - 1,
//                                     L"\\Device\\Harddisk%d\\Partition0",
//                                     commonExtension->PartitionZeroExtension->DeviceNumber);
//         if (NT_SUCCESS(status)) {
//             RtlInitUnicodeString(&unicodeLinkName, wideLinkName);
//             IoDeleteSymbolicLink(&unicodeLinkName);
//         }
//         diskData->LinkStatus.WellKnownNameCreated = FALSE;
//     }
//
//     if(diskData->LinkStatus.PhysicalDriveLinkCreated) {
//
//         status = RtlStringCchPrintfW(wideLinkName, sizeof(wideLinkName) / sizeof(wideLinkName[0]) - 1,
//                                     L"\\DosDevices\\PhysicalDrive%d",
//                                     commonExtension->PartitionZeroExtension->DeviceNumber);
//         if (NT_SUCCESS(status)) {
//             RtlInitUnicodeString(&unicodeLinkName, wideLinkName);
//             IoDeleteSymbolicLink(&unicodeLinkName);
//         }
//         diskData->LinkStatus.PhysicalDriveLinkCreated = FALSE;
//     }
//
//
//     return;
// }
//
//
// NTSTATUS
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// DiskRemoveDevice(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN UCHAR Type
//     )
//
// /*++
//
// Routine Description:
//
//     This routine will release any resources the device may have allocated for
//     this device object and return.
//
// Arguments:
//
//     DeviceObject - the device object being removed
//
// Return Value:
//
//     status
//
// --*/
//
// {
//     PCOMMON_DEVICE_EXTENSION commonExtension = DeviceObject->DeviceExtension;
//     PFUNCTIONAL_DEVICE_EXTENSION fdoExtension = DeviceObject->DeviceExtension;
//
//     PAGED_CODE();
//
//     //
//     // Handle query and cancel
//     //
//
//     if((Type == IRP_MN_QUERY_REMOVE_DEVICE) ||
//        (Type == IRP_MN_CANCEL_REMOVE_DEVICE)) {
//         return STATUS_SUCCESS;
//     }
//
//     //
//     // Delete our object directory.
//     //
//
//     if(fdoExtension->DeviceDirectory != NULL) {
//         ZwMakeTemporaryObject(fdoExtension->DeviceDirectory);
//         ZwClose(fdoExtension->DeviceDirectory);
//         fdoExtension->DeviceDirectory = NULL;
//     }
//
//     if(Type == IRP_MN_REMOVE_DEVICE) {
//
//         FREE_POOL(fdoExtension->SenseData);
//
//         IoGetConfigurationInformation()->DiskCount--;
//
//     }
//
//     DiskDeleteSymbolicLinks(DeviceObject);
//
//     if (Type == IRP_MN_REMOVE_DEVICE)
//     {
//         ClassDeleteSrbLookasideList(commonExtension);
//     }
//
//     return STATUS_SUCCESS;
// }
//
//
// NTSTATUS
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// DiskStartFdo(
//     IN PDEVICE_OBJECT Fdo
//     )
//
// /*++
//
// Routine Description:
//
//     This routine will query the underlying device for any information necessary
//     to complete initialization of the device.  This will include physical
//     disk geometry, mode sense information and such.
//
//     This routine does not perform partition enumeration - that is left to the
//     re-enumeration routine
//
//     If this routine fails it will return an error value.  It does not clean up
//     any resources - that is left for the Stop/Remove routine.
//
// Arguments:
//
//     Fdo - a pointer to the functional device object for this device
//
// Return Value:
//
//     status
//
// --*/
//
// {
//     PFUNCTIONAL_DEVICE_EXTENSION fdoExtension = Fdo->DeviceExtension;
//     PCOMMON_DEVICE_EXTENSION commonExtension = &(fdoExtension->CommonExtension);
//     PDISK_DATA diskData = commonExtension->DriverData;
//     STORAGE_HOTPLUG_INFO hotplugInfo = { 0 };
//     DISK_CACHE_INFORMATION cacheInfo = { 0 };
//     ULONG isPowerProtected = 0;
//     NTSTATUS status;
//
//     PAGED_CODE();
//
//     //
//     // Get the hotplug information, so we can turn off write cache if needed
//     //
//     // NOTE: Capabilities info is not good enough to determine hotplugedness
//     //       as  we cannot determine device relations  information and other
//     //       dependencies. Get the hotplug info instead
//     //
//
//     {
//         PIRP irp;
//         KEVENT event;
//         _IO_STATUS_BLOCK statusBlock = { 0 };
//
//         KeInitializeEvent(&event, SynchronizationEvent, FALSE);
//
//         irp = IoBuildDeviceIoControlRequest(IOCTL_STORAGE_GET_HOTPLUG_INFO,
//                                             Fdo,
//                                             NULL,
//                                             0L,
//                                             &hotplugInfo,
//                                             sizeof(STORAGE_HOTPLUG_INFO),
//                                             FALSE,
//                                             &event,
//                                             &statusBlock);
//
//         if (irp != NULL) {
//
//             // send to self -- classpnp handles this
//             status = IoCallDriver(Fdo, irp);
//             if (status == STATUS_PENDING) {
//                 KeWaitForSingleObject(&event,
//                                       Executive,
//                                       KernelMode,
//                                       FALSE,
//                                       NULL);
//
//                 status = statusBlock.Status;
//             }
//             NT_ASSERT(NT_SUCCESS(status));
//         }
//     }
//
//     //
//     // Clear the DEV_WRITE_CACHE flag now  and set
//     // it below only if we read that from the disk
//     //
//     CLEAR_FLAG(fdoExtension->DeviceFlags, DEV_WRITE_CACHE);
//     ADJUST_FUA_FLAG(fdoExtension);
//
//     diskData->WriteCacheOverride = DiskWriteCacheDefault;
//
//     //
//     // Look into the registry to  see if the user
//     // has chosen to override the default setting
//     //
//     ClassGetDeviceParameter(fdoExtension,
//                             DiskDeviceParameterSubkey,
//                             DiskDeviceUserWriteCacheSetting,
//                             (PULONG)&diskData->WriteCacheOverride);
//
//     if (diskData->WriteCacheOverride == DiskWriteCacheDefault)
//     {
//         //
//         // The user has not overridden the default settings
//         //
//         if (TEST_FLAG(fdoExtension->ScanForSpecialFlags, CLASS_SPECIAL_DISABLE_WRITE_CACHE))
//         {
//             //
//             // This flag indicates that we have faulty firmware and this
//             // may cause the filesystem to refuse to mount on this media
//             //
//             TracePrint((TRACE_LEVEL_WARNING, TRACE_FLAG_PNP, "DiskStartFdo: Turning off write cache for %p due to a firmware issue\n", Fdo));
//
//             diskData->WriteCacheOverride = DiskWriteCacheDisable;
//         }
//         else if (hotplugInfo.DeviceHotplug && !hotplugInfo.WriteCacheEnableOverride)
//         {
//             //
//             // This flag indicates that the device is hotpluggable making it unsafe to enable caching
//             //
//             TracePrint((TRACE_LEVEL_WARNING, TRACE_FLAG_PNP, "DiskStartFdo: Turning off write cache for %p due to hotpluggable device\n", Fdo));
//
//             diskData->WriteCacheOverride = DiskWriteCacheDisable;
//         }
//         else if (hotplugInfo.MediaHotplug)
//         {
//             //
//             // This flag indicates that the media in the device cannot be reliably locked
//             //
//             TracePrint((TRACE_LEVEL_WARNING, TRACE_FLAG_PNP, "DiskStartFdo: Turning off write cache for %p due to unlockable media\n", Fdo));
//
//             diskData->WriteCacheOverride = DiskWriteCacheDisable;
//         }
//         else
//         {
//             //
//             // Even though the device does  not seem to have any obvious problems
//             // we leave it to the user to modify the previous write cache setting
//             //
//         }
//     }
//
//     //
//     // Query the disk to see if write cache is enabled
//     // and  set the DEV_WRITE_CACHE flag appropriately
//     //
//
//     status = DiskGetCacheInformation(fdoExtension, &cacheInfo);
//
//     if (NT_SUCCESS(status))
//     {
//         if (cacheInfo.WriteCacheEnabled == TRUE)
//         {
//             SET_FLAG(fdoExtension->DeviceFlags, DEV_WRITE_CACHE);
//             ADJUST_FUA_FLAG(fdoExtension);
//
//             if (diskData->WriteCacheOverride == DiskWriteCacheDisable)
//             {
//                 //
//                 // Write cache is currently enabled on this
//                 // device, but we would like to turn it off
//                 //
//                 cacheInfo.WriteCacheEnabled = FALSE;
//
//                 DiskSetCacheInformation(fdoExtension, &cacheInfo);
//             }
//         }
//         else
//         {
//             if (diskData->WriteCacheOverride == DiskWriteCacheEnable)
//             {
//                 //
//                 // Write cache is currently disabled on this
//                 // device, but we  would  like to turn it on
//                 //
//                 cacheInfo.WriteCacheEnabled = TRUE;
//
//                 DiskSetCacheInformation(fdoExtension, &cacheInfo);
//             }
//         }
//     }
//
//     //
//     // Query the registry to see if this disk is power-protected or not
//     //
//
//     CLEAR_FLAG(fdoExtension->DeviceFlags, DEV_POWER_PROTECTED);
//
//     ClassGetDeviceParameter(fdoExtension, DiskDeviceParameterSubkey, DiskDeviceCacheIsPowerProtected, &isPowerProtected);
//
//     if (isPowerProtected == 1)
//     {
//         SET_FLAG(fdoExtension->DeviceFlags, DEV_POWER_PROTECTED);
//     }
//
//     ADJUST_FUA_FLAG(fdoExtension);
//
//     return STATUS_SUCCESS;
//
// } // end DiskStartFdo()
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

// DiskDeleteSymbolicLinks — re-integrated from the translation cache
// (tools/rustconv/.responses/disk__pnp/DiskDeleteSymbolicLinks.rs); the stitcher
// dropped it from the assembled file. Deletes the \Device\HarddiskN\Partition0
// well-known link and the \DosDevices\PhysicalDriveN link. (pnp.c:780)
#[no_mangle]
pub unsafe extern "C" fn DiskDeleteSymbolicLinks(DeviceObject: PDEVICE_OBJECT) {
    let commonExtension: PCOMMON_DEVICE_EXTENSION =
        (*DeviceObject).DeviceExtension as PCOMMON_DEVICE_EXTENSION;
    let diskData: PDISK_DATA = (*commonExtension).DriverData as PDISK_DATA;
    let mut wideLinkName: [u16; 64] = [0u16; 64];
    let mut unicodeLinkName: ntkrnl_sys::ddk::UNICODE_STRING = core::mem::zeroed();
    let mut status: NTSTATUS;

    if (*diskData).LinkStatus.WellKnownNameCreated() != 0 {
        status = RtlStringCchPrintfW(
            wideLinkName.as_mut_ptr(),
            (core::mem::size_of::<[u16; 64]>() / core::mem::size_of::<u16>()).wrapping_sub(1),
            core::mem::transmute::<&[u8; 60], &[u16; 30]>(
                b"\\\0D\0e\0v\0i\0c\0e\0\\\0H\0a\0r\0d\0d\0i\0s\0k\0%\0d\0\\\0P\0a\0r\0t\0i\0t\0i\0o\0n\x000\0\0\0"
            ).as_ptr(),
            (*(*commonExtension).PartitionZeroExtension).DeviceNumber,
        );
        if status >= 0 {
            RtlInitUnicodeString(&raw mut unicodeLinkName, wideLinkName.as_ptr());
            IoDeleteSymbolicLink(&raw mut unicodeLinkName);
        }
        (*diskData).LinkStatus.set_WellKnownNameCreated(0u32);
    }

    if (*diskData).LinkStatus.PhysicalDriveLinkCreated() != 0 {
        status = RtlStringCchPrintfW(
            wideLinkName.as_mut_ptr(),
            (core::mem::size_of::<[u16; 64]>() / core::mem::size_of::<u16>()).wrapping_sub(1),
            core::mem::transmute::<&[u8; 56], &[u16; 28]>(
                b"\\\0D\0o\0s\0D\0e\0v\0i\0c\0e\0s\0\\\0P\0h\0y\0s\0i\0c\0a\0l\0D\0r\0i\0v\0e\0%\0d\0\0\0"
            ).as_ptr(),
            (*(*commonExtension).PartitionZeroExtension).DeviceNumber,
        );
        if status >= 0 {
            RtlInitUnicodeString(&raw mut unicodeLinkName, wideLinkName.as_ptr());
            IoDeleteSymbolicLink(&raw mut unicodeLinkName);
        }
        (*diskData).LinkStatus.set_PhysicalDriveLinkCreated(0u32);
    }
}
