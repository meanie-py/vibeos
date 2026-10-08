//! The IID/CLSID data symbols the ring modules reference, DECLARED (not defined).
//!
//! These are real link-time symbols already present in ole32.dll's link:
//!   * interface IIDs  → the `uuid` static lib (`nm libuuid.a` → `R IID_IMalloc`, …)
//!   * moniker CLSIDs  → ole32's own `guid.c` (an `#include <initguid.h>` TU we keep
//!                       compiled precisely so it keeps emitting these)
//!
//! We must NOT redefine them in Rust. Hand-transcribing a GUID is a silent-corruption
//! source: a wrong byte produces no compile or link error, it just makes
//! QueryInterface miss and the object mysteriously "not work". (Contrast oleaut32-rust,
//! where flipping oleaut.c out DELETED the only definition site for the proxy CLSIDs
//! and we had to become their definition site — here the C definition site survives.)
//!
//! Data statics carry no calling convention, so `extern "C"` is exact on every arch.
#![allow(clippy::all)]
use crate::prelude::GUID;

extern "C" {
    // --- core ---
    pub static IID_IUnknown: GUID;
    pub static IID_IClassFactory: GUID;
    pub static IID_IMalloc: GUID;
    pub static IID_IMallocSpy: GUID;
    pub static IID_IMarshal: GUID;
    pub static IID_IMultiQI: GUID;

    // --- streams / storage ---
    pub static IID_ISequentialStream: GUID;
    pub static IID_IStream: GUID;
    pub static IID_IStorage: GUID;
    pub static IID_ILockBytes: GUID;
    pub static IID_IPersist: GUID;
    pub static IID_IPersistStream: GUID;
    pub static IID_IPersistStorage: GUID;
    pub static IID_IPersistFile: GUID;

    // --- monikers / binding ---
    pub static IID_IMoniker: GUID;
    pub static IID_IBindCtx: GUID;
    pub static IID_IRunningObjectTable: GUID;
    pub static IID_IROTData: GUID;
    pub static IID_IEnumMoniker: GUID;
    pub static IID_IEnumString: GUID;
    pub static IID_IEnumUnknown: GUID;
    pub static IID_IParseDisplayName: GUID;

    // --- error info ---
    pub static IID_IErrorInfo: GUID;
    pub static IID_ICreateErrorInfo: GUID;
    pub static IID_ISupportErrorInfo: GUID;

    // --- data transfer / OLE ---
    pub static IID_IDataObject: GUID;
    pub static IID_IAdviseSink: GUID;
    pub static IID_IEnumFORMATETC: GUID;
    pub static IID_IEnumSTATDATA: GUID;
    pub static IID_IDropTarget: GUID;
    pub static IID_IDropSource: GUID;
    pub static IID_IOleObject: GUID;
    pub static IID_IOleClientSite: GUID;

    // --- moniker CLSIDs (ole32's guid.c) ---
    pub static CLSID_FileMoniker: GUID;
    pub static CLSID_ItemMoniker: GUID;
    pub static CLSID_AntiMoniker: GUID;
    pub static CLSID_CompositeMoniker: GUID;
    pub static CLSID_ClassMoniker: GUID;
    pub static CLSID_PointerMoniker: GUID;
    pub static CLSID_ObjrefMoniker: GUID;
    pub static CLSID_DfMarshal: GUID;

    // --- Tier-D: storage / property-set / OLE-embedding interfaces ---
    // Every one of these is VERIFIED present in build-*/sdk/lib/uuid/libuuid.a (which ole32
    // already links), so these are declarations of a real link symbol — never a redefinition.
    // Defining a GUID in Rust instead would be a silent-corruption source: one wrong byte and
    // QueryInterface simply misses, with no compile error and no link error.
    pub static IID_IEnumSTATSTG: GUID;
    pub static IID_IDirectWriterLock: GUID;
    pub static IID_IPropertySetStorage: GUID;
    pub static IID_IPropertyStorage: GUID;
    pub static IID_IEnumSTATPROPSETSTG: GUID;
    pub static IID_IEnumSTATPROPSTG: GUID;
    pub static IID_IViewObject: GUID;
    pub static IID_IViewObject2: GUID;
    pub static IID_IOleCache: GUID;
    pub static IID_IOleCache2: GUID;
    pub static IID_IOleCacheControl: GUID;
    pub static IID_IOleAdviseHolder: GUID;
    pub static IID_IDataAdviseHolder: GUID;
    pub static IID_IRunnableObject: GUID;

    // --- property-set format IDs (objidlbase.idl) ---
    pub static FMTID_SummaryInformation: GUID;
    pub static FMTID_DocSummaryInformation: GUID;
    pub static FMTID_UserDefinedProperties: GUID;

    // --- static-picture CLSIDs the data cache stamps into the presentation stream (cguid.h) ---
    pub static CLSID_Picture_Dib: GUID;
    pub static CLSID_Picture_Metafile: GUID;
    pub static CLSID_Picture_EnhMetafile: GUID;
}
