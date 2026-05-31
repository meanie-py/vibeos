
add_subdirectory(nt_0600)

spec2def(ntdll.dll def/ntdll.spec ADD_IMPORTLIB)

# Link GCC stack protector lib
if (STACK_PROTECTOR)
    target_link_libraries(libntdll INTERFACE gcc_ssp_nt)
endif()

add_definitions(
    -D__NTDLL__
    -D_NTOSKRNL_
    -DCRTDLL)

include_directories(
    BEFORE include
    ${REACTOS_SOURCE_DIR}/sdk/include/reactos/subsys)

add_library(rtl_um OBJECT
    # rtl/libsupp.c — replaced by Rust (ntdll-rust/src/rtl/libsupp.rs);
    #   the rtl↔ntdll glue (RtlpAllocateMemory/RtlpGetStackLimits/
    #   RtlAcquirePebLock/RtlWalkFrameChain/RtlImageNtHeaderEx/atom-table
    #   locks/RtlDosApplyFileIsolationRedirection_Ustr/…) now provided by
    #   libntdll_rust.a (below)
    # rtl/uilist.c — replaced by Rust (ntdll-rust/src/rtl/uilist.rs);
    #   RtlConvertUiListToApiList now provided by libntdll_rust.a (below)
    # rtl/version.c — replaced by Rust (ntdll-rust/src/rtl/version.rs);
    #   RtlGetNtProductType / RtlGetNtVersionNumbers / RtlGetVersion now
    #   provided by libntdll_rust.a (below)
    rtl/rtl_um_rust_stub.c  # keeps the now-empty OBJECT lib valid
    )
target_link_libraries(rtl_um apisets ${PSEH_LIB})
add_dependencies(rtl_um psdk)

