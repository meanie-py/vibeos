//! Windows 10-era kernel32 exports that real desktop apps (Blender, Paint.NET, .NET, Chrome,
//! Steam, CMake) import at load time. Semantics follow Wine's dlls/kernelbase, except where a
//! divergence is called out inline.
//!
//! Covered here: CONTEXT helpers (CopyContext / InitializeContext[2] / *XStateFeature*),
//! CopyFile2, Get/SetProcessMitigationPolicy, ResolveLocaleName, SetWaitableTimerEx,
//! UnmapViewOfFileEx, the AppModel package queries and the Pss* process-snapshot family.
//!
//! Nothing here fakes success for something the system cannot do: where the kernel lacks the
//! backing facility (XSAVE contexts, process snapshots, mitigation enforcement, package
//! registry) the call fails with the error code Windows uses for "not available".
use ntdll_sys::nt::*;
use crate::prelude::*;
use core::ffi::c_void;
use core::mem::size_of;
use core::ptr;

const FALSE: BOOL = 0;
const TRUE: BOOL = 1;

const S_OK_: HRESULT = 0;

const ERROR_SUCCESS_: u32 = 0;
const ERROR_NOT_SUPPORTED_: u32 = 50;
const ERROR_INVALID_PARAMETER_: u32 = 87;
const ERROR_INSUFFICIENT_BUFFER_: u32 = 122;
const ERROR_INVALID_HANDLE_: u32 = 6;
const ERROR_NOT_FOUND_: u32 = 1168;
/// AppModel: the calling process / handle has no package identity (unpackaged desktop app).
const APPMODEL_ERROR_NO_PACKAGE_: u32 = 15700;

const STATUS_SUCCESS_: NTSTATUS = 0;
const STATUS_NOT_SUPPORTED_: NTSTATUS = 0xC000_00BB_u32 as NTSTATUS;
const STATUS_INVALID_PARAMETER_: NTSTATUS = 0xC000_000D_u32 as NTSTATUS;

#[inline]
unsafe fn set_ntstatus(Status: NTSTATUS) -> BOOL {
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return FALSE;
    }
    TRUE
}

#[inline]
fn hresult_from_win32(e: u32) -> HRESULT {
    if (e as i32) <= 0 {
        e as i32
    } else {
        ((e & 0xFFFF) | 0x8007_0000) as i32
    }
}

/* ------------------------------------------------------------------------- *
 * CONTEXT helpers: CopyContext, InitializeContext[2], XState feature access
 *
 * Wine's ntdll/exception.c layout rules. The kernel has no CONTEXT_XSTATE support (it never
 * saves/restores XSAVE state into a CONTEXT, and GetEnabledXStateFeatures reports the
 * legacy x87+SSE baseline only), so every request that needs the extended-state area is
 * answered as Windows answers on a CPU without it: InitializeContext silently drops the
 * CONTEXT_XSTATE bit (Wine does the same) and CopyContext fails with STATUS_NOT_SUPPORTED.
 * ------------------------------------------------------------------------- */

const CONTEXT_I386_ARCH: u32 = 0x0001_0000;
const CONTEXT_AMD64_ARCH: u32 = 0x0010_0000;
const CONTEXT_ARM64_ARCH: u32 = 0x0040_0000;
const CONTEXT_ARCH_MASK: u32 = CONTEXT_I386_ARCH | CONTEXT_AMD64_ARCH | CONTEXT_ARM64_ARCH;
/// CONTEXT_XSTATE low bits on x86/x64 (CONTEXT_<arch> | 0x40).
const CONTEXT_XSTATE_BIT: u32 = 0x40;
/// CONTEXT_SERVICE_ACTIVE | EXCEPTION_ACTIVE | EXCEPTION_REQUEST | EXCEPTION_REPORTING.
const CONTEXT_STATE_BITS: u32 = 0xD800_0000;

/// CONTEXT_CHUNK { LONG Offset; ULONG Length; }.
#[repr(C)]
#[derive(Clone, Copy)]
struct ContextChunk {
    Offset: i32,
    Length: u32,
}

/// CONTEXT_EX. The Windows SDK pads it to 32 bytes on 64-bit targets (Wine: 0x20 on amd64,
/// 0x18 on i386; there is no CONTEXT_EX in the llvm-mingw headers to probe).
#[repr(C)]
struct ContextEx {
    All: ContextChunk,
    Legacy: ContextChunk,
    XState: ContextChunk,
    #[cfg(target_pointer_width = "64")]
    _align: u64,
}
#[cfg(target_pointer_width = "64")]
const _: () = assert!(size_of::<ContextEx>() == 0x20);
#[cfg(target_pointer_width = "32")]
const _: () = assert!(size_of::<ContextEx>() == 0x18);

struct CtxParams {
    arch_flag: u32,
    supported_flags: u32,
    context_size: u32,
    legacy_size: u32,
    /// sizeof(CONTEXT_EX) for this architecture (independent of the host pointer width).
    context_ex_size: u32,
    /// Extra bytes the length query adds so a pointer-aligned buffer can be re-aligned.
    size_slack: u32,
    align_mask: usize,
    flags_offset: usize,
    /// (start offset, context flag) ranges; each range runs to the next entry's start.
    copy_ranges: &'static [(u32, u32)],
}

// Measured with llvm-mingw clang: sizeof(CONTEXT) x64 1232 (ContextFlags at 0x30), i386 716
// (ExtendedRegisters at 0xcc), arm64 912 (statically asserted: Cpsr@4, X@8, Fp@240, Sp@256,
// V@272, Fpcr@784, Bcr@792).
#[cfg(target_arch = "x86_64")]
static CTX_AMD64: CtxParams = CtxParams {
    arch_flag: CONTEXT_AMD64_ARCH,
    supported_flags: CONTEXT_STATE_BITS | CONTEXT_AMD64_ARCH | 0x1F | CONTEXT_XSTATE_BIT,
    context_size: 0x4D0,
    legacy_size: 0x4D0,
    context_ex_size: 0x20,
    size_slack: 7,
    align_mask: 15,
    flags_offset: 0x30,
    copy_ranges: &[
        (0x38, 0x1),
        (0x3A, 0x4),
        (0x42, 0x1),
        (0x48, 0x10),
        (0x78, 0x2),
        (0x98, 0x1),
        (0xA0, 0x2),
        (0xF8, 0x1),
        (0x100, 0x8),
        (0x2A0, 0),
        (0x4B0, 0x10),
        (0x4D0, 0),
    ],
};

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
static CTX_I386: CtxParams = CtxParams {
    arch_flag: CONTEXT_I386_ARCH,
    supported_flags: CONTEXT_STATE_BITS | CONTEXT_I386_ARCH | 0x3F | CONTEXT_XSTATE_BIT,
    context_size: 0x2CC,
    legacy_size: 0xCC,
    context_ex_size: 0x18,
    size_slack: 3,
    align_mask: 3,
    flags_offset: 0,
    copy_ranges: &[
        (0x4, 0x10),
        (0x1C, 0x8),
        (0x8C, 0x4),
        (0x9C, 0x2),
        (0xB4, 0x1),
        (0xCC, 0x20),
        (0x1EC, 0),
        (0x2CC, 0),
    ],
};

