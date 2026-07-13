//! The ole32 "mechanical ring" — the rustconv-translated TUs that sit on top of the
//! compobj/marshal core: the task allocator, streams, the moniker family, the bind
//! context, error info, the enumerator helper, OLE2 init/drag-drop, the clipboard,
//! the component categories, and the NDR user-marshal helpers.
//!
//! Unlike compobj/marshal (which carry hand-rolled FFI surfaces), every module here
//! speaks `crate::prelude` — com-sys types, `crate::win32`/`gdi32`/`user32` for the
//! Win32 imports with the right per-arch ABI, `crate::crt` for the (genuinely cdecl)
//! CRT, `crate::commacros_*` for COBJMACRO vtbl calls, `crate::wine_list` for wine's
//! intrusive list, and `crate::ole32_exports` for ole32's own exports that still live
//! in un-flipped C TUs. See prelude.rs for why.
pub mod antimoniker;
pub mod bindctx;
pub mod classmoniker;
pub mod clipboard;
pub mod comcat;
pub mod compositemoniker;
pub mod dictionary;
pub mod enumx;
pub mod errorinfo;
pub mod filelockbytes;
pub mod filemoniker;
pub mod hglobalstream;
pub mod ifs;
pub mod itemmoniker;
pub mod memlockbytes;
pub mod moniker;
pub mod ole2;
pub mod ole2impl;
pub mod ole2stubs;
pub mod ole32_main;
pub mod oleproxy;
pub mod pointermoniker;
pub mod usrmarshal;
