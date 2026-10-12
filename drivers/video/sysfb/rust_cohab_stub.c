/* Cohabitation placeholder TU for the sysfb kernelmodedriver MODULE.
 *
 * sysfb.c has been translated to Rust
 * (sdk/lib/rust/sysfb-rust/src/sysfb.rs, linked as
 * libsysfb_rust.a). A CMake MODULE target needs at least one source;
 * sysfb.c is commented out of the build (kept on disk per the
 * never-delete-a-translation rule). All real symbols — DriverEntry and
 * the internal driver — come from the linked Rust staticlib. Mirrors the
 * arm64input-rust cohabitation (drivers/input/arm64input/CMakeLists.txt).
 */
typedef int sysfb_rust_cohab_placeholder;
