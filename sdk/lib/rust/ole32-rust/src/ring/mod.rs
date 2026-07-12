//! The ole32 "mechanical ring" — the rustconv-translated TUs that sit on top of the
//! compobj/marshal core: the task allocator, streams, the moniker family, the bind
//! context, error info, the enumerator helper, OLE2 init/drag-drop, the clipboard,
//! and the NDR user-marshal helpers.
//!
//! Unlike compobj/marshal (which carry hand-rolled FFI surfaces), every module here
//! speaks `crate::prelude` — com-sys types, `crate::win32` for the Win32 imports with
//! the right per-arch ABI, `crate::commacros_gen` for COBJMACRO vtbl calls, and
//! `crate::wine_list` for wine's intrusive list. See prelude.rs for why.
pub mod bindctx;
pub mod dictionary;
pub mod enumx;
pub mod errorinfo;
pub mod hglobalstream;
pub mod ifs;
pub mod ole2stubs;
pub mod ole32_main;
