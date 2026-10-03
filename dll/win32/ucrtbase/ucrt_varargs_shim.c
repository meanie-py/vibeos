/*
 * PROJECT:     ReactOS ucrtbase (vibeos Rust port)
 * LICENSE:     MIT (see sdk/lib/ucrt for the bodies these entry points front)
 * PURPOSE:     The irreducible C half of ucrt-rust's variadic exports.
 *
 * Stable Rust cannot DEFINE a C-variadic function or va_arg a va_list
 * (`c_variadic`, rust-lang/rust#44930). So this file supplies exactly those two
 * primitives and nothing else: the va_arg pullers sdk/lib/rust/varargs-rust names
 * (PART 1, identical in every module that uses it), and one three-line trampoline
 * per `...` export that does the va_start and hands `&ap` to its Rust `_v` twin
 * (PART 2). All logic -- argv capture, spawning, flag decoding -- is Rust.
 *
 * Built into the ucrtbase MODULE (see CMakeLists.txt), so these win first-match
 * over the archive's copies while a takeover check still links the Microsoft C.
 * Template: sdk/lib/rust/varargs-rust/rust_varargs_shim.c.template.
 */

#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>
#include <wchar.h>

/* ---- PART 1: the va_arg pullers ---------------------------------------------- */
int                vibeos_va_arg_i32(void *app) { return va_arg(*(va_list *)app, int); }
unsigned int       vibeos_va_arg_u32(void *app) { return va_arg(*(va_list *)app, unsigned int); }
long long          vibeos_va_arg_i64(void *app) { return va_arg(*(va_list *)app, long long); }
unsigned long long vibeos_va_arg_u64(void *app) { return va_arg(*(va_list *)app, unsigned long long); }
double             vibeos_va_arg_f64(void *app) { return va_arg(*(va_list *)app, double); }
void *             vibeos_va_arg_ptr(void *app) { return va_arg(*(va_list *)app, void *); }

/* ---- PART 2: the variadic entry points ----------------------------------------- */

/* exec/spawnl.cpp, spawnlp.cpp: the argument list is `arguments, ..., NULL`, and
 * the `e` forms take the environment pointer after the NULL. Rust: src/exec.rs. */
#define SPAWN_TRAMP(name, ch)                                                          \
    intptr_t __cdecl name##_v(int mode, ch const *file_name, ch const *arguments,     \
                              void *ap);                                              \
    intptr_t __cdecl name(int mode, ch const *file_name, ch const *arguments, ...)    \
    {                                                                                 \
        va_list ap;                                                                   \
        va_start(ap, arguments);                                                      \
        intptr_t r = name##_v(mode, file_name, arguments, (void *)&ap);               \
        va_end(ap);                                                                   \
        return r;                                                                     \
    }
#define EXEC_TRAMP(name, ch)                                                           \
    intptr_t __cdecl name##_v(ch const *file_name, ch const *arguments, void *ap);    \
    intptr_t __cdecl name(ch const *file_name, ch const *arguments, ...)              \
    {                                                                                 \
        va_list ap;                                                                   \
        va_start(ap, arguments);                                                      \
        intptr_t r = name##_v(file_name, arguments, (void *)&ap);                     \
        va_end(ap);                                                                   \
        return r;                                                                     \
    }

SPAWN_TRAMP(_spawnl, char)
SPAWN_TRAMP(_spawnle, char)
SPAWN_TRAMP(_spawnlp, char)
SPAWN_TRAMP(_spawnlpe, char)
SPAWN_TRAMP(_wspawnl, wchar_t)
SPAWN_TRAMP(_wspawnle, wchar_t)
SPAWN_TRAMP(_wspawnlp, wchar_t)
SPAWN_TRAMP(_wspawnlpe, wchar_t)
EXEC_TRAMP(_execl, char)
EXEC_TRAMP(_execle, char)
EXEC_TRAMP(_execlp, char)
EXEC_TRAMP(_execlpe, char)
EXEC_TRAMP(_wexecl, wchar_t)
EXEC_TRAMP(_wexecle, wchar_t)
EXEC_TRAMP(_wexeclp, wchar_t)
EXEC_TRAMP(_wexeclpe, wchar_t)

/* lowio/open.cpp: the `...` is the optional pmode, read only for _O_CREAT.
 * Rust: src/lowio_open.rs. */
int __cdecl _open_v(char const *path, int oflag, void *ap);
int __cdecl _open(char const *path, int oflag, ...)
{
    va_list ap; va_start(ap, oflag);
    int r = _open_v(path, oflag, (void *)&ap);
    va_end(ap);
    return r;
}
int __cdecl _wopen_v(wchar_t const *path, int oflag, void *ap);
int __cdecl _wopen(wchar_t const *path, int oflag, ...)
{
    va_list ap; va_start(ap, oflag);
    int r = _wopen_v(path, oflag, (void *)&ap);
    va_end(ap);
    return r;
}
int __cdecl _sopen_v(char const *path, int oflag, int shflag, void *ap);
int __cdecl _sopen(char const *path, int oflag, int shflag, ...)
{
    va_list ap; va_start(ap, shflag);
    int r = _sopen_v(path, oflag, shflag, (void *)&ap);
    va_end(ap);
    return r;
}
int __cdecl _wsopen_v(wchar_t const *path, int oflag, int shflag, void *ap);
int __cdecl _wsopen(wchar_t const *path, int oflag, int shflag, ...)
{
    va_list ap; va_start(ap, shflag);
    int r = _wsopen_v(path, oflag, shflag, (void *)&ap);
    va_end(ap);
    return r;
}
