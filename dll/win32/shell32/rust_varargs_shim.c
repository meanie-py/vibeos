/* INERT since 2026-09-24: not in CMake. Replaced by Rust: ShellMessageBoxA receives its
 * varargs via va-slots-rust (shell32-rust/src/wine/shellord.rs), and StringCchPrintfW is
 * strsafe_rust::StringCchPrintfW!. Kept on disk as the reference. */

/* rust_varargs_shim.c — the ONLY two things stable Rust cannot express.
 *
 *   1. DEFINING a C-variadic function (`va_start` is legal only inside one), and
 *   2. va_arg.
 *
 * Both are the `c_variadic` feature (rust-lang/rust#44930 — FCP-complete with a merge
 * disposition, so this file is expected to be DELETED once it stabilizes; see
 * docs/engineering/active-lanes/stable-toolchain-plan.md). *Calling* and *declaring*
 * a variadic C fn are already stable, so shell32-rust's call sites are untouched.
 *
 * Every entry below is a 3-line trampoline straight back into Rust: the resource
 * loading, the ANSI/Unicode conversion, the FormatMessageA call and the MessageBoxA
 * all stay in shell32-rust/src/wine/shellord.rs. This is a shim for a language gap,
 * NOT a Rust translation reverted to C (AGENTS.md "Rust Is The Goal").
 *
 * We hand Rust a POINTER to the va_list (`&ap`), never the va_list itself — that
 * avoids assuming `va_list` is a single pointer (true on our three Windows targets,
 * false on SysV x86-64) and is what lets varargs-rust be unit-tested on the host.
 * For ShellMessageBoxA that pointer is *exactly* what Win32's FormatMessageA wants for
 * its `Arguments` parameter (a `va_list *`), so the Rust twin forwards it unchanged.
 *
 * Rust side: sdk/lib/rust/varargs-rust (`VaArgs`) + the `*_v` twins in wine/shellord.rs.
 * Template: sdk/lib/rust/varargs-rust/rust_varargs_shim.c.template
 */

#include <stdarg.h>
#include <stddef.h>

/* ---- PART 1: the va_arg pullers (mirrored by varargs-rust's extern block) ------
 * Each takes a POINTER to the caller's va_list and ADVANCES it, so successive pulls
 * from Rust walk the argument list exactly as C's va_arg would.
 */
int                vibeos_va_arg_i32(void *app) { return va_arg(*(va_list *)app, int); }
unsigned int       vibeos_va_arg_u32(void *app) { return va_arg(*(va_list *)app, unsigned int); }
long long          vibeos_va_arg_i64(void *app) { return va_arg(*(va_list *)app, long long); }
unsigned long long vibeos_va_arg_u64(void *app) { return va_arg(*(va_list *)app, unsigned long long); }
double             vibeos_va_arg_f64(void *app) { return va_arg(*(va_list *)app, double); }
void *             vibeos_va_arg_ptr(void *app) { return va_arg(*(va_list *)app, void *); }

/* ---- PART 2: shell32's variadic entry points -----------------------------------
 * ShellMessageBoxA — an EXPORTED ordinal (shell32.spec:183,
 * `183 varargs ShellMessageBoxA(ptr ptr str str long)`), so external callers depend on
 * this exact symbol name and its variadic cdecl (WINAPIV) ABI. Fixed args cross-checked
 * against the .spec and the Rust definition:
 *     HINSTANCE hInstance, HWND hWnd, LPCSTR lpText, LPCSTR lpCaption, UINT uType
 */

extern int ShellMessageBoxA_v(void *hInstance, void *hWnd, const char *lpText,
                              const char *lpCaption, unsigned int uType, void *ap);

int ShellMessageBoxA(void *hInstance, void *hWnd, const char *lpText,
                     const char *lpCaption, unsigned int uType, ...)
{
    va_list ap;
    int ret;
    va_start(ap, uType);
    ret = ShellMessageBoxA_v(hInstance, hWnd, lpText, lpCaption, uType, (void *)&ap);
    va_end(ap);
    return ret;
}
