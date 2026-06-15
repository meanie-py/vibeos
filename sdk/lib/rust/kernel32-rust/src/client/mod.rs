//! client/ — client-side Win32 API surface of kernel32, ported to Rust (rustconv c2rust).
//! Submodules added as each client/*.c is translated + integrated.
pub mod version;
pub mod handle;
pub mod atom;
pub mod environ;
pub mod path;
