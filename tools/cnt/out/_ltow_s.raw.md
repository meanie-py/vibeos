```toml
[symbols._ltow_s]
c_signature = "int _ltow_s(long value, wchar_t *str, size_t size, int radix)"
family = "radix"
slice_pattern = "dst_fixed_n"
core_fn = "NEW:radix::format_i32_to_wstring"
return_collapse = "i32_passthrough"
inout_effects = "str[] written with NUL-terminated conversion; on EINVAL/ERANGE: str[0]=L'\\0'. On ERANGE if is_negative: partial backwards fill starting str[1], then str[0]=L'\\0'."
preserved_misfeature = "ERANGE truncation writes partial result BACKWARDS from buffer end (lines 273-280): 'for (pos = buffer + 31, i = 0; i < size; i++) *p++ = *pos--;' — fills from right-to-left of the generated string, NOT left-to-right. Then overwrites str[0] with NUL. Negative sign skipped in partial copy but destination adjusted (p++, size--)."
fuzz_domain = ["value={-2147483648,0,42,2147483647}", "radix={1,2,10,16,36,37}", "size={0,1,2,exact_len,exact_len-1,exact_len+1}", "str={NULL,valid}", "negative radix-10 requiring sign with size exactly at boundary"]
```

## Rationale

**1. Exact return value (lines 246-283):**
- Returns `0` on success after `memcpy(str, pos, len * sizeof(wchar_t))` (line 283).
- Returns `EINVAL` if `str==NULL`, `size==0`, or `radix<2` or `radix>36` (lines 246-253).
- Returns `ERANGE` if the generated string length `len` (including NUL) exceeds `size` (lines 266-280), after writing a partial backwards-filled result and setting `str[0]=L'\0'`.

**2. Which parameters written (lines 249, 268-280, 283):**
- `str[]` is always written:
  - On validation failure (`EINVAL`): `str[0]=L'\0'` if `str && size` (lines 249).
  - On `ERANGE`: partial backwards copy from the temporary buffer (lines 273-277), then `str[0]=L'\0'` (line 279).
  - On success: full NUL-terminated wide-string via `memcpy(str, pos, len * sizeof(wchar_t))` (line 283), where `len = buffer + 33 - pos` (line 265) includes the terminator.

**3. NULL/zero-length behavior (lines 246-253):**
- `str==NULL` or `size==0`: validation fails, returns `EINVAL`. If `str && size`, sets `str[0]=L'\0'` before returning (line 249).
- No dereference crash; guarded by `MSVCRT_CHECK_PMT` macros.

**4. Deliberate deviation from textbook (lines 268-280):**
- **ERANGE partial-write is BACKWARDS**: when `len > size`, the code does NOT copy the leftmost `size-1` characters. Instead, it copies from `buffer + 31` (the last non-NUL character) BACKWARDS for `size` iterations (lines 275-277): `for (pos = buffer + 31, i = 0; i < size; i++) *p++ = *pos--;`. This fills `str` with the *rightmost* digits of the conversion, in reverse order from the temporary buffer's perspective.
- After the partial copy, `str[0] = L'\0'` (line 279), obliterating even that partial result's first character.
- If `is_negative`, the destination pointer is adjusted (`p++; size--;` lines 271-274) to skip the negative sign slot, but the sign itself is NOT included in the partial copy.

**5. Minimal fuzz domain:**
- **Radix boundaries**: `radix=1` (invalid), `radix=2,10,16,36` (valid edges), `radix=37` (invalid).
- **Value boundaries**: `LONG_MIN` (-2147483648, triggers sign and potential edge in negation: `val = -value` line 259), `0`, positive, `LONG_MAX`.
- **Size boundaries**: `size=0` (invalid), `size=1` (only room for NUL), `size` exactly equal to required `len`, `size = len - 1` (triggers ERANGE partial-backwards), `size = len + 1` (success with room).
- **Negative with tight buffer**: A negative radix-10 conversion where `len` (including sign and NUL) exactly equals or exceeds `size` by 1, to exercise the `is_negative` partial-copy branch (lines 271-274) and the backwards-fill loop (lines 275-277).
- **NULL**: `str=NULL` to verify `EINVAL` and no crash.

**Proposed NEW core (one-line signature):**
`radix::format_i32_to_wstring(value: i32, radix: u32) -> Result<WideString, RadixError>` — converts signed 32-bit int to wide-char string with lowercase hex digits (`'a'-'f'`), handling radix 2..=36 and negative values in base 10 only. The shim must then handle the bizarre ERANGE partial-backwards truncation and the `str[0]=L'\0'` clobber itself, as this is a C-ABI artifact not suitable for the core.