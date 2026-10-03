//! winsrv-sys — FFI declarations for winsrv.dll's USER + Console
//! server stack (consrv + usersrv + concfg static libs aggregated
//! into winsrv.dll, plus the top-level init.c).
//!
//! Auto-generated via `bindgen` for all 3 architectures from a wrapper
//! that pulls `concfg/precomp.h`, `winsrv.h`, `consrv/consrv.h` and
//! `usersrv/usersrv.h`. The user-mode analog of `ntkrnl-sys`/`win32k-sys`
//! for the Phase 5 winsrv translation (see memory
//! `project_phase5_winsrv_audit`).
//!
//! winsrv-sys is a SUPERSET of `ntdll-sys`'s surface: the win-server
//! stack needs windowing/console/imm types that ntdll-sys deliberately
//! stops short of. To avoid the duplicate-extern noise that would
//! follow from re-bindgen'ing the NDK twice, this crate re-exports
//! `ntdll_sys::nt` and the winsrv-specific bindings sit ALONGSIDE
//! it under `pub mod winsrv`. The translator picks the right umbrella
//! per call site: `nt::*` for Nt/Zw/Rtl/Csr/Sm/Base, `winsrv::*` for
//! consrv/usersrv/concfg / wincon/winuser/wingdi.
//!
//! NOT a standalone static library — an `rlib` declaration dependency.

#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(clippy::upper_case_acronyms)]
#![allow(dead_code)]
// Bindgen re-declares NT/CSR types pulled via winsrv.h's NDK + csrsrv
// includes. Within this crate those are self-consistent; the
// type-identity split vs `ntdll_sys::nt` is handled by casts at the
// boundary (same pattern as `ntkrnl_sys::ddk` vs `win32k_sys::win32k`).
#![allow(clashing_extern_declarations)]

/// `w!("…")` — wide string literal, produces a `&'static [u16; N+1]`
/// with NUL terminator. Mirrors the C `L"..."` literal that winsrv
/// code uses heavily. Marked `#[macro_export]` so it's reachable as
/// `winsrv_sys::w!` from any winsrv-rust crate.
///
/// Returns an array reference; callers wanting a pointer use
/// `w!("…").as_ptr()`.
#[macro_export]
macro_rules! w {
    ($s:literal) => {{
        // Pre-evaluated at macro expansion: const fn builds a
        // [u16; N+1] from the &str literal, no heap, no formatting.
        const __W_BYTES: &str = $s;
        const __W_LEN: usize = {
            // Count UTF-16 code units in the source &str. ASCII-only
            // strings (the vast majority of winsrv literals) round-trip
            // 1:1, but we handle UTF-8 properly via char iteration.
            let bytes = __W_BYTES.as_bytes();
            let mut len = 0usize;
            let mut i = 0;
            while i < bytes.len() {
                let b = bytes[i];
                if b < 0x80 {
                    len += 1;
                    i += 1;
                } else if b < 0xC0 {
                    // continuation byte — should never start a sequence
                    i += 1;
                } else if b < 0xE0 {
                    len += 1;
                    i += 2;
                } else if b < 0xF0 {
                    len += 1;
                    i += 3;
                } else {
                    // BMP overflow — surrogate pair
                    len += 2;
                    i += 4;
                }
            }
            len
        };
        const __W: [u16; __W_LEN + 1] = {
            let bytes = __W_BYTES.as_bytes();
            let mut out = [0u16; __W_LEN + 1];
            let mut i = 0;
            let mut j = 0;
            while i < bytes.len() {
                let b = bytes[i];
                if b < 0x80 {
                    out[j] = b as u16;
                    j += 1;
                    i += 1;
                } else if b < 0xE0 {
                    let c = ((b as u32 & 0x1F) << 6)
                        | (bytes[i + 1] as u32 & 0x3F);
                    out[j] = c as u16;
                    j += 1;
                    i += 2;
                } else if b < 0xF0 {
                    let c = ((b as u32 & 0x0F) << 12)
                        | ((bytes[i + 1] as u32 & 0x3F) << 6)
                        | (bytes[i + 2] as u32 & 0x3F);
                    out[j] = c as u16;
                    j += 1;
                    i += 3;
                } else {
                    let c = ((b as u32 & 0x07) << 18)
                        | ((bytes[i + 1] as u32 & 0x3F) << 12)
                        | ((bytes[i + 2] as u32 & 0x3F) << 6)
                        | (bytes[i + 3] as u32 & 0x3F);
                    let c2 = c - 0x10000;
                    out[j] = (0xD800 | (c2 >> 10)) as u16;
                    out[j + 1] = (0xDC00 | (c2 & 0x3FF)) as u16;
                    j += 2;
                    i += 4;
                }
            }
            // out[__W_LEN] stays 0 (NUL terminator).
            out
        };
        &__W
    }};
}

