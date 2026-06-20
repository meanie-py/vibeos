//! ntwrapper.rs — Rust port of win32ss/user/user32/include/ntwrapper.h
//!
//! Faithful, mechanical translation of the 149 `EXTINLINE` inline wrappers that
//! sit between the user32 public API and win32k. Two families:
//!   1. Direct forwards to a `NtUser*` win32k syscall.
//!   2. The `NtUserx*` family, which forwards to one of the 8 `NtUserCall*`
//!      multiplexers with a `SimpleCallRoutines` selector constant.
//!
//! These are PLAIN `#[inline]` fns (not `#[no_mangle]`, not `extern`) — they
//! inline into the translated user32 bodies that call them. They are NOT
//! exported symbols.
//!
//! Substrate: every `NtUser*` syscall fn, every `NtUserCall*` multiplexer, every
//! `*_ROUTINE_*` selector const (typed `SimpleCallRoutines`), and the Win32 type
//! aliases all come from `win32k-sys`.
//!
//! Casting notes (faithful to the C):
//!   - Routine selector consts are `SimpleCallRoutines`-typed; the multiplexers
//!     take `DWORD Routine` → cast `as DWORD`.
//!   - The multiplexers take `DWORD_PTR` params and (except `NtUserCallHwndParam`,
//!     which returns `DWORD`) return `DWORD_PTR` → cast args/results to match the
//!     C's explicit `(DWORD_PTR)` / `(BOOL)` / `(HANDLE)` casts.
//!   - A few bindgen syscall signatures spell pointer/scalar types differently
//!     than the public user32 prototype (POINT/POINTL, RECT/RECTL, int/LONG,
//!     BOOL/BOOLEAN). These are layout-identical ReactOS aliases; args are cast
//!     with `as _` and divergent return types with `as <CType>` so the wrapper
//!     keeps the public user32 signature.
//!
//! Excluded:
//!   - `#if 0` block (GetScrollBarInfo) and the `/* ... */` GetMenuBarInfo.
//!   - `NtUserxClearWindowState` / `NtUserxSetWindowState`: depend on `TestState`
//!     and `UserHMGetHandle`, neither present in win32k-sys (see TODOs below).
//!   - `BOOL FASTCALL TestState(PWND, UINT);` import: `TestState` not in win32k-sys.
//!
//! WIN32K_VERSION = NTDDI_WINXPSP2 in this build, so any Vista+-gated wrapper is
//! excluded. (None of the wrappers in this header are version-gated.)

use win32k_sys::win32k::*;

// ----------------------------------------------------------------------------
// TODO: not present in win32k-sys — wrappers left out (see module doc):
//   - TestState  (BOOL FASTCALL TestState(PWND, UINT))   -> used by ClearWindowState/SetWindowState
//   - UserHMGetHandle                                     -> used by ClearWindowState/SetWindowState
// Skipped wrappers: NtUserxClearWindowState, NtUserxSetWindowState
// ----------------------------------------------------------------------------

// ===========================================================================
// Direct NtUser* syscall forwards
// ===========================================================================

#[inline]
pub unsafe fn ShowScrollBar(hWnd: HWND, wBar: INT, bShow: BOOL) -> BOOL {
    NtUserShowScrollBar(hWnd, wBar, bShow as DWORD) as BOOL
}

#[inline]
pub unsafe fn CloseDesktop(hDesktop: HDESK) -> BOOL {
    NtUserCloseDesktop(hDesktop)
}

#[inline]
pub unsafe fn OpenInputDesktop(dwFlags: DWORD, fInherit: BOOL, dwDesiredAccess: ACCESS_MASK) -> HDESK {
    NtUserOpenInputDesktop(dwFlags, fInherit, dwDesiredAccess)
}

#[inline]
pub unsafe fn PaintDesktop(hdc: HDC) -> BOOL {
    NtUserPaintDesktop(hdc)
}

#[inline]
pub unsafe fn SetThreadDesktop(hDesktop: HDESK) -> BOOL {
    NtUserSetThreadDesktop(hDesktop)
}

#[inline]
pub unsafe fn SwitchDesktop(hDesktop: HDESK) -> BOOL {
    NtUserSwitchDesktop(hDesktop)
}

#[inline]
pub unsafe fn SetShellWindowEx(hwndShell: HWND, hwndShellListView: HWND) -> BOOL {
    NtUserSetShellWindowEx(hwndShell, hwndShellListView)
}

#[inline]
pub unsafe fn GetGuiResources(hProcess: HANDLE, uiFlags: DWORD) -> DWORD {
    NtUserGetGuiResources(hProcess, uiFlags)
}

