
spec2def(ucrtbase.dll ucrtbase.spec NO_DBG ADD_IMPORTLIB)

add_compile_definitions(
    _UCRT
    _CORECRT_BUILD
    CRTDLL
)

add_library(ucrtbase SHARED
    # printf.c (sprintf/_vsnprintf) and ucrt_varargs_shim.c (the `...` entry points as
    # va_start trampolines, plus varargs-rust's va_arg pullers) are ucrt-rust's
    # va_entries.rs and varargs-rust's `rust-pullers`: va_entry! thunks in the Rust
    # archive linked whole below, so they still win first-match over any archive copy.
    # stubs.c (atexit, operator_new/delete, _chkesp_failed, __acrt_initialize_sse2) is
    # ucrt-rust's ucrt_reactos.rs and internal/init.rs.
    ucrtbase_stubs.c
    ucrtbase.def
)

target_compile_options(ucrtbase PRIVATE -Wno-builtin-declaration-mismatch)

set_entrypoint(ucrtbase __acrt_DllMain 12)


# ── Rust cohab: crt-shared-rust → ucrtbase.dll ───────────────────────────────
# ucrtbase.dll is VENDORED Microsoft UCRT (sdk/lib/ucrt, see vendor.yaml) plus OUR
# msvcrt_shared plus 132 lines of its own C. So the Rust archive linked here must cover
# msvcrt_shared's surface and NOTHING else, and that is the entire reason it is
# `crt-shared-cargo` rather than msvcrt.dll's `msvcrt-crt-cargo`:
#
#     ucrtbase.spec exports       2,584 names
#     msvcrt's cohab archive      1,229 names
#     collisions                    891 — of which msvcrt_shared's are 35
#                                        and GENUINE UCRT bodies are   856
#
# Linking msvcrt's archive here under --whole-archive would replace those 856 with bodies
# written and graded against MSVCRT's C at _MSVCR_VER=0 — our malloc, our _errno, our mbcs
# tables, our printf engine — and because the MODULE link uses
# --allow-multiple-definition it would raise NO diagnostic. It would build, boot, and
# behave subtly like msvcrt forever. tools/verify_ucrtbase_cohab.py asserts the archive
# stays a subset of msvcrt_shared's own objects rather than trusting this comment.
#
# The `ucrtsupport` cargo feature is what makes this the UCRT build rather than a second
# msvcrt one — it drops the 7 bodies ucrt owns (signal/raise/terminate/…) and enables the
# 15 that only exist at `_MSVCR_VER >= 80/140 || __UCRTSUPPORT__`, mirroring the
# `-D__UCRTSUPPORT__` the C half of msvcrt_shared is compiled with.
set(UCRT_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
set(UCRT_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/crt-shared-rust-target")
if(ARCH STREQUAL "i386")
    # SSE2, unlike every other i386 Rust module here: no kernel-side consumer links this
    # archive (the -sse,-sse2 baseline exists for the ones that do), and Microsoft's own
    # x86 ucrtbase.dll is built /arch:SSE2, as is every MSVC-built program that loads it.
    # It is also the only way to use the `libm` crate: under -sse rustc refuses a Rust-ABI
    # function that passes an f64, and libm is all Rust-ABI. Builtin triple plus -sse is
    # the combination rustc has announced it will stop accepting (roadmap, "55 i386 crates
    # build on an SSE/ABI combination rustc is phasing out"), so this is one fewer.
    # Spelled out, not left empty: sdk/lib/rust/.cargo/config.toml gives this triple
    # -sse,-sse2, and an unset RUSTFLAGS would let that apply.
    set(UCRT_RUST_TARGET "i686-pc-windows-gnu")
    set(UCRT_RUST_ENV "RUSTFLAGS=-C target-feature=+sse,+sse2")
elseif(ARCH STREQUAL "amd64")
    set(UCRT_RUST_TARGET "x86_64-pc-windows-gnu")
    set(UCRT_RUST_ENV "")
elseif(ARCH STREQUAL "arm64")
    set(UCRT_RUST_TARGET "aarch64-pc-windows-gnullvm")
    set(UCRT_RUST_ENV "")
endif()

set(UCRT_SHARED_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libcrt_shared_cargo.a")
# Every source compiled into libcrt_shared_cargo.a: crt-shared-cargo's path-dependency
# closure, walked from the manifests (sources, Cargo.toml and build.rs of each crate). A
# crate missing here compiles into the archive but never triggers its rebuild -- a stale
# archive, no error. This was a hand-kept crate list that drifted to two of seven crates;
# verify_ucrtbase_cohab.py fails if the derivation is replaced again.
rust_crate_sources(UCRT_SHARED_RUST_SOURCES "${UCRT_RUST_WORKSPACE_DIR}" crt-shared-cargo)
add_custom_command(
    OUTPUT ${UCRT_SHARED_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${UCRT_RUST_TARGET_DIR} ${UCRT_RUST_ENV}
        cargo build --release --target ${UCRT_RUST_TARGET}
            -p crt-shared-cargo
            --manifest-path ${UCRT_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${UCRT_RUST_TARGET_DIR}/${UCRT_RUST_TARGET}/release/libcrt_shared_cargo.a
        ${UCRT_SHARED_RUST_LIB}
    DEPENDS
            ${UCRT_SHARED_RUST_SOURCES}
            ${UCRT_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMENT "Building Rust ucrtbase msvcrt_shared cohab (${UCRT_RUST_TARGET})"
)
add_custom_target(rust_ucrt_shared DEPENDS ${UCRT_SHARED_RUST_LIB})
add_dependencies(ucrtbase rust_ucrt_shared)

# The Rust archive is the whole DLL: one graph, with the SEH runtime and setjmp/longjmp as
# pseh-rt/setjmp-rt inside it (whole-archived, so defined before ntdll's import library,
# which exports longjmp on amd64, is consulted). So the link tolerates no duplicate symbol at
# all (link_strict, gcc.cmake). No C library: msvcrt_shared, crtmath, vcruntime, vcstartup, chkstk, wine and libsupc++ (once
# for __cxa_guard_*) contributed no object on any arch (tools/link_pulls.py, 2026-10-04), so
# they came off the line as msvcrt.dll's did; a symbol the Rust lacks is now an undefined-
# symbol error rather than a silent C fallback. msvcrt_shared is still compiled: the cohab
# checks read its objects as the reference surface (tools/verify_ucrtbase_cohab.py).
target_link_libraries(ucrtbase
    -Wl,--whole-archive ${UCRT_SHARED_RUST_LIB} -Wl,--no-whole-archive
)
link_strict(ucrtbase)
# Build-order only, nothing is linked: verify_ucrtbase_cohab and ucrt_burndown read these
# libraries' OBJECTS as the reference surface (OURS_REFERENCE), and a build only makes what
# `livecd` depends on. Off the link line, nothing else would build them in a fresh tree --
# exactly how msvcrt_static's absence turned master red (DIFFTEST_STALE_ORACLE, 2026-10-04).
add_dependencies(ucrtbase msvcrt_shared crtmath vcruntime vcstartup chkstk wine setjmp pseh)

# Implicitly link to vcstartup
target_link_libraries(libucrtbase INTERFACE vcstartup)

add_importlibs(ucrtbase kernel32 ntdll)

add_cd_file(TARGET ucrtbase DESTINATION reactos/system32 FOR all)
