
# Unified llvm-mingw toolchain file for ReactOS (i386, amd64, arm64)
#
# Usage:
#   cmake -DARCH=i386  -DCMAKE_TOOLCHAIN_FILE=../reactos/toolchain-llvm-mingw.cmake ../reactos
#   cmake -DARCH=amd64 -DCMAKE_TOOLCHAIN_FILE=../reactos/toolchain-llvm-mingw.cmake ../reactos
#   cmake -DARCH=arm64 -DCMAKE_TOOLCHAIN_FILE=../reactos/toolchain-llvm-mingw.cmake ../reactos

# pass variables necessary for the toolchain (needed for try_compile)
set(CMAKE_TRY_COMPILE_PLATFORM_VARIABLES ARCH LLVM_MINGW_PATH)

# The name of the target operating system
set(CMAKE_SYSTEM_NAME Windows)

# Map ARCH to triplet, system processor, and dlltool machine
if(ARCH STREQUAL "i386")
    set(CMAKE_SYSTEM_PROCESSOR i686)
    set(triplet i686-w64-mingw32)
    set(CMAKE_DLLTOOL_MACHINE "i386" CACHE STRING "dlltool machine type")
elseif(ARCH STREQUAL "amd64")
    set(CMAKE_SYSTEM_PROCESSOR x86_64)
    set(triplet x86_64-w64-mingw32)
    set(CMAKE_DLLTOOL_MACHINE "i386:x86-64" CACHE STRING "dlltool machine type")
elseif(ARCH STREQUAL "arm64")
    set(CMAKE_SYSTEM_PROCESSOR aarch64)
    set(triplet aarch64-w64-mingw32)
    set(CMAKE_DLLTOOL_MACHINE "arm64" CACHE STRING "dlltool machine type")
else()
    message(FATAL_ERROR "Unsupported ARCH: ${ARCH}. Use i386, amd64, or arm64.")
endif()

# Find llvm-mingw - user can override with LLVM_MINGW_PATH
if(NOT DEFINED LLVM_MINGW_PATH)
    # Try .build-tools/llvm-mingw inside the source tree first
    if(EXISTS "${CMAKE_CURRENT_LIST_DIR}/.build-tools/llvm-mingw")
        set(LLVM_MINGW_PATH "${CMAKE_CURRENT_LIST_DIR}/.build-tools/llvm-mingw")
    else()
        # Try to find it relative to source parent (legacy layout)
        get_filename_component(_src_parent "${CMAKE_CURRENT_LIST_DIR}/.." ABSOLUTE)
        file(GLOB _llvm_dirs "${_src_parent}/llvm-mingw-*")
        if(_llvm_dirs)
            list(GET _llvm_dirs 0 LLVM_MINGW_PATH)
        endif()
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

# Signal to gcc.cmake that we're using llvm-mingw (affects nostdinc, libc++, lld behavior)
set(USE_LLVM_MINGW TRUE CACHE BOOL "Using llvm-mingw toolchain" FORCE)

