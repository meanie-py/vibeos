//! Rosload-only stub shims.
//!
//! For freeldr.sys (BIOS) and uefildr.efi (UEFI), `printf`/`puts` live in
//! `tui.rs` (they fan out into the Tui*/MachConsPutChar chain). The rosload
//! variant gates `tui.rs` out (the UI module is owned by freeldr.sys and
//! imported through `add_importlibs(rosload freeldr)`), so the same shims
//! have to be republished from a module that IS built when
//! `feature = "rosload"` is on. The implementations forward to TuiPrintf,
//! exported by freeldr.sys via freeldr.spec — exactly the same chain as
//! tui.rs's printf, just with the indirection crossing the importlib.
//!
//! Both shims are non-variadic by design (Rust stable can't define a
//! variadic fn; see the same degradation in tui.rs printf / keystone TRACE).
//! cdecl caller-cleans → ABI-safe regardless of caller's arg count, only
//! the *format* sees its `%s`/`%u` substitutions skipped — same trade-off
//! freeldr.sys/uefildr already accept.

use ntkrnl_sys::base::*;
type PCSTR = *const i8;

extern "C" {
    // TuiPrintf is exported by freeldr.sys (freeldr.spec `@ cdecl TuiPrintf()`).
    fn TuiPrintf(Format: PCSTR) -> i32;
    // MachConsPutChar is NOT in freeldr.spec, so puts can't trivially fan
    // out to it from rosload. Forward to TuiPrintf instead and let it
    // route through the Mach vtable that freeldr.sys owns.
}

#[no_mangle]
pub unsafe extern "C" fn printf(Format: PCSTR) -> i32 {
    TuiPrintf(Format)
}

#[no_mangle]
pub unsafe extern "C" fn puts(Str: PCSTR) -> i32 {
    let r = TuiPrintf(Str);
    // Approximate puts's trailing newline (puts always appends '\n'); the
    // verbatim-format degradation means we can't tack on a "\n" via printf
    // without a fresh format buffer, so we fan to TuiPrintf("\n").
    TuiPrintf(b"\n\0".as_ptr() as PCSTR);
    r
}
