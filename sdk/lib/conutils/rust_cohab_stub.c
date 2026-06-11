/*
 * conutils Rust cohabitation stub. The console-util TUs (outstream/pager/screen/
 * stream/utils) are translated to Rust (sdk/lib/rust/conutils-rust, kept on disk
 * per the never-delete rule). The static lib needs a C TU; this placeholder
 * supplies it while the real code comes from libconutils_rust.a (linked, so
 * consumers like cmd.exe inherit it).
 */
typedef int conutils_rust_cohab_placeholder;
