# User-mode SEH boot self-test (crash-value lane) — the userland sibling of
# ntoskrnl/seh_poc.c. Compiled with REAL clang native SEH (-fms-extensions).
# i386's frame-based lowering is MSVC-env-only (the gnu target fails with
# L__ehtable$ — dummy-pseh-verdict.md §C), so there it is retargeted to
# i686-pc-windows-msvc the way the kmtests are, and _except_handler3 /
# _local_unwind2 come from libpseh.a's real i386/seh.s. Spawned by userinit at
# boot; prints [SEH-USER] … PASS/FAIL on the kernel debug serial.
add_executable(sehtest sehtest.c)
set_module_type(sehtest win32cui)
target_compile_options(sehtest PRIVATE -fms-extensions)
if(ARCH STREQUAL "i386")
    target_compile_options(sehtest PRIVATE --target=i686-pc-windows-msvc -fasync-exceptions)
    target_link_libraries(sehtest ${PSEH_LIB})
endif()
add_importlibs(sehtest msvcrt kernel32 ntdll)
add_cd_file(TARGET sehtest DESTINATION reactos/system32 FOR all)
