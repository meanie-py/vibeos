
include(../ntos.cmake)

include_directories(${REACTOS_SOURCE_DIR}/ntoskrnl)
spec2def(ntkrnlmp.exe ../ntoskrnl.spec)

add_asm_files(ntkrnlmp_asm ${NTKRNLMP_ASM_SOURCE})

# mproc.c is now translated to Rust (ke/i386/mproc.rs, ke/amd64/mproc.rs)
# and linked via kernel-rust. The C originals are no longer compiled.
#if(ARCH STREQUAL "i386")
#    list(APPEND NTKRNLMP_SOURCE
#        ${REACTOS_SOURCE_DIR}/ntoskrnl/ke/i386/mproc.c)
#elseif(ARCH STREQUAL "amd64")
#    list(APPEND NTKRNLMP_SOURCE
#        ${REACTOS_SOURCE_DIR}/ntoskrnl/ke/amd64/mproc.c)
#elseif(FALSE)
#    #TBD
#endif()

list(APPEND NTKRNLMP_PCH_SKIP_SOURCE
    ${REACTOS_SOURCE_DIR}/ntoskrnl/guid.c
    ${REACTOS_SOURCE_DIR}/ntoskrnl/ke/ke_rust_shims.c
    ${REACTOS_SOURCE_DIR}/ntoskrnl/variadic_shims.c
    ${REACTOS_SOURCE_DIR}/ntoskrnl/lto_workaround.c
    ${REACTOS_SOURCE_DIR}/ntoskrnl/mm_lock_outlines.c
    ${REACTOS_SOURCE_DIR}/ntoskrnl/mm/mm_rust_shims.c
    ${REACTOS_SOURCE_DIR}/ntoskrnl/io/iomgr/io_rust_shims.c)
# kdbg_rust_shims.c is only built on i386/amd64 (KDBG is disabled on ARM64).
# Mirror the conditional gating in ntoskrnl/CMakeLists.txt.
if(KDBG AND NOT ARCH STREQUAL "arm64")
    list(APPEND NTKRNLMP_PCH_SKIP_SOURCE
        ${REACTOS_SOURCE_DIR}/ntoskrnl/kdbg/kdbg_rust_shims.c)
endif()

# ntkrnlmp.exe (SMP kernel): cargo-driven link via ntkrnlmp-cargo — identical to
# the ntoskrnl.exe cargo cutover but the kernel .S is assembled with CONFIG_SMP
# (ntkrnlmp-cargo/build.rs .asm_extra_defines). The Rust body (kernel-rust) is
# the SAME UP/MP. Replaces add_executable + set_module_type(kernel) +
# target_compile_definitions(CONFIG_SMP) + target_link_libraries + add_pch.
# KIND kernelexe → native-pefixup --kernel + i386 fwd fix. spec2def(ntkrnlmp.exe
# ../ntoskrnl.spec) above STAYS. EXTRA_DEPENDS == the old ntkrnlmp dependency
# closure (the same C floor + import libs ntoskrnl uses) so a cold build can't
# race a stale .a the cargo link reuses by path.
add_cargo_module(ntkrnlmp ntkrnlmp-cargo ntkrnlmp.exe
    KIND kernelexe
    BODY_CRATE kernel-rust
    EXTRA_DEPENDS
        ntoskrnl_vista cportlib csq ${PSEH_LIB} arbiter cmlib ntlsalib rtl
        ${ROSSYM_LIB} libcntpr setjmp wdmguid poguid ioevent
        hal kdcom bootvid
        # See ntoskrnl/CMakeLists.txt: the cargo link consumes kdcom/bootvid's
        # import libs by path, which nothing else in the closure builds — wire
        # the implib custom targets so a cold `ninja -t clean` build can't race
        # a missing lib*.a ("missing static lib … run ninja first").
        libkdcom_implib_target libbootvid_implib_target
        psdk asm)
