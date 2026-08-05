/* rust_varargs_shim.c — the one thing stable Rust cannot express here.
 *
 * DEFINING a C-variadic function: `va_start` is legal only inside one, and that is the
 * `c_variadic` feature (rust-lang/rust#44930 — FCP-complete with a merge disposition, so
 * this file is expected to be DELETED once it stabilizes; see
 * docs/engineering/active-lanes/stable-toolchain-plan.md). *Calling* and *declaring* a
 * variadic C function are already stable, so every aclui-rust call site is untouched.
 *
 * aclui needs no `va_arg` pullers at all — unlike the usbhub/usbstor shims, which format
 * their own output. LoadAndFormatString never inspects its arguments; it hands the whole
 * list to FormatMessageW, which walks it. So this is the variadic entry point and nothing
 * else: three lines back into Rust, where the real body (AllocAndLoadString, the
 * FormatMessageW call, the LocalFree) stays. A shim for a language gap, NOT a Rust
 * translation reverted to C (AGENTS.md "the goal is Rust").
 *
 * We hand Rust a POINTER to the va_list (`&ap`), never the va_list itself — that avoids
 * assuming `va_list` is a single pointer (true on our three Windows targets, false on
 * SysV x86-64), and it is the shape FormatMessageW's `Arguments` parameter wants anyway,
 * so the Rust side forwards it unchanged.
 *
 * Template: sdk/lib/rust/varargs-rust/rust_varargs_shim.c.template
 */

#include <stdarg.h>
#include <stddef.h>

typedef unsigned long DWORD_C;
typedef unsigned int UINT_C;
typedef unsigned short WCHAR_C;

extern DWORD_C LoadAndFormatString_v(
    void *hInstance, UINT_C uID, WCHAR_C **lpTarget, void *ap);

DWORD_C LoadAndFormatString(
    void *hInstance, UINT_C uID, WCHAR_C **lpTarget, ...)
{
    va_list ap;
    DWORD_C ret;

    va_start(ap, lpTarget);
    ret = LoadAndFormatString_v(hInstance, uID, lpTarget, &ap);
    va_end(ap);
    return ret;
}
