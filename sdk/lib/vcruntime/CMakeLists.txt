
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


if(${ARCH} STREQUAL "i386")
    list(APPEND VCRUNTIME_ASM_SOURCES
        i386/_chkesp.s
    )
elseif(${ARCH} STREQUAL "amd64")
    list(APPEND VCRUNTIME_ASM_SOURCES

    )
endif()
add_asm_files(vcruntime_asm ${VCRUNTIME_ASM_SOURCES})

list(APPEND VCRUNTIME_SOURCES
    ${vcruntime_asm}
    __std_terminate.c
    __vcrt_init.c
    purecall.cpp
)

if(${ARCH} STREQUAL "i386")
    list(APPEND VCRUNTIME_SOURCES
        i386/_chkesp_failed.c
    )
endif()

# Runtime library (linked into ucrtbase)
add_library(vcruntime ${VCRUNTIME_SOURCES} $<TARGET_OBJECTS:setjmp>)
target_link_libraries(vcruntime ${PSEH_LIB})
target_include_directories(vcruntime INTERFACE inc)
