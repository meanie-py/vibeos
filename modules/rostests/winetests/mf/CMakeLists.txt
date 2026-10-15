
remove_definitions(-D_WIN32_WINNT=0x502)
add_definitions(-D_WIN32_WINNT=0x0A00)

# Wine's mf conformance suite. transform.c is linked for the helpers mf.c shares with it
# (mf_test.h); its own tests need the codec MFTs we do not ship yet.
add_executable(mf_winetest mf.c topology.c transform.c d3d11_shim.c testlist.c resource.rc)
target_compile_definitions(mf_winetest PRIVATE __WINESRC__ WINETEST_USE_DBGSTR_LONGLONG WINETEST_WINE_DBG_STUBS)
target_link_libraries(mf_winetest dmoguids mfuuid strmiids uuid wmcodecdspuuid dxguid)
set_module_type(mf_winetest win32cui)
add_importlibs(mf_winetest mf mfplat ole32 user32 propsys msdmo d3d9 dxva2 msvfw32 winmm oleaut32 shlwapi msvcrt kernel32 kernel32_vista kernelbase ntdll)
add_dependencies(mf_winetest psdk wineheaders)
add_rostests_file(TARGET mf_winetest)
