//! Island-first substrate for translated TUs — glob this INSTEAD of `types`/`win32`.
//!
//! WHY "instead" and not "as well". Globbing `crate::island::*` alongside the hand-written
//! `crate::types::*` / `crate::win32::*` makes every name both define ambiguous (E0659):
//! measured on the first two Wave-5 leaves, the two globs together produced 296 ambiguity
//! errors across 54 names where either glob alone produced none. The island is the strictly
//! larger and header-derived authority (2,337 types / 11,580 constants / 1,151 structs /
//! 3,406 fn declarations, versus 105 / 173 / 89 / 218 hand-written), so it wins outright and
//! the hand-written modules contribute only the names it has no opinion about — re-exported
//! below one at a time, because an explicit import beats a glob and so cannot go ambiguous.
//!
//! Substituting one for the other is safe, and that is a measured claim rather than a hope:
//! all 311 definitions the two share were compared field-by-field and value-by-value, and
//! there is not one wrong constant value, arity mismatch, or width/pointer-class change
//! between them. The 156 textual differences are all spelling — alias-vs-primitive
//! (`DWORD`/`u32`), `Option<fn>`-vs-raw-fn-pointer, PIDL type-safety erasure
//! (`PIDLIST_ABSOLUTE` collapsed to `LPITEMIDLIST`), and the COM boundary below.
//! Net effect on the first two leaves: unresolved names 228 -> 77.
//!
//! THE BOUNDARY. The island owns DATA LAYOUTS; `shellcom` owns VTABLE ABI. bindgen renders a
//! shobjidl interface as a C++ class with a `vtable_` field, but the translation uses the C
//! `lpVtbl` idiom that the 40 landed COM-class modules are written against, so the interfaces
//! are blocklisted out of the island and re-exported from `shellcom` (see the `shell32` cell in
//! tools/rustconv/gen_driver_types.sh).
//!
//! C MACROS STAY CALLABLE AS WRITTEN. The translation emits `ZeroMemory(p, n)`, not
//! `ZeroMemory!(p, n)`, so the accessor and A/W-mapping macros appear here as `#[inline] fn`,
//! generic over the pointee where C's macro was type-agnostic. That is not a violation of the
//! standing rule against lowering a C macro to a fn — that rule is about DEBUG NO-OPS, where an
//! out-of-line empty body is an optimization barrier that uncloaks latent ABI mismatches. These
//! produce values and are `#[inline]`. Only the two whose second argument is a FIELD NAME or an
//! array TYPE — `SHELL_GetSetting` and `_countof` — stay `macro_rules!`, and their call sites
//! carry the `!`.
//!
//! THE RECURRING TRAP, stated once because it cost most of the debugging here: two aliases of
//! the SAME WIDTH are still two different Rust types. `shellcom::LPARAM` (`isize`) and the
//! island's (`i32` on i386) mean the same thing and occupy the same bytes, and mixing them
//! makes every island declaration reject the value. So a TU takes its scalars from ONE
//! authority — the island — and anything imported from a hand-written module comes with the
//! types in its signature, or is re-declared here against island types instead.

// The header-derived authority: struct layouts, constants, and the Win32/shell fn surface.
pub use crate::island::*;

// NOTHING scalar or PIDL-shaped is re-exported from `shellcom` here, and that is deliberate.
// Cherry-picking an alias out of the hand-written substrate into an island-first module
// creates a NOMINAL type split that the widths hide: `shellcom::LPARAM` is `isize` (via
// `types::LONG_PTR`) while the island's is `i32` on i386 (via `LONG_PTR = c_long`). Same
// width, same meaning, different Rust type — so every one of the island's 3,406 fn
// declarations rejects the value with `expected i32, found isize`, and the numeric-repair
// pass then wants to bake a per-arch width at each call. The island owns the scalars because
// its own declarations are written against them. Only the COM interfaces, which the island
// blocklists in favour of the `lpVtbl` shape, come from `shellcom` — and they are imported by
// the TUs that need them, not blanket-exported here.

