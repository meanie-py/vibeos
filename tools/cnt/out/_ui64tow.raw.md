```toml
[symbols._ui64tow]
c_signature = "wchar_t * _ui64tow(unsigned __int64 value, wchar_t *string, int radix)"
family = "radix"
slice_pattern = "raw_dst_radix"
core_fn = "NEW:radix::format_u64"
return_collapse = "ret_dst"
inout_effects = "If string != NULL: writes NUL-terminated wchar_t representation of value in base radix (2-36), digits 0-9 then a-z for 10-35. No validation of radix. If string == NULL: no write."
preserved_misfeature = "Returns `string` unconditionally even when NULL (lines 134-135: `if (string != NULL) { memcpy(...) } return string;`). No radix validation (caller can pass invalid radix causing undefined modulo/div). Lowercase 'a-z' for digits 10-35 (lines 126-130)."
fuzz_domain = ["value=0", "value=ULLONG_MAX", "radix=2", "radix=10", "radix=16", "radix=36", "radix=1 (invalid)", "radix=37 (invalid)", "string=NULL", "string=valid buffer", "value requiring all 64 chars (2^64-1 in base 2)"]
```

## Rationale

**1. Exact return value as a function of inputs:**

Lines 137-138: `return string;` — the function unconditionally returns the `string` parameter verbatim, regardless of whether it was NULL or valid. This is the key misfeature: if `string == NULL`, the function returns NULL but performs no write (guarded by line 134).

**2. Which parameters are written, and precisely what:**

Lines 134-136: Only if `string != NULL`, the function writes the NUL-terminated wchar_t representation of `value` in the specified `radix`. The conversion happens in a local 65-element `buffer` (line 118), building the string from right-to-left starting at `&buffer[64]` (lines 121-122). Each digit is computed via `value % radix` (line 126), then converted to '0'-'9' for digits 0-9 (line 128) or 'a'-'z' for digits 10-35 (line 130). The `memcpy` at line 135 copies `(&buffer[64] - pos + 1) * sizeof(WCHAR)` bytes, including the NUL terminator.

**3. NULL / zero-length / not-found behavior:**

Line 134: `if (string != NULL)` — explicit NULL guard. If `string == NULL`, no dereference occurs, no write happens, and NULL is returned. No crash, just a silent no-op with NULL return. `value == 0` still produces "0" (the do-while on lines 125-131 executes at least once, placing '0' before checking `while (value != 0L)`).

**4. Deliberate deviation from textbook libc:**

The most significant deviation is the **NULL-tolerant behavior** with unconditional return of `string` (lines 134-138). Textbook `_ui64tow` would typically require non-NULL `string` or exhibit undefined behavior. Here, NULL is explicitly handled as a query mode or no-op.

Additionally, there is **no radix validation**. Lines 126-130 blindly use `value % radix` and `value / radix` without checking `2 <= radix <= 36`. Invalid radix values (e.g., 0, 1, 37) cause undefined modulo/division behavior, whereas the `_s` variants validate radix (line 144 in `_ui64tow_s`).

Lowercase letters 'a'-'z' for digits 10-35 (line 130) is standard for Microsoft CRT but worth noting as a design choice (some implementations use uppercase).

**5. Minimal input equivalence classes:**

- **value boundaries**: 0 (minimal do-while iteration), ULLONG_MAX (maximal buffer usage, especially in base 2)
- **radix valid boundaries**: 2 (binary, maximal output length: 64 chars), 10 (decimal, common case), 16 (hex, common case), 36 (maximal valid radix using all alphanumeric)
- **radix invalid**: 1 (division by 1 loops infinitely or undefined), 37 (out of valid range), 0 (division by zero)
- **string parameter**: NULL (tests no-write + NULL return), valid buffer (tests normal write path)
- **edge case**: value that requires maximum buffer (2^64 - 1 in base 2 produces a 64-character string plus NUL, exactly filling the 65-element buffer)

**Proposed new core signature (one sentence):**

`radix::format_u64(value: u64, radix: u32, buf: &mut [u16]) -> Result<usize, RadixError>` — writes the NUL-terminated wide-char (u16) representation of `value` in base `radix` (validated 2-36) to `buf`, returning the number of u16s written (including NUL), or error if radix invalid or buffer too small.

The shim must:
- Map `radix` outside 2-36 to undefined/unchecked behavior (matching the non-`_s` variant's lack of validation, possibly documented as "caller must ensure valid radix"),
- Handle `string == NULL` by skipping the core call entirely and returning NULL,
- Otherwise reconstruct a wide-char slice from `string` with sufficient capacity (65 u16s minimum for u64 in base 2), call the core, and return `string`.