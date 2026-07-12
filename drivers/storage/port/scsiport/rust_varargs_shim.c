/* rust_varargs_shim.c — the ONLY two things stable Rust cannot express.
 *
 *   1. DEFINING a C-variadic function (`va_start` is legal only inside one), and
 *   2. va_arg.
 *
 * Both are the `c_variadic` feature (rust-lang/rust#44930 — FCP-complete with a merge
 * disposition, so this file is expected to be DELETED once it stabilizes; see
 * docs/engineering/active-lanes/stable-toolchain-plan.md). *Calling* and *declaring*
 * a variadic C fn are already stable, so scsiport-rust's call sites are untouched.
 *
 * Every entry below is a 3-line trampoline straight back into Rust: the SCSI
 * notification dispatch and the ScsiDebugPrint level/format semantics stay in
 * scsiport-rust/src/scsiport.rs. This is a shim for a language gap, NOT a Rust
 * translation reverted to C (AGENTS.md "Rust Is The Goal").
 *
 * !! Both are EXPORTED driver-ABI symbols (scsiport.spec) that real miniports
 * (atapi/uniata) call. The fixed-arg prefix and the cdecl variadic ABI below are the
 * contract — they were cross-checked against sdk/include/ddk/srb.h, scsiport.spec and
 * the Rust definitions. Do not "tidy" the types.
 *
 * TWO trampoline flavors, because the entry points need different things:
 *
 *   PULL (ScsiPortNotification) — Rust walks the argument list itself. We hand it a
 *     POINTER to the va_list (`&ap`), never the va_list itself: that avoids assuming
 *     `va_list` is a single pointer (true on our three Windows targets, false on SysV
 *     x86-64) and is what lets varargs-rust be unit-tested on the host. Rust side:
 *     varargs-rust (`VaArgs`) + the `*_v` twin.
 *
 *   FORWARD (ScsiDebugPrint) — the body does not va_arg at all; it hands the va_list
 *     straight to another va_list sink (vsprintf). A `VaArgs` cannot express that, so
 *     the va_list is passed BY VALUE, exactly as the already-shipping
 *     sdk/lib/rtl/dbgprint_variadic.c does for the rtl-diag Rust workers
 *     (`ap: va_list` == `*mut c_char` on all three arches).
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

/* ---- PART 2: scsiport's variadic entry points ---------------------------------- */

/* ScsiDebugPrint — FORWARD flavor (body only re-emits the va_list into vsprintf).
 * srb.h:978        VOID __cdecl ScsiDebugPrint(ULONG DebugPrintLevel,
 *                                              PCCHAR DebugMessage, ...);
 * scsiport.spec:1  @ varargs ScsiDebugPrint(long str)
 */
extern void ScsiDebugPrint_v(ULONG_C DebugPrintLevel, const char *DebugMessage, va_list ap);

void __cdecl
ScsiDebugPrint(ULONG_C DebugPrintLevel, const char *DebugMessage, ...)
{
    va_list ap;
    va_start(ap, DebugMessage);
    ScsiDebugPrint_v(DebugPrintLevel, DebugMessage, ap);
    va_end(ap);
}

/* ScsiPortNotification — PULL flavor.
 * srb.h:933        VOID __cdecl ScsiPortNotification(SCSI_NOTIFICATION_TYPE NotificationType,
 *                                                    PVOID HwDeviceExtension, ...);
 * scsiport.spec:18 @ cdecl ScsiPortNotification()
 * Caller cross-check: atapi-rust/src/inlines.rs declares
 *   pub fn ScsiPortNotification(NotificationType: ULONG, HwDeviceExtension: PVOID, ...);
 * — same two fixed args (SCSI_NOTIFICATION_TYPE is a C enum -> `unsigned int`, == ULONG
 * in width and register class here).
 */
extern void ScsiPortNotification_v(
    unsigned int NotificationType, void *HwDeviceExtension, void *ap);

void __cdecl
ScsiPortNotification(unsigned int NotificationType, void *HwDeviceExtension, ...)
{
    va_list ap;
    va_start(ap, HwDeviceExtension);
    ScsiPortNotification_v(NotificationType, HwDeviceExtension, (void *)&ap);
    va_end(ap);
}
