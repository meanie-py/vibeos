use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::rtl::*;

use ntkrnl_sys::ddk::{
    _ETHREAD, PERESOURCE, PFILE_OBJECT, PDEVICE_OBJECT, PFAST_IO_DISPATCH,
    PFSRTL_COMMON_FCB_HEADER, PLARGE_INTEGER, PIO_STATUS_BLOCK, PMDL,
    PSHARED_CACHE_MAP, PDRIVER_OBJECT, PFS_FILTER_CALLBACKS,
    PEXTENDED_DRIVER_EXTENSION, FILE_STANDARD_INFORMATION, _IO_STATUS_BLOCK,
    KEVENT, PIRP, FS_FILTER_CALLBACK_DATA, CcCanIWrite,
    CcFastCopyRead, CcCopyRead, CcFastCopyWrite, CcCopyWrite, CcZeroData,
    CcMdlRead, CcMdlReadComplete2, CcPrepareMdlWrite, CcMdlWriteComplete2,
    ExAcquireResourceSharedLite, ExAcquireResourceExclusiveLite, ExReleaseResourceLite,
    ExAcquireSharedWaitForExclusive, ExConvertExclusiveToSharedLite, ExIsResourceAcquiredSharedLite,
    IoGetRelatedDeviceObject, IoGetBaseFileSystemDeviceObject, IoAllocateIrp,
    IoSetThreadHardErrorMode, KeInitializeEvent, KeWaitForSingleObject, KeIsExecutingDpc,
    PsGetCurrentThread,
    ACCESS_MASK, KPROCESSOR_MODE,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};
extern "system" {
    static mut CcFastReadResourceMiss: ULONG;
    static mut CcFastReadNotPossible: ULONG;
    static mut CcFastReadWait: ULONG;
    static mut CcFastReadNoWait: ULONG;
    static mut CcFastMdlReadWait: ULONG;
    static mut CcFastMdlReadNotPossible: ULONG;
}


const FSRTL_FAST_IO_TOP_LEVEL_IRP: ULONG_PTR = 0x04;

