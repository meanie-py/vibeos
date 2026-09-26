use ntkrnl_sys::driver_ddk::*;
use win32k_sys::win32k::*;
use ntkrnl_sys::ddk::{ExAllocatePoolWithTag, ExFreePoolWithTag, PsGetCurrentProcessSessionId, PsGetCurrentThreadWin32Thread, RtlAllocateHeap, RtlFreeHeap, RtlSizeHeap, RtlCopyUnicodeString, RtlInitUnicodeString};
use ntkrnl_sys::ob::ObDereferenceObject;
use ntkrnl_sys::ps::PsGetCurrentProcess;
use ntkrnl_sys::base::{NT_SUCCESS, STATUS_SUCCESS, STATUS_INSUFFICIENT_RESOURCES};
use core::ffi::c_void;

// LOWORD/HIWORD/MAKELPARAM/MAKEWPARAM (minwindef.h).
#[inline(always)]
fn LOWORD(l: DWORD) -> DWORD { l & 0xffff }
#[inline(always)]
fn HIWORD(l: DWORD) -> DWORD { (l >> 16) & 0xffff }
#[inline(always)]
fn MAKELPARAM(low: u32, high: u32) -> LPARAM { (((low & 0xffff) | ((high & 0xffff) << 16)) as i32) as LPARAM }
#[inline(always)]
fn MAKEWPARAM(low: u32, high: u32) -> WPARAM { ((low & 0xffff) | ((high & 0xffff) << 16)) as WPARAM }

// max (windef.h).
#[inline(always)]
fn max<T: PartialOrd>(a: T, b: T) -> T { if a > b { a } else { b } }

// WINVER (windef.h default for win32k).
const WINVER: u32 = 0x0502;
// FIELD_OFFSET(type, field) — use offset_of! at call sites; provide a passthrough
// for the (struct, field) form the translation emits is not needed (handled inline).

// HBMMENU_* system-menu bitmap ids (winuser.h), as HBITMAP.
const HBMMENU_CALLBACK: HBITMAP = (-1isize) as HBITMAP;
const HBMMENU_MBAR_MINIMIZE: HBITMAP = 3usize as HBITMAP;
const HBMMENU_MBAR_MINIMIZE_D: HBITMAP = 7usize as HBITMAP;
const HBMMENU_POPUP_CLOSE: HBITMAP = 8usize as HBITMAP;
const HBMMENU_POPUP_RESTORE: HBITMAP = 9usize as HBITMAP;
const HBMMENU_POPUP_MAXIMIZE: HBITMAP = 10usize as HBITMAP;
const HBMMENU_POPUP_MINIMIZE: HBITMAP = 11usize as HBITMAP;
// HWND_TOPMOST = (HWND)-1.
const HWND_TOPMOST: HWND = (-1isize) as HWND;
// WC_MENU = MAKEINTATOM(0x8000) (undocuser.h) = (LPWSTR)(ULONG_PTR)(WORD)0x8000.
const WC_MENU: usize = 0x8000;

// UserHMGetHandle(obj) = obj->head.h (object.h macro). Generic over object type.
#[inline(always)]
unsafe fn UserHMGetHandle<T>(obj: *mut T) -> HANDLE {
    (*(obj as *const _HEAD)).h
}

// IS_SYSTEM_MENU(MenuInfo) = !!(MenuInfo->fFlags & MNF_SYSMENU) (menu.c).
#[inline(always)]
unsafe fn IS_SYSTEM_MENU(menu: PMENU) -> BOOL {
    (!menu.is_null() && ((*menu).fFlags & MNF_SYSMENU) != 0) as BOOL
}

// RECTL_vSetEmptyRect (gdi inline) — zero a rect. Takes RECT (callers pass &kmbi.rcBar).
#[inline(always)]
unsafe fn RECTL_vSetEmptyRect(prcl: *mut RECT) {
    (*prcl).left = 0; (*prcl).top = 0; (*prcl).right = 0; (*prcl).bottom = 0;
}

// IntReleaseMenuObject(MenuObj) = UserDereferenceObject(MenuObj) (menu.h). Mirrors simplecall.rs.
#[inline(always)]
unsafe fn IntReleaseMenuObject(MenuObj: PMENU) {
    UserDereferenceObject(MenuObj as PVOID);
}

// gpsi PERUSERSERVERINFO tail accessor (oembmi etc). Mirrors sysparams.rs.
#[inline(always)]
unsafe fn pusi(p: PSERVERINFO) -> *mut _PERUSERSERVERINFO {
    (p as *mut u8).add(core::mem::offset_of!(tagSERVERINFO, __bindgen_padding_0)) as *mut _PERUSERSERVERINFO
}

// THRDESKHEAD.pti flattened field (offset 2*usize). Mirrors ime.rs/input.rs.
#[inline(always)]
unsafe fn head_pti<T>(obj: *const T) -> PTHREADINFO {
    *((obj as *const u8).add(2 * core::mem::size_of::<usize>()) as *const PTHREADINFO)
}

// GetW32ThreadInfo / GetW32ProcessInfo — win32.h: cast current win32 thread/process.
#[inline(always)]
unsafe fn GetW32ThreadInfo() -> PTHREADINFO { PsGetCurrentThreadWin32Thread() as PTHREADINFO }
#[inline(always)]
unsafe fn GetW32ProcessInfo() -> PPROCESSINFO {
    ntkrnl_sys::ddk::PsGetCurrentProcessWin32Process() as PPROCESSINFO
}

// RtlCopyMemory / RtlZeroMemory — compiler intrinsics.
#[inline(always)]
unsafe fn RtlCopyMemory(dst: PVOID, src: *const c_void, len: SIZE_T) {
    core::ptr::copy_nonoverlapping(src as *const u8, dst as *mut u8, len as usize);
}
#[inline(always)]
unsafe fn RtlZeroMemory(dst: PVOID, len: SIZE_T) {
    core::ptr::write_bytes(dst as *mut u8, 0, len as usize);
}

// NT_VERIFY(x) = (x) in non-checked builds.
#[inline(always)]
fn NT_VERIFY(cond: bool) -> bool { cond }

// wcschr / towupper — CRT (libcntpr).
extern "C" {
    fn wcschr(s: *const u16, c: core::ffi::c_int) -> *mut u16;
    fn towupper(c: core::ffi::c_int) -> core::ffi::c_int;
}

// UpdateMenuItemState!(state, change) — menu.c macro. Updates state bits per change.
macro_rules! UpdateMenuItemState {
    ($state:expr, $change:expr) => {{
        let change = $change;
        for bit in [MF_GRAYED, MF_DISABLED, MFS_CHECKED, MFS_HILITE, MFS_DEFAULT,
                    MF_MOUSESELECT, MF_BITMAP, MF_OWNERDRAW] {
            if (change & bit) != 0 { $state |= bit; } else { $state &= !bit; }
        }
    }};
}

// FreeMenuText!(Menu, MenuItem) — menu.c macro: free the item string buffer.
macro_rules! FreeMenuText {
    ($menu:expr, $item:expr) => {{
        let item = $item;
        if (MENU_ITEM_TYPE((*item).fType) == MF_STRING) && (*item).lpstr.Length != 0 {
            DesktopHeapFree((*($menu as PMENU)).head.rpdesk, (*item).lpstr.Buffer as *mut c_void);
        }
    }};
}

// win32k internal exports referenced by menu (cdecl, FASTCALL on i386).
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn UITOOLS95_DrawFrameMenu(dc: HDC, r: LPRECT, uFlags: UINT) -> BOOL;
    fn NC_GetInsideRect(Wnd: PWND, rect: *mut RECT);
    fn NC_GetSysPopupPos(Wnd: PWND, Rect: *mut RECT);
    fn RECTL_bIntersectRect(prclDst: *mut RECTL, prcl1: *const RECTL, prcl2: *const RECTL) -> BOOL;
    fn VerifyWnd(pWnd: PWND) -> PWND;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn UITOOLS95_DrawFrameMenu(dc: HDC, r: LPRECT, uFlags: UINT) -> BOOL;
    fn NC_GetInsideRect(Wnd: PWND, rect: *mut RECT);
    fn NC_GetSysPopupPos(Wnd: PWND, Rect: *mut RECT);
    fn RECTL_bIntersectRect(prclDst: *mut RECTL, prcl1: *const RECTL, prcl2: *const RECTL) -> BOOL;
    fn VerifyWnd(pWnd: PWND) -> PWND;
}
// RECTL_vSetRect (rect.h inline).
#[inline(always)]
unsafe fn RECTL_vSetRect(rect: *mut RECT, left: LONG, top: LONG, right: LONG, bottom: LONG) {
    (*rect).left = left; (*rect).top = top; (*rect).right = right; (*rect).bottom = bottom;
}
extern "C" {
    // _MmCopyToCaller / _MmCopyFromCaller — C macros forward to these cdecl fns
    // (mmcopy.h); provided by copy.rs. Wrapped below.
    fn _MmCopyToCaller(Target: *mut core::ffi::c_void, Source: *const core::ffi::c_void, Bytes: UINT) -> NTSTATUS;
    fn _MmCopyFromCaller(Target: *mut core::ffi::c_void, Source: *const core::ffi::c_void, Bytes: UINT) -> NTSTATUS;
}
// RECTL_vOffsetRect (rect.h inline).
#[inline(always)]
unsafe fn RECTL_vOffsetRect(rect: *mut RECT, cx: i32, cy: i32) {
    (*rect).left += cx; (*rect).right += cx; (*rect).top += cy; (*rect).bottom += cy;
}
// IS_MAGIC_BITMAP(id) = id && id < 12 && id >= -1 (menu.c macro).
#[inline(always)]
fn IS_MAGIC_BITMAP(id: INT_PTR) -> BOOL {
    (id != 0 && id < 12 && id >= -1) as BOOL
}
// UserGetMenuObject (userfuncs.h FORCEINLINE; mirrors nonclient.rs).
#[inline(always)]
unsafe fn UserGetMenuObject(hMenu: HMENU) -> PMENU {
    let pMenu = UserGetObject(gHandleTable, hMenu as _, TYPE_MENU) as PMENU;
    if pMenu.is_null() {
        EngSetLastError(ERROR_INVALID_MENU_HANDLE);
    }
    pMenu
}
// co_IntSendMessage / co_WinPosSetWindowPos / ValidateHwndNoErr — real FASTCALL
// exports (message.rs / winpos.rs / window.rs). i386 fastcall, cdecl elsewhere.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn co_IntSendMessage(Wnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn co_WinPosSetWindowPos( Wnd: PWND, WndInsertAfter: HWND, x: i32, y: i32, cx: i32, cy: i32, flags: u32, ) -> BOOL;
    fn ValidateHwndNoErr(hWnd: HWND) -> PWND;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn co_IntSendMessage(Wnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn co_WinPosSetWindowPos( Wnd: PWND, WndInsertAfter: HWND, x: i32, y: i32, cx: i32, cy: i32, flags: u32, ) -> BOOL;
    fn ValidateHwndNoErr(hWnd: HWND) -> PWND;
}
extern "system" {
    fn SafeProbeForRead( Address: *mut core::ffi::c_void, Length: SIZE_T, Alignment: ULONG, ) -> NTSTATUS;
    fn SafeProbeForWrite( Address: *mut core::ffi::c_void, Length: usize, Alignment: u32, ) -> NTSTATUS;
    // UserReferenceObject / UserDereferenceObject — fastcall on i386 actually; but
    // UserRefObjectCo (object.h inline) is reimplemented locally below.
}
// fastcall (i386) UserReferenceObject/UserDereferenceObject for UserRefObjectCo inline.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn UserReferenceObject(obj: PVOID);
    fn UserDereferenceObject(Object: PVOID) -> BOOL;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn UserReferenceObject(obj: PVOID);
    fn UserDereferenceObject(Object: PVOID) -> BOOL;
}

// ── header-inline / macro helpers (no exported symbol; reimplemented locally) ──
// HAS_DLGFRAME / HAS_THICKFRAME / HAS_THINFRAME (window.h macros).
#[inline(always)]
fn HAS_DLGFRAME(style: u32, ex_style: u32) -> BOOL {
    ((ex_style & WS_EX_DLGMODALFRAME) != 0
        || ((style & WS_DLGFRAME) != 0 && (style & WS_THICKFRAME) == 0)) as BOOL
}
#[inline(always)]
fn HAS_THICKFRAME(style: u32, _ex_style: u32) -> BOOL {
    ((style & WS_THICKFRAME) != 0
        && !((style & (WS_DLGFRAME | WS_BORDER)) == WS_DLGFRAME)) as BOOL
}
#[inline(always)]
fn HAS_THINFRAME(style: u32, _ex_style: u32) -> BOOL {
    ((style & WS_BORDER) != 0 || (style & (WS_CHILD | WS_POPUP)) == 0) as BOOL
}
// MENU_ITEM_TYPE(flags) = flags & (MF_STRING|MF_BITMAP|MF_OWNERDRAW|MF_SEPARATOR).
#[inline(always)]
fn MENU_ITEM_TYPE(flags: UINT) -> UINT {
    flags & (MF_STRING | MF_BITMAP | MF_OWNERDRAW | MF_SEPARATOR)
}
// RECTL_bPointInRect (rect.h inline).
#[inline(always)]
unsafe fn RECTL_bPointInRect(prcl: *const RECTL, x: INT, y: INT) -> BOOL {
    (x >= (*prcl).left && x < (*prcl).right && y >= (*prcl).top && y < (*prcl).bottom) as BOOL
}
// UlongToHandle (basetsd.h macro).
#[inline(always)]
fn UlongToHandle(ul: ULONG) -> HANDLE {
    ul as ULONG_PTR as HANDLE
}
// DesktopHeapAlloc / DesktopHeapFree / DesktopHeapReAlloc (desktop.h inlines).
#[inline(always)]
unsafe fn DesktopHeapAlloc(Desktop: PDESKTOP, Bytes: usize) -> *mut core::ffi::c_void {
    RtlAllocateHeap((*Desktop).pheapDesktop as PVOID, HEAP_NO_SERIALIZE, Bytes as SIZE_T)
}
#[inline(always)]
unsafe fn DesktopHeapFree(Desktop: PDESKTOP, lpMem: *mut core::ffi::c_void) -> u8 {
    RtlFreeHeap((*Desktop).pheapDesktop as PVOID, HEAP_NO_SERIALIZE, lpMem) as u8
}
#[inline(always)]
unsafe fn DesktopHeapReAlloc(Desktop: PDESKTOP, lpMem: *mut core::ffi::c_void, Bytes: usize) -> *mut core::ffi::c_void {
    // desktop.h inline: alloc-new + copy-old + FREE OLD (RtlReAllocateHeap isn't
    // exported). NB the heap is (*Desktop).pheapDesktop — passing the PDESKTOP
    // itself to RtlAllocateHeap walks the DESKTOP struct as HEAP metadata and
    // BUGCHECKs (0x1E in RtlAllocateHeap on the first start-menu open).
    let heap = (*Desktop).pheapDesktop as PVOID;
    let prev_size = RtlSizeHeap(heap, HEAP_NO_SERIALIZE, lpMem) as usize;
    if prev_size == Bytes {
        return lpMem;
    }
    let new = RtlAllocateHeap(heap, HEAP_NO_SERIALIZE, Bytes as SIZE_T);
    if !new.is_null() {
        let copy = core::cmp::min(prev_size, Bytes);
        core::ptr::copy_nonoverlapping(lpMem as *const u8, new as *mut u8, copy);
        RtlFreeHeap(heap, HEAP_NO_SERIALIZE, lpMem);
    }
    new
}
// UserRefObjectCo / UserDerefObjectCo (object.h static __inline; mirrors focus.rs).
#[inline(always)]
unsafe fn UserRefObjectCo(obj: PWND, UserReferenceEntry: *mut USER_REFERENCE_ENTRY) {
    let W32Thread: PTHREADINFO = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    (*UserReferenceEntry).obj = obj as PVOID;
    UserReferenceObject(obj as PVOID);
    let list_head: *mut SINGLE_LIST_ENTRY = core::ptr::addr_of_mut!((*W32Thread).ReferencesList);
    let entry: *mut SINGLE_LIST_ENTRY = core::ptr::addr_of_mut!((*UserReferenceEntry).Entry);
    (*entry).Next = (*list_head).Next;
    (*list_head).Next = entry;
}
#[inline(always)]
unsafe fn UserDerefObjectCo(obj: PWND) {
    let W32Thread: PTHREADINFO = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    let list_head: *mut SINGLE_LIST_ENTRY = core::ptr::addr_of_mut!((*W32Thread).ReferencesList);
    let reference_entry: *mut SINGLE_LIST_ENTRY = (*list_head).Next;
    if !reference_entry.is_null() {
        (*list_head).Next = (*reference_entry).Next;
    }
    UserDereferenceObject(obj as PVOID);
}
// MmCopyToCaller / MmCopyFromCaller (mmcopy.h macros → _MmCopy* cdecl).
#[inline(always)]
unsafe fn MmCopyToCaller(dst: *mut core::ffi::c_void, src: *const core::ffi::c_void, len: usize) -> NTSTATUS {
    _MmCopyToCaller(dst, src, len as UINT)
}
#[inline(always)]
unsafe fn MmCopyFromCaller(dst: *mut core::ffi::c_void, src: *const core::ffi::c_void, len: usize) -> NTSTATUS {
    _MmCopyFromCaller(dst, src, len as UINT)
}

// ProbeForRead (Mm NTAPI, stdcall `_ProbeForRead@12`) — real export.
extern "system" {
    fn ProbeForRead(Address: *const c_void, Length: SIZE_T, Alignment: ULONG);
}
// ProbeForReadUnicodeString(Ptr) — capture a UNICODE_STRING by value (probe.h).
#[inline(always)]
unsafe fn ProbeForReadUnicodeString(ptr: *const _UNICODE_STRING) -> _UNICODE_STRING {
    ProbeForRead(ptr as *const c_void, core::mem::size_of::<_UNICODE_STRING>() as SIZE_T,
                 core::mem::size_of::<usize>() as ULONG);
    *ptr
}
// ProbeAndCaptureUnicodeString (probe.h static __inline) — probe + deep-copy.
#[inline(always)]
unsafe fn ProbeAndCaptureUnicodeString(
    Dest: PUNICODE_STRING,
    CurrentMode: u8,
    UnsafeSrc: *const _UNICODE_STRING,
) -> NTSTATUS {
    if CurrentMode != 0 {
        *Dest = ProbeForReadUnicodeString(UnsafeSrc);
        if !(*Dest).Buffer.is_null() {
            if (*Dest).Length != 0 {
                ProbeForRead((*Dest).Buffer as *const c_void, (*Dest).Length as SIZE_T,
                             core::mem::size_of::<u16>() as ULONG);
                let Buffer = ExAllocatePoolWithTag(
                    PagedPool,
                    ((*Dest).Length as SIZE_T) + core::mem::size_of::<u16>() as SIZE_T,
                    u32::from_le_bytes(*b"RTSU"),
                ) as PWSTR;
                if Buffer.is_null() {
                    return STATUS_INSUFFICIENT_RESOURCES as NTSTATUS;
                }
                RtlCopyMemory(Buffer as *mut c_void, (*Dest).Buffer as *const c_void, (*Dest).Length as SIZE_T);
                *Buffer.add(((*Dest).Length / 2) as usize) = 0;
                (*Dest).Buffer = Buffer;
                if (*Dest).Length % 2 != 0 {
                    (*Dest).Length -= 1;
                }
                if (*Dest).Length as u32 >= 0xFFFE {
                    (*Dest).MaximumLength = (*Dest).Length;
                } else {
                    (*Dest).MaximumLength = (*Dest).Length + 2;
                }
            } else {
                (*Dest).MaximumLength = 0;
                (*Dest).Buffer = core::ptr::null_mut();
            }
        } else {
            (*Dest).Length = 0;
            (*Dest).MaximumLength = 0;
        }
    } else {
        *Dest = *UnsafeSrc;
    }
    STATUS_SUCCESS as NTSTATUS
}
// ReleaseCapturedUnicodeString (probe.h) — free a user-mode captured buffer.
#[inline(always)]
unsafe fn ReleaseCapturedUnicodeString(CapturedString: PUNICODE_STRING, CurrentMode: u8) {
    if CurrentMode != 0 && !(*CapturedString).Buffer.is_null() {
        ExFreePoolWithTag((*CapturedString).Buffer as *mut c_void, u32::from_le_bytes(*b"RTSU"));
    }
}

// ---- file-scope preamble (translated) ----
pub static mut ghMenuFont: HFONT = core::ptr::null_mut();
pub static mut ghMenuFontBold: HFONT = core::ptr::null_mut();
static mut MenuCharSize: SIZE = SIZE { cx: 0, cy: 0 };
static mut top_popup: HWND = core::ptr::null_mut();
static mut top_popup_hmenu: HMENU = core::ptr::null_mut();
pub static mut fInsideMenuLoop: BOOL = 0;
pub static mut fInEndMenu: BOOL = 0;

pub const MM_SETMENUHANDLE: UINT = WM_USER + 0;
pub const MM_GETMENUHANDLE: UINT = WM_USER + 1;

pub const TF_ENDMENU: UINT = 0x10000;
pub const TF_SUSPENDPOPUP: UINT = 0x20000;
pub const TF_SKIPREMOVE: UINT = 0x40000;

pub const MAXMENUDEPTH: UINT = 30;

pub const MNS_STYLE_MASK: UINT =
    MNS_NOCHECK | MNS_MODELESS | MNS_DRAGDROP | MNS_AUTODISMISS | MNS_NOTIFYBYPOS | MNS_CHECKORBMP;

pub const MENUITEMINFO_TYPE_MASK: UINT = MFT_STRING
    | MFT_BITMAP
    | MFT_OWNERDRAW
    | MFT_SEPARATOR
    | MFT_MENUBARBREAK
    | MFT_MENUBREAK
    | MFT_RADIOCHECK
    | MFT_RIGHTORDER
    | MFT_RIGHTJUSTIFY;

pub const TYPE_MASK: UINT = MENUITEMINFO_TYPE_MASK | MF_POPUP | MF_SYSMENU;

pub const STATE_MASK: UINT = !TYPE_MASK;

pub const MENUITEMINFO_STATE_MASK: UINT = STATE_MASK & !(MF_BYPOSITION | MF_MOUSESELECT);

pub const MII_STATE_MASK: UINT = MFS_GRAYED | MFS_CHECKED | MFS_HILITE | MFS_DEFAULT;

pub const MAX_MENU_ITEMS: UINT = 0x4000;
pub const MAX_GOINTOSUBMENU: UINT = 0x10;
pub const MENU_COL_SPACE: INT = 4;
pub const MENU_ITEM_HBMP_SPACE: INT = 5;
pub const MENU_BAR_ITEMS_SPACE: INT = 12;
pub const SEPARATOR_HEIGHT: INT = 5;
pub const MENU_TAB_SPACE: INT = 8;

#[repr(C)]
pub struct MTRACKER {
    pub TrackFlags: UINT,
    pub CurrentMenu: PMENU,
    pub TopMenu: PMENU,
    pub OwnerWnd: PWND,
    pub Pt: POINT,
}

pub const TPM_INTERNAL: UINT = 0xF0000000;
pub const TPM_BUTTONDOWN: UINT = 0x40000000;
pub const TPM_POPUPMENU: UINT = 0x20000000;

pub const ITEM_PREV: INT = -1;
pub const ITEM_NEXT: INT = 1;

pub const SHOW_DEBUGRECT: INT = 0;

pub const VALID_TPM_FLAGS: UINT = TPM_LAYOUTRTL
    | TPM_NOANIMATION
    | TPM_VERNEGANIMATION
    | TPM_VERPOSANIMATION
    | TPM_HORNEGANIMATION
    | TPM_HORPOSANIMATION
    | TPM_RETURNCMD
    | TPM_NONOTIFY
    | TPM_VERTICAL
    | TPM_BOTTOMALIGN
    | TPM_VCENTERALIGN
    | TPM_RIGHTALIGN
    | TPM_CENTERALIGN
    | TPM_RIGHTBUTTON
    | TPM_RECURSE;


// GetNCHitEx — FASTCALL (nonclient.rs `@GetNCHitEx@12`). i386 fastcall, cdecl else.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn GetNCHitEx(pWnd: PWND, pt: POINT) -> INT;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn GetNCHitEx(pWnd: PWND, pt: POINT) -> INT;
}

