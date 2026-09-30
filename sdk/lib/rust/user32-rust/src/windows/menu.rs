use win32k_sys::win32k::*;
// ntdll Rtl* string helpers live in crate::internal (NOT the kernel ntkrnl_sys crate).
use crate::internal::{
    RtlCreateUnicodeStringFromAsciiz, RtlFreeUnicodeString, RtlInitUnicodeString, ValidateHwnd,
    ZwCallbackReturn, LOWORD, MAKELONG,
};
// gdi32/kernel32/CRT externs + RtlZeroMemory live in crate::winbase.
use crate::winbase::{memcpy, RtlZeroMemory, WideCharToMultiByte};
// NtUserx* convenience wrappers + the DeleteMenu/RemoveMenu spec forwarders.
use crate::ntwrapper::{
    DeleteMenu, NtUserxCreateMenu, NtUserxCreatePopupMenu, NtUserxDrawMenuBar,
    NtUserxGetDesktopMapping, RemoveMenu,
};
use core::ffi::c_int;

// win32k-sys defines FALSE/TRUE as `u32`, but BOOL is `c_int` (i32). Shadow them
// with BOOL-typed constants (same pattern as defwnd.rs / class.rs).
const FALSE: BOOL = 0;
const TRUE: BOOL = 1;

// ── No-op debug macros (ERR/WARN/TRACE/FIXME) — the C macros compile to nothing
// under the Wine debug-channel guard. Consume args so side effects still type-check.

