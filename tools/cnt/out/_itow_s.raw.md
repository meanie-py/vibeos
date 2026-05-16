```toml
[symbols._itow_s]
c_signature = "int _itow_s(int value, wchar_t *str, size_t size, int radix)"
family = "radix"
slice_pattern = "dst_fixed_n"
core_fn = "NEW:radix::format_i32_wrapping"
return_collapse = "errno_or_zero"
inout_effects = "str[0..min(result_len,size)] written; on error: str[0]='\\0' if str!=NULL && size>0; on ERANGE truncation: str[0]='\\0' AND partial backwards fill from buffer[31] for size-1 chars (skipping sign if negative)"
preserved_misfeature = "Delegates to _ltow_s which casts int→long. On ERANGE (len>size), performs bizarre partial backwards copy: skips negative sign, copies from buffer[31] backwards for min(size-1, available) characters, then zeroes str[0]. Lines 288-302 in _ltow_s show this non-standard truncation+backfill behavior instead of simple ERANGE-without-copy."
fuzz_domain = ["value={INT_MIN,INT_MIN+1,-1,0,1,INT_MAX}", "radix={1,2,10,16,36,37}", "size={0,1,2,33,34}", "str={NULL,valid}", "negative radix=10 (triggers sign)", "positive radix=10", "radix=16 (no sign)", "len==size (exact fit)", "len==size+1 (ERANGE boundary)", "size==1 (only room for NUL)"]
```

## Rationale

**Delegation chain**: `_itow_s` (line 175-178) directly delegates to `_ltow_s` after casting `int value` to `long`. The actual implementation is in `_ltow_s` (lines 204-303).

**Q1. Return value**: Returns `0` on success, `EINVAL` for invalid parameters (str==NULL, size==0, radix<2, radix>36), `ERANGE` when the formatted length exceeds `size` (line 285: `if (len > size)`). The `len` calculation (line 282: `len = buffer + 33 - pos`) includes the NUL terminator.

**Q2. Which parameters written**:
- **Success path** (lines 301-302): `memcpy(str, pos, len * sizeof(wchar_t))` writes the entire formatted string including NUL.
- **EINVAL path** (lines 212-213): If validation fails AND `str && size`, writes `str[0] = '\0'`.
- **ERANGE path** (lines 287-300): This is the bizarre misfeature. When `len > size`, it:
  1. If negative sign present, advances `p++` and decrements `size--` (lines 292-295), effectively skipping the sign in output.
  2. Loops from `buffer[31]` (the last digit before NUL) **backwards** for `size` characters, writing `*p++ = *pos--` (lines 297-298).
  3. Then calls `MSVCRT_INVALID_PMT` and sets `str[0] = '\0'` (lines 300-301), **overwriting** the partial backward-filled digits.

**Q3. NULL/zero-length behavior**:
- Lines 207-215: `MSVCRT_CHECK_PMT(str != NULL)` and `MSVCRT_CHECK_PMT(size > 0)` return `EINVAL` immediately if violated. These checks prevent NULL deref.
- Line 212: On validation failure, `if (str && size) str[0] = '\0'` — only writes if both non-null AND non-zero.

**Q4. Deviation from textbook libc**: 
- **ERANGE partial backfill** (lines 287-298): Standard `_itow_s` would leave `str` unmodified or write nothing on ERANGE. ReactOS writes **backwards** from the least-significant digit, omits the sign, writes up to `size-1` chars, then **zeros `str[0]`** anyway. This creates a corrupted buffer with digits in reverse order followed by NUL overwriting the first position.
- **Signed→unsigned cast**: Line 218-225 handles negative values only for `radix==10`. For other radices, negative `int` is cast to `unsigned long` (line 224: `val = value`), producing two's-complement unsigned representation (e.g., `-1` in base 16 becomes `"ffffffff"`).
- **_LIBCNT_ errno suppression**: Lines 215, 300 show `#ifndef _LIBCNT_` guards around `*_errno()` assignment — libcntpr builds omit errno side effects but still return the error code.

**Q5. Fuzzing equivalence classes**:
1. **Validation boundaries**: `str==NULL`, `size==0`, `radix==1`, `radix==37` to trigger each `EINVAL` guard.
2. **Sign handling**: Negative values with `radix==10` vs other radices to test lines 218-225 (two's-complement unsigned cast).
3. **Length boundaries**: `len==size` (exact fit), `len==size+1` (ERANGE by 1), `size==1` (only NUL fits).
4. **ERANGE partial-fill**: `value==INT_MIN, radix==10, size==3` to trigger the backwards copy logic with negative sign.
5. **Extremes**: `INT_MIN`, `INT_MAX`, `radix==36` (longest possible output for given value).
6. **Zero**: `value==0` always produces `"0"` regardless of radix.

**Core needed**: `NEW:radix::format_i32_wrapping` should take `(i32, radix) → (&str, is_negative)` or similar, handling the cast-to-unsigned-then-format logic for non-base-10 negative values. The shim must then orchestrate the buffer-filling, ERANGE partial backwards copy, and NUL-on-error semantics. Alternatively, reuse `radix::format_i64_wrapping` after widening `i32→i64`, but the misfeature's truncation/backfill must live in the shim.