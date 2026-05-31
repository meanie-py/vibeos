//! csrlib — client-side CSR API + capture-buffer marshalling.
//! The LPC protocol contract every CSR server validates against
//! (the user-mode analog of the kernel probe surface). Translated
//! Phase 1; see docs/engineering/smss-csrss-translation-audit.md.

pub mod api;
pub mod capture;
pub mod connect;
