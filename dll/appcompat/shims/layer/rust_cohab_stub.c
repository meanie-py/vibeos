/* aclayers is Rust (sdk/lib/rust/aclayers-rust); a CMake target needs one TU. */
/* Pull the crate's object, and with it every .shm$BBB shim record, out of the archive. */
extern const unsigned char aclayers_rust_anchor;
const void *const aclayers_rust_keep = &aclayers_rust_anchor;
