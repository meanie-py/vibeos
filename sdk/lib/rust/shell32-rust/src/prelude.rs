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

// The COM interfaces, in the `lpVtbl` shape — the boundary the module header describes. The
// island re-exports com-sys's versions of several of these, and com-sys is bindgen output, so
// its vtable slots are `Option<fn>` while `shellcom`'s are raw fn pointers. One crate cannot
// carry both shapes: a translated call through an `Option` slot is `E0618: expected function`,
// and "just add `.unwrap()`" would put a PANIC on every COM call in a shipping DLL. Naming them
// explicitly here beats the island's glob and puts the whole crate on the raw-pointer shape the
// 40 landed COM modules already use.
pub use crate::shellcom::{
    IBindCtx, IBindCtxVtbl, IDataObject, IDataObjectVtbl, IEnumIDList, IEnumIDListVtbl,
    IPersistFolder, IPersistFolder3, IPersistFolder3Vtbl, IPersistFolderVtbl, IShellFolder,
    IShellFolder2, IShellFolder2Vtbl, IShellFolderVtbl, IUnknown, IUnknownVtbl,
};

// ATL's smart pointers and containers. The translated TUs use these by their C++ names, and
// `atl.rs` documents the three invariants those call sites lean on (zeroable, transparent,
// Drop-as-destructor).
pub use crate::atl::{
    CAtlList, CComAllocator, CComHeapPtr, CComPtr, CCRTAllocator, CHeapPtr, CLocalAllocator,
    CRegKey, CSimpleArray, CStringW,
};

// shellutils.h / utils.h header inlines. The explicit `use` deliberately shadows the island
// glob for `CCidaChildArrayHelper`: bindgen renders the C++ class as a bare layout with no
// constructor, and the translated call sites need the real one.
pub use crate::shellutils::{
    hida_clipformat, CCidaChildArrayHelper, CCoInit, COleInit, CScopedSetObjectWithSite,
    DataObject_GetData, DataObject_GetOffset, DataObject_SetData, DataObject_SetOffset,
    DestroyCIDA, HIDA_GetPIDLFolder, HIDA_GetPIDLItem, ILIsSingle, ReleaseStgMedium, SHAlloc,
};

// Accessor macros from `win32.rs`, but ONLY the ones whose signatures are type-neutral:
// `ZeroMemory<T>`/`CopyMemory<T,U>` are generic over the pointee, and `LOWORD`/`HIWORD` take a
// plain `DWORD`. Anything typed on `HWND`, `LPWSTR` or `LPARAM` is re-defined below against the
// island instead — `win32.rs` is written against `crate::types`, and importing one of those
// would re-open exactly the nominal split the module header warns about.
pub use crate::win32::{CopyMemory, HIWORD, LOWORD, ZeroMemory};
// Crate-root `#[macro_export]` macros — text substitution, so no types to split on.
pub use crate::{FIELD_OFFSET, ImageList_AddIcon, MAKELONG, MAKEROP4, TRACE_ON, UNREFERENCED_PARAMETER, V_BSTR, V_I4, V_UI4, V_VT, _countof};

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

/// guiddef.h `REFGUID` — C++ spells it `const GUID &`. A reference IS a pointer at the ABI,
/// so the alias is the pointer; the island renders `REFIID` the same way.
pub type REFGUID = *const crate::island::GUID;

extern "C" {
    /// shell32's type-library GUID. This is what ATL's
    /// `IDispatchImpl<T, piid, plibid = &CAtlModule::m_libid, ...>` resolves `plibid` to for
    /// every dual interface in this module, so `EnsureTILoaded` passes it to LoadRegTypeLib.
    /// Declared here rather than per-file because three classes now need it (CFolder,
    /// CFolderItem, CFolderItems) and `shell_dispatch.rs` had its own private copy.
    pub static LIBID_Shell32: crate::island::GUID;
}

/// oaidl.h `DISP_E_BADINDEX` — `GetTypeInfo`/`Item` reject an out-of-range index with it.
/// Not in the island: the DISP_E_* family lives in winerror.h behind `_OLEAUTO_H_`-style
/// guards the island's include closure does not reach.
pub const DISP_E_BADINDEX: crate::island::HRESULT = 0x8002_000Bu32 as crate::island::HRESULT;

