
function(add_bootvid _target _configfile)
    cmake_parse_arguments(_bviddata "RUST" "NAME_ON_CD" "SOURCES" ${ARGN})

    # Handle the spec file for the dll name
    spec2def(${_target}.dll bootvid.spec ADD_IMPORTLIB)

    # Set default values for the version resource file
    set(REACTOS_STR_INTERNAL_NAME "$<TARGET_NAME:${_target}>")
    set(REACTOS_STR_ORIGINAL_FILENAME "$<TARGET_FILE_NAME:${_target}>")

    # ------------------------------------------------------------------------
    # Rust cohabitation flip — cargo-driven final link (bootvid-cargo)
    # ------------------------------------------------------------------------
    # framebuf/bootvid.c (+ included <drivers/bootvid/framebuf.c>) and the
    # shared common.c/console.c/fontdata.c are translated to Rust
    # (sdk/lib/rust/bootvid-rust, kept on disk per never-delete). The 11
    # bootvid.spec Vid* exports come from libbootvid_rust.a. With RUST set, the
    # final bootvid.dll link is driven by cargo (bootvid-cargo's build.rs →
    # reactos-link, ModuleKind::KernelDll) rather than cmake's
    # add_library(MODULE) — add_cargo_kernel_dll builds bootvid-rust + links it
    # with memcmp + the ntoskrnl/hal import libs into the shipping PE and
    # applies the kernel-dll post-link. The spec2def above keeps the import lib
    # (ntoskrnl/ntkrnlmp import bootvid). cmake's link for this DLL is retired.
    if(_bviddata_RUST)
        if(NOT _bviddata_NAME_ON_CD)
            set(_bviddata_NAME_ON_CD ${_target}.dll)
        endif()
        # ZERO cmake .a: libmemcmp.a dropped (bootvid.sys has no memcmp call;
        # compiler-builtins-mem covers it). EXTRA_DEPENDS empty.
        add_cargo_kernel_dll(${_target} bootvid-cargo ${_target}.dll
            BODY_CRATE bootvid-rust NAME_ON_CD ${_bviddata_NAME_ON_CD})
        return()
    endif()

    # Include the per-target definitions (COMPILE_DEFINITIONS, MODULE_HEADER, and SOURCE)
    include(${_configfile})
    list(APPEND SOURCE ${_bviddata_SOURCES})

    # Manage the module-specific header for precomp.h
    if(MODULE_HEADER)
        #get_filename_component(MODULE_HEADER "${MODULE_HEADER}" ABSOLUTE)
        file(RELATIVE_PATH MODULE_HEADER "${CMAKE_CURRENT_SOURCE_DIR}" "${MODULE_HEADER}")
        #string(REPLACE "\\" "/" MODULE_HEADER "${MODULE_HEADER}")
        file(TO_CMAKE_PATH "${MODULE_HEADER}" MODULE_HEADER)
        list(APPEND COMPILE_DEFINITIONS "MODULE_HEADER=\"${MODULE_HEADER}\"")
    endif()

    # Add the common sources
    list(APPEND SOURCE common.c console.c fontdata.c precomp.h)

    # Generate the per-target version resource file in two passes:
    # 1. Configure-time generation to replace the values with generator expressions;
    # 2. Generation-time by overwriting the file with the resolved expressions.
    configure_file(bootvid.rc ${CMAKE_CURRENT_BINARY_DIR}/${_target}.rc)
    file(GENERATE OUTPUT ${CMAKE_CURRENT_BINARY_DIR}/${_target}.rc
                   INPUT ${CMAKE_CURRENT_BINARY_DIR}/${_target}.rc) # TARGET ${_target}

    # Create the actual target
    add_library(${_target} MODULE
        ${SOURCE}
        ${CMAKE_CURRENT_BINARY_DIR}/${_target}.rc
        ${CMAKE_CURRENT_BINARY_DIR}/${_target}.def)
    add_pch(${_target} precomp.h "")
    target_compile_definitions(${_target} PRIVATE ${COMPILE_DEFINITIONS})

    set_module_type(${_target} kerneldll ENTRYPOINT 0)
    add_importlibs(${_target} ntoskrnl hal)
    add_dependencies(${_target} psdk)

    if(NOT _bviddata_NAME_ON_CD)
        set(_bviddata_NAME_ON_CD "$<TARGET_FILE_NAME:${_target}>")
    endif()
    add_cd_file(TARGET ${_target} DESTINATION reactos/system32 NO_CAB NAME_ON_CD ${_bviddata_NAME_ON_CD} FOR all)
endfunction()

# All architectures now boot via UEFI and drive the GOP linear framebuffer, so
# bootvid uses the framebuffer (lfb) path everywhere. The legacy i386/pc VGA
# INT10h bootvid (i386/pc/vgabvid.cmake) is dead under UEFI (no VGA BIOS / the
# card is in a linear GOP mode) and was removed with the BIOS prune.
# bootvid.dll is flipped to Rust (sdk/lib/rust/bootvid-rust). lfbbvid.dll (a
# vestigial alias not loaded by INBV) stays C for now.
add_bootvid(bootvid framebuf/lfbbvid.cmake RUST)
add_bootvid(lfbbvid framebuf/lfbbvid.cmake)
