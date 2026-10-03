//! Constants + FFI struct layouts for `sdk/lib/rtl/process.c`.
//!
//! Wave PRC is translate-only: 9 fns of `Zw*`/`Nt*` syscall + `Rtl*`
//! helper plumbing (process create + small encode/decode + critical
//! flag + os-version reads). No algorithmic content to host in
//! `core_rs`. Faithful to ReactOS `sdk/lib/rtl/process.c`.

#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use super::thread as thr;

// --- NTSTATUS values used here.
pub use super::thread::{STATUS_NOT_IMPLEMENTED, STATUS_UNSUCCESSFUL};

// --- Memory-protection + section + share + handle flags.
pub use super::thread::{MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_READWRITE};

pub const PAGE_EXECUTE: u32 = 0x10;
pub const SEC_IMAGE: u32 = 0x0100_0000;
pub const SECTION_ALL_ACCESS: u32 = 0x000F_001F;

// --- File / object access masks.
pub const SYNCHRONIZE: u32 = 0x0010_0000;
pub const FILE_READ_DATA: u32 = 0x0001;
pub const FILE_EXECUTE: u32 = 0x0020;
pub const FILE_SHARE_READ: u32 = 0x0001;
pub const FILE_SHARE_DELETE: u32 = 0x0004;
pub const FILE_SYNCHRONOUS_IO_NONALERT: u32 = 0x0000_0020;
pub const FILE_NON_DIRECTORY_FILE: u32 = 0x0000_0040;
pub const OBJ_INHERIT: u32 = 0x0000_0002;
pub const OBJ_CASE_INSENSITIVE: u32 = 0x0000_0040;

// --- PROCESS_ALL_ACCESS — per memory `process_all_access_constant`:
// kernel ntoskrnl-rust uses NT 5.x value (Vista+ extended is 0x001F_FFFF).
pub const PROCESS_ALL_ACCESS: u32 = 0x001F_0FFF;

// --- Duplicate options (winnt.h).
pub const DUPLICATE_SAME_ACCESS: u32 = 0x0000_0002;
pub const DUPLICATE_SAME_ATTRIBUTES: u32 = 0x0000_0004;

// --- PEB.NtGlobalFlag bits (pstypes.h).
pub use super::thread::FLG_ENABLE_SYSTEM_CRIT_BREAKS;
pub const FLG_ENABLE_CSRDEBUG: u32 = 0x0002_0000;

// --- Process / Section info classes.
pub const PROCESS_BASIC_INFORMATION_CLASS: u32 = 0;
pub const PROCESS_BREAK_ON_TERMINATION_CLASS: u32 = 29;
pub const PROCESS_COOKIE_CLASS: u32 = 36;
pub const SECTION_IMAGE_INFORMATION_CLASS: u32 = 1;

// --- RTL_USER_PROCESS_PARAMETERS.Flags bit (rtltypes.h).
// ndk/rtltypes.h:46 = 0x20 (0x40 is RESERVE_16MB).
pub const RTL_USER_PROCESS_PARAMETERS_RESERVE_1MB: u32 = 0x0000_0020;

// --- `_WIN32_WINNT_WS03` baseline used by RtlRosGetAppcompatVersion default.
pub const WIN32_WINNT_WS03: u32 = 0x0502;

// --- KUSER_SHARED_DATA.Cookie offset (ndk/ketypes.h:686). Same on
// all 3 arches (0x330) — RtlEncodeSystemPointer reads it via the
// fixed-VA `KI_USER_SHARED_DATA` mapping.
pub const KUSER_SHARED_COOKIE_OFFSET: usize = 0x330;

#[cfg(target_arch = "x86")]
pub const KI_USER_SHARED_DATA_VA: usize = 0xFFDF0000;
#[cfg(target_arch = "x86_64")]
pub const KI_USER_SHARED_DATA_VA: usize = 0xFFFFF780_00000000;
#[cfg(target_arch = "aarch64")]
pub const KI_USER_SHARED_DATA_VA: usize = 0xFFFFF780_00000000;
#[cfg(not(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64")))]
pub const KI_USER_SHARED_DATA_VA: usize = 0;

/// `MM_SHARED_USER_DATA_VA` — the read-only user-mode mapping of
/// KUSER_SHARED_DATA. Fixed at 0x7FFE0000 on every arch.
pub const USER_SHARED_DATA_VA: usize = 0x7FFE0000;

/// True when executing in kernel mode. rtl-strings is built ONCE (with
/// the `kernel` cargo feature on, via the rtl-rust umbrella) and the
/// resulting librtl.a is linked into BOTH ntoskrnl (kernel) AND ntdll
/// (user), so a compile-time `cfg` cannot tell the two apart. Kernel
/// virtual addresses have the high bit set on all three targets (i386
/// >= 0x80000000; amd64/arm64 canonical kernel addresses), so the sign
/// bit of a stack-probe address distinguishes the mode without a
/// syscall.
#[inline(always)]
pub fn in_kernel_mode() -> bool {
    let probe: u8 = 0;
    (core::ptr::addr_of!(probe) as isize) < 0
}

