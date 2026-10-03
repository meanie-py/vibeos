// rustconv-assembly: free-functions
//! `dll/win32/shell32/utils.cpp` -- all 95 free functions, translated, present, and compiling
//! clean on i686/amd64/arm64.
//! COHAB-FLIPPED 2026-09-19: `utils.cpp` is out of `dll/win32/shell32/CMakeLists.txt` and this
//! module is the shipping implementation of its 95 free functions.
//!
//! Wiring and the flip had to be ONE change, and the ORPHAN-BY-DESIGN guard that used to stand
//! here was recording why. Wired alone, with the `.cpp` still compiled, this module's
//! `#[no_mangle]` names shadowed the live C++ ones with no duplicate-symbol error -- the linker
//! resolves archive members against object files and simply picks one -- so live entry points
//! rerouted into Rust that had never run. Measured: the guest booted, passed smoke on all three
//! arches, then froze between `[EXPLORER] taskbar: OnCreate begin` and `child controls
//! created`, so `ProcessStartupItems` never ran and the kmtest sweep launched 0 of 91. Smoke
//! cannot see that; `vibeos-cli conformance` can, and it is the oracle for this class of change.
//!
//! What the flip actually needed was SMALLER than the guard predicted. Measured against the
//! LIVE CMake source list plus libshell32_rust.a, 36 symbols leave this TU and this module
//! publishes 32 of them. The guard said all 16 `*_RustBridge` wrappers needed rehoming; 14 are
//! published right here. Only the `CStubWindow32` create/destroy pair moved, to
//! `dll/win32/shell32/rust_stubwindow32_stub.cpp`, because only that pair reaches a C++ ATL
//! class that propsheet.cpp and folders/CDrivesFolder.cpp still construct -- and an ATL
//! `CWindowImpl`'s thunk and registered class must have exactly one owner. (Once those two were
//! Rust too, the class followed: crate::atl_stub_windows, 2026-09-24.) The last two,
//! `SHELL_GetCommandStringImpl` and `SHELL_MapContextMenuVerbToCmdId`, had their utils.h
//! declarations made EXTERN_C so shell32.cpp names the unmangled symbols emitted here.
//!
//! One measurement trap worth keeping: an intra-DLL undefined-symbol sweep does NOT see .spec
//! exports, because those are consumed by other modules entirely. `SHOpenEffectiveToken`
//! (SHELL32.235) is one, and it was `verify_lost_exports` that said so -- after the banner came
//! off. Which is the other half of the lesson: `scan_extern_arity`, `scan_cc_mismatch`,
//! `scan_silent_failures` and `verify_lost_exports` all SKIP a file carrying an
//! `ORPHAN-BY-DESIGN` banner, so a stale one silently switches four checks off.
//!
//!
//! Bought 2026-09-13 for $1.17 (97 workers; 91 replayed from bank at $0 on the retry pass),
//! zero AUDIT REJECTs. Six workers failed on TRANSPORT the first time -- "peer closed
//! connection without sending complete message body" -- which is NOT an audit rejection and
//! must not be treated as one: nothing was banked for them, so they were clean cache misses
//! and the retry cost $0.09.
//!
//!
//! RE-VERIFYING the zero-error claim (it will rot otherwise -- that is how this file reached
//! 799 errors once): wire it temporarily, `cargo check --target {i686-pc-windows-gnu,
//! x86_64-pc-windows-gnu,aarch64-pc-windows-gnullvm}` for BOTH shell32-rust and menuband-rust
//! (which #[path]-includes types.rs/shellcom.rs), then UN-WIRE before building, gating or
//! landing. Verified clean on all six combinations 2026-09-19.
//! Things worth carrying to the next TU:
//!
//! - **Vtable slot ORDER is the whole ABI contract and nothing checks it.** Call sites resolve
//!   by NAME, so a misplaced slot calls the wrong method with the wrong arguments and compiles
//!   perfectly. objidl.idl's `IBindCtx` is the live trap: it declares RemoteSetBindOptions and
//!   RemoteGetBindOptions as `[call_as(...)]` RPC proxy SUBSTITUTES, not extra slots. Counting
//!   them gives 15 and shifts everything after SetBindOptions by two.
//!
//! - **Check before authoring.** The estimate was "six vtables to write"; six of the nine
//!   opaque interfaces already had a full vtable elsewhere in the crate and are exercised by
//!   their own modules. Likewise "blocked on substrate: BIND_OPTS / STATSTG / LARGE_INTEGER /
//!   ULARGE_INTEGER are in neither shellcom nor the island" was WRONG and the fix was an import
//!   -- all four already existed. Search the CRATE, not the two places you expect.
//!
//! - **Three-target agreement is on `expected` ALONE.** `found` is what the expression
//!   currently IS, which legitimately differs by target without the repair differing at all.
//!   Comparing the whole (expected, found) pair wrongly quarantined 5 sites whose fix was
//!   identical everywhere. Where `expected` ITSELF differed by target, the cast is the callee's
//!   declared TYPE NAME (`as SIZE_T`, `as UINT_PTR`), width-correct on all three by construction.
//!
//! Real bugs found here, each a class rather than a one-off:
//!
//! - `STR_FILE_SYS_BIND_DATA` is `L"File System Bind Data"` in shobjidl.idl, and the island
//!   carries that name as `&[u8; 22]` -- the NARROW bytes. The C casts the macro to `LPWSTR`
//!   for `IBindCtx::GetObjectParam`, so the island's array read back as UTF-16 is `"iF"`,
//!   `"el"`, `" S"`... and the bind-context lookup would never match. The wide spelling is in
//!   utils_support.rs. The island has 111 such `&[u8; N]` consts; this one is proven wrong.
//! - `CDummyOleWindow` came out of the run as a LAYOUT with no object behind it, because
//!   `new CDummyOleWindow()` has no Rust spelling in a no_std crate. It is implemented here
//!   the way the rest of shell32-rust does a COM object: HeapAlloc + a static vtable,
//!   refcount from 1, `delete this` as HeapFree.
//! - `SHELL_ErrorBoxHelper` is a shellutils.h `static inline`, so there is no symbol to import
//!   and `SHELL_ErrorBox_RustBridge` had nothing to call. Ported into utils_support.rs.
//! - `CStubWindow32_{Create,Destroy}Stub_RustBridge` were REMOVED, not repaired -- see above.
//! - A `Vec<u16>` in SHSetUnreadMailCountW, in a no_std crate with no allocator -- and wrong
//!   even where Vec exists, since `with_capacity` leaves len at 0 and every write went through
//!   `as_mut_ptr()` past the initialized region. Now `CStringW` + `Append`.
//! - `FARPROC` is `Option<unsafe extern "system" fn() -> INT_PTR>`, so a null test on one is
//!   `.is_none()` and a call is a `match`, never `as *const ()` / `.unwrap()`.
use crate::prelude::*;
use crate::atl::CStringW;
use crate::shell32::utils_support::*;
use crate::shellutils::IsUnicode;
use crate::default_context_menu::{IExecuteCommand, IExecuteCommandVtbl};
use crate::control_panel_folder::{IDefaultExtractIconInit, IDefaultExtractIconInitVtbl};
use crate::shell32::shlfileop::{IProgressDialog, IProgressDialogVtbl};
use crate::shellcom::{IContextMenu, IContextMenuVtbl, IStream, IStreamVtbl};
use crate::net_folder::{IShellBrowser, IShellBrowserVtbl};
use crate::shellcom::{IFileSystemBindData, IFileSystemBindDataVtbl,
                      IInitializeCommand, IInitializeCommandVtbl,
                      IObjectWithSelection, IObjectWithSelectionVtbl};
use crate::shell32::utils_support::LOWORD;
use crate::net_folder::CppComPtrGuard;
use crate::fs_drop_target::{IShellLinkW as IShellLink, IID_IShellLinkW as IID_IShellLink};
use crate::default_context_menu_support::DCIA_GetEntry;
use crate::default_context_menu_support::DSA_AppendItem;
use crate::shell32::stubs::ExtractIconResInfoW;
use crate::default_context_menu_support::HRESULT_CODE;
use crate::wine::pidl::ILFindChild;
use crate::default_context_menu_support::IS_INTRESOURCE;
use crate::shellcom::LPBC;
use crate::default_context_menu_tu::MapVerbToDfmCmd;
use crate::win32::PFILETIME;
use crate::shell32::stubs::PathIsSlowW;
use crate::shell_item_array::SHCreateShellItemArrayFromDataObject;
use crate::propsheet::SHELL_GetShellExtensionRegCLSID;
use crate::wine::pidl::SHGetPathCchFromIDListW;
use crate::wine::shellpath::SHGetSpecialFolderLocation;
use crate::shell32::stubs::SHSetFolderPathW;
use crate::shell32::stubs::SHStartNetConnectionDialogW;
use crate::default_context_menu_support::StringCchCopyA;
use crate::win32::max;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_05395ACBD6C2: [WCHAR; 54] = [0x0053, 0x006F, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005C, 0x004D, 0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0057, 0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x005C, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0056, 0x0065, 0x0072, 0x0073, 0x0069, 0x006F, 0x006E, 0x005C, 0x0055, 0x006E, 0x0072, 0x0065, 0x0061, 0x0064, 0x004D, 0x0061, 0x0069, 0x006C, 0x005C, 0x0000];
pub static CPP_COM_WSTR_0C7319B93B0B: [WCHAR; 12] = [0x0041, 0x0070, 0x0070, 0x006C, 0x0069, 0x0063, 0x0061, 0x0074, 0x0069, 0x006F, 0x006E, 0x0000];
pub static CPP_COM_WSTR_2A3E9AB31456: [WCHAR; 53] = [0x0053, 0x006F, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005C, 0x004D, 0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0057, 0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x005C, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0056, 0x0065, 0x0072, 0x0073, 0x0069, 0x006F, 0x006E, 0x005C, 0x0055, 0x006E, 0x0072, 0x0065, 0x0061, 0x0064, 0x004D, 0x0061, 0x0069, 0x006C, 0x0000];
pub static CPP_COM_WSTR_2BF0903157CD: [WCHAR; 8] = [0x005C, 0x0053, 0x0068, 0x0065, 0x006C, 0x006C, 0x005C, 0x0000];
pub static CPP_COM_WSTR_3C3501C6F1F3: [WCHAR; 16] = [0x0053, 0x0068, 0x006F, 0x0077, 0x0057, 0x0069, 0x007A, 0x0061, 0x0072, 0x0064, 0x0073, 0x0054, 0x0045, 0x0053, 0x0054, 0x0000];
pub static CPP_COM_WSTR_3DB566BC3DA4: [WCHAR; 4] = [0x0043, 0x004F, 0x004D, 0x0000];
pub static CPP_COM_WSTR_4E317A7FF63D: [WCHAR; 16] = [0x005C, 0x0053, 0x0068, 0x0065, 0x006C, 0x006C, 0x0053, 0x0074, 0x0079, 0x006C, 0x0065, 0x002E, 0x0064, 0x006C, 0x006C, 0x0000];
pub static CPP_COM_WSTR_5BD07A46AC2D: [WCHAR; 4] = [0x0043, 0x004F, 0x004E, 0x0000];
pub static CPP_COM_WSTR_5F451136015F: [WCHAR; 4] = [0x0041, 0x0055, 0x0058, 0x0000];
pub static CPP_COM_WSTR_648E5B4273EE: [WCHAR; 4] = [0x0050, 0x0052, 0x004E, 0x0000];
pub static CPP_COM_WSTR_64A3D5F8DF2B: [WCHAR; 18] = [0x0055, 0x0049, 0x0020, 0x0044, 0x0075, 0x0072, 0x0069, 0x006E, 0x0067, 0x0020, 0x0042, 0x0069, 0x006E, 0x0064, 0x0069, 0x006E, 0x0067, 0x0000];
pub static CPP_COM_WSTR_8758F49A4AA2: [WCHAR; 13] = [0x004D, 0x0065, 0x0073, 0x0073, 0x0061, 0x0067, 0x0065, 0x0043, 0x006F, 0x0075, 0x006E, 0x0074, 0x0000];
pub static CPP_COM_WSTR_8888EF3C3D8B: [WCHAR; 60] = [0x0053, 0x006F, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005C, 0x004D, 0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0057, 0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x005C, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0056, 0x0065, 0x0072, 0x0073, 0x0069, 0x006F, 0x006E, 0x005C, 0x0045, 0x0078, 0x0070, 0x006C, 0x006F, 0x0072, 0x0065, 0x0072, 0x005C, 0x0041, 0x0064, 0x0076, 0x0061, 0x006E, 0x0063, 0x0065, 0x0064, 0x0000];
pub static CPP_COM_WSTR_9105F5027482: [WCHAR; 37] = [0x0025, 0x0053, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x0052, 0x006F, 0x006F, 0x0074, 0x0025, 0x005C, 0x0053, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x0033, 0x0032, 0x005C, 0x0053, 0x0068, 0x0065, 0x006C, 0x006C, 0x0053, 0x0074, 0x0079, 0x006C, 0x0065, 0x002E, 0x0064, 0x006C, 0x006C, 0x0000];
pub static CPP_COM_WSTR_A8E70516B833: [WCHAR; 4] = [0x004E, 0x0055, 0x004C, 0x0000];
pub static CPP_COM_WSTR_C1DBB4469138: [WCHAR; 8] = [0x0025, 0x0032, 0x0020, 0x0028, 0x0025, 0x0031, 0x0029, 0x0000];
pub static CPP_COM_WSTR_C372F2CCF22D: [WCHAR; 4] = [0x004C, 0x0050, 0x0054, 0x0000];
pub static CPP_COM_WSTR_C6E922ADD3B3: [WCHAR; 8] = [0x0073, 0x0068, 0x0065, 0x006C, 0x006C, 0x0065, 0x0078, 0x0000];
pub static CPP_COM_WSTR_CC16ED28C79D: [WCHAR; 10] = [0x0054, 0x0069, 0x006D, 0x0065, 0x0053, 0x0074, 0x0061, 0x006D, 0x0070, 0x0000];
pub static CPP_COM_WSTR_FB73E589D125: [WCHAR; 8] = [0x0025, 0x0031, 0x0020, 0x0028, 0x0025, 0x0032, 0x0029, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const PATH_VALID_ELEMENT: DWORD = PATH_CHAR_CLASS_DOT | PATH_CHAR_CLASS_SEMICOLON | PATH_CHAR_CLASS_COMMA | PATH_CHAR_CLASS_SPACE | PATH_CHAR_CLASS_OTHER_VALID;

/// `#define COMPUTER_DESCRIPTIONS_KEY L"Software\\...\\ComputerDescriptions"` (utils.cpp).
/// The preamble rustconv emitted referenced a `CPP_COM_WSTR_COMPUTER_DESCRIPTIONS_KEY` that no
/// wide-literal lowering ever produced, so the array is written out here. The length is
/// COMPUTED, not typed: a hand-counted `[u16; N]` losing its terminator is the exact bug the
/// crate's wide-literal rule exists to stop.
static COMPUTER_DESCRIPTIONS_KEY_W: [u16; 72] = [
    0x0053, 0x006F, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005C, 0x004D, 0x0069,
    0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0057, 0x0069, 0x006E,
    0x0064, 0x006F, 0x0077, 0x0073, 0x005C, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E,
    0x0074, 0x0056, 0x0065, 0x0072, 0x0073, 0x0069, 0x006F, 0x006E, 0x005C, 0x0045, 0x0078,
    0x0070, 0x006C, 0x006F, 0x0072, 0x0065, 0x0072, 0x005C, 0x0043, 0x006F, 0x006D, 0x0070,
    0x0075, 0x0074, 0x0065, 0x0072, 0x0044, 0x0065, 0x0073, 0x0063, 0x0072, 0x0069, 0x0070,
    0x0074, 0x0069, 0x006F, 0x006E, 0x0073, 0x0000,
];

#[inline]
pub fn COMPUTER_DESCRIPTIONS_KEY() -> PCWSTR {
    COMPUTER_DESCRIPTIONS_KEY_W.as_ptr()
}

#[repr(C)]
pub struct tagALIAS_MAPPING {
    pub bFlagMask: u8,
    pub bCommonDesktop: u8,
    pub nCsidlSrc: u16,
    pub nCsidlDest: u16,
}
pub type ALIAS_MAPPING = tagALIAS_MAPPING;
pub type PALIAS_MAPPING = *mut tagALIAS_MAPPING;

pub static g_AliasTable: [ALIAS_MAPPING; 4] = [
    ALIAS_MAPPING {
        bFlagMask: ALIAS_USER_FOLDER as u8,
        bCommonDesktop: 0u8,
        nCsidlSrc: (CSIDL_PERSONAL | CSIDL_FLAG_NO_ALIAS) as u16,
        nCsidlDest: CSIDL_PERSONAL as u16,
    },
    ALIAS_MAPPING {
        bFlagMask: (ALIAS_USER_FOLDER | ALIAS_DESKTOP) as u8,
        bCommonDesktop: 0u8,
        nCsidlSrc: (CSIDL_COMMON_DOCUMENTS | CSIDL_FLAG_NO_ALIAS) as u16,
        nCsidlDest: CSIDL_COMMON_DOCUMENTS as u16,
    },
    ALIAS_MAPPING {
        bFlagMask: ALIAS_DESKTOP as u8,
        bCommonDesktop: 0u8,
        nCsidlSrc: CSIDL_DESKTOPDIRECTORY as u16,
        nCsidlDest: CSIDL_DESKTOP as u16,
    },
    ALIAS_MAPPING {
        bFlagMask: ALIAS_DESKTOP as u8,
        bCommonDesktop: 1u8,
        nCsidlSrc: CSIDL_COMMON_DESKTOPDIRECTORY as u16,
        nCsidlDest: CSIDL_DESKTOP as u16,
    },
];

#[repr(C)]
pub struct CDummyOleWindowVtbl {
    pub QueryInterface: unsafe extern "system" fn(*mut CDummyOleWindow, REFIID, *mut LPVOID) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(*mut CDummyOleWindow) -> ULONG,
    pub Release: unsafe extern "system" fn(*mut CDummyOleWindow) -> ULONG,
    pub GetWindow: unsafe extern "system" fn(*mut CDummyOleWindow, *mut HWND) -> HRESULT,
    pub ContextSensitiveHelp: unsafe extern "system" fn(*mut CDummyOleWindow, BOOL) -> HRESULT,
}

#[repr(C)]
pub struct CDummyOleWindow {
    pub lpVtbl: *mut CDummyOleWindowVtbl,
    pub m_cRefs: LONG,
    pub m_hWnd: HWND,
}

// utils.cpp:548 `class CDummyOleWindow : public IOleWindow`. The declaration above came out of
// the paid run; the object itself did not, because `new CDummyOleWindow()` has no Rust
// spelling in a no_std crate. Same shape the rest of shell32-rust uses for a COM object:
// HeapAlloc + a static vtable, refcount starting at 1, `delete this` becoming HeapFree.

/// `IID_IOleWindow` — oleidl.idl:65. Not in shell32's island (olectlid.h only `extern`s it,
/// and shell32 links no ole32 import lib for the data symbol), so it is spelled out.
/// `IID_IUnknown` — unknwn.idl. Same reason as IID_IOleWindow_LOCAL below.
static IID_IUnknown_LOCAL: GUID = GUID {
    Data1: 0x00000000,
    Data2: 0x0000,
    Data3: 0x0000,
    Data4: [0xc0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46],
};

/// `IsEqualIID` — guiddef.h compares the sixteen bytes, nothing more.
#[inline]
unsafe fn cpp_com_iid_equal_utils(left: REFIID, right: *const GUID) -> bool {
    if left.is_null() || right.is_null() {
        return false;
    }
    (*left).Data1 == (*right).Data1
        && (*left).Data2 == (*right).Data2
        && (*left).Data3 == (*right).Data3
        && (*left).Data4 == (*right).Data4
}

static IID_IOleWindow_LOCAL: GUID = GUID {
    Data1: 0x00000114,
    Data2: 0x0000,
    Data3: 0x0000,
    Data4: [0xc0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46],
};

unsafe extern "system" fn CDummyOleWindow_QueryInterface(
    this: *mut CDummyOleWindow,
    riid: REFIID,
    ppvObj: *mut LPVOID,
) -> HRESULT {
    if ppvObj.is_null() {
        return E_POINTER;
    }
    // The C++ is `QISearch` over a one-entry QITAB for IOleWindow; QISearch also answers
    // IUnknown from the first entry's interface pointer, which for a single-base class is
    // the object itself.
    if cpp_com_iid_equal_utils(riid, core::ptr::addr_of!(IID_IOleWindow_LOCAL))
        || cpp_com_iid_equal_utils(riid, core::ptr::addr_of!(IID_IUnknown_LOCAL))
    {
        *ppvObj = this.cast();
        (*this).m_cRefs += 1;
        return S_OK;
    }
    *ppvObj = core::ptr::null_mut();
    E_NOINTERFACE
}

unsafe extern "system" fn CDummyOleWindow_AddRef(this: *mut CDummyOleWindow) -> ULONG {
    (*this).m_cRefs += 1;
    (*this).m_cRefs as ULONG
}

unsafe extern "system" fn CDummyOleWindow_Release(this: *mut CDummyOleWindow) -> ULONG {
    (*this).m_cRefs -= 1;
    if (*this).m_cRefs == 0 {
        HeapFree(GetProcessHeap(), 0, this.cast::<core::ffi::c_void>());
        return 0;
    }
    (*this).m_cRefs as ULONG
}

unsafe extern "system" fn CDummyOleWindow_GetWindow(
    this: *mut CDummyOleWindow,
    phWnd: *mut HWND,
) -> HRESULT {
    *phWnd = (*this).m_hWnd;
    if (*this).m_hWnd.is_null() {
        return E_NOTIMPL;
    }
    S_OK
}

unsafe extern "system" fn CDummyOleWindow_ContextSensitiveHelp(
    _this: *mut CDummyOleWindow,
    _fEnterMode: BOOL,
) -> HRESULT {
    E_NOTIMPL
}

static CDUMMYOLEWINDOW_VTBL: CDummyOleWindowVtbl = CDummyOleWindowVtbl {
    QueryInterface: CDummyOleWindow_QueryInterface,
    AddRef: CDummyOleWindow_AddRef,
    Release: CDummyOleWindow_Release,
    GetWindow: CDummyOleWindow_GetWindow,
    ContextSensitiveHelp: CDummyOleWindow_ContextSensitiveHelp,
};

/// `new CDummyOleWindow()` — utils.cpp:555, `m_cRefs(1), m_hWnd(NULL)`. Returns NULL on an
/// allocation failure, which is the one case the C++ cannot express and the caller already
/// null-checks.
unsafe fn CDummyOleWindow_new() -> *mut CDummyOleWindow {
    let this = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        core::mem::size_of::<CDummyOleWindow>() as SIZE_T,
    ) as *mut CDummyOleWindow;
    if this.is_null() {
        return core::ptr::null_mut();
    }
    core::ptr::write(
        this,
        CDummyOleWindow {
            lpVtbl: core::ptr::addr_of!(CDUMMYOLEWINDOW_VTBL) as *mut CDummyOleWindowVtbl,
            m_cRefs: 1,
            m_hWnd: core::ptr::null_mut(),
        },
    );
    this
}
unsafe extern "C" fn StrEndNA(psz: *const core::ffi::c_char, cch: isize) -> *const core::ffi::c_char {
    let mut pch = psz;
    let pchEnd = psz.offset(cch);
    while *pch != 0 && pch < pchEnd {
        pch = CharNextA(pch);
    }
    if pchEnd < pch {
        pch = pch.offset(-(2 as isize));
    }
    pch
}
unsafe extern "C" fn StrEndNW(psz: PCWSTR, cch: isize) -> PCWSTR {
    unsafe {
        let pchEnd = psz.offset(cch);
        let mut pch = psz;
        while *pch != 0 && (pch as usize) < (pchEnd as usize) {
            pch = pch.offset(1);
        }
        pch
    }
}

