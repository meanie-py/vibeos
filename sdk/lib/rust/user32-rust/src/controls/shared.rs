//! controls/shared.rs — symbols shared between the combo and listbox controls
//! (the only genuinely cross-control surface). Sourced from
//! win32ss/user/user32/include/controls.h. Per-file Win32 API extern blocks and
//! the listbox `IS_OWNERDRAW`/`HAS_STRINGS`/`IS_MULTISELECT` macros stay local to
//! their file (matching the windows/ core convention) — only the combo HEADCOMBO
//! state struct and the combo/listbox message+flag constants live here.

use win32k_sys::win32k::*;

// ─── HEADCOMBO — the combobox internal state (controls.h) ─────────────────────
// `self` is a Rust keyword → `self_` (combo.rs/listbox.rs reference `.self_`).
#[repr(C)]
pub struct HEADCOMBO {
    pub self_: HWND,
    pub owner: HWND,
    pub dwStyle: UINT,
    pub hWndEdit: HWND,
    pub hWndLBox: HWND,
    pub wState: UINT,
    pub hFont: HFONT,
    pub textRect: RECT,
    pub buttonRect: RECT,
    pub droppedRect: RECT,
    pub droppedIndex: INT,
    pub fixedOwnerDrawHeight: INT,
    pub droppedWidth: INT,
    pub editHeight: INT,
    pub UIState: LONG,
}
pub type LPHEADCOMBO = *mut HEADCOMBO;

// ─── CBF_* combo state flags (controls.h) ─────────────────────────────────────
pub const CBF_DROPPED: UINT = 0x0001;
pub const CBF_BUTTONDOWN: UINT = 0x0002;
pub const CBF_NOROLLUP: UINT = 0x0004;
pub const CBF_MEASUREITEM: UINT = 0x0008;
pub const CBF_FOCUSED: UINT = 0x0010;
pub const CBF_CAPTURE: UINT = 0x0020;
pub const CBF_EDIT: UINT = 0x0040;
pub const CBF_NORESIZE: UINT = 0x0080;
pub const CBF_NOTIFY: UINT = 0x0100;
pub const CBF_NOREDRAW: UINT = 0x0200;
pub const CBF_SELCHANGE: UINT = 0x0400;
pub const CBF_NOEDITNOTIFY: UINT = 0x1000;
pub const CBF_NOLBSELECT: UINT = 0x2000;
pub const CBF_BEENFOCUSED: UINT = 0x4000;
pub const CBF_EUI: UINT = 0x8000;

// ─── Undocumented ReactOS listbox messages (controls.h / undocuser.h) ─────────
pub const LB_INSERTSTRING_UPPER: UINT = 0x1AA;
pub const LB_INSERTSTRING_LOWER: UINT = 0x1AB;
pub const LB_ADDSTRING_UPPER: UINT = 0x1AC;
pub const LB_ADDSTRING_LOWER: UINT = 0x1AD;
// LB_CARETON/LB_CARETOFF are already in win32k_sys::win32k — do NOT redefine here
// (would make the name ambiguous under the two glob imports). Use the win32k_sys ones.