/// VA of the KUSER_SHARED_DATA page valid in the *current* execution
/// mode. Reading the kernel VA from user mode faults — that fault is
/// uncatchable under DUMMY_PSEH and manifested as a silent boot hang at
/// the first user-mode `RtlEncodeSystemPointer` (winlogon creating
/// services.exe → apphelp `BaseInitApphelp`). The cookie value itself
/// is system-wide identical through either mapping.
#[inline(always)]
pub fn kuser_shared_data_va() -> usize {
    if in_kernel_mode() {
        KI_USER_SHARED_DATA_VA
    } else {
        USER_SHARED_DATA_VA
    }
}

// --- struct layouts -----------------------------------------------------

/// `IO_STATUS_BLOCK` (ntddk.h). The union is one pointer-sized slot
/// (`Status`/`Pointer`) followed by ULONG_PTR `Information`.
#[repr(C)]
pub struct IO_STATUS_BLOCK {
    pub StatusOrPointer: usize,
    pub Information: usize,
}

/// `PROCESS_BASIC_INFORMATION` (ntpsapi.h).
#[repr(C)]
pub struct PROCESS_BASIC_INFORMATION {
    pub ExitStatus: i32,
    pub PebBaseAddress: *mut u8,
    pub AffinityMask: usize,
    pub BasePriority: i32,
    pub UniqueProcessId: usize,
    pub InheritedFromUniqueProcessId: usize,
}

/// `SECTION_IMAGE_INFORMATION` (ntddk.h). The fields after
/// `CommittedStackSize` are not accessed by Wave PRC, but we
/// keep the layout so the struct passes through `ZwQuerySection`
/// at the correct size.
#[repr(C)]
pub struct SECTION_IMAGE_INFORMATION {
    pub TransferAddress: *mut u8,
    pub ZeroBits: u32,
    pub MaximumStackSize: usize,
    pub CommittedStackSize: usize,
    pub SubSystemType: u32,
    pub SubSystemMinorVersion: u16,
    pub SubSystemMajorVersion: u16,
    pub GpValue: u32,
    pub ImageCharacteristics: u16,
    pub DllCharacteristics: u16,
    pub Machine: u16,
    pub ImageContainsCode: u8,
    pub ImageFlags: u8,
    pub LoaderFlags: u32,
    pub ImageFileSize: u32,
    pub CheckSum: u32,
}

/// `UNICODE_STRING` layout-match (shared with `cabi::mod.rs` but
/// duplicated here so `core_rs::process` is self-contained).
#[repr(C)]
pub struct UNICODE_STRING {
    pub Length: u16,
    pub MaximumLength: u16,
    pub Buffer: *mut u16,
}

/// `STRING` (counted ANSI string).
#[repr(C)]
pub struct STRING {
    pub Length: u16,
    pub MaximumLength: u16,
    pub Buffer: *mut u8,
}

/// `CURDIR` (rtltypes.h).
#[repr(C)]
pub struct CURDIR {
    pub DosPath: UNICODE_STRING,
    pub Handle: *mut u8,
}

/// `RTL_DRIVE_LETTER_CURDIR` (rtltypes.h).
#[repr(C)]
pub struct RTL_DRIVE_LETTER_CURDIR {
    pub Flags: u16,
    pub Length: u16,
    pub TimeStamp: u32,
    pub DosPath: STRING,
}

pub const RTL_MAX_DRIVE_LETTERS: usize = 32;

/// `RTL_USER_PROCESS_PARAMETERS` (rtltypes.h:1551, NTDDI_WS03 baseline).
#[repr(C)]
pub struct RTL_USER_PROCESS_PARAMETERS {
    pub MaximumLength: u32,
    pub Length: u32,
    pub Flags: u32,
    pub DebugFlags: u32,
    pub ConsoleHandle: *mut u8,
    pub ConsoleFlags: u32,
    pub StandardInput: *mut u8,
    pub StandardOutput: *mut u8,
    pub StandardError: *mut u8,
    pub CurrentDirectory: CURDIR,
    pub DllPath: UNICODE_STRING,
    pub ImagePathName: UNICODE_STRING,
    pub CommandLine: UNICODE_STRING,
    pub Environment: *mut u16,
    pub StartingX: u32,
    pub StartingY: u32,
    pub CountX: u32,
    pub CountY: u32,
    pub CountCharsX: u32,
    pub CountCharsY: u32,
    pub FillAttribute: u32,
    pub WindowFlags: u32,
    pub ShowWindowFlags: u32,
    pub WindowTitle: UNICODE_STRING,
    pub DesktopInfo: UNICODE_STRING,
    pub ShellInfo: UNICODE_STRING,
    pub RuntimeData: UNICODE_STRING,
    pub CurrentDirectories: [RTL_DRIVE_LETTER_CURDIR; RTL_MAX_DRIVE_LETTERS],
}

/// `CLIENT_ID` (re-exported from `thread`).
pub use super::thread::CLIENT_ID;

