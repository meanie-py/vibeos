
remove_definitions(-D_WIN32_WINNT=0x502)
add_definitions(-D_WIN32_WINNT=0x0A00)
spec2def(mfplat.dll mfplat.spec ADD_IMPORTLIB)

list(APPEND SOURCE
    buffer.c
    main.c
    mediatype.c
    queue.c
    sample.c
    ${CMAKE_CURRENT_BINARY_DIR}/mfplat_stubs.c)

add_library(mfplat MODULE
    ${SOURCE}
    ${CMAKE_CURRENT_BINARY_DIR}/mfplat.def)

set_module_type(mfplat win32dll)
target_link_libraries(mfplat wine uuid dmoguids mfuuid strmiids dxguid oldnames)
add_delay_importlibs(mfplat bcrypt)
add_importlibs(mfplat advapi32 ole32 oleaut32 propsys rtworkq user32 shlwapi kernelbase msvcrt kernel32 kernel32_vista ntdll)
add_dependencies(mfplat psdk wineheaders)
add_cd_file(TARGET mfplat DESTINATION reactos/system32 FOR all)
set_wine_module_FIXME(mfplat)
