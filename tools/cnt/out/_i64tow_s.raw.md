```toml
[symbols._i64tow_s]
c_signature = "int _i64tow_s(__int64 value, wchar_t *str, size_t size, int radix)"
family = "radix"
slice_pattern = "dst_fixed_n"
core_fn = "NEW:format_i64_radix"
return_collapse = "errno_code"
inout_effects = "str[0..len] written with NUL-terminated radix string on success (0..65 wchars); str[0..size] receives partial backward fill then str[0]=NUL on ERANGE; str[0]=NUL on EINVAL"
preserved_misfeature = "ERANGE buffer-overflow: copies digits BACKWARDS from end of output, skipping negative sign if present (lines 84-93), fills up to size wchars before setting str[0]=NUL and returning ERANGE. This is non-standard partial-fill behavior distinct from truncating the full output."
fuzz_domain = ["null_str", "size_0", "radix_out_of_range_2_36", "negative_radix10", "negative_non_radix10", "positive_min_max", "size_equals_needed", "size_one_less_than_needed", "radix_2_10_16_36"]
```

## Rationale

**Exact return behavior** (lines 49-55, 81-95, 97):
- Returns `0` on success after writing NUL-terminated radix string to `str`.
- Returns `EINVAL` if `str==NULL`, `size==0`, `radix<2`, or `radix>36` (lines 49-55). On EINVAL, sets `str[0]='\0'` if `str!=NULL && size>0` (line 52).
- Returns `ERANGE` if computed output length `len` (including NUL) exceeds `size` (line 81). Before returning, performs partial backward copy (lines 84-93) then sets `str[0]='\0'` (line 94).

**Which parameters written** (lines 52, 84-94, 97):
- On success: `str[0..len-1]` contains radix digits (possibly prefixed with `-`), `str[len-1]` is `'\0'`. The output is built backwards in a stack buffer then memcpyd in one shot (line 97).
- On ERANGE: First, if the output would have had a negative sign (`is_negative`), `str[1..size]` is filled with the rightmost `size-1` digits from the temporary buffer copied backwards from `buffer+63` downward (lines 86-92). If not negative, `str[0..size-1]` is filled. Then `str[0]='\0'` (line 94).
- On EINVAL with valid `str` and `size`: `str[0]='\0'` (line 52).

**NULL/zero-length behavior** (lines 49-55):
- `str==NULL` or `size==0` triggers `EINVAL` return. The guard `MSVCRT_CHECK_PMT` ensures this.
- No dereference if `str==NULL` after the initial check; `str[0]='\0'` is guarded by `if (str && size)` (line 51).

**Deliberate deviation** (lines 84-93):
The ERANGE case does NOT simply truncate the formatted string. Instead:
- Lines 84-93: If `is_negative`, it increments the destination pointer and decrements size (lines 88-89), then copies from the *end* of the temporary buffer (`buffer+63`) *backwards* for `size` characters (lines 91-92). This fills `str` with the least-significant digits in reverse order, omitting the negative sign.
- Line 94: Unconditionally sets `str[0]='\0'`, clobbering the first character of whatever partial data was written.
- This behavior is NOT a standard truncation. It's a peculiar partial fill that surfaces when the buffer is too small.

**Core signature needed**: `NEW:format_i64_radix` with signature:
```
fn format_i64_radix(value: i64, radix: u32, is_radix10_signed: bool) 
   -> Result<ArrayString<65>, FormatError>
```
Returns the NUL-terminated radix string (max 65 wchars: 64 digits + sign for `-2^63` in binary + NUL). The shim then checks length vs. `size`, copies on success, or performs the backward partial-fill dance on `ERANGE`.

**Fuzz domain**:
- `null_str`, `size_0`: trigger EINVAL guard.
- `radix_out_of_range_2_36`: radix=1, radix=37 → EINVAL.
- `negative_radix10`: e.g., `value=-123, radix=10` → negative sign handling (lines 58-67).
- `negative_non_radix10`: e.g., `value=-1, radix=16` → treated unsigned (line 64: `val=value` when `radix!=10`).
- `positive_min_max`: `INT64_MAX` in various radices to test digit loop.
- `size_equals_needed`, `size_one_less_than_needed`: exercise the `len>size` boundary (line 81) and the ERANGE partial-fill logic.
- `radix_2_10_16_36`: exercise digit<10 vs. `'a'+digit-10` branches (lines 75-78), min/max radix.