/// `RTL_USER_PROCESS_INFORMATION` (rtltypes.h:1589).
#[repr(C)]
pub struct RTL_USER_PROCESS_INFORMATION {
    pub Size: u32,
    pub ProcessHandle: *mut u8,
    pub ThreadHandle: *mut u8,
    pub ClientId: CLIENT_ID,
    pub ImageInformation: SECTION_IMAGE_INFORMATION,
}

// Suppress the unused-import warning for `thr` — its `pub use`s are
// what we want, but the bare alias is unused.
#[doc(hidden)]
const _USE_THR: usize = core::mem::size_of::<thr::INITIAL_TEB>();

// --- pure decision kernels --------------------------------------------------

/// `RtlEncodePointer` / `RtlDecodePointer` body — XOR with a cookie.
/// `Encode` and `Decode` are the same op (XOR is self-inverse).
/// Faithful to lines 393-422 of `sdk/lib/rtl/process.c`.
#[inline]
pub fn xor_pointer(pointer: usize, cookie: u32) -> usize {
    pointer ^ (cookie as usize)
}

/// `RtlRosGetAppcompatVersion` decision: `(major << 8) | minor`,
/// or the Server-2003 sentinel `0x0502` when both are zero (which
/// the C original interprets as "no PEB → fallback").
/// Faithful to lines 497-512 of `sdk/lib/rtl/process.c`.
#[inline]
pub fn appcompat_version(os_major: u32, os_minor: u32) -> u32 {
    if os_major == 0 && os_minor == 0 {
        WIN32_WINNT_WS03
    } else {
        (os_major << 8) | os_minor
    }
}

/// Compute the total `WCHAR` count of a double-NUL-terminated
/// environment block, INCLUDING the terminating double-NUL.
///
/// Faithful to the C `do { walk += wcslen+1 } while (*walk); walk++;`
/// loop in `RtlpInitEnvironment` (sdk/lib/rtl/process.c:104). Treats
/// an empty block (first WCHAR is 0) as a single terminator NUL.
///
/// Pure (slice in, count out) — the cabi shim does the `*const u16`
/// → `&[u16]` walk by finding the last `0\0` pair.
pub fn env_block_chars(env: &[u16]) -> usize {
    if env.is_empty() || env[0] == 0 {
        return 1;
    }
    let mut i = 0usize;
    loop {
        // Walk to the end of the current entry's NUL.
        while i < env.len() && env[i] != 0 {
            i += 1;
        }
        i += 1; // consume the NUL
        // If the next char is NUL, block ends here.
        if i >= env.len() || env[i] == 0 {
            return i + 1; // include the second NUL
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // xor_pointer is its own inverse.
    #[test]
    fn xor_pointer_self_inverse() {
        let ptr: usize = 0xDEADBEEF;
        let cookie: u32 = 0x12345678;
        let enc = xor_pointer(ptr, cookie);
        assert_ne!(enc, ptr);
        assert_eq!(xor_pointer(enc, cookie), ptr);
    }
    #[test]
    fn xor_pointer_zero_cookie_is_identity() {
        assert_eq!(xor_pointer(0xCAFE, 0), 0xCAFE);
    }
    #[test]
    fn xor_pointer_null_pointer() {
        assert_eq!(xor_pointer(0, 0x42), 0x42);
    }

    // appcompat_version — the WS03 fallback path.
    #[test]
    fn appcompat_version_ws03_fallback() {
        assert_eq!(appcompat_version(0, 0), 0x0502);
    }
    #[test]
    fn appcompat_version_normal() {
        // Server 2003 actual = 5.2 = 0x0502
        assert_eq!(appcompat_version(5, 2), 0x0502);
        // Vista = 6.0
        assert_eq!(appcompat_version(6, 0), 0x0600);
        // Win10 = 10.0
        assert_eq!(appcompat_version(10, 0), 0x0A00);
    }
    #[test]
    fn appcompat_version_minor_only() {
        // major=0, minor=42 → NOT zero on aggregate, so plain shift.
        assert_eq!(appcompat_version(0, 42), 42);
    }

    // env_block_chars — terminator-only block.
    #[test]
    fn env_block_chars_empty() {
        let env = [0u16];
        assert_eq!(env_block_chars(&env), 1);
    }
    #[test]
    fn env_block_chars_single_entry() {
        // "PATH=C:\\\0\0" — 8 chars + double-NUL.
        let env: Vec<u16> = "PATH=C:\\".encode_utf16().chain([0, 0]).collect();
        assert_eq!(env_block_chars(&env), 10);
    }
    #[test]
    fn env_block_chars_two_entries() {
        // "A=1\0B=2\0\0" — 4 + 4 + 1 = 9
        let env: Vec<u16> = "A=1\0B=2".encode_utf16().chain([0, 0]).collect();
        assert_eq!(env_block_chars(&env), 9);
    }
    #[test]
    fn env_block_chars_three_entries_unicode() {
        // The walk should be independent of content; test ASCII triplet.
        let env: Vec<u16> = "X=1\0YY=22\0ZZZ=333"
            .encode_utf16()
            .chain([0, 0])
            .collect();
        assert_eq!(env_block_chars(&env), 4 + 6 + 8 + 1); // = 19
    }
}
