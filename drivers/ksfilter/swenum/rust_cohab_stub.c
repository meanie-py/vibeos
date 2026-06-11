/* Cohabitation placeholder TU for the swenum kernelmodedriver MODULE.
 *
 * swenum.c (the KS software bus enumerator: DriverEntry + the Sw* dispatch
 * routines) is translated to Rust (sdk/lib/rust/swenum-rust, linked as
 * libswenum_rust.a). swenum.c is removed from the add_library SOURCES and kept
 * on disk per the never-delete rule. DriverEntry + the whole driver come from
 * the linked Rust staticlib; the Ks* bus-enum entry points stay ks.sys imports
 * (add_importlibs(swenum ks ntoskrnl hal)). cmake keeps the MODULE link
 * (set_module_type kernelmodedriver). DeepSeek+c2rust translation (Batch 2).
 */
typedef int swenum_rust_cohab_placeholder;
