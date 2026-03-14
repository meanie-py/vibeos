//! lpc subsystem — translated from C.

pub mod close;
pub mod complete;
// pub mod connect;  // Using C version to isolate LLVM codegen bug
pub mod create;
pub mod listen;
pub mod port;
pub mod reply;
pub mod send;
