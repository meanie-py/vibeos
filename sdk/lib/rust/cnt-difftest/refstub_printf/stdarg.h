/* Flat `va_list` for the printf differential ONLY (this dir is on the
 * include path of the ref_streamout/ref_wstreamout cc units, ahead of
 * the system header, so the genuine `printf/streamout.c`'s
 * `#include <stdarg.h>` resolves here).
 *
 * On the x86-64 host the real `va_list` is the SysV register-save-area
 * struct, which is incompatible with cnt-rust's engine (written for the
 * Windows model where a `va_list` is a plain `char*` walking the
 * spilled args). To diff apples-to-apples we drive BOTH the genuine
 * reference and the cnt engine with the SAME hand-built flat buffer:
 * one 8-byte slot per arg (the 64-bit host slot size cnt-rust uses),
 * little-endian. `va_arg(ap, T)` reads `T` at the cursor then advances
 * by the rounded slot — identical to cnt-rust's `va_next::<T>`.
 *
 * The genuine streamout receives the `va_list` as a by-value parameter
 * (and passes `&va_list` to format_float), so va_start/va_end are
 * inert here — the harness supplies the cursor directly. */
#ifndef CNT_PRINTF_FLAT_STDARG_H
#define CNT_PRINTF_FLAT_STDARG_H

typedef char *va_list;

#define CNT_VA_SLOT(sz) ((((sz) + 7) / 8) * 8)
#define va_start(ap, last) ((ap) = (char *)0)
#define va_end(ap) ((void)0)
#define va_copy(dst, src) ((dst) = (src))
#define va_arg(ap, T) \
    ((ap) += CNT_VA_SLOT(sizeof(T)), *(T *)((ap) - CNT_VA_SLOT(sizeof(T))))

#endif