extern "system" {
    fn FsRtlIsNtstatusExpected(Status: NTSTATUS) -> BOOLEAN;
}
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn IofCallDriver(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn IofCallDriver(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS;
}
extern "C" {
    fn FsRtlEnterFileSystem();
    fn FsRtlExitFileSystem();
}

#[inline(always)]
fn CcCopyWriteWontFlush(_fo: PFILE_OBJECT, _foff: PLARGE_INTEGER, len: ULONG) -> BOOLEAN {
    if len <= 0x10000 { 1u8 } else { 0u8 }
}

const FastIoIsNotPossible: u8 = 0;
const FastIoIsQuestionable: u8 = 2; // header value; the hand copy said 1
const NotificationEvent: u32 = 0;
const Executive: u32 = 0;
const KernelMode: KPROCESSOR_MODE = 0i8;
const IRP_MJ_QUERY_INFORMATION: u8 = 0x05;
const IRP_SYNCHRONOUS_PAGING_IO: ULONG = 0x00000040;
const IRP_PAGING_IO: ULONG = 0x00000002;
const FileStandardInformation: u32 = 5;
const FO_FILE_FAST_IO_READ: ULONG = 0x00080000;
const FO_WRITE_THROUGH: ULONG = 0x10;
const FO_FILE_MODIFIED: ULONG = 0x1000;
const FO_FILE_SIZE_CHANGED: ULONG = 0x2000;
const SyncTypeOther: u32 = 0;
const SyncTypeCreateSection: u32 = 1;
const FS_FILTER_ACQUIRE_FOR_SECTION_SYNCHRONIZATION: UCHAR = 0xFFu8; // (UCHAR)-1
const FSRTL_FLAG_ACQUIRE_MAIN_RSRC_EX: u8 = 0x8;
const FSRTL_FLAG_ACQUIRE_MAIN_RSRC_SH: u8 = 0x10;

const MAXLONGLONG: i64 = 0x7fffffffffffffff;
const STATUS_CANT_WAIT: NTSTATUS = 0xC00000D8u32 as i32;
const STATUS_FILE_IS_A_DIRECTORY: NTSTATUS = 0xC00000BAu32 as i32;
const STATUS_FSFILTER_OP_COMPLETED_SUCCESSFULLY: NTSTATUS = 0x00000126;
const STATUS_FILE_LOCKED_WITH_ONLY_READERS: NTSTATUS = 0x0000012A;
const STATUS_FILE_LOCKED_WITH_WRITERS: NTSTATUS = 0x0000012B;

macro_rules! ADDRESS_AND_SIZE_TO_SPAN_PAGES {
    ($offset:expr, $size:expr) => {
        ((($offset as usize & 0xfff) + $size as usize + 0xfff) >> 12)
    };
}

macro_rules! FlagOn {
    ($flags:expr, $mask:expr) => {
        (($flags) & ($mask)) != 0
    };
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlIncrementCcFastReadResourceMiss() {
    CcFastReadResourceMiss += 1;
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlIncrementCcFastReadNotPossible() {
    CcFastReadNotPossible += 1;
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlIncrementCcFastReadWait() {
    CcFastReadWait += 1;
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlIncrementCcFastReadNoWait() {
    CcFastReadNoWait += 1;
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlCopyRead(
    FileObject: PFILE_OBJECT,
    FileOffset: PLARGE_INTEGER,
    Length: ULONG,
    Wait: BOOLEAN,
    LockKey: ULONG,
    Buffer: PVOID,
    IoStatus: PIO_STATUS_BLOCK,
    DeviceObject: PDEVICE_OBJECT,
) -> BOOLEAN {
    let FileObject = opaque_ptr(FileObject);
    let FileOffset = opaque_ptr(FileOffset);
    let Buffer = opaque_ptr(Buffer);
    let IoStatus = opaque_ptr(IoStatus);
    let DeviceObject = opaque_ptr(DeviceObject);
    let FcbHeader: PFSRTL_COMMON_FCB_HEADER;
    let mut Offset = LARGE_INTEGER { QuadPart: 0 };
    let FastIoDispatch: PFAST_IO_DISPATCH;
    let Device: PDEVICE_OBJECT;
    let mut Result: BOOLEAN = 1u8;
    let PageCount = ADDRESS_AND_SIZE_TO_SPAN_PAGES!((*FileOffset).QuadPart, Length);

    if Length == 0 {
        (*IoStatus).__bindgen_anon_1.Status = STATUS_SUCCESS;
        (*IoStatus).Information = 0;
        return 1u8;
    }

    if Length as i64 > MAXLONGLONG - (*FileOffset).QuadPart {
        (*IoStatus).__bindgen_anon_1.Status = STATUS_INVALID_PARAMETER;
        (*IoStatus).Information = 0;
        return 0u8;
    }

    Offset.QuadPart = (*FileOffset).QuadPart + Length as i64;
    FcbHeader = (*FileObject).FsContext as PFSRTL_COMMON_FCB_HEADER;

    if Wait != 0 {
        FsRtlEnterFileSystem();
        CcFastReadWait += 1;
        ExAcquireResourceSharedLite((*FcbHeader).Resource, 1u8);
    } else {
        FsRtlEnterFileSystem();
        if ExAcquireResourceSharedLite((*FcbHeader).Resource, 0u8) == 0 {
            FsRtlExitFileSystem();
            FsRtlIncrementCcFastReadResourceMiss();
            return 0u8;
        }
    }

    if (*FileObject).PrivateCacheMap.is_null() || (*FcbHeader).IsFastIoPossible == FastIoIsNotPossible {
        Result = 0u8;
        // goto Cleanup
    } else if (*FcbHeader).IsFastIoPossible == FastIoIsQuestionable {
        Device = IoGetRelatedDeviceObject(FileObject);
        FastIoDispatch = (*(*Device).DriverObject).FastIoDispatch;

        if FastIoDispatch.is_null() || (*FastIoDispatch).FastIoCheckIfPossible.is_none() {
            Result = 0u8;
        } else if let Some(check_fn) = (*FastIoDispatch).FastIoCheckIfPossible {
            if check_fn(FileObject, FileOffset, Length, 1u8, LockKey, 1u8, IoStatus, Device) == 0 {
                Result = 0u8;
            }
        }
    }

    if Result != 0 && Offset.QuadPart > (*FcbHeader).FileSize.QuadPart {
        if (*FileOffset).QuadPart >= (*FcbHeader).FileSize.QuadPart {
            (*IoStatus).__bindgen_anon_1.Status = STATUS_END_OF_FILE;
            (*IoStatus).Information = 0;
        } else {
            let Length = ((*FcbHeader).FileSize.QuadPart - (*FileOffset).QuadPart) as ULONG;
            (*PsGetCurrentThread()).TopLevelIrp = FSRTL_FAST_IO_TOP_LEVEL_IRP as _;

            if Wait != 0 && Offset.s.HighPart == 0 && (*FcbHeader).FileSize.u.HighPart == 0 {
                CcFastCopyRead(FileObject, (*FileOffset).u.LowPart, Length, PageCount as ULONG, Buffer, IoStatus);
                (*FileObject).Flags |= FO_FILE_FAST_IO_READ;
            } else {
                Result = CcCopyRead(FileObject, FileOffset, Length, Wait, Buffer, IoStatus);
                (*FileObject).Flags |= FO_FILE_FAST_IO_READ;
            }

            if Result != 0 {
                (*FileObject).CurrentByteOffset.QuadPart = (*FileOffset).QuadPart + (*IoStatus).Information as i64;
            }

            (*PsGetCurrentThread()).TopLevelIrp = 0;
        }
    } else if Result != 0 {
        (*PsGetCurrentThread()).TopLevelIrp = FSRTL_FAST_IO_TOP_LEVEL_IRP as _;

        if Wait != 0 && Offset.s.HighPart == 0 && (*FcbHeader).FileSize.u.HighPart == 0 {
            CcFastCopyRead(FileObject, (*FileOffset).u.LowPart, Length, PageCount as ULONG, Buffer, IoStatus);
            (*FileObject).Flags |= FO_FILE_FAST_IO_READ;
        } else {
            Result = CcCopyRead(FileObject, FileOffset, Length, Wait, Buffer, IoStatus);
            (*FileObject).Flags |= FO_FILE_FAST_IO_READ;
        }

        if Result != 0 {
            (*FileObject).CurrentByteOffset.QuadPart = (*FileOffset).QuadPart + (*IoStatus).Information as i64;
        }

        (*PsGetCurrentThread()).TopLevelIrp = 0;
    }

    ExReleaseResourceLite((*FcbHeader).Resource);
    FsRtlExitFileSystem();

    if Result == 0 {
        CcFastReadNotPossible += 1;
    }

    Result
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlCopyWrite(
    FileObject: PFILE_OBJECT,
    FileOffset: PLARGE_INTEGER,
    Length: ULONG,
    Wait: BOOLEAN,
    LockKey: ULONG,
    Buffer: PVOID,
    IoStatus: PIO_STATUS_BLOCK,
    DeviceObject: PDEVICE_OBJECT,
) -> BOOLEAN {
    let FileObject = opaque_ptr(FileObject);
    let FileOffset = opaque_ptr(FileOffset);
    let Buffer = opaque_ptr(Buffer);
    let IoStatus = opaque_ptr(IoStatus);
    let DeviceObject = opaque_ptr(DeviceObject);
    let mut Result: BOOLEAN = 1u8;
    let FastIoDispatch: PFAST_IO_DISPATCH;
    let Device: PDEVICE_OBJECT;
    let FcbHeader: PFSRTL_COMMON_FCB_HEADER;
    let SharedCacheMap: PSHARED_CACHE_MAP;

    let FileOffsetAppend = (*FileOffset).u.HighPart == -1i32 && (*FileOffset).u.LowPart == 0xffffffff;

    let mut ResourceAcquiredShared: BOOLEAN = 0u8;
    let mut b_4GB: BOOLEAN;
    let mut FileSizeModified: BOOLEAN = 0u8;
    let mut OldFileSize = LARGE_INTEGER { QuadPart: 0 };
    let mut OldValidDataLength = LARGE_INTEGER { QuadPart: 0 };
    let mut NewSize = LARGE_INTEGER { QuadPart: 0 };
    let mut Offset = LARGE_INTEGER { QuadPart: 0 };

    Offset.QuadPart = (*FileOffset).QuadPart + Length as i64;
    FcbHeader = (*FileObject).FsContext as PFSRTL_COMMON_FCB_HEADER;

    if CcCanIWrite(FileObject, Length, Wait, 0u8) == 0
        || CcCopyWriteWontFlush(FileObject, FileOffset, Length) == 0
        || ((*FileObject).Flags & FO_WRITE_THROUGH) != 0
    {
        return 0u8;
    }

    (*IoStatus).__bindgen_anon_1.Status = STATUS_SUCCESS;
    (*IoStatus).Information = Length as _;

    if Length == 0 {
        return 1u8;
    }

    FsRtlEnterFileSystem();

    if Wait != 0 && (*FcbHeader).AllocationSize.u.HighPart == 0 {
        if !FileOffsetAppend && Offset.s.LowPart <= (*FcbHeader).ValidDataLength.u.LowPart {
            ExAcquireResourceSharedLite((*FcbHeader).Resource, 1u8);
            ResourceAcquiredShared = 1u8;
        } else {
            ExAcquireResourceExclusiveLite((*FcbHeader).Resource, 1u8);
        }

        if FileOffsetAppend {
            Offset.s.LowPart = (*FcbHeader).FileSize.u.LowPart;
            NewSize.s.LowPart = (*FcbHeader).FileSize.u.LowPart + Length;
            b_4GB = (NewSize.s.LowPart < (*FcbHeader).FileSize.u.LowPart) as BOOLEAN;
        } else {
            Offset.s.LowPart = (*FileOffset).u.LowPart;
            NewSize.s.LowPart = (*FileOffset).u.LowPart + Length;
            b_4GB = (NewSize.s.LowPart < (*FileOffset).u.LowPart || (*FileOffset).u.HighPart != 0) as BOOLEAN;
        }

        if (*FileObject).PrivateCacheMap.is_null()
            || (*FcbHeader).IsFastIoPossible == FastIoIsNotPossible
            || (*FcbHeader).AllocationSize.u.LowPart < NewSize.s.LowPart
            || Offset.s.LowPart >= (*FcbHeader).ValidDataLength.u.LowPart + 0x2000
            || b_4GB != 0
        {
            ExReleaseResourceLite((*FcbHeader).Resource);
            FsRtlExitFileSystem();
            return 0u8;
        }

        if ResourceAcquiredShared != 0 && NewSize.s.LowPart > (*FcbHeader).ValidDataLength.u.LowPart + 0x2000 {
            ExReleaseResourceLite((*FcbHeader).Resource);
            ExAcquireResourceExclusiveLite((*FcbHeader).Resource, 1u8);
            if FileOffsetAppend {
                Offset.s.LowPart = (*FcbHeader).FileSize.u.LowPart;
                NewSize.s.LowPart = (*FcbHeader).FileSize.u.LowPart + Length;
                b_4GB = (NewSize.s.LowPart < Offset.s.LowPart) as BOOLEAN;
            }

            if (*FileObject).PrivateCacheMap.is_null()
                || (*FcbHeader).IsFastIoPossible == FastIoIsNotPossible
                || (*FcbHeader).AllocationSize.u.LowPart < NewSize.s.LowPart
                || (*FcbHeader).AllocationSize.u.HighPart != 0
                || b_4GB != 0
            {
                ExReleaseResourceLite((*FcbHeader).Resource);
                FsRtlExitFileSystem();
                return 0u8;
            }
        }

        if (*FcbHeader).IsFastIoPossible == FastIoIsQuestionable {
            Device = IoGetRelatedDeviceObject(FileObject);
            FastIoDispatch = (*(*Device).DriverObject).FastIoDispatch;

            if FastIoDispatch.is_null() || (*FastIoDispatch).FastIoCheckIfPossible.is_none() {
                ExReleaseResourceLite((*FcbHeader).Resource);
                FsRtlExitFileSystem();
                return 0u8;
            }

            let check_offset = if FileOffsetAppend {
                core::ptr::addr_of_mut!((*FcbHeader).FileSize)
            } else {
                FileOffset
            };

            if let Some(check_fn) = (*FastIoDispatch).FastIoCheckIfPossible {
                let mut temp_status = _IO_STATUS_BLOCK {
                    __bindgen_anon_1: core::mem::zeroed(),
                    Information: 0,
                };
                if check_fn(FileObject, check_offset, Length, 1u8, LockKey, 0u8, &mut temp_status, Device) == 0 {
                    ExReleaseResourceLite((*FcbHeader).Resource);
                    FsRtlExitFileSystem();
                    return 0u8;
                }
            }
        }

        if NewSize.s.LowPart > (*FcbHeader).FileSize.u.LowPart {
            FileSizeModified = 1u8;
            OldFileSize.s.LowPart = (*FcbHeader).FileSize.u.LowPart;
            OldValidDataLength.s.LowPart = (*FcbHeader).ValidDataLength.u.LowPart;
            (*FcbHeader).FileSize.u.LowPart = NewSize.s.LowPart;
        }

        (*PsGetCurrentThread()).TopLevelIrp = FSRTL_FAST_IO_TOP_LEVEL_IRP as _;

        if Offset.s.LowPart > (*FcbHeader).ValidDataLength.u.LowPart {
            let mut OffsetVar = LARGE_INTEGER { QuadPart: 0 };
            OffsetVar.s.LowPart = Offset.s.LowPart;
            OffsetVar.s.HighPart = 0;
            CcZeroData(FileObject, core::ptr::addr_of_mut!((*FcbHeader).ValidDataLength) as PLARGE_INTEGER, &mut OffsetVar as *mut LARGE_INTEGER as PLARGE_INTEGER, 1u8);
        }

        CcFastCopyWrite(FileObject, Offset.s.LowPart, Length, Buffer);

        (*PsGetCurrentThread()).TopLevelIrp = 0;

        if NewSize.s.LowPart > (*FcbHeader).ValidDataLength.u.LowPart {
            (*FcbHeader).ValidDataLength.u.LowPart = NewSize.s.LowPart;
        }

        (*FileObject).Flags |= FO_FILE_MODIFIED;

        if FileSizeModified != 0 {
            SharedCacheMap = (*(*FileObject).SectionObjectPointer).SharedCacheMap as PSHARED_CACHE_MAP;
            (*SharedCacheMap).FileSize.u.LowPart = NewSize.s.LowPart;
            (*FileObject).Flags |= FO_FILE_SIZE_CHANGED;
        }

        (*FileObject).CurrentByteOffset.QuadPart = NewSize.s.LowPart as i64;

        ExReleaseResourceLite((*FcbHeader).Resource);
        FsRtlExitFileSystem();
        return Result;
    } else {
        if !FileOffsetAppend && (*FileOffset).QuadPart + Length as i64 <= (*FcbHeader).ValidDataLength.QuadPart {
            if ExAcquireResourceSharedLite((*FcbHeader).Resource, Wait) == 0 {
                FsRtlExitFileSystem();
                return 0u8;
            }
            ResourceAcquiredShared = 1u8;
        } else {
            if ExAcquireResourceExclusiveLite((*FcbHeader).Resource, Wait) == 0 {
                FsRtlExitFileSystem();
                return 0u8;
            }
        }

        if FileOffsetAppend {
            Offset.QuadPart = (*FcbHeader).FileSize.QuadPart;
            NewSize.QuadPart = (*FcbHeader).FileSize.QuadPart + Length as i64;
        } else {
            Offset.QuadPart = (*FileOffset).QuadPart;
            NewSize.QuadPart = (*FileOffset).QuadPart + Length as i64;
        }

        if (*FileObject).PrivateCacheMap.is_null()
            || (*FcbHeader).IsFastIoPossible == FastIoIsNotPossible
            || (*FcbHeader).ValidDataLength.QuadPart + 0x2000 <= Offset.QuadPart
            || Length as i64 > MAXLONGLONG - Offset.QuadPart
            || (*FcbHeader).AllocationSize.QuadPart < NewSize.QuadPart
        {
            ExReleaseResourceLite((*FcbHeader).Resource);
            FsRtlExitFileSystem();
            return 0u8;
        }

        if ResourceAcquiredShared != 0 && NewSize.QuadPart > (*FcbHeader).ValidDataLength.QuadPart {
            ExReleaseResourceLite((*FcbHeader).Resource);
            if ExAcquireResourceExclusiveLite((*FcbHeader).Resource, Wait) == 0 {
                FsRtlExitFileSystem();
                return 0u8;
            }

            if FileOffsetAppend {
                Offset.QuadPart = (*FcbHeader).FileSize.QuadPart;
                NewSize.QuadPart = (*FcbHeader).FileSize.QuadPart + Length as i64;
            }

            if (*FileObject).PrivateCacheMap.is_null()
                || (*FcbHeader).IsFastIoPossible == FastIoIsNotPossible
                || (*FcbHeader).AllocationSize.QuadPart <= NewSize.QuadPart
            {
                ExReleaseResourceLite((*FcbHeader).Resource);
                FsRtlExitFileSystem();
                return 0u8;
            }
        }

        if (*FcbHeader).IsFastIoPossible == FastIoIsQuestionable {
            Device = IoGetRelatedDeviceObject(FileObject);
            FastIoDispatch = (*(*Device).DriverObject).FastIoDispatch;

            if FastIoDispatch.is_null() || (*FastIoDispatch).FastIoCheckIfPossible.is_none() {
                ExReleaseResourceLite((*FcbHeader).Resource);
                FsRtlExitFileSystem();
                return 0u8;
            }

            let check_offset = if FileOffsetAppend {
                core::ptr::addr_of_mut!((*FcbHeader).FileSize)
            } else {
                FileOffset
            };

            if let Some(check_fn) = (*FastIoDispatch).FastIoCheckIfPossible {
                let mut temp_status = _IO_STATUS_BLOCK {
                    __bindgen_anon_1: core::mem::zeroed(),
                    Information: 0,
                };
                if check_fn(FileObject, check_offset, Length, Wait, LockKey, 0u8, &mut temp_status, Device) == 0 {
                    ExReleaseResourceLite((*FcbHeader).Resource);
                    FsRtlExitFileSystem();
                    return 0u8;
                }
            }
        }

        if NewSize.QuadPart > (*FcbHeader).FileSize.QuadPart {
            FileSizeModified = 1u8;
            OldFileSize.QuadPart = (*FcbHeader).FileSize.QuadPart;
            OldValidDataLength.QuadPart = (*FcbHeader).ValidDataLength.QuadPart;

            if NewSize.s.HighPart != (*FcbHeader).FileSize.u.HighPart && !(*FcbHeader).PagingIoResource.is_null() {
                ExAcquireResourceExclusiveLite((*FcbHeader).PagingIoResource, 1u8);
                (*FcbHeader).FileSize.QuadPart = NewSize.QuadPart;
                ExReleaseResourceLite((*FcbHeader).PagingIoResource);
            } else {
                (*FcbHeader).FileSize.QuadPart = NewSize.QuadPart;
            }
        }

        (*PsGetCurrentThread()).TopLevelIrp = FSRTL_FAST_IO_TOP_LEVEL_IRP as _;

        let mut CallCc = 1u8;

        if Offset.QuadPart > (*FcbHeader).ValidDataLength.QuadPart {
            if CcZeroData(FileObject, core::ptr::addr_of_mut!((*FcbHeader).ValidDataLength), &mut Offset as *mut LARGE_INTEGER as PLARGE_INTEGER, Wait) == 0 {
                Result = 0u8;
                CallCc = 0u8;
            }
        }

        if CallCc != 0 {
            Result = CcCopyWrite(FileObject, &mut Offset as *mut LARGE_INTEGER as PLARGE_INTEGER, Length, Wait, Buffer);
        }

        (*PsGetCurrentThread()).TopLevelIrp = 0;

        if Result != 0 {
            if NewSize.QuadPart > (*FcbHeader).ValidDataLength.QuadPart {
                if NewSize.s.HighPart != (*FcbHeader).ValidDataLength.u.HighPart && !(*FcbHeader).PagingIoResource.is_null() {
                    ExAcquireResourceExclusiveLite((*FcbHeader).PagingIoResource, 1u8);
                    (*FcbHeader).ValidDataLength.QuadPart = NewSize.QuadPart;
                    ExReleaseResourceLite((*FcbHeader).PagingIoResource);
                } else {
                    (*FcbHeader).ValidDataLength.QuadPart = NewSize.QuadPart;
                }
            }

            (*FileObject).Flags |= FO_FILE_MODIFIED;

            if FileSizeModified != 0 {
                SharedCacheMap = (*(*FileObject).SectionObjectPointer).SharedCacheMap as PSHARED_CACHE_MAP;
                (*SharedCacheMap).FileSize.QuadPart = NewSize.QuadPart;
                (*FileObject).Flags |= FO_FILE_SIZE_CHANGED;
            }

            (*FileObject).CurrentByteOffset.QuadPart = NewSize.QuadPart;
        } else {
            if FileSizeModified != 0 {
                if !(*FcbHeader).PagingIoResource.is_null() {
                    ExAcquireResourceExclusiveLite((*FcbHeader).PagingIoResource, 1u8);
                    (*FcbHeader).FileSize.QuadPart = OldFileSize.QuadPart;
                    (*FcbHeader).ValidDataLength.QuadPart = OldValidDataLength.QuadPart;
                    ExReleaseResourceLite((*FcbHeader).PagingIoResource);
                } else {
                    (*FcbHeader).FileSize.QuadPart = OldFileSize.QuadPart;
                    (*FcbHeader).ValidDataLength.QuadPart = OldValidDataLength.QuadPart;
                }
            }
        }

        ExReleaseResourceLite((*FcbHeader).Resource);
        FsRtlExitFileSystem();
        return Result;
    }
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlGetFileSize(
    FileObject: PFILE_OBJECT,
    FileSize: PLARGE_INTEGER,
) -> NTSTATUS {
    let FileObject = opaque_ptr(FileObject);
    let FileSize = opaque_ptr(FileSize);
    let mut Info: FILE_STANDARD_INFORMATION = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let mut IoStatus = _IO_STATUS_BLOCK {
        __bindgen_anon_1: core::mem::zeroed(),
        Information: 0,
    };
    let DeviceObject: PDEVICE_OBJECT;
    let FastDispatch: PFAST_IO_DISPATCH;
    let mut Event: KEVENT = core::mem::zeroed();
    let Irp: PIRP;
    let OldHardError: BOOLEAN;

    DeviceObject = IoGetRelatedDeviceObject(FileObject);
    FastDispatch = (*(*DeviceObject).DriverObject).FastIoDispatch;

    if FastDispatch.is_null()
        || (*FastDispatch).FastIoQueryStandardInfo.is_none()
        || {
            if let Some(query_fn) = (*FastDispatch).FastIoQueryStandardInfo {
                query_fn(FileObject, 1u8, &mut Info, &mut IoStatus, DeviceObject) == 0
            } else {
                true
            }
        }
    {
        KeInitializeEvent(&mut Event, NotificationEvent, 0u8);

        Irp = IoAllocateIrp((*DeviceObject).StackSize as i8, 0u8);

        if Irp.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        OldHardError = IoSetThreadHardErrorMode(0u8);

        (*Irp).UserIosb = &mut IoStatus;
        (*Irp).UserEvent = &mut Event;
        (*Irp).Tail.Overlay.Thread = PsGetCurrentThread() as *mut _ETHREAD;
        (*Irp).Flags = IRP_SYNCHRONOUS_PAGING_IO | IRP_PAGING_IO;
        (*Irp).RequestorMode = 0i8;
        (*Irp).Tail.Overlay.OriginalFileObject = FileObject;
        (*Irp).AssociatedIrp.SystemBuffer = &mut Info as *mut _ as PVOID;

        let IoStackLocation = ntkrnl_sys::inline::tier1::IoGetNextIrpStackLocation(Irp);
        (*IoStackLocation).MajorFunction = IRP_MJ_QUERY_INFORMATION;
        (*IoStackLocation).FileObject = FileObject;
        (*IoStackLocation).DeviceObject = DeviceObject;
        (*IoStackLocation).Parameters.QueryFile.Length = core::mem::size_of::<FILE_STANDARD_INFORMATION>() as ULONG;
        (*IoStackLocation).Parameters.QueryFile.FileInformationClass = FileStandardInformation;

        Status = IofCallDriver(DeviceObject, Irp);

        if Status == STATUS_PENDING {
            KeWaitForSingleObject(
                &mut Event as *mut _ as PVOID,
                Executive,
                0i8,
                0u8,
                core::ptr::null_mut(),
            );
        }

        if Status < 0 {
            IoStatus.__bindgen_anon_1.Status = Status;
        }

        IoSetThreadHardErrorMode(OldHardError);
    }

    if IoStatus.__bindgen_anon_1.Status >= 0 {
        if Info.Directory != 0 {
            IoStatus.__bindgen_anon_1.Status = STATUS_FILE_IS_A_DIRECTORY;
        } else {
            (*FileSize).QuadPart = Info.EndOfFile.QuadPart;
        }
    }

    IoStatus.__bindgen_anon_1.Status
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlMdlRead(
    FileObject: PFILE_OBJECT,
    FileOffset: PLARGE_INTEGER,
    Length: ULONG,
    LockKey: ULONG,
    MdlChain: *mut PMDL,
    IoStatus: PIO_STATUS_BLOCK,
) -> BOOLEAN {
    let FileObject = opaque_ptr(FileObject);
    let FileOffset = opaque_ptr(FileOffset);
    let MdlChain = opaque_ptr(MdlChain);
    let IoStatus = opaque_ptr(IoStatus);
    let DeviceObject: PDEVICE_OBJECT;
    let BaseDeviceObject: PDEVICE_OBJECT;
    let FastDispatch: PFAST_IO_DISPATCH;

    DeviceObject = IoGetRelatedDeviceObject(FileObject);
    FastDispatch = (*(*DeviceObject).DriverObject).FastIoDispatch;

    if !FastDispatch.is_null() && (*FastDispatch).MdlRead.is_some() {
        if let Some(mdl_read_fn) = (*FastDispatch).MdlRead {
            return mdl_read_fn(FileObject, FileOffset, Length, LockKey, MdlChain, IoStatus, DeviceObject);
        }
    }

    BaseDeviceObject = IoGetBaseFileSystemDeviceObject(FileObject);
    let FastDispatch = (*(*BaseDeviceObject).DriverObject).FastIoDispatch;

    if !FastDispatch.is_null() && (*FastDispatch).MdlRead.is_some() && BaseDeviceObject != DeviceObject {
        return 0u8;
    }

    FsRtlMdlReadDev(FileObject, FileOffset, Length, LockKey, MdlChain, IoStatus, DeviceObject)
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlMdlReadComplete(
    FileObject: PFILE_OBJECT,
    MdlChain: PMDL,
) -> BOOLEAN {
    let FileObject = opaque_ptr(FileObject);
    let MdlChain = opaque_ptr(MdlChain);
    let DeviceObject: PDEVICE_OBJECT;
    let BaseDeviceObject: PDEVICE_OBJECT;
    let FastDispatch: PFAST_IO_DISPATCH;

    DeviceObject = IoGetRelatedDeviceObject(FileObject);
    FastDispatch = (*(*DeviceObject).DriverObject).FastIoDispatch;

    if !FastDispatch.is_null() && (*FastDispatch).MdlReadComplete.is_some() {
        if let Some(complete_fn) = (*FastDispatch).MdlReadComplete {
            return complete_fn(FileObject, MdlChain, DeviceObject);
        }
    }

    BaseDeviceObject = IoGetBaseFileSystemDeviceObject(FileObject);
    let FastDispatch = (*(*BaseDeviceObject).DriverObject).FastIoDispatch;

    if BaseDeviceObject != DeviceObject && !FastDispatch.is_null() && (*FastDispatch).MdlReadComplete.is_some() {
        return 0u8;
    }

    FsRtlMdlReadCompleteDev(FileObject, MdlChain, DeviceObject)
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlMdlReadCompleteDev(
    FileObject: PFILE_OBJECT,
    MemoryDescriptorList: PMDL,
    DeviceObject: PDEVICE_OBJECT,
) -> BOOLEAN {
    let FileObject = opaque_ptr(FileObject);
    let MemoryDescriptorList = opaque_ptr(MemoryDescriptorList);
    let DeviceObject = opaque_ptr(DeviceObject);
    CcMdlReadComplete2(FileObject, MemoryDescriptorList);
    1u8
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlMdlReadDev(
    FileObject: PFILE_OBJECT,
    FileOffset: PLARGE_INTEGER,
    Length: ULONG,
    LockKey: ULONG,
    MdlChain: *mut PMDL,
    IoStatus: PIO_STATUS_BLOCK,
    DeviceObject: PDEVICE_OBJECT,
) -> BOOLEAN {
    let FileObject = opaque_ptr(FileObject);
    let FileOffset = opaque_ptr(FileOffset);
    let MdlChain = opaque_ptr(MdlChain);
    let IoStatus = opaque_ptr(IoStatus);
    let DeviceObject = opaque_ptr(DeviceObject);
    let FcbHeader: PFSRTL_COMMON_FCB_HEADER;
    let mut Result: BOOLEAN = 1u8;
    let mut Offset = LARGE_INTEGER { QuadPart: 0 };
    let FastIoDispatch: PFAST_IO_DISPATCH;
    let Device: PDEVICE_OBJECT;

    if Length == 0 {
        (*IoStatus).__bindgen_anon_1.Status = STATUS_SUCCESS;
        (*IoStatus).Information = 0;
        return 1u8;
    }

    Offset.QuadPart = (*FileOffset).QuadPart + Length as i64;
    FcbHeader = (*FileObject).FsContext as PFSRTL_COMMON_FCB_HEADER;

    FsRtlEnterFileSystem();
    CcFastMdlReadWait += 1;

    ExAcquireResourceSharedLite((*FcbHeader).Resource, 1u8);

    if (*FileObject).PrivateCacheMap.is_null() || (*FcbHeader).IsFastIoPossible == FastIoIsNotPossible {
        CcFastMdlReadNotPossible += 1;
        Result = 0u8;
    } else if (*FcbHeader).IsFastIoPossible == FastIoIsQuestionable {
        Device = IoGetRelatedDeviceObject(FileObject);
        FastIoDispatch = (*(*Device).DriverObject).FastIoDispatch;

        if FastIoDispatch.is_null() || (*FastIoDispatch).FastIoCheckIfPossible.is_none() { CcFastMdlReadNotPossible += 1;
            Result = 0u8;
        } else if let Some(check_fn) = (*FastIoDispatch).FastIoCheckIfPossible {
            if check_fn(FileObject, FileOffset, Length, 1u8, LockKey, 1u8, IoStatus, Device) == 0 { CcFastMdlReadNotPossible += 1;
                Result = 0u8;
            }
        }
    }

    if Result != 0 {
        if Offset.QuadPart > (*FcbHeader).FileSize.QuadPart {
            if (*FileOffset).QuadPart >= (*FcbHeader).FileSize.QuadPart {
                (*IoStatus).__bindgen_anon_1.Status = STATUS_END_OF_FILE;
                (*IoStatus).Information = 0;
            } else {
                let Length = ((*FcbHeader).FileSize.QuadPart - (*FileOffset).QuadPart) as ULONG;

                (*PsGetCurrentThread()).TopLevelIrp = FSRTL_FAST_IO_TOP_LEVEL_IRP as _;

                CcMdlRead(FileObject, FileOffset, Length, MdlChain, IoStatus);
                (*FileObject).Flags |= FO_FILE_FAST_IO_READ;

                (*PsGetCurrentThread()).TopLevelIrp = 0;
            }
        } else {
            (*PsGetCurrentThread()).TopLevelIrp = FSRTL_FAST_IO_TOP_LEVEL_IRP as _;

            CcMdlRead(FileObject, FileOffset, Length, MdlChain, IoStatus);
            (*FileObject).Flags |= FO_FILE_FAST_IO_READ;

            (*PsGetCurrentThread()).TopLevelIrp = 0;
        }
    }

    ExReleaseResourceLite((*FcbHeader).Resource);
    FsRtlExitFileSystem();

    Result
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlMdlWriteComplete(
    FileObject: PFILE_OBJECT,
    FileOffset: PLARGE_INTEGER,
    MdlChain: PMDL,
) -> BOOLEAN {
    let FileObject = opaque_ptr(FileObject);
    let FileOffset = opaque_ptr(FileOffset);
    let MdlChain = opaque_ptr(MdlChain);
    let DeviceObject: PDEVICE_OBJECT;
    let BaseDeviceObject: PDEVICE_OBJECT;
    let FastDispatch: PFAST_IO_DISPATCH;

    DeviceObject = IoGetRelatedDeviceObject(FileObject);
    FastDispatch = (*(*DeviceObject).DriverObject).FastIoDispatch;

    if !FastDispatch.is_null() && (*FastDispatch).MdlWriteComplete.is_some() {
        if let Some(complete_fn) = (*FastDispatch).MdlWriteComplete {
            return complete_fn(FileObject, FileOffset, MdlChain, DeviceObject);
        }
    }

    BaseDeviceObject = IoGetBaseFileSystemDeviceObject(FileObject);
    let FastDispatch = (*(*BaseDeviceObject).DriverObject).FastIoDispatch;

    if !FastDispatch.is_null() && (*FastDispatch).MdlWriteComplete.is_some() && BaseDeviceObject != DeviceObject {
        return 0u8;
    }

    FsRtlMdlWriteCompleteDev(FileObject, FileOffset, MdlChain, DeviceObject)
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlMdlWriteCompleteDev(
    FileObject: PFILE_OBJECT,
    FileOffset: PLARGE_INTEGER,
    MdlChain: PMDL,
    DeviceObject: PDEVICE_OBJECT,
) -> BOOLEAN {
    let FileObject = opaque_ptr(FileObject);
    let FileOffset = opaque_ptr(FileOffset);
    let MdlChain = opaque_ptr(MdlChain);
    let DeviceObject = opaque_ptr(DeviceObject);
    if ((*FileObject).Flags & FO_WRITE_THROUGH) != 0 {
        return 0u8;
    }

    CcMdlWriteComplete2(FileObject, FileOffset, MdlChain);
    1u8
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlPrepareMdlWrite(
    FileObject: PFILE_OBJECT,
    FileOffset: PLARGE_INTEGER,
    Length: ULONG,
    LockKey: ULONG,
    MdlChain: *mut PMDL,
    IoStatus: PIO_STATUS_BLOCK,
) -> BOOLEAN {
    let FileObject = opaque_ptr(FileObject);
    let FileOffset = opaque_ptr(FileOffset);
    let MdlChain = opaque_ptr(MdlChain);
    let IoStatus = opaque_ptr(IoStatus);
    let DeviceObject: PDEVICE_OBJECT;
    let BaseDeviceObject: PDEVICE_OBJECT;
    let FastDispatch: PFAST_IO_DISPATCH;

    DeviceObject = IoGetRelatedDeviceObject(FileObject);
    FastDispatch = (*(*DeviceObject).DriverObject).FastIoDispatch;

    if !FastDispatch.is_null() && (*FastDispatch).PrepareMdlWrite.is_some() {
        if let Some(prepare_fn) = (*FastDispatch).PrepareMdlWrite {
            return prepare_fn(FileObject, FileOffset, Length, LockKey, MdlChain, IoStatus, DeviceObject);
        }
    }

    BaseDeviceObject = IoGetBaseFileSystemDeviceObject(FileObject);
    let FastDispatch = (*(*BaseDeviceObject).DriverObject).FastIoDispatch;

    if !FastDispatch.is_null() && (*FastDispatch).PrepareMdlWrite.is_some() && BaseDeviceObject != DeviceObject {
        return 0u8;
    }

    FsRtlPrepareMdlWriteDev(FileObject, FileOffset, Length, LockKey, MdlChain, IoStatus, DeviceObject)
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlPrepareMdlWriteDev(
    FileObject: PFILE_OBJECT,
    FileOffset: PLARGE_INTEGER,
    Length: ULONG,
    LockKey: ULONG,
    MdlChain: *mut PMDL,
    IoStatus: PIO_STATUS_BLOCK,
    DeviceObject: PDEVICE_OBJECT,
) -> BOOLEAN {
    let FileObject = opaque_ptr(FileObject);
    let FileOffset = opaque_ptr(FileOffset);
    let MdlChain = opaque_ptr(MdlChain);
    let IoStatus = opaque_ptr(IoStatus);
    let DeviceObject = opaque_ptr(DeviceObject);
    let mut Result: BOOLEAN = 1u8;
    let FastIoDispatch: PFAST_IO_DISPATCH;
    let Device: PDEVICE_OBJECT;
    let FcbHeader: PFSRTL_COMMON_FCB_HEADER;
    let SharedCacheMap: PSHARED_CACHE_MAP;

    let mut OldFileSize = LARGE_INTEGER { QuadPart: 0 };
    let mut OldValidDataLength = LARGE_INTEGER { QuadPart: 0 };
    let mut NewSize = LARGE_INTEGER { QuadPart: 0 };
    let mut Offset = LARGE_INTEGER { QuadPart: 0 };

    let FileOffsetAppend = (*FileOffset).u.HighPart == -1i32 && (*FileOffset).u.LowPart == 0xffffffff;

    let mut FileSizeModified: BOOLEAN = 0u8;
    let mut ResourceAcquiredShared: BOOLEAN = 0u8;

    Offset.QuadPart = (*FileOffset).QuadPart + Length as i64;

    if CcCanIWrite(FileObject, Length, 1u8, 0u8) == 0 || ((*FileObject).Flags & FO_WRITE_THROUGH) != 0 {
        return 0u8;
    }

    (*IoStatus).__bindgen_anon_1.Status = STATUS_SUCCESS;

    if Length == 0 {
        return 1u8;
    }

    FcbHeader = (*FileObject).FsContext as PFSRTL_COMMON_FCB_HEADER;
    FsRtlEnterFileSystem();

    if !FileOffsetAppend && (*FileOffset).QuadPart + Length as i64 <= (*FcbHeader).ValidDataLength.QuadPart {
        ExAcquireResourceSharedLite((*FcbHeader).Resource, 1u8);
        ResourceAcquiredShared = 1u8;
    } else {
        ExAcquireResourceExclusiveLite((*FcbHeader).Resource, 1u8);
    }

    if FileOffsetAppend {
        Offset.QuadPart = (*FcbHeader).FileSize.QuadPart;
        NewSize.QuadPart = (*FcbHeader).FileSize.QuadPart + Length as i64;
    } else {
        Offset.QuadPart = (*FileOffset).QuadPart;
        NewSize.QuadPart = (*FileOffset).QuadPart + Length as i64;
    }

    if (*FileObject).PrivateCacheMap.is_null()
        || (*FcbHeader).IsFastIoPossible == 0
        || Length as i64 > MAXLONGLONG - (*FileOffset).QuadPart
        || NewSize.QuadPart > (*FcbHeader).AllocationSize.QuadPart
    {
        ExReleaseResourceLite((*FcbHeader).Resource);
        FsRtlExitFileSystem();
        return 0u8;
    }

    if ResourceAcquiredShared != 0 && NewSize.QuadPart > (*FcbHeader).ValidDataLength.QuadPart {
        ExReleaseResourceLite((*FcbHeader).Resource);
        ExAcquireResourceExclusiveLite((*FcbHeader).Resource, 1u8);

        if FileOffsetAppend {
            Offset.QuadPart = (*FcbHeader).FileSize.QuadPart;
            NewSize.QuadPart = (*FcbHeader).FileSize.QuadPart + Length as i64;
        }

        if (*FileObject).PrivateCacheMap.is_null()
            || (*FcbHeader).IsFastIoPossible == FastIoIsNotPossible
            || (*FcbHeader).AllocationSize.QuadPart <= NewSize.QuadPart
        {
            ExReleaseResourceLite((*FcbHeader).Resource);
            FsRtlExitFileSystem();
            return 0u8;
        }
    }

    if (*FcbHeader).IsFastIoPossible == FastIoIsQuestionable {
        Device = IoGetRelatedDeviceObject(FileObject);
        FastIoDispatch = (*(*Device).DriverObject).FastIoDispatch;

        if FastIoDispatch.is_null() || (*FastIoDispatch).FastIoCheckIfPossible.is_none() {
            ExReleaseResourceLite((*FcbHeader).Resource);
            FsRtlExitFileSystem();
            return 0u8;
        }

        if let Some(check_fn) = (*FastIoDispatch).FastIoCheckIfPossible {
            if check_fn(FileObject, FileOffset, Length, 1u8, LockKey, 0u8, IoStatus, Device) == 0 {
                ExReleaseResourceLite((*FcbHeader).Resource);
                FsRtlExitFileSystem();
                return 0u8;
            }
        }
    }

    if NewSize.QuadPart > (*FcbHeader).FileSize.QuadPart {
        FileSizeModified = 1u8;
        OldFileSize.QuadPart = (*FcbHeader).FileSize.QuadPart;
        OldValidDataLength.QuadPart = (*FcbHeader).ValidDataLength.QuadPart;

        if NewSize.s.HighPart != (*FcbHeader).FileSize.u.HighPart && !(*FcbHeader).PagingIoResource.is_null() {
            ExAcquireResourceExclusiveLite((*FcbHeader).PagingIoResource, 1u8);
            (*FcbHeader).FileSize.QuadPart = NewSize.QuadPart;
            ExReleaseResourceLite((*FcbHeader).PagingIoResource);
        } else {
            (*FcbHeader).FileSize.QuadPart = NewSize.QuadPart;
        }
    }

    (*PsGetCurrentThread()).TopLevelIrp = FSRTL_FAST_IO_TOP_LEVEL_IRP as _;

    if Offset.QuadPart > (*FcbHeader).ValidDataLength.QuadPart {
        Result = CcZeroData(FileObject, core::ptr::addr_of_mut!((*FcbHeader).ValidDataLength) as PLARGE_INTEGER, &mut Offset as *mut LARGE_INTEGER as PLARGE_INTEGER, 1u8);
        if Result != 0 {
            CcPrepareMdlWrite(FileObject, &mut Offset as *mut LARGE_INTEGER as PLARGE_INTEGER, Length, MdlChain, IoStatus);
        }
    } else {
        CcPrepareMdlWrite(FileObject, &mut Offset as *mut LARGE_INTEGER as PLARGE_INTEGER, Length, MdlChain, IoStatus);
    }

    (*PsGetCurrentThread()).TopLevelIrp = 0;

    if Result != 0 {
        if NewSize.QuadPart > (*FcbHeader).ValidDataLength.QuadPart {
            if NewSize.s.HighPart != (*FcbHeader).ValidDataLength.u.HighPart && !(*FcbHeader).PagingIoResource.is_null() {
                ExAcquireResourceExclusiveLite((*FcbHeader).PagingIoResource, 1u8);
                (*FcbHeader).ValidDataLength.QuadPart = NewSize.QuadPart;
                ExReleaseResourceLite((*FcbHeader).PagingIoResource);
            } else {
                (*FcbHeader).ValidDataLength.QuadPart = NewSize.QuadPart;
            }
        }

        (*FileObject).Flags |= FO_FILE_MODIFIED;

        if FileSizeModified != 0 {
            SharedCacheMap = (*(*FileObject).SectionObjectPointer).SharedCacheMap as PSHARED_CACHE_MAP;
            (*SharedCacheMap).FileSize.QuadPart = NewSize.QuadPart;
            (*FileObject).Flags |= FO_FILE_SIZE_CHANGED;
        }
    } else {
        if FileSizeModified != 0 {
            if !(*FcbHeader).PagingIoResource.is_null() {
                ExAcquireResourceExclusiveLite((*FcbHeader).PagingIoResource, 1u8);
                (*FcbHeader).FileSize.QuadPart = OldFileSize.QuadPart;
                (*FcbHeader).ValidDataLength.QuadPart = OldValidDataLength.QuadPart;
                ExReleaseResourceLite((*FcbHeader).PagingIoResource);
            } else {
                (*FcbHeader).FileSize.QuadPart = OldFileSize.QuadPart;
                (*FcbHeader).ValidDataLength.QuadPart = OldValidDataLength.QuadPart;
            }
        }
    }

    ExReleaseResourceLite((*FcbHeader).Resource);
    FsRtlExitFileSystem();
    Result
}

unsafe fn FsRtlAcquireFileExclusiveCommon(
    FileObject: PFILE_OBJECT,
    SyncType: u32,
    Reserved: ULONG,
) -> NTSTATUS {
    let FcbHeader: PFSRTL_COMMON_FCB_HEADER;
    let DeviceObject: PDEVICE_OBJECT;
    let FastDispatch: PFAST_IO_DISPATCH;
    let DriverExtension: PEXTENDED_DRIVER_EXTENSION;
    let FilterCallbacks: PFS_FILTER_CALLBACKS;

    FcbHeader = (*FileObject).FsContext as PFSRTL_COMMON_FCB_HEADER;
    DeviceObject = IoGetRelatedDeviceObject(FileObject);

    FsRtlEnterFileSystem();

    DriverExtension = (*(*DeviceObject).DriverObject).DriverExtension as PEXTENDED_DRIVER_EXTENSION;
    FilterCallbacks = (*DriverExtension).FsFilterCallbacks;

    if !FilterCallbacks.is_null() && (*FilterCallbacks).PreAcquireForSectionSynchronization.is_some() {
        let mut Status: NTSTATUS;
        let mut CompletionContext: PVOID = core::ptr::null_mut();

        let mut CbData: FS_FILTER_CALLBACK_DATA = core::mem::zeroed();

        CbData.SizeOfFsFilterCallbackData = core::mem::size_of::<FS_FILTER_CALLBACK_DATA>() as ULONG;
        CbData.Operation = FS_FILTER_ACQUIRE_FOR_SECTION_SYNCHRONIZATION;
        CbData.Reserved = 0;
        CbData.DeviceObject = DeviceObject;
        CbData.FileObject = FileObject;
        CbData.Parameters.AcquireForSectionSynchronization.SyncType = SyncType;
        CbData.Parameters.AcquireForSectionSynchronization.PageProtection = Reserved;

        if let Some(pre_fn) = (*FilterCallbacks).PreAcquireForSectionSynchronization {
            Status = pre_fn(&mut CbData, &mut CompletionContext);
            if Status < 0 {
                FsRtlExitFileSystem();
                return Status;
            }

            if let Some(post_fn) = (*FilterCallbacks).PostAcquireForSectionSynchronization {
                post_fn(&mut CbData, Status, CompletionContext);
            }

            if Status == STATUS_FSFILTER_OP_COMPLETED_SUCCESSFULLY
                || Status == STATUS_FILE_LOCKED_WITH_ONLY_READERS
                || Status == STATUS_FILE_LOCKED_WITH_WRITERS
            {
                return Status;
            }
        }
    }

    FastDispatch = (*(*DeviceObject).DriverObject).FastIoDispatch;

    if !FastDispatch.is_null() && (*FastDispatch).AcquireFileForNtCreateSection.is_some() {
        if let Some(acquire_fn) = (*FastDispatch).AcquireFileForNtCreateSection {
            acquire_fn(FileObject);
        }
    } else {
        if !FcbHeader.is_null() && !(*FcbHeader).Resource.is_null() {
            ExAcquireResourceExclusiveLite((*FcbHeader).Resource, 1u8);
        }
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlAcquireFileExclusive(FileObject: PFILE_OBJECT) {
    let FileObject = opaque_ptr(FileObject);
    FsRtlAcquireFileExclusiveCommon(FileObject, SyncTypeOther, 0);
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlAcquireToCreateMappedSection(
    FileObject: PFILE_OBJECT,
    SectionPageProtection: ULONG,
) -> NTSTATUS {
    let FileObject = opaque_ptr(FileObject);
    FsRtlAcquireFileExclusiveCommon(FileObject, SyncTypeCreateSection, SectionPageProtection)
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlReleaseFile(FileObject: PFILE_OBJECT) {
    let FileObject = opaque_ptr(FileObject);
    let FcbHeader: PFSRTL_COMMON_FCB_HEADER;
    let DeviceObject: PDEVICE_OBJECT;
    let FastDispatch: PFAST_IO_DISPATCH;

    FcbHeader = (*FileObject).FsContext as PFSRTL_COMMON_FCB_HEADER;
    DeviceObject = IoGetRelatedDeviceObject(FileObject);
    FastDispatch = (*(*DeviceObject).DriverObject).FastIoDispatch;

    if !FastDispatch.is_null() && (*FastDispatch).ReleaseFileForNtCreateSection.is_some() {
        if let Some(release_fn) = (*FastDispatch).ReleaseFileForNtCreateSection {
            release_fn(FileObject);
        }
    } else {
        if !FcbHeader.is_null() && !(*FcbHeader).Resource.is_null() {
            ExReleaseResourceLite((*FcbHeader).Resource);
        }
    }

    FsRtlExitFileSystem();
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlAcquireFileForCcFlushEx(FileObject: PFILE_OBJECT) -> NTSTATUS {
    let FileObject = opaque_ptr(FileObject);
    let FcbHeader: PFSRTL_COMMON_FCB_HEADER;
    let DeviceObject: PDEVICE_OBJECT;
    let BaseDeviceObject: PDEVICE_OBJECT;
    let FastDispatch: PFAST_IO_DISPATCH;
    let mut Status: NTSTATUS;

    FcbHeader = (*FileObject).FsContext as PFSRTL_COMMON_FCB_HEADER;
    DeviceObject = IoGetRelatedDeviceObject(FileObject);
    BaseDeviceObject = IoGetBaseFileSystemDeviceObject(FileObject);
    FastDispatch = (*(*DeviceObject).DriverObject).FastIoDispatch;

    FsRtlEnterFileSystem();

    if !FastDispatch.is_null() && (*FastDispatch).AcquireForCcFlush.is_some() {
        if let Some(acquire_fn) = (*FastDispatch).AcquireForCcFlush {
            Status = acquire_fn(FileObject, BaseDeviceObject);

            if Status >= 0 {
                return Status;
            }

            if Status == STATUS_CANT_WAIT {
                FsRtlExitFileSystem();
                return Status;
            }
        }
    }

    if !(*FcbHeader).Resource.is_null() {
        if ExIsResourceAcquiredSharedLite((*FcbHeader).Resource) != 0 {
            ExAcquireResourceSharedLite((*FcbHeader).Resource, 1u8);
        } else {
            ExAcquireResourceExclusiveLite((*FcbHeader).Resource, 1u8);
        }
    }

    if !(*FcbHeader).PagingIoResource.is_null() {
        ExAcquireResourceSharedLite((*FcbHeader).PagingIoResource, 1u8);
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlAcquireFileForCcFlush(FileObject: PFILE_OBJECT) {
    let FileObject = opaque_ptr(FileObject);
    FsRtlAcquireFileForCcFlushEx(FileObject);
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlReleaseFileForCcFlush(FileObject: PFILE_OBJECT) {
    let FileObject = opaque_ptr(FileObject);
    let FcbHeader: PFSRTL_COMMON_FCB_HEADER;
    let DeviceObject: PDEVICE_OBJECT;
    let BaseDeviceObject: PDEVICE_OBJECT;
    let FastDispatch: PFAST_IO_DISPATCH;
    let mut Status: NTSTATUS = STATUS_INVALID_DEVICE_REQUEST;

    FcbHeader = (*FileObject).FsContext as PFSRTL_COMMON_FCB_HEADER;
    DeviceObject = IoGetRelatedDeviceObject(FileObject);
    BaseDeviceObject = IoGetBaseFileSystemDeviceObject(FileObject);
    FastDispatch = (*(*DeviceObject).DriverObject).FastIoDispatch;

    if !FastDispatch.is_null() && (*FastDispatch).ReleaseForCcFlush.is_some() {
        if let Some(release_fn) = (*FastDispatch).ReleaseForCcFlush {
            Status = release_fn(FileObject, BaseDeviceObject);
        }
    }

    if Status < 0 {
        if !(*FcbHeader).PagingIoResource.is_null() {
            ExReleaseResourceLite((*FcbHeader).PagingIoResource);
        }
        if !(*FcbHeader).Resource.is_null() {
            ExReleaseResourceLite((*FcbHeader).Resource);
        }
    }

    FsRtlExitFileSystem();
}

unsafe fn FsRtlpGetResourceForModWrite(
    FcbHeader: PFSRTL_COMMON_FCB_HEADER,
    EndingOffset: PLARGE_INTEGER,
    ResourceToAcquire: *mut PERESOURCE,
) -> BOOLEAN {
    if FlagOn!((*FcbHeader).Flags, FSRTL_FLAG_ACQUIRE_MAIN_RSRC_EX) {
        *ResourceToAcquire = (*FcbHeader).Resource;
        return 1u8;
    }

    if FlagOn!((*FcbHeader).Flags, FSRTL_FLAG_ACQUIRE_MAIN_RSRC_SH) {
        *ResourceToAcquire = (*FcbHeader).Resource;
    } else {
        *ResourceToAcquire = (*FcbHeader).PagingIoResource;
    }

    if (*EndingOffset).QuadPart > (*FcbHeader).ValidDataLength.QuadPart {
        1u8
    } else {
        0u8
    }
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlAcquireFileForModWriteEx(
    FileObject: PFILE_OBJECT,
    EndingOffset: PLARGE_INTEGER,
    ResourceToRelease: *mut PERESOURCE,
) -> NTSTATUS {
    let FileObject = opaque_ptr(FileObject);
    let EndingOffset = opaque_ptr(EndingOffset);
    let ResourceToRelease = opaque_ptr(ResourceToRelease);
    let FcbHeader: PFSRTL_COMMON_FCB_HEADER;
    let DeviceObject: PDEVICE_OBJECT;
    let BaseDeviceObject: PDEVICE_OBJECT;
    let FastDispatch: PFAST_IO_DISPATCH;
    let mut ResourceToAcquire: PERESOURCE = core::ptr::null_mut();
    let mut Exclusive: BOOLEAN;
    let mut Result: BOOLEAN;
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    FcbHeader = (*FileObject).FsContext as PFSRTL_COMMON_FCB_HEADER;
    DeviceObject = IoGetRelatedDeviceObject(FileObject);
    BaseDeviceObject = IoGetBaseFileSystemDeviceObject(FileObject);
    FastDispatch = (*(*DeviceObject).DriverObject).FastIoDispatch;

    if !FastDispatch.is_null() && (*FastDispatch).AcquireForModWrite.is_some() {
        if let Some(acquire_fn) = (*FastDispatch).AcquireForModWrite {
            Status = acquire_fn(FileObject, EndingOffset, ResourceToRelease, BaseDeviceObject);
            if Status == STATUS_CANT_WAIT {
                static REPORTED: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);
                if REPORTED.fetch_add(1, core::sync::atomic::Ordering::Relaxed) < 8 {
                    ntkrnl_sys::driver_ddk::DbgPrint(
                        c"[FSRTL] mod-write acquire CANT_WAIT from the FS's AcquireForModWrite: file=%wZ\n".as_ptr(),
                        core::ptr::addr_of!((*FileObject).FileName),
                    );
                }
            }

            if Status == STATUS_SUCCESS || Status == STATUS_CANT_WAIT {
                return Status;
            }
        }
    }

    Exclusive = FsRtlpGetResourceForModWrite(FcbHeader, EndingOffset, &mut ResourceToAcquire);

    loop {
        let OldExclusive: BOOLEAN;
        let OldResourceToAcquire: PERESOURCE;

        if ResourceToAcquire.is_null() {
            break;
        }

        if Exclusive != 0 {
            Result = ExAcquireResourceExclusiveLite(ResourceToAcquire, 0u8);
        } else {
            Result = ExAcquireSharedWaitForExclusive(ResourceToAcquire, 0u8);
        }

        if Result == 0 {
            // Capped: the modified-page writer retries, and MiShutdownSystem loops "while
            // dirty", so a resource that never frees spins silently without this line.
            static REPORTED: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);
            if REPORTED.fetch_add(1, core::sync::atomic::Ordering::Relaxed) < 8 {
                ntkrnl_sys::driver_ddk::DbgPrint(
                    c"[FSRTL] mod-write acquire CANT_WAIT: file=%wZ resource=%p excl=%u owner=%p active=%d flag=%x\n".as_ptr(),
                    core::ptr::addr_of!((*FileObject).FileName),
                    ResourceToAcquire,
                    Exclusive as u32,
                    (*ResourceToAcquire).OwnerEntry.OwnerThread as *const u8,
                    (*ResourceToAcquire).ActiveCount as i32,
                    (*ResourceToAcquire).Flag as u32,
                );
            }
            return STATUS_CANT_WAIT;
        }

        OldExclusive = Exclusive;
        OldResourceToAcquire = ResourceToAcquire;
        Exclusive = FsRtlpGetResourceForModWrite(FcbHeader, EndingOffset, &mut ResourceToAcquire);

        if OldExclusive == Exclusive && OldResourceToAcquire == ResourceToAcquire {
            break;
        }

        if OldResourceToAcquire == ResourceToAcquire && Exclusive == 0 {
            ExConvertExclusiveToSharedLite(ResourceToAcquire);
            break;
        }

        ExReleaseResourceLite(OldResourceToAcquire);
    }

    *ResourceToRelease = ResourceToAcquire;
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlReleaseFileForModWrite(
    FileObject: PFILE_OBJECT,
    ResourceToRelease: PERESOURCE,
) {
    let FileObject = opaque_ptr(FileObject);
    let DeviceObject: PDEVICE_OBJECT;
    let BaseDeviceObject: PDEVICE_OBJECT;
    let FastDispatch: PFAST_IO_DISPATCH;
    let mut Status: NTSTATUS = STATUS_UNSUCCESSFUL;

    DeviceObject = IoGetRelatedDeviceObject(FileObject);
    BaseDeviceObject = IoGetBaseFileSystemDeviceObject(FileObject);
    FastDispatch = (*(*DeviceObject).DriverObject).FastIoDispatch;

    if !FastDispatch.is_null() && (*FastDispatch).ReleaseForModWrite.is_some() {
        if let Some(release_fn) = (*FastDispatch).ReleaseForModWrite {
            Status = release_fn(FileObject, ResourceToRelease, BaseDeviceObject);
        }
    }

    if Status < 0 {
        ExReleaseResourceLite(ResourceToRelease);
    }
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlRegisterFileSystemFilterCallbacks(
    FilterDriverObject: PDRIVER_OBJECT,
    Callbacks: PFS_FILTER_CALLBACKS,
) -> NTSTATUS {
    let FilterDriverObject = opaque_ptr(FilterDriverObject);
    let NewCallbacks: PFS_FILTER_CALLBACKS;
    let DriverExtension: PEXTENDED_DRIVER_EXTENSION;

    if FilterDriverObject.is_null() || Callbacks.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    NewCallbacks = ExAllocatePoolWithTag(
        NonPagedPool,
        (*Callbacks).SizeOfFsFilterCallbacks as usize,
        u32::from_ne_bytes(*b"gmSF"),
    ) as PFS_FILTER_CALLBACKS;

    if NewCallbacks.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    core::ptr::copy_nonoverlapping(
        Callbacks as *const u8,
        NewCallbacks as *mut u8,
        (*Callbacks).SizeOfFsFilterCallbacks as usize,
    );

    DriverExtension = (*FilterDriverObject).DriverExtension as PEXTENDED_DRIVER_EXTENSION;
    (*DriverExtension).FsFilterCallbacks = NewCallbacks;

    STATUS_SUCCESS
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/fsrtl/fastio.c
//  * PURPOSE:         Provides Fast I/O entrypoints to the Cache Manager
//  * PROGRAMMERS:     Dominique Cote (buzdelabuz2@gmail.com)
//  *                  Alex Ionescu (alex.ionescu@reactos.org)
//  *                  Aleksey Bragin (aleksey@reactos.org)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* PUBLIC FUNCTIONS **********************************************************/
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// FsRtlIncrement,(VOID)
// {
//,++;
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// FsRtlIncrement,(VOID)
// {
//,++;
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// FsRtlIncrement,(VOID)
// {
//,++;
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// FsRtlIncrement,(VOID)
// {
//,++;
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// NTAPI
// FsRtlCopyRead(IN PFILE_OBJECT FileObject,
//               IN PLARGE_INTEGER FileOffset,
//               IN ULONG Length,
//               IN BOOLEAN Wait,
//               IN ULONG LockKey,
//               OUT PVOID Buffer,
//               OUT PIO_STATUS_BLOCK IoStatus,
//               IN PDEVICE_OBJECT DeviceObject)
// {
//
//     PFSRTL_COMMON_FCB_HEADER FcbHeader;
//     LARGE_INTEGER Offset;
//     PFAST_IO_DISPATCH FastIoDispatch;
//     PDEVICE_OBJECT Device;
//     BOOLEAN Result = TRUE;
//     ULONG PageCount = ADDRESS_AND_SIZE_TO_SPAN_PAGES(FileOffset, Length);
//
//     PAGED_CODE();
//     ASSERT(FileObject);
//     ASSERT(FileObject->FsContext);
//
//     /* No actual read */
//     if (!Length)
//     {
//         /* Return success */
//         IoStatus->Status = STATUS_SUCCESS;
//         IoStatus->Information = 0;
//         return TRUE;
//     }
//
//     if (Length > MAXLONGLONG - FileOffset->QuadPart)
//     {
//         IoStatus->Status = STATUS_INVALID_PARAMETER;
//         IoStatus->Information = 0;
//         return FALSE;
//     }
//
//     /* Get the offset and FCB header */
//     Offset.QuadPart = FileOffset->QuadPart + Length;
//     FcbHeader = (PFSRTL_COMMON_FCB_HEADER)FileObject->FsContext;
//
//     if (Wait)
//     {
//         /* Use a Resource Acquire */
//         FsRtlEnterFileSystem();
//,++;
//         ExAcquireResourceSharedLite(FcbHeader->Resource, TRUE);
//     }
//     else
//     {
//         /* Acquire the resource without blocking. Return false and the I/O manager
//          * will retry using the standard IRP method. Use a Resource Acquire.
//          */
//         FsRtlEnterFileSystem();
//         if (!ExAcquireResourceSharedLite(FcbHeader->Resource, FALSE))
//         {
//             FsRtlExitFileSystem();
//             FsRtlIncrement,();
//             return FALSE;
//         }
//     }
//
//     /* Check if this is a fast I/O cached file */
//     if (!(FileObject->PrivateCacheMap) ||
//         (FcbHeader->IsFastIoPossible == FastIoIsNotPossible))
//     {
//         /* It's not, so fail */
//         Result = FALSE;
//         goto Cleanup;
//     }
//
//     /* Check if we need to find out if fast I/O is available */
//     if (FcbHeader->IsFastIoPossible == FastIoIsQuestionable)
//     {
//         /* Sanity check */
//         ASSERT(!KeIsExecutingDpc());
//
//         /* Get the Fast I/O table */
//         Device = IoGetRelatedDeviceObject(FileObject);
//         FastIoDispatch = Device->DriverObject->FastIoDispatch;
//
//         /* Sanity check */
//         ASSERT(FastIoDispatch != NULL);
//         ASSERT(FastIoDispatch->FastIoCheckIfPossible != NULL);
//
//         /* Ask the driver if we can do it */
//         if (!FastIoDispatch->FastIoCheckIfPossible(FileObject,
//                                                    FileOffset,
//                                                    Length,
//                                                    TRUE,
//                                                    LockKey,
//                                                    TRUE,
//                                                    IoStatus,
//                                                    Device))
//         {
//             /* It's not, fail */
//             Result = FALSE;
//             goto Cleanup;
//         }
//     }
//
//     /* Check if we read too much */
//     if (Offset.QuadPart > FcbHeader->FileSize.QuadPart)
//     {
//         /* We did, check if the file offset is past the end */
//         if (FileOffset->QuadPart >= FcbHeader->FileSize.QuadPart)
//         {
//             /* Set end of file */
//             IoStatus->Status = STATUS_END_OF_FILE;
//             IoStatus->Information = 0;
//             goto Cleanup;
//         }
//
//         /* Otherwise, just normalize the length */
//         Length = (ULONG)(FcbHeader->FileSize.QuadPart - FileOffset->QuadPart);
//     }
//
//     /* Set this as top-level IRP */
//     PsGetCurrentThread()->TopLevelIrp = FSRTL_FAST_IO_TOP_LEVEL_IRP;
//
//     _SEH2_TRY
//     {
//         /* Make sure the IO and file size is below 4GB */
//         if (Wait && !(Offset.HighPart | FcbHeader->FileSize.HighPart))
//         {
//
//             /* Call the cache controller */
//             CcFastCopyRead(FileObject,
//                            FileOffset->LowPart,
//                            Length,
//                            PageCount,
//                            Buffer,
//                            IoStatus);
//
//             /* File was accessed */
//             FileObject->Flags |= FO_FILE_FAST_IO_READ;
//
//             if (IoStatus->Status != STATUS_END_OF_FILE)
//             {
//                 ASSERT((ULONGLONG)FcbHeader->FileSize.QuadPart >=
//                       ((ULONGLONG)FileOffset->QuadPart + IoStatus->Information));
//             }
//         }
//         else
//         {
//
//             /* Call the cache controller */
//             Result = CcCopyRead(FileObject,
//                                 FileOffset,
//                                 Length,
//                                 Wait,
//                                 Buffer,
//                                 IoStatus);
//
//             /* File was accessed */
//             FileObject->Flags |= FO_FILE_FAST_IO_READ;
//
//             if (Result != FALSE)
//             {
//                 ASSERT((IoStatus->Status == STATUS_END_OF_FILE) ||
//                        (((ULONGLONG)FileOffset->QuadPart + IoStatus->Information) <=
//                         (ULONGLONG)FcbHeader->FileSize.QuadPart));
//             }
//         }
//
//         /* Update the current file offset */
//         if (Result != FALSE)
//         {
//             FileObject->CurrentByteOffset.QuadPart = FileOffset->QuadPart + IoStatus->Information;
//         }
//     }
//     _SEH2_EXCEPT(FsRtlIsNtstatusExpected(_SEH2_GetExceptionCode()) ?
//                  EXCEPTION_EXECUTE_HANDLER : EXCEPTION_CONTINUE_SEARCH)
//     {
//         Result = FALSE;
//     }
//     _SEH2_END;
//
//     PsGetCurrentThread()->TopLevelIrp = 0;
//
//     /* Return to caller */
// Cleanup:
//
//     ExReleaseResourceLite(FcbHeader->Resource);
//     FsRtlExitFileSystem();
//
//     if (Result == FALSE)
//     {
//,+= 1;
//     }
//
//     return Result;
// }
//
//
// /*
//  * @implemented
//  */
// BOOLEAN
// NTAPI
// FsRtlCopyWrite(IN PFILE_OBJECT FileObject,
//                IN PLARGE_INTEGER FileOffset,
//                IN ULONG Length,
//                IN BOOLEAN Wait,
//                IN ULONG LockKey,
//                OUT PVOID Buffer,
//                OUT PIO_STATUS_BLOCK IoStatus,
//                IN PDEVICE_OBJECT DeviceObject)
// {
//     BOOLEAN Result = TRUE;
//     PFAST_IO_DISPATCH FastIoDispatch;
//     PDEVICE_OBJECT Device;
//     PFSRTL_COMMON_FCB_HEADER FcbHeader;
//     PSHARED_CACHE_MAP SharedCacheMap;
//
//     /* WDK doc.
//      * Offset == 0xffffffffffffffff indicates append to the end of file.
//      */
//     BOOLEAN FileOffsetAppend = (FileOffset->HighPart == (LONG)0xffffffff) &&
//                                (FileOffset->LowPart == 0xffffffff);
//
//     BOOLEAN ResourceAcquiredShared = FALSE;
//     BOOLEAN b_4GB = FALSE;
//     BOOLEAN FileSizeModified = FALSE;
//     LARGE_INTEGER OldFileSize;
//     LARGE_INTEGER OldValidDataLength;
//     LARGE_INTEGER NewSize;
//     LARGE_INTEGER Offset;
//
//     PAGED_CODE();
//
//     ASSERT(FileObject);
//     ASSERT(FileObject->FsContext);
//
//     /* Initialize some of the vars and pointers */
//     NewSize.QuadPart = 0;
//     Offset.QuadPart = FileOffset->QuadPart + Length;
//     FcbHeader = (PFSRTL_COMMON_FCB_HEADER)FileObject->FsContext;
//
//     /* Nagar p.544.
//      * Check with Cc if we can write and check if the IO > 64kB (WDK macro).
//      */
//     if ((CcCanIWrite(FileObject, Length, Wait, FALSE) == FALSE) ||
//         (,(FileObject, FileOffset, Length) == FALSE) ||
//         ((FileObject->Flags & FO_WRITE_THROUGH)))
//     {
//         return FALSE;
//     }
//
//     /* Already init _IO_STATUS_BLOCK */
//     IoStatus->Status = STATUS_SUCCESS;
//     IoStatus->Information = Length;
//
//     /* No actual read */
//     if (!Length)
//     {
//         return TRUE;
//     }
//
//     FsRtlEnterFileSystem();
//
//     /* Nagar p.544/545.
//      * The CcFastCopyWrite doesn't deal with filesize beyond 4GB.
//      */
//     if (Wait && (FcbHeader->AllocationSize.HighPart == 0))
//     {
//         /* If the file offset is not past the file size,
//          * then we can acquire the lock shared.
//          */
//         if ((FileOffsetAppend == FALSE) &&
//             (Offset.LowPart <= FcbHeader->ValidDataLength.LowPart))
//         {
//             ExAcquireResourceSharedLite(FcbHeader->Resource, TRUE);
//             ResourceAcquiredShared = TRUE;
//         }
//         else
//         {
//             ExAcquireResourceExclusiveLite(FcbHeader->Resource, TRUE);
//         }
//
//         /* Nagar p.544/545.
//          * If we append, use the file size as offset.
//          * Also, check that we aren't crossing the 4GB boundary.
//          */
//         if (FileOffsetAppend != FALSE)
//         {
//             Offset.LowPart = FcbHeader->FileSize.LowPart;
//             NewSize.LowPart = FcbHeader->FileSize.LowPart + Length;
//             b_4GB = (NewSize.LowPart < FcbHeader->FileSize.LowPart);
//
//         }
//         else
//         {
//             Offset.LowPart = FileOffset->LowPart;
//             NewSize.LowPart = FileOffset->LowPart + Length;
//             b_4GB = (NewSize.LowPart < FileOffset->LowPart) ||
//                     (FileOffset->HighPart != 0);
//         }
//
//         /* Nagar p.544/545.
//          * Make sure that caching is initated.
//          * That fast are allowed for this file stream.
//          * That we are not extending past the allocated size.
//          * That we are not creating a hole bigger than 8k.
//          * That we are not crossing the 4GB boundary.
//          */
//         if ((FileObject->PrivateCacheMap != NULL) &&
//             (FcbHeader->IsFastIoPossible != FastIoIsNotPossible) &&
//             (FcbHeader->AllocationSize.LowPart >= NewSize.LowPart) &&
//             (Offset.LowPart < FcbHeader->ValidDataLength.LowPart + 0x2000) &&
//             !b_4GB)
//         {
//             /* If we are extending past the file size, we need to
//              * release the lock and acquire it exclusively, because
//              * we are going to need to update the FcbHeader.
//              */
//             if (ResourceAcquiredShared &&
//                 (NewSize.LowPart > FcbHeader->ValidDataLength.LowPart + 0x2000))
//             {
//                 /* Then we need to acquire the resource exclusive */
//                 ExReleaseResourceLite(FcbHeader->Resource);
//                 ExAcquireResourceExclusiveLite(FcbHeader->Resource, TRUE);
//                 if (FileOffsetAppend != FALSE)
//                 {
//                     Offset.LowPart = FcbHeader->FileSize.LowPart; // ??
//                     NewSize.LowPart = FcbHeader->FileSize.LowPart + Length;
//
//                     /* Make sure we don't cross the 4GB boundary */
//                     b_4GB = (NewSize.LowPart < Offset.LowPart);
//                 }
//
//                 /* Recheck some of the conditions since we let the lock go */
//                 if ((FileObject->PrivateCacheMap != NULL) &&
//                     (FcbHeader->IsFastIoPossible != FastIoIsNotPossible) &&
//                     (FcbHeader->AllocationSize.LowPart >= NewSize.LowPart) &&
//                     (FcbHeader->AllocationSize.HighPart == 0) &&
//                     !b_4GB)
//                 {
//                     /* Do nothing? */
//                 }
//                 else
//                 {
//                     goto FailAndCleanup;
//                 }
//             }
//
//         }
//         else
//         {
//             goto FailAndCleanup;
//         }
//
//         /* Check if we need to find out if fast I/O is available */
//         if (FcbHeader->IsFastIoPossible == FastIoIsQuestionable)
//         {
//             _IO_STATUS_BLOCK FastIoCheckIfPossibleStatus;
//
//             /* Sanity check */
//             ASSERT(!KeIsExecutingDpc());
//
//             /* Get the Fast I/O table */
//             Device = IoGetRelatedDeviceObject(FileObject);
//             FastIoDispatch = Device->DriverObject->FastIoDispatch;
//
//             /* Sanity check */
//             ASSERT(FastIoDispatch != NULL);
//             ASSERT(FastIoDispatch->FastIoCheckIfPossible != NULL);
//
//             /* Ask the driver if we can do it */
//             if (!FastIoDispatch->FastIoCheckIfPossible(FileObject,
//                                                        FileOffsetAppend ?
//                                                         &FcbHeader->FileSize :
//                                                         FileOffset,
//                                                        Length,
//                                                        TRUE,
//                                                        LockKey,
//                                                        FALSE,
//                                                        &FastIoCheckIfPossibleStatus,
//                                                        Device))
//             {
//                 /* It's not, fail */
//                 goto FailAndCleanup;
//             }
//         }
//
//         /* If we are going to extend the file then save
//          * the old file size in case the operation fails.
//          */
//         if (NewSize.LowPart > FcbHeader->FileSize.LowPart)
//         {
//             FileSizeModified = TRUE;
//             OldFileSize.LowPart = FcbHeader->FileSize.LowPart;
//             OldValidDataLength.LowPart = FcbHeader->ValidDataLength.LowPart;
//             FcbHeader->FileSize.LowPart = NewSize.LowPart;
//         }
//
//         /* Set this as top-level IRP */
//         PsGetCurrentThread()->TopLevelIrp = FSRTL_FAST_IO_TOP_LEVEL_IRP;
//
//         _SEH2_TRY
//         {
//             if (Offset.LowPart > FcbHeader->ValidDataLength.LowPart)
//             {
//                 LARGE_INTEGER OffsetVar;
//                 OffsetVar.LowPart = Offset.LowPart;
//                 OffsetVar.HighPart = 0;
//                 CcZeroData(FileObject, &FcbHeader->ValidDataLength, &OffsetVar, TRUE);
//             }
//
//             /* Call the cache manager */
//             CcFastCopyWrite(FileObject, Offset.LowPart, Length, Buffer);
//         }
//         _SEH2_EXCEPT(FsRtlIsNtstatusExpected(_SEH2_GetExceptionCode()) ?
//                                              EXCEPTION_EXECUTE_HANDLER :
//                                              EXCEPTION_CONTINUE_SEARCH)
//         {
//             Result = FALSE;
//         }
//         _SEH2_END;
//
//         /* Remove ourselves at the top level component after the IO is done */
//         PsGetCurrentThread()->TopLevelIrp = 0;
//
//         /* Did the operation succeed? */
//         if (Result != FALSE)
//         {
//             /* Update the valid file size if necessary */
//             if (NewSize.LowPart > FcbHeader->ValidDataLength.LowPart)
//             {
//                 FcbHeader->ValidDataLength.LowPart = NewSize.LowPart;
//             }
//
//             /* Flag the file as modified */
//             FileObject->Flags |= FO_FILE_MODIFIED;
//
//             /* Update the strucutres if the file size changed */
//             if (FileSizeModified)
//             {
//                 SharedCacheMap =
//                     (PSHARED_CACHE_MAP)FileObject->SectionObjectPointer->SharedCacheMap;
//                 SharedCacheMap->FileSize.LowPart = NewSize.LowPart;
//                 FileObject->Flags |= FO_FILE_SIZE_CHANGED;
//             }
//
//             /* Update the file object current file offset */
//             FileObject->CurrentByteOffset.QuadPart = NewSize.LowPart;
//
//         }
//         else
//         {
//             /* Result == FALSE if we get here */
//             if (FileSizeModified)
//             {
//                 /* If the file size was modified then restore the old file size */
//                 if (FcbHeader->PagingIoResource != NULL)
//                 {
//                     /* Nagar P.544.
//                      * Restore the old file size if operation didn't succeed.
//                      */
//                     ExAcquireResourceExclusiveLite(FcbHeader->PagingIoResource, TRUE);
//                     FcbHeader->FileSize.LowPart = OldFileSize.LowPart;
//                     FcbHeader->ValidDataLength.LowPart = OldValidDataLength.LowPart;
//                     ExReleaseResourceLite(FcbHeader->PagingIoResource);
//                 }
//                 else
//                 {
//                     /* If there is no lock and do it without */
//                     FcbHeader->FileSize.LowPart = OldFileSize.LowPart;
//                     FcbHeader->ValidDataLength.LowPart = OldValidDataLength.LowPart;
//                 }
//             }
//             else
//             {
//                 /* Do nothing? */
//             }
//         }
//
//         goto Cleanup;
//     }
//     else
//     {
//         LARGE_INTEGER OldFileSize;
//
//         /* Sanity check */
//         ASSERT(!KeIsExecutingDpc());
//
//         /* Nagar P.544.
//          * Check if we need to acquire the resource exclusive.
//          */
//         if ((FileOffsetAppend == FALSE) &&
//             (FileOffset->QuadPart + Length <= FcbHeader->ValidDataLength.QuadPart))
//         {
//             /* Acquire the resource shared */
//             if (!ExAcquireResourceSharedLite(FcbHeader->Resource, Wait))
//             {
//                 goto LeaveCriticalAndFail;
//             }
//             ResourceAcquiredShared = TRUE;
//         }
//         else
//         {
//             /* Acquire the resource exclusive */
//             if (!ExAcquireResourceExclusiveLite(FcbHeader->Resource, Wait))
//             {
//                 goto LeaveCriticalAndFail;
//             }
//         }
//
//         /* Check if we are appending */
//         if (FileOffsetAppend != FALSE)
//         {
//             Offset.QuadPart = FcbHeader->FileSize.QuadPart;
//             NewSize.QuadPart = FcbHeader->FileSize.QuadPart + Length;
//         }
//         else
//         {
//             Offset.QuadPart = FileOffset->QuadPart;
//             NewSize.QuadPart += FileOffset->QuadPart + Length;
//         }
//
//         /* Nagar p.544/545.
//          * Make sure that caching is initated.
//          * That fast are allowed for this file stream.
//          * That we are not extending past the allocated size.
//          * That we are not creating a hole bigger than 8k.
//          */
//         if ((FileObject->PrivateCacheMap != NULL) &&
//             (FcbHeader->IsFastIoPossible != FastIoIsNotPossible) &&
//             (FcbHeader->ValidDataLength.QuadPart + 0x2000 > Offset.QuadPart) &&
//             (Length <= MAXLONGLONG - Offset.QuadPart) &&
//             (FcbHeader->AllocationSize.QuadPart >= NewSize.QuadPart))
//         {
//             /* Check if we can keep the lock shared */
//             if (ResourceAcquiredShared &&
//                 (NewSize.QuadPart > FcbHeader->ValidDataLength.QuadPart))
//             {
//                 ExReleaseResourceLite(FcbHeader->Resource);
//                 if (!ExAcquireResourceExclusiveLite(FcbHeader->Resource, Wait))
//                 {
//                     goto LeaveCriticalAndFail;
//                 }
//
//                 /* Compute the offset and the new filesize */
//                 if (FileOffsetAppend)
//                 {
//                     Offset.QuadPart = FcbHeader->FileSize.QuadPart;
//                     NewSize.QuadPart = FcbHeader->FileSize.QuadPart + Length;
//                 }
//
//                 /* Recheck the above points since we released and reacquire the lock */
//                 if ((FileObject->PrivateCacheMap != NULL) &&
//                     (FcbHeader->IsFastIoPossible != FastIoIsNotPossible) &&
//                     (FcbHeader->AllocationSize.QuadPart > NewSize.QuadPart))
//                 {
//                     /* Do nothing */
//                 }
//                 else
//                 {
//                     goto FailAndCleanup;
//                 }
//             }
//
//             /* Check if we need to find out if fast I/O is available */
//             if (FcbHeader->IsFastIoPossible == FastIoIsQuestionable)
//             {
//                 _IO_STATUS_BLOCK FastIoCheckIfPossibleStatus;
//
//                 /* Sanity check */
//                 ASSERT(!KeIsExecutingDpc());
//
//                 /* Get the Fast I/O table */
//                 Device = IoGetRelatedDeviceObject(FileObject);
//                 FastIoDispatch = Device->DriverObject->FastIoDispatch;
//
//                 /* Sanity check */
//                 ASSERT(FastIoDispatch != NULL);
//                 ASSERT(FastIoDispatch->FastIoCheckIfPossible != NULL);
//
//                 /* Ask the driver if we can do it */
//                 if (!FastIoDispatch->FastIoCheckIfPossible(FileObject,
//                                                            FileOffsetAppend ?
//                                                             &FcbHeader->FileSize :
//                                                             FileOffset,
//                                                            Length,
//                                                            Wait,
//                                                            LockKey,
//                                                            FALSE,
//                                                            &FastIoCheckIfPossibleStatus,
//                                                            Device))
//                 {
//                     /* It's not, fail */
//                     goto FailAndCleanup;
//                 }
//             }
//
//             /* If we are going to modify the filesize,
//              * save the old fs in case the operation fails.
//              */
//             if (NewSize.QuadPart > FcbHeader->FileSize.QuadPart)
//             {
//                 FileSizeModified = TRUE;
//                 OldFileSize.QuadPart = FcbHeader->FileSize.QuadPart;
//                 OldValidDataLength.QuadPart = FcbHeader->ValidDataLength.QuadPart;
//
//                 /* If the high part of the filesize is going
//                  * to change, grab the Paging IoResouce.
//                  */
//                 if (NewSize.HighPart != FcbHeader->FileSize.HighPart &&
//                     FcbHeader->PagingIoResource)
//                 {
//                     ExAcquireResourceExclusiveLite(FcbHeader->PagingIoResource, TRUE);
//                     FcbHeader->FileSize.QuadPart = NewSize.QuadPart;
//                     ExReleaseResourceLite(FcbHeader->PagingIoResource);
//                 }
//                 else
//                 {
//                     FcbHeader->FileSize.QuadPart = NewSize.QuadPart;
//                 }
//             }
//
//             /* Nagar p.544.
//              * Set ourselves as top component.
//              */
//             PsGetCurrentThread()->TopLevelIrp = FSRTL_FAST_IO_TOP_LEVEL_IRP;
//
//             _SEH2_TRY
//             {
//                 BOOLEAN CallCc = TRUE;
//
//                 /* Check if there is a gap between the end of the file
//                  * and the offset. If yes, then we have to zero the data.
//                  */
//                 if (Offset.QuadPart > FcbHeader->ValidDataLength.QuadPart)
//                 {
//                     if (!(Result = CcZeroData(FileObject,
//                                               &FcbHeader->ValidDataLength,
//                                               &Offset,
//                                               Wait)))
//                     {
//                         /* If this operation fails, then we have to exit. We can't jump
//                          * outside the SEH, so I am using a variable to exit normally.
//                          */
//                         CallCc = FALSE;
//                     }
//                 }
//
//                 /* Unless the CcZeroData failed, call the cache manager */
//                 if (CallCc)
//                 {
//                     Result = CcCopyWrite(FileObject, &Offset, Length, Wait, Buffer);
//                 }
//             }
//             _SEH2_EXCEPT(FsRtlIsNtstatusExpected(_SEH2_GetExceptionCode()) ?
//                                                  EXCEPTION_EXECUTE_HANDLER :
//                                                  EXCEPTION_CONTINUE_SEARCH)
//             {
//                 Result = FALSE;
//             }
//             _SEH2_END;
//
//             /* Reset the top component */
//             PsGetCurrentThread()->TopLevelIrp = FSRTL_FAST_IO_TOP_LEVEL_IRP;
//
//             /* Did the operation suceeded */
//             if (Result)
//             {
//                 /* Check if we need to update the filesize */
//                 if (NewSize.QuadPart > FcbHeader->ValidDataLength.QuadPart)
//                 {
//                     if (NewSize.HighPart != FcbHeader->ValidDataLength.HighPart &&
//                         FcbHeader->PagingIoResource)
//                     {
//                         ExAcquireResourceExclusiveLite(FcbHeader->PagingIoResource, TRUE);
//                         FcbHeader->ValidDataLength.QuadPart = NewSize.QuadPart;
//                         ExReleaseResourceLite(FcbHeader->PagingIoResource);
//                     }
//                     else
//                     {
//                         FcbHeader->ValidDataLength.QuadPart = NewSize.QuadPart;
//                     }
//                 }
//
//                 /* Flag the file as modified */
//                 FileObject->Flags |= FO_FILE_MODIFIED;
//
//                 /* Check if the filesize has changed */
//                 if (FileSizeModified)
//                 {
//                     /* Update the file sizes */
//                     SharedCacheMap =
//                         (PSHARED_CACHE_MAP)FileObject->SectionObjectPointer->SharedCacheMap;
//                     SharedCacheMap->FileSize.QuadPart = NewSize.QuadPart;
//                     FileObject->Flags |= FO_FILE_SIZE_CHANGED;
//                 }
//
//                 /* Update the current FO byte offset */
//                 FileObject->CurrentByteOffset.QuadPart = NewSize.QuadPart;
//             }
//             else
//             {
//                 /* The operation did not succeed.
//                  * Reset the file size to what it should be.
//                  */
//                 if (FileSizeModified)
//                 {
//                     if (FcbHeader->PagingIoResource)
//                     {
//                         ExAcquireResourceExclusiveLite(FcbHeader->PagingIoResource, TRUE);
//                         FcbHeader->FileSize.QuadPart = OldFileSize.QuadPart;
//                         FcbHeader->ValidDataLength.QuadPart = OldValidDataLength.QuadPart;
//                         ExReleaseResourceLite(FcbHeader->PagingIoResource);
//                     }
//                     else
//                     {
//                         FcbHeader->FileSize.QuadPart = OldFileSize.QuadPart;
//                         FcbHeader->ValidDataLength.QuadPart = OldValidDataLength.QuadPart;
//                     }
//                 }
//             }
//
//             goto Cleanup;
//         }
//         else
//         {
//             goto FailAndCleanup;
//         }
//     }
//
// LeaveCriticalAndFail:
//
//     FsRtlExitFileSystem();
//     return FALSE;
//
// FailAndCleanup:
//
//     ExReleaseResourceLite(FcbHeader->Resource);
//     FsRtlExitFileSystem();
//     return FALSE;
//
// Cleanup:
//
//     ExReleaseResourceLite(FcbHeader->Resource);
//     FsRtlExitFileSystem();
//     return Result;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// FsRtlGetFileSize(IN PFILE_OBJECT FileObject,
//                  IN OUT PLARGE_INTEGER FileSize)
// {
//     FILE_STANDARD_INFORMATION Info;
//     NTSTATUS Status;
//     _IO_STATUS_BLOCK IoStatus;
//     PDEVICE_OBJECT DeviceObject;
//     PFAST_IO_DISPATCH FastDispatch;
//     KEVENT Event;
//     PIO_STACK_LOCATION IoStackLocation;
//     PIRP Irp;
//     BOOLEAN OldHardError;
//
//     PAGED_CODE();
//
//     /* Get Device Object and Fast Calls */
//     DeviceObject = IoGetRelatedDeviceObject(FileObject);
//     FastDispatch = DeviceObject->DriverObject->FastIoDispatch;
//
//     /* Check if we support Fast Calls, and check FastIoQueryStandardInfo.
//      * Call the function and see if it succeeds.
//      */
//     if (!FastDispatch ||
//         !FastDispatch->FastIoQueryStandardInfo ||
//         !FastDispatch->FastIoQueryStandardInfo(FileObject,
//                                                TRUE,
//                                                &Info,
//                                                &IoStatus,
//                                                DeviceObject))
//     {
//         /* If any of the above failed, then we are going to send an
//          * IRP to the device object. Initialize the event for the IO.
//          */
//         KeInitializeEvent(&Event, NotificationEvent, FALSE);
//
//         /* Allocate the IRP */
//         Irp = IoAllocateIrp(DeviceObject->StackSize, FALSE);
//
//         if (Irp == NULL)
//         {
//             return STATUS_INSUFFICIENT_RESOURCES;
//         }
//
//         /* Don't process hard error */
//         OldHardError = IoSetThreadHardErrorMode(FALSE);
//
//         /* Setup the IRP */
//         Irp->UserIosb = &IoStatus;
//         Irp->UserEvent = &Event;
//         Irp->Tail.Overlay.Thread = PsGetCurrentThread();
//         Irp->Flags = IRP_SYNCHRONOUS_PAGING_IO | IRP_PAGING_IO;
//         Irp->RequestorMode = KernelMode;
//         Irp->Tail.Overlay.OriginalFileObject = FileObject;
//         Irp->AssociatedIrp.SystemBuffer = &Info;
//
//         /* Setup out stack location */
//         IoStackLocation = Irp->Tail.Overlay.CurrentStackLocation;
//         IoStackLocation--;
//         IoStackLocation->MajorFunction = IRP_MJ_QUERY_INFORMATION;
//         IoStackLocation->FileObject = FileObject;
//         IoStackLocation->DeviceObject = DeviceObject;
//         IoStackLocation->Parameters.QueryFile.Length =
//             sizeof(FILE_STANDARD_INFORMATION);
//         IoStackLocation->Parameters.QueryFile.FileInformationClass =
//             FileStandardInformation;
//
//         /* Send the IRP to the related device object */
//         Status = IofCallDriver(DeviceObject, Irp);
//
//         /* Standard DDK IRP result processing */
//         if (Status == STATUS_PENDING)
//         {
//             KeWaitForSingleObject(&Event, Executive, KernelMode, FALSE, NULL);
//         }
//
//         /* If there was a synchronous error, signal it */
//         if (!NT_SUCCESS(Status))
//         {
//             IoStatus.Status = Status;
//         }
//
//         IoSetThreadHardErrorMode(OldHardError);
//     }
//
//     /* Check the sync/async IO result */
//     if (NT_SUCCESS(IoStatus.Status))
//     {
//         /* Was the request for a directory? */
//         if (Info.Directory)
//         {
//             IoStatus.Status = STATUS_FILE_IS_A_DIRECTORY;
//         }
//         else
//         {
//             FileSize->QuadPart = Info.EndOfFile.QuadPart;
//         }
//     }
//
//     return IoStatus.Status;
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// NTAPI
// FsRtlMdlRead(IN PFILE_OBJECT FileObject,
//              IN PLARGE_INTEGER FileOffset,
//              IN ULONG Length,
//              IN ULONG LockKey,
//              OUT PMDL *MdlChain,
//              OUT PIO_STATUS_BLOCK IoStatus)
// {
//     PDEVICE_OBJECT DeviceObject, BaseDeviceObject;
//     PFAST_IO_DISPATCH FastDispatch;
//
//     /* Get Device Object and Fast Calls */
//     DeviceObject = IoGetRelatedDeviceObject(FileObject);
//     FastDispatch = DeviceObject->DriverObject->FastIoDispatch;
//
//     /* Check if we support Fast Calls, and check this one */
//     if (FastDispatch && FastDispatch->MdlRead)
//     {
//         /* Use the fast path */
//         return FastDispatch->MdlRead(FileObject,
//                                      FileOffset,
//                                      Length,
//                                      LockKey,
//                                      MdlChain,
//                                      IoStatus,
//                                      DeviceObject);
//     }
//
//     /* Get the Base File System (Volume) and Fast Calls */
//     BaseDeviceObject = IoGetBaseFileSystemDeviceObject(FileObject);
//     FastDispatch = BaseDeviceObject->DriverObject->FastIoDispatch;
//
//     /* If the Base Device Object has its own FastDispatch Routine, fail */
//     if (FastDispatch && FastDispatch->MdlRead && BaseDeviceObject != DeviceObject)
//     {
//         return FALSE;
//     }
//
//     /* No fast path, use slow path */
//     return FsRtlMdlReadDev(FileObject,
//                            FileOffset,
//                            Length,
//                            LockKey,
//                            MdlChain,
//                            IoStatus,
//                            DeviceObject);
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// NTAPI
// FsRtlMdlReadComplete(IN PFILE_OBJECT FileObject,
//                      IN OUT PMDL MdlChain)
// {
//     PDEVICE_OBJECT DeviceObject, BaseDeviceObject;
//     PFAST_IO_DISPATCH FastDispatch;
//
//     /* Get Device Object and Fast Calls */
//     DeviceObject = IoGetRelatedDeviceObject(FileObject);
//     FastDispatch = DeviceObject->DriverObject->FastIoDispatch;
//
//     /* Check if we support Fast Calls, and check this one */
//     if (FastDispatch && FastDispatch->MdlReadComplete)
//     {
//         /* Use the fast path */
//         return FastDispatch->MdlReadComplete(FileObject, MdlChain, DeviceObject);
//     }
//
//     /* Get the Base File System (Volume) and Fast Calls */
//     BaseDeviceObject = IoGetBaseFileSystemDeviceObject(FileObject);
//     FastDispatch = BaseDeviceObject->DriverObject->FastIoDispatch;
//
//     /* If the Base Device Object has its own FastDispatch Routine, fail */
//     if ((BaseDeviceObject != DeviceObject) &&
//         FastDispatch &&
//         FastDispatch->MdlReadComplete)
//     {
//         return FALSE;
//     }
//
//     /* No fast path, use slow path */
//     return FsRtlMdlReadCompleteDev(FileObject, MdlChain, DeviceObject);
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// NTAPI
// FsRtlMdlReadCompleteDev(IN PFILE_OBJECT FileObject,
//                         IN PMDL MemoryDescriptorList,
//                         IN PDEVICE_OBJECT DeviceObject)
// {
//     /* Call the Cache Manager */
//     CcMdlReadComplete2(FileObject, MemoryDescriptorList);
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// NTAPI
// FsRtlMdlReadDev(IN PFILE_OBJECT FileObject,
//                 IN PLARGE_INTEGER FileOffset,
//                 IN ULONG Length,
//                 IN ULONG LockKey,
//                 OUT PMDL *MdlChain,
//                 OUT PIO_STATUS_BLOCK IoStatus,
//                 IN PDEVICE_OBJECT DeviceObject)
// {
//     PFSRTL_COMMON_FCB_HEADER FcbHeader;
//     BOOLEAN Result = TRUE;
//     LARGE_INTEGER Offset;
//     PFAST_IO_DISPATCH FastIoDispatch;
//     PDEVICE_OBJECT Device;
//     PAGED_CODE();
//
//     /* No actual read */
//     if (!Length)
//     {
//         /* Return success */
//         IoStatus->Status = STATUS_SUCCESS;
//         IoStatus->Information = 0;
//         return TRUE;
//     }
//
//     /* Sanity check */
//     ASSERT(MAXLONGLONG - FileOffset->QuadPart >= (LONGLONG)Length);
//
//     /* Get the offset and FCB header */
//     Offset.QuadPart = FileOffset->QuadPart + Length;
//     FcbHeader = (PFSRTL_COMMON_FCB_HEADER)FileObject->FsContext;
//
//     /* Enter the FS */
//     FsRtlEnterFileSystem();
//,++;
//
//     /* Lock the FCB */
//     ExAcquireResourceShared(FcbHeader->Resource, TRUE);
//
//     /* Check if this is a fast I/O cached file */
//     if (!(FileObject->PrivateCacheMap) ||
//         (FcbHeader->IsFastIoPossible == FastIoIsNotPossible))
//     {
//         /* It's not, so fail */
//,+= 1;
//         Result = FALSE;
//         goto Cleanup;
//     }
//
//     /* Check if we need to find out if fast I/O is available */
//     if (FcbHeader->IsFastIoPossible == FastIoIsQuestionable)
//     {
//         /* Get the Fast I/O table */
//         Device = IoGetRelatedDeviceObject(FileObject);
//         FastIoDispatch = Device->DriverObject->FastIoDispatch;
//
//         /* Sanity check */
//         ASSERT(!KeIsExecutingDpc());
//         ASSERT(FastIoDispatch != NULL);
//         ASSERT(FastIoDispatch->FastIoCheckIfPossible != NULL);
//
//         /* Ask the driver if we can do it */
//         if (!FastIoDispatch->FastIoCheckIfPossible(FileObject,
//                                                    FileOffset,
//                                                    Length,
//                                                    TRUE,
//                                                    LockKey,
//                                                    TRUE,
//                                                    IoStatus,
//                                                    Device))
//         {
//             /* It's not, fail */
//,+= 1;
//             Result = FALSE;
//             goto Cleanup;
//         }
//     }
//
//     /* Check if we read too much */
//     if (Offset.QuadPart > FcbHeader->FileSize.QuadPart)
//     {
//         /* We did, check if the file offset is past the end */
//         if (FileOffset->QuadPart >= FcbHeader->FileSize.QuadPart)
//         {
//             /* Set end of file */
//             IoStatus->Status = STATUS_END_OF_FILE;
//             IoStatus->Information = 0;
//             goto Cleanup;
//         }
//
//         /* Otherwise, just normalize the length */
//         Length = (ULONG)(FcbHeader->FileSize.QuadPart - FileOffset->QuadPart);
//     }
//
//     /* Set this as top-level IRP */
//     PsGetCurrentThread()->TopLevelIrp = FSRTL_FAST_IO_TOP_LEVEL_IRP;
//
//     _SEH2_TRY
//     {
//         /* Attempt a read */
//         CcMdlRead(FileObject, FileOffset, Length, MdlChain, IoStatus);
//         FileObject->Flags |= FO_FILE_FAST_IO_READ;
//     }
//     _SEH2_EXCEPT(FsRtlIsNtstatusExpected(_SEH2_GetExceptionCode()) ?
//                                          EXCEPTION_EXECUTE_HANDLER :
//                                          EXCEPTION_CONTINUE_SEARCH)
//     {
//         Result = FALSE;
//     }
//     _SEH2_END;
//
//
//     /* Remove the top-level IRP flag */
//     PsGetCurrentThread()->TopLevelIrp = 0;
//
//     /* Return to caller */
// Cleanup:
//
//     ExReleaseResourceLite(FcbHeader->Resource);
//     FsRtlExitFileSystem();
//
//     return Result;
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// NTAPI
// FsRtlMdlWriteComplete(IN PFILE_OBJECT FileObject,
//                       IN PLARGE_INTEGER FileOffset,
//                       IN PMDL MdlChain)
// {
//     PDEVICE_OBJECT DeviceObject, BaseDeviceObject;
//     PFAST_IO_DISPATCH FastDispatch;
//
//     /* Get Device Object and Fast Calls */
//     DeviceObject = IoGetRelatedDeviceObject(FileObject);
//     FastDispatch = DeviceObject->DriverObject->FastIoDispatch;
//
//     /* Check if we support Fast Calls, and check this one */
//     if (FastDispatch && FastDispatch->MdlWriteComplete)
//     {
//         /* Use the fast path */
//         return FastDispatch->MdlWriteComplete(FileObject,
//                                               FileOffset,
//                                               MdlChain,
//                                               DeviceObject);
//     }
//
//     /* Get the Base File System (Volume) and Fast Calls */
//     BaseDeviceObject = IoGetBaseFileSystemDeviceObject(FileObject);
//     FastDispatch = BaseDeviceObject->DriverObject->FastIoDispatch;
//
//     /* If the Base Device Object has its own FastDispatch Routine, fail */
//     if (FastDispatch &&
//         FastDispatch->MdlWriteComplete &&
//         BaseDeviceObject != DeviceObject)
//     {
//         return FALSE;
//     }
//
//     /* No fast path, use slow path */
//     return FsRtlMdlWriteCompleteDev(FileObject,
//                                     FileOffset,
//                                     MdlChain,
//                                     DeviceObject);
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// NTAPI
// FsRtlMdlWriteCompleteDev(IN PFILE_OBJECT FileObject,
//                          IN PLARGE_INTEGER FileOffset,
//                          IN PMDL MdlChain,
//                          IN PDEVICE_OBJECT DeviceObject)
// {
//     if (FileObject->Flags & FO_WRITE_THROUGH)
//     {
//         return FALSE;
//     }
//
//     /* Call the Cache Manager */
//     CcMdlWriteComplete2(FileObject, FileOffset, MdlChain);
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// NTAPI
// FsRtlPrepareMdlWrite(IN PFILE_OBJECT FileObject,
//                      IN PLARGE_INTEGER FileOffset,
//                      IN ULONG Length,
//                      IN ULONG LockKey,
//                      OUT PMDL *MdlChain,
//                      OUT PIO_STATUS_BLOCK IoStatus)
// {
//     PDEVICE_OBJECT DeviceObject, BaseDeviceObject;
//     PFAST_IO_DISPATCH FastDispatch;
//
//     /* Get Device Object and Fast Calls */
//     DeviceObject = IoGetRelatedDeviceObject(FileObject);
//     FastDispatch = DeviceObject->DriverObject->FastIoDispatch;
//
//     /* Check if we support Fast Calls, and check this one */
//     if (FastDispatch && FastDispatch->PrepareMdlWrite)
//     {
//         /* Use the fast path */
//         return FastDispatch->PrepareMdlWrite(FileObject,
//                                              FileOffset,
//                                              Length,
//                                              LockKey,
//                                              MdlChain,
//                                              IoStatus,
//                                              DeviceObject);
//     }
//
//     /* Get the Base File System (Volume) and Fast Calls */
//     BaseDeviceObject = IoGetBaseFileSystemDeviceObject(FileObject);
//     FastDispatch = BaseDeviceObject->DriverObject->FastIoDispatch;
//
//     /* If the Base Device Object has its own FastDispatch Routine, fail */
//     if (FastDispatch &&
//         FastDispatch->PrepareMdlWrite &&
//         BaseDeviceObject != DeviceObject)
//     {
//         return FALSE;
//     }
//
//     /* No fast path, use slow path */
//     return FsRtlPrepareMdlWriteDev(FileObject,
//                                    FileOffset,
//                                    Length,
//                                    LockKey,
//                                    MdlChain,
//                                    IoStatus,
//                                    DeviceObject);
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// NTAPI
// FsRtlPrepareMdlWriteDev(IN PFILE_OBJECT FileObject,
//                         IN PLARGE_INTEGER FileOffset,
//                         IN ULONG Length,
//                         IN ULONG LockKey,
//                         OUT PMDL *MdlChain,
//                         OUT PIO_STATUS_BLOCK IoStatus,
//                         IN PDEVICE_OBJECT DeviceObject)
// {
//     BOOLEAN Result = TRUE;
//     PFAST_IO_DISPATCH FastIoDispatch;
//     PDEVICE_OBJECT Device;
//     PFSRTL_COMMON_FCB_HEADER FcbHeader;
//     PSHARED_CACHE_MAP SharedCacheMap;
//
//     LARGE_INTEGER OldFileSize;
//     LARGE_INTEGER OldValidDataLength;
//     LARGE_INTEGER NewSize;
//     LARGE_INTEGER Offset;
//
//     /* WDK doc.
//      * Offset == 0xffffffffffffffff indicates append to the end of file.
//      */
//     BOOLEAN FileOffsetAppend = (FileOffset->HighPart == (LONG)0xffffffff) &&
//                                (FileOffset->LowPart == 0xffffffff);
//
//     BOOLEAN FileSizeModified = FALSE;
//     BOOLEAN ResourceAcquiredShared = FALSE;
//
//     /* Initialize some of the vars and pointers */
//     OldFileSize.QuadPart = 0;
//     OldValidDataLength.QuadPart = 0;
//
//     PAGED_CODE();
//
//     Offset.QuadPart = FileOffset->QuadPart + Length;
//
//     /* Nagar p.544.
//      * Check with Cc if we can write.
//      */
//     if (!CcCanIWrite(FileObject, Length, TRUE, FALSE) ||
//         (FileObject->Flags & FO_WRITE_THROUGH))
//     {
//         return FALSE;
//     }
//
//     IoStatus->Status = STATUS_SUCCESS;
//
//     /* No actual read */
//     if (!Length)
//     {
//         return TRUE;
//     }
//
//     FcbHeader = (PFSRTL_COMMON_FCB_HEADER)FileObject->FsContext;
//     FsRtlEnterFileSystem();
//
//     /* Check we are going to extend the file */
//     if ((FileOffsetAppend == FALSE) &&
//         (FileOffset->QuadPart + Length <= FcbHeader->ValidDataLength.QuadPart))
//     {
//         /* Acquire the resource shared */
//         ExAcquireResourceSharedLite(FcbHeader->Resource, TRUE);
//         ResourceAcquiredShared = TRUE;
//     }
//     else
//     {
//         /* Acquire the resource exclusive */
//         ExAcquireResourceExclusiveLite(FcbHeader->Resource, TRUE);
//     }
//
//     /* Check if we are appending */
//     if (FileOffsetAppend != FALSE)
//     {
//         Offset.QuadPart = FcbHeader->FileSize.QuadPart;
//         NewSize.QuadPart = FcbHeader->FileSize.QuadPart + Length;
//     }
//     else
//     {
//         Offset.QuadPart = FileOffset->QuadPart;
//         NewSize.QuadPart = FileOffset->QuadPart + Length;
//     }
//
//     if ((FileObject->PrivateCacheMap) &&
//         (FcbHeader->IsFastIoPossible) &&
//         (Length <= MAXLONGLONG - FileOffset->QuadPart) &&
//         (NewSize.QuadPart <= FcbHeader->AllocationSize.QuadPart))
//     {
//         /* Check if we can keep the lock shared */
//         if (ResourceAcquiredShared &&
//             (NewSize.QuadPart > FcbHeader->ValidDataLength.QuadPart))
//         {
//             ExReleaseResourceLite(FcbHeader->Resource);
//             ExAcquireResourceExclusiveLite(FcbHeader->Resource, TRUE);
//
//             /* Compute the offset and the new filesize */
//             if (FileOffsetAppend)
//             {
//                 Offset.QuadPart = FcbHeader->FileSize.QuadPart;
//                 NewSize.QuadPart = FcbHeader->FileSize.QuadPart + Length;
//             }
//
//             /* Recheck the above points since we released and reacquire the lock */
//             if ((FileObject->PrivateCacheMap != NULL) &&
//                 (FcbHeader->IsFastIoPossible != FastIoIsNotPossible) &&
//                 (FcbHeader->AllocationSize.QuadPart > NewSize.QuadPart))
//             {
//                 /* Do nothing */
//             }
//             else
//             {
//                 goto FailAndCleanup;
//             }
//         }
//
//         /* Check if we need to find out if fast I/O is available */
//         if (FcbHeader->IsFastIoPossible == FastIoIsQuestionable)
//         {
//             /* Sanity check */
//             /* ASSERT(!KeIsExecutingDpc()); */
//
//             /* Get the Fast I/O table */
//             Device = IoGetRelatedDeviceObject(FileObject);
//             FastIoDispatch = Device->DriverObject->FastIoDispatch;
//
//             /* Sanity check */
//             ASSERT(FastIoDispatch != NULL);
//             ASSERT(FastIoDispatch->FastIoCheckIfPossible != NULL);
//
//             /* Ask the driver if we can do it */
//             if (!FastIoDispatch->FastIoCheckIfPossible(FileObject,
//                                                        FileOffset,
//                                                        Length,
//                                                        TRUE,
//                                                        LockKey,
//                                                        FALSE,
//                                                        IoStatus,
//                                                        Device))
//             {
//                 /* It's not, fail */
//                 goto FailAndCleanup;
//             }
//         }
//
//         /* If we are going to modify the filesize,
//          * save the old fs in case the operation fails.
//          */
//         if (NewSize.QuadPart > FcbHeader->FileSize.QuadPart)
//         {
//             FileSizeModified = TRUE;
//             OldFileSize.QuadPart = FcbHeader->FileSize.QuadPart;
//             OldValidDataLength.QuadPart = FcbHeader->ValidDataLength.QuadPart;
//
//             /* If the high part of the filesize is going
//              * to change, grab the Paging IoResouce.
//              */
//             if (NewSize.HighPart != FcbHeader->FileSize.HighPart &&
//                 FcbHeader->PagingIoResource)
//             {
//                 ExAcquireResourceExclusiveLite(FcbHeader->PagingIoResource, TRUE);
//                 FcbHeader->FileSize.QuadPart = NewSize.QuadPart;
//                 ExReleaseResourceLite(FcbHeader->PagingIoResource);
//             }
//             else
//             {
//                 FcbHeader->FileSize.QuadPart = NewSize.QuadPart;
//             }
//         }
//
//
//         /* Nagar p.544.
//          * Set ourselves as top component.
//          */
//         PsGetCurrentThread()->TopLevelIrp = FSRTL_FAST_IO_TOP_LEVEL_IRP;
//         _SEH2_TRY
//         {
//             /* Check if there is a gap between the end of the file and the offset.
//              * If yes, then we have to zero the data.
//              */
//             if (Offset.QuadPart > FcbHeader->ValidDataLength.QuadPart)
//             {
//                 Result = CcZeroData(FileObject,
//                                     &FcbHeader->ValidDataLength,
//                                     &Offset,
//                                     TRUE);
//                 if (Result)
//                 {
//                     CcPrepareMdlWrite(FileObject,
//                                       &Offset,
//                                       Length,
//                                       MdlChain,
//                                       IoStatus);
//                 }
//             }
//             else
//             {
//                 CcPrepareMdlWrite(FileObject, &Offset, Length, MdlChain, IoStatus);
//             }
//
//         }
//         _SEH2_EXCEPT(FsRtlIsNtstatusExpected(_SEH2_GetExceptionCode()) ?
//                                              EXCEPTION_EXECUTE_HANDLER :
//                                              EXCEPTION_CONTINUE_SEARCH)
//         {
//             Result = FALSE;
//         }
//         _SEH2_END;
//
//         /* Reset the top component */
//         PsGetCurrentThread()->TopLevelIrp = 0;
//
//         /* Did the operation suceeded */
//         if (Result)
//         {
//             /* Check if we need to update the filesize */
//             if (NewSize.QuadPart > FcbHeader->ValidDataLength.QuadPart)
//             {
//                 if (NewSize.HighPart != FcbHeader->ValidDataLength.HighPart &&
//                     FcbHeader->PagingIoResource)
//                 {
//                     ExAcquireResourceExclusiveLite(FcbHeader->PagingIoResource, TRUE);
//                     FcbHeader->ValidDataLength.QuadPart = NewSize.QuadPart;
//                     ExReleaseResourceLite(FcbHeader->PagingIoResource);
//                 }
//                 else
//                 {
//                     FcbHeader->ValidDataLength.QuadPart = NewSize.QuadPart;
//                 }
//             }
//
//             /* Flag the file as modified */
//             FileObject->Flags |= FO_FILE_MODIFIED;
//
//             /* Check if the filesize has changed */
//             if (FileSizeModified)
//             {
//                 SharedCacheMap =
//                     (PSHARED_CACHE_MAP)FileObject->SectionObjectPointer->SharedCacheMap;
//                 SharedCacheMap->FileSize.QuadPart = NewSize.QuadPart;
//                 FileObject->Flags |= FO_FILE_SIZE_CHANGED;
//             }
//         }
//         else
//         {
//             /* The operation did not succeed.
//              * Reset the file size to what it should be.
//              */
//             if (FileSizeModified)
//             {
//                 if (FcbHeader->PagingIoResource)
//                 {
//                     ExAcquireResourceExclusiveLite(FcbHeader->PagingIoResource, TRUE);
//                     FcbHeader->FileSize.QuadPart = OldFileSize.QuadPart;
//                     FcbHeader->ValidDataLength.QuadPart = OldValidDataLength.QuadPart;
//                     ExReleaseResourceLite(FcbHeader->PagingIoResource);
//                 }
//                 else
//                 {
//                     FcbHeader->FileSize.QuadPart = OldFileSize.QuadPart;
//                     FcbHeader->ValidDataLength.QuadPart = OldValidDataLength.QuadPart;
//                 }
//             }
//         }
//
//         goto Cleanup;
//     }
//     else
//     {
//         goto FailAndCleanup;
//     }
//
// FailAndCleanup:
//
//     ExReleaseResourceLite(FcbHeader->Resource);
//     FsRtlExitFileSystem();
//     return FALSE;
//
// Cleanup:
//
//     ExReleaseResourceLite(FcbHeader->Resource);
//     FsRtlExitFileSystem();
//     return Result;
// }
//
// NTSTATUS
// NTAPI
// FsRtlAcquireFileExclusiveCommon(IN PFILE_OBJECT FileObject,
//                                 IN FS_FILTER_SECTION_SYNC_TYPE SyncType,
//                                 IN ULONG Reserved)
// {
//     PFSRTL_COMMON_FCB_HEADER FcbHeader;
//     PDEVICE_OBJECT DeviceObject;
//     PFAST_IO_DISPATCH FastDispatch;
//     PEXTENDED_DRIVER_EXTENSION DriverExtension;
//     PFS_FILTER_CALLBACKS FilterCallbacks;
//
//     /* Get Device Object and Fast Calls */
//     FcbHeader = (PFSRTL_COMMON_FCB_HEADER)FileObject->FsContext;
//     DeviceObject = IoGetRelatedDeviceObject(FileObject);
//
//     /* Get master FsRtl lock */
//     FsRtlEnterFileSystem();
//
//     DriverExtension = (PEXTENDED_DRIVER_EXTENSION)DeviceObject->DriverObject->DriverExtension;
//     FilterCallbacks = DriverExtension->FsFilterCallbacks;
//
//     /* Check if Filter Cllbacks are supported */
//     if (FilterCallbacks && FilterCallbacks->PreAcquireForSectionSynchronization)
//     {
//         NTSTATUS Status;
//         PVOID CompletionContext;
//
//         FS_FILTER_CALLBACK_DATA CbData;
//
//         RtlZeroMemory(&CbData, sizeof(CbData));
//
//         CbData.SizeOfFsFilterCallbackData = sizeof(CbData);
//         CbData.Operation = FS_FILTER_ACQUIRE_FOR_SECTION_SYNCHRONIZATION;
//         CbData.DeviceObject = DeviceObject;
//         CbData.FileObject = FileObject;
//         CbData.Parameters.AcquireForSectionSynchronization.PageProtection = Reserved;
//         CbData.Parameters.AcquireForSectionSynchronization.SyncType = SyncType;
//
//         Status = FilterCallbacks->PreAcquireForSectionSynchronization(&CbData, &CompletionContext);
//         if (!NT_SUCCESS(Status))
//         {
//             FsRtlExitFileSystem();
//             return Status;
//         }
//
//         /* Should we do something in-between ? */
//
//         if (FilterCallbacks->PostAcquireForSectionSynchronization)
//         {
//             FilterCallbacks->PostAcquireForSectionSynchronization(&CbData, Status, CompletionContext);
//         }
//
//         /* Return here when the status is based on the synchonization type and write access to the file */
//         if (Status == STATUS_FSFILTER_OP_COMPLETED_SUCCESSFULLY ||
//             Status == STATUS_FILE_LOCKED_WITH_ONLY_READERS ||
//             Status == STATUS_FILE_LOCKED_WITH_WRITERS)
//         {
//             return Status;
//         }
//     }
//
//     FastDispatch = DeviceObject->DriverObject->FastIoDispatch;
//
//     /* Check if Fast Calls are supported, and check AcquireFileForNtCreateSection */
//     if (FastDispatch &&
//         FastDispatch->AcquireFileForNtCreateSection)
//     {
//         /* Call the AcquireFileForNtCreateSection FastIo handler */
//         FastDispatch->AcquireFileForNtCreateSection(FileObject);
//     }
//     else
//     {
//         /* No FastIo handler, acquire file's resource exclusively */
//         if (FcbHeader && FcbHeader->Resource) ExAcquireResourceExclusiveLite(FcbHeader->Resource, TRUE);
//     }
//
//     return STATUS_SUCCESS;
// }
//
// /*
// * @implemented
// */
// VOID
// NTAPI
// FsRtlAcquireFileExclusive(IN PFILE_OBJECT FileObject)
// {
//     PAGED_CODE();
//
//     /* Call the common routine. Don't care about the result */
//     (VOID)FsRtlAcquireFileExclusiveCommon(FileObject, SyncTypeOther, 0);
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// FsRtlAcquireToCreateMappedSection(_In_ PFILE_OBJECT FileObject,
//                                   _In_ ULONG SectionPageProtection)
// {
//     PAGED_CODE();
//
//     return FsRtlAcquireFileExclusiveCommon(FileObject, SyncTypeCreateSection, SectionPageProtection);
// }
// /*
// * @implemented
// */
// VOID
// NTAPI
// FsRtlReleaseFile(IN PFILE_OBJECT FileObject)
// {
//     PFSRTL_COMMON_FCB_HEADER FcbHeader;
//     PDEVICE_OBJECT DeviceObject;
//     PFAST_IO_DISPATCH FastDispatch;
//
//     /* Get Device Object and Fast Calls */
//     FcbHeader = (PFSRTL_COMMON_FCB_HEADER)FileObject->FsContext;
//     DeviceObject = IoGetRelatedDeviceObject(FileObject);
//     FastDispatch = DeviceObject->DriverObject->FastIoDispatch;
//
//     /* Check if Fast Calls are supported and check ReleaseFileForNtCreateSection */
//     if (FastDispatch &&
//         FastDispatch->ReleaseFileForNtCreateSection)
//     {
//         /* Call the ReleaseFileForNtCreateSection FastIo handler */
//         FastDispatch->ReleaseFileForNtCreateSection(FileObject);
//     }
//     else
//     {
//         /* No FastIo handler, release file's resource */
//         if (FcbHeader && FcbHeader->Resource) ExReleaseResourceLite(FcbHeader->Resource);
//     }
//
//     /* Release master FsRtl lock */
//     FsRtlExitFileSystem();
// }
//
// /*
// * @implemented
// */
// NTSTATUS
// NTAPI
// FsRtlAcquireFileForCcFlushEx(IN PFILE_OBJECT FileObject)
// {
//     PFSRTL_COMMON_FCB_HEADER FcbHeader;
//     PDEVICE_OBJECT DeviceObject, BaseDeviceObject;
//     PFAST_IO_DISPATCH FastDispatch;
//     NTSTATUS Status;
//
//     /* Get the Base File System (Volume) and Fast Calls */
//     FcbHeader = (PFSRTL_COMMON_FCB_HEADER)FileObject->FsContext;
//     DeviceObject = IoGetRelatedDeviceObject(FileObject);
//     BaseDeviceObject = IoGetBaseFileSystemDeviceObject(FileObject);
//     FastDispatch = DeviceObject->DriverObject->FastIoDispatch;
//
//     /* Get master FsRtl lock */
//     FsRtlEnterFileSystem();
//
//     /* Check if Fast Calls are supported, and check AcquireForCcFlush */
//     if (FastDispatch &&
//         FastDispatch->AcquireForCcFlush)
//     {
//         /* Call the AcquireForCcFlush FastIo handler */
//         Status = FastDispatch->AcquireForCcFlush(FileObject, BaseDeviceObject);
//
//         /* Return either success or inability to wait.
//            In case of other failure - fall through */
//         if (NT_SUCCESS(Status))
//             return Status;
//
//         if (Status == STATUS_CANT_WAIT)
//         {
//             DPRINT1("STATUS_CANT_WAIT\n");
//             FsRtlExitFileSystem();
//             return Status;
//         }
//     }
//
//     /* No FastIo handler (or it failed). Acquire Main resource */
//     if (FcbHeader->Resource)
//     {
//         /* Acquire it - either shared if it's already acquired
//             or exclusively if we are the first */
//         if (ExIsResourceAcquiredSharedLite(FcbHeader->Resource))
//             ExAcquireResourceSharedLite(FcbHeader->Resource, TRUE);
//         else
//             ExAcquireResourceExclusiveLite(FcbHeader->Resource, TRUE);
//     }
//
//     /* Also acquire its PagingIO resource */
//     if (FcbHeader->PagingIoResource)
//         ExAcquireResourceSharedLite(FcbHeader->PagingIoResource, TRUE);
//
//     return STATUS_SUCCESS;
// }
//
// /*
// * @implemented
// */
// VOID
// NTAPI
// FsRtlAcquireFileForCcFlush(IN PFILE_OBJECT FileObject)
// {
//     PAGED_CODE();
//
//     /* Call the common routine. Don't care about the result */
//     (VOID)FsRtlAcquireFileForCcFlushEx(FileObject);
// }
//
//
// /*
// * @implemented
// */
// VOID
// NTAPI
// FsRtlReleaseFileForCcFlush(IN PFILE_OBJECT FileObject)
// {
//     PFSRTL_COMMON_FCB_HEADER FcbHeader;
//     PDEVICE_OBJECT DeviceObject, BaseDeviceObject;
//     PFAST_IO_DISPATCH FastDispatch;
//     NTSTATUS Status = STATUS_INVALID_DEVICE_REQUEST;
//
//     /* Get Device Object and Fast Calls */
//     FcbHeader = (PFSRTL_COMMON_FCB_HEADER)FileObject->FsContext;
//     DeviceObject = IoGetRelatedDeviceObject(FileObject);
//     BaseDeviceObject = IoGetBaseFileSystemDeviceObject(FileObject);
//     FastDispatch = DeviceObject->DriverObject->FastIoDispatch;
//
//     /* Check if Fast Calls are supported, and check ReleaseForCcFlush */
//     if (FastDispatch &&
//         FastDispatch->ReleaseForCcFlush)
//     {
//         /* Call the ReleaseForCcFlush FastIo handler */
//         Status = FastDispatch->ReleaseForCcFlush(FileObject, BaseDeviceObject);
//     }
//
//     if (!NT_SUCCESS(Status))
//     {
//         /* No FastIo handler (or it failed). Release PagingIO resource and
//            then Main resource */
//         if (FcbHeader->PagingIoResource) ExReleaseResourceLite(FcbHeader->PagingIoResource);
//         if (FcbHeader->Resource) ExReleaseResourceLite(FcbHeader->Resource);
//     }
//
//     /* Release master FsRtl lock */
//     FsRtlExitFileSystem();
// }
//
// /**
//  * @brief Get the resource to acquire when Mod Writer flushes data to disk
//  *
//  * @param FcbHeader - FCB header from the file object
//  * @param EndingOffset - The end offset of the write to be done
//  * @param ResourceToAcquire - Pointer receiving the resource to acquire before doing the write
//  *
//  * @return BOOLEAN specifying whether the resource must be acquired exclusively
//  */
// static
// BOOLEAN
// FsRtlpGetResourceForModWrite(_In_ PFSRTL_COMMON_FCB_HEADER FcbHeader,
//                              _In_ PLARGE_INTEGER EndingOffset,
//                              _Outptr_result_maybenull_ PERESOURCE* ResourceToAcquire)
// {
//     /*
//      * Decide on type of locking and type of resource based on
//      *  - Flags
//      *  - Whether we're extending ValidDataLength
//      */
//     if (FlagOn(FcbHeader->Flags, FSRTL_FLAG_ACQUIRE_MAIN_RSRC_EX))
//     {
//         /* Acquire main resource, exclusive */
//         *ResourceToAcquire = FcbHeader->Resource;
//         return TRUE;
//     }
//
//     /* We will acquire shared. Which one ? */
//     if (FlagOn(FcbHeader->Flags, FSRTL_FLAG_ACQUIRE_MAIN_RSRC_SH))
//     {
//         *ResourceToAcquire = FcbHeader->Resource;
//     }
//     else
//     {
//         *ResourceToAcquire = FcbHeader->PagingIoResource;
//     }
//
//     /* We force exclusive lock if this write modifies the valid data length */
//     return (EndingOffset->QuadPart > FcbHeader->ValidDataLength.QuadPart);
// }
//
// /**
//  * @brief Lock a file object before flushing pages to disk. 
//  *        To be called by the Modified Page Writer (MPW)
//  *
//  * @param FileObject - The file object to lock
//  * @param EndingOffset - The end offset of the write to be done
//  * @param ResourceToRelease - Pointer receiving the resource to release after the write
//  *
//  * @return Relevant NTSTATUS value
//  */
// _Check_return_
// NTSTATUS
// NTAPI
// FsRtlAcquireFileForModWriteEx(_In_ PFILE_OBJECT FileObject,
//                               _In_ PLARGE_INTEGER EndingOffset,
//                               _Outptr_result_maybenull_ PERESOURCE *ResourceToRelease)
// {
//     PFSRTL_COMMON_FCB_HEADER FcbHeader;
//     PDEVICE_OBJECT DeviceObject, BaseDeviceObject;
//     PFAST_IO_DISPATCH FastDispatch;
//     PERESOURCE ResourceToAcquire = NULL;
//     BOOLEAN Exclusive;
//     BOOLEAN Result;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     /* Get Device Object and Fast Calls */
//     FcbHeader = (PFSRTL_COMMON_FCB_HEADER)FileObject->FsContext;
//     DeviceObject = IoGetRelatedDeviceObject(FileObject);
//     BaseDeviceObject = IoGetBaseFileSystemDeviceObject(FileObject);
//     FastDispatch = DeviceObject->DriverObject->FastIoDispatch;
//
//     /* Check if Fast Calls are supported, and check AcquireForModWrite */
//     if (FastDispatch &&
//         FastDispatch->AcquireForModWrite)
//     {
//         /* Call the AcquireForModWrite FastIo handler */
//         Status = FastDispatch->AcquireForModWrite(FileObject,
//                                                   EndingOffset,
//                                                   ResourceToRelease,
//                                                   BaseDeviceObject);
//
//         /* Return either success or inability to wait.
//            In case of other failure - fall through */
//         if (Status == STATUS_SUCCESS ||
//             Status == STATUS_CANT_WAIT)
//         {
//             return Status;
//         }
//     }
//
//     /* Check what and how we should acquire */
//     Exclusive = FsRtlpGetResourceForModWrite(FcbHeader, EndingOffset, &ResourceToAcquire);
//
//     /* Acquire the resource and loop until we're sure we got this right. */
//     while (TRUE)
//     {
//         BOOLEAN OldExclusive;
//         PERESOURCE OldResourceToAcquire;
//
//         if (ResourceToAcquire == NULL)
//         {
//             /* 
//              * There's nothing to acquire, we can simply return success
//              */
//
//             break;
//         }
//
//         if (Exclusive)
//         {
//             Result = ExAcquireResourceExclusiveLite(ResourceToAcquire, FALSE);
//         }
//         else
//         {
//             Result = ExAcquireSharedWaitForExclusive(ResourceToAcquire, FALSE);
//         }
//
//         if (!Result)
//         {
//             return STATUS_CANT_WAIT;
//         }
//
//         /* Does this still hold true? */
//         OldExclusive = Exclusive;
//         OldResourceToAcquire = ResourceToAcquire;
//         Exclusive = FsRtlpGetResourceForModWrite(FcbHeader, EndingOffset, &ResourceToAcquire);
//
//         if ((OldExclusive == Exclusive) && (OldResourceToAcquire == ResourceToAcquire))
//         {
//             /* We're good */
//             break;
//         }
//
//         /* Can we fix this situation? */
//         if ((OldResourceToAcquire == ResourceToAcquire) && !Exclusive)
//         {
//             /* We can easily do so */
//             ExConvertExclusiveToSharedLite(ResourceToAcquire);
//             break;
//         }
//
//         /* Things have changed since we acquired the lock. Start again */
//         ExReleaseResourceLite(OldResourceToAcquire);
//     }
//
//     /* If we're here, this means that we succeeded */
//     *ResourceToRelease = ResourceToAcquire;
//     return STATUS_SUCCESS;
// }
//
// /**
//  * @brief Unlock a file object after flushing pages to disk. 
//  *        To be called by the Modified Page Writer (MPW) after a succesful call to
//  *        FsRtlAcquireFileForModWriteEx
//  *
//  * @param FileObject - The file object to unlock
//  * @param ResourceToRelease - The resource to release
//  */
// VOID
// NTAPI
// FsRtlReleaseFileForModWrite(_In_ PFILE_OBJECT FileObject,
//                             _In_ PERESOURCE ResourceToRelease)
// {
//     PDEVICE_OBJECT DeviceObject, BaseDeviceObject;
//     PFAST_IO_DISPATCH FastDispatch;
//     NTSTATUS Status = STATUS_UNSUCCESSFUL;
//
//     /* Get Device Object and Fast Calls */
//     DeviceObject = IoGetRelatedDeviceObject(FileObject);
//     BaseDeviceObject = IoGetBaseFileSystemDeviceObject(FileObject);
//     FastDispatch = DeviceObject->DriverObject->FastIoDispatch;
//
//     /* Check if Fast Calls are supported and check ReleaseForModWrite */
//     if (FastDispatch &&
//         FastDispatch->ReleaseForModWrite)
//     {
//         /* Call the ReleaseForModWrite FastIo handler */
//         Status = FastDispatch->ReleaseForModWrite(FileObject,
//                                                   ResourceToRelease,
//                                                   BaseDeviceObject);
//     }
//
//     /* Just release the resource if previous op failed */
//     if (!NT_SUCCESS(Status))
//     {
//         ExReleaseResourceLite(ResourceToRelease);
//     }
// }
//
//
// /*++
//  * @name FsRtlRegisterFileSystemFilterCallbacks
//  * @unimplemented
//  *
//  * FILLME
//  *
//  * @param FilterDriverObject
//  *        FILLME
//  *
//  * @param Callbacks
//  *        FILLME
//  *
//  * @return None
//  *
//  * @remarks None
//  *
//  *--*/
// NTSTATUS
// NTAPI
// FsRtlRegisterFileSystemFilterCallbacks(
//     PDRIVER_OBJECT FilterDriverObject,
//     PFS_FILTER_CALLBACKS Callbacks)
// {
//     PFS_FILTER_CALLBACKS NewCallbacks;
//     PEXTENDED_DRIVER_EXTENSION DriverExtension;
//     PAGED_CODE();
//
//     /* Verify parameters */
//     if ((FilterDriverObject == NULL) || (Callbacks == NULL))
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Allocate a buffer for a copy of the callbacks */
//     NewCallbacks = ExAllocatePoolWithTag(NonPagedPool,
//                                          Callbacks->SizeOfFsFilterCallbacks,
//                                          'gmSF');
//     if (NewCallbacks == NULL)
//     {
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* Copy the callbacks */
//     RtlCopyMemory(NewCallbacks, Callbacks, Callbacks->SizeOfFsFilterCallbacks);
//
//     /* Set the callbacks in the driver extension */
//     DriverExtension = (PEXTENDED_DRIVER_EXTENSION)FilterDriverObject->DriverExtension;
//     DriverExtension->FsFilterCallbacks = NewCallbacks;
//
//     return STATUS_SUCCESS;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
