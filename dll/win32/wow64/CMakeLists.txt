# vibeos wow64.dll — Win10+-model WOW64 host layer (arm64 via FEX, amd64 via
# native wow64cpu — Phase E). See docs/engineering/wow64-plan.md. The exports
# are arch-gated in wow64.spec, so building this on x86 produces an
# empty-export DLL; only ship it where it means something.
if(ARCH STREQUAL "arm64" OR ARCH STREQUAL "amd64")

spec2def(wow64.dll wow64.spec ADD_IMPORTLIB)

# Phase D: the generated 32->64 syscall thunk table (wow64gen, deterministic
# spec2def-class codegen — see tools/wow64gen/README.md). Indexed by REAL
# service number (sysfuncs.h order == what guest EAX carries); ntdll.spec
# contributes argument types only. wow64.c #includes the output.
include(${REACTOS_SOURCE_DIR}/sdk/cmake/codegen.cmake)
vibeos_codegen_dependencies(_wow64_codegen_deps)

set(_wow64_thunks ${CMAKE_CURRENT_BINARY_DIR}/wow64_thunks.gen.c)
add_custom_command(
    OUTPUT ${_wow64_thunks}
    COMMAND ${REACTOS_SOURCE_DIR}/tools/codegen wow64-thunks
        --spec ${REACTOS_SOURCE_DIR}/dll/ntdll/def/ntdll.spec
        --arch ${ARCH}
        --sysfuncs ${REACTOS_SOURCE_DIR}/ntoskrnl/include/sysfuncs.h
        --manual-list ${REACTOS_SOURCE_DIR}/tools/wow64gen/manual.txt
        --out-c ${_wow64_thunks}
    DEPENDS
        ${_wow64_codegen_deps}
        ${REACTOS_SOURCE_DIR}/dll/ntdll/def/ntdll.spec
        ${REACTOS_SOURCE_DIR}/ntoskrnl/include/sysfuncs.h
        ${REACTOS_SOURCE_DIR}/tools/wow64gen/manual.txt
    VERBATIM
    COMMENT "wow64gen: generating WoW64 syscall thunk table")

# wow64win surface: the win32k (NtGdi*/NtUser*) thunk table, same generator,
# ordered by w32ksvc32.h (guest EAX carries 0x1000+index). Host side resolves
# through the native win32u.dll importlib (pure syscall stubs, no csr).
set(_wow64_w32k_thunks ${CMAKE_CURRENT_BINARY_DIR}/wow64_win32k_thunks.gen.c)
add_custom_command(
    OUTPUT ${_wow64_w32k_thunks}
    COMMAND ${REACTOS_SOURCE_DIR}/tools/codegen wow64-thunks
        --spec ${REACTOS_SOURCE_DIR}/win32ss/win32u/win32u.spec
        --arch ${ARCH}
        --sysfuncs ${REACTOS_SOURCE_DIR}/win32ss/w32ksvc32.h
        --manual-list ${REACTOS_SOURCE_DIR}/tools/wow64gen/manual-win32k.txt,${REACTOS_SOURCE_DIR}/tools/wow64gen/automarshal-win32k.txt
        --symbol-prefix Wow64Win32kThunk
        --out-c ${_wow64_w32k_thunks}
    DEPENDS
        ${_wow64_codegen_deps}
        ${REACTOS_SOURCE_DIR}/win32ss/win32u/win32u.spec
        ${REACTOS_SOURCE_DIR}/win32ss/w32ksvc32.h
        ${REACTOS_SOURCE_DIR}/tools/wow64gen/manual-win32k.txt
        ${REACTOS_SOURCE_DIR}/tools/wow64gen/automarshal-win32k.txt
    VERBATIM
    COMMENT "wow64gen: generating WoW64 win32k thunk table")

# Every translation unit is Rust now (wow64-rust, `rustconv port`) except wow64_thunks.c, wow64_win32k.c,
# kept C by `port derive --keep-c`; the C stays on disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(wow64_rust wow64-rust "" TRUE)
add_library(wow64 MODULE
    # wow64.c  # replaced by Rust (sdk/lib/rust/wow64-rust/src/wow64/wow64.rs)
    wow64_thunks.c
    wow64_win32k.c
    # wow64_manual.c  # replaced by Rust (sdk/lib/rust/wow64-rust/src/wow64/wow64_manual.rs)
    # wow64_automarshal.c  # replaced by Rust (sdk/lib/rust/wow64-rust/src/wow64/wow64_automarshal.rs)
    # wow64_cb_automarshal.c  # replaced by Rust (sdk/lib/rust/wow64-rust/src/wow64/wow64_cb_automarshal.rs)
    # wow64_callback.c  # replaced by Rust (sdk/lib/rust/wow64-rust/src/wow64/wow64_callback.rs)
    # wow64_pfnmap.c  # replaced by Rust (sdk/lib/rust/wow64-rust/src/wow64/wow64_pfnmap.rs)
    rust_cohab_stub.c
    wow64.rc
    ${CMAKE_CURRENT_BINARY_DIR}/wow64.def)
add_dependencies(wow64 rust_wow64_rust)
target_link_libraries(wow64 ${wow64_rust_LIB})
target_include_directories(wow64 PRIVATE ${CMAKE_CURRENT_BINARY_DIR})
# Only the pfnmap source reads SERVERINFO, so only it gets the win32k headers.
set_property(SOURCE wow64_pfnmap.c APPEND PROPERTY INCLUDE_DIRECTORIES
    ${REACTOS_SOURCE_DIR}/win32ss/include
    ${REACTOS_SOURCE_DIR}/sdk/include/reactos/subsys)
set_property(SOURCE wow64_thunks.c APPEND PROPERTY OBJECT_DEPENDS ${_wow64_thunks})
set_property(SOURCE wow64_win32k.c APPEND PROPERTY OBJECT_DEPENDS ${_wow64_w32k_thunks})

set_module_type(wow64 win32dll ENTRYPOINT DllMain 12)
# ntdll + win32u ONLY — a WoW64 process must not pull 64-bit kernel32 in
# through us (the 64-bit loader skips kernel32 for wow64 processes, matching
# Win10). win32u is pure NtGdi*/NtUser* syscall stubs for the win32k table.
add_importlibs(wow64 win32u ntdll)
add_dependencies(wow64 psdk)
add_cd_file(TARGET wow64 DESTINATION reactos/system32 FOR all)

endif()
