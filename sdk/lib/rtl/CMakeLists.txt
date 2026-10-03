
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
    # actctx.c — replaced by Rust implementation (sdk/lib/rust/rtl-actctx/ cabi)
    # timerqueue.c — replaced by Rust implementation (sdk/lib/rust/rtl-threadpool/)
    # threadpool.c — replaced by Rust implementation (sdk/lib/rust/rtl-threadpool/)
)
# threadpool.c was the only __WINESRC__ source; it is now Rust (rtl-threadpool)

if(MSVC)
    # Silence warning C4267: 'initializing': conversion from 'size_t' to 'const int', possible loss of data
    set_source_files_properties(${RTL_WINE_SOURCE} PROPERTIES COMPILE_OPTIONS "/wd4267")
endif()

list(APPEND SOURCE
    ${RTL_WINE_SOURCE}
    dbgprint_variadic.c  # C-variadic trampolines for rtl-diag (DbgPrint/DbgPrintEx/DbgPrintReturnControlC + wine_dbg_*)
    # wine_debug.c — replaced by Rust (sdk/lib/rust/rtl-diag/); C-variadic shims in dbgprint_variadic.c
    # access.c — replaced by Rust implementation (sdk/lib/rust/access/)
    # acl.c — replaced by Rust implementation (sdk/lib/rust/rtl-strings/ cabi, Wave ACL)
    # appverifier.c — replaced by Rust (sdk/lib/rust/rtl-diag/); C-variadic shims in dbgprint_variadic.c
    # assert.c — replaced by Rust (sdk/lib/rust/rtl-diag/); C-variadic shims in dbgprint_variadic.c
    # atom.c — replaced by Rust implementation (sdk/lib/rust/rtl-strings/ cabi, Wave ATM)
    # avltable.c — replaced by Rust implementation (sdk/lib/rust/avltable/)
    # bitmap.c — replaced by Rust implementation (sdk/lib/rust/bitmap/)
    # bootdata.c — replaced by Rust implementation (sdk/lib/rust/rtl-bootdata/)
    # compress.c — replaced by Rust implementation (sdk/lib/rust/compress/)
    # crc32.c — replaced by Rust implementation (sdk/lib/rust/crc32/)
    # critical.c — replaced by Rust implementation (sdk/lib/rust/rtl-sync/, Wave SYNC)
    # dbgbuffer.c — replaced by Rust (sdk/lib/rust/rtl-diag/); C-variadic shims in dbgprint_variadic.c
    # debug.c — replaced by Rust (sdk/lib/rust/rtl-diag/); C-variadic shims in dbgprint_variadic.c
    # dos8dot3.c — replaced by Rust implementation (sdk/lib/rust/rtl-strings/ cabi)
    # encode.c — replaced by Rust implementation (sdk/lib/rust/encode/)
    # env.c — replaced by Rust implementation (sdk/lib/rust/rtl-strings/ cabi, Wave ENV)
    # error.c — replaced by Rust implementation (sdk/lib/rust/rtl-error/)
    # exception.c — replaced by Rust (sdk/lib/rust/rtl-diag/); C-variadic shims in dbgprint_variadic.c
    # generictable.c — replaced by Rust implementation (sdk/lib/rust/generictable/)
    # handle.c — replaced by Rust implementation (sdk/lib/rust/handle/)
    # heap.c — replaced by Rust implementation (sdk/lib/rust/rtl-heap/, Wave HEAP)
    # heapdbg.c — replaced by Rust implementation (sdk/lib/rust/rtl-heap/heapdbg.rs)
    # heappage.c — replaced by Rust implementation (sdk/lib/rust/rtl-heappage/)
    # heapuser.c — replaced by Rust implementation (sdk/lib/rust/rtl-heapuser/)
    # image.c — replaced by Rust (sdk/lib/rust/rtl-diag/); C-variadic shims in dbgprint_variadic.c
    # interlck.c — replaced by Rust implementation (sdk/lib/rust/interlck/)
    # largeint.c — replaced by Rust implementation (sdk/lib/rust/largeint/)
    # locale.c — replaced by Rust implementation (sdk/lib/rust/rtl-sync/, Wave SYNC-locale)
    # luid.c — replaced by Rust implementation (sdk/lib/rust/luid/)
    # memres.c (upstream-new, 2026) — replaced by Rust implementation (sdk/lib/rust/rtl-misc/
    #   memres.rs): the Rtl{Cm,Io}{En,De}codeMemIoResource family + RtlFindClosestEncodableLength,
    #   which ntoskrnl.spec now exports (ntoskrnl gets its rtl surface from rtl-rust).
    #memres.c
    # memstream.c — replaced by Rust implementation (sdk/lib/rust/rtl-misc/, Wave MISC)
    # message.c — replaced by Rust (sdk/lib/rust/rtl-diag/); C-variadic shims in dbgprint_variadic.c
    # network.c — replaced by Rust implementation (sdk/lib/rust/network/, Wave NET: RtlIpv4*/RtlIpv6*)
    # nls.c — replaced by Rust implementation (sdk/lib/rust/rtl-strings/ cabi + nls_tables)
    # path.c — replaced by Rust implementation (sdk/lib/rust/rtl-strings/ cabi, Wave PATH)
    # ppb.c — replaced by Rust implementation (sdk/lib/rust/rtl-ppb/)
    # prefix.c — replaced by Rust implementation (sdk/lib/rust/prefix/)
    # priv.c — replaced by Rust implementation (sdk/lib/rust/rtl-strings/ cabi, Wave PRIV)
    # process.c — replaced by Rust implementation (sdk/lib/rust/rtl-strings/ cabi, Wave PRC)
    # processor.c (upstream-new, 2026) — replaced by Rust (rtl-strings cabi): it defines
    #   RtlGetCurrentProcessorNumber, which rtl-strings already owned, plus the two new
    #   exports RtlIsProcessorFeaturePresent and RtlGetCurrentProcessorNumberEx, now
    #   ported beside it.
    # propvar.c — replaced by Rust implementation (sdk/lib/rust/rtl-propvar/)
    # random.c — replaced by Rust implementation (sdk/lib/rust/random/)
    # rangelist.c — replaced by Rust implementation (sdk/lib/rust/rtl-rangelist/)
    # registry.c — replaced by Rust implementation (sdk/lib/rust/rtl-misc/, Wave MISC)
    # res.c — replaced by Rust implementation (sdk/lib/rust/rtl-misc/, Wave MISC)
    # resource.c — replaced by Rust implementation (sdk/lib/rust/rtl-strings/ cabi, Wave RES)
    # rxact.c — replaced by Rust implementation (sdk/lib/rust/rtl-rxact/)
    # sd.c — replaced by Rust implementation (sdk/lib/rust/rtl-strings/ cabi, Wave SD)
    # security.c — replaced by Rust implementation (sdk/lib/rust/rtl-strings/ cabi, Wave SEC)
    # sid.c — replaced by Rust implementation (sdk/lib/rust/rtl-strings/ cabi, Wave SID)
    # slist.c — replaced by Rust implementation (sdk/lib/rust/slist/, Wave SLIST: Init/First/QueryDepth/PushList + RtlpUse16ByteSLists; Push/Pop/Flush already Rust)
    # splaytree.c — replaced by Rust implementation (sdk/lib/rust/splaytree/)
    # sysvol.c — replaced by Rust implementation (sdk/lib/rust/rtl-sysvol/)
    # thread.c — replaced by Rust implementation (sdk/lib/rust/rtl-strings/ cabi, Wave THR)
    # time.c — replaced by Rust implementation (sdk/lib/rust/time/)
    # timezone.c — replaced by Rust implementation (sdk/lib/rust/rtl-strings/ cabi, Wave TZ)
    # trace.c — replaced by Rust (sdk/lib/rust/rtl-diag/); C-variadic shims in dbgprint_variadic.c
    # unicode.c — replaced by Rust implementation (sdk/lib/rust/rtl-strings/ cabi)
    # unicodeprefix.c — replaced by Rust implementation (sdk/lib/rust/uniprefix/)
    # vectoreh.c — replaced by Rust (sdk/lib/rust/rtl-diag/); C-variadic shims in dbgprint_variadic.c
    # version.c — replaced by Rust implementation (sdk/lib/rust/rtl-version/)
    # wait.c — replaced by Rust implementation (sdk/lib/rust/rtl-sync/, Wave SYNC)
    # workitem.c — replaced by Rust implementation (sdk/lib/rust/rtl-threadpool/)
    rtl.h)

