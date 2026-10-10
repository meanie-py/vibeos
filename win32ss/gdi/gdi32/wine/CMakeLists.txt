
include_directories(
    ${REACTOS_SOURCE_DIR}/sdk/include/wine
    ${REACTOS_SOURCE_DIR}/win32ss/include
    ../include)

list(APPEND SOURCE
    # emfdc.c  # replaced by Rust (sdk/lib/rust/winegdi-rust/src/winegdi/emfdc.rs)
    # emfdrv.c  # replaced by Rust (sdk/lib/rust/winegdi-rust/src/winegdi/emfdrv.rs)
    # enhmetafile.c  # replaced by Rust (sdk/lib/rust/winegdi-rust/src/winegdi/enhmetafile.rs)
    # metadc.c  # replaced by Rust (sdk/lib/rust/winegdi-rust/src/winegdi/metadc.rs)
    # metafile.c  # replaced by Rust (sdk/lib/rust/winegdi-rust/src/winegdi/metafile.rs)
    # rosglue.c  # replaced by Rust (sdk/lib/rust/winegdi-rust/src/winegdi/rosglue.rs)
    rust_cohab_stub.c
    )

# Every translation unit is Rust now (winegdi-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(winegdi_rust winegdi-rust "" TRUE)
add_library(winegdi STATIC ${SOURCE})
add_dependencies(winegdi rust_winegdi_rust)
target_link_libraries(winegdi PRIVATE ${winegdi_rust_LIB})
target_link_libraries(winegdi PRIVATE pseh)
add_dependencies(winegdi psdk)
