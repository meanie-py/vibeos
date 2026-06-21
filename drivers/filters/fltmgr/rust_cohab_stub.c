/* Cohabitation placeholder TU for the fltmgr kernelmodedriver MODULE.
 * The 10 .c bodies (Context/Dispatch/Filter/Interface/Lib/Messaging/Misc/Object/
 * Registry/Volume) are translated to Rust (sdk/lib/rust/fltmgr-rust,
 * libfltmgr_rust.a) and kept on disk per the never-delete rule. fltmgr is an
 * EXPORT driver: spec2def still produces fltmgr.def + the import lib (the Flt*
 * API minifilters link against); the Rust #[no_mangle] FltXxx symbols satisfy it.
 * ntoskrnl/hal link via add_importlibs. Bar: link-clean x3 (NOT boot-loaded in smoke).
 */
typedef int fltmgr_rust_cohab_placeholder;
