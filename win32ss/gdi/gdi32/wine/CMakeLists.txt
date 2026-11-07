
include_directories(
    ${REACTOS_SOURCE_DIR}/sdk/include/wine
    ${REACTOS_SOURCE_DIR}/win32ss/include
    ../include)

# emfdc.c  # replaced by Rust (sdk/lib/rust/winegdi-rust/src/winegdi/emfdc.rs)
# emfdrv.c  # replaced by Rust (sdk/lib/rust/winegdi-rust/src/winegdi/emfdrv.rs)
# enhmetafile.c  # replaced by Rust (sdk/lib/rust/winegdi-rust/src/winegdi/enhmetafile.rs)
# metadc.c  # replaced by Rust (sdk/lib/rust/winegdi-rust/src/winegdi/metadc.rs)
# metafile.c  # replaced by Rust (sdk/lib/rust/winegdi-rust/src/winegdi/metafile.rs)
# rosglue.c  # replaced by Rust (sdk/lib/rust/winegdi-rust/src/winegdi/rosglue.rs)

# Every translation unit is Rust now (winegdi-rust, `rustconv port`); the C stays on
# disk. The target is an INTERFACE library over the Rust archive (`cargo-cell --interface`).
rust_staticlib_variant(winegdi_rust winegdi-rust "" TRUE)
add_library(winegdi INTERFACE)
add_dependencies(winegdi rust_winegdi_rust)
target_link_libraries(winegdi INTERFACE $<LINK_ONLY:${winegdi_rust_LIB}>)
target_link_libraries(winegdi INTERFACE $<LINK_ONLY:pseh>)
add_dependencies(winegdi psdk)

# Layout probe for the marshalgen win32k audit (tools/vibeos-codegen marshal audit): one TU
# compiled with the flags winegdi's stub TU had (this directory's, plus pseh's usage
# requirements), so the audit can still dump precomp.h's records now that no gdi32 C is
# compiled. user32_layout_probe is the same thing for user32.h. Never built.
file(WRITE ${CMAKE_CURRENT_BINARY_DIR}/gdi32_layout_probe.c "#include <precomp.h>\n")
add_library(gdi32_layout_probe OBJECT EXCLUDE_FROM_ALL
    ${CMAKE_CURRENT_BINARY_DIR}/gdi32_layout_probe.c)
target_link_libraries(gdi32_layout_probe PRIVATE pseh)
add_dependencies(gdi32_layout_probe psdk)
