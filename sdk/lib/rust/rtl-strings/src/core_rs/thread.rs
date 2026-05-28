//! Constants + small FFI struct layouts for `sdk/lib/rtl/thread.c`.
//!
//! Wave THR is translate-only: 8 fns chaining `Zw*`/`Nt*` thread &
//! VM syscalls + `Rtl*` helpers. No algorithmic content to host in
//! `core_rs`; this module just centralises layout-matched structs
//! and the few NTSTATUS / memory-protection / info-class constants
//! the shims need.
//!
//! Faithful to ReactOS `sdk/lib/rtl/thread.c` (Alex Ionescu, Eric
//! Kohl, KJK::Hyperion).

#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

// --- Pseudo-handles (winnt.h).
pub const NT_CURRENT_PROCESS: isize = -1;
pub const NT_CURRENT_THREAD: isize = -2;

// --- NTSTATUS values used here.
pub const STATUS_INVALID_IMAGE_FORMAT: i32 = 0xC000_007Bu32 as i32;
pub const STATUS_NOT_IMPLEMENTED: i32 = 0xC000_0002u32 as i32;
pub const STATUS_UNSUCCESSFUL: i32 = 0xC000_0001u32 as i32;

// --- Memory protection flags (winnt.h).
pub const MEM_COMMIT: u32 = 0x0000_1000;
pub const MEM_RESERVE: u32 = 0x0000_2000;
pub const MEM_RELEASE: u32 = 0x0000_8000;
pub const PAGE_READWRITE: u32 = 0x04;
pub const PAGE_GUARD: u32 = 0x100;

// --- THREAD_ALL_ACCESS for NT 5.x (winnt.h pre-Vista value).
// STANDARD_RIGHTS_REQUIRED (0xF0000) | SYNCHRONIZE (0x100000) | 0x3FF.
pub const THREAD_ALL_ACCESS: u32 = 0x001F_03FF;

// --- THREADINFOCLASS subset (ntpsapi.h).
pub const THREAD_BASIC_INFORMATION_CLASS: u32 = 0;
pub const THREAD_BREAK_ON_TERMINATION: u32 = 14;

// --- SYSTEM_INFORMATION_CLASS subset (ntexapi.h).
pub const SYSTEM_BASIC_INFORMATION_CLASS: u32 = 0;

// --- PEB.NtGlobalFlag bit: enable system critical-section breaks (pstypes.h).
pub const FLG_ENABLE_SYSTEM_CRIT_BREAKS: u32 = 0x0010_0000;

/// `SYSTEM_BASIC_INFORMATION` (ntexapi.h). Layout is portable across
/// the 3 arches — ULONG fields first, then ULONG_PTR triple, then a
/// single UCHAR. `#[repr(C)]` handles the per-arch padding.
#[repr(C)]
pub struct SYSTEM_BASIC_INFORMATION {
    pub Reserved: u32,
    pub TimerResolution: u32,
    pub PageSize: u32,
    pub NumberOfPhysicalPages: u32,
    pub LowestPhysicalPageNumber: u32,
    pub HighestPhysicalPageNumber: u32,
    pub AllocationGranularity: u32,
    pub MinimumUserModeAddress: usize,
    pub MaximumUserModeAddress: usize,
    pub ActiveProcessorsAffinityMask: usize,
    pub NumberOfProcessors: u8,
}

/// `INITIAL_TEB` (ndk/pstypes.h:729). Five PVOID fields, no padding.
#[repr(C)]
pub struct INITIAL_TEB {
    pub PreviousStackBase: *mut u8,
    pub PreviousStackLimit: *mut u8,
    pub StackBase: *mut u8,
    pub StackLimit: *mut u8,
    pub AllocatedStackBase: *mut u8,
}

/// `CLIENT_ID` (ndk/extypes.h) — two HANDLEs.
#[repr(C)]
pub struct CLIENT_ID {
    pub UniqueProcess: *mut u8,
    pub UniqueThread: *mut u8,
}

/// `OBJECT_ATTRIBUTES` (ndk/umtypes.h).
#[repr(C)]
pub struct OBJECT_ATTRIBUTES {
    pub Length: u32,
    pub RootDirectory: *mut u8,
    pub ObjectName: *mut u8,
    pub Attributes: u32,
    pub SecurityDescriptor: *mut u8,
    pub SecurityQualityOfService: *mut u8,
}

/// `THREAD_BASIC_INFORMATION` (ntpsapi.h).
#[repr(C)]
pub struct THREAD_BASIC_INFORMATION {
    pub ExitStatus: i32,
    pub TebBaseAddress: *mut u8,
    pub ClientId: CLIENT_ID,
    pub AffinityMask: usize,
    pub Priority: i32,
    pub BasePriority: i32,
}

/// `CONTEXT` size per arch (winnt.h + per-arch context).
/// i386: `sizeof(CONTEXT)` = 716. amd64: 1232. arm64 (CHPE/Win11
/// flavor): 880; ReactOS uses the standard ARM64 layout = 912.
#[cfg(target_arch = "x86")]
pub const CONTEXT_SIZE: usize = 716;
#[cfg(target_arch = "x86_64")]
pub const CONTEXT_SIZE: usize = 1232;
#[cfg(target_arch = "aarch64")]
pub const CONTEXT_SIZE: usize = 912;

/// `CONTEXT_SIZE` for the host build (rtl-difftest target_os="linux"
/// etc.). Wave THR has no diff harness so this is unreachable, but
/// the struct still needs to compile. Pick the largest of the three.
#[cfg(not(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64")))]
pub const CONTEXT_SIZE: usize = 1232;

/// `ROUND_UP(n, multiple)` from ntdef.h — bytes / pages alignment.
/// `multiple` is a power of two.
#[inline]
pub const fn round_up_pow2(n: usize, multiple: usize) -> usize {
    (n + multiple - 1) & !(multiple - 1)
}
