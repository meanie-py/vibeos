##
## PROJECT:     FreeLoader
## LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
## PURPOSE:     Build definitions for PC-AT and "compatibles" (NEC PC-98, XBOX)
## COPYRIGHT:   Copyright 2003 Brian Palmer <brianp@sginet.com>
##              Copyright 2011-2014 Amine Khaldi <amine.khaldi@reactos.org>
##              Copyright 2011-2014 Timo Kreuzer <timo.kreuzer@reactos.org>
##              Copyright 2014 Hervé Poussineau <hpoussin@reactos.org>
##              Copyright 2023 Hermès Bélusca-Maïto <hermes.belusca-maito@reactos.org>
##

if(ARCH STREQUAL "i386")
    CreateBootSectorTarget(frldr16
        ${CMAKE_CURRENT_SOURCE_DIR}/arch/realmode/i386.S
        ${CMAKE_CURRENT_BINARY_DIR}/frldr16.bin
        F800)
elseif(ARCH STREQUAL "amd64")
    CreateBootSectorTarget(frldr16
        ${CMAKE_CURRENT_SOURCE_DIR}/arch/realmode/amd64.S
        ${CMAKE_CURRENT_BINARY_DIR}/frldr16.bin
        F800)
endif()


spec2def(freeldr.sys freeldr.spec ADD_IMPORTLIB)

list(APPEND PCATLDR_ARC_SOURCE
    ${FREELDR_ARC_SOURCE})
    # i386-hw unit → Rust (freeldr-rust/src/hwide.rs). Cohabitation.
    # arch/drivers/hwide.c  # replaced by Rust (freeldr-rust/src/hwide.rs)

list(APPEND PCATLDR_BOOTMGR_SOURCE
    ${FREELDR_BOOTMGR_SOURCE}
    # linuxboot.c    # replaced by Rust (freeldr-rust/src/linuxboot.rs)
)

list(APPEND PCATLDR_BASE_ASM_SOURCE)

if(ARCH STREQUAL "i386")
    list(APPEND PCATLDR_BASE_ASM_SOURCE
        arch/i386/multiboot.S)

    if(CMAKE_ASM16_GCC)
        # drvmap.S has .code16 with symbol arithmetic that Clang can't handle
        compile_asm_with_gcc(arch/i386/drvmap.S _drvmap_obj)
        set(PCATLDR_GCC_ASM_OBJECTS ${_drvmap_obj})
    else()
        list(APPEND PCATLDR_COMMON_ASM_SOURCE arch/i386/drvmap.S)
    endif()
    list(APPEND PCATLDR_COMMON_ASM_SOURCE
        arch/i386/entry.S
        arch/i386/int386.S
        arch/i386/irqsup.S
        arch/i386/pnpbios.S
        # arch/i386/i386trap.S
        arch/i386/linux.S)

    if(USE_LLVM_MINGW)
        list(APPEND PCATLDR_COMMON_ASM_SOURCE
            arch/amd64/bss_markers.S)
    endif()

    list(APPEND PCATLDR_ARC_SOURCE
        # disk/scsiport.c
        # arch/i386/i386bug.c
        )
        # i386-hw unit → Rust (freeldr-rust: hwapm/i386idt.rs). Cohabitation.
        # arch/i386/hwapm.c    # replaced by Rust (freeldr-rust/src/hwapm.rs)
        # arch/i386/i386idt.c  # replaced by Rust (freeldr-rust/src/i386idt.rs)
        # setup/legacy unit → Rust (freeldr-rust: pxe.rs/drivemap.rs).
        # lib/fs/pxe.c        # replaced by Rust (freeldr-rust/src/pxe.rs)
        # arch/i386/drivemap.c  # replaced by Rust (freeldr-rust/src/drivemap.rs)
        # i386-hw unit → Rust (freeldr-rust: hwacpi/hwdisk/hwpci.rs).
        # arch/i386/hwacpi.c  # replaced by Rust (freeldr-rust/src/hwacpi.rs)
        # arch/i386/hwdisk.c  # replaced by Rust (freeldr-rust/src/hwdisk.rs)
        # arch/i386/hwpci.c   # replaced by Rust (freeldr-rust/src/hwpci.rs)

    # pc-hw unit → Rust (freeldr-rust: machpc/pcvideo/pcdisk/pcmem/pcvesa/
    # pchw/pcbeep/pccons/pcrtc.rs). Cohabitation: .c kept on disk, Rust
    # #[no_mangle] takes over.
        # arch/i386/pc/machpc.c   # replaced by Rust (freeldr-rust/src/machpc.rs)
        # arch/i386/pc/pcbeep.c   # replaced by Rust (freeldr-rust/src/pcbeep.rs)
        # arch/i386/pc/pccons.c   # replaced by Rust (freeldr-rust/src/pccons.rs)
        # arch/i386/pc/pcdisk.c   # replaced by Rust (freeldr-rust/src/pcdisk.rs)
        # arch/i386/pc/pchw.c     # replaced by Rust (freeldr-rust/src/pchw.rs)
        # arch/i386/pc/pcmem.c    # replaced by Rust (freeldr-rust/src/pcmem.rs)
        # arch/i386/pc/pcrtc.c    # replaced by Rust (freeldr-rust/src/pcrtc.rs)
        # arch/i386/pc/pcvesa.c   # replaced by Rust (freeldr-rust/src/pcvesa.rs)
        # arch/i386/pc/pcvideo.c  # replaced by Rust (freeldr-rust/src/pcvideo.rs)

