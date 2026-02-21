
add_definitions(
    -D_NTOSKRNL_
    -DNO_RTL_INLINES
    -D_NTSYSTEM_
    -D_NTDLLBUILD_)

if (CMAKE_C_COMPILER_ID STREQUAL "GNU" OR CMAKE_C_COMPILER_ID STREQUAL "Clang")
    # Enable this again. CORE-17637
    add_compile_options(-Wunused-result)

    add_compile_options(-Wno-incompatible-pointer-types)
    add_compile_options(-Wno-missing-braces)
endif()

list(APPEND RTL_WINE_SOURCE
    actctx.c
    timerqueue.c
    threadpool.c
)
set_source_files_properties(threadpool.c PROPERTIES COMPILE_DEFINITIONS __WINESRC__)

if(MSVC)
    # Silence warning C4267: 'initializing': conversion from 'size_t' to 'const int', possible loss of data
    set_source_files_properties(${RTL_WINE_SOURCE} PROPERTIES COMPILE_OPTIONS "/wd4267")
endif()

list(APPEND SOURCE
    ${RTL_WINE_SOURCE}
    wine_debug.c
    # access.c — replaced by Rust implementation (sdk/lib/rust/access/)
    acl.c
    appverifier.c
    assert.c
    atom.c
    # avltable.c — replaced by Rust implementation (sdk/lib/rust/avltable/)
    # bitmap.c — replaced by Rust implementation (sdk/lib/rust/bitmap/)
    bootdata.c
    # compress.c — replaced by Rust implementation (sdk/lib/rust/compress/)
    # crc32.c — replaced by Rust implementation (sdk/lib/rust/crc32/)
    critical.c
    dbgbuffer.c
    debug.c
    dos8dot3.c
    # encode.c — replaced by Rust implementation (sdk/lib/rust/encode/)
    env.c
    error.c
    exception.c
    # generictable.c — replaced by Rust implementation (sdk/lib/rust/generictable/)
    # handle.c — replaced by Rust implementation (sdk/lib/rust/handle/)
    heap.c
    heapdbg.c
    heappage.c
    heapuser.c
    image.c
    # interlck.c — replaced by Rust implementation (sdk/lib/rust/interlck/)
    # largeint.c — replaced by Rust implementation (sdk/lib/rust/largeint/)
    locale.c
    # luid.c — replaced by Rust implementation (sdk/lib/rust/luid/)
    memstream.c
    message.c
    network.c
    nls.c
    path.c
    ppb.c
    # prefix.c — replaced by Rust implementation (sdk/lib/rust/prefix/)
    priv.c
    process.c
    propvar.c
    # random.c — replaced by Rust implementation (sdk/lib/rust/random/)
    rangelist.c
    registry.c
    res.c
    resource.c
    rxact.c
    sd.c
    security.c
    sid.c
    slist.c
    # splaytree.c — replaced by Rust implementation (sdk/lib/rust/splaytree/)
    sysvol.c
    thread.c
    # time.c — replaced by Rust implementation (sdk/lib/rust/time/)
    timezone.c
    trace.c
    unicode.c
    unicodeprefix.c
    vectoreh.c
    version.c
    wait.c
    workitem.c
    rtl.h)

if(ARCH STREQUAL "i386")
    list(APPEND ASM_SOURCE
        i386/debug_asm.S
        i386/except_asm.s
        # i386/interlck.S — replaced by Rust SList (sdk/lib/rust/slist/)
        i386/rtlmem.s
        i386/rtlswap.S
        i386/res_asm.s)
    list(APPEND SOURCE
        i386/except.c
        i386/thread.c)
elseif(ARCH STREQUAL "amd64")
    list(APPEND ASM_SOURCE
        amd64/debug_asm.S
        amd64/except_asm.S
        # amd64/slist.S — replaced by Rust SList (sdk/lib/rust/slist/)
        amd64/unwind-asm.s)
    list(APPEND SOURCE
        bitmap64.c
        # byteswap.c — replaced by Rust implementation (sdk/lib/rust/byteswap/)
        amd64/dynfntbl.c
        amd64/except.c
        amd64/unwind.c
        amd64/stubs.c
        # mem.c — replaced by Rust implementation (sdk/lib/rust/mem/)
        )
