/* Cohabitation placeholder TU for the ks.sys kernelmodedriver MODULE.
 * The Kernel Streaming class library (all 22 .c bodies) is translated to Rust
 * (sdk/lib/rust/ks-rust, libks_rust.a; DeepSeek-chat V3 + c2rust-anchored, constant-
 * safeguard-verified). The .c files are kept on disk per the never-delete rule and listed
 * (commented) in CMakeLists.txt. The no_mangle Ks / Ko / CBaseUnknown exports plus DriverEntry
 * in the rlib satisfy ks.spec and take over the link. ks is an EXPORT driver (216+ Ks exports
 * the audio/avstream stack imports). NOT boot-loaded in the standard smoke (the KS stack is
 * not exercised) -- link-clean x3 is the verification bar.
 */
typedef int ks_rust_cohab_placeholder;
