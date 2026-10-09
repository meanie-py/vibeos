use ntkrnl_sys::globals::*;
// The real definitions, so the compiler checks every call against them.
use crate::timesup::{FatGetCurrentFatTime, FatNtTimeToFatTime};
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{ExAcquireResourceExclusiveLite, ExFreePool, ExIsResourceAcquiredExclusiveLite, ExIsResourceAcquiredSharedLite, ExReleaseResourceLite, LARGE_INTEGER, LARGE_MCB, OEM_STRING, PFILE_OBJECT, PLARGE_INTEGER, PLONGLONG, POEM_STRING, PUNICODE_STRING, RtlAreBitsClear, RtlAreBitsSet, RtlDowncaseUnicodeString, RtlFindClearBits, RtlOemStringToCountedUnicodeString, RtlSetBits, RtlUpcaseUnicodeString};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{CCB, DCB, DELETE_CONTEXT, DIRENT, FAT8DOT3, FAT_BUG_CHECK_DIRSUP, FAT_DATA, FAT_DIRENT_NT_BYTE_3_LOWER_CASE, FAT_DIRENT_NT_BYTE_8_LOWER_CASE, FCB, FCB_STATE_3_LOWER_CASE, FCB_STATE_8_LOWER_CASE, IRP_CONTEXT, LFN_DIRENT, PFCB, VCB};
use ntkrnl_sys::ddk::{FILE_OBJECT, IRP, OPLOCK, _PUBLIC_BCB as BCB, _UNICODE_STRING};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{FAT_NTC_DCB, STATUS_CANNOT_MAKE};
// resolve_imports.py: explicit kernel/private-type imports
use crate::ddk_shims::{ARGUMENT_PRESENT, FsRtlAddLargeMcbEntry, FsRtlAddToTunnelCache, FsRtlAllocatePoolWithTag, FsRtlAreNamesEqual, FsRtlDeleteKeyFromTunnelCache, FsRtlGetNextLargeMcbEntry, FsRtlInitializeLargeMcb, FsRtlIsNameInExpression, FsRtlNumberOfRunsInLargeMcb, FsRtlUninitializeLargeMcb};

extern "system" {
    fn FatDeleteEa( IrpContext: *mut IRP_CONTEXT, Vcb: *mut core::ffi::c_void, ExtendedAttributes: u16, OemName: *const OEM_STRING, );
    fn FatBugCheck(a: u32, b: u32, c: u32);
    fn FatUnpinBcb(IrpContext: *mut IRP_CONTEXT, Bcb: *mut *mut core::ffi::c_void);
    fn FatPrepareWriteDirectoryFile(IrpContext: *mut IRP_CONTEXT, ParentDcb: *mut DCB, Offset: u32, Length: u32, Bcb: *mut *mut BCB, Dirent: *mut *mut DIRENT, LowPriority: u8, ReturnMapped: u8, Status: *mut NTSTATUS);
    fn FatIsFat32(Vcb: *mut core::ffi::c_void) -> u8;
    fn NodeType(obj: *mut core::ffi::c_void) -> u32;
    } extern "C" { fn DebugTrace(level: i32, flag: u32, fmt: *const i8, ...); } extern "system" {
    fn FatStringTo8dot3( IrpContext: PIRP_CONTEXT, FileName: OEM_STRING, OutputString: *mut OEM_STRING, );
    static Dbg: u32;
    static FatData: FAT_DATA;
    static FatOneDay: LARGE_INTEGER;
}
extern "system" {
    fn NT_ASSERT(condition: i32);
    fn FatConstructDot( IrpContext: *mut IRP_CONTEXT, dcb: *mut DCB, parent_dirent: *mut DIRENT, dirent: *mut DIRENT, );
    fn FatConstructDotDot( IrpContext: *mut IRP_CONTEXT, dcb: *mut DCB, parent_dirent: *mut DIRENT, dirent: *mut DIRENT, );
    fn FlagOn(state: u32, mask: u32) -> u32;
    fn BooleanFlagOn(flags: u32, flag: u32) -> u8;
    fn FatDirectoryKey(dcb: *mut core::ffi::c_void) -> u64;
    fn FatReadDirent( irp_context: *mut IRP_CONTEXT, dcb: *mut DCB, vbo: u32, bcb: *mut *mut BCB, dirent: *mut *mut DIRENT, status: *mut i32, );
    fn FatPinMappedData( irp_context: *mut IRP_CONTEXT, dcb: *mut DCB, byte_offset: u32, length: u32, bcb: *mut *mut BCB, );
    fn FatRaiseStatus(IrpContext: *mut IRP_CONTEXT, Status: i32) -> !;
    fn FatReadDirectoryFile( IrpContext: *mut IRP_CONTEXT, ParentDcb: *mut DCB, Offset: u32, Length: u32, Wait: i32, Bcb: *mut *mut BCB, Dirent: *mut *mut DIRENT, Status: *mut i32 ) -> ();
    fn FatVcbAcquiredExclusive(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB) -> BOOLEAN;
    fn FatUnpinRepinnedBcbs(IrpContext: *mut IRP_CONTEXT);
    fn FatMarkFcbCondition(IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB, Condition: u32, Value: u32);
    fn IsFileDeleted(IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB) -> BOOLEAN;
    fn FAT_LFN_DIRENTS_NEEDED(Lfn: *mut _UNICODE_STRING) -> u32;
    fn FatSetDirtyBcb(IrpContext: *mut IRP_CONTEXT, Bcb: *mut BCB, Vcb: *mut VCB, MarkDirty: u8);
    fn ExSystemTimeToLocalTime(SystemTime: *const LARGE_INTEGER, LocalTime: *mut LARGE_INTEGER);
    fn ExLocalTimeToSystemTime(LocalTime: *const LARGE_INTEGER, SystemTime: *mut LARGE_INTEGER);
    fn FatGetFcbOplock(Fcb: *mut FCB) -> *mut OPLOCK;
    fn FatNotifyReportChange( IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB, FcbOrDcb: *mut FCB, NotifyFilter: u32, Action: u32, );
    fn FatResetExceptionState(IrpContext: *mut IRP_CONTEXT);
    fn FsRtlCheckOplockEx( Oplock: *mut OPLOCK, Irp: *mut IRP, Flags: u32, Context: *mut core::ffi::c_void, CompletionRoutine: *mut core::ffi::c_void, PostIrpRoutine: *mut core::ffi::c_void, ) -> u32;
    fn FsRtlIsNtstatusExpected(NtStatus: i32) -> u8;
    fn KeQuerySystemTime(CurrentTime: *mut LARGE_INTEGER);
    fn TimerStart(dbg: u32) -> ();
    fn TimerStop(dbg: u32, name: *const i8) -> ();
    fn FatPopUpFileCorrupt(IrpContext: *mut IRP_CONTEXT, ParentDirectory: *mut DCB) -> ();
    fn FatFreeStringBuffer(Str: *mut _UNICODE_STRING) -> ();
    fn FatEnsureStringBufferEnough(Str: *mut _UNICODE_STRING, Length: u16) -> ();
    fn Fat8dot3ToString(IrpContext: *mut IRP_CONTEXT, Dirent: *mut DIRENT, uppercase: u32, Name: *mut OEM_STRING);
    fn FatIsNameInExpression( IrpContext: *mut IRP_CONTEXT, Expression: OEM_STRING, Name: OEM_STRING, ) -> BOOLEAN;
    fn FatNormalizeAndRaiseStatus(IrpContext: *mut IRP_CONTEXT, Status: NTSTATUS) -> ();
    fn SetFlag(Flags: *mut u32, Flag: u32) -> ();
}

// ---- file-scope preamble (translated) ----
pub const BugCheckFileId: u32 = FAT_BUG_CHECK_DIRSUP;

const FAT_NTC_ROOT_DCB: u32 = 0x0504;

const CCB_FLAG_SKIP_SHORT_NAME_COMPARE: u32 = 0x02;
const CCB_FLAG_QUERY_TEMPLATE_MIXED: u32 = 0x1000;


const FAT_DIRENT_NEVER_USED: u8 = 0x00;
const FAT_DIRENT_DELETED: u8 = 0xE5;
const FAT_DIRENT_ATTR_ARCHIVE: u8 = 0x20;
const FAT_DIRENT_ATTR_VOLUME_ID: u8 = 0x08;

const FcbNeedsToBeVerified: u32 = 3; // placeholder, correct value needed
const VCB_STATE_FLAG_REMOVABLE_MEDIA: u32 = 0x02; // placeholder, correct value needed
const STATUS_FILE_CORRUPT_ERROR: i32 = 0xC0000102_u32 as i32; // placeholder, correct value needed
const DEBUG_TRACE_DIRSUP: u32 = 0x00400000; // placeholder, correct value needed


// Forward declarations for unknown extern functions

// Constants (placeholders for unknown values)
const CCB_FLAG_MATCH_ALL: u32 = 0x00000001;
const CCB_FLAG_MATCH_VOLUME_ID: u32 = 0x4000;
const TAG_FILENAME_BUFFER: u32 = 0x6E746146; // 'Flfr' placeholder
const TAG_DIRENT: u32 = 0x44746146; // 'Dirn' placeholder
const FcbBad: u32 = 2;
const IRP_CONTEXT_FLAG_WAIT: u32 = 0x02;  // nolint: const-contradiction (fastfat's own flag set, fatstruc.h:1563)
const IRP_CONTEXT_FLAG_WRITE_THROUGH: u32 = 0x04;

// These constants are typical Windows/NT values; exact values may differ.
const FcbGood: u32 = 1;
const FAT_NTC_FCB: u32 = 0x0502;
const FO_FILE_MODIFIED: u32 = 0x1000;
const FO_FILE_SIZE_CHANGED: u32 = 0x2000;
const FO_FILE_FAST_IO_READ: u32 = 0x00080000;
const CCB_FLAG_USER_SET_LAST_WRITE: u32 = 0x10;
const CCB_FLAG_USER_SET_LAST_ACCESS: u32 = 0x20;
const VCB_STATE_FLAG_WRITE_PROTECTED: u32 = 0x4000;
const FILE_ATTRIBUTE_ARCHIVE: u32 = 0x20;
const FILE_NOTIFY_CHANGE_ATTRIBUTES: u32 = 0x04;
const FILE_NOTIFY_CHANGE_LAST_WRITE: u32 = 0x10;
const FILE_NOTIFY_CHANGE_LAST_ACCESS: u32 = 0x20;
const FILE_NOTIFY_CHANGE_SIZE: u32 = 8;
const FILE_ACTION_MODIFIED: u32 = 0x03;
const OPLOCK_FLAG_PARENT_OBJECT: u32 = 0x10; // ntifs: OPLOCK_FLAG_PARENT_OBJECT 0x10, as create.rs; was 2 (OPLOCK_KEY_CHECK_ONLY)


// NOTE: FatData is the real global FAT_DATA (imported from crate::types).
// ChicagoMode is a packed bitfield in that struct — it MUST be read via the
// generated accessor FatData.ChicagoMode(), NOT as a plain offset-0 field. A
// prior hand-rolled `FatGlobalData { ChicagoMode: u8 }` stub put it at offset
// 0, which actually read the low byte of NodeTypeCode (0x0500 -> 0x00 == FALSE)
// and silently disabled ALL long-file-name matching (only 8.3 names resolved).

// Forward declarations for unknown external functions

// Constants used by the function
const FAT_DIRENT_ATTR_LFN: u8 = 0x0F;
const FAT_LAST_LONG_ENTRY: u8 = 0x40;
const MAX_LFN_DIRENTS: u8 = 20;
const CCB_FLAG_OPENED_BY_SHORTNAME: u32 = 0x0800;
const STATUS_END_OF_FILE: NTSTATUS = (0xC0000011_u32 as i32) as u32; // likely value

// fat.h:38 is `typedef ULONG32 VBO` — FOUR bytes. This file used to alias it to
// LARGE_INTEGER, so FatLocateDirent stored `(*ByteOffset).QuadPart`, an EIGHT-byte
// write through a pointer the interface says points at four. create.rs:841,
// namesup.rs:272 and fileinfo.rs:1705 each pass a `u32` stack local, so every
// call smashed 4 bytes of the caller's frame — and create.rs is the file-OPEN
// path, which on arm64 runs against a FAT system volume on every open.
use crate::types::VBO;
// The real 4-byte stamp (Time, then Date). A local stand-in here once had them
// swapped plus a byte, so zeroing one cleared 2 bytes past it and an access-date
// update stored the time of day.
use crate::types::FAT_TIME_STAMP;
type BOOLEAN = u8;
type NTSTATUS = u32;
type ULONG = u32;
type USHORT = u16;
type UCHAR = u8;
type WCHAR = u16;
type CHAR = i8;
type PUCHAR = *mut u8;
type PCHAR = *mut CHAR;
type PWCH = *mut WCHAR;
type PULONG = *mut u32;
type PUSHORT = *mut u16;
type PVOID = *mut u8;
type PLFN_DIRENT = *mut LFN_DIRENT;
type PBCB = *mut *mut BCB;
type PCCB = *const CCB;
type PDCB = *mut DCB;
type PIRP_CONTEXT = *mut IRP_CONTEXT;
type PDIRENT = *mut DIRENT;
type PCUNICODE_STRING = *const _UNICODE_STRING;
type PURCH = *mut WCHAR;
type PCWCH = *const WCHAR;
type PBOOLEAN = *mut BOOLEAN;
#[no_mangle]
pub unsafe extern "system" fn FatCreateNewDirent(
    IrpContext: PIRP_CONTEXT,
    ParentDirectory: PDCB,
    DirentsNeeded: u32,
    RescanDir: i32, // BOOLEAN
) -> u32 {
    // Forward declarations for called functions not in the import list

    // Local variables
    let mut UnusedVbo: u32;
    let mut DeletedHint: u32;
    let mut ByteOffset: u32;
    let mut Bcb: *mut BCB = core::ptr::null_mut();
    let mut Dirent: PDIRENT = core::ptr::null_mut();
    let mut Status: NTSTATUS = 0; // STATUS_SUCCESS

    // PAGED_CODE();  // macro, ignored

    DebugTrace(1, Dbg as u32 , b"FatCreateNewDirent\n\0".as_ptr() as *const i8, 0);
    DebugTrace(0, Dbg as u32 , b"  ParentDirectory = %p\n\0".as_ptr() as *const i8, ParentDirectory);

    UnusedVbo = (*ParentDirectory).Specific.Dcb.as_ref().UnusedDirentVbo;
    DeletedHint = (*ParentDirectory).Specific.Dcb.as_ref().DeletedDirentHint;

    // Check for first call or forced rescan
    if UnusedVbo == 0xffffffff_u32 || RescanDir != 0 {
        FatRescanDirectory(IrpContext, ParentDirectory);
        UnusedVbo = (*ParentDirectory).Specific.Dcb.as_ref().UnusedDirentVbo;
        DeletedHint = (*ParentDirectory).Specific.Dcb.as_ref().DeletedDirentHint;
    }

    DebugTrace(0, Dbg as u32 , b"  UnusedVbo   = %08lx\n\0".as_ptr() as *const i8, UnusedVbo);
    DebugTrace(0, Dbg as u32 , b"  DeletedHint = %08lx\n\0".as_ptr() as *const i8, DeletedHint);

    if RescanDir == 0
        && (UnusedVbo.wrapping_add(DirentsNeeded.wrapping_mul(core::mem::size_of::<DIRENT>() as u32))
            <= (*ParentDirectory).Header.AllocationSize.__bindgen_anon_1.LowPart)
    {
        // Easy case: there is a never-used entry
        DebugTrace(0, Dbg as u32 , b"There is a never used entry.\n\0".as_ptr() as *const i8, 0);
        ByteOffset = UnusedVbo;
        UnusedVbo = UnusedVbo.wrapping_add(DirentsNeeded.wrapping_mul(core::mem::size_of::<DIRENT>() as u32));
    } else {
        // Hard case: search for deleted dirents
        ByteOffset = RtlFindClearBits(
            (core::ptr::addr_of_mut!(((*ParentDirectory).Specific.Dcb.as_mut()).FreeDirentBitmap)).cast(),
            DirentsNeeded,
            DeletedHint / (core::mem::size_of::<DIRENT>() as u32),
        );

        // Defrag attempt for root directory on non-FAT32 under size limit
        if FatIsFat32(((*ParentDirectory).Vcb).cast()) == 0
            && ByteOffset == 0xffffffff_u32
            && NodeType((ParentDirectory).cast()) == (FAT_NTC_ROOT_DCB as u32)
            && (*ParentDirectory).Header.AllocationSize.__bindgen_anon_1.LowPart <= 0x40000u32
        {
            ByteOffset = FatDefragDirectory(IrpContext, ParentDirectory, DirentsNeeded);
        }

        if ByteOffset != 0xffffffff_u32 {
            // Consumed deleted or unused dirents, update hints
            ByteOffset = ByteOffset.wrapping_mul(core::mem::size_of::<DIRENT>() as u32);

            if ByteOffset == DeletedHint {
                DeletedHint = DeletedHint.wrapping_add(DirentsNeeded.wrapping_mul(core::mem::size_of::<DIRENT>() as u32));
            }
            if ByteOffset.wrapping_add(DirentsNeeded.wrapping_mul(core::mem::size_of::<DIRENT>() as u32)) > UnusedVbo {
                UnusedVbo = ByteOffset.wrapping_add(DirentsNeeded.wrapping_mul(core::mem::size_of::<DIRENT>() as u32));
            }
        } else {
            // Must allocate another cluster
            DebugTrace(0, Dbg as u32 , b"We have to allocate another cluster.\n\0".as_ptr() as *const i8, 0);

            // Check size limits and read-only root
            if (*ParentDirectory).Header.AllocationSize.__bindgen_anon_1.LowPart >= (64u32 * 1024u32 * core::mem::size_of::<DIRENT>() as u32)
                || (FatIsFat32(((*ParentDirectory).Vcb).cast()) == 0
                    && NodeType((ParentDirectory).cast()) == (FAT_NTC_ROOT_DCB as u32))
            {
                DebugTrace(
                    0,
                    Dbg as u32 ,
                    b"Full root directory or too big on FAT32.  Raise Status.\n\0".as_ptr() as *const i8,
                    0,
                );
                FatRaiseStatus(IrpContext, STATUS_CANNOT_MAKE as i32);
            }

            // Take the last dirent(s) in this cluster, will allocate more below
            ByteOffset = UnusedVbo;
            UnusedVbo = UnusedVbo.wrapping_add(DirentsNeeded.wrapping_mul(core::mem::size_of::<DIRENT>() as u32));

            // Touch the directory file to allocate space for new dirents
            Bcb = core::ptr::null_mut();

            let p_Bcb = core::ptr::addr_of_mut!(Bcb);
            let p_Status = core::ptr::addr_of_mut!(Status);
            crate::seh::try_finally(
                &mut (),
                |_| {
                    let mut Buffer: *mut DIRENT = core::ptr::null_mut();
                    FatPrepareWriteDirectoryFile(
                        IrpContext,
                        ParentDirectory,
                        UnusedVbo,
                        1,
                        p_Bcb,
                        core::ptr::addr_of_mut!(Buffer),
                        (0i32) as _,  // FALSE
                        (1i32) as _,  // TRUE
                        p_Status,
                    );
                },
                |_, _| {
                    FatUnpinBcb(IrpContext, p_Bcb as *mut *mut core::ffi::c_void);
                },
            );
        }
    }

    // The three blocks below were dropped from the translation, and the last one
    // is load-bearing: without RtlSetBits every new dirent stayed FREE in the
    // in-memory bitmap, so a later create that searches for deleted dirents
    // (RtlFindClearBits, the "hard case" above) could be handed one that is in use
    // and overwrite a live file's directory entry. FatDeleteDirent's
    // AreBitsSet ASSERT is where it showed, twice per i386 boot. (dirsup.c:414-534)

    //
    //  If we are only requesting a single dirent, and we did not get the
    //  first dirent in a directory, then check that the preceding dirent
    //  is not an orphaned LFN.  If it is, then mark it deleted.
    //  Only do this when we are in Chicago Mode.
    //  (This block, the unused-dirent check and the RtlSetBits below were
    //  missing from the translation.)
    //

    Bcb = core::ptr::null_mut();
    let p_Bcb = core::ptr::addr_of_mut!(Bcb);
    let p_Dirent = core::ptr::addr_of_mut!(Dirent);
    let p_Status = core::ptr::addr_of_mut!(Status);

    if FatData.ChicagoMode() != 0
        && DirentsNeeded == 1
        && ByteOffset
            > (if NodeType((ParentDirectory).cast()) == FAT_NTC_ROOT_DCB {
                0
            } else {
                2 * core::mem::size_of::<DIRENT>() as u32
            })
    {
        crate::seh::try_finally(
            &mut (),
            |_| {
                FatReadDirent(
                    IrpContext,
                    ParentDirectory,
                    ByteOffset - core::mem::size_of::<DIRENT>() as u32,
                    p_Bcb,
                    p_Dirent,
                    p_Status as *mut i32,
                );

                if *p_Status != 0 || (**p_Dirent).FileName[0] == FAT_DIRENT_NEVER_USED {
                    FatPopUpFileCorrupt(IrpContext, ParentDirectory);
                    FatRaiseStatus(IrpContext, STATUS_FILE_CORRUPT_ERROR);
                }

                if (**p_Dirent).Attributes == FAT_DIRENT_ATTR_LFN
                    && (**p_Dirent).FileName[0] != FAT_DIRENT_DELETED
                {
                    // Pin it, mark it, and set it dirty.
                    FatPinMappedData(
                        IrpContext,
                        ParentDirectory,
                        ByteOffset - core::mem::size_of::<DIRENT>() as u32,
                        core::mem::size_of::<DIRENT>() as u32,
                        p_Bcb,
                    );

                    (**p_Dirent).FileName[0] = FAT_DIRENT_DELETED;

                    FatSetDirtyBcb(IrpContext, *p_Bcb, (*ParentDirectory).Vcb, 1);

                    NT_ASSERT((RtlAreBitsSet(
                        core::ptr::addr_of_mut!(((*ParentDirectory).Specific.Dcb.as_mut()).FreeDirentBitmap).cast(),
                        (ByteOffset - core::mem::size_of::<DIRENT>() as u32) / core::mem::size_of::<DIRENT>() as u32,
                        DirentsNeeded,
                    ) != 0) as i32);

                    RtlClearBits(
                        core::ptr::addr_of_mut!(((*ParentDirectory).Specific.Dcb.as_mut()).FreeDirentBitmap).cast(),
                        (ByteOffset - core::mem::size_of::<DIRENT>() as u32) / core::mem::size_of::<DIRENT>() as u32,
                        DirentsNeeded,
                    );
                }
            },
            |_, _| {
                FatUnpinBcb(IrpContext, p_Bcb as *mut *mut core::ffi::c_void);
            },
        );
    }

    //
    //  Assert that the dirents are in fact unused
    //

    crate::seh::try_finally(
        &mut (),
        |_| {
            *p_Bcb = core::ptr::null_mut();

            for i in 0..DirentsNeeded {
                FatReadDirent(
                    IrpContext,
                    ParentDirectory,
                    ByteOffset + i * core::mem::size_of::<DIRENT>() as u32,
                    p_Bcb,
                    p_Dirent,
                    p_Status as *mut i32,
                );

                if *p_Status != 0
                    || ((**p_Dirent).FileName[0] != FAT_DIRENT_NEVER_USED
                        && (**p_Dirent).FileName[0] != FAT_DIRENT_DELETED)
                {
                    FatPopUpFileCorrupt(IrpContext, ParentDirectory);
                    FatRaiseStatus(IrpContext, STATUS_FILE_CORRUPT_ERROR);
                }
            }
        },
        |_, _| {
            FatUnpinBcb(IrpContext, p_Bcb as *mut *mut core::ffi::c_void);
        },
    );

    //
    //  Set the Bits in the bitmap and move the Unused Dirent Vbo.
    //

    RtlSetBits(
        core::ptr::addr_of_mut!(((*ParentDirectory).Specific.Dcb.as_mut()).FreeDirentBitmap).cast(),
        ByteOffset / core::mem::size_of::<DIRENT>() as u32,
        DirentsNeeded,
    );

    (*ParentDirectory).Specific.Dcb.as_mut().UnusedDirentVbo = UnusedVbo;
    (*ParentDirectory).Specific.Dcb.as_mut().DeletedDirentHint = DeletedHint;

    DebugTrace(-1, Dbg as u32 , b"FatCreateNewDirent -> (VOID)\n\0".as_ptr() as *const i8, 0);

    ByteOffset
}

#[no_mangle]
pub unsafe extern "system" fn FatInitializeDirectoryDirent(
    IrpContext: *mut IRP_CONTEXT,
    Dcb: *mut DCB,
    ParentDirent: *mut DIRENT,
) {
    let mut Bcb: *mut BCB = core::ptr::null_mut();
    let mut Buffer: *mut DIRENT = core::ptr::null_mut();
    let mut DontCare: NTSTATUS = ntkrnl_sys::base::STATUS_SUCCESS as u32;

    PAGED_CODE!();

    DebugTrace(1, (Dbg as i32) as u32, b"FatInitializeDirectoryDirent\n\0".as_ptr() as *const i8, 0i32);
    DebugTrace(0, (Dbg as i32) as u32, b"  Dcb = %p\n\0".as_ptr() as *const i8, Dcb);

    NT_ASSERT((NodeType((Dcb).cast()) != FAT_NTC_ROOT_DCB) as i32);
    NT_ASSERT(((*Dcb).Header.AllocationSize.__bindgen_anon_1.LowPart == 0) as i32);

    FatPrepareWriteDirectoryFile(
        IrpContext,
        Dcb,
        0,
        2 * core::mem::size_of::<DIRENT>() as u32,
        &mut Bcb,
        &mut Buffer,
        (0i32) as _,
        (1i32) as _,
        &mut DontCare,
    );

    NT_ASSERT(ntkrnl_sys::base::NT_SUCCESS(DontCare as i32) as i32);

    // Add the . and .. entries; the finally unpins the buffer.
    let p_Bcb = core::ptr::addr_of_mut!(Bcb);
    crate::seh::try_finally(
        &mut (),
        |_| {
            FatConstructDot(IrpContext, Dcb, ParentDirent, Buffer.add(0));
            FatConstructDotDot(IrpContext, Dcb, ParentDirent, Buffer.add(1));
        },
        |_, _| {
            FatUnpinBcb(IrpContext, p_Bcb as *mut *mut core::ffi::c_void);
        },
    );

    DebugTrace(-1, (Dbg as i32) as u32, b"FatInitializeDirectoryDirent -> (VOID)\n\0".as_ptr() as *const i8, 0i32);
    return;
}

