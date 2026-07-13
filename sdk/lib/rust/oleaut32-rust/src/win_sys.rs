//! win_sys — the non-COM Win32 ABI island. GENERATED; see `generate_win_bindings.sh`.
//!
//! com-sys covers the COM/automation headers only. The Tier-D modules also need
//! wincodec.h (olepicture's WIC decode path), olectl.h (PICTDESC/FONTDESC/OCPFIPARAMS),
//! prsht.h (PROPSHEETHEADERW/PROPSHEETPAGEW), wingdi.h (BITMAP/BITMAPFILEHEADER/
//! ENHMETAHEADER) and winuser.h (DLGTEMPLATE). Those are bindgen'd from the real headers
//! rather than hand-written: the callers `sizeof()` these structs and index their fields,
//! so a mis-transcribed member is a silent ABI break — no compile error, no link error.
//!
//! Types com-sys already provides (base scalars, HBITMAP/HDC/HWND/…, POINT/SIZE/RECT/
//! LOGFONTW/TEXTMETRICW, IUnknown/IStream) are blocklisted in the generator and re-imported
//! from com-sys, so the shared surface stays one nominal type across the boundary.
//!
//! **Deliberately NOT glob-exported into the prelude.** bindgen still re-emits a slice of
//! wincodec/prsht's transitive base surface that overlaps com-sys; globbing both would make
//! every overlapping name an ambiguous-glob error. `prelude.rs` re-exports the specific
//! names Tier-D references, by name.
#![allow(clippy::all)]

#[cfg(target_arch = "x86_64")]
include!("win_sys_amd64.rs");

#[cfg(target_arch = "x86")]
include!("win_sys_i386.rs");

#[cfg(target_arch = "aarch64")]
include!("win_sys_arm64.rs");
