# vibeos wow64.dll — Win10+-model WOW64 host layer skeleton (arm64-only today;
# Phase E reuses on amd64). See docs/engineering/wow64-plan.md. The exports are
# arch-gated in wow64.spec, so building this on x86 produces an empty-export
# DLL; only ship it where it means something.
if(ARCH STREQUAL "arm64")

spec2def(wow64.dll wow64.spec ADD_IMPORTLIB)

add_library(wow64 MODULE
    wow64.c
    wow64.rc
    ${CMAKE_CURRENT_BINARY_DIR}/wow64.def)

set_module_type(wow64 win32dll ENTRYPOINT DllMain 12)
add_importlibs(wow64 kernel32 ntdll)
add_dependencies(wow64 psdk)
add_cd_file(TARGET wow64 DESTINATION reactos/system32 FOR all)

endif()
