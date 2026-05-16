/*
 * Cohabitation stub for the Rust csrss.exe.
 *
 * csrss.c is commented out of add_executable() (see CMakeLists.txt):
 * its `_main` / `CsrpSetDefaultProcessHardErrorMode` are now provided
 * by libsmss_csrss_rust.a (sdk/lib/rust/smss-csrss-rust/). CMake's
 * add_executable() needs at least one linkable TU, and with the C
 * `_main` removed the linker actually pulls the Rust archive member
 * (an undefined symbol — `--allow-multiple-definition` alone does NOT
 * force archive extraction when a direct .obj already defines it).
 *
 * Keep csrss.c on disk (translation philosophy: never delete the C);
 * it is simply not compiled while the Rust translation is active.
 */
