
add_definitions(-D__USE_W32_SOCKETS)
add_executable(arping arping.c arping.rc)
set_module_type(arping win32cui UNICODE)
add_importlibs(arping user32 ws2_32 iphlpapi msvcrt kernel32)

add_cd_file(TARGET arping DESTINATION reactos/system32 FOR all)