// ARM64 has no XSAVE state (so no CONTEXT_XSTATE bit: any request for it is an invalid
// flag). Wine has no arm64 path; the layout and ranges follow winnt.h's ARM64_NT_CONTEXT.
// The length slack is 15 (not amd64's 7) because the context needs 16-byte alignment and
// CONTEXT_EX uses all 24 of its bytes.
#[cfg(target_arch = "aarch64")]
static CTX_ARM64: CtxParams = CtxParams {
    arch_flag: CONTEXT_ARM64_ARCH,
    supported_flags: CONTEXT_STATE_BITS
        | 0x2000_0000 // CONTEXT_UNWOUND_TO_CALL
        | 0x0400_0000 // CONTEXT_RET_TO_GUEST
        | CONTEXT_ARM64_ARCH
        | 0x1F,
    context_size: 912,
    legacy_size: 912,
    context_ex_size: 0x20,
    size_slack: 15,
    align_mask: 15,
    flags_offset: 0,
    copy_ranges: &[
        (0x4, 0x1),   // Cpsr                       CONTROL
        (0x8, 0x2),   // X0..X28                    INTEGER
        (0xF0, 0x1),  // Fp, Lr, Sp, Pc             CONTROL
        (0x110, 0x4), // V0..V31, Fpcr, Fpsr        FLOATING_POINT
        (0x318, 0x8), // Bcr/Bvr/Wcr/Wvr            DEBUG_REGISTERS
        (912, 0),
    ],
};

/// CONTEXT architectures this build can describe: its own, plus i386 contexts on x64
/// (WoW64 callers), exactly as Wine's table.
#[cfg(target_arch = "x86_64")]
static CTX_ARCHES: [&CtxParams; 2] = [&CTX_AMD64, &CTX_I386];
#[cfg(target_arch = "x86")]
static CTX_ARCHES: [&CtxParams; 1] = [&CTX_I386];
#[cfg(target_arch = "aarch64")]
static CTX_ARCHES: [&CtxParams; 1] = [&CTX_ARM64];

/// Wine's context_get_parameters: the architecture is picked by its flag bit and every other
/// flag must be one that architecture defines.
fn ctx_params(context_flags: u32) -> Option<&'static CtxParams> {
    for p in CTX_ARCHES.iter() {
        if context_flags & p.arch_flag != 0 {
            return if context_flags & !p.supported_flags != 0 {
                None
            } else {
                Some(p)
            };
        }
    }
    None
}

/// True when the kernel can save/restore CONTEXT_XSTATE areas. It cannot, see above.
#[inline]
fn xstate_supported() -> bool {
    false
}

#[no_mangle]
pub unsafe extern "system" fn InitializeContext2(
    Buffer: PVOID,
    ContextFlags: DWORD,
    Context: *mut *mut c_void,
    ContextLength: *mut DWORD,
    XStateCompactionMask: ULONG64,
) -> BOOL {
    // Only the extended-state area depends on the mask; without XSTATE it is irrelevant.
    let _ = XStateCompactionMask;
    if ContextLength.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER_);
        return FALSE;
    }
    let orig_length = *ContextLength;

    let mut flags = ContextFlags;
    let mut p = match ctx_params(flags) {
        Some(p) => p,
        None => return set_ntstatus(STATUS_INVALID_PARAMETER_),
    };
    if flags & CONTEXT_XSTATE_BIT != 0 && !xstate_supported() {
        // Wine/Windows: on a system without the feature, retry as a plain context.
        flags &= !CONTEXT_XSTATE_BIT;
        p = match ctx_params(flags) {
            Some(p) => p,
            None => return set_ntstatus(STATUS_INVALID_PARAMETER_),
        };
    }

    // Documented "query required size" protocol: Buffer == NULL (or too small) reports the
    // size in *ContextLength and fails with ERROR_INSUFFICIENT_BUFFER.
    let length = p.context_size + p.context_ex_size + p.size_slack;
    *ContextLength = length;
    if Buffer.is_null() || orig_length < length {
        SetLastError(ERROR_INSUFFICIENT_BUFFER_);
        return FALSE;
    }
    if Context.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER_);
        return FALSE;
    }

    let ctx = ((Buffer as usize + p.align_mask) & !p.align_mask) as *mut u8;
    ptr::write(ctx.add(p.flags_offset) as *mut u32, flags);

    let ex = ctx.add(p.context_size as usize) as *mut ContextEx;
    let back = -(p.context_size as i32);
    (*ex).Legacy.Offset = back;
    (*ex).All.Offset = back;
    (*ex).Legacy.Length = if flags & 0x20 != 0 { p.context_size } else { p.legacy_size };
    // Per Wine's tests: without CONTEXT_XSTATE the XState chunk offset is 25, length 0, and
    // All covers the CONTEXT plus 24 bytes (sizeof(CONTEXT_EX) minus the x64 padding).
    (*ex).XState.Offset = 25;
    (*ex).XState.Length = 0;
    (*ex).All.Length = p.context_size + 24;

    *Context = ctx as *mut c_void;
    TRUE
}

#[no_mangle]
pub unsafe extern "system" fn InitializeContext(
    Buffer: PVOID,
    ContextFlags: DWORD,
    Context: *mut *mut c_void,
    ContextLength: *mut DWORD,
) -> BOOL {
    InitializeContext2(Buffer, ContextFlags, Context, ContextLength, !0u64)
}

/// RtlCopyContext (Wine ntdll/exception.c): copy the pieces selected by ContextFlags that
/// the source actually holds, and mark them valid in the destination.
unsafe fn copy_context(dst: *mut u8, context_flags: u32, src: *const u8) -> NTSTATUS {
    if context_flags & CONTEXT_XSTATE_BIT != 0 && !xstate_supported() {
        return STATUS_NOT_SUPPORTED_;
    }
    let arch_flag = context_flags & CONTEXT_ARCH_MASK;
    let p = match CTX_ARCHES.iter().find(|p| p.arch_flag == arch_flag) {
        Some(p) => *p,
        None => return STATUS_INVALID_PARAMETER_,
    };

    let dst_flags = ptr::read(dst.add(p.flags_offset) as *const u32);
    let src_flags = ptr::read(src.add(p.flags_offset) as *const u32);
    if dst_flags & CONTEXT_ARCH_MASK != arch_flag || src_flags & CONTEXT_ARCH_MASK != arch_flag {
        return STATUS_INVALID_PARAMETER_;
    }

    let context_flags = context_flags & src_flags;
    let p = match ctx_params(context_flags) {
        Some(p) => p,
        None => return STATUS_INVALID_PARAMETER_,
    };

    let df = dst.add(p.flags_offset) as *mut u32;
    ptr::write(df, ptr::read(df) | context_flags);

    let mut start: u32 = 0;
    for &(range_start, flag) in p.copy_ranges.iter() {
        if flag & context_flags != 0 {
            if start == 0 {
                start = range_start;
            }
        } else if start != 0 {
            // ptr::copy: dst and src may be the same CONTEXT.
            ptr::copy(src.add(start as usize), dst.add(start as usize), (range_start - start) as usize);
            start = 0;
        }
    }
    STATUS_SUCCESS_
}

#[no_mangle]
pub unsafe extern "system" fn CopyContext(
    Destination: *mut c_void,
    ContextFlags: DWORD,
    Source: *mut c_void,
) -> BOOL {
    if Destination.is_null() || Source.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER_);
        return FALSE;
    }
    set_ntstatus(copy_context(Destination as *mut u8, ContextFlags, Source as *const u8))
}

