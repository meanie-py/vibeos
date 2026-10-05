
include(ExternalProject)

function(setup_host_tools)
    list(APPEND HOST_TOOLS asmpp bin2c widl gendib cabman fatten hpp isohybrid mkhive mkisofs obj2bin spec2def geninc mkshelllink txt2nls utf16le xml2sdb)
    list(APPEND HOST_TOOLS pefixup)
    if (ARCH STREQUAL "i386")
        list(APPEND HOST_TOOLS rsym)
    endif()
    if ((ARCH STREQUAL "amd64") AND (CMAKE_C_COMPILER_ID STREQUAL "GNU"))
        execute_process(
            COMMAND ${CMAKE_C_COMPILER} --print-file-name=plugin
            OUTPUT_VARIABLE GCC_PLUGIN_DIR)
        string(STRIP ${GCC_PLUGIN_DIR} GCC_PLUGIN_DIR)
        list(APPEND CMAKE_HOST_TOOLS_EXTRA_ARGS -DGCC_PLUGIN_DIR=${GCC_PLUGIN_DIR})
        list(APPEND HOST_MODULES gcc_plugin_seh)
        if (CMAKE_HOST_WIN32)
            list(APPEND HOST_MODULES g++_plugin_seh)
        endif()
    endif()
    list(TRANSFORM HOST_TOOLS PREPEND "${REACTOS_BINARY_DIR}/host-tools/bin/" OUTPUT_VARIABLE HOST_TOOLS_OUTPUT)
    if (CMAKE_HOST_WIN32)
        list(TRANSFORM HOST_TOOLS_OUTPUT APPEND ".exe")
        set(HOST_EXE_SUFFIX ".exe")
        set(HOST_MODULE_SUFFIX ".dll")
    else()
        set(HOST_MODULE_SUFFIX ".so")
    endif()

    # Normalize to the same format as our own ARCH, and add one for the VC shell
    string(TOLOWER "${CMAKE_HOST_SYSTEM_PROCESSOR}" lowercase_CMAKE_HOST_SYSTEM_PROCESSOR)
    if(lowercase_CMAKE_HOST_SYSTEM_PROCESSOR STREQUAL x86 OR lowercase_CMAKE_HOST_SYSTEM_PROCESSOR MATCHES "^i[3456]86$")
        set(HOST_ARCH i386)
        set(VCVARSALL_ARCH x86)
    elseif(lowercase_CMAKE_HOST_SYSTEM_PROCESSOR STREQUAL x86_64 OR lowercase_CMAKE_HOST_SYSTEM_PROCESSOR STREQUAL amd64)
        set(HOST_ARCH amd64)
        set(VCVARSALL_ARCH amd64_x86) # x64 host-tools are not happy compiling for x86...
    elseif(lowercase_CMAKE_HOST_SYSTEM_PROCESSOR STREQUAL arm)
        set(HOST_ARCH arm)
        set(VCVARSALL_ARCH arm)
    # 'aarch64' is used in GNU tools instead of 'arm64'
    elseif(lowercase_CMAKE_HOST_SYSTEM_PROCESSOR STREQUAL arm64 OR lowercase_CMAKE_HOST_SYSTEM_PROCESSOR STREQUAL aarch64)
        set(HOST_ARCH arm64)
        set(VCVARSALL_ARCH arm64)
    else()
        message(FATAL_ERROR "Unknown host architecture: ${lowercase_CMAKE_HOST_SYSTEM_PROCESSOR}")
    endif()

    if(ARCH STREQUAL HOST_ARCH)
        set(HOST_TOOLS_CMAKE_COMMAND "${CMAKE_COMMAND}")
        message("Not cross-compiling, no special host-tools cmake command")
    else()
        set(HOST_TOOLS_CMAKE_COMMAND "${CMAKE_COMMAND}")
        message("Cross-compiling on non-msvc, no special host-tools cmake command")
    endif()

    if(NOT DEFINED HOST_BUILD_TYPE)
        set(HOST_BUILD_TYPE Debug)
    endif()

    # Pass BISON_EXECUTABLE to host-tools if set (needed on macOS where system bison is ancient)
    if(BISON_EXECUTABLE)
        list(APPEND CMAKE_HOST_TOOLS_EXTRA_ARGS -DBISON_EXECUTABLE=${BISON_EXECUTABLE})
    endif()

    ExternalProject_Add(host-tools
        SOURCE_DIR ${REACTOS_SOURCE_DIR}
        PREFIX ${REACTOS_BINARY_DIR}/host-tools
        BINARY_DIR ${REACTOS_BINARY_DIR}/host-tools/bin
        CMAKE_COMMAND ${HOST_TOOLS_CMAKE_COMMAND}
        CMAKE_ARGS
            -UCMAKE_TOOLCHAIN_FILE
            -DARCH:STRING=${ARCH}
            -DCMAKE_INSTALL_PREFIX=${REACTOS_BINARY_DIR}/host-tools
            -DTOOLS_FOLDER=${REACTOS_BINARY_DIR}/host-tools/bin
            -DTARGET_COMPILER_ID=${CMAKE_C_COMPILER_ID}
            -DTARGET_BUILD_TYPE=${CMAKE_BUILD_TYPE}
            -DCMAKE_BUILD_TYPE=${HOST_BUILD_TYPE}
            ${CMAKE_HOST_TOOLS_EXTRA_ARGS}
        INSTALL_COMMAND ${CMAKE_COMMAND} -E true
        BUILD_BYPRODUCTS ${HOST_TOOLS_OUTPUT}
    )

    # Upstream used BUILD_ALWAYS TRUE here. That omits the build stamp, so the
    # phony host-tools chain was DIRTY ON EVERY BUILD and the dirt propagated
    # into ~53k graph nodes — most expensively the ~40 cargo-module custom
    # commands (custom commands re-RUN when any dep is phony-dirty; restat only
    # stops further propagation afterwards), all re-invoking cargo concurrently
    # and serializing on cargo's package-cache lock. Measured cost 2026-07-06:
    # ~87s of phantom re-runs on a literal no-op `ninja livecd`, per arch, on
    # every warm build (so ~4.5 min of pure waste per full gate).
    # Instead: build once, and re-run the step only when a host-tool source
    # actually changes, via file-level step dependencies on the sub-build's
    # real input tree (sdk/tools + the host-built libs + shared headers — the
    # source dirs compiled by host-tools/bin/build.ninja). The inner ninja is
    # incremental, and BUILD_BYPRODUCTS + restat prune downstream when the tool
    # binaries come out unchanged. An over-broad glob is safe (worst case: a
    # cheap inner no-op ninja run); a MISSED input would ship a stale tool, so
    # the glob deliberately sweeps whole directories, all extensions.
    file(GLOB_RECURSE _host_tools_srcs CONFIGURE_DEPENDS
        ${REACTOS_SOURCE_DIR}/sdk/tools/*
        ${REACTOS_SOURCE_DIR}/sdk/lib/cmlib/*
        ${REACTOS_SOURCE_DIR}/sdk/lib/inflib/*
        ${REACTOS_SOURCE_DIR}/sdk/lib/3rdparty/*
        ${REACTOS_SOURCE_DIR}/dll/win32/dbghelp/*
        ${REACTOS_SOURCE_DIR}/sdk/include/*.h)
    ExternalProject_Add_StepDependencies(host-tools build ${_host_tools_srcs})

    ExternalProject_Get_Property(host-tools INSTALL_DIR)

    foreach(_tool ${HOST_TOOLS})
        add_executable(native-${_tool} IMPORTED)
        set_target_properties(native-${_tool} PROPERTIES IMPORTED_LOCATION ${INSTALL_DIR}/bin/${HOST_EXTRA_DIR}${_tool}${HOST_EXE_SUFFIX})
        add_dependencies(native-${_tool} host-tools ${INSTALL_DIR}/bin/${HOST_EXTRA_DIR}${_tool}${HOST_EXE_SUFFIX})
    endforeach()

    foreach(_module ${HOST_MODULES})
        add_library(native-${_module} MODULE IMPORTED)
        set_target_properties(native-${_module} PROPERTIES IMPORTED_LOCATION ${INSTALL_DIR}/bin/${HOST_EXTRA_DIR}${_module}${HOST_MODULE_SUFFIX})
        add_dependencies(native-${_module} host-tools ${INSTALL_DIR}/bin/${HOST_EXTRA_DIR}${_module}${HOST_MODULE_SUFFIX})
    endforeach()
endfunction()
