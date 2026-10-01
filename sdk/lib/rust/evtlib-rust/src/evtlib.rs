//! evtlib — NT <=5.2 event-log file library (.evt format).
//!
//! Preamble sourced from the C header (evtlib.h) and c2rust output; all
//! file-scope struct/const/static definitions are here. Functions bodies
//! come from the LLM translation (triplet-assembled/evtlib/evtlib.rs) with
//! targeted fixes applied (see NOTES.md).
//!
//! ABI: every public Elf* function is `extern "system"` (#[no_mangle]) because
//! they are declared NTAPI in evtlib.h.  On i386 NTAPI = stdcall; on amd64/arm64
//! NTAPI = cdecl — `extern "system"` resolves correctly on all targets.
//!
//! Private helpers (ReadLogBuffer, WriteLogBuffer, ElfpOffsetByNumber,
//! ElfpAddOffsetInformation, ElfpDeleteOffsetInformation, ElfpInitNewFile,
//! ElfpInitExistingFile) are plain `unsafe fn` — no `extern {}` block needed
//! because this is a single-crate cohab; callers reference them by name.

#![allow(
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals,
    unused_variables,
    unused_mut,
    unused_assignments,
    dead_code,
    clippy::missing_safety_doc,
)]

use core::ffi::{c_char, c_void};

// ============================================================================
// External kernel functions
// ============================================================================

extern "C" {
    fn DbgPrint(format: *const c_char, ...) -> u32;
}

extern "system" {
    fn RtlFillMemoryUlong(Destination: *mut c_void, Length: usize, Fill: u32);
    fn RtlCopyMemory(Destination: *mut c_void, Source: *const c_void, Length: usize);
    fn RtlZeroMemory(Destination: *mut c_void, Length: usize);
    fn RtlCopyUnicodeString(
        DestinationString: *mut UNICODE_STRING,
        SourceString: *const UNICODE_STRING,
    );
    fn RtlCompareMemory(
        Source1: *const c_void,
        Source2: *const c_void,
        Length: usize,
    ) -> usize;
    fn RtlAssert(
        FailedAssertion: *const c_void,
        FileName: *const c_void,
        LineNumber: u32,
        Message: *mut c_char,
    );
}

#[inline]
unsafe fn RtlInitEmptyUnicodeString(
    DestinationString: *mut UNICODE_STRING,
    Buffer: *mut u16,
    MaximumLength: u16,
) {
    (*DestinationString).Length = 0;
    (*DestinationString).MaximumLength = MaximumLength;
    (*DestinationString).Buffer = Buffer;
}

// ============================================================================
// Types (preamble — from evtlib.h, c2rust-verified layouts)
// ============================================================================

/// UNICODE_STRING (ndk definition; we redeclare locally to avoid ntkrnl-sys
/// import complexity in this standalone lib).
#[repr(C)]
pub struct UNICODE_STRING {
    pub Length: u16,
    pub MaximumLength: u16,
    pub Buffer: *mut u16,
}
unsafe impl Send for UNICODE_STRING {}
unsafe impl Sync for UNICODE_STRING {}

/// On-disk event log file header (ELF_LOGFILE_HEADER, pack4).
#[repr(C, packed(4))]
pub struct EVENTLOGHEADER {
    pub HeaderSize: u32,
    pub Signature: u32,
    pub MajorVersion: u32,
    pub MinorVersion: u32,
    pub StartOffset: u32,
    pub EndOffset: u32,
    pub CurrentRecordNumber: u32,
    pub OldestRecordNumber: u32,
    pub MaxSize: u32,
    pub Flags: u32,
    pub Retention: u32,
    pub EndHeaderSize: u32,
}
pub type PEVENTLOGHEADER = *mut EVENTLOGHEADER;

/// On-disk event record header (EVENTLOGRECORD, pack4).
#[repr(C, packed(4))]
pub struct EVENTLOGRECORD {
    pub Length: u32,
    pub Reserved: u32,
    pub RecordNumber: u32,
    pub TimeGenerated: u32,
    pub TimeWritten: u32,
    pub EventID: u32,
    pub EventType: u16,
    pub NumStrings: u16,
    pub EventCategory: u16,
    pub ReservedFlags: u16,
    pub ClosingRecordNumber: u32,
    pub StringOffset: u32,
    pub UserSidLength: u32,
    pub UserSidOffset: u32,
    pub DataLength: u32,
    pub DataOffset: u32,
    // variable-length tail follows in the actual record
}
pub type PEVENTLOGRECORD = *mut EVENTLOGRECORD;

/// On-disk EOF record (ELF_EOF_RECORD, pack4).
/// Fields match the C header exactly:
///   RecordSizeBeginning, Ones, Twos, Threes, Fours, BeginRecord,
///   EndRecord, CurrentRecordNumber, OldestRecordNumber, RecordSizeEnd
#[repr(C, packed(4))]
pub struct EVENTLOGEOF {
    pub RecordSizeBeginning: u32,
    pub Ones: u32,
    pub Twos: u32,
    pub Threes: u32,
    pub Fours: u32,
    pub BeginRecord: u32,
    pub EndRecord: u32,
    pub CurrentRecordNumber: u32,
    pub OldestRecordNumber: u32,
    pub RecordSizeEnd: u32,
}
pub type PEVENTLOGEOF = *mut EVENTLOGEOF;

/// EVENT_OFFSET_INFO — per-record offset cache entry.
#[repr(C)]
pub struct EVENT_OFFSET_INFO {
    pub EventNumber: u32,
    pub EventOffset: u32,
}
pub type PEVENT_OFFSET_INFO = *mut EVENT_OFFSET_INFO;

// ============================================================================
// Callback typedefs (all NTAPI = extern "system")
// ============================================================================

pub type PELF_ALLOCATE_ROUTINE = Option<
    unsafe extern "system" fn(Size: usize, Flags: u32, Tag: u32) -> *mut c_void,
>;
pub type PELF_FREE_ROUTINE = Option<
    unsafe extern "system" fn(Ptr: *mut c_void, Flags: u32, Tag: u32),
>;
pub type PELF_FILE_READ_ROUTINE = Option<
    unsafe extern "system" fn(
        LogFile: *mut _EVTLOGFILE,
        FileOffset: *mut LARGE_INTEGER,
        Buffer: *mut c_void,
        Length: usize,
        ReadLength: *mut usize,
    ) -> i32,
>;
pub type PELF_FILE_WRITE_ROUTINE = Option<
    unsafe extern "system" fn(
        LogFile: *mut _EVTLOGFILE,
        FileOffset: *mut LARGE_INTEGER,
        Buffer: *mut c_void,
        Length: usize,
        WrittenLength: *mut usize,
    ) -> i32,
>;
pub type PELF_FILE_SET_SIZE_ROUTINE = Option<
    unsafe extern "system" fn(
        LogFile: *mut _EVTLOGFILE,
        FileSize: u32,
        OldFileSize: u32,
    ) -> i32,
>;
pub type PELF_FILE_FLUSH_ROUTINE = Option<
    unsafe extern "system" fn(
        LogFile: *mut _EVTLOGFILE,
        FileOffset: *mut LARGE_INTEGER,
        Length: u32,
    ) -> i32,
>;

/// LARGE_INTEGER (anonymous union; we use the QuadPart field only).
#[repr(C)]
#[derive(Copy, Clone)]
pub struct LARGE_INTEGER {
    pub QuadPart: i64,
}
pub type PLARGE_INTEGER = *mut LARGE_INTEGER;

/// EVTLOGFILE — the main log-file state structure.
#[repr(C)]
pub struct _EVTLOGFILE {
    pub Allocate: PELF_ALLOCATE_ROUTINE,
    pub Free: PELF_FREE_ROUTINE,
    pub FileSetSize: PELF_FILE_SET_SIZE_ROUTINE,
    pub FileWrite: PELF_FILE_WRITE_ROUTINE,
    pub FileRead: PELF_FILE_READ_ROUTINE,
    pub FileFlush: PELF_FILE_FLUSH_ROUTINE,
    pub Header: EVENTLOGHEADER,
    pub CurrentSize: u32,
    pub FileName: UNICODE_STRING,
    pub OffsetInfo: PEVENT_OFFSET_INFO,
    pub OffsetInfoSize: u32,
    pub OffsetInfoNext: u32,
    pub ReadOnly: u8,       // BOOLEAN
}
pub type EVTLOGFILE = _EVTLOGFILE;
pub type PEVTLOGFILE = *mut _EVTLOGFILE;

// ============================================================================
// Constants (sourced from evtlib.h; c2rust-verified values)
// ============================================================================

const MAJORVER: u32 = 1;
const MINORVER: u32 = 1;
const LOGFILE_SIGNATURE: u32 = 0x654c664c; // "LfLe"

const ELF_LOGFILE_HEADER_DIRTY: u32 = 1;
const ELF_LOGFILE_HEADER_WRAP: u32 = 2;
const ELF_LOGFILE_LOGFULL_WRITTEN: u32 = 4;
const ELF_LOGFILE_ARCHIVE_SET: u32 = 8;

