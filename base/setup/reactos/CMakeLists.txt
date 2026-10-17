
# The TreeList control is vendored (vendor/treelist); reactos.h includes its header.
include_directories(${REACTOS_SOURCE_DIR}/vendor/treelist)

list(APPEND SOURCE
    # spapisup/fileqsup.c  # replaced by Rust (sdk/lib/rust/reactos-rust/src/reactos/fileqsup.rs)
    # spapisup/infsupp.c  # replaced by Rust (sdk/lib/rust/reactos-rust/src/reactos/infsupp.rs)
    # drivepage.c  # replaced by Rust (sdk/lib/rust/reactos-rust/src/reactos/drivepage.rs)
    # reactos.c  # replaced by Rust (sdk/lib/rust/reactos-rust/src/reactos/reactos.rs)
    rust_cohab_stub.c
    rust_varargs_shim.c
    ${REACTOS_SOURCE_DIR}/vendor/treelist/treelist.c
    reactos.h)

file(GLOB reactos_rc_deps res/*.*)
add_rc_deps(reactos.rc ${reactos_rc_deps})
# Every translation unit is Rust now (reactos-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(reactos_rust reactos-rust "" TRUE)
add_executable(reactos ${SOURCE} reactos.rc)
add_dependencies(reactos rust_reactos_rust)
target_link_libraries(reactos ${reactos_rust_LIB})
add_pch(reactos reactos.h SOURCE)

set_module_type(reactos win32gui UNICODE)
target_link_libraries(reactos uuid)
target_link_libraries(reactos zlib_solo) ## We use USETUP's cabinet implementation
add_delay_importlibs(reactos setuplib)
add_importlibs(reactos advapi32 gdi32 user32 comctl32 shlwapi setupapi msvcrt kernel32 ntdll)
add_cd_file(TARGET reactos DESTINATION reactos NO_CAB FOR bootcd)
