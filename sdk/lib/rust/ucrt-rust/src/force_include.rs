//! `_CRT_LINKER_FORCE_INCLUDE(sym)`, for the Rust that replaced its C.
//!
//! UCRT registers startup and shutdown hooks as pointers in `.CRT$XIC` /
//! `.CRT$XPX`, one per file in `sdk/lib/ucrt/initializers/`. Nothing CALLS
//! those pointers by name, so nothing would pull their archive members into
//! ucrtbase -- the owning C file carries a `/include:<sym>` linker directive
//! instead. Delete that C file and the directive goes with it: the member is
//! never linked, and the hook silently stops running (timeset's never sets
//! `_timezone`, locale's never tears down).
//!
//! `force_include!(sym)` puts the directive's effect back: a `#[used]` static
//! holding the symbol's ADDRESS. The Rust archive is linked `--whole-archive`,
//! so the reference is always there and pulls the member exactly as
//! `/include:` did. `verify_ucrtbase_cohab.py` fails if any initializer ends up
//! referenced by nothing.

/// A symbol address, made `Sync` so it can live in a `static`.
#[repr(transparent)]
pub struct ForceInclude(pub *const core::ffi::c_void);
unsafe impl Sync for ForceInclude {}

#[macro_export]
macro_rules! force_include {
    ($sym:ident) => {
        const _: () = {
            extern "C" {
                static $sym: *const core::ffi::c_void;
            }
            #[used]
            static FORCE_INCLUDE: $crate::force_include::ForceInclude =
                $crate::force_include::ForceInclude(core::ptr::addr_of!($sym) as *const core::ffi::c_void);
        };
    };
}
