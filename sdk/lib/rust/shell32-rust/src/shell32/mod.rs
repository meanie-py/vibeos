//! Wave-5 free-function translation units from the root of `dll/win32/shell32/`.
//!
//! These are whole plain-C++ TUs with no COM class of their own, translated through
//! `--free-functions`. They take their substrate from `crate::prelude` — the bindgen island —
//! rather than the hand-written `types.rs`/`win32.rs`; see prelude.rs for why that is
//! "instead of" and not "as well as".

/// SDK constants the island does NOT provide. Everything it did provide was pruned; see that
/// module's header for the value-by-value comparison that justified the pruning.
pub mod sdk_consts;

pub mod stubs;

/// `shldataobject.cpp`, flipped. Its four C++-mangled exports needed `EXTERN_C` in precomp.h
/// first — see the comment there.
pub mod shldataobject;

/// `shlfolder.cpp`, flipped. 19 of its exports had live C++ callers and needed `EXTERN_C` on
/// their declarations (shfldr.h, plus two in precomp.h). One of the 19 was an OVERLOAD PAIR of
/// `SHELL32_CoCreateInitSF`, which `EXTERN_C` cannot express — one name, one symbol — so the
/// csidl-taking form is now `SHELL32_CoCreateInitSFWithCsidl` on both sides. The C++ `const
/// GUID&` / `REFIID` reference parameters here are NOT an obstacle to `EXTERN_C`: it changes
/// linkage, not the type system, and a reference is a pointer at the ABI.
pub mod shlfolder;

/// `iconcache.cpp`, flipped. No `EXTERN_C` work was needed: its four shell32-internal exports
/// (`SIC_Initialize`, `SIC_Destroy`, `SIC_GetIconIndex`, `PidlToSicIndex`) are declared in
/// `wine/shell32_main.h`, which already wraps the whole header in `extern "C"` for C++.
pub mod iconcache;

// ---------------------------------------------------------------------------------------------
// ALL SIX ARE FLIPPED (shlexec, the last, on 2026-08-01). Nothing here is parked any more, so
// the counts and per-file blocker lists that used to live in this header are gone with the
// blockers; each module's own doc comment records what it needed and what it was hiding.
//
// What the batch cost, kept because it is the reusable part: the C++ SUBSTRATE these TUs lean
// on is `crate::atl` (CComPtr/CHeapPtr/CComHeapPtr/CSimpleArray/CStringW), `crate::shellutils`
// (the shellutils.h/utils.h header inlines, plus the CCoInit/COleInit RAII pair), `crate::layout`
// (ui/layout.h) and `crate::shellcom` (the `lpVtbl` shape for interfaces the island renders as
// opaque C++ classes). A header inline or a C++ template reaches no TU, so rustconv never sees a
// body for one and the name simply does not resolve — those are ports, not translations.
//
// Two traps worth keeping, both of which cost real time here:
//
// Do NOT add ULARGE_INTEGER or the ERROR_*/FILE_ATTRIBUTE_*/LANG_* constants to this crate:
// they already arrive through `ntdll_sys::nt::*`, which a scan of the `island_*.rs` files
// alone does not reveal. A second definition is E0659 at every use, not a shadow.
//
// If you bulk-apply casts from rustc spans, GUARD MATCH PATTERNS. A cast is not legal in a
// pattern; `crate-repair` declines those by design and a hand-rolled pass that forgets to
// silently writes `(SE_ERR_FNF) as usize => ...`, which does not compile. Cast the SCRUTINEE.
//
// WARNING for whoever runs `crate-repair` on this crate: **it edits shared MACRO DEFINITIONS
// to satisfy errors reported at their EXPANSION sites, once per call site.** Seen three times,
// on two different macros:
//   * `_countof!` — rewritten `u32` -> `i32` plus compensating casts in three ALREADY-LANDED
//     files. C's `sizeof(a)/sizeof(*a)` is unsigned.
//   * `ARRAY_SIZE!` — stacked into
//     `(((((((($a).len()) as u32) as usize) as u32) as usize) as u32) as usize) as u32`
//     while repairing shlexec (2026-07-31), and rustc asks for the same edit AGAIN at the three
//     `strlenW(x) + 1 > ARRAY_SIZE!(y)` sites in that file (it wants `.try_into().unwrap()` in
//     the macro body). Cast at the call site: `> ARRAY_SIZE!(y) as usize`.
// Revert the macro every time and cast at the call site instead. The test that it was pure
// blast radius: re-run `cargo check` after reverting — the error count does not move (it did
// not, on any of the three occasions).
//
// One measurement worth remembering: resolving a name does not only REMOVE errors, it also
// un-masks the ones that were hiding behind it. brfolder went 235 -> 244 when the ATL types
// started resolving, and shlexec's E0308 count went 56 -> 61 when its externs landed. A rising
// count on a file mid-grind is progress, not regression.
//
// These were PAID artifacts — do not delete a translated `.rs` to tidy up.
//
/// brfolder.cpp — the SHBrowseForFolder dialog. FLIPPED 2026-07-31: 86 -> 0. The residue was
/// this file's own casts plus four defects a green C build hides: amd64 widths baked into
/// `LRESULT`/`UINT_PTR`/`SIZE_T` casts, a C++ `!` turned into Rust's BITWISE not, one of two
/// `BrFolder_GetName` OVERLOADS dropped, and a `COleInit` RAII local left uninitialised so
/// neither OleInitialize nor OleUninitialize ran.
pub mod brfolder;
/// shlexec.cpp — ShellExecute and the whole verb / association / DDE launch path, and the last
/// of the six to flip (2026-08-01: 94 -> 0). Six defects a green C build hides, every one of
/// them somewhere C's type system declines to have an opinion:
///   * THREE dropped `sizeof`s. `sizeof(STARTUPINFOW)` became `size_of_val(&STARTUPINFOW)`, and
///     both `cbSize != sizeof(SHELLEXECUTEINFO{A,W})` entry gates lost the `sizeof` outright —
///     that comparison is what validates a caller's struct before any field of it is read.
///   * The `SHELL_ExecuteW32` typedef was `extern "system"` where neither the C typedef nor
///     `SHELL_ExecuteW` carries WINAPI. stdcall vs cdecl: on i386, caller and callee would have
///     disagreed about who pops the arguments.
///   * `SUCCEEDED(hr)` translated to `(hr as u32) >= 0`, which is unconditionally true, so a
///     FAILED `AssocQueryStringW` still selected the "runas" verb.
///   * `goto done` became a self-referential `'done: { break 'done; }` — a jump to itself, so
///     the runas-cancelled path fell through and overwrote its `retval = 33` with an error.
///   * `GetUIObjectOf(.., addr_of_mut!(cm))` took the address of the out-parameter instead of
///     passing it, so `shellex_get_contextmenu` never filled its caller's pointer.
///   * `RealShellExecuteA`/`ExA` (SHELL32.265/266) had HWND, PHANDLE and HINSTANCE flattened to
///     `isize` while their W twins were typed correctly.
/// `UrlIsFileUrlW` was one more phantom symbol: shlwapi.h:643 `#define`s it to
/// `UrlIsW(x, URLIS_FILEURL)`, so no export of that name exists to link against.
pub mod shlexec;
pub mod shlfileop;
