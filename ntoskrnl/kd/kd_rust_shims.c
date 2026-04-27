/*
 * kd_rust_shims.c — Outline shims for kd subsystem Rust translations.
 *
 * The Rust kd/kdio.rs code needs to set HAL's `KdComPortInUse` global
 * (defined in hal.dll, declared with `NTHALAPI` = `__declspec(dllimport)`
 * via sdk/include/ndk/haltypes.h). Cross-DLL imports require `__imp_`
 * thunks on i386, which Rust extern "C" decls don't generate.
 *
 * This shim exposes a `KdpRustSetComPortInUse(PUCHAR p)` cdecl entry that
 * the Rust side calls — the C code uses the proper dllimport mechanism
 * via the included header.
 */

#include <ntoskrnl.h>

/* extern PUCHAR KdComPortInUse pulls in the dllimport via NTHALAPI. */

VOID NTAPI
KdpRustSetComPortInUse(PUCHAR Port)
{
    KdComPortInUse = Port;
}

PUCHAR NTAPI
KdpRustGetComPortInUse(VOID)
{
    return KdComPortInUse;
}
