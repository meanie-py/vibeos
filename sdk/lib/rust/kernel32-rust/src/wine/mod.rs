//! wine/ — Wine-derived kernel32 helpers, ported to Rust (rustconv c2rust).
pub mod muldiv;
pub mod timezone;
pub mod actctx;
// DEFERRED (subsystem FFI types / heavy type errors; .rs preserved on disk, .c stays C):
//   comm (SERIAL_*/IOCTL_SERIAL_*), profile (INI + CurProfile state),
//   res (Wine resource-tree, 200+ residual type errors — .c is the active provider)
pub mod lzexpand;
// pub mod comm;
pub mod profile;
// pub mod res;
