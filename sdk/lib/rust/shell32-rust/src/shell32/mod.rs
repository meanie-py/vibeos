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
//     shlexec.rs   349     shlfileop.rs  227     iconcache.rs    0 <- flipped
//     brfolder.rs  244     shlfolder.rs    0 <- flipped     shldataobject 0 <- flipped
//
// Read those against the pre-substrate numbers (shldataobject 22, iconcache 68, shlfolder 75,
// shlfileop 227, brfolder 235, shlexec 415) and note that brfolder went UP, 235 -> 244: making
// the ATL types resolve does not only remove errors, it also un-masks the ones that were hiding
// behind an unresolved name. A rising count on a file is progress, not regression.
//
// What each still needs, in cheapest-first order:
//
//   * brfolder (244) — cast/signature integration at volume, plus TreeView_* macro residue.
//   * shlfileop / shlexec (227 / 349) — both additionally want `CStringW`, which `atl.rs`
//     deliberately does NOT yet provide. Its measured surface is new()/Left/GetLength/IsEmpty/
//     SetString/GetBuffer/ReleaseBuffer/as_ptr/`+=` plus 14 `Format` calls. `Format` is
//     printf-variadic, and stable Rust cannot DEFINE a variadic fn — it needs a trampoline in
//     `dll/win32/shell32/rust_varargs_shim.c`, the same mechanism ShellMessageBoxA already uses.
//     Designing CStringW before that decision is made would be guesswork, so it waits.
//
// Each carries an `//! ORPHAN-BY-DESIGN:` banner. PAID artifacts — do not delete them to tidy up.
//
// pub mod brfolder;
// pub mod shlexec;
// pub mod shlfileop;
