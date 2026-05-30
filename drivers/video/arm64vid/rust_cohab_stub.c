/* Cohabitation placeholder TU for the arm64vid kernelmodedriver MODULE.
 *
 * arm64vid.c has been translated to Rust
 * (sdk/lib/rust/arm64vid-rust/src/arm64vid.rs, linked as
 * libarm64vid_rust.a). A CMake MODULE target needs at least one source;
 * arm64vid.c is commented out of the build (kept on disk per the
 * never-delete-a-translation rule). All real symbols — DriverEntry and
 * the internal driver — come from the linked Rust staticlib. Mirrors the
 * arm64input-rust cohabitation (drivers/input/arm64input/CMakeLists.txt).
 */
typedef int arm64vid_rust_cohab_placeholder;
