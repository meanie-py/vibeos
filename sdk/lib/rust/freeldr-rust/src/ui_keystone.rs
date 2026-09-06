//! ui_keystone — shared UI types/consts/macros the ui unit needs.
//!
//! This is the ui-unit analogue of `frldr.rs` (which is owned by the fs-unit
//! sibling and must not be edited from the ui-unit session). It carries the
//! `ui.h` / `ui/*.h` surface that rustconv dropped: UIVTBL, UI_MENU_INFO,
//! UI_PROGRESS_BAR, PALETTE_ENTRY, the screen-color / box-draw `#define`s, the
//! `ATTR()` / `RTL_NUMBER_OF()` macros, and the VIDEODISPLAYMODE enum.
//!
//! When the fs-unit sibling merges these into `frldr.rs`, this module can be
//! folded away. Kept separate now purely to avoid a cross-session edit on the
//! sibling-owned `frldr.rs`.

#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

use ntkrnl_sys::base::{UCHAR, ULONG, LONG, CHAR, BOOLEAN, PVOID, PCHAR, NTSTATUS, SHORT};
use ntkrnl_sys::ddk::PCSTR;

// ---- ui.h shared UI progress-bar / menu types ----
#[repr(C)]
pub struct UI_PROGRESS_BAR_State { pub Floor: ULONG, pub Bias: ULONG }
#[repr(C)]
pub struct UI_PROGRESS_BAR_Indicator { pub Count: ULONG, pub Expected: ULONG, pub Percentage: ULONG }
#[repr(C)]
pub struct UI_PROGRESS_BAR {
    pub State: UI_PROGRESS_BAR_State,
    pub Indicator: UI_PROGRESS_BAR_Indicator,
    pub Left: ULONG, pub Top: ULONG, pub Right: ULONG, pub Bottom: ULONG,
    pub Show: BOOLEAN,
}
pub type PUI_PROGRESS_BAR = *mut UI_PROGRESS_BAR;

#[repr(C)]
pub struct UI_MENU_INFO {
    pub MenuHeader: PCSTR, pub MenuFooter: PCSTR,
    pub MenuItemList: *mut PCSTR, pub MenuItemCount: ULONG,
    pub MenuTimeRemaining: LONG, pub SelectedMenuItem: ULONG,
    pub Context: PVOID,
    pub Left: ULONG, pub Top: ULONG, pub Right: ULONG, pub Bottom: ULONG,
}
pub type PUI_MENU_INFO = *mut UI_MENU_INFO;

pub type UiMenuKeyPressFilterCallback =
    Option<unsafe extern "C" fn(ULONG, ULONG, PVOID) -> BOOLEAN>;

// UIVTBL — 24 Option<extern "C" fn> slots, exact ui.h:252 order.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct UIVTBL {
    pub Initialize:    Option<unsafe extern "C" fn() -> BOOLEAN>,
    pub UnInitialize:  Option<unsafe extern "C" fn()>,
    pub DrawBackdrop:  Option<unsafe extern "C" fn(ULONG)>,
    pub FillArea:      Option<unsafe extern "C" fn(ULONG, ULONG, ULONG, ULONG, CHAR, UCHAR)>,
    pub DrawShadow:    Option<unsafe extern "C" fn(ULONG, ULONG, ULONG, ULONG)>,
    pub DrawBox:       Option<unsafe extern "C" fn(ULONG, ULONG, ULONG, ULONG, UCHAR, UCHAR, BOOLEAN, BOOLEAN, UCHAR)>,
    pub DrawText:      Option<unsafe extern "C" fn(ULONG, ULONG, PCSTR, UCHAR)>,
    pub DrawText2:     Option<unsafe extern "C" fn(ULONG, ULONG, ULONG, PCSTR, UCHAR)>,
    pub DrawCenteredText: Option<unsafe extern "C" fn(ULONG, ULONG, ULONG, ULONG, PCSTR, UCHAR)>,
    pub DrawStatusText: Option<unsafe extern "C" fn(PCSTR)>,
    pub UpdateDateTime: Option<unsafe extern "C" fn()>,
    pub MessageBox:    Option<unsafe extern "C" fn(PCSTR)>,
    pub MessageBoxCritical: Option<unsafe extern "C" fn(PCSTR)>,
    pub DrawProgressBarCenter: Option<unsafe extern "C" fn(PCSTR)>,
    pub DrawProgressBar: Option<unsafe extern "C" fn(ULONG, ULONG, ULONG, ULONG, PCSTR)>,
    pub SetProgressBarText: Option<unsafe extern "C" fn(PCSTR)>,
    pub TickProgressBar: Option<unsafe extern "C" fn(ULONG)>,
    pub EditBox:       Option<unsafe extern "C" fn(PCSTR, PCHAR, ULONG) -> BOOLEAN>,
    pub TextToColor:   Option<unsafe extern "C" fn(PCSTR) -> UCHAR>,
    pub TextToFillStyle: Option<unsafe extern "C" fn(PCSTR) -> UCHAR>,
    pub FadeInBackdrop: Option<unsafe extern "C" fn()>,
    pub FadeOut:       Option<unsafe extern "C" fn()>,
    pub DisplayMenu:   Option<unsafe extern "C" fn(PCSTR, PCSTR, *mut PCSTR, ULONG, ULONG, LONG, *mut ULONG, BOOLEAN, UiMenuKeyPressFilterCallback, PVOID) -> BOOLEAN>,
    pub DrawMenu:      Option<unsafe extern "C" fn(PUI_MENU_INFO)>,
}
pub type PUIVTBL = *mut UIVTBL;
// fn-ptr only; vtbls are effectively read-only.
unsafe impl Sync for UIVTBL {}

