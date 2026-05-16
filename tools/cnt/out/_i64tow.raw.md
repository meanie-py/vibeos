```toml
[symbols._i64tow]
c_signature = "wchar_t * _i64tow(__int64 value, wchar_t *string, int radix)"
family = "radix"
slice_pattern = "ret_dst"
core_fn = "NEW:radix::format_i64_wide"
return_collapse = "ret_dst"
inout_effects = "writes NUL-terminated wide string to `string` if non-NULL; always returns `string` unchanged (even if NULL)"
preserved_misfeature = "Returns `string` even when NULL (line 47), allowing the caller to pass NULL and get NULL back without crash—textbook behavior would be undefined. No validation of radix bounds (accepts invalid radix silently, behavior implementation-defined). Negative values only signed for radix==10 (line 21); other radices treat value as unsigned bit pattern (line 25 casts to ULONGLONG)."
fuzz_domain = ["value={-1, 0, 1, INT64_MIN, INT64_MAX}", "radix={2, 10, 16, 36, 1, 37}", "string={NULL, valid_ptr}", "negative_radix_10={true, false}", "single_digit={true, false}", "max_digits_base2"]
```

## Rationale

Analyzing `_i64tow` from lines 16-48:

**1. Return value:** Always returns the input `string` pointer verbatim (line 47), regardless of whether it's NULL. If `string` is NULL, no conversion output occurs (line 45-46 guard), but the function still returns NULL without error—caller responsibility to check.

**2. Output written:** If `string != NULL`, writes the NUL-terminated wide-character string representation of `value` in the specified `radix` (lines 45-46). Uses a 65-wchar_t stack buffer (line 19), builds the string backwards from position 64 (lines 21-22, 27-37), then `memcpy`s the result including the NUL terminator (line 46: `&buffer[64] - pos + 1` calculates length including NUL). The output is:
   - Sign prefix: `'-'` only if `value < 0 && radix == 10` (lines 21, 39-40)
   - Digits: `'0'-'9'` for digit < 10, `'a'-'z'` for digit >= 10 (lines 32-36)
   - NUL terminator included in copy (line 46)

**3. NULL behavior:** Explicitly allows `string == NULL` (line 45) without crash—function completes conversion to internal buffer but skips the `memcpy`, returning NULL. This is a deliberate ReactOS extension; standard `_i64tow` typically requires non-NULL output.

**4. Deviations from textbook:**
   - **NULL tolerance (line 45):** Standard libc would invoke undefined behavior; ReactOS makes it a safe no-op.
   - **No radix validation:** Unlike `_i64tow_s` (lines 55-57), this function accepts any `int radix` without bounds checking. Radix < 2 or > 36 causes implementation-defined behavior (likely modulo/divide by bad radix). The code assumes valid radix in the digit conversion logic (lines 32-36).
   - **Signed/unsigned handling (lines 21-25):** Negative values only treated as signed for `radix == 10`. For all other radices (2, 8, 16, etc.), the `__int64` is cast to `ULONGLONG` (line 25), treating the bit pattern as unsigned. This matches MSVC behavior but differs from some libc implementations that might sign-extend or error on negative non-decimal.

**5. Fuzzing domains:**
   - **Value boundaries:** `{-1, 0, 1, INT64_MIN, INT64_MAX}` to test sign handling, zero, and overflow edge cases for the negation (line 23: `-value` when `value == INT64_MIN` wraps in two's complement, yielding `ULONGLONG` bit pattern).
   - **Radix:** `{2, 10, 16, 36}` (valid extremes and common bases), `{1, 37}` (invalid to confirm no crash), `10` vs non-10 for sign logic bifurcation (line 21).
   - **String pointer:** `{NULL, valid_ptr}` to test the NULL guard (line 45).
   - **Digit count:** Single-digit (0-9 in any valid radix) vs multi-digit to ensure loop termination (line 38) and buffer arithmetic (line 46).
   - **Maximum digits:** Base-2 representation of `INT64_MIN` (treated as `ULONGLONG` max) requires 64 digits + sign + NUL = 66 wchars, but buffer is 65 (line 19). Testing `value == INT64_MIN, radix == 2` confirms if this overruns (it doesn't: unsigned cast means no sign, 64 digits + NUL = 65 exactly).

**Core function needed:** `NEW:radix::format_i64_wide` with signature:
```
fn format_i64_wide(value: i64, radix: i32) -> Vec<u16>
```
Returns a NUL-terminated UTF-16 vector matching the C behavior: signed conversion for radix==10, unsigned bit pattern otherwise, lowercase hex digits `'a'-'z'`. The shim reconstructs a mutable wide slice from `string` if non-NULL, calls the core, writes the result, and returns `string`.