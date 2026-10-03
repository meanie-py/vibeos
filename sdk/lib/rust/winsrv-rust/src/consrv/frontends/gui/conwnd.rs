use winsrv_sys::nt::*;
use winsrv_sys::winsrv::*;
use winsrv_sys::consts::*;
use super::helpers::*;
use super::guiterm::*;
use super::fullscreen::*;
use super::text::*;
use super::graphics::*;
use super::guisettings::*;
use super::super::super::condrv::text::ConioCoordToPointer;
use super::super::input::ConioProcessKey;
use super::super::super::coninput::ConioProcessInputEvent;
use super::super::super::{wcslen, wcscat, wcschr, wcsncpy, memcmp};

// ---- win32ss/user/winsrv/concfg/settings.h: #define WM_SETCONSOLEINFO (WM_USER + 201),
// and winuser.h: #define WM_USER 1024. Private window messages live in a project header
// no island covers, so the arm was a catch-all over every window message. ----
const _WM_SETCONSOLEINFO: u32 = 1024 + 201;

// w! — NUL-terminated UTF-16 array from an ASCII literal (2-aligned 'static;
// a b"...\0" byte string cast to *const u16 is both ANSI-not-wide and
// potentially odd-aligned — see field manual / basesrv init.rs CSDVersion).
macro_rules! w {
    ($s:literal) => {{
        const __W: &[u16] = &{
            const BYTES: &[u8] = $s.as_bytes();
            let mut out = [0u16; { BYTES.len() + 1 }];
            let mut i = 0;
            while i < BYTES.len() {
                out[i] = BYTES[i] as u16;
                i += 1;
            }
            out
        };
        __W
    }};
}


// `ConioPause`/`ConioUnpause` are plain-cdecl (no NTAPI) in console.c
// (now the Rust consrv::console) → extern "C", UCHAR Flags.
extern "C" {
    pub fn ConioPause(Console: PCONSRV_CONSOLE, Flags: u8);
    pub fn ConioUnpause(Console: PCONSRV_CONSOLE, Flags: u8);
}

// Local extern fn block — kept lean because the bulk of Win32 +
// consrv-internal fns now resolve via use winsrv_sys::winsrv::*;
// (post-regen) + super::*; sibling imports + super::helpers::*.
// condrv flip: TermResizeTerminal is term.rs's vtbl dispatcher, exported cdecl
// `_TermResizeTerminal` (abstract PCONSOLE arg; cast at call sites).
extern "C" {
    pub fn TermResizeTerminal(Console: *mut core::ffi::c_void);
}
extern "system" {
    /// `ConsoleAllocHeap`/`ConsoleFreeHeap` live in consrv/heap.c; not
    /// in the bindgen allowlist (no real "outer ring" prefix).
    pub fn ConsoleAllocHeap(Flags: u32, Size: usize) -> *mut core::ffi::c_void;
    pub fn ConsoleFreeHeap(HeapBase: winsrv_sys::winsrv::PVOID) -> winsrv_sys::winsrv::BOOLEAN; // def returns BOOLEAN
    /// `GetType` — sibling C-side helper in conoutput.c that returns
    /// the buffer-type discriminant (TEXTMODE_BUFFER vs GRAPHICS_BUFFER).
    pub fn GetType(Buffer: PCONSOLE_SCREEN_BUFFER) -> u32;
    // GetScreenBufferSizeUnits is now a plain Rust helper in super::helpers
    // (FORCEINLINE in guiterm.h), resolved via `use super::helpers::*`.
    /// `CON_SET_OUTPUT_CP` — code-page helper macro lowered to fn.
    pub fn CON_SET_OUTPUT_CP(Console: PCONSRV_CONSOLE, CodePage: UINT);
    /// `GetConsoleInputBufferMode` — consrv/coninput.c.
    #[link_name = "GetConsoleInputBufferMode"]
    fn GetConsoleInputBufferMode__c(Console: PCONSRV_CONSOLE) -> u16;
}

// consrv/mod.rs returns InputBuffer.Mode, a USHORT; a u32 decl read 16 bits nobody wrote.
#[allow(non_snake_case)]
unsafe fn GetConsoleInputBufferMode(Console: PCONSRV_CONSOLE) -> u32 {
    GetConsoleInputBufferMode__c(Console) as u32
}

// DPRINT/DPRINT1 are debug macros — translate to no-ops here so call
// sites can keep their `DPRINT1("...", ...)` syntax (variadic).
//
// `NDEBUG` silences DPRINT only — DPRINT1 sits OUTSIDE that guard in
// sdk/include/reactos/debug.h and is live under DBG, which our media always is.
// ntdll exports DbgPrint; file!()/line!() fold in via concat! the way the C's
// __RELFILE__/__LINE__ varargs do.
extern "C" {
    fn DbgPrint(Format: *const core::ffi::c_char, ...) -> u32;
}
macro_rules! DPRINT  { ($($t:tt)*) => { () } }
macro_rules! DPRINT1 {
    ($fmt:literal $(, $arg:expr)* $(,)?) => {
        unsafe {
            DbgPrint(
                concat!("(", file!(), ":", line!(), ") ", $fmt, "\0").as_ptr() as *const _
                $(, $arg)*,
            )
        }
    };
}

// `max(a, b)`/`min(a, b)` — C `MAX/MIN` macros, polymorphic. Use Rust's
// generic Ord-based versions so we don't need monomorphic ULONG vs i16
// duplicates.
#[inline] pub fn max<T: Ord>(a: T, b: T) -> T { if a >= b { a } else { b } }
#[inline] pub fn min<T: Ord>(a: T, b: T) -> T { if a <= b { a } else { b } }

// ---- file-scope preamble (translated) ----
pub const CONGUI_MIN_WIDTH: i32 = 10;

pub const CONGUI_MIN_HEIGHT: i32 = 10;

pub const CONGUI_UPDATE_TIME: u32 = 0;

pub const CONGUI_UPDATE_TIMER: u32 = 1;

pub const CURSOR_BLINK_TIME: u32 = 500;

pub const GWLP_CONWND_ALLOC: isize = (2 * core::mem::size_of::<isize>()) as isize;

pub const GWLP_CONSOLE_LEADER_PID: i32 = 0;

pub const GWLP_CONSOLE_LEADER_TID: i32 = 4;

pub static mut ghDefaultIcon: HICON = core::ptr::null_mut();
pub static mut ghDefaultIconSm: HICON = core::ptr::null_mut();
pub static mut ghDefaultCursor: HCURSOR = core::ptr::null_mut();

#[repr(C)]
pub struct _GUICONSOLE_MENUITEM {
    pub uID: UINT,
    pub SubMenu: *const _GUICONSOLE_MENUITEM,
    pub wCmdID: WORD,
}
pub type GUICONSOLE_MENUITEM = _GUICONSOLE_MENUITEM;
pub type PGUICONSOLE_MENUITEM = *mut _GUICONSOLE_MENUITEM;

pub static mut GuiConsoleEditMenuItems: [GUICONSOLE_MENUITEM; 7] = [
    GUICONSOLE_MENUITEM { uID: IDS_MARK, SubMenu: core::ptr::null(), wCmdID: ID_SYSTEM_EDIT_MARK as u16 },
    GUICONSOLE_MENUITEM { uID: IDS_COPY, SubMenu: core::ptr::null(), wCmdID: ID_SYSTEM_EDIT_COPY as u16 },
    GUICONSOLE_MENUITEM { uID: IDS_PASTE, SubMenu: core::ptr::null(), wCmdID: ID_SYSTEM_EDIT_PASTE as u16 },
    GUICONSOLE_MENUITEM { uID: IDS_SELECTALL, SubMenu: core::ptr::null(), wCmdID: ID_SYSTEM_EDIT_SELECTALL as u16 },
    GUICONSOLE_MENUITEM { uID: IDS_SCROLL, SubMenu: core::ptr::null(), wCmdID: ID_SYSTEM_EDIT_SCROLL as u16 },
    GUICONSOLE_MENUITEM { uID: IDS_FIND, SubMenu: core::ptr::null(), wCmdID: ID_SYSTEM_EDIT_FIND as u16 },
    GUICONSOLE_MENUITEM { uID: 0, SubMenu: core::ptr::null(), wCmdID: 0 },
];

pub static mut GuiConsoleMainMenuItems: [GUICONSOLE_MENUITEM; 4] = [
    GUICONSOLE_MENUITEM { uID: IDS_EDIT, SubMenu: unsafe { GuiConsoleEditMenuItems.as_ptr() }, wCmdID: 0 },
    GUICONSOLE_MENUITEM { uID: IDS_DEFAULTS, SubMenu: core::ptr::null(), wCmdID: ID_SYSTEM_DEFAULTS as u16 },
    GUICONSOLE_MENUITEM { uID: IDS_PROPERTIES, SubMenu: core::ptr::null(), wCmdID: ID_SYSTEM_PROPERTIES as u16 },
    GUICONSOLE_MENUITEM { uID: 0, SubMenu: core::ptr::null(), wCmdID: 0 },
];

// `const COLORREF s_Colors[16]` in conwnd.c — a real static array with
// external linkage (settings.rs reads it via `extern "C" { static s_Colors }`).
// Must be a #[no_mangle] static (a `const` is inlined and exports no symbol).
#[no_mangle]
pub static s_Colors: [COLORREF; 16] = [
    0x00000000,
    0x00800000,
    0x00008000,
    0x00808000,
    0x00000080,
    0x00800080,
    0x00008080,
    0x00c0c0c0,
    0x00808080,
    0x00ff0000,
    0x0000ff00,
    0x00ffff00,
    0x000000ff,
    0x00ff00ff,
    0x0000ffff,
    0x00ffffff,
];

pub unsafe fn SetConWndConsoleLeaderCID(GuiData: PGUI_CONSOLE_DATA) {
    let ProcessData: PCONSOLE_PROCESS_DATA;

    ProcessData = ConSrvGetConsoleLeaderProcess((*GuiData).Console);

    if !(*ProcessData).Process.is_null() {
        let ConsoleLeaderCID: _CLIENT_ID = (*(*ProcessData).Process).ClientId;
        SetWindowLongPtrW((*GuiData).hWindow, GWLP_CONSOLE_LEADER_PID,
                          ConsoleLeaderCID.UniqueProcess as LONG_PTR);
        SetWindowLongPtrW((*GuiData).hWindow, GWLP_CONSOLE_LEADER_TID,
                          ConsoleLeaderCID.UniqueThread as LONG_PTR);
    } else {
        SetWindowLongPtrW((*GuiData).hWindow, GWLP_CONSOLE_LEADER_PID, 0);
        SetWindowLongPtrW((*GuiData).hWindow, GWLP_CONSOLE_LEADER_TID, 0);
    }
}

pub unsafe fn RegisterConWndClass(hInstance: HINSTANCE) -> u8 {
    let mut WndClass: WNDCLASSEXW = core::mem::zeroed();
    let WndClassAtom: ATOM;

    ghDefaultIcon = LoadImageW(
        hInstance,
        MAKEINTRESOURCEW(IDI_TERMINAL),
        IMAGE_ICON,
        GetSystemMetrics(SM_CXICON as i32),
        GetSystemMetrics(SM_CYICON as i32),
        LR_SHARED,
    ) as HICON;
    ghDefaultIconSm = LoadImageW(
        hInstance,
        MAKEINTRESOURCEW(IDI_TERMINAL),
        IMAGE_ICON,
        GetSystemMetrics(SM_CXSMICON as i32),
        GetSystemMetrics(SM_CYSMICON as i32),
        LR_SHARED,
    ) as HICON;
    ghDefaultCursor = LoadCursorW(core::ptr::null_mut(), IDC_ARROW);

    WndClass.cbSize = core::mem::size_of::<WNDCLASSEXW>() as u32;
    WndClass.lpszClassName = GUI_CONWND_CLASS_W.as_ptr();
    WndClass.lpfnWndProc = Some(core::mem::transmute(ConWndProc as *const ()));
    WndClass.style = CS_DBLCLKS;
    WndClass.hInstance = hInstance;
    WndClass.hIcon = ghDefaultIcon;
    WndClass.hIconSm = ghDefaultIconSm;
    WndClass.hCursor = ghDefaultCursor;
    WndClass.hbrBackground = core::ptr::null_mut();
    WndClass.lpszMenuName = core::ptr::null();
    WndClass.cbClsExtra = 0;
    WndClass.cbWndExtra = GWLP_CONWND_ALLOC as i32;

    WndClassAtom = RegisterClassExW(&WndClass);
    if WndClassAtom == 0 {
    } else {
        NtUserConsoleControl(
            GuiConsoleWndClassAtom,
            &WndClassAtom as *const ATOM as *mut core::ffi::c_void,
            core::mem::size_of::<ATOM>() as u32,
        );
    }

    if WndClassAtom != 0 { 1u8 } else { 0u8 }
}

pub unsafe fn UnRegisterConWndClass(hInstance: HINSTANCE) -> u8 {
    if UnregisterClassW(w!("ConWndClass").as_ptr() as *const u16, hInstance) != 0 {
        1u8
    } else {
        0u8
    }
}

unsafe fn AppendMenuItems(hMenu: HMENU, Items: *const GUICONSOLE_MENUITEM) {
    let mut i: u32 = 0;
    let mut szMenuString: [u16; 255] = [0; 255];
    let mut hSubMenu: HMENU;

    loop {
        if (*Items.offset(i as isize)).uID != u32::MAX {
            if LoadStringW(
                ConSrvDllInstance,
                (*Items.offset(i as isize)).uID,
                szMenuString.as_mut_ptr(),
                255,
            ) > 0
            {
                if (*Items.offset(i as isize)).SubMenu != core::ptr::null() {
                    hSubMenu = CreatePopupMenu();
                    if hSubMenu != core::ptr::null_mut() {
                        AppendMenuItems(hSubMenu, (*Items.offset(i as isize)).SubMenu);

                        if AppendMenuW(
                            hMenu,
                            MF_STRING | MF_POPUP,
                            hSubMenu as UINT_PTR,
                            szMenuString.as_ptr(),
                        ) == 0
                        {
                            DestroyMenu(hSubMenu);
                        }
                    }
                } else {
                    AppendMenuW(
                        hMenu,
                        MF_STRING,
                        (*Items.offset(i as isize)).wCmdID as UINT_PTR,
                        szMenuString.as_ptr(),
                    );
                }
            }
        } else {
            AppendMenuW(hMenu, MF_SEPARATOR, 0, core::ptr::null());
        }
        i += 1;

        if (*Items.offset(i as isize)).uID == 0
            && (*Items.offset(i as isize)).SubMenu == core::ptr::null()
            && (*Items.offset(i as isize)).wCmdID == 0
        {
            break;
        }
    }
}

pub unsafe fn CreateSysMenu(hWnd: HWND) {
    let mut mii: MENUITEMINFOW = core::mem::zeroed();
    let hMenu: HMENU;
    let ptrTab: *mut u16;
    let mut szMenuStringBack: [u16; 255] = [0; 255];

    hMenu = GetSystemMenu(hWnd, 0);
    if hMenu.is_null() {
        return;
    }

    mii.cbSize = core::mem::size_of::<MENUITEMINFOW>() as u32;
    mii.fMask = MIIM_STRING;
    mii.dwTypeData = szMenuStringBack.as_mut_ptr();
    mii.cch = 255;

    GetMenuItemInfoW(hMenu, SC_CLOSE, 0, &mut mii);

    ptrTab = wcschr(szMenuStringBack.as_ptr(), 0x0009u16);
    if !ptrTab.is_null() {
        *ptrTab = 0x0000u16;
        mii.cch = wcslen(szMenuStringBack.as_ptr()) as u32;

        SetMenuItemInfoW(hMenu, SC_CLOSE, 0, &mii);
    }

    AppendMenuItems(hMenu, GuiConsoleMainMenuItems.as_ptr());
    DrawMenuBar(hWnd);
}

unsafe fn SendMenuEvent(Console: PCONSRV_CONSOLE, CmdId: u32) {
    let mut er: INPUT_RECORD = core::mem::zeroed();

    if ConDrvValidateConsoleUnsafe(Console as PCONSOLE, CONSOLE_RUNNING, 1u8) == 0 {
        return;
    }

    er.EventType = MENU_EVENT as u16;
    er.Event.MenuEvent.dwCommandId = CmdId;
    ConioProcessInputEvent(Console, &mut er);

    LeaveCriticalSection(core::ptr::addr_of_mut!((*Console).Lock));
}

unsafe fn Mark(GuiData: PGUI_CONSOLE_DATA) {
    let ActiveBuffer: PCONSOLE_SCREEN_BUFFER = (*GuiData).ActiveBuffer;

    (*GuiData).Selection.dwFlags = CONSOLE_NO_SELECTION;

    (*GuiData).dwSelectionCursor = (*ActiveBuffer).ViewOrigin;
    UpdateSelection(
        GuiData,
        core::ptr::addr_of_mut!((*GuiData).dwSelectionCursor),
        core::ptr::addr_of_mut!((*GuiData).dwSelectionCursor),
    );
}

unsafe fn SelectAll(GuiData: PGUI_CONSOLE_DATA) {
    let ActiveBuffer: PCONSOLE_SCREEN_BUFFER = (*GuiData).ActiveBuffer;
    let mut SelectionAnchor: COORD = core::mem::zeroed();

    (*GuiData).Selection.dwFlags = CONSOLE_NO_SELECTION;

    SelectionAnchor.X = 0;
    SelectionAnchor.Y = 0;
    (*GuiData).dwSelectionCursor.X = (*ActiveBuffer).ScreenBufferSize.X - 1;

    if GetType(ActiveBuffer) == TEXTMODE_BUFFER {
        (*GuiData).dwSelectionCursor.Y = (*ActiveBuffer).CursorPosition.Y;
    } else {
        (*GuiData).dwSelectionCursor.Y = (*ActiveBuffer).ScreenBufferSize.Y - 1;
    }

    (*GuiData).Selection.dwFlags |= CONSOLE_MOUSE_SELECTION;
    UpdateSelection(GuiData, core::ptr::addr_of_mut!(SelectionAnchor), core::ptr::addr_of_mut!((*GuiData).dwSelectionCursor));
}

unsafe fn OnCommand(GuiData: PGUI_CONSOLE_DATA, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    let mut Ret: LRESULT = 1;
    let Console: PCONSRV_CONSOLE = (*GuiData).Console;

    'Quit: {
        if (*GuiData).CmdIdLow <= wParam as u32 && (wParam as u32) <= (*GuiData).CmdIdHigh {
            SendMenuEvent(Console, wParam as u32);
            break 'Quit;
        }

        match wParam as u32 {
            ID_SYSTEM_EDIT_MARK => {
                Mark(GuiData);
            }
            ID_SYSTEM_EDIT_COPY => {
                Copy(GuiData);
            }
            ID_SYSTEM_EDIT_PASTE => {
                Paste(GuiData);
            }
            ID_SYSTEM_EDIT_SELECTALL => {
                SelectAll(GuiData);
            }
            ID_SYSTEM_EDIT_SCROLL => {
            }
            ID_SYSTEM_EDIT_FIND => {
            }
            ID_SYSTEM_DEFAULTS => {
                GuiConsoleShowConsoleProperties(GuiData, 1u8);
            }
            ID_SYSTEM_PROPERTIES => {
                GuiConsoleShowConsoleProperties(GuiData, 0u8);
            }
            _ => {
                Ret = 0;
            }
        }
    }

    if Ret == 0 {
        Ret = DefWindowProcW((*GuiData).hWindow, WM_SYSCOMMAND, wParam, lParam);
    }

    return Ret;
}

unsafe fn GuiGetGuiData(hWnd: HWND) -> PGUI_CONSOLE_DATA {
    let GuiData = GetWindowLongPtrW(hWnd, GWLP_USERDATA) as PGUI_CONSOLE_DATA;
    if GuiData.is_null() || ((*GuiData).hWindow == hWnd && !(*GuiData).Console.is_null()) {
        GuiData
    } else {
        core::ptr::null_mut()
    }
}

pub unsafe fn ResizeConWnd(GuiData: PGUI_CONSOLE_DATA, WidthUnit: DWORD, HeightUnit: DWORD) {
    let Buff: PCONSOLE_SCREEN_BUFFER = (*GuiData).ActiveBuffer;
    let mut sInfo: SCROLLINFO = core::mem::zeroed();

    let mut Width: DWORD;
    let mut Height: DWORD;

    Width = (*Buff).ViewSize.X as DWORD * WidthUnit +
            2 * (GetSystemMetrics(SM_CXFRAME as i32) as DWORD + GetSystemMetrics(SM_CXEDGE as i32) as DWORD);
    Height = (*Buff).ViewSize.Y as DWORD * HeightUnit +
             2 * (GetSystemMetrics(SM_CYFRAME as i32) as DWORD + GetSystemMetrics(SM_CYEDGE as i32) as DWORD) + GetSystemMetrics(SM_CYCAPTION as i32) as DWORD;

    sInfo.cbSize = core::mem::size_of::<SCROLLINFO>() as UINT;
    sInfo.fMask = SIF_RANGE | SIF_PAGE | SIF_POS;
    sInfo.nMin = 0;
    if (*Buff).ScreenBufferSize.Y > (*Buff).ViewSize.Y {
        sInfo.nMax = (*Buff).ScreenBufferSize.Y as i32 - 1;
        sInfo.nPage = (*Buff).ViewSize.Y as UINT;
        sInfo.nPos = (*Buff).ViewOrigin.Y as i32;
        SetScrollInfo((*GuiData).hWindow, SB_VERT as i32, core::ptr::addr_of_mut!(sInfo), 1);
        Width += GetSystemMetrics(SM_CXVSCROLL as i32) as DWORD;
        ShowScrollBar((*GuiData).hWindow, SB_VERT as i32, 1);
    } else {
        ShowScrollBar((*GuiData).hWindow, SB_VERT as i32, 0);
    }

    if (*Buff).ScreenBufferSize.X > (*Buff).ViewSize.X {
        sInfo.nMax = (*Buff).ScreenBufferSize.X as i32 - 1;
        sInfo.nPage = (*Buff).ViewSize.X as UINT;
        sInfo.nPos = (*Buff).ViewOrigin.X as i32;
        SetScrollInfo((*GuiData).hWindow, SB_HORZ as i32, core::ptr::addr_of_mut!(sInfo), 1);
        Height += GetSystemMetrics(SM_CYHSCROLL as i32) as DWORD;
        ShowScrollBar((*GuiData).hWindow, SB_HORZ as i32, 1);
    } else {
        ShowScrollBar((*GuiData).hWindow, SB_HORZ as i32, 0);
    }

    SetWindowPos((*GuiData).hWindow, core::ptr::null_mut(), 0, 0, Width as i32, Height as i32,
                 SWP_NOZORDER | SWP_NOMOVE | SWP_NOACTIVATE | SWP_NOCOPYBITS);
}

pub unsafe fn DeleteFonts(GuiData: PGUI_CONSOLE_DATA) {
    let mut i: ULONG = 0;
    while i < ((*GuiData).Font.len() as ULONG) {
        if (*GuiData).Font[i as usize] != core::ptr::null_mut() {
            DeleteObject((*GuiData).Font[i as usize] as *mut core::ffi::c_void);
        }
        (*GuiData).Font[i as usize] = core::ptr::null_mut();
        i += 1;
    }
}

unsafe fn CreateDerivedFont(
    OrgFont: HFONT,
    FontWeight: ULONG,
    bUnderline: BOOLEAN,
    bStrikeOut: BOOLEAN,
) -> HFONT {
    let mut lf: LOGFONTW = core::mem::zeroed();

    if GetObjectW(OrgFont as *mut core::ffi::c_void, core::mem::size_of::<LOGFONTW>() as i32, &mut lf as *mut _ as *mut core::ffi::c_void) == 0 {
        return core::ptr::null_mut();
    }

    lf.lfWeight = FontWeight as i32;
    lf.lfUnderline = bUnderline;
    lf.lfStrikeOut = bStrikeOut;

    CreateFontIndirectW(&lf)
}

