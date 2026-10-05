
spec2def(msv1_0.dll msv1_0.spec)

# msv1_0.dll — Rust cohabitation flip (FULL: all 7 compiled src *.c → msv1_0-rust).
#
# The MSV1_0 authentication package: the LSA-side and user-side halves of NTLM/interactive
# logon that lsasrv loads and dispatches through SECPKG_FUNCTION_TABLE. Plain C, no COM, and
# no C shim — the module has no variadic entry point and no inline asm, so the whole DLL is
# Rust plus the generated .def and stubs.
#
# `ntlm/global.c` never went through `rustconv translate`: it defines only globals and no
# functions, so there was nothing to buy. Its six definitions live in src/ntlm/global.rs, with
# the two 27-/14-slot dispatch tables emitted mechanically from the C's designated
# initializers — see that file for why a hand-typed slot list was not acceptable here.
#
# The island uses BLOCKLIST_FUNCS_FROM_CRATE rather than BLOCKLIST_FUNCS: msv1_0 is a thin
# module over other people's APIs (the samsrv/lsasrv delay imports, advapi32's
# SystemFunction006/007/012), so blocklisting all functions would have meant hand-writing ~60
# RPC-shaped prototypes whose pointer levels nothing would ever check.
set(MSV1_0_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
if(ARCH STREQUAL "i386")
    set(MSV1_0_RUST_TARGET "i686-pc-windows-gnu")
elseif(ARCH STREQUAL "amd64")
    set(MSV1_0_RUST_TARGET "x86_64-pc-windows-gnu")
elseif(ARCH STREQUAL "arm64")
    set(MSV1_0_RUST_TARGET "aarch64-pc-windows-gnullvm")
endif()
rust_storage_target(MSV1_0_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-msv1_0-target" msv1_0-rust)
set(MSV1_0_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libmsv1_0_rust.a")
rust_crate_sources(MSV1_0_RUST_SRCS "${MSV1_0_RUST_WORKSPACE_DIR}" msv1_0-rust)
add_custom_command(
    OUTPUT ${MSV1_0_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${MSV1_0_RUST_TARGET_DIR}
        cargo build --release --target ${MSV1_0_RUST_TARGET}
            -p msv1_0-rust
            --manifest-path ${MSV1_0_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy_if_different
        ${MSV1_0_RUST_TARGET_DIR}/${MSV1_0_RUST_TARGET}/release/libmsv1_0_rust.a
        ${MSV1_0_RUST_LIB}
    DEPENDS ${MSV1_0_RUST_SRCS}
            ${MSV1_0_RUST_WORKSPACE_DIR}/msv1_0-rust/Cargo.toml
    WORKING_DIRECTORY ${MSV1_0_RUST_WORKSPACE_DIR}
    COMMENT "Building msv1_0-rust (${MSV1_0_RUST_TARGET})"
)
add_custom_target(rust_msv1_0 DEPENDS ${MSV1_0_RUST_LIB})

# All 7 compiled src *.c → msv1_0-rust, kept on disk per the never-delete rule.
#   lsa.c         → src/msv1_0/lsa.rs
#   msv1_0.c      → src/msv1_0/msv1_0.rs
#   sam.c         → src/msv1_0/sam.rs
#   user.c        → src/msv1_0/user.rs
#   usercontext.c → src/msv1_0/usercontext.rs
#   ntlm/util.c   → src/ntlm/util.rs
#   ntlm/global.c → src/ntlm/global.rs  (hand-carried; globals only, never translated)
# list(APPEND SOURCE
#     lsa.c
#     msv1_0.c
#     ntlm/global.c
#     ntlm/util.c
#     sam.c
#     user.c
#     usercontext.c)
list(APPEND SOURCE
    ${CMAKE_CURRENT_BINARY_DIR}/msv1_0_stubs.c
    ${CMAKE_CURRENT_BINARY_DIR}/msv1_0.def)

add_library(msv1_0 MODULE ${SOURCE} msv1_0.rc)
set_module_type(msv1_0 win32dll UNICODE ENTRYPOINT 0)
# libmsv1_0_rust.a FIRST so its #[no_mangle] symbols win.
target_link_libraries(msv1_0 ${MSV1_0_RUST_LIB} ${PSEH_LIB})
add_dependencies(msv1_0 rust_msv1_0)
add_delay_importlibs(msv1_0 samsrv lsasrv)
add_importlibs(msv1_0 advapi32 kernel32 ntdll)
add_dependencies(msv1_0 psdk)
add_cd_file(TARGET msv1_0 DESTINATION reactos/system32 FOR all)
