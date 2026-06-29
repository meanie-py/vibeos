//! The 5 translated msafd translation units (dll/win32/msafd/misc/*.c → Rust).
//! Cohab flip per file: each module's #[no_mangle] WSP*/Sock*/helper symbols take
//! over from the commented-out C. event.c is the async-select worker; stubs.c the
//! WSP entrypoints msafd leaves unimplemented.
pub mod dllmain;
pub mod event;
pub mod helpers;
pub mod sndrcv;
pub mod stubs;
