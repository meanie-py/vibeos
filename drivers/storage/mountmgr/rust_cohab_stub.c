/* Cohabitation placeholder TU for the mountmgr.sys kernelmodedriver MODULE.
 *
 * All 7 mountmgr C sources (database/device/mountmgr/notify/point/symlink/
 * uniqueid) are translated to Rust (sdk/lib/rust/mountmgr-rust, linked as
 * libmountmgr_rust.a) and kept on disk per the never-delete rule. The PE
 * DriverEntry comes from the linked Rust staticlib; cmake keeps the MODULE link
 * (set_module_type kernelmodedriver + add_importlibs(ntoskrnl hal)). mountmgr.sys
 * exports NOTHING but DriverEntry (no .spec). Mirrors disk's rust_cohab_stub.c.
 *
 * Cohab helper C functions (DDK macros/inlines the Rust translation calls as
 * functions, leading-underscore names rustc can't reliably decorate on i386) get
 * added here as the cohab link surfaces undefined symbols.
 */
typedef int mountmgr_rust_cohab_placeholder;
