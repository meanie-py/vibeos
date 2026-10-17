
add_definitions(
    -DNDIS40
    -D_NTDRIVER_
    -D_TDI_)

spec2def(tdi.sys misc/tdi.spec)

list(APPEND SOURCE
    # cte/events.c  # replaced by Rust (sdk/lib/rust/tdi-rust/src/tdi/events.rs)
    # cte/string.c  # replaced by Rust (sdk/lib/rust/tdi-rust/src/tdi/string.rs)
    # cte/timer.c  # replaced by Rust (sdk/lib/rust/tdi-rust/src/tdi/timer.rs)
    # misc/main.c  # replaced by Rust (sdk/lib/rust/tdi-rust/src/tdi/main.rs)
    # tdi/dereg.c  # replaced by Rust (sdk/lib/rust/tdi-rust/src/tdi/dereg.rs)
    # tdi/handler.c  # replaced by Rust (sdk/lib/rust/tdi-rust/src/tdi/handler.rs)
    # tdi/obsolete.c  # replaced by Rust (sdk/lib/rust/tdi-rust/src/tdi/obsolete.rs)
    # tdi/stubs.c  # replaced by Rust (sdk/lib/rust/tdi-rust/src/tdi/stubs.rs)
    rust_cohab_stub.c
    precomp.h)

# Every translation unit is Rust now (tdi-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(tdi_rust tdi-rust "" TRUE)
add_library(tdi MODULE
    ${SOURCE}
    misc/tdi.rc
    ${CMAKE_CURRENT_BINARY_DIR}/tdi.def)
add_dependencies(tdi rust_tdi_rust)
target_link_libraries(tdi ${tdi_rust_LIB})

add_pch(tdi precomp.h SOURCE)
set_module_type(tdi kernelmodedriver)
add_importlibs(tdi ntoskrnl hal)
add_cd_file(TARGET tdi DESTINATION reactos/system32/drivers FOR all)
