/* Shared host environment for grading dll/win32/version/version.c.
 * Linked into BOTH the genuine C and the Rust candidate, so these definitions are
 * the world both sides see; they are not under grade. 16-bit WCHAR helpers (host
 * glibc's are 4-byte), an ASCII-only CP_ACP (bytes >= 0x80 and non-ASCII units map
 * to '?'), a malloc-backed process heap, and a last-error cell. */
#include <stdarg.h>
#include <stdlib.h>
#include <string.h>
#include "windef.h"
#include "winbase.h"
#include "winnls.h"
#include "version_env.h"

static DWORD env_last_error;
void WINAPI SetLastError(DWORD code) { env_last_error = code; }
DWORD WINAPI GetLastError(void) { return env_last_error; }

HANDLE WINAPI GetProcessHeap(void) { return (HANDLE)0x1000; }
LPVOID WINAPI HeapAlloc(HANDLE heap, DWORD flags, SIZE_T size)
{
    (void)heap;
    return (flags & HEAP_ZERO_MEMORY) ? calloc(1, size ? size : 1) : malloc(size ? size : 1);
}
BOOL WINAPI HeapFree(HANDLE heap, DWORD flags, LPVOID mem) { (void)heap; (void)flags; free(mem); return TRUE; }

size_t wcslen(const WCHAR *s) { const WCHAR *p = s; while (*p) p++; return p - s; }
static WCHAR up(WCHAR c) { return (c >= 'a' && c <= 'z') ? c - 32 : c; }
static WCHAR low(WCHAR c) { return (c >= 'A' && c <= 'Z') ? c + 32 : c; }
int _wcsnicmp(const WCHAR *a, const WCHAR *b, size_t n)
{
    if (!n) return 0;
    do {
        if (up(*a) != up(*b++)) return up(*a) - up(*--b);
        if (*a++ == 0) break;
    } while (--n);
    return 0;
}
int _wcsicmp(const WCHAR *a, const WCHAR *b)
{
    while (low(*a) == low(*b)) { if (!*a) return 0; a++; b++; }
    return low(*a) - low(*b);
}
int wcsicmp(const WCHAR *a, const WCHAR *b) { return _wcsicmp(a, b); }

int WINAPI MultiByteToWideChar(UINT cp, DWORD flags, LPCSTR src, int srclen, LPWSTR dst, int dstlen)
{
    (void)cp; (void)flags;
    int n = srclen < 0 ? (int)strlen(src) + 1 : srclen;
    if (!dstlen) return n;
    if (dstlen < n) { SetLastError(ERROR_INSUFFICIENT_BUFFER); return 0; }
    for (int i = 0; i < n; i++) dst[i] = (unsigned char)src[i] < 0x80 ? (unsigned char)src[i] : '?';
    return n;
}
int WINAPI WideCharToMultiByte(UINT cp, DWORD flags, LPCWSTR src, int srclen, LPSTR dst, int dstlen,
                               LPCSTR defchar, LPBOOL used)
{
    (void)cp; (void)flags; (void)defchar; (void)used;
    int n = srclen < 0 ? (int)wcslen(src) + 1 : srclen;
    if (!dstlen) return n;
    if (dstlen < n) { SetLastError(ERROR_INSUFFICIENT_BUFFER); return 0; }
    for (int i = 0; i < n; i++) dst[i] = src[i] < 0x80 ? (char)src[i] : '?';
    return n;
}

int _strnicmp(const char *a, const char *b, size_t n)
{
    unsigned char x, y;
    if (!n) return 0;
    do {
        x = (unsigned char)*a++; y = (unsigned char)*b++;
        if (x >= 'A' && x <= 'Z') x += 32;
        if (y >= 'A' && y <= 'Z') y += 32;
    } while (--n && x && x == y);
    return x - y;
}

/* Wine debug channels: every channel off, as in a release build. TRACE/WARN evaluate
   their arguments only when the channel is on, so nothing else of wine/debug.h runs. */
struct __wine_debug_channel;
unsigned char __wine_dbg_get_channel_flags(struct __wine_debug_channel *channel)
{
    (void)channel;
    return 0;
}

/* ReactOS's TRACE evaluates debugstr_w()/debugstr_a() even with the channel off; the
   string is only ever formatted by a disabled logger, so its content is irrelevant. */
const char *wine_dbgstr_wn(const WCHAR *s, int n) { (void)s; (void)n; return ""; }
const char *wine_dbgstr_an(const char *s, int n) { (void)s; (void)n; return ""; }

/* ReactOS routes TRACE/WARN/FIXME through ros_dbg_log (static channel flags default on,
   filtering is the logger's). Logging has no effect on anything graded: discard. */
int ros_dbg_log(int cls, struct __wine_debug_channel *ch, const char *file, const char *func,
                int line, const char *format, ...)
{
    (void)cls; (void)ch; (void)file; (void)func; (void)line; (void)format;
    return 0;
}

/* kernel32 lstrlenW: NULL is 0 (the SEH guard for a bad pointer is not modeled). */
int WINAPI lstrlenW(LPCWSTR s) { return s ? (int)wcslen(s) : 0; }
