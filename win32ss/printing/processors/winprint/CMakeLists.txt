
spec2def(winprint.dll winprint.spec ADD_IMPORTLIB)

list(APPEND SOURCE
    # main.c  # replaced by Rust (sdk/lib/rust/winprint-rust/src/winprint/main.rs)
    # raw.c  # replaced by Rust (sdk/lib/rust/winprint-rust/src/winprint/raw.rs)
    rust_cohab_stub.c
    precomp.h)

# Every translation unit is Rust now (winprint-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(winprint_rust winprint-rust "" TRUE)
add_library(winprint MODULE
    ${SOURCE}
    winprint.rc
    ${CMAKE_CURRENT_BINARY_DIR}/winprint.def)
add_dependencies(winprint rust_winprint_rust)
target_link_libraries(winprint ${winprint_rust_LIB})

set_module_type(winprint win32dll UNICODE)
target_link_libraries(winprint wine)
add_importlibs(winprint spoolss msvcrt kernel32 ntdll)
add_pch(winprint precomp.h SOURCE)
if(ARCH STREQUAL "amd64")
    add_cd_file(TARGET winprint DESTINATION reactos/system32/spool/prtprocs/x64 FOR all)
elseif(ARCH STREQUAL "arm64")
    add_cd_file(TARGET winprint DESTINATION reactos/system32/spool/prtprocs/arm64 FOR all)
else()
    add_cd_file(TARGET winprint DESTINATION reactos/system32/spool/prtprocs/w32x86 FOR all)
endif()
