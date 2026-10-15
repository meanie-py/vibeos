
remove_definitions(-D_WIN32_WINNT=0x502)
add_definitions(-D_WIN32_WINNT=0x600 -D__WINESRC__)

spec2def(xinput1_4.dll xinput1_4.spec)

# The same source as xinput1_3 (Wine's PARENTSRC), with xinput1_4's ordinal exports:
# XInputGetCapabilitiesEx (#108) and XInputGetAudioDeviceIds (#10, audio.c) are 1_4 only.
list(APPEND SOURCE
    ../xinput1_3/main.c
    audio.c
    ${CMAKE_CURRENT_BINARY_DIR}/xinput1_4.def)

add_library(xinput1_4 MODULE ${SOURCE} version.rc)
set_module_type(xinput1_4 win32dll)
target_link_libraries(xinput1_4 wine)
add_importlibs(xinput1_4 hid setupapi advapi32 user32 msvcrt kernel32 kernel32_vista ntdll)
add_cd_file(TARGET xinput1_4 DESTINATION reactos/system32 FOR all)
set_wine_module(xinput1_4)