# For i386/amd64: Find GNU assembler for 16-bit real-mode code (boot sectors, frldr16, MVDM).
# Clang's integrated assembler can't handle some 16-bit constructs (.org with backwards offset,
# symbol arithmetic in memory operands). We use GNU as from Homebrew mingw-w64 as a fallback.
if(NOT ARCH STREQUAL "arm64")
    if(NOT DEFINED GNU_AS_COMPILER)
        # Try x86_64 version first (can assemble both 32-bit and 16-bit code)
        find_program(GNU_AS_COMPILER
            NAMES x86_64-w64-mingw32-as i686-w64-mingw32-as)
        if(NOT GNU_AS_COMPILER)
            file(GLOB _gas_candidates
                "/opt/homebrew/Cellar/mingw-w64/*/toolchain-x86_64/bin/x86_64-w64-mingw32-as"
                "/opt/homebrew/Cellar/mingw-w64/*/toolchain-i686/bin/i686-w64-mingw32-as")
            if(_gas_candidates)
                list(GET _gas_candidates -1 GNU_AS_COMPILER)
            endif()
        endif()
    endif()
    # Boot sectors are always 16/32-bit x86, assembled with GCC + GNU as.
    # Find Homebrew's i686-w64-mingw32-gcc which handles both preprocessing and assembly.
    if(NOT DEFINED CMAKE_ASM16_GCC)
        find_program(CMAKE_ASM16_GCC NAMES i686-w64-mingw32-gcc)
        if(NOT CMAKE_ASM16_GCC)
            file(GLOB _gcc16_candidates "/opt/homebrew/Cellar/mingw-w64/*/toolchain-i686/bin/i686-w64-mingw32-gcc")
            if(_gcc16_candidates)
                list(GET _gcc16_candidates -1 CMAKE_ASM16_GCC)
            endif()
        endif()
    endif()
    if(CMAKE_ASM16_GCC)
        message(STATUS "16-bit assembler (GCC): ${CMAKE_ASM16_GCC}")
    else()
        message(WARNING "i686-w64-mingw32-gcc not found. 16-bit assembly (boot sectors) will fail. Install mingw-w64: brew install mingw-w64")
    endif()
endif()

# This allows to have CMake test the compiler without linking
set(CMAKE_TRY_COMPILE_TARGET_TYPE STATIC_LIBRARY)

# Delete the archive first. `ar r` replaces or inserts members and never removes one, so
# without this an object dropped from a library's source list keeps linking from the stale
# .obj a warm build dir still holds: vcruntime's setjmp objects did, after setjmp-rust took
# their place (2026-10-04), and amd64/i386 l3codeca.acm linked a stale flat-layout libmpg123
# over the current one (found by tools/link_guard.py). CMake's own default rule removes the
# target; overriding it lost that.
set(CMAKE_C_CREATE_STATIC_LIBRARY "${CMAKE_COMMAND} -E rm -f <TARGET>" "<CMAKE_AR> crT <TARGET> <LINK_FLAGS> <OBJECTS>")
set(CMAKE_CXX_CREATE_STATIC_LIBRARY ${CMAKE_C_CREATE_STATIC_LIBRARY})
set(CMAKE_ASM_CREATE_STATIC_LIBRARY ${CMAKE_C_CREATE_STATIC_LIBRARY})

# Don't link with anything by default unless we say so
set(CMAKE_C_STANDARD_LIBRARIES "" CACHE STRING "Standard C Libraries")
set(CMAKE_CXX_STANDARD_LIBRARIES "" CACHE STRING "Standard C++ Libraries")

# Use lld linker.
# --no-insert-timestamp: reproducible builds (deterministic-builds-plan.md rung 2).
# The gnu ld.lld driver defaults to --insert-timestamp, stamping the PE header with
# wall-clock time so every rebuild diverges and cross-worktree dedup can't fire. Zero
# it here for all 679 cmake DLL/driver/exe links (the cargo-driven links zero it in
# reactos-link). Verify with `vibeos-cli repro report <pe>` (TimeDateStamp → 0).
set(_VIBEOS_LLD_COMMON "-nostdlib -fuse-ld=lld -Wl,--enable-auto-image-base,--disable-auto-import -Wl,--no-insert-timestamp")
set(CMAKE_SHARED_LINKER_FLAGS_INIT "${_VIBEOS_LLD_COMMON}")
set(CMAKE_MODULE_LINKER_FLAGS_INIT "${_VIBEOS_LLD_COMMON}")
set(CMAKE_EXE_LINKER_FLAGS_INIT "${_VIBEOS_LLD_COMMON}")

set(CMAKE_USER_MAKE_RULES_OVERRIDE "${CMAKE_CURRENT_LIST_DIR}/overrides-gcc.cmake")
