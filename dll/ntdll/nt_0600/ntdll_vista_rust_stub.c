/*
 * Cohabitation stub for the Rust ntdll_vista.dll entry point.
 *
 * DllMain.c is commented out of add_library(ntdll_vista MODULE ...)
 * (see CMakeLists.txt): the module ENTRYPOINT `DllMain` is now provided
 * by libntdll_rust.a (sdk/lib/rust/ntdll-rust/src/nt_0600/dllmain.rs).
 * A MODULE library still needs at least one linkable TU; with the C
 * DllMain object removed the linker pulls the Rust archive member to
 * satisfy the (undefined) entry-point reference (an .a is only
 * extracted to satisfy an undefined symbol — a direct .obj defining it
 * would block extraction; --allow-multiple-definition does NOT force
 * it). See the matching csrsrv_rust_stub.c.
 *
 * Keep DllMain.c on disk (translation philosophy: never delete the C);
 * it is simply not compiled while the Rust translation is active.
 */
