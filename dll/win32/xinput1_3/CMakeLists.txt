
remove_definitions(-D_WIN32_WINNT=0x502)
add_definitions(-D_WIN32_WINNT=0x600 -D__WINESRC__)

spec2def(xinput1_3.dll xinput1_3.spec ADD_IMPORTLIB)

# Wine 11.19's HID implementation (dlls/xinput1_3/main.c); xinput1_4 builds the same
# source with its own spec, as Wine does.
list(APPEND SOURCE
    main.c
    ${CMAKE_CURRENT_BINARY_DIR}/xinput1_3.def)

add_library(xinput1_3 MODULE ${SOURCE} version.rc)
set_module_type(xinput1_3 win32dll)
target_link_libraries(xinput1_3 wine)
add_importlibs(xinput1_3 hid setupapi advapi32 user32 msvcrt kernel32 kernel32_vista ntdll)
add_cd_file(TARGET xinput1_3 DESTINATION reactos/system32 FOR all)
set_wine_module(xinput1_3)
