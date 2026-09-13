//! Hand-written substrate for `utils.cpp`'s translation — NOT a translated TU.
//!
//! SDK constants and header inlines outside the bindgen island's closure. Every value is copied
//! from the header that defines it, and the ones C composes from other constants keep the same
//! EXPRESSION rather than a hand-evaluated number.

use crate::prelude::*;

// ------------------------------------------------------------------------------------------
// winnt.h / winnls.h
// ------------------------------------------------------------------------------------------

/// `MAKELANGID(LANG_NEUTRAL, SUBLANG_DEFAULT)` = `(SUBLANG_DEFAULT << 10) | LANG_NEUTRAL`.
pub const LANG_NEUTRAL: WORD = 0x00;
pub const SUBLANG_DEFAULT: WORD = 0x01;
pub const LANG_USER_DEFAULT: WORD = (SUBLANG_DEFAULT << 10) | LANG_NEUTRAL;
pub const SORT_DEFAULT: WORD = 0x0;
/// `MAKELCID(lgid, srtid)` = `(srtid << 16) | lgid`.
pub const LOCALE_USER_DEFAULT: LCID = ((SORT_DEFAULT as LCID) << 16) | (LANG_USER_DEFAULT as LCID);

/// `#define ANSI_NULL ((CHAR)0)` — winnt.h.
pub const ANSI_NULL: CHAR = 0;

/// `#define MAXUINT ((UINT)~((UINT)0))` — intsafe.h.
pub const MAXUINT: UINT = UINT::MAX;

/// `#define SE_SHUTDOWN_NAME TEXT("SeShutdownPrivilege")` — winnt.h. Written out as an array
/// with the length COMPUTED rather than typed; a hand-counted `[u16; N]` that loses its NUL is
/// the bug the crate's wide-literal rule exists to stop.
static SE_SHUTDOWN_NAME_W: [u16; 20] = [
    'S' as u16, 'e' as u16, 'S' as u16, 'h' as u16, 'u' as u16, 't' as u16, 'd' as u16,
    'o' as u16, 'w' as u16, 'n' as u16, 'P' as u16, 'r' as u16, 'i' as u16, 'v' as u16,
    'i' as u16, 'l' as u16, 'e' as u16, 'g' as u16, 'e' as u16, 0,
];

#[inline]
pub fn SE_SHUTDOWN_NAME() -> PCWSTR {
    SE_SHUTDOWN_NAME_W.as_ptr()
}

// ------------------------------------------------------------------------------------------
// objbase.h / combaseapi.h
// ------------------------------------------------------------------------------------------

pub const COINIT_APARTMENTTHREADED: DWORD = 0x2;
pub const COINIT_DISABLE_OLE1DDE: DWORD = 0x4;
pub const CLSCTX_INPROC_SERVER: DWORD = 0x00000001;

/// `HRESULT_FROM_WIN32(ERROR_INSUFFICIENT_BUFFER)`. The win32 code itself is NOT redefined
/// here -- query_associations.rs already publishes it, and a second glob-visible copy is an
/// ambiguity error rather than a harmless duplicate.
pub const E_NOT_SUFFICIENT_BUFFER: HRESULT = 0x8007_007Au32 as HRESULT; // 122

// ------------------------------------------------------------------------------------------
// oleidl.h / objidl.h / shobjidl.h
// ------------------------------------------------------------------------------------------

pub const OLECONTF_LINKS: DWORD = 2;
pub const STATFLAG_NONAME: DWORD = 1;
pub const SHGDN_INFOLDER: UINT = 1;

/// shellapi.h resolves `GCS_VERB` through WINELIB_NAME_AW; shell32 is a Unicode build.
/// `GCS_VERBW` itself comes from the ISLAND -- redefining it here made the name ambiguous
/// across two glob imports, which silently turned a `match` arm into a binding pattern.
pub const GCS_VERB: UINT = crate::island::GCS_VERBW;

/// `#define CMIC_MASK_FLAG_NO_UI SEE_MASK_FLAG_NO_UI` — shellapi.h. Same: the island owns
/// SEE_MASK_FLAG_NO_UI; only the alias belongs here.
pub const CMIC_MASK_FLAG_NO_UI: DWORD = crate::island::SEE_MASK_FLAG_NO_UI;

/// `#define CMIC_MASK_UNICODE SEE_MASK_UNICODE` — shellapi.h.
pub const SEE_MASK_UNICODE: DWORD = 0x00004000;
pub const CMIC_MASK_UNICODE: DWORD = SEE_MASK_UNICODE;

