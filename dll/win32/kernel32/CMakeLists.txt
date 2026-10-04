
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

list(APPEND SOURCE
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
    # client/console/* — replaced by Rust (kernel32-rust/src/console/); cohab-linked via
    #   libkernel32_rust.a (CSR CONSOLE_API_MESSAGE family bridged through winsrv-sys):
    # client/console/alias.c
    # client/console/console.c
    # client/console/history.c
    # client/console/init.c
    # client/console/readwrite.c
    # client/file/move.c — replaced by Rust (kernel32-rust/src/file/move.rs); cohab-linked via libkernel32_rust.a
    # client/file/tape.c — replaced by Rust (kernel32-rust/src/file/tape.rs); cohab-linked via libkernel32_rust.a
    # client/file/{backup,cnotify,copy,create,delete,deviceio,dir,disk,fileinfo,filemap,filename,find,
    #   hardlink,iocompl,lfile,lock,mailslot,mntpoint,npipe,rw,volume}.c — replaced by Rust (kernel32-rust/src/file/)
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
    # --- new upstream TUs since the fork -------------------------------------
    # The *ansi.c files are upstream splitting the ANSI wrappers out of TUs we have
    # already flipped to Rust; kernel32-rust exports both the A and W entry points,
    # so building them would duplicate symbols the Rust twins own.
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
    # kernel32.spec already exports Get/SetProcessDEPPolicy and GetSystemDEPPolicy
    # with nothing implementing them, so this fills a real gap.
    client/dep.c
    k32.h)

# Disable warning: 'variable' defined but not used
set_source_files_properties(winnls/string/locale.c PROPERTIES COMPILE_OPTIONS "-Wno-unused-variable")

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
rust_storage_target(KERNEL32_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-kernel32-target" kernel32-rust)
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
rust_crate_sources(KERNEL32_RUST_SRCS "${KERNEL32_RUST_WORKSPACE_DIR}" kernel32-rust)
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

# KERNEL32_RUST_LIB first: its locale defs (e.g. LCIDToLocaleName) win so the vista-rust
# locale cgu stays dormant in kernel32.dll (as the C kernel32_vista_static did) — it only
# goes live in kernel32_vista.dll, which has the ntdll_vista (0x600) importlib for RtlLcid*.
target_link_libraries(kernel32 psapi_static ${KERNEL32_RUST_LIB} ${KERNEL32_VISTA_RUST_LIB} ${KERNEL32_SHARED_RUST_LIB} wine chkstk ${PSEH_LIB})
add_importlibs(kernel32 ntdll)
# kernel32-vista-rust's locale/sync/initonce code references the NT6 (0x600+) ntdll Rtl
# exports (RtlLcidToLocaleName/RtlLocaleNameToLcid/RtlRunOnce*/SRW*); the coarse Rust
# codegen unit pulls them into kernel32.dll even though the locale path is dead here, so
# resolve them via the ntdll_vista (0x600) import lib (delay-loaded; same runtime ntdll.dll).
add_delay_importlibs(kernel32 ntdll_vista)
add_pch(kernel32 k32.h SOURCE)
add_dependencies(kernel32 psdk errcodes asm rust_kernel32 rust_kernel32_shared rust_kernel32_vista)
add_cd_file(TARGET kernel32 DESTINATION reactos/system32 FOR all)
