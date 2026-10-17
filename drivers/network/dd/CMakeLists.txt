
# Legacy NICs dropped (ne2000 ISA, pcnet, dc21x4 DEC Tulip, nvnet proprietary):
# pre-Vista / not the QEMU-relevant cards. Kept: e1000 + rtl8139 (QEMU-emulated
# PCI) + netkvm (virtio) as the networking translation targets.
add_subdirectory(e1000)
# The Pis' Wi-Fi is found only through the arm64 HAL's ACPI PnP root.
if(ARCH STREQUAL "arm64")
    add_subdirectory(bwfm)
endif()
add_subdirectory(${REACTOS_SOURCE_DIR}/vendor/netkvm netkvm)
add_subdirectory(rtl8139)
