//! Island-first substrate for menuband-rust's translated TUs — the same contract as
//! `shell32-rust/src/prelude.rs`, which this crate's `types`/`island`/`shellcom`/`atl` are
//! all `#[path]`-included copies of.
//!
//! WHY THIS EXISTS. menuband-rust had no import surface a generated module could use. A
//! `--target shell32` run emits `use crate::prelude::*` and there was no prelude here; a
//! `--target menuband` run emits `use com_sys::*`, which is the COM ABI only and knows
//! nothing about `IShellFolder`, PIDLs, `CSIDL_*` or `SMINIT_*` — 71 errors for five
//! functions. The substrate was never missing, only unreachable.
//!
//! THE POLICY, copied deliberately rather than reinvented (see shell32-rust's prelude for
//! the measurements behind it):
//!
//!   * The island is globbed and is the authority for layouts, constants and the Win32/shell
//!     fn surface.
//!   * **Nothing scalar or PIDL-shaped is re-exported from `shellcom`.** That is the rule
//!     that matters. Cherry-picking an alias out of the hand-written substrate into an
//!     island-first module creates a NOMINAL split the widths hide — `shellcom::LPARAM` is
//!     `isize` while the island's is `i32` on i386: same width, same meaning, different Rust
//!     type, and every island fn declaration then rejects the value. Only vtable INTERFACE
//!     types come across, because those have no island equivalent in the `lpVtbl` idiom.
//!   * ATL's smart pointers come from `atl`, by the C++ names the translated call sites use.

// The header-derived authority: struct layouts, constants, and the Win32/shell fn surface.
pub use crate::island::*;
pub use crate::own_exports_decls::*;

// Vtable INTERFACE shapes only — see the rule above. The island renders these as opaque
// bindgen C++ classes with no callable slots, so the hand-written `lpVtbl` forms are the ones
// a translated call site can actually dispatch through.
pub use crate::shellcom::{
    IBindCtx, IBindCtxVtbl, IDataObject, IDataObjectVtbl, IEnumIDList, IEnumIDListVtbl,
    IPersistFolder, IPersistFolder3, IPersistFolder3Vtbl, IPersistFolderVtbl, IShellFolder,
    IShellFolder2, IShellFolder2Vtbl, IShellFolderVtbl, IUnknown, IUnknownVtbl,
};

// `REFIID` is the ONE alias that crosses, and it is not an exception to the rule above: it is
// `*const IID` where `IID` is re-exported FROM the island (shellcom.rs:61), so there is no
// second nominal type to split against — unlike `LPARAM`, which shellcom builds on its own
// `types::LONG_PTR`.
pub use crate::shellcom::REFIID;

// Shell display-name / STRRET constants. Plain integer constants with no island equivalent,
// so there is no second nominal type for them to split against.
pub use crate::shellcom::{SHGDN_FORPARSING, SHGDN_NORMAL, STRRET_WSTR};

/// CMenuToolbars.h:29-30 — the two private window messages CMenuFocusManager posts to a
/// toolbar to ask about / change the tracked item.
pub const WM_USER_ISTRACKEDITEM: crate::island::DWORD = crate::island::WM_APP + 41;
pub const WM_USER_CHANGETRACKEDITEM: crate::island::DWORD = crate::island::WM_APP + 42;

/// CMenuFocusManager.h — the depth cap on the menu-bar/popup stack walk.
pub const MAX_RECURSE: i32 = 20;

/// winbase.h `#define TLS_OUT_OF_INDEXES ((DWORD)0xFFFFFFFF)` — what TlsAlloc returns on
/// failure. In the substrate for the same reason as ANCESTORDEFAULT below.
pub const TLS_OUT_OF_INDEXES: crate::island::DWORD = 0xFFFF_FFFF;

/// shobjidl.idl:2886 `#define ANCESTORDEFAULT (UINT)-1` — "use the ancestor's setting" for
/// IShellMenu::Initialize's uIdAncestor. Declared in the substrate, not in a worker body: the
/// deterministic audit REJECTS a body that redefines a source symbolic constant, and that is
/// the check standing between us and an invented value.
pub const ANCESTORDEFAULT: crate::island::UINT = (-1i32) as crate::island::UINT;

/// shlobj.h `SHGDN_INFOLDER` — the name relative to its folder rather than the desktop.
///
/// Declared HERE rather than added next to its two siblings in the shared `shellcom.rs`: the
/// shell32-rust modules that use it declare it locally AND glob `crate::shellcom::*`
/// alongside their own module, so a definition in shellcom is `E0659: ambiguous name` at
/// every one of those sites. Adding it there broke `cfs_folder_tu.rs` immediately.
pub const SHGDN_INFOLDER: crate::shellcom::SHGDNF = 0x0001;

