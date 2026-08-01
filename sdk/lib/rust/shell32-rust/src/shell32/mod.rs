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
// STILL PARKED. The C++ SUBSTRATE that blocked them is now largely built: `crate::atl` supplies
// CComPtr/CHeapPtr/CComHeapPtr/CSimpleArray and `crate::shellutils` the shellutils.h/utils.h
// header inlines, so what is left per file is ordinary integration work rather than a missing
// foundation. Counts measured with all six wired, after the substrate landed:
//
//     shlexec.rs   102     shlfileop.rs  227     iconcache.rs    0 <- flipped
//     brfolder.rs    0 <- flipped     shlfolder.rs    0 <- flipped     shldataobject 0 <- flipped
//
// shlexec went 349 -> 102 on 2026-07-31/08-01, and its recorded blocker was WRONG: it has ZERO
// `CStringW::Format` calls, so it never needed the printf-variadic decision at all (that is
// shlfileop's, 7 sites). What it needed was the plain string type — `crate::atl::CStringW`,
// LocalAlloc-backed, ported operation-by-operation from `sdk/lib/atl/cstringt.h` — plus the
// `shlexec.cpp` name residue, all of it now landed and shared: the wine/unicode.h A/W aliases
// (`strcpyW`/`strstrW`/`strrchrW`/`strncmpiW`/`isspaceW`), the CRT wide fns and
// `HKEY_LOCAL_MACHINE` in `island_extra`, `SeeFlagsToCmicFlags` + `__SHCloneStrW{,toA}` in
// `shellutils` (all three are HEADER inlines, so they reach no TU), a dozen shlwapi/shell32
// externs in the prelude, and an `lpVtbl`-shaped `IShellExtInit` in `shellcom`.
// What is LEFT is this file's own tail, ~102 errors: mostly pointer/handle conversions
// (`int`->`*mut`, `&ref`->raw) that a bulk numeric pass must NOT touch, plus the arch-DIVERGENT
// numeric sites. On those last ones do what `crate-repair` says and name the semantic alias
// (SIZE_T / UINT_PTR / INT_PTR / LONG_PTR): `as usize` is u32 on i386 and u64 elsewhere, so
// either literal width is wrong on some arch. No shared substrate remains, so it blocks nothing.
//
// If you bulk-apply casts from rustc spans, GUARD MATCH PATTERNS. A cast is not legal in a
// pattern; `crate-repair` declines those by design and a hand-rolled pass that forgets to
// silently writes `(SE_ERR_FNF) as usize => ...`, which does not compile. Cast the SCRUTINEE.
//
// brfolder went 244 -> 86 on 2026-07-31: every SUBSTRATE gap it had is now closed and landed
// (14 more TreeView_* wrappers and `TV_ITEM` in the prelude, `crate::layout` for the whole of
// `ui/layout.h`, `SHELL_IsContextMenuMsg` / `SHILClone` / `RosGetProcessEffectiveVersion` in
// `shellutils`, `Deref`+`Get`+`GetAddressOf`+`put` on the ATL pointers, and real signatures
// for `IShellFolderVtbl::SetNameOf` and `IEnumIDListVtbl::{Skip,Reset}` plus an
// `lpVtbl`-shaped `IContextMenu` in `shellcom`). What is LEFT is only this file's own numeric
// and pointer casts — no shared substrate, so it blocks nothing else. `rustconv crate-repair`
// declines the remainder by design (78 are `match`-pattern casts), so they are a hand pass.
//
// WARNING for whoever runs `crate-repair` on this crate: **it edits shared MACRO DEFINITIONS
// to satisfy errors reported at their EXPANSION sites, once per call site.** Seen twice, on two
// different macros:
//   * `_countof!` — rewritten `u32` -> `i32` plus compensating casts in three ALREADY-LANDED
//     files. C's `sizeof(a)/sizeof(*a)` is unsigned.
//   * `ARRAY_SIZE!` — stacked into
//     `(((((((($a).len()) as u32) as usize) as u32) as usize) as u32) as usize) as u32`
//     while repairing shlexec (2026-07-31).
// Revert the macro every time and cast at the call site instead. The test that it was pure
// blast radius: re-run `cargo check` after reverting — the error count does not move (it did
// not, on either occasion).
//
// Read those against the pre-substrate numbers (shldataobject 22, iconcache 68, shlfolder 75,
// shlfileop 227, brfolder 235, shlexec 415) and note that brfolder went UP, 235 -> 244: making
// the ATL types resolve does not only remove errors, it also un-masks the ones that were hiding
// behind an unresolved name. A rising count on a file is progress, not regression.
//
// What each still needs, in cheapest-first order:
//
//   * shlfileop / shlexec (227 / 349) — both additionally want `CStringW`, which `atl.rs`
//     deliberately does NOT yet provide. Its measured surface is new()/Left/GetLength/IsEmpty/
//     SetString/GetBuffer/ReleaseBuffer/as_ptr/`+=` plus 14 `Format` calls. `Format` is
//     printf-variadic, and stable Rust cannot DEFINE a variadic fn — it needs a trampoline in
//     `dll/win32/shell32/rust_varargs_shim.c`, the same mechanism ShellMessageBoxA already uses.
//     Designing CStringW before that decision is made would be guesswork, so it waits.
//
// Each carries an `//! ORPHAN-BY-DESIGN:` banner. PAID artifacts — do not delete them to tidy up.
//
/// brfolder.cpp — the SHBrowseForFolder dialog. FLIPPED 2026-07-31: 86 -> 0. The residue was
/// this file's own casts plus four defects a green C build hides: amd64 widths baked into
/// `LRESULT`/`UINT_PTR`/`SIZE_T` casts, a C++ `!` turned into Rust's BITWISE not, one of two
/// `BrFolder_GetName` OVERLOADS dropped, and a `COleInit` RAII local left uninitialised so
/// neither OleInitialize nor OleUninitialize ran.
pub mod brfolder;
// pub mod shlexec;
// pub mod shlfileop;
