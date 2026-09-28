//! `sdk/lib/ucrt/misc/*.cpp` -- the grab-bag: errno/strerror/perror, cwd and
//! drives, signal/terminate/exception-filter, beep/sleep, pid, systime, and a
//! handful of one-function files. Replaces every file in `UCRT_MISC_SOURCES`
//! (`misc/misc.cmake`) except `invalid_parameter.cpp` (already Rust,
//! `crate::invalid_parameter`).
//!
//! Split one submodule per C file, or per closely related pair, matching the
//! deletion granularity in the batch report.

pub mod chdir;
pub mod drive;
pub mod errno;
pub mod exception_filter;
pub mod getcwd;
pub mod perror;
pub mod signal;
pub mod small;
pub mod strerror;
pub mod syserr;
pub mod terminate;
