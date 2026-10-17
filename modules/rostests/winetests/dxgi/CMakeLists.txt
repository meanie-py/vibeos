
# Wine's dxgi tests at the DLL's own sync point (wine-10.1-dev b00c7619f1b).
include_directories(BEFORE ${REACTOS_SOURCE_DIR}/sdk/include/wine)
add_definitions(-DUSE_WINE_TODOS -DWINETEST_USE_DBGSTR_LONGLONG -DWIDL_C_INLINE_WRAPPERS)
add_executable(dxgi_winetest dxgi.c testlist.c)
target_link_libraries(dxgi_winetest uuid dxguid dx10guid)
set_module_type(dxgi_winetest win32cui)
add_importlibs(dxgi_winetest d3d10_1 dxgi user32 msvcrt kernel32 ntdll)
add_rostests_file(TARGET dxgi_winetest)