// ---- ui/video.h PALETTE_ENTRY (#pragma pack(1)) ----
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct PALETTE_ENTRY { pub Red: UCHAR, pub Green: UCHAR, pub Blue: UCHAR }
pub type PPALETTE_ENTRY = *mut PALETTE_ENTRY;

// ---- machine.h VIDEODISPLAYMODE enum ----
pub type VIDEODISPLAYMODE = u32;
pub const VideoTextMode: VIDEODISPLAYMODE = 0;
pub const VideoGraphicsMode: VIDEODISPLAYMODE = 1;

// ---- ui.h #defines: fill styles / screen colors ----
pub const LIGHT_FILL: UCHAR = 0xB0;
pub const MEDIUM_FILL: UCHAR = 0xB1;
pub const DARK_FILL: UCHAR = 0xB2;
pub const COLOR_BLACK: UCHAR = 0;
pub const COLOR_BLUE: UCHAR = 1;
pub const COLOR_GREEN: UCHAR = 2;
pub const COLOR_CYAN: UCHAR = 3;
pub const COLOR_RED: UCHAR = 4;
pub const COLOR_MAGENTA: UCHAR = 5;
pub const COLOR_BROWN: UCHAR = 6;
pub const COLOR_GRAY: UCHAR = 7; // nolint: const-contradiction (unrelated namespaces: an ICM colour type vs a console colour)
pub const COLOR_DARKGRAY: UCHAR = 8;
pub const COLOR_LIGHTBLUE: UCHAR = 9;
pub const COLOR_LIGHTGREEN: UCHAR = 10;
pub const COLOR_LIGHTCYAN: UCHAR = 11;
pub const COLOR_LIGHTRED: UCHAR = 12;
pub const COLOR_LIGHTMAGENTA: UCHAR = 13;
pub const COLOR_YELLOW: UCHAR = 14;
pub const COLOR_WHITE: UCHAR = 15;

// ---- ui.h / ui/tui.h box-draw chars ----
pub const HORZ: UCHAR = 0xC4;
pub const D_HORZ: UCHAR = 0xCD;
pub const VERT: UCHAR = 0xB3;
pub const D_VERT: UCHAR = 0xBA;
pub const UL: UCHAR = 0xDA;
pub const UR: UCHAR = 0xBF;
pub const LL: UCHAR = 0xC0;
pub const LR: UCHAR = 0xD9;
pub const D_UL: UCHAR = 0xC9;
pub const D_UR: UCHAR = 0xBB;
pub const D_LL: UCHAR = 0xC8;
pub const D_LR: UCHAR = 0xBC;
pub const HD_UL: UCHAR = 0xD5;
pub const HD_UR: UCHAR = 0xB8;
pub const HD_LL: UCHAR = 0xD4;
pub const HD_LR: UCHAR = 0xBE;
pub const VD_UL: UCHAR = 0xD6;
pub const VD_UR: UCHAR = 0xB7;
pub const VD_LL: UCHAR = 0xD3;
pub const VD_LR: UCHAR = 0xBD;
pub const TUI_TITLE_BOX_CHAR_HEIGHT: ULONG = 5;
pub const TITLE_BOX_CHAR_HEIGHT: ULONG = 5;
pub const ANSI_NULL: CHAR = 0;

// ---- ui.h ATTR() macro ; reactos RTL_NUMBER_OF() ----
#[inline(always)]
pub fn ATTR(fg: UCHAR, bg: UCHAR) -> UCHAR { (bg << 4) | fg }
#[inline(always)]
pub fn RTL_NUMBER_OF<T, const N: usize>(_a: *const [T; N]) -> usize { N }

