//! The IID/CLSID data symbols the Tier-D modules reference, DECLARED (not defined).
//!
//! These are real link-time symbols already in oleaut32's link: `dll/win32/oleaut32/
//! CMakeLists.txt:119` links the `uuid` static lib, and every symbol below was verified
//! present with `llvm-nm --defined-only build-i386/sdk/lib/uuid/libuuid.a` (they appear as
//! `_IID_IFont`, `_CLSID_StdFont`, … — the i386 leading-underscore form).
//!
//! We must NOT redefine them in Rust. Hand-transcribing a GUID is a silent-corruption
//! source of the worst kind: a single wrong byte produces no compile error and no link
//! error — QueryInterface simply misses and the object mysteriously "doesn't work". Same
//! rationale (and same shape) as `ole32-rust/src/iids.rs`; read that file's header for the
//! one case where the C definition site is deleted and Rust *must* become the definer.
//! That is NOT the case here: oleaut32 keeps linking `uuid`, so the C definitions survive.
//!
//! Data statics carry no calling convention, so `extern "C"` is exact on every arch — the
//! stdcall/cdecl distinction that governs FUNCTION imports simply does not apply to data.
//!
//! Deliberately NOT declared here: `IID_NULL`, `IID_IUnknown` and `IID_IDispatch`, which
//! `winemacros.rs` already defines as Rust statics. Re-declaring them would be an
//! ambiguous-glob error (E0659) once both are re-exported through the prelude.
#![allow(clippy::all)]
use crate::prelude::GUID;

extern "C" {
    // --- the null GUID (guiddef.h / uuid) ---
    pub static GUID_NULL: GUID;

    // --- core COM (connpt.rs, olefont.rs, olepicture.rs class factories) ---
    pub static IID_IClassFactory: GUID;

    // --- connection points (connpt.rs, olefont.rs, olepicture.rs) ---
    pub static IID_IConnectionPoint: GUID;
    pub static IID_IConnectionPointContainer: GUID;
    pub static IID_IEnumConnections: GUID;
    pub static IID_IPropertyNotifySink: GUID;

    // --- persistence (olefont.rs, olepicture.rs) ---
    pub static IID_IPersist: GUID;
    pub static IID_IPersistStream: GUID;
    pub static IID_IPersistStreamInit: GUID;
    pub static IID_IPersistPropertyBag: GUID;
    pub static IID_IStream: GUID;

    // --- OLE controls: font (olefont.rs) ---
    pub static CLSID_StdFont: GUID;
    pub static IID_IFont: GUID;
    pub static IID_IFontDisp: GUID;
    pub static IID_IFontEventsDisp: GUID;

    // --- OLE controls: picture (olepicture.rs) ---
    pub static CLSID_StdPicture: GUID;
    pub static IID_IPicture: GUID;
    pub static IID_IPictureDisp: GUID;

    // --- property frame (olepropframe.rs) ---
    pub static IID_IPropertyPage: GUID;
    pub static IID_IPropertyPageSite: GUID;

    // --- record info (recinfo.rs) ---
    pub static IID_IRecordInfo: GUID;
}
