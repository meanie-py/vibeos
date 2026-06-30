//! msafd #define consts / type aliases / macro shims the substrate (`types`) and
//! ntdll-sys don't provide. Winsock/FD/MSG constant values match sdk/include/psdk/
//! winsock2.h; the wine debug macros (TRACE/ERR/WARN/FIXME) are release-noop, matching
//! the C build (DPRINT-style, compiled out). FIELD_OFFSET/NT_SUCCESS are the SDK macros
//! the C source uses as expressions.

// --- wine debug channel macros (release no-op) ------------------------------
// The C uses TRACE/ERR/WARN/FIXME(fmt, ...) as statement calls; under the ReactOS
// release build they compile to nothing. Make them no-op macros (faithful + no link
// symbol). #[macro_use] on `mod glue` puts them in scope crate-wide. The translation
// also emitted `extern "C" { fn TRACE(...) }` decls in some files — those are deleted
// per-file during integration (the macro supersedes them).
#[macro_export]
macro_rules! TRACE { ($($arg:tt)*) => {{}}; }
#[macro_export]
macro_rules! ERR { ($($arg:tt)*) => {{}}; }
#[macro_export]
macro_rules! WARN { ($($arg:tt)*) => {{}}; }
#[macro_export]
macro_rules! FIXME { ($($arg:tt)*) => {{}}; }

// FIELD_OFFSET(type, field) — byte offset of a struct field (SDK macro used as a
// const expression in IOCTL buffer-size math). offset_of! is the Rust equivalent.
#[macro_export]
macro_rules! FIELD_OFFSET {
    ($ty:ty, $field:tt) => { core::mem::offset_of!($ty, $field) };
}

// NT_SUCCESS(Status) — Status >= 0 (top bit clear). The C macro used as a bool expr.
#[inline(always)]
pub fn NT_SUCCESS(status: i32) -> bool { status >= 0 }

// --- winsock2.h constants (values verified vs sdk/include/psdk/winsock2.h) ----
pub const SOCKET_ERROR: i32 = -1;
pub const INVALID_SOCKET: usize = !0usize; // (SOCKET)(~0)

// FD_* network-event bitmasks (1 << FD_*_BIT)
pub const FD_READ: u32 = 1;
pub const FD_WRITE: u32 = 2;
pub const FD_OOB: u32 = 4;
pub const FD_ACCEPT: u32 = 8;
pub const FD_CONNECT: u32 = 16;
pub const FD_CLOSE: u32 = 32;

// MSG_* send/recv flags
pub const MSG_OOB: u32 = 1;
pub const MSG_PEEK: u32 = 2;
pub const MSG_DONTROUTE: u32 = 4;
pub const MSG_WAITALL: u32 = 8;
pub const MSG_PARTIAL: u32 = 0x8000;
