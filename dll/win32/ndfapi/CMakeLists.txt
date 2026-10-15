
remove_definitions(-D_WIN32_WINNT=0x502)
add_definitions(-D_WIN32_WINNT=0x0A00)
spec2def(ndfapi.dll ndfapi.spec ADD_IMPORTLIB)

list(APPEND SOURCE
    ndfapi_main.c
    ${CMAKE_CURRENT_BINARY_DIR}/ndfapi_stubs.c)

add_library(ndfapi MODULE
    ${SOURCE}
    ${CMAKE_CURRENT_BINARY_DIR}/ndfapi.def)

set_module_type(ndfapi win32dll)
target_link_libraries(ndfapi wine wine_dll_register wine_dll_canunload)
add_importlibs(ndfapi msvcrt kernel32 ntdll)
add_dependencies(ndfapi psdk)
add_cd_file(TARGET ndfapi DESTINATION reactos/system32 FOR all)
set_wine_module_FIXME(ndfapi)