# --- Rust cohabitation (ntdll-rust, ntdll.dll proper) ---------------
# Builds ntdll-rust with DEFAULT features (NO `vista` — excludes nt_0600's
# DllMain/LdrpHeap, which belong to ntdll_vista.dll). Provides the rtl_um
# translations (rtl/uilist.rs, …). Its own rust-target dir, distinct from
# nt_0600's vista build → a separate libntdll_rust.a. Mirrors the csrss/
# csrsrv dual-archive + sdk/lib/rtl cargo→.a pattern. The .c stay on disk.
# See docs/engineering/ntdll-translation-audit.md (Phase 2).
set(NTDLL_RUST_WS "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
set(NTDLL_RUST_TGTDIR "${CMAKE_CURRENT_BINARY_DIR}/rust-target")
if(ARCH STREQUAL "i386")
    set(NTDLL_RUST_TGT "i686-pc-windows-gnu")
    set(NTDLL_RUST_ENV "RUSTFLAGS=-C target-feature=-sse,-sse2 -C no-prepopulate-passes")
elseif(ARCH STREQUAL "amd64")
    set(NTDLL_RUST_TGT "x86_64-pc-windows-gnu")
    set(NTDLL_RUST_ENV "")
elseif(ARCH STREQUAL "arm64")
    set(NTDLL_RUST_TGT "aarch64-pc-windows-gnullvm")
    set(NTDLL_RUST_ENV "")
endif()
set(NTDLL_RUST_MAIN_LIB "${CMAKE_CURRENT_BINARY_DIR}/libntdll_rust.a")
add_custom_command(
    OUTPUT ${NTDLL_RUST_MAIN_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${NTDLL_RUST_TGTDIR} ${NTDLL_RUST_ENV}
        cargo build --release --target ${NTDLL_RUST_TGT}
            -p ntdll-rust
            --manifest-path ${NTDLL_RUST_WS}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${NTDLL_RUST_TGTDIR}/${NTDLL_RUST_TGT}/release/libntdll_rust.a
        ${NTDLL_RUST_MAIN_LIB}
    DEPENDS
        ${NTDLL_RUST_WS}/ntdll-rust/src/lib.rs
        ${NTDLL_RUST_WS}/ntdll-rust/src/rtl/mod.rs
        ${NTDLL_RUST_WS}/ntdll-rust/src/rtl/uilist.rs
        ${NTDLL_RUST_WS}/ntdll-rust/src/rtl/version.rs
        ${NTDLL_RUST_WS}/ntdll-rust/src/rtl/libsupp.rs
        ${NTDLL_RUST_WS}/ntdll-rust/src/etw/mod.rs
        ${NTDLL_RUST_WS}/ntdll-rust/src/etw/trace.rs
        ${NTDLL_RUST_WS}/ntdll-rust/src/dbg/mod.rs
        ${NTDLL_RUST_WS}/ntdll-rust/src/dbg/dbgui.rs
        ${NTDLL_RUST_WS}/ntdll-rust/src/ldr/mod.rs
        ${NTDLL_RUST_WS}/ntdll-rust/src/ldr/verifier.rs
        ${NTDLL_RUST_WS}/ntdll-rust/src/nt_0600/mod.rs
        ${NTDLL_RUST_WS}/ntdll-rust/src/nt_0600/ldr/mod.rs
        ${NTDLL_RUST_WS}/ntdll-rust/src/nt_0600/ldr/ldrinit.rs
        ${NTDLL_RUST_WS}/ntdll-rust/Cargo.toml
        ${NTDLL_RUST_WS}/ntdll-sys/src/lib.rs
    COMMENT "Building ntdll-rust (${NTDLL_RUST_TGT}, default features) for ntdll.dll"
    VERBATIM)
add_custom_target(ntdll_rust_main_lib DEPENDS ${NTDLL_RUST_MAIN_LIB})
# --------------------------------------------------------------------

# On NT6+ this is used by advapi32 (only linked at DLL_EXPORT_VERSION>=0x600;
# at 0x502 — our whole build matrix — etwtrace is linked ONLY into ntdll).
# etw/trace.c replaced by Rust (ntdll-rust/src/etw/trace.rs); the Etw* exports
# now come from libntdll_rust.a (linked into ntdll below). The stub keeps the
# library target valid. The .c stays on disk. See etw/etwtrace_rust_stub.c.
add_library(etwtrace etw/etwtrace_rust_stub.c)
target_link_libraries(etwtrace ${PSEH_LIB})
add_dependencies(etwtrace psdk)

list(APPEND SOURCE
    # dbg/dbgui.c — replaced by Rust (ntdll-rust/src/dbg/dbgui.rs);
    #   DbgUi* now provided by libntdll_rust.a (below)
    ldr/ldrapi.c
    ldr/ldrinit.c
    ldr/ldrpe.c
    ldr/ldrutils.c
    # ldr/verifier.c — replaced by Rust (ntdll-rust/src/ldr/verifier.rs);
    #   AVrf*/LdrpInitializeApplicationVerifierPackage now provided by
    #   libntdll_rust.a (below)
    )

if(ARCH STREQUAL "i386")
    list(APPEND ASM_SOURCE dispatch/i386/dispatch.S)
elseif(ARCH STREQUAL "amd64")
    list(APPEND ASM_SOURCE dispatch/amd64/dispatch.S)
elseif(ARCH STREQUAL "arm64")
    list(APPEND ASM_SOURCE dispatch/arm64/dispatch.S)
    list(APPEND SOURCE dispatch/dispatch.c)
else()
    list(APPEND SOURCE dispatch/dispatch.c)
endif()

add_asm_files(ntdll_asm ${ASM_SOURCE})

list(APPEND PCH_SKIP_SOURCE
    ${CMAKE_CURRENT_BINARY_DIR}/ntdll_stubs.c)

add_library(ntdll MODULE
    ${SOURCE}
    ${ntdll_asm}
    ${PCH_SKIP_SOURCE}
    def/ntdll.rc
    ${CMAKE_CURRENT_BINARY_DIR}/ntdll.def)

set_module_type(ntdll win32dll ENTRYPOINT 0)
set_target_properties(ntdll PROPERTIES C_STANDARD 23)
#############################################
## HACK FOR MSVC COMPILATION WITH win32dll ##
set_subsystem(ntdll console)
################# END  HACK #################

target_link_libraries(ntdll ntdll_vista_static etwtrace csrlib rtl rtl_um rtl_vista ntdllsys libcntpr setjmp uuid ${PSEH_LIB} ${NTDLL_RUST_MAIN_LIB})
# Rust .a provides the rtl_um symbols whose .c are commented out
# (RtlConvertUiListToApiList, …); --allow-multiple-definition covers the
# panic_handler/eh_personality overlap with librtl's Rust and the cohab
# swap-in window (lld first-match). add_dependencies forces the cargo build.
add_dependencies(ntdll ntdll_rust_main_lib)
target_link_options(ntdll PRIVATE "-Wl,--allow-multiple-definition")
if(DLL_EXPORT_VERSION GREATER_EQUAL 0x600)
    target_link_libraries(ntdll cryptlib)
endif()

if (STACK_PROTECTOR)
    target_link_libraries(ntdll gcc_ssp_nt)
endif()

add_pch(ntdll include/ntdll.h "${PCH_SKIP_SOURCE}")
add_dependencies(ntdll ntstatus asm)
add_cd_file(TARGET ntdll DESTINATION reactos/system32 NO_CAB FOR all)

# Newer VS versions link to _ftol2 / _ftoul2_legacy
if(MSVC AND ARCH STREQUAL "i386")
    target_link_libraries(libntdll ftol2)
endif()
