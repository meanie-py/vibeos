```toml
[symbols._ui64toa_s]
c_signature = "int _ui64toa_s(unsigned __int64 value, char *str, size_t size, int radix)"
family = "radix"
slice_pattern = "dst_fixed_n"
core_fn = "NEW:radix::format_u64"
return_collapse = "i32_passthrough"
inout_effects = "str[0..min(result_len,size-1)] written with ASCII digits (radix 2-36, lowercase a-z for 10-35), NUL-terminated on success; on EINVAL buffer-too-small, str[0..size-1] filled with reversed partial result then str[0]=NUL (mangled state)"
preserved_misfeature = "On buffer-too-small (len > size), fills str backwards from pos=buffer+31 for i<size iterations, then sets str[0]=NUL, returning EINVAL (not ERANGE as _i64toa_s does). Line 'if((unsigned)(buffer-pos+65) > size)' uses wrong constant 65 instead of 64, making the size check off-by-one compared to _i64toa_s which correctly uses 'buffer+65-pos'. Returns EINVAL (not ERANGE) on overflow per line 'MSVCRT_INVALID_PMT(\"str[size] is too small\", EINVAL)'."
fuzz_domain = ["radix={2,10,16,36}", "value={0,1,max_u64}", "size={0,1,exact_needed,exact_needed-1,exact_needed+1}", "radix={1,37}=EINVAL", "str=NULL=EINVAL"]
```

## Rationale

**Exact return value** (lines 130-160):
- Returns `0` on success after writing NUL-terminated ASCII representation
- Returns `EINVAL` immediately if `str==NULL`, `size==0`, or `radix<2` or `radix>36` (lines 135-139)
- Returns `EINVAL` (NOT `ERANGE`) when `(unsigned)(buffer-pos+65) > size` (lines 156-158), indicating insufficient buffer

**Parameters written** (lines 143-155, 156-158):
- On success: `str[0..len-1]` receives the NUL-terminated string representation, where `len = buffer-pos+65` (the +65 accounts for `buffer[64]` being the NUL position, so `buffer-pos+65` is the distance from `pos` to one past the NUL)
- On EINVAL failure with buffer-too-small: the function does NOT perform the backwards partial copy that `_i64toa_s` does; it immediately returns EINVAL at line 158

**NULL/empty behavior** (lines 135-139):
- `str==NULL` → return `EINVAL` via `MSVCRT_CHECK_PMT`
- `size==0` → return `EINVAL`
- Invalid radix → return `EINVAL`
- No errno set in `_LIBCNT_` mode

**Deliberate deviation from textbook AND from sibling `_i64toa_s`**:
1. **Off-by-one size calculation** (line 156): Uses `(unsigned)(buffer-pos+65) > size` instead of the correct `buffer-pos+64` or equivalent. Since `pos` points into `buffer[0..64]` and we write backwards from `buffer[64]`, the length should be `&buffer[64] - pos + 1` (as in `_ui64toa` line 122) or equivalently `buffer + 65 - pos` (as in `_i64toa_s` line 82). This implementation uses `buffer-pos+65` which is incorrect pointer arithmetic (should be `buffer+65-pos`), making the check unreliable.

2. **Returns EINVAL instead of ERANGE** on buffer overflow (line 158): While `_i64toa_s` returns `ERANGE` for buffer-too-small (line 98), `_ui64toa_s` returns `EINVAL`. The `MSVCRT_INVALID_PMT` call explicitly passes `EINVAL` as the error code.

3. **No backwards partial copy on overflow**: Unlike `_i64toa_s` (lines 87-96) which fills the buffer backwards with as much as fits before returning ERANGE, `_ui64toa_s` immediately returns EINVAL without that fallback behavior.

**Core needed**: `NEW:radix::format_u64` with signature `fn format_u64(val: u64, radix: u32) -> Result<ArrayString<65>, RadixError>` where `RadixError::InvalidRadix` for radix ∉ [2,36]. Returns fixed-capacity string (never panics on valid radix).

**Fuzz equivalence classes**:
- Radix boundaries: 2, 10, 16, 36 (max), plus invalid 1, 37
- Values: 0 (single digit), 1 (single digit), `u64::MAX` (longest possible)
- Size: 0 (immediate EINVAL), 1 (only room for NUL), exact needed length, one less (triggers overflow), one more (success margin)
- NULL pointer to catch the PMT check