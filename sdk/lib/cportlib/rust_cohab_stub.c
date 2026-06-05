/*
 * Placeholder TU for the cportlib target. cport.c is replaced by Rust
 * (sdk/lib/rust/cport-rust -> libcport_rust.a, linked into this target),
 * kept on disk per the never-delete rule. A static library needs at least
 * one source, and the cargo-driven consumers (kdcom/uefildr/rosload) still
 * reference libcportlib.a by path, so keep the archive present. The Cp*
 * NTAPI exports all come from the Rust staticlib.
 */
void __cportlib_rust_cohab_stub(void) { }
