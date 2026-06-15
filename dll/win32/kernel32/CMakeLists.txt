
add_subdirectory(kernel32_vista)

add_definitions(-D_KERNEL32_)
include_directories(${REACTOS_SOURCE_DIR}/sdk/include/reactos/subsys)
spec2def(kernel32.dll kernel32.spec ADD_IMPORTLIB)

# Shared between kernel32 and kernel32_vista
add_library(kernel32_shared
    client/shared_utils.c
    client/file/fileutils.c
)
target_link_libraries(kernel32_shared PRIVATE pseh)
add_dependencies(kernel32_shared psdk)

list(APPEND SOURCE
    client/actctx.c
    client/appcache.c
    # client/atom.c       — replaced by Rust (kernel32-rust/src/client/atom.rs)
    client/compname.c
    client/debugger.c
    client/dosdev.c
    client/dllmain.c
    # client/environ.c    — replaced by Rust (kernel32-rust/src/client/environ.rs)
    client/except.c
    client/fiber.c
    # client/handle.c     — replaced by Rust (kernel32-rust/src/client/handle.rs)
    client/heapmem.c
    client/job.c
    client/loader.c
    # client/path.c       — replaced by Rust (kernel32-rust/src/client/path.rs)
    client/perfcnt.c
    client/power.c
    # client/proc.c   — replaced by Rust (sdk/lib/rust/kernel32-rust/); cohab-linked
    #                   via libkernel32_rust.a below. Kept on disk (never-delete rule).
    client/resntfy.c
    client/session.c
    client/synch.c
    client/sysinfo.c
    client/time.c
    client/timerqueue.c
    client/toolhelp.c
    client/utils.c
    client/thread.c
    client/vdm.c
    # client/version.c    — replaced by Rust (kernel32-rust/src/client/version.rs)
    client/virtmem.c
    client/console/alias.c
    client/console/console.c
    client/console/history.c
    client/console/init.c
    client/console/readwrite.c
    client/file/backup.c
    client/file/cnotify.c
    client/file/copy.c
    client/file/create.c
    client/file/delete.c
    client/file/deviceio.c
    client/file/dir.c
    client/file/disk.c
    client/file/fileinfo.c
    client/file/filemap.c
    client/file/filename.c
    client/file/find.c
    client/file/hardlink.c
    client/file/iocompl.c
    client/file/lfile.c
    client/file/lock.c
    client/file/mailslot.c
    client/file/move.c
    client/file/mntpoint.c
    client/file/npipe.c
    client/file/rw.c
    client/file/tape.c
    client/file/volume.c
    wine/actctx.c
    wine/comm.c
    wine/lzexpand.c
    # wine/muldiv.c       — replaced by Rust (kernel32-rust/src/wine/muldiv.rs)
    wine/profile.c
    wine/res.c
    wine/timezone.c
    # winnls/string/casemap.c    — replaced by Rust (kernel32-rust/src/string/casemap.rs)
    # winnls/string/collation.c  — replaced by Rust (kernel32-rust/src/string/collation.rs)
    # winnls/string/digitmap.c   — replaced by Rust (kernel32-rust/src/string/digitmap.rs)
    # winnls/string/fold.c       — replaced by Rust (kernel32-rust/src/string/fold.rs)
    # winnls/string/format_msg.c — replaced by Rust (kernel32-rust/src/string/format_msg.rs)
    # winnls/string/japanese.c   — replaced by Rust (kernel32-rust/src/string/japanese.rs)
    # winnls/string/locale.c   — replaced by Rust (kernel32-rust/src/string/locale.rs)
    # winnls/string/lcformat.c — replaced by Rust (kernel32-rust/src/string/lcformat.rs)
    # winnls/string/lstring.c    — replaced by Rust (kernel32-rust/src/string/lstring.rs)
    # winnls/string/nls.c      — replaced by Rust (kernel32-rust/src/string/nls.rs)
    # winnls/string/sortkey.c    — replaced by Rust (kernel32-rust/src/string/sortkey.rs)
    k32.h)

if(MSVC)
    # Disable warning C4146: unary minus operator applied to unsigned type, result still unsigned
    set_source_files_properties(wine/res.c PROPERTIES COMPILE_OPTIONS "/wd4146")

    # Disable warning C4267: 'initializing': conversion from 'size_t' to 'DWORD', possible loss of data
    set_source_files_properties(winnls/string/locale.c PROPERTIES COMPILE_OPTIONS "/wd4267")
