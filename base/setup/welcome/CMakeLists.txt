
file(GLOB welcome_rc_deps res/*.*)
add_rc_deps(welcome.rc ${welcome_rc_deps})
add_executable(welcome welcome.c welcome.rc)
set_module_type(welcome win32gui UNICODE)
add_importlibs(welcome gdi32 user32 shell32 msvcrt kernel32 ntdll)

# Reduce the required subsystem to WinNT 4.0 for i386 builds only.
if(ARCH STREQUAL "i386")
    # The binutils linker does not set a lower limit on the subsystem.
    # Otherwise we would use: objcopy --subsystem windows:4.00 $<TARGET_FILE:welcome>
    target_link_options(welcome PRIVATE "-Wl,--subsystem,windows:4.00")
endif()

add_cd_file(TARGET welcome DESTINATION reactos FOR all)

# welcome.exe renamed as setup.exe for the BootCD
add_cd_file(TARGET welcome DESTINATION root NO_CAB NAME_ON_CD setup.exe FOR bootcd)
