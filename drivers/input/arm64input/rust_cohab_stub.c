/* Cohabitation placeholder TU for the arm64input kernelmodedriver MODULE.
 *
 * arm64input.c has been translated to Rust
 * (sdk/lib/rust/arm64input-rust/src/arm64input.rs, linked as
 * libarm64input_rust.a). A CMake MODULE target needs at least one source;
 * arm64input.c is commented out of the build (kept on disk per the
 * never-delete-a-translation rule). All real symbols — DriverEntry and
 * the internal driver — come from the linked Rust staticlib. Mirrors the
 * arm64 hal-rust cohabitation (hal/halarm64/CMakeLists.txt).
 */
typedef int arm64input_rust_cohab_placeholder;
