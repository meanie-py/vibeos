
# serenum (serial-port PnP enumerator) dropped: its only purpose was probing
# serial mice / legacy serial PnP, gone with sermouse. serial (COM) kept.
add_subdirectory(serial)
if(ARCH STREQUAL "arm64")
    add_subdirectory(pl011)
endif()