if(ARCH STREQUAL "i386")
    list(APPEND ASM_SOURCE
        i386/debug_asm.S
        i386/except_asm.s
        # i386/interlck.S — replaced by Rust SList (sdk/lib/rust/slist/)
        i386/rtlmem.s
        i386/rtlswap.S)
        # i386/res_asm.s — replaced by Rust (sdk/lib/rust/rtl-misc/src/res.rs: LdrAccessResource@16).
        # The asm stub's _LdrpAccessResource@16 callee lived in res.c.obj (now Rust); keeping
        # the asm caused an undefined symbol once the stale fat librtl.a was regenerated lean.
    # i386/except.c — replaced by Rust implementation (sdk/lib/rust/rtl-except/src/i386/except.rs)
    # i386/thread.c — replaced by Rust implementation (sdk/lib/rust/rtl-except/src/i386/thread.rs)
elseif(ARCH STREQUAL "amd64")
    list(APPEND ASM_SOURCE
        amd64/debug_asm.S
        amd64/except_asm.S
        # amd64/slist.S — replaced by Rust SList (sdk/lib/rust/slist/)
        amd64/unwind-asm.s)
    list(APPEND SOURCE
        # bitmap64.c — replaced by Rust implementation (sdk/lib/rust/bitmap/ ops64)
        # byteswap.c — replaced by Rust implementation (sdk/lib/rust/byteswap/)
        # amd64/dynfntbl.c — replaced by Rust implementation (sdk/lib/rust/rtl-except/src/amd64/dynfntbl.rs)
        # amd64/except.c — replaced by Rust implementation (sdk/lib/rust/rtl-except/src/amd64/except.rs)
        # amd64/unwind.c — replaced by Rust implementation (sdk/lib/rust/rtl-except/src/amd64/unwind.rs)
        # amd64/stubs.c — replaced by Rust implementation (sdk/lib/rust/rtl-except/src/amd64/stubs.rs)
        # mem.c — replaced by Rust implementation (sdk/lib/rust/mem/)
        )
