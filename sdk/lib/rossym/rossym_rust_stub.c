/* rossym is fully translated to Rust (sdk/lib/rust/rossym-rust). Every rossym .c
 * is flipped, so the kernel `rossym` static library has no C compiland of its
 * own — this stub provides one valid translation unit so the target builds, and
 * the RosSym* (cdecl) symbols are supplied by librossym_rust.a, linked below.
 * The C sources remain on disk for the host-tools (non-cross) build path. */
typedef int rossym_rust_cohab_stub_t;
