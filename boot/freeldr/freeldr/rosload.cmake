##
## PROJECT:     FreeLoader
## LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
## PURPOSE:     Build definitions for rosload 2nd stage loader
## COPYRIGHT:   Copyright 2024 Timo Kreuzer <timo.kreuzer@reactos.org>
##

spec2def(rosload.exe rosload.spec)

list(APPEND ROSLOAD_SOURCE
    include/freeldr.h
    # strsafe_shim.c — variadic RtlStringCbPrintfA out-of-line shim;
    # the Rust freeldr-rust crate can't host a variadic def on stable
    # (same constraint as freeldr_pe.exe / uefildr.efi). Compiled into
    # rosload.exe directly, matching FREELDR_BASE_SOURCE in CMakeLists.txt.
    lib/strsafe_shim.c
    # Cohabitation: each .c below is replaced by its Rust translation in
    # libfreeldr_rust_rosload.a (--features rosload). .c kept on disk.
    # bootmgr.c          # replaced by Rust (freeldr-rust/src/bootmgr.rs)
    # custom.c           # replaced by Rust (freeldr-rust/src/custom.rs)
    # linuxboot.c        # replaced by Rust (freeldr-rust/src/linuxboot.rs)
    # miscboot.c         # replaced by Rust (freeldr-rust/src/miscboot.rs)
    # options.c          # replaced by Rust (freeldr-rust/src/options.rs)
    # oslist.c           # replaced by Rust (freeldr-rust/src/oslist.rs)
    # lib/rtl/libsupp.c  # replaced by Rust (freeldr-rust/src/libsupp.rs)
    ${REACTOS_SOURCE_DIR}/ntoskrnl/config/cmboot.c
    # ntldr/conversion.c # replaced by Rust (freeldr-rust/src/conversion.rs)
    # ntldr/inffile.c    # replaced by Rust (freeldr-rust/src/inffile.rs)
    # ntldr/registry.c   # replaced by Rust (freeldr-rust/src/registry.rs)
    # ntldr/setupldr.c   # replaced by Rust (freeldr-rust/src/setupldr.rs)
    # ntldr/winldr.c     # replaced by Rust (freeldr-rust/src/winldr.rs)
    # ntldr/wlmemory.c   # replaced by Rust (freeldr-rust/src/wlmemory.rs)
    # ntldr/wlregistry.c # replaced by Rust (freeldr-rust/src/wlregistry.rs)
)

if(ARCH STREQUAL "i386")

    list(APPEND ROSLOAD_SOURCE
        arch/i386/halstub.c
        # arch/i386/ntoskrnl.c       # replaced by Rust (freeldr-rust/src/ntoskrnl.rs)
        disk/scsiport.c
        # ntldr/arch/i386/winldr.c   # replaced by Rust (freeldr-rust/src/winldr_i386.rs)
        # ntldr/headless.c           # replaced by Rust (freeldr-rust/src/headless.rs)
    )

    if(CMAKE_ASM16_GCC)
        compile_asm_with_gcc(arch/i386/drvmap.S _rosload_drvmap_obj)
        set(ROSLOAD_GCC_OBJECTS ${_rosload_drvmap_obj})
    else()
        list(APPEND ROSLOAD_ASM_SOURCE arch/i386/drvmap.S)
    endif()
    list(APPEND ROSLOAD_ASM_SOURCE
        arch/i386/linux.S)

elseif(ARCH STREQUAL "amd64")

    list(APPEND ROSLOAD_SOURCE
        # ntldr/arch/amd64/winldr.c  # replaced by Rust (freeldr-rust/src/winldr_amd64.rs)
    )

    list(APPEND ROSLOAD_ASM_SOURCE
        arch/amd64/misc.S
        arch/amd64/linux.S
    )

else()
    #TBD
endif()

add_asm_files(rosload_asm ${ROSLOAD_ASM_SOURCE})

if(ROSLOAD_GCC_OBJECTS)
    set_source_files_properties(${ROSLOAD_GCC_OBJECTS} PROPERTIES GENERATED TRUE EXTERNAL_OBJECT TRUE)
endif()

add_executable(rosload
    ${ROSLOAD_SOURCE}
    ${rosload_asm}
    ${ROSLOAD_GCC_OBJECTS}
    ${CMAKE_CURRENT_BINARY_DIR}/rosload.def
)

set_target_properties(rosload
    PROPERTIES
    ENABLE_EXPORTS TRUE
    DEFINE_SYMBOL "")

set_image_base(rosload 0x10000) # 0x200000
set_subsystem(rosload native)
set_entrypoint(rosload RunLoader)

if(ARCH STREQUAL "i386")
    target_link_libraries(rosload mini_hal ${HAL_RUST_MINI_LIB})
    add_dependencies(rosload rust_hal_mini)
endif()

# rosload cohabitation flip — link the rosload variant of libfreeldr_rust.a
# (built from boot/freeldr/freeldr/CMakeLists.txt with --features rosload).
# The Rust archive defines bootmgr/oslist/options/custom/winldr/wlmemory/
# wlregistry/registry/setupldr/inffile/headless/conversion/libsupp/miscboot/
# linuxboot/drivemap/ntoskrnl/winldr_<arch>. The corresponding rosload .c
# entries below are commented out (cohabitation) so the Rust #[no_mangle]
# exports take over. Mach* / Ui* / Ini* / FrLdrHeap* / etc. are owned by
# freeldr.sys and reach rosload through add_importlibs(rosload freeldr).
target_link_libraries(rosload ${FREELDR_RUST_ROSLOAD_LIB} blcmlib blrtl libcntpr)
add_dependencies(rosload rust_freeldr_rosload)
add_importlibs(rosload freeldr)

# dynamic analysis switches
if(STACK_PROTECTOR)
    target_link_libraries(rosload gcc_ssp_nt)
endif()

if(RUNTIME_CHECKS)
    target_link_libraries(rosload runtmchk)
endif()

add_dependencies(rosload bugcodes asm xdk)

add_cd_file(TARGET rosload DESTINATION loader NO_CAB FOR bootcd regtest livecd hybridcd)
