
cmake_minimum_required(VERSION 3.17.0)

if(NOT CMAKE_VERSION MATCHES "ReactOS")
    message(WARNING "Building with \"${CMAKE_COMMAND}\", which is not the custom CMake included in RosBE, might cause build issues...")
endif()

include(CMakeDependentOption)

# CMAKE_CROSSCOMPILING and MSVC_IDE are not set until project() is called, so let's test this instead
if ((DEFINED CMAKE_TOOLCHAIN_FILE) AND (CMAKE_GENERATOR MATCHES "Visual Studio.*"))
# Do not use MSVC_RUNTIME_LIBRARY target property. We use our own flags instead
message(WARNING "Setting policy CMP0091 to OLD behaviour")
cmake_policy(SET CMP0091 OLD)
endif()

# Emit build-<arch>/compile_commands.json. Required by the cargo-drive's
# in-cargo C-residue compile (reactos-link c_from_compile_db replays cmake's
# exact clang invocation per source — e.g. the loaders' blcmlib/cportlib C).
# Configure-time only; harmless otherwise.
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

project(REACTOS)

# Emit build-<arch>/compile_commands.json. Required by the cargo-drive's
# in-cargo C-residue compile (reactos-link c_from_compile_db replays cmake's
# exact clang invocation per source — e.g. ntdll's rtl_vista). Configure-time
# only; harmless otherwise.
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

set(CMAKE_INCLUDE_CURRENT_DIR ON)
set(CMAKE_INCLUDE_DIRECTORIES_PROJECT_BEFORE ON)
set(CMAKE_SHARED_LIBRARY_PREFIX "")
set(CMAKE_SHARED_MODULE_PREFIX "")
set(CMAKE_SKIP_PREPROCESSED_SOURCE_RULES TRUE)
set(CMAKE_SKIP_ASSEMBLY_SOURCE_RULES TRUE)
set(CMAKE_COLOR_MAKEFILE OFF)
set(CMAKE_POSITION_INDEPENDENT_CODE OFF)
set(CMAKE_C_STANDARD 99)
set(CMAKE_CXX_STANDARD 11)
#set_property(GLOBAL PROPERTY RULE_MESSAGES OFF)

# check that the ARCH (target architecture) variable is defined
if(NOT ARCH)
    message(FATAL_ERROR "Target architecture (ARCH) is not defined. Please, choose one of: i386, amd64, arm64")
endif()
# Now the ARCH variable will be in lowercase.
# It is needed because STREQUAL comparison
# is case-sensitive.
# See http://cmake.3232098.n2.nabble.com/Case-insensitive-string-compare-td7580269.html
# for more information.
string(TOLOWER ${ARCH} ARCH)

# set possible values for cmake GUI
set_property(CACHE ARCH PROPERTY STRINGS "i386" "amd64" "arm64")

# Alternative WinNT-compatible architecture string
if(ARCH STREQUAL "i386")
    set(WINARCH "x86")
else()
    set(WINARCH ${ARCH})
endif()

# set CMAKE_BUILD_TYPE if not set
if(NOT CMAKE_BUILD_TYPE AND NOT CMAKE_CONFIGURATION_TYPES)
    message(STATUS "Setting build type to Debug as none was specified.")
    set(CMAKE_BUILD_TYPE "Debug" CACHE
        STRING "Choose the type of build." FORCE)
    # Set the possible values of build type for cmake-gui
    set_property(CACHE CMAKE_BUILD_TYPE PROPERTY STRINGS
                 "Debug" "Release" "MinSizeRel" "RelWithDebInfo")
endif()

# Versioning
include(sdk/include/reactos/version.cmake)

# Compile options
include(sdk/cmake/config.cmake)

# Compiler flags handling
include(sdk/cmake/compilerflags.cmake)

# set_wine_module function
include(sdk/cmake/set_wine_module.cmake)

add_definitions(
    -D__REACTOS__
)

# There doesn't seem to be a standard for __FILE__ being relative or absolute, so detect it at runtime.
file(RELATIVE_PATH _PATH_PREFIX ${REACTOS_BINARY_DIR} ${REACTOS_SOURCE_DIR})
if((CMAKE_C_COMPILER_ID STREQUAL "GNU") AND (CMAKE_C_COMPILER_VERSION VERSION_GREATER_EQUAL "8.0.0") OR (CMAKE_C_COMPILER_ID STREQUAL "Clang") AND (CMAKE_C_COMPILER_VERSION VERSION_GREATER_EQUAL "10.0.0"))
    # Thankfully, GCC has this
    add_compile_options(-ffile-prefix-map=${REACTOS_SOURCE_DIR}=)
    add_compile_options(-ffile-prefix-map=${_PATH_PREFIX}=)
