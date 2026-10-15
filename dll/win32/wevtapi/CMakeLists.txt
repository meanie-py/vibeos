
remove_definitions(-D_WIN32_WINNT=0x502)
add_definitions(-D_WIN32_WINNT=0x0A00)
spec2def(wevtapi.dll wevtapi.spec ADD_IMPORTLIB)

list(APPEND SOURCE
    main.c
    ${CMAKE_CURRENT_BINARY_DIR}/wevtapi_stubs.c)

add_library(wevtapi MODULE
    ${SOURCE}
    ${CMAKE_CURRENT_BINARY_DIR}/wevtapi.def)

set_module_type(wevtapi win32dll)
target_link_libraries(wevtapi wine)
add_importlibs(wevtapi msvcrt kernel32 ntdll)
add_dependencies(wevtapi psdk)
add_cd_file(TARGET wevtapi DESTINATION reactos/system32 FOR all)
set_wine_module_FIXME(wevtapi)