elseif(ARCH STREQUAL "amd64")
    list(APPEND PCATLDR_BASE_ASM_SOURCE
        arch/i386/multiboot.S)

    list(APPEND PCATLDR_COMMON_ASM_SOURCE
        arch/amd64/entry.S
        arch/amd64/int386.S
        arch/amd64/pnpbios.S
        arch/amd64/linux.S)

    if(USE_LLVM_MINGW)
        list(APPEND PCATLDR_COMMON_ASM_SOURCE
            arch/amd64/bss_markers.S)
    endif()

    list(APPEND PCATLDR_ARC_SOURCE
        # arch/i386/i386bug.c
        )
        # setup/legacy unit → Rust (freeldr-rust: pxe.rs/drivemap.rs).
        # lib/fs/pxe.c          # replaced by Rust (freeldr-rust/src/pxe.rs)
        # arch/i386/drivemap.c  # replaced by Rust (freeldr-rust/src/drivemap.rs)
        # i386-hw unit → Rust (freeldr-rust: hwacpi/hwapm/hwdisk/hwpci.rs).
        # arch/i386/hwacpi.c  # replaced by Rust (freeldr-rust/src/hwacpi.rs)
        # arch/i386/hwapm.c   # replaced by Rust (freeldr-rust/src/hwapm.rs)
        # arch/i386/hwdisk.c  # replaced by Rust (freeldr-rust/src/hwdisk.rs)
        # arch/i386/hwpci.c   # replaced by Rust (freeldr-rust/src/hwpci.rs)
        # pc-hw unit → Rust (freeldr-rust). Cohabitation: .c kept on disk.
        # arch/i386/pc/machpc.c   # replaced by Rust (freeldr-rust/src/machpc.rs)
        # arch/i386/pc/pcbeep.c   # replaced by Rust (freeldr-rust/src/pcbeep.rs)
        # arch/i386/pc/pccons.c   # replaced by Rust (freeldr-rust/src/pccons.rs)
        # arch/i386/pc/pcdisk.c   # replaced by Rust (freeldr-rust/src/pcdisk.rs)
        # arch/i386/pc/pchw.c     # replaced by Rust (freeldr-rust/src/pchw.rs)
        # arch/i386/pc/pcmem.c    # replaced by Rust (freeldr-rust/src/pcmem.rs)
        # arch/i386/pc/pcrtc.c    # replaced by Rust (freeldr-rust/src/pcrtc.rs)
        # arch/i386/pc/pcvesa.c   # replaced by Rust (freeldr-rust/src/pcvesa.rs)
        # arch/i386/pc/pcvideo.c  # replaced by Rust (freeldr-rust/src/pcvideo.rs)

else()
    #TBD
endif()

add_asm_files(freeldr_common_asm ${FREELDR_COMMON_ASM_SOURCE} ${PCATLDR_COMMON_ASM_SOURCE})

# Mark GCC-compiled objects as generated so CMake tracks the dependency
if(PCATLDR_GCC_ASM_OBJECTS)
    set_source_files_properties(${PCATLDR_GCC_ASM_OBJECTS} PROPERTIES GENERATED TRUE EXTERNAL_OBJECT TRUE)
endif()

add_library(freeldr_common
    ${freeldr_common_asm}
    ${PCATLDR_GCC_ASM_OBJECTS}
    ${PCATLDR_ARC_SOURCE}
    ${FREELDR_BOOTLIB_SOURCE}
    ${PCATLDR_BOOTMGR_SOURCE}
)
target_compile_definitions(freeldr_common PRIVATE _FRLDRLIB_)

if(MSVC AND CMAKE_C_COMPILER_ID STREQUAL "Clang")
    # We need to reduce the binary size
    target_compile_options(freeldr_common PRIVATE "/Os")
