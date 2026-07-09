# User-mode SEH boot self-test (crash-value lane) — the userland sibling of
# ntoskrnl/seh_poc.c. Compiled with REAL clang native SEH (-fms-extensions),
# which only lowers on amd64/arm64 (i386's L__ehtable$ lowering is
# MSVC-env-only — dummy-pseh-verdict.md §C). Spawned by userinit at boot;
# prints [SEH-USER] … PASS/FAIL on the kernel debug serial.
if(ARCH STREQUAL "amd64" OR ARCH STREQUAL "arm64")
    add_executable(sehtest sehtest.c)
    set_module_type(sehtest win32cui)
    target_compile_options(sehtest PRIVATE -fms-extensions)
    add_importlibs(sehtest msvcrt kernel32 ntdll)
    add_cd_file(TARGET sehtest DESTINATION reactos/system32 FOR all)
endif()
