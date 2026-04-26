add_executable(testgui WIN32 testgui.c)
set_module_type(testgui win32gui UNICODE)
add_importlibs(testgui user32 gdi32 msvcrt kernel32 ntdll)
add_cd_file(TARGET testgui DESTINATION reactos/system32 FOR all)
