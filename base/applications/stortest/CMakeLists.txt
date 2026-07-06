
add_executable(stortest stortest.c)
set_module_type(stortest win32cui)
add_importlibs(stortest msvcrt kernel32 ntdll)
add_cd_file(TARGET stortest DESTINATION reactos/system32 FOR all)
