/*
 * Cohabitation anchor TU for the Rust dnslib static library.
 *
 * TWO facts that make this whole library a perennial false-positive for
 * "remaining C to translate" (several agents have re-flagged it):
 *
 * 1. dnslib IS NOT BUILT. `add_subdirectory(dnslib)` is commented out in
 *    sdk/lib/CMakeLists.txt ("Nothing links to this lib.") — no module links
 *    dnslib (dnsapi.dll has its own resolver). This CMakeLists is therefore
 *    not even processed today; the cohab below is dormant, ready for if/when a
 *    consumer re-enables the subdirectory.
 *
 * 2. Of the 17 .c, the 8 FUNCTIONAL TUs (addr/dnsaddr/flatbuf/hostent/memory/
 *    sablob/straddr/string) are translated to Rust (sdk/lib/rust/dnslib-rust →
 *    libdnslib_rust.a, linked in CMakeLists.txt). The other 9 (debug/dnsutil/
 *    ip6/name/print/record/rrprint/table/utf8) are EMPTY ReactOS placeholder
 *    stubs — header comment + #include "precomp.h", ZERO functions, ZERO
 *    symbols (verify: `grep -c '(' *.c`). They were never real C debt, so
 *    there is nothing to translate.
 *
 * Net: dnslib is effectively 100% Rust (every real symbol is in
 * libdnslib_rust.a). All 17 .c are kept on disk (never-delete rule); the 8
 * functional and the 9 empty .c are all dropped from add_library().
 *
 * CMake's add_library() needs at least one TU it always compiles; this file is
 * that anchor. It intentionally defines no symbols — the linker pulls every
 * dnslib symbol from libdnslib_rust.a.
 */

#include "precomp.h"