pub unsafe fn InitFonts(
    GuiData: PGUI_CONSOLE_DATA,
    FaceName: PCWSTR,
    FontWeight: ULONG,
    FontFamily: ULONG,
    FontSize: COORD,
    CodePage: UINT,
    UseDefaultFallback: BOOL,
) -> BOOL {
    
    const ARRAYSIZE_LF_FACESIZE: usize = LF_FACESIZE as usize;
    
    let hDC: HDC;
    let hFont: HFONT;
    let mut FontData: FONT_DATA = core::mem::zeroed();
    let OldCharWidth: UINT = (*GuiData).CharWidth;
    let OldCharHeight: UINT = (*GuiData).CharHeight;
    let OldFontSize: COORD = (*GuiData).GuiInfo.FontSize;
    let mut NewFaceName: [u16; LF_FACESIZE as usize] = [0; LF_FACESIZE as usize];

    let mut CodePage = CodePage;
    if CodePage == 0 {
        CodePage = (*(*GuiData).Console).OutputCodePage;
    }

    StringCchCopyNW(
        NewFaceName.as_mut_ptr(),
        ARRAYSIZE_LF_FACESIZE,
        FaceName,
        LF_FACESIZE as usize,
    );

    hFont = CreateConsoleFontEx(
        FontSize.Y as LONG,
        FontSize.X as LONG,
        NewFaceName.as_mut_ptr(),
        FontWeight,
        FontFamily,
        CodePage,
        UseDefaultFallback,
        &mut FontData,
    );
    if hFont == core::ptr::null_mut() {
        DPRINT1!("InitFonts: CreateConsoleFontEx('%S') failed\n", NewFaceName.as_ptr());
        return 0;
    }

    hDC = GetDC((*GuiData).hWindow);
    if GetFontCellSize(
        hDC,
        hFont,
        &mut (*GuiData).CharHeight,
        &mut (*GuiData).CharWidth,
    ) == 0
    {
        DPRINT1!("InitFonts: GetFontCellSize failed\n");
        ReleaseDC((*GuiData).hWindow, hDC);
        DeleteObject(hFont as *mut core::ffi::c_void);
        return 0;
    }
    ReleaseDC((*GuiData).hWindow, hDC);

    DeleteFonts(GuiData);
    (*GuiData).Font[0] = hFont;

    (*GuiData).Font[1] = CreateDerivedFont(
        (*GuiData).Font[0],
        max(FW_BOLD, FontData.Weight),
        0u8,
        0u8,
    );
    (*GuiData).Font[2] = CreateDerivedFont(
        (*GuiData).Font[0],
        FontData.Weight,
        1u8,
        0u8,
    );
    (*GuiData).Font[3] = CreateDerivedFont(
        (*GuiData).Font[0],
        max(FW_BOLD, FontData.Weight),
        1u8,
        0u8,
    );

    StringCchCopyNW(
        (*GuiData).GuiInfo.FaceName.as_mut_ptr(),
        ARRAYSIZE_LF_FACESIZE,
        NewFaceName.as_ptr(),
        ARRAYSIZE_LF_FACESIZE,
    );
    (*GuiData).GuiInfo.FontWeight = FontData.Weight;
    (*GuiData).GuiInfo.FontFamily = FontData.Family;
    (*GuiData).GuiInfo.FontSize = FontData.Size;

    if (OldCharWidth != (*GuiData).CharWidth)
        || (OldCharHeight != (*GuiData).CharHeight)
        || (OldFontSize.X != FontData.Size.X || OldFontSize.Y != FontData.Size.Y)
    {
        TermResizeTerminal((*GuiData).Console as *mut core::ffi::c_void);
    }

    1
}

unsafe fn OnNcCreate(hWnd: HWND, Create: LPCREATESTRUCTW) -> BOOL {

    let GuiData = (*Create).lpCreateParams as PGUI_CONSOLE_DATA;

    if GuiData.is_null() {
        DPRINT1!("GuiConsoleNcCreate: No GUI data\n");
        return 0;
    }

    let Console = (*GuiData).Console;

    (*GuiData).hWindow = hWnd;
    (*GuiData).hSysMenu = GetSystemMenu(hWnd, 0);
    (*GuiData).IsWindowActive = 0;

    if InitFonts(
        GuiData,
        (*GuiData).GuiInfo.FaceName.as_ptr(),
        (*GuiData).GuiInfo.FontWeight,
        (*GuiData).GuiInfo.FontFamily,
        (*GuiData).GuiInfo.FontSize,
        0,
        0,
    ) == 0
    {
        let mut AltCodePage = GetOEMCP();

        if AltCodePage == (*Console).OutputCodePage {
            AltCodePage = CP_USA;
        }

        DPRINT1!("Could not initialize font '%S' for code page %d - Resetting CP to %d\n",
                 (*GuiData).GuiInfo.FaceName.as_ptr(), (*Console).OutputCodePage, AltCodePage);

        CON_SET_OUTPUT_CP(Console, AltCodePage);

        if InitFonts(
            GuiData,
            (*GuiData).GuiInfo.FaceName.as_ptr(),
            (*GuiData).GuiInfo.FontWeight,
            (*GuiData).GuiInfo.FontFamily,
            (*GuiData).GuiInfo.FontSize,
            0,
            1,
        ) == 0
        {
            DPRINT1!("Failed to initialize font '%S' for code page %d\n",
                     (*GuiData).GuiInfo.FaceName.as_ptr(), (*Console).OutputCodePage);

            DPRINT1!("GuiConsoleNcCreate: InitFonts failed\n");
            (*GuiData).hWindow = core::ptr::null_mut();
            NtSetEvent((*GuiData).hGuiInitEvent, core::ptr::null_mut());
            return 0;
        }
    }

    (*GuiData).hMemDC = CreateCompatibleDC(core::ptr::null_mut());
    (*GuiData).hBitmap = core::ptr::null_mut();
    (*GuiData).hSysPalette = core::ptr::null_mut();

    if (*GuiData).hIcon != ghDefaultIcon {
        DefWindowProcW((*GuiData).hWindow, WM_SETICON, ICON_BIG as WPARAM, (*GuiData).hIcon as LPARAM);
        DefWindowProcW((*GuiData).hWindow, WM_SETICON, ICON_SMALL as WPARAM, (*GuiData).hIconSm as LPARAM);
    }

    (*(*Console).ActiveBuffer).CursorBlinkOn = 1;
    (*(*Console).ActiveBuffer).ForceCursorOff = 0;

    SetWindowLongPtrW((*GuiData).hWindow, GWLP_USERDATA, GuiData as LONG_PTR);

    if (*GuiData).IsWindowVisible != 0 {
        SetTimer((*GuiData).hWindow, CONGUI_UPDATE_TIMER as UINT_PTR, CONGUI_UPDATE_TIME, None);
    }

    // DPRINT removed
    NtSetEvent((*GuiData).hGuiInitEvent, core::ptr::null_mut());

    DragAcceptFiles((*GuiData).hWindow, 1);

    DefWindowProcW((*GuiData).hWindow, WM_NCCREATE, 0, Create as LPARAM) as BOOL
}

unsafe fn OnActivate(GuiData: PGUI_CONSOLE_DATA, wParam: WPARAM) {

    let ActivationState: u16 = LOWORD(wParam) as u16;

    (*GuiData).IsWindowActive = (ActivationState != WA_INACTIVE as u16) as u8;

    if ActivationState == WA_ACTIVE as u16 ||
       ActivationState == WA_CLICKACTIVE as u16
    {
        if (*GuiData).GuiInfo.FullScreen != 0 {
            EnterFullScreen(GuiData);
        }
    }
    else
    {
        if (*GuiData).GuiInfo.FullScreen != 0 {
            SendMessageW((*GuiData).hWindow, WM_SYSCOMMAND, SC_MINIMIZE as WPARAM, 0);
            LeaveFullScreen(GuiData);
        }
    }

    if ActivationState == WA_CLICKACTIVE as u16 {
        (*GuiData).IgnoreNextMouseEvent = 1;
    }
}

unsafe fn OnFocus(GuiData: PGUI_CONSOLE_DATA, SetFocus: BOOL) {
    let Console: PCONSRV_CONSOLE = (*GuiData).Console;
    let mut er: INPUT_RECORD = core::mem::zeroed();

    if ConDrvValidateConsoleUnsafe(Console as PCONSOLE, CONSOLE_RUNNING, 1u8) == 0 {
        return;
    }

    (*Console).HasFocus = SetFocus as u8;

    ConSrvSetConsoleProcessFocus(Console, SetFocus as u8);

    er.EventType = FOCUS_EVENT as u16;
    (*(&mut er.Event.FocusEvent)).bSetFocus = SetFocus;
    ConioProcessInputEvent(Console, &mut er);

    LeaveCriticalSection(&mut (*Console).Lock);

    if SetFocus != 0 {
    } else {
    }
}

// Promoted #[no_mangle] extern "system" so text.rs (now Rust) can extern-link
// it across the gui/* modules (#36 frontends flip).
#[no_mangle]
pub unsafe extern "system" fn GetSelectionBeginEnd(
    Begin: PCOORD,
    End: PCOORD,
    SelectionAnchor: PCOORD,
    SmallRect: PSMALL_RECT,
) {
    if Begin.is_null() || End.is_null() {
        return;
    }

    *Begin = *SelectionAnchor;
    (*End).X = if (*SelectionAnchor).X == (*SmallRect).Left {
        (*SmallRect).Right
    } else {
        (*SmallRect).Left
    };
    (*End).Y = if (*SelectionAnchor).Y == (*SmallRect).Top {
        (*SmallRect).Bottom
    } else {
        (*SmallRect).Top
    };

    if (*Begin).Y > (*End).Y || ((*Begin).Y == (*End).Y && (*Begin).X > (*End).X) {
        (*End).X = atomic_xchg_seqcst(core::ptr::addr_of_mut!((*Begin).X), (*End).X);
        (*End).Y = atomic_xchg_seqcst(core::ptr::addr_of_mut!((*Begin).Y), (*End).Y);
    }
}

unsafe fn CreateSelectionRgn(
    GuiData: PGUI_CONSOLE_DATA,
    LineSelection: BOOL,
    SelectionAnchor: PCOORD,
    SmallRect: PSMALL_RECT,
) -> HRGN {
    if LineSelection == 0 {
        let mut rect: RECT = core::mem::zeroed();
        SmallRectToRect(GuiData, &mut rect, SmallRect);
        return CreateRectRgnIndirect(&rect);
    } else {
        let SelRgn: HRGN;
        let mut Begin: COORD = core::mem::zeroed();
        let mut End: COORD = core::mem::zeroed();

        GetSelectionBeginEnd(&mut Begin, &mut End, SelectionAnchor, SmallRect);

        if Begin.Y == End.Y {
            let mut sr: SMALL_RECT = core::mem::zeroed();
            let mut r: RECT = core::mem::zeroed();

            sr.Left = Begin.X;
            sr.Top = Begin.Y;
            sr.Right = End.X;
            sr.Bottom = End.Y;

            if (*SmallRect).Left != sr.Left
                || (*SmallRect).Top != sr.Top
                || (*SmallRect).Right != sr.Right
                || (*SmallRect).Bottom != sr.Bottom
            {
                // DPRINT removed
            }

            SmallRectToRect(GuiData, &mut r, &mut sr);
            SelRgn = CreateRectRgnIndirect(&r);
        } else {
            let ActiveBuffer: PCONSOLE_SCREEN_BUFFER = (*GuiData).ActiveBuffer;

            let rg1: HRGN;
            let rg2: HRGN;
            let rg3: HRGN;
            let mut sr1: SMALL_RECT = core::mem::zeroed();
            let mut sr2: SMALL_RECT = core::mem::zeroed();
            let mut sr3: SMALL_RECT = core::mem::zeroed();
            let mut r1: RECT = core::mem::zeroed();
            let mut r2: RECT = core::mem::zeroed();
            let mut r3: RECT = core::mem::zeroed();

            sr1.Left = Begin.X;
            sr1.Top = Begin.Y;
            sr1.Right = (*ActiveBuffer).ScreenBufferSize.X - 1;
            sr1.Bottom = Begin.Y;

            sr2.Left = 0;
            sr2.Top = Begin.Y + 1;
            sr2.Right = (*ActiveBuffer).ScreenBufferSize.X - 1;
            sr2.Bottom = End.Y - 1;

            sr3.Left = 0;
            sr3.Top = End.Y;
            sr3.Right = End.X;
            sr3.Bottom = End.Y;

            SmallRectToRect(GuiData, &mut r1, &mut sr1);
            SmallRectToRect(GuiData, &mut r2, &mut sr2);
            SmallRectToRect(GuiData, &mut r3, &mut sr3);

            rg1 = CreateRectRgnIndirect(&r1);
            rg2 = CreateRectRgnIndirect(&r2);
            rg3 = CreateRectRgnIndirect(&r3);

            CombineRgn(rg1, rg1, rg2, RGN_XOR as i32);
            CombineRgn(rg1, rg1, rg3, RGN_XOR as i32);
            DeleteObject(rg3 as *mut core::ffi::c_void);
            DeleteObject(rg2 as *mut core::ffi::c_void);

            SelRgn = rg1;
        }

        return SelRgn;
    }
}

unsafe fn PaintSelectionRect(GuiData: PGUI_CONSOLE_DATA, pps: PPAINTSTRUCT) {
    let rgnPaint: HRGN = CreateRectRgnIndirect(&(*pps).rcPaint);
    let rgnSel: HRGN = CreateSelectionRgn(
        GuiData,
        (*GuiData).LineSelection,
        core::ptr::addr_of_mut!((*GuiData).Selection.dwSelectionAnchor),
        core::ptr::addr_of_mut!((*GuiData).Selection.srSelection),
    );

    let ErrorCode: i32 = CombineRgn(rgnPaint, rgnPaint, rgnSel, RGN_AND as i32);
    if ErrorCode != ERROR as i32 && ErrorCode != NULLREGION as i32 {
        InvertRgn((*pps).hdc, rgnPaint);
    }

    DeleteObject(rgnSel as *mut core::ffi::c_void);
    DeleteObject(rgnPaint as *mut core::ffi::c_void);
}

pub unsafe fn UpdateSelection(
    GuiData: PGUI_CONSOLE_DATA,
    SelectionAnchor: PCOORD,
    coord: PCOORD,
) {
    let Console: PCONSRV_CONSOLE = (*GuiData).Console;
    let oldRgn: HRGN = CreateSelectionRgn(
        GuiData,
        (*GuiData).LineSelection,
        core::ptr::addr_of_mut!((*GuiData).Selection.dwSelectionAnchor),
        core::ptr::addr_of_mut!((*GuiData).Selection.srSelection),
    );

    if !SelectionAnchor.is_null() {
        (*GuiData).Selection.dwSelectionAnchor = *SelectionAnchor;
    }

    if !coord.is_null() {
        let mut rc: SMALL_RECT = core::mem::zeroed();
        let newRgn: HRGN;

        let OldLineSel: BOOL = (*GuiData).LineSelection;
        (*GuiData).LineSelection = if (GetKeyState(VK_CONTROL as i32) & 0x8000u16 as i16) != 0 { 1 } else { 0 };

        rc.Left = min((*GuiData).Selection.dwSelectionAnchor.X, (*coord).X);
        rc.Top = min((*GuiData).Selection.dwSelectionAnchor.Y, (*coord).Y);
        rc.Right = max((*GuiData).Selection.dwSelectionAnchor.X, (*coord).X);
        rc.Bottom = max((*GuiData).Selection.dwSelectionAnchor.Y, (*coord).Y);

        newRgn = CreateSelectionRgn(
            GuiData,
            (*GuiData).LineSelection,
            core::ptr::addr_of_mut!((*GuiData).Selection.dwSelectionAnchor),
            &mut rc,
        );

        if ((*GuiData).Selection.dwFlags & CONSOLE_SELECTION_NOT_EMPTY) != 0 {
            if OldLineSel != (*GuiData).LineSelection
                || memcmp(
                    &rc as *const _ as *const core::ffi::c_void,
                    core::ptr::addr_of!((*GuiData).Selection.srSelection) as *const core::ffi::c_void,
                    core::mem::size_of::<SMALL_RECT>(),
                ) != 0
            {
                if !oldRgn.is_null() && !newRgn.is_null() && CombineRgn(newRgn, newRgn, oldRgn, RGN_XOR as i32) != ERROR as i32 {
                    InvalidateRgn((*GuiData).hWindow, newRgn, 0);
                }
            }
        } else {
            InvalidateRgn((*GuiData).hWindow, newRgn, 0);
        }

        DeleteObject(newRgn as *mut core::ffi::c_void);

        (*GuiData).Selection.dwFlags |= CONSOLE_SELECTION_NOT_EMPTY;
        (*GuiData).Selection.srSelection = rc;
        (*GuiData).dwSelectionCursor = *coord;

        if ((*GuiData).Selection.dwFlags & CONSOLE_SELECTION_IN_PROGRESS) == 0 {
            let mut SelTypeStr: LPWSTR = core::ptr::null_mut();
            let mut WindowTitle: LPWSTR = core::ptr::null_mut();
            let mut SelTypeStrLength: usize = 0;
            let mut Length: usize = 0;

            if ((*GuiData).Selection.dwFlags & CONSOLE_SELECTION_NOT_EMPTY) != 0 {
                InvalidateRgn((*GuiData).hWindow, oldRgn, 0);
            }

            SelTypeStrLength = LoadStringW(
                ConSrvDllInstance,
                if ((*GuiData).Selection.dwFlags & CONSOLE_MOUSE_SELECTION) != 0 {
                    IDS_SELECT_TITLE
                } else {
                    IDS_MARK_TITLE
                },
                core::ptr::addr_of_mut!(SelTypeStr) as LPWSTR,
                0,
            ) as usize;

            if !SelTypeStr.is_null() {
                Length = (*Console).Title.Length as usize + (SelTypeStrLength + 3 + 1) * core::mem::size_of::<u16>();
                WindowTitle = ConsoleAllocHeap(0, Length) as LPWSTR;

                wcsncpy(WindowTitle, SelTypeStr, SelTypeStrLength);
                *WindowTitle.add(SelTypeStrLength) = 0u16;
                wcscat(WindowTitle, w!("L - ").as_ptr() as *const u16);
                wcscat(WindowTitle, (*Console).Title.Buffer);

                SetWindowTextW((*GuiData).hWindow, WindowTitle);
                ConsoleFreeHeap(WindowTitle as *mut core::ffi::c_void);
            }

            (*GuiData).Selection.dwFlags |= CONSOLE_SELECTION_IN_PROGRESS;
            ConioPause(Console, PAUSED_FROM_SELECTION as u8);
        }
    } else {
        if ((*GuiData).Selection.dwFlags & CONSOLE_SELECTION_NOT_EMPTY) != 0 {
            InvalidateRgn((*GuiData).hWindow, oldRgn, 0);
        }

        (*GuiData).Selection.dwFlags = CONSOLE_NO_SELECTION;
        ConioUnpause(Console, PAUSED_FROM_SELECTION as u8);

        SetWindowTextW((*GuiData).hWindow, (*Console).Title.Buffer);
    }

    DeleteObject(oldRgn as *mut core::ffi::c_void);
}

unsafe fn OnPaint(GuiData: PGUI_CONSOLE_DATA) {

    let ActiveBuffer: PCONSOLE_SCREEN_BUFFER = (*GuiData).ActiveBuffer;
    let mut ps: PAINTSTRUCT = core::mem::zeroed();
    let mut rcPaint: RECT = core::mem::zeroed();

    if (*GuiData).IsWindowVisible == 0 {
        return;
    }

    BeginPaint((*GuiData).hWindow, &mut ps);
    if ps.hdc != core::ptr::null_mut() &&
        ps.rcPaint.left < ps.rcPaint.right &&
        ps.rcPaint.top < ps.rcPaint.bottom
    {
        EnterCriticalSection(&mut (*GuiData).Lock);

        if GetType(ActiveBuffer) == TEXTMODE_BUFFER {
            GuiPaintTextModeBuffer(
                ActiveBuffer as PTEXTMODE_SCREEN_BUFFER,
                GuiData,
                core::ptr::addr_of_mut!(ps.rcPaint),
                &mut rcPaint,
            );
        } else {
            GuiPaintGraphicsBuffer(
                ActiveBuffer as PGRAPHICS_SCREEN_BUFFER,
                GuiData,
                core::ptr::addr_of_mut!(ps.rcPaint),
                &mut rcPaint,
            );
        }

        BitBlt(
            ps.hdc,
            ps.rcPaint.left,
            ps.rcPaint.top,
            rcPaint.right - rcPaint.left,
            rcPaint.bottom - rcPaint.top,
            (*GuiData).hMemDC,
            rcPaint.left,
            rcPaint.top,
            SRCCOPY,
        );

        if ((*GuiData).Selection.dwFlags & CONSOLE_SELECTION_NOT_EMPTY) != 0 {
            PaintSelectionRect(GuiData, &mut ps);
        }

        LeaveCriticalSection(&mut (*GuiData).Lock);
    }
    EndPaint((*GuiData).hWindow, &ps);
}

unsafe fn OnPaletteChanged(GuiData: PGUI_CONSOLE_DATA) {
    let ActiveBuffer: PCONSOLE_SCREEN_BUFFER = (*GuiData).ActiveBuffer;

    if (*GuiData).IsWindowVisible == 0 {
        return;
    }

    if !(*ActiveBuffer).PaletteHandle.is_null() {
        SetSystemPaletteUse((*GuiData).hMemDC, (*ActiveBuffer).PaletteUsage);
        RealizePalette((*GuiData).hMemDC);
    }
}

unsafe fn IsSystemKey(VirtualKeyCode: WORD) -> BOOL {
    match VirtualKeyCode as u32 {
        VK_RETURN | VK_SHIFT | VK_CONTROL | VK_MENU | VK_PAUSE | VK_CAPITAL | 
        VK_ESCAPE | VK_LWIN | VK_RWIN | VK_NUMLOCK | VK_SCROLL => 1,
        _ => 0,
    }
}

unsafe fn OnKey(GuiData: PGUI_CONSOLE_DATA, msg: UINT, wParam: WPARAM, lParam: LPARAM) {
    let Console: PCONSRV_CONSOLE = (*GuiData).Console;
    let ActiveBuffer: PCONSOLE_SCREEN_BUFFER;

    if ConDrvValidateConsoleUnsafe(Console as PCONSOLE, CONSOLE_RUNNING, 1) == 0 {
        return;
    }

    ActiveBuffer = (*GuiData).ActiveBuffer;

    if ((*GuiData).Selection.dwFlags & CONSOLE_SELECTION_IN_PROGRESS) != 0 {
        let VirtualKeyCode: WORD = LOWORD(wParam) as WORD;

        if msg != WM_KEYDOWN {
            LeaveCriticalSection(core::ptr::addr_of_mut!((*Console).Lock));
            return;
        }

        if VirtualKeyCode == VK_RETURN as WORD {
            Copy(GuiData);
            LeaveCriticalSection(core::ptr::addr_of_mut!((*Console).Lock));
            return;
        } else if VirtualKeyCode == VK_ESCAPE as WORD ||
                  (VirtualKeyCode == b'C' as WORD && (GetKeyState(VK_CONTROL as i32) & KEY_PRESSED as i16) != 0) {
            UpdateSelection(GuiData, core::ptr::null_mut(), core::ptr::null_mut());
            LeaveCriticalSection(core::ptr::addr_of_mut!((*Console).Lock));
            return;
        }

        if ((*GuiData).Selection.dwFlags & CONSOLE_MOUSE_SELECTION) == 0 {
            let mut Interpreted: BOOL = 0;
            let MajPressed: BOOL = if (GetKeyState(VK_SHIFT as i32) & KEY_PRESSED as i16) != 0 { 1 } else { 0 };

            match VirtualKeyCode as u32 {
                VK_LEFT => {
                    Interpreted = 1;
                    if (*GuiData).dwSelectionCursor.X > 0 {
                        (*GuiData).dwSelectionCursor.X -= 1;
                    }
                }

                VK_RIGHT => {
                    Interpreted = 1;
                    if (*GuiData).dwSelectionCursor.X < (*ActiveBuffer).ScreenBufferSize.X - 1 {
                        (*GuiData).dwSelectionCursor.X += 1;
                    }
                }

                VK_UP => {
                    Interpreted = 1;
                    if (*GuiData).dwSelectionCursor.Y > 0 {
                        (*GuiData).dwSelectionCursor.Y -= 1;
                    }
                }

                VK_DOWN => {
                    Interpreted = 1;
                    if (*GuiData).dwSelectionCursor.Y < (*ActiveBuffer).ScreenBufferSize.Y - 1 {
                        (*GuiData).dwSelectionCursor.Y += 1;
                    }
                }

                VK_HOME => {
                    Interpreted = 1;
                    (*GuiData).dwSelectionCursor.X = 0;
                    (*GuiData).dwSelectionCursor.Y = 0;
                }

                VK_END => {
                    Interpreted = 1;
                    (*GuiData).dwSelectionCursor.Y = (*ActiveBuffer).ScreenBufferSize.Y - 1;
                }

                VK_PRIOR => {
                    Interpreted = 1;
                    (*GuiData).dwSelectionCursor.Y -= (*ActiveBuffer).ViewSize.Y;
                    if (*GuiData).dwSelectionCursor.Y < 0 {
                        (*GuiData).dwSelectionCursor.Y = 0;
                    }
                }

                VK_NEXT => {
                    Interpreted = 1;
                    (*GuiData).dwSelectionCursor.Y += (*ActiveBuffer).ViewSize.Y;
                    if (*GuiData).dwSelectionCursor.Y >= (*ActiveBuffer).ScreenBufferSize.Y {
                        (*GuiData).dwSelectionCursor.Y = (*ActiveBuffer).ScreenBufferSize.Y - 1;
                    }
                }

                _ => {}
            }

            if Interpreted != 0 {
                UpdateSelection(
                    GuiData,
                    if MajPressed == 0 {
                        core::ptr::addr_of_mut!((*GuiData).dwSelectionCursor)
                    } else {
                        core::ptr::null_mut()
                    },
                    core::ptr::addr_of_mut!((*GuiData).dwSelectionCursor),
                );
            } else if IsSystemKey(VirtualKeyCode) == 0 {
                SendNotifyMessage((*GuiData).hWindow, PM_CONSOLE_BEEP, 0, 0);
            }

            LeaveCriticalSection(core::ptr::addr_of_mut!((*Console).Lock));
            return;
        } else {
            if IsSystemKey(VirtualKeyCode) == 0 {
                UpdateSelection(GuiData, core::ptr::null_mut(), core::ptr::null_mut());
            } else {
                LeaveCriticalSection(core::ptr::addr_of_mut!((*Console).Lock));
                return;
            }
        }
    }

    if ((*GuiData).Selection.dwFlags & CONSOLE_SELECTION_IN_PROGRESS) == 0 {
        let mut Message: MSG = core::mem::zeroed();

        Message.hwnd = (*GuiData).hWindow;
        Message.message = msg;
        Message.wParam = wParam;
        Message.lParam = lParam;

        ConioProcessKey(Console, core::ptr::addr_of_mut!(Message));
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*Console).Lock));
}

