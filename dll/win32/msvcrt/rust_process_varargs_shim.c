/*
 * PROJECT:     ReactOS CRT (vibeos Rust port)
 * LICENSE:     LGPL-2.1-or-later (see process.c, from which these bodies are taken)
 * PURPOSE:     The irreducible C half of process.c: its sixteen `_exec*`/`_spawn*`
 *              WINAPIV variadic entry points, as va_start trampolines over the Rust
 *              bodies in msvcrt-crt-rust/src/msvcrt/process.rs.
 *
 * This is the process.c counterpart of rust_varargs_shim.c (wcs.c, printf),
 * rust_scanf_shim.c (scanf.c) and rust_file_varargs_shim.c (file.c), and it exists for
 * the same language limit, not for any translation gap: stable Rust cannot DEFINE a
 * C-variadic function (`c_variadic` is nightly, and this project is on pinned stable).
 * RECEIVING an already-started `va_list` is fine — it is a plain pointer on every
 * Windows target we build — which is why all sixteen twins below are Rust. Only
 * `va_start` is out of reach.
 *
 * Twin naming: real MSVCRT spells "the va_list version of X" by inserting a `v` right
 * after X's leading underscore (`vfprintf`/`_vsnprintf`/...). process.c has no such
 * twins of its own (`_execl` &c. have no documented va_list form), so the Rust side
 * invents the same convention rather than a new one: `_wexecl`'s twin is `_vwexecl`,
 * `_spawnlpe`'s is `_vspawnlpe`. None of the sixteen collide with a real msvcrt.spec
 * export (checked).
 *
 * WHY ITS OWN TU, and not a block left behind in process.c: symbol replacement flips a
 * TU's CALLERS, not its HOUSEMATES. Left in process.c these trampolines would keep
 * calling process.c's own C bodies — intra-TU inlining never consults the linker — and
 * the flip would be a green build over dead Rust.
 *
 * Link order carries the same weight: this file is compiled into the msvcrt MODULE
 * target, not into msvcrt_static. Module objects are resolved before any archive
 * member, so these definitions win first-match over process.o's copies
 * (--allow-multiple-definition), exactly as --whole-archive does for
 * libmsvcrt_crt_cargo.a.
 *
 * Double-walk families (`_execle`, `_execlpe`, `_wexecle`, `_wexeclpe`, and their four
 * `_spawn` counterparts): process.c calls `va_start` on the same parameter TWICE — once
 * to build the command line, once more to skip past the argument list and read `envp`.
 * Only ONE `va_start`/`va_end` pair happens here, and the resulting single `va_list`
 * snapshot is handed to the Rust twin once; the twin does both walks itself from its own
 * copies of that one snapshot. See the comment on the `_exec*` twin block in process.rs
 * for why that is equivalent to the C's two `va_start` calls on the flat-pointer Windows
 * `va_list` model we build for.
 *
 * Every body below is process.c verbatim (`va_start`/call-twin/`va_end`/`return`); no
 * rewrites in this TU.
 */

#include <stdarg.h>

#include "msvcrt.h"

/*
 * The sixteen Rust twins. Declared here because they are not exported (not in
 * msvcrt.spec) and appear in no header — an implicit declaration would silently assume
 * `int(...)` and mis-pass the `intptr_t`/`va_list` arguments.
 */
intptr_t CDECL _vwexecl(const wchar_t *name, const wchar_t *arg0, va_list ap);
intptr_t CDECL _vexecl(const char *name, const char *arg0, va_list ap);
intptr_t CDECL _vwexecle(const wchar_t *name, const wchar_t *arg0, va_list ap);
intptr_t CDECL _vexecle(const char *name, const char *arg0, va_list ap);
intptr_t CDECL _vwexeclp(const wchar_t *name, const wchar_t *arg0, va_list ap);
intptr_t CDECL _vexeclp(const char *name, const char *arg0, va_list ap);
intptr_t CDECL _vwexeclpe(const wchar_t *name, const wchar_t *arg0, va_list ap);
intptr_t CDECL _vexeclpe(const char *name, const char *arg0, va_list ap);
intptr_t CDECL _vwspawnl(int flags, const wchar_t *name, const wchar_t *arg0, va_list ap);
intptr_t CDECL _vspawnl(int flags, const char *name, const char *arg0, va_list ap);
intptr_t CDECL _vwspawnle(int flags, const wchar_t *name, const wchar_t *arg0, va_list ap);
intptr_t CDECL _vspawnle(int flags, const char *name, const char *arg0, va_list ap);
intptr_t CDECL _vwspawnlp(int flags, const wchar_t *name, const wchar_t *arg0, va_list ap);
intptr_t CDECL _vspawnlp(int flags, const char *name, const char *arg0, va_list ap);
intptr_t CDECL _vwspawnlpe(int flags, const wchar_t *name, const wchar_t *arg0, va_list ap);
intptr_t CDECL _vspawnlpe(int flags, const char *name, const char *arg0, va_list ap);

/* ---- _exec* family (process.c:390-598) ----------------------------------- */

/*********************************************************************
 *      _wexecl (MSVCRT.@)
 */
intptr_t WINAPIV _wexecl(const wchar_t *name, const wchar_t *arg0, ...)
{
    va_list ap;
    intptr_t ret;

    va_start(ap, arg0);
    ret = _vwexecl(name, arg0, ap);
    va_end(ap);

    return ret;
}

/*********************************************************************
 *		_execl (MSVCRT.@)
 */
