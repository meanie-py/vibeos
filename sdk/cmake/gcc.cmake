
# Set up dlltool machine flag for llvm-dlltool (needed for ARM64)
if(CMAKE_DLLTOOL_MACHINE)
    set(DLLTOOL_MACHINE_FLAG -m ${CMAKE_DLLTOOL_MACHINE})
else()
    set(DLLTOOL_MACHINE_FLAG "")
endif()

# Real delay-load imports for llvm-mingw (sdk/tools/gen-delaylib.py). OFF
# links every add_delay_importlibs() library as an ORDINARY import, loaded
# eagerly: rpcrt4 then pulls wininet -> shell32 -> comctl32 into every process
# that touches RPC, which is why a plain console winetest found comctl32
# already loaded (user32:class). It was parked OFF for an i386 0x1E after
# explorer spawned; that was the pushlock-wait-block bug, fixed 2026-07-05
# and re-verified with delay-load ON on 2026-07-11. VIBEOS_NO_DELAYLOAD still
# forces the old behaviour. arm64 was held OFF for a day: with it ON, arm64
# kmtest boots hit 0xE3 in 4 of 5 rounds. That was not delay-load but a fastfat
# shim testing the wrong IRP_CONTEXT WAIT bit (the fastfat cohab shim), which
# delay-load's later DLL page-ins made collide; fixed, 10 of 10 clean.
option(VIBEOS_ENABLE_DELAYLOAD "Generate real delay-import libraries" ON)

# Show a note about ccache build
if(ENABLE_CCACHE)
    message("-- Enabling ccache build - done")
    set(CMAKE_C_USE_RESPONSE_FILE_FOR_INCLUDES OFF)
    set(CMAKE_CXX_USE_RESPONSE_FILE_FOR_INCLUDES OFF)
endif()

# PDB style debug info
if(NOT DEFINED SEPARATE_DBG)
    set(SEPARATE_DBG FALSE)
endif()

# Dwarf based builds (no rsym)
if(CMAKE_BUILD_TYPE STREQUAL "Release")
    set(NO_ROSSYM TRUE)
elseif(NOT ARCH STREQUAL "i386")
    set(NO_ROSSYM TRUE)
elseif(USE_LLVM_MINGW)
    # rsym can't handle Clang-generated binaries (long mangled names, different debug info)
    set(NO_ROSSYM TRUE)
elseif(NOT DEFINED NO_ROSSYM)
    set(NO_ROSSYM FALSE)
endif()

# PSEH configuration for llvm-mingw:
# - i386: Setjmp-based PSEH (USE_PSEH_CLANG). Always catches via longjmp, then
#   evaluates the filter in the original function context where ESP is correct.
#   Avoids PSEH3's embedded filter issue (Clang ESP-relative locals + different
#   stack in handler context = corruption).
# - ARM64/amd64: DUMMY_PSEH (no i386-style fs:[0] SEH chain; separate workarounds).
if(ARCH STREQUAL "arm64")
    set(USE_PSEH3 0)
    set(USE_DUMMY_PSEH 1)
elseif(USE_LLVM_MINGW AND ARCH STREQUAL "i386")
    set(USE_PSEH3 0)
    set(USE_DUMMY_PSEH 1)
    # TODO: Enable USE_PSEH_CLANG once setjmp-based PSEH is debugged
    # set(USE_DUMMY_PSEH 0)
    # set(USE_PSEH_CLANG 1)
elseif(USE_LLVM_MINGW)
    set(USE_PSEH3 0)
    set(USE_DUMMY_PSEH 1)
else()
    if(NOT DEFINED USE_PSEH3)
        set(USE_PSEH3 1)
    endif()
    if(NOT DEFINED USE_DUMMY_PSEH)
        set(USE_DUMMY_PSEH 0)
    endif()
endif()

if(USE_PSEH3)
    add_definitions(-D_USE_PSEH3=1)
endif()

if(USE_DUMMY_PSEH)
    add_definitions(-D_USE_DUMMY_PSEH=1)
endif()

if(USE_PSEH_CLANG)
    add_definitions(-D_USE_PSEH_CLANG=1)
endif()

if(USE_NATIVE_SEH)
    add_definitions(-D_USE_NATIVE_SEH=1)
endif()


if(STACK_PROTECTOR)
    add_compile_options(-fstack-protector-strong)
endif()

# Compiler Core
# note: -fno-common is default since GCC 10
add_compile_options(-pipe -fms-extensions -fno-strict-aliasing -fno-common)

# A long double is 64 bits (x86/amd64 only)
if(ARCH STREQUAL "i386" OR ARCH STREQUAL "amd64")
    add_compile_options(-mlong-double-64)
endif()

# Prevent GCC from searching any of the default directories.
if(USE_LLVM_MINGW)
    # llvm-mingw: use -nostdinc for all languages.
    # sdk/include/crt is excluded from include_directories() in CMakeLists.txt
    # and re-added here with lower priority so libc++ wrappers are found first.
    add_compile_options(-nostdinc)

    # Discover libc++ and clang builtin include paths from the toolchain
    get_filename_component(_llvm_mingw_compiler_dir ${CMAKE_CXX_COMPILER} DIRECTORY)
    get_filename_component(_llvm_mingw_toolchain_root ${_llvm_mingw_compiler_dir} DIRECTORY)
    get_filename_component(_llvm_mingw_compiler_name ${CMAKE_CXX_COMPILER} NAME)
    string(REGEX REPLACE "-clang\\+\\+$" "" _llvm_mingw_triplet "${_llvm_mingw_compiler_name}")
    set(_llvm_mingw_libcxx "${_llvm_mingw_toolchain_root}/${_llvm_mingw_triplet}/include/c++/v1")
    execute_process(
        COMMAND ${CMAKE_CXX_COMPILER} --version
        OUTPUT_VARIABLE _llvm_mingw_clang_ver_out OUTPUT_STRIP_TRAILING_WHITESPACE)
    string(REGEX MATCH "clang version ([0-9]+)" _llvm_mingw_match "${_llvm_mingw_clang_ver_out}")
    set(_llvm_mingw_clang_inc "${_llvm_mingw_toolchain_root}/lib/clang/${CMAKE_MATCH_1}/include")

    # For C/ASM: add CRT and clang builtins as -isystem
    # (vcruntime stays in -I via include_directories)
    add_compile_options("$<$<COMPILE_LANGUAGE:C,ASM>:-isystem${REACTOS_SOURCE_DIR}/sdk/include/crt>")
    if(EXISTS "${_llvm_mingw_clang_inc}")
        add_compile_options("$<$<COMPILE_LANGUAGE:C,ASM>:-isystem${_llvm_mingw_clang_inc}>")
    endif()
    # For RC (resource compiler): add CRT path via CMAKE_RC_FLAGS since windres
    # uses -I directly and doesn't support -isystem/-idirafter.
    set(CMAKE_RC_FLAGS "${CMAKE_RC_FLAGS} -I ${REACTOS_SOURCE_DIR}/sdk/include/crt")

    # For C++: add libc++ and clang builtins as -isystem, then CRT as -idirafter.
    # Search order: -I (ReactOS non-CRT) > -isystem (libc++ wrappers) > -idirafter (CRT)
    # This way libc++ wrapper headers are found first, and #include_next chains to CRT.
    if(EXISTS "${_llvm_mingw_libcxx}")
        add_compile_options("$<$<COMPILE_LANGUAGE:CXX>:-isystem${_llvm_mingw_libcxx}>")
    endif()
    if(EXISTS "${_llvm_mingw_clang_inc}")
        add_compile_options("$<$<COMPILE_LANGUAGE:CXX>:-isystem${_llvm_mingw_clang_inc}>")
    endif()
    add_compile_options("$<$<COMPILE_LANGUAGE:CXX>:-idirafter${REACTOS_SOURCE_DIR}/sdk/include/crt>")
else()
    # The case for C++ is handled through the reactos_c++ INTERFACE library
    add_compile_options("$<$<NOT:$<COMPILE_LANGUAGE:CXX>>:-nostdinc>")
endif()

if(CMAKE_C_COMPILER_ID STREQUAL "GNU")
    add_compile_options(-fno-aggressive-loop-optimizations)
    if (DBG)
        add_compile_options("$<$<COMPILE_LANGUAGE:C>:-Wold-style-declaration>")
    endif()

    # Disable all math intrinsics. The reason is that these are implicitly declared
    # extern by GCC, which causes inline functions to generate global symbols.
    # And since GCC is retarded, these symbols are not marked as weak, so they
    # conflict with each other in multiple compilation units.
    add_compile_options(-fno-builtin-acosf)
    add_compile_options(-fno-builtin-acosl)
    add_compile_options(-fno-builtin-asinf)
    add_compile_options(-fno-builtin-asinl)
    add_compile_options(-fno-builtin-atan2f)
    add_compile_options(-fno-builtin-atan2l)
    add_compile_options(-fno-builtin-atanf)
    add_compile_options(-fno-builtin-atanl)
    add_compile_options(-fno-builtin-ceilf)
    add_compile_options(-fno-builtin-ceill)
    add_compile_options(-fno-builtin-coshf)
    add_compile_options(-fno-builtin-coshl)
    add_compile_options(-fno-builtin-cosf)
    add_compile_options(-fno-builtin-cosl)
    add_compile_options(-fno-builtin-expf)
    add_compile_options(-fno-builtin-expl)
    add_compile_options(-fno-builtin-fabsf)
    add_compile_options(-fno-builtin-fabsl)
    add_compile_options(-fno-builtin-floorf)
    add_compile_options(-fno-builtin-floorl)
    add_compile_options(-fno-builtin-fmodf)
    add_compile_options(-fno-builtin-fmodl)
    add_compile_options(-fno-builtin-frexpf)
    add_compile_options(-fno-builtin-frexpl)
    add_compile_options(-fno-builtin-hypotf)
    add_compile_options(-fno-builtin-hypotl)
    add_compile_options(-fno-builtin-ldexpf)
    add_compile_options(-fno-builtin-ldexpl)
    add_compile_options(-fno-builtin-logf)
    add_compile_options(-fno-builtin-logl)
    add_compile_options(-fno-builtin-log10f)
    add_compile_options(-fno-builtin-log10l)
    add_compile_options(-fno-builtin-modff)
    add_compile_options(-fno-builtin-modfl)
    add_compile_options(-fno-builtin-powf)
    add_compile_options(-fno-builtin-powl)
    add_compile_options(-fno-builtin-sinhf)
    add_compile_options(-fno-builtin-sinhl)
    add_compile_options(-fno-builtin-sinf)
    add_compile_options(-fno-builtin-sinl)
    add_compile_options(-fno-builtin-sqrtf)
    add_compile_options(-fno-builtin-sqrtl)
    add_compile_options(-fno-builtin-tanhf)
    add_compile_options(-fno-builtin-tanhl)
    add_compile_options(-fno-builtin-tanf)
    add_compile_options(-fno-builtin-tanl)
    add_compile_options(-fno-builtin-feraiseexcept)
    add_compile_options(-fno-builtin-feupdateenv)

    if(CMAKE_C_COMPILER_VERSION VERSION_GREATER_EQUAL 11)
        add_compile_options(-fno-builtin-ceil)
        add_compile_options(-fno-builtin-ceilf)
        add_compile_options(-fno-builtin-cos)
        add_compile_options(-fno-builtin-floor)
        add_compile_options(-fno-builtin-floorf)
        add_compile_options(-fno-builtin-pow)
        add_compile_options(-fno-builtin-sin)
        add_compile_options(-fno-builtin-sincos)
        add_compile_options(-fno-builtin-sqrt)
        add_compile_options(-fno-builtin-sqrtf)
    endif()
    if(CMAKE_C_COMPILER_VERSION VERSION_GREATER_EQUAL 13)
        add_compile_options(-fno-builtin-erf)
        add_compile_options(-fno-builtin-erff)
        add_compile_options(-fno-builtin-execv)
        add_compile_options(-fno-builtin-execve)
        add_compile_options(-fno-builtin-execvp)
    endif()

elseif(CMAKE_C_COMPILER_ID STREQUAL "Clang")
    add_compile_options(-fno-associative-math)

    if(CMAKE_C_COMPILER_VERSION VERSION_GREATER_EQUAL 12.0)
        # disable "libcall optimization"
        # see https://mudongliang.github.io/2020/12/02/undefined-reference-to-stpcpy.html
        add_compile_options(-fno-builtin-stpcpy)
    endif()

    set(CMAKE_LINK_DEF_FILE_FLAG "")
    set(CMAKE_STATIC_LIBRARY_SUFFIX ".a")
    set(CMAKE_LINK_LIBRARY_SUFFIX "")
    set(CMAKE_CREATE_WIN32_EXE "")
    set(CMAKE_C_COMPILE_OPTIONS_PIC "")
    set(CMAKE_CXX_COMPILE_OPTIONS_PIC "")
    set(CMAKE_C_COMPILE_OPTIONS_PIE "")
    set(CMAKE_CXX_COMPILE_OPTIONS_PIE "")
    set(CMAKE_ASM_FLAGS_DEBUG "")
    set(CMAKE_C_FLAGS_DEBUG "")
    set(CMAKE_CXX_FLAGS_DEBUG "")