else()
    # Disable warning: 'variable' defined but not used
    set_source_files_properties(winnls/string/locale.c PROPERTIES COMPILE_OPTIONS "-Wno-unused-variable")
endif()

if(ARCH STREQUAL "i386")
    list(APPEND ASM_SOURCE
        client/i386/fiber.S
        client/i386/thread.S)
elseif(ARCH STREQUAL "amd64")
    list(APPEND ASM_SOURCE
        client/amd64/fiber.S)
elseif(ARCH STREQUAL "arm64")
    list(APPEND ASM_SOURCE
        client/arm64/fiber.S)
endif()

#
# kernel32-rust cohab: client/proc.c (the CreateProcessInternalW path) → Rust.
# Builds libkernel32_rust.a from the workspace crate (STANDARD user-mode per-arch
# targets; i386 gets -sse,-sse2 via sdk/lib/rust/.cargo/config.toml — the user-mode
# no-SSE rule). The crate's #[no_mangle] symbols are the sole providers of proc.c's
# functions+globals now that the C TU is commented out (conutils-style cohab; no
# --allow-multiple-definition needed since every proc.c symbol was removed).
#
set(KERNEL32_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
set(KERNEL32_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-kernel32-target")
set(KERNEL32_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libkernel32_rust.a")
if(ARCH STREQUAL "i386")
    set(KERNEL32_RUST_TARGET "i686-pc-windows-gnu")
elseif(ARCH STREQUAL "amd64")
    set(KERNEL32_RUST_TARGET "x86_64-pc-windows-gnu")
elseif(ARCH STREQUAL "arm64")
    set(KERNEL32_RUST_TARGET "aarch64-pc-windows-gnullvm")
endif()
# GLOB_RECURSE so .rs under src/<subdir>/ (e.g. src/string/locale.rs) also retrigger
# the libkernel32_rust.a rebuild — a non-recursive src/*.rs glob silently used a stale
# archive when only subdir files changed.
file(GLOB_RECURSE KERNEL32_RUST_SRCS "${KERNEL32_RUST_WORKSPACE_DIR}/kernel32-rust/src/*.rs")
add_custom_command(
    OUTPUT ${KERNEL32_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${KERNEL32_RUST_TARGET_DIR}
        cargo build --release --target ${KERNEL32_RUST_TARGET}
            -p kernel32-rust
            --manifest-path ${KERNEL32_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${KERNEL32_RUST_TARGET_DIR}/${KERNEL32_RUST_TARGET}/release/libkernel32_rust.a
        ${KERNEL32_RUST_LIB}
    DEPENDS ${KERNEL32_RUST_SRCS}
            ${KERNEL32_RUST_WORKSPACE_DIR}/kernel32-rust/Cargo.toml
    WORKING_DIRECTORY ${KERNEL32_RUST_WORKSPACE_DIR}
    COMMENT "Building kernel32-rust (${KERNEL32_RUST_TARGET})"
    VERBATIM)
add_custom_target(rust_kernel32 DEPENDS ${KERNEL32_RUST_LIB})

add_asm_files(kernel32_asm ${ASM_SOURCE})
add_library(kernel32 MODULE
    ${SOURCE}
    ${kernel32_asm}
    kernel32.rc
    ${CMAKE_CURRENT_BINARY_DIR}/kernel32_stubs.c
    ${CMAKE_CURRENT_BINARY_DIR}/kernel32.def)

set_module_type(kernel32 win32dll ENTRYPOINT DllMain 12)
#############################################
## HACK FOR MSVC COMPILATION WITH win32dll ##
set_subsystem(kernel32 console)
################# END  HACK #################

target_link_libraries(kernel32 psapi_static kernel32_vista_static kernel32_shared wine chkstk ${PSEH_LIB} ${KERNEL32_RUST_LIB})
add_importlibs(kernel32 ntdll)
add_pch(kernel32 k32.h SOURCE)
add_dependencies(kernel32 psdk errcodes asm rust_kernel32)
add_cd_file(TARGET kernel32 DESTINATION reactos/system32 FOR all)