/// `RTL_CONSTANT_STRING(s)` — produce a `UNICODE_STRING` aimed at a
/// pre-NUL-terminated wide string literal. Maps the C macro:
/// `{ sizeof(s) - sizeof(WCHAR), sizeof(s), s }`. Callers typically
/// pass `w!("…").as_ptr() as *mut u16` or another `*mut u16`.
///
/// Implemented as a non-`const` inline fn so the produced
/// UNICODE_STRING is usable at runtime (`let s = RTL_CONSTANT_STRING(...)`).
#[macro_export]
macro_rules! RTL_CONSTANT_STRING {
    ($s:expr) => {{
        let p: *const u16 = $s;
        let mut n: u16 = 0;
        // SAFETY: caller passes a NUL-terminated wide pointer (w!()
        // literal or similar). Walking until 0 is the standard way.
        unsafe {
            while *p.add(n as usize) != 0 {
                n += 1;
            }
        }
        let bytes = n * (core::mem::size_of::<u16>() as u16);
        $crate::winsrv::_UNICODE_STRING {
            Length: bytes,
            MaximumLength: bytes + (core::mem::size_of::<u16>() as u16),
            Buffer: p as *mut u16,
        }
    }};
}

mod bindgen_common;

#[cfg(target_arch = "aarch64")]
mod bindgen_arm64;

#[cfg(target_arch = "x86_64")]
mod bindgen_amd64;

#[cfg(target_arch = "x86")]
mod bindgen_i386;

/// Re-export ONLY the hand-written user-mode helpers from ntdll-sys
/// (FORCEINLINE accessors + STATUS_* constants). NOT a glob of
/// `ntdll_sys::nt::*` — that would re-publish HKEY/HANDLE/LPWSTR/
/// MAX_PATH etc. that the winsrv-sys bindgen already defines (via
/// the windef.h chain pulled by winsrv.h/consrv.h/usersrv.h/
/// concfg/precomp.h), making any `use winsrv_sys::nt::*` glob
/// E0659-ambiguous against the winsrv-side definitions. With this
/// shape winsrv-rust can rely on a SINGLE glob: `use
/// winsrv_sys::winsrv::*` — every NT/CSR/SM/BASE fn from the
/// bindgen below resolves through the same module, and the
/// hand-written inlines are namespaced under `winsrv_sys::nt`.
pub mod nt {
    pub use ntdll_sys::nt::{
        NtCurrentPeb, NtCurrentProcess, NtCurrentTeb, NtCurrentThread,
        ZwCurrentProcess, ZwCurrentThread,
    };

    // LIST_ENTRY helpers are winsrv-flavored — the bindgen-generated
    // `winsrv_sys::winsrv::_LIST_ENTRY` is layout-identical to
    // `ntdll_sys::nt::_LIST_ENTRY`, but the type identities are
    // distinct (windef.h pulls them through different chains for each
    // *-sys crate). Defining the helpers here keeps the type
    // identities consistent across all callers in the
    // `use winsrv_sys::winsrv::*` one-glob world.
    use crate::winsrv::{LIST_ENTRY, PLIST_ENTRY};

    #[inline]
    pub unsafe fn InitializeListHead(head: *mut LIST_ENTRY) {
        (*head).Flink = head;
        (*head).Blink = head;
    }

    #[inline]
    pub unsafe fn InsertHeadList(head: *mut LIST_ENTRY, entry: *mut LIST_ENTRY) {
        let flink = (*head).Flink;
        (*entry).Flink = flink;
        (*entry).Blink = head;
        (*flink).Blink = entry;
        (*head).Flink = entry;
    }

