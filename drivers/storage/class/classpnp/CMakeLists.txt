
spec2def(classpnp.sys classpnp.spec ADD_IMPORTLIB)

remove_definitions(-D_WIN32_WINNT=0x502)

# ============================================================================
# classpnp.sys — Rust cohabitation flip (all 18 C sources -> sdk/lib/rust/classpnp-rust)
# ============================================================================
# Every classpnp C source (autorun/class/classwmi/clntirp/create/data/debug/dictlib/
# dispatch/guid/history/lock/obsolete/power/retry/srblib/utils/xferpkt) is translated to
# Rust (classpnp-rust, kept on disk per the never-delete rule). The 60 classpnp.spec
# exports (Class*) plus the PE DriverEntry come from libclasspnp_rust.a; cmake keeps the
# MODULE link (set_module_type kernelmodedriver + spec2def .def + add_importlibs).
# Mirrors wmilib-rust's static-lib cohab. classpnp is BOOT-CRITICAL (IDE/CD storage path).
set(CLASSPNP_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
set(CLASSPNP_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-classpnp-target")
set(CLASSPNP_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libclasspnp_rust.a")
if(ARCH STREQUAL "i386")
    set(CLASSPNP_RUST_TARGET "i686-pc-windows-gnu")
    set(CLASSPNP_RUST_RUSTFLAGS "-C target-feature=-sse,-sse2")
elseif(ARCH STREQUAL "amd64")
    set(CLASSPNP_RUST_TARGET "x86_64-pc-windows-gnu")
    set(CLASSPNP_RUST_RUSTFLAGS "")
elseif(ARCH STREQUAL "arm64")
    set(CLASSPNP_RUST_TARGET "aarch64-pc-windows-gnullvm")
    set(CLASSPNP_RUST_RUSTFLAGS "")
endif()
add_custom_command(
    OUTPUT ${CLASSPNP_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${CLASSPNP_RUST_TARGET_DIR}
        RUSTFLAGS=${CLASSPNP_RUST_RUSTFLAGS}
        cargo build --release --target ${CLASSPNP_RUST_TARGET}
            -p classpnp-rust
            --manifest-path ${CLASSPNP_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${CLASSPNP_RUST_TARGET_DIR}/${CLASSPNP_RUST_TARGET}/release/libclasspnp_rust.a
        ${CLASSPNP_RUST_LIB}
    DEPENDS ${CLASSPNP_RUST_WORKSPACE_DIR}/classpnp-rust/Cargo.toml
            ${CLASSPNP_RUST_WORKSPACE_DIR}/classpnp-rust/src/lib.rs
    WORKING_DIRECTORY ${CLASSPNP_RUST_WORKSPACE_DIR}
    COMMENT "Building classpnp-rust (${CLASSPNP_RUST_TARGET})"
    VERBATIM)
add_custom_target(rust_classpnp DEPENDS ${CLASSPNP_RUST_LIB})

list(APPEND SOURCE
    rust_cohab_stub.c
    class.rc
    ${CMAKE_CURRENT_BINARY_DIR}/classpnp.def)
#   autorun.c class.c classwmi.c clntirp.c create.c data.c debug.c dictlib.c
#   dispatch.c guid.c history.c lock.c obsolete.c power.c retry.c srblib.c
#   utils.c xferpkt.c  # all replaced by Rust (classpnp-rust); kept on disk

add_library(classpnp MODULE ${SOURCE})

target_compile_definitions(classpnp PUBLIC
    DEBUG_USE_KDPRINT
    _WIN32_WINNT=0x602
    NTDDI_VERSION=0x06020000) # NTDDI_WIN8

set_module_type(classpnp kernelmodedriver)
add_dependencies(classpnp rust_classpnp)
target_link_libraries(classpnp ${CLASSPNP_RUST_LIB} ntoskrnl_vista libcntpr ${PSEH_LIB})
add_importlibs(classpnp ntoskrnl hal)
add_cd_file(TARGET classpnp DESTINATION reactos/system32/drivers NO_CAB FOR all)
