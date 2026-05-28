//! consrv/ — Console subsystem server (CSR server DLL surface for
//! kernel32 console APIs + console host process integration).
//!
//! `condrv/` is the UI-agnostic core (buffers, input queue, lifetime,
//! dummy terminal vtbl). Frontends (TUI/GUI terminals) plug in via
//! `TERMINAL_VTBL`. Phase 5.3 lands `condrv/`; Phase 5.4 lands the
//! consrv top + frontends dispatch (alias/coninput/conoutput/console/
//! frontendctl/handle/history/init/lineinput/popup/settings/shutdown/
//! subsysreg + `frontends/{input,terminal,wcwidth}`).

pub mod condrv;

// Phase 5.4: consrv top
pub mod alias;
pub mod coninput;
pub mod conoutput;
pub mod console;
pub mod frontendctl;
pub mod handle;
pub mod history;
pub mod init;
pub mod lineinput;
pub mod popup;
pub mod settings;
pub mod shutdown;
pub mod subsysreg;
pub mod term;

// include/conio.h macros — bindgen doesn't see file-scope macros.
#[inline(always)]
#[allow(non_snake_case)]
pub unsafe fn GetType(this: winsrv_sys::winsrv::PCONSOLE_SCREEN_BUFFER) -> winsrv_sys::winsrv::CONSOLE_IO_OBJECT_TYPE {
    (*this).Header.Type
}

// InputBuffer.Mode is USHORT (u16); the conio.h macro expands to it directly.
#[inline(always)]
#[allow(non_snake_case)]
pub unsafe fn GetConsoleInputBufferMode(Console: winsrv_sys::winsrv::PCONSRV_CONSOLE) -> winsrv_sys::winsrv::USHORT {
    (*Console).InputBuffer.Mode
}

// Phase 5.4: frontends dispatch
pub mod frontends;

// consrv/heap.h macro shims:
//   #define ConsoleAllocHeap(Flags, Size) RtlAllocateHeap(ConSrvHeap, Flags, Size)
//   #define ConsoleFreeHeap(HeapBase)     RtlFreeHeap(ConSrvHeap, 0, HeapBase)
// Glob-re-export via consrv root so child files can `use super::*` (mod.rs
// inheritance) or extern-link from siblings.
#[inline]
pub unsafe fn ConsoleAllocHeap(
    Flags: winsrv_sys::winsrv::ULONG,
    Size: usize,
) -> winsrv_sys::winsrv::PVOID {
    // RtlAllocateHeap takes SIZE_T (u32 on i386, u64 on amd64/arm64).
    // Cast lossy on i386; allocations > 4GB aren't a real concern.
    winsrv_sys::winsrv::RtlAllocateHeap(
        init::ConSrvHeap as _,
        Flags,
        Size as winsrv_sys::winsrv::SIZE_T,
    )
}

#[inline]
pub unsafe fn ConsoleFreeHeap(HeapBase: winsrv_sys::winsrv::PVOID) -> winsrv_sys::winsrv::BOOLEAN {
    winsrv_sys::winsrv::RtlFreeHeap(init::ConSrvHeap as _, 0, HeapBase)
}

// Helpers that bindgen didn't pull in (user32/msvcrt fns outside FN_ALLOW).
// Declared here so all 5.4 + 5.5 files can resolve them via super:: globs.
extern "system" {
    pub fn wcslen(s: *const u16) -> usize;
    pub fn wcsrchr(s: *const u16, c: u16) -> *mut u16;
    pub fn wcsncpy(dest: *mut u16, src: *const u16, n: usize) -> *mut u16;
    pub fn wcscat(dest: *mut u16, src: *const u16) -> *mut u16;
    pub fn wcschr(s: *const u16, c: u16) -> *mut u16;
    pub fn wcscpy(dest: *mut u16, src: *const u16) -> *mut u16;
    pub fn wcscmp(s1: *const u16, s2: *const u16) -> i32;
    pub fn memcmp(s1: *const core::ffi::c_void, s2: *const core::ffi::c_void, n: usize) -> i32;
    pub fn _wcsicmp(s1: *const u16, s2: *const u16) -> i32;
    pub fn _stricmp(s1: *const i8, s2: *const i8) -> i32;
    pub fn strchr(s: *const i8, c: i32) -> *mut i8;
    pub fn PrivateExtractIconsW(
        szFileName: winsrv_sys::winsrv::LPCWSTR,
        nIconIndex: i32,
        cxIcon: i32,
        cyIcon: i32,
        phicon: *mut winsrv_sys::winsrv::HICON,
        piconid: *mut u32,
        nIcons: u32,
        flags: u32,
    ) -> u32;
    pub fn VkKeyScanW(ch: u16) -> i16;
    pub fn ToUnicodeEx(
        wVirtKey: u32,
        wScanCode: u32,
        lpKeyState: *const u8,
        pwszBuff: *mut u16,
        cchBuff: i32,
        wFlags: u32,
        dwhkl: winsrv_sys::winsrv::HKL,
    ) -> i32;
}