/// EVENTLOGEOF_SIZE_FIXED = FIELD_OFFSET(EVENTLOGEOF, BeginRecord)
/// = 5 * sizeof(ULONG) = 20.
/// (The LLM wrongly used sizeof(EVENTLOGEOF)-4 = 36; that prevented
///  any existing log file from ever opening — see NOTES.md.)
const EVENTLOGEOF_SIZE_FIXED: usize = 5 * core::mem::size_of::<u32>(); // = 20

const OFFSET_INFO_INCREMENT: u32 = 64;
const HEAP_ZERO_MEMORY: u32 = 0x8; // standard HeapAlloc flag

/// Pool tags: ' flE' = 0x456c6620 and 'BflE' = 0x456c6642.
/// (The LLM used 0x464C4545 / 0 — both wrong; these are debug-only but
///  we fix them for correctness.)
const TAG_ELF: u32 = 0x456c6620;     // ' flE' reversed LE byte order
const TAG_ELF_BUF: u32 = 0x456c6642; // 'BflE'

/// STATUS_EVENTLOG_FILE_CORRUPT = 0xC000018E.
/// (The LLM used 0xC0000188 in InitExistingFile and 0xC0000189 in
///  WriteRecord — BOTH WRONG: 0x188=LOG_FILE_FULL, 0x189=TOO_LATE.)
const STATUS_EVENTLOG_FILE_CORRUPT: i32 = 0xC000018Eu32 as i32;

const STATUS_SUCCESS: i32 = 0;
const STATUS_NO_MEMORY: i32 = 0xC0000017u32 as i32;
const STATUS_ACCESS_DENIED: i32 = 0xC0000022u32 as i32;
const STATUS_INVALID_PARAMETER: i32 = 0xC000000Du32 as i32;
const STATUS_NOT_FOUND: i32 = 0xC0000225u32 as i32;
const STATUS_BUFFER_TOO_SMALL: i32 = 0xC0000023u32 as i32;
const STATUS_LOG_FILE_FULL: i32 = 0xC0000188u32 as i32; // correct: 0xC0000188

// ============================================================================
// File-scope global: the canonical EOF record template
// ============================================================================

static EOFRecord: EVENTLOGEOF = EVENTLOGEOF {
    RecordSizeBeginning: core::mem::size_of::<EVENTLOGEOF>() as u32,
    Ones:   0x11111111,
    Twos:   0x22222222,
    Threes: 0x33333333,
    Fours:  0x44444444,
    BeginRecord: 0,
    EndRecord: 0,
    CurrentRecordNumber: 0,
    OldestRecordNumber: 0,
    RecordSizeEnd: core::mem::size_of::<EVENTLOGEOF>() as u32,
};

// ============================================================================
// Helper macros (ROUND_UP / ROUND_DOWN — from evtlib.h, ULONG semantics)
// ============================================================================

#[inline(always)]
fn round_down_ulong(n: u32, align: u32) -> u32 {
    n & !(align - 1)
}

// Windows grows event-log FILES in 64 KiB steps; MaxSize is a cap, not the allocation
// (upstream 24e904e0956).
const EVENTLOG_FILE_GRANULARITY: u32 = 64 * 1024;

#[inline(always)]
fn round_up_ulong(n: u32, align: u32) -> u32 {
    round_down_ulong(n.wrapping_add(align - 1), align)
}

// ============================================================================
// Private helpers
// (No extern {} blocks — all called by path within this crate module.)
// ============================================================================

/// Read `Length` bytes from `ByteOffset` in the (possibly-wrapping) log file
/// into `Buffer`. On wrap the second segment is read from sizeof(EVENTLOGHEADER).
/// Returns the offset just past the end of the read data in `NextOffset`.
unsafe fn ReadLogBuffer(
    LogFile: *mut _EVTLOGFILE,
    Buffer: *mut c_void,
    Length: usize,
    ReadLength: *mut usize,
    ByteOffset: *const LARGE_INTEGER,
    NextOffset: *mut LARGE_INTEGER,
) -> i32 {
    let mut Status: i32;
    let mut BufSize: usize;
    let mut ReadBufLength: usize = 0;
    let mut OldReadBufLength: usize;

    if !ReadLength.is_null() {
        *ReadLength = 0;
    }
    if !NextOffset.is_null() {
        (*NextOffset).QuadPart = 0;
    }

    // Read first part
    let mut FileOffset: LARGE_INTEGER = *ByteOffset;
    let remaining = ((*LogFile).CurrentSize as i64)
        .wrapping_sub(FileOffset.QuadPart) as usize;
    BufSize = if Length < remaining { Length } else { remaining };

    Status = (*LogFile).FileRead.expect("non-null FileRead")(
        LogFile,
        &raw mut FileOffset,
        Buffer,
        BufSize,
        &raw mut ReadBufLength,
    );
    if Status < 0 {
        return Status;
    }

    if Length > BufSize {
        OldReadBufLength = ReadBufLength;

        // Buffer wraps: second segment at the beginning of the log data
        let buf_ptr = Buffer as usize;
        let buf2 = (buf_ptr.wrapping_add(BufSize)) as *mut c_void;
        BufSize = Length.wrapping_sub(BufSize);
        FileOffset.QuadPart = core::mem::size_of::<EVENTLOGHEADER>() as i64;

        Status = (*LogFile).FileRead.expect("non-null FileRead")(
            LogFile,
            &raw mut FileOffset,
            buf2,
            BufSize,
            &raw mut ReadBufLength,
        );
        if Status < 0 {
            return Status;
        }
        ReadBufLength = ReadBufLength.wrapping_add(OldReadBufLength);
    }

    if !ReadLength.is_null() {
        *ReadLength = ReadBufLength;
    }

    // NextOffset = offset just after end of read data (computed in i64, cast at end)
    if !NextOffset.is_null() {
        (*NextOffset).QuadPart =
            (FileOffset.QuadPart as i64).wrapping_add(BufSize as i64);
    }

    Status
}

/// Write `Length` bytes from `Buffer` to `ByteOffset` in the (possibly-wrapping)
/// log file. On wrap the second segment is written at sizeof(EVENTLOGHEADER).
/// Sets the HEADER_WRAP flag if the write wraps.
unsafe fn WriteLogBuffer(
    LogFile: *mut _EVTLOGFILE,
    Buffer: *mut c_void,
    Length: usize,
    WrittenLength: *mut usize,
    ByteOffset: *mut LARGE_INTEGER,
    NextOffset: *mut LARGE_INTEGER,
) -> i32 {
    let mut Status: i32;
    let mut BufSize: usize;
    let mut WrittenBufLength: usize = 0;
    let mut OldWrittenBufLength: usize;

    if !WrittenLength.is_null() {
        *WrittenLength = 0;
    }
    if !NextOffset.is_null() {
        (*NextOffset).QuadPart = 0;
    }

    let mut FileOffset: LARGE_INTEGER = *ByteOffset;
    let remaining = ((*LogFile).CurrentSize as i64)
        .wrapping_sub(FileOffset.QuadPart) as usize;
    BufSize = if Length < remaining { Length } else { remaining };

    let file_write = (*LogFile).FileWrite.expect("non-null FileWrite");
    Status = file_write(
        LogFile,
        &raw mut FileOffset,
        Buffer,
        BufSize,
        &raw mut WrittenBufLength,
    );
    if Status < 0 {
        return Status;
    }

    if Length > BufSize {
        OldWrittenBufLength = WrittenBufLength;

        let buf2 = (Buffer as usize).wrapping_add(BufSize) as *mut c_void;
        BufSize = Length.wrapping_sub(BufSize);
        FileOffset.QuadPart = core::mem::size_of::<EVENTLOGHEADER>() as i64;

        Status = file_write(
            LogFile,
            &raw mut FileOffset,
            buf2,
            BufSize,
            &raw mut WrittenBufLength,
        );
        if Status < 0 {
            return Status;
        }

        WrittenBufLength = WrittenBufLength.wrapping_add(OldWrittenBufLength);
        (*LogFile).Header.Flags |= ELF_LOGFILE_HEADER_WRAP;
    }

    if !WrittenLength.is_null() {
        *WrittenLength = WrittenBufLength;
    }

    // NextOffset computed in i64 to avoid 32-bit truncation (i386 fix)
    if !NextOffset.is_null() {
        (*NextOffset).QuadPart =
            (FileOffset.QuadPart as i64).wrapping_add(BufSize as i64);
    }

    Status
}

/// Returns the file offset for record `RecordNumber`, or 0 if not found.
unsafe fn ElfpOffsetByNumber(
    LogFile: *mut _EVTLOGFILE,
    RecordNumber: u32,
) -> u32 {
    for i in 0..(*LogFile).OffsetInfoNext {
        let entry = (*LogFile).OffsetInfo.add(i as usize);
        if (*entry).EventNumber == RecordNumber {
            return (*entry).EventOffset;
        }
    }
    0
}

