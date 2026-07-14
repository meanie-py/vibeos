
add_definitions(-D__WINESRC__)
spec2def(version.dll version.spec ADD_IMPORTLIB)

list(APPEND SOURCE
    # version.c — replaced by Rust (sdk/lib/rust/version-rust); cohab-linked
    #             via libversion_rust.a below. Kept on disk (never-delete rule).
    ${CMAKE_CURRENT_BINARY_DIR}/version.def)

#
# version-rust cohab: the entire version.c body (GetFileVersionInfo*/VerQueryValue*/
# VerFindFile*/VerInstallFile*) → Rust. Builds libversion_rust.a from the workspace
# crate (STANDARD user-mode per-arch targets; i386 gets -sse,-sse2 via
# sdk/lib/rust/.cargo/config.toml — the user-mode no-SSE rule). The crate's
# #[no_mangle] symbols are the sole providers of version.c's functions now that the
# C TU is commented out (conutils-style cohab; no --allow-multiple-definition needed
# since every version.c symbol was removed).
#
set(VERSION_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
set(VERSION_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-version-target")
set(VERSION_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libversion_rust.a")
if(ARCH STREQUAL "i386")
    set(VERSION_RUST_TARGET "i686-pc-windows-gnu")
elseif(ARCH STREQUAL "amd64")
    set(VERSION_RUST_TARGET "x86_64-pc-windows-gnu")
elseif(ARCH STREQUAL "arm64")
    set(VERSION_RUST_TARGET "aarch64-pc-windows-gnullvm")
endif()
file(GLOB_RECURSE VERSION_RUST_SRCS "${VERSION_RUST_WORKSPACE_DIR}/version-rust/src/*.rs")
add_custom_command(
    OUTPUT ${VERSION_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${VERSION_RUST_TARGET_DIR}
       
        cargo build --release --target ${VERSION_RUST_TARGET}
            -p version-rust
            --manifest-path ${VERSION_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${VERSION_RUST_TARGET_DIR}/${VERSION_RUST_TARGET}/release/libversion_rust.a
        ${VERSION_RUST_LIB}
    DEPENDS ${VERSION_RUST_SRCS}
            ${VERSION_RUST_WORKSPACE_DIR}/version-rust/Cargo.toml
    WORKING_DIRECTORY ${VERSION_RUST_WORKSPACE_DIR}
    COMMENT "Building version-rust (${VERSION_RUST_TARGET})"
    VERBATIM)
add_custom_target(rust_version DEPENDS ${VERSION_RUST_LIB})

add_library(version MODULE ${SOURCE} version.rc)
set_module_type(version win32dll)
target_link_libraries(version wine oldnames ${VERSION_RUST_LIB})
add_importlibs(version msvcrt kernel32 ntdll)
add_dependencies(version rust_version)
add_cd_file(TARGET version DESTINATION reactos/system32 FOR all)
