
include_directories(${REACTOS_SOURCE_DIR}/boot/freeldr)

if(ARCH STREQUAL "i386" OR ARCH STREQUAL "amd64")

    CreateBootSectorTarget(dosmbr ${CMAKE_CURRENT_SOURCE_DIR}/dosmbr.S ${CMAKE_CURRENT_BINARY_DIR}/dosmbr.bin 7c00)
    CreateBootSectorTarget(ext ${CMAKE_CURRENT_SOURCE_DIR}/ext.S ${CMAKE_CURRENT_BINARY_DIR}/ext.bin 7c00)
    CreateBootSectorTarget(extldr ${CMAKE_CURRENT_SOURCE_DIR}/extldr.S ${CMAKE_CURRENT_BINARY_DIR}/extldr.sys 8000)

    CreateBootSectorTarget(fat ${CMAKE_CURRENT_SOURCE_DIR}/fat.S ${CMAKE_CURRENT_BINARY_DIR}/fat.bin 7c00)
    CreateBootSectorTarget(fat32 ${CMAKE_CURRENT_SOURCE_DIR}/fat32.S ${CMAKE_CURRENT_BINARY_DIR}/fat32.bin 7c00)
    # ntfs/btrfs VBR templates removed — NTFS/btrfs install-boot dropped

    ## New versions using FATY.S (experimental)
    # add_definitions(-DFAT12)
    # CreateBootSectorTarget(fat_new ${CMAKE_CURRENT_SOURCE_DIR}/faty.S ${CMAKE_CURRENT_BINARY_DIR}/fat_new.bin 7c00)
    # remove_definitions(-DFAT12)
    # add_definitions(-DFAT16)
    # CreateBootSectorTarget(fat16_new ${CMAKE_CURRENT_SOURCE_DIR}/faty.S ${CMAKE_CURRENT_BINARY_DIR}/fat16_new.bin 7c00)
    # remove_definitions(-DFAT16)
    # add_definitions(-DFAT32)
    # CreateBootSectorTarget(fat32_new ${CMAKE_CURRENT_SOURCE_DIR}/faty.S ${CMAKE_CURRENT_BINARY_DIR}/fat32_new.bin 7c00)
    # remove_definitions(-DFAT32)

    # isoboot/isobtrt/isombr (BIOS El Torito boot sector + isohybrid MBR) removed
    # with the BIOS prune — the ISO boots UEFI-only via the EFI El Torito entry
    # (efisys.bin). dosmbr/ext/extldr stay (legacy BIOS disk-boot sectors,
    # bootcd-only, not on the booted livecd); fat/fat32 stay because their FAT
    # VBR is reused to synthesize the UEFI efisys.bin FAT image.

    add_cd_file(TARGET dosmbr DESTINATION loader NO_CAB FILE ${CMAKE_CURRENT_BINARY_DIR}/dosmbr.bin FOR bootcd regtest)
    add_cd_file(TARGET ext DESTINATION loader NO_CAB FILE ${CMAKE_CURRENT_BINARY_DIR}/ext.bin FOR bootcd regtest)
    add_cd_file(TARGET extldr DESTINATION loader NO_CAB FILE ${CMAKE_CURRENT_BINARY_DIR}/extldr.sys FOR bootcd regtest)
    add_cd_file(TARGET fat DESTINATION loader NO_CAB FILE ${CMAKE_CURRENT_BINARY_DIR}/fat.bin FOR bootcd regtest)
    add_cd_file(TARGET fat32 DESTINATION loader NO_CAB FILE ${CMAKE_CURRENT_BINARY_DIR}/fat32.bin FOR bootcd regtest)
endif()
