
add_definitions(-DUSE_WINE_TODOS)

spec2def(testdll.dll testdll.spec)

add_library(imagehlp_testdll MODULE testdll.c ${CMAKE_CURRENT_BINARY_DIR}/testdll.def)
target_link_libraries(imagehlp_testdll wine)
set_module_type(imagehlp_testdll win32dll IMAGEBASE default)
add_importlibs(imagehlp_testdll ole32 oleaut32 shlwapi comdlg32 user32 msvcrt kernel32 ntdll)
add_rostests_file(TARGET imagehlp_testdll)
set_source_files_properties(imagehlp.rc OBJECT_DEPENDS imagehlp_testdll)

list(APPEND SOURCE
    image.c
    integrity.c
    testlist.c
    imagehlp.rc)

add_executable(imagehlp_winetest ${SOURCE})
set_module_type(imagehlp_winetest win32cui)
add_importlibs(imagehlp_winetest imagehlp psapi msvcrt kernel32 ntdll)
add_rostests_file(TARGET imagehlp_winetest)
