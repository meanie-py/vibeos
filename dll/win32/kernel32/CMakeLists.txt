add_definitions(-D_KERNEL32_)
include_directories(${REACTOS_SOURCE_DIR}/sdk/include/reactos/subsys)
spec2def(kernel32.dll kernel32.spec ADD_IMPORTLIB)

# client/shared_utils.c + client/file/fileutils.c — replaced by Rust
# (sdk/lib/rust/kernel32-shared-rust). This .a is linked into BOTH kernel32.dll
# AND kernel32_vista.dll (the old kernel32_shared C static lib is dropped). The .c
# files stay on disk (never-delete). Defined here, BEFORE add_subdirectory(kernel32_vista),
# so the subdir can depend on it. See docs/engineering/kernel32-shared-vista-rust-plan.md.
set(KERNEL32_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
if(ARCH STREQUAL "i386")
    set(KERNEL32_RUST_TARGET "i686-pc-windows-gnu")
elseif(ARCH STREQUAL "amd64")
    set(KERNEL32_RUST_TARGET "x86_64-pc-windows-gnu")
elseif(ARCH STREQUAL "arm64")
    set(KERNEL32_RUST_TARGET "aarch64-pc-windows-gnullvm")
endif()
rust_storage_target(KERNEL32_SHARED_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-kernel32-shared-target" kernel32-shared-rust)
set(KERNEL32_SHARED_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libkernel32_shared_rust.a")
rust_crate_sources(KERNEL32_SHARED_RUST_SRCS "${KERNEL32_RUST_WORKSPACE_DIR}" kernel32-shared-rust)
add_custom_command(
    OUTPUT ${KERNEL32_SHARED_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${KERNEL32_SHARED_RUST_TARGET_DIR}
        cargo build --release --target ${KERNEL32_RUST_TARGET}
            -p kernel32-shared-rust
            --manifest-path ${KERNEL32_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${KERNEL32_SHARED_RUST_TARGET_DIR}/${KERNEL32_RUST_TARGET}/release/libkernel32_shared_rust.a
        ${KERNEL32_SHARED_RUST_LIB}
    DEPENDS ${KERNEL32_SHARED_RUST_SRCS}
            ${KERNEL32_RUST_WORKSPACE_DIR}/kernel32-shared-rust/Cargo.toml
    WORKING_DIRECTORY ${KERNEL32_RUST_WORKSPACE_DIR}
    COMMENT "Building kernel32-shared-rust (${KERNEL32_RUST_TARGET})"
    VERBATIM)
add_custom_target(rust_kernel32_shared DEPENDS ${KERNEL32_SHARED_RUST_LIB})

# kernel32_vista_static C (the NT6/Vista+ API set) → Rust (kernel32-vista-rust).
# Built once → libkernel32_vista_rust.a, linked into BOTH kernel32.dll AND
# kernel32_vista.dll (the C kernel32_vista_static target is dropped; .c kept on disk).
# kernel32_vista/DllMain.c stays C (a #[no_mangle] DllMain here would collide with
# kernel32-rust's DllMain in kernel32.dll, which this .a also links into).
rust_storage_target(KERNEL32_VISTA_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-kernel32-vista-target" kernel32-vista-rust)
set(KERNEL32_VISTA_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libkernel32_vista_rust.a")
rust_crate_sources(KERNEL32_VISTA_RUST_SRCS "${KERNEL32_RUST_WORKSPACE_DIR}" kernel32-vista-rust)
add_custom_command(
    OUTPUT ${KERNEL32_VISTA_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${KERNEL32_VISTA_RUST_TARGET_DIR}
        cargo build --release --target ${KERNEL32_RUST_TARGET}
            -p kernel32-vista-rust
            --manifest-path ${KERNEL32_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${KERNEL32_VISTA_RUST_TARGET_DIR}/${KERNEL32_RUST_TARGET}/release/libkernel32_vista_rust.a
        ${KERNEL32_VISTA_RUST_LIB}
    DEPENDS ${KERNEL32_VISTA_RUST_SRCS}
            ${KERNEL32_RUST_WORKSPACE_DIR}/kernel32-vista-rust/Cargo.toml
    WORKING_DIRECTORY ${KERNEL32_RUST_WORKSPACE_DIR}
    COMMENT "Building kernel32-vista-rust (${KERNEL32_RUST_TARGET})"
    VERBATIM)
add_custom_target(rust_kernel32_vista DEPENDS ${KERNEL32_VISTA_RUST_LIB})

# kernel32_vista links KERNEL32_{SHARED,VISTA}_RUST_LIB + deps on the rust targets (above).
add_subdirectory(kernel32_vista)

# Disable warning: 'variable' defined but not used
set_source_files_properties(winnls/string/locale.c PROPERTIES COMPILE_OPTIONS "-Wno-unused-variable")

if(ARCH STREQUAL "i386")
    list(APPEND ASM_SOURCE
        client/i386/fiber.S
        client/i386/thread.S)
# client/amd64/fiber.S, client/arm64/fiber.S — replaced by Rust: SwitchToFiber and
#   BaseFiberStartup are naked functions in kernel32-rust src/client/fiber.rs (the .S were
#   an UNIMPLEMENTED stub and a brk, so 64-bit fibers never worked).
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

add_asm_files(kernel32_asm ${ASM_SOURCE})

################# END  HACK #################

# kernel32.dll is all Rust (kernel32-rust) and cargo links it:
# sdk/lib/rust/kernel32-cargo (reactos-link, generated by tools/cargo_cell_gen.py).
# The C stays on disk, uncompiled.
# The translated C, kept on disk:
# client/actctx.c — replaced by Rust
# client/appcache.c — replaced by Rust
# client/atom.c       — replaced by Rust (kernel32-rust/src/client/atom.rs)
# client/compname.c — replaced by Rust
# client/debugger.c — replaced by Rust
# client/dosdev.c — replaced by Rust
# client/dllmain.c    — replaced by Rust (kernel32-rust/src/client/dllmain.rs); cohab-linked via libkernel32_rust.a
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
# client/console/alias.c
# client/console/console.c
# client/console/history.c
# client/console/init.c
# client/console/readwrite.c
# client/file/move.c — replaced by Rust (kernel32-rust/src/file/move.rs); cohab-linked via libkernel32_rust.a
# client/file/tape.c — replaced by Rust (kernel32-rust/src/file/tape.rs); cohab-linked via libkernel32_rust.a
# wine/actctx.c — replaced by Rust
# wine/comm.c — replaced by Rust (kernel32-rust/src/wine/comm.rs); cohab-linked via libkernel32_rust.a
# wine/lzexpand.c — replaced by Rust (kernel32-rust/src/wine/lzexpand.rs); cohab-linked via libkernel32_rust.a
# wine/muldiv.c       — replaced by Rust (kernel32-rust/src/wine/muldiv.rs)
# wine/profile.c — replaced by Rust (kernel32-rust/src/wine/profile.rs); cohab-linked via libkernel32_rust.a
# wine/res.c — replaced by Rust (kernel32-rust/src/wine/res.rs); cohab-linked via libkernel32_rust.a
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
# The *ansi.c files are upstream splitting the ANSI wrappers out of TUs we have
#client/atomansi.c
#client/file/copyansi.c
#client/file/diransi.c
#client/file/fileinfoansi.c
#client/file/mntpointansi.c
#client/file/moveansi.c
#client/file/volumeansi.c
#client/pathansi.c
#client/procansi.c
#client/synchansi.c
#client/sysinfoansi.c
#client/versionansi.c
#winnls/string/lcformatansi.c
#winnls/string/localeansi.c
# dep.c is genuinely new code (extracted from proc.c upstream) with no Rust twin;
# The C still compiled, in an OBJECT library whose compile the cargo link replays.
set(KERNEL32_SHIMS client/dep.c)
if(ARCH STREQUAL "i386")
    list(APPEND KERNEL32_SHIMS client/i386/fiber.S client/i386/thread.S)
endif()
set(KERNEL32_SHIMS_TARGET)
# Generated inputs those compiles read, ordered before them.
if(KERNEL32_SHIMS)
    add_library(kernel32_shims OBJECT ${KERNEL32_SHIMS})
    set(KERNEL32_SHIMS_TARGET kernel32_shims)
    target_link_libraries(kernel32_shims PRIVATE psapi_static wine chkstk libntdll)
    add_dependencies(kernel32_shims psdk errcodes asm)
endif()
add_cargo_module(kernel32 kernel32-cargo kernel32.dll
    KIND win32dll
    BODY_CRATE kernel32-rust
    DESTINATION reactos/system32
    DELAY_IMPORTS
    EXTRA_DEPENDS ${KERNEL32_SHIMS_TARGET} psdk)