/// Appends an (EventNumber, EventOffset) pair to the offset table, growing
/// it by OFFSET_INFO_INCREMENT if needed. Returns TRUE (1) on success.
unsafe fn ElfpAddOffsetInformation(
    LogFile: *mut _EVTLOGFILE,
    ulNumber: u32,
    ulOffset: u32,
) -> i32 {
    if (*LogFile).OffsetInfoNext == (*LogFile).OffsetInfoSize {
        let new_count = ((*LogFile).OffsetInfoSize as usize)
            .wrapping_add(OFFSET_INFO_INCREMENT as usize);
        let new_bytes = new_count
            .wrapping_mul(core::mem::size_of::<EVENT_OFFSET_INFO>());

        let NewOffsetInfo: *mut c_void = (*LogFile)
            .Allocate
            .expect("non-null Allocate")(
                new_bytes,
                HEAP_ZERO_MEMORY,
                TAG_ELF,
            );
        if NewOffsetInfo.is_null() {
            return 0; // FALSE
        }

        if !(*LogFile).OffsetInfo.is_null() {
            let copy_bytes = ((*LogFile).OffsetInfoSize as usize)
                .wrapping_mul(core::mem::size_of::<EVENT_OFFSET_INFO>());
            core::ptr::copy_nonoverlapping(
                (*LogFile).OffsetInfo as *const u8,
                NewOffsetInfo as *mut u8,
                copy_bytes,
            );
            (*LogFile).Free.expect("non-null Free")(
                (*LogFile).OffsetInfo as *mut c_void,
                0,
                TAG_ELF,
            );
        }

        (*LogFile).OffsetInfo = NewOffsetInfo as PEVENT_OFFSET_INFO;
        (*LogFile).OffsetInfoSize = (*LogFile)
            .OffsetInfoSize
            .wrapping_add(OFFSET_INFO_INCREMENT);
    }

    let slot = (*LogFile).OffsetInfo.add((*LogFile).OffsetInfoNext as usize);
    (*slot).EventNumber = ulNumber;
    (*slot).EventOffset = ulOffset;
    (*LogFile).OffsetInfoNext = (*LogFile).OffsetInfoNext.wrapping_add(1);
    1 // TRUE
}

/// Removes offset entries for records [ulNumberMin, ulNumberMax] (inclusive).
/// The list must be contiguous and ulNumberMin must be the first element.
/// Returns FALSE (0) on validation failure.
unsafe fn ElfpDeleteOffsetInformation(
    LogFile: *mut _EVTLOGFILE,
    mut ulNumberMin: u32,
    ulNumberMax: u32,
) -> i32 {
    if ulNumberMin > ulNumberMax {
        return 0;
    }
    while ulNumberMin <= ulNumberMax {
        if (*(*LogFile).OffsetInfo).EventNumber != ulNumberMin {
            return 0;
        }
        // Shift entries down
        let count = (*LogFile).OffsetInfoNext.wrapping_sub(1) as usize;
        for i in 0..count {
            let src = (*LogFile).OffsetInfo.add(i + 1);
            let dst = (*LogFile).OffsetInfo.add(i);
            (*dst).EventNumber = (*src).EventNumber;
            (*dst).EventOffset = (*src).EventOffset;
        }
        (*LogFile).OffsetInfoNext = (*LogFile).OffsetInfoNext.wrapping_sub(1);
        ulNumberMin = ulNumberMin.wrapping_add(1);
    }
    1 // TRUE
}

/// Initialize a new (empty) event log file: writes header + initial EOF record.
unsafe fn ElfpInitNewFile(
    LogFile: *mut _EVTLOGFILE,
    FileSize: u32,
    MaxSize: u32,
    Retention: u32,
) -> i32 {
    let mut Status: i32;
    let mut FileOffset: LARGE_INTEGER = LARGE_INTEGER { QuadPart: 0 };
    let mut WrittenLength: usize = 0;
    let mut EofRec: EVENTLOGEOF = core::mem::zeroed();

    // Zero the header
    core::ptr::write_bytes(
        core::ptr::addr_of_mut!((*LogFile).Header) as *mut u8,
        0,
        core::mem::size_of::<EVENTLOGHEADER>(),
    );

    (*LogFile).Header.HeaderSize = core::mem::size_of::<EVENTLOGHEADER>() as u32;
    (*LogFile).Header.Signature = LOGFILE_SIGNATURE;
    (*LogFile).Header.MajorVersion = MAJORVER;
    (*LogFile).Header.MinorVersion = MINORVER;
    (*LogFile).Header.StartOffset = core::mem::size_of::<EVENTLOGHEADER>() as u32;
    (*LogFile).Header.EndOffset = core::mem::size_of::<EVENTLOGHEADER>() as u32;
    (*LogFile).Header.CurrentRecordNumber = 1;
    (*LogFile).Header.OldestRecordNumber = 0;

    // Start at the header + EOF record rounded to the 64 KiB granularity, not at MaxSize
    // (upstream 24e904e0956): the old code preallocated every log to its full cap and
    // ignored a FileSetSize failure.
    let InitialSize = round_up_ulong(
        (core::mem::size_of::<EVENTLOGHEADER>() + core::mem::size_of::<EVENTLOGEOF>()) as u32,
        EVENTLOG_FILE_GRANULARITY,
    );
    (*LogFile).CurrentSize = InitialSize;
    Status = (*LogFile).FileSetSize.unwrap()(LogFile, (*LogFile).CurrentSize, FileSize);
    if Status < 0 {
        return Status;
    }

    // The cap is at least the initial file size.
    (*LogFile).Header.MaxSize = round_up_ulong(MaxSize.max(InitialSize), EVENTLOG_FILE_GRANULARITY);

    (*LogFile).Header.Flags = 0;
    (*LogFile).Header.Retention = Retention;
    (*LogFile).Header.EndHeaderSize = core::mem::size_of::<EVENTLOGHEADER>() as u32;

    // Write header
    FileOffset.QuadPart = 0;
    Status = (*LogFile).FileWrite.unwrap()(
        LogFile,
        &raw mut FileOffset,
        core::ptr::addr_of_mut!((*LogFile).Header) as *mut c_void,
        core::mem::size_of::<EVENTLOGHEADER>(),
        &raw mut WrittenLength,
    );
    if Status < 0 {
        return Status;
    }

    // Initialize and write EOF record
    core::ptr::copy_nonoverlapping(
        &EOFRecord as *const EVENTLOGEOF as *const u8,
        &raw mut EofRec as *mut u8,
        core::mem::size_of::<EVENTLOGEOF>(),
    );
    EofRec.BeginRecord = (*LogFile).Header.StartOffset;
    EofRec.EndRecord = (*LogFile).Header.EndOffset;
    EofRec.CurrentRecordNumber = (*LogFile).Header.CurrentRecordNumber;
    EofRec.OldestRecordNumber = (*LogFile).Header.OldestRecordNumber;

    Status = (*LogFile).FileWrite.unwrap()(
        LogFile,
        core::ptr::null_mut::<LARGE_INTEGER>(),
        &raw mut EofRec as *mut c_void,
        core::mem::size_of::<EVENTLOGEOF>(),
        &raw mut WrittenLength,
    );
    if Status < 0 {
        return Status;
    }

    Status = (*LogFile).FileFlush.unwrap()(
        LogFile,
        core::ptr::null_mut::<LARGE_INTEGER>(),
        0,
    );
    if Status < 0 {
        return Status;
    }

    STATUS_SUCCESS
}

