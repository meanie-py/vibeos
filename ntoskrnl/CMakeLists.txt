
PROJECT(NTOS)

if (CMAKE_C_COMPILER_ID STREQUAL "GNU" OR CMAKE_C_COMPILER_ID STREQUAL "Clang")
    # Enable this again. CORE-17637
    add_compile_options(-Wunused-result)
endif()

# cswitchdep taps 4/5 (docs/engineering/cswitchdep-design.md): light the
# #ifdef VIBEOS_CSWITCHDEP tap sites in cmake-compiled C/asm. NOTE the LIVE
# syscall paths are NOT here: the trap handlers are Rust (traphandler.rs /
# traphdlr.rs, feature-gated) and ke/amd64/trap.S is cargo-assembled by
# ntoskrnl-cargo/build.rs (which plumbs the same env to the assembler). This
# define only keeps the dead-mirror C copies (traphandler.c/traphdlr.c,
# commented out of ntos.cmake) compiling faithfully if ever re-enabled.
if(VIBEOS_CSWITCHDEP)
    add_definitions(-DVIBEOS_CSWITCHDEP)
endif()

include(ntos.cmake)

set(NTOSKRNL_SOURCE ${SOURCE})
set(NTOSKRNL_ASM_SOURCE ${ASM_SOURCE})

set(NTKRNLMP_SOURCE ${SOURCE})
set(NTKRNLMP_ASM_SOURCE ${ASM_SOURCE})

spec2def(ntoskrnl.exe ntoskrnl.spec ADD_IMPORTLIB)

# Link GCC stack protector lib
if (STACK_PROTECTOR)
    target_link_libraries(libntoskrnl INTERFACE gcc_ssp_nt)
endif()

add_asm_files(ntoskrnl_asm ${NTOSKRNL_ASM_SOURCE})

if(CMAKE_C_COMPILER_ID STREQUAL "Clang")
    # Clang optimises strcmp calls to memcmp.
    target_link_libraries(libntoskrnl INTERFACE memcmp)
endif()

if((CMAKE_C_COMPILER_ID STREQUAL "Clang") OR (CMAKE_C_COMPILER_ID STREQUAL "GNU"))
    # Enable xsave feature (x86/AMD64 only)
    if(ARCH STREQUAL "i386" OR ARCH STREQUAL "amd64")
        add_compile_options(-mxsave)
    endif()
endif()

# UART traces for ARM64 boot debugging (disabled — too verbose for normal operation)
# if(ARCH STREQUAL "arm64")
#     add_definitions(-DARM64_UART_TRACE_ENABLE)
# endif()

# guid.c — replaced by Rust (ntoskrnl-rust/src/guid.rs)
#list(APPEND PCH_SKIP_SOURCE
#    guid.c)

list(APPEND PCH_SKIP_SOURCE
    ${REACTOS_SOURCE_DIR}/ntoskrnl/ke/ke_rust_shims.c
    ${REACTOS_SOURCE_DIR}/ntoskrnl/variadic_shims.c
    ${REACTOS_SOURCE_DIR}/ntoskrnl/lto_workaround.c
    ${REACTOS_SOURCE_DIR}/ntoskrnl/mm_lock_outlines.c
    ${REACTOS_SOURCE_DIR}/ntoskrnl/mm/mm_rust_shims.c
    ${REACTOS_SOURCE_DIR}/ntoskrnl/io/iomgr/io_rust_shims.c)
# kdbg_rust_shims.c is only built on i386/amd64 (KDBG is disabled on ARM64).
# Add to PCH_SKIP_SOURCE conditionally to mirror the SOURCE-list gating in
# ntos.cmake.
if(KDBG AND NOT ARCH STREQUAL "arm64")
    list(APPEND PCH_SKIP_SOURCE
        ${REACTOS_SOURCE_DIR}/ntoskrnl/kdbg/kdbg_rust_shims.c)
endif()

