/*
 * Cohabitation stub for the Rust services.exe (SCM) flip (services-rust).
 *
 * Flipped .c files are commented out of add_executable() in CMakeLists.txt;
 * their symbols (wave 1: ScmAcquire/Release/QueryServiceLockStatus{W,A} from
 * lock.c) are now provided by libservices_rust.a built from
 * sdk/lib/rust/services-rust/.
 *
 * CMake's add_executable() needs at least one TU it always compiles; this stub
 * fills that role so the linker pulls the Rust archive members for the
 * now-undefined symbols (--allow-multiple-definition alone does NOT force
 * archive extraction when a direct .obj already defines a symbol). Keep the
 * flipped .c files on disk (translation philosophy: never delete the C).
 */