    #[inline]
    pub unsafe fn InsertTailList(head: *mut LIST_ENTRY, entry: *mut LIST_ENTRY) {
        let blink = (*head).Blink;
        (*entry).Flink = head;
        (*entry).Blink = blink;
        (*blink).Flink = entry;
        (*head).Blink = entry;
    }

    #[inline]
    pub unsafe fn RemoveEntryList(entry: *mut LIST_ENTRY) -> u8 {
        let flink = (*entry).Flink;
        let blink = (*entry).Blink;
        (*blink).Flink = flink;
        (*flink).Blink = blink;
        if flink == blink { 1 } else { 0 }
    }

    #[inline]
    pub unsafe fn RemoveHeadList(head: *mut LIST_ENTRY) -> PLIST_ENTRY {
        let entry = (*head).Flink;
        let flink = (*entry).Flink;
        (*head).Flink = flink;
        (*flink).Blink = head;
        entry
    }

    #[inline]
    pub unsafe fn RemoveTailList(head: *mut LIST_ENTRY) -> PLIST_ENTRY {
        let entry = (*head).Blink;
        let blink = (*entry).Blink;
        (*head).Blink = blink;
        (*blink).Flink = head;
        entry
    }
    // STATUS_* hand-written codes (ntstatus.h cast-macros are not
    // bindgen-evaluable, so they live in ntdll-sys::constants).
    // Re-export ALL of them — they're pure `pub const … = i32 = …;`
    // entries, no collision risk with the winsrv bindgen.
    pub use ntdll_sys::nt::{
        STATUS_SUCCESS,
        STATUS_UNSUCCESSFUL, STATUS_INVALID_HANDLE, STATUS_INVALID_PARAMETER,
        STATUS_ACCESS_DENIED, STATUS_OBJECT_TYPE_MISMATCH,
        STATUS_OBJECT_NAME_COLLISION, STATUS_QUOTA_EXCEEDED,
        STATUS_PROCESS_IS_TERMINATING, STATUS_HANDLE_NOT_CLOSABLE,
        STATUS_OBJECT_NAME_EXISTS, STATUS_OBJECT_NAME_NOT_FOUND,
        STATUS_PORT_ALREADY_SET, STATUS_PORT_DISCONNECTED,
        STATUS_REPLY_MESSAGE_MISMATCH, STATUS_INVALID_PORT_HANDLE,
        STATUS_PORT_MESSAGE_TOO_LONG, STATUS_SERVER_SID_MISMATCH,
        STATUS_PORT_CONNECTION_REFUSED, STATUS_PORT_NOT_SET,
        STATUS_INVALID_PARAMETER_3, STATUS_INVALID_PARAMETER_4,
        STATUS_LPC_REPLY_LOST, STATUS_OBJECT_NAME_INVALID,
        STATUS_OBJECT_PATH_NOT_FOUND, STATUS_OBJECT_PATH_INVALID,
        STATUS_OBJECT_PATH_SYNTAX_BAD, STATUS_INFO_LENGTH_MISMATCH,
        STATUS_BUFFER_TOO_SMALL, STATUS_BUFFER_OVERFLOW,
        STATUS_NOT_IMPLEMENTED, STATUS_NO_MEMORY,
        STATUS_LOGON_SESSION_EXISTS, STATUS_NOT_LOGON_PROCESS,
        STATUS_INSUFFICIENT_RESOURCES, STATUS_NO_TOKEN,
        STATUS_TIMEOUT, STATUS_INVALID_PARAMETER_MIX,
        STATUS_PENDING, STATUS_BAD_IMPERSONATION_LEVEL,
    };

    // STATUS_* codes that aren't in ntdll-sys::constants — hand-written here.
    // ntstatus.h:215  STATUS_LOG_HARD_ERROR     0x4000001A
    pub const STATUS_LOG_HARD_ERROR: u32 = 0x4000_001A;
    // ntstatus.h     STATUS_CANT_WAIT          0xC00000D8
    pub const STATUS_CANT_WAIT: i32 = 0xC00000D8_u32 as i32;
    // ntstatus.h     STATUS_RETRY              0xC000022D
    pub const STATUS_RETRY: i32 = 0xC000022D_u32 as i32;

    // Inline FORCEINLINE macro/accessor wrappers — bindgen can't see
    // through #define chains.

