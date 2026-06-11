
include_directories(
    ${REACTOS_SOURCE_DIR}/sdk/lib/drivers/arbiter
    ${REACTOS_SOURCE_DIR}/sdk/include/reactos/drivers)

# ============================================================================
# pcix.sys — Rust cohabitation flip (28 of 34 .c -> pcix-rust)
# ============================================================================
# The full MS-architecture PCI bus driver (arb/ resource arbiters, intrface/ bus
# interfaces, pci/ config-space, + the top-level FDO/PDO/enum/dispatch/power/init)
# is translated to Rust (sdk/lib/rust/pcix-rust, kept on disk per the never-delete
# rule). 28 .c bodies are replaced by libpcix_rust.a; guid.c (GUID data instantiation,
# referenced by the Rust modules) + 4 empty stub TUs (usage/ideintrf/pci-devhere/
# pci-romimage, 0 functions) stay C. pcix is NOT loaded at boot (pci.sys is the live
# PNP0A03 driver), so the integration bar is a clean link ×3, not smoke boot.
#
# RUSTC_BOOTSTRAP=1: pci/id.rs faithfully translates id.c's C-variadic PciIdPrintf/
# PciIdPrintfAppend (va_start/vsprintf), which need the nightly `c_variadic` feature;
# build.rs sets the pcix_nightly cfg when bootstrap/nightly is available. The real
# build host is nightly anyway (cargo-drive); this keeps the standard-target staticlib
# build self-sufficient. Mirrors the pci-rust / arbiter cohab.
set(PCIX_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
set(PCIX_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-pcix-target")
set(PCIX_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libpcix_rust.a")
if(ARCH STREQUAL "i386")
    set(PCIX_RUST_TARGET "i686-pc-windows-gnu")
    set(PCIX_RUST_RUSTFLAGS "-C target-feature=-sse,-sse2")
elseif(ARCH STREQUAL "amd64")
    set(PCIX_RUST_TARGET "x86_64-pc-windows-gnu")
    set(PCIX_RUST_RUSTFLAGS "")
elseif(ARCH STREQUAL "arm64")
    set(PCIX_RUST_TARGET "aarch64-pc-windows-gnullvm")
    set(PCIX_RUST_RUSTFLAGS "")
endif()
file(GLOB_RECURSE PCIX_RUST_SRCS ${PCIX_RUST_WORKSPACE_DIR}/pcix-rust/src/*.rs)
add_custom_command(
    OUTPUT ${PCIX_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${PCIX_RUST_TARGET_DIR}
        RUSTFLAGS=${PCIX_RUST_RUSTFLAGS}
        RUSTC_BOOTSTRAP=1
        cargo build --release --target ${PCIX_RUST_TARGET}
            -p pcix-rust
            --manifest-path ${PCIX_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${PCIX_RUST_TARGET_DIR}/${PCIX_RUST_TARGET}/release/libpcix_rust.a
        ${PCIX_RUST_LIB}
    DEPENDS ${PCIX_RUST_SRCS}
            ${PCIX_RUST_WORKSPACE_DIR}/pcix-rust/Cargo.toml
            ${PCIX_RUST_WORKSPACE_DIR}/pcix-rust/build.rs
    WORKING_DIRECTORY ${PCIX_RUST_WORKSPACE_DIR}
    COMMENT "Building pcix-rust (${PCIX_RUST_TARGET})"
    VERBATIM)
add_custom_target(rust_pcix DEPENDS ${PCIX_RUST_LIB})

# 28 .c bodies replaced by Rust (pcix-rust), kept on disk per the never-delete rule:
#   arb/{ar_busno,ar_memio,arb_comn,tr_irq}.c
#   intrface/{agpintrf,busintrf,cardbus,devhere,intrface,lddintrf,locintrf,pmeintf,routintf}.c
#   pci/{busno,config,id,ppbridge,state}.c
#   debug.c device.c dispatch.c enum.c fdo.c hookhal.c init.c pcivrify.c pdo.c power.c utils.c
# guid.c (GUID data) + the 4 empty stub TUs stay C:
list(APPEND SOURCE
    usage.c
    intrface/ideintrf.c
    pci/devhere.c
    pci/romimage.c)

list(APPEND PCH_SKIP_SOURCE
    guid.c)

add_library(pcix MODULE
    ${SOURCE}
    ${PCH_SKIP_SOURCE}
    pci.rc)

set_module_type(pcix kernelmodedriver)
target_link_libraries(pcix arbiter)
add_dependencies(pcix rust_pcix)
target_link_libraries(pcix ${PCIX_RUST_LIB})
if(MSVC)
    target_link_options(pcix PRIVATE /SECTION:.rsrc,!D)
else()
    add_custom_command(
        TARGET pcix POST_BUILD
        COMMAND native-pefixup --section:.rsrc,!D $<TARGET_FILE:pcix>
        VERBATIM)
endif()
add_importlibs(pcix ntoskrnl hal)
add_dependencies(pcix pciclass)
add_cd_file(TARGET pcix DESTINATION reactos/system32/drivers NO_CAB FOR all)
