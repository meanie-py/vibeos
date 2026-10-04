
include_directories(inc)

if(DBG)
    add_compile_definitions(_DEBUG) # TODO: define this globally
endif()

# Silence GCC/Clang warnings
if(CMAKE_C_COMPILER_ID STREQUAL "GNU" OR
   CMAKE_C_COMPILER_ID STREQUAL "Clang")
    add_compile_options(
        -Wno-builtin-declaration-mismatch
        -Wno-unused-function
    )
endif()

add_compile_definitions(
    _CORECRT_BUILD
    _CRT_DECLARE_GLOBAL_VARIABLES_DIRECTLY
    _NTSYSTEM_
)

if(${ARCH} STREQUAL "i386")
    list(APPEND VCRT_SETJMP_SOURCES
        i386/longjmp.c
    )
    list(APPEND VCRT_SETJMP_ASM_SOURCES
        i386/__longjmp_nounwind.s
        i386/_seh_longjmp_unwind.s
        i386/_setjmp.s
        i386/_setjmp3.s
    )
elseif(${ARCH} STREQUAL "amd64")
    list(APPEND VCRT_SETJMP_SOURCES
        amd64/longjmp.c
    )
    list(APPEND VCRT_SETJMP_ASM_SOURCES
        amd64/__longjmp_noframe.s
        amd64/_setjmp.s
    )
elseif(${ARCH} STREQUAL "arm")
    list(APPEND VCRT_SETJMP_SOURCES
        arm/longjmp.c
    )
    list(APPEND VCRT_SETJMP_ASM_SOURCES
        arm/__longjmp_noframe.s
        arm/_setjmp.s
    )
elseif(${ARCH} STREQUAL "arm64")
    list(APPEND VCRT_SETJMP_SOURCES
        arm64/longjmp.c
    )
    list(APPEND VCRT_SETJMP_ASM_SOURCES
        arm64/_setjmp.s
        arm64/__longjmp_noframe.s
    )
endif()
add_asm_files(vcrt_setjmp_asm ${VCRT_SETJMP_ASM_SOURCES})

# Separate library, so we can link it to ftfd
add_library(setjmp ${VCRT_SETJMP_SOURCES} ${vcrt_setjmp_asm})
add_dependencies(setjmp psdk asm)


# i386/_chkesp.s is ucrt-rust's ucrt_reactos.rs (global_asm!) for ucrtbase, and msvcrt's
# misc.rs carries Wine's own for msvcrt/msvcrt20/msvcrt40/crtdll; vcruntime builds no asm.

list(APPEND VCRUNTIME_SOURCES
    __std_terminate.c
    __vcrt_init.c
    purecall.cpp
)

if(${ARCH} STREQUAL "i386")
    list(APPEND VCRUNTIME_SOURCES
        i386/_chkesp_failed.c
    )
endif()

# setjmp/longjmp for the runtime library: setjmp-rust (2026-10-04), the same .s/.c
# transcribed (global_asm! with offset_of! operands; instruction-identical to these
# objects). The C `setjmp` library above stays for its non-CRT consumers (ftfd,
# static_crt_apitest); vcruntime no longer embeds its objects.
rust_staticlib_variant(vcrt_setjmp_rust setjmp-rust "user" FALSE)
# The CRT DLLs also name it by path, ${VCRT_SETJMP_LIB} (gcc.cmake).
if(NOT vcrt_setjmp_rust_LIB STREQUAL VCRT_SETJMP_LIB)
    message(FATAL_ERROR "VCRT_SETJMP_LIB (${VCRT_SETJMP_LIB}) is not setjmp-rust's archive (${vcrt_setjmp_rust_LIB})")
endif()

# Runtime library (linked into ucrtbase)
add_library(vcruntime ${VCRUNTIME_SOURCES})
add_dependencies(vcruntime rust_vcrt_setjmp_rust)
target_link_libraries(vcruntime ${PSEH_LIB} ${vcrt_setjmp_rust_LIB})
target_include_directories(vcruntime INTERFACE inc)
