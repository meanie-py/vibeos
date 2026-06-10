/* Cohabitation placeholder TU for the classpnp kerneldll MODULE.
 *
 * All 18 classpnp C sources (autorun/class/classwmi/clntirp/create/data/debug/
 * dictlib/dispatch/guid/history/lock/obsolete/power/retry/srblib/utils/xferpkt) are
 * translated to Rust (sdk/lib/rust/classpnp-rust, linked as libclasspnp_rust.a) and
 * kept on disk per the never-delete rule. The 60 classpnp.spec exports (Class*) plus
 * the PE DriverEntry come from the linked Rust staticlib; cmake keeps the MODULE link
 * (set_module_type kernelmodedriver + spec2def .def + add_importlibs(ntoskrnl hal)).
 * Mirrors the wmilib-rust / i8042prt-rust cohabitation.
 */
typedef int classpnp_rust_cohab_placeholder;