#[no_mangle]
pub unsafe extern "system" fn FatTunnelFcbOrDcb(FcbOrDcb: *mut FCB, Ccb: *mut CCB) {
    let mut ShortNameWithCase: _UNICODE_STRING = core::mem::zeroed();
    let mut DownCaseSeg: _UNICODE_STRING = core::mem::zeroed();
    let mut ShortNameBuffer: [u16; 12] = [0u16; 12];
    let mut Status: NTSTATUS;
    let mut i: u16;

    PAGED_CODE!();

    DebugTrace(1, Dbg, b"FatTunnelFcbOrDcb\n\0".as_ptr() as *const i8, 0);

    if NodeType(FcbOrDcb as *mut core::ffi::c_void) == FAT_NTC_DCB {
        FsRtlDeleteKeyFromTunnelCache(
            core::ptr::addr_of_mut!((*(*(FcbOrDcb)).Vcb).Tunnel),
            FatDirectoryKey((FcbOrDcb as *mut DCB).cast()),
        );
    } else {
        ShortNameWithCase.Length = 0;
        ShortNameWithCase.MaximumLength = core::mem::size_of_val(&ShortNameBuffer) as u16;
        ShortNameWithCase.Buffer = ShortNameBuffer.as_mut_ptr();

        Status = (RtlOemStringToCountedUnicodeString(
            &mut ShortNameWithCase as *mut _UNICODE_STRING,
            ((*(FcbOrDcb)).ShortName.Name.Oem).as_ref() as *const OEM_STRING,
            0u8,
        )) as u32;

        NT_ASSERT((ShortNameWithCase.Length != 0) as i32);
        NT_ASSERT(NT_SUCCESS((Status) as i32) as i32);

        if FlagOn(
            (*(FcbOrDcb)).FcbState,
            FCB_STATE_8_LOWER_CASE | FCB_STATE_3_LOWER_CASE,
        ) != 0
        {
            let wchar_count = ShortNameWithCase.Length / core::mem::size_of::<u16>() as u16;
            i = 0;
            while i < wchar_count && *((ShortNameWithCase.Buffer).add(i as usize)) != 0x2Eu16 {
                i += 1;
            }

            if FlagOn((*(FcbOrDcb)).FcbState, FCB_STATE_8_LOWER_CASE) != 0 {
                DownCaseSeg.Buffer = ShortNameWithCase.Buffer;
                DownCaseSeg.Length = (i as u16) * core::mem::size_of::<u16>() as u16;
                DownCaseSeg.MaximumLength = DownCaseSeg.Length;
                RtlDowncaseUnicodeString(
                    &mut DownCaseSeg as *mut _UNICODE_STRING,
                    &mut DownCaseSeg as *mut _UNICODE_STRING,
                    0u8,
                );
            }

            i += 1;

            if FlagOn((*(FcbOrDcb)).FcbState, FCB_STATE_3_LOWER_CASE) != 0 {
                if ((i as u16) * core::mem::size_of::<u16>() as u16) < ShortNameWithCase.Length {
                    DownCaseSeg.Buffer = (ShortNameWithCase.Buffer as *mut u16).add(i as usize);
                    DownCaseSeg.Length =
                        ShortNameWithCase.Length - (i as u16) * core::mem::size_of::<u16>() as u16;
                    DownCaseSeg.MaximumLength = DownCaseSeg.Length;
                    RtlDowncaseUnicodeString(
                        &mut DownCaseSeg as *mut _UNICODE_STRING,
                        &mut DownCaseSeg as *mut _UNICODE_STRING,
                        0u8,
                    );
                }
            }
        }

        FsRtlAddToTunnelCache(
            core::ptr::addr_of_mut!((*(*(FcbOrDcb)).Vcb).Tunnel),
            FatDirectoryKey(((*(FcbOrDcb)).ParentDcb as *mut DCB).cast()),
            core::ptr::addr_of_mut!(ShortNameWithCase),
            core::ptr::addr_of_mut!((*(FcbOrDcb)).ExactCaseLongName),
            BooleanFlagOn((*(Ccb)).Flags, CCB_FLAG_OPENED_BY_SHORTNAME) as u8,
            core::mem::size_of::<LARGE_INTEGER>() as u32,
            core::ptr::addr_of!((*(FcbOrDcb)).CreationTime) as *const LARGE_INTEGER as *mut core::ffi::c_void,
        );
    }

    DebugTrace(-1, Dbg, b"FatTunnelFcbOrDcb -> (VOID)\n\0".as_ptr() as *const i8, 0);
}


const PAGE_SIZE: u32 = 4096; // fallback if not from ntkrnl_sys::base
#[no_mangle]
pub unsafe extern "system" fn FatDeleteDirent(
    IrpContext: PIRP_CONTEXT,
    FcbOrDcb: *mut FCB,
    DeleteContext: *mut DELETE_CONTEXT,
    DeleteEa: BOOLEAN,
) {
    let mut Bcb: *mut BCB = core::ptr::null_mut();
    let mut Dirent: *mut DIRENT = core::ptr::null_mut();
    let mut DontCare: NTSTATUS = ntkrnl_sys::base::STATUS_SUCCESS as u32;
    let mut Offset: u32 = 0;
    let mut DirentsToDelete: u32 = 0;

    DebugTrace(1, Dbg, b"FatDeleteDirent\n\0".as_ptr() as *const i8, 0i32);
    DebugTrace(0, Dbg, b"  FcbOrDcb = %p\n\0".as_ptr() as *const i8, FcbOrDcb as *mut core::ffi::c_void);

    ntkrnl_sys::ASSERT!(ExIsResourceAcquiredExclusiveLite(
            core::ptr::addr_of_mut!((*(*FcbOrDcb).Vcb).Resource)
        ) != 0);
    ntkrnl_sys::ASSERT!(NodeType((FcbOrDcb).cast()) != FAT_NTC_ROOT_DCB);

    if DeleteEa != 0
        && ((*FcbOrDcb).Header.AllocationSize.__bindgen_anon_1.LowPart != 0
            || (NodeType((FcbOrDcb).cast()) == FAT_NTC_FCB
                && (*FcbOrDcb).Header.FileSize.__bindgen_anon_1.LowPart != 0))
    {
        DebugTrace(0, Dbg, b"Called with non zero allocation/file size.\n\0".as_ptr() as *const i8, 0i32);
        FatBugCheck(0, 0, 0);
    }

    let p_Bcb = core::ptr::addr_of_mut!(Bcb);
    crate::seh::try_finally(
        &mut (),
        |_| {
            ExAcquireResourceExclusiveLite(
                (*(*FcbOrDcb).ParentDcb).Header.Resource,
                1u8,
            );

            Offset = (*FcbOrDcb).LfnOffsetWithinDirectory;
            while Offset <= (*FcbOrDcb).DirentOffsetWithinDirectory {
                if Offset == (*FcbOrDcb).LfnOffsetWithinDirectory
                    || (Offset & (PAGE_SIZE - 1)) == 0
                {
                    FatUnpinBcb(IrpContext, p_Bcb as *mut *mut core::ffi::c_void);
                    FatPrepareWriteDirectoryFile(
                        IrpContext,
                        (*FcbOrDcb).ParentDcb,
                        Offset,
                        core::mem::size_of::<DIRENT>() as u32,
                        p_Bcb,
                        &mut Dirent,
                        (0i32) as _,
                        (1i32) as _,
                        &mut DontCare,
                    );
                }

                ntkrnl_sys::ASSERT!((*Dirent).FirstClusterOfFile == 0 || DeleteEa == 0);
                (*Dirent).FileName[0] = FAT_DIRENT_DELETED;

                Offset += core::mem::size_of::<DIRENT>() as u32;
                Dirent = Dirent.add(1);
            }

            Dirent = Dirent.sub(1);

            if FatIsFat32(((*FcbOrDcb).Vcb).cast()) == 0
                && DeleteEa != 0
                && *(*Dirent).__bindgen_anon_1.ExtendedAttributes.as_ref() != 0
            {
                // We catch all exceptions that Fat catches, but don't do anything with them.
                let ea = *(*Dirent).__bindgen_anon_1.ExtendedAttributes.as_ref();
                let _ = crate::seh::fat_try_except(core::ptr::addr_of!(IrpContext), || {
                    FatDeleteEa(
                        IrpContext,
                        ((*FcbOrDcb).Vcb).cast(),
                        ea,
                        (*FcbOrDcb).ShortName.Name.Oem.as_ref() as *const OEM_STRING,
                    );
                });
            }

            let dcb = (*(*FcbOrDcb).ParentDcb).Specific.Dcb.as_mut();

            DirentsToDelete = ((*FcbOrDcb).DirentOffsetWithinDirectory
                - (*FcbOrDcb).LfnOffsetWithinDirectory)
                / (core::mem::size_of::<DIRENT>() as u32)
                + 1;

            ntkrnl_sys::ASSERT!(dcb.UnusedDirentVbo == 0xffffffff_u32
                    || RtlAreBitsSet(
                        core::ptr::addr_of_mut!(dcb.FreeDirentBitmap) as *mut _,
                        (*FcbOrDcb).LfnOffsetWithinDirectory / (core::mem::size_of::<DIRENT>() as u32),
                        DirentsToDelete,
                    ) != 0);

            RtlClearBits(
                core::ptr::addr_of_mut!(dcb.FreeDirentBitmap) as *mut _,
                (*FcbOrDcb).LfnOffsetWithinDirectory / (core::mem::size_of::<DIRENT>() as u32),
                DirentsToDelete,
            );

            if !DeleteContext.is_null() {
                (*Dirent).FileSize = (*DeleteContext).FileSize;
                (*Dirent).FirstClusterOfFile = (*DeleteContext).FirstClusterOfFile as u16;
            }

            if (*FcbOrDcb).DirentOffsetWithinDirectory
                < dcb.DeletedDirentHint
            {
                dcb.DeletedDirentHint =
                    (*FcbOrDcb).LfnOffsetWithinDirectory;
            }
        },
        |_, _| {
            FatUnpinBcb(IrpContext, p_Bcb as *mut *mut core::ffi::c_void);
            // Release our parent.
            ExReleaseResourceLite((*(*FcbOrDcb).ParentDcb).Header.Resource);
        },
    );

    DebugTrace(-1, Dbg, b"FatDeleteDirent -> (VOID)\n\0".as_ptr() as *const i8, 0i32);
}

#[no_mangle]
pub unsafe extern "system" fn FatLfnDirentExists(
    IrpContext: *mut IRP_CONTEXT,
    Dcb: *mut DCB,
    Lfn: *mut _UNICODE_STRING,
    LfnTmp: *mut _UNICODE_STRING,
) -> i32 {
    let mut ccb: CCB = core::mem::zeroed();
    let mut dirent: *mut DIRENT = core::ptr::null_mut();
    let mut dirent_bcb: *mut BCB = core::ptr::null_mut();
    let mut dirent_byte_offset: u32 = 0;
    let mut result: i32 = 0;
    let mut flags: u32 = 0;

    PAGED_CODE!();

    ccb.__bindgen_anon_1.__bindgen_anon_1.as_mut().UnicodeQueryTemplate = *Lfn;
    ccb.ContainsWildCards = 0;
    ccb.Flags = CCB_FLAG_SKIP_SHORT_NAME_COMPARE | CCB_FLAG_QUERY_TEMPLATE_MIXED;

    let p_dirent_bcb = core::ptr::addr_of_mut!(dirent_bcb);
    let p_result = core::ptr::addr_of_mut!(result);
    crate::seh::try_finally(
        &mut (),
        |_| {
            let mut dirent_byte_offset_li: VBO = 0;
            FatLocateDirent(
                IrpContext,
                Dcb,
                &mut ccb,
                0u32,
                &mut flags,
                &mut dirent,
                p_dirent_bcb,
                &mut dirent_byte_offset_li,
                core::ptr::null_mut(),
                LfnTmp,
                core::ptr::null_mut(),
            );
            dirent_byte_offset = dirent_byte_offset_li;
        },
        |_, _| {
            if !(*p_dirent_bcb).is_null() {
                *p_result = 1;
            }
            FatUnpinBcb(IrpContext, p_dirent_bcb as *mut *mut core::ffi::c_void);
        },
    );

    result
}


