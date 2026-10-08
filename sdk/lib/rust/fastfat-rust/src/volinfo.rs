use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{FILE_FS_ATTRIBUTE_INFORMATION, FILE_FS_DEVICE_INFORMATION, FILE_FS_FULL_SIZE_INFORMATION, FILE_FS_SIZE_INFORMATION, FILE_FS_VOLUME_INFORMATION, IoGetCurrentIrpStackLocation, IoSetTopLevelIrp, OEM_STRING, PFILE_FS_ATTRIBUTE_INFORMATION, PFILE_FS_DEVICE_INFORMATION, PFILE_FS_FULL_SIZE_INFORMATION, PFILE_FS_LABEL_INFORMATION, PFILE_FS_SIZE_INFORMATION, PFILE_FS_VOLUME_INFORMATION, PIO_STACK_LOCATION, PIRP, RtlAreBitsSet, RtlOemStringToCountedUnicodeString, RtlUpcaseUnicodeStringToCountedOemString};
// FS information class enum values (from _FSINFOCLASS_* bindgen consts)
use ntkrnl_sys::ddk::{_FSINFOCLASS_FileFsLabelInformation as FileFsLabelInformation};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{CCB, DIRENT, FAT_DATA, FCB, PCCB, PDCB, PFCB, PIRP_CONTEXT, PVCB, PVOLUME_DEVICE_OBJECT, VCB, _TYPE_OF_OPEN as TYPE_OF_OPEN};
use ntkrnl_sys::ddk::{FS_INFORMATION_CLASS, IO_STACK_LOCATION, PFILE_OBJECT, _PUBLIC_BCB as BCB};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{DEBUG_TRACE_VOLINFO};
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{IRP_CONTEXT};
// resolve_imports.py: explicit kernel/private-type imports
use crate::ddk_shims::{FsRtlEnterFileSystem, FsRtlExitFileSystem};
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{DCB, PBCB};
use ntkrnl_sys::ddk::{RTL_BITMAP};

// FILE_FS_SECTOR_SIZE_INFORMATION — Win8+ type not in ReactOS headers; define locally.
// Layout matches WDK: ULONG fields, 6 fields total = 24 bytes.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct FatFileFsSectorSizeInformation {
    pub LogicalBytesPerSector: u32,
    pub PhysicalBytesPerSectorForAtomicity: u32,
    pub PhysicalBytesPerSectorForPerformance: u32,
    pub FileSystemEffectivePhysicalBytesPerSectorForAtomicity: u32,
    pub Flags: u32,
    pub ByteOffsetForSectorAlignment: u32,
    pub ByteOffsetForPartitionAlignment: u32,
}

extern "system" {
    static mut FatData: FAT_DATA;
}
extern "system" {
    } extern "C" { fn DebugTrace(level: u32, component: u32, fmt: *const i8, ...); } extern "system" {
    fn UNREFERENCED_PARAMETER(_: *mut core::ffi::c_void);
    fn FlagOn(flags: u32, flag: u32) -> u32;
    fn SetFlag(attr: *mut u32, flag: u32);
    fn FatIsFat32(Vcb: *mut VCB) -> u8;
    fn DebugUnwind(name: *const i8);
    fn FatConstructLabelDirent( IrpContext: PIRP_CONTEXT, Dirent: *mut DIRENT, OemLabel: *mut OEM_STRING, );
    fn FatCreateNewDirent(IrpContext: PIRP_CONTEXT, RootDcb: *mut DCB, /* placeholder */ flag: i32, flag2: u32) -> ULONG;
    fn FatLocateVolumeLabel( IrpContext: PIRP_CONTEXT, Vcb: PVCB, Dirent: *mut *mut DIRENT, DirentBcb: *mut *mut BCB, ByteOffset: *mut ULONG, );
    fn FatPrepareWriteDirectoryFile( IrpContext: PIRP_CONTEXT, RootDcb: *mut DCB, ByteOffset: ULONG, size: ULONG, DirentBcb: *mut *mut BCB, Dirent: *mut *mut DIRENT, flag1: BOOLEAN, flag2: BOOLEAN, Status: *mut NTSTATUS, );
    fn FatSetDirtyBcb( IrpContext: PIRP_CONTEXT, Bcb: *mut BCB, Vcb: PVCB, dirty: BOOLEAN, );
    fn FatUnpinBcb( IrpContext: PIRP_CONTEXT, Bcb: *mut *mut core::ffi::c_void);
    // cachesup.rs defines this as `*mut IRP_CONTEXT` (== PIRP_CONTEXT). The extra `*`
    // here made the caller pass &PIRP_CONTEXT, which the callee then read as the
    // struct itself. rustc never compares a decl against a def in another module.
    fn FatUnpinRepinnedBcbs(IrpContext: PIRP_CONTEXT);
    fn FsRtlIsAnsiCharacterLegalFat( ch: UCHAR, second: BOOLEAN, ) -> BOOLEAN;
    fn FsRtlIsLeadDbcsCharacter(ch: UCHAR) -> BOOLEAN;
    fn NT_ASSERT(expr: BOOLEAN);
    fn _Analysis_assume_(condition: i32);
    fn ASSERT(condition: i32);
    fn FsRtlGetSectorSizeInformation( DeviceObject: *mut core::ffi::c_void, Buffer: *mut FatFileFsSectorSizeInformation, ) -> i32;
    fn FatAcquireExclusiveVcb(IrpContext: PIRP_CONTEXT, Vcb: PVCB) -> BOOLEAN; // fn FatCompleteRequest(IrpContext: PIRP_CONTEXT, Irp: PIRP, Status: NTSTATUS);
    fn FatDecodeFileObject(FileObject: PFILE_OBJECT, Vcb: *mut PVCB, Fcb: *mut PFCB, Ccb: *mut PCCB) -> TYPE_OF_OPEN;
    fn FatFsdPostRequest(IrpContext: PIRP_CONTEXT, Irp: PIRP) -> NTSTATUS;
    fn FatReleaseVcb(IrpContext: PIRP_CONTEXT, Vcb: PVCB);
    fn FatVerifyFcb(IrpContext: PIRP_CONTEXT, Fcb: PDCB);
    fn FatAcquireSharedVcb(IrpContext: PIRP_CONTEXT, Vcb: *mut VCB) -> i32;
    fn FatCompleteRequest(IrpContext: PIRP_CONTEXT, Irp: PIRP, Status: NTSTATUS);
    fn CanFsdWait(Irp: PIRP) -> BOOLEAN;
    fn FatCreateIrpContext(Irp: PIRP, canWait: u32) -> PIRP_CONTEXT;
    fn FatIsIrpTopLevel(Irp: PIRP) -> BOOLEAN;
    fn FatProcessException(IrpContext: PIRP_CONTEXT, Irp: PIRP, ExceptionCode: i32) -> NTSTATUS;
    static Dbg: i32;
}

// ---- file-scope preamble (translated) ----
// ---- re-stitched from bank: 20260612-143211-volinfo.c/01_FatQueryFsVolumeInfo.rs ----
#[no_mangle]
pub unsafe extern "system" fn FatQueryFsVolumeInfo(
    IrpContext: *mut PIRP_CONTEXT,
    Vcb: *mut PVCB,
    Buffer: *mut FILE_FS_VOLUME_INFORMATION,
    Length: *mut u32,
) -> NTSTATUS {
    // PAGED_CODE();

    // DebugTrace forward declaration (caller deduplicates)
    extern "system" {
        } extern "C" { fn DebugTrace(level: u32, component: u32, fmt: *const i8, ...); } extern "system" {
    }

    DebugTrace(
        0,
        0, // Dbg (DEBUG_TRACE_VOLINFO) – exact value unknown, using placeholder
        b"FatQueryFsVolumeInfo...\n\0".as_ptr() as *const i8,
        0,
    );

    let mut BytesToCopy: u32;
    let mut Status: NTSTATUS;

    // RtlZeroMemory(Buffer, sizeof(FILE_FS_VOLUME_INFORMATION));
    core::ptr::write_bytes(
        Buffer as *mut u8,
        0,
        core::mem::size_of::<FILE_FS_VOLUME_INFORMATION>(),
    );

    // Buffer->VolumeSerialNumber = Vcb->Vpb->SerialNumber;
    (*Buffer).VolumeSerialNumber = (*(**Vcb).Vpb).SerialNumber;

    // Buffer->SupportsObjects = FALSE;
    (*Buffer).SupportsObjects = 0; // FALSE

    // *Length -= FIELD_OFFSET(FILE_FS_VOLUME_INFORMATION, VolumeLabel[0]);
    *Length -= core::mem::offset_of!(FILE_FS_VOLUME_INFORMATION, VolumeLabel) as u32;

    let volume_label_length = (*(**Vcb).Vpb).VolumeLabelLength;

    // Check if the buffer is long enough
    if (*Length) >= (volume_label_length as u32) {
        BytesToCopy = volume_label_length as u32;
        Status = STATUS_SUCCESS;
    } else {
        BytesToCopy = *Length;
        Status = STATUS_BUFFER_OVERFLOW;
    }

    // Buffer->VolumeLabelLength = Vcb->Vpb->VolumeLabelLength;
    (*Buffer).VolumeLabelLength = volume_label_length as u32;

    // RtlCopyMemory(&Buffer->VolumeLabel[0], &Vcb->Vpb->VolumeLabel[0], BytesToCopy);
    let src = core::ptr::addr_of!((*(**Vcb).Vpb).VolumeLabel[0]) as *const u16 as *const u8;
    let dst = core::ptr::addr_of_mut!((*Buffer).VolumeLabel[0]) as *mut u16 as *mut u8;
    core::ptr::copy_nonoverlapping(src, dst, BytesToCopy as usize);

    // *Length -= BytesToCopy;
    *Length -= BytesToCopy;

    // UNREFERENCED_PARAMETER(IrpContext);
    let _ = IrpContext;

    Status
}
// ---- re-stitched from bank: 20260612-143211-volinfo.c/02_FatQueryFsSizeInfo.rs ----

unsafe extern "system" fn FatQueryFsSizeInfo(
    IrpContext: *mut PIRP_CONTEXT,
    Vcb: *mut PVCB,
    Buffer: *mut FILE_FS_SIZE_INFORMATION,
    Length: *mut u32,
) -> NTSTATUS {
    PAGED_CODE!();

    DebugTrace(0, /* Dbg (DEBUG_TRACE_VOLINFO) */ 0, b"FatQueryFsSizeInfo...\n\0".as_ptr() as *const i8, 0);

    core::ptr::write_bytes(Buffer as *mut u8, 0, core::mem::size_of::<FILE_FS_SIZE_INFORMATION>());

    (*Buffer).TotalAllocationUnits.__bindgen_anon_1.LowPart = (**Vcb).AllocationSupport.NumberOfClusters;
    (*Buffer).AvailableAllocationUnits.__bindgen_anon_1.LowPart = (**Vcb).AllocationSupport.NumberOfFreeClusters;
    (*Buffer).SectorsPerAllocationUnit = (**Vcb).Bpb.SectorsPerCluster as u32;
    (*Buffer).BytesPerSector = (**Vcb).Bpb.BytesPerSector as u32;

    *Length = (*Length).wrapping_sub(core::mem::size_of::<FILE_FS_SIZE_INFORMATION>() as u32);

    UNREFERENCED_PARAMETER(IrpContext as *mut core::ffi::c_void);

    STATUS_SUCCESS as i32
}
// ---- re-stitched from bank: 20260612-143211-volinfo.c/03_FatQueryFsDeviceInfo.rs ----

