/*
 * Cohabitation placeholder TU for framebuf.dll.
 *
 * The framebuffer display driver's C sources (enable.c, palette.c,
 * pointer.c, screen.c, surface.c) have been translated to Rust and live in
 * sdk/lib/rust/framebuf-rust/ (libframebuf_rust.a, linked via CMakeLists).
 * The C files are commented out of the add_library() SOURCES but kept on
 * disk per the never-delete-a-translation rule.
 *
 * A MODULE/library target still needs at least one compiled source, so this
 * one-line TU stands in. All real code is the #[no_mangle] Rust symbols.
 */
typedef int framebuf_rust_cohab_placeholder;
