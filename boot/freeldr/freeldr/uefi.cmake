##
## PROJECT:     FreeLoader
## LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
## PURPOSE:     Build definitions for UEFI
## COPYRIGHT:   Copyright 2023 Justin Miller <justinmiller100@gmail.com>
##

include_directories(BEFORE
    ${REACTOS_SOURCE_DIR}/sdk/include/reactos/edk2
    ${REACTOS_SOURCE_DIR}/boot/freeldr/freeldr
    ${REACTOS_SOURCE_DIR}/boot/freeldr/freeldr/include
    ${REACTOS_SOURCE_DIR}/boot/freeldr/freeldr/include/arch/uefi)

list(APPEND UEFILDR_ARC_SOURCE
    ${FREELDR_ARC_SOURCE}
    arch/uefi/stubs.c
    arch/uefi/uefisetup.c
    arch/uefi/uefiutil.c
    arch/vidfb.c
    arch/vgafont.c)
    # uefi-hw unit → Rust (freeldr-rust: ueficon/uefidisk/uefihw/uefimem/
    # uefivid.rs). Cohabitation: .c kept on disk, Rust #[no_mangle] takes
    # over. (stubs/uefisetup/uefiutil kept — not translated.)
    # arch/uefi/ueficon.c   # replaced by Rust (freeldr-rust/src/ueficon.rs)
    # arch/uefi/uefidisk.c  # replaced by Rust (freeldr-rust/src/uefidisk.rs)
    # arch/uefi/uefihw.c    # replaced by Rust (freeldr-rust/src/uefihw.rs)
    # arch/uefi/uefimem.c   # replaced by Rust (freeldr-rust/src/uefimem.rs)
    # arch/uefi/uefivid.c   # replaced by Rust (freeldr-rust/src/uefivid.rs)

if(ARCH STREQUAL "i386")
    # arch/i386/i386idt.c  # replaced by Rust (freeldr-rust/src/i386idt.rs)
    list(APPEND UEFILDR_COMMON_ASM_SOURCE
        arch/uefi/i386/uefiasm.S
        arch/i386/i386trap.S)
elseif(ARCH STREQUAL "amd64")
    list(APPEND UEFILDR_COMMON_ASM_SOURCE
        arch/uefi/amd64/uefiasm.S)
elseif(ARCH STREQUAL "arm64")
    list(APPEND UEFILDR_ARC_SOURCE
        arch/arm64/debug.c)
    list(APPEND UEFILDR_COMMON_ASM_SOURCE
        arch/uefi/arm64/uefiasm.S)
else()
    #TBD
endif()

list(APPEND UEFILDR_BOOTMGR_SOURCE
    ${FREELDR_BOOTMGR_SOURCE}
    custom.c
    options.c
    oslist.c
)

add_asm_files(uefifreeldr_common_asm ${FREELDR_COMMON_ASM_SOURCE} ${UEFILDR_COMMON_ASM_SOURCE})

add_library(uefifreeldr_common
    ${uefifreeldr_common_asm}
    ${UEFILDR_ARC_SOURCE}
    ${FREELDR_BOOTLIB_SOURCE}
    ${UEFILDR_BOOTMGR_SOURCE}
    ${FREELDR_NTLDR_SOURCE})

target_compile_definitions(uefifreeldr_common PRIVATE _FRLDRLIB_ UEFIBOOT)

if(CMAKE_C_COMPILER_ID STREQUAL "GNU" OR CMAKE_C_COMPILER_ID STREQUAL "Clang")
    # Prevent using SSE (no support in freeldr) - x86/x64 only
    if(ARCH STREQUAL "i386" OR ARCH STREQUAL "amd64")
        target_compile_options(uefifreeldr_common PUBLIC -mno-sse)
    endif()
endif()

set(PCH_SOURCE
    ${UEFILDR_ARC_SOURCE}
    ${FREELDR_BOOTLIB_SOURCE}
    ${UEFILDR_BOOTMGR_SOURCE}
    ${FREELDR_NTLDR_SOURCE})

