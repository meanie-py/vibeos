
include_directories(${REACTOS_SOURCE_DIR}/boot/freeldr)

if(ARCH STREQUAL "i386" OR ARCH STREQUAL "amd64")

    # UEFI-only: the pure-BIOS disk/ext boot sectors (dosmbr/ext/extldr) and the
    # experimental FATY.S generator were pruned — nothing executes a legacy MBR/VBR
    # under UEFI (firmware loads EFI/BOOT/boot*.efi by FAT path, not via boot code).
    #
    # fat/fat32 stay: their 512-byte FAT VBR is read back by `native-fatten -boot`
    # to stamp a valid boot sector into the UEFI efisys.bin FAT image, and bin2c'd
    # into installfreeldr (boot/freeldr/install). Neither VBR is *executed* on a
    # UEFI boot, but the image-build tooling still consumes the bytes.
    CreateBootSectorTarget(fat ${CMAKE_CURRENT_SOURCE_DIR}/fat.S ${CMAKE_CURRENT_BINARY_DIR}/fat.bin 7c00)
    CreateBootSectorTarget(fat32 ${CMAKE_CURRENT_SOURCE_DIR}/fat32.S ${CMAKE_CURRENT_BINARY_DIR}/fat32.bin 7c00)

    add_cd_file(TARGET fat DESTINATION loader NO_CAB FILE ${CMAKE_CURRENT_BINARY_DIR}/fat.bin FOR bootcd regtest)
    add_cd_file(TARGET fat32 DESTINATION loader NO_CAB FILE ${CMAKE_CURRENT_BINARY_DIR}/fat32.bin FOR bootcd regtest)
endif()
