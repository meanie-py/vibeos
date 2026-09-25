//! Host differential harness for strsafe-rust's printf family; see Cargo.toml and tests/diff.rs.
//! The genuine strsafe.h wrappers and the shared engine live in csrc/, built by build.rs; they
//! are declared HERE so the test binary links the archive (an unreferenced dependency's native
//! library would be dropped).
pub type HRESULT = i32;
#[link(name = "strsafe_ref", kind = "static")]
extern "C" {
    pub fn ref_StringCchPrintfW(d: *mut u16, cch: usize, f: *const u16, a: i32, s: *const u16) -> HRESULT;
    pub fn ref_StringCbPrintfW(d: *mut u16, cb: usize, f: *const u16, a: i32, s: *const u16) -> HRESULT;
    pub fn ref_StringCbPrintfExW(d: *mut u16, cb: usize, end: *mut *mut u16, rem: *mut usize,
                                 flags: u32, f: *const u16, a: i32, s: *const u16) -> HRESULT;
    /// The shared engine's `_snwprintf`, which strsafe-rust's macros call.
    pub fn _snwprintf(buf: *mut u16, n: usize, fmt: *const u16, ...) -> core::ffi::c_int;
}
