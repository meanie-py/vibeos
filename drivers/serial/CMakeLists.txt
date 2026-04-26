
add_subdirectory(serenum)
add_subdirectory(serial)
if(ARCH STREQUAL "arm64")
    add_subdirectory(pl011)
endif()
