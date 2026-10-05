
spec2def(crtdll.dll crtdll.spec ADD_IMPORTLIB)

add_definitions(
    -DUSE_MSVCRT_PREFIX
    -DCRTDLL)

include_directories(${REACTOS_SOURCE_DIR}/sdk/lib/crt/include)

# dllmain.c is msvcrt-crt-rust src/legacy.rs (CRTDLL_* names, which crtdll.spec maps).
list(APPEND SOURCE
    crtdll.rc
    ${CMAKE_CURRENT_BINARY_DIR}/crtdll_stubs.c
    ${CMAKE_CURRENT_BINARY_DIR}/crtdll.def)

add_library(crtdll MODULE ${SOURCE})
set_module_type(crtdll win32dll ENTRYPOINT DllMain 12)
include(${REACTOS_SOURCE_DIR}/dll/win32/msvcrt/msvcrt_rust_cohab.cmake)
link_msvcrt_rust_cohab(crtdll)
target_link_libraries(crtdll ${PSEH_USER_LIB} ${VCRT_SETJMP_LIB})
add_dependencies(crtdll psdk)

add_importlibs(crtdll kernel32 kernel32_vista ntdll)
# advapi32: the shared CRT's rand_s calls SystemFunction036 (RtlGenRandom). dllmain.c used to
# define an assert(FALSE) dummy for it; crtdll exports no rand_s, but msvcrt20/40 import it this way.
add_delay_importlibs(crtdll user32 advapi32)
add_cd_file(TARGET crtdll DESTINATION reactos/system32 FOR all)
