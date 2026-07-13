//! COBJMACROS call helpers, cfg-selected per arch.
//!
//! These MUST be generated per-arch (tools/gen_commacros.py, once per bindgen_<arch>.rs).
//! It is not enough that the wrapper body is ABI-agnostic: a method whose PARAMETER is
//! itself a function pointer — `IViewObject::Draw`'s `pfnContinue` callback is the one
//! that caught us — carries that callback's calling convention in its TYPE, and bindgen
//! spells it per-arch (`extern "stdcall"` on i386, `extern "C"` elsewhere). Sharing one
//! amd64-generated file breaks the i386 build with
//! `expected "stdcall" fn, found "C" fn`. Same reason com-sys ships three bindgen files.
#[cfg_attr(target_arch = "x86", path = "commacros_i386.rs")]
#[cfg_attr(target_arch = "x86_64", path = "commacros_amd64.rs")]
#[cfg_attr(target_arch = "aarch64", path = "commacros_arm64.rs")]
mod generated;

pub use generated::*;