endif()

# Debugging
if(NOT CMAKE_BUILD_TYPE STREQUAL "Release")
    if(SEPARATE_DBG)
        add_compile_options(-gdwarf-2 -ggdb)
    else()
        add_compile_options(-gdwarf-2 -gstrict-dwarf)
        if(NOT CMAKE_C_COMPILER_ID STREQUAL Clang)
            add_compile_options(-femit-struct-debug-detailed=none -feliminate-unused-debug-symbols)
        endif()
    endif()
endif()

# Tuning
add_compile_options(-march=${OARCH} -mtune=${TUNE})

# Warnings, errors
if(NOT CMAKE_C_COMPILER_ID STREQUAL "Clang" AND
   NOT ARCH STREQUAL "amd64" AND
   NOT CMAKE_BUILD_TYPE STREQUAL "Release" AND
   NOT CMAKE_BUILD_TYPE STREQUAL "RelWithDebInfo" AND
   NOT CMAKE_BUILD_TYPE STREQUAL "MinSizeRel")
    add_compile_options(-Werror)
endif()

add_compile_options(-Wall -Wpointer-arith)

# Disable some overzealous warnings
add_compile_options(
    -Wno-char-subscripts
    -Wno-multichar
    -Wno-unused-value
    -Wno-unused-const-variable
    -Wno-unused-local-typedefs
    -Wno-deprecated
    -Wno-unused-result # FIXME To be removed when CORE-17637 is resolved
    -Wno-format
    -Wno-maybe-uninitialized
    -Wno-nonnull-compare
)

if(FALSE)
    add_compile_options(-Wno-attributes)
endif()

if(CMAKE_C_COMPILER_ID STREQUAL "GNU")
    add_compile_options(
        -Wno-unknown-pragmas
    )
elseif(CMAKE_C_COMPILER_ID STREQUAL "Clang")
    add_compile_options("$<$<COMPILE_LANGUAGE:C>:-Wno-microsoft>")
    add_compile_options(
        -Wno-pragma-pack
        -Wno-unknown-warning-option
    )
endif()

# Optimizations
# FIXME: Revisit this to see if we even need these levels
if(CMAKE_BUILD_TYPE STREQUAL "Release")
    add_compile_options(-O2 -DNDEBUG=)
    add_compile_options(-Wno-unused-variable)
    add_compile_options(-Wno-unused-but-set-variable)
else()
    if(OPTIMIZE STREQUAL "1")
        add_compile_options(-Os)
        if (CMAKE_C_COMPILER_ID STREQUAL "GNU")
            add_compile_options(-ftracer)
        endif()
    elseif(OPTIMIZE STREQUAL "2")
        add_compile_options(-Os)
    elseif(OPTIMIZE STREQUAL "3")
        add_compile_options(-Og)
    elseif(OPTIMIZE STREQUAL "4")
        add_compile_options(-O1)
    elseif(OPTIMIZE STREQUAL "5")
        add_compile_options(-O2)
    elseif(OPTIMIZE STREQUAL "6")
        add_compile_options(-O3)
    elseif(OPTIMIZE STREQUAL "7")
        add_compile_options(-Ofast)
    endif()
endif()

# Link-time code generation
if(LTCG)
    add_compile_options(-flto -fno-fat-lto-objects)
endif()

if(ARCH STREQUAL "i386")
    add_compile_options(-fno-optimize-sibling-calls -fno-omit-frame-pointer -mstackrealign)
    if(NOT CMAKE_C_COMPILER_ID STREQUAL "Clang")
        add_compile_options(-mpreferred-stack-boundary=3 -fno-set-stack-executable)
    endif()
    # FIXME: this doesn't work. CMAKE_BUILD_TYPE is always "Debug"
    if(NOT CMAKE_BUILD_TYPE STREQUAL "Debug")
        add_compile_options(-momit-leaf-frame-pointer)
    endif()
elseif(ARCH STREQUAL "amd64")
    if (CMAKE_C_COMPILER_ID STREQUAL "GNU")
        add_compile_options(-mpreferred-stack-boundary=4)
    endif()
endif()

# Other
if(ARCH STREQUAL "amd64")
    add_compile_options(-mcx16) # Generate CMPXCHG16
    add_definitions(-U_X86_ -UWIN32)
endif()

# Fix build with GLIBCXX + our c++ headers
add_definitions(-D_GLIBCXX_HAVE_BROKEN_VSWPRINTF)


# Fix build with UCRT headers
add_definitions(-D_CRT_SUPPRESS_RESTRICT)

# Alternative arch name
if(ARCH STREQUAL "amd64")
    set(ARCH2 x86_64)
else()
    set(ARCH2 ${ARCH})
endif()

if(SEPARATE_DBG)
    # PDB style debug puts all dwarf debug info in a separate dbg file
    message(STATUS "Building separate debug symbols")
    file(MAKE_DIRECTORY ${REACTOS_BINARY_DIR}/symbols)
    if(CMAKE_GENERATOR STREQUAL "Ninja")
        # Those variables seems to be set but empty in newer CMake versions
        # and Ninja generator relies on them to generate PDB name, so unset them.
        unset(MSVC_C_ARCHITECTURE_ID)
        unset(MSVC_CXX_ARCHITECTURE_ID)
        set(CMAKE_DEBUG_SYMBOL_SUFFIX "")
        set(SYMBOL_FILE <TARGET_PDB>)
    else()
        set(SYMBOL_FILE <TARGET>)
    endif()

    if (NOT NO_ROSSYM)
        get_target_property(RSYM native-rsym IMPORTED_LOCATION)
        set(strip_debug "${RSYM} -s ${REACTOS_SOURCE_DIR} <TARGET> <TARGET>")
    else()
        set(strip_debug "${CMAKE_STRIP} --strip-debug <TARGET>")
    endif()

    set(CMAKE_C_LINK_EXECUTABLE
        "<CMAKE_C_COMPILER> -Wl,--start-group <CMAKE_C_LINK_FLAGS> <LINK_FLAGS> -o <TARGET> <OBJECTS> <LINK_LIBRARIES> -Wl,--end-group"
        "${CMAKE_STRIP} --only-keep-debug <TARGET> -o ${REACTOS_BINARY_DIR}/symbols/${SYMBOL_FILE}"
        ${strip_debug})
    set(CMAKE_CXX_LINK_EXECUTABLE
        "<CMAKE_CXX_COMPILER> -Wl,--start-group <CMAKE_CXX_LINK_FLAGS> <LINK_FLAGS> -o <TARGET> <OBJECTS> <LINK_LIBRARIES> -Wl,--end-group"
        "${CMAKE_STRIP} --only-keep-debug <TARGET> -o ${REACTOS_BINARY_DIR}/symbols/${SYMBOL_FILE}"
        ${strip_debug})
    set(CMAKE_C_CREATE_SHARED_LIBRARY
        "<CMAKE_C_COMPILER> -Wl,--start-group <CMAKE_SHARED_LIBRARY_C_FLAGS> <LINK_FLAGS> <CMAKE_SHARED_LIBRARY_CREATE_C_FLAGS> -o <TARGET> <OBJECTS> <LINK_LIBRARIES> -Wl,--end-group"
        "${CMAKE_STRIP} --only-keep-debug <TARGET> -o ${REACTOS_BINARY_DIR}/symbols/${SYMBOL_FILE}"
        ${strip_debug})
    set(CMAKE_CXX_CREATE_SHARED_LIBRARY
        "<CMAKE_CXX_COMPILER> -Wl,--start-group <CMAKE_SHARED_LIBRARY_CXX_FLAGS> <LINK_FLAGS> <CMAKE_SHARED_LIBRARY_CREATE_CXX_FLAGS> -o <TARGET> <OBJECTS> <LINK_LIBRARIES> -Wl,--end-group"
        "${CMAKE_STRIP} --only-keep-debug <TARGET> -o ${REACTOS_BINARY_DIR}/symbols/${SYMBOL_FILE}"
        ${strip_debug})
    set(CMAKE_RC_CREATE_SHARED_LIBRARY
        "<CMAKE_C_COMPILER> -Wl,--start-group <CMAKE_SHARED_LIBRARY_C_FLAGS> <LINK_FLAGS> <CMAKE_SHARED_LIBRARY_CREATE_C_FLAGS> -o <TARGET> <OBJECTS> <LINK_LIBRARIES> -Wl,--end-group"
        "${CMAKE_STRIP} --only-keep-debug <TARGET> -o ${REACTOS_BINARY_DIR}/symbols/${SYMBOL_FILE}"
        ${strip_debug})
elseif(NO_ROSSYM)
    # Dwarf-based build
    message(STATUS "Generating a dwarf-based build (no rsym)")
    set(CMAKE_C_LINK_EXECUTABLE "<CMAKE_C_COMPILER> -Wl,--start-group ${CMAKE_C_FLAGS} <CMAKE_C_LINK_FLAGS> <LINK_FLAGS> -o <TARGET> <OBJECTS> <LINK_LIBRARIES> -Wl,--end-group")
    set(CMAKE_CXX_LINK_EXECUTABLE "<CMAKE_CXX_COMPILER> -Wl,--start-group ${CMAKE_CXX_FLAGS} <CMAKE_CXX_LINK_FLAGS> <LINK_FLAGS> -o <TARGET> <OBJECTS> <LINK_LIBRARIES> -Wl,--end-group")
    set(CMAKE_C_CREATE_SHARED_LIBRARY "<CMAKE_C_COMPILER> -Wl,--start-group ${CMAKE_C_FLAGS} <CMAKE_SHARED_LIBRARY_C_FLAGS> <LINK_FLAGS> <CMAKE_SHARED_LIBRARY_CREATE_C_FLAGS> -o <TARGET> <OBJECTS> <LINK_LIBRARIES> -Wl,--end-group")
    set(CMAKE_CXX_CREATE_SHARED_LIBRARY "<CMAKE_CXX_COMPILER> -Wl,--start-group ${CMAKE_CXX_FLAGS} <CMAKE_SHARED_LIBRARY_CXX_FLAGS> <LINK_FLAGS> <CMAKE_SHARED_LIBRARY_CREATE_CXX_FLAGS> -o <TARGET> <OBJECTS> <LINK_LIBRARIES> -Wl,--end-group")
    set(CMAKE_RC_CREATE_SHARED_LIBRARY "<CMAKE_C_COMPILER> -Wl,--start-group ${CMAKE_C_FLAGS} <CMAKE_SHARED_LIBRARY_C_FLAGS> <LINK_FLAGS> <CMAKE_SHARED_LIBRARY_CREATE_C_FLAGS> -o <TARGET> <OBJECTS> <LINK_LIBRARIES> -Wl,--end-group")
else()
    # Normal rsym build
    get_target_property(RSYM native-rsym IMPORTED_LOCATION)

    set(CMAKE_C_LINK_EXECUTABLE
        "<CMAKE_C_COMPILER> -Wl,--start-group ${CMAKE_C_FLAGS} <CMAKE_C_LINK_FLAGS> <LINK_FLAGS> -o <TARGET> <OBJECTS> <LINK_LIBRARIES> -Wl,--end-group"
        "${RSYM} -s ${REACTOS_SOURCE_DIR} <TARGET> <TARGET>")
    set(CMAKE_CXX_LINK_EXECUTABLE
        "<CMAKE_CXX_COMPILER> -Wl,--start-group ${CMAKE_CXX_FLAGS} <CMAKE_CXX_LINK_FLAGS> <LINK_FLAGS> -o <TARGET> <OBJECTS> <LINK_LIBRARIES> -Wl,--end-group"
        "${RSYM} -s ${REACTOS_SOURCE_DIR} <TARGET> <TARGET>")
    set(CMAKE_C_CREATE_SHARED_LIBRARY
        "<CMAKE_C_COMPILER> -Wl,--start-group ${CMAKE_C_FLAGS} <CMAKE_SHARED_LIBRARY_C_FLAGS> <LINK_FLAGS> <CMAKE_SHARED_LIBRARY_CREATE_C_FLAGS> -o <TARGET> <OBJECTS> <LINK_LIBRARIES> -Wl,--end-group"
        "${RSYM} -s ${REACTOS_SOURCE_DIR} <TARGET> <TARGET>")
    set(CMAKE_CXX_CREATE_SHARED_LIBRARY
        "<CMAKE_CXX_COMPILER> -Wl,--start-group ${CMAKE_CXX_FLAGS} <CMAKE_SHARED_LIBRARY_CXX_FLAGS> <LINK_FLAGS> <CMAKE_SHARED_LIBRARY_CREATE_CXX_FLAGS> -o <TARGET> <OBJECTS> <LINK_LIBRARIES> -Wl,--end-group"
        "${RSYM} -s ${REACTOS_SOURCE_DIR} <TARGET> <TARGET>")
    set(CMAKE_RC_CREATE_SHARED_LIBRARY
        "<CMAKE_C_COMPILER> -Wl,--start-group ${CMAKE_C_FLAGS} <CMAKE_SHARED_LIBRARY_C_FLAGS> <LINK_FLAGS> <CMAKE_SHARED_LIBRARY_CREATE_C_FLAGS> -o <TARGET> <OBJECTS> <LINK_LIBRARIES> -Wl,--end-group")
