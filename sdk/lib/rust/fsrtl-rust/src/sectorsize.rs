//! FsRtlGetSectorSizeInformation (Windows 8): the FILE_FS_SECTOR_SIZE_INFORMATION a file
//! system returns for FileFsSectorSizeInformation, and kernel32 for FileStorageInfo, built from
//! what the storage stack under `DeviceObject` reports.
//!
//! Measured on Windows 11 (hostapps fsquery.win11.txt): a virtio disk under FAT32 or NTFS
//! reads 512 everywhere, flags ALIGNED_DEVICE | PARTITION_ALIGNED_ON_DEVICE | TRIM_ENABLED and
//! both offsets 0; the CD, whose class driver answers no alignment query, reads 2048, flags
//! ALIGNED_DEVICE | PARTITION_ALIGNED_ON_DEVICE and both offsets SSINFO_OFFSET_UNKNOWN. So a
//! device that cannot say how it is aligned is taken to be aligned, with the offsets unknown.
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ddk::{
    IoBuildDeviceIoControlRequest, IofCallDriver, KeInitializeEvent, KeWaitForSingleObject,
    PDEVICE_OBJECT, KEVENT, _IO_STATUS_BLOCK,
};
use ntkrnl_sys::opaque_ptr;

const IOCTL_DISK_GET_DRIVE_GEOMETRY: ULONG = 0x0007_0000;
const IOCTL_CDROM_GET_DRIVE_GEOMETRY: ULONG = 0x0002_404C;
const IOCTL_DISK_GET_PARTITION_INFO_EX: ULONG = 0x0007_0048;
const IOCTL_STORAGE_QUERY_PROPERTY: ULONG = 0x002D_1400;

const STORAGE_ACCESS_ALIGNMENT_PROPERTY: ULONG = 6;
const STORAGE_DEVICE_SEEK_PENALTY_PROPERTY: ULONG = 7;
const STORAGE_DEVICE_TRIM_PROPERTY: ULONG = 8;
const PROPERTY_STANDARD_QUERY: ULONG = 0;

const SSINFO_FLAGS_ALIGNED_DEVICE: ULONG = 0x1;
const SSINFO_FLAGS_PARTITION_ALIGNED_ON_DEVICE: ULONG = 0x2;
const SSINFO_FLAGS_NO_SEEK_PENALTY: ULONG = 0x4;
const SSINFO_FLAGS_TRIM_ENABLED: ULONG = 0x8;
const SSINFO_OFFSET_UNKNOWN: ULONG = 0xFFFF_FFFF;

#[repr(C)]
pub struct FILE_FS_SECTOR_SIZE_INFORMATION {
    pub LogicalBytesPerSector: ULONG,
    pub PhysicalBytesPerSectorForAtomicity: ULONG,
    pub PhysicalBytesPerSectorForPerformance: ULONG,
    pub FileSystemEffectivePhysicalBytesPerSectorForAtomicity: ULONG,
    pub Flags: ULONG,
    pub ByteOffsetForSectorAlignment: ULONG,
    pub ByteOffsetForPartitionAlignment: ULONG,
}

#[repr(C)]
struct DiskGeometry {
    Cylinders: i64,
    MediaType: ULONG,
    TracksPerCylinder: ULONG,
    SectorsPerTrack: ULONG,
    BytesPerSector: ULONG,
}

#[repr(C)]
struct StoragePropertyQuery {
    PropertyId: ULONG,
    QueryType: ULONG,
    AdditionalParameters: [u8; 4],
}

#[repr(C)]
struct AccessAlignmentDescriptor {
    Version: ULONG,
    Size: ULONG,
    BytesPerCacheLine: ULONG,
    BytesOffsetForCacheAlignment: ULONG,
    BytesPerLogicalSector: ULONG,
    BytesPerPhysicalSector: ULONG,
    BytesOffsetForSectorAlignment: ULONG,
}

/// DEVICE_SEEK_PENALTY_DESCRIPTOR and DEVICE_TRIM_DESCRIPTOR share this shape.
#[repr(C)]
struct BooleanDescriptor {
    Version: ULONG,
    Size: ULONG,
    Value: BOOLEAN,
}

/// PARTITION_INFORMATION_EX: style, padding, then StartingOffset; the rest (to 144 bytes)
/// is read only for its size.
#[repr(C)]
struct PartitionInfoEx {
    PartitionStyle: ULONG,
    _pad: ULONG,
    StartingOffset: i64,
    _rest: [u8; 128],
}

unsafe fn ioctl<I, O>(device: PDEVICE_OBJECT, code: ULONG, input: *mut I, output: *mut O) -> NTSTATUS {
    let mut event: KEVENT = core::mem::zeroed();
    let mut iosb: _IO_STATUS_BLOCK = core::mem::zeroed();
    KeInitializeEvent(&mut event, ntkrnl_sys::ddk::_EVENT_TYPE_NotificationEvent as _, 0u8);
    let in_len = if input.is_null() { 0 } else { core::mem::size_of::<I>() as ULONG };
    let irp = IoBuildDeviceIoControlRequest(
        code,
        device,
        input as PVOID,
        in_len,
        output as PVOID,
        core::mem::size_of::<O>() as ULONG,
        0u8,
        &mut event,
        &mut iosb,
    );
    if irp.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    let mut status = IofCallDriver(device, irp);
    if status == STATUS_PENDING {
        KeWaitForSingleObject(&mut event as *mut KEVENT as PVOID, 0, 0, 0, core::ptr::null_mut());
        status = iosb.__bindgen_anon_1.Status;
    }
    status
}

