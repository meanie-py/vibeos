use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{CcCopyWrite, CcPinRead, CcPurgeCacheSection, CcSetDirtyPinnedData, CcSetFileSizes, CcUnpinData, ExAcquireResourceExclusiveLite, ExAcquireResourceSharedLite, ExFreePool, ExReleaseResourceLite, _IO_STATUS_BLOCK, LARGE_INTEGER, LARGE_MCB, OEM_STRING, PCC_FILE_SIZES, PFILE_FULL_EA_INFORMATION, PFILE_OBJECT, POEM_STRING, RtlCompareString};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{DCB, DIRENT, EA_BCB_ARRAY_SIZE, EA_FILE_HEADER, EA_OFF_TABLE, EA_RANGE, EA_SET_HEADER, FCB, IRP_CONTEXT, IRP_CONTEXT_FLAG_WAIT, PACKED_EA, PBCB, PDIRENT, PEA_RANGE, PFCB, PIRP_CONTEXT, PVCB, VCB};
use ntkrnl_sys::ddk::{CC_FILE_SIZES, FILE_FULL_EA_INFORMATION, FILE_OBJECT, _PUBLIC_BCB as BCB};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{TAG_EA_DATA};
// resolve_imports.py: explicit kernel/private-type imports
use crate::ddk_shims::{FsRtlAllocatePoolWithTag, FsRtlInitializeLargeMcb, FsRtlUninitializeLargeMcb};

extern "system" {
    } extern "C" { fn DebugTrace(severity: i32, component: u32, format: *const i8, ...); } extern "system" {
    fn Fat8dot3ToString( irp_context: *mut IRP_CONTEXT, dirent: *const DIRENT, flag: i32, filename: *mut OEM_STRING, );
    fn FatUnpinBcb(irp_context: *mut IRP_CONTEXT, bcb: *mut *mut core::ffi::c_void);
    fn FatReleaseFcb(irp_context: *mut IRP_CONTEXT, fcb: *mut FCB);
}

/// `SizeOfPackedEa` (`fat.h:738`) is a MACRO, never a function. The `extern` that used
/// to sit in the block above therefore became an unresolved symbol, and the cohab stub
/// generator filled it with `KeBugCheckEx(0x23, 0xDEAD0000)` — armed on any EA-bearing
/// FAT file. Same class as the FatGenerateFileId* pair (create/fileid). `ea.rs` already
/// had the macro right, so share it rather than keep a second copy; the out-param shape
/// is preserved so the call sites below are untouched.
#[inline]
unsafe fn SizeOfPackedEa(ea: *const PACKED_EA, out_size: *mut u32) {
    *out_size = crate::ea::SizeOfPackedEaInline(ea);
}
extern "system" {
    fn PagedCode();
    fn SizeOfFullEa(ea: *mut FILE_FULL_EA_INFORMATION) -> u32;
    fn FatUpcaseEaName( IrpContext: *mut IRP_CONTEXT, Source: *mut OEM_STRING, Destination: *mut OEM_STRING, ) -> ();
    fn FlagOn(flags: u8, bit: u32) -> i32;
    fn FsRtlIsAnsiCharacterLegalFat(Character: u8, WildCardsPermissible: i32) -> u8;
    fn FsRtlIsLeadDbcsCharacter(Character: u8) -> u8;
    fn NT_ASSERT(Expression: i32);
    fn FatRaiseStatus(IrpContext: *mut IRP_CONTEXT, ErrorStatus: NTSTATUS);
    fn Add2Ptr(Ptr: *mut core::ffi::c_void, Bytes: u32) -> *mut core::ffi::c_void;
    fn BooleanFlagOn(Flags: u32, Flag: u32) -> u8;
    fn DebugUnwind(fn_name: usize);
    fn FatAcquireExclusiveFcb(IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB) -> u8;
    fn FatAcquireSharedFcb(IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB) -> u8;
    fn FatAllocateDiskSpace( IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB, Hint: u32, AllocationSize: *mut u32, IsDirectory: u8, Mcb: *mut LARGE_MCB, );
    fn FatConstructDirent(IrpContext: *mut IRP_CONTEXT, Dirent: *mut DIRENT, FileName: *mut OEM_STRING, IsDirectory: u8, IsExtended: u8, Lbo: *mut LARGE_INTEGER, Attributes: u16, Initialize: u8, SetCreationTime: *const LARGE_INTEGER);
    fn FatCreateNewDirent(IrpContext: *mut IRP_CONTEXT, Dcb: *mut DCB, DirentIndex: u32, IsExtended: u32) -> u32;
    fn FatDeallocateDiskSpace( IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB, Mcb: *mut LARGE_MCB, IsDirectory: u8, );
    fn FatDeleteDirent(IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB, EaHandle: *mut u16, DeleteFile: u32);
    fn FatGetDirentFromFcbOrDcb(IrpContext: *mut IRP_CONTEXT, FcbOrDcb: *mut FCB, Unpin: u32, EaDirent: *mut *mut DIRENT, EaBcb: *mut *mut BCB);
    fn FatGetIndexFromLbo(Vcb: *mut VCB, Lbo: i64) -> u16;
    fn FatLocateSimpleOemDirent( IrpContext: *mut IRP_CONTEXT, Dcb: *mut DCB, FileName: *mut OEM_STRING, EaDirent: *mut *mut DIRENT, EaBcb: *mut *mut BCB, ByteOffset: *mut u32, );
    fn FatLookupFileAllocationSize(IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB);
    fn FatLookupMcbEntry(Vcb: *mut VCB, Mcb: *mut LARGE_MCB, Vbo: u32, Lbo: *mut i64, Offset: *mut u32, Length: *mut u32) -> u8;
    fn FatOpenEaFile(IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB) -> *mut FILE_OBJECT;
    fn FatPinMappedData( IrpContext: *mut IRP_CONTEXT, Dcb: *mut DCB, ByteOffset: u32, Length: u32, Bcb: *mut *mut BCB, );
    fn FatPrepareWriteDirectoryFile( IrpContext: *mut IRP_CONTEXT, Dcb: *mut DCB, ByteOffset: u32, Length: u32, Bcb: *mut *mut BCB, Dirent: *mut *mut DIRENT, UpdateSize: u8, Initialize: u8, Status: *mut i32, );
    fn FatVerifyFcb(IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB);
    fn FatSetDirtyBcb(IrpContext: *mut IRP_CONTEXT, Bcb: *mut BCB, Vcb: *mut VCB, Dirty: u8);
    fn FatSplitAllocation(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB, Mcb: *mut LARGE_MCB, Vbo: u32, NewMcb: *mut LARGE_MCB);
    fn FatMergeAllocation(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB, TargetMcb: *mut LARGE_MCB, SourceMcb: *mut LARGE_MCB);
    fn FatIsFat32(Vcb: *mut VCB) -> u8;
    fn CcFlushCache( SectionObjectPointer: *mut core::ffi::c_void, FileOffset: *mut LARGE_INTEGER, Length: u32, IoStatus: *mut _IO_STATUS_BLOCK, );
}

/// fat.h `GetcbList` macro: the unaligned little-endian ULONG in cbList. (This was an
/// extern bound to a rust_cohab_stub.c stub that always returned FALSE.)
#[inline(always)]
unsafe fn GetcbList(ea_set: *mut EA_SET_HEADER) -> u32 {
    u32::from_le_bytes((*ea_set).cbList)
}

// ---- file-scope preamble (translated) ----
pub const Dbg: u32 = DEBUG_TRACE_EA;
pub const EA_SECTION_SIZE: u32 = 0x00040000;

const EA_NEED_EA_FLAG: u32 = 0x80; // placeholder, actual value unknown

// Constants used by this function
const DEBUG_TRACE_EA: u32 = 0x0100; // Dbg
const EA_FILE_SIGNATURE: u32 = 0x4445; // 'EFAS' or similar; placeholder value
const MAX_EA_BASE_INDEX: u32 = 0xF0; // placeholder
const UNUSED_EA_HANDLE: u16 = 0xFFFF;
const FCB_LOOKUP_ALLOCATIONSIZE_HINT: i64 = -1; // placeholder sentinel
const FcbGood: u32 = 1; // enum variant placeholder
const FAT_DIRENT_ATTR_READ_ONLY: u32 = 0x01;
const FAT_DIRENT_ATTR_HIDDEN: u32 = 0x02;
const FAT_DIRENT_ATTR_SYSTEM: u32 = 0x04;
const FAT_DIRENT_ATTR_ARCHIVE: u32 = 0x20;
const STATUS_CANT_WAIT: i32 = 0xC00000D8_u32 as i32;
const STATUS_DATA_ERROR: i32 = 0xC000003E_u32 as i32;

// Extern declarations for unknown functions


const MIN_EA_HANDLE: u16 = 1;
const MAX_EA_HANDLE: u16 = 0x77FF;
const MAX_EA_OFFSET_INDEX: u32 = 128;
const EA_CBLIST_OFFSET: u32 = 0x1A;
const EA_SET_SIGNATURE: u32 = 0x4145;
const STATUS_NONEXISTENT_EA_ENTRY: i32 = 0xC0000051_u32 as i32;
const STATUS_UNABLE_TO_DELETE_SECTION: i32 = 0xC000001B_u32 as i32;
const PAGE_SIZE: u32 = 0x1000;


const STATUS_NO_EAS_ON_FILE: i32 = 0xC0000052_u32 as i32; // NTSTATUS placeholder

// Unknown constants
const STATUS_INVALID_EA_NAME: NTSTATUS = 0x80000013_u32 as i32;
const STATUS_EA_TOO_LARGE: NTSTATUS = 0xC0000050_u32 as i32;
const FILE_NEED_EA: u8 = 0x80;
const MAXIMUM_EA_SIZE: u32 = 0xFFFF;

// Unknown forward declarations
#[no_mangle]
pub unsafe extern "system" fn FatGetEaLength(
    IrpContext: *mut IRP_CONTEXT,
    Vcb: *mut VCB,
    Dirent: *mut DIRENT,
    EaLength: *mut u32,
) {
    const Dbg: i32 = 0; // DEBUG_TRACE_EA
    const STATUS_NO_EAS_ON_FILE: i32 = 0xC0000052_u32 as i32;

    let mut EaBcb: *mut BCB = core::ptr::null_mut();
    let mut LockedEaFcb: i32 = 0;

    // C: EA_RANGE EaSetRange; RtlZeroMemory( &EaSetRange, sizeof( EA_RANGE ));
    let mut EaSetRange_uninit = core::mem::MaybeUninit::<EA_RANGE>::uninit();
    ntkrnl_sys::rtl::RtlZeroMemory(
        EaSetRange_uninit.as_mut_ptr() as *mut c_void,
        core::mem::size_of::<EA_RANGE>(),
    );
    let mut EaSetRange = EaSetRange_uninit.assume_init();

    PAGED_CODE!();

    DebugTrace(
        1,
        (Dbg) as u32,
        b"FatGetEaLength ...\n\0".as_ptr() as *const i8,
        0,
    );

    // If this is Fat32 volume, or handle is 0 then the Ea length is 0.
    if FatIsFat32(Vcb as *mut VCB) != 0 || (*Dirent).__bindgen_anon_1.bindgen_union_field == 0 {
        *EaLength = 0;
        DebugTrace(
            -1,
            (Dbg) as u32,
            b"FatGetEaLength -> %08lx\n\0".as_ptr() as *const i8,
            1,
        );
        return;
    }

    let p_EaBcb = core::ptr::addr_of_mut!(EaBcb);
    let p_LockedEaFcb = core::ptr::addr_of_mut!(LockedEaFcb);
    let p_EaSetRange = core::ptr::addr_of_mut!(EaSetRange);
    // _SEH2_TRY / _SEH2_FINALLY
    crate::seh::try_finally(&mut (), |_| {
        let mut EaDirent: *mut DIRENT = core::ptr::null_mut();
        let mut ThisFilename: OEM_STRING = core::mem::zeroed();
        let mut Buffer: [u8; 12] = [0u8; 12];
        let EaSetHeader: *mut EA_SET_HEADER;

        // Initial the local values.
        *p_EaBcb = core::ptr::null_mut();
        *p_LockedEaFcb = 0;

        // Try to get the Ea file object.
        FatGetEaFile(
            IrpContext,
            Vcb,
            &raw mut EaDirent,
            (&raw mut (*p_EaBcb)).cast::<*mut BCB>(),
            0, // CreateFile
            0, // ExclusiveFcb
        );
        *p_LockedEaFcb = 1;

        // If we didn't get the file because it doesn't exist, the disk is corrupted.
        if (*Vcb).VirtualEaFile.is_null() {
            DebugTrace(
                0,
                (Dbg) as u32,
                b"FatGetEaLength:  Ea file doesn't exist\n\0".as_ptr() as *const i8,
                0,
            );
            FatRaiseStatus(IrpContext, STATUS_NO_EAS_ON_FILE);
        }

        // Try to pin down the Ea set header for the index in the dirent.
        ThisFilename.Buffer = Buffer.as_mut_ptr() as *mut i8;
        Fat8dot3ToString(IrpContext, Dirent, 0, &raw mut ThisFilename);

        FatReadEaSet(
            IrpContext,
            Vcb,
            (*Dirent).__bindgen_anon_1.bindgen_union_field,
            &raw mut ThisFilename,
            0,
            &raw mut (*p_EaSetRange),
        );

        EaSetHeader = (*p_EaSetRange).Data as *mut EA_SET_HEADER;

        // We now have the Ea set header for this file. Simply copy the Ea length field.
        // CopyUchar4( EaLength, EaSetHeader->cbList ). The extern this called passed the
        // VALUE where the stub dereferences a source pointer.
        *EaLength = u32::from_le_bytes((*EaSetHeader).cbList);
        DebugTrace(
            0,
            (Dbg) as u32,
            b"FatGetEaLength:  Length of Ea is -> %08lx\n\0".as_ptr() as *const i8,
            *EaLength,
        );
    }, |_, _abnormal| {
        DebugUnwind(FatGetEaLength as usize);

        // Unpin the EaDirent and the EaSetHeader if pinned.
        FatUnpinBcb(IrpContext, core::ptr::addr_of_mut!(*p_EaBcb) as *mut *mut core::ffi::c_void);

        FatUnpinEaRange(IrpContext, &raw mut (*p_EaSetRange));

        // Release the Fcb for the Ea file if locked.
        if (*p_LockedEaFcb) != 0 {
            FatReleaseFcb(IrpContext, (*Vcb).EaFcb);
        }

        DebugTrace(
            -1,
            (Dbg) as u32,
            b"FatGetEaLength:  Ea length -> %08lx\n\0".as_ptr() as *const i8,
            *EaLength,
        );
    });

    return;
}

#[no_mangle]
pub unsafe extern "system" fn FatGetNeedEaCount(
    IrpContext: *mut IRP_CONTEXT,
    Vcb: *const VCB,
    Dirent: *const DIRENT,
    NeedEaCount: *mut u32,
) {
    unsafe {
        // local variables
        let mut EaBcb: *mut BCB = core::ptr::null_mut();
        let mut LockedEaFcb: i32 = 0; // FALSE
        let mut EaSetRange: EA_RANGE = unsafe { core::mem::zeroed() };
        core::ptr::write_bytes(
            &mut EaSetRange as *mut EA_RANGE as *mut u8,
            0u8,
            core::mem::size_of::<EA_RANGE>(),
        );

        // PAGED_CODE();  -- left as comment, expands to nothing in release
        const Dbg: u32 = 0; // DEBUG_TRACE_EA
        DebugTrace(1, Dbg, b"FatGetNeedEaCount ...\n\0".as_ptr() as *const i8 , 0i32);

        if (*Dirent).__bindgen_anon_1.bindgen_union_field == 0 {
            *NeedEaCount = 0;
            DebugTrace(
                -1,
                Dbg,
                b"FatGetNeedEaCount -> %08lx\n\0".as_ptr() as *const i8 ,
                1i32, // TRUE
            );
            return;
        }

        let p_EaBcb = core::ptr::addr_of_mut!(EaBcb);
        let p_LockedEaFcb = core::ptr::addr_of_mut!(LockedEaFcb);
        let p_EaSetRange = core::ptr::addr_of_mut!(EaSetRange);
        // _SEH2_TRY / _SEH2_FINALLY
        crate::seh::try_finally(&mut (), |_| {
            let mut EaDirent: *mut DIRENT = core::ptr::null_mut();
            let mut ThisFilename: OEM_STRING = core::mem::zeroed();
            let mut Buffer: [u8; 12] = [0u8; 12];
            let mut EaSetHeader: *mut EA_SET_HEADER;

            // initialize locals again (as in C)
            *p_EaBcb = core::ptr::null_mut();
            *p_LockedEaFcb = 0;

            FatGetEaFile(
                IrpContext as *mut IRP_CONTEXT,
                // Vcb is passed as IN OUT, cast const to mut
                Vcb as *mut VCB,
                &mut EaDirent,
                &mut (*p_EaBcb),
                0, // FALSE
                0, // FALSE
            );

            *p_LockedEaFcb = 1; // TRUE

            if (*Vcb).VirtualEaFile.is_null() {
                DebugTrace(
                    0,
                    Dbg,
                    b"FatGetNeedEaCount:  Ea file doesn't exist\n\0".as_ptr() as *const i8 ,
                    0i32,
                );
                FatRaiseStatus(IrpContext as *mut IRP_CONTEXT, STATUS_NO_EAS_ON_FILE);
                // FatRaiseStatus doesn't return
            }

            // Fat8dot3ToString uses ThisFilename.Buffer as a PCHAR (pointer to Buffer)
            ThisFilename.Buffer = (Buffer.as_mut_ptr() as *mut u8).cast();
            Fat8dot3ToString(
                IrpContext as *mut IRP_CONTEXT,
                Dirent as *mut DIRENT,
                (0u8) as i32, // FALSE as u8
                &mut ThisFilename as *mut OEM_STRING,
            );

            FatReadEaSet(
                IrpContext as *mut IRP_CONTEXT,
                Vcb as *mut VCB,
                (*Dirent).__bindgen_anon_1.bindgen_union_field,
                &mut ThisFilename as *mut OEM_STRING,
                (0i32) as u8, // FALSE
                &mut (*p_EaSetRange) as *mut EA_RANGE,
            );

            EaSetHeader = (*p_EaSetRange).Data as *mut EA_SET_HEADER;

            *NeedEaCount = (*EaSetHeader).NeedEaCount;
        }, |_, _abnormal| {
            DebugUnwind(b"FatGetNeedEaCount\0".as_ptr() as usize);
            FatUnpinBcb(IrpContext as *mut IRP_CONTEXT, core::ptr::addr_of_mut!(*p_EaBcb) as *mut *mut core::ffi::c_void);
            FatUnpinEaRange(IrpContext as *mut IRP_CONTEXT, &mut (*p_EaSetRange) as PEA_RANGE);
            if (*p_LockedEaFcb) != 0 {
                FatReleaseFcb(IrpContext as *mut IRP_CONTEXT, (*Vcb).EaFcb);
            }
            DebugTrace(
                -1,
                Dbg,
                b"FatGetNeedEaCount:  NeedEaCount -> %08lx\n\0".as_ptr() as *const i8 ,
                *NeedEaCount,
            );
        });
    }
}


#[no_mangle]
pub unsafe extern "system" fn FatCreateEa(
    irp_context: PIRP_CONTEXT,
    vcb: PVCB,
    buffer: PUCHAR,
    length: ULONG,
    file_name: POEM_STRING,
    ea_handle: *mut USHORT,
) {
    // Pre-try locals used in finally
    let mut ea_bcb: *mut BCB = core::ptr::null_mut();
    let mut locked_ea_fcb: BOOLEAN = 0;

    let mut ea_set_range: EA_RANGE = core::mem::zeroed();
    let ea_set_range_ptr: *mut EA_RANGE = core::ptr::addr_of_mut!(ea_set_range);

    core::ptr::write_bytes(
        ea_set_range_ptr as *mut u8,
        0,
        core::mem::size_of::<EA_RANGE>(),
    );

    let mut ea_set_header: *mut EA_SET_HEADER = core::ptr::null_mut();
    let ea_set_header_ptr: *mut *mut EA_SET_HEADER = core::ptr::addr_of_mut!(ea_set_header);

    let p_ea_bcb = core::ptr::addr_of_mut!(ea_bcb);
    let p_locked_ea_fcb = core::ptr::addr_of_mut!(locked_ea_fcb);
    // _SEH2_TRY / _SEH2_FINALLY
    crate::seh::try_finally(&mut (), |_| {
        unsafe {
            // ---------- _SEH2_TRY body ----------
            let mut packed_eas_length: u32 = 0;
            let bytes_per_cluster: u32 = 1u32 << (*vcb).AllocationSupport.LogOfBytesPerCluster;

            // SIZE_OF_EA_SET_HEADER is 30 (fat.h); sizeof(EA_SET_HEADER) pads to 32.
            let size_of_ea_set_header: u32 = 30;
            let mut allocation_length = ((packed_eas_length
                + size_of_ea_set_header
                + bytes_per_cluster
                - 1)
                & !(bytes_per_cluster - 1)) as u32;

            *ea_set_header_ptr = FsRtlAllocatePoolWithTag(
                PagedPool,
                allocation_length,
                0x6574_6146, /* TAG_EA_SET_HEADER 'etaF' (was 0) */
            ) as *mut EA_SET_HEADER;

            core::ptr::write_bytes(
                *ea_set_header_ptr as *mut u8,
                0,
                allocation_length as usize,
            );

            let owner_file_name_offset = core::mem::offset_of!(EA_SET_HEADER, OwnerFileName);
            core::ptr::copy_nonoverlapping(
                (*file_name).Buffer as *const u8,
                (*ea_set_header_ptr as *mut u8).add(owner_file_name_offset),
                (*file_name).Length as usize,
            );

            allocation_length -= size_of_ea_set_header;

            let buffer_end = buffer.add(length as usize) as *const FILE_FULL_EA_INFORMATION;
            let mut full_ea = buffer as *const FILE_FULL_EA_INFORMATION;

            while full_ea < buffer_end {
                let ea_name = OEM_STRING {
                    Length: (*full_ea).EaNameLength as u16,
                    MaximumLength: 0,
                    Buffer: core::ptr::addr_of!((*full_ea).EaName[0]) as *mut i8,
                };

                if FatIsEaNameValid(irp_context, ea_name) == 0 {
                    DebugTrace(
                        0,
                        Dbg,
                        b"FatCreateEa:  Invalid Ea Name -> %Z\n\0".as_ptr() as *const i8,
                        &ea_name,
                    );
                    (*(*irp_context).OriginatingIrp).IoStatus.Information =
                        ((full_ea as *const u8).offset_from(buffer as *const u8) as usize) as _;
                    (*(*irp_context).OriginatingIrp).IoStatus.__bindgen_anon_1.Status = STATUS_INVALID_EA_NAME;
                    FatRaiseStatus(irp_context, STATUS_INVALID_EA_NAME);
                }

                if (*full_ea).Flags != 0 && (*full_ea).Flags != FILE_NEED_EA {
                    (*(*irp_context).OriginatingIrp).IoStatus.Information =
                        ((full_ea as *const u8).offset_from(buffer as *const u8) as usize) as _;
                    (*(*irp_context).OriginatingIrp).IoStatus.__bindgen_anon_1.Status = STATUS_INVALID_EA_NAME;
                    FatRaiseStatus(irp_context, STATUS_INVALID_EA_NAME);
                }

                let mut ea_offset: ULONG = 0;
                let mut ea_name_copy = ea_name;
                if FatLocateEaByName(
                    irp_context,
                    core::ptr::addr_of_mut!((*(*ea_set_header_ptr)).PackedEas) as *mut PACKED_EA,
                    packed_eas_length,
                    &mut ea_name_copy as *mut OEM_STRING,
                    &mut ea_offset,
                ) != 0 {
                    DebugTrace(
                        0,
                        Dbg,
                        b"FatCreateEa:  Duplicate name found\n\0".as_ptr() as *const i8,
                    );
                    FatDeletePackedEa(
                        irp_context,
                        *ea_set_header_ptr,
                        &mut packed_eas_length,
                        ea_offset,
                    );
                }

                if (*full_ea).EaValueLength == 0 {
                    DebugTrace(
                        0,
                        Dbg,
                        b"FatCreateEa:  Empty ea\n\0".as_ptr() as *const i8,
                    );
                    // advance full_ea for the next iteration
                    if (*full_ea).NextEntryOffset == 0 {
                        full_ea = buffer_end;
                    } else {
                        full_ea = (full_ea as *const u8)
                            .add((*full_ea).NextEntryOffset as usize)
                            as *const FILE_FULL_EA_INFORMATION;
                    }
                    continue;
                }

                FatAppendPackedEa(
                    irp_context,
                    ea_set_header_ptr,
                    core::ptr::addr_of_mut!(packed_eas_length),
                    core::ptr::addr_of_mut!(allocation_length),
                    full_ea as *mut FILE_FULL_EA_INFORMATION,
                    bytes_per_cluster,
                );

                // Advance to next FULL_EA
                if (*full_ea).NextEntryOffset == 0 {
                    full_ea = buffer_end;
                } else {
                    full_ea = (full_ea as *const u8)
                        .add((*full_ea).NextEntryOffset as usize)
                        as *const FILE_FULL_EA_INFORMATION;
                }
            }

            // ---------- after loop ----------
            if packed_eas_length != 0 {
                if packed_eas_length + 4 > MAXIMUM_EA_SIZE {
                    DebugTrace(
                        0,
                        Dbg,
                        b"Ea length is greater than maximum\n\0".as_ptr() as *const i8,
                    );
                    FatRaiseStatus(irp_context, STATUS_EA_TOO_LARGE);
                }

                let mut ea_dirent: PDIRENT = core::ptr::null_mut();
                FatGetEaFile(
                    irp_context,
                    vcb,
                    &mut ea_dirent,
                    &mut *p_ea_bcb,
                    1, /* TRUE */
                    1, /* TRUE */
                );
                *p_locked_ea_fcb = 1;

                FatAddEaSet(
                    irp_context,
                    vcb,
                    packed_eas_length + size_of_ea_set_header,
                    (*p_ea_bcb).cast(),
                    ea_dirent,
                    ea_handle,
                    ea_set_range_ptr,
                );

                let new_ea_set_header: *mut EA_SET_HEADER =
                    (*ea_set_range_ptr).Data as *mut EA_SET_HEADER;
                packed_eas_length += 4;

                // CopyU4char( EaSetHeader->cbList, &PackedEasLength ) -- cbList is unaligned.
                (**ea_set_header_ptr).cbList = packed_eas_length.to_le_bytes();

                let need_ea_count_offset = core::mem::offset_of!(EA_SET_HEADER, NeedEaCount);
                let src = (*ea_set_header_ptr as *const u8).add(need_ea_count_offset);
                let dst = (new_ea_set_header as *mut u8).add(need_ea_count_offset);
                let copy_size = packed_eas_length as usize + size_of_ea_set_header as usize - 8;
                core::ptr::copy_nonoverlapping(src, dst, copy_size);

                FatMarkEaRangeDirty(
                    irp_context,
                    (*vcb).VirtualEaFile,
                    ea_set_range_ptr,
                );
                FatUnpinEaRange(irp_context, (ea_set_range_ptr).cast());
                CcFlushCache(
                    ((*(*vcb).VirtualEaFile).SectionObjectPointer).cast(),
                    core::ptr::null_mut(),
                    0,
                    core::ptr::null_mut(),
                );
            } else {
                *ea_handle = 0;
            }
            // ---------- end of _SEH2_TRY ----------
        }
    }, |_, _abnormal| {
        // ---------- _SEH2_FINALLY ----------
        DebugUnwind(b"FatCreateEa\0".as_ptr() as usize);

        if !(*ea_set_header_ptr).is_null() {
            ExFreePool(*ea_set_header_ptr as *mut c_void);
        }

        if *p_locked_ea_fcb != 0 {
            FatReleaseFcb(irp_context, (*vcb).EaFcb as *mut FCB);
        }

        FatUnpinBcb(irp_context, p_ea_bcb as *mut *mut core::ffi::c_void);
        FatUnpinEaRange(irp_context, (ea_set_range_ptr).cast());

        DebugTrace(-1, Dbg, b"FatCreateEa -> Exit\n\0".as_ptr() as *const i8);
    });
    // ---------- _SEH2_END ----------
}
#[no_mangle]
pub unsafe extern "system" fn FatDeleteEa(
    IrpContext: *mut IRP_CONTEXT,
    Vcb: *mut VCB,
    EaHandle: u16,
    FileName: *mut OEM_STRING,
) {
    let mut EaBcb: *mut BCB = core::ptr::null_mut();
    let mut LockedEaFcb: i32 = 0; // BOOLEAN

    PAGED_CODE!();

    DebugTrace(1, Dbg, b"FatDeleteEa...\n\0".as_ptr() as *const i8, 0);

    EaBcb = core::ptr::null_mut();
    LockedEaFcb = 0;

    let p_EaBcb = core::ptr::addr_of_mut!(EaBcb);
    let p_LockedEaFcb = core::ptr::addr_of_mut!(LockedEaFcb);
    // _SEH2_TRY / _SEH2_FINALLY
    crate::seh::try_finally(&mut (), |_| {
        let mut EaDirent: *mut DIRENT = core::ptr::null_mut();

        FatGetEaFile(
            IrpContext as *mut IRP_CONTEXT,
            Vcb as *mut VCB,
            core::ptr::addr_of_mut!(EaDirent),
            core::ptr::addr_of_mut!(*p_EaBcb),
            0, // FALSE
            1, // TRUE
        );

        *p_LockedEaFcb = 1;

        if (*p_EaBcb).is_null() {
            DebugTrace(
                0,
                Dbg,
                b"FatDeleteEa:  No Ea file exists\n\0".as_ptr() as *const i8,
                0,
            );
            FatRaiseStatus(IrpContext as *mut IRP_CONTEXT, STATUS_NO_EAS_ON_FILE);
        }

        FatDeleteEaSet(
            IrpContext as *mut IRP_CONTEXT,
            Vcb as *mut VCB,
            (*p_EaBcb),
            EaDirent,
            EaHandle,
            FileName,
        );

        CcFlushCache(
            (*(* (Vcb as *mut VCB) ).VirtualEaFile).SectionObjectPointer as *mut c_void,
            core::ptr::null_mut(),
            0,
            core::ptr::null_mut(),
        );
    }, |_, _abnormal| {
        DebugUnwind(b"FatDeleteEa\0".as_ptr() as usize);

        if (*p_LockedEaFcb) != 0 {
            FatReleaseFcb(
                IrpContext as *mut IRP_CONTEXT,
                (* (Vcb as *mut VCB) ).EaFcb,
            );
        }

        FatUnpinBcb(IrpContext as *mut IRP_CONTEXT, core::ptr::addr_of_mut!(*p_EaBcb) as *mut *mut core::ffi::c_void);

        DebugTrace(-1, Dbg, b"FatDeleteEa -> Exit\n\0".as_ptr() as *const i8, 0);
    });

    return;
}

