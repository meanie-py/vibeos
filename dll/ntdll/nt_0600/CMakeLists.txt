
spec2def(ntdll_vista.dll ntdll_vista.spec ADD_IMPORTLIB)

add_definitions(
    -D__NTDLL__
    -D_NTOSKRNL_
    -DCRTDLL)

include_directories(
    BEFORE include
    ${REACTOS_SOURCE_DIR}/sdk/include/reactos/subsys)

list(APPEND SOURCE
    ldr/ldrinit.c
    ldr/ldrnotify.c)

add_library(ntdll_vista_static ${SOURCE})
target_link_libraries(ntdll_vista_static)
add_dependencies(ntdll_vista_static psdk)

# --- Rust cohabitation (ntdll-rust) ---------------------------------
# DllMain.c (the module ENTRYPOINT) is commented out of add_library()
# below; the #[no_mangle] DllMain in libntdll_rust.a
# (sdk/lib/rust/ntdll-rust/src/nt_0600/dllmain.rs) takes over. The .c
# stays on disk (never delete the C). ntdll_vista_rust_stub.c keeps the
# MODULE target valid. Mirrors the csrsrv / sdk/lib/rtl cargo→.a flip.
# First ntdll cohabitation proof — see
# docs/engineering/ntdll-translation-audit.md Phase 2 step 1.
set(RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
set(RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-target")
if(ARCH STREQUAL "i386")
    set(RUST_TARGET "i686-pc-windows-gnu")
    set(RUST_EXTRA_ENV "RUSTFLAGS=-C target-feature=-sse,-sse2 -C no-prepopulate-passes")
elseif(ARCH STREQUAL "amd64")
    set(RUST_TARGET "x86_64-pc-windows-gnu")
    set(RUST_EXTRA_ENV "")
elseif(ARCH STREQUAL "arm64")
    set(RUST_TARGET "aarch64-pc-windows-gnullvm")
    set(RUST_EXTRA_ENV "")
endif()
set(NTDLL_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libntdll_rust.a")
add_custom_command(
    OUTPUT ${NTDLL_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${RUST_TARGET_DIR} ${RUST_EXTRA_ENV}
        cargo build --release --target ${RUST_TARGET}
            -p ntdll-rust
            --manifest-path ${RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${RUST_TARGET_DIR}/${RUST_TARGET}/release/libntdll_rust.a
        ${NTDLL_RUST_LIB}
    DEPENDS
        ${RUST_WORKSPACE_DIR}/ntdll-rust/src/lib.rs
        ${RUST_WORKSPACE_DIR}/ntdll-rust/src/nt_0600/mod.rs
        ${RUST_WORKSPACE_DIR}/ntdll-rust/src/nt_0600/dllmain.rs
        ${RUST_WORKSPACE_DIR}/ntdll-rust/Cargo.toml
        ${RUST_WORKSPACE_DIR}/ntdll-sys/src/lib.rs
    COMMENT "Building ntdll-rust (${RUST_TARGET}) for ntdll_vista.dll"
    VERBATIM)
add_custom_target(ntdll_vista_rust_lib DEPENDS ${NTDLL_RUST_LIB})
# --------------------------------------------------------------------

add_library(ntdll_vista MODULE
    ntdll_vista_rust_stub.c
    # DllMain.c — replaced by Rust (ntdll-rust/src/nt_0600/dllmain.rs)
    ${CMAKE_CURRENT_BINARY_DIR}/ntdll_vista.def
    ntdll_vista.rc)
set_module_type(ntdll_vista win32dll ENTRYPOINT DllMain 12)
target_link_libraries(ntdll_vista ntdll_vista_static smlib rtl_vista ${PSEH_LIB} ${NTDLL_RUST_LIB})
add_dependencies(ntdll_vista ntdll_vista_rust_lib)
target_link_options(ntdll_vista PRIVATE "-Wl,--allow-multiple-definition")
if(FALSE)
    target_link_libraries(ntdll_vista chkstk)
endif()
add_importlibs(ntdll_vista ntdll)
add_dependencies(ntdll_vista psdk)
add_cd_file(TARGET ntdll_vista DESTINATION reactos/system32 FOR all)