// Accessor macros from `win32.rs`, but ONLY the ones whose signatures are type-neutral:
// `ZeroMemory<T>`/`CopyMemory<T,U>` are generic over the pointee, and `LOWORD`/`HIWORD` take a
// plain `DWORD`. Anything typed on `HWND`, `LPWSTR` or `LPARAM` is re-defined below against the
// island instead — `win32.rs` is written against `crate::types`, and importing one of those
// would re-open exactly the nominal split the module header warns about.
pub use crate::win32::{CopyMemory, HIWORD, LOWORD, ZeroMemory};
// Crate-root `#[macro_export]` macros — text substitution, so no types to split on.
pub use crate::{FIELD_OFFSET, TRACE_ON, UNREFERENCED_PARAMETER, _countof};

// shell32-internal helpers that already have a Rust home. These are shell32's own EXPORTS —
// the island correctly omits them, because `BLOCKLIST_FUNCS_FROM_CRATE` drops every function
// the crate defines itself, and importing the crate's own definition is the point.
pub use crate::dialogs_view::PostCabinetMessage;
pub use crate::wine::shpolicy::SHRestricted;

extern "C" {
    /// precomp.h `HPROPSHEETPAGE SH_CreatePropertySheetPage(WORD, DLGPROC, LPARAM, LPCWSTR)`.
    ///
    /// Declared, not imported, for the same reason as the block below: `propsheet.rs` types it
    /// against `crate::types`, whose `DLGPROC` is `extern "system" fn(*mut c_void, ..)` while a
    /// dialog proc written against the island is `extern "stdcall" fn(*mut HWND__, ..)` on i386.
    /// Those are the same ABI and the same machine code, but two different Rust types, so the
    /// call cannot type-check across the boundary. Declaring against the island's `DLGPROC`
    /// puts both sides on one spelling.
    pub fn SH_CreatePropertySheetPage(
        wDialogId: crate::island::WORD,
        pfnDlgProc: DLGPROC,
        lParam: crate::island::LPARAM,
        pwszTitle: crate::island::LPCWSTR,
    ) -> crate::island::HPROPSHEETPAGE;
}

/// `DLGPROC`, spelled `extern "system"` on EVERY arch.
///
/// bindgen writes the literal convention it saw, so `ntdll-sys` says `extern "stdcall"` on i386
/// and `extern "C"` on amd64/arm64 — three spellings of one ABI, and a dialog proc can only be
/// written to match one of them, so a TU ends up carrying a `#[cfg]`-duplicated copy of the
/// whole procedure per arch. (recycler_prop.rs did, and the two copies had already drifted
/// apart under separate repair passes.) `extern "system"` IS stdcall on i386-windows and IS the
/// C convention on 64-bit, so one spelling is correct everywhere and one copy of the procedure
/// suffices. Same reasoning as `tools/rustconv/normalize_island_abi.py`, which normalizes a
/// generated island's callback slots; this alias is the hand-written equivalent for a type that
/// comes from `ntdll-sys` rather than from shell32's own island.
pub type DLGPROC = ::core::option::Option<
    unsafe extern "system" fn(
        crate::island::HWND,
        crate::island::UINT,
        crate::island::WPARAM,
        crate::island::LPARAM,
    ) -> crate::island::INT_PTR,
>;

// SHELLSTATE/CABINETSTATE come from the ISLAND (above), so a TU gets bindgen's real bitfield
// accessors — `ss.fShowAllObjects()` — instead of hand-masking a `w0: u32` word.
//
// That forces these three to be DECLARED rather than imported. `wine/shellord.rs` implements
// them, but against its OWN copies of those structs, and its copies carry hand-rolled `g0`/`s0`
// bit helpers instead of accessors. Importing its Rust `fn` would drag its struct back in and
// split every caller nominally (`expected shellord::SHELLSTATE, found types_arch::SHELLSTATE`),
// while importing the island's struct and calling its `fn` cannot type-check at all. Since all
// three are `#[no_mangle] extern "system"` EXPORTS, declaring them here is what the C does —
// include the header, call the symbol — and it types them against the header truth. The island
// omits them only because `BLOCKLIST_FUNCS_FROM_CRATE` drops every function the crate defines.
// Layout equivalence is not assumed: both structs were compared field-by-field against the
// island before this was written.
extern "system" {
    /// shlobj.h:1583 `VOID WINAPI SHGetSetSettings(LPSHELLSTATE lpss, DWORD dwMask, BOOL bSet);`
    pub fn SHGetSetSettings(
        lpss: *mut crate::island::SHELLSTATE,
        dwMask: crate::island::DWORD,
        bSet: crate::island::BOOL,
    );
    /// shlobj.h:2411 `BOOL WINAPI ReadCabinetState(CABINETSTATE *, int cLength);`
    pub fn ReadCabinetState(
        cs: *mut crate::island::CABINETSTATE,
        cLength: ::core::ffi::c_int,
    ) -> crate::island::BOOL;
    /// shlobj.h:2415 `BOOL WINAPI WriteCabinetState(CABINETSTATE *);`
    pub fn WriteCabinetState(cs: *mut crate::island::CABINETSTATE) -> crate::island::BOOL;
}