#[inline]
pub unsafe fn GetUserObjectInformationW(
    hObj: HANDLE,
    nIndex: INT,
    pvInfo: PVOID,
    nLength: DWORD,
    lpnLengthNeeded: LPDWORD,
) -> BOOL {
    NtUserGetObjectInformation(hObj, nIndex as DWORD, pvInfo, nLength, lpnLengthNeeded as _)
}

#[inline]
pub unsafe fn LockWindowUpdate(hWndLock: HWND) -> BOOL {
    NtUserLockWindowUpdate(hWndLock)
}

#[inline]
pub unsafe fn LockWorkStation() -> BOOL {
    NtUserLockWorkStation()
}

#[inline]
pub unsafe fn RegisterTasklist(x: DWORD) -> DWORD {
    NtUserRegisterTasklist(x)
}

#[inline]
pub unsafe fn DragObject(hwnd1: HWND, hwnd2: HWND, u1: UINT, dw1: ULONG_PTR, hc1: HCURSOR) -> DWORD {
    NtUserDragObject(hwnd1, hwnd2, u1, dw1 as DWORD, hc1)
}

#[inline]
pub unsafe fn KillTimer(hWnd: HWND, IDEvent: UINT_PTR) -> BOOL {
    NtUserKillTimer(hWnd, IDEvent)
}

#[inline]
pub unsafe fn SetSystemTimer(hWnd: HWND, IDEvent: UINT_PTR, Period: UINT, TimerFunc: TIMERPROC) -> UINT_PTR {
    NtUserSetSystemTimer(hWnd, IDEvent, Period, TimerFunc)
}

#[inline]
pub unsafe fn SetTimer(hWnd: HWND, IDEvent: UINT_PTR, Period: UINT, TimerFunc: TIMERPROC) -> UINT_PTR {
    NtUserSetTimer(hWnd, IDEvent, Period, TimerFunc)
}

#[inline]
pub unsafe fn CloseWindowStation(hWinSta: HWINSTA) -> BOOL {
    NtUserCloseWindowStation(hWinSta)
}

#[inline]
pub unsafe fn GetProcessWindowStation() -> HWINSTA {
    NtUserGetProcessWindowStation()
}

#[inline]
pub unsafe fn SetProcessWindowStation(hWinSta: HWINSTA) -> BOOL {
    NtUserSetProcessWindowStation(hWinSta)
}

#[inline]
pub unsafe fn LockWindowStation(hWinSta: HWINSTA) -> BOOL {
    NtUserLockWindowStation(hWinSta)
}

#[inline]
pub unsafe fn UnlockWindowStation(hWinSta: HWINSTA) -> BOOL {
    NtUserUnlockWindowStation(hWinSta)
}

#[inline]
pub unsafe fn CopyAcceleratorTableW(hAccelSrc: HACCEL, lpAccelDst: LPACCEL, cAccelEntries: INT) -> INT {
    NtUserCopyAcceleratorTable(hAccelSrc, lpAccelDst, cAccelEntries as ULONG) as INT
}

#[inline]
pub unsafe fn CreateAcceleratorTableW(lpaccl: LPACCEL, cEntries: INT) -> HACCEL {
    NtUserCreateAcceleratorTable(lpaccl, cEntries as ULONG)
}

#[inline]
pub unsafe fn CreateCaret(hWnd: HWND, hBitmap: HBITMAP, nWidth: INT, nHeight: INT) -> BOOL {
    NtUserCreateCaret(hWnd, hBitmap, nWidth, nHeight)
}

#[inline]
pub unsafe fn GetCaretBlinkTime() -> UINT {
    NtUserGetCaretBlinkTime()
}

#[inline]
pub unsafe fn GetCaretPos(lpPoint: LPPOINT) -> BOOL {
    NtUserGetCaretPos(lpPoint as _)
}

#[inline]
pub unsafe fn CloseClipboard() -> BOOL {
    NtUserCloseClipboard()
}

#[inline]
pub unsafe fn CountClipboardFormats() -> INT {
    NtUserCountClipboardFormats() as INT
}

#[inline]
pub unsafe fn EmptyClipboard() -> BOOL {
    NtUserEmptyClipboard()
}

#[inline]
pub unsafe fn GetClipboardOwner() -> HWND {
    NtUserGetClipboardOwner()
}

#[inline]
pub unsafe fn GetClipboardSequenceNumber() -> DWORD {
    NtUserGetClipboardSequenceNumber()
}

#[inline]
pub unsafe fn GetClipboardViewer() -> HWND {
    NtUserGetClipboardViewer()
}

#[inline]
pub unsafe fn GetOpenClipboardWindow() -> HWND {
    NtUserGetOpenClipboardWindow()
}

