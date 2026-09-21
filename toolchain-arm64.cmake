
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
# Found by NAME then PROVEN by --version, because llvm-mingw's bin/ is usually
# on PATH and its windres answers to the same name -- taking the first hit would
# silently re-select the compiler we are trying to avoid.
if(NOT DEFINED GNU_RC_COMPILER)
    find_program(_gnu_rc_candidate
        NAMES x86_64-w64-mingw32-windres i686-w64-mingw32-windres windres
        PATHS /usr/bin /usr/local/bin /opt/homebrew/bin
        NO_CMAKE_FIND_ROOT_PATH)
    if(_gnu_rc_candidate)
        execute_process(COMMAND ${_gnu_rc_candidate} --version
                        OUTPUT_VARIABLE _gnu_rc_version
                        ERROR_QUIET OUTPUT_STRIP_TRAILING_WHITESPACE)
        if(_gnu_rc_version MATCHES "GNU windres")
            set(GNU_RC_COMPILER "${_gnu_rc_candidate}")
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

set(CMAKE_C_CREATE_STATIC_LIBRARY "<CMAKE_AR> crT <TARGET> <LINK_FLAGS> <OBJECTS>")
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
