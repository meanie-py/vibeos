
spec2def(msvcrt20.dll msvcrt20.spec)

# msvcrt20.c is msvcrt-crt-rust src/legacy.rs (MSVCRT20__getmainargs/__wgetmainargs).
add_library(msvcrt20 MODULE
    ${CMAKE_CURRENT_BINARY_DIR}/msvcrt20_stubs.c
    ${CMAKE_CURRENT_BINARY_DIR}/msvcrt20.def)

set_module_type(msvcrt20 win32dll ENTRYPOINT DllMain 12)
add_dependencies(msvcrt20 psdk)
include(${REACTOS_SOURCE_DIR}/dll/win32/msvcrt/msvcrt_rust_cohab.cmake)
link_msvcrt_rust_cohab(msvcrt20)
# The whole DLL is msvcrt-crt-rust (pseh-rt/setjmp-rt inside); one Rust graph, so no duplicate.
link_strict(msvcrt20)

add_importlibs(msvcrt20 kernel32 kernel32_vista ntdll)
add_delay_importlibs(msvcrt20 user32 advapi32)

add_cd_file(TARGET msvcrt20 DESTINATION reactos/system32 FOR all)