#[cfg(target_arch = "x86")]
#[no_mangle]
unsafe extern "fastcall" fn IntGetMenuObject(hMenu: HMENU) -> PMENU {
    let Menu: PMENU = UserGetMenuObject(hMenu);
    if !Menu.is_null() {
        (*(Menu as *mut _HEAD)).cLockObj += 1;
    }
    Menu
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
unsafe extern "C" fn IntGetMenuObject(hMenu: HMENU) -> PMENU {
    let Menu: PMENU = UserGetMenuObject(hMenu);
    if !Menu.is_null() {
        (*(Menu as *mut _HEAD)).cLockObj += 1;
    }
    Menu
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn VerifyMenu(pMenu: PMENU) -> PMENU {
    VerifyMenu_impl(pMenu)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn VerifyMenu(pMenu: PMENU) -> PMENU {
    VerifyMenu_impl(pMenu)
}

unsafe fn VerifyMenu_impl(pMenu: PMENU) -> PMENU {
    let _hMenu: HMENU;
    let pItem: PITEM;
    let Error: ULONG;
    let mut i: UINT;

    if pMenu.is_null() {
        return core::ptr::null_mut();
    }

    Error = EngGetLastError();

    _hMenu = UserHMGetHandle(pMenu) as HMENU ;
    pItem = (*pMenu).rgItems;
    if !pItem.is_null() {
        // ---- dropped _SEH2_TRY/_SEH2_EXCEPT recovered here ----
        // C (menu.c:187-228, VerifyMenu): the guarded region is a plain
        // self-touch of `pMenu->rgItems[0].wID` -- a corruption probe on a
        // KERNEL structure, not a user-mode pointer, so there is no
        // ProbeFor* call to restore here (unlike the syscall sites in this
        // pair). It exists only to detect a bad `rgItems` pointer/page via
        // the fault itself, which is exactly what `safe_copy_raw` (no
        // probing, just a fault-tolerant copy) exists for.
        //
        // Handler shape is the ODD one out of this file's two sites -- NOT
        // "assign Status and fall through". It is:
        //   ERR("Run away LOOP!\n"); EngSetLastError(Error); _SEH2_YIELD(return NULL);
        // an early return of a POINTER (not an NTSTATUS), restoring `Error`
        // (the value read BEFORE the touch, not a fresh EngGetLastError())
        // before returning. Replicated exactly: no fall-through here.
        i = 0;
        if let Err(_st) = ntkrnl_sys::probe::safe_copy_raw(
            core::ptr::addr_of_mut!(i) as *mut c_void,
            core::ptr::addr_of!((*pItem).wID) as *const c_void,
            core::mem::size_of::<UINT>(),
        ) {
            ERR!("Run away LOOP!\n");
            EngSetLastError(Error);
            return core::ptr::null_mut();
        }
        if let Err(_st) = ntkrnl_sys::probe::safe_copy_raw(
            core::ptr::addr_of_mut!((*pItem).wID) as *mut c_void,
            core::ptr::addr_of!(i) as *const c_void,
            core::mem::size_of::<UINT>(),
        ) {
            ERR!("Run away LOOP!\n");
            EngSetLastError(Error);
            return core::ptr::null_mut();
        }
    }

    EngSetLastError(Error);
    pMenu
}

#[cfg(target_arch = "x86")]
#[no_mangle]
unsafe extern "fastcall" fn IntIsMenu(Menu: HMENU) -> u8 {
    if !UserGetMenuObject(Menu).is_null() {
        return 1u8;
    }
    0u8
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
unsafe extern "C" fn IntIsMenu(Menu: HMENU) -> u8 {
    if !UserGetMenuObject(Menu).is_null() {
        return 1u8;
    }
    0u8
}

unsafe fn IntGetMenu(hWnd: HWND) -> PMENU {
    let Wnd: PWND = ValidateHwndNoErr(hWnd);

    if Wnd.is_null() {
        return core::ptr::null_mut();
    }

    UserGetMenuObject(UlongToHandle((*Wnd).IDMenu as u32) as HMENU)
}

unsafe fn get_win_sys_menu(hwnd: HWND) -> PMENU {
    let mut ret: PMENU = core::ptr::null_mut();
    let win: *mut WND = ValidateHwndNoErr(hwnd);
    if !win.is_null() {
        ret = UserGetMenuObject((*win).SystemMenu);
    }
    ret
}

unsafe fn IntDestroyMenu(pMenu: PMENU, bRecurse: BOOL) -> BOOL {
    let SubMenu: PMENU;

    if !(*pMenu).rgItems.is_null() {
        let mut item = (*pMenu).rgItems;
        let mut i = (*pMenu).cItems;
        while i > 0 {
            let SubMenu = (*item).spSubMenu;
            (*item).spSubMenu = core::ptr::null_mut();

            /* Remove Item Text */
            if (((*item).fType & MENUITEMINFO_TYPE_MASK) == MF_STRING)
                && (*item).lpstr.Length != 0
            {
                DesktopHeapFree((*pMenu).head.rpdesk, (*item).lpstr.Buffer as *mut _);
            }

            /* Remove Item Bitmap and set it for this process */
            if !(*item).hbmp.is_null() && ((*item).fState & MFS_HBMMENUBMP) == 0 {
                GreSetObjectOwner((*item).hbmp as *mut core::ffi::c_void , GDI_OBJ_HMGR_POWNED);
                (*item).hbmp = core::ptr::null_mut();
            }

            /* Remove Item submenu */
            if bRecurse != 0 && !SubMenu.is_null() {
                IntReleaseMenuObject(SubMenu);
                IntDestroyMenuObject(SubMenu, bRecurse);
            }

            item = item.add(1);
            i -= 1;
        }
        /* Free the Item */
        DesktopHeapFree((*pMenu).head.rpdesk, (*pMenu).rgItems as *mut _);
        (*pMenu).rgItems = core::ptr::null_mut();
        (*pMenu).cItems = 0;
    }
    let _ = SubMenu;
    1
}

#[no_mangle]
unsafe extern "C" fn UserDestroyMenuObject(Object: *mut core::ffi::c_void) -> u8 {
    IntDestroyMenuObject(Object as PMENU, 1u8 as _) as u8
}

#[cfg(target_arch = "x86")]
#[no_mangle]
unsafe extern "fastcall" fn IntDestroyMenuObject(Menu: PMENU, bRecurse: BOOL) -> BOOL {
    IntDestroyMenuObject_impl(Menu, bRecurse)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
unsafe extern "C" fn IntDestroyMenuObject(Menu: PMENU, bRecurse: BOOL) -> BOOL {
    IntDestroyMenuObject_impl(Menu, bRecurse)
}

unsafe fn IntDestroyMenuObject_impl(Menu: PMENU, bRecurse: BOOL) -> BOOL {
    if !Menu.is_null() {
        let Window: PWND;

        if PsGetCurrentProcessSessionId()
            == (*(*(*Menu).head.rpdesk).rpwinstaParent).dwSessionId
        {
            let ret: BOOL;
            if !(*Menu).hWnd.is_null() {
                Window = ValidateHwndNoErr((*Menu).hWnd);
                if !Window.is_null() {
                    if ((*Menu).fFlags & (MNF_POPUP | MNF_SYSSUBMENU)) == MNF_POPUP {
                        // ERR!("FIXME Pop up menu window thing'ie\n");
                    }
                }
            }

            if UserMarkObjectDestroy(Menu as *mut _) == 0 {
                return 1;
            }

            IntDestroyMenu(Menu, bRecurse);

            ret = UserDeleteObject(UserHMGetHandle(Menu), TYPE_MENU);
            return ret;
        }
    }
    0
}

#[no_mangle]
unsafe extern "C" fn MenuInit() -> BOOL {
    let mut ncm: NONCLIENTMETRICSW = core::mem::zeroed();

    if ghMenuFont.is_null() || ghMenuFontBold.is_null() {
        ncm.cbSize = core::mem::size_of::<NONCLIENTMETRICSW>() as u32;
        if UserSystemParametersInfo(
            SPI_GETNONCLIENTMETRICS,
            core::mem::size_of::<NONCLIENTMETRICSW>() as u32,
            core::ptr::addr_of_mut!(ncm) as *mut core::ffi::c_void,
            0,
        ) == 0
        {
            return 0;
        }

        ghMenuFont = GreCreateFontIndirectW(core::ptr::addr_of_mut!(ncm.lfMenuFont));
        if ghMenuFont.is_null() {
            return 0;
        }

        let new_weight = ncm.lfMenuFont.lfWeight + (FW_BOLD as i32 - FW_NORMAL as i32);
        ncm.lfMenuFont.lfWeight = core::cmp::min(new_weight, FW_HEAVY as i32);

        ghMenuFontBold = GreCreateFontIndirectW(core::ptr::addr_of_mut!(ncm.lfMenuFont));
        if ghMenuFontBold.is_null() {
            GreDeleteObject(ghMenuFont as *mut core::ffi::c_void );
            ghMenuFont = core::ptr::null_mut();
            return 0;
        }

        GreSetObjectOwner(ghMenuFont as *mut core::ffi::c_void , GDI_OBJ_HMGR_PUBLIC);
        GreSetObjectOwner(ghMenuFontBold as *mut core::ffi::c_void , GDI_OBJ_HMGR_PUBLIC);

        DbgPrint(b"MenuInit: skipping co_IntSetupOBM (DUMMY_PSEH)\n\0".as_ptr() as *const i8);
    }

    1
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn MENU_depth(pmenu: PMENU, mut depth: core::ffi::c_int) -> core::ffi::c_int {
    if pmenu.is_null() {
        return depth;
    }

    depth += 1;
    if depth > MAXMENUDEPTH as core::ffi::c_int {
        return depth;
    }

    let mut item: *mut ITEM = (*pmenu).rgItems;
    let mut subdepth = depth;
    let mut i: UINT = 0;
    while i < (*pmenu).cItems && subdepth <= MAXMENUDEPTH as core::ffi::c_int {
        if !(*item).spSubMenu.is_null() {
            let bdepth = MENU_depth((*item).spSubMenu, depth);
            if bdepth > subdepth {
                subdepth = bdepth;
            }
        }
        if subdepth > MAXMENUDEPTH as core::ffi::c_int {
            TRACE!(b"<- hmenu %p\n\0".as_ptr() as *const u8, (*item).spSubMenu);
        }
        i += 1;
        item = item.add(1);
    }
    subdepth
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn MENU_depth(pmenu: PMENU, mut depth: core::ffi::c_int) -> core::ffi::c_int {
    if pmenu.is_null() {
        return depth;
    }

    depth += 1;
    if depth > MAXMENUDEPTH as core::ffi::c_int {
        return depth;
    }

    let mut item: *mut ITEM = (*pmenu).rgItems;
    let mut subdepth = depth;
    let mut i: UINT = 0;
    while i < (*pmenu).cItems && subdepth <= MAXMENUDEPTH as core::ffi::c_int {
        if !(*item).spSubMenu.is_null() {
            let bdepth = MENU_depth((*item).spSubMenu, depth);
            if bdepth > subdepth {
                subdepth = bdepth;
            }
        }
        if subdepth > MAXMENUDEPTH as core::ffi::c_int {
            TRACE!(b"<- hmenu %p\n\0".as_ptr() as *const u8, (*item).spSubMenu);
        }
        i += 1;
        item = item.add(1);
    }
    subdepth
}

unsafe fn MENU_GetStartOfNextColumn(menu: PMENU) -> UINT {
    let pItem: PITEM;
    let mut i: UINT;

    if menu.is_null() {
        return NO_SELECTED_ITEM as UINT;
    }

    i = ((*menu).iItem + 1) as UINT;
    if i == NO_SELECTED_ITEM as UINT {
        return i;
    }

    pItem = (*menu).rgItems;
    if pItem.is_null() {
        return NO_SELECTED_ITEM as UINT;
    }

    while i < (*menu).cItems as UINT {
        if (*pItem.offset(i as isize)).fType & (MF_MENUBREAK | MF_MENUBARBREAK) != 0 {
            return i;
        }
        i += 1;
    }

    NO_SELECTED_ITEM as UINT
}

unsafe fn MENU_GetStartOfPrevColumn(menu: PMENU) -> UINT {
    let mut i: UINT;
    let pItem: PITEM;

    if menu.is_null() {
        return NO_SELECTED_ITEM;
    }

    if (*menu).iItem == 0 || (*menu).iItem == (NO_SELECTED_ITEM as i32) {
        return NO_SELECTED_ITEM;
    }

    pItem = (*menu).rgItems;
    if pItem.is_null() {
        return NO_SELECTED_ITEM;
    }

    i = (*menu).iItem as u32 ;
    while i != 0 && ((*pItem.add(i as usize)).fType & (MF_MENUBREAK | MF_MENUBARBREAK)) == 0 {
        i -= 1;
    }

    if i == 0 {
        return NO_SELECTED_ITEM;
    }

    i -= 1;
    while i != 0 {
        if ((*pItem.add(i as usize)).fType & (MF_MENUBREAK | MF_MENUBARBREAK)) != 0 {
            break;
        }
        i -= 1;
    }

    i
}

fastcall_fn! {
#[no_mangle]
unsafe fn MENU_FindItem(pmenu: *mut PMENU, nPos: *mut UINT, wFlags: UINT) -> PITEM {
    let menu: PMENU = *pmenu;
    let mut fallback: PITEM = core::ptr::null_mut();
    let mut fallback_pos: UINT = 0;
    let mut i: UINT;

    if menu.is_null() {
        return core::ptr::null_mut();
    }

    if (wFlags & MF_BYPOSITION) != 0 {
        if (*menu).cItems == 0 {
            return core::ptr::null_mut();
        }
        if *nPos >= (*menu).cItems as UINT {
            return core::ptr::null_mut();
        }
        return (*menu).rgItems.offset(*nPos as isize);
    } else {
        let mut item: PITEM = (*menu).rgItems;
        i = 0;
        while i < (*menu).cItems as UINT {
            if !(*item).spSubMenu.is_null() {
                let mut psubmenu: PMENU = (*item).spSubMenu;
                let subitem = MENU_FindItem(core::ptr::addr_of_mut!(psubmenu), nPos, wFlags);
                if !subitem.is_null() {
                    *pmenu = psubmenu;
                    return subitem;
                } else if (*item).wID as UINT == *nPos {
                    fallback_pos = i;
                    fallback = item;
                }
            } else if (*item).wID as UINT == *nPos {
                *nPos = i;
                return item;
            }
            i += 1;
            item = item.offset(1);
        }
    }

    if !fallback.is_null() {
        *nPos = fallback_pos;
    }

    fallback
}
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn MENU_FindSubMenu(menu: *mut PMENU, SubTarget: PMENU) -> UINT {
    MENU_FindSubMenu_impl(menu, SubTarget)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn MENU_FindSubMenu(menu: *mut PMENU, SubTarget: PMENU) -> UINT {
    MENU_FindSubMenu_impl(menu, SubTarget)
}

unsafe fn MENU_FindSubMenu_impl(menu: *mut PMENU, SubTarget: PMENU) -> UINT {
    let mut i: UINT;
    let mut item: PITEM;

    item = (*(*menu)).rgItems;
    i = 0;
    while i < (*(*menu)).cItems {
        if (*item).spSubMenu.is_null() {
            i += 1;
            item = item.add(1);
            continue;
        } else {
            if (*item).spSubMenu == SubTarget {
                return i;
            } else {
                let mut pSubMenu: PMENU = (*item).spSubMenu;
                let pos = MENU_FindSubMenu(core::ptr::addr_of_mut!(pSubMenu), SubTarget);
                if pos != NO_SELECTED_ITEM as UINT {
                    *menu = pSubMenu;
                    return pos;
                }
            }
        }
        i += 1;
        item = item.add(1);
    }
    NO_SELECTED_ITEM as UINT
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntRemoveMenuItem(
    pMenu: PMENU,
    mut nPos: UINT,
    wFlags: UINT,
    bRecurse: BOOL,
) -> BOOL {
    IntRemoveMenuItem_impl(pMenu, nPos, wFlags, bRecurse)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntRemoveMenuItem(
    pMenu: PMENU,
    nPos: UINT,
    wFlags: UINT,
    bRecurse: BOOL,
) -> BOOL {
    IntRemoveMenuItem_impl(pMenu, nPos, wFlags, bRecurse)
}

unsafe fn IntRemoveMenuItem_impl(
    mut pMenu: PMENU,
    mut nPos: UINT,
    wFlags: UINT,
    bRecurse: BOOL,
) -> BOOL {
    let mut item: PITEM;
    let newItems: PITEM;

    item = MENU_FindItem(core::ptr::addr_of_mut!(pMenu), core::ptr::addr_of_mut!(nPos), wFlags);
    if item.is_null() {
        return 0;
    }

    /* Remove item */

    if (MENU_ITEM_TYPE((*item).fType) == MF_STRING) && (*item).lpstr.Length != 0 {
        DesktopHeapFree((*pMenu).head.rpdesk, (*item).lpstr.Buffer as *mut _);
    }

    if bRecurse != 0 && !(*item).spSubMenu.is_null() {
        IntDestroyMenuObject((*item).spSubMenu, bRecurse);
    }

    (*pMenu).cItems -= 1;
    if (*pMenu).cItems == 0 {
        DesktopHeapFree((*pMenu).head.rpdesk, (*pMenu).rgItems as *mut _);
        (*pMenu).rgItems = core::ptr::null_mut();
    } else {
        while nPos < (*pMenu).cItems {
            *item = *(item.add(1));
            item = item.add(1);
            nPos += 1;
        }
        newItems = DesktopHeapReAlloc(
            (*pMenu).head.rpdesk,
            (*pMenu).rgItems as *mut _,
            (*pMenu).cItems as usize * core::mem::size_of::<ITEM>(),
        ) as PITEM;
        if !newItems.is_null() {
            (*pMenu).rgItems = newItems;
        }
    }
    1
}

unsafe extern "C" fn MENU_InsertItem(mut menu: PMENU, mut pos: UINT, flags: UINT, submenu: *mut PMENU, npos: *mut UINT) -> *mut ITEM {
    let newItems: *mut ITEM;

    if flags & MF_BYPOSITION != 0 {
        if pos > (*menu).cItems {
            pos = (*menu).cItems;
        }
    } else {
        if MENU_FindItem(core::ptr::addr_of_mut!(menu), core::ptr::addr_of_mut!(pos), flags).is_null() {
            if !submenu.is_null() {
                *submenu = menu;
            }
            if !npos.is_null() {
                *npos = pos;
            }
            pos = (*menu).cItems;
        }
    }

    while pos > 0
        && (*(*menu).rgItems.offset((pos - 1) as isize)).hbmp as INT_PTR >= 1 as INT_PTR
        && (*(*menu).rgItems.offset((pos - 1) as isize)).hbmp as INT_PTR <= 11 as INT_PTR
    {
        pos -= 1;
    }

    newItems = DesktopHeapAlloc(
        (*menu).head.rpdesk,
        core::mem::size_of::<ITEM>() * ((*menu).cItems + 1) as usize,
    ) as *mut ITEM;
    if newItems.is_null() {
        return core::ptr::null_mut();
    }
    if (*menu).cItems > 0 {
        if pos > 0 {
            RtlCopyMemory(
                newItems as *mut _,
                (*menu).rgItems as *const _,
                (pos as usize * core::mem::size_of::<ITEM>()) as SIZE_T,
            );
        }
        if pos < (*menu).cItems {
            RtlCopyMemory(
                newItems.offset((pos + 1) as isize) as *mut _,
                (*menu).rgItems.offset(pos as isize) as *const _,
                (((*menu).cItems - pos) as usize * core::mem::size_of::<ITEM>()) as SIZE_T,
            );
        }
        DesktopHeapFree((*menu).head.rpdesk, (*menu).rgItems as *mut _);
    }
    (*menu).rgItems = newItems;
    (*menu).cItems += 1;
    RtlZeroMemory(newItems.offset(pos as isize) as *mut _, core::mem::size_of::<ITEM>() as SIZE_T);
    (*menu).cyMenu = 0;
    newItems.offset(pos as isize)
}

fastcall_fn! {
#[no_mangle]
unsafe fn IntInsertMenuItem(
    MenuObject: PMENU,
    mut uItem: UINT,
    fByPosition: BOOL,
    ItemInfo: PROSMENUITEMINFO,
    lpstr: PUNICODE_STRING,
) -> BOOL {
    let MenuItem: PITEM;
    let mut SubMenu: PMENU = core::ptr::null_mut();

    if MAX_MENU_ITEMS <= (*MenuObject).cItems as u32 {
        EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }

    SubMenu = MenuObject;

    MenuItem = MENU_InsertItem(
        SubMenu,
        uItem,
        if fByPosition != 0 { MF_BYPOSITION } else { MF_BYCOMMAND },
        &mut SubMenu,
        &mut uItem,
    );
    if MenuItem.is_null() {
        return 0;
    }

    if IntSetMenuItemInfo(SubMenu, MenuItem, ItemInfo, lpstr) == 0 {
        IntRemoveMenuItem(
            SubMenu,
            uItem,
            if fByPosition != 0 { MF_BYPOSITION } else { MF_BYCOMMAND },
            0,
        );
        return 0;
    }

    (*SubMenu).cyMenu = 0;
    (*MenuItem).hbmpUnchecked = 0 as HBITMAP as *mut core::ffi::c_void ;
    (*MenuItem).hbmpChecked = (*MenuItem).hbmpUnchecked;

    1
}
}

fastcall_fn! {
unsafe fn IntCreateMenu(
    Handle: PHANDLE,
    IsMenuBar: BOOL,
    Desktop: PDESKTOP,
    ppi: PPROCESSINFO,
) -> PMENU {
    let Menu: PMENU = UserCreateObject(
        gHandleTable,
        Desktop,
        (*ppi).ptiList,
        Handle,
        TYPE_MENU,
        core::mem::size_of::<MENU>() as u32,
    ) as PMENU;

    if Menu.is_null() {
        *Handle = core::ptr::null_mut();
        return core::ptr::null_mut();
    }

    (*Menu).cyMax = 0;
    (*Menu).hbrBack = core::ptr::null_mut();
    (*Menu).dwContextHelpId = 0;
    (*Menu).dwMenuData = 0;
    (*Menu).iItem = NO_SELECTED_ITEM as i32 ;
    (*Menu).fFlags = if IsMenuBar != 0 { 0 } else { MNF_POPUP } as u32;
    (*Menu).spwndNotify = core::ptr::null_mut();
    (*Menu).cyMenu = 0;
    (*Menu).cxMenu = 0;
    (*Menu).cItems = 0;
    (*Menu).iTop = 0;
    (*Menu).iMaxTop = 0;
    (*Menu).cxTextAlign = 0;
    (*Menu).rgItems = core::ptr::null_mut();

    (*Menu).hWnd = core::ptr::null_mut();
    (*Menu).TimeToHide = 0u8 as i32 ;

    Menu
}
}

#[cfg(target_arch = "x86")]
pub unsafe extern "fastcall" fn IntCloneMenuItems(Destination: PMENU, Source: PMENU) -> i32 {
    IntCloneMenuItems_impl(Destination, Source)
}

#[cfg(not(target_arch = "x86"))]
pub unsafe extern "C" fn IntCloneMenuItems(Destination: PMENU, Source: PMENU) -> i32 {
    IntCloneMenuItems_impl(Destination, Source)
}

unsafe fn IntCloneMenuItems_impl(Destination: PMENU, Source: PMENU) -> i32 {

    let mut MenuItem: PITEM;
    let mut NewMenuItem: PITEM;
    let mut i: UINT;

    if (*Source).cItems == 0 {
        return 0;
    }

    NewMenuItem = DesktopHeapAlloc(
        (*Source).head.rpdesk,
        (*Source).cItems as usize * core::mem::size_of::<ITEM>(),
    ) as PITEM;
    if NewMenuItem.is_null() {
        return 0;
    }

    core::ptr::write_bytes(
        NewMenuItem as *mut u8,
        0,
        (*Source).cItems as usize * core::mem::size_of::<ITEM>(),
    );

    (*Destination).rgItems = NewMenuItem;

    MenuItem = (*Source).rgItems;
    i = 0;
    while i < (*Source).cItems {
        (*NewMenuItem).fType = (*MenuItem).fType;
        (*NewMenuItem).fState = (*MenuItem).fState;
        (*NewMenuItem).wID = (*MenuItem).wID;
        (*NewMenuItem).spSubMenu = (*MenuItem).spSubMenu;
        (*NewMenuItem).hbmpChecked = (*MenuItem).hbmpChecked;
        (*NewMenuItem).hbmpUnchecked = (*MenuItem).hbmpUnchecked;
        (*NewMenuItem).dwItemData = (*MenuItem).dwItemData;
        if (*MenuItem).lpstr.Length != 0 {
            (*NewMenuItem).lpstr.Length = 0;
            (*NewMenuItem).lpstr.MaximumLength = (*MenuItem).lpstr.MaximumLength;
            (*NewMenuItem).lpstr.Buffer = DesktopHeapAlloc(
                (*Destination).head.rpdesk,
                (*MenuItem).lpstr.MaximumLength as usize,
            ) as *mut u16;
            if (*NewMenuItem).lpstr.Buffer.is_null() {
                DesktopHeapFree((*Destination).head.rpdesk, NewMenuItem as *mut core::ffi::c_void);
                break;
            }
            RtlCopyUnicodeString(
                core::ptr::addr_of_mut!((*NewMenuItem).lpstr).cast(),
                core::ptr::addr_of_mut!((*MenuItem).lpstr).cast(),
            );
            *(*NewMenuItem)
                .lpstr
                .Buffer
                .add((*MenuItem).lpstr.Length as usize / core::mem::size_of::<u16>()) = 0;
            (*NewMenuItem).Xlpstr = (*NewMenuItem).lpstr.Buffer;
        } else {
            (*NewMenuItem).lpstr.Buffer = (*MenuItem).lpstr.Buffer;
            (*NewMenuItem).Xlpstr = (*NewMenuItem).lpstr.Buffer;
        }
        (*NewMenuItem).hbmp = (*MenuItem).hbmp;
        (*Destination).cItems = i + 1;

        i += 1;
        MenuItem = MenuItem.add(1);
        NewMenuItem = NewMenuItem.add(1);
    }
    1
}

fastcall_fn! {
#[no_mangle]
pub unsafe fn IntCloneMenu(Source: PMENU) -> PMENU {
    let mut hMenu: HANDLE = core::ptr::null_mut();
    let Menu: PMENU;

    if Source.is_null() {
        return core::ptr::null_mut();
    }

    Menu = UserCreateObject(
        gHandleTable,
        (*Source).head.rpdesk,
        (*((*Source).head.hTaskWow as PPROCESSINFO)).ptiList,
        &mut hMenu,
        TYPE_MENU as u8 as u32 ,
        core::mem::size_of::<MENU>() as u32,
    ) as PMENU;

    if Menu.is_null() {
        return core::ptr::null_mut();
    }

    (*Menu).fFlags = (*Source).fFlags;
    (*Menu).cyMax = (*Source).cyMax;
    (*Menu).hbrBack = (*Source).hbrBack;
    (*Menu).dwContextHelpId = (*Source).dwContextHelpId;
    (*Menu).dwMenuData = (*Source).dwMenuData;
    (*Menu).iItem = NO_SELECTED_ITEM as i32 ;
    (*Menu).spwndNotify = core::ptr::null_mut();
    (*Menu).cyMenu = 0;
    (*Menu).cxMenu = 0;
    (*Menu).cItems = 0;
    (*Menu).iTop = 0;
    (*Menu).iMaxTop = 0;
    (*Menu).cxTextAlign = 0;
    (*Menu).rgItems = core::ptr::null_mut();

    (*Menu).hWnd = core::ptr::null_mut();
    (*Menu).TimeToHide = 0u8 as i32 ;

    IntCloneMenuItems(Menu, Source);

    Menu
}
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntSetMenuFlagRtoL(Menu: PMENU) -> BOOL {
    (*Menu).fFlags |= MNF_RTOL;
    1
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntSetMenuFlagRtoL(Menu: PMENU) -> BOOL {
    (*Menu).fFlags |= MNF_RTOL;
    1
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntSetMenuContextHelpId(Menu: PMENU, dwContextHelpId: u32) -> i32 {
    (*Menu).dwContextHelpId = dwContextHelpId;
    1
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntSetMenuContextHelpId(Menu: PMENU, dwContextHelpId: u32) -> i32 {
    (*Menu).dwContextHelpId = dwContextHelpId;
    1
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntGetMenuInfo(Menu: PMENU, lpmi: PROSMENUINFO) -> bool {
    IntGetMenuInfo_impl(Menu, lpmi)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntGetMenuInfo(Menu: PMENU, lpmi: PROSMENUINFO) -> bool {
    IntGetMenuInfo_impl(Menu, lpmi)
}

unsafe fn IntGetMenuInfo_impl(Menu: PMENU, lpmi: PROSMENUINFO) -> bool {
    if (*lpmi).fMask & MIM_BACKGROUND != 0 {
        (*lpmi).hbrBack = (*Menu).hbrBack;
    }
    if (*lpmi).fMask & MIM_HELPID != 0 {
        (*lpmi).dwContextHelpID = (*Menu).dwContextHelpId;
    }
    if (*lpmi).fMask & MIM_MAXHEIGHT != 0 {
        (*lpmi).cyMax = (*Menu).cyMax;
    }
    if (*lpmi).fMask & MIM_MENUDATA != 0 {
        (*lpmi).dwMenuData = (*Menu).dwMenuData;
    }
    if (*lpmi).fMask & MIM_STYLE != 0 {
        (*lpmi).dwStyle = (*Menu).fFlags & MNS_STYLE_MASK;
    }

    if (core::mem::size_of::<MENUINFO>() as u32) < (*lpmi).cbSize {
        (*lpmi).cItems = (*Menu).cItems;

        (*lpmi).iItem = (*Menu).iItem as u32 ;
        (*lpmi).cxMenu = (*Menu).cxMenu as u16 ;
        (*lpmi).cyMenu = (*Menu).cyMenu as u16 ;
        (*lpmi).spwndNotify = (*Menu).spwndNotify;
        (*lpmi).cxTextAlign = (*Menu).cxTextAlign;
        (*lpmi).iTop = (*Menu).iTop;
        (*lpmi).iMaxTop = (*Menu).iMaxTop;
        (*lpmi).set_dwArrowsOn((*Menu).dwArrowsOn());

        (*lpmi).fFlags = (*Menu).fFlags;
        (*lpmi).Self_ = UserHMGetHandle(Menu) as HMENU;
        (*lpmi).TimeToHide = (*Menu).TimeToHide;
        (*lpmi).Wnd = (*Menu).hWnd;
    }
    true
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntSetMenuInfo(Menu: PMENU, lpmi: PROSMENUINFO) -> u8 {
    IntSetMenuInfoImpl(Menu, lpmi)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntSetMenuInfo(Menu: PMENU, lpmi: PROSMENUINFO) -> u8 {
    IntSetMenuInfoImpl(Menu, lpmi)
}

unsafe fn IntSetMenuInfoImpl(Menu: PMENU, lpmi: PROSMENUINFO) -> u8 {
    if (*lpmi).fMask & MIM_BACKGROUND != 0 {
        (*Menu).hbrBack = (*lpmi).hbrBack;
    }
    if (*lpmi).fMask & MIM_HELPID != 0 {
        (*Menu).dwContextHelpId = (*lpmi).dwContextHelpID;
    }
    if (*lpmi).fMask & MIM_MAXHEIGHT != 0 {
        (*Menu).cyMax = (*lpmi).cyMax;
    }
    if (*lpmi).fMask & MIM_MENUDATA != 0 {
        (*Menu).dwMenuData = (*lpmi).dwMenuData;
    }
    if (*lpmi).fMask & MIM_STYLE != 0 {
        (*Menu).fFlags ^= ((*Menu).fFlags ^ (*lpmi).dwStyle) & MNS_STYLE_MASK;
    }
    if (*lpmi).fMask & MIM_APPLYTOSUBMENUS != 0 {
        let mut item: PITEM = (*Menu).rgItems;
        let mut i = (*Menu).cItems;
        while i != 0 {
            if !(*item).spSubMenu.is_null() {
                IntSetMenuInfo((*item).spSubMenu, lpmi);
            }
            i -= 1;
            item = item.add(1);
        }
    }
    if (core::mem::size_of::<MENUINFO>() as u32) < (*lpmi).cbSize {
        (*Menu).iItem = (*lpmi).iItem as i32 ;
        (*Menu).cyMenu = (*lpmi).cyMenu as u32 ;
        (*Menu).cxMenu = (*lpmi).cxMenu as u32 ;
        (*Menu).spwndNotify = (*lpmi).spwndNotify;
        (*Menu).cxTextAlign = (*lpmi).cxTextAlign;
        (*Menu).iTop = (*lpmi).iTop;
        (*Menu).iMaxTop = (*lpmi).iMaxTop;
        (*Menu).set_dwArrowsOn((*lpmi).dwArrowsOn());

        (*Menu).TimeToHide = (*lpmi).TimeToHide;
        (*Menu).hWnd = (*lpmi).Wnd;
    }
    if (*lpmi).fMask & MIM_STYLE != 0 {
        if (*lpmi).dwStyle & MNS_AUTODISMISS != 0 {
            FIXME!(b"MNS_AUTODISMISS unimplemented wine\n\0".as_ptr() as _);
        }
        if (*lpmi).dwStyle & MNS_DRAGDROP != 0 {
            FIXME!(b"MNS_DRAGDROP unimplemented wine\n\0".as_ptr() as _);
        }
        if (*lpmi).dwStyle & MNS_MODELESS != 0 {
            FIXME!(b"MNS_MODELESS unimplemented wine\n\0".as_ptr() as _);
        }
    }
    1u8
}

#[cfg(target_pointer_width = "32")]
unsafe extern "fastcall" fn IntGetMenuItemInfo(
    _Menu: PMENU,
    MenuItem: PITEM,
    lpmii: PROSMENUITEMINFO,
) -> BOOL {
    IntGetMenuItemInfo_impl(_Menu, MenuItem, lpmii)
}

#[cfg(not(target_pointer_width = "32"))]
unsafe extern "C" fn IntGetMenuItemInfo(
    _Menu: PMENU,
    MenuItem: PITEM,
    lpmii: PROSMENUITEMINFO,
) -> BOOL {
    IntGetMenuItemInfo_impl(_Menu, MenuItem, lpmii)
}

unsafe fn IntGetMenuItemInfo_impl(
    _Menu: PMENU,
    MenuItem: PITEM,
    lpmii: PROSMENUITEMINFO,
) -> BOOL {
    let Status: NTSTATUS;

    if (*lpmii).fMask & (MIIM_FTYPE | MIIM_TYPE) != 0 {
        (*lpmii).fType = (*MenuItem).fType;
    }
    if (*lpmii).fMask & MIIM_BITMAP != 0 {
        (*lpmii).hbmpItem = (*MenuItem).hbmp;
    }
    if (*lpmii).fMask & MIIM_CHECKMARKS != 0 {
        (*lpmii).hbmpChecked = (*MenuItem).hbmpChecked as HBITMAP ;
        (*lpmii).hbmpUnchecked = (*MenuItem).hbmpUnchecked as HBITMAP ;
    }
    if (*lpmii).fMask & MIIM_DATA != 0 {
        (*lpmii).dwItemData = (*MenuItem).dwItemData as _;
    }
    if (*lpmii).fMask & MIIM_ID != 0 {
        (*lpmii).wID = (*MenuItem).wID;
    }
    if (*lpmii).fMask & MIIM_STATE != 0 {
        (*lpmii).fState = (*MenuItem).fState;
    }
    if (*lpmii).fMask & MIIM_SUBMENU != 0 {
        (*lpmii).hSubMenu = if !(*MenuItem).spSubMenu.is_null() {
            UserHMGetHandle((*MenuItem).spSubMenu as *mut core::ffi::c_void) as HMENU
        } else {
            core::ptr::null_mut()
        };
    }

    if ((*lpmii).fMask & MIIM_STRING != 0)
        || (((*lpmii).fMask & MIIM_TYPE != 0)
            && (MENU_ITEM_TYPE((*lpmii).fType) == MF_STRING))
    {
        if (*lpmii).dwTypeData.is_null() {
            (*lpmii).cch = (*MenuItem).lpstr.Length as u32 / core::mem::size_of::<u16>() as u32;
        } else {
            let copy_len = core::cmp::min(
                (*lpmii).cch as usize * core::mem::size_of::<u16>(),
                (*MenuItem).lpstr.MaximumLength as usize,
            );
            Status = MmCopyToCaller(
                (*lpmii).dwTypeData as *mut core::ffi::c_void,
                (*MenuItem).lpstr.Buffer as *const core::ffi::c_void,
                copy_len,
            );
            if !NT_SUCCESS(Status) {
                SetLastNtError(Status);
                return 0;
            }
        }
    }

    if core::mem::size_of::<ROSMENUITEMINFO>() == (*lpmii).cbSize as usize {
        (*lpmii).Rect.left = (*MenuItem).xItem as i32 ;
        (*lpmii).Rect.top = (*MenuItem).yItem as i32 ;
        (*lpmii).Rect.right = (*MenuItem).cxItem as i32 ;
        (*lpmii).Rect.bottom = (*MenuItem).cyItem as i32 ;
        (*lpmii).dxTab = (*MenuItem).dxTab;
        (*lpmii).lpstr = (*MenuItem).lpstr.Buffer;
        (*lpmii).maxBmpSize.cx = (*MenuItem).cxBmp;
        (*lpmii).maxBmpSize.cy = (*MenuItem).cyBmp;
    }

    1
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntSetMenuItemInfo(
    MenuObject: PMENU,
    MenuItem: PITEM,
    lpmii: PROSMENUITEMINFO,
    lpstr: PUNICODE_STRING,
) -> BOOL {
    let mut SubMenuObject: PMENU;
    let mut circref: BOOL = 0;

    if MenuItem.is_null() || MenuObject.is_null() || lpmii.is_null() {
        return 0;
    }
    if (*lpmii).fMask & MIIM_FTYPE != 0 {
        (*MenuItem).fType &= !MENUITEMINFO_TYPE_MASK;
        (*MenuItem).fType |= (*lpmii).fType & MENUITEMINFO_TYPE_MASK;
    }
    if (*lpmii).fMask & MIIM_TYPE != 0 {
        if (*MenuItem).fType != (*lpmii).fType
            && MENU_ITEM_TYPE((*MenuItem).fType) == MFT_STRING
        {
            FreeMenuText!(MenuObject, MenuItem);
            RtlInitUnicodeString(
                core::ptr::addr_of_mut!((*MenuItem).lpstr).cast(),
                core::ptr::null_mut(),
            );
            (*MenuItem).Xlpstr = core::ptr::null_mut();
        }
        if (*lpmii).fType & MFT_BITMAP != 0 {
            if !(*lpmii).hbmpItem.is_null() {
                (*MenuItem).hbmp = (*lpmii).hbmpItem;
            } else {
                (*MenuItem).hbmp =
                    (LOWORD((*lpmii).dwTypeData as _) as usize) as HBITMAP;
            }
            (*lpmii).dwTypeData = core::ptr::null_mut();
        }
    }
    if (*lpmii).fMask & MIIM_BITMAP != 0 {
        (*MenuItem).hbmp = (*lpmii).hbmpItem;
        if (*MenuItem).hbmp as usize <= HBMMENU_POPUP_MINIMIZE as usize
            && (*MenuItem).hbmp as isize >= HBMMENU_CALLBACK as isize
        {
            (*MenuItem).fState |= MFS_HBMMENUBMP;
        } else {
            (*MenuItem).fState &= !MFS_HBMMENUBMP;
        }
    }
    if (*lpmii).fMask & MIIM_CHECKMARKS != 0 {
        (*MenuItem).hbmpChecked = ((*lpmii).hbmpChecked as *mut core::ffi::c_void);
        (*MenuItem).hbmpUnchecked = ((*lpmii).hbmpUnchecked as *mut core::ffi::c_void);
    }
    if (*lpmii).fMask & MIIM_DATA != 0 {
        (*MenuItem).dwItemData = (*lpmii).dwItemData;
    }
    if (*lpmii).fMask & MIIM_ID != 0 {
        (*MenuItem).wID = (*lpmii).wID;
    }
    if (*lpmii).fMask & MIIM_STATE != 0 {
        if (*lpmii).fState & MFS_DEFAULT != 0 {
            UserSetMenuDefaultItem(MenuObject, -1i32 as UINT, 0);
        }
        UpdateMenuItemState!((*MenuItem).fState, (*lpmii).fState);
    }

    if (*lpmii).fMask & MIIM_SUBMENU != 0 {
        if !(*lpmii).hSubMenu.is_null() {
            SubMenuObject = UserGetMenuObject((*lpmii).hSubMenu);
            if !SubMenuObject.is_null() && UserObjectInDestroy(((*lpmii).hSubMenu as *mut core::ffi::c_void)) == 0 {
                if MenuObject == SubMenuObject {
                    let mut hMenu: HANDLE = core::ptr::null_mut();
                    ERR!("Pop Up Menu Double Trouble!\n");
                    SubMenuObject = IntCreateMenu(
                        core::ptr::addr_of_mut!(hMenu),
                        0,
                        (*MenuObject).head.rpdesk,
                        (*MenuObject).head.hTaskWow as PPROCESSINFO,
                    );
                    if SubMenuObject.is_null() {
                        return 0;
                    }
                    IntReleaseMenuObject(SubMenuObject);
                    circref = 1;
                }
                if MENU_depth(SubMenuObject, 0) > (MAXMENUDEPTH as i32) {
                    ERR!("Loop detected in menu hierarchy or maximum menu depth exceeded!\n");
                    if circref != 0 {
                        IntDestroyMenuObject(SubMenuObject, 0);
                    }
                    return 0;
                }
                (*SubMenuObject).fFlags |= MNF_POPUP;
                if !(*MenuItem).spSubMenu.is_null() {
                    IntReleaseMenuObject((*MenuItem).spSubMenu);
                }
                (*MenuItem).spSubMenu = SubMenuObject;
                UserReferenceObject(SubMenuObject as _);
            } else {
                EngSetLastError(ERROR_INVALID_PARAMETER);
                return 0;
            }
        } else {
            if !(*MenuItem).spSubMenu.is_null() {
                IntReleaseMenuObject((*MenuItem).spSubMenu);
            }
            (*MenuItem).spSubMenu = core::ptr::null_mut();
        }
    }

    if (*lpmii).fMask & MIIM_STRING != 0
        || ((*lpmii).fMask & MIIM_TYPE != 0
            && MENU_ITEM_TYPE((*lpmii).fType) == MF_STRING)
    {
        FreeMenuText!(MenuObject, MenuItem);
        RtlInitUnicodeString(
            core::ptr::addr_of_mut!((*MenuItem).lpstr).cast(),
            core::ptr::null_mut(),
        );
        (*MenuItem).Xlpstr = core::ptr::null_mut();

        if !(*lpmii).dwTypeData.is_null()
            && (*lpmii).cch != 0
            && !lpstr.is_null()
            && !(*lpstr).Buffer.is_null()
        {
            let mut Source: _UNICODE_STRING = core::mem::zeroed();

            if !NT_VERIFY((*lpmii).cch <= UNICODE_STRING_MAX_CHARS) {
                return 0;
            }

            Source.Length = ((*lpmii).cch as usize * core::mem::size_of::<u16>()) as USHORT;
            Source.MaximumLength = Source.Length;
            Source.Buffer = (*lpmii).dwTypeData as LPWSTR;

            (*MenuItem).lpstr.Buffer = DesktopHeapAlloc(
                (*MenuObject).head.rpdesk,
                Source.Length as usize + core::mem::size_of::<u16>(),
            ) as LPWSTR;
            if !(*MenuItem).lpstr.Buffer.is_null() {
                (*MenuItem).lpstr.Length = 0;
                (*MenuItem).lpstr.MaximumLength =
                    Source.Length + core::mem::size_of::<u16>() as USHORT;
                RtlCopyUnicodeString(
                    core::ptr::addr_of_mut!((*MenuItem).lpstr).cast(),
                    core::ptr::addr_of_mut!(Source).cast(),
                );
                *(*MenuItem)
                    .lpstr
                    .Buffer
                    .add((*MenuItem).lpstr.Length as usize / core::mem::size_of::<u16>()) = 0;

                (*MenuItem).cch =
                    (*MenuItem).lpstr.Length as u32 / core::mem::size_of::<u16>() as u32;
                (*MenuItem).Xlpstr = (*MenuItem).lpstr.Buffer as *mut USHORT;
            }
        }
    }

    if (*MenuObject).fFlags & MNF_SYSMENU == 0
        && (*MenuItem).Xlpstr.is_null()
        && (*lpmii).dwTypeData.is_null()
        && (*MenuItem).fType & MFT_OWNERDRAW == 0
        && (*MenuItem).hbmp.is_null()
    {
        (*MenuItem).fType |= MFT_SEPARATOR;
    }

    if core::mem::size_of::<ROSMENUITEMINFO>() == (*lpmii).cbSize as usize {
        (*MenuItem).xItem = ((*lpmii).Rect.left as u32);
        (*MenuItem).yItem = ((*lpmii).Rect.top as u32);
        (*MenuItem).cxItem = ((*lpmii).Rect.right as u32);
        (*MenuItem).cyItem = ((*lpmii).Rect.bottom as u32);
        (*MenuItem).dxTab = (*lpmii).dxTab;
        (*lpmii).lpstr = (*MenuItem).lpstr.Buffer;
        (*MenuItem).cxBmp = (*lpmii).maxBmpSize.cx;
        (*MenuItem).cyBmp = (*lpmii).maxBmpSize.cy;
    }

    1
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntSetMenuItemInfo(
    MenuObject: PMENU,
    MenuItem: PITEM,
    lpmii: PROSMENUITEMINFO,
    lpstr: PUNICODE_STRING,
) -> BOOL {
    let mut SubMenuObject: PMENU;
    let mut circref: BOOL = 0;

    if MenuItem.is_null() || MenuObject.is_null() || lpmii.is_null() {
        return 0;
    }
    if (*lpmii).fMask & MIIM_FTYPE != 0 {
        (*MenuItem).fType &= !MENUITEMINFO_TYPE_MASK;
        (*MenuItem).fType |= (*lpmii).fType & MENUITEMINFO_TYPE_MASK;
    }
    if (*lpmii).fMask & MIIM_TYPE != 0 {
        if (*MenuItem).fType != (*lpmii).fType
            && MENU_ITEM_TYPE((*MenuItem).fType) == MFT_STRING
        {
            FreeMenuText!(MenuObject, MenuItem);
            RtlInitUnicodeString(
                core::ptr::addr_of_mut!((*MenuItem).lpstr).cast(),
                core::ptr::null_mut(),
            );
            (*MenuItem).Xlpstr = core::ptr::null_mut();
        }
        if (*lpmii).fType & MFT_BITMAP != 0 {
            if !(*lpmii).hbmpItem.is_null() {
                (*MenuItem).hbmp = (*lpmii).hbmpItem;
            } else {
                (*MenuItem).hbmp =
                    (LOWORD((*lpmii).dwTypeData as _) as usize) as HBITMAP;
            }
            (*lpmii).dwTypeData = core::ptr::null_mut();
        }
    }
    if (*lpmii).fMask & MIIM_BITMAP != 0 {
        (*MenuItem).hbmp = (*lpmii).hbmpItem;
        if (*MenuItem).hbmp as usize <= HBMMENU_POPUP_MINIMIZE as usize
            && (*MenuItem).hbmp as isize >= HBMMENU_CALLBACK as isize
        {
            (*MenuItem).fState |= MFS_HBMMENUBMP;
        } else {
            (*MenuItem).fState &= !MFS_HBMMENUBMP;
        }
    }
    if (*lpmii).fMask & MIIM_CHECKMARKS != 0 {
        (*MenuItem).hbmpChecked = (*lpmii).hbmpChecked as _;
        (*MenuItem).hbmpUnchecked = (*lpmii).hbmpUnchecked as _;
    }
    if (*lpmii).fMask & MIIM_DATA != 0 {
        (*MenuItem).dwItemData = (*lpmii).dwItemData as _;
    }
    if (*lpmii).fMask & MIIM_ID != 0 {
        (*MenuItem).wID = (*lpmii).wID;
    }
    if (*lpmii).fMask & MIIM_STATE != 0 {
        if (*lpmii).fState & MFS_DEFAULT != 0 {
            UserSetMenuDefaultItem(MenuObject, -1i32 as UINT, 0);
        }
        UpdateMenuItemState!((*MenuItem).fState, (*lpmii).fState);
    }

    if (*lpmii).fMask & MIIM_SUBMENU != 0 {
        if !(*lpmii).hSubMenu.is_null() {
            SubMenuObject = UserGetMenuObject((*lpmii).hSubMenu);
            if !SubMenuObject.is_null() && UserObjectInDestroy((*lpmii).hSubMenu as _) == 0 {
                if MenuObject == SubMenuObject {
                    let mut hMenu: HANDLE = core::ptr::null_mut();
                    ERR!("Pop Up Menu Double Trouble!\n");
                    SubMenuObject = IntCreateMenu(
                        core::ptr::addr_of_mut!(hMenu),
                        0,
                        (*MenuObject).head.rpdesk,
                        (*MenuObject).head.hTaskWow as PPROCESSINFO,
                    );
                    if SubMenuObject.is_null() {
                        return 0;
                    }
                    IntReleaseMenuObject(SubMenuObject);
                    circref = 1;
                }
                if MENU_depth(SubMenuObject, 0) > MAXMENUDEPTH as i32 {
                    ERR!("Loop detected in menu hierarchy or maximum menu depth exceeded!\n");
                    if circref != 0 {
                        IntDestroyMenuObject(SubMenuObject, 0);
                    }
                    return 0;
                }
                (*SubMenuObject).fFlags |= MNF_POPUP;
                if !(*MenuItem).spSubMenu.is_null() {
                    IntReleaseMenuObject((*MenuItem).spSubMenu);
                }
                (*MenuItem).spSubMenu = SubMenuObject;
                UserReferenceObject(SubMenuObject as _);
            } else {
                EngSetLastError(ERROR_INVALID_PARAMETER);
                return 0;
            }
        } else {
            if !(*MenuItem).spSubMenu.is_null() {
                IntReleaseMenuObject((*MenuItem).spSubMenu);
            }
            (*MenuItem).spSubMenu = core::ptr::null_mut();
        }
    }

    if (*lpmii).fMask & MIIM_STRING != 0
        || ((*lpmii).fMask & MIIM_TYPE != 0
            && MENU_ITEM_TYPE((*lpmii).fType) == MF_STRING)
    {
        FreeMenuText!(MenuObject, MenuItem);
        RtlInitUnicodeString(
            core::ptr::addr_of_mut!((*MenuItem).lpstr).cast(),
            core::ptr::null_mut(),
        );
        (*MenuItem).Xlpstr = core::ptr::null_mut();

        if !(*lpmii).dwTypeData.is_null()
            && (*lpmii).cch != 0
            && !lpstr.is_null()
            && !(*lpstr).Buffer.is_null()
        {
            let mut Source: _UNICODE_STRING = core::mem::zeroed();

            if !NT_VERIFY((*lpmii).cch <= UNICODE_STRING_MAX_CHARS) {
                return 0;
            }

            Source.Length = ((*lpmii).cch as usize * core::mem::size_of::<u16>()) as USHORT;
            Source.MaximumLength = Source.Length;
            Source.Buffer = (*lpmii).dwTypeData as LPWSTR;

            (*MenuItem).lpstr.Buffer = DesktopHeapAlloc(
                (*MenuObject).head.rpdesk,
                Source.Length as usize + core::mem::size_of::<u16>(),
            ) as LPWSTR;
            if !(*MenuItem).lpstr.Buffer.is_null() {
                (*MenuItem).lpstr.Length = 0;
                (*MenuItem).lpstr.MaximumLength =
                    Source.Length + core::mem::size_of::<u16>() as USHORT;
                RtlCopyUnicodeString(
                    core::ptr::addr_of_mut!((*MenuItem).lpstr).cast(),
                    core::ptr::addr_of_mut!(Source).cast(),
                );
                *(*MenuItem)
                    .lpstr
                    .Buffer
                    .add((*MenuItem).lpstr.Length as usize / core::mem::size_of::<u16>()) = 0;

                (*MenuItem).cch =
                    (*MenuItem).lpstr.Length as u32 / core::mem::size_of::<u16>() as u32;
                (*MenuItem).Xlpstr = (*MenuItem).lpstr.Buffer as *mut USHORT;
            }
        }
    }

    if (*MenuObject).fFlags & MNF_SYSMENU == 0
        && (*MenuItem).Xlpstr.is_null()
        && (*lpmii).dwTypeData.is_null()
        && (*MenuItem).fType & MFT_OWNERDRAW == 0
        && (*MenuItem).hbmp.is_null()
    {
        (*MenuItem).fType |= MFT_SEPARATOR;
    }

    if core::mem::size_of::<ROSMENUITEMINFO>() == (*lpmii).cbSize as usize {
        (*MenuItem).xItem = (*lpmii).Rect.left as _;
        (*MenuItem).yItem = (*lpmii).Rect.top as _;
        (*MenuItem).cxItem = (*lpmii).Rect.right as _;
        (*MenuItem).cyItem = (*lpmii).Rect.bottom as _;
        (*MenuItem).dxTab = (*lpmii).dxTab;
        (*lpmii).lpstr = (*MenuItem).lpstr.Buffer;
        (*MenuItem).cxBmp = (*lpmii).maxBmpSize.cx;
        (*MenuItem).cyBmp = (*lpmii).maxBmpSize.cy;
    }

    1
}

fastcall_fn! {
unsafe fn IntEnableMenuItem(MenuObject: PMENU, uIDEnableItem: UINT, uEnable: UINT) -> UINT {
    let MenuItem: PITEM;
    let res: UINT;

    let mut pMenu = MenuObject;
    let mut uID = uIDEnableItem;

    MenuItem = MENU_FindItem(&mut pMenu, &mut uID, uEnable);
    if MenuItem.is_null() {
        return (-1i32) as UINT;
    }

    res = (*MenuItem).fState & (MF_GRAYED | MF_DISABLED);

    (*MenuItem).fState ^= (res ^ uEnable) & (MF_GRAYED | MF_DISABLED);

    if res != uEnable {
        match (*MenuItem).wID {
            x if x == SC_CLOSE
                || x == SC_MAXIMIZE
                || x == SC_MINIMIZE
                || x == SC_MOVE
                || x == SC_RESTORE
                || x == SC_SIZE =>
            {
                if (*pMenu).fFlags & MNF_SYSSUBMENU != 0 && !(*pMenu).spwndNotify.is_null() {
                    UserPaintCaption((*pMenu).spwndNotify, DC_BUTTONS as INT);
                }
            }
            _ => {}
        }
    }
    res
}
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntCheckMenuItem(MenuObject: PMENU, mut uIDCheckItem: UINT, uCheck: UINT) -> DWORD {
    let MenuItem: PITEM;
    let res: DWORD;

    let mut pMenu = MenuObject;
    MenuItem = MENU_FindItem(&mut pMenu, &mut uIDCheckItem, uCheck);
    if MenuItem.is_null() {
        return u32::MAX;
    }

    res = ((*MenuItem).fState & MF_CHECKED) as DWORD;

    (*MenuItem).fState ^= (res ^ uCheck) & MF_CHECKED;

    res
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntCheckMenuItem(MenuObject: PMENU, mut uIDCheckItem: UINT, uCheck: UINT) -> DWORD {
    let MenuItem: PITEM;
    let res: DWORD;

    let mut pMenu = MenuObject;
    MenuItem = MENU_FindItem(&mut pMenu, &mut uIDCheckItem, uCheck);
    if MenuItem.is_null() {
        return u32::MAX;
    }

    res = ((*MenuItem).fState & MF_CHECKED) as DWORD;

    (*MenuItem).fState ^= (res ^ uCheck) & MF_CHECKED;

    res
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn UserSetMenuDefaultItem(
    MenuObject: PMENU,
    uItem: UINT,
    fByPos: UINT,
) -> BOOL {
    UserSetMenuDefaultItem_impl(MenuObject, uItem, fByPos)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn UserSetMenuDefaultItem(
    MenuObject: PMENU,
    uItem: UINT,
    fByPos: UINT,
) -> BOOL {
    UserSetMenuDefaultItem_impl(MenuObject, uItem, fByPos)
}

unsafe fn UserSetMenuDefaultItem_impl(MenuObject: PMENU, uItem: UINT, fByPos: UINT) -> BOOL {
    let mut i: UINT;
    let mut MenuItem: PITEM = (*MenuObject).rgItems;

    if MenuItem.is_null() {
        return 0;
    }

    i = 0;
    while i < (*MenuObject).cItems {
        (*MenuItem).fState &= !MFS_DEFAULT;
        i += 1;
        MenuItem = MenuItem.add(1);
    }

    if uItem == (-1i32) as UINT {
        return 1;
    }

    MenuItem = (*MenuObject).rgItems;
    if fByPos != 0 {
        if uItem >= (*MenuObject).cItems {
            return 0;
        }
        (*MenuItem.add(uItem as usize)).fState |= MFS_DEFAULT;
        return 1;
    } else {
        i = 0;
        while i < (*MenuObject).cItems {
            if (*MenuItem).wID == uItem {
                (*MenuItem).fState |= MFS_DEFAULT;
                return 1;
            }
            i += 1;
            MenuItem = MenuItem.add(1);
        }
    }
    0
}

fastcall_fn! {
unsafe fn IntGetMenuDefaultItem(MenuObject: PMENU, fByPos: UINT, gmdiFlags: UINT, gismc: *mut DWORD) -> UINT {
    let mut i: UINT = 0;
    let mut MenuItem: PITEM = (*MenuObject).rgItems;

    /* empty menu */
    if MenuItem.is_null() {
        return -1i32 as UINT;
    }

    while ((*MenuItem).fState & MFS_DEFAULT) == 0 {
        i += 1;
        MenuItem = MenuItem.add(1);
        if i >= (*MenuObject).cItems {
            return -1i32 as UINT;
        }
    }

    /* default: don't return disabled items */
    if (GMDI_USEDISABLED & gmdiFlags) == 0 && ((*MenuItem).fState & MFS_DISABLED) != 0 {
        return -1i32 as UINT;
    }

    /* search rekursiv when needed */
    if (gmdiFlags & GMDI_GOINTOPOPUPS) != 0 && !(*MenuItem).spSubMenu.is_null() {
        *gismc += 1;
        let ret = IntGetMenuDefaultItem((*MenuItem).spSubMenu, fByPos, gmdiFlags, gismc);
        *gismc -= 1;
        if ret != -1i32 as UINT {
            return ret;
        }

        /* when item not found in submenu, return the popup item */
    }

    if fByPos != 0 {
        i
    } else {
        (*MenuItem).wID as UINT
    }
}
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn co_IntGetSubMenu(mut pMenu: PMENU, mut nPos: core::ffi::c_int) -> PMENU {
    let pItem: PITEM = MENU_FindItem(
        core::ptr::addr_of_mut!(pMenu),
        core::ptr::addr_of_mut!(nPos) as *mut UINT,
        MF_BYPOSITION,
    );
    if pItem.is_null() {
        return core::ptr::null_mut();
    }
    (*pItem).spSubMenu
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn co_IntGetSubMenu(mut pMenu: PMENU, mut nPos: core::ffi::c_int) -> PMENU {
    let pItem: PITEM = MENU_FindItem(
        core::ptr::addr_of_mut!(pMenu),
        core::ptr::addr_of_mut!(nPos) as *mut UINT,
        MF_BYPOSITION,
    );
    if pItem.is_null() {
        return core::ptr::null_mut();
    }
    (*pItem).spSubMenu
}

fastcall_fn! {
#[no_mangle]
unsafe fn MENU_InitSysMenuPopup(menu: PMENU, style: DWORD, clsStyle: DWORD, HitTest: LONG) {
    let mut gray: BOOL;
    let DefItem: UINT;

    gray = ((style & WS_THICKFRAME) == 0 || (style & (WS_MAXIMIZE | WS_MINIMIZE)) != 0) as BOOL;
    IntEnableMenuItem(menu, SC_SIZE, if gray != 0 { MF_GRAYED } else { MF_ENABLED });
    gray = ((style & (WS_MAXIMIZE | WS_MINIMIZE)) != 0) as BOOL;
    IntEnableMenuItem(menu, SC_MOVE, if gray != 0 { MF_GRAYED } else { MF_ENABLED });
    gray = ((style & WS_MINIMIZEBOX) == 0 || (style & WS_MINIMIZE) != 0) as BOOL;
    IntEnableMenuItem(menu, SC_MINIMIZE, if gray != 0 { MF_GRAYED } else { MF_ENABLED });
    gray = ((style & WS_MAXIMIZEBOX) == 0 || (style & WS_MAXIMIZE) != 0) as BOOL;
    IntEnableMenuItem(menu, SC_MAXIMIZE, if gray != 0 { MF_GRAYED } else { MF_ENABLED });
    gray = ((style & (WS_MAXIMIZE | WS_MINIMIZE)) == 0) as BOOL;
    IntEnableMenuItem(menu, SC_RESTORE, if gray != 0 { MF_GRAYED } else { MF_ENABLED });
    gray = ((clsStyle & CS_NOCLOSE) != 0) as BOOL;

    if gray != 0 {
        IntEnableMenuItem(menu, SC_CLOSE, MF_GRAYED);
    }

    if (style & WS_MINIMIZE) != 0 {
        DefItem = SC_RESTORE;
    } else if HitTest == HTCAPTION as LONG {
        DefItem = if (style & (WS_MAXIMIZE | WS_MINIMIZE)) != 0 { SC_RESTORE } else { SC_MAXIMIZE };
    } else {
        DefItem = SC_CLOSE;
    }

    UserSetMenuDefaultItem(menu, DefItem, MF_BYCOMMAND);
}
}

#[cfg(target_pointer_width = "64")]
unsafe extern "C" fn MENU_DrawPopupGlyph(
    dc: HDC,
    r: LPRECT,
    popupMagic: INT_PTR,
    inactive: i32,
    hilite: i32,
) {
    MENU_DrawPopupGlyph_impl(dc, r, popupMagic, inactive, hilite)
}

#[cfg(not(target_pointer_width = "64"))]
unsafe extern "fastcall" fn MENU_DrawPopupGlyph(
    dc: HDC,
    r: LPRECT,
    popupMagic: INT_PTR,
    inactive: i32,
    hilite: i32,
) {
    MENU_DrawPopupGlyph_impl(dc, r, popupMagic, inactive, hilite)
}

unsafe fn MENU_DrawPopupGlyph_impl(
    dc: HDC,
    r: LPRECT,
    popupMagic: INT_PTR,
    inactive: i32,
    hilite: i32,
) {
    let mut lf: LOGFONTW = core::mem::zeroed();
    let hFont: HFONT;
    let hOldFont: HFONT;
    let clrsave: COLORREF;
    let bkmode: INT;
    let symbol: WCHAR;

    const HBMMENU_POPUP_RESTORE: INT_PTR = 9;
    const HBMMENU_POPUP_MINIMIZE: INT_PTR = 11;
    const HBMMENU_POPUP_MAXIMIZE: INT_PTR = 10;
    const HBMMENU_POPUP_CLOSE: INT_PTR = 8;

    match popupMagic {
        HBMMENU_POPUP_RESTORE => symbol = b'2' as WCHAR,
        HBMMENU_POPUP_MINIMIZE => symbol = b'0' as WCHAR,
        HBMMENU_POPUP_MAXIMIZE => symbol = b'1' as WCHAR,
        HBMMENU_POPUP_CLOSE => symbol = b'r' as WCHAR,
        _ => {
            return;
        }
    }

    core::ptr::write_bytes(&mut lf as *mut LOGFONTW, 0, 1);
    RECTL_vInflateRect(r, -2, -2);
    lf.lfHeight = (*r).bottom - (*r).top;
    lf.lfWidth = 0;
    lf.lfWeight = FW_NORMAL as i32;
    lf.lfCharSet = DEFAULT_CHARSET as u8;

    let marlett: [u16; 8] = [
        b'M' as u16,
        b'a' as u16,
        b'r' as u16,
        b'l' as u16,
        b'e' as u16,
        b't' as u16,
        b't' as u16,
        0,
    ];
    core::ptr::copy_nonoverlapping(
        marlett.as_ptr() as *const u8,
        lf.lfFaceName.as_mut_ptr() as *mut u8,
        core::mem::size_of_val(&marlett),
    );

    hFont = GreCreateFontIndirectW(&mut lf);
    hOldFont = NtGdiSelectFont(dc, hFont);
    clrsave = GreGetTextColor(dc);
    bkmode = GreGetBkMode(dc);

    IntGdiSetBkMode(dc, TRANSPARENT as i32);
    if inactive != 0 {
        if hilite == 0 {
            IntGdiSetTextColor(dc, IntGetSysColor(COLOR_HIGHLIGHTTEXT as i32));
            GreTextOutW(dc, (*r).left + 1, (*r).top + 1, &symbol, 1);
        }
    }
    IntGdiSetTextColor(
        dc,
        IntGetSysColor(if inactive != 0 {
            COLOR_GRAYTEXT as i32
        } else if hilite != 0 {
            COLOR_HIGHLIGHTTEXT as i32
        } else {
            COLOR_MENUTEXT as i32
        }),
    );
    GreTextOutW(dc, (*r).left, (*r).top, &symbol, 1);
    IntGdiSetTextColor(dc, clrsave);
    NtGdiSelectFont(dc, hOldFont);
    IntGdiSetBkMode(dc, bkmode);
    GreDeleteObject(hFont as _);
}


fastcall_fn! {
unsafe fn MENU_AdjustMenuItemRect(menu: PMENU, rect: PRECTL) {
    if (*menu).dwArrowsOn() != 0 {
        let arrow_bitmap_height: UINT;
        arrow_bitmap_height = (*pusi(gpsi)).oembmi[OBI_UPARROW as usize].cy as UINT;
        (*rect).top += arrow_bitmap_height as i32 - (*menu).iTop as i32;
        (*rect).bottom += arrow_bitmap_height as i32 - (*menu).iTop as i32;
    }
}
}

unsafe fn MENU_FindItemByCoords(menu: *mut MENU, mut pt: POINT, pos: *mut UINT) -> *mut ITEM {
    let mut item: *mut ITEM;
    let mut i: UINT;
    let cx: INT;
    let cy: INT;
    let mut rect: RECT = core::mem::zeroed();
    let pWnd: PWND = ValidateHwndNoErr((*menu).hWnd);

    if IntGetWindowRect(pWnd, (&mut rect as *mut RECT).cast()) == 0 {
        return core::ptr::null_mut();
    }

    cx = UserGetSystemMetrics(SM_CXDLGFRAME);
    cy = UserGetSystemMetrics(SM_CYDLGFRAME);
    RECTL_vInflateRect(&mut rect, -cx, -cy);

    if (*pWnd).ExStyle & WS_EX_LAYOUTRTL != 0 {
        pt.x = rect.right - 1 - pt.x;
    } else {
        pt.x -= rect.left;
    }
    pt.y -= rect.top;
    item = (*menu).rgItems;
    i = 0;
    while i < (*menu).cItems {
        rect.left = (*item).xItem as i32;
        rect.top = (*item).yItem as i32;
        rect.right = (*item).cxItem as i32;
        rect.bottom = (*item).cyItem as i32;

        MENU_AdjustMenuItemRect(menu, &mut rect as *mut RECT as PRECTL);
        if RECTL_bPointInRect((&mut rect as *mut RECT).cast(), pt.x, pt.y) != 0 {
            if !pos.is_null() {
                *pos = i;
            }
            return item;
        }
        i += 1;
        item = item.add(1);
    }
    core::ptr::null_mut()
}


#[cfg(target_arch = "x86")]
#[no_mangle]
unsafe extern "fastcall" fn IntMenuItemFromPoint(pWnd: PWND, hMenu: HMENU, ptScreen: POINT) -> INT {
    IntMenuItemFromPoint_impl(pWnd, hMenu, ptScreen)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
unsafe extern "C" fn IntMenuItemFromPoint(pWnd: PWND, hMenu: HMENU, ptScreen: POINT) -> INT {
    IntMenuItemFromPoint_impl(pWnd, hMenu, ptScreen)
}

unsafe fn IntMenuItemFromPoint_impl(_pWnd: PWND, hMenu: HMENU, ptScreen: POINT) -> INT {
    let menu: *mut MENU = UserGetMenuObject(hMenu);
    let mut pos: UINT = 0;

    if menu.is_null() {
        return -1;
    }
    if MENU_FindItemByCoords(menu, ptScreen, &mut pos).is_null() {
        return -1;
    }
    pos as INT
}

fastcall_fn! {
unsafe fn MENU_FindItemByKey(WndOwner: PWND, mut menu: PMENU, Key: WCHAR, ForceMenuChar: BOOL) -> UINT {
    let MenuChar: LRESULT;
    let mut Flags: WORD = 0;

    if menu.is_null() || VerifyMenu(menu).is_null() {
        menu = co_IntGetSubMenu(UserGetMenuObject((*WndOwner).SystemMenu), 0);
    }
    if !menu.is_null() {
        let mut item: *mut ITEM = (*menu).rgItems;

        if ForceMenuChar == 0 {
            let mut i: UINT;
            let cjk: BOOL = UserGetSystemMetrics(SM_DBCSENABLED as core::ffi::c_int as u32 );

            i = 0;
            while i < (*menu).cItems {
                let text: LPWSTR = (*item).Xlpstr;
                if !text.is_null() {
                    let mut p: *const WCHAR = text.offset(-2);
                    loop {
                        let q: *const WCHAR = p.offset(2);
                        p = wcschr(q, '&' as WCHAR as core::ffi::c_int);
                        if p.is_null() && cjk != 0 {
                            p = wcschr(q, 0o36 as core::ffi::c_int);
                        }
                        if !(!p.is_null() && *p.offset(1) == '&' as WCHAR) {
                            break;
                        }
                    }
                    if !p.is_null() && towupper(*p.offset(1) as core::ffi::c_int) == towupper(Key as core::ffi::c_int) {
                        return i;
                    }
                }
                i += 1;
                item = item.offset(1);
            }
        }

        Flags |= if (*menu).fFlags & MNF_POPUP != 0 { MF_POPUP as WORD } else { 0 };
        Flags |= if (*menu).fFlags & MNF_SYSMENU != 0 { MF_SYSMENU as WORD } else { 0 };

        MenuChar = co_IntSendMessage(
            UserHMGetHandle(WndOwner) as HWND,
            WM_MENUCHAR,
            MAKEWPARAM(Key as u32, Flags as u32) as WPARAM,
            UserHMGetHandle(menu) as LPARAM,
        );
        if HIWORD(MenuChar as u32) as u32 == MNC_EXECUTE {
            return LOWORD(MenuChar as u32) as UINT;
        }
        if HIWORD(MenuChar as u32) as u32 == MNC_CLOSE {
            return (-2i32) as UINT;
        }
    }
    (-1i32) as UINT
}
}

unsafe fn MENU_GetBitmapItemSize(lpitem: PITEM, size: *mut SIZE, WndOwner: PWND) {
    let mut bm: BITMAP = core::mem::zeroed();
    let mut bmp: HBITMAP = (*lpitem).hbmp;

    (*size).cx = 0;
    (*size).cy = 0;

    if IS_MAGIC_BITMAP(bmp as INT_PTR) != 0 {
        // HBMMENU_* as INT_PTR values (winuser.h) for the match below.
        const HBMMENU_CALLBACK: INT_PTR = -1;
        const HBMMENU_SYSTEM: INT_PTR = 1;
        const HBMMENU_MBAR_RESTORE: INT_PTR = 2;
        const HBMMENU_MBAR_MINIMIZE: INT_PTR = 3;
        const HBMMENU_MBAR_CLOSE: INT_PTR = 5;
        const HBMMENU_MBAR_CLOSE_D: INT_PTR = 6;
        const HBMMENU_MBAR_MINIMIZE_D: INT_PTR = 7;
        const HBMMENU_POPUP_CLOSE: INT_PTR = 8;
        const HBMMENU_POPUP_RESTORE: INT_PTR = 9;
        const HBMMENU_POPUP_MAXIMIZE: INT_PTR = 10;
        const HBMMENU_POPUP_MINIMIZE: INT_PTR = 11;
        match bmp as INT_PTR {
            HBMMENU_CALLBACK => {
                let mut measItem: MEASUREITEMSTRUCT = core::mem::zeroed();
                measItem.CtlType = ODT_MENU;
                measItem.CtlID = 0;
                measItem.itemID = (*lpitem).wID as u32;
                measItem.itemWidth = ((*lpitem).cxItem - (*lpitem).xItem) as u32;
                measItem.itemHeight = ((*lpitem).cyItem - (*lpitem).yItem) as u32;
                measItem.itemData = (*lpitem).dwItemData;
                co_IntSendMessage(
                    UserHMGetHandle(WndOwner) as HWND,
                    WM_MEASUREITEM,
                    0,
                    core::ptr::addr_of_mut!(measItem) as LPARAM,
                );
                (*size).cx = measItem.itemWidth as i32;
                (*size).cy = measItem.itemHeight as i32;
                TRACE!(
                    b"HBMMENU_CALLBACK Height %d Width %d\n\0".as_ptr(),
                    measItem.itemHeight,
                    measItem.itemWidth,
                );
                return;
            }
            HBMMENU_SYSTEM => {
                if (*lpitem).dwItemData != 0 {
                    bmp = (*lpitem).dwItemData as HBITMAP;
                } else {
                    (*size).cx = UserGetSystemMetrics(SM_CXSIZE) - 2;
                    (*size).cy = UserGetSystemMetrics(SM_CYSIZE) - 4;
                    return;
                }
            }
            HBMMENU_MBAR_RESTORE
            | HBMMENU_MBAR_MINIMIZE
            | HBMMENU_MBAR_CLOSE
            | HBMMENU_MBAR_MINIMIZE_D
            | HBMMENU_MBAR_CLOSE_D
            | HBMMENU_POPUP_CLOSE
            | HBMMENU_POPUP_RESTORE
            | HBMMENU_POPUP_MAXIMIZE
            | HBMMENU_POPUP_MINIMIZE => {
                (*size).cx = UserGetSystemMetrics(SM_CXSIZE) - 2;
                (*size).cy = UserGetSystemMetrics(SM_CYSIZE) - 4;
                return;
            }
            _ => {}
        }
    }

    if GreGetObject(
        bmp as _,
        core::mem::size_of::<BITMAP>() as _,
        core::ptr::addr_of_mut!(bm) as _,
    ) != 0
    {
        (*size).cx = bm.bmWidth;
        (*size).cy = bm.bmHeight;
    }
}

#[cfg(target_pointer_width = "32")]
unsafe extern "fastcall" fn MENU_DrawBitmapItem(
    hdc: HDC,
    lpitem: PITEM,
    rect: *const RECT,
    Menu: PMENU,
    WndOwner: PWND,
    odaction: UINT,
    MenuBar: BOOL,
) {
    MENU_DrawBitmapItem_impl(hdc, lpitem, rect, Menu, WndOwner, odaction, MenuBar)
}

#[cfg(target_pointer_width = "64")]
unsafe extern "C" fn MENU_DrawBitmapItem(
    hdc: HDC,
    lpitem: PITEM,
    rect: *const RECT,
    Menu: PMENU,
    WndOwner: PWND,
    odaction: UINT,
    MenuBar: BOOL,
) {
    MENU_DrawBitmapItem_impl(hdc, lpitem, rect, Menu, WndOwner, odaction, MenuBar)
}

unsafe fn MENU_DrawBitmapItem_impl(
    hdc: HDC,
    lpitem: PITEM,
    rect: *const RECT,
    Menu: PMENU,
    WndOwner: PWND,
    odaction: UINT,
    MenuBar: BOOL,
) {

    const HBMMENU_SYSTEM: INT_PTR = 1;
    const HBMMENU_MBAR_RESTORE: INT_PTR = 2;
    const HBMMENU_MBAR_MINIMIZE: INT_PTR = 3;
    const HBMMENU_MBAR_MINIMIZE_D: INT_PTR = 7;
    const HBMMENU_MBAR_CLOSE: INT_PTR = 5;
    const HBMMENU_MBAR_CLOSE_D: INT_PTR = 6;
    const HBMMENU_CALLBACK: INT_PTR = -1;
    const HBMMENU_POPUP_CLOSE: INT_PTR = 8;
    const HBMMENU_POPUP_RESTORE: INT_PTR = 9;
    const HBMMENU_POPUP_MAXIMIZE: INT_PTR = 10;
    const HBMMENU_POPUP_MINIMIZE: INT_PTR = 11;

    let mut bm: BITMAP = core::mem::zeroed();
    let rop: DWORD;
    let hdcMem: HDC;
    let mut bmp: HBITMAP;
    let w = (*rect).right - (*rect).left;
    let h = (*rect).bottom - (*rect).top;
    let bmp_xoffset = 0;
    let mut left;
    let top;
    let mut flat_menu: BOOL = 0;
    let hbmToDraw: HBITMAP = (*lpitem).hbmp;
    bmp = hbmToDraw;

    UserSystemParametersInfo(
        SPI_GETFLATMENU,
        0,
        core::ptr::addr_of_mut!(flat_menu) as *mut core::ffi::c_void,
        0,
    );

    if IS_MAGIC_BITMAP(hbmToDraw as INT_PTR) != 0 {
        let mut flags: UINT = 0;
        let mut r: RECT;

        r = *rect;
        'got_bitmap: {
            match hbmToDraw as INT_PTR {
                HBMMENU_SYSTEM => {
                    if (*lpitem).dwItemData != 0 {
                        if !ValidateHwndNoErr((*lpitem).dwItemData as HWND).is_null() {
                            ERR!("Get Item Data from this Window!!!\n");
                        }

                        ERR!("Draw Bitmap\n");
                        bmp = (*lpitem).dwItemData as HBITMAP;
                        if GreGetObject(
                            bmp as *mut core::ffi::c_void,
                            core::mem::size_of::<BITMAP>() as i32,
                            core::ptr::addr_of_mut!(bm) as *mut core::ffi::c_void,
                        ) == 0
                        {
                            return;
                        }
                    } else {
                        let mut pIcon: PCURICON_OBJECT = core::ptr::null_mut();
                        if !WndOwner.is_null() {
                            pIcon = NC_IconForWindow(WndOwner);
                            if !pIcon.is_null() {
                                UserReferenceObject(pIcon as *mut core::ffi::c_void);
                            }
                        }
                        ERR!("Draw ICON\n");
                        if !pIcon.is_null() {
                            let cx = UserGetSystemMetrics(SM_CXSMICON);
                            let cy = UserGetSystemMetrics(SM_CYSMICON);
                            let x = (*rect).left - cx / 2 + 1 + ((*rect).bottom - (*rect).top) / 2;
                            let y = ((*rect).top + (*rect).bottom) / 2 - cy / 2;
                            UserDrawIconEx(
                                hdc,
                                x,
                                y,
                                pIcon,
                                cx,
                                cy,
                                0,
                                core::ptr::null_mut(),
                                DI_NORMAL,
                            );
                            UserDereferenceObject(pIcon as *mut core::ffi::c_void);
                        }
                        return;
                    }
                    break 'got_bitmap;
                }
                HBMMENU_MBAR_RESTORE => {
                    flags = DFCS_CAPTIONRESTORE;
                }
                HBMMENU_MBAR_MINIMIZE => {
                    r.right += 1;
                    flags = DFCS_CAPTIONMIN;
                }
                HBMMENU_MBAR_MINIMIZE_D => {
                    r.right += 1;
                    flags = DFCS_CAPTIONMIN | DFCS_INACTIVE;
                }
                HBMMENU_MBAR_CLOSE => {
                    flags = DFCS_CAPTIONCLOSE;
                }
                HBMMENU_MBAR_CLOSE_D => {
                    flags = DFCS_CAPTIONCLOSE | DFCS_INACTIVE;
                }
                HBMMENU_CALLBACK => {
                    let mut drawItem: DRAWITEMSTRUCT = core::mem::zeroed();
                    let mut origorg: POINT = core::mem::zeroed();
                    drawItem.CtlType = ODT_MENU;
                    drawItem.CtlID = 0;
                    drawItem.itemID = (*lpitem).wID;
                    drawItem.itemAction = odaction;
                    drawItem.itemState = if (*lpitem).fState & MF_CHECKED != 0 {
                        ODS_CHECKED
                    } else {
                        0
                    };
                    drawItem.itemState |= if (*lpitem).fState & MF_DEFAULT != 0 {
                        ODS_DEFAULT
                    } else {
                        0
                    };
                    drawItem.itemState |= if (*lpitem).fState & MF_DISABLED != 0 {
                        ODS_DISABLED
                    } else {
                        0
                    };
                    drawItem.itemState |= if (*lpitem).fState & MF_GRAYED != 0 {
                        ODS_GRAYED | ODS_DISABLED
                    } else {
                        0
                    };
                    drawItem.itemState |= if (*lpitem).fState & MF_HILITE != 0 {
                        ODS_SELECTED
                    } else {
                        0
                    };
                    drawItem.itemState |= if (*Menu).fFlags & MNF_UNDERLINE == 0 {
                        ODS_NOACCEL
                    } else {
                        0
                    };
                    drawItem.itemState |= if (*Menu).fFlags & MNF_INACTIVE != 0 {
                        ODS_INACTIVE
                    } else {
                        0
                    };
                    drawItem.hwndItem = UserHMGetHandle(Menu as *mut core::ffi::c_void) as HWND ;
                    drawItem.hDC = hdc;
                    drawItem.rcItem = core::mem::transmute_copy(&*rect);
                    drawItem.itemData = (*lpitem).dwItemData;
                    GreSetViewportOrgEx(
                        hdc,
                        (*lpitem).xItem as INT,
                        (*lpitem).yItem as INT,
                        core::ptr::addr_of_mut!(origorg).cast(),
                    );
                    RECTL_vOffsetRect(
                        core::ptr::addr_of_mut!(drawItem.rcItem) as *mut RECT ,
                        -((*lpitem).xItem as i32),
                        -((*lpitem).yItem as i32),
                    );
                    co_IntSendMessage(
                        UserHMGetHandle(WndOwner as *mut core::ffi::c_void) as HWND,
                        WM_DRAWITEM,
                        0,
                        core::ptr::addr_of_mut!(drawItem) as LPARAM,
                    );
                    GreSetViewportOrgEx(hdc, origorg.x, origorg.y, core::ptr::null_mut());
                    return;
                }
                HBMMENU_POPUP_CLOSE | HBMMENU_POPUP_RESTORE | HBMMENU_POPUP_MAXIMIZE
                | HBMMENU_POPUP_MINIMIZE => {
                    MENU_DrawPopupGlyph(
                        hdc,
                        core::ptr::addr_of_mut!(r),
                        hbmToDraw as INT_PTR,
                        ((*lpitem).fState & MF_GRAYED) as BOOL,
                        ((*lpitem).fState & MF_HILITE) as BOOL,
                    );
                    return;
                }
                _ => {}
            }
            RECTL_vInflateRect(core::ptr::addr_of_mut!(r), -1, -1);
            if (*lpitem).fState & MF_HILITE != 0 {
                flags |= DFCS_PUSHED;
            }
            DrawFrameControl(hdc, core::ptr::addr_of_mut!(r), DFC_CAPTION, flags);
            return;
        }
    }

    if bmp.is_null()
        || GreGetObject(
            bmp as *mut core::ffi::c_void,
            core::mem::size_of::<BITMAP>() as i32,
            core::ptr::addr_of_mut!(bm) as *mut core::ffi::c_void,
        ) == 0
    {
        return;
    }

    hdcMem = NtGdiCreateCompatibleDC(hdc);
    NtGdiSelectBitmap(hdcMem, bmp);
    top = if h > bm.bmHeight {
        (*rect).top + (h - bm.bmHeight) / 2
    } else {
        (*rect).top
    };
    left = (*rect).left;
    rop = if (*lpitem).fState & MF_HILITE != 0 && IS_MAGIC_BITMAP(hbmToDraw as INT_PTR) == 0 {
        NOTSRCCOPY
    } else {
        SRCCOPY
    };
    if (*lpitem).fState & MF_HILITE != 0 && !(*lpitem).hbmp.is_null() {
        IntGdiSetBkColor(hdc, IntGetSysColor(COLOR_HIGHLIGHT as INT));
    }
    if MenuBar != 0
        && flat_menu == 0
        && ((*lpitem).fState & (MF_HILITE | MF_GRAYED)) == MF_HILITE
    {
        left += 1;
    }
    let top = if MenuBar != 0
        && flat_menu == 0
        && ((*lpitem).fState & (MF_HILITE | MF_GRAYED)) == MF_HILITE
    {
        top + 1
    } else {
        top
    };
    NtGdiBitBlt(
        hdc,
        left,
        top,
        w,
        h,
        hdcMem,
        bmp_xoffset,
        0,
        rop,
        CLR_INVALID,
        0,
    );
    IntGdiDeleteDC(hdcMem, 0);
}

unsafe fn IntGetDialogBaseUnits() -> LONG {
    static mut units: DWORD = 0;

    if units == 0 {
        let hdc: HDC;
        let mut size: SIZE = core::mem::zeroed();

        hdc = UserGetDCEx(core::ptr::null_mut(), core::ptr::null_mut(), DCX_CACHE);
        if !hdc.is_null() {
            size.cx = IntGetCharDimensions(hdc, core::ptr::null_mut(), core::ptr::addr_of_mut!(size.cy) as PDWORD) as i32 ;
            if size.cx != 0 {
                units = ((size.cx as u16 as u32) | ((size.cy as u16 as u32) << 16)) as DWORD;
            }
            UserReleaseDC(core::ptr::null_mut(), hdc, 0u8 as i32 );
        }
    }
    units as LONG
}

fastcall_fn! {
unsafe fn MENU_CalcItemSize(
    hdc: HDC,
    lpitem: PITEM,
    Menu: PMENU,
    pwndOwner: PWND,
    orgX: INT,
    orgY: INT,
    menuBar: BOOL,
    textandbmp: BOOL,
) {
    let p: *mut WCHAR;
    let check_bitmap_width: UINT = UserGetSystemMetrics(SM_CXMENUCHECK) as UINT;
    let arrow_bitmap_width: UINT;
    let mut Rect: RECT = core::mem::zeroed();
    let mut itemheight: INT = 0;

    arrow_bitmap_width = (*pusi(gpsi)).oembmi[OBI_MNARROW as usize].cx as UINT;

    MenuCharSize.cx = IntGetCharDimensions(hdc, core::ptr::null_mut(), core::ptr::addr_of_mut!(MenuCharSize.cy) as PDWORD) as i32 ;

    RECTL_vSetRect(core::ptr::addr_of_mut!(Rect) as *mut _, orgX, orgY, orgX, orgY);

    if (*lpitem).fType & MF_OWNERDRAW != 0 {
        let mut mis: MEASUREITEMSTRUCT = core::mem::zeroed();
        mis.CtlType = ODT_MENU;
        mis.CtlID = 0;
        mis.itemID = (*lpitem).wID as UINT;
        mis.itemData = (*lpitem).dwItemData;
        mis.itemHeight = HIWORD(IntGetDialogBaseUnits() as u32) as UINT;
        mis.itemWidth = 0;
        co_IntSendMessage(
            UserHMGetHandle(pwndOwner) as HWND,
            WM_MEASUREITEM,
            0,
            core::ptr::addr_of_mut!(mis) as LPARAM,
        );
        Rect.right += (mis.itemWidth + 2 * MenuCharSize.cx as UINT) as LONG;
        if menuBar != 0 {
            Rect.bottom += UserGetSystemMetrics(SM_CYMENUSIZE);
        } else {
            Rect.bottom += mis.itemHeight as LONG;
        }

        (*lpitem).xItem = Rect.left as u32 ;
        (*lpitem).yItem = Rect.top as u32 ;
        (*lpitem).cxItem = Rect.right as u32 ;
        (*lpitem).cyItem = Rect.bottom as u32 ;

        return;
    }

    (*lpitem).xItem = orgX as u32 ;
    (*lpitem).yItem = orgY as u32 ;
    (*lpitem).cxItem = orgX as u32 ;
    (*lpitem).cyItem = orgY as u32 ;

    if (*lpitem).fType & MF_SEPARATOR != 0 {
        (*lpitem).cyItem += (UserGetSystemMetrics(SM_CYMENUSIZE) / 2) as u32;
        if menuBar == 0 {
            (*lpitem).cxItem += (arrow_bitmap_width as LONG + MenuCharSize.cx) as u32;
        }
        return;
    }

    (*lpitem).dxTab = 0;

    if !(*lpitem).hbmp.is_null() {
        let mut size: SIZE = core::mem::zeroed();

        if menuBar == 0 {
            MENU_GetBitmapItemSize(lpitem, core::ptr::addr_of_mut!(size), pwndOwner);
            (*lpitem).cxBmp = size.cx;
            (*lpitem).cyBmp = size.cy;
            (*Menu).cxTextAlign = max((*Menu).cxTextAlign, size.cx as u32 );
            (*lpitem).cxItem += (size.cx + 2) as u32;
            itemheight = size.cy + 2;

            if ((*Menu).fFlags & (MNS_STYLE_MASK as u32)) & (MNS_NOCHECK as u32) == 0 {
                (*lpitem).cxItem += 2 * check_bitmap_width as LONG as u32 ;
            }
            (*lpitem).cxItem += 4 + MenuCharSize.cx as u32 ;
            (*lpitem).dxTab = (*lpitem).cxItem;
            (*lpitem).cxItem += arrow_bitmap_width as LONG as u32 ;
        } else {
            MENU_GetBitmapItemSize(lpitem, core::ptr::addr_of_mut!(size), pwndOwner);
            (*lpitem).cxItem += size.cx as u32 ;
            if !(*lpitem).Xlpstr.is_null() {
                (*lpitem).cxItem += 2;
            }
            itemheight = size.cy;

            if (*lpitem).hbmp == HBMMENU_MBAR_MINIMIZE as HBITMAP
                || (*lpitem).hbmp == HBMMENU_MBAR_MINIMIZE_D as HBITMAP
            {
                (*lpitem).cxItem -= 1;
            }
        }
    } else if menuBar == 0 {
        if ((*Menu).fFlags & (MNS_STYLE_MASK as u32)) & (MNS_NOCHECK as u32) == 0 {
            (*lpitem).cxItem += check_bitmap_width as LONG as u32 ;
        }
        (*lpitem).cxItem += 4 + MenuCharSize.cx as u32 ;
        (*lpitem).dxTab = (*lpitem).cxItem;
        (*lpitem).cxItem += arrow_bitmap_width as LONG as u32 ;
    }

    if (*lpitem).fType & MF_SYSMENU == 0 && !(*lpitem).Xlpstr.is_null() {
        let mut hfontOld: HFONT = core::ptr::null_mut();
        let mut rc: RECT = core::mem::zeroed();
        let mut txtheight: LONG;
        let txtwidth: LONG;

        rc.left = (*lpitem).xItem as i32;
        rc.top = (*lpitem).yItem as i32;
        rc.right = (*lpitem).cxItem as i32;
        rc.bottom = (*lpitem).cyItem as i32;

        if (*lpitem).fState & MFS_DEFAULT != 0 {
            hfontOld = NtGdiSelectFont(hdc, ghMenuFontBold);
        }
        if menuBar != 0 {
            txtheight = DrawTextW(
                hdc,
                (*lpitem).Xlpstr,
                -1,
                core::ptr::addr_of_mut!(rc) as LPRECT,
                (DT_SINGLELINE | DT_CALCRECT) as UINT,
            );

            (*lpitem).cxItem += (rc.right - rc.left) as u32;
            itemheight = max(max(itemheight, txtheight), UserGetSystemMetrics(SM_CYMENU) - 1);

            (*lpitem).cxItem += 2 * MenuCharSize.cx as u32 ;
        } else {
            let pp = wcschr((*lpitem).Xlpstr, '\t' as WCHAR as i32);
            if !pp.is_null() {
                let mut tmprc: RECT = rc;
                let tmpheight: LONG;
                let n: i32 = (pp as usize - (*lpitem).Xlpstr as usize) as i32 / core::mem::size_of::<WCHAR>() as i32;
                txtheight = DrawTextW(
                    hdc,
                    (*lpitem).Xlpstr,
                    n,
                    core::ptr::addr_of_mut!(rc) as LPRECT,
                    (DT_SINGLELINE | DT_CALCRECT) as UINT,
                );
                let mut txtwidth_v = rc.right - rc.left;
                let pp2 = pp.add(1);
                tmpheight = DrawTextW(
                    hdc,
                    pp2,
                    -1,
                    core::ptr::addr_of_mut!(tmprc) as LPRECT,
                    (DT_SINGLELINE | DT_CALCRECT) as UINT,
                );
                (*lpitem).dxTab += txtwidth_v as u32 ;
                txtheight = max(txtheight, tmpheight);
                txtwidth_v += MenuCharSize.cx + tmprc.right - tmprc.left;
                (*lpitem).cxItem += 2 + txtwidth_v as u32 ;
                itemheight = max(itemheight, max(txtheight + 2, MenuCharSize.cy + 4));
            } else {
                txtheight = DrawTextW(
                    hdc,
                    (*lpitem).Xlpstr,
                    -1,
                    core::ptr::addr_of_mut!(rc) as LPRECT,
                    (DT_SINGLELINE | DT_CALCRECT) as UINT,
                );
                let txtwidth_v = rc.right - rc.left;
                (*lpitem).dxTab += txtwidth_v as u32 ;
                (*lpitem).cxItem += 2 + txtwidth_v as u32 ;
                itemheight = max(itemheight, max(txtheight + 2, MenuCharSize.cy + 4));
            }
            let _ = p;
        }
        if !hfontOld.is_null() {
            NtGdiSelectFont(hdc, hfontOld);
        }
    } else if menuBar != 0 {
        itemheight = max(itemheight, UserGetSystemMetrics(SM_CYMENU) - 1);
    }
    (*lpitem).cyItem += itemheight as u32 ;
    let _ = (textandbmp, check_bitmap_width);
}
}

unsafe fn MENU_GetMaxPopupHeight(lppop: PMENU) -> UINT {
    if (*lppop).cyMax != 0 {
        return (*lppop).cyMax;
    }
    (UserGetSystemMetrics(SM_CYSCREEN as ULONG) - UserGetSystemMetrics(SM_CYBORDER as ULONG)) as UINT
}

fastcall_fn! {
unsafe fn MENU_PopupMenuCalcSize(Menu: PMENU, WndOwner: PWND) {
    let mut lpitem: PITEM;
    let hdc: HDC;
    let mut start: i32;
    let mut i: i32 = 0;
    let mut orgX: i32;
    let mut orgY: i32;
    let mut maxX: i32;
    let mut maxTab: i32;
    let mut maxTabWidth: i32;
    let maxHeight: i32;
    let mut textandbmp: BOOL = 0;

    (*Menu).cxMenu = 0;
    (*Menu).cyMenu = 0;
    if (*Menu).cItems == 0 {
        return;
    }

    hdc = UserGetDCEx(core::ptr::null_mut(), core::ptr::null_mut(), DCX_CACHE);

    NtGdiSelectFont(hdc, ghMenuFont);

    start = 0;
    maxX = 0;

    (*Menu).cxTextAlign = 0;

    while start < (*Menu).cItems as i32 {
        lpitem = (*Menu).rgItems.offset(start as isize) as PITEM;
        lpitem = (*Menu).rgItems.offset(start as isize);
        orgX = maxX;
        if (*lpitem).fType & (MF_MENUBREAK | MF_MENUBARBREAK) != 0 {
            orgX += MENU_COL_SPACE as i32;
        }
        orgY = 0;

        maxTab = 0;
        maxTabWidth = 0;

        i = start;
        while i < (*Menu).cItems as i32 {
            if i != start && (*lpitem).fType & (MF_MENUBREAK | MF_MENUBARBREAK) != 0 {
                break;
            }

            MENU_CalcItemSize(hdc, lpitem, Menu, WndOwner, orgX, orgY, 0, textandbmp);
            maxX = core::cmp::max(maxX, (*lpitem).cxItem as i32 );
            orgY = (*lpitem).cyItem as i32 ;
            if (MF_STRING == ((*lpitem).fType & MENUITEMINFO_TYPE_MASK)) && (*lpitem).dxTab != 0 {
                maxTab = core::cmp::max(maxTab, (*lpitem).dxTab as i32 );
        maxTabWidth = core::cmp::max(maxTabWidth, ((*lpitem).cxItem - (*lpitem).dxTab) as i32);
            }
            if (*lpitem).Xlpstr != core::ptr::null_mut() && (*lpitem).hbmp != core::ptr::null_mut() {
                textandbmp = 1;
            }

            i += 1;
            lpitem = lpitem.offset(1);
        }

        maxX = core::cmp::max(maxX, maxTab + maxTabWidth);
        lpitem = (*Menu).rgItems.offset(start as isize);
        while start < i {
            (*lpitem).cxItem = maxX as u32 ;
            if (MF_STRING == ((*lpitem).fType & MENUITEMINFO_TYPE_MASK)) && (*lpitem).dxTab != 0 {
                (*lpitem).dxTab = maxTab as u32 ;
            }
            start += 1;
            lpitem = lpitem.offset(1);
        }
        (*Menu).cyMenu = core::cmp::max((*Menu).cyMenu, orgY as u32 );
    }

    (*Menu).cxMenu = maxX as u32 ;

    if textandbmp == 0 {
        (*Menu).cxTextAlign = 0;
    }

    maxHeight = MENU_GetMaxPopupHeight(Menu) as i32;
    (*Menu).iMaxTop = (*Menu).cyMenu as i32 ;
    if (*Menu).cyMenu >= (maxHeight as u32) {
        (*Menu).cyMenu = maxHeight as u32 ;
        (*Menu).set_dwArrowsOn(1);
    } else {
        (*Menu).set_dwArrowsOn(0);
    }
    UserReleaseDC(core::ptr::null_mut(), hdc, 0);
}
}

fastcall_fn! {
unsafe fn MENU_MenuBarCalcSize(hdc: HDC, lprect: LPRECT, lppop: PMENU, pwndOwner: PWND) {
    let mut lpitem: *mut ITEM;
    let mut start: UINT;
    let mut i: UINT;
    let mut helpPos: UINT;
    let mut orgX: INT;
    let mut orgY: INT;
    let mut maxY: INT;

    if lprect.is_null() || lppop.is_null() {
        return;
    }
    if (*lppop).cItems == 0 {
        return;
    }

        (*lppop).cxMenu = ((*lprect).right - (*lprect).left) as u32;
    (*lppop).cyMenu = 0;
    maxY = (*lprect).top;
    start = 0;
    helpPos = !0u32;
    (*lppop).cxTextAlign = 0;

    i = start;
    while start < (*lppop).cItems {
        lpitem = core::ptr::addr_of_mut!(*(*lppop).rgItems.offset(start as isize));
        orgX = (*lprect).left;
        orgY = maxY;

        i = start;
        while i < (*lppop).cItems {
            if (helpPos == !0u32) && ((*lpitem).fType & MF_RIGHTJUSTIFY) != 0 {
                helpPos = i;
            }
            if (i != start) && ((*lpitem).fType & (MF_MENUBREAK | MF_MENUBARBREAK)) != 0 {
                break;
            }

            MENU_CalcItemSize(hdc, lpitem, lppop, pwndOwner, orgX, orgY, 1, 0);

            if (*lpitem).cxItem > ((*lprect).right as u32) {
                if i != start {
                    break;
                } else {
                    (*lpitem).cxItem = (*lprect).right as u32 ;
                }
            }
            maxY = if maxY > ((*lpitem).cyItem as i32) { maxY } else { (*lpitem).cyItem as i32  };
            orgX = (*lpitem).cxItem as i32 ;

            i += 1;
            lpitem = lpitem.offset(1);
        }

        start = i;
    }

    (*lprect).bottom = maxY + 1;
        (*lppop).cyMenu = ((*lprect).bottom - (*lprect).top) as u32;

    if helpPos == !0u32 {
        return;
    }
    lpitem = core::ptr::addr_of_mut!(*(*lppop).rgItems.offset(((*lppop).cItems - 1) as isize));
    orgY = (*lpitem).yItem as i32 ;
    orgX = (*lprect).right;
    i = (*lppop).cItems - 1;
    loop {
        if i < helpPos {
            break;
        }
        if (*lpitem).yItem != (orgY as u32) {
            break;
        }
        if (*lpitem).cxItem >= (orgX as u32) {
            break;
        }
        (*lpitem).xItem = ((*lpitem).xItem as i32 + orgX - (*lpitem).cxItem as i32) as u32;
        (*lpitem).cxItem = orgX as u32 ;
        orgX = (*lpitem).xItem as i32 ;

        if i == 0 {
            break;
        }
        i -= 1;
        lpitem = lpitem.offset(-1);
    }
}
}

unsafe fn MENU_DrawScrollArrows(lppop: PMENU, hdc: HDC) {
    let arrow_bitmap_height: UINT;
    let mut rect: RECT = core::mem::zeroed();
    let mut Flags: UINT = 0;

    arrow_bitmap_height = (*pusi(gpsi)).oembmi[OBI_DNARROW as usize].cy as UINT;

    rect.left = 0;
    rect.top = 0;
    rect.right = (*lppop).cxMenu as i32;
    rect.bottom = arrow_bitmap_height as i32;
    FillRect(hdc, &mut rect, IntGetSysColorBrush(COLOR_MENU as i32) as HBRUSH );
    UITOOLS95_DrawFrameMenu(hdc, &mut rect, (if (*lppop).iTop != 0 { 0 } else { DFCS_INACTIVE }) | DFCS_MENUARROWUP);

    rect.top = ((*lppop).cyMenu - arrow_bitmap_height) as i32;
    rect.bottom = (*lppop).cyMenu as i32;
    FillRect(hdc, &mut rect, IntGetSysColorBrush(COLOR_MENU as i32) as HBRUSH );
    if !((*lppop).iTop < (*lppop).iMaxTop - (MENU_GetMaxPopupHeight(lppop) as i32 - 2 * arrow_bitmap_height as i32)) {
        Flags = DFCS_INACTIVE;
    }
    UITOOLS95_DrawFrameMenu(hdc, &mut rect, Flags | DFCS_MENUARROWDOWN);
}

fastcall_fn! {
unsafe fn MENU_DrawMenuItem(
    Wnd: PWND,
    Menu: PMENU,
    WndOwner: PWND,
    hdc: HDC,
    lpitem: PITEM,
    Height: UINT,
    menuBar: BOOL,
    odaction: UINT,
) {
    let mut rect: RECT = core::mem::zeroed();
    let Text: PWCHAR;
    let mut flat_menu: BOOL = 0;
    let bkgnd: core::ffi::c_int;
    let mut arrow_bitmap_width: UINT = 0;

    if menuBar == 0 {
        arrow_bitmap_width = (*pusi(gpsi)).oembmi[OBI_MNARROW as usize].cx as UINT;
    }

    if (*lpitem).fType & MF_SYSMENU != 0 {
        if (*Wnd).style & (WS_MINIMIZE as u32) == 0 {
            NC_GetInsideRect(Wnd, &mut rect);
            UserDrawSysMenuButton(
                Wnd,
                hdc,
                (&mut rect as *mut RECT).cast(),
                ((*lpitem).fState & (MF_HILITE | MF_MOUSESELECT)) as BOOL,
            );
        }
        return;
    }

    UserSystemParametersInfo(
        SPI_GETFLATMENU,
        0,
        core::ptr::addr_of_mut!(flat_menu) as *mut core::ffi::c_void,
        0,
    );
    bkgnd = if menuBar != 0 && flat_menu != 0 {
        COLOR_MENUBAR as core::ffi::c_int
    } else {
        COLOR_MENU as core::ffi::c_int
    };

    /* Setup colors */
    if (*lpitem).fState & MF_HILITE != 0 {
        if menuBar != 0 && flat_menu == 0 {
            IntGdiSetTextColor(hdc, IntGetSysColor(COLOR_MENUTEXT as INT));
            IntGdiSetBkColor(hdc, IntGetSysColor(COLOR_MENU as INT));
        } else {
            if (*lpitem).fState & MF_GRAYED != 0 {
                IntGdiSetTextColor(hdc, IntGetSysColor(COLOR_GRAYTEXT as INT));
            } else {
                IntGdiSetTextColor(hdc, IntGetSysColor(COLOR_HIGHLIGHTTEXT as INT));
            }
            IntGdiSetBkColor(hdc, IntGetSysColor(COLOR_HIGHLIGHT as INT));
        }
    } else {
        if (*lpitem).fState & MF_GRAYED != 0 {
            IntGdiSetTextColor(hdc, IntGetSysColor(COLOR_GRAYTEXT as INT));
        } else {
            IntGdiSetTextColor(hdc, IntGetSysColor(COLOR_MENUTEXT as INT));
        }
        IntGdiSetBkColor(hdc, IntGetSysColor(bkgnd as INT));
    }

    rect.left = (*lpitem).xItem as i32;
    rect.top = (*lpitem).yItem as i32;
    rect.right = (*lpitem).cxItem as i32;
    rect.bottom = (*lpitem).cyItem as i32;

    MENU_AdjustMenuItemRect(Menu, &mut rect as *mut RECT as PRECTL);

    if (*lpitem).fType & MF_OWNERDRAW != 0 {
        let mut dis: DRAWITEMSTRUCT = core::mem::zeroed();
        let old_bk: COLORREF;
        let old_text: COLORREF;

        dis.CtlType = ODT_MENU;
        dis.CtlID = 0;
        dis.itemID = (*lpitem).wID;
        dis.itemData = (*lpitem).dwItemData as ULONG_PTR;
        dis.itemState = 0;
        if (*lpitem).fState & MF_CHECKED != 0 {
            dis.itemState |= ODS_CHECKED;
        }
        if (*lpitem).fState & MF_DEFAULT != 0 {
            dis.itemState |= ODS_DEFAULT;
        }
        if (*lpitem).fState & MF_DISABLED != 0 {
            dis.itemState |= ODS_DISABLED;
        }
        if (*lpitem).fState & MF_GRAYED != 0 {
            dis.itemState |= ODS_GRAYED | ODS_DISABLED;
        }
        if (*lpitem).fState & MF_HILITE != 0 {
            dis.itemState |= ODS_SELECTED;
        }
        if (*Menu).fFlags & MNF_UNDERLINE as u32 == 0 {
            dis.itemState |= ODS_NOACCEL;
        }
        if (*Menu).fFlags & MNF_INACTIVE as u32 != 0 {
            dis.itemState |= ODS_INACTIVE;
        }
        dis.itemAction = odaction;
        dis.hwndItem = UserHMGetHandle(Menu) as HWND;
        dis.hDC = hdc;
        dis.rcItem = core::mem::transmute_copy(&rect);
        old_bk = GreGetBkColor(hdc);
        old_text = GreGetTextColor(hdc);
        co_IntSendMessage(
            UserHMGetHandle(WndOwner) as HWND,
            WM_DRAWITEM,
            0,
            &mut dis as *mut DRAWITEMSTRUCT as LPARAM,
        );
        IntGdiSetBkColor(hdc, old_bk);
        IntGdiSetTextColor(hdc, old_text);
        /* Draw the popup-menu arrow */
        if menuBar == 0 && !(*lpitem).spSubMenu.is_null() {
            let mut rectTemp: RECT = core::mem::zeroed();
            core::ptr::copy_nonoverlapping(&rect, &mut rectTemp, 1);
            rectTemp.left = rectTemp.right - UserGetSystemMetrics(SM_CXMENUCHECK);
            UITOOLS95_DrawFrameMenu(hdc, &mut rectTemp, DFCS_MENUARROW);
        }
        return;
    }

    if menuBar != 0 && (*lpitem).fType & MF_SEPARATOR != 0 {
        return;
    }

    if (*lpitem).fState & MF_HILITE != 0 {
        if flat_menu != 0 {
            RECTL_vInflateRect(&mut rect, -1, -1);
            FillRect(hdc, &mut rect, IntGetSysColorBrush(COLOR_MENUHILIGHT as INT) as HBRUSH );
            RECTL_vInflateRect(&mut rect, 1, 1);
            FrameRect(hdc, &mut rect, IntGetSysColorBrush(COLOR_HIGHLIGHT as INT) as HBRUSH );
        } else {
            if menuBar != 0 {
                FillRect(hdc, &mut rect, IntGetSysColorBrush(COLOR_MENU as INT) as HBRUSH );
                DrawEdge(hdc, &mut rect, BDR_SUNKENOUTER, BF_RECT);
            } else {
                FillRect(hdc, &mut rect, IntGetSysColorBrush(COLOR_HIGHLIGHT as INT) as HBRUSH );
            }
        }
    } else {
        FillRect(hdc, &mut rect, IntGetSysColorBrush(bkgnd as INT) as HBRUSH );
    }

    IntGdiSetBkMode(hdc, TRANSPARENT as core::ffi::c_int);

    /* vertical separator */
    if menuBar == 0 && (*lpitem).fType & MF_MENUBARBREAK != 0 {
        let oldPen: HPEN;
        let mut rc: RECT = rect;

        rc.left -= 3;
        rc.top = 3;
        rc.bottom = Height as core::ffi::c_long - 3;
        if flat_menu != 0 {
            oldPen = NtGdiSelectPen(hdc, NtGdiGetStockObject(DC_PEN as INT) as HPEN );
            IntSetDCPenColor(hdc, IntGetSysColor(COLOR_BTNSHADOW as INT));
            GreMoveTo(hdc, rc.left, rc.top, core::ptr::null_mut());
            NtGdiLineTo(hdc, rc.left, rc.bottom);
            NtGdiSelectPen(hdc, oldPen);
        } else {
            DrawEdge(hdc, &mut rc, EDGE_ETCHED, BF_LEFT);
        }
    }

    /* horizontal separator */
    if (*lpitem).fType & MF_SEPARATOR != 0 {
        let oldPen: HPEN;
        let mut rc: RECT = rect;

        rc.left += 1;
        rc.right -= 1;
        rc.top = (rc.top + rc.bottom) / 2 - 1;
        if flat_menu != 0 {
            oldPen = NtGdiSelectPen(hdc, NtGdiGetStockObject(DC_PEN as INT) as HPEN );
            IntSetDCPenColor(hdc, IntGetSysColor(COLOR_BTNSHADOW as INT));
            GreMoveTo(hdc, rc.left, rc.top, core::ptr::null_mut());
            NtGdiLineTo(hdc, rc.right, rc.top);
            NtGdiSelectPen(hdc, oldPen);
        } else {
            DrawEdge(hdc, &mut rc, EDGE_ETCHED, BF_TOP);
        }
        return;
    }

    if menuBar == 0 {
        let bm: HBITMAP;
        let y: INT = rect.top + rect.bottom;
        let rc: RECT = rect;
        let mut checked: BOOL = 0;
        let check_bitmap_width: UINT = UserGetSystemMetrics(SM_CXMENUCHECK) as UINT;
        let check_bitmap_height: UINT = UserGetSystemMetrics(SM_CYMENUCHECK) as UINT;

        if (*Menu).fFlags & (MNS_STYLE_MASK as u32) & (MNS_NOCHECK as u32) == 0 {
            bm = if (*lpitem).fState & MF_CHECKED != 0 {
                (*lpitem).hbmpChecked as HBITMAP
            } else {
                (*lpitem).hbmpUnchecked as HBITMAP
            };
            if !bm.is_null() {
                let hdcMem: HDC = NtGdiCreateCompatibleDC(hdc);

                NtGdiSelectBitmap(hdcMem, bm);
                NtGdiBitBlt(
                    hdc,
                    rc.left,
                    (y - check_bitmap_height as INT) / 2,
                    check_bitmap_width as INT,
                    check_bitmap_height as INT,
                    hdcMem,
                    0,
                    0,
                    SRCCOPY,
                    CLR_INVALID,
                    0,
                );
                IntGdiDeleteDC(hdcMem, 0);
                checked = 1;
            } else if (*lpitem).fState & MF_CHECKED != 0 {
                let mut r: RECT = rect;
                r.right = r.left + check_bitmap_width as INT;
                UITOOLS95_DrawFrameMenu(
                    hdc,
                    &mut r,
                    if (*lpitem).fType & MFT_RADIOCHECK != 0 {
                        DFCS_MENUBULLET
                    } else {
                        DFCS_MENUCHECK
                    },
                );
                checked = 1;
            }
        }
        if !(*lpitem).hbmp.is_null() {
            let mut bmpRect: RECT = rect;
            if (*Menu).fFlags & (MNS_STYLE_MASK as u32) & (MNS_CHECKORBMP as u32) == 0
                && (*Menu).fFlags & (MNS_STYLE_MASK as u32) & (MNS_NOCHECK as u32) == 0
            {
                bmpRect.left += check_bitmap_width as INT + 2;
            }
            if !(checked != 0
                && (*Menu).fFlags & (MNS_STYLE_MASK as u32) & (MNS_CHECKORBMP as u32) != 0)
            {
                bmpRect.right = bmpRect.left + (*lpitem).cxBmp;
                MENU_DrawBitmapItem(hdc, lpitem, &bmpRect, Menu, WndOwner, odaction, menuBar);
            }
        }
        /* Draw the popup-menu arrow */
        if !(*lpitem).spSubMenu.is_null() {
            let mut rectTemp: RECT = core::mem::zeroed();
            core::ptr::copy_nonoverlapping(&rect, &mut rectTemp, 1);
            rectTemp.left = rectTemp.right - check_bitmap_width as INT;
            UITOOLS95_DrawFrameMenu(hdc, &mut rectTemp, DFCS_MENUARROW);
        }
        rect.left += 4;
        if (*Menu).fFlags & (MNS_STYLE_MASK as u32) & (MNS_NOCHECK as u32) == 0 {
            rect.left += check_bitmap_width as INT;
        }
        rect.right -= arrow_bitmap_width as INT;
    } else if !(*lpitem).hbmp.is_null() {
        MENU_DrawBitmapItem(hdc, lpitem, &rect, Menu, WndOwner, odaction, menuBar);
    }

    /* process text if present */
    if !(*lpitem).Xlpstr.is_null() {
        let mut i: core::ffi::c_int = 0;
        let mut hfontOld: HFONT = core::ptr::null_mut();

        let mut uFormat: UINT = if menuBar != 0 {
            DT_CENTER | DT_VCENTER | DT_SINGLELINE
        } else {
            DT_LEFT | DT_VCENTER | DT_SINGLELINE
        };

        if (*Menu).fFlags & (MNS_STYLE_MASK as u32) & (MNS_CHECKORBMP as u32) != 0 {
            rect.left += max(
                0,
                (*Menu).cxTextAlign as core::ffi::c_int
                    - UserGetSystemMetrics(SM_CXMENUCHECK),
            );
        } else {
            rect.left += (*Menu).cxTextAlign as INT;
        }

        if (*lpitem).fState & MFS_DEFAULT != 0 {
            hfontOld = NtGdiSelectFont(hdc, ghMenuFontBold);
        }

        if menuBar != 0 {
            if !(*lpitem).hbmp.is_null() {
                rect.left += (*lpitem).cxBmp;
            }
            if (*lpitem).hbmp != HBMMENU_CALLBACK {
                rect.left += MenuCharSize.cx;
            }
            rect.right -= MenuCharSize.cx;
        }

        Text = (*lpitem).Xlpstr;
        if !Text.is_null() {
            i = 0;
            while *Text.offset(i as isize) != 0 {
                if *Text.offset(i as isize) == '\t' as u16
                    || *Text.offset(i as isize) == '\u{8}' as u16
                {
                    break;
                }
                i += 1;
            }
        }

        if menuBar != 0
            && flat_menu == 0
            && ((*lpitem).fState & (MF_HILITE | MF_GRAYED)) == MF_HILITE
        {
            RECTL_vOffsetRect(&mut rect, 1, 1);
        }

        if menuBar == 0 {
            rect.bottom -= 1;
        }

        if (*lpitem).fState & MF_GRAYED != 0 {
            if (*lpitem).fState & MF_HILITE == 0 {
                rect.left += 1;
                rect.top += 1;
                rect.right += 1;
                rect.bottom += 1;
                IntGdiSetTextColor(hdc, IntGetSysColor(COLOR_BTNHIGHLIGHT as INT));
                DrawTextW(hdc, Text, i, &mut rect, uFormat);
                rect.left -= 1;
                rect.top -= 1;
                rect.right -= 1;
                rect.bottom -= 1;
            }
            IntGdiSetTextColor(hdc, IntGetSysColor(COLOR_BTNSHADOW as INT));
        }
        DrawTextW(hdc, Text, i, &mut rect, uFormat);

        /* paint the shortcut text */
        if menuBar == 0 && *Text.offset(i as isize) != 0 {
            if '\t' as u16 == *Text.offset(i as isize) {
                rect.left = (*lpitem).dxTab as i32;
                uFormat = DT_LEFT | DT_VCENTER | DT_SINGLELINE;
            } else {
                rect.right = (*lpitem).dxTab as i32;
                uFormat = DT_RIGHT | DT_VCENTER | DT_SINGLELINE;
            }

            if (*lpitem).fState & MF_GRAYED != 0 {
                if (*lpitem).fState & MF_HILITE == 0 {
                    rect.left += 1;
                    rect.top += 1;
                    rect.right += 1;
                    rect.bottom += 1;
                    IntGdiSetTextColor(hdc, IntGetSysColor(COLOR_BTNHIGHLIGHT as INT));
                    DrawTextW(hdc, Text.offset(i as isize + 1), -1, &mut rect, uFormat);
                    rect.left -= 1;
                    rect.top -= 1;
                    rect.right -= 1;
                    rect.bottom -= 1;
                }
                IntGdiSetTextColor(hdc, IntGetSysColor(COLOR_BTNSHADOW as INT));
            }
            DrawTextW(hdc, Text.offset(i as isize + 1), -1, &mut rect, uFormat);
        }

        if menuBar == 0 {
            rect.bottom += 1;
        }

        if menuBar != 0
            && flat_menu == 0
            && ((*lpitem).fState & (MF_HILITE | MF_GRAYED)) == MF_HILITE
        {
            RECTL_vOffsetRect(&mut rect, -1, -1);
        }

        if !hfontOld.is_null() {
            NtGdiSelectFont(hdc, hfontOld);
        }
    }
}
}

fastcall_fn! {
unsafe fn MENU_DrawPopupMenu(wnd: PWND, hdc: HDC, menu: PMENU) {
    let mut hPrevBrush: HBRUSH = core::ptr::null_mut();
    let mut brush: HBRUSH = IntGetSysColorBrush(COLOR_MENU as i32) as HBRUSH ;
    let mut rect: RECT = core::mem::zeroed();

    IntGetClientRect(wnd, (&mut rect as *mut RECT).cast());

    if !menu.is_null() && !(*menu).hbrBack.is_null() {
        brush = (*menu).hbrBack;
    }

    hPrevBrush = NtGdiSelectBrush(hdc, brush);
    if !hPrevBrush.is_null() && !NtGdiSelectFont(hdc, ghMenuFont).is_null() {
        FillRect(hdc, &mut rect, brush);

        let hPrevPen: HPEN = NtGdiSelectPen(hdc, NtGdiGetStockObject(NULL_PEN as i32) as HPEN );
        if !hPrevPen.is_null() {
            if !menu.is_null() && (*menu).cItems != 0 {
                let mut item: *mut ITEM = (*menu).rgItems;
                let mut u: UINT = (*menu).cItems;
                while u > 0 {
                    MENU_DrawMenuItem(
                        wnd,
                        menu,
                        (*menu).spwndNotify,
                        hdc,
                        item,
                        (*menu).cyMenu as UINT,
                        0,
                        ODA_DRAWENTIRE,
                    );
                    u -= 1;
                    item = item.add(1);
                }
                if (*menu).dwArrowsOn() != 0 {
                    MENU_DrawScrollArrows(menu, hdc);
                }
            }
        } else {
            NtGdiSelectBrush(hdc, hPrevBrush);
        }
    }
}
}

#[no_mangle]
unsafe extern "C" fn MENU_IsMenuActive() -> PWND {
    ValidateHwndNoErr(top_popup)
}

#[no_mangle]
unsafe extern "C" fn MENU_EndMenu(pwnd: PWND) {
    let mut menu: PMENU = core::ptr::null_mut();
    menu = UserGetMenuObject(top_popup_hmenu);
    if !menu.is_null()
        && (UserHMGetHandle(pwnd as *mut core::ffi::c_void) == ((*menu).hWnd as *mut core::ffi::c_void)
            || pwnd == (*menu).spwndNotify)
    {
        if fInsideMenuLoop != 0 && !top_popup.is_null() {
            fInsideMenuLoop = 0;

            if fInEndMenu != 0 {
                ERR!("Already in End loop\n");
                return;
            }

            fInEndMenu = 1;
            UserPostMessage(top_popup, WM_CANCELMODE, 0, 0);
        }
    }
}

unsafe extern "system" fn IntDrawMenuBarTemp(
    pWnd: PWND,
    hDC: HDC,
    Rect: LPRECT,
    mut pMenu: PMENU,
    mut Font: HFONT,
) -> DWORD {
    let mut i: UINT;
    let mut FontOld: HFONT = core::ptr::null_mut();
    let mut flat_menu: BOOL = 0;

    UserSystemParametersInfo(
        SPI_GETFLATMENU,
        0,
        core::ptr::addr_of_mut!(flat_menu) as *mut core::ffi::c_void,
        0,
    );

    if pMenu.is_null() {
        pMenu = UserGetMenuObject(UlongToHandle((*pWnd).IDMenu as u32) as HMENU);
    }

    if Font.is_null() {
        Font = ghMenuFont;
    }

    if Rect.is_null() || pMenu.is_null() {
        return UserGetSystemMetrics(SM_CYMENU) as DWORD;
    }

    FontOld = NtGdiSelectFont(hDC, Font);

    if (*pMenu).cyMenu == 0 {
        MENU_MenuBarCalcSize(hDC, Rect, pMenu, pWnd);
    }

    (*Rect).bottom = (*Rect).top + ((*pMenu).cyMenu as i32);

    FillRect(
        hDC,
        Rect,
        IntGetSysColorBrush(if flat_menu != 0 {
            COLOR_MENUBAR as i32
        } else {
            COLOR_MENU as i32
        }) as HBRUSH,
    );

    NtGdiSelectPen(hDC, NtGdiGetStockObject(DC_PEN as i32) as HPEN);
    IntSetDCPenColor(hDC, IntGetSysColor(COLOR_3DFACE as i32));
    GreMoveTo(hDC, (*Rect).left, (*Rect).bottom - 1, core::ptr::null_mut());
    NtGdiLineTo(hDC, (*Rect).right, (*Rect).bottom - 1);

    if (*pMenu).cItems == 0 {
        NtGdiSelectFont(hDC, FontOld);
        return UserGetSystemMetrics(SM_CYMENU) as DWORD;
    }

    i = 0;
    while i < (*pMenu).cItems {
        MENU_DrawMenuItem(
            pWnd,
            pMenu,
            pWnd,
            hDC,
            core::ptr::addr_of_mut!(*(*pMenu).rgItems.offset(i as isize)),
            (*pMenu).cyMenu as UINT,
            1,
            ODA_DRAWENTIRE,
        );
        i += 1;
    }

    NtGdiSelectFont(hDC, FontOld);

    (*pMenu).cyMenu as DWORD
}

#[no_mangle]
unsafe extern "C" fn MENU_DrawMenuBar(
    hDC: HDC,
    lprect: LPRECT,
    pWnd: PWND,
    suppress_draw: BOOL,
) -> UINT {
    let mut hfontOld: HFONT = 0 as HFONT;
    let lppop: PMENU = UserGetMenuObject(UlongToHandle((*pWnd).IDMenu as u32) as HMENU);

    if lppop.is_null() {
        return 0;
    }

    if lprect.is_null() {
        return UserGetSystemMetrics(SM_CYMENU) as UINT;
    }

    if suppress_draw != 0 {
        hfontOld = NtGdiSelectFont(hDC, ghMenuFont);

        MENU_MenuBarCalcSize(hDC, lprect, lppop, pWnd);

        (*lprect).bottom = (*lprect).top + ((*lppop).cyMenu as i32);

        if !hfontOld.is_null() {
            NtGdiSelectFont(hDC, hfontOld);
        }

        (*lppop).cyMenu as UINT
    } else {
        IntDrawMenuBarTemp(pWnd, hDC, lprect, lppop, core::ptr::null_mut()) as UINT
    }
}


unsafe fn MENU_InitPopup(pWndOwner: PWND, menu: PMENU, flags: UINT) -> BOOL {
    let pWndCreated: PWND;
    let pPopupMenu: PPOPUPMENU;
    let mut Cs: CREATESTRUCTW = core::mem::zeroed();
    let mut WindowName: LARGE_STRING = core::mem::zeroed();
    let mut ClassName: _UNICODE_STRING = core::mem::zeroed();
    let mut ex_style: DWORD = WS_EX_PALETTEWINDOW | WS_EX_DLGMODALFRAME;

    (*menu).spwndNotify = pWndOwner;

    if (flags & TPM_LAYOUTRTL) != 0 || ((*pWndOwner).ExStyle & WS_EX_LAYOUTRTL) != 0 {
        ex_style |= WS_EX_LAYOUTRTL;
    }

    ClassName.Buffer = WC_MENU as *mut _;
    ClassName.Length = 0;

    RtlZeroMemory(
        core::ptr::addr_of_mut!(WindowName) as *mut _,
        core::mem::size_of::<LARGE_STRING>() as _,
    );
    RtlZeroMemory(
        core::ptr::addr_of_mut!(Cs) as *mut _,
        core::mem::size_of::<CREATESTRUCTW>() as _,
    );
    Cs.style = (WS_POPUP | WS_CLIPSIBLINGS | WS_BORDER) as _;
    Cs.dwExStyle = ex_style;
    Cs.hInstance = hModClient as _;
    Cs.lpszName = core::ptr::addr_of!(WindowName) as LPCWSTR;
    Cs.lpszClass = core::ptr::addr_of!(ClassName) as LPCWSTR;
    Cs.lpCreateParams = UserHMGetHandle(menu) as *mut _;
    Cs.hwndParent = UserHMGetHandle(pWndOwner) as HWND;

    pWndCreated = co_UserCreateWindowEx(
        core::ptr::addr_of_mut!(Cs),
        core::ptr::addr_of_mut!(ClassName),
        core::ptr::addr_of_mut!(WindowName),
        core::ptr::null_mut(),
        WINVER as _,
    );

    if pWndCreated.is_null() {
        return 0u8 as BOOL;
    }

    (*menu).hWnd = UserHMGetHandle(pWndCreated) as HWND;

    pPopupMenu = (*(pWndCreated as PMENUWND)).ppopupmenu;

    (*pPopupMenu).spwndActivePopup = pWndCreated;
    (*pPopupMenu).spwndNotify = pWndOwner;

    (*pPopupMenu).set_fIsTrackPopup(((flags & TPM_POPUPMENU) != 0) as _);
    (*pPopupMenu).set_fIsSysMenu(((flags & TPM_SYSTEM_MENU) != 0) as _);
    (*pPopupMenu).set_fNoNotify(((flags & TPM_NONOTIFY) != 0) as _);
    (*pPopupMenu).set_fRightButton(((flags & TPM_RIGHTBUTTON) != 0) as _);
    (*pPopupMenu).set_fSynchronous(((flags & TPM_RETURNCMD) != 0) as _);

    if (*pPopupMenu).fRightButton() != 0 {
        (*pPopupMenu).set_fFirstClick(((UserGetKeyState(VK_RBUTTON as _) as i32 & 0x8000) != 0) as _);
    } else {
        (*pPopupMenu).set_fFirstClick(((UserGetKeyState(VK_LBUTTON as _) as i32 & 0x8000) != 0) as _);
    }

    if (*pusi(gpsi)).aiSysMet[SM_MENUDROPALIGNMENT as usize] != 0
        || ((*menu).fFlags & MNF_RTOL) != 0
    {
        (*pPopupMenu).set_fDroppedLeft(1);
    }

    1u8 as BOOL
}

unsafe fn DebugRect(rectl: *const RECT, color: COLORREF) {

    let brush: HBRUSH;
    let mut rr: RECT;
    let hdc: HDC;

    if rectl.is_null() {
        return;
    }

    hdc = UserGetDCEx(core::ptr::null_mut(), 0 as usize as *mut core::ffi::c_void , DCX_USESTYLE);

    brush = IntGdiCreateSolidBrush(color);

    rr = *rectl;
    RECTL_vInflateRect(core::ptr::addr_of_mut!(rr), 1, 1);
    FrameRect(hdc, rectl, brush);
    FrameRect(hdc, core::ptr::addr_of!(rr), brush);

    NtGdiDeleteObjectApp(brush as *mut core::ffi::c_void );
    UserReleaseDC(core::ptr::null_mut(), hdc, 1u8 as i32 );
}

unsafe fn DebugPoint(x: INT, y: INT, color: COLORREF) {
    let r1 = RECT { left: x - 10, top: y, right: x + 10, bottom: y };
    let r2 = RECT { left: x, top: y - 10, right: x, bottom: y + 10 };
    DebugRect(&r1, color);
    DebugRect(&r2, color);
}

unsafe fn RECTL_Intersect(pRect: *const RECT, x: INT, y: INT, width: UINT, height: UINT) -> BOOL {
    let mut other: RECT = RECT {
        left: x,
        top: y,
        right: x + width as INT,
        bottom: y + height as INT,
    };
    let mut dum: RECT = core::mem::zeroed();

    RECTL_bIntersectRect((&mut dum as *mut RECT).cast(), pRect.cast(), (&mut other as *mut RECT).cast())
}

unsafe fn MENU_MoveRect(flags: UINT, x: *mut INT, y: *mut INT, width: INT, height: INT, pExclude: *const RECT, monitor: PMONITOR) -> BOOL {
    if flags & TPM_VERTICAL != 0 {
        if flags & TPM_BOTTOMALIGN != 0 {
            if (*pExclude).top - height >= (*monitor).rcMonitor.top {
                *y = (*pExclude).top - height;
                return 1;
            }
        } else {
            if (*pExclude).bottom + height < (*monitor).rcMonitor.bottom {
                *y = (*pExclude).bottom;
                return 1;
            }
        }
    } else {
        if flags & TPM_RIGHTALIGN != 0 {
            if (*pExclude).left - width >= (*monitor).rcMonitor.left {
                *x = (*pExclude).left - width;
                return 1;
            }
        } else {
            if (*pExclude).right + width < (*monitor).rcMonitor.right {
                *x = (*pExclude).right;
                return 1;
            }
        }
    }
    0
}

fastcall_fn! {
unsafe fn MENU_ShowPopup(pwndOwner: PWND, menu: PMENU, id: UINT, mut flags: UINT, mut x: INT, mut y: INT, pExclude: *const RECT) -> BOOL {
    let width: INT;
    let height: INT;
    let mut ptx: POINT = core::mem::zeroed();
    let monitor: PMONITOR;
    let pWnd: PWND;
    let mut Ref: USER_REFERENCE_ENTRY = core::mem::zeroed();
    let bIsPopup: BOOL = ((flags & TPM_POPUPMENU) != 0) as BOOL;

    if (*menu).iItem != NO_SELECTED_ITEM as i32 {
        (*(*menu).rgItems.offset((*menu).iItem as isize)).fState &= !(MF_HILITE | MF_MOUSESELECT) as i32 as u32 ;
        (*menu).iItem = NO_SELECTED_ITEM as i32;
    }

    (*menu).set_dwArrowsOn(0);
    MENU_PopupMenuCalcSize(menu, pwndOwner);

    width = (*menu).cxMenu as INT + UserGetSystemMetrics(SM_CXDLGFRAME as ULONG) as INT * 2;
    height = (*menu).cyMenu as INT + UserGetSystemMetrics(SM_CYDLGFRAME as ULONG) as INT * 2;

    if flags & TPM_LAYOUTRTL != 0 {
        flags ^= TPM_RIGHTALIGN;
    }

    if flags & TPM_RIGHTALIGN != 0 {
        x -= width;
    }
    if flags & TPM_CENTERALIGN != 0 {
        x -= width / 2;
    }

    if flags & TPM_BOTTOMALIGN != 0 {
        y -= height;
    }
    if flags & TPM_VCENTERALIGN != 0 {
        y -= height / 2;
    }

    ptx.x = x;
    ptx.y = y;

    monitor = UserMonitorFromPoint(core::mem::transmute(ptx), MONITOR_DEFAULTTONEAREST as u32);

    if x + width > (*monitor).rcMonitor.right {
        x = (*monitor).rcMonitor.right - width;
    }

    if x < (*monitor).rcMonitor.left {
        x = 0;

        if x < (*monitor).rcMonitor.left || x >= (*monitor).rcMonitor.right || bIsPopup != 0 {
            x = (*monitor).rcMonitor.left;
        }
    }

    if y < (*monitor).rcMonitor.top {
        y += height;

        if y < (*monitor).rcMonitor.top || y >= (*monitor).rcMonitor.bottom || bIsPopup != 0 {
            y = (*monitor).rcMonitor.top;
        }
    }

    if y + height > (*monitor).rcMonitor.bottom {
        if (y - height) < (*monitor).rcMonitor.top || y >= (*monitor).rcMonitor.bottom {
            y = (*monitor).rcMonitor.bottom - height;
        } else {
            let adjHgt: INT = y + UserGetSystemMetrics(SM_CYMENUSIZE as ULONG) +
                2 * UserGetSystemMetrics(SM_CYDLGFRAME as ULONG);
            if adjHgt >= (*monitor).rcMonitor.bottom {
                y -= height;
            } else {
                y = adjHgt - height;
            }
        }
    }

    if !pExclude.is_null() {
        let mut Cleaned: RECT = core::mem::zeroed();

        if RECTL_bIntersectRect((&mut Cleaned as *mut RECT).cast(), pExclude.cast(), core::ptr::addr_of!((*monitor).rcMonitor).cast()) != 0 &&
            RECTL_Intersect(&Cleaned, x, y, width as u32, height as u32) != 0 {
            let flag_mods: [UINT; 4] = [
                0,
                TPM_BOTTOMALIGN | TPM_RIGHTALIGN,
                TPM_VERTICAL,
                TPM_BOTTOMALIGN | TPM_RIGHTALIGN | TPM_VERTICAL,
            ];

            let mut n: UINT = 0;
            while (n as usize) < flag_mods.len() {
                let mut tx: INT = x;
                let mut ty: INT = y;

                if MENU_MoveRect(flags ^ flag_mods[n as usize], &mut tx, &mut ty, width, height, &Cleaned, monitor) != 0 &&
                    RECTL_Intersect(&Cleaned, tx, ty, width as u32, height as u32) == 0 {
                    x = tx;
                    y = ty;
                    break;
                }
                n += 1;
            }
        }
    }

    pWnd = ValidateHwndNoErr((*menu).hWnd);

    if pWnd.is_null() {
        ERR!("menu->hWnd bad hwnd %p\n", (*menu).hWnd);
        return 0;
    }

    if top_popup.is_null() {
        top_popup = (*menu).hWnd;
        top_popup_hmenu = UserHMGetHandle(menu as *mut core::ffi::c_void) as HMENU;
    }

    UserRefObjectCo(pWnd, &mut Ref);
    co_WinPosSetWindowPos(pWnd, HWND_TOPMOST, x, y, width, height, (SWP_SHOWWINDOW | SWP_NOACTIVATE) as u32);

    co_IntUpdateWindows(pWnd, RDW_ALLCHILDREN, 0);

    IntNotifyWinEvent(EVENT_SYSTEM_MENUPOPUPSTART as u32, pWnd, OBJID_CLIENT as i32, CHILDID_SELF as i32, 0);
    UserDerefObjectCo(pWnd);

    1
}
}

unsafe fn MENU_EnsureMenuItemVisible(lppop: PMENU, wIndex: UINT, hdc: HDC) {
    let mut Ref: USER_REFERENCE_ENTRY = core::mem::zeroed();
    if (*lppop).dwArrowsOn() != 0 {
        let item: *mut ITEM = core::ptr::addr_of_mut!((*(*lppop).rgItems.offset(wIndex as isize)));
        let mut nMaxHeight: UINT = MENU_GetMaxPopupHeight(lppop);
        let nOldPos: UINT = (*lppop).iTop as UINT;
        let mut rc: RECT = core::mem::zeroed();
        let arrow_bitmap_height: UINT;
        let pWnd: PWND = ValidateHwndNoErr((*lppop).hWnd);

        IntGetClientRect(pWnd, &mut rc as *mut RECT as PRECTL);

        arrow_bitmap_height = (*pusi(gpsi)).oembmi[OBI_DNARROW as usize].cy as UINT;

        rc.top += arrow_bitmap_height as i32;
        rc.bottom -= arrow_bitmap_height as i32;

        nMaxHeight -= UserGetSystemMetrics(SM_CYBORDER) as UINT + 2 * arrow_bitmap_height;
        UserRefObjectCo(pWnd, &mut Ref);
        if (*item).cyItem as UINT > (*lppop).iTop as UINT + nMaxHeight {
            (*lppop).iTop = ((*item).cyItem as UINT - nMaxHeight) as i32;
            IntScrollWindow(pWnd, 0, (nOldPos as i32 - (*lppop).iTop) as i32, (&mut rc as *mut RECT).cast(), (&mut rc as *mut RECT).cast());
            MENU_DrawScrollArrows(lppop, hdc);
        } else if ((*item).yItem) < ((*lppop).iTop as u32) {
            (*lppop).iTop = (*item).yItem as i32 ;
            IntScrollWindow(pWnd, 0, (nOldPos as i32 - (*lppop).iTop) as i32, (&mut rc as *mut RECT).cast(), (&mut rc as *mut RECT).cast());
            MENU_DrawScrollArrows(lppop, hdc);
        }
        UserDerefObjectCo(pWnd);
    }
}

fastcall_fn! {
unsafe fn MENU_SelectItem(
    pwndOwner: PWND,
    menu: PMENU,
    wIndex: UINT,
    sendMenuSelect: BOOL,
    topmenu: PMENU,
) {
    let hdc: HDC;
    let pWnd: PWND;

    if menu.is_null() || (*menu).cItems == 0 {
        return;
    }

    pWnd = ValidateHwndNoErr((*menu).hWnd);

    if pWnd.is_null() {
        return;
    }

    if (*menu).iItem == wIndex as i32 {
        return;
    }

    if (*menu).fFlags & MNF_POPUP != 0 {
        hdc = UserGetDCEx(pWnd, 0 as _, DCX_USESTYLE);
    } else {
        hdc = UserGetDCEx(pWnd, 0 as _, DCX_CACHE | DCX_WINDOW);
    }

    if top_popup.is_null() {
        top_popup = (*menu).hWnd;
        top_popup_hmenu = UserHMGetHandle(menu) as HMENU ;
    }

    NtGdiSelectFont(hdc, ghMenuFont);

    if (*menu).iItem != NO_SELECTED_ITEM as i32 {
        (*(*menu).rgItems.offset((*menu).iItem as isize)).fState &= !(MF_HILITE | MF_MOUSESELECT);
        MENU_DrawMenuItem(
            pWnd,
            menu,
            pwndOwner,
            hdc,
            (*menu).rgItems.offset((*menu).iItem as isize),
            (*menu).cyMenu as UINT,
            ((*menu).fFlags & MNF_POPUP == 0) as BOOL,
            ODA_SELECT,
        );
    }

    (*menu).iItem = wIndex as i32;
    if (*menu).iItem != NO_SELECTED_ITEM as i32 {
        if (*(*menu).rgItems.offset(wIndex as isize)).fType & MF_SEPARATOR == 0 {
            (*(*menu).rgItems.offset(wIndex as isize)).fState |= MF_HILITE;
            MENU_EnsureMenuItemVisible(menu, wIndex, hdc);
            MENU_DrawMenuItem(
                pWnd,
                menu,
                pwndOwner,
                hdc,
                (*menu).rgItems.offset(wIndex as isize),
                (*menu).cyMenu as UINT,
                ((*menu).fFlags & MNF_POPUP == 0) as BOOL,
                ODA_SELECT,
            );
        }
        if sendMenuSelect != 0 {
            let ip = (*menu).rgItems.offset((*menu).iItem as isize);
            let lo = if !(*ip).spSubMenu.is_null() { wIndex } else { (*ip).wID as UINT };
            let hi = (*ip).fType
                | (*ip).fState
                | (if !(*ip).spSubMenu.is_null() { MF_POPUP } else { 0 })
                | (if (*menu).fFlags & MNF_SYSMENU != 0 { MF_SYSMENU } else { 0 });
            let wParam: WPARAM = ((lo & 0xFFFF) | ((hi & 0xFFFF) << 16)) as WPARAM;

            co_IntSendMessage(
                UserHMGetHandle(pwndOwner) as HWND,
                WM_MENUSELECT,
                wParam,
                UserHMGetHandle(menu) as LPARAM,
            );
        }
    } else if sendMenuSelect != 0 {
        if !topmenu.is_null() {
            let pos: i32;
            let mut topmenu_local = topmenu;
            pos = MENU_FindSubMenu(&mut topmenu_local, menu) as i32;
            if pos != NO_SELECTED_ITEM as i32 {
                let ip = (*topmenu_local).rgItems.offset(pos as isize);
                let lo = pos;
                let hi = (*ip).fType
                    | (*ip).fState
                    | (if !(*ip).spSubMenu.is_null() { MF_POPUP } else { 0 })
                    | (if (*topmenu_local).fFlags & MNF_SYSMENU != 0 { MF_SYSMENU } else { 0 });
                let wParam: WPARAM = ((lo as u32 & 0xFFFF) | ((hi & 0xFFFF) << 16)) as WPARAM;

                co_IntSendMessage(
                    UserHMGetHandle(pwndOwner) as HWND,
                    WM_MENUSELECT,
                    wParam,
                    UserHMGetHandle(topmenu_local) as LPARAM,
                );
            }
        }
    }
    UserReleaseDC(pWnd, hdc, 0u8 as BOOL);
}
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn MENU_MoveSelection(pwndOwner: PWND, menu: PMENU, offset: INT) {
    MENU_MoveSelection_impl(pwndOwner, menu, offset)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn MENU_MoveSelection(pwndOwner: PWND, menu: PMENU, offset: INT) {
    MENU_MoveSelection_impl(pwndOwner, menu, offset)
}

unsafe fn MENU_MoveSelection_impl(pwndOwner: PWND, menu: PMENU, offset: INT) {
    let mut i: INT;

    if menu.is_null() || (*menu).rgItems.is_null() {
        return;
    }

    if (*menu).iItem != NO_SELECTED_ITEM as INT {
        if (*menu).cItems == 1 {
            return;
        } else {
            i = (*menu).iItem + offset;
            while i >= 0 && i < ((*menu).cItems as i32) {
                if (*(*menu).rgItems.offset(i as isize)).fType & MF_SEPARATOR == 0 {
                    MENU_SelectItem(pwndOwner, menu, i as UINT, 1, core::ptr::null_mut());
                    return;
                }
                i += offset;
            }
        }
    }

            i = if offset > 0 { 0 } else { ((*menu).cItems - 1) as i32 };
    while i >= 0 && i < ((*menu).cItems as i32) {
        if (*(*menu).rgItems.offset(i as isize)).fType & MF_SEPARATOR == 0 {
            MENU_SelectItem(pwndOwner, menu, i as UINT, 1, core::ptr::null_mut());
            return;
        }
        i += offset;
    }
}

fastcall_fn! {
unsafe fn MENU_HideSubPopups(
    pWndOwner: PWND,
    Menu: PMENU,
    SendMenuSelect: BOOL,
    wFlags: UINT,
) {
    if !Menu.is_null() && !top_popup.is_null() {
        let Item: PITEM;

        if (*Menu).iItem != (NO_SELECTED_ITEM as UINT as i32) {
            Item = core::ptr::addr_of_mut!(*(*Menu).rgItems.offset((*Menu).iItem as isize));
            if (*Item).spSubMenu.is_null() || ((*Item).fState & MF_MOUSESELECT as UINT) == 0 {
                return;
            }
            (*Item).fState &= !(MF_MOUSESELECT as UINT);
        } else {
            return;
        }

        if !(*Item).spSubMenu.is_null() {
            let pWnd: PWND;
            if VerifyMenu((*Item).spSubMenu).is_null() {
                return;
            }
            MENU_HideSubPopups(pWndOwner, (*Item).spSubMenu, FALSE as BOOL, wFlags);
            MENU_SelectItem(pWndOwner, (*Item).spSubMenu, NO_SELECTED_ITEM as UINT, SendMenuSelect, core::ptr::null_mut());
            pWnd = ValidateHwndNoErr((*(*Item).spSubMenu).hWnd);
            if !pWnd.is_null() {
                co_UserDestroyWindow(pWnd as *mut core::ffi::c_void );
            }

            if (wFlags & TPM_NONOTIFY) == 0 {
                co_IntSendMessage(
                    UserHMGetHandle(pWndOwner) as HWND,
                    WM_UNINITMENUPOPUP,
                    UserHMGetHandle((*Item).spSubMenu) as WPARAM,
                    MAKELPARAM(0, if IS_SYSTEM_MENU((*Item).spSubMenu) != 0 { MF_SYSMENU } else { 0 }),
                );
            }

            (*(*Item).spSubMenu).hWnd = core::ptr::null_mut();
        }
    }
}
}

fastcall_fn! {
unsafe fn MENU_ShowSubPopup(WndOwner: PWND, Menu: PMENU, SelectFirst: BOOL, mut Flags: UINT) -> PMENU {
    let mut Rect: RECT = core::mem::zeroed();
    let mut ParentRect: RECT = core::mem::zeroed();
    let Item: *mut ITEM;
    let Dc: HDC;
    let pWnd: PWND;

    if Menu.is_null() {
        return Menu;
    }

    if (*Menu).iItem == NO_SELECTED_ITEM as _ {
        return Menu;
    }

        Item = (*Menu).rgItems.offset((*Menu).iItem as isize) as *mut ITEM;
    let Item = (*Menu).rgItems.offset((*Menu).iItem as isize);
    if (*Item).spSubMenu.is_null()
        || ((*Item).fState & (MF_GRAYED | MF_DISABLED)) != 0
    {
        return Menu;
    }

    if (Flags & TPM_NONOTIFY) == 0 {
        co_IntSendMessage(
            UserHMGetHandle(WndOwner) as HWND,
            WM_INITMENUPOPUP,
            UserHMGetHandle((*Item).spSubMenu) as WPARAM,
            MAKELPARAM((*Menu).iItem as _, IS_SYSTEM_MENU(Menu) as _),
        );
    }

    let Item = (*Menu).rgItems.offset((*Menu).iItem as isize);
    Rect.left = (*Item).xItem as i32 ;
    Rect.top = (*Item).yItem as i32 ;
    Rect.right = (*Item).cxItem as i32 ;
    Rect.bottom = (*Item).cyItem as i32 ;

    pWnd = ValidateHwndNoErr((*Menu).hWnd);

    if ((*Menu).fFlags & MNF_POPUP) != 0 {
        if IntGetWindowRect(pWnd, (&mut ParentRect as *mut RECT).cast()) == 0 {
            ParentRect = Rect;
        }

        RECTL_vInflateRect(
            &mut ParentRect,
            -UserGetSystemMetrics(SM_CXEDGE) * 2,
            0,
        );
    } else {
        ParentRect = Rect;
        if !pWnd.is_null() {
            RECTL_vOffsetRect(
                &mut ParentRect,
                (*pWnd).rcWindow.left,
                (*pWnd).rcWindow.top,
            );
        }
    }

    if ((*Item).fState & MF_HILITE) == 0 {
        if ((*Menu).fFlags & MNF_POPUP) != 0 {
            Dc = UserGetDCEx(pWnd, core::ptr::null_mut(), DCX_USESTYLE);
        } else {
            Dc = UserGetDCEx(pWnd, 0 as _, DCX_CACHE | DCX_WINDOW);
        }

        NtGdiSelectFont(Dc, ghMenuFont);

        (*Item).fState |= MF_HILITE;
        MENU_DrawMenuItem(
            pWnd,
            Menu,
            WndOwner,
            Dc,
            Item,
            (*Menu).cyMenu as UINT,
            (((*Menu).fFlags & MNF_POPUP) == 0) as BOOL,
            ODA_DRAWENTIRE,
        );

        UserReleaseDC(pWnd, Dc, 0u8 as _);
    }

    if (*Item).yItem == 0 && (*Item).xItem == 0 && (*Item).cyItem == 0 && (*Item).cxItem == 0 {
        (*Item).xItem = Rect.left as u32 ;
        (*Item).yItem = Rect.top as u32 ;
        (*Item).cxItem = Rect.right as u32 ;
        (*Item).cyItem = Rect.bottom as u32 ;
    }
    (*Item).fState |= MF_MOUSESELECT;

    if IS_SYSTEM_MENU(Menu) != 0 && ((*Menu).fFlags & MNF_POPUP) == 0 {
        MENU_InitSysMenuPopup(
            (*Item).spSubMenu,
            (*pWnd).style,
            (*(*pWnd).pcls).style,
            HTSYSMENU as LONG,
        );

        NC_GetSysPopupPos(pWnd, &mut Rect);
        ParentRect = Rect;
        if (Flags & TPM_LAYOUTRTL) != 0 {
            Rect.left = Rect.right;
        }
        Rect.top = Rect.bottom;
        Rect.right = UserGetSystemMetrics(SM_CXSIZE);
        Rect.bottom = UserGetSystemMetrics(SM_CYSIZE);
    } else {
        IntGetWindowRect(pWnd, (&mut Rect as *mut RECT).cast());
        if ((*Menu).fFlags & MNF_POPUP) != 0 {
            let mut rc: RECT = core::mem::zeroed();
            rc.left = (*Item).xItem as i32;
            rc.top = (*Item).yItem as i32;
            rc.right = (*Item).cxItem as i32;
            rc.bottom = (*Item).cyItem as i32;

            MENU_AdjustMenuItemRect(Menu, &mut rc as *mut RECT as PRECTL);

            if (Flags & TPM_LAYOUTRTL) != 0 {
                Rect.left += UserGetSystemMetrics(SM_CXDLGFRAME);
            } else {
                Rect.left += rc.right - UserGetSystemMetrics(SM_CXDLGFRAME);
            }

            Rect.top += rc.top;
        } else {
            if (Flags & TPM_LAYOUTRTL) != 0 {
                Rect.left += Rect.right - ((*Item).xItem as i32);
            } else {
                Rect.left += (*Item).xItem as i32 ;
            }
            Rect.top += (*Item).cyItem as i32 ;
            Rect.right = ((*Item).cxItem - (*Item).xItem) as i32;
            Rect.bottom = ((*Item).cyItem - (*Item).yItem) as i32;
        }
    }

    if ((*Menu).fFlags & MNF_POPUP) != 0 {
        Flags &= !TPM_VERTICAL;
    }

    Flags &= !(TPM_CENTERALIGN | TPM_RIGHTALIGN | TPM_VCENTERALIGN | TPM_BOTTOMALIGN);

    MENU_InitPopup(WndOwner, (*Item).spSubMenu, Flags);

    MENU_ShowPopup(
        WndOwner,
        (*Item).spSubMenu,
        (*Menu).iItem as UINT,
        Flags,
        Rect.left,
        Rect.top,
        &ParentRect,
    );
    if SelectFirst != 0 {
        MENU_MoveSelection(WndOwner, (*Item).spSubMenu, ITEM_NEXT);
    }
    (*Item).spSubMenu
}
}

fastcall_fn! {
unsafe fn MENU_ExecFocusedItem(pmt: *mut MTRACKER, Menu: PMENU, Flags: UINT) -> INT {
    let Item: PITEM;

    if Menu.is_null() || (*Menu).cItems == 0 || (*Menu).iItem == NO_SELECTED_ITEM as i32 {
        return -1;
    }

    Item = core::ptr::addr_of_mut!(*(*Menu).rgItems.offset((*Menu).iItem as isize));

    if (*Item).spSubMenu.is_null() {
        if ((*Item).fState & (MF_GRAYED | MF_DISABLED)) == 0 && ((*Item).fType & MF_SEPARATOR) == 0 {
            if (Flags & TPM_RETURNCMD) == 0 {
                if ((*Menu).fFlags & MNF_SYSMENU as u32) != 0 {
                    UserPostMessage(
                        UserHMGetHandle((*pmt).OwnerWnd) as HWND,
                        WM_SYSCOMMAND,
                        (*Item).wID as WPARAM,
        MAKELPARAM((*pmt).Pt.x as u32, (*pmt).Pt.y as u32),
                    );
                } else {
                    let dwStyle: DWORD = ((*Menu).fFlags & MNS_STYLE_MASK)
                        | (if !(*pmt).TopMenu.is_null() {
                            (*(*pmt).TopMenu).fFlags & MNS_STYLE_MASK
                        } else {
                            0
                        });

                    if (dwStyle & MNS_NOTIFYBYPOS) != 0 {
                        UserPostMessage(
                            UserHMGetHandle((*pmt).OwnerWnd) as HWND,
                            WM_MENUCOMMAND,
                            (*Menu).iItem as WPARAM,
                            UserHMGetHandle(Menu) as LPARAM,
                        );
                    } else {
                        UserPostMessage(
                            UserHMGetHandle((*pmt).OwnerWnd) as HWND,
                            WM_COMMAND,
                            (*Item).wID as WPARAM,
                            0,
                        );
                    }
                }
            }
            return (*Item).wID as INT;
        }
    } else {
        (*pmt).CurrentMenu = MENU_ShowSubPopup((*pmt).OwnerWnd, Menu, 1, Flags);
        return -2;
    }

    -1
}
}

fastcall_fn! {
unsafe fn MENU_SwitchTracking(
    pmt: *mut MTRACKER,
    PtMenu: PMENU,
    Index: UINT,
    wFlags: UINT,
) {
    if (*pmt).TopMenu != PtMenu
        && ((*PtMenu).fFlags | (*(*pmt).TopMenu).fFlags) & MNF_POPUP == 0
    {
        MENU_HideSubPopups((*pmt).OwnerWnd, (*pmt).TopMenu, 0, wFlags);
        MENU_SelectItem(
            (*pmt).OwnerWnd,
            (*pmt).TopMenu,
            NO_SELECTED_ITEM as UINT,
            0,
            core::ptr::null_mut(),
        );
        (*pmt).TopMenu = PtMenu;
    } else {
        MENU_HideSubPopups((*pmt).OwnerWnd, PtMenu, 0, wFlags);
    }

    MENU_SelectItem((*pmt).OwnerWnd, PtMenu, Index, 1, core::ptr::null_mut());
}
}

fastcall_fn! {
unsafe fn MENU_ButtonDown(pmt: *mut MTRACKER, PtMenu: PMENU, Flags: UINT) -> BOOL {
    if !PtMenu.is_null() {
        let mut id: UINT = 0;
        let item: PITEM;

        if IS_SYSTEM_MENU(PtMenu) != 0 && ((*PtMenu).fFlags & MNF_POPUP) == 0 {
            item = (*PtMenu).rgItems;
        } else {
            item = MENU_FindItemByCoords(PtMenu, (*pmt).Pt, &mut id);
        }

        if !item.is_null() {
            if (*PtMenu).iItem != id as i32 {
                MENU_SwitchTracking(pmt, PtMenu, id, Flags);
            }

            if ((*item).fState & MF_MOUSESELECT) == 0 {
                (*pmt).CurrentMenu = MENU_ShowSubPopup((*pmt).OwnerWnd, PtMenu, 0, Flags);
            }

            return 1;
        }
    }
    0
}
}

fastcall_fn! {
unsafe fn MENU_ButtonUp(pmt: *mut MTRACKER, PtMenu: PMENU, Flags: UINT) -> INT {
    if !PtMenu.is_null() {
        let mut Id: UINT = 0;
        let item: *mut ITEM;

        if ((*PtMenu).fFlags & MNF_SYSMENU as u32) != 0 && ((*PtMenu).fFlags & MNF_POPUP as u32) == 0 {
            item = (*PtMenu).rgItems;
        } else {
            item = MENU_FindItemByCoords(PtMenu, (*pmt).Pt, &mut Id);
        }

        if !item.is_null() && ((*PtMenu).iItem == Id as i32) {
            if (*item).spSubMenu.is_null() {
                let ExecutedMenuId = MENU_ExecFocusedItem(pmt, PtMenu, Flags);
                if ExecutedMenuId == -1 || ExecutedMenuId == -2 {
                    return -1;
                }
                return ExecutedMenuId;
            }

            if (*pmt).TopMenu == PtMenu && (*PtMenu).TimeToHide != 0 {
                return 0;
            }
        }
        if IntGetMenu((*PtMenu).hWnd) == PtMenu {
            (*PtMenu).TimeToHide = 1;
        }
    }
    -1
}
}

unsafe fn MENU_PtMenu(menu: PMENU, pt: POINT) -> PMENU {
    let mut ret: PMENU = core::ptr::null_mut();

    if menu.is_null() {
        return core::ptr::null_mut();
    }

    if (*menu).iItem != NO_SELECTED_ITEM as i32 {
        let mut pItem: PITEM = (*menu).rgItems;
        if !pItem.is_null() {
            pItem = pItem.offset((*menu).iItem as isize);
        }
        if !pItem.is_null()
            && !(*pItem).spSubMenu.is_null()
            && ((*pItem).fState & MF_MOUSESELECT) != 0
        {
            ret = MENU_PtMenu((*pItem).spSubMenu, pt);
        }
    }

    if ret.is_null() {
        let pWnd: PWND = ValidateHwndNoErr((*menu).hWnd);
        let ht: INT = GetNCHitEx(pWnd, pt);
        if ((*menu).fFlags & MNF_POPUP) != 0 {
            if ht != HTNOWHERE as INT && ht != HTERROR as INT {
                ret = menu;
            }
        } else if ht == HTSYSMENU as INT {
            ret = get_win_sys_menu((*menu).hWnd);
        } else if ht == HTMENU as INT {
            ret = IntGetMenu((*menu).hWnd);
        }
    }
    ret
}

fastcall_fn! {
unsafe fn MENU_MouseMove(pmt: *mut MTRACKER, PtMenu: PMENU, Flags: UINT) -> BOOL {
    let mut Index: UINT = NO_SELECTED_ITEM as UINT;

    if !PtMenu.is_null() {
        MENU_FindItemByCoords(PtMenu, (*pmt).Pt, &mut Index);
    }

    if Index == NO_SELECTED_ITEM as UINT {
        MENU_SelectItem((*pmt).OwnerWnd, (*pmt).CurrentMenu, NO_SELECTED_ITEM as UINT, 1, (*pmt).TopMenu);
    } else if (*PtMenu).iItem != Index as _ {
        MENU_SwitchTracking(pmt, PtMenu, Index, Flags);
        (*pmt).CurrentMenu = MENU_ShowSubPopup((*pmt).OwnerWnd, PtMenu, 0, Flags);
    }
    1
}
}

unsafe fn MENU_GetSubPopup(menu: PMENU) -> PMENU {
    let item: *mut ITEM;

    if menu.is_null() || (*menu).iItem == NO_SELECTED_ITEM as i32 {
        return core::ptr::null_mut();
    }

    item = (*menu).rgItems.offset((*menu).iItem as isize);
    if !item.is_null() && !(*item).spSubMenu.is_null() && ((*item).fState & MF_MOUSESELECT) != 0 {
        return (*item).spSubMenu;
    }
    core::ptr::null_mut()
}

fastcall_fn! {
unsafe fn MENU_DoNextMenu(pmt: *mut MTRACKER, Vk: UINT, wFlags: UINT) -> LRESULT {
    let mut atEnd: BOOL = 0;

    if Vk == VK_LEFT && (*(*pmt).TopMenu).iItem == 0 {
        atEnd = 1;
    } else if Vk == VK_RIGHT && !(IS_SYSTEM_MENU((*pmt).TopMenu) != 0) {
        let mut i: UINT = ((*(*pmt).TopMenu).iItem + 1) as UINT;
        while i < (*(*pmt).TopMenu).cItems as UINT {
            let pItem = (*(*pmt).TopMenu).rgItems.offset(i as isize);
            if (*pItem).wID >= SC_SIZE as UINT && (*pItem).wID <= SC_RESTORE as UINT {
                i += 1;
            } else {
                break;
            }
        }
        if i == (*(*pmt).TopMenu).cItems as UINT {
            atEnd = 1;
        }
    } else if Vk == VK_RIGHT && IS_SYSTEM_MENU((*pmt).TopMenu) != 0 {
        if (*(*pmt).TopMenu).iItem == (((*(*pmt).TopMenu).cItems - 1) as i32) {
            atEnd = 1;
        }
    }

    if atEnd != 0 {
        let mut NextMenu: MDINEXTMENU = core::mem::zeroed();
        let mut MenuTmp: PMENU;
        let pwndTemp: PWND;
        let mut hNewMenu: HMENU;
        let mut hNewWnd: HWND;
        let mut Id: UINT = 0;

        MenuTmp = if IS_SYSTEM_MENU((*pmt).TopMenu) != 0 {
            co_IntGetSubMenu((*pmt).TopMenu, 0)
        } else {
            (*pmt).TopMenu
        };
        NextMenu.hmenuIn = UserHMGetHandle(MenuTmp as *mut core::ffi::c_void) as HMENU;
        NextMenu.hmenuNext = core::ptr::null_mut();
        NextMenu.hwndNext = core::ptr::null_mut();
        co_IntSendMessage(
            UserHMGetHandle((*pmt).OwnerWnd as *mut core::ffi::c_void) as HWND,
            WM_NEXTMENU,
            Vk as WPARAM,
            core::ptr::addr_of_mut!(NextMenu) as LPARAM,
        );

        if NextMenu.hmenuNext.is_null() || NextMenu.hwndNext.is_null() {
            hNewWnd = UserHMGetHandle((*pmt).OwnerWnd as *mut core::ffi::c_void) as HWND;
            if IS_SYSTEM_MENU((*pmt).TopMenu) != 0 {
                if (*(*pmt).OwnerWnd).style & WS_CHILD as u32 != 0 {
                    return 0;
                }
                MenuTmp = IntGetMenu(hNewWnd);
                if MenuTmp.is_null() {
                    return 0;
                }

                if Vk == VK_LEFT {
                    Id = ((*MenuTmp).cItems - 1) as UINT;

                    while Id > 0
                        && (*(*MenuTmp).rgItems.offset(Id as isize)).wID >= SC_SIZE as UINT
                        && (*(*MenuTmp).rgItems.offset(Id as isize)).wID <= SC_RESTORE as UINT
                    {
                        Id -= 1;
                    }
                }
                hNewMenu = UserHMGetHandle(MenuTmp as *mut core::ffi::c_void) as HMENU;
            } else if (*(*pmt).OwnerWnd).style & WS_SYSMENU as u32 != 0 {
                MenuTmp = get_win_sys_menu(hNewWnd);
                if !MenuTmp.is_null() {
                    hNewMenu = UserHMGetHandle(MenuTmp as *mut core::ffi::c_void) as HMENU;
                } else {
                    hNewMenu = core::ptr::null_mut();
                }
            } else {
                return 0;
            }
        } else {
            hNewMenu = NextMenu.hmenuNext;
            hNewWnd = NextMenu.hwndNext;

            MenuTmp = UserGetMenuObject(hNewMenu);
            if !MenuTmp.is_null() && {
                pwndTemp = ValidateHwndNoErr(hNewWnd);
                !pwndTemp.is_null()
            } {
                if (*pwndTemp).style & WS_SYSMENU as u32 != 0
                    && get_win_sys_menu(hNewWnd) == MenuTmp
                {
                    MenuTmp = get_win_sys_menu(hNewWnd);
                    hNewMenu = UserHMGetHandle(MenuTmp as *mut core::ffi::c_void) as HMENU;
                } else if (*pwndTemp).style & WS_CHILD as u32 != 0
                    || IntGetMenu(hNewWnd) != MenuTmp
                {
                    return 0;
                }
            } else {
                return 0;
            }
        }

        if hNewMenu != UserHMGetHandle((*pmt).TopMenu as *mut core::ffi::c_void) as HMENU {
            MENU_SelectItem(
                (*pmt).OwnerWnd,
                (*pmt).TopMenu,
                NO_SELECTED_ITEM as UINT,
                0,
                core::ptr::null_mut(),
            );

            if (*pmt).CurrentMenu != (*pmt).TopMenu {
                MENU_HideSubPopups((*pmt).OwnerWnd, (*pmt).TopMenu, 0, wFlags);
            }
        }

        if hNewWnd != UserHMGetHandle((*pmt).OwnerWnd as *mut core::ffi::c_void) as HWND {
            let pti: PTHREADINFO = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
            (*pmt).OwnerWnd = ValidateHwndNoErr(hNewWnd);
            MsqSetStateWindow(pti, MSQ_STATE_MENUOWNER, hNewWnd);
            (*(*pti).MessageQueue).QF_flags &= !(QF_CAPTURELOCKED as u32);
            co_UserSetCapture(UserHMGetHandle((*pmt).OwnerWnd as *mut core::ffi::c_void) as HWND);
            (*(*pti).MessageQueue).QF_flags |= QF_CAPTURELOCKED as u32;
        }

        (*pmt).TopMenu = UserGetMenuObject(hNewMenu);
        (*pmt).CurrentMenu = (*pmt).TopMenu;
        MENU_SelectItem((*pmt).OwnerWnd, (*pmt).TopMenu, Id, 1, core::ptr::null_mut());

        return 1;
    }
    0
}
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn MENU_SuspendPopup(pmt: *mut MTRACKER, uMsg: UINT) -> BOOL {
    MENU_SuspendPopup_impl(pmt, uMsg)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn MENU_SuspendPopup(pmt: *mut MTRACKER, uMsg: UINT) -> BOOL {
    MENU_SuspendPopup_impl(pmt, uMsg)
}

unsafe fn MENU_SuspendPopup_impl(pmt: *mut MTRACKER, uMsg: UINT) -> BOOL {
    let mut msg: MSG = core::mem::zeroed();

    msg.hwnd = UserHMGetHandle((*pmt).OwnerWnd) as HWND ;

    co_IntGetPeekMessage(&mut msg, 0 as HWND, uMsg, uMsg, (PM_NOYIELD | PM_REMOVE) as UINT, 0u8 as i32 );
    (*pmt).TrackFlags |= TF_SUSPENDPOPUP as UINT >> 0 | 0; // placeholder removed below
    (*pmt).TrackFlags |= TF_SKIPREMOVE as UINT;

    match uMsg {
        WM_KEYDOWN => {
            co_IntGetPeekMessage(&mut msg, 0 as HWND, 0, 0, (PM_NOYIELD | PM_NOREMOVE) as UINT, 0u8 as i32 );
            if msg.message == WM_KEYUP || msg.message == WM_PAINT {
                co_IntGetPeekMessage(&mut msg, 0 as HWND, 0, 0, (PM_NOYIELD | PM_REMOVE) as UINT, 0u8 as i32 );
                co_IntGetPeekMessage(&mut msg, 0 as HWND, 0, 0, (PM_NOYIELD | PM_NOREMOVE) as UINT, 0u8 as i32 );
                if msg.message == WM_KEYDOWN
                    && (msg.wParam == VK_LEFT as WPARAM || msg.wParam == VK_RIGHT as WPARAM)
                {
                    (*pmt).TrackFlags |= TF_SUSPENDPOPUP as UINT;
                    return 1;
                }
            }
        }
        _ => {}
    }

    (*pmt).TrackFlags &= !(TF_SUSPENDPOPUP as UINT);
    0
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn MENU_KeyEscape(pmt: *mut MTRACKER, Flags: UINT) -> BOOL {
    MENU_KeyEscape_impl(pmt, Flags)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn MENU_KeyEscape(pmt: *mut MTRACKER, Flags: UINT) -> BOOL {
    MENU_KeyEscape_impl(pmt, Flags)
}

unsafe fn MENU_KeyEscape_impl(pmt: *mut MTRACKER, Flags: UINT) -> BOOL {
    let mut EndMenu: BOOL = 1;

    if (*pmt).CurrentMenu != (*pmt).TopMenu {
        if !(*pmt).CurrentMenu.is_null() && ((*(*pmt).CurrentMenu).fFlags & MNF_POPUP) != 0 {
            let mut MenuPrev: PMENU;
            let mut MenuTmp: PMENU;

            MenuPrev = (*pmt).TopMenu;
            MenuTmp = (*pmt).TopMenu;

            while MenuTmp != (*pmt).CurrentMenu {
                MenuPrev = MenuTmp;
                MenuTmp = MENU_GetSubPopup(MenuPrev);
            }

            MENU_HideSubPopups((*pmt).OwnerWnd, MenuPrev, 1, Flags);
            (*pmt).CurrentMenu = MenuPrev;
            EndMenu = 0;
        }
    }

    EndMenu
}

fastcall_fn! {
unsafe fn MENU_KeyLeft(pmt: *mut MTRACKER, Flags: UINT, msg: UINT) {
    let mut MenuTmp: PMENU;
    let mut MenuPrev: PMENU;
    let PrevCol: UINT;

    MenuPrev = (*pmt).TopMenu;
    MenuTmp = (*pmt).TopMenu;

    /* Try to move 1 column left (if possible) */
    PrevCol = MENU_GetStartOfPrevColumn((*pmt).CurrentMenu);
    if PrevCol != NO_SELECTED_ITEM {
        MENU_SelectItem((*pmt).OwnerWnd, (*pmt).CurrentMenu, PrevCol, 1, core::ptr::null_mut());
        return;
    }

    /* close topmost popup */
    while MenuTmp != (*pmt).CurrentMenu {
        MenuPrev = MenuTmp;
        MenuTmp = MENU_GetSubPopup(MenuPrev);
    }

    MENU_HideSubPopups((*pmt).OwnerWnd, MenuPrev, 1, Flags);
    (*pmt).CurrentMenu = MenuPrev;

    if MenuPrev == (*pmt).TopMenu && ((*(*pmt).TopMenu).fFlags & MNF_POPUP) == 0 {
        /* move menu bar selection if no more popups are left */

        if MENU_DoNextMenu(pmt, VK_LEFT, Flags) == 0 {
            MENU_MoveSelection((*pmt).OwnerWnd, (*pmt).TopMenu, ITEM_PREV);
        }

        if MenuPrev != MenuTmp || ((*pmt).TrackFlags & TF_SUSPENDPOPUP) != 0 {
            /* A sublevel menu was displayed - display the next one
             * unless there is another displacement coming up */

            if MENU_SuspendPopup(pmt, msg) == 0 {
                (*pmt).CurrentMenu = MENU_ShowSubPopup((*pmt).OwnerWnd, (*pmt).TopMenu, 1, Flags);
            }
        }
    }
}
}

fastcall_fn! {
unsafe fn MENU_KeyRight(pmt: *mut MTRACKER, Flags: UINT, msg: UINT) {
    let mut menutmp: PMENU;
    let NextCol: UINT;

    if ((*(*pmt).TopMenu).fFlags & MNF_POPUP) != 0 || (*pmt).CurrentMenu != (*pmt).TopMenu {
        /* If already displaying a popup, try to display sub-popup */

        menutmp = (*pmt).CurrentMenu;
        (*pmt).CurrentMenu = MENU_ShowSubPopup((*pmt).OwnerWnd, menutmp, 1, Flags);

        /* if subpopup was displayed then we are done */
        if menutmp != (*pmt).CurrentMenu {
            return;
        }
    }

    /* Check to see if there's another column */
    NextCol = MENU_GetStartOfNextColumn((*pmt).CurrentMenu);
    if NextCol != NO_SELECTED_ITEM {
        MENU_SelectItem((*pmt).OwnerWnd, (*pmt).CurrentMenu, NextCol, 1, core::ptr::null_mut());
        return;
    }

    if ((*(*pmt).TopMenu).fFlags & MNF_POPUP) == 0 {
        /* menu bar tracking */
        if (*pmt).CurrentMenu != (*pmt).TopMenu {
            MENU_HideSubPopups((*pmt).OwnerWnd, (*pmt).TopMenu, 0, Flags);
            (*pmt).CurrentMenu = (*pmt).TopMenu;
            menutmp = (*pmt).TopMenu;
        } else {
            menutmp = core::ptr::null_mut();
        }

        /* try to move to the next item */
        if MENU_DoNextMenu(pmt, VK_RIGHT as UINT, Flags) == 0 {
            MENU_MoveSelection((*pmt).OwnerWnd, (*pmt).TopMenu, ITEM_NEXT);
        }

        if !menutmp.is_null() || ((*pmt).TrackFlags & TF_SUSPENDPOPUP) != 0 {
            if MENU_SuspendPopup(pmt, msg) == 0 {
                (*pmt).CurrentMenu = MENU_ShowSubPopup((*pmt).OwnerWnd, (*pmt).TopMenu, 1, Flags);
            }
        }
    }
}
}

fastcall_fn! {
unsafe fn MENU_TrackMenu(
    pmenu: PMENU,
    wFlags: UINT,
    x: INT,
    y: INT,
    pwnd: PWND,
) -> INT {
    let mut msg: MSG = core::mem::zeroed();
    let mut fRemove: BOOL;
    let mut executedMenuId: INT = -1;
    let mut mt: MTRACKER = core::mem::zeroed();
    let capture_win: HWND;
    let mut pmMouse: PMENU;
    let mut enterIdleSent: BOOL = 0;
    let mut firstClick: BOOL = 1;
    let mut pWnd: PWND;
    let pti: PTHREADINFO = PsGetCurrentThreadWin32Thread() as PTHREADINFO;

    if pti != head_pti(pwnd) {
        ERR!("Not the same PTI!!!!\n");
    }

    mt.TrackFlags = 0;
    mt.CurrentMenu = pmenu;
    mt.TopMenu = pmenu;
    mt.OwnerWnd = pwnd;
    mt.Pt.x = x;
    mt.Pt.y = y;

    TRACE!(b"MTM : hmenu=%p flags=0x%08x (%d,%d) hwnd=%x\n\0".as_ptr());

    (*(*pti).MessageQueue).QF_flags &= !QF_ACTIVATIONCHANGE;

    if wFlags & TPM_BUTTONDOWN != 0 {
        fRemove = MENU_ButtonDown(&mut mt, pmenu, wFlags);
        fInsideMenuLoop = fRemove;
    }

    if wFlags & TF_ENDMENU != 0 {
        fInsideMenuLoop = 0;
    }

    if (wFlags & TPM_POPUPMENU != 0) && (*pmenu).cItems == 0 {
        MsqSetStateWindow(pti, MSQ_STATE_MENUOWNER, core::ptr::null_mut());
        (*(*pti).MessageQueue).QF_flags &= !QF_CAPTURELOCKED;
        co_UserSetCapture(core::ptr::null_mut());
        return 0;
    }

    capture_win = IntGetCapture();

    while fInsideMenuLoop != 0 {
        let mut ErrorExit: BOOL = 0;
        if VerifyMenu(mt.CurrentMenu).is_null() {
            break;
        }

        loop {
            if co_IntGetPeekMessage(
                &mut msg,
                core::ptr::null_mut(),
                0,
                0,
                PM_NOREMOVE,
                0,
            ) != 0
            {
                if IntCallMsgFilter(&mut msg, MSGF_MENU as i32 ) == 0 {
                    break;
                }
                co_IntGetPeekMessage(
                    &mut msg,
                    core::ptr::null_mut(),
                    msg.message,
                    msg.message,
                    PM_REMOVE,
                    0,
                );
            } else {
                if VerifyWnd(mt.OwnerWnd) == (0 as usize as PWND)
                    || ValidateHwndNoErr((*mt.CurrentMenu).hWnd).is_null()
                    || capture_win != IntGetCapture()
                {
                    ErrorExit = 1;
                    break;
                }

                if enterIdleSent == 0 {
                    let win: HWND = if (*mt.CurrentMenu).fFlags & MNF_POPUP != 0 {
                        (*mt.CurrentMenu).hWnd
                    } else {
                        core::ptr::null_mut()
                    };
                    enterIdleSent = 1;
                    co_IntSendMessage(
                        UserHMGetHandle(mt.OwnerWnd) as HWND,
                        WM_ENTERIDLE,
                        MSGF_MENU as WPARAM,
                        win as LPARAM,
                    );
                }
                co_IntWaitMessage(core::ptr::null_mut(), 0, 0);
            }
        }

        if ErrorExit != 0 {
            break;
        }

        if msg.message == WM_CANCELMODE {
            fInsideMenuLoop = 0;
            co_IntGetPeekMessage(
                &mut msg,
                core::ptr::null_mut(),
                msg.message,
                msg.message,
                PM_REMOVE,
                0,
            );
            break;
        }

        mt.Pt = core::mem::transmute_copy(&msg.pt);

        if (msg.hwnd == (*mt.CurrentMenu).hWnd)
            || ((msg.message != WM_TIMER) && (msg.message != WM_SYSTIMER))
        {
            enterIdleSent = 0;
        }

        fRemove = 0;
        if (msg.message >= WM_MOUSEFIRST) && (msg.message <= WM_MOUSELAST) {
            mt.Pt.x = (LOWORD(msg.lParam as DWORD) as i16) as INT;
            mt.Pt.y = (HIWORD(msg.lParam as DWORD) as i16) as INT;

            pmMouse = MENU_PtMenu(mt.TopMenu, mt.Pt);

            match msg.message {
                m if m == WM_RBUTTONDBLCLK || m == WM_RBUTTONDOWN => {
                    if wFlags & TPM_RIGHTBUTTON == 0 {
                        if msg.message == WM_RBUTTONDBLCLK {
                            fInsideMenuLoop = 0;
                        }
                    } else {
                        pWnd = ValidateHwndNoErr((*mt.TopMenu).hWnd);
                        if msg.message == WM_LBUTTONDBLCLK
                            && GetNCHitEx(pWnd, mt.Pt) == (HTSYSMENU as i32)
                        {
                            fRemove = 0;
                        } else {
                            fRemove = MENU_ButtonDown(&mut mt, pmMouse, wFlags);
                        }
                        fInsideMenuLoop = fRemove;
                        if msg.message == WM_RBUTTONDBLCLK {
                            fInsideMenuLoop = 0;
                        }
                    }
                }
                m if m == WM_LBUTTONDBLCLK || m == WM_LBUTTONDOWN => {
                    pWnd = ValidateHwndNoErr((*mt.TopMenu).hWnd);
                    if msg.message == WM_LBUTTONDBLCLK
                        && GetNCHitEx(pWnd, mt.Pt) == (HTSYSMENU as i32)
                    {
                        fRemove = 0;
                    } else {
                        fRemove = MENU_ButtonDown(&mut mt, pmMouse, wFlags);
                    }
                    fInsideMenuLoop = fRemove;
                    if msg.message == WM_RBUTTONDBLCLK {
                        fInsideMenuLoop = 0;
                    }
                }
                m if m == WM_RBUTTONUP => {
                    if wFlags & TPM_RIGHTBUTTON == 0 {
                    } else if !pmMouse.is_null() {
                        pWnd = ValidateHwndNoErr((*mt.TopMenu).hWnd);
                        if firstClick == 0 && GetNCHitEx(pWnd, mt.Pt) == (HTSYSMENU as i32) {
                            fRemove = 1;
                            fInsideMenuLoop = 0;
                        } else {
                            executedMenuId = MENU_ButtonUp(&mut mt, pmMouse, wFlags);
                            fRemove = (executedMenuId != -1) as BOOL;
                            fInsideMenuLoop = (fRemove == 0) as BOOL;
                            firstClick = 0;
                        }
                    } else {
                        fInsideMenuLoop =
                            if wFlags & TPM_POPUPMENU != 0 { 1 } else { 0 };
                    }
                }
                m if m == WM_LBUTTONUP => {
                    if !pmMouse.is_null() {
                        pWnd = ValidateHwndNoErr((*mt.TopMenu).hWnd);
                        if firstClick == 0 && GetNCHitEx(pWnd, mt.Pt) == (HTSYSMENU as i32) {
                            fRemove = 1;
                            fInsideMenuLoop = 0;
                        } else {
                            executedMenuId = MENU_ButtonUp(&mut mt, pmMouse, wFlags);
                            fRemove = (executedMenuId != -1) as BOOL;
                            fInsideMenuLoop = (fRemove == 0) as BOOL;
                            firstClick = 0;
                        }
                    } else {
                        fInsideMenuLoop =
                            if wFlags & TPM_POPUPMENU != 0 { 1 } else { 0 };
                    }
                }
                m if m == WM_MOUSEMOVE => {
                    if !pmMouse.is_null() {
                        fInsideMenuLoop |= MENU_MouseMove(&mut mt, pmMouse, wFlags);
                    }
                }
                _ => {}
            }
        } else if (msg.message >= WM_KEYFIRST) && (msg.message <= WM_KEYLAST) {
            fRemove = 1;
            match msg.message {
                m if m == WM_KEYDOWN || m == WM_SYSKEYDOWN => {
                    match msg.wParam as UINT {
                        k if k == VK_MENU || k == VK_F10 => {
                            fInsideMenuLoop = 0;
                        }
                        k if k == VK_HOME || k == VK_END => {
                            MENU_SelectItem(
                                mt.OwnerWnd,
                                mt.CurrentMenu,
                                NO_SELECTED_ITEM as UINT,
                                0,
                                core::ptr::null_mut(),
                            );
                            MENU_MoveSelection(
                                mt.OwnerWnd,
                                mt.CurrentMenu,
                                if VK_HOME == msg.wParam as UINT {
                                    ITEM_NEXT
                                } else {
                                    ITEM_PREV
                                },
                            );
                        }
                        k if k == VK_UP || k == VK_DOWN => {
                            if (*mt.CurrentMenu).fFlags & MNF_POPUP == 0 {
                                mt.CurrentMenu = MENU_ShowSubPopup(
                                    mt.OwnerWnd,
                                    mt.TopMenu,
                                    1,
                                    wFlags,
                                );
                            } else {
                                MENU_MoveSelection(
                                    mt.OwnerWnd,
                                    mt.CurrentMenu,
                                    if msg.wParam as UINT == VK_UP {
                                        ITEM_PREV
                                    } else {
                                        ITEM_NEXT
                                    },
                                );
                            }
                        }
                        k if k == VK_LEFT => {
                            MENU_KeyLeft(&mut mt, wFlags, msg.message);
                        }
                        k if k == VK_RIGHT => {
                            MENU_KeyRight(&mut mt, wFlags, msg.message);
                        }
                        k if k == VK_ESCAPE => {
                            fInsideMenuLoop = (MENU_KeyEscape(&mut mt, wFlags) == 0) as BOOL;
                        }
                        k if k == VK_F1 => {
                            let mut hi: HELPINFO = core::mem::zeroed();
                            hi.cbSize = core::mem::size_of::<HELPINFO>() as UINT;
                            hi.iContextType = HELPINFO_MENUITEM as i32 ;
                            if (*mt.CurrentMenu).iItem == (NO_SELECTED_ITEM as i32) {
                                hi.iCtrlId = 0;
                            } else {
                                hi.iCtrlId =
                                    (*(*pmenu).rgItems.offset((*mt.CurrentMenu).iItem as isize)).wID as INT;
                            }
                            hi.hItemHandle = UserHMGetHandle(mt.CurrentMenu);
                            hi.dwContextId = (*pmenu).dwContextHelpId as _;
                            hi.MousePos = msg.pt;
                            co_IntSendMessage(
                                UserHMGetHandle(pwnd) as HWND,
                                WM_HELP,
                                0,
                                &mut hi as *mut HELPINFO as LPARAM,
                            );
                        }
                        _ => {
                            IntTranslateKbdMessage(&mut msg, 0);
                        }
                    }
                }
                m if m == WM_CHAR || m == WM_SYSCHAR => {
                    let pos: UINT;
                    let fEndMenu: BOOL;

                    if msg.wParam == b'\r' as WPARAM || msg.wParam == b' ' as WPARAM {
                        executedMenuId =
                            MENU_ExecFocusedItem(&mut mt, mt.CurrentMenu, wFlags);
                        fEndMenu = (executedMenuId != -2) as BOOL;
                        fInsideMenuLoop = (fEndMenu == 0) as BOOL;
                    } else if msg.wParam < 32 {
                    } else {
                        pos = MENU_FindItemByKey(
                            mt.OwnerWnd,
                            mt.CurrentMenu,
                            LOWORD(msg.wParam as DWORD) as u16 ,
                            0,
                        );

                        if pos == (-2i32) as UINT {
                            fInsideMenuLoop = 0;
                        } else if pos == (-1i32) as UINT {
                            UserPostMessage(hwndSAS, WM_LOGONNOTIFY, LN_MESSAGE_BEEP as WPARAM, 0);
                        } else {
                            MENU_SelectItem(mt.OwnerWnd, mt.CurrentMenu, pos, 1, core::ptr::null_mut());
                            executedMenuId =
                                MENU_ExecFocusedItem(&mut mt, mt.CurrentMenu, wFlags);
                            fEndMenu = (executedMenuId != -2) as BOOL;
                            fInsideMenuLoop = (fEndMenu == 0) as BOOL;
                        }
                    }
                }
                _ => {}
            }
        } else {
            co_IntGetPeekMessage(
                &mut msg,
                core::ptr::null_mut(),
                msg.message,
                msg.message,
                PM_REMOVE,
                0,
            );
            IntDispatchMessage(&mut msg);
            continue;
        }

        if fInsideMenuLoop != 0 {
            fRemove = 1;
        }

        if fRemove != 0 && (mt.TrackFlags & TF_SKIPREMOVE as UINT) == 0 {
            co_IntGetPeekMessage(
                &mut msg,
                core::ptr::null_mut(),
                msg.message,
                msg.message,
                PM_REMOVE,
                0,
            );
        } else {
            mt.TrackFlags &= !(TF_SKIPREMOVE as UINT);
        }
    }

    MsqSetStateWindow(pti, MSQ_STATE_MENUOWNER, core::ptr::null_mut());
    (*(*pti).MessageQueue).QF_flags &= !QF_CAPTURELOCKED;
    co_UserSetCapture(core::ptr::null_mut());

    if !VerifyMenu(mt.TopMenu).is_null() {
        if VerifyWnd(mt.OwnerWnd) != (0 as usize as PWND) {
            MENU_HideSubPopups(mt.OwnerWnd, mt.TopMenu, 0, wFlags);

            if (*mt.TopMenu).fFlags & MNF_POPUP != 0 {
                let pwndTM = ValidateHwndNoErr((*mt.TopMenu).hWnd);
                if !pwndTM.is_null() {
                    IntNotifyWinEvent(
                        EVENT_SYSTEM_MENUPOPUPEND,
                        pwndTM,
                        OBJID_CLIENT as LONG,
                        CHILDID_SELF as LONG,
                        0,
                    );

                    co_UserDestroyWindow(pwndTM as *mut core::ffi::c_void );
                }
                (*mt.TopMenu).hWnd = core::ptr::null_mut();

                if wFlags & TPM_NONOTIFY == 0 {
                    co_IntSendMessage(
                        UserHMGetHandle(mt.OwnerWnd) as HWND,
                        WM_UNINITMENUPOPUP,
                        UserHMGetHandle(mt.TopMenu) as WPARAM,
                        MAKELPARAM(
                            0,
                            if IS_SYSTEM_MENU(mt.TopMenu) != 0 {
                                MF_SYSMENU
                            } else {
                                0
                            },
                        ),
                    );
                }
            }
            MENU_SelectItem(
                mt.OwnerWnd,
                mt.TopMenu,
                NO_SELECTED_ITEM as UINT,
                0,
                core::ptr::null_mut(),
            );
            co_IntSendMessage(
                UserHMGetHandle(mt.OwnerWnd) as HWND,
                WM_MENUSELECT,
                MAKEWPARAM(0, 0xffff),
                0,
            );
        }

        (*mt.TopMenu).TimeToHide = 0;
    }

    EngSetLastError(ERROR_SUCCESS);
    if wFlags & TPM_RETURNCMD == 0 {
        return 1;
    }
    if executedMenuId == -1 {
        executedMenuId = 0;
    }
    executedMenuId
}
}


fastcall_fn! {
unsafe fn MENU_InitTracking(pWnd: PWND, Menu: PMENU, bPopup: u8, wFlags: UINT) -> u8 {
    let capture_win: HWND;
    let pti: PTHREADINFO = PsGetCurrentThreadWin32Thread() as PTHREADINFO;

    co_UserHideCaret(core::ptr::null_mut());

    if bPopup == 0 {
        (*Menu).hWnd = UserHMGetHandle(pWnd as *mut core::ffi::c_void) as HWND;
    }

    if top_popup.is_null() {
        top_popup = (*Menu).hWnd;
        top_popup_hmenu = UserHMGetHandle(Menu as *mut core::ffi::c_void) as HMENU;
    }

    fInsideMenuLoop = 1;
    fInEndMenu = 0;

    if (wFlags & TPM_NONOTIFY) == 0 {
        co_IntSendMessage(UserHMGetHandle(pWnd as *mut core::ffi::c_void) as HWND, WM_ENTERMENULOOP, bPopup as WPARAM, 0);
    }

    capture_win = if (wFlags & TPM_POPUPMENU) != 0 {
        (*Menu).hWnd
    } else {
        UserHMGetHandle(pWnd as *mut core::ffi::c_void) as HWND
    };
    MsqSetStateWindow(pti, MSQ_STATE_MENUOWNER, capture_win);
    co_UserSetCapture(capture_win);
    (*(*pti).MessageQueue).QF_flags |= QF_CAPTURELOCKED;

    co_IntSendMessage(
        UserHMGetHandle(pWnd as *mut core::ffi::c_void) as HWND,
        WM_SETCURSOR,
        UserHMGetHandle(pWnd as *mut core::ffi::c_void) as WPARAM,
        HTCAPTION as LPARAM,
    );

    if (wFlags & TPM_NONOTIFY) == 0 {
        co_IntSendMessage(
            UserHMGetHandle(pWnd as *mut core::ffi::c_void) as HWND,
            WM_INITMENU,
            UserHMGetHandle(Menu as *mut core::ffi::c_void) as WPARAM,
            0,
        );
    }

    IntNotifyWinEvent(
        EVENT_SYSTEM_MENUSTART,
        pWnd,
        if (*Menu).fFlags & MNF_SYSMENU != 0 { OBJID_SYSMENU as i32  } else { OBJID_MENU as i32  },
        CHILDID_SELF as LONG,
        0,
    );
    1
}
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn MENU_ExitTracking(pWnd: PWND, bPopup: BOOL, wFlags: UINT) -> BOOL {
    MENU_ExitTracking_impl(pWnd, bPopup, wFlags)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn MENU_ExitTracking(pWnd: PWND, bPopup: BOOL, wFlags: UINT) -> BOOL {
    MENU_ExitTracking_impl(pWnd, bPopup, wFlags)
}

unsafe fn MENU_ExitTracking_impl(pWnd: PWND, bPopup: BOOL, wFlags: UINT) -> BOOL {
    IntNotifyWinEvent(EVENT_SYSTEM_MENUEND as DWORD, pWnd, OBJID_WINDOW as LONG, CHILDID_SELF as LONG, 0);

    if (wFlags & TPM_NONOTIFY) == 0 {
        co_IntSendMessage(UserHMGetHandle(pWnd) as HWND, WM_EXITMENULOOP, bPopup as WPARAM, 0);
    }

    co_UserShowCaret(core::ptr::null_mut());

    top_popup = core::ptr::null_mut();
    top_popup_hmenu = core::ptr::null_mut();

    TRUE as BOOL
}

// MENU_TrackMouseMenuBar — cdecl per userfuncs.h (`_MENU_TrackMouseMenuBar`).
#[no_mangle]
unsafe extern "C" fn MENU_TrackMouseMenuBar(pWnd: PWND, ht: ULONG, mut pt: POINT) {
    let mut pMenu: PMENU = if ht == HTSYSMENU as ULONG {
        IntGetSystemMenu(pWnd, 0)
    } else {
        IntGetMenu(UserHMGetHandle(pWnd as *mut _) as HWND)
    };
    let mut wFlags: UINT = TPM_BUTTONDOWN | TPM_LEFTALIGN | TPM_LEFTBUTTON | TPM_VERTICAL;

    if (*pWnd).ExStyle & WS_EX_LAYOUTRTL != 0 {
        wFlags |= TPM_LAYOUTRTL;
    }
    if !VerifyMenu(pMenu).is_null() {
        let Parent: PWND = UserGetAncestor(pWnd, GA_PARENT);
        if Parent != UserGetDesktopWindow() {
            IntScreenToClient(Parent, (&mut pt as *mut POINT).cast());
        }

        MENU_InitTracking(pWnd, pMenu, 0, wFlags);
        pMenu = if ht == HTSYSMENU as ULONG {
            get_win_sys_menu(UserHMGetHandle(pWnd as *mut _) as HWND)
        } else {
            IntGetMenu(UserHMGetHandle(pWnd as *mut _) as HWND)
        };
        MENU_TrackMenu(pMenu, wFlags, pt.x, pt.y, pWnd);
        MENU_ExitTracking(pWnd, 0, wFlags);
    }
}

#[no_mangle]
unsafe extern "C" fn MENU_TrackKbdMenuBar(mut pwnd: PWND, mut wParam: UINT, wChar: WCHAR) {
    let mut uItem: UINT = NO_SELECTED_ITEM as UINT;
    let mut TrackMenu: PMENU;
    let mut wFlags: UINT = TPM_LEFTALIGN | TPM_LEFTBUTTON;

    /* find window that has a menu */
    while ((*pwnd).style & (WS_CHILD | WS_POPUP)) != WS_CHILD {
        pwnd = UserGetAncestor(pwnd, GA_PARENT);
        if pwnd.is_null() {
            return;
        }
    }

    /* check if we have to track a system menu */
    TrackMenu = IntGetMenu(UserHMGetHandle(pwnd as _) as HWND);
    if TrackMenu.is_null() || ((*pwnd).style & WS_MINIMIZE) != 0 || wChar == b' ' as WCHAR {
        if ((*pwnd).style & WS_SYSMENU) == 0 {
            return;
        }
        TrackMenu = get_win_sys_menu(UserHMGetHandle(pwnd as _) as HWND);
        uItem = 0;
        wParam |= HTSYSMENU;
    }

    if VerifyMenu(TrackMenu).is_null() {
        return;
    }

    MENU_InitTracking(pwnd, TrackMenu, 0u8, wFlags);

    /* fetch the window menu again, it may have changed */
    TrackMenu = if (wParam & HTSYSMENU) != 0 {
        get_win_sys_menu(UserHMGetHandle(pwnd as _) as HWND)
    } else {
        IntGetMenu(UserHMGetHandle(pwnd as _) as HWND)
    };

    'track_menu: {
        if wChar != 0 && wChar != b' ' as WCHAR {
            uItem = MENU_FindItemByKey(pwnd, TrackMenu, wChar, (wParam & HTSYSMENU) as BOOL);
            if uItem >= (-2i32) as UINT {
                if uItem == (-1i32) as UINT {
                    UserPostMessage(hwndSAS, WM_LOGONNOTIFY, LN_MESSAGE_BEEP as WPARAM, 0);
                }
                wFlags |= TF_ENDMENU;
                break 'track_menu;
            }
        }

        MENU_SelectItem(pwnd, TrackMenu, uItem, 1, core::ptr::null_mut());

        if (wParam & HTSYSMENU) == 0 || wChar == b' ' as WCHAR {
            if uItem == NO_SELECTED_ITEM as UINT {
                MENU_MoveSelection(pwnd, TrackMenu, ITEM_NEXT);
            } else {
                UserPostMessage(UserHMGetHandle(pwnd as _) as HWND, WM_KEYDOWN, VK_RETURN as WPARAM, 0);
            }
        }
    }

    MENU_TrackMenu(TrackMenu, wFlags, 0, 0, pwnd);
    MENU_ExitTracking(pwnd, 0, wFlags);
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntTrackPopupMenuEx(
    menu: PMENU,
    wFlags: UINT,
    x: INT,
    y: INT,
    pWnd: PWND,
    lpTpm: *const TPMPARAMS,
) -> BOOL {
    let mut ret: BOOL = 0;
    let pti: PTHREADINFO = PsGetCurrentThreadWin32Thread() as PTHREADINFO;

    if !lpTpm.is_null()
        && (*lpTpm).cbSize as usize != core::mem::size_of::<TPMPARAMS>()
    {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    if pti != head_pti(pWnd) {
        return ret;
    }

    if (*menu).hWnd != core::ptr::null_mut() && IntIsWindow((*menu).hWnd) != 0 {
        EngSetLastError(ERROR_POPUP_ALREADY_ACTIVE);
        return 0;
    }

    if MENU_InitPopup(pWnd, menu, wFlags) != 0 {
        MENU_InitTracking(pWnd, menu, 1u8, wFlags);

        if (wFlags & TPM_NONOTIFY) == 0 {
            co_IntSendMessage(
                UserHMGetHandle(pWnd) as HWND,
                WM_INITMENUPOPUP,
                UserHMGetHandle(menu) as WPARAM,
                MAKELPARAM(0, (IS_SYSTEM_MENU(menu) as u32)) as LPARAM,
            );
        }

        if ((*menu).fFlags & MNF_SYSMENU) != 0 {
            MENU_InitSysMenuPopup(
                menu,
                (*pWnd).style,
                (*(*pWnd).pcls).style,
                HTSYSMENU as LONG,
            );
        }

        if MENU_ShowPopup(
            pWnd,
            menu,
            0,
            wFlags | TPM_POPUPMENU,
            x,
            y,
            if !lpTpm.is_null() {
                core::ptr::addr_of!((*lpTpm).rcExclude).cast()
            } else {
                core::ptr::null()
            },
        ) != 0
        {
            ret = MENU_TrackMenu(menu, wFlags | TPM_POPUPMENU, 0, 0, pWnd);
        } else {
            MsqSetStateWindow(pti, MSQ_STATE_MENUOWNER, core::ptr::null_mut());
            (*(*pti).MessageQueue).QF_flags &= !QF_CAPTURELOCKED;
            co_UserSetCapture(core::ptr::null_mut());
        }

        MENU_ExitTracking(pWnd, 1, wFlags);

        if (*menu).hWnd != core::ptr::null_mut() {
            let pwndM: PWND = ValidateHwndNoErr((*menu).hWnd);
            if !pwndM.is_null() {
                if ((*pWnd).state & WNDS_DESTROYED) == 0 {
                    co_UserDestroyWindow((pwndM as *mut core::ffi::c_void));
                }
            }
            (*menu).hWnd = core::ptr::null_mut();

            if (wFlags & TPM_NONOTIFY) == 0 {
                co_IntSendMessage(
                    UserHMGetHandle(pWnd) as HWND,
                    WM_UNINITMENUPOPUP,
                    UserHMGetHandle(menu) as WPARAM,
                    MAKELPARAM(0, if IS_SYSTEM_MENU(menu) != 0 { MF_SYSMENU } else { 0 }) as LPARAM,
                );
            }
        }
    }
    ret
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntTrackPopupMenuEx(
    menu: PMENU,
    wFlags: UINT,
    x: INT,
    y: INT,
    pWnd: PWND,
    lpTpm: *const TPMPARAMS,
) -> BOOL {
    let mut ret: BOOL = 0;
    let pti: PTHREADINFO = PsGetCurrentThreadWin32Thread() as PTHREADINFO;

    if !lpTpm.is_null()
        && (*lpTpm).cbSize as usize != core::mem::size_of::<TPMPARAMS>()
    {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    if pti != head_pti(pWnd) {
        return ret;
    }

    if (*menu).hWnd != core::ptr::null_mut() && IntIsWindow((*menu).hWnd) != 0 {
        EngSetLastError(ERROR_POPUP_ALREADY_ACTIVE);
        return 0;
    }

    if MENU_InitPopup(pWnd, menu, wFlags) != 0 {
        MENU_InitTracking(pWnd, menu, 1u8, wFlags);

        if (wFlags & TPM_NONOTIFY) == 0 {
            co_IntSendMessage(
                UserHMGetHandle(pWnd) as HWND,
                WM_INITMENUPOPUP,
                UserHMGetHandle(menu) as WPARAM,
                MAKELPARAM(0, IS_SYSTEM_MENU(menu) as u32) as LPARAM,
            );
        }

        if ((*menu).fFlags & MNF_SYSMENU) != 0 {
            MENU_InitSysMenuPopup(
                menu,
                (*pWnd).style,
                (*(*pWnd).pcls).style,
                HTSYSMENU as LONG,
            );
        }

        if MENU_ShowPopup(
            pWnd,
            menu,
            0,
            wFlags | TPM_POPUPMENU,
            x,
            y,
            if !lpTpm.is_null() {
                core::ptr::addr_of!((*lpTpm).rcExclude).cast()
            } else {
                core::ptr::null()
            },
        ) != 0
        {
            ret = MENU_TrackMenu(menu, wFlags | TPM_POPUPMENU, 0, 0, pWnd);
        } else {
            MsqSetStateWindow(pti, MSQ_STATE_MENUOWNER, core::ptr::null_mut());
            (*(*pti).MessageQueue).QF_flags &= !QF_CAPTURELOCKED;
            co_UserSetCapture(core::ptr::null_mut());
        }

        MENU_ExitTracking(pWnd, 1, wFlags);

        if (*menu).hWnd != core::ptr::null_mut() {
            let pwndM: PWND = ValidateHwndNoErr((*menu).hWnd);
            if !pwndM.is_null() {
                if ((*pWnd).state & WNDS_DESTROYED) == 0 {
                    co_UserDestroyWindow(pwndM as _);
                }
            }
            (*menu).hWnd = core::ptr::null_mut();

            if (wFlags & TPM_NONOTIFY) == 0 {
                co_IntSendMessage(
                    UserHMGetHandle(pWnd) as HWND,
                    WM_UNINITMENUPOPUP,
                    UserHMGetHandle(menu) as WPARAM,
                    MAKELPARAM(0, if IS_SYSTEM_MENU(menu) != 0 { MF_SYSMENU } else { 0 }) as LPARAM,
                );
            }
        }
    }
    ret
}

#[no_mangle]
pub unsafe extern "system" fn PopupMenuWndProc(
    Wnd: PWND,
    Message: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
    lResult: *mut LRESULT,
) -> BOOL {
    let mut pPopupMenu: PPOPUPMENU;

    *lResult = 0;

    if !Wnd.is_null() {
        if (*Wnd).fnid == 0 {
            if Message != WM_NCCREATE {
                *lResult = IntDefWindowProc(Wnd, Message, wParam, lParam, 0u8 as BOOL);
                return 1;
            }
            (*Wnd).fnid = FNID_MENU as u16 as u32 ;
            pPopupMenu = DesktopHeapAlloc(
                (*Wnd).head.rpdesk,
                core::mem::size_of::<POPUPMENU>() as u32 as usize ,
            ) as PPOPUPMENU;
            if pPopupMenu.is_null() {
                return 1;
            }
            (*pPopupMenu).posSelectedItem = NO_SELECTED_ITEM as u32;
            (*pPopupMenu).spwndPopupMenu = Wnd;
            (*(Wnd as PMENUWND)).ppopupmenu = pPopupMenu;
            *lResult = 1;
            return 1;
        } else {
            if (*Wnd).fnid != (FNID_MENU as u16 as u32) {
                return 1;
            }
            pPopupMenu = (*(Wnd as PMENUWND)).ppopupmenu;
        }
    } else {
        pPopupMenu = core::ptr::null_mut();
    }

    match Message {
        WM_CREATE => {
            let cs = lParam as *mut CREATESTRUCTW;
            (*pPopupMenu).spmenu = UserGetMenuObject((*cs).lpCreateParams as HMENU);
            if !(*pPopupMenu).spmenu.is_null() {
                UserReferenceObject((*pPopupMenu).spmenu as *mut core::ffi::c_void);
            }
        }

        WM_MOUSEACTIVATE => {
            *lResult = MA_NOACTIVATE as LRESULT;
        }

        WM_PAINT => {
            let mut ps: PAINTSTRUCT = core::mem::zeroed();
            IntBeginPaint(Wnd, &mut ps);
            MENU_DrawPopupMenu(Wnd, ps.hdc, (*pPopupMenu).spmenu);
            IntEndPaint(Wnd, &mut ps);
        }

        WM_PRINTCLIENT => {
            MENU_DrawPopupMenu(Wnd, wParam as HDC, (*pPopupMenu).spmenu);
        }

        WM_ERASEBKGND => {
            *lResult = 1;
        }

        WM_DESTROY => {
            if !pPopupMenu.is_null() {
                if UserHMGetHandle(Wnd) as HWND == top_popup {
                    top_popup = core::ptr::null_mut();
                    top_popup_hmenu = core::ptr::null_mut();
                }
            }
        }

        WM_NCDESTROY => {
            if !(*pPopupMenu).spmenu.is_null() {
                IntReleaseMenuObject((*pPopupMenu).spmenu);
            }
            DesktopHeapFree((*Wnd).head.rpdesk, pPopupMenu as *mut core::ffi::c_void);
            (*(Wnd as PMENUWND)).ppopupmenu = core::ptr::null_mut();
            (*Wnd).fnid = FNID_DESTROY as u16 as u32 ;
        }

        x if x == MM_SETMENUHANDLE || x == MN_SETHMENU => {
            let pmenu = UserGetMenuObject(wParam as HMENU);
            if pmenu.is_null() {
                // Bad Menu Handle
            } else {
                UserReferenceObject(pmenu as *mut core::ffi::c_void);
                if !(*pPopupMenu).spmenu.is_null() {
                    IntReleaseMenuObject((*pPopupMenu).spmenu);
                }
                (*pPopupMenu).spmenu = pmenu;
            }
        }

        x if x == MM_GETMENUHANDLE || x == MN_GETHMENU => {
            *lResult = if !pPopupMenu.is_null() {
                if !(*pPopupMenu).spmenu.is_null() {
                    UserHMGetHandle((*pPopupMenu).spmenu) as LRESULT
                } else {
                    0
                }
            } else {
                0
            };
        }

        _ => {
            *lResult = IntDefWindowProc(Wnd, Message, wParam, lParam, 0u8 as BOOL);
        }
    }

    1
}

// MM_SETMENUHANDLE/MM_GETMENUHANDLE are defined as pub const near the top of this file.

fastcall_fn! {
#[no_mangle]
pub unsafe fn IntHiliteMenuItem(
    WindowObject: PWND,
    MenuObject: PMENU,
    uItemHilite: UINT,
    uHilite: UINT,
) -> BOOL {
    let MenuItem: PITEM;
    let mut uItem: UINT = uItemHilite;
    let mut pMenu: PMENU = MenuObject;

    MenuItem = MENU_FindItem(
        core::ptr::addr_of_mut!(pMenu),
        core::ptr::addr_of_mut!(uItem),
        uHilite,
    );
    if MenuItem.is_null() {
        return 1;
    }

    if uHilite & MF_HILITE != 0 {
        (*MenuItem).fState |= MF_HILITE;
    } else {
        (*MenuItem).fState &= !MF_HILITE;
    }

    if (*pMenu).iItem == uItemHilite as i32 {
        return 1;
    }

    MENU_HideSubPopups(WindowObject, pMenu, 0, 0);
    MENU_SelectItem(WindowObject, pMenu, uItemHilite, 1, core::ptr::null_mut());

    1
}
}

unsafe extern "system" fn intGetTitleBarInfo(pWindowObject: PWND, bti: PTITLEBARINFO) -> BOOLEAN {
    let mut dwStyle: DWORD;
    let mut dwExStyle: DWORD;
    let mut retValue: BOOLEAN = 1u8;

    if (*bti).cbSize as usize == core::mem::size_of::<TITLEBARINFO>() {
        core::ptr::write_bytes(
            core::ptr::addr_of_mut!((*bti).rgstate[0]),
            0,
            (CCHILDREN_TITLEBAR + 1) as usize,
        );

        (*bti).rgstate[0] = STATE_SYSTEM_FOCUSABLE as DWORD;

        dwStyle = (*pWindowObject).style;
        dwExStyle = (*pWindowObject).ExStyle;

        (*bti).rcTitleBar.top = 0;
        (*bti).rcTitleBar.left = 0;
        (*bti).rcTitleBar.right =
            (*pWindowObject).rcWindow.right - (*pWindowObject).rcWindow.left;
        (*bti).rcTitleBar.bottom =
            (*pWindowObject).rcWindow.bottom - (*pWindowObject).rcWindow.top;

        if (dwStyle & WS_ICONIC) != WS_ICONIC {
            if HAS_THICKFRAME(dwStyle, dwExStyle) != 0 {
                RECTL_vInflateRect(
                    core::ptr::addr_of_mut!((*bti).rcTitleBar) as *mut RECT ,
                    -UserGetSystemMetrics(SM_CXFRAME as ULONG),
                    -UserGetSystemMetrics(SM_CYFRAME as ULONG),
                );
            } else if HAS_DLGFRAME(dwStyle, dwExStyle) != 0 {
                RECTL_vInflateRect(
                    core::ptr::addr_of_mut!((*bti).rcTitleBar) as *mut RECT ,
                    -UserGetSystemMetrics(SM_CXDLGFRAME as ULONG),
                    -UserGetSystemMetrics(SM_CYDLGFRAME as ULONG),
                );
            } else if HAS_THINFRAME(dwStyle, dwExStyle) != 0 {
                RECTL_vInflateRect(
                    core::ptr::addr_of_mut!((*bti).rcTitleBar) as *mut RECT ,
                    -UserGetSystemMetrics(SM_CXBORDER as ULONG),
                    -UserGetSystemMetrics(SM_CYBORDER as ULONG),
                );
            }

            if (dwStyle & WS_CHILD) != 0 && (dwExStyle & WS_EX_MDICHILD) == 0 {
                if dwExStyle & WS_EX_CLIENTEDGE != 0 {
                    RECTL_vInflateRect(
                        core::ptr::addr_of_mut!((*bti).rcTitleBar) as *mut RECT ,
                        -UserGetSystemMetrics(SM_CXEDGE as ULONG),
                        -UserGetSystemMetrics(SM_CYEDGE as ULONG),
                    );
                }

                if dwExStyle & WS_EX_STATICEDGE != 0 {
                    RECTL_vInflateRect(
                        core::ptr::addr_of_mut!((*bti).rcTitleBar) as *mut RECT ,
                        -UserGetSystemMetrics(SM_CXBORDER as ULONG),
                        -UserGetSystemMetrics(SM_CYBORDER as ULONG),
                    );
                }
            }
        }

        (*bti).rcTitleBar.top += (*pWindowObject).rcWindow.top;
        (*bti).rcTitleBar.left += (*pWindowObject).rcWindow.left;
        (*bti).rcTitleBar.right += (*pWindowObject).rcWindow.left;

        (*bti).rcTitleBar.bottom = (*bti).rcTitleBar.top;
        if dwExStyle & WS_EX_TOOLWINDOW != 0 {
            (*bti).rcTitleBar.bottom += UserGetSystemMetrics(SM_CYSMCAPTION as ULONG);
        } else {
            (*bti).rcTitleBar.bottom += UserGetSystemMetrics(SM_CYCAPTION as ULONG);
            (*bti).rcTitleBar.left += UserGetSystemMetrics(SM_CXSIZE as ULONG);
        }

        if dwStyle & WS_CAPTION != 0 {
            (*bti).rgstate[1] = STATE_SYSTEM_INVISIBLE as DWORD;
            if dwStyle & WS_SYSMENU != 0 {
                if (dwStyle & (WS_MINIMIZEBOX | WS_MAXIMIZEBOX)) == 0 {
                    (*bti).rgstate[2] = STATE_SYSTEM_INVISIBLE as DWORD;
                    (*bti).rgstate[3] = STATE_SYSTEM_INVISIBLE as DWORD;
                } else {
                    if (dwStyle & WS_MINIMIZEBOX) == 0 {
                        (*bti).rgstate[2] = STATE_SYSTEM_UNAVAILABLE as DWORD;
                    }
                    if (dwStyle & WS_MAXIMIZEBOX) == 0 {
                        (*bti).rgstate[3] = STATE_SYSTEM_UNAVAILABLE as DWORD;
                    }
                }

                if (dwExStyle & WS_EX_CONTEXTHELP) == 0 {
                    (*bti).rgstate[4] = STATE_SYSTEM_INVISIBLE as DWORD;
                }
                if (*(*pWindowObject).pcls).style & CS_NOCLOSE != 0 {
                    (*bti).rgstate[5] = STATE_SYSTEM_UNAVAILABLE as DWORD;
                }
            } else {
                (*bti).rgstate[2] = STATE_SYSTEM_INVISIBLE as DWORD;
                (*bti).rgstate[3] = STATE_SYSTEM_INVISIBLE as DWORD;
                (*bti).rgstate[4] = STATE_SYSTEM_INVISIBLE as DWORD;
                (*bti).rgstate[5] = STATE_SYSTEM_INVISIBLE as DWORD;
            }
        } else {
            (*bti).rgstate[0] |= STATE_SYSTEM_INVISIBLE as DWORD;
        }
    } else {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        retValue = 0u8;
    }

    retValue
}

fastcall_fn! {
unsafe fn UserInsertMenuItem(
    Menu: PMENU,
    uItem: UINT,
    fByPosition: BOOL,
    UnsafeItemInfo: LPCMENUITEMINFOW,
    lpstr: PUNICODE_STRING,
) -> DWORD {
    let mut Status: NTSTATUS;
    let mut ItemInfo: ROSMENUITEMINFO = core::mem::zeroed();

    Status = MmCopyFromCaller(
        core::ptr::addr_of_mut!(ItemInfo) as *mut core::ffi::c_void,
        UnsafeItemInfo as *const core::ffi::c_void,
        core::mem::size_of::<MENUITEMINFOW>(),
    );
    if NT_SUCCESS(Status) {
        let cbHbmpItem = {
            let base = core::ptr::null::<MENUITEMINFOW>();
            core::ptr::addr_of!((*base).hbmpItem) as usize
        };
        if core::mem::size_of::<MENUITEMINFOW>() as u32 != ItemInfo.cbSize
            && cbHbmpItem as u32 != ItemInfo.cbSize
        {
            EngSetLastError(ERROR_INVALID_PARAMETER);
            return 0;
        }
        return IntInsertMenuItem(
            Menu,
            uItem,
            fByPosition,
            core::ptr::addr_of_mut!(ItemInfo),
            lpstr,
        ) as DWORD;
    }

    let cbHbmpItem = {
        let base = core::ptr::null::<MENUITEMINFOW>();
        core::ptr::addr_of!((*base).hbmpItem) as usize
    };

    Status = MmCopyFromCaller(
        core::ptr::addr_of_mut!(ItemInfo) as *mut core::ffi::c_void,
        UnsafeItemInfo as *const core::ffi::c_void,
        cbHbmpItem,
    );
    if NT_SUCCESS(Status) {
        if cbHbmpItem as u32 != ItemInfo.cbSize {
            EngSetLastError(ERROR_INVALID_PARAMETER);
            return 0;
        }
        ItemInfo.hbmpItem = core::ptr::null_mut();
        return IntInsertMenuItem(
            Menu,
            uItem,
            fByPosition,
            core::ptr::addr_of_mut!(ItemInfo),
            lpstr,
        ) as DWORD;
    }

    SetLastNtError(Status);
    0
}
}

#[cfg(target_arch = "x86")]
#[no_mangle]
unsafe extern "fastcall" fn IntGetMenuState(hMenu: HMENU, mut uId: UINT, uFlags: UINT) -> UINT {
    IntGetMenuState_impl(hMenu, uId, uFlags)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
unsafe extern "C" fn IntGetMenuState(hMenu: HMENU, uId: UINT, uFlags: UINT) -> UINT {
    IntGetMenuState_impl(hMenu, uId, uFlags)
}

unsafe fn IntGetMenuState_impl(hMenu: HMENU, mut uId: UINT, uFlags: UINT) -> UINT {
    let mut MenuObject: PMENU;
    let pItem: PITEM;

    MenuObject = UserGetMenuObject(hMenu);
    if MenuObject.is_null() {
        return (-1i32) as UINT;
    }

    pItem = MENU_FindItem(
        core::ptr::addr_of_mut!(MenuObject),
        core::ptr::addr_of_mut!(uId),
        uFlags,
    );
    if pItem.is_null() {
        return (-1i32) as UINT;
    }

    if !(*pItem).spSubMenu.is_null() {
        return (((*(*pItem).spSubMenu).cItems as UINT) << 8)
            | (((*pItem).fState | (*pItem).fType | MF_POPUP) & 0xff);
    } else {
        return (*pItem).fType | (*pItem).fState;
    }
}

#[cfg(target_pointer_width = "32")]
unsafe extern "fastcall" fn IntGetSubMenu(hMenu: HMENU, nPos: core::ffi::c_int) -> HMENU {
    IntGetSubMenu_impl(hMenu, nPos)
}

#[cfg(target_pointer_width = "64")]
unsafe extern "C" fn IntGetSubMenu(hMenu: HMENU, nPos: core::ffi::c_int) -> HMENU {
    IntGetSubMenu_impl(hMenu, nPos)
}

unsafe fn IntGetSubMenu_impl(hMenu: HMENU, mut nPos: core::ffi::c_int) -> HMENU {
    let mut MenuObject: PMENU;
    let pItem: PITEM;

    MenuObject = UserGetMenuObject(hMenu);
    if MenuObject.is_null() {
        return core::ptr::null_mut();
    }

    pItem = MENU_FindItem(
        core::ptr::addr_of_mut!(MenuObject),
        core::ptr::addr_of_mut!(nPos) as *mut UINT,
        MF_BYPOSITION,
    );
    if pItem.is_null() {
        return core::ptr::null_mut();
    }

    if !(*pItem).spSubMenu.is_null() {
        let hsubmenu = UserHMGetHandle((*pItem).spSubMenu as *mut core::ffi::c_void) as HMENU;
        return hsubmenu;
    }
    core::ptr::null_mut()
}

fastcall_fn! {
#[no_mangle]
unsafe fn IntFindSubMenu(hMenu: *mut HMENU, hSubTarget: HMENU) -> UINT {
    let mut menu: PMENU;
    let pSubTarget: PMENU;
    let posIdx: UINT;

    if (*hMenu) == (0xffff as HMENU) || {
        menu = UserGetMenuObject(*hMenu);
        menu.is_null()
    } {
        return NO_SELECTED_ITEM;
    }

    pSubTarget = UserGetMenuObject(hSubTarget);

    posIdx = MENU_FindSubMenu(&mut menu, pSubTarget);

    *hMenu = if !menu.is_null() {
        UserHMGetHandle(menu) as HMENU
    } else {
        core::ptr::null_mut()
    };

    posIdx
}
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn UserCreateMenu(Desktop: PDESKTOP, PopupMenu: BOOL) -> HMENU {
    let mut WinStaObject: PWINSTATION_OBJECT = core::ptr::null_mut();
    let mut Handle: HANDLE = core::ptr::null_mut();
    let Menu: PMENU;
    let Status: NTSTATUS;
    let CurrentProcess: PEPROCESS = PsGetCurrentProcess() as PEPROCESS;

    if gpepCSRSS != CurrentProcess {
        Status = IntValidateWindowStationHandle(
            (*CurrentProcess).Win32WindowStation as HWINSTA,
            1i32 as i8,
            0,
            &mut WinStaObject,
            core::ptr::null_mut(),
        );

        if !NT_SUCCESS(Status) {
            ERR!(
                "Validation of window station handle (%p) failed\n",
                (*CurrentProcess).Win32WindowStation as HWINSTA,
            );
            SetLastNtError(Status);
            return core::ptr::null_mut();
        }

        Menu = IntCreateMenu(
            &mut Handle,
            (PopupMenu == 0) as BOOL,
            Desktop,
            GetW32ProcessInfo(),
        );
        if !Menu.is_null()
            && (*(*Menu).head.rpdesk).rpwinstaParent != WinStaObject
        {
            ERR!("Desktop Window Station does not match Process one!\n");
        }
        ObDereferenceObject(WinStaObject as *mut _);
    } else {
        Menu = IntCreateMenu(
            &mut Handle,
            (PopupMenu == 0) as BOOL,
            (*GetW32ThreadInfo()).rpdesk,
            GetW32ProcessInfo(),
        );
    }

    if !Menu.is_null() {
        UserDereferenceObject(Menu as *mut _);
    }
    Handle as HMENU
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn UserCreateMenu(Desktop: PDESKTOP, PopupMenu: BOOL) -> HMENU {
    let mut WinStaObject: PWINSTATION_OBJECT = core::ptr::null_mut();
    let mut Handle: HANDLE = core::ptr::null_mut();
    let Menu: PMENU;
    let Status: NTSTATUS;
    let CurrentProcess: PEPROCESS = PsGetCurrentProcess() as PEPROCESS;

    if gpepCSRSS != CurrentProcess {
        Status = IntValidateWindowStationHandle(
            (*CurrentProcess).Win32WindowStation as HWINSTA,
            1i32 as i8,
            0,
            &mut WinStaObject,
            core::ptr::null_mut(),
        );

        if !NT_SUCCESS(Status) {
            ERR!(
                "Validation of window station handle (%p) failed\n",
                (*CurrentProcess).Win32WindowStation as HWINSTA,
            );
            SetLastNtError(Status);
            return core::ptr::null_mut();
        }

        Menu = IntCreateMenu(
            &mut Handle,
            (PopupMenu == 0) as BOOL,
            Desktop,
            GetW32ProcessInfo(),
        );
        if !Menu.is_null()
            && (*(*Menu).head.rpdesk).rpwinstaParent != WinStaObject
        {
            ERR!("Desktop Window Station does not match Process one!\n");
        }
        ObDereferenceObject(WinStaObject as *mut _);
    } else {
        Menu = IntCreateMenu(
            &mut Handle,
            (PopupMenu == 0) as BOOL,
            (*GetW32ThreadInfo()).rpdesk,
            GetW32ProcessInfo(),
        );
    }

    if !Menu.is_null() {
        UserDereferenceObject(Menu as *mut _);
    }
    Handle as HMENU
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntMenuItemInfo(
    mut Menu: PMENU,
    mut Item: UINT,
    ByPosition: BOOL,
    ItemInfo: PROSMENUITEMINFO,
    SetOrGet: BOOL,
    lpstr: PUNICODE_STRING,
) -> BOOL {
    let MenuItem: PITEM;
    let Ret: BOOL;

    MenuItem = MENU_FindItem(
        &mut Menu,
        &mut Item,
        if ByPosition != 0 { MF_BYPOSITION } else { MF_BYCOMMAND },
    );
    if MenuItem.is_null() {
        EngSetLastError(ERROR_MENU_ITEM_NOT_FOUND);
        return 0;
    }
    if SetOrGet != 0 {
        Ret = IntSetMenuItemInfo(Menu, MenuItem, ItemInfo, lpstr);
    } else {
        Ret = IntGetMenuItemInfo(Menu, MenuItem, ItemInfo);
    }
    Ret
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntMenuItemInfo(
    mut Menu: PMENU,
    mut Item: UINT,
    ByPosition: BOOL,
    ItemInfo: PROSMENUITEMINFO,
    SetOrGet: BOOL,
    lpstr: PUNICODE_STRING,
) -> BOOL {
    let MenuItem: PITEM;
    let Ret: BOOL;

    MenuItem = MENU_FindItem(
        &mut Menu,
        &mut Item,
        if ByPosition != 0 { MF_BYPOSITION } else { MF_BYCOMMAND },
    );
    if MenuItem.is_null() {
        EngSetLastError(ERROR_MENU_ITEM_NOT_FOUND);
        return 0;
    }
    if SetOrGet != 0 {
        Ret = IntSetMenuItemInfo(Menu, MenuItem, ItemInfo, lpstr);
    } else {
        Ret = IntGetMenuItemInfo(Menu, MenuItem, ItemInfo);
    }
    Ret
}

fastcall_fn! {
#[no_mangle]
pub unsafe fn UserMenuItemInfo(
    mut Menu: PMENU,
    mut Item: UINT,
    ByPosition: BOOL,
    UnsafeItemInfo: PROSMENUITEMINFO,
    SetOrGet: BOOL,
    lpstr: PUNICODE_STRING,
) -> BOOL {
    let MenuItem: PITEM;
    let mut ItemInfo: ROSMENUITEMINFO = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let mut Size: UINT = 0;
    let Ret: BOOL;

    Status = MmCopyFromCaller(
        core::ptr::addr_of_mut!(Size) as *mut core::ffi::c_void,
        core::ptr::addr_of_mut!((*UnsafeItemInfo).cbSize) as *mut core::ffi::c_void,
        core::mem::size_of::<UINT>(),
    );
    if !NT_SUCCESS(Status) {
        SetLastNtError(Status);
        return 0;
    }
    if Size != core::mem::size_of::<MENUITEMINFOW>() as UINT
        && Size != core::mem::offset_of!(MENUITEMINFOW, hbmpItem) as u32
        && Size != core::mem::size_of::<ROSMENUITEMINFO>() as UINT
    {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }
    Status = MmCopyFromCaller(
        core::ptr::addr_of_mut!(ItemInfo) as *mut core::ffi::c_void,
        UnsafeItemInfo as *mut core::ffi::c_void,
        Size as usize,
    );
    if !NT_SUCCESS(Status) {
        SetLastNtError(Status);
        return 0;
    }
    if core::mem::offset_of!(MENUITEMINFOW, hbmpItem) == (Size as usize)
        && 0 != (ItemInfo.fMask & MIIM_BITMAP)
    {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    MenuItem = MENU_FindItem(
        core::ptr::addr_of_mut!(Menu),
        core::ptr::addr_of_mut!(Item),
        if ByPosition != 0 { MF_BYPOSITION } else { MF_BYCOMMAND },
    );
    if MenuItem.is_null() {
        if SetOrGet != 0 && Item == SC_TASKLIST && ByPosition == 0 {
            return 1;
        }

        EngSetLastError(ERROR_MENU_ITEM_NOT_FOUND);
        return 0;
    }

    if SetOrGet != 0 {
        Ret = IntSetMenuItemInfo(
            Menu,
            MenuItem,
            core::ptr::addr_of_mut!(ItemInfo),
            lpstr,
        );
    } else {
        Ret = IntGetMenuItemInfo(Menu, MenuItem, core::ptr::addr_of_mut!(ItemInfo));
        if Ret != 0 {
            Status = MmCopyToCaller(
                UnsafeItemInfo as *mut core::ffi::c_void,
                core::ptr::addr_of_mut!(ItemInfo) as *mut core::ffi::c_void,
                Size as usize,
            );
            if !NT_SUCCESS(Status) {
                SetLastNtError(Status);
                return 0;
            }
        }
    }

    Ret
}
}

fastcall_fn! {
unsafe fn UserMenuInfo(
    Menu: PMENU,
    UnsafeMenuInfo: PROSMENUINFO,
    SetOrGet: i32,
) -> i32 {

    let mut Res: i32;
    let mut Size: u32 = 0;
    let mut Status: i32;
    let mut MenuInfo: ROSMENUINFO = core::mem::zeroed();

    Status = MmCopyFromCaller(
        core::ptr::addr_of_mut!(Size) as *mut core::ffi::c_void,
        core::ptr::addr_of!((*UnsafeMenuInfo).cbSize) as *const core::ffi::c_void,
        core::mem::size_of::<u32>(),
    );
    if !NT_SUCCESS(Status) {
        SetLastNtError(Status);
        return 0;
    }
    if (Size as usize) < core::mem::size_of::<MENUINFO>()
        || (Size as usize) > core::mem::size_of::<ROSMENUINFO>()
    {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }
    Status = MmCopyFromCaller(
        core::ptr::addr_of_mut!(MenuInfo) as *mut core::ffi::c_void,
        UnsafeMenuInfo as *const core::ffi::c_void,
        Size as usize,
    );
    if !NT_SUCCESS(Status) {
        SetLastNtError(Status);
        return 0;
    }

    if SetOrGet != 0 {
        Res = IntSetMenuInfo(Menu, core::ptr::addr_of_mut!(MenuInfo)) as i32;
    } else {
        Res = IntGetMenuInfo(Menu, core::ptr::addr_of_mut!(MenuInfo)) as i32;
        if Res != 0 {
            Status = MmCopyToCaller(
                UnsafeMenuInfo as *mut core::ffi::c_void,
                core::ptr::addr_of!(MenuInfo) as *const core::ffi::c_void,
                Size as usize,
            );
            if !NT_SUCCESS(Status) {
                SetLastNtError(Status);
                return 0;
            }
        }
    }

    Res
}
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntGetMenuItemRect(
    pWnd: PWND,
    Menu: PMENU,
    uItem: UINT,
    Rect: PRECTL,
) -> BOOL {
    IntGetMenuItemRect_impl(pWnd, Menu, uItem, Rect)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntGetMenuItemRect(
    pWnd: PWND,
    Menu: PMENU,
    uItem: UINT,
    Rect: PRECTL,
) -> BOOL {
    IntGetMenuItemRect_impl(pWnd, Menu, uItem, Rect)
}

unsafe fn IntGetMenuItemRect_impl(
    mut pWnd: PWND,
    Menu: PMENU,
    uItem: UINT,
    Rect: PRECTL,
) -> BOOL {
    let XMove: LONG;
    let YMove: LONG;
    let MenuItem: PITEM;
    let mut I: UINT = uItem;
    let mut LocalMenu: PMENU = Menu;

    MenuItem = MENU_FindItem(
        core::ptr::addr_of_mut!(LocalMenu),
        core::ptr::addr_of_mut!(I),
        MF_BYPOSITION,
    );
    if !MenuItem.is_null() {
        (*Rect).left = (*MenuItem).xItem as i32 ;
        (*Rect).top = (*MenuItem).yItem as i32 ;
        (*Rect).right = (*MenuItem).cxItem as i32 ;
        (*Rect).bottom = (*MenuItem).cyItem as i32 ;
    } else {
        return 0;
    }

    if pWnd.is_null() {
        let hWnd: HWND = (*LocalMenu).hWnd;
        pWnd = UserGetWindowObject(hWnd);
        if pWnd.is_null() {
            return 0;
        }
    }

    if (*LocalMenu).fFlags & MNF_POPUP != 0 {
        XMove = (*pWnd).rcClient.left;
        YMove = (*pWnd).rcClient.top;
    } else {
        XMove = (*pWnd).rcWindow.left;
        YMove = (*pWnd).rcWindow.top;
    }

    (*Rect).left += XMove;
    (*Rect).top += YMove;
    (*Rect).right += XMove;
    (*Rect).bottom += YMove;

    1
}

// IntSetSystemMenu (menu.c) — BOOL FASTCALL. Cross-module export (window.rs etc.).
#[cfg(target_arch = "x86")]
#[no_mangle]
unsafe extern "fastcall" fn IntSetSystemMenu(Window: PWND, Menu: PMENU) -> BOOL {
    IntSetSystemMenu_impl(Window, Menu)
}
#[cfg(not(target_arch = "x86"))]
#[no_mangle]
unsafe extern "C" fn IntSetSystemMenu(Window: PWND, Menu: PMENU) -> BOOL {
    IntSetSystemMenu_impl(Window, Menu)
}
#[inline(always)]
unsafe fn IntSetSystemMenu_impl(Window: PWND, Menu: PMENU) -> BOOL {
    if (*Window).style & WS_SYSMENU == 0 {
        return FALSE as BOOL;
    }

    if !(*Window).SystemMenu.is_null() {
        let OldMenu = UserGetMenuObject((*Window).SystemMenu);
        if !OldMenu.is_null() {
            (*OldMenu).fFlags &= !MNF_SYSMENU;
            IntDestroyMenuObject(OldMenu, TRUE as BOOL);
        }
    }

    let OldMenu = MENU_GetSystemMenu(Window, Menu);
    if !OldMenu.is_null() {
        // Use spmenuSys too!
        (*Window).SystemMenu = UserHMGetHandle(OldMenu) as HMENU;
    } else {
        (*Window).SystemMenu = core::ptr::null_mut();
    }

    if !Menu.is_null() && Window != (*Menu).spwndNotify {
        (*Menu).spwndNotify = Window;
    }

    TRUE as BOOL
}

fastcall_fn! {
unsafe fn MENU_GetSystemMenu(Window: PWND, Popup: PMENU) -> PMENU {
    let Menu: PMENU;
    let mut NewMenu: PMENU = core::ptr::null_mut();
    let SysMenu: PMENU;
    let hSysMenu: HMENU;
    let mut hNewMenu: HMENU = core::ptr::null_mut();
    let mut ItemInfoSet: ROSMENUITEMINFO = core::mem::zeroed();
    let mut ItemInfo: ROSMENUITEMINFO = core::mem::zeroed();
    let mut MenuName: _UNICODE_STRING = core::mem::zeroed();

    hSysMenu = UserCreateMenu((*Window).head.rpdesk, 0);
    if hSysMenu.is_null() {
        return core::ptr::null_mut();
    }
    SysMenu = UserGetMenuObject(hSysMenu);
    if SysMenu.is_null() {
        UserDestroyMenu(hSysMenu);
        return core::ptr::null_mut();
    }

    (*SysMenu).fFlags |= MNF_SYSMENU;
    (*SysMenu).hWnd = UserHMGetHandle(Window) as HWND ;

    if Popup.is_null() {
        if (*Window).ExStyle & WS_EX_MDICHILD != 0 {
            const SYSMENUMDI: [u16; 11] = [b'S' as u16, b'Y' as u16, b'S' as u16, b'M' as u16, b'E' as u16, b'N' as u16, b'U' as u16, b'M' as u16, b'D' as u16, b'I' as u16, 0];
            RtlInitUnicodeString((&mut MenuName as *mut UNICODE_STRING).cast(), SYSMENUMDI.as_ptr());
            hNewMenu = co_IntCallLoadMenu(hModClient, &mut MenuName);
        } else {
            const SYSMENU: [u16; 8] = [b'S' as u16, b'Y' as u16, b'S' as u16, b'M' as u16, b'E' as u16, b'N' as u16, b'U' as u16, 0];
            RtlInitUnicodeString((&mut MenuName as *mut UNICODE_STRING).cast(), SYSMENU.as_ptr());
            hNewMenu = co_IntCallLoadMenu(hModClient, &mut MenuName);
        }
        if hNewMenu.is_null() {
            ERR!("No Menu!!\n");
            IntDestroyMenuObject(SysMenu, 0);
            return core::ptr::null_mut();
        }
        Menu = UserGetMenuObject(hNewMenu);
        if Menu.is_null() {
            IntDestroyMenuObject(SysMenu, 0);
            return core::ptr::null_mut();
        }

        (*Menu).fFlags |= MNS_CHECKORBMP | MNF_SYSMENU | MNF_POPUP;

        ItemInfoSet.cbSize = core::mem::size_of::<MENUITEMINFOW>() as u32;
        ItemInfoSet.fMask = MIIM_BITMAP;
        ItemInfoSet.hbmpItem = HBMMENU_POPUP_CLOSE;
        IntMenuItemInfo(Menu, SC_CLOSE, 0, &mut ItemInfoSet, 1, core::ptr::null_mut());
        ItemInfoSet.hbmpItem = HBMMENU_POPUP_RESTORE;
        IntMenuItemInfo(Menu, SC_RESTORE, 0, &mut ItemInfoSet, 1, core::ptr::null_mut());
        ItemInfoSet.hbmpItem = HBMMENU_POPUP_MAXIMIZE;
        IntMenuItemInfo(Menu, SC_MAXIMIZE, 0, &mut ItemInfoSet, 1, core::ptr::null_mut());
        ItemInfoSet.hbmpItem = HBMMENU_POPUP_MINIMIZE;
        IntMenuItemInfo(Menu, SC_MINIMIZE, 0, &mut ItemInfoSet, 1, core::ptr::null_mut());

        NewMenu = IntCloneMenu(Menu);
        if NewMenu.is_null() {
            IntDestroyMenuObject(Menu, 0);
            IntDestroyMenuObject(SysMenu, 0);
            return core::ptr::null_mut();
        }

        IntReleaseMenuObject(NewMenu);
        UserSetMenuDefaultItem(NewMenu, SC_CLOSE, 0);

        IntDestroyMenuObject(Menu, 0);
    } else {
        NewMenu = Popup;
    }
    if !NewMenu.is_null() {
        (*NewMenu).fFlags |= MNF_SYSMENU | MNF_POPUP;

        if (*(*Window).pcls).style & CS_NOCLOSE != 0 {
            IntRemoveMenuItem(NewMenu, SC_CLOSE, MF_BYCOMMAND, 1);
        }

        ItemInfo.cbSize = core::mem::size_of::<MENUITEMINFOW>() as u32;
        ItemInfo.fMask = MIIM_FTYPE | MIIM_STRING | MIIM_STATE | MIIM_SUBMENU;
        ItemInfo.fType = MF_POPUP;
        ItemInfo.fState = MFS_ENABLED;
        ItemInfo.dwTypeData = core::ptr::null_mut();
        ItemInfo.cch = 0;
        ItemInfo.hSubMenu = UserHMGetHandle(NewMenu) as HMENU ;
        IntInsertMenuItem(SysMenu, !0u32, 1, &mut ItemInfo, core::ptr::null_mut());

        return SysMenu;
    }
    ERR!("failed to load system menu!\n");
    core::ptr::null_mut()
}
}

#[cfg(target_pointer_width = "32")]
#[no_mangle]
unsafe extern "fastcall" fn IntGetSystemMenu(Window: PWND, bRevert: BOOL) -> PMENU {
    IntGetSystemMenu_impl(Window, bRevert)
}

#[cfg(not(target_pointer_width = "32"))]
#[no_mangle]
unsafe extern "C" fn IntGetSystemMenu(Window: PWND, bRevert: BOOL) -> PMENU {
    IntGetSystemMenu_impl(Window, bRevert)
}

unsafe fn IntGetSystemMenu_impl(Window: PWND, bRevert: BOOL) -> PMENU {

    let mut Menu: PMENU;

    if bRevert != 0 {
        if !(*Window).SystemMenu.is_null() {
            Menu = UserGetMenuObject((*Window).SystemMenu);
            if !Menu.is_null() && ((*Menu).fFlags & MNF_SYSDESKMN) == 0 {
                IntDestroyMenuObject(Menu, 1);
                (*Window).SystemMenu = core::ptr::null_mut();
            }
        }
    } else {
        Menu = if !(*Window).SystemMenu.is_null() {
            UserGetMenuObject((*Window).SystemMenu)
        } else {
            core::ptr::null_mut()
        };
        if (Menu.is_null() || ((*Menu).fFlags & MNF_SYSDESKMN) != 0)
            && ((*Window).style & WS_SYSMENU) != 0
        {
            Menu = MENU_GetSystemMenu(Window, core::ptr::null_mut());
            (*Window).SystemMenu = if !Menu.is_null() {
                UserHMGetHandle(Menu) as HMENU
            } else {
                core::ptr::null_mut()
            };
        }
    }

    if !(*Window).SystemMenu.is_null() {
        let hMenu = IntGetSubMenu((*Window).SystemMenu, 0);
        Menu = UserGetMenuObject(hMenu);
        if !Menu.is_null() {
            (*Menu).spwndNotify = Window;
            (*Menu).fFlags |= MNF_SYSSUBMENU;
        }
        return Menu;
    }
    core::ptr::null_mut()
}

fastcall_fn! {
#[no_mangle]
unsafe fn IntSetMenu(
    Wnd: PWND,
    Menu: HMENU,
    Changed: *mut BOOL,
) -> BOOL {
    let OldMenu: PMENU;
    let mut NewMenu: PMENU = core::ptr::null_mut();

    if ((*Wnd).style & (WS_CHILD | WS_POPUP)) == WS_CHILD {
        EngSetLastError(ERROR_INVALID_WINDOW_HANDLE);
        return 0;
    }

    *Changed = (UlongToHandle((*Wnd).IDMenu as u32) != (Menu as *mut core::ffi::c_void)) as BOOL;
    if *Changed == 0 {
        return 1;
    }

    if (*Wnd).IDMenu != 0 {
        OldMenu = IntGetMenuObject(UlongToHandle((*Wnd).IDMenu as u32) as HMENU);
    } else {
        OldMenu = core::ptr::null_mut();
    }

    if !Menu.is_null() {
        NewMenu = IntGetMenuObject(Menu);
        if NewMenu.is_null() {
            if !OldMenu.is_null() {
                IntReleaseMenuObject(OldMenu);
            }
            EngSetLastError(ERROR_INVALID_MENU_HANDLE);
            return 0;
        }
        if !(*NewMenu).hWnd.is_null() {
            if !OldMenu.is_null() {
                IntReleaseMenuObject(OldMenu);
            }
            EngSetLastError(ERROR_INVALID_MENU_HANDLE);
            return 0;
        }
    }

    (*Wnd).IDMenu = Menu as UINT_PTR;
    if !NewMenu.is_null() {
        (*NewMenu).hWnd = UserHMGetHandle(Wnd) as HWND ;
        IntReleaseMenuObject(NewMenu);
    }
    if !OldMenu.is_null() {
        (*OldMenu).hWnd = core::ptr::null_mut();
        IntReleaseMenuObject(OldMenu);
    }

    1
}
}
#[no_mangle]
pub unsafe extern "system" fn NtUserCalcMenuBar(
    hwnd: HWND,
    leftBorder: DWORD,
    rightBorder: DWORD,
    top: DWORD,
    prc: LPRECT,
) -> DWORD {
    let hdc: HDC;
    let Window: PWND;
    let mut Rect: RECT = core::mem::zeroed();
    let ret: DWORD;

    UserEnterExclusive();

    Window = UserGetWindowObject(hwnd);
    if Window.is_null() {
        UserLeave();
        EngSetLastError(ERROR_INVALID_WINDOW_HANDLE);
        return 0;
    }

    hdc = UserGetDCEx(core::ptr::null_mut(), core::ptr::null_mut(), DCX_CACHE);
    if hdc.is_null() {
        UserLeave();
        return 0;
    }

    Rect.left = leftBorder as _;
    Rect.right = ((*Window).rcWindow.right - (*Window).rcWindow.left - rightBorder as i32) as _;
    Rect.top = top as _;
    Rect.bottom = 0;

    ret = MENU_DrawMenuBar(hdc, &mut Rect, Window, 1) as DWORD;

    UserReleaseDC(core::ptr::null_mut(), hdc, 0);

    UserLeave();
    ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserCheckMenuItem(
    hMenu: HMENU,
    uIDCheckItem: UINT,
    uCheck: UINT,
) -> DWORD {
    let mut Ret: DWORD = (-1i32) as DWORD;

    UserEnterExclusive();

    let Menu: PMENU = UserGetMenuObject(hMenu);
    if !Menu.is_null() {
        Ret = IntCheckMenuItem(Menu, uIDCheckItem, uCheck);
    }

    UserLeave();
    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserDeleteMenu(
    hMenu: HMENU,
    uPosition: UINT,
    uFlags: UINT,
) -> BOOL {
    let Menu: PMENU;
    let mut Ret: BOOL = 0;

    UserEnterExclusive();

    Menu = UserGetMenuObject(hMenu);
    if !Menu.is_null() {
        Ret = IntRemoveMenuItem(Menu, uPosition, uFlags, 1);
    }

    UserLeave();
    Ret
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn NtUserGetSystemMenu(hWnd: HWND, bRevert: BOOL) -> HMENU {
    let Window: PWND;
    let Menu: PMENU;
    let mut Ret: HMENU = core::ptr::null_mut();

    UserEnterExclusive();

    'Exit: {
        Window = UserGetWindowObject(hWnd);
        if Window.is_null() {
            break 'Exit;
        }

        Menu = IntGetSystemMenu(Window, bRevert);
        if Menu.is_null() {
            break 'Exit;
        }

        Ret = (UserHMGetHandle(Menu) as HMENU);
    }

    UserLeave();
    Ret
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn NtUserGetSystemMenu(hWnd: HWND, bRevert: BOOL) -> HMENU {
    let Window: PWND;
    let Menu: PMENU;
    let mut Ret: HMENU = core::ptr::null_mut();

    UserEnterExclusive();

    'Exit: {
        Window = UserGetWindowObject(hWnd);
        if Window.is_null() {
            break 'Exit;
        }

        Menu = IntGetSystemMenu(Window, bRevert);
        if Menu.is_null() {
            break 'Exit;
        }

        Ret = UserHMGetHandle(Menu) as HMENU;
    }

    UserLeave();
    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserSetSystemMenu(hWnd: HWND, hMenu: HMENU) -> BOOL {
    let mut Result: BOOL = 0;
    let Window: PWND;
    let Menu: PMENU;

    UserEnterExclusive();

    'exit: {
        Window = UserGetWindowObject(hWnd);
        if Window.is_null() {
            break 'exit;
        }

        if !hMenu.is_null() {
            Menu = IntGetMenuObject(hMenu);
            if Menu.is_null() {
                break 'exit;
            }

            Result = IntSetSystemMenu(Window, Menu);
        } else {
            EngSetLastError(ERROR_INVALID_MENU_HANDLE);
        }
    }

    UserLeave();
    Result
}
#[no_mangle]
pub unsafe extern "system" fn NtUserGetTitleBarInfo(
    hwnd: HWND,
    bti: PTITLEBARINFO,
) -> BOOLEAN {
    let WindowObject: PWND;
    let mut bartitleinfo: TITLEBARINFO = core::mem::zeroed();
    let mut retValue: BOOLEAN = 0u8;
    let Status: NTSTATUS;

    UserEnterExclusive();

    'exit: {
        WindowObject = UserGetWindowObject(hwnd);
        if WindowObject.is_null() {
            EngSetLastError(ERROR_INVALID_WINDOW_HANDLE);
            break 'exit;
        }

        {
            let __SehStatus = SafeProbeForRead(
                bti as *mut core::ffi::c_void,
                core::mem::size_of::<TITLEBARINFO>() as _,
                1,
            );
            if !NT_SUCCESS(__SehStatus) {
                EngSetLastError(ERROR_INVALID_PARAMETER);
                break 'exit;
            }
        }
        if !crate::copy::capture_n(
            &mut bartitleinfo as *mut TITLEBARINFO,
            bti,
            1,
        ) {
            EngSetLastError(ERROR_INVALID_PARAMETER);
            break 'exit;
        }

        retValue = intGetTitleBarInfo(WindowObject, &mut bartitleinfo as *mut TITLEBARINFO);
        if retValue == 0 {
            break 'exit;
        }

        Status = SafeProbeForWrite(
            bti as *mut core::ffi::c_void,
            core::mem::size_of::<TITLEBARINFO>() as _,
            1,
        );
        if !NT_SUCCESS(Status) {
            EngSetLastError(ERROR_INVALID_PARAMETER);
            retValue = 0u8;
        } else {
            core::ptr::copy_nonoverlapping(
                &bartitleinfo as *const TITLEBARINFO,
                bti,
                1,
            );
        }
    }

    UserLeave();
    retValue
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn UserDestroyMenu(hMenu: HMENU) -> BOOL {
    UserDestroyMenu_impl(hMenu)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn UserDestroyMenu(hMenu: HMENU) -> BOOL {
    UserDestroyMenu_impl(hMenu)
}

unsafe fn UserDestroyMenu_impl(hMenu: HMENU) -> BOOL {
    let pti: PTHREADINFO = PsGetCurrentThreadWin32Thread() as PTHREADINFO;

    let Menu: PMENU = UserGetMenuObject(hMenu);
    if Menu.is_null() {
        return 0;
    }

    if (*Menu).head.rpdesk != (*pti).rpdesk {
        EngSetLastError(ERROR_ACCESS_DENIED);
        return 0;
    }
    IntDestroyMenuObject(Menu, 0)
}
#[no_mangle]
pub unsafe extern "system" fn NtUserDestroyMenu(hMenu: HMENU) -> BOOL {
    let Menu: PMENU;
    let mut Ret: BOOL = 0;

    UserEnterExclusive();

    'Exit: {
        Menu = UserGetMenuObject(hMenu);
        if Menu.is_null() {
            break 'Exit;
        }
        if (*Menu).head.rpdesk != (*gptiCurrent).rpdesk {
            EngSetLastError(ERROR_ACCESS_DENIED);
            break 'Exit;
        }
        Ret = IntDestroyMenuObject(Menu, 1) as BOOL;
    }

    UserLeave();
    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserEnableMenuItem(
    hMenu: HMENU,
    uIDEnableItem: UINT,
    uEnable: UINT,
) -> UINT {
    let Menu: PMENU;
    let mut Ret: UINT = (-1i32) as UINT;

    UserEnterExclusive();

    Menu = UserGetMenuObject(hMenu);
    if !Menu.is_null() {
        Ret = IntEnableMenuItem(Menu, uIDEnableItem, uEnable);
    }

    UserLeave();
    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserEndMenu() -> BOOL {
    UserEnterExclusive();
    if fInsideMenuLoop != 0 && !top_popup.is_null() {
        fInsideMenuLoop = 0;
        UserPostMessage(top_popup, WM_CANCELMODE, 0, 0);
    }
    UserLeave();
    1
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn NtUserGetMenuBarInfo(
    hwnd: HWND,
    idObject: LONG,
    idItem: LONG,
    pmbi: PMENUBARINFO,
) -> BOOL {
    let mut pWnd: PWND = core::ptr::null_mut();
    let mut hMenu: HMENU;
    let mut kmbi: MENUBARINFO = core::mem::zeroed();
    let mut Ret: BOOL = 0;
    let pPopupMenu: PPOPUPMENU;
    let mut Ref: USER_REFERENCE_ENTRY = core::mem::zeroed();
    let mut Status: NTSTATUS = STATUS_SUCCESS as NTSTATUS;
    let mut Menu: PMENU = core::ptr::null_mut();

    UserEnterShared();

    'cleanup: {
        pWnd = UserGetWindowObject(hwnd);
        if pWnd.is_null() {
            EngSetLastError(ERROR_INVALID_WINDOW_HANDLE);
            break 'cleanup;
        }

        UserRefObjectCo(pWnd, &mut Ref);

        RECTL_vSetEmptyRect(core::ptr::addr_of_mut!(kmbi.rcBar).cast());
        kmbi.hMenu = core::ptr::null_mut();
        kmbi.hwndMenu = core::ptr::null_mut();
        kmbi.set_fBarFocused(0);
        kmbi.set_fFocused(0);

        match idObject {
            x if x == OBJID_CLIENT as LONG => {
                if (*(*pWnd).pcls).fnid == 0 {
                    break 'cleanup;
                }
                if (*(*pWnd).pcls).fnid as u32 != FNID_MENU {
                    EngSetLastError(ERROR_INVALID_MENU_HANDLE);
                    break 'cleanup;
                }
                hMenu = co_IntSendMessage(hwnd, MN_GETHMENU, 0, 0) as HMENU;
                pPopupMenu = (*(pWnd as PMENUWND)).ppopupmenu;
                if !pPopupMenu.is_null() && !(*pPopupMenu).spmenu.is_null() {
                    if UserHMGetHandle((*pPopupMenu).spmenu) as HMENU != hMenu {
                        ERR!(
                            "Window Pop Up hMenu %p not the same as Get hMenu %p!\n",
                            UserHMGetHandle((*pPopupMenu).spmenu),
                            hMenu,
                        );
                    }
                }
            }
            x if x == OBJID_MENU as LONG => {
                if (*pWnd).style & WS_CHILD as u32 != 0 {
                    break 'cleanup;
                }
                hMenu = UlongToHandle((*pWnd).IDMenu as u32) as HMENU;
            }
            x if x == OBJID_SYSMENU as LONG => {
                if (*pWnd).style & WS_SYSMENU as u32 == 0 {
                    break 'cleanup;
                }
                Menu = IntGetSystemMenu(pWnd, 0);
                hMenu = UserHMGetHandle(Menu) as HMENU;
            }
            _ => {
                break 'cleanup;
            }
        }

        if hMenu.is_null() {
            break 'cleanup;
        }

        Status = SafeProbeForRead(
            pmbi as *mut core::ffi::c_void,
            (core::mem::size_of::<MENUBARINFO>() as u32),
            1,
        );
        if !NT_SUCCESS(Status) {
            kmbi.cbSize = 0;
        } else if !crate::copy::capture_from_user(
            core::ptr::addr_of_mut!(kmbi.cbSize) as *mut core::ffi::c_void,
            core::ptr::addr_of!((*pmbi).cbSize) as *const core::ffi::c_void,
            core::mem::size_of_val(&kmbi.cbSize),
        ) {
            // Probe is validate-only; the C had probe AND load in one _SEH2_TRY.
            kmbi.cbSize = 0;
        }

        if kmbi.cbSize as usize != core::mem::size_of::<MENUBARINFO>() {
            EngSetLastError(ERROR_INVALID_PARAMETER);
            break 'cleanup;
        }

        if Menu.is_null() {
            Menu = UserGetMenuObject(hMenu);
            if Menu.is_null() {
                break 'cleanup;
            }
        }

        if (idItem < 0) || ((idItem as ULONG) > (*Menu).cItems) {
            break 'cleanup;
        }

        if idItem == 0 {
            Ret = IntGetMenuItemRect(pWnd, Menu, 0, core::ptr::addr_of_mut!(kmbi.rcBar));
            kmbi.rcBar.right = kmbi.rcBar.left + ((*Menu).cxMenu as i32);
            kmbi.rcBar.bottom = kmbi.rcBar.top + ((*Menu).cyMenu as i32);
        } else {
            Ret = IntGetMenuItemRect(
                pWnd,
                Menu,
                (idItem - 1) as UINT,
                core::ptr::addr_of_mut!(kmbi.rcBar),
            );
        }

        kmbi.hMenu = hMenu;
        kmbi.set_fBarFocused((top_popup_hmenu == hMenu) as BOOL);
        if idItem != 0 {
            kmbi.set_fFocused(((*Menu).iItem == idItem - 1) as BOOL);
            if kmbi.fFocused() != 0
                && !(*(*Menu).rgItems.offset((idItem - 1) as isize)).spSubMenu.is_null()
            {
                kmbi.hwndMenu =
                    (*(*(*Menu).rgItems.offset((idItem - 1) as isize)).spSubMenu).hWnd;
            }
        } else {
            kmbi.set_fFocused(kmbi.fBarFocused());
        }

        Status = SafeProbeForWrite(
            pmbi as *mut core::ffi::c_void,
            core::mem::size_of::<MENUBARINFO>(),
            1,
        );
        if NT_SUCCESS(Status) {
            RtlCopyMemory(
                pmbi as *mut core::ffi::c_void,
                core::ptr::addr_of!(kmbi) as *const core::ffi::c_void,
                (core::mem::size_of::<MENUBARINFO>() as u32),
            );
        }

        if !NT_SUCCESS(Status) {
            SetLastNtError(Status);
            Ret = 0;
            break 'cleanup;
        }

        Ret = 1;
    }

    if !pWnd.is_null() {
        UserDerefObjectCo(pWnd);
    }
    UserLeave();
    Ret
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn NtUserGetMenuBarInfo(
    hwnd: HWND,
    idObject: LONG,
    idItem: LONG,
    pmbi: PMENUBARINFO,
) -> BOOL {
    let mut pWnd: PWND = core::ptr::null_mut();
    let mut hMenu: HMENU;
    let mut kmbi: MENUBARINFO = core::mem::zeroed();
    let mut Ret: BOOL = 0;
    let pPopupMenu: PPOPUPMENU;
    let mut Ref: USER_REFERENCE_ENTRY = core::mem::zeroed();
    let mut Status: NTSTATUS = STATUS_SUCCESS as NTSTATUS;
    let mut Menu: PMENU = core::ptr::null_mut();

    UserEnterShared();

    'cleanup: {
        pWnd = UserGetWindowObject(hwnd);
        if pWnd.is_null() {
            EngSetLastError(ERROR_INVALID_WINDOW_HANDLE);
            break 'cleanup;
        }

        UserRefObjectCo(pWnd, &mut Ref);

        RECTL_vSetEmptyRect(core::ptr::addr_of_mut!(kmbi.rcBar).cast());
        kmbi.hMenu = core::ptr::null_mut();
        kmbi.hwndMenu = core::ptr::null_mut();
        kmbi.set_fBarFocused(0);
        kmbi.set_fFocused(0);

        match idObject {
            x if x == OBJID_CLIENT as LONG => {
                if (*(*pWnd).pcls).fnid == 0 {
                    break 'cleanup;
                }
                if (*(*pWnd).pcls).fnid as u32 != FNID_MENU {
                    EngSetLastError(ERROR_INVALID_MENU_HANDLE);
                    break 'cleanup;
                }
                hMenu = co_IntSendMessage(hwnd, MN_GETHMENU, 0, 0) as HMENU;
                pPopupMenu = (*(pWnd as PMENUWND)).ppopupmenu;
                if !pPopupMenu.is_null() && !(*pPopupMenu).spmenu.is_null() {
                    if UserHMGetHandle((*pPopupMenu).spmenu) as HMENU != hMenu {
                        ERR!(
                            "Window Pop Up hMenu %p not the same as Get hMenu %p!\n",
                            UserHMGetHandle((*pPopupMenu).spmenu),
                            hMenu,
                        );
                    }
                }
            }
            x if x == OBJID_MENU as LONG => {
                if (*pWnd).style & WS_CHILD as u32 != 0 {
                    break 'cleanup;
                }
                hMenu = UlongToHandle((*pWnd).IDMenu as u32) as HMENU;
            }
            x if x == OBJID_SYSMENU as LONG => {
                if (*pWnd).style & WS_SYSMENU as u32 == 0 {
                    break 'cleanup;
                }
                Menu = IntGetSystemMenu(pWnd, 0);
                hMenu = UserHMGetHandle(Menu) as HMENU;
            }
            _ => {
                break 'cleanup;
            }
        }

        if hMenu.is_null() {
            break 'cleanup;
        }

        Status = SafeProbeForRead(
            pmbi as *mut core::ffi::c_void,
            core::mem::size_of::<MENUBARINFO>() as _,
            1,
        );
        if !NT_SUCCESS(Status) {
            kmbi.cbSize = 0;
        } else if !crate::copy::capture_from_user(
            core::ptr::addr_of_mut!(kmbi.cbSize) as *mut core::ffi::c_void,
            core::ptr::addr_of!((*pmbi).cbSize) as *const core::ffi::c_void,
            core::mem::size_of_val(&kmbi.cbSize),
        ) {
            // Probe is validate-only; the C had probe AND load in one _SEH2_TRY.
            kmbi.cbSize = 0;
        }

        if kmbi.cbSize as usize != core::mem::size_of::<MENUBARINFO>() {
            EngSetLastError(ERROR_INVALID_PARAMETER);
            break 'cleanup;
        }

        if Menu.is_null() {
            Menu = UserGetMenuObject(hMenu);
            if Menu.is_null() {
                break 'cleanup;
            }
        }

        if (idItem < 0) || ((idItem as ULONG) > (*Menu).cItems) {
            break 'cleanup;
        }

        if idItem == 0 {
            Ret = IntGetMenuItemRect(pWnd, Menu, 0, core::ptr::addr_of_mut!(kmbi.rcBar));
            kmbi.rcBar.right = kmbi.rcBar.left + (*Menu).cxMenu as i32;
            kmbi.rcBar.bottom = kmbi.rcBar.top + (*Menu).cyMenu as i32;
        } else {
            Ret = IntGetMenuItemRect(
                pWnd,
                Menu,
                (idItem - 1) as UINT,
                core::ptr::addr_of_mut!(kmbi.rcBar),
            );
        }

        kmbi.hMenu = hMenu;
        kmbi.set_fBarFocused((top_popup_hmenu == hMenu) as BOOL);
        if idItem != 0 {
            kmbi.set_fFocused(((*Menu).iItem == idItem - 1) as BOOL);
            if kmbi.fFocused() != 0
                && !(*(*Menu).rgItems.offset((idItem - 1) as isize)).spSubMenu.is_null()
            {
                kmbi.hwndMenu =
                    (*(*(*Menu).rgItems.offset((idItem - 1) as isize)).spSubMenu).hWnd;
            }
        } else {
            kmbi.set_fFocused(kmbi.fBarFocused());
        }

        Status = SafeProbeForWrite(
            pmbi as *mut core::ffi::c_void,
            core::mem::size_of::<MENUBARINFO>(),
            1,
        );
        if NT_SUCCESS(Status) {
            RtlCopyMemory(
                pmbi as *mut core::ffi::c_void,
                core::ptr::addr_of!(kmbi) as *const core::ffi::c_void,
                core::mem::size_of::<MENUBARINFO>() as SIZE_T,
            );
        }

        if !NT_SUCCESS(Status) {
            SetLastNtError(Status);
            Ret = 0;
            break 'cleanup;
        }

        Ret = 1;
    }

    if !pWnd.is_null() {
        UserDerefObjectCo(pWnd);
    }
    UserLeave();
    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserGetMenuIndex(
    hMenu: HMENU,
    hSubMenu: HMENU,
) -> UINT {
    let Menu: PMENU;
    let SubMenu: PMENU;
    let mut MenuItem: PITEM;
    let mut i: UINT;
    let mut Ret: UINT = 0xFFFFFFFF;

    UserEnterShared();

    'Exit: {
        Menu = UserGetMenuObject(hMenu);
        if Menu.is_null() {
            break 'Exit;
        }
        SubMenu = UserGetMenuObject(hSubMenu);
        if SubMenu.is_null() {
            break 'Exit;
        }

        MenuItem = (*Menu).rgItems;
        i = 0;
        while i < (*Menu).cItems {
            if (*MenuItem).spSubMenu == SubMenu {
                Ret = (*MenuItem).wID;
                break;
            }
            i += 1;
            MenuItem = MenuItem.add(1);
        }
    }

    UserLeave();
    Ret
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn NtUserGetMenuItemRect(
    mut hWnd: HWND,
    hMenu: HMENU,
    mut uItem: UINT,
    lprcItem: PRECTL,
) -> BOOL {
    let ReferenceWnd: PWND;
    let XMove: LONG;
    let YMove: LONG;
    let mut Rect: RECTL = core::mem::zeroed();
    let mut Menu: PMENU;
    let MenuItem: PITEM;
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut Ret: BOOL = 0;

    UserEnterShared();

    'exit: {
        Menu = UserGetMenuObject(hMenu);
        if Menu.is_null() {
            break 'exit;
        }

        MenuItem = MENU_FindItem(
            core::ptr::addr_of_mut!(Menu) as *mut PMENU,
            &mut uItem,
            MF_BYPOSITION,
        );
        if !MenuItem.is_null() {
            Rect.left = ((*MenuItem).xItem as i32);
            Rect.top = ((*MenuItem).yItem as i32);
            Rect.right = ((*MenuItem).cxItem as i32);
            Rect.bottom = ((*MenuItem).cyItem as i32);
        } else {
            break 'exit;
        }

        if hWnd.is_null() {
            hWnd = (*Menu).hWnd;
        }

        if lprcItem.is_null() {
            break 'exit;
        }

        ReferenceWnd = UserGetWindowObject(hWnd);
        if ReferenceWnd.is_null() {
            break 'exit;
        }

        if (*Menu).fFlags & MNF_POPUP != 0 {
            XMove = (*ReferenceWnd).rcClient.left;
            YMove = (*ReferenceWnd).rcClient.top;
        } else {
            XMove = (*ReferenceWnd).rcWindow.left;
            YMove = (*ReferenceWnd).rcWindow.top;
        }

        Rect.left += XMove;
        Rect.top += YMove;
        Rect.right += XMove;
        Rect.bottom += YMove;

        // ---- dropped _SEH2_TRY/_SEH2_EXCEPT recovered here ----
        // C (menu.c:6181-6189, NtUserGetMenuItemRect): bare
        //   _SEH2_TRY { RtlCopyMemory(lprcItem, &Rect, sizeof(RECTL)); }
        //   _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER) { Status = _SEH2_GetExceptionCode(); }
        //   _SEH2_END;
        // with no ProbeFor* call at all -- the same bare-store shape as
        // console.rs's ConsoleCtrlDesktopConsoleThread case, so
        // `safe_copy_raw` (no probing, just a fault-tolerant copy) is the
        // faithful replacement. Handler only assigns Status and falls
        // through -- no return/goto inside the SEH itself -- to the
        // `if (!NT_SUCCESS(Status))` check that already exists right below.
        if let Err(st) = ntkrnl_sys::probe::safe_copy_raw(
            lprcItem as *mut c_void,
            core::ptr::addr_of!(Rect) as *const c_void,
            core::mem::size_of::<RECTL>(),
        ) {
            Status = st;
        }

        if !NT_SUCCESS(Status) {
            SetLastNtError(Status);
            break 'exit;
        }
        Ret = 1;
    }

    UserLeave();
    Ret
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn NtUserGetMenuItemRect(
    mut hWnd: HWND,
    hMenu: HMENU,
    mut uItem: UINT,
    lprcItem: PRECTL,
) -> BOOL {
    let ReferenceWnd: PWND;
    let XMove: LONG;
    let YMove: LONG;
    let mut Rect: RECTL = core::mem::zeroed();
    let mut Menu: PMENU;
    let MenuItem: PITEM;
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut Ret: BOOL = 0;

    UserEnterShared();

    'exit: {
        Menu = UserGetMenuObject(hMenu);
        if Menu.is_null() {
            break 'exit;
        }

        MenuItem = MENU_FindItem(
            core::ptr::addr_of_mut!(Menu) as *mut PMENU,
            &mut uItem,
            MF_BYPOSITION,
        );
        if !MenuItem.is_null() {
            Rect.left = (*MenuItem).xItem as i32;
            Rect.top = (*MenuItem).yItem as i32;
            Rect.right = (*MenuItem).cxItem as i32;
            Rect.bottom = (*MenuItem).cyItem as i32;
        } else {
            break 'exit;
        }

        if hWnd.is_null() {
            hWnd = (*Menu).hWnd;
        }

        if lprcItem.is_null() {
            break 'exit;
        }

        ReferenceWnd = UserGetWindowObject(hWnd);
        if ReferenceWnd.is_null() {
            break 'exit;
        }

        if (*Menu).fFlags & MNF_POPUP != 0 {
            XMove = (*ReferenceWnd).rcClient.left;
            YMove = (*ReferenceWnd).rcClient.top;
        } else {
            XMove = (*ReferenceWnd).rcWindow.left;
            YMove = (*ReferenceWnd).rcWindow.top;
        }

        Rect.left += XMove;
        Rect.top += YMove;
        Rect.right += XMove;
        Rect.bottom += YMove;

        // ---- dropped _SEH2_TRY/_SEH2_EXCEPT recovered here ----
        // C (menu.c:6181-6189, NtUserGetMenuItemRect): bare
        //   _SEH2_TRY { RtlCopyMemory(lprcItem, &Rect, sizeof(RECTL)); }
        //   _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER) { Status = _SEH2_GetExceptionCode(); }
        //   _SEH2_END;
        // with no ProbeFor* call at all -- the same bare-store shape as
        // console.rs's ConsoleCtrlDesktopConsoleThread case, so
        // `safe_copy_raw` (no probing, just a fault-tolerant copy) is the
        // faithful replacement. Handler only assigns Status and falls
        // through -- no return/goto inside the SEH itself -- to the
        // `if (!NT_SUCCESS(Status))` check that already exists right below.
        if let Err(st) = ntkrnl_sys::probe::safe_copy_raw(
            lprcItem as *mut c_void,
            core::ptr::addr_of!(Rect) as *const c_void,
            core::mem::size_of::<RECTL>(),
        ) {
            Status = st;
        }

        if !NT_SUCCESS(Status) {
            SetLastNtError(Status);
            break 'exit;
        }
        Ret = 1;
    }

    UserLeave();
    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserHiliteMenuItem(
    hWnd: HWND,
    hMenu: HMENU,
    uItemHilite: UINT,
    uHilite: UINT,
) -> BOOL {
    let Menu: PMENU;
    let Window: PWND;
    let mut Ret: BOOL = 0;

    UserEnterExclusive();

    'Exit: {
        Window = UserGetWindowObject(hWnd);
        if Window.is_null() {
            EngSetLastError(ERROR_INVALID_WINDOW_HANDLE);
            break 'Exit;
        }

        Menu = UserGetMenuObject(hMenu);
        if Menu.is_null() {
            EngSetLastError(ERROR_INVALID_MENU_HANDLE);
            break 'Exit;
        }

        Ret = IntHiliteMenuItem(Window, Menu, uItemHilite, uHilite);
    }

    UserLeave();
    Ret
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn NtUserDrawMenuBarTemp(
    hWnd: HWND,
    hDC: HDC,
    pRect: PRECT,
    hMenu: HMENU,
    hFont: HFONT,
) -> DWORD {
    let Menu: PMENU;
    let Window: PWND;
    let mut Rect: RECT = core::mem::zeroed();
    let mut Status: NTSTATUS = STATUS_SUCCESS as NTSTATUS;
    let mut Ret: DWORD = 0;

    UserEnterExclusive();

    'exit: {
        Window = UserGetWindowObject(hWnd);
        if Window.is_null() {
            EngSetLastError(ERROR_INVALID_WINDOW_HANDLE);
            break 'exit;
        }

        Menu = UserGetMenuObject(hMenu);
        if Menu.is_null() {
            EngSetLastError(ERROR_INVALID_MENU_HANDLE);
            break 'exit;
        }

        Status = SafeProbeForRead(
            pRect as *mut core::ffi::c_void,
            core::mem::size_of::<RECT>() as ULONG,
            core::mem::size_of::<ULONG>() as ULONG,
        );
        if NT_SUCCESS(Status) {
            core::ptr::copy_nonoverlapping(
                pRect as *const RECT,
                &mut Rect as *mut RECT,
                1,
            );
        }

        if Status != STATUS_SUCCESS as NTSTATUS {
            SetLastNtError(Status);
            break 'exit;
        }

        Ret = IntDrawMenuBarTemp(Window, hDC, &mut Rect, Menu, hFont);
    }

    UserLeave();
    Ret
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn NtUserDrawMenuBarTemp(
    hWnd: HWND,
    hDC: HDC,
    pRect: PRECT,
    hMenu: HMENU,
    hFont: HFONT,
) -> DWORD {
    let Menu: PMENU;
    let Window: PWND;
    let mut Rect: RECT = core::mem::zeroed();
    let mut Status: NTSTATUS = STATUS_SUCCESS as NTSTATUS;
    let mut Ret: DWORD = 0;

    UserEnterExclusive();

    'exit: {
        Window = UserGetWindowObject(hWnd);
        if Window.is_null() {
            EngSetLastError(ERROR_INVALID_WINDOW_HANDLE);
            break 'exit;
        }

        Menu = UserGetMenuObject(hMenu);
        if Menu.is_null() {
            EngSetLastError(ERROR_INVALID_MENU_HANDLE);
            break 'exit;
        }

        Status = SafeProbeForRead(
            pRect as *mut core::ffi::c_void,
            core::mem::size_of::<RECT>() as _,
            core::mem::size_of::<ULONG>() as ULONG,
        );
        if NT_SUCCESS(Status) {
            core::ptr::copy_nonoverlapping(
                pRect as *const RECT,
                &mut Rect as *mut RECT,
                1,
            );
        }

        if Status != STATUS_SUCCESS as NTSTATUS {
            SetLastNtError(Status);
            break 'exit;
        }

        Ret = IntDrawMenuBarTemp(Window, hDC, &mut Rect, Menu, hFont);
    }

    UserLeave();
    Ret
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn NtUserMenuItemFromPoint(
    hWnd: HWND,
    hMenu: HMENU,
    mut X: DWORD,
    mut Y: DWORD,
) -> ::core::ffi::c_int {
    let Menu: PMENU;
    let Window: PWND;
    let mut mi: PITEM;
    let mut i: ULONG;
    let mut Ret: ::core::ffi::c_int = -1;

    UserEnterExclusive();

    'exit: {
        Menu = UserGetMenuObject(hMenu);
        if Menu.is_null() {
            break 'exit;
        }

        Window = UserGetWindowObject((*Menu).hWnd);
        if Window.is_null() {
            break 'exit;
        }

        X -= (*Window).rcWindow.left as DWORD;
        Y -= (*Window).rcWindow.top as DWORD;

        mi = (*Menu).rgItems;
        i = 0;
        while i < (*Menu).cItems {
            let mut Rect: RECTL = core::mem::zeroed();

            Rect.left = ((*mi).xItem as i32);
            Rect.top = ((*mi).yItem as i32);
            Rect.right = ((*mi).cxItem as i32);
            Rect.bottom = ((*mi).cyItem as i32);

            MENU_AdjustMenuItemRect(Menu, &mut Rect);

            if RECTL_bPointInRect(&Rect, X as INT, Y as INT) != 0 {
                break;
            }

            i += 1;
            mi = mi.add(1);
        }

        Ret = if !mi.is_null() {
            i as ::core::ffi::c_int
        } else {
            NO_SELECTED_ITEM as ::core::ffi::c_int
        };
    }

    UserLeave();
    Ret
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn NtUserMenuItemFromPoint(
    hWnd: HWND,
    hMenu: HMENU,
    mut X: DWORD,
    mut Y: DWORD,
) -> ::core::ffi::c_int {
    let Menu: PMENU;
    let Window: PWND;
    let mut mi: PITEM;
    let mut i: ULONG;
    let mut Ret: ::core::ffi::c_int = -1;

    UserEnterExclusive();

    'exit: {
        Menu = UserGetMenuObject(hMenu);
        if Menu.is_null() {
            break 'exit;
        }

        Window = UserGetWindowObject((*Menu).hWnd);
        if Window.is_null() {
            break 'exit;
        }

        X -= (*Window).rcWindow.left as DWORD;
        Y -= (*Window).rcWindow.top as DWORD;

        mi = (*Menu).rgItems;
        i = 0;
        while i < (*Menu).cItems {
            let mut Rect: RECTL = core::mem::zeroed();

            Rect.left = (*mi).xItem as i32;
            Rect.top = (*mi).yItem as i32;
            Rect.right = (*mi).cxItem as i32;
            Rect.bottom = (*mi).cyItem as i32;

            MENU_AdjustMenuItemRect(Menu, &mut Rect);

            if RECTL_bPointInRect(&Rect, X as INT, Y as INT) != 0 {
                break;
            }

            i += 1;
            mi = mi.add(1);
        }

        Ret = if !mi.is_null() {
            i as ::core::ffi::c_int
        } else {
            NO_SELECTED_ITEM as ::core::ffi::c_int
        };
    }

    UserLeave();
    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserPaintMenuBar(
    hWnd: HWND,
    hDC: HDC,
    leftBorder: ULONG,
    rightBorder: ULONG,
    top: ULONG,
    bActive: BOOL,
) -> DWORD {
    let Window: PWND;
    let mut Rect: RECT = core::mem::zeroed();
    let ret: DWORD;

    UserEnterExclusive();

    Window = UserGetWindowObject(hWnd);
    if Window.is_null() {
        UserLeave();
        EngSetLastError(ERROR_INVALID_WINDOW_HANDLE);
        return 0;
    }

    Rect.left = leftBorder as _;
    Rect.right = (*Window).rcWindow.right - (*Window).rcWindow.left - rightBorder as i32;
    Rect.top = top as _;
    Rect.bottom = 0;

    ret = MENU_DrawMenuBar(hDC, &mut Rect, Window, 0) as DWORD;

    UserLeave();
    ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserRemoveMenu(
    hMenu: HMENU,
    uPosition: UINT,
    uFlags: UINT,
) -> BOOL {
    let Menu: PMENU;
    let mut Ret: BOOL = 0;

    UserEnterExclusive();

    Menu = UserGetMenuObject(hMenu);
    if !Menu.is_null() {
        Ret = IntRemoveMenuItem(Menu, uPosition, uFlags, 0);
    }

    UserLeave();
    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserSetMenu(
    hWnd: HWND,
    Menu: HMENU,
    Repaint: BOOL,
) -> BOOL {
    let Window: PWND;
    let mut Changed: BOOL = 0;
    let mut Ret: BOOL = 0;

    UserEnterExclusive();

    'block: {
        Window = UserGetWindowObject(hWnd);
        if Window.is_null() {
            break 'block;
        }

        if IntSetMenu(Window, Menu, &mut Changed) == 0 {
            break 'block;
        }

        if ((*Window).style & WS_MINIMIZE) == 0 && (Repaint != 0 || Changed != 0) {
            let mut Ref: USER_REFERENCE_ENTRY = core::mem::zeroed();
            UserRefObjectCo(Window, &mut Ref);
            co_WinPosSetWindowPos(
                Window,
                0 as HWND,
                0,
                0,
                0,
                0,
                SWP_NOSIZE | SWP_NOMOVE | SWP_NOACTIVATE | SWP_NOZORDER | SWP_FRAMECHANGED,
            );
            UserDerefObjectCo(Window);
        }

        Ret = 1;
    }

    UserLeave();
    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserSetMenuContextHelpId(
    hMenu: HMENU,
    dwContextHelpId: u32,
) -> i32 {
    let Menu: PMENU;
    let mut Ret: i32 = 0;

    UserEnterExclusive();

    Menu = UserGetMenuObject(hMenu);
    if !Menu.is_null() {
        Ret = IntSetMenuContextHelpId(Menu, dwContextHelpId);
    }

    UserLeave();
    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserSetMenuDefaultItem(
    hMenu: HMENU,
    uItem: UINT,
    fByPos: UINT,
) -> BOOL {
    let Menu: PMENU;
    let mut Ret: BOOL = 0;

    UserEnterExclusive();

    Menu = UserGetMenuObject(hMenu);
    if !Menu.is_null() {
        Ret = UserSetMenuDefaultItem(Menu, uItem, fByPos);
    }

    UserLeave();
    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserSetMenuFlagRtoL(hMenu: HMENU) -> BOOL {
    let Menu: PMENU;
    let mut Ret: BOOL = 0;

    UserEnterExclusive();

    Menu = UserGetMenuObject(hMenu);
    if !Menu.is_null() {
        Ret = IntSetMenuFlagRtoL(Menu);
    }

    UserLeave();
    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserThunkedMenuInfo(
    hMenu: HMENU,
    lpcmi: LPCMENUINFO,
) -> i32 {
    let Menu: PMENU;
    let mut Ret: i32 = 0;

    UserEnterExclusive();

    Menu = UserGetMenuObject(hMenu);
    if !Menu.is_null() {
        Ret = UserMenuInfo(Menu, lpcmi as PROSMENUINFO, 1);
    }

    UserLeave();
    Ret
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn NtUserThunkedMenuItemInfo(
    hMenu: HMENU,
    uItem: UINT,
    fByPosition: BOOL,
    bInsert: BOOL,
    lpmii: LPMENUITEMINFOW,
    lpszCaption: PUNICODE_STRING,
) -> BOOL {
    let Menu: PMENU;
    let Status: NTSTATUS;
    let mut lstrCaption: _UNICODE_STRING = core::mem::zeroed();
    let mut Ret: BOOL = 0;

    UserEnterExclusive();

    'cleanup: {
        Menu = UserGetMenuObject(hMenu);
        if Menu.is_null() {
            break 'cleanup;
        }

        if !lpszCaption.is_null() && !(*lpszCaption).Buffer.is_null() {
            Status = ProbeAndCaptureUnicodeString(
                core::ptr::addr_of_mut!(lstrCaption),
                (1i32 as KPROCESSOR_MODE as u8),
                lpszCaption,
            );
            if !NT_SUCCESS(Status) {
                SetLastNtError(Status);
                break 'cleanup;
            }
        }

        if bInsert != 0 {
            Ret = UserInsertMenuItem(
                Menu,
                uItem,
                fByPosition,
                lpmii,
                core::ptr::addr_of_mut!(lstrCaption),
            ) as BOOL;
        } else {
            Ret = UserMenuItemInfo(
                Menu,
                uItem,
                fByPosition,
                lpmii as PROSMENUITEMINFO,
                1,
                core::ptr::addr_of_mut!(lstrCaption),
            );
        }
    }

    UserLeave();

    if !lstrCaption.Buffer.is_null() {
        ReleaseCapturedUnicodeString(core::ptr::addr_of_mut!(lstrCaption), (1i32 as KPROCESSOR_MODE as u8));
    }

    Ret
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn NtUserThunkedMenuItemInfo(
    hMenu: HMENU,
    uItem: UINT,
    fByPosition: BOOL,
    bInsert: BOOL,
    lpmii: LPMENUITEMINFOW,
    lpszCaption: PUNICODE_STRING,
) -> BOOL {
    let Menu: PMENU;
    let Status: NTSTATUS;
    let mut lstrCaption: _UNICODE_STRING = core::mem::zeroed();
    let mut Ret: BOOL = 0;

    UserEnterExclusive();

    'cleanup: {
        Menu = UserGetMenuObject(hMenu);
        if Menu.is_null() {
            break 'cleanup;
        }

        if !lpszCaption.is_null() && !(*lpszCaption).Buffer.is_null() {
            Status = ProbeAndCaptureUnicodeString(
                core::ptr::addr_of_mut!(lstrCaption),
                1u8,
                lpszCaption,
            );
            if !NT_SUCCESS(Status) {
                SetLastNtError(Status);
                break 'cleanup;
            }
        }

        if bInsert != 0 {
            Ret = UserInsertMenuItem(
                Menu,
                uItem,
                fByPosition,
                lpmii,
                core::ptr::addr_of_mut!(lstrCaption),
            ) as BOOL;
        } else {
            Ret = UserMenuItemInfo(
                Menu,
                uItem,
                fByPosition,
                lpmii as PROSMENUITEMINFO,
                1,
                core::ptr::addr_of_mut!(lstrCaption),
            );
        }
    }

    UserLeave();

    if !lstrCaption.Buffer.is_null() {
        ReleaseCapturedUnicodeString(core::ptr::addr_of_mut!(lstrCaption), 1u8);
    }

    Ret
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn NtUserTrackPopupMenuEx(
    hMenu: HMENU,
    fuFlags: UINT,
    x: INT,
    y: INT,
    hWnd: HWND,
    lptpm: LPTPMPARAMS,
) -> BOOL {
    let mut menu: PMENU;
    let mut pWnd: PWND;
    let mut tpm: TPMPARAMS = core::mem::zeroed();
    let mut Ret: BOOL = 0;
    let mut WndRef: USER_REFERENCE_ENTRY = core::mem::zeroed();
    let mut MenuRef: USER_REFERENCE_ENTRY = core::mem::zeroed();

    'Exit0: {
        if (fuFlags & !VALID_TPM_FLAGS) != 0 {
            EngSetLastError(ERROR_INVALID_FLAGS);
            break 'Exit0;
        }

        UserEnterExclusive();

        'Exit: {
            menu = UserGetMenuObject(hMenu);
            if menu.is_null() {
                EngSetLastError(ERROR_INVALID_MENU_HANDLE);
                break 'Exit;
            }

            pWnd = UserGetWindowObject(hWnd);
            if pWnd.is_null() {
                break 'Exit;
            }

            if !lptpm.is_null() {
                let __SehStatus = SafeProbeForRead(
                    lptpm as *mut core::ffi::c_void,
                    core::mem::size_of::<TPMPARAMS>() as ULONG,
                    core::mem::size_of::<u32>() as ULONG,
                );
                if !NT_SUCCESS(__SehStatus) {
                    break 'Exit;
                }
                // Probe is validate-only; the C had probe AND load in one _SEH2_TRY.
                if !crate::copy::capture_from_user(
                    core::ptr::addr_of_mut!(tpm) as *mut core::ffi::c_void,
                    lptpm as *const core::ffi::c_void,
                    core::mem::size_of::<TPMPARAMS>(),
                ) {
                    break 'Exit;
                }
            }

            UserRefObjectCo((pWnd as *mut core::ffi::c_void as PWND), &mut WndRef);
            UserRefObjectCo((menu as *mut core::ffi::c_void as PWND), &mut MenuRef);
            Ret = IntTrackPopupMenuEx(
                menu,
                fuFlags,
                x,
                y,
                pWnd,
                if !lptpm.is_null() {
                    &tpm
                } else {
                    core::ptr::null()
                },
            );
            UserDerefObjectCo((menu as *mut core::ffi::c_void as PWND));
            UserDerefObjectCo((pWnd as *mut core::ffi::c_void as PWND));
        }

        UserLeave();
    }

    Ret
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn NtUserTrackPopupMenuEx(
    hMenu: HMENU,
    fuFlags: UINT,
    x: INT,
    y: INT,
    hWnd: HWND,
    lptpm: LPTPMPARAMS,
) -> BOOL {
    let mut menu: PMENU;
    let mut pWnd: PWND;
    let mut tpm: TPMPARAMS = core::mem::zeroed();
    let mut Ret: BOOL = 0;
    let mut WndRef: USER_REFERENCE_ENTRY = core::mem::zeroed();
    let mut MenuRef: USER_REFERENCE_ENTRY = core::mem::zeroed();

    'Exit0: {
        if (fuFlags & !VALID_TPM_FLAGS) != 0 {
            EngSetLastError(ERROR_INVALID_FLAGS);
            break 'Exit0;
        }

        UserEnterExclusive();

        'Exit: {
            menu = UserGetMenuObject(hMenu);
            if menu.is_null() {
                EngSetLastError(ERROR_INVALID_MENU_HANDLE);
                break 'Exit;
            }

            pWnd = UserGetWindowObject(hWnd);
            if pWnd.is_null() {
                break 'Exit;
            }

            if !lptpm.is_null() {
                let __SehStatus = SafeProbeForRead(
                    lptpm as *mut core::ffi::c_void,
                    core::mem::size_of::<TPMPARAMS>() as _,
                    core::mem::size_of::<u32>() as ULONG,
                );
                if !NT_SUCCESS(__SehStatus) {
                    break 'Exit;
                }
                // Probe is validate-only; the C had probe AND load in one _SEH2_TRY.
                if !crate::copy::capture_from_user(
                    core::ptr::addr_of_mut!(tpm) as *mut core::ffi::c_void,
                    lptpm as *const core::ffi::c_void,
                    core::mem::size_of::<TPMPARAMS>(),
                ) {
                    break 'Exit;
                }
            }

            UserRefObjectCo(pWnd as PWND, &mut WndRef);
            UserRefObjectCo(menu as PWND, &mut MenuRef);
            Ret = IntTrackPopupMenuEx(
                menu,
                fuFlags,
                x,
                y,
                pWnd,
                if !lptpm.is_null() {
                    &tpm
                } else {
                    core::ptr::null()
                },
            );
            UserDerefObjectCo(menu as PWND);
            UserDerefObjectCo(pWnd as PWND);
        }

        UserLeave();
    }

    Ret
}


// ABI-corrected extern decls — i386 calling-convention must match the
// definition; a flat extern "C" decl of a stdcall/fastcall fn is UB -> ud2.
extern "system" {
    fn DrawTextW(hdc: HDC, str: LPCWSTR, count: INT, rect: LPRECT, flags: UINT) -> INT;
    fn DrawEdge(hdc: HDC, qrc: *mut RECT, edge: UINT, grfFlags: UINT) -> BOOL;
    fn FrameRect(hdc: HDC, lprc: *const RECT, hbr: HBRUSH) -> core::ffi::c_int;
    fn DrawFrameControl(dc: HDC, r: LPRECT, uType: UINT, uState: UINT) -> BOOL;
    fn FillRect(hDC: HDC, lprc: LPRECT, hbr: HBRUSH) -> i32;
}
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn SetLastNtError(Status: NTSTATUS);
    fn RECTL_vInflateRect(rect: *mut RECT, dx: core::ffi::c_int, dy: core::ffi::c_int);
    fn MsqSetStateWindow(pti: PTHREADINFO, Type: UINT, hWnd: HWND) -> HWND;
    fn co_UserHideCaret(Wnd: HWND) -> u8;
    fn co_UserSetCapture(hWnd: HWND) -> HWND;
    fn NC_IconForWindow(Wnd: PWND) -> PCURICON_OBJECT;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn SetLastNtError(Status: NTSTATUS);
    fn RECTL_vInflateRect(rect: *mut RECT, dx: core::ffi::c_int, dy: core::ffi::c_int);
    fn MsqSetStateWindow(pti: PTHREADINFO, Type: UINT, hWnd: HWND) -> HWND;
    fn co_UserHideCaret(Wnd: HWND) -> u8;
    fn co_UserSetCapture(hWnd: HWND) -> HWND;
    fn NC_IconForWindow(Wnd: PWND) -> PCURICON_OBJECT;
}
