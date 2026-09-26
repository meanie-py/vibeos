/* The ONE wide-printf engine both sides call, with msvcrt's _vsnwprintf contract: write at
 * most `count` WCHARs; if the output fits, NUL-terminate when room remains and return its
 * length; otherwise return -1 (no NUL). Supports %d, %u, %ls/%s (wide string), %c and %%,
 * which is every conversion the harness formats. */
#include <stdarg.h>
#include <stddef.h>

typedef unsigned short WCH;

static int put(WCH *buf, size_t count, size_t *n, WCH c)
{
    if (*n >= count) return 0;
    buf[(*n)++] = c;
    return 1;
}

int _vsnwprintf(WCH *buf, size_t count, const WCH *fmt, va_list ap)
{
    size_t n = 0;
    int over = 0;
    for (; *fmt; fmt++) {
        if (*fmt != '%') { over |= !put(buf, count, &n, *fmt); continue; }
        fmt++;
        if (*fmt == 'l') fmt++;
        switch (*fmt) {
        case '%': over |= !put(buf, count, &n, '%'); break;
        case 'c': over |= !put(buf, count, &n, (WCH)va_arg(ap, int)); break;
        case 's': {
            const WCH *s = va_arg(ap, const WCH *);
            if (!s) { static const WCH nul[] = {'(','n','u','l','l',')',0}; s = nul; }
            while (*s) over |= !put(buf, count, &n, *s++);
            break;
        }
        case 'd': case 'u': {
            char tmp[24]; int t = 0;
            long long v = (*fmt == 'd') ? (long long)va_arg(ap, int) : (long long)va_arg(ap, unsigned);
            unsigned long long u = v < 0 ? (unsigned long long)(-v) : (unsigned long long)v;
            if (v < 0) over |= !put(buf, count, &n, '-');
            do { tmp[t++] = (char)('0' + u % 10); u /= 10; } while (u);
            while (t) over |= !put(buf, count, &n, (WCH)tmp[--t]);
            break;
        }
        default: return -2; /* the harness never asks for anything else */
        }
    }
    if (over) return -1;
    if (n < count) buf[n] = 0;
    return (int)n;
}

int _snwprintf(WCH *buf, size_t count, const WCH *fmt, ...)
{
    va_list ap;
    int r;
    va_start(ap, fmt);
    r = _vsnwprintf(buf, count, fmt, ap);
    va_end(ap);
    return r;
}
