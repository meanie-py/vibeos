/*
 * COPYRIGHT:       GNU GPL, see COPYING in the top level directory
 * PROJECT:         ReactOS crt library
 * FILE:            lib/sdk/crt/printf/format_float.c
 * PURPOSE:         The float-formatting half of streamout, extracted so
 *                  the integer/string engine can be the Rust
 *                  `streamout`/`wstreamout` (cnt-rust) while this
 *                  irreducibly-floating-point path stays C.
 *
 *                  On i386 the kernel builds Rust no-SSE (CR4.OSFXSR
 *                  unset in Phase 0) so cnt-rust has no by-value `f64`
 *                  ABI; this C residual keeps the genuine x87 path
 *                  there. On amd64/arm64 the Rust `format_float`
 *                  (cnt-rust, behind the `float` feature) takes over
 *                  and this file is NOT compiled into libcntpr.
 *
 *                  Verbatim from streamout.c (Timo Kreuzer / Katayama
 *                  Hirofumi MZ) — same enum, same va_arg_ffp/get_exp/
 *                  round macros, same body. The Rust `streamout` calls
 *                  it with `(&string, &prefix, &cursor)`; on Windows a
 *                  `va_list` is a plain `char*`, so the engine's cursor
 *                  IS a live va_list and this advances it in lockstep.
 */

#include <stdio.h>
#include <stdarg.h>
#include <tchar.h>
#include <strings.h>
#include <math.h>
#include <float.h>

#ifdef _UNICODE
# define format_float format_floatw
#endif

enum
{
    /* Formatting flags */
    FLAG_ALIGN_LEFT =    0x01,
    FLAG_FORCE_SIGN =    0x02,
    FLAG_FORCE_SIGNSP =  0x04,
    FLAG_PAD_ZERO =      0x08,
    FLAG_SPECIAL =       0x10,

    /* Data format flags */
    FLAG_SHORT =         0x100,
    FLAG_LONG =          0x200,
    FLAG_WIDECHAR =      FLAG_LONG,
    FLAG_INT64 =         0x400,
#ifdef _WIN64
    FLAG_INTPTR =        FLAG_INT64,
#else
    FLAG_INTPTR =        0,
#endif
    FLAG_LONGDOUBLE =    0x800,
};

#define va_arg_ffp(argptr, flags) \
    (flags & FLAG_LONGDOUBLE) ? va_arg(argptr, long double) : \
    va_arg(argptr, double)

#define get_exp(f) (int)floor(f == 0 ? 0 : (f >= 0 ? log10(f) : log10(-f)))
#define round(x) floor((x) + 0.5)

void
#ifdef _LIBCNT_
/* Due to restrictions in kernel mode regarding the use of floating point,
   we prevent it from being inlined */
__declspec(noinline)
#endif
format_float(
    _TCHAR chr,
    unsigned int flags,
    int precision,
    _TCHAR **string,
    const _TCHAR **prefix,
    va_list *argptr)
{
    static const _TCHAR digits_l[] = _T("0123456789abcdef0x");
    static const _TCHAR digits_u[] = _T("0123456789ABCDEF0X");
    static const _TCHAR _nan[] = _T("#QNAN");
    static const _TCHAR _infinity[] = _T("#INF");
    const _TCHAR *digits = digits_l;
    int exponent = 0, sign;
    long double fpval, fpval2;
    int padding = 0, num_digits, val32, base = 10;

    /* Normalize the precision */
    if (precision < 0) precision = 6;
    else if (precision > 17)
    {
        padding = precision - 17;
        precision = 17;
    }

    /* Get the float value and calculate the exponent */
    fpval = va_arg_ffp(*argptr, flags);
    exponent = get_exp(fpval);
    sign = fpval < 0 ? -1 : 1;

    switch (chr)
    {
        case _T('G'):
            digits = digits_u;
        case _T('g'):
            if (precision > 0) precision--;
            if (exponent < -4 || exponent >= precision) goto case_e;

            /* Shift the decimal point and round */
            fpval2 = round(sign * fpval * pow(10., precision));

            /* Skip trailing zeroes */
            while (precision && (unsigned __int64)fpval2 % 10 == 0)
            {
                precision--;
                fpval2 /= 10;
            }
            break;

        case _T('E'):
            digits = digits_u;
        case _T('e'):
        case_e:
            /* Shift the decimal point and round */
            fpval2 = round(sign * fpval * pow(10., precision - exponent));

            /* Compensate for changed exponent through rounding */
            if (fpval2 >= (unsigned __int64)pow(10., precision + 1))
            {
                exponent++;
                fpval2 = round(sign * fpval * pow(10., precision - exponent));
            }

            val32 = exponent >= 0 ? exponent : -exponent;

            // FIXME: handle length of exponent field:
            // http://msdn.microsoft.com/de-de/library/0fatw238%28VS.80%29.aspx (DEAD_LINK)
            num_digits = 3;
            while (num_digits--)
            {
                *--(*string) = digits[val32 % 10];
                val32 /= 10;
            }

            /* Sign for the exponent */
            *--(*string) = exponent >= 0 ? _T('+') : _T('-');

            /* Add 'e' or 'E' separator */
            *--(*string) = digits[0xe];
            break;

        case _T('A'):
            digits = digits_u;
        case _T('a'):
//            base = 16;
            // FIXME: TODO

        case _T('f'):
        default:
            /* Shift the decimal point and round */
            fpval2 = round(sign * fpval * pow(10., precision));
            break;
    }

    /* Handle sign */
    if (fpval < 0)
    {
        *prefix = _T("-");
    }
    else if (flags & FLAG_FORCE_SIGN)
        *prefix = _T("+");
    else if (flags & FLAG_FORCE_SIGNSP)
        *prefix = _T(" ");

    /* Handle special cases first */
    if (_isnan(fpval))
    {
        (*string) -= sizeof(_nan) / sizeof(_TCHAR) - 1;
        _tcscpy((*string), _nan);
        fpval2 = 1;
    }
    else if (!_finite(fpval))
    {
        (*string) -= sizeof(_infinity) / sizeof(_TCHAR) - 1;
        _tcscpy((*string), _infinity);
        fpval2 = 1;
    }
    else
    {
        /* Zero padding */
        while (padding-- > 0) *--(*string) = _T('0');

        /* Digits after the decimal point */
        num_digits = precision;
        while (num_digits-- > 0)
        {
            *--(*string) = digits[(unsigned __int64)fpval2 % 10];
            fpval2 /= base;
        }
    }

    if (precision > 0 || flags & FLAG_SPECIAL)
        *--(*string) = _T('.');

    /* Digits before the decimal point */
    do
    {
        *--(*string) = digits[(unsigned __int64)fpval2 % base];
        fpval2 /= base;
    }
    while ((unsigned __int64)fpval2);

}
