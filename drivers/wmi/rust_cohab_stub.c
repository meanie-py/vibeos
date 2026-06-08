/* Cohabitation placeholder TU for the wmilib kerneldll MODULE.
 *
 * wmilib.c (WmiCompleteRequest / WmiFireEvent / WmiSystemControl) is
 * translated to Rust (sdk/lib/rust/wmilib-rust, linked as libwmilib_rust.a).
 * wmilib.c is removed from the add_library SOURCES and kept on disk per the
 * never-delete rule. The 3 wmilib.spec exports (Wmi*) come from the linked
 * Rust staticlib; cmake keeps the MODULE link (set_module_type ... ENTRYPOINT 0
 * + spec2def .def + add_importlibs(ntoskrnl)). Mirrors the bootvid-rust /
 * i8042prt-rust cohabitation.
 */
typedef int wmilib_rust_cohab_placeholder;
