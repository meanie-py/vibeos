## efisys.bin

# EFI platform ID, used in environ/CMakelists.txt for bootmgfw filename naming also.
if(ARCH STREQUAL "amd64")
    set(EFI_PLATFORM_ID "x64")
elseif(ARCH STREQUAL "i386")
    set(EFI_PLATFORM_ID "ia32")
elseif(ARCH STREQUAL "arm64")
    set(EFI_PLATFORM_ID "aa64")
else()
    message(FATAL_ERROR "Unknown ARCH '" ${ARCH} "', cannot generate a valid UEFI boot filename.")
endif()

if(DEFINED EFI_PLATFORM_ID)
    # uefildr.efi is now a cargo-driven custom_target (no $<TARGET_FILE:>), so
    # reference its known output path. boot_images.cmake is include()'d from
    # boot/CMakeLists.txt, so CMAKE_CURRENT_BINARY_DIR == build-<arch>/boot and
    # the loader lands at freeldr/freeldr/uefildr.efi under it. `DEPENDS uefildr`
    # (the custom_target) keeps efisys rebuilding when the loader relinks.
    add_custom_target(efisys
        COMMAND native-fatten ${CMAKE_CURRENT_BINARY_DIR}/efisys.bin -format 2880 EFIBOOT
            -boot ${CMAKE_CURRENT_BINARY_DIR}/freeldr/bootsect/fat.bin
            -mkdir EFI -mkdir EFI/BOOT
            -add ${CMAKE_CURRENT_BINARY_DIR}/freeldr/freeldr/uefildr.efi EFI/BOOT/boot${EFI_PLATFORM_ID}.efi
        DEPENDS native-fatten fat uefildr
        VERBATIM)
endif()

# ISO image EFI boot parameters
set(ISO_EFI_BOOT_PARAMS)

# Create an 'empty' directory (guaranteed to be empty) to be able to add
# arbitrary empty directories to the ISO image using mkisofs.
file(MAKE_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR}/empty)

# Retrieve the full path to the generated 'efisys' target file. The legacy BIOS
# bootsector files (isombr/isoboot/isobtrt) were removed with the BIOS prune.
if(DEFINED EFI_PLATFORM_ID)
    set(_efisys_file  ${CMAKE_CURRENT_BINARY_DIR}/efisys.bin) # get_target_property(_efisys_file  efisys  LOCATION)
    # UEFI-only boot: the EFI entry is now the PRIMARY (and only) El Torito boot
    # entry, so it must NOT be prefixed with -eltorito-alt-boot (that flag only
    # separates a secondary entry from a preceding BIOS primary, which is gone).
    list(APPEND ISO_EFI_BOOT_PARAMS -eltorito-platform efi -eltorito-boot loader/efisys.bin -no-emul-boot)
endif()

