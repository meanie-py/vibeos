//! Rust transcription of the `Term*` macros in
//! `win32ss/user/winsrv/consrv/include/term.h`. Each macro is a
//! one-liner that calls into the `TERMINAL_VTBL` or `FRONTEND_VTBL`
//! virtual table on `Console->TermIFace` or `Console->FrontEndIFace`.
//!
//! Translated to `pub unsafe fn` — bodies dereference the vtbl and
//! call through it. The callees set `extern "stdcall"` on i386 and
//! `extern "C"` on amd64/arm64 (bindgen FN-pointer field types), but
//! at the IR level the call site is the same per arch.

use winsrv_sys::winsrv::*;

#[inline(always)]
pub unsafe fn TermReadStream(
    Console: PCONSRV_CONSOLE,
    Unicode: BOOLEAN,
    Buffer: PVOID,
    ReadControl: PCONSOLE_READCONSOLE_CONTROL,
    Parameter: PVOID,
    NumCharsToRead: ULONG,
    NumCharsRead: PULONG,
) -> NTSTATUS {
    let term_iface = core::ptr::addr_of_mut!((*Console).TermIFace);
    let vtbl = (*term_iface).Vtbl;
    ((*vtbl).ReadStream.unwrap())(term_iface, Unicode, Buffer, ReadControl, Parameter, NumCharsToRead, NumCharsRead)
}

#[inline(always)]
pub unsafe fn TermWriteStream(
    Console: PCONSRV_CONSOLE,
    ScreenBuffer: PTEXTMODE_SCREEN_BUFFER,
    Buffer: PWCHAR,
    Length: DWORD,
    Attrib: BOOL,
) -> NTSTATUS {
    let term_iface = core::ptr::addr_of_mut!((*Console).TermIFace);
    let vtbl = (*term_iface).Vtbl;
    ((*vtbl).WriteStream.unwrap())(term_iface, ScreenBuffer, Buffer, Length, Attrib)
}

#[inline(always)]
pub unsafe fn TermDrawRegion(Console: PCONSRV_CONSOLE, Region: *mut SMALL_RECT) {
    let term_iface = core::ptr::addr_of_mut!((*Console).TermIFace);
    let vtbl = (*term_iface).Vtbl;
    ((*vtbl).DrawRegion.unwrap())(term_iface, Region);
}

#[inline(always)]
pub unsafe fn TermSetCursorInfo(Console: PCONSRV_CONSOLE, ScreenBuffer: PCONSOLE_SCREEN_BUFFER) -> BOOL {
    let term_iface = core::ptr::addr_of_mut!((*Console).TermIFace);
    let vtbl = (*term_iface).Vtbl;
    ((*vtbl).SetCursorInfo.unwrap())(term_iface, ScreenBuffer)
}

#[inline(always)]
pub unsafe fn TermSetScreenInfo(
    Console: PCONSRV_CONSOLE,
    ScreenBuffer: PCONSOLE_SCREEN_BUFFER,
    OldCursorX: SHORT,
    OldCursorY: SHORT,
) -> BOOL {
    let term_iface = core::ptr::addr_of_mut!((*Console).TermIFace);
    let vtbl = (*term_iface).Vtbl;
    ((*vtbl).SetScreenInfo.unwrap())(term_iface, ScreenBuffer, OldCursorX, OldCursorY)
}

// condrv flip: settings.rs/conwnd.rs reference this as a linked cdecl symbol
// `_TermResizeTerminal` (vtbl dispatcher) — export it (was inline-only).
#[no_mangle]
pub unsafe extern "C" fn TermResizeTerminal(Console: PCONSRV_CONSOLE) {
    let term_iface = core::ptr::addr_of_mut!((*Console).TermIFace);
    let vtbl = (*term_iface).Vtbl;
    ((*vtbl).ResizeTerminal.unwrap())(term_iface);
}

#[inline(always)]
pub unsafe fn TermSetActiveScreenBuffer(Console: PCONSRV_CONSOLE) {
    let term_iface = core::ptr::addr_of_mut!((*Console).TermIFace);
    let vtbl = (*term_iface).Vtbl;
    ((*vtbl).SetActiveScreenBuffer.unwrap())(term_iface);
}

#[inline(always)]
pub unsafe fn TermReleaseScreenBuffer(Console: PCONSRV_CONSOLE, ScreenBuffer: PCONSOLE_SCREEN_BUFFER) {
    let term_iface = core::ptr::addr_of_mut!((*Console).TermIFace);
    let vtbl = (*term_iface).Vtbl;
    ((*vtbl).ReleaseScreenBuffer.unwrap())(term_iface, ScreenBuffer);
}

#[inline(always)]
pub unsafe fn TermGetLargestConsoleWindowSize(Console: PCONSRV_CONSOLE, pSize: PCOORD) {
    let term_iface = core::ptr::addr_of_mut!((*Console).TermIFace);
    let vtbl = (*term_iface).Vtbl;
    ((*vtbl).GetLargestConsoleWindowSize.unwrap())(term_iface, pSize);
}

#[inline(always)]
pub unsafe fn TermSetPalette(Console: PCONSRV_CONSOLE, PaletteHandle: HPALETTE, PaletteUsage: UINT) -> BOOL {
    let term_iface = core::ptr::addr_of_mut!((*Console).TermIFace);
    let vtbl = (*term_iface).Vtbl;
    ((*vtbl).SetPalette.unwrap())(term_iface, PaletteHandle, PaletteUsage)
}