#[no_mangle]
pub unsafe extern "system" fn FatGetEaFile(
    IrpContext: *mut IRP_CONTEXT,
    Vcb: *mut VCB,
    EaDirent: *mut *mut DIRENT,
    EaBcb: *mut *mut BCB,
    CreateFile: u8,
    ExclusiveFcb: u8,
) {
    let mut EaStreamFile: *mut FILE_OBJECT = core::ptr::null_mut();
    let mut EaFileRange: EA_RANGE = core::mem::zeroed();

    let mut UnwindLockedEaFcb: u8 = 0;
    let mut UnwindLockedRootDcb: u8 = 0;
    let mut UnwindAllocatedDiskSpace: u8 = 0;
    let mut UnwindEaDirentCreated: u8 = 0;
    let mut UnwindUpdatedSizes: u8 = 0;

    // PAGED_CODE(); — pageability directive, ignored in Rust

    DebugTrace(1, DEBUG_TRACE_EA, b"FatGetEaFile ...\n\0".as_ptr() as *const i8, 0);

    RtlZeroMemory(
        core::ptr::addr_of_mut!(EaFileRange) as *mut c_void,
        core::mem::size_of::<EA_RANGE>(),
    );

    let p_EaStreamFile = core::ptr::addr_of_mut!(EaStreamFile);
    let p_EaFileRange = core::ptr::addr_of_mut!(EaFileRange);
    let p_UnwindLockedEaFcb = core::ptr::addr_of_mut!(UnwindLockedEaFcb);
    let p_UnwindLockedRootDcb = core::ptr::addr_of_mut!(UnwindLockedRootDcb);
    let p_UnwindAllocatedDiskSpace = core::ptr::addr_of_mut!(UnwindAllocatedDiskSpace);
    let p_UnwindEaDirentCreated = core::ptr::addr_of_mut!(UnwindEaDirentCreated);
    let p_UnwindUpdatedSizes = core::ptr::addr_of_mut!(UnwindUpdatedSizes);
    // _SEH2_TRY / _SEH2_FINALLY
    crate::seh::try_finally(&mut (), |_| {
        let mut EaFileName: OEM_STRING = unsafe { core::mem::zeroed() };
        let mut SectionSize: LARGE_INTEGER = unsafe { core::mem::zeroed() };

        // Check if the Vcb already has the file object
        if (*Vcb).VirtualEaFile.is_null() {
            // Always lock the Ea file exclusively if we have to create the file
            if FatAcquireExclusiveFcb(IrpContext, (*Vcb).EaFcb) == 0 {
                DebugTrace(
                    0,
                    DEBUG_TRACE_EA,
                    b"FatGetEaFile:  Can't grab exclusive\n\0".as_ptr() as *const i8,
                    0,
                );
                FatRaiseStatus(IrpContext, STATUS_CANT_WAIT);
            }
            *p_UnwindLockedEaFcb = 1;
        } else {
            // Otherwise we acquire the Fcb as the caller requested
            if (ExclusiveFcb != 0
                && FatAcquireExclusiveFcb(IrpContext, (*Vcb).EaFcb) == 0)
                || (ExclusiveFcb == 0
                    && FatAcquireSharedFcb(IrpContext, (*Vcb).EaFcb) == 0)
            {
                DebugTrace(
                    0,
                    DEBUG_TRACE_EA,
                    b"FatGetEaFile:  Can't grab EaFcb\n\0".as_ptr() as *const i8,
                    0,
                );
                FatRaiseStatus(IrpContext, STATUS_CANT_WAIT);
            }
            *p_UnwindLockedEaFcb = 1;

            // If the file now does not exist we need to release the Fcb and
            // reacquire exclusive if we acquired shared.
            if (*Vcb).VirtualEaFile.is_null() && ExclusiveFcb == 0 {
                FatReleaseFcb(IrpContext, (*Vcb).EaFcb);
                *p_UnwindLockedEaFcb = 0;

                if FatAcquireExclusiveFcb(IrpContext, (*Vcb).EaFcb) == 0 {
                    DebugTrace(
                        0,
                        DEBUG_TRACE_EA,
                        b"FatGetEaFile:  Can't grab EaFcb\n\0".as_ptr() as *const i8,
                        0,
                    );
                    FatRaiseStatus(IrpContext, STATUS_CANT_WAIT);
                }
                *p_UnwindLockedEaFcb = 1;
            }
        }

        // If the file object is now there we only need to get the dirent
        if !(*Vcb).VirtualEaFile.is_null() {
            FatVerifyFcb(IrpContext, (*Vcb).EaFcb);
            FatGetDirentFromFcbOrDcb(
                IrpContext,
                (*Vcb).EaFcb,
                0,
                EaDirent,
                EaBcb,
            );
            // try_return( NOTHING );
            return;
        } else {
            let mut ByteOffset: u32 = 0;

            // Always mark the ea fcb as good
            (*(*Vcb).EaFcb).FcbCondition = FcbGood;

            // We try to lookup the dirent for the Ea Fcb
            EaFileName = OEM_STRING {
                Length: 11,
                MaximumLength: 12,
                Buffer: b"EA DATA. SF\0".as_ptr() as *mut i8,
            };

            // Now pick up the root directory
            if CreateFile != 0 {
                ExAcquireResourceExclusiveLite(
                    // C passes the PERESOURCE itself, not the address of the Header field.
                    (*(*Vcb).RootDcb).Header.Resource.cast(),
                    1,
                );
            } else {
                ExAcquireResourceSharedLite(
                    (*(*Vcb).RootDcb).Header.Resource.cast(),
                    1,
                );
            }
            *p_UnwindLockedRootDcb = 1;

            FatLocateSimpleOemDirent(
                IrpContext,
                (*(*Vcb).EaFcb).ParentDcb,
                core::ptr::addr_of_mut!(EaFileName),
                EaDirent,
                EaBcb,
                core::ptr::addr_of_mut!(ByteOffset),
            );

            // If the file exists, we need to create the virtual file object
            if !(*EaDirent).is_null() {
                // Pin the data now
                FatPinMappedData(
                    IrpContext,
                    (*(*Vcb).EaFcb).ParentDcb,
                    ByteOffset,
                    core::mem::size_of::<DIRENT>() as u32,
                    EaBcb,
                );

                // Update the Fcb with information
                (*(*Vcb).EaFcb).FirstClusterOfFile = ((*(*EaDirent)).FirstClusterOfFile) as u32;
                (*(*Vcb).EaFcb).DirentOffsetWithinDirectory = ByteOffset;

                // Find the allocation size
                (*(*Vcb).EaFcb).Header.AllocationSize.QuadPart =
                    FCB_LOOKUP_ALLOCATIONSIZE_HINT;
                FatLookupFileAllocationSize(IrpContext, (*Vcb).EaFcb);

                // Compute section size
                SectionSize.QuadPart = ((*(*EaDirent)).FileSize) as i64;
                (*(*Vcb).EaFcb).Header.AllocationSize = SectionSize;
                (*(*Vcb).EaFcb).Header.FileSize = SectionSize;

                // Create and initialize the file object
                *p_EaStreamFile = FatOpenEaFile(IrpContext, (*Vcb).EaFcb);
                (*Vcb).VirtualEaFile = (*p_EaStreamFile);
            } else if CreateFile != 0 {
                // Else there was no dirent, create it
                let BytesPerCluster: u32;
                let OffsetTableSize: u32;
                let mut AllocationSize: u32 = 0;
                let FileHeader: *mut EA_FILE_HEADER;
                let AllocatedClusters: u16;
                let mut CurrentIndex: *mut u16 = core::ptr::null_mut();
                let mut Index: u32 = 0;
                let mut Status: i32 = STATUS_SUCCESS;

                DebugTrace(
                    0,
                    DEBUG_TRACE_EA,
                    b"FatGetEaFile:  Creating local IrpContext\n\0".as_ptr()
                        as *const i8,
                    0,
                );

                BytesPerCluster =
                    1u32 << (*Vcb).AllocationSupport.LogOfBytesPerCluster;

                AllocationSize = ((core::mem::size_of::<EA_FILE_HEADER>() as u32) << 1)
                    .wrapping_add(BytesPerCluster)
                    .wrapping_sub(1)
                    & !(BytesPerCluster.wrapping_sub(1));

                AllocatedClusters = (AllocationSize
                    >> (*Vcb).AllocationSupport.LogOfBytesPerCluster)
                    as u16;

                OffsetTableSize =
                    AllocationSize - core::mem::size_of::<EA_FILE_HEADER>() as u32;

                // Allocate disk space
                FatAllocateDiskSpace(
                    IrpContext,
                    Vcb,
                    0,
                    core::ptr::addr_of_mut!(AllocationSize),
                    0,
                    core::ptr::addr_of_mut!((*(*Vcb).EaFcb).Mcb),
                );
                *p_UnwindAllocatedDiskSpace = 1;

                // Allocate and initialize a dirent in the root directory
                (*(*Vcb).EaFcb).DirentOffsetWithinDirectory = FatCreateNewDirent(
                    IrpContext,
                    (*(*Vcb).EaFcb).ParentDcb,
                    1,
                    0,
                );

                FatPrepareWriteDirectoryFile(
                    IrpContext,
                    (*(*Vcb).EaFcb).ParentDcb,
                    (*(*Vcb).EaFcb).DirentOffsetWithinDirectory,
                    core::mem::size_of::<DIRENT>() as u32,
                    EaBcb,
                    EaDirent,
                    0,
                    1,
                    core::ptr::addr_of_mut!(Status),
                );

                NT_ASSERT(NT_SUCCESS(Status) as i32);

                *p_UnwindEaDirentCreated = 1;

                FatConstructDirent(
                    IrpContext,
                    *EaDirent,
                    core::ptr::addr_of_mut!(EaFileName),
                    0,
                    0,
                    core::ptr::null_mut(),
                    (FAT_DIRENT_ATTR_READ_ONLY
                        | FAT_DIRENT_ATTR_HIDDEN
                        | FAT_DIRENT_ATTR_SYSTEM
                        | FAT_DIRENT_ATTR_ARCHIVE) as _,
                    1,
                    core::ptr::null_mut(),
                );

                (*(*EaDirent)).FileSize = (AllocationSize as i64) as u32;

                // Initialize the Fcb for this file and the cache map
                SectionSize.QuadPart = ((*(*EaDirent)).FileSize) as i64;
                (*(*Vcb).EaFcb).Header.AllocationSize = SectionSize;
                (*(*Vcb).EaFcb).Header.FileSize = SectionSize;
                *p_UnwindUpdatedSizes = 1;

                // Create and initialize the file object
                *p_EaStreamFile = FatOpenEaFile(IrpContext, (*Vcb).EaFcb);

                // Update the Fcb with disk location
                {
                    let mut FirstLboOfFile: i64 = 0;
                    FatLookupMcbEntry(
                        Vcb,
                        core::ptr::addr_of_mut!((*(*Vcb).EaFcb).Mcb),
                        0,
                        core::ptr::addr_of_mut!(FirstLboOfFile),
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                    );

                    (*(*EaDirent)).FirstClusterOfFile =
                        FatGetIndexFromLbo(Vcb, FirstLboOfFile);
                }

                (*(*Vcb).EaFcb).FirstClusterOfFile =
                    ((*(*EaDirent)).FirstClusterOfFile) as u32;

                // Initialize the Ea file header and mark the Bcb as dirty
                FatPinEaRange(
                    IrpContext,
                    (*p_EaStreamFile),
                    (*Vcb).EaFcb,
                    core::ptr::addr_of_mut!(*p_EaFileRange),
                    0,
                    AllocationSize,
                    STATUS_DATA_ERROR,
                );

                FileHeader = (*p_EaFileRange).Data as *mut EA_FILE_HEADER;

                RtlZeroMemory(
                    FileHeader as *mut c_void,
                    AllocationSize as usize,
                );
                (*FileHeader).Signature = (EA_FILE_SIGNATURE) as u16;

                Index = MAX_EA_BASE_INDEX;
                CurrentIndex =
                    core::ptr::addr_of_mut!((*FileHeader).EaBaseTable) as *mut u16;
                while Index != 0 {
                    *CurrentIndex = AllocatedClusters;
                    Index = Index.wrapping_sub(1);
                    CurrentIndex = CurrentIndex.add(1);
                }

                // Initialize the offset table
                Index = OffsetTableSize >> 1;
                CurrentIndex = (FileHeader as *mut u8)
                    .add(core::mem::size_of::<EA_FILE_HEADER>()) as *mut u16;
                while Index != 0 {
                    *CurrentIndex = UNUSED_EA_HANDLE;
                    Index = Index.wrapping_sub(1);
                    CurrentIndex = CurrentIndex.add(1);
                }

                // Unpin the file header and offset table
                FatMarkEaRangeDirty(IrpContext, (*p_EaStreamFile), core::ptr::addr_of_mut!(*p_EaFileRange));
                FatUnpinEaRange(IrpContext, (core::ptr::addr_of_mut!(*p_EaFileRange)).cast());

                CcFlushCache(
                    ((**p_EaStreamFile).SectionObjectPointer).cast(),
                    core::ptr::null_mut(),
                    0,
                    core::ptr::null_mut(),
                );

                // Return the Ea file object
                (*Vcb).VirtualEaFile = (*p_EaStreamFile);
            }
        }
    }, |_, abnormal| {
        DebugUnwind(FatGetEaFile as usize);

        // If abnormal termination and disk space allocated, deallocate it
        if abnormal {
            // Deallocate the Ea file
            if (*p_UnwindAllocatedDiskSpace) != 0 {
                FatDeallocateDiskSpace(
                    IrpContext,
                    Vcb,
                    core::ptr::addr_of_mut!((*(*Vcb).EaFcb).Mcb),
                    0,
                );
            }

            // Delete the dirent for the Ea file, if created
            if (*p_UnwindEaDirentCreated) != 0 {
                if (*p_UnwindUpdatedSizes) != 0 {
                    (*(*Vcb).EaFcb).Header.AllocationSize.QuadPart = 0;
                    (*(*Vcb).EaFcb).Header.FileSize.QuadPart = 0;
                }

                FatUnpinBcb(IrpContext, core::ptr::addr_of_mut!(*EaBcb) as *mut *mut core::ffi::c_void);
                FatDeleteDirent(IrpContext, (*Vcb).EaFcb, core::ptr::null_mut(), 1);
            }

            // Release the EA Fcb if held
            if (*p_UnwindLockedEaFcb) != 0 {
                FatReleaseFcb(IrpContext, (*Vcb).EaFcb);
            }

            // Dereference the Ea stream file if created
            if !(*p_EaStreamFile).is_null() {
                ObDereferenceObject((*p_EaStreamFile) as *mut core::ffi::c_void);
            }
        }

        // Always release the root Dcb
        if (*p_UnwindLockedRootDcb) != 0 {
            FatReleaseFcb(IrpContext, (*Vcb).RootDcb);
        }

        // If the Ea file header is locked down, unpin it now
        FatUnpinEaRange(IrpContext, (core::ptr::addr_of_mut!(*p_EaFileRange)).cast());

        DebugTrace(
            -1,
            DEBUG_TRACE_EA,
            b"FatGetEaFile:  Exit\n\0".as_ptr() as *const i8,
            0,
        );
    });
}