// ── user32-internal helpers exported from still-C user32 object files (link within DLL).
//   GET_WORD/GET_DWORD are C macros (`*(WORD*)ptr`) — reproduced as inline fns below,
//   not extern. ValidateHandle/ValidateHandleNoErr/SetLastError/DestroyMenu/strlenW are
//   real exports from other user32 TUs.
// ValidateHandle/ValidateHandleNoErr are `PVOID FASTCALL` (user32p.h) → @Name@N on i386.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn ValidateHandle(hObject: HANDLE, ObjType: UINT) -> *mut core::ffi::c_void;
    fn ValidateHandleNoErr(handle: HANDLE, type_: UINT) -> *mut core::ffi::c_void;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn ValidateHandle(hObject: HANDLE, ObjType: UINT) -> *mut core::ffi::c_void;
    fn ValidateHandleNoErr(handle: HANDLE, type_: UINT) -> *mut core::ffi::c_void;
}
// DestroyMenu is a user32.spec forwarder (→ NtUserDestroyMenu); strlenW is
// `#define strlenW wcslen` (wine/unicode.h) — neither has an internal linkable symbol.
#[inline]
unsafe fn DestroyMenu(hMenu: HMENU) -> BOOL {
    NtUserDestroyMenu(hMenu)
}
#[inline]
unsafe fn strlenW(s: LPCWSTR) -> UINT {
    crate::winbase::wcslen(s) as UINT
}
extern "system" {
    fn SetLastError(dwErrCode: u32);
    fn GdiValidateHandle(hobj: HGDIOBJ) -> BOOL; // gdi32
    fn GetSystemMetrics(nIndex: core::ffi::c_int) -> core::ffi::c_int;
    fn OutputDebugStringA(lpOutputString: LPCSTR);
    fn lstrcpynW(lpString1: LPWSTR, lpString2: LPCWSTR, iMaxLength: core::ffi::c_int) -> LPWSTR;
    fn FindResourceW(hModule: HINSTANCE, lpName: LPCWSTR, lpType: LPCWSTR) -> HANDLE;
    fn LoadResource(hModule: HINSTANCE, hResInfo: HANDLE) -> *mut core::ffi::c_void;
    fn FindResourceA(hModule: HINSTANCE, lpName: LPCSTR, lpType: LPCSTR) -> *mut core::ffi::c_void;
    // user32 own-exports from windows/defwnd.rs + windows/class.rs (link within DLL).
    fn DefWindowProcW(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn DefWindowProcA(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn GetWindowLongPtrW(hWnd: HWND, nIndex: core::ffi::c_int) -> LONG_PTR;
}
extern "C" {
    fn strlen(s: LPCSTR) -> usize; // CRT
}

// C `#define GET_WORD(ptr) (*(WORD*)(ptr))` / `GET_DWORD(ptr) (*(DWORD*)(ptr))`.
#[inline]
unsafe fn GET_WORD(ptr: *const core::ffi::c_void) -> WORD {
    core::ptr::read_unaligned(ptr as *const WORD)
}
#[inline]
unsafe fn GET_DWORD(ptr: *const core::ffi::c_void) -> DWORD {
    core::ptr::read_unaligned(ptr as *const DWORD)
}

// winuser.h `MAKEINTRESOURCEA(i)` == `(LPSTR)(ULONG_PTR)(WORD)(i)`.
#[inline]
fn MAKEINTRESOURCEA(i: core::ffi::c_int) -> LPCSTR {
    (i as u16) as usize as LPCSTR
}

// undocuser.h `WC_MENU == MAKEINTATOM(0x8000)`.
const WC_MENU: LPCWSTR = 0x8000usize as LPCWSTR;
// winuser.h `RT_MENU == MAKEINTRESOURCE(4)`.
const RT_MENU: LPCWSTR = 4usize as LPCWSTR;
// winuser.h `IDC_ARROW == MAKEINTRESOURCEW(32512)`.
const IDC_ARROW: LPCWSTR = 32512usize as LPCWSTR;
// winnls.h `CP_ACP == 0`.
const CP_ACP: UINT = 0;
// ntstatus.h `STATUS_SUCCESS == 0`.
const STATUS_SUCCESS: NTSTATUS = 0;

// basetsd.h `UlongToHandle(ul) == (HANDLE)(ULONG_PTR)(ul)`.
#[inline]
fn UlongToHandle(ul: ULONG) -> HANDLE {
    ul as usize as HANDLE
}

// menu.c file-local masks/macros (header section, lines 22-43).
const MENU_TYPE_MASK: UINT = MF_STRING | MF_BITMAP | MF_OWNERDRAW | MF_SEPARATOR;
#[inline]
fn MENU_ITEM_TYPE(flags: UINT) -> UINT {
    flags & MENU_TYPE_MASK
}
const MNS_STYLE_MASK: UINT =
    MNS_NOCHECK | MNS_MODELESS | MNS_DRAGDROP | MNS_AUTODISMISS | MNS_NOTIFYBYPOS | MNS_CHECKORBMP;
const MENUITEMINFO_TYPE_MASK: UINT = MFT_STRING
    | MFT_BITMAP
    | MFT_OWNERDRAW
    | MFT_SEPARATOR
    | MFT_MENUBARBREAK
    | MFT_MENUBREAK
    | MFT_RADIOCHECK
    | MFT_RIGHTORDER
    | MFT_RIGHTJUSTIFY;
const TYPE_MASK: UINT = MENUITEMINFO_TYPE_MASK | MF_POPUP | MF_SYSMENU;
const STATE_MASK: UINT = !TYPE_MASK;
const MENUITEMINFO_STATE_MASK: UINT = STATE_MASK & !(MF_BYPOSITION | MF_MOUSESELECT);
const MII_STATE_MASK: UINT = MFS_GRAYED | MFS_CHECKED | MFS_HILITE | MFS_DEFAULT;
#[inline]
fn IS_STRING_ITEM(flags: UINT) -> bool {
    MF_STRING == MENU_ITEM_TYPE(flags)
}
// `#define IS_MAGIC_BITMAP(id) ((id) && ((INT_PTR)(id) < 12) && ((INT_PTR)(id) >= -1))`.
// Used in C as a boolean; here returns BOOL to match the `== 0` test site.
#[inline]
unsafe fn IS_MAGIC_BITMAP(id: HBITMAP) -> BOOL {
    let v = id as isize;
    (!id.is_null() && v < 12 && v >= -1) as BOOL
}

// ── ntuser.h / user_x.h inline helpers (C macros, no exported symbol) ──
// `GetWin32ClientInfo()` == `(PCLIENTINFO)(NtCurrentTeb()->Win32ClientInfo)`.
#[inline]
unsafe fn GetWin32ClientInfo() -> PCLIENTINFO {
    core::ptr::addr_of_mut!((*crate::internal::NtCurrentTeb()).Win32ClientInfo) as *mut _
        as PCLIENTINFO
}
// `GetW32ThreadInfo()` == `(PTHREADINFO)(NtCurrentTeb()->Win32ThreadInfo)`.
#[inline]
unsafe fn GetW32ThreadInfo() -> PTHREADINFO {
    (*crate::internal::NtCurrentTeb()).Win32ThreadInfo as PTHREADINFO
}
// user_x.h `DesktopPtrToUser` — server→client desktop-heap remapping (fast path
// subtracts ulClientDelta, slow path calls win32k via NtUserxGetDesktopMapping).
#[inline]
unsafe fn DesktopPtrToUser(ptr: PVOID) -> PVOID {
    // WoW64: DESKTOPINFO is 64-bit-laid-out for a 32-bit guest, so the
    // base/limit range check must be done through the normalizing view.
    // Single definition lives in crate::wow64.
    crate::wow64::desktop_ptr_to_user(ptr)
}
// ntuser.h `UserHMGetHandle(obj) == ((obj)->head.h)`. The handle lives at offset 0
// of the (PROC|THR)DESKHEAD: HEAD.h.
// Read through the typed head, never an index into bindgen's __bindgen_padding_0: its
// ELEMENT type is the generator's choice. The 2026-09-27 regeneration turned [u64; 3]
// into [u8; 24], `[2]` became one byte of the handle, and every posted message dispatched
// through IntCallMessageProc was dropped as ERROR_MESSAGE_SYNC_ONLY (no app closed on
// WM_CLOSE) with every build still green.
#[inline]
unsafe fn UserHMGetHandle(pMenu: PMENU) -> HWND {
    (*(pMenu as *const _HEAD)).h as HWND
}

// ---- file-scope preamble (translated) ----
pub const MM_SETMENUHANDLE: u32 = WM_USER + 0;

pub const MM_GETMENUHANDLE: u32 = WM_USER + 1;

#[repr(C)]
pub struct builtin_class_descr {
    pub name: LPCWSTR,
    pub style: UINT,
    pub procA: WNDPROC,
    pub procW: WNDPROC,
    pub extra: INT,
    pub cursor: LPCWSTR,
    pub brush: HBRUSH,
}

unsafe impl Sync for builtin_class_descr {}

// NOT #[no_mangle]: regcontrol.c's `&POPUPMENU_builtin_class` reference is inside
// `#if 0`, so this is not a live cross-TU symbol (per integration spec §3).
pub static POPUPMENU_builtin_class: builtin_class_descr = builtin_class_descr {
    name: WC_MENU,
    style: CS_SAVEBITS | CS_DBLCLKS,
    procA: None,
    // PopupMenuWndProcW is `extern "system"`; WNDPROC's inner fn is `extern "C"`
    // (amd64/arm64) / `extern "stdcall"` (i386) — same ABI, distinct Rust type.
    // Transmute straight to WNDPROC (same pattern as button.rs / dialog.rs).
    procW: unsafe {
        core::mem::transmute::<
            unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT,
            WNDPROC,
        >(PopupMenuWndProcW)
    },
    extra: core::mem::size_of::<*mut MENUINFO>() as INT,
    cursor: IDC_ARROW,
    brush: (COLOR_MENU + 1) as HBRUSH,
};

unsafe fn MENU_GetMenu(hMenu: HMENU) -> PMENU {
    ValidateHandleNoErr(hMenu as HANDLE, TYPE_MENU) as PMENU
}

unsafe fn MENU_FindItem(hmenu: *mut HMENU, nPos: *mut UINT, wFlags: UINT) -> *mut ITEM {
    let menu: PMENU;
    let mut fallback: *mut ITEM = core::ptr::null_mut();
    let mut fallback_pos: UINT = 0;
    let mut i: UINT;

    if (*hmenu == 0xffff as HMENU) || {
        menu = MENU_GetMenu(*hmenu);
        menu.is_null()
    } {
        return core::ptr::null_mut();
    }

    if (wFlags & MF_BYPOSITION) != 0 {
        if *nPos >= (*menu).cItems {
            return core::ptr::null_mut();
        }
        let mut pItem: PITEM = if !(*menu).rgItems.is_null() {
            {
                crate::wow64::wow64_unsupported("MENU/ITEM");
                DesktopPtrToUser((*menu).rgItems as _) as PITEM
            }
        } else {
            core::ptr::null_mut()
        };
        if !pItem.is_null() {
            pItem = pItem.add(*nPos as usize);
        }
        return pItem;
    } else {
        let mut item: PITEM = if !(*menu).rgItems.is_null() {
            DesktopPtrToUser((*menu).rgItems as _) as PITEM
        } else {
            core::ptr::null_mut()
        };
        i = 0;
        while !item.is_null() && i < (*menu).cItems {
            if !(*item).spSubMenu.is_null() {
                let pSubMenu: PMENU = DesktopPtrToUser((*item).spSubMenu as _) as PMENU;
                let mut hsubmenu: HMENU = UserHMGetHandle(pSubMenu as _) as HMENU;
                let subitem = MENU_FindItem(core::ptr::addr_of_mut!(hsubmenu), nPos, wFlags);
                if !subitem.is_null() {
                    *hmenu = hsubmenu;
                    return subitem;
                } else if (*item).wID == *nPos {
                    fallback_pos = i;
                    fallback = item;
                }
            } else if (*item).wID == *nPos {
                *nPos = i;
                return item;
            }
            i += 1;
            item = item.add(1);
        }
    }

    if !fallback.is_null() {
        *nPos = fallback_pos;
    }

    fallback
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntGetMenuDefaultItem(
    Menu: PMENU,
    fByPos: i32,
    gmdiFlags: UINT,
    gismc: *mut DWORD,
) -> UINT {
    IntGetMenuDefaultItemImpl(Menu, fByPos, gmdiFlags, gismc)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntGetMenuDefaultItem(
    Menu: PMENU,
    fByPos: i32,
    gmdiFlags: UINT,
    gismc: *mut DWORD,
) -> UINT {
    IntGetMenuDefaultItemImpl(Menu, fByPos, gmdiFlags, gismc)
}

unsafe fn IntGetMenuDefaultItemImpl(
    Menu: PMENU,
    fByPos: i32,
    gmdiFlags: UINT,
    gismc: *mut DWORD,
) -> UINT {
    let mut i: UINT = 0;
    let mut Item: PITEM = if !(*Menu).rgItems.is_null() {
        DesktopPtrToUser((*Menu).rgItems as *mut _) as PITEM
    } else {
        core::ptr::null_mut()
    };

    if Item.is_null() {
        return u32::MAX;
    }

    while ((*Item).fState & MFS_DEFAULT) == 0 {
        i += 1;
        Item = Item.add(1);
        if i >= (*Menu).cItems {
            return u32::MAX;
        }
    }

    if (GMDI_USEDISABLED & gmdiFlags) == 0 && ((*Item).fState & MFS_DISABLED) != 0 {
        return u32::MAX;
    }

    if (gmdiFlags & GMDI_GOINTOPOPUPS) != 0 && !(*Item).spSubMenu.is_null() {
        *gismc += 1;
        let ret = IntGetMenuDefaultItemImpl(
            DesktopPtrToUser((*Item).spSubMenu as *mut _) as PMENU,
            fByPos,
            gmdiFlags,
            gismc,
        );
        *gismc -= 1;
        if ret != u32::MAX {
            return ret;
        }
    }

    if fByPos != 0 {
        i
    } else {
        (*Item).wID
    }
}

unsafe fn GetMenuItemInfo_common(
    mut hmenu: HMENU,
    mut item: UINT,
    bypos: BOOL,
    lpmii: LPMENUITEMINFOW,
    unicode: BOOL,
) -> BOOL {
    let pItem: *mut ITEM = MENU_FindItem(
        &mut hmenu,
        &mut item,
        if bypos != 0 { MF_BYPOSITION } else { 0 },
    );

    if pItem.is_null() {
        SetLastError(ERROR_MENU_ITEM_NOT_FOUND);
        return 0;
    }

    if (*lpmii).fMask & MIIM_TYPE != 0 {
        if (*lpmii).fMask & (MIIM_STRING | MIIM_FTYPE | MIIM_BITMAP) != 0 {
            ERR!("invalid combination of fMask bits used\n");
            SetLastError(ERROR_INVALID_PARAMETER);
            return 0;
        }
        (*lpmii).fType = (*pItem).fType & MENUITEMINFO_TYPE_MASK;
        if !(*pItem).hbmp.is_null() && IS_MAGIC_BITMAP((*pItem).hbmp) == 0 {
            (*lpmii).fType |= MFT_BITMAP;
        }
        (*lpmii).hbmpItem = (*pItem).hbmp;
        if (*lpmii).fType & MFT_BITMAP != 0 {
            (*lpmii).dwTypeData = (*pItem).hbmp as LPWSTR;
            (*lpmii).cch = 0;
        } else if (*lpmii).fType & (MFT_OWNERDRAW | MFT_SEPARATOR) != 0 {
            (*lpmii).dwTypeData = core::ptr::null_mut();
            (*lpmii).cch = 0;
        }
    }

    if (*lpmii).fMask & (MIIM_TYPE | MIIM_STRING) != 0 {
        if (*pItem).Xlpstr.is_null() {
            if !(*lpmii).dwTypeData.is_null()
                && (*lpmii).cch != 0
                && GdiValidateHandle((*lpmii).dwTypeData as HGDIOBJ) == 0
            {
                if unicode != 0 {
                    *((*lpmii).dwTypeData as *mut u16) = 0;
                } else {
                    *((*lpmii).dwTypeData as *mut i8) = 0;
                }
            }
            (*lpmii).cch = 0;
        } else {
            let len: i32;
            let text: LPWSTR = DesktopPtrToUser((*pItem).Xlpstr as PVOID) as LPWSTR;
            if unicode != 0 {
                len = strlenW(text) as i32;
                if !(*lpmii).dwTypeData.is_null() && (*lpmii).cch != 0 {
                    lstrcpynW((*lpmii).dwTypeData, text, (*lpmii).cch as core::ffi::c_int);
                }
            } else {
                len = WideCharToMultiByte(
                    CP_ACP,
                    0,
                    text,
                    -1,
                    core::ptr::null_mut(),
                    0,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                ) - 1;
                if !(*lpmii).dwTypeData.is_null() && (*lpmii).cch != 0 {
                    if WideCharToMultiByte(
                        CP_ACP,
                        0,
                        text,
                        -1,
                        (*lpmii).dwTypeData as LPSTR,
                        (*lpmii).cch as c_int,
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                    ) == 0
                    {
                        *(((*lpmii).dwTypeData as LPSTR).offset(((*lpmii).cch - 1) as isize)) = 0;
                    }
                }
            }
            if !(*lpmii).dwTypeData.is_null() && (*lpmii).cch != 0 {
                if (*lpmii).cch <= (len + 1) as UINT {
                    (*lpmii).cch -= 1;
                } else {
                    (*lpmii).cch = len as UINT;
                }
            } else {
                (*lpmii).cch = len as UINT;
            }
        }
    }

    if (*lpmii).fMask & MIIM_FTYPE != 0 {
        (*lpmii).fType = (*pItem).fType & MENUITEMINFO_TYPE_MASK;
    }

    if (*lpmii).fMask & MIIM_BITMAP != 0 {
        (*lpmii).hbmpItem = (*pItem).hbmp;
    }

    if (*lpmii).fMask & MIIM_STATE != 0 {
        (*lpmii).fState = (*pItem).fState & MENUITEMINFO_STATE_MASK;
    }

    if (*lpmii).fMask & MIIM_ID != 0 {
        (*lpmii).wID = (*pItem).wID;
    }

    if (*lpmii).fMask & MIIM_SUBMENU != 0 && !(*pItem).spSubMenu.is_null() {
        let pSubMenu: PMENU = DesktopPtrToUser((*pItem).spSubMenu as PVOID) as PMENU;
        let hSubMenu: HMENU = UserHMGetHandle(pSubMenu) as HMENU;
        (*lpmii).hSubMenu = hSubMenu;
    } else {
        (*lpmii).hSubMenu = core::ptr::null_mut();
    }

    if (*lpmii).fMask & MIIM_CHECKMARKS != 0 {
        (*lpmii).hbmpChecked = (*pItem).hbmpChecked as HBITMAP;
        (*lpmii).hbmpUnchecked = (*pItem).hbmpUnchecked as HBITMAP;
    }
    if (*lpmii).fMask & MIIM_DATA != 0 {
        (*lpmii).dwItemData = (*pItem).dwItemData;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn PopupMenuWndProcW(
    Wnd: HWND,
    Message: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let mut lResult: LRESULT = 0;
    let pWnd: PWND;

    pWnd = ValidateHwnd(Wnd);
    if !pWnd.is_null() {
        if (*pWnd).fnid == 0 {
            if Message != WM_NCCREATE {
                return DefWindowProcW(Wnd, Message, wParam, lParam);
            }
        } else {
            if (*pWnd).fnid != FNID_MENU as DWORD {
                ERR!("Wrong window class for Menu!\n");
                return 0;
            }
        }
    }

    const MM_SETMENUHANDLE: UINT = WM_USER + 0;
    const MM_GETMENUHANDLE: UINT = WM_USER + 1;

    match Message {
        WM_DESTROY | WM_NCDESTROY | WM_NCCREATE | WM_CREATE | MM_SETMENUHANDLE
        | MM_GETMENUHANDLE | MN_SETHMENU | MN_GETHMENU | WM_PAINT | WM_PRINTCLIENT => {
            NtUserMessageCall(
                Wnd,
                Message,
                wParam,
                lParam,
                core::ptr::addr_of_mut!(lResult) as ULONG_PTR,
                FNID_MENU,
                0,
            );
            return lResult;
        }
        WM_MOUSEACTIVATE => {
            return MA_NOACTIVATE as LRESULT;
        }
        WM_ERASEBKGND => {
            return 1;
        }
        WM_SHOWWINDOW => {
            if wParam != 0 {
                if GetWindowLongPtrW(Wnd, 0) == 0 {
                    OutputDebugStringA(b"no menu to display\n\0".as_ptr() as *const i8);
                }
            }
        }
        _ => {
            return DefWindowProcW(Wnd, Message, wParam, lParam);
        }
    }

    0
}

#[no_mangle]
pub unsafe extern "system" fn PopupMenuWndProcA(
    Wnd: HWND,
    Message: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let pWnd: PWND = ValidateHwnd(Wnd);

    if !pWnd.is_null() && (*pWnd).fnid == 0 && Message != WM_NCCREATE {
        return DefWindowProcA(Wnd, Message, wParam, lParam);
    }

    TRACE!(
        b"YES! hwnd=%x msg=0x%04x wp=0x%04lx lp=0x%08lx\n\0".as_ptr() as *const i8,
        Wnd,
        Message,
        wParam,
        lParam,
    );

    match Message {
        WM_NCCREATE
        | WM_CREATE
        | WM_MOUSEACTIVATE
        | WM_PAINT
        | WM_PRINTCLIENT
        | WM_ERASEBKGND
        | WM_DESTROY
        | WM_NCDESTROY
        | WM_SHOWWINDOW
        | MM_SETMENUHANDLE
        | MM_GETMENUHANDLE
        | MN_SETHMENU
        | MN_GETHMENU => PopupMenuWndProcW(Wnd, Message, wParam, lParam),

        _ => DefWindowProcA(Wnd, Message, wParam, lParam),
    }
}

unsafe fn MENU_ParseResource(mut res: LPCSTR, hMenu: HMENU) -> LPCSTR {
    let mut flags: WORD;
    let mut id: WORD = 0;
    let mut hSubMenu: HMENU;
    let str: LPCWSTR;
    let mut end: BOOL;

    loop {
        flags = GET_WORD(res as *const core::ffi::c_void);

        end = (flags as u32 & MF_END) as BOOL;
        if end != 0 {
            flags ^= MF_END as WORD;
        }

        res = res.add(core::mem::size_of::<WORD>());
        if (flags as u32 & MF_POPUP) == 0 {
            id = GET_WORD(res as *const core::ffi::c_void);
            res = res.add(core::mem::size_of::<WORD>());
        }
        let str = res as LPCWSTR;
        res = res.add((strlenW(str) + 1) as usize * core::mem::size_of::<u16>());

        if (flags as u32 & MF_POPUP) != 0 {
            hSubMenu = CreatePopupMenu();
            if hSubMenu.is_null() {
                return core::ptr::null_mut();
            }
            res = MENU_ParseResource(res, hSubMenu);
            if res.is_null() {
                return core::ptr::null_mut();
            }
            AppendMenuW(hMenu, flags as UINT, hSubMenu as UINT_PTR, str);
        } else {
            AppendMenuW(
                hMenu,
                flags as UINT,
                id as UINT_PTR,
                if *str != 0 { str } else { core::ptr::null_mut() },
            );
        }

        if end != 0 {
            break;
        }
    }
    res
}

unsafe fn MENUEX_ParseResource(mut res: LPCSTR, hMenu: HMENU) -> LPCSTR {
    let mut resinfo: WORD;
    loop {
        let mut mii: MENUITEMINFOW = core::mem::zeroed();

        mii.cbSize = core::mem::size_of::<MENUITEMINFOW>() as UINT;
        mii.fMask = MIIM_STATE | MIIM_ID | MIIM_TYPE;
        mii.fType = GET_DWORD(res as *const core::ffi::c_void);
        res = res.add(core::mem::size_of::<DWORD>());
        mii.fState = GET_DWORD(res as *const core::ffi::c_void);
        res = res.add(core::mem::size_of::<DWORD>());
        mii.wID = GET_DWORD(res as *const core::ffi::c_void);
        res = res.add(core::mem::size_of::<DWORD>());
        resinfo = GET_WORD(res as *const core::ffi::c_void);
        res = res.add(core::mem::size_of::<WORD>());
        /* Align the text on a word boundary.  */
        res = res.add(((!((res as UINT_PTR).wrapping_sub(1))) & 1) as usize);
        mii.dwTypeData = res as LPWSTR;
        mii.cch = strlenW(mii.dwTypeData);
        res = res.add(((1 + strlenW(mii.dwTypeData)) as usize) * core::mem::size_of::<WCHAR>());
        /* Align the following fields on a dword boundary.  */
        res = res.add(((!((res as UINT_PTR).wrapping_sub(1))) & 3) as usize);

        if resinfo & 1 != 0 {
            res = res.add(core::mem::size_of::<DWORD>());
            mii.hSubMenu = CreatePopupMenu();
            if mii.hSubMenu.is_null() {
                return core::ptr::null_mut();
            }

            res = MENUEX_ParseResource(res, mii.hSubMenu);
            if res.is_null() {
                DestroyMenu(mii.hSubMenu);
                return core::ptr::null_mut();
            }
            mii.fMask |= MIIM_SUBMENU;
            mii.fType |= MF_POPUP;
        } else if *mii.dwTypeData == 0 && (mii.fType & MF_SEPARATOR) == 0 {
            mii.fType |= MF_SEPARATOR;
        }
        InsertMenuItemW(hMenu, -1i32 as UINT, MF_BYPOSITION as BOOL, &mii);

        if resinfo & (MF_END as WORD) != 0 {
            break;
        }
    }
    res
}

unsafe fn MENU_mnu2mnuii(mut flags: UINT, id: UINT_PTR, mut str: LPCWSTR, pmii: LPMENUITEMINFOW, Unicode: BOOL) {
    RtlZeroMemory(pmii as *mut _, core::mem::size_of::<MENUITEMINFOW>());
    (*pmii).cbSize = core::mem::size_of::<MENUITEMINFOW>() as u32;
    (*pmii).fMask = MIIM_STATE | MIIM_ID | MIIM_FTYPE;
    if IS_STRING_ITEM(flags) {
        (*pmii).fMask |= MIIM_STRING | MIIM_BITMAP;
        if str.is_null() {
            flags |= MF_SEPARATOR;
        } else {
            if Unicode != 0 {
                if *str == b'\x08' as u16 {
                    flags |= MF_HELP;
                    str = str.add(1);
                }
            } else {
                let mut NewItemA: LPCSTR = str as LPCSTR;
                if *NewItemA == b'\x08' as i8 {
                    flags |= MF_HELP;
                    NewItemA = NewItemA.add(1);
                    str = NewItemA as LPCWSTR;
                }
                TRACE!(b"A cch %d\n\0".as_ptr() as *const i8, strlen(NewItemA));
            }
        }
        (*pmii).dwTypeData = str as LPWSTR;
    } else if flags & MFT_BITMAP != 0 {
        (*pmii).fMask |= MIIM_BITMAP | MIIM_STRING;
        (*pmii).hbmpItem = str as HBITMAP;
    }
    if flags & MF_OWNERDRAW != 0 {
        (*pmii).fMask |= MIIM_DATA;
        (*pmii).dwItemData = str as ULONG_PTR;
    }
    if flags & MF_POPUP != 0 && !MENU_GetMenu(id as HMENU).is_null() {
        (*pmii).fMask |= MIIM_SUBMENU;
        (*pmii).hSubMenu = id as HMENU;
    }
    if flags & MF_SEPARATOR != 0 {
        flags |= MF_GRAYED | MF_DISABLED;
    }
    (*pmii).fState = flags & MENUITEMINFO_STATE_MASK & !MFS_DEFAULT;
    (*pmii).fType = flags & MENUITEMINFO_TYPE_MASK;
    (*pmii).wID = id as UINT;
}

unsafe fn MENU_NormalizeMenuItemInfoStruct(pmii_in: *const MENUITEMINFOW, pmii_out: *mut MENUITEMINFOW) -> BOOL {
    if pmii_in.is_null()
        || ((*pmii_in).cbSize != core::mem::size_of::<MENUITEMINFOW>() as u32
            && (*pmii_in).cbSize
                != (core::mem::size_of::<MENUITEMINFOW>()
                    - core::mem::size_of_val(&(*pmii_in).hbmpItem)) as u32)
    {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }

    core::ptr::copy_nonoverlapping(
        pmii_in as *const u8,
        pmii_out as *mut u8,
        (*pmii_in).cbSize as usize,
    );

    if (*pmii_in).cbSize != core::mem::size_of::<MENUITEMINFOW>() as u32 {
        (*pmii_out).cbSize = core::mem::size_of::<MENUITEMINFOW>() as u32;
        (*pmii_out).hbmpItem = core::ptr::null_mut();
    }

    if ((*pmii_out).fMask & MIIM_TYPE != 0
        && (*pmii_out).fMask & (MIIM_STRING | MIIM_FTYPE | MIIM_BITMAP) != 0)
        || ((*pmii_out).fMask & MIIM_FTYPE != 0 && (*pmii_out).fType & MFT_BITMAP != 0)
    {
        ERR!("invalid combination of fMask bits used\n");
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }

    if (*pmii_out).fMask & MIIM_TYPE != 0 {
        (*pmii_out).fMask |= MIIM_FTYPE;
        if IS_STRING_ITEM((*pmii_out).fType) {
            (*pmii_out).fMask |= MIIM_STRING;
        } else if (*pmii_out).fType & MFT_BITMAP != 0 {
            (*pmii_out).fMask |= MIIM_BITMAP;
            (*pmii_out).hbmpItem =
                UlongToHandle(LOWORD((*pmii_out).dwTypeData as usize as DWORD) as ULONG) as HBITMAP;
        }
    }

    if (*pmii_out).fMask & MIIM_FTYPE != 0 {
        (*pmii_out).fType &= !MENUITEMINFO_TYPE_MASK;
        (*pmii_out).fType |= (*pmii_in).fType & MENUITEMINFO_TYPE_MASK;
    }

    if (*pmii_out).fMask & MIIM_STATE != 0 {
        (*pmii_out).fState = (*pmii_in).fState & MENUITEMINFO_STATE_MASK;
    }

    if (*pmii_out).fMask & MIIM_SUBMENU != 0 {
        if !(*pmii_out).hSubMenu.is_null() && IsMenu((*pmii_out).hSubMenu) == 0 {
            return FALSE;
        }
    }

    TRUE
}

// MenuInit/MenuCleanup are plain-cdecl C (menu.c), called by the surviving dllmain.c
// as `_MenuInit`/`_MenuCleanup` (no @N) → must be extern "C", not stdcall.
#[no_mangle]
pub unsafe extern "C" fn MenuInit() -> BOOL {
    1
}

#[no_mangle]
pub unsafe extern "C" fn MenuCleanup() {
}
#[no_mangle]
pub unsafe extern "system" fn User32LoadSysMenuTemplateForKernel(Arguments: PVOID, ArgumentLength: ULONG) -> NTSTATUS {
    let mut Result: LRESULT = 0;

    ZwCallbackReturn(
        core::ptr::addr_of_mut!(Result) as PVOID,
        core::mem::size_of::<LRESULT>() as ULONG,
        STATUS_SUCCESS,
    )
}
#[no_mangle]
pub unsafe extern "system" fn User32CallLoadMenuFromKernel(Arguments: *mut core::ffi::c_void, _ArgumentLength: ULONG) -> NTSTATUS {
    let Common: PLOADMENU_CALLBACK_ARGUMENTS = Arguments as PLOADMENU_CALLBACK_ARGUMENTS;

    let lpMenuName: LPCWSTR = if !(*Common).InterSource.is_null() {
        (*Common).InterSource
    } else {
        core::ptr::addr_of_mut!((*Common).MenuName) as LPCWSTR
    };

    let mut Result: LRESULT = LoadMenuW((*Common).hModule, lpMenuName) as LRESULT;

    ZwCallbackReturn(
        core::ptr::addr_of_mut!(Result) as *mut core::ffi::c_void,
        core::mem::size_of::<LRESULT>() as ULONG,
        STATUS_SUCCESS,
    )
}

#[no_mangle]
pub unsafe extern "system" fn AppendMenuA(
    hMenu: HMENU,
    uFlags: UINT,
    uIDNewItem: UINT_PTR,
    lpNewItem: LPCSTR,
) -> BOOL {
    let mut mii: MENUITEMINFOW = core::mem::zeroed();
    let mut UnicodeString: _UNICODE_STRING = core::mem::zeroed();
    let res: BOOL;

    RtlInitUnicodeString(&mut UnicodeString, core::ptr::null_mut());

    MENU_mnu2mnuii(uFlags, uIDNewItem, lpNewItem as LPCWSTR, &mut mii, 0);

    if !lpNewItem.is_null()
        && (mii.fMask & MIIM_STRING) != 0
        && mii.hbmpItem.is_null()
        && !mii.dwTypeData.is_null()
    {
        if RtlCreateUnicodeStringFromAsciiz(&mut UnicodeString, mii.dwTypeData as LPSTR) == 0 {
            SetLastError(ERROR_NOT_ENOUGH_MEMORY);
            return 0;
        }
        mii.dwTypeData = UnicodeString.Buffer;
        mii.cch = (UnicodeString.Length as u32) / core::mem::size_of::<u16>() as u32;
    } else {
        TRACE!(b"AMA Handle bitmaps\n\0".as_ptr() as *const i8);
    }

    res = NtUserThunkedMenuItemInfo(hMenu, -1i32 as UINT, 1, 1, &mut mii, &mut UnicodeString);
    if !UnicodeString.Buffer.is_null() {
        RtlFreeUnicodeString(&mut UnicodeString);
    }
    res
}

#[no_mangle]
pub unsafe extern "system" fn AppendMenuW(
    hMenu: HMENU,
    uFlags: UINT,
    uIDNewItem: UINT_PTR,
    lpNewItem: LPCWSTR,
) -> BOOL {
    let mut mii: MENUITEMINFOW = core::mem::zeroed();
    let mut MenuText: _UNICODE_STRING = core::mem::zeroed();
    let res: BOOL;

    RtlInitUnicodeString(&mut MenuText, core::ptr::null_mut());

    MENU_mnu2mnuii(uFlags, uIDNewItem, lpNewItem, &mut mii, 1);

    if !lpNewItem.is_null()
        && (mii.fMask & MIIM_STRING) != 0
        && mii.hbmpItem.is_null()
        && mii.dwTypeData != core::ptr::null_mut()
    {
        RtlInitUnicodeString(&mut MenuText, mii.dwTypeData as PWSTR);
        mii.dwTypeData = MenuText.Buffer;
        mii.cch = (MenuText.Length as usize / core::mem::size_of::<u16>()) as UINT;
    }

    res = NtUserThunkedMenuItemInfo(hMenu, -1i32 as UINT, 1, 1, &mut mii, &mut MenuText);
    res
}

#[no_mangle]
pub unsafe extern "system" fn CheckMenuItem(
    hmenu: HMENU,
    uIDCheckItem: UINT,
    uCheck: UINT,
) -> DWORD {
    let item: PITEM;
    let Ret: DWORD;
    let mut uID: UINT = uIDCheckItem;
    let mut hmenu = hmenu;

    if ValidateHandle(hmenu as HANDLE, TYPE_MENU).is_null() {
        return -1i32 as DWORD;
    }

    item = MENU_FindItem(&mut hmenu, &mut uID, uCheck);
    if item.is_null() {
        return -1i32 as DWORD;
    }

    Ret = (*item).fState & MFS_CHECKED;
    if Ret == (uCheck & MFS_CHECKED) {
        return Ret;
    }

    NtUserCheckMenuItem(hmenu, uIDCheckItem, uCheck)
}

#[no_mangle]
pub unsafe extern "system" fn CheckMenuRadioItem(
    hMenu: HMENU,
    first: UINT,
    last: UINT,
    check: UINT,
    bypos: UINT,
) -> BOOL {
    let mut done: BOOL = 0;
    let mut i: UINT;
    let mut mi_first: PITEM = core::ptr::null_mut();
    let mut mi_check: PITEM;
    let mut m_first: HMENU = core::ptr::null_mut();
    let mut m_check: HMENU;
    let mut mii: MENUITEMINFOW = core::mem::zeroed();
    mii.cbSize = core::mem::size_of::<MENUITEMINFOW>() as UINT;

    i = first;
    while i <= last {
        let mut pos: UINT = i;

        if mi_first.is_null() {
            m_first = hMenu;
            mi_first = MENU_FindItem(
                core::ptr::addr_of_mut!(m_first),
                core::ptr::addr_of_mut!(pos),
                bypos,
            );
            if mi_first.is_null() {
                i += 1;
                continue;
            }
            mi_check = mi_first;
            m_check = m_first;
        } else {
            m_check = hMenu;
            mi_check = MENU_FindItem(
                core::ptr::addr_of_mut!(m_check),
                core::ptr::addr_of_mut!(pos),
                bypos,
            );
            if mi_check.is_null() {
                i += 1;
                continue;
            }
        }

        if m_first != m_check {
            i += 1;
            continue;
        }
        if (*mi_check).fType == MFT_SEPARATOR {
            i += 1;
            continue;
        }

        if i == check {
            if ((*mi_check).fType & MFT_RADIOCHECK) == 0
                || ((*mi_check).fState & MFS_CHECKED) == 0
            {
                mii.fMask = MIIM_FTYPE | MIIM_STATE;
                mii.fType = ((*mi_check).fType & MENUITEMINFO_TYPE_MASK) | MFT_RADIOCHECK;
                mii.fState = ((*mi_check).fState & MII_STATE_MASK) | MFS_CHECKED;
                NtUserThunkedMenuItemInfo(
                    m_check,
                    i,
                    bypos as BOOL,
                    0,
                    core::ptr::addr_of_mut!(mii),
                    core::ptr::null_mut(),
                );
            }
            done = 1;
        } else {
            if ((*mi_check).fState & MFS_CHECKED) != 0 {
                mii.fMask = MIIM_STATE;
                mii.fState = ((*mi_check).fState & MII_STATE_MASK) & !MFS_CHECKED;
                NtUserThunkedMenuItemInfo(
                    m_check,
                    i,
                    bypos as BOOL,
                    0,
                    core::ptr::addr_of_mut!(mii),
                    core::ptr::null_mut(),
                );
            }
        }

        i += 1;
    }
    done
}

#[no_mangle]
pub unsafe extern "system" fn CreateMenu() -> HMENU {
    NtUserxCreateMenu()
}

#[no_mangle]
pub unsafe extern "system" fn CreatePopupMenu() -> HMENU {
    NtUserxCreatePopupMenu()
}

#[no_mangle]
pub unsafe extern "system" fn DrawMenuBar(hWnd: HWND) -> BOOL {
    NtUserxDrawMenuBar(hWnd)
}

#[no_mangle]
pub unsafe extern "system" fn EnableMenuItem(
    hMenu: HMENU,
    uIDEnableItem: UINT,
    uEnable: UINT,
) -> BOOL {
    NtUserEnableMenuItem(hMenu, uIDEnableItem, uEnable) as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn GetMenu(hWnd: HWND) -> HMENU {
    let Wnd: PWND = ValidateHwnd(hWnd);

    if Wnd.is_null() {
        return core::ptr::null_mut();
    }

    UlongToHandle((*Wnd).IDMenu as ULONG) as HMENU
}

#[no_mangle]
pub unsafe extern "system" fn GetMenuCheckMarkDimensions() -> LONG {
    MAKELONG(
        GetSystemMetrics(SM_CXMENUCHECK as i32) as WORD,
        GetSystemMetrics(SM_CYMENUCHECK as i32) as WORD,
    )
}

#[no_mangle]
pub unsafe extern "system" fn GetMenuContextHelpId(hmenu: HMENU) -> DWORD {
    let pMenu: PMENU = ValidateHandle(hmenu as HANDLE, TYPE_MENU) as PMENU;
    if !pMenu.is_null() {
        return (*pMenu).dwContextHelpId;
    }
    0
}

#[no_mangle]
pub unsafe extern "system" fn GetMenuDefaultItem(
    hMenu: HMENU,
    fByPos: UINT,
    gmdiFlags: UINT,
) -> UINT {
    let pMenu: PMENU;
    let mut gismc: DWORD = 0;

    pMenu = ValidateHandle(hMenu as HANDLE, TYPE_MENU) as PMENU;
    if pMenu.is_null() {
        return (-1i32) as UINT;
    }

    IntGetMenuDefaultItem(pMenu, fByPos as i32, gmdiFlags, &mut gismc)
}

#[no_mangle]
pub unsafe extern "system" fn GetMenuInfo(hmenu: HMENU, lpcmi: LPMENUINFO) -> i32 {
    let pMenu: PMENU;

    if lpcmi.is_null() || ((*lpcmi).cbSize as usize != core::mem::size_of::<MENUINFO>()) {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    pMenu = ValidateHandle(hmenu as HANDLE, TYPE_MENU) as PMENU;
    if pMenu.is_null() {
        return 0;
    }

    if (*lpcmi).fMask & MIM_BACKGROUND != 0 {
        (*lpcmi).hbrBack = (*pMenu).hbrBack;
    }

    if (*lpcmi).fMask & MIM_HELPID != 0 {
        (*lpcmi).dwContextHelpID = (*pMenu).dwContextHelpId;
    }

    if (*lpcmi).fMask & MIM_MAXHEIGHT != 0 {
        (*lpcmi).cyMax = (*pMenu).cyMax;
    }

    if (*lpcmi).fMask & MIM_MENUDATA != 0 {
        (*lpcmi).dwMenuData = (*pMenu).dwMenuData;
    }

    if (*lpcmi).fMask & MIM_STYLE != 0 {
        (*lpcmi).dwStyle = (*pMenu).fFlags & MNS_STYLE_MASK;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn GetMenuItemCount(hmenu: HMENU) -> core::ffi::c_int {
    let pMenu: PMENU = ValidateHandle(hmenu as HANDLE, TYPE_MENU) as PMENU;
    if !pMenu.is_null() {
        return (*pMenu).cItems as core::ffi::c_int;
    }
    -1
}

#[no_mangle]
pub unsafe extern "system" fn GetMenuItemID(mut hMenu: HMENU, mut nPos: core::ffi::c_int) -> UINT {
    let lpmi: *mut ITEM = MENU_FindItem(
        core::ptr::addr_of_mut!(hMenu),
        core::ptr::addr_of_mut!(nPos) as *mut UINT,
        MF_BYPOSITION,
    );
    if lpmi.is_null() {
        return -1i32 as UINT;
    }
    if !(*lpmi).spSubMenu.is_null() {
        return -1i32 as UINT;
    }
    (*lpmi).wID
}

#[no_mangle]
pub unsafe extern "system" fn GetMenuItemInfoA(
    hmenu: HMENU,
    item: UINT,
    bypos: BOOL,
    lpmii: LPMENUITEMINFOA,
) -> BOOL {
    let mut mii: MENUITEMINFOA = core::mem::zeroed();

    if (*lpmii).cbSize as usize != core::mem::size_of::<MENUITEMINFOA>()
        && (*lpmii).cbSize as usize
            != core::mem::size_of::<MENUITEMINFOA>() - core::mem::size_of_val(&mii.hbmpItem)
    {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    memcpy(
        core::ptr::addr_of_mut!(mii) as *mut core::ffi::c_void,
        lpmii as *const core::ffi::c_void,
        (*lpmii).cbSize as usize,
    );
    mii.cbSize = core::mem::size_of::<MENUITEMINFOA>() as UINT;
    let ret = GetMenuItemInfo_common(
        hmenu,
        item,
        bypos,
        core::ptr::addr_of_mut!(mii) as LPMENUITEMINFOW,
        0,
    );
    mii.cbSize = (*lpmii).cbSize;
    memcpy(
        lpmii as *mut core::ffi::c_void,
        core::ptr::addr_of!(mii) as *const core::ffi::c_void,
        mii.cbSize as usize,
    );
    ret
}

#[no_mangle]
pub unsafe extern "system" fn GetMenuItemInfoW(
    hMenu: HMENU,
    Item: UINT,
    bypos: BOOL,
    lpmii: LPMENUITEMINFOW,
) -> BOOL {
    let ret: BOOL;
    let mut mii: MENUITEMINFOW = core::mem::zeroed();
    if (*lpmii).cbSize as usize != core::mem::size_of::<MENUITEMINFOW>()
        && (*lpmii).cbSize as usize
            != core::mem::size_of::<MENUITEMINFOW>() - core::mem::size_of_val(&mii.hbmpItem)
    {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }
    memcpy(
        core::ptr::addr_of_mut!(mii) as *mut _,
        lpmii as *const _,
        (*lpmii).cbSize as usize,
    );
    mii.cbSize = core::mem::size_of::<MENUITEMINFOW>() as UINT;
    ret = GetMenuItemInfo_common(hMenu, Item, bypos, core::ptr::addr_of_mut!(mii), 1);
    mii.cbSize = (*lpmii).cbSize;
    memcpy(
        lpmii as *mut _,
        core::ptr::addr_of!(mii) as *const _,
        mii.cbSize as usize,
    );
    ret
}

#[no_mangle]
pub unsafe extern "system" fn GetMenuState(
    hMenu: HMENU,
    uId: UINT,
    uFlags: UINT,
) -> UINT {
    let mut hMenu = hMenu;
    let mut uId = uId;
    let pItem: PITEM;
    let mut Type: UINT = 0;

    pItem = MENU_FindItem(
        core::ptr::addr_of_mut!(hMenu),
        core::ptr::addr_of_mut!(uId),
        uFlags,
    );
    if pItem.is_null() {
        return -1i32 as UINT;
    }

    if (*pItem).Xlpstr.is_null() && !(*pItem).hbmp.is_null() {
        Type = MFT_BITMAP;
    }

    if !(*pItem).spSubMenu.is_null() {
        let pSubMenu: PMENU = DesktopPtrToUser((*pItem).spSubMenu as *mut _) as PMENU;
        let hsubmenu: HMENU = UserHMGetHandle(pSubMenu as *mut _) as HMENU;
        Type |= MF_POPUP;
        if IsMenu(hsubmenu) == 0 {
            return -1i32 as UINT;
        } else {
            return ((*pSubMenu).cItems << 8)
                | (((*pItem).fState | (*pItem).fType | Type) & 0xff);
        }
    } else {
        return (*pItem).fType | (*pItem).fState | Type;
    }
}

#[no_mangle]
pub unsafe extern "system" fn GetMenuStringA(
    hMenu: HMENU,
    uIDItem: UINT,
    lpString: LPSTR,
    nMaxCount: c_int,
    uFlag: UINT,
) -> c_int {
    let item: *mut ITEM;
    let text: LPWSTR;

    let mut hMenu = hMenu;
    let mut uIDItem = uIDItem;

    if !lpString.is_null() && nMaxCount != 0 {
        *lpString = 0;
    }

    item = MENU_FindItem(
        core::ptr::addr_of_mut!(hMenu),
        core::ptr::addr_of_mut!(uIDItem),
        uFlag,
    );
    if item.is_null() {
        SetLastError(ERROR_MENU_ITEM_NOT_FOUND);
        return 0;
    }

    text = if !(*item).Xlpstr.is_null() {
        DesktopPtrToUser((*item).Xlpstr as *mut _) as LPWSTR
    } else {
        core::ptr::null_mut()
    };

    if text.is_null() {
        return 0;
    }
    if lpString.is_null() || nMaxCount == 0 {
        return WideCharToMultiByte(
            CP_ACP,
            0,
            text,
            -1,
            core::ptr::null_mut(),
            0,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
    }
    if WideCharToMultiByte(
        CP_ACP,
        0,
        text,
        -1,
        lpString,
        nMaxCount,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    ) == 0
    {
        *lpString.offset((nMaxCount - 1) as isize) = 0;
    }
    strlen(lpString) as c_int
}

#[no_mangle]
pub unsafe extern "system" fn GetMenuStringW(
    hMenu: HMENU,
    uIDItem: UINT,
    lpString: LPWSTR,
    nMaxCount: core::ffi::c_int,
    uFlag: UINT,
) -> core::ffi::c_int {
    let item: *mut ITEM;
    let text: LPWSTR;

    let mut hMenu = hMenu;
    let mut uIDItem = uIDItem;

    if !lpString.is_null() && nMaxCount != 0 {
        *lpString.offset(0) = 0;
    }

    item = MENU_FindItem(
        core::ptr::addr_of_mut!(hMenu),
        core::ptr::addr_of_mut!(uIDItem),
        uFlag,
    );
    if item.is_null() {
        SetLastError(ERROR_MENU_ITEM_NOT_FOUND);
        return 0;
    }

    text = if !(*item).Xlpstr.is_null() {
        DesktopPtrToUser((*item).Xlpstr as *mut core::ffi::c_void) as LPWSTR
    } else {
        core::ptr::null_mut()
    };

    if lpString.is_null() || nMaxCount == 0 {
        return if !text.is_null() { strlenW(text) as core::ffi::c_int } else { 0 };
    }
    if text.is_null() {
        *lpString.offset(0) = 0;
        return 0;
    }
    lstrcpynW(lpString, text, nMaxCount);
    strlenW(lpString) as core::ffi::c_int
}

#[no_mangle]
pub unsafe extern "system" fn GetSubMenu(hMenu: HMENU, nPos: core::ffi::c_int) -> HMENU {
    let mut hMenu = hMenu;
    let mut nPos = nPos;
    let pItem: PITEM = MENU_FindItem(
        core::ptr::addr_of_mut!(hMenu),
        core::ptr::addr_of_mut!(nPos) as *mut UINT,
        MF_BYPOSITION,
    );
    if pItem.is_null() {
        return core::ptr::null_mut();
    }

    if !(*pItem).spSubMenu.is_null() {
        let pSubMenu: PMENU = DesktopPtrToUser((*pItem).spSubMenu as *mut _) as PMENU;
        let hsubmenu: HMENU = UserHMGetHandle(pSubMenu) as HMENU;
        if IsMenu(hsubmenu) != 0 {
            return hsubmenu;
        }
    }
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn GetSystemMenu(hWnd: HWND, bRevert: BOOL) -> HMENU {
    NtUserGetSystemMenu(hWnd, bRevert)
}

#[no_mangle]
pub unsafe extern "system" fn InsertMenuA(
    hMenu: HMENU,
    uPosition: UINT,
    uFlags: UINT,
    uIDNewItem: UINT_PTR,
    lpNewItem: LPCSTR,
) -> BOOL {
    let mut mii: MENUITEMINFOW = core::mem::zeroed();
    let mut UnicodeString: _UNICODE_STRING = core::mem::zeroed();

    RtlInitUnicodeString(&mut UnicodeString, core::ptr::null_mut());

    MENU_mnu2mnuii(uFlags, uIDNewItem, lpNewItem as LPCWSTR, &mut mii, 0);

    if !lpNewItem.is_null()
        && (mii.fMask & MIIM_STRING) != 0
        && mii.hbmpItem.is_null()
        && !mii.dwTypeData.is_null()
    {
        if RtlCreateUnicodeStringFromAsciiz(&mut UnicodeString, mii.dwTypeData as LPSTR) == 0 {
            SetLastError(ERROR_NOT_ENOUGH_MEMORY);
            return 0;
        }
        mii.dwTypeData = UnicodeString.Buffer;
        mii.cch = (UnicodeString.Length as UINT) / core::mem::size_of::<u16>() as UINT;
    } else {
        TRACE!(b"Handle bitmaps\n\0".as_ptr() as LPCSTR);
    }

    let res = NtUserThunkedMenuItemInfo(
        hMenu,
        uPosition,
        (uFlags & MF_BYPOSITION) as BOOL,
        1,
        &mut mii,
        &mut UnicodeString,
    );

    if !UnicodeString.Buffer.is_null() {
        RtlFreeUnicodeString(&mut UnicodeString);
    }

    res
}

#[no_mangle]
pub unsafe extern "system" fn InsertMenuItemA(
    hMenu: HMENU,
    uItem: UINT,
    fByPosition: BOOL,
    lpmii: LPCMENUITEMINFOA,
) -> BOOL {
    let mut mii: MENUITEMINFOW = core::mem::zeroed();
    let mut UnicodeString: _UNICODE_STRING = core::mem::zeroed();
    let res: BOOL;

    RtlInitUnicodeString(&mut UnicodeString, core::ptr::null_mut());

    if MENU_NormalizeMenuItemInfoStruct(lpmii as *const MENUITEMINFOW, &mut mii) == 0 {
        return 0;
    }

    if (((mii.fMask & MIIM_STRING) != 0)
        || ((mii.fMask & MIIM_TYPE) != 0 && (MENU_ITEM_TYPE(mii.fType) == MF_STRING)))
        && !mii.dwTypeData.is_null()
        && GdiValidateHandle(mii.dwTypeData as HGDIOBJ) == 0
    {
        if RtlCreateUnicodeStringFromAsciiz(&mut UnicodeString, mii.dwTypeData as LPSTR) == 0 {
            SetLastError(ERROR_NOT_ENOUGH_MEMORY);
            return 0;
        }
        mii.dwTypeData = UnicodeString.Buffer;
        mii.cch = (UnicodeString.Length as usize / core::mem::size_of::<u16>()) as UINT;
    }

    res = NtUserThunkedMenuItemInfo(hMenu, uItem, fByPosition, 1, &mut mii, &mut UnicodeString);

    if !UnicodeString.Buffer.is_null() {
        RtlFreeUnicodeString(&mut UnicodeString);
    }
    res
}

#[no_mangle]
pub unsafe extern "system" fn InsertMenuItemW(
    hMenu: HMENU,
    uItem: UINT,
    fByPosition: BOOL,
    lpmii: LPCMENUITEMINFOW,
) -> BOOL {
    let mut mii: MENUITEMINFOW = core::mem::zeroed();
    let mut MenuText: _UNICODE_STRING = core::mem::zeroed();
    let mut res: BOOL = 0;

    RtlInitUnicodeString(&mut MenuText, core::ptr::null_mut());

    if MENU_NormalizeMenuItemInfoStruct(lpmii as *const MENUITEMINFOW, &mut mii) == 0 {
        return 0;
    }

    if ((mii.fMask & MIIM_STRING) != 0
        || ((mii.fMask & MIIM_TYPE) != 0 && (mii.fType & MENU_TYPE_MASK) == MF_STRING))
        && mii.dwTypeData != core::ptr::null_mut()
        && GdiValidateHandle(mii.dwTypeData as HGDIOBJ) == 0
    {
        RtlInitUnicodeString(&mut MenuText, (*lpmii).dwTypeData as PWSTR);
        mii.dwTypeData = MenuText.Buffer;
        mii.cch = (MenuText.Length as u32) / (core::mem::size_of::<u16>() as u32);
    }

    res = NtUserThunkedMenuItemInfo(
        hMenu,
        uItem,
        fByPosition,
        1,
        &mut mii,
        &mut MenuText,
    );
    res
}

#[no_mangle]
pub unsafe extern "system" fn InsertMenuW(
    hMenu: HMENU,
    uPosition: UINT,
    uFlags: UINT,
    uIDNewItem: UINT_PTR,
    lpNewItem: LPCWSTR,
) -> BOOL {
    let mut mii: MENUITEMINFOW = core::mem::zeroed();
    let mut MenuText: _UNICODE_STRING = core::mem::zeroed();
    let res: BOOL;

    RtlInitUnicodeString(&mut MenuText, core::ptr::null_mut());

    MENU_mnu2mnuii(uFlags, uIDNewItem, lpNewItem, &mut mii, 1);

    if !lpNewItem.is_null()
        && (mii.fMask & MIIM_STRING) != 0
        && mii.hbmpItem.is_null()
        && mii.dwTypeData != core::ptr::null_mut()
    {
        RtlInitUnicodeString(&mut MenuText, mii.dwTypeData as PWSTR);
        mii.dwTypeData = MenuText.Buffer;
        mii.cch = (MenuText.Length as usize / core::mem::size_of::<u16>()) as UINT;
    }

    res = NtUserThunkedMenuItemInfo(
        hMenu,
        uPosition,
        ((uFlags & MF_BYPOSITION) != 0) as BOOL,
        1,
        &mut mii,
        &mut MenuText,
    );
    res
}

#[no_mangle]
pub unsafe extern "system" fn IsMenu(Menu: HMENU) -> BOOL {
    if !ValidateHandle(Menu as HANDLE, TYPE_MENU).is_null() {
        return 1;
    }
    0
}

#[no_mangle]
pub unsafe extern "system" fn LoadMenuA(hInstance: HINSTANCE, lpMenuName: LPCSTR) -> HMENU {
    let Resource = FindResourceA(hInstance, lpMenuName, MAKEINTRESOURCEA(4));
    if Resource.is_null() {
        return core::ptr::null_mut();
    }
    LoadMenuIndirectA(LoadResource(hInstance, Resource) as *const MENUTEMPLATE)
}

#[no_mangle]
pub unsafe extern "system" fn LoadMenuIndirectA(lpMenuTemplate: *const MENUTEMPLATE) -> HMENU {
    LoadMenuIndirectW(lpMenuTemplate)
}

#[no_mangle]
pub unsafe extern "system" fn LoadMenuIndirectW(lpMenuTemplate: *const MENUTEMPLATE) -> HMENU {
    let hMenu: HMENU;
    let version: u16;
    let offset: u16;
    let mut p: LPCSTR = lpMenuTemplate as LPCSTR;

    version = *(p as *const u16);
    p = p.add(core::mem::size_of::<u16>());

    match version {
        0 => {
            offset = *(p as *const u16);
            p = p.add(core::mem::size_of::<u16>() + offset as usize);
            hMenu = CreateMenu();
            if hMenu.is_null() {
                return core::ptr::null_mut();
            }
            if MENU_ParseResource(p, hMenu).is_null() {
                DestroyMenu(hMenu);
                return core::ptr::null_mut();
            }
            hMenu
        }
        1 => {
            offset = *(p as *const u16);
            p = p.add(core::mem::size_of::<u16>() + offset as usize);
            hMenu = CreateMenu();
            if hMenu.is_null() {
                return core::ptr::null_mut();
            }
            if MENUEX_ParseResource(p, hMenu).is_null() {
                DestroyMenu(hMenu);
                return core::ptr::null_mut();
            }
            hMenu
        }
        _ => core::ptr::null_mut(),
    }
}

#[no_mangle]
pub unsafe extern "system" fn LoadMenuW(hInstance: HINSTANCE, lpMenuName: LPCWSTR) -> HMENU {
    let Resource = FindResourceW(hInstance, lpMenuName, RT_MENU);
    if Resource.is_null() {
        return core::ptr::null_mut();
    }
    LoadMenuIndirectW(LoadResource(hInstance, Resource) as *const MENUTEMPLATE)
}

#[no_mangle]
pub unsafe extern "system" fn ModifyMenuA(
    hMenu: HMENU,
    uPosition: UINT,
    uFlags: UINT,
    uIDNewItem: UINT_PTR,
    lpNewItem: LPCSTR,
) -> BOOL {
    let mut mii: MENUITEMINFOW = core::mem::zeroed();
    let mut UnicodeString: _UNICODE_STRING = core::mem::zeroed();

    RtlInitUnicodeString(&mut UnicodeString, core::ptr::null_mut());

    MENU_mnu2mnuii(uFlags, uIDNewItem, lpNewItem as LPCWSTR, &mut mii, 0);

    if !lpNewItem.is_null()
        && (mii.fMask & MIIM_STRING) != 0
        && mii.hbmpItem.is_null()
        && !mii.dwTypeData.is_null()
    {
        if RtlCreateUnicodeStringFromAsciiz(&mut UnicodeString, mii.dwTypeData as LPSTR) == 0 {
            SetLastError(ERROR_NOT_ENOUGH_MEMORY);
            return 0;
        }
        mii.dwTypeData = UnicodeString.Buffer;
        mii.cch = (UnicodeString.Length as usize / core::mem::size_of::<u16>()) as UINT;
    } else {
        TRACE!(b"Handle bitmaps\n\0".as_ptr() as LPCSTR);
    }

    let res = NtUserThunkedMenuItemInfo(
        hMenu,
        uPosition,
        (uFlags & MF_BYPOSITION) as BOOL,
        0,
        &mut mii,
        &mut UnicodeString,
    );

    if !UnicodeString.Buffer.is_null() {
        RtlFreeUnicodeString(&mut UnicodeString);
    }
    res
}

#[no_mangle]
pub unsafe extern "system" fn ModifyMenuW(
    hMenu: HMENU,
    uPosition: UINT,
    uFlags: UINT,
    uIDNewItem: UINT_PTR,
    lpNewItem: LPCWSTR,
) -> BOOL {
    let mut mii: MENUITEMINFOW = core::mem::zeroed();
    let mut MenuText: _UNICODE_STRING = core::mem::zeroed();
    let res: BOOL;

    RtlInitUnicodeString(&mut MenuText, core::ptr::null_mut());

    MENU_mnu2mnuii(uFlags, uIDNewItem, lpNewItem, &mut mii, 1);

    if !lpNewItem.is_null()
        && (mii.fMask & MIIM_STRING) != 0
        && mii.hbmpItem.is_null()
        && mii.dwTypeData != core::ptr::null_mut()
    {
        RtlInitUnicodeString(&mut MenuText, mii.dwTypeData as PWSTR);
        mii.dwTypeData = MenuText.Buffer;
        mii.cch = (MenuText.Length / core::mem::size_of::<u16>() as u16) as UINT;
    } else {
        TRACE!(b"Handle bitmaps\n\0".as_ptr() as LPCSTR);
    }

    res = NtUserThunkedMenuItemInfo(
        hMenu,
        uPosition,
        (uFlags & MF_BYPOSITION) as BOOL,
        0,
        &mut mii,
        &mut MenuText,
    );
    res
}

#[no_mangle]
pub unsafe extern "system" fn SetMenu(hWnd: HWND, hMenu: HMENU) -> BOOL {
    NtUserSetMenu(hWnd, hMenu, 1u8 as _)
}

#[no_mangle]
pub unsafe extern "system" fn SetMenuInfo(hmenu: HMENU, lpcmi: LPCMENUINFO) -> BOOL {
    let mut mi: MENUINFO = core::mem::zeroed();
    let res: BOOL = 0;

    if lpcmi.is_null() || (*lpcmi).cbSize as usize != core::mem::size_of::<MENUINFO>() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return res;
    }

    memcpy(
        core::ptr::addr_of_mut!(mi) as *mut core::ffi::c_void,
        lpcmi as *const core::ffi::c_void,
        core::mem::size_of::<MENUINFO>(),
    );
    NtUserThunkedMenuInfo(hmenu, core::ptr::addr_of_mut!(mi) as LPCMENUINFO)
}

#[no_mangle]
pub unsafe extern "system" fn SetMenuItemBitmaps(
    hMenu: HMENU,
    uPosition: UINT,
    uFlags: UINT,
    hBitmapUnchecked: HBITMAP,
    hBitmapChecked: HBITMAP,
) -> BOOL {
    let mut uItem: MENUITEMINFOW = core::mem::zeroed();
    uItem.cbSize = core::mem::size_of::<MENUITEMINFOW>() as UINT;
    uItem.fMask = MIIM_CHECKMARKS;
    uItem.hbmpUnchecked = hBitmapUnchecked;
    uItem.hbmpChecked = hBitmapChecked;
    SetMenuItemInfoW(
        hMenu,
        uPosition,
        (uFlags & MF_BYPOSITION) as BOOL,
        &uItem,
    )
}

#[no_mangle]
pub unsafe extern "system" fn SetMenuItemInfoA(
    hmenu: HMENU,
    item: UINT,
    bypos: BOOL,
    lpmii: LPCMENUITEMINFOA,
) -> BOOL {
    let mut mii: MENUITEMINFOW = core::mem::zeroed();
    let mut UnicodeString: _UNICODE_STRING = core::mem::zeroed();
    let Ret: BOOL;

    RtlInitUnicodeString(&mut UnicodeString, core::ptr::null());

    if MENU_NormalizeMenuItemInfoStruct(lpmii as *const MENUITEMINFOW, &mut mii) == 0 {
        return 0;
    }

    if ((mii.fMask & MIIM_STRING) != 0
        || ((mii.fMask & MIIM_TYPE) != 0 && MENU_ITEM_TYPE(mii.fType) == MF_STRING))
        && !mii.dwTypeData.is_null()
        && GdiValidateHandle(mii.dwTypeData as HGDIOBJ) == 0
    {
        if RtlCreateUnicodeStringFromAsciiz(&mut UnicodeString, mii.dwTypeData as LPSTR) == 0 {
            SetLastError(ERROR_NOT_ENOUGH_MEMORY);
            return 0;
        }
        mii.dwTypeData = UnicodeString.Buffer;
        mii.cch = (UnicodeString.Length as usize / core::mem::size_of::<u16>()) as UINT;
    } else {
        UnicodeString.Buffer = core::ptr::null_mut();
    }

    Ret = NtUserThunkedMenuItemInfo(hmenu, item, bypos, 0, &mut mii, &mut UnicodeString);
    if !UnicodeString.Buffer.is_null() {
        RtlFreeUnicodeString(&mut UnicodeString);
    }
    Ret
}

#[no_mangle]
pub unsafe extern "system" fn SetMenuItemInfoW(
    hMenu: HMENU,
    uItem: UINT,
    fByPosition: BOOL,
    lpmii: LPCMENUITEMINFOW,
) -> BOOL {
    let mut MenuItemInfoW: MENUITEMINFOW = core::mem::zeroed();
    let mut UnicodeString: _UNICODE_STRING = core::mem::zeroed();
    let Ret: BOOL;

    RtlInitUnicodeString(&mut UnicodeString, core::ptr::null_mut());

    if MENU_NormalizeMenuItemInfoStruct(lpmii as *const MENUITEMINFOW, &mut MenuItemInfoW) == 0 {
        return 0;
    }

    if ((MenuItemInfoW.fMask & MIIM_STRING) != 0
        || ((MenuItemInfoW.fMask & MIIM_TYPE) != 0
            && (MENU_ITEM_TYPE(MenuItemInfoW.fType) == MF_STRING)))
        && !MenuItemInfoW.dwTypeData.is_null()
        && GdiValidateHandle(MenuItemInfoW.dwTypeData as HGDIOBJ) == 0
    {
        RtlInitUnicodeString(&mut UnicodeString, MenuItemInfoW.dwTypeData as PCWSTR);
        MenuItemInfoW.cch = strlenW(MenuItemInfoW.dwTypeData);
    }

    Ret = NtUserThunkedMenuItemInfo(
        hMenu,
        uItem,
        fByPosition,
        0,
        &mut MenuItemInfoW as LPMENUITEMINFOW,
        &mut UnicodeString,
    );

    Ret
}

#[no_mangle]
pub unsafe extern "system" fn SetSystemMenu(hwnd: HWND, hMenu: HMENU) -> BOOL {
    if hwnd.is_null() {
        SetLastError(ERROR_INVALID_WINDOW_HANDLE);
        return 0;
    }
    if hMenu.is_null() {
        SetLastError(ERROR_INVALID_MENU_HANDLE);
        return 0;
    }
    NtUserSetSystemMenu(hwnd, hMenu)
}

#[no_mangle]
pub unsafe extern "system" fn TrackPopupMenu(
    Menu: HMENU,
    Flags: UINT,
    x: ::core::ffi::c_int,
    y: ::core::ffi::c_int,
    Reserved: ::core::ffi::c_int,
    Wnd: HWND,
    Rect: *const RECT,
) -> ::core::ffi::c_int {
    NtUserTrackPopupMenuEx(Menu, Flags, x, y, Wnd, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "system" fn MenuWindowProcA(
    hWnd: HWND,
    Result: ULONG_PTR,
    Msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    if Msg < WM_USER {
        return PopupMenuWndProcA(hWnd, Msg, wParam, lParam);
    }
    NtUserMessageCall(hWnd, Msg, wParam, lParam, Result, FNID_MENU, 1) as LRESULT
}

#[no_mangle]
pub unsafe extern "system" fn MenuWindowProcW(
    hWnd: HWND,
    Result: ULONG_PTR,
    Msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    if Msg < WM_USER {
        return PopupMenuWndProcW(hWnd, Msg, wParam, lParam);
    }
    NtUserMessageCall(hWnd, Msg, wParam, lParam, Result, FNID_MENU, 0) as LRESULT
}

#[no_mangle]
pub unsafe extern "system" fn ChangeMenuW(
    hMenu: HMENU,
    cmd: UINT,
    lpszNewItem: LPCWSTR,
    cmdInsert: UINT,
    flags: UINT,
) -> BOOL {
    match flags & (MF_APPEND | MF_DELETE | MF_CHANGE | MF_REMOVE | MF_INSERT) {
        MF_APPEND => AppendMenuW(hMenu, flags & !MF_APPEND, cmdInsert as UINT_PTR, lpszNewItem),

        MF_DELETE => DeleteMenu(hMenu, cmd, flags & !MF_DELETE),

        MF_CHANGE => ModifyMenuW(hMenu, cmd, flags & !MF_CHANGE, cmdInsert as UINT_PTR, lpszNewItem),

        MF_REMOVE => RemoveMenu(
            hMenu,
            if flags & MF_BYPOSITION != 0 { cmd } else { cmdInsert },
            flags & !MF_REMOVE,
        ),

        _ => InsertMenuW(hMenu, cmd, flags, cmdInsert as UINT_PTR, lpszNewItem),
    }
}

#[no_mangle]
pub unsafe extern "system" fn ChangeMenuA(
    hMenu: HMENU,
    cmd: UINT,
    lpszNewItem: LPCSTR,
    cmdInsert: UINT,
    flags: UINT,
) -> BOOL {

    match flags & (MF_APPEND | MF_DELETE | MF_CHANGE | MF_REMOVE | MF_INSERT) {
        x if x == MF_APPEND => {
            AppendMenuA(hMenu, flags & !MF_APPEND, cmdInsert as UINT_PTR, lpszNewItem)
        }
        x if x == MF_DELETE => DeleteMenu(hMenu, cmd, flags & !MF_DELETE),
        x if x == MF_CHANGE => {
            ModifyMenuA(hMenu, cmd, flags & !MF_CHANGE, cmdInsert as UINT_PTR, lpszNewItem)
        }
        x if x == MF_REMOVE => RemoveMenu(
            hMenu,
            if (flags & MF_BYPOSITION) != 0 { cmd } else { cmdInsert },
            flags & !MF_REMOVE,
        ),
        _ => InsertMenuA(hMenu, cmd, flags, cmdInsert as UINT_PTR, lpszNewItem),
    }
}