const FILE_DEVICE_DISK: u32 = 0x0000_0007;

#[no_mangle]
pub unsafe extern "system" fn FatQueryFsDeviceInfo(
    IrpContext: *mut IRP_CONTEXT,
    Vcb: *mut VCB,
    Buffer: *mut ntkrnl_sys::ddk::FILE_FS_DEVICE_INFORMATION,
    Length: *mut u32,
) -> i32 {
    // PAGED_CODE(); – macro expanded as no‑op

    unsafe {
        DebugTrace(0, 0 /* Dbg */, b"FatQueryFsDeviceInfo...\n\0".as_ptr() as *const i8, 0);
        core::ptr::write_bytes(
            Buffer as *mut u8,
            0,
            core::mem::size_of::<ntkrnl_sys::ddk::FILE_FS_DEVICE_INFORMATION>(),
        );

        (*Buffer).DeviceType = FILE_DEVICE_DISK;

        let target_device_object = (*Vcb).TargetDeviceObject;
        (*Buffer).Characteristics = (*target_device_object).Characteristics;

        *Length -= core::mem::size_of::<ntkrnl_sys::ddk::FILE_FS_DEVICE_INFORMATION>() as u32;

        let _ = IrpContext;   // UNREFERENCED_PARAMETER(IrpContext)

        ntkrnl_sys::base::STATUS_SUCCESS
    }
}
// ---- re-stitched from bank: 20260612-143211-volinfo.c/04_FatQueryFsAttributeInfo.rs ----


// (FatData is the real global FAT_DATA; ChicagoMode is a packed bitfield read
//  via FatData.ChicagoMode() — see the dirsup.rs note. The old FatDataStruct
//  stub read offset 0 (NodeTypeCode low byte == 0) and wrongly reported 8.3.)

#[no_mangle]
pub unsafe extern "system" fn FatQueryFsAttributeInfo(
    IrpContext: *mut IRP_CONTEXT,
    Vcb: *mut VCB,
    Buffer: *mut FILE_FS_ATTRIBUTE_INFORMATION,
    Length: *mut u32,
) -> NTSTATUS {
    const DEBUG_TRACE_VOLINFO: u32 = 0x2000; // unknown placeholder
    const Dbg: u32 = DEBUG_TRACE_VOLINFO;
    const FILE_CASE_PRESERVED_NAMES: u32 = 0x00000002;
    const FILE_UNICODE_ON_DISK: u32 = 0x00000004;
    const FILE_READ_ONLY_VOLUME: u32 = 0x00080000;
    const VCB_STATE_FLAG_WRITE_PROTECTED: u32 = 0x4000; // assumed
    const FAT32_BYTES: [u8; 10] = [0x46, 0x00, 0x41, 0x00, 0x54, 0x00, 0x33, 0x00, 0x32, 0x00];
    const FAT_BYTES: [u8; 6] = [0x46, 0x00, 0x41, 0x00, 0x54, 0x00];

    let mut BytesToCopy: u32;
    let Status: NTSTATUS;

    PAGED_CODE!();

    DebugTrace(0, Dbg, b"FatQueryFsAttributeInfo...\n\0".as_ptr() as *const i8, 0);

    (*Buffer).FileSystemAttributes = FILE_CASE_PRESERVED_NAMES | FILE_UNICODE_ON_DISK;

    if FlagOn((*Vcb).VcbState, VCB_STATE_FLAG_WRITE_PROTECTED) != 0 {
        SetFlag(
            &mut (*Buffer).FileSystemAttributes as *mut u32,
            FILE_READ_ONLY_VOLUME,
        );
    }

    (*Buffer).MaximumComponentNameLength = if (*&raw const FatData).ChicagoMode() != 0 { 255 } else { 12 };

    if FatIsFat32(Vcb) != 0 {
        let offset_field = core::mem::offset_of!(FILE_FS_ATTRIBUTE_INFORMATION, FileSystemName) as u32;
        let len_minus_offset = (*Length).wrapping_sub(offset_field);
        if len_minus_offset >= 10 {
            BytesToCopy = 10;
            *Length = (*Length).wrapping_sub(offset_field + 10);
            Status = STATUS_SUCCESS;
        } else {
            BytesToCopy = (*Length) - offset_field;
            *Length = 0;
            Status = STATUS_BUFFER_OVERFLOW;
        }
        RtlCopyMemory(
            &mut (*Buffer).FileSystemName[0] as *mut _ as *mut core::ffi::c_void,
            FAT32_BYTES.as_ptr() as *const core::ffi::c_void,
            BytesToCopy as usize,
        );
    } else {
        let offset_field = core::mem::offset_of!(FILE_FS_ATTRIBUTE_INFORMATION, FileSystemName) as u32;
        let len_minus_offset = (*Length).wrapping_sub(offset_field);
        if len_minus_offset >= 6 {
            BytesToCopy = 6;
            *Length = (*Length).wrapping_sub(offset_field + 6);
            Status = STATUS_SUCCESS;
        } else {
            BytesToCopy = (*Length) - offset_field;
            *Length = 0;
            Status = STATUS_BUFFER_OVERFLOW;
        }
        RtlCopyMemory(
            &mut (*Buffer).FileSystemName[0] as *mut _ as *mut core::ffi::c_void,
            FAT_BYTES.as_ptr() as *const core::ffi::c_void,
            BytesToCopy as usize,
        );
    }

    (*Buffer).FileSystemNameLength = BytesToCopy;

    let _ = IrpContext;
    let _ = Vcb;

    Status
}

// ---- re-stitched from bank: 20260612-143211-volinfo.c/05_FatQueryFsFullSizeInfo.rs ----

unsafe extern "system" fn FatQueryFsFullSizeInfo(
    IrpContext: *mut IRP_CONTEXT,
    Vcb: *mut VCB,
    Buffer: *mut FILE_FS_FULL_SIZE_INFORMATION,
    Length: *mut u32,
) -> i32 {
    // PAGED_CODE();

    let fmt = b"FatQueryFsSizeInfo...\n\0".as_ptr() as *const i8;
    DebugTrace(0, (Dbg) as u32, fmt, 0);

    RtlZeroMemory(
        Buffer as *mut core::ffi::c_void,
        core::mem::size_of::<FILE_FS_FULL_SIZE_INFORMATION>(),
    );

    (*Buffer).TotalAllocationUnits.__bindgen_anon_1.LowPart = (*Vcb).AllocationSupport.NumberOfClusters;
    (*Buffer).CallerAvailableAllocationUnits.__bindgen_anon_1.LowPart =
        (*Vcb).AllocationSupport.NumberOfFreeClusters;
    (*Buffer).ActualAvailableAllocationUnits.__bindgen_anon_1.LowPart =
        (*Buffer).CallerAvailableAllocationUnits.__bindgen_anon_1.LowPart;
    (*Buffer).SectorsPerAllocationUnit = (*Vcb).Bpb.SectorsPerCluster as u32;
    (*Buffer).BytesPerSector = (*Vcb).Bpb.BytesPerSector as u32;

    *Length -= core::mem::size_of::<FILE_FS_FULL_SIZE_INFORMATION>() as u32;

    let _ = IrpContext;

    STATUS_SUCCESS
}

