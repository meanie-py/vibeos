
# ARM64 toolchain file for ReactOS using llvm-mingw

# pass variables necessary for the toolchain (needed for try_compile)
set(CMAKE_TRY_COMPILE_PLATFORM_VARIABLES ARCH LLVM_MINGW_PATH)

# The name of the target operating system
set(CMAKE_SYSTEM_NAME Windows)
set(CMAKE_SYSTEM_PROCESSOR aarch64)

# Set the triplet
set(triplet aarch64-w64-mingw32)

# Find llvm-mingw - user can override with LLVM_MINGW_PATH
if(NOT DEFINED LLVM_MINGW_PATH)
    # Try to find it relative to source
    get_filename_component(_src_parent "${CMAKE_CURRENT_LIST_DIR}/.." ABSOLUTE)
    file(GLOB _llvm_dirs "${_src_parent}/llvm-mingw-*")
    if(_llvm_dirs)
        list(GET _llvm_dirs 0 LLVM_MINGW_PATH)
    endif()
endif()

if(LLVM_MINGW_PATH)
    message(STATUS "Using llvm-mingw from: ${LLVM_MINGW_PATH}")
    set(TOOLCHAIN_PREFIX "${LLVM_MINGW_PATH}/bin/${triplet}-")
else()
    # Assume tools are in PATH
    set(TOOLCHAIN_PREFIX "${triplet}-")
endif()

# Which tools to use
set(CMAKE_C_COMPILER "${TOOLCHAIN_PREFIX}clang")
set(CMAKE_C_COMPILER_TARGET ${triplet})
set(CMAKE_CXX_COMPILER "${TOOLCHAIN_PREFIX}clang++")
set(CMAKE_CXX_COMPILER_TARGET ${triplet})
set(CMAKE_ASM_COMPILER "${TOOLCHAIN_PREFIX}clang")
set(CMAKE_ASM_COMPILER_ID Clang)
# windmc is a host tool (generates headers from .mc files)
# llvm-mingw doesn't include windmc, so we need one from mingw-w64.
# Override with -DCMAKE_MC_COMPILER=/path/to/windmc if needed.
if(NOT DEFINED CMAKE_MC_COMPILER)
    find_program(CMAKE_MC_COMPILER
        NAMES x86_64-w64-mingw32-windmc windmc)
    if(NOT CMAKE_MC_COMPILER)
        # Try Homebrew on macOS
        file(GLOB _windmc_candidates "/opt/homebrew/Cellar/mingw-w64/*/toolchain-x86_64/bin/x86_64-w64-mingw32-windmc")
        if(_windmc_candidates)
            list(GET _windmc_candidates -1 CMAKE_MC_COMPILER)
        endif()
    endif()
endif()
set(CMAKE_RC_COMPILER "${TOOLCHAIN_PREFIX}windres")

