
if(ARCH STREQUAL "i386")
    list(APPEND ASM_SOURCE
        i386/seh.s
        i386/seh_prolog.s)
endif()

if(USE_NATIVE_SEH OR ((CMAKE_C_COMPILER_ID STREQUAL "Clang") AND (ARCH STREQUAL "amd64")))
    list(APPEND SOURCE dummy.c)
    add_asm_files(pseh_asm ${ASM_SOURCE})
    add_library(pseh ${SOURCE} ${pseh_asm})
    add_dependencies(pseh asm)
elseif((CMAKE_C_COMPILER_ID STREQUAL "GNU") AND (ARCH STREQUAL "amd64"))
    # for GCC amd64 this is just an interface library, with our home-made plugin
    add_library(pseh INTERFACE)
    if (CMAKE_HOST_WIN32)
        target_compile_options(pseh INTERFACE
            $<$<COMPILE_LANGUAGE:C>:-fplugin=$<TARGET_FILE:native-gcc_plugin_seh>>
            $<$<COMPILE_LANGUAGE:CXX>:-fplugin=$<TARGET_FILE:native-g++_plugin_seh>>)
    else()
        target_compile_options(pseh INTERFACE $<$<COMPILE_LANGUAGE:C,CXX>:-fplugin=$<TARGET_FILE:native-gcc_plugin_seh>>)
    endif()
else()

    if(USE_PSEH3)
        include_directories(${REACTOS_SOURCE_DIR}/sdk/include/reactos/libs/pseh)
        list(APPEND SOURCE
            i386/pseh3.c
            i386/pseh3_i386.S)
    elseif(USE_PSEH_CLANG)
        list(APPEND SOURCE
            i386/pseh_clang.c
            i386/pseh_clang_asm.S)
    elseif(USE_DUMMY_PSEH)
        list(APPEND SOURCE dummy.c)
    elseif(ARCH STREQUAL "i386")
        list(APPEND SOURCE
            i386/framebased.S
            i386/framebased-gcchack.c
            i386/framebased-gcchack-asm.S)
    endif()

    add_library(pseh ${SOURCE} ${ASM_SOURCE})
    target_link_libraries(pseh chkstk)
    add_dependencies(pseh psdk)

    target_include_directories(pseh PRIVATE include/pseh)
endif()

target_include_directories(pseh INTERFACE include)

# Make it clear that we are using PSEH2
if ((CMAKE_C_COMPILER_ID STREQUAL "GNU") OR
    ((CMAKE_C_COMPILER_ID STREQUAL "Clang") AND (NOT (ARCH STREQUAL "amd64"))))
endif()

# pseh-rust in its user-mode build (seh.s's SEH runtime + dummy.c's volatile symbols, the
# same Rust the kernel links): the CRT DLLs list it before their other libraries so lld
# resolves the runtime from it, and compile no PSEH C or asm of their own.
rust_staticlib_variant(pseh_user_rust_a pseh-rust "user" FALSE)
# Consumers write ${PSEH_USER_LIB} (gcc.cmake), this archive's path.
if(NOT pseh_user_rust_a_LIB STREQUAL PSEH_USER_LIB)
    message(FATAL_ERROR "PSEH_USER_LIB (${PSEH_USER_LIB}) is not pseh-rust's archive (${pseh_user_rust_a_LIB})")
endif()
