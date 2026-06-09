//! Rust translation of `sdk/lib/rtl/ppb.c` — process-parameter block
//! (`RTL_USER_PROCESS_PARAMETERS`) management:
//!   * `RtlCreateProcessParameters`   — allocate + pack a parameter block
//!   * `RtlDestroyProcessParameters`  — free it
//!   * `RtlDeNormalizeProcessParams`  — buffer pointers → offsets
//!   * `RtlNormalizeProcessParams`    — offsets → buffer pointers
//!
//! The block is a single heap allocation with all the UNICODE_STRING
//! buffers packed in a tail region; "normalized" form stores absolute
//! pointers, "denormalized" stores offsets-from-base (so it can be handed
//! to a freshly-created process at a different address). The layout is
//! therefore byte-exact and is taken from ntkrnl-sys (size-asserted).
//!
//! Faithful to ReactOS ppb.c (Ariadne). Shared kernel+user: the PEB is
//! only touched on the UserMode (`RtlpGetMode`) branch.
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case, non_upper_case_globals)]

use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{
    HANDLE, KPROCESSOR_MODE, PPEB, PRTL_USER_PROCESS_PARAMETERS, PUNICODE_STRING, UNICODE_STRING,
    _TEB,
};

// --- constants not surfaced by ntkrnl-sys (fixed winnt.h / pstypes values). ---
const MAX_PATH: u32 = 260;
const HEAP_ZERO_MEMORY: u32 = 0x0000_0008;
const RTL_USER_PROCESS_PARAMETERS_NORMALIZED: u32 = 0x0000_0001;
const UserMode: KPROCESSOR_MODE = 1;

extern "system" {
    fn RtlAcquirePebLock();
    fn RtlReleasePebLock();
    fn RtlpGetMode() -> KPROCESSOR_MODE;
    // NTAPI (stdcall on i386). Declared extern "system" to match the
    // rtl-heap exports (NOT bindgen's extern "C", which mis-decorates @N
    // on i386). See heap.rs.
    fn RtlAllocateHeap(HeapHandle: PVOID, Flags: ULONG, Size: SIZE_T) -> PVOID;
    fn RtlFreeHeap(HeapHandle: PVOID, Flags: ULONG, BaseAddress: PVOID) -> BOOLEAN;
}

/// `NtCurrentPeb()` — TEB->ProcessEnvironmentBlock (FORCEINLINE in headers).
#[inline]
unsafe fn NtCurrentPeb() -> PPEB {
    (*(ntkrnl_sys::inline::ke::NtCurrentTeb() as *mut _TEB)).ProcessEnvironmentBlock
}

/// `RtlGetProcessHeap()` — PEB->ProcessHeap (FORCEINLINE in headers).
#[inline]
unsafe fn RtlGetProcessHeap() -> PVOID {
    (*NtCurrentPeb()).ProcessHeap
}

/// `ALIGN(x, align)` macro: round `x` up to a multiple of `align`.
#[inline]
fn ALIGN(x: u32, align: u32) -> u32 {
    (x + align - 1) & !(align - 1)
}

const SIZEOF_WCHAR: usize = 2;
const SIZEOF_ULONG: u32 = 4;

/// `static __inline RtlpCopyParameterString` — copy `Source` into the
/// packed tail region at `*Ptr`, NUL-terminate, advance `*Ptr`.
unsafe fn RtlpCopyParameterString(
    Ptr: *mut *mut u16,
    Destination: PUNICODE_STRING,
    Source: PUNICODE_STRING,
    Size: u16,
) {
    (*Destination).Length = (*Source).Length;
    (*Destination).MaximumLength = if Size != 0 {
        Size
    } else {
        (*Source).MaximumLength
    };
    (*Destination).Buffer = *Ptr;
    if (*Source).Length != 0 {
        // memmove: Length is in bytes.
        core::ptr::copy(
            (*Source).Buffer as *const u8,
            (*Destination).Buffer as *mut u8,
            (*Source).Length as usize,
        );
    }
    // Destination->Buffer[Length / sizeof(WCHAR)] = 0;
    *(*Destination)
        .Buffer
        .add((*Destination).Length as usize / SIZEOF_WCHAR) = 0;
    // *Ptr += MaximumLength / sizeof(WCHAR);
    *Ptr = (*Ptr).add((*Destination).MaximumLength as usize / SIZEOF_WCHAR);
}

