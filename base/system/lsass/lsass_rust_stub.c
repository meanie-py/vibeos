/*
 * Cohabitation stub for the Rust lsass.exe (FULL flip).
 *
 * The translated lsass.c is commented out of add_executable() in
 * CMakeLists.txt; its symbol (wWinMain) is now provided by liblsass_rust.a
 * built from sdk/lib/rust/lsass-rust/.
 *
 * CMake's add_executable() needs at least one TU it always compiles; with the
 * C file removed the linker pulls the Rust archive member for the now-undefined
 * wWinMain (--allow-multiple-definition alone does NOT force archive extraction
 * when a direct .obj already defines the symbol). Keep lsass.c on disk
 * (translation philosophy: never delete the C).
 */
