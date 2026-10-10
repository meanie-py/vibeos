/* acgenral is Rust (sdk/lib/rust/acgenral-rust); a CMake target needs one TU. */
/* Pull the crate's object, and with it every .shm$BBB shim record, out of the archive. */
extern const unsigned char acgenral_rust_anchor;
const void *const acgenral_rust_keep = &acgenral_rust_anchor;
