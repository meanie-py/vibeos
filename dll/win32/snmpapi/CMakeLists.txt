
remove_definitions(-D_WIN32_WINNT=0x502)
add_definitions(-D_WIN32_WINNT=0x600)

spec2def(snmpapi.dll snmpapi.spec ADD_IMPORTLIB)

# snmpapi.dll — Rust cohabitation flip (FULL: the single compiled src *.c → snmpapi-rust).
# The AsnAny/AsnObjectIdentifier/SnmpVarBind layouts are bindgen-generated per arch into the
# crate's `types` module by tools/rustconv/gen_driver_types.sh (profile: snmpapi), not
# hand-transcribed — which matters here because AsnAny is a tagged UNION and the ASN_* tags
# it dispatches on are macro EXPRESSIONS, so island completeness is a correctness property
# (field manual §9.112). main.c is commented out below and kept on disk per the
# never-delete rule.
set(SNMPAPI_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
if(ARCH STREQUAL "i386")
    set(SNMPAPI_RUST_TARGET "i686-pc-windows-gnu")
elseif(ARCH STREQUAL "amd64")
    set(SNMPAPI_RUST_TARGET "x86_64-pc-windows-gnu")
elseif(ARCH STREQUAL "arm64")
    set(SNMPAPI_RUST_TARGET "aarch64-pc-windows-gnullvm")
endif()
rust_storage_target(SNMPAPI_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-snmpapi-target" snmpapi-rust)
set(SNMPAPI_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libsnmpapi_rust.a")
rust_crate_sources(SNMPAPI_RUST_SRCS "${SNMPAPI_RUST_WORKSPACE_DIR}" snmpapi-rust)
add_custom_command(
    OUTPUT ${SNMPAPI_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${SNMPAPI_RUST_TARGET_DIR}
        cargo build --release --target ${SNMPAPI_RUST_TARGET}
            -p snmpapi-rust
            --manifest-path ${SNMPAPI_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy_if_different
        ${SNMPAPI_RUST_TARGET_DIR}/${SNMPAPI_RUST_TARGET}/release/libsnmpapi_rust.a
        ${SNMPAPI_RUST_LIB}
    DEPENDS ${SNMPAPI_RUST_SRCS}
            ${SNMPAPI_RUST_WORKSPACE_DIR}/snmpapi-rust/Cargo.toml
    WORKING_DIRECTORY ${SNMPAPI_RUST_WORKSPACE_DIR}
    COMMENT "Building snmpapi-rust (${SNMPAPI_RUST_TARGET})"
)
add_custom_target(rust_snmpapi DEPENDS ${SNMPAPI_RUST_LIB})

# The only compiled src *.c → snmpapi-rust (src/snmpapi/main.rs), kept on disk per the
# never-delete rule.
#   main.c  → snmpapi-rust
# list(APPEND SOURCE
#     main.c)
list(APPEND SOURCE
    ${CMAKE_CURRENT_BINARY_DIR}/snmpapi_stubs.c
    ${CMAKE_CURRENT_BINARY_DIR}/snmpapi.def)

add_library(snmpapi MODULE ${SOURCE} snmpapi.rc)
set_module_type(snmpapi win32dll)
# libsnmpapi_rust.a FIRST so its #[no_mangle] symbols win.
target_link_libraries(snmpapi ${SNMPAPI_RUST_LIB} wine)
add_dependencies(snmpapi rust_snmpapi)
add_importlibs(snmpapi msvcrt kernel32_vista kernel32 ntdll)
add_cd_file(TARGET snmpapi DESTINATION reactos/system32 FOR all)
set_wine_module(snmpapi)
