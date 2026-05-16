```toml
[symbols._atoi64]
c_signature = "__int64 _atoi64(const char *nptr)"
family = "radix"
slice_pattern = "nul_in"
core_fn = "radix::parse_i64_wrapping"
return_collapse = "i64_passthrough"
inout_effects = "none (read-only)"
preserved_misfeature = "NULL returns 0 (line 21-22). Wrapping overflow with NO error signal: `acc = 10 * acc + ...` (line 35) continues on i64 overflow, wraps silently, then applies negation (line 39-40). Non-digit suffix silently stops parsing (line 34 loop exit)."
fuzz_domain = [
  "NULL",
  "empty string \"\"",
  "whitespace-only \"  \\t\\n\"",
  "valid small \"-123\"",
  "valid large near i64::MAX \"9223372036854775807\"",
  "overflow wrapping \"9223372036854775808\" (i64::MAX+1, wraps to i64::MIN)",
  "double overflow \"92233720368547758070\" (wraps multiple times)",
  "leading zeros \"000123\"",
  "plus sign \"+456\"",
  "trailing non-digits \"789abc\"",
  "non-digit prefix \"abc\" (returns 0)"
]
```

## Rationale

**1. Exact return value:**
- **NULL** → `0` (line 21-22: explicit guard)
- **Empty/whitespace-only** → `0` (lines 24-25 skip whitespace; line 34 loop never enters)
- **Valid integer** → parsed value as `__int64`
- **Overflow** → **wraps silently** (line 35: `acc = 10 * acc + ...` with no overflow check; i64 arithmetic wraps in C undefined behavior, but ReactOS compiled behavior wraps)
- **Non-digit suffix** → stops parsing, returns accumulated value (line 34 loop condition `isdigit`)

The function accumulates into `__int64 acc` (line 19), multiplies by 10 and adds digit values (line 35) with **no overflow detection**, then applies negation if needed (lines 38-40).

**2. Which parameters are written:**
None. `s` is a local pointer walking the input; `nptr` itself is `const char *` and never modified.

**3. NULL/zero-length/not-found:**
- **NULL** → `return 0` (lines 21-22)
- **Empty string / no digits** → `return 0` (loop at line 34 never executes if `*s` is not a digit after optional sign)
- Leading whitespace skipped (lines 24-25)
- Parsing stops at first non-digit (line 34 loop exit)

**4. Deliberate deviation from textbook:**
- **NULL safety:** Line 21-22 `if (nptr == NULL) return 0;` — textbook `atoi` typically segfaults on NULL.
- **Silent wrapping overflow:** Line 35 `acc = 10 * acc + ((int)*s - '0');` with no check. C undefined behavior on signed overflow, but compiled ReactOS behavior wraps. After wrapping, negation is applied (lines 38-40), so `-9223372036854775808` input wraps to `i64::MIN`, then negating `i64::MIN` wraps back to `i64::MIN` (two's complement).
- **No error signaling:** No `errno`, no `*endptr` output. Stops parsing at first non-digit, returns whatever accumulated (which may be 0 if no digits found).

**5. Minimal test domain:**
Must cover:
- **NULL** (explicit guard)
- **Empty / whitespace-only** (acc remains 0)
- **Valid small negative** (sign handling lines 26-31)
- **Valid large near boundary** (i64::MAX = 9223372036854775807)
- **Single-step overflow** (9223372036854775808 wraps to i64::MIN)
- **Multi-step overflow** (e.g., 92233720368547758070 wraps multiple times)
- **Leading zeros** (ensure no octal interpretation)
- **Plus sign** (line 30-31)
- **Trailing garbage** (stops at non-digit)
- **Non-digit prefix** (returns 0)

The core function should be `radix::parse_i64_wrapping` returning `(i64, overflowed: bool)`, but the C ABI **discards the overflow flag** — `return_collapse = "i64_passthrough"` means we ignore the `overflowed` and return only the wrapped `i64`.