/* Cohabitation placeholder TU for the hidparse kernelmodedriver MODULE.
 *
 * hidparse.c (DriverEntry + the 5 hidparser_km callbacks AllocFunction /
 * FreeFunction / ZeroFunction / CopyFunction / DebugFunction) is translated to
 * Rust (sdk/lib/rust/hidparse-rust, linked as libhidparse_rust.a). hidparse.c
 * is removed from the add_library SOURCES and kept on disk per the
 * never-delete rule. DriverEntry + the callbacks come from the linked Rust
 * staticlib; the hidparser_km C library (sdk/lib/drivers/hidparser) stays C
 * (Batch 1 — it links against the Rust callbacks). cmake keeps the MODULE link
 * (set_module_type kernelmodedriver + spec2def .def + add_importlibs(ntoskrnl)).
 * Mirrors the i8042prt-rust cohabitation.
 */
typedef int hidparse_rust_cohab_placeholder;
