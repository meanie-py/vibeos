// geometry.c is guarded by #if defined(_X86_) || defined(_AMD64_) in the C source.
// The DISK_GEOMETRY_SOURCE type and GeometrySource/RealGeometry fields in _DISK_DATA
// are x86/amd64-only; arm64 bindgen omits them. Gate the entire module accordingly.
#![cfg(any(target_arch = "x86", target_arch = "x86_64"))]

use crate::types::*;
use crate::glue::*;
use core::ffi::{c_char, c_int, c_long, c_uchar, c_uint, c_ushort};
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{CM_FULL_RESOURCE_DESCRIPTOR, CM_INT13_DRIVE_PARAMETER, DISK_GEOMETRY, DISK_SIGNATURE, _KWAIT_REASON_Executive, _IO_STATUS_BLOCK, IoAllocateIrp, IoBuildSynchronousFsdRequest, IoFreeIrp, IoGetNextIrpStackLocation, IoReadDiskSignature, IoSetCompletionRoutine, KEVENT, KeInitializeEvent, KeWaitForSingleObject, LARGE_INTEGER, _EVENT_TYPE_NotificationEvent, _OBJECT_ATTRIBUTES, PCM_FULL_RESOURCE_DESCRIPTOR, PCM_INT13_DRIVE_PARAMETER, PCM_PARTIAL_RESOURCE_DESCRIPTOR, PDEVICE_OBJECT, PDISK_DETECTION_INFO, PDISK_GEOMETRY, PDRIVER_OBJECT, PIO_STACK_LOCATION, PIRP, PKEY_VALUE_FULL_INFORMATION, PPARTITION_DESCRIPTOR, RtlInitUnicodeString, RtlUnicodeStringToInteger, _EVENT_TYPE_SynchronizationEvent, ZwClose, ZwOpenKey, ZwQueryValueKey};
use ntkrnl_sys::ddk::{_KEY_VALUE_INFORMATION_CLASS_KeyValueFullInformation as KeyValueFullInformation, _PARTITION_STYLE_PARTITION_STYLE_MBR as PARTITION_STYLE_MBR, _PARTITION_STYLE_PARTITION_STYLE_GPT as PARTITION_STYLE_GPT};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{PFUNCTIONAL_DEVICE_EXTENSION, _DISK_GEOMETRY_SOURCE_DiskGeometryFromDefault as DiskGeometryFromDefault, _DISK_GEOMETRY_SOURCE_DiskGeometryGuessedFromBios as DiskGeometryGuessedFromBios, _DISK_GEOMETRY_SOURCE_DiskGeometryUnknown as DiskGeometryUnknown};
use ntkrnl_sys::ddk::{CM_PARTIAL_RESOURCE_DESCRIPTOR, DEVICE_OBJECT, DRIVER_OBJECT, IO_STACK_LOCATION, IRP, KEY_VALUE_FULL_INFORMATION, PCUNICODE_STRING, _KEY_VALUE_FULL_INFORMATION, _UNICODE_STRING};
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{FUNCTIONAL_DEVICE_EXTENSION};
use ntkrnl_sys::ddk::{KeGetCurrentIrql};
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{_FUNCTIONAL_DEVICE_EXTENSION};
// resolve_imports.py: explicit kernel/private-type imports
use ntkrnl_sys::ddk::{CCHAR, DISK_DETECTION_INFO};
// resolve_imports.py: explicit kernel/private-type imports
use ntkrnl_sys::ddk::{ExFreePool, KPROCESSOR_MODE};

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


extern "C" {
    fn TracePrint(level: i32, flag: u32, format: *const i8, ...);
    fn DbgPrint(format: *const i8, ...);
    fn StorDebugPrint(level: i32, flags: i32, format: *const i8, ...);
}
#[no_mangle]
pub unsafe extern "C" fn __int2c() {
    // debug-break intrinsic stub — empty body is correct
}

extern "C" {
    fn RtlStringCchPrintfW(pszDest: *mut u16, cchDest: usize, pszFormat: *const u16, ...) -> NTSTATUS;
}

// Private DISK_DETECT_INFO struct (file-local to geometry.c — not the public DISK_DETECTION_INFO).
// C source: typedef struct _DISK_DETECT_INFO { BOOLEAN Initialized; ULONG Style; ULONG Signature;
//   ULONG MbrCheckSum; PDEVICE_OBJECT Device; CM_INT13_DRIVE_PARAMETER DriveParameters; }
#[repr(C)]
pub struct DISK_DETECT_INFO {
    pub Initialized: ntkrnl_sys::ddk::BOOLEAN,
    pub Style: ntkrnl_sys::ddk::ULONG,
    pub Signature: ntkrnl_sys::ddk::ULONG,
    pub MbrCheckSum: ntkrnl_sys::ddk::ULONG,
    pub Device: ntkrnl_sys::ddk::PDEVICE_OBJECT,
    pub DriveParameters: CM_INT13_DRIVE_PARAMETER,
}
pub type PDISK_DETECT_INFO = *mut DISK_DETECT_INFO;

pub const NUM_PARTITION_TABLE_ENTRIES: u32 = 4;
pub const PARTITION_TABLE_OFFSET: usize = 0x1be / 2;
pub const BOOT_SIGNATURE_OFFSET: usize = (0x200 / 2) - 1;
pub const BOOT_RECORD_SIGNATURE: u16 = 0xaa55;

// geometry.c file-scope globals (BIOS Int13 disk-detection state). `static mut`
// matches the C mutable file-statics; no Sync needed.
#[no_mangle]
pub static mut DetectInfoList: PDISK_DETECT_INFO = core::ptr::null_mut();
#[no_mangle]
pub static mut DetectInfoCount: ULONG = 0;
#[no_mangle]
pub static mut DetectInfoUsedCount: LONG = 0;


const APC_LEVEL: u32 = 1;

const SL_OVERRIDE_VERIFY_VOLUME: i32 = 0x02;
const NonPagedPool: i32 = 0;
const DISK_TAG_UPDATE_GEOM: u32 = 0x67446353; // placeholder
const PARTITION_EXTENDED: u8 = 0x05;
const PARTITION_XINT13_EXTENDED: u8 = 0x0F;
const PARTITION_LDM: u8 = 0x42;

const TRACE_LEVEL_INFORMATION: i32 = 4; // storswtr.h:26-30
const TRACE_FLAG_GENERAL: u32 = 0; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)
const DetectInt13: i32 = 1;

type KIRQL = u8;

const TRACE_LEVEL_ERROR: i32 = 2; // storswtr.h:26-30

// Forward declarations for functions/constants not in the import list


#[no_mangle]
pub unsafe extern "C" fn DiskSaveDetectInfo(
    DriverObject: *mut DRIVER_OBJECT,
) -> NTSTATUS {
    let mut objectAttributes: _OBJECT_ATTRIBUTES = _OBJECT_ATTRIBUTES {
        Length: 0u32,
        RootDirectory: core::ptr::null_mut::<core::ffi::c_void>(),
        ObjectName: core::ptr::null_mut::<_UNICODE_STRING>(),
        Attributes: 0u32,
        SecurityDescriptor: core::ptr::null_mut::<core::ffi::c_void>(),
        SecurityQualityOfService: core::ptr::null_mut::<core::ffi::c_void>(),
    };
    let mut hardwareKey: HANDLE = core::ptr::null_mut::<core::ffi::c_void>();
    let mut unicodeString: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0u16,
        MaximumLength: 0u16,
        Buffer: core::ptr::null_mut::<u16>(),
    };
    let mut busKey: HANDLE = core::ptr::null_mut::<core::ffi::c_void>();
    let mut status: NTSTATUS = 0;

    // PAGED_CODE expansion (as in behavior anchor)
    if KeGetCurrentIrql() as i32 > (APC_LEVEL) as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0i32 == 0 {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const i8,
                b"drivers/storage/class/disk/geometry.c\0"
                    .as_ptr() as *const i8,
                195i32,
                b"0\0".as_ptr() as *const i8,
            );
            __int2c();
        }
    }

    // InitializeObjectAttributes (expanded inline)
    objectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    objectAttributes.RootDirectory = core::ptr::null_mut::<core::ffi::c_void>();
    objectAttributes.ObjectName = (*DriverObject).HardwareDatabase;
    objectAttributes.Attributes = (0x40 | 0x200) as u32; // OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE
    objectAttributes.SecurityDescriptor = core::ptr::null_mut::<core::ffi::c_void>();
    objectAttributes.SecurityQualityOfService = core::ptr::null_mut::<core::ffi::c_void>();

    status = ZwOpenKey(
        &raw mut hardwareKey,
        KEY_READ,
        &raw mut objectAttributes,
    );

    if !NT_SUCCESS(status) {
        TracePrint(
            TRACE_LEVEL_ERROR,
            TRACE_FLAG_GENERAL,
            b"DiskSaveDetectInfo: Cannot open hardware data. Name: %wZ\n\0".as_ptr() as *const i8,
            (*DriverObject).HardwareDatabase,
        );
        return status;
    }

    status = DiskSaveGeometryDetectInfo((DriverObject).cast(), hardwareKey);

    if !NT_SUCCESS(status) {
        TracePrint(
            TRACE_LEVEL_ERROR,
            TRACE_FLAG_GENERAL,
            b"DiskSaveDetectInfo: Can't query configuration data (%#08lx)\n\0".as_ptr() as *const i8,
            status,
        );
        ZwClose(hardwareKey);
        return status;
    }

    // Open EISA bus key
    const EISA_ADAPTER: [u16; 12] = [
        0x45, 0x69, 0x73, 0x61, 0x41, 0x64, 0x61, 0x70, 0x74, 0x65, 0x72, 0x00,
    ];
    RtlInitUnicodeString(
        &raw mut unicodeString,
        &EISA_ADAPTER as *const u16,
    );
    objectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    objectAttributes.RootDirectory = hardwareKey;
    objectAttributes.ObjectName = &raw mut unicodeString;
    objectAttributes.Attributes = (0x40 | 0x200) as u32;
    objectAttributes.SecurityDescriptor = core::ptr::null_mut::<core::ffi::c_void>();
    objectAttributes.SecurityQualityOfService = core::ptr::null_mut::<core::ffi::c_void>();

    status = ZwOpenKey(
        &raw mut busKey,
        KEY_READ,
        &raw mut objectAttributes,
    );

    if NT_SUCCESS(status) {
        TracePrint(
            TRACE_LEVEL_INFORMATION,
            TRACE_FLAG_GENERAL,
            b"DiskSaveDetectInfo: Opened EisaAdapter key\n\0".as_ptr() as *const i8,
        );
        DiskScanBusDetectInfo((DriverObject).cast(), busKey);
        ZwClose(busKey);
    }

    // Open MultiFunction bus key
    const MULTIFUNCTION_ADAPTER: [u16; 21] = [
        0x4D, 0x75, 0x6C, 0x74, 0x69, 0x66, 0x75, 0x6E, 0x63, 0x74,
        0x69, 0x6F, 0x6E, 0x41, 0x64, 0x61, 0x70, 0x74, 0x65, 0x72, 0x00,
    ];
    RtlInitUnicodeString(
        &raw mut unicodeString,
        &MULTIFUNCTION_ADAPTER as *const u16,
    );
    objectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    objectAttributes.RootDirectory = hardwareKey;
    objectAttributes.ObjectName = &raw mut unicodeString;
    objectAttributes.Attributes = (0x40 | 0x200) as u32;
    objectAttributes.SecurityDescriptor = core::ptr::null_mut::<core::ffi::c_void>();
    objectAttributes.SecurityQualityOfService = core::ptr::null_mut::<core::ffi::c_void>();

    status = ZwOpenKey(
        &raw mut busKey,
        KEY_READ,
        &raw mut objectAttributes,
    );

    if NT_SUCCESS(status) {
        TracePrint(
            TRACE_LEVEL_INFORMATION,
            TRACE_FLAG_GENERAL,
            b"DiskSaveDetectInfo: Opened MultifunctionAdapter key\n\0".as_ptr() as *const i8,
        );
        DiskScanBusDetectInfo((DriverObject).cast(), busKey);
        ZwClose(busKey);
    }

    ZwClose(hardwareKey);

    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "C" fn DiskCleanupDetectInfo(DriverObject: PDRIVER_OBJECT) {
    let _ = DriverObject;
    if !DetectInfoList.is_null() {
        ExFreePool((DetectInfoList).cast());
        DetectInfoList = core::ptr::null_mut();
    }
}

