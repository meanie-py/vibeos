
add_definitions(
    -D_NTHALDLL_
    -D_NTHAL_)

include_directories(
    include
    ${REACTOS_SOURCE_DIR}/ntoskrnl/include
    ${REACTOS_SOURCE_DIR}/sdk/include/reactos/libs/fast486)

# ============================================================================
# Rust HAL static archive (sdk/lib/rust/hal-rust → libhal_rust.a)
# ============================================================================
# The hal-rust crate is a single staticlib that contains all translated
# HAL files (per file, see hal-rust/src/{x86,i386,amd64,irql,...}/).  Its
# #[no_mangle] exports replace the corresponding C functions which are
# removed from generic.cmake / legacy.cmake / etc. as files are translated.
#
# Mirrors the ntoskrnl integration pattern in ntoskrnl/CMakeLists.txt.

set(HAL_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
set(HAL_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-hal-target")
if(ARCH STREQUAL "i386")
    set(HAL_RUST_TARGET "i686-pc-windows-gnu")
    # i386: disable SSE — same constraint as kernel-rust (CR4.OSFXSR not
    # set in the early HAL init path; play it safe and avoid SSE entirely
    # in HAL since some HAL paths run extremely early).
    set(HAL_RUST_EXTRA_ENV "RUSTFLAGS=-C target-feature=-sse,-sse2")
elseif(ARCH STREQUAL "amd64")
    set(HAL_RUST_TARGET "x86_64-pc-windows-gnu")
    set(HAL_RUST_EXTRA_ENV "")
endif()

set(HAL_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libhal_rust.a")
add_custom_command(
    OUTPUT ${HAL_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${HAL_RUST_TARGET_DIR} ${HAL_RUST_EXTRA_ENV}
        cargo build --release --target ${HAL_RUST_TARGET}
            -p hal-rust
            --manifest-path ${HAL_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${HAL_RUST_TARGET_DIR}/${HAL_RUST_TARGET}/release/libhal_rust.a
        ${HAL_RUST_LIB}
    DEPENDS ${HAL_RUST_WORKSPACE_DIR}/hal-rust/Cargo.toml
            ${HAL_RUST_WORKSPACE_DIR}/hal-rust/src/lib.rs
            ${HAL_RUST_WORKSPACE_DIR}/hal-rust/src/x86/mod.rs
            ${HAL_RUST_WORKSPACE_DIR}/hal-rust/src/x86/apic_halinit.rs
            ${HAL_RUST_WORKSPACE_DIR}/hal-rust/src/x86/beep.rs
            ${HAL_RUST_WORKSPACE_DIR}/hal-rust/src/x86/buildtype.rs
            ${HAL_RUST_WORKSPACE_DIR}/hal-rust/src/x86/clock.rs
            ${HAL_RUST_WORKSPACE_DIR}/hal-rust/src/x86/cmos.rs
            ${HAL_RUST_WORKSPACE_DIR}/hal-rust/src/x86/cmosbus.rs
            ${HAL_RUST_WORKSPACE_DIR}/hal-rust/src/x86/display.rs
            ${HAL_RUST_WORKSPACE_DIR}/hal-rust/src/x86/drive.rs
            ${HAL_RUST_WORKSPACE_DIR}/hal-rust/src/x86/halinit.rs
            ${HAL_RUST_WORKSPACE_DIR}/hal-rust/src/x86/halpcat.rs
            ${HAL_RUST_WORKSPACE_DIR}/hal-rust/src/x86/irql.rs
            ${HAL_RUST_WORKSPACE_DIR}/hal-rust/src/x86/memory.rs
            ${HAL_RUST_WORKSPACE_DIR}/hal-rust/src/x86/misc.rs
            ${HAL_RUST_WORKSPACE_DIR}/hal-rust/src/x86/nmi.rs
            ${HAL_RUST_WORKSPACE_DIR}/hal-rust/src/x86/pcr.rs
            ${HAL_RUST_WORKSPACE_DIR}/hal-rust/src/x86/pic.rs
            ${HAL_RUST_WORKSPACE_DIR}/hal-rust/src/x86/pic_halinit.rs
            ${HAL_RUST_WORKSPACE_DIR}/hal-rust/src/x86/portio.rs
            ${HAL_RUST_WORKSPACE_DIR}/hal-rust/src/x86/processor.rs
            ${HAL_RUST_WORKSPACE_DIR}/hal-rust/src/x86/profil.rs
            ${HAL_RUST_WORKSPACE_DIR}/hal-rust/src/x86/reboot.rs
            ${HAL_RUST_WORKSPACE_DIR}/hal-rust/src/x86/spinlock.rs
            ${HAL_RUST_WORKSPACE_DIR}/hal-rust/src/x86/sysbus.rs
            ${HAL_RUST_WORKSPACE_DIR}/hal-rust/src/x86/sysinfo.rs
            ${HAL_RUST_WORKSPACE_DIR}/hal-rust/src/x86/up.rs
            ${HAL_RUST_WORKSPACE_DIR}/hal-rust/src/irql/mod.rs
            ${HAL_RUST_WORKSPACE_DIR}/hal-rust/src/arbiter/mod.rs
            ${HAL_RUST_WORKSPACE_DIR}/hal-rust/src/acpi/mod.rs
            ${HAL_RUST_WORKSPACE_DIR}/hal-rust/src/kdpci/mod.rs
            ${HAL_RUST_WORKSPACE_DIR}/hal-rust/src/bus/mod.rs
            ${HAL_RUST_WORKSPACE_DIR}/hal-rust/src/pnpdd/mod.rs
            ${HAL_RUST_WORKSPACE_DIR}/hal-rust/src/i386/mod.rs
            ${HAL_RUST_WORKSPACE_DIR}/hal-rust/src/amd64/mod.rs
    WORKING_DIRECTORY ${HAL_RUST_WORKSPACE_DIR}
    COMMENT "Building hal-rust (${HAL_RUST_TARGET})"
    VERBATIM
)
add_custom_target(rust_hal DEPENDS ${HAL_RUST_LIB})

function(add_hal _halname)
    cmake_parse_arguments(_haldata "" "" "SOURCES;COMPONENTS" ${ARGN})

    # Handle the spec file for the dll name
    spec2def(${_halname}.dll ../hal.spec ADD_IMPORTLIB)

    # Create the actual target
    foreach(_component ${_haldata_COMPONENTS})
        list(APPEND _haldata_SOURCES "$<TARGET_OBJECTS:lib_hal_${_component}>")
    endforeach()
    add_library(${_halname} MODULE
        ${_haldata_SOURCES}
        ${CMAKE_CURRENT_BINARY_DIR}/hal.def)

    if(${_halname} STREQUAL "hal")
        target_link_libraries(${_halname} libcntpr setjmp arbiter fast486 ${PSEH_LIB} ${HAL_RUST_LIB})
    else()
        target_link_libraries(${_halname} libcntpr setjmp fast486 ${PSEH_LIB})
    endif()

    add_importlibs(${_halname} ntoskrnl)
    #add_pch(${_halname} include/hal.h)
    add_dependencies(${_halname} psdk asm)
    if(${_halname} STREQUAL "hal")
        add_dependencies(${_halname} rust_hal)
    endif()
    set_module_type(${_halname} kerneldll ENTRYPOINT HalInitSystem 8)
    add_cd_file(TARGET ${_halname} DESTINATION reactos/system32 NO_CAB FOR all)
    if(MSVC)
        target_link_options(${_halname} PRIVATE /ignore:4216 /SECTION:INIT,ERWD)
    elseif(USE_LLVM_MINGW AND TARGET libclang_rt_builtins)
        target_link_libraries(${_halname} libclang_rt_builtins)
    else()
        target_link_libraries(${_halname} -lgcc)
    endif()
endfunction()

# The components
include(generic.cmake)
include(acpi.cmake)
include(apic.cmake)
# up.cmake removed — all UP-only HAL surface (buildtype, spinlock, up.c) is
# now in hal-rust.

if(ARCH STREQUAL "i386")
    include(pcidata.cmake)
    include(legacy.cmake)
    include(pic.cmake)
    add_subdirectory(minihal)

    add_hal(hal SOURCES pic/halpic.rc COMPONENTS generic legacy pic)

elseif(ARCH STREQUAL "amd64")

    add_hal(hal SOURCES ${HAL_SOURCE} COMPONENTS generic acpi apic)

endif()
