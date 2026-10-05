
###############################################################################
# Global compile options
###############################################################################

# This requires NT6
remove_definitions(-D_WIN32_WINNT=0x502 -DWINVER=0x502)
add_definitions(-D_WIN32_WINNT=0x601 -DWINVER=0x601)

# Replace the global CRT include directory with the msvcrt include directory
get_property(INCLUDE_DIRS DIRECTORY . PROPERTY INCLUDE_DIRECTORIES)
list(REMOVE_ITEM INCLUDE_DIRS "${REACTOS_SOURCE_DIR}/sdk/include/crt")
set_property(DIRECTORY . PROPERTY INCLUDE_DIRECTORIES ${INCLUDE_DIRS})
include_directories(include)

# includes for wine code
include_directories(BEFORE ${REACTOS_SOURCE_DIR}/sdk/include/wine)

add_definitions(
    -D_CRTIMP=

    # Workarounds
    -DARRAY_SIZE=_countof
    -DI64_MIN=_I64_MIN
    -DI64_MAX=_I64_MAX
    -DUI64_MAX=_UI64_MAX
    -Dx87_asin=asin
    -Dmath_error=_invoke_matherr
    -Dasm_sqrt=sqrt
    -Dasm_sqrtf=sqrtf
)

if(ARCH STREQUAL "i386")
    add_definitions(
        -D__ASM_USE_THISCALL_WRAPPER
        -DRtlUnwind=__wine_RtlUnwind # See i386/__wine_RtlUnwind.s
    )
endif()

###############################################################################
# A static library shared with ucrtbase
###############################################################################

list(APPEND SHARED_SOURCE
    cpp.c
    except.c
    handler4.c
    undname.c
)

if(ARCH STREQUAL "i386")
    list(APPEND SHARED_SOURCE
        except_i386.c
    )
    # i386/__wine_RtlUnwind.s, i386/exception_ptr.s, i386/inout.s -- replaced by Rust
    # (crt-shared-rust src/asm_i386.rs, global_asm!).
elseif(ARCH STREQUAL "amd64")
    list(APPEND SHARED_SOURCE
        except_x86_64.c
    )
elseif(ARCH STREQUAL "arm64")
    list(APPEND SHARED_SOURCE
        except_arm64.c
    )
endif()

add_asm_files(msvcrt_shared_asm ${SHARED_ASM_SOURCE})

add_library(msvcrt_shared
    ${SHARED_SOURCE}
    ${msvcrt_shared_asm}
    reactos/ucrt_tls_sup.c
)

target_compile_definitions(msvcrt_shared PRIVATE _MSVCR_VER=0 __UCRTSUPPORT__)
add_dependencies(msvcrt_shared psdk asm)

###############################################################################
# The Wine C of msvcrt.dll, as an unlinked REFERENCE library
#
# No DLL links it: msvcrt.dll, msvcrt20, msvcrt40 and crtdll are msvcrt-crt-rust alone (see
# msvcrt_rust_cohab.cmake). It is still built because tools read its objects as an oracle:
# gen_ctype_tables.py checks the Rust ctype tables against ctype.c.obj's .data,
# gen_msvcrt_ref_protos.py reads prototypes, verify_cohab_coverage.py the C surface.
###############################################################################

list(APPEND MSVCRT_SOURCE
    ${SHARED_SOURCE}
    ${msvcrt_shared_asm}
    reactos/misc.c
    reactos/system.cpp
    # GUARDIAN: no code, only _Static_asserts. The Rust CRT bodies we cohab into this
    # DLL (msvcrt-crt-rust) reach into `struct __thread_data` and `struct
    # threadlocaleinfostruct` via a bindgen SNAPSHOT of their layout. This TU is
    # compiled with msvcrt's real flags on all 3 arches, and fails the build the moment
    # the C moves out from under that snapshot. @generated together with the Rust-side
    # twin by sdk/lib/rust/msvcrt-sys/generate_bindings.py — regenerate both, never
    # hand-edit one.
    rust_layout_assert.c
    concurrency.c
    console.c
    ctype.c
    data.c
    dir.c
    environ.c
    errno.c
    exit.c
    file.c
    heap.c
    locale.c
    lock.c
    main.c
    math.c
    mbcs.c
    misc.c
    process.c
    scanf.c
    sincos.c
    string.c
    thread.c
    time.c
    wcs.c
)

