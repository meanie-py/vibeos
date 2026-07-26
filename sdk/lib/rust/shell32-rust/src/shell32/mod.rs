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

// ---------------------------------------------------------------------------------------------
// STILL PARKED — but no longer for want of substrate. The island closed that: the batch went
// from 2,831 errors to 1,112 on the swap alone, and what remains in each file is the C++
// SUBSTRATE the translation leaned on, which this crate has no equivalent for yet — ATL smart
// pointers (`CComPtr`, `CComHeapPtr`), `CStringW`, `CSimpleArray`, and COM interface METHOD
// calls written in C++ syntax (`p->Release()`) instead of through the `lpVtbl` idiom the landed
// COM modules use. Per-file counts after the swap, so the next lane can pick the cheapest:
//
//     shldataobject.rs   20     iconcache.rs       68     shlfolder.rs       75
//     shlfileop.rs      227     brfolder.rs       235     shlexec.rs        415
//
// The first three came down from 26/94/112 once island_adopt grew its COM-dispatch rule and
// prelude.rs grew the OLE residue. Two further errors surface in prelude.rs itself when they
// are wired: `FAILED_UNEXPECTEDLY` applied to a `()`-valued expression, a translation artifact
// where C's `FAILED_UNEXPECTEDLY(hr = Call())` lost the inner assignment. These three are the
// cheapest next targets and are no longer blocked on anything structural — what is left is the
// long tail of ATL types and shell32-internal helpers, not a wall.
//
// Each carries an `//! ORPHAN-BY-DESIGN:` banner. PAID artifacts — do not delete them to tidy up.
//
// pub mod brfolder;
// pub mod iconcache;
// pub mod shldataobject;
// pub mod shlexec;
// pub mod shlfileop;
// pub mod shlfolder;
