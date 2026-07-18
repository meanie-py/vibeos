/*
 * iphlpapi_rust_shims.c — link-time no-op providers for the wine debug macros that the
 * DeepSeek-translated iphlpapi_main.rs (sdk/lib/rust/iphlpapi-rust) references as BARE SYMBOLS.
 *
 * The c2rust/DeepSeek output emits two debug forms: (a) the inline-expanded
 * `if (*__wine_dbch___default).flags & (1<<DBCL) { ros_dbg_log(...) }` form — ros_dbg_log is a
 * real wine export and links fine, but it dereferences the default channel pointer
 * `__wine_dbch___default`; and (b) bare `TRACE(fmt, ...)` / `FIXME(fmt)` calls that in C are
 * wine MACROS (no symbol) but in the Rust translation are plain calls needing a linkable symbol.
 *
 * This shim provides those symbols as release-inert no-ops: the channel's flags=0 so the inline
 * `if` never fires, and the bare variadic calls do nothing. Kept freestanding (NO includes) so
 * wine/debug.h's TRACE/FIXME/ERR/WARN macros never collide with these definitions.
 *
 * NOTE: the still-C iphlpapi files use WINE_DEFAULT_DEBUG_CHANNEL(iphlpapi) → __wine_dbch_iphlpapi,
 * a DIFFERENT symbol, so there is no clash with the channel below.
 */

struct __wine_debug_channel_shim { unsigned char flags; char name[15]; };
static struct __wine_debug_channel_shim __wine_dbch___default_storage = { 0, "iphlpapi" };
void *__wine_dbch___default = &__wine_dbch___default_storage;

void TRACE(const char *fmt, ...) { (void)fmt; }
void FIXME(const char *fmt, ...) { (void)fmt; }
void ERR(const char *fmt, ...)   { (void)fmt; }
void WARN(const char *fmt, ...)  { (void)fmt; }

/*
 * StringCbPrintfW — strsafe.h ships it only as a `static inline` (no linkable symbol) and it is
 * VARIADIC (WINAPIV = __cdecl), which stable Rust cannot define. Provide a real out-of-line
 * __cdecl symbol via _vsnwprintf (msvcrt). cbDest is a BYTE count (Cb), so convert to a WCHAR
 * count for _vsnwprintf. Kept freestanding — only <stdarg.h>/<stddef.h> (neither pulls
 * wine/debug.h, so the TRACE/FIXME macros never clash with the no-ops above).
 */
#include <stdarg.h>
#include <stddef.h>

int __cdecl _vsnwprintf(wchar_t *buffer, size_t count, const wchar_t *format, va_list argptr);

long __cdecl StringCbPrintfW(wchar_t *pszDest, size_t cbDest, const wchar_t *pszFormat, ...)
{
    va_list args;
    int r;
    size_t cchDest = cbDest / sizeof(wchar_t);

    if (cchDest == 0)
        return 0x80070057L; /* STRSAFE_E_INVALID_PARAMETER */

    va_start(args, pszFormat);
    r = _vsnwprintf(pszDest, cchDest, pszFormat, args);
    va_end(args);

    if (r < 0 || (size_t)r >= cchDest)
    {
        pszDest[cchDest - 1] = 0;
        return 0x8007007AL; /* STRSAFE_E_INSUFFICIENT_BUFFER */
    }
    return 0L; /* S_OK */
}
