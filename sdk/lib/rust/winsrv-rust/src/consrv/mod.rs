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
pub mod shim;
pub mod shutdown;
pub mod subsysreg;
pub mod term;

// include/conio.h macros — bindgen doesn't see file-scope macros.
// Promoted to #[no_mangle] extern "system" so the (now-Rust) frontends/ files
// can extern-link them across the gui/* modules (#36 frontends flip). Internal
// callers still reach them via `super::GetType` (same symbol).
#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "system" fn GetType(this: winsrv_sys::winsrv::PCONSOLE_SCREEN_BUFFER) -> winsrv_sys::winsrv::CONSOLE_IO_OBJECT_TYPE {
    (*this).Header.Type
}

// InputBuffer.Mode is USHORT (u16); the conio.h macro expands to it directly.
#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "system" fn GetConsoleInputBufferMode(Console: winsrv_sys::winsrv::PCONSRV_CONSOLE) -> winsrv_sys::winsrv::USHORT {
    (*Console).InputBuffer.Mode
}

// CON_SET_OUTPUT_CP(Console, CodePage) — used by the (now-Rust) settings.rs +
// conwnd.rs. There is NO C macro/def for this in the tree (the only C callers
// were settings.c/conwnd.c, already flipped); semantically it just sets the
// output code page. Provided here as the canonical owner of the conio helpers.
#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "system" fn CON_SET_OUTPUT_CP(
    Console: winsrv_sys::winsrv::PCONSRV_CONSOLE,
    CodePage: winsrv_sys::winsrv::UINT,
) {
    (*Console).OutputCodePage = CodePage;
}

// guiterm.h FORCEINLINE helpers (GetScreenBufferSizeUnits, SmallRectToRect).
// rustconv emitted extern decls for these across the (now-Rust) frontends/gui
// files but never a body, so flipping the frontends surfaced them as undefined
// link symbols. They are ALSO declared by `winsrv_sys::winsrv` (bindgen turns
// FORCEINLINE into an extern "C" decl), so the gui/*.rs callers already resolve
// the NAME via their `use winsrv_sys::winsrv::*` glob — we only need to PROVIDE
// THE SYMBOL. Defining them here (a module the gui files do NOT glob-import)
// avoids the E0659 ambiguity that a frontends/gui/helpers.rs def would cause,
// and #[no_mangle] extern "C" matches bindgen's cdecl decl exactly.
mod gui_forceinline_helpers {
    use winsrv_sys::winsrv::*;

    /// Pixel size of one console cell: TEXTMODE uses the font cell size,
    /// GRAPHICS buffers are 1:1.
    #[no_mangle]
    #[allow(non_snake_case)]
    pub unsafe extern "C" fn GetScreenBufferSizeUnits(
        Buffer: PCONSOLE_SCREEN_BUFFER,
        GuiData: PGUI_CONSOLE_DATA,
        WidthUnit: PUINT,
        HeightUnit: PUINT,
    ) {
        if (*Buffer).Header.Type as u32 == TEXTMODE_BUFFER as u32 {
            *WidthUnit = (*GuiData).CharWidth;
            *HeightUnit = (*GuiData).CharHeight;
        } else {
            *WidthUnit = 1;
            *HeightUnit = 1;
        }
    }

    /// Convert a console SMALL_RECT (cell coords) to a window RECT (pixels)
    /// via the active buffer's view origin and the per-cell unit size.
    #[no_mangle]
    #[allow(non_snake_case)]
    pub unsafe extern "C" fn SmallRectToRect(
        GuiData: PGUI_CONSOLE_DATA,
        Rect: PRECT,
        SmallRect: PSMALL_RECT,
    ) {
        let Buffer: PCONSOLE_SCREEN_BUFFER = (*GuiData).ActiveBuffer;
        let mut WidthUnit: UINT = 0;
        let mut HeightUnit: UINT = 0;
        GetScreenBufferSizeUnits(Buffer, GuiData, &mut WidthUnit, &mut HeightUnit);
        let ox = (*Buffer).ViewOrigin.X as i32;
        let oy = (*Buffer).ViewOrigin.Y as i32;
        (*Rect).left   = (((*SmallRect).Left   as i32     - ox) * WidthUnit  as i32) as _;
        (*Rect).top    = (((*SmallRect).Top    as i32     - oy) * HeightUnit as i32) as _;
        (*Rect).right  = (((*SmallRect).Right  as i32 + 1 - ox) * WidthUnit  as i32) as _;
        (*Rect).bottom = (((*SmallRect).Bottom as i32 + 1 - oy) * HeightUnit as i32) as _;
    }
}

// Phase 5.4: frontends dispatch
pub mod frontends;

// consrv/heap.h macro shims:
//   #define ConsoleAllocHeap(Flags, Size) RtlAllocateHeap(ConSrvHeap, Flags, Size)
//   #define ConsoleFreeHeap(HeapBase)     RtlFreeHeap(ConSrvHeap, 0, HeapBase)
// Glob-re-export via consrv root so child files can `use super::*` (mod.rs
// inheritance) or extern-link from siblings.
#[no_mangle]
pub unsafe extern "system" fn ConsoleAllocHeap(
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

#[no_mangle]
pub unsafe extern "system" fn ConsoleFreeHeap(HeapBase: winsrv_sys::winsrv::PVOID) -> winsrv_sys::winsrv::BOOLEAN {
    winsrv_sys::winsrv::RtlFreeHeap(init::ConSrvHeap as _, 0, HeapBase)
}

// Helpers that bindgen didn't pull in (user32/msvcrt fns outside FN_ALLOW).
// Declared here so all 5.4 + 5.5 files can resolve them via super:: globs.
//
// CRT string/mem helpers are plain cdecl (provided by libcntpr as `_name`),
// so they MUST be extern "C" — declaring them extern "system" decorates them
// `_name@N` on i386, which libcntpr does not export.
extern "C" {
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
}
// Win32 user32/shell32 APIs are stdcall → extern "system".
extern "system" {
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

