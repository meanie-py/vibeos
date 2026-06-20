
add_executable(nettest nettest.c)
set_module_type(nettest win32cui)
add_importlibs(nettest ws2_32 msvcrt kernel32 ntdll)
add_cd_file(TARGET nettest DESTINATION reactos/system32 FOR all)
