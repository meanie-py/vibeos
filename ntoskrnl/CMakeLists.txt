
PROJECT(NTOS)

if (CMAKE_C_COMPILER_ID STREQUAL "GNU" OR CMAKE_C_COMPILER_ID STREQUAL "Clang")
    # Enable this again. CORE-17637
    add_compile_options(-Wunused-result)
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

list(APPEND PCH_SKIP_SOURCE
    guid.c)

list(APPEND PCH_SKIP_SOURCE
    ${REACTOS_SOURCE_DIR}/ntoskrnl/kd64/kd64_rust_shims.c
    ${REACTOS_SOURCE_DIR}/ntoskrnl/ob/ob_rust_shims.c
    ${REACTOS_SOURCE_DIR}/ntoskrnl/ps/ps_rust_shims.c
    ${REACTOS_SOURCE_DIR}/ntoskrnl/se/se_rust_shims.c
    ${REACTOS_SOURCE_DIR}/ntoskrnl/tier1_rust_shims.c)

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
    DEPENDS ${RUST_WORKSPACE_DIR}/kernel-rust/src/lib.rs
            ${RUST_WORKSPACE_DIR}/kernel-rust/Cargo.toml
            ${RUST_WORKSPACE_DIR}/pool/src/lib.rs
            ${RUST_WORKSPACE_DIR}/pool/src/expool.rs
            ${RUST_WORKSPACE_DIR}/pool/src/pool.rs
            ${RUST_WORKSPACE_DIR}/pool/src/globals.rs
            ${RUST_WORKSPACE_DIR}/pool/src/types.rs
            ${RUST_WORKSPACE_DIR}/pool/Cargo.toml
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/lib.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ob/mod.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ob/obhandle.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/se/mod.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/se/access.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/se/accesschk.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/se/acl.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/se/audit.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/se/client.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/se/debug.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/se/objtype.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/se/priv_.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/se/sd.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/se/semgr.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/se/sid.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/se/sqos.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/se/srm.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/se/subject.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/se/token.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/se/tokenadj.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/se/tokencls.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/se/tokenlif.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ps/mod.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ps/apphelp.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ps/debug.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ps/job.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ps/kill.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ps/process.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ps/psmgr.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ps/psnotify.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ps/query.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ps/quota.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ps/security.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ps/state.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ps/thread.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/ps/win32.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/lpc/mod.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/lpc/close.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/lpc/complete.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/lpc/connect.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/lpc/create.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/lpc/listen.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/lpc/port.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/lpc/reply.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/lpc/send.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/dbgk/mod.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/dbgk/dbgkobj.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/dbgk/dbgkutil.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/po/mod.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/po/events.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/po/poshtdwn.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/po/povolume.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/po/power.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/inbv/mod.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/inbv/bootanim.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/inbv/inbv.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/inbv/inbvport.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/wmi/mod.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/wmi/guidobj.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/wmi/smbios.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/wmi/wmi.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/wmi/wmidrv.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/vdm/mod.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/vdm/vdmexec.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/vdm/vdmmain.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/vf/mod.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/vf/driver.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/rtl/mod.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/rtl/libsupp.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/src/rtl/misc.rs
            ${RUST_WORKSPACE_DIR}/ntoskrnl-rust/Cargo.toml
            ${RUST_WORKSPACE_DIR}/ntkrnl-sys/src/lib.rs
            ${RUST_WORKSPACE_DIR}/ntkrnl-sys/src/base.rs
            ${RUST_WORKSPACE_DIR}/ntkrnl-sys/src/mm.rs
            ${RUST_WORKSPACE_DIR}/ntkrnl-sys/src/ke.rs
            ${RUST_WORKSPACE_DIR}/ntkrnl-sys/src/ex.rs
            ${RUST_WORKSPACE_DIR}/ntkrnl-sys/src/ps.rs
            ${RUST_WORKSPACE_DIR}/ntkrnl-sys/src/ob.rs
            ${RUST_WORKSPACE_DIR}/ntkrnl-sys/src/rtl.rs
            ${RUST_WORKSPACE_DIR}/ntkrnl-sys/src/interlocked.rs
            ${RUST_WORKSPACE_DIR}/ntkrnl-sys/src/constants.rs
            ${RUST_WORKSPACE_DIR}/Cargo.toml
    COMMENT "Building Rust kernel code (${RUST_TARGET})"
)
add_custom_target(rust_kernel DEPENDS ${RUST_KERNEL_LIB})

add_executable(ntoskrnl
    ${ntoskrnl_asm}
    ${NTOSKRNL_SOURCE}
    ${PCH_SKIP_SOURCE}
    ntoskrnl.rc
    ${CMAKE_CURRENT_BINARY_DIR}/ntoskrnl.def)
set_module_type(ntoskrnl kernel)
set_target_properties(ntoskrnl PROPERTIES C_STANDARD 23)

source_group(TREE ${REACTOS_SOURCE_DIR}/ntoskrnl PREFIX "Source Files" FILES ${NTOSKRNL_SOURCE})

target_link_libraries(ntoskrnl ntoskrnl_vista cportlib csq ${PSEH_LIB} arbiter cmlib ntlsalib rtl ${ROSSYM_LIB} libcntpr setjmp wdmguid poguid ioevent ${RUST_KERNEL_LIB})

if(STACK_PROTECTOR)
    target_link_libraries(ntoskrnl gcc_ssp_nt)
endif()

add_importlibs(ntoskrnl hal kdcom bootvid)
add_pch(ntoskrnl ${REACTOS_SOURCE_DIR}/ntoskrnl/include/ntoskrnl.h "${PCH_SKIP_SOURCE}")
add_dependencies(ntoskrnl psdk asm rust_kernel)
add_cd_file(TARGET ntoskrnl DESTINATION reactos/system32 NO_CAB FOR all)

add_subdirectory(ntkrnlmp)

add_asm_files(ntdllsys_asm ntdll.S)
add_library(ntdllsys ${ntdllsys_asm})
set_target_properties(ntdllsys PROPERTIES LINKER_LANGUAGE "C")
add_dependencies(ntdllsys asm)