intptr_t WINAPIV _execl(const char *name, const char *arg0, ...)
{
    va_list ap;
    intptr_t ret;

    va_start(ap, arg0);
    ret = _vexecl(name, arg0, ap);
    va_end(ap);

    return ret;
}

/*********************************************************************
 *      _wexecle (MSVCRT.@)
 */
intptr_t WINAPIV _wexecle(const wchar_t *name, const wchar_t *arg0, ...)
{
    va_list ap;
    intptr_t ret;

    va_start(ap, arg0);
    ret = _vwexecle(name, arg0, ap);
    va_end(ap);

    return ret;
}

/*********************************************************************
 *		_execle (MSVCRT.@)
 */
intptr_t WINAPIV _execle(const char *name, const char *arg0, ...)
{
    va_list ap;
    intptr_t ret;

    va_start(ap, arg0);
    ret = _vexecle(name, arg0, ap);
    va_end(ap);

    return ret;
}

/*********************************************************************
 *      _wexeclp (MSVCRT.@)
 */
intptr_t WINAPIV _wexeclp(const wchar_t *name, const wchar_t *arg0, ...)
{
    va_list ap;
    intptr_t ret;

    va_start(ap, arg0);
    ret = _vwexeclp(name, arg0, ap);
    va_end(ap);

    return ret;
}

/*********************************************************************
 *		_execlp (MSVCRT.@)
 */
intptr_t WINAPIV _execlp(const char *name, const char *arg0, ...)
{
    va_list ap;
    intptr_t ret;

    va_start(ap, arg0);
    ret = _vexeclp(name, arg0, ap);
    va_end(ap);

    return ret;
}

/*********************************************************************
 *      _wexeclpe (MSVCRT.@)
 */
intptr_t WINAPIV _wexeclpe(const wchar_t *name, const wchar_t *arg0, ...)
{
    va_list ap;
    intptr_t ret;

    va_start(ap, arg0);
    ret = _vwexeclpe(name, arg0, ap);
    va_end(ap);

    return ret;
}

/*********************************************************************
 *		_execlpe (MSVCRT.@)
 */
intptr_t WINAPIV _execlpe(const char *name, const char *arg0, ...)
{
    va_list ap;
    intptr_t ret;

    va_start(ap, arg0);
    ret = _vexeclpe(name, arg0, ap);
    va_end(ap);

    return ret;
}

/* ---- _spawn* family (process.c:685-892) ----------------------------------- */

/*********************************************************************
 *      _wspawnl (MSVCRT.@)
 */
intptr_t WINAPIV _wspawnl(int flags, const wchar_t *name, const wchar_t *arg0, ...)
{
    va_list ap;
    intptr_t ret;

    va_start(ap, arg0);
    ret = _vwspawnl(flags, name, arg0, ap);
    va_end(ap);

    return ret;
}

/*********************************************************************
 *		_spawnl (MSVCRT.@)
 */
intptr_t WINAPIV _spawnl(int flags, const char *name, const char *arg0, ...)
{
    va_list ap;
    intptr_t ret;

    va_start(ap, arg0);
    ret = _vspawnl(flags, name, arg0, ap);
    va_end(ap);

    return ret;
}

/*********************************************************************
 *      _wspawnle (MSVCRT.@)
 */
intptr_t WINAPIV _wspawnle(int flags, const wchar_t *name, const wchar_t *arg0, ...)
{
    va_list ap;
    intptr_t ret;

    va_start(ap, arg0);
    ret = _vwspawnle(flags, name, arg0, ap);
    va_end(ap);

    return ret;
}

/*********************************************************************
 *		_spawnle (MSVCRT.@)
 */
intptr_t WINAPIV _spawnle(int flags, const char *name, const char *arg0, ...)
{
    va_list ap;
    intptr_t ret;

    va_start(ap, arg0);
    ret = _vspawnle(flags, name, arg0, ap);
    va_end(ap);

    return ret;
}

/*********************************************************************
 *      _wspawnlp (MSVCRT.@)
 */
intptr_t WINAPIV _wspawnlp(int flags, const wchar_t *name, const wchar_t *arg0, ...)
{
    va_list ap;
    intptr_t ret;

    va_start(ap, arg0);
    ret = _vwspawnlp(flags, name, arg0, ap);
    va_end(ap);

    return ret;
}

/*********************************************************************
 *		_spawnlp (MSVCRT.@)
 */
intptr_t WINAPIV _spawnlp(int flags, const char *name, const char *arg0, ...)
{
    va_list ap;
    intptr_t ret;

    va_start(ap, arg0);
    ret = _vspawnlp(flags, name, arg0, ap);
    va_end(ap);

    return ret;
}

/*********************************************************************
 *      _wspawnlpe (MSVCRT.@)
 */
intptr_t WINAPIV _wspawnlpe(int flags, const wchar_t *name, const wchar_t *arg0, ...)
{
    va_list ap;
    intptr_t ret;

    va_start(ap, arg0);
    ret = _vwspawnlpe(flags, name, arg0, ap);
    va_end(ap);

    return ret;
}

/*********************************************************************
 *		_spawnlpe (MSVCRT.@)
 */
intptr_t WINAPIV _spawnlpe(int flags, const char *name, const char *arg0, ...)
{
    va_list ap;
    intptr_t ret;

    va_start(ap, arg0);
    ret = _vspawnlpe(flags, name, arg0, ap);
    va_end(ap);

    return ret;
}
