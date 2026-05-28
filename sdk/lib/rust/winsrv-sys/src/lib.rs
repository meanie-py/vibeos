//! winsrv-sys — FFI declarations for winsrv.dll's USER + Console
//! server stack (consrv + usersrv + concfg static libs aggregated
//! into winsrv.dll, plus the top-level init.c).
//!
//! Auto-generated via `bindgen` for all 3 architectures from a wrapper
//! that pulls `concfg/precomp.h`, `winsrv.h`, `consrv/consrv.h` and
//! `usersrv/usersrv.h`. The user-mode analog of `ntkrnl-sys`/`win32k-sys`
//! for the Phase 5 winsrv translation (see memory
//! `project_phase5_winsrv_audit`).
//!
//! winsrv-sys is a SUPERSET of `ntdll-sys`'s surface: the win-server
//! stack needs windowing/console/imm types that ntdll-sys deliberately
//! stops short of. To avoid the duplicate-extern noise that would
//! follow from re-bindgen'ing the NDK twice, this crate re-exports
//! `ntdll_sys::nt` and the winsrv-specific bindings sit ALONGSIDE
//! it under `pub mod winsrv`. The translator picks the right umbrella
//! per call site: `nt::*` for Nt/Zw/Rtl/Csr/Sm/Base, `winsrv::*` for
//! consrv/usersrv/concfg / wincon/winuser/wingdi.
//!
//! NOT a standalone static library — an `rlib` declaration dependency.

#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(clippy::upper_case_acronyms)]
#![allow(dead_code)]
// Bindgen re-declares NT/CSR types pulled via winsrv.h's NDK + csrsrv
// includes. Within this crate those are self-consistent; the
// type-identity split vs `ntdll_sys::nt` is handled by casts at the
// boundary (same pattern as `ntkrnl_sys::ddk` vs `win32k_sys::win32k`).
#![allow(clashing_extern_declarations)]

mod bindgen_common;

#[cfg(target_arch = "aarch64")]
mod bindgen_arm64;

#[cfg(target_arch = "x86_64")]
mod bindgen_amd64;

#[cfg(target_arch = "x86")]
mod bindgen_i386;

/// Re-export ONLY the hand-written user-mode helpers from ntdll-sys
/// (FORCEINLINE accessors + STATUS_* constants). NOT a glob of
/// `ntdll_sys::nt::*` — that would re-publish HKEY/HANDLE/LPWSTR/
/// MAX_PATH etc. that the winsrv-sys bindgen already defines (via
/// the windef.h chain pulled by winsrv.h/consrv.h/usersrv.h/
/// concfg/precomp.h), making any `use winsrv_sys::nt::*` glob
/// E0659-ambiguous against the winsrv-side definitions. With this
/// shape winsrv-rust can rely on a SINGLE glob: `use
/// winsrv_sys::winsrv::*` — every NT/CSR/SM/BASE fn from the
/// bindgen below resolves through the same module, and the
/// hand-written inlines are namespaced under `winsrv_sys::nt`.
pub mod nt {
    pub use ntdll_sys::nt::{
        NtCurrentPeb, NtCurrentProcess, NtCurrentTeb, NtCurrentThread,
        ZwCurrentProcess, ZwCurrentThread,
        InitializeListHead, IsListEmpty, InsertHeadList, InsertTailList,
        RemoveEntryList, RemoveHeadList, RemoveTailList,
    };
    // STATUS_* hand-written codes (ntstatus.h cast-macros are not
    // bindgen-evaluable, so they live in ntdll-sys::constants).
    // Re-export ALL of them — they're pure `pub const … = i32 = …;`
    // entries, no collision risk with the winsrv bindgen.
    pub use ntdll_sys::nt::{
        STATUS_SUCCESS,
        STATUS_UNSUCCESSFUL, STATUS_INVALID_HANDLE, STATUS_INVALID_PARAMETER,
        STATUS_ACCESS_DENIED, STATUS_OBJECT_TYPE_MISMATCH,
        STATUS_OBJECT_NAME_COLLISION, STATUS_QUOTA_EXCEEDED,
        STATUS_PROCESS_IS_TERMINATING, STATUS_HANDLE_NOT_CLOSABLE,
        STATUS_OBJECT_NAME_EXISTS, STATUS_OBJECT_NAME_NOT_FOUND,
        STATUS_PORT_ALREADY_SET, STATUS_PORT_DISCONNECTED,
        STATUS_REPLY_MESSAGE_MISMATCH, STATUS_INVALID_PORT_HANDLE,
        STATUS_PORT_MESSAGE_TOO_LONG, STATUS_SERVER_SID_MISMATCH,
        STATUS_PORT_CONNECTION_REFUSED, STATUS_PORT_NOT_SET,
        STATUS_INVALID_PARAMETER_3, STATUS_INVALID_PARAMETER_4,
        STATUS_LPC_REPLY_LOST, STATUS_OBJECT_NAME_INVALID,
        STATUS_OBJECT_PATH_NOT_FOUND, STATUS_OBJECT_PATH_INVALID,
        STATUS_OBJECT_PATH_SYNTAX_BAD, STATUS_INFO_LENGTH_MISMATCH,
        STATUS_BUFFER_TOO_SMALL, STATUS_BUFFER_OVERFLOW,
        STATUS_NOT_IMPLEMENTED, STATUS_NO_MEMORY,
        STATUS_LOGON_SESSION_EXISTS, STATUS_NOT_LOGON_PROCESS,
        STATUS_INSUFFICIENT_RESOURCES, STATUS_NO_TOKEN,
        STATUS_TIMEOUT, STATUS_INVALID_PARAMETER_MIX,
    };
}