#[no_mangle]
pub unsafe extern "system" fn DiskSaveGeometryDetectInfo(
    DriverObject: PDRIVER_OBJECT,
    HardwareKey: HANDLE,
) -> NTSTATUS {
    // Expanded PAGED_CODE() macro (from c2rust anchor)
    if KeGetCurrentIrql() as i32 > (APC_LEVEL) as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0i32 == 0 {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0" as *const u8 as *const i8,
                b"drivers/storage/class/disk/geometry.c\0"
                    as *const u8 as *const i8,
                322i32,
                b"0\0" as *const u8 as *const i8,
            );
            __int2c();
        }
    }

    let mut unicodeString: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0u16,
        MaximumLength: 0u16,
        Buffer: core::ptr::null_mut::<u16>(),
    };
    let mut keyData: *mut KEY_VALUE_FULL_INFORMATION = core::ptr::null_mut();
    let mut length: u32 = 0u32;
    let mut fullDescriptor: *mut CM_FULL_RESOURCE_DESCRIPTOR = core::ptr::null_mut();
    let mut partialDescriptor: *mut CM_PARTIAL_RESOURCE_DESCRIPTOR = core::ptr::null_mut();
    let mut driveParameters: *mut CM_INT13_DRIVE_PARAMETER = core::ptr::null_mut();
    let mut numberOfDrives: u32 = 0u32;
    let mut i: u32 = 0u32;
    let mut status: NTSTATUS = 0i32;

    RtlInitUnicodeString(
        &raw mut unicodeString,
        w!("Configuration Data").as_ptr() as *const u16,
    );

    keyData = ExAllocatePoolWithTag(
        PagedPool,
        VALUE_BUFFER_SIZE as usize,
        DISK_TAG_UPDATE_GEOM,
    ) as *mut KEY_VALUE_FULL_INFORMATION;

    if keyData.is_null() {
        StorDebugPrint(
            2i32,
            (TRACE_FLAG_GENERAL as u32) as i32,
            b"DiskSaveGeometryDetectInfo: Can't allocate config data buffer\n\0" as *const u8
                as *const i8,
        );
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    status = ZwQueryValueKey(
        HardwareKey,
        &raw mut unicodeString,
        KeyValueFullInformation,
        keyData as *mut core::ffi::c_void,
        VALUE_BUFFER_SIZE,
        &raw mut length,
    );

    if !(status >= 0i32) {
        StorDebugPrint(
            2i32,
            (TRACE_FLAG_GENERAL as u32) as i32,
            b"DiskSaveGeometryDetectInfo: Can't query configuration data (%#08lx)\n\0"
                as *const u8 as *const i8,
        );
        if !keyData.is_null() {
            ExFreePool(keyData as *mut core::ffi::c_void);
            keyData = core::ptr::null_mut::<KEY_VALUE_FULL_INFORMATION>();
        }
        return status;
    }

    fullDescriptor =
        (keyData as *mut u8).add((*keyData).DataOffset as usize) as *mut CM_FULL_RESOURCE_DESCRIPTOR;
    partialDescriptor =
        &raw mut (*fullDescriptor).PartialResourceList.PartialDescriptors
            as *mut CM_PARTIAL_RESOURCE_DESCRIPTOR;
    length = (*partialDescriptor).u.DeviceSpecificData.DataSize;

    if ((*keyData).DataLength as usize) < core::mem::size_of::<CM_FULL_RESOURCE_DESCRIPTOR>()
        || (*fullDescriptor).PartialResourceList.Count == 0u32
        || (*partialDescriptor).Type as i32 != 5
        || (length as usize) < core::mem::size_of::<u32>()
    {
        StorDebugPrint(
            2i32,
            (TRACE_FLAG_GENERAL as u32) as i32,
            b"DiskSaveGeometryDetectInfo: BIOS header data too small or invalid\n\0"
                as *const u8 as *const i8,
        );
        if !keyData.is_null() {
            ExFreePool(keyData as *mut core::ffi::c_void);
            keyData = core::ptr::null_mut::<KEY_VALUE_FULL_INFORMATION>();
        }
        return STATUS_INVALID_PARAMETER;
    }

    let mut buffer: *mut u8 = keyData as *mut u8;
    buffer = buffer.add((*keyData).DataOffset as usize);
    buffer = buffer.add(core::mem::size_of::<CM_FULL_RESOURCE_DESCRIPTOR>());
    driveParameters = buffer as *mut CM_INT13_DRIVE_PARAMETER;

    numberOfDrives = (length as usize)
        .wrapping_div(core::mem::size_of::<CM_INT13_DRIVE_PARAMETER>()) as u32;

    length = (core::mem::size_of::<DISK_DETECT_INFO>())
        .wrapping_mul(numberOfDrives as usize) as u32;

    DetectInfoList = ExAllocatePoolWithTag(
        PagedPool,
        length as usize,
        DISK_TAG_UPDATE_GEOM,
    ) as *mut DISK_DETECT_INFO;

    if DetectInfoList.is_null() {
        StorDebugPrint(
            2i32,
            (TRACE_FLAG_GENERAL as u32) as i32,
            b"DiskSaveGeometryDetectInfo: Couldn't allocate %x bytes for DetectInfoList\n\0"
                as *const u8 as *const i8,
            length,
        );
        if !keyData.is_null() {
            ExFreePool(keyData as *mut core::ffi::c_void);
            keyData = core::ptr::null_mut::<KEY_VALUE_FULL_INFORMATION>();
        }
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    DetectInfoCount = numberOfDrives;
    core::ptr::write_bytes(
        DetectInfoList as *mut u8,
        0u8,
        length as usize,
    );

    i = 0u32;
    while i < numberOfDrives {
        (*DetectInfoList.add(i as usize)).DriveParameters =
            *driveParameters.add(i as usize);
        i = i.wrapping_add(1);
    }

    if !keyData.is_null() {
        ExFreePool(keyData as *mut core::ffi::c_void);
        keyData = core::ptr::null_mut::<KEY_VALUE_FULL_INFORMATION>();
    }

    return STATUS_SUCCESS;
}





#[no_mangle]
pub unsafe extern "system" fn DiskScanBusDetectInfo(
    DriverObject: PDRIVER_OBJECT,
    BusKey: HANDLE,
) {
    let mut busNumber: u32 = 0;
    let mut status: NTSTATUS;

    busNumber = 0u32;
    loop {
        let mut buffer: [u16; 32] = [0u16; 32];
        let mut unicodeString: _UNICODE_STRING = core::mem::zeroed();
        let mut objectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
        let mut spareKey: HANDLE = core::ptr::null_mut();
        let mut adapterKey: HANDLE = core::ptr::null_mut();
        let mut adapterNumber: u32 = 0;

        TracePrint(
            TRACE_LEVEL_INFORMATION,
            TRACE_FLAG_GENERAL,
            core::ffi::CStr::from_bytes_with_nul_unchecked(
                b"DiskScanBusDetectInfo: Scanning bus %d\n\0",
            )
            .as_ptr(),
            busNumber,
        );

        let fmt_percent_d: &[u16] = &[0x0025, 0x0064, 0x0000];
        status = RtlStringCchPrintfW(
            core::ptr::addr_of_mut!(buffer) as *mut u16,
            buffer.len() - 1,
            fmt_percent_d.as_ptr() as *const u16,
            busNumber,
        );
        if !NT_SUCCESS(status) {
            TracePrint(
                TRACE_LEVEL_ERROR,
                TRACE_FLAG_GENERAL,
                core::ffi::CStr::from_bytes_with_nul_unchecked(
                    b"DiskScanBusDetectInfo: Format symbolic link failed with error: 0x%X\n\0",
                )
                .as_ptr(),
                status,
            );
            break;
        }

        RtlInitUnicodeString(
            &raw mut unicodeString,
            core::ptr::addr_of!(buffer) as *const u16,
        );

        objectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
        objectAttributes.RootDirectory = BusKey;
        objectAttributes.ObjectName = &raw mut unicodeString as *mut _;
        objectAttributes.Attributes = (OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE) as u32;
        objectAttributes.SecurityDescriptor = core::ptr::null_mut();
        objectAttributes.SecurityQualityOfService = core::ptr::null_mut();

        status = ZwOpenKey(&raw mut spareKey, KEY_READ, &raw mut objectAttributes);

        if !NT_SUCCESS(status) {
            TracePrint(
                TRACE_LEVEL_ERROR,
                TRACE_FLAG_GENERAL,
                core::ffi::CStr::from_bytes_with_nul_unchecked(
                    b"DiskScanBusDetectInfo: Error %#08lx opening bus key %#x\n\0",
                )
                .as_ptr(),
                status,
                busNumber,
            );
            break;
        }

        let disk_controller_str: &[u16] = &[
            0x0044, 0x0069, 0x0073, 0x006B, 0x0043, 0x006F, 0x006E, 0x0074,
            0x0072, 0x006F, 0x006C, 0x006C, 0x0065, 0x0072, 0x0000,
        ];
        RtlInitUnicodeString(
            &raw mut unicodeString,
            disk_controller_str.as_ptr() as *const u16,
        );

        objectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
        objectAttributes.RootDirectory = spareKey;
        objectAttributes.ObjectName = &raw mut unicodeString as *mut _;
        objectAttributes.Attributes = (OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE) as u32;
        objectAttributes.SecurityDescriptor = core::ptr::null_mut();
        objectAttributes.SecurityQualityOfService = core::ptr::null_mut();

        status = ZwOpenKey(&raw mut adapterKey, KEY_READ, &raw mut objectAttributes);
        ZwClose(spareKey);

        if !NT_SUCCESS(status) {
            TracePrint(
                TRACE_LEVEL_ERROR,
                TRACE_FLAG_GENERAL,
                core::ffi::CStr::from_bytes_with_nul_unchecked(
                    b"DiskScanBusDetectInfo: Error %#08lx opening DiskController key\n\0",
                )
                .as_ptr(),
                status,
            );
        } else {
            adapterNumber = 0u32;
            loop {
                let mut diskKey: HANDLE = core::ptr::null_mut();
                let mut diskNumber: u32 = 0;

                TracePrint(
                    TRACE_LEVEL_INFORMATION,
                    TRACE_FLAG_GENERAL,
                    core::ffi::CStr::from_bytes_with_nul_unchecked(
                        b"DiskScanBusDetectInfo: Scanning disk key %d\\DiskController\\%d\\DiskPeripheral\n\0",
                    )
                    .as_ptr(),
                    busNumber,
                    adapterNumber,
                );

                let fmt_percent_d_diskperipheral: &[u16] = &[
                    0x0025, 0x0064, 0x005C, 0x0044, 0x0069, 0x0073, 0x006B,
                    0x0050, 0x0065, 0x0072, 0x0069, 0x0070, 0x0068, 0x0065,
                    0x0072, 0x0061, 0x006C, 0x0000,
                ];
                status = RtlStringCchPrintfW(
                    core::ptr::addr_of_mut!(buffer) as *mut u16,
                    buffer.len() - 1,
                    fmt_percent_d_diskperipheral.as_ptr() as *const u16,
                    adapterNumber,
                );
                if !NT_SUCCESS(status) {
                    TracePrint(
                        TRACE_LEVEL_ERROR,
                        TRACE_FLAG_GENERAL,
                        core::ffi::CStr::from_bytes_with_nul_unchecked(
                            b"DiskScanBusDetectInfo: Format symbolic link failed with error: 0x%X\n\0",
                        )
                        .as_ptr(),
                        status,
                    );
                    break;
                }

                RtlInitUnicodeString(
                    &raw mut unicodeString,
                    core::ptr::addr_of!(buffer) as *const u16,
                );

                objectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
                objectAttributes.RootDirectory = adapterKey;
                objectAttributes.ObjectName = &raw mut unicodeString as *mut _;
                objectAttributes.Attributes =
                    (OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE) as u32;
                objectAttributes.SecurityDescriptor = core::ptr::null_mut();
                objectAttributes.SecurityQualityOfService = core::ptr::null_mut();

                status = ZwOpenKey(&raw mut diskKey, KEY_READ, &raw mut objectAttributes);

                if !NT_SUCCESS(status) {
                    TracePrint(
                        TRACE_LEVEL_ERROR,
                        TRACE_FLAG_GENERAL,
                        core::ffi::CStr::from_bytes_with_nul_unchecked(
                            b"DiskScanBusDetectInfo: Error %#08lx opening disk key\n\0",
                        )
                        .as_ptr(),
                        status,
                    );
                    break;
                }

                diskNumber = 0u32;
                loop {
                    let mut targetKey: HANDLE = core::ptr::null_mut();

                    TracePrint(
                        TRACE_LEVEL_INFORMATION,
                        TRACE_FLAG_GENERAL,
                        core::ffi::CStr::from_bytes_with_nul_unchecked(
                            b"DiskScanBusDetectInfo: Scanning target key %d\\DiskController\\%d\\DiskPeripheral\\%d\n\0",
                        )
                        .as_ptr(),
                        busNumber,
                        adapterNumber,
                        diskNumber,
                    );

                    status = RtlStringCchPrintfW(
                        core::ptr::addr_of_mut!(buffer) as *mut u16,
                        buffer.len() - 1,
                        fmt_percent_d.as_ptr() as *const u16,
                        diskNumber,
                    );
                    if !NT_SUCCESS(status) {
                        TracePrint(
                            TRACE_LEVEL_ERROR,
                            TRACE_FLAG_GENERAL,
                            core::ffi::CStr::from_bytes_with_nul_unchecked(
                                b"DiskScanBusDetectInfo: Format symbolic link failed with error: 0x%X\n\0",
                            )
                            .as_ptr(),
                            status,
                        );
                        break;
                    }

                    RtlInitUnicodeString(
                        &raw mut unicodeString,
                        core::ptr::addr_of!(buffer) as *const u16,
                    );

                    objectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
                    objectAttributes.RootDirectory = diskKey;
                    objectAttributes.ObjectName = &raw mut unicodeString as *mut _;
                    objectAttributes.Attributes =
                        (OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE) as u32;
                    objectAttributes.SecurityDescriptor = core::ptr::null_mut();
                    objectAttributes.SecurityQualityOfService = core::ptr::null_mut();

                    status = ZwOpenKey(
                        &raw mut targetKey,
                        KEY_READ,
                        &raw mut objectAttributes,
                    );

                    if !NT_SUCCESS(status) {
                        TracePrint(
                            TRACE_LEVEL_ERROR,
                            TRACE_FLAG_GENERAL,
                            core::ffi::CStr::from_bytes_with_nul_unchecked(
                                b"DiskScanBusDetectInfo: Error %#08lx opening target key\n\0",
                            )
                            .as_ptr(),
                            status,
                        );
                        break;
                    }

                    DiskSaveBusDetectInfo(DriverObject, targetKey, diskNumber);
                    ZwClose(targetKey);

                    diskNumber = diskNumber.wrapping_add(1);
                }

                ZwClose(diskKey);
                adapterNumber = adapterNumber.wrapping_add(1);
            }
            ZwClose(adapterKey);
        }

        busNumber = busNumber.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "system" fn DiskSaveBusDetectInfo(
    DriverObject: PDRIVER_OBJECT,
    TargetKey: HANDLE,
    DiskNumber: ULONG,
) -> NTSTATUS {
    // Declarations for symbols not in imports

    const APC_LEVEL: u8 = 1;
    const TRACE_FLAG_GENERAL: i32 = 0; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)
    const VALUE_BUFFER_SIZE: usize = 0x0800; // placeholder - adjust if needed
    const DISK_TAG_UPDATE_GEOM: u32 = 0x67446353; // example tag

    let mut diskInfo: *mut DISK_DETECT_INFO = core::ptr::null_mut();
    let mut unicodeString: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut keyData: *mut _KEY_VALUE_FULL_INFORMATION = core::ptr::null_mut();
    let mut length: ULONG = 0;
    let mut status: NTSTATUS = 0;

    // PAGED_CODE();
    if KeGetCurrentIrql() as i32 > APC_LEVEL as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0 == 0 {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const i8,
                b"drivers/storage/class/disk/geometry.c\0"
                    .as_ptr() as *const i8,
                642i32,
                b"0\0".as_ptr() as *const i8,
            );
            __int2c();
        }
    }

    if DiskNumber >= DetectInfoCount {
        return STATUS_UNSUCCESSFUL;
    }

    diskInfo = DetectInfoList.add(DiskNumber as usize) as *mut DISK_DETECT_INFO;

    if (*diskInfo).Initialized != 0 {
        if 0 == 0 {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const i8,
                b"drivers/storage/class/disk/geometry.c\0"
                    .as_ptr() as *const i8,
                654i32,
                b"FALSE\0".as_ptr() as *const i8,
            );
            __int2c();
        }
        StorDebugPrint(
            2i32,
            TRACE_FLAG_GENERAL,
            b"DiskSaveBusDetectInfo: disk entry %#x already has a signature of %#08lx and mbr checksum of %#08lx\n\0"
                .as_ptr() as *const i8,
            DiskNumber,
            (*diskInfo).Signature,
            (*diskInfo).MbrCheckSum,
        );
        return STATUS_UNSUCCESSFUL;
    }

    RtlInitUnicodeString(
        &raw mut unicodeString,
        w!("Identifier").as_ptr() as *const u16,
    );

    keyData = ExAllocatePoolWithTag(
        PagedPool,
        (VALUE_BUFFER_SIZE as u64) as usize,
        DISK_TAG_UPDATE_GEOM,
    ) as *mut _KEY_VALUE_FULL_INFORMATION;

    if keyData.is_null() {
        StorDebugPrint(
            2i32,
            TRACE_FLAG_GENERAL,
            b"DiskSaveBusDetectInfo: Couldn't allocate space for registry data\n\0"
                .as_ptr() as *const i8,
        );
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    status = ZwQueryValueKey(
        TargetKey,
        &raw mut unicodeString,
        KeyValueFullInformation,
        keyData as PVOID,
        VALUE_BUFFER_SIZE as u32,
        &raw mut length,
    );

    if !(status >= 0) {
        StorDebugPrint(
            2i32,
            TRACE_FLAG_GENERAL,
            b"DiskSaveBusDetectInfo: Error %#08lx getting Identifier\n\0"
                .as_ptr() as *const i8,
            status,
        );
        if !keyData.is_null() {
            ExFreePool(keyData as PVOID);
            keyData = core::ptr::null_mut();
        }
        return status;
    } else if ((*keyData).DataLength as usize)
        < (9 as usize).wrapping_mul(core::mem::size_of::<u16>())
    {
        StorDebugPrint(
            2i32,
            TRACE_FLAG_GENERAL,
            b"DiskSaveBusDetectInfo: Saved data was invalid, not enough data in registry!\n\0"
                .as_ptr() as *const i8,
        );
        if !keyData.is_null() {
            ExFreePool(keyData as PVOID);
            keyData = core::ptr::null_mut();
        }
        return STATUS_UNSUCCESSFUL;
    } else {
        let mut identifier: _UNICODE_STRING = _UNICODE_STRING {
            Length: 0,
            MaximumLength: 0,
            Buffer: core::ptr::null_mut(),
        };
        let mut value: ULONG = 0;

        identifier.Buffer =
            (keyData as *mut u8).add((*keyData).DataOffset as usize) as *mut u16;
        identifier.Length = (*keyData).DataLength as u16;
        identifier.MaximumLength = (*keyData).DataLength as u16;

        status = RtlUnicodeStringToInteger(
            &raw mut identifier as PCUNICODE_STRING,
            16u32,
            &raw mut value,
        );

        if !(status >= 0) {
            StorDebugPrint(
                2i32,
                TRACE_FLAG_GENERAL,
                b"DiskSaveBusDetectInfo: Error %#08lx converting identifier %wZ into MBR xsum\n\0"
                    .as_ptr() as *const i8,
                status,
                &raw mut identifier,
            );
            if !keyData.is_null() {
                ExFreePool(keyData as PVOID);
                keyData = core::ptr::null_mut();
            }
            return status;
        }

        (*diskInfo).MbrCheckSum = value;

        identifier.Buffer = identifier.Buffer.add(9);
        identifier.Length = identifier.Length.wrapping_sub(
            (9usize.wrapping_mul(core::mem::size_of::<u16>())) as u16,
        );
        identifier.MaximumLength = identifier.MaximumLength.wrapping_sub(
            (9usize.wrapping_mul(core::mem::size_of::<u16>())) as u16,
        );

        status = RtlUnicodeStringToInteger(
            &raw mut identifier as PCUNICODE_STRING,
            16u32,
            &raw mut value,
        );

        if !(status >= 0) {
            StorDebugPrint(
                2i32,
                TRACE_FLAG_GENERAL,
                b"DiskSaveBusDetectInfo: Error %#08lx converting identifier %wZ into disk signature\n\0"
                    .as_ptr() as *const i8,
                status,
                &raw mut identifier,
            );
            value = 0u32;
        }

        (*diskInfo).Signature = value;
    }

    (*diskInfo).Initialized = 1;

    if !keyData.is_null() {
        ExFreePool(keyData as PVOID);
        keyData = core::ptr::null_mut();
    }

    return STATUS_SUCCESS;
}