# Create a mkisofs sort file to specify an explicit ordering for the boot files
# to place them at the beginning of the image (makes ISO image analysis easier).
# See vendor/schilytools/mkisofs/README.sort for more details.
# As the default file sort weight is '0', give the boot files sort weights >= 1.
# Note that it is sad that '-sort' does not work using grafted points, and as a
# result we need in particular to use the boot catalog file "path" mkisofs that
# mkisofs expects, that is, the boot catalog file name is appended to the first
# host-system path listed in the file list, whatever it is, and that does not
# work well if the first item is a graft point (and especially if it's a file
# and not a directory). To fix that, the trick is to use as the first file item
# the empty directory created earlier. This ensures that:
# - the boot catalog file path is meaningful;
# - since its contents are included by mkisofs in the root of the ISO image,
#   using the empty directory ensures that no extra unwanted files are added.
#
set(ISO_SORT_FILE_DATA "\
${CMAKE_CURRENT_BINARY_DIR}/empty/boot.catalog 4
")
if(DEFINED EFI_PLATFORM_ID)
    string(APPEND ISO_SORT_FILE_DATA "${_efisys_file} 1\n")
endif()
file(WRITE ${CMAKE_CURRENT_BINARY_DIR}/bootfiles.sort ${ISO_SORT_FILE_DATA})

# ISO image identifier names
set(ISO_MANUFACTURER "ReactOS Project") # For both the publisher and the preparer
set(ISO_VOLNAME      "ReactOS")         # For both the Volume ID and the Volume set ID


# Create user profile directories in the LiveImage
function(add_allusers_profile_dirs _image_filelist _rootdir)
    file(APPEND ${_image_filelist} "${_rootdir}/All Users/Application Data=${CMAKE_CURRENT_BINARY_DIR}/empty\n")
    file(APPEND ${_image_filelist} "${_rootdir}/All Users/Documents/My Music=${CMAKE_CURRENT_BINARY_DIR}/empty\n")
    file(APPEND ${_image_filelist} "${_rootdir}/All Users/Documents/My Pictures=${CMAKE_CURRENT_BINARY_DIR}/empty\n")
    file(APPEND ${_image_filelist} "${_rootdir}/All Users/Documents/My Videos=${CMAKE_CURRENT_BINARY_DIR}/empty\n")
    file(APPEND ${_image_filelist} "${_rootdir}/All Users/Favorites=${CMAKE_CURRENT_BINARY_DIR}/empty\n")
    file(APPEND ${_image_filelist} "${_rootdir}/All Users/My Documents=${CMAKE_CURRENT_BINARY_DIR}/empty\n")
    file(APPEND ${_image_filelist} "${_rootdir}/All Users/Start Menu/Programs/StartUp=${CMAKE_CURRENT_BINARY_DIR}/empty\n")
    file(APPEND ${_image_filelist} "${_rootdir}/All Users/Templates=${CMAKE_CURRENT_BINARY_DIR}/empty\n")
endfunction()
function(add_user_profile_dirs _image_filelist _rootdir _username)
    file(APPEND ${_image_filelist} "${_rootdir}/${_username}/Application Data=${CMAKE_CURRENT_BINARY_DIR}/empty\n")
    file(APPEND ${_image_filelist} "${_rootdir}/${_username}/Application Data/Microsoft/Internet Explorer/Quick Launch=${CMAKE_CURRENT_BINARY_DIR}/empty\n")
    file(APPEND ${_image_filelist} "${_rootdir}/${_username}/Cookies=${CMAKE_CURRENT_BINARY_DIR}/empty\n")
    file(APPEND ${_image_filelist} "${_rootdir}/${_username}/Desktop=${CMAKE_CURRENT_BINARY_DIR}/empty\n")
    file(APPEND ${_image_filelist} "${_rootdir}/${_username}/Favorites=${CMAKE_CURRENT_BINARY_DIR}/empty\n")
    file(APPEND ${_image_filelist} "${_rootdir}/${_username}/Local Settings/Application Data=${CMAKE_CURRENT_BINARY_DIR}/empty\n")
    file(APPEND ${_image_filelist} "${_rootdir}/${_username}/Local Settings/History=${CMAKE_CURRENT_BINARY_DIR}/empty\n")
    file(APPEND ${_image_filelist} "${_rootdir}/${_username}/Local Settings/Temporary Internet Files=${CMAKE_CURRENT_BINARY_DIR}/empty\n")
    file(APPEND ${_image_filelist} "${_rootdir}/${_username}/My Music=${CMAKE_CURRENT_BINARY_DIR}/empty\n")
    file(APPEND ${_image_filelist} "${_rootdir}/${_username}/My Pictures=${CMAKE_CURRENT_BINARY_DIR}/empty\n")
    file(APPEND ${_image_filelist} "${_rootdir}/${_username}/My Videos=${CMAKE_CURRENT_BINARY_DIR}/empty\n")
    file(APPEND ${_image_filelist} "${_rootdir}/${_username}/NetHood=${CMAKE_CURRENT_BINARY_DIR}/empty\n")
    file(APPEND ${_image_filelist} "${_rootdir}/${_username}/PrintHood=${CMAKE_CURRENT_BINARY_DIR}/empty\n")
    file(APPEND ${_image_filelist} "${_rootdir}/${_username}/Recent=${CMAKE_CURRENT_BINARY_DIR}/empty\n")
    file(APPEND ${_image_filelist} "${_rootdir}/${_username}/SendTo=${CMAKE_CURRENT_BINARY_DIR}/empty\n")
    file(APPEND ${_image_filelist} "${_rootdir}/${_username}/Start Menu/Programs=${CMAKE_CURRENT_BINARY_DIR}/empty\n")
    file(APPEND ${_image_filelist} "${_rootdir}/${_username}/Start Menu/Programs/Administrative Tools=${CMAKE_CURRENT_BINARY_DIR}/empty\n")
    file(APPEND ${_image_filelist} "${_rootdir}/${_username}/Start Menu/Programs/StartUp=${CMAKE_CURRENT_BINARY_DIR}/empty\n")
    file(APPEND ${_image_filelist} "${_rootdir}/${_username}/Templates=${CMAKE_CURRENT_BINARY_DIR}/empty\n")
endfunction()


## BootCD
# Create the file list
file(WRITE ${CMAKE_CURRENT_BINARY_DIR}/bootcd.cmake.lst "")
file(WRITE ${CMAKE_CURRENT_BINARY_DIR}/bootcd.cmake.lst "${CMAKE_CURRENT_BINARY_DIR}/empty\n")

# Same debug-strip pass as livecd below (reactos.cab contents are NOT stripped
# — cabman compresses from the .dff list; only direct NO_CAB grafts are).
add_custom_target(bootcd
    COMMAND ${CMAKE_COMMAND} -E env python3 ${REACTOS_SOURCE_DIR}/tools/strip_cd_media.py
        --lst ${CMAKE_CURRENT_BINARY_DIR}/bootcd.$<CONFIG>.lst
        --out ${CMAKE_CURRENT_BINARY_DIR}/bootcd.stripped.lst
        --cache ${CMAKE_CURRENT_BINARY_DIR}/stripped
        --objcopy ${CMAKE_OBJCOPY}
    COMMAND native-mkisofs -quiet -o ${REACTOS_BINARY_DIR}/bootcd.iso -iso-level 4
        -publisher ${ISO_MANUFACTURER} -preparer ${ISO_MANUFACTURER} -volid ${ISO_VOLNAME} -volset ${ISO_VOLNAME}
        ${ISO_EFI_BOOT_PARAMS} -hide boot.catalog
        -sort ${CMAKE_CURRENT_BINARY_DIR}/bootfiles.sort
        -no-cache-inodes -graft-points -path-list ${CMAKE_CURRENT_BINARY_DIR}/bootcd.stripped.lst
    DEPENDS native-mkisofs
    VERBATIM)

## BootCDRegTest
# Create the file list
file(WRITE ${CMAKE_CURRENT_BINARY_DIR}/bootcdregtest.cmake.lst "")
file(WRITE ${CMAKE_CURRENT_BINARY_DIR}/bootcdregtest.cmake.lst "${CMAKE_CURRENT_BINARY_DIR}/empty\n")

add_custom_target(bootcdregtest
    COMMAND ${CMAKE_COMMAND} -E env python3 ${REACTOS_SOURCE_DIR}/tools/strip_cd_media.py
        --lst ${CMAKE_CURRENT_BINARY_DIR}/bootcdregtest.$<CONFIG>.lst
        --out ${CMAKE_CURRENT_BINARY_DIR}/bootcdregtest.stripped.lst
        --cache ${CMAKE_CURRENT_BINARY_DIR}/stripped
        --objcopy ${CMAKE_OBJCOPY}
    COMMAND native-mkisofs -quiet -o ${REACTOS_BINARY_DIR}/bootcdregtest.iso -iso-level 4
        -publisher ${ISO_MANUFACTURER} -preparer ${ISO_MANUFACTURER} -volid ${ISO_VOLNAME} -volset ${ISO_VOLNAME}
        ${ISO_EFI_BOOT_PARAMS} -hide boot.catalog
        -sort ${CMAKE_CURRENT_BINARY_DIR}/bootfiles.sort
        -no-cache-inodes -graft-points -path-list ${CMAKE_CURRENT_BINARY_DIR}/bootcdregtest.stripped.lst
    DEPENDS native-mkisofs
    VERBATIM)

## LiveCD
# Create the file list
file(WRITE ${CMAKE_CURRENT_BINARY_DIR}/livecd.cmake.lst "")
file(WRITE ${CMAKE_CURRENT_BINARY_DIR}/livecd.cmake.lst "${CMAKE_CURRENT_BINARY_DIR}/empty\n")

# Create TEMP dir
file(APPEND ${CMAKE_CURRENT_BINARY_DIR}/livecd.cmake.lst "reactos/TEMP=${CMAKE_CURRENT_BINARY_DIR}/empty\n")

# Create user profile directories
add_allusers_profile_dirs(${CMAKE_CURRENT_BINARY_DIR}/livecd.cmake.lst "Profiles")
add_user_profile_dirs(${CMAKE_CURRENT_BINARY_DIR}/livecd.cmake.lst "Profiles" "Default User")

# Graft debug-stripped PE twins onto the CD instead of the build binaries:
# the boot set is 70-92% DWARF that the PE loader never maps but freeldr
# streams off the CD at ~2.2MB/s (see docs/engineering/performance-investigation.md).
# The build tree keeps full DWARF — every sym tool reads build-dir binaries.
if(ARCH STREQUAL "arm64")
    # arm64 boots a FAT ramdisk, not an ISO: qemu/mkramdisk.sh lays this same
    # livecd.$<CONFIG>.lst out into ramdisk.img (tools/ramdisk_tree.py), so here
    # `livecd` only has to BUILD everything the list names. Every add_cd_file(FOR
    # livecd) adds its target as a dependency of this one, exactly as on x86.
    add_custom_target(livecd)
else()
    add_custom_target(livecd
        COMMAND ${CMAKE_COMMAND} -E env python3 ${REACTOS_SOURCE_DIR}/tools/strip_cd_media.py
            --lst ${CMAKE_CURRENT_BINARY_DIR}/livecd.$<CONFIG>.lst
            --out ${CMAKE_CURRENT_BINARY_DIR}/livecd.stripped.lst
            --cache ${CMAKE_CURRENT_BINARY_DIR}/stripped
            --objcopy ${CMAKE_OBJCOPY}
        COMMAND native-mkisofs -quiet -o ${REACTOS_BINARY_DIR}/livecd.iso -iso-level 4
            -publisher ${ISO_MANUFACTURER} -preparer ${ISO_MANUFACTURER} -volid ${ISO_VOLNAME} -volset ${ISO_VOLNAME}
            ${ISO_EFI_BOOT_PARAMS} -hide boot.catalog
            -sort ${CMAKE_CURRENT_BINARY_DIR}/bootfiles.sort
            -no-cache-inodes -graft-points -path-list ${CMAKE_CURRENT_BINARY_DIR}/livecd.stripped.lst
        DEPENDS native-mkisofs
        VERBATIM)
endif()

## HybridCD
# Create the file list
file(WRITE ${CMAKE_CURRENT_BINARY_DIR}/hybridcd.cmake.lst "")
file(WRITE ${CMAKE_CURRENT_BINARY_DIR}/hybridcd.cmake.lst "${CMAKE_CURRENT_BINARY_DIR}/empty\n")

# Create user profile directories
add_allusers_profile_dirs(${CMAKE_CURRENT_BINARY_DIR}/hybridcd.cmake.lst "livecd/Profiles")
add_user_profile_dirs(${CMAKE_CURRENT_BINARY_DIR}/hybridcd.cmake.lst "livecd/Profiles" "Default User")

add_custom_target(hybridcd
    COMMAND native-mkisofs -quiet -o ${REACTOS_BINARY_DIR}/hybridcd.iso -iso-level 4
        -publisher ${ISO_MANUFACTURER} -preparer ${ISO_MANUFACTURER} -volid ${ISO_VOLNAME} -volset ${ISO_VOLNAME}
        ${ISO_EFI_BOOT_PARAMS} -hide boot.catalog
        -sort ${CMAKE_CURRENT_BINARY_DIR}/bootfiles.sort
        -duplicates-once -no-cache-inodes -graft-points -path-list ${CMAKE_CURRENT_BINARY_DIR}/hybridcd.$<CONFIG>.lst
    DEPENDS bootcd livecd
    VERBATIM)

if(DEFINED EFI_PLATFORM_ID)
    # For things like flashing USB drives, we also add the efi file into efi/boot.
    # The El Torito EFI image is for CD boot; the arm64 ramdisk's boot.img carries
    # BOOTAA64.EFI itself, and efisys needs the BIOS FAT boot sector arm64 never builds.
    if(ARCH STREQUAL "arm64")
        add_cd_file(TARGET efisys FILE ${CMAKE_CURRENT_BINARY_DIR}/efisys.bin DESTINATION loader NO_CAB NOT_IN_HYBRIDCD FOR bootcd regtest hybridcd)
    else()
        add_cd_file(TARGET efisys FILE ${CMAKE_CURRENT_BINARY_DIR}/efisys.bin DESTINATION loader NO_CAB NOT_IN_HYBRIDCD FOR bootcd regtest livecd hybridcd)
    endif()

    # uefildr is a cargo-driven custom_target (no $<TARGET_FILE:>); pass TARGET
    # for the build dependency AND FILE for the explicit path (same pattern as
    # the efisys add_cd_file above).
    add_cd_file(
        TARGET uefildr
        FILE ${CMAKE_CURRENT_BINARY_DIR}/freeldr/freeldr/uefildr.efi
        DESTINATION efi/boot
        NO_CAB
        NAME_ON_CD boot${EFI_PLATFORM_ID}.efi
        FOR livecd hybridcd)
endif()