add_pch(uefifreeldr_common include/arch/uefi/uefildr.h PCH_SOURCE)
add_dependencies(uefifreeldr_common bugcodes asm xdk)

# Rust peloader (replaces lib/peloader.c — see boot/freeldr/freeldr/CMakeLists.txt)
target_link_libraries(uefifreeldr_common ${FREELDR_RUST_UEFI_LIB})
add_dependencies(uefifreeldr_common rust_freeldr_uefi)

## GCC builds need this extra thing for some reason...
if(ARCH STREQUAL "i386" AND NOT MSVC)
    target_link_libraries(uefifreeldr_common mini_hal ${HAL_RUST_MINI_LIB})
    add_dependencies(uefifreeldr_common rust_hal_mini)
endif()


spec2def(uefildr.exe freeldr.spec)

list(APPEND UEFILDR_BASE_SOURCE
    include/arch/uefi/uefildr.h
    arch/uefi/uefildr.c
    bootmgr.c
    ${FREELDR_BASE_SOURCE})
    # setup/legacy unit → Rust (freeldr-rust: setupldr.rs/inffile.rs).
    # ntldr/setupldr.c  # replaced by Rust (freeldr-rust/src/setupldr.rs)
    # ntldr/inffile.c   # replaced by Rust (freeldr-rust/src/inffile.rs)

if(ARCH STREQUAL "i386")
    # Must be included together with disk/scsiport.c
    list(APPEND UEFILDR_BASE_SOURCE
        ${CMAKE_CURRENT_BINARY_DIR}/uefildr.def)
endif()

add_executable(uefildr ${UEFILDR_BASE_SOURCE})
set_target_properties(uefildr PROPERTIES SUFFIX ".efi")

target_compile_definitions(uefildr PRIVATE UEFIBOOT)

# On AMD64 we only map 1GB with freeloader, tell UEFI to keep us low!
if(ARCH STREQUAL "amd64")
    set_image_base(uefildr 0x10000)
endif()

if(MSVC)
target_link_options(uefildr PRIVATE /DYNAMICBASE:NO)
    target_link_options(uefildr PRIVATE /NXCOMPAT:NO /ignore:4078 /ignore:4254 /DRIVER)
    # We don't need hotpatching
    remove_target_compile_option(uefildr "/hotpatch")
else()
    # ARM64 requires 4KB alignment for proper ldr/str instruction offsets
    if(ARCH STREQUAL "arm64")
        target_link_options(uefildr PRIVATE -Wl,--exclude-all-symbols,--file-alignment,0x1000,--section-alignment,0x1000)
    else()
        target_link_options(uefildr PRIVATE -Wl,--exclude-all-symbols,--file-alignment,0x200,--section-alignment,0x200)
    endif()
    # Strip everything, including rossym data
    add_custom_command(TARGET uefildr
                    POST_BUILD
                    COMMAND ${CMAKE_STRIP} --remove-section=.rossym $<TARGET_FILE:uefildr>
                    COMMAND ${CMAKE_STRIP} --strip-all $<TARGET_FILE:uefildr>)
endif()

if(MSVC)
    set_subsystem(uefildr EFI_APPLICATION)
else()
    set_subsystem(uefildr 10)
endif()

set_entrypoint(uefildr EfiEntry)

if(ARCH STREQUAL "i386")
    target_link_libraries(uefildr mini_hal ${HAL_RUST_MINI_LIB})
    add_dependencies(uefildr rust_hal_mini)
endif()

target_link_libraries(uefildr uefifreeldr_common cportlib blcmlib blrtl libcntpr chkstk)

# dynamic analysis switches
if(STACK_PROTECTOR)
    target_link_libraries(uefildr gcc_ssp_nt)
endif()

if(RUNTIME_CHECKS)
    target_link_libraries(uefildr runtmchk)
endif()

add_dependencies(uefildr xdk)
