```toml
[symbols._ui64toa]
c_signature = "char *_ui64toa(unsigned __int64 value, char *string, int radix)"
family = "radix"
slice_pattern = "none"
core_fn = "NEW:format_u64"
return_collapse = "ret_dst"
inout_effects = "Writes NUL-terminated radix-formatted string to `string` (2..36, lowercase a-z for 10+). No length checks — caller must ensure sufficient space (≤65 bytes for radix=2). Returns the same `string` pointer passed in."
preserved_misfeature = "No validation of radix or dst size. Unconditionally writes to `string` without bounds checking. Radix < 2 or > 36 causes undefined behavior (divide-by-zero or wrong chars). Caller responsible for 65-byte buffer minimum (radix=2 worst case: 64 digits + NUL)."
fuzz_domain = ["value=0", "value=max_u64", "radix=2 (longest output)", "radix=10", "radix=16", "radix=36 (uses 'z')", "small value with high radix"]
```

## Rationale

**1. Exact return value as a function of inputs:**
Lines 91-107 show the core logic. The function converts an unsigned 64-bit value to its string representation in the given radix. It builds the string backwards in a local 65-byte buffer (`buffer[64]` is initially `'\0'`, then digits are placed at `*--pos`). The loop (lines 97-104) repeatedly extracts `digit = value % radix`, divides `value /= radix`, and writes either `'0'+digit` (for 0-9) or `'a'+digit-10` (for 10+). Once `value == 0`, the loop exits. Then `memcpy(string, pos, &buffer[64] - pos + 1)` (line 106) copies the result including NUL to the caller's `string` buffer, and returns `string` (line 107).

**2. Which parameters are written:**
Only `string` is written. The function writes the NUL-terminated radix representation of `value` into `string`, using lowercase letters `a-z` for digits 10-35. No other output parameters exist. The write length is variable (1 to 65 bytes depending on value and radix), always NUL-terminated. No truncation or padding occurs.

**3. NULL / zero-length / not-found behavior:**
- If `string` is NULL, the `memcpy` at line 106 will crash (undefined behavior).
- If `value == 0`, the do-while loop (lines 97-104) executes exactly once, writing `'0'` and terminating (correct).
- If `radix < 2`, the modulo and division are undefined (likely divide-by-zero crash).
- If `radix > 36`, digits ≥36 produce characters beyond `'z'`, which is nonsensical but not checked.

There are NO guards for `string == NULL`, `radix < 2`, or `radix > 36`. Compare to `_ui64toa_s` (lines 113-141), which validates these conditions and returns `EINVAL`. The non-`_s` variant has zero safety checks.

**4. Deliberate deviation from textbook libc:**
Standard `itoa`-family functions are not in ISO C (they're Microsoft/POSIX extensions), but conventionally they validate radix in [2, 36]. **This implementation has NO validation** (lines 91-107 contain no checks). This matches the ReactOS Wine-derived pattern: the non-`_s` variants are unsafe, while `_s` variants validate. The lack of bounds checking on `string` is the primary misfeature: the caller must know the worst-case length (65 bytes for radix=2: 64 bits + NUL) and provide it. This is load-bearing for performance in kernel-mode code where validation overhead is avoided.

**5. Minimal input equivalence-classes:**
- **value=0**: Single-digit output `"0"`.
- **value=UINT64_MAX**: Maximum length for any radix (64 chars for radix=2).
- **radix=2**: Longest possible output (tests buffer boundary).
- **radix=10**: Common case, all decimal digits.
- **radix=16**: Hex output with `a-f`.
- **radix=36**: Uses full `a-z` range (tests `'a'+digit-10` for digit=35 → `'z'`).
- **Small value with high radix** (e.g., value=10, radix=36): Ensures single-letter digit logic (`'a'`) works.

**NEW core:** `format_u64(value: u64, radix: u32) -> Result<ArrayVec<u8, 65>, ()>` (or similar fixed-size buffer return). The shim would then unsafely `memcpy` the result to the C `string` pointer without validation, preserving the misfeature. The core validates radix in [2, 36] internally, returning `Err(())` for invalid radix — but the shim for `_ui64toa` would skip the check and let UB happen (or panic), while `_ui64toa_s` would return `EINVAL`. Alternatively, the core could be `format_u64_unchecked` that panics on bad radix, and the shim just calls it.