#[inline]
pub unsafe fn GetPriorityClipboardFormat(paFormatPriorityList: *mut UINT, cFormats: INT) -> INT {
    NtUserGetPriorityClipboardFormat(paFormatPriorityList, cFormats)
}

#[inline]
pub unsafe fn IsClipboardFormatAvailable(format: UINT) -> BOOL {
    NtUserIsClipboardFormatAvailable(format)
}

#[inline]
pub unsafe fn SetClipboardViewer(hWndNewViewer: HWND) -> HWND {
    NtUserSetClipboardViewer(hWndNewViewer)
}

#[inline]
pub unsafe fn ChangeClipboardChain(hWndRemove: HWND, hWndNewNext: HWND) -> BOOL {
    NtUserChangeClipboardChain(hWndRemove, hWndNewNext)
}

#[inline]
pub unsafe fn GetClipCursor(lpRect: LPRECT) -> BOOL {
    NtUserGetClipCursor(lpRect as _)
}

#[inline]
pub unsafe fn GetControlBrush(hwnd: HWND, hdc: HDC, ctlType: UINT) -> HBRUSH {
    NtUserGetControlBrush(hwnd, hdc, ctlType)
}

#[inline]
pub unsafe fn GetControlColor(hwndParent: HWND, hwnd: HWND, hdc: HDC, CtlMsg: UINT) -> HBRUSH {
    NtUserGetControlColor(hwndParent, hwnd, hdc, CtlMsg)
}

#[inline]
pub unsafe fn GetCursorInfo(pci: PCURSORINFO) -> BOOL {
    NtUserGetCursorInfo(pci)
}

#[inline]
pub unsafe fn ClipCursor(lpRect: *const RECT) -> BOOL {
    // C: NtUserClipCursor((RECT *)lpRect)
    NtUserClipCursor(lpRect as *mut _)
}

#[inline]
pub unsafe fn SetCursor(hCursor: HCURSOR) -> HCURSOR {
    NtUserSetCursor(hCursor)
}

#[inline]
pub unsafe fn GetDC(hWnd: HWND) -> HDC {
    NtUserGetDC(hWnd)
}

#[inline]
pub unsafe fn GetDCEx(hWnd: HWND, hrgnClip: HRGN, flags: DWORD) -> HDC {
    NtUserGetDCEx(hWnd, hrgnClip as HANDLE, flags)
}

#[inline]
pub unsafe fn GetWindowDC(hWnd: HWND) -> HDC {
    NtUserGetWindowDC(hWnd)
}

#[inline]
pub unsafe fn FlashWindowEx(pfwi: PFLASHWINFO) -> BOOL {
    NtUserFlashWindowEx(pfwi)
}

#[inline]
pub unsafe fn DrawAnimatedRects(hWnd: HWND, idAni: INT, lprcFrom: *const RECT, lprcTo: *const RECT) -> BOOL {
    // C: NtUserDrawAnimatedRects(hWnd, idAni, (RECT*)lprcFrom, (RECT*)lprcTo)
    NtUserDrawAnimatedRects(hWnd, idAni, lprcFrom as *mut _, lprcTo as *mut _)
}

#[inline]
pub unsafe fn BlockInput(fBlockIt: BOOL) -> BOOL {
    NtUserBlockInput(fBlockIt)
}

#[inline]
pub unsafe fn GetDoubleClickTime() -> UINT {
    NtUserGetDoubleClickTime()
}

#[inline]
pub unsafe fn GetKeyboardState(lpKeyState: PBYTE) -> BOOL {
    // C: NtUserGetKeyboardState((LPBYTE)lpKeyState)
    NtUserGetKeyboardState(lpKeyState as LPBYTE) as BOOL
}

#[inline]
pub unsafe fn RegisterHotKey(hWnd: HWND, id: INT, fsModifiers: UINT, vk: UINT) -> BOOL {
    NtUserRegisterHotKey(hWnd, id, fsModifiers, vk)
}

#[inline]
pub unsafe fn SetFocus(hWnd: HWND) -> HWND {
    NtUserSetFocus(hWnd)
}

#[inline]
pub unsafe fn SetKeyboardState(lpKeyState: LPBYTE) -> BOOL {
    // C: NtUserSetKeyboardState((LPBYTE)lpKeyState)
    NtUserSetKeyboardState(lpKeyState as LPBYTE)
}

#[inline]
pub unsafe fn SendInput(nInputs: UINT, pInputs: LPINPUT, cbSize: INT) -> UINT {
    NtUserSendInput(nInputs, pInputs, cbSize)
}

#[inline]
pub unsafe fn WaitMessage() -> BOOL {
    NtUserWaitMessage()
}

