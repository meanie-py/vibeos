/*
 * Cohabitation stub for the Rust userinit.exe (FULL flip).
 *
 * The translated userinit files (userinit.c, livecd.c) are commented out of
 * add_executable() in CMakeLists.txt; their symbols (wWinMain, StartShell,
 * IsLiveCD, SetUserSettings, RunLiveCD, the dialog procs, ParseUnattend, …)
 * are now provided by libuserinit_rust.a built from sdk/lib/rust/userinit-rust/.
 *
 * CMake's add_executable() needs at least one TU it always compiles; with both
 * C files removed the linker pulls the Rust archive members for the
 * now-undefined symbols (--allow-multiple-definition alone does NOT force
 * archive extraction when a direct .obj already defines a symbol). Keep both
 * .c files on disk (translation philosophy: never delete the C).
 */