// XState feature access. Feature 0 is the legacy x87/FXSAVE header (offsetof(XSAVE_FORMAT,
// XmmRegisters) == 160 bytes), feature 1 the XMM registers; features >= 2 live in the
// CONTEXT_XSTATE area, which we never produce, so they locate to NULL (Wine/Windows: also
// NULL when the feature is not enabled).
const XSAVE_FORMAT_XMM_OFFSET: u32 = 160;

#[cfg(target_arch = "x86_64")]
mod xs_native {
    pub const ARCH: u32 = super::CONTEXT_AMD64_ARCH;
    pub const FLAGS_OFFSET: usize = 0x30;
    /// CONTEXT_FLOATING_POINT
    pub const LEGACY_FLAG: u32 = ARCH | 0x8;
    /// FltSave (XSAVE_FORMAT) inside CONTEXT.
    pub const LEGACY_OFFSET: usize = 0x100;
    pub const XMM_COUNT: u32 = 16;
}
#[cfg(target_arch = "x86")]
mod xs_native {
    pub const ARCH: u32 = super::CONTEXT_I386_ARCH;
    pub const FLAGS_OFFSET: usize = 0;
    /// CONTEXT_EXTENDED_REGISTERS
    pub const LEGACY_FLAG: u32 = ARCH | 0x20;
    /// ExtendedRegisters (an FXSAVE image) inside CONTEXT.
    pub const LEGACY_OFFSET: usize = 0xCC;
    pub const XMM_COUNT: u32 = 8;
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[no_mangle]
pub unsafe extern "system" fn LocateXStateFeature(
    Context: *mut c_void,
    FeatureId: DWORD,
    Length: *mut DWORD,
) -> PVOID {
    use xs_native::*;
    let base = Context as *mut u8;
    if base.is_null() || ptr::read(base.add(FLAGS_OFFSET) as *const u32) & ARCH == 0 {
        return ptr::null_mut();
    }
    if FeatureId >= 2 {
        return ptr::null_mut();
    }
    if FeatureId == 1 {
        if !Length.is_null() {
            *Length = 16 * XMM_COUNT;
        }
        return base.add(LEGACY_OFFSET + XSAVE_FORMAT_XMM_OFFSET as usize) as PVOID;
    }
    if !Length.is_null() {
        *Length = XSAVE_FORMAT_XMM_OFFSET;
    }
    base.add(LEGACY_OFFSET) as PVOID
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[no_mangle]
pub unsafe extern "system" fn SetXStateFeaturesMask(Context: *mut c_void, FeatureMask: DWORD64) -> BOOL {
    use xs_native::*;
    let base = Context as *mut u8;
    if base.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER_);
        return FALSE;
    }
    let fp = base.add(FLAGS_OFFSET) as *mut u32;
    let flags = ptr::read(fp);
    if flags & ARCH == 0 {
        SetLastError(ERROR_INVALID_PARAMETER_);
        return FALSE;
    }
    if FeatureMask & 0x3 != 0 {
        ptr::write(fp, flags | LEGACY_FLAG);
    }
    // Only the two legacy components can be recorded; any extended component needs the
    // CONTEXT_XSTATE area (Wine: same result for a context without CONTEXT_XSTATE).
    if FeatureMask & !3u64 != 0 {
        SetLastError(ERROR_INVALID_PARAMETER_);
        return FALSE;
    }
    TRUE
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[no_mangle]
pub unsafe extern "system" fn GetXStateFeaturesMask(Context: *mut c_void, FeatureMask: *mut DWORD64) -> BOOL {
    use xs_native::*;
    let base = Context as *mut u8;
    if base.is_null() || FeatureMask.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER_);
        return FALSE;
    }
    let flags = ptr::read(base.add(FLAGS_OFFSET) as *const u32);
    if flags & ARCH == 0 {
        SetLastError(ERROR_INVALID_PARAMETER_);
        return FALSE;
    }
    // Legacy x87+SSE are reported as present exactly when the CONTEXT carries the
    // floating-point / extended-register part; no extended components exist.
    *FeatureMask = if flags & LEGACY_FLAG == LEGACY_FLAG { 3 } else { 0 };
    TRUE
}

// ARM64 has no XSAVE components. These return what a context with no extended state means:
// nothing to locate, an empty mask, and only an empty mask can be set.
#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub unsafe extern "system" fn LocateXStateFeature(
    Context: *mut c_void,
    FeatureId: DWORD,
    Length: *mut DWORD,
) -> PVOID {
    let _ = (Context, FeatureId);
    if !Length.is_null() {
        *Length = 0;
    }
    ptr::null_mut()
}

#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub unsafe extern "system" fn SetXStateFeaturesMask(Context: *mut c_void, FeatureMask: DWORD64) -> BOOL {
    if Context.is_null() || ptr::read(Context as *const u32) & CONTEXT_ARM64_ARCH == 0 || FeatureMask != 0 {
        SetLastError(ERROR_INVALID_PARAMETER_);
        return FALSE;
    }
    TRUE
}

#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub unsafe extern "system" fn GetXStateFeaturesMask(Context: *mut c_void, FeatureMask: *mut DWORD64) -> BOOL {
    if Context.is_null()
        || FeatureMask.is_null()
        || ptr::read(Context as *const u32) & CONTEXT_ARM64_ARCH == 0
    {
        SetLastError(ERROR_INVALID_PARAMETER_);
        return FALSE;
    }
    *FeatureMask = 0;
    TRUE
}

/* ------------------------------------------------------------------------- *
 * CopyFile2
 * ------------------------------------------------------------------------- */

// COPYFILE2_MESSAGE_TYPE
const COPYFILE2_CALLBACK_CHUNK_FINISHED: u32 = 2;
const COPYFILE2_CALLBACK_STREAM_STARTED: u32 = 3;
// COPYFILE2_MESSAGE_ACTION
const COPYFILE2_PROGRESS_CANCEL: u32 = 1;
const COPYFILE2_PROGRESS_STOP: u32 = 2;
const COPYFILE2_PROGRESS_QUIET: u32 = 3;
const COPYFILE2_PROGRESS_PAUSE: u32 = 4;
// LPPROGRESS_ROUTINE callback reasons / return values
const CALLBACK_CHUNK_FINISHED_: u32 = 0;
const CALLBACK_STREAM_SWITCH_: u32 = 1;
const PROGRESS_CANCEL_: u32 = 1;
const PROGRESS_STOP_: u32 = 2;
const PROGRESS_QUIET_: u32 = 3;