#[no_mangle]
pub unsafe extern "system" fn DiskUpdateGeometry(
    FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION,
) -> DISK_GEOMETRY_SOURCE {
    const APC_LEVEL: u8 = 1;
    const FILE_REMOVABLE_MEDIA: u32 = 0x1;
    const DiskGeometryUnknown: DISK_GEOMETRY_SOURCE = 0;
    const DiskGeometryFromBios: DISK_GEOMETRY_SOURCE = 1;
    const DiskGeometryFromPort: DISK_GEOMETRY_SOURCE = 2;
    const TRACE_FLAG_GENERAL: u32 = 0; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)

    let diskData: PDISK_DATA = (*FdoExtension).__bindgen_anon_1.CommonExtension.as_ref().DriverData as PDISK_DATA;
    let mut i: u32;
    let mut diskInfo: PDISK_DETECT_INFO = core::ptr::null_mut::<DISK_DETECT_INFO>();
    let mut found: i32 = 0;
    let mut status: NTSTATUS;

    // PAGED_CODE implementation
    if KeGetCurrentIrql() as i32 > APC_LEVEL as i32 {
        DbgPrint(
            (b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const u8).cast(),
            KeGetCurrentIrql() as i32,
        );
        if 0 == 0 {
            DbgPrint(
                (b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const u8).cast(),
                b"drivers/storage/class/disk/geometry.c\0"
                    .as_ptr() as *const i8,
                809i32,
                b"0\0".as_ptr() as *const i8,
            );
            __int2c();
        }
    }

    // NT_ASSERT((FdoExtension->DeviceObject->Characteristics & FILE_REMOVABLE_MEDIA) == 0)
    if !((*(*FdoExtension).__bindgen_anon_1.CommonExtension.as_ref().DeviceObject).Characteristics as u32 & FILE_REMOVABLE_MEDIA == 0) {
        DbgPrint(
            (b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const u8).cast(),
            b"drivers/storage/class/disk/geometry.c\0"
                .as_ptr() as *const i8,
            812i32,
            b"(FdoExtension->DeviceObject->Characteristics & FILE_REMOVABLE_MEDIA) == 0\0"
                .as_ptr() as *const i8,
        );
        __int2c();
    }

    // If we've already set a non-default geometry, return it.
    if (*diskData).GeometrySource as u32 != DiskGeometryUnknown as u32 {
        return (*diskData).GeometrySource;
    }

    // Scan through saved detect info
    i = 0;
    while i < unsafe { DetectInfoCount } {
        if unsafe { DetectInfoList.is_null() } {
            DbgPrint(
                (b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const u8).cast(),
                b"drivers/storage/class/disk/geometry.c\0"
                    .as_ptr() as *const i8,
                830i32,
                b"DetectInfoList != NULL\0".as_ptr() as *const i8,
            );
            __int2c();
        }
        diskInfo = unsafe { DetectInfoList.offset(i as isize) as *mut DISK_DETECT_INFO as PDISK_DETECT_INFO };

        if (*diskData).__bindgen_anon_1.Mbr.as_ref().Signature != 0
            && (*diskData).__bindgen_anon_1.Mbr.as_ref().Signature == (*diskInfo).Signature
        {
            StorDebugPrint(
                4,
                (TRACE_FLAG_GENERAL) as i32,
                b"DiskUpdateGeometry: found match for signature %#08lx\n\0".as_ptr() as *const i8,
                (*diskData).__bindgen_anon_1.Mbr.as_ref().Signature,
            );
            found = 1;
            break;
        } else if (*diskData).__bindgen_anon_1.Mbr.as_ref().Signature == 0
            && (*diskData).__bindgen_anon_1.Mbr.as_ref().MbrCheckSum != 0
            && (*diskData).__bindgen_anon_1.Mbr.as_ref().MbrCheckSum == (*diskInfo).MbrCheckSum
        {
            StorDebugPrint(
                4,
                (TRACE_FLAG_GENERAL) as i32,
                b"DiskUpdateGeometry: found match for xsum %#08lx\n\0".as_ptr() as *const i8,
                (*diskData).__bindgen_anon_1.Mbr.as_ref().MbrCheckSum,
            );
            found = 1;
            break;
        } else {
            i = i.wrapping_add(1);
        }
    }

    if found != 0 {
        let cylinders: u32;
        let sectorsPerTrack: u32;
        let tracksPerCylinder: u32;
        let length: u32;

        cylinders = ((*diskInfo).DriveParameters.MaxCylinders as u32).wrapping_add(1);
        sectorsPerTrack = (*diskInfo).DriveParameters.SectorsPerTrack as u32;
        tracksPerCylinder = ((*diskInfo).DriveParameters.MaxHeads as i32 + 1) as u32;
        length = tracksPerCylinder.wrapping_mul(sectorsPerTrack);

        if length == 0 {
            StorDebugPrint(
                4,
                (TRACE_FLAG_GENERAL) as i32,
                b"DiskUpdateGeometry: H (%d) or S(%d) is zero\n\0".as_ptr() as *const i8,
                tracksPerCylinder,
                sectorsPerTrack,
            );
            return DiskGeometryUnknown;
        }

        (*diskData).RealGeometry = (*FdoExtension).DiskGeometry;
        (*diskData).RealGeometry.SectorsPerTrack = sectorsPerTrack;
        (*diskData).RealGeometry.TracksPerCylinder = tracksPerCylinder;
        (*diskData).RealGeometry.Cylinders.QuadPart = cylinders as i64;

        StorDebugPrint(
            4,
            (TRACE_FLAG_GENERAL) as i32,
            b"DiskUpdateGeometry: BIOS spt %#x, #heads %#x, #cylinders %#x\n\0".as_ptr() as *const i8,
            sectorsPerTrack,
            tracksPerCylinder,
            cylinders,
        );

        (*diskData).GeometrySource = DiskGeometryFromBios;
        (*diskInfo).Device = (*FdoExtension).__bindgen_anon_1.CommonExtension.as_ref().DeviceObject;

        InterlockedIncrement(core::ptr::addr_of_mut!(DetectInfoUsedCount) as *mut i32);
    } else {
        StorDebugPrint(
            2,
            (TRACE_FLAG_GENERAL) as i32,
            b"DiskUpdateGeometry: no match found for signature %#08lx\n\0".as_ptr() as *const i8,
            (*diskData).__bindgen_anon_1.Mbr.as_ref().Signature,
        );
    }

    if (*diskData).GeometrySource as u32 == DiskGeometryUnknown as u32 {
        status = DiskGetPortGeometry(FdoExtension, core::ptr::addr_of_mut!((*diskData).RealGeometry));
        if NT_SUCCESS(status) {
            if ((*diskData).RealGeometry.TracksPerCylinder.wrapping_mul(
                (*diskData).RealGeometry.SectorsPerTrack,
            )) != 0
            {
                (*diskData).GeometrySource = DiskGeometryFromPort;
                StorDebugPrint(
                    4,
                    (TRACE_FLAG_GENERAL) as i32,
                    b"DiskUpdateGeometry: using Port geometry for disk %#p\n\0".as_ptr() as *const i8,
                    FdoExtension,
                );
                if (*diskData).RealGeometry.BytesPerSector == 0 {
                    StorDebugPrint(
                        2,
                        (TRACE_FLAG_GENERAL) as i32,
                        b"DiskDriverReinit: Port driver failed to set BytesPerSector in the RealGeometry\n\0"
                            .as_ptr() as *const i8,
                    );
                    (*diskData).RealGeometry.BytesPerSector =
                        (*FdoExtension).DiskGeometry.BytesPerSector;
                    if (*diskData).RealGeometry.BytesPerSector == 0 {
                        // NT_ASSERT(!"...") asserts FALSE: it fires whenever reached. The
                        // translation spelled that as a null check on a literal, which is
                        // the same constant -- written out here so it reads as one.
                        {
                            DbgPrint(
                                (b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const u8).cast(),
                                b"drivers/storage/class/disk/geometry.c\0"
                                    .as_ptr() as *const i8,
                                949i32,
                                b"!\"BytesPerSector is still zero!\"\0".as_ptr() as *const i8,
                            );
                            __int2c();
                        }
                    }
                }
            }
        }
    }

    if (*diskData).GeometrySource as u32 != DiskGeometryUnknown as u32 {
        (*FdoExtension).DiskGeometry = (*diskData).RealGeometry;
    }

    return (*diskData).GeometrySource;
}





#[no_mangle]
pub unsafe extern "system" fn DiskUpdateRemovableGeometry(
    FdoExtension: *mut FUNCTIONAL_DEVICE_EXTENSION,
) -> i32 {
    // PAGED_CODE expansion
    if KeGetCurrentIrql() as i32 > 1 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0i32 == 0 {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const i8,
                b"drivers/storage/class/disk/geometry.c\0"
                    .as_ptr() as *const i8,
                1004i32,
                b"0\0".as_ptr() as *const i8,
            );
            __int2c();
        }
    }

    let commonExtension: *mut COMMON_DEVICE_EXTENSION = (&raw mut (*FdoExtension).__bindgen_anon_1.CommonExtension).cast();
    let diskData: *mut DISK_DATA = (*commonExtension).DriverData as *mut DISK_DATA;
    let geometry: *mut DISK_GEOMETRY = &raw mut (*diskData).RealGeometry;
    let mut status: i32;

    // NT_ASSERT(FdoExtension->DeviceDescriptor->RemovableMedia)
    if !(*FdoExtension).DeviceDescriptor.is_null() {
        if (*(*FdoExtension).DeviceDescriptor).RemovableMedia == 0 {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const i8,
                b"drivers/storage/class/disk/geometry.c\0"
                    .as_ptr() as *const i8,
                1007i32,
                b"FdoExtension->DeviceDescriptor->RemovableMedia\0".as_ptr() as *const i8,
            );
            __int2c();
        }
    }

    // NT_ASSERT(TEST_FLAG(FdoExtension->DeviceObject->Characteristics, FILE_REMOVABLE_MEDIA))
    if ((*(*FdoExtension).LowerPdo).Characteristics as u32 & 0x1u32) == 0 {
        DbgPrint(
            b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const i8,
            b"drivers/storage/class/disk/geometry.c\0"
                .as_ptr() as *const i8,
            1010i32,
            b"TEST_FLAG(FdoExtension->DeviceObject->Characteristics, FILE_REMOVABLE_MEDIA)\0".as_ptr() as *const i8,
        );
        __int2c();
    }

    status = DiskGetPortGeometry(FdoExtension, geometry);

    if status >= 0
        && (*geometry)
            .TracksPerCylinder
            .wrapping_mul((*geometry).SectorsPerTrack)
            != 0u32
    {
        (*FdoExtension).DiskGeometry = *geometry;
    }

    status
}

