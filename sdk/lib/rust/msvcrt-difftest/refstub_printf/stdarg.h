/* Flat `va_list` for the printf PROBE ONLY (this dir sits on the include
 * path of the pfflat_* wcs.c instantiation, ahead of the system header,
 * so every `#include <stdarg.h>` in that TU resolves here).
 *
 * On the x86-64 host the real `va_list` is the SysV register-save-area
 * struct — and the msvcrt printf engine is not even CALLABLE under it:
 * `_vsnprintf` forwards `&valist` as `va_list*`, an idiom that is only
 * coherent when `va_list` is a plain pointer (`*valist` on a pointer-to-
 * decayed-array reinterprets the pointer variable itself as the SysV tag
 * struct → wild read; found the hard way when the probe first called the
 * engine). On the WINDOWS x64 target `va_list` IS a `char*` walking
 * 8-byte slots — exactly this model — so the flat header is both the fix
 * and the more target-faithful compilation. `va_arg(ap, T)` reads `T` at
 * the cursor then advances by the rounded slot — identical to cnt-rust's
 * `va_next::<T>`, so the streamout side and the msvcrt side of the probe
 * consume the SAME hand-packed buffer.
 *
 * The glibc headers this TU also pulls (<wchar.h> etc.) expect gcc's
 * <stdarg.h> contract: `__gnuc_va_list` always defined, and re-inclusion
 * under `__need___va_list` yielding only that name. Both are honored
 * below (everything is idempotent, so the simple guard suffices). */
#ifndef CNT_PRINTF_FLAT_STDARG_H
#define CNT_PRINTF_FLAT_STDARG_H

typedef char *__gnuc_va_list;
typedef char *va_list;
#define _VA_LIST_DEFINED
#define __va_list__

#define CNT_VA_SLOT(sz) ((((sz) + 7) / 8) * 8)
#define va_start(ap, last) ((ap) = (char *)0)
#define va_end(ap) ((void)0)
#define va_copy(dst, src) ((dst) = (src))
#define va_arg(ap, T) \
    ((ap) += CNT_VA_SLOT(sizeof(T)), *(T *)((ap) - CNT_VA_SLOT(sizeof(T))))

#endif
#undef __need___va_list