#[no_mangle]
pub unsafe extern "system" fn FatReadEaSet(
    IrpContext: *mut IRP_CONTEXT,
    Vcb: *mut VCB,
    EaHandle: u16,
    FileName: *const OEM_STRING,
    ReturnEntireSet: u8,
    EaSetRange: *mut EA_RANGE,
) {
    let bytes_per_cluster: u32 = 1u32 << ((*Vcb).AllocationSupport.LogOfBytesPerCluster as u32);
    let mut ea_offset_vbo: u32;
    let mut ea_header_range: EA_RANGE = core::mem::zeroed();
    let mut ea_offset_range: EA_RANGE = core::mem::zeroed();
    let mut ea_offset_cluster: u16 = 0;
    let mut ea_set_vbo: u32 = 0;
    let mut cb_list: u32 = 0;

    core::ptr::write_bytes(core::ptr::addr_of_mut!(ea_header_range) as *mut u8, 0, core::mem::size_of::<EA_RANGE>());
    core::ptr::write_bytes(core::ptr::addr_of_mut!(ea_offset_range) as *mut u8, 0, core::mem::size_of::<EA_RANGE>());

    let _ = &FileName;

    DebugTrace(1, Dbg, (b"FatReadEaSet\n\0").as_ptr() as *const core::ffi::c_char, 0);
    DebugTrace(0, Dbg, (b"  Vcb      = %p\n\0").as_ptr() as *const core::ffi::c_char, 0, Vcb);

    if EaHandle < 1 /* MIN_EA_HANDLE */ || EaHandle > 0x77FF /* MAX_EA_HANDLE */ {
        DebugTrace(-1, Dbg, (b"FatReadEaSet: Illegal handle value\n\0").as_ptr() as *const core::ffi::c_char, 0);
        FatRaiseStatus(IrpContext, 0xC0000051_u32 as i32); // STATUS_NONEXISTENT_EA_ENTRY
    }

    ea_offset_vbo = core::mem::size_of::<EA_FILE_HEADER>() as u32 + ((((EaHandle as u64) >> 7) << 8) as u32);

    let p_ea_header_range = core::ptr::addr_of_mut!(ea_header_range);
    let p_ea_offset_range = core::ptr::addr_of_mut!(ea_offset_range);
    // _SEH2_TRY / _SEH2_FINALLY
    crate::seh::try_finally(&mut (), |_| {
        FatPinEaRange(
            IrpContext,
            (*Vcb).VirtualEaFile,
            (*Vcb).EaFcb,
            core::ptr::addr_of_mut!(*p_ea_header_range),
            0,
            core::mem::size_of::<EA_FILE_HEADER>() as u32,
            0xC0000051_u32 as i32 /* STATUS_NONEXISTENT_EA_ENTRY */,
        );

        let ea_header: *mut EA_FILE_HEADER = (*p_ea_header_range).Data as *mut EA_FILE_HEADER;

        FatPinEaRange(
            IrpContext,
            (*Vcb).VirtualEaFile,
            (*Vcb).EaFcb,
            core::ptr::addr_of_mut!(*p_ea_offset_range),
            ea_offset_vbo,
            core::mem::size_of::<EA_OFF_TABLE>() as u32,
            0xC0000051_u32 as i32 /* STATUS_NONEXISTENT_EA_ENTRY */,
        );

        ea_offset_cluster = *(((*p_ea_offset_range).Data as *mut u16).add((EaHandle & (128 /* MAX_EA_OFFSET_INDEX */ - 1)) as usize));

        if ea_offset_cluster == 0xFFFF /* UNUSED_EA_HANDLE */ {
            DebugTrace(0, Dbg, (b"FatReadEaSet: Ea handle is unused\n\0").as_ptr() as *const core::ffi::c_char, 0);
            FatRaiseStatus(IrpContext, 0xC0000051_u32 as i32 /* STATUS_NONEXISTENT_EA_ENTRY */);
        }

        ea_set_vbo = ((*ea_header).EaBaseTable[(EaHandle >> 7) as usize] as u32 + ea_offset_cluster as u32)
                      << (*Vcb).AllocationSupport.LogOfBytesPerCluster as u32;

        FatUnpinEaRange(IrpContext, (core::ptr::addr_of_mut!(*p_ea_header_range)).cast());
        FatUnpinEaRange(IrpContext, (core::ptr::addr_of_mut!(*p_ea_offset_range)).cast());

        FatPinEaRange(
            IrpContext,
            (*Vcb).VirtualEaFile,
            (*Vcb).EaFcb,
            EaSetRange,
            ea_set_vbo,
            bytes_per_cluster,
            0xC000003E_u32 as i32 /* STATUS_DATA_ERROR */,
        );

        let ea_set: *mut EA_SET_HEADER = (*EaSetRange).Data as *mut EA_SET_HEADER;

        if (*ea_set).Signature != 0x4145 /* EA_SET_SIGNATURE */ || (*ea_set).OwnEaHandle != EaHandle {
            DebugTrace(0, Dbg, (b"FatReadEaSet: Ea set header is corrupt\n\0").as_ptr() as *const core::ffi::c_char, 0);
            FatRaiseStatus(IrpContext, 0xC000003E_u32 as i32 /* STATUS_DATA_ERROR */);
        }

        cb_list = GetcbList(ea_set) as u32;

        if ReturnEntireSet != 0 && cb_list > bytes_per_cluster {
            cb_list = (cb_list + 0x1a /* EA_CBLIST_OFFSET */ + bytes_per_cluster - 1) & !(bytes_per_cluster - 1) ;

            FatUnpinEaRange(IrpContext, (EaSetRange).cast());

            core::ptr::write_bytes(EaSetRange as *mut u8, 0, core::mem::size_of::<EA_RANGE>());

            FatPinEaRange(
                IrpContext,
                (*Vcb).VirtualEaFile,
                (*Vcb).EaFcb,
                EaSetRange,
                ea_set_vbo,
                cb_list,
                0xC000003E_u32 as i32 /* STATUS_DATA_ERROR */,
            );
        }
    }, |_, _abnormal| {
        DebugUnwind(b"FatReadEaSet\0".as_ptr() as usize);

        FatUnpinEaRange(IrpContext, (core::ptr::addr_of_mut!(*p_ea_header_range)).cast());
        FatUnpinEaRange(IrpContext, (core::ptr::addr_of_mut!(*p_ea_offset_range)).cast());

        DebugTrace(-1, Dbg, (b"FatReadEaSet:  Exit\n\0").as_ptr() as *const core::ffi::c_char, 0);
    });
}
#[no_mangle]
pub unsafe extern "system" fn FatDeleteEaSet(
    IrpContext: *mut IRP_CONTEXT,
    Vcb: *mut VCB,
    EaBcb: *mut BCB,
    EaDirent: *mut DIRENT,
    EaHandle: u16,
    FileName: *mut OEM_STRING,
) {
    let mut BytesPerCluster: u32 = 1u32 << (*Vcb).AllocationSupport.LogOfBytesPerCluster;
    let mut CbList: u32 = 0;
    let mut FileOffset: LARGE_INTEGER = unsafe { core::mem::zeroed() };

    let mut DataMcb: LARGE_MCB = unsafe { core::mem::zeroed() };
    let mut UnwindInitializeDataMcb: i32 = 0;
    let mut UnwindSplitData: i32 = 0;

    let mut TailMcb: LARGE_MCB = unsafe { core::mem::zeroed() };
    let mut UnwindInitializeTailMcb: i32 = 0;
    let mut UnwindSplitTail: i32 = 0;
    let mut UnwindMergeTail: i32 = 0;

    let mut UnwindModifiedEaHeader: i32 = 0;
    let mut UnwindCacheValues: i32 = 0;
    let mut UnwindPrevFileSize: u32 = 0;

    let mut EaOffsetVbo: u32;
    let mut EaOffsetIndex: u16 = 0;
    let mut EaOffsetRange: EA_RANGE = unsafe { core::mem::zeroed() };
    let mut EaOffsetCluster: u16 = 0;

    let VirtualEaFile = (*Vcb).VirtualEaFile;
    let EaFcb = (*Vcb).EaFcb;

    let mut EaHeaderRange: EA_RANGE = unsafe { core::mem::zeroed() };
    let mut EaHeader: *mut EA_FILE_HEADER = core::ptr::null_mut();
    let mut EaHeaderBaseIndex: u16 = 0;

    let mut EaSetVbo: u32 = 0;
    let mut EaSetLength: u32 = 0;
    let mut EaSetRange: EA_RANGE = unsafe { core::mem::zeroed() };
    let mut EaSet: *mut EA_SET_HEADER = core::ptr::null_mut();
    let mut EaSetClusterCount: u16 = 0;

    PAGED_CODE!();

    UNREFERENCED_PARAMETER!(FileName);

    if EaHandle < MIN_EA_HANDLE || EaHandle > MAX_EA_HANDLE {
        DebugTrace(-1, Dbg, b"FatDeleteEaSet: Illegal handle value\n\0".as_ptr() as *const i8, 0);
        FatRaiseStatus(IrpContext, STATUS_NONEXISTENT_EA_ENTRY);
    }

    EaOffsetVbo = core::mem::size_of::<EA_FILE_HEADER>() as u32 + (((EaHandle as u64 >> 7) << 8) as u32);

    RtlZeroMemory(core::ptr::addr_of_mut!(EaHeaderRange) as *mut core::ffi::c_void, core::mem::size_of::<EA_RANGE>());
    RtlZeroMemory(core::ptr::addr_of_mut!(EaOffsetRange) as *mut core::ffi::c_void, core::mem::size_of::<EA_RANGE>());
    RtlZeroMemory(core::ptr::addr_of_mut!(EaSetRange) as *mut core::ffi::c_void, core::mem::size_of::<EA_RANGE>());

    let p_EaHeaderRange = core::ptr::addr_of_mut!(EaHeaderRange);
    let p_EaOffsetRange = core::ptr::addr_of_mut!(EaOffsetRange);
    let p_EaSetRange = core::ptr::addr_of_mut!(EaSetRange);
    let p_DataMcb = core::ptr::addr_of_mut!(DataMcb);
    let p_TailMcb = core::ptr::addr_of_mut!(TailMcb);
    let p_UnwindInitializeDataMcb = core::ptr::addr_of_mut!(UnwindInitializeDataMcb);
    let p_UnwindSplitData = core::ptr::addr_of_mut!(UnwindSplitData);
    let p_UnwindInitializeTailMcb = core::ptr::addr_of_mut!(UnwindInitializeTailMcb);
    let p_UnwindSplitTail = core::ptr::addr_of_mut!(UnwindSplitTail);
    let p_UnwindMergeTail = core::ptr::addr_of_mut!(UnwindMergeTail);
    let p_UnwindModifiedEaHeader = core::ptr::addr_of_mut!(UnwindModifiedEaHeader);
    let p_UnwindCacheValues = core::ptr::addr_of_mut!(UnwindCacheValues);
    let p_UnwindPrevFileSize = core::ptr::addr_of_mut!(UnwindPrevFileSize);
    let p_EaSetVbo = core::ptr::addr_of_mut!(EaSetVbo);
    // _SEH2_TRY / _SEH2_FINALLY
    crate::seh::try_finally(&mut (), |_| {
        FatPinEaRange(
            IrpContext,
            VirtualEaFile,
            EaFcb,
            core::ptr::addr_of_mut!(*p_EaHeaderRange),
            0,
            core::mem::size_of::<EA_FILE_HEADER>() as u32,
            STATUS_NONEXISTENT_EA_ENTRY,
        );

        EaHeader = (*p_EaHeaderRange).Data as *mut EA_FILE_HEADER;

        FatPinEaRange(
            IrpContext,
            VirtualEaFile,
            EaFcb,
            core::ptr::addr_of_mut!(*p_EaOffsetRange),
            EaOffsetVbo,
            core::mem::size_of::<EA_OFF_TABLE>() as u32,
            STATUS_NONEXISTENT_EA_ENTRY,
        );

        EaOffsetIndex = EaHandle & ((MAX_EA_OFFSET_INDEX - 1) as u16);
        EaOffsetCluster = *(((*p_EaOffsetRange).Data as *mut u16).add(EaOffsetIndex as usize));

        if EaOffsetCluster == UNUSED_EA_HANDLE {
            DebugTrace(0, Dbg, b"FatReadEaSet: Ea handle is unused\n\0".as_ptr() as *const i8, 0);
            FatRaiseStatus(IrpContext, STATUS_NONEXISTENT_EA_ENTRY);
        }

        EaHeaderBaseIndex = EaHandle >> 7;
        *p_EaSetVbo = ((*EaHeader).EaBaseTable[EaHeaderBaseIndex as usize] as u32 + EaOffsetCluster as u32)
            << (*Vcb).AllocationSupport.LogOfBytesPerCluster;

        FatUnpinEaRange(IrpContext, (core::ptr::addr_of_mut!(*p_EaHeaderRange)).cast());
        FatUnpinEaRange(IrpContext, (core::ptr::addr_of_mut!(*p_EaOffsetRange)).cast());

        FatPinEaRange(
            IrpContext,
            VirtualEaFile,
            EaFcb,
            core::ptr::addr_of_mut!(*p_EaSetRange),
            (*p_EaSetVbo),
            BytesPerCluster,
            STATUS_DATA_ERROR,
        );

        EaSet = (*p_EaSetRange).Data as *mut EA_SET_HEADER;

        if (*EaSet).Signature != (EA_SET_SIGNATURE) as u16 || (*EaSet).OwnEaHandle != EaHandle {
            DebugTrace(0, Dbg, b"FatReadEaSet: Ea set header is corrupt\n\0".as_ptr() as *const i8, 0);
            FatRaiseStatus(IrpContext, STATUS_DATA_ERROR);
        }

        CbList = GetcbList(EaSet) as u32;

        EaSetClusterCount = ((CbList + EA_CBLIST_OFFSET + BytesPerCluster - 1)
            >> (*Vcb).AllocationSupport.LogOfBytesPerCluster) as u16;

        EaSetLength = (EaSetClusterCount as u32) << (*Vcb).AllocationSupport.LogOfBytesPerCluster;

        if EaSetLength > BytesPerCluster {
            if (*EaFcb).Header.FileSize.__bindgen_anon_1.LowPart.wrapping_sub(*p_EaSetVbo) < EaSetLength {
                DebugTrace(0, Dbg, b"FatDeleteEaSet: Full Ea set not contained in file\n\0".as_ptr() as *const i8, 0);
                FatRaiseStatus(IrpContext, STATUS_DATA_ERROR);
            }
        }

        FatUnpinEaRange(IrpContext, (core::ptr::addr_of_mut!(*p_EaSetRange)).cast());

        FileOffset.__bindgen_anon_1.LowPart = (*p_EaSetVbo);
        FileOffset.__bindgen_anon_1.HighPart = 0;

        FileOffset.__bindgen_anon_1.LowPart &= !(PAGE_SIZE - 1);

        {
            let mut Iosb: _IO_STATUS_BLOCK = unsafe { core::mem::zeroed() };
            let mut PurgeCount: u32 = 5;

            while {
                PurgeCount = PurgeCount.wrapping_sub(1);
                PurgeCount != 0
            } {
                Iosb.__bindgen_anon_1.Status = STATUS_SUCCESS;

                // CcFlushCache( Sop, NULL /* FileOffset */, 0, &Iosb ): the translation passed
                // the Iosb as the FileOffset.
                CcFlushCache(
                    (*VirtualEaFile).SectionObjectPointer as *mut c_void,
                    core::ptr::null_mut(),
                    0,
                    core::ptr::addr_of_mut!(Iosb),
                );

                NT_ASSERT((Iosb.__bindgen_anon_1.Status == STATUS_SUCCESS) as i32);

                if CcPurgeCacheSection(
                    (*VirtualEaFile).SectionObjectPointer,
                    core::ptr::addr_of_mut!(FileOffset),
                    0,
                    0,
                ) != 0 {
                    break;
                }
            }

            if PurgeCount == 0 {
                FatRaiseStatus(IrpContext, STATUS_UNABLE_TO_DELETE_SECTION);
            }
        }

        FileOffset.__bindgen_anon_1.LowPart = (*EaFcb).Header.FileSize.__bindgen_anon_1.LowPart.wrapping_sub(EaSetLength);

        {
            FsRtlInitializeLargeMcb(core::ptr::addr_of_mut!(*p_DataMcb), PagedPool);

            *p_UnwindInitializeDataMcb = 1;

            FatSplitAllocation(
                IrpContext,
                Vcb,
                &mut (*EaFcb).Mcb,
                (*p_EaSetVbo),
                core::ptr::addr_of_mut!(*p_DataMcb),
            );

            *p_UnwindSplitData = 1;

            if EaSetLength.wrapping_add(*p_EaSetVbo) != (*EaFcb).Header.FileSize.__bindgen_anon_1.LowPart {
                FsRtlInitializeLargeMcb(core::ptr::addr_of_mut!(*p_TailMcb), PagedPool);

                *p_UnwindInitializeTailMcb = 1;

                FatSplitAllocation(
                    IrpContext,
                    Vcb,
                    core::ptr::addr_of_mut!(*p_DataMcb),
                    EaSetLength,
                    core::ptr::addr_of_mut!(*p_TailMcb),
                );

                *p_UnwindSplitTail = 1;

                FatMergeAllocation(
                    IrpContext,
                    Vcb,
                    &mut (*EaFcb).Mcb,
                    core::ptr::addr_of_mut!(*p_TailMcb),
                );

                *p_UnwindMergeTail = 1;
            }
        }

        *p_UnwindPrevFileSize = (*EaFcb).Header.FileSize.__bindgen_anon_1.LowPart;

        ExAcquireResourceExclusiveLite(
            (*EaFcb).Header.PagingIoResource,
            1,
        );

        (*EaFcb).Header.FileSize.__bindgen_anon_1.LowPart = (*EaFcb).Header.FileSize.__bindgen_anon_1.LowPart.wrapping_sub(EaSetLength);
        (*EaFcb).Header.AllocationSize = (*EaFcb).Header.FileSize;

        CcSetFileSizes(
            VirtualEaFile,
            core::ptr::addr_of_mut!((*EaFcb).Header.AllocationSize) as *mut CC_FILE_SIZES,
        );

        ExReleaseResourceLite((*EaFcb).Header.PagingIoResource);

        *p_UnwindCacheValues = 1;

        (*EaDirent).FileSize = (*EaFcb).Header.FileSize.__bindgen_anon_1.LowPart;

        FatSetDirtyBcb(IrpContext, EaBcb, Vcb, 1);

        RtlZeroMemory(core::ptr::addr_of_mut!(*p_EaHeaderRange) as *mut core::ffi::c_void, core::mem::size_of::<EA_RANGE>());

        FatPinEaRange(
            IrpContext,
            VirtualEaFile,
            EaFcb,
            core::ptr::addr_of_mut!(*p_EaHeaderRange),
            0,
            core::mem::size_of::<EA_FILE_HEADER>() as u32,
            STATUS_NONEXISTENT_EA_ENTRY,
        );

        EaHeader = (*p_EaHeaderRange).Data as *mut EA_FILE_HEADER;

        RtlZeroMemory(core::ptr::addr_of_mut!(*p_EaOffsetRange) as *mut core::ffi::c_void, core::mem::size_of::<EA_RANGE>());

        FatPinEaRange(
            IrpContext,
            VirtualEaFile,
            EaFcb,
            core::ptr::addr_of_mut!(*p_EaOffsetRange),
            EaOffsetVbo,
            core::mem::size_of::<EA_OFF_TABLE>() as u32,
            STATUS_NONEXISTENT_EA_ENTRY,
        );

        {
            let mut Count: u32 = 0;
            let mut NextEaIndex: *mut u16 = core::ptr::null_mut();

            Count = MAX_EA_BASE_INDEX - EaHeaderBaseIndex as u32 - 1;

            NextEaIndex = core::ptr::addr_of_mut!((*EaHeader).EaBaseTable[EaHeaderBaseIndex as usize + 1]);

            while Count != 0 {
                *NextEaIndex = (*NextEaIndex).wrapping_sub(EaSetClusterCount);
                NextEaIndex = NextEaIndex.add(1);
                Count = Count.wrapping_sub(1);
            }

            FatMarkEaRangeDirty(IrpContext, VirtualEaFile, core::ptr::addr_of_mut!(*p_EaHeaderRange));

            Count = MAX_EA_OFFSET_INDEX - EaOffsetIndex as u32 - 1;
            NextEaIndex = ((*p_EaOffsetRange).Data as *mut u16).add(EaOffsetIndex as usize);

            *NextEaIndex = UNUSED_EA_HANDLE;
            NextEaIndex = NextEaIndex.add(1);

            while Count != 0 {
                if *NextEaIndex != UNUSED_EA_HANDLE {
                    *NextEaIndex = (*NextEaIndex).wrapping_sub(EaSetClusterCount);
                }
                NextEaIndex = NextEaIndex.add(1);
                Count = Count.wrapping_sub(1);
            }

            FatMarkEaRangeDirty(IrpContext, VirtualEaFile, core::ptr::addr_of_mut!(*p_EaOffsetRange));
        }

        *p_UnwindModifiedEaHeader = 1;

        FatDeallocateDiskSpace(
            IrpContext,
            Vcb,
            core::ptr::addr_of_mut!(*p_DataMcb),
            0,
        );
    }, |_, abnormal| {
        DebugUnwind(b"FatDeleteEaSet\0".as_ptr() as usize);

        if abnormal && (*p_UnwindModifiedEaHeader) == 0 {
            if (*p_UnwindPrevFileSize) != 0 {
                (*EaFcb).Header.FileSize.__bindgen_anon_1.LowPart = (*p_UnwindPrevFileSize);
                (*EaFcb).Header.AllocationSize.__bindgen_anon_1.LowPart = (*p_UnwindPrevFileSize);
                (*EaDirent).FileSize = (*p_UnwindPrevFileSize);

                if (*p_UnwindCacheValues) != 0 {
                    CcSetFileSizes(
                        VirtualEaFile,
                        core::ptr::addr_of_mut!((*EaFcb).Header.AllocationSize) as *mut CC_FILE_SIZES,
                    );
                }
            }

            if (*p_UnwindMergeTail) != 0 {
                FatSplitAllocation(
                    IrpContext,
                    Vcb,
                    &mut (*EaFcb).Mcb,
                    (*p_EaSetVbo),
                    core::ptr::addr_of_mut!(*p_TailMcb),
                );
            }

            if (*p_UnwindSplitTail) != 0 {
                FatMergeAllocation(
                    IrpContext,
                    Vcb,
                    core::ptr::addr_of_mut!(*p_DataMcb),
                    core::ptr::addr_of_mut!(*p_TailMcb),
                );
            }

            if (*p_UnwindSplitData) != 0 {
                FatMergeAllocation(
                    IrpContext,
                    Vcb,
                    &mut (*EaFcb).Mcb,
                    core::ptr::addr_of_mut!(*p_DataMcb),
                );
            }
        }

        FatUnpinEaRange(IrpContext, (core::ptr::addr_of_mut!(*p_EaHeaderRange)).cast());
        FatUnpinEaRange(IrpContext, (core::ptr::addr_of_mut!(*p_EaOffsetRange)).cast());
        FatUnpinEaRange(IrpContext, (core::ptr::addr_of_mut!(*p_EaSetRange)).cast());

        if (*p_UnwindInitializeDataMcb) != 0 {
            FsRtlUninitializeLargeMcb(core::ptr::addr_of_mut!(*p_DataMcb));
        }

        if (*p_UnwindInitializeTailMcb) != 0 {
            FsRtlUninitializeLargeMcb(core::ptr::addr_of_mut!(*p_TailMcb));
        }

        DebugTrace(-1, Dbg, b"FatDeleteEaSet -> Exit\n\0".as_ptr() as *const i8, 0);
    });
}
#[no_mangle]
pub unsafe extern "system" fn FatAddEaSet(
    IrpContext: PIRP_CONTEXT,
    Vcb: PVCB,
    mut EaSetLength: u32,
    EaBcb: PBCB,
    EaDirent: PDIRENT,
    EaHandle: *mut u16,
    EaSetRange: *mut EA_RANGE,
) {
    let BytesPerCluster: u32 = 1u32 << (*Vcb).AllocationSupport.LogOfBytesPerCluster;

    let mut EaHeaderRange: EA_RANGE = core::mem::zeroed();
    let mut EaHeaderIndex: u16 = 0;
    let mut EaHeader: *mut EA_FILE_HEADER = core::ptr::null_mut();

    let mut EaOffsetRange: EA_RANGE = core::mem::zeroed();
    let mut EaNewOffsetVbo: u32 = 0;
    let mut EaOffsetIndex: u16 = 0;
    let mut EaOffsetTableSize: u32 = 0;
    let mut EaOffsetTable: *mut u16 = core::ptr::null_mut();

    let mut EaSetClusterOffset: u32 = 0;
    let mut EaSetVbo: u32 = 0;
    let mut EaSetClusterCount: u16 = 0;
    let mut EaSet: *mut EA_SET_HEADER = core::ptr::null_mut();

    let VirtualEaFile: PFILE_OBJECT = (*Vcb).VirtualEaFile;
    let EaFcb: PFCB = (*Vcb).EaFcb;

    let mut EaSetMcb: LARGE_MCB = core::mem::zeroed();
    let mut UnwindInitializedEaSetMcb: i32 = 0;
    let mut UnwindAllocatedNewAllocation: i32 = 0;
    let mut UnwindMergedNewEaSet: i32 = 0;

    let mut EaOffsetMcb: LARGE_MCB = core::mem::zeroed();
    let mut UnwindInitializedOffsetMcb: i32 = 0;
    let mut UnwindSplitNewAllocation: i32 = 0;
    let mut UnwindMergedNewOffset: i32 = 0;

    let mut EaTailMcb: LARGE_MCB = core::mem::zeroed();
    let mut UnwindInitializedTailMcb: i32 = 0;
    let mut UnwindSplitTail: i32 = 0;
    let mut UnwindMergedTail: i32 = 0;

    let mut EaInitialEaMcb: LARGE_MCB = core::mem::zeroed();
    let mut UnwindInitializedInitialEaMcb: i32 = 0;
    let mut UnwindSplitInitialEa: i32 = 0;
    let mut UnwindMergedInitialEa: i32 = 0;

    let mut NewEaIndex: u16 = 0;
    let mut NextEaOffset: *mut u16 = core::ptr::null_mut();

    let mut NewAllocation: u32 = 0;
    let mut FileOffset: LARGE_INTEGER = core::mem::zeroed();
    let mut Count: u32 = 0;

    let mut UnwindPrevFileSize: u32 = 0;
    let mut UnwindCacheValues: i32 = 0;

    let mut TailExists: i32 = 0;
    let mut AddedOffsetTableCluster: i32 = 0;
    let mut UnwindPurgeCacheMap: i32 = 0;

    let p_EaHeaderRange = core::ptr::addr_of_mut!(EaHeaderRange);
    let p_EaOffsetRange = core::ptr::addr_of_mut!(EaOffsetRange);
    let p_EaSetMcb = core::ptr::addr_of_mut!(EaSetMcb);
    let p_EaOffsetMcb = core::ptr::addr_of_mut!(EaOffsetMcb);
    let p_EaTailMcb = core::ptr::addr_of_mut!(EaTailMcb);
    let p_EaInitialEaMcb = core::ptr::addr_of_mut!(EaInitialEaMcb);
    let p_UnwindInitializedEaSetMcb = core::ptr::addr_of_mut!(UnwindInitializedEaSetMcb);
    let p_UnwindAllocatedNewAllocation = core::ptr::addr_of_mut!(UnwindAllocatedNewAllocation);
    let p_UnwindMergedNewEaSet = core::ptr::addr_of_mut!(UnwindMergedNewEaSet);
    let p_UnwindInitializedOffsetMcb = core::ptr::addr_of_mut!(UnwindInitializedOffsetMcb);
    let p_UnwindSplitNewAllocation = core::ptr::addr_of_mut!(UnwindSplitNewAllocation);
    let p_UnwindMergedNewOffset = core::ptr::addr_of_mut!(UnwindMergedNewOffset);
    let p_UnwindInitializedTailMcb = core::ptr::addr_of_mut!(UnwindInitializedTailMcb);
    let p_UnwindSplitTail = core::ptr::addr_of_mut!(UnwindSplitTail);
    let p_UnwindMergedTail = core::ptr::addr_of_mut!(UnwindMergedTail);
    let p_UnwindInitializedInitialEaMcb = core::ptr::addr_of_mut!(UnwindInitializedInitialEaMcb);
    let p_UnwindSplitInitialEa = core::ptr::addr_of_mut!(UnwindSplitInitialEa);
    let p_UnwindMergedInitialEa = core::ptr::addr_of_mut!(UnwindMergedInitialEa);
    let p_UnwindPrevFileSize = core::ptr::addr_of_mut!(UnwindPrevFileSize);
    let p_UnwindCacheValues = core::ptr::addr_of_mut!(UnwindCacheValues);
    let p_UnwindPurgeCacheMap = core::ptr::addr_of_mut!(UnwindPurgeCacheMap);
    let p_EaSetVbo = core::ptr::addr_of_mut!(EaSetVbo);
    let p_EaSetLength = core::ptr::addr_of_mut!(EaSetLength);
    let p_EaNewOffsetVbo = core::ptr::addr_of_mut!(EaNewOffsetVbo);
    // _SEH2_TRY / _SEH2_FINALLY
    crate::seh::try_finally(&mut (), |_| {
        core::ptr::write_bytes(core::ptr::addr_of_mut!(*p_EaHeaderRange) as *mut u8, 0, core::mem::size_of::<EA_RANGE>());
        core::ptr::write_bytes(core::ptr::addr_of_mut!(*p_EaOffsetRange) as *mut u8, 0, core::mem::size_of::<EA_RANGE>());

        FatPinEaRange(
            IrpContext,
            VirtualEaFile,
            EaFcb,
            core::ptr::addr_of_mut!(*p_EaHeaderRange),
            0,
            core::mem::size_of::<EA_FILE_HEADER>() as u32,
            STATUS_DATA_ERROR,
        );
        EaHeader = (*p_EaHeaderRange).Data as *mut EA_FILE_HEADER;

        *p_EaNewOffsetVbo = ((*EaHeader).EaBaseTable[0] as u32) << (*Vcb).AllocationSupport.LogOfBytesPerCluster;
        EaOffsetTableSize = (*p_EaNewOffsetVbo) - core::mem::size_of::<EA_FILE_HEADER>() as u32;

        FatPinEaRange(
            IrpContext,
            VirtualEaFile,
            EaFcb,
            core::ptr::addr_of_mut!(*p_EaOffsetRange),
            core::mem::size_of::<EA_FILE_HEADER>() as u32,
            EaOffsetTableSize,
            STATUS_DATA_ERROR,
        );

        EaSetClusterOffset = (*EaFcb).Header.FileSize.__bindgen_anon_1.LowPart
            >> (*Vcb).AllocationSupport.LogOfBytesPerCluster;

        NewEaIndex = (EaOffsetTableSize >> 1).wrapping_sub(1) as u16;

        NextEaOffset = ((*p_EaOffsetRange).Data as *mut u16).add(NewEaIndex as usize);

        if *NextEaOffset != UNUSED_EA_HANDLE {
            while NewEaIndex != 0 {
                if *(NextEaOffset.offset(-1)) == UNUSED_EA_HANDLE {
                    if NewEaIndex != 1 {
                        EaSetClusterOffset = (*NextEaOffset as u32)
                            .wrapping_add((*EaHeader).EaBaseTable[(NewEaIndex >> 7) as usize] as u32);
                        TailExists = 1;
                    }
                    NewEaIndex = NewEaIndex.wrapping_sub(1);
                    NextEaOffset = NextEaOffset.offset(-1);
                    break;
                }
                NewEaIndex = NewEaIndex.wrapping_sub(1);
                NextEaOffset = NextEaOffset.offset(-1);
            }
        }

        while NewEaIndex != 0 {
            if *(NextEaOffset.offset(-1)) != UNUSED_EA_HANDLE {
                break;
            }
            NextEaOffset = NextEaOffset.offset(-1);
            NewEaIndex = NewEaIndex.wrapping_sub(1);
        }

        if NewEaIndex == 0 {
            if *(NextEaOffset.offset(1)) == UNUSED_EA_HANDLE {
                NewEaIndex = 1;
            } else {
                NewEaIndex = (EaOffsetTableSize >> 1) as u16;
                AddedOffsetTableCluster = 1;
            }
        }

        if NewEaIndex > MAX_EA_HANDLE {
            DebugTrace(-1, Dbg as u32, b"FatAddEaSet: Illegal handle value for new handle\n\0".as_ptr() as *const i8, 0);
            FatRaiseStatus(IrpContext, STATUS_INSUFFICIENT_RESOURCES);
        }

        EaHeaderIndex = (NewEaIndex >> 7) as u16;
        EaOffsetIndex = (NewEaIndex as u16) & (MAX_EA_OFFSET_INDEX as u16 - 1);

        *p_EaSetVbo = EaSetClusterOffset << (*Vcb).AllocationSupport.LogOfBytesPerCluster;

        *p_EaSetLength = ((*p_EaSetLength) + BytesPerCluster - 1) & !(BytesPerCluster - 1);

        NewAllocation = (*p_EaSetLength)
            + if AddedOffsetTableCluster != 0 { BytesPerCluster } else { 0 };

        if ((0x0000FFFFu32 << (*Vcb).AllocationSupport.LogOfBytesPerCluster)
            .wrapping_sub((*EaFcb).Header.FileSize.__bindgen_anon_1.LowPart))
            < NewAllocation
        {
            DebugTrace(-1, Dbg as u32, b"FatAddEaSet: New Ea file size is too large\n\0".as_ptr() as *const i8, 0);
            FatRaiseStatus(IrpContext, STATUS_INSUFFICIENT_RESOURCES);
        }

        FsRtlInitializeLargeMcb(core::ptr::addr_of_mut!(*p_EaSetMcb) as *mut LARGE_MCB, PagedPool);
        *p_UnwindInitializedEaSetMcb = 1;

        FatAllocateDiskSpace(
            IrpContext,
            Vcb,
            0,
            core::ptr::addr_of_mut!(NewAllocation),
            0,
            core::ptr::addr_of_mut!(*p_EaSetMcb) as *mut LARGE_MCB,
        );
        *p_UnwindAllocatedNewAllocation = 1;

        EaSetClusterCount = ((*p_EaSetLength) >> (*Vcb).AllocationSupport.LogOfBytesPerCluster) as u16;

        if AddedOffsetTableCluster != 0 {
            FsRtlInitializeLargeMcb(core::ptr::addr_of_mut!(*p_EaOffsetMcb) as *mut LARGE_MCB, PagedPool);
            *p_UnwindInitializedOffsetMcb = 1;

            FatSplitAllocation(
                IrpContext,
                Vcb,
                core::ptr::addr_of_mut!(*p_EaSetMcb) as *mut LARGE_MCB,
                (*p_EaSetLength),
                core::ptr::addr_of_mut!(*p_EaOffsetMcb) as *mut LARGE_MCB,
            );
            *p_UnwindSplitNewAllocation = 1;
        }

        FatUnpinEaRange(IrpContext, (core::ptr::addr_of_mut!(*p_EaHeaderRange)).cast());
        FatUnpinEaRange(IrpContext, (core::ptr::addr_of_mut!(*p_EaOffsetRange)).cast());

        if AddedOffsetTableCluster != 0 {
            FileOffset.__bindgen_anon_1.LowPart = (*p_EaNewOffsetVbo);
        } else {
            FileOffset.__bindgen_anon_1.LowPart = (*p_EaSetVbo);
        }
        FileOffset.__bindgen_anon_1.HighPart = 0;

        FileOffset.__bindgen_anon_1.LowPart &= !(PAGE_SIZE - 1);

        {
            let mut Iosb: _IO_STATUS_BLOCK = core::mem::zeroed();
            let mut PurgeCount: u32 = 5;
            while {
                PurgeCount = PurgeCount.wrapping_sub(1);
                PurgeCount
            } != 0 {
                Iosb.__bindgen_anon_1.Status = STATUS_SUCCESS;
                // CcFlushCache( Sop, NULL /* FileOffset */, 0, &Iosb ).
                CcFlushCache(
                    (*VirtualEaFile).SectionObjectPointer as *mut c_void,
                    core::ptr::null_mut(),
                    0,
                    core::ptr::addr_of_mut!(Iosb),
                );
                NT_ASSERT((Iosb.__bindgen_anon_1.Status == STATUS_SUCCESS as i32) as i32);

                if CcPurgeCacheSection(
                    (*VirtualEaFile).SectionObjectPointer,
                    core::ptr::addr_of_mut!(FileOffset),
                    0,
                    0,
                ) != 0 {
                    break;
                }
            }
            if PurgeCount == 0 {
                FatRaiseStatus(IrpContext, STATUS_UNABLE_TO_DELETE_SECTION);
            }
        }

        *p_UnwindPurgeCacheMap = 1;

        FileOffset.__bindgen_anon_1.LowPart = (*EaFcb).Header.FileSize.__bindgen_anon_1.LowPart + NewAllocation;

        if TailExists != 0 {
            FsRtlInitializeLargeMcb(core::ptr::addr_of_mut!(*p_EaTailMcb) as *mut LARGE_MCB, PagedPool);
            *p_UnwindInitializedTailMcb = 1;

            FatSplitAllocation(
                IrpContext,
                Vcb,
                &(*EaFcb).Mcb as *const _ as *mut LARGE_MCB,
                (*p_EaSetVbo),
                core::ptr::addr_of_mut!(*p_EaTailMcb) as *mut LARGE_MCB,
            );
            *p_UnwindSplitTail = 1;
        }

        if AddedOffsetTableCluster != 0 && (*p_EaSetVbo) != (*p_EaNewOffsetVbo) {
            FsRtlInitializeLargeMcb(core::ptr::addr_of_mut!(*p_EaInitialEaMcb) as *mut LARGE_MCB, PagedPool);
            *p_UnwindInitializedInitialEaMcb = 1;

            FatSplitAllocation(
                IrpContext,
                Vcb,
                &(*EaFcb).Mcb as *const _ as *mut LARGE_MCB,
                (*p_EaNewOffsetVbo),
                core::ptr::addr_of_mut!(*p_EaInitialEaMcb) as *mut LARGE_MCB,
            );
            *p_UnwindSplitInitialEa = 1;
        }

        if AddedOffsetTableCluster != 0 {
            FatMergeAllocation(
                IrpContext,
                Vcb,
                &(*EaFcb).Mcb as *const _ as *mut LARGE_MCB,
                core::ptr::addr_of_mut!(*p_EaOffsetMcb) as *mut LARGE_MCB,
            );
            FsRtlUninitializeLargeMcb(core::ptr::addr_of_mut!(*p_EaOffsetMcb) as *mut LARGE_MCB);
            FsRtlInitializeLargeMcb(core::ptr::addr_of_mut!(*p_EaOffsetMcb) as *mut LARGE_MCB, PagedPool);
            *p_UnwindMergedNewOffset = 1;
        }

        if (*p_UnwindInitializedInitialEaMcb) != 0 {
            FatMergeAllocation(
                IrpContext,
                Vcb,
                &(*EaFcb).Mcb as *const _ as *mut LARGE_MCB,
                core::ptr::addr_of_mut!(*p_EaInitialEaMcb) as *mut LARGE_MCB,
            );
            FsRtlUninitializeLargeMcb(core::ptr::addr_of_mut!(*p_EaInitialEaMcb) as *mut LARGE_MCB);
            FsRtlInitializeLargeMcb(core::ptr::addr_of_mut!(*p_EaInitialEaMcb) as *mut LARGE_MCB, PagedPool);
            *p_UnwindMergedInitialEa = 1;
        }

        if AddedOffsetTableCluster != 0 {
            EaSetClusterOffset += 1;
            (*p_EaSetVbo) += BytesPerCluster;
        }

        FatMergeAllocation(
            IrpContext,
            Vcb,
            &(*EaFcb).Mcb as *const _ as *mut LARGE_MCB,
            core::ptr::addr_of_mut!(*p_EaSetMcb) as *mut LARGE_MCB,
        );
        FsRtlUninitializeLargeMcb(core::ptr::addr_of_mut!(*p_EaSetMcb) as *mut LARGE_MCB);
        FsRtlInitializeLargeMcb(core::ptr::addr_of_mut!(*p_EaSetMcb) as *mut LARGE_MCB, PagedPool);
        *p_UnwindMergedNewEaSet = 1;

        if (*p_UnwindInitializedTailMcb) != 0 {
            FatMergeAllocation(
                IrpContext,
                Vcb,
                &(*EaFcb).Mcb as *const _ as *mut LARGE_MCB,
                core::ptr::addr_of_mut!(*p_EaTailMcb) as *mut LARGE_MCB,
            );
            FsRtlUninitializeLargeMcb(core::ptr::addr_of_mut!(*p_EaTailMcb) as *mut LARGE_MCB);
            FsRtlInitializeLargeMcb(core::ptr::addr_of_mut!(*p_EaTailMcb) as *mut LARGE_MCB, PagedPool);
            *p_UnwindMergedTail = 1;
        }

        *p_UnwindPrevFileSize = (*EaFcb).Header.FileSize.__bindgen_anon_1.LowPart;

        (*EaFcb).Header.FileSize.__bindgen_anon_1.LowPart += NewAllocation;
        (*EaFcb).Header.AllocationSize = (*EaFcb).Header.FileSize;
        (*EaDirent).FileSize = (*EaFcb).Header.FileSize.__bindgen_anon_1.LowPart;

        FatSetDirtyBcb(IrpContext, (EaBcb).cast(), Vcb, 1);

        CcSetFileSizes(
            VirtualEaFile,
            &(*EaFcb).Header.AllocationSize as *const _ as *mut _,
        );
        *p_UnwindCacheValues = 1;

        core::ptr::write_bytes(core::ptr::addr_of_mut!(*p_EaHeaderRange) as *mut u8, 0, core::mem::size_of::<EA_RANGE>());

        FatPinEaRange(
            IrpContext,
            VirtualEaFile,
            EaFcb,
            core::ptr::addr_of_mut!(*p_EaHeaderRange),
            0,
            core::mem::size_of::<EA_FILE_HEADER>() as u32,
            STATUS_DATA_ERROR,
        );
        EaHeader = (*p_EaHeaderRange).Data as *mut EA_FILE_HEADER;

        core::ptr::write_bytes(core::ptr::addr_of_mut!(*p_EaOffsetRange) as *mut u8, 0, core::mem::size_of::<EA_RANGE>());

        FatPinEaRange(
            IrpContext,
            VirtualEaFile,
            EaFcb,
            core::ptr::addr_of_mut!(*p_EaOffsetRange),
            core::mem::size_of::<EA_FILE_HEADER>() as u32 + (((NewEaIndex as u64) >> 7) << 8) as u32,
            core::mem::size_of::<EA_OFF_TABLE>() as u32,
            STATUS_DATA_ERROR,
        );
        EaOffsetTable = (*p_EaOffsetRange).Data as *mut u16;

        FatPinEaRange(
            IrpContext,
            VirtualEaFile,
            EaFcb,
            EaSetRange,
            (*p_EaSetVbo),
            (*p_EaSetLength),
            STATUS_DATA_ERROR,
        );
        EaSet = (*EaSetRange).Data as *mut EA_SET_HEADER;

        (*EaSet).Signature = (EA_SET_SIGNATURE) as u16;
        (*EaSet).OwnEaHandle = NewEaIndex;

        FatMarkEaRangeDirty(IrpContext, VirtualEaFile, EaSetRange);

        Count = MAX_EA_BASE_INDEX as u32 - EaHeaderIndex as u32 - 1;
        NextEaOffset = &mut (*EaHeader).EaBaseTable[EaHeaderIndex as usize + 1 ] as *mut u16;
        while Count > 0 {
            Count = Count.wrapping_sub(1);
            *NextEaOffset = (*NextEaOffset).wrapping_add((EaSetClusterCount as u32) as u16);
            NextEaOffset = NextEaOffset.add(1);
        }

        if AddedOffsetTableCluster != 0 {
            Count = MAX_EA_BASE_INDEX as u32;
            NextEaOffset = &mut (*EaHeader).EaBaseTable[0] as *mut u16;
            while Count > 0 {
                Count = Count.wrapping_sub(1);
                *NextEaOffset = (*NextEaOffset).wrapping_add(1);
                NextEaOffset = NextEaOffset.add(1);
            }
        }

        FatMarkEaRangeDirty(IrpContext, VirtualEaFile, core::ptr::addr_of_mut!(*p_EaHeaderRange));

        if AddedOffsetTableCluster != 0 {
            Count = (BytesPerCluster >> 1) - 1;
            NextEaOffset = EaOffsetTable;
            *NextEaOffset = 0;
            NextEaOffset = NextEaOffset.add(1);
            while Count > 0 {
                Count = Count.wrapping_sub(1);
                *NextEaOffset = UNUSED_EA_HANDLE;
                NextEaOffset = NextEaOffset.add(1);
            }
        }

        NextEaOffset = EaOffsetTable.add(EaOffsetIndex as usize);
        *NextEaOffset = (EaSetClusterOffset - (*EaHeader).EaBaseTable[EaHeaderIndex as usize] as u32) as u16;
        NextEaOffset = NextEaOffset.add(1);

        Count = MAX_EA_OFFSET_INDEX as u32 - EaOffsetIndex as u32 - 1;
        while Count > 0 {
            Count = Count.wrapping_sub(1);
            if *NextEaOffset != UNUSED_EA_HANDLE {
                *NextEaOffset = (*NextEaOffset as u32 + EaSetClusterCount as u32) as u16;
            }
            NextEaOffset = NextEaOffset.add(1);
        }

        FatMarkEaRangeDirty(IrpContext, VirtualEaFile, core::ptr::addr_of_mut!(*p_EaOffsetRange));

        *EaHandle = NewEaIndex;

        DebugTrace(0, Dbg as u32, b"FatAddEaSet: Return values\n\0".as_ptr() as *const i8, 0);
        DebugTrace(0, Dbg as u32, b"FatAddEaSet: New Handle -> %x\n\0".as_ptr() as *const i8, *EaHandle as u32);
    }, |_, abnormal| {
        if abnormal && (*p_UnwindAllocatedNewAllocation) != 0 {
            {
                if (*p_UnwindPrevFileSize) != 0 {
                    (*EaFcb).Header.FileSize.__bindgen_anon_1.LowPart = (*p_UnwindPrevFileSize);
                    (*EaFcb).Header.AllocationSize.__bindgen_anon_1.LowPart = (*p_UnwindPrevFileSize);
                    (*EaDirent).FileSize = (*p_UnwindPrevFileSize);
                    if (*p_UnwindCacheValues) != 0 {
                        CcSetFileSizes(
                            VirtualEaFile,
                            &(*EaFcb).Header.AllocationSize as *const _ as *mut _,
                        );
                    }
                }

                if (*p_UnwindMergedTail) != 0 {
                    let NewTailPosition: u32 = (*p_EaSetVbo) + (*p_EaSetLength);
                    FatSplitAllocation(
                        IrpContext,
                        Vcb,
                        &(*EaFcb).Mcb as *const _ as *mut LARGE_MCB,
                        NewTailPosition,
                        core::ptr::addr_of_mut!(*p_EaTailMcb) as *mut LARGE_MCB,
                    );
                }

                if (*p_UnwindMergedNewEaSet) != 0 {
                    FatSplitAllocation(
                        IrpContext,
                        Vcb,
                        &(*EaFcb).Mcb as *const _ as *mut LARGE_MCB,
                        (*p_EaSetVbo),
                        core::ptr::addr_of_mut!(*p_EaSetMcb) as *mut LARGE_MCB,
                    );
                }

                if (*p_UnwindMergedInitialEa) != 0 {
                    FatSplitAllocation(
                        IrpContext,
                        Vcb,
                        &(*EaFcb).Mcb as *const _ as *mut LARGE_MCB,
                        (*p_EaNewOffsetVbo) + BytesPerCluster,
                        core::ptr::addr_of_mut!(*p_EaInitialEaMcb) as *mut LARGE_MCB,
                    );
                }

                if (*p_UnwindMergedNewOffset) != 0 {
                    FatSplitAllocation(
                        IrpContext,
                        Vcb,
                        &(*EaFcb).Mcb as *const _ as *mut LARGE_MCB,
                        (*p_EaNewOffsetVbo),
                        core::ptr::addr_of_mut!(*p_EaOffsetMcb) as *mut LARGE_MCB,
                    );
                }

                if (*p_UnwindSplitInitialEa) != 0 {
                    FatMergeAllocation(
                        IrpContext,
                        Vcb,
                        &(*EaFcb).Mcb as *const _ as *mut LARGE_MCB,
                        core::ptr::addr_of_mut!(*p_EaInitialEaMcb) as *mut LARGE_MCB,
                    );
                }

                if (*p_UnwindSplitTail) != 0 {
                    FatMergeAllocation(
                        IrpContext,
                        Vcb,
                        &(*EaFcb).Mcb as *const _ as *mut LARGE_MCB,
                        core::ptr::addr_of_mut!(*p_EaTailMcb) as *mut LARGE_MCB,
                    );
                }

                if (*p_UnwindPurgeCacheMap) != 0 {
                    (*Vcb).VirtualEaFile = core::ptr::null_mut();
                    ObDereferenceObject((VirtualEaFile).cast());
                }

                if (*p_UnwindSplitNewAllocation) != 0 {
                    FatDeallocateDiskSpace(
                        IrpContext,
                        Vcb,
                        core::ptr::addr_of_mut!(*p_EaOffsetMcb) as *mut LARGE_MCB,
                        0,
                    );
                }

                FatDeallocateDiskSpace(
                    IrpContext,
                    Vcb,
                    core::ptr::addr_of_mut!(*p_EaSetMcb) as *mut LARGE_MCB,
                    0,
                );
            }
        }

        FatUnpinEaRange(IrpContext, (core::ptr::addr_of_mut!(*p_EaHeaderRange)).cast());
        FatUnpinEaRange(IrpContext, (core::ptr::addr_of_mut!(*p_EaOffsetRange)).cast());

        if (*p_UnwindInitializedEaSetMcb) != 0 {
            FsRtlUninitializeLargeMcb(core::ptr::addr_of_mut!(*p_EaSetMcb) as *mut LARGE_MCB);
        }
        if (*p_UnwindInitializedOffsetMcb) != 0 {
            FsRtlUninitializeLargeMcb(core::ptr::addr_of_mut!(*p_EaOffsetMcb) as *mut LARGE_MCB);
        }
        if (*p_UnwindInitializedTailMcb) != 0 {
            FsRtlUninitializeLargeMcb(core::ptr::addr_of_mut!(*p_EaTailMcb) as *mut LARGE_MCB);
        }
        if (*p_UnwindInitializedInitialEaMcb) != 0 {
            FsRtlUninitializeLargeMcb(core::ptr::addr_of_mut!(*p_EaInitialEaMcb) as *mut LARGE_MCB);
        }

        DebugTrace(-1, Dbg as u32, b"FatAddEaSet ->  Exit\n\0".as_ptr() as *const i8, 0);
    });
}

