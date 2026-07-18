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

# E3: stage the i386 SysWOW64 guest set onto the amd64 livecd — the amd64
# mirror of qemu/mkramdisk.sh's VIBEOS_FEX staging on arm64. The files come
# from SIBLING build trees (build-i386 + build-fex-hello) via the canonical
# path, so this is inherently cross-build: an amd64 configure only stages when
# the i386 artifacts already exist (build i386 first). Opportunistic like the
# arm64 probe — winlogon's TryStartWow64Hello is a silent no-op when absent —
# but the skip itself is loud. All-or-nothing: a partial guest set produces a
# confusing mid-loader stall instead of a clean probe skip.
set(_wow64_guest_files
    ${REACTOS_SOURCE_DIR}/build-i386/dll/ntdll/ntdll.dll
    ${REACTOS_SOURCE_DIR}/build-i386/dll/win32/kernel32/kernel32.dll
    ${REACTOS_SOURCE_DIR}/build-i386/dll/ntdll/nt_0600/ntdll_vista.dll
    ${REACTOS_SOURCE_DIR}/build-fex-hello/hello32.exe)
set(_wow64_guest_ok TRUE)
foreach(_f ${_wow64_guest_files})
    if(NOT EXISTS ${_f})
        set(_wow64_guest_ok FALSE)
        message(STATUS "WOW64 E3: ${_f} missing — syswow64 NOT staged on livecd (hello32 probe will no-op)")
    endif()
endforeach()
if(_wow64_guest_ok)
    message(STATUS "WOW64 E3: staging i386 syswow64 guest set onto livecd")
    foreach(_f ${_wow64_guest_files})
        add_cd_file(FILE ${_f} DESTINATION reactos/syswow64 FOR livecd)
    endforeach()
endif()

endif()