// ---- re-stitched from bank: 20260612-143211-volinfo.c/06_FatSetFsLabelInfo.rs ----
#[no_mangle]
pub unsafe extern "system" fn FatSetFsLabelInfo(
    IrpContext: PIRP_CONTEXT,
    Vcb: PVCB,
    Buffer: PFILE_FS_LABEL_INFORMATION,
) -> NTSTATUS {
    // Unknown externs
    extern "system" {
        } extern "C" { fn DebugTrace(arg1: i32, arg2: u32, fmt: *const i8, ...); } extern "system" {
        fn DebugUnwind(name: usize);
        fn FatConstructLabelDirent(
            IrpContext: *mut IRP_CONTEXT,
            Dirent: *mut DIRENT,
            OemLabel: *mut OEM_STRING,
        );
        fn FatCreateNewDirent(IrpContext: *mut IRP_CONTEXT, RootDcb: *mut DCB, flag: i32, flag2: u32) -> u32;
        fn FatLocateVolumeLabel(
            IrpContext: *mut IRP_CONTEXT,
            Vcb: *mut VCB,
            Dirent: *mut *mut DIRENT,
            DirentBcb: *mut *mut BCB,
            ByteOffset: *mut u32,
        );
        fn FatPrepareWriteDirectoryFile(
            IrpContext: *mut IRP_CONTEXT,
            RootDcb: *mut DCB,
            ByteOffset: u32,
            size: u32,
            DirentBcb: *mut *mut BCB,
            Dirent: *mut *mut DIRENT,
            flag1: u8,
            flag2: u8,
            Status: *mut i32,
        );
        fn FatSetDirtyBcb(
            IrpContext: *mut IRP_CONTEXT,
            Bcb: *mut BCB,
            Vcb: *mut VCB,
            dirty: u8,
        );
        fn FatUnpinBcb(
            IrpContext: *mut IRP_CONTEXT, Bcb: *mut *mut core::ffi::c_void);
        fn FatUnpinRepinnedBcbs(IrpContext: PIRP_CONTEXT);
        fn FsRtlIsAnsiCharacterLegalFat(
            ch: UCHAR,
            second: BOOLEAN,
        ) -> BOOLEAN;
        fn FsRtlIsLeadDbcsCharacter(ch: UCHAR) -> BOOLEAN;
        fn NT_ASSERT(expr: i32);
        fn SetFlag(flags: *mut u8, flag: u8);
    }

    // Unknown constants
    const DEBUG_TRACE_VOLINFO: u32 = 0x2000; // placeholder
    const STATUS_INVALID_VOLUME_LABEL: NTSTATUS = 0xC0000086_u32 as i32; // approximate
    const MAXUSHORT: USHORT = 0xFFFF;
    const FAT_DIRENT_REALLY_0E5: UCHAR = 0x05;
    const FAT_DIRENT_DELETED: UCHAR = 0xE5;
    const IRP_CONTEXT_FLAG_WRITE_THROUGH: ULONG = 0x04; // placeholder

    // Local variables
    let mut Status: NTSTATUS;
    let mut Dirent: *mut DIRENT = core::ptr::null_mut();
    let mut DirentBcb: *mut BCB = core::ptr::null_mut();
    let mut ByteOffset: ULONG = 0;
    let mut TmpBuffer: [WCHAR; 11] = [0; 11];
    let mut OemBuffer: [UCHAR; 11] = [0; 11];
    let mut OemLabel: OEM_STRING = unsafe { core::mem::zeroed() };
    let mut UnicodeString: UNICODE_STRING = unsafe { core::mem::zeroed() };
    let mut UpcasedLabel: UNICODE_STRING = unsafe { core::mem::zeroed() };

    // PAGED_CODE(); omitted

    DebugTrace(1, DEBUG_TRACE_VOLINFO, b"FatSetFsLabelInfo...\n\0".as_ptr() as *const i8, 0);

    // Setup local variables
    let buffer = &mut *Buffer;
    UnicodeString.Length = buffer.VolumeLabelLength as USHORT;
    UnicodeString.MaximumLength = UnicodeString.Length;
    UnicodeString.Buffer = (&mut buffer.VolumeLabel[0]) as *mut WCHAR;

    // Name length check
    if UnicodeString.Length > 11 * core::mem::size_of::<WCHAR>() as USHORT {
        return STATUS_INVALID_VOLUME_LABEL;
    }

    // Upcase and convert to OEM
    OemLabel.Buffer = OemBuffer.as_mut_ptr() as *mut i8;
    OemLabel.Length = 0;
    OemLabel.MaximumLength = 11;

    Status = RtlUpcaseUnicodeStringToCountedOemString(
        &mut OemLabel as *mut OEM_STRING,
        &UnicodeString as *const UNICODE_STRING as *const _,
        FALSE,
    );

    if !NT_SUCCESS(Status) {
        DebugTrace(
            -1,
            DEBUG_TRACE_VOLINFO,
            b"FatSetFsLabelInfo:  Label must be too long. %08lx\n\0".as_ptr() as *const i8,
            Status,
        );
        return STATUS_INVALID_VOLUME_LABEL;
    }

    // Strip spaces
    if OemLabel.Length > 0 {
        let mut i: USHORT = 0;
        let mut LastSpaceIndex: USHORT = MAXUSHORT;

        while (i as u32) < (OemLabel.Length as u32) {
            let c = *OemLabel.Buffer.add(i as usize) as UCHAR;

            if FsRtlIsLeadDbcsCharacter(c) != 0 {
                LastSpaceIndex = MAXUSHORT;
                i += 1;
                i += 1; // skip next byte (i already incremented separately)
                continue;
            }

            if FsRtlIsAnsiCharacterLegalFat(c, FALSE) == 0
                || c == '.' as UCHAR
            {
                return STATUS_INVALID_VOLUME_LABEL;
            }

            if c == ' ' as UCHAR && LastSpaceIndex == MAXUSHORT {
                LastSpaceIndex = i;
            } else {
                LastSpaceIndex = MAXUSHORT;
            }

            i += 1;
        }

        if LastSpaceIndex != MAXUSHORT {
            OemLabel.Length = LastSpaceIndex;
        }
    }

    // Get Unicode upcased string for VPB
    UpcasedLabel.Length = UnicodeString.Length;
    UpcasedLabel.MaximumLength = 11 * core::mem::size_of::<WCHAR>() as USHORT;
    UpcasedLabel.Buffer = TmpBuffer.as_mut_ptr();

    Status = RtlOemStringToCountedUnicodeString(
        &mut UpcasedLabel as *mut UNICODE_STRING as *mut _,
        &OemLabel as *const OEM_STRING,
        FALSE,
    );

    if !NT_SUCCESS(Status) {
        DebugTrace(
            -1,
            DEBUG_TRACE_VOLINFO,
            b"FatSetFsLabelInfo:  Label must be too long. %08lx\n\0".as_ptr() as *const i8,
            Status,
        );
        return STATUS_INVALID_VOLUME_LABEL;
    }

    DirentBcb = core::ptr::null_mut();

    // Mark write through
    SetFlag(
        &mut (*IrpContext).Flags as *mut ULONG as *mut u8,
        IRP_CONTEXT_FLAG_WRITE_THROUGH as u8,
    );

    // _SEH2_TRY / _SEH2_FINALLY
    let p_dirent_bcb = core::ptr::addr_of_mut!(DirentBcb);
    crate::seh::try_finally(&mut (), |_| {
        {
            if OemLabel.Length > 0 {
                // Handle first character 0xe5 -> 0x05
                if *OemLabel.Buffer as UCHAR == 0xe5 {
                    *OemLabel.Buffer = (FAT_DIRENT_REALLY_0E5) as i8;
                }

                // Locate existing volume label
                FatLocateVolumeLabel(
                    IrpContext,
                    Vcb,
                    &mut Dirent,
                    &mut *p_dirent_bcb,
                    &mut ByteOffset as *mut ULONG as *mut u32,
                );

                if Dirent.is_null() {
                    ByteOffset = FatCreateNewDirent(
                        IrpContext,
                        (*Vcb).RootDcb as *mut DCB,
                        1,
                        (FALSE as u8) as _,
                    );

                    FatPrepareWriteDirectoryFile(
                        IrpContext,
                        (*Vcb).RootDcb as *mut DCB,
                        ByteOffset,
                        core::mem::size_of::<DIRENT>() as u32,
                        &mut *p_dirent_bcb,
                        &mut Dirent,
                        FALSE as u8,
                        TRUE as u8,
                        &mut Status as *mut NTSTATUS as *mut i32,
                    );

                    NT_ASSERT(NT_SUCCESS(Status) as i32);
                } else {
                    FatSetDirtyBcb(IrpContext, *p_dirent_bcb, Vcb, TRUE as u8);
                }

                // Construct label dirent
                FatConstructLabelDirent(
                    IrpContext,
                    Dirent,
                    &mut OemLabel,
                );

                // Unpin to catch IO errors before VPB change
                FatUnpinBcb(IrpContext, p_dirent_bcb as *mut *mut core::ffi::c_void);
                FatUnpinRepinnedBcbs(IrpContext);

                // Set upcased label in VPB
                core::ptr::copy_nonoverlapping::<WCHAR>(
                    UpcasedLabel.Buffer,
                    (*(*Vcb).Vpb).VolumeLabel.as_mut_ptr(),
                    UpcasedLabel.Length as usize / core::mem::size_of::<WCHAR>(),
                );
                (*(*Vcb).Vpb).VolumeLabelLength = (UpcasedLabel.Length as ULONG) as u16;
            } else {
                // Delete label
                FatLocateVolumeLabel(
                    IrpContext,
                    Vcb,
                    &mut Dirent,
                    &mut *p_dirent_bcb,
                    &mut ByteOffset as *mut ULONG as *mut u32,
                );

                if Dirent.is_null() {
                    Status = STATUS_SUCCESS;
                    return; // try_return
                }

                // Mark directory entry as deleted
                (*Dirent).FileName[0] = FAT_DIRENT_DELETED;

                // Assert free dirent bitmap state
                let root_dcb = (*Vcb).RootDcb;
                let condition = (*root_dcb).Specific.Dcb.as_ref().UnusedDirentVbo == 0xffffffff_u32
                    || RtlAreBitsSet(
                        &mut (*root_dcb).Specific.Dcb.as_mut().FreeDirentBitmap as *mut _ as *mut RTL_BITMAP,
                        (ByteOffset / core::mem::size_of::<DIRENT>() as ULONG) as u32,
                        1u32,
                    ) != 0;

                NT_ASSERT(condition as i32);

                // Clear bit in free dirent bitmap
                RtlClearBits(
                    &mut (*root_dcb).Specific.Dcb.as_mut().FreeDirentBitmap as *mut _ as *mut _,
                    (ByteOffset / core::mem::size_of::<DIRENT>() as ULONG) as u32,
                    1u32,
                );

                FatSetDirtyBcb(IrpContext, *p_dirent_bcb, Vcb, TRUE as u8);

                // Unpin
                FatUnpinBcb(IrpContext, p_dirent_bcb as *mut *mut core::ffi::c_void);
                FatUnpinRepinnedBcbs(IrpContext);

                // Clear label in VPB
                (*(*Vcb).Vpb).VolumeLabelLength = 0;
            }

            Status = STATUS_SUCCESS;
        }
    }, |_, _abnormal| {
        DebugUnwind(b"FatSetFsALabelInfo\0".as_ptr() as usize);
        FatUnpinBcb(IrpContext, p_dirent_bcb as *mut *mut core::ffi::c_void);
        DebugTrace(
            -1,
            DEBUG_TRACE_VOLINFO,
            b"FatSetFsALabelInfo -> STATUS_SUCCESS\n\0".as_ptr() as *const i8,
            0,
        );
    });

    Status
}


// #if (NTDDI_VERSION >= NTDDI_WIN8)
// ---- re-stitched from bank: 20260612-143211-volinfo.c/07_FatQueryFsSectorSizeInfo.rs ----

#[no_mangle]
pub unsafe extern "system" fn FatQueryFsSectorSizeInfo(
    IrpContext: PIRP_CONTEXT,
    Vcb: PVCB,
    Buffer: *mut FatFileFsSectorSizeInformation,
    Length: *mut u32,
) -> i32 {
    let Status: i32;

    PAGED_CODE!();
    UNREFERENCED_PARAMETER(IrpContext as *mut core::ffi::c_void);

    ASSERT((*Length as usize >= core::mem::size_of::<FatFileFsSectorSizeInformation>()) as i32);
    // _Analysis_assume_ removed: SAL no-op annotation; argument must never be evaluated (0xB8 class)

    Status = FsRtlGetSectorSizeInformation(
        (*(*Vcb).Vpb).RealDevice as *mut core::ffi::c_void,
        Buffer,
    );

    if ntkrnl_sys::base::NT_SUCCESS(Status) {
        *Length -= core::mem::size_of::<FatFileFsSectorSizeInformation>() as u32;
    }

    return Status;
}

// #endif



// Forward declarations for unknown externs

// Constants (placeholders; actual values from the kernel)
const UserVolumeOpen: TYPE_OF_OPEN = 4; // FIXED 2026-07-05: was LLM-guessed "typical" value; verified vs C header (_TYPE_OF_OPEN_UserVolumeOpen=4; 2 is UserFileOpen!)

