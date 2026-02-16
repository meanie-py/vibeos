
add_subdirectory(i8042prt)
add_subdirectory(inport)
add_subdirectory(kbdclass)
add_subdirectory(mouclass)
add_subdirectory(sermouse)

if(ARCH STREQUAL "arm64")
    add_subdirectory(arm64input)
endif()
