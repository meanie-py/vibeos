//! csrsrv.dll — the CSR server runtime: process/thread tracking,
//! the LPC API dispatch loop, sessions, the server-DLL loader.
//! 35 csrsrv.spec exports (the byte-for-byte ABI contract consumed
//! by csrss.exe + basesrv + winsrv). Translated Phase 2; see
//! docs/engineering/smss-csrss-translation-audit.md.

pub mod api;
pub mod init;
pub mod procsup;
pub mod server;
pub mod session;
pub mod srv_inlines;
pub mod thredsup;
pub mod wait;
