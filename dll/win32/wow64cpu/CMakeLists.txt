# vibeos wow64cpu.dll — native amd64 WOW64 CPU layer (Phase E). The heaven's-
# gate mode switch is x86-64-specific, so this module only exists on amd64.
if(ARCH STREQUAL "amd64")

spec2def(wow64cpu.dll wow64cpu.spec ADD_IMPORTLIB)

list(APPEND ASM_SOURCE wow64cpu.S)
add_asm_files(wow64cpu_asm ${ASM_SOURCE})

add_library(wow64cpu MODULE
    wow64cpu.c
    ${wow64cpu_asm}
    wow64cpu.rc
    ${CMAKE_CURRENT_BINARY_DIR}/wow64cpu.def)

set_module_type(wow64cpu win32dll ENTRYPOINT DllMain 12)
# ntdll for the syscall surface; wow64 for Wow64SystemServiceEx. A WoW64
# process must not pull 64-bit kernel32 through the CPU layer (Win10 parity).
add_importlibs(wow64cpu ntdll wow64)
add_dependencies(wow64cpu psdk)
add_cd_file(TARGET wow64cpu DESTINATION reactos/system32 FOR all)

endif()
