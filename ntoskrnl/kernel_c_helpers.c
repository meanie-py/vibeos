/*
 * kernel_c_helpers.c — residual ps_i.h re-include trick.
 *
 * Most of what used to live here has been split into focused trampolines
 * (`docs/engineering/shim-elimination.md`):
 *   - `variadic_shims.c` — KdpDprintf, RtlStringCbPrintfA.
 *   - `lto_workaround.c` — SepInitTokensShim (LLVM i386 fat-LTO ud2 bug).
 *   - `mm_lock_outlines.c` — MmLockSectionSegment trio (`__FILE__`/`__LINE__`).
 *   - `arm64_uart.c` — ARM64_UART_TRACE / ARM64_UART_TRACE_HEX.
 *   - PS-subsystem globals → ntoskrnl-rust::ps::ps_globals.
 *   - ExpKdbgExtHandle / ExpKdbgExtFileCache / ExpKdbgExtDefWrites →
 *     ntoskrnl-rust::ex::handle (uses Rust kdbp_print! macro).
 *
 * What's left in this file:
 *   - The ps_i.h re-include trick. The header defines two static const
 *     arrays of NTQUERY_INFO_RECORD entries (PsProcessInfoClass /
 *     PsThreadInfoClass) used by NtQuery/SetInformationProcess/Thread.
 *     The first inclusion (via ntoskrnl.h elsewhere in the C build)
 *     creates per-translation-unit `_static_*` copies; this re-include
 *     with `#define static` neutralised provides the externally-linkable
 *     names that the Rust query-info dispatchers extern-link against.
 *
 *     Translatable to Rust eventually (~250 entries each, mechanical
 *     but careful). For now, isolated here in 6 lines of C with full
 *     `<ntoskrnl.h>` for the prototypes.
 */

#define PsProcessInfoClass _static_PsProcessInfoClass
#define PsThreadInfoClass  _static_PsThreadInfoClass

#include <ntoskrnl.h>
#define NDEBUG
#include <debug.h>

#undef PsProcessInfoClass
#undef PsThreadInfoClass

#pragma push_macro("static")
#define static
#include "internal/ps_i.h"
#pragma pop_macro("static")
