
include_directories(${REACTOS_SOURCE_DIR}/boot/freeldr)

if(ARCH STREQUAL "i386" OR ARCH STREQUAL "amd64")

    # Legacy BIOS boot (restored 2026-10-05 for Libreboot/SeaBIOS machines): the
    # El Torito no-emulation boot sector, its regtest twin, the isohybrid MBR that
    # makes the ISO bootable when dd'd to a USB stick, and the MBR for installed
    # disks. fat/fat32 also serve UEFI: `native-fatten -boot` stamps their VBR into
    # efisys.bin, and installfreeldr bin2c's them.
    CreateBootSectorTarget(dosmbr ${CMAKE_CURRENT_SOURCE_DIR}/dosmbr.S ${CMAKE_CURRENT_BINARY_DIR}/dosmbr.bin 7c00)
    CreateBootSectorTarget(isoboot ${CMAKE_CURRENT_SOURCE_DIR}/isoboot.S ${CMAKE_CURRENT_BINARY_DIR}/isoboot.bin 7000)
    CreateBootSectorTarget(isobtrt ${CMAKE_CURRENT_SOURCE_DIR}/isobtrt.S ${CMAKE_CURRENT_BINARY_DIR}/isobtrt.bin 7000)
    CreateBootSectorTarget(isombr ${CMAKE_CURRENT_SOURCE_DIR}/isombr.S ${CMAKE_CURRENT_BINARY_DIR}/isombr.bin 7000)
    CreateBootSectorTarget(fat ${CMAKE_CURRENT_SOURCE_DIR}/fat.S ${CMAKE_CURRENT_BINARY_DIR}/fat.bin 7c00)
    CreateBootSectorTarget(fat32 ${CMAKE_CURRENT_SOURCE_DIR}/fat32.S ${CMAKE_CURRENT_BINARY_DIR}/fat32.bin 7c00)

    add_cd_file(TARGET dosmbr DESTINATION loader NO_CAB FILE ${CMAKE_CURRENT_BINARY_DIR}/dosmbr.bin FOR bootcd regtest)
    add_cd_file(TARGET isoboot DESTINATION loader NO_CAB NOT_IN_HYBRIDCD FILE ${CMAKE_CURRENT_BINARY_DIR}/isoboot.bin FOR all hybridcd)
    add_cd_file(TARGET isobtrt DESTINATION loader NO_CAB NOT_IN_HYBRIDCD FILE ${CMAKE_CURRENT_BINARY_DIR}/isobtrt.bin FOR bootcd regtest)
    add_cd_file(TARGET isombr DESTINATION loader NO_CAB NOT_IN_HYBRIDCD FILE ${CMAKE_CURRENT_BINARY_DIR}/isombr.bin FOR all hybridcd)
    add_cd_file(TARGET fat DESTINATION loader NO_CAB FILE ${CMAKE_CURRENT_BINARY_DIR}/fat.bin FOR bootcd regtest)
    add_cd_file(TARGET fat32 DESTINATION loader NO_CAB FILE ${CMAKE_CURRENT_BINARY_DIR}/fat32.bin FOR bootcd regtest)
endif()
