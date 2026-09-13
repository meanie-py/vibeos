

remove_definitions(-D_WIN32_WINNT=0x502)
add_definitions(-D_WIN32_WINNT=0x600)

spec2def(portcls.sys portcls.spec ADD_IMPORTLIB)

list(APPEND SOURCE
    adapter.cpp
    api.cpp
    connection.cpp
    dispatcher.cpp
    dll.cpp
    dma_slave.cpp
    drm.cpp
    drm_port.cpp
    filter_topology.cpp
    filter_dmus.cpp
    filter_wavecyclic.cpp
    filter_wavepci.cpp
    filter_wavert.cpp
    interrupt.cpp
    irp.cpp
    irpstream.cpp
    miniport.cpp
    miniport_dmus.cpp
    miniport_fmsynth.cpp
    pin_dmus.cpp
    pin_wavecyclic.cpp
    pin_wavepci.cpp
    pin_wavert.cpp
    pool.cpp
    port.cpp
    port_dmus.cpp
    port_topology.cpp
    port_wavecyclic.cpp
    port_wavepci.cpp
    port_wavert.cpp
    port_wavertstream.cpp
    power.cpp
    propertyhandler.cpp
    purecall.cpp
    registry.cpp
    # resource.cpp # replaced by sdk/lib/rust/portcls-rust/src/resource_*.rs
    service_group.cpp
    undoc.cpp
    unregister.cpp
    version.cpp)

list(APPEND PCH_SKIP_SOURCE
    guid.cpp)

set(PORTCLS_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
set(PORTCLS_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-portcls-target")
set(PORTCLS_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libportcls_rust.a")
if(ARCH STREQUAL "i386")
    set(PORTCLS_RUST_TARGET "i686-pc-windows-gnu")
    set(PORTCLS_RUST_RUSTFLAGS "-C target-feature=-sse,-sse2")
elseif(ARCH STREQUAL "amd64")
    set(PORTCLS_RUST_TARGET "x86_64-pc-windows-gnu")
    set(PORTCLS_RUST_RUSTFLAGS "")
elseif(ARCH STREQUAL "arm64")
    set(PORTCLS_RUST_TARGET "aarch64-pc-windows-gnullvm")
    set(PORTCLS_RUST_RUSTFLAGS "")
endif()
rust_crate_sources(PORTCLS_RUST_SRCS "${PORTCLS_RUST_WORKSPACE_DIR}" portcls-rust)
add_custom_command(
    OUTPUT ${PORTCLS_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${PORTCLS_RUST_TARGET_DIR}
        RUSTFLAGS=${PORTCLS_RUST_RUSTFLAGS}
        cargo build --release --target ${PORTCLS_RUST_TARGET}
            -p portcls-rust
            --manifest-path ${PORTCLS_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${PORTCLS_RUST_TARGET_DIR}/${PORTCLS_RUST_TARGET}/release/libportcls_rust.a
        ${PORTCLS_RUST_LIB}
    DEPENDS ${PORTCLS_RUST_SRCS} ${PORTCLS_RUST_WORKSPACE_DIR}/portcls-rust/Cargo.toml
    WORKING_DIRECTORY ${PORTCLS_RUST_WORKSPACE_DIR}
    COMMENT "Building portcls-rust (${PORTCLS_RUST_TARGET})"
    VERBATIM)
add_custom_target(rust_portcls DEPENDS ${PORTCLS_RUST_LIB})

add_library(portcls MODULE
    ${SOURCE}
    ${PCH_SKIP_SOURCE}
    portcls.rc
    ${CMAKE_CURRENT_BINARY_DIR}/portcls.def)

if(CMAKE_C_COMPILER_ID STREQUAL "Clang")
    target_compile_options(portcls PRIVATE "-Wno-missing-braces")
endif()

add_dependencies(portcls rust_portcls)
target_link_libraries(portcls ${PORTCLS_RUST_LIB} libcntpr ${PSEH_LIB})
set_module_type(portcls kernelmodedriver ENTRYPOINT 0 )
add_pch(portcls private.hpp "${PCH_SKIP_SOURCE}")
add_importlibs(portcls ntoskrnl ks drmk hal)
add_cd_file(TARGET portcls DESTINATION reactos/system32/drivers FOR all)

# Original C++ object mechanics remain only in these validation oracle targets.
# Their real compilation commands drive pre-payment C/C++ validation and anchors.
add_library(portcls_resource_reference OBJECT EXCLUDE_FROM_ALL resource_rust_reference.c)
add_library(portcls_resource_bridge_check OBJECT EXCLUDE_FROM_ALL resource_rust_bridge.cpp)