// ---------------------------------------------------------------------------
// The TCHAR family, for shell32's UNICODE build.
//
// NOT cosmetic, and not something the island can fix. `ntdll-sys` is generated WITHOUT
// `UNICODE`, so it says `TCHAR = c_char` and `LPTSTR = LPSTR`; the island's preamble globs
// `ntdll_sys::nt::*`, so those ANSI spellings reach every TU that uses the island. shell32 is
// `set_module_type(shell32 win32dll UNICODE)` (CMakeLists.txt:199) and its `.cpp` were written
// against the wide forms, so a `TCHAR` buffer sized from the leaked alias is HALF the bytes it
// should be and an `LPTSTR` walks bytes where the callee walks wide chars. These explicit
// definitions beat the glob and put the whole family back on the UNICODE side.
// ---------------------------------------------------------------------------
pub type TCHAR = crate::island::WCHAR;
pub type LPTSTR = crate::island::LPWSTR;
pub type PTSTR = crate::island::LPWSTR;
pub type LPCTSTR = crate::island::LPCWSTR;
pub type PCTSTR = crate::island::LPCWSTR;

/// `TRUE`/`FALSE` are `#define`s of bare `1`/`0`, so bindgen types them `u32`. Every use in
/// the shell is as a `BOOL` (a SIGNED int), and the C `#define`s carry no type of their own —
/// so the island's spelling is an artifact of the macro, not of the API. Explicit beats glob.
pub const TRUE: crate::island::BOOL = 1;
pub const FALSE: crate::island::BOOL = 0;

// NOTE: no private `use crate::island::{...}` in this module. A private import SHADOWS the
// `pub use crate::island::*` glob above for the names it mentions and is itself not
// re-exported, so every consumer of this prelude loses them — that silently un-resolved
// HWND/UINT/WPARAM/LPCWSTR at 22 sites. Spell island paths out in full below instead.

// ---------------------------------------------------------------------------
// A/W mapping. `SendMessage` is not a function — it is `#define SendMessage SendMessageW`
// under UNICODE, which every ReactOS TU builds with. Same for the rest of this block.
// ---------------------------------------------------------------------------

