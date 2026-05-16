```toml
[symbols._strtoi64_l]
c_signature = "__int64 CDECL _strtoi64_l(const char *nptr, char **endptr, int base, _locale_t locale)"
family = "radix"
slice_pattern = "nul_in"
core_fn = "NEW:parse_i64_clamping"
return_collapse = "i64_passthrough"
inout_effects = "endptr written with pointer to first unparsed char if got_digit, else original nptr; written regardless of overflow"
preserved_misfeature = "clamps to I64_MAX/I64_MIN on overflow WITHOUT setting errno (lines 73-74, 77-78: `#ifndef _LIBCNT_` guards errno); locale param ignored (line 15 comment); NULL nptr crashes under _LIBCNT_ (line 24 `#ifndef __REACTOS__`)"
fuzz_domain = ["valid_digits_no_overflow", "overflow_positive", "overflow_negative", "leading_whitespace", "plus_minus_signs", "hex_prefix_0x", "octal_prefix_0", "base_0_inference", "invalid_digit_midstring", "no_valid_digits", "base_out_of_range_2_to_36", "empty_after_sign", "endptr_null", "endptr_nonnull"]
```

## Rationale

**Exact return value (Q1):** The function parses `nptr` as a base-N signed integer, returning the accumulated `__int64` value. On overflow, it **clamps** to `_I64_MAX` (lines 72-75) or `_I64_MIN` (lines 76-79) rather than wrapping. The overflow checks compare `ret > _I64_MAX/base` and `ret*base > _I64_MAX-v` before accumulating, setting `ret` to the sentinel and continuing to advance `nptr`. Crucially, under `_LIBCNT_` (the cntpr build), the `*_errno() = ERANGE` assignments are compiled out (lines 74, 78), so **errno is never set** — the clamp is silent. Base validation (lines 24-26) returns 0 for invalid base, but only if PMT checks are enabled (they are under libcnt per the guard structure).

**Which parameters written (Q2):** `endptr` is written if non-NULL (line 84). It receives a pointer to the first character that could not be parsed as a valid digit in the given base. If `got_digit` is TRUE (at least one valid digit was consumed), `endptr` points to `nptr` (the position after the last valid digit). If `got_digit` is FALSE (no digits matched, e.g., input was `"xyz"` or empty after sign), `endptr` points to the original `p` (the start after leading whitespace/sign stripping). This is the standard `strtol` family contract. The `endptr` write happens **even if overflow occurred** (line 84 is unconditional after the loop).

**NULL/zero-length/not-found (Q3):** Under `_LIBCNT_` (ReactOS cntpr), the NULL check for `nptr` is **disabled** (line 24 `#ifndef __REACTOS__`), so `_strtoi64_l(NULL, ...)` will **dereference and crash**. This is the critical deviation. Zero-length input (empty string or no valid digits after whitespace/sign) sets `got_digit = FALSE`, returns `ret = 0` (unchanged from initialization line 18), and `endptr` points to the original input start (line 84). If base is out of range (lines 24-26), the function returns 0 immediately (standard PMT behavior, though the PMT macros might be no-ops under libcnt).

**Deliberate deviation (Q4):**
1. **Clamping without errno under libcnt** (lines 73-74, 77-78): The `#ifndef _LIBCNT_` guards mean overflow is clamped to `_I64_MAX/_I64_MIN` but `errno` is **not** set to `ERANGE` in the cntpr build. Textbook `strtoll` sets errno.
2. **NULL nptr crashes in libcnt** (line 24): The `#ifndef __REACTOS__` disables the NULL safety check, allowing a NULL dereference crash at line 28 (`isspace(*nptr)`). Standard libc `strtoll` often tolerates NULL (returning 0 and setting `endptr` to NULL or leaving it unchanged).
3. **Locale ignored** (line 15 comment): The `locale` parameter is accepted but unused. Textbook `_strtoi64_l` would use locale for character classification (`isspace`, `isdigit`, `tolower`). Here, they use the global locale implicitly.
4. **Base inference** (lines 35-43): If `base==0`, the function infers base 16 from `"0x"` prefix, base 8 from `"0"` prefix, else base 10. This is standard, but the sequence (hex check first even if base is explicitly 16) is worth noting.

**Minimal test cases (Q5):**
- **valid_digits_no_overflow**: `"12345"` base 10 → parse correctly, `endptr` at end.
- **overflow_positive**: `"9223372036854775808"` (I64_MAX+1) base 10 → clamp to I64_MAX, `endptr` at end, **errno unchanged** under libcnt.
- **overflow_negative**: `"-9223372036854775809"` (I64_MIN-1) base 10 → clamp to I64_MIN, `endptr` at end, **errno unchanged**.
- **leading_whitespace**: `"  \t\n42"` → skip whitespace (line 28), parse 42.
- **plus_minus_signs**: `"+123"` and `"-123"` → handle sign (lines 30-33).
- **hex_prefix_0x**: `"0x1a"` base 0 or 16 → infer/use base 16 (lines 35-38).
- **octal_prefix_0**: `"077"` base 0 → infer base 8 (lines 40-43).
- **base_0_inference**: `"99"` base 0 → infer base 10 (line 42).
- **invalid_digit_midstring**: `"12z34"` base 10 → parse 12, `endptr` at 'z' (lines 48-56 break).
- **no_valid_digits**: `"xyz"` → `got_digit=FALSE`, return 0, `endptr` at original start (line 84).
- **base_out_of_range_2_to_36**: `base=1` or `base=37` → return 0 immediately (lines 24-26, if PMT active).
- **empty_after_sign**: `"+"` or `"-"` → no digits, return 0, `endptr` at the sign character (since `p` is set before sign processing, line 16, and `got_digit=FALSE` points to `p`).
- **endptr_null**: `endptr==NULL` → no write, normal parse.
- **endptr_nonnull**: `endptr!=NULL` → write first unparsed char location.

**NEW core function needed:** `parse_i64_clamping(s: &str, base: u32) -> (i64, *const u8, bool /*got_digit*/)`. Returns `(value, first_unparsed_byte_ptr, got_digit_flag)`. The core clamps on overflow (no Result/error), matching the libcnt behavior. The shim reconstructs `endptr` from `first_unparsed_byte_ptr` if `got_digit`, else uses the original `p` pointer. The core does NOT skip leading whitespace or handle signs — the shim does that (`isspace` loop line 28, sign check lines 30-33) before calling the core on the trimmed slice. The core also does base inference (0x/octal/decimal) if `base==0`, per lines 35-43.