endif()

set(CMAKE_C_CREATE_SHARED_MODULE ${CMAKE_C_CREATE_SHARED_LIBRARY})
set(CMAKE_CXX_CREATE_SHARED_MODULE ${CMAKE_CXX_CREATE_SHARED_LIBRARY})
set(CMAKE_RC_CREATE_SHARED_MODULE ${CMAKE_RC_CREATE_SHARED_LIBRARY})

# Duplicate symbols: --allow-multiple-definition turns lld's duplicate ERROR into
# a warning, and tools/link_guard.py (reached through --ld-path) reads those
# warnings and fails the link on any duplicate that is not irreducible toolchain
# runtime or an import. The flag is load-bearing only for the runtime cases (each
# Rust staticlib carries its own panic runtime; compiler_builtins overlaps
# libclang_rt.builtins). Before the guard it silently let C glue beat translated
# Rust by link order: cdrom, fastfat, disk, crtdll, ucrtbase.
set(_vibeos_dup_policy "--ld-path=${REACTOS_SOURCE_DIR}/tools/link_guard/ld.lld")
# ... and the flag itself is per target: a module with the VIBEOS_STRICT_LINK property
# (link_strict(), below) links without it, so lld rejects ANY duplicate, toolchain runtime
# included. Such a module must be one Rust graph (one core / panic runtime /
# compiler_builtins) and also drops clang's libclang_rt.builtins (see the
# -print-libgcc-file-name block): lld fetches it for intrinsics that Rust's whole-archived
# compiler_builtins defines too, whenever a Rust member referencing one is parsed before the
# member defining it. As generator expressions because the property is set after
# set_module_type, and an unset property keeps every other module's link exactly as it was.
add_link_options("$<$<NOT:$<BOOL:$<TARGET_PROPERTY:VIBEOS_STRICT_LINK>>>:-Wl,--allow-multiple-definition>")

# link_strict(<target>): link <target> with no duplicate symbol tolerated (see above).
function(link_strict target)
    set_target_properties(${target} PROPERTIES VIBEOS_STRICT_LINK ON)
endfunction()
set(CMAKE_EXE_LINKER_FLAGS "${CMAKE_EXE_LINKER_FLAGS_INIT} -Wl,--disable-stdcall-fixup,--gc-sections ${_vibeos_dup_policy}")
set(CMAKE_SHARED_LINKER_FLAGS "${CMAKE_SHARED_LINKER_FLAGS_INIT} -Wl,--disable-stdcall-fixup ${_vibeos_dup_policy}")
set(CMAKE_MODULE_LINKER_FLAGS "${CMAKE_MODULE_LINKER_FLAGS_INIT} -Wl,--disable-stdcall-fixup ${_vibeos_dup_policy}")

set(CMAKE_C_COMPILE_OBJECT "<CMAKE_C_COMPILER> <DEFINES> ${_compress_debug_sections_flag} <INCLUDES> <FLAGS> -o <OBJECT> -c <SOURCE>")
# FIXME: Once the GCC toolchain bugs are fixed, add _compress_debug_sections_flag to CXX too
set(CMAKE_CXX_COMPILE_OBJECT "<CMAKE_CXX_COMPILER> <DEFINES> <INCLUDES> <FLAGS> -o <OBJECT> -c <SOURCE>")
set(CMAKE_ASM_COMPILE_OBJECT "<CMAKE_ASM_COMPILER> ${_compress_debug_sections_flag} -x assembler-with-cpp -o <OBJECT> -I${REACTOS_SOURCE_DIR}/sdk/include/asm -I${REACTOS_BINARY_DIR}/sdk/include/asm <INCLUDES> <FLAGS> <DEFINES> -D__ASM__ -c <SOURCE>")

# --codepage=65001: llvm-rc IGNORES `#pragma code_page`, so without a default every
# UTF-8 resource script was decoded as cp1252 -- "fran\xc3\xa7ais" for "français" in
# every translated string table, and kernel32's French LOCALE_STHOUSAND came out as
# U+00C2 U+00A0, which VarParseNumFromStr then took as the separator. The tree's
# resource scripts are all UTF-8 (the six that declare 1251/1252/932 are pure ASCII),
# so UTF-8 is the one right default, and llvm-rc rejects invalid UTF-8 loudly.
set(RC_CODEPAGE_FLAG "--codepage=65001")
set(CMAKE_RC_COMPILE_OBJECT "<CMAKE_RC_COMPILER> -O coff ${RC_CODEPAGE_FLAG} <INCLUDES> <FLAGS> -DRC_INVOKED -D__WIN32__=1 -D__FLAT__=1 ${I18N_DEFS} <DEFINES> <SOURCE> <OBJECT>")

# use_gnu_rc() — compile THIS DIRECTORY's .rc files with GNU windres instead of
# llvm-mingw's (which is llvm-rc in a trench coat). Only for the handful of
# resource scripts that use syntax llvm-rc has never implemented; everything
# else stays on the pinned toolchain, so this is an escape hatch and not a
# second resource pipeline.
#
# It works by overriding the rule variable rather than CMAKE_RC_COMPILER,
# because CMake expands <CMAKE_RC_COMPILER> from the global value and would
# ignore a directory-scoped one. Flags, includes, defines and the depfile
# plumbing are IDENTICAL to the rule above -- the compiler is the only change --
# and the preprocessor is still the pinned clang, so the preprocessed text the
# two compilers see is the same text.
#
# Missing GNU windres is reported and the caller decides; it must not silently
# fall back to llvm-rc, which would "succeed" at configure time and then fail
# the build with a parse error that names the resource script instead of the
# missing host tool.
function(use_gnu_rc)
    if(NOT GNU_RC_COMPILER)
        message(WARNING
            "GNU windres not found: resource scripts in ${CMAKE_CURRENT_SOURCE_DIR} "
            "cannot be built (llvm-rc does not implement the syntax they use). "
            "Install mingw-w64 binutils (Debian/Ubuntu: binutils-mingw-w64-x86-64; "
            "Homebrew: mingw-w64) or pass -DGNU_RC_COMPILER=/path/to/windres.")
        return()
    endif()
    set(CMAKE_RC_COMPILE_OBJECT
        "${GNU_RC_COMPILER} -O coff ${RC_CODEPAGE_FLAG} <INCLUDES> <FLAGS> -DRC_INVOKED -D__WIN32__=1 -D__FLAT__=1 ${I18N_DEFS} <DEFINES> <SOURCE> <OBJECT>"
        PARENT_SCOPE)
endfunction()

if (CMAKE_C_COMPILER_ID STREQUAL "Clang")
    set(RC_PREPROCESSOR_TARGET "--preprocessor-arg=--target=${CMAKE_C_COMPILER_TARGET}")
else()
    set(RC_PREPROCESSOR_TARGET "")
endif()

# We have to pass args to windres. one... by... one...
# The placeholder is <DEP_FILE>. It was spelled <DEPFILE>, which CMake leaves as literal text:
# every resource compile wrote its depfile to one file named DEPFILE in the build root, ninja
# found none at $DEP_FILE, and no .res ever tracked its includes -- a regenerated message
# table (errcodes.rc) or an edited dialog header left the old resources linked, warm.
set(CMAKE_DEPFILE_FLAGS_RC "--preprocessor=\"${CMAKE_C_COMPILER}\" ${RC_PREPROCESSOR_TARGET} --preprocessor-arg=-E --preprocessor-arg=-nostdinc --preprocessor-arg=-xc-header --preprocessor-arg=-MMD --preprocessor-arg=-MF --preprocessor-arg=<DEP_FILE> --preprocessor-arg=-MT --preprocessor-arg=<OBJECT>")

# Optional 3rd parameter: stdcall stack bytes
function(set_entrypoint MODULE ENTRYPOINT)
    # Which Rust CRT-startup personality msvcrtex links into this module
    # (sdk/lib/crt/msvcrtex.cmake). Derived from the entry symbol itself, so the last
    # set_entrypoint wins exactly as the linker's -entry does.
    if(ENTRYPOINT STREQUAL "DllMainCRTStartup")
        set_property(TARGET ${MODULE} PROPERTY REACTOS_CRT_KIND dll)
    elseif(ENTRYPOINT MATCHES "^(mainCRTStartup|WinMainCRTStartup)$")
        set_property(TARGET ${MODULE} PROPERTY REACTOS_CRT_KIND exe)
    elseif(ENTRYPOINT MATCHES "^(wmainCRTStartup|wWinMainCRTStartup)$")
        set_property(TARGET ${MODULE} PROPERTY REACTOS_CRT_KIND wexe)
    else()
        set_property(TARGET ${MODULE} PROPERTY REACTOS_CRT_KIND bare)
    endif()
    if(${ENTRYPOINT} STREQUAL "0")
        # lld interprets "0" as a symbol name, not an address
        # Use __ImageBase as a dummy entry point for modules that don't need one
        if(CMAKE_C_COMPILER_ID STREQUAL "Clang")
            if(ARCH STREQUAL "i386")
                # i386 adds _ prefix: __ImageBase becomes ___ImageBase
                target_link_options(${MODULE} PRIVATE "-Wl,-e,___ImageBase")
            else()
                target_link_options(${MODULE} PRIVATE "-Wl,-e,__ImageBase")
            endif()
        else()
            target_link_options(${MODULE} PRIVATE "-Wl,-entry,0")
        endif()
    elseif(ARCH STREQUAL "i386")
        set(_entrysymbol _${ENTRYPOINT})
        if(${ARGC} GREATER 2)
            set(_entrysymbol ${_entrysymbol}@${ARGV2})
        endif()
        target_link_options(${MODULE} PRIVATE "-Wl,-entry,${_entrysymbol}")
    else()
        target_link_options(${MODULE} PRIVATE "-Wl,-entry,${ENTRYPOINT}")
    endif()
endfunction()

function(set_subsystem MODULE SUBSYSTEM)
    # LLD doesn't recognize numeric subsystem IDs; map EFI types to names
    if(SUBSYSTEM STREQUAL "10")
        set(SUBSYSTEM "efi_application")
    elseif(SUBSYSTEM STREQUAL "11")
        set(SUBSYSTEM "efi_boot_service_driver")
    elseif(SUBSYSTEM STREQUAL "12")
        set(SUBSYSTEM "efi_runtime_driver")
    elseif(SUBSYSTEM STREQUAL "13")
        set(SUBSYSTEM "efi_rom")
    endif()
    target_link_options(${MODULE} PRIVATE "-Wl,--subsystem,${SUBSYSTEM}:5.01")
endfunction()

function(set_image_base MODULE IMAGE_BASE)
    target_link_options(${MODULE} PRIVATE "-Wl,--image-base,${IMAGE_BASE}")
endfunction()

function(set_module_type_toolchain MODULE TYPE)
    # Set the PE image version numbers from the NT OS version ReactOS is based on
    target_link_options(${MODULE} PRIVATE
        -Wl,--major-image-version,5 -Wl,--minor-image-version,01 -Wl,--major-os-version,5 -Wl,--minor-os-version,01)

    if(TYPE IN_LIST KERNEL_MODULE_TYPES)
        # "kmdfdriver" is treated the same way as "wdmdriver" in toolchain-specific set_module_type
        if(${TYPE} STREQUAL "kmdfdriver")
            set(TYPE "wdmdriver")
        endif()

        target_link_options(${MODULE} PRIVATE -Wl,--exclude-all-symbols,-file-alignment=0x1000,-section-alignment=0x1000)

        if(${TYPE} STREQUAL "wdmdriver")
            # --wdmdriver is a GNU ld custom flag, not supported by lld
            if(NOT CMAKE_C_COMPILER_ID STREQUAL "Clang")
                target_link_options(${MODULE} PRIVATE "-Wl,--wdmdriver")
            endif()
        endif()

        # Place INIT &.rsrc section at the tail of the module, before .reloc
        add_linker_script(${MODULE} ${REACTOS_SOURCE_DIR}/sdk/cmake/init-section.lds)

        # Fixup section characteristics
        #  - Remove flags that LD overzealously puts (alignment flag, Initialized flags for code sections)
        #  - INIT section is made discardable
        #  - .rsrc is made read-only and discardable
        #  - PAGE & .edata sections are made pageable.
        add_custom_command(TARGET ${MODULE} POST_BUILD
            COMMAND native-pefixup --${TYPE} $<TARGET_FILE:${MODULE}>)

        # Believe it or not, cmake doesn't do that
        set_property(TARGET ${MODULE} APPEND PROPERTY LINK_DEPENDS $<TARGET_PROPERTY:native-pefixup,IMPORTED_LOCATION>)
    endif()

    # Fix lld i386 forwarding string underscore bug.
    # lld adds '_' prefix to DLL forwarding targets (e.g. ntdll.Xxx -> _ntdll.Xxx).
    # This post-build step patches the PE export table to skip the underscore.
    if(USE_LLVM_MINGW AND ARCH STREQUAL "i386")
        add_custom_command(TARGET ${MODULE} POST_BUILD
            COMMAND ${CMAKE_COMMAND} -E env python3
                ${REACTOS_SOURCE_DIR}/sdk/tools/fix_lld_forwarding.py
                $<TARGET_FILE:${MODULE}>)
    endif()
