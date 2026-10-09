
spec2def(timedate.cpl timedate.spec)

list(APPEND SOURCE
    # clock.c  # replaced by Rust (sdk/lib/rust/timedate-rust/src/timedate/clock.rs)
    # dateandtime.c  # replaced by Rust (sdk/lib/rust/timedate-rust/src/timedate/dateandtime.rs)
    # internettime.c  # replaced by Rust (sdk/lib/rust/timedate-rust/src/timedate/internettime.rs)
    # monthcal.c  # replaced by Rust (sdk/lib/rust/timedate-rust/src/timedate/monthcal.rs)
    # timedate.c  # replaced by Rust (sdk/lib/rust/timedate-rust/src/timedate/timedate.rs)
    # timezone.c  # replaced by Rust (sdk/lib/rust/timedate-rust/src/timedate/timezone.rs)
    rust_cohab_stub.c
    timedate.h)

file(GLOB timedate_rc_deps resources/*.*)
add_rc_deps(timedate.rc ${timedate_rc_deps})

# Every translation unit is Rust now (timedate-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(timedate_rust timedate-rust "" TRUE)
add_library(timedate MODULE
    ${SOURCE}
    timedate.rc
    ${CMAKE_CURRENT_BINARY_DIR}/timedate.def)
add_dependencies(timedate rust_timedate_rust)
target_link_libraries(timedate ${timedate_rust_LIB})
add_pch(timedate timedate.h SOURCE)

set_module_type(timedate cpl UNICODE)
target_link_libraries(timedate tzlib)
add_importlibs(timedate w32time advapi32 user32 gdi32 comctl32 ws2_32 iphlpapi msvcrt kernel32 ntdll)
add_cd_file(TARGET timedate DESTINATION reactos/system32 FOR all)