/// `NTSTATUS NTAPI RtlCreateProcessParameters(...)`.
#[no_mangle]
pub unsafe extern "system" fn RtlCreateProcessParameters(
    ProcessParameters: *mut PRTL_USER_PROCESS_PARAMETERS,
    ImagePathName: PUNICODE_STRING,
    mut DllPath: PUNICODE_STRING,
    mut CurrentDirectory: PUNICODE_STRING,
    mut CommandLine: PUNICODE_STRING,
    mut Environment: PWSTR,
    mut WindowTitle: PUNICODE_STRING,
    mut DesktopInfo: PUNICODE_STRING,
    mut ShellInfo: PUNICODE_STRING,
    mut RuntimeData: PUNICODE_STRING,
) -> NTSTATUS {
    let Param: PRTL_USER_PROCESS_PARAMETERS;
    let mut Length: u32;
    let Dest: *mut u16;
    let mut EmptyString: UNICODE_STRING = core::mem::zeroed();
    let CurrentDirectoryHandle: HANDLE;
    let ConsoleHandle: HANDLE;
    let ConsoleFlags: ULONG;

    RtlAcquirePebLock();

    // L"" — a single NUL wchar; only the pointer is stored (never read for
    // a zero-Length source), so a shared static is fine.
    static EMPTY_WCHAR: u16 = 0;
    EmptyString.Length = 0;
    EmptyString.MaximumLength = SIZEOF_WCHAR as u16;
    EmptyString.Buffer = core::ptr::addr_of!(EMPTY_WCHAR) as *mut u16;

    if RtlpGetMode() == UserMode {
        let pp = (*NtCurrentPeb()).ProcessParameters;
        if DllPath.is_null() {
            DllPath = core::ptr::addr_of_mut!((*pp).DllPath);
        }
        if Environment.is_null() {
            Environment = (*pp).Environment;
        }
        if CurrentDirectory.is_null() {
            CurrentDirectory = core::ptr::addr_of_mut!((*pp).CurrentDirectory.DosPath);
        }
        CurrentDirectoryHandle = (*pp).CurrentDirectory.Handle;
        ConsoleHandle = (*pp).ConsoleHandle;
        ConsoleFlags = (*pp).ConsoleFlags;
    } else {
        if DllPath.is_null() {
            DllPath = core::ptr::addr_of_mut!(EmptyString);
        }
        if CurrentDirectory.is_null() {
            CurrentDirectory = core::ptr::addr_of_mut!(EmptyString);
        }
        CurrentDirectoryHandle = core::ptr::null_mut();
        ConsoleHandle = core::ptr::null_mut();
        ConsoleFlags = 0;
    }

    if CommandLine.is_null() {
        CommandLine = core::ptr::addr_of_mut!(EmptyString);
    }
    if WindowTitle.is_null() {
        WindowTitle = core::ptr::addr_of_mut!(EmptyString);
    }
    if DesktopInfo.is_null() {
        DesktopInfo = core::ptr::addr_of_mut!(EmptyString);
    }
    if ShellInfo.is_null() {
        ShellInfo = core::ptr::addr_of_mut!(EmptyString);
    }
    if RuntimeData.is_null() {
        RuntimeData = core::ptr::addr_of_mut!(EmptyString);
    }

    /* size of process parameter block */
    Length = core::mem::size_of::<ntkrnl_sys::ddk::RTL_USER_PROCESS_PARAMETERS>() as u32;
    /* size of current directory buffer */
    Length += MAX_PATH * SIZEOF_WCHAR as u32;
    /* add string lengths */
    Length += ALIGN((*DllPath).MaximumLength as u32, SIZEOF_ULONG);
    Length += ALIGN((*ImagePathName).Length as u32 + SIZEOF_WCHAR as u32, SIZEOF_ULONG);
    Length += ALIGN((*CommandLine).Length as u32 + SIZEOF_WCHAR as u32, SIZEOF_ULONG);
    Length += ALIGN((*WindowTitle).MaximumLength as u32, SIZEOF_ULONG);
    Length += ALIGN((*DesktopInfo).MaximumLength as u32, SIZEOF_ULONG);
    Length += ALIGN((*ShellInfo).MaximumLength as u32, SIZEOF_ULONG);
    Length += ALIGN((*RuntimeData).MaximumLength as u32, SIZEOF_ULONG);

    /* Allocate the block */
    Param = RtlAllocateHeap(RtlGetProcessHeap(), HEAP_ZERO_MEMORY, Length as SIZE_T)
        as PRTL_USER_PROCESS_PARAMETERS;
    if Param.is_null() {
        RtlReleasePebLock();
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*Param).MaximumLength = Length;
    (*Param).Length = Length;
    (*Param).Flags = RTL_USER_PROCESS_PARAMETERS_NORMALIZED;
    (*Param).Environment = Environment;
    (*Param).CurrentDirectory.Handle = CurrentDirectoryHandle;
    (*Param).ConsoleHandle = ConsoleHandle;
    (*Param).ConsoleFlags = ConsoleFlags;

    Dest = (Param as *mut u8)
        .add(core::mem::size_of::<ntkrnl_sys::ddk::RTL_USER_PROCESS_PARAMETERS>())
        as *mut u16;
    let mut DestPtr = Dest;

    /* copy current directory */
    RtlpCopyParameterString(
        core::ptr::addr_of_mut!(DestPtr),
        core::ptr::addr_of_mut!((*Param).CurrentDirectory.DosPath),
        CurrentDirectory,
        (MAX_PATH * SIZEOF_WCHAR as u32) as u16,
    );

    /* make sure the current directory has a trailing backslash */
    if (*Param).CurrentDirectory.DosPath.Length > 0 {
        Length = (*Param).CurrentDirectory.DosPath.Length as u32 / SIZEOF_WCHAR as u32;
        let buf = (*Param).CurrentDirectory.DosPath.Buffer;
        if *buf.add(Length as usize - 1) != b'\\' as u16 {
            *buf.add(Length as usize) = b'\\' as u16;
            *buf.add(Length as usize + 1) = 0;
            (*Param).CurrentDirectory.DosPath.Length += SIZEOF_WCHAR as u16;
        }
    }

    /* copy dll path */
    RtlpCopyParameterString(
        core::ptr::addr_of_mut!(DestPtr),
        core::ptr::addr_of_mut!((*Param).DllPath),
        DllPath,
        0,
    );

    /* copy image path name */
    RtlpCopyParameterString(
        core::ptr::addr_of_mut!(DestPtr),
        core::ptr::addr_of_mut!((*Param).ImagePathName),
        ImagePathName,
        (*ImagePathName).Length + SIZEOF_WCHAR as u16,
    );

    /* copy command line */
    RtlpCopyParameterString(
        core::ptr::addr_of_mut!(DestPtr),
        core::ptr::addr_of_mut!((*Param).CommandLine),
        CommandLine,
        (*CommandLine).Length + SIZEOF_WCHAR as u16,
    );

    /* copy title */
    RtlpCopyParameterString(
        core::ptr::addr_of_mut!(DestPtr),
        core::ptr::addr_of_mut!((*Param).WindowTitle),
        WindowTitle,
        0,
    );

    /* copy desktop */
    RtlpCopyParameterString(
        core::ptr::addr_of_mut!(DestPtr),
        core::ptr::addr_of_mut!((*Param).DesktopInfo),
        DesktopInfo,
        0,
    );

    /* copy shell info */
    RtlpCopyParameterString(
        core::ptr::addr_of_mut!(DestPtr),
        core::ptr::addr_of_mut!((*Param).ShellInfo),
        ShellInfo,
        0,
    );

    /* copy runtime info */
    RtlpCopyParameterString(
        core::ptr::addr_of_mut!(DestPtr),
        core::ptr::addr_of_mut!((*Param).RuntimeData),
        RuntimeData,
        0,
    );

    RtlDeNormalizeProcessParams(Param);
    *ProcessParameters = Param;
    RtlReleasePebLock();

    STATUS_SUCCESS
}

