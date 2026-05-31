/*
 * Cohabitation stub for the Rust usersrv (winsrv.dll).
 *
 * harderror.c / init.c / register.c / shutdown.c are commented out of
 * add_library(usersrv) (see ../usersrv.cmake): their symbols — the
 * winsrv.spec exports UserServerDllInitialization + _UserSoundSentry
 * plus the internal USER server-API table entries — are now provided
 * by libwinsrv_rust.a (sdk/lib/rust/winsrv-rust/src/usersrv/). A static
 * library still needs at least one linkable TU; with the C objects
 * removed the linker pulls the Rust archive members (an .a is only
 * extracted to satisfy an undefined symbol — a direct .obj defining it
 * would block extraction; --allow-multiple-definition does NOT force
 * it). The winsrv.def export roots force extraction at the winsrv.dll
 * link.
 *
 * Keep the C on disk (translation philosophy: never delete the C);
 * it is simply not compiled while the Rust translation is active.
 */
