/* Cohabitation placeholder TU for the ksecdd kernelmodedriver MODULE.
 * The 5 .c bodies (ksecdd/dispatch/crypt/random/stubs) are translated to Rust
 * (sdk/lib/rust/ksecdd-rust, libksecdd_rust.a) and kept on disk per the
 * never-delete rule. ksecdd is an EXPORT driver: spec2def produces ksecdd.def +
 * the import lib (the stdcall SSPI API); the Rust #[no_mangle] symbols satisfy it.
 * cryptlib (already Rust) provides md4/md5/aes/des; ntoskrnl/hal via add_importlibs.
 * Bar: link-clean x3 (NOT boot-loaded in smoke).
 */
typedef int ksecdd_rust_cohab_placeholder;