/// Open and validate an existing event log file: reads header, locates EOF
/// record, rebuilds offset table, flushes if dirty.
unsafe fn ElfpInitExistingFile(
    LogFile: *mut _EVTLOGFILE,
    FileSize: u32,
    Retention: u32,
) -> i32 {
    let mut Status: i32;
    let mut FileOffset: LARGE_INTEGER = LARGE_INTEGER { QuadPart: 0 };
    let mut NextOffset: LARGE_INTEGER = LARGE_INTEGER { QuadPart: 0 };
    let mut ReadLength: usize = 0;
    let mut RecordNumber: u32 = 0;
    let mut RecOffset: u32;
    let mut pRecSize2: *mut u32;
    let mut EofRec: EVENTLOGEOF = core::mem::zeroed();
    let mut RecBuf: EVENTLOGRECORD = core::mem::zeroed();
    let mut pRecBuf: *mut EVENTLOGRECORD;
    let mut Wrapping: bool = false;
    let mut IsLogDirty: bool;

    // Read the log header
    FileOffset.QuadPart = 0;
    Status = (*LogFile).FileRead.expect("non-null FileRead")(
        LogFile,
        &raw mut FileOffset,
        core::ptr::addr_of_mut!((*LogFile).Header) as *mut c_void,
        core::mem::size_of::<EVENTLOGHEADER>(),
        &raw mut ReadLength,
    );
    if Status < 0 {
        return STATUS_EVENTLOG_FILE_CORRUPT;
    }
    if ReadLength != core::mem::size_of::<EVENTLOGHEADER>() {
        return STATUS_EVENTLOG_FILE_CORRUPT;
    }

    // Header validity
    if (*LogFile).Header.HeaderSize != core::mem::size_of::<EVENTLOGHEADER>() as u32
        || (*LogFile).Header.EndHeaderSize != core::mem::size_of::<EVENTLOGHEADER>() as u32
    {
        return STATUS_EVENTLOG_FILE_CORRUPT;
    }
    if (*LogFile).Header.Signature != LOGFILE_SIGNATURE {
        return STATUS_EVENTLOG_FILE_CORRUPT;
    }

    IsLogDirty = ((*LogFile).Header.Flags & ELF_LOGFILE_HEADER_DIRTY) != 0;

    if (*LogFile).ReadOnly != 0 && IsLogDirty {
        return STATUS_EVENTLOG_FILE_CORRUPT;
    }

    (*LogFile).CurrentSize = FileSize;
    if (*LogFile).CurrentSize > (*LogFile).Header.MaxSize {
        (*LogFile).Header.MaxSize = (*LogFile).CurrentSize;
    }
    (*LogFile).Header.Retention = Retention;

    // Determine starting search offset for EOF record
    if (*LogFile).Header.EndOffset >= core::mem::size_of::<EVENTLOGHEADER>() as u32
        && (*LogFile).Header.EndOffset < (*LogFile).CurrentSize
        && ((*LogFile).Header.EndOffset & 3) == 0
    {
        RecOffset = (*LogFile).Header.EndOffset;
    } else {
        RecOffset = core::mem::size_of::<EVENTLOGHEADER>() as u32;
    }

    FileOffset.QuadPart = RecOffset as i64;
    Wrapping = false;

    // ---- Locate EVENTLOGEOF record by scanning ----
    loop {
        if Wrapping && FileOffset.QuadPart >= RecOffset as i64 {
            return STATUS_EVENTLOG_FILE_CORRUPT;
        }

        // Read the fixed part (5 ULONGs = 20 bytes = EVENTLOGEOF_SIZE_FIXED)
        Status = ReadLogBuffer(
            LogFile,
            core::ptr::addr_of_mut!(EofRec) as *mut c_void,
            EVENTLOGEOF_SIZE_FIXED, // 20 — FIXED (was wrongly 36 in LLM output)
            &raw mut ReadLength,
            &raw const FileOffset,
            core::ptr::null_mut(),
        );
        if Status < 0 {
            return STATUS_EVENTLOG_FILE_CORRUPT;
        }
        if ReadLength != EVENTLOGEOF_SIZE_FIXED {
            return STATUS_EVENTLOG_FILE_CORRUPT;
        }

        // Compare the fixed prefix against the canonical EOF record template
        if RtlCompareMemory(
            core::ptr::addr_of!(EofRec) as *const c_void,
            core::ptr::addr_of!(EOFRecord) as *const c_void,
            EVENTLOGEOF_SIZE_FIXED,
        ) == EVENTLOGEOF_SIZE_FIXED
        {
            // Found it
            break;
        }

        // Advance the search by scanning for the first ULONG of EOFRecord
        let eof_base = core::ptr::addr_of!(EofRec) as usize;
        let ref_val = *(core::ptr::addr_of!(EOFRecord) as *const u32);
        let advance: i64;
        if *(eof_base.wrapping_add(1 * core::mem::size_of::<u32>()) as *const u32) == ref_val {
            advance = core::mem::size_of::<u32>() as i64;
        } else if *(eof_base.wrapping_add(2 * core::mem::size_of::<u32>()) as *const u32) == ref_val {
            advance = 2 * core::mem::size_of::<u32>() as i64;
        } else if *(eof_base.wrapping_add(3 * core::mem::size_of::<u32>()) as *const u32) == ref_val {
            advance = 3 * core::mem::size_of::<u32>() as i64;
        } else if *(eof_base.wrapping_add(4 * core::mem::size_of::<u32>()) as *const u32) == ref_val {
            advance = 4 * core::mem::size_of::<u32>() as i64;
        } else {
            advance = EVENTLOGEOF_SIZE_FIXED as i64; // 5 * sizeof(u32)
        }
        FileOffset.QuadPart = FileOffset.QuadPart.wrapping_add(advance);

        if FileOffset.QuadPart >= (*LogFile).CurrentSize as i64 {
            FileOffset.QuadPart = FileOffset.QuadPart.wrapping_sub(
                ((*LogFile).CurrentSize as i64)
                    .wrapping_sub(core::mem::size_of::<EVENTLOGHEADER>() as i64),
            );
            Wrapping = true;
        }
    }

    // Read the full EVENTLOGEOF (10 ULONGs = 40 bytes)
    Status = ReadLogBuffer(
        LogFile,
        core::ptr::addr_of_mut!(EofRec) as *mut c_void,
        core::mem::size_of::<EVENTLOGEOF>(),
        &raw mut ReadLength,
        &raw const FileOffset,
        core::ptr::null_mut(),
    );
    if Status < 0 {
        return STATUS_EVENTLOG_FILE_CORRUPT;
    }
    if ReadLength != core::mem::size_of::<EVENTLOGEOF>() {
        return STATUS_EVENTLOG_FILE_CORRUPT;
    }

    // Validate EOF record
    if EofRec.RecordSizeEnd != EofRec.RecordSizeBeginning
        || EofRec.EndRecord as i64 != FileOffset.QuadPart
    {
        return STATUS_EVENTLOG_FILE_CORRUPT;
    }

    // If log is clean, header values must match EOF
    if !IsLogDirty {
        if (*LogFile).Header.StartOffset != EofRec.BeginRecord
            || (*LogFile).Header.EndOffset != EofRec.EndRecord
            || (*LogFile).Header.CurrentRecordNumber != EofRec.CurrentRecordNumber
            || (*LogFile).Header.OldestRecordNumber != EofRec.OldestRecordNumber
        {
            return STATUS_EVENTLOG_FILE_CORRUPT;
        }
    }

    // If dirty, patch header from EOF
    if (*LogFile).ReadOnly == 0 && IsLogDirty {
        (*LogFile).Header.StartOffset = EofRec.BeginRecord;
        (*LogFile).Header.EndOffset = EofRec.EndRecord;
        (*LogFile).Header.CurrentRecordNumber = EofRec.CurrentRecordNumber;
        (*LogFile).Header.OldestRecordNumber = EofRec.OldestRecordNumber;
    }

    // Offset boundary checks
    if (*LogFile).Header.StartOffset >= (*LogFile).CurrentSize
        || ((*LogFile).Header.StartOffset & 3) != 0
    {
        return STATUS_EVENTLOG_FILE_CORRUPT;
    }
    if (*LogFile).Header.EndOffset >= (*LogFile).CurrentSize
        || ((*LogFile).Header.EndOffset & 3) != 0
    {
        return STATUS_EVENTLOG_FILE_CORRUPT;
    }
    if (*LogFile).Header.StartOffset != (*LogFile).Header.EndOffset
        && ((*LogFile).Header.MaxSize.wrapping_sub((*LogFile).Header.StartOffset) as usize)
            < core::mem::size_of::<EVENTLOGRECORD>()
    {
        return STATUS_EVENTLOG_FILE_CORRUPT;
    }
    if (*LogFile).Header.StartOffset < (*LogFile).Header.EndOffset
        && ((*LogFile).Header.EndOffset.wrapping_sub((*LogFile).Header.StartOffset) as usize)
            < core::mem::size_of::<EVENTLOGRECORD>()
    {
        return STATUS_EVENTLOG_FILE_CORRUPT;
    }
    if (*LogFile).Header.StartOffset <= (*LogFile).Header.EndOffset {
        let free_wrap = (*LogFile).Header.MaxSize
            .wrapping_sub((*LogFile).Header.EndOffset)
            .wrapping_add((*LogFile).Header.StartOffset)
            .wrapping_sub(core::mem::size_of::<EVENTLOGHEADER>() as u32);
        if (free_wrap as usize) < core::mem::size_of::<EVENTLOGEOF>() {
            return STATUS_EVENTLOG_FILE_CORRUPT;
        }
    } else {
        if ((*LogFile).Header.StartOffset.wrapping_sub((*LogFile).Header.EndOffset) as usize)
            < core::mem::size_of::<EVENTLOGEOF>()
        {
            return STATUS_EVENTLOG_FILE_CORRUPT;
        }
    }

    // ---- Enumerate records and rebuild offset table ----
    RecOffset = (*LogFile).Header.StartOffset;
    FileOffset.QuadPart = RecOffset as i64;
    Wrapping = false;

    while FileOffset.QuadPart != (*LogFile).Header.EndOffset as i64 {
        if Wrapping && FileOffset.QuadPart >= RecOffset as i64 {
            break;
        }

        // Read header (cannot be split)
        Status = (*LogFile).FileRead.expect("non-null FileRead")(
            LogFile,
            &raw mut FileOffset,
            &raw mut RecBuf as *mut c_void,
            core::mem::size_of::<EVENTLOGRECORD>(),
            &raw mut ReadLength,
        );
        if Status < 0 {
            return STATUS_EVENTLOG_FILE_CORRUPT;
        }
        if ReadLength != core::mem::size_of::<EVENTLOGRECORD>() {
            break;
        }
        if RecBuf.Reserved != LOGFILE_SIGNATURE
            || RecBuf.Length < core::mem::size_of::<EVENTLOGRECORD>() as u32
        {
            break;
        }

        // Allocate full record buffer
        pRecBuf = (*LogFile)
            .Allocate
            .expect("non-null Allocate")(
                RecBuf.Length as usize,
                0,
                TAG_ELF_BUF,
            ) as *mut EVENTLOGRECORD;
        if pRecBuf.is_null() {
            return STATUS_NO_MEMORY;
        }

        // Read full record (may wrap)
        Status = ReadLogBuffer(
            LogFile,
            pRecBuf as *mut c_void,
            RecBuf.Length as usize,
            &raw mut ReadLength,
            &raw const FileOffset,
            &raw mut NextOffset,
        );
        if Status < 0 {
            (*LogFile).Free.expect("non-null Free")(
                pRecBuf as *mut c_void, 0, TAG_ELF_BUF,
            );
            return STATUS_EVENTLOG_FILE_CORRUPT;
        }
        if ReadLength != RecBuf.Length as usize {
            (*LogFile).Free.expect("non-null Free")(
                pRecBuf as *mut c_void, 0, TAG_ELF_BUF,
            );
            break;
        }

        // Validate trailing size field
        pRecSize2 = (pRecBuf as usize)
            .wrapping_add(RecBuf.Length as usize)
            .wrapping_sub(4) as *mut u32;
        if *pRecSize2 != RecBuf.Length {
            (*LogFile).Free.expect("non-null Free")(
                pRecBuf as *mut c_void, 0, TAG_ELF_BUF,
            );
            break;
        }

        RecordNumber = RecordNumber.wrapping_add(1);

        if ElfpAddOffsetInformation(
            LogFile,
            (*pRecBuf).RecordNumber,
            FileOffset.QuadPart as u32,
        ) == 0
        {
            (*LogFile).Free.expect("non-null Free")(
                pRecBuf as *mut c_void, 0, TAG_ELF_BUF,
            );
            return STATUS_EVENTLOG_FILE_CORRUPT;
        }

        (*LogFile).Free.expect("non-null Free")(
            pRecBuf as *mut c_void, 0, TAG_ELF_BUF,
        );

        if NextOffset.QuadPart == (*LogFile).Header.EndOffset as i64 {
            break;
        }

        // Wrap if near end of file
        if ((*LogFile).Header.MaxSize as i64).wrapping_sub(NextOffset.QuadPart)
            < core::mem::size_of::<EVENTLOGRECORD>() as i64
        {
            NextOffset.QuadPart = core::mem::size_of::<EVENTLOGHEADER>() as i64;
        }

        if FileOffset.QuadPart > NextOffset.QuadPart {
            Wrapping = true;
        }

        FileOffset = NextOffset;
    }

    if RecordNumber != 0 && (*LogFile).Header.OldestRecordNumber == 0 {
        (*LogFile).Header.OldestRecordNumber = 1;
    }
    (*LogFile).Header.CurrentRecordNumber =
        RecordNumber.wrapping_add((*LogFile).Header.OldestRecordNumber);
    if (*LogFile).Header.CurrentRecordNumber == 0 {
        (*LogFile).Header.CurrentRecordNumber = 1;
    }

    // Flush if writable
    if (*LogFile).ReadOnly == 0 {
        Status = ElfFlushFile(LogFile);
        if Status < 0 {
            return STATUS_EVENTLOG_FILE_CORRUPT;
        }
    }

    STATUS_SUCCESS
}

