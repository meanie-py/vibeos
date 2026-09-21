
# Replace the old CRT include directory with the UCRT include directory
get_property(INCLUDE_DIRS DIRECTORY . PROPERTY INCLUDE_DIRECTORIES)
list(REMOVE_ITEM INCLUDE_DIRS "${REACTOS_SOURCE_DIR}/sdk/include/crt")
set_property(DIRECTORY . PROPERTY INCLUDE_DIRECTORIES ${INCLUDE_DIRS})
include_directories(${REACTOS_SOURCE_DIR}/sdk/include/ucrt)

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
    list(APPEND VCRT_COMMON_ASM_SOURCES
        i386/__security_check_cookie.s
        i386/_chkesp.s
    )
elseif(${ARCH} STREQUAL "amd64")
    list(APPEND VCRT_COMMON_ASM_SOURCES
        amd64/__security_check_cookie.s
    )
endif()
add_asm_files(vcrt_common_asm ${VCRT_COMMON_ASM_SOURCES})

list(APPEND VCRT_COMMON_SOURCES
    ${vcrt_common_asm}
    __report_gsfailure.c
    __report_rangecheckfailure.c
    __security_init_cookie.c
    _fltused.c
    initializers.cpp
    isa_available.cpp
    tlssup.c
)

# Common between vcstartup and vcruntime
add_library(vcrt_common ${VCRT_COMMON_SOURCES})
target_link_libraries(vcrt_common ${PSEH_LIB})
add_dependencies(vcrt_common psdk)

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

list(APPEND VCRT_RUNTIME_SOURCES
    ${vcrt_runtime_asm}
    __std_terminate.c
    __vcrt_init.c
    purecall.cpp
)

if(${ARCH} STREQUAL "i386")
    list(APPEND VCRT_RUNTIME_SOURCES
        i386/_chkesp_failed.c
    )
endif()

# Runtime library (linked into ucrtbase)
add_library(vcruntime ${VCRT_RUNTIME_SOURCES} $<TARGET_OBJECTS:vcrt_common> $<TARGET_OBJECTS:setjmp>)
target_link_libraries(vcruntime ${PSEH_LIB})
target_include_directories(vcruntime INTERFACE inc)

list(APPEND VCRT_STARTUP_SOURCES
    __acrt_initialize_stub.cpp
    __scrt_uninitialize_crt.cpp
    __vcrt_init_stubs.c
    _onexit.c
    atexit.c
    mainCRTStartup.cpp
    wmainCRTStartup.cpp
)

if(NOT MSVC)
    # clang/gcc-mingw emit a `call __main` at the top of every `main` to run the
    # `.ctors` global constructors, so an exe that links vcstartup instead of
    # msvcrtex needs the same hook msvcrtex gets from startup/gccmain.c. It used
    # to come from `-lgcc`, which the pinned llvm-mingw toolchain does not ship:
    # that is why ucrtbase_winetest was the one winetest that could not link.
    # A no-op would also satisfy the linker and would silently skip every global
    # constructor, so use the real walk.
    list(APPEND VCRT_STARTUP_SOURCES
        ${REACTOS_SOURCE_DIR}/sdk/lib/crt/startup/gccmain.c
    )
endif()

# Startup library (linked into executables, linking to ucrtbase)
add_library(vcstartup ${VCRT_STARTUP_SOURCES} $<TARGET_OBJECTS:vcrt_common>)
target_link_libraries(vcstartup ${PSEH_LIB} libucrtbase libkernel32)
