##
## PROJECT:     FreeLoader
## LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
## PURPOSE:     Build definitions for the PC-AT (legacy BIOS) loader, freeldr.sys
## COPYRIGHT:   Copyright 2003 Brian Palmer <brianp@sginet.com>
##              Copyright 2011-2014 Amine Khaldi <amine.khaldi@reactos.org>
##              Copyright 2011-2014 Timo Kreuzer <timo.kreuzer@reactos.org>
##              Copyright 2014 Hervé Poussineau <hpoussin@reactos.org>
##              Copyright 2023 Hermès Bélusca-Maïto <hermes.belusca-maito@reactos.org>
##
## Pruned 2026-05-29 (UEFI-only) and restored 2026-10-05 for Libreboot/SeaBIOS
## machines. freeldr.sys = frldr16.bin (16-bit real-mode entry, loaded by the
## boot sector at 0xF800) + freeldr_pe.exe (the long-mode loader body). The body
## is all Rust now (freeldr-rust without the `uefi` feature); its final link is
## driven by cargo (sdk/lib/rust/freeldr-cargo), like uefildr.efi and rosload.exe.
## amd64 only: see freeldr-cargo/build.rs for why not i386.

CreateBootSectorTarget(frldr16
    ${CMAKE_CURRENT_SOURCE_DIR}/arch/realmode/amd64.S
    ${CMAKE_CURRENT_BINARY_DIR}/frldr16.bin
    F800)

# The loader's entry and BIOS-call asm, assembled with freeldr's own include set
# (include/, arch/pc/*.h) and linked by freeldr-cargo as libfreeldr_bios_asm.a.
# misc.S and linux.S were flipped to Rust global_asm! (loader_asm.rs).
# multiboot.S carries the multiboot header and owns `cmdline`, which entry.S
# hands to BootMain.
add_asm_files(freeldr_bios_asm_objs
    arch/i386/multiboot.S
    arch/amd64/entry.S
    arch/amd64/int386.S
    arch/amd64/pnpbios.S
    arch/amd64/bss_markers.S)
add_library(freeldr_bios_asm STATIC ${freeldr_bios_asm_objs})
set_target_properties(freeldr_bios_asm PROPERTIES LINKER_LANGUAGE C)
add_dependencies(freeldr_bios_asm asm xdk)

set(FREELDR_BIOS_EXTRA_DEPENDS
    freeldr_bios_asm
    ${FREELDR_BLRTL_SOURCES}
    ${CMAKE_CURRENT_SOURCE_DIR}/lib/strsafe_shim.c
    ${CMAKE_CURRENT_SOURCE_DIR}/lib/cmlib_loader_stubs.c
    ${CMAKE_CURRENT_SOURCE_DIR}/freeldr.spec
    ${FREELDR_CMBOOT_RUST_SOURCES}
    ${FREELDR_CNT_RUST_SOURCES}
    ${FREELDR_CPORT_RUST_SOURCES})

add_cargo_module(freeldr_pe freeldr-cargo freeldr_pe.exe
    KIND biosloader NO_CD BODY_CRATE freeldr-rust
    EXTRA_DEPENDS ${FREELDR_BIOS_EXTRA_DEPENDS})

concatenate_files(
    ${CMAKE_CURRENT_BINARY_DIR}/freeldr.sys
    ${CMAKE_CURRENT_BINARY_DIR}/frldr16.bin
    ${CMAKE_CURRENT_BINARY_DIR}/freeldr_pe.exe)
add_custom_target(freeldr ALL DEPENDS ${CMAKE_CURRENT_BINARY_DIR}/freeldr.sys frldr16 freeldr_pe)

add_cd_file(TARGET freeldr FILE ${CMAKE_CURRENT_BINARY_DIR}/freeldr.sys DESTINATION loader NO_CAB NOT_IN_HYBRIDCD FOR bootcd livecd hybridcd regtest)