add_asm_files(msvcrt_asm ${MSVCRT_ASM_SOURCE})
list(APPEND MSVCRT_SOURCE ${msvcrt_asm})

add_library(msvcrt_static
    ${MSVCRT_SOURCE}
)

target_compile_definitions(msvcrt_static PRIVATE _MSVCR_VER=0)
add_dependencies(msvcrt_static psdk asm)

###############################################################################
# MSVCRT.DLL
###############################################################################

spec2def(msvcrt.dll msvcrt.spec ADD_IMPORTLIB)

# The `...` entry-point shells over the Rust engines were the C OBJECT library
# msvcrt_rust_shims (rust_varargs_shim.c, rust_scanf_shim.c, rust_file_varargs_shim.c,
# rust_process_varargs_shim.c, rust_console_varargs_shim.c), C only because stable Rust
# cannot write va_start. They are msvcrt-crt-rust src/va_entries.rs now: varargs-rust's
# va_entry! thunks in the archive every CRT DLL links.

add_library(msvcrt MODULE
    ${CMAKE_CURRENT_BINARY_DIR}/msvcrt_stubs.c
    ${CMAKE_CURRENT_BINARY_DIR}/msvcrt.def
    msvcrt.rc
)

target_compile_definitions(msvcrt PRIVATE _MSVCR_VER=0)
set_module_type(msvcrt win32dll ENTRYPOINT DllMain 12)

