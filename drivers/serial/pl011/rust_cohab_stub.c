/*
 * pl011.sys Rust cohabitation stub.
 *
 * pl011.c (DriverEntry + the Pl011* dispatch routines) is translated to Rust
 * (sdk/lib/rust/pl011-rust, kept on disk per the never-delete rule). The MODULE
 * link needs at least one C TU; this placeholder supplies it while the real
 * code comes from libpl011_rust.a.
 */
typedef int pl011_rust_cohab_placeholder;
