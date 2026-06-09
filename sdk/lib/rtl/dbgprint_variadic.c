/*
 * PROJECT:     ReactOS Runtime Library
 * LICENSE:     GPL-2.0-or-later
 * PURPOSE:     C-variadic trampolines for the rtl-diag Rust crate.
 *
 * debug.c and wine_debug.c are translated to Rust in
 * sdk/lib/rust/rtl-diag/. The rtl-rust staticlib builds on the *stable*
 * Rust toolchain, which cannot DEFINE C-variadic ("...") functions
 * (#![feature(c_variadic)] is nightly-only). So the thin `va_start`
 * trampolines for DbgPrint / DbgPrintEx / DbgPrintReturnControlC and
 * wine_dbg_sprintf / wine_dbg_printf / ros_dbg_log remain in C here and
 * forward into the Rust va_list workers.
 *
 * This file also hosts rtldiag_format_va(), which preserves the *weak*
 * ntprintf_va_format_a dispatch (non-NULL only in ntoskrnl; NULL in ntdll
 * where it falls back to _vsnprintf). Weak linkage is expressible in C but
 * not in stable Rust, and it resolves at each consumer's final link.
 */

#include <rtl.h>
#include <ndk/kdfuncs.h>
#define NDEBUG
#include <debug.h>
#include <wine/debug.h>

/* --- Rust va_list workers (sdk/lib/rust/rtl-diag/src/{debug,wine_debug}.rs) --- */
ULONG NTAPI
vDbgPrintExWithPrefixInternal(IN PCCH Prefix,
                              IN ULONG ComponentId,
                              IN ULONG Level,
                              IN PCCH Format,
                              IN va_list ap,
                              IN BOOLEAN HandleBreakpoint);

const char *wine_dbg_vsprintf(const char *format, va_list args);
int rtldiag_wine_dbg_printf_v(const char *format, va_list args);
int rtldiag_ros_dbg_log_v(enum __wine_debug_class cls,
                          struct __wine_debug_channel *channel,
                          const char *file, const char *func, int line,
                          const char *format, va_list args);

/* --- weak ntprintf formatter: non-NULL only in ntoskrnl --- */
__attribute__((weak)) extern size_t
ntprintf_va_format_a(char *dst, size_t cap, const char *fmt, va_list ap);

size_t
rtldiag_format_va(char *dst, size_t cap, const char *fmt, va_list ap)
{
    if (ntprintf_va_format_a)
        return ntprintf_va_format_a(dst, cap, fmt, ap);
    return (size_t)_vsnprintf(dst, cap, fmt, ap);
}

/*
 * __wine_dbg_get_channel_flags: in the full libwine this consults a parsed
 * debug-options table. The kernel/ntdll librtl has no such table (the C
 * ros_dbg_log was always NDEBUG-dead-stripped, so this was never linked).
 * The Rust ros_dbg_log path keeps it live, so provide the minimal faithful
 * behaviour: report the channel's own flags (0 by default → no trace output).
 */
unsigned char
__wine_dbg_get_channel_flags(struct __wine_debug_channel *channel)
{
    return channel ? channel->flags : 0;
}

/* ---------------- debug.c variadic entry points ---------------- */

ULONG
__cdecl
DbgPrint(PCCH Format, ...)
{
    va_list ap;
    ULONG Status;
    va_start(ap, Format);
    Status = vDbgPrintExWithPrefixInternal("", -1, DPFLTR_ERROR_LEVEL, Format, ap, TRUE);
    va_end(ap);
    return Status;
}

ULONG
__cdecl
DbgPrintEx(IN ULONG ComponentId, IN ULONG Level, IN PCCH Format, ...)
{
    va_list ap;
    ULONG Status;
    va_start(ap, Format);
    Status = vDbgPrintExWithPrefixInternal("", ComponentId, Level, Format, ap, TRUE);
    va_end(ap);
    return Status;
}

ULONG
__cdecl
DbgPrintReturnControlC(PCCH Format, ...)
{
    va_list ap;
    ULONG Status;
    va_start(ap, Format);
    Status = vDbgPrintExWithPrefixInternal("", -1, DPFLTR_ERROR_LEVEL, Format, ap, FALSE);
    va_end(ap);
    return Status;
}

/* ---------------- wine_debug.c variadic entry points ---------------- */

const char *
__cdecl
wine_dbg_sprintf(const char *format, ...)
{
    const char *ret;
    va_list valist;
    va_start(valist, format);
    ret = wine_dbg_vsprintf(format, valist);
    va_end(valist);
    return ret;
}

int
__cdecl
wine_dbg_printf(const char *format, ...)
{
    int ret;
    va_list valist;
    va_start(valist, format);
    ret = rtldiag_wine_dbg_printf_v(format, valist);
    va_end(valist);
    return ret;
}

int
__cdecl
ros_dbg_log(enum __wine_debug_class cls,
            struct __wine_debug_channel *channel,
            const char *file, const char *func, const int line,
            const char *format, ...)
{
    int ret;
    va_list valist;
    va_start(valist, format);
    ret = rtldiag_ros_dbg_log_v(cls, channel, file, func, line, format, valist);
    va_end(valist);
    return ret;
}
