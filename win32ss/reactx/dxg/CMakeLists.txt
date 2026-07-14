
spec2def(dxg.sys dxg.spec)

# dxg.sys body translated to Rust (sdk/lib/rust/dxg-rust → libdxg_rust.a),
# cohab-linked (uniata pattern). The 9 .c are replaced by Rust and kept on disk:
#   main.c        → dxg-rust/src/driver_main.rs   (+ gaDxgFuncs pdrvfn wiring)
#   ddhmg.c       → dxg-rust/src/ddhmg.rs         (Dd handle manager)
#   d3d.c         → dxg-rust/src/d3d.rs
#   dd.c          → dxg-rust/src/dd.rs
#   ddraw.c       → dxg-rust/src/ddraw.rs
#   ddsurf.c      → dxg-rust/src/ddsurf.rs
#   dxgallocator.c→ dxg-rust/src/dxgallocator.rs
#   eng.c         → dxg-rust/src/eng.rs
#   historic.c    → dxg-rust/src/historic.rs
#   dxg_driver.h  → dxg-rust/src/dxg_driver.rs    (~75 stub thunks + gaDxgFuncs[92])
#   dxg_int.h     → dxg-rust/src/{types,tags}.rs  (DXENG_FUNCTIONS, DD_ENTRY, tags)
# list(APPEND SOURCE
#     main.c ddhmg.c d3d.c dd.c ddraw.c ddsurf.c dxgallocator.c eng.c historic.c
#     dxg_int.h)   # replaced by Rust (sdk/lib/rust/dxg-rust) — cohabitation

set(DXG_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
set(DXG_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-dxg-target")
set(DXG_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libdxg_rust.a")
if(ARCH STREQUAL "i386")
    set(DXG_RUST_TARGET "i686-pc-windows-gnu")
    set(DXG_RUST_RUSTFLAGS "-C target-feature=-sse,-sse2")
elseif(ARCH STREQUAL "amd64")
    set(DXG_RUST_TARGET "x86_64-pc-windows-gnu")
    set(DXG_RUST_RUSTFLAGS "")
elseif(ARCH STREQUAL "arm64")
    set(DXG_RUST_TARGET "aarch64-pc-windows-gnullvm")
    set(DXG_RUST_RUSTFLAGS "")
endif()
file(GLOB DXG_RUST_SRCS "${DXG_RUST_WORKSPACE_DIR}/dxg-rust/src/*.rs")
add_custom_command(
    OUTPUT ${DXG_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${DXG_RUST_TARGET_DIR}
        RUSTFLAGS=${DXG_RUST_RUSTFLAGS}
        cargo build --release --target ${DXG_RUST_TARGET}
            -p dxg-rust
            --manifest-path ${DXG_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${DXG_RUST_TARGET_DIR}/${DXG_RUST_TARGET}/release/libdxg_rust.a
        ${DXG_RUST_LIB}
    DEPENDS ${DXG_RUST_SRCS}
            ${DXG_RUST_WORKSPACE_DIR}/dxg-rust/Cargo.toml
    WORKING_DIRECTORY ${DXG_RUST_WORKSPACE_DIR}
    COMMENT "Building dxg-rust (${DXG_RUST_TARGET})"
    VERBATIM)
add_custom_target(rust_dxg DEPENDS ${DXG_RUST_LIB})

include_directories(
    ${REACTOS_SOURCE_DIR}/win32ss/include)

add_library(dxg MODULE
    rust_cohab_stub.c
    dxg.rc
    ${CMAKE_CURRENT_BINARY_DIR}/dxg.def)

set_module_type(dxg kernelmodedriver)
# libdxg_rust.a MUST precede the dxgthk importlib: the `-entry _DriverEntry@8`
# reference is satisfied by dxg-rust's strong DriverEntry first, so dxgthk's
# importlib copy of DriverEntry is never pulled (else lld "_DriverEntry@8 was
# replaced" — import-vs-local, not covered by --allow-multiple-definition).
# In the C build DriverEntry was an object (main.c.obj, linked before importlibs);
# the cohab moved it into a .a, so we restore the ordering explicitly.
add_dependencies(dxg rust_dxg)
target_link_libraries(dxg ${DXG_RUST_LIB})
add_importlibs(dxg dxgthk ntoskrnl)
add_cd_file(TARGET dxg DESTINATION reactos/system32/drivers FOR all)
