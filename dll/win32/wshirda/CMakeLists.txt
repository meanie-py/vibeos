
spec2def(wshirda.dll wshirda.spec)

# wshirda.dll — Rust cohabitation flip (wshirda.c → wshirda-rust). The translated bodies come
# from libwshirda_rust.a (sdk/lib/rust/wshirda-rust, kept on disk per the never-delete rule);
# wshirda.c is commented out below. wshirda is a placeholder IrDA WSH provider — every export
# is an UNIMPLEMENTED stub (returns 0), so the Rust needs no bindgen struct layouts, only the
# stdcall #[no_mangle] exports that the .spec (ABI ground truth) decorates.
set(WSHIRDA_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
if(ARCH STREQUAL "i386")
    set(WSHIRDA_RUST_TARGET "i686-pc-windows-gnu")
elseif(ARCH STREQUAL "amd64")
    set(WSHIRDA_RUST_TARGET "x86_64-pc-windows-gnu")
elseif(ARCH STREQUAL "arm64")
    set(WSHIRDA_RUST_TARGET "aarch64-pc-windows-gnullvm")
endif()
set(WSHIRDA_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-wshirda-target")
set(WSHIRDA_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libwshirda_rust.a")
rust_crate_sources(WSHIRDA_RUST_SRCS "${WSHIRDA_RUST_WORKSPACE_DIR}" wshirda-rust)
add_custom_command(
    OUTPUT ${WSHIRDA_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${WSHIRDA_RUST_TARGET_DIR}
        cargo build --release --target ${WSHIRDA_RUST_TARGET}
            -p wshirda-rust
            --manifest-path ${WSHIRDA_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy_if_different
        ${WSHIRDA_RUST_TARGET_DIR}/${WSHIRDA_RUST_TARGET}/release/libwshirda_rust.a
        ${WSHIRDA_RUST_LIB}
    DEPENDS ${WSHIRDA_RUST_SRCS}
            ${WSHIRDA_RUST_WORKSPACE_DIR}/wshirda-rust/Cargo.toml
    WORKING_DIRECTORY ${WSHIRDA_RUST_WORKSPACE_DIR}
    COMMENT "Building wshirda-rust (${WSHIRDA_RUST_TARGET})"
)
add_custom_target(rust_wshirda DEPENDS ${WSHIRDA_RUST_LIB})

# wshirda.c → wshirda-rust (src/wshirda/wshirda.rs — 11 WSH* stub exports + DllMain), kept on
# disk per the never-delete rule.
# list(APPEND SOURCE
#     wshirda.c)
list(APPEND SOURCE
    wshirda.rc
    ${CMAKE_CURRENT_BINARY_DIR}/wshirda.def)

add_library(wshirda MODULE ${SOURCE})
set_module_type(wshirda win32dll UNICODE)
# libwshirda_rust.a first so its #[no_mangle] symbols win.
target_link_libraries(wshirda ${WSHIRDA_RUST_LIB})
add_dependencies(wshirda rust_wshirda)
add_importlibs(wshirda ws2_32 msvcrt kernel32 ntdll)
add_cd_file(TARGET wshirda DESTINATION reactos/system32 FOR all)
