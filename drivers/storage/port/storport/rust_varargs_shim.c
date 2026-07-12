/* rust_varargs_shim.c — the ONLY two things stable Rust cannot express.
 *
 *   1. DEFINING a C-variadic function (`va_start` is legal only inside one), and
 *   2. va_arg.
 *
 * Both are the `c_variadic` feature (rust-lang/rust#44930 — FCP-complete with a merge
 * disposition, so this file is expected to be DELETED once it stabilizes; see
 * docs/engineering/active-lanes/stable-toolchain-plan.md). *Calling* and *declaring*
 * a variadic C fn are already stable, so storport-rust's call sites are untouched.
 *
 * Every entry below is a 3-line trampoline straight back into Rust: the StorPort
 * notification dispatch and the STORMINI debug-print semantics stay in
 * storport-rust/src/storport.rs. This is a shim for a language gap, NOT a Rust
 * translation reverted to C (AGENTS.md "Rust Is The Goal").
 *
 * !! These three are EXPORTED driver-ABI symbols (storport.spec) that real miniports
 * (storahci) call. The fixed-arg prefix and the cdecl variadic ABI below are the
 * contract — they were cross-checked against sdk/include/ddk/storport.h, storport.spec
 * and the Rust definitions. Do not "tidy" the types.
 *
 * TWO trampoline flavors, because the entry points need different things:
 *
 *   PULL (StorPortExtendedFunction, StorPortNotification) — Rust walks the argument
 *     list itself. We hand it a POINTER to the va_list (`&ap`), never the va_list
 *     itself: that avoids assuming `va_list` is a single pointer (true on our three
 *     Windows targets, false on SysV x86-64) and is what lets varargs-rust be
 *     unit-tested on the host. Rust side: varargs-rust (`VaArgs`) + the `*_v` twins.
 *
 *   FORWARD (StorPortDebugPrint) — the body does not va_arg at all; it hands the
 *     va_list straight to another va_list sink (vDbgPrintExWithPrefix). A `VaArgs`
 *     cannot express that, so the va_list is passed BY VALUE, exactly as the
 *     already-shipping sdk/lib/rtl/dbgprint_variadic.c does for the rtl-diag Rust
 *     workers (`ap: va_list` == `*mut c_char` on all three arches).
 *
 * Template: sdk/lib/rust/varargs-rust/rust_varargs_shim.c.template
 */

#include <stdarg.h>
#include <stddef.h>

/* `long` is 32-bit on all Windows arches (LLP64), so ULONG == unsigned long. */
typedef unsigned long ULONG_C;

/* ---- PART 1: the va_arg pullers (mirrored by varargs-rust's extern block) ------
 * Each takes a POINTER to the caller's va_list and ADVANCES it, so successive pulls
 * from Rust walk the argument list exactly as C's va_arg would.
 */
int                vibeos_va_arg_i32(void *app) { return va_arg(*(va_list *)app, int); }
unsigned int       vibeos_va_arg_u32(void *app) { return va_arg(*(va_list *)app, unsigned int); }
long long          vibeos_va_arg_i64(void *app) { return va_arg(*(va_list *)app, long long); }
unsigned long long vibeos_va_arg_u64(void *app) { return va_arg(*(va_list *)app, unsigned long long); }
double             vibeos_va_arg_f64(void *app) { return va_arg(*(va_list *)app, double); }
void *             vibeos_va_arg_ptr(void *app) { return va_arg(*(va_list *)app, void *); }

/* ---- PART 2: storport's variadic entry points ---------------------------------- */

/* StorPortDebugPrint — FORWARD flavor (body only re-emits the va_list).
 * storport.h:2442  VOID __cdecl StorPortDebugPrint(ULONG DebugPrintLevel,
 *                                                  PCCHAR DebugMessage, ...);
 * storport.spec:9  @ varargs StorPortDebugPrint(long str)
 */
extern void StorPortDebugPrint_v(ULONG_C DebugPrintLevel, char *DebugMessage, va_list ap);

void __cdecl
StorPortDebugPrint(ULONG_C DebugPrintLevel, char *DebugMessage, ...)
{
    va_list ap;
    va_start(ap, DebugMessage);
    StorPortDebugPrint_v(DebugPrintLevel, DebugMessage, ap);
    va_end(ap);
}

/* StorPortExtendedFunction — PULL flavor.
 * storport.h:2976  ULONG StorPortExtendedFunction(STORPORT_FUNCTION_CODE FunctionCode,
 *                                                 PVOID HwDeviceExtension, ...);
 * storport.spec:12 @ cdecl StorPortExtendedFunction()
 * STORPORT_FUNCTION_CODE is a C enum -> `unsigned int` (bindgen: c_uint).
 */
extern ULONG_C StorPortExtendedFunction_v(
    unsigned int FunctionCode, void *HwDeviceExtension, void *ap);

ULONG_C __cdecl
StorPortExtendedFunction(unsigned int FunctionCode, void *HwDeviceExtension, ...)
{
    va_list ap;
    ULONG_C result;
    va_start(ap, HwDeviceExtension);
    result = StorPortExtendedFunction_v(FunctionCode, HwDeviceExtension, (void *)&ap);
    va_end(ap);
    return result;
}

/* StorPortNotification — PULL flavor.
 * storport.h:2584  VOID __cdecl StorPortNotification(SCSI_NOTIFICATION_TYPE NotificationType,
 *                                                    PVOID HwDeviceExtension, ...);
 * storport.spec:26 @ cdecl StorPortNotification()   (:3 also forwards ScsiPortNotification here)
 * Caller cross-check: storahci-rust declares
 *   fn StorPortNotification(NotificationType: u32, HwDeviceExtension: PVOID, ...) -> ULONG;
 * — same two fixed args (SCSI_NOTIFICATION_TYPE is a C enum -> `unsigned int`). Its
 * ULONG return is a pre-existing mismatch vs the VOID definition; harmless on cdecl
 * (the caller just reads a dead register) and unchanged by this shim.
 */
extern void StorPortNotification_v(
    unsigned int NotificationType, void *HwDeviceExtension, void *ap);

void __cdecl
StorPortNotification(unsigned int NotificationType, void *HwDeviceExtension, ...)
{
    va_list ap;
    va_start(ap, HwDeviceExtension);
    StorPortNotification_v(NotificationType, HwDeviceExtension, (void *)&ap);
    va_end(ap);
}