/// The C++ declares `CUnknownVtbl`/`CUnknown` LOCAL to SHIsBadInterfacePtr; Rust has no
/// function-local struct with methods, so they lift to module scope. Only `Release` is
/// actually read (the probe checks it is a valid code pointer).
#[repr(C)]
pub struct CUnknownVtbl {
    pub QueryInterface: *mut c_void,
    pub AddRef: *mut c_void,
    pub Release: *mut c_void,
}

#[repr(C)]
pub struct CUnknown {
    pub lpVtbl: *const CUnknownVtbl,
}

/// shlobj.h RESTRICTIONS member.
pub const REST_ALLOWCOMMENTTOGGLE: DWORD = 0x41000004;

/// `EXTENDED_NAME_FORMAT::NameDisplay` — secext.h.
pub const NameDisplay: c_int = 3;

/// `#define COPY_STREAM_DEFAULT_BUFFER_SIZE 0x4000` — utils.cpp's own.
pub const COPY_STREAM_DEFAULT_BUFFER_SIZE: DWORD = 0x4000;

/// `NET_API_STATUS` is a DWORD (lmcons.h).
pub type NET_API_STATUS = DWORD;

/// `#define LOWORD(l) ((WORD)(((DWORD_PTR)(l)) & 0xffff))` — minwindef.h. A macro because the
/// argument is any integer-ish expression at the call sites.
macro_rules! LOWORD {
    ($l:expr) => {
        (($l as usize) & 0xFFFF) as WORD
    };
}
pub(crate) use LOWORD;

// ------------------------------------------------------------------------------------------
// utils.h `struct ClipboardViewerChain` — a HEADER-ONLY C++ struct
// ------------------------------------------------------------------------------------------

/// `winuser.h` `#define HWND_BOTTOM ((HWND)1)` — a cast macro, so no island can carry it.
/// It is a SENTINEL here, not a window: `ClipboardViewerChain` uses it to mean "not hooked",
/// which is why the distinction from a null `HWND` is load-bearing (see below).
pub const HWND_BOTTOM: HWND = 1 as HWND;

/// `utils.h:132`. bindgen renders the STRUCT and none of the methods, because they are defined
/// inline in the header — the same shape as `ui/rosctrls.h`'s CListView, and handled the same
/// way: the layout stays the island's, the behaviour is written here against it.
///
/// The C++ struct carries a DEFAULT MEMBER INITIALIZER, `HWND m_hWndNext = HWND_BOTTOM`, and
/// that is not decoration. `Hook` only hooks when `m_hWndNext == HWND_BOTTOM`, so a
/// zero-initialized chain never hooks at all and `Unhook` passes a null next-window to
/// ChangeClipboardChain. A translated owner must therefore use `new()` rather than
/// `core::mem::zeroed()` for this field.
impl ClipboardViewerChain {
    #[inline]
    pub const fn new() -> Self {
        Self { m_hWndNext: HWND_BOTTOM }
    }

    pub unsafe fn Unhook(&mut self, hWnd: HWND) {
        if self.m_hWndNext != HWND_BOTTOM {
            ChangeClipboardChain(hWnd, self.m_hWndNext);
        }
        self.m_hWndNext = HWND_BOTTOM;
    }

    pub unsafe fn Hook(&mut self, hWnd: HWND) {
        if self.m_hWndNext == HWND_BOTTOM {
            self.m_hWndNext = SetClipboardViewer(hWnd);
        }
    }

    pub unsafe fn HandleChangeCBChain(&mut self, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
        if self.m_hWndNext == wParam as HWND {
            self.m_hWndNext = lParam as HWND;
            return self.m_hWndNext as LRESULT;
        } else if !self.m_hWndNext.is_null() && self.m_hWndNext != HWND_BOTTOM {
            return SendMessageW(self.m_hWndNext, WM_CHANGECBCHAIN, wParam, lParam);
        }
        0
    }

    pub unsafe fn HandleDrawClipboard(&mut self, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
        if !self.m_hWndNext.is_null() && self.m_hWndNext != HWND_BOTTOM {
            return SendMessageW(self.m_hWndNext, WM_DRAWCLIPBOARD, wParam, lParam);
        }
        0
    }
}

impl Default for ClipboardViewerChain {
    fn default() -> Self {
        Self::new()
    }
}