// ============================================================================
// Public API (all NTAPI = extern "system")
// ============================================================================

/// Create or open an event log file.
#[no_mangle]
pub unsafe extern "system" fn ElfCreateFile(
    LogFile: *mut _EVTLOGFILE,
    FileName: *mut UNICODE_STRING,
    FileSize: u32,
    MaxSize: u32,
    Retention: u32,
    CreateNew: u8,   // BOOLEAN
    ReadOnly: u8,    // BOOLEAN
    Allocate: PELF_ALLOCATE_ROUTINE,
    Free: PELF_FREE_ROUTINE,
    FileSetSize: PELF_FILE_SET_SIZE_ROUTINE,
    FileWrite: PELF_FILE_WRITE_ROUTINE,
    FileRead: PELF_FILE_READ_ROUTINE,
    FileFlush: PELF_FILE_FLUSH_ROUTINE,
) -> i32 {
    // ASSERT(LogFile)
    if LogFile.is_null() {
        RtlAssert(
            b"LogFile\0".as_ptr() as *const c_void,
            b"evtlib.c\0".as_ptr() as *const c_void,
            893,
            core::ptr::null_mut(),
        );
    }

    // Creating a new read-only log is invalid
    if CreateNew != 0 && ReadOnly != 0 {
        return STATUS_INVALID_PARAMETER;
    }

    RtlZeroMemory(LogFile as *mut c_void, core::mem::size_of::<_EVTLOGFILE>());

    (*LogFile).Allocate = Allocate;
    (*LogFile).Free = Free;
    (*LogFile).FileSetSize = FileSetSize;
    (*LogFile).FileWrite = FileWrite;
    (*LogFile).FileRead = FileRead;
    (*LogFile).FileFlush = FileFlush;

    // Initialize FileName to empty
    RtlInitEmptyUnicodeString(
        &raw mut (*LogFile).FileName,
        core::ptr::null_mut::<u16>(),
        0,
    );

    // Copy file name if provided
    if !FileName.is_null()
        && !(*FileName).Buffer.is_null()
        && (*FileName).Length != 0
        && (*FileName).Length <= (*FileName).MaximumLength
    {
        let buf = (*LogFile).Allocate.expect("non-null Allocate")(
            (*FileName).Length as usize,
            HEAP_ZERO_MEMORY,
            TAG_ELF,
        ) as *mut u16;
        if !buf.is_null() {
            (*LogFile).FileName.MaximumLength = (*FileName).Length;
            (*LogFile).FileName.Buffer = buf;
            RtlCopyUnicodeString(
                &raw mut (*LogFile).FileName,
                FileName as *const UNICODE_STRING,
            );
        }
    }

    // Allocate offset table
    let oi = (*LogFile).Allocate.expect("non-null Allocate")(
        (OFFSET_INFO_INCREMENT as usize)
            .wrapping_mul(core::mem::size_of::<EVENT_OFFSET_INFO>()),
        HEAP_ZERO_MEMORY,
        TAG_ELF,
    ) as PEVENT_OFFSET_INFO;

    let mut Status: i32;
    if oi.is_null() {
        Status = STATUS_NO_MEMORY;
    } else {
        (*LogFile).OffsetInfo = oi;
        (*LogFile).OffsetInfoSize = OFFSET_INFO_INCREMENT;
        (*LogFile).OffsetInfoNext = 0;
        (*LogFile).ReadOnly = ReadOnly;

        if CreateNew != 0 {
            Status = ElfpInitNewFile(LogFile, FileSize, MaxSize, Retention);
        } else {
            Status = ElfpInitExistingFile(LogFile, FileSize, Retention);
        }
    }

    // Cleanup on failure
    if Status < 0 {
        // ElfCreateFile `Status >= 0` comparison fix: plain i32 comparison, no i64 cast
        if !(*LogFile).OffsetInfo.is_null() {
            (*LogFile).Free.expect("non-null Free")(
                (*LogFile).OffsetInfo as *mut c_void, 0, TAG_ELF,
            );
        }
        if !(*LogFile).FileName.Buffer.is_null() {
            (*LogFile).Free.expect("non-null Free")(
                (*LogFile).FileName.Buffer as *mut c_void, 0, TAG_ELF,
            );
        }
    }

    Status
}

/// Re-create an already-opened log file (clear and re-initialize).
#[no_mangle]
pub unsafe extern "system" fn ElfReCreateFile(
    LogFile: *mut _EVTLOGFILE,
) -> i32 {
    if LogFile.is_null() {
        RtlAssert(
            b"LogFile\0".as_ptr() as *const c_void,
            b"evtlib.c\0".as_ptr() as *const c_void,
            969,
            core::ptr::null_mut(),
        );
    }
    ElfpInitNewFile(
        LogFile,
        (*LogFile).CurrentSize,
        (*LogFile).Header.MaxSize,
        (*LogFile).Header.Retention,
    )
}