/// Hand-written constants for winsrv that aren't bindgen-derivable
/// (cast-macros in ntstatus.h, #defines in headers bindgen doesn't
/// follow, etc.).
pub mod consts {
    /// `USER_VERSION = MAKELONG(0x0000, 0x0005)` — passed to
    /// `NtUserInitialize`. ntuser.h:1149.
    pub const USER_VERSION: u32 = 0x0005_0000;

    /// `DWLP_USER` — winuser.h GetWindowLongPtr offset for user
    /// data. The DWLP_USER value depends on pointer size:
    /// DLGWINDOWEXTRA + sizeof(LRESULT) + sizeof(DLGPROC).
    /// i386: 8 + 4 + 4 = 16 → DLGWINDOWEXTRA(30) + 4 + 4 = 38.
    /// Actually: `#define DWLP_MSGRESULT 0` `#define DWLP_DLGPROC
    /// (DWLP_MSGRESULT + sizeof(LRESULT))` `#define DWLP_USER
    /// (DWLP_DLGPROC + sizeof(DLGPROC))`. sizeof(LRESULT) =
    /// sizeof(DLGPROC) = ptr size, so on i386 = 0 + 4 + 4 = 8.
    /// On amd64/arm64 = 0 + 8 + 8 = 16.
    #[cfg(target_pointer_width = "32")]
    pub const DWLP_USER: i32 = 8;
    #[cfg(target_pointer_width = "64")]
    pub const DWLP_USER: i32 = 16;

    /// `IDC_PROGRESS` — commctrl.h resource id for the progress
    /// control. Used by usersrv shutdown progress dialog.
    pub const IDC_PROGRESS: i32 = 100;

    /// `STATUS_SERVICE_NOTIFICATION` — used by usersrv harderror.c
    /// to detect service-notification dialog mode (no host process).
    /// ntstatus.h: 0x40000018.
    pub const STATUS_SERVICE_NOTIFICATION: i32 = 0x40000018u32 as i32;

    /// `LANGIDFROMLCID(lcid)` — macro `(WORD)(lcid)`.
    #[inline]
    pub fn LANGIDFROMLCID(lcid: u32) -> u16 {
        (lcid & 0xFFFF) as u16
    }

    /// `NT_SUCCESS(Status)` — macro `(((NTSTATUS)(Status)) >= 0)`.
    #[inline]
    pub fn NT_SUCCESS(status: i32) -> bool {
        status >= 0
    }
}

/// Hand-written type aliases for winsrv that bindgen sees only under
/// their `_USERCONNECT` original names — the win/winmsg.h `#define`
/// chain (`#define _USERSRV_API_CONNECTINFO _USERCONNECT`) isn't
/// followed by bindgen.
pub mod aliases {
    pub use crate::winsrv::{
        _USERCONNECT as _USERSRV_API_CONNECTINFO,
        _USERCONNECT as USERSRV_API_CONNECTINFO,
        USERCONNECT,
        PUSERCONNECT as PUSERSRV_API_CONNECTINFO,
    };
}

/// winsrv-internal API — auto-generated bindings with correct per-arch
/// calling conventions (i386 stdcall/fastcall decoration too).
///
/// Covers the consrv driver core (`ConDrv*`/`ConSrv*`), the
/// TERMINAL/FRONTEND vtables and their impls (Term* in tui/gui/serial
/// frontends), the concfg surface (`TT*`/`Settings*`/`Font*`), and the
/// usersrv surface (`UserServer*`/`UserClient*`/`UserInit*`/
/// `UserSound*`). Also the wincon/winuser/wingdi/imm types these
/// internals project (CONSOLE_*/COORD/INPUT_RECORD/HWND/HDC/LOGFONTW/
/// WNDPROC/MSG/…).
pub mod winsrv {
    pub use super::bindgen_common::*;

    #[cfg(target_arch = "aarch64")]
    pub use super::bindgen_arm64::*;

    #[cfg(target_arch = "x86_64")]
    pub use super::bindgen_amd64::*;

    #[cfg(target_arch = "x86")]
    pub use super::bindgen_i386::*;
}
