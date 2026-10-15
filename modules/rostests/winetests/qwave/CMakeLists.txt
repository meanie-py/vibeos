
remove_definitions(-D_WIN32_WINNT=0x502)
add_definitions(-D_WIN32_WINNT=0x0A00)

add_executable(qwave_winetest qos.c testlist.c)
target_compile_definitions(qwave_winetest PRIVATE __WINESRC__)
set_module_type(qwave_winetest win32cui)
add_importlibs(qwave_winetest qwave ws2_32 msvcrt kernel32)
add_rostests_file(TARGET qwave_winetest)
