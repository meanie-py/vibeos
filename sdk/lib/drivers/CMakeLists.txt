
add_subdirectory(arbiter)
add_subdirectory(aux_klib)
#add_subdirectory(copysup) # Nothing in this tree links it (2026-10-16).
add_subdirectory(csq)
add_subdirectory(hidparser)
add_subdirectory(ntoskrnl_vista)
#add_subdirectory(rdbsslib) # Nothing in this tree links it (2026-10-16).
add_subdirectory(rtlver)
#add_subdirectory(rxce) # Nothing in this tree links it (2026-10-16).
add_subdirectory(sound)
add_subdirectory(sptilib)
add_subdirectory(ucxstub)
add_subdirectory(usbdex)
add_subdirectory(${REACTOS_SOURCE_DIR}/vendor/virtio virtio)
add_subdirectory(${REACTOS_SOURCE_DIR}/vendor/wdf wdf)
add_subdirectory(wdfdriverentry)
add_subdirectory(wdfldr)
