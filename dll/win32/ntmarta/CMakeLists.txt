
spec2def(ntmarta.dll ntmarta.spec)

# ntmarta.dll — Rust cohabitation flip (FULL: the single compiled src *.c → ntmarta-rust).
#
# The MARTA provider: the ACL/SID/trustee back end advapi32's Get/SetNamedSecurityInfo calls
# through. Plain C, no COM, and — unusually — no C shim at all: the module has no variadic
# entry point and no inline asm, so the whole DLL is Rust plus the generated .def.
#
# Its island profile allowlists winsvc.h by hand. The derive does not pull it in, because
# SWITCH_CONST_HEADER_ADDED only adds a header that owns a `case` constant, and ntmarta reaches
# the service database through OpenSCManager/QueryServiceObjectSecurity — SC_HANDLE and
# SC_MANAGER_CONNECT are ordinary uses, not case labels.
set(NTMARTA_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
if(ARCH STREQUAL "i386")
    set(NTMARTA_RUST_TARGET "i686-pc-windows-gnu")
elseif(ARCH STREQUAL "amd64")
    set(NTMARTA_RUST_TARGET "x86_64-pc-windows-gnu")
elseif(ARCH STREQUAL "arm64")
    set(NTMARTA_RUST_TARGET "aarch64-pc-windows-gnullvm")
endif()
rust_storage_target(NTMARTA_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-ntmarta-target" ntmarta-rust)
set(NTMARTA_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libntmarta_rust.a")
rust_crate_sources(NTMARTA_RUST_SRCS "${NTMARTA_RUST_WORKSPACE_DIR}" ntmarta-rust)
add_custom_command(
    OUTPUT ${NTMARTA_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${NTMARTA_RUST_TARGET_DIR}
        cargo build --release --target ${NTMARTA_RUST_TARGET}
            -p ntmarta-rust
            --manifest-path ${NTMARTA_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy_if_different
        ${NTMARTA_RUST_TARGET_DIR}/${NTMARTA_RUST_TARGET}/release/libntmarta_rust.a
        ${NTMARTA_RUST_LIB}
    DEPENDS ${NTMARTA_RUST_SRCS}
            ${NTMARTA_RUST_WORKSPACE_DIR}/ntmarta-rust/Cargo.toml
    WORKING_DIRECTORY ${NTMARTA_RUST_WORKSPACE_DIR}
    COMMENT "Building ntmarta-rust (${NTMARTA_RUST_TARGET})"
)
add_custom_target(rust_ntmarta DEPENDS ${NTMARTA_RUST_LIB})

# The only compiled src *.c → ntmarta-rust (src/ntmarta/ntmarta.rs), kept on disk per the
# never-delete rule.
#   ntmarta.c → ntmarta-rust
# list(APPEND SOURCE
#     ntmarta.c)
list(APPEND SOURCE
    ntmarta.rc
    ${CMAKE_CURRENT_BINARY_DIR}/ntmarta.def)

add_library(ntmarta MODULE ${SOURCE})
set_module_type(ntmarta win32dll UNICODE)
# libntmarta_rust.a FIRST so its #[no_mangle] symbols win.
target_link_libraries(ntmarta ${NTMARTA_RUST_LIB})
add_dependencies(ntmarta rust_ntmarta)
add_importlibs(ntmarta user32 advapi32 msvcrt kernel32 ntdll)
add_cd_file(TARGET ntmarta DESTINATION reactos/system32 FOR all)