#[no_mangle]
pub unsafe extern "system" fn DiskGetPortGeometry(
    FdoExtension: *mut FUNCTIONAL_DEVICE_EXTENSION,
    Geometry: *mut DISK_GEOMETRY,
) -> NTSTATUS {
    let commonExtension: *mut COMMON_DEVICE_EXTENSION =
        core::ptr::addr_of_mut!((*FdoExtension).__bindgen_anon_1.CommonExtension) as *mut COMMON_DEVICE_EXTENSION;
    let mut irp: *mut IRP = core::ptr::null_mut();
    let mut irpStack: *mut IO_STACK_LOCATION;
    let mut event: KEVENT = core::mem::zeroed();
    let mut status: NTSTATUS = 0;

    // PAGED_CODE() expansion
    if KeGetCurrentIrql() > APC_LEVEL as u8 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0 == 0 {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0" as *const u8 as *const i8,
                b"geometry.c\0" as *const u8 as *const i8,
                1063 as i32,
                b"0\0" as *const u8 as *const i8,
            );
            __int2c();
        }
    }

    irp = IoAllocateIrp((*commonExtension).LowerDeviceObject.read().StackSize as CCHAR, 0u8);
    if irp.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    irpStack = IoGetNextIrpStackLocation(irp);
    (*irpStack).MajorFunction = IRP_MJ_DEVICE_CONTROL as u8;
    (*irpStack).Parameters.DeviceIoControl.IoControlCode =
        IOCTL_DISK_GET_DRIVE_GEOMETRY as u32;
    (*irpStack).Parameters.DeviceIoControl.OutputBufferLength =
        core::mem::size_of::<DISK_GEOMETRY>() as u32;
    (*irp).AssociatedIrp.SystemBuffer = Geometry as *mut core::ffi::c_void;
    KeInitializeEvent(&mut event as *mut KEVENT, _EVENT_TYPE_SynchronizationEvent, 0u8);
    IoSetCompletionRoutine(
        irp,
        Some(ClassSignalCompletion),
        &mut event as *mut KEVENT as *mut core::ffi::c_void,
        1u8,
        1u8,
        1u8,
    );
    status = IoCallDriver((*commonExtension).LowerDeviceObject, irp);
    if status == STATUS_PENDING {
        KeWaitForSingleObject(
            &mut event as *mut KEVENT as *mut core::ffi::c_void,
            _KWAIT_REASON_Executive,
            (0u8) as KPROCESSOR_MODE,
            0u8,
            core::ptr::null_mut(),
        );
        status = (*irp).IoStatus.__bindgen_anon_1.Status;
    }
    IoFreeIrp(irp);
    status
}







#[no_mangle]
pub unsafe extern "C" fn DiskIsNT4Geometry(
    FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION,
) -> BOOLEAN {
    let mut readBuffer: PUSHORT = core::ptr::null_mut();
    let mut bFoundNT4: BOOLEAN = 0;

    // PAGED_CODE expansion
    if KeGetCurrentIrql() as i32 > (APC_LEVEL) as i32 {
        DbgPrint(
            (b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as *const u8).cast(),
            KeGetCurrentIrql() as i32,
        );
        if 0 == 0 {
            DbgPrint(
                (b"Assertion failed at %s(%d): %s\n\0" as *const u8 as *const u8).cast(),
                b"drivers/storage/class/disk/geometry.c\0"
                    as *const u8 as *const u8,
                1143 as i32,
                b"0\0" as *const u8 as *const u8,
            );
            __int2c();
        }
    }

    readBuffer = ExAllocatePoolWithTag(
        (NonPagedPool) as u32,
        (*FdoExtension).DiskGeometry.BytesPerSector as SIZE_T,
        DISK_TAG_UPDATE_GEOM as u32,
    ) as PUSHORT;

    if !readBuffer.is_null() {
        let mut event: KEVENT = core::mem::zeroed();
        KeInitializeEvent(&raw mut event, _EVENT_TYPE_NotificationEvent, 0);

        let mut diskOffset: LARGE_INTEGER = core::mem::zeroed();
        diskOffset.QuadPart = 0 as LONGLONG;

        let mut ioStatus: _IO_STATUS_BLOCK = core::mem::zeroed();
        let mut irp: PIRP = core::ptr::null_mut();

        irp = IoBuildSynchronousFsdRequest(
            (IRP_MJ_READ) as u32,
            (*FdoExtension).__bindgen_anon_1.CommonExtension.as_ref().DeviceObject,
            readBuffer as *mut core::ffi::c_void,
            (*FdoExtension).DiskGeometry.BytesPerSector,
            &raw mut diskOffset,
            &raw mut event,
            &raw mut ioStatus,
        );

        if !irp.is_null() {
            let irpSp: PIO_STACK_LOCATION = IoGetNextIrpStackLocation(irp);
            let mut status: NTSTATUS;

            (*irpSp).Flags = ((*irpSp).Flags as i32 | SL_OVERRIDE_VERIFY_VOLUME) as u8;
            status = IoCallDriver((*FdoExtension).__bindgen_anon_1.CommonExtension.as_ref().DeviceObject, irp);

            if status == STATUS_PENDING {
                KeWaitForSingleObject(
                    &raw mut event as *mut _,
                    _KWAIT_REASON_Executive,
                    (0i32) as i8,
                    0,
                    core::ptr::null_mut(),
                );
                status = ioStatus.__bindgen_anon_1.Status;
            }

            if status >= 0 {
                if *readBuffer.offset((BOOT_SIGNATURE_OFFSET) as isize) as i32 == BOOT_RECORD_SIGNATURE as i32 {
                    let mut partitionTableEntry: PPARTITION_DESCRIPTOR =
                        readBuffer.offset((PARTITION_TABLE_OFFSET) as isize) as *mut u16 as PPARTITION_DESCRIPTOR;
                    let mut uCount: u32 = 0;

                    while uCount < NUM_PARTITION_TABLE_ENTRIES {
                        let pt = *partitionTableEntry;
                        if (pt.PartitionType as i32 == PARTITION_EXTENDED as i32)
                            || (pt.PartitionType as i32 == PARTITION_XINT13_EXTENDED as i32)
                            || (pt.PartitionType as i32 == PARTITION_LDM as i32)
                        {
                            if pt.StartingTrack as i32 == 1
                                && ((pt.StartingSectorLsb0 as u32)
                                    .wrapping_add(((pt.StartingSectorLsb1 as i32) << 8) as u32)
                                    .wrapping_add(((pt.StartingSectorMsb0 as i32) << 16) as u32)
                                    .wrapping_add(((pt.StartingSectorMsb1 as i32) << 24) as u32))
                                    == 0x20
                            {
                                bFoundNT4 = 1;
                                break;
                            } else if pt.EndingTrack as i32 == 0x3F
                                && ((pt.EndingCylinderLsb as i32 & 0x3F) as u8) as i32 == 0x20
                            {
                                bFoundNT4 = 1;
                                break;
                            }
                        }
                        partitionTableEntry = partitionTableEntry.offset(1);
                        uCount = uCount.wrapping_add(1);
                    }
                }
            }
        }

        if !readBuffer.is_null() {
            ExFreePool(readBuffer as *mut core::ffi::c_void);
            readBuffer = core::ptr::null_mut();
        }
    }

    return bFoundNT4;
}

#[no_mangle]
pub unsafe extern "C" fn DiskReadDriveCapacity(Fdo: PDEVICE_OBJECT) -> NTSTATUS {
    let fdoExtension: PFUNCTIONAL_DEVICE_EXTENSION = (*Fdo).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    let status: NTSTATUS;
    if ((*Fdo).Characteristics as u32 & 0x1u32) != 0u32 {
        DiskUpdateRemovableGeometry(fdoExtension);
    } else {
        DiskUpdateGeometry(fdoExtension);
    }
    status = ClassReadDriveCapacity(Fdo);
    return status;
}
#[no_mangle]
pub unsafe extern "C" fn DiskDriverReinitialization(
    DriverObject: ntkrnl_sys::ddk::PDRIVER_OBJECT,
    Nothing: ntkrnl_sys::base::PVOID,
    Count: u32,
) {
    let mut deviceObject: ntkrnl_sys::ddk::PDEVICE_OBJECT = core::ptr::null_mut();
    let mut fdoExtension: *mut _FUNCTIONAL_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut diskData: PDISK_DATA = core::ptr::null_mut();
    let mut unmatchedDiskCount: u32 = 0;
    let mut unmatchedDisk: ntkrnl_sys::ddk::PDEVICE_OBJECT = core::ptr::null_mut();
    let mut i: u32 = 0;
    let mut diskInfo: PDISK_DETECT_INFO = core::ptr::null_mut();

    let _ = Nothing; // UNREFERENCED_PARAMETER

    if Count != 1 {
        TracePrint(
            3,
            TRACE_FLAG_GENERAL,
            b"DiskDriverReinitialization: ignoring call %d\n\0".as_ptr() as *const i8,
            Count,
        );
        return;
    }

    if DetectInfoCount == 0 {
        TracePrint(
            3,
            TRACE_FLAG_GENERAL,
            b"DiskDriverReinitialization: no detect info saved\n\0".as_ptr() as *const i8,
        );
        return;
    }

    if (DetectInfoCount as u32).wrapping_sub(DetectInfoUsedCount as u32) != 1 {
        TracePrint(
            3,
            TRACE_FLAG_GENERAL,
            b"DiskDriverReinitialization: %d of %d geometry entries used - will not attempt match\n\0"
                .as_ptr() as *const i8,
            DetectInfoUsedCount,
            DetectInfoCount,
        );
        return;
    }

    unmatchedDiskCount = 0;
    deviceObject = (*DriverObject).DeviceObject;

    while !deviceObject.is_null() {
        fdoExtension = (*deviceObject).DeviceExtension
            as *mut _FUNCTIONAL_DEVICE_EXTENSION;

        if (*fdoExtension).__bindgen_anon_1.CommonExtension.as_ref().__bindgen_anon_1.IsFdo() as i32 == 0 {
            TracePrint(
                4,
                TRACE_FLAG_GENERAL,
                b"DiskDriverReinit: %#p is not an FDO\n\0".as_ptr() as *const i8,
                deviceObject,
            );
        } else {
            diskData = (*fdoExtension).__bindgen_anon_1.CommonExtension.as_ref().DriverData as PDISK_DATA;

            if (*diskData).GeometrySource as u32 != DiskGeometryUnknown as u32 {
                TracePrint(
                    4,
                    TRACE_FLAG_GENERAL,
                    b"DiskDriverReinit: FDO %#p has a geometry\n\0".as_ptr() as *const i8,
                    deviceObject,
                );
            } else {
                TracePrint(
                    3,
                    TRACE_FLAG_GENERAL,
                    b"DiskDriverReinit: FDO %#p has no geometry\n\0".as_ptr() as *const i8,
                    deviceObject,
                );

                (*diskData).GeometrySource = DiskGeometryFromDefault;
                unmatchedDiskCount = unmatchedDiskCount.wrapping_add(1);

                if unmatchedDiskCount > 1 {
                    if unmatchedDisk.is_null() {
                        // NT_ASSERT(unmatchedDisk != NULL)
                        DbgPrint(
                            b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const i8,
                            b"drivers/storage/class/disk/geometry.c\0"
                                .as_ptr() as *const i8,
                            1407i32,
                            b"unmatchedDisk != NULL\0".as_ptr() as *const i8,
                        );
                        __int2c();
                    }

                    TracePrint(
                        3,
                        TRACE_FLAG_GENERAL,
                        b"DiskDriverReinit: FDO %#p also has no geometry\n\0".as_ptr() as *const i8,
                        unmatchedDisk,
                    );

                    unmatchedDisk = core::ptr::null_mut();
                    break;
                }

                unmatchedDisk = deviceObject;
            }
        }

        deviceObject = (*deviceObject).NextDevice as ntkrnl_sys::ddk::PDEVICE_OBJECT;
    }

    if unmatchedDiskCount != 1 {
        TracePrint(
            2,
            TRACE_FLAG_GENERAL,
            b"DiskDriverReinit: Unable to match geometry\n\0".as_ptr() as *const i8,
        );
        return;
    }

    fdoExtension =
        (*unmatchedDisk).DeviceExtension as *mut _FUNCTIONAL_DEVICE_EXTENSION;
    diskData = (*fdoExtension).__bindgen_anon_1.CommonExtension.as_ref().DriverData as PDISK_DATA;

    TracePrint(
        4,
        TRACE_FLAG_GENERAL,
        b"DiskDriverReinit: Found possible match\n\0".as_ptr() as *const i8,
    );

    // Find the geometry which wasn't assigned.
    i = 0;
    while i < DetectInfoCount {
        if (*(DetectInfoList.offset(i as isize) as PDISK_DETECT_INFO)).Device.is_null() {
            diskInfo = DetectInfoList.offset(i as isize) as PDISK_DETECT_INFO;
            break;
        }
        i = i.wrapping_add(1);
    }

    if !diskInfo.is_null() {
        let mut cylinders: u32;
        let mut sectorsPerTrack: u32;
        let mut tracksPerCylinder: u32;
        let mut length: u32;

        cylinders = ((*diskInfo).DriveParameters.MaxCylinders as u32).wrapping_add(1);
        sectorsPerTrack = (*diskInfo).DriveParameters.SectorsPerTrack as u32;
        tracksPerCylinder = ((*diskInfo).DriveParameters.MaxHeads as i32 + 1) as u32;

        length = tracksPerCylinder.wrapping_mul(sectorsPerTrack);

        if length == 0 {
            TracePrint(
                4,
                TRACE_FLAG_GENERAL,
                b"DiskDriverReinit: H (%d) or S(%d) is zero\n\0".as_ptr() as *const i8,
                tracksPerCylinder,
                sectorsPerTrack,
            );
            return;
        }

        // copy fdoExtension->DiskGeometry into diskData->RealGeometry
        core::ptr::copy_nonoverlapping(
            core::ptr::addr_of!((*fdoExtension).DiskGeometry),
            core::ptr::addr_of_mut!((*diskData).RealGeometry),
            1,
        );

        (*diskData).RealGeometry.SectorsPerTrack = sectorsPerTrack;
        (*diskData).RealGeometry.TracksPerCylinder = tracksPerCylinder;
        (*diskData).RealGeometry.Cylinders.QuadPart = cylinders as i64;

        TracePrint(
            4,
            TRACE_FLAG_GENERAL,
            b"DiskDriverReinit: BIOS spt %#x, #heads %#x, #cylinders %#x\n\0".as_ptr() as *const i8,
            sectorsPerTrack,
            tracksPerCylinder,
            cylinders,
        );

        (*diskData).GeometrySource = DiskGeometryGuessedFromBios;
        (*diskInfo).Device = unmatchedDisk;

        // copy diskData->RealGeometry back into fdoExtension->DiskGeometry
        core::ptr::copy_nonoverlapping(
            core::ptr::addr_of!((*diskData).RealGeometry),
            core::ptr::addr_of_mut!((*fdoExtension).DiskGeometry),
            1,
        );

        ClassReadDriveCapacity(unmatchedDisk);

        if (*diskData).RealGeometry.BytesPerSector == 0 {
            // NT_ASSERT(!"...") asserts FALSE: it fires whenever reached.
            {
                DbgPrint(
                    b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const i8,
                    b"drivers/storage/class/disk/geometry.c\0"
                        .as_ptr() as *const i8,
                    1526i32,
                    b"!\"RealGeometry not set to non-zero bps\n\"\0".as_ptr() as *const i8,
                );
                __int2c();
            }
        }
    }
}







