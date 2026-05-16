```toml
[symbols._strtoi64]
c_signature = "__int64 _strtoi64(const char *nptr, char **endptr, int base)"
family = "radix"
slice_pattern = "nul_in"
core_fn = "NEW:radix::parse_i64_clamped"
return_collapse = "i64_passthrough"
inout_effects = "endptr written with (first-unparsed | original-nptr): *endptr = got_digit ? advanced_ptr : original_nptr"
preserved_misfeature = "Overflow clamps to _I64_MAX/_I64_MIN (no wrapping). LIBCNT build silently clamps without errno. endptr written even on empty/invalid input (points to original nptr if no digits parsed). Base validation via MSVCRT_CHECK_PMT but ReactOS __REACTOS__ disables the nptr!=NULL check (lines 20-22), allowing NULL to pass validation then crash on dereference."
fuzz_domain = ["valid_parse:base10", "valid_parse:base16_0x_prefix", "valid_parse:base8_leading0", "valid_parse:base_2_to_36", "overflow_positive_clamp_I64_MAX", "overflow_negative_clamp_I64_MIN", "leading_whitespace_skipped", "sign_plus_minus", "no_digits_endptr_eq_nptr", "partial_parse_endptr_at_first_invalid", "base0_autodetect_oct_dec_hex", "empty_after_sign_endptr_eq_nptr", "invalid_digit_for_base_stops_parse"]
```

## Rationale

**Q1: Exact return value** — Lines 44-77 implement digit-by-digit accumulation in `ret` (initially 0), building `ret = ret*base + v` where `v` is the digit value (negated if `negative`). Overflow is detected (lines 63-73) and clamps to `_I64_MAX` or `_I64_MIN` without wrapping. Returns the clamped value. If no valid digits are parsed (`got_digit==FALSE`), returns 0 (line 15 initialization, never modified if loop doesn't execute).

**Q2: Which parameters written** — Only `endptr` is written (lines 75-76): `*endptr = (char*)(got_digit ? nptr : p)`. If at least one digit was parsed (`got_digit==TRUE`), points to the first character that stopped the parse (after the loop exits at line 77). If no digits parsed, points to the original `p` (saved at line 13 before whitespace skip). Written unconditionally if `endptr!=NULL` (line 75 guard), even on empty/invalid input.

**Q3: NULL/zero-length/not-found behavior** — NULL handling: Lines 20-22 show `#ifndef __REACTOS__` around the `MSVCRT_CHECK_PMT(nptr != NULL)`, meaning ReactOS **disables** the NULL check in the `__REACTOS__` build. This allows NULL to pass validation, then crash on `isspace(*nptr)` at line 26. Zero-length/not-found: If `*nptr=='\0'` after whitespace skip (line 26), the while loop (line 49) never executes, `got_digit` remains FALSE, `endptr` gets original `p`, returns 0. If base is invalid (not 0, not in [2,36]), `MSVCRT_CHECK_PMT` returns 0 early (lines 24-25).

**Q4: Deliberate deviation** — 
1. **Overflow clamping not wrapping** (lines 63-73): Textbook `strtoll` sets `errno=ERANGE` and returns `LLONG_MAX/MIN`, but the clamping logic here is explicit saturation arithmetic. The `_LIBCNT_` build (lines 66, 71) **silently clamps without setting errno**, whereas non-LIBCNT sets `*_errno()=ERANGE`. This is a misfeature: libcntpr gives no indication of overflow.
2. **ReactOS NULL tolerance** (lines 20-22): The `__REACTOS__` build **disables** the `nptr!=NULL` validation that Wine has, creating a NULL-deref crash path rather than returning 0 safely.
3. **endptr always written on success** (line 75-76): Even if no digits parsed, `endptr` is written (pointing to original `nptr`). Some implementations leave `endptr` unmodified on parse failure.

**Q5: Minimal fuzz domain** — Must cover:
- **Base variants**: base 10 (default), base 16 with `0x` prefix (lines 33-36), base 8 with leading `0` (lines 38-40), base 0 autodetect (lines 38-41), explicit bases 2-36 (line 25 validation).
- **Overflow boundaries**: positive overflow → `_I64_MAX` (line 64), negative overflow → `_I64_MIN` (line 69). Critical to test the silent clamping in LIBCNT.
- **Sign handling**: `+` (line 31), `-` (lines 28-30, sets `negative`, negates `v` at line 56).
- **Whitespace**: leading whitespace skipped (line 26).
- **Partial parse**: valid digits followed by invalid character (loop breaks at line 52 or 57, `endptr` points to first invalid).
- **No digits**: empty string after optional sign (e.g., `"+"`, `"-"`, `""`) → `got_digit==FALSE`, `endptr` points to original `p`, return 0.
- **Base validation**: base<2, base>36 (lines 24-25 early return 0).

**New core needed**: `NEW:radix::parse_i64_clamped` with signature `fn parse_i64_clamped(s: &[u8], base: u32) -> (i64, usize, bool_overflowed)` — returns `(value_clamped_to_I64_MIN_I64_MAX, bytes_consumed, did_overflow)`. The shim must handle leading whitespace skip (line 26), sign extraction (lines 28-31), base autodetection (lines 33-41), then call core on the remaining slice. The `endptr` calculation (lines 75-76) uses `bytes_consumed>0` to determine `got_digit`. The shim must reconstruct `*endptr = nptr_start + whitespace_len + sign_len + prefix_len + bytes_consumed` if digits parsed, else `original_nptr`.