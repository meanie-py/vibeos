
file(GLOB welcome_rc_deps res/*.*)
add_rc_deps(welcome.rc ${welcome_rc_deps})
# Every translation unit is Rust now (welcome-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(welcome_rust welcome-rust "" TRUE)
add_executable(welcome
    # welcome.c  # replaced by Rust (sdk/lib/rust/welcome-rust/src/welcome/welcome.rs)
    rust_cohab_stub.c
    welcome.rc)
add_dependencies(welcome rust_welcome_rust)
target_link_libraries(welcome ${welcome_rust_LIB})
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