unsafe fn OnTimer(GuiData: PGUI_CONSOLE_DATA) {
    let Console = (*GuiData).Console;
    let Buff: PCONSOLE_SCREEN_BUFFER;

    if (*GuiData).IsWindowVisible == 0 {
        return;
    }

    SetTimer((*GuiData).hWindow, CONGUI_UPDATE_TIMER as UINT_PTR, CURSOR_BLINK_TIME, None);

    if ConDrvValidateConsoleUnsafe(Console as PCONSOLE, CONSOLE_RUNNING, 1u8) == 0 {
        return;
    }

    Buff = (*GuiData).ActiveBuffer;

    if GetType(Buff) == TEXTMODE_BUFFER {
        if (*GuiData).IsWindowActive != 0 || (*Buff).CursorBlinkOn != 0 {
            InvalidateCell(GuiData, (*Buff).CursorPosition.X, (*Buff).CursorPosition.Y);
            (*Buff).CursorBlinkOn = if (*Buff).CursorBlinkOn != 0 { 0u8 } else { 1u8 };
        }

        if (*GuiData).OldCursor.x != (*Buff).CursorPosition.X as i32 as i32 ||
           (*GuiData).OldCursor.y != (*Buff).CursorPosition.Y as i32 as i32 {
            let mut sInfo: SCROLLINFO = core::mem::zeroed();
            let mut OldScrollX: i32 = -1;
            let mut OldScrollY: i32 = -1;
            let mut NewScrollX: i32 = -1;
            let mut NewScrollY: i32 = -1;

            sInfo.cbSize = core::mem::size_of::<SCROLLINFO>() as u32;
            sInfo.fMask = SIF_POS;
            
            if GetScrollInfo((*GuiData).hWindow, SB_HORZ as i32, &mut sInfo) != 0 {
                OldScrollX = sInfo.nPos;
            }
            if GetScrollInfo((*GuiData).hWindow, SB_VERT as i32, &mut sInfo) != 0 {
                OldScrollY = sInfo.nPos;
            }

            if OldScrollX >= 0 {
                if (*Buff).CursorPosition.X < (*Buff).ViewOrigin.X ||
                   (*Buff).CursorPosition.X >= (*Buff).ViewOrigin.X + (*Buff).ViewSize.X {
                    if (*Buff).CursorPosition.X >= (*Buff).ViewSize.X {
                        NewScrollX = ((*Buff).CursorPosition.X - (*Buff).ViewSize.X + 1) as i32;
                    } else {
                        NewScrollX = 0;
                    }
                } else {
                    NewScrollX = OldScrollX;
                }
            }
            
            if OldScrollY >= 0 {
                if (*Buff).CursorPosition.Y < (*Buff).ViewOrigin.Y ||
                   (*Buff).CursorPosition.Y >= (*Buff).ViewOrigin.Y + (*Buff).ViewSize.Y {
                    if (*Buff).CursorPosition.Y >= (*Buff).ViewSize.Y {
                        NewScrollY = ((*Buff).CursorPosition.Y - (*Buff).ViewSize.Y + 1) as i32;
                    } else {
                        NewScrollY = 0;
                    }
                } else {
                    NewScrollY = OldScrollY;
                }
            }

            if OldScrollX != NewScrollX || OldScrollY != NewScrollY {
                (*Buff).ViewOrigin.X = NewScrollX as i16;
                (*Buff).ViewOrigin.Y = NewScrollY as i16;
                ScrollWindowEx(
                    (*GuiData).hWindow,
                    (OldScrollX - NewScrollX) * (*GuiData).CharWidth as i32,
                    (OldScrollY - NewScrollY) * (*GuiData).CharHeight as i32,
                    core::ptr::null(),
                    core::ptr::null(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    SW_INVALIDATE
                );
                if NewScrollX >= 0 {
                    sInfo.nPos = NewScrollX;
                    SetScrollInfo((*GuiData).hWindow, SB_HORZ as i32, &mut sInfo, 1);
                }
                if NewScrollY >= 0 {
                    sInfo.nPos = NewScrollY;
                    SetScrollInfo((*GuiData).hWindow, SB_VERT as i32, &mut sInfo, 1);
                }
                UpdateWindow((*GuiData).hWindow);
                (*GuiData).OldCursor.x = (*Buff).CursorPosition.X as i32;
                (*GuiData).OldCursor.y = (*Buff).CursorPosition.Y as i32;
            }
        }
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*Console).Lock));
}

unsafe fn OnClose(GuiData: PGUI_CONSOLE_DATA) -> BOOL {
    let Console: PCONSRV_CONSOLE = (*GuiData).Console;

    if ConDrvValidateConsoleUnsafe(Console as PCONSOLE, CONSOLE_RUNNING, 1u8) == 0 {
        return 1;
    }

    ConSrvConsoleProcessCtrlEvent(Console, 0, CTRL_CLOSE_EVENT);

    LeaveCriticalSection(core::ptr::addr_of_mut!((*Console).Lock));
    return 0;
}

unsafe fn OnNcDestroy(hWnd: HWND) -> LRESULT {
    let GuiData = GuiGetGuiData(hWnd);

    SetWindowLongPtrW(hWnd, GWLP_USERDATA, 0 as LONG_PTR);

    GetSystemMenu(hWnd, 1);

    if !GuiData.is_null() {
        if (*GuiData).IsWindowVisible != 0 {
            KillTimer(hWnd, 1);
        }

        if (*GuiData).hMemDC != core::ptr::null_mut() {
            DeleteDC((*GuiData).hMemDC);
        }
        if (*GuiData).hBitmap != core::ptr::null_mut() {
            DeleteObject((*GuiData).hBitmap as *mut core::ffi::c_void);
        }
        DeleteFonts(GuiData);
    }

    DefWindowProcW(hWnd, WM_NCDESTROY, 0, 0)
}

unsafe fn OnScroll(GuiData: PGUI_CONSOLE_DATA, nBar: INT, sbCode: WORD) {
    let Console: PCONSRV_CONSOLE = (*GuiData).Console;
    let Buff: PCONSOLE_SCREEN_BUFFER;
    let mut sInfo: SCROLLINFO = core::mem::zeroed();
    let oldPos: INT;
    let Maximum: INT;
    let pOriginXY: PSHORT;

    if nBar != SB_HORZ as i32 && nBar != SB_VERT as i32 {
        return;
    }

    if ConDrvValidateConsoleUnsafe(Console as PCONSOLE, CONSOLE_RUNNING, 1u8) == 0 {
        return;
    }

    Buff = (*GuiData).ActiveBuffer;

    if nBar == SB_HORZ as i32 {
        Maximum = (*Buff).ScreenBufferSize.X as INT - (*Buff).ViewSize.X as INT;
        pOriginXY = core::ptr::addr_of_mut!((*Buff).ViewOrigin.X);
    } else {
        Maximum = (*Buff).ScreenBufferSize.Y as INT - (*Buff).ViewSize.Y as INT;
        pOriginXY = core::ptr::addr_of_mut!((*Buff).ViewOrigin.Y);
    }

    sInfo.cbSize = core::mem::size_of::<SCROLLINFO>() as UINT;
    sInfo.fMask = SIF_RANGE | SIF_POS | SIF_PAGE | SIF_TRACKPOS;

    if GetScrollInfo((*GuiData).hWindow, nBar, &mut sInfo) == 0 {
        LeaveCriticalSection(core::ptr::addr_of_mut!((*Console).Lock));
        return;
    }

    oldPos = sInfo.nPos;

    match sbCode as u32 {
        SB_LINEUP => {
            sInfo.nPos -= 1;
        }
        SB_LINEDOWN => {
            sInfo.nPos += 1;
        }
        SB_PAGEUP => {
            sInfo.nPos -= sInfo.nPage as INT;
        }
        SB_PAGEDOWN => {
            sInfo.nPos += sInfo.nPage as INT;
        }
        SB_THUMBTRACK => {
            sInfo.nPos = sInfo.nTrackPos;
            ConioPause(Console, PAUSED_FROM_SCROLLBAR as u8);
        }
        SB_THUMBPOSITION => {
            sInfo.nPos = sInfo.nTrackPos;
            ConioUnpause(Console, PAUSED_FROM_SCROLLBAR as u8);
        }
        SB_TOP => {
            sInfo.nPos = sInfo.nMin;
        }
        SB_BOTTOM => {
            sInfo.nPos = sInfo.nMax;
        }
        _ => {}
    }

    sInfo.nPos = min(max(sInfo.nPos, 0), Maximum);

    if oldPos != sInfo.nPos {
        let OldX: USHORT = (*Buff).ViewOrigin.X as USHORT;
        let OldY: USHORT = (*Buff).ViewOrigin.Y as USHORT;
        let mut WidthUnit: UINT = 0;
        let mut HeightUnit: UINT = 0;

        *pOriginXY = sInfo.nPos as SHORT;

        GetScreenBufferSizeUnits(Buff, GuiData, &mut WidthUnit, &mut HeightUnit);

        ScrollWindowEx(
            (*GuiData).hWindow,
            (OldX as INT - (*Buff).ViewOrigin.X as INT) * WidthUnit as INT,
            (OldY as INT - (*Buff).ViewOrigin.Y as INT) * HeightUnit as INT,
            core::ptr::null(),
            core::ptr::null(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            SW_INVALIDATE,
        );

        sInfo.fMask = SIF_POS;
        SetScrollInfo((*GuiData).hWindow, nBar, &sInfo, 1);

        UpdateWindow((*GuiData).hWindow);
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*Console).Lock));
}

unsafe fn PointToCoord(GuiData: PGUI_CONSOLE_DATA, lParam: LPARAM) -> COORD {
    let Buffer: PCONSOLE_SCREEN_BUFFER = (*GuiData).ActiveBuffer;
    let mut Coord: COORD = core::mem::zeroed();
    let mut WidthUnit: UINT = 0;
    let mut HeightUnit: UINT = 0;

    GetScreenBufferSizeUnits(Buffer, GuiData, &mut WidthUnit, &mut HeightUnit);

    Coord.X = (*Buffer).ViewOrigin.X + ((LOWORD(lParam as u32) as i16) / (WidthUnit as i32) as i16);
    Coord.Y = (*Buffer).ViewOrigin.Y + ((HIWORD(lParam as u32) as i16) / (HeightUnit as i32) as i16);

    if Coord.X < 0 {
        Coord.X = 0;
    } else if Coord.X >= (*Buffer).ScreenBufferSize.X {
        Coord.X = (*Buffer).ScreenBufferSize.X - 1;
    }

    if Coord.Y < 0 {
        Coord.Y = 0;
    } else if Coord.Y >= (*Buffer).ScreenBufferSize.Y {
        Coord.Y = (*Buffer).ScreenBufferSize.Y - 1;
    }

    Coord
}

unsafe fn OnMouse(GuiData: PGUI_CONSOLE_DATA, msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    let mut DoDefault: BOOL = 0;
    let Console: PCONSRV_CONSOLE = (*GuiData).Console;

    if (*GuiData).HackCORE8394IgnoreNextMove != 0 && msg == WM_MOUSEMOVE {
        (*GuiData).HackCORE8394IgnoreNextMove = 0;
        'Quit: {
            if DoDefault == 0 {
                return 0;
            }

            if msg == WM_MOUSEWHEEL || msg == WM_MOUSEHWHEEL {
                let nBar: INT;
                let sbCode: WORD;
                let wDelta: SHORT = GET_WHEEL_DELTA_WPARAM(wParam);

                if msg == WM_MOUSEWHEEL {
                    nBar = SB_VERT as INT;
                } else {
                    nBar = SB_HORZ as INT;
                }

                if GetKeyState(VK_SHIFT as i32) & KEY_PRESSED as i16 != 0 {
                    sbCode = if wDelta >= 0 { SB_PAGEUP as u16 } else { SB_PAGEDOWN as u16 };
                } else {
                    sbCode = if wDelta >= 0 { SB_LINEUP as u16 } else { SB_LINEDOWN as u16 };
                }

                OnScroll(GuiData, nBar, sbCode);
            }

            return DefWindowProcW((*GuiData).hWindow, msg, wParam, lParam);
        }
    }
    (*GuiData).HackCORE8394IgnoreNextMove = 0;

    if (*GuiData).IgnoreNextMouseEvent != 0 {
        if msg != WM_LBUTTONDOWN &&
            msg != WM_MBUTTONDOWN &&
            msg != WM_RBUTTONDOWN &&
            msg != WM_XBUTTONDOWN
        {
            (*GuiData).IgnoreNextMouseEvent = 0;
        } else {
            DoDefault = 1;
        }
        'Quit: {
            if DoDefault == 0 {
                return 0;
            }

            if msg == WM_MOUSEWHEEL || msg == WM_MOUSEHWHEEL {
                let nBar: INT;
                let sbCode: WORD;
                let wDelta: SHORT = GET_WHEEL_DELTA_WPARAM(wParam);

                if msg == WM_MOUSEWHEEL {
                    nBar = SB_VERT as INT;
                } else {
                    nBar = SB_HORZ as INT;
                }

                if GetKeyState(VK_SHIFT as i32) & KEY_PRESSED as i16 != 0 {
                    sbCode = if wDelta >= 0 { SB_PAGEUP as u16 } else { SB_PAGEDOWN as u16 };
                } else {
                    sbCode = if wDelta >= 0 { SB_LINEUP as u16 } else { SB_LINEDOWN as u16 };
                }

                OnScroll(GuiData, nBar, sbCode);
            }

            return DefWindowProcW((*GuiData).hWindow, msg, wParam, lParam);
        }
    }

    if ConDrvValidateConsoleUnsafe(Console as PCONSOLE, CONSOLE_RUNNING, 1u8) == 0 {
        DoDefault = 1;
        'Quit: {
            if DoDefault == 0 {
                return 0;
            }

            if msg == WM_MOUSEWHEEL || msg == WM_MOUSEHWHEEL {
                let nBar: INT;
                let sbCode: WORD;
                let wDelta: SHORT = GET_WHEEL_DELTA_WPARAM(wParam);

                if msg == WM_MOUSEWHEEL {
                    nBar = SB_VERT as INT;
                } else {
                    nBar = SB_HORZ as INT;
                }

                if GetKeyState(VK_SHIFT as i32) & KEY_PRESSED as i16 != 0 {
                    sbCode = if wDelta >= 0 { SB_PAGEUP as u16 } else { SB_PAGEDOWN as u16 };
                } else {
                    sbCode = if wDelta >= 0 { SB_LINEUP as u16 } else { SB_LINEDOWN as u16 };
                }

                OnScroll(GuiData, nBar, sbCode);
            }

            return DefWindowProcW((*GuiData).hWindow, msg, wParam, lParam);
        }
    }

    if ((*GuiData).Selection.dwFlags & CONSOLE_SELECTION_IN_PROGRESS) != 0 ||
        (*Console).QuickEdit != 0
    {
        match msg {
            WM_LBUTTONDOWN => {
                if ((*GuiData).Selection.dwFlags & CONSOLE_SELECTION_IN_PROGRESS) != 0 &&
                    ((*GuiData).Selection.dwFlags & CONSOLE_MOUSE_SELECTION) != 0 &&
                    (GetKeyState(VK_SHIFT as i32) & KEY_PRESSED as i16) != 0
                {
                    (*GuiData).dwSelectionCursor = PointToCoord(GuiData, lParam);
                    UpdateSelection(GuiData, core::ptr::null_mut(), core::ptr::addr_of_mut!((*GuiData).dwSelectionCursor));
                } else {
                    (*GuiData).Selection.dwFlags = CONSOLE_NO_SELECTION;

                    (*GuiData).dwSelectionCursor = PointToCoord(GuiData, lParam);
                    SetCapture((*GuiData).hWindow);
                    (*GuiData).Selection.dwFlags |= CONSOLE_MOUSE_SELECTION | CONSOLE_MOUSE_DOWN;
                    UpdateSelection(GuiData,
                                    core::ptr::addr_of_mut!((*GuiData).dwSelectionCursor),
                                    core::ptr::addr_of_mut!((*GuiData).dwSelectionCursor));
                }
            }

            WM_LBUTTONUP => {
                if ((*GuiData).Selection.dwFlags & CONSOLE_MOUSE_DOWN) == 0 {
                } else {
                    (*GuiData).Selection.dwFlags &= !CONSOLE_MOUSE_DOWN;
                    ReleaseCapture();
                }
            }

            WM_LBUTTONDBLCLK => {
                let Buffer: PCONSOLE_SCREEN_BUFFER = (*GuiData).ActiveBuffer;

                if GetType(Buffer) == TEXTMODE_BUFFER {
                    let TextBuffer: PTEXTMODE_SCREEN_BUFFER = Buffer as PTEXTMODE_SCREEN_BUFFER;
                    let mut cL: COORD;
                    let mut cR: COORD;
                    let mut ptrL: PCHAR_INFO;
                    let mut ptrR: PCHAR_INFO;

                    cL = PointToCoord(GuiData, lParam);
                    cR = cL;
                    ptrL = ConioCoordToPointer(TextBuffer, cL.X as u32, cL.Y as u32);
                    ptrR = ptrL;

                    while (0 < cL.X) && !IS_WORD_SEP((*ptrL).Char.UnicodeChar)
                                      && !IS_WORD_SEP((*ptrL.offset(-1)).Char.UnicodeChar)
                    {
                        cL.X -= 1;
                        ptrL = ptrL.offset(-1);
                    }
                    while (cR.X < (*TextBuffer).ScreenBufferSize.X - 1) &&
                           !IS_WORD_SEP((*ptrR).Char.UnicodeChar)        &&
                           !IS_WORD_SEP((*ptrR.offset(1)).Char.UnicodeChar)
                    {
                        cR.X += 1;
                        ptrR = ptrR.offset(1);
                    }

                    (*GuiData).Selection.dwFlags |= CONSOLE_MOUSE_SELECTION | CONSOLE_MOUSE_DOWN;
                    UpdateSelection(GuiData, core::ptr::addr_of_mut!(cL), core::ptr::addr_of_mut!(cR));

                    (*GuiData).IgnoreNextMouseEvent = 1;
                }
            }

            WM_RBUTTONDOWN | WM_RBUTTONDBLCLK => {
                if ((*GuiData).Selection.dwFlags & CONSOLE_SELECTION_NOT_EMPTY) == 0 {
                    Paste(GuiData);
                } else {
                    Copy(GuiData);
                }

                (*GuiData).IgnoreNextMouseEvent = 1;
            }

            WM_MOUSEMOVE => {
                if (GET_KEYSTATE_WPARAM(wParam) & MK_LBUTTON) == 0 {
                } else if ((*GuiData).Selection.dwFlags & CONSOLE_MOUSE_DOWN) == 0 {
                } else {
                    (*GuiData).dwSelectionCursor = PointToCoord(GuiData, lParam);
                    UpdateSelection(GuiData, core::ptr::null_mut(), core::ptr::addr_of_mut!((*GuiData).dwSelectionCursor));
                }
            }

            _ => {
                DoDefault = 1;
            }
        }

        match msg {
            WM_LBUTTONDOWN | WM_MBUTTONDOWN | WM_RBUTTONDOWN | WM_XBUTTONDOWN => {
                (*GuiData).HackCORE8394IgnoreNextMove = 1;
            }
            _ => {}
        }
    } else if (GetConsoleInputBufferMode(Console) & ENABLE_MOUSE_INPUT as u32) != 0 {
        let mut er: INPUT_RECORD = core::mem::zeroed();
        let wKeyState: WORD = GET_KEYSTATE_WPARAM(wParam) as WORD;
        let mut dwButtonState: DWORD = 0;
        let mut dwControlKeyState: DWORD = 0;
        let mut dwEventFlags: DWORD = 0;

        match msg {
            WM_LBUTTONDOWN => {
                SetCapture((*GuiData).hWindow);
                dwButtonState = FROM_LEFT_1ST_BUTTON_PRESSED;
                dwEventFlags  = 0;
            }

            WM_MBUTTONDOWN => {
                SetCapture((*GuiData).hWindow);
                dwButtonState = FROM_LEFT_2ND_BUTTON_PRESSED;
                dwEventFlags  = 0;
            }

            WM_RBUTTONDOWN => {
                SetCapture((*GuiData).hWindow);
                dwButtonState = RIGHTMOST_BUTTON_PRESSED;
                dwEventFlags  = 0;
            }

            WM_XBUTTONDOWN => {
                let wButton: WORD = GET_XBUTTON_WPARAM(wParam) as WORD;

                if (wButton & !((XBUTTON1 | XBUTTON2) as WORD)) != 0 {
                    DoDefault = 1;
                } else {
                    SetCapture((*GuiData).hWindow);
                    dwButtonState = if wButton == XBUTTON1 as WORD { FROM_LEFT_3RD_BUTTON_PRESSED } else { FROM_LEFT_4TH_BUTTON_PRESSED };
                    dwEventFlags  = 0;
                }
            }

            WM_LBUTTONUP => {
                ReleaseCapture();
                dwButtonState = 0;
                dwEventFlags  = 0;
            }

            WM_MBUTTONUP => {
                ReleaseCapture();
                dwButtonState = 0;
                dwEventFlags  = 0;
            }

            WM_RBUTTONUP => {
                ReleaseCapture();
                dwButtonState = 0;
                dwEventFlags  = 0;
            }

            WM_XBUTTONUP => {
                let wButton: WORD = GET_XBUTTON_WPARAM(wParam) as WORD;

                if (wButton & !((XBUTTON1 | XBUTTON2) as WORD)) != 0 {
                }

                ReleaseCapture();
                dwButtonState = 0;
                dwEventFlags  = 0;
            }

            WM_LBUTTONDBLCLK => {
                dwButtonState = FROM_LEFT_1ST_BUTTON_PRESSED;
                dwEventFlags  = DOUBLE_CLICK;
            }

            WM_MBUTTONDBLCLK => {
                dwButtonState = FROM_LEFT_2ND_BUTTON_PRESSED;
                dwEventFlags  = DOUBLE_CLICK;
            }

            WM_RBUTTONDBLCLK => {
                dwButtonState = RIGHTMOST_BUTTON_PRESSED;
                dwEventFlags  = DOUBLE_CLICK;
            }

            WM_XBUTTONDBLCLK => {
                let wButton: WORD = GET_XBUTTON_WPARAM(wParam) as WORD;

                if (wButton & !((XBUTTON1 | XBUTTON2) as WORD)) != 0 {
                    DoDefault = 1;
                } else {
                    dwButtonState = if wButton == XBUTTON1 as WORD { FROM_LEFT_3RD_BUTTON_PRESSED } else { FROM_LEFT_4TH_BUTTON_PRESSED };
                    dwEventFlags  = DOUBLE_CLICK;
                }
            }

            WM_MOUSEMOVE => {
                dwButtonState = 0;
                dwEventFlags  = MOUSE_MOVED;
            }

            WM_MOUSEWHEEL => {
                dwButtonState = (GET_WHEEL_DELTA_WPARAM(wParam) as DWORD) << 16;
                dwEventFlags  = MOUSE_WHEELED;
            }

            WM_MOUSEHWHEEL => {
                dwButtonState = (GET_WHEEL_DELTA_WPARAM(wParam) as DWORD) << 16;
                dwEventFlags  = MOUSE_HWHEELED;
            }

            _ => {
                DoDefault = 1;
            }
        }

        match msg {
            WM_LBUTTONDOWN | WM_MBUTTONDOWN | WM_RBUTTONDOWN | WM_XBUTTONDOWN => {
                (*GuiData).HackCORE8394IgnoreNextMove = 1;
            }
            _ => {}
        }

        if DoDefault == 0 {
            if (wKeyState & MK_LBUTTON as u16) != 0 {
                dwButtonState |= FROM_LEFT_1ST_BUTTON_PRESSED;
            }
            if (wKeyState & MK_MBUTTON as u16) != 0 {
                dwButtonState |= FROM_LEFT_2ND_BUTTON_PRESSED;
            }
            if (wKeyState & MK_RBUTTON as u16) != 0 {
                dwButtonState |= RIGHTMOST_BUTTON_PRESSED;
            }
            if (wKeyState & MK_XBUTTON1 as u16) != 0 {
                dwButtonState |= FROM_LEFT_3RD_BUTTON_PRESSED;
            }
            if (wKeyState & MK_XBUTTON2 as u16) != 0 {
                dwButtonState |= FROM_LEFT_4TH_BUTTON_PRESSED;
            }

            if (GetKeyState(VK_RMENU as i32) & KEY_PRESSED as i16) != 0 {
                dwControlKeyState |= RIGHT_ALT_PRESSED;
            }
            if (GetKeyState(VK_LMENU as i32) & KEY_PRESSED as i16) != 0 {
                dwControlKeyState |= LEFT_ALT_PRESSED;
            }
            if (GetKeyState(VK_RCONTROL as i32) & KEY_PRESSED as i16) != 0 {
                dwControlKeyState |= RIGHT_CTRL_PRESSED;
            }
            if (GetKeyState(VK_LCONTROL as i32) & KEY_PRESSED as i16) != 0 {
                dwControlKeyState |= LEFT_CTRL_PRESSED;
            }
            if (GetKeyState(VK_SHIFT as i32) & KEY_PRESSED as i16) != 0 {
                dwControlKeyState |= SHIFT_PRESSED;
            }
            if (GetKeyState(VK_NUMLOCK as i32) & KEY_TOGGLED as i16) != 0 {
                dwControlKeyState |= NUMLOCK_ON;
            }
            if (GetKeyState(VK_SCROLL as i32) & KEY_TOGGLED as i16) != 0 {
                dwControlKeyState |= SCROLLLOCK_ON;
            }
            if (GetKeyState(VK_CAPITAL as i32) & KEY_TOGGLED as i16) != 0 {
                dwControlKeyState |= CAPSLOCK_ON;
            }
            if (HIWORD(lParam as u32) & KF_EXTENDED as u16) != 0 {
                dwControlKeyState |= ENHANCED_KEY;
            }

            er.EventType = MOUSE_EVENT as u16;
            er.Event.MouseEvent.dwMousePosition   = PointToCoord(GuiData, lParam);
            er.Event.MouseEvent.dwButtonState     = dwButtonState;
            er.Event.MouseEvent.dwControlKeyState = dwControlKeyState;
            er.Event.MouseEvent.dwEventFlags      = dwEventFlags;

            ConioProcessInputEvent(Console, core::ptr::addr_of_mut!(er));
        }
    } else {
        DoDefault = 1;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*Console).Lock));

    if DoDefault == 0 {
        return 0;
    }

    if msg == WM_MOUSEWHEEL || msg == WM_MOUSEHWHEEL {
        let nBar: INT;
        let sbCode: WORD;
        let wDelta: SHORT = GET_WHEEL_DELTA_WPARAM(wParam);

        if msg == WM_MOUSEWHEEL {
            nBar = SB_VERT as INT;
        } else {
            nBar = SB_HORZ as INT;
        }

        if (GetKeyState(VK_SHIFT as i32) & KEY_PRESSED as i16) != 0 {
            sbCode = if wDelta >= 0 { SB_PAGEUP as u16 } else { SB_PAGEDOWN as u16 };
        } else {
            sbCode = if wDelta >= 0 { SB_LINEUP as u16 } else { SB_LINEDOWN as u16 };
        }

        OnScroll(GuiData, nBar, sbCode);
    }

    DefWindowProcW((*GuiData).hWindow, msg, wParam, lParam)
}

