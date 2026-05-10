//! ob subsystem — translated from C.

pub mod devicemap;
pub mod obdir;
pub mod obhandle;
pub mod obinit;
pub mod oblife;
pub mod oblink;
pub mod obname;
pub mod obref;
pub mod obsdcach;
pub mod obsecure;
pub mod obwait;

// Folded back 2026-05-09 from per-subsystem rlibs (pilot validated
// the captures(none) firewall is sufficient).
pub mod inlines;
