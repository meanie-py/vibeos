//! client/console/ — console client APIs, ported to Rust (rustconv c2rust).
//! DEFERRED: the whole console family depends on the CSR CONSOLE_API_MESSAGE type
//! hierarchy (~80 message structs) which is not yet in the FFI layer. The .rs
//! translations are preserved on disk; the .c files remain active C in CMakeLists.
//! Re-enable these (and `pub mod console;` in lib.rs) once the CSR console message
//! types are defined.
// pub mod alias;
// pub mod console;
// pub mod history;
// pub mod init;
// pub mod readwrite;