#[no_mangle]
pub unsafe extern "system" fn FatLocateDirent(
    IrpContext: PIRP_CONTEXT,
    ParentDirectory: PDCB,
    Ccb: PCCB,
    OffsetToStartSearchFrom: u32, // C: VBO=ULONG32 (4 bytes) → @44 matches callers
    Flags: PULONG,
    Dirent: *mut PDIRENT,
    Bcb: PBCB,
    ByteOffset: *mut VBO,
    FileNameDos: PBOOLEAN,
    LongFileName: PUNICODE_STRING,
    OrigLongFileName: PUNICODE_STRING,
) {
    let mut Status: NTSTATUS = (STATUS_SUCCESS) as u32;
    let mut Name: OEM_STRING = OEM_STRING {
        Length: 0,
        MaximumLength: 12,
        Buffer: core::ptr::null_mut::<CHAR>(),
    };
    let mut NameBuffer: [u8; 12] = [0u8; 12];

    let mut UpcasedLfnValid: BOOLEAN = 0;
    let mut UpcasedLfn: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut::<WCHAR>(),
    };
    let mut LocalLfnBuffer: [WCHAR; 32] = [0u16; 32];

    let mut LfnInProgress: BOOLEAN = 0;
    let mut LfnChecksum: u8 = 0;
    let mut LfnSize: u32 = 0;
    let mut LfnIndex: u32 = 0;
    let mut Ordinal: u8 = 0;
    let mut LfnByteOffset: VBO = 0;

    TimerStart(0u32);
    // PAGED_CODE() ignored

    DebugTrace(1i32, 0u32, b"FatLocateDirent\n\0".as_ptr() as *const i8, 0usize);
    DebugTrace(0i32, 0u32, b"  ParentDirectory         = %p\n\0".as_ptr() as *const i8, ParentDirectory as usize);
    DebugTrace(0i32, 0u32, b"  OffsetToStartSearchFrom = %08lx\n\0".as_ptr() as *const i8, (OffsetToStartSearchFrom as i64) as u32 as usize);
    DebugTrace(0i32, 0u32, b"  Dirent                  = %p\n\0".as_ptr() as *const i8, Dirent as usize);
    DebugTrace(0i32, 0u32, b"  Bcb                     = %p\n\0".as_ptr() as *const i8, Bcb as usize);
    DebugTrace(0i32, 0u32, b"  ByteOffset              = %08lx\n\0".as_ptr() as *const i8, *ByteOffset as usize);

    // C source is NT_ASSERT (dirsup.c:1175) — a DEBUG-only assertion that is a no-op in
    // the shipping (free) build; the cohab shim `NT_ASSERT` (rust_cohab_stub.c) is a no-op.
    // The original translation rendered it as a hard `assert!`, which panics unconditionally
    // and BUGCHECKed arm64 boot once IRP_MJ_CLEANUP started being delivered (ob-rust
    // ObpDecrementHandleCount fix): FatCommonCleanup -> FatNotifyReportChange ->
    // FatSetFullFileNameInFcb -> FatGetUnicodeNameFromFcb -> here, on the normal
    // UserFileOpen cleanup path where FatCommonCleanup legitimately holds only the file's
    // Fcb (not the parent Dcb / Vcb) — exactly as ReactOS does. Route through NT_ASSERT
    // like every other NT_ASSERT in this file (cf. dirsup.rs:1157/1164).
    NT_ASSERT(
        (ExIsResourceAcquiredSharedLite((*ParentDirectory).Header.Resource) != 0
            || ExIsResourceAcquiredExclusiveLite((*ParentDirectory).Header.Resource) != 0
            || ExIsResourceAcquiredSharedLite(core::ptr::addr_of_mut!((*(*ParentDirectory).Vcb).Resource)) != 0
            || ExIsResourceAcquiredExclusiveLite(core::ptr::addr_of_mut!((*(*ParentDirectory).Vcb).Resource)) != 0)
            as i32,
    );

    // UNREFERENCED_PARAMETER( Flags ) -> ignore
    let _ = Flags;

    Name.MaximumLength = 12;
    Name.Buffer = NameBuffer.as_mut_ptr() as *mut CHAR;

    UpcasedLfn.Length = 0;
    UpcasedLfn.MaximumLength = core::mem::size_of_val(&LocalLfnBuffer) as u16;
    UpcasedLfn.Buffer = LocalLfnBuffer.as_mut_ptr();

    if !(*Bcb).is_null() {
        if (OffsetToStartSearchFrom / PAGE_SIZE as u32) == (*ByteOffset / PAGE_SIZE as u32) {
            *Dirent = (*Dirent).add(((OffsetToStartSearchFrom - *ByteOffset) / core::mem::size_of::<DIRENT>() as u32) as usize);
        } else {
            FatUnpinBcb(IrpContext, core::ptr::addr_of_mut!(*Bcb) as *mut *mut core::ffi::c_void);
        }
    }

    if ARGUMENT_PRESENT(LongFileName) {
        (*LongFileName).Length = 0;
    }
    if ARGUMENT_PRESENT(OrigLongFileName) {
        (*OrigLongFileName).Length = 0;
    }

    if !FileNameDos.is_null() {
        *FileNameDos = 0;
    }

    *ByteOffset = (OffsetToStartSearchFrom + (core::mem::size_of::<DIRENT>() as u32 - 1)) & !(core::mem::size_of::<DIRENT>() as u32 - 1);

    let p_UpcasedLfn = core::ptr::addr_of_mut!(UpcasedLfn);
    crate::seh::try_finally(
        &mut (),
        |_| {
            loop {
                let mut FoundValidLfn: BOOLEAN = 0;

                UpcasedLfnValid = 0;

                // FatReadDirent macro expansion
                {
                    if *ByteOffset >= (*ParentDirectory).Header.AllocationSize.__bindgen_anon_1.LowPart {
                        Status = STATUS_END_OF_FILE;
                        FatUnpinBcb(IrpContext, core::ptr::addr_of_mut!(*Bcb) as *mut *mut core::ffi::c_void);
                    } else if (*ByteOffset % PAGE_SIZE as u32 == 0) || (*Bcb).is_null()  {
                        FatUnpinBcb(IrpContext, core::ptr::addr_of_mut!(*Bcb) as *mut *mut core::ffi::c_void);
                        FatReadDirectoryFile(
                            IrpContext,
                            ParentDirectory,
                            *ByteOffset & !(PAGE_SIZE as u32 - 1),
                            PAGE_SIZE,
                            0i32,
                            Bcb,
                            Dirent,
                            &mut Status as *mut NTSTATUS as *mut i32,
                        );
                        *Dirent = ((*Dirent as *mut u8).add((*ByteOffset % PAGE_SIZE as u32) as usize)) as *mut DIRENT;
                    }
                }

                if Status == STATUS_END_OF_FILE || (*(*Dirent)).FileName[0] == FAT_DIRENT_NEVER_USED {
                    DebugTrace(0i32, 0u32, b"End of directory: entry not found.\n\0".as_ptr() as *const i8, 0usize);
                    FatUnpinBcb(IrpContext, core::ptr::addr_of_mut!(*Bcb) as *mut *mut core::ffi::c_void);
                    *Dirent = core::ptr::null_mut();
                    *ByteOffset = 0;
                    break;
                }

                if (*(*Dirent)).FileName[0] == FAT_DIRENT_DELETED {
                    LfnInProgress = 0;
                    // goto GetNextDirent: advance offset+pointer THEN loop. A bare
                    // `continue` jumps to the loop header and skips the increment at
                    // the bottom (lines below), pinning ByteOffset on one dirent forever.
                    *ByteOffset += core::mem::size_of::<DIRENT>() as u32;
                    *Dirent = (*Dirent).add(1);
                    continue;
                }

                if FatData.ChicagoMode() != 0 && ARGUMENT_PRESENT(LongFileName) && (*(*Dirent)).Attributes == FAT_DIRENT_ATTR_LFN {
                    let Lfn: PLFN_DIRENT = *Dirent as PLFN_DIRENT;
                    if LfnInProgress != 0 {
                        if ((*Lfn).Ordinal & FAT_LAST_LONG_ENTRY) != 0
                            || (*Lfn).Ordinal == 0
                            || (*Lfn).Ordinal != Ordinal - 1
                            || (*Lfn).Checksum != LfnChecksum
                            || (*Lfn).MustBeZero != 0
                        {
                            LfnInProgress = 0;
                        } else {
                            ntkrnl_sys::ASSERT!((LfnIndex % 13) == 0 && LfnIndex != 0);
                            LfnIndex -= 13;

                            core::ptr::copy_nonoverlapping(
                                core::ptr::addr_of!((*Lfn).Name1) as *const WCHAR,
                                (*LongFileName).Buffer.add(LfnIndex as usize),
                                5usize,
                            );
                            core::ptr::copy_nonoverlapping(
                                core::ptr::addr_of!((*Lfn).Name2) as *const WCHAR,
                                (*LongFileName).Buffer.add(LfnIndex as usize + 5),
                                6usize,
                            );
                            core::ptr::copy_nonoverlapping(
                                core::ptr::addr_of!((*Lfn).Name3) as *const WCHAR,
                                (*LongFileName).Buffer.add(LfnIndex as usize + 11),
                                2usize,
                            );

                            Ordinal = (*Lfn).Ordinal;
                            LfnByteOffset = *ByteOffset;
                        }
                    }

                    if LfnInProgress == 0
                        && ((*Lfn).Ordinal & FAT_LAST_LONG_ENTRY) != 0
                        && ((*Lfn).Ordinal & !FAT_LAST_LONG_ENTRY) <= MAX_LFN_DIRENTS
                        && (*Lfn).MustBeZero == 0
                    {
                        let mut CheckTail: BOOLEAN = 0;

                        Ordinal = (*Lfn).Ordinal & !FAT_LAST_LONG_ENTRY;

                        if Ordinal == 0 {
                            FatPopUpFileCorrupt(IrpContext, ParentDirectory);
                            FatRaiseStatus(IrpContext, STATUS_FILE_CORRUPT_ERROR);
                        }

                        LfnIndex = (Ordinal as u32 - 1) * 13;
                        FatEnsureStringBufferEnough(LongFileName, ((LfnIndex + 13) * 2) as u16);

                        core::ptr::copy_nonoverlapping(
                            core::ptr::addr_of!((*Lfn).Name1) as *const WCHAR,
                            (*LongFileName).Buffer.add(LfnIndex as usize),
                            5usize,
                        );
                        core::ptr::copy_nonoverlapping(
                            core::ptr::addr_of!((*Lfn).Name2) as *const WCHAR,
                            (*LongFileName).Buffer.add(LfnIndex as usize + 5),
                            6usize,
                        );
                        core::ptr::copy_nonoverlapping(
                            core::ptr::addr_of!((*Lfn).Name3) as *const WCHAR,
                            (*LongFileName).Buffer.add(LfnIndex as usize + 11),
                            2usize,
                        );

                        // Compute Lfn size and check tail
                        while LfnIndex != (Ordinal as u32 * 13) {
                            if CheckTail == 0 {
                                if *(*LongFileName).Buffer.add(LfnIndex as usize) == 0x0000 {
                                    LfnSize = LfnIndex;
                                    CheckTail = 1;
                                }
                            } else {
                                if *(*LongFileName).Buffer.add(LfnIndex as usize) != 0xffff {
                                    break;
                                }
                            }
                            LfnIndex += 1;
                        }

                        if LfnIndex == (Ordinal as u32 * 13) {
                            if CheckTail == 0 {
                                LfnSize = LfnIndex;
                            }
                            LfnIndex -= 13;
                            LfnInProgress = 1;
                            LfnChecksum = (*Lfn).Checksum;
                            LfnByteOffset = *ByteOffset;
                        }
                    }

                    // goto GetNextDirent: advance offset+pointer THEN loop. A bare
                    // `continue` jumps to the loop header and skips the increment at
                    // the bottom (lines below), pinning ByteOffset on one dirent forever.
                    *ByteOffset += core::mem::size_of::<DIRENT>() as u32;
                    *Dirent = (*Dirent).add(1);
                    continue;
                }

                if FlagOn((*(*Dirent)).Attributes as u32, FAT_DIRENT_ATTR_VOLUME_ID as u32) != 0 {
                    if FlagOn((*Ccb).Flags, CCB_FLAG_MATCH_VOLUME_ID) != 0 {
                        break;
                    }
                    // goto GetNextDirent: advance offset+pointer THEN loop. A bare
                    // `continue` jumps to the loop header and skips the increment at
                    // the bottom (lines below), pinning ByteOffset on one dirent forever.
                    *ByteOffset += core::mem::size_of::<DIRENT>() as u32;
                    *Dirent = (*Dirent).add(1);
                    continue;
                }

                // Check for valid LFN run
                if LfnInProgress != 0
                    && *ByteOffset == LfnByteOffset + core::mem::size_of::<DIRENT>() as u32
                    && LfnIndex == 0
                    && FatComputeLfnChecksum(*Dirent) == LfnChecksum
                {
                    ntkrnl_sys::ASSERT!(Ordinal == 1);
                    FoundValidLfn = 1;
                    (*LongFileName).Length = (LfnSize * core::mem::size_of::<WCHAR>() as u32) as u16;

                    if ARGUMENT_PRESENT(OrigLongFileName) {
                        *OrigLongFileName = *LongFileName;
                    }
                } else {
                    FoundValidLfn = 0;
                }

                if FlagOn((*Ccb).Flags, CCB_FLAG_MATCH_ALL) != 0 {
                    break;
                }

                if FlagOn((*Ccb).Flags, CCB_FLAG_SKIP_SHORT_NAME_COMPARE) == 0 {
                    if (*Ccb).ContainsWildCards != 0 {
                        _ = Fat8dot3ToString(IrpContext, *Dirent, 0, &mut Name);

                        if Name.Length == 2
                            && *Name.Buffer.add(0) == '.' as i8
                            && *Name.Buffer.add(1) == '.' as i8
                            && (*Ccb).__bindgen_anon_1.__bindgen_anon_1.as_ref().OemQueryTemplate.Wild.as_ref().Length > 1
                        {
                            Name.Length = 1;
                        }

                        if FatIsNameInExpression(
                            IrpContext,
                            *(*Ccb).__bindgen_anon_1.__bindgen_anon_1.as_ref().OemQueryTemplate.Wild.as_ref(),
                            Name,
                        ) != 0 {
                            DebugTrace(0i32, 0u32, b"Entry found: Name = \"%Z\"\n\0".as_ptr() as *const i8, &Name as *const _ as usize);
                            DebugTrace(0i32, 0u32, b"             VBO  = %08lx\n\0".as_ptr() as *const i8, *ByteOffset as usize);

                            if !FileNameDos.is_null() {
                                *FileNameDos = 1;
                            }

                            SetFlag(core::ptr::addr_of_mut!((*(Ccb as *mut CCB)).Flags), CCB_FLAG_OPENED_BY_SHORTNAME);

                            break;
                        }
                    } else {
                        if FlagOn((*(*Dirent)).Attributes as u32, FAT_DIRENT_ATTR_VOLUME_ID as u32) == 0
                            && (*Ccb).__bindgen_anon_1.__bindgen_anon_1.as_ref().OemQueryTemplate.Constant.as_ref().as_ptr().add(0).cast::<u32>().read_unaligned() == *((*(*Dirent)).FileName.as_ptr().add(0) as *const u32)
                            && (*Ccb).__bindgen_anon_1.__bindgen_anon_1.as_ref().OemQueryTemplate.Constant.as_ref().as_ptr().add(4).cast::<u32>().read_unaligned() == *((*(*Dirent)).FileName.as_ptr().add(4) as *const u32)
                            && (*Ccb).__bindgen_anon_1.__bindgen_anon_1.as_ref().OemQueryTemplate.Constant.as_ref().as_ptr().add(8).cast::<u16>().read_unaligned() == *((*(*Dirent)).FileName.as_ptr().add(8) as *const u16)
                            && *(*Ccb).__bindgen_anon_1.__bindgen_anon_1.as_ref().OemQueryTemplate.Constant.as_ref().as_ptr().add(10) == *((*(*Dirent)).FileName.as_ptr().add(10) as *const u8)
                        {
                            DebugTrace(0i32, 0u32, b"Entry found.\n\0".as_ptr() as *const i8, 0usize);

                            if !FileNameDos.is_null() {
                                *FileNameDos = 1;
                            }

                            SetFlag(core::ptr::addr_of_mut!((*(Ccb as *mut CCB)).Flags), CCB_FLAG_OPENED_BY_SHORTNAME);

                            break;
                        }
                    }
                }

                if FoundValidLfn != 0 {
                    if (*Ccb).ContainsWildCards == 0
                        && (*Ccb).__bindgen_anon_1.__bindgen_anon_1.as_ref().UnicodeQueryTemplate.Length != (LfnSize * core::mem::size_of::<WCHAR>() as u32) as u16
                    {
                        FoundValidLfn = 0;
                        (*LongFileName).Length = 0;
                        if ARGUMENT_PRESENT(OrigLongFileName) {
                            (*OrigLongFileName).Length = 0;
                        }
                        // goto GetNextDirent: advance offset+pointer THEN loop (a bare
                        // `continue` would skip the bottom-of-loop increment).
                        *ByteOffset += core::mem::size_of::<DIRENT>() as u32;
                        *Dirent = (*Dirent).add(1);
                        continue;
                    }

                    if UpcasedLfnValid == 0 {
                        FatEnsureStringBufferEnough(p_UpcasedLfn, (*LongFileName).Length);

                        Status = (RtlUpcaseUnicodeString(p_UpcasedLfn, LongFileName, 0)) as u32;
                        if !NT_SUCCESS((Status) as i32) {
                            FatNormalizeAndRaiseStatus(IrpContext, Status);
                        }

                        UpcasedLfnValid = 1;
                    }

                    if (*Ccb).ContainsWildCards != 0 {
                        if FsRtlIsNameInExpression(
                            &(*Ccb).__bindgen_anon_1.__bindgen_anon_1.as_ref().UnicodeQueryTemplate as *const _UNICODE_STRING as *mut _UNICODE_STRING,
                            p_UpcasedLfn as *const _UNICODE_STRING as *mut _UNICODE_STRING,
                            1,
                            core::ptr::null_mut(),
                        ) != 0 {
                            break;
                        }
                    } else {
                        if FsRtlAreNamesEqual(
                            &(*Ccb).__bindgen_anon_1.__bindgen_anon_1.as_ref().UnicodeQueryTemplate as *const _UNICODE_STRING,
                            p_UpcasedLfn as *const _UNICODE_STRING,
                            BooleanFlagOn((*Ccb).Flags, CCB_FLAG_QUERY_TEMPLATE_MIXED),
                            core::ptr::null(),
                        ) != 0 {
                            break;
                        }
                    }
                }

                if FoundValidLfn != 0 {
                    FoundValidLfn = 0;
                    (*LongFileName).Length = 0;
                    if ARGUMENT_PRESENT(OrigLongFileName) {
                        (*OrigLongFileName).Length = 0;
                    }
                }

                // GetNextDirent: (we fall through here via continue above)
                *ByteOffset += core::mem::size_of::<DIRENT>() as u32;
                *Dirent = (*Dirent).add(1);
            }
        },
        |_, _| {
            FatFreeStringBuffer(p_UpcasedLfn);
        },
    );

    DebugTrace(-1i32, 0u32, b"FatLocateDirent -> (VOID)\n\0".as_ptr() as *const i8, 0usize);
    TimerStop(0u32, b"FatLocateDirent\0".as_ptr() as *const i8);
}
#[no_mangle]
pub unsafe extern "system" fn FatLocateSimpleOemDirent(
    IrpContext: PIRP_CONTEXT,
    ParentDirectory: PDCB,
    FileName: *const OEM_STRING,
    Dirent: *mut *mut DIRENT,
    Bcb: *mut *mut BCB,
    ByteOffset: *mut u32,
) {
    // PAGED_CODE();

    let mut LocalCcb: CCB = core::mem::zeroed();
    
    FatStringTo8dot3(
        IrpContext,
        *FileName,
        core::ptr::addr_of_mut!((*LocalCcb.__bindgen_anon_1.__bindgen_anon_1.as_mut()).OemQueryTemplate) as *mut OEM_STRING,
    );
    
    LocalCcb.ContainsWildCards = 0;
    LocalCcb.Flags = 0;
    
    // The caller's ByteOffset itself, as in the C (see FatDefragDirectory): with a pinned
    // Bcb, FatLocateDirent reads it as where Dirent was left. A fresh 0 made a re-scan
    // skip everything before the previous hit, so FatSelectNames' numeric-tail loop
    // could call a taken 8.3 name free and create a duplicate short name.
    FatLocateDirent(
        IrpContext,
        ParentDirectory,
        &mut LocalCcb,
        0u32,
        core::ptr::null_mut(),
        Dirent,
        Bcb,
        ByteOffset,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
}

#[no_mangle]
pub unsafe extern "system" fn FatLocateVolumeLabel(
    irp_context: *mut IRP_CONTEXT,
    vcb: *mut VCB,
    dirent: *mut *mut DIRENT,
    bcb: *mut *mut BCB,
    byte_offset: *mut u32,
) {
    let status_success: i32 = 0; // STATUS_SUCCESS assumed 0x00000000
    let status_end_of_file: i32 = 0xC0000011_u32 as i32; // STATUS_END_OF_FILE (from constants, but defined inline for clarity)

    let mut status: i32 = status_success;

    DebugTrace(1, Dbg, b"FatLocateVolumeLabel\n\0".as_ptr() as *const i8, 0);
    DebugTrace(0, Dbg, b"  Vcb        = %p\n\0".as_ptr() as *const i8, vcb);
    DebugTrace(0, Dbg, b"  Dirent     = %p\n\0".as_ptr() as *const i8, dirent);
    DebugTrace(0, Dbg, b"  Bcb        = %p\n\0".as_ptr() as *const i8, bcb);
    DebugTrace(0, Dbg, b"  ByteOffset = %08lx\n\0".as_ptr() as *const i8, *byte_offset);

    *bcb = core::ptr::null_mut();
    *byte_offset = 0;

    loop {
        FatReadDirent(
            irp_context,
            (*vcb).RootDcb,
            *byte_offset,
            bcb,
            dirent,
            &raw mut status,
        );

        if (status == status_end_of_file)
            || ((**dirent).FileName[0] == FAT_DIRENT_NEVER_USED)
        {
            DebugTrace(0, Dbg, b"Volume label not found.\n\0".as_ptr() as *const i8, 0);

            FatUnpinBcb(irp_context, core::ptr::addr_of_mut!(*bcb) as *mut *mut core::ffi::c_void);

            *dirent = core::ptr::null_mut();
            *byte_offset = 0;
            break;
        }

        if (((**dirent).Attributes & !FAT_DIRENT_ATTR_ARCHIVE) == FAT_DIRENT_ATTR_VOLUME_ID)
            && ((**dirent).FileName[0] != FAT_DIRENT_DELETED)
        {
            DebugTrace(0, Dbg, b"Volume label found at VBO = %08lx\n\0".as_ptr() as *const i8, *byte_offset);

            FatPinMappedData(
                irp_context,
                (*vcb).RootDcb,
                *byte_offset,
                core::mem::size_of::<DIRENT>() as u32,
                bcb,
            );
            break;
        }

        *byte_offset += core::mem::size_of::<DIRENT>() as u32;
        *dirent = (*dirent).add(1);
    }

    DebugTrace(-1, Dbg, b"FatLocateVolumeLabel -> (VOID)\n\0".as_ptr() as *const i8, 0);
}
#[no_mangle]
pub unsafe extern "system" fn FatGetDirentFromFcbOrDcb(
    IrpContext: *mut IRP_CONTEXT,
    FcbOrDcb: *mut FCB,
    ReturnOnFailure: BOOLEAN,
    Dirent: *mut *mut DIRENT,
    Bcb: *mut *mut BCB,
) -> () {
    let mut DontCare: i32 = ntkrnl_sys::base::STATUS_SUCCESS;

    PAGED_CODE!();

    DebugTrace(
        1,
        Dbg,
        b"FatGetDirentFromFcbOrDcb\n\0".as_ptr() as *const i8,
        0usize,
    );

    DebugTrace(
        0,
        Dbg,
        b"  FcbOrDcb = %p\n\0".as_ptr() as *const i8,
        FcbOrDcb as usize,
    );
    DebugTrace(
        0,
        Dbg,
        b"  Dirent   = %p\n\0".as_ptr() as *const i8,
        Dirent as usize,
    );
    DebugTrace(
        0,
        Dbg,
        b"  Bcb      = %p\n\0".as_ptr() as *const i8,
        Bcb as usize,
    );

    // Assert that we are not attempting this on the root directory.
    NT_ASSERT((NodeType((FcbOrDcb).cast()) != FAT_NTC_ROOT_DCB as u32) as i32);

    // We know the offset of the dirent within the directory file,
    // so we just read it (with pinning).
    FatReadDirectoryFile(
        IrpContext,
        (*FcbOrDcb).ParentDcb,
        (*FcbOrDcb).DirentOffsetWithinDirectory,
        core::mem::size_of::<DIRENT>() as u32,
        1, // TRUE
        Bcb,
        Dirent,
        &raw mut DontCare,
    );

    // Previous call can fail.  We used to assert success, but we use this
    // as part of volume verification after media has been removed.
    let vcb_state = (*((*FcbOrDcb).Vcb)).VcbState as u8;
    let cond1 = FlagOn((vcb_state) as u32, VCB_STATE_FLAG_REMOVABLE_MEDIA) != 0;
    let cond2 = ntkrnl_sys::base::NT_SUCCESS(DontCare);
    NT_ASSERT((cond1 || cond2) as i32);

    // Note also that the only way this could fail is if the Fcb was being
    // verified.  This can't happen if the Fcb is in good condition.
    // Also a Prefix catch.
    let cond3 = ntkrnl_sys::base::NT_SUCCESS(DontCare);
    let cond4 = (*FcbOrDcb).FcbCondition == FcbNeedsToBeVerified;
    NT_ASSERT((cond3 || cond4) as i32);

    // This should never happen except in very specific cases (during volume
    // verify) but we'll handle and raise here to save all callers checking the
    // pointers.
    if (*Dirent).is_null() && ReturnOnFailure == 0 {
        NT_ASSERT(0i32); // FALSE
        FatRaiseStatus(IrpContext, STATUS_FILE_CORRUPT_ERROR);
    }

    DebugTrace(
        -1,
        Dbg,
        b"FatGetDirentFromFcbOrDcb -> (VOID)\n\0".as_ptr() as *const i8,
        0usize,
    );
}

#[no_mangle]
pub unsafe extern "system" fn FatIsDirectoryEmpty(
    IrpContext: PIRP_CONTEXT,
    Dcb: *mut DCB,
) -> i32 {
    let mut Bcb: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut ByteOffset: u32 = if NodeType((Dcb).cast()) == 0x504 /* FAT_NTC_ROOT_DCB */ {
        0u32
    } else {
        2u32 * core::mem::size_of::<DIRENT>() as u32
    };
    let mut Dirent: *mut DIRENT = core::ptr::null_mut();
    let mut IsDirectoryEmpty: i32 = 0;
    let mut Status: i32 = STATUS_SUCCESS;

    DebugTrace(1, 0 /* Dbg */, b"FatIsDirectoryEmpty\n\0".as_ptr() as *const i8, 0);
    DebugTrace(0, 0 /* Dbg */, b"  Dcb              = %p\n\0".as_ptr() as *const i8, Dcb as *mut core::ffi::c_void);
    DebugTrace(0, 0 /* Dbg */, b"  IsDirectoryEmpty = %08lx\n\0".as_ptr() as *const i8, IsDirectoryEmpty as u32);

    let p_Bcb = core::ptr::addr_of_mut!(Bcb);
    crate::seh::try_finally(
        &mut (),
        |_| {
            loop {
                FatReadDirent(
                    IrpContext,
                    Dcb,
                    ByteOffset,
                    p_Bcb as *mut *mut BCB,
                    &mut Dirent,
                    &mut Status,
                );

                if (Status == (STATUS_END_OF_FILE) as i32) ||
                   ((*Dirent).FileName[0] == 0x00u8 /* FAT_DIRENT_NEVER_USED */) {
                    DebugTrace(0, 0 /* Dbg */,
                        b"Empty.  Last exempt entry at VBO = %08lx\n\0".as_ptr() as *const i8,
                        ByteOffset as u32);
                    IsDirectoryEmpty = 1;
                    break;
                }

                if ((*Dirent).FileName[0] != 0xE5u8 /* FAT_DIRENT_DELETED */) &&
                   ((*Dirent).Attributes != 0x0Fu8 /* FAT_DIRENT_ATTR_LFN */) {
                    break;
                }

                ByteOffset += core::mem::size_of::<DIRENT>() as u32;
                Dirent = Dirent.add(1);
            }
        },
        |_, _| {
            FatUnpinBcb(IrpContext, p_Bcb);
        },
    );

    DebugTrace(-1, 0 /* Dbg */, b"FatIsDirectoryEmpty -> %ld\n\0".as_ptr() as *const i8, IsDirectoryEmpty as i32);

    return IsDirectoryEmpty;
}
#[no_mangle]
pub unsafe extern "system" fn FatConstructDirent(
    IrpContext: *mut IRP_CONTEXT,
    Dirent: *mut DIRENT,
    FileName: *const OEM_STRING,
    ComponentReallyLowercase: BOOLEAN,
    ExtensionReallyLowercase: BOOLEAN,
    Lfn: *const _UNICODE_STRING,
    Attributes: u16,
    ZeroAndSetTimeFields: BOOLEAN,
    SetCreationTime: *const i64, // PLARGE_INTEGER
) {
    PAGED_CODE!();

    DebugTrace(1, Dbg, b"FatConstructDirent\n\0" as *const u8 as *const i8, 0);
    DebugTrace(
        0,
        Dbg,
        b"  Dirent             = %p\n\0" as *const u8 as *const i8,
        Dirent,
    );
    DebugTrace(
        0,
        Dbg,
        b"  FileName           = %Z\n\0" as *const u8 as *const i8,
        FileName,
    );
    DebugTrace(
        0,
        Dbg,
        b"  Attributes         = %08lx\n\0" as *const u8 as *const i8,
        Attributes as i32,
    );

    if ZeroAndSetTimeFields != 0 {
        RtlZeroMemory(Dirent as *mut core::ffi::c_void, core::mem::size_of::<DIRENT>());
    }

    // Pass the OEM_STRING by value (dereference the pointer)
    FatStringTo8dot3(
        IrpContext,
        *FileName,
        (core::ptr::addr_of_mut!((*Dirent).FileName[0]) as *mut u8).cast(),
    );

    if ZeroAndSetTimeFields != 0 || !SetCreationTime.is_null() {
        let mut Time: i64 = 0;
        let mut SaveTime: i64 = 0;

        KeQuerySystemTime(&mut Time as *mut i64 as *mut LARGE_INTEGER);

        if FatData.ChicagoMode() != 0 {
            if SetCreationTime.is_null()
                || FatNtTimeToFatTime(
                    IrpContext,
                    SetCreationTime as *mut LARGE_INTEGER,
                    0u8,
                    core::ptr::addr_of_mut!((*Dirent).CreationTime) as *mut FAT_TIME_STAMP,
                    core::ptr::addr_of_mut!((*Dirent).CreationMSec),
                ) == 0
            {
                SaveTime = Time;
                if FatNtTimeToFatTime(
                    IrpContext,
                    &mut SaveTime as *mut i64 as *mut LARGE_INTEGER,
                    0u8,
                    core::ptr::addr_of_mut!((*Dirent).CreationTime) as *mut FAT_TIME_STAMP,
                    core::ptr::addr_of_mut!((*Dirent).CreationMSec),
                ) == 0
                {
                    RtlZeroMemory(
                        core::ptr::addr_of_mut!((*Dirent).CreationTime) as *mut core::ffi::c_void,
                        core::mem::size_of::<FAT_TIME_STAMP>(),
                    );
                    (*Dirent).CreationMSec = 0;
                }
            }
        }

        if ZeroAndSetTimeFields != 0 {
            if FatNtTimeToFatTime(
                IrpContext,
                &Time as *const i64 as *mut LARGE_INTEGER,
                1u8,
                core::ptr::addr_of_mut!((*Dirent).LastWriteTime) as *mut FAT_TIME_STAMP,
                core::ptr::null_mut(),
            ) == 0
            {
                DebugTrace(0, Dbg, b"Current time invalid.\n\0" as *const u8 as *const i8, 0);
                RtlZeroMemory(
                    core::ptr::addr_of_mut!((*Dirent).LastWriteTime) as *mut core::ffi::c_void,
                    core::mem::size_of::<FAT_TIME_STAMP>(),
                );
            }
            if FatData.ChicagoMode() != 0 {
                (*Dirent).LastAccessDate = core::ptr::read(core::ptr::addr_of!((*Dirent).LastWriteTime.Date));
            }
        }
    }

    (*Dirent).Attributes = Attributes as u8;
    (*Dirent).NtByte = 0;

    if ComponentReallyLowercase != 0 {
        SetFlag(
            (core::ptr::addr_of_mut!((*Dirent).NtByte) as *mut u8).cast(),
            FAT_DIRENT_NT_BYTE_8_LOWER_CASE,
        );
    }
    if ExtensionReallyLowercase != 0 {
        SetFlag(
            (core::ptr::addr_of_mut!((*Dirent).NtByte) as *mut u8).cast(),
            FAT_DIRENT_NT_BYTE_3_LOWER_CASE,
        );
    }

    if !Lfn.is_null() {
        let DirentChecksum: u8;
        let DirentsInLfn: u8;
        let mut LfnOrdinal: u8;
        let mut LfnBuffer: *const u16; // PWCHAR
        let mut LfnDirent: *mut LFN_DIRENT;

        NT_ASSERT((FatData.ChicagoMode() as u32) as i32);

        DirentChecksum = FatComputeLfnChecksum(Dirent);
        LfnOrdinal = FAT_LFN_DIRENTS_NEEDED(Lfn as *mut _UNICODE_STRING) as u8;
        DirentsInLfn = LfnOrdinal;

        LfnBuffer = (*Lfn).Buffer
            .add(((DirentsInLfn as usize).wrapping_sub(1)).wrapping_mul(13));

        NT_ASSERT(((DirentsInLfn <= MAX_LFN_DIRENTS) as u32) as i32);

        LfnDirent = (Dirent as *mut LFN_DIRENT).offset(-(DirentsInLfn as isize));

        while LfnDirent < (Dirent as *mut LFN_DIRENT) {
            let mut FinalLfnBuffer: [u16; 13] = [0u16; 13];
            let Buffer: *const u16;

            if LfnOrdinal == DirentsInLfn {
                let RemainderChars: usize;
                let mut i: usize;

                RemainderChars =
                    ((*Lfn).Length as usize / core::mem::size_of::<u16>()) % 13;

                (*LfnDirent).Ordinal = (LfnOrdinal | FAT_LAST_LONG_ENTRY) as u8;

                if RemainderChars != 0 {
                    RtlCopyMemory(
                        FinalLfnBuffer.as_mut_ptr() as *mut core::ffi::c_void,
                        LfnBuffer as *const core::ffi::c_void,
                        RemainderChars.wrapping_mul(core::mem::size_of::<u16>()),
                    );

                    i = RemainderChars;
                    while i < 13 {
                        if i == RemainderChars {
                            FinalLfnBuffer[i] = 0x0000;
                        } else {
                            FinalLfnBuffer[i] = 0xFFFF;
                        }
                        i += 1;
                    }
                    Buffer = FinalLfnBuffer.as_ptr();
                } else {
                    Buffer = LfnBuffer;
                }
            } else {
                (*LfnDirent).Ordinal = LfnOrdinal;
                Buffer = LfnBuffer;
            }

            // Copy LFN fragments
            RtlCopyMemory(
                (*LfnDirent).Name1.as_mut_ptr() as *mut core::ffi::c_void,
                Buffer as *const core::ffi::c_void,
                5 * core::mem::size_of::<u16>(),
            );
            RtlCopyMemory(
                (*LfnDirent).Name2.as_mut_ptr() as *mut core::ffi::c_void,
                Buffer.add(5) as *const core::ffi::c_void,
                6 * core::mem::size_of::<u16>(),
            );
            RtlCopyMemory(
                (*LfnDirent).Name3.as_mut_ptr() as *mut core::ffi::c_void,
                Buffer.add(11) as *const core::ffi::c_void,
                2 * core::mem::size_of::<u16>(),
            );

            // Set remaining fields
            (*LfnDirent).Attributes = FAT_DIRENT_ATTR_LFN;
            (*LfnDirent).Type = 0;
            (*LfnDirent).Checksum = DirentChecksum;
            (*LfnDirent).MustBeZero = 0;

            LfnDirent = LfnDirent.offset(1);
            LfnOrdinal -= 1;
            LfnBuffer = LfnBuffer.sub(13);
        }
    }

    DebugTrace(-1, Dbg, b"FatConstructDirent -> (VOID)\n\0" as *const u8 as *const i8, 0);
}

#[no_mangle]
pub unsafe extern "system" fn FatConstructLabelDirent(
    IrpContext: *mut IRP_CONTEXT,
    Dirent: *mut DIRENT,
    Label: *mut OEM_STRING,
) {
    PAGED_CODE!();

    DebugTrace(1, Dbg, b"FatConstructLabelDirent\n\0".as_ptr() as *const i8, 0);
    DebugTrace(0, Dbg, b"  Dirent             = %p\n\0".as_ptr() as *const i8, Dirent);
    DebugTrace(0, Dbg, b"  Label              = %Z\n\0".as_ptr() as *const i8, Label);

    core::ptr::write_bytes(Dirent as *mut u8, 0u8, core::mem::size_of::<DIRENT>());

    let label_buffer = (*Label).Buffer;
    let label_length = (*Label).Length;
    let dirent_filename_ptr = core::ptr::addr_of_mut!((*Dirent).FileName) as *mut u8;
    core::ptr::copy_nonoverlapping(label_buffer as *const u8, dirent_filename_ptr, label_length as usize);

    let fill_len = (11 - label_length as usize) as usize;
    core::ptr::write_bytes(dirent_filename_ptr.add(label_length as usize), b' ', fill_len);

    core::ptr::write_unaligned(core::ptr::addr_of_mut!((*Dirent).LastWriteTime), FatGetCurrentFatTime(IrpContext));
    (*Dirent).Attributes = FAT_DIRENT_ATTR_VOLUME_ID;
    *(*Dirent).__bindgen_anon_1.ExtendedAttributes.as_mut() = 0;
    (*Dirent).FileSize = 0;

    DebugTrace(-1, Dbg, b"FatConstructLabelDirent -> (VOID)\n\0".as_ptr() as *const i8, 0);
}

#[no_mangle]
pub unsafe extern "system" fn FatSetFileSizeInDirent(
    IrpContext: *mut IRP_CONTEXT,
    Fcb: *mut FCB,
    AlternativeFileSize: *mut u32,
) {
    let mut Dirent: *mut DIRENT = core::ptr::null_mut();
    let mut DirentBcb: *mut BCB = core::ptr::null_mut();

    PAGED_CODE!();

    NT_ASSERT(((*Fcb).FcbCondition == FcbGood) as i32);

    FatGetDirentFromFcbOrDcb(
        IrpContext,
        Fcb,
        0u8,
        &mut Dirent as *mut *mut DIRENT,
        &mut DirentBcb as *mut *mut BCB,
    );

    let p_DirentBcb = core::ptr::addr_of_mut!(DirentBcb);
    crate::seh::try_finally(
        &mut (),
        |_| {
            (*Dirent).FileSize = if !AlternativeFileSize.is_null() {
                *AlternativeFileSize
            } else {
                (*Fcb).Header.FileSize.__bindgen_anon_1.LowPart
            };
            FatSetDirtyBcb(IrpContext, *p_DirentBcb, (*Fcb).Vcb, ((1u8) as i32) as _);
        },
        |_, _| {
            FatUnpinBcb(IrpContext, p_DirentBcb as *mut *mut core::ffi::c_void);
        },
    );
}

#[no_mangle]
pub unsafe extern "system" fn FatSetFileSizeInDirentNoRaise(
    IrpContext: PIRP_CONTEXT,
    Fcb: PFCB,
    AlternativeFileSize: *mut ULONG,
) {

    // try { FatSetFileSizeInDirent } except (FatExceptionFilter) { NOTHING; }
    let _ = crate::seh::fat_try_except(core::ptr::addr_of!(IrpContext), || {
        FatSetFileSizeInDirent(IrpContext, Fcb, AlternativeFileSize);
    });
}
#[no_mangle]
pub unsafe extern "system" fn FatUpdateDirentFromFcb(
    IrpContext: *mut IRP_CONTEXT,
    FileObject: *mut FILE_OBJECT,
    FcbOrDcb: *mut FCB,
    Ccb: *mut CCB,
) {
    let mut SetArchiveBit: u8;
    let mut UpdateFileSize: u8;
    let mut UpdateLastWriteTime: u8;
    let mut UpdateLastAccessTime: u8 = 0;
    let mut UpdateDirent: u8 = 0;
    let mut Dirent: *mut DIRENT = core::ptr::null_mut();
    let mut DirentBcb: *mut BCB = core::ptr::null_mut();
    let mut NotifyFilter: u32 = 0;
    let mut CurrentFatTime: FAT_TIME_STAMP = core::mem::zeroed();
    let mut CurrentTime: LARGE_INTEGER = core::mem::zeroed();
    let mut CurrentDay: LARGE_INTEGER = core::mem::zeroed();
    let mut LastAccessDay: LARGE_INTEGER = core::mem::zeroed();

    // PAGED_CODE(); // skipped

    if (*FcbOrDcb).FcbCondition != FcbGood
        || NodeType(FcbOrDcb as *mut core::ffi::c_void) == FAT_NTC_ROOT_DCB
        || FlagOn(
            (*(*FcbOrDcb).Vcb).VcbState,
            VCB_STATE_FLAG_WRITE_PROTECTED,
        ) != 0
    {
        return;
    }

    KeQuerySystemTime(core::ptr::addr_of_mut!(CurrentTime));

    SetArchiveBit = BooleanFlagOn((*FileObject).Flags, FO_FILE_MODIFIED);

    UpdateLastWriteTime =
        (FlagOn((*FileObject).Flags, FO_FILE_MODIFIED) != 0
            && FlagOn((*Ccb).Flags, CCB_FLAG_USER_SET_LAST_WRITE) == 0) as u8;

    UpdateFileSize =
        (NodeType(FcbOrDcb as *mut core::ffi::c_void) == FAT_NTC_FCB
            && BooleanFlagOn((*FileObject).Flags, FO_FILE_SIZE_CHANGED) != 0) as u8;

    if FatData.ChicagoMode() != 0
        && (UpdateLastWriteTime != 0
            || FlagOn((*FileObject).Flags, FO_FILE_FAST_IO_READ) != 0)
        && FlagOn((*Ccb).Flags, CCB_FLAG_USER_SET_LAST_ACCESS) == 0
    {
        ExSystemTimeToLocalTime(
            core::ptr::addr_of!((*FcbOrDcb).LastAccessTime),
            core::ptr::addr_of_mut!(LastAccessDay),
        );
        ExSystemTimeToLocalTime(
            core::ptr::addr_of!(CurrentTime),
            core::ptr::addr_of_mut!(CurrentDay),
        );

        LastAccessDay.QuadPart /= FatOneDay.QuadPart;
        CurrentDay.QuadPart /= FatOneDay.QuadPart;

        if LastAccessDay.__bindgen_anon_1.LowPart != CurrentDay.__bindgen_anon_1.LowPart {
            UpdateLastAccessTime = 1;
        } else {
            UpdateLastAccessTime = 0;
        }
    } else {
        UpdateLastAccessTime = 0;
    }


    if SetArchiveBit != 0
        || UpdateFileSize != 0
        || UpdateLastWriteTime != 0
        || UpdateLastAccessTime != 0
    {
        DebugTrace(0, Dbg, (b"Update Time and/or file size on File/Dir\n".as_ptr() as *const u8).cast(), 0);

        // try { try { ... } except (FsRtlIsNtstatusExpected(code)) { FatResetExceptionState }
        // } finally { FatUnpinBcb }
        let p_DirentBcb = core::ptr::addr_of_mut!(DirentBcb);
        crate::seh::try_finally(
            &mut (),
            |_| {
                let r = crate::seh::try_except(
                    &mut (),
                    |_| {
                    // Break parent directory oplock -- NTDDI >= WIN8 only (dirsup.c 2720-2735).
                    #[cfg(ntddi_ge_ntddi_win8)]
                    if !(*FcbOrDcb).ParentDcb.is_null() {
                        FsRtlCheckOplockEx(
                            FatGetFcbOplock((*FcbOrDcb).ParentDcb),
                            (*IrpContext).OriginatingIrp,
                            OPLOCK_FLAG_PARENT_OBJECT,
                            core::ptr::null_mut(),
                            core::ptr::null_mut(),
                            core::ptr::null_mut(),
                        );
                    }

                    FatGetDirentFromFcbOrDcb(
                        IrpContext,
                        FcbOrDcb,
                        0, // FALSE
                        core::ptr::addr_of_mut!(Dirent),
                        p_DirentBcb,
                    );

                    if UpdateLastWriteTime != 0 || UpdateLastAccessTime != 0 {
                        FatNtTimeToFatTime(
                            IrpContext,
                            core::ptr::addr_of!(CurrentTime) as *mut LARGE_INTEGER,
                            1u8, // TRUE
                            core::ptr::addr_of_mut!(CurrentFatTime),
                            core::ptr::null_mut(),
                        );
                    }

                    if SetArchiveBit != 0 {
                        (*Dirent).Attributes |= (FILE_ATTRIBUTE_ARCHIVE) as u8;
                        (*FcbOrDcb).DirentFatFlags |= (FILE_ATTRIBUTE_ARCHIVE) as u8;
                        NotifyFilter |= FILE_NOTIFY_CHANGE_ATTRIBUTES;
                        UpdateDirent = 1;
                    }

                    if UpdateLastWriteTime != 0 {
                        (*FcbOrDcb).LastWriteTime = CurrentTime;
                        core::ptr::write_unaligned(
                            core::ptr::addr_of_mut!((*Dirent).LastWriteTime),
                            core::ptr::read(core::ptr::addr_of!(CurrentFatTime)),
                        );
                        NotifyFilter |= FILE_NOTIFY_CHANGE_LAST_WRITE;
                        UpdateDirent = 1;
                    }

                    if UpdateLastAccessTime != 0 {
                        (*FcbOrDcb).LastAccessTime.QuadPart =
                            CurrentDay.QuadPart * FatOneDay.QuadPart;
                        ExLocalTimeToSystemTime(
                            core::ptr::addr_of!((*FcbOrDcb).LastAccessTime),
                            core::ptr::addr_of_mut!((*FcbOrDcb).LastAccessTime),
                        );
                        core::ptr::write_unaligned(
                            core::ptr::addr_of_mut!((*Dirent).LastAccessDate),
                            core::ptr::read(core::ptr::addr_of!(CurrentFatTime.Date)),
                        );
                        NotifyFilter |= FILE_NOTIFY_CHANGE_LAST_ACCESS;
                        UpdateDirent = 1;
                    }

                    if UpdateFileSize != 0 {
                        NT_ASSERT(
                            (NodeType(FcbOrDcb as *mut core::ffi::c_void) == FAT_NTC_FCB) as i32,
                        );
                        if (*Dirent).FileSize != (*FcbOrDcb).Header.FileSize.__bindgen_anon_1.LowPart {
                            (*Dirent).FileSize = (*FcbOrDcb).Header.FileSize.__bindgen_anon_1.LowPart;
                            NotifyFilter |= FILE_NOTIFY_CHANGE_SIZE;
                            UpdateDirent = 1;
                        }
                    }

                    FatNotifyReportChange(
                        IrpContext,
                        (*FcbOrDcb).Vcb,
                        FcbOrDcb,
                        NotifyFilter,
                        FILE_ACTION_MODIFIED,
                    );

                    if UpdateDirent != 0 {
                        FatSetDirtyBcb(
                            IrpContext,
                            *p_DirentBcb,
                            if NotifyFilter == FILE_NOTIFY_CHANGE_LAST_ACCESS {
                                core::ptr::null_mut()
                            } else {
                                (*FcbOrDcb).Vcb
                            },
                            1, // TRUE
                        );
                    }
                    },
                    |_, pointers| {
                        if FsRtlIsNtstatusExpected(crate::seh::exception_code(pointers) as i32) != 0 {
                            crate::seh::EXCEPTION_EXECUTE_HANDLER
                        } else {
                            crate::seh::EXCEPTION_CONTINUE_SEARCH
                        }
                    },
                );
                if r.is_err() {
                    FatResetExceptionState(IrpContext);
                }
            },
            |_, _| {
                FatUnpinBcb(IrpContext, p_DirentBcb as *mut *mut core::ffi::c_void);
            },
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn FatComputeLfnChecksum(Dirent: *mut DIRENT) -> u8 {
    let mut i: u32;
    let mut Checksum: u8;

    // PAGED_CODE();

    Checksum = (*Dirent).FileName[0] as u8;

    i = 1;
    while i < 11 {
        let byte = (*Dirent).FileName[i as usize] as u8;
        Checksum = (if (Checksum & 1) != 0 { 0x80u8 } else { 0u8 })
            .wrapping_add(Checksum >> 1)
            .wrapping_add(byte);
        i += 1;
    }

    Checksum
}

#[no_mangle]
pub unsafe extern "system" fn FatRescanDirectory(IrpContext: *mut IRP_CONTEXT, Dcb: *mut DCB) {
    let mut bcb: *mut BCB = core::ptr::null_mut();
    let mut dirent: *mut DIRENT = core::ptr::null_mut();
    let mut status: NTSTATUS = (STATUS_SUCCESS) as u32;

    let mut unused_vbo: ULONG = 0;
    let mut deleted_hint: ULONG = 0xFFFFFFFF;
    let mut dirent_index: ULONG = 0;
    let mut dirents_this_run: ULONG = 0;
    let mut start_index_of_this_run: ULONG = 0;

    // Enum RunType
    const INITIAL_RUN: i32 = 0;
    const FREE_DIRENTS: i32 = 1;
    const ALLOCATED_DIRENTS: i32 = 2;
    let mut current_run: i32 = INITIAL_RUN;

    // PAGED_CODE(); // macro ignored
    DebugTrace(
        -1,
        Dbg,
        b"We must scan the whole directory.\n\0".as_ptr() as *const i8,
        0,
    );

    let p_bcb = core::ptr::addr_of_mut!(bcb);
    crate::seh::try_finally(
        &mut (),
        |_| {
            // Inner while(TRUE)
            loop {
                let mut dirent_deleted: u8 = 0; // BOOLEAN

                FatReadDirent(
                    IrpContext,
                    Dcb,
                    unused_vbo,
                    p_bcb,
                    core::ptr::addr_of_mut!(dirent),
                    (core::ptr::addr_of_mut!(status)).cast(),
                );

                if status == STATUS_END_OF_FILE
                    || (*dirent).FileName[0] as u8 == FAT_DIRENT_NEVER_USED
                {
                    break; // exit while
                }

                if (*dirent).FileName[0] as u8 == FAT_DIRENT_DELETED {
                    dirent_deleted = 1; // TRUE
                    if deleted_hint == 0xFFFFFFFF_u32 {
                        deleted_hint = unused_vbo;
                    }
                } else {
                    dirent_deleted = 0; // FALSE
                }

                if current_run == INITIAL_RUN {
                    current_run = if dirent_deleted != 0 { FREE_DIRENTS } else { ALLOCATED_DIRENTS };
                } else {
                    // switch from free to allocated?
                    if current_run == FREE_DIRENTS && dirent_deleted == 0 {
                        dirents_this_run = dirent_index - start_index_of_this_run;
                        RtlClearBits(
                            core::ptr::addr_of_mut!(((*Dcb).Specific.Dcb.as_mut()).FreeDirentBitmap) as *mut _,
                            start_index_of_this_run,
                            dirents_this_run,
                        );
                        current_run = ALLOCATED_DIRENTS;
                        start_index_of_this_run = dirent_index;
                    }

                    // switch from allocated to free?
                    if current_run == ALLOCATED_DIRENTS && dirent_deleted != 0 {
                        dirents_this_run = dirent_index - start_index_of_this_run;
                        RtlSetBits(
                            core::ptr::addr_of_mut!(((*Dcb).Specific.Dcb.as_mut()).FreeDirentBitmap) as *mut _,
                            start_index_of_this_run,
                            dirents_this_run,
                        );
                        current_run = FREE_DIRENTS;
                        start_index_of_this_run = dirent_index;
                    }
                }

                unused_vbo += core::mem::size_of::<DIRENT>() as ULONG;
                dirent = dirent.add(1);
                dirent_index += 1;
            } // end while

            // post‑loop final run
            dirents_this_run = dirent_index - start_index_of_this_run;
            if current_run == FREE_DIRENTS || current_run == INITIAL_RUN {
                RtlClearBits(
                    core::ptr::addr_of_mut!(((*Dcb).Specific.Dcb.as_mut()).FreeDirentBitmap) as *mut _,
                    start_index_of_this_run,
                    dirents_this_run,
                );
            } else {
                RtlSetBits(
                    core::ptr::addr_of_mut!(((*Dcb).Specific.Dcb.as_mut()).FreeDirentBitmap) as *mut _,
                    start_index_of_this_run,
                    dirents_this_run,
                );
            }

            if unused_vbo < (*Dcb).Header.AllocationSize.__bindgen_anon_1.LowPart {
                start_index_of_this_run = unused_vbo / (core::mem::size_of::<DIRENT>() as ULONG);
                dirents_this_run = ((*Dcb).Header.AllocationSize.__bindgen_anon_1.LowPart - unused_vbo)
                    / (core::mem::size_of::<DIRENT>() as ULONG);
                RtlClearBits(
                    core::ptr::addr_of_mut!(((*Dcb).Specific.Dcb.as_mut()).FreeDirentBitmap) as *mut _,
                    start_index_of_this_run,
                    dirents_this_run,
                );
            }

        },
        |_, _| {
            // unpin the Bcb
            FatUnpinBcb(IrpContext, p_bcb as *mut *mut core::ffi::c_void);
        },
    );

    // post‑SEH adjustments
    if deleted_hint == 0xFFFFFFFF_u32 {
        deleted_hint = unused_vbo;
    }
    (*Dcb).Specific.Dcb.as_mut().UnusedDirentVbo = unused_vbo;
    (*Dcb).Specific.Dcb.as_mut().DeletedDirentHint = deleted_hint;
}


#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "system" fn FatDefragDirectory(
    IrpContext: *mut IRP_CONTEXT,
    Dcb: *mut DCB,
    DirentsNeeded: u32,
) -> u32 {
    let mut SavedIrpContextFlag: u32;
    let mut Links: *mut LIST_ENTRY;
    let mut ReturnValue: u32 = 0;
    let mut Fcb: *mut FCB = core::ptr::null_mut();

    let mut bcb: *mut BCB = core::ptr::null_mut();
    let mut Dirent: *mut DIRENT = core::ptr::null_mut();
    let mut Lfn: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };

    let mut Mcb: LARGE_MCB = core::mem::zeroed();
    let mut McbInitialized: BOOLEAN = 0;
    let mut InvalidateFcbs: BOOLEAN = 0;

    let mut Directory: *mut u8 = core::ptr::null_mut();
    let mut UnusedDirents: *mut u8 = core::ptr::null_mut();
    let mut UnusedDirentBuffer: *mut u8 = core::ptr::null_mut();
    let mut UsedDirents: *mut u8 = core::ptr::null_mut();
    let mut UsedDirentBuffer: *mut u8 = core::ptr::null_mut();

    let mut Bcbs: *mut *mut BCB = core::ptr::null_mut();
    let mut Page: u32 = 0;
    let mut PagesPinned: u32 = 0;

    let mut DcbSize: u32;
    let mut TotalBytesAllocated: u32 = 0;

    // PAGED_CODE();
    PAGED_CODE!();

    // NT_ASSERT( FatVcbAcquiredExclusive(IrpContext, Dcb->Vcb) );
    NT_ASSERT((FatVcbAcquiredExclusive(IrpContext, (*Dcb).Vcb)) as i32);

    // DcbSize = Dcb->Header.AllocationSize.LowPart;
    DcbSize = (*Dcb).Header.AllocationSize.__bindgen_anon_1.LowPart;

    // if (DcbSize > 0x40000) { return (ULONG)-1; }
    if DcbSize > 0x40000 {
        return (-1i32) as u32;
    }

    // SavedIrpContextFlag = IrpContext->Flags;
    SavedIrpContextFlag = (*IrpContext).Flags;

    // SetFlag( IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT | IRP_CONTEXT_FLAG_WRITE_THROUGH );
    (*IrpContext).Flags |= IRP_CONTEXT_FLAG_WAIT | IRP_CONTEXT_FLAG_WRITE_THROUGH;

    // for (Links = Dcb->Specific.Dcb.ParentDcbQueue.Flink;
    //      Links != &Dcb->Specific.Dcb.ParentDcbQueue;
    //      Links = Links->Flink) {
    //     Fcb = CONTAINING_RECORD( Links, FCB, ParentDcbLinks );
    //     (VOID)ExAcquireResourceExclusiveLite( Fcb->Header.Resource, TRUE );
    // }
    Links = ((*Dcb).Specific.Dcb.as_mut()).ParentDcbQueue.Flink;
    while Links
        != core::ptr::addr_of_mut!(((*Dcb).Specific.Dcb.as_mut()).ParentDcbQueue) as *mut LIST_ENTRY
    {
        Fcb = (Links as *mut u8)
            .offset(-(core::mem::offset_of!(FCB, ParentDcbLinks) as isize))
            as *mut FCB;
        ExAcquireResourceExclusiveLite((*Fcb).Header.Resource, 1);
        Links = (*Links).Flink;
    }

    // The finally reads everything through raw pointers so the body can own the locals.
    let p_McbInitialized = core::ptr::addr_of_mut!(McbInitialized);
    let p_Mcb = core::ptr::addr_of_mut!(Mcb);
    let p_Lfn = core::ptr::addr_of_mut!(Lfn);
    let p_UnusedDirentBuffer = core::ptr::addr_of_mut!(UnusedDirentBuffer);
    let p_UsedDirentBuffer = core::ptr::addr_of_mut!(UsedDirentBuffer);
    let p_Bcbs = core::ptr::addr_of_mut!(Bcbs);
    let p_PagesPinned = core::ptr::addr_of_mut!(PagesPinned);
    let p_bcb = core::ptr::addr_of_mut!(bcb);
    crate::seh::try_finally(
        &mut (),
        |_| {
            let mut Ccb: CCB = core::mem::zeroed();
            let mut QueryOffset: u32 = 0;
            let mut FoundOffset: u32 = 0;
            let mut BytesUsed: u64 = 0;

            let mut DontCare: NTSTATUS = ntkrnl_sys::base::STATUS_SUCCESS as u32;
            let mut Run: u32 = 0;
            let mut TotalRuns: u32 = 0;
            let mut Result: BOOLEAN;
            let mut Char: *mut u8;

            // RtlZeroMemory( &Ccb, sizeof(CCB) );
            RtlZeroMemory(
                core::ptr::addr_of_mut!(Ccb) as *mut c_void,
                (core::mem::size_of::<CCB>() as u32) as usize,
            );

            // Ccb.Flags = CCB_FLAG_MATCH_ALL | CCB_FLAG_MATCH_VOLUME_ID;
            Ccb.Flags = CCB_FLAG_MATCH_ALL | CCB_FLAG_MATCH_VOLUME_ID;

            // Lfn.MaximumLength = 260 * sizeof(WCHAR);
            Lfn.MaximumLength = (260 * core::mem::size_of::<u16>()) as u16;

            // Lfn.Buffer = FsRtlAllocatePoolWithTag( PagedPool, 260*sizeof(WCHAR), TAG_FILENAME_BUFFER );
            Lfn.Buffer = FsRtlAllocatePoolWithTag(
                PagedPool,
                (260 * core::mem::size_of::<u16>()) as u32,
                TAG_FILENAME_BUFFER,
            ) as *mut u16;

            // FsRtlInitializeLargeMcb( &Mcb, PagedPool );
            FsRtlInitializeLargeMcb(core::ptr::addr_of_mut!(Mcb), PagedPool);

            // McbInitialized = TRUE;
            McbInitialized = 1;

            // do { ... } while ((Dirent != NULL) && (QueryOffset < DcbSize));
            loop {
                // FatLocateDirent( IrpContext, Dcb, &Ccb, QueryOffset, NULL,
                //                  &Dirent, &Bcb, (PVBO)&FoundOffset, NULL,
                //                  &Lfn, NULL );
                // `FoundOffset` itself, as in the C: FatLocateDirent reads *ByteOffset as
                // where the previous call left the pinned Dirent. A fresh 0 each pass made
                // it advance Dirent by QueryOffset/32 from the last hit on page 0, so the
                // compaction scan read the wrong dirents, ended early, and the bitmap it
                // rebuilt called live root entries free -- after which every create in a
                // full FAT16 root raised STATUS_FILE_CORRUPT_ERROR from
                // FatCreateNewDirent. (Same class as FatQueryDirectory's NextVbo.)
                FatLocateDirent(
                    IrpContext,
                    Dcb,
                    core::ptr::addr_of_mut!(Ccb),
                    QueryOffset as u32,
                    core::ptr::null_mut(),
                    core::ptr::addr_of_mut!(Dirent),
                    core::ptr::addr_of_mut!(bcb) as *mut *mut BCB,
                    core::ptr::addr_of_mut!(FoundOffset),
                    core::ptr::null_mut(),
                    core::ptr::addr_of_mut!(Lfn),
                    core::ptr::null_mut(),
                );

                if Dirent != core::ptr::null_mut() {
                    let mut LfnByteOffset: u32;

                    // LfnByteOffset = FoundOffset - FAT_LFN_DIRENTS_NEEDED(&Lfn) * sizeof(LFN_DIRENT);
                    LfnByteOffset = FoundOffset
                        - FAT_LFN_DIRENTS_NEEDED(core::ptr::addr_of_mut!(Lfn))
                            * (core::mem::size_of::<LFN_DIRENT>() as u32);

                    // BytesUsed = FoundOffset - LfnByteOffset + sizeof(DIRENT);
                    BytesUsed =
                        (FoundOffset - LfnByteOffset + core::mem::size_of::<DIRENT>() as u32) as u64;

                    // Result = FsRtlAddLargeMcbEntry( &Mcb, LfnByteOffset, LfnByteOffset, BytesUsed );
                    Result = FsRtlAddLargeMcbEntry(
                        core::ptr::addr_of_mut!(Mcb),
                        LfnByteOffset as i64,
                        LfnByteOffset as i64,
                        BytesUsed as i64,
                    );

                    // NT_ASSERT( Result );
                    NT_ASSERT((Result) as i32);

                    // TotalBytesAllocated += (ULONG) BytesUsed;
                    TotalBytesAllocated += BytesUsed as u32;

                    // QueryOffset = FoundOffset + sizeof(DIRENT);
                    QueryOffset = FoundOffset + core::mem::size_of::<DIRENT>() as u32;
                }

                if !(Dirent != core::ptr::null_mut() && QueryOffset < DcbSize) {
                    break;
                }
            }

            // if (Bcb != NULL) { FatUnpinBcb( IrpContext, Bcb ); }
            if bcb != core::ptr::null_mut() {
                FatUnpinBcb(IrpContext, core::ptr::addr_of_mut!(bcb) as *mut *mut core::ffi::c_void);
            }

            // if (DirentsNeeded > (DcbSize - TotalBytesAllocated)/sizeof(DIRENT)) {
            //     try_return(ReturnValue = (ULONG)-1);
            // }
            if DirentsNeeded
                > (DcbSize - TotalBytesAllocated) / (core::mem::size_of::<DIRENT>() as u32)
            {
                ReturnValue = (-1i32) as u32;
                return; // try_return
            }

            // Allocate UnusedDirentBuffer
            UnusedDirentBuffer = FsRtlAllocatePoolWithTag(
                PagedPool,
                DcbSize - TotalBytesAllocated,
                TAG_DIRENT,
            ) as *mut u8;
            UnusedDirents = UnusedDirentBuffer;

            // Allocate UsedDirentBuffer
            UsedDirentBuffer =
                FsRtlAllocatePoolWithTag(PagedPool, TotalBytesAllocated, TAG_DIRENT) as *mut u8;
            UsedDirents = UsedDirentBuffer;

            // PagesPinned = (DcbSize + (PAGE_SIZE - 1 )) / PAGE_SIZE;
            PagesPinned = (DcbSize + (PAGE_SIZE - 1)) / PAGE_SIZE;

            // Bcbs = FsRtlAllocatePoolWithTag( PagedPool, PagesPinned * sizeof(PBCB), TAG_BCB );
            Bcbs = FsRtlAllocatePoolWithTag(
                PagedPool,
                PagesPinned * (core::mem::size_of::<*mut BCB>() as u32),
                TAG_BCB,
            ) as *mut *mut BCB;

            // RtlZeroMemory( Bcbs, PagesPinned * sizeof(PBCB) );
            RtlZeroMemory(
                Bcbs as *mut c_void,
                (PagesPinned * (core::mem::size_of::<*mut BCB>() as u32)) as usize,
            );

            // for (Page = 0; Page < PagesPinned; Page += 1) { ... }
            Page = 0;
            while Page < PagesPinned {
                let mut PinSize: u32;

                // if ((Page + 1) * PAGE_SIZE > DcbSize) { PinSize = DcbSize - (Page * PAGE_SIZE); }
                // else { PinSize = PAGE_SIZE; }
                if (Page + 1) * PAGE_SIZE > DcbSize {
                    PinSize = DcbSize - (Page * PAGE_SIZE);
                } else {
                    PinSize = PAGE_SIZE;
                }

                // FatPrepareWriteDirectoryFile( IrpContext, Dcb, Page * PAGE_SIZE, PinSize,
                //                               &Bcbs[Page],
                //                               (PVOID *)&Dirent,
                //                               FALSE, TRUE, &DontCare );
                FatPrepareWriteDirectoryFile(
                    (IrpContext).cast(),
                    Dcb,
                    Page * PAGE_SIZE,
                    PinSize,
                    Bcbs.offset(Page as isize),
                    core::ptr::addr_of_mut!(Dirent),
                    0,
                    1,
                    core::ptr::addr_of_mut!(DontCare),
                );

                // if (Page == 0) { Directory = (PUCHAR)Dirent; }
                if Page == 0 {
                    Directory = Dirent as *mut u8;
                }

                Page += 1;
            }

            // TotalRuns = FsRtlNumberOfRunsInLargeMcb( &Mcb );
            TotalRuns = FsRtlNumberOfRunsInLargeMcb(core::ptr::addr_of_mut!(Mcb));

            // for (Run = 0; Run < TotalRuns; Run++) { ... }
            Run = 0;
            while Run < TotalRuns {
                let mut Vbo: i64 = 0;
                let mut Lbo: i64 = 0;

                // Result = FsRtlGetNextLargeMcbEntry( &Mcb, Run, &Vbo, &Lbo, (PLONGLONG)&BytesUsed );
                Result = FsRtlGetNextLargeMcbEntry(
                    core::ptr::addr_of_mut!(Mcb),
                    Run,
                    core::ptr::addr_of_mut!(Vbo),
                    core::ptr::addr_of_mut!(Lbo),
                    core::ptr::addr_of_mut!(BytesUsed) as *mut i64,
                );

                // NT_ASSERT(Result);
                NT_ASSERT((Result) as i32);

                // if (Lbo != -1) {
                if Lbo != -1 {
                    // RtlCopyMemory( UsedDirents, Directory + Vbo, (ULONG) BytesUsed );
                    RtlCopyMemory(
                        UsedDirents as *mut c_void,
                        Directory.offset(Vbo as isize) as *const c_void,
                        (BytesUsed as u32) as usize,
                    );
                    // UsedDirents += BytesUsed;
                    UsedDirents = UsedDirents.offset(BytesUsed as isize);
                } else {
                    // RtlCopyMemory( UnusedDirents, Directory + Vbo, (ULONG) BytesUsed );
                    RtlCopyMemory(
                        UnusedDirents as *mut c_void,
                        Directory.offset(Vbo as isize) as *const c_void,
                        (BytesUsed as u32) as usize,
                    );
                    // UnusedDirents += BytesUsed;
                    UnusedDirents = UnusedDirents.offset(BytesUsed as isize);
                }

                Run += 1;
            }

            // for (Char = UnusedDirentBuffer; Char < UnusedDirents; Char += sizeof(DIRENT)) {
            //     *Char = FAT_DIRENT_DELETED;
            // }
            Char = UnusedDirentBuffer;
            while Char < UnusedDirents {
                *Char = FAT_DIRENT_DELETED;
                Char = Char.offset(core::mem::size_of::<DIRENT>() as isize);
            }

            // NT_ASSERT( TotalBytesAllocated == (ULONG)(UsedDirents - UsedDirentBuffer) );
            NT_ASSERT((TotalBytesAllocated == (UsedDirents.offset_from(UsedDirentBuffer) as u32)) as i32);

            // RtlCopyMemory( Directory, UsedDirentBuffer, TotalBytesAllocated );
            RtlCopyMemory(
                Directory as *mut c_void,
                UsedDirentBuffer as *const c_void,
                (TotalBytesAllocated) as usize,
            );

            // RtlCopyMemory( Directory + TotalBytesAllocated,
            //                UnusedDirentBuffer,
            //                UnusedDirents - UnusedDirentBuffer );
            RtlCopyMemory(
                Directory.offset(TotalBytesAllocated as isize) as *mut c_void,
                UnusedDirentBuffer as *const c_void,
                ((UnusedDirents.offset_from(UnusedDirentBuffer)) as u32) as usize,
            );

            // if (Bcbs) { for (Page = 0; Page < PagesPinned; Page += 1) { FatUnpinBcb( IrpContext, Bcbs[Page] ); }
            //            ExFreePool(Bcbs); Bcbs = NULL; }
            if !Bcbs.is_null() {
                Page = 0;
                while Page < PagesPinned {
                    FatUnpinBcb(IrpContext, core::ptr::addr_of_mut!(*(Bcbs.offset(Page as isize))) as *mut *mut core::ffi::c_void);
                    Page += 1;
                }
                ExFreePool(Bcbs as *mut c_void);
                Bcbs = core::ptr::null_mut();
            }

            // RtlSetBits( &Dcb->Specific.Dcb.FreeDirentBitmap, 0,
            //             TotalBytesAllocated / sizeof(DIRENT) );
            RtlSetBits(
                core::ptr::addr_of_mut!(((*Dcb).Specific.Dcb.as_mut()).FreeDirentBitmap) as *mut _,
                0,
                TotalBytesAllocated / (core::mem::size_of::<DIRENT>() as u32),
            );

            // RtlClearBits( &Dcb->Specific.Dcb.FreeDirentBitmap,
            //               TotalBytesAllocated / sizeof(DIRENT),
            //               (DcbSize - TotalBytesAllocated) / sizeof(DIRENT) );
            RtlClearBits(
                core::ptr::addr_of_mut!(((*Dcb).Specific.Dcb.as_mut()).FreeDirentBitmap) as *mut _,
                TotalBytesAllocated / (core::mem::size_of::<DIRENT>() as u32),
                (DcbSize - TotalBytesAllocated) / (core::mem::size_of::<DIRENT>() as u32),
            );

            // ReturnValue = TotalBytesAllocated / sizeof(DIRENT);
            ReturnValue = TotalBytesAllocated / (core::mem::size_of::<DIRENT>() as u32);

            // Flush the directory to disk.  If we raise, we will need to invalidate
            // all of the children.
            if crate::seh::try_except(
                &mut (),
                |_| FatUnpinRepinnedBcbs(IrpContext),
                |_, pointers| {
                    if FsRtlIsNtstatusExpected(crate::seh::exception_code(pointers) as i32) != 0 {
                        crate::seh::EXCEPTION_EXECUTE_HANDLER
                    } else {
                        crate::seh::EXCEPTION_CONTINUE_SEARCH
                    }
                },
            )
            .is_err()
            {
                InvalidateFcbs = 1;
            }

            // for (Links = Dcb->Specific.Dcb.ParentDcbQueue.Flink;
            //      Links != &Dcb->Specific.Dcb.ParentDcbQueue;
            //      Links = Links->Flink) {
            Links = (*Dcb).Specific.Dcb.as_mut().ParentDcbQueue.Flink;
            while Links
                != core::ptr::addr_of_mut!(((*Dcb).Specific.Dcb.as_mut()).ParentDcbQueue)
                    as *mut LIST_ENTRY
            {
                let mut TmpBcb: *mut BCB = core::ptr::null_mut();
                let mut TmpOffset: u32 = 0;
                let mut TmpDirent: *mut DIRENT = core::ptr::null_mut();
                let mut PreviousLfnSpread: u32;

                // Fcb = CONTAINING_RECORD( Links, FCB, ParentDcbLinks );
                Fcb = (Links as *mut u8)
                    .offset(-(core::mem::offset_of!(FCB, ParentDcbLinks) as isize))
                    as *mut FCB;

                // if (IsFileDeleted( IrpContext, Fcb )) { continue; }
                if IsFileDeleted(IrpContext, Fcb) != 0 {
                    Links = (*Links).Flink;
                    continue;
                }

                // if (!InvalidateFcbs) {
                if InvalidateFcbs == 0 {
                    // Safely try to pick up the dirent; if this raises, give up
                    // and blow everyone else away too.
                    if crate::seh::try_except(
                        &mut (),
                        |_| {
                            FatLocateSimpleOemDirent(
                                IrpContext,
                                Dcb,
                                (*Fcb).ShortName.Name.Oem.as_ref() as *const OEM_STRING,
                                core::ptr::addr_of_mut!(TmpDirent),
                                core::ptr::addr_of_mut!(TmpBcb),
                                core::ptr::addr_of_mut!(TmpOffset) as *mut u32,
                            );
                        },
                        |_, pointers| {
                            if FsRtlIsNtstatusExpected(crate::seh::exception_code(pointers) as i32) != 0 {
                                crate::seh::EXCEPTION_EXECUTE_HANDLER
                            } else {
                                crate::seh::EXCEPTION_CONTINUE_SEARCH
                            }
                        },
                    )
                    .is_err()
                    {
                        InvalidateFcbs = 1;
                    }
                }

                // if (TmpBcb == NULL || InvalidateFcbs) {
                if TmpBcb == core::ptr::null_mut() || InvalidateFcbs != 0 {
                    // FatUnpinBcb( IrpContext, TmpBcb );
                    FatUnpinBcb(IrpContext, core::ptr::addr_of_mut!(TmpBcb) as *mut *mut core::ffi::c_void);
                    // FatMarkFcbCondition( IrpContext, Fcb, FcbBad, TRUE );
                    FatMarkFcbCondition(IrpContext, Fcb, FcbBad, 1);
                } else {
                    // FatUnpinBcb( IrpContext, TmpBcb );
                    FatUnpinBcb(IrpContext, core::ptr::addr_of_mut!(TmpBcb) as *mut *mut core::ffi::c_void);

                    // PreviousLfnSpread = Fcb->DirentOffsetWithinDirectory - Fcb->LfnOffsetWithinDirectory;
                    PreviousLfnSpread =
                        (*Fcb).DirentOffsetWithinDirectory - (*Fcb).LfnOffsetWithinDirectory;

                    // Fcb->DirentOffsetWithinDirectory = TmpOffset;
                    (*Fcb).DirentOffsetWithinDirectory = TmpOffset;

                    // Fcb->LfnOffsetWithinDirectory = TmpOffset - PreviousLfnSpread;
                    (*Fcb).LfnOffsetWithinDirectory = TmpOffset - PreviousLfnSpread;
                }

                Links = (*Links).Flink;
            }

        },
        |_, _| {
            // Free all our resources and stuff.
            if *p_McbInitialized != 0 {
                FsRtlUninitializeLargeMcb(p_Mcb);
            }

            if !(*p_Lfn).Buffer.is_null() {
                ExFreePool((*p_Lfn).Buffer as *mut c_void);
            }

            if !(*p_UnusedDirentBuffer).is_null() {
                ExFreePool(*p_UnusedDirentBuffer as *mut c_void);
            }

            if !(*p_UsedDirentBuffer).is_null() {
                ExFreePool(*p_UsedDirentBuffer as *mut c_void);
            }

            if !(*p_Bcbs).is_null() {
                let mut page: u32 = 0;
                while page < *p_PagesPinned {
                    FatUnpinBcb(IrpContext, (*p_Bcbs).offset(page as isize) as *mut *mut core::ffi::c_void);
                    page += 1;
                }
                ExFreePool(*p_Bcbs as *mut c_void);
            }

            FatUnpinBcb(IrpContext, p_bcb as *mut *mut core::ffi::c_void);

            // Release Fcb resources
            let mut links = (*Dcb).Specific.Dcb.as_mut().ParentDcbQueue.Flink;
            while links
                != core::ptr::addr_of_mut!(((*Dcb).Specific.Dcb.as_mut()).ParentDcbQueue)
                    as *mut LIST_ENTRY
            {
                let fcb = (links as *mut u8)
                    .offset(-(core::mem::offset_of!(FCB, ParentDcbLinks) as isize))
                    as *mut FCB;
                ExReleaseResourceLite((*fcb).Header.Resource);
                links = (*links).Flink;
            }

            // Restore IrpContext->Flags
            (*IrpContext).Flags = SavedIrpContextFlag;
        },
    );

    // Now return the offset of the first free dirent to the caller.  (The old
    // emulation returned 0, not -1, on the try_return path.)
    return ReturnValue;
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
//     DirSup.c
//
// Abstract:
//
//     This module implements the dirent support routines for Fat.
//
//
// --*/
//
// #include "fatprocs.h"
//
// //
// //  The Bug check file id for this module
// //
//
// #define BugCheckFileId                   (FAT_BUG_CHECK_DIRSUP)
//
// //
// //  Local debug trace level
// //
//
// #define Dbg                              (DEBUG_TRACE_DIRSUP)
//
// //
// //  The following three macro all assume the input dirent has been zeroed.
// //
//
// //
// //  VOID
// //  FatConstructDot (
// //      IN PIRP_CONTEXT IrpContext,
// //      IN PDCB Directory,
// //      IN PDIRENT ParentDirent,
// //      IN OUT PDIRENT Dirent
// //      );
// //
// //  The following macro is called to initalize the "." dirent.
// //
// //  Always setting FirstClusterOfFileHi is OK because it will be zero
// //  unless we're working on a FAT 32 disk.
// //
//
// #define FatConstructDot(IRPCONTEXT,DCB,PARENT,DIRENT) {                  \
//                                                                          \
//     RtlCopyMemory( (PUCHAR)(DIRENT), ".          ", 11 );                \
//     (DIRENT)->Attributes = FAT_DIRENT_ATTR_DIRECTORY;                    \
//     (DIRENT)->LastWriteTime = (PARENT)->LastWriteTime;                   \
//     if (FatData.ChicagoMode()) {                                           \
//         (DIRENT)->CreationTime = (PARENT)->CreationTime;                 \
//         (DIRENT)->CreationMSec = (PARENT)->CreationMSec;                 \
//         (DIRENT)->LastAccessDate = (PARENT)->LastAccessDate;             \
//     }                                                                    \
//     (DIRENT)->FirstClusterOfFile =                                       \
//             (USHORT)(DCB)->FirstClusterOfFile;                           \
//     (DIRENT)->FirstClusterOfFileHi =                                     \
//             (USHORT)((DCB)->FirstClusterOfFile/0x10000);                 \
// }
//
// //
// //  VOID
// //  FatConstructDotDot (
// //      IN PIRP_CONTEXT IrpContext,
// //      IN PDCB Directory,
// //      IN PDIRENT ParentDirent,
// //      IN OUT PDIRENT Dirent
// //      );
// //
// //  The following macro is called to initalize the ".." dirent.
// //
// //  Always setting FirstClusterOfFileHi is OK because it will be zero
// //  unless we're working on a FAT 32 disk.
// //
//
// #define FatConstructDotDot(IRPCONTEXT,DCB,PARENT,DIRENT) {   \
//                                                              \
//     RtlCopyMemory( (PUCHAR)(DIRENT), "..         ", 11 );    \
//     (DIRENT)->Attributes = FAT_DIRENT_ATTR_DIRECTORY;        \
//     (DIRENT)->LastWriteTime = (PARENT)->LastWriteTime;       \
//     if (FatData.ChicagoMode()) {                               \
//         (DIRENT)->CreationTime = (PARENT)->CreationTime;     \
//         (DIRENT)->CreationMSec = (PARENT)->CreationMSec;     \
//         (DIRENT)->LastAccessDate = (PARENT)->LastAccessDate; \
//     }                                                        \
//     if (NodeType((DCB)->ParentDcb) == FAT_NTC_ROOT_DCB) {    \
//         (DIRENT)->FirstClusterOfFile = 0;                    \
//         (DIRENT)->FirstClusterOfFileHi = 0;                  \
//     } else {                                                 \
//         (DIRENT)->FirstClusterOfFile = (USHORT)              \
//             ((DCB)->ParentDcb->FirstClusterOfFile);          \
//         (DIRENT)->FirstClusterOfFileHi = (USHORT)            \
//             ((DCB)->ParentDcb->FirstClusterOfFile/0x10000);  \
//     }                                                        \
// }
//
// //
// //  VOID
// //  FatConstructEndDirent (
// //      IN PIRP_CONTEXT IrpContext,
// //      IN OUT PDIRENT Dirent
// //      );
// //
// //  The following macro created the end dirent.  Note that since the
// //  dirent was zeroed, the first byte of the name already contains 0x0,
// //  so there is nothing to do.
// //
//
// #define FatConstructEndDirent(IRPCONTEXT,DIRENT) NOTHING
//
// //
// //  VOID
// //  FatReadDirent (
// //      IN PIRP_CONTEXT IrpContext,
// //      IN PDCB Dcb,
// //      IN VBO Vbo,
// //      OUT PBCB *Bcb,
// //      OUT PVOID *Dirent,
// //      OUT PNTSTATUS Status
// //      );
// //
//
// //
// //  This macro reads in a page of dirents when we step onto a new page,
// //  or this is the first iteration of a loop and Bcb is NULL.
// //
//
// #define FatReadDirent(IRPCONTEXT,DCB,VBO,BCB,DIRENT,STATUS)       \
// if ((VBO) >= (DCB)->Header.AllocationSize.LowPart) {              \
//     *(STATUS) = STATUS_END_OF_FILE;                               \
//     FatUnpinBcb( (IRPCONTEXT), *(BCB) );                          \
// } else if ( ((VBO) % PAGE_SIZE == 0) || (*(BCB) == NULL) ) {      \
//     FatUnpinBcb( (IRPCONTEXT), *(BCB) );                          \
//     FatReadDirectoryFile( (IRPCONTEXT),                           \
//                           (DCB),                                  \
//                           (VBO) & ~(PAGE_SIZE - 1),               \
//                           PAGE_SIZE,                              \
//                           FALSE,                                  \
//                           (BCB),                                  \
//                           (PVOID *)(DIRENT),                      \
//                           (STATUS) );                             \
//     *(DIRENT) = (PVOID)((PUCHAR)*(DIRENT) + ((VBO) % PAGE_SIZE)); \
// }
//
// //
// //  Internal support routines
// //
//
// UCHAR
// FatComputeLfnChecksum (
//     PDIRENT Dirent
//     );
//
// _Requires_lock_held_(_Global_critical_region_)
// VOID
// FatRescanDirectory (
//     PIRP_CONTEXT IrpContext,
//     PDCB Dcb
//     );
//
// _Requires_lock_held_(_Global_critical_region_)
// ULONG
// FatDefragDirectory (
//     IN PIRP_CONTEXT IrpContext,
//     IN PDCB Dcb,
//     IN ULONG DirentsNeeded
//     );
//
//
// #ifdef ALLOC_PRAGMA
// #pragma alloc_text(PAGE, FatComputeLfnChecksum)
// #pragma alloc_text(PAGE, FatConstructDirent)
// #pragma alloc_text(PAGE, FatConstructLabelDirent)
// #pragma alloc_text(PAGE, FatCreateNewDirent)
// #pragma alloc_text(PAGE, FatDefragDirectory)
// #pragma alloc_text(PAGE, FatDeleteDirent)
// #pragma alloc_text(PAGE, FatGetDirentFromFcbOrDcb)
// #pragma alloc_text(PAGE, FatInitializeDirectoryDirent)
// #pragma alloc_text(PAGE, FatIsDirectoryEmpty)
// #pragma alloc_text(PAGE, FatLfnDirentExists)
// #pragma alloc_text(PAGE, FatLocateDirent)
// #pragma alloc_text(PAGE, FatLocateSimpleOemDirent)
// #pragma alloc_text(PAGE, FatLocateVolumeLabel)
// #pragma alloc_text(PAGE, FatRescanDirectory)
// #pragma alloc_text(PAGE, FatSetFileSizeInDirent)
// #pragma alloc_text(PAGE, FatSetFileSizeInDirentNoRaise)
// #pragma alloc_text(PAGE, FatTunnelFcbOrDcb)
// #pragma alloc_text(PAGE, FatUpdateDirentFromFcb)
//
//
// #endif
//
//
// _Requires_lock_held_(_Global_critical_region_)
// ULONG
// FatCreateNewDirent (
//     IN PIRP_CONTEXT IrpContext,
//     IN PDCB ParentDirectory,
//     IN ULONG DirentsNeeded,
//     IN BOOLEAN RescanDir
//     )
//
// /*++
//
// Routine Description:
//
//     This routine allocates on the disk a new dirent inside of the
//     parent directory.  If a new dirent cannot be allocated (i.e.,
//     because the disk is full or the root directory is full) then
//     it raises the appropriate status.  The dirent itself is
//     neither initialized nor pinned by this procedure.
//
// Arguments:
//
//     ParentDirectory - Supplies the DCB for the directory in which
//         to create the new dirent
//
//     DirentsNeeded - This is the number of continginous dirents required
//
// Return Value:
//
//     ByteOffset - Returns the VBO within the Parent directory where
//         the dirent has been allocated
//
// --*/
//
// {
//     VBO UnusedVbo;
//     VBO DeletedHint;
//     ULONG ByteOffset;
//
//     PBCB Bcb = NULL;
//     PDIRENT Dirent = NULL;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     PAGED_CODE();
//
//     DebugTrace(+1, Dbg, "FatCreateNewDirent\n", 0);
//
//     DebugTrace( 0, Dbg, "  ParentDirectory = %p\n", ParentDirectory);
//
//     //
//     //  If UnusedDirentVbo is within our current file allocation then we
//     //  don't have to search through the directory at all; we know just
//     //  where to put it.
//     //
//     //  If UnusedDirentVbo is beyond the current file allocation then
//     //  there are no more unused dirents in the current allocation, though
//     //  upon adding another cluster of allocation UnusedDirentVbo
//     //  will point to an unused dirent.  Haveing found no unused dirents
//     //  we use the DeletedDirentHint to try and find a deleted dirent in
//     //  the current allocation.  In this also runs off the end of the file,
//     //  we finally have to break down and allocate another sector.  Note
//     //  that simply writing beyond the current allocation will automatically
//     //  do just this.
//     //
//     //  We also must deal with the special case where UnusedDirentVbo and
//     //  DeletedDirentHint have yet to be initialized.  In this case we must
//     //  first walk through the directory looking for the first deleted entry
//     //  first unused dirent.  After this point we continue as before.
//     //  This initial state is denoted by the special value of 0xffffffff.
//     //
//
//     UnusedVbo = ParentDirectory->Specific.Dcb.UnusedDirentVbo;
//     DeletedHint = ParentDirectory->Specific.Dcb.DeletedDirentHint;
//
//     //
//     //  Check for our first call to this routine with this Dcb.  If so
//     //  we have to correctly set the two hints in the Dcb.
//     //
//
//     if (UnusedVbo == 0xffffffff_u32 || RescanDir) {
//
//         FatRescanDirectory( IrpContext, ParentDirectory );
//
//         UnusedVbo = ParentDirectory->Specific.Dcb.UnusedDirentVbo;
//         DeletedHint = ParentDirectory->Specific.Dcb.DeletedDirentHint;
//     }
//
//     //
//     //  Now we know that UnusedDirentVbo and DeletedDirentHint are correctly
//     //  set so we check if there is already an unused dirent in the the
//     //  current allocation.  This is the easy case.
//     //
//
//     DebugTrace( 0, Dbg, "  UnusedVbo   = %08lx\n", UnusedVbo);
//     DebugTrace( 0, Dbg, "  DeletedHint = %08lx\n", DeletedHint);
//
//     if (!RescanDir && ( UnusedVbo + (DirentsNeeded * sizeof(DIRENT)) <=
//          ParentDirectory->Header.AllocationSize.LowPart )) {
//
//         //
//         //  Get this unused dirent for the caller.  We have a
//         //  sporting chance that we won't have to wait.
//         //
//
//         DebugTrace( 0, Dbg, "There is a never used entry.\n", 0);
//
//         ByteOffset = UnusedVbo;
//
//         UnusedVbo += DirentsNeeded * sizeof(DIRENT);
//
//     } else {
//
//         //
//         //  Life is tough.  We have to march from the DeletedDirentHint
//         //  looking for a deleted dirent.  If we get to EOF without finding
//         //  one, we will have to allocate a new cluster.
//         //
//
//         ByteOffset =
//             RtlFindClearBits( &ParentDirectory->Specific.Dcb.FreeDirentBitmap,
//                               DirentsNeeded,
//                               DeletedHint / sizeof(DIRENT) );
//
//         //
//         //  Do a quick check for a root directory allocation that failed
//         //  simply because of fragmentation.  Also, only attempt to defrag
//         //  if the length is less that 0x40000.  This is to avoid
//         //  complications arising from crossing a MM view boundary (256kb).
//         //  By default on DOS the root directory is only 0x2000 long.
//         //
//         //  Don't try to defrag fat32 root dirs.
//         //
//
//         if (!FatIsFat32(ParentDirectory->Vcb) &&
//             (ByteOffset == -1) &&
//             (NodeType(ParentDirectory) == FAT_NTC_ROOT_DCB) &&
//             (ParentDirectory->Header.AllocationSize.LowPart <= 0x40000)) {
//
//             ByteOffset = FatDefragDirectory( IrpContext, ParentDirectory, DirentsNeeded );
//         }
//
//         if (ByteOffset != -1) {
//
//             //
//             //  If we consuemed deleted dirents at Deleted Hint, update.
//             //  We also may have consumed some un-used dirents as well,
//             //  so be sure to check for that as well.
//             //
//
//             ByteOffset *= sizeof(DIRENT);
//
//             if (ByteOffset == DeletedHint) {
//
//                 DeletedHint += DirentsNeeded * sizeof(DIRENT);
//             }
//
//             if (ByteOffset + DirentsNeeded * sizeof(DIRENT) > UnusedVbo) {
//
//                 UnusedVbo = ByteOffset + DirentsNeeded * sizeof(DIRENT);
//             }
//
//         } else {
//
//             //
//             //  We are going to have to allocate another cluster.  Do
//             //  so, update both the UnusedVbo and the DeletedHint and bail.
//             //
//
//             DebugTrace( 0, Dbg, "We have to allocate another cluster.\n", 0);
//
//             //
//             //  A reason why we might fail, unrelated to physical reasons,
//             //  is that we constrain to 64k directory entries to match the
//             //  restriction on Win95.  There are fundamental reasons to do
//             //  this since searching a FAT directory is a linear operation
//             //  and to allow FAT32 to toss us over the cliff is not permissable.
//             //
//
//             if (ParentDirectory->Header.AllocationSize.LowPart >= (64 * 1024 * sizeof(DIRENT)) ||
//
//                 //
//                 //  Make sure we are not trying to expand the root directory on non
//                 //  FAT32.  FAT16 and FAT12 have fixed size allocations.
//                 //
//
//                 (!FatIsFat32(ParentDirectory->Vcb) &&
//                  NodeType(ParentDirectory) == FAT_NTC_ROOT_DCB)) {
//
//                 DebugTrace(0, Dbg, "Full root directory or too big on FAT32.  Raise Status.\n", 0);
//
//                 FatRaiseStatus( IrpContext, STATUS_CANNOT_MAKE );
//             }
//
//             //
//             //  Take the last dirent(s) in this cluster.  We will allocate
//             //  more clusters below.
//             //
//
//             ByteOffset = UnusedVbo;
//             UnusedVbo += DirentsNeeded * sizeof(DIRENT);
//
//             //
//             //  Touch the directory file to cause space for the new dirents
//             //  to be allocated.
//             //
//
//             Bcb = NULL;
//
//             _SEH2_TRY {
//
//                 PVOID Buffer;
//
//                 FatPrepareWriteDirectoryFile( IrpContext,
//                                               ParentDirectory,
//                                               UnusedVbo,
//                                               1,
//                                               &Bcb,
//                                               &Buffer,
//                                               FALSE,
//                                               TRUE,
//                                               &Status );
//
//             } _SEH2_FINALLY {
//
//                 FatUnpinBcb( IrpContext, Bcb );
//             } _SEH2_END;
//         }
//     }
//
//     //
//     //  If we are only requesting a single dirent, and we did not get the
//     //  first dirent in a directory, then check that the preceding dirent
//     //  is not an orphaned LFN.  If it is, then mark it deleted.  Thus
//     //  reducing the possibility of an accidental pairing.
//     //
//     //  Only do this when we are in Chicago Mode.
//     //
//
//     Bcb = NULL;
//
//     if (FatData.ChicagoMode() &&
//         (DirentsNeeded == 1) &&
//         (ByteOffset > (NodeType(ParentDirectory) == FAT_NTC_ROOT_DCB ?
//                        0 : 2 * sizeof(DIRENT)))) {
//         _SEH2_TRY {
//
//             FatReadDirent( IrpContext,
//                            ParentDirectory,
//                            ByteOffset - sizeof(DIRENT),
//                            &Bcb,
//                            &Dirent,
//                            &Status );
//
//             if ((Status != STATUS_SUCCESS) ||
//                 (Dirent->FileName[0] == FAT_DIRENT_NEVER_USED)) {
//
//                 FatPopUpFileCorrupt( IrpContext, ParentDirectory );
//
//                 FatRaiseStatus( IrpContext, STATUS_FILE_CORRUPT_ERROR );
//             }
//
//             if ((Dirent->Attributes == FAT_DIRENT_ATTR_LFN) &&
//                 (Dirent->FileName[0] != FAT_DIRENT_DELETED)) {
//
//                 //
//                 //  Pin it, mark it, and set it dirty.
//                 //
//
//                 FatPinMappedData( IrpContext,
//                                   ParentDirectory,
//                                   ByteOffset - sizeof(DIRENT),
//                                   sizeof(DIRENT),
//                                   &Bcb );
//
//                 Dirent->FileName[0] = FAT_DIRENT_DELETED;
//
//                 FatSetDirtyBcb( IrpContext, Bcb, ParentDirectory->Vcb, TRUE );
//
//                 NT_ASSERT( RtlAreBitsSet( &ParentDirectory->Specific.Dcb.FreeDirentBitmap,
//                                        (ByteOffset - sizeof(DIRENT))/ sizeof(DIRENT),
//                                        DirentsNeeded ) );
//
//                 RtlClearBits( &ParentDirectory->Specific.Dcb.FreeDirentBitmap,
//                               (ByteOffset - sizeof(DIRENT))/ sizeof(DIRENT),
//                               DirentsNeeded );
//
//             }
//
//         } _SEH2_FINALLY {
//
//             FatUnpinBcb( IrpContext, Bcb );
//         } _SEH2_END;
//     }
//
//     //
//     //  Assert that the dirents are in fact unused
//     //
//
//     _SEH2_TRY {
//
//         ULONG i;
//
//         Bcb = NULL;
//
//         for (i = 0; i < DirentsNeeded; i++) {
//
//             FatReadDirent( IrpContext,
//                            ParentDirectory,
//                            ByteOffset + i*sizeof(DIRENT),
//                            &Bcb,
//                            &Dirent,
//                            &Status );
//
//             if ((Status != STATUS_SUCCESS) ||
//                 ((Dirent->FileName[0] != FAT_DIRENT_NEVER_USED) &&
//                  (Dirent->FileName[0] != FAT_DIRENT_DELETED))) {
//
//                 FatPopUpFileCorrupt( IrpContext, ParentDirectory );
//                 FatRaiseStatus( IrpContext, STATUS_FILE_CORRUPT_ERROR );
//             }
//         }
//
//     } _SEH2_FINALLY {
//
//         FatUnpinBcb( IrpContext, Bcb );
//     } _SEH2_END;
//
//     //
//     //  Set the Bits in the bitmap and move the Unused Dirent Vbo.
//     //
//
//     NT_ASSERT( RtlAreBitsClear( &ParentDirectory->Specific.Dcb.FreeDirentBitmap,
//                              ByteOffset / sizeof(DIRENT),
//                              DirentsNeeded ) );
//
//     RtlSetBits( &ParentDirectory->Specific.Dcb.FreeDirentBitmap,
//                 ByteOffset / sizeof(DIRENT),
//                 DirentsNeeded );
//
//     //
//     //  Save the newly computed values in the Parent Directory Fcb
//     //
//
//     ParentDirectory->Specific.Dcb.UnusedDirentVbo = UnusedVbo;
//     ParentDirectory->Specific.Dcb.DeletedDirentHint = DeletedHint;
//
//     DebugTrace(-1, Dbg, "FatCreateNewDirent -> (VOID)\n", 0);
//
//     return ByteOffset;
// }
//
//
//
// _Requires_lock_held_(_Global_critical_region_)
// VOID
// FatInitializeDirectoryDirent (
//     IN PIRP_CONTEXT IrpContext,
//     IN PDCB Dcb,
//     IN PDIRENT ParentDirent
//     )
//
// /*++
//
// Routine Description:
//
//     This routine converts a dirent into a directory on the disk.  It does this
//     setting the directory flag in the dirent, and by allocating the necessary
//     space for the "." and ".." dirents and initializing them.
//
//     If a new dirent cannot be allocated (i.e., because the disk is full) then
//     it raises the appropriate status.
//
// Arguments:
//
//     Dcb - Supplies the Dcb denoting the file that is to be made into a
//         directory.  This must be input a completely empty file with
//         an allocation size of zero.
//
//     ParentDirent - Provides the parent Dirent for a time-stamp model.
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     PBCB Bcb;
//     PVOID Buffer;
//     NTSTATUS DontCare = STATUS_SUCCESS;
//
//     PAGED_CODE();
//
//     DebugTrace(+1, Dbg, "FatInitializeDirectoryDirent\n", 0);
//
//     DebugTrace( 0, Dbg, "  Dcb = %p\n", Dcb);
//
//     //
//     //  Assert that we are not attempting this on the root directory.
//     //
//
//     NT_ASSERT( NodeType(Dcb) != FAT_NTC_ROOT_DCB );
//
//     //
//     //  Assert that this is only attempted on newly created directories.
//     //
//
//     NT_ASSERT( Dcb->Header.AllocationSize.LowPart == 0 );
//
//     //
//     //  Prepare the directory file for writing.  Note that we can use a single
//     //  Bcb for these two entries because we know they are the first two in
//     //  the directory, and thus together do not span a page boundry.  Also
//     //  note that we prepare write 2 entries: one for "." and one for "..".
//     //  The end of directory marker is automatically set since the whole
//     //  directory is initially zero (DIRENT_NEVER_USED).
//     //
//
//     FatPrepareWriteDirectoryFile( IrpContext,
//                                   Dcb,
//                                   0,
//                                   2 * sizeof(DIRENT),
//                                   &Bcb,
//                                   &Buffer,
//                                   FALSE,
//                                   TRUE,
//                                   &DontCare );
//
//     NT_ASSERT( NT_SUCCESS( DontCare ));
//
//     //
//     //  Add the . and .. entries
//     //
//
//     _SEH2_TRY {
//
//         FatConstructDot( IrpContext, Dcb, ParentDirent, (PDIRENT)Buffer + 0);
//
//         FatConstructDotDot( IrpContext, Dcb, ParentDirent, (PDIRENT)Buffer + 1);
//
//     //
//     //  Unpin the buffer and return to the caller.
//     //
//
//     } _SEH2_FINALLY {
//
//         FatUnpinBcb( IrpContext, Bcb );
//     } _SEH2_END;
//
//     DebugTrace(-1, Dbg, "FatInitializeDirectoryDirent -> (VOID)\n", 0);
//     return;
// }
//
//
// VOID
// FatTunnelFcbOrDcb (
//     IN PFCB FcbOrDcb,
//     IN PCCB Ccb OPTIONAL
//     )
// /*++
//
// Routine Description:
//
//     This routine handles tunneling of an Fcb or Dcb associated with
//     an object whose name is disappearing from a directory.
//
// Arguments:
//
//     FcbOrDcb - Supplies the Fcb/Dcb whose name will be going away
//
//     Ccb - Supplies the Ccb for the Fcb (not reqired for a Dcb) so
//         that we know which name the Fcb was opened by
//
// Return Value:
//
//     None.
//
// --*/
// {
//     UNICODE_STRING ShortNameWithCase = {0};
//     UNICODE_STRING DownCaseSeg;
//     WCHAR ShortNameBuffer[8+1+3];
//     NTSTATUS Status;
//     USHORT i;
//
//     PAGED_CODE();
//
//     DebugTrace(+1, Dbg, "FatTunnelFcbOrDcb\n", 0);
//
//     if (NodeType(FcbOrDcb) == FAT_NTC_DCB) {
//
//         //
//         //  Directory deletion. Flush all entries from this directory in
//         //  the cache for this volume
//         //
//
//         FsRtlDeleteKeyFromTunnelCache( &FcbOrDcb->Vcb->Tunnel,
//                                        FatDirectoryKey(FcbOrDcb) );
//
//     } else {
//
//         //
//         //  Was a file, so throw it into the tunnel cache
//         //
//
//         //
//         //  Get the short name into UNICODE
//         //
//
//         ShortNameWithCase.Length = 0;
//         ShortNameWithCase.MaximumLength = sizeof(ShortNameBuffer);
//         ShortNameWithCase.Buffer = ShortNameBuffer;
//
// #ifdef _MSC_VER
// #pragma prefast( suppress:28931, "needed for debug build" )
// #endif
//         Status = RtlOemStringToCountedUnicodeString( &ShortNameWithCase,
//                                                      &FcbOrDcb->ShortName.Name.Oem,
//                                                      FALSE);
//
//         NT_ASSERT(ShortNameWithCase.Length != 0);
//
//         NT_ASSERT(NT_SUCCESS(Status));
//
//         if (FlagOn(FcbOrDcb->FcbState, FCB_STATE_8_LOWER_CASE | FCB_STATE_3_LOWER_CASE)) {
//
//             //
//             //  Have to repair the case of the short name
//             //
//
//             for (i = 0; i < (ShortNameWithCase.Length/sizeof(WCHAR)) &&
//                         ShortNameWithCase.Buffer[i] != L'.'; i++);
//
//             //
//             //  Now pointing at the '.', or otherwise the end of name component
//             //
//
//             if (FlagOn(FcbOrDcb->FcbState, FCB_STATE_8_LOWER_CASE)) {
//
//                 DownCaseSeg.Buffer = ShortNameWithCase.Buffer;
//                 DownCaseSeg.MaximumLength = DownCaseSeg.Length = i*sizeof(WCHAR);
//
//                 RtlDowncaseUnicodeString(&DownCaseSeg, &DownCaseSeg, FALSE);
//             }
//
//             i++;
//
//             //
//             //  Now pointing at first wchar of the extension.
//             //
//
//             if (FlagOn(FcbOrDcb->FcbState, FCB_STATE_3_LOWER_CASE)) {
//
//                 //
//                 //  It is not neccesarily the case that we can rely on the flag
//                 //  indicating that we really have an extension.
//                 //
//
//                 if ((i*sizeof(WCHAR)) < ShortNameWithCase.Length) {
//                     DownCaseSeg.Buffer = &ShortNameWithCase.Buffer[i];
//                     DownCaseSeg.MaximumLength = DownCaseSeg.Length = ShortNameWithCase.Length - i*sizeof(WCHAR);
//
//                     RtlDowncaseUnicodeString(&DownCaseSeg, &DownCaseSeg, FALSE);
//                 }
//             }
//         }
//
//         //
//         //  ... and add it in
//         //
//
//         FsRtlAddToTunnelCache( &FcbOrDcb->Vcb->Tunnel,
//                                FatDirectoryKey(FcbOrDcb->ParentDcb),
//                                &ShortNameWithCase,
//                                &FcbOrDcb->ExactCaseLongName,
//                                BooleanFlagOn(Ccb->Flags, CCB_FLAG_OPENED_BY_SHORTNAME),
//                                sizeof(LARGE_INTEGER),
//                                &FcbOrDcb->CreationTime );
//     }
//
//     DebugTrace(-1, Dbg, "FatTunnelFcbOrDcb -> (VOID)\n", 0);
//
//     return;
// }
//
//
//
// _Requires_lock_held_(_Global_critical_region_)
// VOID
// FatDeleteDirent (
//     IN PIRP_CONTEXT IrpContext,
//     IN PFCB FcbOrDcb,
//     IN PDELETE_CONTEXT DeleteContext OPTIONAL,
//     IN BOOLEAN DeleteEa
//     )
//
// /*++
//
// Routine Description:
//
//     This routine Deletes on the disk the indicated dirent.  It does
//     this by marking the dirent as deleted.
//
// Arguments:
//
//     FcbOrDcb - Supplies the FCB/DCB for the file/directory being
//         deleted.  For a file the file size and allocation must be zero.
//         (Zero allocation is implied by a zero cluster index).
//         For a directory the allocation must be zero.
//
//     DeleteContext - This variable, if speicified, may be used to preserve
//         the file size and first cluster of file information in the dirent
//         fot the benefit of unerase utilities.
//
//     DeleteEa - Tells us whether to delete the EA and whether to check
//         for no allocation/  Mainly TRUE.  FALSE passed in from rename.
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     PBCB Bcb = NULL;
//     PDIRENT Dirent = NULL;
//     NTSTATUS DontCare;
//     ULONG Offset;
//     ULONG DirentsToDelete;
//
//     PAGED_CODE();
//
//     DebugTrace(+1, Dbg, "FatDeleteDirent\n", 0);
//
//     DebugTrace( 0, Dbg, "  FcbOrDcb = %p\n", FcbOrDcb);
//
//     //
//     //  We must be holding the vcb exclusive here to deal with the locate dirent
//     //  cases where it cannot be holding the parent simply.  This is actually
//     //  a true statement from olden daze, lets just wire in our assertion.
//     //
//     //  Among other reasons, it'd be unfortunate if this raced with the
//     //  rename path.
//     //
//
//     NT_ASSERT( ExIsResourceAcquiredExclusiveLite( &FcbOrDcb->Vcb->Resource ));
//
//     //
//     //  Assert that we are not attempting this on the root directory.
//     //
//
//     NT_ASSERT( NodeType(FcbOrDcb) != FAT_NTC_ROOT_DCB );
//
//     //
//     //  Make sure all requests have zero allocation/file size
//     //
//
//     if (DeleteEa &&
//         ((FcbOrDcb->Header.AllocationSize.LowPart != 0) ||
//          ((NodeType(FcbOrDcb) == FAT_NTC_FCB) &&
//           (FcbOrDcb->Header.FileSize.LowPart != 0)))) {
//
//         DebugTrace( 0, Dbg, "Called with non zero allocation/file size.\n", 0);
//
// #ifdef _MSC_VER
// #pragma prefast( suppress:28159, "things are seriously wrong if we get here" )
// #endif
//         FatBugCheck( 0, 0, 0 );
//     }
//
//     //
//     //  Now, mark the dirents deleted, unpin the Bcb, and return to the caller.
//     //  Assert that there isn't any allocation associated with this dirent.
//     //
//     //  Note that this loop will end with Dirent pointing to the short name.
//     //
//
//     _SEH2_TRY {
//
//         //
//         //  We must acquire our parent exclusive to synchronize with enumerators
//         //  who do not hold the vcb (ex: dirctrl).
//         //
//         //  This relies on our bottom up lockorder.
//         //
//
//         ExAcquireResourceExclusiveLite( FcbOrDcb->ParentDcb->Header.Resource, TRUE );
//
//         for ( Offset = FcbOrDcb->LfnOffsetWithinDirectory;
//               Offset <= FcbOrDcb->DirentOffsetWithinDirectory;
//               Offset += sizeof(DIRENT), Dirent += 1 ) {
//
//             //
//             //  If we stepped onto a new page, or this is the first iteration,
//             //  unpin the old page, and pin the new one.
//             //
//
//             if ((Offset == FcbOrDcb->LfnOffsetWithinDirectory) ||
//                 ((Offset & (PAGE_SIZE - 1)) == 0)) {
//
//                 FatUnpinBcb( IrpContext, Bcb );
//
//                 FatPrepareWriteDirectoryFile( IrpContext,
//                                               FcbOrDcb->ParentDcb,
//                                               Offset,
//                                               sizeof(DIRENT),
//                                               &Bcb,
//                                               (PVOID *)&Dirent,
//                                               FALSE,
//                                               TRUE,
//                                               &DontCare );
//             }
//
//             NT_ASSERT( (Dirent->FirstClusterOfFile == 0) || !DeleteEa );
//             Dirent->FileName[0] = FAT_DIRENT_DELETED;
//         }
//
//         //
//         //  Back Dirent off by one to point back to the short dirent.
//         //
//
//         Dirent -= 1;
//
//         //
//         //  If there are extended attributes for this dirent, we will attempt
//         //  to remove them.  We ignore any errors in removing Eas.
//         //
//
//         if (!FatIsFat32(FcbOrDcb->Vcb) &&
//             DeleteEa && (Dirent->ExtendedAttributes != 0)) {
//
//             _SEH2_TRY {
//
//                 FatDeleteEa( IrpContext,
//                              FcbOrDcb->Vcb,
//                              Dirent->ExtendedAttributes,
//                              &FcbOrDcb->ShortName.Name.Oem );
//
//             } _SEH2_EXCEPT(FatExceptionFilter( IrpContext, _SEH2_GetExceptionInformation() )) {
//
//                 //
//                 //  We catch all exceptions that Fat catches, but don't do
//                 //  anything with them.
//                 //
//             } _SEH2_END;
//         }
//
//         //
//         //  Now clear the bits in the free dirent mask.
//         //
//
//         DirentsToDelete = (FcbOrDcb->DirentOffsetWithinDirectory -
//                            FcbOrDcb->LfnOffsetWithinDirectory) / sizeof(DIRENT) + 1;
//
//
//         NT_ASSERT( (FcbOrDcb->ParentDcb->Specific.Dcb.UnusedDirentVbo == 0xffffffff_u32) ||
//                 RtlAreBitsSet( &FcbOrDcb->ParentDcb->Specific.Dcb.FreeDirentBitmap,
//                                FcbOrDcb->LfnOffsetWithinDirectory / sizeof(DIRENT),
//                                DirentsToDelete ) );
//
//         RtlClearBits( &FcbOrDcb->ParentDcb->Specific.Dcb.FreeDirentBitmap,
//                       FcbOrDcb->LfnOffsetWithinDirectory / sizeof(DIRENT),
//                       DirentsToDelete );
//
//         //
//         //  Now, if the caller specified a DeleteContext, use it.
//         //
//
//         if ( ARGUMENT_PRESENT( DeleteContext ) ) {
//
//             Dirent->FileSize = DeleteContext->FileSize;
//
//
//             Dirent->FirstClusterOfFile = (USHORT)DeleteContext->FirstClusterOfFile;
//         }
//
//         //
//         //  If this newly deleted dirent is before the DeletedDirentHint, change
//         //  the DeletedDirentHint to point here.
//         //
//
//         if (FcbOrDcb->DirentOffsetWithinDirectory <
//                             FcbOrDcb->ParentDcb->Specific.Dcb.DeletedDirentHint) {
//
//             FcbOrDcb->ParentDcb->Specific.Dcb.DeletedDirentHint =
//                                             FcbOrDcb->LfnOffsetWithinDirectory;
//         }
//
//     } _SEH2_FINALLY {
//
//         FatUnpinBcb( IrpContext, Bcb );
//
//         //
//         //  Release our parent.
//         //
//
//         ExReleaseResourceLite( FcbOrDcb->ParentDcb->Header.Resource );
//     } _SEH2_END;
//
//     DebugTrace(-1, Dbg, "FatDeleteDirent -> (VOID)\n", 0);
//     return;
// }
//
// _Requires_lock_held_(_Global_critical_region_)
// BOOLEAN
// FatLfnDirentExists (
//     IN PIRP_CONTEXT IrpContext,
//     IN PDCB Dcb,
//     IN PUNICODE_STRING Lfn,
//     IN PUNICODE_STRING LfnTmp
//     )
// /*++
//
// Routine Description:
//
//     This routine looks for a given Lfn in a directory
//
// Arguments:
//
//     Dcb - The directory to search
//
//     Lfn - The Lfn to look for
//
//     Lfn - Temporary buffer to use to search for Lfn with (if < MAX_LFN then this
//         function may cause it to be allocated from pool if not large enough.
//
// Retrn Value:
//
//     BOOLEAN TRUE if it exists, FALSE if not
//
// --*/
// {
//     CCB Ccb;
//     PDIRENT Dirent;
//     PBCB DirentBcb = NULL;
//     VBO DirentByteOffset;
//     BOOLEAN Result = FALSE;
//     ULONG Flags = 0;
//
//     PAGED_CODE();
//
//     //
//     //  Pay performance penalty by forcing the compares to be case insensitive as
//     //  opposed to grabbing more pool for a monocased copy of the Lfn. This is slight.
//     //
//
//     Ccb.UnicodeQueryTemplate =  *Lfn;
//     Ccb.ContainsWildCards = FALSE;
//     Ccb.Flags = CCB_FLAG_SKIP_SHORT_NAME_COMPARE | CCB_FLAG_QUERY_TEMPLATE_MIXED;
//
//     _SEH2_TRY {
//
//         FatLocateDirent( IrpContext,
//                          Dcb,
//                          &Ccb,
//                          0,
//                          &Flags,
//                          &Dirent,
//                          &DirentBcb,
//                          &DirentByteOffset,
//                          NULL,
//                          LfnTmp,
//                          NULL );
//
//     } _SEH2_FINALLY {
//
//         if (DirentBcb) {
//
//             Result = TRUE;
//         }
//
//         FatUnpinBcb(IrpContext, DirentBcb);
//     } _SEH2_END;
//
//     return Result;
// }
//
//
// _Requires_lock_held_(_Global_critical_region_)
// VOID
// FatLocateDirent (
//     IN PIRP_CONTEXT IrpContext,
//     IN PDCB ParentDirectory,
//     IN PCCB Ccb,
//     IN VBO OffsetToStartSearchFrom,
//     IN OUT PULONG Flags,
//     OUT PDIRENT *Dirent,
//     OUT PBCB *Bcb,
//     OUT PVBO ByteOffset,
//     OUT PBOOLEAN FileNameDos OPTIONAL,
//     IN OUT PUNICODE_STRING LongFileName OPTIONAL,
//     IN OUT PUNICODE_STRING OrigLongFileName OPTIONAL
//     )
//
// /*++
//
// Routine Description:
//
//     This routine locates on the disk an undeleted dirent matching a given name.
//
// Arguments:
//
//     ParentDirectory - Supplies the DCB for the directory to search
//
//     Ccb - Contains a context control block with all matching information.
//
//     OffsetToStartSearchFrom - Supplies the VBO within the parent directory
//         from which to start looking for another real dirent.
//
//     Dirent - Receives a pointer to the located dirent if one was found
//         or NULL otherwise.
//
//     Bcb - Receives the Bcb for the located dirent if one was found or
//         NULL otherwise.
//
//     ByteOffset - Receives the VBO within the Parent directory for
//         the located dirent if one was found, or 0 otherwise.
//
//     FileNameDos - Receives TRUE if the element of the dirent we hit on
//         was the short (non LFN) side
//
//     LongFileName - If specified, this parameter returns the long file name
//         associated with the returned dirent.  Note that it is the caller's
//         responsibility to provide the buffer (and set MaximumLength
//         accordingly) for this unicode string.  The Length field is reset
//         to 0 by this routine on invocation.  If the supplied buffer is not
//         large enough,  a new one will be allocated from pool.
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     OEM_STRING Name;
//     UCHAR NameBuffer[12];
//
//     BOOLEAN UpcasedLfnValid = FALSE;
//     UNICODE_STRING UpcasedLfn = {0};
//     WCHAR LocalLfnBuffer[32];
//
//
//     BOOLEAN LfnInProgress = FALSE;
//     UCHAR LfnChecksum = 0;
//     ULONG LfnSize = 0;
//     ULONG LfnIndex = 0;
//     UCHAR Ordinal = 0;
//     VBO LfnByteOffset = 0;
//
//     TimerStart(Dbg);
//
//     PAGED_CODE();
//
//     DebugTrace(+1, Dbg, "FatLocateDirent\n", 0);
//
//     DebugTrace( 0, Dbg, "  ParentDirectory         = %p\n", ParentDirectory);
//     DebugTrace( 0, Dbg, "  OffsetToStartSearchFrom = %08lx\n", OffsetToStartSearchFrom);
//     DebugTrace( 0, Dbg, "  Dirent                  = %p\n", Dirent);
//     DebugTrace( 0, Dbg, "  Bcb                     = %p\n", Bcb);
//     DebugTrace( 0, Dbg, "  ByteOffset              = %08lx\n", *ByteOffset);
//
//     //
//     //  We must have acquired the parent or the vcb to synchronize with deletion.  This
//     //  is important since we can't survive racing a thread marking a series of lfn
//     //  dirents deleted - we'd get a bogus ordinal, and otherwise get really messed up.
//     //
//     //  This routine cannot do the acquire since it would be out-of-order with respect
//     //  to the Bcb resources on iterative calls.  Our order has Bcbs as the inferior resource.
//     //
//     //  Deletion always grabs the parent (safely - this used to not be possible until the
//     //  multiple fcb lockorder was fixed to be bottom up!).  Deletion always occurs with
//     //  the vcb held exclusive as well, and this will cover the cases where we can't easily
//     //  hold the parent here, see above.
//     //
//
//     NT_ASSERT( ExIsResourceAcquiredSharedLite( ParentDirectory->Header.Resource ) ||
//             ExIsResourceAcquiredExclusiveLite( ParentDirectory->Header.Resource ) ||
//             ExIsResourceAcquiredSharedLite( &ParentDirectory->Vcb->Resource ) ||
//             ExIsResourceAcquiredExclusiveLite( &ParentDirectory->Vcb->Resource ));
//
//     //
//     //  The algorithm here is pretty simple.  We just walk through the
//     //  parent directory until we:
//     //
//     //      A)  Find a matching entry.
//     //      B)  Can't Wait
//     //      C)  Hit the End of Directory
//     //      D)  Hit Eof
//     //
//     //  In the first case we found it, in the latter three cases we did not.
//     //
//
//     UNREFERENCED_PARAMETER( Flags ); // future use
//
//
//     Name.MaximumLength = 12;
//     Name.Buffer = (PCHAR)NameBuffer;
//
//     UpcasedLfn.Length = 0;
//     UpcasedLfn.MaximumLength = sizeof( LocalLfnBuffer);
//     UpcasedLfn.Buffer = LocalLfnBuffer;
//
//
//     //
//     //  If we were given a non-NULL Bcb, compute the new Dirent address
//     //  from the prior one, or unpin the Bcb if the new Dirent is not pinned.
//     //
//
//     if (*Bcb != NULL) {
//
//         if ((OffsetToStartSearchFrom / PAGE_SIZE) == (*ByteOffset / PAGE_SIZE)) {
//
//             *Dirent += (OffsetToStartSearchFrom - *ByteOffset) / sizeof(DIRENT);
//
//         } else {
//
//             FatUnpinBcb( IrpContext, *Bcb );
//         }
//     }
//
//     //
//     //  Init the Lfn if we were given one.
//     //
//
//     if (ARGUMENT_PRESENT(LongFileName)) {
//
//         LongFileName->Length = 0;
//     }
//
//     if (ARGUMENT_PRESENT(OrigLongFileName)) {
//
//         OrigLongFileName->Length = 0;
//     }
//
//     //
//     //  Init the FileNameDos flag
//     //
//
//     if (FileNameDos) {
//
//         *FileNameDos = FALSE;
//     }
//
//     //
//     //  Round up OffsetToStartSearchFrom to the nearest Dirent, and store
//     //  in ByteOffset.  Note that this wipes out the prior value.
//     //
//
//     *ByteOffset = (OffsetToStartSearchFrom +  (sizeof(DIRENT) - 1))
//                                            & ~(sizeof(DIRENT) - 1);
//
//     _SEH2_TRY {
//
//         while ( TRUE ) {
//
//             BOOLEAN FoundValidLfn;
//
//             UpcasedLfnValid = FALSE;
//
//
//             //
//             //  Try to read in the dirent
//             //
//
//             FatReadDirent( IrpContext,
//                            ParentDirectory,
//                            *ByteOffset,
//                            Bcb,
//                            Dirent,
//                            &Status );
//
//             //
//             //  If End Directory dirent or EOF, set all out parameters to
//             //  indicate entry not found and, like, bail.
//             //
//             //  Note that the order of evaluation here is important since we
//             //  cannot check the first character of the dirent until after we
//             //  know we are not beyond EOF
//             //
//
//             if ((Status == STATUS_END_OF_FILE) ||
//                 ((*Dirent)->FileName[0] == FAT_DIRENT_NEVER_USED)) {
//
//                 DebugTrace( 0, Dbg, "End of directory: entry not found.\n", 0);
//
//                 //
//                 //  If there is a Bcb, unpin it and set it to null
//                 //
//
//                 FatUnpinBcb( IrpContext, *Bcb );
//
//                 *Dirent = NULL;
//                 *ByteOffset = 0;
//                 break;
//             }
//
//             //
//             //  If the entry is marked deleted, skip.  If there was an Lfn in
//             //  progress we throw it out at this point.
//             //
//
//             if ((*Dirent)->FileName[0] == FAT_DIRENT_DELETED) {
//
//                 LfnInProgress = FALSE;
//                 goto GetNextDirent;
//             }
//
//             //
//             //  If we have wandered onto an LFN entry, try to interpret it.
//             //
//
//             if (FatData.ChicagoMode() &&
//                 ARGUMENT_PRESENT(LongFileName) &&
//                 ((*Dirent)->Attributes == FAT_DIRENT_ATTR_LFN)) {
//
//                 PLFN_DIRENT Lfn;
//
//                 Lfn = (PLFN_DIRENT)*Dirent;
//
//                 if (LfnInProgress) {
//
//                     //
//                     //  Check for a proper continuation of the Lfn in progress.
//                     //
//
//                     if ((Lfn->Ordinal & FAT_LAST_LONG_ENTRY) ||
//                         (Lfn->Ordinal == 0) ||
//                         (Lfn->Ordinal != Ordinal - 1) ||
//                         (Lfn->Checksum != LfnChecksum) ||
//                         (Lfn->MustBeZero != 0)) {
//
//                         //
//                         //  The Lfn is not proper, stop constructing it.
//                         //
//
//                         LfnInProgress = FALSE;
//
//                     } else {
//
//                         NT_ASSERT( ((LfnIndex % 13) == 0) && LfnIndex );
//
//                         LfnIndex -= 13;
//
//                         RtlCopyMemory( &LongFileName->Buffer[LfnIndex+0],
//                                        &Lfn->Name1[0],
//                                        5*sizeof(WCHAR) );
//
//                         RtlCopyMemory( &LongFileName->Buffer[LfnIndex+5],
//                                        &Lfn->Name2[0],
//                                        6 * sizeof(WCHAR) );
//
//                         RtlCopyMemory( &LongFileName->Buffer[LfnIndex+11],
//                                        &Lfn->Name3[0],
//                                        2 * sizeof(WCHAR) );
//
//                         Ordinal = Lfn->Ordinal;
//                         LfnByteOffset = *ByteOffset;
//                     }
//                 }
//
//                 //
//                 //  Now check (maybe again) if we should analyze this entry
//                 //  for a possible last entry.
//                 //
//
//                 if ((!LfnInProgress) &&
//                     (Lfn->Ordinal & FAT_LAST_LONG_ENTRY) &&
//                     ((Lfn->Ordinal & ~FAT_LAST_LONG_ENTRY) <= MAX_LFN_DIRENTS) &&
//                     (Lfn->MustBeZero == 0)) {
//
//                     BOOLEAN CheckTail = FALSE;
//
//                     Ordinal = Lfn->Ordinal & ~FAT_LAST_LONG_ENTRY;
//
//                     //
//                     //  We're usually permissive (following the lead of Win9x) when we find
//                     //  malformation of the LFN dirent pile.  I'm not sure this is a good idea,
//                     //  so I'm going to trigger corruption on this particularly ugly one.  Perhaps
//                     //  we should come back and redo the original code here with this in mind in the
//                     //  future.
//                     //
//
//                     if (Ordinal == 0) {
//
//                         //
//                         //  First LFN in the pile was zero marked as the last. This is never
//                         //  possible since oridinals are 1-based.
//                         //
//
//                         FatPopUpFileCorrupt( IrpContext, ParentDirectory );
//                         FatRaiseStatus( IrpContext, STATUS_FILE_CORRUPT_ERROR );
//                     }
//
//                     LfnIndex = (Ordinal - 1) * 13;
//
//                     FatEnsureStringBufferEnough( LongFileName,
//                                                  (USHORT)((LfnIndex + 13) << 1));
//
//                     RtlCopyMemory( &LongFileName->Buffer[LfnIndex+0],
//                                    &Lfn->Name1[0],
//                                    5*sizeof(WCHAR));
//
//                     RtlCopyMemory( &LongFileName->Buffer[LfnIndex+5],
//                                    &Lfn->Name2[0],
//                                    6 * sizeof(WCHAR) );
//
//                     RtlCopyMemory( &LongFileName->Buffer[LfnIndex+11],
//                                    &Lfn->Name3[0],
//                                    2 * sizeof(WCHAR) );
//
//                     //
//                     //  Now compute the Lfn size and make sure that the tail
//                     //  bytes are correct.
//                     //
//
//                     while (LfnIndex != (ULONG)Ordinal * 13) {
//
//                         if (!CheckTail) {
//
//                             if (LongFileName->Buffer[LfnIndex] == 0x0000) {
//
//                                 LfnSize = LfnIndex;
//                                 CheckTail = TRUE;
//                             }
//
//                         } else {
//
//                             if (LongFileName->Buffer[LfnIndex] != 0xffff) {
//
//                                 break;
//                             }
//                         }
//
//                         LfnIndex += 1;
//                     }
//
//                     //
//                     //  If we exited this loop prematurely, the LFN is not valid.
//                     //
//
//                     if (LfnIndex == (ULONG)Ordinal * 13) {
//
//                         //
//                         //  If we didn't find the NULL terminator, then the size
//                         //  is LfnIndex.
//                         //
//
//                         if (!CheckTail) {
//
//                             LfnSize = LfnIndex;
//                         }
//
//                         LfnIndex -= 13;
//                         LfnInProgress = TRUE;
//                         LfnChecksum = Lfn->Checksum;
//                         LfnByteOffset = *ByteOffset;
//                     }
//                 }
//
//                 //
//                 //  Move on to the next dirent.
//                 //
//
//                 goto GetNextDirent;
//             }
//
//             //
//             //  If this is the volume label, skip.  Note that we never arrive here
//             //  while building the LFN.  If we did, we weren't asked to find LFNs
//             //  and that is another good reason to skip this LFN fragment.
//             //
//
//             if (FlagOn((*Dirent)->Attributes, FAT_DIRENT_ATTR_VOLUME_ID)) {
//
//                 //
//                 //  If we actually were asked to hand back volume labels,
//                 //  do it.
//                 //
//
//                 if (FlagOn(Ccb->Flags, CCB_FLAG_MATCH_VOLUME_ID)) {
//
//                     break;
//                 }
//
//                 goto GetNextDirent;
//             }
//
//             //
//             //  We may have just stepped off a valid Lfn run.  Check to see if
//             //  it is indeed valid for the following dirent.
//             //
//
//             if (LfnInProgress &&
//                 (*ByteOffset == LfnByteOffset + sizeof(DIRENT)) &&
//                 (LfnIndex == 0) &&
//                 (FatComputeLfnChecksum(*Dirent) == LfnChecksum)) {
//
//                 NT_ASSERT( Ordinal == 1);
//
//                 FoundValidLfn = TRUE;
//                 LongFileName->Length = (USHORT)(LfnSize * sizeof(WCHAR));
//
//
//                 if (ARGUMENT_PRESENT(OrigLongFileName)) {
//                     *OrigLongFileName = *LongFileName;
//                 }
//
//             } else {
//
//                 FoundValidLfn = FALSE;
//             }
//
//
//
//             //
//             //  If we are supposed to match all entries, then match this entry.
//             //
//
//             if (FlagOn(Ccb->Flags, CCB_FLAG_MATCH_ALL)) {
//
//                 break;
//             }
//
//             //
//             //  Check against the short name given if one was.
//             //
//
//             if (!FlagOn( Ccb->Flags, CCB_FLAG_SKIP_SHORT_NAME_COMPARE )) {
//
//                 if (Ccb->ContainsWildCards) {
//
//                     //
//                     //  If we get one, note that all out parameters are already set.
//                     //
//
//                     (VOID)Fat8dot3ToString( IrpContext, (*Dirent), FALSE, &Name );
//
//                     //
//                     //  For fat we special case the ".." dirent because we want it to
//                     //  match ????????.??? and to do that we change ".." to "." before
//                     //  calling the Fsrtl routine.  But only do this if the expression
//                     //  is greater than one character long.
//                     //
//
//                     if ((Name.Length == 2) &&
//                         (Name.Buffer[0] == '.') &&
//                         (Name.Buffer[1] == '.') &&
//                         (Ccb->OemQueryTemplate.Wild.Length > 1)) {
//
//                         Name.Length = 1;
//                     }
//
//                     if (FatIsNameInExpression( IrpContext,
//                                                Ccb->OemQueryTemplate.Wild,
//                                                Name)) {
//
//                         DebugTrace( 0, Dbg, "Entry found: Name = \"%Z\"\n", &Name);
//                         DebugTrace( 0, Dbg, "             VBO  = %08lx\n", *ByteOffset);
//
//                         if (FileNameDos) {
//
//                             *FileNameDos = TRUE;
//                         }
//
//                         SetFlag( Ccb->Flags, CCB_FLAG_OPENED_BY_SHORTNAME );
//
//                         break;
//                     }
//
//                 } else {
//
//                     //
//                     //  Do the quickest 8.3 equivalency check possible
//                     //
//
//                     if (!FlagOn((*Dirent)->Attributes, FAT_DIRENT_ATTR_VOLUME_ID) &&
//                         (*(PULONG)&(Ccb->OemQueryTemplate.Constant[0]) == *(PULONG)&((*Dirent)->FileName[0])) &&
//                         (*(PULONG)&(Ccb->OemQueryTemplate.Constant[4]) == *(PULONG)&((*Dirent)->FileName[4])) &&
//                         (*(PUSHORT)&(Ccb->OemQueryTemplate.Constant[8]) == *(PUSHORT)&((*Dirent)->FileName[8])) &&
//                         (*(PUCHAR)&(Ccb->OemQueryTemplate.Constant[10]) == *(PUCHAR)&((*Dirent)->FileName[10]))) {
//
//                         DebugTrace( 0, Dbg, "Entry found.\n", 0);
//
//                         if (FileNameDos) {
//
//                             *FileNameDos = TRUE;
//                         }
//
//                         SetFlag( Ccb->Flags, CCB_FLAG_OPENED_BY_SHORTNAME );
//
//                         break;
//                     }
//                 }
//             }
//
//             //
//             //  No matches were found with the short name.  If an LFN exists,
//             //  use it for the search.
//             //
//
//             if (FoundValidLfn) {
//
//
//                 //
//                 //  First do a quick check here for different sized constant
//                 //  name and expression before upcasing.
//                 //
//
//                 if (!Ccb->ContainsWildCards &&
//                     (Ccb->UnicodeQueryTemplate.Length != (USHORT)(LfnSize * sizeof(WCHAR)))) {
//
//                     //
//                     //  Move on to the next dirent.
//                     //
//
//                     FoundValidLfn = FALSE;
//                     LongFileName->Length = 0;
//                     if (OrigLongFileName) {
//                         OrigLongFileName->Length = 0;
//                     }
//
//                     goto GetNextDirent;
//                 }
//
//
//
//                 if (!UpcasedLfnValid) {
//
//                     //
//                     //  We need to upcase the name we found on disk.
//                     //  We need a buffer.  Try to avoid doing an allocation.
//                     //
//
//                     FatEnsureStringBufferEnough( &UpcasedLfn,
//                                                  LongFileName->Length);
//
//                     Status = RtlUpcaseUnicodeString( &UpcasedLfn,
//                                                      LongFileName,
//                                                      FALSE );
//
//                     if (!NT_SUCCESS(Status)) {
//
//                         FatNormalizeAndRaiseStatus( IrpContext, Status );
//                     }
//
//
//                     UpcasedLfnValid = TRUE;
//
//                 }
//
//                 //
//                 //  Do the compare
//                 //
//
//                 if (Ccb->ContainsWildCards) {
//
//                     if (FsRtlIsNameInExpression( &Ccb->UnicodeQueryTemplate,
//                                                  &UpcasedLfn,
//                                                  TRUE,
//                                                  NULL )) {
//
//                         break;
//                     }
//
//                 } else {
//
//                     if (FsRtlAreNamesEqual( &Ccb->UnicodeQueryTemplate,
//                                             &UpcasedLfn,
//                                             BooleanFlagOn( Ccb->Flags, CCB_FLAG_QUERY_TEMPLATE_MIXED ),
//                                             NULL )) {
//
//                         break;
//                     }
//
//
//                 }
//
//             }
//
//             //
//             //  This long name was not a match.  Zero out the Length field.
//             //
//
//             if (FoundValidLfn) {
//
//                 FoundValidLfn = FALSE;
//                 LongFileName->Length = 0;
//
//
//                 if (OrigLongFileName) {
//                     OrigLongFileName->Length = 0;
//                 }
//             }
//
// GetNextDirent:
//
//             //
//             //  Move on to the next dirent.
//             //
//
//             *ByteOffset += sizeof(DIRENT);
//             *Dirent += 1;
//         }
//
//     } _SEH2_FINALLY {
//
//         FatFreeStringBuffer( &UpcasedLfn );
//
//
//     } _SEH2_END;
//
//     DebugTrace(-1, Dbg, "FatLocateDirent -> (VOID)\n", 0);
//
//     TimerStop(Dbg,"FatLocateDirent");
//
//     return;
// }
//
//
// _Requires_lock_held_(_Global_critical_region_)
// VOID
// FatLocateSimpleOemDirent (
//     IN PIRP_CONTEXT IrpContext,
//     IN PDCB ParentDirectory,
//     IN POEM_STRING FileName,
//     OUT PDIRENT *Dirent,
//     OUT PBCB *Bcb,
//     OUT PVBO ByteOffset
//     )
//
// /*++
//
// Routine Description:
//
//     This routine locates on the disk an undelted simple Oem dirent.  By simple
//     I mean that FileName cannot contain any extended characters, and we do
//     not search LFNs or return them.
//
// Arguments:
//
//     ParentDirectory - Supplies the DCB for the directory in which
//         to search
//
//     FileName - Supplies the filename to search for.  The name may contain
//         wild cards
//
//     OffsetToStartSearchFrom - Supplies the VBO within the parent directory
//         from which to start looking for another real dirent.
//
//     Dirent - Receives a pointer to the located dirent if one was found
//         or NULL otherwise.
//
//     Bcb - Receives the Bcb for the located dirent if one was found or
//         NULL otherwise.
//
//     ByteOffset - Receives the VBO within the Parent directory for
//         the located dirent if one was found, or 0 otherwise.
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     CCB LocalCcb;
//
//     PAGED_CODE();
//
//     //
//     //  Note, this routine is called rarely, so performance is not critical.
//     //  Just fill in a Ccb structure on my stack with the values that are
//     //  required.
//     //
//
//     FatStringTo8dot3( IrpContext,
//                       *FileName,
//                       &LocalCcb.OemQueryTemplate.Constant );
//     LocalCcb.ContainsWildCards = FALSE;
//     LocalCcb.Flags = 0;
//
//     FatLocateDirent( IrpContext,
//                      ParentDirectory,
//                      &LocalCcb,
//                      0,
//                      NULL,
//                      Dirent,
//                      Bcb,
//                      ByteOffset,
//                      NULL,
//                      NULL,
//                      NULL );
//
//     return;
// }
//
//
//
// _Requires_lock_held_(_Global_critical_region_)
// VOID
// FatLocateVolumeLabel (
//     IN PIRP_CONTEXT IrpContext,
//     IN PVCB Vcb,
//     OUT PDIRENT *Dirent,
//     OUT PBCB *Bcb,
//     OUT PVBO ByteOffset
//     )
//
// /*++
//
// Routine Description:
//
//     This routine locates on the disk a dirent representing the volume
//     label.  It does this by searching the root directory for a special
//     volume label dirent.
//
// Arguments:
//
//     Vcb - Supplies the VCB for the volume to search
//
//     Dirent - Receives a pointer to the located dirent if one was found
//         or NULL otherwise.
//
//     Bcb - Receives the Bcb for the located dirent if one was found or
//         NULL otherwise.
//
//     ByteOffset - Receives the VBO within the Parent directory for
//         the located dirent if one was found, or 0 otherwise.
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     PAGED_CODE();
//
//     DebugTrace(+1, Dbg, "FatLocateVolumeLabel\n", 0);
//
//     DebugTrace( 0, Dbg, "  Vcb        = %p\n", Vcb);
//     DebugTrace( 0, Dbg, "  Dirent     = %p\n", Dirent);
//     DebugTrace( 0, Dbg, "  Bcb        = %p\n", Bcb);
//     DebugTrace( 0, Dbg, "  ByteOffset = %08lx\n", *ByteOffset);
//
//     //
//     //  The algorithm here is really simple.  We just walk through the
//     //  root directory until we:
//     //
//     //      A)  Find the non-deleted volume label
//     //      B)  Can't Wait
//     //      C)  Hit the End of Directory
//     //      D)  Hit Eof
//     //
//     //  In the first case we found it, in the latter three cases we did not.
//     //
//
//     *Bcb = NULL;
//     *ByteOffset = 0;
//
//     while ( TRUE ) {
//
//         //
//         //  Try to read in the dirent
//         //
//
//         FatReadDirent( IrpContext,
//                        Vcb->RootDcb,
//                        *ByteOffset,
//                        Bcb,
//                        Dirent,
//                        &Status );
//
//         //
//         //  If End Directory dirent or EOF, set all out parameters to
//         //  indicate volume label not found and, like, bail.
//         //
//         //  Note that the order of evaluation here is important since we cannot
//         //  check the first character of the dirent until after we know we
//         //  are not beyond EOF
//         //
//
//         if ((Status == STATUS_END_OF_FILE) ||
//             ((*Dirent)->FileName[0] == FAT_DIRENT_NEVER_USED)) {
//
//             DebugTrace( 0, Dbg, "Volume label not found.\n", 0);
//
//             //
//             //  If there is a Bcb, unpin it and set it to null
//             //
//
//             FatUnpinBcb( IrpContext, *Bcb );
//
//             *Dirent = NULL;
//             *ByteOffset = 0;
//             break;
//         }
//
//         //
//         //  If the entry is the non-deleted volume label break from the loop.
//         //
//         //  Note that all out parameters are already correctly set.
//         //
//
//         if ((((*Dirent)->Attributes & ~FAT_DIRENT_ATTR_ARCHIVE) == FAT_DIRENT_ATTR_VOLUME_ID) &&
//             ((*Dirent)->FileName[0] != FAT_DIRENT_DELETED)) {
//
//             DebugTrace( 0, Dbg, "Volume label found at VBO = %08lx\n", *ByteOffset);
//
//             //
//             //  We may set this dirty, so pin it.
//             //
//
//             FatPinMappedData( IrpContext,
//                               Vcb->RootDcb,
//                               *ByteOffset,
//                               sizeof(DIRENT),
//                               Bcb );
//
//             break;
//         }
//
//         //
//         //  Move on to the next dirent.
//         //
//
//         *ByteOffset += sizeof(DIRENT);
//         *Dirent += 1;
//     }
//
//
//     DebugTrace(-1, Dbg, "FatLocateVolumeLabel -> (VOID)\n", 0);
//
//     return;
// }
//
//
//
// _Requires_lock_held_(_Global_critical_region_)
// VOID
// FatGetDirentFromFcbOrDcb (
//     IN PIRP_CONTEXT IrpContext,
//     IN PFCB FcbOrDcb,
//     IN BOOLEAN ReturnOnFailure,
//     OUT PDIRENT *Dirent,
//     OUT PBCB *Bcb
//     )
//
// /*++
//
// Routine Description:
//
//     This routine reads locates on the disk the dirent denoted by the
//     specified Fcb/Dcb.
//
// Arguments:
//
//     FcbOrDcb - Supplies the FCB/DCB for the file/directory whose dirent
//         we are trying to read in.  This must not be the root dcb.
//
//     Dirent - Receives a pointer to the dirent
//
//     Bcb - Receives the Bcb for the dirent
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     NTSTATUS DontCare = STATUS_SUCCESS;
//
//     PAGED_CODE();
//
//     DebugTrace(+1, Dbg, "FatGetDirentFromFcbOrDcb\n", 0);
//
//     DebugTrace( 0, Dbg, "  FcbOrDcb = %p\n", FcbOrDcb);
//     DebugTrace( 0, Dbg, "  Dirent   = %p\n", Dirent);
//     DebugTrace( 0, Dbg, "  Bcb      = %p\n", Bcb);
//
//     //
//     //  Assert that we are not attempting this on the root directory.
//     //
//
//     NT_ASSERT( NodeType(FcbOrDcb) != FAT_NTC_ROOT_DCB );
//
//     //
//     //  We know the offset of the dirent within the directory file,
//     //  so we just read it (with pinning).
//     //
//
//     FatReadDirectoryFile( IrpContext,
//                           FcbOrDcb->ParentDcb,
//                           FcbOrDcb->DirentOffsetWithinDirectory,
//                           sizeof(DIRENT),
//                           TRUE,
//                           Bcb,
//                           (PVOID *)Dirent,
//                           &DontCare );
//
//     //
//     //  Previous call can fail.  We used to assert success, but we use this
//     //  as part of volume verification (DetermineAndMarkFcbCondition) after
//     //  media has been removed.  Clearly the directory could shrink and we
//     //  would try to read beyond filesize.
//     //
//     //  The caller will note this via NULL pointers for Bcb/Buffer.  Note that
//     //  both asserts below are OK since this should never happen fixed media.
//     //
//     //  This was a Prefix catch.
//     //
//
//     NT_ASSERT( FlagOn( FcbOrDcb->Vcb->VcbState, VCB_STATE_FLAG_REMOVABLE_MEDIA) ||
//             NT_SUCCESS( DontCare ));
//
//     //
//     //  Note also that the only way this could fail is if the Fcb was being
//     //  verified.  This can't happen if the Fcb is in good condition.
//     //
//     //  Also a Prefix catch.
//     //
//
//     NT_ASSERT( NT_SUCCESS( DontCare ) || FcbOrDcb->FcbCondition == FcbNeedsToBeVerified );
//
//     //
//     //  This should never happen except in very specific cases (during volume
//     //  verify) but we'll handle and raise here to save all callers checking the
//     //  pointers.
//     //
//
//     if ((NULL == *Dirent) && !ReturnOnFailure) {
//
//         NT_ASSERT( FALSE);
//         FatRaiseStatus( IrpContext, STATUS_FILE_CORRUPT_ERROR);
//     }
//
//     DebugTrace(-1, Dbg, "FatGetDirentFromFcbOrDcb -> (VOID)\n", 0);
// }
//
//
//
// _Requires_lock_held_(_Global_critical_region_)
// BOOLEAN
// FatIsDirectoryEmpty (
//     IN PIRP_CONTEXT IrpContext,
//     IN PDCB Dcb
//     )
//
// /*++
//
// Routine Description:
//
//     This routine indicates to the caller if the specified directory
//     is empty.  (i.e., it is not the root dcb and it only contains
//     the "." and ".." entries, or deleted files).
//
// Arguments:
//
//     Dcb - Supplies the DCB for the directory being queried.
//
// Return Value:
//
//     BOOLEAN - Returns TRUE if the directory is empty and
//         FALSE if the directory and is not empty.
//
// --*/
//
// {
//     PBCB Bcb;
//     ULONG ByteOffset;
//     PDIRENT Dirent = NULL;
//
//     BOOLEAN IsDirectoryEmpty = FALSE;
//
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     PAGED_CODE();
//
//     DebugTrace(+1, Dbg, "FatIsDirectoryEmpty\n", 0);
//
//     DebugTrace( 0, Dbg, "  Dcb              = %p\n", Dcb);
//     DebugTrace( 0, Dbg, "  IsDirectoryEmpty = %08lx\n", IsDirectoryEmpty);
//
//     //
//     //  Check to see if the first entry is an and of directory marker.
//     //  For the root directory we check at Vbo = 0, for normal directories
//     //  we check after the "." and ".." entries.
//     //
//
//     ByteOffset = (NodeType(Dcb) == FAT_NTC_ROOT_DCB) ? 0 : 2*sizeof(DIRENT);
//
//     //
//     //  We just march through the directory looking for anything other
//     //  than deleted files, LFNs, an EOF, or end of directory marker.
//     //
//
//     Bcb = NULL;
//
//     _SEH2_TRY {
//
//         while ( TRUE ) {
//
//             //
//             //  Try to read in the dirent
//             //
//
//             FatReadDirent( IrpContext,
//                            Dcb,
//                            ByteOffset,
//                            &Bcb,
//                            &Dirent,
//                            &Status );
//
//             //
//             //  If End Directory dirent or EOF, set IsDirectoryEmpty to TRUE and,
//             //  like, bail.
//             //
//             //  Note that the order of evaluation here is important since we cannot
//             //  check the first character of the dirent until after we know we
//             //  are not beyond EOF
//             //
//
//             if ((Status == STATUS_END_OF_FILE) ||
//                 (Dirent->FileName[0] == FAT_DIRENT_NEVER_USED)) {
//
//                 DebugTrace( 0, Dbg, "Empty.  Last exempt entry at VBO = %08lx\n", ByteOffset);
//
//                 IsDirectoryEmpty = TRUE;
//                 break;
//             }
//
//             //
//             //  If this dirent is NOT deleted or an LFN set IsDirectoryEmpty to
//             //  FALSE and, like, bail.
//             //
//
//             if ((Dirent->FileName[0] != FAT_DIRENT_DELETED) &&
//                 (Dirent->Attributes != FAT_DIRENT_ATTR_LFN)) {
//
//
//                     break;
//
//
//             }
//
//             //
//             //  Move on to the next dirent.
//             //
//
//             ByteOffset += sizeof(DIRENT);
//             Dirent += 1;
//         }
//
//     } _SEH2_FINALLY {
//
//         FatUnpinBcb( IrpContext, Bcb );
//     } _SEH2_END;
//
//     DebugTrace(-1, Dbg, "FatIsDirectoryEmpty -> %ld\n", IsDirectoryEmpty);
//
//     return IsDirectoryEmpty;
// }
//
//
//
//
//
// VOID
// FatConstructDirent (
//     IN PIRP_CONTEXT IrpContext,
//     IN OUT PDIRENT Dirent,
//     IN POEM_STRING FileName,
//     IN BOOLEAN ComponentReallyLowercase,
//     IN BOOLEAN ExtensionReallyLowercase,
//     IN PUNICODE_STRING Lfn OPTIONAL,
//     IN USHORT Attributes,
//     IN BOOLEAN ZeroAndSetTimeFields,
//     IN PLARGE_INTEGER SetCreationTime OPTIONAL
//     )
//
// /*++
//
// Routine Description:
//
//     This routine modifies the fields of a dirent.
//
// Arguments:
//
//     Dirent - Supplies the dirent being modified.
//
//     FileName - Supplies the name to store in the Dirent.  This
//         name must not contain wildcards.
//
//     ComponentReallyLowercase - This boolean indicates that the User Specified
//         compoent name was really all a-z and < 0x80 characters.  We set the
//         magic bit in this case.
//
//     ExtensionReallyLowercase - Same as above, but for the extension.
//
//     Lfn - May supply a long file name.
//
//     Attributes - Supplies the attributes to store in the dirent
//
//     ZeroAndSetTimeFields - Tells whether or not to initially zero the dirent
//         and update the time fields.
//
//     SetCreationTime - If specified, contains a timestamp to use as the creation
//         time of this dirent
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     PAGED_CODE();
//
//     DebugTrace(+1, Dbg, "FatConstructDirent\n", 0);
//
//     DebugTrace( 0, Dbg, "  Dirent             = %p\n", Dirent);
//     DebugTrace( 0, Dbg, "  FileName           = %Z\n", FileName);
//     DebugTrace( 0, Dbg, "  Attributes         = %08lx\n", Attributes);
//
//     if (ZeroAndSetTimeFields) {
//
//         RtlZeroMemory( Dirent, sizeof(DIRENT) );
//     }
//
//     //
//     //  We just merrily go and fill up the dirent with the fields given.
//     //
//
//     FatStringTo8dot3( IrpContext, *FileName, (PFAT8DOT3)&Dirent->FileName[0] );
//
//     if (ZeroAndSetTimeFields || SetCreationTime) {
//
//         LARGE_INTEGER Time, SaveTime;
//
//         KeQuerySystemTime( &Time );
//
//         if (FatData.ChicagoMode()) {
//
//             if (!SetCreationTime || !FatNtTimeToFatTime( IrpContext,
//                                                          SetCreationTime,
//                                                          FALSE,
//                                                          &Dirent->CreationTime,
//                                                          &Dirent->CreationMSec )) {
//
//                 //
//                 //  No tunneled time or the tunneled time was bogus. Since we aren't
//                 //  responsible for initializing the to-be-created Fcb with creation
//                 //  time, we can't do the usual thing and let NtTimeToFatTime perform
//                 //  rounding on the timestamp - this would mess up converting to the
//                 //  LastWriteTime below.
//                 //
//
//                 SaveTime = Time;
//
//                 if (!FatNtTimeToFatTime( IrpContext,
//                                          &SaveTime,
//                                          FALSE,
//                                          &Dirent->CreationTime,
//                                          &Dirent->CreationMSec )) {
//
//                     //
//                     //  Failed again. Wow.
//                     //
//
//                     RtlZeroMemory( &Dirent->CreationTime, sizeof(FAT_TIME_STAMP));
//                     Dirent->CreationMSec = 0;
//                 }
//             }
//         }
//
//         if (ZeroAndSetTimeFields) {
//
//             //
//             //  We only touch the other timestamps if we are initializing the dirent
//             //
//
//             if (!FatNtTimeToFatTime( IrpContext,
//                                      &Time,
//                                      TRUE,
//                                      &Dirent->LastWriteTime,
//                                      NULL )) {
//
//                 DebugTrace( 0, Dbg, "Current time invalid.\n", 0);
//
//                 RtlZeroMemory( &Dirent->LastWriteTime, sizeof(FAT_TIME_STAMP) );
//             }
//
//             if (FatData.ChicagoMode()) {
//
//                 Dirent->LastAccessDate = Dirent->LastWriteTime.Date;
//             }
//         }
//     }
//
//     //
//     //  Copy the attributes
//     //
//
//     Dirent->Attributes = (UCHAR)Attributes;
//
//     //
//     //  Set the magic bit here, to tell dirctrl.c that this name is really
//     //  lowercase.
//     //
//
//     Dirent->NtByte = 0;
//
//     if (ComponentReallyLowercase) {
//
//         SetFlag( Dirent->NtByte, FAT_DIRENT_NT_BYTE_8_LOWER_CASE );
//     }
//
//     if (ExtensionReallyLowercase) {
//
//         SetFlag( Dirent->NtByte, FAT_DIRENT_NT_BYTE_3_LOWER_CASE );
//     }
//
//     //
//     //  See if we have to create an Lfn entry
//     //
//
//     if (ARGUMENT_PRESENT(Lfn)) {
//
//         UCHAR DirentChecksum;
//         UCHAR DirentsInLfn;
//         UCHAR LfnOrdinal;
//         PWCHAR LfnBuffer;
//         PLFN_DIRENT LfnDirent;
//
//         NT_ASSERT( FatData.ChicagoMode() );
//
//         DirentChecksum = FatComputeLfnChecksum( Dirent );
//
//         LfnOrdinal =
//         DirentsInLfn = (UCHAR)FAT_LFN_DIRENTS_NEEDED(Lfn);
//
//         LfnBuffer = &Lfn->Buffer[(DirentsInLfn - 1) * 13];
//
//         NT_ASSERT( DirentsInLfn <= MAX_LFN_DIRENTS );
//
//         for (LfnDirent = (PLFN_DIRENT)Dirent - DirentsInLfn;
//              LfnDirent < (PLFN_DIRENT)Dirent;
//              LfnDirent += 1, LfnOrdinal -= 1, LfnBuffer -= 13) {
//
//             WCHAR FinalLfnBuffer[13];
//             PWCHAR Buffer;
//
//             //
//             //  We need to special case the "final" dirent.
//             //
//
//             if (LfnOrdinal == DirentsInLfn) {
//
//                 ULONG i;
//                 ULONG RemainderChars;
//
//                 RemainderChars = (Lfn->Length / sizeof(WCHAR)) % 13;
//
//                 LfnDirent->Ordinal = LfnOrdinal | FAT_LAST_LONG_ENTRY;
//
//                 if (RemainderChars != 0) {
//
//                     RtlCopyMemory( FinalLfnBuffer,
//                                    LfnBuffer,
//                                    RemainderChars * sizeof(WCHAR) );
//
//                     for (i = RemainderChars; i < 13; i++) {
//
//                         //
//                         //  Figure out which character to use.
//                         //
//
//                         if (i == RemainderChars) {
//
//                             FinalLfnBuffer[i] = 0x0000;
//
//                         } else {
//
//                             FinalLfnBuffer[i] = 0xffff;
//                         }
//                     }
//
//                     Buffer = FinalLfnBuffer;
//
//                 } else {
//
//                     Buffer = LfnBuffer;
//                 }
//
//             } else {
//
//                 LfnDirent->Ordinal = LfnOrdinal;
//
//                 Buffer = LfnBuffer;
//             }
//
//             //
//             //  Now fill in the name.
//             //
//
//             RtlCopyMemory( &LfnDirent->Name1[0],
//                            &Buffer[0],
//                            5 * sizeof(WCHAR) );
//
//             RtlCopyMemory( &LfnDirent->Name2[0],
//                            &Buffer[5],
//                            6 * sizeof(WCHAR) );
//
//             RtlCopyMemory( &LfnDirent->Name3[0],
//                            &Buffer[11],
//                            2 * sizeof(WCHAR) );
//
//             //
//             //  And the other fields
//             //
//
//             LfnDirent->Attributes = FAT_DIRENT_ATTR_LFN;
//
//             LfnDirent->Type = 0;
//
//             LfnDirent->Checksum = DirentChecksum;
//
//             LfnDirent->MustBeZero = 0;
//         }
//     }
//
//     DebugTrace(-1, Dbg, "FatConstructDirent -> (VOID)\n", 0);
//     return;
// }
//
//
// VOID
// FatConstructLabelDirent (
//     IN PIRP_CONTEXT IrpContext,
//     IN OUT PDIRENT Dirent,
//     IN POEM_STRING Label
//     )
//
// /*++
//
// Routine Description:
//
//     This routine modifies the fields of a dirent to be used for a label.
//
// Arguments:
//
//     Dirent - Supplies the dirent being modified.
//
//     Label - Supplies the name to store in the Dirent.  This
//             name must not contain wildcards.
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     PAGED_CODE();
//
//     DebugTrace(+1, Dbg, "FatConstructLabelDirent\n", 0);
//
//     DebugTrace( 0, Dbg, "  Dirent             = %p\n", Dirent);
//     DebugTrace( 0, Dbg, "  Label              = %Z\n", Label);
//
//     RtlZeroMemory( Dirent, sizeof(DIRENT) );
//
//     //
//     //  We just merrily go and fill up the dirent with the fields given.
//     //
//
//     RtlCopyMemory( Dirent->FileName, Label->Buffer, Label->Length );
//
//     //
//     // Pad the label with spaces, not nulls.
//     //
//
//     RtlFillMemory( &Dirent->FileName[Label->Length], 11 - Label->Length, ' ');
//
//     Dirent->LastWriteTime = FatGetCurrentFatTime( IrpContext );
//
//     Dirent->Attributes = FAT_DIRENT_ATTR_VOLUME_ID;
//     Dirent->ExtendedAttributes = 0;
//     Dirent->FileSize = 0;
//
//     DebugTrace(-1, Dbg, "FatConstructLabelDirent -> (VOID)\n", 0);
//     return;
// }
//
//
// _Requires_lock_held_(_Global_critical_region_)
// VOID
// FatSetFileSizeInDirent (
//     IN PIRP_CONTEXT IrpContext,
//     IN PFCB Fcb,
//     IN PULONG AlternativeFileSize OPTIONAL
//     )
//
// /*++
//
// Routine Description:
//
//     This routine saves the file size in an fcb into its dirent.
//
// Arguments:
//
//     Fcb - Supplies the Fcb being referenced
//
//     AlternativeFileSize - If non-null we use the ULONG it points to as
//         the new file size.  Otherwise we use the one in the Fcb.
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     PDIRENT Dirent;
//     PBCB DirentBcb;
//
//     PAGED_CODE();
//
//     NT_ASSERT( Fcb->FcbCondition == FcbGood );
//
//     FatGetDirentFromFcbOrDcb( IrpContext,
//                               Fcb,
//                               FALSE,
//                               &Dirent,
//                               &DirentBcb );
//     _SEH2_TRY {
//
//         Dirent->FileSize = ARGUMENT_PRESENT( AlternativeFileSize ) ?
//                            *AlternativeFileSize : Fcb->Header.FileSize.LowPart;
//
//
//         FatSetDirtyBcb( IrpContext, DirentBcb, Fcb->Vcb, TRUE );
//
//     } _SEH2_FINALLY {
//
//         FatUnpinBcb( IrpContext, DirentBcb );
//     } _SEH2_END;
// }
//
// _Requires_lock_held_(_Global_critical_region_)
// VOID
// FatSetFileSizeInDirentNoRaise (
//     IN PIRP_CONTEXT IrpContext,
//     IN PFCB Fcb,
//     IN PULONG AlternativeFileSize OPTIONAL
//     )
//
// /*++
//
// Routine Description:
//
//     This routine saves the file size in an fcb into its dirent.
//     All exceptions thrown from FatSetFileSizeInDirent are
//     silently swallowed.
//
// Arguments:
//
//     Fcb - Supplies the Fcb being referenced
//
//     AlternativeFileSize - If non-null we use the ULONG it points to as
//         the new file size.  Otherwise we use the one in the Fcb.
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     _SEH2_TRY {
//
//         FatSetFileSizeInDirent( IrpContext, Fcb, AlternativeFileSize );
//
//     } _SEH2_EXCEPT(FatExceptionFilter( IrpContext, _SEH2_GetExceptionInformation() )) {
//
//         NOTHING;
//     } _SEH2_END;
// }
//
//
// _Requires_lock_held_(_Global_critical_region_)
// VOID
// FatUpdateDirentFromFcb (
//    IN PIRP_CONTEXT IrpContext,
//    IN PFILE_OBJECT FileObject,
//    IN PFCB FcbOrDcb,
//    IN PCCB Ccb
//    )
//
//
// /*++
//
// Routine Description:
//
//     This routine modifies an objects directory entry based on the hints
//     that have been built up over previous operations on a handle.  Notify
//     change filters are built and fired as a result of these updates.
//
// Arguments:
//
//     FileObject - Fileobject representing the handle involved
//
//     FcbOrDcb - File/Dir involved
//
//     Ccb - User context involved
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     BOOLEAN SetArchiveBit;
//
//     BOOLEAN UpdateFileSize;
//     BOOLEAN UpdateLastWriteTime;
//     BOOLEAN UpdateLastAccessTime;
//     BOOLEAN UpdateDirent = FALSE;
//
//     PDIRENT Dirent;
//     PBCB DirentBcb = NULL;
//     ULONG NotifyFilter = 0;
//     FAT_TIME_STAMP CurrentFatTime = {0};
//
//     LARGE_INTEGER CurrentTime;
//     LARGE_INTEGER CurrentDay = {0};
//     LARGE_INTEGER LastAccessDay;
//
//     PAGED_CODE();
//
//     //
//     //  Nothing to do if the fcb is bad, volume is readonly or we got the
//     //  root dir.
//     //
//
//     if (FcbOrDcb->FcbCondition != FcbGood ||
//         NodeType(FcbOrDcb) == FAT_NTC_ROOT_DCB ||
//         FlagOn(FcbOrDcb->Vcb->VcbState, VCB_STATE_FLAG_WRITE_PROTECTED)) {
//
//         return;
//     }
//
//     //
//     //  Check if we should be changing the time or file size and set
//     //  the archive bit on the file.
//     //
//
//     KeQuerySystemTime( &CurrentTime );
//
//     //
//     //  Note that we HAVE to use BooleanFlagOn() here because
//     //  FO_FILE_SIZE_CHANGED > 0x80 (i.e., not in the first byte).
//     //
//
//     SetArchiveBit = BooleanFlagOn(FileObject->Flags, FO_FILE_MODIFIED);
//
//     UpdateLastWriteTime = FlagOn(FileObject->Flags, FO_FILE_MODIFIED) &&
//                           !FlagOn(Ccb->Flags, CCB_FLAG_USER_SET_LAST_WRITE);
//
//     UpdateFileSize = NodeType(FcbOrDcb) == FAT_NTC_FCB &&
//                      BooleanFlagOn(FileObject->Flags, FO_FILE_SIZE_CHANGED);
//
//     //
//     //  Do one further check here of access time.  Only update it if
//     //  the current version is at least one day old.  We know that
//     //  the current FcbOrDcb->LastAccessTime corresponds to 12 midnight local
//     //  time, so just see if the current time is on the same day.
//     //
//
//     if (FatData.ChicagoMode() &&
//         (UpdateLastWriteTime ||
//          FlagOn(FileObject->Flags, FO_FILE_FAST_IO_READ)) &&
//         !FlagOn(Ccb->Flags, CCB_FLAG_USER_SET_LAST_ACCESS)) {
//
//         ExSystemTimeToLocalTime( &FcbOrDcb->LastAccessTime, &LastAccessDay );
//         ExSystemTimeToLocalTime( &CurrentTime, &CurrentDay );
//
//         LastAccessDay.QuadPart /= FatOneDay.QuadPart;
//         CurrentDay.QuadPart /= FatOneDay.QuadPart;
//
//         if (LastAccessDay.LowPart != CurrentDay.LowPart) {
//
//             UpdateLastAccessTime = TRUE;
//
//         } else {
//
//             UpdateLastAccessTime = FALSE;
//         }
//
//     } else {
//
//         UpdateLastAccessTime = FALSE;
//     }
//
//     if (SetArchiveBit ||
//         UpdateFileSize ||
//         UpdateLastWriteTime ||
//         UpdateLastAccessTime
//         ) {
//
//         DebugTrace(0, Dbg, "Update Time and/or file size on File/Dir\n", 0);
//
//         _SEH2_TRY {
//
//             _SEH2_TRY {
//
// #if (NTDDI_VERSION >= NTDDI_WIN8)
//                 //
//                 //  Break parent directory oplock.  Directory oplock breaks are
//                 //  always advisory, so we will never block/get STATUS_PENDING here.
//                 //
//
//                 if (FcbOrDcb->ParentDcb != NULL) {
//
//                     FsRtlCheckOplockEx( FatGetFcbOplock(FcbOrDcb->ParentDcb),
//                                         IrpContext->OriginatingIrp,
//                                         OPLOCK_FLAG_PARENT_OBJECT,
//                                         NULL,
//                                         NULL,
//                                         NULL );
//                 }
// #endif
//
//                 //
//                 //  Get the dirent
//                 //
//
//                 FatGetDirentFromFcbOrDcb( IrpContext,
//                                           FcbOrDcb,
//                                           FALSE,
//                                           &Dirent,
//                                           &DirentBcb );
//
//                 if (UpdateLastWriteTime || UpdateLastAccessTime) {
//
//                     (VOID)FatNtTimeToFatTime( IrpContext,
//                                               &CurrentTime,
//                                               TRUE,
//                                               &CurrentFatTime,
//                                               NULL );
//                 }
//
//                 if (SetArchiveBit) {
//
//                     Dirent->Attributes |= FILE_ATTRIBUTE_ARCHIVE;
//                     FcbOrDcb->DirentFatFlags |= FILE_ATTRIBUTE_ARCHIVE;
//
//                     NotifyFilter |= FILE_NOTIFY_CHANGE_ATTRIBUTES;
//                     UpdateDirent = TRUE;
//                 }
//
//                 if (UpdateLastWriteTime) {
//
//                     //
//                     //  Update its time of last write
//                     //
//
//                     FcbOrDcb->LastWriteTime = CurrentTime;
//                     Dirent->LastWriteTime = CurrentFatTime;
//
//                     //
//                     //  We call the notify package to report that the
//                     //  last modification time has changed.
//                     //
//
//                     NotifyFilter |= FILE_NOTIFY_CHANGE_LAST_WRITE;
//                     UpdateDirent = TRUE;
//                 }
//
//                 if (UpdateLastAccessTime) {
//
//                     //
//                     //  Now we have to truncate the local time down
//                     //  to the current day, then convert back to UTC.
//                     //
//
//                     FcbOrDcb->LastAccessTime.QuadPart =
//                         CurrentDay.QuadPart * FatOneDay.QuadPart;
//
//                     ExLocalTimeToSystemTime( &FcbOrDcb->LastAccessTime,
//                                              &FcbOrDcb->LastAccessTime );
//
//                     Dirent->LastAccessDate = CurrentFatTime.Date;
//
//                     //
//                     //  We call the notify package to report that the
//                     //  last access time has changed.
//                     //
//
//                     NotifyFilter |= FILE_NOTIFY_CHANGE_LAST_ACCESS;
//                     UpdateDirent = TRUE;
//                 }
//
//                 if (UpdateFileSize) {
//
//                     //
//                     //  Perhaps we were called to make certain that the
//                     //  filesize on disc was updated - don't bother updating
//                     //  and firing the filter if nothing changed.
//                     //
//
//                     NT_ASSERT( NodeType(FcbOrDcb) == FAT_NTC_FCB );
//
//                     if (Dirent->FileSize != FcbOrDcb->Header.FileSize.LowPart) {
//
//                         //
//                         //  Update the dirent file size
//                         //
//
//                         Dirent->FileSize = FcbOrDcb->Header.FileSize.LowPart;
//
//                         //
//                         //  We call the notify package to report that the
//                         //  size has changed.
//                         //
//
//                         NotifyFilter |= FILE_NOTIFY_CHANGE_SIZE;
//                         UpdateDirent = TRUE;
//                     }
//
//
//                 }
//
//
//                 FatNotifyReportChange( IrpContext,
//                                        FcbOrDcb->Vcb,
//                                        FcbOrDcb,
//                                        NotifyFilter,
//                                        FILE_ACTION_MODIFIED );
//
//                 if (UpdateDirent) {
//
//                     //
//                     //  If all we did was update last access time,
//                     //  don't mark the volume dirty.
//                     //
//
//                     FatSetDirtyBcb( IrpContext,
//                                     DirentBcb,
//                                     NotifyFilter == FILE_NOTIFY_CHANGE_LAST_ACCESS ?
//                                     NULL : FcbOrDcb->Vcb,
//                                     TRUE );
//                 }
//
//             } _SEH2_EXCEPT( FsRtlIsNtstatusExpected(_SEH2_GetExceptionCode()) ?
//                       EXCEPTION_EXECUTE_HANDLER : EXCEPTION_CONTINUE_SEARCH ) {
//
//                   FatResetExceptionState( IrpContext );
//             } _SEH2_END;
//
//         } _SEH2_FINALLY {
//
//             FatUnpinBcb( IrpContext, DirentBcb );
//         } _SEH2_END;
//     }
//
// }
//
//
// //
// //  Internal support routine
// //
//
// UCHAR
// FatComputeLfnChecksum (
//     PDIRENT Dirent
//     )
//
// /*++
//
// Routine Description:
//
//     This routine computes the Chicago long file name checksum.
//
// Arguments:
//
//     Dirent - Specifies the dirent that we are to compute a checksum for.
//
// Return Value:
//
//     The checksum.
//
// --*/
//
// {
//     ULONG i;
//     UCHAR Checksum;
//
//     PAGED_CODE();
//
//     Checksum = Dirent->FileName[0];
//
//     for (i=1; i < 11; i++) {
//
//         Checksum = ((Checksum & 1) ? 0x80 : 0) +
//                     (Checksum >> 1) +
//                     Dirent->FileName[i];
//     }
//
//     return Checksum;
// }
//
//
//
//
// //
// //  Internal support routine
// //
//
// _Requires_lock_held_(_Global_critical_region_)
// VOID
// FatRescanDirectory (
//     PIRP_CONTEXT IrpContext,
//     PDCB Dcb
//     )
//
// /*++
//
// Routine Description:
//
//     This routine rescans the given directory, finding the first unused
//     dirent, first deleted dirent, and setting the free dirent bitmap
//     appropriately.
//
// Arguments:
//
//     Dcb - Supplies the directory to rescan.
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     PBCB Bcb = NULL;
//     PDIRENT Dirent = NULL;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     ULONG UnusedVbo;
//     ULONG DeletedHint;
//     ULONG DirentIndex;
//     ULONG DirentsThisRun;
//     ULONG StartIndexOfThisRun;
//
//     enum RunType {
//         InitialRun,
//         FreeDirents,
//         AllocatedDirents,
//     } CurrentRun;
//
//     PAGED_CODE();
//
//     DebugTrace( 0, Dbg, "We must scan the whole directory.\n", 0);
//
//     UnusedVbo = 0;
//     DeletedHint = 0xffffffff;
//
//     //
//     //  To start with, we have to find out if the first dirent is free.
//     //
//
//     CurrentRun = InitialRun;
//     DirentIndex =
//     StartIndexOfThisRun = 0;
//
//     _SEH2_TRY {
//
//         while ( TRUE ) {
//
//             BOOLEAN DirentDeleted;
//
//             //
//             //  Read a dirent
//             //
//
//             FatReadDirent( IrpContext,
//                            Dcb,
//                            UnusedVbo,
//                            &Bcb,
//                            &Dirent,
//                            &Status );
//
//             //
//             //  If EOF, or we found a NEVER_USED entry, we exit the loop
//             //
//
//             if ( (Status == STATUS_END_OF_FILE ) ||
//                  (Dirent->FileName[0] == FAT_DIRENT_NEVER_USED)) {
//
//                 break;
//             }
//
//             //
//             //  If the dirent is DELETED, and it is the first one we found, set
//             //  it in the deleted hint.
//             //
//
//             if (Dirent->FileName[0] == FAT_DIRENT_DELETED) {
//
//                 DirentDeleted = TRUE;
//
//                 if (DeletedHint == 0xffffffff_u32) {
//
//                     DeletedHint = UnusedVbo;
//                 }
//
//             } else {
//
//                 DirentDeleted = FALSE;
//             }
//
//             //
//             //  Check for the first time through the loop, and determine
//             //  the current run type.
//             //
//
//             if (CurrentRun == InitialRun) {
//
//                 CurrentRun = DirentDeleted ?
//                              FreeDirents : AllocatedDirents;
//
//             } else {
//
//                 //
//                 //  Are we switching from a free run to an allocated run?
//                 //
//
//                 if ((CurrentRun == FreeDirents) && !DirentDeleted) {
//
//                     DirentsThisRun = DirentIndex - StartIndexOfThisRun;
//
//                     RtlClearBits( &Dcb->Specific.Dcb.FreeDirentBitmap,
//                                   StartIndexOfThisRun,
//                                   DirentsThisRun );
//
//                     CurrentRun = AllocatedDirents;
//                     StartIndexOfThisRun = DirentIndex;
//                 }
//
//                 //
//                 //  Are we switching from an allocated run to a free run?
//                 //
//
//                 if ((CurrentRun == AllocatedDirents) && DirentDeleted) {
//
//                     DirentsThisRun = DirentIndex - StartIndexOfThisRun;
//
//                     RtlSetBits( &Dcb->Specific.Dcb.FreeDirentBitmap,
//                                 StartIndexOfThisRun,
//                                 DirentsThisRun );
//
//                     CurrentRun = FreeDirents;
//                     StartIndexOfThisRun = DirentIndex;
//                 }
//             }
//
//             //
//             //  Move on to the next dirent.
//             //
//
//             UnusedVbo += sizeof(DIRENT);
//             Dirent += 1;
//             DirentIndex += 1;
//         }
//
//         //
//         //  Now we have to record the final run we encoutered
//         //
//
//         DirentsThisRun = DirentIndex - StartIndexOfThisRun;
//
//         if ((CurrentRun == FreeDirents) || (CurrentRun == InitialRun)) {
//
//             RtlClearBits( &Dcb->Specific.Dcb.FreeDirentBitmap,
//                           StartIndexOfThisRun,
//                           DirentsThisRun );
//
//         } else {
//
//             RtlSetBits( &Dcb->Specific.Dcb.FreeDirentBitmap,
//                         StartIndexOfThisRun,
//                         DirentsThisRun );
//         }
//
//         //
//         //  Now if there we bailed prematurely out of the loop because
//         //  we hit an unused entry, set all the rest as free.
//         //
//
//         if (UnusedVbo < Dcb->Header.AllocationSize.LowPart) {
//
//             StartIndexOfThisRun = UnusedVbo / sizeof(DIRENT);
//
//             DirentsThisRun = (Dcb->Header.AllocationSize.LowPart -
//                               UnusedVbo) / sizeof(DIRENT);
//
//             RtlClearBits( &Dcb->Specific.Dcb.FreeDirentBitmap,
//                           StartIndexOfThisRun,
//                           DirentsThisRun);
//         }
//
//     } _SEH2_FINALLY {
//
//         FatUnpinBcb( IrpContext, Bcb );
//     } _SEH2_END;
//
//     //
//     //  If there weren't any DELETED entries, set the index to our current
//     //  position.
//     //
//
//     if (DeletedHint == 0xffffffff_u32) { DeletedHint = UnusedVbo; }
//
//     Dcb->Specific.Dcb.UnusedDirentVbo = UnusedVbo;
//     Dcb->Specific.Dcb.DeletedDirentHint = DeletedHint;
//
//     return;
// }
//
//
// //
// //  Internal support routine
// //
//
// _Requires_lock_held_(_Global_critical_region_)
// ULONG
// FatDefragDirectory (
//     IN PIRP_CONTEXT IrpContext,
//     IN PDCB Dcb,
//     IN ULONG DirentsNeeded
//     )
//
// /*++
//
// Routine Description:
//
//     This routine determines if the requested number of dirents can be found
//     in the directory, looking for deleted dirents and orphaned LFNs.  If the
//     request can be satisifed, orphaned LFNs are marked as deleted, and deleted
//     dirents are all grouped together at the end of the directory.
//
//     Note that this routine is currently used only on the root directory, but
//     it is completely general and could be used on any directory.
//
// Arguments:
//
//     Dcb - Supplies the directory to defrag.
//
// Return Value:
//
//     The Index of the first dirent available for use, or -1 if the
//     request cannot be satisfied.
//
// --*/
//
// {
//     ULONG SavedIrpContextFlag;
//     PLIST_ENTRY Links;
//     ULONG ReturnValue = 0;
//     PFCB Fcb;
//
//     PBCB Bcb = NULL;
//     PDIRENT Dirent = NULL;
//     UNICODE_STRING Lfn = {0,0,NULL};
//
//     LARGE_MCB Mcb;
//     BOOLEAN McbInitialized = FALSE;
//     BOOLEAN InvalidateFcbs = FALSE;
//
//     PUCHAR Directory = NULL;
//     PUCHAR UnusedDirents;
//     PUCHAR UnusedDirentBuffer = NULL;
//     PUCHAR UsedDirents;
//     PUCHAR UsedDirentBuffer = NULL;
//
//     PBCB *Bcbs = NULL;
//     ULONG Page;
//     ULONG PagesPinned = 0;
//
//     ULONG DcbSize;
//     ULONG TotalBytesAllocated = 0;
//
//     PAGED_CODE();
//
//     //
//     //  We assume we own the Vcb.
//     //
//
//     NT_ASSERT( FatVcbAcquiredExclusive(IrpContext, Dcb->Vcb) );
//
//     //
//     //  We will only attempt this on directories less than 0x40000 bytes
//     //  long (by default on DOS the root directory is only 0x2000 long).
//     //  This is to avoid a cache manager complication.
//     //
//
//     DcbSize = Dcb->Header.AllocationSize.LowPart;
//
//     if (DcbSize > 0x40000) {
//
//         return (ULONG)-1;
//     }
//
//     //
//     //  Force wait to TRUE
//     //
//
//     SavedIrpContextFlag = IrpContext->Flags;
//
//     SetFlag( IrpContext->Flags,
//              IRP_CONTEXT_FLAG_WAIT | IRP_CONTEXT_FLAG_WRITE_THROUGH );
//
//     //
//     //  Now acquire all open Fcbs in the Dcb exclusive.
//     //
//
//     for (Links = Dcb->Specific.Dcb.ParentDcbQueue.Flink;
//          Links != &Dcb->Specific.Dcb.ParentDcbQueue;
//          Links = Links->Flink) {
//
//         Fcb = CONTAINING_RECORD( Links, FCB, ParentDcbLinks );
//
//         (VOID)ExAcquireResourceExclusiveLite( Fcb->Header.Resource, TRUE );
//     }
//
//     _SEH2_TRY {
//
//         CCB Ccb;
//         ULONG QueryOffset = 0;
//         ULONG FoundOffset = 0;
//         ULONGLONG BytesUsed = 0;
//
//         NTSTATUS DontCare;
//         ULONG Run;
//         ULONG TotalRuns;
//         BOOLEAN Result;
//         PUCHAR Char;
//
//         //
//         //  We are going to build a new bitmap that will show all orphaned
//         //  LFNs as well as deleted dirents as available.
//         //
//         //  Initialize our local CCB that will match all files and even
//         //  a label if it is here.
//         //
//
//         RtlZeroMemory( &Ccb, sizeof(CCB) );
//         Ccb.Flags = CCB_FLAG_MATCH_ALL | CCB_FLAG_MATCH_VOLUME_ID;
//
//         //
//         //  Init the Long File Name string.
//         //
//
//         Lfn.MaximumLength = 260 * sizeof(WCHAR);
//         Lfn.Buffer = FsRtlAllocatePoolWithTag( PagedPool,
//                                                260*sizeof(WCHAR),
//                                                TAG_FILENAME_BUFFER );
//
//         //
//         //  Initalize the Mcb.  We use this structure to keep track of runs
//         //  of free and allocated dirents.  Runs are identity allocations, and
//         //  holes are free dirents.
//         //
//
//         FsRtlInitializeLargeMcb( &Mcb, PagedPool );
//
//         McbInitialized = TRUE;
//
//         do {
//
//             FatLocateDirent( IrpContext,
//                              Dcb,
//                              &Ccb,
//                              QueryOffset,
//                              NULL,
//                              &Dirent,
//                              &Bcb,
//                              (PVBO)&FoundOffset,
//                              NULL,
//                              &Lfn,
//                              NULL );
//
//             if (Dirent != NULL) {
//
//                 ULONG LfnByteOffset;
//
//                 //
//                 //  Compute the LfnByteOffset.
//                 //
//
//                 LfnByteOffset = FoundOffset -
//                                 FAT_LFN_DIRENTS_NEEDED(&Lfn) * sizeof(LFN_DIRENT);
//
//                 BytesUsed = FoundOffset - LfnByteOffset + sizeof(DIRENT);
//
//                 //
//                 //  Set a run to represent all the dirents used for this
//                 //  file in the Dcb dir.
//                 //
//
// #ifdef _MSC_VER
// #pragma prefast( suppress:28931, "needed for debug build" )
// #endif
//                 Result = FsRtlAddLargeMcbEntry( &Mcb,
//                                                 LfnByteOffset,
//                                                 LfnByteOffset,
//                                                 BytesUsed );
//
//                 NT_ASSERT( Result );
//
//                 //
//                 //  Move on to the next dirent.
//                 //
//
//                 TotalBytesAllocated += (ULONG) BytesUsed;
//                 QueryOffset = FoundOffset + sizeof(DIRENT);
//             }
//
//         } while ((Dirent != NULL) && (QueryOffset < DcbSize));
//
//         if (Bcb != NULL) {
//
//             FatUnpinBcb( IrpContext, Bcb );
//         }
//
//         //
//         //  If we need more dirents than are available, bail.
//         //
//
//         if (DirentsNeeded > (DcbSize - TotalBytesAllocated)/sizeof(DIRENT)) {
//
//             try_return(ReturnValue = (ULONG)-1);
//         }
//
//         //
//         //  Now we are going to copy all the used and un-used parts of the
//         //  directory to separate pool.
//         //
//         //  Allocate these buffers and pin the entire directory.
//         //
//
//         UnusedDirents =
//         UnusedDirentBuffer = FsRtlAllocatePoolWithTag( PagedPool,
//                                                        DcbSize - TotalBytesAllocated,
//                                                        TAG_DIRENT );
//
//         UsedDirents =
//         UsedDirentBuffer = FsRtlAllocatePoolWithTag( PagedPool,
//                                                      TotalBytesAllocated,
//                                                      TAG_DIRENT );
//
//         PagesPinned = (DcbSize + (PAGE_SIZE - 1 )) / PAGE_SIZE;
//
//         Bcbs = FsRtlAllocatePoolWithTag( PagedPool,
//                                          PagesPinned * sizeof(PBCB),
//                                          TAG_BCB );
//
//         RtlZeroMemory( Bcbs, PagesPinned * sizeof(PBCB) );
//
//         for (Page = 0; Page < PagesPinned; Page += 1) {
//
//             ULONG PinSize;
//
//             //
//             //  Don't try to pin beyond the Dcb size.
//             //
//
//             if ((Page + 1) * PAGE_SIZE > DcbSize) {
//
//                 PinSize = DcbSize - (Page * PAGE_SIZE);
//
//             } else {
//
//                 PinSize = PAGE_SIZE;
//             }
//
//             FatPrepareWriteDirectoryFile( IrpContext,
//                                           Dcb,
//                                           Page * PAGE_SIZE,
//                                           PinSize,
//                                           &Bcbs[Page],
// #ifndef __REACTOS__
//                                           &Dirent,
// #else
//                                           (PVOID *)&Dirent,
// #endif
//                                           FALSE,
//                                           TRUE,
//                                           &DontCare );
//
//             if (Page == 0) {
//                 Directory = (PUCHAR)Dirent;
//             }
//         }
//
//         TotalRuns = FsRtlNumberOfRunsInLargeMcb( &Mcb );
//
//         for (Run = 0; Run < TotalRuns; Run++) {
//
//             LBO Vbo;
//             LBO Lbo;
//
// #ifdef _MSC_VER
// #pragma prefast( suppress:28931, "needed for debug build" )
// #endif
//             Result = FsRtlGetNextLargeMcbEntry( &Mcb,
//                                                 Run,
//                                                 &Vbo,
//                                                 &Lbo,
//                                                 (PLONGLONG)&BytesUsed );
//
//             NT_ASSERT(Result);
//
//             //
//             //  Copy each run to their specific pool.
//             //
//
//             if (Lbo != -1) {
//
//                 RtlCopyMemory( UsedDirents,
//                                Directory + Vbo,
//                                (ULONG) BytesUsed );
//
//                 UsedDirents += BytesUsed;
//
//             } else {
//
//                 RtlCopyMemory( UnusedDirents,
//                                Directory + Vbo,
//                                (ULONG) BytesUsed );
//
//                 UnusedDirents += BytesUsed;
//             }
//         }
//
//         //
//         //  Marking all the un-used dirents as "deleted".  This will reclaim
//         //  storage used by orphaned LFNs.
//         //
//
//         for (Char = UnusedDirentBuffer; Char < UnusedDirents; Char += sizeof(DIRENT)) {
//
//             *Char = FAT_DIRENT_DELETED;
//         }
//
//         //
//         //  Now, for the permanent step.  Copy the two pool buffer back to the
//         //  real Dcb directory, and flush the Dcb directory
//         //
//
//         NT_ASSERT( TotalBytesAllocated == (ULONG)(UsedDirents - UsedDirentBuffer) );
//
//         RtlCopyMemory( Directory, UsedDirentBuffer, TotalBytesAllocated );
//
//         RtlCopyMemory( Directory + TotalBytesAllocated,
//                        UnusedDirentBuffer,
//                        UnusedDirents - UnusedDirentBuffer );
//
//         //
//         //  We need to unpin here so that the UnpinRepinned won't deadlock.
//         //
//
//         if (Bcbs) {
//             for (Page = 0; Page < PagesPinned; Page += 1) {
//                 FatUnpinBcb( IrpContext, Bcbs[Page] );
//             }
//             ExFreePool(Bcbs);
//             Bcbs = NULL;
//         }
//
//         //
//         //  Now make the free dirent bitmap reflect the new state of the Dcb
//         //  directory.
//         //
//
//         RtlSetBits( &Dcb->Specific.Dcb.FreeDirentBitmap,
//                     0,
//                     TotalBytesAllocated / sizeof(DIRENT) );
//
//         RtlClearBits( &Dcb->Specific.Dcb.FreeDirentBitmap,
//                       TotalBytesAllocated / sizeof(DIRENT),
//                       (DcbSize - TotalBytesAllocated) / sizeof(DIRENT) );
//
//         ReturnValue = TotalBytesAllocated / sizeof(DIRENT);
//
//         //
//         //  Flush the directory to disk.  If we raise, we will need to invalidate
//         //  all of the children.  Sorry, guys, but I can't figure out where you are
//         //  now - if this failed I probably can't read the media either.  And we
//         //  probably purged the cache to boot.
//         //
//
//         _SEH2_TRY {
//
//             FatUnpinRepinnedBcbs( IrpContext );
//
//         } _SEH2_EXCEPT(FsRtlIsNtstatusExpected(_SEH2_GetExceptionCode()) ?
//                  EXCEPTION_EXECUTE_HANDLER : EXCEPTION_CONTINUE_SEARCH) {
//
//               InvalidateFcbs = TRUE;
//         } _SEH2_END;
//
//         //
//         //  OK, now nothing can go wrong.  We have two more things to do.
//         //  First, we have to fix up all the dirent offsets in any open Fcbs.
//         //  If we cannot now find the Fcb, the file is marked invalid.  Also,
//         //  we skip deleted files.
//         //
//
//         for (Links = Dcb->Specific.Dcb.ParentDcbQueue.Flink;
//              Links != &Dcb->Specific.Dcb.ParentDcbQueue;
//              Links = Links->Flink) {
//
//             PBCB TmpBcb = NULL;
//             ULONG TmpOffset = 0;
//             PDIRENT TmpDirent = NULL;
//             ULONG PreviousLfnSpread;
//
//             Fcb = CONTAINING_RECORD( Links, FCB, ParentDcbLinks );
//
//             if (IsFileDeleted( IrpContext, Fcb )) {
//
//                 continue;
//             }
//
//             //
//             //  If we aren't already giving up, safely try to pick up the dirent
//             //  to update the Fcb.  If this raises, we have to give up and blow
//             //  evenyone else away too.
//             //
//
//             if (!InvalidateFcbs) {
//
//                 _SEH2_TRY {
//
//                     FatLocateSimpleOemDirent( IrpContext,
//                                               Dcb,
//                                               &Fcb->ShortName.Name.Oem,
//                                               &TmpDirent,
//                                               &TmpBcb,
//                                               (PVBO)&TmpOffset );
//
//                 } _SEH2_EXCEPT(FsRtlIsNtstatusExpected(_SEH2_GetExceptionCode()) ?
//                          EXCEPTION_EXECUTE_HANDLER : EXCEPTION_CONTINUE_SEARCH) {
//
//                       InvalidateFcbs = TRUE;
//                 } _SEH2_END;
//             }
//
//             if (TmpBcb == NULL || InvalidateFcbs) {
//
//                 FatUnpinBcb( IrpContext, TmpBcb );
//                 FatMarkFcbCondition( IrpContext, Fcb, FcbBad, TRUE );
//
//             } else {
//
//                 FatUnpinBcb( IrpContext, TmpBcb );
//
//                 PreviousLfnSpread = Fcb->DirentOffsetWithinDirectory -
//                                     Fcb->LfnOffsetWithinDirectory;
//
//                 Fcb->DirentOffsetWithinDirectory = TmpOffset;
//                 Fcb->LfnOffsetWithinDirectory = TmpOffset - PreviousLfnSpread;
//             }
//         }
//
//     try_exit: NOTHING;
//     } _SEH2_FINALLY {
//
//         //
//         //  Free all our resources and stuff.
//         //
//
//         if (McbInitialized) {
//             FsRtlUninitializeLargeMcb( &Mcb );
//         }
//
//         if (Lfn.Buffer) {
//             ExFreePool( Lfn.Buffer );
//         }
//
//         if (UnusedDirentBuffer) {
//             ExFreePool( UnusedDirentBuffer );
//         }
//
//         if (UsedDirentBuffer) {
//             ExFreePool( UsedDirentBuffer );
//         }
//
//         if (Bcbs) {
//             for (Page = 0; Page < PagesPinned; Page += 1) {
//                 FatUnpinBcb( IrpContext, Bcbs[Page] );
//             }
//             ExFreePool(Bcbs);
//         }
//
//         FatUnpinBcb( IrpContext, Bcb );
//
//         for (Links = Dcb->Specific.Dcb.ParentDcbQueue.Flink;
//              Links != &Dcb->Specific.Dcb.ParentDcbQueue;
//              Links = Links->Flink) {
//
//             Fcb = CONTAINING_RECORD( Links, FCB, ParentDcbLinks );
//
//             ExReleaseResourceLite( Fcb->Header.Resource );
//         }
//
//         IrpContext->Flags = SavedIrpContextFlag;
//     } _SEH2_END;
//
//     //
//     //  Now return the offset of the first free dirent to the caller.
//     //
//
//     return ReturnValue;
// }
//
//
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