endif()
if(CMAKE_C_COMPILER_ID STREQUAL "GNU" OR CMAKE_C_COMPILER_ID STREQUAL "Clang")
    # Prevent using SSE (no support in freeldr)
    target_compile_options(freeldr_common PUBLIC -mno-sse)
endif()

set(PCH_SOURCE
    ${PCATLDR_ARC_SOURCE}
    ${FREELDR_BOOTLIB_SOURCE}
    ${PCATLDR_BOOTMGR_SOURCE}
)

add_pch(freeldr_common include/freeldr.h PCH_SOURCE)
add_dependencies(freeldr_common bugcodes asm xdk)

# Rust peloader (replaces lib/peloader.c — see boot/freeldr/freeldr/CMakeLists.txt)
target_link_libraries(freeldr_common ${FREELDR_RUST_LIB})
add_dependencies(freeldr_common rust_freeldr)

## GCC builds need this extra thing for some reason...
if(ARCH STREQUAL "i386" AND NOT MSVC)
    target_link_libraries(freeldr_common mini_hal ${HAL_RUST_MINI_LIB})
    add_dependencies(freeldr_common rust_hal_mini)
endif()

add_asm_files(freeldr_base_asm ${PCATLDR_BASE_ASM_SOURCE})

list(APPEND PCATLDR_BASE_SOURCE
    ${freeldr_base_asm}
    ${FREELDR_BASE_SOURCE})

add_executable(freeldr_pe ${PCATLDR_BASE_SOURCE})

set_target_properties(freeldr_pe
    PROPERTIES
    ENABLE_EXPORTS TRUE
    DEFINE_SYMBOL "")

if(MSVC)
    target_link_options(freeldr_pe PRIVATE /ignore:4078 /ignore:4254 /DYNAMICBASE:NO /FIXED /FILEALIGN:512 /ALIGN:512)
    add_linker_script(freeldr_pe freeldr_i386.msvc.lds)
    # We don't need hotpatching
    remove_target_compile_option(freeldr_pe "/hotpatch")
    remove_target_compile_option(freeldr_common "/hotpatch")
else()
    target_link_options(freeldr_pe PRIVATE -Wl,--exclude-all-symbols,--file-alignment,0x200,--section-alignment,0x200)
    add_linker_script(freeldr_pe freeldr_gcc.lds)
    # Strip everything, including rossym data
    add_custom_command(TARGET freeldr_pe
                    POST_BUILD
                    COMMAND ${CMAKE_STRIP} --remove-section=.rossym $<TARGET_FILE:freeldr_pe>
                    COMMAND ${CMAKE_STRIP} --strip-all $<TARGET_FILE:freeldr_pe>)
endif()

set_image_base(freeldr_pe 0x10000)
set_subsystem(freeldr_pe native)
set_entrypoint(freeldr_pe RealEntryPoint)

if(ARCH STREQUAL "i386")
    target_link_libraries(freeldr_pe mini_hal ${HAL_RUST_MINI_LIB})
    add_dependencies(freeldr_pe rust_hal_mini)
endif()

# libfreeldr_rust.a explicitly BEFORE libcntpr so freeldr.rs's
# FPU-free pow/log/log10/wctomb stubs satisfy refs first; otherwise
# CMake's propagated-dep ordering puts libfreeldr_rust AFTER libcntpr,
# libcntpr's pow_asm.s/log10_asm.s/wcstombs_nt.c get pulled in, then
# the Rust stubs collide as dups. The Rust stubs match the C ones
# (deliberately FPU-free; just int3/__debugbreak — see freeldr.rs).
target_link_libraries(freeldr_pe freeldr_common ${FREELDR_RUST_LIB} cportlib libcntpr blrtl)

# dynamic analysis switches
if(STACK_PROTECTOR)
    target_link_libraries(freeldr_pe gcc_ssp_nt)
endif()

if(RUNTIME_CHECKS)
    target_link_libraries(freeldr_pe runtmchk)
    target_link_options(freeldr_pe PRIVATE "/MERGE:.rtc=.text")
endif()

add_dependencies(freeldr_pe asm)

concatenate_files(
    ${CMAKE_CURRENT_BINARY_DIR}/freeldr.sys
    ${CMAKE_CURRENT_BINARY_DIR}/frldr16.bin
    ${CMAKE_CURRENT_BINARY_DIR}/$<TARGET_FILE_NAME:freeldr_pe>)
add_custom_target(freeldr ALL DEPENDS ${CMAKE_CURRENT_BINARY_DIR}/freeldr.sys)

add_cd_file(TARGET freeldr FILE ${CMAKE_CURRENT_BINARY_DIR}/freeldr.sys DESTINATION loader NO_CAB NOT_IN_HYBRIDCD FOR bootcd livecd hybridcd regtest)
