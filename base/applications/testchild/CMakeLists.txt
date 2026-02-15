add_executable(testchild testchild.c)
set_module_type(testchild win32cui)
add_importlibs(testchild msvcrt kernel32 ntdll)
add_cd_file(TARGET testchild DESTINATION reactos/system32 FOR all)