#[inline]
pub unsafe fn BeginPaint(hwnd: HWND, lpPaint: LPPAINTSTRUCT) -> HDC {
    NtUserBeginPaint(hwnd, lpPaint)
}

#[inline]
pub unsafe fn EndPaint(hWnd: HWND, lpPaint: *const PAINTSTRUCT) -> BOOL {
    NtUserEndPaint(hWnd, lpPaint)
}

#[inline]
pub unsafe fn ExcludeUpdateRgn(hDC: HDC, hWnd: HWND) -> INT {
    NtUserExcludeUpdateRgn(hDC, hWnd)
}

#[inline]
pub unsafe fn InvalidateRgn(hWnd: HWND, hRgn: HRGN, bErase: BOOL) -> BOOL {
    NtUserInvalidateRgn(hWnd, hRgn, bErase)
}

#[inline]
pub unsafe fn RedrawWindow(hWnd: HWND, lprcUpdate: *const RECT, hrgnUpdate: HRGN, flags: UINT) -> BOOL {
    NtUserRedrawWindow(hWnd, lprcUpdate as _, hrgnUpdate, flags)
}

#[inline]
pub unsafe fn DestroyWindow(hWnd: HWND) -> BOOL {
    NtUserDestroyWindow(hWnd) as BOOL
}

#[inline]
pub unsafe fn GetForegroundWindow() -> HWND {
    NtUserGetForegroundWindow()
}

#[inline]
pub unsafe fn GetGUIThreadInfo(idThread: DWORD, lpgui: LPGUITHREADINFO) -> BOOL {
    NtUserGetGUIThreadInfo(idThread, lpgui)
}

#[inline]
pub unsafe fn GetTitleBarInfo(hwnd: HWND, pti: PTITLEBARINFO) -> BOOL {
    NtUserGetTitleBarInfo(hwnd, pti) as BOOL
}

#[inline]
pub unsafe fn GetWindowPlacement(hWnd: HWND, lpwndpl: *mut WINDOWPLACEMENT) -> BOOL {
    NtUserGetWindowPlacement(hWnd, lpwndpl)
}

#[inline]
pub unsafe fn MoveWindow(hWnd: HWND, X: INT, Y: INT, nWidth: INT, nHeight: INT, bRepaint: BOOL) -> BOOL {
    NtUserMoveWindow(hWnd, X, Y, nWidth, nHeight, bRepaint)
}

#[inline]
pub unsafe fn SetParent(hWndChild: HWND, hWndNewParent: HWND) -> HWND {
    NtUserSetParent(hWndChild, hWndNewParent)
}

#[inline]
pub unsafe fn SetWindowPlacement(hWnd: HWND, lpwndpl: *const WINDOWPLACEMENT) -> BOOL {
    // C: NtUserSetWindowPlacement(hWnd, (WINDOWPLACEMENT *)lpwndpl)
    NtUserSetWindowPlacement(hWnd, lpwndpl as *mut _)
}

#[inline]
pub unsafe fn SetWindowPos(hWnd: HWND, hWndAfter: HWND, X: INT, Y: INT, cx: INT, cy: INT, uFlags: UINT) -> BOOL {
    NtUserSetWindowPos(hWnd, hWndAfter, X, Y, cx, cy, uFlags)
}

#[inline]
pub unsafe fn ShowWindow(hWnd: HWND, nCmdShow: INT) -> BOOL {
    NtUserShowWindow(hWnd, nCmdShow as LONG)
}

#[inline]
pub unsafe fn ShowWindowAsync(hWnd: HWND, nCmdShow: INT) -> BOOL {
    NtUserShowWindowAsync(hWnd, nCmdShow as LONG)
}

#[inline]
pub unsafe fn SetActiveWindow(hWnd: HWND) -> HWND {
    NtUserSetActiveWindow(hWnd)
}

#[inline]
pub unsafe fn GetListBoxInfo(hwnd: HWND) -> DWORD {
    // Do it right! Have the message org from kmode!
    NtUserGetListBoxInfo(hwnd)
}

#[inline]
pub unsafe fn DeleteMenu(hMenu: HMENU, uPosition: UINT, uFlags: UINT) -> BOOL {
    NtUserDeleteMenu(hMenu, uPosition, uFlags)
}

#[inline]
pub unsafe fn DestroyMenu(hMenu: HMENU) -> BOOL {
    NtUserDestroyMenu(hMenu)
}

#[inline]
pub unsafe fn GetMenuItemRect(hWnd: HWND, hMenu: HMENU, uItem: UINT, lprcItem: LPRECT) -> BOOL {
    NtUserGetMenuItemRect(hWnd, hMenu, uItem, lprcItem as _)
}