// ---- machine-layer real C extern (machpc.c / uefihw.c) ----
extern "C" { pub static mut MachDefaultTextColor: UCHAR; }

// ---- keycodes.h scancodes (compared against MachConsGetCh() -> ULONG) ----
pub const KEY_EXTENDED: ULONG = 0x00;
pub const KEY_ENTER: ULONG = 0x0D;
pub const KEY_BACKSPACE: ULONG = 0x08;
pub const KEY_DELETE: ULONG = 0x53;
pub const KEY_SPACE: ULONG = 0x20;
pub const KEY_LEFTSHIFT: ULONG = 0x2A;
pub const KEY_HOME: ULONG = 0x47;
pub const KEY_UP: ULONG = 0x48;
pub const KEY_DOWN: ULONG = 0x50;
pub const KEY_LEFT: ULONG = 0x4B;
pub const KEY_RIGHT: ULONG = 0x4D;
pub const KEY_ESC: ULONG = 0x1B;
pub const KEY_CAPS_LOCK: ULONG = 0x3A;
pub const KEY_F1: ULONG = 0x3B;
pub const KEY_F2: ULONG = 0x3C;
pub const KEY_F3: ULONG = 0x3D;
pub const KEY_F4: ULONG = 0x3E;
pub const KEY_F5: ULONG = 0x3F;
pub const KEY_F6: ULONG = 0x40;
pub const KEY_F7: ULONG = 0x41;
pub const KEY_F8: ULONG = 0x42;
pub const KEY_F9: ULONG = 0x43;
pub const KEY_F10: ULONG = 0x44;
pub const KEY_F11: ULONG = 0x57;
pub const KEY_F12: ULONG = 0x58;
pub const KEY_KEYPAD_PLUS: ULONG = 0x4E;
pub const KEY_END: ULONG = 0x4F;
pub const KEY_SEND: ULONG = 0xE7;

// ---- ver.h version strings (PCSTR; const raw ptr — Copy, no Sync issue) ----
pub const VERSION: *const i8 = b"FreeLoader v3.2\0".as_ptr() as *const i8;
pub const AUTHOR_EMAIL: *const i8 = b"<www.reactos.org>\0".as_ptr() as *const i8;
pub const BY_AUTHOR: *const i8 = b"by ReactOS Project\0".as_ptr() as *const i8;

// ---- wincon.h SMALL_RECT (SHORT x4) ----
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SMALL_RECT { pub Left: SHORT, pub Top: SHORT, pub Right: SHORT, pub Bottom: SHORT }
pub type PSMALL_RECT = *mut SMALL_RECT;

// arc.h TIMEINFO — re-export the bindgen type.
pub use ntkrnl_sys::ddk::TIMEINFO;
pub use ntkrnl_sys::ddk::{PCSTR as PCSTR_, PSTR};

// Shared public freeldr UI API + machine-layer/ARC/CRT symbols the ui unit
// calls cross-module. Declared here so every `use crate::ui_keystone::*`
// site resolves the name; a module that *defines* the #[no_mangle] symbol
// shadows this glob decl with its local item (Rust glob-vs-local rule), and
// the linker unifies via #[no_mangle]. Mach* are machine.h macros over
// MachVtbl in C — resolved at the freeldr.sys link, not the staticlib.
extern "C" {
    pub fn TuiFillArea(Left: ULONG, Top: ULONG, Right: ULONG, Bottom: ULONG, FillChar: CHAR, Attr: UCHAR);
    pub fn TuiTruncateStringEllipsis(StringText: PSTR, MaxChars: u32);
    pub fn TuiDrawBoxBottomLine(Left: ULONG, Bottom: ULONG, Right: ULONG, VertStyle: UCHAR, HorzStyle: UCHAR, Attr: UCHAR);
    pub fn UiDrawBox(Left: ULONG, Top: ULONG, Right: ULONG, Bottom: ULONG, VertStyle: UCHAR, HorzStyle: UCHAR, Fill: BOOLEAN, Shadow: BOOLEAN, Attr: UCHAR);
    pub fn UiDrawBackdrop(DrawHeight: ULONG);
    pub fn UiGetScreenHeight() -> ULONG;
    pub fn VideoRestorePaletteState(Palette: PPALETTE_ENTRY, ColorCount: ULONG);
    pub fn VideoFreeOffScreenBuffer();
    pub fn VideoFadeOut(ColorCount: ULONG);
    pub fn MachVideoSetDisplayMode(DisplayMode: *const i8, Init: BOOLEAN) -> VIDEODISPLAYMODE;
    pub fn ArcGetTime() -> *mut TIMEINFO;
    pub fn RtlStringCbPrintfA(pszDest: *mut i8, cbDest: usize, pszFormat: *const i8, ...) -> NTSTATUS;
}
