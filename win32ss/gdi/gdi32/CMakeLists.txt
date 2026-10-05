
add_subdirectory(wine)

add_definitions(-DLANGPACK)

include_directories(
    include
    ${REACTOS_SOURCE_DIR}/win32ss/include)

add_definitions(-DLANGPACK)
spec2def(gdi32.dll gdi32.spec ADD_IMPORTLIB)

if(ARCH STREQUAL "i386")
    # objects/efloat.c — replaced by Rust (gdi32-rust/src/objects/efloat.rs, i386-only cfg); cohab via libgdi32_rust.a
endif()

list(APPEND SOURCE
    # main/dllmain.c — replaced by Rust (gdi32-rust/src/main/dllmain.rs); cohab via libgdi32_rust.a
    # misc/heap.c — replaced by Rust (gdi32-rust/src/misc/heap.rs); cohab via libgdi32_rust.a
    # misc/gdientry.c — replaced by Rust (gdi32-rust/src/misc/gdientry.rs); cohab via libgdi32_rust.a
    # misc/historic.c — replaced by Rust (gdi32-rust/src/misc/historic.rs); cohab via libgdi32_rust.a
    # misc/misc.c — replaced by Rust (gdi32-rust/src/misc/misc.rs); cohab via libgdi32_rust.a
    # misc/wingl.c — replaced by Rust (gdi32-rust/src/misc/wingl.rs); cohab via libgdi32_rust.a
    # objects/arc.c — replaced by Rust (gdi32-rust/src/objects/arc.rs); cohab via libgdi32_rust.a
    # objects/bitmap.c — replaced by Rust (gdi32-rust/src/objects/bitmap.rs); cohab via libgdi32_rust.a
    # objects/brush.c — replaced by Rust (gdi32-rust/src/objects/brush.rs); cohab via libgdi32_rust.a
    # objects/clientobj.c — replaced by Rust (gdi32-rust/src/objects/clientobj.rs); cohab via libgdi32_rust.a
    # objects/colorspace.c — replaced by Rust (gdi32-rust/src/objects/colorspace.rs); cohab via libgdi32_rust.a
    # objects/coord.c — replaced by Rust (gdi32-rust/src/objects/coord.rs); cohab via libgdi32_rust.a
    # objects/dc.c — replaced by Rust (gdi32-rust/src/objects/dc.rs); cohab via libgdi32_rust.a
    # objects/eng.c — replaced by Rust (gdi32-rust/src/objects/eng.rs); cohab via libgdi32_rust.a
    # objects/enhmfile.c — replaced by Rust (gdi32-rust/src/objects/enhmfile.rs); cohab via libgdi32_rust.a
    # objects/font.c — replaced by Rust (gdi32-rust/src/objects/font.rs); cohab via libgdi32_rust.a
    # objects/gdiobj.c — replaced by Rust (gdi32-rust/src/objects/gdiobj.rs); cohab via libgdi32_rust.a
    # objects/icm.c — replaced by Rust (gdi32-rust/src/objects/icm.rs); cohab via libgdi32_rust.a
    # objects/linedda.c — replaced by Rust (gdi32-rust/src/objects/linedda.rs); cohab via libgdi32_rust.a
    # objects/metafile.c — replaced by Rust (gdi32-rust/src/objects/metafile.rs); cohab via libgdi32_rust.a
    # objects/painting.c — replaced by Rust (gdi32-rust/src/objects/painting.rs); cohab via libgdi32_rust.a
    # objects/printdrv.c — replaced by Rust (gdi32-rust/src/objects/printdrv.rs); cohab via libgdi32_rust.a
    # objects/palette.c — replaced by Rust (gdi32-rust/src/objects/palette.rs); cohab via libgdi32_rust.a
    # objects/pen.c — replaced by Rust (gdi32-rust/src/objects/pen.rs); cohab via libgdi32_rust.a
    # objects/region.c — replaced by Rust (gdi32-rust/src/objects/region.rs); cohab via libgdi32_rust.a
    # objects/text.c — replaced by Rust (gdi32-rust/src/objects/text.rs); cohab via libgdi32_rust.a
    # objects/utils.c — replaced by Rust (gdi32-rust/src/objects/utils.rs); cohab via libgdi32_rust.a
    # objects/path.c — replaced by Rust (gdi32-rust/src/objects/path.rs); cohab via libgdi32_rust.a
    include/precomp.h)

# --- gdi32-rust cohab staticlib (libgdi32_rust.a) --------------------------------------
# Rust translation of the gdi32 sources, built as a staticlib and cohab-linked into
# gdi32.dll with the translated .c commented out of SOURCE above (per-file flip, mirrors
# dll/win32/kernel32 / win32ss/user/user32). User-mode STANDARD per-arch targets; i386
# gets -sse,-sse2 via sdk/lib/rust/.cargo/config.toml (user-mode no-SSE rule). The crate's
# #[no_mangle] symbols are the sole providers of the commented-out .c functions, so no
# --allow-multiple-definition is needed. NtGdi* imports resolve via the win32ksys importlib.
set(GDI32_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
rust_storage_target(GDI32_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-gdi32-target" gdi32-rust)
set(GDI32_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libgdi32_rust.a")
if(ARCH STREQUAL "i386")
    set(GDI32_RUST_TARGET "i686-pc-windows-gnu")
elseif(ARCH STREQUAL "amd64")
    set(GDI32_RUST_TARGET "x86_64-pc-windows-gnu")
elseif(ARCH STREQUAL "arm64")
    set(GDI32_RUST_TARGET "aarch64-pc-windows-gnullvm")
endif()
# GLOB_RECURSE so .rs under src/<subdir>/ also retrigger the libgdi32_rust.a rebuild.
rust_crate_sources(GDI32_RUST_SRCS "${GDI32_RUST_WORKSPACE_DIR}" gdi32-rust)
add_custom_command(
    OUTPUT ${GDI32_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${GDI32_RUST_TARGET_DIR}
        cargo build --release --target ${GDI32_RUST_TARGET}
            -p gdi32-rust
            --manifest-path ${GDI32_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${GDI32_RUST_TARGET_DIR}/${GDI32_RUST_TARGET}/release/libgdi32_rust.a
        ${GDI32_RUST_LIB}
    DEPENDS ${GDI32_RUST_SRCS}
            ${GDI32_RUST_WORKSPACE_DIR}/gdi32-rust/Cargo.toml
    WORKING_DIRECTORY ${GDI32_RUST_WORKSPACE_DIR}
    COMMENT "Building gdi32-rust (${GDI32_RUST_TARGET})"
    VERBATIM)
add_custom_target(rust_gdi32 DEPENDS ${GDI32_RUST_LIB})

add_library(gdi32 MODULE
    ${SOURCE}
    gdi32.rc
    ${CMAKE_CURRENT_BINARY_DIR}/gdi32.def)

set_module_type(gdi32 win32dll UNICODE ENTRYPOINT DllMain 12)

target_link_libraries(gdi32
    gdi32_vista_static
    winegdi
    wine
    win32ksys
    dxguid
    ${ATAN2_LIB}
    ${PSEH_LIB}
    ${GDI32_RUST_LIB})

# atan2-rust, by path ahead of the import libraries (gcc.cmake ATAN2_LIB).
add_dependencies(gdi32 rust_atan2_rust)
add_importlibs(gdi32 user32 advapi32 kernel32 ntdll)
add_pch(gdi32 include/precomp.h SOURCE)
add_dependencies(gdi32 psdk rust_gdi32)
add_cd_file(TARGET gdi32 DESTINATION reactos/system32 FOR all)
