```toml
[symbols.strtoul]
c_signature = "unsigned long strtoul(const char* nptr, char** end, int base)"
family = "radix"
slice_pattern = "nul_in"
core_fn = "NEW:radix::parse_u64_clamped"
return_collapse = "u64_trunc_ulong"
inout_effects = "*end written to first invalid char (or nptr if no parse); no other mutation"
preserved_misfeature = "Negative i64 parse results <-ULONG_MAX clamp to 1 (not 0); delegates to _strtoi64_l so inherits signed parse then range-checks the i64, asymmetric clamp [1, ULONG_MAX]"
fuzz_domain = ["valid in-range", "overflow >ULONG_MAX", "negative valid magnitude ≤ULONG_MAX", "negative magnitude >ULONG_MAX (→1)", "leading whitespace", "invalid base", "no digits", "endptr tracking"]
```

## Rationale

**Source lines 9-10, 16-17**: The function delegates to `_strtoi64_l` (signed 64-bit parse), then performs range validation on the *signed* i64 result. This creates asymmetric clamping behavior.

**Preserved misfeature (lines 12-17)**: 
- Values `> ULONG_MAX` clamp to `ULONG_MAX` (expected)
- Values `< -(__int64)ULONG_MAX` clamp to **1** (not 0 or wrapping behavior)

This is because the signed parse can yield large negative i64 values. When `ret < -ULONG_MAX` (e.g., parsing "-4294967296" or lower on 32-bit), it clamps to 1, not the wraparound value or 0. This is the load-bearing deviation: textbook `strtoul` would parse unsigned directly; ReactOS parses signed then clamps asymmetrically.

**Return collapse**: The i64 result is implicitly truncated to `unsigned long` (line 19), so we need `u64_trunc_ulong` to model the truncation from core's u64.

**Inout effects**: Line 9 shows `end` is passed through to `_strtoi64_l`, which writes `*end` to the first character that stopped the parse (or `nptr` if no valid parse). No other parameters are mutated.

**Core function needed**: `NEW:radix::parse_u64_clamped` with signature `fn(input: &str, base: i32) -> (u64, endpos: usize, clamped: ClampResult)` where `ClampResult` encodes {`InRange`, `High`, `LowNeg`, `LowNegExtreme`}. The shim must detect `LowNegExtreme` (parsed i64 < -ULONG_MAX) to return 1, else clamp to `ULONG_MAX` on `High`, else cast the u64 to `unsigned long`.

**Fuzz domains** must cover:
1. Valid unsigned values in range
2. Overflow >ULONG_MAX (→ ULONG_MAX)
3. Small negative values like "-100" (→ wraps as unsigned in i64 parse, then clamped)
4. Extreme negative values <-ULONG_MAX (→ 1)
5. Leading whitespace (inherited from _strtoi64_l)
6. Invalid base or no valid digits (endptr semantics)
7. Endptr update correctness at boundaries

The core must faithfully reproduce the signed-parse-then-clamp logic to maintain the `ret=1` misfeature for extreme negatives.