#[no_mangle]
pub unsafe extern "system" fn StrRStrA(
    pszSrc: *const core::ffi::c_char,
    pszLast: *const core::ffi::c_char,
    pszSearch: *const core::ffi::c_char,
) -> *mut core::ffi::c_char {
    let cchSearch = lstrlenA(pszSearch);

    let mut pchEnd: *const core::ffi::c_char = if !pszLast.is_null() {
        pszLast
    } else {
        pszSrc.offset(lstrlenA(pszSrc) as isize)
    };
    if pchEnd == pszSrc {
        return core::ptr::null_mut();
    }

    let mut cchEnd: isize = (pchEnd as usize).wrapping_sub(pszSrc as usize) as isize;

    loop {
        pchEnd = pchEnd.offset(-1);
        cchEnd -= 1;
        if pchEnd.is_null() {
            break;
        }
        if StrCmpNA(pchEnd, pszSearch, cchSearch) == 0
            && pchEnd == StrEndNA(pszSrc, cchEnd)
        {
            break;
        }
        if pchEnd == pszSrc {
            return core::ptr::null_mut();
        }
    }

    pchEnd as *mut core::ffi::c_char
}

#[no_mangle]
pub unsafe extern "system" fn StrRStrW(
    pszSrc: PCWSTR,
    pszLast: PCWSTR,
    pszSearch: PCWSTR,
) -> PWSTR {
    let cchSearch: i32 = lstrlenW(pszSearch);

    let mut pchEnd: PCWSTR = if !pszLast.is_null() {
        pszLast
    } else {
        pszSrc.add(lstrlenW(pszSrc) as usize)
    };
    if pchEnd == pszSrc {
        return core::ptr::null_mut();
    }

    let mut cchEnd: isize = pchEnd.offset_from(pszSrc);
    loop {
        pchEnd = pchEnd.sub(1);
        cchEnd -= 1;
        if pchEnd.is_null() {
            break;
        }
        if StrCmpNW(pchEnd, pszSearch, cchSearch) == 0
            && pchEnd == StrEndNW(pszSrc, cchEnd)
        {
            break;
        }
        if pchEnd == pszSrc {
            return core::ptr::null_mut();
        }
    }

    pchEnd as PWSTR
}
#[no_mangle]
pub unsafe extern "C" fn SHILClone(pidl: LPCITEMIDLIST, ppidl: *mut LPITEMIDLIST) -> HRESULT {
    if pidl.is_null() {
        *ppidl = core::ptr::null_mut();
        return S_OK;
    }
    *ppidl = ILClone(pidl);
    if !(*ppidl).is_null() {
        S_OK
    } else {
        E_OUTOFMEMORY
    }
}
#[no_mangle]
pub unsafe extern "C" fn PathIsDotOrDotDotW(pszPath: PCWSTR) -> i32 {
    if *pszPath.add(0) != 0x002Eu16 {
        return FALSE;
    }
    (*pszPath.add(1) == 0
        || (*pszPath.add(1) == 0x002Eu16
            && *pszPath.add(2) == 0)) as i32
}
#[no_mangle]
pub unsafe extern "C" fn PathIsValidElement(pszPath: PCWSTR) -> i32 {
    if (*pszPath) == 0 || PathIsDotOrDotDotW(pszPath) != 0 {
        return FALSE;
    }

    let mut pch: PCWSTR = pszPath;
    while (*pch) != 0 {
        if PathIsValidCharW(*pch, PATH_VALID_ELEMENT) == 0 {
            return FALSE;
        }
        pch = pch.add(1);
    }

    TRUE
}
#[no_mangle]
pub unsafe extern "C" fn PathIsDosDevice(pszName: PCWSTR) -> BOOL {
    let mut szPath = [0u16; ((MAX_PATH) as usize)];

    StringCchCopyW(szPath.as_mut_ptr(), szPath.len(), pszName);
    PathRemoveExtensionW(szPath.as_mut_ptr());

    if lstrcmpiW(szPath.as_ptr(), CPP_COM_WSTR_A8E70516B833.as_ptr()) == 0
        || lstrcmpiW(szPath.as_ptr(), CPP_COM_WSTR_648E5B4273EE.as_ptr()) == 0
        || lstrcmpiW(szPath.as_ptr(), CPP_COM_WSTR_5BD07A46AC2D.as_ptr()) == 0
        || lstrcmpiW(szPath.as_ptr(), CPP_COM_WSTR_5F451136015F.as_ptr()) == 0
    {
        return TRUE;
    }

    if _wcsnicmp(szPath.as_ptr(), CPP_COM_WSTR_C372F2CCF22D.as_ptr(), 3) == 0
        || _wcsnicmp(szPath.as_ptr(), CPP_COM_WSTR_3DB566BC3DA4.as_ptr(), 3) == 0
    {
        if (0x0030u16 <= szPath[3] && szPath[3] <= 0x0039u16)
            && szPath[4] == UNICODE_NULL
        {
            return TRUE;
        }
    }

    return FALSE;
}
#[no_mangle]
pub unsafe extern "C" fn SHILAppend(pidl: LPITEMIDLIST, ppidl: *mut LPITEMIDLIST) -> HRESULT {
    let pidlOld = *ppidl;
    if pidlOld.is_null() {
        *ppidl = pidl;
        return S_OK;
    }

    let hr = SHILCombine(*ppidl, pidl, ppidl);
    ILFree(pidlOld);
    ILFree(pidl);
    hr
}

#[no_mangle]
pub unsafe extern "system" fn SHShouldShowWizards(
    pUnknown: *mut IUnknown,
) -> HRESULT {
    let mut pBrowser: *mut IShellBrowser = core::ptr::null_mut();
    let mut hr: HRESULT = IUnknown_QueryService(
        pUnknown.cast(),
        &SID_STopWindow,
        &IID_IShellBrowser,
        core::ptr::addr_of_mut!(pBrowser).cast(),
    );
    if hr < 0 {
        return hr;
    }

    let mut state: SHELLSTATE = core::mem::zeroed();
    SHGetSetSettings(
        &mut state,
        SSF_WEBVIEW,
        FALSE,
    );
    if state.fWebView() != 0
        && SHRegGetBoolUSValueW(
            CPP_COM_WSTR_8888EF3C3D8B.as_ptr(),
            CPP_COM_WSTR_3C3501C6F1F3.as_ptr(),
            FALSE,
            FALSE,
        ) == 0
    {
        hr = S_FALSE;
    }

    ((*(*pBrowser).lpVtbl).Release)(pBrowser);
    return hr;
}
unsafe extern "C" fn OpenEffectiveToken(DesiredAccess: DWORD, phToken: *mut HANDLE) -> BOOL {
    if phToken == core::ptr::null_mut() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }

    *phToken = core::ptr::null_mut();

    let mut ret = OpenThreadToken(
        GetCurrentThread(),
        DesiredAccess,
        FALSE,
        phToken,
    );
    if ret == 0 && GetLastError() == ERROR_NO_TOKEN {
        ret = OpenProcessToken(
            GetCurrentProcess(),
            DesiredAccess,
            phToken,
        );
    }

    ret
}
#[no_mangle]
pub unsafe extern "C" fn BindCtx_ContainsObject(pBindCtx: *mut IBindCtx, pszName: LPCWSTR) -> i32 {
    let mut punk: *mut IUnknown = core::ptr::null_mut();
    let _punk_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(punk));
    let hr = if pBindCtx.is_null() {
        -1
    } else {
        ((*(*pBindCtx).lpVtbl).GetObjectParam)(pBindCtx, pszName as LPOLESTR, &mut punk)
    };
    let result = if pBindCtx.is_null() || hr < 0 {
        FALSE
    } else {
        TRUE
    };
    result
}
#[no_mangle]
pub unsafe extern "C" fn BindCtx_GetMode(pbc: *mut IBindCtx, dwDefault: u32) -> u32 {
    if pbc.is_null() {
        return dwDefault;
    }

    let mut BindOpts = core::mem::zeroed::<BIND_OPTS>();
    BindOpts.cbStruct = core::mem::size_of::<BIND_OPTS>() as u32;
    let hr = ((*(*pbc).lpVtbl).GetBindOptions)(pbc, core::ptr::addr_of_mut!(BindOpts).cast());
    if hr < 0 {
        return dwDefault;
    }

    BindOpts.grfMode
}
#[no_mangle]
pub unsafe extern "C" fn SHSkipJunctionBinding(pbc: *mut IBindCtx, pclsid: *mut CLSID) -> i32 {
    if pbc.is_null() {
        return FALSE;
    }

    let mut BindOps: BIND_OPTS = core::mem::zeroed();
    BindOps.cbStruct = core::mem::size_of::<BIND_OPTS>() as u32;

    if (((*(*pbc).lpVtbl).GetBindOptions)(pbc, ((core::ptr::addr_of_mut!(BindOps)) as _)) >= 0) &&
        BindOps.grfFlags == OLECONTF_LINKS
    {
        return TRUE;
    }

    if pclsid.is_null() {
        return 0;
    }
    SHSkipJunction(pbc.cast(), pclsid)
}
#[no_mangle]
pub unsafe extern "C" fn SHIsFileSysBindCtx(
    pBindCtx: *mut IBindCtx,
    pFindData: *mut WIN32_FIND_DATAW,
) -> HRESULT {
    let mut punk: *mut IUnknown = core::ptr::null_mut();
    let _punk_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(punk));
    let mut pBindData: *mut IFileSystemBindData = core::ptr::null_mut();
    let _pBindData_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pBindData));

    if pBindCtx.is_null()
        || ((*(*pBindCtx).lpVtbl).GetObjectParam)(pBindCtx,
            STR_FILE_SYS_BIND_DATA_W.as_ptr() as PWSTR,
            core::ptr::addr_of_mut!(punk),
        ) < 0
    {
        return S_FALSE;
    }

    if ((*(*punk).lpVtbl).QueryInterface)(punk, core::ptr::addr_of!(IID_IFileSystemBindData), core::ptr::addr_of_mut!(pBindData).cast())
        < 0
    {
        return S_FALSE;
    }

    if !pFindData.is_null() {
        ((*(*pBindData).lpVtbl).GetFindData)(pBindData, ((pFindData) as _));
    }

    return S_OK;
}
#[no_mangle]
pub unsafe extern "C" fn Shell_FailForceReturn(hr: HRESULT) -> BOOL {
    let code: DWORD = HRESULT_CODE(hr) as DWORD;

    match code {
        ERROR_BAD_NETPATH |
        ERROR_BAD_NET_NAME |
        ERROR_CANCELLED => return TRUE as BOOL,
        _ => return (ERROR_FILE_NOT_FOUND <= code && code <= ERROR_PATH_NOT_FOUND) as BOOL,
    }
}
#[no_mangle]
pub unsafe extern "C" fn SHBindToObjectEx(
    mut pShellFolder: *mut IShellFolder,
    pidl: LPCITEMIDLIST,
    pBindCtx: *mut IBindCtx,
    riid: REFIID,
    ppvObj: *mut *mut core::ffi::c_void,
) -> i32 {
    unsafe {
        let mut psfDesktop: *mut IShellFolder = core::ptr::null_mut();
        let _psfDesktop_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(psfDesktop));

        *ppvObj = core::ptr::null_mut();

        if pShellFolder.is_null() {
            SHGetDesktopFolder(core::ptr::addr_of_mut!(psfDesktop));
            if psfDesktop.is_null() {
                return E_FAIL;
            }
            pShellFolder = psfDesktop;
        }

        let mut hr: i32;
        if _ILIsDesktop(pidl) != 0 {
            hr = ((*(*pShellFolder).lpVtbl).QueryInterface)(pShellFolder, ((riid) as _), ppvObj);
        } else {
            hr = ((*(*pShellFolder).lpVtbl).BindToObject)(pShellFolder, pidl, pBindCtx, riid.cast(), ppvObj);
        }

        if hr >= 0 && (*ppvObj).is_null() {
            hr = E_FAIL;
        }


        hr
    }
}

#[no_mangle]
pub unsafe extern "C" fn SHBindToObject(
    psf: *mut IShellFolder,
    pidl: LPCITEMIDLIST,
    riid: REFIID,
    ppvObj: *mut *mut core::ffi::c_void,
) -> HRESULT {
    SHBindToObjectEx(psf, pidl, core::ptr::null_mut(), riid, ppvObj)
}

#[no_mangle]
pub unsafe extern "C" fn SHELL_GetUIObjectOfAbsoluteItem(
    hWnd: HWND,
    pidl: PCIDLIST_ABSOLUTE,
    riid: REFIID,
    ppvObj: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppvObj.is_null() {
        return E_INVALIDARG;
    }
    *ppvObj = core::ptr::null_mut();

    let mut psf: *mut IShellFolder = core::ptr::null_mut();
    let mut pidlChild: PCUITEMID_CHILD = core::ptr::null_mut();

    let mut hr = SHBindToParent(
        pidl,
        core::ptr::addr_of!(IID_IShellFolder) as REFIID,
        core::ptr::addr_of_mut!(psf) as *mut *mut core::ffi::c_void,
        core::ptr::addr_of_mut!(pidlChild) as *mut PCUITEMID_CHILD,
    );

    if hr >= 0 {
        hr = ((*(*psf).lpVtbl).GetUIObjectOf)(psf,
            hWnd.cast(),
            1,
            &pidlChild,
            riid.cast(),
            core::ptr::null_mut(),
            ppvObj,
        );
        ((*(*psf).lpVtbl).Release)(psf);

        if hr >= 0 {
            if !(*ppvObj).is_null() {
                return hr;
            }
            hr = E_FAIL;
        }
    }
    hr
}
#[no_mangle]
pub unsafe extern "C" fn SHELL_DisplayNameOf(
    psf: *mut IShellFolder,
    pidl: LPCITEMIDLIST,
    Flags: u32,
    ppStr: *mut PWSTR,
) -> HRESULT {
    let mut psf = psf;
    let mut pidl = pidl;
    let mut psfRoot: *mut IShellFolder = core::ptr::null_mut();
    let _psfRoot_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(psfRoot));
    let mut hr: HRESULT = 0;
    if psf.is_null() {
        let mut pidlChild: PCUITEMID_CHILD = core::ptr::null_mut();
        hr = SHBindToParent(pidl, core::ptr::addr_of!(IID_IShellFolder).cast(), core::ptr::addr_of_mut!(psfRoot).cast(), core::ptr::addr_of_mut!(pidlChild));
        if hr < 0 {
            return hr;
        }
        psf = psfRoot;
        pidl = pidlChild;
    }
    let mut sr: STRRET = core::mem::zeroed();
    hr = ((*(*psf).lpVtbl).GetDisplayNameOf)(psf, pidl as PCUITEMID_CHILD, Flags, core::ptr::addr_of_mut!(sr).cast());
    let result = if hr >= 0 {
        StrRetToStrW(core::ptr::addr_of_mut!(sr), pidl, ppStr)
    } else {
        hr
    };
    result
}

#[no_mangle]
pub unsafe extern "system" fn DisplayNameOfW(
    psf: *mut IShellFolder,
    pidl: LPCITEMIDLIST,
    dwFlags: DWORD,
    pszBuf: LPWSTR,
    cchBuf: UINT,
) -> HRESULT {
    *pszBuf = UNICODE_NULL;
    let mut sr: STRRET = core::mem::zeroed();
    let hr = ((*(*psf).lpVtbl).GetDisplayNameOf)(psf, pidl, dwFlags, core::ptr::addr_of_mut!(sr).cast());
    if hr < 0 {
        return hr;
    }
    return StrRetToBufW(&mut sr, pidl, pszBuf, cchBuf);
}
#[no_mangle]
pub unsafe extern "C" fn SHGetAttributes(
    mut psf: *mut IShellFolder,
    pidl: LPCITEMIDLIST,
    mut dwAttributes: u32
) -> u32 {
    let mut pidlLast: LPCITEMIDLIST = pidl;
    let mut release: *mut IShellFolder = core::ptr::null_mut();

    if psf.is_null() {
        SHBindToParent(
            pidl,
            core::ptr::addr_of!(IID_IShellFolder) as REFIID,
            core::ptr::addr_of_mut!(psf) as *mut *mut core::ffi::c_void,
            core::ptr::addr_of_mut!(pidlLast)
        );
        if psf.is_null() {
            return 0;
        }
        release = psf;
    }

    let oldAttrs = dwAttributes;
    if (((*(*psf).lpVtbl).GetAttributesOf)(
        psf,
        1,
        ((core::ptr::addr_of_mut!(pidlLast)) as _),
        core::ptr::addr_of_mut!(dwAttributes)
    ) < 0) {
        dwAttributes = 0;
    } else {
        dwAttributes &= oldAttrs;
    }

    if (dwAttributes & SFGAO_FOLDER) != 0 &&
       (dwAttributes & SFGAO_STREAM) != 0 &&
       (dwAttributes & SFGAO_STORAGEANCESTOR) == 0 &&
       (oldAttrs & SFGAO_STORAGEANCESTOR) != 0 &&
       (SHGetObjectCompatFlags(psf.cast(), core::ptr::null()) & 0x200) != 0
    {
        dwAttributes &= !(SFGAO_STREAM | SFGAO_STORAGEANCESTOR);
        dwAttributes |= SFGAO_STORAGEANCESTOR;
    }

    if !release.is_null() {
        ((*(*release).lpVtbl).Release)(release);
    }

    dwAttributes
}
#[no_mangle]
pub unsafe extern "C" fn SHELL_GetIDListTarget(
    pidl: LPCITEMIDLIST,
    ppidl: *mut PIDLIST_ABSOLUTE,
) -> HRESULT {
    let mut pSL: *mut IShellLink = core::ptr::null_mut();
    let mut hr: HRESULT = unsafe {
        SHBindToObject(
            core::ptr::null_mut(),
            pidl,
            ((core::ptr::addr_of!(IID_IShellLink)) as _), core::ptr::addr_of_mut!(pSL).cast(),
        )
    };
    if hr >= 0 {
        unsafe {
            hr = ((*(*pSL).lpVtbl).GetIDList)(pSL, ppidl);
            ((*(*pSL).lpVtbl).Release)(pSL);
        }
    }
    hr
}
#[no_mangle]
pub unsafe extern "C" fn SHCoInitializeAnyApartment() -> i32 {
    let mut hr: i32 = CoInitializeEx(
        core::ptr::null_mut(),
        (COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE) as u32,
    );
    if hr < 0 {
        hr = CoInitializeEx(core::ptr::null_mut(), COINIT_DISABLE_OLE1DDE as u32);
    }
    hr
}
#[no_mangle]
pub unsafe extern "C" fn SHGetNameAndFlagsW(
    pidl: LPCITEMIDLIST,
    dwFlags: u32,
    pszText: LPWSTR,
    cchBuf: u32,
    pdwAttributes: *mut u32,
) -> HRESULT {
    if !pszText.is_null() {
        *pszText = UNICODE_NULL;
    }

    let hrCoInit = SHCoInitializeAnyApartment();

    let mut psfFolder: *mut IShellFolder = core::ptr::null_mut();
    let _psfFolder_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(psfFolder));
    let mut ppidlLast: LPCITEMIDLIST = core::ptr::null();
    let mut hr = SHBindToParent(
        pidl,
        core::ptr::addr_of!(IID_IShellFolder).cast(),
        core::ptr::addr_of_mut!(psfFolder) as *mut *mut core::ffi::c_void,
        core::ptr::addr_of_mut!(ppidlLast),
    );
    if hr >= 0 {
        if !pszText.is_null() {
            hr = DisplayNameOfW(psfFolder, ppidlLast, dwFlags, pszText, cchBuf);
        }

        if hr >= 0 {
            if !pdwAttributes.is_null() {
                *pdwAttributes = SHGetAttributes(psfFolder, ppidlLast, *pdwAttributes);
            }
        }
    }

    if hrCoInit >= 0 {
        CoUninitialize();
    }

    if !psfFolder.is_null() {
        ((*(*psfFolder).lpVtbl).Release)(psfFolder);
        psfFolder = core::ptr::null_mut();
    }

    hr
}

#[no_mangle]
pub unsafe extern "system" fn CShellLink_SHGetNameAndFlagsW_RustBridge(
    pidl: LPCITEMIDLIST,
    dwFlags: DWORD,
    pszText: LPWSTR,
    cchBuf: UINT,
    pdwAttributes: *mut DWORD,
) -> HRESULT {
    SHGetNameAndFlagsW(pidl, dwFlags, pszText, cchBuf, pdwAttributes)
}

#[no_mangle]
pub unsafe extern "C" fn BindCtx_GetUIWindow(pBindCtx: *mut IBindCtx) -> HWND {
    let mut hWnd: HWND = core::ptr::null_mut();
    let mut punk: *mut IUnknown = core::ptr::null_mut();
    let _punk_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(punk));
    if !pBindCtx.is_null() && (((*(*pBindCtx).lpVtbl).GetObjectParam)(pBindCtx, CPP_COM_WSTR_64A3D5F8DF2B.as_ptr() as LPOLESTR, core::ptr::addr_of_mut!(punk)) >= 0) {
        IUnknown_GetWindow(punk.cast(), core::ptr::addr_of_mut!(hWnd));
    }
    hWnd
}

#[no_mangle]
pub unsafe extern "C" fn BindCtx_RegisterObjectParam(
    pBindCtx: LPBC,
    pszKey: LPOLESTR,
    mut punk: *mut IUnknown,
    ppbc: *mut LPBC,
) -> HRESULT {
    let mut hr: HRESULT = S_OK;
    let mut p_unknown: *mut CDummyOleWindow = core::ptr::null_mut();

    *ppbc = pBindCtx;

    if !pBindCtx.is_null() {
        ((*(*pBindCtx).lpVtbl).AddRef)(pBindCtx);
    } else {
        hr = CreateBindCtx(0, ppbc.cast());
        if hr < 0 {
            return hr;
        }
    }

    if punk.is_null() {
        p_unknown = CDummyOleWindow_new();
        punk = p_unknown as *mut IUnknown;
    }

    hr = ((*(*(*ppbc)).lpVtbl).RegisterObjectParam)(*ppbc, pszKey, punk);

    if !p_unknown.is_null() {
        ((*(*p_unknown).lpVtbl).Release)(p_unknown);
    }

    if hr < 0 {
        ((*(*(*ppbc)).lpVtbl).Release)(*ppbc);
        *ppbc = core::ptr::null_mut();
    }

    hr
}

#[no_mangle]
pub unsafe extern "system" fn SHSetFolderPathA(
    csidl: INT,
    hToken: HANDLE,
    dwFlags: DWORD,
    pszPath: LPCSTR,
) -> HRESULT {
    let strPathW = CStringW::from_ansi(pszPath);
    SHSetFolderPathW(csidl, hToken, dwFlags, strPathW.as_ptr())
}

#[no_mangle]
pub unsafe extern "system" fn PathIsSlowA(
    pszFile: *const i8,
    dwAttr: u32,
) -> i32 {
    let strFileW = CStringW::from_ansi(pszFile);
    ((PathIsSlowW(strFileW.as_ptr(), dwAttr)) as i32)
}

