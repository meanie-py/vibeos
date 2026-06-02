/*
 * Cohabitation stub for the Rust smss.exe (PARTIAL flip — task #18).
 *
 * The translated smss files (crashdmp.c, smloop.c, smsbapi.c, smsessn.c,
 * smsubsys.c, smutil.c, smss.c) are commented out of add_executable() in
 * CMakeLists.txt; their symbols (`_main`, SmpApiLoop, SmpExecuteCommand,
 * SmpLoadSubSystem, SmpSbCreateSession, SmpTerminate, the SmpSubSystem*
 * globals, …) are now provided by libsmss_csrss_rust.a built with
 * `--no-default-features --features smss_exe_only,ntddi_ws03`
 * (sdk/lib/rust/smss-csrss-rust/src/smss/ + src/smlib/).
 *
 * pagefile.c + sminit.c have no Rust equivalent yet, so they stay C and keep
 * defining SmpCreatePagingFiles / SmpInit (which the Rust `_main` extern-refs).
 *
 * CMake's add_executable() needs at least one TU it always compiles; with the
 * seven C files removed the linker pulls the Rust archive members for the
 * now-undefined symbols (--allow-multiple-definition alone does NOT force
 * archive extraction when a direct .obj already defines a symbol). Keep the
 * seven .c files on disk (translation philosophy: never delete the C).
 */
