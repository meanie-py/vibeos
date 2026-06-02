/*
 * Cohabitation stub for the Rust csrlib (client CSR API).
 *
 * api.c/capture.c/connect.c are commented out of add_library() (see
 * CMakeLists.txt): their 16 client Csr* exports (CsrClientCallServer,
 * CsrClientConnectToServer, CsrCaptureMessageBuffer, …) are now
 * provided by the LEAN `csrlib_only` build of libsmss_csrss_rust.a
 * (sdk/lib/rust/smss-csrss-rust/src/csrlib/), linked into ntdll.dll as
 * libcsrlib_rust.a. A static library still needs at least one linkable
 * TU; with the C objects removed the linker pulls the Rust archive
 * members (an .a is only extracted to satisfy an undefined symbol — a
 * direct .obj defining it would BLOCK extraction;
 * --allow-multiple-definition does NOT force it).
 *
 * Keep the C on disk (translation philosophy: never delete the C);
 * it is simply not compiled while the Rust translation is active.
 */
