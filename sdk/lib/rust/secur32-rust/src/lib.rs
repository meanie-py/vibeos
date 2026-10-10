//! secur32-rust — the secur32.dll C-to-Rust body translation.
//!
//! Cohab with the still-C secur32 .c files (commented out of
//! dll/win32/secur32/CMakeLists.txt as each flips). The `#[no_mangle]`
//! WINAPI/SEC_ENTRY/cdecl functions here are the sole providers of those exported
//! symbols once their C TU is commented out; the not-yet-flipped C bodies resolve
//! at link the other way (--allow-multiple-definition, lld first-match). The LSA
//! channel is NDK LPC (no RPC stubs). FFI substrate = ntdll-sys (NT base) +
//! secur32-sys (SSPI/LSA-LPC/NTLM-MSV/wincred declaration surface).
#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(unused_unsafe, dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(unused_assignments, static_mut_refs)]
#![allow(clippy::all)]

// Wine debug macros — no-ops in the NDEBUG build (TRACE/WARN/ERR/FIXME compile
// out in the C too). Defined at crate root before the body modules so textual
// macro scoping makes them visible to every translated file.
// Lowercase aliases — some translated files emit `trace!`/`warn!`/etc.
macro_rules! trace { ($($arg:tt)*) => {{}}; }
macro_rules! warn { ($($arg:tt)*) => {{}}; }
macro_rules! err { ($($arg:tt)*) => {{}}; }
macro_rules! fixme { ($($arg:tt)*) => {{}}; }

// Compile-time UTF-16 NUL-terminated literal: `wide!("NTLM")` → &'static [u16; N]
// (no alloc; the #![no_std] crate has no Vec). ASCII literals only.
#[macro_export]
macro_rules! wide {
    ($s:literal) => {{
        const B: &[u8] = $s.as_bytes();
        const N: usize = B.len() + 1;
        const W: [u16; N] = {
            let mut a = [0u16; N];
            let mut i = 0;
            while i < B.len() {
                a[i] = B[i] as u16;
                i += 1;
            }
            a
        };
        &W
    }};
}
// FIELD_OFFSET(type, field) → byte offset of `field` within `type` (usize).
#[macro_export]
macro_rules! FIELD_OFFSET {
    ($t:ty, $f:ident) => {{
        let u = core::mem::MaybeUninit::<$t>::uninit();
        let p = u.as_ptr();
        unsafe { (core::ptr::addr_of!((*p).$f) as usize) - (p as usize) }
    }};
}

// Build a SecurityFunctionTable* fn-pointer field from a SEC_ENTRY SSP function.
// SEC_ENTRY == WINAPI == stdcall on i386 / C on amd64+arm64, which is exactly what
// `extern "system"` lowers to — so the calling convention always matches. The
// bindgen fn-pointer fields, however, are spelled `extern "stdcall"`(i386) /
// `extern "C"`(elsewhere): on i386 the type-checker treats the "system" and
// "stdcall" ABI *tokens* as distinct even though the convention is identical, so
// we transmute the (identically-sized, identically-laid-out) fn pointer. This is
// behavior-preserving — it relabels the pointer's ABI token, not its convention.
macro_rules! sec_entry {
    ($f:expr) => {
        Some(unsafe { core::mem::transmute::<*const (), _>($f as *const ()) })
    };
}

#[macro_export]
macro_rules! wstr { ($s:literal) => { $crate::wide!($s) }; }
#[macro_export]
macro_rules! w { ($s:literal) => { $crate::wide!($s) }; }
#[macro_export]
macro_rules! L { ($s:literal) => { $crate::wide!($s) }; }
#[macro_export]
macro_rules! widestring { ($s:literal) => { $crate::wide!($s) }; }

// Foundation: the cohab import surface (NT base re-exports, SSPI/LSA-LPC/NTLM
// types + consts, Win32 helpers). Body modules add `use crate::prelude::*;`.
// Wine's ERR/FIXME/WARN/TRACE, shared by every Wine-derived crate.
#[macro_use]
extern crate winedebug_rust;

pub mod externs;
pub mod prelude;
pub mod types;

// USER-MODE no_std staticlib → own panic handler + eh_personality (deduped at
// link by --allow-multiple-definition, same as advapi32-rust/lsasrv-rust).
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[no_mangle]
extern "C" fn rust_eh_personality() {}

// ---- translated body modules (one per .c; declared as each file flips) ----
// The assembler namespaces by C parent dir: files directly under dll/win32/
// secur32/ land in src/secur32/, the wine SSPs in src/wine/.
// Wave 1 — leaf/small files.
#[path = "secur32/dllmain.rs"]
pub mod dllmain;
#[path = "secur32/stubs.rs"]
pub mod stubs;
#[path = "wine/hmac_md5.rs"]
pub mod hmac_md5;
#[path = "wine/base64_codec.rs"]
pub mod base64_codec;
#[path = "wine/util.rs"]
pub mod util;
// Wave 2 — mid files.
#[path = "secur32/lsalpc.rs"]
pub mod lsalpc;
#[path = "wine/kerberos.rs"]
pub mod kerberos;
#[path = "wine/dispatcher.rs"]
pub mod dispatcher;
#[path = "wine/negotiate.rs"]
pub mod negotiate;
// Wave 3 — heavy files.
#[path = "secur32/thunks.rs"]
pub mod thunks;
#[path = "secur32/wrapper.rs"]
pub mod wrapper;
#[path = "secur32/sspi.rs"]
pub mod sspi;
// Wave 4 — wine/ntlm.rs (largest, the NTLM SSP).
#[path = "wine/ntlm.rs"]
pub mod ntlm;