/// `NTSTATUS NTAPI RtlDestroyProcessParameters(PRTL_USER_PROCESS_PARAMETERS)`.
#[no_mangle]
pub unsafe extern "system" fn RtlDestroyProcessParameters(
    ProcessParameters: PRTL_USER_PROCESS_PARAMETERS,
) -> NTSTATUS {
    RtlFreeHeap(RtlGetProcessHeap(), 0, ProcessParameters as PVOID);
    STATUS_SUCCESS
}

/// `DENORMALIZE(x, addr)` — pointer → offset (skip NULL).
#[inline]
unsafe fn denormalize(buffer: *mut *mut u16, base: PRTL_USER_PROCESS_PARAMETERS) {
    if !(*buffer).is_null() {
        *buffer = ((*buffer as usize) - (base as usize)) as *mut u16;
    }
}

/// `NORMALIZE(x, addr)` — offset → pointer (skip NULL/zero).
#[inline]
unsafe fn normalize(buffer: *mut *mut u16, base: PRTL_USER_PROCESS_PARAMETERS) {
    if !(*buffer).is_null() {
        *buffer = ((*buffer as usize) + (base as usize)) as *mut u16;
    }
}

/// `PRTL_USER_PROCESS_PARAMETERS NTAPI RtlDeNormalizeProcessParams(...)`.
#[no_mangle]
pub unsafe extern "system" fn RtlDeNormalizeProcessParams(
    Params: PRTL_USER_PROCESS_PARAMETERS,
) -> PRTL_USER_PROCESS_PARAMETERS {
    if !Params.is_null()
        && ((*Params).Flags & RTL_USER_PROCESS_PARAMETERS_NORMALIZED) != 0
    {
        denormalize(core::ptr::addr_of_mut!((*Params).CurrentDirectory.DosPath.Buffer), Params);
        denormalize(core::ptr::addr_of_mut!((*Params).DllPath.Buffer), Params);
        denormalize(core::ptr::addr_of_mut!((*Params).ImagePathName.Buffer), Params);
        denormalize(core::ptr::addr_of_mut!((*Params).CommandLine.Buffer), Params);
        denormalize(core::ptr::addr_of_mut!((*Params).WindowTitle.Buffer), Params);
        denormalize(core::ptr::addr_of_mut!((*Params).DesktopInfo.Buffer), Params);
        denormalize(core::ptr::addr_of_mut!((*Params).ShellInfo.Buffer), Params);
        denormalize(core::ptr::addr_of_mut!((*Params).RuntimeData.Buffer), Params);

        (*Params).Flags &= !RTL_USER_PROCESS_PARAMETERS_NORMALIZED;
    }

    Params
}

