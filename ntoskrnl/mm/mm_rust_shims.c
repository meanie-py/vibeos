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
 *     direct UART pokes. The `Arm64DumpPte` body uses `MMPTE.u.Hard.X`
 *     bitfield accessors that bindgen renders as `__bindgen_anon_1`
 *     unions — translatable to Rust but verbose; left here pending a
 *     focused ARM64 PTE-dump migration pass.
 */

#include <ntoskrnl.h>
#define NDEBUG
#include <debug.h>
#include <mm/ARM3/miarm.h>
#include <internal/arch/mm.h>

#ifdef _M_ARM64
extern volatile PVOID Arm64UartKernelBase;

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

VOID
NTAPI
Arm64DumpPte(IN PCSTR Prefix, IN PVOID Address)
{
    PMMPTE Pte = MiAddressToPte(Address);

    ARM64_UART_TRACE_HEX(Prefix, (ULONG_PTR)Address);
    ARM64_UART_TRACE_HEX("[PTE] Entry=", Pte->u.Long);
    ARM64_UART_TRACE_HEX("[PTE] Valid=", Pte->u.Hard.Valid);
    ARM64_UART_TRACE_HEX("[PTE] Owner=", Pte->u.Hard.Owner);
    ARM64_UART_TRACE_HEX("[PTE] Writable=", Pte->u.Hard.Writable);
    ARM64_UART_TRACE_HEX("[PTE] UserNoExec=", Pte->u.Hard.UserNoExecute);
    ARM64_UART_TRACE_HEX("[PTE] CopyOnWrite=", Pte->u.Hard.CopyOnWrite);
    ARM64_UART_TRACE_HEX("[PTE] NotDirty=", Pte->u.Hard.NotDirty);
    ARM64_UART_TRACE_HEX("[PTE] Accessed=", Pte->u.Hard.Accessed);
    ARM64_UART_TRACE_HEX("[PTE] NonGlobal=", Pte->u.Hard.NonGlobal);
    ARM64_UART_TRACE_HEX("[PTE] Pfn=", Pte->u.Hard.PageFrameNumber);
}

VOID NTAPI MmDbgCheckIrql(PCSTR label)
{
    KIRQL irql = KeGetCurrentIrql();
    MmUartPuts("[IRQL-CHK] ");
    MmUartPuts(label);
    MmUartHex(" irql=", irql);
}
#endif
