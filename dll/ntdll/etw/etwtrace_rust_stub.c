/*
 * Cohabitation stub for the Rust ntdll.dll ETW API.
 *
 * etw/trace.c is commented out of add_library(etwtrace ...) (see
 * ../CMakeLists.txt): the Etw* exports are now provided by
 * libntdll_rust.a (sdk/lib/rust/ntdll-rust/src/etw/trace.rs), which is
 * already linked into ntdll.dll. An .a member is only extracted to
 * satisfy an undefined symbol, so removing the C trace.obj lets the
 * linker pull the Rust definitions in its place (mirrors uilist.rs /
 * version.rs in rtl_um and the csrsrv stub).
 *
 * A CMake library target needs at least one source; this keeps the
 * etwtrace target valid while trace.c is inactive. (At our entire build
 * matrix — DLL_EXPORT_VERSION=0x502 — etwtrace is linked ONLY into
 * ntdll; advapi32 links it solely at 0x600+. If the 0x600+ matrix is
 * ever built, advapi32 would need libntdll_rust.a's Etw symbols wired in
 * — deferred, not built/tested here.)
 *
 * Keep etw/trace.c on disk (translation philosophy: never delete the C);
 * it is simply not compiled while the Rust translation is active.
 */

/* A defined symbol keeps the TU non-empty for archivers/linkers that
 * dislike a completely empty object. Unreferenced and harmless. */
const int __etwtrace_rust_cohab_stub = 0;
