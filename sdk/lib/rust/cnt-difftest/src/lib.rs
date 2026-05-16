//! Differential harness scaffold. Reference FFI decls are filled in by
//! build.rs (cc-compiled isolated libcntpr logic) + tests/ — see
//! task 263. This lib exists so the workspace resolves; the real
//! cross-checks live in `tests/`.

// The scanf C trampoline (scanf_trampoline.c, linked via build.rs as a
// trailing object) calls these `#[no_mangle]` engine entry points from
// cnt-rust's rlib. Nothing references them from Rust, so without an
// anchor the rlib archive member isn't pulled before the trailing
// object and the link fails (`undefined symbol: cnt_vsscanf`). This
// `#[used]` anchor — compiled into the cnt-difftest lib, which both
// the lib test and the `diff` integration test link — forces the
// members in early. (The trampoline owns the variadic ABI Rust can't
// express on stable; see cabi/manual.rs.)
#[used]
static _SCANF_ANCHOR_A: unsafe extern "C" fn(
    *const u8,
    *const u8,
    *mut core::ffi::c_void,
) -> i32 = cnt_rust::cabi::cnt_vsscanf;
#[used]
static _SCANF_ANCHOR_W: unsafe extern "C" fn(
    *const u16,
    *const u16,
    *mut core::ffi::c_void,
) -> i32 = cnt_rust::cabi::cnt_vswscanf;
#[used]
static _SCANF_ANCHOR_NA: unsafe extern "C" fn(
    *const u8,
    usize,
    *const u8,
    *mut core::ffi::c_void,
) -> i32 = cnt_rust::cabi::cnt_vsnscanf;
#[used]
static _SCANF_ANCHOR_NW: unsafe extern "C" fn(
    *const u16,
    usize,
    *const u16,
    *mut core::ffi::c_void,
) -> i32 = cnt_rust::cabi::cnt_vsnwscanf;
