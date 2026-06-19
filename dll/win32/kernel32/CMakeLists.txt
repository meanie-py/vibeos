
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
    # client/actctx.c — replaced by Rust
    # client/appcache.c — replaced by Rust
    # client/atom.c       — replaced by Rust (kernel32-rust/src/client/atom.rs)
    # client/compname.c — replaced by Rust
    # client/debugger.c — replaced by Rust
    # client/dosdev.c — replaced by Rust
    client/dllmain.c
    # client/environ.c    — replaced by Rust (kernel32-rust/src/client/environ.rs)
    # client/except.c — replaced by Rust
    # client/fiber.c — replaced by Rust
    # client/handle.c     — replaced by Rust (kernel32-rust/src/client/handle.rs)
    # client/heapmem.c — replaced by Rust
    # client/job.c — replaced by Rust
    # client/loader.c — replaced by Rust
    # client/path.c       — replaced by Rust (kernel32-rust/src/client/path.rs)
    # client/perfcnt.c — replaced by Rust
    # client/power.c — replaced by Rust
    # client/proc.c   — replaced by Rust (sdk/lib/rust/kernel32-rust/); cohab-linked
    #                   via libkernel32_rust.a below. Kept on disk (never-delete rule).
    # client/resntfy.c — replaced by Rust
    # client/session.c — replaced by Rust
    # client/synch.c — replaced by Rust
    # client/sysinfo.c — replaced by Rust
    # client/time.c — replaced by Rust
    # client/timerqueue.c — replaced by Rust
    # client/toolhelp.c — replaced by Rust (kernel32-rust/src/client/toolhelp.rs); cohab-linked via libkernel32_rust.a
    # client/utils.c — replaced by Rust
    # client/thread.c — replaced by Rust
    # client/vdm.c — replaced by Rust
    # client/version.c    — replaced by Rust (kernel32-rust/src/client/version.rs)
    # client/virtmem.c — replaced by Rust
    # client/console/* — DEFERRED back to C (CSR CONSOLE_API_MESSAGE types not yet in FFI;
    #   console/*.rs translations preserved on disk):
    client/console/alias.c
    client/console/console.c
    client/console/history.c
    client/console/init.c
    client/console/readwrite.c
    client/file/move.c
    # client/file/tape.c — replaced by Rust (kernel32-rust/src/file/tape.rs); cohab-linked via libkernel32_rust.a
    # client/file/{backup,cnotify,copy,create,delete,deviceio,dir,disk,fileinfo,filemap,filename,find,
    #   hardlink,iocompl,lfile,lock,mailslot,mntpoint,npipe,rw,volume}.c — replaced by Rust (kernel32-rust/src/file/)
    # wine/actctx.c — replaced by Rust
    # wine/comm.c — replaced by Rust (kernel32-rust/src/wine/comm.rs); cohab-linked via libkernel32_rust.a
    # wine/lzexpand.c — replaced by Rust (kernel32-rust/src/wine/lzexpand.rs); cohab-linked via libkernel32_rust.a
    # wine/muldiv.c       — replaced by Rust (kernel32-rust/src/wine/muldiv.rs)
    # wine/profile.c — replaced by Rust (kernel32-rust/src/wine/profile.rs); cohab-linked via libkernel32_rust.a
    wine/res.c
    # wine/timezone.c — replaced by Rust
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
