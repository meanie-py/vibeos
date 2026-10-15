
remove_definitions(-D_WIN32_WINNT=0x502)
add_definitions(-D_WIN32_WINNT=0x0A00)
spec2def(tbs.dll tbs.spec ADD_IMPORTLIB)

list(APPEND SOURCE
    tbs.c
    ${CMAKE_CURRENT_BINARY_DIR}/tbs_stubs.c)

add_library(tbs MODULE
    ${SOURCE}
    ${CMAKE_CURRENT_BINARY_DIR}/tbs.def)

set_module_type(tbs win32dll)
target_link_libraries(tbs wine)
add_importlibs(tbs msvcrt kernel32 ntdll)
add_dependencies(tbs psdk)
add_cd_file(TARGET tbs DESTINATION reactos/system32 FOR all)
set_wine_module_FIXME(tbs)
