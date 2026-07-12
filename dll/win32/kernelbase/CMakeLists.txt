
# Minimal, Vista-scoped kernelbase.dll (vibeos) — the modern *synchronization*
# export home so api-ms-win-core-synch-* api-sets resolve for foreign binaries.
# NOT the full wine kernelbase (wine/ subdir, still stubbed — see below). Almost
# pure forwarders; only WaitOnAddress carries local code. See kernelbase.spec +
# dllmain.c + docs/engineering/fex-bringup-plan.md rung 2.

remove_definitions(-D_WIN32_WINNT=0x502 -DWINVER=0x502)
add_definitions(-D_WIN32_WINNT=0x600 -DWINVER=0x600)

spec2def(kernelbase.dll kernelbase.spec ADD_IMPORTLIB)

add_library(kernelbase MODULE
    dllmain.c
    kernelbase.rc
    ${CMAKE_CURRENT_BINARY_DIR}/kernelbase.def)

set_module_type(kernelbase win32dll ENTRYPOINT DllMain 12)
# Import libs for the local C in dllmain.c (SetLastError -> kernel32;
# RtlWaitOnAddress -> ntdll_vista, the 0x600 export home; RtlNtStatusToDosError
# -> base ntdll). The spec forwarders to kernel32_vista/ntdll_vista are resolved
# by the loader at runtime, so they need no link-time import lib here.
add_importlibs(kernelbase kernel32 ntdll_vista ntdll)
add_dependencies(kernelbase psdk)
add_cd_file(TARGET kernelbase DESTINATION reactos/system32 FOR all)

# The upstream wine kernelbase (wine/) remains out of the build:
#   TODO: ReactOS Isn't ready for the full Kernelbase.dll
#add_subdirectory(wine)