/// `#define SendMessage SendMessageW` (winuser.h).
#[inline]
pub unsafe fn SendMessage(
    hWnd: crate::island::HWND,
    Msg: crate::island::UINT,
    wParam: crate::island::WPARAM,
    lParam: crate::island::LPARAM,
) -> crate::island::LRESULT {
    crate::island::SendMessageW(hWnd, Msg, wParam, lParam)
}
/// `#define PostMessage PostMessageW` (winuser.h).
#[inline]
pub unsafe fn PostMessage(
    hWnd: crate::island::HWND,
    Msg: crate::island::UINT,
    wParam: crate::island::WPARAM,
    lParam: crate::island::LPARAM,
) -> crate::island::BOOL {
    crate::island::PostMessageW(hWnd, Msg, wParam, lParam)
}
/// `#define SendDlgItemMessage SendDlgItemMessageW` (winuser.h).
#[inline]
pub unsafe fn SendDlgItemMessage(
    hDlg: crate::island::HWND,
    nIDDlgItem: ::core::ffi::c_int,
    Msg: crate::island::UINT,
    wParam: crate::island::WPARAM,
    lParam: crate::island::LPARAM,
) -> crate::island::LRESULT {
    crate::island::SendDlgItemMessageW(hDlg, nIDDlgItem, Msg, wParam, lParam)
}
/// `#define LoadImage LoadImageW` (winuser.h).
#[inline]
pub unsafe fn LoadImage(
    hInst: crate::island::HINSTANCE,
    name: crate::island::LPCWSTR,
    type_: crate::island::UINT,
    cx: ::core::ffi::c_int,
    cy: ::core::ffi::c_int,
    fuLoad: crate::island::UINT,
) -> crate::island::HANDLE {
    crate::island::LoadImageW(hInst, name, type_, cx, cy, fuLoad)
}
/// `#define SetWindowLongPtr SetWindowLongPtrW` (winuser.h).
///
/// On 32-bit there IS no `SetWindowLongPtrW` export — winuser.h defines it as a macro for
/// `SetWindowLongW`, whose value parameter is a plain `LONG`. Only the 64-bit builds have the
/// real entry point, which is why the island carries it for amd64/arm64 and not for i386.
#[cfg(target_pointer_width = "32")]
#[inline]
pub unsafe fn SetWindowLongPtr(
    hWnd: crate::island::HWND,
    nIndex: ::core::ffi::c_int,
    dwNewLong: crate::island::LONG_PTR,
) -> crate::island::LONG_PTR {
    crate::island::SetWindowLongW(hWnd, nIndex, dwNewLong as crate::island::LONG)
        as crate::island::LONG_PTR
}
#[cfg(target_pointer_width = "64")]
#[inline]
pub unsafe fn SetWindowLongPtr(
    hWnd: crate::island::HWND,
    nIndex: ::core::ffi::c_int,
    dwNewLong: crate::island::LONG_PTR,
) -> crate::island::LONG_PTR {
    crate::island::SetWindowLongPtrW(hWnd, nIndex, dwNewLong)
}
/// `#define SHRegGetUSValue SHRegGetUSValueW` (shlwapi.h).
#[inline]
pub unsafe fn SHRegGetUSValue(
    subkey: crate::island::LPCWSTR,
    value: crate::island::LPCWSTR,
    type_: *mut crate::island::DWORD,
    data: *mut ::core::ffi::c_void,
    count: *mut crate::island::DWORD,
    ignoreHKCU: crate::island::BOOL,
    default: *mut ::core::ffi::c_void,
    defaultLen: crate::island::DWORD,
) -> crate::island::LONG {
    crate::island::SHRegGetUSValueW(
        subkey, value, type_, data, count, ignoreHKCU, default, defaultLen,
    )
}
/// `#define SHRegSetUSValue SHRegSetUSValueW` (shlwapi.h).
#[inline]
pub unsafe fn SHRegSetUSValue(
    subkey: crate::island::LPCWSTR,
    value: crate::island::LPCWSTR,
    type_: crate::island::DWORD,
    data: *const ::core::ffi::c_void,
    count: crate::island::DWORD,
    flags: crate::island::DWORD,
) -> crate::island::LONG {
    // shlwapi.h types the value as a non-const `LPVOID` even though the SET path only reads it.
    crate::island::SHRegSetUSValueW(subkey, value, type_, data as crate::island::LPVOID, count, flags)
}
/// `#define MAKEINTRESOURCE MAKEINTRESOURCEW` (winuser.h). The cast through `usize` is
/// load-bearing: a resource ID travels in a pointer slot. Returns the MUTABLE spelling because
/// that is what `LPTSTR` is, and every site here assigns it to one.
#[inline]
pub fn MAKEINTRESOURCE(i: crate::island::UINT) -> crate::island::LPWSTR {
    i as usize as crate::island::LPWSTR
}
/// The explicitly-wide spelling, same value.
#[inline]
pub fn MAKEINTRESOURCEW(i: crate::island::UINT) -> crate::island::LPWSTR {
    i as usize as crate::island::LPWSTR
}

/// prsht.h `#define PropSheet_Changed(d, w) SNDMSG(d, PSM_CHANGED, (WPARAM)(w), 0)`.
#[inline]
pub unsafe fn PropSheet_Changed(
    hPropSheetDlg: crate::island::HWND,
    hwndPage: crate::island::HWND,
) -> crate::island::LRESULT {
    crate::island::SendMessageW(
        hPropSheetDlg,
        crate::island::PSM_CHANGED,
        hwndPage as crate::island::WPARAM,
        0,
    )
}

/// `#define ListView_GetItemCount(w) (int)SNDMSG((w), LVM_GETITEMCOUNT, 0, 0)` (commctrl.h).
#[inline]
pub unsafe fn ListView_GetItemCount(hwnd: crate::island::HWND) -> ::core::ffi::c_int {
    crate::island::SendMessageW(hwnd, crate::island::LVM_GETITEMCOUNT, 0, 0) as ::core::ffi::c_int
}

