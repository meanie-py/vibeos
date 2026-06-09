/*
 * ntlsalib Rust cohabitation stub. The kernel (_NTSYSTEM_) variant of lsa.c is
 * translated to Rust (sdk/lib/rust/lsalib-rust, kept on disk per the never-delete
 * rule). The static lib needs a C TU; this placeholder supplies it while the real
 * code comes from liblsalib_rust.a (PUBLIC-linked, so ntoskrnl inherits it). The
 * user-mode `lsalib` C variant (secur32) still compiles lsa.c directly.
 */
typedef int ntlsalib_rust_cohab_placeholder;