/// Flush the event log header and underlying file.
#[no_mangle]
pub unsafe extern "system" fn ElfFlushFile(
    LogFile: *mut _EVTLOGFILE,
) -> i32 {
    if LogFile.is_null() {
        RtlAssert(
            b"LogFile\0".as_ptr() as *const c_void,
            b"evtlib.c\0".as_ptr() as *const c_void,
            1141,
            core::ptr::null_mut(),
        );
    }

    if (*LogFile).ReadOnly != 0 {
        return STATUS_SUCCESS;
    }

    // Remove the dirty flag
    (*LogFile).Header.Flags &= !ELF_LOGFILE_HEADER_DIRTY;

    let mut FileOffset: LARGE_INTEGER = LARGE_INTEGER { QuadPart: 0 };
    let mut WrittenLength: usize = 0;

    let Status = (*LogFile).FileWrite.expect("non-null FileWrite")(
        LogFile,
        &raw mut FileOffset,
        core::ptr::addr_of_mut!((*LogFile).Header) as *mut c_void,
        core::mem::size_of::<EVENTLOGHEADER>(),
        &raw mut WrittenLength,
    );
    if Status < 0 {
        return Status;
    }

    let Status = (*LogFile).FileFlush.expect("non-null FileFlush")(
        LogFile,
        core::ptr::null_mut::<LARGE_INTEGER>(),
        0,
    );
    if Status < 0 {
        return Status;
    }

    STATUS_SUCCESS
}

/// Close an event log file (flush + free resources).
#[no_mangle]
pub unsafe extern "system" fn ElfCloseFile(
    LogFile: *mut _EVTLOGFILE,
) {
    if LogFile.is_null() {
        RtlAssert(
            b"LogFile\0".as_ptr() as *const c_void,
            b"evtlib.c\0".as_ptr() as *const c_void,
            1182,
            core::ptr::null_mut(),
        );
    }

    ElfFlushFile(LogFile);

    (*LogFile).Free.expect("non-null Free")(
        (*LogFile).OffsetInfo as *mut c_void,
        0,
        TAG_ELF,
    );

    if !(*LogFile).FileName.Buffer.is_null() {
        (*LogFile).Free.expect("non-null Free")(
            (*LogFile).FileName.Buffer as *mut c_void,
            0,
            TAG_ELF,
        );
    }

    RtlInitEmptyUnicodeString(
        &raw mut (*LogFile).FileName,
        core::ptr::null_mut::<u16>(),
        0,
    );
}

/// Read one event record by record number.
#[no_mangle]
pub unsafe extern "system" fn ElfReadRecord(
    LogFile: *mut _EVTLOGFILE,
    RecordNumber: u32,
    Record: *mut EVENTLOGRECORD,
    BufSize: usize,
    BytesRead: *mut usize,
    BytesNeeded: *mut usize,
) -> i32 {
    if LogFile.is_null() {
        RtlAssert(
            b"LogFile\0".as_ptr() as *const c_void,
            b"evtlib.c\0".as_ptr() as *const c_void,
            1211,
            core::ptr::null_mut(),
        );
    }

    if !BytesRead.is_null() { *BytesRead = 0; }
    if !BytesNeeded.is_null() { *BytesNeeded = 0; }

    let RecOffset = ElfpOffsetByNumber(LogFile, RecordNumber);
    if RecOffset == 0 {
        return STATUS_NOT_FOUND;
    }

    // Read just the Length field first
    let mut FileOffset: LARGE_INTEGER = LARGE_INTEGER { QuadPart: RecOffset as i64 };
    let mut RecSize: u32 = 0;
    let mut ReadLength: usize = 0;

    let Status = (*LogFile).FileRead.expect("non-null FileRead")(
        LogFile,
        &raw mut FileOffset,
        &raw mut RecSize as *mut c_void,
        core::mem::size_of::<u32>(),
        &raw mut ReadLength,
    );
    if Status < 0 {
        return Status;
    }

    if BufSize < RecSize as usize {
        if !BytesNeeded.is_null() {
            *BytesNeeded = RecSize as usize;
        }
        return STATUS_BUFFER_TOO_SMALL;
    }

    // Read full record (may wrap)
    FileOffset.QuadPart = RecOffset as i64;
    let Status = ReadLogBuffer(
        LogFile,
        Record as *mut c_void,
        RecSize as usize,
        &raw mut ReadLength,
        &raw const FileOffset,
        core::ptr::null_mut(),
    );

    if !BytesRead.is_null() {
        *BytesRead = ReadLength;
    }

    Status
}

