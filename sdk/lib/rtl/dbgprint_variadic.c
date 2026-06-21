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
 *
 * SELF-CONTAINED / HEADERLESS (cmake-off spike, 2026-06-20): this file is
 * compiled directly by cc-rs via reactos-link's `.c_shims()` — NOT replayed
 * from cmake's compile_commands.json (`.c_from_compile_db`). It therefore
 * pulls NONE of the cmake header chain (was: <rtl.h>, <ndk/kdfuncs.h>,
 * <debug.h>, <wine/debug.h>); the handful of types/macros/consts those
 * provided are hand-defined below, layout-faithful (verified symbol-identical
 * vs the cmake-built object via nm). Same pattern as strsafe_shim.c /
 * variadic_shims.c. This is what lets the kernel/ntdll cargo link drop its
 * dependency on cmake having generated compile_commands.json for the C floor.
 */

/* ---- minimal hand-defined types/macros (was: the 4 cmake headers) ---- */
typedef unsigned long  ULONG;     /* `long` is 32-bit on all Windows arches (LLP64) */
typedef unsigned char  BOOLEAN;
typedef const char    *PCCH;
typedef __SIZE_TYPE__  size_t;
typedef __builtin_va_list va_list;
#define va_start(ap, p) __builtin_va_start(ap, p)
#define va_end(ap)      __builtin_va_end(ap)
#define IN
#define NTAPI __stdcall                 /* stdcall@N on i386; ignored on amd64/arm64 */
#define DPFLTR_ERROR_LEVEL 0            /* ndk/kdtypes.h + psdk/dpfilter.h */
#define TRUE  1                         /* ntdef.h BOOLEAN truth values */
#define FALSE 0

/* wine/debug.h — layout-faithful minimal (flags@0, name[15]; 16 bytes) */
enum __wine_debug_class
{
    __WINE_DBCL_FIXME = 0,
    __WINE_DBCL_ERR   = 1,
    __WINE_DBCL_WARN  = 2,
    __WINE_DBCL_TRACE = 3,
    __WINE_DBCL_INIT  = 0x7
};
struct __wine_debug_channel
{
    unsigned char flags;
    char name[15];
};

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

/* libcntpr provides _vsnprintf (sdk/lib/crt/printf/_vsnprintf.c). */
int __cdecl _vsnprintf(char *str, size_t size, const char *fmt, va_list ap);

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