endfunction()

function(add_delay_importlibs _module)
    get_target_property(_module_type ${_module} TYPE)
    if(_module_type STREQUAL "STATIC_LIBRARY")
        message(FATAL_ERROR "Cannot add delay imports to a static library")
    endif()
    foreach(_lib ${ARGN})
        get_filename_component(_basename "${_lib}" NAME_WE)
        # llvm-dlltool lacks --output-delaylib; when VIBEOS_ENABLE_DELAYLOAD is set
        # we generate a real delay-import lib ourselves (sdk/tools/gen-delaylib.py,
        # used by generate_import_lib below), so delay-load works on all llvm-mingw
        # arches. That is the default; the old behaviour (empty _delayed.a +
        # regular imports) is kept behind VIBEOS_NO_DELAYLOAD. See
        # docs/engineering/archive/delay-import-llvm-mingw-design.md.
        if(CMAKE_DLLTOOL_MACHINE AND (VIBEOS_NO_DELAYLOAD OR NOT VIBEOS_ENABLE_DELAYLOAD))
            target_link_libraries(${_module} lib${_basename})
        else()
            target_link_libraries(${_module} lib${_basename}_delayed)
        endif()
    endforeach()
    # lld leaves the DELAY_IMPORT data directory empty for gen-delaylib's descriptors;
    # point it at their table (gen-delaylib.py brackets it) once the module is linked.
    if(CMAKE_DLLTOOL_MACHINE AND VIBEOS_ENABLE_DELAYLOAD AND NOT VIBEOS_NO_DELAYLOAD)
        get_target_property(_has_fixup ${_module} VIBEOS_DELAYIMPORT_FIXUP)
        if(NOT _has_fixup)
            set_target_properties(${_module} PROPERTIES VIBEOS_DELAYIMPORT_FIXUP TRUE)
            add_custom_command(TARGET ${_module} POST_BUILD
                COMMAND native-pefixup --delayimport "$<TARGET_FILE:${_module}>"
                COMMENT "Setting the DELAY_IMPORT directory of ${_module}"
                VERBATIM)
            set_property(TARGET ${_module} APPEND PROPERTY
                LINK_DEPENDS $<TARGET_PROPERTY:native-pefixup,IMPORTED_LOCATION>)
        endif()
    endif()
    # Link the delay-load runtime unconditionally — not just when delay-loading.
    # Modules may reference the __pfnDli*Hook2 variables directly (delayimp.h
    # declares them; dbghelp_apitest's pdb.c assigns the failure hook), and those
    # data symbols are defined in delayimp. On the no-delayload fallback path the
    # helper is never called and static-lib semantics pull in only the referenced
    # hook-variable objects, so this costs nothing there.
    target_link_libraries(${_module} delayimp)
endfunction()

if(NOT ARCH STREQUAL "i386")
    set(DECO_OPTION "-@")
endif()

function(fixup_load_config _target)
    add_custom_command(TARGET ${_target} POST_BUILD
        COMMAND native-pefixup --loadconfig "$<TARGET_FILE:${_target}>"
        COMMENT "Patching in LOAD_CONFIG"
        DEPENDS native-pefixup)
endfunction()

# Transitive closure of a crate's Cargo `path = "..."` dependencies, as absolute
# directories. Used to build the rebuild-trigger glob for a cargo module: the
# crates a module's bodies reach are part of its source, and watching only the
# declared one leaves a silently stale binary (see the call site).
#
# Iterative rather than recursive because the graph has diamonds (several kernel
# crates share panic-common) and cmake has no memoized recursion; `_seen` also
# makes a dependency cycle terminate instead of hanging the configure.
function(_rust_path_dep_closure _ws _names _out)
    set(_pending "")
    foreach(_n IN LISTS _names)
        list(APPEND _pending "${_ws}/${_n}")
    endforeach()
    set(_seen "")
    while(_pending)
        list(POP_FRONT _pending _dir)
        get_filename_component(_dir "${_dir}" ABSOLUTE)
        # list(FIND) rather than IN_LIST: the latter needs policy CMP0057 NEW,
        # which holds in the real configure but not when this file is read in
        # script mode, and a helper that only works in one of those is a trap.
        list(FIND _seen "${_dir}" _idx)
        if(NOT _idx EQUAL -1)
            continue()
        endif()
        if(NOT EXISTS "${_dir}/Cargo.toml")
            # A path dep pointing outside the workspace, or a crate not vendored
            # here. Nothing to glob; skipping beats failing the configure.
            continue()
        endif()
        list(APPEND _seen "${_dir}")
        file(READ "${_dir}/Cargo.toml" _toml)
        string(REGEX MATCHALL "path[ \t]*=[ \t]*\"[^\"]+\"" _hits "${_toml}")
        foreach(_h IN LISTS _hits)
            string(REGEX REPLACE "^path[ \t]*=[ \t]*\"(.*)\"$" "\\1" _p "${_h}")
            get_filename_component(_abs "${_p}" ABSOLUTE BASE_DIR "${_dir}")
            list(APPEND _pending "${_abs}")
        endforeach()
    endwhile()
    set(${_out} "${_seen}" PARENT_SCOPE)
endfunction()

## Drive a PE module's FINAL LINK from cargo (the sdk/lib/rust/<crate>
## reactos-link manifest) instead of cmake's own add_library/add_executable.
## The "turn off the cmake link" step: cargo produces the shipping binary;
## cmake keeps only what the rest of the tree needs (import lib via
## spec2def(... ADD_IMPORTLIB), header-dependent objects via an OBJECT lib,
## boot-media registration, and the kind-specific post-link).
##
##   add_cargo_module(<target> <crate> <output>
##                    [KIND kerneldll|efi|native]      (default kerneldll)
##                    [BODY_CRATE <rust-staticlib-crate>]
##                    [BODY_CRATES <rust-crate> ...]
##                    [EXTRA_DEPENDS <targets/files…>]  (e.g. the OBJECT lib
##                                                       whose .obj the link
##                                                       reuses)
##        NOTE: the cargo link is opaque to cmake, so EXTRA_DEPENDS is the ONLY
##        thing wiring its inputs into the ninja graph. An add_library/OBJECT
##        TARGET here auto-tracks its output FILE (relinks on change). But a
##        custom-TARGET (e.g. rust_freeldr_uefi, rust_cnt — the rust .a's built
##        by add_custom_command) gives only an ORDER-ONLY edge: ninja will NOT
##        relink when its .a content changes, silently shipping a stale module.
##        For those, ALSO pass the .a FILE PATH (the custom_command OUTPUT) so
##        ninja file-tracks it. reactos-link emits cargo:rerun-if-changed per
##        static_lib, so once ninja re-invokes cargo the link actually re-runs.
##                    [NO_CD]                           (skip add_cd_file —
##                                                       e.g. uefildr, which
##                                                       boot_images embeds)
##                    [DESTINATION <cd-dir>] [NAME_ON_CD <name>])
##
## <crate> is the cargo bin crate whose build.rs uses reactos-link and is
## arch-aware (i686/x86_64/aarch64-kernel.json under the crate dir); its
## [[bin]] name must be <crate> minus the `-cargo` suffix. The cargo build
## uses nightly + build-std (the custom kernel json), so the build host needs
## the nightly toolchain — inherent to the cargo-drive path.
##
## KIND selects the post-link (matching what cmake's set_module_type /
## per-target POST_BUILD would have applied):
##   kerneldll : native-pefixup --kerneldll  (+ i386 forwarder fix)
##   kernelmodedriver : native-pefixup --kernelmodedriver (+ i386 forwarder fix)  [win32k]
##   win32dll  : i386 lld forwarder fix only  (user DLL — no pefixup)  [ntdll]
##   efi       : strip --remove-section=.rossym  +  strip --strip-all
##   native    : none
include(${CMAKE_CURRENT_LIST_DIR}/cargo_link_context.cmake)