    /// `RtlGetProcessHeap()` — ndk/rtlfuncs.h:1127:
    /// `(NtCurrentPeb()->ProcessHeap)`. NTAPI macro; live in nt:: so
    /// translated code uses `RtlGetProcessHeap()` via the same glob.
    #[inline]
    pub unsafe fn RtlGetProcessHeap() -> *mut core::ffi::c_void {
        let peb = ntdll_sys::nt::NtCurrentPeb();
        (*peb).ProcessHeap as *mut _
    }

    /// `CsrGetClientThread()` — csr/csrsrv.h:77:
    /// `((PCSR_THREAD)(NtCurrentTeb()->CsrClientThread))`. The result
    /// is treated as `*mut crate::winsrv::CSR_THREAD` at call sites
    /// (Rust handles the cast through the return type).
    #[inline]
    pub unsafe fn CsrGetClientThread() -> *mut crate::winsrv::CSR_THREAD {
        let teb = ntdll_sys::nt::NtCurrentTeb();
        (*teb).CsrClientThread as *mut crate::winsrv::CSR_THREAD
    }

    /// `UlongToHandle(ul)` — basetsd.h:91 is `ULongToHandle(ul)` which is
    /// `((HANDLE)(ULONG_PTR)(ul))`. On i386 ULONG_PTR is u32 = ptr size
    /// so the cast widens through usize. Returns a HANDLE (raw pointer).
    #[inline]
    pub fn UlongToHandle(ul: u32) -> *mut core::ffi::c_void {
        ul as usize as *mut _
    }

    /// `ULongToHandle(ul)` — same as `UlongToHandle`. basetsd.h spells
    /// it with capital `L`; provide both spellings.
    #[inline]
    pub fn ULongToHandle(ul: u32) -> *mut core::ffi::c_void {
        ul as usize as *mut _
    }

    /// `HandleToULong(h)` — basetsd.h cast macro: truncate the pointer
    /// back to u32. On i386 it's a straight cast; on amd64/arm64 the
    /// truncation is lossy by design (the caller knows the upper bits
    /// were ULONG-zero).
    #[inline]
    pub fn HandleToULong(h: *mut core::ffi::c_void) -> u32 {
        h as usize as u32
    }

    /// `HandleToUlong(h)` — alternative spelling.
    #[inline]
    pub fn HandleToUlong(h: *mut core::ffi::c_void) -> u32 {
        h as usize as u32
    }

    /// `UlongToPtr(ul)` — basetsd.h cast macro: same shape as
    /// `UlongToHandle` but returns `*mut c_void` directly.
    #[inline]
    pub fn UlongToPtr(ul: u32) -> *mut core::ffi::c_void {
        ul as usize as *mut _
    }

    /// `INVALID_HANDLE_VALUE` — winbase.h: `((HANDLE)(LONG_PTR)-1)`.
    /// Bindgen treats this as an opaque pointer; hand-define for
    /// translated consumers.
    pub const INVALID_HANDLE_VALUE: *mut core::ffi::c_void = !0usize as *mut _;

    /// `GetWindowLongPtrW(hWnd, nIndex)` — on i386 winuser.h:4931
    /// `#define GetWindowLongPtrW GetWindowLongW` (no real Ptr variant);
    /// bindgen doesn't follow the `#define` so we hand-write the
    /// alias here. On amd64/arm64 the real fn IS bindgen-derivable —
    /// use that directly; gate the shim to i386 only to avoid an
    /// E0659 ambiguity with the bindgen-declared one.
    #[cfg(target_arch = "x86")]
    #[inline]
    pub unsafe fn GetWindowLongPtrW(
        hWnd: crate::winsrv::HWND,
        nIndex: i32,
    ) -> i32 {
        crate::winsrv::GetWindowLongW(hWnd, nIndex)
    }

    /// `SetWindowLongPtrW(hWnd, nIndex, dwNewLong)` — i386 macro
    /// alias for `SetWindowLongW`. Same shape as `GetWindowLongPtrW`:
    /// hand-written shim ON i386 only; on amd64/arm64 the bindgen
    /// version is the canonical one.
    #[cfg(target_arch = "x86")]
    #[inline]
    pub unsafe fn SetWindowLongPtrW(
        hWnd: crate::winsrv::HWND,
        nIndex: i32,
        dwNewLong: i32,
    ) -> i32 {
        crate::winsrv::SetWindowLongW(hWnd, nIndex, dwNewLong)
    }
}