/// Write one event record to the log (may overwrite oldest records on wrap).
#[no_mangle]
pub unsafe extern "system" fn ElfWriteRecord(
    LogFile: *mut _EVTLOGFILE,
    Record: *mut EVENTLOGRECORD,
    BufSize: usize,
) -> i32 {
    if LogFile.is_null() {
        RtlAssert(
            b"LogFile\0".as_ptr() as *const c_void,
            b"evtlib.c\0".as_ptr() as *const c_void,
            1283,
            core::ptr::null_mut(),
        );
    }

    if (*LogFile).ReadOnly != 0 {
        return STATUS_ACCESS_DENIED;
    }
    if Record.is_null() || BufSize < core::mem::size_of::<EVENTLOGRECORD>() {
        return STATUS_INVALID_PARAMETER;
    }

    (*Record).RecordNumber = (*LogFile).Header.CurrentRecordNumber;

    // Compute free space
    let mut FreeSpace: u32;
    if (*LogFile).Header.StartOffset <= (*LogFile).Header.EndOffset {
        FreeSpace = (*LogFile).Header.MaxSize
            .wrapping_sub((*LogFile).Header.EndOffset)
            .wrapping_add((*LogFile).Header.StartOffset)
            .wrapping_sub(core::mem::size_of::<EVENTLOGHEADER>() as u32);
    } else {
        FreeSpace = (*LogFile).Header.StartOffset
            .wrapping_sub((*LogFile).Header.EndOffset);
    }

    (*LogFile).Header.Flags |= ELF_LOGFILE_HEADER_DIRTY;

    if (*LogFile).Header.OldestRecordNumber == 0 {
        (*LogFile).Header.OldestRecordNumber = 1;
    }

    let mut WriteOffset: u32 = (*LogFile).Header.EndOffset;

    let mut UpperBound: u32 = if (*LogFile).Header.StartOffset <= (*LogFile).Header.EndOffset {
        (*LogFile).Header.MaxSize
    } else {
        (*LogFile).Header.StartOffset
    };

    // Need to rewind to beginning of log?
    if (*LogFile).Header.StartOffset < (*LogFile).Header.EndOffset
        && ((*LogFile).Header.MaxSize.wrapping_sub(WriteOffset) as usize)
            < core::mem::size_of::<EVENTLOGRECORD>()
    {
        WriteOffset = core::mem::size_of::<EVENTLOGHEADER>() as u32;
        UpperBound = (*LogFile).Header.StartOffset;
        FreeSpace = (*LogFile).Header.StartOffset.wrapping_sub(WriteOffset);
        (*LogFile).Header.Flags |= ELF_LOGFILE_HEADER_WRAP;
    }

    // Overwrite oldest records until there is enough space
    if (UpperBound.wrapping_sub(WriteOffset) as usize)
        < BufSize.wrapping_add(core::mem::size_of::<EVENTLOGEOF>())
    {
        let OrgOldestRecordNumber = (*LogFile).Header.OldestRecordNumber;
        let mut OldestRecordNumber = OrgOldestRecordNumber;

        while (FreeSpace as usize) < BufSize.wrapping_add(core::mem::size_of::<EVENTLOGEOF>()) {
            let RecOffset = ElfpOffsetByNumber(LogFile, OldestRecordNumber);
            if RecOffset == 0 {
                (*LogFile).Header.Flags |= ELF_LOGFILE_LOGFULL_WRITTEN;
                return STATUS_LOG_FILE_FULL;
            }

            // Zero-fill RecBuf (fix: was RtlCopyMemory from &0u8 = 48-byte overread)
            let mut RecBuf: EVENTLOGRECORD = core::mem::zeroed();
            // Use write_bytes to zero the struct (safe zero-fill, no UB)
            core::ptr::write_bytes(
                &raw mut RecBuf as *mut u8,
                0,
                core::mem::size_of::<EVENTLOGRECORD>(),
            );

            let mut FileOffset: LARGE_INTEGER = LARGE_INTEGER { QuadPart: RecOffset as i64 };
            let mut ReadLength: usize = 0;

            let Status = (*LogFile).FileRead.expect("non-null FileRead")(
                LogFile,
                &raw mut FileOffset,
                &raw mut RecBuf as *mut c_void,
                core::mem::size_of::<EVENTLOGRECORD>(),
                &raw mut ReadLength,
            );
            if Status < 0 {
                return Status;
            }

            if RecBuf.Reserved != LOGFILE_SIGNATURE {
                return STATUS_EVENTLOG_FILE_CORRUPT;
            }

            // Retention check
            if (*LogFile).Header.Retention != 0
                && ((*Record).TimeWritten < RecBuf.TimeWritten
                    || ((*Record).TimeWritten >= RecBuf.TimeWritten
                        && (*Record).TimeWritten.wrapping_sub(RecBuf.TimeWritten)
                            < (*LogFile).Header.Retention))
            {
                (*LogFile).Header.Flags |= ELF_LOGFILE_LOGFULL_WRITTEN;
                return STATUS_LOG_FILE_FULL;
            }

            OldestRecordNumber = OldestRecordNumber.wrapping_add(1);
            let new_rec_offset = RecOffset.wrapping_add(RecBuf.Length);
            FreeSpace = FreeSpace.wrapping_add(RecBuf.Length);

            if ((*LogFile).Header.MaxSize.wrapping_sub(new_rec_offset) as usize)
                < core::mem::size_of::<EVENTLOGRECORD>()
            {
                FreeSpace = FreeSpace.wrapping_add(
                    (*LogFile).Header.MaxSize.wrapping_sub(new_rec_offset),
                );
            }
        }

        (*LogFile).Header.Flags |= ELF_LOGFILE_HEADER_WRAP;

        ElfpDeleteOffsetInformation(
            LogFile,
            OrgOldestRecordNumber,
            OldestRecordNumber.wrapping_sub(1),
        );
        (*LogFile).Header.OldestRecordNumber = OldestRecordNumber;
        (*LogFile).Header.StartOffset = ElfpOffsetByNumber(LogFile, OldestRecordNumber);

        if (*LogFile).Header.StartOffset == 0 {
            (*LogFile).Header.StartOffset = core::mem::size_of::<EVENTLOGHEADER>() as u32;
            WriteOffset = (*LogFile).Header.StartOffset;
            (*LogFile).Header.EndOffset = WriteOffset;
        }
    }

    // Expand the log file if needed: to fit this record (granularity-rounded), or straight
    // to MaxSize when wrapping or past the cap (upstream 24e904e0956).
    if (*LogFile).CurrentSize < (*LogFile).Header.MaxSize {
        let mut NewSize = WriteOffset + BufSize as u32 + core::mem::size_of::<EVENTLOGEOF>() as u32;
        if WriteOffset < (*LogFile).Header.EndOffset || NewSize > (*LogFile).Header.MaxSize {
            NewSize = (*LogFile).Header.MaxSize;
        } else {
            NewSize = round_up_ulong(NewSize, EVENTLOG_FILE_GRANULARITY);
        }
        if NewSize > (*LogFile).CurrentSize {
            let Status = (*LogFile).FileSetSize.expect("non-null FileSetSize")(
                LogFile,
                NewSize,
                (*LogFile).CurrentSize,
            );
            if Status < 0 {
                return Status;
            }
            (*LogFile).CurrentSize = NewSize;
        }
    }

    // Clear log-full flag
    (*LogFile).Header.Flags &= !ELF_LOGFILE_LOGFULL_WRITTEN;

    // Pad end of log with 0x27 bytes if wrapping
    if WriteOffset < (*LogFile).Header.EndOffset {
        // Pad from EndOffset to MaxSize, rounded down to ULONG
        let pad_len = round_down_ulong(
            (*LogFile).Header.MaxSize.wrapping_sub((*LogFile).Header.EndOffset),
            4,
        ) as usize;

        // We use a stack buffer for the pattern fill — but RecBuf is only
        // sizeof(EVENTLOGRECORD) = 48 bytes while pad_len can be up to MaxSize.
        // Use RtlFillMemoryUlong on the file directly via write path.
        // Build a scratch buffer on the stack (max pad = header size - 1 < 48 bytes):
        // Actually MaxSize - EndOffset can be up to MaxSize, but we only write
        // what fits in RecBuf (EVENTLOGRECORD header = 48 bytes, which is the
        // documented max pad = sizeof(EVENTLOGRECORD) - 1 = 47 bytes).
        // Use a heap-less approach: write directly at EndOffset.
        let mut PadBuf: EVENTLOGRECORD = core::mem::zeroed();
        RtlFillMemoryUlong(
            &raw mut PadBuf as *mut c_void,
            pad_len.min(core::mem::size_of::<EVENTLOGRECORD>()),
            0x00000027,
        );
        let mut fo: LARGE_INTEGER = LARGE_INTEGER {
            QuadPart: (*LogFile).Header.EndOffset as i64,
        };
        let mut wl: usize = 0;
        (*LogFile).FileWrite.expect("non-null FileWrite")(
            LogFile,
            &raw mut fo,
            &raw mut PadBuf as *mut c_void,
            pad_len.min(core::mem::size_of::<EVENTLOGRECORD>()),
            &raw mut wl,
        );
        // Ignore error — same as C source (comment: // return Status;)
    }

    // Write the event record (may wrap)
    let mut FileOffset: LARGE_INTEGER = LARGE_INTEGER { QuadPart: WriteOffset as i64 };
    let mut WrittenLength: usize = 0;
    let mut NextOffset: LARGE_INTEGER = LARGE_INTEGER { QuadPart: 0 };

    let Status = WriteLogBuffer(
        LogFile,
        Record as *mut c_void,
        BufSize,
        &raw mut WrittenLength,
        &raw mut FileOffset,
        &raw mut NextOffset,
    );
    if Status < 0 {
        return Status;
    }
    FileOffset = NextOffset;

    // Register the new record's offset
    if ElfpAddOffsetInformation(
        LogFile,
        (*Record).RecordNumber,
        WriteOffset,
    ) == 0
    {
        return STATUS_NO_MEMORY;
    }

    (*LogFile).Header.CurrentRecordNumber =
        (*LogFile).Header.CurrentRecordNumber.wrapping_add(1);
    if (*LogFile).Header.CurrentRecordNumber == 0 {
        (*LogFile).Header.CurrentRecordNumber = 1;
    }

    (*LogFile).Header.EndOffset = FileOffset.QuadPart as u32;

    // Write EOF record (may wrap)
    let mut EofRec: EVENTLOGEOF = core::mem::zeroed();
    core::ptr::copy_nonoverlapping(
        &EOFRecord as *const EVENTLOGEOF as *const u8,
        &raw mut EofRec as *mut u8,
        core::mem::size_of::<EVENTLOGEOF>(),
    );
    EofRec.BeginRecord = (*LogFile).Header.StartOffset;
    EofRec.EndRecord = (*LogFile).Header.EndOffset;
    EofRec.CurrentRecordNumber = (*LogFile).Header.CurrentRecordNumber;
    EofRec.OldestRecordNumber = (*LogFile).Header.OldestRecordNumber;

    let mut WrLen2: usize = 0;
    let mut NextOffset2: LARGE_INTEGER = LARGE_INTEGER { QuadPart: 0 };
    let Status = WriteLogBuffer(
        LogFile,
        &raw mut EofRec as *mut c_void,
        core::mem::size_of::<EVENTLOGEOF>(),
        &raw mut WrLen2,
        &raw mut FileOffset,
        &raw mut NextOffset2,
    );
    if Status < 0 {
        return Status;
    }

    // Flush
    let Status = ElfFlushFile(LogFile);
    if Status < 0 {
        return STATUS_EVENTLOG_FILE_CORRUPT;
    }

    Status
}

