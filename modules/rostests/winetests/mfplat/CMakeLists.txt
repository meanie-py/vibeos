
remove_definitions(-D_WIN32_WINNT=0x502)
add_definitions(-D_WIN32_WINNT=0x0A00)

add_executable(mfplat_winetest mfplat.c testlist.c resource.rc)
target_compile_definitions(mfplat_winetest PRIVATE __WINESRC__ WINETEST_USE_DBGSTR_LONGLONG)
target_link_libraries(mfplat_winetest mfuuid strmiids uuid dxguid)
set_module_type(mfplat_winetest win32cui)
add_importlibs(mfplat_winetest ole32 mfplat user32 d3d9 dxva2 propsys oleaut32 shlwapi msvcrt kernel32 kernel32_vista kernelbase ntdll)
add_dependencies(mfplat_winetest psdk wineheaders)
add_rostests_file(TARGET mfplat_winetest)
