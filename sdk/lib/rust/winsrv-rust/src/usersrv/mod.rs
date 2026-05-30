//! usersrv/ — USER API server (CSR server DLL surface for user32
//! ExitWindowsEx / EndTask / hard-error message-box / shutdown
//! sequencing / debugger activation / shutdown reason logging).
//!
//! Self-contained CSR server (no consrv deps). Loaded by csrss
//! alongside basesrv via winsrv.dll's `UserServerDllInitialization`
//! spec entry.

pub mod harderror;
pub mod init;
pub mod register;
pub mod shutdown;
