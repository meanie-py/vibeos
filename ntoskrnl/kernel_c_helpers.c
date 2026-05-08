/*
 * kernel_c_helpers.c — residual ps_i.h re-include trick.
 *
 * The ps_i.h re-include trick provides the externally-linkable
 * `PsProcessInfoClass` / `PsThreadInfoClass` arrays. The first inclusion
 * (via ntoskrnl.h elsewhere in the C build) creates per-translation-unit
 * `_static_*` copies; this re-include with `#define static` neutralised
 * provides the global symbols that the Rust query-info dispatchers
 * extern-link against.
 *
 * A parallel byte-equal Rust translation lives at
 * ntoskrnl-rust/src/ps/info_class.rs as `_rs_PsProcessInfoClass` /
 * `_rs_PsThreadInfoClass`. Renaming those to `PsProcessInfoClass` (so
 * Rust supplies the linker symbol) reproducibly bugchecks i386 with
 * STATUS_DATATYPE_MISALIGNMENT after `[WIN32K] shell ready` — under
 * investigation. Until that's diagnosed, this C trampoline supplies the
 * canonical symbol and the Rust file is the future-ready translation.
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
