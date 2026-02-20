
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

# Rust Pool Allocator — replaces mm/ARM3/expool.c and mm/ARM3/pool.c
set(RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
set(RUST_POOL_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-pool-target")
if(ARCH STREQUAL "i386")
    set(RUST_POOL_TARGET "i686-pc-windows-gnu")
elseif(ARCH STREQUAL "amd64")
    set(RUST_POOL_TARGET "x86_64-pc-windows-gnu")
elseif(ARCH STREQUAL "arm64")
    set(RUST_POOL_TARGET "aarch64-pc-windows-gnullvm")
endif()

set(RUST_POOL_LIB "${CMAKE_CURRENT_BINARY_DIR}/libreactos_pool.a")
add_custom_command(
    OUTPUT ${RUST_POOL_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${RUST_POOL_TARGET_DIR}
        cargo build --release --target ${RUST_POOL_TARGET}
            -p reactos-pool
            --manifest-path ${RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${RUST_POOL_TARGET_DIR}/${RUST_POOL_TARGET}/release/libreactos_pool.a
        ${RUST_POOL_LIB}
    DEPENDS ${RUST_WORKSPACE_DIR}/pool/src/lib.rs
            ${RUST_WORKSPACE_DIR}/pool/src/expool.rs
            ${RUST_WORKSPACE_DIR}/pool/src/pool.rs
            ${RUST_WORKSPACE_DIR}/pool/src/globals.rs
            ${RUST_WORKSPACE_DIR}/pool/src/types.rs
            ${RUST_WORKSPACE_DIR}/pool/Cargo.toml
            ${RUST_WORKSPACE_DIR}/ntkrnl-sys/src/lib.rs
            ${RUST_WORKSPACE_DIR}/ntkrnl-sys/src/base.rs
            ${RUST_WORKSPACE_DIR}/ntkrnl-sys/src/mm.rs
            ${RUST_WORKSPACE_DIR}/ntkrnl-sys/src/ke.rs
            ${RUST_WORKSPACE_DIR}/ntkrnl-sys/src/ex.rs
            ${RUST_WORKSPACE_DIR}/ntkrnl-sys/src/ps.rs
            ${RUST_WORKSPACE_DIR}/ntkrnl-sys/src/rtl.rs
            ${RUST_WORKSPACE_DIR}/ntkrnl-sys/src/interlocked.rs
            ${RUST_WORKSPACE_DIR}/Cargo.toml
    COMMENT "Building Rust Pool Allocator (${RUST_POOL_TARGET})"
)
add_custom_target(rust_pool DEPENDS ${RUST_POOL_LIB})

add_executable(ntoskrnl
    ${ntoskrnl_asm}
    ${NTOSKRNL_SOURCE}
    ${PCH_SKIP_SOURCE}
    ntoskrnl.rc
    ${CMAKE_CURRENT_BINARY_DIR}/ntoskrnl.def)
set_module_type(ntoskrnl kernel)
set_target_properties(ntoskrnl PROPERTIES C_STANDARD 23)

source_group(TREE ${REACTOS_SOURCE_DIR}/ntoskrnl PREFIX "Source Files" FILES ${NTOSKRNL_SOURCE})

target_link_libraries(ntoskrnl ntoskrnl_vista cportlib csq ${PSEH_LIB} arbiter cmlib ntlsalib rtl ${ROSSYM_LIB} libcntpr setjmp wdmguid poguid ioevent ${RUST_POOL_LIB})

if(STACK_PROTECTOR)
    target_link_libraries(ntoskrnl gcc_ssp_nt)
endif()

add_importlibs(ntoskrnl hal kdcom bootvid)
add_pch(ntoskrnl ${REACTOS_SOURCE_DIR}/ntoskrnl/include/ntoskrnl.h "${PCH_SKIP_SOURCE}")
add_dependencies(ntoskrnl psdk asm rust_pool)
add_cd_file(TARGET ntoskrnl DESTINATION reactos/system32 NO_CAB FOR all)

add_subdirectory(ntkrnlmp)

add_asm_files(ntdllsys_asm ntdll.S)
add_library(ntdllsys ${ntdllsys_asm})
set_target_properties(ntdllsys PROPERTIES LINKER_LANGUAGE "C")
add_dependencies(ntdllsys asm)
