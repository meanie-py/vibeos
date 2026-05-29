
add_subdirectory(acpi)
# isapnp + pcmcia dropped: pre-Vista ISA-PnP / PCMCIA bus drivers, dead under
# UEFI/ACPI (QEMU enumerates legacy devices via ACPI, not ISA read-data-port).
add_subdirectory(pci)
add_subdirectory(pcix)