#[no_mangle]
pub unsafe extern "system" fn ExtractIconResInfoA(
    hHandle: *mut core::ffi::c_void,
    lpFileName: *const i8,
    wIndex: u16,
    lpSize: *mut u16,
    lpIcon: *mut *mut core::ffi::c_void,
) -> u16 {
    if lpFileName.is_null() {
        return 0;
    }

    let strFileNameW = CStringW::from_ansi(lpFileName);
    ExtractIconResInfoW(hHandle, strFileNameW.as_ptr(), wIndex, lpSize, lpIcon)
}

#[no_mangle]
pub unsafe extern "system" fn ShortSizeFormatW(
    dwNumber: u32,
    pszBuffer: PWSTR,
) -> PWSTR {
    StrFormatByteSizeW(((dwNumber) as i64), pszBuffer, 0x8FFF)
}
#[no_mangle]
#[no_mangle] pub unsafe extern "system" fn SHOpenEffectiveToken(phToken: *mut HANDLE) -> BOOL {
    OpenEffectiveToken(TOKEN_QUERY | TOKEN_ADJUST_PRIVILEGES, phToken)
}

#[no_mangle]
pub unsafe extern "system" fn SHGetUserSessionId(mut hToken: HANDLE) -> DWORD {
    let mut dwSessionId: DWORD = 0;
    let mut dwLength: DWORD = 0;
    let mut bOpenToken: BOOL = FALSE;

    if hToken.is_null() {
        bOpenToken = SHOpenEffectiveToken(core::ptr::addr_of_mut!(hToken));
    }

    if hToken.is_null()
        || GetTokenInformation(
            hToken,
            TokenSessionId,
            ((core::ptr::addr_of_mut!(dwSessionId)) as _),
            core::mem::size_of::<DWORD>() as DWORD,
            core::ptr::addr_of_mut!(dwLength),
        ) == 0
    {
        dwSessionId = 0;
    }

    if bOpenToken != 0 {
        CloseHandle(hToken);
    }

    dwSessionId
}
#[no_mangle]
#[no_mangle] pub unsafe extern "system" fn SHInvokePrivilegedFunctionW(
    pszName: PCWSTR,
    fn_: PRIVILEGED_FUNCTION,
    lParam: LPARAM,
) -> HRESULT {
    unsafe {
        if pszName.is_null() || fn_.is_none() {
            return E_INVALIDARG;
        }

        let mut hToken: HANDLE = core::ptr::null_mut();
        let mut NewPriv = core::mem::zeroed::<TOKEN_PRIVILEGES>();
        let mut PrevPriv = core::mem::zeroed::<TOKEN_PRIVILEGES>();
        let mut bAdjusted: BOOL = FALSE;

        if SHOpenEffectiveToken(core::ptr::addr_of_mut!(hToken)) != 0
            && LookupPrivilegeValueW(
                core::ptr::null_mut(),
                pszName,
                core::ptr::addr_of_mut!(NewPriv.Privileges[0].Luid),
            ) != 0
        {
            NewPriv.PrivilegeCount = 1;
            NewPriv.Privileges[0].Attributes = SE_PRIVILEGE_ENABLED;

            let mut dwReturnSize: u32 = 0;
            bAdjusted = AdjustTokenPrivileges(
                hToken,
                FALSE,
                core::ptr::addr_of_mut!(NewPriv),
                core::mem::size_of::<TOKEN_PRIVILEGES>() as u32,
                core::ptr::addr_of_mut!(PrevPriv),
                core::ptr::addr_of_mut!(dwReturnSize),
            );
        }

        // `fn_` is a FARPROC-shaped Option; the null test above already returned, and a
        // `.unwrap()` here would put a panic on a live shell path.
        let hr: HRESULT = match fn_ { Some(f) => f(lParam), None => E_INVALIDARG };

        if bAdjusted != 0 {
            AdjustTokenPrivileges(
                hToken,
                FALSE,
                core::ptr::addr_of_mut!(PrevPriv),
                0,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );
        }

        if !hToken.is_null() {
            CloseHandle(hToken);
        }

        hr
    }
}

#[no_mangle]
pub unsafe extern "system" fn SHTestTokenPrivilegeW(
    mut hToken: HANDLE,
    lpName: LPCWSTR,
) -> BOOL {
    let mut Luid: LUID = core::mem::zeroed();
    let mut dwLength: DWORD;
    let mut pTokenPriv: PTOKEN_PRIVILEGES;
    let mut hNewToken: HANDLE = core::ptr::null_mut();
    let mut ret: BOOL = FALSE;

    'quit: {
        if lpName.is_null() {
            return FALSE;
        }

        if hToken.is_null() {
            if SHOpenEffectiveToken(core::ptr::addr_of_mut!(hNewToken)) == 0 {
                break 'quit;
            }

            if hNewToken.is_null() {
                return FALSE;
            }

            hToken = hNewToken;
        }

        if LookupPrivilegeValueW(core::ptr::null_mut(), lpName, core::ptr::addr_of_mut!(Luid)) == 0 {
            return FALSE;
        }

        dwLength = 0;
        if GetTokenInformation(hToken, TokenPrivileges, core::ptr::null_mut(), 0, core::ptr::addr_of_mut!(dwLength)) == 0 {
            break 'quit;
        }

        pTokenPriv = LocalAlloc(LPTR, dwLength as SIZE_T) as PTOKEN_PRIVILEGES;
        if pTokenPriv.is_null() {
            break 'quit;
        }

        if GetTokenInformation(hToken, TokenPrivileges, ((pTokenPriv) as _), dwLength, core::ptr::addr_of_mut!(dwLength)) != 0 {
            let mut iPriv: UINT = 0;
            let cPrivs: UINT = (*pTokenPriv).PrivilegeCount;
            // Privileges is a C flexible array (`LUID_AND_ATTRIBUTES
            // Privileges[ANYSIZE_ARRAY]`, xdk/setypes.h:1036, and ANYSIZE_ARRAY is 1) in a
            // buffer LocalAlloc'd to the size GetTokenInformation asked for, so it holds
            // PrivilegeCount of them. bindgen's [LUID_AND_ATTRIBUTES; 1] bounds-checks this
            // walk and panics at the SECOND privilege -- and a real token carries ~20, with
            // SE_SHUTDOWN_NAME nowhere near first, so IsShutdownAllowed() panics rather
            // than answering. Constant index 0 is checked at compile time.
            while ret == 0 && iPriv < cPrivs {
                let priv_ = (&raw const (*pTokenPriv).Privileges[0]).add(iPriv as usize);
                ret = ((Luid.LowPart == (*priv_).Luid.LowPart) &&
                       (Luid.HighPart == (*priv_).Luid.HighPart)) as BOOL;
                iPriv += 1;
            }
        }

        LocalFree(((pTokenPriv) as _));
    }

    if hToken == hNewToken {
        CloseHandle(hNewToken);
    }

    ret
}
#[no_mangle]
pub unsafe extern "C" fn IsShutdownAllowed() -> BOOL {
    SHTestTokenPrivilegeW(core::ptr::null_mut(), SE_SHUTDOWN_NAME())
}

#[no_mangle]
pub unsafe extern "system" fn IsSuspendAllowed() -> BOOL {
    if IsShutdownAllowed() != 0 && IsPwrSuspendAllowed() != 0 {
        1
    } else {
        0
    }
}

#[no_mangle]
pub unsafe extern "system" fn SHGetShellStyleHInstance() -> HINSTANCE {
    TRACE!("SHGetShellStyleHInstance called\n");
    let mut hInst: HINSTANCE = core::ptr::null_mut();
    let mut szPath: [u16; ((MAX_PATH) as usize)] = [0u16; ((MAX_PATH) as usize)];
    let mut szColorName: [u16; 100] = [0u16; 100];
    let mut strShellStyle: [u16; ((MAX_PATH + 100 + 64) as usize)] =
        [0u16; ((MAX_PATH + 100 + 64) as usize)];
    let mut strShellStyleLen: usize = 0;

    unsafe fn copy_wz(src: *const u16, dst: *mut u16) -> usize {
        let mut len = 0usize;
        let mut s = src;
        let mut d = dst;
        loop {
            let ch = *s;
            if ch == 0 {
                break;
            }
            *d = ch;
            s = s.add(1);
            d = d.add(1);
            len += 1;
        }
        len
    }

    let hr: HRESULT = GetCurrentThemeName(
        szPath.as_mut_ptr(),
        ((szPath.len()) as i32),
        szColorName.as_mut_ptr(),
        ((szColorName.len()) as i32),
        core::ptr::null_mut(),
        0,
    );

    if hr >= 0 {
        PathRemoveFileSpecW(szPath.as_mut_ptr());

        strShellStyleLen = copy_wz(szPath.as_ptr(), strShellStyle.as_mut_ptr());
        strShellStyleLen += copy_wz(
            CPP_COM_WSTR_2BF0903157CD.as_ptr(),
            strShellStyle.as_mut_ptr().add(strShellStyleLen),
        );
        strShellStyleLen += copy_wz(
            szColorName.as_ptr(),
            strShellStyle.as_mut_ptr().add(strShellStyleLen),
        );
        strShellStyleLen += copy_wz(
            CPP_COM_WSTR_4E317A7FF63D.as_ptr(),
            strShellStyle.as_mut_ptr().add(strShellStyleLen),
        );

        hInst = LoadLibraryExW(
            strShellStyle.as_ptr(),
            core::ptr::null_mut(),
            LOAD_LIBRARY_AS_DATAFILE,
        );
        if !hInst.is_null() {
            return hInst;
        }
    }

    if ExpandEnvironmentStringsW(
        CPP_COM_WSTR_9105F5027482.as_ptr(),
        szPath.as_mut_ptr(),
        ((szPath.len()) as u32),
    ) == 0
    {
        return core::ptr::null_mut();
    }

    return LoadLibraryExW(
        szPath.as_ptr(),
        core::ptr::null_mut(),
        LOAD_LIBRARY_AS_DATAFILE,
    );
}

#[no_mangle]
pub unsafe extern "system" fn SHCreatePropertyBag(
    riid: REFIID,
    ppvObj: *mut *mut core::ffi::c_void,
) -> i32 {
    SHCreatePropertyBagOnMemory(STGM_READWRITE, riid.cast(), ppvObj)
}
unsafe extern "C" fn SHELL_ReadSingleUnreadMailCount(
    hKey: HKEY,
    pdwCount: PDWORD,
    pFileTime: PFILETIME,
    pszShellExecuteCommand: PWSTR,
    cchShellExecuteCommand: INT,
) -> DWORD {
    let mut dwType: DWORD = 0;
    let mut dwCount: DWORD = 0;
    let mut cbSize: DWORD = core::mem::size_of::<DWORD>() as DWORD;

    let mut error: DWORD = SHQueryValueExW(
        hKey,
        CPP_COM_WSTR_8758F49A4AA2.as_ptr(),
        core::ptr::null_mut(),
        &mut dwType,
        core::ptr::addr_of_mut!(dwCount).cast(),
        &mut cbSize,
    );
    if error != 0 {
        return error;
    }
    if !pdwCount.is_null() && dwType == REG_DWORD {
        *pdwCount = dwCount;
    }

    let mut FileTime: FILETIME = core::mem::zeroed();
    cbSize = core::mem::size_of::<FILETIME>() as DWORD;
    error = SHQueryValueExW(
        hKey,
        CPP_COM_WSTR_CC16ED28C79D.as_ptr(),
        core::ptr::null_mut(),
        &mut dwType,
        core::ptr::addr_of_mut!(FileTime).cast(),
        &mut cbSize,
    );
    if error != 0 {
        return error;
    }
    if !pFileTime.is_null() && dwType == REG_BINARY {
        // FILETIME here is the island's `_FILETIME`; `*pFileTime` is the hand-written
        // one, so the C++ struct assignment is a byte copy.
        core::ptr::copy_nonoverlapping(core::ptr::addr_of!(FileTime).cast(), pFileTime, 1);
    }

    let mut szName = [0u16; (2 * MAX_PATH) as usize];
    cbSize = core::mem::size_of_val(&szName) as DWORD;
    error = SHQueryValueExW(
        hKey,
        CPP_COM_WSTR_0C7319B93B0B.as_ptr(),
        core::ptr::null_mut(),
        &mut dwType,
        szName.as_mut_ptr().cast(),
        &mut cbSize,
    );
    if error != 0 {
        return error;
    }

    if !pszShellExecuteCommand.is_null()
        && dwType == REG_SZ
        && (StringCchCopyW(
            pszShellExecuteCommand,
            cchShellExecuteCommand as usize,
            szName.as_ptr(),
        ) < 0)
    {
        return ERROR_INSUFFICIENT_BUFFER;
    }

    return ERROR_SUCCESS;
}
#[no_mangle]
pub unsafe extern "system" fn SHGetUnreadMailCountW(
    mut hKeyUser: HKEY,
    pszMailAddress: PCWSTR,
    pdwCount: PDWORD,
    pFileTime: PFILETIME,
    pszShellExecuteCommand: PWSTR,
    cchShellExecuteCommand: INT,
) -> i32 {
    let mut error: u32;
    let mut hKey: HKEY = core::ptr::null_mut();

    if hKeyUser.is_null() {
        hKeyUser = HKEY_CURRENT_USER;
    }

    if !pszMailAddress.is_null() {
        let mut strKey: [u16; (2 * MAX_PATH) as usize] =
            [0u16; (2 * MAX_PATH) as usize];
        let mut cch = 0usize;
        let mut p = CPP_COM_WSTR_2A3E9AB31456.as_ptr();
        while *p != 0 {
            strKey[cch] = *p;
            cch += 1;
            p = p.add(1);
        }
        strKey[cch] = 0x005Cu16;
        cch += 1;
        let mut p = pszMailAddress;
        while *p != 0 {
            strKey[cch] = *p;
            cch += 1;
            p = p.add(1);
        }
        strKey[cch] = 0;

        error = RegOpenKeyExW(
            hKeyUser,
            strKey.as_ptr(),
            0,
            KEY_QUERY_VALUE,
            &mut hKey,
        ) as u32;
        if error != 0 {
            return HRESULT_FROM_WIN32(error as LONG);
        }

        error = SHELL_ReadSingleUnreadMailCount(
            hKey,
            pdwCount,
            pFileTime,
            pszShellExecuteCommand,
            cchShellExecuteCommand,
        ) as u32;
    } else {
        if !pszShellExecuteCommand.is_null() || cchShellExecuteCommand != 0 {
            return E_INVALIDARG;
        }

        *pdwCount = 0;

        error = RegOpenKeyExW(
            hKeyUser,
            CPP_COM_WSTR_2A3E9AB31456.as_ptr(),
            0,
            KEY_ENUMERATE_SUB_KEYS,
            &mut hKey,
        ) as u32;
        if error != 0 {
            return HRESULT_FROM_WIN32(error as LONG);
        }

        let mut dwIndex: u32 = 0;
        while error == 0 {
            let mut Name: [u16; (2 * MAX_PATH) as usize] =
                [0u16; (2 * MAX_PATH) as usize];
            let mut cchName: u32 = Name.len() as u32;
            let mut LastWritten: FILETIME = core::mem::zeroed();

            error = RegEnumKeyExW(
                hKey,
                dwIndex,
                Name.as_mut_ptr(),
                &mut cchName,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                &mut LastWritten,
            ) as u32;
            if error != 0 {
                break;
            }

            let mut hSubKey: HKEY = core::ptr::null_mut();
            error = RegOpenKeyExW(
                hKey,
                Name.as_mut_ptr(),
                0,
                KEY_QUERY_VALUE,
                &mut hSubKey,
            ) as u32;
            if error != 0 {
                break;
            }

            let mut FileTime: FILETIME = core::mem::zeroed();
            let mut dwCount: u32 = 0;

            error = SHELL_ReadSingleUnreadMailCount(
                hSubKey,
                &mut dwCount,
                core::ptr::addr_of_mut!(FileTime).cast(),
                core::ptr::null_mut(),
                0,
            ) as u32;

            if error == 0
                && (pFileTime.is_null()
                    || CompareFileTime(
                        &FileTime as *const FILETIME,
                        ((pFileTime) as _),
                    ) >= 0)
            {
                *pdwCount += dwCount;
            }

            RegCloseKey(hSubKey);

            dwIndex += 1;
        }

        if error == ERROR_NO_MORE_ITEMS {
            error = ERROR_SUCCESS;
        }
    }

    RegCloseKey(hKey);

    if error != 0 {
        HRESULT_FROM_WIN32(error as LONG)
    } else {
        S_OK
    }
}

#[no_mangle]
pub unsafe extern "system" fn SHSetUnreadMailCountW(
    pszMailAddress: PCWSTR,
    dwCount: DWORD,
    pszShellExecuteCommand: PCWSTR,
) -> HRESULT {
    // The C++ is `CString strKey = L"...\\UnreadMail\\"; strKey += pszMailAddress;`.
    // rustconv reached for a `Vec<u16>`, which this `no_std` crate does not have -- and the
    // shape was wrong even where Vec exists: `with_capacity` leaves LEN at 0, so every write
    // went through `as_mut_ptr()` past the end of the initialized region.
    let mut strKeyBuf = CStringW::from_ptr(CPP_COM_WSTR_05395ACBD6C2.as_ptr());
    strKeyBuf.Append(pszMailAddress);
    let strKey: PCWSTR = strKeyBuf.as_ptr();

    let mut hKey: HKEY = core::ptr::null_mut();
    let mut dwDisposition: DWORD = 0;
    let mut error: LSTATUS = 0;

    error = RegCreateKeyExW(
        HKEY_CURRENT_USER,
        strKey,
        0,
        core::ptr::null_mut(),
        0,
        KEY_SET_VALUE,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(hKey),
        core::ptr::addr_of_mut!(dwDisposition),
    );
    if error != 0 {
        return HRESULT_FROM_WIN32(error as LONG);
    }

    error = RegSetValueExW(
        hKey,
        CPP_COM_WSTR_8758F49A4AA2.as_ptr(),
        0,
        REG_DWORD,
        core::ptr::addr_of!(dwCount) as *const u8,
        core::mem::size_of::<DWORD>() as DWORD,
    );
    if error != 0 {
        RegCloseKey(hKey);
        return HRESULT_FROM_WIN32(error as LONG);
    }

    let mut FileTime: FILETIME = core::mem::zeroed();
    GetSystemTimeAsFileTime(core::ptr::addr_of_mut!(FileTime));

    error = RegSetValueExW(
        hKey,
        CPP_COM_WSTR_CC16ED28C79D.as_ptr(),
        0,
        REG_BINARY,
        core::ptr::addr_of!(FileTime) as *const u8,
        core::mem::size_of::<FILETIME>() as DWORD,
    );
    if error != 0 {
        RegCloseKey(hKey);
        return HRESULT_FROM_WIN32(error as LONG);
    }

    let mut szBuff = [0u16; (2 * MAX_PATH) as usize];
    if PathUnExpandEnvStringsW(
        pszShellExecuteCommand,
        szBuff.as_mut_ptr(),
        szBuff.len() as u32,
    ) == FALSE
    {
        let hr: HRESULT = StringCchCopyW(
            szBuff.as_mut_ptr(),
            szBuff.len() as usize,
            pszShellExecuteCommand,
        );
        if ((hr) < 0) {
            RegCloseKey(hKey);
            return hr;
        }
    }

    let cbValue: DWORD = ((lstrlenW(szBuff.as_ptr()) + 1) as usize
        * core::mem::size_of::<u16>()) as DWORD;
    error = RegSetValueExW(
        hKey,
        CPP_COM_WSTR_0C7319B93B0B.as_ptr(),
        0,
        REG_SZ,
        szBuff.as_ptr() as *const u8,
        cbValue,
    );

    RegCloseKey(hKey);
    if error != 0 {
        HRESULT_FROM_WIN32(error as LONG)
    } else {
        S_OK
    }
}

#[no_mangle]
pub unsafe extern "system" fn SheRemoveQuotesA(psz: PCHAR) -> PCHAR {
    if *psz == '"' as c_char {
        let mut pch = psz.add(1);
        while *pch != 0 && *pch != '"' as c_char {
            *(pch.sub(1)) = *pch;
            pch = pch.add(1);
        }
        if *pch == '"' as c_char {
            *(pch.sub(1)) = ANSI_NULL as c_char;
        }
    }
    psz
}

#[no_mangle]
pub unsafe extern "system" fn SheRemoveQuotesW(psz: PWSTR) -> PWSTR {
    unsafe {
        let mut pch: *mut u16;

        if *psz == 0x0022u16 {
            pch = psz.add(1);
            while *pch != 0 && *pch != 0x0022u16 {
                pch.sub(1).write(*pch);
                pch = pch.add(1);
            }

            if *pch == 0x0022u16 {
                *pch.sub(1) = UNICODE_NULL;
            }
        }

        psz
    }
}

#[no_mangle]
pub unsafe extern "system" fn SHEnumerateUnreadMailAccountsW(
    mut hKeyUser: HKEY,
    dwIndex: DWORD,
    pszMailAddress: PWSTR,
    cchMailAddress: INT,
) -> HRESULT {
    if hKeyUser.is_null() {
        hKeyUser = HKEY_CURRENT_USER;
    }

    let mut hKey: HKEY = core::ptr::null_mut();
    let mut error: LSTATUS = RegOpenKeyExW(
        hKeyUser,
        CPP_COM_WSTR_2A3E9AB31456.as_ptr(),
        0,
        KEY_ENUMERATE_SUB_KEYS,
        core::ptr::addr_of_mut!(hKey),
    );
    if error != 0 {
        return HRESULT_FROM_WIN32(error as LONG);
    }

    let mut FileTime: FILETIME = core::mem::zeroed();
    let mut cchMailAddress_dword = cchMailAddress as DWORD;
    error = RegEnumKeyExW(
        hKey,
        dwIndex,
        pszMailAddress,
        core::ptr::addr_of_mut!(cchMailAddress_dword),
        core::ptr::null_mut::<DWORD>(),
        core::ptr::null_mut::<u16>(),
        core::ptr::null_mut::<DWORD>(),
        core::ptr::addr_of_mut!(FileTime),
    );
    if error != 0 {
        *pszMailAddress = UNICODE_NULL;
    }

    RegCloseKey(hKey);
    if error != 0 {
        HRESULT_FROM_WIN32(error as LONG)
    } else {
        S_OK
    }
}