#[inline]
pub unsafe fn RemoveMenu(hMenu: HMENU, uPosition: UINT, uFlags: UINT) -> BOOL {
    NtUserRemoveMenu(hMenu, uPosition, uFlags)
}

#[inline]
pub unsafe fn SetMenuDefaultItem(hMenu: HMENU, uItem: UINT, fByPos: UINT) -> BOOL {
    NtUserSetMenuDefaultItem(hMenu, uItem, fByPos)
}

#[inline]
pub unsafe fn SetMenuContextHelpId(hmenu: HMENU, dwContextHelpId: DWORD) -> BOOL {
    NtUserSetMenuContextHelpId(hmenu, dwContextHelpId)
}

#[inline]
pub unsafe fn SetCapture(hWnd: HWND) -> HWND {
    NtUserSetCapture(hWnd)
}

#[inline]
pub unsafe fn InvalidateRect(hWnd: HWND, lpRect: *const RECT, bErase: BOOL) -> BOOL {
    NtUserInvalidateRect(hWnd, lpRect as _, bErase)
}

#[inline]
pub unsafe fn ValidateRect(hWnd: HWND, lpRect: *const RECT) -> BOOL {
    NtUserValidateRect(hWnd, lpRect as _)
}

#[inline]
pub unsafe fn ShowCaret(hWnd: HWND) -> BOOL {
    NtUserShowCaret(hWnd)
}

#[inline]
pub unsafe fn HideCaret(hWnd: HWND) -> BOOL {
    NtUserHideCaret(hWnd)
}

// ===========================================================================
// NtUserx* family — forward to the NtUserCall* multiplexers
//
// Inline functions that make calling NtUserCall*** functions readable.
// These functions are prepended with NtUserx because they are not real
// syscalls and they are inlined.
// ===========================================================================

#[inline]
pub unsafe fn NtUserxDestroyCaret() -> BOOL {
    NtUserCallNoParam(NOPARAM_ROUTINE_DESTROY_CARET as DWORD) as BOOL
}

#[inline]
pub unsafe fn NtUserxMsqClearWakeMask() {
    NtUserCallNoParam(NOPARAM_ROUTINE_MSQCLEARWAKEMASK as DWORD);
}

#[inline]
pub unsafe fn NtUserxCreateMenu() -> HMENU {
    NtUserCallNoParam(NOPARAM_ROUTINE_CREATEMENU as DWORD) as HMENU
}

#[inline]
pub unsafe fn NtUserxCreatePopupMenu() -> HMENU {
    NtUserCallNoParam(NOPARAM_ROUTINE_CREATEMENUPOPUP as DWORD) as HMENU
}

#[inline]
pub unsafe fn NtUserxGetMessagePos() -> DWORD {
    NtUserCallNoParam(NOPARAM_ROUTINE_GETMSESSAGEPOS as DWORD) as DWORD
}

#[inline]
pub unsafe fn NtUserxReleaseCapture() -> BOOL {
    NtUserCallNoParam(NOPARAM_ROUTINE_RELEASECAPTURE as DWORD) as BOOL
}

#[inline]
pub unsafe fn NtUserxInitMessagePump() -> BOOL {
    NtUserCallNoParam(NOPARAM_ROUTINE_INIT_MESSAGE_PUMP as DWORD) as BOOL
}

#[inline]
pub unsafe fn NtUserxUnInitMessagePump() -> BOOL {
    NtUserCallNoParam(NOPARAM_ROUTINE_UNINIT_MESSAGE_PUMP as DWORD) as BOOL
}

#[inline]
pub unsafe fn NtUserxMsqSetWakeMask(dwWaitMask: DWORD_PTR) -> HANDLE {
    NtUserCallOneParam(dwWaitMask, ONEPARAM_ROUTINE_GETINPUTEVENT as DWORD) as HANDLE
}

#[inline]
pub unsafe fn NtUserxSetCaretBlinkTime(uMSeconds: UINT) -> BOOL {
    NtUserCallOneParam(uMSeconds as DWORD_PTR, ONEPARAM_ROUTINE_SETCARETBLINKTIME as DWORD) as BOOL
}

#[inline]
pub unsafe fn NtUserxWindowFromDC(hDC: HDC) -> HWND {
    NtUserCallOneParam(hDC as DWORD_PTR, ONEPARAM_ROUTINE_WINDOWFROMDC as DWORD) as HWND
}

#[inline]
pub unsafe fn NtUserxSwapMouseButton(fSwap: BOOL) -> BOOL {
    NtUserCallOneParam(fSwap as DWORD_PTR, ONEPARAM_ROUTINE_SWAPMOUSEBUTTON as DWORD) as BOOL
}

