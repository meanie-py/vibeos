
function(add_bootvid _target _configfile)
    cmake_parse_arguments(_bviddata "RUST" "NAME_ON_CD" "SOURCES" ${ARGN})

    # Handle the spec file for the dll name
    spec2def(${_target}.dll bootvid.spec ADD_IMPORTLIB)

    # Set default values for the version resource file
    set(REACTOS_STR_INTERNAL_NAME "$<TARGET_NAME:${_target}>")
    set(REACTOS_STR_ORIGINAL_FILENAME "$<TARGET_FILE_NAME:${_target}>")

    # ------------------------------------------------------------------------
    # Rust cohabitation flip (sdk/lib/rust/bootvid-rust -> libbootvid_rust.a)
    # ------------------------------------------------------------------------
    # framebuf/bootvid.c (+ included <drivers/bootvid/framebuf.c>) and the
    # shared common.c/console.c/fontdata.c are translated to Rust. With RUST
    # set, the C logic sources are dropped (kept on disk per never-delete) and
    # the 11 bootvid.spec exports come from the per-arch Rust staticlib.
    if(_bviddata_RUST)
        set(BV_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
        set(BV_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-${_target}-target")
        set(BV_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/lib${_target}_rust.a")
        if(ARCH STREQUAL "i386")
            set(BV_RUST_TARGET "i686-pc-windows-gnu")
        elseif(ARCH STREQUAL "amd64")
            set(BV_RUST_TARGET "x86_64-pc-windows-gnu")
        elseif(ARCH STREQUAL "arm64")
            set(BV_RUST_TARGET "aarch64-pc-windows-gnullvm")
        endif()
        add_custom_command(
            OUTPUT ${BV_RUST_LIB}
            COMMAND ${CMAKE_COMMAND} -E env
                CARGO_TARGET_DIR=${BV_RUST_TARGET_DIR}
                cargo build --release --target ${BV_RUST_TARGET}
                    -p bootvid-rust
                    --manifest-path ${BV_RUST_WORKSPACE_DIR}/Cargo.toml
            COMMAND ${CMAKE_COMMAND} -E copy
                ${BV_RUST_TARGET_DIR}/${BV_RUST_TARGET}/release/libbootvid_rust.a
                ${BV_RUST_LIB}
            DEPENDS ${BV_RUST_WORKSPACE_DIR}/bootvid-rust/Cargo.toml
                    ${BV_RUST_WORKSPACE_DIR}/bootvid-rust/src/lib.rs
                    ${BV_RUST_WORKSPACE_DIR}/bootvid-rust/src/pc.rs
                    ${BV_RUST_WORKSPACE_DIR}/bootvid-rust/src/fontdata.rs
                    ${BV_RUST_WORKSPACE_DIR}/bootvid-rust/src/common.rs
                    ${BV_RUST_WORKSPACE_DIR}/bootvid-rust/src/console.rs
                    ${BV_RUST_WORKSPACE_DIR}/bootvid-rust/src/bootvid.rs
                    ${BV_RUST_WORKSPACE_DIR}/bootvid-rust/src/framebuf_impl.rs
            WORKING_DIRECTORY ${BV_RUST_WORKSPACE_DIR}
            COMMENT "Building bootvid-rust (${BV_RUST_TARGET})"
            VERBATIM)
        add_custom_target(rust_${_target} DEPENDS ${BV_RUST_LIB})

        configure_file(bootvid.rc ${CMAKE_CURRENT_BINARY_DIR}/${_target}.rc)
        file(GENERATE OUTPUT ${CMAKE_CURRENT_BINARY_DIR}/${_target}.rc
                       INPUT ${CMAKE_CURRENT_BINARY_DIR}/${_target}.rc)

        # framebuf/bootvid.c, common.c, console.c, fontdata.c replaced by Rust
        # (kept on disk). rust_cohab_stub.c is the MODULE placeholder TU.
        add_library(${_target} MODULE
            rust_cohab_stub.c
            ${CMAKE_CURRENT_BINARY_DIR}/${_target}.rc
            ${CMAKE_CURRENT_BINARY_DIR}/${_target}.def)
        set_module_type(${_target} kerneldll ENTRYPOINT 0)
        add_importlibs(${_target} ntoskrnl hal)
        target_link_libraries(${_target} ${BV_RUST_LIB})
        add_dependencies(${_target} psdk rust_${_target})

        if(NOT _bviddata_NAME_ON_CD)
            set(_bviddata_NAME_ON_CD "$<TARGET_FILE_NAME:${_target}>")
        endif()
        add_cd_file(TARGET ${_target} DESTINATION reactos/system32 NO_CAB NAME_ON_CD ${_bviddata_NAME_ON_CD} FOR all)
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
