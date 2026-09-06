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
//! Stable Rust can't DEFINE a variadic, so `printf` declares the trailing
//! slots as fixed parameters and replays them into freeldr.sys's TuiPrintf --
//! see `va_slots` for the technique. cdecl caller-cleans, so the arity is
//! invisible to callers, and `%s`/`%u` now substitute rather than printing
//! literally.

use ntkrnl_sys::base::*;
type PCSTR = *const i8;

extern "C" {
    // TuiPrintf is exported by freeldr.sys (freeldr.spec `@ cdecl TuiPrintf()`).
    // Declared variadic: stable Rust cannot DEFINE one but may DECLARE one, and
    // `spec2def` emits a plain undecorated cdecl name with no relay thunk, so
    // the declared arg count in the .spec is documentation only.
    fn TuiPrintf(Format: PCSTR, ...) -> i32;
    // MachConsPutChar is NOT in freeldr.spec, so puts can't trivially fan
    // out to it from rosload. Forward to TuiPrintf instead and let it
    // route through the Mach vtable that freeldr.sys owns.
}

#[no_mangle]
pub unsafe extern "C" fn printf(
    Format: PCSTR,
    a0: usize, a1: usize, a2: usize, a3: usize, a4: usize, a5: usize,
    a6: usize, a7: usize, a8: usize, a9: usize, a10: usize, a11: usize,
) -> i32 {
    TuiPrintf(Format, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11)
}

#[no_mangle]
pub unsafe extern "C" fn puts(Str: PCSTR) -> i32 {
    // `Str` is DATA, never a format. Passing it as TuiPrintf's format string was
    // harmless only while TuiPrintf printed verbatim; now that it expands, a
    // string containing `%` would consume slots nobody pushed. `%s` also gets
    // puts's mandatory trailing newline in the same call.
    TuiPrintf(b"%s\n\0".as_ptr() as PCSTR, Str)
}
