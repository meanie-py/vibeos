/*
 * Cohabitation stub for the Rust smlib (SM client library).
 *
 * smclient.c is commented out of add_library() (see CMakeLists.txt):
 * its 7 live SM-client exports (SmConnectToSm, SmSendMsgToSm,
 * SmSessionComplete, SmExecPgm, SmLoadDeferedSubsystem, SmStartCsr,
 * SmStopCsr) are now provided by libsmss_csrss_rust.a
 * (sdk/lib/rust/smss-csrss-rust/src/smlib/smclient.rs). Both consumers
 * of smlib — smss.exe and csrsrv.dll — already link that archive, so
 * with the C object removed the linker pulls the Rust members instead
 * (an .a is only extracted to satisfy an undefined symbol; a direct
 * .obj defining it would block extraction — --allow-multiple-definition
 * does NOT force it). A static library still needs at least one
 * linkable TU, which this stub provides.
 *
 * smutils.c is dead (smutils.cmake is included nowhere) and was never
 * translated; it stays C-on-disk and uncompiled, same as smclient.c.
 *
 * Keep the C on disk (translation philosophy: never delete the C);
 * it is simply not compiled while the Rust translation is active.
 */
