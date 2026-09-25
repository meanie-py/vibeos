
spec2def(userenv.dll userenv.spec ADD_IMPORTLIB)

# userenv.dll — Rust cohabitation flip (all 10 compiled src *.c → userenv-rust). The translated
# bodies come from libuserenv_rust.a (sdk/lib/rust/userenv-rust, kept on disk per the
# never-delete rule); the C is commented out below. userenv_stubs.c (spec2def @stub stubs) and
# userenv.rc (the logon-timeout dialog resources) stay C.
#
# userenv implements user profiles (Load/UnloadUserProfile, Create/DeleteProfile), environment
# blocks, group-policy path resolution, and per-user shell-folder/registry setup — LOADED ON THE
# BOOT PATH by winlogon→userinit. The private structs (DYN_FUNCS/POLICY_VALUES, PROFILEINFO) are
# bindgen-GENERATED per arch into the crate's `types` module (profile: userenv), not hand-transcribed.
set(USERENV_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
if(ARCH STREQUAL "i386")
    set(USERENV_RUST_TARGET "i686-pc-windows-gnu")
elseif(ARCH STREQUAL "amd64")
    set(USERENV_RUST_TARGET "x86_64-pc-windows-gnu")
elseif(ARCH STREQUAL "arm64")
    set(USERENV_RUST_TARGET "aarch64-pc-windows-gnullvm")
endif()
rust_storage_target(USERENV_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-userenv-target" userenv-rust)
set(USERENV_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libuserenv_rust.a")
rust_crate_sources(USERENV_RUST_SRCS "${USERENV_RUST_WORKSPACE_DIR}" userenv-rust)
add_custom_command(
    OUTPUT ${USERENV_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${USERENV_RUST_TARGET_DIR}
        cargo build --release --target ${USERENV_RUST_TARGET}
            -p userenv-rust
            --manifest-path ${USERENV_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy_if_different
        ${USERENV_RUST_TARGET_DIR}/${USERENV_RUST_TARGET}/release/libuserenv_rust.a
        ${USERENV_RUST_LIB}
    DEPENDS ${USERENV_RUST_SRCS}
            ${USERENV_RUST_WORKSPACE_DIR}/userenv-rust/Cargo.toml
    WORKING_DIRECTORY ${USERENV_RUST_WORKSPACE_DIR}
    COMMENT "Building userenv-rust (${USERENV_RUST_TARGET})"
)
add_custom_target(rust_userenv DEPENDS ${USERENV_RUST_LIB})

# 10 src *.c bodies replaced by Rust (userenv-rust/src/userenv/*.rs), kept on disk per the
# never-delete rule. (DllMain lives in userenv.rs; win32dll's default entry calls it.)
#   desktop.c     — replaced by Rust (userenv-rust src/userenv/desktop.rs — GetAllUsers/DefaultUserProfileDirectory)
#   directory.c   — replaced by Rust (userenv-rust src/userenv/directory.rs — CreateDirectoryPath + recursive dir helpers)
#   environment.c — replaced by Rust (userenv-rust src/userenv/environment.rs — Create/DestroyEnvironmentBlock + expansion)
#   gpolicy.c     — replaced by Rust (userenv-rust src/userenv/gpolicy.rs — group-policy path resolution)
#   misc.c        — replaced by Rust (userenv-rust src/userenv/misc.rs — AppendBackslash/SD/refcount + ReportError)
#   profile.c     — replaced by Rust (userenv-rust src/userenv/profile.rs — Load/Unload/Create/DeleteUserProfile)
#   registry.c    — replaced by Rust (userenv-rust src/userenv/registry.rs — per-user hive load/unload + copy)
#   setup.c       — replaced by Rust (userenv-rust src/userenv/setup.rs — the shell-folder tables setup)
#   sid.c         — replaced by Rust (userenv-rust src/userenv/sid.rs — SID → profile-key string helpers)
#   userenv.c     — replaced by Rust (userenv-rust src/userenv/userenv.rs — DllMain + module glue)
# list(APPEND SOURCE
#     desktop.c
#     directory.c
#     environment.c
#     gpolicy.c
#     misc.c
#     profile.c
#     registry.c
#     setup.c
#     sid.c
#     userenv.c)

list(APPEND PCH_SKIP_SOURCE
    ${CMAKE_CURRENT_BINARY_DIR}/userenv_stubs.c)
    # userenv_rust_shims.c is no longer built (retired 2026-09-25; the file stays on disk,
    # inert): hInstance and LocalSystemAuthority are #[no_mangle] Rust statics, and
    # StringCchPrintfW is a strsafe-rust macro that CALLS msvcrt's variadic _snwprintf.

add_library(userenv MODULE
    ${PCH_SKIP_SOURCE}
    userenv.rc
    ${CMAKE_CURRENT_BINARY_DIR}/userenv.def)

set_module_type(userenv win32dll)
# libuserenv_rust.a provides the #[no_mangle] export + DllMain symbols; force the cargo build.
# uuid stays: the Rust references CLSID_ShellLink/IID_IPersistFile/IID_IShellLinkW (uuid GUIDs).
target_link_libraries(userenv ${USERENV_RUST_LIB} uuid)
add_dependencies(userenv rust_userenv)
add_importlibs(userenv advapi32 advapi32_vista user32 msvcrt kernel32 ntdll)
# add_pch removed: no C translation units use precomp.h anymore (all 10 src *.c → Rust;
# userenv_stubs.c is PCH_SKIP_SOURCE and never used it).
add_cd_file(TARGET userenv DESTINATION reactos/system32 FOR all)
