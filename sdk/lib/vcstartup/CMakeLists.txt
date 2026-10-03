
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
    list(APPEND VCSTARTUP_ASM_SOURCES
        i386/__security_check_cookie.s
    )
elseif(${ARCH} STREQUAL "amd64")
    list(APPEND VCSTARTUP_ASM_SOURCES
        amd64/__security_check_cookie.s
    )
endif()
add_asm_files(vcstartup_asm ${VCSTARTUP_ASM_SOURCES})

list(APPEND VCSTARTUP_SOURCES
    ${vcstartup_asm}
    __acrt_initialize_stub.cpp
    __report_gsfailure.c
    __report_rangecheckfailure.c
    __scrt_uninitialize_crt.cpp
    __security_init_cookie.c
    __vcrt_init_stubs.c
    _fltused.c
    _onexit.c
    atexit.c
    initializers.cpp
    isa_available.cpp
    mainCRTStartup.cpp
    tlssup.c
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
    list(APPEND VCSTARTUP_SOURCES
        ${REACTOS_SOURCE_DIR}/sdk/lib/crt/startup/gccmain.c
    )
endif()

# Startup library (linked into executables, linking to ucrtbase)
add_library(vcstartup ${VCSTARTUP_SOURCES})
target_link_libraries(vcstartup ${PSEH_LIB} libkernel32)
target_include_directories(vcstartup PRIVATE  $<TARGET_PROPERTY:vcruntime,INTERFACE_INCLUDE_DIRECTORIES>)