#[inline]
pub unsafe fn NtUserxSetMessageExtraInfo(lParam: LPARAM) -> LPARAM {
    NtUserCallOneParam(lParam as DWORD_PTR, ONEPARAM_ROUTINE_SETMESSAGEEXTRAINFO as DWORD) as LPARAM
}

#[inline]
pub unsafe fn NtUserxShowCursor(bShow: BOOL) -> INT {
    NtUserCallOneParam(bShow as DWORD_PTR, ONEPARAM_ROUTINE_SHOWCURSOR as DWORD) as INT
}

#[inline]
pub unsafe fn NtUserxEnumClipboardFormats(format: UINT) -> UINT {
    NtUserCallOneParam(format as DWORD_PTR, ONEPARAM_ROUTINE_ENUMCLIPBOARDFORMATS as DWORD) as UINT
}

#[inline]
pub unsafe fn NtUserxCreateEmptyCurObject(Param: DWORD_PTR) -> HICON {
    NtUserCallOneParam(Param, ONEPARAM_ROUTINE_CREATEEMPTYCUROBJECT as DWORD) as HICON
}

#[inline]
pub unsafe fn NtUserxMessageBeep(uType: UINT) -> BOOL {
    NtUserCallOneParam(uType as DWORD_PTR, ONEPARAM_ROUTINE_MESSAGEBEEP as DWORD) as BOOL
}

#[inline]
pub unsafe fn NtUserxGetKeyboardLayout(idThread: DWORD) -> HKL {
    NtUserCallOneParam(idThread as DWORD_PTR, ONEPARAM_ROUTINE_GETKEYBOARDLAYOUT as DWORD) as HKL
}

#[inline]
pub unsafe fn NtUserxGetKeyboardType(nTypeFlag: INT) -> INT {
    NtUserCallOneParam(nTypeFlag as DWORD_PTR, ONEPARAM_ROUTINE_GETKEYBOARDTYPE as DWORD) as INT
}

#[inline]
pub unsafe fn NtUserxReleaseDC(hDC: HDC) -> INT {
    NtUserCallOneParam(hDC as DWORD_PTR, ONEPARAM_ROUTINE_RELEASEDC as DWORD) as INT
}

#[inline]
pub unsafe fn NtUserxRealizePalette(hDC: HDC) -> UINT {
    NtUserCallOneParam(hDC as DWORD_PTR, ONEPARAM_ROUTINE_REALIZEPALETTE as DWORD) as UINT
}

#[inline]
pub unsafe fn NtUserxCreateSystemThreads(bRemoteProcess: BOOL) {
    NtUserCallOneParam(bRemoteProcess as DWORD_PTR, ONEPARAM_ROUTINE_CREATESYSTEMTHREADS as DWORD);
}

#[inline]
pub unsafe fn NtUserxBeginDeferWindowPos(nNumWindows: INT) -> HDWP {
    NtUserCallOneParam(nNumWindows as DWORD_PTR, ONEPARAM_ROUTINE_BEGINDEFERWNDPOS as DWORD) as HDWP
}

#[inline]
pub unsafe fn NtUserxReplyMessage(lResult: LRESULT) -> BOOL {
    NtUserCallOneParam(lResult as DWORD_PTR, ONEPARAM_ROUTINE_REPLYMESSAGE as DWORD) as BOOL
}

#[inline]
pub unsafe fn NtUserxPostQuitMessage(nExitCode: INT) {
    NtUserCallOneParam(nExitCode as DWORD_PTR, ONEPARAM_ROUTINE_POSTQUITMESSAGE as DWORD);
}

#[inline]
pub unsafe fn NtUserxGetQueueStatus(flags: UINT) -> DWORD {
    NtUserCallOneParam(flags as DWORD_PTR, ONEPARAM_ROUTINE_GETQUEUESTATUS as DWORD) as DWORD
}

#[inline]
pub unsafe fn NtUserxValidateRgn(hWnd: HWND, hRgn: HRGN) -> BOOL {
    NtUserCallHwndParamLock(hWnd, hRgn as DWORD_PTR, TWOPARAM_ROUTINE_VALIDATERGN as DWORD) as BOOL
}

#[inline]
pub unsafe fn NtUserxSetCursorPos(x: INT, y: INT) -> BOOL {
    NtUserCallTwoParam(x as DWORD as DWORD_PTR, y as DWORD as DWORD_PTR, TWOPARAM_ROUTINE_SETCURSORPOS as DWORD) as BOOL
}

