/* Minimal <wine/debug.h>: scanf.c #includes it before winesup.h.
 * winesup.h then `#define __WINE_DEBUG_H` and re-neutralizes TRACE/
 * WINE_DEFAULT_DEBUG_CHANNEL/debugstr_a, so this only needs to make
 * the include resolve and provide harmless no-op defaults. */
#ifndef __WINE_DEBUG_H
#define __WINE_DEBUG_H
#define WINE_DEFAULT_DEBUG_CHANNEL(ch)
#define WINE_DECLARE_DEBUG_CHANNEL(ch)
#define TRACE(...)
#define WARN(...)
#define FIXME(...)
#define ERR(...)
#define debugstr_a(s) (s)
#define debugstr_w(s) (s)
#endif