/// Hand-written constants for winsrv that aren't bindgen-derivable
/// (cast-macros in ntstatus.h, #defines in headers bindgen doesn't
/// follow, etc.).
pub mod consts {
    /// `USER_VERSION = MAKELONG(0x0000, 0x0005)` — passed to
    /// `NtUserInitialize`. ntuser.h:1149.
    pub const USER_VERSION: u32 = 0x0005_0000;

    /// `DWLP_USER` — winuser.h GetWindowLongPtr offset for user
    /// data. The DWLP_USER value depends on pointer size:
    /// DLGWINDOWEXTRA + sizeof(LRESULT) + sizeof(DLGPROC).
    /// i386: 8 + 4 + 4 = 16 → DLGWINDOWEXTRA(30) + 4 + 4 = 38.
    /// Actually: `#define DWLP_MSGRESULT 0` `#define DWLP_DLGPROC
    /// (DWLP_MSGRESULT + sizeof(LRESULT))` `#define DWLP_USER
    /// (DWLP_DLGPROC + sizeof(DLGPROC))`. sizeof(LRESULT) =
    /// sizeof(DLGPROC) = ptr size, so on i386 = 0 + 4 + 4 = 8.
    /// On amd64/arm64 = 0 + 8 + 8 = 16.
    #[cfg(target_pointer_width = "32")]
    pub const DWLP_USER: i32 = 8;
    #[cfg(target_pointer_width = "64")]
    pub const DWLP_USER: i32 = 16;

    /// `PBM_SETPOS` — `WM_USER + 2` from commctrl.h. Progress bar
    /// position-set message. commctrl.h isn't pulled by winsrv's
    /// internal-header chain so the const is hand-written here.
    pub const PBM_SETPOS: u32 = 0x0402;

    /// `PBM_SETRANGE32` — `WM_USER + 6` from commctrl.h.
    pub const PBM_SETRANGE32: u32 = 0x0406;

    /// `MUTANT_ALL_ACCESS` — STANDARD_RIGHTS_REQUIRED (0x000F0000) |
    /// SYNCHRONIZE (0x00100000) | MUTANT_QUERY_STATE (0x1) = 0x001F0001.
    /// extypes.h:110 / winnt_old.h:393.
    pub const MUTANT_ALL_ACCESS: u32 = 0x001F_0001;

    /// `STATUS_SERVICE_NOTIFICATION` — used by usersrv harderror.c
    /// to detect service-notification dialog mode (no host process).
    /// ntstatus.h: 0x40000018.
    pub const STATUS_SERVICE_NOTIFICATION: i32 = 0x40000018u32 as i32;

    /// `STATUS_NOT_SUPPORTED` — ntstatus.h: 0xC00000BB. Re-exported
    /// here so condrv/text.rs (and friends) can pull it via the
    /// `winsrv_sys::consts::*` glob.
    pub const STATUS_NOT_SUPPORTED: i32 = 0xC00000BB_u32 as i32;

    /// `STATUS_THREAD_IS_TERMINATING` — ntstatus.h: 0xC000004B.
    pub const STATUS_THREAD_IS_TERMINATING: i32 = 0xC000004B_u32 as i32;

    /// `STATUS_ALERTED` — ntstatus.h: 0x00000101 (success-class).
    pub const STATUS_ALERTED: i32 = 0x00000101;

    /// `STATUS_USER_APC` — ntstatus.h: 0x000000C0.
    pub const STATUS_USER_APC: i32 = 0x000000C0;

    /// `SUCCEEDED(hr)` — winerror.h macro: `(((HRESULT)(hr)) >= 0)`.
    /// HRESULT is i32 in winsrv-sys::winsrv.
    #[inline]
    pub fn SUCCEEDED(hr: i32) -> bool {
        hr >= 0
    }

    /// `FAILED(hr)` — winerror.h macro.
    #[inline]
    pub fn FAILED(hr: i32) -> bool {
        hr < 0
    }

