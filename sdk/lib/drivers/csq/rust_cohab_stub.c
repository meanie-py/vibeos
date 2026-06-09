/*
 * csq Rust cohabitation stub. csq.c (the IoCsq* exports) is translated to Rust
 * (sdk/lib/rust/csq-rust, kept on disk per the never-delete rule). The static
 * lib needs a C TU; this placeholder supplies it while the real code comes from
 * libcsq_rust.a (PUBLIC-linked, so ntoskrnl inherits it).
 */
typedef int csq_rust_cohab_placeholder;
