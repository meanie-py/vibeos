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
//     shldataobject.rs   26     iconcache.rs       94     shlfolder.rs      112
//     shlfileop.rs      227     brfolder.rs       235     shlexec.rs        415
//
// Each carries an `//! ORPHAN-BY-DESIGN:` banner. PAID artifacts — do not delete them to tidy up.
//
// pub mod brfolder;
// pub mod iconcache;
// pub mod shldataobject;
// pub mod shlexec;
// pub mod shlfileop;
// pub mod shlfolder;