#[no_mangle]
pub unsafe extern "system" fn SHFindComputer(pidlRoot: LPCITEMIDLIST, pidlSavedSearch: LPCITEMIDLIST) -> i32 {
    UNREFERENCED_PARAMETER!(pidlRoot);
    UNREFERENCED_PARAMETER!(pidlSavedSearch);

    TRACE!("%p %p\n", pidlRoot, pidlSavedSearch);

    let mut pCM: *mut IContextMenu = core::ptr::null_mut();
    let mut hr: i32 = CoCreateInstance(
        &CLSID_ShellSearchExt,
        core::ptr::null_mut(),
        CLSCTX_INPROC_SERVER,
        &IID_IContextMenu,
        core::ptr::addr_of_mut!(pCM) as *mut *mut core::ffi::c_void,
    );
    if hr < 0 {
        return hr;
    }

    let mut InvokeInfo: CMINVOKECOMMANDINFO = core::mem::zeroed();
    InvokeInfo.cbSize = core::mem::size_of::<CMINVOKECOMMANDINFO>() as u32;
    InvokeInfo.lpParameters = b"{996E1EB1-B524-11D1-9120-00A0C98BA67D}\0".as_ptr() as *const i8;
    InvokeInfo.nShow = ((SW_SHOWNORMAL) as i32);
    hr = ((*(*pCM).lpVtbl).InvokeCommand)(pCM, core::ptr::addr_of_mut!(InvokeInfo).cast());
    ((*(*pCM).lpVtbl).Release)(pCM);

    (hr >= 0) as i32
}
unsafe extern "C" fn Int64ToStr(
    mut llValue: i64,
    pszValue: PWSTR,
    cchValue: u32
) -> i32 {
    unsafe {
        let mut szBuff = [0u16; 40];
        let mut ich: u32 = 0;
        let mut ichValue: u32;
        let bMinus: i32 = if llValue < 0 { 1 } else { 0 };

        if bMinus != 0 {
            llValue = llValue.wrapping_neg();
        }

        if cchValue <= 0 {
            return E_FAIL;
        }

        loop {
            szBuff[ich as usize] = (0x0030u16 + ((llValue % 10) as u16));
            ich += 1;
            llValue /= 10;
            if !(llValue != 0 && (ich as usize) < &szBuff.len() - 1) {
                break;
            }
        }

        if bMinus != 0 && (ich as usize) < szBuff.len() {
            szBuff[ich as usize] = '-' as u16;
            ich += 1;
        }

        ichValue = 0;
        while ich > 0 && ichValue < cchValue {
            ich -= 1;
            *pszValue.add(ichValue as usize) = szBuff[ich as usize];
            ichValue += 1;
        }

        if ichValue >= cchValue {
            *pszValue.add((cchValue - 1) as usize) = UNICODE_NULL;
            return E_FAIL;
        }

        *pszValue.add(ichValue as usize) = UNICODE_NULL;
        return S_OK;
    }
}
unsafe extern "C" fn Int64GetNumFormat(
    pDest: *mut NUMBERFMTW,
    pSrc: *const NUMBERFMTW,
    mut dwNumberFlags: u32,
    pszDecimal: PWSTR,
    cchDecimal: i32,
    pszThousand: PWSTR,
    cchThousand: i32,
) {
    let mut szBuff = [0u16; 20];

    if !pSrc.is_null() {
        // NUMBERFMTW is a bindgen struct with no Copy; the C++ is a struct assignment.
        core::ptr::copy_nonoverlapping(pSrc, pDest, 1);
    } else {
        dwNumberFlags = 0;
    }

    if (dwNumberFlags & FMT_USE_NUMDIGITS) == 0 {
        GetLocaleInfoW(
            LOCALE_USER_DEFAULT,
            LOCALE_IDIGITS,
            szBuff.as_mut_ptr(),
            szBuff.len() as i32,
        );
        (*pDest).NumDigits = StrToIntW(szBuff.as_ptr()) as u32;
    }

    if (dwNumberFlags & FMT_USE_LEADZERO) == 0 {
        GetLocaleInfoW(
            LOCALE_USER_DEFAULT,
            LOCALE_ILZERO,
            szBuff.as_mut_ptr(),
            szBuff.len() as i32,
        );
        (*pDest).LeadingZero = StrToIntW(szBuff.as_ptr()) as u32;
    }

    if (dwNumberFlags & FMT_USE_GROUPING) == 0 {
        GetLocaleInfoW(
            LOCALE_USER_DEFAULT,
            LOCALE_SGROUPING,
            szBuff.as_mut_ptr(),
            szBuff.len() as i32,
        );
        (*pDest).Grouping = StrToIntW(szBuff.as_ptr()) as u32;
    }

    if (dwNumberFlags & FMT_USE_DECIMAL) == 0 {
        GetLocaleInfoW(
            LOCALE_USER_DEFAULT,
            LOCALE_SDECIMAL,
            pszDecimal,
            cchDecimal,
        );
        (*pDest).lpDecimalSep = pszDecimal;
    }

    if (dwNumberFlags & FMT_USE_THOUSAND) == 0 {
        GetLocaleInfoW(
            LOCALE_USER_DEFAULT,
            LOCALE_STHOUSAND,
            pszThousand,
            cchThousand,
        );
        (*pDest).lpThousandSep = pszThousand;
    }

    if (dwNumberFlags & FMT_USE_NEGNUMBER) == 0 {
        GetLocaleInfoW(
            LOCALE_USER_DEFAULT,
            LOCALE_INEGNUMBER,
            szBuff.as_mut_ptr(),
            szBuff.len() as i32,
        );
        (*pDest).NegativeOrder = StrToIntW(szBuff.as_ptr()) as u32;
    }
}

#[no_mangle]
pub unsafe extern "system" fn Int64ToString(
    llValue: i64,
    pszOut: PWSTR,
    cchOut: u32,
    bUseFormat: i32,
    pNumberFormat: *const NUMBERFMTW,
    dwNumberFlags: u32,
) -> i32 {
    let mut szValue: [u16; 80] = [0; 80];
    let mut szDecimalSep: [u16; 6] = [0; 6];
    let mut szThousandSep: [u16; 6] = [0; 6];
    let mut NumFormat: NUMBERFMTW = core::mem::zeroed();

    Int64ToStr(llValue, szValue.as_mut_ptr(), szValue.len() as u32);

    if bUseFormat != 0 {
        Int64GetNumFormat(
            core::ptr::addr_of_mut!(NumFormat),
            pNumberFormat,
            dwNumberFlags,
            szDecimalSep.as_mut_ptr(),
            szDecimalSep.len() as i32,
            szThousandSep.as_mut_ptr(),
            szThousandSep.len() as i32,
        );
        let mut ret: i32 = GetNumberFormatW(
            LOCALE_USER_DEFAULT,
            0,
            szValue.as_ptr(),
            core::ptr::addr_of!(NumFormat),
            pszOut,
            cchOut as i32,
        );
        if ret != 0 {
            ret -= 1;
        }
        return ret;
    }

    if StringCchCopyW(pszOut, cchOut as usize, szValue.as_ptr()) < 0 {
        return 0;
    }

    return lstrlenW(pszOut);
}

#[no_mangle]
pub unsafe extern "system" fn LargeIntegerToString(
    pLargeInt: *const LARGE_INTEGER,
    pszOut: PWSTR,
    cchOut: u32,
    bUseFormat: i32,
    pNumberFormat: *const NUMBERFMTW,
    dwNumberFlags: u32,
) -> i32 {
    Int64ToString((*pLargeInt).QuadPart, pszOut, cchOut, bUseFormat, pNumberFormat, dwNumberFlags)
}

