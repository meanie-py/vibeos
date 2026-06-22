/* Wrapper header for bindgen — the SAM RPC marshalled-type substrate
 * (samsrv-sys) for the SAM client surface lsasrv-rust (authpackage.c) and the
 * future samsrv-rust translation need.
 *
 * Pulls the widl-generated SAM client header sam_c.h (SAMPR_* marshalled types +
 * Samr* __stdcall client stub prototypes). sam_c.h is self-contained — it
 * sequences rpc.h, rpcndr.h, windows.h and ole2.h itself — so this wrapper is a
 * thin include, kept apart from lsasrv-sys's NDK/ntlsa context (windows.h/ole2.h
 * would collide with WIN32_NO_STATUS + NDK there). The allowlist (SAMPR_, Samr,
 * SamIFree_ prefixes) keeps the surface to the marshalled types + client stubs.
 * Per-arch (pointer widths + RPC string layout differ). */
#include <sam_c.h>
