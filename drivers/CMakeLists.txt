
PROJECT(Drivers)

# acpi (cmbatt) + battery (battc/compbatt) dropped: ACPI/composite battery
# drivers, no battery in QEMU VMs, never load — pre-Vista-era optional hardware.
add_subdirectory(base)
# bluetooth (fbtusb USB BT dongle) dropped: niche, not emulated by QEMU.
add_subdirectory(bus)
add_subdirectory(crypto)
add_subdirectory(filesystems)
add_subdirectory(filters)
#add_subdirectory(fs_minifilter)
add_subdirectory(hid)
add_subdirectory(input)
add_subdirectory(ksfilter)
# multimedia (bdasup/bdaplgin Broadcast Driver Arch, TV tuner) dropped: niche,
# not emulated by QEMU.
add_subdirectory(network)
# parallel (parport) dropped: legacy LPT, not emulated by QEMU q35/i440fx/virt.
add_subdirectory(processor)
add_subdirectory(serial)
add_subdirectory(setup)
add_subdirectory(storage)
add_subdirectory(usb)
# drivers/video holds sysfb, the video driver over the framebuffer the firmware
# hands over (UEFI GOP). It is the display on every arch: the VGA/VBE miniports
# that legacy-BIOS boots used are gone (bdea86df186).
if((ARCH STREQUAL "arm64") OR (ARCH STREQUAL "i386") OR (ARCH STREQUAL "amd64"))
    add_subdirectory(video)
endif()
add_subdirectory(wdm)
add_subdirectory(wmi)
