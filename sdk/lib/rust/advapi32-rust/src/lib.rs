//! advapi32-rust — the advapi32.dll C-to-Rust body translation.
//!
//! Cohab with the still-C advapi32 .c files (commented out of
//! dll/win32/advapi32/CMakeLists.txt as each flips). The `#[no_mangle]`
//! WINAPI/cdecl functions here are the sole providers of those exported symbols
//! once their C TU is commented out; the not-yet-flipped C bodies resolve at link
//! the other way (--allow-multiple-definition, lld first-match). RPC *client*
//! stubs (svcctl/winreg/lsa/seclogon/eventlogrpc) stay generated-C and are called
//! by decorated name. FFI substrate = ntdll-sys (NT base) + the crate's own
//! security / RPC-client / Win32 declaration modules.
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

// Compile-time UTF-16 NUL-terminated literal: `wide!("Software")` → &'static [u16; N]
// (no alloc; the #![no_std] crate has no Vec). Replaces the c2rust
// `"X\0".encode_utf16().collect::<Vec<u16>>().as_ptr()` idiom (which both needed
// alloc AND was a use-after-free of the dropped Vec). ASCII literals only.
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
// FIELD_OFFSET(type, field) → byte offset of `field` within `type`. The C macro
// yields LONG; every advapi32 use site feeds it into a memcpy length, so return
// `usize` to match those call sites directly.
#[macro_export]
macro_rules! FIELD_OFFSET {
    ($t:ty, $f:ident) => {{
        let u = core::mem::MaybeUninit::<$t>::uninit();
        let p = u.as_ptr();
        unsafe { (core::ptr::addr_of!((*p).$f) as usize) - (p as usize) }
    }};
}

#[macro_export]
macro_rules! wstr { ($s:literal) => { $crate::wide!($s) }; }
#[macro_export]
macro_rules! w { ($s:literal) => { $crate::wide!($s) }; }
#[macro_export]
macro_rules! L { ($s:literal) => { $crate::wide!($s) }; }
#[macro_export]
macro_rules! widestring { ($s:literal) => { $crate::wide!($s) }; }

// Foundation: the cohab import surface (NT base re-exports, security/RPC-client
// types + consts, Win32 helpers). Body modules add `use crate::prelude::*;`.
// Wine's ERR/FIXME/WARN/TRACE, shared by every Wine-derived crate.
#[macro_use]
extern crate winedebug_rust;

pub mod externs;
pub mod prelude;

// USER-MODE no_std staticlib → own panic handler + eh_personality (deduped at
// link by --allow-multiple-definition, same as lsasrv-rust/samsrv-rust).
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[no_mangle]
extern "C" fn rust_eh_personality() {}

// ---- translated body modules (one per .c; declared as each file flips) ----
// Wave 1 — leaf/small files.
#[path = "token/token.rs"]
pub mod token;
#[path = "misc/unicode.rs"]
pub mod misc_unicode;
#[path = "misc/msi.rs"]
pub mod misc_msi;
#[path = "misc/dllmain.rs"]
pub mod misc_dllmain;
#[path = "misc/evntrace.rs"]
pub mod misc_evntrace;
#[path = "misc/hwprofiles.rs"]
pub mod misc_hwprofiles;
#[path = "misc/efs.rs"]
pub mod misc_efs;
#[path = "misc/shutdown.rs"]
pub mod misc_shutdown;
#[path = "service/winreg.rs"]
pub mod service_winreg;
#[path = "sec/trustee.rs"]
pub mod sec_trustee;
#[path = "sec/safer.rs"]
pub mod sec_safer;
#[path = "wine/crypt_des.rs"]
pub mod wine_crypt_des;
#[path = "wine/crypt_lmhash.rs"]
pub mod wine_crypt_lmhash;
// Wave 2 — mid-size files.
#[path = "sec/sec.rs"]
pub mod sec_sec;
#[path = "sec/audit.rs"]
pub mod sec_audit;
// sec/ac.rs — Acc* access-control calls route through sec_misc's NtMartaStatic
// dynamic-ntmarta function-pointer table + CheckNtMartaPresent (now flipped).
#[path = "sec/ac.rs"]
pub mod sec_ac;
#[path = "misc/sysfunc.rs"]
pub mod misc_sysfunc;
#[path = "reg/hkcr.rs"]
pub mod reg_hkcr;
#[path = "service/sctrl.rs"]
pub mod service_sctrl;
// Wave 3 — large files.
#[path = "misc/logon.rs"]
pub mod misc_logon;
#[path = "service/eventlog.rs"]
pub mod service_eventlog;
#[path = "sec/misc.rs"]
pub mod sec_misc;
#[path = "wine/cred.rs"]
pub mod wine_cred;
#[path = "sec/lsa.rs"]
pub mod sec_lsa;
#[path = "wine/crypt.rs"]
pub mod wine_crypt;
// Wave 4 — largest files.
#[path = "service/scm.rs"]
pub mod service_scm;
#[path = "wine/security.rs"]
pub mod wine_security;
#[path = "reg/reg.rs"]
pub mod reg_reg;
#[path = "service/rpc.rs"]
pub mod service_rpc;
