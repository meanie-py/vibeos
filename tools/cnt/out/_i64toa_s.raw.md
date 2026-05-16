```toml
[symbols._i64toa_s]
c_signature = "int _i64toa_s(__int64 value, char *str, size_t size, int radix)"
family = "radix"
slice_pattern = "dst_fixed_n"
core_fn = "NEW:radix::format_i64_bounded"
return_collapse = "i32_passthrough"
inout_effects = "str[0..min(len,size)] written with NUL-terminated decimal/radix representation; on ERANGE, str partially filled backward from end (excluding sign if negative), then str[0]=NUL"
preserved_misfeature = "On buffer overflow (len > size), copies size-1 chars BACKWARD from end of temp buffer (buffer[31] downward, skipping sign), then sets str[0]=NUL (lines 73-84). This leaves a RIGHT-aligned partial suffix without NUL terminator in str[1..size-1], then overwrites str[0] with NUL. Textbook would left-truncate or fail cleanly."
fuzz_domain = ["value={-9223372036854775808,0,42,9223372036854775807}", "radix={2,10,16,36}", "size={1,exact_len,exact_len-1,exact_len+1,65}", "null_str"]
```

## Rationale

**Exact return value** (lines 19-27, 72-84, 86-87): Returns 0 on success. Returns EINVAL if str==NULL, size==0, radix<2, or radix>36 (validation lines 19-20). Returns ERANGE if the formatted string length (including NUL) exceeds size (line 72). The formatted length includes optional '-' sign, all digits, and NUL terminator.

**Parameters written** (lines 50-67, 73-84, 86): `str` is written with the NUL-terminated radix representation of `value`. On success (lines 86), `memcpy(str, pos, len)` copies the complete formatted string including NUL. On ERANGE overflow (lines 73-84), the function performs a bizarre backward-copy: if `is_negative`, it increments `p` and decrements `size` (lines 77-79), then copies `size` characters from `buffer[63]` downward via `*p++ = *pos--` (lines 81-82), and finally sets `str[0] = '\0'` (line 84). This leaves a right-aligned partial result in `str[1..size-1]` with a NUL at `str[0]`.

**NULL/zero-length behavior** (lines 19-27): If `str==NULL` or `size==0`, the function returns EINVAL. If `str!=NULL && size>0`, it sets `str[0]='\0'` before returning EINVAL (lines 23-24). No crash, but no work done.

**Deliberate deviation from textbook** (lines 73-84): The overflow handling is extraordinary. Rather than left-truncating or simply failing, it copies the *rightmost* `size-1` characters of the formatted result backward from the temporary buffer (starting at `buffer[63]`), excluding the sign if present, then unconditionally writes NUL to `str[0]`. This produces a right-aligned suffix in `str[1..size-1]` with `str[0]=='\0'`, effectively discarding both the sign and the most significant digits. Textbook implementations either leave the buffer untouched, write a truncated left-aligned result, or simply return error. This implementation's backward loop (lines 81-82) is the smoking gun.

**Conversion logic** (lines 29-66): For radix 10 and `value < 0`, sets `is_negative=1` and converts to unsigned via `val = -value` (lines 29-37). Otherwise treats as unsigned. Builds the string backward in a 65-byte buffer (pos starts at buffer[64], decrements with `*--pos`). Digits 0-9 use '0'+digit, 10-35 use 'a'+(digit-10) (lines 50-56). Prepends '-' if negative (lines 59-60). Length is `buffer + 65 - pos` (line 62).

**Proposed core signature**: `radix::format_i64_bounded(val: i64, radix: u32, dst: &mut [u8]) -> Result<usize, RadixError>` returning `Ok(len_written_including_nul)` on success, `Err(RadixError::InvalidRadix)` for radix ∉ [2,36], `Err(RadixError::BufferTooSmall)` if formatted length exceeds `dst.len()`. The core does NOT implement the backward-copy misfeature; the shim must.

**Fuzz domain coverage**: Must test minimum/maximum i64 values (two's complement edge), zero, typical positive/negative values; boundary radices (2, 10, 36); size exactly matching, one under, one over, and minimal (1); and NULL str to trigger EINVAL. The ERANGE case (size < formatted length) MUST be tested with both negative and non-negative values to exercise the sign-handling branch (lines 77-79) and verify the backward-copy behavior.