    /// `RtlZeroMemory(dest, len)` — Windows macro that aliases memset(dest,0,len).
    #[inline]
    pub unsafe fn RtlZeroMemory(
        dest: *mut core::ffi::c_void,
        len: usize,
    ) {
        core::ptr::write_bytes(dest as *mut u8, 0, len);
    }

    /// `EnterCriticalSection(cs)` — kernel32.dll API. Bindgen sees only
    /// `DeleteCriticalSection`; Enter/Leave/Initialize aren't in our
    /// allowlist (kernel32 not strictly needed at translation time).
    /// Delegate to the Rtl* form which bindgen DOES expose.
    #[inline]
    pub unsafe fn EnterCriticalSection(cs: crate::winsrv::PRTL_CRITICAL_SECTION) {
        let _ = crate::winsrv::RtlEnterCriticalSection(cs);
    }

    /// `LeaveCriticalSection(cs)`.
    #[inline]
    pub unsafe fn LeaveCriticalSection(cs: crate::winsrv::PRTL_CRITICAL_SECTION) {
        let _ = crate::winsrv::RtlLeaveCriticalSection(cs);
    }

    /// `InitializeCriticalSection(cs)`.
    #[inline]
    pub unsafe fn InitializeCriticalSection(cs: crate::winsrv::PRTL_CRITICAL_SECTION) {
        let _ = crate::winsrv::RtlInitializeCriticalSection(cs);
    }

    /// `SE_LOAD_DRIVER_PRIVILEGE` — winnt.h: 10. Privilege LUID
    /// low part.
    pub const SE_LOAD_DRIVER_PRIVILEGE: u32 = 10;

    /// `SE_SHUTDOWN_PRIVILEGE` — winnt.h: 19.
    pub const SE_SHUTDOWN_PRIVILEGE: u32 = 19;

    /// `min(a, b)` — windef.h macro. Generic since C usage spans
    /// every integer/short/long width.
    #[inline]
    pub fn min<T: PartialOrd>(a: T, b: T) -> T {
        if a < b { a } else { b }
    }

    /// `max(a, b)` — windef.h macro. Generic for the same reason.
    #[inline]
    pub fn max<T: PartialOrd>(a: T, b: T) -> T {
        if a > b { a } else { b }
    }

    /// `RtlMoveMemory(dest, src, len)` — Windows macro that aliases
    /// memmove. Overlap-safe (uses `core::ptr::copy`).
    #[inline]
    pub unsafe fn RtlMoveMemory(
        dest: *mut core::ffi::c_void,
        src: *const core::ffi::c_void,
        len: usize,
    ) {
        core::ptr::copy(src as *const u8, dest as *mut u8, len);
    }

    /// `RtlFillMemory(dest, len, val)` — Windows macro for memset.
    #[inline]
    pub unsafe fn RtlFillMemory(
        dest: *mut core::ffi::c_void,
        len: usize,
        val: u8,
    ) {
        core::ptr::write_bytes(dest as *mut u8, val, len);
    }

    // conoutput.h macros — expand to ConSrvGetObject with the right
    // buffer-type enum tag. Bindgen sees ConSrvGetObject but not the
    // wrapper macros.
    use crate::winsrv::{
        PCONSOLE_PROCESS_DATA, PCONSOLE_IO_OBJECT, TEXTMODE_BUFFER,
        GRAPHICS_BUFFER, SCREEN_BUFFER, ConSrvGetObject, ConSrvReleaseObject,
    };
    use crate::winsrv::{HANDLE, BOOLEAN, NTSTATUS, ULONG, PVOID};

    #[inline]
    pub unsafe fn ConSrvGetTextModeBuffer(
        ProcessData: PCONSOLE_PROCESS_DATA,
        Handle: HANDLE,
        Ptr: *mut PCONSOLE_IO_OBJECT,
        Access: ULONG,
        LockConsole: BOOLEAN,
    ) -> NTSTATUS {
        ConSrvGetObject(ProcessData, Handle, Ptr, core::ptr::null_mut(),
                        Access, LockConsole, TEXTMODE_BUFFER)
    }

