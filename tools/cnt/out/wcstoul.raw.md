```toml
[symbols.wcstoul]
c_signature = "unsigned long wcstoul(const wchar_t *nptr, wchar_t **endptr, int base)"
family = "radix"
slice_pattern = "nul_in"
core_fn = "NEW:radix::parse_ulong_wide"
return_collapse = "ulong_passthrough"
inout_effects = "if endptr!=NULL: writes *endptr = first-invalid-wchar ptr (or nptr if no digits parsed)"
preserved_misfeature = "Lines 45-47,55-56: Overflow saturates to ULONG_MAX but still advances *endptr past ALL valid digits (any=-1 sticky). Line 57: Negation applied AFTER saturation: neg overflow yields 0-(ULONG_MAX)=1, not ULONG_MAX. Lines 26-27: Leading '-' parses, sets neg=1, then negates the accumulated unsigned result at the end."
fuzz_domain = ["empty", "whitespace-only", "no-digits (base-prefix only)", "valid digits", "overflow-saturate", "negative (wrapping-subtract)", "base 0/8/10/16", "0x-prefix", "mixed case digits", "endptr==NULL"]
```

## Rationale

**1. Return value as function of inputs:**

The function parses a wide-character string to `unsigned long` in the specified base (lines 49-54 accumulate). Lines 26-31: skip leading whitespace, parse optional '+'/'-' sign. Lines 32-38: auto-detect base from '0x'/'0' prefix if base==0. Lines 55-56: On overflow (`any < 0`), saturates to `ULONG_MAX`. Line 57: If `neg` was set (leading '-'), applies wrapping subtraction `0 - acc`, which for `ULONG_MAX` yields `1` on typical 32-bit (NOT clamped to 0 or kept at `ULONG_MAX`). Lines 46-47: Overflow detection uses `cutoff`/`cutlim` but once `any=-1` is set (overflow occurred), continues parsing digits without further accumulation. Line 51: loop breaks on non-digit or digit >= base.

**2. Parameters written:**

Lines 59-60: If `endptr != 0`, writes `*endptr` to point at the first invalid wide character (one past the last valid digit), or to the original `nptr` if no digits were successfully parsed (`any==0` after the loop). The pointer arithmetic `s - 1` accounts for the loop's post-increment.

**3. NULL / zero-length / not-found:**

No explicit NULL guard on `nptr` — derefs immediately (line 16 initializes `s = nptr`, line 23 deref `c = *s++`). Zero-length or all-whitespace → `any=0` → `*endptr = nptr` (line 60). Not-found digits (e.g., "0x" with no hex digits) → `any=0` as well.

**4. Deliberate deviation from textbook:**

Lines 55-57: **Overflow returns `ULONG_MAX`, but if the input had a leading '-', negation is applied AFTER saturation**, so `wcstoul(L"-9999999999", &end, 10)` on overflow yields `0 - ULONG_MAX = 1` (wrapping subtraction), not `0` or `ULONG_MAX`. This is the most critical misfeature. Additionally, lines 46-47 set `any = -1` on overflow but **continue parsing**, so `*endptr` advances past all valid digits even though accumulation stopped.

**5. Minimal input equivalence-classes:**

- Empty string `L""` (c==0 immediately, `any=0`, `*endptr=nptr`)
- Whitespace-only `L"   "` (loop exits at NUL, `any=0`)
- Sign-only `L"-"` or `L"+"` (no digits follow, `any=0`)
- Base-prefix-only `L"0x"` (base 0 or 16, c==0 or non-hex after 'x', `any=0`)
- Valid single-digit `L"5"` (any=1, acc=5)
- Valid multi-digit `L"123"` (accumulation, no overflow)
- Overflow `L"4294967296"` (>ULONG_MAX on 32-bit, saturate, `any=-1`, `*endptr` at end)
- Negative valid `L"-10"` (acc=10, then `acc = 0-10` wraps to `ULONG_MAX - 9`)
- Negative overflow `L"-9999999999"` (saturate to `ULONG_MAX`, then `0 - ULONG_MAX = 1`)
- Base 0 with octal `L"077"` (auto-detect base 8)
- Base 0 with hex `L"0x1F"` (auto-detect base 16)
- Base 16 explicit `L"aB"` (mixed case, line 49 handles _UPPER vs lowercase)
- Trailing invalid `L"123abc"` base 10 (`*endptr` points at 'a')
- `endptr == NULL` (line 59 guard, no crash)

**Core signature needed:**

`NEW:radix::parse_ulong_wide(s: &[u16], base: i32) -> (u32, usize, Overflowed)` — returns `(accumulated_value, count_of_valid_wchars_parsed, overflowed_flag)`. The shim then applies negation-via-wrapping-sub if the sign was '-', sets `*endptr` from the count (or nptr if count==0), and collapses overflow to `ULONG_MAX` before negation if `overflowed`. The core must implement the cutoff logic and sticky `any=-1` overflow with continued digit-skipping to match lines 43-54.