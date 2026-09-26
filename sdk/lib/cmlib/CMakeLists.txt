
add_definitions(
    -D_NTSYSTEM_
    -DNASSERT)

list(APPEND SOURCE
    cmcheck.c
    cminit.c
    cmheal.c
    cmindex.c
    cmkeydel.c
    cmname.c
    cmse.c
    cmvalue.c
    hivebin.c
    hivecell.c
    hiveinit.c
    hivesum.c
    hivewrt.c
    cmlib.h)

# Rust cohabitation (KERNEL cmlib only) — sdk/lib/rust/cmlib-rust.
# Each cmlib .c translated to Rust is REMOVE_ITEM'd from the kernel cmlib's
# source list and provided instead by libcmlib_rust.a (the cmlib-rust staticlib
# linked below). The bootloader (blcmlib) and host (cmlibhost) variants keep the
# full C SOURCE for now. CMLIB_RUST_FLIPPED lists the flipped .c in ONE place.
# Mirrors the cnt-rust ↔ libcntpr hook in sdk/lib/crt/libcntpr.cmake.
set(CMLIB_RUST_FLIPPED
    hivesum.c
    cmse.c
    cminit.c
    hivebin.c
    cmkeydel.c
    cmname.c
    cmcheck.c
    cmheal.c
    cmindex.c
    cmvalue.c
    hivecell.c
    hiveinit.c
    hivewrt.c)

if(CMAKE_CROSSCOMPILING)
    # CMLIB for NT bootloader (still 100% C)
    add_library(blcmlib STATIC ${SOURCE})
    add_pch(blcmlib cmlib.h SOURCE)
    target_compile_definitions(blcmlib PRIVATE _BLDR_)
    target_include_directories(blcmlib INTERFACE ${CMAKE_CURRENT_SOURCE_DIR})
    add_dependencies(blcmlib bugcodes xdk)

    # CMLIB for NT kernel — drop the Rust-flipped .c, link libcmlib_rust.a.
    set(CMLIB_KERNEL_SOURCE ${SOURCE})
    list(REMOVE_ITEM CMLIB_KERNEL_SOURCE ${CMLIB_RUST_FLIPPED})
    # All 13 cmlib .c are now Rust (libcmlib_rust.a) → CMLIB_KERNEL_SOURCE has no
    # .c left, so cmake can't determine the cmlib target's linker language.
    # Add a placeholder TU (mirrors the HAL lib_hal_*_empty.c pattern); the real
    # Hv*/Cm* symbols come from libcmlib_rust.a linked below.
    set(CMLIB_KERNEL_STUB ${CMAKE_CURRENT_BINARY_DIR}/cmlib_rust_placeholder.c)
    file(WRITE ${CMLIB_KERNEL_STUB} "/* placeholder: kernel cmlib sources are in cmlib-rust (libcmlib_rust.a) */\n")
    list(APPEND CMLIB_KERNEL_SOURCE ${CMLIB_KERNEL_STUB})
    add_library(cmlib ${CMLIB_KERNEL_SOURCE})
    add_dependencies(cmlib bugcodes xdk)
    add_pch(cmlib cmlib.h CMLIB_KERNEL_SOURCE)

    # ── Rust integration: cmlib-rust → kernel cmlib cohabitation ──────────
    set(RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
    rust_storage_target(RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/cmlib-rust-target" cmlib-rust)
    # NOTE: unlike libcntpr's cohab hook, cmlib-rust does NOT use
    # `-C no-prepopulate-passes`. That flag leaves trivial code unoptimized, so an
    # iterator/`.offset()` loop emits external `call`s into precompiled-core
    # (Range::next, ptr-offset, panic machinery) which get pulled into the cohab
    # archive — the precompiled-std-in-kernel hazard (bootvid PoC). Optimizing
    # normally collapses each Hv* helper to a self-contained loop with no core
    # deps, which is what makes the precompiled-std cohab safe. (libcntpr needs
    # the flag because LLVM's whole-pipeline pass miscompiles its memset/strnicmp
    # shims; the smoke ud2 scan guards against any cmlib regression of that kind.)
    if(ARCH STREQUAL "i386")
        set(RUST_TARGET "i686-pc-windows-gnu")
        # i386 kernel runs no-SSE (CR4.OSFXSR unset in Phase 0).
        set(CMLIB_RUST_ENV "RUSTFLAGS=-C target-feature=-sse,-sse2")
    elseif(ARCH STREQUAL "amd64")
        set(RUST_TARGET "x86_64-pc-windows-gnu")
        # explicit empty assignment (not "") so `cmake -E env` gets a valid
        # NAME=VALUE, not an empty arg it would treat as the command.
        set(CMLIB_RUST_ENV "RUSTFLAGS=")
    elseif(ARCH STREQUAL "arm64")
        set(RUST_TARGET "aarch64-pc-windows-gnullvm")
        set(CMLIB_RUST_ENV "RUSTFLAGS=")
    endif()

    set(RUST_CMLIB_LIB "${CMAKE_CURRENT_BINARY_DIR}/libcmlib_rust.a")
    file(GLOB_RECURSE CMLIB_RUST_SOURCES CONFIGURE_DEPENDS
        ${RUST_WORKSPACE_DIR}/cmlib-rust/src/*.rs)
    add_custom_command(
        OUTPUT ${RUST_CMLIB_LIB}
        COMMAND ${CMAKE_COMMAND} -E env
            CARGO_TARGET_DIR=${RUST_TARGET_DIR} ${CMLIB_RUST_ENV}
            cargo build --release --target ${RUST_TARGET}
                -p cmlib-rust
                --manifest-path ${RUST_WORKSPACE_DIR}/Cargo.toml
        COMMAND ${CMAKE_COMMAND} -E copy
            ${RUST_TARGET_DIR}/${RUST_TARGET}/release/libcmlib_rust.a
            ${RUST_CMLIB_LIB}
        DEPENDS
            ${CMLIB_RUST_SOURCES}
            ${RUST_WORKSPACE_DIR}/cmlib-rust/Cargo.toml
            ${RUST_WORKSPACE_DIR}/Cargo.toml
        COMMENT "Building Rust cmlib umbrella (${RUST_TARGET})"
    )
    add_custom_target(rust_cmlib DEPENDS ${RUST_CMLIB_LIB})
    add_dependencies(cmlib rust_cmlib)
    target_link_libraries(cmlib PRIVATE ${RUST_CMLIB_LIB})
else()
    # CMLIB for host-tools
    add_definitions(
        -D__NO_CTYPE_INLINES
        -DCMLIB_HOST)
    add_library(cmlibhost STATIC ${SOURCE})
    target_include_directories(cmlibhost INTERFACE ${CMAKE_CURRENT_SOURCE_DIR})

    if(NOT MSVC)
        target_compile_options(cmlibhost PRIVATE -fshort-wchar -Wno-multichar)
    endif()

    target_link_libraries(cmlibhost PRIVATE host_includes)
endif()