/// precomp.h `AddPropSheetPage` (a `static inline`, so no symbol exists to import and every
/// TU that used it got its own copy).
#[inline]
pub unsafe fn AddPropSheetPage(
    hPage: crate::island::HPROPSHEETPAGE,
    pfnAddPage: crate::island::LPFNADDPROPSHEETPAGE,
    lParam: crate::island::LPARAM,
) -> crate::island::HRESULT {
    if hPage.is_null() {
        return E_FAIL;
    }
    if let Some(add) = pfnAddPage {
        if add(hPage, lParam) != 0 {
            return S_OK;
        }
    }
    crate::island::DestroyPropertySheetPage(hPage);
    E_FAIL
}

// ---------------------------------------------------------------------------
// Residue no header-derived generator can produce.
// ---------------------------------------------------------------------------

/// `RESTRICTIONS` is a C enum, so bindgen prefixes every member with the enum's own name.
pub use crate::island::RESTRICTIONS_REST_CLASSICSHELL as REST_CLASSICSHELL;

/// commctrl.h declares `NMHDR` but the `P`-prefixed alias only exists as a `typedef` bindgen
/// did not reach; the struct itself comes from the island.
pub type PNMHDR = *mut crate::island::NMHDR;
/// `#define PROPSHEETPAGE PROPSHEETPAGEW` (prsht.h).
pub type PROPSHEETPAGE = crate::island::PROPSHEETPAGEW;

/// regstr.h `#define REGSTR_PATH_EXPLORER TEXT("Software\\Microsoft\\Windows\\CurrentVersion\\Explorer")`.
/// A `TEXT()` macro expands to a wide string LITERAL, which bindgen cannot emit as a constant.
pub static REGSTR_PATH_EXPLORER_BUF: [crate::island::WCHAR; 51] = [
    0x0053, 0x006F, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005C, 0x004D, 0x0069, 0x0063,
    0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0057, 0x0069, 0x006E, 0x0064, 0x006F,
    0x0077, 0x0073, 0x005C, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0056, 0x0065,
    0x0072, 0x0073, 0x0069, 0x006F, 0x006E, 0x005C, 0x0045, 0x0078, 0x0070, 0x006C, 0x006F, 0x0072,
    0x0065, 0x0072, 0x0000,
];
/// The pointer form the C uses (`REGSTR_PATH_EXPLORER` decays to `LPCWSTR` at every site).
pub static REGSTR_PATH_EXPLORER: &[crate::island::WCHAR; 51] = &REGSTR_PATH_EXPLORER_BUF;

/// `sdkddkver.h` version gates. These are BUILD defines (`-D_WIN32_WINNT=0x502`, CMakeLists.txt),
/// not header constants, so no island can carry them; keeping them named rather than folding the
/// comparison to a literal keeps the C's intent legible and survives a future bump.
pub const _WIN32_WINNT: u32 = 0x0502;
pub const _WIN32_WINNT_WIN2K: u32 = 0x0500;

/// shellutils.h `#define SHELL_GetSetting(ssf, field) (SHELL_GetSettingImpl(ssf)->field)` —
/// a C++ RAII temporary whose second argument is a FIELD NAME, so this cannot be a fn.
#[macro_export]
macro_rules! SHELL_GetSetting {
    ($ssf:expr, $field:ident) => {{
        let mut ss: $crate::island::SHELLSTATE = ::core::mem::zeroed();
        $crate::prelude::SHGetSetSettings(::core::ptr::addr_of_mut!(ss), $ssf, 0);
        ss.$field()
    }};
}
pub use crate::SHELL_GetSetting;

pub use core::cmp::min;

extern "C" {
    /// wchar.h: under `__cplusplus` (which every shell32 TU is) `swprintf` is the
    /// NON-CONFORMING two-fixed-argument form — no `count` parameter. The C99
    /// `swprintf(buf, count, fmt, ...)` spelling elsewhere in the tree is for C TUs and
    /// would shift every variadic argument by one slot here.
    pub fn swprintf(
        buffer: *mut crate::island::WCHAR,
        format: *const crate::island::WCHAR,
        ...
    ) -> ::core::ffi::c_int;
}