#[no_mangle]
pub unsafe extern "system" fn CopyStreamUI(
    pSrc: *mut IStream,
    pDst: *mut IStream,
    mut pProgress: *mut IProgressDialog,
    mut dwlSize: u64,
) -> i32 {
    let mut hr: i32 = E_FAIL;
    let mut cbBuff: u32 = 0;
    let mut cbRead: u32 = 0;
    let mut dwSizeToWrite: u32 = 0;
    let mut cbDone: u64 = 0;
    let mut pBuff: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut pHeapPtr: *mut u8 = core::ptr::null_mut();
    let mut heap_allocated = false;
    let mut Stat: STATSTG = core::mem::zeroed();
    let mut abBuff: [u8; 1024] = [0; 1024];

    ZeroMemory(
        &mut Stat as *mut STATSTG as *mut core::ffi::c_void,
        core::mem::size_of::<STATSTG>(),
    );

    if dwlSize == 0 {
        let stat_hr = ((*(*pSrc).lpVtbl).Stat)(pSrc, core::ptr::addr_of_mut!(Stat).cast(), STATFLAG_NONAME);
        if stat_hr < 0 {
            pProgress = core::ptr::null_mut();
        } else {
            dwlSize = Stat.cbSize.QuadPart;
        }
    }

    if pProgress.is_null() {
        let mut uliSize: crate::island::ULARGE_INTEGER = core::mem::zeroed();
        if dwlSize > 0 {
            uliSize.QuadPart = dwlSize;
        } else {
            uliSize.u.HighPart = INVALID_FILE_SIZE;
            uliSize.u.LowPart = INVALID_FILE_SIZE;
        }
        return ((*(*pSrc).lpVtbl).CopyTo)(
            pSrc,
            pDst,
            uliSize,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
    }

    if dwlSize > 0 && dwlSize <= core::mem::size_of::<[u8; 1024]>() as u64 {
        cbBuff = core::mem::size_of::<[u8; 1024]>() as u32;
        pBuff = abBuff.as_mut_ptr() as *mut core::ffi::c_void;
    } else {
        cbBuff = COPY_STREAM_DEFAULT_BUFFER_SIZE;
        let alloc_ptr = CoTaskMemAlloc(cbBuff as SIZE_T) as *mut u8;
        if !alloc_ptr.is_null() {
            pBuff = alloc_ptr as *mut core::ffi::c_void;
            pHeapPtr = alloc_ptr;
            heap_allocated = true;
        } else {
            cbBuff = core::mem::size_of::<[u8; 1024]>() as u32;
            pBuff = abBuff.as_mut_ptr() as *mut core::ffi::c_void;
        }
    }

    let mut zero: crate::island::LARGE_INTEGER = core::mem::zeroed();
    zero.QuadPart = 0;
    ((*(*pSrc).lpVtbl).Seek)(pSrc, zero, 0, core::ptr::null_mut());
    ((*(*pDst).lpVtbl).Seek)(pDst, zero, 0, core::ptr::null_mut());
    cbDone = 0;
    ((*(*pProgress).lpVtbl).SetProgress64)(pProgress, cbDone, dwlSize);

    loop {
        hr = ((*(*pSrc).lpVtbl).Read)(pSrc, pBuff, cbBuff, &mut cbRead);
        if hr < 0 {
            break;
        }

        if dwlSize > 0 {
            let remaining = dwlSize - cbDone;
            let read_amount = cbRead as u64;
            dwSizeToWrite = if remaining < read_amount { remaining as u32 } else { cbRead };
        } else {
            dwSizeToWrite = cbRead;
        }

        if dwSizeToWrite == 0 {
            hr = S_OK;
            break;
        }

        hr = ((*(*pDst).lpVtbl).Write)(pDst, pBuff, dwSizeToWrite, core::ptr::null_mut());
        if hr != S_OK {
            break;
        }

        cbDone += dwSizeToWrite as u64;

        if ((*(*pProgress).lpVtbl).HasUserCancelled)(pProgress) != 0 {
            hr = HRESULT_FROM_WIN32(ERROR_CANCELLED as LONG);
            break;
        }
        ((*(*pProgress).lpVtbl).SetProgress64)(pProgress, cbDone, dwlSize);

        if dwlSize > 0 && cbDone >= dwlSize {
            hr = S_OK;
            break;
        }
    }

    if heap_allocated {
        CoTaskMemFree(((pHeapPtr) as _));
    }
    hr
}

#[no_mangle]
pub unsafe extern "system" fn Activate_RunDLL(
    hwnd: HWND,
    hinst: HINSTANCE,
    cmdline: PCWSTR,
    cmdshow: INT,
) -> BOOL {
    let mut dwProcessID: DWORD = 0;

    UNREFERENCED_PARAMETER!(hwnd);
    UNREFERENCED_PARAMETER!(hinst);
    UNREFERENCED_PARAMETER!(cmdline);
    UNREFERENCED_PARAMETER!(cmdshow);

    GetWindowThreadProcessId(GetShellWindow(), &mut dwProcessID);
    AllowSetForegroundWindow(dwProcessID)
}

#[no_mangle]
pub unsafe extern "system" fn SHStartNetConnectionDialogA(
    hwnd: HWND,
    pszRemoteName: LPCSTR,
    dwType: DWORD,
) -> HRESULT {
    let mut pszRemoteNameW: PCWSTR = core::ptr::null_mut() as PCWSTR;
    let strRemoteNameW = if !pszRemoteName.is_null() {
        let s = CStringW::from_ansi(pszRemoteName);
        pszRemoteNameW = s.as_ptr() as PCWSTR;
        Some(s)
    } else {
        None
    };
    SHStartNetConnectionDialogW(hwnd, pszRemoteNameW, dwType)
}
unsafe extern "C" fn DynamicPathCommonPrefixW(
    lpszPath1: PCWSTR,
    lpszPath2: PCWSTR,
    strPath: &mut CStringW,
) -> i32 {
    let cchPath1 = wcslen(lpszPath1);
    let cchPath2 = wcslen(lpszPath2);
    let lpszPath = strPath.GetBuffer(max(cchPath1, cchPath2) as i32 + 16 as i32);
    let ret = PathCommonPrefixW(lpszPath1, lpszPath2, lpszPath);
    strPath.ReleaseBuffer();
    ret
}
unsafe extern "C" fn DynamicSHGetPathFromIDListW(
    pidl: LPCITEMIDLIST,
    strPath: &mut CStringW,
) -> HRESULT {
    let mut hr;
    let mut cchPath: u32 = MAX_PATH;
    loop {
        let lpszPath = strPath.GetBuffer(cchPath as i32);
        if lpszPath.is_null() {
            return E_OUTOFMEMORY;
        }

        hr = SHGetPathCchFromIDListW(((pidl) as _), lpszPath, cchPath as usize);
        strPath.ReleaseBuffer();

        if hr != E_NOT_SUFFICIENT_BUFFER {
            break;
        }

        if cchPath >= MAXUINT / 2 {
            hr = E_FAIL;
            break;
        }

        cchPath *= 2;
    }

    if hr < 0 {
        strPath.Empty();
    }

    hr
}
unsafe extern "C" fn DynamicSHGetSpecialFolderPathW(
    hwndOwner: HWND,
    strPath: &mut CStringW,
    nCSIDL: i32,
    bCreate: i32,
) -> HRESULT {
    let mut pidl: LPITEMIDLIST = core::ptr::null_mut();
    let mut hr: HRESULT = SHGetSpecialFolderLocation(hwndOwner.cast(), nCSIDL, core::ptr::addr_of_mut!(pidl));
    if hr >= 0 {
        hr = DynamicSHGetPathFromIDListW(pidl, &mut *strPath);
        CoTaskMemFree(((pidl) as _));
    }
    if hr < 0 {
        strPath.Empty();
    } else if bCreate != 0 {
        CreateDirectoryW(strPath.as_ptr(), core::ptr::null_mut());
    }
    hr
}
unsafe extern "C" fn DynamicPathRemoveBackslashW(strPath: &mut CStringW) {
    let nLength = strPath.GetLength();
    if nLength > 0 && strPath.GetAt(nLength - 1) == 0x005Cu16 {
        *strPath = strPath.Left(nLength - 1);
    }
}

#[no_mangle]
pub unsafe extern "system" fn PathIsEqualOrSubFolder(
    pszPath1OrCSIDL: PCWSTR,
    pszPath2: PCWSTR,
) -> BOOL {
    let mut strCommon = CStringW::new();
    let mut strPath1 = CStringW::new();

    if IS_INTRESOURCE(pszPath1OrCSIDL) {
        DynamicSHGetSpecialFolderPathW(
            core::ptr::null_mut(),
            &mut strPath1,
            (LOWORD!(pszPath1OrCSIDL) as i32) | ((CSIDL_FLAG_DONT_VERIFY) as i32),
            FALSE,
        );
    } else {
        strPath1 = CStringW::from_ptr(pszPath1OrCSIDL);
    }

    DynamicPathRemoveBackslashW(&mut strPath1);

    if DynamicPathCommonPrefixW(strPath1.as_ptr(), pszPath2, &mut strCommon) == 0 {
        return FALSE;
    }

    (strPath1.CompareNoCase(strCommon.as_ptr()) == 0) as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn SHGetRealIDL(
    psf: *mut IShellFolder,
    pidlSimple: PCUITEMID_CHILD,
    ppidlReal: *mut PITEMID_CHILD,
) -> HRESULT {
    let mut hr: HRESULT;
    let mut strret: STRRET = core::mem::zeroed();
    let mut szPath = [0u16; ((MAX_PATH) as usize)];
    let mut attrs: SFGAOF = SFGAO_FILESYSTEM;

    *ppidlReal = core::ptr::null_mut();

    hr = IShellFolder_GetDisplayNameOf(
        psf,
        pidlSimple,
        SHGDN_INFOLDER | SHGDN_FORPARSING,
        core::ptr::addr_of_mut!(strret),
        0
    );
    if ((hr) < 0) {
        return hr;
    }

    hr = StrRetToBufW(
        core::ptr::addr_of_mut!(strret),
        pidlSimple,
        szPath.as_mut_ptr(),
        ((szPath.len()) as u32)
    );
    if ((hr) < 0) {
        return hr;
    }

    hr = ((*(*psf).lpVtbl).GetAttributesOf)(psf,
        1,
        ((core::ptr::addr_of!(pidlSimple)) as _),
        core::ptr::addr_of_mut!(attrs)
    );
    if hr >= 0 && (attrs & SFGAO_FILESYSTEM) == 0 {
        return SHILClone(pidlSimple, ppidlReal);
    }

    hr = IShellFolder_ParseDisplayName(
        psf,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        szPath.as_mut_ptr(),
        core::ptr::null_mut(),
        ppidlReal,
        core::ptr::null_mut()
    );
    if hr == E_INVALIDARG || hr == E_NOTIMPL {
        return SHILClone(pidlSimple, ppidlReal);
    }

    hr
}

#[no_mangle]
pub unsafe extern "C" fn IUnknown_InitializeCommand(
    pUnk: *mut IUnknown,
    pszCommandName: PCWSTR,
    pPB: *mut IPropertyBag,
) -> HRESULT {
    let mut hr;
    let mut pIC: *mut IInitializeCommand = core::ptr::null_mut();
    let _pIC_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pIC));
    if {
        hr = ((*(*pUnk).lpVtbl).QueryInterface)(pUnk, ((core::ptr::addr_of!(IID_IInitializeCommand)) as _), core::ptr::addr_of_mut!(pIC).cast());
        hr >= 0
    } {
        hr = ((*(*pIC).lpVtbl).Initialize)(pIC, pszCommandName, ((pPB) as _));
    }
    hr
}
#[no_mangle]
pub unsafe extern "C" fn InvokeIExecuteCommand(
    pEC: *mut IExecuteCommand,
    pszCommandName: PCWSTR,
    pPB: *mut IPropertyBag,
    pSIA: *mut IShellItemArray,
    pICI: *const CMINVOKECOMMANDINFOEX,
    pSite: *mut IUnknown,
) -> HRESULT {
    if pEC.is_null() {
        return E_INVALIDARG;
    }

    let _site = CScopedSetObjectWithSite::new(pEC.cast(), pSite.cast());
    IUnknown_InitializeCommand(((pEC) as _), pszCommandName, pPB);

    let mut pOWS: *mut IObjectWithSelection = core::ptr::null_mut();
    let _pOWS_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pOWS));
    if !pSIA.is_null() {
        let hr = ((*(*pEC).lpVtbl).QueryInterface)(pEC,
            core::ptr::addr_of!(IID_IObjectWithSelection).cast(),
            core::ptr::addr_of_mut!(pOWS) as *mut *mut core::ffi::c_void,
        );
        if hr >= 0 && !pOWS.is_null() {
            ((*(*pOWS).lpVtbl).SetSelection)(pOWS, pSIA);
        }
    }

    let mut dwKeyState: u32 = 0;
    let fMask: u32 = if !pICI.is_null() {
        (*pICI).fMask
    } else {
        0
    };

    ((*(*pEC).lpVtbl).SetNoShowUI)(pEC, if (fMask & CMIC_MASK_FLAG_NO_UI) != 0 {
        1
    } else {
        0
    });
    ((*(*pEC).lpVtbl).SetShowWindow)(pEC, if !pICI.is_null() {
        (*pICI).nShow
    } else {
        ((SW_SHOW) as i32)
    });

    if (fMask & CMIC_MASK_SHIFT_DOWN) != 0 {
        dwKeyState |= MK_SHIFT;
    }
    if (fMask & CMIC_MASK_CONTROL_DOWN) != 0 {
        dwKeyState |= MK_CONTROL;
    }
    ((*(*pEC).lpVtbl).SetKeyState)(pEC, dwKeyState);

    if !pICI.is_null()
        && (fMask & CMIC_MASK_UNICODE) != 0
        && !(*pICI).lpDirectoryW.is_null()
    {
        ((*(*pEC).lpVtbl).SetDirectory)(pEC, (*pICI).lpDirectoryW);
    }
    if !pICI.is_null()
        && (fMask & CMIC_MASK_UNICODE) != 0
        && !(*pICI).lpParametersW.is_null()
    {
        ((*(*pEC).lpVtbl).SetParameters)(pEC, (*pICI).lpParametersW);
    }
    if !pICI.is_null() && (fMask & CMIC_MASK_PTINVOKE) != 0 {
        ((*(*pEC).lpVtbl).SetPosition)(pEC, (*pICI).ptInvoke);
    }

    let result = ((*(*pEC).lpVtbl).Execute)(pEC);
    result
}
#[no_mangle]
pub unsafe extern "C" fn InvokeIExecuteCommandWithDataObject(
    pEC: *mut IExecuteCommand,
    pszCommandName: PCWSTR,
    pPB: *mut IPropertyBag,
    pDO: *mut IDataObject,
    pICI: *const CMINVOKECOMMANDINFOEX,
    pSite: *mut IUnknown,
) -> HRESULT {
    let mut pSIA: *mut IShellItemArray = core::ptr::null_mut();
    let _pSIA_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pSIA));
    let hr: HRESULT = SHCreateShellItemArrayFromDataObject(
        pDO,
        ((core::ptr::addr_of!(IID_IShellItemArray)) as _), core::ptr::addr_of_mut!(pSIA).cast(),
    );

    let result = if hr >= 0 {
        InvokeIExecuteCommand(pEC, pszCommandName, pPB, pSIA, pICI, pSite)
    } else {
        hr
    };


    result
}
unsafe extern "C" fn GetCommandStringA(
    pCM: *mut IContextMenu,
    Id: usize,
    GCS: u32,
    Buf: *mut i8,
    cchMax: u32
) -> i32 {
    let mut hr = ((*(*pCM).lpVtbl).GetCommandString)(
        pCM,
        Id,
        GCS & !GCS_UNICODE,
        core::ptr::null_mut::<u32>(),
        Buf,
        cchMax
    );
    if hr < 0 {
        let mut buf = [0u16; ((MAX_PATH) as usize)];
        hr = ((*(*pCM).lpVtbl).GetCommandString)(
            pCM,
            Id,
            GCS | GCS_UNICODE,
            core::ptr::null_mut::<u32>(),
            buf.as_mut_ptr() as *mut i8,
            buf.len() as u32
        );
        if hr >= 0 {
            hr = if SHUnicodeToAnsi(buf.as_ptr(), Buf, ((cchMax) as i32)) > 0 {
                S_OK
            } else {
                E_FAIL
            };
        }
    }
    hr
}
#[no_mangle]
pub unsafe extern "C" fn GetDfmCmd(pCM: *mut IContextMenu, mut verba: *const i8) -> u32 {
    unsafe {
        let mut buf: [i8; ((MAX_PATH) as usize)] = [0; ((MAX_PATH) as usize)];
        if IS_INTRESOURCE(verba) as u32 != 0 {
            if GetCommandStringA(
                pCM,
                LOWORD!(verba) as usize,
                GCS_VERB,
                buf.as_mut_ptr(),
                buf.len() as u32,
            ) < 0 {
                return 0;
            }
            verba = buf.as_ptr();
        }
        MapVerbToDfmCmd(verba)
    }
}
#[no_mangle]
pub unsafe extern "C" fn SHELL_MapContextMenuVerbToCmdId(
    pICI: LPCMINVOKECOMMANDINFO,
    pMap: *const CMVERBMAP,
) -> HRESULT {
    let mut pVerbA: LPCSTR = (*pICI).lpVerb;
    let mut buf: [CHAR; ((MAX_PATH) as usize)] = [0; ((MAX_PATH) as usize)];
    let pICIX = pICI as LPCMINVOKECOMMANDINFOEX;

    if IsUnicode(pICIX.cast_const().cast()) && !IS_INTRESOURCE((*pICIX).lpVerbW) {
        if SHUnicodeToAnsi((*pICIX).lpVerbW, buf.as_mut_ptr(), ((buf.len()) as i32)) != 0 {
            pVerbA = buf.as_ptr();
        }
    }

    if IS_INTRESOURCE(pVerbA) {
        return LOWORD!(pVerbA) as HRESULT;
    }

    let mut i: SIZE_T = 0;
    while !(*pMap.add(((i) as usize))).Verb.is_null() {
        assert!(((*pMap.add(((i) as usize))).CmdId as i32) >= 0);
        if lstrcmpiA((*pMap.add(((i) as usize))).Verb, pVerbA) == 0
            && *pVerbA.add(0) != 0
        {
            return (*pMap.add(((i) as usize))).CmdId as HRESULT;
        }
        i += 1;
    }

    return E_FAIL;
}
unsafe extern "C" fn FindVerbMapEntry(CmdId: usize, pMap: *const CMVERBMAP) -> *const CMVERBMAP {
    let mut i: usize = 0;
    loop {
        if (*pMap.add(i)).Verb.is_null() {
            break;
        }
        if (*pMap.add(i)).CmdId == ((CmdId) as u16) {
            return pMap.add(i) as *const CMVERBMAP;
        }
        i += 1;
    }
    core::ptr::null_mut::<CMVERBMAP>() as *const CMVERBMAP
}
#[no_mangle]
pub unsafe extern "C" fn SHELL_GetCommandStringImpl(
    CmdId: usize,
    uFlags: u32,
    Buf: LPSTR,
    cchBuf: u32,
    pMap: *const CMVERBMAP,
) -> HRESULT {
    match uFlags | GCS_UNICODE {
        GCS_VALIDATEW | GCS_VERBW => {
            let pEntry: *const CMVERBMAP = FindVerbMapEntry(CmdId, pMap);
            if (uFlags | GCS_UNICODE) == GCS_VERBW {
                if pEntry.is_null() {
                    return E_INVALIDARG;
                } else if (uFlags & GCS_UNICODE) != 0 {
                    return if SHAnsiToUnicode(
                        (*pEntry).Verb,
                        Buf as LPWSTR,
                        cchBuf as i32,
                    ) != 0 {
                        S_OK
                    } else {
                        E_FAIL
                    };
                } else {
                    return StringCchCopyA(
                        Buf,
                        cchBuf as usize,
                        (*pEntry).Verb,
                    );
                }
            }
            return if !pEntry.is_null() {
                S_OK
            } else {
                S_FALSE
            };
        }
        _ => {
            return E_NOTIMPL;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn SHELL_CreateShell32DefaultExtractIcon(
    IconIndex: i32,
    riid: REFIID,
    ppvOut: *mut *mut c_void
) -> HRESULT {
    let mut initIcon: *mut IDefaultExtractIconInit = core::ptr::null_mut();
    let _initIcon_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(initIcon));
    let mut hr = SHCreateDefaultExtractIcon(
        &IID_IDefaultExtractIconInit,
        &mut initIcon as *mut *mut IDefaultExtractIconInit as *mut *mut c_void,
    );
    if ((hr) < 0) {
        return hr;
    }
    ((*(*initIcon).lpVtbl).SetNormalIcon)(initIcon, swShell32Name.as_ptr(), IconIndex);
    let result = ((*(*initIcon).lpVtbl).QueryInterface)(initIcon, ((riid) as _), ppvOut);
    result
}
#[no_mangle]
pub unsafe extern "C" fn DCIA_AddEntry(hDCIA: HDCIA, rClsId: REFCLSID) -> i32 {
    let mut i: u32 = 0;
    loop {
        let pClsId = DCIA_GetEntry(hDCIA, ((i) as i32));
        if pClsId.is_null() {
            break;
        }
        if IsEqualGUID(pClsId, rClsId) {
            return i as i32;
        }
        i += 1;
    }
    DSA_AppendItem(hDCIA as HDSA, rClsId as *mut core::ffi::c_void)
}
#[no_mangle]
pub unsafe extern "C" fn DCIA_AddShellExSubkey(hDCIA: HDCIA, hProgId: HKEY, pszSubkey: PCWSTR) {
    let mut szKey: [u16; 200] = [0; 200];
    PathCombineW(szKey.as_mut_ptr(), CPP_COM_WSTR_C6E922ADD3B3.as_ptr(), pszSubkey);
    let mut hEnum: HKEY = core::ptr::null_mut();
    if RegOpenKeyExW(
        hProgId,
        szKey.as_ptr(),
        0,
        KEY_READ,
        core::ptr::addr_of_mut!(hEnum),
    ) != ((ERROR_SUCCESS) as i32)
    {
        return;
    }
    let mut i: u32 = 0;
    loop {
        let reg_result = RegEnumKeyW(hEnum, i, szKey.as_mut_ptr(), ((szKey.len()) as u32));
        i = i.wrapping_add(1);
        if reg_result != ((ERROR_SUCCESS) as i32) {
            break;
        }
        let mut clsid: CLSID = core::mem::zeroed();
        if SHELL_GetShellExtensionRegCLSID(hEnum.cast(), szKey.as_ptr(), core::ptr::addr_of_mut!(clsid))
            >= 0
        {
            DCIA_AddEntry(hDCIA, core::ptr::addr_of!(clsid).cast());
        }
    }
    RegCloseKey(hEnum);
}

#[no_mangle]
pub unsafe extern "system" fn SHIsBadInterfacePtr(
    pv: *const core::ffi::c_void,
    ucb: usize,
) -> i32 {

    #[repr(C)]
    struct CUnknown {
        lpVtbl: *const CUnknownVtbl,
    }

    let punk = pv as *const CUnknown;

    if punk.is_null() {
        return 1;
    }

    if IsBadReadPtr(
        punk as *const core::ffi::c_void,
        core::mem::size_of::<*const CUnknownVtbl>() as UINT_PTR,
    ) != 0
    {
        return 1;
    }

    if IsBadReadPtr(
        (*punk).lpVtbl as *const core::ffi::c_void,
        ucb as UINT_PTR,
    ) != 0
    {
        return 1;
    }

    if IsBadCodePtr(
        // FARPROC is `Option<unsafe extern "system" fn() -> INT_PTR>`; the slot is a data
        // pointer in this vtable rendering, so it takes a transmute, not an `as` cast.
        Some(core::mem::transmute::<
            *mut core::ffi::c_void,
            unsafe extern "system" fn() -> INT_PTR,
        >((*(*punk).lpVtbl).Release)),
    ) != 0
    {
        return 1;
    }

    0
}

#[no_mangle]
pub unsafe extern "system" fn SHGetUserDisplayName(
    pName: PWSTR,
    puSize: PULONG,
) -> HRESULT {
    if pName.is_null() || puSize.is_null() {
        return E_INVALIDARG;
    }

    if GetUserNameExW(((NameDisplay) as u32), pName, puSize) != 0 {
        return S_OK;
    }

    let error: u32 = GetLastError();
    let mut hr: HRESULT = HRESULT_FROM_WIN32(error as LONG);

    let mut UserName = [0u16; MAX_PATH as usize];
    let mut cchUserName: u32 = ((UserName.len()) as u32);

    if GetUserNameW(UserName.as_mut_ptr(), &mut cchUserName) == 0 {
        return HRESULT_FROM_WIN32(((GetLastError()) as i32));
    }

    if error == ERROR_NONE_MAPPED {
        let mut UserInfo: PUSER_INFO_2 = core::ptr::null_mut();
        let NetError: u32 = NetUserGetInfo(
            core::ptr::null_mut(),
            UserName.as_ptr(),
            2,
            core::ptr::addr_of_mut!(UserInfo) as *mut *mut u8,
        );

        if NetError != 0 {
            hr = HRESULT_FROM_WIN32(NetError as LONG);
        } else {
            if !(*UserInfo).usri2_full_name.is_null() {
                hr = StringCchCopyW(
                    pName,
                    *puSize as usize,
                    (*UserInfo).usri2_full_name,
                );

                if hr >= 0 {
                    *puSize = (lstrlenW((*UserInfo).usri2_full_name)
                        + 1) as u32;
                }
            }

            NetApiBufferFree(((UserInfo) as _));
        }
    }

    if hr < 0 {
        hr = StringCchCopyW(
            pName,
            *puSize as usize,
            UserName.as_ptr(),
        );

        if hr >= 0 {
            *puSize = cchUserName;
        }
    }

    hr
}
unsafe extern "C" fn SHELL_SkipServerSlashes(pszPath: PCWSTR) -> PCWSTR {
    let mut pch = pszPath;
    while *pch == 0x005Cu16 {
        pch = pch.add(1);
    }
    pch
}
unsafe extern "C" fn SHELL_GetCachedComputerDescription(
    pszDesc: PWSTR,
    mut cchDescMax: u32,
    pszServerName: PCWSTR,
) -> HRESULT {
    cchDescMax *= core::mem::size_of::<u16>() as u32;
    let error = SHGetValueW(
        HKEY_CURRENT_USER,
        COMPUTER_DESCRIPTIONS_KEY(),
        SHELL_SkipServerSlashes(pszServerName),
        core::ptr::null_mut(),
        ((pszDesc) as _),
        core::ptr::addr_of_mut!(cchDescMax),
    ) as u32;
    HRESULT_FROM_WIN32(error as LONG)
}
unsafe extern "C" fn SHELL_CacheComputerDescription(pszServerName: PCWSTR, pszDesc: PCWSTR) {
    if pszDesc.is_null() {
        return;
    }

    let cbDesc: usize = (wcslen(pszDesc) as usize + 1) * core::mem::size_of::<u16>();

    SHSetValueW(
        HKEY_CURRENT_USER,
        COMPUTER_DESCRIPTIONS_KEY(),
        SHELL_SkipServerSlashes(pszServerName),
        REG_SZ,
        ((pszDesc) as _),
        cbDesc as u32,
    );
}
unsafe extern "C" fn SHELL_GetComputerDescription(
    pszDesc: PWSTR,
    cchDescMax: SIZE_T,
    pszServerName: PWSTR,
) -> HRESULT {
    let mut bufptr: PSERVER_INFO_101 = core::ptr::null_mut();
    let error: NET_API_STATUS = NetServerGetInfo(
        pszServerName,
        101,
        core::ptr::addr_of_mut!(bufptr) as *mut *mut u8,
    );
    let mut hr: HRESULT = if error > 0 {
        HRESULT_FROM_WIN32(error as LONG)
    } else {
        error as HRESULT
    };
    if ((hr) < 0) {
        return hr;
    }

    let comment: PCWSTR = (*bufptr).sv101_comment;
    if !comment.is_null() && *comment.add(0 as usize) != 0 {
        StringCchCopyW(pszDesc, ((cchDescMax) as usize), comment);
    } else {
        hr = E_FAIL;
    }

    NetApiBufferFree(((bufptr) as _));
    hr
}
unsafe extern "C" fn SHELL_BuildDisplayMachineName(
    pszName: PWSTR,
    cchNameMax: u32,
    pszServerName: PCWSTR,
    pszDescription: PCWSTR,
) -> HRESULT {
    if pszDescription.is_null() || *pszDescription == 0 {
        return E_FAIL;
    }

    let pszFormat: PCWSTR = if SHRestricted(REST_ALLOWCOMMENTTOGGLE) != 0 {
        CPP_COM_WSTR_C1DBB4469138.as_ptr()
    } else {
        CPP_COM_WSTR_FB73E589D125.as_ptr()
    };

    let mut args: [PCWSTR; 2] = [pszDescription, SHELL_SkipServerSlashes(pszServerName)];

    if FormatMessageW(
        FORMAT_MESSAGE_ARGUMENT_ARRAY | FORMAT_MESSAGE_FROM_STRING,
        ((pszFormat) as _),
        0,
        0,
        pszName,
        cchNameMax,
        args.as_mut_ptr() as _,
    ) != 0
    {
        S_OK
    } else {
        E_FAIL
    }
}

#[no_mangle]
pub unsafe extern "system" fn SHGetComputerDisplayNameW(
    mut pszServerName: PWSTR,
    mut dwFlags: u32,
    pszName: PWSTR,
    cchNameMax: u32,
) -> HRESULT {
    let mut szDesc = [0u16; 256 as usize];
    let mut szCompName = [0u16; (MAX_COMPUTERNAME_LENGTH + 1) as usize];

    if pszServerName.is_null() {
        let mut cchCompName: u32 = szCompName.len() as u32;
        if GetComputerNameW(szCompName.as_mut_ptr(), &mut cchCompName) == 0 {
            return E_FAIL as HRESULT;
        }
        pszServerName = szCompName.as_mut_ptr();
        dwFlags |= SHGCDN_NOCACHE as u32;
    }

    let mut hr: HRESULT = E_FAIL as HRESULT;
    if (dwFlags & (SHGCDN_NOCACHE as u32)) == 0 {
        hr = SHELL_GetCachedComputerDescription(
            szDesc.as_mut_ptr(),
            szDesc.len() as u32,
            pszServerName as PCWSTR,
        );
    }

    if hr < (0 as HRESULT) {
        hr = SHELL_GetComputerDescription(
            szDesc.as_mut_ptr(),
            szDesc.len() as SIZE_T,
            pszServerName,
        );
        if hr < (0 as HRESULT) {
            szDesc[0 as usize] = UNICODE_NULL as u16;
        }

        if (dwFlags & (SHGCDN_NOCACHE as u32)) == 0 {
            SHELL_CacheComputerDescription(pszServerName as PCWSTR, szDesc.as_ptr());
        }
    }

    if hr < (0 as HRESULT) || szDesc[0 as usize] == 0 {
        if (dwFlags & (SHGCDN_NOSERVERNAME as u32)) != 0 {
            return hr;
        }

        StringCchCopyW(
            pszName,
            cchNameMax as usize,
            SHELL_SkipServerSlashes(pszServerName as PCWSTR),
        );
        return S_OK as HRESULT;
    }

    if (dwFlags & (SHGCDN_NOSERVERNAME as u32)) != 0 {
        StringCchCopyW(pszName, cchNameMax as usize, szDesc.as_ptr());
        return S_OK as HRESULT;
    }

    SHELL_BuildDisplayMachineName(pszName, cchNameMax, pszServerName as PCWSTR, szDesc.as_ptr())
}
#[no_mangle]
pub unsafe extern "C" fn SHELL32_ReparentAsAliasPidl(
    hwnd: HWND,
    hToken: HANDLE,
    pidlTarget: LPCITEMIDLIST,
    ppidlNew: *mut LPITEMIDLIST,
    dwFlags: DWORD,
) -> BOOL {
    if pidlTarget.is_null() || ppidlNew.is_null() {
        return FALSE;
    }

    (*ppidlNew) = core::ptr::null_mut();

    let mut iEntry = 0;
    while iEntry < g_AliasTable.len() {
        let pEntry = core::ptr::addr_of!(g_AliasTable[iEntry]);

        if (dwFlags & ((*pEntry).bFlagMask as DWORD)) == 0 {
            iEntry += 1;
            continue;
        }

        let mut pidlSrcRoot: LPITEMIDLIST = core::ptr::null_mut();
        let mut hr = SHGetFolderLocation(
            hwnd,
            (((*pEntry).nCsidlSrc) as i32),
            hToken,
            0,
            core::ptr::addr_of_mut!(pidlSrcRoot),
        );
        if hr < 0 {
            iEntry += 1;
            continue;
        }

        let pidlRelative: LPCITEMIDLIST = ((ILFindChild(pidlSrcRoot, pidlTarget)) as _);
        if pidlRelative.is_null() {
            ILFree(pidlSrcRoot);
            iEntry += 1;
            continue;
        }

        let mut pidlDestRoot: LPITEMIDLIST = core::ptr::null_mut();
        hr = SHGetFolderLocation(
            hwnd,
            (((*pEntry).nCsidlDest) as i32),
            hToken,
            0,
            core::ptr::addr_of_mut!(pidlDestRoot),
        );
        if hr >= 0 {
            (*ppidlNew) = ILCombine(pidlDestRoot, pidlRelative);
            if !(*ppidlNew).is_null() {
                if (*pEntry).bCommonDesktop != 0 && (**ppidlNew).mkid.cb >= 3 {
                    (**ppidlNew).mkid.abID[0] |= ((PT_FS | PT_FS_COMMON_FLAG) as u8);
                }
            }
            ILFree(pidlDestRoot);
        }

        ILFree(pidlSrcRoot);
        break;
    }

    return !(*ppidlNew).is_null() as BOOL;
}

#[no_mangle]
pub unsafe extern "C" fn SHELL32_AliasTranslatePidl(
    pidl: LPCITEMIDLIST,
    ppidlNew: *mut LPITEMIDLIST,
    dwFlags: DWORD,
) -> HRESULT {
    if SHELL32_ReparentAsAliasPidl(core::ptr::null_mut(), core::ptr::null_mut(), pidl, ppidlNew, dwFlags) != 0 {
        S_OK
    } else {
        E_FAIL
    }
}

#[no_mangle]
pub unsafe extern "system" fn BindCtx_GetMode_RustBridge(
    pbc: *mut IBindCtx,
    dwDefault: DWORD,
) -> DWORD {
    BindCtx_GetMode(pbc, dwDefault)
}

#[no_mangle]
pub unsafe extern "system" fn SHIsFileSysBindCtx_RustBridge(
    pBindCtx: *mut IBindCtx,
    pFindData: *mut WIN32_FIND_DATAW,
) -> HRESULT {
    SHIsFileSysBindCtx(pBindCtx, pFindData)
}

#[no_mangle]
pub unsafe extern "system" fn SHELL_ErrorBox_RustBridge(hwndOwner: HWND, Error: UINT) -> UINT {
    SHELL_ErrorBoxHelper(hwndOwner, Error)
}

#[no_mangle]
pub unsafe extern "system" fn DataObject_GetOffset_RustBridge(
    pDataObject: *mut IDataObject,
    point: *mut POINT,
) -> HRESULT {
    DataObject_GetOffset(pDataObject, point)
}

// CStubWindow32_CreateStub_RustBridge / _DestroyStub_RustBridge are NOT translated, and the
// two bodies rustconv produced for them were removed rather than repaired.
//
// They are the C++ side of a bridge that exists so Rust never needs CStubWindow32 at all.
// utils.cpp says why, above the C++ definitions: an ATL CWindowImpl's per-instance thunk and
// registered window class must have exactly ONE owner, and utils.cpp + CDrivesFolder.cpp still
// use the class directly -- so a Rust implementation would register the same class name and
// split FindStubWindow. Translating the bridge produces precisely the second implementation
// the bridge was built to prevent.
//
// The tell that it could never work: the island renders `CStubWindow32` as a layout-only
// struct (bindgen cannot emit C++ methods), so the model reached for a `CStubWindow32_new()`
// and a `core::boxed::CStubWindow32_delete()` that do not exist anywhere. It came out as a
// syntax error, which is the lucky version -- the same category error in a body with a
// plausible Rust spelling would have compiled.
//
// Rust CALLS these through an extern "C" declaration at the use site; it does not define them.
// utils.cpp therefore stays a hybrid TU until CStubWindow32's last C++ user is gone.

#[no_mangle]
pub unsafe extern "system" fn SHELL_CreateShell32DefaultExtractIcon_RustBridge(
    iconIndex: INT,
    riid: REFIID,
    ppvOut: *mut LPVOID,
) -> HRESULT {
    SHELL_CreateShell32DefaultExtractIcon(iconIndex, riid, ppvOut)
}

#[no_mangle]
pub unsafe extern "system" fn SHILAppend_RustBridge(
    pidl: LPITEMIDLIST,
    ppidl: *mut LPITEMIDLIST,
) -> HRESULT {
    SHILAppend(pidl, ppidl)
}

#[no_mangle]
pub unsafe extern "system" fn SHGetAttributes_RustBridge(
    psf: *mut IShellFolder,
    pidl: LPCITEMIDLIST,
    dwAttributes: DWORD,
) -> DWORD {
    SHGetAttributes(psf, pidl, dwAttributes)
}

#[no_mangle]
pub unsafe extern "system" fn GetDfmCmd_RustBridge(
    pCM: *mut IContextMenu,
    verba: LPCSTR,
) -> UINT {
    GetDfmCmd(pCM, verba)
}

#[no_mangle]
pub unsafe extern "system" fn PathIsDosDevice_RustBridge(pszName: LPCWSTR) -> BOOL {
    PathIsDosDevice(pszName)
}

#[no_mangle]
pub unsafe extern "system" fn BindCtx_ContainsObject_RustBridge(
    pbc: *mut IBindCtx,
    pszName: LPCWSTR,
) -> BOOL {
    BindCtx_ContainsObject(pbc, pszName)
}

#[no_mangle]
pub unsafe extern "system" fn SHSkipJunctionBinding_RustBridge(
    pbc: *mut IBindCtx,
    pclsid: *mut CLSID,
) -> BOOL {
    SHSkipJunctionBinding(pbc, pclsid)
}

#[no_mangle]
pub unsafe extern "system" fn Shell_FailForceReturn_RustBridge(hr: HRESULT) -> BOOL {
    Shell_FailForceReturn(hr)
}

#[no_mangle]
pub unsafe extern "system" fn SHGetNameAndFlagsW_RustBridge(
    pidl: LPCITEMIDLIST,
    dwFlags: DWORD,
    pszText: LPWSTR,
    cchBuf: UINT,
    pdwAttributes: *mut DWORD,
) -> HRESULT {
    SHGetNameAndFlagsW(pidl, dwFlags, pszText, cchBuf, pdwAttributes)
}

#[no_mangle]
pub unsafe extern "system" fn DCIA_AddShellExSubkey_RustBridge(
    hDCIA: HDCIA,
    hProgId: HKEY,
    pszSubkey: PCWSTR,
) {
    DCIA_AddShellExSubkey(hDCIA, hProgId, pszSubkey);
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     shell32
//  * LICENSE:     LGPL-2.1+ (https://spdx.org/licenses/LGPL-2.1+)
//  * PURPOSE:     Utility functions
//  * COPYRIGHT:   Copyright 2023-2024 Katayama Hirofumi MZ <katayama.hirofumi.mz@gmail.com>
//  */
//
// #include "precomp.h"
// #include <lmcons.h>
// #include <lmapibuf.h>
// #include <lmaccess.h>
// #include <lmserver.h>
// #include <secext.h>
//
// WINE_DEFAULT_DEBUG_CHANNEL(shell);
//
// static PCSTR StrEndNA(     PCSTR psz,      INT_PTR cch)
// {
//     PCSTR pch, pchEnd = &psz[cch];
//     for (pch = psz; *pch && pch < pchEnd; pch = CharNextA(pch))
//         ;
//     if (pchEnd < pch) // A double-byte character detected at last?
//         pch -= 2; // The width of a double-byte character is 2
//     return pch;
// }
// static PCWSTR StrEndNW(     PCWSTR psz,      INT_PTR cch)
// {
//     PCWSTR pch, pchEnd = &psz[cch];
//     for (pch = psz; *pch && pch < pchEnd; ++pch)
//         ;
//     return pch;
// }
// PSTR WINAPI
// StrRStrA(
//          PCSTR pszSrc,
//              PCSTR pszLast,
//          PCSTR pszSearch)
// {
//     INT cchSearch = lstrlenA(pszSearch);
//
//     PCSTR pchEnd = pszLast ? pszLast : &pszSrc[lstrlenA(pszSrc)];
//     if (pchEnd == pszSrc)
//         return NULL;
//
//     INT_PTR cchEnd = pchEnd - pszSrc;
//     for (;;)
//     {
//         --pchEnd;
//         --cchEnd;
//         if (!pchEnd)
//             break;
//         if (!StrCmpNA(pchEnd, pszSearch, cchSearch) && pchEnd == StrEndNA(pszSrc, cchEnd))
//             break;
//         if (pchEnd == pszSrc)
//             return NULL;
//     }
//
//     return const_cast<PSTR>(pchEnd);
// }
// PWSTR WINAPI
// StrRStrW(
//          PCWSTR pszSrc,
//              PCWSTR pszLast,
//          PCWSTR pszSearch)
// {
//     INT cchSearch = lstrlenW(pszSearch);
//
//     PCWSTR pchEnd = pszLast ? pszLast : &pszSrc[lstrlenW(pszSrc)];
//     if (pchEnd == pszSrc)
//         return NULL;
//
//     INT_PTR cchEnd = pchEnd - pszSrc;
//     for (;;)
//     {
//         --pchEnd;
//         --cchEnd;
//         if (!pchEnd)
//             break;
//         if (!StrCmpNW(pchEnd, pszSearch, cchSearch) && pchEnd == StrEndNW(pszSrc, cchEnd))
//             break;
//         if (pchEnd == pszSrc)
//             return NULL;
//     }
//
//     return const_cast<PWSTR>(pchEnd);
// }
// HRESULT
// SHILClone(
//              LPCITEMIDLIST pidl,
//              LPITEMIDLIST *ppidl)
// {
//     if (!pidl)
//     {
//         *ppidl = NULL;
//         return S_OK;
//     }
//     *ppidl = ILClone(pidl);
//     return (*ppidl ? S_OK : E_OUTOFMEMORY);
// }
// BOOL PathIsDotOrDotDotW(     LPCWSTR pszPath)
// {
//     if (pszPath[0] != L'.')
//         return FALSE;
//     return !pszPath[1] || (pszPath[1] == L'.' && !pszPath[2]);
// }
// BOOL PathIsValidElement(     LPCWSTR pszPath)
// {
//     if (!*pszPath || PathIsDotOrDotDotW(pszPath))
//         return FALSE;
//
//     for (LPCWSTR pch = pszPath; *pch; ++pch)
//     {
//         if (!PathIsValidCharW(*pch, PATH_VALID_ELEMENT))
//             return FALSE;
//     }
//
//     return TRUE;
// }
// BOOL PathIsDosDevice(     LPCWSTR pszName)
// {
//     WCHAR szPath[MAX_PATH];
//     StringCchCopyW(szPath, _countof(szPath), pszName);
//     PathRemoveExtensionW(szPath);
//
//     if (lstrcmpiW(szPath, L"NUL") == 0 || lstrcmpiW(szPath, L"PRN") == 0 ||
//         lstrcmpiW(szPath, L"CON") == 0 || lstrcmpiW(szPath, L"AUX") == 0)
//     {
//         return TRUE;
//     }
//
//     if (_wcsnicmp(szPath, L"LPT", 3) == 0 || _wcsnicmp(szPath, L"COM", 3) == 0)
//     {
//         if ((L'0' <= szPath[3] && szPath[3] <= L'9') && szPath[4] == UNICODE_NULL)
//             return TRUE;
//     }
//
//     return FALSE;
// }
// HRESULT SHILAppend(        LPITEMIDLIST pidl,         LPITEMIDLIST *ppidl)
// {
//     LPITEMIDLIST pidlOld = *ppidl;
//     if (!pidlOld)
//     {
//         *ppidl = pidl;
//         return S_OK;
//     }
//
//     HRESULT hr = SHILCombine(*ppidl, pidl, ppidl);
//     ILFree(pidlOld);
//     ILFree(pidl);
//     return hr;
// }
// HRESULT WINAPI
// SHShouldShowWizards(     IUnknown *pUnknown)
// {
//     HRESULT hr;
//     IShellBrowser *pBrowser;
//
//     hr = IUnknown_QueryService(pUnknown, SID_STopWindow, IID_PPV_ARG(IShellBrowser, &pBrowser));
//     if (FAILED(hr))
//         return hr;
//
//     SHELLSTATE state;
//     SHGetSetSettings(&state, SSF_WEBVIEW, FALSE);
//     if (state.fWebView &&
//         !SHRegGetBoolUSValueW(L"Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Advanced",
//                               L"ShowWizardsTEST", FALSE, FALSE))
//     {
//         hr = S_FALSE;
//     }
//
//     pBrowser->Release();
//     return hr;
// }
// static BOOL
// OpenEffectiveToken(
//          DWORD DesiredAccess,
//           HANDLE *phToken)
// {
//     BOOL ret;
//
//     if (phToken == NULL)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     *phToken = NULL;
//
//     ret = OpenThreadToken(GetCurrentThread(), DesiredAccess, FALSE, phToken);
//     if (!ret && GetLastError() == ERROR_NO_TOKEN)
//         ret = OpenProcessToken(GetCurrentProcess(), DesiredAccess, phToken);
//
//     return ret;
// }
// BOOL BindCtx_ContainsObject(     IBindCtx *pBindCtx,      LPCWSTR pszName)
// {
//     CComPtr<IUnknown> punk;
//     if (!pBindCtx || FAILED(pBindCtx->GetObjectParam(const_cast<LPWSTR>(pszName), &punk)))
//         return FALSE;
//     return TRUE;
// }
// DWORD BindCtx_GetMode(     IBindCtx *pbc,      DWORD dwDefault)
// {
//     if (!pbc)
//         return dwDefault;
//
//     BIND_OPTS BindOpts = { sizeof(BindOpts) };
//     HRESULT hr = pbc->GetBindOptions(&BindOpts);
//     if (FAILED(hr))
//         return dwDefault;
//
//     return BindOpts.grfMode;
// }
// BOOL SHSkipJunctionBinding(     IBindCtx *pbc,      CLSID *pclsid)
// {
//     if (!pbc)
//         return FALSE;
//
//     BIND_OPTS BindOps = { sizeof(BindOps) };
//     if (SUCCEEDED(pbc->GetBindOptions(&BindOps)) && BindOps.grfFlags == OLECONTF_LINKS)
//         return TRUE;
//
//     return pclsid && SHSkipJunction(pbc, pclsid);
// }
// HRESULT SHIsFileSysBindCtx(     IBindCtx *pBindCtx,           WIN32_FIND_DATAW *pFindData)
// {
//     CComPtr<IUnknown> punk;
//     CComPtr<IFileSystemBindData> pBindData;
//
//     if (!pBindCtx || FAILED(pBindCtx->GetObjectParam((LPWSTR)STR_FILE_SYS_BIND_DATA, &punk)))
//         return S_FALSE;
//
//     if (FAILED(punk->QueryInterface(IID_PPV_ARG(IFileSystemBindData, &pBindData))))
//         return S_FALSE;
//
//     if (pFindData)
//         pBindData->GetFindData(pFindData);
//
//     return S_OK;
// }
// BOOL Shell_FailForceReturn(     HRESULT hr)
// {
//     DWORD code = HRESULT_CODE(hr);
//
//     switch (code)
//     {
//         case ERROR_BAD_NETPATH:
//         case ERROR_BAD_NET_NAME:
//         case ERROR_CANCELLED:
//             return TRUE;
//
//         default:
//             return (ERROR_FILE_NOT_FOUND <= code && code <= ERROR_PATH_NOT_FOUND);
//     }
// }
// HRESULT
// SHBindToObjectEx(
//              IShellFolder *pShellFolder,
//          LPCITEMIDLIST pidl,
//              IBindCtx *pBindCtx,
//          REFIID riid,
//           void **ppvObj)
// {
//     CComPtr<IShellFolder> psfDesktop;
//
//     *ppvObj = NULL;
//
//     if (!pShellFolder)
//     {
//         SHGetDesktopFolder(&psfDesktop);
//         if (!psfDesktop)
//             return E_FAIL;
//
//         pShellFolder = psfDesktop;
//     }
//
//     HRESULT hr;
//     if (_ILIsDesktop(pidl))
//         hr = pShellFolder->QueryInterface(riid, ppvObj);
//     else
//         hr = pShellFolder->BindToObject(pidl, pBindCtx, riid, ppvObj);
//
//     if (SUCCEEDED(hr) && !*ppvObj)
//         hr = E_FAIL;
//
//     return hr;
// }
// HRESULT SHBindToObject(
//              IShellFolder *psf,
//          LPCITEMIDLIST pidl,
//          REFIID riid,
//           void **ppvObj)
// {
//     return SHBindToObjectEx(psf, pidl, NULL, riid, ppvObj);
// }
// extern "C" HRESULT
// SHELL_GetUIObjectOfAbsoluteItem(
//              HWND hWnd,
//          PCIDLIST_ABSOLUTE pidl,
//          REFIID riid,       void **ppvObj)
// {
//     if (!ppvObj)
//         return E_INVALIDARG;
//     *ppvObj = NULL;
//     IShellFolder *psf;
//     PCUITEMID_CHILD pidlChild;
//     HRESULT hr = SHBindToParent(pidl, IID_PPV_ARG(IShellFolder, &psf), &pidlChild);
//     if (SUCCEEDED(hr))
//     {
//         hr = psf->GetUIObjectOf(hWnd, 1, &pidlChild, riid, NULL, ppvObj);
//         psf->Release();
//         if (SUCCEEDED(hr))
//         {
//             if (*ppvObj)
//                 return hr;
//             hr = E_FAIL;
//         }
//     }
//     return hr;
// }
// HRESULT
// SHELL_DisplayNameOf(
//              IShellFolder *psf,
//          LPCITEMIDLIST pidl,
//              UINT Flags,
//           PWSTR *ppStr)
// {
//     HRESULT hr;
//     CComPtr<IShellFolder> psfRoot;
//     if (!psf)
//     {
//         PCUITEMID_CHILD pidlChild;
//         hr = SHBindToParent(pidl, IID_PPV_ARG(IShellFolder, &psfRoot), &pidlChild);
//         if (FAILED(hr))
//             return hr;
//         psf = psfRoot;
//         pidl = pidlChild;
//     }
//     STRRET sr;
//     hr = psf->GetDisplayNameOf((PCUITEMID_CHILD)pidl, Flags, &sr);
//     return SUCCEEDED(hr) ? StrRetToStrW(&sr, pidl, ppStr) : hr;
// }
// extern "C" HRESULT WINAPI
// DisplayNameOfW(
//          IShellFolder *psf,
//          LPCITEMIDLIST pidl,
//          DWORD dwFlags,
//           LPWSTR pszBuf,
//          UINT cchBuf)
// {
//     *pszBuf = UNICODE_NULL;
//     STRRET sr;
//     HRESULT hr = psf->GetDisplayNameOf(pidl, dwFlags, &sr);
//     if (FAILED(hr))
//         return hr;
//     return StrRetToBufW(&sr, pidl, pszBuf, cchBuf);
// }
// DWORD
// SHGetAttributes(     IShellFolder *psf,      LPCITEMIDLIST pidl,      DWORD dwAttributes)
// {
//     LPCITEMIDLIST pidlLast = pidl;
//     IShellFolder *release = NULL;
//
//     if (!psf)
//     {
//         SHBindToParent(pidl, IID_PPV_ARG(IShellFolder, &psf), &pidlLast);
//         if (!psf)
//             return 0;
//         release = psf;
//     }
//
//     DWORD oldAttrs = dwAttributes;
//     if (FAILED(psf->GetAttributesOf(1, &pidlLast, &dwAttributes)))
//         dwAttributes = 0;
//     else
//         dwAttributes &= oldAttrs;
//
//     if ((dwAttributes & SFGAO_FOLDER) &&
//         (dwAttributes & SFGAO_STREAM) &&
//         !(dwAttributes & SFGAO_STORAGEANCESTOR) &&
//         (oldAttrs & SFGAO_STORAGEANCESTOR) &&
//         (SHGetObjectCompatFlags(psf, NULL) & 0x200))
//     {
//         dwAttributes &= ~(SFGAO_STREAM | SFGAO_STORAGEANCESTOR);
//         dwAttributes |= SFGAO_STORAGEANCESTOR;
//     }
//
//     if (release)
//         release->Release();
//     return dwAttributes;
// }
// HRESULT SHELL_GetIDListTarget(     LPCITEMIDLIST pidl,       PIDLIST_ABSOLUTE *ppidl)
// {
//     IShellLink *pSL;
//     HRESULT hr = SHBindToObject(NULL, pidl, IID_PPV_ARG(IShellLink, &pSL));
//     if (SUCCEEDED(hr))
//     {
//         hr = pSL->GetIDList(ppidl); // Note: Returns S_FALSE if no target pidl
//         pSL->Release();
//     }
//     return hr;
// }
// HRESULT SHCoInitializeAnyApartment(VOID)
// {
//     HRESULT hr = CoInitializeEx(NULL, COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE);
//     if (FAILED(hr))
//         hr = CoInitializeEx(NULL, COINIT_DISABLE_OLE1DDE);
//     return hr;
// }
// HRESULT
// SHGetNameAndFlagsW(
//          LPCITEMIDLIST pidl,
//          DWORD dwFlags,
//               LPWSTR pszText,
//          UINT cchBuf,
//                 DWORD *pdwAttributes)
// {
//     if (pszText)
//         *pszText = UNICODE_NULL;
//
//     HRESULT hrCoInit = SHCoInitializeAnyApartment();
//
//     CComPtr<IShellFolder> psfFolder;
//     LPCITEMIDLIST ppidlLast;
//     HRESULT hr = SHBindToParent(pidl, IID_PPV_ARG(IShellFolder, &psfFolder), &ppidlLast);
//     if (SUCCEEDED(hr))
//     {
//         if (pszText)
//             hr = DisplayNameOfW(psfFolder, ppidlLast, dwFlags, pszText, cchBuf);
//
//         if (SUCCEEDED(hr))
//         {
//             if (pdwAttributes)
//                 *pdwAttributes = SHGetAttributes(psfFolder, ppidlLast, *pdwAttributes);
//         }
//     }
//
//     if (SUCCEEDED(hrCoInit))
//         CoUninitialize();
//
//     return hr;
// }
// extern "C" HRESULT WINAPI
// CShellLink_SHGetNameAndFlagsW_RustBridge(
//     LPCITEMIDLIST pidl, DWORD dwFlags, LPWSTR pszText, UINT cchBuf, DWORD *pdwAttributes)
// {
//     return SHGetNameAndFlagsW(pidl, dwFlags, pszText, cchBuf, pdwAttributes);
// }
// extern "C" HWND
// BindCtx_GetUIWindow(     IBindCtx *pBindCtx)
// {
//     HWND hWnd = NULL;
//
//     CComPtr<IUnknown> punk;
//     if (pBindCtx && SUCCEEDED(pBindCtx->GetObjectParam((LPWSTR)L"UI During Binding", &punk)))
//         IUnknown_GetWindow(punk, &hWnd);
//
//     return hWnd;
// }
// extern "C" HRESULT
// BindCtx_RegisterObjectParam(
//          IBindCtx *pBindCtx,
//          LPOLESTR pszKey,
//              IUnknown *punk,
//           LPBC *ppbc)
// {
//     HRESULT hr = S_OK;
//     CDummyOleWindow *pUnknown = NULL;
//
//     *ppbc = pBindCtx;
//
//     if (pBindCtx)
//     {
//         pBindCtx->AddRef();
//     }
//     else
//     {
//         hr = CreateBindCtx(0, ppbc);
//         if (FAILED(hr))
//             return hr;
//     }
//
//     if (!punk)
//         punk = pUnknown = new CDummyOleWindow();
//
//     hr = (*ppbc)->RegisterObjectParam(pszKey, punk);
//
//     if (pUnknown)
//         pUnknown->Release();
//
//     if (FAILED(hr))
//     {
//         (*ppbc)->Release();
//         *ppbc = NULL;
//     }
//
//     return hr;
// }
// HRESULT WINAPI
// SHSetFolderPathA(
//          INT csidl,
//          HANDLE hToken,
//          DWORD dwFlags,
//          LPCSTR pszPath)
// {
//     TRACE("(%d, %p, 0x%X, %s)\n", csidl, hToken, dwFlags, debugstr_a(pszPath));
//     CStringW strPathW(pszPath);
//     return SHSetFolderPathW(csidl, hToken, dwFlags, strPathW);
// }
// BOOL WINAPI
// PathIsSlowA(
//          LPCSTR pszFile,
//          DWORD dwAttr)
// {
//     TRACE("(%s, 0x%X)\n", debugstr_a(pszFile), dwAttr);
//     CStringW strFileW(pszFile);
//     return PathIsSlowW(strFileW, dwAttr);
// }
// WORD WINAPI
// ExtractIconResInfoA(
//          HANDLE hHandle,
//          LPCSTR lpFileName,
//          WORD wIndex,
//           LPWORD lpSize,
//           LPHANDLE lpIcon)
// {
//     TRACE("(%p, %s, %u, %p, %p)\n", hHandle, debugstr_a(lpFileName), wIndex, lpSize, lpIcon);
//
//     if (!lpFileName)
//         return 0;
//
//     CStringW strFileNameW(lpFileName);
//     return ExtractIconResInfoW(hHandle, strFileNameW, wIndex, lpSize, lpIcon);
// }
// LPWSTR WINAPI
// ShortSizeFormatW(
//          DWORD dwNumber,
//                          LPWSTR pszBuffer)
// {
//     TRACE("(%lu, %p)\n", dwNumber, pszBuffer);
//     return StrFormatByteSizeW(dwNumber, pszBuffer, 0x8FFF);
// }
// extern "C" BOOL WINAPI SHOpenEffectiveToken(      LPHANDLE phToken)
// {
//     TRACE("%p\n", phToken);
//     return OpenEffectiveToken(TOKEN_QUERY | TOKEN_ADJUST_PRIVILEGES, phToken);
// }
// extern "C" DWORD WINAPI SHGetUserSessionId(         HANDLE hToken)
// {
//     DWORD dwSessionId, dwLength;
//     BOOL bOpenToken = FALSE;
//
//     TRACE("%p\n", hToken);
//
//     if (!hToken)
//         bOpenToken = SHOpenEffectiveToken(&hToken);
//
//     if (!hToken ||
//         !GetTokenInformation(hToken, TokenSessionId, &dwSessionId, sizeof(dwSessionId), &dwLength))
//     {
//         dwSessionId = 0;
//     }
//
//     if (bOpenToken)
//         CloseHandle(hToken);
//
//     return dwSessionId;
// }
// HRESULT WINAPI
// SHInvokePrivilegedFunctionW(
//          LPCWSTR pszName,
//          PRIVILEGED_FUNCTION fn,
//              LPARAM lParam)
// {
//     TRACE("(%s %p %p)\n", debugstr_w(pszName), fn, lParam);
//
//     if (!pszName || !fn)
//         return E_INVALIDARG;
//
//     HANDLE hToken = NULL;
//     TOKEN_PRIVILEGES NewPriv, PrevPriv;
//     BOOL bAdjusted = FALSE;
//
//     if (SHOpenEffectiveToken(&hToken) &&
//         ::LookupPrivilegeValueW(NULL, pszName, &NewPriv.Privileges[0].Luid))
//     {
//         NewPriv.PrivilegeCount = 1;
//         NewPriv.Privileges[0].Attributes = SE_PRIVILEGE_ENABLED;
//
//         DWORD dwReturnSize;
//         bAdjusted = ::AdjustTokenPrivileges(hToken, FALSE, &NewPriv,
//                                             sizeof(PrevPriv), &PrevPriv, &dwReturnSize);
//     }
//
//     HRESULT hr = fn(lParam);
//
//     if (bAdjusted)
//         ::AdjustTokenPrivileges(hToken, FALSE, &PrevPriv, 0, NULL, NULL);
//
//     if (hToken)
//         ::CloseHandle(hToken);
//
//     return hr;
// }
// BOOL WINAPI
// SHTestTokenPrivilegeW(
//              HANDLE hToken,
//          LPCWSTR lpName)
// {
//     LUID Luid;
//     DWORD dwLength;
//     PTOKEN_PRIVILEGES pTokenPriv;
//     HANDLE hNewToken = NULL;
//     BOOL ret = FALSE;
//
//     TRACE("(%p, %s)\n", hToken, debugstr_w(lpName));
//
//     if (!lpName)
//         return FALSE;
//
//     if (!hToken)
//     {
//         if (!SHOpenEffectiveToken(&hNewToken))
//             goto Quit;
//
//         if (!hNewToken)
//             return FALSE;
//
//         hToken = hNewToken;
//     }
//
//     if (!LookupPrivilegeValueW(NULL, lpName, &Luid))
//         return FALSE;
//
//     dwLength = 0;
//     if (!GetTokenInformation(hToken, TokenPrivileges, NULL, 0, &dwLength))
//         goto Quit;
//
//     pTokenPriv = (PTOKEN_PRIVILEGES)LocalAlloc(LPTR, dwLength);
//     if (!pTokenPriv)
//         goto Quit;
//
//     if (GetTokenInformation(hToken, TokenPrivileges, pTokenPriv, dwLength, &dwLength))
//     {
//         UINT iPriv, cPrivs;
//         cPrivs = pTokenPriv->PrivilegeCount;
//         for (iPriv = 0; !ret && iPriv < cPrivs; ++iPriv)
//         {
//             ret = RtlEqualLuid(&Luid, &pTokenPriv->Privileges[iPriv].Luid);
//         }
//     }
//
//     LocalFree(pTokenPriv);
//
// Quit:
//     if (hToken == hNewToken)
//         CloseHandle(hNewToken);
//
//     return ret;
// }
// BOOL IsShutdownAllowed(VOID)
// {
//     return SHTestTokenPrivilegeW(NULL, SE_SHUTDOWN_NAME);
// }
// BOOL WINAPI IsSuspendAllowed(VOID)
// {
//     TRACE("()\n");
//     return IsShutdownAllowed() && IsPwrSuspendAllowed();
// }
// extern "C" HINSTANCE
// WINAPI
// SHGetShellStyleHInstance(VOID)
// {
//     HINSTANCE hInst = NULL;
//     WCHAR szPath[MAX_PATH], szColorName[100];
//     HRESULT hr;
//     CStringW strShellStyle;
//
//     TRACE("SHGetShellStyleHInstance called\n");
//
//     /* First, attempt to load the shellstyle dll from the current active theme */
//     hr = GetCurrentThemeName(szPath, _countof(szPath), szColorName, _countof(szColorName), NULL, 0);
//     if (FAILED(hr))
//         goto DoDefault;
//
//     /* Strip the theme filename */
//     PathRemoveFileSpecW(szPath);
//
//     strShellStyle = szPath;
//     strShellStyle += L"\\Shell\\";
//     strShellStyle += szColorName;
//     strShellStyle += L"\\ShellStyle.dll";
//
//     hInst = LoadLibraryExW(strShellStyle, NULL, LOAD_LIBRARY_AS_DATAFILE);
//     if (hInst)
//         return hInst;
//
//     /* Otherwise, use the version stored in the System32 directory */
// DoDefault:
//     if (!ExpandEnvironmentStringsW(L"%SystemRoot%\\System32\\ShellStyle.dll",
//                                    szPath, _countof(szPath)))
//     {
//         ERR("Expand failed\n");
//         return NULL;
//     }
//     return LoadLibraryExW(szPath, NULL, LOAD_LIBRARY_AS_DATAFILE);
// }
// extern "C" HRESULT
// WINAPI
// SHCreatePropertyBag(     REFIID riid,       void **ppvObj)
// {
//     return SHCreatePropertyBagOnMemory(STGM_READWRITE, riid, ppvObj);
// }
// static DWORD
// SHELL_ReadSingleUnreadMailCount(
//          HKEY hKey,
//               PDWORD pdwCount,
//               PFILETIME pFileTime,
//                                              LPWSTR pszShellExecuteCommand,
//          INT cchShellExecuteCommand)
// {
//     DWORD dwType, dwCount, cbSize = sizeof(dwCount);
//     DWORD error = SHQueryValueExW(hKey, L"MessageCount", 0, &dwType, &dwCount, &cbSize);
//     if (error)
//         return error;
//     if (pdwCount && dwType == REG_DWORD)
//         *pdwCount = dwCount;
//
//     FILETIME FileTime;
//     cbSize = sizeof(FileTime);
//     error = SHQueryValueExW(hKey, L"TimeStamp", 0, &dwType, &FileTime, &cbSize);
//     if (error)
//         return error;
//     if (pFileTime && dwType == REG_BINARY)
//         *pFileTime = FileTime;
//
//     WCHAR szName[2 * MAX_PATH];
//     cbSize = sizeof(szName);
//     error = SHQueryValueExW(hKey, L"Application", 0, &dwType, szName, &cbSize);
//     if (error)
//         return error;
//
//     if (pszShellExecuteCommand && dwType == REG_SZ &&
//         FAILED(StringCchCopyW(pszShellExecuteCommand, cchShellExecuteCommand, szName)))
//     {
//         return ERROR_INSUFFICIENT_BUFFER;
//     }
//
//     return ERROR_SUCCESS;
// }
// HRESULT WINAPI
// SHGetUnreadMailCountW(
//              HKEY hKeyUser,
//              PCWSTR pszMailAddress,
//               PDWORD pdwCount,
//                 PFILETIME pFileTime,
//                                              PWSTR pszShellExecuteCommand,
//          INT cchShellExecuteCommand)
// {
//     LSTATUS error;
//     HKEY hKey;
//
//     if (!hKeyUser)
//         hKeyUser = HKEY_CURRENT_USER;
//
//     if (pszMailAddress)
//     {
//         CStringW strKey = L"Software\\Microsoft\\Windows\\CurrentVersion\\UnreadMail";
//         strKey += L'\\';
//         strKey += pszMailAddress;
//
//         error = RegOpenKeyExW(hKeyUser, strKey, 0, KEY_QUERY_VALUE, &hKey);
//         if (error)
//             return HRESULT_FROM_WIN32(error);
//
//         error = SHELL_ReadSingleUnreadMailCount(hKey, pdwCount, pFileTime,
//                                                 pszShellExecuteCommand, cchShellExecuteCommand);
//     }
//     else
//     {
//         if (pszShellExecuteCommand || cchShellExecuteCommand)
//             return E_INVALIDARG;
//
//         *pdwCount = 0;
//
//         error = RegOpenKeyExW(hKeyUser, L"Software\\Microsoft\\Windows\\CurrentVersion\\UnreadMail",
//                               0, KEY_ENUMERATE_SUB_KEYS, &hKey);
//         if (error)
//             return HRESULT_FROM_WIN32(error);
//
//         for (DWORD dwIndex = 0; !error; ++dwIndex)
//         {
//             WCHAR Name[2 * MAX_PATH];
//             DWORD cchName = _countof(Name);
//             FILETIME LastWritten;
//             error = RegEnumKeyExW(hKey, dwIndex, Name, &cchName, NULL, NULL, NULL, &LastWritten);
//             if (error)
//                 break;
//
//             HKEY hSubKey;
//             error = RegOpenKeyExW(hKey, Name, 0, KEY_QUERY_VALUE, &hSubKey);
//             if (error)
//                 break;
//
//             FILETIME FileTime;
//             DWORD dwCount;
//             error = SHELL_ReadSingleUnreadMailCount(hSubKey, &dwCount, &FileTime, NULL, 0);
//             if (!error && (!pFileTime || CompareFileTime(&FileTime, pFileTime) >= 0))
//                 *pdwCount += dwCount;
//
//             RegCloseKey(hSubKey);
//         }
//
//         if (error == ERROR_NO_MORE_ITEMS)
//             error = ERROR_SUCCESS;
//     }
//
//     RegCloseKey(hKey);
//
//     return error ? HRESULT_FROM_WIN32(error) : S_OK;
// }
// HRESULT WINAPI
// SHSetUnreadMailCountW(
//          PCWSTR pszMailAddress,
//          DWORD dwCount,
//          PCWSTR pszShellExecuteCommand)
// {
//     CString strKey = L"Software\\Microsoft\\Windows\\CurrentVersion\\UnreadMail\\";
//     strKey += pszMailAddress;
//
//     HKEY hKey;
//     DWORD dwDisposition;
//     LSTATUS error = RegCreateKeyExW(HKEY_CURRENT_USER, strKey, 0, NULL, 0, KEY_SET_VALUE, NULL,
//                                     &hKey, &dwDisposition);
//     if (error)
//         return HRESULT_FROM_WIN32(error);
//
//     error = RegSetValueExW(hKey, L"MessageCount", 0, REG_DWORD, (PBYTE)&dwCount, sizeof(dwCount));
//     if (error)
//     {
//         RegCloseKey(hKey);
//         return HRESULT_FROM_WIN32(error);
//     }
//
//     FILETIME FileTime;
//     GetSystemTimeAsFileTime(&FileTime);
//
//     error = RegSetValueExW(hKey, L"TimeStamp", 0, REG_BINARY, (PBYTE)&FileTime, sizeof(FileTime));
//     if (error)
//     {
//         RegCloseKey(hKey);
//         return HRESULT_FROM_WIN32(error);
//     }
//
//     WCHAR szBuff[2 * MAX_PATH];
//     if (!PathUnExpandEnvStringsW(pszShellExecuteCommand, szBuff, _countof(szBuff)))
//     {
//         HRESULT hr = StringCchCopyW(szBuff, _countof(szBuff), pszShellExecuteCommand);
//         if (FAILED_UNEXPECTEDLY(hr))
//         {
//             RegCloseKey(hKey);
//             return hr;
//         }
//     }
//
//     DWORD cbValue = (lstrlenW(szBuff) + 1) * sizeof(WCHAR);
//     error = RegSetValueExW(hKey, L"Application", 0, REG_SZ, (PBYTE)szBuff, cbValue);
//
//     RegCloseKey(hKey);
//     return (error ? HRESULT_FROM_WIN32(error) : S_OK);
// }
// extern "C" LPSTR
// WINAPI
// SheRemoveQuotesA(LPSTR psz)
// {
//     PCHAR pch;
//
//     if (*psz == '"')
//     {
//         for (pch = psz + 1; *pch && *pch != '"'; ++pch)
//         {
//             *(pch - 1) = *pch;
//         }
//
//         if (*pch == '"')
//             *(pch - 1) = ANSI_NULL;
//     }
//
//     return psz;
// }
// extern "C" LPWSTR
// WINAPI
// SheRemoveQuotesW(LPWSTR psz)
// {
//     PWCHAR pch;
//
//     if (*psz == L'"')
//     {
//         for (pch = psz + 1; *pch && *pch != L'"'; ++pch)
//         {
//             *(pch - 1) = *pch;
//         }
//
//         if (*pch == L'"')
//             *(pch - 1) = UNICODE_NULL;
//     }
//
//     return psz;
// }
// HRESULT WINAPI
// SHEnumerateUnreadMailAccountsW(
//              HKEY hKeyUser,
//          DWORD dwIndex,
//                                  PWSTR pszMailAddress,
//          INT cchMailAddress)
// {
//     if (!hKeyUser)
//         hKeyUser = HKEY_CURRENT_USER;
//
//     HKEY hKey;
//     LSTATUS error = RegOpenKeyExW(hKeyUser,
//                                   L"Software\\Microsoft\\Windows\\CurrentVersion\\UnreadMail",
//                                   0, KEY_ENUMERATE_SUB_KEYS, &hKey);
//     if (error)
//         return HRESULT_FROM_WIN32(error);
//
//     FILETIME FileTime;
//     error = RegEnumKeyExW(hKey, dwIndex, pszMailAddress, (PDWORD)&cchMailAddress, NULL, NULL,
//                           NULL, &FileTime);
//     if (error)
//         *pszMailAddress = UNICODE_NULL;
//
//     RegCloseKey(hKey);
//     return error ? HRESULT_FROM_WIN32(error) : S_OK;
// }
// extern "C" BOOL
// WINAPI
// SHFindComputer(LPCITEMIDLIST pidlRoot, LPCITEMIDLIST pidlSavedSearch)
// {
//     UNREFERENCED_PARAMETER(pidlRoot);
//     UNREFERENCED_PARAMETER(pidlSavedSearch);
//
//     TRACE("%p %p\n", pidlRoot, pidlSavedSearch);
//
//     IContextMenu *pCM;
//     HRESULT hr = CoCreateInstance(CLSID_ShellSearchExt, NULL, CLSCTX_INPROC_SERVER,
//                                   IID_IContextMenu, (void **)&pCM);
//     if (FAILED(hr))
//     {
//         ERR("0x%08X\n", hr);
//         return hr;
//     }
//
//     CMINVOKECOMMANDINFO InvokeInfo = { sizeof(InvokeInfo) };
//     InvokeInfo.lpParameters = "{996E1EB1-B524-11D1-9120-00A0C98BA67D}";
//     InvokeInfo.nShow = SW_SHOWNORMAL;
//     hr = pCM->InvokeCommand(&InvokeInfo);
//     pCM->Release();
//
//     return SUCCEEDED(hr);
// }
// static HRESULT
// Int64ToStr(
//          LONGLONG llValue,
//                            LPWSTR pszValue,
//          UINT cchValue)
// {
//     WCHAR szBuff[40];
//     UINT ich = 0, ichValue;
//     BOOL bMinus = (llValue < 0);
//
//     if (bMinus)
//         llValue = -llValue;
//
//     if (cchValue <= 0)
//         return E_FAIL;
//
//     do
//     {
//         szBuff[ich++] = (WCHAR)(L'0' + (llValue % 10));
//         llValue /= 10;
//     } while (llValue != 0 && ich < _countof(szBuff) - 1);
//
//     if (bMinus && ich < _countof(szBuff))
//         szBuff[ich++] = '-';
//
//     for (ichValue = 0; ich > 0 && ichValue < cchValue; ++ichValue)
//     {
//         --ich;
//         pszValue[ichValue] = szBuff[ich];
//     }
//
//     if (ichValue >= cchValue)
//     {
//         pszValue[cchValue - 1] = UNICODE_NULL;
//         return E_FAIL;
//     }
//
//     pszValue[ichValue] = UNICODE_NULL;
//     return S_OK;
// }
// static VOID
// Int64GetNumFormat(
//           NUMBERFMTW *pDest,
//              const NUMBERFMTW *pSrc,
//          DWORD dwNumberFlags,
//                              LPWSTR pszDecimal,
//          INT cchDecimal,
//                               LPWSTR pszThousand,
//          INT cchThousand)
// {
//     WCHAR szBuff[20];
//
//     if (pSrc)
//         *pDest = *pSrc;
//     else
//         dwNumberFlags = 0;
//
//     if (!(dwNumberFlags & FMT_USE_NUMDIGITS))
//     {
//         GetLocaleInfoW(LOCALE_USER_DEFAULT, LOCALE_IDIGITS, szBuff, _countof(szBuff));
//         pDest->NumDigits = StrToIntW(szBuff);
//     }
//
//     if (!(dwNumberFlags & FMT_USE_LEADZERO))
//     {
//         GetLocaleInfoW(LOCALE_USER_DEFAULT, LOCALE_ILZERO, szBuff, _countof(szBuff));
//         pDest->LeadingZero = StrToIntW(szBuff);
//     }
//
//     if (!(dwNumberFlags & FMT_USE_GROUPING))
//     {
//         GetLocaleInfoW(LOCALE_USER_DEFAULT, LOCALE_SGROUPING, szBuff, _countof(szBuff));
//         pDest->Grouping = StrToIntW(szBuff);
//     }
//
//     if (!(dwNumberFlags & FMT_USE_DECIMAL))
//     {
//         GetLocaleInfoW(LOCALE_USER_DEFAULT, LOCALE_SDECIMAL, pszDecimal, cchDecimal);
//         pDest->lpDecimalSep = pszDecimal;
//     }
//
//     if (!(dwNumberFlags & FMT_USE_THOUSAND))
//     {
//         GetLocaleInfoW(LOCALE_USER_DEFAULT, LOCALE_STHOUSAND, pszThousand, cchThousand);
//         pDest->lpThousandSep = pszThousand;
//     }
//
//     if (!(dwNumberFlags & FMT_USE_NEGNUMBER))
//     {
//         GetLocaleInfoW(LOCALE_USER_DEFAULT, LOCALE_INEGNUMBER, szBuff, _countof(szBuff));
//         pDest->NegativeOrder = StrToIntW(szBuff);
//     }
// }
// INT WINAPI
// Int64ToString(
//          LONGLONG llValue,
//                          LPWSTR pszOut,
//          UINT cchOut,
//          BOOL bUseFormat,
//              const NUMBERFMTW *pNumberFormat,
//          DWORD dwNumberFlags)
// {
//     INT ret;
//     NUMBERFMTW NumFormat;
//     WCHAR szValue[80], szDecimalSep[6], szThousandSep[6];
//
//     Int64ToStr(llValue, szValue, _countof(szValue));
//
//     if (bUseFormat)
//     {
//         Int64GetNumFormat(&NumFormat, pNumberFormat, dwNumberFlags,
//                           szDecimalSep, _countof(szDecimalSep),
//                           szThousandSep, _countof(szThousandSep));
//         ret = GetNumberFormatW(LOCALE_USER_DEFAULT, 0, szValue, &NumFormat, pszOut, cchOut);
//         if (ret)
//             --ret;
//         return ret;
//     }
//
//     if (FAILED(StringCchCopyW(pszOut, cchOut, szValue)))
//         return 0;
//
//     return lstrlenW(pszOut);
// }
// INT WINAPI
// LargeIntegerToString(
//          const LARGE_INTEGER *pLargeInt,
//                          LPWSTR pszOut,
//          UINT cchOut,
//          BOOL bUseFormat,
//              const NUMBERFMTW *pNumberFormat,
//          DWORD dwNumberFlags)
// {
//     return Int64ToString(pLargeInt->QuadPart, pszOut, cchOut, bUseFormat,
//                          pNumberFormat, dwNumberFlags);
// }
// HRESULT WINAPI
// CopyStreamUI(
//          IStream *pSrc,
//           IStream *pDst,
//                 IProgressDialog *pProgress,
//              DWORDLONG dwlSize)
// {
//     HRESULT hr = E_FAIL;
//     DWORD cbBuff, cbRead, dwSizeToWrite;
//     DWORDLONG cbDone;
//     LPVOID pBuff;
//     CComHeapPtr<BYTE> pHeapPtr;
//     STATSTG Stat;
//     BYTE abBuff[1024];
//
//     TRACE("(%p, %p, %p, %I64u)\n", pSrc, pDst, pProgress, dwlSize);
//
//     if (dwlSize == 0) // Invalid size?
//     {
//         // Get the stream size
//         ZeroMemory(&Stat, sizeof(Stat));
//         if (FAILED(pSrc->Stat(&Stat, STATFLAG_NONAME)))
//             pProgress = NULL; // No size info. Disable progress
//         else
//             dwlSize = Stat.cbSize.QuadPart;
//     }
//
//     if (!pProgress) // Progress is disabled?
//     {
//         ULARGE_INTEGER uliSize;
//
//         if (dwlSize > 0)
//             uliSize.QuadPart = dwlSize;
//         else
//             uliSize.HighPart = uliSize.LowPart = INVALID_FILE_SIZE;
//
//         return pSrc->CopyTo(pDst, uliSize, NULL, NULL); // One punch
//     }
//
//     // Allocate the buffer if necessary
//     if (dwlSize > 0 && dwlSize <= sizeof(abBuff))
//     {
//         cbBuff = sizeof(abBuff);
//         pBuff = abBuff;
//     }
//     else
//     {
// #define COPY_STREAM_DEFAULT_BUFFER_SIZE 0x4000
//         cbBuff = COPY_STREAM_DEFAULT_BUFFER_SIZE;
//         if (pHeapPtr.AllocateBytes(cbBuff))
//         {
//             pBuff = pHeapPtr;
//         }
//         else // Low memory?
//         {
//             cbBuff = sizeof(abBuff);
//             pBuff = abBuff;
//         }
// #undef COPY_STREAM_DEFAULT_BUFFER_SIZE
//     }
//
//     // Start reading
//     LARGE_INTEGER zero;
//     zero.QuadPart = 0;
//     pSrc->Seek(zero, 0, NULL);
//     pDst->Seek(zero, 0, NULL);
//     cbDone = 0;
//     pProgress->SetProgress64(cbDone, dwlSize);
//
//     // Repeat reading and writing until goal
//     for (;;)
//     {
//         hr = pSrc->Read(pBuff, cbBuff, &cbRead);
//         if (FAILED(hr))
//             break;
//
//         // Calculate the size to write
//         if (dwlSize > 0)
//             dwSizeToWrite = (DWORD)min((DWORDLONG)(dwlSize - cbDone), (DWORDLONG)cbRead);
//         else
//             dwSizeToWrite = cbRead;
//
//         if (dwSizeToWrite == 0) // No need to write?
//         {
//             hr = S_OK;
//             break;
//         }
//
//         hr = pDst->Write(pBuff, dwSizeToWrite, NULL);
//         if (hr != S_OK)
//             break;
//
//         cbDone += dwSizeToWrite;
//
//         if (pProgress->HasUserCancelled()) // Cancelled?
//         {
//             hr = HRESULT_FROM_WIN32(ERROR_CANCELLED);
//             break;
//         }
//         pProgress->SetProgress64(cbDone, dwlSize);
//
//         if (dwlSize > 0 && cbDone >= dwlSize) // Reached the goal?
//         {
//             hr = S_OK;
//             break;
//         }
//     }
//
//     return hr;
// }
// BOOL WINAPI
// Activate_RunDLL(
//          HWND hwnd,
//          HINSTANCE hinst,
//          LPCWSTR cmdline,
//          INT cmdshow)
// {
//     DWORD dwProcessID;
//
//     UNREFERENCED_PARAMETER(hwnd);
//     UNREFERENCED_PARAMETER(hinst);
//     UNREFERENCED_PARAMETER(cmdline);
//     UNREFERENCED_PARAMETER(cmdshow);
//
//     TRACE("(%p, %p, %s, %d)\n", hwnd, hinst, debugstr_w(cmdline), cmdline);
//
//     GetWindowThreadProcessId(GetShellWindow(), &dwProcessID);
//     return AllowSetForegroundWindow(dwProcessID);
// }
// HRESULT WINAPI
// SHStartNetConnectionDialogA(
//          HWND hwnd,
//          LPCSTR pszRemoteName,
//          DWORD dwType)
// {
//     LPCWSTR pszRemoteNameW = NULL;
//     CStringW strRemoteNameW;
//
//     TRACE("(%p, %s, %lu)\n", hwnd, debugstr_a(pszRemoteName), dwType);
//
//     if (pszRemoteName)
//     {
//         strRemoteNameW = pszRemoteName;
//         pszRemoteNameW = strRemoteNameW;
//     }
//
//     return SHStartNetConnectionDialogW(hwnd, pszRemoteNameW, dwType);
// }
// static INT
// DynamicPathCommonPrefixW(
//          LPCWSTR lpszPath1,
//          LPCWSTR lpszPath2,
//           CStringW& strPath)
// {
//     SIZE_T cchPath1 = wcslen(lpszPath1);
//     SIZE_T cchPath2 = wcslen(lpszPath2);
//     LPWSTR lpszPath = strPath.GetBuffer((INT)max(cchPath1, cchPath2) + 16);
//     INT ret = PathCommonPrefixW(lpszPath1, lpszPath2, lpszPath);
//     strPath.ReleaseBuffer();
//     return ret;
// }
// static HRESULT
// DynamicSHGetPathFromIDListW(
//          LPCITEMIDLIST pidl,
//           CStringW& strPath)
// {
//     HRESULT hr;
//
//     for (UINT cchPath = MAX_PATH;; cchPath *= 2)
//     {
//         LPWSTR lpszPath = strPath.GetBuffer(cchPath);
//         if (!lpszPath)
//             return E_OUTOFMEMORY;
//
//         hr = SHGetPathCchFromIDListW(pidl, lpszPath, cchPath);
//         strPath.ReleaseBuffer();
//
//         if (hr != E_NOT_SUFFICIENT_BUFFER)
//             break;
//
//         if (cchPath >= MAXUINT / 2)
//         {
//             hr = E_FAIL;
//             break;
//         }
//     }
//
//     if (FAILED(hr))
//         strPath.Empty();
//
//     return hr;
// }
// static HRESULT
// DynamicSHGetSpecialFolderPathW(
//          HWND hwndOwner,
//           CStringW& strPath,
//          INT nCSIDL,
//          BOOL bCreate)
// {
//     LPITEMIDLIST pidl;
//     HRESULT hr = SHGetSpecialFolderLocation(hwndOwner, nCSIDL, &pidl);
//     if (SUCCEEDED(hr))
//     {
//         hr = DynamicSHGetPathFromIDListW(pidl, strPath);
//         CoTaskMemFree(pidl);
//     }
//
//     if (FAILED(hr))
//         strPath.Empty();
//     else if (bCreate)
//         CreateDirectoryW(strPath, NULL);
//
//     return hr;
// }
// static VOID
// DynamicPathRemoveBackslashW(
//           CStringW& strPath)
// {
//     INT nLength = strPath.GetLength();
//     if (nLength > 0 && strPath[nLength - 1] == L'\\')
//         strPath = strPath.Left(nLength - 1);
// }
// BOOL WINAPI
// PathIsEqualOrSubFolder(
//          LPCWSTR pszPath1OrCSIDL,
//          LPCWSTR pszPath2)
// {
//     CStringW strCommon, strPath1;
//
//     TRACE("(%s %s)\n", debugstr_w(pszPath1OrCSIDL), debugstr_w(pszPath2));
//
//     if (IS_INTRESOURCE(pszPath1OrCSIDL))
//     {
//         DynamicSHGetSpecialFolderPathW(
//             NULL, strPath1, LOWORD(pszPath1OrCSIDL) | CSIDL_FLAG_DONT_VERIFY, FALSE);
//     }
//     else
//     {
//         strPath1 = pszPath1OrCSIDL;
//     }
//
//     DynamicPathRemoveBackslashW(strPath1);
//
//     if (!DynamicPathCommonPrefixW(strPath1, pszPath2, strCommon))
//         return FALSE;
//
//     return strPath1.CompareNoCase(strCommon) == 0;
// }
// HRESULT WINAPI
// SHGetRealIDL(
//          IShellFolder *psf,
//          PCUITEMID_CHILD pidlSimple,
//              PITEMID_CHILD *ppidlReal)
// {
//     HRESULT hr;
//     STRRET strret;
//     WCHAR szPath[MAX_PATH];
//     SFGAOF attrs;
//
//     *ppidlReal = NULL;
//
//     hr = IShellFolder_GetDisplayNameOf(psf, pidlSimple, SHGDN_INFOLDER | SHGDN_FORPARSING,
//                                        &strret, 0);
//     if (FAILED_UNEXPECTEDLY(hr))
//         return hr;
//
//     hr = StrRetToBufW(&strret, pidlSimple, szPath, _countof(szPath));
//     if (FAILED_UNEXPECTEDLY(hr))
//         return hr;
//
//     attrs = SFGAO_FILESYSTEM;
//     hr = psf->GetAttributesOf(1, &pidlSimple, &attrs);
//     if (SUCCEEDED(hr) && !(attrs & SFGAO_FILESYSTEM))
//         return SHILClone(pidlSimple, ppidlReal);
//
//     hr = IShellFolder_ParseDisplayName(psf, NULL, NULL, szPath, NULL, ppidlReal, NULL);
//     if (hr == E_INVALIDARG || hr == E_NOTIMPL)
//         return SHILClone(pidlSimple, ppidlReal);
//
//     return hr;
// }
// extern "C" HRESULT
// IUnknown_InitializeCommand(
//          IUnknown *pUnk,
//          PCWSTR pszCommandName,
//              IPropertyBag *pPB)
// {
//     HRESULT hr;
//     CComPtr<IInitializeCommand> pIC;
//     if (SUCCEEDED(hr = pUnk->QueryInterface(IID_PPV_ARG(IInitializeCommand, &pIC))))
//         hr = pIC->Initialize(pszCommandName, pPB);
//     return hr;
// }
// extern "C" HRESULT
// InvokeIExecuteCommand(
//          IExecuteCommand *pEC,
//          PCWSTR pszCommandName,
//              IPropertyBag *pPB,
//              IShellItemArray *pSIA,
//              LPCMINVOKECOMMANDINFOEX pICI,
//              IUnknown *pSite)
// {
//     if (!pEC)
//         return E_INVALIDARG;
//
//     CScopedSetObjectWithSite site(pEC, pSite);
//     IUnknown_InitializeCommand(pEC, pszCommandName, pPB);
//
//     CComPtr<IObjectWithSelection> pOWS;
//     if (pSIA && SUCCEEDED(pEC->QueryInterface(IID_PPV_ARG(IObjectWithSelection, &pOWS))))
//         pOWS->SetSelection(pSIA);
//
//     DWORD dwKeyState = 0, fMask = pICI ? pICI->fMask : 0;
//     pEC->SetNoShowUI((fMask & CMIC_MASK_FLAG_NO_UI) != 0);
//     pEC->SetShowWindow(pICI ? pICI->nShow : SW_SHOW);
//     if (fMask & CMIC_MASK_SHIFT_DOWN)
//         dwKeyState |= MK_SHIFT;
//     if (fMask & CMIC_MASK_CONTROL_DOWN)
//         dwKeyState |= MK_CONTROL;
//     pEC->SetKeyState(dwKeyState);
//     if ((fMask & CMIC_MASK_UNICODE) && pICI->lpDirectoryW)
//         pEC->SetDirectory(pICI->lpDirectoryW);
//     if ((fMask & CMIC_MASK_UNICODE) && pICI->lpParametersW)
//         pEC->SetParameters(pICI->lpParametersW);
//     if (fMask & CMIC_MASK_PTINVOKE)
//         pEC->SetPosition(pICI->ptInvoke);
//
//     return pEC->Execute();
// }
// extern "C" HRESULT
// InvokeIExecuteCommandWithDataObject(
//          IExecuteCommand *pEC,
//          PCWSTR pszCommandName,
//              IPropertyBag *pPB,
//          IDataObject *pDO,
//              LPCMINVOKECOMMANDINFOEX pICI,
//              IUnknown *pSite)
// {
//     CComPtr<IShellItemArray> pSIA;
//     HRESULT hr = SHCreateShellItemArrayFromDataObject(pDO, IID_PPV_ARG(IShellItemArray, &pSIA));
//     return SUCCEEDED(hr) ? InvokeIExecuteCommand(pEC, pszCommandName, pPB, pSIA, pICI, pSite) : hr;
// }
// static HRESULT
// GetCommandStringA(     IContextMenu *pCM,      UINT_PTR Id,      UINT GCS,                      LPSTR Buf,      UINT cchMax)
// {
//     HRESULT hr = pCM->GetCommandString(Id, GCS & ~GCS_UNICODE, NULL, Buf, cchMax);
//     if (FAILED(hr))
//     {
//         WCHAR buf[MAX_PATH];
//         hr = pCM->GetCommandString(Id, GCS | GCS_UNICODE, NULL, (LPSTR)buf, _countof(buf));
//         if (SUCCEEDED(hr))
//             hr = SHUnicodeToAnsi(buf, Buf, cchMax) > 0 ? S_OK : E_FAIL;
//     }
//     return hr;
// }
// UINT
// GetDfmCmd(     IContextMenu *pCM,      LPCSTR verba)
// {
//     CHAR buf[MAX_PATH];
//     if (IS_INTRESOURCE(verba))
//     {
//         if (FAILED(GetCommandStringA(pCM, LOWORD(verba), GCS_VERB, buf, _countof(buf))))
//             return 0;
//         verba = buf;
//     }
//     return MapVerbToDfmCmd(verba); // Returns DFM_CMD_* or 0
// }
// HRESULT
// SHELL_MapContextMenuVerbToCmdId(LPCMINVOKECOMMANDINFO pICI, const CMVERBMAP *pMap)
// {
//     LPCSTR pVerbA = pICI->lpVerb;
//     CHAR buf[MAX_PATH];
//     LPCMINVOKECOMMANDINFOEX pICIX = (LPCMINVOKECOMMANDINFOEX)pICI;
//     if (IsUnicode(*pICIX) && !IS_INTRESOURCE(pICIX->lpVerbW))
//     {
//         if (SHUnicodeToAnsi(pICIX->lpVerbW, buf, _countof(buf)))
//             pVerbA = buf;
//     }
//
//     if (IS_INTRESOURCE(pVerbA))
//         return LOWORD(pVerbA);
//     for (SIZE_T i = 0; pMap[i].Verb; ++i)
//     {
//         assert(SUCCEEDED((int)(pMap[i].CmdId))); // The id must be >= 0 and ideally in the 0..0x7fff range
//         if (!lstrcmpiA(pMap[i].Verb, pVerbA) && pVerbA[0])
//             return pMap[i].CmdId;
//     }
//     return E_FAIL;
// }
// static const CMVERBMAP*
// FindVerbMapEntry(UINT_PTR CmdId, const CMVERBMAP *pMap)
// {
//     for (SIZE_T i = 0; pMap[i].Verb; ++i)
//     {
//         if (pMap[i].CmdId == CmdId)
//             return &pMap[i];
//     }
//     return NULL;
// }
// HRESULT
// SHELL_GetCommandStringImpl(SIZE_T CmdId, UINT uFlags, LPSTR Buf, UINT cchBuf, const CMVERBMAP *pMap)
// {
//     const CMVERBMAP* pEntry;
//     switch (uFlags | GCS_UNICODE)
//     {
//         case GCS_VALIDATEW:
//         case GCS_VERBW:
//             pEntry = FindVerbMapEntry(CmdId, pMap);
//             if ((uFlags | GCS_UNICODE) == GCS_VERBW)
//             {
//                 if (!pEntry)
//                     return E_INVALIDARG;
//                 else if (uFlags & GCS_UNICODE)
//                     return SHAnsiToUnicode(pEntry->Verb, (LPWSTR)Buf, cchBuf) ? S_OK : E_FAIL;
//                 else
//                     return StringCchCopyA(Buf, cchBuf, pEntry->Verb);
//             }
//             return pEntry ? S_OK : S_FALSE; // GCS_VALIDATE
//     }
//     return E_NOTIMPL;
// }
// HRESULT
// SHELL_CreateShell32DefaultExtractIcon(int IconIndex, REFIID riid, LPVOID *ppvOut)
// {
//     CComPtr<IDefaultExtractIconInit> initIcon;
//     HRESULT hr = SHCreateDefaultExtractIcon(IID_PPV_ARG(IDefaultExtractIconInit, &initIcon));
//     if (FAILED_UNEXPECTEDLY(hr))
//         return hr;
//     initIcon->SetNormalIcon(swShell32Name, IconIndex);
//     return initIcon->QueryInterface(riid, ppvOut);
// }
// int DCIA_AddEntry(HDCIA hDCIA, REFCLSID rClsId)
// {
//     for (UINT i = 0;; ++i)
//     {
//         const CLSID *pClsId = DCIA_GetEntry(hDCIA, i);
//         if (!pClsId)
//             break;
//         if (IsEqualGUID(*pClsId, rClsId))
//             return i; // Don't allow duplicates
//     }
//     return DSA_AppendItem((HDSA)hDCIA, const_cast<CLSID*>(&rClsId));
// }
// void DCIA_AddShellExSubkey(HDCIA hDCIA, HKEY hProgId, PCWSTR pszSubkey)
// {
//     WCHAR szKey[200];
//     PathCombineW(szKey, L"shellex", pszSubkey);
//     HKEY hEnum;
//     if (RegOpenKeyExW(hProgId, szKey, 0, KEY_READ, &hEnum) != ERROR_SUCCESS)
//         return;
//     for (UINT i = 0; RegEnumKeyW(hEnum, i++, szKey, _countof(szKey)) == ERROR_SUCCESS;)
//     {
//         CLSID clsid;
//         if (SUCCEEDED(SHELL_GetShellExtensionRegCLSID(hEnum, szKey, &clsid)))
//             DCIA_AddEntry(hDCIA, clsid);
//     }
//     RegCloseKey(hEnum);
// }
// BOOL WINAPI
// SHIsBadInterfacePtr(
//          LPCVOID pv,
//          UINT_PTR ucb)
// {
//     struct CUnknownVtbl
//     {
//         HRESULT (STDMETHODCALLTYPE *QueryInterface)(REFIID riid, LPVOID *ppvObj);
//         ULONG (STDMETHODCALLTYPE *AddRef)();
//         ULONG (STDMETHODCALLTYPE *Release)();
//     };
//     struct CUnknown { CUnknownVtbl *lpVtbl; };
//     const CUnknown *punk = reinterpret_cast<const CUnknown *>(pv);
//     return !punk || IsBadReadPtr(punk, sizeof(punk->lpVtbl)) ||
//            IsBadReadPtr(punk->lpVtbl, ucb) ||
//            IsBadCodePtr((FARPROC)punk->lpVtbl->Release);
// }
// HRESULT WINAPI
// SHGetUserDisplayName(
//                                       PWSTR pName,
//             PULONG puSize)
// {
//     if (!pName || !puSize)
//         return E_INVALIDARG;
//
//     if (GetUserNameExW(NameDisplay, pName, puSize))
//         return S_OK;
//
//     LONG error = GetLastError(); // for ERROR_NONE_MAPPED
//     HRESULT hr = HRESULT_FROM_WIN32(error);
//
//     WCHAR UserName[MAX_PATH];
//     DWORD cchUserName = _countof(UserName);
//     if (!GetUserNameW(UserName, &cchUserName))
//         return HRESULT_FROM_WIN32(GetLastError());
//
//     // Was the user name not available in the specified format (NameDisplay)?
//     if (error == ERROR_NONE_MAPPED)
//     {
//         // Try to get the user name by using Network API
//         PUSER_INFO_2 UserInfo;
//         DWORD NetError = NetUserGetInfo(NULL, UserName, 2, (PBYTE*)&UserInfo);
//         if (NetError)
//         {
//             hr = HRESULT_FROM_WIN32(NetError);
//         }
//         else
//         {
//             if (UserInfo->usri2_full_name)
//             {
//                 hr = StringCchCopyW(pName, *puSize, UserInfo->usri2_full_name);
//                 if (SUCCEEDED(hr))
//                 {
//                     // Include the NUL-terminator
//                     *puSize = lstrlenW(UserInfo->usri2_full_name) + 1;
//                 }
//             }
//
//             NetApiBufferFree(UserInfo);
//         }
//     }
//
//     if (FAILED(hr))
//     {
//         hr = StringCchCopyW(pName, *puSize, UserName);
//         if (SUCCEEDED(hr))
//             *puSize = cchUserName;
//     }
//
//     return hr;
// }
// static PCWSTR
// SHELL_SkipServerSlashes(
//          PCWSTR pszPath)
// {
//     PCWSTR pch;
//     for (pch = pszPath; *pch == L'\\'; ++pch)
//         ;
//     return pch;
// }
// static HRESULT
// SHELL_GetCachedComputerDescription(
//                                PWSTR pszDesc,
//          DWORD cchDescMax,
//          PCWSTR pszServerName)
// {
//     cchDescMax *= sizeof(WCHAR);
//     DWORD error = SHGetValueW(HKEY_CURRENT_USER, COMPUTER_DESCRIPTIONS_KEY,
//                               SHELL_SkipServerSlashes(pszServerName), NULL, pszDesc, &cchDescMax);
//     return HRESULT_FROM_WIN32(error);
// }
// static VOID
// SHELL_CacheComputerDescription(
//          PCWSTR pszServerName,
//          PCWSTR pszDesc)
// {
//     if (!pszDesc)
//         return;
//
//     SIZE_T cbDesc = (wcslen(pszDesc) + 1) * sizeof(WCHAR);
//     SHSetValueW(HKEY_CURRENT_USER, COMPUTER_DESCRIPTIONS_KEY,
//                 SHELL_SkipServerSlashes(pszServerName), REG_SZ, pszDesc, (DWORD)cbDesc);
// }
// static HRESULT
// SHELL_GetComputerDescription(
//                                PWSTR pszDesc,
//          SIZE_T cchDescMax,
//          PWSTR pszServerName)
// {
//     PSERVER_INFO_101 bufptr;
//     NET_API_STATUS error = NetServerGetInfo(pszServerName, 101, (PBYTE*)&bufptr);
//     HRESULT hr = (error > 0) ? HRESULT_FROM_WIN32(error) : error;
//     if (FAILED_UNEXPECTEDLY(hr))
//         return hr;
//
//     PCWSTR comment = bufptr->sv101_comment;
//     if (comment && comment[0])
//         StringCchCopyW(pszDesc, cchDescMax, comment);
//     else
//         hr = E_FAIL;
//
//     NetApiBufferFree(bufptr);
//     return hr;
// }
// static HRESULT
// SHELL_BuildDisplayMachineName(
//                                PWSTR pszName,
//          DWORD cchNameMax,
//          PCWSTR pszServerName,
//          PCWSTR pszDescription)
// {
//     if (!pszDescription || !*pszDescription)
//         return E_FAIL;
//
//     PCWSTR pszFormat = (SHRestricted(REST_ALLOWCOMMENTTOGGLE) ? L"%2 (%1)" : L"%1 (%2)");
//     PCWSTR args[] = { pszDescription , SHELL_SkipServerSlashes(pszServerName) };
//     return (FormatMessageW(FORMAT_MESSAGE_ARGUMENT_ARRAY | FORMAT_MESSAGE_FROM_STRING,
//                            pszFormat, 0, 0, pszName, cchNameMax, (va_list *)args) ? S_OK : E_FAIL);
// }
// HRESULT WINAPI
// SHGetComputerDisplayNameW(
//              PWSTR pszServerName,
//          DWORD dwFlags,
//                                PWSTR pszName,
//          DWORD cchNameMax)
// {
//     WCHAR szDesc[256], szCompName[MAX_COMPUTERNAME_LENGTH + 1];
//
//     // If no server name is specified, retrieve the local computer name
//     if (!pszServerName)
//     {
//         // Use computer name as server name
//         DWORD cchCompName = _countof(szCompName);
//         if (!GetComputerNameW(szCompName, &cchCompName))
//             return E_FAIL;
//         pszServerName = szCompName;
//
//         // Don't use the cache for the local machine
//         dwFlags |= SHGCDN_NOCACHE;
//     }
//
//     // Get computer description from cache if necessary
//     HRESULT hr = E_FAIL;
//     if (!(dwFlags & SHGCDN_NOCACHE))
//         hr = SHELL_GetCachedComputerDescription(szDesc, _countof(szDesc), pszServerName);
//
//     // Actually retrieve the computer description if it is not in the cache
//     if (FAILED(hr))
//     {
//         hr = SHELL_GetComputerDescription(szDesc, _countof(szDesc), pszServerName);
//         if (FAILED(hr))
//             szDesc[0] = UNICODE_NULL;
//
//         // Cache the description if necessary
//         if (!(dwFlags & SHGCDN_NOCACHE))
//             SHELL_CacheComputerDescription(pszServerName, szDesc);
//     }
//
//     // If getting the computer description failed, store the server name only
//     if (FAILED(hr) || !szDesc[0])
//     {
//         if (dwFlags & SHGCDN_NOSERVERNAME)
//             return hr; // Bail out if no server name is requested
//
//         StringCchCopyW(pszName, cchNameMax, SHELL_SkipServerSlashes(pszServerName));
//         return S_OK;
//     }
//
//     // If no server name is requested, store the description only
//     if (dwFlags & SHGCDN_NOSERVERNAME)
//     {
//         StringCchCopyW(pszName, cchNameMax, szDesc);
//         return S_OK;
//     }
//
//     // Build a string like "Description (SERVERNAME)"
//     return SHELL_BuildDisplayMachineName(pszName, cchNameMax, pszServerName, szDesc);
// }
// extern "C" BOOL
// SHELL32_ReparentAsAliasPidl(
//              HWND hwnd,
//              HANDLE hToken,
//          LPCITEMIDLIST pidlTarget,
//           LPITEMIDLIST *ppidlNew,
//          DWORD dwFlags)
// {
//     if (!pidlTarget || !ppidlNew)
//         return FALSE;
//
//     *ppidlNew = NULL;
//
//     for (SIZE_T iEntry = 0; iEntry < _countof(g_AliasTable); ++iEntry)
//     {
//         const ALIAS_MAPPING *pEntry = &g_AliasTable[iEntry];
//
//         if (!(dwFlags & pEntry->bFlagMask))
//             continue;
//
//         // Get the source root PIDL
//         LPITEMIDLIST pidlSrcRoot = NULL;
//         HRESULT hr = SHGetFolderLocation(hwnd, pEntry->nCsidlSrc, hToken, 0, &pidlSrcRoot);
//         if (FAILED(hr))
//             continue;
//
//         // Check whether the input pidlTarget is under the source folder.
//         // If it matches, ILFindChild returns the relative PIDL in the pidlTarget.
//         LPCITEMIDLIST pidlRelative = ILFindChild(pidlSrcRoot, pidlTarget);
//         if (!pidlRelative) // Not found?
//         {
//             ILFree(pidlSrcRoot);
//             continue;
//         }
//
//         // Found. Get the destination root PIDL
//         LPITEMIDLIST pidlDestRoot = NULL;
//         hr = SHGetFolderLocation(hwnd, pEntry->nCsidlDest, hToken, 0, &pidlDestRoot);
//         if (SUCCEEDED(hr))
//         {
//             // Create a new PIDL by combining the destination root PIDL and the relative PIDL
//             *ppidlNew = ILCombine(pidlDestRoot, pidlRelative);
//             if (*ppidlNew)
//             {
//                 // Manipulate specific flags in the PIDL if necessary
//                 if (pEntry->bCommonDesktop && (*ppidlNew)->mkid.cb >= 3)
//                 {
//                     (*ppidlNew)->mkid.abID[0] |= (PT_FS | PT_FS_COMMON_FLAG);
//                 }
//             }
//             ILFree(pidlDestRoot);
//         }
//
//         ILFree(pidlSrcRoot);
//         break; // A match was found, so exit the loop
//     }
//
//     return (*ppidlNew != NULL);
// }
// extern "C" HRESULT
// SHELL32_AliasTranslatePidl(
//          LPCITEMIDLIST pidl,
//           LPITEMIDLIST *ppidlNew,
//          DWORD dwFlags)
// {
//     return SHELL32_ReparentAsAliasPidl(NULL, NULL, pidl, ppidlNew, dwFlags) ? S_OK : E_FAIL;
// }
// extern "C" DWORD WINAPI
// BindCtx_GetMode_RustBridge(IBindCtx *pbc, DWORD dwDefault)
// {
//     return BindCtx_GetMode(pbc, dwDefault);
// }
// extern "C" HRESULT WINAPI
// SHIsFileSysBindCtx_RustBridge(IBindCtx *pBindCtx, WIN32_FIND_DATAW *pFindData)
// {
//     return SHIsFileSysBindCtx(pBindCtx, pFindData);
// }
// extern "C" UINT WINAPI
// SHELL_ErrorBox_RustBridge(HWND hwndOwner, UINT Error)
// {
//     return SHELL_ErrorBoxHelper(hwndOwner, Error);
// }
// extern "C" HRESULT WINAPI
// DataObject_GetOffset_RustBridge(IDataObject *pDataObject, POINT *point)
// {
//     return DataObject_GetOffset(pDataObject, point);
// }
// extern "C" HRESULT WINAPI
// CStubWindow32_CreateStub_RustBridge(
//     UINT Type, LPCWSTR Path, const POINT *pPt, HWND *phWnd, void **ppStub)
// {
//     *phWnd = NULL;
//     *ppStub = NULL;
//     CStubWindow32 *pStub = new CStubWindow32();
//     if (!pStub)
//         return E_OUTOFMEMORY;
//     HRESULT hr = pStub->CreateStub(Type, Path, pPt);
//     if (FAILED(hr))
//     {
//         delete pStub;
//         return hr;
//     }
//     *phWnd = *pStub;
//     *ppStub = pStub;
//     return hr;
// }
// extern "C" void WINAPI
// CStubWindow32_DestroyStub_RustBridge(void *pStub)
// {
//     CStubWindow32 *p = static_cast<CStubWindow32 *>(pStub);
//     if (!p)
//         return;
//     if (p->m_hWnd)
//         p->DestroyWindow();
//     delete p;
// }
// extern "C" HRESULT WINAPI
// SHELL_CreateShell32DefaultExtractIcon_RustBridge(INT iconIndex, REFIID riid, LPVOID *ppvOut)
// {
//     return SHELL_CreateShell32DefaultExtractIcon(iconIndex, riid, ppvOut);
// }
// extern "C" HRESULT WINAPI
// SHILAppend_RustBridge(LPITEMIDLIST pidl, LPITEMIDLIST *ppidl)
// {
//     return SHILAppend(pidl, ppidl);
// }
// extern "C" DWORD WINAPI
// SHGetAttributes_RustBridge(IShellFolder *psf, LPCITEMIDLIST pidl, DWORD dwAttributes)
// {
//     return SHGetAttributes(psf, pidl, dwAttributes);
// }
// extern "C" UINT WINAPI
// GetDfmCmd_RustBridge(IContextMenu *pCM, LPCSTR verba)
// {
//     return GetDfmCmd(pCM, verba);
// }
// extern "C" BOOL WINAPI
// PathIsDosDevice_RustBridge(LPCWSTR pszName)
// {
//     return PathIsDosDevice(pszName);
// }
// extern "C" BOOL WINAPI
// BindCtx_ContainsObject_RustBridge(IBindCtx *pbc, LPCWSTR pszName)
// {
//     return BindCtx_ContainsObject(pbc, pszName);
// }
// extern "C" BOOL WINAPI
// SHSkipJunctionBinding_RustBridge(IBindCtx *pbc, CLSID *pclsid)
// {
//     return SHSkipJunctionBinding(pbc, pclsid);
// }
// extern "C" BOOL WINAPI
// Shell_FailForceReturn_RustBridge(HRESULT hr)
// {
//     return Shell_FailForceReturn(hr);
// }
// extern "C" HRESULT WINAPI
// SHGetNameAndFlagsW_RustBridge(LPCITEMIDLIST pidl, DWORD dwFlags, LPWSTR pszText, UINT cchBuf, DWORD *pdwAttributes)
// {
//     return SHGetNameAndFlagsW(pidl, dwFlags, pszText, cchBuf, pdwAttributes);
// }
// extern "C" void WINAPI
// DCIA_AddShellExSubkey_RustBridge(HDCIA hDCIA, HKEY hProgId, PCWSTR pszSubkey)
// {
//     DCIA_AddShellExSubkey(hDCIA, hProgId, pszSubkey);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