#[inline]
pub unsafe fn NtUserxEnableWindow(hWnd: HWND, bEnable: BOOL) -> BOOL {
    NtUserCallTwoParam(hWnd as DWORD_PTR, bEnable as DWORD_PTR, TWOPARAM_ROUTINE_ENABLEWINDOW as DWORD) as BOOL
}

#[inline]
pub unsafe fn NtUserxUpdateUiState(hWnd: HWND, Param: DWORD) -> BOOL {
    NtUserCallTwoParam(hWnd as DWORD_PTR, Param as DWORD_PTR, TWOPARAM_ROUTINE_ROS_UPDATEUISTATE as DWORD) as BOOL
}

#[inline]
pub unsafe fn NtUserxSwitchToThisWindow(hWnd: HWND, fAltTab: BOOL) {
    NtUserCallTwoParam(hWnd as DWORD_PTR, fAltTab as DWORD_PTR, TWOPARAM_ROUTINE_SWITCHTOTHISWINDOW as DWORD);
}

#[inline]
pub unsafe fn NtUserxShowOwnedPopups(hWnd: HWND, fShow: BOOL) -> BOOL {
    NtUserCallTwoParam(hWnd as DWORD_PTR, fShow as DWORD_PTR, TWOPARAM_ROUTINE_SHOWOWNEDPOPUPS as DWORD) as BOOL
}

#[inline]
pub unsafe fn NtUserxUnhookWindowsHook(nCode: INT, pfnFilterProc: HOOKPROC) -> BOOL {
    NtUserCallTwoParam(nCode as DWORD_PTR, pfnFilterProc.map_or(0, |p| p as usize) as DWORD_PTR, TWOPARAM_ROUTINE_UNHOOKWINDOWSHOOK as DWORD) as BOOL
}

#[inline]
pub unsafe fn NtUserxSetWindowContextHelpId(hWnd: HWND, dwContextHelpId: DWORD_PTR) -> BOOL {
    NtUserCallHwndParam(hWnd, dwContextHelpId, HWNDPARAM_ROUTINE_SETWNDCONTEXTHLPID as DWORD) as BOOL
}

#[inline]
pub unsafe fn NtUserxKillSystemTimer(hWnd: HWND, IDEvent: UINT_PTR) -> BOOL {
    NtUserCallHwndParam(hWnd, IDEvent as DWORD_PTR, HWNDPARAM_ROUTINE_KILLSYSTEMTIMER as DWORD) as BOOL
}

#[inline]
pub unsafe fn NtUserxSetDialogPointer(hWnd: HWND, dlgInfo: PVOID) {
    NtUserCallHwndParam(hWnd, dlgInfo as DWORD_PTR, HWNDPARAM_ROUTINE_SETDIALOGPOINTER as DWORD);
}

#[inline]
pub unsafe fn NtUserxNotifyWinEvent(hWnd: HWND, ne: PVOID) {
    NtUserCallHwndParam(hWnd, ne as DWORD_PTR, HWNDPARAM_ROUTINE_ROS_NOTIFYWINEVENT as DWORD);
}

#[inline]
pub unsafe fn NtUserxGetWindowContextHelpId(hwnd: HWND) -> DWORD {
    NtUserCallHwnd(hwnd, HWND_ROUTINE_GETWNDCONTEXTHLPID as DWORD) as DWORD
}

#[inline]
pub unsafe fn NtUserxDeregisterShellHookWindow(hWnd: HWND) -> BOOL {
    NtUserCallHwnd(hWnd, HWND_ROUTINE_DEREGISTERSHELLHOOKWINDOW as DWORD) as BOOL
}

#[inline]
pub unsafe fn NtUserxRegisterShellHookWindow(hWnd: HWND) -> BOOL {
    NtUserCallHwnd(hWnd, HWND_ROUTINE_REGISTERSHELLHOOKWINDOW as DWORD) as BOOL
}

#[inline]
pub unsafe fn NtUserxSetMessageBox(hWnd: HWND) -> BOOL {
    NtUserCallHwnd(hWnd, HWND_ROUTINE_SETMSGBOX as DWORD) as BOOL
}

// NOTE: NtUserxClearWindowState / NtUserxSetWindowState omitted — they call
// TestState() and UserHMGetHandle(), neither of which exist in win32k-sys.
// (HWNDPARAM_ROUTINE_CLEARWINDOWSTATE / _SETWINDOWSTATE consts DO exist.)

#[inline]
pub unsafe fn NtUserxSetTaskmanWindow(hWnd: HWND) -> HWND {
    NtUserCallHwndOpt(hWnd, HWNDOPT_ROUTINE_SETTASKMANWINDOW as DWORD) as HWND
}