# ── msvcrt-crt-rust → msvcrt.dll ─────────────────────────────────────
# The whole CRT body, user-mode codegen (i386 is SSE2 since 2026-10-03 -- see the
# RUSTFLAGS below). It began as a cohab ahead of msvcrt_static and replaced it one
# family at a time; no C is linked now (msvcrt_rust_cohab.cmake). See
# docs/engineering/active-lanes/msvcrt-from-pieces-plan.md.
# NOTE: `-C no-prepopulate-passes` (the captures(none) class-guard, field manual
# §1253) is deliberately NOT set here. It disables rustc's optimization pipeline
# outright — with it, msvcrt's memset/memcpy/memcmp compile to a FUNCTION CALL PER
# BYTE. That is msvcrt.dll's CRT, served to every user-mode caller in the system:
# it blew the net suite's 120s marker timeouts (71s -> 208s) and got wave 2
# reverted off master. The class it guards is toolchain-fixed on LLVM >= 21 (we
# ship 21.1.3), the crate is `#![no_builtins]` (so the fill/copy loops are not
# rewritten into self-calls), and memset now stores through a RAW POINTER rather
# than a slice — which is the signature the captures(none) DCE actually came in on.
# Throughput is pinned by `cargo test -p msvcrt-difftest --test perf` (>2x slower
# than the C it replaces = FAIL); ud2/sacred-primitive scans + the x3 smoke gate
# cover the miscompile class. Re-check with tools/scan_captures.py after any
# rustc/LLVM bump.
set(MSVCRT_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
set(MSVCRT_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/msvcrt-crt-rust-target")
if(ARCH STREQUAL "i386")
    # SSE2, as ucrtbase (2026-10-03): no kernel-side consumer links this archive, every
    # MSVC-built x86 program has required SSE2 since VS2012, and it is the only way the
    # `libm` crate builds here (under -sse rustc rejects its Rust-ABI f64 functions). The
    # cost is pre-SSE2 CPUs (Pentium III / Athlon XP and older), which nothing we target
    # has. The FP-control quartet (_control87/_statusfp/_clearfp) is already Wine's
    # both-unit version (msvcrt-crt-rust/src/msvcrt/math.rs), so fesetround reaches MXCSR.
    # Spelled out, not left empty: sdk/lib/rust/.cargo/config.toml gives this triple
    # -sse,-sse2, and an unset RUSTFLAGS would let that apply. msvcrt20/msvcrt40/crtdll
    # link this same archive.
    set(MSVCRT_RUST_TARGET "i686-pc-windows-gnu")
    set(MSVCRT_RUST_ENV "RUSTFLAGS=-C target-feature=+sse,+sse2")
elseif(ARCH STREQUAL "amd64")
    set(MSVCRT_RUST_TARGET "x86_64-pc-windows-gnu")
    set(MSVCRT_RUST_ENV "")
elseif(ARCH STREQUAL "arm64")
    set(MSVCRT_RUST_TARGET "aarch64-pc-windows-gnullvm")
    set(MSVCRT_RUST_ENV "")
endif()

include(msvcrt_rust_cohab.cmake)
set(MSVCRT_CRT_RUST_LIB "${MSVCRT_RUST_COHAB_LIB}")
# Every source compiled into libmsvcrt_crt_cargo.a: msvcrt-crt-cargo's path-dependency
# closure, walked from the manifests (sources, Cargo.toml and build.rs of each crate). A
# crate missing here compiles into the archive but never triggers its rebuild -- a stale
# archive, no error. This was a hand-kept crate list, and it drifted three times (most
# recently it lacked varargs-rust, so a varargs-rust edit left msvcrt.dll stale while
# ucrtbase rebuilt); verify_ucrtbase_cohab.py fails if the derivation is replaced again.
rust_crate_sources(MSVCRT_CRT_RUST_SOURCES "${MSVCRT_RUST_WORKSPACE_DIR}" msvcrt-crt-cargo)
add_custom_command(
    OUTPUT ${MSVCRT_CRT_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${MSVCRT_RUST_TARGET_DIR} ${MSVCRT_RUST_ENV}
        cargo build --release --target ${MSVCRT_RUST_TARGET}
            -p msvcrt-crt-cargo
            --manifest-path ${MSVCRT_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${MSVCRT_RUST_TARGET_DIR}/${MSVCRT_RUST_TARGET}/release/libmsvcrt_crt_cargo.a
        ${MSVCRT_CRT_RUST_LIB}
    DEPENDS
            ${MSVCRT_CRT_RUST_SOURCES}
            ${MSVCRT_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMENT "Building Rust msvcrt CRT-body cohab (${MSVCRT_RUST_TARGET})"
)
add_custom_target(rust_msvcrt_crt DEPENDS ${MSVCRT_CRT_RUST_LIB})

link_msvcrt_rust_cohab(msvcrt)
# Build-order only, nothing is linked: msvcrt_static is no longer on any link line, but the
# host difftests (msvcrt-difftest, msvcrt-scanf-difftest) and gen_ctype_tables.py /
# gen_msvcrt_ref_protos.py / verify_cohab_coverage.py read its OBJECTS as their oracle, and
# a build only makes what `livecd` depends on. Without this a fresh tree never compiled it
# and the gate's difftest failed DIFFTEST_STALE_ORACLE (master red, 2026-10-04), while every
# existing build dir still held the objects from before the unlink.
add_dependencies(msvcrt msvcrt_static)
# ${PSEH_USER_LIB} and ${VCRT_SETJMP_LIB} by path (gcc.cmake): the SEH runtime and
# setjmp/longjmp resolve from Rust before ntdll's import library, which exports
# _setjmp/longjmp and would otherwise answer them. Nothing else: msvcrt_static, crtmath,
# vcruntime, wine, chkstk and chkesp contributed no object to any arch's link (link maps,
# 2026-10-04), and dbgrpt is msvcrt-crt-rust's dbgrpt.rs.
target_link_libraries(msvcrt ${PSEH_USER_LIB} ${VCRT_SETJMP_LIB})
add_importlibs(msvcrt kernel32 kernel32_vista ntdll)
add_delay_importlibs(msvcrt user32 advapi32)

add_cd_file(TARGET msvcrt DESTINATION reactos/system32 FOR all)

###############################################################################
# Import library
###############################################################################

# Let consumers of msvcrt have the right defines
target_compile_definitions(libmsvcrt INTERFACE _DLL __USE_CRTIMP)

# Link libmsvcrt to msvcrtex
target_link_libraries(libmsvcrt INTERFACE msvcrtex)

# Link GCC stack protector lib
if (STACK_PROTECTOR)
    target_link_libraries(libmsvcrt INTERFACE gcc_ssp_msvcrt)
    target_link_libraries(libmsvcrt INTERFACE libkernel32) # For OutputDebugStringA
endif()
