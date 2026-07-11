# vibeos wow64.dll — Win10+-model WOW64 host layer skeleton (arm64-only today;
# Phase E reuses on amd64). See docs/engineering/wow64-plan.md. The exports are
# arch-gated in wow64.spec, so building this on x86 produces an empty-export
# DLL; only ship it where it means something.
if(ARCH STREQUAL "arm64")

spec2def(wow64.dll wow64.spec ADD_IMPORTLIB)

# Phase D: the generated 32->64 syscall thunk table (wow64gen, deterministic
# spec2def-class codegen — see tools/wow64gen/README.md). Indexed by REAL
# service number (sysfuncs.h order == what guest EAX carries); ntdll.spec
# contributes argument types only. wow64.c #includes the output.
set(_wow64_thunks ${CMAKE_CURRENT_BINARY_DIR}/wow64_thunks.gen.c)
add_custom_command(
    OUTPUT ${_wow64_thunks}
    COMMAND ${CMAKE_COMMAND} -E env python3
        ${REACTOS_SOURCE_DIR}/tools/wow64gen/wow64gen.py
        --spec ${REACTOS_SOURCE_DIR}/dll/ntdll/def/ntdll.spec
        --arch ${ARCH}
        --sysfuncs ${REACTOS_SOURCE_DIR}/ntoskrnl/include/sysfuncs.h
        --manual-list ${REACTOS_SOURCE_DIR}/tools/wow64gen/manual.txt
        --out-c ${_wow64_thunks}
    DEPENDS
        ${REACTOS_SOURCE_DIR}/tools/wow64gen/wow64gen.py
        ${REACTOS_SOURCE_DIR}/dll/ntdll/def/ntdll.spec
        ${REACTOS_SOURCE_DIR}/ntoskrnl/include/sysfuncs.h
        ${REACTOS_SOURCE_DIR}/tools/wow64gen/manual.txt
    COMMENT "wow64gen: generating WoW64 syscall thunk table")

add_library(wow64 MODULE
    wow64.c
    wow64_thunks.c
    wow64.rc
    ${CMAKE_CURRENT_BINARY_DIR}/wow64.def)
target_include_directories(wow64 PRIVATE ${CMAKE_CURRENT_BINARY_DIR})
set_property(SOURCE wow64_thunks.c APPEND PROPERTY OBJECT_DEPENDS ${_wow64_thunks})

set_module_type(wow64 win32dll ENTRYPOINT DllMain 12)
# ntdll ONLY — a WoW64 process must not pull 64-bit kernel32 in through us
# (the 64-bit loader skips kernel32 for wow64 processes, matching Win10).
add_importlibs(wow64 ntdll)
add_dependencies(wow64 psdk)
add_cd_file(TARGET wow64 DESTINATION reactos/system32 FOR all)

endif()
