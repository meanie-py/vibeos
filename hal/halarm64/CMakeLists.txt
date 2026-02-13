
spec2def(hal.dll ../hal.spec ADD_IMPORTLIB)

add_definitions(
    -D_NTHALDLL_
    -D_NTHAL_)

include_directories(
    generic
    ${REACTOS_SOURCE_DIR}/ntoskrnl/include)

list(APPEND SOURCES
    generic/halstubs.c
    ${CMAKE_CURRENT_BINARY_DIR}/hal.def)

add_library(hal MODULE ${SOURCES})

set_module_type(hal kerneldll ENTRYPOINT HalInitSystem 8)
add_importlibs(hal ntoskrnl)
add_dependencies(hal psdk)
add_cd_file(TARGET hal DESTINATION reactos/system32 FOR all)