    #[inline]
    pub unsafe fn ConSrvGetTextModeBufferAndHandleEntry(
        ProcessData: PCONSOLE_PROCESS_DATA,
        Handle: HANDLE,
        Ptr: *mut PCONSOLE_IO_OBJECT,
        Entry: *mut PVOID,
        Access: ULONG,
        LockConsole: BOOLEAN,
    ) -> NTSTATUS {
        ConSrvGetObject(ProcessData, Handle, Ptr, Entry,
                        Access, LockConsole, TEXTMODE_BUFFER)
    }

    #[inline]
    pub unsafe fn ConSrvGetGraphicsBuffer(
        ProcessData: PCONSOLE_PROCESS_DATA,
        Handle: HANDLE,
        Ptr: *mut PCONSOLE_IO_OBJECT,
        Access: ULONG,
        LockConsole: BOOLEAN,
    ) -> NTSTATUS {
        ConSrvGetObject(ProcessData, Handle, Ptr, core::ptr::null_mut(),
                        Access, LockConsole, GRAPHICS_BUFFER)
    }

    #[inline]
    pub unsafe fn ConSrvGetScreenBuffer(
        ProcessData: PCONSOLE_PROCESS_DATA,
        Handle: HANDLE,
        Ptr: *mut PCONSOLE_IO_OBJECT,
        Access: ULONG,
        LockConsole: BOOLEAN,
    ) -> NTSTATUS {
        ConSrvGetObject(ProcessData, Handle, Ptr, core::ptr::null_mut(),
                        Access, LockConsole, SCREEN_BUFFER)
    }

    #[inline]
    pub unsafe fn ConSrvGetScreenBufferAndHandleEntry(
        ProcessData: PCONSOLE_PROCESS_DATA,
        Handle: HANDLE,
        Ptr: *mut PCONSOLE_IO_OBJECT,
        Entry: *mut PVOID,
        Access: ULONG,
        LockConsole: BOOLEAN,
    ) -> NTSTATUS {
        ConSrvGetObject(ProcessData, Handle, Ptr, Entry,
                        Access, LockConsole, SCREEN_BUFFER)
    }

    /// `ConSrvReleaseScreenBuffer(Buff, IsConsoleLocked)` —
    /// `ConSrvReleaseObject(&(Buff)->Header, IsConsoleLocked)`.
    /// The macro is generic over screen/textmode/graphics buffer
    /// pointer types since each has a `Header` of type
    /// `CONSOLE_IO_OBJECT`; we just take a `*mut c_void` and
    /// cast.
    #[inline]
    pub unsafe fn ConSrvReleaseScreenBuffer<T>(
        Buff: *mut T,
        IsConsoleLocked: BOOLEAN,
    ) {
        // CONSOLE_IO_OBJECT is the first member of the screen buffer
        // structs (CONSOLE_SCREEN_BUFFER.Header, TEXTMODE_SCREEN_BUFFER
        // also has .Header at offset 0). Cast and dispatch.
        ConSrvReleaseObject(Buff as PCONSOLE_IO_OBJECT, IsConsoleLocked);
    }

    /// `IsConsoleHandle(h)` — console.h:14 macro:
    /// `(((ULONG_PTR)(h) & 0x10000003) == 0x3)`.
    #[inline]
    pub fn IsConsoleHandle(h: *mut core::ffi::c_void) -> bool {
        (h as usize & 0x10000003) == 0x3
    }

    /// `ConsoleGetPerProcessData(Process)` — consrv.h:38 macro:
    /// `((PCONSOLE_PROCESS_DATA)((Process)->ServerData[CONSRV_SERVERDLL_INDEX]))`.
    /// CSR_PROCESS has a ServerData PVOID flexible array; index 2 is consrv's slot.
    #[inline]
    pub unsafe fn ConsoleGetPerProcessData(
        Process: *mut crate::winsrv::CSR_PROCESS,
    ) -> crate::winsrv::PCONSOLE_PROCESS_DATA {
        let server_data_base = core::ptr::addr_of_mut!((*Process).ServerData) as *mut crate::winsrv::PVOID;
        let slot = *server_data_base.add(crate::winsrv::CONSRV_SERVERDLL_INDEX as usize);
        slot as crate::winsrv::PCONSOLE_PROCESS_DATA
    }

    /// `LANGIDFROMLCID(lcid)` — macro `(WORD)(lcid)`.
    #[inline]
    pub fn LANGIDFROMLCID(lcid: u32) -> u16 {
        (lcid & 0xFFFF) as u16
    }