unsafe fn property<O>(device: PDEVICE_OBJECT, id: ULONG, out: *mut O) -> NTSTATUS {
    let mut query = StoragePropertyQuery { PropertyId: id, QueryType: PROPERTY_STANDARD_QUERY, AdditionalParameters: [0; 4] };
    ioctl(device, IOCTL_STORAGE_QUERY_PROPERTY, &mut query, out)
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlGetSectorSizeInformation(
    DeviceObject: PDEVICE_OBJECT,
    SectorSizeInfo: *mut FILE_FS_SECTOR_SIZE_INFORMATION,
) -> NTSTATUS {
    let DeviceObject = opaque_ptr(DeviceObject);
    if DeviceObject.is_null() || SectorSizeInfo.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    let mut geometry: DiskGeometry = core::mem::zeroed();
    let mut status = ioctl(DeviceObject, IOCTL_DISK_GET_DRIVE_GEOMETRY, core::ptr::null_mut::<u8>(), &mut geometry);
    if !NT_SUCCESS(status) {
        status = ioctl(DeviceObject, IOCTL_CDROM_GET_DRIVE_GEOMETRY, core::ptr::null_mut::<u8>(), &mut geometry);
    }
    if !NT_SUCCESS(status) {
        return status;
    }
    let logical = geometry.BytesPerSector;
    if logical == 0 {
        return STATUS_INVALID_DEVICE_REQUEST;
    }

    let mut info = FILE_FS_SECTOR_SIZE_INFORMATION {
        LogicalBytesPerSector: logical,
        PhysicalBytesPerSectorForAtomicity: logical,
        PhysicalBytesPerSectorForPerformance: logical,
        FileSystemEffectivePhysicalBytesPerSectorForAtomicity: logical,
        Flags: SSINFO_FLAGS_ALIGNED_DEVICE | SSINFO_FLAGS_PARTITION_ALIGNED_ON_DEVICE,
        ByteOffsetForSectorAlignment: SSINFO_OFFSET_UNKNOWN,
        ByteOffsetForPartitionAlignment: SSINFO_OFFSET_UNKNOWN,
    };

    let mut align: AccessAlignmentDescriptor = core::mem::zeroed();
    if NT_SUCCESS(property(DeviceObject, STORAGE_ACCESS_ALIGNMENT_PROPERTY, &mut align))
        && align.BytesPerPhysicalSector >= logical
    {
        let physical = align.BytesPerPhysicalSector;
        info.PhysicalBytesPerSectorForAtomicity = physical;
        info.PhysicalBytesPerSectorForPerformance = physical;
        info.FileSystemEffectivePhysicalBytesPerSectorForAtomicity = physical;
        info.ByteOffsetForSectorAlignment = align.BytesOffsetForSectorAlignment;
        info.Flags = if align.BytesOffsetForSectorAlignment == 0 { SSINFO_FLAGS_ALIGNED_DEVICE } else { 0 };

        // The partition is aligned when its start, shifted by the device's own sector
        // offset, falls on a physical sector.
        let mut part: PartitionInfoEx = core::mem::zeroed();
        if NT_SUCCESS(ioctl(DeviceObject, IOCTL_DISK_GET_PARTITION_INFO_EX, core::ptr::null_mut::<u8>(), &mut part)) {
            let start = part.StartingOffset as u64 + align.BytesOffsetForSectorAlignment as u64;
            let rem = (start % physical as u64) as ULONG;
            info.ByteOffsetForPartitionAlignment = if rem == 0 { 0 } else { physical - rem };
            if rem == 0 {
                info.Flags |= SSINFO_FLAGS_PARTITION_ALIGNED_ON_DEVICE;
            }
        } else {
            info.Flags |= SSINFO_FLAGS_PARTITION_ALIGNED_ON_DEVICE;
        }
    }

    let mut seek: BooleanDescriptor = core::mem::zeroed();
    if NT_SUCCESS(property(DeviceObject, STORAGE_DEVICE_SEEK_PENALTY_PROPERTY, &mut seek)) && seek.Value == 0 {
        info.Flags |= SSINFO_FLAGS_NO_SEEK_PENALTY;
    }
    let mut trim: BooleanDescriptor = core::mem::zeroed();
    if NT_SUCCESS(property(DeviceObject, STORAGE_DEVICE_TRIM_PROPERTY, &mut trim)) && trim.Value != 0 {
        info.Flags |= SSINFO_FLAGS_TRIM_ENABLED;
    }

    *SectorSizeInfo = info;
    STATUS_SUCCESS
}