#[no_mangle]
pub unsafe extern "system" fn FatAppendPackedEa(
    IrpContext: *mut IRP_CONTEXT,
    EaSetHeader: *mut *mut EA_SET_HEADER,
    PackedEasLength: *mut u32,
    AllocationLength: *mut u32,
    FullEa: *mut FILE_FULL_EA_INFORMATION,
    BytesPerCluster: u32,
) {
    let mut PackedEaSize: u32;
    let mut ThisPackedEa: *mut PACKED_EA;
    let mut EaName: OEM_STRING = unsafe { core::mem::zeroed() };
    let Temp: *mut core::ffi::c_void;
    let NewAllocationSize: u32;
    let OldAllocationSize: u32;

    PagedCode();

    DebugTrace(
        1i32,
        0u32, // DEBUG_TRACE_EA
        b"FatAppendPackedEa...\n\0".as_ptr() as *const i8,
        0usize,
    );

    PackedEaSize = SizeOfFullEa(FullEa) - 4u32;

    if PackedEaSize + *PackedEasLength > *AllocationLength {
        DebugTrace(
            0i32,
            0u32, // DEBUG_TRACE_EA
            b"Allocate a new ea list buffer\n\0".as_ptr() as *const i8,
            0usize,
        );

        // SIZE_OF_EA_SET_HEADER is 30 (fat.h); must agree with FatCreateEa's accounting.
        let size_of_ea_set_header: u32 = 30;
        NewAllocationSize = (size_of_ea_set_header
            .wrapping_add(PackedEaSize)
            .wrapping_add(*PackedEasLength)
            .wrapping_add(BytesPerCluster)
            .wrapping_sub(1))
            & !(BytesPerCluster.wrapping_sub(1));

        Temp = FsRtlAllocatePoolWithTag(
            ntkrnl_sys::mm::PagedPool,
            NewAllocationSize,
            // nodetype.h TAG_EA_SET_HEADER ('etaF'); 0 was PoolHitTag's default, a
            // DbgBreakPoint in the pool tracker on every EA-set grow.
            u32::from_le_bytes(*b"Fate"),
        );

        OldAllocationSize = *AllocationLength + size_of_ea_set_header;

        core::ptr::copy_nonoverlapping(
            *EaSetHeader as *const u8,
            Temp as *mut u8,
            OldAllocationSize as usize,
        );

        ExFreePool(*EaSetHeader as *mut core::ffi::c_void);

        *EaSetHeader = Temp as *mut EA_SET_HEADER;

        let target =
            (*(*EaSetHeader)).PackedEas.as_mut_ptr().add(*AllocationLength as usize) as *mut u8;
        core::ptr::write_bytes(
            target,
            0u8,
            (NewAllocationSize - OldAllocationSize) as usize,
        );

        *AllocationLength = NewAllocationSize - size_of_ea_set_header;
    }

    if (*FullEa).Flags & 0x80u8 /* FILE_NEED_EA */ != 0 {
        (*(*EaSetHeader)).NeedEaCount += 1;
    }

    ThisPackedEa = ((*(*EaSetHeader)).PackedEas.as_mut_ptr() as *mut u8)
        .add(*PackedEasLength as usize) as *mut PACKED_EA;

    core::ptr::copy_nonoverlapping(
        (FullEa as *mut u8).add(4),
        ThisPackedEa as *mut u8,
        PackedEaSize as usize,
    );

    let ea_name_len = (*FullEa).EaNameLength;
    EaName.MaximumLength = (ea_name_len) as u16;
    EaName.Length = (ea_name_len) as u16;
    EaName.Buffer = (*ThisPackedEa).EaName.as_mut_ptr() as *mut i8;

    FatUpcaseEaName(IrpContext, &mut EaName, &mut EaName);

    *PackedEasLength = (*PackedEasLength).wrapping_add(PackedEaSize);

    DebugTrace(
        -1i32,
        0u32, // DEBUG_TRACE_EA
        b"FatAppendPackedEa -> VOID\n\0".as_ptr() as *const i8,
        0usize,
    );

    let _ = IrpContext;
}
#[no_mangle]
pub unsafe extern "system" fn FatDeletePackedEa(
    IrpContext: *const IRP_CONTEXT,
    EaSetHeader: *mut EA_SET_HEADER,
    PackedEasLength: *mut u32,
    Offset: u32,
) {
    unsafe {
        PAGED_CODE!();

        DebugTrace(1, Dbg, (b"FatDeletePackedEa, Offset = %08lx\n\0".as_ptr() as *const u8).cast(), Offset);

        let packed_ea: *mut PACKED_EA = (*EaSetHeader).PackedEas.as_mut_ptr().add(Offset as usize) as *mut PACKED_EA;
        let mut packed_ea_size: u32 = 0;
        SizeOfPackedEa(packed_ea, &mut packed_ea_size as *mut u32);

        if FlagOn((*packed_ea).Flags, EA_NEED_EA_FLAG) != 0 {
            (*EaSetHeader).NeedEaCount -= 1;
        }

        let copy_len = *PackedEasLength - (Offset + packed_ea_size);
        let src = (packed_ea as *mut u8).add(packed_ea_size as usize);
        core::ptr::copy_nonoverlapping(src as *const u8, packed_ea as *mut u8, copy_len as usize);

        let zero_addr = (*EaSetHeader).PackedEas.as_mut_ptr().add((*PackedEasLength - packed_ea_size) as usize);
        core::ptr::write_bytes(zero_addr, 0, packed_ea_size as usize);

        *PackedEasLength -= packed_ea_size;

        DebugTrace(-1, Dbg, (b"FatDeletePackedEa -> VOID\n\0".as_ptr() as *const u8).cast(), 0);

        UNREFERENCED_PARAMETER!(IrpContext as *mut core::ffi::c_void);
    }
}
#[no_mangle]
pub unsafe extern "system" fn FatLocateNextEa(
    IrpContext: *mut IRP_CONTEXT,
    FirstPackedEa: *mut PACKED_EA,
    PackedEasLength: u32,
    PreviousOffset: u32,
) -> u32 {
    // PAGED_CODE();

    DebugTrace(1, Dbg, b"FatLocateNextEa, PreviousOffset = %08lx\n\0".as_ptr() as *const i8, PreviousOffset);

    if PreviousOffset >= PackedEasLength {
        DebugTrace(-1, Dbg, b"FatLocateNextEa -> 0xffffffff\n\0".as_ptr() as *const i8, 0);
        return 0xffffffff;
    }

    let packed_ea: *mut PACKED_EA = (FirstPackedEa as *mut u8).add(PreviousOffset as usize) as *mut PACKED_EA;
    let mut packed_ea_size: u32 = 0;
    SizeOfPackedEa(packed_ea as *const PACKED_EA, core::ptr::addr_of_mut!(packed_ea_size));

    let mut offset = PreviousOffset.wrapping_add(packed_ea_size);

    if offset >= PackedEasLength {
        offset = 0xffffffff;
    }

    DebugTrace(-1, Dbg, b"FatLocateNextEa -> %08lx\n\0".as_ptr() as *const i8, offset);

    let _ = IrpContext;

    offset
}
#[no_mangle]
pub unsafe extern "system" fn FatLocateEaByName(
    IrpContext: PIRP_CONTEXT,
    FirstPackedEa: *mut PACKED_EA,
    PackedEasLength: u32,
    EaName: *mut OEM_STRING,
    Offset: *mut u32,
) -> i32 {
    // PAGED_CODE();

    DebugTrace(
        1,
        Dbg,
        b"FatLocateEaByName, EaName = %Z\n\0".as_ptr() as *const i8,
        EaName,
    );

    *Offset = 0;
    while *Offset < PackedEasLength {
        let packed_ea = (FirstPackedEa as *const u8).add(*Offset as usize) as *mut PACKED_EA;

        let mut name: OEM_STRING = core::mem::zeroed();
        name.Buffer = (core::ptr::addr_of_mut!((*packed_ea).EaName).cast::<u8>()).cast();
        name.Length = ((*packed_ea).EaNameLength) as u16;
        name.MaximumLength = ((*packed_ea).EaNameLength) as u16;

        if RtlCompareString(EaName, core::ptr::addr_of!(name), 1) == 0 {
            DebugTrace(
                -1,
                Dbg,
                b"FatLocateEaByName -> TRUE, *Offset = %08lx\n\0".as_ptr() as *const i8,
                *Offset,
            );
            return 1;
        }

        *Offset = FatLocateNextEa((IrpContext).cast(), FirstPackedEa, PackedEasLength, *Offset);
    }

    DebugTrace(-1, Dbg, b"FatLocateEaByName -> FALSE\n\0".as_ptr() as *const i8, 0);
    return 0;
}
#[no_mangle]
pub unsafe extern "system" fn FatIsEaNameValid(
    IrpContext: *mut IRP_CONTEXT,
    Name: OEM_STRING,
) -> u8 {
    let mut Index: u32;
    let mut Char: u8 = 0;

    PAGED_CODE!();

    _ = IrpContext;

    if Name.Length == 0 {
        return 0;
    }

    if Name.Length > 254 {
        return 0;
    }

    Index = 0;
    loop {
        if (Index as u16) >= Name.Length {
            break;
        }
        Char = (*Name.Buffer.offset(Index as isize)) as u8;

        if FsRtlIsLeadDbcsCharacter(Char) != 0 {
            NT_ASSERT((Index != (Name.Length as u32 - 1)) as i32);
            Index += 2;
            continue;
        }

        if FsRtlIsAnsiCharacterLegalFat(Char, 0) == 0 {
            return 0;
        }

        Index += 1;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn FatPinEaRange(
    IrpContext: *mut IRP_CONTEXT,
    VirtualEaFile: *mut FILE_OBJECT,
    EaFcb: *mut FCB,
    EaRange: *mut EA_RANGE,
    mut StartingVbo: u32,
    mut Length: u32,
    ErrorStatus: NTSTATUS,
) {
    let mut large_vbo: LARGE_INTEGER = core::mem::zeroed();
    let mut byte_count: u32;
    let mut next_bcb: *mut *mut BCB;
    let mut buffer: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut destination_buffer: *mut i8 = core::ptr::null_mut();
    let mut first_page: i32 = 1; // BOOLEAN TRUE

    // PAGED_CODE();

    //
    // Verify that the entire read is contained within the Ea file.
    //
    if Length == 0
        || StartingVbo >= (*EaFcb).Header.AllocationSize.__bindgen_anon_1.LowPart
        || ((*EaFcb).Header.AllocationSize.__bindgen_anon_1.LowPart - StartingVbo) < Length
    {
        FatRaiseStatus(IrpContext, ErrorStatus as i32);
    }

    //
    // If the read will span a section, allocate a separate buffer.
    //
    if ((StartingVbo & (EA_SECTION_SIZE - 1)) + Length) > EA_SECTION_SIZE {
        (*EaRange).Data = (FsRtlAllocatePoolWithTag(
            PagedPool,
            Length,
            TAG_EA_DATA,
        )).cast();
        (*EaRange).AuxilaryBuffer = 1; // TRUE
        destination_buffer = (*EaRange).Data as *mut i8;
    } else {
        NT_ASSERT(((*EaRange).AuxilaryBuffer == 0) as i32);
    }

    //
    // Compute number of pages needed and allocate Bcb chain if too many.
    //
    (*EaRange).BcbChainLength = (((StartingVbo & (PAGE_SIZE - 1)) + Length + PAGE_SIZE - 1) / PAGE_SIZE) as u16;

    if (*EaRange).BcbChainLength as u32 > EA_BCB_ARRAY_SIZE {
        (*EaRange).BcbChain = (FsRtlAllocatePoolWithTag(
            PagedPool,
            (core::mem::size_of::<*mut BCB>() as u32) * (*EaRange).BcbChainLength as u32,
            TAG_BCB,
        )).cast();
        core::ptr::write_bytes(
            (*EaRange).BcbChain as *mut u8,
            0,
            (core::mem::size_of::<*mut BCB>() as usize) * (*EaRange).BcbChainLength as usize,
        );
    } else {
        (*EaRange).BcbChain = (core::ptr::addr_of_mut!((*EaRange).BcbArray) as *mut *mut BCB).cast();
    }

    //
    // Store the byte range data in the Ea Range structure.
    //
    (*EaRange).StartingVbo = StartingVbo;
    (*EaRange).Length = Length;

    //
    // Compute the initial pin length.
    //
    byte_count = PAGE_SIZE - (StartingVbo & (PAGE_SIZE - 1));

    //
    // For each page in the range; pin the page and update the Bcb count, copy to
    // the auxiliary buffer.
    //
    next_bcb = ((*EaRange).BcbChain).cast();

    while Length != 0 {
        //
        // Pin the page and remember the data start.
        //
        large_vbo.QuadPart = StartingVbo as i64;

        if byte_count > Length {
            byte_count = Length;
        }

        if CcPinRead(
               VirtualEaFile,
               core::ptr::addr_of_mut!(large_vbo),
               byte_count,
               BooleanFlagOn((*IrpContext).Flags, IRP_CONTEXT_FLAG_WAIT) as u32,
               next_bcb as *mut *mut c_void,
               &mut buffer,
           ) == 0
        {
            FatRaiseStatus(IrpContext, STATUS_CANT_WAIT as i32);
        }

        //
        // Increment the Bcb pointer and copy to the auxiliary buffer if necessary.
        //
        next_bcb = next_bcb.add(1);

        if (*EaRange).AuxilaryBuffer != 0 {
            core::ptr::copy_nonoverlapping(
                buffer as *const u8,
                destination_buffer as *mut u8,
                byte_count as usize,
            );

            destination_buffer = Add2Ptr(
                destination_buffer as *mut core::ffi::c_void,
                byte_count,
            ) as *mut i8;
        }

        StartingVbo += byte_count;
        Length -= byte_count;

        //
        // If this is the first page then update the Ea Range structure.
        //
        if first_page != 0 {
            first_page = 0; // FALSE
            byte_count = PAGE_SIZE;

            if (*EaRange).AuxilaryBuffer == 0 {
                (*EaRange).Data = (buffer).cast();
            }
        }
    }
}


#[no_mangle]
pub unsafe extern "system" fn FatMarkEaRangeDirty(
    IrpContext: *mut IRP_CONTEXT,
    EaFileObject: *mut FILE_OBJECT,
    EaRange: *mut EA_RANGE,
) {
    PAGED_CODE!();
    UNREFERENCED_PARAMETER!(IrpContext as *mut core::ffi::c_void);

    if (*EaRange).AuxilaryBuffer == 1 {
        let mut LargeVbo: LARGE_INTEGER = core::mem::zeroed();
        LargeVbo.QuadPart = ((*EaRange).StartingVbo) as i64;

        CcCopyWrite(
            EaFileObject,
            core::ptr::addr_of_mut!(LargeVbo),
            (*EaRange).Length,
            1u8,
            (*EaRange).Data as *mut c_void,
        );
    }

    let mut BcbCount: u32 = ((*EaRange).BcbChainLength) as u32;
    let mut NextBcb: *mut *mut core::ffi::c_void = (*EaRange).BcbChain;

    loop {
        if BcbCount == 0 {
            break;
        }
        if !(*NextBcb).is_null() {
            CcSetDirtyPinnedData(*NextBcb, core::ptr::null_mut());
        }
        NextBcb = NextBcb.add(1);
        BcbCount -= 1;
    }
}
#[no_mangle]
// `IN OUT PEA_RANGE EaRange` — ONE pointer to the struct, exactly like
// FatMarkEaRangeDirty right above. This took `*mut PEA_RANGE` and double-
// dereferenced, so all but two of its twenty call sites — which hand over a
// `*mut EA_RANGE` to a stack local, some via a `.cast()` that made the
// mismatch invisible — had their struct's first eight bytes read as a pointer
// and then dereferenced. rustc cannot see it: fileinfo.rs declares this
// function correctly and is compiled against that declaration, never against
// the definition here.
pub unsafe extern "system" fn FatUnpinEaRange(
    IrpContext: *mut IRP_CONTEXT,
    EaRange: PEA_RANGE,
) {
    let _ = IrpContext;

    if (*EaRange).AuxilaryBuffer == 1 {
        ExFreePool((*EaRange).Data as *mut core::ffi::c_void);
        (*EaRange).AuxilaryBuffer = 0;
    }

    if !(*EaRange).BcbChain.is_null() {
        let mut BcbCount = (*EaRange).BcbChainLength;
        let mut NextBcb = (*EaRange).BcbChain;

        while {
            let count = BcbCount;
            BcbCount = BcbCount.wrapping_sub(1);
            count > 0
        } {
            if !(*NextBcb).is_null() {
                CcUnpinData(*NextBcb);
                *NextBcb = core::ptr::null_mut();
            }
            NextBcb = NextBcb.add(1);
        }

        if (*EaRange).BcbChain != (*EaRange).BcbArray.as_mut_ptr() {
            ExFreePool((*EaRange).BcbChain as *mut core::ffi::c_void);
        }

        (*EaRange).BcbChain = core::ptr::null_mut();
    }
}



// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*++
//
// Copyright (c) 1990-2000 Microsoft Corporation
//
// Module Name:
//
//     EaSup.c
//
// Abstract:
//
//     This module implements the cluster operations on the EA file for Fat.
//
//
// --*/
//
// #include "fatprocs.h"
//
// //
// //  Local debug trace level
// //
//
// #define Dbg                              (DEBUG_TRACE_EA)
//
// #ifdef ALLOC_PRAGMA
// #pragma alloc_text(PAGE, FatAddEaSet)
// #pragma alloc_text(PAGE, FatAppendPackedEa)
// #pragma alloc_text(PAGE, FatCreateEa)
// #pragma alloc_text(PAGE, FatDeleteEa)
// #pragma alloc_text(PAGE, FatDeleteEaSet)
// #pragma alloc_text(PAGE, FatDeletePackedEa)
// #pragma alloc_text(PAGE, FatGetEaFile)
// #pragma alloc_text(PAGE, FatGetEaLength)
// #pragma alloc_text(PAGE, FatGetNeedEaCount)
// #pragma alloc_text(PAGE, FatIsEaNameValid)
// #pragma alloc_text(PAGE, FatLocateEaByName)
// #pragma alloc_text(PAGE, FatLocateNextEa)
// #pragma alloc_text(PAGE, FatReadEaSet)
// #pragma alloc_text(PAGE, FatPinEaRange)
// #pragma alloc_text(PAGE, FatMarkEaRangeDirty)
// #pragma alloc_text(PAGE, FatUnpinEaRange)
// #endif
//
//
// //
// //  Any access to the Ea file must recognize when a section boundary is being
// //  crossed.
// //
//
// #define EA_SECTION_SIZE             (0x00040000)
//
//
// _Requires_lock_held_(_Global_critical_region_)
// VOID
// FatGetEaLength (
//     IN PIRP_CONTEXT IrpContext,
//     IN PVCB Vcb,
//     IN PDIRENT Dirent,
//     OUT PULONG EaLength
//     )
//
// /*++
//
// Routine Description:
//
//     This routine looks up the Ea length for the Eas of the file.  This
//     length is the  length of the packed eas, including the 4 bytes which
//     contain the Ea length.
//
//     This routine pins down the Ea set for the desired file and copies
//     this field from the Ea set header.
//
// Arguments:
//
//     Vcb - Vcb for the volume containing the Eas.
//
//     Dirent - Supplies a pointer to the dirent for the file in question.
//
//     EaLength - Supplies the address to store the length of the Eas.
//
// Return Value:
//
//     None
//
// --*/
//
// {
//     PBCB EaBcb = NULL;
//     BOOLEAN LockedEaFcb = FALSE;
//     EA_RANGE EaSetRange;
//
//     PAGED_CODE();
//
//     DebugTrace(+1, Dbg, "FatGetEaLength ...\n", 0);
//
//     //
//     //  If this is Fat32 volume, or if the handle is 0 then the Ea length is 0.
//     //
//
//     if (FatIsFat32( Vcb ) ||
//         Dirent->ExtendedAttributes == 0) {
//
//         *EaLength = 0;
//         DebugTrace(-1, Dbg, "FatGetEaLength -> %08lx\n", TRUE);
//         return;
//     }
//
//     RtlZeroMemory( &EaSetRange, sizeof( EA_RANGE ));
//
//     //
//     //  Use a try to facilitate cleanup.
//     //
//
//     _SEH2_TRY {
//
//         PDIRENT EaDirent;
//         OEM_STRING ThisFilename;
//         UCHAR Buffer[12];
//         PEA_SET_HEADER EaSetHeader;
//
//         //
//         //  Initial the local values.
//         //
//
//         EaBcb = NULL;
//         LockedEaFcb = FALSE;
//
//         //
//         //  Try to get the Ea file object.  Return FALSE on failure.
//         //
//
//         FatGetEaFile( IrpContext,
//                       Vcb,
//                       &EaDirent,
//                       &EaBcb,
//                       FALSE,
//                       FALSE );
//
//         LockedEaFcb = TRUE;
//
//         //
//         //  If we didn't get the file because it doesn't exist, then the
//         //  disk is corrupted.
//         //
//
//         if (Vcb->VirtualEaFile == NULL) {
//
//             DebugTrace(0, Dbg, "FatGetEaLength:  Ea file doesn't exist\n", 0);
//             FatRaiseStatus( IrpContext, STATUS_NO_EAS_ON_FILE );
//         }
//
//         //
//         //  Try to pin down the Ea set header for the index in the
//         //  dirent.  If the operation doesn't complete, return FALSE
//         //  from this routine.
//         //
//
//         ThisFilename.Buffer = (PCHAR)Buffer;
//         Fat8dot3ToString( IrpContext, Dirent, FALSE, &ThisFilename );
//
//         FatReadEaSet( IrpContext,
//                       Vcb,
//                       Dirent->ExtendedAttributes,
//                       &ThisFilename,
//                       FALSE,
//                       &EaSetRange );
//
//         EaSetHeader = (PEA_SET_HEADER) EaSetRange.Data;
//
//         //
//         //  We now have the Ea set header for this file.  We simply copy
//         //  the Ea length field.
//         //
//
//         CopyUchar4( EaLength, EaSetHeader->cbList );
//         DebugTrace(0, Dbg, "FatGetEaLength:  Length of Ea is -> %08lx\n",
//                    *EaLength);
//
//     } _SEH2_FINALLY {
//
//         DebugUnwind( FatGetEaLength );
//
//         //
//         //  Unpin the EaDirent and the EaSetHeader if pinned.
//         //
//
//         FatUnpinBcb( IrpContext, EaBcb );
//
//         FatUnpinEaRange( IrpContext, &EaSetRange );
//
//         //
//         //  Release the Fcb for the Ea file if locked.
//         //
//
//         if (LockedEaFcb) {
//
//             FatReleaseFcb( IrpContext, Vcb->EaFcb );
//         }
//
//         DebugTrace(-1, Dbg, "FatGetEaLength:  Ea length -> %08lx\n", *EaLength);
//     } _SEH2_END;
//
//     return;
// }
//
//
// _Requires_lock_held_(_Global_critical_region_)
// VOID
// FatGetNeedEaCount (
//     IN PIRP_CONTEXT IrpContext,
//     IN PVCB Vcb,
//     IN PDIRENT Dirent,
//     OUT PULONG NeedEaCount
//     )
//
// /*++
//
// Routine Description:
//
//     This routine looks up the Need Ea count for the file.  The value is the
//     in the ea header for the file.
//
// Arguments:
//
//     Vcb - Vcb for the volume containing the Eas.
//
//     Dirent - Supplies a pointer to the dirent for the file in question.
//
//     NeedEaCount - Supplies the address to store the Need Ea count.
//
// Return Value:
//
//     None
//
// --*/
//
// {
//     PBCB EaBcb = NULL;
//     BOOLEAN LockedEaFcb = FALSE;
//     EA_RANGE EaSetRange;
//
//     PAGED_CODE();
//
//     DebugTrace(+1, Dbg, "FatGetNeedEaCount ...\n", 0);
//
//     //
//     //  If the handle is 0 then the Need Ea count is 0.
//     //
//
//     if (Dirent->ExtendedAttributes == 0) {
//
//         *NeedEaCount = 0;
//         DebugTrace(-1, Dbg, "FatGetNeedEaCount -> %08lx\n", TRUE);
//         return;
//     }
//
//     RtlZeroMemory( &EaSetRange, sizeof( EA_RANGE ));
//
//     //
//     //  Use a try to facilitate cleanup.
//     //
//
//     _SEH2_TRY {
//
//         PDIRENT EaDirent;
//         OEM_STRING ThisFilename;
//         UCHAR Buffer[12];
//         PEA_SET_HEADER EaSetHeader;
//
//         //
//         //  Initial the local values.
//         //
//
//         EaBcb = NULL;
//         LockedEaFcb = FALSE;
//
//         //
//         //  Try to get the Ea file object.  Return FALSE on failure.
//         //
//
//         FatGetEaFile( IrpContext,
//                       Vcb,
//                       &EaDirent,
//                       &EaBcb,
//                       FALSE,
//                       FALSE );
//
//         LockedEaFcb = TRUE;
//
//         //
//         //  If we didn't get the file because it doesn't exist, then the
//         //  disk is corrupted.
//         //
//
//         if (Vcb->VirtualEaFile == NULL) {
//
//             DebugTrace(0, Dbg, "FatGetNeedEaCount:  Ea file doesn't exist\n", 0);
//             FatRaiseStatus( IrpContext, STATUS_NO_EAS_ON_FILE );
//         }
//
//         //
//         //  Try to pin down the Ea set header for the index in the
//         //  dirent.  If the operation doesn't complete, return FALSE
//         //  from this routine.
//         //
//
//         ThisFilename.Buffer = (PCHAR)Buffer;
//         Fat8dot3ToString( IrpContext, Dirent, FALSE, &ThisFilename );
//
//         FatReadEaSet( IrpContext,
//                       Vcb,
//                       Dirent->ExtendedAttributes,
//                       &ThisFilename,
//                       FALSE,
//                       &EaSetRange );
//
//         EaSetHeader = (PEA_SET_HEADER) EaSetRange.Data;
//
//         //
//         //  We now have the Ea set header for this file.  We simply copy
//         //  the Need Ea field.
//         //
//
//         *NeedEaCount = EaSetHeader->NeedEaCount;
//
//     } _SEH2_FINALLY {
//
//         DebugUnwind( FatGetNeedEaCount );
//
//         //
//         //  Unpin the EaDirent and the EaSetHeader if pinned.
//         //
//
//         FatUnpinBcb( IrpContext, EaBcb );
//
//         FatUnpinEaRange( IrpContext, &EaSetRange );
//
//         //
//         //  Release the Fcb for the Ea file if locked.
//         //
//
//         if (LockedEaFcb) {
//
//             FatReleaseFcb( IrpContext, Vcb->EaFcb );
//         }
//
//         DebugTrace(-1, Dbg, "FatGetNeedEaCount:  NeedEaCount -> %08lx\n", *NeedEaCount);
//     } _SEH2_END;
//
//     return;
// }
//
//
// _Requires_lock_held_(_Global_critical_region_)
// VOID
// FatCreateEa (
//     IN PIRP_CONTEXT IrpContext,
//     IN PVCB Vcb,
//     IN PUCHAR Buffer,
//     IN ULONG Length,
//     IN POEM_STRING FileName,
//     OUT PUSHORT EaHandle
//     )
//
// /*++
//
// Routine Description:
//
//     This routine adds an entire ea set to the Ea file.  The owning file
//     is specified in 'FileName'.  This is used to replace the Ea set attached
//     to an existing file during a supersede operation.
//
//     NOTE: This routine may block, it should not be called unless the
//     thread is waitable.
//
// Arguments:
//
//     Vcb - Supplies the Vcb for the volume.
//
//     Buffer - Buffer with the Ea list to add.
//
//     Length - Length of the buffer.
//
//     FileName - The Ea's will be attached to this file.
//
//     EaHandle - The new ea handle will be assigned to this address.
//
// Return Value:
//
//     None
//
// --*/
//
// {
//     PBCB EaBcb;
//     BOOLEAN LockedEaFcb;
//
//     PEA_SET_HEADER EaSetHeader = NULL;
//     EA_RANGE EaSetRange;
//
//     PAGED_CODE();
//
//     DebugTrace(+1, Dbg, "FatCreateEa...\n", 0);
//
//     EaBcb = NULL;
//     LockedEaFcb = FALSE;
//
//     RtlZeroMemory( &EaSetRange, sizeof( EA_RANGE ));
//
//     //
//     //  Use 'try' to facilitate cleanup.
//     //
//
//     _SEH2_TRY {
//
//         PDIRENT EaDirent;
//
//         ULONG PackedEasLength;
//         ULONG AllocationLength;
//         ULONG BytesPerCluster;
//
//         PFILE_FULL_EA_INFORMATION FullEa;
//
//         //
//         //  We will allocate a buffer and copy the Ea list from the user's
//         //  buffer to a FAT packed Ea list.  Initial allocation is one
//         //  cluster, our starting offset into the packed Ea list is 0.
//         //
//
//         PackedEasLength = 0;
//
//         BytesPerCluster = 1 << Vcb->AllocationSupport.LogOfBytesPerCluster;
//
//         AllocationLength = (PackedEasLength
//                             + SIZE_OF_EA_SET_HEADER
//                             + BytesPerCluster - 1)
//                            & ~(BytesPerCluster - 1);
//
//         //
//         //  Allocate the memory and store the file name into it.
//         //
//
//         EaSetHeader = FsRtlAllocatePoolWithTag( PagedPool,
//                                                 AllocationLength,
//                                                 TAG_EA_SET_HEADER );
//
//         RtlZeroMemory( EaSetHeader, AllocationLength );
//
//         RtlCopyMemory( EaSetHeader->OwnerFileName,
//                        FileName->Buffer,
//                        FileName->Length );
//
//         AllocationLength -= SIZE_OF_EA_SET_HEADER;
//
//         //
//         //  Loop through the user's Ea list.  Catch any error for invalid
//         //  name or non-existent Ea value.
//         //
//
//         for ( FullEa = (PFILE_FULL_EA_INFORMATION) Buffer;
//               FullEa < (PFILE_FULL_EA_INFORMATION) &Buffer[Length];
//               FullEa = (PFILE_FULL_EA_INFORMATION) (FullEa->NextEntryOffset == 0 ?
//                                    &Buffer[Length] :
//                                    (PUCHAR) FullEa + FullEa->NextEntryOffset)) {
//
//             OEM_STRING EaName;
//             ULONG EaOffset;
//
//             EaName.Length = FullEa->EaNameLength;
//             EaName.Buffer = &FullEa->EaName[0];
//
//             //
//             //  Make sure the ea name is valid
//             //
//
//             if (!FatIsEaNameValid( IrpContext, EaName )) {
//
//                 DebugTrace(0, Dbg,
//                            "FatCreateEa:  Invalid Ea Name -> %Z\n",
//                            EaName);
//
//                 IrpContext->OriginatingIrp->IoStatus.Information = (PUCHAR)FullEa - Buffer;
//                 IrpContext->OriginatingIrp->IoStatus.Status = STATUS_INVALID_EA_NAME;
//                 FatRaiseStatus( IrpContext, STATUS_INVALID_EA_NAME );
//             }
//
//             //
//             //  Check that no invalid ea flags are set.
//             //
//
//             //
//             //  TEMPCODE  We are returning STATUS_INVALID_EA_NAME
//             //  until a more appropriate error code exists.
//             //
//
//             if (FullEa->Flags != 0
//                 && FullEa->Flags != FILE_NEED_EA) {
//
//                 IrpContext->OriginatingIrp->IoStatus.Information = (PUCHAR)FullEa - Buffer;
//                 IrpContext->OriginatingIrp->IoStatus.Status = STATUS_INVALID_EA_NAME;
//                 FatRaiseStatus( IrpContext, STATUS_INVALID_EA_NAME );
//             }
//
//             //
//             //  If this is a duplicate name then delete the current ea
//             //  value.
//             //
//
//             if (FatLocateEaByName( IrpContext,
//                                    (PPACKED_EA) EaSetHeader->PackedEas,
//                                    PackedEasLength,
//                                    &EaName,
//                                    &EaOffset )) {
//
//                 DebugTrace(0, Dbg, "FatCreateEa:  Duplicate name found\n", 0);
//
//                 FatDeletePackedEa( IrpContext,
//                                    EaSetHeader,
//                                    &PackedEasLength,
//                                    EaOffset );
//             }
//
//             //
//             //  We ignore this value if the eavalue length is zero.
//             //
//
//             if (FullEa->EaValueLength == 0) {
//
//                 DebugTrace(0, Dbg,
//                            "FatCreateEa:  Empty ea\n",
//                            0);
//
//                 continue;
//             }
//
//             FatAppendPackedEa( IrpContext,
//                                &EaSetHeader,
//                                &PackedEasLength,
//                                &AllocationLength,
//                                FullEa,
//                                BytesPerCluster );
//         }
//
//         //
//         //  If the resulting length isn't zero, then allocate a FAT cluster
//         //  to store the data.
//         //
//
//         if (PackedEasLength != 0) {
//
//             PEA_SET_HEADER NewEaSetHeader;
//
//             //
//             //  If the packed eas length (plus 4 bytes) is greater
//             //  than the maximum allowed ea size, we return an error.
//             //
//
//             if (PackedEasLength + 4 > MAXIMUM_EA_SIZE) {
//
//                 DebugTrace( 0, Dbg, "Ea length is greater than maximum\n", 0 );
//
//                 FatRaiseStatus( IrpContext, STATUS_EA_TOO_LARGE );
//             }
//
//             //
//             //  Get the Ea file.
//             //
//
//             FatGetEaFile( IrpContext,
//                           Vcb,
//                           &EaDirent,
//                           &EaBcb,
//                           TRUE,
//                           TRUE );
//
//             LockedEaFcb = TRUE;
//
//             FatAddEaSet( IrpContext,
//                          Vcb,
//                          PackedEasLength + SIZE_OF_EA_SET_HEADER,
//                          EaBcb,
//                          EaDirent,
//                          EaHandle,
//                          &EaSetRange );
//
//             NewEaSetHeader = (PEA_SET_HEADER) EaSetRange.Data;
//
//             //
//             //  Store the length of the new Ea's into the NewEaSetHeader.
//             //  This is the PackedEasLength + 4.
//             //
//
//             PackedEasLength += 4;
//
//             CopyU4char( EaSetHeader->cbList, &PackedEasLength );
//
//             //
//             //  Copy all but the first four bytes of EaSetHeader into
//             //  the new ea.  The signature and index fields have
//             //  already been filled in.
//             //
//
//             RtlCopyMemory( &NewEaSetHeader->NeedEaCount,
//                            &EaSetHeader->NeedEaCount,
//                            PackedEasLength + SIZE_OF_EA_SET_HEADER - 8 );
//
//             FatMarkEaRangeDirty( IrpContext, Vcb->VirtualEaFile, &EaSetRange );
//             FatUnpinEaRange( IrpContext, &EaSetRange );
//
//             CcFlushCache( Vcb->VirtualEaFile->SectionObjectPointer, NULL, 0, NULL );
//
//         //
//         //  There was no data added to the Ea file.  Return a handle
//         //  of 0.
//         //
//
//         } else {
//
//             *EaHandle = 0;
//         }
//
//     } _SEH2_FINALLY {
//
//         DebugUnwind( FatCreateEa );
//
//         //
//         //  Deallocate the EaSetHeader if present.
//         //
//
//         if (EaSetHeader) {
//
//             ExFreePool( EaSetHeader );
//         }
//
//         //
//         //  Release the EaFcb if held.
//         //
//
//         if (LockedEaFcb) {
//
//             FatReleaseFcb( IrpContext, Vcb->EaFcb );
//         }
//
//         //
//         //  Unpin the dirents for the EaFcb and EaSetFcb if necessary.
//         //
//
//         FatUnpinBcb( IrpContext, EaBcb );
//         FatUnpinEaRange( IrpContext, &EaSetRange );
//
//         DebugTrace(-1, Dbg, "FatCreateEa -> Exit\n", 0);
//     } _SEH2_END;
//
//     return;
// }
//
// _Requires_lock_held_(_Global_critical_region_)
// VOID
// FatDeleteEa (
//     IN PIRP_CONTEXT IrpContext,
//     IN PVCB Vcb,
//     IN USHORT EaHandle,
//     IN POEM_STRING FileName
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is called to remove an entire ea set.  Most of the work
//     is done in the call to 'FatDeleteEaSet'.  This routine opens the
//     Ea file and then calls the support routine.
//
//     NOTE: This routine may block, it should not be called unless the
//     thread is waitable.
//
// Arguments:
//
//     Vcb - Vcb for the volume
//
//     EaHandle - The handle for the Ea's to remove.  This handle will be
//                verified during this operation.
//
//     FileName - The name of the file whose Ea's are being removed.  This
//                name is compared against the Ea owner's name in the Ea set.
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     PBCB EaBcb;
//     BOOLEAN LockedEaFcb;
//
//     PAGED_CODE();
//
//     DebugTrace(+1, Dbg, "FatDeleteEa...\n", 0);
//
//     //
//     //  Initialize local values.
//     //
//
//     EaBcb = NULL;
//     LockedEaFcb = FALSE;
//
//     //
//     //  Use a try statement to facilitate cleanup.
//     //
//
//     _SEH2_TRY {
//
//         PDIRENT EaDirent;
//
//         //
//         //  Get the Ea stream file.  If the file doesn't exist on the disk
//         //  then the disk has been corrupted.
//         //
//
//         FatGetEaFile( IrpContext,
//                       Vcb,
//                       &EaDirent,
//                       &EaBcb,
//                       FALSE,
//                       TRUE );
//
//         LockedEaFcb = TRUE;
//
//         //
//         //  If we didn't get the Ea file, then the disk is corrupt.
//         //
//
//         if ( EaBcb == NULL ) {
//
//
//             DebugTrace(0, Dbg,
//                        "FatDeleteEa:  No Ea file exists\n",
//                        0);
//
//             FatRaiseStatus( IrpContext, STATUS_NO_EAS_ON_FILE );
//         }
//
//         //
//         //  We now have everything we need to delete the ea set.  Call the
//         //  support routine to do this.
//         //
//
//         FatDeleteEaSet( IrpContext,
//                         Vcb,
//                         EaBcb,
//                         EaDirent,
//                         EaHandle,
//                         FileName );
//
//         CcFlushCache( Vcb->VirtualEaFile->SectionObjectPointer, NULL, 0, NULL );
//
//     } _SEH2_FINALLY {
//
//         DebugUnwind( FatDeleteEa );
//
//         //
//         //  Release the EaFcb if held.
//         //
//
//         if (LockedEaFcb) {
//
//             FatReleaseFcb( IrpContext, Vcb->EaFcb );
//         }
//
//         //
//         //  Unpin the dirent for the Ea file if pinned.
//         //
//
//         FatUnpinBcb( IrpContext, EaBcb );
//
//         DebugTrace(-1, Dbg, "FatDeleteEa -> Exit\n", 0);
//     } _SEH2_END;
//
//     return;
// }
//
//
// _Requires_lock_held_(_Global_critical_region_)
// VOID
// FatGetEaFile (
//     IN PIRP_CONTEXT IrpContext,
//     IN OUT PVCB Vcb,
//     OUT PDIRENT *EaDirent,
//     OUT PBCB *EaBcb,
//     IN BOOLEAN CreateFile,
//     IN BOOLEAN ExclusiveFcb
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is used to completely initialize the Vcb and
//     the Ea file for the Vcb.
//
//     If the Vcb doesn't have the Ea file object, then we first try to
//     lookup the Ea data file in the root directory and if that fails
//     we try to create the file.  The 'CreateFile' flag is used to check
//     whether it is necessary to create the Ea file.
//
//     This routine will lock down the Fcb for exclusive or shared access before
//     performing any operations.  If the operation does not complete due
//     to blocking, exclusive or shared access will be given up before returning.
//
//     If we are creating the Ea file and marking sections of it dirty,
//     we can't use the repin feature through the cache map.  In that case
//     we use a local IrpContext and then unpin all of the Bcb's before
//     continuing.
//
//     Note: If this routine will be creating the Ea file, we are guaranteed
//     to be waitable.
//
// Arguments:
//
//     Vcb - Vcb for the volume
//
//     EaDirent - Location to store the address of the pinned dirent for the
//                Ea file.
//
//     EaBcb - Location to store the address of the Bcb for the pinned dirent.
//
//     CreateFile - Boolean indicating whether we should create the Ea file
//                  on the disk.
//
//     ExclusiveFcb - Indicates whether shared or exclusive access is desired
//                    for the EaFcb.
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     PFILE_OBJECT EaStreamFile = NULL;
//     EA_RANGE EaFileRange;
//
//     BOOLEAN UnwindLockedEaFcb = FALSE;
//     BOOLEAN UnwindLockedRootDcb = FALSE;
//     BOOLEAN UnwindAllocatedDiskSpace = FALSE;
//     BOOLEAN UnwindEaDirentCreated = FALSE;
//     BOOLEAN UnwindUpdatedSizes = FALSE;
//
//     PAGED_CODE();
//
//     DebugTrace(+1, Dbg, "FatGetEaFile ...\n", 0);
//
//     RtlZeroMemory( &EaFileRange, sizeof( EA_RANGE ));
//
//     //
//     //  Use a try to facilitate cleanup
//     //
//
//     _SEH2_TRY {
//
//         OEM_STRING EaFileName;
//         LARGE_INTEGER SectionSize;
//
//         //
//         //  Check if the Vcb already has the file object.  If it doesn't, then
//         //  we need to search the root directory for the Ea data file.
//         //
//
//         if (Vcb->VirtualEaFile == NULL) {
//
//             //
//             //  Always lock the Ea file exclusively if we have to create the file.
//             //
//
//             if ( !FatAcquireExclusiveFcb( IrpContext, Vcb->EaFcb )) {
//
//                 DebugTrace(0, Dbg, "FatGetEaFile:  Can't grab exclusive\n", 0);
//                 FatRaiseStatus( IrpContext, STATUS_CANT_WAIT );
//             }
//
//             UnwindLockedEaFcb = TRUE;
//
//         //
//         //  Otherwise we acquire the Fcb as the caller requested.
//         //
//
//         } else {
//
//             if ((ExclusiveFcb && !FatAcquireExclusiveFcb( IrpContext, Vcb->EaFcb ))
//                 || (!ExclusiveFcb && !FatAcquireSharedFcb( IrpContext, Vcb->EaFcb))) {
//
//                 DebugTrace(0, Dbg, "FatGetEaFile:  Can't grab EaFcb\n", 0);
//
//                 FatRaiseStatus( IrpContext, STATUS_CANT_WAIT );
//             }
//
//             UnwindLockedEaFcb = TRUE;
//
//             //
//             //  If the file now does not exist we need to release the Fcb and
//             //  reacquire exclusive if we acquired shared.
//             //
//
//             if ((Vcb->VirtualEaFile == NULL) && !ExclusiveFcb) {
//
//                 FatReleaseFcb( IrpContext, Vcb->EaFcb );
//                 UnwindLockedEaFcb = FALSE;
//
//                 if (!FatAcquireExclusiveFcb( IrpContext, Vcb->EaFcb )) {
//
//                     DebugTrace(0, Dbg, "FatGetEaFile:  Can't grab EaFcb\n", 0);
//
//                     FatRaiseStatus( IrpContext, STATUS_CANT_WAIT );
//                 }
//
//                 UnwindLockedEaFcb = TRUE;
//             }
//         }
//
//         //
//         //  If the file object is now there we only need to get the
//         //  dirent for the Ea file.
//         //
//
//         if (Vcb->VirtualEaFile != NULL) {
//
//             FatVerifyFcb( IrpContext, Vcb->EaFcb );
//
//             FatGetDirentFromFcbOrDcb( IrpContext,
//                                       Vcb->EaFcb,
//                                       FALSE,
//                                       EaDirent,
//                                       EaBcb );
//
//             try_return( NOTHING );
//
//         } else {
//
//             VBO ByteOffset = 0;
//
//             //
//             //  Always mark the ea fcb as good.
//             //
//
//             Vcb->EaFcb->FcbCondition = FcbGood;
//
//             //
//             //  We try to lookup the dirent for the Ea Fcb.
//             //
//
//             EaFileName.Buffer = "EA DATA. SF";
//             EaFileName.Length = 11;
//             EaFileName.MaximumLength = 12;
//
//             //
//             //  Now pick up the root directory to be synchronized with
//             //  deletion/creation of entries.  If we may create the file,
//             //  get it exclusive right now.
//             //
//             //  Again, note how we are relying on bottom-up lockorder. We
//             //  already got the EaFcb.
//             //
//
//             if (CreateFile) {
//                 ExAcquireResourceExclusiveLite( Vcb->RootDcb->Header.Resource, TRUE );
//             } else {
//                 ExAcquireResourceSharedLite( Vcb->RootDcb->Header.Resource, TRUE );
//             }
//             UnwindLockedRootDcb = TRUE;
//
//             FatLocateSimpleOemDirent( IrpContext,
//                                       Vcb->EaFcb->ParentDcb,
//                                       &EaFileName,
//                                       EaDirent,
//                                       EaBcb,
//                                       &ByteOffset );
//
//             //
//             //  If the file exists, we need to create the virtual file
//             //  object for it.
//             //
//
//             if (*EaDirent != NULL) {
//
//                 //
//                 //  Since we may be modifying the dirent, pin the data now.
//                 //
//
//                 FatPinMappedData( IrpContext,
//                                   Vcb->EaFcb->ParentDcb,
//                                   ByteOffset,
//                                   sizeof(DIRENT),
//                                   EaBcb );
//
//                 //
//                 //  Update the Fcb with information on the file size
//                 //  and disk location.  Also increment the open/unclean
//                 //  counts in the EaFcb and the open count in the
//                 //  Vcb.
//                 //
//
//                 Vcb->EaFcb->FirstClusterOfFile = (*EaDirent)->FirstClusterOfFile;
//                 Vcb->EaFcb->DirentOffsetWithinDirectory = ByteOffset;
//
//                 //
//                 //  Find the allocation size.  The purpose here is
//                 //  really to completely fill in the Mcb for the
//                 //  file.
//                 //
//
//                 Vcb->EaFcb->Header.AllocationSize.QuadPart = FCB_LOOKUP_ALLOCATIONSIZE_HINT;
//
//                 FatLookupFileAllocationSize( IrpContext, Vcb->EaFcb );
//
//                 //
//                 //  Start by computing the section size for the cache
//                 //  manager.
//                 //
//
//                 SectionSize.QuadPart = (*EaDirent)->FileSize;
//                 Vcb->EaFcb->Header.AllocationSize = SectionSize;
//                 Vcb->EaFcb->Header.FileSize = SectionSize;
//
//                 //
//                 //  Create and initialize the file object for the
//                 //  Ea virtual file.
//                 //
//
//                 EaStreamFile = FatOpenEaFile( IrpContext, Vcb->EaFcb );
//
//                 Vcb->VirtualEaFile = EaStreamFile;
//
//             //
//             //  Else there was no dirent.  If we were instructed to
//             //  create the file object, we will try to create the dirent,
//             //  allocate disk space, initialize the Ea file header and
//             //  return this information to the user.
//             //
//
//             } else if (CreateFile) {
//
//                 ULONG BytesPerCluster;
//                 ULONG OffsetTableSize;
//                 ULONG AllocationSize;
//                 PEA_FILE_HEADER FileHeader;
//                 USHORT AllocatedClusters;
//                 PUSHORT CurrentIndex;
//                 ULONG Index;
//                 NTSTATUS Status = STATUS_SUCCESS;
//
//                 DebugTrace(0, Dbg, "FatGetEaFile:  Creating local IrpContext\n", 0);
//
//                 BytesPerCluster = 1 << Vcb->AllocationSupport.LogOfBytesPerCluster;
//
//                 AllocationSize = (((ULONG) sizeof( EA_FILE_HEADER ) << 1) + BytesPerCluster - 1)
//                                  & ~(BytesPerCluster - 1);
//
//                 AllocatedClusters = (USHORT) (AllocationSize
//                                     >> Vcb->AllocationSupport.LogOfBytesPerCluster);
//
//                 OffsetTableSize = AllocationSize - sizeof( EA_FILE_HEADER );
//
//                 //
//                 //  Allocate disk space, the space allocated is 1024 bytes
//                 //  rounded up to the nearest cluster size.
//                 //
//
//                 FatAllocateDiskSpace( IrpContext,
//                                       Vcb,
//                                       0,
//                                       &AllocationSize,
//                                       FALSE,
//                                       &Vcb->EaFcb->Mcb );
//
//                 UnwindAllocatedDiskSpace = TRUE;
//
//                 //
//                 //  Allocate and initialize a dirent in the root directory
//                 //  to describe this new file.
//                 //
//
//                 Vcb->EaFcb->DirentOffsetWithinDirectory =
//                     FatCreateNewDirent( IrpContext,
//                                         Vcb->EaFcb->ParentDcb,
//                                         1,
//                                         FALSE );
//
//                 FatPrepareWriteDirectoryFile( IrpContext,
//                                               Vcb->EaFcb->ParentDcb,
//                                               Vcb->EaFcb->DirentOffsetWithinDirectory,
//                                               sizeof(DIRENT),
//                                               EaBcb,
// #ifndef __REACTOS__
//                                               EaDirent,
// #else
//                                               (PVOID *)EaDirent,
// #endif
//                                               FALSE,
//                                               TRUE,
//                                               &Status );
//
//                 NT_ASSERT( NT_SUCCESS( Status ));
//
//                 UnwindEaDirentCreated = TRUE;
//
//                 FatConstructDirent( IrpContext,
//                                     *EaDirent,
//                                     &EaFileName,
//                                     FALSE,
//                                     FALSE,
//                                     NULL,
//                                     FAT_DIRENT_ATTR_READ_ONLY
//                                     | FAT_DIRENT_ATTR_HIDDEN
//                                     | FAT_DIRENT_ATTR_SYSTEM
//                                     | FAT_DIRENT_ATTR_ARCHIVE,
//                                     TRUE,
//                                     NULL );
//
//                 (*EaDirent)->FileSize = AllocationSize;
//
//                 //
//                 //  Initialize the Fcb for this file and initialize the
//                 //  cache map as well.
//                 //
//
//                 //
//                 //  Start by computing the section size for the cache
//                 //  manager.
//                 //
//
//                 SectionSize.QuadPart = (*EaDirent)->FileSize;
//                 Vcb->EaFcb->Header.AllocationSize = SectionSize;
//                 Vcb->EaFcb->Header.FileSize = SectionSize;
//                 UnwindUpdatedSizes = TRUE;
//
//                 //
//                 //  Create and initialize the file object for the
//                 //  Ea virtual file.
//                 //
//
//                 EaStreamFile = FatOpenEaFile( IrpContext, Vcb->EaFcb );
//
//                 //
//                 //  Update the Fcb with information on the file size
//                 //  and disk location.  Also increment the open/unclean
//                 //  counts in the EaFcb and the open count in the
//                 //  Vcb.
//                 //
//
//                 {
//                     LBO FirstLboOfFile;
//
//                     FatLookupMcbEntry( Vcb, &Vcb->EaFcb->Mcb,
//                                        0,
//                                        &FirstLboOfFile,
//                                        NULL,
//                                        NULL );
//
//                     //
//                     //  The discerning reader will note that this doesn't take
//                     //  FAT32 into account, which is of course intentional.
//                     //
//
//                     (*EaDirent)->FirstClusterOfFile =
//                         (USHORT) FatGetIndexFromLbo( Vcb, FirstLboOfFile );
//                 }
//
//                 Vcb->EaFcb->FirstClusterOfFile = (*EaDirent)->FirstClusterOfFile;
//
//                 //
//                 //  Initialize the Ea file header and mark the Bcb as dirty.
//                 //
//
//                 FatPinEaRange( IrpContext,
//                                EaStreamFile,
//                                Vcb->EaFcb,
//                                &EaFileRange,
//                                0,
//                                AllocationSize,
//                                STATUS_DATA_ERROR );
//
//                 FileHeader = (PEA_FILE_HEADER) EaFileRange.Data;
//
//                 RtlZeroMemory( FileHeader, AllocationSize );
//                 FileHeader->Signature = EA_FILE_SIGNATURE;
//
//                 for (Index = MAX_EA_BASE_INDEX, CurrentIndex = FileHeader->EaBaseTable;
//                      Index;
//                      Index--, CurrentIndex++) {
//
//                     *CurrentIndex = AllocatedClusters;
//                 }
//
//                 //
//                 //  Initialize the offset table with the offset set to
//                 //  after the just allocated clusters.
//                 //
//
//                 for (Index = OffsetTableSize >> 1,
//                         CurrentIndex = (PUSHORT) ((PUCHAR) FileHeader + sizeof( EA_FILE_HEADER ));
//                      Index;
//                      Index--, CurrentIndex++) {
//
//                     *CurrentIndex = UNUSED_EA_HANDLE;
//                 }
//
//                 //
//                 //  Unpin the file header and offset table.
//                 //
//
//                 FatMarkEaRangeDirty( IrpContext, EaStreamFile, &EaFileRange );
//                 FatUnpinEaRange( IrpContext, &EaFileRange );
//
//                 CcFlushCache( EaStreamFile->SectionObjectPointer, NULL, 0, NULL );
//
//                 //
//                 //  Return the Ea file object to the user.
//                 //
//
//                 Vcb->VirtualEaFile = EaStreamFile;
//             }
//         }
//     try_exit:  NOTHING;
//     } _SEH2_FINALLY {
//
//         DebugUnwind( FatGetEaFile );
//
//         //
//         //  If this is abnormal termination and disk space has been
//         //  allocated.  We deallocate it now.
//         //
//
//         if (_SEH2_AbnormalTermination()) {
//
//             //
//             //  Deallocate the Ea file
//             //
//
//             if (UnwindAllocatedDiskSpace) {
//
//                 FatDeallocateDiskSpace( IrpContext,
//                                         Vcb,
//                                         &Vcb->EaFcb->Mcb,
//                                         FALSE );
//             }
//
//             //
//             //  Delete the dirent for the Ea file, if created.
//             //
//
//             if (UnwindEaDirentCreated) {
//
//                 if (UnwindUpdatedSizes) {
//
//                     Vcb->EaFcb->Header.AllocationSize.QuadPart = 0;
//                     Vcb->EaFcb->Header.FileSize.QuadPart = 0;
//                 }
//
//                 FatUnpinBcb( IrpContext, *EaBcb );
//                 FatDeleteDirent( IrpContext, Vcb->EaFcb, NULL, TRUE );
//             }
//
//             //
//             //  Release the EA Fcb if held
//             //
//
//             if (UnwindLockedEaFcb) {
//
//                 FatReleaseFcb( IrpContext, Vcb->EaFcb );
//             }
//
//             //
//             //  Dereference the Ea stream file if created.
//             //
//
//             if (EaStreamFile != NULL) {
//
//                 ObDereferenceObject( EaStreamFile );
//             }
//         }
//
//         //
//         //  Always release the root Dcb (our caller releases the EA Fcb if we
//         //  do not raise).
//         //
//
//         if (UnwindLockedRootDcb) {
//
//             FatReleaseFcb( IrpContext, Vcb->RootDcb );
//         }
//
//         //
//         //  If the Ea file header is locked down.  We unpin it now.
//         //
//
//         FatUnpinEaRange( IrpContext, &EaFileRange );
//
//         DebugTrace(-1, Dbg, "FatGetEaFile:  Exit\n", 0);
//     } _SEH2_END;
//
//     return;
// }
//
//
// VOID
// FatReadEaSet (
//     IN PIRP_CONTEXT IrpContext,
//     IN PVCB Vcb,
//     IN USHORT EaHandle,
//     IN POEM_STRING FileName,
//     IN BOOLEAN ReturnEntireSet,
//     OUT PEA_RANGE EaSetRange
//     )
//
// /*++
//
// Routine Description:
//
//     This routine pins the Ea set for the given ea handle within the
//     Ea stream file.  The EaHandle, after first comparing against valid
//     index values, is used to compute the cluster offset for this
//     this Ea set.  The Ea set is then verified as belonging to this
//     index and lying within the Ea data file.
//
//     The caller of this function will have verified that the Ea file
//     exists and that the Vcb field points to an initialized cache file.
//     The caller will already have gained exclusive access to the
//     EaFcb.
//
// Arguments:
//
//     Vcb - Supplies the Vcb for the volume.
//
//     EaHandle - Supplies the handle for the Ea's to read.
//
//     FileName - Name of the file whose Ea's are being read.
//
//     ReturnEntireSet - Indicates if the caller needs the entire set
//         as opposed to just the header.
//
//     EaSetRange - Pointer to the EaRange structure which will describe the Ea
//         on return.
//
// Return Value:
//
//     None
//
// --*/
//
// {
//     ULONG BytesPerCluster = 1 << Vcb->AllocationSupport.LogOfBytesPerCluster;
//
//     ULONG EaOffsetVbo;
//     EA_RANGE EaOffsetRange;
//     USHORT EaOffsetCluster;
//
//     EA_RANGE EaHeaderRange;
//     PEA_FILE_HEADER EaHeader;
//
//     ULONG EaSetVbo;
//     PEA_SET_HEADER EaSet;
//
//     ULONG CbList;
//
//     PAGED_CODE();
//
//     UNREFERENCED_PARAMETER( FileName );
//
//     DebugTrace(+1, Dbg, "FatReadEaSet\n", 0);
//     DebugTrace( 0, Dbg, "  Vcb      = %p\n", Vcb);
//
//     //
//     //  Verify that the Ea index has a legal value.  Raise status
//     //  STATUS_NONEXISTENT_EA_ENTRY if illegal.
//     //
//
//     if (EaHandle < MIN_EA_HANDLE
//         || EaHandle > MAX_EA_HANDLE) {
//
//         DebugTrace(-1, Dbg, "FatReadEaSet: Illegal handle value\n", 0);
//         FatRaiseStatus( IrpContext, STATUS_NONEXISTENT_EA_ENTRY );
//     }
//
//     //
//     //  Verify that the virtual Ea file is large enough for us to read
//     //  the EaOffet table for this index.
//     //
//
//     EaOffsetVbo = sizeof( EA_FILE_HEADER ) + (((ULONGLONG)EaHandle >> 7) << 8);
//
//     //
//     //  Zero the Ea range structures.
//     //
//
//     RtlZeroMemory( &EaHeaderRange, sizeof( EA_RANGE ));
//     RtlZeroMemory( &EaOffsetRange, sizeof( EA_RANGE ));
//
//     //
//     //  Use a try statement to clean up on exit.
//     //
//
//     _SEH2_TRY {
//
//         //
//         //  Pin down the EA file header.
//         //
//
//         FatPinEaRange( IrpContext,
//                        Vcb->VirtualEaFile,
//                        Vcb->EaFcb,
//                        &EaHeaderRange,
//                        0,
//                        sizeof( EA_FILE_HEADER ),
//                        STATUS_NONEXISTENT_EA_ENTRY );
//
//         EaHeader = (PEA_FILE_HEADER) EaHeaderRange.Data;
//
//         //
//         //  Pin down the Ea offset table for the particular index.
//         //
//
//         FatPinEaRange( IrpContext,
//                        Vcb->VirtualEaFile,
//                        Vcb->EaFcb,
//                        &EaOffsetRange,
//                        EaOffsetVbo,
//                        sizeof( EA_OFF_TABLE ),
//                        STATUS_NONEXISTENT_EA_ENTRY );
//
//         //
//         //  Check if the specifific handle is currently being used.
//         //
//
//         EaOffsetCluster = *((PUSHORT) EaOffsetRange.Data
//                             + (EaHandle & (MAX_EA_OFFSET_INDEX - 1)));
//
//         if (EaOffsetCluster == UNUSED_EA_HANDLE) {
//
//             DebugTrace(0, Dbg, "FatReadEaSet: Ea handle is unused\n", 0);
//             FatRaiseStatus( IrpContext, STATUS_NONEXISTENT_EA_ENTRY );
//         }
//
//         //
//         //  Compute the file offset for the Ea data.
//         //
//
//         EaSetVbo = (EaHeader->EaBaseTable[EaHandle >> 7] + EaOffsetCluster)
//                    << Vcb->AllocationSupport.LogOfBytesPerCluster;
//
//         //
//         //  Unpin the file header and offset table.
//         //
//
//         FatUnpinEaRange( IrpContext, &EaHeaderRange );
//         FatUnpinEaRange( IrpContext, &EaOffsetRange );
//
//         //
//         //  Pin the ea set.
//         //
//
//         FatPinEaRange( IrpContext,
//                        Vcb->VirtualEaFile,
//                        Vcb->EaFcb,
//                        EaSetRange,
//                        EaSetVbo,
//                        BytesPerCluster,
//                        STATUS_DATA_ERROR );
//
//         //
//         //  Verify that the Ea set is valid and belongs to this index.
//         //  Raise STATUS_DATA_ERROR if there is a data conflict.
//         //
//
//         EaSet = (PEA_SET_HEADER) EaSetRange->Data;
//
//         if (EaSet->Signature != EA_SET_SIGNATURE
//             || EaSet->OwnEaHandle != EaHandle ) {
//
//             DebugTrace(0, Dbg, "FatReadEaSet: Ea set header is corrupt\n", 0);
//             FatRaiseStatus( IrpContext, STATUS_DATA_ERROR );
//         }
//
//         //
//         //  At this point we have pinned a single cluster of Ea data.  If
//         //  this represents the entire Ea data for the Ea index, we are
//         //  done.  Otherwise we need to check on the entire size of
//         //  of the Ea set header and whether it is contained in the allocated
//         //  size of the Ea virtual file.  At that point we can unpin
//         //  the partial Ea set header and repin the entire header.
//         //
//
//         CbList = GetcbList( EaSet );
//
//         if (ReturnEntireSet
//             && CbList > BytesPerCluster ) {
//
//             //
//             //  Round up to the cluster size.
//             //
//
//             CbList = (CbList + EA_CBLIST_OFFSET + BytesPerCluster - 1)
//                      & ~(BytesPerCluster - 1);
//
//             FatUnpinEaRange( IrpContext, EaSetRange );
//
//             RtlZeroMemory( EaSetRange, sizeof( EA_RANGE ));
//
//             FatPinEaRange( IrpContext,
//                            Vcb->VirtualEaFile,
//                            Vcb->EaFcb,
//                            EaSetRange,
//                            EaSetVbo,
//                            CbList,
//                            STATUS_DATA_ERROR );
//         }
//
//     } _SEH2_FINALLY {
//
//         DebugUnwind( FatReadEaSet );
//
//         //
//         //  Unpin the Ea base and offset tables if locked down.
//         //
//
//         FatUnpinEaRange( IrpContext, &EaHeaderRange );
//         FatUnpinEaRange( IrpContext, &EaOffsetRange );
//
//         DebugTrace(-1, Dbg, "FatReadEaSet:  Exit\n", 0);
//     } _SEH2_END;
//
//     return;
// }
//
//
// _Requires_lock_held_(_Global_critical_region_)
// VOID
// FatDeleteEaSet (
//     IN PIRP_CONTEXT IrpContext,
//     IN PVCB Vcb,
//     IN PBCB EaBcb,
//     OUT PDIRENT EaDirent,
//     IN USHORT EaHandle,
//     IN POEM_STRING FileName
//     )
//
// /*++
//
// Routine Description:
//
//     This routines clips the Ea set for a particular index out of the
//     Ea file for a volume.  The index is verified as belonging to a valid
//     handle.  The clusters are removed and the Ea stream file along with
//     the Ea base and offset files are updated.
//
//     The caller of this function will have verified that the Ea file
//     exists and that the Vcb field points to an initialized cache file.
//     The caller will already have gained exclusive access to the
//     EaFcb.
//
// Arguments:
//
//     Vcb - Supplies the Vcb for the volume.
//
//     VirtualEeFile - Pointer to the file object for the virtual Ea file.
//
//     EaFcb - Supplies the pointer to the Fcb for the Ea file.
//
//     EaBcb - Supplies a pointer to the Bcb for the Ea dirent.
//
//     EaDirent - Supplies a pointer to the dirent for the Ea file.
//
//     EaHandle - Supplies the handle for the Ea's to read.
//
//     FileName - Name of the file whose Ea's are being read.
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     ULONG BytesPerCluster = 1 << Vcb->AllocationSupport.LogOfBytesPerCluster;
//     ULONG CbList;
//     LARGE_INTEGER FileOffset;
//
//     LARGE_MCB DataMcb;
//     BOOLEAN UnwindInitializeDataMcb = FALSE;
//     BOOLEAN UnwindSplitData = FALSE;
//
//     LARGE_MCB TailMcb;
//     BOOLEAN UnwindInitializeTailMcb = FALSE;
//     BOOLEAN UnwindSplitTail = FALSE;
//     BOOLEAN UnwindMergeTail = FALSE;
//
//     BOOLEAN UnwindModifiedEaHeader = FALSE;
//     BOOLEAN UnwindCacheValues = FALSE;
//     ULONG UnwindPrevFileSize = 0;
//
//     ULONG EaOffsetVbo;
//     USHORT EaOffsetIndex;
//     EA_RANGE EaOffsetRange;
//     USHORT EaOffsetCluster;
//
//     PFILE_OBJECT VirtualEaFile = Vcb->VirtualEaFile;
//     PFCB EaFcb = Vcb->EaFcb;
//
//     EA_RANGE EaHeaderRange;
//     PEA_FILE_HEADER EaHeader;
//     USHORT EaHeaderBaseIndex;
//
//     ULONG EaSetVbo = 0;
//     ULONG EaSetLength;
//     EA_RANGE EaSetRange;
//     PEA_SET_HEADER EaSet;
//     USHORT EaSetClusterCount;
//
//     PAGED_CODE();
//
//     UNREFERENCED_PARAMETER( FileName );
//
//     //
//     //  Verify that the Ea index has a legal value.  Raise status
//     //  STATUS_INVALID_HANDLE if illegal.
//     //
//
//     if (EaHandle < MIN_EA_HANDLE
//         || EaHandle > MAX_EA_HANDLE) {
//
//         DebugTrace(-1, Dbg, "FatDeleteEaSet: Illegal handle value\n", 0);
//         FatRaiseStatus( IrpContext, STATUS_NONEXISTENT_EA_ENTRY );
//     }
//
//     //
//     //  Verify that the virtual Ea file is large enough for us to read
//     //  the EaOffet table for this index.
//     //
//
//     EaOffsetVbo = sizeof( EA_FILE_HEADER ) + (((ULONGLONG)EaHandle >> 7) << 8);
//
//     //
//     //  Zero the Ea range structures.
//     //
//
//     RtlZeroMemory( &EaHeaderRange, sizeof( EA_RANGE ));
//     RtlZeroMemory( &EaOffsetRange, sizeof( EA_RANGE ));
//     RtlZeroMemory( &EaSetRange, sizeof( EA_RANGE ));
//
//     //
//     //  Use a try to facilitate cleanup.
//     //
//
//     _SEH2_TRY {
//
//         //
//         //  Pin down the EA file header.
//         //
//
//         FatPinEaRange( IrpContext,
//                        VirtualEaFile,
//                        EaFcb,
//                        &EaHeaderRange,
//                        0,
//                        sizeof( EA_FILE_HEADER ),
//                        STATUS_NONEXISTENT_EA_ENTRY );
//
//         EaHeader = (PEA_FILE_HEADER) EaHeaderRange.Data;
//
//         //
//         //  Pin down the Ea offset table for the particular index.
//         //
//
//         FatPinEaRange( IrpContext,
//                        VirtualEaFile,
//                        EaFcb,
//                        &EaOffsetRange,
//                        EaOffsetVbo,
//                        sizeof( EA_OFF_TABLE ),
//                        STATUS_NONEXISTENT_EA_ENTRY );
//
//         //
//         //  Check if the specifific handle is currently being used.
//         //
//
//         EaOffsetIndex = EaHandle & (MAX_EA_OFFSET_INDEX - 1);
//         EaOffsetCluster = *((PUSHORT) EaOffsetRange.Data + EaOffsetIndex);
//
//         if (EaOffsetCluster == UNUSED_EA_HANDLE) {
//
//             DebugTrace(0, Dbg, "FatReadEaSet: Ea handle is unused\n", 0);
//             FatRaiseStatus( IrpContext, STATUS_NONEXISTENT_EA_ENTRY );
//         }
//
//         //
//         //  Compute the file offset for the Ea data.
//         //
//
//         EaHeaderBaseIndex = EaHandle >> 7;
//         EaSetVbo = (EaHeader->EaBaseTable[EaHeaderBaseIndex] + EaOffsetCluster)
//                    << Vcb->AllocationSupport.LogOfBytesPerCluster;
//
//         //
//         //  Unpin the file header and offset table.
//         //
//
//         FatUnpinEaRange( IrpContext, &EaHeaderRange );
//         FatUnpinEaRange( IrpContext, &EaOffsetRange );
//
//         //
//         //  Try to pin the requested Ea set.
//         //
//
//         FatPinEaRange( IrpContext,
//                        VirtualEaFile,
//                        EaFcb,
//                        &EaSetRange,
//                        EaSetVbo,
//                        BytesPerCluster,
//                        STATUS_DATA_ERROR );
//
//         EaSet = (PEA_SET_HEADER) EaSetRange.Data;
//
//         if (EaSet->Signature != EA_SET_SIGNATURE
//             || EaSet->OwnEaHandle != EaHandle ) {
//
//             DebugTrace(0, Dbg, "FatReadEaSet: Ea set header is corrupt\n", 0);
//             FatRaiseStatus( IrpContext, STATUS_DATA_ERROR );
//         }
//
//         //
//         //  At this point we have pinned a single cluster of Ea data.  If
//         //  this represents the entire Ea data for the Ea index, we know
//         //  the number of clusters to remove.  Otherwise we need to check
//         //  on the entire size of the Ea set header and whether it is
//         //  contained in the allocated size of the Ea virtual file.  At
//         //  that point we unpin the partial Ea set header and remember the
//         //  starting cluster offset and number of clusters in both cluster
//         //  and Vbo formats.
//         //
//         //  At that point the following variables have the described
//         //  values.
//         //
//         //      EaSetVbo - Vbo to start splice at.
//         //      EaSetLength - Number of bytes to splice.
//         //      EaSetClusterCount - Number of clusters to splice.
//         //
//
//         CbList = GetcbList( EaSet );
//
//         EaSetClusterCount = (USHORT) ((CbList + EA_CBLIST_OFFSET + BytesPerCluster - 1)
//                                       >> Vcb->AllocationSupport.LogOfBytesPerCluster);
//
//         EaSetLength = EaSetClusterCount << Vcb->AllocationSupport.LogOfBytesPerCluster;
//
//         if (EaSetLength > BytesPerCluster) {
//
//             if (EaFcb->Header.FileSize.LowPart - EaSetVbo < EaSetLength) {
//
//                 DebugTrace(0, Dbg, "FatDeleteEaSet: Full Ea set not contained in file\n", 0);
//
//                 FatRaiseStatus( IrpContext, STATUS_DATA_ERROR );
//             }
//         }
//
//         FatUnpinEaRange( IrpContext, &EaSetRange );
//
//         //
//         //  Update the cache manager for this file.  This is done by
//         //  truncating to the point where the data was spliced and
//         //  reinitializing with the modified size of the file.
//         //
//         //  NOTE: Even if the all the EA's are removed the Ea file will
//         //  always exist and the header area will never shrink.
//         //
//
//         FileOffset.LowPart = EaSetVbo;
//         FileOffset.HighPart = 0;
//
//         //
//         //  Round the cache map down to a system page boundary.
//         //
//
//         FileOffset.LowPart &= ~(PAGE_SIZE - 1);
//
//         //
//         //  Make sure all the data gets out to the disk.
//         //
//
//         {
//             IO_STATUS_BLOCK Iosb;
//             ULONG PurgeCount = 5;
//
//             while (--PurgeCount) {
//
//                 Iosb.Status = STATUS_SUCCESS;
//
//                 CcFlushCache( VirtualEaFile->SectionObjectPointer,
//                               NULL,
//                               0,
//                               &Iosb );
//
//                 NT_ASSERT( Iosb.Status == STATUS_SUCCESS );
//
//                 //
//                 //  We do not have to worry about a lazy writer firing in parallel
//                 //  with our CcFlushCache since we have the EaFcb exclusive.  Thus
//                 //  we know all data is out.
//                 //
//
//                 //
//                 //  We throw the unwanted pages out of the cache and then
//                 //  truncate the Ea File for the new size.
//                 //
//
//                 if (CcPurgeCacheSection( VirtualEaFile->SectionObjectPointer,
//                                          &FileOffset,
//                                          0,
//                                          FALSE )) {
//
//                     break;
//                 }
//             }
//
//             if (!PurgeCount) {
//
//                 FatRaiseStatus( IrpContext, STATUS_UNABLE_TO_DELETE_SECTION );
//             }
//         }
//
//         FileOffset.LowPart = EaFcb->Header.FileSize.LowPart - EaSetLength;
//
//         //
//         //  Perform the splice operation on the FAT chain.  This is done
//         //  by splitting the target clusters out and merging the remaining
//         //  clusters around them.  We can ignore the return value from
//         //  the merge and splice functions because we are guaranteed
//         //  to be able to block.
//         //
//
//         {
//             FsRtlInitializeLargeMcb( &DataMcb, PagedPool );
//
//             UnwindInitializeDataMcb = TRUE;
//
//             FatSplitAllocation( IrpContext,
//                                 Vcb,
//                                 &EaFcb->Mcb,
//                                 EaSetVbo,
//                                 &DataMcb );
//
//             UnwindSplitData = TRUE;
//
//             if (EaSetLength + EaSetVbo != EaFcb->Header.FileSize.LowPart) {
//
//                 FsRtlInitializeLargeMcb( &TailMcb, PagedPool );
//
//                 UnwindInitializeTailMcb = TRUE;
//
//                 FatSplitAllocation( IrpContext,
//                                     Vcb,
//                                     &DataMcb,
//                                     EaSetLength,
//                                     &TailMcb );
//
//                 UnwindSplitTail = TRUE;
//
//                 FatMergeAllocation( IrpContext,
//                                     Vcb,
//                                     &EaFcb->Mcb,
//                                     &TailMcb );
//
//                 UnwindMergeTail = TRUE;
//             }
//         }
//
//         //
//         //  Update the Fcb for the Ea file
//         //
//
//         UnwindPrevFileSize = EaFcb->Header.FileSize.LowPart;
//
//         (VOID)ExAcquireResourceExclusiveLite( EaFcb->Header.PagingIoResource,
//                                           TRUE );
//
//         EaFcb->Header.FileSize.LowPart = EaFcb->Header.FileSize.LowPart - EaSetLength;
//         EaFcb->Header.AllocationSize = EaFcb->Header.FileSize;
//
//
//         CcSetFileSizes( VirtualEaFile,
//                         (PCC_FILE_SIZES)&EaFcb->Header.AllocationSize );
//
//         ExReleaseResourceLite( EaFcb->Header.PagingIoResource );
//
//         UnwindCacheValues = TRUE;
//
//         EaDirent->FileSize = EaFcb->Header.FileSize.LowPart;
//
//         FatSetDirtyBcb( IrpContext, EaBcb, Vcb, TRUE );
//
//         //
//         //  Update the Ea base and offset tables.  For the Ea base table,
//         //  all subsequent index values must be decremented by the number
//         //  of clusters removed.
//         //
//         //  For the entries in the relevant Ea offset table, all entries
//         //  after this index must also be decreased by the number of
//         //  clusters removed.
//         //
//
//         //
//         //  Pin down the EA file header.
//         //
//
//         RtlZeroMemory( &EaHeaderRange,
//                        sizeof( EA_RANGE ));
//
//         FatPinEaRange( IrpContext,
//                        VirtualEaFile,
//                        EaFcb,
//                        &EaHeaderRange,
//                        0,
//                        sizeof( EA_FILE_HEADER ),
//                        STATUS_NONEXISTENT_EA_ENTRY );
//
//         EaHeader = (PEA_FILE_HEADER) EaHeaderRange.Data;
//
//         //
//         //  Pin down the Ea offset table for the particular index.
//         //
//
//         RtlZeroMemory( &EaOffsetRange,
//                        sizeof( EA_RANGE ));
//
//         FatPinEaRange( IrpContext,
//                        VirtualEaFile,
//                        EaFcb,
//                        &EaOffsetRange,
//                        EaOffsetVbo,
//                        sizeof( EA_OFF_TABLE ),
//                        STATUS_NONEXISTENT_EA_ENTRY );
//
//         {
//             ULONG Count;
//             PUSHORT NextEaIndex;
//
//             Count = MAX_EA_BASE_INDEX - EaHeaderBaseIndex - 1;
//
//             NextEaIndex = &EaHeader->EaBaseTable[EaHeaderBaseIndex + 1];
//
//             while (Count--) {
//
//                 *(NextEaIndex++) -= EaSetClusterCount;
//             }
//
//             FatMarkEaRangeDirty( IrpContext, VirtualEaFile, &EaHeaderRange );
//
//             Count = MAX_EA_OFFSET_INDEX - EaOffsetIndex - 1;
//             NextEaIndex = (PUSHORT) EaOffsetRange.Data + EaOffsetIndex;
//
//             *(NextEaIndex++) = UNUSED_EA_HANDLE;
//
//             while (Count--) {
//
//                 if (*NextEaIndex != UNUSED_EA_HANDLE) {
//
//                     *NextEaIndex -= EaSetClusterCount;
//                 }
//
//                 NextEaIndex++;
//             }
//
//             FatMarkEaRangeDirty( IrpContext, VirtualEaFile, &EaOffsetRange );
//         }
//
//         UnwindModifiedEaHeader = TRUE;
//
//         //
//         //  Deallocate the ea set removed
//         //
//
//         FatDeallocateDiskSpace( IrpContext,
//                                 Vcb,
//                                 &DataMcb,
//                                 FALSE );
//
//     } _SEH2_FINALLY {
//
//         DebugUnwind( FatDeleteEaSet );
//
//         //
//         //  Restore file if abnormal termination.
//         //
//         //  If we have modified the ea file header we ignore this
//         //  error.  Otherwise we walk through the state variables.
//         //
//
//         if (_SEH2_AbnormalTermination()
//             && !UnwindModifiedEaHeader) {
//
//             //
//             //  If we modified the Ea dirent or Fcb, recover the previous
//             //  values.
//             //
//
//             if (UnwindPrevFileSize) {
//
//                 EaFcb->Header.FileSize.LowPart = UnwindPrevFileSize;
//                 EaFcb->Header.AllocationSize.LowPart = UnwindPrevFileSize;
//                 EaDirent->FileSize = UnwindPrevFileSize;
//
//                 if (UnwindCacheValues) {
//
//                     CcSetFileSizes( VirtualEaFile,
//                                     (PCC_FILE_SIZES)&EaFcb->Header.AllocationSize );
//                 }
//             }
//
//             //
//             //  If we merged the tail with the
//             //  ea file header.  We split it out
//             //  again.
//             //
//
//             if (UnwindMergeTail) {
//
//                 FatSplitAllocation( IrpContext,
//                                     Vcb,
//                                     &EaFcb->Mcb,
//                                     EaSetVbo,
//                                     &TailMcb );
//             }
//
//             //
//             //  If we split the tail off we merge the tail back
//             //  with the ea data to remove.
//             //
//
//             if (UnwindSplitTail) {
//
//                 FatMergeAllocation( IrpContext,
//                                     Vcb,
//                                     &DataMcb,
//                                     &TailMcb );
//             }
//
//             //
//             //  If the ea set has been split out, we merge that
//             //  cluster string back in the file.  Otherwise we
//             //  simply uninitialize the local Mcb.
//             //
//
//             if (UnwindSplitData) {
//
//                 FatMergeAllocation( IrpContext,
//                                     Vcb,
//                                     &EaFcb->Mcb,
//                                     &DataMcb );
//             }
//         }
//
//         //
//         //  Unpin any Bcb's still active.
//         //
//
//         FatUnpinEaRange( IrpContext, &EaHeaderRange );
//         FatUnpinEaRange( IrpContext, &EaOffsetRange );
//         FatUnpinEaRange( IrpContext, &EaSetRange );
//
//         //
//         //  Uninitialize any initialized Mcbs
//         //
//
//         if (UnwindInitializeDataMcb) {
//
//             FsRtlUninitializeLargeMcb( &DataMcb );
//         }
//
//         if (UnwindInitializeTailMcb) {
//
//             FsRtlUninitializeLargeMcb( &TailMcb );
//         }
//
//         DebugTrace(-1, Dbg, "FatDeleteEaSet -> Exit\n", 0);
//     } _SEH2_END;
//
//     return;
// }
//
//
//
// _Requires_lock_held_(_Global_critical_region_)
// VOID
// FatAddEaSet (
//     IN PIRP_CONTEXT IrpContext,
//     IN PVCB Vcb,
//     IN ULONG EaSetLength,
//     IN PBCB EaBcb,
//     OUT PDIRENT EaDirent,
//     OUT PUSHORT EaHandle,
//     OUT PEA_RANGE EaSetRange
//     )
//
// /*++
//
// Routine Description:
//
//     This routine will add the necessary clusters to support a new
//     Ea set of the given size.  This is done by splicing a chain of
//     clusters into the existing Ea file.  An Ea index is assigned to
//     this new chain and the Ea base and offset tables are updated to
//     include this new handle.  This routine also pins the added
//     clusters and returns their address and a Bcb.
//
//     The caller of this function will have verified that the Ea file
//     exists and that the Vcb field points to an initialized cache file.
//     The caller will already have gained exclusive access to the
//     EaFcb.
//
// Arguments:
//
//     Vcb - Supplies the Vcb to fill in.
//
//     EaSetLength - The number of bytes needed to contain the Ea set.  This
//         routine will round this up the next cluster size.
//
//     EaBcb - Supplies a pointer to the Bcb for the Ea dirent.
//
//     EaDirent - Supplies a pointer to the dirent for the Ea file.
//
//     EaHandle - Supplies the address to store the ea index generated here.
//
//     EaSetRange - This is the structure that describes new range in the Ea file.
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     ULONG BytesPerCluster = 1 << Vcb->AllocationSupport.LogOfBytesPerCluster;
//
//     EA_RANGE EaHeaderRange;
//     USHORT EaHeaderIndex;
//     PEA_FILE_HEADER EaHeader;
//
//     EA_RANGE EaOffsetRange;
//     ULONG EaNewOffsetVbo = 0;
//     USHORT EaOffsetIndex;
//     ULONG EaOffsetTableSize;
//     PUSHORT EaOffsetTable;
//
//     ULONG EaSetClusterOffset;
//     ULONG EaSetVbo = 0;
//     USHORT EaSetClusterCount;
//     PEA_SET_HEADER EaSet;
//
//     PFILE_OBJECT VirtualEaFile = Vcb->VirtualEaFile;
//     PFCB EaFcb = Vcb->EaFcb;
//
//     LARGE_MCB EaSetMcb;
//     BOOLEAN UnwindInitializedEaSetMcb = FALSE;
//     BOOLEAN UnwindAllocatedNewAllocation = FALSE;
//     BOOLEAN UnwindMergedNewEaSet = FALSE;
//
//     LARGE_MCB EaOffsetMcb;
//     BOOLEAN UnwindInitializedOffsetMcb = FALSE;
//     BOOLEAN UnwindSplitNewAllocation = FALSE;
//     BOOLEAN UnwindMergedNewOffset = FALSE;
//
//     LARGE_MCB EaTailMcb;
//     BOOLEAN UnwindInitializedTailMcb = FALSE;
//     BOOLEAN UnwindSplitTail = FALSE;
//     BOOLEAN UnwindMergedTail = FALSE;
//
//     LARGE_MCB EaInitialEaMcb;
//     BOOLEAN UnwindInitializedInitialEaMcb = FALSE;
//     BOOLEAN UnwindSplitInitialEa = FALSE;
//     BOOLEAN UnwindMergedInitialEa = FALSE;
//
//     USHORT NewEaIndex;
//     PUSHORT NextEaOffset;
//
//     ULONG NewAllocation;
//     LARGE_INTEGER FileOffset;
//     ULONG Count;
//
//     ULONG UnwindPrevFileSize = 0;
//     BOOLEAN UnwindCacheValues = FALSE;
//
//     BOOLEAN TailExists = FALSE;
//     BOOLEAN AddedOffsetTableCluster = FALSE;
//     BOOLEAN UnwindPurgeCacheMap = FALSE;
//
//     PAGED_CODE();
//
//     DebugTrace(+1, Dbg, "FatAddEaSet\n", 0);
//     DebugTrace( 0, Dbg, "  Vcb         = %p\n", Vcb);
//     DebugTrace( 0, Dbg, "  EaSetLength = %ul\n", EaSetLength );
//
//     //
//     //  Zero the Ea range structures.
//     //
//
//     RtlZeroMemory( &EaHeaderRange, sizeof( EA_RANGE ));
//     RtlZeroMemory( &EaOffsetRange, sizeof( EA_RANGE ));
//
//     //
//     //  Use a try statement to facilitate cleanup.
//     //
//
//     _SEH2_TRY {
//
//         //
//         //  Pin down the file header.
//         //
//
//         FatPinEaRange( IrpContext,
//                        VirtualEaFile,
//                        EaFcb,
//                        &EaHeaderRange,
//                        0,
//                        sizeof( EA_FILE_HEADER ),
//                        STATUS_DATA_ERROR );
//
//         EaHeader = (PEA_FILE_HEADER) EaHeaderRange.Data;
//
//         //
//         //  Compute the size of the offset table.
//         //
//
//         EaNewOffsetVbo = EaHeader->EaBaseTable[0] << Vcb->AllocationSupport.LogOfBytesPerCluster;
//         EaOffsetTableSize = EaNewOffsetVbo - sizeof( EA_FILE_HEADER );
//
//         //
//         //  Pin down the entire offset table.
//         //
//
//         FatPinEaRange( IrpContext,
//                        VirtualEaFile,
//                        EaFcb,
//                        &EaOffsetRange,
//                        sizeof( EA_FILE_HEADER ),
//                        EaOffsetTableSize,
//                        STATUS_DATA_ERROR );
//
//         //
//         //  We now look for a valid handle out of the existing offset table.
//         //  We start at the last entry and walk backwards.  We stop at the
//         //  first unused handle which is preceded by a used handle (or handle
//         //  1).
//         //
//         //  As we walk backwards, we need to remember the file offset of the
//         //  cluster which will follow the clusters we add.  We initially
//         //  remember the end of the file.  If the end of the offset table
//         //  consists of a string of used handles, we remember the offset of
//         //  the handle prior to the transition from used to unused handles.
//         //
//
//         EaSetClusterOffset = EaFcb->Header.FileSize.LowPart
//                              >> Vcb->AllocationSupport.LogOfBytesPerCluster;
//
//         NewEaIndex = (USHORT) ((EaOffsetTableSize >> 1) - 1);
//
//         NextEaOffset = (PUSHORT) EaOffsetRange.Data + NewEaIndex;
//
//         //
//         //  Walk through the used handles at the end of the offset table.
//         //
//
//         if (*NextEaOffset != UNUSED_EA_HANDLE) {
//
//             while (NewEaIndex != 0) {
//
//                 if (*(NextEaOffset - 1) == UNUSED_EA_HANDLE) {
//
//                     //
//                     //  If the handle is 1, we take no action.  Otherwise
//                     //  we save the cluster offset of the current handle
//                     //  knowing we will use a previous handle and insert
//                     //  a chain of clusters.
//                     //
//
//                     if (NewEaIndex != 1) {
//
//                         EaSetClusterOffset = *NextEaOffset
//                                              + EaHeader->EaBaseTable[NewEaIndex >> 7];
//
//                         TailExists = TRUE;
//                     }
//
//                     NewEaIndex--;
//                     NextEaOffset--;
//
//                     break;
//                 }
//
//                 NewEaIndex--;
//                 NextEaOffset--;
//             }
//         }
//
//         //
//         //  Walk through looking for the first unused handle in a string
//         //  of unused handles.
//         //
//
//         while (NewEaIndex) {
//
//             if (*(NextEaOffset - 1) != UNUSED_EA_HANDLE) {
//
//                 break;
//             }
//
//             NextEaOffset--;
//             NewEaIndex--;
//         }
//
//         //
//         //  If the handle is zero, we do a special test to see if handle 1
//         //  is available.  Otherwise we will use the first handle of a new
//         //  cluster.  A non-zero handle now indicates that a handle was found
//         //  in an existing offset table cluster.
//         //
//
//         if (NewEaIndex == 0) {
//
//             if (*(NextEaOffset + 1) == UNUSED_EA_HANDLE) {
//
//                 NewEaIndex = 1;
//
//             } else {
//
//                 NewEaIndex = (USHORT) EaOffsetTableSize >> 1;
//                 AddedOffsetTableCluster = TRUE;
//             }
//         }
//
//         //
//         //  If the Ea index is outside the legal range then raise an
//         //  exception.
//         //
//
//         if (NewEaIndex > MAX_EA_HANDLE) {
//
//             DebugTrace(-1, Dbg,
//                        "FatAddEaSet: Illegal handle value for new handle\n", 0);
//
//             FatRaiseStatus( IrpContext, STATUS_INSUFFICIENT_RESOURCES );
//         }
//
//         //
//         //  Compute the base and offset indexes.
//         //
//
//         EaHeaderIndex = NewEaIndex >> 7;
//         EaOffsetIndex = NewEaIndex & (MAX_EA_OFFSET_INDEX - 1);
//
//         //
//         //  Compute the byte offset of the new ea data in the file.
//         //
//
//         EaSetVbo = EaSetClusterOffset << Vcb->AllocationSupport.LogOfBytesPerCluster;
//
//         //
//         //  Allocate all the required disk space together to insure this
//         //  operation is atomic.  We don't want to allocate one block
//         //  of disk space and then fail on a second allocation.
//         //
//
//         EaSetLength = (EaSetLength + BytesPerCluster - 1)
//                       & ~(BytesPerCluster - 1);
//
//         NewAllocation = EaSetLength
//                         + (AddedOffsetTableCluster ? BytesPerCluster : 0);
//
//         //
//         //  Verify that adding these clusters will not grow the Ea file
//         //  beyond its legal value.  The maximum number of clusters is
//         //  2^16 since the Ea sets are referenced by a 16 bit cluster
//         //  offset value.
//         //
//
//         if ((ULONG) ((0x0000FFFF << Vcb->AllocationSupport.LogOfBytesPerCluster)
//                      - EaFcb->Header.FileSize.LowPart)
//             < NewAllocation) {
//
//             DebugTrace(-1, Dbg,
//                        "FatAddEaSet: New Ea file size is too large\n", 0);
//
//             FatRaiseStatus( IrpContext, STATUS_INSUFFICIENT_RESOURCES );
//         }
//
//         FsRtlInitializeLargeMcb( &EaSetMcb, PagedPool );
//
//         UnwindInitializedEaSetMcb = TRUE;
//
//         FatAllocateDiskSpace( IrpContext,
//                               Vcb,
//                               0,
//                               &NewAllocation,
//                               FALSE,
//                               &EaSetMcb );
//
//         UnwindAllocatedNewAllocation = TRUE;
//
//         EaSetClusterCount = (USHORT) (EaSetLength >> Vcb->AllocationSupport.LogOfBytesPerCluster);
//
//         if (AddedOffsetTableCluster) {
//
//             FsRtlInitializeLargeMcb( &EaOffsetMcb, PagedPool );
//
//             UnwindInitializedOffsetMcb = TRUE;
//
//             FatSplitAllocation( IrpContext,
//                                 Vcb,
//                                 &EaSetMcb,
//                                 EaSetLength,
//                                 &EaOffsetMcb );
//
//             UnwindSplitNewAllocation = TRUE;
//         }
//
//         FatUnpinEaRange( IrpContext, &EaHeaderRange );
//         FatUnpinEaRange( IrpContext, &EaOffsetRange );
//
//         if (AddedOffsetTableCluster) {
//
//             FileOffset.LowPart = EaNewOffsetVbo;
//
//         } else {
//
//             FileOffset.LowPart = EaSetVbo;
//         }
//
//         FileOffset.HighPart = 0;
//
//         //
//         //  Round the cache map down to a system page boundary.
//         //
//
//         FileOffset.LowPart &= ~(PAGE_SIZE - 1);
//
//         {
//             IO_STATUS_BLOCK Iosb;
//             ULONG PurgeCount = 5;
//
//             while (--PurgeCount) {
//
//                 Iosb.Status = STATUS_SUCCESS;
//
//                 CcFlushCache( VirtualEaFile->SectionObjectPointer,
//                               NULL,
//                               0,
//                               &Iosb );
//
//                 NT_ASSERT( Iosb.Status == STATUS_SUCCESS );
//
//                 //
//                 //  We do not have to worry about a lazy writer firing in parallel
//                 //  with our CcFlushCache since we have the EaFcb exclusive.  Thus
//                 //  we know all data is out.
//                 //
//
//                 //
//                 //  We throw the unwanted pages out of the cache and then
//                 //  truncate the Ea File for the new size.
//                 //
//                 //
//
//                 if (CcPurgeCacheSection( VirtualEaFile->SectionObjectPointer,
//                                          &FileOffset,
//                                          0,
//                                          FALSE )) {
//
//                     break;
//                 }
//             }
//
//             if (!PurgeCount) {
//
//                 FatRaiseStatus( IrpContext, STATUS_UNABLE_TO_DELETE_SECTION );
//             }
//         }
//
//         UnwindPurgeCacheMap = TRUE;
//
//         FileOffset.LowPart = EaFcb->Header.FileSize.LowPart + NewAllocation;
//
//         //
//         //  If there is a tail to the file, then we initialize an Mcb
//         //  for the file section and split the tail from the file.
//         //
//
//         if (TailExists) {
//
//             FsRtlInitializeLargeMcb( &EaTailMcb, PagedPool );
//
//             UnwindInitializedTailMcb = TRUE;
//
//             FatSplitAllocation( IrpContext,
//                                 Vcb,
//                                 &EaFcb->Mcb,
//                                 EaSetVbo,
//                                 &EaTailMcb );
//
//             UnwindSplitTail = TRUE;
//         }
//
//         //
//         //  If there is an initial section of ea data, we initialize an
//         //  Mcb for that section.
//         //
//
//         if (AddedOffsetTableCluster
//             && EaSetVbo != EaNewOffsetVbo) {
//
//             FsRtlInitializeLargeMcb( &EaInitialEaMcb, PagedPool );
//
//             UnwindInitializedInitialEaMcb = TRUE;
//
//             FatSplitAllocation( IrpContext,
//                                 Vcb,
//                                 &EaFcb->Mcb,
//                                 EaNewOffsetVbo,
//                                 &EaInitialEaMcb );
//
//             UnwindSplitInitialEa = TRUE;
//         }
//
//         //
//         //  We have now split the new file allocation into the new
//         //  ea set and possibly a new offset table.
//         //
//         //  We have also split the existing file data into a file
//         //  header, an initial section of ea data and the tail of the
//         //  file.  These last 2 may not exist.
//         //
//         //  Each section is described by an Mcb.
//         //
//
//         //
//         //  Merge the new offset information if it exists.
//         //
//
//         if (AddedOffsetTableCluster) {
//
//             FatMergeAllocation( IrpContext,
//                                 Vcb,
//                                 &EaFcb->Mcb,
//                                 &EaOffsetMcb );
//
//             FsRtlUninitializeLargeMcb( &EaOffsetMcb );
//             FsRtlInitializeLargeMcb( &EaOffsetMcb, PagedPool );
//
//             UnwindMergedNewOffset = TRUE;
//         }
//
//         //
//         //  Merge the existing initial ea data if it exists.
//         //
//
//         if (UnwindInitializedInitialEaMcb) {
//
//             FatMergeAllocation( IrpContext,
//                                 Vcb,
//                                 &EaFcb->Mcb,
//                                 &EaInitialEaMcb );
//
//             FsRtlUninitializeLargeMcb( &EaInitialEaMcb );
//             FsRtlInitializeLargeMcb( &EaInitialEaMcb, PagedPool );
//
//             UnwindMergedInitialEa = TRUE;
//         }
//
//         //
//         //  We modify the offset of the new ea set by one cluster if
//         //  we added one to the offset table.
//         //
//
//         if (AddedOffsetTableCluster) {
//
//             EaSetClusterOffset += 1;
//             EaSetVbo += BytesPerCluster;
//         }
//
//         //
//         //  Merge the new ea set.
//         //
//
//         FatMergeAllocation( IrpContext,
//                             Vcb,
//                             &EaFcb->Mcb,
//                             &EaSetMcb );
//
//         FsRtlUninitializeLargeMcb( &EaSetMcb );
//         FsRtlInitializeLargeMcb( &EaSetMcb, PagedPool );
//
//         UnwindMergedNewEaSet = TRUE;
//
//         //
//         //  Merge the tail if it exists.
//         //
//
//         if (UnwindInitializedTailMcb) {
//
//             FatMergeAllocation( IrpContext,
//                                 Vcb,
//                                 &EaFcb->Mcb,
//                                 &EaTailMcb );
//
//             FsRtlUninitializeLargeMcb( &EaTailMcb );
//             FsRtlInitializeLargeMcb( &EaTailMcb, PagedPool );
//
//             UnwindMergedTail = TRUE;
//         }
//
//         //
//         //  If we added a new cluster for the offset table, we need to
//         //  lock the entire cluster down and initialize all the handles to
//         //  the unused state except the first one.
//         //
//
//         //
//         //  Update the Fcb information.
//         //
//
//         UnwindPrevFileSize = EaFcb->Header.FileSize.LowPart;
//
//         EaFcb->Header.FileSize.LowPart += NewAllocation;
//         EaFcb->Header.AllocationSize = EaFcb->Header.FileSize;
//         EaDirent->FileSize = EaFcb->Header.FileSize.LowPart;
//
//         FatSetDirtyBcb( IrpContext, EaBcb, Vcb, TRUE );
//
//         //
//         //  Let Mm and Cc know the new file sizes.
//         //
//
//         CcSetFileSizes( VirtualEaFile,
//                         (PCC_FILE_SIZES)&EaFcb->Header.AllocationSize );
//
//         UnwindCacheValues = TRUE;
//
//         //
//         //  Pin down the file header.
//         //
//
//         RtlZeroMemory( &EaHeaderRange, sizeof( EA_RANGE ));
//
//         FatPinEaRange( IrpContext,
//                        VirtualEaFile,
//                        EaFcb,
//                        &EaHeaderRange,
//                        0,
//                        sizeof( EA_FILE_HEADER ),
//                        STATUS_DATA_ERROR );
//
//         EaHeader = (PEA_FILE_HEADER) EaHeaderRange.Data;
//
//         //
//         //  Pin down the entire offset table.
//         //
//
//
//         RtlZeroMemory( &EaOffsetRange, sizeof( EA_RANGE ));
//
//         FatPinEaRange( IrpContext,
//                        VirtualEaFile,
//                        EaFcb,
//                        &EaOffsetRange,
//                        sizeof( EA_FILE_HEADER ) + (((ULONGLONG)NewEaIndex >> 7) << 8),
//                        sizeof( EA_OFF_TABLE ),
//                        STATUS_DATA_ERROR );
//
//         EaOffsetTable = (PUSHORT) EaOffsetRange.Data;
//
//         //
//         //  Pin the Ea set header for the added clusters and initialize
//         //  the fields of interest.  These are the signature field, the
//         //  owning handle field, the need Ea field and the cbList field.
//         //  Also mark the data as dirty.
//         //
//
//         //
//         //  Pin the ea set.
//         //
//
//         FatPinEaRange( IrpContext,
//                        VirtualEaFile,
//                        EaFcb,
//                        EaSetRange,
//                        EaSetVbo,
//                        EaSetLength,
//                        STATUS_DATA_ERROR );
//
//         EaSet = (PEA_SET_HEADER) EaSetRange->Data;
//
//         EaSet->Signature = EA_SET_SIGNATURE;
//         EaSet->OwnEaHandle = NewEaIndex;
//
//         FatMarkEaRangeDirty( IrpContext, VirtualEaFile, EaSetRange );
//
//         //
//         //  Update the Ea base and offset tables.  For the Ea base table,
//         //  all subsequent index values must be incremented by the number
//         //  of clusters added.
//         //
//         //  For the entries in the relevant Ea offset table, all entries
//         //  after this index must also be increased by the number of
//         //  clusters added.
//         //
//         //  If we added another cluster to the offset table, then we increment
//         //  all the base table values by 1.
//         //
//
//         Count = MAX_EA_BASE_INDEX - EaHeaderIndex - 1;
//
//         NextEaOffset = &EaHeader->EaBaseTable[EaHeaderIndex + 1];
//
//         while (Count--) {
//
//             *(NextEaOffset++) += EaSetClusterCount;
//         }
//
//         if (AddedOffsetTableCluster) {
//
//             Count = MAX_EA_BASE_INDEX;
//
//             NextEaOffset = &EaHeader->EaBaseTable[0];
//
//             while (Count--) {
//
//                 *(NextEaOffset++) += 1;
//             }
//         }
//
//         FatMarkEaRangeDirty( IrpContext, VirtualEaFile, &EaHeaderRange );
//
//         //
//         //  If we added an offset table cluster, we need to initialize
//         //  the handles to unused.
//         //
//
//         if (AddedOffsetTableCluster) {
//
//             Count = (BytesPerCluster >> 1) - 1;
//             NextEaOffset = EaOffsetTable;
//
//             *NextEaOffset++ = 0;
//
//             while (Count--) {
//
//                 *NextEaOffset++ = UNUSED_EA_HANDLE;
//             }
//         }
//
//         //
//         //  We need to compute the offset of the added Ea set clusters
//         //  from their base.
//         //
//
//         NextEaOffset = EaOffsetTable + EaOffsetIndex;
//
//         *NextEaOffset++ = (USHORT) (EaSetClusterOffset
//                                     - EaHeader->EaBaseTable[EaHeaderIndex]);
//
//         Count = MAX_EA_OFFSET_INDEX - EaOffsetIndex - 1;
//
//         while (Count--) {
//
//             if (*NextEaOffset != UNUSED_EA_HANDLE) {
//
//                 *NextEaOffset += EaSetClusterCount;
//             }
//
//             NextEaOffset++;
//         }
//
//         FatMarkEaRangeDirty( IrpContext, VirtualEaFile, &EaOffsetRange );
//
//         //
//         //  Update the callers parameters.
//         //
//
//         *EaHandle = NewEaIndex;
//
//         DebugTrace(0, Dbg, "FatAddEaSet: Return values\n", 0);
//
//         DebugTrace(0, Dbg, "FatAddEaSet: New Handle -> %x\n",
//                    *EaHandle);
//
//     } _SEH2_FINALLY {
//
//         DebugUnwind( FatAddEaSet );
//
//         //
//         //  Handle cleanup for abnormal termination only if we allocated
//         //  disk space for the new ea set.
//         //
//
//         if (_SEH2_AbnormalTermination() && UnwindAllocatedNewAllocation) {
//
//             //
//             //  If we modified the Ea dirent or Fcb, recover the previous
//             //  values.  Even though we are decreasing FileSize here, we
//             //  don't need to synchronize to synchronize with paging Io
//             //  because there was no dirty data generated in the new allocation.
//             //
//
//             if (UnwindPrevFileSize) {
//
//                 EaFcb->Header.FileSize.LowPart = UnwindPrevFileSize;
//                 EaFcb->Header.AllocationSize.LowPart = UnwindPrevFileSize;
//                 EaDirent->FileSize = UnwindPrevFileSize;
//
//                 if (UnwindCacheValues) {
//
//                     CcSetFileSizes( VirtualEaFile,
//                                     (PCC_FILE_SIZES)&EaFcb->Header.AllocationSize );
//                 }
//             }
//
//             //
//             //  If we merged the tail then split it off.
//             //
//
//             if (UnwindMergedTail) {
//
//                 VBO NewTailPosition;
//
//                 NewTailPosition = EaSetVbo + EaSetLength;
//
//                 FatSplitAllocation( IrpContext,
//                                     Vcb,
//                                     &EaFcb->Mcb,
//                                     NewTailPosition,
//                                     &EaTailMcb );
//             }
//
//             //
//             //  If we merged the new ea data then split it out.
//             //
//
//             if (UnwindMergedNewEaSet) {
//
//                 FatSplitAllocation( IrpContext,
//                                     Vcb,
//                                     &EaFcb->Mcb,
//                                     EaSetVbo,
//                                     &EaSetMcb );
//             }
//
//             //
//             //  If we merged the initial ea data then split it out.
//             //
//
//             if (UnwindMergedInitialEa) {
//
//                 FatSplitAllocation( IrpContext,
//                                     Vcb,
//                                     &EaFcb->Mcb,
//                                     EaNewOffsetVbo + BytesPerCluster,
//                                     &EaInitialEaMcb );
//             }
//
//             //
//             //  If we added a new offset cluster, then split it out.
//             //
//
//             if (UnwindMergedNewOffset) {
//
//                 FatSplitAllocation( IrpContext,
//                                     Vcb,
//                                     &EaFcb->Mcb,
//                                     EaNewOffsetVbo,
//                                     &EaOffsetMcb );
//             }
//
//             //
//             //  If there is an initial ea section prior to the new section, merge
//             //  it with the rest of the file.
//             //
//
//             if (UnwindSplitInitialEa) {
//
//                 FatMergeAllocation( IrpContext, Vcb, &EaFcb->Mcb, &EaInitialEaMcb );
//             }
//
//             //
//             //  If there is a file tail split off, merge it with the
//             //  rest of the file.
//             //
//
//             if (UnwindSplitTail) {
//
//                 FatMergeAllocation( IrpContext, Vcb, &EaFcb->Mcb, &EaTailMcb );
//             }
//
//             //
//             //  If we modified the cache initialization for the ea file,
//             //  then throw away the ea file object.
//             //
//
//             if (UnwindPurgeCacheMap) {
//
//                 Vcb->VirtualEaFile = NULL;
//                 ObDereferenceObject( VirtualEaFile );
//             }
//
//             //
//             //  If we split the allocation, then deallocate the block for
//             //  the new offset information.
//             //
//
//             if (UnwindSplitNewAllocation) {
//
//                 FatDeallocateDiskSpace( IrpContext, Vcb, &EaOffsetMcb, FALSE );
//             }
//
//             //
//             //  Deallocate the disk space.
//             //
//
//             FatDeallocateDiskSpace( IrpContext, Vcb, &EaSetMcb, FALSE );
//         }
//
//         //
//         //  Unpin the Ea ranges.
//         //
//
//         FatUnpinEaRange( IrpContext, &EaHeaderRange );
//         FatUnpinEaRange( IrpContext, &EaOffsetRange );
//
//         //
//         //  Uninitialize any local Mcbs
//         //
//
//         if (UnwindInitializedEaSetMcb) {
//
//             FsRtlUninitializeLargeMcb( &EaSetMcb );
//         }
//
//         if (UnwindInitializedOffsetMcb) {
//
//             FsRtlUninitializeLargeMcb( &EaOffsetMcb );
//         }
//
//         if (UnwindInitializedTailMcb) {
//
//             FsRtlUninitializeLargeMcb( &EaTailMcb );
//         }
//
//         if (UnwindInitializedInitialEaMcb) {
//
//             FsRtlUninitializeLargeMcb( &EaInitialEaMcb );
//         }
//
//         DebugTrace(-1, Dbg, "FatAddEaSet ->  Exit\n", 0);
//     } _SEH2_END;
//
//     return;
// }
//
//
// VOID
// FatAppendPackedEa (
//     IN PIRP_CONTEXT IrpContext,
//     IN OUT PEA_SET_HEADER *EaSetHeader,
//     IN OUT PULONG PackedEasLength,
//     IN OUT PULONG AllocationLength,
//     IN PFILE_FULL_EA_INFORMATION FullEa,
//     IN ULONG BytesPerCluster
//     )
//
// /*++
//
// Routine Description:
//
//     This routine appends a new packed ea onto an existing packed ea list,
//     it also will allocate/dealloate pool as necessary to hold the ea list.
//
// Arguments:
//
//     EaSetHeader - Supplies the address to store the pointer to pool memory
//                   which contains the Ea list for a file.
//
//     PackedEasLength - Supplies the length of the actual Ea data.  The
//                       new Ea data will be appended at this point.
//
//     AllocationLength - Supplies the allocated length available for Ea
//                        data.
//
//     FullEa - Supplies a pointer to the new full ea that is to be appended
//              (in packed form) to the packed ea list.
//
//     BytesPerCluster - Number of bytes per cluster on this volume.
//
//     NOTE: The EaSetHeader refers to the entire block of Ea data for a
//           file.  This includes the Ea's and their values as well as the
//           header information.  The PackedEasLength and AllocationLength
//           parameters refer to the name/value pairs only.
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     ULONG PackedEaSize;
//     PPACKED_EA ThisPackedEa;
//     OEM_STRING EaName;
//
//     PAGED_CODE();
//
//     DebugTrace(+1, Dbg, "FatAppendPackedEa...\n", 0);
//
//     //
//     //  As a quick check see if the computed packed ea size plus the
//     //  current packed ea list size will overflow the buffer.  Full Ea and
//     //  packed Ea only differ by 4 in their size
//     //
//
//     PackedEaSize = SizeOfFullEa( FullEa ) - 4;
//
//     if ( PackedEaSize + *PackedEasLength > *AllocationLength ) {
//
//         //
//         //  We will overflow our current work buffer so allocate a larger
//         //  one and copy over the current buffer
//         //
//
//         PVOID Temp;
//         ULONG NewAllocationSize;
//         ULONG OldAllocationSize;
//
//         DebugTrace(0, Dbg, "Allocate a new ea list buffer\n", 0);
//
//         //
//         //  Compute a new size and allocate space.  Always increase the
//         //  allocation in cluster increments.
//         //
//
//         NewAllocationSize = (SIZE_OF_EA_SET_HEADER
//                              + PackedEaSize
//                              + *PackedEasLength
//                              + BytesPerCluster - 1)
//                             & ~(BytesPerCluster - 1);
//
//         Temp = FsRtlAllocatePoolWithTag( PagedPool,
//                                          NewAllocationSize,
//                                          TAG_EA_SET_HEADER );
//
//         //
//         //  Move over the existing ea list, and deallocate the old one
//         //
//
//         RtlCopyMemory( Temp,
//                        *EaSetHeader,
//                        OldAllocationSize = *AllocationLength
//                                            + SIZE_OF_EA_SET_HEADER );
//
//         ExFreePool( *EaSetHeader );
//
//         //
//         //  Set up so we will use the new packed ea list
//         //
//
//         *EaSetHeader = Temp;
//
//         //
//         //  Zero out the added memory.
//         //
//
//         RtlZeroMemory( &(*EaSetHeader)->PackedEas[*AllocationLength],
//                        NewAllocationSize - OldAllocationSize );
//
//         *AllocationLength = NewAllocationSize - SIZE_OF_EA_SET_HEADER;
//     }
//
//     //
//     //  Determine if we need to increment our need ea changes count
//     //
//
//     if ( FlagOn(FullEa->Flags, FILE_NEED_EA )) {
//
//         //
//         //  The NeedEaCount field is long aligned so we will write
//         //  directly to it.
//         //
//
//         (*EaSetHeader)->NeedEaCount++;
//     }
//
//     //
//     //  Now copy over the ea, full ea's and packed ea are identical except
//     //  that full ea also have a next ea offset that we skip over
//     //
//     //  Before:
//     //             UsedSize                     Allocated
//     //                |                             |
//     //                V                             V
//     //      +xxxxxxxx+-----------------------------+
//     //
//     //  After:
//     //                              UsedSize    Allocated
//     //                                 |            |
//     //                                 V            V
//     //      +xxxxxxxx+yyyyyyyyyyyyyyyy+------------+
//     //
//
//     ThisPackedEa = (PPACKED_EA) (RtlOffsetToPointer( (*EaSetHeader)->PackedEas,
//                                                      *PackedEasLength ));
//
//     RtlCopyMemory( ThisPackedEa,
//                    (PUCHAR) FullEa + 4,
//                    PackedEaSize );
//
//     //
//     //  Now convert the name to uppercase.
//     //
//
//     EaName.MaximumLength = EaName.Length = FullEa->EaNameLength;
//     EaName.Buffer = ThisPackedEa->EaName;
//
//     FatUpcaseEaName( IrpContext, &EaName, &EaName );
//
//     //
//     //  Increment the used size in the packed ea list structure
//     //
//
//     *PackedEasLength += PackedEaSize;
//
//     //
//     //  And return to our caller
//     //
//
//     DebugTrace(-1, Dbg, "FatAppendPackedEa -> VOID\n", 0);
//
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     return;
// }
//
//
// VOID
// FatDeletePackedEa (
//     IN PIRP_CONTEXT IrpContext,
//     IN OUT PEA_SET_HEADER EaSetHeader,
//     IN OUT PULONG PackedEasLength,
//     IN ULONG Offset
//     )
//
// /*++
//
// Routine Description:
//
//     This routine deletes an individual packed ea from the supplied
//     packed ea list.
//
// Arguments:
//
//     EaSetHeader - Supplies the address to store the pointer to pool memory
//                   which contains the Ea list for a file.
//
//     PackedEasLength - Supplies the length of the actual Ea data.  The
//                       new Ea data will be appended at this point.
//
//     Offset - Supplies the offset to the individual ea in the list to delete
//
//     NOTE: The EaSetHeader refers to the entire block of Ea data for a
//           file.  This includes the Ea's and their values as well as the
//           header information.  The PackedEasLength parameter refer to the
//           name/value pairs only.
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     PPACKED_EA PackedEa;
//     ULONG PackedEaSize;
//
//     PAGED_CODE();
//
//     DebugTrace(+1, Dbg, "FatDeletePackedEa, Offset = %08lx\n", Offset);
//
//     //
//     //  Get a reference to the packed ea and figure out its size
//     //
//
//     PackedEa = (PPACKED_EA) (&EaSetHeader->PackedEas[Offset]);
//
//     SizeOfPackedEa( PackedEa, &PackedEaSize );
//
//     //
//     //  Determine if we need to decrement our need ea changes count
//     //
//
//     if (FlagOn(PackedEa->Flags, EA_NEED_EA_FLAG)) {
//
//         EaSetHeader->NeedEaCount--;
//     }
//
//     //
//     //  Shrink the ea list over the deleted ea.  The amount to copy is the
//     //  total size of the ea list minus the offset to the end of the ea
//     //  we're deleting.
//     //
//     //  Before:
//     //              Offset    Offset+PackedEaSize      UsedSize    Allocated
//     //                |                |                  |            |
//     //                V                V                  V            V
//     //      +xxxxxxxx+yyyyyyyyyyyyyyyy+zzzzzzzzzzzzzzzzzz+------------+
//     //
//     //  After
//     //              Offset            UsedSize                     Allocated
//     //                |                  |                             |
//     //                V                  V                             V
//     //      +xxxxxxxx+zzzzzzzzzzzzzzzzzz+-----------------------------+
//     //
//
//     RtlCopyMemory( PackedEa,
//                    (PUCHAR) PackedEa + PackedEaSize,
//                    *PackedEasLength - (Offset + PackedEaSize) );
//
//     //
//     //  And zero out the remaing part of the ea list, to make things
//     //  nice and more robust
//     //
//
//     RtlZeroMemory( &EaSetHeader->PackedEas[*PackedEasLength - PackedEaSize],
//                    PackedEaSize );
//
//     //
//     //  Decrement the used size by the amount we just removed
//     //
//
//     *PackedEasLength -= PackedEaSize;
//
//     //
//     //  And return to our caller
//     //
//
//     DebugTrace(-1, Dbg, "FatDeletePackedEa -> VOID\n", 0);
//
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     return;
// }
//
//
// ULONG
// FatLocateNextEa (
//     IN PIRP_CONTEXT IrpContext,
//     IN PPACKED_EA FirstPackedEa,
//     IN ULONG PackedEasLength,
//     IN ULONG PreviousOffset
//     )
//
// /*++
//
// Routine Description:
//
//     This routine locates the offset for the next individual packed ea
//     inside of a packed ea list, given the offset to a previous Ea.
//     Instead of returing boolean to indicate if we've found the next one
//     we let the return offset be so large that it overuns the used size
//     of the packed ea list, and that way it's an easy construct to use
//     in a for loop.
//
// Arguments:
//
//     FirstPackedEa - Supplies a pointer to the packed ea list structure
//
//     PackedEasLength - Supplies the length of the packed ea list
//
//     PreviousOffset - Supplies the offset to a individual packed ea in the
//         list
//
// Return Value:
//
//     ULONG - The offset to the next ea in the list or 0xffffffff of one
//         does not exist.
//
// --*/
//
// {
//     PPACKED_EA PackedEa;
//     ULONG PackedEaSize;
//     ULONG Offset;
//
//     PAGED_CODE();
//
//     DebugTrace(+1, Dbg, "FatLocateNextEa, PreviousOffset = %08lx\n",
//                PreviousOffset);
//
//     //
//     //  Make sure the previous offset is within the used size range
//     //
//
//     if ( PreviousOffset >= PackedEasLength ) {
//
//         DebugTrace(-1, Dbg, "FatLocateNextEa -> 0xffffffff\n", 0);
//         return 0xffffffff;
//     }
//
//     //
//     //  Get a reference to the previous packed ea, and compute its size
//     //
//
//     PackedEa = (PPACKED_EA) ((PUCHAR) FirstPackedEa + PreviousOffset );
//     SizeOfPackedEa( PackedEa, &PackedEaSize );
//
//     //
//     //  Compute to the next ea
//     //
//
//     Offset = PreviousOffset + PackedEaSize;
//
//     //
//     //  Now, if the new offset is beyond the ea size then we know
//     //  that there isn't one so, we return an offset of 0xffffffff.
//     //  otherwise we'll leave the new offset alone.
//     //
//
//     if ( Offset >= PackedEasLength ) {
//
//         Offset = 0xffffffff;
//     }
//
//     DebugTrace(-1, Dbg, "FatLocateNextEa -> %08lx\n", Offset);
//
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     return Offset;
// }
//
//
// BOOLEAN
// FatLocateEaByName (
//     IN PIRP_CONTEXT IrpContext,
//     IN PPACKED_EA FirstPackedEa,
//     IN ULONG PackedEasLength,
//     IN POEM_STRING EaName,
//     OUT PULONG Offset
//     )
//
// /*++
//
// Routine Description:
//
//     This routine locates the offset for the next individual packed ea
//     inside of a packed ea list, given the name of the ea to locate
//
// Arguments:
//
//     FirstPackedEa - Supplies a pointer to the packed ea list structure
//
//     PackedEasLength - Supplies the length of the packed ea list
//
//     EaName - Supplies the name of the ea search for
//
//     Offset - Receives the offset to the located individual ea in the list
//         if one exists.
//
// Return Value:
//
//     BOOLEAN - TRUE if the named packed ea exists in the list and FALSE
//         otherwise.
//
// --*/
//
// {
//     PPACKED_EA PackedEa;
//     OEM_STRING Name;
//
//     PAGED_CODE();
//
//     DebugTrace(+1, Dbg, "FatLocateEaByName, EaName = %Z\n", EaName);
//
//     //
//     //  For each packed ea in the list check its name against the
//     //  ea name we're searching for
//     //
//
//     for ( *Offset = 0;
//           *Offset < PackedEasLength;
//           *Offset = FatLocateNextEa( IrpContext,
//                                      FirstPackedEa,
//                                      PackedEasLength,
//                                      *Offset )) {
//
//         //
//         //  Reference the packed ea and get a string to its name
//         //
//
//         PackedEa = (PPACKED_EA) ((PUCHAR) FirstPackedEa + *Offset);
//
//         Name.Buffer = &PackedEa->EaName[0];
//         Name.Length = PackedEa->EaNameLength;
//         Name.MaximumLength = PackedEa->EaNameLength;
//
//         //
//         //  Compare the two strings, if they are equal then we've
//         //  found the caller's ea
//         //
//
//         if ( RtlCompareString( EaName, &Name, TRUE ) == 0 ) {
//
//             DebugTrace(-1, Dbg, "FatLocateEaByName -> TRUE, *Offset = %08lx\n", *Offset);
//             return TRUE;
//         }
//     }
//
//     //
//     //  We've exhausted the ea list without finding a match so return false
//     //
//
//     DebugTrace(-1, Dbg, "FatLocateEaByName -> FALSE\n", 0);
//     return FALSE;
// }
//
//
// BOOLEAN
// FatIsEaNameValid (
//     IN PIRP_CONTEXT IrpContext,
//     IN OEM_STRING Name
//     )
//
// /*++
//
// Routine Description:
//
//     This routine simple returns whether the specified file names conforms
//     to the file system specific rules for legal Ea names.
//
//     For Ea names, the following rules apply:
//
//     A. An Ea name may not contain any of the following characters:
//
//        0x0000 - 0x001F  \ / : * ? " < > | , + = [ ] ;
//
// Arguments:
//
//     Name - Supllies the name to check.
//
// Return Value:
//
//     BOOLEAN - TRUE if the name is legal, FALSE otherwise.
//
// --*/
//
// {
//     ULONG Index;
//
//     UCHAR Char;
//
//     PAGED_CODE();
//
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     //
//     //  Empty names are not valid.
//     //
//
//     if ( Name.Length == 0 ) { return FALSE; }
//
//     //
//     //  At this point we should only have a single name, which can't have
//     //  more than 254 characters
//     //
//
//     if ( Name.Length > 254 ) { return FALSE; }
//
//     for ( Index = 0; Index < (ULONG)Name.Length; Index += 1 ) {
//
//         Char = Name.Buffer[ Index ];
//
//         //
//         //  Skip over and Dbcs chacters
//         //
//
//         if ( FsRtlIsLeadDbcsCharacter( Char ) ) {
//
//             NT_ASSERT( Index != (ULONG)(Name.Length - 1) );
//
//             Index += 1;
//
//             continue;
//         }
//
//         //
//         //  Make sure this character is legal, and if a wild card, that
//         //  wild cards are permissible.
//         //
//
//         if ( !FsRtlIsAnsiCharacterLegalFat(Char, FALSE) ) {
//
//             return FALSE;
//         }
//     }
//
//     return TRUE;
// }
//
//
// VOID
// FatPinEaRange (
//     IN PIRP_CONTEXT IrpContext,
//     IN PFILE_OBJECT VirtualEaFile,
//     IN PFCB EaFcb,
//     IN OUT PEA_RANGE EaRange,
//     IN ULONG StartingVbo,
//     IN ULONG Length,
//     IN NTSTATUS ErrorStatus
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is called to pin a range within the Ea file.  It will follow all the
//     rules required by the cache manager so that we don't have overlapping pin operations.
//     If the range being pinned spans a section then the desired data will be copied into
//     an auxilary buffer.  FatMarkEaRangeDirty will know whether to copy the data back
//     into the cache or whether to simply mark the pinned data dirty.
//
// Arguments:
//
//     VirtualEaFile - This is the stream file for the Ea file.
//
//     EaFcb - This is the Fcb for the Ea file.
//
//     EaRange - This is the Ea range structure for this request.
//
//     StartingVbo - This is the starting offset in the Ea file to read from.
//
//     Length - This is the length of the read.
//
//     ErrorStatus - This is the error status to use if we are reading outside
//         of the file.
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     LARGE_INTEGER LargeVbo;
//     ULONG ByteCount;
//     PBCB *NextBcb;
//     PVOID Buffer;
//     PCHAR DestinationBuffer = NULL;
//     BOOLEAN FirstPage = TRUE;
//
//     PAGED_CODE();
//
//     //
//     //  Verify that the entire read is contained within the Ea file.
//     //
//
//     if (Length == 0
//         || StartingVbo >= EaFcb->Header.AllocationSize.LowPart
//         || (EaFcb->Header.AllocationSize.LowPart - StartingVbo) < Length) {
//
//         FatRaiseStatus( IrpContext, ErrorStatus );
//     }
//
//     //
//     //  If the read will span a section, the system addresses may not be contiguous.
//     //  Allocate a separate buffer in this case.
//     //
//
//     if (((StartingVbo & (EA_SECTION_SIZE - 1)) + Length) > EA_SECTION_SIZE) {
//
//         EaRange->Data = FsRtlAllocatePoolWithTag( PagedPool,
//                                                   Length,
//                                                   TAG_EA_DATA );
//         EaRange->AuxilaryBuffer = TRUE;
//
//         DestinationBuffer = EaRange->Data;
//
//     } else {
//
//         //
//         //  PREfix correctly notes that if we don't decide here to have an aux buffer
//         //  and the flag is up in the EaRange, we'll party on random memory since
//         //  DestinationBuffer won't be set; however, this will never happen due to
//         //  initialization of ea ranges and the cleanup in UnpinEaRange.
//         //
//
//         NT_ASSERT( EaRange->AuxilaryBuffer == FALSE );
//     }
//
//
//     //
//     //  If the read will require more pages than our structure will hold then
//     //  allocate an auxilary buffer.  We have to figure the number of pages
//     //  being requested so we have to include the page offset of the first page of
//     //  the request.
//     //
//
//     EaRange->BcbChainLength = (USHORT) (((StartingVbo & (PAGE_SIZE - 1)) + Length + PAGE_SIZE - 1) / PAGE_SIZE);
//
//     if (EaRange->BcbChainLength > EA_BCB_ARRAY_SIZE) {
//
//         EaRange->BcbChain = FsRtlAllocatePoolWithTag( PagedPool,
//                                                       sizeof( PBCB ) * EaRange->BcbChainLength,
//                                                       TAG_BCB );
//
//         RtlZeroMemory( EaRange->BcbChain, sizeof( PBCB ) * EaRange->BcbChainLength );
//
//     } else {
//
//         EaRange->BcbChain = (PBCB *) &EaRange->BcbArray;
//     }
//
//     //
//     //  Store the byte range data in the Ea Range structure.
//     //
//
//     EaRange->StartingVbo = StartingVbo;
//     EaRange->Length = Length;
//
//     //
//     //  Compute the initial pin length.
//     //
//
//     ByteCount = PAGE_SIZE - (StartingVbo & (PAGE_SIZE - 1));
//
//     //
//     //  For each page in the range; pin the page and update the Bcb count, copy to
//     //  the auxiliary buffer.
//     //
//
//     NextBcb = EaRange->BcbChain;
//
//     while (Length != 0) {
//
//         //
//         //  Pin the page and remember the data start.
//         //
//
//         LargeVbo.QuadPart = StartingVbo;
//
//         if (ByteCount > Length) {
//
//             ByteCount = Length;
//         }
//
//         if (!CcPinRead( VirtualEaFile,
//                         &LargeVbo,
//                         ByteCount,
//                         BooleanFlagOn(IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT),
//                         NextBcb,
//                         &Buffer )) {
//
//             //
//             // Could not read the data without waiting (cache miss).
//             //
//
//             FatRaiseStatus( IrpContext, STATUS_CANT_WAIT );
//         }
//
//         //
//         //  Increment the Bcb pointer and copy to the auxilary buffer if necessary.
//         //
//
//         NextBcb += 1;
//
//         if (EaRange->AuxilaryBuffer == TRUE) {
//
//             RtlCopyMemory( DestinationBuffer,
//                            Buffer,
//                            ByteCount );
//
//             DestinationBuffer = (PCHAR) Add2Ptr( DestinationBuffer, ByteCount );
//         }
//
//         StartingVbo += ByteCount;
//         Length -= ByteCount;
//
//         //
//         //  If this is the first page then update the Ea Range structure.
//         //
//
//         if (FirstPage) {
//
//             FirstPage = FALSE;
//             ByteCount = PAGE_SIZE;
//
//             if (EaRange->AuxilaryBuffer == FALSE) {
//
//                 EaRange->Data = Buffer;
//             }
//         }
//     }
//
//     return;
// }
//
//
// VOID
// FatMarkEaRangeDirty (
//     IN PIRP_CONTEXT IrpContext,
//     IN PFILE_OBJECT EaFileObject,
//     IN OUT PEA_RANGE EaRange
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is called to mark a range of the Ea file as dirty.  If the modified
//     data is sitting in an auxilary buffer then we will copy it back into the cache.
//     In any case we will go through the list of Bcb's and mark them dirty.
//
// Arguments:
//
//     EaFileObject - This is the file object for the Ea file.
//
//     EaRange - This is the Ea range structure for this request.
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     PBCB *NextBcb;
//     ULONG BcbCount;
//
//     PAGED_CODE();
//
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     //
//     //  If there is an auxilary buffer we need to copy the data back into the cache.
//     //
//
//     if (EaRange->AuxilaryBuffer == TRUE) {
//
//         LARGE_INTEGER LargeVbo;
//
//         LargeVbo.QuadPart = EaRange->StartingVbo;
//
//         CcCopyWrite( EaFileObject,
//                      &LargeVbo,
//                      EaRange->Length,
//                      TRUE,
//                      EaRange->Data );
//     }
//
//     //
//     //  Now walk through the Bcb chain and mark everything dirty.
//     //
//
//     BcbCount = EaRange->BcbChainLength;
//     NextBcb = EaRange->BcbChain;
//
//     while (BcbCount--) {
//
//         if (*NextBcb != NULL) {
//
//             CcSetDirtyPinnedData( *NextBcb, NULL );
//         }
//
//         NextBcb += 1;
//     }
//
//     return;
// }
//
//
// VOID
// FatUnpinEaRange (
//     IN PIRP_CONTEXT IrpContext,
//     IN OUT PEA_RANGE EaRange
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is called to unpin a range in the Ea file.  Any structures allocated
//     will be deallocated here.
//
// Arguments:
//
//     EaRange - This is the Ea range structure for this request.
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     PBCB *NextBcb;
//     ULONG BcbCount;
//
//     PAGED_CODE();
//
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     //
//     //  If we allocated a auxilary buffer, deallocate it here.
//     //
//
//     if (EaRange->AuxilaryBuffer == TRUE) {
//
//         ExFreePool( EaRange->Data );
//         EaRange->AuxilaryBuffer = FALSE;
//     }
//
//     //
//     //  Walk through the Bcb chain and unpin the data.
//     //
//
//     if (EaRange->BcbChain != NULL) {
//
//         BcbCount = EaRange->BcbChainLength;
//         NextBcb = EaRange->BcbChain;
//
//         while (BcbCount--) {
//
//             if (*NextBcb != NULL) {
//
//                 CcUnpinData( *NextBcb );
//                 *NextBcb = NULL;
//             }
//
//             NextBcb += 1;
//         }
//
//         //
//         //  If we allocated a Bcb chain, deallocate it here.
//         //
//
//         if (EaRange->BcbChain != &EaRange->BcbArray[0]) {
//
//             ExFreePool( EaRange->BcbChain );
//         }
//
//         EaRange->BcbChain = NULL;
//     }
//
//     return;
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