elseif(ARCH STREQUAL "arm64")
    list(APPEND ASM_SOURCE
        arm64/debug_asm.S)
    # arm64/slist.S replaced by Rust implementation (sdk/lib/rust/slist/)
    list(APPEND SOURCE
        # arm64/except.c — replaced by Rust implementation (sdk/lib/rust/rtl-except/src/arm64/except.rs)
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
    set(RUST_EXTRA_ENV "RUSTFLAGS=-C target-feature=-sse,-sse2 -C no-prepopulate-passes")
elseif(ARCH STREQUAL "amd64")
    set(RUST_TARGET "x86_64-pc-windows-gnu")
    set(RUST_EXTRA_ENV "")
elseif(ARCH STREQUAL "arm64")
    set(RUST_TARGET "aarch64-pc-windows-gnullvm")
    set(RUST_EXTRA_ENV "")
endif()

# Rust RTL umbrella (all architectures) — combines 18 algorithm rlibs
# (access, avltable, bitmap, byteswap, compress, crc32, encode,
# generictable, handle, interlck, largeint, luid, mem, prefix, random,
# slist, splaytree, time) into one staticlib that feeds librtl.a.
set(RUST_RTL_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/librtl_rust.a")
add_custom_command(
    OUTPUT ${RUST_RTL_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${RUST_TARGET_DIR} ${RUST_EXTRA_ENV}
        cargo build --release --target ${RUST_TARGET}
            -p rtl-rust
            --manifest-path ${RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${RUST_TARGET_DIR}/${RUST_TARGET}/release/librtl_rust.a
        ${RUST_RTL_RUST_LIB}
    DEPENDS
            ${RUST_WORKSPACE_DIR}/access/src/lib.rs
            ${RUST_WORKSPACE_DIR}/access/Cargo.toml
            ${RUST_WORKSPACE_DIR}/avltable/src/lib.rs
            ${RUST_WORKSPACE_DIR}/avltable/src/tree.rs
            ${RUST_WORKSPACE_DIR}/avltable/src/types.rs
            ${RUST_WORKSPACE_DIR}/avltable/Cargo.toml
            ${RUST_WORKSPACE_DIR}/bitmap/src/lib.rs
            ${RUST_WORKSPACE_DIR}/bitmap/src/ops.rs
            ${RUST_WORKSPACE_DIR}/bitmap/Cargo.toml
            ${RUST_WORKSPACE_DIR}/byteswap/src/lib.rs
            ${RUST_WORKSPACE_DIR}/byteswap/Cargo.toml
            ${RUST_WORKSPACE_DIR}/compress/src/lib.rs
            ${RUST_WORKSPACE_DIR}/compress/src/lznt1.rs
            ${RUST_WORKSPACE_DIR}/compress/Cargo.toml
            ${RUST_WORKSPACE_DIR}/crc32/src/lib.rs
            ${RUST_WORKSPACE_DIR}/crc32/Cargo.toml
            ${RUST_WORKSPACE_DIR}/encode/src/lib.rs
            ${RUST_WORKSPACE_DIR}/encode/Cargo.toml
            ${RUST_WORKSPACE_DIR}/generictable/src/lib.rs
            ${RUST_WORKSPACE_DIR}/generictable/Cargo.toml
            ${RUST_WORKSPACE_DIR}/handle/src/lib.rs
            ${RUST_WORKSPACE_DIR}/handle/Cargo.toml
            ${RUST_WORKSPACE_DIR}/interlck/src/lib.rs
            ${RUST_WORKSPACE_DIR}/interlck/Cargo.toml
            ${RUST_WORKSPACE_DIR}/largeint/src/lib.rs
            ${RUST_WORKSPACE_DIR}/largeint/Cargo.toml
            ${RUST_WORKSPACE_DIR}/luid/src/lib.rs
            ${RUST_WORKSPACE_DIR}/luid/Cargo.toml
            ${RUST_WORKSPACE_DIR}/mem/src/lib.rs
            ${RUST_WORKSPACE_DIR}/mem/Cargo.toml
            ${RUST_WORKSPACE_DIR}/prefix/src/lib.rs
            ${RUST_WORKSPACE_DIR}/prefix/Cargo.toml
            ${RUST_WORKSPACE_DIR}/random/src/lib.rs
            ${RUST_WORKSPACE_DIR}/random/Cargo.toml
            ${RUST_WORKSPACE_DIR}/slist/src/algorithm.rs
            ${RUST_WORKSPACE_DIR}/slist/src/atomics.rs
            ${RUST_WORKSPACE_DIR}/slist/src/header.rs
            ${RUST_WORKSPACE_DIR}/slist/src/lib.rs
            ${RUST_WORKSPACE_DIR}/slist/Cargo.toml
            ${RUST_WORKSPACE_DIR}/splaytree/src/lib.rs
            ${RUST_WORKSPACE_DIR}/splaytree/Cargo.toml
            ${RUST_WORKSPACE_DIR}/uniprefix/src/lib.rs
            ${RUST_WORKSPACE_DIR}/uniprefix/Cargo.toml
            ${RUST_WORKSPACE_DIR}/rtl-heap/src/lib.rs
            ${RUST_WORKSPACE_DIR}/rtl-heap/src/types.rs
            ${RUST_WORKSPACE_DIR}/rtl-heap/src/heap.rs
            ${RUST_WORKSPACE_DIR}/rtl-heap/src/heapdbg.rs
            ${RUST_WORKSPACE_DIR}/rtl-heap/Cargo.toml
            ${RUST_WORKSPACE_DIR}/rtl-version/src/lib.rs
            ${RUST_WORKSPACE_DIR}/rtl-version/Cargo.toml
            ${RUST_WORKSPACE_DIR}/rtl-ppb/src/lib.rs
            ${RUST_WORKSPACE_DIR}/rtl-ppb/Cargo.toml
            ${RUST_WORKSPACE_DIR}/rtl-propvar/src/lib.rs
            ${RUST_WORKSPACE_DIR}/rtl-propvar/Cargo.toml
            ${RUST_WORKSPACE_DIR}/rtl-bootdata/src/lib.rs
            ${RUST_WORKSPACE_DIR}/rtl-bootdata/Cargo.toml
            ${RUST_WORKSPACE_DIR}/rtl-rangelist/src/lib.rs
            ${RUST_WORKSPACE_DIR}/rtl-rangelist/Cargo.toml
            ${RUST_WORKSPACE_DIR}/rtl-sysvol/src/lib.rs
            ${RUST_WORKSPACE_DIR}/rtl-sysvol/Cargo.toml
            ${RUST_WORKSPACE_DIR}/rtl-rxact/src/lib.rs
            ${RUST_WORKSPACE_DIR}/rtl-rxact/Cargo.toml
            ${RUST_WORKSPACE_DIR}/rtl-error/src/lib.rs
            ${RUST_WORKSPACE_DIR}/rtl-error/src/winerr.rs
            ${RUST_WORKSPACE_DIR}/rtl-error/src/tables.rs
            ${RUST_WORKSPACE_DIR}/rtl-error/Cargo.toml
            ${RUST_WORKSPACE_DIR}/rtl-heappage/src/lib.rs
            ${RUST_WORKSPACE_DIR}/rtl-heappage/src/heappage.rs
            ${RUST_WORKSPACE_DIR}/rtl-heappage/Cargo.toml
            ${RUST_WORKSPACE_DIR}/rtl-heapuser/src/lib.rs
            ${RUST_WORKSPACE_DIR}/rtl-heapuser/src/heapuser.rs
            ${RUST_WORKSPACE_DIR}/rtl-heapuser/Cargo.toml
            ${RUST_WORKSPACE_DIR}/rtl-strings/src/lib.rs
            ${RUST_WORKSPACE_DIR}/rtl-strings/src/cabi/mod.rs
            ${RUST_WORKSPACE_DIR}/rtl-strings/src/cabi/nls_tables.rs
            ${RUST_WORKSPACE_DIR}/rtl-strings/src/core_rs/mod.rs
            ${RUST_WORKSPACE_DIR}/rtl-strings/src/core_rs/conv.rs
            ${RUST_WORKSPACE_DIR}/rtl-strings/src/core_rs/dos8dot3.rs
            ${RUST_WORKSPACE_DIR}/rtl-strings/src/core_rs/env.rs
            ${RUST_WORKSPACE_DIR}/rtl-strings/src/core_rs/misc.rs
            ${RUST_WORKSPACE_DIR}/rtl-strings/src/core_rs/nls.rs
            ${RUST_WORKSPACE_DIR}/rtl-strings/src/core_rs/path.rs
            ${RUST_WORKSPACE_DIR}/rtl-strings/src/core_rs/priv_mod.rs
            ${RUST_WORKSPACE_DIR}/rtl-strings/src/core_rs/process.rs
            ${RUST_WORKSPACE_DIR}/rtl-strings/src/core_rs/resource.rs
            ${RUST_WORKSPACE_DIR}/rtl-strings/src/core_rs/acl.rs
            ${RUST_WORKSPACE_DIR}/rtl-strings/src/core_rs/sd.rs
            ${RUST_WORKSPACE_DIR}/rtl-strings/src/core_rs/sec.rs
            ${RUST_WORKSPACE_DIR}/rtl-strings/src/core_rs/sid.rs
            ${RUST_WORKSPACE_DIR}/rtl-strings/src/core_rs/thread.rs
            ${RUST_WORKSPACE_DIR}/rtl-strings/src/core_rs/timezone.rs
            ${RUST_WORKSPACE_DIR}/rtl-strings/src/core_rs/ustr.rs
            ${RUST_WORKSPACE_DIR}/rtl-strings/src/core_rs/utf8.rs
            ${RUST_WORKSPACE_DIR}/rtl-strings/Cargo.toml
            ${RUST_WORKSPACE_DIR}/rtl-actctx/src/lib.rs
            ${RUST_WORKSPACE_DIR}/rtl-actctx/src/types.rs
            ${RUST_WORKSPACE_DIR}/rtl-actctx/src/shim.rs
            ${RUST_WORKSPACE_DIR}/rtl-actctx/src/strings.rs
            ${RUST_WORKSPACE_DIR}/rtl-actctx/src/actctx.rs
            ${RUST_WORKSPACE_DIR}/rtl-actctx/Cargo.toml
            ${RUST_WORKSPACE_DIR}/rtl-diag/src/lib.rs
            ${RUST_WORKSPACE_DIR}/rtl-diag/src/shim.rs
            ${RUST_WORKSPACE_DIR}/rtl-diag/src/image.rs
            ${RUST_WORKSPACE_DIR}/rtl-diag/src/assert.rs
            ${RUST_WORKSPACE_DIR}/rtl-diag/src/appverifier.rs
            ${RUST_WORKSPACE_DIR}/rtl-diag/src/trace.rs
            ${RUST_WORKSPACE_DIR}/rtl-diag/src/message.rs
            ${RUST_WORKSPACE_DIR}/rtl-diag/src/debug.rs
            ${RUST_WORKSPACE_DIR}/rtl-diag/src/dbgbuffer.rs
            ${RUST_WORKSPACE_DIR}/rtl-diag/src/wine_debug.rs
            ${RUST_WORKSPACE_DIR}/rtl-diag/src/exception.rs
            ${RUST_WORKSPACE_DIR}/rtl-diag/src/vectoreh.rs
            ${RUST_WORKSPACE_DIR}/rtl-diag/Cargo.toml
            ${RUST_WORKSPACE_DIR}/rtl-except/src/lib.rs
            ${RUST_WORKSPACE_DIR}/rtl-except/src/types.rs
            ${RUST_WORKSPACE_DIR}/rtl-except/src/amd64/mod.rs
            ${RUST_WORKSPACE_DIR}/rtl-except/src/amd64/unwind.rs
            ${RUST_WORKSPACE_DIR}/rtl-except/src/amd64/except.rs
            ${RUST_WORKSPACE_DIR}/rtl-except/src/amd64/stubs.rs
            ${RUST_WORKSPACE_DIR}/rtl-except/src/amd64/dynfntbl.rs
            ${RUST_WORKSPACE_DIR}/rtl-except/src/i386/mod.rs
            ${RUST_WORKSPACE_DIR}/rtl-except/src/i386/except.rs
            ${RUST_WORKSPACE_DIR}/rtl-except/src/i386/thread.rs
            ${RUST_WORKSPACE_DIR}/rtl-except/src/arm64/mod.rs
            ${RUST_WORKSPACE_DIR}/rtl-except/src/arm64/except.rs
            ${RUST_WORKSPACE_DIR}/rtl-except/Cargo.toml
            ${RUST_WORKSPACE_DIR}/rtl-misc/src/lib.rs
            ${RUST_WORKSPACE_DIR}/rtl-misc/src/shared.rs
            ${RUST_WORKSPACE_DIR}/rtl-misc/src/res.rs
            ${RUST_WORKSPACE_DIR}/rtl-misc/src/memstream.rs
            ${RUST_WORKSPACE_DIR}/rtl-misc/src/registry.rs
            ${RUST_WORKSPACE_DIR}/rtl-misc/src/memres.rs
            ${RUST_WORKSPACE_DIR}/rtl-misc/Cargo.toml
            ${RUST_WORKSPACE_DIR}/rtl-threadpool/src/lib.rs
            ${RUST_WORKSPACE_DIR}/rtl-threadpool/src/types.rs
            ${RUST_WORKSPACE_DIR}/rtl-threadpool/src/wine_list.rs
            ${RUST_WORKSPACE_DIR}/rtl-threadpool/src/threadpool.rs
            ${RUST_WORKSPACE_DIR}/rtl-threadpool/src/workitem.rs
            ${RUST_WORKSPACE_DIR}/rtl-threadpool/src/timerqueue.rs
            ${RUST_WORKSPACE_DIR}/rtl-threadpool/Cargo.toml
            ${RUST_WORKSPACE_DIR}/rtl-sync/src/lib.rs
            ${RUST_WORKSPACE_DIR}/rtl-sync/src/shared.rs
            ${RUST_WORKSPACE_DIR}/rtl-sync/src/critical.rs
            ${RUST_WORKSPACE_DIR}/rtl-sync/src/wait.rs
            ${RUST_WORKSPACE_DIR}/rtl-sync/src/atomics.rs
            ${RUST_WORKSPACE_DIR}/rtl-sync/src/srw.rs
            ${RUST_WORKSPACE_DIR}/rtl-sync/src/condvar.rs
            ${RUST_WORKSPACE_DIR}/rtl-sync/src/runonce.rs
            ${RUST_WORKSPACE_DIR}/rtl-sync/src/locale.rs
            ${RUST_WORKSPACE_DIR}/rtl-sync/Cargo.toml
            ${RUST_WORKSPACE_DIR}/cnt-rust/src/lib.rs
            ${RUST_WORKSPACE_DIR}/cnt-rust/Cargo.toml
            ${RUST_WORKSPACE_DIR}/time/src/lib.rs
            ${RUST_WORKSPACE_DIR}/time/Cargo.toml
            ${RUST_WORKSPACE_DIR}/rtl-rust/src/lib.rs
            ${RUST_WORKSPACE_DIR}/rtl-rust/Cargo.toml
            ${RUST_WORKSPACE_DIR}/Cargo.toml
    COMMENT "Building Rust RTL umbrella (${RUST_TARGET})"
)
add_custom_target(rust_rtl DEPENDS ${RUST_RTL_RUST_LIB})
add_dependencies(rtl rust_rtl)
target_link_libraries(rtl PRIVATE ${RUST_RTL_RUST_LIB})

    # utf8.c — replaced by Rust (utf8 crate, linked below).
    # condvar.c / runonce.c / srw.c / locale.c — replaced by Rust implementation
    #   (sdk/lib/rust/rtl-sync/, Wave SYNC + the `vista` feature). The standalone
    #   ntdll_vista.dll forwarder now links the rtl-sync `vista` archive directly
    #   via reactos-link `.rust_lib("rtl-sync", ["vista"])` (see
    #   dll/ntdll/nt_0600/ntdll_vista-cargo/build.rs) — it no longer links
    #   librtl_vista.a. The only other consumer, rtl_unittest, resolves
    #   RtlInitializeConditionVariable / RtlInitializeSRWLock / RtlRunOnce* /
    #   RtlLcidToLocaleName from the main `rtl` library (rtl-rust → rtl-sync
    #   `cabi`), which precedes rtl_vista on its link line. rtl_vista therefore
    #   compiles no C synchronization code — just the placeholder TU below — so
    #   the COLD ntdll_vista link the C files used to back is now 100% Rust.
list(APPEND SOURCE_VISTA
    rtl_vista_stub.c
    )

add_library(rtl_vista ${SOURCE_VISTA})
add_pch(rtl_vista rtl_vista.h SOURCE_VISTA)
add_dependencies(rtl_vista psdk)
target_link_libraries(rtl_vista PRIVATE pseh)

# Rust UTF-8 library (all architectures) — links into rtl_vista
set(RUST_UTF8_LIB "${CMAKE_CURRENT_BINARY_DIR}/libutf8.a")
add_custom_command(
    OUTPUT ${RUST_UTF8_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${RUST_TARGET_DIR} ${RUST_EXTRA_ENV}
        cargo build --release --target ${RUST_TARGET}
            -p utf8
            --manifest-path ${RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${RUST_TARGET_DIR}/${RUST_TARGET}/release/libutf8.a
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