    /// `NT_SUCCESS(Status)` — macro `(((NTSTATUS)(Status)) >= 0)`.
    /// Returns `bool` (Rust-idiomatic); call sites using the C
    /// `NT_SUCCESS(s) == 0` pattern must be rewritten to
    /// `!NT_SUCCESS(s)`.
    #[inline]
    pub fn NT_SUCCESS(status: i32) -> bool {
        status >= 0
    }

    /// `RtlCopyMemory(dest, src, len)` — Windows macro that aliases
    /// memcpy. Translated as inline wrapper around `core::ptr::copy`.
    #[inline]
    pub unsafe fn RtlCopyMemory(
        dest: *mut core::ffi::c_void,
        src: *const core::ffi::c_void,
        len: usize,
    ) {
        core::ptr::copy(src as *const u8, dest as *mut u8, len);
    }

    /// `InterlockedExchangePointer(target, value)` — atomic
    /// pointer swap. Implemented via `AtomicPtr::swap` with
    /// `SeqCst` ordering (the conservative match for the Win32
    /// Interlocked* full-barrier semantics).
    #[inline]
    pub unsafe fn InterlockedExchangePointer(
        target: *mut *mut core::ffi::c_void,
        value: *mut core::ffi::c_void,
    ) -> *mut core::ffi::c_void {
        use core::sync::atomic::{AtomicPtr, Ordering};
        let atomic = &*(target as *const AtomicPtr<core::ffi::c_void>);
        atomic.swap(value, Ordering::SeqCst)
    }
}

/// Hand-written type aliases for winsrv that bindgen sees only under
/// their `_USERCONNECT` original names — the win/winmsg.h `#define`
/// chain (`#define _USERSRV_API_CONNECTINFO _USERCONNECT`) isn't
/// followed by bindgen. Glob-re-exported into `winsrv::*` so the
/// translator's single glob picks them up.
pub mod aliases {
    pub use crate::winsrv::{
        _USERCONNECT as _USERSRV_API_CONNECTINFO,
        _USERCONNECT as USERSRV_API_CONNECTINFO,
        USERCONNECT,
        PUSERCONNECT as PUSERSRV_API_CONNECTINFO,
    };
}

/// winsrv-internal API — auto-generated bindings with correct per-arch
/// calling conventions (i386 stdcall/fastcall decoration too).
///
/// Covers the consrv driver core (`ConDrv*`/`ConSrv*`), the
/// TERMINAL/FRONTEND vtables and their impls (Term* in tui/gui/serial
/// frontends), the concfg surface (`TT*`/`Settings*`/`Font*`), and the
/// usersrv surface (`UserServer*`/`UserClient*`/`UserInit*`/
/// `UserSound*`). Also the wincon/winuser/wingdi/imm types these
/// internals project (CONSOLE_*/COORD/INPUT_RECORD/HWND/HDC/LOGFONTW/
/// WNDPROC/MSG/…).
pub mod winsrv {
    pub use super::bindgen_common::*;

    #[cfg(target_arch = "aarch64")]
    pub use super::bindgen_arm64::*;

    #[cfg(target_arch = "x86_64")]
    pub use super::bindgen_amd64::*;

    #[cfg(target_arch = "x86")]
    pub use super::bindgen_i386::*;

    // Hand-written aliases the bindgen #define chain doesn't follow.
    pub use super::aliases::*;
}

// conmsg.h's own C_ASSERT, plus the invariant postprocess_bindgen.py's anonymous-member
// inlining relies on: CONSOLE_PROPERTIES spliced in flat must end exactly where the C struct
// does. bindgen 0.73 changed the padding's spelling and the inline silently stopped
// happening; a layout check is what turns the next such change into a compile error.
#[cfg(target_arch = "x86")]
const _: () = assert!(core::mem::size_of::<winsrv::CONSOLE_START_INFO>() == 0xFC);
const _: () = assert!(
    core::mem::offset_of!(winsrv::CONSOLE_START_INFO, IconIndex)
        + core::mem::size_of::<winsrv::CONSOLE_PROPERTIES>()
        == core::mem::size_of::<winsrv::CONSOLE_START_INFO>()
);
