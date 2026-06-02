//! Shared types from userinit.h + livecd.c, dropped by the per-file
//! translation (rustconv only captures each .c's top-of-file preamble, so
//! the shared header structs/enums vanish). Both userinit.rs and livecd.rs
//! `use crate::types::*` so `*mut STATE` is one type identity across the
//! file boundary (wWinMain → RunLiveCD passes &mut STATE).
//!
//! C enums are used as bare int values in the translated code
//! (`State.Run = SHELL`, `match State.Run { SHELL => ... }`), so they are
//! plain consts over an INT alias, matching C enum-as-int semantics — not
//! Rust enums (which would need `RUN::SHELL` paths the LLM didn't emit).

use winsrv_sys::winsrv::*;

// typedef enum { LOCALEPAGE, STARTPAGE, DONE } PAGESTATE;
pub type PAGESTATE = INT;
pub const LOCALEPAGE: PAGESTATE = 0;
pub const STARTPAGE: PAGESTATE = 1;
pub const DONE: PAGESTATE = 2;

// typedef enum { SHELL, INSTALLER, REBOOT } RUN;
pub type RUN = INT;
pub const SHELL: RUN = 0;
pub const INSTALLER: RUN = 1;
pub const REBOOT: RUN = 2;

// typedef struct _IMGINFO { HBITMAP; INT cxSource; INT cySource; INT iPlanes; INT iBits; } IMGINFO, *PIMGINFO;
#[repr(C)]
#[derive(Clone, Copy)]
pub struct _IMGINFO {
    pub hBitmap: HBITMAP,
    pub cxSource: INT,
    pub cySource: INT,
    pub iPlanes: INT,
    pub iBits: INT,
}
pub type IMGINFO = _IMGINFO;
pub type PIMGINFO = *mut _IMGINFO;

// typedef struct _LIVECD_UNATTEND { BOOL bEnabled; LCID LocaleID; } LIVECD_UNATTEND;
#[repr(C)]
#[derive(Clone, Copy)]
pub struct _LIVECD_UNATTEND {
    pub bEnabled: BOOL,
    pub LocaleID: LCID,
}
pub type LIVECD_UNATTEND = _LIVECD_UNATTEND;

// typedef struct { PAGESTATE NextPage; RUN Run; IMGINFO ImageInfo; struct _LIVECD_UNATTEND* Unattend; } STATE, *PSTATE;
#[repr(C)]
#[derive(Clone, Copy)]
pub struct STATE {
    pub NextPage: PAGESTATE,
    pub Run: RUN,
    pub ImageInfo: IMGINFO,
    pub Unattend: *mut _LIVECD_UNATTEND,
}
pub type PSTATE = *mut STATE;

// extern HINSTANCE hInstance; — defined once in userinit.c, extern in
// livecd.c via userinit.h. Single crate-wide definition both modules share.
pub static mut hInstance: HINSTANCE = core::ptr::null_mut();