# GNU windres, kept DISTINCT from llvm-mingw's windres above. llvm-mingw's
# `windres` is a thin wrapper over llvm-rc, and llvm-rc cannot parse DIALOGEX
# control creation data at all (`CONTROL "t",-1,"C",style,x,y,w,h { 1,2,3 }` ->
# "expected identifier, got {", verified on llvm-rc 18 and 19). One file in the
# tree needs it -- user32_winetest's resource.rc, which is Wine's oracle and not
# ours to rewrite -- so that ONE directory switches its RC rule to GNU windres
# via use_gnu_rc() (sdk/cmake/gcc.cmake). Same host dependency as windmc and
# GNU_AS_COMPILER: mingw-w64 binutils.
#
# Searched by the ARCH-PREFIXED name only. GNU windres emits for whatever target
# its binary was configured for and takes no cue from the enclosing build, so a
# bare `windres` (or the x86_64 one on an i386 build) produces a .res of the
# WRONG machine type -- `resource.rc.res: machine type x64 conflicts with x86`
# at link, which is at least loud, but the fix is to never pick it. arm64 has no
# GNU windres in mingw-w64 at all, so GNU_RC_COMPILER stays empty there and
# use_gnu_rc()'s callers skip themselves.
#
# Found by name and then PROVEN by --version, because llvm-mingw's bin/ is
# usually on PATH and its windres answers to the same prefixed name -- taking
# the first hit would silently re-select the compiler we are trying to avoid.
if(NOT DEFINED GNU_RC_COMPILER)
    if(ARCH STREQUAL "i386")
        set(_gnu_rc_names i686-w64-mingw32-windres)
    elseif(ARCH STREQUAL "amd64")
        set(_gnu_rc_names x86_64-w64-mingw32-windres)
    else()
        set(_gnu_rc_names "")
    endif()
    if(_gnu_rc_names)
        # Cache var keyed by ARCH: find_program CACHES its result, and a bare
        # name would make the first arch configured in a shared cache decide
        # for the others -- which is the wrong-machine-type .res again.
        find_program(_gnu_rc_candidate_${ARCH}
            NAMES ${_gnu_rc_names}
            PATHS /usr/bin /usr/local/bin /opt/homebrew/bin
            NO_CMAKE_FIND_ROOT_PATH)
        if(_gnu_rc_candidate_${ARCH})
            execute_process(COMMAND ${_gnu_rc_candidate_${ARCH}} --version
                            OUTPUT_VARIABLE _gnu_rc_version
                            ERROR_QUIET OUTPUT_STRIP_TRAILING_WHITESPACE)
            if(_gnu_rc_version MATCHES "GNU windres")
                set(GNU_RC_COMPILER "${_gnu_rc_candidate_${ARCH}}")
            endif()
        endif()
    endif()
endif()

set(CMAKE_AR "${TOOLCHAIN_PREFIX}ar")
set(CMAKE_DLLTOOL "${TOOLCHAIN_PREFIX}dlltool")
set(CMAKE_OBJCOPY "${TOOLCHAIN_PREFIX}objcopy")

# llvm-dlltool requires -m flag for target machine
set(CMAKE_DLLTOOL_MACHINE "arm64" CACHE STRING "dlltool machine type")

# This allows to have CMake test the compiler without linking
set(CMAKE_TRY_COMPILE_TARGET_TYPE STATIC_LIBRARY)

# Delete the archive first. `ar r` replaces or inserts members and never removes one, so
# without this an object dropped from a library's source list keeps linking from the stale
# .obj a warm build dir still holds: vcruntime's setjmp objects did, after setjmp-rust took
# their place (2026-10-04). CMake's own default rule removes the target; overriding it lost that.
set(CMAKE_C_CREATE_STATIC_LIBRARY "${CMAKE_COMMAND} -E rm -f <TARGET>" "<CMAKE_AR> crT <TARGET> <LINK_FLAGS> <OBJECTS>")
set(CMAKE_CXX_CREATE_STATIC_LIBRARY ${CMAKE_C_CREATE_STATIC_LIBRARY})
set(CMAKE_ASM_CREATE_STATIC_LIBRARY ${CMAKE_C_CREATE_STATIC_LIBRARY})

# Don't link with anything by default unless we say so
set(CMAKE_C_STANDARD_LIBRARIES "" CACHE STRING "Standard C Libraries")
set(CMAKE_CXX_STANDARD_LIBRARIES "" CACHE STRING "Standard C++ Libraries")

# Use lld linker
set(CMAKE_SHARED_LINKER_FLAGS_INIT "-nostdlib -fuse-ld=lld -Wl,--enable-auto-image-base,--disable-auto-import")
set(CMAKE_MODULE_LINKER_FLAGS_INIT "-nostdlib -fuse-ld=lld -Wl,--enable-auto-image-base,--disable-auto-import")
set(CMAKE_EXE_LINKER_FLAGS_INIT "-nostdlib -fuse-ld=lld -Wl,--enable-auto-image-base,--disable-auto-import")

# Signal gcc.cmake that we're using llvm-mingw (for nostdinc/libc++ paths)
set(USE_LLVM_MINGW TRUE CACHE BOOL "Using llvm-mingw toolchain")

set(CMAKE_USER_MAKE_RULES_OVERRIDE "${CMAKE_CURRENT_LIST_DIR}/overrides-gcc.cmake")
