/*
 * Cohabitation stub for the Rust csrsrv.dll.
 *
 * api.c/init.c/procsup.c/server.c/session.c/thredsup.c/wait.c are
 * commented out of add_library() (see CMakeLists.txt): their 35
 * csrsrv.spec exports (CsrCallServerFromServer, CsrCreateProcess,
 * CsrCreateThread, …) are now provided by libsmss_csrss_rust.a
 * (sdk/lib/rust/smss-csrss-rust/src/csrsrv/). An MODULE library still
 * needs at least one linkable TU; with the C objects removed the
 * linker pulls the Rust archive members (an .a is only extracted to
 * satisfy an undefined symbol — a direct .obj defining it would block
 * extraction; --allow-multiple-definition does NOT force it).
 *
 * Keep the C on disk (translation philosophy: never delete the C);
 * it is simply not compiled while the Rust translation is active.
 */
