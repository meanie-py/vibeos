```toml
[symbols._ltoa_s]
c_signature = "int _ltoa_s(long value, char *str, size_t size, int radix)"
family = "radix"
slice_pattern = "dst_fixed_n"
core_fn = "NEW:radix::i64_to_str"
return_collapse = "i32_passthrough"
inout_effects = "str[0..len] written with NUL-terminated radix string; on ERANGE, str partially filled backwards from end (skipping sign if present), then str[0]=NUL"
preserved_misfeature = "On overflow (len>size), writes backwards from buffer end up to size-1 chars (excluding sign if negative), then sets str[0]=NUL and returns ERANGE. This creates a *right-aligned truncated* string with leading NUL, unlike typical left-truncation. Lines 217-227: skip sign position, copy backwards from buffer[31] for size chars, then str[0]=NUL."
fuzz_domain = ["negative radix-10 (value<0, radix==10)", "positive/zero radix-10", "non-decimal radix (2-9,11-36)", "exact fit (len==size)", "overflow by 1 (len==size+1)", "overflow by many", "radix boundaries (1,2,36,37)", "size==1 (only room for NUL)", "value edge cases (LONG_MIN, LONG_MAX, 0)"]
```

## Rationale

**1. Exact return value:**
Lines 193-202: Parameter validation returns `EINVAL` if `str==NULL`, `size==0`, `radix<2`, or `radix>36`. Lines 214-228: Returns `ERANGE` if the computed result length (including NUL) exceeds `size`. Line 230: Returns `0` on success. The function never fails for arithmetic reasons—it converts `long` to `unsigned long` for processing.

**2. Parameters written:**
Line 200: On validation failure with valid `str` and `size`, writes `str[0]='\0'`. Lines 203-215: On success, constructs digits backwards into stack buffer, prepending '-' if `is_negative`. Lines 214-227: On overflow, writes up to `size-1` characters *backwards from buffer end* (skipping the sign position if negative), then sets `str[0]='\0'`. Line 230: On success, `memcpy(str, pos, len)` writes the complete NUL-terminated result.

**3. NULL/zero-length/not-found behavior:**
Lines 193-202: `!MSVCRT_CHECK_PMT(str != NULL)` and `!MSVCRT_CHECK_PMT(size > 0)` trigger validation failure, returning `EINVAL` and writing `str[0]='\0'` if `str && size`. No dereference crash; the guards are explicit.

**4. Deliberate deviation from textbook libc:**
The **overflow handling is the critical misfeature**. Lines 217-227 show that when `len > size`, instead of left-truncating or simply failing, the code:
- Increments `p` (output pointer) and decrements `size` if `is_negative`, effectively reserving space for a sign that *won't be written*.
- Copies from `buffer + 31` (one before the NUL) *backwards* for `i < size` iterations, writing `*p++ = *pos--`.
- Sets `str[0] = '\0'`.

This creates a string like `"\0...789"` if the result was `"-123456789"` and only 4 bytes were available—right-aligned truncation with a leading NUL. Textbook `ltoa_s` would typically left-truncate or return the sign followed by as many leading digits as fit. The comment on lines 218-219 explicitly states *"Don't copy the negative sign if present"*, but the logic also results in this bizarre right-aligned, NUL-prefixed behavior.

Additionally, line 200: setting `str[0]='\0'` on validation failure before returning `EINVAL` is a msvcrt-ism (defensive programming to avoid uninitialized reads), not mandated by C standard, though common in "secure" CRT variants.

**5. Minimal fuzz domain:**
- **Negative radix-10 vs positive/zero**: Lines 203-212 branch on `value < 0 && radix == 10` to set `is_negative` and negate. Must test both paths.
- **Non-decimal radix**: The negation only applies for radix 10; other radices treat `long` bits as unsigned. Test radix 2, 16, etc.
- **Exact fit vs overflow**: Lines 214-228 branch on `len > size`. Test `len==size` (success) and `len==size+1` (overflow by 1).
- **Overflow by many**: Ensures the backwards-copy loop (lines 224-225) is exercised for multiple iterations.
- **Radix boundaries**: radix==1, 2, 36, 37 to test validation (lines 194-195).
- **size==1**: Only room for NUL; triggers overflow for any non-empty result (tests line 214 boundary).
- **Value edge cases**: `LONG_MIN` (most negative, tests negation on line 206), `LONG_MAX`, and `0` (minimal output).
- **is_negative interaction with overflow**: The sign-skipping logic (lines 219-221) must be tested with a negative value that overflows.

**Core function needed:**
`NEW:radix::i64_to_str` with signature `fn i64_to_str(value: i64, radix: u32, is_signed: bool, out: &mut [u8]) -> Result<usize, RadixError>` where `RadixError` variants include `InvalidRadix`, `BufferTooSmall`. The core should:
- Validate `2 <= radix <= 36`.
- Handle signed negation only for radix 10 when `is_signed && value < 0`.
- Write lowercase a-z for digits 10-35.
- Return the number of bytes written (including NUL) on success.
- Return `BufferTooSmall` if output doesn't fit, WITHOUT performing the bizarre right-aligned partial write—that collapse is the shim's job to preserve the misfeature.

The shim will then implement the lines 217-227 logic: on `BufferTooSmall`, reconstruct the truncated, backwards, NUL-prefixed output as ReactOS does.