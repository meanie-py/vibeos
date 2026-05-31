
# Legacy video miniports (vga/vbe/bochs/vmx_svga) dropped: pre-Vista INT10/VGA
# BIOS-era display path. Under UEFI the display is the GOP linear framebuffer
# (arm64vid.sys boot-start -> framebuf.dll), which does not use videoprt INT10.
