/*
 * mm_rust_shims.c — surviving C-side helpers for the Rust MM crate.
 *
 * Most of what used to live here has been promoted to bare-name
 * `#[no_mangle] extern "C" fn` exports in
 * `sdk/lib/rust/ntoskrnl-rust/src/shim_exports.rs` or to sibling
 * `*-inlines` / `*-macros` rlibs (see `docs/engineering/shim-elimination.md`).
 *
 * What's left in this file:
 *   - ARM64 debug helpers: `Arm64DumpPte`, `MmDbgCheckIrql`. Called
 *     from procsup.rs / ps/process.rs and the ke/arm64/stubs.rs APC +
 *     fault paths. `DbgPrint` doesn't reach serial on ARM64, hence the
 *     direct UART pokes. The body uses the ARM64 hardware-PTE bitfield
 *     layout (matches `_HARDWARE_PTE` in sdk/include/ndk/arm64/mmtypes.h)
 *     — translatable to Rust but verbose; left here pending a focused
 *     ARM64 PTE-dump migration pass.
 *
 * Header-free since the cargo-driven build PoC: no `<ntoskrnl.h>`, no
 * `<mm/ARM3/miarm.h>`, no `<internal/arch/mm.h>`. Local typedefs +
 * inlined `_MiAddressToPte` formula from `internal/arm64/mm.h` cover
 * everything we need; the formula and the bitfield layout are stable
 * ABI for the ARM64 self-map.
 */

#ifdef _M_ARM64

#include <stddef.h>
#include <stdint.h>

/* --- Local NT type aliases (replaces <ntoskrnl.h>) ------------------ */
typedef void              VOID;
typedef void             *PVOID;
typedef unsigned long     ULONG;
typedef unsigned long long ULONG64;
typedef uint64_t          ULONG_PTR;
typedef unsigned char     UCHAR;
typedef UCHAR             KIRQL;
typedef const char       *PCSTR;
#define NTAPI
#define IN
#define FORCEINLINE static inline __attribute__((always_inline))

/* --- ARM64 self-map PTE base + MiAddressToPte ----------------------- *
 * Pulled forward verbatim from sdk/include/xdk/arm64/mm.h and          *
 * ntoskrnl/include/internal/arm64/mm.h. Kept here so this file can     *
 * compile without dragging in the heavy MM headers.                    */
#define ARM64_PTE_BASE 0xFFFFF68000000000ULL

/* --- ARM64 hardware PTE bitfield layout ----------------------------- *
 * Matches _HARDWARE_PTE in sdk/include/ndk/arm64/mmtypes.h.            *
 * Layout: low → high.                                                  */
typedef struct _ARM64_HARDWARE_PTE
{
    ULONG64 Valid:1;
    ULONG64 NotLargePage:1;
    ULONG64 CacheType:2;
    ULONG64 OsAvailable2:1;
    ULONG64 NonSecure:1;
    ULONG64 Owner:1;
    ULONG64 NotDirty:1;
    ULONG64 Shareability:2;
    ULONG64 Accessed:1;
    ULONG64 NonGlobal:1;
    ULONG64 PageFrameNumber:36;
    ULONG64 RsvdZ1:4;
    ULONG64 ContigousBit:1;
    ULONG64 PrivilegedNoExecute:1;
    ULONG64 UserNoExecute:1;
    ULONG64 Writable:1;
    ULONG64 CopyOnWrite:1;
    ULONG64 OsAvailable:2;
    ULONG64 PxnTable:1;
    ULONG64 UxnTable:1;
    ULONG64 ApTable:2;
    ULONG64 NsTable:1;
} ARM64_HARDWARE_PTE;

typedef union _ARM64_PTE_VIEW
{
    ULONG64 Long;
    ARM64_HARDWARE_PTE Hard;
} ARM64_PTE_VIEW;

FORCEINLINE
__attribute__((unused))
ARM64_PTE_VIEW *
Arm64AddressToPte(PVOID Address)
{
    ULONG64 Offset = (ULONG64)Address >> (12 - 3);
    Offset &= 0xFFFFFFFFFULL << 3;
    return (ARM64_PTE_VIEW *)(ARM64_PTE_BASE + Offset);
}

/* --- Externs from the rest of the kernel ---------------------------- */
extern volatile PVOID Arm64UartKernelBase;
extern KIRQL NTAPI KeGetCurrentIrql(VOID);

/* --- UART helpers (unchanged from the previous header-bound version) */

static void MmUartPuts(const char *s) {
    volatile ULONG *u = (volatile ULONG *)Arm64UartKernelBase;
    if (u) while (*s) *u = (ULONG)*s++;
}

static void MmUartHex(const char *prefix, ULONG_PTR val) {
    char buf[20];
    int i;
    MmUartPuts(prefix);
    for (i = 15; i >= 0; i--)
        buf[15-i] = "0123456789ABCDEF"[(val >> (i*4)) & 0xF];
    buf[16] = '\r'; buf[17] = '\n'; buf[18] = 0;
    MmUartPuts(buf);
}

/* Uncomment to enable per-PTE dump (matches the historical
 * ARM64_UART_TRACE_ENABLE gate from internal/arm64/ke.h). Default-off
 * because callers in stubs.rs invoke this on hot APC/fault paths. */
/* #define ARM64_PTE_DUMP_ENABLE */

VOID
NTAPI
Arm64DumpPte(IN PCSTR Prefix, IN PVOID Address)
{
#ifdef ARM64_PTE_DUMP_ENABLE
    ARM64_PTE_VIEW *Pte = Arm64AddressToPte(Address);

    MmUartHex(Prefix, (ULONG_PTR)Address);
    MmUartHex("[PTE] Entry=", Pte->Long);
    MmUartHex("[PTE] Valid=", Pte->Hard.Valid);
    MmUartHex("[PTE] Owner=", Pte->Hard.Owner);
    MmUartHex("[PTE] Writable=", Pte->Hard.Writable);
    MmUartHex("[PTE] UserNoExec=", Pte->Hard.UserNoExecute);
    MmUartHex("[PTE] CopyOnWrite=", Pte->Hard.CopyOnWrite);
    MmUartHex("[PTE] NotDirty=", Pte->Hard.NotDirty);
    MmUartHex("[PTE] Accessed=", Pte->Hard.Accessed);
    MmUartHex("[PTE] NonGlobal=", Pte->Hard.NonGlobal);
    MmUartHex("[PTE] Pfn=", Pte->Hard.PageFrameNumber);
#else
    (void)Prefix;
    (void)Address;
#endif
}

VOID NTAPI MmDbgCheckIrql(PCSTR label)
{
    KIRQL irql = KeGetCurrentIrql();
    MmUartPuts("[IRQL-CHK] ");
    MmUartPuts(label);
    MmUartHex(" irql=", irql);
}

#endif /* _M_ARM64 */
