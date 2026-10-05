
spec2def(authz.dll authz.spec)

# authz.dll — Rust cohabitation flip (FULL: all 3 compiled src *.c → authz-rust).
#
# The authorization API: resource managers, client contexts, and the audit-event surface.
# Plain C, and — like ntmarta — no C shim: no inline asm, and the three C-VARIADIC exports
# (AuthzInitializeObjectAccessAuditEvent{,2}, AuthzReportSecurityEvent) are `UNIMPLEMENTED;
# return FALSE;` stubs that never read their varargs. authz.spec declares all three `cdecl`
# with only the FIXED parameters, and under cdecl the CALLER cleans the stack, so a fixed-arity
# callee is ABI-identical to the variadic one for every caller. Implementing any of them later
# means adding the varargs trampoline (sdk/lib/rust/varargs-rust) — see the note at each fn.
set(AUTHZ_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
if(ARCH STREQUAL "i386")
    set(AUTHZ_RUST_TARGET "i686-pc-windows-gnu")
elseif(ARCH STREQUAL "amd64")
    set(AUTHZ_RUST_TARGET "x86_64-pc-windows-gnu")
elseif(ARCH STREQUAL "arm64")
    set(AUTHZ_RUST_TARGET "aarch64-pc-windows-gnullvm")
endif()
rust_storage_target(AUTHZ_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-authz-target" authz-rust)
set(AUTHZ_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libauthz_rust.a")
rust_crate_sources(AUTHZ_RUST_SRCS "${AUTHZ_RUST_WORKSPACE_DIR}" authz-rust)
add_custom_command(
    OUTPUT ${AUTHZ_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${AUTHZ_RUST_TARGET_DIR}
        cargo build --release --target ${AUTHZ_RUST_TARGET}
            -p authz-rust
            --manifest-path ${AUTHZ_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy_if_different
        ${AUTHZ_RUST_TARGET_DIR}/${AUTHZ_RUST_TARGET}/release/libauthz_rust.a
        ${AUTHZ_RUST_LIB}
    DEPENDS ${AUTHZ_RUST_SRCS}
            ${AUTHZ_RUST_WORKSPACE_DIR}/authz-rust/Cargo.toml
    WORKING_DIRECTORY ${AUTHZ_RUST_WORKSPACE_DIR}
    COMMENT "Building authz-rust (${AUTHZ_RUST_TARGET})"
)
add_custom_target(rust_authz DEPENDS ${AUTHZ_RUST_LIB})

# All 3 compiled src *.c → authz-rust (src/authz/*.rs), kept on disk per the never-delete rule.
#   authz.c clictx.c resman.c → authz-rust
# list(APPEND SOURCE
#     authz.c
#     clictx.c
#     resman.c
#     precomp.h)
list(APPEND SOURCE
    ${CMAKE_CURRENT_BINARY_DIR}/authz_stubs.c)

add_library(authz MODULE
    ${SOURCE}
    authz.rc
    ${CMAKE_CURRENT_BINARY_DIR}/authz.def)

set_module_type(authz win32dll UNICODE)
# libauthz_rust.a FIRST so its #[no_mangle] symbols win.
target_link_libraries(authz ${AUTHZ_RUST_LIB})
add_dependencies(authz rust_authz)
add_importlibs(authz advapi32 msvcrt kernel32 ntdll)
# add_pch dropped with precomp.h: the only C left is the generated authz_stubs.c, which does
# not include it.
add_cd_file(TARGET authz DESTINATION reactos/system32 FOR all)