#[no_mangle]
pub unsafe extern "system" fn FatFsdSetVolumeInformation(
    VolumeDeviceObject: PVOLUME_DEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut Status: NTSTATUS = 0;
    // PIRP_CONTEXT, not `*mut PIRP_CONTEXT`. FatCreateIrpContext returns one
    // pointer; the local was typed as two, and the call below then wrote
    // `(*IrpContext)` — reading the first eight bytes of the IRP_CONTEXT struct
    // and passing THAT as the context, on every IRP_MJ_SET_VOLUME_INFORMATION.
    // It compiled because volinfo's own extern block declared
    // FatCreateIrpContext as returning the deeper type too, so the lie was
    // self-consistent and rustc had nothing to compare it against.
    let mut IrpContext: PIRP_CONTEXT = core::ptr::null_mut();
    let TopLevel: BOOLEAN;

    PAGED_CODE!();

    DebugTrace(1u32, Dbg as u32, b"FatFsdSetVolumeInformation\n\0".as_ptr() as *const i8, 0u32);

    FsRtlEnterFileSystem();
    TopLevel = FatIsIrpTopLevel(Irp);

    // _SEH2_TRY { ... } _SEH2_EXCEPT(FatExceptionFilter(IrpContext, ...)) { FatProcessException }
    let p_status = core::ptr::addr_of_mut!(Status);
    let p_irp_context = core::ptr::addr_of_mut!(IrpContext);
    if let Err(code) = crate::seh::fat_try_except(p_irp_context as *const PIRP_CONTEXT, || {
        *p_irp_context = FatCreateIrpContext(Irp, (CanFsdWait(Irp)) as _);
        *p_status = FatCommonSetVolumeInfo((*p_irp_context).cast(), Irp);
    }) {
        Status = FatProcessException(IrpContext, Irp, code);
    }

    if TopLevel != 0 {
        IoSetTopLevelIrp(core::ptr::null_mut());
    }

    FsRtlExitFileSystem();

    DebugTrace((-1i32) as u32, Dbg as u32, b"FatFsdSetVolumeInformation -> %08lx\n\0".as_ptr() as *const i8, Status as u32);

    // UNREFERENCED_PARAMETER(VolumeDeviceObject);
    let _ = VolumeDeviceObject;

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn FatCommonQueryVolumeInfo(IrpContext: PIRP_CONTEXT, Irp: PIRP) -> NTSTATUS {
    let mut IrpContext = IrpContext;
    let mut Irp = Irp;
    let mut Status: NTSTATUS = 0; // STATUS_SUCCESS
    let IrpSp: *mut IO_STACK_LOCATION;
    let mut Vcb: *mut VCB = core::ptr::null_mut();
    let mut Fcb: *mut FCB = core::ptr::null_mut();
    let mut Ccb: *mut CCB = core::ptr::null_mut();
    let mut Length: u32 = 0;
    let FsInformationClass: u32;
    let Buffer: *mut core::ffi::c_void;
    let mut WeAcquiredVcb: i32 = 0;

    IrpSp = IoGetCurrentIrpStackLocation(Irp);

    DebugTrace(1, 0, b"FatCommonQueryVolumeInfo...\n\0".as_ptr() as *const i8, 0);
    DebugTrace(0, 0, b"Irp                  = %p\n\0".as_ptr() as *const i8, (Irp as isize) as u32);
    DebugTrace(0, 0, b"->Length             = %08lx\n\0".as_ptr() as *const i8, ((*IrpSp).Parameters.QueryVolume.Length as isize) as u32);
    DebugTrace(0, 0, b"->FsInformationClass = %08lx\n\0".as_ptr() as *const i8, ((*IrpSp).Parameters.QueryVolume.FsInformationClass as isize) as u32);
    DebugTrace(0, 0, b"->Buffer             = %p\n\0".as_ptr() as *const i8, ((*Irp).AssociatedIrp.SystemBuffer as isize) as u32);

    Length = (*IrpSp).Parameters.QueryVolume.Length;
    FsInformationClass = (*IrpSp).Parameters.QueryVolume.FsInformationClass;
    Buffer = (*Irp).AssociatedIrp.SystemBuffer;

    FatDecodeFileObject((*IrpSp).FileObject, core::ptr::addr_of_mut!(Vcb), core::ptr::addr_of_mut!(Fcb), core::ptr::addr_of_mut!(Ccb));

    NT_ASSERT(((Vcb as *mut core::ffi::c_void != core::ptr::null_mut()) as i32) as u8);
    // _Analysis_assume_ removed: SAL no-op annotation; argument must never be evaluated (0xB8 class)

    // _SEH2_TRY / _SEH2_FINALLY
    let p_irp_context = core::ptr::addr_of_mut!(IrpContext);
    let p_irp = core::ptr::addr_of_mut!(Irp);
    let p_status = core::ptr::addr_of_mut!(Status);
    let p_we_acquired_vcb = core::ptr::addr_of_mut!(WeAcquiredVcb);
    let p_length = core::ptr::addr_of_mut!(Length);
    let p_vcb = core::ptr::addr_of_mut!(Vcb);
    crate::seh::try_finally(&mut (), |_| {
    let IrpContext = *p_irp_context;
    let Irp = *p_irp;
    let Vcb = *p_vcb;
    FatVerifyFcb(IrpContext, (*Vcb).RootDcb);

    match FsInformationClass {
        1 => { // FileFsVolumeInformation
            if FatAcquireSharedVcb(IrpContext.cast(), Vcb) == 0 {
                DebugTrace(0, 0, b"Cannot acquire Vcb\n\0".as_ptr() as *const i8, 0);
                *p_status = FatFsdPostRequest(IrpContext, Irp);
                *p_irp_context = core::ptr::null_mut();
                *p_irp = core::ptr::null_mut();
            } else {
                *p_we_acquired_vcb = 1;
                *p_status = FatQueryFsVolumeInfo(p_irp_context, p_vcb, Buffer as *mut FILE_FS_VOLUME_INFORMATION, p_length);
            }
        },
        3 => { // FileFsSizeInformation
            *p_status = FatQueryFsSizeInfo(p_irp_context, p_vcb, Buffer as *mut FILE_FS_SIZE_INFORMATION, p_length);
        },
        4 => { // FileFsDeviceInformation
            *p_status = FatQueryFsDeviceInfo(IrpContext, Vcb, Buffer as *mut ntkrnl_sys::ddk::FILE_FS_DEVICE_INFORMATION, p_length);
        },
        5 => { // FileFsAttributeInformation
            *p_status = FatQueryFsAttributeInfo(IrpContext, Vcb, Buffer as *mut FILE_FS_ATTRIBUTE_INFORMATION, p_length);
        },
        7 => { // FileFsFullSizeInformation
            *p_status = FatQueryFsFullSizeInfo(IrpContext, Vcb, Buffer as *mut FILE_FS_FULL_SIZE_INFORMATION, p_length);
        },
        // FileFsSectorSizeInformation exists from NTDDI >= WIN8 (volinfo.c 395-400); pre-WIN8 it
        // is the default arm's STATUS_INVALID_PARAMETER.
        #[cfg(ntddi_ge_ntddi_win8)]
        11 => {
            *p_status = FatQueryFsSectorSizeInfo(IrpContext, Vcb, Buffer as *mut FatFileFsSectorSizeInformation, p_length);
        },
        _ => {
            *p_status = STATUS_INVALID_PARAMETER;
        }
    }

    if !(*p_irp).is_null() {
        (*(*p_irp)).IoStatus.Information = ((*IrpSp).Parameters.QueryVolume.Length - *p_length) as _;
    }
    }, |_, abnormal| {
    let IrpContext = *p_irp_context;
    let Irp = *p_irp;
    let Status = *p_status;
    DebugUnwind(b"FatCommonQueryVolumeInfo\0".as_ptr() as *const i8);

    if *p_we_acquired_vcb != 0 {
        FatReleaseVcb(IrpContext, *p_vcb);
    }

    if !abnormal {
        FatCompleteRequest(IrpContext, Irp, Status);
    }

    DebugTrace((-1i32) as u32, 0, b"FatCommonQueryVolumeInfo -> %08lx\n\0".as_ptr() as *const i8, (Status as isize) as u32);
    });

    Status
}

#[no_mangle]
pub unsafe extern "system" fn FatCommonSetVolumeInfo(
    IrpContext: PIRP_CONTEXT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut Irp = Irp;
    let mut IrpContext = IrpContext;
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let IrpSp: PIO_STACK_LOCATION;

    let mut Vcb: PVCB = core::ptr::null_mut();
    let mut Fcb: PFCB = core::ptr::null_mut();
    let mut Ccb: PCCB = core::ptr::null_mut();
    let TypeOfOpen: TYPE_OF_OPEN;

    let FsInformationClass: FS_INFORMATION_CLASS;
    let Buffer: *mut core::ffi::c_void;

    PAGED_CODE!();

    // Get the current stack location
    IrpSp = IoGetCurrentIrpStackLocation(Irp);

    DebugTrace(1, Dbg as u32, b"FatCommonSetVolumeInfo...\n\0".as_ptr() as *const i8, (0i32) as u32);
    DebugTrace(0, Dbg as u32, b"Irp                  = %p\n\0".as_ptr() as *const i8, Irp as usize as u32);
    DebugTrace(
        0,
        (Dbg) as u32,
        b"->Length             = %08lx\n\0".as_ptr() as *const i8,
        (*IrpSp).Parameters.SetVolume.Length as u32,
    );
    DebugTrace(
        0,
        (Dbg) as u32,
        b"->FsInformationClass = %08lx\n\0".as_ptr() as *const i8,
        (*IrpSp).Parameters.SetVolume.FsInformationClass as u32,
    );
    DebugTrace(
        0,
        Dbg as u32,
        b"->Buffer             = %p\n\0".as_ptr() as *const i8,
        (*Irp).AssociatedIrp.SystemBuffer as usize as u32,
    );

    // Reference our input parameters to make things easier
    FsInformationClass = (*IrpSp).Parameters.SetVolume.FsInformationClass;
    Buffer = (*Irp).AssociatedIrp.SystemBuffer;

    // Decode the file object to get the Vcb
    TypeOfOpen = FatDecodeFileObject(
        (*IrpSp).FileObject,
        &mut Vcb,
        &mut Fcb,
        &mut Ccb,
    );

    if TypeOfOpen != UserVolumeOpen {
        FatCompleteRequest(IrpContext, Irp, STATUS_ACCESS_DENIED);
        DebugTrace((-1i32) as u32, Dbg as u32, b"FatCommonSetVolumeInfo -> STATUS_ACCESS_DENIED\n\0".as_ptr() as *const i8, 0u32);
        return STATUS_ACCESS_DENIED;
    }

    // Acquire exclusive access to the Vcb and enqueue the Irp if we didn't get access
    if FatAcquireExclusiveVcb(IrpContext, Vcb) == 0 {
        DebugTrace(0, Dbg as u32, b"Cannot acquire Vcb\n\0".as_ptr() as *const i8, (0i32) as u32);
        Status = FatFsdPostRequest(IrpContext, Irp);
        DebugTrace((-1i32) as u32, (Dbg) as u32, b"FatCommonSetVolumeInfo -> %08lx\n\0".as_ptr() as *const i8, Status as u32);
        return Status;
    }

    // _SEH2_TRY / _SEH2_FINALLY
    let p_status = core::ptr::addr_of_mut!(Status);
    crate::seh::try_finally(&mut (), |_| {
        // Make sure the vcb is in a usable condition.  This will raise
        // an error condition if the volume is unusable
        // Also verify the Root Dcb since we need info from there.
        FatVerifyFcb(IrpContext, (*Vcb).RootDcb);

        // Based on the information class we'll do different actions.  Each
        // of the procedures that we're calling performs the action if
        // possible and returns true if it successful and false if it couldn't
        // wait for any I/O to complete.
        let buf_label: PFILE_FS_LABEL_INFORMATION = Buffer as PFILE_FS_LABEL_INFORMATION;
        match FsInformationClass {
            FileFsLabelInformation => {
                *p_status = FatSetFsLabelInfo(IrpContext, Vcb, buf_label);
            }
            _ => {
                *p_status = STATUS_INVALID_PARAMETER;
            }
        }

        FatUnpinRepinnedBcbs(IrpContext);
    }, |_, abnormal| {
        DebugUnwind(b"FatCommonSetVolumeInfo\0".as_ptr() as *const i8);
        FatReleaseVcb(IrpContext, Vcb);
        if !abnormal {
            FatCompleteRequest(IrpContext, Irp, *p_status);
        }
        DebugTrace(
            (-1i32) as u32,
            (Dbg) as u32,
            b"FatCommonSetVolumeInfo -> %08lx\n\0".as_ptr() as *const i8,
            *p_status as u32,
        );
    });

    return Status;
}



// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*++
//
// Copyright (c) 1989-2000 Microsoft Corporation
//
// Module Name:
//
//     VolInfo.c
//
// Abstract:
//
//     This module implements the volume information routines for Fat called by
//     the dispatch driver.
//
//
// --*/
//
// #include "fatprocs.h"
//
// //
// //  The local debug trace level
// //
//
// #define Dbg                              (DEBUG_TRACE_VOLINFO)
//
/* NTSTATUS */
// FatQueryFsVolumeInfo (
//     IN PIRP_CONTEXT IrpContext,
//     IN PVCB Vcb,
//     IN PFILE_FS_VOLUME_INFORMATION Buffer,
//     IN OUT PULONG Length
//     );
//
/* NTSTATUS */
// FatQueryFsSizeInfo (
//     IN PIRP_CONTEXT IrpContext,
//     IN PVCB Vcb,
//     IN PFILE_FS_SIZE_INFORMATION Buffer,
//     IN OUT PULONG Length
//     );
//
/* NTSTATUS */
// FatQueryFsDeviceInfo (
//     IN PIRP_CONTEXT IrpContext,
//     IN PVCB Vcb,
//     IN PFILE_FS_DEVICE_INFORMATION Buffer,
//     IN OUT PULONG Length
//     );
//
/* NTSTATUS */
// FatQueryFsAttributeInfo (
//     IN PIRP_CONTEXT IrpContext,
//     IN PVCB Vcb,
//     IN PFILE_FS_ATTRIBUTE_INFORMATION Buffer,
//     IN OUT PULONG Length
//     );
//
/* NTSTATUS */
// FatQueryFsFullSizeInfo (
//     IN PIRP_CONTEXT IrpContext,
//     IN PVCB Vcb,
//     IN PFILE_FS_FULL_SIZE_INFORMATION Buffer,
//     IN OUT PULONG Length
//     );
//
/* NTSTATUS */
// FatSetFsLabelInfo (
//     IN PIRP_CONTEXT IrpContext,
//     IN PVCB Vcb,
//     IN PFILE_FS_LABEL_INFORMATION Buffer
//     );
//
// #if (NTDDI_VERSION >= NTDDI_WIN8)
/* NTSTATUS */
// FatQueryFsSectorSizeInfo (
//      PIRP_CONTEXT IrpContext,
//      PVCB Vcb,
//      PFILE_FS_SECTOR_SIZE_INFORMATION Buffer,
//      PULONG Length
//     );
// #endif
//
// #ifdef ALLOC_PRAGMA
// #pragma alloc_text(PAGE, FatCommonQueryVolumeInfo)
// #pragma alloc_text(PAGE, FatCommonSetVolumeInfo)
// #pragma alloc_text(PAGE, FatFsdQueryVolumeInformation)
// #pragma alloc_text(PAGE, FatFsdSetVolumeInformation)
// #pragma alloc_text(PAGE, FatQueryFsAttributeInfo)
// #pragma alloc_text(PAGE, FatQueryFsDeviceInfo)
// #pragma alloc_text(PAGE, FatQueryFsSizeInfo)
// #pragma alloc_text(PAGE, FatQueryFsVolumeInfo)
// #pragma alloc_text(PAGE, FatQueryFsFullSizeInfo)
// #pragma alloc_text(PAGE, FatSetFsLabelInfo)
// #if (NTDDI_VERSION >= NTDDI_WIN8)
// #pragma alloc_text(PAGE, FatQueryFsSectorSizeInfo)
// #endif
// #endif
//
//
// _Function_class_(IRP_MJ_QUERY_VOLUME_INFORMATION)
// _Function_class_(DRIVER_DISPATCH)
/* NTSTATUS */
// NTAPI
// FatFsdQueryVolumeInformation (
//      PVOLUME_DEVICE_OBJECT VolumeDeviceObject,
//      PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This routine implements the Fsd part of the NtQueryVolumeInformation API
//     call.
//
// Arguments:
//
//     VolumeDeviceObject - Supplies the volume device object where the file
//         being queried exists.
//
//     Irp - Supplies the Irp being processed.
//
// Return Value:
//
//     NTSTATUS - The FSD status for the Irp.
//
// --*/
//
// {
//     NTSTATUS Status;
//     _SEH2_VOLATILE PIRP_CONTEXT IrpContext = NULL;
//
//     BOOLEAN TopLevel;
//
//     PAGED_CODE();
//
//     DebugTrace(+1, Dbg, "FatFsdQueryVolumeInformation\n", 0);
//
//     //
//     //  Call the common query routine, with blocking allowed if synchronous
//     //
//
//     FsRtlEnterFileSystem();
//
//     TopLevel = FatIsIrpTopLevel( Irp );
//
//     _SEH2_TRY {
//
//         IrpContext = FatCreateIrpContext( Irp, CanFsdWait( Irp ) );
//
//         Status = FatCommonQueryVolumeInfo( IrpContext, Irp );
//
//     } _SEH2_EXCEPT(FatExceptionFilter( IrpContext, _SEH2_GetExceptionInformation() )) {
//
//         //
//         //  We had some trouble trying to perform the requested
//         //  operation, so we'll abort the I/O request with
//         //  the error status that we get back from the
//         //  exception code
//         //
//
//         Status = FatProcessException( IrpContext, Irp, _SEH2_GetExceptionCode() );
//     } _SEH2_END;
//
//     if (TopLevel) { IoSetTopLevelIrp( NULL ); }
//
//     FsRtlExitFileSystem();
//
//     //
//     //  And return to our caller
//     //
//
//     DebugTrace(-1, Dbg, "FatFsdQueryVolumeInformation -> %08lx\n", Status);
//
//     UNREFERENCED_PARAMETER( VolumeDeviceObject );
//
//     return Status;
// }
//
//
// _Function_class_(IRP_MJ_SET_VOLUME_INFORMATION)
// _Function_class_(DRIVER_DISPATCH)
/* NTSTATUS */
// NTAPI
// FatFsdSetVolumeInformation (
//      PVOLUME_DEVICE_OBJECT VolumeDeviceObject,
//      PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This routine implements the FSD part of the NtSetVolumeInformation API
//     call.
//
// Arguments:
//
//     VolumeDeviceObject - Supplies the volume device object where the file
//         being set exists.
//
//     Irp - Supplies the Irp being processed.
//
// Return Value:
//
//     NTSTATUS - The FSD status for the Irp.
//
// --*/
//
// {
//     NTSTATUS Status;
//     _SEH2_VOLATILE PIRP_CONTEXT IrpContext = NULL;
//
//     BOOLEAN TopLevel;
//
//     PAGED_CODE();
//
//     DebugTrace(+1, Dbg, "FatFsdSetVolumeInformation\n", 0);
//
//     //
//     //  Call the common set routine
//     //
//
//     FsRtlEnterFileSystem();
//
//     TopLevel = FatIsIrpTopLevel( Irp );
//
//     _SEH2_TRY {
//
//         IrpContext = FatCreateIrpContext( Irp, CanFsdWait( Irp ) );
//
//         Status = FatCommonSetVolumeInfo( IrpContext, Irp );
//
//     } _SEH2_EXCEPT(FatExceptionFilter( IrpContext, _SEH2_GetExceptionInformation() )) {
//
//         //
//         //  We had some trouble trying to perform the requested
//         //  operation, so we'll abort the I/O request with
//         //  the error status that we get back from the
//         //  exception code
//         //
//
//         Status = FatProcessException( IrpContext, Irp, _SEH2_GetExceptionCode() );
//     } _SEH2_END;
//
//     if (TopLevel) { IoSetTopLevelIrp( NULL ); }
//
//     FsRtlExitFileSystem();
//
//     //
//     //  And return to our caller
//     //
//
//     DebugTrace(-1, Dbg, "FatFsdSetVolumeInformation -> %08lx\n", Status);
//
//     UNREFERENCED_PARAMETER( VolumeDeviceObject );
//
//     return Status;
// }
//
//
// _Requires_lock_held_(_Global_critical_region_)
/* NTSTATUS */
// FatCommonQueryVolumeInfo (
//     IN PIRP_CONTEXT IrpContext,
//     IN PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This is the common routine for querying volume information called by both
//     the fsd and fsp threads.
//
// Arguments:
//
//     Irp - Supplies the Irp being processed
//
// Return Value:
//
//     NTSTATUS - The return status for the operation
//
// --*/
//
// {
//     NTSTATUS Status = STATUS_SUCCESS;
//     PIO_STACK_LOCATION IrpSp;
//
//     PVCB Vcb;
//     PFCB Fcb;
//     PCCB Ccb;
//
//     ULONG Length;
//     FS_INFORMATION_CLASS FsInformationClass;
//     PVOID Buffer;
//
//     BOOLEAN WeAcquiredVcb = FALSE;
//
//     PAGED_CODE();
//
//     //
//     //  Get the current stack location
//     //
//
//     IrpSp = IoGetCurrentIrpStackLocation( Irp );
//
//     DebugTrace(+1, Dbg, "FatCommonQueryVolumeInfo...\n", 0);
//     DebugTrace( 0, Dbg, "Irp                  = %p\n", Irp );
//     DebugTrace( 0, Dbg, "->Length             = %08lx\n", IrpSp->Parameters.QueryVolume.Length);
//     DebugTrace( 0, Dbg, "->FsInformationClass = %08lx\n", IrpSp->Parameters.QueryVolume.FsInformationClass);
//     DebugTrace( 0, Dbg, "->Buffer             = %p\n", Irp->AssociatedIrp.SystemBuffer);
//
//     //
//     //  Reference our input parameters to make things easier
//     //
//
//     Length = IrpSp->Parameters.QueryVolume.Length;
//     FsInformationClass = IrpSp->Parameters.QueryVolume.FsInformationClass;
//     Buffer = Irp->AssociatedIrp.SystemBuffer;
//
//     //
//     //  Decode the file object to get the Vcb
//     //
//
//     (VOID) FatDecodeFileObject( IrpSp->FileObject, &Vcb, &Fcb, &Ccb );
//
//     NT_ASSERT( Vcb != NULL );
//     _Analysis_assume_( Vcb != NULL );
//
//     _SEH2_TRY {
//
//         //
//         //  Make sure the vcb is in a usable condition.  This will raise
//         //  an error condition if the volume is unusable
//         //
//         //  Also verify the Root Dcb since we need info from there.
//         //
//
//         FatVerifyFcb( IrpContext, Vcb->RootDcb );
//
//         //
//         //  Based on the information class we'll do different actions.  Each
//         //  of the procedures that we're calling fills up the output buffer
//         //  if possible and returns true if it successfully filled the buffer
//         //  and false if it couldn't wait for any I/O to complete.
//         //
//
//         switch (FsInformationClass) {
//
//         case FileFsVolumeInformation:
//
//             //
//             //  This is the only routine we need the Vcb shared because of
//             //  copying the volume label.  All other routines copy fields that
//             //  cannot change or are just manifest constants.
//             //
//
//             if (!FatAcquireSharedVcb( IrpContext, Vcb )) {
//
//                 DebugTrace(0, Dbg, "Cannot acquire Vcb\n", 0);
//
//                 Status = FatFsdPostRequest( IrpContext, Irp );
//                 IrpContext = NULL;
//                 Irp = NULL;
//
//             } else {
//
//                 WeAcquiredVcb = TRUE;
//
//                 Status = FatQueryFsVolumeInfo( IrpContext, Vcb, Buffer, &Length );
//             }
//
//             break;
//
//         case FileFsSizeInformation:
//
//             Status = FatQueryFsSizeInfo( IrpContext, Vcb, Buffer, &Length );
//             break;
//
//         case FileFsDeviceInformation:
//
//             Status = FatQueryFsDeviceInfo( IrpContext, Vcb, Buffer, &Length );
//             break;
//
//         case FileFsAttributeInformation:
//
//             Status = FatQueryFsAttributeInfo( IrpContext, Vcb, Buffer, &Length );
//             break;
//
//         case FileFsFullSizeInformation:
//
//             Status = FatQueryFsFullSizeInfo( IrpContext, Vcb, Buffer, &Length );
//             break;
//
// #if (NTDDI_VERSION >= NTDDI_WIN8)
//         case FileFsSectorSizeInformation:
//
//             Status = FatQueryFsSectorSizeInfo( IrpContext, Vcb, Buffer, &Length );
//             break;
// #endif
//
//         default:
//
//             Status = STATUS_INVALID_PARAMETER;
//             break;
//         }
//
//         //
//         //  Set the information field to the number of bytes actually filled in.
//         //
//
//         if (Irp != NULL) {
//
//             Irp->IoStatus.Information = IrpSp->Parameters.QueryVolume.Length - Length;
//         }
//
//     } _SEH2_FINALLY {
//
//         DebugUnwind( FatCommonQueryVolumeInfo );
//
//         if (WeAcquiredVcb) {
//
//             FatReleaseVcb( IrpContext, Vcb );
//         }
//
//         if (!_SEH2_AbnormalTermination()) {
//
//             FatCompleteRequest( IrpContext, Irp, Status );
//         }
//
//         DebugTrace(-1, Dbg, "FatCommonQueryVolumeInfo -> %08lx\n", Status);
//     } _SEH2_END;
//
//     return Status;
// }
//
//
// _Requires_lock_held_(_Global_critical_region_)
/* NTSTATUS */
// FatCommonSetVolumeInfo (
//     IN PIRP_CONTEXT IrpContext,
//     IN PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This is the common routine for setting Volume Information called by both
//     the fsd and fsp threads.
//
// Arguments:
//
//     Irp - Supplies the Irp being processed
//
// Return Value:
//
//     NTSTATUS - The return status for the operation
//
// --*/
//
// {
//     NTSTATUS Status = STATUS_SUCCESS;
//     PIO_STACK_LOCATION IrpSp;
//
//     PVCB Vcb;
//     PFCB Fcb;
//     PCCB Ccb;
//     TYPE_OF_OPEN TypeOfOpen;
//
//     FS_INFORMATION_CLASS FsInformationClass;
//     PVOID Buffer;
//
//     PAGED_CODE();
//
//     //
//     //  Get the current stack location
//     //
//
//     IrpSp = IoGetCurrentIrpStackLocation( Irp );
//
//     DebugTrace(+1, Dbg, "FatCommonSetVolumeInfo...\n", 0);
//     DebugTrace( 0, Dbg, "Irp                  = %p\n", Irp );
//     DebugTrace( 0, Dbg, "->Length             = %08lx\n", IrpSp->Parameters.SetVolume.Length);
//     DebugTrace( 0, Dbg, "->FsInformationClass = %08lx\n", IrpSp->Parameters.SetVolume.FsInformationClass);
//     DebugTrace( 0, Dbg, "->Buffer             = %p\n", Irp->AssociatedIrp.SystemBuffer);
//
//     //
//     //  Reference our input parameters to make things easier
//     //
//
//     FsInformationClass = IrpSp->Parameters.SetVolume.FsInformationClass;
//     Buffer = Irp->AssociatedIrp.SystemBuffer;
//
//     //
//     //  Decode the file object to get the Vcb
//     //
//
//     TypeOfOpen = FatDecodeFileObject( IrpSp->FileObject, &Vcb, &Fcb, &Ccb );
//
//     if (TypeOfOpen != UserVolumeOpen) {
//
//         FatCompleteRequest( IrpContext, Irp, STATUS_ACCESS_DENIED );
//
//         DebugTrace(-1, Dbg, "FatCommonSetVolumeInfo -> STATUS_ACCESS_DENIED\n", 0);
//
//         return STATUS_ACCESS_DENIED;
//     }
//
//     //
//     //  Acquire exclusive access to the Vcb and enqueue the Irp if we didn't
//     //  get access
//     //
//
//     if (!FatAcquireExclusiveVcb( IrpContext, Vcb )) {
//
//         DebugTrace(0, Dbg, "Cannot acquire Vcb\n", 0);
//
//         Status = FatFsdPostRequest( IrpContext, Irp );
//
//         DebugTrace(-1, Dbg, "FatCommonSetVolumeInfo -> %08lx\n", Status );
//         return Status;
//     }
//
//     _SEH2_TRY {
//
//         //
//         //  Make sure the vcb is in a usable condition.  This will raise
//         //  an error condition if the volume is unusable
//         //
//         //  Also verify the Root Dcb since we need info from there.
//         //
//
//         FatVerifyFcb( IrpContext, Vcb->RootDcb );
//
//         //
//         //  Based on the information class we'll do different actions.  Each
//         //  of the procedures that we're calling performs the action if
//         //  possible and returns true if it successful and false if it couldn't
//         //  wait for any I/O to complete.
//         //
//
//         switch (FsInformationClass) {
//
//         case FileFsLabelInformation:
//
//             Status = FatSetFsLabelInfo( IrpContext, Vcb, Buffer );
//             break;
//
//         default:
//
//             Status = STATUS_INVALID_PARAMETER;
//             break;
//         }
//
//         FatUnpinRepinnedBcbs( IrpContext );
//
//     } _SEH2_FINALLY {
//
//         DebugUnwind( FatCommonSetVolumeInfo );
//
//         FatReleaseVcb( IrpContext, Vcb );
//
//         if (!_SEH2_AbnormalTermination()) {
//
//             FatCompleteRequest( IrpContext, Irp, Status );
//         }
//
//         DebugTrace(-1, Dbg, "FatCommonSetVolumeInfo -> %08lx\n", Status);
//     } _SEH2_END;
//
//     return Status;
// }
//
//
// //
// //  Internal support routine
// //
//
/* NTSTATUS */
// FatQueryFsVolumeInfo (
//     IN PIRP_CONTEXT IrpContext,
//     IN PVCB Vcb,
//     IN PFILE_FS_VOLUME_INFORMATION Buffer,
//     IN OUT PULONG Length
//     )
//
// /*++
//
// Routine Description:
//
//     This routine implements the query volume info call
//
// Arguments:
//
//     Vcb - Supplies the Vcb being queried
//
//     Buffer - Supplies a pointer to the output buffer where the information
//         is to be returned
//
//     Length - Supplies the length of the buffer in bytes.  This variable
//         upon return receives the remaining bytes free in the buffer
//
// Return Value:
//
//     NTSTATUS - Returns the status for the query
//
// --*/
//
// {
//     ULONG BytesToCopy;
//
//     NTSTATUS Status;
//
//     PAGED_CODE();
//
//     DebugTrace(0, Dbg, "FatQueryFsVolumeInfo...\n", 0);
//
//     //
//     //  Zero out the buffer, then extract and fill up the non zero fields.
//     //
//
//     RtlZeroMemory( Buffer, sizeof(FILE_FS_VOLUME_INFORMATION) );
//
//     Buffer->VolumeSerialNumber = Vcb->Vpb->SerialNumber;
//
//     Buffer->SupportsObjects = FALSE;
//
//     *Length -= FIELD_OFFSET(FILE_FS_VOLUME_INFORMATION, VolumeLabel[0]);
//
//     //
//     //  Check if the buffer we're given is long enough
//     //
//
//     if ( *Length >= (ULONG)Vcb->Vpb->VolumeLabelLength ) {
//
//         BytesToCopy = Vcb->Vpb->VolumeLabelLength;
//
//         Status = STATUS_SUCCESS;
//
//     } else {
//
//         BytesToCopy = *Length;
//
//         Status = STATUS_BUFFER_OVERFLOW;
//     }
//
//     //
//     //  Copy over what we can of the volume label, and adjust *Length
//     //
//
//     Buffer->VolumeLabelLength = Vcb->Vpb->VolumeLabelLength;
//
//     RtlCopyMemory( &Buffer->VolumeLabel[0],
//                    &Vcb->Vpb->VolumeLabel[0],
//                    BytesToCopy );
//
//     *Length -= BytesToCopy;
//
//     //
//     //  Set our status and return to our caller
//     //
//
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     return Status;
// }
//
//
// //
// //  Internal support routine
// //
//
/* NTSTATUS */
// FatQueryFsSizeInfo (
//     IN PIRP_CONTEXT IrpContext,
//     IN PVCB Vcb,
//     IN PFILE_FS_SIZE_INFORMATION Buffer,
//     IN OUT PULONG Length
//     )
//
// /*++
//
// Routine Description:
//
//     This routine implements the query volume size call
//
// Arguments:
//
//     Vcb - Supplies the Vcb being queried
//
//     Buffer - Supplies a pointer to the output buffer where the information
//         is to be returned
//
//     Length - Supplies the length of the buffer in bytes.  This variable
//         upon return receives the remaining bytes free in the buffer
//
// Return Value:
//
//     Status - Returns the status for the query
//
// --*/
//
// {
//     PAGED_CODE();
//
//     DebugTrace(0, Dbg, "FatQueryFsSizeInfo...\n", 0);
//
//     RtlZeroMemory( Buffer, sizeof(FILE_FS_SIZE_INFORMATION) );
//
//     //
//     //  Set the output buffer.
//     //
//
//     Buffer->TotalAllocationUnits.LowPart =
//                                     Vcb->AllocationSupport.NumberOfClusters;
//     Buffer->AvailableAllocationUnits.LowPart =
//                                     Vcb->AllocationSupport.NumberOfFreeClusters;
//
//     Buffer->SectorsPerAllocationUnit = Vcb->Bpb.SectorsPerCluster;
//     Buffer->BytesPerSector = Vcb->Bpb.BytesPerSector;
//
//     //
//     //  Adjust the length variable
//     //
//
//     *Length -= sizeof(FILE_FS_SIZE_INFORMATION);
//
//     //
//     //  And return success to our caller
//     //
//
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     return STATUS_SUCCESS;
// }
//
//
// //
// //  Internal support routine
// //
//
/* NTSTATUS */
// FatQueryFsDeviceInfo (
//     IN PIRP_CONTEXT IrpContext,
//     IN PVCB Vcb,
//     IN PFILE_FS_DEVICE_INFORMATION Buffer,
//     IN OUT PULONG Length
//     )
//
// /*++
//
// Routine Description:
//
//     This routine implements the query volume device call
//
// Arguments:
//
//     Vcb - Supplies the Vcb being queried
//
//     Buffer - Supplies a pointer to the output buffer where the information
//         is to be returned
//
//     Length - Supplies the length of the buffer in bytes.  This variable
//         upon return receives the remaining bytes free in the buffer
//
// Return Value:
//
//     Status - Returns the status for the query
//
// --*/
//
// {
//     PAGED_CODE();
//
//     DebugTrace(0, Dbg, "FatQueryFsDeviceInfo...\n", 0);
//
//     RtlZeroMemory( Buffer, sizeof(FILE_FS_DEVICE_INFORMATION) );
//
//     //
//     //  Set the output buffer
//     //
//
//     Buffer->DeviceType = FILE_DEVICE_DISK;
//
//     Buffer->Characteristics = Vcb->TargetDeviceObject->Characteristics;
//
//     //
//     //  Adjust the length variable
//     //
//
//     *Length -= sizeof(FILE_FS_DEVICE_INFORMATION);
//
//     //
//     //  And return success to our caller
//     //
//
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     return STATUS_SUCCESS;
// }
//
//
// //
// //  Internal support routine
// //
//
/* NTSTATUS */
// FatQueryFsAttributeInfo (
//     IN PIRP_CONTEXT IrpContext,
//     IN PVCB Vcb,
//     IN PFILE_FS_ATTRIBUTE_INFORMATION Buffer,
//     IN OUT PULONG Length
//     )
//
// /*++
//
// Routine Description:
//
//     This routine implements the query volume attribute call
//
// Arguments:
//
//     Vcb - Supplies the Vcb being queried
//
//     Buffer - Supplies a pointer to the output buffer where the information
//         is to be returned
//
//     Length - Supplies the length of the buffer in bytes.  This variable
//         upon return receives the remaining bytes free in the buffer
//
// Return Value:
//
//     Status - Returns the status for the query
//
// --*/
//
// {
//     ULONG BytesToCopy;
//
//     NTSTATUS Status;
//
//     PAGED_CODE();
//
//     DebugTrace(0, Dbg, "FatQueryFsAttributeInfo...\n", 0);
//
//     //
//     //  Set the output buffer
//     //
//
//     Buffer->FileSystemAttributes = FILE_CASE_PRESERVED_NAMES |
//                                    FILE_UNICODE_ON_DISK;
//
//     if (FlagOn( Vcb->VcbState, VCB_STATE_FLAG_WRITE_PROTECTED )) {
//
//         SetFlag( Buffer->FileSystemAttributes, FILE_READ_ONLY_VOLUME );
//     }
//
//
//     Buffer->MaximumComponentNameLength = FatData.ChicagoMode() ? 255 : 12;
//
//     if (FatIsFat32(Vcb)) {
//
//         //
//         //  Determine how much of the file system name will fit.
//         //
//
//         if ( (*Length - FIELD_OFFSET( FILE_FS_ATTRIBUTE_INFORMATION,
//                                       FileSystemName[0] )) >= 10 ) {
//
//             BytesToCopy = 10;
//             *Length -= FIELD_OFFSET( FILE_FS_ATTRIBUTE_INFORMATION,
//                                      FileSystemName[0] ) + 10;
//             Status = STATUS_SUCCESS;
//
//         } else {
//
//             BytesToCopy = *Length - FIELD_OFFSET( FILE_FS_ATTRIBUTE_INFORMATION,
//                                                   FileSystemName[0]);
//             *Length = 0;
//
//             Status = STATUS_BUFFER_OVERFLOW;
//         }
//
//         RtlCopyMemory( &Buffer->FileSystemName[0], L"FAT32", BytesToCopy );
//
//     } else {
//
//         //
//         //  Determine how much of the file system name will fit.
//         //
//
//         if ( (*Length - FIELD_OFFSET( FILE_FS_ATTRIBUTE_INFORMATION,
//                                       FileSystemName[0] )) >= 6 ) {
//
//             BytesToCopy = 6;
//             *Length -= FIELD_OFFSET( FILE_FS_ATTRIBUTE_INFORMATION,
//                                      FileSystemName[0] ) + 6;
//             Status = STATUS_SUCCESS;
//
//         } else {
//
//             BytesToCopy = *Length - FIELD_OFFSET( FILE_FS_ATTRIBUTE_INFORMATION,
//                                                   FileSystemName[0]);
//             *Length = 0;
//
//             Status = STATUS_BUFFER_OVERFLOW;
//         }
//
//
//         RtlCopyMemory( &Buffer->FileSystemName[0], L"FAT", BytesToCopy );
//     }
//
//     Buffer->FileSystemNameLength       = BytesToCopy;
//
//     //
//     //  And return success to our caller
//     //
//
//     UNREFERENCED_PARAMETER( IrpContext );
//     UNREFERENCED_PARAMETER( Vcb );
//
//     return Status;
// }
//
//
// //
// //  Internal support routine
// //
//
/* NTSTATUS */
// FatQueryFsFullSizeInfo (
//     IN PIRP_CONTEXT IrpContext,
//     IN PVCB Vcb,
//     IN PFILE_FS_FULL_SIZE_INFORMATION Buffer,
//     IN OUT PULONG Length
//     )
//
// /*++
//
// Routine Description:
//
//     This routine implements the query volume full size call
//
// Arguments:
//
//     Vcb - Supplies the Vcb being queried
//
//     Buffer - Supplies a pointer to the output buffer where the information
//         is to be returned
//
//     Length - Supplies the length of the buffer in bytes.  This variable
//         upon return receives the remaining bytes free in the buffer
//
// Return Value:
//
//     Status - Returns the status for the query
//
// --*/
//
// {
//     PAGED_CODE();
//
//     DebugTrace(0, Dbg, "FatQueryFsSizeInfo...\n", 0);
//
//     RtlZeroMemory( Buffer, sizeof(FILE_FS_FULL_SIZE_INFORMATION) );
//
//     Buffer->TotalAllocationUnits.LowPart =
//                                 Vcb->AllocationSupport.NumberOfClusters;
//     Buffer->CallerAvailableAllocationUnits.LowPart =
//                                 Vcb->AllocationSupport.NumberOfFreeClusters;
//     Buffer->ActualAvailableAllocationUnits.LowPart =
//         Buffer->CallerAvailableAllocationUnits.LowPart;
//     Buffer->SectorsPerAllocationUnit = Vcb->Bpb.SectorsPerCluster;
//     Buffer->BytesPerSector = Vcb->Bpb.BytesPerSector;
//
//     //
//     //  Adjust the length variable
//     //
//
//     *Length -= sizeof(FILE_FS_FULL_SIZE_INFORMATION);
//
//     //
//     //  And return success to our caller
//     //
//
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     return STATUS_SUCCESS;
// }
//
//
// //
// //  Internal support routine
// //
//
/* NTSTATUS */
// FatSetFsLabelInfo (
//     IN PIRP_CONTEXT IrpContext,
//     IN PVCB Vcb,
//     IN PFILE_FS_LABEL_INFORMATION Buffer
//     )
//
// /*++
//
// Routine Description:
//
//     This routine implements the set volume label call
//
// Arguments:
//
//     Vcb - Supplies the Vcb being queried
//
//     Buffer - Supplies the input where the information is stored.
//
// Return Value:
//
//     NTSTATUS - Returns the status for the operation
//
// --*/
//
// {
//     NTSTATUS Status;
//
//     PDIRENT Dirent;
//     PBCB DirentBcb = NULL;
//     ULONG ByteOffset;
//
//     WCHAR TmpBuffer[11];
//     UCHAR OemBuffer[11];
//     OEM_STRING OemLabel;
//     UNICODE_STRING UnicodeString;
//     UNICODE_STRING UpcasedLabel;
//
//     PAGED_CODE();
//
//     DebugTrace(+1, Dbg, "FatSetFsLabelInfo...\n", 0);
//
//     //
//     //  Setup our local variable
//     //
//
//     UnicodeString.Length = (USHORT)Buffer->VolumeLabelLength;
//     UnicodeString.MaximumLength = UnicodeString.Length;
//     UnicodeString.Buffer = (PWSTR) &Buffer->VolumeLabel[0];
//
//     //
//     //  Make sure the name can fit into the stack buffer
//     //
//
//     if ( UnicodeString.Length > 11*sizeof(WCHAR) ) {
//
//         return STATUS_INVALID_VOLUME_LABEL;
//     }
//
//     //
//     //  Upcase the name and convert it to the Oem code page.
//     //
//
//     OemLabel.Buffer = (PCHAR)&OemBuffer[0];
//     OemLabel.Length = 0;
//     OemLabel.MaximumLength = 11;
//
//     Status = RtlUpcaseUnicodeStringToCountedOemString( &OemLabel,
//                                                        &UnicodeString,
//                                                        FALSE );
//
//     //
//     //  Volume label that fits in 11 unicode character length limit
//     //  is not necessarily within 11 characters in OEM character set.
//     //
//
//     if (!NT_SUCCESS( Status )) {
//
//         DebugTrace(-1, Dbg, "FatSetFsLabelInfo:  Label must be too long. %08lx\n", Status );
//
//         return STATUS_INVALID_VOLUME_LABEL;
//     }
//
//     //
//     //  Strip spaces off of the label.
//     //
//
//     if (OemLabel.Length > 0) {
//
//         USHORT i;
//         USHORT LastSpaceIndex = MAXUSHORT;
//
//         //
//         //  Check the label for illegal characters
//         //
//
//         for ( i = 0; i < (ULONG)OemLabel.Length; i += 1 ) {
//
//             if ( FsRtlIsLeadDbcsCharacter( OemLabel.Buffer[i] ) ) {
//
//                 LastSpaceIndex = MAXUSHORT;
//                 i += 1;
//                 continue;
//             }
//
//             if (!FsRtlIsAnsiCharacterLegalFat(OemLabel.Buffer[i], FALSE) ||
//                 (OemLabel.Buffer[i] == '.')) {
//
//                 return STATUS_INVALID_VOLUME_LABEL;
//             }
//
//             //
//             //  Watch for the last run of spaces, so we can strip them.
//             //
//
//             if (OemLabel.Buffer[i] == ' ' &&
//                 LastSpaceIndex == MAXUSHORT) {
//                 LastSpaceIndex = i;
//             } else {
//                 LastSpaceIndex = MAXUSHORT;
//             }
//         }
//
//         if (LastSpaceIndex != MAXUSHORT) {
//             OemLabel.Length = LastSpaceIndex;
//         }
//     }
//
//     //
//     //  Get the Unicode upcased string to store in the VPB.
//     //
//
//     UpcasedLabel.Length = UnicodeString.Length;
//     UpcasedLabel.MaximumLength = 11*sizeof(WCHAR);
//     UpcasedLabel.Buffer = &TmpBuffer[0];
//
//     Status = RtlOemStringToCountedUnicodeString( &UpcasedLabel,
//                                                  &OemLabel,
//                                                  FALSE );
//
//     if (!NT_SUCCESS( Status )) {
//
//         DebugTrace(-1, Dbg, "FatSetFsLabelInfo:  Label must be too long. %08lx\n", Status );
//
//         return STATUS_INVALID_VOLUME_LABEL;
//     }
//
//     DirentBcb = NULL;
//
//     //
//     //  Make this look like a write through to disk.  This is important to
//     //  avoid a unpleasant window where it looks like we have the wrong volume.
//     //
//
//     SetFlag( IrpContext->Flags, IRP_CONTEXT_FLAG_WRITE_THROUGH );
//
//     _SEH2_TRY {
//
//         //
//         //  Are we setting or removing the label?  Note that shaving spaces could
//         //  make this different than wondering if the input buffer is non-zero length.
//         //
//
//         if (OemLabel.Length > 0) {
//
//             //
//             //  Translate the first character from 0xe5 to 0x5.
//             //
//
//             if ((UCHAR)OemLabel.Buffer[0] == 0xe5) {
//
//                 OemLabel.Buffer[0] = FAT_DIRENT_REALLY_0E5;
//             }
//
//             //
//             //  Locate the volume label if there already is one
//             //
//
//             FatLocateVolumeLabel( IrpContext,
//                                   Vcb,
//                                   &Dirent,
//                                   &DirentBcb,
//                                   (PVBO)&ByteOffset );
//
//             //
//             //  Check that we really got one, if not then we need to create
//             //  a new one.  The procedure we call will raise an appropriate
//             //  status if we are not able to allocate a new dirent
//             //
//
//             if (Dirent == NULL) {
//
//                 ByteOffset = FatCreateNewDirent( IrpContext,
//                                                  Vcb->RootDcb,
//                                                  1,
//                                                  FALSE );
//
//                 FatPrepareWriteDirectoryFile( IrpContext,
//                                               Vcb->RootDcb,
//                                               ByteOffset,
//                                               sizeof(DIRENT),
//                                               &DirentBcb,
// #ifndef __REACTOS__
//                                               &Dirent,
// #else
//                                               (PVOID *)&Dirent,
// #endif
//                                               FALSE,
//                                               TRUE,
//                                               &Status );
//
//                 NT_ASSERT( NT_SUCCESS( Status ));
//
//             } else {
//
//                 //
//                 //  Just mark this guy dirty now.
//                 //
//
//                 FatSetDirtyBcb( IrpContext, DirentBcb, Vcb, TRUE );
//             }
//
//             //
//             //  Now reconstruct the volume label dirent.
//             //
//
//             FatConstructLabelDirent( IrpContext,
//                                      Dirent,
//                                      &OemLabel );
//
//             //
//             //  Unpin the Bcb here so that we will get any IO errors
//             //  here before changing the VPB label.
//             //
//
//             FatUnpinBcb( IrpContext, DirentBcb );
//             FatUnpinRepinnedBcbs( IrpContext );
//
//             //
//             //  Now set the upcased label in the VPB
//             //
//
//             RtlCopyMemory( &Vcb->Vpb->VolumeLabel[0],
//                            &UpcasedLabel.Buffer[0],
//                            UpcasedLabel.Length );
//
//             Vcb->Vpb->VolumeLabelLength = UpcasedLabel.Length;
//
//         } else {
//
//             //
//             //  Otherwise we're trying to delete the label
//             //  Locate the current volume label if there already is one
//             //
//
//             FatLocateVolumeLabel( IrpContext,
//                                   Vcb,
//                                   &Dirent,
//                                   &DirentBcb,
//                                   (PVBO)&ByteOffset );
//
//             //
//             //  Check that we really got one
//             //
//
//             if (Dirent == NULL) {
//
//                 try_return( Status = STATUS_SUCCESS );
//             }
//
//             //
//             //  Now delete the current label.
//             //
//
//             Dirent->FileName[0] = FAT_DIRENT_DELETED;
//
//             NT_ASSERT( (Vcb->RootDcb->Specific.Dcb.UnusedDirentVbo == 0xffffffff_u32) ||
//                     RtlAreBitsSet( &Vcb->RootDcb->Specific.Dcb.FreeDirentBitmap,
//                                    ByteOffset / sizeof(DIRENT),
//                                    1 ) );
//
//             RtlClearBits( &Vcb->RootDcb->Specific.Dcb.FreeDirentBitmap,
//                           ByteOffset / sizeof(DIRENT),
//                           1 );
//
//             FatSetDirtyBcb( IrpContext, DirentBcb, Vcb, TRUE );
//
//             //
//             //  Unpin the Bcb here so that we will get any IO errors
//             //  here before changing the VPB label.
//             //
//
//             FatUnpinBcb( IrpContext, DirentBcb );
//             FatUnpinRepinnedBcbs( IrpContext );
//
//             //
//             //  Now set the label in the VPB
//             //
//
//             Vcb->Vpb->VolumeLabelLength = 0;
//         }
//
//         Status = STATUS_SUCCESS;
//
//     try_exit: NOTHING;
//     } _SEH2_FINALLY {
//
//         DebugUnwind( FatSetFsALabelInfo );
//
//         FatUnpinBcb( IrpContext, DirentBcb );
//
//         DebugTrace(-1, Dbg, "FatSetFsALabelInfo -> STATUS_SUCCESS\n", 0);
//     } _SEH2_END;
//
//     return Status;
// }
//
// #if (NTDDI_VERSION >= NTDDI_WIN8)
//
/* NTSTATUS */
// FatQueryFsSectorSizeInfo (
//      PIRP_CONTEXT IrpContext,
//      PVCB Vcb,
//      PFILE_FS_SECTOR_SIZE_INFORMATION Buffer,
//      PULONG Length
//     )
//
// /*++
//
// Routine Description:
//
//     This routine implements the query sector size information call
//     This operation will work on any handle and requires no privilege.
//
// Arguments:
//
//     Vcb - Supplies the Vcb being queried
//
//     Buffer - Supplies a pointer to the output buffer where the information
//         is to be returned
//
//     Length - Supplies the length of the buffer in bytes.  This variable
//         upon return receives the remaining bytes free in the buffer
//
// Return Value:
//
//     NTSTATUS - Returns the status for the query
//
// --*/
//
// {
//     NTSTATUS Status;
//
//     PAGED_CODE();
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     //
//     //  Sufficient buffer size is guaranteed by the I/O manager or the
//     //  originating kernel mode driver.
//     //
//
//     ASSERT( *Length >= sizeof( FILE_FS_SECTOR_SIZE_INFORMATION ));
//     _Analysis_assume_( *Length >= sizeof( FILE_FS_SECTOR_SIZE_INFORMATION ));
//
//     //
//     //  Retrieve the sector size information
//     //
//
//     Status = FsRtlGetSectorSizeInformation( Vcb->Vpb->RealDevice,
//                                             Buffer );
//
//     //
//     //  Adjust the length variable
//     //
//
//     if (NT_SUCCESS( Status )) {
//
//         *Length -= sizeof( FILE_FS_SECTOR_SIZE_INFORMATION );
//     }
//
//     return Status;
// }
//
// #endif
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

// FatFsdQueryVolumeInformation — FSD dispatch entry (translated from commented-out C)
#[no_mangle]
pub unsafe extern "system" fn FatFsdQueryVolumeInformation(
    VolumeDeviceObject: PVOLUME_DEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut Status: NTSTATUS = 0;
    // PIRP_CONTEXT — one pointer. See FatFsdSetVolumeInformation above; this
    // path never dereferenced the over-deep local, so it was mistyped rather
    // than broken, but the type is what the next reader will trust.
    let mut IrpContext: PIRP_CONTEXT = core::ptr::null_mut();
    let mut Irp = Irp;
    let TopLevel: BOOLEAN;
    DebugTrace(1u32, Dbg as u32, b"FatFsdQueryVolumeInformation\n\0".as_ptr() as *const i8, 0u32);
    FsRtlEnterFileSystem();
    TopLevel = FatIsIrpTopLevel(Irp);
    // _SEH2_TRY { ... } _SEH2_EXCEPT(FatExceptionFilter(IrpContext, ...)) { FatProcessException }
    let p_status = core::ptr::addr_of_mut!(Status);
    let p_irp_context = core::ptr::addr_of_mut!(IrpContext);
    if let Err(code) = crate::seh::fat_try_except(p_irp_context as *const PIRP_CONTEXT, || {
        *p_irp_context = FatCreateIrpContext(Irp, (CanFsdWait(Irp)) as _);
        *p_status = FatCommonQueryVolumeInfo((*p_irp_context).cast(), Irp);
    }) {
        Status = FatProcessException(IrpContext, Irp, code);
    }
    if TopLevel != 0 { IoSetTopLevelIrp(core::ptr::null_mut()); }
    FsRtlExitFileSystem();
    DebugTrace((-1i32) as u32, Dbg as u32, b"FatFsdQueryVolumeInformation -> %08lx\n\0".as_ptr() as *const i8, Status as u32);
    let _ = VolumeDeviceObject;
    Status
}
