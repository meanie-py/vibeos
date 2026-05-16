```toml
[symbols._ltow]
c_signature = "wchar_t* _ltow(long value, wchar_t *string, int radix)"
family = "radix"
slice_pattern = "dst_unchecked"
core_fn = "NEW:radix::format_i32_wide"
return_collapse = "ret_dst"
inout_effects = "string written with NUL-terminated base-`radix` representation of value (negative sign for radix==10 negatives, lowercase hex a-z for digits>9); ENTIRE buffer written even if string==NULL (misfeature: the function still performs formatting into temp buffer, just doesn't copy)"
preserved_misfeature = "Returns the input `string` pointer UNCONDITIONALLY (lines 243-244), even if string==NULL. Textbook would return NULL or assert. Also: negative values with radix!=10 are cast to unsigned (line 231 `val = value`), producing large positive output. Also: no radix validation—invalid radix (e.g., 1, 37) silently divides-by-zero or produces garbage."
fuzz_domain = ["value={-1,0,1,INT32_MIN,INT32_MAX}", "radix={2,10,16,36,1,37}", "string={valid_ptr,NULL}"]
```

## Rationale

**1. Exact return value:** Lines 243-244 unconditionally `return string`, regardless of NULL. The formatted result is the base-`radix` representation using '0'-'9' for digits 0-9, 'a'-'z' for 10-35 (lines 237-240), with '-' prefix only for negative values when `radix == 10` (lines 228-231, 241-242). No error codes—always returns `string`.

**2. Which parameters written:** The `string` parameter (if non-NULL) receives the NUL-terminated result via `memcpy` (line 243). Formatting always occurs into local `buffer[33]`, building the string backwards from position 32 (lines 233-242), but the copy to `string` is guarded by the NULL check (line 243). Crucially, the temp buffer is ALWAYS populated even when `string==NULL`, wasting work—this is the implementation strategy, not a side-effect we document, but it explains why NULL doesn't short-circuit.

**3. NULL behavior:** Line 243: `if (string != NULL)` guards the memcpy, so NULL is safe—no crash. However, the function still returns the NULL pointer as-is (line 244 is unconditional). This is bizarre but explicit: the caller gets their NULL back, presumably to chain or detect failure, but the function provides no failure signal.

**4. Deliberate deviations:**
   - **NULL returns NULL (not crash, not error):** Line 243 guards memcpy; line 244 returns `string` whether NULL or not. Textbook `ltow` is unspecified for NULL, but typical implementations would crash or return NULL as error—ReactOS returns the NULL input as success-ambiguous.
   - **Negative non-decimal treated as unsigned:** Lines 228-231: `if (value < 0 && radix == 10)` negates; else `val = value` (unsigned assignment). A negative value with radix 16 produces the large unsigned hex (e.g., -1 → 0xFFFFFFFF on 32-bit long). Textbook is silent, but this is unusual.
   - **No radix validation:** Lines 235-240 use `val % radix` and `val / radix` with no check that `radix >= 2 && radix <= 36`. Radix 1 divides by zero; radix > 36 produces bogus characters (e.g., `'a' + 40 - 10` = out of range). The `_s` variant (lines 247-313) validates, but the non-`_s` does not.

**5. Differential test must cover:**
   - **Negative radix-10:** value=-123, radix=10 → "-123" (sign handling)
   - **Negative radix-16:** value=-1, radix=16 → "ffffffff" (or "ffffffffffffffff" on 64-bit long—assumes 32-bit) via unsigned cast
   - **Zero:** value=0, radix=10 → "0" (do-while ensures at least one digit)
   - **Extremes:** value=INT32_MAX, radix=2 (31 '1's + sign bit); value=INT32_MIN, radix=10 → overflow/wraparound or large positive via unsigned cast
   - **Radix boundaries:** radix=2, 10, 16, 36 (valid), radix=1 (div-by-zero UB), radix=37 (char overflow)
   - **NULL string:** value=42, string=NULL → returns NULL, no crash (memcpy skipped)

**Core needed:** `NEW:radix::format_i32_wide(value: i32, radix: u32) -> WideString` — formats signed 32-bit `long` (assuming ILP32/LLP64 where `long` is 4 bytes, per buffer[33] size for 32 bits + sign + NUL) into wide-char string with ReactOS semantics: radix-10 negatives prefixed '-', others cast to unsigned; digits '0'-'9', 'a'-'z'; no validation. Returns heap or stack wide string (or `Vec<u16>`) for the shim to memcpy if dst non-NULL. The shim must NOT validate radix (preserve the UB), and must return the input pointer unconditionally.