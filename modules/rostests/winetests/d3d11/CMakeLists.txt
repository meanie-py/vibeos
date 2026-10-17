
# Wine's d3d11 tests at the DLL's own sync point (wine-10.1-dev b00c7619f1b).
include_directories(BEFORE ${REACTOS_SOURCE_DIR}/sdk/include/wine)
add_definitions(-DUSE_WINE_TODOS -D_USE_MATH_DEFINES -DWINETEST_USE_DBGSTR_LONGLONG -DWINETEST_WINE_DBG_STUBS)
add_executable(d3d11_winetest d3d11.c testlist.c)
target_link_libraries(d3d11_winetest uuid dxguid dx10guid)
set_module_type(d3d11_winetest win32cui)
add_importlibs(d3d11_winetest d3d11 dxgi d3dcompiler_47 user32 gdi32 msvcrt kernel32 ntdll)
add_rostests_file(TARGET d3d11_winetest)
