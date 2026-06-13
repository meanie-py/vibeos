
# ============================================================================
# mup.sys — Rust cohabitation flip (mup.c + dfs.c -> mup-rust)
# ============================================================================
# The Multiple UNC Provider (mup.c DriverEntry + IRP_MJ_{CREATE,WRITE,FILE_SYSTEM_CONTROL,
# CLEANUP,CLOSE} dispatch + the prefix-table / provider-registration / query-path engine;
# dfs.c DFS stubs) is translated to Rust (sdk/lib/rust/mup-rust, kept on disk per the
# never-delete rule). DriverEntry + all routines come from libmup_rust.a. mup has no GUID
# data TU, so a tiny rust_cohab_stub.c is the only C source the MODULE link needs. mup is
# NOT boot-loaded (no txtsetup.sif entry → not on the LiveCD load path), so the bar is a
# clean cold link ×3 + no smoke regression. Driver-private node structs (MUP_VCB/FCB/CCB/
# MIC/UNC/PFX/MQC) are defined in-crate (mup-rust/src/types.rs); kernel/Io/FsRtl/Rtl types
# resolve to the single ntkrnl-sys ddk identity. Mirrors the fs_rec-rust / partmgr-rust cohab.
set(MUP_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
set(MUP_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-mup-target")
set(MUP_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libmup_rust.a")
if(ARCH STREQUAL "i386")
    set(MUP_RUST_TARGET "i686-pc-windows-gnu")
    set(MUP_RUST_RUSTFLAGS "-C target-feature=-sse,-sse2")
elseif(ARCH STREQUAL "amd64")
    set(MUP_RUST_TARGET "x86_64-pc-windows-gnu")
    set(MUP_RUST_RUSTFLAGS "")
elseif(ARCH STREQUAL "arm64")
    set(MUP_RUST_TARGET "aarch64-pc-windows-gnullvm")
    set(MUP_RUST_RUSTFLAGS "")
endif()
file(GLOB_RECURSE MUP_RUST_SRCS ${MUP_RUST_WORKSPACE_DIR}/mup-rust/src/*.rs)
add_custom_command(
    OUTPUT ${MUP_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${MUP_RUST_TARGET_DIR}
        RUSTFLAGS=${MUP_RUST_RUSTFLAGS}
        RUSTC_BOOTSTRAP=1
        cargo build --release --target ${MUP_RUST_TARGET}
            -p mup-rust
            --manifest-path ${MUP_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${MUP_RUST_TARGET_DIR}/${MUP_RUST_TARGET}/release/libmup_rust.a
        ${MUP_RUST_LIB}
    DEPENDS ${MUP_RUST_SRCS}
            ${MUP_RUST_WORKSPACE_DIR}/mup-rust/Cargo.toml
    WORKING_DIRECTORY ${MUP_RUST_WORKSPACE_DIR}
    COMMENT "Building mup-rust (${MUP_RUST_TARGET})"
    VERBATIM)
add_custom_target(rust_mup DEPENDS ${MUP_RUST_LIB})

# Both .c bodies replaced by Rust (mup-rust), kept on disk per the never-delete rule:
#   dfs.c mup.c
# list(APPEND SOURCE dfs.c mup.c dfs.h mup.h)

add_library(mup MODULE rust_cohab_stub.c mup.rc)
set_module_type(mup kernelmodedriver)
add_importlibs(mup ntoskrnl hal)
add_dependencies(mup rust_mup)
target_link_libraries(mup ${MUP_RUST_LIB})
add_cd_file(TARGET mup DESTINATION reactos/system32/drivers FOR all)
