use win32k_sys::win32k::*;
#[allow(unused_imports)]
use core::ffi::c_void;

// ValidateHwnd + NT_SUCCESS live in the substrate.
use crate::internal::ValidateHwnd;
use crate::winbase::NT_SUCCESS;

// ZeroMemory(dst, len) — Win32 macro for RtlZeroMemory.
#[inline]
unsafe fn ZeroMemory(dst: PVOID, len: SIZE_T) {
    core::ptr::write_bytes(dst as *mut u8, 0, len as usize);
}

// ICLASS_TO_MASK(iCls) = (1 << iCls)   — win32ss/include/ntuser.h
#[inline]
fn ICLASS_TO_MASK(iCls: u32) -> DWORD {
    1u32 << iCls
}

// IS_IMM_MODE() = gpsi && (gpsi->dwSRVIFlags & SRVINFO_IMM32)  — ntuser.h macro.
#[inline]
unsafe fn IS_IMM_MODE() -> BOOL {
    let psi = crate::misc::dllmain::gpsi;
    (!psi.is_null() && ((*psi).dwSRVIFlags & SRVINFO_IMM32) != 0) as BOOL
}

// user32's own exports / gdi32 imports referenced here (extern "system" = stdcall on i386).
extern "system" {
    fn DefDlgProcW(hwnd: HWND, uMsg: u32, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn DefDlgProcA(hwnd: HWND, uMsg: u32, wParam: WPARAM, lParam: LPARAM) -> LRESULT;

    // gdi32 imports
    fn CreateRectRgn(x1: INT, y1: INT, x2: INT, y2: INT) -> HRGN;
    fn GetClipRgn(hdc: HDC, hrgn: HRGN) -> INT;
    fn DeleteObject(ho: HGDIOBJ) -> BOOL;
    fn DPtoLP(hdc: HDC, lppt: LPPOINT, c: INT) -> BOOL;
    fn GetLayout(hdc: HDC) -> DWORD;
    fn IntersectClipRect(hdc: HDC, left: INT, top: INT, right: INT, bottom: INT) -> INT;

    // user32 own-exports (resolve within this DLL).
    fn LoadCursorW(hInstance: HINSTANCE, lpCursorName: LPCWSTR) -> HCURSOR;
    fn RegisterClassExWOWW(
        lpwcx: *mut WNDCLASSEXW,
        pdwWowData: LPDWORD,
        fnID: u16,
        dwFlags: u32,
        ChkRegCls: i32,
    ) -> ATOM;
    // controls.h: LRESULT WINAPI User32DefWindowProc(HWND,UINT,WPARAM,LPARAM,BOOL)
    // — the 5-arg internal dispatcher (defwnd.rs `@20`), NOT a 4-arg public proc.
    fn User32DefWindowProc(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM, bUnicode: BOOL) -> LRESULT;

    // Control / window-class WndProcs — #[no_mangle] exports of the sibling Rust
    // control modules (controls/button.rs, controls/combo.rs, …). They link by
    // symbol, so mirror the C regcontrol.c references with a local decl block.
    fn ScrollBarWndProcA(hwnd: HWND, msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn ScrollBarWndProcW(hwnd: HWND, msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn IconTitleWndProc(hwnd: HWND, msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn PopupMenuWndProcA(hwnd: HWND, msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn PopupMenuWndProcW(hwnd: HWND, msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn DesktopWndProcA(hwnd: HWND, msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn DesktopWndProcW(hwnd: HWND, msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn DefWindowProcA(hwnd: HWND, msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn DefWindowProcW(hwnd: HWND, msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn SwitchWndProcA(hwnd: HWND, msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn SwitchWndProcW(hwnd: HWND, msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn ButtonWndProcA(hwnd: HWND, msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn ButtonWndProcW(hwnd: HWND, msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn ComboWndProcA(hwnd: HWND, msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn ComboWndProcW(hwnd: HWND, msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn ListBoxWndProcA(hwnd: HWND, msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn ListBoxWndProcW(hwnd: HWND, msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn EditWndProcA(hwnd: HWND, msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn EditWndProcW(hwnd: HWND, msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn MDIClientWndProcA(hwnd: HWND, msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn MDIClientWndProcW(hwnd: HWND, msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn StaticWndProcA(hwnd: HWND, msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn StaticWndProcW(hwnd: HWND, msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn ImeWndProcA(hwnd: HWND, msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn ImeWndProcW(hwnd: HWND, msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn GhostWndProcA(hwnd: HWND, msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;

    // *_common workers carry the extra `unicode: BOOL` arg (WNDPROC_EX).
    fn ButtonWndProc_common(hwnd: HWND, msg: UINT, wParam: WPARAM, lParam: LPARAM, u: BOOL) -> LRESULT;
    fn ComboWndProc_common(hwnd: HWND, msg: UINT, wParam: WPARAM, lParam: LPARAM, u: BOOL) -> LRESULT;
    fn ListBoxWndProc_common(hwnd: HWND, msg: UINT, wParam: WPARAM, lParam: LPARAM, u: BOOL) -> LRESULT;
    fn EditWndProc_common(hwnd: HWND, msg: UINT, wParam: WPARAM, lParam: LPARAM, u: BOOL) -> LRESULT;
    fn MDIClientWndProc_common(hwnd: HWND, msg: UINT, wParam: WPARAM, lParam: LPARAM, u: BOOL) -> LRESULT;
    fn StaticWndProc_common(hwnd: HWND, msg: UINT, wParam: WPARAM, lParam: LPARAM, u: BOOL) -> LRESULT;
    fn GhostWndProc_common(hwnd: HWND, msg: UINT, wParam: WPARAM, lParam: LPARAM, u: BOOL) -> LRESULT;

    // User32Instance — dllmain.c / user32p.h global.
    static User32Instance: HINSTANCE;
}

// `GhostWndProcW` and `ImeWndProc_common` are file-private (non-#[no_mangle]) in
// their sibling modules, so they have no linkable symbol. Reconstruct the faithful
// thin wrappers here (they only forward to the public *_common / A/W exports).
unsafe extern "system" fn GhostWndProcW(hwnd: HWND, msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    GhostWndProc_common(hwnd, msg, wParam, lParam, 1)
}
#[no_mangle]
pub unsafe extern "system" fn ImeWndProc_common(hwnd: HWND, msg: UINT, wParam: WPARAM, lParam: LPARAM, unicode: BOOL) -> LRESULT {
    if unicode != 0 {
        ImeWndProcW(hwnd, msg, wParam, lParam)
    } else {
        ImeWndProcA(hwnd, msg, wParam, lParam)
    }
}

// ---- file-scope preamble (translated) ----
#[no_mangle]
pub static mut RegisterDefaultClasses: DWORD = 0;
static mut pfnClientA: PFNCLIENT = unsafe { core::mem::zeroed() };
static mut pfnClientW: PFNCLIENT = unsafe { core::mem::zeroed() };

/// This user32's own client proc tables (A, W). Under WoW64 they are the only
/// procs the guest can call: gpsi's copies are the 64-bit user32's.
pub(crate) unsafe fn guest_pfn_client() -> (*const PFNCLIENT, *const PFNCLIENT) {
    (core::ptr::addr_of!(pfnClientA), core::ptr::addr_of!(pfnClientW))
}
static mut pfnClientWorker: PFNCLIENTWORKER = unsafe { core::mem::zeroed() };

#[repr(C)]
pub struct g_SysClasses_entry {
    pub desc: *const builtin_class_descr,
    pub fnid: WORD,
    pub ClsId: WORD,
}
unsafe impl Sync for g_SysClasses_entry {}

// Each sibling control module defines its own (layout-identical, #[repr(C)])
// `builtin_class_descr` + a #[no_mangle] `*_builtin_class` static. Cast each to the
// pointer type of our local struct — same layout, distinct nominal type.
pub static g_SysClasses: [g_SysClasses_entry; 10] = [
    g_SysClasses_entry { desc: &crate::windows::dialog::DIALOG_builtin_class    as *const _ as *const builtin_class_descr, fnid: FNID_DIALOG as WORD,    ClsId: ICLS_DIALOG as WORD },
    g_SysClasses_entry { desc: &crate::controls::combo::COMBO_builtin_class      as *const _ as *const builtin_class_descr, fnid: FNID_COMBOBOX as WORD,  ClsId: ICLS_COMBOBOX as WORD },
    g_SysClasses_entry { desc: &crate::controls::listbox::COMBOLBOX_builtin_class as *const _ as *const builtin_class_descr, fnid: FNID_COMBOLBOX as WORD, ClsId: ICLS_COMBOLBOX as WORD },
    g_SysClasses_entry { desc: &MDICLIENT_builtin_class                         as *const _ as *const builtin_class_descr, fnid: FNID_MDICLIENT as WORD, ClsId: ICLS_MDICLIENT as WORD },
    g_SysClasses_entry { desc: &crate::controls::button::BUTTON_builtin_class    as *const _ as *const builtin_class_descr, fnid: FNID_BUTTON as WORD,    ClsId: ICLS_BUTTON as WORD },
    g_SysClasses_entry { desc: &crate::controls::listbox::LISTBOX_builtin_class  as *const _ as *const builtin_class_descr, fnid: FNID_LISTBOX as WORD,   ClsId: ICLS_LISTBOX as WORD },
    g_SysClasses_entry { desc: &crate::controls::edit::EDIT_builtin_class        as *const _ as *const builtin_class_descr, fnid: FNID_EDIT as WORD,      ClsId: ICLS_EDIT as WORD },
    g_SysClasses_entry { desc: &crate::controls::static_::STATIC_builtin_class   as *const _ as *const builtin_class_descr, fnid: FNID_STATIC as WORD,    ClsId: ICLS_STATIC as WORD },
    g_SysClasses_entry { desc: &crate::controls::ghost::GHOST_builtin_class      as *const _ as *const builtin_class_descr, fnid: FNID_GHOST as WORD,     ClsId: ICLS_GHOST as WORD },
    g_SysClasses_entry { desc: &IME_builtin_class                               as *const _ as *const builtin_class_descr, fnid: FNID_IME as WORD,       ClsId: ICLS_IME as WORD },
];

// builtin_class_descr — the regcontrol.h cross-TU struct. Each sibling control
// module declares its own layout-identical copy; this is regcontrol's own.
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

const IDC_ARROW_W: LPCWSTR = 32512 as LPCWSTR; // MAKEINTRESOURCE(IDC_ARROW)

// "MDIClient" / "IME" class name literals (UTF-16, NUL-terminated).
static MDICLIENT_NAME: [u16; 10] = [b'M' as u16, b'D' as u16, b'I' as u16, b'C' as u16, b'l' as u16, b'i' as u16, b'e' as u16, b'n' as u16, b't' as u16, 0];
static IME_NAME: [u16; 4] = [b'I' as u16, b'M' as u16, b'E' as u16, 0];

// MDICLIENT_builtin_class / IME_builtin_class — faithful to windows/mdi.c:213 and
// misc/imm.c:1183. The Rust mdi/imm modules don't emit these data symbols, so the
// table owner (regcontrol) carries them. procA/procW link to the sibling exports.
#[no_mangle]
pub static MDICLIENT_builtin_class: builtin_class_descr = builtin_class_descr {
    name: MDICLIENT_NAME.as_ptr(),
    style: 0,
    procA: unsafe { core::mem::transmute::<unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT, WNDPROC>(MDIClientWndProcA) },
    procW: unsafe { core::mem::transmute::<unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT, WNDPROC>(MDIClientWndProcW) },
    extra: core::mem::size_of::<MDIWND>() as INT,
    cursor: IDC_ARROW_W,
    brush: (COLOR_APPWORKSPACE + 1) as HBRUSH,
};

#[no_mangle]
pub static IME_builtin_class: builtin_class_descr = builtin_class_descr {
    name: IME_NAME.as_ptr(),
    style: CS_GLOBALCLASS,
    procA: unsafe { core::mem::transmute::<unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT, WNDPROC>(ImeWndProcA) },
    procW: unsafe { core::mem::transmute::<unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT, WNDPROC>(ImeWndProcW) },
    extra: (core::mem::size_of::<IMEWND>() - core::mem::size_of::<WND>()) as INT,
    cursor: IDC_ARROW_W,
    brush: 0 as HBRUSH,
};

#[no_mangle]
pub unsafe extern "C" fn set_control_clipping(hdc: HDC, rect: *const RECT) -> HRGN {
    let mut rc: RECT = *rect;
    let mut hrgn: HRGN = CreateRectRgn(0, 0, 0, 0);

    if GetClipRgn(hdc, hrgn) != 1 {
        DeleteObject(hrgn as _);
        hrgn = core::ptr::null_mut();
    }
    DPtoLP(hdc, core::ptr::addr_of_mut!(rc) as *mut POINT, 2);
    if GetLayout(hdc) & LAYOUT_RTL != 0 {
        rc.left += 1;
        rc.right += 1;
    }
    IntersectClipRect(hdc, rc.left, rc.top, rc.right, rc.bottom);
    hrgn
}

#[no_mangle]
pub unsafe extern "system" fn RegisterSystemControls() -> BOOL {
    let mut WndClass: WNDCLASSEXW = core::mem::zeroed();
    let mut i: UINT;
    let mut atom: ATOM;

    if RegisterDefaultClasses != 0 {
        return TRUE as BOOL;
    }

    ZeroMemory(
        core::ptr::addr_of_mut!(WndClass) as *mut _,
        core::mem::size_of::<WNDCLASSEXW>() as _,
    );

    WndClass.cbSize = core::mem::size_of::<WNDCLASSEXW>() as UINT;

    i = 0;
    while i != (core::mem::size_of_val(&g_SysClasses) / core::mem::size_of_val(&g_SysClasses[0])) as UINT {
        let idx = i as usize;

        if g_SysClasses[idx].fnid == FNID_IME as WORD {
            if IS_IMM_MODE() == 0 || (RegisterDefaultClasses & ICLASS_TO_MASK(ICLS_IME)) != 0 {
                i += 1;
                continue;
            }
        }

        WndClass.lpszClassName = (*g_SysClasses[idx].desc).name;

        WndClass.style = (*g_SysClasses[idx].desc).style | CS_GLOBALCLASS;
        WndClass.lpfnWndProc = (*g_SysClasses[idx].desc).procW;
        WndClass.cbWndExtra = (*g_SysClasses[idx].desc).extra;
        WndClass.hCursor = LoadCursorW(core::ptr::null_mut(), (*g_SysClasses[idx].desc).cursor);
        WndClass.hbrBackground = (*g_SysClasses[idx].desc).brush;

        atom = RegisterClassExWOWW(
            core::ptr::addr_of_mut!(WndClass),
            core::ptr::null_mut(),
            g_SysClasses[idx].fnid,
            0,
            FALSE as BOOL,
        );
        if atom != 0 {
            RegisterDefaultClasses |= ICLASS_TO_MASK(g_SysClasses[idx].ClsId as u32);
        }

        i += 1;
    }

    TRUE as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn MsgWindowProc(hwnd: HWND, message: u32, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    let pWnd: PWND = ValidateHwnd(hwnd);
    if !pWnd.is_null() {
        if (*pWnd).fnid == 0 {
            NtUserSetWindowFNID(hwnd, FNID_MESSAGEWND as u16);
        }
    }

    if message == WM_NCCREATE {
        return 1 as LRESULT;
    }

    if message == WM_DESTROY {
        NtUserSetWindowFNID(hwnd, FNID_DESTROY as u16);
    }

    // C `DefWindowProc` is the winuser.h macro → `DefWindowProcW` (user32 is UNICODE).
    DefWindowProcW(hwnd, message, wParam, lParam)
}

unsafe extern "system" fn DialogWndProc_common(hwnd: HWND, uMsg: u32, wParam: WPARAM, lParam: LPARAM, unicode: i32) -> LRESULT {
    if unicode != 0 {
        return DefDlgProcW(hwnd, uMsg, wParam, lParam);
    }
    DefDlgProcA(hwnd, uMsg, wParam, lParam)
}

#[no_mangle]
// `extern "system"` and the bindgen `WNDPROC`/`WNDPROC_EX` (`extern "stdcall"` on
// i386, `extern "C"` elsewhere) have the *same* per-arch calling convention but are
// nominally-distinct fn types to rustc. These transmute the fn item to the field type.
#[inline]
fn wp(f: unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT) -> WNDPROC {
    unsafe { core::mem::transmute::<unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT, WNDPROC>(f) }
}
#[inline]
fn wpx(f: unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM, BOOL) -> LRESULT) -> WNDPROC_EX {
    unsafe { core::mem::transmute::<unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM, BOOL) -> LRESULT, WNDPROC_EX>(f) }
}

#[no_mangle]
pub unsafe extern "system" fn RegisterClientPFN() -> BOOL {
    let Status: NTSTATUS;

    pfnClientA.pfnScrollBarWndProc = wp(ScrollBarWndProcA);
    pfnClientW.pfnScrollBarWndProc = wp(ScrollBarWndProcW);
    pfnClientA.pfnTitleWndProc = wp(IconTitleWndProc);
    pfnClientW.pfnTitleWndProc = wp(IconTitleWndProc);
    pfnClientA.pfnMenuWndProc = wp(PopupMenuWndProcA);
    pfnClientW.pfnMenuWndProc = wp(PopupMenuWndProcW);
    pfnClientA.pfnDesktopWndProc = wp(DesktopWndProcA);
    pfnClientW.pfnDesktopWndProc = wp(DesktopWndProcW);
    pfnClientA.pfnDefWindowProc = wp(DefWindowProcA);
    pfnClientW.pfnDefWindowProc = wp(DefWindowProcW);
    pfnClientA.pfnMessageWindowProc = wp(MsgWindowProc);
    pfnClientW.pfnMessageWindowProc = wp(MsgWindowProc);
    pfnClientA.pfnSwitchWindowProc = wp(SwitchWndProcA);
    pfnClientW.pfnSwitchWindowProc = wp(SwitchWndProcW);
    pfnClientA.pfnButtonWndProc = wp(ButtonWndProcA);
    pfnClientW.pfnButtonWndProc = wp(ButtonWndProcW);
    pfnClientA.pfnComboBoxWndProc = wp(ComboWndProcA);
    pfnClientW.pfnComboBoxWndProc = wp(ComboWndProcW);
    pfnClientA.pfnComboListBoxProc = wp(ListBoxWndProcA);
    pfnClientW.pfnComboListBoxProc = wp(ListBoxWndProcW);
    pfnClientA.pfnDialogWndProc = wp(DefDlgProcA);
    pfnClientW.pfnDialogWndProc = wp(DefDlgProcW);
    pfnClientA.pfnEditWndProc = wp(EditWndProcA);
    pfnClientW.pfnEditWndProc = wp(EditWndProcW);
    pfnClientA.pfnListBoxWndProc = wp(ListBoxWndProcA);
    pfnClientW.pfnListBoxWndProc = wp(ListBoxWndProcW);
    pfnClientA.pfnMDIClientWndProc = wp(MDIClientWndProcA);
    pfnClientW.pfnMDIClientWndProc = wp(MDIClientWndProcW);
    pfnClientA.pfnStaticWndProc = wp(StaticWndProcA);
    pfnClientW.pfnStaticWndProc = wp(StaticWndProcW);
    pfnClientA.pfnImeWndProc = wp(ImeWndProcA);
    pfnClientW.pfnImeWndProc = wp(ImeWndProcW);
    pfnClientA.pfnGhostWndProc = wp(GhostWndProcA);
    pfnClientW.pfnGhostWndProc = wp(GhostWndProcW);
    pfnClientA.pfnHkINLPCWPSTRUCT = wp(DefWindowProcA);
    pfnClientW.pfnHkINLPCWPSTRUCT = wp(DefWindowProcW);
    pfnClientA.pfnHkINLPCWPRETSTRUCT = wp(DefWindowProcA);
    pfnClientW.pfnHkINLPCWPRETSTRUCT = wp(DefWindowProcW);
    pfnClientA.pfnDispatchHook = wp(DefWindowProcA);
    pfnClientW.pfnDispatchHook = wp(DefWindowProcW);
    pfnClientA.pfnDispatchDefWindowProc = wp(DefWindowProcA);
    pfnClientW.pfnDispatchDefWindowProc = wp(DefWindowProcW);
    pfnClientA.pfnDispatchMessage = wp(DefWindowProcA);
    pfnClientW.pfnDispatchMessage = wp(DefWindowProcW);
    pfnClientA.pfnMDIActivateDlgProc = wp(DefWindowProcA);
    pfnClientW.pfnMDIActivateDlgProc = wp(DefWindowProcW);

    pfnClientWorker.pfnButtonWndProc = wpx(ButtonWndProc_common);
    pfnClientWorker.pfnComboBoxWndProc = wpx(ComboWndProc_common);
    pfnClientWorker.pfnComboListBoxProc = wpx(ListBoxWndProc_common);
    pfnClientWorker.pfnDialogWndProc = wpx(DialogWndProc_common);
    pfnClientWorker.pfnEditWndProc = wpx(EditWndProc_common);
    pfnClientWorker.pfnListBoxWndProc = wpx(ListBoxWndProc_common);
    pfnClientWorker.pfnMDIClientWndProc = wpx(MDIClientWndProc_common);
    pfnClientWorker.pfnStaticWndProc = wpx(StaticWndProc_common);
    pfnClientWorker.pfnImeWndProc = wpx(ImeWndProc_common);
    pfnClientWorker.pfnGhostWndProc = wpx(GhostWndProc_common);
    pfnClientWorker.pfnCtfHookProc = wpx(User32DefWindowProc);

    Status = NtUserInitializeClientPfnArrays(
        core::ptr::addr_of_mut!(pfnClientA),
        core::ptr::addr_of_mut!(pfnClientW),
        core::ptr::addr_of_mut!(pfnClientWorker),
        User32Instance,
    );

    if NT_SUCCESS(Status) { 1 } else { 0 }
}