function(add_cargo_module _target _crate _output)
    cmake_parse_arguments(_acm "NO_CD" "KIND;DESTINATION;NAME_ON_CD;BODY_CRATE" "BODY_CRATES;EXTRA_DEPENDS" ${ARGN})
    if(NOT _acm_KIND)
        set(_acm_KIND kerneldll)
    endif()

    set(_ws "${CMAKE_SOURCE_DIR}/sdk/lib/rust")

    # ARCH → the rust target (== cargo's output subdir stem).
    #
    # amd64 + arm64 build the BUILTIN triple on pinned STABLE: precompiled
    # core/alloc/compiler_builtins from the rustup `rust-std-*` component, NO
    # `-Z build-std`, NO custom target json (stable-toolchain-plan.md, Axis B).
    # The old custom `{x86_64,aarch64}-kernel.json` were the builtin spec minus
    # the user-mode CRT link plumbing; the plumbing is neutralised at LINK time
    # instead (reactos-link: stub archives + `-C link-self-contained=no`), and
    # the handful of non-default codegen opts the json carried (frame-pointer,
    # function-sections, panic) are restored as explicit `-C` flags. Proven
    # ABI-identical: `abi-return-struct-as-int` — the only spec field that flips
    # — is read solely by the 32-bit x86 callconv, so it is inert here.
    #
    # i386 stays on the custom json + nightly build-std: it needs
    # `rustc-abi: x86-softfloat`, which has no builtin triple. That cell is
    # retired by the wow64 lane (i386 kernel goes away), not fixed here. NOTE that
    # is necessary, not sufficient: this branch has no KIND test, so user-mode
    # cells take the kernel json too (ntdll is KIND win32dll), and SysWOW64 stages
    # i386 user-mode PEs for as long as we run x86 apps. Splitting this by KIND is
    # what would actually shrink the nightly surface -- untested, see
    # docs/engineering/active-lanes/x64-primary-plan.md item 4.
    if(ARCH STREQUAL "i386")
        set(_stem "i686-kernel")
        set(_rust_target "${_ws}/${_crate}/${_stem}.json")
        set(_build_std -Z build-std=core,alloc,compiler_builtins
                       -Z build-std-features=compiler-builtins-mem)
    elseif(ARCH STREQUAL "amd64")
        set(_stem "x86_64-pc-windows-gnu")
        set(_rust_target "${_stem}")
        set(_build_std "")
    elseif(ARCH STREQUAL "arm64")
        set(_stem "aarch64-pc-windows-gnullvm")
        set(_rust_target "${_stem}")
        set(_build_std "")
    else()
        message(FATAL_ERROR "add_cargo_module: unsupported ARCH ${ARCH}")
    endif()

    if(NOT ARCH STREQUAL "i386")
        # The precompiled `core` calls memcmp (core::slice::cmp) — `-Z build-std`
        # used to supply it via `compiler-builtins-mem`. reactos-link links
        # sdk/lib/crt/libmemcmp.a as a link-group FALLBACK on the builtin targets,
        # so every cell needs it BUILT first (it panics loudly if it's missing).
        list(APPEND _acm_EXTRA_DEPENDS memcmp)
    endif()

    # Determinism rung 1 (Rust paths): handled by the CANONICAL BUILD PATH now —
    # every worktree builds bind-mounted at /tmp/vibeos (tools/canon-run.sh), so
    # rustc-embedded paths are byte-identical across worktrees with NO remap flag.
    # The old per-worktree `--config det-remap.toml` (--remap-path-prefix) was
    # REMOVED 2026-07-03: sccache 0.8.2 marks any rustc invocation carrying
    # --remap-path-prefix as non-cacheable, which silently disabled ALL Rust
    # build caching. See docs/engineering/build-cache-and-determinism-plan.md.
    string(REGEX REPLACE "-cargo$" "" _bin "${_crate}")
    reactos_cargo_link_context("${_ws}" "${_crate}" "${ARCH}" _cargo_target_dir)
    set(_cargo_exe "${_cargo_target_dir}/${_stem}/release/${_bin}.exe")
    set(_out "${CMAKE_CURRENT_BINARY_DIR}/${_output}")

    # sync-instrumentation §4 lockdep (Tier-2 opt-in): -DVIBEOS_LOCKDEP=ON lights
    # up the self-deadlock detector. Plumbed as ENV to the kernel modules' cargo
    # drive — their build.rs (ntoskrnl-cargo/ntkrnlmp-cargo) reads it and adds
    # `--features lockdep` to the INNER `cargo build -p kernel-rust` (the body
    # crate is not a cargo dep of the bin crate, so an outer --features can't
    # reach it). Confined to the kernel — other modules' build.rs ignore the
    # env. Default off = zero cost.
    set(_cargo_env "CARGO_TARGET_DIR=${_cargo_target_dir}" "CARGO_BUILD_JOBS=1")
    if(VIBEOS_LOCKDEP AND _crate MATCHES "^(ntoskrnl-cargo|ntkrnlmp-cargo)$")
        list(APPEND _cargo_env VIBEOS_LOCKDEP=1)
    endif()
    # cswitchdep (cswitchdep-design.md): -DVIBEOS_CSWITCHDEP=ON arms the
    # context-switch hand-off verifier (same env→build.rs→--features plumbing;
    # the cargo feature implies lockdep so the held-set mirror is present).
    if(VIBEOS_CSWITCHDEP AND _crate MATCHES "^(ntoskrnl-cargo|ntkrnlmp-cargo)$")
        list(APPEND _cargo_env VIBEOS_CSWITCHDEP=1)
    endif()
    # (The former VIBEOS_SMP_DEFAULT_BOOT → VIBEOS_SMP_HAL=1 env, which rebuilt
    # hal.dll with hal-rust's `smp` feature, is RETIRED. The SMP HAL is now its
    # own cell — halmp-cargo → halmp.dll, built unconditionally and shipped
    # alongside hal.dll, selected at boot by livecd.ini's `/HAL=halmp.dll`.
    # Two reasons it had to go: switching flavors through a build-dir-global
    # cache var cost a reconfigure + full HAL relink (9m25s i386 / 12m22s amd64
    # warm), which is why there was never an MP gate leg; and a `hal.dll` that
    # is secretly an MP HAL passes every name-based check there is. The flavor
    # is now the file name.)
    # i386 legacy PIC HAL escape hatch (-DVIBEOS_PIC_HAL=ON, `vibeos-cli build
    # --hal-pic`): build hal.dll with the 8259 PIC flavor instead of the DEFAULT
    # APIC flavor (LAPIC/IOAPIC/TPR). i386 only — amd64 hal is always APIC. A
    # plain build (no env) is the APIC HAL; hal-link treats the ABSENCE of
    # VIBEOS_PIC_HAL as "build APIC". BOTH HAL cells take it — the interrupt
    # model is orthogonal to the UP/MP flavor, and letting the twins disagree
    # about it would ship an APIC halmp.dll against a PIC hal.dll.
    if(VIBEOS_PIC_HAL AND _crate MATCHES "^(hal-cargo|halmp-cargo)$")
        list(APPEND _cargo_env VIBEOS_PIC_HAL=1)
    endif()

    # Rebuild triggers: glob the bin crate (build.rs + target jsons + Cargo.toml
    # + any src), the shared reactos-link engine, and the rust body crate. A
    # cargo build is otherwise opaque to cmake; an incomplete DEPENDS lets a
    # stale binary mask a source fix (the libcntpr.cmake lesson).
    file(GLOB_RECURSE _srcs CONFIGURE_DEPENDS
        ${_ws}/${_crate}/*.rs ${_ws}/${_crate}/*.json ${_ws}/${_crate}/Cargo.toml
        ${_ws}/reactos-link/src/*.rs)
    if(_acm_BODY_CRATE)
        list(APPEND _acm_BODY_CRATES ${_acm_BODY_CRATE})
    endif()
    # reactos-link's msvcrtex_crt() builds the Rust CRT floor (crt-startup-rust,
    # crt-dllmain-rust, crt0-rust) as nested libs. Cargo's depfile does not carry
    # them, so editing crtdll.rs left lsass/winlogon linked against the old CRT
    # (`build` reported them stale, ninja had no edge to rerun). Watch them for
    # every module whose build.rs asks for that CRT; the build.rs is the truth.
    file(READ "${_ws}/${_crate}/build.rs" _acm_buildrs)
    set_property(DIRECTORY APPEND PROPERTY CMAKE_CONFIGURE_DEPENDS "${_ws}/${_crate}/build.rs")
    string(FIND "${_acm_buildrs}" "msvcrtex_crt" _acm_uses_crt)
    if(NOT _acm_uses_crt EQUAL -1)
        list(APPEND _acm_BODY_CRATES crt-startup-rust crt-dllmain-rust crt0-rust)
    endif()
    # Follow Cargo path dependencies TRANSITIVELY, not just the crates named in
    # BODY_CRATES. A declaration names the crate that holds the module's bodies,
    # but those bodies call into sibling crates by path, and editing one of THOSE
    # produced no cmake dependency at all: ninja saw no reason to relink, `build`
    # reported the module stale forever, and `ninja <target>` said "no work to
    # do". The only remedy was deleting the output — which works precisely
    # BECAUSE the edge is missing. Concretely: ntkrnlmp declares
    # BODY_CRATE kernel-rust, and ke/census.rs lives in ntoskrnl-rust, a path
    # dependency of it.
    #
    # This does NOT become "watch the whole workspace", which was rejected here
    # for relinking unrelated DLLs after every kernel edit. Closures are narrow
    # and well separated: of 314 workspace crates, kernel-rust reaches 33,
    # win32k-rust 5, shell32-rust 4, ntdll-rust 3. A module rebuilds when
    # something it actually links changes, which is the whole point.
    # User-mode modules also link the CRT startup crates, which reactos-link's
    # .msvcrtex_crt() builds in a NESTED cargo invocation: Cargo's depfile never
    # sees them, so without this an edit to crt-startup-rust relinked no cargo
    # module and `build` reported lsass/services stale (2026-10-04).
    set(_closure_roots ${_acm_BODY_CRATES})
    if(_acm_KIND MATCHES "^win32")
        list(APPEND _closure_roots crt-startup-rust crt0-rust crt-dllmain-rust)
    endif()
    _rust_path_dep_closure("${_ws}" "${_closure_roots}" _body_dirs)
    foreach(_body_dir IN LISTS _body_dirs)
        file(GLOB_RECURSE _bsrcs CONFIGURE_DEPENDS
            ${_body_dir}/*.rs ${_body_dir}/Cargo.toml)
        list(APPEND _srcs ${_bsrcs})
    endforeach()
    # Bootstrap inputs; after the first link, Cargo's depfile carries the complete
    # compiler-discovered closure, including nested body builds and native headers.
    # Watching the entire workspace here AND in build.rs relinked unrelated DLLs
    # after every kernel edit. Keep graph/config changes explicit.
    list(APPEND _srcs "${_ws}/Cargo.toml" "${_ws}/Cargo.lock"
        "${_ws}/.cargo/config.toml" "${REACTOS_SOURCE_DIR}/rust-toolchain.toml"
        "${REACTOS_SOURCE_DIR}/.rust-nightly")

    # Kind-specific post-link (cmake passes these as the target's POST_BUILD).
    set(_postlink)
    set(_postdep)
    if(_acm_KIND STREQUAL "kerneldll")
        set(_postlink COMMAND $<TARGET_FILE:native-pefixup> --kerneldll ${_out})
        set(_postdep native-pefixup)
        # i386-only forwarder fix mirrors set_module_type's i386-only POST_BUILD.
        if(USE_LLVM_MINGW AND ARCH STREQUAL "i386")
            list(APPEND _postlink COMMAND ${CMAKE_COMMAND} -E env python3
                ${REACTOS_SOURCE_DIR}/sdk/tools/fix_lld_forwarding.py ${_out})
        endif()
    elseif(_acm_KIND STREQUAL "kernelmodedriver")
        # Kernel-mode driver .sys (win32k.sys): set_module_type(kernelmodedriver)
        # applies native-pefixup --kernelmodedriver (same driver_fixup as
        # --kerneldll but the kernelmodedriver mode flag) + the i386 fwd fix.
        set(_postlink COMMAND $<TARGET_FILE:native-pefixup> --kernelmodedriver ${_out})
        set(_postdep native-pefixup)
        if(USE_LLVM_MINGW AND ARCH STREQUAL "i386")
            list(APPEND _postlink COMMAND ${CMAKE_COMMAND} -E env python3
                ${REACTOS_SOURCE_DIR}/sdk/tools/fix_lld_forwarding.py ${_out})
        endif()
    elseif(_acm_KIND STREQUAL "kernelexe")
        # Kernel EXE (ntoskrnl.exe / ntkrnlmp.exe): set_module_type(kernel) applies
        # native-pefixup --kernel (NOT --kerneldll) + the i386 lld forwarder fix.
        # Exactly the post-link ntoskrnl-cargo/boot-swap-smoke.sh applies (×3 boot).
        set(_postlink COMMAND $<TARGET_FILE:native-pefixup> --kernel ${_out})
        set(_postdep native-pefixup)
        if(USE_LLVM_MINGW AND ARCH STREQUAL "i386")
            list(APPEND _postlink COMMAND ${CMAKE_COMMAND} -E env python3
                ${REACTOS_SOURCE_DIR}/sdk/tools/fix_lld_forwarding.py ${_out})
        endif()
    elseif(_acm_KIND STREQUAL "win32dll")
        # User-mode DLL (ntdll): NOT a KERNEL_MODULE_TYPE, so set_module_type
        # applies NO native-pefixup (that's gated to KERNEL_MODULE_TYPES). The
        # ONLY post-link a win32dll gets is the i386 lld forwarder-underscore
        # fix, which set_module_type applies to every PE module on i386/llvm-mingw.
        if(USE_LLVM_MINGW AND ARCH STREQUAL "i386")
            set(_postlink COMMAND ${CMAKE_COMMAND} -E env python3
                ${REACTOS_SOURCE_DIR}/sdk/tools/fix_lld_forwarding.py ${_out})
        endif()
    elseif(_acm_KIND STREQUAL "win32exe")
        # User-mode win32 exe (lsass.exe): like win32dll, NOT a KERNEL_MODULE_TYPE
        # so NO native-pefixup; the only post-link set_module_type applies on
        # i386/llvm-mingw is the lld forwarder-underscore fix (a no-op for an
        # export-less exe, kept for parity with every PE module).
        if(USE_LLVM_MINGW AND ARCH STREQUAL "i386")
            set(_postlink COMMAND ${CMAKE_COMMAND} -E env python3
                ${REACTOS_SOURCE_DIR}/sdk/tools/fix_lld_forwarding.py ${_out})
        endif()
    elseif(_acm_KIND STREQUAL "efi" OR _acm_KIND STREQUAL "biosloader")
        # Strip rossym + all (matches uefi.cmake's uefildr POST_BUILD, and the
        # old pcat.cmake freeldr_pe POST_BUILD for the BIOS loader).
        set(_postlink
            COMMAND ${CMAKE_STRIP} --remove-section=.rossym ${_out}
            COMMAND ${CMAKE_STRIP} --strip-all ${_out})
    elseif(_acm_KIND STREQUAL "native")
        # No post-link (rosload's cmake POST_BUILD was a no-op).
    else()
        message(FATAL_ERROR "add_cargo_module: unknown KIND ${_acm_KIND}")
    endif()

    # i386 ONLY: the pinned nightly, single-sourced from `.rust-nightly` at the repo
    # root (bump it THERE, not here). `-Z build-std` + the custom `x86-softfloat`
    # kernel json need nightly; amd64/arm64 are on pinned STABLE via
    # rust-toolchain.toml (no RUSTUP_TOOLCHAIN override → the toolchain file wins).
    # See docs/engineering/stable-toolchain-plan.md.
    set(_toolchain_env "")
    if(ARCH STREQUAL "i386")
        file(STRINGS "${CMAKE_SOURCE_DIR}/.rust-nightly" _rust_nightly LIMIT_COUNT 1)
        set(_toolchain_env RUSTUP_TOOLCHAIN=${_rust_nightly})
    else()
        # Builtin-triple (stable) cells: restore what the custom kernel json used to
        # carry and neutralise what it used to strip.
        #   force-frame-pointers=yes : the json said `frame-pointer: always`; the
        #       builtin triples default to may-omit (x86_64) / non-leaf (aarch64).
        #       The kernel debug toolkit walks the frame chain (inspect.sh bt).
        #   link-self-contained=no (amd64 ONLY) : x86_64-pc-windows-gnu declares
        #       `crt-objects-fallback: mingw`, so without this rustc prepends the
        #       user-mode CRT startup objects (crt2.o / dllcrt2.o) to a kernel module.
        #       Objects can't be shadowed by a stub archive, so this is the only lever.
        #       aarch64-pc-windows-gnullvm does NOT allow the flag ("not supported on
        #       this target" — its self-contained mode is fixed, and it links no CRT
        #       objects), and passing it is a hard rustc error at bin-link time.
        #       (The CRT *libraries* both triples name in late-link-args — -lmsvcrt
        #       -lgcc -lkernel32 … — are walled off by reactos-link's stub archives.)
        # (The linker itself is pinned to llvm-mingw's clang in
        # sdk/lib/rust/.cargo/config.toml — a `linker` key, so it COMPOSES with this
        # env RUSTFLAGS instead of being clobbered by it. Keep this in sync with
        # tests/qemu/cli/lints.py, tools/cargo_orchestrate.py and
        # sdk/lib/rust/cargo-cell-args.sh, which re-run the same cargo links.)
        set(_rustflags "-C force-frame-pointers=yes")
        if(ARCH STREQUAL "amd64")
            set(_rustflags "${_rustflags} -C link-self-contained=no")
        endif()
        set(_toolchain_env "RUSTFLAGS=${_rustflags}")
    endif()

    # Cargo exports native and nested-Rust inputs through the outer depfile.
    set(_cargo_depfile "${_cargo_target_dir}/${_stem}/release/${_bin}.d")
    set(_retarget_depfile "${REACTOS_SOURCE_DIR}/tools/retarget_cargo_depfile.py")
    set(_dep_target "${_out}")
    if(POLICY CMP0116)
        cmake_policy(PUSH)
        cmake_policy(SET CMP0116 NEW)
    else()
        # CMake 3.17 passes DEPFILE through unchanged: Ninja expects its own
        # build-root-relative output spelling, not Cargo's absolute output.
        file(RELATIVE_PATH _dep_target "${CMAKE_BINARY_DIR}" "${_out}")
    endif()
    add_custom_command(
        OUTPUT ${_out}
        # cargo drive of the reactos-link manifest: build.rs links the cmake-built
        # leaf libs / reused objects + self-built rust staticlib into the final PE,
        # then the kind post-link runs. amd64/arm64: stable + builtin triple.
        # i386: nightly + build-std + the softfloat kernel json.
        COMMAND ${CMAKE_COMMAND} -E env ${_toolchain_env} ${_cargo_env}
            cargo build --release
                --target ${_rust_target}
                ${_build_std}
                -p ${_crate} --bin ${_bin}
                --manifest-path ${_ws}/Cargo.toml
        COMMAND ${CMAKE_COMMAND} -E copy ${_cargo_exe} ${_out}
        COMMAND ${CMAKE_COMMAND} -E env python3 "${_retarget_depfile}"
            --source "${_cargo_depfile}" --cargo-output "${_cargo_exe}"
            --target "${_dep_target}" --output "${_out}.d"
        ${_postlink}
        DEPENDS ${_srcs} ${_postdep} ${_acm_EXTRA_DEPENDS} "${_retarget_depfile}"
        DEPFILE "${_out}.d"
        JOB_POOL vibeos_cargo_links
        WORKING_DIRECTORY ${_ws}
        COMMENT "Cargo-linking ${_output} (${_stem}) [reactos-link]"
        VERBATIM)
    if(POLICY CMP0116)
        cmake_policy(POP)
    endif()
    add_custom_target(${_target} ALL DEPENDS ${_out})

    if(NOT _acm_NO_CD)
        if(NOT _acm_DESTINATION)
            set(_acm_DESTINATION reactos/system32)
        endif()
        if(NOT _acm_NAME_ON_CD)
            set(_acm_NAME_ON_CD ${_output})
        endif()
        # add_cd_file's FILE form does NOT auto-wire the cd-target dependency
        # (only the TARGET form does), so wire it by hand. boot/ is processed
        # before drivers/, so livecd/bootcd already exist at the call site.
        add_cd_file(FILE ${_out} DESTINATION ${_acm_DESTINATION}
            NAME_ON_CD ${_acm_NAME_ON_CD} NO_CAB FOR all)
        foreach(_cd bootcd livecd bootcdregtest)
            if(TARGET ${_cd})
                add_dependencies(${_cd} ${_target})
            endif()
        endforeach()
    endif()
endfunction()

## Thin wrapper: a cargo-driven kernel-mode .dll (kdcom/bootvid). See
## add_cargo_module. Keep spec2def(... ADD_IMPORTLIB) before calling (consumers
## import the .dll's exports; the importlib is independent of this target).
function(add_cargo_kernel_dll _target _crate _dllname)
    cmake_parse_arguments(_ackd "" "DESTINATION;NAME_ON_CD;BODY_CRATE" "EXTRA_DEPENDS" ${ARGN})
    set(_args KIND kerneldll)
    if(_ackd_BODY_CRATE)
        list(APPEND _args BODY_CRATE ${_ackd_BODY_CRATE})
    endif()
    if(_ackd_DESTINATION)
        list(APPEND _args DESTINATION ${_ackd_DESTINATION})
    endif()
    if(_ackd_NAME_ON_CD)
        list(APPEND _args NAME_ON_CD ${_ackd_NAME_ON_CD})
    endif()
    if(_ackd_EXTRA_DEPENDS)
        list(APPEND _args EXTRA_DEPENDS ${_ackd_EXTRA_DEPENDS})
    endif()
    add_cargo_module(${_target} ${_crate} ${_dllname} ${_args})
endfunction()

function(generate_import_lib _libname _dllname _spec_file __version_arg __dbg_arg)
    # Generate the def for the import lib
    add_custom_command(
        OUTPUT ${CMAKE_CURRENT_BINARY_DIR}/${_libname}_implib.def
        COMMAND native-spec2def ${__version_arg} ${__dbg_arg} -n=${_dllname} -a=${ARCH2} ${ARGN} --implib -d=${CMAKE_CURRENT_BINARY_DIR}/${_libname}_implib.def ${CMAKE_CURRENT_SOURCE_DIR}/${_spec_file}
        DEPENDS ${CMAKE_CURRENT_SOURCE_DIR}/${_spec_file} native-spec2def)

    # With this, we let DLLTOOL create an import library
    # Note: previously we re-archived the import library created by dlltool into
    # a thin archive ('ar crT'). This is broken with binutils 2.40+ (bug #31614)
    # and leads to a linker crash. The new method of using an IMPORTED library
    # avoids this. It does no longer allow to embed new object files into the
    # import libraries, but this is replaced by linking the additinal libraries
    # to the import library.
    set(LIBRARY_PRIVATE_DIR ${CMAKE_CURRENT_BINARY_DIR}${CMAKE_FILES_DIRECTORY}/${_libname}.dir)
    add_custom_command(
        OUTPUT ${LIBRARY_PRIVATE_DIR}/${_libname}.a
        # ar just puts stuff into the archive, without looking twice. Just delete the lib, we're going to rebuild it anyway
        COMMAND ${CMAKE_COMMAND} -E rm -f ${LIBRARY_PRIVATE_DIR}/${_libname}.a
        COMMAND ${CMAKE_DLLTOOL} ${DLLTOOL_MACHINE_FLAG} -d ${CMAKE_CURRENT_BINARY_DIR}/${_libname}_implib.def -k -l ${_libname}.a -t ${_libname}
        DEPENDS ${CMAKE_CURRENT_BINARY_DIR}/${_libname}_implib.def
        WORKING_DIRECTORY ${LIBRARY_PRIVATE_DIR})

    # Create a custom target for the import library generation
    add_custom_target(${_libname}_implib_target DEPENDS ${LIBRARY_PRIVATE_DIR}/${_libname}.a)

    # Create an IMPORTED library that references the dlltool output
    _add_library(${_libname} STATIC IMPORTED GLOBAL)
    set_target_properties(${_libname} PROPERTIES IMPORTED_LOCATION ${LIBRARY_PRIVATE_DIR}/${_libname}.a)
    add_dependencies(${_libname} ${_libname}_implib_target)

    # Do the same with delay-import libs
    set(LIBRARY_PRIVATE_DIR ${CMAKE_CURRENT_BINARY_DIR}${CMAKE_FILES_DIRECTORY}/${_libname}_delayed.dir)
    # llvm-dlltool doesn't support --output-delaylib. Instead of an empty archive (which
    # silently degraded delay-imports to regular imports OS-wide), generate a real
    # delay-import lib ourselves: sdk/tools/gen-delaylib.py emits the descriptor + INT/IAT
    # + per-function thunk from the implib .def and clang-compiles it. The delay-load
    # runtime helper is sdk/lib/delayimp (__delayLoadHelper2). See
    # docs/engineering/archive/delay-import-llvm-mingw-design.md. VIBEOS_ENABLE_DELAYLOAD
    # (default ON) selects the generated delay libs; VIBEOS_NO_DELAYLOAD keeps the
    # empty-archive + regular-import fallback.
    if(CMAKE_DLLTOOL_MACHINE AND VIBEOS_ENABLE_DELAYLOAD AND NOT VIBEOS_NO_DELAYLOAD)
        add_custom_command(
            OUTPUT ${LIBRARY_PRIVATE_DIR}/${_libname}_delayed.a
            COMMAND ${CMAKE_COMMAND} -E rm -f ${LIBRARY_PRIVATE_DIR}/${_libname}_delayed.a
            COMMAND ${CMAKE_COMMAND} -E env python3
                ${REACTOS_SOURCE_DIR}/sdk/tools/gen-delaylib.py
                --def ${CMAKE_CURRENT_BINARY_DIR}/${_libname}_implib.def
                --arch ${ARCH}
                --out ${LIBRARY_PRIVATE_DIR}/${_libname}_delayed.a
                --clang ${CMAKE_C_COMPILER} --ar ${CMAKE_AR}
            DEPENDS ${CMAKE_CURRENT_BINARY_DIR}/${_libname}_implib.def
                ${REACTOS_SOURCE_DIR}/sdk/tools/gen-delaylib.py
            WORKING_DIRECTORY ${LIBRARY_PRIVATE_DIR}
            COMMENT "Generating delay-import lib for ${_libname} (gen-delaylib.py, ${ARCH})")
    elseif(CMAKE_DLLTOOL_MACHINE)
        # Default/VIBEOS_NO_DELAYLOAD fallback: empty archive → add_delay_importlibs()
        # uses regular imports.
        add_custom_command(
            OUTPUT ${LIBRARY_PRIVATE_DIR}/${_libname}_delayed.a
            COMMAND ${CMAKE_COMMAND} -E rm -f ${LIBRARY_PRIVATE_DIR}/${_libname}_delayed.a
            COMMAND ${CMAKE_AR} crT ${LIBRARY_PRIVATE_DIR}/${_libname}_delayed.a
            DEPENDS ${CMAKE_CURRENT_BINARY_DIR}/${_libname}_implib.def
            WORKING_DIRECTORY ${LIBRARY_PRIVATE_DIR}
            COMMENT "Creating empty delay-import stub for ${_libname} (delay-load disabled)")
    else()
        add_custom_command(
            OUTPUT ${LIBRARY_PRIVATE_DIR}/${_libname}_delayed.a
            # ar just puts stuff into the archive, without looking twice. Just delete the lib, we're going to rebuild it anyway
            COMMAND ${CMAKE_COMMAND} -E rm -f ${LIBRARY_PRIVATE_DIR}/${_libname}_delayed.a
            COMMAND ${CMAKE_DLLTOOL} ${DLLTOOL_MACHINE_FLAG} -d ${CMAKE_CURRENT_BINARY_DIR}/${_libname}_implib.def -k --output-delaylib=${_libname}_delayed.a -t ${_libname}_delayed
            DEPENDS ${CMAKE_CURRENT_BINARY_DIR}/${_libname}_implib.def
            WORKING_DIRECTORY ${LIBRARY_PRIVATE_DIR})
    endif()

    # Create a custom target for the delay-import library generation
    add_custom_target(${_libname}_delayed_implib_target DEPENDS ${LIBRARY_PRIVATE_DIR}/${_libname}_delayed.a)

    # Create an IMPORTED library for delay-import
    _add_library(${_libname}_delayed STATIC IMPORTED GLOBAL)
    set_target_properties(${_libname}_delayed PROPERTIES IMPORTED_LOCATION ${LIBRARY_PRIVATE_DIR}/${_libname}_delayed.a)
    add_dependencies(${_libname}_delayed ${_libname}_delayed_implib_target)
endfunction()

# spec2def's stubs are Rust: one `rustc --emit=obj` per module, no cargo. amd64 and
# arm64 compile on pinned stable against the builtin triples' prebuilt core. i386
# compiles on the pinned nightly against the user-mode cells' pentium/x86-softfloat
# json, whose core comes from one `-Z build-std` run (tools/spec-stubs-core) shared
# by every module. Stable i686 cannot stand in: it requires sse2 and the C here is
# -march=pentium.
set(SPEC_STUBS_RUSTC_FLAGS
    --edition 2021 --crate-type=lib --emit=obj
    -C opt-level=2 -C panic=abort -C force-frame-pointers=yes)
set(SPEC_STUBS_RUSTC_ENV)
set(SPEC_STUBS_RUSTC_DEPENDS)
if(ARCH STREQUAL "i386")
    file(STRINGS "${REACTOS_SOURCE_DIR}/.rust-nightly" _spec_stubs_nightly LIMIT_COUNT 1)
    set(_spec_stubs_json "${REACTOS_SOURCE_DIR}/sdk/lib/rust/ntdll-cargo/i686-kernel.json")
    set(_spec_stubs_target_dir "${CMAKE_BINARY_DIR}/spec-stubs-core/${_spec_stubs_nightly}")
    set(_spec_stubs_sysroot "${CMAKE_BINARY_DIR}/spec-stubs-core/sysroot")
    set(_spec_stubs_stamp "${_spec_stubs_sysroot}.stamp")
    add_custom_command(
        OUTPUT ${_spec_stubs_stamp}
        COMMAND ${CMAKE_COMMAND} -E env RUSTUP_TOOLCHAIN=${_spec_stubs_nightly}
            CARGO_TARGET_DIR=${_spec_stubs_target_dir}
            cargo build --release --locked --target ${_spec_stubs_json} -Z build-std=core
                --manifest-path ${REACTOS_SOURCE_DIR}/tools/spec-stubs-core/Cargo.toml
        COMMAND ${CMAKE_COMMAND}
            -DDEPS=${_spec_stubs_target_dir}/i686-kernel/release/deps
            -DSYSROOT=${_spec_stubs_sysroot} -DTARGET=i686-kernel
            -P ${REACTOS_SOURCE_DIR}/tools/spec-stubs-core/sysroot.cmake
        COMMAND ${CMAKE_COMMAND} -E touch ${_spec_stubs_stamp}
        DEPENDS ${REACTOS_SOURCE_DIR}/tools/spec-stubs-core/Cargo.toml
            ${REACTOS_SOURCE_DIR}/tools/spec-stubs-core/src/lib.rs
            ${REACTOS_SOURCE_DIR}/tools/spec-stubs-core/sysroot.cmake
            ${_spec_stubs_json} ${REACTOS_SOURCE_DIR}/.rust-nightly
        WORKING_DIRECTORY ${REACTOS_SOURCE_DIR}
        COMMENT "Building core for spec2def's i386 Rust stubs"
        VERBATIM)
    add_custom_target(spec-stubs-core DEPENDS ${_spec_stubs_stamp})
    list(APPEND SPEC_STUBS_RUSTC_FLAGS
        --target ${_spec_stubs_json} -Z unstable-options --sysroot ${_spec_stubs_sysroot})
    set(SPEC_STUBS_RUSTC_ENV RUSTUP_TOOLCHAIN=${_spec_stubs_nightly})
    set(SPEC_STUBS_RUSTC_DEPENDS spec-stubs-core ${_spec_stubs_stamp})
elseif(ARCH STREQUAL "amd64")
    list(APPEND SPEC_STUBS_RUSTC_FLAGS --target x86_64-pc-windows-gnu)
elseif(ARCH STREQUAL "arm64")
    list(APPEND SPEC_STUBS_RUSTC_FLAGS --target aarch64-pc-windows-gnullvm)
else()
    message(FATAL_ERROR "spec2def Rust stubs: no rustc target for ARCH ${ARCH}")
endif()

# Deferred to the end of the directory that called spec2def(): every module there
# lists ${file}_stubs.c among its sources, and this swaps it for the Rust object, so
# no module's CMakeLists has to change. A module in ANOTHER directory naming the .c
# would fail at generate time ("Cannot find source file"), which is the loud outcome.
function(_spec2def_use_rust_stubs _c_stub _rs_obj)
    get_property(_targets DIRECTORY PROPERTY BUILDSYSTEM_TARGETS)
    foreach(_t IN LISTS _targets)
        get_target_property(_srcs ${_t} SOURCES)
        if(NOT _srcs)
            continue()
        endif()
        # A module names the generated stub by its absolute binary-dir path or by its
        # bare name, which CMake resolves against the binary dir at generate time.
        get_filename_component(_c_name "${_c_stub}" NAME)
        list(FIND _srcs "${_c_stub}" _at)
        if(_at EQUAL -1)
            list(FIND _srcs "${_c_name}" _at)
        endif()
        if(_at GREATER -1)
            list(REMOVE_AT _srcs ${_at})
            list(INSERT _srcs ${_at} "${_rs_obj}")
            set_property(TARGET ${_t} PROPERTY SOURCES ${_srcs})
            # A DLL that is nothing but stubs now has no C source for CMake to infer
            # the linker language from; it links through the C toolchain as before.
            get_target_property(_lang ${_t} LINKER_LANGUAGE)
            if(NOT _lang)
                set_property(TARGET ${_t} PROPERTY LINKER_LANGUAGE C)
            endif()
        endif()
    endforeach()
endfunction()

function(spec2def _dllname _spec_file)

    cmake_parse_arguments(__spec2def "ADD_IMPORTLIB;NO_PRIVATE_WARNINGS;WITH_RELAY;WITH_DBG;NO_DBG" "VERSION" "" ${ARGN})

    # Get library basename
    get_filename_component(_file ${_dllname} NAME_WLE)

    # Error out on anything else than spec
    if(NOT ${_spec_file} MATCHES ".*\\.spec")
        message(FATAL_ERROR "spec2def only takes spec files as input.")
    endif()

    if(__spec2def_WITH_RELAY)
        set(__with_relay_arg "--with-tracing")
    endif()

    if(__spec2def_VERSION)
        set(__version_arg "--version=0x${__spec2def_VERSION}")
    else()
        set(__version_arg "--version=${DLL_EXPORT_VERSION}")
    endif()

    if(__spec2def_WITH_DBG OR (DBG AND NOT __spec2def_NO_DBG))
        set(__dbg_arg "--dbg")
    endif()

    # Generate the exports def and the stubs for the DLL. Relay-tracing thunks have
    # no Rust form, so a WITH_RELAY module keeps spec2def's C emitter.
    if(__spec2def_WITH_RELAY)
        set(_stubs_src ${CMAKE_CURRENT_BINARY_DIR}/${_file}_stubs.c)
    else()
        set(_stubs_src ${CMAKE_CURRENT_BINARY_DIR}/${_file}_stubs.rs)
    endif()
    add_custom_command(
        OUTPUT ${CMAKE_CURRENT_BINARY_DIR}/${_file}.def ${_stubs_src}
        COMMAND native-spec2def -n=${_dllname} -a=${ARCH2} -d=${CMAKE_CURRENT_BINARY_DIR}/${_file}.def -s=${_stubs_src} ${__with_relay_arg} ${__version_arg} ${__dbg_arg} ${CMAKE_CURRENT_SOURCE_DIR}/${_spec_file}
        DEPENDS ${CMAKE_CURRENT_SOURCE_DIR}/${_spec_file} native-spec2def)

    if(__spec2def_WITH_RELAY)
        # Do not use precompiled headers for the stub file
        set_source_files_properties(${_stubs_src} PROPERTIES SKIP_PRECOMPILE_HEADERS ON)
    else()
        set(_stubs_obj ${CMAKE_CURRENT_BINARY_DIR}/${_file}_stubs.rs.obj)
        add_custom_command(
            OUTPUT ${_stubs_obj}
            COMMAND ${CMAKE_COMMAND} -E env ${SPEC_STUBS_RUSTC_ENV}
                rustc ${SPEC_STUBS_RUSTC_FLAGS} ${_stubs_src} -o ${_stubs_obj}
            DEPENDS ${_stubs_src} ${SPEC_STUBS_RUSTC_DEPENDS}
            WORKING_DIRECTORY ${REACTOS_SOURCE_DIR}
            COMMENT "Compiling ${_file}_stubs.rs"
            VERBATIM)
        # DEFER CALL evaluates its arguments when the call RUNS, after this function's
        # locals are gone; EVAL CODE bakes the two paths in now.
        cmake_language(EVAL CODE "cmake_language(DEFER CALL _spec2def_use_rust_stubs [[${CMAKE_CURRENT_BINARY_DIR}/${_file}_stubs.c]] [[${_stubs_obj}]])")
    endif()

    if(__spec2def_ADD_IMPORTLIB)
        set(_extraflags)
        if(__spec2def_NO_PRIVATE_WARNINGS)
            set(_extraflags --no-private-warnings)
        endif()

        generate_import_lib(lib${_file} ${_dllname} ${_spec_file} ${_extraflags} "${__version_arg}" "${__dbg_arg}")
        target_include_directories(lib${_file} INTERFACE $<TARGET_PROPERTY:${_file},INTERFACE_INCLUDE_DIRECTORIES>)
        target_include_directories(lib${_file}_delayed INTERFACE $<TARGET_PROPERTY:${_file},INTERFACE_INCLUDE_DIRECTORIES>)
    endif()
endfunction()

macro(macro_mc FLAG FILE)
    set(COMMAND_MC ${CMAKE_MC_COMPILER} -u ${FLAG} -b -h ${CMAKE_CURRENT_BINARY_DIR}/ -r ${CMAKE_CURRENT_BINARY_DIR}/ ${FILE})
endmacro()

# PSEH lib, needed with mingw
set(PSEH_LIB "pseh")

# A Rust archive a module names BY PATH on its link line: a path is placed exactly where it
# is written (an INTERFACE wrapper's library lands after the direct items, behind import
# libraries), and Ninja still orders the link after the custom command that produces it.
# Spelled here, not exported from the producing directory, because dll/ and win32ss/ are
# configured BEFORE sdk/lib/: a variable set there is empty when they read it on a fresh
# configure; sdk/lib/crt FATAL_ERRORs if its rust_staticlib_variant disagrees. gdi32's: as the
# `atan2` INTERFACE target it landed behind ntdll's import library, which exports atan2 on
# arm64, and gdi32 imported ntdll's. (The CRT DLLs' pseh/setjmp archives were named this way
# too until they moved inside the CRTs' own Rust graphs.)
set(ATAN2_LIB "${CMAKE_BINARY_DIR}/sdk/lib/crt/libatan2_rust.a")

function(CreateBootSectorTarget _target_name _asm_file _binary_file _base_address)
    set(_object_file ${_binary_file}.o)

    get_defines(_defines)
    get_includes(_includes)

    if(CMAKE_ASM16_GCC)
        # Use i686 mingw GCC for 16-bit boot sector code (handles preprocessing + GNU as)
        add_custom_command(
            OUTPUT ${_object_file}
            COMMAND ${CMAKE_ASM16_GCC} -x assembler-with-cpp -o ${_object_file} -I${REACTOS_SOURCE_DIR}/sdk/include/asm -I${REACTOS_BINARY_DIR}/sdk/include/asm ${_includes} ${_defines} -D__ASM__ -c ${_asm_file}
            DEPENDS ${_asm_file})
    else()
        add_custom_command(
            OUTPUT ${_object_file}
            COMMAND ${CMAKE_ASM_COMPILER} -x assembler-with-cpp -o ${_object_file} -I${REACTOS_SOURCE_DIR}/sdk/include/asm -I${REACTOS_BINARY_DIR}/sdk/include/asm ${_includes} ${_defines} -D__ASM__ -c ${_asm_file}
            DEPENDS ${_asm_file})
    endif()

    add_custom_command(
        OUTPUT ${_binary_file}
        COMMAND native-obj2bin ${_object_file} ${_binary_file} ${_base_address}
        # COMMAND objcopy --output-target binary --image-base 0x${_base_address} ${_object_file} ${_binary_file}
        DEPENDS ${_object_file} native-obj2bin)

    set_source_files_properties(${_object_file} ${_binary_file} PROPERTIES GENERATED TRUE)

    add_custom_target(${_target_name} ALL DEPENDS ${_binary_file})
endfunction()

# Compile an assembly file with i686 mingw GCC instead of Clang.
# Used for 16-bit code that Clang's assembler can't handle (symbol arithmetic, .code16, etc.)
# Returns the object file path in the parent scope variable _result_var.
function(compile_asm_with_gcc _source_file _result_var)
    if(NOT CMAKE_ASM16_GCC)
        message(FATAL_ERROR "compile_asm_with_gcc: CMAKE_ASM16_GCC not set")
    endif()
    get_filename_component(_src_name ${_source_file} NAME_WE)
    set(_object_file ${CMAKE_CURRENT_BINARY_DIR}/${_src_name}_gcc.obj)
    get_filename_component(_source_file_full_path ${_source_file} ABSOLUTE)

    get_defines(_defines)
    get_includes(_includes)

    add_custom_command(
        OUTPUT ${_object_file}
        COMMAND ${CMAKE_ASM16_GCC} -x assembler-with-cpp -o ${_object_file} ${_includes} ${_defines} -I${REACTOS_SOURCE_DIR}/sdk/include/asm -I${REACTOS_BINARY_DIR}/sdk/include/asm -D__ASM__ -c ${_source_file_full_path}
        DEPENDS ${_source_file_full_path})

    set(${_result_var} ${_object_file} PARENT_SCOPE)
endfunction()

function(allow_warnings __module)
    # We don't allow warnings in trunk, this needs to be reworked. See CORE-6959.
    #target_compile_options(${__module} PRIVATE "-Wno-error")
endfunction()

function(convert_asm_file _source_file _target_file)
    get_filename_component(_source_file_base_name ${_source_file} NAME_WE)
    get_filename_component(_source_file_full_path ${_source_file} ABSOLUTE)
    set(_preprocessed_asm_file ${CMAKE_CURRENT_BINARY_DIR}/${_target_file})
    add_custom_command(
        OUTPUT ${_preprocessed_asm_file}
        COMMAND native-asmpp ${_source_file_full_path} > ${_preprocessed_asm_file}
        DEPENDS native-asmpp ${_source_file_full_path})

endfunction()

function(convert_asm_files)
    foreach(_source_file ${ARGN})
        convert_asm_file(${_source_file} ${_source_file}.s)
    endforeach()
endfunction()

macro(add_asm_files _target)
    foreach(_source_file ${ARGN})
        get_filename_component(_extension ${_source_file} EXT)
        get_filename_component(_source_file_base_name ${_source_file} NAME_WE)
        if (${_extension} STREQUAL ".asm")
            convert_asm_file(${_source_file} ${_source_file}.s)
            list(APPEND ${_target} ${CMAKE_CURRENT_BINARY_DIR}/${_source_file}.s)
        elseif (${_extension} STREQUAL ".inc")
            convert_asm_file(${_source_file} ${_source_file}.h)
            list(APPEND ${_target} ${CMAKE_CURRENT_BINARY_DIR}/${_source_file}.h)
        else()
            list(APPEND ${_target} ${_source_file})
        endif()
    endforeach()
endmacro()

function(add_linker_script _target _linker_script_file)
    # LLD in MinGW PE mode does not support -T linker scripts
    if(USE_LLVM_MINGW)
        return()
    endif()
    get_filename_component(_file_full_path ${_linker_script_file} ABSOLUTE)
    target_link_options(${_target} PRIVATE "-Wl,-T,${_file_full_path}")
    set_property(TARGET ${_target} APPEND PROPERTY LINK_DEPENDS ${_file_full_path})
endfunction()

# Manage our C++ options
# we disable standard includes if we don't use the STL
add_compile_options("$<$<AND:$<COMPILE_LANGUAGE:CXX>,$<NOT:$<IN_LIST:cppstl,$<TARGET_PROPERTY:LINK_LIBRARIES>>>>:-nostdinc>")
# we disable RTTI, unless said so
add_compile_options("$<$<COMPILE_LANGUAGE:CXX>:$<IF:$<BOOL:$<TARGET_PROPERTY:WITH_CXX_RTTI>>,-frtti,-fno-rtti>>")
# We disable exceptions, unless said so
add_compile_options("$<$<COMPILE_LANGUAGE:CXX>:$<IF:$<BOOL:$<TARGET_PROPERTY:WITH_CXX_EXCEPTIONS>>,-fexceptions,-fno-exceptions>>")

# G++ shipped with ROSBE uses sjlj exceptions on i386. Tell Clang it is so.
# But when using llvm-mingw, use DWARF exceptions (matching libunwind).
if(CMAKE_C_COMPILER_ID STREQUAL "Clang" AND ARCH STREQUAL "i386" AND NOT USE_LLVM_MINGW)
    add_compile_options("$<$<AND:$<COMPILE_LANGUAGE:CXX>,$<BOOL:$<TARGET_PROPERTY:WITH_CXX_EXCEPTIONS>>>:-fsjlj-exceptions>")
endif()

# Find default G++ libraries
# For llvm-mingw, we use clang directly and provide fallbacks for missing GCC libraries
if(CMAKE_C_COMPILER_ID STREQUAL "Clang")
    # Use clang to find libraries
    set(GXX_EXECUTABLE ${CMAKE_CXX_COMPILER})
else()
    set(GXX_EXECUTABLE ${CMAKE_CXX_COMPILER})
endif()

# Helper macro to find a library with fallback to interface
macro(find_compiler_lib LIBNAME LIBFILE LIBVAR)
    execute_process(
        COMMAND ${GXX_EXECUTABLE} -print-file-name=${LIBFILE}
        OUTPUT_VARIABLE ${LIBVAR}
        OUTPUT_STRIP_TRAILING_WHITESPACE
        ERROR_QUIET)
    # Check if the file actually exists (clang returns just the name if not found)
    if(${LIBVAR} AND EXISTS "${${LIBVAR}}")
        set(${LIBVAR}_FOUND TRUE)
    else()
        set(${LIBVAR}_FOUND FALSE)
    endif()
endmacro()

find_compiler_lib(winpthread libwinpthread.a LIBWINPTHREAD_LOCATION)
if(LIBWINPTHREAD_LOCATION_FOUND)
    add_library(libwinpthread STATIC IMPORTED)
    message(STATUS "Using libwinpthread from ${LIBWINPTHREAD_LOCATION}")
    set_target_properties(libwinpthread PROPERTIES IMPORTED_LOCATION ${LIBWINPTHREAD_LOCATION})
    # libwinpthread needs kernel32 imports, a CRT and msvcrtex
    target_link_libraries(libwinpthread INTERFACE libkernel32 libmsvcrt msvcrtex)
else()
    add_library(libwinpthread INTERFACE)
    message(STATUS "libwinpthread not found, using interface library")
endif()

find_compiler_lib(gcc libgcc.a LIBGCC_LOCATION)
if(LIBGCC_LOCATION_FOUND)
    add_library(libgcc STATIC IMPORTED)
    set_target_properties(libgcc PROPERTIES IMPORTED_LOCATION ${LIBGCC_LOCATION})
    # libgcc needs kernel32 and winpthread (an appropriate CRT must be linked manually)
    target_link_libraries(libgcc INTERFACE libwinpthread libkernel32)
else()
    # For clang/llvm-mingw, use compiler-rt builtins instead of libgcc
    # (needed for i386 64-bit division: __divdi3, __udivdi3, __umoddi3)
    add_library(libgcc INTERFACE)
    if(USE_LLVM_MINGW)
        execute_process(
            COMMAND ${CMAKE_C_COMPILER} --target=${CMAKE_C_COMPILER_TARGET} -print-libgcc-file-name
            OUTPUT_VARIABLE _clang_rt_builtins OUTPUT_STRIP_TRAILING_WHITESPACE)
        if(EXISTS "${_clang_rt_builtins}")
            add_library(libclang_rt_builtins STATIC IMPORTED)
            set_target_properties(libclang_rt_builtins PROPERTIES IMPORTED_LOCATION "${_clang_rt_builtins}")
            target_link_libraries(libgcc INTERFACE libclang_rt_builtins)
            # Also add to global link flags so kernel-mode drivers get it too -- except a
            # link_strict() module, whose Rust compiler_builtins supplies these (see
            # _vibeos_dup_policy).
            add_link_options("$<$<NOT:$<BOOL:$<TARGET_PROPERTY:VIBEOS_STRICT_LINK>>>:${_clang_rt_builtins}>")
            message(STATUS "Using compiler-rt builtins: ${_clang_rt_builtins}")
        endif()
    endif()
    target_link_libraries(libgcc INTERFACE libwinpthread libkernel32)
    message(STATUS "libgcc not found (using clang runtime), using interface library")
endif()

find_compiler_lib(supc++ libsupc++.a LIBSUPCXX_LOCATION)
if(LIBSUPCXX_LOCATION_FOUND)
    add_library(libsupc++ STATIC IMPORTED GLOBAL)
    set_target_properties(libsupc++ PROPERTIES IMPORTED_LOCATION ${LIBSUPCXX_LOCATION})
    # libsupc++ requires libgcc and stdc++compat, and on a GCC SEH toolchain also the
    # unwinder in libgcc_eh.a (SJLJ/DWARF toolchains have none). Upstream gcc.cmake does the
    # same; llvm-mingw never reaches this branch (libc++/libunwind below).
    find_compiler_lib(gcc_eh libgcc_eh.a LIBGCC_EH_LOCATION)
    if(LIBGCC_EH_LOCATION_FOUND)
        target_link_libraries(libsupc++ INTERFACE "${LIBGCC_EH_LOCATION}")
    endif()
    target_link_libraries(libsupc++ INTERFACE libgcc stdc++compat)
else()
    # For clang/llvm-mingw, use libc++, libc++abi, and libunwind instead of libsupc++
    find_compiler_lib(c++ libc++.a LIBCXX_LOCATION)
    find_compiler_lib(c++abi libc++abi.a LIBCXXABI_LOCATION)
    find_compiler_lib(unwind libunwind.a LIBUNWIND_LOCATION)
    if(LIBCXX_LOCATION_FOUND AND LIBCXXABI_LOCATION_FOUND AND LIBUNWIND_LOCATION_FOUND)
        add_library(libcxx STATIC IMPORTED GLOBAL)
        set_target_properties(libcxx PROPERTIES IMPORTED_LOCATION ${LIBCXX_LOCATION})
        add_library(libcxxabi STATIC IMPORTED GLOBAL)
        set_target_properties(libcxxabi PROPERTIES IMPORTED_LOCATION ${LIBCXXABI_LOCATION})
        add_library(libunwind STATIC IMPORTED GLOBAL)
        set_target_properties(libunwind PROPERTIES IMPORTED_LOCATION ${LIBUNWIND_LOCATION})
        # libc++ and libc++abi have circular deps, resolved by --start-group
        target_link_libraries(libcxxabi INTERFACE libunwind)
        target_link_libraries(libcxx INTERFACE libcxxabi)
        add_library(libsupc++ INTERFACE)
        target_link_libraries(libsupc++ INTERFACE libcxx libgcc stdc++compat)
        message(STATUS "Using libc++ (${LIBCXX_LOCATION}), libc++abi (${LIBCXXABI_LOCATION}), libunwind (${LIBUNWIND_LOCATION})")
    else()
        add_library(libsupc++ INTERFACE)
        target_link_libraries(libsupc++ INTERFACE libgcc stdc++compat)
        message(STATUS "libsupc++ not found (using clang runtime), using interface library")
    endif()
endif()

find_compiler_lib(mingwex libmingwex.a LIBMINGWEX_LOCATION)
if(LIBMINGWEX_LOCATION_FOUND)
    add_library(libmingwex STATIC IMPORTED)
    set_target_properties(libmingwex PROPERTIES IMPORTED_LOCATION ${LIBMINGWEX_LOCATION})
    # libmingwex requires a CRT and imports from kernel32
    target_link_libraries(libmingwex INTERFACE libmsvcrt libkernel32)
else()
    add_library(libmingwex INTERFACE)
    target_link_libraries(libmingwex INTERFACE libmsvcrt libkernel32)
    message(STATUS "libmingwex not found, using interface library")
endif()

find_compiler_lib(stdc++ libstdc++.a LIBSTDCXX_LOCATION)
if(LIBSTDCXX_LOCATION_FOUND)
    add_library(libstdc++ STATIC IMPORTED GLOBAL)
    set_target_properties(libstdc++ PROPERTIES IMPORTED_LOCATION ${LIBSTDCXX_LOCATION})
    # libstdc++ requires libsupc++ and mingwex provided by GCC
    target_link_libraries(libstdc++ INTERFACE libsupc++ libmingwex oldnames)
else()
    # For clang/llvm-mingw, we use libc++ instead (libcxx target already created above)
    add_library(libstdc++ INTERFACE)
    if(TARGET libcxx)
        target_link_libraries(libstdc++ INTERFACE libcxx stdc++compat libmingwex oldnames)
    else()
        target_link_libraries(libstdc++ INTERFACE libsupc++ libmingwex oldnames)
    endif()
    message(STATUS "libstdc++ not found (using clang runtime), using interface library")

    # For llvm-mingw, we need to add the libc++ include paths
    # Since -nostdinc removes all default paths, we need to add them back for C++
    get_filename_component(_compiler_dir ${CMAKE_CXX_COMPILER} DIRECTORY)
    get_filename_component(_toolchain_root ${_compiler_dir} DIRECTORY)
    # Get the target triplet (e.g., aarch64-w64-mingw32)
    get_filename_component(_compiler_name ${CMAKE_CXX_COMPILER} NAME)
    string(REGEX REPLACE "-clang\\+\\+$" "" _target_triplet "${_compiler_name}")
    set(_libcxx_include "${_toolchain_root}/${_target_triplet}/include/c++/v1")
    set(_mingw_include "${_toolchain_root}/${_target_triplet}/include")
    # Find clang version for builtin headers
    execute_process(
        COMMAND ${CMAKE_CXX_COMPILER} --version
        OUTPUT_VARIABLE _clang_version_output
        OUTPUT_STRIP_TRAILING_WHITESPACE)
    string(REGEX MATCH "clang version ([0-9]+)" _clang_match "${_clang_version_output}")
    set(_clang_major "${CMAKE_MATCH_1}")
    set(_clang_include "${_toolchain_root}/lib/clang/${_clang_major}/include")
    if(EXISTS "${_libcxx_include}")
        target_include_directories(libstdc++ SYSTEM INTERFACE ${_libcxx_include})
        message(STATUS "Found libc++ headers at: ${_libcxx_include}")
    endif()
    if(EXISTS "${_clang_include}")
        target_include_directories(libstdc++ SYSTEM INTERFACE ${_clang_include})
        message(STATUS "Found clang headers at: ${_clang_include}")
    endif()
    if(EXISTS "${_mingw_include}")
        target_include_directories(libstdc++ SYSTEM INTERFACE ${_mingw_include})
        message(STATUS "Found mingw headers at: ${_mingw_include}")
    endif()
endif()
# this is for our SAL annotations
target_compile_definitions(libstdc++ INTERFACE "$<$<COMPILE_LANGUAGE:CXX>:PAL_STDCPP_COMPAT>")

# Create our alias libraries
add_library(cppstl ALIAS libstdc++)
