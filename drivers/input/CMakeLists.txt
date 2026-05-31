
add_subdirectory(i8042prt)
add_subdirectory(kbdclass)
add_subdirectory(mouclass)
# inport (MS InPort bus mouse) + sermouse (serial mouse) dropped: pre-PS/2
# legacy pointers, never emulated. QEMU input is PS/2 (i8042prt) / virtio.

if(ARCH STREQUAL "arm64")
    add_subdirectory(arm64input)
endif()