// condrv flip: settings.rs references this as a linked cdecl `_TermSetCodePage`.
#[no_mangle]
pub unsafe extern "C" fn TermSetCodePage(Console: PCONSRV_CONSOLE, CodePage: UINT) -> BOOL {
    let term_iface = core::ptr::addr_of_mut!((*Console).TermIFace);
    let vtbl = (*term_iface).Vtbl;
    ((*vtbl).SetCodePage.unwrap())(term_iface, CodePage)
}

#[inline(always)]
pub unsafe fn TermShowMouseCursor(Console: PCONSRV_CONSOLE, Show: BOOL) -> INT {
    let term_iface = core::ptr::addr_of_mut!((*Console).TermIFace);
    let vtbl = (*term_iface).Vtbl;
    ((*vtbl).ShowMouseCursor.unwrap())(term_iface, Show)
}

// --- FRONTEND_VTBL ---

#[inline(always)]
pub unsafe fn TermRefreshInternalInfo(Console: PCONSRV_CONSOLE) {
    let fe_iface = core::ptr::addr_of_mut!((*Console).FrontEndIFace);
    let vtbl = (*fe_iface).Vtbl;
    ((*vtbl).RefreshInternalInfo.unwrap())(fe_iface);
}

#[inline(always)]
pub unsafe fn TermChangeTitle(Console: PCONSRV_CONSOLE) {
    let fe_iface = core::ptr::addr_of_mut!((*Console).FrontEndIFace);
    let vtbl = (*fe_iface).Vtbl;
    ((*vtbl).ChangeTitle.unwrap())(fe_iface);
}

#[inline(always)]
pub unsafe fn TermChangeIcon(Console: PCONSRV_CONSOLE, IconHandle: HICON) -> BOOL {
    let fe_iface = core::ptr::addr_of_mut!((*Console).FrontEndIFace);
    let vtbl = (*fe_iface).Vtbl;
    ((*vtbl).ChangeIcon.unwrap())(fe_iface, IconHandle)
}

#[inline(always)]
pub unsafe fn TermGetThreadConsoleDesktop(Console: PCONSRV_CONSOLE) -> HDESK {
    let fe_iface = core::ptr::addr_of_mut!((*Console).FrontEndIFace);
    let vtbl = (*fe_iface).Vtbl;
    ((*vtbl).GetThreadConsoleDesktop.unwrap())(fe_iface)
}

#[inline(always)]
pub unsafe fn TermGetConsoleWindowHandle(Console: PCONSRV_CONSOLE) -> HWND {
    let fe_iface = core::ptr::addr_of_mut!((*Console).FrontEndIFace);
    let vtbl = (*fe_iface).Vtbl;
    ((*vtbl).GetConsoleWindowHandle.unwrap())(fe_iface)
}

#[inline(always)]
pub unsafe fn TermGetSelectionInfo(Console: PCONSRV_CONSOLE, pSelectionInfo: PCONSOLE_SELECTION_INFO) -> BOOL {
    let fe_iface = core::ptr::addr_of_mut!((*Console).FrontEndIFace);
    let vtbl = (*fe_iface).Vtbl;
    ((*vtbl).GetSelectionInfo.unwrap())(fe_iface, pSelectionInfo)
}

#[inline(always)]
pub unsafe fn TermGetDisplayMode(Console: PCONSRV_CONSOLE) -> ULONG {
    let fe_iface = core::ptr::addr_of_mut!((*Console).FrontEndIFace);
    let vtbl = (*fe_iface).Vtbl;
    ((*vtbl).GetDisplayMode.unwrap())(fe_iface)
}

#[inline(always)]
pub unsafe fn TermSetDisplayMode(Console: PCONSRV_CONSOLE, NewMode: ULONG) -> BOOL {
    let fe_iface = core::ptr::addr_of_mut!((*Console).FrontEndIFace);
    let vtbl = (*fe_iface).Vtbl;
    ((*vtbl).SetDisplayMode.unwrap())(fe_iface, NewMode)
}

#[inline(always)]
pub unsafe fn TermSetMouseCursor(Console: PCONSRV_CONSOLE, CursorHandle: HCURSOR) -> BOOL {
    let fe_iface = core::ptr::addr_of_mut!((*Console).FrontEndIFace);
    let vtbl = (*fe_iface).Vtbl;
    ((*vtbl).SetMouseCursor.unwrap())(fe_iface, CursorHandle)
}

#[inline(always)]
pub unsafe fn TermMenuControl(Console: PCONSRV_CONSOLE, CmdIdLow: UINT, CmdIdHigh: UINT) -> HMENU {
    let fe_iface = core::ptr::addr_of_mut!((*Console).FrontEndIFace);
    let vtbl = (*fe_iface).Vtbl;
    ((*vtbl).MenuControl.unwrap())(fe_iface, CmdIdLow, CmdIdHigh)
}

#[inline(always)]
pub unsafe fn TermSetMenuClose(Console: PCONSRV_CONSOLE, Enable: BOOL) -> BOOL {
    let fe_iface = core::ptr::addr_of_mut!((*Console).FrontEndIFace);
    let vtbl = (*fe_iface).Vtbl;
    ((*vtbl).SetMenuClose.unwrap())(fe_iface, Enable)
}

#[inline(always)]
pub unsafe fn TermGetFontSize(Console: PCONSRV_CONSOLE, pSize: PCOORD) {
    let fe_iface = core::ptr::addr_of_mut!((*Console).FrontEndIFace);
    let vtbl = (*fe_iface).Vtbl;
    // A frontend without the op reports no font (0x0) rather than taking csrss down.
    match (*vtbl).GetFontSize {
        Some(GetFontSize) => GetFontSize(fe_iface, pSize),
        None => *pSize = COORD { X: 0, Y: 0 },
    }
}
