```toml
[symbols._strtoul_l]
c_signature = "unsigned long _strtoul_l(const char* nptr, char** end, int base, _locale_t locale)"
family = "radix"
slice_pattern = "nul_in"
core_fn = "NEW:radix::parse_u64_l"
return_collapse = "i64_clamp_ulong"
inout_effects = "writes *end to point after last parsed char (via _strtoi64_l delegation); if nptr invalid/empty, *end may equal nptr"
preserved_misfeature = "Negative i64 results below -ULONG_MAX clamp to 1 (NOT 0), lines 16-20. Positive i64 above ULONG_MAX clamps to ULONG_MAX (lines 12-15). Silently saturates without setting errno in libcntpr (_LIBCNT_ defined). Delegates to _strtoi64_l which parses as signed i64, then post-clamps the signed result into unsigned long range with asymmetric negative handling."
fuzz_domain = ["valid_decimal_in_range", "hex_0x_prefix", "octal_0_prefix", "whitespace_prefix", "plus_sign", "minus_sign_small_negative", "minus_sign_large_negative_below_minus_ULONG_MAX", "overflow_above_ULONG_MAX", "overflow_far_above", "empty_string", "invalid_chars", "base_0_auto", "base_2_to_36", "invalid_base", "null_end_ptr", "endptr_tracking"]
```

## Rationale

**Exact return semantics (lines 9-21):**
`_strtoul_l` delegates to `_strtoi64_l(nptr, end, base, locale)` which parses the input as a *signed* i64. The function then applies asymmetric clamping:
- If `ret > ULONG_MAX` (4294967295 on 32-bit, but this is i64 context): clamp to `ULONG_MAX` (line 13).
- If `ret < -(__int64)ULONG_MAX` (i.e., < -4294967295): clamp to `1` (line 17), NOT zero. This is the critical misfeature—large negative values wrap to 1.
- Otherwise, return `ret` cast to `unsigned long` (line 22), which naturally handles small negatives via two's complement truncation.

The libcntpr build (`_LIBCNT_` defined) omits `errno` assignments (lines 14, 19), making overflow silent.

**Inout effects (implicit via delegation):**
`_strtoi64_l` writes `*end` to point past the last successfully parsed character. If parsing fails immediately, `*end == nptr`. If `end` is NULL, no write occurs (standard tolerance).

**Deliberate deviation (lines 16-20):**
Textbook `strtoul` clamps negative overflows to 0. ReactOS clamps large negatives (below -ULONG_MAX) to **1**. This is explicit in the source and is NOT a typo—it's the shipped behavior and must be preserved exactly. Small negatives (e.g., "-1") naturally become `ULONG_MAX - 1` via i64→ulong truncation, but "-5000000000" becomes 1.

**Required new core:**
`NEW:radix::parse_u64_l` must replicate `_strtoi64_l`'s signed i64 parsing (including locale support), returning `(i64, *endptr_offset)`. The shim then applies the exact asymmetric clamp logic and casts to `unsigned long`.

**Fuzz domain:**
Must cover: valid small/large positives, ULONG_MAX boundary, overflow above ULONG_MAX, small negatives (natural truncation), large negatives triggering the "clamp to 1" path, bases 0/2-36, whitespace/sign prefixes, empty/invalid strings, and NULL `end` pointer tolerance.