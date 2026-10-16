
remove_definitions(-D_WIN32_WINNT=0x502)
add_definitions(-D_WIN32_WINNT=0x0A00)

add_executable(wevtapi_winetest wevtapi.c testlist.c)
target_compile_definitions(wevtapi_winetest PRIVATE __WINESRC__)
set_module_type(wevtapi_winetest win32cui)
add_importlibs(wevtapi_winetest wevtapi advapi32 msvcrt kernel32)
add_rostests_file(TARGET wevtapi_winetest)
