
# Console Configuration library
add_subdirectory(concfg)

include_directories(
    ${REACTOS_SOURCE_DIR}/sdk/include/reactos/subsys
    ${REACTOS_SOURCE_DIR}/win32ss/include)

spec2def(winsrv.dll winsrv.spec)

# The components
include(consrv.cmake)
include(usersrv.cmake)

# --- Rust cohabitation (winsrv-rust, Phase 5 SMSS+CSRSS spine) -------
# Builds libwinsrv_rust.a from the winsrv-rust crate and links it into
# winsrv.dll. The flip is file-by-file: a .c gets commented out of its
# consrv/usersrv/concfg SOURCE list, its symbol goes undefined, and lld
# pulls the Rust replacement from the archive. --allow-multiple-definition
# covers any side-by-side overlap (lld first-match wins). The .c stay on
# disk (never delete the C). Mirrors the csrsrv.dll / sdk/lib/rtl pattern.
set(WINSRV_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
set(WINSRV_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-target")
if(ARCH STREQUAL "i386")
    set(WINSRV_RUST_TARGET "i686-pc-windows-gnu")
    set(WINSRV_RUST_EXTRA_ENV "RUSTFLAGS=-C target-feature=-sse,-sse2 -C no-prepopulate-passes")
elseif(ARCH STREQUAL "amd64")
    set(WINSRV_RUST_TARGET "x86_64-pc-windows-gnu")
    set(WINSRV_RUST_EXTRA_ENV "")
elseif(ARCH STREQUAL "arm64")
    set(WINSRV_RUST_TARGET "aarch64-pc-windows-gnullvm")
    set(WINSRV_RUST_EXTRA_ENV "")
endif()
set(WINSRV_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libwinsrv_rust.a")
file(GLOB_RECURSE WINSRV_RUST_SOURCES CONFIGURE_DEPENDS
    "${WINSRV_RUST_WORKSPACE_DIR}/winsrv-rust/src/*.rs")
add_custom_command(
    OUTPUT ${WINSRV_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${WINSRV_RUST_TARGET_DIR} ${WINSRV_RUST_EXTRA_ENV}
        cargo build --release --target ${WINSRV_RUST_TARGET}
            -p winsrv-rust
            --no-default-features --features winsrv_dll_only,ntddi_ws03
            --manifest-path ${WINSRV_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${WINSRV_RUST_TARGET_DIR}/${WINSRV_RUST_TARGET}/release/libwinsrv_rust.a
        ${WINSRV_RUST_LIB}
    DEPENDS ${WINSRV_RUST_SOURCES}
        ${WINSRV_RUST_WORKSPACE_DIR}/winsrv-rust/Cargo.toml
    COMMENT "Building winsrv-rust (${WINSRV_RUST_TARGET}) for winsrv.dll"
    VERBATIM)
add_custom_target(winsrv_rust_lib DEPENDS ${WINSRV_RUST_LIB})
# --------------------------------------------------------------------

list(APPEND SOURCE
    init.c
    winsrv.rc
    ${CMAKE_CURRENT_BINARY_DIR}/winsrv.def)

add_library(winsrv MODULE ${SOURCE})
set_module_type(winsrv win32dll UNICODE ENTRYPOINT DllMain 12)
#############################################
## HACK FOR MSVC COMPILATION WITH win32dll ##
set_subsystem(winsrv console)
################# END  HACK #################

# Add the components
target_link_libraries(winsrv consrv usersrv)
# Add win32ksys because of NtUser...()
target_link_libraries(winsrv ${CONSRV_TARGET_LINK_LIBS} win32ksys libcntpr ${PSEH_LIB} ${WINSRV_RUST_LIB})
add_dependencies(winsrv winsrv_rust_lib)
target_link_options(winsrv PRIVATE "-Wl,--allow-multiple-definition")

add_delay_importlibs(winsrv ${CONSRV_DELAY_IMPORT_LIBS} ${USERSRV_DELAY_IMPORT_LIBS} advapi32)
add_importlibs(winsrv ${CONSRV_IMPORT_LIBS} ${USERSRV_IMPORT_LIBS} csrsrv gdi32 user32 kernel32 ntdll)
add_dependencies(winsrv bugcodes xdk)
add_cd_file(TARGET winsrv DESTINATION reactos/system32 FOR all)