// oleaut32. The island renders the oaidl/oleauto TYPES but not these entry points, so the
// automation objects (CFolder / CFolderItem / CFolderItems) had no declaration to call.
extern "system" {
    /// `BSTR SysAllocString(const OLECHAR *)` — allocates a BSTR copy; NULL on failure.
    pub fn SysAllocString(psz: *const crate::island::OLECHAR) -> crate::island::BSTR;
    /// `BSTR SysAllocStringLen(const OLECHAR *, UINT)` — allocates a BSTR of EXACTLY the
    /// given length, so unlike SysAllocString it does not stop at an embedded NUL and does
    /// not require the source to be NUL-terminated.
    pub fn SysAllocStringLen(psz: *const crate::island::OLECHAR, len: crate::island::UINT)
        -> crate::island::BSTR;
    /// `HRESULT VariantClear(VARIANT *)` — releases whatever the variant owns and sets it
    /// VT_EMPTY. This is what `~CComVariant` calls.
    pub fn VariantClear(pvarg: *mut crate::island::VARIANT) -> crate::island::HRESULT;
    /// `HRESULT VariantCopyInd(VARIANT *pvarDest, const VARIANT *pvarSrc)` — copies, and
    /// DEREFERENCES a by-reference source (`VT_BYREF`), which plain `VariantCopy` does not.
    pub fn VariantCopyInd(
        pvarDest: *mut crate::island::VARIANT,
        pvarSrc: *const crate::island::VARIANT,
    ) -> crate::island::HRESULT;
    /// `HRESULT LoadRegTypeLib(REFGUID, WORD, WORD, LCID, ITypeLib **)` — the registered
    /// type library, which is how ATL's `IDispatchImpl::EnsureTILoaded` gets its ITypeInfo.
    pub fn LoadRegTypeLib(
        rguid: REFGUID,
        wVerMajor: crate::island::WORD,
        wVerMinor: crate::island::WORD,
        lcid: crate::island::LCID,
        pptlib: *mut *mut crate::island::ITypeLib,
    ) -> crate::island::HRESULT;
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

// ===========================================================================================
// Wave-5 batch additions (the `shell32/` free-function TUs).
//
// Everything below follows the two rules in the module header: types come from the island, and
// a shell32-owned export is DECLARED against island types rather than imported from the Rust
// module that happens to define it.
// ===========================================================================================

// The C scalar spellings. bindgen writes `::core::ffi::c_int` in its own signatures but the
// translation writes the bare name, and `core::ffi` is not in a `no_std` prelude.
pub use ::core::ffi::{c_char, c_int, c_long, c_short, c_uchar, c_uint, c_ulong, c_ushort, c_void};

/// shell32_main.h:31 `#define ARRAY_SIZE(a) (sizeof(a) / sizeof(*(a)))`.
#[macro_export]
macro_rules! ARRAY_SIZE {
    ($a:expr) => {
        (($a).len()) as u32
    };
}
/// shellutils.h:88 `#define FAILED_UNEXPECTEDLY(hr) FAILED(hr)` in a non-debug build — the
/// debug spelling only adds a trace, so the VALUE is `FAILED`.
#[macro_export]
macro_rules! FAILED_UNEXPECTEDLY {
    ($hr:expr) => {
        (($hr) as $crate::island::HRESULT) < 0
    };
}
/// minwindef.h:188 `#define MAKELONG(wLow, wHigh) ((LONG)(((WORD)((DWORD_PTR)(wLow) & 0xffff)) |
/// (((DWORD)((WORD)((DWORD_PTR)(wHigh) & 0xffff))) << 16)))`. The `& 0xffff` truncations are
/// load-bearing: C narrows each half to a WORD before combining, so a caller passing a value
/// wider than 16 bits must lose the high bits rather than corrupt the other half.
#[macro_export]
macro_rules! MAKELONG {
    ($wLow:expr, $wHigh:expr $(,)?) => {
        (((($wLow) as u32) & 0xffff) | (((($wHigh) as u32) & 0xffff) << 16)) as i32
    };
}
/// wingdi.h:3392 `#define MAKEROP4(f,b) (DWORD)((((b)<<8)&0xFF000000)|(f))`.
#[macro_export]
macro_rules! MAKEROP4 {
    ($f:expr, $b:expr $(,)?) => {
        (((((($b) as u32) << 8) & 0xFF000000) | (($f) as u32)) as u32)
    };
}
/// commctrl.h:415 `#define ImageList_AddIcon(himl,hicon) ImageList_ReplaceIcon(himl,-1,hicon)`.
#[macro_export]
macro_rules! ImageList_AddIcon {
    ($himl:expr, $hicon:expr $(,)?) => {
        $crate::island::ImageList_ReplaceIcon($himl, -1, $hicon)
    };
}
/// shellutils.h:678 `#define MAKE_COMPARE_HRESULT(x) ((x)>0 ? S_GREATERTHAN : ((x)<0 ? S_LESSTHAN : S_EQUAL))`.
#[macro_export]
macro_rules! MAKE_COMPARE_HRESULT {
    ($x:expr) => {
        if ($x) > 0 {
            $crate::prelude::S_GREATERTHAN
        } else if ($x) < 0 {
            $crate::prelude::S_LESSTHAN
        } else {
            $crate::prelude::S_EQUAL
        }
    };
}
/// shlfileop.cpp:28-30 — file-attribute predicates, kept as macros because C used them on
/// anything integral.
#[macro_export]
macro_rules! IsAttrib {
    ($x:expr, $y:expr) => {
        ($crate::prelude::INVALID_FILE_ATTRIBUTES != ($x)) && (($x) & ($y)) != 0
    };
}
#[macro_export]
macro_rules! IsAttribFile {
    ($x:expr) => {
        (($x) & $crate::island::FILE_ATTRIBUTE_DIRECTORY) == 0
    };
}
#[macro_export]
macro_rules! IsAttribDir {
    ($x:expr) => {
        $crate::IsAttrib!($x, $crate::island::FILE_ATTRIBUTE_DIRECTORY)
    };
}
pub use crate::{ARRAY_SIZE, FAILED_UNEXPECTEDLY, IsAttrib, IsAttribDir, IsAttribFile, MAKE_COMPARE_HRESULT};

/// shellutils.h:675-677. `S_LESSTHAN` really is `0xffff`: `IShellFolder::CompareIDs` returns the
/// result in the LOW SHORT of the HRESULT, and `0xffff` is `-1` read as a `short`.
pub const S_LESSTHAN: crate::island::HRESULT = 0xffff;
pub const S_EQUAL: crate::island::HRESULT = 0;
pub const S_GREATERTHAN: crate::island::HRESULT = 1;

/// fileapi.h `#define INVALID_FILE_ATTRIBUTES ((DWORD)-1)` — a CAST macro, so bindgen has
/// nothing to bind and the island cannot carry it.
pub const INVALID_FILE_ATTRIBUTES: crate::island::DWORD = 0xFFFF_FFFF;

/// windowsx.h / winuser.h byte and word extraction.
#[inline]
pub const fn LOBYTE(v: crate::island::DWORD) -> crate::island::BYTE {
    (v & 0xff) as crate::island::BYTE
}
#[inline]
pub const fn HIBYTE(v: crate::island::DWORD) -> crate::island::BYTE {
    ((v >> 8) & 0xff) as crate::island::BYTE
}

/// shellutils.h:474-475 `static inline bool StrIsNullOrEmpty(LPCWSTR str)`.
#[inline]
pub unsafe fn StrIsNullOrEmpty(str_: crate::island::LPCWSTR) -> bool {
    str_.is_null() || *str_ == 0
}

/// commctrl.h `TreeView_*` — `SNDMSG` wrappers, so they are messages, not entry points.
#[inline]
pub unsafe fn TreeView_GetSelection(hwnd: crate::island::HWND) -> crate::island::HTREEITEM {
    crate::island::SendMessageW(
        hwnd,
        crate::island::TVM_GETNEXTITEM,
        crate::island::TVGN_CARET as crate::island::WPARAM,
        0,
    ) as crate::island::HTREEITEM
}
#[inline]
pub unsafe fn TreeView_GetChild(
    hwnd: crate::island::HWND,
    hitem: crate::island::HTREEITEM,
) -> crate::island::HTREEITEM {
    crate::island::SendMessageW(
        hwnd,
        crate::island::TVM_GETNEXTITEM,
        crate::island::TVGN_CHILD as crate::island::WPARAM,
        hitem as crate::island::LPARAM,
    ) as crate::island::HTREEITEM
}
#[inline]
pub unsafe fn TreeView_Expand(
    hwnd: crate::island::HWND,
    hitem: crate::island::HTREEITEM,
    code: crate::island::UINT,
) -> crate::island::BOOL {
    crate::island::SendMessageW(
        hwnd,
        crate::island::TVM_EXPAND,
        code as crate::island::WPARAM,
        hitem as crate::island::LPARAM,
    ) as crate::island::BOOL
}

/// commctrl.h:3455. The five `TreeView_Get*` below are all this macro with a fixed `code`.
#[inline]
pub unsafe fn TreeView_GetNextItem(
    hwnd: crate::island::HWND,
    hitem: crate::island::HTREEITEM,
    code: crate::island::UINT,
) -> crate::island::HTREEITEM {
    crate::island::SendMessageW(
        hwnd,
        crate::island::TVM_GETNEXTITEM,
        code as crate::island::WPARAM,
        hitem as crate::island::LPARAM,
    ) as crate::island::HTREEITEM
}
#[inline]
pub unsafe fn TreeView_GetRoot(hwnd: crate::island::HWND) -> crate::island::HTREEITEM {
    TreeView_GetNextItem(hwnd, core::ptr::null_mut(), crate::island::TVGN_ROOT)
}
#[inline]
pub unsafe fn TreeView_GetNextSibling(
    hwnd: crate::island::HWND,
    hitem: crate::island::HTREEITEM,
) -> crate::island::HTREEITEM {
    TreeView_GetNextItem(hwnd, hitem, crate::island::TVGN_NEXT)
}
#[inline]
pub unsafe fn TreeView_GetParent(
    hwnd: crate::island::HWND,
    hitem: crate::island::HTREEITEM,
) -> crate::island::HTREEITEM {
    TreeView_GetNextItem(hwnd, hitem, crate::island::TVGN_PARENT)
}

// `TVM_GETITEM`/`TVM_SETITEM`/`TVM_INSERTITEM` are the A/W-mapped spellings; the DLL builds
// UNICODE, so they are the `*W` messages. These take the item pointer generically for the same
// reason `ZeroMemory` does — the message is type-agnostic and the call sites pass `TVITEMW`,
// `TVITEMEXW` and `TV_ITEM` alike, which is exactly what the C macro's cast erased.
#[inline]
pub unsafe fn TreeView_GetItem<T>(hwnd: crate::island::HWND, pitem: *mut T) -> crate::island::BOOL {
    crate::island::SendMessageW(hwnd, crate::island::TVM_GETITEMW, 0, pitem as crate::island::LPARAM)
        as crate::island::BOOL
}
#[inline]
pub unsafe fn TreeView_SetItem<T>(
    hwnd: crate::island::HWND,
    pitem: *const T,
) -> crate::island::BOOL {
    crate::island::SendMessageW(hwnd, crate::island::TVM_SETITEMW, 0, pitem as crate::island::LPARAM)
        as crate::island::BOOL
}
#[inline]
pub unsafe fn TreeView_InsertItem<T>(
    hwnd: crate::island::HWND,
    lpis: *const T,
) -> crate::island::HTREEITEM {
    crate::island::SendMessageW(
        hwnd,
        crate::island::TVM_INSERTITEMW,
        0,
        lpis as crate::island::LPARAM,
    ) as crate::island::HTREEITEM
}
#[inline]
pub unsafe fn TreeView_DeleteItem(
    hwnd: crate::island::HWND,
    hitem: crate::island::HTREEITEM,
) -> crate::island::BOOL {
    crate::island::SendMessageW(
        hwnd,
        crate::island::TVM_DELETEITEM,
        0,
        hitem as crate::island::LPARAM,
    ) as crate::island::BOOL
}
/// commctrl.h:3486 `TreeView_SelectItem` = `TreeView_Select(.., TVGN_CARET)`.
#[inline]
pub unsafe fn TreeView_SelectItem(
    hwnd: crate::island::HWND,
    hitem: crate::island::HTREEITEM,
) -> crate::island::BOOL {
    crate::island::SendMessageW(
        hwnd,
        crate::island::TVM_SELECTITEM,
        crate::island::TVGN_CARET as crate::island::WPARAM,
        hitem as crate::island::LPARAM,
    ) as crate::island::BOOL
}
#[inline]
pub unsafe fn TreeView_SortChildren(
    hwnd: crate::island::HWND,
    hitem: crate::island::HTREEITEM,
    recurse: crate::island::BOOL,
) -> crate::island::BOOL {
    crate::island::SendMessageW(
        hwnd,
        crate::island::TVM_SORTCHILDREN,
        recurse as crate::island::WPARAM,
        hitem as crate::island::LPARAM,
    ) as crate::island::BOOL
}
#[inline]
pub unsafe fn TreeView_HitTest<T>(
    hwnd: crate::island::HWND,
    lpht: *mut T,
) -> crate::island::HTREEITEM {
    crate::island::SendMessageW(hwnd, crate::island::TVM_HITTEST, 0, lpht as crate::island::LPARAM)
        as crate::island::HTREEITEM
}
/// commctrl.h:3434. Note the comma operator in the C macro: the caller's `RECT` doubles as the
/// IN parameter, so the item handle is written OVER its first bytes before the message goes out.
/// Dropping that store leaves the control reading a garbage handle.
#[inline]
pub unsafe fn TreeView_GetItemRect(
    hwnd: crate::island::HWND,
    hitem: crate::island::HTREEITEM,
    prc: *mut crate::island::RECT,
    code: crate::island::BOOL,
) -> crate::island::BOOL {
    *(prc as *mut crate::island::HTREEITEM) = hitem;
    crate::island::SendMessageW(
        hwnd,
        crate::island::TVM_GETITEMRECT,
        code as crate::island::WPARAM,
        prc as crate::island::LPARAM,
    ) as crate::island::BOOL
}
#[inline]
pub unsafe fn TreeView_GetEditControl(hwnd: crate::island::HWND) -> crate::island::HWND {
    crate::island::SendMessageW(hwnd, crate::island::TVM_GETEDITCONTROL, 0, 0)
        as crate::island::HWND
}
#[inline]
pub unsafe fn TreeView_EditLabel(
    hwnd: crate::island::HWND,
    hitem: crate::island::HTREEITEM,
) -> crate::island::HWND {
    crate::island::SendMessageW(
        hwnd,
        crate::island::TVM_EDITLABELW,
        0,
        hitem as crate::island::LPARAM,
    ) as crate::island::HWND
}
#[inline]
pub unsafe fn TreeView_SetImageList(
    hwnd: crate::island::HWND,
    himl: crate::island::HIMAGELIST,
    iImage: crate::island::INT,
) -> crate::island::HIMAGELIST {
    crate::island::SendMessageW(
        hwnd,
        crate::island::TVM_SETIMAGELIST,
        iImage as crate::island::WPARAM,
        himl as crate::island::LPARAM,
    ) as crate::island::HIMAGELIST
}

/// commctrl.h `TV_ITEM` is the pre-Win32 spelling of `TVITEMW` in a UNICODE build.
pub type TV_ITEM = crate::island::TVITEMW;

/// `MAKEINTRESOURCEA` — the same cast as `MAKEINTRESOURCEW` above, narrow side.
#[inline]
pub fn MAKEINTRESOURCEA(i: crate::island::UINT) -> crate::island::LPSTR {
    i as usize as crate::island::LPSTR
}

/// winuser.h `DWLP_USER = DWLP_DLGPROC + sizeof(DLGPROC)`, i.e. two pointers past the start of
/// the dialog's extra bytes — so it is POINTER-WIDTH dependent, not a fixed 8.
pub const DWLP_USER: crate::island::INT = (2 * core::mem::size_of::<usize>()) as crate::island::INT;

/// windowsx.h — the signed halves of a packed mouse-position LPARAM.
#[inline]
pub fn GET_X_LPARAM(lp: crate::island::LPARAM) -> crate::island::INT {
    (lp & 0xFFFF) as i16 as crate::island::INT
}
#[inline]
pub fn GET_Y_LPARAM(lp: crate::island::LPARAM) -> crate::island::INT {
    ((lp >> 16) & 0xFFFF) as i16 as crate::island::INT
}

/// shobjidl.h `SHGDN_NORMAL` — the island's include closure does not reach it.
pub use crate::shellcom::SHGDN_NORMAL;

// The shell32-private PIDL helpers that live in `wine/pidl.rs` and `shell_link.rs`. Imported
// rather than re-declared: those modules are `extern "C"` and already typed compatibly here.
pub use crate::shell_link::__SHCloneStrAtoW;
pub use crate::wine::pidl::{_ILCreateDesktop, _ILGetDataPointer, _ILIsMyComputer};

extern "system" {
    /// `wine/shell32_main.h:190` — shell32's own menu-item helper.
    pub fn _InsertMenuItemW(
        hmenu: crate::island::HMENU,
        indexMenu: crate::island::UINT,
        fByPosition: crate::island::BOOL,
        wID: crate::island::UINT,
        fType: crate::island::UINT,
        dwTypeData: crate::island::LPCWSTR,
        fState: crate::island::UINT,
    ) -> crate::island::BOOL;
    /// C++ linkage in `utils.cpp`; reached through the WINAPI shim like its neighbours.
    #[link_name = "SHILAppend_RustBridge"]
    pub fn SHILAppend(
        pidl: crate::island::LPITEMIDLIST,
        ppidl: *mut crate::island::LPITEMIDLIST,
    ) -> crate::island::HRESULT;
}

extern "C" {
    /// msvcrt.
    pub fn free(block: *mut core::ffi::c_void);
    /// `precomp.h:291`, defined in `utils.cpp`. Plain __cdecl — NOT WINAPI — so it belongs in
    /// the `extern "C"` block; `extern "system"` mangles it to `_SHELL_GetIDListTarget@8` on
    /// i386, which is the only arch that catches the mismatch.
    pub fn SHELL_GetIDListTarget(
        pidl: crate::island::LPCITEMIDLIST,
        ppidl: *mut crate::island::LPITEMIDLIST,
    ) -> crate::island::HRESULT;
}

// `ui/layout.h` and the remaining `shellutils.h` / `compat_undoc.h` inlines. All header inlines,
// so they live in no TU — see those modules' headers.
pub use crate::layout::{LayoutDestroy, LayoutEnableResize, LayoutInit, LayoutShowGrip, LayoutUpdate};
pub use crate::shellutils::{
    RosGetProcessCompatVersion, RosGetProcessEffectiveVersion, SeeFlagsToCmicFlags,
    SHELL_IsContextMenuMsg, SHILClone, __SHCloneStrW, __SHCloneStrWtoA,
};

// shell32's OWN exports. Declared, not imported, for the reason given above the
// SHGetSetSettings block: the Rust modules that define them are typed against
// `crate::types`, and mixing those aliases with the island's splits every call site.
extern "system" {
    /// shlobj.h — the PIDL allocator family, implemented in `wine/pidl.rs`.
    pub fn ILFree(pidl: crate::island::LPITEMIDLIST);
    pub fn ILClone(pidl: crate::island::LPCITEMIDLIST) -> crate::island::LPITEMIDLIST;
    pub fn ILCombine(
        p1: crate::island::LPCITEMIDLIST,
        p2: crate::island::LPCITEMIDLIST,
    ) -> crate::island::LPITEMIDLIST;
    pub fn ILGetNext(pidl: crate::island::LPCITEMIDLIST) -> crate::island::LPITEMIDLIST;
    pub fn ILFindLastID(pidl: crate::island::LPCITEMIDLIST) -> crate::island::LPITEMIDLIST;
    pub fn ILIsEqual(
        p1: crate::island::LPCITEMIDLIST,
        p2: crate::island::LPCITEMIDLIST,
    ) -> crate::island::BOOL;
    /// shlobj.h `void WINAPI SHFree(LPVOID)` — implemented in `wine/shellole.rs`.
    pub fn SHFree(pv: crate::island::LPVOID);
    /// pidl.h — first-item clone, `#[no_mangle] extern "system"` in `wine/pidl.rs`.
    pub fn ILCloneFirst(pidl: crate::island::LPCITEMIDLIST) -> crate::island::LPITEMIDLIST;
    pub fn ILRemoveLastID(pidl: crate::island::LPITEMIDLIST) -> crate::island::BOOL;
    pub fn ILCreateFromPathW(path: crate::island::LPCWSTR) -> crate::island::LPITEMIDLIST;
    /// shlobj.h SDK imports the island's include closure does not reach.
    pub fn SHParseDisplayName(
        pszName: crate::island::PCWSTR,
        pbc: *mut core::ffi::c_void,
        ppidl: *mut crate::island::LPITEMIDLIST,
        sfgaoIn: crate::island::SFGAOF,
        psfgaoOut: *mut crate::island::SFGAOF,
    ) -> crate::island::HRESULT;
    pub fn SHGetFolderPathW(
        hwnd: crate::island::HWND,
        csidl: crate::island::INT,
        hToken: crate::island::HANDLE,
        dwFlags: crate::island::DWORD,
        pszPath: crate::island::LPWSTR,
    ) -> crate::island::HRESULT;
    pub fn SHGetFolderLocation(
        hwnd: crate::island::HWND,
        csidl: crate::island::INT,
        hToken: crate::island::HANDLE,
        dwReserved: crate::island::DWORD,
        ppidl: *mut crate::island::LPITEMIDLIST,
    ) -> crate::island::HRESULT;
    pub fn SHGetPathFromIDListW(
        pidl: crate::island::LPCITEMIDLIST,
        pszPath: crate::island::LPWSTR,
    ) -> crate::island::BOOL;
    pub fn SHGetFileInfoW(
        pszPath: crate::island::LPCWSTR,
        dwFileAttributes: crate::island::DWORD,
        psfi: *mut crate::island::SHFILEINFOW,
        cbFileInfo: crate::island::UINT,
        uFlags: crate::island::UINT,
    ) -> crate::island::DWORD_PTR;
    pub fn SHCloneSpecialIDList(
        hwnd: crate::island::HWND,
        csidl: crate::island::INT,
        bCreate: crate::island::BOOL,
    ) -> crate::island::LPITEMIDLIST;
    pub fn StrRetToStrNW(
        dest: crate::island::LPWSTR,
        len: crate::island::DWORD,
        pstr: *mut crate::island::STRRET,
        pidl: crate::island::LPCITEMIDLIST,
    ) -> crate::island::HRESULT;
    pub fn PathYetAnotherMakeUniqueName(
        pszUniqueName: crate::island::LPWSTR,
        pszPath: crate::island::LPCWSTR,
        pszShort: crate::island::LPCWSTR,
        pszFileSpec: crate::island::LPCWSTR,
    ) -> crate::island::BOOL;
    /// `precomp.h:290` — a real function, not an inline, defined in `utils.cpp` with C++
    /// linkage and no WINAPI, so it is reached through a shim rather than declared directly.
    #[link_name = "SHGetAttributes_RustBridge"]
    pub fn SHGetAttributes(
        psf: *mut crate::shellcom::IShellFolder,
        pidl: crate::island::LPCITEMIDLIST,
        dwAttributes: crate::island::DWORD,
    ) -> crate::island::DWORD;
    /// `precomp.h:217` — maps a verb string to its DFM_CMD_*. C++ linkage; shimmed.
    #[link_name = "GetDfmCmd_RustBridge"]
    pub fn GetDfmCmd(
        pCM: *mut core::ffi::c_void,
        verba: crate::island::LPCSTR,
    ) -> crate::island::UINT;
}

/// `SetWindowLongPtrW` is a winuser.h MACRO for `SetWindowLongW` on 32-bit — there is no export
/// of that name to link against, which is why the i386 island does not carry one. Same split as
/// `SetWindowLongPtr` above; kept under the `W` name because that is what the translated call
/// sites in `brfolder`, `layout` and `shlfileop` emit.
#[cfg(target_pointer_width = "32")]
#[inline]
pub unsafe fn SetWindowLongPtrW(
    hWnd: crate::island::HWND,
    nIndex: crate::island::INT,
    dwNewLong: crate::island::LONG_PTR,
) -> crate::island::LONG_PTR {
    crate::island::SetWindowLongW(hWnd, nIndex, dwNewLong as crate::island::LONG)
        as crate::island::LONG_PTR
}
#[cfg(target_pointer_width = "64")]
#[inline]
pub unsafe fn SetWindowLongPtrW(
    hWnd: crate::island::HWND,
    nIndex: crate::island::INT,
    dwNewLong: crate::island::LONG_PTR,
) -> crate::island::LONG_PTR {
    crate::island::SetWindowLongPtrW(hWnd, nIndex, dwNewLong)
}
#[cfg(target_pointer_width = "32")]
#[inline]
pub unsafe fn GetWindowLongPtrW(
    hWnd: crate::island::HWND,
    nIndex: crate::island::INT,
) -> crate::island::LONG_PTR {
    crate::island::GetWindowLongW(hWnd, nIndex) as crate::island::LONG_PTR
}
#[cfg(target_pointer_width = "64")]
#[inline]
pub unsafe fn GetWindowLongPtrW(
    hWnd: crate::island::HWND,
    nIndex: crate::island::INT,
) -> crate::island::LONG_PTR {
    crate::island::GetWindowLongPtrW(hWnd, nIndex)
}

// The icon cache, flipped in `shell32::iconcache` — imported rather than re-declared, because
// that module is already written against the island (it is a `shell32/` TU like this one).
pub use crate::shell32::iconcache::{
    SHMapPIDLToSystemImageListIndex, Shell_GetCachedImageIndexW, Shell_GetImageLists,
};

/// commctrl.h:3373 `#define TVI_ROOT ((HTREEITEM)(ULONG_PTR)-0x10000)`.
pub const TVI_ROOT: crate::island::HTREEITEM = -0x10000isize as crate::island::HTREEITEM;
/// winuser.h:695/697 — `MAKEINTRESOURCE`d cursor ids.
pub const IDC_ARROW: crate::island::LPCWSTR = 32512isize as crate::island::LPCWSTR;
pub const IDC_WAIT: crate::island::LPCWSTR = 32514isize as crate::island::LPCWSTR;
/// shtypes.h — the PIDL type-safety family the island erases to one pointer type.
pub type PCIDLIST_RELATIVE = crate::island::LPCITEMIDLIST;
/// commctrl.h — the A/W mapping, UNICODE side.
pub type NMTVDISPINFO = crate::island::NMTVDISPINFOW;

// The shell32-private `_IL*` family. Same reasoning as the block above — these ARE defined in
// this crate (`wine/pidl.rs`), but against `crate::types` aliases, so they are declared rather
// than imported. They are `extern "C"` (cdecl) there, not `extern "system"`; declaring the
// wrong one links to a differently-decorated symbol on i386 and fails loudly.
extern "C" {
    pub fn _ILIsDesktop(pidl: crate::island::LPCITEMIDLIST) -> crate::island::BOOL;
    pub fn _ILIsPidlSimple(pidl: crate::island::LPCITEMIDLIST) -> crate::island::BOOL;
    pub fn _ILIsValue(pidl: crate::island::LPCITEMIDLIST) -> crate::island::BOOL;
    pub fn _ILIsFolder(pidl: crate::island::LPCITEMIDLIST) -> crate::island::BOOL;
    pub fn _ILIsDrive(pidl: crate::island::LPCITEMIDLIST) -> crate::island::BOOL;
    pub fn _ILGetDepth(pidl: crate::island::LPCITEMIDLIST) -> crate::island::UINT;
    pub fn _ILGetFileStructW(
        pidl: crate::island::LPCITEMIDLIST,
    ) -> *mut crate::island::FileStructW;
    pub fn _ILSimpleGetTextW(
        pidl: crate::island::LPCITEMIDLIST,
        szOut: crate::island::LPWSTR,
        uOutSize: crate::island::UINT,
    ) -> crate::island::DWORD;
    pub fn _ILCopyCidaToaPidl(
        pidl: *mut crate::island::LPITEMIDLIST,
        cida: *const crate::island::CIDA,
    ) -> *mut crate::island::LPITEMIDLIST;
}

/// `shfldr.h` — combine and report allocation failure as an HRESULT. A C++ header inline, so it
/// is in no translation unit and no translated TU carries it.
#[inline]
pub unsafe fn SHILCombine(
    base: crate::island::LPCITEMIDLIST,
    sub: crate::island::LPCITEMIDLIST,
    ppOut: *mut crate::island::LPITEMIDLIST,
) -> crate::island::HRESULT {
    *ppOut = ILCombine(base, sub);
    if (*ppOut).is_null() {
        E_OUTOFMEMORY
    } else {
        S_OK
    }
}

/// `pidl.h` — an empty PIDL and the desktop PIDL are the same thing (a bare terminator).
#[inline]
pub unsafe fn _ILIsEmpty(pidl: crate::island::LPCITEMIDLIST) -> crate::island::BOOL {
    _ILIsDesktop(pidl)
}

// ---------------------------------------------------------------------------------------------
// Residue for the `shell32/` free-function TUs. Split by WHERE the definition lives, because
// that decides whether a name can be declared or has to be written.
// ---------------------------------------------------------------------------------------------

extern "system" {
    /// shlwapi/ole32/shell32 imports the island's include closure did not reach.
    pub fn SHCoCreateInstance(
        aclsid: crate::island::LPCWSTR,
        clsid: *const com_sys::IID,
        pUnkOuter: *mut crate::shellcom::IUnknown,
        riid: REFIID,
        ppv: *mut *mut ::core::ffi::c_void,
    ) -> crate::island::HRESULT;
    pub fn VariantChangeType(
        dst: *mut crate::island::VARIANT,
        src: *const crate::island::VARIANT,
        flags: crate::island::USHORT,
        vt: crate::island::VARTYPE,
    ) -> crate::island::HRESULT;
    pub fn ShellExecuteExW(info: *mut crate::island::SHELLEXECUTEINFOW) -> crate::island::BOOL;
    /// shell32's own, implemented in `wine/pidl.rs`.
    pub fn SHGetIDListFromObject(
        punk: *mut crate::shellcom::IUnknown,
        ppidl: *mut crate::island::LPITEMIDLIST,
    ) -> crate::island::HRESULT;
    /// `CFSFolder::CompareUiStrings` is a C++ STATIC MEMBER, so it has no callable name from
    /// Rust; CFSFolder.cpp exports this bridge for exactly that reason, and `recycle_bin.rs`
    /// already goes through it. It is `extern "C" INT WINAPI` — the `extern "C"` is linkage,
    /// the WINAPI is the CALLING CONVENTION, so this belongs in the `extern "system"` block.
    /// Declaring it `extern "C"` cost an i386 link failure on `_CFSFolder_..._RustBridge`
    /// against the defined `_CFSFolder_..._RustBridge@12`.
    pub fn CFSFolder_CompareUiStrings_RustBridge(
        a: crate::island::LPCWSTR,
        b: crate::island::LPCWSTR,
        lParam: crate::island::LPARAM,
    ) -> ::core::ffi::c_int;
}

extern "C" {
    /// utils.cpp / propsheet-side C++, reachable only because precomp.h marks them EXTERN_C.
    pub fn PathIsValidElement(pszPath: crate::island::LPCWSTR) -> crate::island::BOOL;
    pub fn SHELL32_ShowFilesystemItemsPropertiesDialogAsync(
        hOwner: crate::island::HWND,
        pDO: *mut crate::shellcom::IDataObject,
    ) -> crate::island::HRESULT;
}

/// `wine/shell32_main.h` property-system inlines — see `crate::shellutils` for why these are
/// Rust now rather than declared.
pub use crate::shellutils::{SHELL_CreateSHDESCRIPTIONID, SHELL_CreateVariantBuffer};

/// shtypes.idl — STRRET carries an ANSI string inline in `cStr`.
pub const STRRET_CSTR: crate::island::UINT = 2;
/// shlobj.h — the parsing-name form of a display name.
pub const SHGDN_FORPARSING: crate::island::DWORD = 0x8000;
/// FMTID_SummaryInformation {F29F85E0-4FF9-1068-AB91-08002B27B3D9}.
pub static FMTID_SummaryInformation: crate::island::GUID = crate::island::GUID {
    Data1: 0xF29F_85E0,
    Data2: 0x4FF9,
    Data3: 0x1068,
    Data4: [0xAB, 0x91, 0x08, 0x00, 0x2B, 0x27, 0xB3, 0xD9],
};

/// `strsafe.h` `StringCchCopyW` — an INLINE, so no symbol exists to import and it has to be
/// written. Truncates and returns STRSAFE_E_INSUFFICIENT_BUFFER rather than overrunning.
pub unsafe fn StringCchCopyW(
    dst: crate::island::LPWSTR,
    cchDest: usize,
    src: crate::island::LPCWSTR,
) -> crate::island::HRESULT {
    StringCchCopyNW(dst, cchDest, src, usize::MAX)
}

/// `strsafe.h` `StringCchCopyNW` — copy at most `cchToCopy` chars, always NUL-terminating.
pub unsafe fn StringCchCopyNW(
    dst: crate::island::LPWSTR,
    cchDest: usize,
    src: crate::island::LPCWSTR,
    cchToCopy: usize,
) -> crate::island::HRESULT {
    const STRSAFE_E_INVALID_PARAMETER: crate::island::HRESULT = 0x8007_0057u32 as _;
    const STRSAFE_E_INSUFFICIENT_BUFFER: crate::island::HRESULT = 0x8007_007Au32 as _;
    if dst.is_null() || cchDest == 0 {
        return STRSAFE_E_INVALID_PARAMETER;
    }
    let mut i = 0usize;
    if !src.is_null() {
        while i < cchToCopy && i + 1 < cchDest && *src.add(i) != 0 {
            *dst.add(i) = *src.add(i);
            i += 1;
        }
    }
    *dst.add(i) = 0;
    // Truncated only if the source still had characters we were asked to copy.
    if !src.is_null() && i < cchToCopy && *src.add(i) != 0 {
        return STRSAFE_E_INSUFFICIENT_BUFFER;
    }
    S_OK
}

/// oaidl.h `V_VT(pvar)` — the discriminant of a VARIANT.
#[macro_export]
macro_rules! V_VT {
    ($v:expr) => {
        (*($v)).n1.n2.vt
    };
}

/// oaidl.h `V_BSTR(pvar)`.
#[macro_export]
macro_rules! V_BSTR {
    ($v:expr) => {
        (*($v)).n1.n2.n3.bstrVal
    };
}

/// oaidl.h `V_I4(pvar)` — the `VT_I4` arm, `lVal`.
#[macro_export]
macro_rules! V_I4 {
    ($v:expr) => {
        (*($v)).n1.n2.n3.lVal
    };
}

/// oaidl.h `V_UI4(pvar)` — the `VT_UI4` arm, `ulVal`. CFolderItems::Item reads it to take
/// the integer index out of the caller's VARIANT.
#[macro_export]
macro_rules! V_UI4 {
    ($v:expr) => {
        (*($v)).n1.n2.n3.ulVal
    };
}

/// shobjidl.h `IsEqualPropertyKey(a, b)` — same FMTID and same PID.
#[inline]
pub unsafe fn IsEqualPropertyKey(
    a: *const crate::island::SHCOLUMNID,
    b: *const crate::island::SHCOLUMNID,
) -> bool {
    // By POINTER, not by value: SHCOLUMNID is packed, so a field reference is unaligned, and
    // the call sites hold it behind a raw pointer they cannot move out of.
    core::ptr::read_unaligned(core::ptr::addr_of!((*a).pid))
        == core::ptr::read_unaligned(core::ptr::addr_of!((*b).pid))
        && IsEqualGUID(core::ptr::addr_of!((*a).fmtid), core::ptr::addr_of!((*b).fmtid))
}

/// guiddef.h `IsEqualGUID` — bindgen does not derive PartialEq for `_GUID`, and a byte compare
/// is what the C macro does anyway.
///
/// Generic over the pointee on purpose. `ntdll-sys` and `com-sys` each emit their OWN `_GUID`,
/// and rustc says so directly: "have similar names, but are actually distinct types". Pinning
/// this to either one makes every call from the other side fail, and casting at each call site
/// would just spread the choice around. A GUID is 16 bytes in every spelling, so compare bytes.
#[inline]
pub unsafe fn IsEqualGUID<A, B>(a: *const A, b: *const B) -> bool {
    const GUID_BYTES: usize = 16;
    core::slice::from_raw_parts(a as *const u8, GUID_BYTES)
        == core::slice::from_raw_parts(b as *const u8, GUID_BYTES)
}

// ---------------------------------------------------------------------------------------------
// OLE residue. `com-sys` supplies the STRUCTS (STGMEDIUM, FORMATETC, the interface family) and
// the island re-exports them, but these four have no binding to inherit: TYMED and DVASPECT are
// IDL enums that never reach a C header bindgen parses, E_UNEXPECTED is a cast macro, and REFIID
// is `const IID&` in C++ / `const IID* const` in C — a spelling, not a type bindgen can emit.
// Values read from the headers, not transcribed from another Rust file.
// ---------------------------------------------------------------------------------------------

/// guiddef.h:118 `#define REFIID const IID* __MIDL_CONST` (the C spelling; the C++ one is a
/// reference, which has the same representation).
pub type REFIID = *const com_sys::IID;
pub type REFCLSID = *const com_sys::IID;
pub use com_sys::{GUID, IID};

/// objidl.idl:979-986 `enum TYMED`.
pub const TYMED_NULL: crate::island::DWORD = 0;
pub const TYMED_HGLOBAL: crate::island::DWORD = 1;
pub const TYMED_FILE: crate::island::DWORD = 2;
pub const TYMED_ISTREAM: crate::island::DWORD = 4;
pub const TYMED_ISTORAGE: crate::island::DWORD = 8;
pub const TYMED_GDI: crate::island::DWORD = 16;
pub const TYMED_MFPICT: crate::island::DWORD = 32;
pub const TYMED_ENHMF: crate::island::DWORD = 64;

/// wtypes.idl:606 `enum DVASPECT`.
pub const DVASPECT_CONTENT: crate::island::DWORD = 1;
pub const DVASPECT_THUMBNAIL: crate::island::DWORD = 2;
pub const DVASPECT_ICON: crate::island::DWORD = 4;
pub const DVASPECT_DOCPRINT: crate::island::DWORD = 8;

/// winerror.h:3528 `#define E_UNEXPECTED _HRESULT_TYPEDEF_(0x8000FFFF)`.
pub const E_UNEXPECTED: crate::island::HRESULT = 0x8000FFFFu32 as crate::island::HRESULT;

extern "C" {
    /// The CRT entry points the DLL already imports. NEVER hand-write these — LLVM folds a
    /// byte-loop implementation back into a call to itself.
    pub fn memset(dst: *mut ::core::ffi::c_void, c: ::core::ffi::c_int, n: usize)
        -> *mut ::core::ffi::c_void;
    pub fn memcpy(dst: *mut ::core::ffi::c_void, src: *const ::core::ffi::c_void, n: usize)
        -> *mut ::core::ffi::c_void;
    pub fn wcscat(dst: crate::island::LPWSTR, src: crate::island::LPCWSTR)
        -> crate::island::LPWSTR;
    pub fn _wtoi(s: crate::island::LPCWSTR) -> ::core::ffi::c_int;
    pub fn _wcsicmp(a: crate::island::LPCWSTR, b: crate::island::LPCWSTR) -> ::core::ffi::c_int;
}

/// wine's shell32 string shims are `#define`s onto the CRT wide entry points.
pub use crate::island::{wcschr as strchrW, wcscmp as strcmpW, wcslen as strlenW};
pub use wcscat as strcatW;

// ── shlexec.cpp residue ────────────────────────────────────────────────────────────────────
// Declared here against the ISLAND rather than imported from `win32.rs`, for the reason given
// above the "shell32's OWN exports" block: those are typed against `crate::types`, and mixing
// the two alias families splits every call site.

extern "system" {
    /// shlwapi ordinal 51. Resolves + qualifies a path in place.
    pub fn PathResolveW(
        path: crate::island::LPWSTR,
        dirs: *mut crate::island::LPCWSTR,
        flags: crate::island::DWORD,
    ) -> crate::island::BOOL;
    /// shlwapi `SHRegQueryValueExW` — defined in-crate (`wine/shellreg.rs`).
    pub fn SHRegQueryValueExW(
        hkey: crate::island::HKEY,
        pszValue: crate::island::LPCWSTR,
        pdwReserved: crate::island::LPDWORD,
        pdwType: crate::island::LPDWORD,
        pvData: crate::island::LPVOID,
        pcbData: crate::island::LPDWORD,
    ) -> crate::island::LONG;
    pub fn SHBindToParent(
        pidl: crate::island::PCIDLIST_ABSOLUTE,
        riid: REFIID,
        ppv: *mut *mut core::ffi::c_void,
        ppidlLast: *mut crate::island::PCUITEMID_CHILD,
    ) -> crate::island::HRESULT;
    pub fn SHExtCoCreateInstance(
        aclsid: crate::island::LPCWSTR,
        clsid: *const crate::island::CLSID,
        pUnkOuter: *mut core::ffi::c_void,
        riid: REFIID,
        ppv: *mut crate::island::LPVOID,
    ) -> crate::island::HRESULT;
}

extern "C" {
    /// `COpenWithMenu.cpp:32` `EXTERN_C BOOL PathIsExeW(LPCWSTR)` — EXTERN_C is LINKAGE; there
    /// is no WINAPI here, so the convention is cdecl. `scan_cc_mismatch` caught this declared
    /// as `extern "system"`, which on i386 is a stack-imbalance waiting to happen.
    pub fn PathIsExeW(path: crate::island::LPCWSTR) -> crate::island::BOOL;
    /// `wine/shell32_main.h:70` — no WINAPI, and that header wraps itself in `extern "C"`.
    pub fn HCR_GetExecuteCommandW(
        hkeyClass: crate::island::HKEY,
        szClass: crate::island::LPCWSTR,
        szVerb: crate::island::LPCWSTR,
        szDest: crate::island::LPWSTR,
        len: crate::island::DWORD,
    ) -> crate::island::BOOL;
    /// `precomp.h:294` — plain `HRESULT SHCoInitializeAnyApartment(VOID)`, no WINAPI.
    pub fn SHCoInitializeAnyApartment() -> crate::island::HRESULT;
    /// `precomp.h:296` — plain `HRESULT SHGetNameAndFlagsW(...)`, no WINAPI.
    pub fn SHGetNameAndFlagsW(
        pidl: crate::island::LPCITEMIDLIST,
        dwFlags: crate::island::DWORD,
        pszText: crate::island::LPWSTR,
        cchBuf: crate::island::UINT,
        pdwAttributes: *mut crate::island::DWORD,
    ) -> crate::island::HRESULT;
}

// `wine/unicode.h`'s A/W-generic aliases. Text substitution in C, so they are `#[inline]`
// forwarders here rather than a second extern for the same CRT symbol.
/// `#define strcpyW(d,s) wcscpy((d),(s))`
#[inline]
pub unsafe fn strcpyW(d: crate::island::LPWSTR, s: crate::island::LPCWSTR) -> crate::island::LPWSTR {
    crate::island::wcscpy(d, s)
}
/// `#define strstrW(d,s) wcsstr((d),(s))`
#[inline]
pub unsafe fn strstrW(d: crate::island::LPCWSTR, s: crate::island::LPCWSTR) -> crate::island::LPWSTR {
    crate::island::wcsstr(d, s)
}
/// `#define strrchrW(s,c) wcsrchr((s),(c))`
#[inline]
pub unsafe fn strrchrW(s: crate::island::LPCWSTR, c: crate::island::WCHAR) -> crate::island::LPWSTR {
    crate::island::wcsrchr(s, c)
}
/// `#define strncmpiW(s1,s2,n) _wcsnicmp((s1),(s2),(n))`
#[inline]
pub unsafe fn strncmpiW(
    s1: crate::island::LPCWSTR,
    s2: crate::island::LPCWSTR,
    n: usize,
) -> crate::island::INT {
    crate::island::_wcsnicmp(s1, s2, n)
}
/// `#define isspaceW(n) iswspace((n))`
#[inline]
pub unsafe fn isspaceW(c: crate::island::WCHAR) -> crate::island::INT {
    crate::island::iswspace(c as crate::island::INT)
}

extern "C" {
    /// `strsafe.h:515` — the bounded `swprintf`. Rust can CALL a C variadic on stable, but
    /// there is nothing here to call: `STRSAFEAPI` is `static __inline`, so this is a PHANTOM
    /// SYMBOL with no export, and the `extern` decl this used to be linked nowhere. It now
    /// binds to the forwarding trampoline in `dll/win32/shell32/rust_strsafe_shim.c` — its own
    /// TU, NOT an append to `rust_varargs_shim.c`, which deliberately includes no Windows
    /// headers so it can hand-declare the variadic it defines —
    /// which calls the header's own `StringCchVPrintfW` and so keeps strsafe's truncation
    /// contract rather than reimplementing it.
    #[link_name = "StringCchPrintfW_RustBridge"]
    pub fn StringCchPrintfW(
        pszDest: crate::island::LPWSTR,
        cchDest: usize,
        pszFormat: crate::island::LPCWSTR,
        ...
    ) -> crate::island::HRESULT;
}

// ── dialogs/drive.cpp residue ──────────────────────────────────────────────────────────────

/// `strsafe.h` `StringCchCatW` — an INLINE, so there is no symbol to import. Appends `src` to
/// whatever `dst` already holds, never writing past `cchDest` chars and always NUL-terminating;
/// `STRSAFE_E_INSUFFICIENT_BUFFER` on truncation. That truncation contract is the whole reason
/// a caller reaches for strsafe instead of `wcscat`, so it is preserved rather than approximated.
pub unsafe fn StringCchCatW(
    dst: crate::island::LPWSTR,
    cchDest: usize,
    src: crate::island::LPCWSTR,
) -> crate::island::HRESULT {
    const STRSAFE_E_INVALID_PARAMETER: crate::island::HRESULT = 0x8007_0057u32 as _;
    const STRSAFE_E_INSUFFICIENT_BUFFER: crate::island::HRESULT = 0x8007_007Au32 as _;
    if dst.is_null() || cchDest == 0 {
        return STRSAFE_E_INVALID_PARAMETER;
    }
    // Find the existing terminator; an unterminated destination is a caller error, and strsafe
    // reports it rather than walking off the end.
    let mut i = 0usize;
    while i < cchDest && *dst.add(i) != 0 {
        i += 1;
    }
    if i == cchDest {
        return STRSAFE_E_INVALID_PARAMETER;
    }
    let mut j = 0usize;
    if !src.is_null() {
        while i + 1 < cchDest && *src.add(j) != 0 {
            *dst.add(i) = *src.add(j);
            i += 1;
            j += 1;
        }
    }
    *dst.add(i) = 0;
    if !src.is_null() && *src.add(j) != 0 {
        return STRSAFE_E_INSUFFICIENT_BUFFER;
    }
    S_OK
}

/// `strsafe.h` `StringCbCatW` — the same call with the destination size in BYTES. The `/ 2` is
/// the whole difference and the whole hazard: passing a byte count to the `Cch` form would let
/// the append run twice as far as the buffer.
#[inline]
pub unsafe fn StringCbCatW(
    dst: crate::island::LPWSTR,
    cbDest: usize,
    src: crate::island::LPCWSTR,
) -> crate::island::HRESULT {
    StringCchCatW(dst, cbDest / core::mem::size_of::<crate::island::WCHAR>(), src)
}

/// `winioctl.h:143` `#define CTL_CODE(t,f,m,a) (((t)<<16)|((a)<<14)|((f)<<2)|(m))`.
///
/// A function-like macro, so bindgen emits nothing for it OR for the ~200 `IOCTL_*` codes
/// defined through it — the island carries `FILE_DEVICE_*`, `METHOD_*` and `FILE_*_ACCESS` (it
/// allowlists winioctl.h) but not a single assembled code. Computing them here from those parts
/// keeps them derived from the header rather than transcribed as magic numbers.
#[inline]
pub const fn CTL_CODE(
    DeviceType: crate::island::DWORD,
    Function: crate::island::DWORD,
    Method: crate::island::DWORD,
    Access: crate::island::DWORD,
) -> crate::island::DWORD {
    (DeviceType << 16) | (Access << 14) | (Function << 2) | Method
}

/// `winioctl.h:180-202` — the four disk codes `dialogs/drive.rs` issues.
pub const IOCTL_DISK_GET_DRIVE_GEOMETRY: crate::island::DWORD = CTL_CODE(
    crate::island::FILE_DEVICE_DISK,
    0,
    crate::island::METHOD_BUFFERED,
    crate::island::FILE_ANY_ACCESS,
);
pub const IOCTL_DISK_GET_PARTITION_INFO: crate::island::DWORD = CTL_CODE(
    crate::island::FILE_DEVICE_DISK,
    1,
    crate::island::METHOD_BUFFERED,
    crate::island::FILE_READ_ACCESS,
);
pub const IOCTL_DISK_GET_PARTITION_INFO_EX: crate::island::DWORD = CTL_CODE(
    crate::island::FILE_DEVICE_DISK,
    0x12,
    crate::island::METHOD_BUFFERED,
    crate::island::FILE_ANY_ACCESS,
);
pub const IOCTL_DISK_GET_LENGTH_INFO: crate::island::DWORD = CTL_CODE(
    crate::island::FILE_DEVICE_DISK,
    0x17,
    crate::island::METHOD_BUFFERED,
    crate::island::FILE_READ_ACCESS,
);

/// `basetsd.h` `#define UlongToHandle(h) ((HANDLE)(ULONG_PTR)(h))`.
#[inline]
pub fn UlongToHandle(h: crate::island::ULONG) -> crate::island::HANDLE {
    h as usize as crate::island::HANDLE
}

/// `winnt.h` `SecureZeroMemory` — a zeroing the optimiser may not elide. `write_volatile`
/// is the Rust spelling of that guarantee; a plain `write_bytes` is exactly what the C
/// intrinsic exists to prevent.
#[inline]
pub unsafe fn SecureZeroMemory(ptr: *mut core::ffi::c_void, cnt: usize) {
    let p = ptr as *mut u8;
    for i in 0..cnt {
        core::ptr::write_volatile(p.add(i), 0);
    }
}