#[no_mangle]
pub unsafe extern "C" fn DiskGetDetectInfo(
    FdoExtension: PFUNCTIONAL_DEVICE_EXTENSION,
    DetectInfo: PDISK_DETECTION_INFO,
) -> NTSTATUS {
    let mut i: ULONG = 0;
    let mut found: BOOLEAN = 0;
    let mut diskInfo: *mut DISK_DETECT_INFO = core::ptr::null_mut::<DISK_DETECT_INFO>();
    let diskData: PDISK_DATA =
        (*FdoExtension).__bindgen_anon_1.CommonExtension.as_ref().DriverData as PDISK_DATA;

    // PAGED_CODE inline
    if KeGetCurrentIrql() > (APC_LEVEL) as u8 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0 == 0 {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const i8,
                b"drivers/storage/class/disk/geometry.c\0".as_ptr()
                    as *const i8,
                1563i32,
                b"0\0".as_ptr() as *const i8,
            );
            __int2c();
        }
    }

    if ((*(*FdoExtension).__bindgen_anon_1.__bindgen_anon_1.as_ref().DeviceObject).Characteristics as u32
        & FILE_REMOVABLE_MEDIA)
        != 0
    {
        return STATUS_NOT_SUPPORTED;
    }

    if (*diskData).PartitionStyle as u32 == PARTITION_STYLE_GPT as u32 {
        return STATUS_NOT_SUPPORTED;
    }

    i = 0;
    while i < DetectInfoCount {
        if DetectInfoList.is_null() {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const i8,
                b"drivers/storage/class/disk/geometry.c\0".as_ptr()
                    as *const i8,
                1584i32,
                b"DetectInfoList != NULL\0".as_ptr() as *const i8,
            );
            __int2c();
        }

        diskInfo = DetectInfoList.offset(i as isize) as *mut DISK_DETECT_INFO;

        if (*diskData).__bindgen_anon_1.Mbr.as_ref().Signature != 0
            && (*diskData).__bindgen_anon_1.Mbr.as_ref().Signature == (*diskInfo).Signature
        {
            TracePrint(
                TRACE_LEVEL_INFORMATION,
                TRACE_FLAG_GENERAL,
                b"DiskGetDetectInfo: found match for signature %#08lx\n\0".as_ptr() as *const i8,
                (*diskData).__bindgen_anon_1.Mbr.as_ref().Signature,
            );
            found = 1;
            break;
        } else if (*diskData).__bindgen_anon_1.Mbr.as_ref().Signature == 0
            && (*diskData).__bindgen_anon_1.Mbr.as_ref().MbrCheckSum != 0
            && (*diskData).__bindgen_anon_1.Mbr.as_ref().MbrCheckSum == (*diskInfo).MbrCheckSum
        {
            TracePrint(
                TRACE_LEVEL_INFORMATION,
                TRACE_FLAG_GENERAL,
                b"DiskGetDetectInfo: found match for xsum %#08lx\n\0".as_ptr() as *const i8,
                (*diskData).__bindgen_anon_1.Mbr.as_ref().MbrCheckSum,
            );
            found = 1;
            break;
        } else {
            i = i.wrapping_add(1);
        }
    }

    if found != 0 {
        (*DetectInfo).DetectionType = (DetectInt13) as u32;
        (*DetectInfo).__bindgen_anon_1.__bindgen_anon_1.Int13.DriveSelect =
            (*diskInfo).DriveParameters.DriveSelect;
        (*DetectInfo).__bindgen_anon_1.__bindgen_anon_1.Int13.MaxCylinders =
            (*diskInfo).DriveParameters.MaxCylinders;
        (*DetectInfo).__bindgen_anon_1.__bindgen_anon_1.Int13.SectorsPerTrack =
            (*diskInfo).DriveParameters.SectorsPerTrack;
        (*DetectInfo).__bindgen_anon_1.__bindgen_anon_1.Int13.MaxHeads =
            (*diskInfo).DriveParameters.MaxHeads;
        (*DetectInfo).__bindgen_anon_1.__bindgen_anon_1.Int13.NumberDrives =
            (*diskInfo).DriveParameters.NumberDrives;
        core::ptr::write_bytes(
            &raw mut (*DetectInfo).__bindgen_anon_1.__bindgen_anon_1.ExInt13 as *mut u8,
            0,
            core::mem::size_of_val(&(*DetectInfo).__bindgen_anon_1.__bindgen_anon_1.ExInt13),
        );
    }

    if found != 0 {
        STATUS_SUCCESS
    } else {
        STATUS_UNSUCCESSFUL
    }
}





