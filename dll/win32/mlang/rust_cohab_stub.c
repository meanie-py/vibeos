/* Cohabitation placeholder TU for the mlang.dll win32dll MODULE.
 *
 * mlang.c is translated to Rust (sdk/lib/rust/mlang-rust, linked as
 * libmlang_rust.a). mlang.c is removed from the add_library SOURCES and kept on
 * disk per the never-delete rule. The mlang.spec exports plus DllMain and the
 * COM entry points (DllGetClassObject / DllCanUnloadNow / DllRegisterServer /
 * DllUnregisterServer) come from the linked Rust staticlib; cmake keeps the
 * win32dll MODULE link (spec2def .def + mlang_stubs.c for the `stub`-only spec
 * entries + importlibs). Mirrors the wmilib-rust / bootvid-rust cohabitation.
 */
typedef int mlang_rust_cohab_placeholder;
