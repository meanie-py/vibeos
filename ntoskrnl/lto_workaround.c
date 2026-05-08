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
 * Isolation note: this file pulls the full <ntoskrnl.h> chain (Se*, Ob*,
 * Ps* APIs all needed). The whole point is that this compile-unit is NOT
 * subject to Rust's LTO firewall — keeping it in its own .c file makes the
 * isolation explicit and survives any future cargo-driven cc-rs split.
 */

#include <ntoskrnl.h>
#define NDEBUG
#include <debug.h>

BOOLEAN
SepInitTokensShim(VOID)
{
    PTOKEN tok;

    /* Initialize process token fast reference to NULL */
    ExInitializeFastReference(&PsGetCurrentProcess()->Token, NULL);

    /* Create the system process token */
    tok = SepCreateSystemProcessToken();
    ObInitializeFastReference(&PsGetCurrentProcess()->Token, tok);

    /* Create anonymous logon tokens */
    SeAnonymousLogonToken = SepCreateSystemAnonymousLogonToken();
    if (!SeAnonymousLogonToken) return FALSE;

    SeAnonymousLogonTokenNoEveryone = SepCreateSystemAnonymousLogonTokenNoEveryone();
    if (!SeAnonymousLogonTokenNoEveryone) return FALSE;

    return TRUE;
}