else()
    string(LENGTH ${_PATH_PREFIX} _PATH_PREFIX_LENGTH)
    string(LENGTH ${REACTOS_SOURCE_DIR} REACTOS_SOURCE_DIR_LENGTH)
    math(EXPR REACTOS_SOURCE_DIR_LENGTH "${REACTOS_SOURCE_DIR_LENGTH} + 1")
    add_compile_definitions("$<$<COMPILE_LANGUAGE:C,CXX>:__RELFILE__=&__FILE__[__FILE__[0] == '.' ? ${_PATH_PREFIX_LENGTH} : ${REACTOS_SOURCE_DIR_LENGTH}]>")
endif()

# Bison and Flex support
find_package(BISON REQUIRED)
find_package(FLEX REQUIRED)

if(NOT CMAKE_CROSSCOMPILING)
    include(sdk/cmake/host-jobs.cmake)
    set(TOOLS_FOLDER ${CMAKE_CURRENT_BINARY_DIR})
    add_definitions(-DTARGET_${ARCH})

    add_subdirectory(sdk/include/host)

    add_subdirectory(dll/win32/dbghelp)
    add_subdirectory(sdk/tools)
    add_subdirectory(sdk/lib)

    set(NATIVE_TARGETS asmpp bin2c widl gendib cabman fatten hpp isohybrid mkhive mkisofs obj2bin spec2def geninc mkshelllink utf16le xml2sdb)
    list(APPEND NATIVE_TARGETS pefixup)
    if (ARCH STREQUAL "i386")
        list(APPEND NATIVE_TARGETS rsym)
    endif()

    install(TARGETS ${NATIVE_TARGETS})
