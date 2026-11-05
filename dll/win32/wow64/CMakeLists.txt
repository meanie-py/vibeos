# vibeos wow64.dll — Win10+-model WOW64 host layer (arm64 via FEX, amd64 via
# native wow64cpu — Phase E). See docs/engineering/wow64-plan.md. The exports
# are arch-gated in wow64.spec, so building this on x86 produces an
# empty-export DLL; only ship it where it means something.
if(ARCH STREQUAL "arm64" OR ARCH STREQUAL "amd64")

spec2def(wow64.dll wow64.spec ADD_IMPORTLIB)

# Phase D: the generated 32->64 syscall thunk tables (wow64gen, deterministic
# spec2def-class codegen -- see tools/wow64gen/README.md). Indexed by REAL
# service number (sysfuncs.h / w32ksvc32.h order == what guest EAX carries; win32k
# entries are 0x1000 + index); ntdll.spec and win32u.spec contribute argument types
# only. The tables are Rust modules of wow64-rust, COMMITTED under
# sdk/lib/rust/wow64-rust/src/wow64/gen/ so a plain cargo check sees them; this step
# fails the build when they no longer match their inputs. Regenerate with
# `tools/codegen wow64-thunks --emit-rs`.
include(${REACTOS_SOURCE_DIR}/sdk/cmake/codegen.cmake)
vibeos_codegen_dependencies(_wow64_codegen_deps)

set(_wow64_gen_rs ${REACTOS_SOURCE_DIR}/sdk/lib/rust/wow64-rust/src/wow64/gen)
set(_wow64_thunks_check ${CMAKE_CURRENT_BINARY_DIR}/wow64_thunks_rs.checked)
add_custom_command(
    OUTPUT ${_wow64_thunks_check}
    COMMAND ${REACTOS_SOURCE_DIR}/tools/codegen wow64-thunks --emit-rs --check
        --root ${REACTOS_SOURCE_DIR}
    COMMAND ${CMAKE_COMMAND} -E touch ${_wow64_thunks_check}
    DEPENDS
        ${_wow64_codegen_deps}
        ${REACTOS_SOURCE_DIR}/dll/ntdll/def/ntdll.spec
        ${REACTOS_SOURCE_DIR}/ntoskrnl/include/sysfuncs.h
        ${REACTOS_SOURCE_DIR}/tools/wow64gen/manual.txt
        ${REACTOS_SOURCE_DIR}/win32ss/win32u/win32u.spec
        ${REACTOS_SOURCE_DIR}/win32ss/w32ksvc32.h
        ${REACTOS_SOURCE_DIR}/tools/wow64gen/manual-win32k.txt
        ${REACTOS_SOURCE_DIR}/tools/wow64gen/automarshal-win32k.txt
        ${_wow64_gen_rs}/wow64_thunks.rs
        ${_wow64_gen_rs}/wow64_win32k_thunks.rs
    VERBATIM
    COMMENT "wow64gen: checking the committed WoW64 syscall thunk tables")

# Every translation unit is Rust now (wow64-rust): six from `rustconv port`, the two table
# dispatchers by hand, and the tables plus both automarshal TUs from the generator, which
# still writes the C alongside. The C stays on disk. The target keeps a stub TU and links
# the Rust archive.
rust_staticlib_variant(wow64_rust wow64-rust "" TRUE ${_wow64_thunks_check})
add_library(wow64 MODULE
    # wow64.c  # replaced by Rust (sdk/lib/rust/wow64-rust/src/wow64/wow64.rs)
    # wow64_thunks.c  # replaced by Rust (sdk/lib/rust/wow64-rust/src/wow64/wow64_thunks.rs)
    # wow64_win32k.c  # replaced by Rust (sdk/lib/rust/wow64-rust/src/wow64/wow64_win32k.rs)
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

set_module_type(wow64 win32dll ENTRYPOINT DllMain 12)
# ntdll + win32u ONLY — a WoW64 process must not pull 64-bit kernel32 in
# through us (the 64-bit loader skips kernel32 for wow64 processes, matching
# Win10). win32u is pure NtGdi*/NtUser* syscall stubs for the win32k table.
add_importlibs(wow64 win32u ntdll)
add_dependencies(wow64 psdk)
add_cd_file(TARGET wow64 DESTINATION reactos/system32 FOR all)

endif()
