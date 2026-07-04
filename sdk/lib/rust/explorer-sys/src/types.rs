//! Explorer-wide shared C++ types with EXACT `#[repr(C)]` layout mirrors of
//! their base/shell/explorer/precomp.h definitions. These are referenced
//! across many .cpp files (g_TaskbarSettings alone is touched by traywnd,
//! trayclock, taskswnd, syspager, trayntfy, startmnucust, …), so they live in
//! the shared substrate rather than any one translated module. Layout is
//! load-bearing — g_TaskbarSettings is serialized to the registry as raw
//! bytes (TW_STUCKRECTS2) and shared with still-C++ code during cohab.

use winsrv_sys::winsrv::{BOOL, DWORD, LONG, RECT, SIZE};

/// `enum TrayIconsMode` (precomp.h). C++ unscoped enum → i32.
pub const TIM_Default: i32 = 0;
pub const TIM_NeverCompact: i32 = 1;
pub const TIM_AlwaysCompact: i32 = 2;
pub const TIM_Max: i32 = TIM_AlwaysCompact;

/// `TW_STRUCKRECTS2` (precomp.h `_TW_STUCKRECTS2`). The anonymous
/// `union { DWORD dwFlags; struct{ bitfields } }` is represented as the
/// `dwFlags` word (same 4 bytes); the four 1-bit flags are bit 0..3.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct TW_STRUCKRECTS2 {
    pub cbSize: DWORD,
    pub Unknown: LONG,
    pub dwFlags: DWORD,
    pub Position: DWORD,
    pub Size: SIZE,
    pub Rect: RECT,
}

impl TW_STRUCKRECTS2 {
    // Bitfield accessors matching the C++ `DWORD field : 1` order.
    pub const F_AUTOHIDE: DWORD = 1 << 0;
    pub const F_ALWAYSONTOP: DWORD = 1 << 1;
    pub const F_SMSMALLICONS: DWORD = 1 << 2;
    pub const F_HIDECLOCK: DWORD = 1 << 3;

    #[inline]
    pub fn SmSmallIcons(&self) -> DWORD {
        (self.dwFlags & Self::F_SMSMALLICONS) >> 2
    }
    #[inline]
    pub fn set_SmSmallIcons(&mut self, v: DWORD) {
        self.dwFlags = (self.dwFlags & !Self::F_SMSMALLICONS) | ((v & 1) << 2);
    }
    #[inline]
    pub fn HideClock(&self) -> DWORD {
        (self.dwFlags & Self::F_HIDECLOCK) >> 3
    }
    #[inline]
    pub fn set_HideClock(&mut self, v: DWORD) {
        self.dwFlags = (self.dwFlags & !Self::F_HIDECLOCK) | ((v & 1) << 3);
    }
    #[inline]
    pub fn AutoHide(&self) -> DWORD {
        self.dwFlags & Self::F_AUTOHIDE
    }
    #[inline]
    pub fn AlwaysOnTop(&self) -> DWORD {
        (self.dwFlags & Self::F_ALWAYSONTOP) >> 1
    }
}

/// `struct TaskbarSettings` (precomp.h). `eCompactTrayIcons` is
/// `enum TrayIconsMode` → i32.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct TaskbarSettings {
    pub bLock: BOOL,
    pub bGroupButtons: BOOL,
    pub bShowSeconds: BOOL,
    pub bPreferDate: BOOL,
    pub bHideInactiveIcons: BOOL,
    pub bSmallIcons: BOOL,
    pub eCompactTrayIcons: i32,
    pub bShowDesktopButton: BOOL,
    pub sr: TW_STRUCKRECTS2,
}

impl TaskbarSettings {
    /// `TaskbarSettings::UseCompactTrayIcons` (inline in precomp.h).
    #[inline]
    pub fn UseCompactTrayIcons(&self) -> BOOL {
        match self.eCompactTrayIcons {
            x if x == TIM_NeverCompact => 0,
            x if x == TIM_AlwaysCompact => 1,
            _ => self.bSmallIcons,
        }
    }
}
