/*
 * lto_workaround.c — isolated C compile-unit for the LLVM i386 fat-LTO bug
 * working around `SepInitTokensShim`.
 *
 * During fat LTO on i386, LLVM incorrectly inserts unreachable/ud2 after the
 * `@ObInitializeFastReference@8` call that sets up the system process token,
 * dropping all subsequent code (anonymous logon token creation, return TRUE).
 * The bug does not affect amd64 or arm64.
 *
 * Re-probed 2026-05-07 by inlining the body into se/semgr.rs. None of
 * `opaque_ptr`, `compiler_fence(SeqCst)`, `core::hint::black_box` defeat
 * the miscompilation. The C-side execution is the only working workaround
 * because it sits OUTSIDE the Rust LTO scope.
 *
 * Header-free since the cargo-driven build PoC: no `<ntoskrnl.h>`, no
 * `<debug.h>`. The EPROCESS layout is hidden behind a Rust helper
 * `_rs_PsCurrentProcessTokenAddr()` (see shim_exports.rs) so this file
 * doesn't need `_EPROCESS`/`EX_FAST_REF` definitions at all.
 *
 * Isolation note: this file MUST stay in its own .c file. The whole point
 * is that this compile-unit is NOT subject to Rust's LTO firewall, and
 * dropping the headers doesn't change that.
 */

#include <stddef.h>
#include <stdint.h>

/* --- Local NT type aliases (replaces <ntoskrnl.h>) ------------------ */
typedef void              VOID;
typedef void             *PVOID;
typedef unsigned char     UCHAR;
typedef UCHAR             BOOLEAN;

#define FALSE 0
#define TRUE  1

#if defined(_M_IX86)
# define NTAPI    __attribute__((stdcall))
# define FASTCALL __attribute__((fastcall))
#else
# define NTAPI
# define FASTCALL
#endif

/* --- Externs from the rest of the kernel ---------------------------- */

/* Rust helper that returns &CurrentProcess->Token. Hides the EPROCESS
 * layout from this compile unit so we don't need <ntoskrnl.h>. */
extern PVOID *_rs_PsCurrentProcessTokenAddr(void);

/* Executive fast-reference primitives. ExInitializeFastReference is
 * cdecl on every arch; ObInitializeFastReference is fastcall on i386
 * and cdecl elsewhere. Both are exported by ntoskrnl-rust. */
extern void ExInitializeFastReference(PVOID *fast_ref, PVOID object);
extern void FASTCALL ObInitializeFastReference(PVOID *fast_ref, PVOID object);

/* Sep* primitives — extern "system" / NTAPI on i386. */
extern PVOID NTAPI SepCreateSystemProcessToken(void);
extern PVOID NTAPI SepCreateSystemAnonymousLogonToken(void);
extern PVOID NTAPI SepCreateSystemAnonymousLogonTokenNoEveryone(void);

/* Globals owned by the Rust se subsystem. */
extern PVOID SeAnonymousLogonToken;
extern PVOID SeAnonymousLogonTokenNoEveryone;

BOOLEAN
SepInitTokensShim(VOID)
{
    PVOID tok;
    PVOID *token_addr = _rs_PsCurrentProcessTokenAddr();

    /* Initialize process token fast reference to NULL */
    ExInitializeFastReference(token_addr, NULL);

    /* Create the system process token */
    tok = SepCreateSystemProcessToken();
    ObInitializeFastReference(token_addr, tok);

    /* Create anonymous logon tokens */
    SeAnonymousLogonToken = SepCreateSystemAnonymousLogonToken();
    if (!SeAnonymousLogonToken) return FALSE;

    SeAnonymousLogonTokenNoEveryone = SepCreateSystemAnonymousLogonTokenNoEveryone();
    if (!SeAnonymousLogonTokenNoEveryone) return FALSE;

    return TRUE;
}