else()
    # Add host tools target
    include(sdk/cmake/host-tools.cmake)
    setup_host_tools()

    # We don't need CMake importlib handling.
    unset(CMAKE_IMPORT_LIBRARY_SUFFIX)

    # Print build type(s)
    if(CMAKE_CONFIGURATION_TYPES)
        # Multi-config generators, like Visual Studio (MSBuild).
        message("-- Configuration types: ${CMAKE_CONFIGURATION_TYPES}")
    else()
        # Single-configuration generators, like Ninja.
        message("-- Build type: ${CMAKE_BUILD_TYPE}")
    endif()


    # adjust the default behaviour of the FIND_XXX() commands:
    # search headers and libraries in the target environment, search
    # programs in the host environment
    set(CMAKE_FIND_ROOT_PATH_MODE_PROGRAM NEVER)
    set(CMAKE_FIND_ROOT_PATH_MODE_LIBRARY NEVER)
    set(CMAKE_FIND_ROOT_PATH_MODE_INCLUDE NEVER)

    # Add our own target properties
    # General module definitions
    define_property(TARGET PROPERTY REACTOS_MODULE_TYPE
        BRIEF_DOCS "The type of this module"
        FULL_DOCS [[
The type of this module.
One of "nativecui", "nativedll", "kernelmodedriver", "wdmdriver", "kerneldll", "win32cui", "win32gui", "win32dll", "win32ocx", "cpl" or "module"]])

    # C++
    define_property(TARGET PROPERTY WITH_CXX_EXCEPTIONS
        BRIEF_DOCS "Enable C++ exceptions on this target"
        FULL_DOCS [[
Enables C++ exception handling.
Enable this if the module uses try/catch or throw. You might also need this if you use a standard operator new (the one without nothrow).]])
    define_property(TARGET PROPERTY WITH_CXX_RTTI
        BRIEF_DOCS "Enable C++ RTTI on this target"
        FULL_DOCS [[
Enables run-time type information.
Enable this if the module uses typeid or dynamic_cast. You will probably need to link yith cpprt as well, if you are not already using STL.]])


    if(DBG)
        add_definitions(-DDBG=1 -D_SEH_ENABLE_TRACE)
    else()
        add_definitions(-DDBG=0)
    endif()

    if(ENABLE_CCACHE)
        message(WARNING "-- Disabling precompiled headers support (ccache).")
        option(PCH "Whether to use precompiled headers" OFF)
        set_property(GLOBAL PROPERTY RULE_LAUNCH_COMPILE ccache)
    elseif(CMAKE_C_COMPILER_ID STREQUAL "GNU")
        message(WARNING "-- Disabling precompiled headers on GCC by default CORE-17108.")
        option(PCH "Whether to use precompiled headers" OFF)
    else()
        option(PCH "Whether to use precompiled headers" ON)
    endif()

    include(sdk/cmake/host-jobs.cmake)

    # Version Options
    add_definitions(-DWINVER=0x502
                    -D_WIN32_IE=0x603
                    -D_WIN32_WINNT=0x502
                    -D_WIN32_WINDOWS=0x502
                    -D_SETUPAPI_VER=0x502
                    -DMINGW_HAS_SECURE_API=1
                    -DD3D_UMD_INTERFACE_VERSION=0x000C # Vista
                    -DDXGKDDI_INTERFACE_VERSION=0x1052 # Vista
                    -DDLL_EXPORT_VERSION=${DLL_EXPORT_VERSION})

    # Arch Options
    if(ARCH STREQUAL "i386")
        add_definitions(-D_M_IX86)
        add_definitions(-D_X86_ -D__i386__ -Di386)
    elseif(ARCH STREQUAL "amd64")
        add_compile_definitions(_M_AMD64 _M_X64)
        add_definitions(-D_AMD64_ -D__x86_64__ -D_WIN64)
    elseif(ARCH STREQUAL "arm64")
        # GNU tools refer to arm64 as aarch64
        # _M_ARM64 is needed for MSVC-style headers (e.g., setjmp.h)
        add_definitions(-D_ARM64_ -D_M_ARM64 -D__arm64__ -D__aarch64__ -D_WIN64)
    endif()

    # Other
    add_definitions(-D_NEW_DELETE_OPERATORS_)
    if(ARCH STREQUAL "i386")
        add_definitions(-DUSE_COMPILER_EXCEPTIONS -D_USE_32BIT_TIME_T)
    elseif(ARCH STREQUAL "amd64")
        add_compile_definitions(USE_COMPILER_EXCEPTIONS)
    elseif(ARCH STREQUAL "arm64")
        add_compile_definitions(USE_COMPILER_EXCEPTIONS)
    endif()

    # Activate support for assembly source files
    enable_language(ASM)

    # Activate language support for resource files
    enable_language(RC)

    # Localization definitions
    include(sdk/cmake/localization.cmake)
    set(I18N_DEFS "")
    # This will set I18N_DEFS for later use
    set_i18n_language(${I18N_LANG})

    # Compiler specific definitions and macros
    include(sdk/cmake/gcc.cmake)

    # Generic macros
    include(sdk/cmake/CMakeMacros.cmake)

    # rust_crate_sources() — a cargo edge's DEPENDS must span the crate's path-dependency
    # closure, or a change to a shared -sys crate leaves every dependent .a silently stale.
    include(sdk/cmake/rust-crate-deps.cmake)

    # IDL macros for widl/midl
    # We're using widl now for both MSVC and GCC builds
    include(sdk/cmake/widl-support.cmake)

    # Note: On ARM64 with libc++, sdk/include/crt is NOT included here.
    # It is added back in gcc.cmake with lower priority (-idirafter for C++,
    # -isystem for C/ASM) so that libc++'s wrapper headers (stdio.h, etc.)
    # are found first and their #include_next chains work correctly.
    # sdk/include/vcruntime stays in -I because msvcrt and other Wine components
    # depend on it being found before their own local headers.
    # Public headers of vendored libraries, reached as <GL/gl.h> / <d3dkmthk.h> through
    # a root exactly as when they lived under sdk/include. One list, so the next
    # vendor/<lib>/include that serves the whole tree is added here and nowhere else.
    set(VENDOR_INCLUDE_ROOTS
        vendor/opengl-headers/include
        vendor/libdxg/include)

    if(ARCH STREQUAL "arm64")
        include_directories(
            sdk/include
            sdk/include/ddk
            sdk/include/ndk
            sdk/include/psdk
            sdk/include/reactos
            sdk/include/reactos/libs
            sdk/include/vcruntime
            sdk/include/winrt
            ${REACTOS_BINARY_DIR}/sdk/include
            ${REACTOS_BINARY_DIR}/sdk/include/psdk
            ${REACTOS_BINARY_DIR}/sdk/include/ddk
            ${REACTOS_BINARY_DIR}/sdk/include/dxsdk
            ${REACTOS_BINARY_DIR}/sdk/include/reactos
            ${REACTOS_BINARY_DIR}/sdk/include/reactos/mc
            sdk/include/dxsdk
            sdk/lib/pseh/include
            ${VENDOR_INCLUDE_ROOTS}
        )
    else()
        include_directories(
            sdk/include
            sdk/include/crt
            sdk/include/ddk
            sdk/include/ndk
            sdk/include/psdk
            sdk/include/reactos
            sdk/include/reactos/libs
            sdk/include/vcruntime
            sdk/include/winrt
            ${REACTOS_BINARY_DIR}/sdk/include
            ${REACTOS_BINARY_DIR}/sdk/include/psdk
            ${REACTOS_BINARY_DIR}/sdk/include/ddk
            ${REACTOS_BINARY_DIR}/sdk/include/dxsdk
            ${REACTOS_BINARY_DIR}/sdk/include/reactos
            ${REACTOS_BINARY_DIR}/sdk/include/reactos/mc
            sdk/include/dxsdk
            sdk/lib/pseh/include
            ${VENDOR_INCLUDE_ROOTS}
        )
    endif()

    if(FALSE)
        include_directories(${REACTOS_SOURCE_DIR}/sdk/include/reactos/arm)
    endif()

    add_dependency_header()

    add_subdirectory(sdk/include)

    if(ARCH MATCHES "64$")
        include(sdk/cmake/baseaddress64.cmake)
    elseif(NO_ROSSYM)
        include(sdk/cmake/baseaddress_dwarf.cmake)
    else()
        include(sdk/cmake/baseaddress.cmake)
    endif()

    if(SEPARATE_DBG)
        set(CMAKE_PDB_OUTPUT_DIRECTORY "${REACTOS_BINARY_DIR}/symbols")
    endif()

    # (HAL_RUST_MINI_LIB removed: FreeLDR's loader-time Rust mini-HAL is now
    # built by the loaders' own cargo links (.rust_lib("hal-rust", ["minihal"])),
    # not reused from a cmake-built libhal_rust_mini.a — so the cmake
    # rust_hal_mini custom_command and this shared-path var are both retired.)

    # --- menuband-rust shared cohab staticlib (libmenuband_rust.a) ----------------------
    # CMenuBand's Rust translation (sdk/lib/rust/menuband-rust). Unlike the shell32-private
    # COM classes, CMenuBand lived in the SHARED dll/win32/shell32/shellmenu object library
    # and compiled into BOTH shell32.dll AND base/shell/rshell/rshell.dll, so its Rust form
    # is a standalone staticlib linked by both. Defined HERE (not in shellmenu) because
    # `base` (rshell) configures before `dll` (shell32/shellmenu) — the var + target must
    # exist before either link recipe. Each linking DLL provides MenuBandRustModuleLock/
    # Unlock over its own COM module. Mirrors the shell32-rust custom-command shape.
    set(MENUBAND_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
    set(MENUBAND_RUST_TARGET_DIR "${CMAKE_BINARY_DIR}/sdk/rust-menuband-target")
    set(MENUBAND_RUST_LIB "${CMAKE_BINARY_DIR}/sdk/libmenuband_rust.a" CACHE INTERNAL "menuband-rust staticlib")
    if(ARCH STREQUAL "i386")
        set(MENUBAND_RUST_TARGET "i686-pc-windows-gnu")
    elseif(ARCH STREQUAL "amd64")
        set(MENUBAND_RUST_TARGET "x86_64-pc-windows-gnu")
    elseif(ARCH STREQUAL "arm64")
        set(MENUBAND_RUST_TARGET "aarch64-pc-windows-gnullvm")
    endif()
    rust_crate_sources(MENUBAND_RUST_SRCS "${MENUBAND_RUST_WORKSPACE_DIR}" menuband-rust)
    add_custom_command(
        OUTPUT ${MENUBAND_RUST_LIB}
        COMMAND ${CMAKE_COMMAND} -E env
            CARGO_TARGET_DIR=${MENUBAND_RUST_TARGET_DIR}
            cargo build --release --target ${MENUBAND_RUST_TARGET}
                -p menuband-rust
                --manifest-path ${MENUBAND_RUST_WORKSPACE_DIR}/Cargo.toml
        COMMAND ${CMAKE_COMMAND} -E copy
            ${MENUBAND_RUST_TARGET_DIR}/${MENUBAND_RUST_TARGET}/release/libmenuband_rust.a
            ${MENUBAND_RUST_LIB}
        DEPENDS ${MENUBAND_RUST_SRCS}
                ${MENUBAND_RUST_WORKSPACE_DIR}/menuband-rust/Cargo.toml
        WORKING_DIRECTORY ${MENUBAND_RUST_WORKSPACE_DIR}
        COMMENT "Building menuband-rust (${MENUBAND_RUST_TARGET})"
        VERBATIM)
    add_custom_target(rust_menuband DEPENDS ${MENUBAND_RUST_LIB})

    # --- recyclebin-rust cohab staticlib (librecyclebin_rust.a) -------------------------
    # The whole `shellrecyclebin` object library in Rust (sdk/lib/rust/recyclebin-rust):
    # recyclebin.c, recyclebin_generic{,_enumerator}.cpp, recyclebin_v5{,_enumerator}.cpp
    # and guid.c. Unusual among shell32's flips in being a WHOLE LIBRARY TARGET swap rather
    # than a per-TU cohab — those six files were the entire `recyclebin` static library, so
    # there is no half left behind to keep compiling and shell32 links this instead.
    # Measured capture set at the swap: 7 symbols (CloseRecycleBinHandle,
    # DeleteFileToRecycleBinW, EmptyRecycleBinW, EnumerateRecycleBinW, GetDefaultRecycleBin,
    # GetRecycleBinPathFromDriveNumber, RemoveFromRecycleBinDatabase), all published. The
    # rest of what the C++ library defined is C++-mangled class methods and ATL guard
    # variables that never left it.
    # Defined here rather than in shellrecyclebin/ for the same ordering reason as menuband
    # above, and mirroring the same custom-command shape.
    set(RECYCLEBIN_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
    set(RECYCLEBIN_RUST_TARGET_DIR "${CMAKE_BINARY_DIR}/sdk/rust-recyclebin-target")
    set(RECYCLEBIN_RUST_LIB "${CMAKE_BINARY_DIR}/sdk/librecyclebin_rust.a" CACHE INTERNAL "recyclebin-rust staticlib")
    if(ARCH STREQUAL "i386")
        set(RECYCLEBIN_RUST_TARGET "i686-pc-windows-gnu")
    elseif(ARCH STREQUAL "amd64")
        set(RECYCLEBIN_RUST_TARGET "x86_64-pc-windows-gnu")
    elseif(ARCH STREQUAL "arm64")
        set(RECYCLEBIN_RUST_TARGET "aarch64-pc-windows-gnullvm")
    endif()
    rust_crate_sources(RECYCLEBIN_RUST_SRCS "${RECYCLEBIN_RUST_WORKSPACE_DIR}" recyclebin-rust)
    add_custom_command(
        OUTPUT ${RECYCLEBIN_RUST_LIB}
        COMMAND ${CMAKE_COMMAND} -E env
            CARGO_TARGET_DIR=${RECYCLEBIN_RUST_TARGET_DIR}
            cargo build --release --target ${RECYCLEBIN_RUST_TARGET}
                -p recyclebin-rust
                --manifest-path ${RECYCLEBIN_RUST_WORKSPACE_DIR}/Cargo.toml
        COMMAND ${CMAKE_COMMAND} -E copy
            ${RECYCLEBIN_RUST_TARGET_DIR}/${RECYCLEBIN_RUST_TARGET}/release/librecyclebin_rust.a
            ${RECYCLEBIN_RUST_LIB}
        DEPENDS ${RECYCLEBIN_RUST_SRCS}
                ${RECYCLEBIN_RUST_WORKSPACE_DIR}/recyclebin-rust/Cargo.toml
        WORKING_DIRECTORY ${RECYCLEBIN_RUST_WORKSPACE_DIR}
        COMMENT "Building recyclebin-rust (${RECYCLEBIN_RUST_TARGET})"
        VERBATIM)
    add_custom_target(rust_recyclebin DEPENDS ${RECYCLEBIN_RUST_LIB})

    #begin with boot so reactos_cab target is defined before all other modules
    add_subdirectory(boot)
    add_subdirectory(base)
    add_subdirectory(dll)
    add_subdirectory(drivers)
    add_subdirectory(hal)
    add_subdirectory(sdk/lib)
    add_subdirectory(media)
    add_subdirectory(modules)
    add_subdirectory(ntoskrnl)
    add_subdirectory(subsystems)
    add_subdirectory(sdk/tools/wpp)
    add_subdirectory(win32ss)

    # Create the registry hives
    create_registry_hives()

    # Create {bootcd, bootcdregtest, livecd}.lst
    create_iso_lists()

    file(MAKE_DIRECTORY ${REACTOS_BINARY_DIR}/sdk/include/reactos)

    add_dependency_footer()
endif()
