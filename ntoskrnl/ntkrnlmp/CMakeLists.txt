
include(../ntos.cmake)

include_directories(${REACTOS_SOURCE_DIR}/ntoskrnl)
spec2def(ntkrnlmp.exe ../ntoskrnl.spec)

if(MSVC_IDE)
    set_source_files_properties(${NTKRNLMP_ASM_SOURCE} PROPERTIES COMPILE_DEFINITIONS "CONFIG_SMP")
endif()

add_asm_files(ntkrnlmp_asm ${NTKRNLMP_ASM_SOURCE})

# mproc.c is now translated to Rust (ke/i386/mproc.rs, ke/amd64/mproc.rs)
# and linked via kernel-rust. The C originals are no longer compiled.
#if(ARCH STREQUAL "i386")
#    list(APPEND NTKRNLMP_SOURCE
#        ${REACTOS_SOURCE_DIR}/ntoskrnl/ke/i386/mproc.c)
#elseif(ARCH STREQUAL "amd64")
#    list(APPEND NTKRNLMP_SOURCE
#        ${REACTOS_SOURCE_DIR}/ntoskrnl/ke/amd64/mproc.c)
#elseif(ARCH STREQUAL "arm")
#    #TBD
#endif()

list(APPEND NTKRNLMP_PCH_SKIP_SOURCE
    ${REACTOS_SOURCE_DIR}/ntoskrnl/guid.c
    ${REACTOS_SOURCE_DIR}/ntoskrnl/ke/ke_rust_shims.c
    ${REACTOS_SOURCE_DIR}/ntoskrnl/kd/kd_rust_shims.c
    ${REACTOS_SOURCE_DIR}/ntoskrnl/kernel_c_helpers.c
    ${REACTOS_SOURCE_DIR}/ntoskrnl/mm/mm_rust_shims.c
    ${REACTOS_SOURCE_DIR}/ntoskrnl/config/cm_rust_shims.c)

add_executable(ntkrnlmp
    ${ntkrnlmp_asm}
    ${NTKRNLMP_SOURCE}
    ${NTKRNLMP_PCH_SKIP_SOURCE}
    ${REACTOS_SOURCE_DIR}/ntoskrnl/ntoskrnl.rc
    ${CMAKE_CURRENT_BINARY_DIR}/ntkrnlmp.def)
set_module_type(ntkrnlmp kernel)
target_compile_definitions(ntkrnlmp PRIVATE CONFIG_SMP)

# Allow multiple definitions: Rust compiler_builtins and libclang_rt both provide __chkstk
target_link_options(ntkrnlmp PRIVATE "LINKER:--allow-multiple-definition")

# dynamic analysis switches
if(STACK_PROTECTOR)
    target_link_libraries(ntkrnlmp gcc_ssp_nt)
endif()

target_link_libraries(ntkrnlmp ntoskrnl_vista cportlib csq ${PSEH_LIB} arbiter cmlib ntlsalib rtl ${ROSSYM_LIB} libcntpr setjmp wdmguid poguid ioevent ${RUST_KERNEL_LIB})
add_importlibs(ntkrnlmp hal kdcom bootvid)
add_pch(ntkrnlmp ${REACTOS_SOURCE_DIR}/ntoskrnl/include/ntoskrnl.h "${NTKRNLMP_PCH_SKIP_SOURCE}")
add_dependencies(ntkrnlmp psdk asm rust_kernel)
add_cd_file(TARGET ntkrnlmp DESTINATION reactos/system32 NO_CAB FOR all)
