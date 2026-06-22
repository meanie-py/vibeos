/* Wrapper header for bindgen — the lsasrv marshalled-type FFI substrate
 * (lsasrv-sys) for the lsasrv.dll C-to-Rust translation.
 *
 * Pulls the LSAPR RPC marshalled types, the DSROLER/DSROLE dssetup types, and
 * the LSA policy/trusted-domain/secret info classes the Lsar server methods take
 * — by including lsasrv.h (the module's own win32+NDK context, which already
 * sequences windows.h, ntsam.h, ntlsa.h, lsass.h and the generated lsa_s.h
 * correctly) plus the generated dssetup_s.h. The allowlist (LSAPR, DSROLE,
 * POLICY_, LSA_ prefixes) keeps the surface to the marshalled + info types
 * absent from ntkrnl-sys (this is a usermode DLL, not kernel). Per-arch (pointer
 * widths and RPC string layout differ).
 *
 * NOTE: avoid the literal close-comment digraph in this banner — prefix globs
 * are written without a trailing star so they cannot terminate the comment. */
#include <stdarg.h>
#define WIN32_NO_STATUS
#define _USE_DUMMY_PSEH 1
#include "lsasrv.h"
#include "dssetup_s.h"
/* SRM port message types (srm.c) — needs PORT_MESSAGE from the NDK above. */
#include "srmp.h"
