
add_subdirectory(csr)
# Virtual DOS Machine is only for x86 architectures
if(NOT ARCH STREQUAL "arm64")
    add_subdirectory(mvdm)
endif()
add_subdirectory(win)