// NO `atl` here, deliberately. Two reasons, and the second is the load-bearing one:
//   * nothing needs it — the modules translated into this crate use no ATL smart pointer,
//     and the two that mention `CComPtr` (menu_band, merged_folder) declare their own, and
//   * `atl.rs` reaches for `crate::win32::shell32_hInstance`, and THIS STATICLIB LINKS INTO
//     BOTH shell32.dll AND rshell.dll. Pulling it in would put a shell32-only symbol
//     reference into rshell's link for code rshell never calls.
// If a future TU here genuinely needs ATL, give this crate its own copy rather than
// path-including shell32-rust's, and keep that DLL-specific symbol out of it.

// ---------------------------------------------------------------------------------------------
// Macros the translated TUs use. Copied rather than shared: a `#[macro_export]` macro belongs
// to the crate that DEFINES it, and shell32-rust's prelude cannot be path-included here (it
// reaches for `crate::wine`, `crate::shellutils` and six more modules this crate does not
// have). They are three lines each and their definitions are fixed by the C headers.
// ---------------------------------------------------------------------------------------------

/// shellutils.h:88 `#define FAILED_UNEXPECTEDLY(hr) FAILED(hr)` in a non-debug build.
#[macro_export]
macro_rules! FAILED_UNEXPECTEDLY {
    ($hr:expr) => {
        (($hr) as $crate::island::HRESULT) < 0
    };
}

/// CRT `_ASSERT` — a no-op in a shipping build, exactly as the C++ compiles it here. It must
/// stay a MACRO: an out-of-line no-op fn is an optimization barrier that has uncloaked latent
/// ABI mismatches in this tree before.
#[macro_export]
macro_rules! _ASSERT {
    ($($arg:tt)*) => {{}};
}

/// minwindef.h `#define MAKELPARAM(l, h)` — the two `& 0xffff` truncations are load-bearing.
#[macro_export]
macro_rules! MAKELPARAM {
    ($l:expr, $h:expr) => {
        ((((($l) as u32) & 0xffff) | (((($h) as u32) & 0xffff) << 16)) as $crate::island::LPARAM)
    };
}

pub use crate::{FAILED_UNEXPECTEDLY, MAKELPARAM, _ASSERT};

// ---------------------------------------------------------------------------------------------
// shell32.dll entry points the island does not carry. shell32-rust reaches these through its
// own `crate::wine::*` modules, which this crate does not have; both DLLs that link this
// staticlib (shell32.dll itself, and rshell.dll via shell32's import lib) resolve them.
// Signatures from shlobj.h / shellapi.h / shlwapi.h.
// ---------------------------------------------------------------------------------------------

/// guiddef.h `REFGUID` — `const GUID &` in C++, a pointer at the ABI.
pub type REFGUID = *const crate::island::GUID;

extern "system" {
    pub fn SHGetSpecialFolderLocation(
        hwndOwner: HWND,
        nFolder: INT,
        ppidl: *mut LPITEMIDLIST,
    ) -> HRESULT;
    pub fn SHGetSpecialFolderPathW(
        hwndOwner: HWND,
        szPath: LPWSTR,
        nFolder: INT,
        bCreate: BOOL,
    ) -> BOOL;
    pub fn SHGetFolderLocation(
        hwndOwner: HWND,
        nFolder: INT,
        hToken: HANDLE,
        dwReserved: DWORD,
        ppidl: *mut LPITEMIDLIST,
    ) -> HRESULT;
    pub fn SHBindToParent(
        pidl: *const ITEMIDLIST,
        riid: crate::shellcom::REFIID,
        ppv: *mut *mut core::ffi::c_void,
        ppidlLast: *mut *const ITEMIDLIST,
    ) -> HRESULT;
    pub fn ShellExecuteW(
        hwnd: HWND,
        lpOperation: LPCWSTR,
        lpFile: LPCWSTR,
        lpParameters: LPCWSTR,
        lpDirectory: LPCWSTR,
        nShowCmd: INT,
    ) -> HINSTANCE;
    /// shlwapi ordinal — the shell policy/restriction query.
    pub fn SHRestricted(rest: DWORD) -> DWORD;
    /// shell32 ordinal 72.
    pub fn Shell_GetCachedImageIndex(szPath: LPCWSTR, nIndex: INT, bSimulateDoc: BOOL) -> INT;
    pub fn ILFree(pidl: LPITEMIDLIST);
}

/// CMenuFocusManager's class-static TLS slot, re-exported so the three translated bodies that
/// share it (`GetManager`, `AcquireManager`, `ReleaseManager`) see one storage location.
pub use crate::menu_focus_manager_support::TlsIndex;