#[no_mangle]
pub unsafe extern "C" fn DiskReadSignature(
    Fdo: PDEVICE_OBJECT,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut fdoExtension: PFUNCTIONAL_DEVICE_EXTENSION =
        (*Fdo).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    let mut diskData: PDISK_DATA =
        (*fdoExtension).__bindgen_anon_1.CommonExtension.as_ref().DriverData as PDISK_DATA;
    let mut Signature: DISK_SIGNATURE = core::mem::zeroed();

    // PAGED_CODE() expansion
    if KeGetCurrentIrql() as i32 > APC_LEVEL as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0 == 0 {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0" as *const u8 as *const i8,
                b"drivers/storage/class/disk/geometry.c\0"
                    as *const u8 as *const i8,
                1650i32,
                b"0\0" as *const u8 as *const i8,
            );
            __int2c();
        }
    }

    Status = IoReadDiskSignature(
        Fdo,
        (*fdoExtension).DiskGeometry.BytesPerSector,
        core::ptr::addr_of_mut!(Signature),
    );

    if !NT_SUCCESS(Status) {
        return Status;
    }

    if Signature.PartitionStyle == PARTITION_STYLE_GPT as u32 {
        (*diskData).PartitionStyle = PARTITION_STYLE_GPT;
        (*diskData).__bindgen_anon_1.Efi.as_mut().DiskId = Signature.__bindgen_anon_1.Gpt.DiskId;
    } else if Signature.PartitionStyle == PARTITION_STYLE_MBR as u32 {
        (*diskData).PartitionStyle = PARTITION_STYLE_MBR;
        (*diskData).__bindgen_anon_1.Mbr.as_mut().Signature = Signature.__bindgen_anon_1.Mbr.Signature;
        (*diskData).__bindgen_anon_1.Mbr.as_mut().MbrCheckSum = Signature.__bindgen_anon_1.Mbr.CheckSum;
    } else {
        if 0 == 0 {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0" as *const u8 as *const i8,
                b"drivers/storage/class/disk/geometry.c\0"
                    as *const u8 as *const i8,
                1668i32,
                b"FALSE\0" as *const u8 as *const i8,
            );
            __int2c();
        }
        Status = STATUS_UNSUCCESSFUL;
    }

    return Status;
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
//     geometry.c
//
// Abstract:
//
//     SCSI disk class driver - this module contains all the code for generating
//     disk geometries.
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
//
// #include "disk.h"
// #include "ntddstor.h"
//
// #ifdef DEBUG_USE_WPP
// #include "geometry.tmh"
// #endif
//
// #if defined(_X86_) || defined(_AMD64_)
//
// DISK_GEOMETRY_SOURCE
// DiskUpdateGeometry(
//     IN PFUNCTIONAL_DEVICE_EXTENSION DeviceExtension
//     );
//
// NTSTATUS
// DiskUpdateRemovableGeometry (
//     IN PFUNCTIONAL_DEVICE_EXTENSION FdoExtension
//     );
//
// VOID
// DiskScanBusDetectInfo(
//     IN PDRIVER_OBJECT DriverObject,
//     IN HANDLE BusKey
//     );
//
// NTSTATUS
// DiskSaveBusDetectInfo(
//     IN PDRIVER_OBJECT DriverObject,
//     IN HANDLE TargetKey,
//     IN ULONG DiskNumber
//     );
//
// NTSTATUS
// DiskSaveGeometryDetectInfo(
//     IN PDRIVER_OBJECT DriverObject,
//     IN HANDLE HardwareKey
//     );
//
// NTSTATUS
// DiskGetPortGeometry(
//     IN PFUNCTIONAL_DEVICE_EXTENSION FdoExtension,
//     OUT PDISK_GEOMETRY Geometry
//     );
//
// typedef struct _DISK_DETECT_INFO {
//     BOOLEAN Initialized;
//     ULONG Style;
//     ULONG Signature;
//     ULONG MbrCheckSum;
//     PDEVICE_OBJECT Device;
//     CM_INT13_DRIVE_PARAMETER DriveParameters;
// } DISK_DETECT_INFO, *PDISK_DETECT_INFO;
//
// //
// // Information about the disk geometries collected and saved into the registry
// // by NTDETECT.COM or the system firmware.
// //
//
// PDISK_DETECT_INFO DetectInfoList = NULL;
// ULONG DetectInfoCount            = 0;
// LONG DetectInfoUsedCount        = 0;
//
// #define GET_STARTING_SECTOR(p)                (     \
//         (ULONG) (p->StartingSectorLsb0)       +     \
//         (ULONG) (p->StartingSectorLsb1 << 8 ) +     \
//         (ULONG) (p->StartingSectorMsb0 << 16) +     \
//         (ULONG) (p->StartingSectorMsb1 << 24) )
//
// #define GET_ENDING_S_OF_CHS(p)                (     \
//         (UCHAR) (p->EndingCylinderLsb & 0x3F) )
//
// //
// // Definitions from hal.h
// //
//
// //
// // Boot record disk partition table entry structure format
// //
//
// typedef struct _PARTITION_DESCRIPTOR
// {
//     UCHAR ActiveFlag;
//     UCHAR StartingTrack;
//     UCHAR StartingCylinderLsb;
//     UCHAR StartingCylinderMsb;
//     UCHAR PartitionType;
//     UCHAR EndingTrack;
//     UCHAR EndingCylinderLsb;
//     UCHAR EndingCylinderMsb;
//     UCHAR StartingSectorLsb0;
//     UCHAR StartingSectorLsb1;
//     UCHAR StartingSectorMsb0;
//     UCHAR StartingSectorMsb1;
//     UCHAR PartitionLengthLsb0;
//     UCHAR PartitionLengthLsb1;
//     UCHAR PartitionLengthMsb0;
//     UCHAR PartitionLengthMsb1;
//
// } PARTITION_DESCRIPTOR, *PPARTITION_DESCRIPTOR;
//
// //
// // Number of partition table entries
// //
//
// #define NUM_PARTITION_TABLE_ENTRIES     4
//
// //
// // Partition table record and boot signature offsets in 16-bit words
// //
//
// #define PARTITION_TABLE_OFFSET          ( 0x1be / 2)
// #define BOOT_SIGNATURE_OFFSET           ((0x200 / 2) - 1)
//
// //
// // Boot record signature value
// //
//
// #define BOOT_RECORD_SIGNATURE           (0xaa55)
//
//
// #ifdef ALLOC_PRAGMA
// #pragma alloc_text(INIT, DiskSaveDetectInfo)
// #pragma alloc_text(INIT, DiskScanBusDetectInfo)
// #pragma alloc_text(INIT, DiskSaveBusDetectInfo)
// #pragma alloc_text(INIT, DiskSaveGeometryDetectInfo)
//
// #pragma alloc_text(PAGE, DiskUpdateGeometry)
// #pragma alloc_text(PAGE, DiskUpdateRemovableGeometry)
// #pragma alloc_text(PAGE, DiskGetPortGeometry)
// #pragma alloc_text(PAGE, DiskIsNT4Geometry)
// #pragma alloc_text(PAGE, DiskGetDetectInfo)
// #pragma alloc_text(PAGE, DiskReadSignature)
// #endif
//
//
// NTSTATUS
// DiskSaveDetectInfo(
//     PDRIVER_OBJECT DriverObject
//     )
//
// /*++
//
// Routine Description:
//
//     This routine saves away the firmware information about the disks which has
//     been saved in the registry.  It generates a list (DetectInfoList) which
//     contains the disk geometries, signatures & checksums of all drives which
//     were examined by NtDetect.  This list is later used to assign geometries
//     to disks as they are initialized.
//
// Arguments:
//
//     DriverObject - the driver being initialized.  This is used to get to the
//                    hardware database.
//
// Return Value:
//
//     status.
//
// --*/
//
// {
//     _OBJECT_ATTRIBUTES objectAttributes = {0};
//     HANDLE hardwareKey;
//
//     _UNICODE_STRING unicodeString;
//     HANDLE busKey;
//
//     NTSTATUS status;
//
//     PAGED_CODE();
//
//     InitializeObjectAttributes(
//         &objectAttributes,
//         DriverObject->HardwareDatabase,
//         OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
//         NULL,
//         NULL);
//
//     //
//     // Create the hardware base key.
//     //
//
//     status = ZwOpenKey(&hardwareKey, KEY_READ, &objectAttributes);
//
//     if(!NT_SUCCESS(status)) {
//         TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_GENERAL, "DiskSaveDetectInfo: Cannot open hardware data. "
//                        "Name: %wZ\n",
//                     DriverObject->HardwareDatabase));
//         return status;
//     }
//
//     status = DiskSaveGeometryDetectInfo(DriverObject, hardwareKey);
//
//     if(!NT_SUCCESS(status)) {
//         TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_GENERAL, "DiskSaveDetectInfo: Can't query configuration data "
//                        "(%#08lx)\n",
//                     status));
//         ZwClose(hardwareKey);
//         return status;
//     }
//
//     //
//     // Open EISA bus key.
//     //
//
//     RtlInitUnicodeString(&unicodeString, L"EisaAdapter");
//     InitializeObjectAttributes(&objectAttributes,
//                                &unicodeString,
//                                OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
//                                hardwareKey,
//                                NULL);
//
//     status = ZwOpenKey(&busKey,
//                        KEY_READ,
//                        &objectAttributes);
//
//     if(NT_SUCCESS(status)) {
//         TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_GENERAL, "DiskSaveDetectInfo: Opened EisaAdapter key\n"));
//         DiskScanBusDetectInfo(DriverObject, busKey);
//         ZwClose(busKey);
//     }
//
//     //
//     // Open MultiFunction bus key.
//     //
//
//     RtlInitUnicodeString(&unicodeString, L"MultifunctionAdapter");
//     InitializeObjectAttributes(&objectAttributes,
//                                &unicodeString,
//                                OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
//                                hardwareKey,
//                                NULL);
//
//     status = ZwOpenKey(&busKey,
//                        KEY_READ,
//                        &objectAttributes);
//
//     if(NT_SUCCESS(status)) {
//         TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_GENERAL, "DiskSaveDetectInfo: Opened MultifunctionAdapter key\n"));
//         DiskScanBusDetectInfo(DriverObject, busKey);
//         ZwClose(busKey);
//     }
//
//     ZwClose(hardwareKey);
//
//     return STATUS_SUCCESS;
// }
//
//
// VOID
// DiskCleanupDetectInfo(
//     IN PDRIVER_OBJECT DriverObject
//     )
// /*++
//
// Routine Description:
//
//     This routine will cleanup the data structure built by DiskSaveDetectInfo.
//
// Arguments:
//
//     DriverObject - a pointer to the kernel object for this driver.
//
// Return Value:
//
//     none
//
// --*/
//
// {
//     UNREFERENCED_PARAMETER(DriverObject);
//     FREE_POOL(DetectInfoList);
//     return;
// }
//
//
// NTSTATUS
// DiskSaveGeometryDetectInfo(
//     IN PDRIVER_OBJECT DriverObject,
//     IN HANDLE HardwareKey
//     )
// {
//     _UNICODE_STRING unicodeString;
//     PKEY_VALUE_FULL_INFORMATION keyData;
//     ULONG length;
//
//     PCM_FULL_RESOURCE_DESCRIPTOR fullDescriptor;
//     PCM_PARTIAL_RESOURCE_DESCRIPTOR partialDescriptor;
//
//     PCM_INT13_DRIVE_PARAMETER driveParameters;
//     ULONG numberOfDrives;
//
//     ULONG i;
//
//     NTSTATUS status;
//
//     PAGED_CODE();
//     UNREFERENCED_PARAMETER(DriverObject);
//
//     //
//     // Get disk BIOS geometry information.
//     //
//
//     RtlInitUnicodeString(&unicodeString, L"Configuration Data");
//
//     keyData = ExAllocatePoolWithTag(PagedPool,
//                                     VALUE_BUFFER_SIZE,
//                                     DISK_TAG_UPDATE_GEOM);
//
//     if(keyData == NULL) {
//         TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_GENERAL, "DiskSaveGeometryDetectInfo: Can't allocate config "
//                        "data buffer\n"));
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     status = ZwQueryValueKey(HardwareKey,
//                              &unicodeString,
//                              KeyValueFullInformation,
//                              keyData,
//                              VALUE_BUFFER_SIZE,
//                              &length);
//
//     if(!NT_SUCCESS(status)) {
//         TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_GENERAL, "DiskSaveGeometryDetectInfo: Can't query configuration "
//                        "data (%#08lx)\n",
//                     status));
//         FREE_POOL(keyData);
//         return status;
//     }
//
//     //
//     // Extract the resource list out of the key data.
//     //
//
//     fullDescriptor = (PCM_FULL_RESOURCE_DESCRIPTOR)
//                       (((PUCHAR) keyData) + keyData->DataOffset);
//     partialDescriptor =
//         fullDescriptor->PartialResourceList.PartialDescriptors;
//     length = partialDescriptor->u.DeviceSpecificData.DataSize;
//
//     if((keyData->DataLength < sizeof(CM_FULL_RESOURCE_DESCRIPTOR)) ||
//        (fullDescriptor->PartialResourceList.Count == 0) ||
//        (partialDescriptor->Type != CmResourceTypeDeviceSpecific) ||
//        (length < sizeof(ULONG))) {
//
//            TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_GENERAL, "DiskSaveGeometryDetectInfo: BIOS header data too small "
//                        "or invalid\n"));
//         FREE_POOL(keyData);
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     //
//     // Point to the BIOS data.  THe BIOS data is located after the first
//     // partial Resource list which should be device specific data.
//     //
//
//     {
//         PUCHAR buffer = (PUCHAR) keyData;
//         buffer += keyData->DataOffset;
//         buffer += sizeof(CM_FULL_RESOURCE_DESCRIPTOR);
//         driveParameters = (PCM_INT13_DRIVE_PARAMETER) buffer;
//     }
//
//     numberOfDrives = length / sizeof(CM_INT13_DRIVE_PARAMETER);
//
//     //
//     // Allocate our detect info list now that we know how many entries there
//     // are going to be.  No other routine allocates detect info and this is
//     // done out of DriverEntry so we don't need to synchronize it's creation.
//     //
//
//     length = sizeof(DISK_DETECT_INFO) * numberOfDrives;
//     DetectInfoList = ExAllocatePoolWithTag(PagedPool,
//                                            length,
//                                            DISK_TAG_UPDATE_GEOM);
//
//     if(DetectInfoList == NULL) {
//         TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_GENERAL, "DiskSaveGeometryDetectInfo: Couldn't allocate %x bytes "
//                        "for DetectInfoList\n",
//                     length));
//
//         FREE_POOL(keyData);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     DetectInfoCount = numberOfDrives;
//
//     RtlZeroMemory(DetectInfoList, length);
//
//     //
//     // Copy the information out of the key data and into the list we've
//     // allocated.
//     //
//
//     for(i = 0; i < numberOfDrives; i++) {
// #ifdef _MSC_VER
// #pragma warning(suppress: 6386) // PREFast bug means it doesn't correctly remember the size of DetectInfoList
// #endif
//         DetectInfoList[i].DriveParameters = driveParameters[i];
//     }
//
//     FREE_POOL(keyData);
//     return STATUS_SUCCESS;
// }
//
//
// VOID
// DiskScanBusDetectInfo(
//     IN PDRIVER_OBJECT DriverObject,
//     IN HANDLE BusKey
//     )
// /*++
//
// Routine Description:
//
//     The routine queries the registry to determine which disks are visible to
//     the BIOS.  If a disk is visable to the BIOS then the geometry information
//     is updated with the disk's signature and MBR checksum.
//
// Arguments:
//
//     DriverObject - the object for this driver.
//     BusKey - handle to the bus key to be enumerated.
//
// Return Value:
//
//     status
//
// --*/
// {
//     ULONG busNumber;
//
//     NTSTATUS status;
//
//     for(busNumber = 0; ; busNumber++) {
//
//         WCHAR buffer[32] = { 0 };
//         _UNICODE_STRING unicodeString;
//
//         _OBJECT_ATTRIBUTES objectAttributes = {0};
//
//         HANDLE spareKey;
//         HANDLE adapterKey;
//
//         ULONG adapterNumber;
//
//         TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_GENERAL, "DiskScanBusDetectInfo: Scanning bus %d\n", busNumber));
//
//         //
//         // Open controller name key.
//         //
//
//         status = RtlStringCchPrintfW(buffer, sizeof(buffer) / sizeof(buffer[0]) - 1, L"%d", busNumber);
//         if (!NT_SUCCESS(status)) {
//             TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_GENERAL, "DiskScanBusDetectInfo: Format symbolic link failed with error: 0x%X\n", status));
//             break;
//         }
//
//         RtlInitUnicodeString(&unicodeString, buffer);
//
//         InitializeObjectAttributes(&objectAttributes,
//                                    &unicodeString,
//                                    OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
//                                    BusKey,
//                                    NULL);
//
//         status = ZwOpenKey(&spareKey, KEY_READ, &objectAttributes);
//
//         if(!NT_SUCCESS(status)) {
//             TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_GENERAL, "DiskScanBusDetectInfo: Error %#08lx opening bus "
//                            "key %#x\n",
//                         status, busNumber));
//             break;
//         }
//
//         //
//         // Open up a controller ordinal key.
//         //
//
//         RtlInitUnicodeString(&unicodeString, L"DiskController");
//         InitializeObjectAttributes(&objectAttributes,
//                                    &unicodeString,
//                                    OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
//                                    spareKey,
//                                    NULL);
//
//         status = ZwOpenKey(&adapterKey, KEY_READ, &objectAttributes);
//         ZwClose(spareKey);
//
//         if(!NT_SUCCESS(status)) {
//             TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_GENERAL, "DiskScanBusDetectInfo: Error %#08lx opening "
//                            "DiskController key\n",
//                            status));
//             continue;
//         }
//
//         for(adapterNumber = 0; ; adapterNumber++) {
//
//             HANDLE diskKey;
//             ULONG diskNumber;
//
//             //
//             // Open disk key.
//             //
//
//             TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_GENERAL, "DiskScanBusDetectInfo: Scanning disk key "
//                            "%d\\DiskController\\%d\\DiskPeripheral\n",
//                            busNumber, adapterNumber));
//
//             status = RtlStringCchPrintfW(buffer, sizeof(buffer) / sizeof(buffer[0]) - 1, L"%d\\DiskPeripheral", adapterNumber);
//             if (!NT_SUCCESS(status)) {
//                 TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_GENERAL, "DiskScanBusDetectInfo: Format symbolic link failed with error: 0x%X\n", status));
//                 break;
//             }
//
//             RtlInitUnicodeString(&unicodeString, buffer);
//
//             InitializeObjectAttributes(&objectAttributes,
//                                        &unicodeString,
//                                        OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
//                                        adapterKey,
//                                        NULL);
//
//             status = ZwOpenKey(&diskKey, KEY_READ, &objectAttributes);
//
//             if(!NT_SUCCESS(status)) {
//                 TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_GENERAL, "DiskScanBusDetectInfo: Error %#08lx opening "
//                                "disk key\n",
//                                status));
//                 break;
//             }
//
//             for(diskNumber = 0; ; diskNumber++) {
//
//                 HANDLE targetKey;
//
//                 TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_GENERAL, "DiskScanBusDetectInfo: Scanning target key "
//                                "%d\\DiskController\\%d\\DiskPeripheral\\%d\n",
//                                busNumber, adapterNumber, diskNumber));
//
//                 status = RtlStringCchPrintfW(buffer, sizeof(buffer) / sizeof(buffer[0]) - 1, L"%d", diskNumber);
//                 if (!NT_SUCCESS(status)) {
//                     TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_GENERAL, "DiskScanBusDetectInfo: Format symbolic link failed with error: 0x%X\n", status));
//                     break;
//                 }
//
//                 RtlInitUnicodeString(&unicodeString, buffer);
//
//                 InitializeObjectAttributes(&objectAttributes,
//                                            &unicodeString,
//                                            OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
//                                            diskKey,
//                                            NULL);
//
//                 status = ZwOpenKey(&targetKey, KEY_READ, &objectAttributes);
//
//                 if(!NT_SUCCESS(status)) {
//                     TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_GENERAL, "DiskScanBusDetectInfo: Error %#08lx "
//                                    "opening target key\n",
//                                 status));
//                     break;
//                 }
//
//                 DiskSaveBusDetectInfo(DriverObject, targetKey, diskNumber);
//
//                 ZwClose(targetKey);
//             }
//
//             ZwClose(diskKey);
//         }
//
//         ZwClose(adapterKey);
//     }
//
//     return;
// }
//
//
// NTSTATUS
// DiskSaveBusDetectInfo(
//     IN PDRIVER_OBJECT DriverObject,
//     IN HANDLE TargetKey,
//     IN ULONG DiskNumber
//     )
// /*++
//
// Routine Description:
//
//     This routine will transfer the firmware/ntdetect reported information
//     in the specified target key into the appropriate entry in the
//     DetectInfoList.
//
// Arguments:
//
//     DriverObject - the object for this driver.
//
//     TargetKey - the key for the disk being saved.
//
//     DiskNumber - the ordinal of the entry in the DiskPeripheral tree for this
//                  entry
//
// Return Value:
//
//     status
//
// --*/
// {
//     PDISK_DETECT_INFO diskInfo;
//
//     _UNICODE_STRING unicodeString;
//
//     PKEY_VALUE_FULL_INFORMATION keyData;
//     ULONG length;
//
//     NTSTATUS status;
//
//     PAGED_CODE();
//     UNREFERENCED_PARAMETER(DriverObject);
//
//     if (DiskNumber >= DetectInfoCount)
//     {
//         return STATUS_UNSUCCESSFUL;
//     }
//
//     diskInfo = &(DetectInfoList[DiskNumber]);
//
//     if(diskInfo->Initialized) {
//
//         NT_ASSERT(FALSE);
//         TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_GENERAL, "DiskSaveBusDetectInfo: disk entry %#x already has a "
//                         "signature of %#08lx and mbr checksum of %#08lx\n",
//                         DiskNumber,
//                         diskInfo->Signature,
//                         diskInfo->MbrCheckSum));
//         return STATUS_UNSUCCESSFUL;
//     }
//
//     RtlInitUnicodeString(&unicodeString, L"Identifier");
//
//     keyData = ExAllocatePoolWithTag(PagedPool,
//                                     VALUE_BUFFER_SIZE,
//                                     DISK_TAG_UPDATE_GEOM);
//
//     if(keyData == NULL) {
//         TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_GENERAL, "DiskSaveBusDetectInfo: Couldn't allocate space for "
//                        "registry data\n"));
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     //
//     // Get disk peripheral identifier.
//     //
//
//     status = ZwQueryValueKey(TargetKey,
//                              &unicodeString,
//                              KeyValueFullInformation,
//                              keyData,
//                              VALUE_BUFFER_SIZE,
//                              &length);
//
//     if(!NT_SUCCESS(status)) {
//         TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_GENERAL, "DiskSaveBusDetectInfo: Error %#08lx getting "
//                        "Identifier\n",
//                     status));
//         FREE_POOL(keyData);
//         return status;
//
//     } else if (keyData->DataLength < 9*sizeof(WCHAR)) {
//
//         //
//         // the data is too short to use (we subtract 9 chars in normal path)
//         //
//         TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_GENERAL, "DiskSaveBusDetectInfo: Saved data was invalid, "
//                     "not enough data in registry!\n"));
//         FREE_POOL(keyData);
//         return STATUS_UNSUCCESSFUL;
//
//     } else {
//
//         _UNICODE_STRING identifier;
//         ULONG value;
//
//         //
//         // Complete unicode string.
//         //
//
//         identifier.Buffer = (PWSTR) ((PUCHAR)keyData + keyData->DataOffset);
//         identifier.Length = (USHORT) keyData->DataLength;
//         identifier.MaximumLength = (USHORT) keyData->DataLength;
//
//         //
//         // Get the first value out of the identifier - this will be the MBR
//         // checksum.
//         //
//
//         status = RtlUnicodeStringToInteger(&identifier, 16, &value);
//
//         if(!NT_SUCCESS(status)) {
//             TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_GENERAL, "DiskSaveBusDetectInfo: Error %#08lx converting "
//                            "identifier %wZ into MBR xsum\n",
//                            status,
//                            &identifier));
//             FREE_POOL(keyData);
//             return status;
//         }
//
//         diskInfo->MbrCheckSum = value;
//
//         //
//         // Shift the string over to get the disk signature
//         //
//
//         identifier.Buffer += 9;
//         identifier.Length -= 9 * sizeof(WCHAR);
//         identifier.MaximumLength -= 9 * sizeof(WCHAR);
//
//         status = RtlUnicodeStringToInteger(&identifier, 16, &value);
//
//         if(!NT_SUCCESS(status)) {
//             TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_GENERAL, "DiskSaveBusDetectInfo: Error %#08lx converting "
//                            "identifier %wZ into disk signature\n",
//                            status,
//                            &identifier));
//             value = 0;
//         }
//
//         diskInfo->Signature = value;
//     }
//
//     //
//     // Here is where we would save away the extended int13 data.
//     //
//
//     //
//     // Mark this entry as initialized so we can make sure not to do it again.
//     //
//
//     diskInfo->Initialized = TRUE;
//
//     FREE_POOL(keyData);
//
//     return STATUS_SUCCESS;
// }
//
//
// DISK_GEOMETRY_SOURCE
// DiskUpdateGeometry(
//     IN PFUNCTIONAL_DEVICE_EXTENSION FdoExtension
//     )
// /*++
//
// Routine Description:
//
//     This routine checks the DetectInfoList saved away during disk driver init
//     to see if any geometry information was reported for this drive.  If the
//     geometry data exists (determined by matching non-zero signatures or
//     non-zero MBR checksums) then it will be saved in the RealGeometry member
//     of the disk data block.
//
//     ClassReadDriveCapacity MUST be called after calling this routine to update
//     the cylinder count based on the size of the disk and the presence of any
//     disk management software.
//
// Arguments:
//
//     DeviceExtension - Supplies a pointer to the device information for disk.
//
// Return Value:
//
//     Inidicates whether the "RealGeometry" in the data block is now valid.
//
// --*/
//
// {
//     PDISK_DATA diskData = FdoExtension->CommonExtension.DriverData;
//
//     ULONG i;
//     PDISK_DETECT_INFO diskInfo = NULL;
//
//     BOOLEAN found = FALSE;
//
//     NTSTATUS status;
//
//     PAGED_CODE();
//
//
//     NT_ASSERT((FdoExtension->DeviceObject->Characteristics & FILE_REMOVABLE_MEDIA) == 0);
//
//     //
//     // If we've already set a non-default geometry for this drive then there's
//     // no need to try and update again.
//     //
//
//     if(diskData->GeometrySource != DiskGeometryUnknown) {
//         return diskData->GeometrySource;
//     }
//
//     //
//     // Scan through the saved detect info to see if we can find a match
//     // for this device.
//     //
//
//     for(i = 0; i < DetectInfoCount; i++) {
//
//         NT_ASSERT(DetectInfoList != NULL);
//
//         diskInfo = &(DetectInfoList[i]);
//
//         if((diskData->Mbr.Signature != 0) &&
//            (diskData->Mbr.Signature == diskInfo->Signature)) {
//                TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_GENERAL, "DiskUpdateGeometry: found match for signature "
//                            "%#08lx\n",
//                         diskData->Mbr.Signature));
//             found = TRUE;
//             break;
//         } else if((diskData->Mbr.Signature == 0) &&
//                   (diskData->Mbr.MbrCheckSum != 0) &&
//                   (diskData->Mbr.MbrCheckSum == diskInfo->MbrCheckSum)) {
//                       TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_GENERAL, "DiskUpdateGeometry: found match for xsum %#08lx\n",
//                         diskData->Mbr.MbrCheckSum));
//             found = TRUE;
//             break;
//         }
//     }
//
//     if(found) {
//
//         ULONG cylinders;
//         ULONG sectorsPerTrack;
//         ULONG tracksPerCylinder;
//
//         ULONG length;
//
//         //
//         // Point to the array of drive parameters.
//         //
//
//         cylinders = diskInfo->DriveParameters.MaxCylinders + 1;
//         sectorsPerTrack = diskInfo->DriveParameters.SectorsPerTrack;
//         tracksPerCylinder = diskInfo->DriveParameters.MaxHeads + 1;
//
//         //
//         // Since the BIOS may not report the full drive, recalculate the drive
//         // size based on the volume size and the BIOS values for tracks per
//         // cylinder and sectors per track..
//         //
//
//         length = tracksPerCylinder * sectorsPerTrack;
//
//         if (length == 0) {
//
//             //
//             // The BIOS information is bogus.
//             //
//
//             TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_GENERAL, "DiskUpdateGeometry: H (%d) or S(%d) is zero\n",
//                         tracksPerCylinder, sectorsPerTrack));
//             return DiskGeometryUnknown;
//         }
//
//         //
//         // since we are copying the structure RealGeometry here, we should
//         // really initialize all the fields, especially since a zero'd
//         // BytesPerSector field would cause a trap in xHalReadPartitionTable()
//         //
//
//         diskData->RealGeometry = FdoExtension->DiskGeometry;
//
//         //
//         // Save the geometry information away in the disk data block and
//         // set the bit indicating that we found a valid one.
//         //
//
//         diskData->RealGeometry.SectorsPerTrack = sectorsPerTrack;
//         diskData->RealGeometry.TracksPerCylinder = tracksPerCylinder;
//         diskData->RealGeometry.Cylinders.QuadPart = (LONGLONG)cylinders;
//
//         TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_GENERAL, "DiskUpdateGeometry: BIOS spt %#x, #heads %#x, "
//                        "#cylinders %#x\n",
//                    sectorsPerTrack, tracksPerCylinder, cylinders));
//
//         diskData->GeometrySource = DiskGeometryFromBios;
//         diskInfo->Device = FdoExtension->DeviceObject;
//
//         //
//         // Increment the count of used geometry entries.
//         //
//
//         InterlockedIncrement(&DetectInfoUsedCount);
//
//     } else {
//
//         TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_GENERAL, "DiskUpdateGeometry: no match found for signature %#08lx\n", diskData->Mbr.Signature));
//     }
//
//     if(diskData->GeometrySource == DiskGeometryUnknown) {
//
//         //
//         // We couldn't find a geometry from the BIOS.  Check with the port
//         // driver and see if it can provide one.
//         //
//
//         status = DiskGetPortGeometry(FdoExtension, &(diskData->RealGeometry));
//
//         if(NT_SUCCESS(status)) {
//
//             //
//             // Check the geometry to make sure it's valid.
//             //
//
//             if((diskData->RealGeometry.TracksPerCylinder *
//                 diskData->RealGeometry.SectorsPerTrack) != 0) {
//
//                 diskData->GeometrySource = DiskGeometryFromPort;
//                 TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_GENERAL, "DiskUpdateGeometry: using Port geometry for disk %#p\n", FdoExtension));
//
//                 if (diskData->RealGeometry.BytesPerSector == 0) {
//
//                     TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_GENERAL, "DiskDriverReinit: Port driver failed to "
//                                 "set BytesPerSector in the RealGeometry\n"));
//                     diskData->RealGeometry.BytesPerSector =
//                         FdoExtension->DiskGeometry.BytesPerSector;
//                     if (diskData->RealGeometry.BytesPerSector == 0) {
//                         NT_ASSERT(!"BytesPerSector is still zero!");
//                     }
//
//                 }
//             }
//         }
//     }
//
//     //
//     // If we came up with a "real" geometry for this drive then set it in the
//     // device extension.
//     //
//
//     if (diskData->GeometrySource != DiskGeometryUnknown) {
//
//         FdoExtension->DiskGeometry = diskData->RealGeometry;
//     }
//
//     return diskData->GeometrySource;
// }
//
//
// NTSTATUS
// DiskUpdateRemovableGeometry (
//     IN PFUNCTIONAL_DEVICE_EXTENSION FdoExtension
//     )
//
// /*++
//
// Routine Description:
//
//     This routine updates the geometry of the disk.  It will query the port
//     driver to see if it can provide any geometry info.  If not it will use
//     the current head & sector count.
//
//     Based on these values & the capacity of the drive as reported by
//     ClassReadDriveCapacity it will determine a new cylinder count for the
//     device.
//
// Arguments:
//
//     Fdo - Supplies the functional device object whos size needs to be updated.
//
// Return Value:
//
//     Returns the status of the opertion.
//
// --*/
// {
//     PCOMMON_DEVICE_EXTENSION commonExtension = &(FdoExtension->CommonExtension);
//     PDISK_DATA diskData = commonExtension->DriverData;
//     PDISK_GEOMETRY geometry = &(diskData->RealGeometry);
//
//     NTSTATUS status;
//
//     PAGED_CODE();
//
//     if (FdoExtension->DeviceDescriptor) {
//         NT_ASSERT(FdoExtension->DeviceDescriptor->RemovableMedia);
//     }
//     NT_ASSERT(TEST_FLAG(FdoExtension->DeviceObject->Characteristics,
//                      FILE_REMOVABLE_MEDIA));
//
//     //
//     // Attempt to determine the disk geometry.  First we'll check with the
//     // port driver to see what it suggests for a value.
//     //
//
//     status = DiskGetPortGeometry(FdoExtension, geometry);
//
//     if(NT_SUCCESS(status) &&
//        ((geometry->TracksPerCylinder * geometry->SectorsPerTrack) != 0)) {
//
//         FdoExtension->DiskGeometry = (*geometry);
//     }
//
//     return status;
// }
//
//
// NTSTATUS
// DiskGetPortGeometry(
//     IN PFUNCTIONAL_DEVICE_EXTENSION FdoExtension,
//     OUT PDISK_GEOMETRY Geometry
//     )
// /*++
//
// Routine Description:
//
//     This routine will query the port driver for disk geometry.  Some port
//     drivers (in particular IDEPORT) may be able to provide geometry for the
//     device.
//
// Arguments:
//
//     FdoExtension - the device object for the disk.
//
//     Geometry - a structure to save the geometry information into (if any is
//                available)
//
// Return Value:
//
//     STATUS_SUCCESS if geometry information can be provided or
//     error status indicating why it can't.
//
// --*/
// {
//     PCOMMON_DEVICE_EXTENSION commonExtension = &(FdoExtension->CommonExtension);
//     PIRP irp;
//     PIO_STACK_LOCATION irpStack;
//     KEVENT event;
//
//     NTSTATUS status;
//
//     PAGED_CODE();
//
//     //
//     // Build an irp to send IOCTL_DISK_GET_DRIVE_GEOMETRY to the lower driver.
//     //
//
//     irp = IoAllocateIrp(commonExtension->LowerDeviceObject->StackSize, FALSE);
//
//     if(irp == NULL) {
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     irpStack = IoGetNextIrpStackLocation(irp);
//
//     irpStack->MajorFunction = IRP_MJ_DEVICE_CONTROL;
//
//     irpStack->Parameters.DeviceIoControl.IoControlCode =
//         IOCTL_DISK_GET_DRIVE_GEOMETRY;
//     irpStack->Parameters.DeviceIoControl.OutputBufferLength =
//         sizeof(DISK_GEOMETRY);
//
//     irp->AssociatedIrp.SystemBuffer = Geometry;
//
//     KeInitializeEvent(&event, SynchronizationEvent, FALSE);
//
//     IoSetCompletionRoutine(irp,
//                            ClassSignalCompletion,
//                            &event,
//                            TRUE,
//                            TRUE,
//                            TRUE);
//
//     status = IoCallDriver(commonExtension->LowerDeviceObject, irp);
//     if (status == STATUS_PENDING) {
//         KeWaitForSingleObject(&event, Executive, KernelMode, FALSE, NULL);
//         status = irp->IoStatus.Status;
//     }
//
//     IoFreeIrp(irp);
//
//     return status;
// }
//
//
// BOOLEAN
// DiskIsNT4Geometry(
//     IN PFUNCTIONAL_DEVICE_EXTENSION FdoExtension
//     )
//
// /*++
//
// Routine Description:
//
//     The default geometry that was used in partitioning disks under Windows NT 4.0 was
//
//     Sectors per Track   = 0x20 =  32
//     Tracks per Cylinder = 0x40 =  64
//
//     This was changed in Windows 2000 to
//
//     Sectors per Track   = 0x3F =  63
//     Tracks per Cylinder = 0xFF = 255
//
//     If neither the BIOS nor the port driver can report the correct geometry,  we will
//     default to the new numbers on such disks. Now LVM uses the geometry when creating
//     logical volumes and dynamic disks.  So reporting an incorrect geometry will cause
//     the entire extended partition / dynamic disk to be destroyed
//
//     In this routine, we will look at the Master Boot Record. In 90% of the cases, the
//     first entry corresponds to a partition that starts on the first track. If this is
//     so,  we shall retrieve the logical block address associated with it and calculate
//     the correct geometry.  Now, all partitions start on a cylinder boundary.  So, for
//     the remaining 10% we will look at the ending CHS number to determine the geometry
//
// --*/
//
// {
//     PUSHORT readBuffer = NULL;
//     BOOLEAN bFoundNT4 = FALSE;
//
//     PAGED_CODE();
//
//     readBuffer = ExAllocatePoolWithTag(NonPagedPoolNx, FdoExtension->DiskGeometry.BytesPerSector, DISK_TAG_UPDATE_GEOM);
//
//     if (readBuffer)
//     {
//         KEVENT event;
//         LARGE_INTEGER diskOffset;
//         _IO_STATUS_BLOCK ioStatus = { 0 };
//         PIRP irp;
//
//         KeInitializeEvent(&event, NotificationEvent, FALSE);
//
//         //
//         // Read the Master Boot Record at disk offset 0
//         //
//
//         diskOffset.QuadPart = 0;
//
//         irp = IoBuildSynchronousFsdRequest(IRP_MJ_READ, FdoExtension->DeviceObject, readBuffer, FdoExtension->DiskGeometry.BytesPerSector, &diskOffset, &event, &ioStatus);
//
//         if (irp)
//         {
//             PIO_STACK_LOCATION irpSp = IoGetNextIrpStackLocation(irp);
//             NTSTATUS status;
//
//             irpSp->Flags |= SL_OVERRIDE_VERIFY_VOLUME;
//
//             status = IoCallDriver(FdoExtension->DeviceObject, irp);
//
//             if (status == STATUS_PENDING)
//             {
//                 KeWaitForSingleObject(&event, Executive, KernelMode, FALSE, NULL);
//                 status = ioStatus.Status;
//             }
//
//             if (NT_SUCCESS(status))
//             {
//                 //
//                 // Match the boot record signature
//                 //
//
//                 if (readBuffer[BOOT_SIGNATURE_OFFSET] == BOOT_RECORD_SIGNATURE)
//                 {
//                     PPARTITION_DESCRIPTOR partitionTableEntry = (PPARTITION_DESCRIPTOR)&readBuffer[PARTITION_TABLE_OFFSET];
//                     ULONG uCount = 0;
//
//                     //
//                     // Walk the entries looking for a clue as to what the geometry might be
//                     //
//
//                     for (uCount = 0; uCount < NUM_PARTITION_TABLE_ENTRIES; uCount++)
//                     {
//                         //
//                         // We are only concerned if there might be a logical volume or if this disk is part of a dynamic set
//                         //
//
//                         if (IsContainerPartition(partitionTableEntry->PartitionType) || partitionTableEntry->PartitionType == PARTITION_LDM)
//                         {
//                             //
//                             // In 90% of the cases, the first entry corresponds to a partition that starts on the first track
//                             //
//
//                             if (partitionTableEntry->StartingTrack == 1 && GET_STARTING_SECTOR(partitionTableEntry) == 0x20)
//                             {
//                                 bFoundNT4 = TRUE;
//                                 break;
//                             }
//
//                             //
//                             // In almost every case, the ending CHS number is on a cylinder boundary
//                             //
//
//                             if (partitionTableEntry->EndingTrack == 0x3F && GET_ENDING_S_OF_CHS(partitionTableEntry) == 0x20)
//                             {
//                                 bFoundNT4 = TRUE;
//                                 break;
//                             }
//                         }
//
//                         partitionTableEntry++;
//                     }
//                 }
//                 else
//                 {
//                     //
//                     // The Master Boot Record is invalid
//                     //
//                 }
//             }
//         }
//
//         FREE_POOL(readBuffer);
//     }
//
//     return bFoundNT4;
// }
//
//
// NTSTATUS
// DiskReadDriveCapacity(
//     IN PDEVICE_OBJECT Fdo
//     )
// /*++
//
// Routine Description:
//
//     This routine is used by disk.sys as a wrapper for the classpnp API
//     ClassReadDriveCapacity.  It will perform some additional operations to
//     attempt to determine drive geometry before it calls the classpnp version
//     of the routine.
//
//     For fixed disks this involves calling DiskUpdateGeometry which will check
//     various sources (the BIOS, the port driver) for geometry information.
//
// Arguments:
//
//     Fdo - a pointer to the device object to be checked.
//
// Return Value:
//
//     status of ClassReadDriveCapacity.
//
// --*/
//
// {
//     PFUNCTIONAL_DEVICE_EXTENSION fdoExtension = Fdo->DeviceExtension;
//     NTSTATUS status;
//
//     if (TEST_FLAG(Fdo->Characteristics, FILE_REMOVABLE_MEDIA)) {
//         DiskUpdateRemovableGeometry(fdoExtension);
//     } else {
//         DiskUpdateGeometry(fdoExtension);
//     }
//
//     status = ClassReadDriveCapacity(Fdo);
//
//     return status;
// }
//
//
// VOID
// DiskDriverReinitialization(
//     IN PDRIVER_OBJECT DriverObject,
//     IN PVOID Nothing,
//     IN ULONG Count
//     )
// /*++
//
// Routine Description:
//
//     This routine will scan through the current list of disks and attempt to
//     match them to any remaining geometry information.  This will only be done
//     on the first call to the routine.
//
//     Note: This routine assumes that the system will not be adding or removing
//           devices during this phase of the init process.  This is very likely
//           a bad assumption but it greatly simplifies the code.
//
// Arguments:
//
//     DriverObject - a pointer to the object for the disk driver.
//
//     Nothing - unused
//
//     Count - an indication of how many times this routine has been called.
//
// Return Value:
//
//     none
//
// --*/
//
// {
//     PDEVICE_OBJECT deviceObject;
//     PFUNCTIONAL_DEVICE_EXTENSION fdoExtension;
//     PDISK_DATA diskData;
//
//     ULONG unmatchedDiskCount;
//     PDEVICE_OBJECT unmatchedDisk = NULL;
//
//     ULONG i;
//     PDISK_DETECT_INFO diskInfo = NULL;
//
//     UNREFERENCED_PARAMETER(Nothing);
//
//     if(Count != 1) {
//         TracePrint((TRACE_LEVEL_WARNING, TRACE_FLAG_GENERAL, "DiskDriverReinitialization: ignoring call %d\n",
//                     Count));
//         return;
//     }
//
//     //
//     // Check to see how many entries in the detect info list have been matched.
//     // If there's only one remaining we'll see if we can find a disk to go with
//     // it.
//     //
//
//     if(DetectInfoCount == 0) {
//         TracePrint((TRACE_LEVEL_WARNING, TRACE_FLAG_GENERAL, "DiskDriverReinitialization: no detect info saved\n"));
//         return;
//     }
//
//     if((DetectInfoCount - DetectInfoUsedCount) != 1) {
//         TracePrint((TRACE_LEVEL_WARNING, TRACE_FLAG_GENERAL, "DiskDriverReinitialization: %d of %d geometry entries "
//                        "used - will not attempt match\n", DetectInfoUsedCount, DetectInfoCount));
//         return;
//     }
//
//     //
//     // Scan through the list of disks and see if any of them are missing
//     // geometry information.  If there is only one such disk we'll try to
//     // match it to the unmatched geometry.
//     //
//
//
//     //
//     // ISSUE-2000/5/24-henrygab - figure out if there's a way to keep
//     //                            removals from happening while doing this.
//     //
//     unmatchedDiskCount = 0;
//     for(deviceObject = DriverObject->DeviceObject;
//         deviceObject != NULL;
// #ifdef _MSC_VER
// #pragma prefast(suppress:28175, "Need to access the opaque field to scan through the list of disks")
// #endif
//         deviceObject = deviceObject->NextDevice) {
//
//         fdoExtension = deviceObject->DeviceExtension;
//
//         if (!fdoExtension->CommonExtension.IsFdo) {
//             TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_GENERAL, "DiskDriverReinit: %#p is not an FDO\n",
//                            deviceObject));
//             continue;
//         }
//
//         //
//         // If the geometry for this one is already known then skip it.
//         //
//
//         diskData = fdoExtension->CommonExtension.DriverData;
//         if(diskData->GeometrySource != DiskGeometryUnknown) {
//             TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_GENERAL, "DiskDriverReinit: FDO %#p has a geometry\n",
//                            deviceObject));
//             continue;
//         }
//
//         TracePrint((TRACE_LEVEL_WARNING, TRACE_FLAG_GENERAL, "DiskDriverReinit: FDO %#p has no geometry\n",
//                        deviceObject));
//
//         //
//         // Mark this one as using the default.  It's past the time when disk
//         // might blunder across the geometry info.  If we set the geometry
//         // from the bios we'll reset this field down below.
//         //
//
//         diskData->GeometrySource = DiskGeometryFromDefault;
//
//         //
//         // As long as we've only got one unmatched disk we're fine.
//         //
//
//         unmatchedDiskCount++;
//         if(unmatchedDiskCount > 1) {
//             NT_ASSERT(unmatchedDisk != NULL);
//             TracePrint((TRACE_LEVEL_WARNING, TRACE_FLAG_GENERAL, "DiskDriverReinit: FDO %#p also has no geometry\n",
//                            unmatchedDisk));
//             unmatchedDisk = NULL;
//             break;
//         }
//
//         unmatchedDisk = deviceObject;
//     }
//
//     //
//     // If there's more or less than one ungeometried disk then we can't do
//     // anything about the geometry.
//     //
//
//     if(unmatchedDiskCount != 1) {
//         TracePrint((TRACE_LEVEL_ERROR, TRACE_FLAG_GENERAL, "DiskDriverReinit: Unable to match geometry\n"));
//         return;
//
//     }
//
//     fdoExtension = unmatchedDisk->DeviceExtension;
//     diskData = fdoExtension->CommonExtension.DriverData;
//
//     TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_GENERAL, "DiskDriverReinit: Found possible match\n"));
//
//     //
//     // Find the geometry which wasn't assigned.
//     //
//
//     for(i = 0; i < DetectInfoCount; i++) {
//         if(DetectInfoList[i].Device == NULL) {
//             diskInfo = &(DetectInfoList[i]);
//             break;
//         }
//     }
//
//     if (diskInfo != NULL)
//     {
//         //
//         // Save the geometry information away in the disk data block and
//         // set the bit indicating that we found a valid one.
//         //
//
//         ULONG cylinders;
//         ULONG sectorsPerTrack;
//         ULONG tracksPerCylinder;
//
//         ULONG length;
//
//         //
//         // Point to the array of drive parameters.
//         //
//
//         cylinders = diskInfo->DriveParameters.MaxCylinders + 1;
//         sectorsPerTrack = diskInfo->DriveParameters.SectorsPerTrack;
//         tracksPerCylinder = diskInfo->DriveParameters.MaxHeads + 1;
//
//         //
//         // Since the BIOS may not report the full drive, recalculate the drive
//         // size based on the volume size and the BIOS values for tracks per
//         // cylinder and sectors per track..
//         //
//
//         length = tracksPerCylinder * sectorsPerTrack;
//
//         if (length == 0) {
//
//             //
//             // The BIOS information is bogus.
//             //
//
//             TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_GENERAL, "DiskDriverReinit: H (%d) or S(%d) is zero\n",
//                         tracksPerCylinder, sectorsPerTrack));
//             return;
//         }
//
//         //
//         // since we are copying the structure RealGeometry here, we should
//         // really initialize all the fields, especially since a zero'd
//         // BytesPerSector field would cause a trap in xHalReadPartitionTable()
//         //
//
//         diskData->RealGeometry = fdoExtension->DiskGeometry;
//
//         //
//         // Save the geometry information away in the disk data block and
//         // set the bit indicating that we found a valid one.
//         //
//
//         diskData->RealGeometry.SectorsPerTrack = sectorsPerTrack;
//         diskData->RealGeometry.TracksPerCylinder = tracksPerCylinder;
//         diskData->RealGeometry.Cylinders.QuadPart = (LONGLONG)cylinders;
//
//         TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_GENERAL, "DiskDriverReinit: BIOS spt %#x, #heads %#x, "
//                        "#cylinders %#x\n",
//                    sectorsPerTrack, tracksPerCylinder, cylinders));
//
//         diskData->GeometrySource = DiskGeometryGuessedFromBios;
//         diskInfo->Device = unmatchedDisk;
//
//         //
//         // Now copy the geometry over to the fdo extension and call
//         // classpnp to redetermine the disk size and cylinder count.
//         //
//
//         fdoExtension->DiskGeometry = diskData->RealGeometry;
//
//         (VOID)ClassReadDriveCapacity(unmatchedDisk);
//
//         if (diskData->RealGeometry.BytesPerSector == 0) {
//
//             //
//             // if the BytesPerSector field is set to zero for a disk
//             // listed in the bios, then the system will bugcheck in
//             // xHalReadPartitionTable().  assert here since it is
//             // easier to determine what is happening this way.
//             //
//
//             NT_ASSERT(!"RealGeometry not set to non-zero bps\n");
//         }
//     }
//
//     return;
// }
//
//
// NTSTATUS
// DiskGetDetectInfo(
//     IN PFUNCTIONAL_DEVICE_EXTENSION FdoExtension,
//     OUT PDISK_DETECTION_INFO DetectInfo
//     )
// /*++
//
// Routine Description:
//
//     Get the Int13 information from the BIOS DetectInfoList.
//
// Arguments:
//
//     FdoExtension - Supplies a pointer to the FDO extension that we want to
//             obtain the detect information for.
//
//     DetectInfo - A buffer where the detect information will be copied to.
//
// Return Value:
//
//     NTSTATUS code.
//
// --*/
// {
//     ULONG i;
//     BOOLEAN found = FALSE;
//     PDISK_DETECT_INFO diskInfo = NULL;
//     PDISK_DATA diskData = FdoExtension->CommonExtension.DriverData;
//
//     PAGED_CODE ();
//
//     //
//     // Fail for non-fixed drives.
//     //
//
//     if (TEST_FLAG (FdoExtension->DeviceObject->Characteristics, FILE_REMOVABLE_MEDIA)) {
//         return STATUS_NOT_SUPPORTED;
//     }
//
//     //
//     // There is no GPT detection info, so fail this.
//     //
//
//     if (diskData->PartitionStyle == PARTITION_STYLE_GPT) {
//         return STATUS_NOT_SUPPORTED;
//     }
//
//     for(i = 0; i < DetectInfoCount; i++) {
//
//
//         NT_ASSERT(DetectInfoList != NULL);
//
//         diskInfo = &(DetectInfoList[i]);
//
//         if((diskData->Mbr.Signature != 0) &&
//            (diskData->Mbr.Signature == diskInfo->Signature)) {
//                TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_GENERAL, "DiskGetDetectInfo: found match for signature "
//                            "%#08lx\n",
//                         diskData->Mbr.Signature));
//             found = TRUE;
//             break;
//         } else if((diskData->Mbr.Signature == 0) &&
//                   (diskData->Mbr.MbrCheckSum != 0) &&
//                   (diskData->Mbr.MbrCheckSum == diskInfo->MbrCheckSum)) {
//                       TracePrint((TRACE_LEVEL_INFORMATION, TRACE_FLAG_GENERAL, "DiskGetDetectInfo: found match for xsum %#08lx\n",
//                         diskData->Mbr.MbrCheckSum));
//             found = TRUE;
//             break;
//         }
//     }
//
//     if ( found ) {
//         DetectInfo->DetectionType = DetectInt13;
//         DetectInfo->Int13.DriveSelect = diskInfo->DriveParameters.DriveSelect;
//         DetectInfo->Int13.MaxCylinders = diskInfo->DriveParameters.MaxCylinders;
//         DetectInfo->Int13.SectorsPerTrack = diskInfo->DriveParameters.SectorsPerTrack;
//         DetectInfo->Int13.MaxHeads = diskInfo->DriveParameters.MaxHeads;
//         DetectInfo->Int13.NumberDrives = diskInfo->DriveParameters.NumberDrives;
//         RtlZeroMemory (&DetectInfo->ExInt13, sizeof (DetectInfo->ExInt13));
//     }
//
//     return (found ? STATUS_SUCCESS : STATUS_UNSUCCESSFUL);
// }
//
//
// NTSTATUS
// DiskReadSignature(
//     IN PDEVICE_OBJECT Fdo
//     )
//
// /*++
//
// Routine Description:
//
//     Read the disks signature from the drive. The signature can be either
//     a MBR signature or a GPT/EFI signature.
//
//     The low-level signature reading is done by IoReadDiskSignature().
//
// Arguments:
//
//     Fdo - Pointer to the FDO of a disk to read the signature for.
//
// Return Value:
//
//     NTSTATUS code.
//
// --*/
//
//
// {
//     NTSTATUS Status;
//     PFUNCTIONAL_DEVICE_EXTENSION fdoExtension = Fdo->DeviceExtension;
//     PDISK_DATA diskData = fdoExtension->CommonExtension.DriverData;
//     DISK_SIGNATURE Signature = { 0 };
//
//     PAGED_CODE();
//
//     Status = IoReadDiskSignature (Fdo,
//                                   fdoExtension->DiskGeometry.BytesPerSector,
//                                   &Signature);
//
//     if (!NT_SUCCESS (Status)) {
//         return Status;
//     }
//
//     if (Signature.PartitionStyle == PARTITION_STYLE_GPT) {
//         diskData->PartitionStyle = PARTITION_STYLE_GPT;
//         diskData->Efi.DiskId = Signature.Gpt.DiskId;
//     } else if (Signature.PartitionStyle == PARTITION_STYLE_MBR) {
//         diskData->PartitionStyle = PARTITION_STYLE_MBR;
//         diskData->Mbr.Signature = Signature.Mbr.Signature;
//         diskData->Mbr.MbrCheckSum = Signature.Mbr.CheckSum;
//     } else {
//         NT_ASSERT (FALSE);
//         Status = STATUS_UNSUCCESSFUL;
//     }
//
//     return Status;
// }
//
// #endif // defined(_X86_) || defined(_AMD64_)
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
