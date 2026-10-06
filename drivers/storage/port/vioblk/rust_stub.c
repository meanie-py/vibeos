/* vioblk.sys is entirely Rust (sdk/lib/rust/vioblk-rust, libvioblk_rust.a): the
 * #[no_mangle] DriverEntry there is the module's entry point. This TU exists only
 * because add_library needs a C source; it has no C counterpart to preserve. */
typedef int vioblk_rust_placeholder;