unsafe fn Copy(GuiData: PGUI_CONSOLE_DATA) {
    if OpenClipboard((*GuiData).hWindow) != 0 {
        let Buffer: PCONSOLE_SCREEN_BUFFER = (*GuiData).ActiveBuffer;

        if GetType(Buffer) == TEXTMODE_BUFFER {
            GuiCopyFromTextModeBuffer(Buffer as PTEXTMODE_SCREEN_BUFFER, GuiData);
        } else {
            GuiCopyFromGraphicsBuffer(Buffer as PGRAPHICS_SCREEN_BUFFER, GuiData);
        }

        CloseClipboard();
    }

    UpdateSelection(GuiData, core::ptr::null_mut(), core::ptr::null_mut());
}

unsafe fn Paste(GuiData: PGUI_CONSOLE_DATA) {
    if OpenClipboard((*GuiData).hWindow) != 0 {
        let Buffer: PCONSOLE_SCREEN_BUFFER = (*GuiData).ActiveBuffer;

        if GetType(Buffer) == TEXTMODE_BUFFER {
            GuiPasteToTextModeBuffer(Buffer as PTEXTMODE_SCREEN_BUFFER, GuiData);
        } else {
            GuiPasteToGraphicsBuffer(Buffer as PGRAPHICS_SCREEN_BUFFER, GuiData);
        }

        CloseClipboard();
    }
}

unsafe fn OnGetMinMaxInfo(GuiData: PGUI_CONSOLE_DATA, minMaxInfo: PMINMAXINFO) {
    let Console: PCONSRV_CONSOLE = (*GuiData).Console;
    let ActiveBuffer: PCONSOLE_SCREEN_BUFFER;
    let mut windx: u32;
    let mut windy: u32;
    let mut WidthUnit: u32 = 0;
    let mut HeightUnit: u32 = 0;

    if ConDrvValidateConsoleUnsafe(Console as PCONSOLE, CONSOLE_RUNNING, 1u8) == 0 {
        return;
    }

    ActiveBuffer = (*GuiData).ActiveBuffer;

    GetScreenBufferSizeUnits(ActiveBuffer, GuiData, &mut WidthUnit, &mut HeightUnit);

    windx = CONGUI_MIN_WIDTH as u32 * WidthUnit + 2 * (GetSystemMetrics(SM_CXFRAME as i32) as u32 + GetSystemMetrics(SM_CXEDGE as i32) as u32);
    windy = CONGUI_MIN_HEIGHT as u32 * HeightUnit + 2 * (GetSystemMetrics(SM_CYFRAME as i32) as u32 + GetSystemMetrics(SM_CYEDGE as i32) as u32) + GetSystemMetrics(SM_CYCAPTION as i32) as u32;

    (*minMaxInfo).ptMinTrackSize.x = windx as i32;
    (*minMaxInfo).ptMinTrackSize.y = windy as i32;

    windx = (*ActiveBuffer).ScreenBufferSize.X as u32 * WidthUnit + 2 * (GetSystemMetrics(SM_CXFRAME as i32) as u32 + GetSystemMetrics(SM_CXEDGE as i32) as u32);
    windy = (*ActiveBuffer).ScreenBufferSize.Y as u32 * HeightUnit + 2 * (GetSystemMetrics(SM_CYFRAME as i32) as u32 + GetSystemMetrics(SM_CYEDGE as i32) as u32) + GetSystemMetrics(SM_CYCAPTION as i32) as u32;

    if (*ActiveBuffer).ViewSize.X < (*ActiveBuffer).ScreenBufferSize.X {
        windy += GetSystemMetrics(SM_CYHSCROLL as i32) as u32;
    }
    if (*ActiveBuffer).ViewSize.Y < (*ActiveBuffer).ScreenBufferSize.Y {
        windx += GetSystemMetrics(SM_CXVSCROLL as i32) as u32;
    }

    (*minMaxInfo).ptMaxTrackSize.x = windx as i32;
    (*minMaxInfo).ptMaxTrackSize.y = windy as i32;

    LeaveCriticalSection(core::ptr::addr_of_mut!((*Console).Lock));
}

unsafe fn OnSize(GuiData: PGUI_CONSOLE_DATA, wParam: WPARAM, lParam: LPARAM) {
    let Console: PCONSRV_CONSOLE = (*GuiData).Console;

    if (*GuiData).IsWindowVisible == 0 || IsIconic((*GuiData).hWindow) != 0 {
        return;
    }

    if ConDrvValidateConsoleUnsafe(Console as PCONSOLE, CONSOLE_RUNNING, 1u8) == 0 {
        return;
    }

    if (*GuiData).WindowSizeLock == 0 &&
       (wParam == SIZE_RESTORED as WPARAM || wParam == SIZE_MAXIMIZED as WPARAM || wParam == SIZE_MINIMIZED as WPARAM)
    {
        let Buff: PCONSOLE_SCREEN_BUFFER = (*GuiData).ActiveBuffer;
        let mut windx: DWORD;
        let mut windy: DWORD;
        let mut charx: DWORD;
        let mut chary: DWORD;
        let mut WidthUnit: UINT = 0;
        let mut HeightUnit: UINT = 0;

        GetScreenBufferSizeUnits(Buff, GuiData, &mut WidthUnit, &mut HeightUnit);

        (*GuiData).WindowSizeLock = 1;

        windx = LOWORD(lParam as u32) as DWORD;
        windy = HIWORD(lParam as u32) as DWORD;

        if (*Buff).ViewSize.X < (*Buff).ScreenBufferSize.X {
            windy = windy.wrapping_add(GetSystemMetrics(SM_CYHSCROLL as i32) as DWORD);
        }
        if (*Buff).ViewSize.Y < (*Buff).ScreenBufferSize.Y {
            windx = windx.wrapping_add(GetSystemMetrics(SM_CXVSCROLL as i32) as DWORD);
        }

        charx = windx / WidthUnit;
        chary = windy / HeightUnit;

        if windx % WidthUnit >= WidthUnit / 2 {
            charx = charx.wrapping_add(1);
        }
        if windy % HeightUnit >= HeightUnit / 2 {
            chary = chary.wrapping_add(1);
        }

        if (*Buff).ViewSize.X < (*Buff).ScreenBufferSize.X {
            windy = windy.wrapping_sub(GetSystemMetrics(SM_CYHSCROLL as i32) as DWORD);
        }
        if (*Buff).ViewSize.Y < (*Buff).ScreenBufferSize.Y {
            windx = windx.wrapping_sub(GetSystemMetrics(SM_CXVSCROLL as i32) as DWORD);
        }

        charx = windx / WidthUnit;
        chary = windy / HeightUnit;

        if windx % WidthUnit >= WidthUnit / 2 {
            charx = charx.wrapping_add(1);
        }
        if windy % HeightUnit >= HeightUnit / 2 {
            chary = chary.wrapping_add(1);
        }

        if charx != (*Buff).ViewSize.X as DWORD || chary != (*Buff).ViewSize.Y as DWORD {
            (*Buff).ViewSize.X = if charx <= (*Buff).ScreenBufferSize.X as DWORD {
                charx as SHORT
            } else {
                (*Buff).ScreenBufferSize.X
            };
            (*Buff).ViewSize.Y = if chary <= (*Buff).ScreenBufferSize.Y as DWORD {
                chary as SHORT
            } else {
                (*Buff).ScreenBufferSize.Y
            };
        }

        ResizeConWnd(GuiData, WidthUnit, HeightUnit);

        if ((*Buff).ScreenBufferSize.X - (*Buff).ViewOrigin.X) < (*Buff).ViewSize.X {
            (*Buff).ViewOrigin.X = (*Buff).ScreenBufferSize.X - (*Buff).ViewSize.X;
        }
        if ((*Buff).ScreenBufferSize.Y - (*Buff).ViewOrigin.Y) < (*Buff).ViewSize.Y {
            (*Buff).ViewOrigin.Y = (*Buff).ScreenBufferSize.Y - (*Buff).ViewSize.Y;
        }
        InvalidateRect((*GuiData).hWindow, core::ptr::null_mut(), 1);

        (*GuiData).WindowSizeLock = 0;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*Console).Lock));
}

unsafe fn OnMove(GuiData: PGUI_CONSOLE_DATA) {
    let mut rcWnd: RECT = core::mem::zeroed();

    GetWindowRect((*GuiData).hWindow, &mut rcWnd);
    (*GuiData).GuiInfo.WindowOrigin.x = rcWnd.left;
    (*GuiData).GuiInfo.WindowOrigin.y = rcWnd.top;
}

unsafe fn OnDropFiles(Console: PCONSRV_CONSOLE, hDrop: HDROP) {
    let pszPath: LPWSTR;
    let mut szPath: [u16; MAX_PATH as usize + 2] = [0; MAX_PATH as usize + 2];

    szPath[0] = b'"' as u16;

    DragQueryFileW(hDrop, 0, szPath.as_mut_ptr().add(1), (ARRAYSIZE(&szPath) - 1) as u32);
    DragFinish(hDrop);

    if !wcschr(szPath.as_ptr().add(1), b' ' as u16).is_null() {
        StringCchCatW(szPath.as_mut_ptr(), ARRAYSIZE(&szPath), w!("\"").as_ptr() as *const u16);
        pszPath = szPath.as_mut_ptr();
    } else {
        pszPath = szPath.as_mut_ptr().add(1);
    }

    super::text::PasteText(Console, pszPath, wcslen(pszPath) as usize);
}

unsafe fn CreateFrameBufferBitmap(hDC: HDC, width: i32, height: i32) -> HBITMAP {
    let mut bmi: BITMAPINFO = core::mem::zeroed();
    bmi.bmiHeader.biSize = core::mem::size_of::<BITMAPINFOHEADER>() as u32;
    bmi.bmiHeader.biWidth = width;
    bmi.bmiHeader.biHeight = height;
    bmi.bmiHeader.biPlanes = 1;
    bmi.bmiHeader.biBitCount = GetDeviceCaps(hDC, BITSPIXEL as i32) as u16;
    bmi.bmiHeader.biCompression = BI_RGB;
    CreateDIBSection(
        core::ptr::null_mut(),
        &bmi,
        DIB_RGB_COLORS,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        0,
    )
}

