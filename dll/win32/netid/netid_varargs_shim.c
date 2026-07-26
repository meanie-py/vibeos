/*
 * netid_varargs_shim.c — the variadic entry point for FormatMessageBox.
 *
 * Stable Rust cannot DEFINE a C-variadic function (rust-lang/rust#44930), and `va_start` may
 * only legally appear inside such a definition. So this file supplies the entry point and
 * nothing else: it starts the va_list and immediately hands control back to Rust, passing a
 * POINTER to the list. Every line of real logic — the two LoadStringW calls, FormatMessageW,
 * MessageBoxW — stays in netid-rust. This is a shim for a language gap, not a translation
 * reverted to C. See sdk/lib/rust/varargs-rust/src/lib.rs for the pattern and its contract.
 *
 * Pointer-to-va_list rather than the va_list itself: it is `char*` on our three Windows
 * targets but an array-of-struct on SysV x86-64, and the by-pointer form is correct on both.
 *
 * WINAPIV (__cdecl), not WINAPI: a variadic cannot be stdcall, because the callee cannot know
 * how many argument bytes to pop.
 */

#include <stdarg.h>
#include <windef.h>
#include <winuser.h>

int __cdecl netid_FormatMessageBox_body(HWND hDlg, UINT uType, DWORD dwMessage, void *ap);

int WINAPIV
FormatMessageBox(HWND hDlg, UINT uType, DWORD dwMessage, ...)
{
    va_list args;
    int result;

    va_start(args, dwMessage);
    result = netid_FormatMessageBox_body(hDlg, uType, dwMessage, &args);
    va_end(args);

    return result;
}
