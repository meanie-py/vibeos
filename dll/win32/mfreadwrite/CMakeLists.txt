
remove_definitions(-D_WIN32_WINNT=0x502)
add_definitions(-D_WIN32_WINNT=0x0A00)
spec2def(mfreadwrite.dll mfreadwrite.spec ADD_IMPORTLIB)

list(APPEND SOURCE
    reader.c
    writer.c
    ${CMAKE_CURRENT_BINARY_DIR}/mfreadwrite_stubs.c)

add_library(mfreadwrite MODULE
    ${SOURCE}
    ${CMAKE_CURRENT_BINARY_DIR}/mfreadwrite.def)

set_module_type(mfreadwrite win32dll)
add_idl_reg_scripts(mfreadwrite registry mf_classes.idl)
target_link_libraries(mfreadwrite wine wine_dll_register wine_dll_canunload uuid mfuuid oldnames)
add_delay_importlibs(mfreadwrite mf)
add_importlibs(mfreadwrite mfplat ole32 kernelbase msvcrt kernel32 kernel32_vista ntdll)
add_dependencies(mfreadwrite psdk wineheaders)
add_cd_file(TARGET mfreadwrite DESTINATION reactos/system32 FOR all)
set_wine_module_FIXME(mfreadwrite)
