
remove_definitions(-D_WIN32_WINNT=0x502)
add_definitions(-D_WIN32_WINNT=0x0A00)
spec2def(qwave.dll qwave.spec ADD_IMPORTLIB)

list(APPEND SOURCE
    main.c
    ${CMAKE_CURRENT_BINARY_DIR}/qwave_stubs.c)

add_library(qwave MODULE
    ${SOURCE}
    ${CMAKE_CURRENT_BINARY_DIR}/qwave.def)

set_module_type(qwave win32dll)
target_link_libraries(qwave wine)
add_importlibs(qwave msvcrt kernel32 ntdll)
add_dependencies(qwave psdk)
add_cd_file(TARGET qwave DESTINATION reactos/system32 FOR all)
set_wine_module_FIXME(qwave)
