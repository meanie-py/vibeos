/*
 * Cohabitation stub for the Rust smlib (SM client).
 *
 * smclient.c is commented out of add_library() (see CMakeLists.txt):
 * its 7 SM-client exports (SmConnectToSm, SmSendMsgToSm, SmExecPgm,
 * SmStartCsr, SmStopCsr, SmLoadDeferedSubsystem, SmSessionComplete)
 * are now provided by the Rust translation in
 * sdk/lib/rust/smss-csrss-rust/src/smlib/smclient.rs. The three PEs
 * that link smlib each pull the Rust symbols from a Rust archive:
 *   - csrsrv.dll  → libsmss_csrss_rust.a (default features)
 *   - smss.exe    → libsmss_csrss_rust.a (smss_exe_only)
 *   - ntdll_vista → libsmlib_rust.a      (smlib_only, new this flip)
 *
 * A static library still needs at least one linkable TU; with
 * smclient.c removed the linker pulls the Rust archive members (an .a
 * is only extracted to satisfy an undefined symbol — a direct .obj
 * defining it would BLOCK extraction; --allow-multiple-definition does
 * NOT force it).
 *
 * Keep the C on disk (translation philosophy: never delete the C);
 * it is simply not compiled while the Rust translation is active.
 */