#[inline]
pub unsafe fn NtUserxSetProgmanWindow(hWnd: HWND) -> HWND {
    NtUserCallHwndOpt(hWnd, HWNDOPT_ROUTINE_SETPROGMANWINDOW as DWORD) as HWND
}

#[inline]
pub unsafe fn NtUserxArrangeIconicWindows(hWnd: HWND) -> UINT {
    NtUserCallHwndLock(hWnd, HWNDLOCK_ROUTINE_ARRANGEICONICWINDOWS as DWORD) as UINT
}

#[inline]
pub unsafe fn NtUserxUpdateWindow(hWnd: HWND) -> BOOL {
    NtUserCallHwndLock(hWnd, HWNDLOCK_ROUTINE_UPDATEWINDOW as DWORD) as BOOL
}

#[inline]
pub unsafe fn NtUserxDrawMenuBar(hWnd: HWND) -> BOOL {
    NtUserCallHwndLock(hWnd, HWNDLOCK_ROUTINE_DRAWMENUBAR as DWORD) as BOOL
}

#[inline]
pub unsafe fn NtUserxMDIRedrawFrame(hWnd: HWND) -> BOOL {
    NtUserCallHwndLock(hWnd, HWNDLOCK_ROUTINE_REDRAWFRAME as DWORD) as BOOL
}

#[inline]
pub unsafe fn NtUserxSetForegroundWindow(hWnd: HWND) -> BOOL {
    NtUserCallHwndLock(hWnd, HWNDLOCK_ROUTINE_SETFOREGROUNDWINDOW as DWORD) as BOOL
}

// ===========================================================================
// ReactOS-specific definitions
// ===========================================================================

#[inline]
pub unsafe fn NtUserxGetMessageExtraInfo() -> LPARAM {
    NtUserGetThreadState(THREADSTATE_GETMESSAGEEXTRAINFO as DWORD) as LPARAM
}

#[inline]
pub unsafe fn NtUserxEnableProcessWindowGhosting(bEnable: BOOL) {
    NtUserCallOneParam(bEnable as DWORD_PTR, ONEPARAM_ROUTINE_ENABLEPROCWNDGHSTING as DWORD);
}

#[inline]
pub unsafe fn NtUserxGetDesktopMapping(ptr: PVOID) -> PVOID {
    NtUserCallOneParam(ptr as DWORD_PTR, ONEPARAM_ROUTINE_GETDESKTOPMAPPING as DWORD) as PVOID
}

#[inline]
pub unsafe fn NtUserxGetCursorPos(lpPoint: *mut POINT) -> BOOL {
    NtUserCallOneParam(lpPoint as DWORD_PTR, ONEPARAM_ROUTINE_GETCURSORPOS as DWORD) as BOOL
}

#[inline]
pub unsafe fn NtUserxSetMenuBarHeight(menu: HMENU, height: INT) -> BOOL {
    NtUserCallTwoParam(menu as DWORD_PTR, height as DWORD_PTR, TWOPARAM_ROUTINE_SETMENUBARHEIGHT as DWORD) as BOOL
}

#[inline]
pub unsafe fn NtUserxSetGUIThreadHandle(field: DWORD_PTR, hwnd: HWND) -> BOOL {
    NtUserCallTwoParam(field, hwnd as DWORD_PTR, TWOPARAM_ROUTINE_SETGUITHRDHANDLE as DWORD) as BOOL
}

#[inline]
pub unsafe fn NtUserxSetCaretPos(x: INT, y: INT) -> BOOL {
    NtUserCallTwoParam(x as DWORD_PTR, y as DWORD_PTR, TWOPARAM_ROUTINE_SETCARETPOS as DWORD) as BOOL
}

#[inline]
pub unsafe fn NtUserxRegisterLogonProcess(dwProcessId: DWORD, bRegister: BOOL) -> BOOL {
    NtUserCallTwoParam(dwProcessId as DWORD_PTR, bRegister as DWORD_PTR, TWOPARAM_ROUTINE_REGISTERLOGONPROCESS as DWORD) as BOOL
}

#[inline]
pub unsafe fn NtUserxAllowSetForegroundWindow(dwProcessId: DWORD) -> BOOL {
    NtUserCallOneParam(dwProcessId as DWORD_PTR, ONEPARAM_ROUTINE_ALLOWSETFOREGND as DWORD) as BOOL
}

#[inline]
pub unsafe fn NtUserxLockSetForegroundWindow(uLockCode: UINT) -> BOOL {
    NtUserCallOneParam(uLockCode as DWORD_PTR, ONEPARAM_ROUTINE_LOCKFOREGNDWINDOW as DWORD) as BOOL
}
