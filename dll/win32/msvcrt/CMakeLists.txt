
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
    list(APPEND SHARED_ASM_SOURCE
        i386/__wine_RtlUnwind.s
        i386/exception_ptr.s
        i386/inout.s
    )
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
# A static library shared between msvcrt.dll msvcrt20.dll and msvcrt40.dll
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

# The `...` entry-point shells over the Rust engines, built ONCE with this directory's
# flags and linked by every CRT DLL that takes the Rust cohab (msvcrt, msvcrt20,
# msvcrt40, crtdll -- see msvcrt_rust_cohab.cmake). An OBJECT library, so its
# objects are module objects in each consumer and win first-match over
# msvcrt_static's archive members, exactly as when they were msvcrt's own sources.
add_library(msvcrt_rust_shims OBJECT
    # The printf `...` shells, as va_start trampolines over the Rust engine.
    # In the MODULE and not in msvcrt_static ON PURPOSE: module objects are
    # resolved before any archive member, so these win first-match over
    # wcs.o's copies. Inside msvcrt_static the winner would depend on member
    # order within one archive. See the file's own header for why the shells
    # cannot be Rust (no va_start on stable) and cannot stay in wcs.c
    # (clang inlines the C engine into its housemates).
    rust_varargs_shim.c
    # The scanf `...` shells, same story and same placement as the printf ones
    # above: 40 va_start trampolines over the Rust engine in scanf.rs, in the
    # MODULE so they win first-match over scanf.o's copies. There is a second
    # reason here — scanf.c's 16 engines are `static`, so a Rust engine cannot
    # replace them by first-match at all; the only way in is to define the
    # trampolines somewhere the linker prefers, which is exactly this target.
    rust_scanf_shim.c
    # file.c's 20 `...` entry points — the 16 fprintf/printf/fwprintf/wprintf
    # shells plus the four _open/_sopen ones, whose `...` carries an optional
    # pmode rather than a format. Same reason and same placement as the two
    # above: no va_start on stable, and left in file.c they would keep calling
    # file.c's own C bodies. Their delegates are all external (the `v*` twins
    # and `_sopen_dispatch`/`_wsopen_dispatch` are Rust in file.rs, and unlike
    # scanf.c's engines they are not `static`), so first-match reaches Rust.
    rust_file_varargs_shim.c
    # process.c's 16 `_exec*`/`_spawn*` WINAPIV `...` entry points. Same reason and same
    # placement as the three above: no va_start on stable, and left in process.c they
    # would keep calling process.c's own C bodies. Their `_v*`-prefixed twins are Rust in
    # process.rs and are not `static`, so first-match reaches Rust.
    rust_process_varargs_shim.c
    # console.c's 4 `_cprintf`/`_cwprintf` WINAPIV `...` entry points. Same reason and same
    # placement as the four above. Their va_list twins needed no invention — `_vcprintf`,
    # `_vcprintf_l`, `_vcwprintf` and `_vcwprintf_l` are console.c's own exports, and are
    # Rust in console.rs.
    rust_console_varargs_shim.c
)
target_compile_definitions(msvcrt_rust_shims PRIVATE _MSVCR_VER=0)
add_dependencies(msvcrt_rust_shims psdk)

add_library(msvcrt MODULE
    ${CMAKE_CURRENT_BINARY_DIR}/msvcrt_stubs.c
    ${CMAKE_CURRENT_BINARY_DIR}/msvcrt.def
    msvcrt.rc
)

target_compile_definitions(msvcrt PRIVATE _MSVCR_VER=0)
set_module_type(msvcrt win32dll ENTRYPOINT DllMain 12)

# ── Rust CRT-body cohab: msvcrt-crt-rust → msvcrt.dll ────────────────
# Scoped user-mode cohab: msvcrt-crt-rust re-exports a growing set of pure
# CRT-body symbols over cnt-rust's difftest-pinned `core_rs`. Linked BEFORE
# msvcrt_static so lld first-match selects the Rust symbols; the C monolith
# copies (string.c etc.) go dead (--allow-multiple-definition, MODULE-link
# default). Mirrors libcntpr.cmake's cnt-rust swap but with USER-MODE codegen
# (i386 keeps the ReactOS -sse,-sse2 baseline like the C build; wave 1 is
# integer-only so SSE is irrelevant). See
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
    set(MSVCRT_RUST_TARGET "i686-pc-windows-gnu")
    set(MSVCRT_RUST_ENV "RUSTFLAGS=-C target-feature=-sse,-sse2")
elseif(ARCH STREQUAL "amd64")
    set(MSVCRT_RUST_TARGET "x86_64-pc-windows-gnu")
    set(MSVCRT_RUST_ENV "")
elseif(ARCH STREQUAL "arm64")
    set(MSVCRT_RUST_TARGET "aarch64-pc-windows-gnullvm")
    set(MSVCRT_RUST_ENV "")
endif()

include(msvcrt_rust_cohab.cmake)
set(MSVCRT_CRT_RUST_LIB "${MSVCRT_RUST_COHAB_LIB}")
# Every crate whose code lands in libmsvcrt_crt_cargo.a -- the path-dependency
# closure of msvcrt-crt-cargo. A crate missing here compiles into the archive but
# never triggers its rebuild (a stale archive, no error); crt-portable-rust was
# missing the day the string family moved into it. verify_ucrtbase_cohab.py
# derives the closure from the manifests and fails if this list and it disagree.
set(MSVCRT_CRT_RUST_CRATES
    msvcrt-crt-cargo cnt-rust crt-portable-rust crt-shared-rust msvcrt-crt-rust
    msvcrt-sys panic-common)
set(MSVCRT_CRT_RUST_GLOBS)
set(MSVCRT_CRT_RUST_MANIFESTS)
foreach(crate ${MSVCRT_CRT_RUST_CRATES})
    list(APPEND MSVCRT_CRT_RUST_GLOBS ${MSVCRT_RUST_WORKSPACE_DIR}/${crate}/src/*.rs)
    list(APPEND MSVCRT_CRT_RUST_MANIFESTS ${MSVCRT_RUST_WORKSPACE_DIR}/${crate}/Cargo.toml)
endforeach()
file(GLOB_RECURSE MSVCRT_CRT_RUST_SOURCES CONFIGURE_DEPENDS ${MSVCRT_CRT_RUST_GLOBS})
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
            ${MSVCRT_CRT_RUST_MANIFESTS}
            ${MSVCRT_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMENT "Building Rust msvcrt CRT-body cohab (${MSVCRT_RUST_TARGET})"
)
add_custom_target(rust_msvcrt_crt DEPENDS ${MSVCRT_CRT_RUST_LIB})

# msvcrt_crt_rust under --whole-archive so ALL its symbols are DEFINED before
# the C monolith (string.c.obj) is pulled. Plain link-order is NOT enough: the
# C monolith still gets pulled for its ~100 un-flipped symbols (strlen, …) and
# drags in its own copy of the flipped functions, which would otherwise win
# first-match. --whole-archive forces the Rust defs in first; the C copies then
# become the ignored duplicates (--allow-multiple-definition). See lane plan §1b.
link_msvcrt_rust_cohab(msvcrt)
target_link_libraries(msvcrt msvcrt_static crtmath dbgrpt vcruntime wine chkstk ${PSEH_LIB})
if(ARCH STREQUAL "i386")
    target_link_libraries(msvcrt chkesp)
endif()
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
