```toml
[symbols.atol]
c_signature = "long atol(const char *str)"
family = "radix"
slice_pattern = "nul_in"
core_fn = "radix::parse_i64_wrapping"
return_collapse = "i64_trunc_i32"
inout_effects = "none (read-only)"
preserved_misfeature = "Wrapping overflow without error: accumulates digit-by-digit in i64 with wrapping arithmetic (acc = 10 * acc + digit), then truncates to long (int32). Source: _atoi64 lines 'acc = 10 * acc + ((int)*s - '0');' with no overflow check, then atol casts '(long)_atoi64(str)'. Textbook atol would detect/clamp overflow."
fuzz_domain = ["NULL", "empty_string", "whitespace_only", "+123", "-456", "overflow_pos_i64", "overflow_neg_i64", "overflow_then_trunc_to_i32", "non_digit_after_valid", "leading_zeros", "max_i32", "min_i32"]
```

## Rationale

**1. Exact return value:**
`atol` delegates to `_atoi64` (line `return (long)_atoi64(str);`), which:
- Returns 0 for NULL (line `if (nptr == NULL) return 0;`)
- Skips leading whitespace (line `while(isspace((int)*s))`)
- Parses optional sign `+` or `-` (lines checking `*s == '-'` and `*s == '+'`)
- Accumulates digits with wrapping i64 arithmetic: `acc = 10 * acc + ((int)*s - '0');` (no overflow detection)
- Applies negation if negative (line `if (neg) acc *= -1;`)
- `atol` then truncates i64→long (i32 on ReactOS)

**2. Written parameters:** None. All params read-only.

**3. NULL/zero-length behavior:** 
- NULL: explicit guard returns 0 (line `if (nptr == NULL) return 0;`)
- Empty or whitespace-only: stops at `isdigit` check, returns `acc=0`

**4. Deviation from textbook libc:**
The source performs **wrapping i64 accumulation** (line `acc = 10 * acc + ((int)*s - '0');`) with no overflow detection, then **truncates to long**. Standard `atol` typically clamps to `LONG_MIN`/`LONG_MAX` or signals error on overflow. ReactOS silently wraps in i64 space, then truncates. This matches the `atoi` misfeature already documented.

**5. Fuzz domain (input equivalence classes):**
- NULL (returns 0)
- Empty string (returns 0)
- Whitespace-only (returns 0)
- Valid positives with/without `+` sign
- Valid negatives with `-` sign
- Overflow boundaries: values that wrap i64, values that fit i64 but overflow i32 after truncation
- Non-digit after valid parse (stops at first non-digit)
- Leading zeros (accumulated correctly)
- `INT32_MAX`, `INT32_MIN` boundaries to verify truncation behavior