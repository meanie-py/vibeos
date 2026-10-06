add_subdirectory(class)
# floppy (fdc/floppy/flpydisk) dropped: no floppy in QEMU q35/i440fx/virt.
add_subdirectory(ide)
add_subdirectory(mountmgr)
add_subdirectory(partmgr)
add_subdirectory(port)
# The SD host is found only through the arm64 HAL's ACPI PnP root.
if(ARCH STREQUAL "arm64")
    add_subdirectory(sdhc)
endif()
