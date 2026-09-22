
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

# NOTE: the cmake custom_commands that used to build libntdll_rust.a (ntdll-rust,
# default features) and libcsrlib_rust.a (smss-csrss-rust csrlib_only) for the
# old cmake ntdll link were REMOVED when ntdll.dll graduated to the cargo-driven
# link (add_cargo_module below) — ntdll-cargo/build.rs now compiles those crates
# itself via reactos-link. rtl_um (above) + etwtrace (below) stay: they're stub
# OBJECT/static libs still referenced OUTSIDE ntdll (rtl_unittest links rtl_um;
# advapi32 links etwtrace at DLL_EXPORT_VERSION>=0x600).

# etwtrace: stub kept for advapi32 (links it at DLL_EXPORT_VERSION>=0x600). The
# real Etw* exports live in ntdll (etw/trace.c → ntdll-rust/src/etw/trace.rs).
# etw/trace.c stays on disk. See etw/etwtrace_rust_stub.c.
add_library(etwtrace etw/etwtrace_rust_stub.c)
target_link_libraries(etwtrace ${PSEH_LIB})
add_dependencies(etwtrace psdk)

list(APPEND SOURCE
    # dbg/dbgui.c — replaced by Rust (ntdll-rust/src/dbg/dbgui.rs);
    #   DbgUi* now provided by libntdll_rust.a (below)
    # ldr/ldrapi.c — replaced by Rust (ntdll-rust/src/ldr/ldrapi.rs);
    #   LdrLoadDll/LdrUnloadDll/LdrGetProcedureAddress/LdrGetDllHandleEx +
    #   loader public API (LdrLock/UnlockLoaderLock, LdrFindEntryForAddress,
    #   LdrQueryProcessModuleInformation, LdrEnumerateLoadedModules, …) now
    #   provided by libntdll_rust.a (below)
    # ldr/ldrinit.c — replaced by Rust (ntdll-rust/src/ldr/ldrinit.rs);
    #   LdrpInitialize/LdrpInitializeProcess/LdrpInitializeThread + loader-init
    #   core (TLS init, security cookie, exec-options, process-compat) now
    #   provided by libntdll_rust.a (below)
    # ldr/ldrpe.c — replaced by Rust (ntdll-rust/src/ldr/ldrpe.rs);
    #   LdrpSnapIAT/LdrpSnapThunk/LdrpWalkImportDescriptor/LdrpLoadImportModule
    #   + import-descriptor handlers now provided by libntdll_rust.a (below)
    # ldr/ldrutils.c — replaced by Rust (ntdll-rust/src/ldr/ldrutils.rs);
    #   LdrpMapDll/LdrpCheckForLoadedDll/LdrpGetProcedureAddress/LdrpSearchPath
    #   + loader-utility helpers now provided by libntdll_rust.a (below)
    # ldr/verifier.c — replaced by Rust (ntdll-rust/src/ldr/verifier.rs);
    #   AVrf*/LdrpInitializeApplicationVerifierPackage now provided by
    #   libntdll_rust.a (below)
    )

if(ARCH STREQUAL "i386")
    list(APPEND ASM_SOURCE dispatch/i386/dispatch.S)
elseif(ARCH STREQUAL "amd64")
    list(APPEND ASM_SOURCE
        dispatch/amd64/dispatch.S
        compat/amd64/msys2.S)
elseif(ARCH STREQUAL "arm64")
    list(APPEND ASM_SOURCE dispatch/arm64/dispatch.S)
    # dispatch/dispatch.c — replaced by Rust (ntdll-rust/src/dispatch/mod.rs,
    #   #[cfg(aarch64)]); the arm64 .S still owns KiUserExceptionDispatcher /
    #   KiUserApcDispatcher / LdrInitializeThunk, so the Rust module provides only
    #   KiRaiseUserExceptionDispatcher + KiUserCallbackDispatcher (the two the C
    #   contributed on arm64). Now provided by libntdll_rust.a (below).
else()
    list(APPEND SOURCE dispatch/dispatch.c)
endif()

add_asm_files(ntdll_asm ${ASM_SOURCE})

list(APPEND PCH_SKIP_SOURCE
    ${CMAKE_CURRENT_BINARY_DIR}/ntdll_stubs.c)

# --- ntdll.dll: link driven by cargo (graduated from cmake add_library) ------
# ntdll-cargo/build.rs (reactos-link) produces the shipping ntdll.dll:
#   rust_staticlib(ntdll-rust) + csrlib_only smss-csrss-rust + rtl-rust + utf8
#   + cnt-rust-lib (Tier-B, cargo-compiled) + the pseh dummy shim + dispatch &
#   ntdllsys asm (pre-expanded) + spec2def/-stubs + the ntdll .res,
# reusing only the Tier-C deep-C librtl/librtl_vista/libcntpr.a
# (rtl/rtl_vista/libcntpr — deferred to the rtl-priv + libcntpr-crt flips).
# Replaces add_library(ntdll MODULE …) + set_module_type(win32dll ENTRYPOINT 0)
# + set_subsystem(console) + the target_link_libraries closure + add_pch +
# add_cd_file. KIND win32dll → i386 lld forwarder fix only (ntdll is user-mode;
# no native-pefixup). image base / entry (___ImageBase) / console:5.01 subsystem
# are baked into the build.rs. spec2def(ntdll.dll … ADD_IMPORTLIB) above STAYS —
# the whole tree imports libntdll. (The old cmake Rust custom_commands that built
# libntdll_rust.a / libcsrlib_rust.a for this link were removed — the cargo build
# compiles those crates itself; rtl_um + etwtrace stubs are kept above because
# rtl_unittest / advapi32 still link them.)
add_cargo_module(ntdll ntdll-cargo ntdll.dll
    KIND win32dll
    BODY_CRATES ntdll-rust smss-csrss-rust
    # asm = generates the asm includes (ks*.inc / syscalls.inc / sysfuncs.h) the
    # engine needs to assemble dispatch.S + ntoskrnl/ntdll.S from source.
    EXTRA_DEPENDS rtl rtl_vista libcntpr ntstatus asm)

# Newer VS versions link to _ftol2 / _ftoul2_legacy
if(MSVC AND ARCH STREQUAL "i386")
    target_link_libraries(libntdll ftol2)
endif()