pub unsafe extern "system" fn ConWndProc(hWnd: HWND, msg: u32, wParam: WPARAM, lParam: LPARAM) -> LRESULT {

    let mut Result: LRESULT = 0;
    let mut GuiData: PGUI_CONSOLE_DATA = core::ptr::null_mut();
    let mut Console: PCONSRV_CONSOLE = core::ptr::null_mut();

    if msg == WM_NCCREATE {
        return OnNcCreate(hWnd, lParam as LPCREATESTRUCTW) as LRESULT;
    } else if msg == WM_NCDESTROY {
        return OnNcDestroy(hWnd);
    }

    GuiData = GuiGetGuiData(hWnd);
    if GuiData.is_null() {
        return DefWindowProcW(hWnd, msg, wParam, lParam);
    }

    if (*GuiData).ActiveBuffer.is_null() {
        return DefWindowProcW(hWnd, msg, wParam, lParam);
    }

    Console = (*GuiData).Console;

    'Default: loop {
        match msg {
            WM_ACTIVATE => {
                OnActivate(GuiData, wParam);
            }

            WM_CLOSE => {
                if OnClose(GuiData) != 0 {
                    break 'Default;
                }
            }

            WM_ERASEBKGND => {
                return 1;
            }

            WM_PAINT => {
                OnPaint(GuiData);
            }

            WM_TIMER => {
                OnTimer(GuiData);
            }

            WM_PALETTECHANGED => {
                if hWnd as HWND == wParam as HWND {
                    break;
                }

                OnPaletteChanged(GuiData);
            }

            WM_KEYDOWN | WM_KEYUP | WM_CHAR | WM_DEADCHAR | WM_SYSKEYDOWN | WM_SYSKEYUP | WM_SYSCHAR | WM_SYSDEADCHAR => {
                if msg == WM_SYSKEYDOWN && (HIWORD(lParam as u32) & KF_ALTDOWN as u16) != 0 && wParam == VK_RETURN as WPARAM {
                    if (HIWORD(lParam as u32) & (KF_UP | KF_REPEAT) as u16) != KF_REPEAT as u16 {
                        GuiConsoleSwitchFullScreen(GuiData);
                    }
                    break;
                }

                if (HIWORD(lParam as u32) & KF_ALTDOWN as u16) != 0 && (wParam == VK_ESCAPE as WPARAM || wParam == VK_SPACE as WPARAM || wParam == VK_TAB as WPARAM) {
                    return DefWindowProcW(hWnd, msg, wParam, lParam);
                }

                if wParam == VK_SHIFT as WPARAM && (msg == WM_SYSKEYDOWN || msg == WM_SYSKEYUP) {
                    return DefWindowProcW(hWnd, msg, wParam, lParam);
                }

                OnKey(GuiData, msg, wParam, lParam);
            }

            WM_SETCURSOR => {
                if (*GuiData).IsWindowVisible == 0 {
                    break 'Default;
                }

                let mut final_wParam = wParam;
                let mut final_lParam = lParam;

                if wParam == (-1i32) as WPARAM && lParam == (-1i32) as LPARAM {
                    let mut mouseCoords: POINT = core::mem::zeroed();
                    let hWndHit: HWND;

                    GetCursorPos(&mut mouseCoords);
                    hWndHit = WindowFromPoint(mouseCoords);

                    if hWndHit == hWnd {
                        final_wParam = hWnd as WPARAM;
                        final_lParam = DefWindowProcW(hWndHit, WM_NCHITTEST, 0, MAKELPARAM(mouseCoords.x as u16, mouseCoords.y as u16));
                    }
                }

                if (final_wParam as usize) as HWND == hWnd && LOWORD(final_lParam as u32) == HTCLIENT as u16 {
                    if (*GuiData).MouseCursorRefCount >= 0 {
                        SetCursor((*GuiData).hCursor);
                    } else {
                        SetCursor(core::ptr::null_mut());
                    }
                    return 1;
                } else {
                    break 'Default;
                }
            }

            WM_LBUTTONDOWN | WM_MBUTTONDOWN | WM_RBUTTONDOWN | WM_XBUTTONDOWN | WM_LBUTTONUP | WM_MBUTTONUP | WM_RBUTTONUP | WM_XBUTTONUP | WM_LBUTTONDBLCLK | WM_MBUTTONDBLCLK | WM_RBUTTONDBLCLK | WM_XBUTTONDBLCLK | WM_MOUSEMOVE | WM_MOUSEWHEEL | WM_MOUSEHWHEEL => {
                Result = OnMouse(GuiData, msg, wParam, lParam);
            }

            WM_HSCROLL => {
                OnScroll(GuiData, SB_HORZ as i32, LOWORD(wParam));
            }

            WM_VSCROLL => {
                OnScroll(GuiData, SB_VERT as i32, LOWORD(wParam));
            }

            WM_CONTEXTMENU => {
                if (*GuiData).IsWindowVisible == 0 {
                    break;
                }

                if DefWindowProcW(hWnd, WM_NCHITTEST, 0, lParam) == HTCLIENT as LRESULT {
                    let hMenu: HMENU = CreatePopupMenu();
                    if !hMenu.is_null() {
                        AppendMenuItems(hMenu, GuiConsoleEditMenuItems.as_ptr());
                        TrackPopupMenuEx(hMenu, TPM_RIGHTBUTTON, GET_X_LPARAM(lParam), GET_Y_LPARAM(lParam), hWnd, core::ptr::null_mut());
                        DestroyMenu(hMenu);
                    }
                    break;
                } else {
                    break 'Default;
                }
            }

            WM_INITMENU => {
                let hMenu: HMENU = wParam as HMENU;
                if !hMenu.is_null() {
                    EnableMenuItem(hMenu, SC_CLOSE, MF_BYCOMMAND | if (*GuiData).IsCloseButtonEnabled != 0 { MF_ENABLED } else { MF_GRAYED });

                    EnableMenuItem(hMenu, ID_SYSTEM_EDIT_COPY, MF_BYCOMMAND | if ((*GuiData).Selection.dwFlags & CONSOLE_SELECTION_IN_PROGRESS) != 0 && ((*GuiData).Selection.dwFlags & CONSOLE_SELECTION_NOT_EMPTY) != 0 { MF_ENABLED } else { MF_GRAYED });

                    EnableMenuItem(hMenu, ID_SYSTEM_EDIT_PASTE, MF_BYCOMMAND | if ((*GuiData).Selection.dwFlags & CONSOLE_SELECTION_IN_PROGRESS) == 0 && IsClipboardFormatAvailable(CF_UNICODETEXT) != 0 { MF_ENABLED } else { MF_GRAYED });
                }

                SendMenuEvent(Console, WM_INITMENU);
            }

            WM_MENUSELECT => {
                if HIWORD(wParam) == 0xFFFF {
                    SendMenuEvent(Console, WM_MENUSELECT);
                }
            }

            WM_COMMAND | WM_SYSCOMMAND => {
                Result = OnCommand(GuiData, wParam, lParam);
            }

            WM_DROPFILES => {
                OnDropFiles(Console, wParam as HDROP);
            }

            WM_SETFOCUS | WM_KILLFOCUS => {
                OnFocus(GuiData, if msg == WM_SETFOCUS { 1 } else { 0 });
            }

            WM_GETMINMAXINFO => {
                OnGetMinMaxInfo(GuiData, lParam as PMINMAXINFO);
            }

            WM_MOVE => {
                OnMove(GuiData);
            }

            WM_SIZE => {
                OnSize(GuiData, wParam, lParam);
            }

            PM_RESIZE_TERMINAL => {
                let Buff: PCONSOLE_SCREEN_BUFFER = (*GuiData).ActiveBuffer;
                let hDC: HDC;
                let hnew: HBITMAP;
                let hold: *mut core::ffi::c_void;

                let Width: u32;
                let Height: u32;
                let mut WidthUnit: u32 = 0;
                let mut HeightUnit: u32 = 0;

                if (*GuiData).IsWindowVisible == 0 {
                    break;
                }

                GetScreenBufferSizeUnits(Buff, GuiData, &mut WidthUnit, &mut HeightUnit);

                Width = (*Buff).ScreenBufferSize.X as u32 * WidthUnit;
                Height = (*Buff).ScreenBufferSize.Y as u32 * HeightUnit;

                hDC = GetDC((*GuiData).hWindow);
                hnew = CreateFrameBufferBitmap(hDC, Width as i32, Height as i32);
                ReleaseDC((*GuiData).hWindow, hDC);
                hold = SelectObject((*GuiData).hMemDC, hnew as *mut core::ffi::c_void);
                if !(*GuiData).hBitmap.is_null() {
                    if hold == (*GuiData).hBitmap as *mut core::ffi::c_void {
                        DeleteObject((*GuiData).hBitmap as *mut core::ffi::c_void);
                    }
                }
                (*GuiData).hBitmap = hnew;

                (*GuiData).WindowSizeLock = 1;
                ResizeConWnd(GuiData, WidthUnit, HeightUnit);
                (*GuiData).WindowSizeLock = 0;
            }

            _WM_SETCONSOLEINFO => {
                GuiApplyUserSettings(GuiData, wParam as *mut core::ffi::c_void);
            }

            PM_CONSOLE_BEEP => {
                // DPRINT removed
                Beep(800, 200);
            }

            PM_CONSOLE_SET_TITLE => {
                SetWindowTextW((*GuiData).hWindow, (*(*GuiData).Console).Title.Buffer);
            }

            _ => {
                break 'Default;
            }
        }
        return Result;
    }

    Result = DefWindowProcW(hWnd, msg, wParam, lParam);
    Result
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS Console Server DLL
//  * FILE:            win32ss/user/winsrv/consrv/frontends/gui/conwnd.c
//  * PURPOSE:         GUI Console Window Class
//  * PROGRAMMERS:     Gé van Geldorp
//  *                  Johannes Anderwald
//  *                  Jeffrey Morlan
//  *                  Hermes Belusca-Maito (hermes.belusca@sfr.fr)
//  *                  Katayama Hirofumi MZ (katayama.hirofumi.mz@gmail.com)
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include <consrv.h>
// #include <intrin.h>
// #include <windowsx.h>
// #include <shellapi.h>
//
// #define NDEBUG
// #include <debug.h>
//
// #include "concfg/font.h"
// #include "guiterm.h"
// #include "resource.h"
//
// /* GLOBALS ********************************************************************/
//
// // #define PM_CREATE_CONSOLE       (WM_APP + 1)
// // #define PM_DESTROY_CONSOLE      (WM_APP + 2)
//
// // See guiterm.c
// #define CONGUI_MIN_WIDTH      10
// #define CONGUI_MIN_HEIGHT     10
// #define CONGUI_UPDATE_TIME    0
// #define CONGUI_UPDATE_TIMER   1
//
// #define CURSOR_BLINK_TIME 500
//
//
// /**************************************************************\
// \** Define the Console Leader Process for the console window **/
// #define GWLP_CONWND_ALLOC      (2 * sizeof(LONG_PTR))
// #define GWLP_CONSOLE_LEADER_PID 0
// #define GWLP_CONSOLE_LEADER_TID 4
//
// VOID
// SetConWndConsoleLeaderCID(IN PGUI_CONSOLE_DATA GuiData)
// {
//     PCONSOLE_PROCESS_DATA ProcessData;
//
//     ProcessData = ConSrvGetConsoleLeaderProcess(GuiData->Console);
//
//     ASSERT(ProcessData != NULL);
//     DPRINT("ProcessData: %p, ProcessData->Process %p.\n", ProcessData, ProcessData->Process);
//
//     if (ProcessData->Process)
//     {
//         CLIENT_ID ConsoleLeaderCID = ProcessData->Process->ClientId;
//         SetWindowLongPtrW(GuiData->hWindow, GWLP_CONSOLE_LEADER_PID,
//                           (LONG_PTR)(ConsoleLeaderCID.UniqueProcess));
//         SetWindowLongPtrW(GuiData->hWindow, GWLP_CONSOLE_LEADER_TID,
//                           (LONG_PTR)(ConsoleLeaderCID.UniqueThread));
//     }
//     else
//     {
//         SetWindowLongPtrW(GuiData->hWindow, GWLP_CONSOLE_LEADER_PID, 0);
//         SetWindowLongPtrW(GuiData->hWindow, GWLP_CONSOLE_LEADER_TID, 0);
//     }
// }
// /**************************************************************/
//
// HICON   ghDefaultIcon = NULL;
// HICON   ghDefaultIconSm = NULL;
// HCURSOR ghDefaultCursor = NULL;
//
// typedef struct _GUICONSOLE_MENUITEM
// {
//     UINT uID;
//     const struct _GUICONSOLE_MENUITEM *SubMenu;
//     WORD wCmdID;
// } GUICONSOLE_MENUITEM, *PGUICONSOLE_MENUITEM;
//
// static const GUICONSOLE_MENUITEM GuiConsoleEditMenuItems[] =
// {
//     { IDS_MARK,         NULL, ID_SYSTEM_EDIT_MARK       },
//     { IDS_COPY,         NULL, ID_SYSTEM_EDIT_COPY       },
//     { IDS_PASTE,        NULL, ID_SYSTEM_EDIT_PASTE      },
//     { IDS_SELECTALL,    NULL, ID_SYSTEM_EDIT_SELECTALL  },
//     { IDS_SCROLL,       NULL, ID_SYSTEM_EDIT_SCROLL     },
//     { IDS_FIND,         NULL, ID_SYSTEM_EDIT_FIND       },
//
//     { 0, NULL, 0 } /* End of list */
// };
//
// static const GUICONSOLE_MENUITEM GuiConsoleMainMenuItems[] =
// {
//     { IDS_EDIT,         GuiConsoleEditMenuItems, 0 },
//     { IDS_DEFAULTS,     NULL, ID_SYSTEM_DEFAULTS   },
//     { IDS_PROPERTIES,   NULL, ID_SYSTEM_PROPERTIES },
//
//     { 0, NULL, 0 } /* End of list */
// };
//
// /*
//  * Default 16-color palette for foreground and background
//  * (corresponding flags in comments).
//  */
// const COLORREF s_Colors[16] =
// {
//     RGB(0, 0, 0),       // (Black)
//     RGB(0, 0, 128),     // BLUE
//     RGB(0, 128, 0),     // GREEN
//     RGB(0, 128, 128),   // BLUE  | GREEN
//     RGB(128, 0, 0),     // RED
//     RGB(128, 0, 128),   // BLUE  | RED
//     RGB(128, 128, 0),   // GREEN | RED
//     RGB(192, 192, 192), // BLUE  | GREEN | RED
//
//     RGB(128, 128, 128), // (Grey)  INTENSITY
//     RGB(0, 0, 255),     // BLUE  | INTENSITY
//     RGB(0, 255, 0),     // GREEN | INTENSITY
//     RGB(0, 255, 255),   // BLUE  | GREEN | INTENSITY
//     RGB(255, 0, 0),     // RED   | INTENSITY
//     RGB(255, 0, 255),   // BLUE  | RED   | INTENSITY
//     RGB(255, 255, 0),   // GREEN | RED   | INTENSITY
//     RGB(255, 255, 255)  // BLUE  | GREEN | RED | INTENSITY
// };
//
// /* FUNCTIONS ******************************************************************/
//
// static LRESULT CALLBACK
// ConWndProc(HWND hWnd, UINT msg, WPARAM wParam, LPARAM lParam);
//
// BOOLEAN
// RegisterConWndClass(IN HINSTANCE hInstance)
// {
//     WNDCLASSEXW WndClass;
//     ATOM WndClassAtom;
//
//     ghDefaultIcon   = LoadImageW(hInstance,
//                                  MAKEINTRESOURCEW(IDI_TERMINAL),
//                                  IMAGE_ICON,
//                                  GetSystemMetrics(SM_CXICON as i32),
//                                  GetSystemMetrics(SM_CYICON as i32),
//                                  LR_SHARED);
//     ghDefaultIconSm = LoadImageW(hInstance,
//                                  MAKEINTRESOURCEW(IDI_TERMINAL),
//                                  IMAGE_ICON,
//                                  GetSystemMetrics(SM_CXSMICON as i32),
//                                  GetSystemMetrics(SM_CYSMICON as i32),
//                                  LR_SHARED);
//     ghDefaultCursor = LoadCursorW(NULL, IDC_ARROW);
//
//     WndClass.cbSize = sizeof(WNDCLASSEXW);
//     WndClass.lpszClassName = GUI_CONWND_CLASS_W.as_ptr();
//     WndClass.lpfnWndProc = ConWndProc;
//     WndClass.style = CS_DBLCLKS /* | CS_HREDRAW | CS_VREDRAW */;
//     WndClass.hInstance = hInstance;
//     WndClass.hIcon = ghDefaultIcon;
//     WndClass.hIconSm = ghDefaultIconSm;
//     WndClass.hCursor = ghDefaultCursor;
//     WndClass.hbrBackground = NULL;
//     WndClass.lpszMenuName = NULL;
//     WndClass.cbClsExtra = 0;
//     WndClass.cbWndExtra = GWLP_CONWND_ALLOC as i32;
//
//     WndClassAtom = RegisterClassExW(&WndClass);
//     if (WndClassAtom == 0)
//     {
//         DPRINT1("Failed to register GUI console class\n");
//     }
//     else
//     {
//         NtUserConsoleControl(GuiConsoleWndClassAtom, &WndClassAtom, sizeof(ATOM));
//     }
//
//     return (WndClassAtom != 0);
// }
//
// BOOLEAN
// UnRegisterConWndClass(HINSTANCE hInstance)
// {
//     return !!UnregisterClassW(GUI_CONWND_CLASS_W.as_ptr(), hInstance);
// }
//
// static VOID
// AppendMenuItems(HMENU hMenu,
//                 const GUICONSOLE_MENUITEM *Items)
// {
//     UINT i = 0;
//     WCHAR szMenuString[255];
//     HMENU hSubMenu;
//
//     do
//     {
//         if (Items[i].uID != (UINT)-1)
//         {
//             if (LoadStringW(ConSrvDllInstance,
//                             Items[i].uID,
//                             szMenuString,
//                             ARRAYSIZE(szMenuString)) > 0)
//             {
//                 if (Items[i].SubMenu != NULL)
//                 {
//                     hSubMenu = CreatePopupMenu();
//                     if (hSubMenu != NULL)
//                     {
//                         AppendMenuItems(hSubMenu, Items[i].SubMenu);
//
//                         if (!AppendMenuW(hMenu,
//                                          MF_STRING | MF_POPUP,
//                                          (UINT_PTR)hSubMenu,
//                                          szMenuString))
//                         {
//                             DestroyMenu(hSubMenu);
//                         }
//                     }
//                 }
//                 else
//                 {
//                     AppendMenuW(hMenu,
//                                 MF_STRING,
//                                 Items[i].wCmdID,
//                                 szMenuString);
//                 }
//             }
//         }
//         else
//         {
//             AppendMenuW(hMenu,
//                         MF_SEPARATOR,
//                         0,
//                         NULL);
//         }
//         i++;
//     } while (!(Items[i].uID == 0 && Items[i].SubMenu == NULL && Items[i].wCmdID == 0));
// }
//
// //static
// VOID
// CreateSysMenu(HWND hWnd)
// {
//     MENUITEMINFOW mii;
//     HMENU hMenu;
//     PWCHAR ptrTab;
//     WCHAR szMenuStringBack[255];
//
//     hMenu = GetSystemMenu(hWnd, FALSE);
//     if (hMenu == NULL)
//         return;
//
//     mii.cbSize = sizeof(mii);
//     mii.fMask = MIIM_STRING;
//     mii.dwTypeData = szMenuStringBack;
//     mii.cch = ARRAYSIZE(szMenuStringBack);
//
//     GetMenuItemInfoW(hMenu, SC_CLOSE, FALSE, &mii);
//
//     ptrTab = wcschr(szMenuStringBack, L'\t');
//     if (ptrTab)
//     {
//         *ptrTab = L'\0';
//         mii.cch = (UINT)wcslen(szMenuStringBack);
//
//         SetMenuItemInfoW(hMenu, SC_CLOSE, FALSE, &mii);
//     }
//
//     AppendMenuItems(hMenu, GuiConsoleMainMenuItems);
//     DrawMenuBar(hWnd);
// }
//
// static VOID
// SendMenuEvent(PCONSRV_CONSOLE Console, UINT CmdId)
// {
//     INPUT_RECORD er;
//
//     DPRINT("Menu item ID: %d\n", CmdId);
//
//     if (!ConDrvValidateConsoleUnsafe((PCONSOLE)Console, CONSOLE_RUNNING, TRUE)) return;
//
//     /* Send a menu event */
//     er.EventType = MENU_EVENT as u16;
//     er.Event.MenuEvent.dwCommandId = CmdId;
//     ConioProcessInputEvent(Console, &er);
//
//     LeaveCriticalSection(&Console->Lock);
// }
//
// static VOID
// Copy(PGUI_CONSOLE_DATA GuiData);
// static VOID
// Paste(PGUI_CONSOLE_DATA GuiData);
// static VOID
// UpdateSelection(PGUI_CONSOLE_DATA GuiData,
//                 PCOORD SelectionAnchor OPTIONAL,
//                 PCOORD coord);
//
// static VOID
// Mark(PGUI_CONSOLE_DATA GuiData)
// {
//     PCONSOLE_SCREEN_BUFFER ActiveBuffer = GuiData->ActiveBuffer;
//
//     /* Clear the old selection */
//     GuiData->Selection.dwFlags = CONSOLE_NO_SELECTION;
//
//     /* Restart a new selection */
//     GuiData->dwSelectionCursor = ActiveBuffer->ViewOrigin;
//     UpdateSelection(GuiData,
//                     &GuiData->dwSelectionCursor,
//                     &GuiData->dwSelectionCursor);
// }
//
// static VOID
// SelectAll(PGUI_CONSOLE_DATA GuiData)
// {
//     PCONSOLE_SCREEN_BUFFER ActiveBuffer = GuiData->ActiveBuffer;
//     COORD SelectionAnchor;
//
//     /* Clear the old selection */
//     GuiData->Selection.dwFlags = CONSOLE_NO_SELECTION;
//
//     /*
//      * The selection area extends to the whole screen buffer's width.
//      */
//     SelectionAnchor.X = SelectionAnchor.Y = 0;
//     GuiData->dwSelectionCursor.X = ActiveBuffer->ScreenBufferSize.X - 1;
//
//     /*
//      * Determine whether the selection must extend to just some part
//      * (for text-mode screen buffers) or to all of the screen buffer's
//      * height (for graphics ones).
//      */
//     if (GetType(ActiveBuffer) == TEXTMODE_BUFFER)
//     {
//         /*
//          * We select all the characters from the first line
//          * to the line where the cursor is positioned.
//          */
//         GuiData->dwSelectionCursor.Y = ActiveBuffer->CursorPosition.Y;
//     }
//     else /* if (GetType(ActiveBuffer) == GRAPHICS_BUFFER) */
//     {
//         /*
//          * We select all the screen buffer area.
//          */
//         GuiData->dwSelectionCursor.Y = ActiveBuffer->ScreenBufferSize.Y - 1;
//     }
//
//     /* Restart a new selection */
//     GuiData->Selection.dwFlags |= CONSOLE_MOUSE_SELECTION;
//     UpdateSelection(GuiData, &SelectionAnchor, &GuiData->dwSelectionCursor);
// }
//
// static LRESULT
// OnCommand(PGUI_CONSOLE_DATA GuiData, WPARAM wParam, LPARAM lParam)
// {
//     LRESULT Ret = TRUE;
//     PCONSRV_CONSOLE Console = GuiData->Console;
//
//     /*
//      * In case the selected menu item belongs to the user-reserved menu id range,
//      * send to him a menu event and return directly. The user must handle those
//      * reserved menu commands...
//      */
//     if (GuiData->CmdIdLow <= (UINT)wParam && (UINT)wParam <= GuiData->CmdIdHigh)
//     {
//         SendMenuEvent(Console, (UINT)wParam);
//         goto Quit;
//     }
//
//     /* ... otherwise, perform actions. */
//     switch (wParam)
//     {
//         case ID_SYSTEM_EDIT_MARK:
//             Mark(GuiData);
//             break;
//
//         case ID_SYSTEM_EDIT_COPY:
//             Copy(GuiData);
//             break;
//
//         case ID_SYSTEM_EDIT_PASTE:
//             Paste(GuiData);
//             break;
//
//         case ID_SYSTEM_EDIT_SELECTALL:
//             SelectAll(GuiData);
//             break;
//
//         case ID_SYSTEM_EDIT_SCROLL:
//             DPRINT1("Scrolling is not handled yet\n");
//             break;
//
//         case ID_SYSTEM_EDIT_FIND:
//             DPRINT1("Finding is not handled yet\n");
//             break;
//
//         case ID_SYSTEM_DEFAULTS:
//             GuiConsoleShowConsoleProperties(GuiData, TRUE);
//             break;
//
//         case ID_SYSTEM_PROPERTIES:
//             GuiConsoleShowConsoleProperties(GuiData, FALSE);
//             break;
//
//         default:
//             Ret = FALSE;
//             break;
//     }
//
// Quit:
//     if (!Ret)
//         Ret = DefWindowProcW(GuiData->hWindow, WM_SYSCOMMAND, wParam, lParam);
//
//     return Ret;
// }
//
// static PGUI_CONSOLE_DATA
// GuiGetGuiData(HWND hWnd)
// {
//     /* This function ensures that the console pointer is not NULL */
//     PGUI_CONSOLE_DATA GuiData = (PGUI_CONSOLE_DATA)GetWindowLongPtrW(hWnd, GWLP_USERDATA);
//     return ( ((GuiData == NULL) || (GuiData->hWindow == hWnd && GuiData->Console != NULL)) ? GuiData : NULL );
// }
//
// static VOID
// ResizeConWnd(PGUI_CONSOLE_DATA GuiData, DWORD WidthUnit, DWORD HeightUnit)
// {
//     PCONSOLE_SCREEN_BUFFER Buff = GuiData->ActiveBuffer;
//     SCROLLINFO sInfo;
//
//     DWORD Width, Height;
//
//     Width  = Buff->ViewSize.X * WidthUnit  +
//              2 * (GetSystemMetrics(SM_CXFRAME as i32) + GetSystemMetrics(SM_CXEDGE as i32));
//     Height = Buff->ViewSize.Y * HeightUnit +
//              2 * (GetSystemMetrics(SM_CYFRAME as i32) + GetSystemMetrics(SM_CYEDGE as i32)) + GetSystemMetrics(SM_CYCAPTION as i32);
//
//     /* Set scrollbar sizes */
//     sInfo.cbSize = sizeof(sInfo);
//     sInfo.fMask = SIF_RANGE | SIF_PAGE | SIF_POS;
//     sInfo.nMin = 0;
//     if (Buff->ScreenBufferSize.Y > Buff->ViewSize.Y)
//     {
//         sInfo.nMax  = Buff->ScreenBufferSize.Y - 1;
//         sInfo.nPage = Buff->ViewSize.Y;
//         sInfo.nPos  = Buff->ViewOrigin.Y;
//         SetScrollInfo(GuiData->hWindow, SB_VERT as i32, &sInfo, TRUE);
//         Width += GetSystemMetrics(SM_CXVSCROLL as i32);
//         ShowScrollBar(GuiData->hWindow, SB_VERT as i32, TRUE);
//     }
//     else
//     {
//         ShowScrollBar(GuiData->hWindow, SB_VERT as i32, FALSE);
//     }
//
//     if (Buff->ScreenBufferSize.X > Buff->ViewSize.X)
//     {
//         sInfo.nMax  = Buff->ScreenBufferSize.X - 1;
//         sInfo.nPage = Buff->ViewSize.X;
//         sInfo.nPos  = Buff->ViewOrigin.X;
//         SetScrollInfo(GuiData->hWindow, SB_HORZ as i32, &sInfo, TRUE);
//         Height += GetSystemMetrics(SM_CYHSCROLL as i32);
//         ShowScrollBar(GuiData->hWindow, SB_HORZ as i32, TRUE);
//     }
//     else
//     {
//         ShowScrollBar(GuiData->hWindow, SB_HORZ as i32, FALSE);
//     }
//
//     /* Resize the window */
//     SetWindowPos(GuiData->hWindow, NULL, 0, 0, Width, Height,
//                  SWP_NOZORDER | SWP_NOMOVE | SWP_NOACTIVATE | SWP_NOCOPYBITS);
//     // NOTE: The SWP_NOCOPYBITS flag can be replaced by a subsequent call
//     // to: InvalidateRect(GuiData->hWindow, NULL, TRUE);
// }
//
//
// VOID
// DeleteFonts(PGUI_CONSOLE_DATA GuiData)
// {
//     ULONG i;
//     for (i = 0; i < ARRAYSIZE(GuiData->Font); ++i)
//     {
//         if (GuiData->Font[i] != NULL) DeleteObject(GuiData->Font[i]);
//         GuiData->Font[i] = NULL;
//     }
// }
//
// static HFONT
// CreateDerivedFont(HFONT OrgFont,
//                   // COORD   FontSize,
//                   ULONG   FontWeight,
//                   // BOOLEAN bItalic,
//                   BOOLEAN bUnderline,
//                   BOOLEAN bStrikeOut)
// {
//     LOGFONTW lf;
//
//     /* Initialize the LOGFONT structure */
//     RtlZeroMemory(&lf, sizeof(lf));
//
//     /* Retrieve the details of the current font */
//     if (GetObjectW(OrgFont as *mut core::ffi::c_void, sizeof(lf), &lf) == 0)
//         return NULL;
//
//     /* Change the font attributes */
//     // lf.lfHeight = FontSize.Y;
//     // lf.lfWidth  = FontSize.X;
//     lf.lfWeight = FontWeight;
//     // lf.lfItalic = bItalic;
//     lf.lfUnderline = bUnderline;
//     lf.lfStrikeOut = bStrikeOut;
//
//     /* Build a new font */
//     return CreateFontIndirectW(&lf);
// }
//
// BOOL
// InitFonts(
//     _Inout_ PGUI_CONSOLE_DATA GuiData,
//     _In_reads_or_z_(LF_FACESIZE)
//          PCWSTR FaceName,
//     _In_ ULONG FontWeight,
//     _In_ ULONG FontFamily,
//     _In_ COORD FontSize,
//     _In_opt_ UINT CodePage,
//     _In_ BOOL UseDefaultFallback)
// {
//     HDC hDC;
//     HFONT hFont;
//     FONT_DATA FontData;
//     UINT OldCharWidth  = GuiData->CharWidth;
//     UINT OldCharHeight = GuiData->CharHeight;
//     COORD OldFontSize  = GuiData->GuiInfo.FontSize;
//     WCHAR NewFaceName[LF_FACESIZE];
//
//     /* Default to current code page if none has been provided */
//     if (!CodePage)
//         CodePage = GuiData->Console->OutputCodePage;
//
//     /*
//      * Initialize a new NORMAL font.
//      */
//
//     /* Copy the requested face name into the local buffer.
//      * It will be modified in output by CreateConsoleFontEx()
//      * to hold a possible fallback font face name. */
//     StringCchCopyNW(NewFaceName, ARRAYSIZE(NewFaceName),
//                     FaceName, LF_FACESIZE);
//
//     /* NOTE: FontSize is always in cell height/width units (pixels) */
//     hFont = CreateConsoleFontEx((LONG)(ULONG)FontSize.Y,
//                                 (LONG)(ULONG)FontSize.X,
//                                 NewFaceName,
//                                 FontWeight,
//                                 FontFamily,
//                                 CodePage,
//                                 UseDefaultFallback,
//                                 &FontData);
//     if (!hFont)
//     {
//         DPRINT1("InitFonts: CreateConsoleFontEx('%S') failed\n", NewFaceName);
//         return FALSE;
//     }
//
//     /* Retrieve its character cell size */
//     hDC = GetDC(GuiData->hWindow);
//     if (!GetFontCellSize(hDC, hFont, &GuiData->CharHeight, &GuiData->CharWidth))
//     {
//         DPRINT1("InitFonts: GetFontCellSize failed\n");
//         ReleaseDC(GuiData->hWindow, hDC);
//         DeleteObject(hFont as *mut core::ffi::c_void);
//         return FALSE;
//     }
//     ReleaseDC(GuiData->hWindow, hDC);
//
//     /*
//      * Initialization succeeded.
//      */
//     // Delete all the old fonts first.
//     DeleteFonts(GuiData);
//     GuiData->Font[FONT_NORMAL] = hFont;
//
//     /*
//      * Now build the optional fonts (bold, underlined, mixed).
//      * Do not error in case they fail to be created.
//      */
//     GuiData->Font[FONT_BOLD] =
//         CreateDerivedFont(GuiData->Font[FONT_NORMAL],
//                           max(FW_BOLD, FontData.Weight),
//                           FALSE,
//                           FALSE);
//     GuiData->Font[FONT_UNDERLINE] =
//         CreateDerivedFont(GuiData->Font[FONT_NORMAL],
//                           FontData.Weight,
//                           TRUE,
//                           FALSE);
//     GuiData->Font[FONT_BOLD | FONT_UNDERLINE] =
//         CreateDerivedFont(GuiData->Font[FONT_NORMAL],
//                           max(FW_BOLD, FontData.Weight),
//                           TRUE,
//                           FALSE);
//
//     /*
//      * Save the new font characteristics.
//      */
//     StringCchCopyNW(GuiData->GuiInfo.FaceName,
//                     ARRAYSIZE(GuiData->GuiInfo.FaceName),
//                     NewFaceName, ARRAYSIZE(NewFaceName));
//     GuiData->GuiInfo.FontWeight = FontData.Weight;
//     GuiData->GuiInfo.FontFamily = FontData.Family;
//     GuiData->GuiInfo.FontSize   = FontData.Size;
//
//     /* Resize the terminal, in case the new font has a different size */
//     if ((OldCharWidth  != GuiData->CharWidth)  ||
//         (OldCharHeight != GuiData->CharHeight) ||
//         (OldFontSize.X != FontData.Size.X ||
//          OldFontSize.Y != FontData.Size.Y))
//     {
//         TermResizeTerminal(GuiData->Console);
//     }
//
//     return TRUE;
// }
//
//
// static BOOL
// OnNcCreate(HWND hWnd, LPCREATESTRUCTW Create)
// {
//     PGUI_CONSOLE_DATA GuiData = (PGUI_CONSOLE_DATA)Create->lpCreateParams;
//     PCONSRV_CONSOLE Console;
//
//     if (GuiData == NULL)
//     {
//         DPRINT1("GuiConsoleNcCreate: No GUI data\n");
//         return FALSE;
//     }
//
//     Console = GuiData->Console;
//
//     GuiData->hWindow = hWnd;
//     GuiData->hSysMenu = GetSystemMenu(hWnd, FALSE);
//     GuiData->IsWindowActive = FALSE;
//
//     /* Initialize the fonts */
//     if (!InitFonts(GuiData,
//                    GuiData->GuiInfo.FaceName,
//                    GuiData->GuiInfo.FontWeight,
//                    GuiData->GuiInfo.FontFamily,
//                    GuiData->GuiInfo.FontSize,
//                    0, FALSE))
//     {
//         /* Reset only the output code page if we don't have a suitable
//          * font for it, possibly falling back to "United States (OEM)". */
//         UINT AltCodePage = GetOEMCP();
//
//         if (AltCodePage == Console->OutputCodePage)
//             AltCodePage = CP_USA;
//
//         DPRINT1("Could not initialize font '%S' for code page %d - Resetting CP to %d\n",
//                 GuiData->GuiInfo.FaceName, Console->OutputCodePage, AltCodePage);
//
//         CON_SET_OUTPUT_CP(Console, AltCodePage);
//
//         /* We will use a fallback font if we cannot find
//          * anything for this replacement code page. */
//         if (!InitFonts(GuiData,
//                        GuiData->GuiInfo.FaceName,
//                        GuiData->GuiInfo.FontWeight,
//                        GuiData->GuiInfo.FontFamily,
//                        GuiData->GuiInfo.FontSize,
//                        0, TRUE))
//         {
//             DPRINT1("Failed to initialize font '%S' for code page %d\n",
//                     GuiData->GuiInfo.FaceName, Console->OutputCodePage);
//
//             DPRINT1("GuiConsoleNcCreate: InitFonts failed\n");
//             GuiData->hWindow = NULL;
//             NtSetEvent(GuiData->hGuiInitEvent, NULL);
//             return FALSE;
//         }
//     }
//
//     /* Initialize the terminal framebuffer */
//     GuiData->hMemDC  = CreateCompatibleDC(NULL);
//     GuiData->hBitmap = NULL;
//     GuiData->hSysPalette = NULL; /* Original system palette */
//
//     /* Update the icons of the window */
//     if (GuiData->hIcon != ghDefaultIcon)
//     {
//         DefWindowProcW(GuiData->hWindow, WM_SETICON, ICON_BIG  , (LPARAM)GuiData->hIcon  );
//         DefWindowProcW(GuiData->hWindow, WM_SETICON, ICON_SMALL, (LPARAM)GuiData->hIconSm);
//     }
//
//     // FIXME: Keep these instructions here ? ///////////////////////////////////
//     Console->ActiveBuffer->CursorBlinkOn = TRUE;
//     Console->ActiveBuffer->ForceCursorOff = FALSE;
//     ////////////////////////////////////////////////////////////////////////////
//
//     SetWindowLongPtrW(GuiData->hWindow, GWLP_USERDATA, (DWORD_PTR)GuiData);
//
//     if (GuiData->IsWindowVisible)
//     {
//         SetTimer(GuiData->hWindow, CONGUI_UPDATE_TIMER, CONGUI_UPDATE_TIME, NULL);
//     }
//
//     // FIXME: HACK: Potential HACK for CORE-8129; see revision 63595.
//     //CreateSysMenu(GuiData->hWindow);
//
//     DPRINT("OnNcCreate - setting start event\n");
//     NtSetEvent(GuiData->hGuiInitEvent, NULL);
//
//     /* We accept dropped files */
//     DragAcceptFiles(GuiData->hWindow, TRUE);
//
//     return (BOOL)DefWindowProcW(GuiData->hWindow, WM_NCCREATE, 0, (LPARAM)Create);
// }
//
// static VOID
// OnActivate(PGUI_CONSOLE_DATA GuiData, WPARAM wParam)
// {
//     WORD ActivationState = LOWORD(wParam);
//
//     DPRINT("WM_ACTIVATE - ActivationState = %d\n", ActivationState);
//
//     GuiData->IsWindowActive = (ActivationState != WA_INACTIVE);
//
//     if ( ActivationState == WA_ACTIVE ||
//          ActivationState == WA_CLICKACTIVE )
//     {
//         if (GuiData->GuiInfo.FullScreen)
//         {
//             EnterFullScreen(GuiData);
//             // // PostMessageW(GuiData->hWindow, WM_SYSCOMMAND, SC_RESTORE as i32, 0);
//             // SendMessageW(GuiData->hWindow, WM_SYSCOMMAND, SC_RESTORE as i32, 0);
//         }
//     }
//     else // if (ActivationState == WA_INACTIVE)
//     {
//         if (GuiData->GuiInfo.FullScreen)
//         {
//             SendMessageW(GuiData->hWindow, WM_SYSCOMMAND, SC_MINIMIZE as i32, 0);
//             LeaveFullScreen(GuiData);
//             // // PostMessageW(GuiData->hWindow, WM_SYSCOMMAND, SC_MINIMIZE as i32, 0);
//             // SendMessageW(GuiData->hWindow, WM_SYSCOMMAND, SC_MINIMIZE as i32, 0);
//         }
//     }
//
//     /*
//      * Ignore the next mouse event when we are going to be enabled again via
//      * the mouse, in order to prevent, e.g. when we are in Edit mode, erroneous
//      * mouse actions from the user that could spoil text selection or copy/pastes.
//      */
//     if (ActivationState == WA_CLICKACTIVE)
//         GuiData->IgnoreNextMouseEvent = TRUE;
// }
//
// static VOID
// OnFocus(PGUI_CONSOLE_DATA GuiData, BOOL SetFocus)
// {
//     PCONSRV_CONSOLE Console = GuiData->Console;
//     INPUT_RECORD er;
//
//     if (!ConDrvValidateConsoleUnsafe((PCONSOLE)Console, CONSOLE_RUNNING, TRUE)) return;
//
//     /* Set console focus state */
//     Console->HasFocus = SetFocus;
//
//     /*
//      * Set the priority of the processes of this console
//      * in accordance with the console focus state.
//      */
//     ConSrvSetConsoleProcessFocus(Console, SetFocus as u8);
//
//     /* Send a focus event */
//     er.EventType = FOCUS_EVENT as u16;
//     er.Event.FocusEvent.bSetFocus = SetFocus;
//     ConioProcessInputEvent(Console, &er);
//
//     LeaveCriticalSection(&Console->Lock);
//
//     if (SetFocus)
//         DPRINT("TODO: Create console caret\n");
//     else
//         DPRINT("TODO: Destroy console caret\n");
// }
//
// VOID
// GetSelectionBeginEnd(PCOORD Begin, PCOORD End,
//                      PCOORD SelectionAnchor,
//                      PSMALL_RECT SmallRect)
// {
//     if (Begin == NULL || End == NULL) return;
//
//     *Begin = *SelectionAnchor;
//     End->X = (SelectionAnchor->X == SmallRect->Left) ? SmallRect->Right
//               /* Case X != Left, must be == Right */ : SmallRect->Left;
//     End->Y = (SelectionAnchor->Y == SmallRect->Top ) ? SmallRect->Bottom
//               /* Case Y != Top, must be == Bottom */ : SmallRect->Top;
//
//     /* Exchange Begin / End if Begin > End lexicographically */
//     if (Begin->Y > End->Y || (Begin->Y == End->Y && Begin->X > End->X))
//     {
//         End->X = _InterlockedExchange16(&Begin->X, End->X);
//         End->Y = _InterlockedExchange16(&Begin->Y, End->Y);
//     }
// }
//
// static HRGN
// CreateSelectionRgn(PGUI_CONSOLE_DATA GuiData,
//                    BOOL LineSelection,
//                    PCOORD SelectionAnchor,
//                    PSMALL_RECT SmallRect)
// {
//     if (!LineSelection)
//     {
//         RECT rect;
//         SmallRectToRect(GuiData, &rect, SmallRect);
//         return CreateRectRgnIndirect(&rect);
//     }
//     else
//     {
//         HRGN SelRgn;
//         COORD Begin, End;
//
//         GetSelectionBeginEnd(&Begin, &End, SelectionAnchor, SmallRect);
//
//         if (Begin.Y == End.Y)
//         {
//             SMALL_RECT sr;
//             RECT       r ;
//
//             sr.Left   = Begin.X;
//             sr.Top    = Begin.Y;
//             sr.Right  = End.X;
//             sr.Bottom = End.Y;
//
//             // Debug thingie to see whether I can put this corner case
//             // together with the previous one.
//             if (SmallRect->Left   != sr.Left  ||
//                 SmallRect->Top    != sr.Top   ||
//                 SmallRect->Right  != sr.Right ||
//                 SmallRect->Bottom != sr.Bottom)
//             {
//                 DPRINT1("\n"
//                         "SmallRect = (%d, %d, %d, %d)\n"
//                         "sr = (%d, %d, %d, %d)\n"
//                         "\n",
//                         SmallRect->Left, SmallRect->Top, SmallRect->Right, SmallRect->Bottom,
//                         sr.Left, sr.Top, sr.Right, sr.Bottom);
//             }
//
//             SmallRectToRect(GuiData, &r, &sr);
//             SelRgn = CreateRectRgnIndirect(&r);
//         }
//         else
//         {
//             PCONSOLE_SCREEN_BUFFER ActiveBuffer = GuiData->ActiveBuffer;
//
//             HRGN       rg1, rg2, rg3;
//             SMALL_RECT sr1, sr2, sr3;
//             RECT       r1 , r2 , r3 ;
//
//             sr1.Left   = Begin.X;
//             sr1.Top    = Begin.Y;
//             sr1.Right  = ActiveBuffer->ScreenBufferSize.X - 1;
//             sr1.Bottom = Begin.Y;
//
//             sr2.Left   = 0;
//             sr2.Top    = Begin.Y + 1;
//             sr2.Right  = ActiveBuffer->ScreenBufferSize.X - 1;
//             sr2.Bottom = End.Y - 1;
//
//             sr3.Left   = 0;
//             sr3.Top    = End.Y;
//             sr3.Right  = End.X;
//             sr3.Bottom = End.Y;
//
//             SmallRectToRect(GuiData, &r1, &sr1);
//             SmallRectToRect(GuiData, &r2, &sr2);
//             SmallRectToRect(GuiData, &r3, &sr3);
//
//             rg1 = CreateRectRgnIndirect(&r1);
//             rg2 = CreateRectRgnIndirect(&r2);
//             rg3 = CreateRectRgnIndirect(&r3);
//
//             CombineRgn(rg1, rg1, rg2, RGN_XOR as i32);
//             CombineRgn(rg1, rg1, rg3, RGN_XOR as i32);
//             DeleteObject(rg3 as *mut core::ffi::c_void);
//             DeleteObject(rg2 as *mut core::ffi::c_void);
//
//             SelRgn = rg1;
//         }
//
//         return SelRgn;
//     }
// }
//
// static VOID
// PaintSelectionRect(PGUI_CONSOLE_DATA GuiData, PPAINTSTRUCT pps)
// {
//     HRGN rgnPaint = CreateRectRgnIndirect(&pps->rcPaint);
//     HRGN rgnSel   = CreateSelectionRgn(GuiData, GuiData->LineSelection,
//                                        &GuiData->Selection.dwSelectionAnchor,
//                                        &GuiData->Selection.srSelection);
//
//     /* Invert the selection */
//
//     int ErrorCode = CombineRgn(rgnPaint, rgnPaint, rgnSel, RGN_AND as i32);
//     if (ErrorCode != ERROR as i32 && ErrorCode != NULLREGION as i32)
//     {
//         InvertRgn(pps->hdc, rgnPaint);
//     }
//
//     DeleteObject(rgnSel as *mut core::ffi::c_void);
//     DeleteObject(rgnPaint as *mut core::ffi::c_void);
// }
//
// static VOID
// UpdateSelection(PGUI_CONSOLE_DATA GuiData,
//                 PCOORD SelectionAnchor OPTIONAL,
//                 PCOORD coord)
// {
//     PCONSRV_CONSOLE Console = GuiData->Console;
//     HRGN oldRgn = CreateSelectionRgn(GuiData, GuiData->LineSelection,
//                                      &GuiData->Selection.dwSelectionAnchor,
//                                      &GuiData->Selection.srSelection);
//
//     /* Update the anchor if needed (use the old one if NULL) */
//     if (SelectionAnchor)
//         GuiData->Selection.dwSelectionAnchor = *SelectionAnchor;
//
//     // TODO: Scroll buffer to bring 'coord' into view
//
//     if (coord != NULL)
//     {
//         SMALL_RECT rc;
//         HRGN newRgn;
//
//         /*
//          * Pressing the Control key while selecting text, allows us to enter
//          * into line-selection mode, the selection mode of *nix terminals.
//          */
//         BOOL OldLineSel = GuiData->LineSelection;
//         GuiData->LineSelection = !!(GetKeyState(VK_CONTROL as i32) & KEY_PRESSED as i16);
//
//         /* Exchange left/top with right/bottom if required */
//         rc.Left   = min(GuiData->Selection.dwSelectionAnchor.X, coord->X);
//         rc.Top    = min(GuiData->Selection.dwSelectionAnchor.Y, coord->Y);
//         rc.Right  = max(GuiData->Selection.dwSelectionAnchor.X, coord->X);
//         rc.Bottom = max(GuiData->Selection.dwSelectionAnchor.Y, coord->Y);
//
//         newRgn = CreateSelectionRgn(GuiData, GuiData->LineSelection,
//                                     &GuiData->Selection.dwSelectionAnchor,
//                                     &rc);
//
//         if (GuiData->Selection.dwFlags & CONSOLE_SELECTION_NOT_EMPTY)
//         {
//             if (OldLineSel != GuiData->LineSelection ||
//                 memcmp(&rc, &GuiData->Selection.srSelection, sizeof(SMALL_RECT)) != 0)
//             {
//                 /* Calculate the region that needs to be updated */
//                 if (oldRgn && newRgn && CombineRgn(newRgn, newRgn, oldRgn, RGN_XOR as i32) != ERROR)
//                 {
//                     InvalidateRgn(GuiData->hWindow, newRgn, FALSE);
//                 }
//             }
//         }
//         else
//         {
//             InvalidateRgn(GuiData->hWindow, newRgn, FALSE);
//         }
//
//         DeleteObject(newRgn);
//
//         GuiData->Selection.dwFlags |= CONSOLE_SELECTION_NOT_EMPTY;
//         GuiData->Selection.srSelection = rc;
//         GuiData->dwSelectionCursor = *coord;
//
//         if ((GuiData->Selection.dwFlags & CONSOLE_SELECTION_IN_PROGRESS) == 0)
//         {
//             LPWSTR SelTypeStr = NULL   , WindowTitle = NULL;
//             SIZE_T SelTypeStrLength = 0, Length = 0;
//
//             /* Clear the old selection */
//             if (GuiData->Selection.dwFlags & CONSOLE_SELECTION_NOT_EMPTY)
//             {
//                 InvalidateRgn(GuiData->hWindow, oldRgn, FALSE);
//             }
//
//             /*
//              * When passing a zero-length buffer size, LoadString(...) returns
//              * a read-only pointer buffer to the program's resource string.
//              */
//             SelTypeStrLength =
//                 LoadStringW(ConSrvDllInstance,
//                             (GuiData->Selection.dwFlags & CONSOLE_MOUSE_SELECTION)
//                                 ? IDS_SELECT_TITLE : IDS_MARK_TITLE,
//                             (LPWSTR)&SelTypeStr, 0);
//
//             /*
//              * Prepend the selection type string to the current console title
//              * if we succeeded in retrieving a valid localized string.
//              */
//             if (SelTypeStr)
//             {
//                 // 3 for " - " and 1 for NULL
//                 Length = Console->Title.Length + (SelTypeStrLength + 3 + 1) * sizeof(WCHAR);
//                 WindowTitle = ConsoleAllocHeap(0, Length);
//
//                 wcsncpy(WindowTitle, SelTypeStr, SelTypeStrLength);
//                 WindowTitle[SelTypeStrLength] = UNICODE_NULL;
//                 wcscat(WindowTitle, L" - ");
//                 wcscat(WindowTitle, Console->Title.Buffer);
//
//                 SetWindowTextW(GuiData->hWindow, WindowTitle);
//                 ConsoleFreeHeap(WindowTitle);
//             }
//
//             GuiData->Selection.dwFlags |= CONSOLE_SELECTION_IN_PROGRESS;
//             ConioPause(Console, PAUSED_FROM_SELECTION);
//         }
//     }
//     else
//     {
//         /* Clear the selection */
//         if (GuiData->Selection.dwFlags & CONSOLE_SELECTION_NOT_EMPTY)
//         {
//             InvalidateRgn(GuiData->hWindow, oldRgn, FALSE);
//         }
//
//         GuiData->Selection.dwFlags = CONSOLE_NO_SELECTION;
//         ConioUnpause(Console, PAUSED_FROM_SELECTION);
//
//         /* Restore the console title */
//         SetWindowTextW(GuiData->hWindow, Console->Title.Buffer);
//     }
//
//     DeleteObject(oldRgn);
// }
//
// static VOID
// OnPaint(PGUI_CONSOLE_DATA GuiData)
// {
//     PCONSOLE_SCREEN_BUFFER ActiveBuffer = GuiData->ActiveBuffer;
//     PAINTSTRUCT ps;
//     RECT rcPaint;
//
//     /* Do nothing if the window is hidden */
//     if (!GuiData->IsWindowVisible) return;
//
//     BeginPaint(GuiData->hWindow, &ps);
//     if (ps.hdc != NULL &&
//         ps.rcPaint.left < ps.rcPaint.right &&
//         ps.rcPaint.top < ps.rcPaint.bottom)
//     {
//         EnterCriticalSection(&GuiData->Lock);
//
//         /* Compose the current screen-buffer on-memory */
//         if (GetType(ActiveBuffer) == TEXTMODE_BUFFER)
//         {
//             GuiPaintTextModeBuffer((PTEXTMODE_SCREEN_BUFFER)ActiveBuffer,
//                                    GuiData, core::ptr::addr_of_mut!(ps.rcPaint), &rcPaint);
//         }
//         else /* if (GetType(ActiveBuffer) == GRAPHICS_BUFFER) */
//         {
//             GuiPaintGraphicsBuffer((PGRAPHICS_SCREEN_BUFFER)ActiveBuffer,
//                                    GuiData, core::ptr::addr_of_mut!(ps.rcPaint), &rcPaint);
//         }
//
//         /* Send it to screen */
//         BitBlt(ps.hdc,
//                ps.rcPaint.left,
//                ps.rcPaint.top,
//                rcPaint.right  - rcPaint.left,
//                rcPaint.bottom - rcPaint.top,
//                GuiData->hMemDC,
//                rcPaint.left,
//                rcPaint.top,
//                SRCCOPY);
//
//         /* Draw the selection region if needed */
//         if (GuiData->Selection.dwFlags & CONSOLE_SELECTION_NOT_EMPTY)
//         {
//             PaintSelectionRect(GuiData, &ps);
//         }
//
//         // TODO: Move cursor display here!
//
//         LeaveCriticalSection(&GuiData->Lock);
//     }
//     EndPaint(GuiData->hWindow, &ps);
//
//     return;
// }
//
// static VOID
// OnPaletteChanged(PGUI_CONSOLE_DATA GuiData)
// {
//     PCONSOLE_SCREEN_BUFFER ActiveBuffer = GuiData->ActiveBuffer;
//
//     /* Do nothing if the window is hidden */
//     if (!GuiData->IsWindowVisible) return;
//
//     // See WM_PALETTECHANGED message
//     // if ((HWND)wParam == hWnd) break;
//
//     // if (GetType(ActiveBuffer) == GRAPHICS_BUFFER)
//     if (ActiveBuffer->PaletteHandle)
//     {
//         DPRINT("WM_PALETTECHANGED changing palette\n");
//
//         /* Specify the use of the system palette for the framebuffer */
//         SetSystemPaletteUse(GuiData->hMemDC, ActiveBuffer->PaletteUsage);
//
//         /* Realize the (logical) palette */
//         RealizePalette(GuiData->hMemDC);
//     }
// }
//
// static BOOL
// IsSystemKey(WORD VirtualKeyCode)
// {
//     switch (VirtualKeyCode)
//     {
//         /* From MSDN, "Virtual-Key Codes" */
//         case VK_RETURN:
//         case VK_SHIFT:
//         case VK_CONTROL:
//         case VK_MENU:
//         case VK_PAUSE:
//         case VK_CAPITAL:
//         case VK_ESCAPE:
//         case VK_LWIN:
//         case VK_RWIN:
//         case VK_NUMLOCK:
//         case VK_SCROLL:
//             return TRUE;
//         default:
//             return FALSE;
//     }
// }
//
// static VOID
// OnKey(PGUI_CONSOLE_DATA GuiData, UINT msg, WPARAM wParam, LPARAM lParam)
// {
//     PCONSRV_CONSOLE Console = GuiData->Console;
//     PCONSOLE_SCREEN_BUFFER ActiveBuffer;
//
//     if (!ConDrvValidateConsoleUnsafe((PCONSOLE)Console, CONSOLE_RUNNING, TRUE)) return;
//
//     ActiveBuffer = GuiData->ActiveBuffer;
//
//     if (GuiData->Selection.dwFlags & CONSOLE_SELECTION_IN_PROGRESS)
//     {
//         WORD VirtualKeyCode = LOWORD(wParam);
//
//         if (msg != WM_KEYDOWN) goto Quit;
//
//         if (VirtualKeyCode == VK_RETURN)
//         {
//             /* Copy (and clear) selection if ENTER is pressed */
//             Copy(GuiData);
//             goto Quit;
//         }
//         else if ( VirtualKeyCode == VK_ESCAPE ||
//                  (VirtualKeyCode == 'C' && (GetKeyState(VK_CONTROL as i32) & KEY_PRESSED as i16)) )
//         {
//             /* Cancel selection if ESC or Ctrl-C are pressed */
//             UpdateSelection(GuiData, NULL, NULL);
//             goto Quit;
//         }
//
//         if ((GuiData->Selection.dwFlags & CONSOLE_MOUSE_SELECTION) == 0)
//         {
//             /* Keyboard selection mode */
//             BOOL Interpreted = FALSE;
//             BOOL MajPressed  = !!(GetKeyState(VK_SHIFT as i32) & KEY_PRESSED as i16);
//
//             switch (VirtualKeyCode)
//             {
//                 case VK_LEFT:
//                 {
//                     Interpreted = TRUE;
//                     if (GuiData->dwSelectionCursor.X > 0)
//                         GuiData->dwSelectionCursor.X--;
//
//                     break;
//                 }
//
//                 case VK_RIGHT:
//                 {
//                     Interpreted = TRUE;
//                     if (GuiData->dwSelectionCursor.X < ActiveBuffer->ScreenBufferSize.X - 1)
//                         GuiData->dwSelectionCursor.X++;
//
//                     break;
//                 }
//
//                 case VK_UP:
//                 {
//                     Interpreted = TRUE;
//                     if (GuiData->dwSelectionCursor.Y > 0)
//                         GuiData->dwSelectionCursor.Y--;
//
//                     break;
//                 }
//
//                 case VK_DOWN:
//                 {
//                     Interpreted = TRUE;
//                     if (GuiData->dwSelectionCursor.Y < ActiveBuffer->ScreenBufferSize.Y - 1)
//                         GuiData->dwSelectionCursor.Y++;
//
//                     break;
//                 }
//
//                 case VK_HOME:
//                 {
//                     Interpreted = TRUE;
//                     GuiData->dwSelectionCursor.X = 0;
//                     GuiData->dwSelectionCursor.Y = 0;
//                     break;
//                 }
//
//                 case VK_END:
//                 {
//                     Interpreted = TRUE;
//                     GuiData->dwSelectionCursor.Y = ActiveBuffer->ScreenBufferSize.Y - 1;
//                     break;
//                 }
//
//                 case VK_PRIOR:
//                 {
//                     Interpreted = TRUE;
//                     GuiData->dwSelectionCursor.Y -= ActiveBuffer->ViewSize.Y;
//                     if (GuiData->dwSelectionCursor.Y < 0)
//                         GuiData->dwSelectionCursor.Y = 0;
//
//                     break;
//                 }
//
//                 case VK_NEXT:
//                 {
//                     Interpreted = TRUE;
//                     GuiData->dwSelectionCursor.Y += ActiveBuffer->ViewSize.Y;
//                     if (GuiData->dwSelectionCursor.Y >= ActiveBuffer->ScreenBufferSize.Y)
//                         GuiData->dwSelectionCursor.Y  = ActiveBuffer->ScreenBufferSize.Y - 1;
//
//                     break;
//                 }
//
//                 default:
//                     break;
//             }
//
//             if (Interpreted)
//             {
//                 UpdateSelection(GuiData,
//                                 !MajPressed ? &GuiData->dwSelectionCursor : NULL,
//                                 &GuiData->dwSelectionCursor);
//             }
//             else if (!IsSystemKey(VirtualKeyCode))
//             {
//                 /* Emit an error beep sound */
//                 SendNotifyMessage(GuiData->hWindow, PM_CONSOLE_BEEP, 0, 0);
//             }
//
//             goto Quit;
//         }
//         else
//         {
//             /* Mouse selection mode */
//
//             if (!IsSystemKey(VirtualKeyCode))
//             {
//                 /* Clear the selection and send the key into the input buffer */
//                 UpdateSelection(GuiData, NULL, NULL);
//             }
//             else
//             {
//                 goto Quit;
//             }
//         }
//     }
//
//     if ((GuiData->Selection.dwFlags & CONSOLE_SELECTION_IN_PROGRESS) == 0)
//     {
//         MSG Message;
//
//         Message.hwnd = GuiData->hWindow;
//         Message.message = msg;
//         Message.wParam = wParam;
//         Message.lParam = lParam;
//
//         ConioProcessKey(Console, &Message);
//     }
//
// Quit:
//     LeaveCriticalSection(&Console->Lock);
// }
//
//
// // FIXME: Remove after fixing OnTimer
// VOID
// InvalidateCell(PGUI_CONSOLE_DATA GuiData,
//                SHORT x, SHORT y);
//
// static VOID
// OnTimer(PGUI_CONSOLE_DATA GuiData)
// {
//     PCONSRV_CONSOLE Console = GuiData->Console;
//     PCONSOLE_SCREEN_BUFFER Buff;
//
//     /* Do nothing if the window is hidden */
//     if (!GuiData->IsWindowVisible) return;
//
//     SetTimer(GuiData->hWindow, CONGUI_UPDATE_TIMER, CURSOR_BLINK_TIME, NULL);
//
//     if (!ConDrvValidateConsoleUnsafe((PCONSOLE)Console, CONSOLE_RUNNING, TRUE)) return;
//
//     Buff = GuiData->ActiveBuffer;
//
//     if (GetType(Buff) == TEXTMODE_BUFFER)
//     {
//         /* Repaint the caret */
//         if (GuiData->IsWindowActive || Buff->CursorBlinkOn)
//         {
//             InvalidateCell(GuiData, Buff->CursorPosition.X, Buff->CursorPosition.Y);
//             Buff->CursorBlinkOn = !Buff->CursorBlinkOn;
//         }
//
//         if ((GuiData->OldCursor.x != Buff->CursorPosition.X) ||
//             (GuiData->OldCursor.y != Buff->CursorPosition.Y))
//         {
//             SCROLLINFO sInfo;
//             int OldScrollX = -1, OldScrollY = -1;
//             int NewScrollX = -1, NewScrollY = -1;
//
//             sInfo.cbSize = sizeof(sInfo);
//             sInfo.fMask = SIF_POS;
//             // Capture the original position of the scroll bars and save them.
//             if (GetScrollInfo(GuiData->hWindow, SB_HORZ as i32, &sInfo)) OldScrollX = sInfo.nPos;
//             if (GetScrollInfo(GuiData->hWindow, SB_VERT as i32, &sInfo)) OldScrollY = sInfo.nPos;
//
//             // If we successfully got the info for the horizontal scrollbar
//             if (OldScrollX >= 0)
//             {
//                 if ((Buff->CursorPosition.X < Buff->ViewOrigin.X) ||
//                     (Buff->CursorPosition.X >= (Buff->ViewOrigin.X + Buff->ViewSize.X)))
//                 {
//                     // Handle the horizontal scroll bar
//                     if (Buff->CursorPosition.X >= Buff->ViewSize.X)
//                         NewScrollX = Buff->CursorPosition.X - Buff->ViewSize.X + 1;
//                     else
//                         NewScrollX = 0;
//                 }
//                 else
//                 {
//                     NewScrollX = OldScrollX;
//                 }
//             }
//             // If we successfully got the info for the vertical scrollbar
//             if (OldScrollY >= 0)
//             {
//                 if ((Buff->CursorPosition.Y < Buff->ViewOrigin.Y) ||
//                     (Buff->CursorPosition.Y >= (Buff->ViewOrigin.Y + Buff->ViewSize.Y)))
//                 {
//                     // Handle the vertical scroll bar
//                     if (Buff->CursorPosition.Y >= Buff->ViewSize.Y)
//                         NewScrollY = Buff->CursorPosition.Y - Buff->ViewSize.Y + 1;
//                     else
//                         NewScrollY = 0;
//                 }
//                 else
//                 {
//                     NewScrollY = OldScrollY;
//                 }
//             }
//
//             // Adjust scroll bars and refresh the window if the cursor has moved outside the visible area
//             // NOTE: OldScroll# and NewScroll# will both be -1 (initial value) if the info for the respective scrollbar
//             //       was not obtained successfully in the previous steps. This means their difference is 0 (no scrolling)
//             //       and their associated scrollbar is left alone.
//             if ((OldScrollX != NewScrollX) || (OldScrollY != NewScrollY))
//             {
//                 Buff->ViewOrigin.X = NewScrollX;
//                 Buff->ViewOrigin.Y = NewScrollY;
//                 ScrollWindowEx(GuiData->hWindow,
//                                (OldScrollX - NewScrollX) * GuiData->CharWidth,
//                                (OldScrollY - NewScrollY) * GuiData->CharHeight,
//                                NULL,
//                                NULL,
//                                NULL,
//                                NULL,
//                                SW_INVALIDATE);
//                 if (NewScrollX >= 0)
//                 {
//                     sInfo.nPos = NewScrollX;
//                     SetScrollInfo(GuiData->hWindow, SB_HORZ as i32, &sInfo, TRUE);
//                 }
//                 if (NewScrollY >= 0)
//                 {
//                     sInfo.nPos = NewScrollY;
//                     SetScrollInfo(GuiData->hWindow, SB_VERT as i32, &sInfo, TRUE);
//                 }
//                 UpdateWindow(GuiData->hWindow);
//                 // InvalidateRect(GuiData->hWindow, NULL, FALSE);
//                 GuiData->OldCursor.x = Buff->CursorPosition.X;
//                 GuiData->OldCursor.y = Buff->CursorPosition.Y;
//             }
//         }
//     }
//     else /* if (GetType(Buff) == GRAPHICS_BUFFER) */
//     {
//     }
//
//     LeaveCriticalSection(&Console->Lock);
// }
//
// static BOOL
// OnClose(PGUI_CONSOLE_DATA GuiData)
// {
//     PCONSRV_CONSOLE Console = GuiData->Console;
//
//     if (!ConDrvValidateConsoleUnsafe((PCONSOLE)Console, CONSOLE_RUNNING, TRUE))
//         return TRUE;
//
//     // TODO: Prompt for termination ? (Warn the user about possible apps running in this console)
//
//     /*
//      * FIXME: Windows will wait up to 5 seconds for the thread to exit.
//      * We shouldn't wait here, though, since the console lock is entered.
//      * A copy of the thread list probably needs to be made.
//      */
//     ConSrvConsoleProcessCtrlEvent(Console, 0, CTRL_CLOSE_EVENT);
//
//     LeaveCriticalSection(&Console->Lock);
//     return FALSE;
// }
//
// static LRESULT
// OnNcDestroy(HWND hWnd)
// {
//     PGUI_CONSOLE_DATA GuiData = GuiGetGuiData(hWnd);
//
//     /* Free the GuiData registration */
//     SetWindowLongPtrW(hWnd, GWLP_USERDATA, (DWORD_PTR)NULL);
//
//     /* Reset the system menu back to default and destroy the previous menu */
//     GetSystemMenu(hWnd, TRUE);
//
//     if (GuiData)
//     {
//         if (GuiData->IsWindowVisible)
//             KillTimer(hWnd, CONGUI_UPDATE_TIMER);
//
//         /* Free the terminal framebuffer */
//         if (GuiData->hMemDC ) DeleteDC(GuiData->hMemDC);
//         if (GuiData->hBitmap) DeleteObject(GuiData->hBitmap);
//         // if (GuiData->hSysPalette) DeleteObject(GuiData->hSysPalette);
//         DeleteFonts(GuiData);
//     }
//
//     return DefWindowProcW(hWnd, WM_NCDESTROY, 0, 0);
// }
//
// static VOID
// OnScroll(PGUI_CONSOLE_DATA GuiData, INT nBar, WORD sbCode)
// {
//     PCONSRV_CONSOLE Console = GuiData->Console;
//     PCONSOLE_SCREEN_BUFFER Buff;
//     SCROLLINFO sInfo;
//     INT oldPos, Maximum;
//     PSHORT pOriginXY;
//
//     ASSERT(nBar == SB_HORZ as i32 || nBar == SB_VERT as i32);
//
//     if (!ConDrvValidateConsoleUnsafe((PCONSOLE)Console, CONSOLE_RUNNING, TRUE)) return;
//
//     Buff = GuiData->ActiveBuffer;
//
//     if (nBar == SB_HORZ as i32)
//     {
//         Maximum = Buff->ScreenBufferSize.X - Buff->ViewSize.X;
//         pOriginXY = &Buff->ViewOrigin.X;
//     }
//     else // if (nBar == SB_VERT as i32)
//     {
//         Maximum = Buff->ScreenBufferSize.Y - Buff->ViewSize.Y;
//         pOriginXY = &Buff->ViewOrigin.Y;
//     }
//
//     /* Set scrollbar sizes */
//     sInfo.cbSize = sizeof(sInfo);
//     sInfo.fMask = SIF_RANGE | SIF_POS | SIF_PAGE | SIF_TRACKPOS;
//
//     if (!GetScrollInfo(GuiData->hWindow, nBar, &sInfo)) goto Quit;
//
//     oldPos = sInfo.nPos;
//
//     switch (sbCode)
//     {
//         case SB_LINEUP:   // SB_LINELEFT:
//             sInfo.nPos--;
//             break;
//
//         case SB_LINEDOWN: // SB_LINERIGHT:
//             sInfo.nPos++;
//             break;
//
//         case SB_PAGEUP:   // SB_PAGELEFT:
//             sInfo.nPos -= sInfo.nPage;
//             break;
//
//         case SB_PAGEDOWN: // SB_PAGERIGHT:
//             sInfo.nPos += sInfo.nPage;
//             break;
//
//         case SB_THUMBTRACK:
//             sInfo.nPos = sInfo.nTrackPos;
//             ConioPause(Console, PAUSED_FROM_SCROLLBAR);
//             break;
//
//         case SB_THUMBPOSITION:
//             sInfo.nPos = sInfo.nTrackPos;
//             ConioUnpause(Console, PAUSED_FROM_SCROLLBAR);
//             break;
//
//         case SB_TOP:    // SB_LEFT:
//             sInfo.nPos = sInfo.nMin;
//             break;
//
//         case SB_BOTTOM: // SB_RIGHT:
//             sInfo.nPos = sInfo.nMax;
//             break;
//
//         default:
//             break;
//     }
//
//     sInfo.nPos = min(max(sInfo.nPos, 0), Maximum);
//
//     if (oldPos != sInfo.nPos)
//     {
//         USHORT OldX = Buff->ViewOrigin.X;
//         USHORT OldY = Buff->ViewOrigin.Y;
//         UINT   WidthUnit, HeightUnit;
//
//         /* We now modify Buff->ViewOrigin */
//         *pOriginXY = sInfo.nPos;
//
//         GetScreenBufferSizeUnits(Buff, GuiData, &WidthUnit, &HeightUnit);
//
//         ScrollWindowEx(GuiData->hWindow,
//                        (OldX - Buff->ViewOrigin.X) * WidthUnit ,
//                        (OldY - Buff->ViewOrigin.Y) * HeightUnit,
//                        NULL,
//                        NULL,
//                        NULL,
//                        NULL,
//                        SW_INVALIDATE);
//
//         sInfo.fMask = SIF_POS;
//         SetScrollInfo(GuiData->hWindow, nBar, &sInfo, TRUE);
//
//         UpdateWindow(GuiData->hWindow);
//         // InvalidateRect(GuiData->hWindow, NULL, FALSE);
//     }
//
// Quit:
//     LeaveCriticalSection(&Console->Lock);
//     return;
// }
//
// static COORD
// PointToCoord(PGUI_CONSOLE_DATA GuiData, LPARAM lParam)
// {
//     PCONSOLE_SCREEN_BUFFER Buffer = GuiData->ActiveBuffer;
//     COORD Coord;
//     UINT  WidthUnit, HeightUnit;
//
//     GetScreenBufferSizeUnits(Buffer, GuiData, &WidthUnit, &HeightUnit);
//
//     Coord.X = Buffer->ViewOrigin.X + ((SHORT)LOWORD(lParam) / (int)WidthUnit );
//     Coord.Y = Buffer->ViewOrigin.Y + ((SHORT)HIWORD(lParam) / (int)HeightUnit);
//
//     /* Clip coordinate to ensure it's inside buffer */
//     if (Coord.X < 0)
//         Coord.X = 0;
//     else if (Coord.X >= Buffer->ScreenBufferSize.X)
//         Coord.X = Buffer->ScreenBufferSize.X - 1;
//
//     if (Coord.Y < 0)
//         Coord.Y = 0;
//     else if (Coord.Y >= Buffer->ScreenBufferSize.Y)
//         Coord.Y = Buffer->ScreenBufferSize.Y - 1;
//
//     return Coord;
// }
//
// static LRESULT
// OnMouse(PGUI_CONSOLE_DATA GuiData, UINT msg, WPARAM wParam, LPARAM lParam)
// {
//     BOOL DoDefault = FALSE;
//     PCONSRV_CONSOLE Console = GuiData->Console;
//
//     /*
//      * HACK FOR CORE-8394 (Part 2):
//      *
//      * Check whether we should ignore the next mouse move event.
//      * In either case we reset the HACK flag.
//      *
//      * See Part 1 of this hack below.
//      */
//     if (GuiData->HackCORE8394IgnoreNextMove && msg == WM_MOUSEMOVE)
//     {
//         GuiData->HackCORE8394IgnoreNextMove = FALSE;
//         goto Quit;
//     }
//     GuiData->HackCORE8394IgnoreNextMove = FALSE;
//
//     // FIXME: It's here that we need to check whether we have focus or not
//     // and whether we are or not in edit mode, in order to know if we need
//     // to deal with the mouse.
//
//     if (GuiData->IgnoreNextMouseEvent)
//     {
//         if (msg != WM_LBUTTONDOWN &&
//             msg != WM_MBUTTONDOWN &&
//             msg != WM_RBUTTONDOWN &&
//             msg != WM_XBUTTONDOWN)
//         {
//             /*
//              * If this mouse event is not a button-down action
//              * then this is the last one being ignored.
//              */
//             GuiData->IgnoreNextMouseEvent = FALSE;
//         }
//         else
//         {
//             /*
//              * This mouse event is a button-down action.
//              * Ignore it and perform default action.
//              */
//             DoDefault = TRUE;
//         }
//         goto Quit;
//     }
//
//     if (!ConDrvValidateConsoleUnsafe((PCONSOLE)Console, CONSOLE_RUNNING, TRUE))
//     {
//         DoDefault = TRUE;
//         goto Quit;
//     }
//
//     if ( (GuiData->Selection.dwFlags & CONSOLE_SELECTION_IN_PROGRESS) ||
//          (Console->QuickEdit) )
//     {
//         switch (msg)
//         {
//             case WM_LBUTTONDOWN:
//             {
//                 /* Check for selection state */
//                 if ( (GuiData->Selection.dwFlags & CONSOLE_SELECTION_IN_PROGRESS) &&
//                      (GuiData->Selection.dwFlags & CONSOLE_MOUSE_SELECTION)       &&
//                      (GetKeyState(VK_SHIFT as i32) & KEY_PRESSED as i16) )
//                 {
//                     /*
//                      * A mouse selection is currently in progress and the user
//                      * has pressed the SHIFT key and clicked somewhere, update
//                      * the selection.
//                      */
//                     GuiData->dwSelectionCursor = PointToCoord(GuiData, lParam);
//                     UpdateSelection(GuiData, NULL, &GuiData->dwSelectionCursor);
//                 }
//                 else
//                 {
//                     /* Clear the old selection */
//                     GuiData->Selection.dwFlags = CONSOLE_NO_SELECTION;
//
//                     /* Restart a new selection */
//                     GuiData->dwSelectionCursor = PointToCoord(GuiData, lParam);
//                     SetCapture(GuiData->hWindow);
//                     GuiData->Selection.dwFlags |= CONSOLE_MOUSE_SELECTION | CONSOLE_MOUSE_DOWN;
//                     UpdateSelection(GuiData,
//                                     &GuiData->dwSelectionCursor,
//                                     &GuiData->dwSelectionCursor);
//                 }
//
//                 break;
//             }
//
//             case WM_LBUTTONUP:
//             {
//                 if (!(GuiData->Selection.dwFlags & CONSOLE_MOUSE_DOWN)) break;
//
//                 // GuiData->dwSelectionCursor = PointToCoord(GuiData, lParam);
//                 GuiData->Selection.dwFlags &= ~CONSOLE_MOUSE_DOWN;
//                 // UpdateSelection(GuiData, NULL, &GuiData->dwSelectionCursor);
//                 ReleaseCapture();
//
//                 break;
//             }
//
//             case WM_LBUTTONDBLCLK:
//             {
//                 PCONSOLE_SCREEN_BUFFER Buffer = GuiData->ActiveBuffer;
//
//                 if (GetType(Buffer) == TEXTMODE_BUFFER)
//                 {
// #define IS_WORD_SEP(c)  \
//     ((c) == L'\0' || (c) == L' ' || (c) == L'\t' || (c) == L'\r' || (c) == L'\n')
//
//                     PTEXTMODE_SCREEN_BUFFER TextBuffer = (PTEXTMODE_SCREEN_BUFFER)Buffer;
//                     COORD cL, cR;
//                     PCHAR_INFO ptrL, ptrR;
//
//                     /* Starting point */
//                     cL = cR = PointToCoord(GuiData, lParam);
//                     ptrL = ptrR = ConioCoordToPointer(TextBuffer, cL.X as u32, cL.Y as u32);
//
//                     /* Enlarge the selection by checking for whitespace */
//                     while ((0 < cL.X) && !IS_WORD_SEP(ptrL->Char.UnicodeChar)
//                                       && !IS_WORD_SEP((ptrL-1)->Char.UnicodeChar))
//                     {
//                         --cL.X;
//                         --ptrL;
//                     }
//                     while ((cR.X < TextBuffer->ScreenBufferSize.X - 1) &&
//                            !IS_WORD_SEP(ptrR->Char.UnicodeChar)        &&
//                            !IS_WORD_SEP((ptrR+1)->Char.UnicodeChar))
//                     {
//                         ++cR.X;
//                         ++ptrR;
//                     }
//
//                     /*
//                      * Update the selection started with the single
//                      * left-click that preceded this double-click.
//                      */
//                     GuiData->Selection.dwFlags |= CONSOLE_MOUSE_SELECTION | CONSOLE_MOUSE_DOWN;
//                     UpdateSelection(GuiData, &cL, &cR);
//
//                     /* Ignore the next mouse move event */
//                     GuiData->IgnoreNextMouseEvent = TRUE;
// #undef IS_WORD_SEP
//                 }
//
//                 break;
//             }
//
//             case WM_RBUTTONDOWN:
//             case WM_RBUTTONDBLCLK:
//             {
//                 if (!(GuiData->Selection.dwFlags & CONSOLE_SELECTION_NOT_EMPTY))
//                 {
//                     Paste(GuiData);
//                 }
//                 else
//                 {
//                     Copy(GuiData);
//                 }
//
//                 /* Ignore the next mouse move event */
//                 GuiData->IgnoreNextMouseEvent = TRUE;
//                 break;
//             }
//
//             case WM_MOUSEMOVE:
//             {
//                 if (!(GET_KEYSTATE_WPARAM(wParam) & MK_LBUTTON)) break;
//                 if (!(GuiData->Selection.dwFlags & CONSOLE_MOUSE_DOWN)) break;
//
//                 GuiData->dwSelectionCursor = PointToCoord(GuiData, lParam);
//                 UpdateSelection(GuiData, NULL, &GuiData->dwSelectionCursor);
//                 break;
//             }
//
//             default:
//                 DoDefault = TRUE; // FALSE;
//                 break;
//         }
//
//         /*
//          * HACK FOR CORE-8394 (Part 1):
//          *
//          * It appears that when running ReactOS on VBox with Mouse Integration
//          * enabled, the next mouse event coming after a button-down action is
//          * a mouse-move. However it is NOT always a rule, so that we cannot use
//          * the IgnoreNextMouseEvent flag to just "ignore" the next mouse event,
//          * thinking it would always be a mouse-move event.
//          *
//          * To work around this problem (that should really be fixed in Win32k),
//          * we use a second flag to ignore this possible next mouse move event.
//          */
//         switch (msg)
//         {
//             case WM_LBUTTONDOWN:
//             case WM_MBUTTONDOWN:
//             case WM_RBUTTONDOWN:
//             case WM_XBUTTONDOWN:
//                 GuiData->HackCORE8394IgnoreNextMove = TRUE;
//             default:
//                 break;
//         }
//     }
//     else if (GetConsoleInputBufferMode(Console) & ENABLE_MOUSE_INPUT)
//     {
//         INPUT_RECORD er;
//         WORD  wKeyState         = GET_KEYSTATE_WPARAM(wParam);
//         DWORD dwButtonState     = 0;
//         DWORD dwControlKeyState = 0;
//         DWORD dwEventFlags      = 0;
//
//         switch (msg)
//         {
//             case WM_LBUTTONDOWN:
//                 SetCapture(GuiData->hWindow);
//                 dwButtonState = FROM_LEFT_1ST_BUTTON_PRESSED;
//                 dwEventFlags  = 0;
//                 break;
//
//             case WM_MBUTTONDOWN:
//                 SetCapture(GuiData->hWindow);
//                 dwButtonState = FROM_LEFT_2ND_BUTTON_PRESSED;
//                 dwEventFlags  = 0;
//                 break;
//
//             case WM_RBUTTONDOWN:
//                 SetCapture(GuiData->hWindow);
//                 dwButtonState = RIGHTMOST_BUTTON_PRESSED;
//                 dwEventFlags  = 0;
//                 break;
//
//             case WM_XBUTTONDOWN:
//             {
//                 /* Get which X-button was pressed */
//                 WORD wButton = GET_XBUTTON_WPARAM(wParam);
//
//                 /* Check for X-button validity */
//                 if (wButton & ~((XBUTTON1 | XBUTTON2) as WORD))
//                 {
//                     DPRINT1("X-button 0x%04x invalid\n", wButton);
//                     DoDefault = TRUE;
//                     break;
//                 }
//
//                 SetCapture(GuiData->hWindow);
//                 dwButtonState = (wButton == XBUTTON1 as WORD ? FROM_LEFT_3RD_BUTTON_PRESSED
//                                                      : FROM_LEFT_4TH_BUTTON_PRESSED);
//                 dwEventFlags  = 0;
//                 break;
//             }
//
//             case WM_LBUTTONUP:
//                 ReleaseCapture();
//                 dwButtonState = 0;
//                 dwEventFlags  = 0;
//                 break;
//
//             case WM_MBUTTONUP:
//                 ReleaseCapture();
//                 dwButtonState = 0;
//                 dwEventFlags  = 0;
//                 break;
//
//             case WM_RBUTTONUP:
//                 ReleaseCapture();
//                 dwButtonState = 0;
//                 dwEventFlags  = 0;
//                 break;
//
//             case WM_XBUTTONUP:
//             {
//                 /* Get which X-button was released */
//                 WORD wButton = GET_XBUTTON_WPARAM(wParam);
//
//                 /* Check for X-button validity */
//                 if (wButton & ~((XBUTTON1 | XBUTTON2) as WORD))
//                 {
//                     DPRINT1("X-button 0x%04x invalid\n", wButton);
//                     /* Ok, just release the button anyway... */
//                 }
//
//                 ReleaseCapture();
//                 dwButtonState = 0;
//                 dwEventFlags  = 0;
//                 break;
//             }
//
//             case WM_LBUTTONDBLCLK:
//                 dwButtonState = FROM_LEFT_1ST_BUTTON_PRESSED;
//                 dwEventFlags  = DOUBLE_CLICK;
//                 break;
//
//             case WM_MBUTTONDBLCLK:
//                 dwButtonState = FROM_LEFT_2ND_BUTTON_PRESSED;
//                 dwEventFlags  = DOUBLE_CLICK;
//                 break;
//
//             case WM_RBUTTONDBLCLK:
//                 dwButtonState = RIGHTMOST_BUTTON_PRESSED;
//                 dwEventFlags  = DOUBLE_CLICK;
//                 break;
//
//             case WM_XBUTTONDBLCLK:
//             {
//                 /* Get which X-button was double-clicked */
//                 WORD wButton = GET_XBUTTON_WPARAM(wParam);
//
//                 /* Check for X-button validity */
//                 if (wButton & ~((XBUTTON1 | XBUTTON2) as WORD))
//                 {
//                     DPRINT1("X-button 0x%04x invalid\n", wButton);
//                     DoDefault = TRUE;
//                     break;
//                 }
//
//                 dwButtonState = (wButton == XBUTTON1 as WORD ? FROM_LEFT_3RD_BUTTON_PRESSED
//                                                      : FROM_LEFT_4TH_BUTTON_PRESSED);
//                 dwEventFlags  = DOUBLE_CLICK;
//                 break;
//             }
//
//             case WM_MOUSEMOVE:
//                 dwButtonState = 0;
//                 dwEventFlags  = MOUSE_MOVED;
//                 break;
//
//             case WM_MOUSEWHEEL:
//                 dwButtonState = GET_WHEEL_DELTA_WPARAM(wParam) << 16;
//                 dwEventFlags  = MOUSE_WHEELED;
//                 break;
//
//             case WM_MOUSEHWHEEL:
//                 dwButtonState = GET_WHEEL_DELTA_WPARAM(wParam) << 16;
//                 dwEventFlags  = MOUSE_HWHEELED;
//                 break;
//
//             default:
//                 DoDefault = TRUE;
//                 break;
//         }
//
//         /*
//          * HACK FOR CORE-8394 (Part 1):
//          *
//          * It appears that when running ReactOS on VBox with Mouse Integration
//          * enabled, the next mouse event coming after a button-down action is
//          * a mouse-move. However it is NOT always a rule, so that we cannot use
//          * the IgnoreNextMouseEvent flag to just "ignore" the next mouse event,
//          * thinking it would always be a mouse-move event.
//          *
//          * To work around this problem (that should really be fixed in Win32k),
//          * we use a second flag to ignore this possible next mouse move event.
//          */
//         switch (msg)
//         {
//             case WM_LBUTTONDOWN:
//             case WM_MBUTTONDOWN:
//             case WM_RBUTTONDOWN:
//             case WM_XBUTTONDOWN:
//                 GuiData->HackCORE8394IgnoreNextMove = TRUE;
//             default:
//                 break;
//         }
//
//         if (!DoDefault)
//         {
//             if (wKeyState & MK_LBUTTON as u16)
//                 dwButtonState |= FROM_LEFT_1ST_BUTTON_PRESSED;
//             if (wKeyState & MK_MBUTTON as u16)
//                 dwButtonState |= FROM_LEFT_2ND_BUTTON_PRESSED;
//             if (wKeyState & MK_RBUTTON as u16)
//                 dwButtonState |= RIGHTMOST_BUTTON_PRESSED;
//             if (wKeyState & MK_XBUTTON1 as u16)
//                 dwButtonState |= FROM_LEFT_3RD_BUTTON_PRESSED;
//             if (wKeyState & MK_XBUTTON2 as u16)
//                 dwButtonState |= FROM_LEFT_4TH_BUTTON_PRESSED;
//
//             if (GetKeyState(VK_RMENU as i32) & KEY_PRESSED as i16)
//                 dwControlKeyState |= RIGHT_ALT_PRESSED;
//             if (GetKeyState(VK_LMENU as i32) & KEY_PRESSED as i16)
//                 dwControlKeyState |= LEFT_ALT_PRESSED;
//             if (GetKeyState(VK_RCONTROL as i32) & KEY_PRESSED as i16)
//                 dwControlKeyState |= RIGHT_CTRL_PRESSED;
//             if (GetKeyState(VK_LCONTROL as i32) & KEY_PRESSED as i16)
//                 dwControlKeyState |= LEFT_CTRL_PRESSED;
//             if (GetKeyState(VK_SHIFT as i32) & KEY_PRESSED as i16)
//                 dwControlKeyState |= SHIFT_PRESSED;
//             if (GetKeyState(VK_NUMLOCK as i32) & KEY_TOGGLED as i16)
//                 dwControlKeyState |= NUMLOCK_ON;
//             if (GetKeyState(VK_SCROLL as i32) & KEY_TOGGLED as i16)
//                 dwControlKeyState |= SCROLLLOCK_ON;
//             if (GetKeyState(VK_CAPITAL as i32) & KEY_TOGGLED as i16)
//                 dwControlKeyState |= CAPSLOCK_ON;
//             /* See WM_CHAR MSDN documentation for instance */
//             if (HIWORD(lParam as u32) & KF_EXTENDED as u16)
//                 dwControlKeyState |= ENHANCED_KEY;
//
//             /* Send a mouse event */
//             er.EventType = MOUSE_EVENT as u16;
//             er.Event.MouseEvent.dwMousePosition   = PointToCoord(GuiData, lParam);
//             er.Event.MouseEvent.dwButtonState     = dwButtonState;
//             er.Event.MouseEvent.dwControlKeyState = dwControlKeyState;
//             er.Event.MouseEvent.dwEventFlags      = dwEventFlags;
//
//             ConioProcessInputEvent(Console, &er);
//         }
//     }
//     else
//     {
//         DoDefault = TRUE;
//     }
//
//     LeaveCriticalSection(&Console->Lock);
//
// Quit:
//     if (!DoDefault)
//         return 0;
//
//     if (msg == WM_MOUSEWHEEL || msg == WM_MOUSEHWHEEL)
//     {
//         INT   nBar;
//         WORD  sbCode;
//         // WORD  wKeyState = GET_KEYSTATE_WPARAM(wParam);
//         SHORT wDelta    = GET_WHEEL_DELTA_WPARAM(wParam);
//
//         if (msg == WM_MOUSEWHEEL)
//             nBar = SB_VERT as INT;
//         else // if (msg == WM_MOUSEHWHEEL)
//             nBar = SB_HORZ as INT;
//
//         // NOTE: We currently do not support zooming...
//         // if (wKeyState & MK_CONTROL)
//
//         // FIXME: For some reason our win32k does not set the key states
//         // when sending WM_MOUSEWHEEL or WM_MOUSEHWHEEL ...
//         // if (wKeyState & MK_SHIFT)
//         if (GetKeyState(VK_SHIFT as i32) & KEY_PRESSED as i16)
//             sbCode = (wDelta >= 0 ? SB_PAGEUP : SB_PAGEDOWN);
//         else
//             sbCode = (wDelta >= 0 ? SB_LINEUP : SB_LINEDOWN);
//
//         OnScroll(GuiData, nBar, sbCode);
//     }
//
//     return DefWindowProcW(GuiData->hWindow, msg, wParam, lParam);
// }
//
//
// static VOID
// Copy(PGUI_CONSOLE_DATA GuiData)
// {
//     if (OpenClipboard(GuiData->hWindow))
//     {
//         PCONSOLE_SCREEN_BUFFER Buffer = GuiData->ActiveBuffer;
//
//         if (GetType(Buffer) == TEXTMODE_BUFFER)
//         {
//             GuiCopyFromTextModeBuffer((PTEXTMODE_SCREEN_BUFFER)Buffer, GuiData);
//         }
//         else /* if (GetType(Buffer) == GRAPHICS_BUFFER) */
//         {
//             GuiCopyFromGraphicsBuffer((PGRAPHICS_SCREEN_BUFFER)Buffer, GuiData);
//         }
//
//         CloseClipboard();
//     }
//
//     /* Clear the selection */
//     UpdateSelection(GuiData, NULL, NULL);
// }
//
// static VOID
// Paste(PGUI_CONSOLE_DATA GuiData)
// {
//     if (OpenClipboard(GuiData->hWindow))
//     {
//         PCONSOLE_SCREEN_BUFFER Buffer = GuiData->ActiveBuffer;
//
//         if (GetType(Buffer) == TEXTMODE_BUFFER)
//         {
//             GuiPasteToTextModeBuffer((PTEXTMODE_SCREEN_BUFFER)Buffer, GuiData);
//         }
//         else /* if (GetType(Buffer) == GRAPHICS_BUFFER) */
//         {
//             GuiPasteToGraphicsBuffer((PGRAPHICS_SCREEN_BUFFER)Buffer, GuiData);
//         }
//
//         CloseClipboard();
//     }
// }
//
// static VOID
// OnGetMinMaxInfo(PGUI_CONSOLE_DATA GuiData, PMINMAXINFO minMaxInfo)
// {
//     PCONSRV_CONSOLE Console = GuiData->Console;
//     PCONSOLE_SCREEN_BUFFER ActiveBuffer;
//     DWORD windx, windy;
//     UINT  WidthUnit, HeightUnit;
//
//     if (!ConDrvValidateConsoleUnsafe((PCONSOLE)Console, CONSOLE_RUNNING, TRUE)) return;
//
//     ActiveBuffer = GuiData->ActiveBuffer;
//
//     GetScreenBufferSizeUnits(ActiveBuffer, GuiData, &WidthUnit, &HeightUnit);
//
//     windx = CONGUI_MIN_WIDTH  * WidthUnit  + 2 * (GetSystemMetrics(SM_CXFRAME as i32) + GetSystemMetrics(SM_CXEDGE as i32));
//     windy = CONGUI_MIN_HEIGHT * HeightUnit + 2 * (GetSystemMetrics(SM_CYFRAME as i32) + GetSystemMetrics(SM_CYEDGE as i32)) + GetSystemMetrics(SM_CYCAPTION as i32);
//
//     minMaxInfo->ptMinTrackSize.x = windx;
//     minMaxInfo->ptMinTrackSize.y = windy;
//
//     windx = (ActiveBuffer->ScreenBufferSize.X) * WidthUnit  + 2 * (GetSystemMetrics(SM_CXFRAME as i32) + GetSystemMetrics(SM_CXEDGE as i32));
//     windy = (ActiveBuffer->ScreenBufferSize.Y) * HeightUnit + 2 * (GetSystemMetrics(SM_CYFRAME as i32) + GetSystemMetrics(SM_CYEDGE as i32)) + GetSystemMetrics(SM_CYCAPTION as i32);
//
//     if (ActiveBuffer->ViewSize.X < ActiveBuffer->ScreenBufferSize.X) windy += GetSystemMetrics(SM_CYHSCROLL as i32); // Window currently has a horizontal scrollbar
//     if (ActiveBuffer->ViewSize.Y < ActiveBuffer->ScreenBufferSize.Y) windx += GetSystemMetrics(SM_CXVSCROLL as i32); // Window currently has a vertical scrollbar
//
//     minMaxInfo->ptMaxTrackSize.x = windx;
//     minMaxInfo->ptMaxTrackSize.y = windy;
//
//     LeaveCriticalSection(&Console->Lock);
// }
//
// static VOID
// OnSize(PGUI_CONSOLE_DATA GuiData, WPARAM wParam, LPARAM lParam)
// {
//     PCONSRV_CONSOLE Console = GuiData->Console;
//
//     /* Do nothing if the window is hidden */
//     if (!GuiData->IsWindowVisible || IsIconic(GuiData->hWindow)) return;
//
//     if (!ConDrvValidateConsoleUnsafe((PCONSOLE)Console, CONSOLE_RUNNING, TRUE)) return;
//
//     if (!GuiData->WindowSizeLock &&
//         (wParam == SIZE_RESTORED || wParam == SIZE_MAXIMIZED || wParam == SIZE_MINIMIZED))
//     {
//         PCONSOLE_SCREEN_BUFFER Buff = GuiData->ActiveBuffer;
//         DWORD windx, windy, charx, chary;
//         UINT  WidthUnit, HeightUnit;
//
//         GetScreenBufferSizeUnits(Buff, GuiData, &WidthUnit, &HeightUnit);
//
//         GuiData->WindowSizeLock = TRUE;
//
//         windx = LOWORD(lParam);
//         windy = HIWORD(lParam);
//
//         /* Compensate for existing scroll bars (because lParam values do not accommodate scroll bar) */
//         if (Buff->ViewSize.X < Buff->ScreenBufferSize.X) windy += GetSystemMetrics(SM_CYHSCROLL as i32); // Window currently has a horizontal scrollbar
//         if (Buff->ViewSize.Y < Buff->ScreenBufferSize.Y) windx += GetSystemMetrics(SM_CXVSCROLL as i32); // Window currently has a vertical scrollbar
//
//         charx = windx / (int)WidthUnit ;
//         chary = windy / (int)HeightUnit;
//
//         /* Character alignment (round size up or down) */
//         if ((windx % WidthUnit ) >= (WidthUnit  / 2)) ++charx;
//         if ((windy % HeightUnit) >= (HeightUnit / 2)) ++chary;
//
//         /* Compensate for added scroll bars in window */
//         if (Buff->ViewSize.X < Buff->ScreenBufferSize.X) windy -= GetSystemMetrics(SM_CYHSCROLL as i32); // Window will have a horizontal scroll bar
//         if (Buff->ViewSize.Y < Buff->ScreenBufferSize.Y) windx -= GetSystemMetrics(SM_CXVSCROLL as i32); // Window will have a vertical scroll bar
//
//         charx = windx / (int)WidthUnit ;
//         chary = windy / (int)HeightUnit;
//
//         /* Character alignment (round size up or down) */
//         if ((windx % WidthUnit ) >= (WidthUnit  / 2)) ++charx;
//         if ((windy % HeightUnit) >= (HeightUnit / 2)) ++chary;
//
//         /* Resize window */
//         if ((charx != Buff->ViewSize.X) || (chary != Buff->ViewSize.Y))
//         {
//             Buff->ViewSize.X = (charx <= (DWORD)Buff->ScreenBufferSize.X) ? charx : Buff->ScreenBufferSize.X;
//             Buff->ViewSize.Y = (chary <= (DWORD)Buff->ScreenBufferSize.Y) ? chary : Buff->ScreenBufferSize.Y;
//         }
//
//         ResizeConWnd(GuiData, WidthUnit, HeightUnit);
//
//         /* Adjust the start of the visible area if we are attempting to show nonexistent areas */
//         if ((Buff->ScreenBufferSize.X - Buff->ViewOrigin.X) < Buff->ViewSize.X) Buff->ViewOrigin.X = Buff->ScreenBufferSize.X - Buff->ViewSize.X;
//         if ((Buff->ScreenBufferSize.Y - Buff->ViewOrigin.Y) < Buff->ViewSize.Y) Buff->ViewOrigin.Y = Buff->ScreenBufferSize.Y - Buff->ViewSize.Y;
//         InvalidateRect(GuiData->hWindow, NULL, TRUE);
//
//         GuiData->WindowSizeLock = FALSE;
//     }
//
//     LeaveCriticalSection(&Console->Lock);
// }
//
// static VOID
// OnMove(PGUI_CONSOLE_DATA GuiData)
// {
//     RECT rcWnd;
//
//     // TODO: Simplify the code.
//     // See: GuiConsoleNotifyWndProc() PM_CREATE_CONSOLE.
//
//     /* Retrieve our real position */
//     GetWindowRect(GuiData->hWindow, &rcWnd);
//     GuiData->GuiInfo.WindowOrigin.x = rcWnd.left;
//     GuiData->GuiInfo.WindowOrigin.y = rcWnd.top;
// }
//
// static VOID
// OnDropFiles(PCONSRV_CONSOLE Console, HDROP hDrop)
// {
//     LPWSTR pszPath;
//     WCHAR szPath[MAX_PATH + 2];
//
//     szPath[0] = L'"';
//
//     DragQueryFileW(hDrop, 0, &szPath[1], ARRAYSIZE(szPath) - 1);
//     DragFinish(hDrop);
//
//     if (wcschr(&szPath[1], L' ') != NULL)
//     {
//         StringCchCatW(szPath, ARRAYSIZE(szPath), L"\"");
//         pszPath = szPath;
//     }
//     else
//     {
//         pszPath = &szPath[1];
//     }
//
//     PasteText(Console, pszPath, wcslen(pszPath));
// }
//
// /*
// // HACK: This functionality is standard for general scrollbars. Don't add it by hand.
//
// VOID
// GuiConsoleHandleScrollbarMenu(VOID)
// {
//     HMENU hMenu;
//
//     hMenu = CreatePopupMenu();
//     if (hMenu == NULL)
//     {
//         DPRINT("CreatePopupMenu failed\n");
//         return;
//     }
//
//     //InsertItem(hMenu, MIIM_STRING, MIIM_ID | MIIM_FTYPE | MIIM_STRING, 0, NULL, IDS_SCROLLHERE);
//     //InsertItem(hMenu, MFT_SEPARATOR, MIIM_FTYPE, 0, NULL, -1);
//     //InsertItem(hMenu, MIIM_STRING, MIIM_ID | MIIM_FTYPE | MIIM_STRING, 0, NULL, IDS_SCROLLTOP);
//     //InsertItem(hMenu, MIIM_STRING, MIIM_ID | MIIM_FTYPE | MIIM_STRING, 0, NULL, IDS_SCROLLBOTTOM);
//     //InsertItem(hMenu, MFT_SEPARATOR, MIIM_FTYPE, 0, NULL, -1);
//     //InsertItem(hMenu, MIIM_STRING, MIIM_ID | MIIM_FTYPE | MIIM_STRING, 0, NULL, IDS_SCROLLPAGE_UP);
//     //InsertItem(hMenu, MIIM_STRING, MIIM_ID | MIIM_FTYPE | MIIM_STRING, 0, NULL, IDS_SCROLLPAGE_DOWN);
//     //InsertItem(hMenu, MFT_SEPARATOR, MIIM_FTYPE, 0, NULL, -1);
//     //InsertItem(hMenu, MIIM_STRING, MIIM_ID | MIIM_FTYPE | MIIM_STRING, 0, NULL, IDS_SCROLLUP);
//     //InsertItem(hMenu, MIIM_STRING, MIIM_ID | MIIM_FTYPE | MIIM_STRING, 0, NULL, IDS_SCROLLDOWN);
// }
// */
//
// HBITMAP
// CreateFrameBufferBitmap(HDC hDC, int width, int height)
// {
//     BITMAPINFO bmi;
//     ZeroMemory(&bmi, sizeof(BITMAPINFO));
//     bmi.bmiHeader.biSize = sizeof(BITMAPINFOHEADER);
//     bmi.bmiHeader.biWidth = width;
//     bmi.bmiHeader.biHeight = height;
//     bmi.bmiHeader.biPlanes = 1;
//     bmi.bmiHeader.biBitCount = GetDeviceCaps(hDC, BITSPIXEL);
//     bmi.bmiHeader.biCompression = BI_RGB;
//     return CreateDIBSection(NULL, &bmi, DIB_RGB_COLORS, NULL, NULL, 0);
// }
//
// static LRESULT CALLBACK
// ConWndProc(HWND hWnd, UINT msg, WPARAM wParam, LPARAM lParam)
// {
//     LRESULT Result = 0;
//     PGUI_CONSOLE_DATA GuiData = NULL;
//     PCONSRV_CONSOLE Console = NULL;
//
//     /*
//      * - If it's the first time we create a window for the terminal,
//      *   just initialize it and return.
//      *
//      * - If we are destroying the window, just do it and return.
//      */
//     if (msg == WM_NCCREATE)
//     {
//         return (LRESULT)OnNcCreate(hWnd, (LPCREATESTRUCTW)lParam);
//     }
//     else if (msg == WM_NCDESTROY)
//     {
//         return OnNcDestroy(hWnd);
//     }
//
//     /*
//      * Now the terminal window is initialized.
//      * Get the terminal data via the window's data.
//      * If there is no data, just go away.
//      */
//     GuiData = GuiGetGuiData(hWnd);
//     if (GuiData == NULL) return DefWindowProcW(hWnd, msg, wParam, lParam);
//
//     // TEMPORARY HACK until all of the functions can deal with a NULL GuiData->ActiveBuffer ...
//     if (GuiData->ActiveBuffer == NULL) return DefWindowProcW(hWnd, msg, wParam, lParam);
//
//     /*
//      * Just retrieve a pointer to the console in case somebody needs it.
//      * It is not NULL because it was checked in GuiGetGuiData.
//      * Each helper function which needs the console has to validate and lock it.
//      */
//     Console = GuiData->Console;
//
//     /* We have a console, start message dispatching */
//     switch (msg)
//     {
//         case WM_ACTIVATE:
//             OnActivate(GuiData, wParam);
//             break;
//
//         case WM_CLOSE:
//             if (OnClose(GuiData)) goto Default;
//             break;
//
//         case WM_ERASEBKGND:
//             return TRUE;
//
//         case WM_PAINT:
//             OnPaint(GuiData);
//             break;
//
//         case WM_TIMER:
//             OnTimer(GuiData);
//             break;
//
//         case WM_PALETTECHANGED:
//         {
//             DPRINT("WM_PALETTECHANGED called\n");
//
//             /*
//              * Protects against infinite loops:
//              * "... A window that receives this message must not realize
//              * its palette, unless it determines that wParam does not contain
//              * its own window handle." (WM_PALETTECHANGED description - MSDN)
//              *
//              * This message is sent to all windows, including the one that
//              * changed the system palette and caused this message to be sent.
//              * The wParam of this message contains the handle of the window
//              * that caused the system palette to change. To avoid an infinite
//              * loop, care must be taken to check that the wParam of this message
//              * does not match the window's handle.
//              */
//             if ((HWND)wParam == hWnd) break;
//
//             DPRINT("WM_PALETTECHANGED ok\n");
//             OnPaletteChanged(GuiData);
//             DPRINT("WM_PALETTECHANGED quit\n");
//             break;
//         }
//
//         case WM_KEYDOWN:
//         case WM_KEYUP:
//         case WM_CHAR:
//         case WM_DEADCHAR:
//         case WM_SYSKEYDOWN:
//         case WM_SYSKEYUP:
//         case WM_SYSCHAR:
//         case WM_SYSDEADCHAR:
//         {
//             /* Detect Alt-Enter presses and switch back and forth to fullscreen mode */
//             if (msg == WM_SYSKEYDOWN && (HIWORD(lParam as u32) & KF_ALTDOWN as u16) && wParam == VK_RETURN)
//             {
//                 /* Switch only at first Alt-Enter press, and ignore subsequent key repetitions */
//                 if ((HIWORD(lParam as u32) & (KF_UP | KF_REPEAT) as u16) != KF_REPEAT as u16)
//                     GuiConsoleSwitchFullScreen(GuiData);
//
//                 break;
//             }
//             /* Detect Alt-Esc/Space/Tab presses defer to DefWindowProc */
//             if ( (HIWORD(lParam as u32) & KF_ALTDOWN as u16) && (wParam == VK_ESCAPE || wParam == VK_SPACE || wParam == VK_TAB))
//             {
//                 return DefWindowProcW(hWnd, msg, wParam, lParam);
//             }
//             /* Detect Alt+Shift */
//             if (wParam == VK_SHIFT && (msg == WM_SYSKEYDOWN || msg == WM_SYSKEYUP))
//             {
//                 return DefWindowProcW(hWnd, msg, wParam, lParam);
//             }
//
//             OnKey(GuiData, msg, wParam, lParam);
//             break;
//         }
//
//         case WM_SETCURSOR:
//         {
//             /* Do nothing if the window is hidden */
//             if (!GuiData->IsWindowVisible) goto Default;
//
//             /*
//              * The message was sent because we are manually triggering a change.
//              * Check whether the mouse is indeed present on this console window
//              * and take appropriate decisions.
//              */
//             if (wParam == -1 && lParam == -1)
//             {
//                 POINT mouseCoords;
//                 HWND  hWndHit;
//
//                 /* Get the placement of the mouse */
//                 GetCursorPos(&mouseCoords);
//
//                 /* On which window is placed the mouse ? */
//                 hWndHit = WindowFromPoint(mouseCoords);
//
//                 /* It's our window. Perform the hit-test to be used later on. */
//                 if (hWndHit == hWnd)
//                 {
//                     wParam = (WPARAM)hWnd;
//                     lParam = DefWindowProcW(hWndHit, WM_NCHITTEST, 0,
//                                             MAKELPARAM(mouseCoords.x, mouseCoords.y));
//                 }
//             }
//
//             /* Set the mouse cursor only when we are in the client area */
//             if ((HWND)wParam == hWnd && LOWORD(lParam) == HTCLIENT)
//             {
//                 if (GuiData->MouseCursorRefCount >= 0)
//                 {
//                     /* Show the cursor */
//                     SetCursor(GuiData->hCursor);
//                 }
//                 else
//                 {
//                     /* Hide the cursor if the reference count is negative */
//                     SetCursor(NULL);
//                 }
//                 return TRUE;
//             }
//             else
//             {
//                 goto Default;
//             }
//         }
//
//         case WM_LBUTTONDOWN:
//         case WM_MBUTTONDOWN:
//         case WM_RBUTTONDOWN:
//         case WM_XBUTTONDOWN:
//         case WM_LBUTTONUP:
//         case WM_MBUTTONUP:
//         case WM_RBUTTONUP:
//         case WM_XBUTTONUP:
//         case WM_LBUTTONDBLCLK:
//         case WM_MBUTTONDBLCLK:
//         case WM_RBUTTONDBLCLK:
//         case WM_XBUTTONDBLCLK:
//         case WM_MOUSEMOVE:
//         case WM_MOUSEWHEEL:
//         case WM_MOUSEHWHEEL:
//         {
//             Result = OnMouse(GuiData, msg, wParam, lParam);
//             break;
//         }
//
//         case WM_HSCROLL:
//             OnScroll(GuiData, SB_HORZ as i32, LOWORD(wParam));
//             break;
//
//         case WM_VSCROLL:
//             OnScroll(GuiData, SB_VERT as i32, LOWORD(wParam));
//             break;
//
//         case WM_CONTEXTMENU:
//         {
//             /* Do nothing if the window is hidden */
//             if (!GuiData->IsWindowVisible) break;
//
//             if (DefWindowProcW(hWnd /*GuiData->hWindow*/, WM_NCHITTEST, 0, lParam) == HTCLIENT)
//             {
//                 HMENU hMenu = CreatePopupMenu();
//                 if (hMenu != NULL)
//                 {
//                     AppendMenuItems(hMenu, GuiConsoleEditMenuItems);
//                     TrackPopupMenuEx(hMenu,
//                                      TPM_RIGHTBUTTON,
//                                      GET_X_LPARAM(lParam),
//                                      GET_Y_LPARAM(lParam),
//                                      hWnd,
//                                      NULL);
//                     DestroyMenu(hMenu);
//                 }
//                 break;
//             }
//             else
//             {
//                 goto Default;
//             }
//         }
//
//         case WM_INITMENU:
//         {
//             HMENU hMenu = (HMENU)wParam;
//             if (hMenu != NULL)
//             {
//                 /* Enable or disable the Close menu item */
//                 EnableMenuItem(hMenu, SC_CLOSE, MF_BYCOMMAND |
//                                (GuiData->IsCloseButtonEnabled ? MF_ENABLED : MF_GRAYED));
//
//                 /* Enable or disable the Copy and Paste items */
//                 EnableMenuItem(hMenu, ID_SYSTEM_EDIT_COPY , MF_BYCOMMAND |
//                                ((GuiData->Selection.dwFlags & CONSOLE_SELECTION_IN_PROGRESS) &&
//                                 (GuiData->Selection.dwFlags & CONSOLE_SELECTION_NOT_EMPTY) ? MF_ENABLED : MF_GRAYED));
//                 // FIXME: Following whether the active screen buffer is text-mode
//                 // or graphics-mode, search for CF_UNICODETEXT or CF_BITMAP formats.
//                 EnableMenuItem(hMenu, ID_SYSTEM_EDIT_PASTE, MF_BYCOMMAND |
//                                (!(GuiData->Selection.dwFlags & CONSOLE_SELECTION_IN_PROGRESS) &&
//                                 IsClipboardFormatAvailable(CF_UNICODETEXT) ? MF_ENABLED : MF_GRAYED));
//             }
//
//             SendMenuEvent(Console, WM_INITMENU);
//             break;
//         }
//
//         case WM_MENUSELECT:
//         {
//             if (HIWORD(wParam) == 0xFFFF) // Allow all the menu flags
//             {
//                 SendMenuEvent(Console, WM_MENUSELECT);
//             }
//             break;
//         }
//
//         case WM_COMMAND:
//         case WM_SYSCOMMAND:
//         {
//             Result = OnCommand(GuiData, wParam, lParam);
//             break;
//         }
//
//         case WM_DROPFILES:
//             OnDropFiles(Console, (HDROP)wParam);
//             break;
//
//         case WM_SETFOCUS:
//         case WM_KILLFOCUS:
//             OnFocus(GuiData, (msg == WM_SETFOCUS));
//             break;
//
//         case WM_GETMINMAXINFO:
//             OnGetMinMaxInfo(GuiData, (PMINMAXINFO)lParam);
//             break;
//
//         case WM_MOVE:
//             OnMove(GuiData);
//             break;
//
// #if 0 // This code is here to prepare & control dynamic console SB resizing.
//         case WM_SIZING:
//         {
//             PRECT dragRect = (PRECT)lParam;
//             switch (wParam)
//             {
//                 case WMSZ_LEFT:
//                     DPRINT1("WMSZ_LEFT\n");
//                     break;
//                 case WMSZ_RIGHT:
//                     DPRINT1("WMSZ_RIGHT\n");
//                     break;
//                 case WMSZ_TOP:
//                     DPRINT1("WMSZ_TOP\n");
//                     break;
//                 case WMSZ_TOPLEFT:
//                     DPRINT1("WMSZ_TOPLEFT\n");
//                     break;
//                 case WMSZ_TOPRIGHT:
//                     DPRINT1("WMSZ_TOPRIGHT\n");
//                     break;
//                 case WMSZ_BOTTOM:
//                     DPRINT1("WMSZ_BOTTOM\n");
//                     break;
//                 case WMSZ_BOTTOMLEFT:
//                     DPRINT1("WMSZ_BOTTOMLEFT\n");
//                     break;
//                 case WMSZ_BOTTOMRIGHT:
//                     DPRINT1("WMSZ_BOTTOMRIGHT\n");
//                     break;
//                 default:
//                     DPRINT1("wParam = %d\n", wParam);
//                     break;
//             }
//             DPRINT1("dragRect = {.left = %d ; .top = %d ; .right = %d ; .bottom = %d}\n",
//                     dragRect->left, dragRect->top, dragRect->right, dragRect->bottom);
//             break;
//         }
// #endif
//
//         case WM_SIZE:
//             OnSize(GuiData, wParam, lParam);
//             break;
//
//         case PM_RESIZE_TERMINAL:
//         {
//             PCONSOLE_SCREEN_BUFFER Buff = GuiData->ActiveBuffer;
//             HDC hDC;
//             HBITMAP hnew, hold;
//
//             DWORD Width, Height;
//             UINT  WidthUnit, HeightUnit;
//
//             /* Do nothing if the window is hidden */
//             if (!GuiData->IsWindowVisible) break;
//
//             GetScreenBufferSizeUnits(Buff, GuiData, &WidthUnit, &HeightUnit);
//
//             Width  = Buff->ScreenBufferSize.X * WidthUnit ;
//             Height = Buff->ScreenBufferSize.Y * HeightUnit;
//
//             /* Recreate the framebuffer */
//             hDC  = GetDC(GuiData->hWindow);
//             hnew = CreateFrameBufferBitmap(hDC, Width, Height);
//             ReleaseDC(GuiData->hWindow, hDC);
//             hold = SelectObject(GuiData->hMemDC, hnew);
//             if (GuiData->hBitmap)
//             {
//                 if (hold == GuiData->hBitmap) DeleteObject(GuiData->hBitmap);
//             }
//             GuiData->hBitmap = hnew;
//
//             /* Resize the window to the user's values */
//             GuiData->WindowSizeLock = TRUE;
//             ResizeConWnd(GuiData, WidthUnit, HeightUnit);
//             GuiData->WindowSizeLock = FALSE;
//             break;
//         }
//
//         /*
//          * Undocumented message sent by Windows' console.dll for applying console info.
//          * See https://web.archive.org/web/20160307053337/https://www.catch22.net/sites/default/source/files/setconsoleinfo.c
//          * and https://dl.packetstormsecurity.net/papers/win/MSBugPaper.pdf
//          * for more information.
//          */
//         case WM_SETCONSOLEINFO:
//         {
//             GuiApplyUserSettings(GuiData, (HANDLE)wParam);
//             break;
//         }
//
//         case PM_CONSOLE_BEEP:
//             DPRINT1("Beep\n");
//             Beep(800, 200);
//             break;
//
//          case PM_CONSOLE_SET_TITLE:
//             SetWindowTextW(GuiData->hWindow, GuiData->Console->Title.Buffer);
//             break;
//
//         default: Default:
//             Result = DefWindowProcW(hWnd, msg, wParam, lParam);
//             break;
//     }
//
//     return Result;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
