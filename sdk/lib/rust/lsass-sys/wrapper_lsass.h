/* Wrapper header for bindgen - focused LSA LPC-message substrate (lsass-sys).
 *
 * Uses the NDK umbrella (ndk/ntndk.h) for the base kernel types, same setup as
 * ntkrnl-sys/wrapper_ndk.h, then appends the LSASS API header. (lsa.c itself is
 * built in NDK context; the ntoskrnl.h umbrella security defs collide with
 * ntsecapi.h.) The allowlist (LSA_ and LSASS_ prefixes) keeps the surface to the
 * LPC message types the kernel ntlsalib (lsa.c) needs: LSA_API_MSG,
 * LSA_CONNECTION_INFO, the LSA_API_NUMBER request enum, the per-request structs,
 * absent from ntkrnl-sys. The embedded PORT_MESSAGE and ntsecapi types are pulled
 * in by bindgen as its own copies; lsa.rs casts at the Zw Port boundary where it
 * mixes with ntkrnl-sys. */
#define _NTOSKRNL_
#define _NTSYSTEM_
#define NO_RTL_INLINES
#define _USE_DUMMY_PSEH 1
#define __REACTOS__
#define _INTRIN_H_
#define _INTRIN_X86_H_
#ifndef __int64
#define __int64 long long
#endif
#undef FORCEINLINE
#define FORCEINLINE
#include <ndk/ntndk.h>
#include <lsass/lsass.h>
