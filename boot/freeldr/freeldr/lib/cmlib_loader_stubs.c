/*
 * Loader-only stubs for kernel-mode symbols referenced by cmlib-rust's
 * write/validate/self-heal code paths (HvpWriteHive, CmpValidateRegistryInternal,
 * CmIsSelfHealEnabled). cmlib-rust is the FULL kernel Configuration Manager hive
 * library (the bootloader previously used the reduced `_BLDR_` C variant which
 * #if'd these paths out). Those paths are never executed during the bootloader's
 * read-only SYSTEM hive parse, but the cohab staticlib references the symbols, so
 * the loader link needs them resolved. The kernel build resolves them from
 * ntoskrnl proper; this shim is compiled ONLY into uefildr/rosload.
 *
 * NTAPI = __stdcall on i386 (matches cmlib-rust's `extern "system"` →
 * `_Name@N`); __stdcall is ignored on amd64/arm64 where the ABI is cdecl, so the
 * undecorated names match there too.
 */

typedef unsigned char  BOOLEAN;
typedef unsigned long  ULONG;
typedef long long      LONGLONG;

#if defined(__i386__)
#define NTAPI __stdcall
#else
#define NTAPI
#endif

/* Consumer globals (ntoskrnl provides the real ones in the kernel). */
void   *CmiVolatileHive = 0;
BOOLEAN CmpSelfHeal     = 0;
ULONG   CmpBootType     = 0;

/* KeQuerySystemTime — zero the timestamp (write path is never taken at boot). */
void NTAPI KeQuerySystemTime(LONGLONG *SystemTime)
{
    if (SystemTime)
        *SystemTime = 0;
}

/* IoSetThreadHardErrorMode — no-op; report hard-errors "enabled" (TRUE). */
BOOLEAN NTAPI IoSetThreadHardErrorMode(BOOLEAN Enable)
{
    (void)Enable;
    return 1;
}