elseif(ARCH STREQUAL "arm")
    list(APPEND ASM_SOURCE arm/debug_asm.S)
    list(APPEND SOURCE
        arm/except.c
        # byteswap.c — replaced by Rust implementation (sdk/lib/rust/byteswap/)
        # mem.c — replaced by Rust implementation (sdk/lib/rust/mem/)
        )
elseif(ARCH STREQUAL "arm64")
    list(APPEND ASM_SOURCE
        arm64/debug_asm.S)
    # arm64/slist.S replaced by Rust implementation (sdk/lib/rust/slist/)
    list(APPEND SOURCE
        arm64/except.c
        # byteswap.c — replaced by Rust implementation (sdk/lib/rust/byteswap/)
        # mem.c — replaced by Rust implementation (sdk/lib/rust/mem/)
        )
endif()

add_asm_files(rtl_asm ${ASM_SOURCE})
add_library(rtl ${SOURCE} ${rtl_asm})
target_link_libraries(rtl PRIVATE pseh)
add_pch(rtl rtl.h SOURCE)
add_dependencies(rtl psdk asm)

# Rust integration — map ARCH to Rust target triple (GNU/LLVM targets, not MSVC)
set(RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
set(RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-target")
if(ARCH STREQUAL "i386")
    set(RUST_TARGET "i686-pc-windows-gnu")
    # i386: disable SSE — CR4.OSFXSR not set until Phase 1 but pool init runs in Phase 0
    set(RUST_EXTRA_ENV "RUSTFLAGS=-C target-feature=-sse,-sse2")
elseif(ARCH STREQUAL "amd64")
    set(RUST_TARGET "x86_64-pc-windows-gnu")
    set(RUST_EXTRA_ENV "")
elseif(ARCH STREQUAL "arm64")
    set(RUST_TARGET "aarch64-pc-windows-gnullvm")
    set(RUST_EXTRA_ENV "")
endif()

# Rust AVL Table library (all architectures)
set(RUST_AVLTABLE_LIB "${CMAKE_CURRENT_BINARY_DIR}/libreactos_avltable.a")
add_custom_command(
    OUTPUT ${RUST_AVLTABLE_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${RUST_TARGET_DIR} ${RUST_EXTRA_ENV}
        cargo build --release --target ${RUST_TARGET}
            -p reactos-avltable
            --manifest-path ${RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${RUST_TARGET_DIR}/${RUST_TARGET}/release/libreactos_avltable.a
        ${RUST_AVLTABLE_LIB}
    DEPENDS ${RUST_WORKSPACE_DIR}/avltable/src/lib.rs
            ${RUST_WORKSPACE_DIR}/avltable/src/types.rs
            ${RUST_WORKSPACE_DIR}/avltable/src/tree.rs
            ${RUST_WORKSPACE_DIR}/avltable/Cargo.toml
            ${RUST_WORKSPACE_DIR}/Cargo.toml
    COMMENT "Building Rust AVL Table library (${RUST_TARGET})"
)
add_custom_target(rust_avltable DEPENDS ${RUST_AVLTABLE_LIB})
add_dependencies(rtl rust_avltable)
target_link_libraries(rtl PRIVATE ${RUST_AVLTABLE_LIB})

# Rust Bitmap library (all architectures)
set(RUST_BITMAP_LIB "${CMAKE_CURRENT_BINARY_DIR}/libreactos_bitmap.a")
add_custom_command(
    OUTPUT ${RUST_BITMAP_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${RUST_TARGET_DIR} ${RUST_EXTRA_ENV}
        cargo build --release --target ${RUST_TARGET}
            -p reactos-bitmap
            --manifest-path ${RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${RUST_TARGET_DIR}/${RUST_TARGET}/release/libreactos_bitmap.a
        ${RUST_BITMAP_LIB}
    DEPENDS ${RUST_WORKSPACE_DIR}/bitmap/src/lib.rs
            ${RUST_WORKSPACE_DIR}/bitmap/src/ops.rs
            ${RUST_WORKSPACE_DIR}/bitmap/Cargo.toml
            ${RUST_WORKSPACE_DIR}/Cargo.toml
    COMMENT "Building Rust Bitmap library (${RUST_TARGET})"
)
add_custom_target(rust_bitmap DEPENDS ${RUST_BITMAP_LIB})
add_dependencies(rtl rust_bitmap)
target_link_libraries(rtl PRIVATE ${RUST_BITMAP_LIB})

# Rust SList library (all architectures)
set(RUST_SLIST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libreactos_slist.a")
add_custom_command(
    OUTPUT ${RUST_SLIST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${RUST_TARGET_DIR} ${RUST_EXTRA_ENV}
        cargo build --release --target ${RUST_TARGET}
            -p reactos-slist
            --manifest-path ${RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${RUST_TARGET_DIR}/${RUST_TARGET}/release/libreactos_slist.a
        ${RUST_SLIST_LIB}
    DEPENDS ${RUST_WORKSPACE_DIR}/slist/src/lib.rs
            ${RUST_WORKSPACE_DIR}/slist/src/algorithm.rs
            ${RUST_WORKSPACE_DIR}/slist/src/atomics.rs
            ${RUST_WORKSPACE_DIR}/slist/src/header.rs
            ${RUST_WORKSPACE_DIR}/slist/Cargo.toml
            ${RUST_WORKSPACE_DIR}/Cargo.toml
    COMMENT "Building Rust SList library (${RUST_TARGET})"
)
add_custom_target(rust_slist DEPENDS ${RUST_SLIST_LIB})
add_dependencies(rtl rust_slist)
target_link_libraries(rtl PRIVATE ${RUST_SLIST_LIB})

# Rust Compress library (all architectures)
set(RUST_COMPRESS_LIB "${CMAKE_CURRENT_BINARY_DIR}/libreactos_compress.a")
add_custom_command(
    OUTPUT ${RUST_COMPRESS_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${RUST_TARGET_DIR} ${RUST_EXTRA_ENV}
        cargo build --release --target ${RUST_TARGET}
            -p reactos-compress
            --manifest-path ${RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${RUST_TARGET_DIR}/${RUST_TARGET}/release/libreactos_compress.a
        ${RUST_COMPRESS_LIB}
    DEPENDS ${RUST_WORKSPACE_DIR}/compress/src/lib.rs
            ${RUST_WORKSPACE_DIR}/compress/src/lznt1.rs
            ${RUST_WORKSPACE_DIR}/compress/Cargo.toml
            ${RUST_WORKSPACE_DIR}/Cargo.toml
    COMMENT "Building Rust Compress library (${RUST_TARGET})"
)
add_custom_target(rust_compress DEPENDS ${RUST_COMPRESS_LIB})
add_dependencies(rtl rust_compress)
target_link_libraries(rtl PRIVATE ${RUST_COMPRESS_LIB})

# Rust CRC32 library (all architectures)
set(RUST_CRC32_LIB "${CMAKE_CURRENT_BINARY_DIR}/libreactos_crc32.a")
add_custom_command(
    OUTPUT ${RUST_CRC32_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${RUST_TARGET_DIR} ${RUST_EXTRA_ENV}
        cargo build --release --target ${RUST_TARGET}
            -p reactos-crc32
            --manifest-path ${RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${RUST_TARGET_DIR}/${RUST_TARGET}/release/libreactos_crc32.a
        ${RUST_CRC32_LIB}
    DEPENDS ${RUST_WORKSPACE_DIR}/crc32/src/lib.rs
            ${RUST_WORKSPACE_DIR}/crc32/Cargo.toml
            ${RUST_WORKSPACE_DIR}/Cargo.toml
    COMMENT "Building Rust CRC32 library (${RUST_TARGET})"
)
add_custom_target(rust_crc32 DEPENDS ${RUST_CRC32_LIB})
add_dependencies(rtl rust_crc32)
target_link_libraries(rtl PRIVATE ${RUST_CRC32_LIB})

# Rust Random library (all architectures)
set(RUST_RANDOM_LIB "${CMAKE_CURRENT_BINARY_DIR}/libreactos_random.a")
add_custom_command(
    OUTPUT ${RUST_RANDOM_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${RUST_TARGET_DIR} ${RUST_EXTRA_ENV}
        cargo build --release --target ${RUST_TARGET}
            -p reactos-random
            --manifest-path ${RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${RUST_TARGET_DIR}/${RUST_TARGET}/release/libreactos_random.a
        ${RUST_RANDOM_LIB}
    DEPENDS ${RUST_WORKSPACE_DIR}/random/src/lib.rs
            ${RUST_WORKSPACE_DIR}/random/Cargo.toml
            ${RUST_WORKSPACE_DIR}/Cargo.toml
    COMMENT "Building Rust Random library (${RUST_TARGET})"
)
add_custom_target(rust_random DEPENDS ${RUST_RANDOM_LIB})
add_dependencies(rtl rust_random)
target_link_libraries(rtl PRIVATE ${RUST_RANDOM_LIB})

# Rust Encode library (all architectures)
set(RUST_ENCODE_LIB "${CMAKE_CURRENT_BINARY_DIR}/libreactos_encode.a")
add_custom_command(
    OUTPUT ${RUST_ENCODE_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${RUST_TARGET_DIR} ${RUST_EXTRA_ENV}
        cargo build --release --target ${RUST_TARGET}
            -p reactos-encode
            --manifest-path ${RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${RUST_TARGET_DIR}/${RUST_TARGET}/release/libreactos_encode.a
        ${RUST_ENCODE_LIB}
    DEPENDS ${RUST_WORKSPACE_DIR}/encode/src/lib.rs
            ${RUST_WORKSPACE_DIR}/encode/Cargo.toml
            ${RUST_WORKSPACE_DIR}/Cargo.toml
    COMMENT "Building Rust Encode library (${RUST_TARGET})"
)
add_custom_target(rust_encode DEPENDS ${RUST_ENCODE_LIB})
add_dependencies(rtl rust_encode)
target_link_libraries(rtl PRIVATE ${RUST_ENCODE_LIB})

# Rust LargeInt library (all architectures)
# Note: The C largeint.c had a linker hack (PVOID Dummy = RtlUshortByteSwap)
# to force-include RtlUshortByteSwap. With LLD, exported symbols from .spec
# files are included automatically, so this hack is not needed.
set(RUST_LARGEINT_LIB "${CMAKE_CURRENT_BINARY_DIR}/libreactos_largeint.a")
add_custom_command(
    OUTPUT ${RUST_LARGEINT_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${RUST_TARGET_DIR} ${RUST_EXTRA_ENV}
        cargo build --release --target ${RUST_TARGET}
            -p reactos-largeint
            --manifest-path ${RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${RUST_TARGET_DIR}/${RUST_TARGET}/release/libreactos_largeint.a
        ${RUST_LARGEINT_LIB}
    DEPENDS ${RUST_WORKSPACE_DIR}/largeint/src/lib.rs
            ${RUST_WORKSPACE_DIR}/largeint/Cargo.toml
            ${RUST_WORKSPACE_DIR}/Cargo.toml
    COMMENT "Building Rust LargeInt library (${RUST_TARGET})"
)
add_custom_target(rust_largeint DEPENDS ${RUST_LARGEINT_LIB})
add_dependencies(rtl rust_largeint)
target_link_libraries(rtl PRIVATE ${RUST_LARGEINT_LIB})

# Rust Access library (all architectures)
set(RUST_ACCESS_LIB "${CMAKE_CURRENT_BINARY_DIR}/libreactos_access.a")
add_custom_command(
    OUTPUT ${RUST_ACCESS_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${RUST_TARGET_DIR} ${RUST_EXTRA_ENV}
        cargo build --release --target ${RUST_TARGET}
            -p reactos-access
            --manifest-path ${RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${RUST_TARGET_DIR}/${RUST_TARGET}/release/libreactos_access.a
        ${RUST_ACCESS_LIB}
    DEPENDS ${RUST_WORKSPACE_DIR}/access/src/lib.rs
            ${RUST_WORKSPACE_DIR}/access/Cargo.toml
            ${RUST_WORKSPACE_DIR}/Cargo.toml
    COMMENT "Building Rust Access library (${RUST_TARGET})"
)
add_custom_target(rust_access DEPENDS ${RUST_ACCESS_LIB})
add_dependencies(rtl rust_access)
target_link_libraries(rtl PRIVATE ${RUST_ACCESS_LIB})

# Rust ByteSwap library (all architectures — i386 lib is empty, rtlswap.S provides those)
set(RUST_BYTESWAP_LIB "${CMAKE_CURRENT_BINARY_DIR}/libreactos_byteswap.a")
add_custom_command(
    OUTPUT ${RUST_BYTESWAP_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${RUST_TARGET_DIR} ${RUST_EXTRA_ENV}
        cargo build --release --target ${RUST_TARGET}
            -p reactos-byteswap
            --manifest-path ${RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${RUST_TARGET_DIR}/${RUST_TARGET}/release/libreactos_byteswap.a
        ${RUST_BYTESWAP_LIB}
    DEPENDS ${RUST_WORKSPACE_DIR}/byteswap/src/lib.rs
            ${RUST_WORKSPACE_DIR}/byteswap/Cargo.toml
            ${RUST_WORKSPACE_DIR}/Cargo.toml
    COMMENT "Building Rust ByteSwap library (${RUST_TARGET})"
)
add_custom_target(rust_byteswap DEPENDS ${RUST_BYTESWAP_LIB})
add_dependencies(rtl rust_byteswap)
target_link_libraries(rtl PRIVATE ${RUST_BYTESWAP_LIB})

# Rust Interlck library (all architectures)
set(RUST_INTERLCK_LIB "${CMAKE_CURRENT_BINARY_DIR}/libreactos_interlck.a")
add_custom_command(
    OUTPUT ${RUST_INTERLCK_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${RUST_TARGET_DIR} ${RUST_EXTRA_ENV}
        cargo build --release --target ${RUST_TARGET}
            -p reactos-interlck
            --manifest-path ${RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${RUST_TARGET_DIR}/${RUST_TARGET}/release/libreactos_interlck.a
        ${RUST_INTERLCK_LIB}
    DEPENDS ${RUST_WORKSPACE_DIR}/interlck/src/lib.rs
            ${RUST_WORKSPACE_DIR}/interlck/Cargo.toml
            ${RUST_WORKSPACE_DIR}/Cargo.toml
    COMMENT "Building Rust Interlck library (${RUST_TARGET})"
)
add_custom_target(rust_interlck DEPENDS ${RUST_INTERLCK_LIB})
add_dependencies(rtl rust_interlck)
target_link_libraries(rtl PRIVATE ${RUST_INTERLCK_LIB})

# Rust LUID library (all architectures)
set(RUST_LUID_LIB "${CMAKE_CURRENT_BINARY_DIR}/libreactos_luid.a")
add_custom_command(
    OUTPUT ${RUST_LUID_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${RUST_TARGET_DIR} ${RUST_EXTRA_ENV}
        cargo build --release --target ${RUST_TARGET}
            -p reactos-luid
            --manifest-path ${RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${RUST_TARGET_DIR}/${RUST_TARGET}/release/libreactos_luid.a
        ${RUST_LUID_LIB}
    DEPENDS ${RUST_WORKSPACE_DIR}/luid/src/lib.rs
            ${RUST_WORKSPACE_DIR}/luid/Cargo.toml
            ${RUST_WORKSPACE_DIR}/Cargo.toml
    COMMENT "Building Rust LUID library (${RUST_TARGET})"
)
add_custom_target(rust_luid DEPENDS ${RUST_LUID_LIB})
add_dependencies(rtl rust_luid)
target_link_libraries(rtl PRIVATE ${RUST_LUID_LIB})

# Rust Prefix library (all architectures)
set(RUST_PREFIX_LIB "${CMAKE_CURRENT_BINARY_DIR}/libreactos_prefix.a")
add_custom_command(
    OUTPUT ${RUST_PREFIX_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${RUST_TARGET_DIR} ${RUST_EXTRA_ENV}
        cargo build --release --target ${RUST_TARGET}
            -p reactos-prefix
            --manifest-path ${RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${RUST_TARGET_DIR}/${RUST_TARGET}/release/libreactos_prefix.a
        ${RUST_PREFIX_LIB}
    DEPENDS ${RUST_WORKSPACE_DIR}/prefix/src/lib.rs
            ${RUST_WORKSPACE_DIR}/prefix/Cargo.toml
            ${RUST_WORKSPACE_DIR}/Cargo.toml
    COMMENT "Building Rust Prefix library (${RUST_TARGET})"
)
add_custom_target(rust_prefix DEPENDS ${RUST_PREFIX_LIB})
add_dependencies(rtl rust_prefix)
target_link_libraries(rtl PRIVATE ${RUST_PREFIX_LIB})

# Rust Mem library (all architectures — i386 lib is empty, rtlmem.s provides those)
set(RUST_MEM_LIB "${CMAKE_CURRENT_BINARY_DIR}/libreactos_mem.a")
add_custom_command(
    OUTPUT ${RUST_MEM_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${RUST_TARGET_DIR} ${RUST_EXTRA_ENV}
        cargo build --release --target ${RUST_TARGET}
            -p reactos-mem
            --manifest-path ${RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${RUST_TARGET_DIR}/${RUST_TARGET}/release/libreactos_mem.a
        ${RUST_MEM_LIB}
    DEPENDS ${RUST_WORKSPACE_DIR}/mem/src/lib.rs
            ${RUST_WORKSPACE_DIR}/mem/Cargo.toml
            ${RUST_WORKSPACE_DIR}/Cargo.toml
    COMMENT "Building Rust Mem library (${RUST_TARGET})"
)
add_custom_target(rust_mem DEPENDS ${RUST_MEM_LIB})
add_dependencies(rtl rust_mem)
target_link_libraries(rtl PRIVATE ${RUST_MEM_LIB})

# Rust Handle library (all architectures)
set(RUST_HANDLE_LIB "${CMAKE_CURRENT_BINARY_DIR}/libreactos_handle.a")
add_custom_command(
    OUTPUT ${RUST_HANDLE_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${RUST_TARGET_DIR} ${RUST_EXTRA_ENV}
        cargo build --release --target ${RUST_TARGET}
            -p reactos-handle
            --manifest-path ${RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${RUST_TARGET_DIR}/${RUST_TARGET}/release/libreactos_handle.a
        ${RUST_HANDLE_LIB}
    DEPENDS ${RUST_WORKSPACE_DIR}/handle/src/lib.rs
            ${RUST_WORKSPACE_DIR}/handle/Cargo.toml
            ${RUST_WORKSPACE_DIR}/Cargo.toml
    COMMENT "Building Rust Handle library (${RUST_TARGET})"
)
add_custom_target(rust_handle DEPENDS ${RUST_HANDLE_LIB})
add_dependencies(rtl rust_handle)
target_link_libraries(rtl PRIVATE ${RUST_HANDLE_LIB})

# Rust Time library (all architectures)
set(RUST_TIME_LIB "${CMAKE_CURRENT_BINARY_DIR}/libreactos_time.a")
add_custom_command(
    OUTPUT ${RUST_TIME_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${RUST_TARGET_DIR} ${RUST_EXTRA_ENV}
        cargo build --release --target ${RUST_TARGET}
            -p reactos-time
            --manifest-path ${RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${RUST_TARGET_DIR}/${RUST_TARGET}/release/libreactos_time.a
        ${RUST_TIME_LIB}
    DEPENDS ${RUST_WORKSPACE_DIR}/time/src/lib.rs
            ${RUST_WORKSPACE_DIR}/time/Cargo.toml
            ${RUST_WORKSPACE_DIR}/Cargo.toml
    COMMENT "Building Rust Time library (${RUST_TARGET})"
)
add_custom_target(rust_time DEPENDS ${RUST_TIME_LIB})
add_dependencies(rtl rust_time)
target_link_libraries(rtl PRIVATE ${RUST_TIME_LIB})

# Rust SplayTree library (all architectures)
set(RUST_SPLAYTREE_LIB "${CMAKE_CURRENT_BINARY_DIR}/libreactos_splaytree.a")
add_custom_command(
    OUTPUT ${RUST_SPLAYTREE_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${RUST_TARGET_DIR} ${RUST_EXTRA_ENV}
        cargo build --release --target ${RUST_TARGET}
            -p reactos-splaytree
            --manifest-path ${RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${RUST_TARGET_DIR}/${RUST_TARGET}/release/libreactos_splaytree.a
        ${RUST_SPLAYTREE_LIB}
    DEPENDS ${RUST_WORKSPACE_DIR}/splaytree/src/lib.rs
            ${RUST_WORKSPACE_DIR}/splaytree/Cargo.toml
            ${RUST_WORKSPACE_DIR}/Cargo.toml
    COMMENT "Building Rust SplayTree library (${RUST_TARGET})"
)
add_custom_target(rust_splaytree DEPENDS ${RUST_SPLAYTREE_LIB})
add_dependencies(rtl rust_splaytree)
target_link_libraries(rtl PRIVATE ${RUST_SPLAYTREE_LIB})

# Rust GenericTable library (all architectures)
set(RUST_GENERICTABLE_LIB "${CMAKE_CURRENT_BINARY_DIR}/libreactos_generictable.a")
add_custom_command(
    OUTPUT ${RUST_GENERICTABLE_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${RUST_TARGET_DIR} ${RUST_EXTRA_ENV}
        cargo build --release --target ${RUST_TARGET}
            -p reactos-generictable
            --manifest-path ${RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${RUST_TARGET_DIR}/${RUST_TARGET}/release/libreactos_generictable.a
        ${RUST_GENERICTABLE_LIB}
    DEPENDS ${RUST_WORKSPACE_DIR}/generictable/src/lib.rs
            ${RUST_WORKSPACE_DIR}/generictable/Cargo.toml
            ${RUST_WORKSPACE_DIR}/Cargo.toml
    COMMENT "Building Rust GenericTable library (${RUST_TARGET})"
)
add_custom_target(rust_generictable DEPENDS ${RUST_GENERICTABLE_LIB})
add_dependencies(rtl rust_generictable)
target_link_libraries(rtl PRIVATE ${RUST_GENERICTABLE_LIB})

list(APPEND SOURCE_VISTA
    condvar.c
    locale.c
    runonce.c
    srw.c
    # utf8.c — replaced by Rust implementation (sdk/lib/rust/utf8/)
    )

add_library(rtl_vista ${SOURCE_VISTA})
add_pch(rtl_vista rtl_vista.h SOURCE_VISTA)
add_dependencies(rtl_vista psdk)
target_link_libraries(rtl_vista PRIVATE pseh)

# Rust UTF-8 library (all architectures) — links into rtl_vista
set(RUST_UTF8_LIB "${CMAKE_CURRENT_BINARY_DIR}/libreactos_utf8.a")
add_custom_command(
    OUTPUT ${RUST_UTF8_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${RUST_TARGET_DIR} ${RUST_EXTRA_ENV}
        cargo build --release --target ${RUST_TARGET}
            -p reactos-utf8
            --manifest-path ${RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${RUST_TARGET_DIR}/${RUST_TARGET}/release/libreactos_utf8.a
        ${RUST_UTF8_LIB}
    DEPENDS ${RUST_WORKSPACE_DIR}/utf8/src/lib.rs
            ${RUST_WORKSPACE_DIR}/utf8/src/conv.rs
            ${RUST_WORKSPACE_DIR}/utf8/Cargo.toml
            ${RUST_WORKSPACE_DIR}/Cargo.toml
    COMMENT "Building Rust UTF-8 library (${RUST_TARGET})"
)
add_custom_target(rust_utf8 DEPENDS ${RUST_UTF8_LIB})
add_dependencies(rtl_vista rust_utf8)
target_link_libraries(rtl_vista PRIVATE ${RUST_UTF8_LIB})
