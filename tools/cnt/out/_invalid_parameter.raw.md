```toml
[symbols._invalid_parameter]
c_signature = "void __cdecl _invalid_parameter(wchar_t const* expression, wchar_t const* function_name, wchar_t const* file_name, unsigned int line_number, uintptr_t reserved)"
family = "printf"
slice_pattern = "nul_in"
core_fn = "NEW:debug_print_invalid_parameter"
return_collapse = "void"
inout_effects = "none (read-only: formats diagnostic from all params, writes to debug output only)"
preserved_misfeature = "none - textbook CRT invalid parameter handler that prints diagnostic via DbgPrint and returns (does NOT terminate process, unlike standard _invalid_parameter which typically calls _invoke_watson)"
fuzz_domain = ["all_valid_wstrs", "mixed_nulls (any/all of expression/function_name/file_name NULL)", "line_zero", "reserved_nonzero"]
```

## Rationale

This is ReactOS's NT-specific implementation of the CRT invalid parameter handler. The function signature takes 5 parameters: three optional wide-string pointers (`_In_opt_z_` annotation confirms NULL is valid), an unsigned line number, and a reserved `uintptr_t`.

**Exact return behavior (Q1):** Returns `void` after printing diagnostic. No error codes, no exceptions.

**Which parameters written (Q2):** None. All string parameters are `const*` and read-only. The function only *reads* them to format a diagnostic message via `DbgPrint`. Writes occur only to the debug output stream, not to any caller-provided buffers.

**NULL behavior (Q3):** The `_In_opt_z_` SAL annotations explicitly mark `expression`, `function_name`, and `file_name` as optional (may be NULL). The source passes all three directly to `DbgPrint` with `%ws` format, which in ReactOS's DbgPrint implementation handles NULL wide-string pointers gracefully (typically prints "(null)" or similar). No dereferencing precedes the DbgPrint call, so NULL inputs do not crash.

**Deviation from textbook (Q4):** The critical deviation is that this implementation **does NOT terminate the process**. Standard MSVC `_invalid_parameter` (when no custom handler is installed) invokes `_invoke_watson`, which terminates the process after attempting to generate a crash dump. ReactOS's NT version merely prints the diagnostic and returns control to the caller. This is load-bearing behavior for ReactOS—it allows NT kernel-mode and certain system components to log parameter validation failures without halting, which would be catastrophic in kernel context. The function also ignores the `reserved` parameter entirely (textbook CRT reserves it for future use but this implementation doesn't store or check it).

**Fuzz domain (Q5):**
- **all_valid_wstrs**: All four string params valid, non-NULL, non-empty wide strings, line_number nonzero, reserved=0 (baseline path)
- **mixed_nulls**: Any combination of expression/function_name/file_name being NULL (exercises `%ws` NULL-handling in DbgPrint)
- **line_zero**: `line_number == 0` (edge case for `%u` formatting)
- **reserved_nonzero**: Non-zero reserved value (confirms it's truly unused/ignored)

**Core needed:** `NEW:debug_print_invalid_parameter` — Takes four optional wide-string slices (`Option<&[u16]>`), a `u32` line number, and a `usize` reserved param. Formats the diagnostic message matching the `DbgPrint` format string pattern, writes to platform debug output (no-op in pure no_std tests, actual debug output in integration tests), returns `()`. The core does NOT terminate.