#[repr(C)]
#[derive(Clone, Copy)]
struct Cf2ChunkFinished {
    dwStreamNumber: u32,
    dwFlags: u32,
    hSourceFile: HANDLE,
    hDestinationFile: HANDLE,
    uliChunkNumber: u64,
    uliChunkSize: u64,
    uliStreamSize: u64,
    uliStreamBytesTransferred: u64,
    uliTotalFileSize: u64,
    uliTotalBytesTransferred: u64,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Cf2StreamStarted {
    dwStreamNumber: u32,
    dwReserved: u32,
    hSourceFile: HANDLE,
    hDestinationFile: HANDLE,
    uliStreamSize: u64,
    uliTotalFileSize: u64,
}

#[repr(C)]
union Cf2Info {
    ChunkFinished: Cf2ChunkFinished,
    StreamStarted: Cf2StreamStarted,
}

/// COPYFILE2_MESSAGE (only the variants this shim can report).
#[repr(C, align(8))]
struct Copyfile2Message {
    Type: u32,
    dwPadding: u32,
    Info: Cf2Info,
}
#[cfg(target_pointer_width = "64")]
const _: () = assert!(size_of::<Copyfile2Message>() == 80);
#[cfg(target_pointer_width = "32")]
const _: () = assert!(size_of::<Copyfile2Message>() == 72);

type PCopyfile2ProgressRoutine = Option<unsafe extern "system" fn(*const Copyfile2Message, PVOID) -> u32>;

/// COPYFILE2_EXTENDED_PARAMETERS (V1 prefix; the Win11 V2 struct only appends fields).
#[repr(C)]
pub struct Copyfile2ExtendedParameters {
    dwSize: DWORD,
    dwCopyFlags: DWORD,
    pfCancel: *mut BOOL,
    pProgressRoutine: PCopyfile2ProgressRoutine,
    pvCallbackContext: PVOID,
}
#[cfg(target_pointer_width = "64")]
const _: () = assert!(size_of::<Copyfile2ExtendedParameters>() == 32);
#[cfg(target_pointer_width = "32")]
const _: () = assert!(size_of::<Copyfile2ExtendedParameters>() == 20);

struct Cf2Adapter {
    routine: unsafe extern "system" fn(*const Copyfile2Message, PVOID) -> u32,
    context: PVOID,
    chunk_number: u64,
    last_total: u64,
}

/// Bridges CopyFileExW's LPPROGRESS_ROUTINE to the COPYFILE2 callback. The two callbacks
/// report the same events; CopyFile2's richer ones (chunk start, errors, poll) have no
/// counterpart in the underlying copy and are not synthesized.
unsafe extern "system" fn cf2_progress_adapter(
    TotalFileSize: LARGE_INTEGER,
    TotalBytesTransferred: LARGE_INTEGER,
    StreamSize: LARGE_INTEGER,
    StreamBytesTransferred: LARGE_INTEGER,
    dwStreamNumber: DWORD,
    dwCallbackReason: DWORD,
    hSourceFile: HANDLE,
    hDestinationFile: HANDLE,
    lpData: LPVOID,
) -> DWORD {
    let a = &mut *(lpData as *mut Cf2Adapter);
    let total_file = TotalFileSize.QuadPart as u64;
    let total_done = TotalBytesTransferred.QuadPart as u64;
    let stream_size = StreamSize.QuadPart as u64;
    let stream_done = StreamBytesTransferred.QuadPart as u64;

    let mut msg: Copyfile2Message = core::mem::zeroed();
    match dwCallbackReason {
        CALLBACK_CHUNK_FINISHED_ => {
            a.chunk_number += 1;
            msg.Type = COPYFILE2_CALLBACK_CHUNK_FINISHED;
            msg.Info.ChunkFinished = Cf2ChunkFinished {
                dwStreamNumber,
                dwFlags: 0,
                hSourceFile,
                hDestinationFile,
                uliChunkNumber: a.chunk_number,
                uliChunkSize: total_done.saturating_sub(a.last_total),
                uliStreamSize: stream_size,
                uliStreamBytesTransferred: stream_done,
                uliTotalFileSize: total_file,
                uliTotalBytesTransferred: total_done,
            };
            a.last_total = total_done;
        }
        CALLBACK_STREAM_SWITCH_ => {
            msg.Type = COPYFILE2_CALLBACK_STREAM_STARTED;
            msg.Info.StreamStarted = Cf2StreamStarted {
                dwStreamNumber,
                dwReserved: 0,
                hSourceFile,
                hDestinationFile,
                uliStreamSize: stream_size,
                uliTotalFileSize: total_file,
            };
        }
        _ => return 0, // PROGRESS_CONTINUE
    }

    match (a.routine)(&msg, a.context) {
        COPYFILE2_PROGRESS_CANCEL => PROGRESS_CANCEL_,
        // PAUSE would let the caller resume later; no resumable copy exists, so it stops.
        COPYFILE2_PROGRESS_STOP | COPYFILE2_PROGRESS_PAUSE => PROGRESS_STOP_,
        COPYFILE2_PROGRESS_QUIET => PROGRESS_QUIET_,
        _ => 0,
    }
}

/// Wine's CopyFile2 runs the CopyFileExW engine, ignoring the progress routine; this
/// honours it (as events mapped from CopyFileExW's own callback) and pfCancel as well.
/// Divergence from Wine: dwSize is validated (Windows rejects a wrong size) and a NULL file
/// name is an invalid parameter instead of a crash.
#[no_mangle]
pub unsafe extern "system" fn CopyFile2(
    pwszExistingFileName: PCWSTR,
    pwszNewFileName: PCWSTR,
    pExtendedParameters: *mut Copyfile2ExtendedParameters,
) -> HRESULT {
    if pwszExistingFileName.is_null() || pwszNewFileName.is_null() {
        return hresult_from_win32(ERROR_INVALID_PARAMETER_);
    }

    let mut flags: DWORD = 0;
    let mut cancel: *mut BOOL = ptr::null_mut();
    let mut adapter: Option<Cf2Adapter> = None;

    if !pExtendedParameters.is_null() {
        let pp = &*pExtendedParameters;
        if (pp.dwSize as usize) < size_of::<Copyfile2ExtendedParameters>() {
            return hresult_from_win32(ERROR_INVALID_PARAMETER_);
        }
        flags = pp.dwCopyFlags;
        cancel = pp.pfCancel;
        if let Some(routine) = pp.pProgressRoutine {
            adapter = Some(Cf2Adapter {
                routine,
                context: pp.pvCallbackContext,
                chunk_number: 0,
                last_total: 0,
            });
        }
    }

    // `adapter` outlives the CopyFileExW call below, which is the only user of the pointer.
    let (progress, data): (LPPROGRESS_ROUTINE, LPVOID) = match adapter.as_mut() {
        Some(a) => (Some(cf2_progress_adapter), a as *mut Cf2Adapter as LPVOID),
        None => (None, ptr::null_mut()),
    };
    let ok = crate::file::copy::CopyFileExW(
        pwszExistingFileName,
        pwszNewFileName,
        progress,
        data,
        cancel,
        flags,
    );
    if ok != 0 {
        S_OK_
    } else {
        hresult_from_win32(GetLastError())
    }
}

/* ------------------------------------------------------------------------- *
 * Process mitigation policies
 *
 * PROCESS_MITIGATION_POLICY values (winnt.h): 0 DEP, 1 ASLR, 2 DynamicCode,
 * 3 StrictHandleCheck, 4 SystemCallDisable, 5 MitigationOptionsMask, 6 ExtensionPointDisable,
 * 7 ControlFlowGuard, 8 Signature, 9 FontDisable, 10 ImageLoad, 11 SystemCallFilter,
 * 12 PayloadRestriction, 13 ChildProcess, 14 SideChannelIsolation, 15 UserShadowStack,
 * 16 RedirectionTrust, 17 UserPointerAuth, 18 SEHOP; Max = 19 (measured).
 *
 * This system enforces none of the per-process mitigations. Get therefore reports "no
 * mitigation active" (zeroed structs, with DEP taken from ProcessExecuteFlags when the
 * kernel answers). Set refuses to claim an enforcement it cannot provide.
 * ------------------------------------------------------------------------- */

const MEM_EXECUTE_OPTION_DISABLE: u32 = 0x1; // DEP on
const MEM_EXECUTE_OPTION_DISABLE_THUNK_EMULATION: u32 = 0x4;
const MEM_EXECUTE_OPTION_PERMANENT: u32 = 0x8;

/// Struct size for a policy class (PROCESS_MITIGATION_DEP_POLICY is {DWORD flags;
/// BOOLEAN Permanent} = 8, every other class is one DWORD of flags = 4; measured with
/// llvm-mingw on x64 and i386). None = not a class this API accepts.
fn mitigation_policy_size(policy: i32) -> Option<usize> {
    match policy {
        0 => Some(8),
        // ProcessMitigationOptionsMask is a 3x ULONG64 blob handled by NtSetInformationProcess
        // only; Windows rejects it here.
        5 => None,
        1..=18 => Some(4),
        _ => None,
    }
}

#[no_mangle]
pub unsafe extern "system" fn GetProcessMitigationPolicy(
    hProcess: HANDLE,
    Policy: i32,
    lpBuffer: PVOID,
    dwLength: SIZE_T,
) -> BOOL {
    let size = match mitigation_policy_size(Policy) {
        Some(s) => s,
        None => {
            SetLastError(ERROR_INVALID_PARAMETER_);
            return FALSE;
        }
    };
    if lpBuffer.is_null() || dwLength as usize != size {
        SetLastError(ERROR_INVALID_PARAMETER_);
        return FALSE;
    }

    // Fail on a bad process handle like Windows does.
    let mut pbi: PROCESS_BASIC_INFORMATION = core::mem::zeroed();
    let status = NtQueryInformationProcess(
        hProcess,
        ProcessBasicInformation,
        &mut pbi as *mut _ as PVOID,
        size_of::<PROCESS_BASIC_INFORMATION>() as ULONG,
        ptr::null_mut(),
    );
    if !NT_SUCCESS(status) {
        BaseSetLastNTError(status);
        return FALSE;
    }

    ptr::write_bytes(lpBuffer as *mut u8, 0, size);
    if Policy == 0 {
        let mut exec_flags: ULONG = 0;
        let status = NtQueryInformationProcess(
            hProcess,
            ProcessExecuteFlags,
            &mut exec_flags as *mut _ as PVOID,
            size_of::<ULONG>() as ULONG,
            ptr::null_mut(),
        );
        if NT_SUCCESS(status) {
            // PROCESS_MITIGATION_DEP_POLICY: Flags bit0 Enable, bit1 DisableAtlThunkEmulation;
            // then BOOLEAN Permanent at offset 4.
            let mut f: u32 = 0;
            if exec_flags & MEM_EXECUTE_OPTION_DISABLE != 0 {
                f |= 1;
            }
            if exec_flags & MEM_EXECUTE_OPTION_DISABLE_THUNK_EMULATION != 0 {
                f |= 2;
            }
            ptr::write(lpBuffer as *mut u32, f);
            *(lpBuffer as *mut u8).add(4) = (exec_flags & MEM_EXECUTE_OPTION_PERMANENT != 0) as u8;
        }
        // If the kernel does not implement ProcessExecuteFlags the policy stays zeroed.
    }
    TRUE
}

#[no_mangle]
pub unsafe extern "system" fn SetProcessMitigationPolicy(
    Policy: i32,
    lpBuffer: PVOID,
    dwLength: SIZE_T,
) -> BOOL {
    let size = match mitigation_policy_size(Policy) {
        Some(s) => s,
        None => {
            SetLastError(ERROR_INVALID_PARAMETER_);
            return FALSE;
        }
    };
    if lpBuffer.is_null() || dwLength as usize != size {
        SetLastError(ERROR_INVALID_PARAMETER_);
        return FALSE;
    }
    // DEP: 64-bit Windows always refuses to change it through this API
    // (ERROR_NOT_SUPPORTED), and nothing here can enforce a 32-bit change either.
    if Policy == 0 {
        SetLastError(ERROR_NOT_SUPPORTED_);
        return FALSE;
    }
    // Every other class is a one-DWORD flag set that turns restrictions on. A request that
    // turns nothing on matches the state Get reports, so it is accepted as a no-op; a
    // request for a restriction cannot be honoured and is refused rather than pretended.
    let flags = ptr::read(lpBuffer as *const u32);
    if flags != 0 {
        SetLastError(ERROR_NOT_SUPPORTED_);
        return FALSE;
    }
    TRUE
}

/* ------------------------------------------------------------------------- *
 * ResolveLocaleName, SetWaitableTimerEx, UnmapViewOfFileEx
 * ------------------------------------------------------------------------- */

extern "system" {
    fn GetLocaleInfoEx(lpLocaleName: LPCWSTR, LCType: DWORD, lpLCData: LPWSTR, cchData: i32) -> i32;
    fn IsValidLocaleName(lpLocaleName: LPCWSTR) -> BOOL;
}

const LOCALE_NAME_MAX_LENGTH_: usize = 85;
const LOCALE_SNAME_: u32 = 0x5C;
const LOCALE_SSORTLOCALE_: u32 = 0x7B;

#[inline]
fn is_locale_name_char(c: u16) -> bool {
    (c >= b'A' as u16 && c <= b'Z' as u16)
        || (c >= b'a' as u16 && c <= b'z' as u16)
        || (c >= b'0' as u16 && c <= b'9' as u16)
        || c == b'-' as u16
        || c == b'_' as u16
}

/// Resolve an arbitrary locale name to the closest supported one by trimming trailing
/// "-xx"/"_xx" subtags (Wine kernelbase/locale.c). Returns the length including the NUL;
/// an unresolvable name yields an empty string (length 1), like Windows.
#[no_mangle]
pub unsafe extern "system" fn ResolveLocaleName(
    lpNameToResolve: LPCWSTR,
    lpLocaleName: LPWSTR,
    cchLocaleName: i32,
) -> i32 {
    if lpNameToResolve.is_null() || cchLocaleName < 0 || (cchLocaleName > 0 && lpLocaleName.is_null()) {
        SetLastError(ERROR_INVALID_PARAMETER_);
        return 0;
    }

    let mut tmp = [0u16; LOCALE_NAME_MAX_LENGTH_];
    let mut found = false;
    if IsValidLocaleName(lpNameToResolve) != 0 {
        let mut n = 0usize;
        while n < LOCALE_NAME_MAX_LENGTH_ - 1 && *lpNameToResolve.add(n) != 0 {
            tmp[n] = *lpNameToResolve.add(n);
            n += 1;
        }
        found = true;
    } else {
        // Reject names with characters no locale name can contain.
        let mut i = 0usize;
        while *lpNameToResolve.add(i) != 0 {
            if !is_locale_name_char(*lpNameToResolve.add(i)) {
                SetLastError(ERROR_INVALID_PARAMETER_);
                return 0;
            }
            i += 1;
        }
        // lstrcpynW(tmp, name, LOCALE_NAME_MAX_LENGTH)
        let mut n = 0usize;
        while n < LOCALE_NAME_MAX_LENGTH_ - 1 && *lpNameToResolve.add(n) != 0 {
            tmp[n] = *lpNameToResolve.add(n);
            n += 1;
        }
        tmp[n] = 0;
        let mut len = n;
        while len > 0 {
            // Drop the last subtag.
            let mut p = len;
            while p > 0 && tmp[p - 1] != b'-' as u16 && tmp[p - 1] != b'_' as u16 {
                p -= 1;
            }
            if p <= 1 {
                break;
            }
            tmp[p - 1] = 0;
            len = p - 1;
            if IsValidLocaleName(tmp.as_ptr()) != 0 {
                found = true;
                break;
            }
        }
    }

    // Canonical spelling: the sort locale for a neutral name, the locale's own name
    // otherwise (Wine's `inotneutral ? sname : ssortlocale`). Asking for the sort locale
    // first and falling back to the name gives the same answer for the locales we ship.
    let mut res = [0u16; LOCALE_NAME_MAX_LENGTH_];
    let mut datalen: i32 = 1;
    if found {
        let mut n = GetLocaleInfoEx(tmp.as_ptr(), LOCALE_SSORTLOCALE_, res.as_mut_ptr(), res.len() as i32);
        if n <= 1 {
            n = GetLocaleInfoEx(tmp.as_ptr(), LOCALE_SNAME_, res.as_mut_ptr(), res.len() as i32);
        }
        if n > 1 {
            datalen = n;
        } else {
            res[0] = 0;
        }
    }

    if cchLocaleName == 0 {
        return datalen;
    }
    // lstrcpynW(buffer, res, len): copy up to len-1 chars and always NUL-terminate.
    let mut i = 0usize;
    while i + 1 < cchLocaleName as usize && res[i] != 0 {
        *lpLocaleName.add(i) = res[i];
        i += 1;
    }
    *lpLocaleName.add(i) = 0;
    if datalen > cchLocaleName {
        SetLastError(ERROR_INSUFFICIENT_BUFFER_);
        return 0;
    }
    datalen
}

/// Wine: semi-stub over SetWaitableTimer. WakeContext (power-request reason) and
/// TolerableDelay (coalescing hint, ms) are advisory and the timer has no coalescing, so
/// they are accepted and ignored; the timer fires on the regular schedule, which is a
/// valid outcome of any non-zero tolerance.
#[no_mangle]
pub unsafe extern "system" fn SetWaitableTimerEx(
    hTimer: HANDLE,
    lpDueTime: *const LARGE_INTEGER,
    lPeriod: LONG,
    pfnCompletionRoutine: crate::client::synch::PTIMERAPCROUTINE,
    lpArgToCompletionRoutine: LPVOID,
    WakeContext: *mut c_void,
    TolerableDelay: ULONG,
) -> BOOL {
    let _ = (WakeContext, TolerableDelay);
    crate::client::synch::SetWaitableTimer(
        hTimer,
        lpDueTime,
        lPeriod,
        pfnCompletionRoutine,
        lpArgToCompletionRoutine,
        FALSE,
    )
}

const MEM_UNMAP_WITH_TRANSIENT_BOOST_: u32 = 0x1;
const MEM_PRESERVE_PLACEHOLDER_: u32 = 0x2;

/// UnmapViewOfFile plus flags.
/// - MEM_UNMAP_WITH_TRANSIENT_BOOST: a working-set priority hint for the pages being
///   released; ignoring it is correct (the unmap itself is unchanged).
/// - MEM_PRESERVE_PLACEHOLDER: converts the view into a placeholder reservation. There is
///   no placeholder support (no NtUnmapViewOfSectionEx / MapViewOfFile3), so it is refused
///   with ERROR_NOT_SUPPORTED instead of silently freeing the range.
/// - Any other bit: ERROR_INVALID_PARAMETER.
#[no_mangle]
pub unsafe extern "system" fn UnmapViewOfFileEx(BaseAddress: PVOID, UnmapFlags: ULONG) -> BOOL {
    if UnmapFlags & !(MEM_UNMAP_WITH_TRANSIENT_BOOST_ | MEM_PRESERVE_PLACEHOLDER_) != 0 {
        SetLastError(ERROR_INVALID_PARAMETER_);
        return FALSE;
    }
    if UnmapFlags & MEM_PRESERVE_PLACEHOLDER_ != 0 {
        SetLastError(ERROR_NOT_SUPPORTED_);
        return FALSE;
    }
    crate::file::filemap::UnmapViewOfFile(BaseAddress as LPCVOID)
}

/* ------------------------------------------------------------------------- *
 * AppModel (package) APIs. Return LONG error codes, never SetLastError.
 *
 * Every process here is unpackaged and no package is installed:
 *  - "current/process package" queries: APPMODEL_ERROR_NO_PACKAGE (15700), as Wine returns;
 *  - lookups of a package by full name: APPMODEL_ERROR_NO_PACKAGE for the path query (Wine),
 *    ERROR_NOT_FOUND for the ones Wine leaves unimplemented (a missing package is "not
 *    found", and nothing hands out a PACKAGE_INFO_REFERENCE, so those are invalid);
 *  - family enumeration: success with zero packages (Wine).
 * ------------------------------------------------------------------------- */

#[no_mangle]
pub unsafe extern "system" fn GetCurrentPackageFullName(length: *mut UINT32, name: *mut u16) -> LONG {
    let _ = (length, name);
    APPMODEL_ERROR_NO_PACKAGE_ as LONG
}

#[no_mangle]
pub unsafe extern "system" fn GetCurrentPackageFamilyName(length: *mut UINT32, name: *mut u16) -> LONG {
    let _ = (length, name);
    APPMODEL_ERROR_NO_PACKAGE_ as LONG
}

#[no_mangle]
pub unsafe extern "system" fn GetCurrentPackagePath(length: *mut UINT32, path: *mut u16) -> LONG {
    let _ = (length, path);
    APPMODEL_ERROR_NO_PACKAGE_ as LONG
}

#[no_mangle]
pub unsafe extern "system" fn GetCurrentPackageInfo(
    flags: UINT32,
    buffer_size: *mut UINT32,
    buffer: *mut BYTE,
    count: *mut UINT32,
) -> LONG {
    let _ = (flags, buffer_size, buffer, count);
    APPMODEL_ERROR_NO_PACKAGE_ as LONG
}

/// Returns the "package virtualization context" of the current package: NULL when the
/// process is not packaged.
#[no_mangle]
pub unsafe extern "system" fn GetCurrentPackageVirtualizationContext() -> PVOID {
    ptr::null_mut()
}

/// Shared by GetPackageFullName / GetPackageFamilyName: the process handle is validated
/// (a bad handle is a real error), after which an unpackaged process has no package.
unsafe fn process_package_query(hProcess: HANDLE, length: *mut UINT32) -> LONG {
    if length.is_null() {
        return ERROR_INVALID_PARAMETER_ as LONG;
    }
    let mut pbi: PROCESS_BASIC_INFORMATION = core::mem::zeroed();
    let status = NtQueryInformationProcess(
        hProcess,
        ProcessBasicInformation,
        &mut pbi as *mut _ as PVOID,
        size_of::<PROCESS_BASIC_INFORMATION>() as ULONG,
        ptr::null_mut(),
    );
    if !NT_SUCCESS(status) {
        return RtlNtStatusToDosError(status) as LONG;
    }
    APPMODEL_ERROR_NO_PACKAGE_ as LONG
}

#[no_mangle]
pub unsafe extern "system" fn GetPackageFullName(
    hProcess: HANDLE,
    length: *mut UINT32,
    name: *mut u16,
) -> LONG {
    let _ = name;
    process_package_query(hProcess, length)
}

#[no_mangle]
pub unsafe extern "system" fn GetPackageFamilyName(
    hProcess: HANDLE,
    length: *mut UINT32,
    name: *mut u16,
) -> LONG {
    let _ = name;
    process_package_query(hProcess, length)
}

#[no_mangle]
pub unsafe extern "system" fn GetPackagePathByFullName(
    packageFullName: PCWSTR,
    pathLength: *mut UINT32,
    path: *mut u16,
) -> LONG {
    let _ = path;
    if packageFullName.is_null() || pathLength.is_null() {
        return ERROR_INVALID_PARAMETER_ as LONG;
    }
    APPMODEL_ERROR_NO_PACKAGE_ as LONG
}

#[no_mangle]
pub unsafe extern "system" fn GetStagedPackagePathByFullName(
    packageFullName: PCWSTR,
    pathLength: *mut UINT32,
    path: *mut u16,
) -> LONG {
    let _ = path;
    if packageFullName.is_null() || pathLength.is_null() {
        return ERROR_INVALID_PARAMETER_ as LONG;
    }
    ERROR_NOT_FOUND_ as LONG
}

/// GetPackagePath(const PACKAGE_ID*, UINT32 reserved, UINT32 *pathLength, PWSTR path).
#[no_mangle]
pub unsafe extern "system" fn GetPackagePath(
    packageId: *const c_void,
    reserved: UINT32,
    pathLength: *mut UINT32,
    path: *mut u16,
) -> LONG {
    let _ = path;
    if packageId.is_null() || pathLength.is_null() || reserved != 0 {
        return ERROR_INVALID_PARAMETER_ as LONG;
    }
    ERROR_NOT_FOUND_ as LONG
}

/// Shared tail of GetPackagesByPackageFamily / FindPackagesByPackageFamily: no package of
/// any family is installed. Success, zero packages, zero characters (Wine).
unsafe fn no_packages_in_family(count: *mut UINT32, bufferLength: *mut UINT32) -> LONG {
    if count.is_null() || bufferLength.is_null() {
        return ERROR_INVALID_PARAMETER_ as LONG;
    }
    *count = 0;
    *bufferLength = 0;
    ERROR_SUCCESS_ as LONG
}

#[no_mangle]
pub unsafe extern "system" fn GetPackagesByPackageFamily(
    packageFamilyName: PCWSTR,
    count: *mut UINT32,
    packageFullNames: *mut *mut u16,
    bufferLength: *mut UINT32,
    buffer: *mut u16,
) -> LONG {
    let _ = (packageFullNames, buffer);
    if packageFamilyName.is_null() {
        return ERROR_INVALID_PARAMETER_ as LONG;
    }
    no_packages_in_family(count, bufferLength)
}

#[no_mangle]
pub unsafe extern "system" fn FindPackagesByPackageFamily(
    packageFamilyName: PCWSTR,
    packageFilters: UINT32,
    count: *mut UINT32,
    packageFullNames: *mut *mut u16,
    bufferLength: *mut UINT32,
    buffer: *mut u16,
    packageProperties: *mut UINT32,
) -> LONG {
    let _ = (packageFilters, packageFullNames, buffer, packageProperties);
    if packageFamilyName.is_null() {
        return ERROR_INVALID_PARAMETER_ as LONG;
    }
    no_packages_in_family(count, bufferLength)
}

/// OpenPackageInfoByFullName(PCWSTR, UINT32 reserved, PACKAGE_INFO_REFERENCE *).
#[no_mangle]
pub unsafe extern "system" fn OpenPackageInfoByFullName(
    packageFullName: PCWSTR,
    reserved: UINT32,
    packageInfoReference: *mut PVOID,
) -> LONG {
    if packageFullName.is_null() || packageInfoReference.is_null() || reserved != 0 {
        return ERROR_INVALID_PARAMETER_ as LONG;
    }
    *packageInfoReference = ptr::null_mut();
    ERROR_NOT_FOUND_ as LONG
}

// No PACKAGE_INFO_REFERENCE can exist (OpenPackageInfoByFullName never returns one), so
// operations on one are invalid-parameter errors.
#[no_mangle]
pub unsafe extern "system" fn ClosePackageInfo(packageInfoReference: PVOID) -> LONG {
    let _ = packageInfoReference;
    ERROR_INVALID_PARAMETER_ as LONG
}

#[no_mangle]
pub unsafe extern "system" fn GetPackageInfo(
    packageInfoReference: PVOID,
    flags: UINT32,
    bufferLength: *mut UINT32,
    buffer: *mut BYTE,
    count: *mut UINT32,
) -> LONG {
    let _ = (packageInfoReference, flags, bufferLength, buffer, count);
    ERROR_INVALID_PARAMETER_ as LONG
}

#[no_mangle]
pub unsafe extern "system" fn GetPackageId(
    packageInfoReference: PVOID,
    bufferLength: *mut UINT32,
    buffer: *mut BYTE,
) -> LONG {
    let _ = (packageInfoReference, bufferLength, buffer);
    ERROR_INVALID_PARAMETER_ as LONG
}

#[no_mangle]
pub unsafe extern "system" fn GetPackageApplicationIds(
    packageInfoReference: PVOID,
    bufferLength: *mut UINT32,
    buffer: *mut BYTE,
    count: *mut UINT32,
) -> LONG {
    let _ = (packageInfoReference, bufferLength, buffer, count);
    ERROR_INVALID_PARAMETER_ as LONG
}

// Package names: Name_Version_Architecture_ResourceId_PublisherId; family name is
// Name_PublisherId; PublisherId is always 13 characters.
const PACKAGE_PUBLISHERID_LEN: usize = 13;
const PACKAGE_NAME_MAX_LEN: usize = 50;

unsafe fn wide_len(p: *const u16) -> usize {
    wcslen(p)
}

/// Copy `parts` (NUL-free wide slices, concatenated) into `out`, honouring the Windows
/// in/out length protocol: *length is the capacity on entry and the required size
/// (including the NUL) on exit.
unsafe fn emit_wide(parts: &[(*const u16, usize)], length: *mut UINT32, out: *mut u16) -> LONG {
    let total: usize = parts.iter().map(|p| p.1).sum();
    let capacity = *length as usize;
    *length = (total + 1) as UINT32;
    if out.is_null() || capacity < total + 1 {
        return ERROR_INSUFFICIENT_BUFFER_ as LONG;
    }
    let mut at = out;
    for &(src, n) in parts {
        ptr::copy_nonoverlapping(src, at, n);
        at = at.add(n);
    }
    *at = 0;
    ERROR_SUCCESS_ as LONG
}

#[no_mangle]
pub unsafe extern "system" fn PackageFamilyNameFromFullName(
    packageFullName: PCWSTR,
    packageFamilyNameLength: *mut UINT32,
    packageFamilyName: *mut u16,
) -> LONG {
    if packageFullName.is_null() || packageFamilyNameLength.is_null() {
        return ERROR_INVALID_PARAMETER_ as LONG;
    }
    // Four underscores separate the five fields; the last is the publisher id.
    let mut field_start = [packageFullName; 5];
    let mut at = packageFullName;
    for i in 1..5 {
        let us = wcschr(at, b'_' as u16);
        if us.is_null() {
            return ERROR_INVALID_PARAMETER_ as LONG;
        }
        at = us.add(1);
        field_start[i] = at;
    }
    let name_len = field_start[1].offset_from(field_start[0]) as usize - 1;
    let publisher = field_start[4];
    if name_len == 0 || name_len > PACKAGE_NAME_MAX_LEN || wide_len(publisher) != PACKAGE_PUBLISHERID_LEN {
        return ERROR_INVALID_PARAMETER_ as LONG;
    }
    let underscore: u16 = b'_' as u16;
    emit_wide(
        &[
            (packageFullName, name_len),
            (&underscore as *const u16, 1),
            (publisher, PACKAGE_PUBLISHERID_LEN),
        ],
        packageFamilyNameLength,
        packageFamilyName,
    )
}

#[no_mangle]
pub unsafe extern "system" fn PackageNameAndPublisherIdFromFamilyName(
    packageFamilyName: PCWSTR,
    packageNameLength: *mut UINT32,
    packageName: *mut u16,
    packagePublisherIdLength: *mut UINT32,
    packagePublisherId: *mut u16,
) -> LONG {
    if packageFamilyName.is_null() || packageNameLength.is_null() || packagePublisherIdLength.is_null() {
        return ERROR_INVALID_PARAMETER_ as LONG;
    }
    let us = wcschr(packageFamilyName, b'_' as u16);
    if us.is_null() {
        return ERROR_INVALID_PARAMETER_ as LONG;
    }
    let name_len = us.offset_from(packageFamilyName) as usize;
    let publisher = us.add(1);
    if name_len == 0 || name_len > PACKAGE_NAME_MAX_LEN || wide_len(publisher) != PACKAGE_PUBLISHERID_LEN {
        return ERROR_INVALID_PARAMETER_ as LONG;
    }
    // Both lengths are always reported, then both buffers are filled or neither is.
    let name_cap = *packageNameLength as usize;
    let pub_cap = *packagePublisherIdLength as usize;
    *packageNameLength = (name_len + 1) as UINT32;
    *packagePublisherIdLength = (PACKAGE_PUBLISHERID_LEN + 1) as UINT32;
    if packageName.is_null()
        || packagePublisherId.is_null()
        || name_cap < name_len + 1
        || pub_cap < PACKAGE_PUBLISHERID_LEN + 1
    {
        return ERROR_INSUFFICIENT_BUFFER_ as LONG;
    }
    ptr::copy_nonoverlapping(packageFamilyName, packageName, name_len);
    *packageName.add(name_len) = 0;
    ptr::copy_nonoverlapping(publisher, packagePublisherId, PACKAGE_PUBLISHERID_LEN);
    *packagePublisherId.add(PACKAGE_PUBLISHERID_LEN) = 0;
    ERROR_SUCCESS_ as LONG
}

/* ------------------------------------------------------------------------- *
 * Pss* (process snapshotting). Return DWORD error codes, not BOOL.
 *
 * Capturing needs the kernel to clone the target's address space and enumerate its
 * handles/threads/VA space; this kernel has neither, so PssCaptureSnapshot fails with
 * ERROR_NOT_SUPPORTED (Wine returns ERROR_NOT_FOUND, which reads as a transient condition).
 * Since no HPSS / HPSSWALK can therefore exist, every operation on one is an invalid
 * handle (NULL is an invalid parameter), where Wine "succeeds" in freeing a snapshot it
 * never made.
 * ------------------------------------------------------------------------- */

unsafe fn pss_bad_handle(handle: HANDLE) -> DWORD {
    if handle.is_null() {
        ERROR_INVALID_PARAMETER_
    } else {
        ERROR_INVALID_HANDLE_
    }
}

#[no_mangle]
pub unsafe extern "system" fn PssCaptureSnapshot(
    ProcessHandle: HANDLE,
    CaptureFlags: DWORD,
    ThreadContextFlags: DWORD,
    SnapshotHandle: *mut HANDLE,
) -> DWORD {
    let _ = (ProcessHandle, CaptureFlags, ThreadContextFlags);
    if SnapshotHandle.is_null() {
        return ERROR_INVALID_PARAMETER_;
    }
    *SnapshotHandle = ptr::null_mut();
    ERROR_NOT_SUPPORTED_
}

#[no_mangle]
pub unsafe extern "system" fn PssDuplicateSnapshot(
    SourceProcessHandle: HANDLE,
    SnapshotHandle: HANDLE,
    TargetProcessHandle: HANDLE,
    TargetSnapshotHandle: *mut HANDLE,
    Flags: DWORD,
) -> DWORD {
    let _ = (SourceProcessHandle, TargetProcessHandle, Flags);
    if TargetSnapshotHandle.is_null() {
        return ERROR_INVALID_PARAMETER_;
    }
    *TargetSnapshotHandle = ptr::null_mut();
    pss_bad_handle(SnapshotHandle)
}

#[no_mangle]
pub unsafe extern "system" fn PssFreeSnapshot(ProcessHandle: HANDLE, SnapshotHandle: HANDLE) -> DWORD {
    let _ = ProcessHandle;
    pss_bad_handle(SnapshotHandle)
}

#[no_mangle]
pub unsafe extern "system" fn PssQuerySnapshot(
    SnapshotHandle: HANDLE,
    InformationClass: DWORD,
    Buffer: PVOID,
    BufferLength: DWORD,
) -> DWORD {
    let _ = (InformationClass, Buffer, BufferLength);
    pss_bad_handle(SnapshotHandle)
}

#[no_mangle]
pub unsafe extern "system" fn PssWalkMarkerCreate(Allocator: *const c_void, WalkMarkerHandle: *mut HANDLE) -> DWORD {
    let _ = Allocator;
    if WalkMarkerHandle.is_null() {
        return ERROR_INVALID_PARAMETER_;
    }
    *WalkMarkerHandle = ptr::null_mut();
    // A marker is only useful for walking a snapshot, and none can be captured.
    ERROR_NOT_SUPPORTED_
}

#[no_mangle]
pub unsafe extern "system" fn PssWalkMarkerFree(WalkMarkerHandle: HANDLE) -> DWORD {
    pss_bad_handle(WalkMarkerHandle)
}

#[no_mangle]
pub unsafe extern "system" fn PssWalkMarkerGetPosition(WalkMarkerHandle: HANDLE, Position: *mut ULONG_PTR) -> DWORD {
    if Position.is_null() {
        return ERROR_INVALID_PARAMETER_;
    }
    pss_bad_handle(WalkMarkerHandle)
}

#[no_mangle]
pub unsafe extern "system" fn PssWalkMarkerSetPosition(WalkMarkerHandle: HANDLE, Position: ULONG_PTR) -> DWORD {
    let _ = Position;
    pss_bad_handle(WalkMarkerHandle)
}

#[no_mangle]
pub unsafe extern "system" fn PssWalkMarkerSeekToBeginning(WalkMarkerHandle: HANDLE) -> DWORD {
    pss_bad_handle(WalkMarkerHandle)
}

#[no_mangle]
pub unsafe extern "system" fn PssWalkSnapshot(
    SnapshotHandle: HANDLE,
    InformationClass: DWORD,
    WalkMarkerHandle: HANDLE,
    Buffer: PVOID,
    BufferLength: DWORD,
) -> DWORD {
    let _ = (InformationClass, WalkMarkerHandle, Buffer, BufferLength);
    pss_bad_handle(SnapshotHandle)
}