/// Backup the event log to another (freshly initialized) log file.
#[no_mangle]
pub unsafe extern "system" fn ElfBackupFile(
    LogFile: *mut _EVTLOGFILE,
    BackupLogFile: *mut _EVTLOGFILE,
) -> i32 {
    if LogFile.is_null() {
        RtlAssert(
            b"LogFile\0".as_ptr() as *const c_void,
            b"evtlib.c\0".as_ptr() as *const c_void,
            994,
            core::ptr::null_mut(),
        );
    }

    RtlZeroMemory(BackupLogFile as *mut c_void, core::mem::size_of::<_EVTLOGFILE>());
    (*BackupLogFile).FileSetSize = (*LogFile).FileSetSize;
    (*BackupLogFile).FileWrite = (*LogFile).FileWrite;
    (*BackupLogFile).FileFlush = (*LogFile).FileFlush;
    (*BackupLogFile).ReadOnly = 0;

    let Header = &raw mut (*BackupLogFile).Header;
    (*Header).HeaderSize = core::mem::size_of::<EVENTLOGHEADER>() as u32;
    (*Header).Signature = LOGFILE_SIGNATURE;
    (*Header).MajorVersion = MAJORVER;
    (*Header).MinorVersion = MINORVER;
    (*Header).StartOffset = core::mem::size_of::<EVENTLOGHEADER>() as u32;
    (*Header).EndOffset = core::mem::size_of::<EVENTLOGHEADER>() as u32;
    (*Header).CurrentRecordNumber = 1;
    (*Header).OldestRecordNumber = 0;
    (*Header).MaxSize = (*LogFile).Header.MaxSize;
    (*Header).Flags = ELF_LOGFILE_HEADER_DIRTY;
    (*Header).Retention = (*LogFile).Header.Retention;
    (*Header).EndHeaderSize = core::mem::size_of::<EVENTLOGHEADER>() as u32;

    let mut FileOffset: LARGE_INTEGER = LARGE_INTEGER { QuadPart: 0 };
    let mut WrittenLength: usize = 0;

    let mut Status = (*BackupLogFile).FileWrite.expect("non-null FileWrite")(
        BackupLogFile,
        &raw mut FileOffset,
        Header as *mut c_void,
        core::mem::size_of::<EVENTLOGHEADER>(),
        &raw mut WrittenLength,
    );
    if Status < 0 {
        return Status;
    }

    let mut i = (*LogFile).Header.OldestRecordNumber;
    'outer: loop {
        if i >= (*LogFile).Header.CurrentRecordNumber {
            // All records written — write EOF record below
            let mut EofRec: EVENTLOGEOF = core::mem::zeroed();
            core::ptr::copy_nonoverlapping(
                &EOFRecord as *const EVENTLOGEOF as *const u8,
                &raw mut EofRec as *mut u8,
                core::mem::size_of::<EVENTLOGEOF>(),
            );
            EofRec.BeginRecord = (*Header).StartOffset;
            EofRec.EndRecord = (*Header).EndOffset;
            EofRec.CurrentRecordNumber = (*LogFile).Header.CurrentRecordNumber;
            EofRec.OldestRecordNumber = (*LogFile).Header.OldestRecordNumber;

            Status = (*BackupLogFile).FileWrite.expect("non-null FileWrite")(
                BackupLogFile,
                core::ptr::null_mut(),
                &raw mut EofRec as *mut c_void,
                core::mem::size_of::<EVENTLOGEOF>(),
                &raw mut WrittenLength,
            );
            if Status < 0 {
                break 'outer;
            }

            (*Header).CurrentRecordNumber = (*LogFile).Header.CurrentRecordNumber;
            (*Header).OldestRecordNumber = (*LogFile).Header.OldestRecordNumber;
            (*Header).MaxSize = round_up_ulong(
                (*Header).EndOffset.wrapping_add(core::mem::size_of::<EVENTLOGEOF>() as u32),
                4,
            );
            (*Header).Flags = 0;

            Status = ElfFlushFile(BackupLogFile);
            break 'outer;
        }

        let RecOffset = ElfpOffsetByNumber(LogFile, i);
        if RecOffset == 0 {
            break 'outer;
        }

        FileOffset.QuadPart = RecOffset as i64;
        let mut RecBuf: EVENTLOGRECORD = core::mem::zeroed();
        let mut ReadLength: usize = 0;

        Status = (*LogFile).FileRead.expect("non-null FileRead")(
            LogFile,
            &raw mut FileOffset,
            &raw mut RecBuf as *mut c_void,
            core::mem::size_of::<EVENTLOGRECORD>(),
            &raw mut ReadLength,
        );
        if Status < 0 {
            break 'outer;
        }

        let Buffer = (*LogFile).Allocate.expect("non-null Allocate")(
            RecBuf.Length as usize,
            0,
            TAG_ELF_BUF,
        );
        if Buffer.is_null() {
            break 'outer;
        }

        Status = ReadLogBuffer(
            LogFile,
            Buffer,
            RecBuf.Length as usize,
            &raw mut ReadLength,
            &raw const FileOffset,
            core::ptr::null_mut(),
        );
        if Status < 0 {
            (*LogFile).Free.expect("non-null Free")(Buffer, 0, TAG_ELF_BUF);
            break 'outer;
        }

        Status = (*BackupLogFile).FileWrite.expect("non-null FileWrite")(
            BackupLogFile,
            core::ptr::null_mut(),
            Buffer,
            RecBuf.Length as usize,
            &raw mut WrittenLength,
        );
        (*LogFile).Free.expect("non-null Free")(Buffer, 0, TAG_ELF_BUF);
        if Status < 0 {
            break 'outer;
        }

        (*Header).EndOffset = (*Header).EndOffset.wrapping_add(RecBuf.Length);
        i = i.wrapping_add(1);
    }

    Status
}

/// Return the oldest record number in the log.
#[no_mangle]
pub unsafe extern "system" fn ElfGetOldestRecord(
    LogFile: *mut _EVTLOGFILE,
) -> u32 {
    if LogFile.is_null() {
        RtlAssert(
            b"LogFile\0".as_ptr() as *const c_void,
            b"evtlib.c\0".as_ptr() as *const c_void,
            1592,
            core::ptr::null_mut(),
        );
    }
    (*LogFile).Header.OldestRecordNumber
}

/// Return the next record number (current write position).
#[no_mangle]
pub unsafe extern "system" fn ElfGetCurrentRecord(
    LogFile: *mut _EVTLOGFILE,
) -> u32 {
    if LogFile.is_null() {
        RtlAssert(
            b"LogFile\0".as_ptr() as *const c_void,
            b"evtlib.c\0".as_ptr() as *const c_void,
            1601,
            core::ptr::null_mut(),
        );
    }
    (*LogFile).Header.CurrentRecordNumber
}

/// Return the log file flags.
#[no_mangle]
pub unsafe extern "system" fn ElfGetFlags(
    LogFile: *mut _EVTLOGFILE,
) -> u32 {
    if LogFile.is_null() {
        RtlAssert(
            b"LogFile\0".as_ptr() as *const c_void,
            b"evtlib.c\0".as_ptr() as *const c_void,
            1610,
            core::ptr::null_mut(),
        );
    }
    (*LogFile).Header.Flags
}

/// Debug helper — print all header fields (conditional on DBG build).
/// In release builds this is a no-op (function still exported for ABI compat).
#[no_mangle]
pub unsafe extern "C" fn PRINT_HEADER(Header: *mut EVENTLOGHEADER) {
    #[cfg(debug_assertions)]
    {
        let mut Flags = (*Header).Flags;
        DbgPrint(b"HeaderSize    = %lu\n\0".as_ptr() as *const c_char, (*Header).HeaderSize);
        DbgPrint(b"Signature     = 0x%x\n\0".as_ptr() as *const c_char, (*Header).Signature);
        DbgPrint(b"MajorVersion  = %lu\n\0".as_ptr() as *const c_char, (*Header).MajorVersion);
        DbgPrint(b"MinorVersion  = %lu\n\0".as_ptr() as *const c_char, (*Header).MinorVersion);
        DbgPrint(b"StartOffset   = 0x%x\n\0".as_ptr() as *const c_char, (*Header).StartOffset);
        DbgPrint(b"EndOffset     = 0x%x\n\0".as_ptr() as *const c_char, (*Header).EndOffset);
        DbgPrint(b"CurrentRecordNumber = %lu\n\0".as_ptr() as *const c_char, (*Header).CurrentRecordNumber);
        DbgPrint(b"OldestRecordNumber  = %lu\n\0".as_ptr() as *const c_char, (*Header).OldestRecordNumber);
        DbgPrint(b"MaxSize       = 0x%x\n\0".as_ptr() as *const c_char, (*Header).MaxSize);
        DbgPrint(b"Retention     = 0x%x\n\0".as_ptr() as *const c_char, (*Header).Retention);
        DbgPrint(b"EndHeaderSize = %lu\n\0".as_ptr() as *const c_char, (*Header).EndHeaderSize);
        DbgPrint(b"Flags: \0".as_ptr() as *const c_char);
        if Flags & ELF_LOGFILE_HEADER_DIRTY != 0 {
            DbgPrint(b"ELF_LOGFILE_HEADER_DIRTY\0".as_ptr() as *const c_char);
            Flags &= !ELF_LOGFILE_HEADER_DIRTY;
        }
        if Flags != 0 { DbgPrint(b" | \0".as_ptr() as *const c_char); }
        if Flags & ELF_LOGFILE_HEADER_WRAP != 0 {
            DbgPrint(b"ELF_LOGFILE_HEADER_WRAP\0".as_ptr() as *const c_char);
            Flags &= !ELF_LOGFILE_HEADER_WRAP;
        }
        if Flags != 0 { DbgPrint(b" | \0".as_ptr() as *const c_char); }
        if Flags & ELF_LOGFILE_LOGFULL_WRITTEN != 0 {
            DbgPrint(b"ELF_LOGFILE_LOGFULL_WRITTEN\0".as_ptr() as *const c_char);
            Flags &= !ELF_LOGFILE_LOGFULL_WRITTEN;
        }
        if Flags != 0 { DbgPrint(b" | \0".as_ptr() as *const c_char); }
        if Flags & ELF_LOGFILE_ARCHIVE_SET != 0 {
            DbgPrint(b"ELF_LOGFILE_ARCHIVE_SET\0".as_ptr() as *const c_char);
            Flags &= !ELF_LOGFILE_ARCHIVE_SET;
        }
        if Flags != 0 {
            DbgPrint(b" | 0x%x\0".as_ptr() as *const c_char, Flags);
        }
        DbgPrint(b"\n\0".as_ptr() as *const c_char);
    }
}