# Rust kernel code — single umbrella staticlib (pool + ob/obhandle + future modules)
set(RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
set(RUST_KERNEL_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-kernel-target")
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

set(RUST_KERNEL_LIB "${CMAKE_CURRENT_BINARY_DIR}/libkernel_rust.a")

# kernel-rust is an umbrella staticlib that pulls in EVERY sibling rlib via
# `extern crate`, so its true input is the whole rust workspace — not the
# hand-maintained 200-line file list this used to carry. That list was
# necessarily incomplete (ntoskrnl-rust alone is ~405 .rs): editing any .rs not
# listed left ninja thinking the staticlib was current, so a stale kernel
# silently masked the source fix — field-manual §9.6b. Glob the whole workspace
# (same mechanism as add_cargo_module in gcc.cmake), excluding cargo target/
# dirs (build artifacts, not source). CONFIGURE_DEPENDS re-globs when files are
# added/removed.
file(GLOB_RECURSE _rust_kernel_srcs CONFIGURE_DEPENDS
    ${RUST_WORKSPACE_DIR}/*.rs ${RUST_WORKSPACE_DIR}/*.toml)
list(FILTER _rust_kernel_srcs EXCLUDE REGEX "/target/")

add_custom_command(
    OUTPUT ${RUST_KERNEL_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${RUST_KERNEL_TARGET_DIR} ${RUST_EXTRA_ENV}
        cargo build --release --target ${RUST_TARGET}
            -p kernel-rust
            --manifest-path ${RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${RUST_KERNEL_TARGET_DIR}/${RUST_TARGET}/release/libkernel_rust.a
        ${RUST_KERNEL_LIB}
    DEPENDS ${_rust_kernel_srcs}
    COMMENT "Building Rust kernel code (${RUST_TARGET})"
)
add_custom_target(rust_kernel DEPENDS ${RUST_KERNEL_LIB})

# ntoskrnl.exe: link driven by cargo (graduated from cmake add_executable).
# ntoskrnl-cargo/build.rs (reactos-link) self-hosts the kernel-rust staticlib +
# the cmake C-floor static libs (ntoskrnl_vista/cportlib/csq/pseh/arbiter/cmlib/
# lsalib/rtl/rossym/setjmp/wdmguid/poguid/ioevent/hal/kdcom/bootvid, + cnt-rust
# replacing libcntpr) + self-assembled asm + spec2def + the .res, replacing
# add_executable + set_module_type(kernel) + target_link_libraries + add_pch.
# KIND kernelexe → native-pefixup --kernel + i386 forwarder fix. Proven ×3
# boot-equivalent via ntoskrnl-cargo/boot-swap-smoke.sh. The spec2def(...
# ADD_IMPORTLIB) at the top of this file STAYS — the tree imports libntoskrnl.
# EXTRA_DEPENDS == the OLD ntoskrnl dependency closure (target_link_libraries +
# add_importlibs + add_dependencies), so a cold build can't race a stale .a /
# generated header the cargo link reuses by path. (rust_kernel kept as a dep:
# the cargo build links its OWN kernel-rust, but the cmake target's build is the
# proven-sufficient source of any generated prerequisites; revisit to drop it.)
add_cargo_module(ntoskrnl ntoskrnl-cargo ntoskrnl.exe
    KIND kernelexe
    BODY_CRATE kernel-rust
    EXTRA_DEPENDS
        ntoskrnl_vista cportlib csq ${PSEH_LIB} arbiter cmlib ntlsalib rtl
        ${ROSSYM_LIB} libcntpr setjmp wdmguid poguid ioevent
        hal kdcom bootvid
        # The cargo link (reactos-link) consumes kdcom/bootvid's *import libs*
        # (CMakeFiles/lib{kdcom,bootvid}.dir/lib*.a) by path. The DLL targets
        # above do NOT build their own import libs (only consumers do), and
        # nothing else in the kernel's closure pulls them — so a from-scratch
        # `ninja -t clean` build leaves them missing and the cargo link panics
        # ("missing static lib … run ninja first"). Depend on the implib custom
        # targets explicitly so a cold build always produces them first.
        libkdcom_implib_target libbootvid_implib_target
        psdk asm rust_kernel)

add_subdirectory(ntkrnlmp)

add_asm_files(ntdllsys_asm ntdll.S)
add_library(ntdllsys ${ntdllsys_asm})
set_target_properties(ntdllsys PROPERTIES LINKER_LANGUAGE "C")
add_dependencies(ntdllsys asm)
