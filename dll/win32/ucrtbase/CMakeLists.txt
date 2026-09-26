
spec2def(ucrtbase.dll ucrtbase.spec NO_DBG ADD_IMPORTLIB)

# Hack to replace the old CRT include directory with the UCRT include directory
get_property(INCLUDE_DIRS DIRECTORY . PROPERTY INCLUDE_DIRECTORIES)
list(REMOVE_ITEM INCLUDE_DIRS "${REACTOS_SOURCE_DIR}/sdk/include/crt")
set_property(DIRECTORY . PROPERTY INCLUDE_DIRECTORIES ${INCLUDE_DIRS})
include_directories(${REACTOS_SOURCE_DIR}/sdk/include/ucrt)

add_compile_definitions(
    _UCRT
    _CORECRT_BUILD
    CRTDLL
)

add_library(ucrtbase SHARED
    printf.c
    stubs.c
    ucrtbase_stubs.c
    ucrtbase.def
)

if(NOT MSVC)
    target_compile_options(ucrtbase PRIVATE -Wno-builtin-declaration-mismatch)
endif()

if(CMAKE_C_COMPILER_ID STREQUAL "Clang")
    # Silence warnings in ucrtbase_stubs.c
    target_compile_options(ucrtbase PRIVATE -Wno-incompatible-library-redeclaration)
endif()

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
    set(UCRT_RUST_TARGET "i686-pc-windows-gnu")
    set(UCRT_RUST_ENV "RUSTFLAGS=-C target-feature=-sse,-sse2")
elseif(ARCH STREQUAL "amd64")
    set(UCRT_RUST_TARGET "x86_64-pc-windows-gnu")
    set(UCRT_RUST_ENV "")
elseif(ARCH STREQUAL "arm64")
    set(UCRT_RUST_TARGET "aarch64-pc-windows-gnullvm")
    set(UCRT_RUST_ENV "")
endif()

set(UCRT_SHARED_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libcrt_shared_cargo.a")
# Every crate whose code lands in libcrt_shared_cargo.a -- the path-dependency
# closure of crt-shared-cargo. A crate missing here compiles into the archive but
# never triggers its rebuild, so an edit to it ships a STALE archive with no error:
# this list had drifted to two of seven crates (ucrt-rust's takeover families and
# cnt-rust's math were both invisible to it). verify_ucrtbase_cohab.py derives the
# closure from the manifests and fails if this list and it disagree.
set(UCRT_SHARED_RUST_CRATES
    crt-shared-cargo cnt-rust crt-portable-rust crt-shared-rust panic-common msvcrt-sys
    ucrt-rust)
set(UCRT_SHARED_RUST_GLOBS)
set(UCRT_SHARED_RUST_MANIFESTS)
foreach(crate ${UCRT_SHARED_RUST_CRATES})
    list(APPEND UCRT_SHARED_RUST_GLOBS ${UCRT_RUST_WORKSPACE_DIR}/${crate}/src/*.rs)
    list(APPEND UCRT_SHARED_RUST_MANIFESTS ${UCRT_RUST_WORKSPACE_DIR}/${crate}/Cargo.toml)
endforeach()
file(GLOB_RECURSE UCRT_SHARED_RUST_SOURCES CONFIGURE_DEPENDS ${UCRT_SHARED_RUST_GLOBS})
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
            ${UCRT_SHARED_RUST_MANIFESTS}
            ${UCRT_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMENT "Building Rust ucrtbase msvcrt_shared cohab (${UCRT_RUST_TARGET})"
)
add_custom_target(rust_ucrt_shared DEPENDS ${UCRT_SHARED_RUST_LIB})
add_dependencies(ucrtbase rust_ucrt_shared)

# --whole-archive for the same reason msvcrt.dll needs it: plain link order is not enough,
# because msvcrt_shared still gets pulled for the symbols Rust does NOT define and drags in
# its own copies of the ones it does, which would then win first-match. Forcing the Rust
# defs in first makes the C copies the ignored duplicates.
target_link_libraries(ucrtbase
    ucrt
    -Wl,--whole-archive ${UCRT_SHARED_RUST_LIB} -Wl,--no-whole-archive
    msvcrt_shared
    crtmath
    vcruntime
    vcstartup
    chkstk
    wine
)

# Implicitly link to vcstartup
target_link_libraries(libucrtbase INTERFACE vcstartup)

if(MSVC)
    target_link_libraries(ucrtbase runtmchk)
else()
    # For __cxa_guard_acquire / __cxa_guard_release
    target_link_libraries(ucrtbase libsupc++)
endif()

add_importlibs(ucrtbase kernel32 ntdll)

add_cd_file(TARGET ucrtbase DESTINATION reactos/system32 FOR all)
