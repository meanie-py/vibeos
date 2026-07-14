//! com-sys — bindgen-generated COM ABI declarations for user-mode Rust DLLs.
//!
//! One flat module of `#[repr(C)]` interface + vtbl structs (IUnknown, IStream,
//! IDataObject, IMoniker, ITypeInfo, IDispatch, IConnectionPoint, … — ~239
//! interfaces), IID/CLSID types, and the automation data types (VARIANT,
//! SAFEARRAY, TYPEATTR, FUNCDESC, PROPVARIANT, FORMATETC, STGMEDIUM, …),
//! generated per-arch from the real SDK headers (objbase/objidl/oleidl/
//! servprov/oaidl/ocidl/propidl) so layouts and calling conventions can't
//! drift from the C: i386 vtbl slots are `extern "stdcall"`, amd64/arm64 are
//! `extern "C"` — exactly what STDMETHODCALLTYPE compiles to on each target.
//!
//! Regenerate with `./generate_bindings.sh` after SDK COM-header changes;
//! bindings are checked in and diff-reviewed like ntkrnl-sys Layer 2.
#![no_std]
#![allow(
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    dead_code,
    improper_ctypes,
    clippy::all
)]

#[cfg(target_arch = "x86_64")]
include!("bindgen_amd64.rs");

#[cfg(target_arch = "x86")]
include!("bindgen_i386.rs");

#[cfg(target_arch = "aarch64")]
include!("bindgen_arm64.rs");
