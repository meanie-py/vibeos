//! userenv.dll — Rust translation of dll/win32/userenv (the user-environment DLL).
//! staticlib → libuserenv_rust.a, cohab-linked into the cmake-built userenv.dll with the
//! translated .c commented out (per-file flip, like the winsock-family crates).
//!
//! userenv implements user profiles (Load/UnloadUserProfile, Create/DeleteProfile), environment
//! blocks (CreateEnvironmentBlock), group-policy path resolution, and per-user shell-folder /
//! registry-hive setup — loaded on the boot path by winlogon→userinit. Private structs
//! (DYN_FUNCS/DYN_MODULE/POLICY_VALUES, PROFILEINFO) are bindgen-generated per arch into `types`.
//!
//! Per-file cohab flip: each translated .c becomes a module here and its #[no_mangle]
//! symbols take over from the commented-out C.
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(unused_assignments, static_mut_refs)]
#![allow(clippy::all)]

// ntdll, cdecl. The C's DPRINT1s on the first-boot profile path (InitializeProfiles,
// CreateStandardProfile, CopySystemProfile), restored: GUI setup reports only "X() failed /
// Rebooting now!" in a message box, which serial never sees.
extern "C" {
    pub fn DbgPrint(Format: *const core::ffi::c_char, ...) -> u32;
}

/// The C's DPRINT1 on the profile path: `[USERENV] profiles: <step> failed (error N)`.
macro_rules! DPRINT1 {
    ($step:expr, $err:expr) => {
        $crate::DbgPrint(
            concat!("[USERENV] profiles: ", $step, " failed (error %lu)\n\0").as_ptr()
                as *const core::ffi::c_char,
            ($err) as u32,
        )
    };
}

pub mod types; // DYN_FUNCS/POLICY_VALUES/PROFILEINFO layouts (bindgen, per arch) + re-exported nt
pub mod prelude; // the ONE import surface: types glob + enum aliases + skipped consts + bridges

// The 10 translated translation units. `userenv` mirrors the C layout (dll/win32/userenv/).
#[allow(clippy::module_inception)]
pub mod userenv {
    pub mod desktop; // GetAllUsers/DefaultUserProfileDirectory + the desktop/common paths
    pub mod directory; // CreateDirectoryPath / recursive dir helpers
    pub mod environment; // Create/DestroyEnvironmentBlock + env-var expansion
    pub mod gpolicy; // group-policy path resolution (POLICY_VALUES)
    pub mod misc; // AppendBackslash / security-descriptor / refcount helpers
    pub mod profile; // Load/Unload/Create/DeleteUserProfile + the profile mutex
    pub mod registry; // per-user hive load/unload + registry copy helpers
    pub mod setup; // the shell-folder tables (CommonShellFolders/UserShellFolders) setup
    pub mod sid; // SID → profile-key string helpers
    pub mod sha256; // FIPS 180-4 SHA-256 (AppContainer SID derivation)
    pub mod userenv; // DllMain + the module entry glue
}

// staticlib (libuserenv_rust.a) needs a panic handler + eh personality (no_std cohab),
// gated off for `cargo test` where std provides them.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
