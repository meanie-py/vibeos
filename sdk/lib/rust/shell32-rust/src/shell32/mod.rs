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

// ---------------------------------------------------------------------------------------------
// STILL PARKED. The C++ SUBSTRATE that blocked them is now largely built: `crate::atl` supplies
// CComPtr/CHeapPtr/CComHeapPtr/CSimpleArray and `crate::shellutils` the shellutils.h/utils.h
// header inlines, so what is left per file is ordinary integration work rather than a missing
// foundation. Counts measured with all six wired, after the substrate landed:
//
//     shlexec.rs   349     shlfileop.rs  227     iconcache.rs   59
//     brfolder.rs  244     shlfolder.rs    0     shldataobject   0  <- flipped
//
// Read those against the pre-substrate numbers (shldataobject 22, iconcache 68, shlfolder 75,
// shlfileop 227, brfolder 235, shlexec 415) and note that brfolder went UP, 235 -> 244: making
// the ATL types resolve does not only remove errors, it also un-masks the ones that were hiding
// behind an unresolved name. A rising count on a file is progress, not regression.
//
// SHLFOLDER COMPILES CLEAN on all three arches and is still parked, which is a DIFFERENT
// blocker from the rest — read this before trying to flip it. `shlfolder.cpp` exports two
// OVERLOADS of `SHELL32_CoCreateInitSF` (a 7-parameter and a 5-parameter form; see
// `llvm-nm --defined-only` on the .obj), and several of its exports take C++ `const GUID&`
// reference parameters. `EXTERN_C` cannot express an overload SET — one name, one symbol — and
// the translation produced a single `SHELL32_CoCreateInitSF`, so the other overload's callers
// would lose their definition. Meanwhile the Rust module's `#[no_mangle]` exports (SHCreateLinks,
// SHELL32_CompareDetails, every `*_RustBridge`) collide with the .cpp's identical unmangled
// symbols, so it cannot be left wired while the .cpp compiles either. The resolution is a HYBRID
// cohab — retain the overload pair (and the reference-taking helpers) in C++, flip the rest —
// which is a deliberate split, not a mechanical edit, and is owed rather than done.
//
// What each still needs, in cheapest-first order:
//
//   * iconcache (59) — cast and signature integration plus a handful of one-offs: a function-
//     local `static int s_imgListIdx` the translation dropped, the nested-anon-union
//     `RTL_CRITICAL_SECTION_DEBUG` initialiser, `IID_NULL_PPV_ARG` (a macro that expands to
//     THREE arguments, so its call sites have to be rewritten rather than macro-ised), and
//     `MAKEROP4`/`ImageList_AddIcon` for the prelude.
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
// pub mod iconcache;
// pub mod shlexec;
// pub mod shlfileop;
// pub mod shlfolder;