/// `PRTL_USER_PROCESS_PARAMETERS NTAPI RtlNormalizeProcessParams(...)`.
#[no_mangle]
pub unsafe extern "system" fn RtlNormalizeProcessParams(
    Params: PRTL_USER_PROCESS_PARAMETERS,
) -> PRTL_USER_PROCESS_PARAMETERS {
    if !Params.is_null()
        && ((*Params).Flags & RTL_USER_PROCESS_PARAMETERS_NORMALIZED) == 0
    {
        normalize(core::ptr::addr_of_mut!((*Params).CurrentDirectory.DosPath.Buffer), Params);
        normalize(core::ptr::addr_of_mut!((*Params).DllPath.Buffer), Params);
        normalize(core::ptr::addr_of_mut!((*Params).ImagePathName.Buffer), Params);
        normalize(core::ptr::addr_of_mut!((*Params).CommandLine.Buffer), Params);
        normalize(core::ptr::addr_of_mut!((*Params).WindowTitle.Buffer), Params);
        normalize(core::ptr::addr_of_mut!((*Params).DesktopInfo.Buffer), Params);
        normalize(core::ptr::addr_of_mut!((*Params).ShellInfo.Buffer), Params);
        normalize(core::ptr::addr_of_mut!((*Params).RuntimeData.Buffer), Params);

        (*Params).Flags |= RTL_USER_PROCESS_PARAMETERS_NORMALIZED;
    }

    Params
}
