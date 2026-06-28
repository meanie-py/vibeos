#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(unused_assignments)]
#![allow(static_mut_refs)]
#![allow(clippy::all)]

//! shell32.dll — Rust translation of dll/win32/shell32 (the usermode Shell client DLL).
//! staticlib → libshell32_rust.a, cohab-linked into the cmake shell32.dll with the
//! translated .c commented out. The wine-derived pure-C half of the Shell API (registry
//! helpers, PIDL, shell paths, ordinal exports, policy, string utils, control-panel,
//! menus, OLE glue); the 81 .cpp COM-vtable classes (CDefView/CShellLink/...) stay C++.
//! Most bodies forward to other usermode DLLs (advapi32/shlwapi/ole32/kernel32) resolved
//! via shell32's existing import libs. See docs/engineering/shell32-translation-audit.md.

// Foundation: minimal Win32 type aliases (grows as waves land).
pub mod types;

// wine/ — wine-derived pure-C bodies. Wave 0: shellreg (SHReg* registry wrappers).
pub mod wine;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
